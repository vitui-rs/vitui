//! **Runtime ticket 06's reports**: what motion costs, and what the terminal does with it.
//!
//! ```text
//! cargo run --release --example anim_numbers -p vitui-runtime
//! ```
//!
//! Seven sections, and the first is the one the module's rustdoc quotes:
//!
//! 1. **The price of motion.** What a fading chip, a spinner, a caret and a spring cost per frame,
//!    against the frame each of them asks for. The arithmetic is three orders of magnitude cheaper
//!    than the wake, which is why the accounting is a ledger and not a stopwatch.
//! 2. **The ledger's own cost**, which is what makes it unconditional. Both signs against a whole
//!    frame is the finding: it is under the run-to-run spread of the frame it sits in.
//! 3. **The sizes**, because *there is no animation object* is a size before it is an argument.
//! 4. **Distinct values on the wire**, measured by driving the engine at three tiers rather than by
//!    consulting our own copy of the quantiser. This is the evidence for *the tier decides whether
//!    an animation is one at all*.
//! 5. **The fourteen shipped schemes at 256 colours** — the in-tree replacement for spec §15's *165
//!    of 338*, which is unreachable by construction. See the ticket's answer; this section is the
//!    finding's measurement.
//! 6. **Drift**, both ways: an anchored phase against an accumulated one, and an anchored spinner
//!    against one asked at `now + per`.
//! 7. **The spring**: what a dropped velocity costs, what an integrator costs, and what each
//!    threshold costs in frames.
//!
//! Every number here is a **report**. The counts that are gates live in
//! `crates/vitui-runtime/src/anim.rs` and in `tests/alloc.rs`.

use std::hint::black_box;
use std::io::Write as _;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use vitui_bench::Bench;
use vitui_engine::{Clock, ColorDepth, Config, Output, Overrides, Rect, Rgb};
use vitui_runtime::anim::{Easing, Spring, Steps, Tween, WakeLedger};
use vitui_runtime::ctx::Driver;
use vitui_runtime::theme::{Density, Distinction, GlyphSet, Role, Theme, schemes};

/// 60 Hz, rounded to the nearest nanosecond — `anim`'s own `TICK`.
const TICK: Duration = Duration::from_nanos(16_666_667);

/// How many frames a 300 ms fade takes at 60 Hz. Nineteen, and `anim`'s gate says why.
const FADE_FRAMES: u32 = 19;

fn main() {
    println!("runtime ticket 06 — what motion costs, and what the terminal does with it\n");
    the_price_of_motion();
    what_the_ledger_costs();
    the_sizes();
    distinct_values_on_the_wire();
    the_fourteen_schemes_at_256_colours();
    drift_both_ways();
    the_spring();
}

/// Report 1: the arithmetic against the frame it asks for.
fn the_price_of_motion() {
    let start = Instant::now();
    let theme = Theme::default().resolve(ColorDepth::TrueColor);
    let fade: Tween<f32> =
        Tween::new(start, Duration::from_millis(300), 0.0, 1.0).eased(Easing::Out);
    let slide: Tween<i32> = Tween::new(start, Duration::from_millis(200), 0, 20);
    let spinner = Steps::new(start, Duration::from_millis(80));
    let caret = Steps::new(start, Duration::from_millis(530));
    let spring = Spring::new(start, 0.0, 20.0);
    let mid = start + Duration::from_millis(97);

    let mut driver = Driver::headless(80, 24).expect("attaching to a sink cannot fail");
    let mut frame = || {
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            for row in 0..24i32 {
                cx.label(
                    0,
                    row,
                    format_args!("row {row:>3}  a line of an application that is not moving"),
                    body,
                );
            }
        });
    };

    let report = Bench::new(40)
        .case("a fading chip's arithmetic", 10_000, || {
            let t = black_box(&fade).phase(black_box(mid));
            black_box(black_box(&theme).mix(Role::Face, Role::FaceHover, t));
        })
        .case("a spinner", 10_000, || {
            black_box(black_box(&spinner).cycle(black_box(mid), 4));
        })
        .case("a caret", 10_000, || {
            black_box(black_box(&caret).on(black_box(mid)));
        })
        .case("a sliding panel", 10_000, || {
            black_box(black_box(&slide).value(black_box(mid)));
        })
        .case("a spring", 10_000, || {
            black_box(black_box(&spring).value(black_box(mid)));
        })
        .case("the frame it asks for", 200, &mut frame)
        .run();
    println!("report  the price of motion, minimum of 40 rounds:\n{report}");

    let chip = report.get("a fading chip's arithmetic").expect("measured");
    let whole = report.get("the frame it asks for").expect("measured");
    println!(
        "        the fading chip is {:.3}% of the frame it asks for — {chip:.1} ns against \
         {:.2} µs, so\n\
         \x20       **{:.2}% of what an animation costs is redrawing a screen that did not \
         change.**\n\
         \x20       Spec §16 measured 114 ns against 34.17 µs — 0.33%, and 99.7% the same \
         sentence. The\n\
         \x20       arms differ because these are the *helpers* (a phase and a `Theme::mix`) where \
         the\n\
         \x20       map's 114 ns is a whole chip including its drawing verbs, which cannot be \
         measured\n\
         \x20       outside a frame; the conclusion is the one that reproduces, and it reproduces \
         harder.\n\
         \x20       Nothing in this runtime skips composition, so an animation's real cost is the \
         frame.\n\
         \x20       Sixty of those a second is {:.2} ms/s, {:.2}% of a core.\n",
        100.0 * chip / whole,
        whole / 1000.0,
        100.0 * (1.0 - chip / whole),
        60.0 * whole / 1e6,
        100.0 * 60.0 * whole / 1e9,
    );
}

/// Report 2: the ledger, which is why it needs no feature flag.
fn what_the_ledger_costs() {
    let now = Instant::now();
    // Two drivers, because two cases cannot both hold the same one — and each keeps its own clock
    // pinned, so neither is measuring `Instant::now`.
    let mut quiet_driver = Driver::headless(80, 24).expect("sink");
    let mut twin_driver = Driver::headless(80, 24).expect("sink");
    let mut asking_driver = Driver::headless(80, 24).expect("sink");
    quiet_driver.pin_clock(now);
    twin_driver.pin_clock(now);
    asking_driver.pin_clock(now);

    let report = Bench::new(40)
        .case("a frame that asks for nothing", 200, || {
            quiet_driver.frame(|cx| {
                let body = cx.theme().paint(Role::Body);
                for row in 0..24i32 {
                    cx.text(0, row, "a line of an application that is not moving", body);
                }
            });
        })
        // **The same frame under a second name**, which is what makes the row below a claim rather
        // than an assertion: it prices the run-to-run spread the ledger has to be smaller than.
        .case("the same frame again", 200, || {
            twin_driver.frame(|cx| {
                let body = cx.theme().paint(Role::Body);
                for row in 0..24i32 {
                    cx.text(0, row, "a line of an application that is not moving", body);
                }
            });
        })
        .case("a frame that asks", 200, || {
            asking_driver.frame(|cx| {
                let body = cx.theme().paint(Role::Body);
                for row in 0..24i32 {
                    cx.text(0, row, "a line of an application that is not moving", body);
                }
                cx.request_frame();
            });
        })
        .run();
    println!("report  a frame with an ask against one without, minimum of 40 rounds:\n{report}");

    let quiet = report
        .get("a frame that asks for nothing")
        .expect("measured");
    let twin = report.get("the same frame again").expect("measured");
    let asking = report.get("a frame that asks").expect("measured");
    println!(
        "        the ask costs {:+.2} ns a frame against a frame of {:.2} µs — {:+.4}% — and **two \
         runs of\n\
         \x20       the identical frame differ by {:+.2} ns**, which is the row that matters. The \
         whole ask\n\
         \x20       is `Location::caller`, a fold, a scan of one entry and a note, and its sign is \
         not\n\
         \x20       stable from one run of this report to the next: it lands on either side of \
         zero, at the\n\
         \x20       same magnitude as two runs of the frame with nothing changed at all. Spec §16 \
         measured\n\
         \x20       1.51–1.61 ns **of both signs** against a 34 µs frame and drew the conclusion \
         this row\n\
         \x20       supports — there is no `debug_assertions` guard and no feature flag, because a \
         detector\n\
         \x20       armed only in a debug build never sees the application, and this is what it \
         costs to\n\
         \x20       arm it always: nothing that can be told apart from noise.\n",
        asking - quiet,
        quiet / 1000.0,
        100.0 * (asking - quiet) / quiet,
        twin - quiet,
    );

    println!(
        "        The ledger's own two writes are not measured separately here **and cannot be**: \
         `asked`\n\
         \x20       and `note` are `pub(crate)`, because only the frame may write to the sink. That \
         is the\n\
         \x20       right way round — an application that could bump the ledger could hide a \
         runaway — and\n\
         \x20       the frame delta above is the number a caller can actually observe. It is a \
         fixed array\n\
         \x20       with no allocation at any point in its life, which is why `tests/alloc.rs` can \
         count a\n\
         \x20       hundred animating frames at 0.\n"
    );
}

/// Report 3: the sizes, because *no animation object* is a size first.
fn the_sizes() {
    println!("report  the sizes:");
    println!(
        "        Tween<f32>  {:>3} B   (16 Instant + 16 Duration + 2 values + 1 easing, padded)",
        std::mem::size_of::<Tween<f32>>()
    );
    println!(
        "        Tween<i32>  {:>3} B",
        std::mem::size_of::<Tween<i32>>()
    );
    println!(
        "        Steps       {:>3} B   (an anchor and a period)",
        std::mem::size_of::<Steps>()
    );
    println!(
        "        Spring      {:>3} B   (four fields: start, x0, v0, target — and no duration)",
        std::mem::size_of::<Spring>()
    );
    println!(
        "        WakeLedger  {:>3} B   ({} lines and {} census entries, both fixed)\n",
        std::mem::size_of::<WakeLedger>(),
        vitui_runtime::anim::LINES,
        vitui_runtime::anim::LINES,
    );
}

/// A sink the report can read back while the screen is still alive.
#[derive(Clone)]
struct Shared(Arc<Mutex<Vec<u8>>>);

impl std::io::Write for Shared {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .expect("no panic holds this lock")
            .extend_from_slice(b);
        Ok(b.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// How many of a fade's frames put **new bytes on the wire** at `depth`.
///
/// **The engine is the instrument and not our copy of the quantiser.** The serialiser writes against
/// its mirror of what the terminal shows, so a frame whose colour quantises onto the previous one
/// writes nothing at all — and the count of frames that wrote is exactly the number of distinct
/// values the terminal was given.
fn wire_values(theme: &Theme, depth: ColorDepth, samples: u32) -> usize {
    let buf = Arc::new(Mutex::new(Vec::new()));
    let mut driver = Driver::attach(
        Config {
            clock: Clock::Manual,
            output: Output::Sink(Box::new(Shared(Arc::clone(&buf)))),
            size: (4, 1),
            overrides: Overrides {
                colors: Some(depth),
                // Declared, so the engine narrows against a stated ground rather than a guess.
                default_fg: Some(Rgb::new(0xcd, 0xd6, 0xf4)),
                default_bg: Some(Rgb::new(0x1e, 0x1e, 0x2e)),
                ..Default::default()
            },
            ..Default::default()
        },
        *theme,
    )
    .expect("attaching to a sink cannot fail");

    let mut written = 0;
    let mut seen = 0;
    for i in 0..samples {
        #[expect(
            clippy::cast_precision_loss,
            reason = "a sample index under a hundred is exact in f32"
        )]
        let t = i as f32 / f32::from(u16::try_from(samples - 1).unwrap_or(1));
        driver.frame(|cx| {
            let paint = cx.theme().mix(Role::Face, Role::FaceHover, t);
            cx.fill(Rect::new(0, 0, 4, 1), " ", paint);
        });
        let now = buf.lock().expect("no panic holds this lock").len();
        if now > written {
            seen += 1;
            written = now;
        }
    }
    seen
}

/// Report 4: the tier decides whether an animation is one at all.
fn distinct_values_on_the_wire() {
    println!(
        "report  distinct values on the wire for a Face → FaceHover fade, asked of the engine:"
    );
    println!("        tier          values   the same fade's wakeups, ungated / gated");
    for depth in [
        ColorDepth::TrueColor,
        ColorDepth::Indexed256,
        ColorDepth::Ansi16,
    ] {
        let theme = Theme::default().resolve(depth);
        let values = wire_values(&theme, depth, FADE_FRAMES);
        let gated = if theme.shows(Distinction::Fade) {
            FADE_FRAMES
        } else {
            2
        };
        println!(
            "        {:<12} {values:>6}   {FADE_FRAMES} / {gated}{}",
            format!("{depth:?}"),
            if values < 4 {
                "        ← nineteen frames buy that many pictures"
            } else {
                ""
            }
        );
    }
    println!(
        "        Spec §16 measured 30 / 1 / 1 over a thirty-sample fade; this is nineteen samples, \
         which\n\
         \x20       is what 300 ms at 60 Hz actually gives. **The middle column does not \
         reproduce**: the\n\
         \x20       default palette's two face backgrounds are 237 and 239 and the ramp between \
         them\n\
         \x20       contains a third index, so a 256-colour terminal is shown three pictures rather \
         than\n\
         \x20       one. That is ticket 05's finding measured from the other side, and it is the \
         next\n\
         \x20       section. The conclusion is unmoved: nineteen wakeups for three pictures is \
         still a\n\
         \x20       capability read away from two.\n"
    );
}

/// Report 5: the in-tree replacement for spec §15's *165 of 338*.
///
/// **This is the finding's measurement**, and the finding is in the ticket's answer:
/// `Distinction::Fade` is gated on the tier, so no palette can make it true below truecolor. What a
/// palette *can* do is carry a ramp the terminal can still show, and that is what this counts.
fn the_fourteen_schemes_at_256_colours() {
    println!("report  the fourteen shipped schemes at 256 colours:");
    println!("        scheme                     ramp values   shows(Fade)");
    let mut showable = 0;
    for scheme in schemes::STANDARD {
        let theme = scheme
            .theme(GlyphSet::Extended, Density::Cosy)
            .resolve(ColorDepth::Indexed256);
        let values = wire_values(&theme, ColorDepth::Indexed256, FADE_FRAMES);
        if values > 1 {
            showable += 1;
        }
        println!(
            "        {:<26} {values:>11}   {}",
            scheme.slug(),
            theme.shows(Distinction::Fade),
        );
    }
    println!(
        "        **{showable} of {} shipped schemes carry a ramp a 256-colour terminal can show**, \
         and\n\
         \x20       `shows(Fade)` says false for all fourteen — because ticket 04 implements it as\n\
         \x20       `differs(Face, FaceHover) && tier == TrueColor`. The error is in the safe \
         direction: a\n\
         \x20       component is told *do not animate* when it could have, which costs an animation \
         and\n\
         \x20       never a wrong cell. Spec §15's *165 of 338* is a statement about palettes and \
         is\n\
         \x20       unreachable by construction; §16's *19 / 2 / 2* is a statement about tiers and \
         is\n\
         \x20       exactly what ships. Filed as a finding against §15 rather than decided here.\n",
        schemes::STANDARD.len(),
    );
}

/// Report 6: drift, both ways.
fn drift_both_ways() {
    let start = Instant::now();
    let second = Duration::from_secs(1);
    let fade: Tween<f32> = Tween::new(start, second, 0.0, 1.0);

    println!("report  drift, which is the runtime's and compounds with the run:");
    println!("        lateness   anchored phase   accumulated phase   frames");
    for late in [
        Duration::ZERO,
        Duration::from_millis(1),
        Duration::from_micros(1_333),
    ] {
        let step = TICK + late;
        let mut sum = 0.0f32;
        let mut now = start;
        let mut frames = 0u32;
        while fade.phase(now) < 1.0 {
            now += step;
            frames += 1;
            sum += 1.0 / 60.0;
        }
        println!(
            "        {:>8.2?}   {:>14.4}   {sum:>17.4}   {frames:>6}",
            late,
            fade.phase(now),
        );
    }
    println!(
        "        Spec §16 measured 1.0000 against 0.9222. The anchored column is 1.0000 on every\n\
         \x20       schedule **by construction** — it is a division, not a sum — and the \
         accumulated one\n\
         \x20       is short by exactly however late the loop was."
    );

    let per = Duration::from_millis(80);
    let thirty = Duration::from_secs(30);
    let anchored = Steps::new(start, per);
    println!("\n        spinner steps in 30 s, per = 80 ms:");
    println!("        lateness   anchored   asked at `now + per`");
    for late in [
        Duration::ZERO,
        Duration::from_millis(1),
        Duration::from_millis(5),
    ] {
        let mut at = start;
        let mut drifting = 0u64;
        while at + per + late <= start + thirty {
            at += per + late;
            drifting += 1;
        }
        println!(
            "        {:>8.2?}   {:>8}   {drifting:>20}",
            late,
            anchored.index(start + thirty),
        );
    }
    println!(
        "        Spec §16 measured 375 against 353, which is this table's 5 ms row to within one\n\
         \x20       step of the window's own edge. **The anchored count does not have a lateness \
         column**:\n\
         \x20       `next_at` is `start + (n + 1) × per` and lands on the grid whenever it is \
         asked.\n"
    );
}

/// Report 7: the spring.
fn the_spring() {
    let start = Instant::now();
    let flying = Spring::new(start, 0.0, 20.0);
    let mid = start + Duration::from_millis(120);

    let kept = flying.retarget(mid, 40.0);
    let dropped = Spring::new(mid, flying.value(mid), 40.0);
    let next = mid + TICK;
    println!("report  the spring:");
    println!(
        "        a retarget that drops the velocity is {:.2} cells and {:.0} cells/s apart one \
         frame later\n\
         \x20       (spec §16: 3.17 cells and 141 cells/s — the same shape at this module's own \
         RATE of {:.0}).",
        kept.value(next) - dropped.value(next),
        kept.velocity(mid) - dropped.velocity(mid),
        Spring::RATE,
    );

    print!("        a per-frame integrator is off the closed form by");
    for (label, jitter) in [
        ("steady", Duration::ZERO),
        ("jittered", Duration::from_millis(4)),
    ] {
        let mut x = 0.0f32;
        let mut v = 0.0f32;
        let mut now = start;
        let mut worst = 0.0f32;
        for i in 0..60 {
            let step = if i % 3 == 0 { TICK + jitter } else { TICK };
            let h = step.as_secs_f32();
            let a = -Spring::RATE * Spring::RATE * (x - 20.0) - 2.0 * Spring::RATE * v;
            v += a * h;
            x += v * h;
            now += step;
            worst = worst.max((x - flying.value(now)).abs());
        }
        print!(" {worst:.2} cells {label},");
    }
    println!(
        "\n\
         \x20       against **0 by construction** — so a spring may compute no interval at all \
         (spec §16:\n\
         \x20       3.95 / 5.50 cells, and the worst error over the flight rather than the error at \
         the end,\n\
         \x20       because both arrive: converging is the one thing a spring cannot get wrong).\n"
    );

    println!("        threshold   frames to sleep, over a twenty-cell move at 60 Hz");
    for threshold in [0.5f32, 1e-3, 1e-6, 0.0] {
        let mut now = start;
        let mut frames = None;
        for frame in 0..100_000u32 {
            if flying.settled(now, threshold) {
                frames = Some(frame);
                break;
            }
            now += TICK;
        }
        match frames {
            Some(n) if threshold == 0.0 => println!(
                "        {threshold:>9}   {n:>6}   ← not *never*: this is where `e^(−ω·t)` \
                 underflows, {:.1} s of\n\
                 \x20                            a screen redrawing for a picture that stopped \
                 changing after one.",
                f64::from(n) * TICK.as_secs_f64(),
            ),
            Some(n) => println!("        {threshold:>9}   {n:>6}"),
            None => println!("        {threshold:>9}      none"),
        }
    }
    println!(
        "        Spec §16 measured 32 / 79 / never. The ratio between two thresholds is fixed by \
         the\n\
         \x20       displacement alone for a critically damped spring, and 32 : 79 needs a move of \
         about\n\
         \x20       630 cells — off a 300-column screen — so R11's spring was not this one. What \
         holds\n\
         \x20       mechanically is the ordering and the last row: **a threshold is what lets the \
         screen\n\
         \x20       sleep, and zero is a runaway the detector catches.**"
    );
    std::io::stdout().flush().expect("stdout");
}
