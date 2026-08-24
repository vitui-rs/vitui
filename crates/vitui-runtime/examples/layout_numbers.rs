//! **Runtime ticket 02's report**: what a realistic screen's layout costs, and that it is flat
//! across terminal sizes.
//!
//! ```text
//! cargo run --release --example layout_numbers -p vitui-runtime
//! ```
//!
//! # The claim being reported, and the claim being asserted
//!
//! Reported: **32 splits and 119 lanes cost about 1.8 µs, which is 1.8% of the 100 µs damage-tracked
//! frame.** A timing, so a report.
//!
//! Asserted: **the cost is flat across terminal sizes**, because it is proportional to *lanes* and
//! not to cells. That is a ratio between two measurements on the same machine in the same process,
//! which is the one shape §14 allows a timing to be gated in — and it is the interesting claim
//! anyway. An absolute microsecond figure says something about this laptop; *80×24 costs what 300×80
//! costs* says something about the algorithm, and it is what fails if somebody makes the solver
//! touch a cell.
//!
//! The shape itself is a **count**: thirty-two splits and one hundred and nineteen lanes, asserted,
//! so that a report which quietly started measuring a smaller screen fails instead of looking good.
//!
//! # Provenance
//!
//! **R 02** took these numbers and **R 20** re-ran the report on 2026-08-24: Apple M1 Max, macOS
//! 26.5.2, rustc 1.97.1, `--release`, unloaded, minimum of 40 rounds. The screen reads 2.75 µs
//! against §11's 1 784 ns, and **the flatness is what carried across and the microseconds are not**:
//! 1.01× over 12× the cells here, 1.01× there. That is the report agreeing with itself in the only
//! currency it can — the absolute figure is this laptop's and the ratio is the algorithm's, which is
//! why the ratio is the gate.
//!
//! **`crate::ledger` carries no row for layout**, because a row is a marginal cost on the frame path
//! and this is not one: the solver runs once a frame whatever the screen does, and the ledger's
//! dense-frame row already contains it.
//!
//! The 100 µs the three percentages divide by is **read from `crate::ledger` and is not this file's
//! to choose**: spec §19 inherits it from the engine map, and **a budget figure may not move without
//! a new map decision.**

use std::hint::black_box;

use vitui_bench::Bench;
// The one realistic screen, shared with the allocation gates and the text report. See
// `src/screen.rs` for why it is included rather than exported.
#[allow(dead_code)]
#[path = "../src/screen.rs"]
mod screen;
use screen::screen_frame;

// **The frame budget every ratio here is against, read rather than written.** See the provenance
// note above: this file used to declare its own copy of the figure.
#[allow(dead_code)]
#[path = "../src/ledger.rs"]
mod ledger;
use ledger::frame_budget_ns;

/// The three terminal sizes. A big one, an old-fashioned one, and one in between — and the point of
/// having three rather than two is that flatness across a 15× range of *cells* is a stronger
/// statement than flatness across two points.
const SIZES: [(u16, u16); 3] = [(300, 80), (120, 40), (80, 24)];

fn main() {
    let (splits, lanes) = screen_frame(300, 80);
    println!("runtime ticket 02 — what a screen of layout costs");
    println!("shape: {splits} splits, {lanes} lanes — a header, a sidebar, a detail pane,");
    println!("       a footer of six buttons, and 24 list rows of 4 columns each\n");
    assert_eq!(
        (splits, lanes),
        (32, 119),
        "the shape is a count and it has changed, so the timings below are about something else"
    );

    // Every size is a case in one bench, so they are round-robined inside each round and share the
    // same interference. Measuring them in three separate benches would compare three different
    // moments in the machine's life.
    //
    // **A hundred iterations a sample, and the first version used one.** `Instant` on this machine
    // has a 41.67 ns quantum (a 24 MHz timebase), and one screen is about 2.8 µs — so all three arms
    // reported *exactly* 2 833 ns, which is 68 ticks. A flatness of 1.00x across three sizes looked
    // like a beautiful result and was the clock rounding three different numbers to the same tick.
    // At a hundred iterations the quantum is 0.015% of the sample and the flatness figure means
    // something.
    let report = Bench::new(40)
        .case("300x80", 100, || {
            black_box(screen_frame(300, 80));
        })
        .case("120x40", 100, || {
            black_box(screen_frame(120, 40));
        })
        .case("80x24", 100, || {
            black_box(screen_frame(80, 24));
        })
        .run();

    println!("report  a whole screen's layout, minimum of 40 rounds:\n{report}");

    let mut worst = 0.0f64;
    let mut best = f64::MAX;
    for (w, h) in SIZES {
        let name = format!("{w}x{h}");
        let ns = report.get(&name).expect("measured");
        worst = worst.max(ns);
        best = best.min(ns);
        println!(
            "        {name:<8} {:>8.2} us  {:>5.2}% of a {:.0} us frame  ({} cells)",
            ns / 1e3,
            ns / frame_budget_ns() * 100.0,
            frame_budget_ns() / 1e3,
            u32::from(w) * u32::from(h),
        );
    }

    let spread = worst / best;
    let cells = f64::from(u32::from(SIZES[0].0) * u32::from(SIZES[0].1))
        / f64::from(u32::from(SIZES[2].0) * u32::from(SIZES[2].1));
    println!(
        "\n        flatness: {spread:.2}x across {cells:.0}x the cells. Cost is proportional to \
         lanes,\n        not to cells, and this is the number that says so.\n        spec §11 \
         measured 1 784 ns at 300x80 (1.8% of the frame) and 1 762 ns at 80x24, 1.01x."
    );
    // **A ratio, so it is a gate.** The band is wide because the three arms are microsecond-scale on
    // a machine that may be doing anything else; what it catches is not a regression but a change of
    // complexity class — a solver that touched cells would be 15x here, not 1.5x.
    assert!(
        spread < 1.5,
        "layout cost varies {spread:.2}x across terminal sizes, so it is no longer flat in the \
         cells — something in the solver is walking the band"
    );
    println!("        gate: flat within 1.5x. Measured {spread:.2}x.");
}
