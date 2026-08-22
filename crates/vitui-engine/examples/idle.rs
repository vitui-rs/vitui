//! Register entry #17's other half: **an idle application costs nothing at all.**
//!
//! The count half is `crate::gates::an_idle_application_parks_once_and_wakes_for_nothing`, and a
//! count does not need thirty seconds to be zero. This is the half that does: `0.00 user 0.00 sys`
//! is a claim about a *process*, `/usr/bin/time` is what measures one, and **at three seconds the
//! difference between a parked application and a 120 Hz ticker is below the tool's resolution.** At
//! thirty it is 3 600 wakeups against zero.
//!
//! So this is a binary rather than a test, and `scripts/idle-gate.sh` is what runs it: a test would
//! measure the libtest harness, and `cargo run` would measure cargo.
//!
//! ```text
//! cargo build --release --example idle -p vitui-engine
//! /usr/bin/time -l ./target/release/examples/idle 30      # macOS
//! /usr/bin/time -v ./target/release/examples/idle 30      # GNU
//! ```
//!
//! # What is idle, and what is not
//!
//! Two of the engine's threads exist here and both are parked: the app thread inside
//! [`Screen::wait`], indefinitely, because no deadline is registered; and the render thread inside
//! the mailbox's condvar, because nothing has been submitted. The input thread is ticket 20's and
//! needs a tty, which a headless run does not have.
//!
//! The third thread is this file's own watchdog, and it is the honest way to end an indefinite park:
//! it sleeps once and posts a quit. Registering a deadline instead would end the run without a
//! second thread and would also be measuring the wrong thing — *with no deadline registered the wait
//! is indefinite* is the property, and a deadline is how you opt out of it.
//!
//! One `sleep` and one `quit` is what the watchdog costs: two voluntary context switches for the
//! whole run, against 3 600 for a ticker.

use std::time::{Duration, Instant};

use vitui_engine::{Clock, Config, Engine, Output, Wake};

/// How long to be idle for when nobody says. Thirty seconds, because that is the floor at which the
/// numbers are above the tool's resolution.
const DEFAULT_SECONDS: u64 = 30;

/// A sink that **counts**, so that *nothing was painted* is a fact rather than an absence nobody
/// could have observed.
///
/// The first draft of this file discarded its bytes and asserted nothing about them, which would have
/// let an idle application that composed a frame pass in silence — a gate that cannot fail, which is
/// exactly what §14 warns against. `AtomicUsize` rather than a `Mutex` because the write happens on
/// the render thread and the count is read on this one.
struct Loud(std::sync::Arc<std::sync::atomic::AtomicUsize>);

impl std::io::Write for Loud {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // The prologue goes through here, and only the prologue. Anything after it is a frame an idle
        // application composed, which is the failure this whole entry is about.
        self.0
            .fetch_add(buf.len(), std::sync::atomic::Ordering::Relaxed);
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
    let quiet = Duration::from_secs(seconds);

    let written = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let counted = std::sync::Arc::clone(&written);
    let (mut screen, wake) = Engine::new(Config {
        // A render thread, which is the point: two threads parked rather than one.
        clock: Clock::System,
        // A ceiling that would be a ticker if the clock were one. At 120 Hz a fixed-rate ticker
        // would wake 120 times a second whether or not anything changed; the gap here arms only
        // after a frame, and no frame is ever submitted.
        max_frame_rate: 120.0,
        // Headless, because this must not reach for the terminal of whoever runs it.
        output: Output::Sink(Box::new(Loud(counted))),
        ..Default::default()
    })
    .attach()
    .expect("attaching to a sink cannot fail");

    // Everything `attach` wrote: raw mode is not in here, but `DECAWM off` and impl 21's negotiation
    // are, and that is the whole of the session prologue. Whatever arrives after this line is a
    // frame.
    let prologue = written.load(std::sync::atomic::Ordering::Relaxed);

    let watchdog = std::thread::spawn(move || {
        std::thread::sleep(quiet);
        wake.quit();
    });

    let began = Instant::now();
    // **One call.** Nothing is pending and no deadline is registered, so this parks indefinitely and
    // the only thing that can end it is the watchdog's quit. A loop here would be measuring the loop.
    let woke = screen.wait();
    let elapsed = began.elapsed();
    watchdog.join().expect("the watchdog does not panic");

    assert_eq!(woke, Wake::Quit, "something woke the app thread: {woke:?}");
    assert!(
        elapsed >= quiet,
        "the wait came back after {elapsed:?} of an intended {quiet:?}"
    );
    // **The count is what makes the absence observable.** `0.00 user 0.00 sys` is consistent with an
    // application that painted nothing and with a probe whose sink was never wired to anything, and
    // this is the assertion that tells those apart.
    let after = written.load(std::sync::atomic::Ordering::Relaxed);
    assert!(
        prologue > 0,
        "the sink was never written to at all, so a byte count here proves nothing"
    );
    assert_eq!(
        after,
        prologue,
        "an idle application put {} bytes on the wire after the prologue",
        after - prologue
    );
    println!(
        "idle {elapsed:?} · app waits 1 · woke for {woke:?} and nothing else \
         · {prologue} prologue bytes and not one since"
    );
    println!(
        "a {} Hz ticker would have woken {} times over the same window",
        120,
        120 * seconds
    );
}
