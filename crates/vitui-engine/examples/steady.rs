//! **Report #25, measured rather than derived: 60 frames a second, and what it costs a core.**
//!
//! Register entry #25 is *60 fps steady state under 5% of a core*, and until impl 26 it was
//! arithmetic: one animated frame's cost from `examples/budget.rs`, times sixty, over one second of
//! one core. That number was **0.0029%**, and the reason it cannot be trusted is not that it is
//! small — it is that every term in it is app-thread CPU on the fastest scene on the list, and the
//! quantity the register names is a *process*. Missing from the arithmetic, and present here:
//!
//! - the render thread, which composites nothing on a caret and still wakes sixty times a second;
//! - the serializer and the write, which are on that thread and not in the app thread's share;
//! - sixty condvar round trips a second, which is scheduler time charged to this process;
//! - and the timer itself.
//!
//! So this is a binary and `scripts/steady-report.sh` is what runs it, for the same reason
//! `examples/idle.rs` is one: a test would measure libtest and `cargo run` would measure cargo.
//!
//! ```text
//! cargo build --release --example steady -p vitui-engine
//! scripts/steady-report.sh 10
//! ```
//!
//! # Why this is a report with a gate underneath it, and not one or the other
//!
//! §14's rule is that a timing is a report and a gate only at cliff granularity. A CPU *percentage*
//! is a timing, so it is reported. But the sentence *60 fps steady state costs 5% of a core* has a
//! precondition — that sixty frames a second actually happened — and **that is a count**. A process
//! that painted four frames would report a beautiful percentage.
//!
//! So the count is asserted here and the percentage is printed by the script. That is the register's
//! own shape applied to its own entry, and it is the same split entry #17 makes: a unit test counts
//! the wakeups, and a shell script reads `/usr/bin/time`.
//!
//! # What is being animated, and why it is the caret
//!
//! The caret, because entry #25 says *steady state* and the caret is the cheapest thing that is
//! genuinely a state change every frame. The point of the number is the **fixed cost of running at
//! 60 Hz at all** — three threads, a clock, a mailbox and a wire — and any richer scene would bury
//! that under its own drawing. A scene that painted a full screen sixty times a second would be
//! measuring §13's 1 ms budget for the second time and would tell nobody what an idling-but-animating
//! application costs.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use vitui_engine::{Clock, Config, Cursor, CursorShape, Engine, Output, Rect, Style, Wake};

/// How long to hold the steady state for when nobody says. Ten seconds: long enough that the
/// process's own start-up is a rounding error in the percentage, short enough to sit in a CI job
/// beside four other gates.
const DEFAULT_SECONDS: u64 = 10;

/// The rate the entry is about — the rate the *application* animates at, by registering a deadline
/// a sixtieth of a second out.
const HZ: u32 = 60;

/// `Config::max_frame_rate`, and it is **twice** the animation rate rather than equal to it.
///
/// **This is a finding rather than a tuning, and it cost the first run of this probe.** With the
/// ceiling at 60 and deadlines at 60, the run painted **441 frames of a nominal 601 — 73%** — and
/// nothing was wrong with either mechanism. `max_frame_rate` is a *minimum gap measured from the
/// last present* (ADR 0004), and a deadline placed exactly one interval after the last deadline is
/// always a hair earlier than one interval after the last *present*, which happened microseconds
/// later. So `present` is refused, the frame coalesces, and the next deadline is already in the
/// past: the loop loses a frame and then chases the schedule it has fallen behind.
///
/// The engine is doing exactly what ADR 0004 says. What was wrong was the configuration: **a ceiling
/// equal to the rate you intend to animate at is a ceiling you will hit on every frame.** The
/// ceiling exists to stop a runaway producer, and it wants headroom over the rate an application
/// actually asks for — which is a sentence worth having in the repository, because the obvious
/// configuration is the broken one and it fails as a quarter of the frames rather than as an error.
///
/// Recorded in impl ticket 26 and not made a gate: it is a property of a configuration a caller
/// chooses, not of the engine.
const CEILING_HZ: u32 = 2 * HZ;

/// How far below the nominal frame count the run may land before the *precondition* has failed.
///
/// Eighty per cent, and the width is deliberate: this has to survive a shared runner with six
/// concurrent slots, where a sixtieth of a second is not a promise anybody can keep. What it is
/// guarding against is not jitter but a collapse — a process that painted a tenth of the frames and
/// reported a tenth of the CPU, which is the one way this report can lie. Provenance: impl 26,
/// Apple M1 Max, macOS 26.5.2 — 600 of 600 frames locally, and the band exists for the container.
const MIN_FRACTION_OF_NOMINAL: f64 = 0.80;

/// A sink that counts, so that *frames reached the wire* is a fact rather than an assumption.
struct Counting(Arc<AtomicUsize>);

impl std::io::Write for Counting {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.fetch_add(buf.len(), Ordering::Relaxed);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn main() {
    let seconds = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(DEFAULT_SECONDS);
    let window = Duration::from_secs(seconds);

    let bytes = Arc::new(AtomicUsize::new(0));
    let (mut screen, _wake) = Engine::new(Config {
        // The real three-thread path. `Clock::Manual` would put the serializer on this thread and
        // make the number smaller by measuring less of the engine, which is the wrong direction for
        // a figure whose whole purpose is to be an upper bound on what a process costs.
        clock: Clock::System,
        max_frame_rate: CEILING_HZ as f32,
        output: Output::Sink(Box::new(Counting(Arc::clone(&bytes)))),
        size: (300, 80),
        ..Default::default()
    })
    .attach()
    .expect("attaching to a sink cannot fail");

    let layer = screen
        .layers()
        .add_content(0, Rect::new(0, 0, 300, 80), true);
    {
        let mut view = screen.layers().view(layer).expect("the layer just added");
        view.text(0, 0, "steady", Style::new());
    }
    let prologue = bytes.load(Ordering::Relaxed);

    let began = Instant::now();
    let interval = Duration::from_nanos(u64::from(1_000_000_000 / HZ));
    let mut painted: u32 = 0;
    let mut deadlines: u32 = 0;
    let mut tick: u32 = 0;
    loop {
        let elapsed = began.elapsed();
        if elapsed >= window {
            break;
        }
        // **An absolute schedule, and the first draft's relative one was the second half of the same
        // defect.** Deriving the next deadline from the frames *painted* means one coalesced frame
        // moves every deadline after it, so a single miss becomes a permanently slower loop. The
        // tick is wall-clock and does not care what happened to frame `n - 1`.
        tick += 1;
        screen.request_wake_at(began + interval * tick);
        match screen.wait() {
            Wake::Deadline => deadlines += 1,
            Wake::Quit => break,
            // Nothing posts and there is no tty, so neither of these can happen.
            Wake::Input | Wake::Posted => {}
        }
        // The caret, and nothing else. `set_cursor` is applied by `present` after the frame's last
        // write (ADR 0005), so this is one state change a frame that touches no cell at all — which
        // is what makes the number the fixed cost of the loop rather than the cost of drawing. An
        // animation registers a deadline and nothing more; the engine has no animation concept
        // (arch 09), and without the deadline above `wait` would park indefinitely — which is
        // exactly the property `examples/idle.rs` measures from the other side.
        screen.set_cursor(if painted % 2 == 0 {
            Some(Cursor {
                x: 6,
                y: 0,
                shape: CursorShape::Block,
            })
        } else {
            None
        });
        if screen.present().submitted {
            painted += 1;
        }
    }
    let ran = began.elapsed();

    let nominal = f64::from(HZ) * ran.as_secs_f64();
    let floor = nominal * MIN_FRACTION_OF_NOMINAL;
    // **The count, which is the gate.** A percentage measured over four frames is not a measurement
    // of a steady state, and this is the assertion that tells the two apart.
    assert!(
        f64::from(painted) >= floor,
        "painted {painted} frames in {ran:?}, against a nominal {nominal:.0} at {HZ} Hz and a floor \
         of {floor:.0} — this is not a steady state and any CPU figure taken over it is meaningless"
    );
    let written = bytes.load(Ordering::Relaxed);
    assert!(
        written > prologue,
        "not one byte reached the wire after the prologue, so {painted} frames were composed and \
         thrown away"
    );

    println!(
        "steady {ran:.2?} · {painted} frames at a nominal {HZ} Hz ({:.1}% of {nominal:.0}) \
         · {deadlines} deadline wakes · {} bytes after a {prologue}-byte prologue",
        f64::from(painted) / nominal * 100.0,
        written - prologue,
    );
    // Printed for the script to read, which is cheaper and less brittle than the script knowing the
    // shape of the sentence above.
    println!("frames={painted} seconds={:.3}", ran.as_secs_f64());
}
