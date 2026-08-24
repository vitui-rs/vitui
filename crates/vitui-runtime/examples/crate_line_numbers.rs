//! **Runtime ticket 17's report**: what the crate boundary costs the frame, measured rather than
//! assumed.
//!
//! ```text
//! cargo run --release --example crate_line_numbers -p vitui-runtime
//! CARGO_PROFILE_RELEASE_LTO=false cargo run --release --example crate_line_numbers -p vitui-runtime
//! ```
//!
//! # The two arms, and why they are two commands
//!
//! An example is a **separate crate** that links `vitui-runtime` as an rlib: every `cx.text` in
//! `dense_draw` below is a cross-crate call, and that is the crate line. The other arm has to be the
//! identical source compiled *inside* the library, and the only place that exists is the crate's own
//! `cfg(test)` build — `crate::screen` is `#[path]`-included there as well as here, which is why the
//! fixture is one file and not two. So the pair is two commands rather than two cases:
//!
//! ```text
//! line=1 cargo test --release -p vitui-runtime --lib \
//!     line::tests::the_frame_measured_inside_the_crate -- --nocapture
//! ```
//!
//! **The monolith arm R14 measured cannot be reproduced from this workspace, and that is a finding
//! rather than an omission.** Its 33.0 / 28.7 / 28.7 / 28.4 µs came from a prototype where the same
//! code existed both as one crate and as four; here the runtime *is* a crate, and collapsing it into
//! one would be undoing the split this ticket exists to build. What is checkable is the half that
//! matters for the profile decision — whether the boundary costs anything with ThinLTO on — and the
//! two commands above answer it on the two settings.
//!
//! # What was measured, on this machine
//!
//! MacBook (arm64, Darwin 25.5.0), release, minimum of 40 rounds, four repetitions of each arm on
//! each setting. The **layout** arm is the instrument and the **frame** arm is the context:
//!
//! | layout, 32 splits / 119 lanes | in-binary | crate line | delta |
//! |---|---|---|---|
//! | `lto = "thin"` | 2.83–2.87 µs | 2.77–2.82 µs | **−0.07 µs** — the line is not visible |
//! | LTO off | 2.86–2.89 µs | 2.96–3.00 µs | **+0.11 µs, +3.7%** |
//!
//! **R14's shape is confirmed and its magnitude is not reproducible here.** It measured +0.7 µs with
//! LTO off and nothing with ThinLTO, against a monolith; the pair above is the boundary itself, on
//! the smallest workload that crosses it, and it says the same thing: ThinLTO earns the inlining
//! back, and without it the boundary costs something. +0.7 µs was a whole-frame figure on a
//! four-crate-versus-one-crate comparison, so it is not the same number and is not corrected by
//! this one.
//!
//! The **frame** arm is 91–94 µs on both arms and both settings, and that is the honest report of
//! it: 92% of a dense frame is the engine serialising 7 488 damaged cells — `overlay_numbers` prints
//! the same base at 95.9% of the budget — so a boundary worth a tenth of a microsecond is 0.1% of
//! it and inside a ±4 µs run-to-run spread. **A whole frame cannot see this question**, which is why
//! there are two workloads and not one.
//!
//! What the report *asserts* is the **shape**, which is a count: thirty-two splits, a hundred and
//! nineteen lanes, and 5 902 cells written twice when a component pads before it writes — so a
//! report that quietly started measuring a smaller screen fails instead of looking good.

use std::hint::black_box;

use vitui_bench::Bench;
use vitui_runtime::ctx::Driver;

// The one realistic screen, shared with the allocation gates and the other fifteen reports. See
// `src/screen.rs` for why it is included rather than exported.
#[allow(dead_code)]
#[path = "../src/screen.rs"]
mod screen;
use screen::{dense_draw, screen_frame};

/// The frame budget every ratio here is against.
const FRAME_NS: f64 = 100_000.0;

fn main() {
    let lto = option_env!("CARGO_PROFILE_RELEASE_LTO").unwrap_or("thin (the manifest's)");
    let (splits, lanes) = screen_frame(300, 80);
    assert_eq!(
        (splits, lanes),
        (32, 119),
        "the shape is a count and it has changed, so the timings below are about something else"
    );

    println!("runtime ticket 17 — what the crate line costs the frame");
    println!(
        "arm:    the crate line. This example is a separate crate linking vitui-runtime as an"
    );
    println!("        rlib, so every drawing verb below is a cross-crate call.");
    println!("lto:    {lto}\n");

    let mut driver = Driver::headless(300, 80).expect("a sink cannot fail to attach");
    let mut twice = 0;
    driver.frame(|cx| twice = dense_draw(cx, false));
    assert_eq!(
        twice, 5902,
        "the fixture's double-write count has changed, so this is a different screen"
    );

    // **Two workloads, and the small one is the instrument.** A whole frame is ~90 µs and 92% of it
    // is the engine serialising 7 488 damaged cells (`overlay_numbers` reports the same base), so a
    // boundary worth 0.7 µs is 0.8% of it and inside the run-to-run spread. The layout arm is 32
    // splits and 119 lanes of pure runtime arithmetic at ~1.8 µs — every call of it crosses the line
    // — and that is where a cross-crate call that stopped being inlined would be unmissable.
    let report = Bench::new(40)
        .case("frame/text-first", 1, || {
            driver.frame(|cx| {
                black_box(dense_draw(cx, true));
            });
        })
        .case("layout/32-splits", 100, || {
            black_box(screen_frame(300, 80));
        })
        .run();
    let ns = report.get("frame/text-first").expect("measured");
    let layout_ns = report.get("layout/32-splits").expect("measured");

    println!("report  a whole dense frame, minimum of 40 rounds:\n{report}");
    println!(
        "        {:>8.2} us  {:>5.2}% of a {:.0} us frame  (300x80, 312 regions, text-first)",
        ns / 1e3,
        ns / FRAME_NS * 100.0,
        FRAME_NS / 1e3,
    );
    println!(
        "        {:>8.2} us  {:>5.2}% of a {:.0} us frame  (32 splits, 119 lanes, layout only)",
        layout_ns / 1e3,
        layout_ns / FRAME_NS * 100.0,
        FRAME_NS / 1e3,
    );
    println!(
        "\n        Pair it with the in-binary arm and compare:\n        line=1 cargo test \
         --release -p vitui-runtime --lib \\\n            \
         line::tests::the_frame_measured_inside_the_crate -- --nocapture"
    );
    println!(
        "\n        R14 measured the boundary at +0.7 us with LTO off and nothing with ThinLTO,"
    );
    println!("        against a monolith this workspace cannot build. Run both commands on both");
    println!("        settings; the delta between the two arms is the boundary.");
    // **A ratio against the budget, so it is a gate at cliff granularity.** What it catches is not a
    // regression of a microsecond but a frame that has stopped being proportional to visible cells.
    assert!(
        ns < FRAME_NS * 2.0,
        "a dense frame costs {:.1} us, which is over twice the 100 us damage-tracked budget — the \
         boundary is not what went wrong at that size",
        ns / 1e3
    );
}
