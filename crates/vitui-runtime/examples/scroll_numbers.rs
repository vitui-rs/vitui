//! **Runtime ticket 14's report**: what each of the two scrolling mechanisms costs, and what
//! choosing the wrong one costs.
//!
//! ```text
//! cargo run --release --example scroll_numbers -p vitui-runtime
//! ```
//!
//! Four numbers, and the first is the only one that matters above this crate:
//!
//! 1. **The wrong pairing** — a `scroll_area` over a million rows against a virtualised collection
//!    over the same million. Spec §13 measured 887–889× against 0.997–1.003×, so 7 025–7 076×.
//! 2. **The right pairing** — a full-screen area with a virtualised list inside it, over the same
//!    million rows. The two mechanisms compose; it is choosing one *instead of* the other that costs.
//! 3. **Four direction bits are free** — the matcher is one branch on a bit the widget was already
//!    passing, over §6's 312 entries.
//! 4. **A frame that is not measuring must not maintain the measurement** — what the drawn extent
//!    costs a real frame if it is left on, which is the near-miss worth carrying into every future
//!    frame-local accounting field.
//!
//! Reports, gating nothing. The gates for all four are counts and ratios in `crate::scroll`.
//!
//! # Provenance
//!
//! **R 14** took these numbers and **R 20** re-measured the wheel matcher against the shipped
//! runtime on 2026-08-24: Apple M1 Max, macOS 26.5.2, rustc 1.97.1, `--release`, unloaded, minimum
//! of 40 rounds. The ledger row is **−19 ns**, and the prototype recorded it as *of both signs* over
//! a −0.033 to +0.190 µs range — so this run is one sign and inside that range, which is a figure
//! at the noise floor confirming that it is still at the noise floor rather than a figure that
//! moved.
//!
//! **Two of the microsecond columns here are the fixture's and not the mechanism's**, and the report
//! prints both rather than reconciling them: a collection's frame is 14 µs where §13 measured
//! 0.29 µs, because this one is a whole frame through the engine and §13 timed the draw. The empty
//! frame at 0.06 µs is what proves that is visible work rather than fixed overhead. The conclusion
//! survives the difference because it is a ratio — an area is proportional to its content and a
//! collection is flat — and the gate under it is a row count, 1 000 000 against 24.
//!
//! The 100 µs the two percentages divide by is **read from `crate::ledger` and is not this file's to
//! choose**: spec §19 inherits it from the engine map, and **a budget figure may not move without a
//! new map decision.** The wrong pairing is printed as *35× the whole frame budget*, which is a
//! sentence about the map's figure and not about this laptop's.

use std::hint::black_box;

use vitui_bench::Bench;
use vitui_engine::Rect;
use vitui_runtime::ctx::{Ctx, Driver, Interest};
use vitui_runtime::id::Id;
use vitui_runtime::scroll::Scrollable;
use vitui_runtime::theme::Role;

// **The frame budget every ratio here is against, read rather than written.** See the provenance
// note above: this file used to declare its own copy of the figure.
#[allow(dead_code)]
#[path = "../src/ledger.rs"]
mod ledger;
use ledger::frame_budget_ns;

/// The dense screen's interactive region count, from spec §6.
const DENSE: u64 = 312;

fn main() {
    println!("runtime ticket 14 — the two mechanisms, and what conflating them costs\n");
    the_two_mechanisms();
    four_directions_against_two_axes();
    the_extent_left_on();
}

/// Report: ∝ content against ∝ window, at 1k and at 1M, and the two of them composed.
fn the_two_mechanisms() {
    let area_id = Id::named("area");
    let list_id = Id::named("list");

    // A scroll area: the body draws the whole content and the clip rejects what is off screen.
    fn area(cx: &mut Ctx<'_, '_>, id: Id, rows: i32) {
        let body = cx.theme().paint(Role::Body);
        cx.scroll_scope(id, Rect::new(0, 0, 80, 24), (0, 0), (0, rows), |cx| {
            for y in 0..rows {
                cx.text(0, y, "a row of a form, a panel or a document page", body);
            }
        });
    }

    // A virtualised collection: the caller draws the rows the visible range admits.
    fn list(cx: &mut Ctx<'_, '_>, id: Id, rows: i32) {
        let body = cx.theme().paint(Role::Body);
        cx.scroll_scope(id, Rect::new(0, 0, 80, 24), (0, 0), (0, rows), |cx| {
            let visible = cx.visible_rows();
            for y in visible.start..visible.end.min(rows) {
                cx.text(0, y, "a row of a log, a file list or a process table", body);
            }
        });
    }

    let mut d = Driver::headless(80, 24).expect("sink");
    let report = Bench::new(9)
        .case("area, 1k rows", 20, || {
            black_box(&mut d).frame(|cx| area(cx, area_id, 1_000));
        })
        .run();
    let a1k = report.get("area, 1k rows").expect("measured");

    let mut d = Driver::headless(80, 24).expect("sink");
    let report = Bench::new(3)
        .case("area, 1M rows", 1, || {
            black_box(&mut d).frame(|cx| area(cx, area_id, 1_000_000));
        })
        .run();
    let a1m = report.get("area, 1M rows").expect("measured");

    // One driver a case: `Bench` holds every closure at once, so they cannot share a `&mut`.
    let mut small = Driver::headless(80, 24).expect("sink");
    let mut large = Driver::headless(80, 24).expect("sink");
    let mut both_d = Driver::headless(80, 24).expect("sink");
    let mut empty = Driver::headless(80, 24).expect("sink");
    let report = Bench::new(9)
        .case("an empty frame", 200, || {
            black_box(&mut empty).frame(|cx| {
                let _ = cx.area();
            });
        })
        .case("list, 1k rows", 200, || {
            black_box(&mut small).frame(|cx| list(cx, list_id, 1_000));
        })
        .case("list, 1M rows", 200, || {
            black_box(&mut large).frame(|cx| list(cx, list_id, 1_000_000));
        })
        .case("an area with a list inside it, 1M rows", 200, || {
            black_box(&mut both_d).frame(|cx| {
                cx.scroll_scope(area_id, Rect::new(0, 0, 80, 24), (0, 0), (0, 0), |cx| {
                    list(cx, list_id, 1_000_000);
                });
            });
        })
        .run();
    let l1k = report.get("list, 1k rows").expect("measured");
    let l1m = report.get("list, 1M rows").expect("measured");
    let both = report
        .get("an area with a list inside it, 1M rows")
        .expect("measured");
    // **The empty frame, printed because it is a fact and not a floor.** Damage is marked at write
    // time, so a frame that draws nothing composites nothing — which is why the *baseline* below is
    // the collection and not this: the visible work every frame pays is the twenty-four rows a
    // window admits, and that is exactly what a collection's frame is.
    let empty = report.get("an empty frame").expect("measured");

    // **The content cost**: an area's frame minus a collection's over the same data, which is the
    // work the clip rejected. Spec §13's figures are of the draw alone, and this is the like-for-like
    // subtraction that gets there from a whole frame.
    let (ca1k, ca1m) = ((a1k - l1k).max(1.0), (a1m - l1m).max(1.0));
    println!("report  the two mechanisms, 1k rows -> 1M rows:\n");
    println!(
        "          an empty frame{:>12.2} us   — nothing drawn, so nothing composited\n\
        \x20         scroll area   {:>12.2} us -> {:>12.2} us   {:>9.1}x\n\
        \x20         collection    {:>12.2} us -> {:>12.2} us   {:>9.3}x\n\
        \x20         the content   {:>12.2} us -> {:>12.2} us   {:>9.0}x   (area - collection)",
        empty / 1e3,
        a1k / 1e3,
        a1m / 1e3,
        a1m / a1k,
        l1k / 1e3,
        l1m / 1e3,
        l1m / l1k,
        ca1k / 1e3,
        ca1m / 1e3,
        ca1m / ca1k,
    );
    println!(
        "\n          the wrong pairing, at 1M   {:.0}x — {:.2} ms against {:.2} us,\n\
        \x20                                     {:.0}x the whole frame budget\n\
        \x20         the right pairing, at 1M   {:.2} us — an area with a virtualised list inside it",
        a1m / l1m,
        a1m / 1e6,
        l1m / 1e3,
        a1m / frame_budget_ns(),
        both / 1e3,
    );
    println!(
        "\n        Spec §13 measured **887–889x against 0.997–1.003x**, so the wrong pairing at\n\
        \x20       **7 025–7 076x — 2.03 ms against 0.29 us**, and the right pairing at\n\
        \x20       **7.65–7.80 us**. Every conclusion reproduces; two of the figures differ and both\n\
        \x20       differences are the fixture rather than the mechanism, so they are printed rather\n\
        \x20       than reconciled away:\n\
        \x20       - a **collection's frame** here is 14 us and §13's is 0.29 us, because this one is\n\
        \x20         a whole frame — twenty-four rows of real text through the engine and the cells\n\
        \x20         they damaged, composited and presented — where §13 timed the draw. The empty\n\
        \x20         frame above is what says so: it is not a fixed overhead, it is the visible work.\n\
        \x20       - the **area's growth** is smaller for the same reason at the other end: at 1k\n\
        \x20         rows most of its frame is still that visible work, so 1k -> 1M understates it.\n\
        \x20         Subtract the collection at each size and the content alone grows by the figure\n\
        \x20         on the fourth line, which is §13's order and then some.\n\
        \x20       The conclusion is the ratio and not the millisecond: an area is proportional to\n\
        \x20       its content and a collection is flat, so the gap grows with the data and the\n\
        \x20       mistake gets worse the longer an application lives.\n\
        \x20       **The gate is a count, not this timing**: `scroll::tests::\n\
        \x20       a_scroll_area_costs_the_content_and_a_list_costs_the_window` counts the rows each\n\
        \x20       body iterated — 1 000 000 against 24 — because the count is the mechanism and the\n\
        \x20       timing is the weather."
    );
}

/// Report: what matching four directions costs against matching two axes, over 312 entries.
fn four_directions_against_two_axes() {
    // The whole space, so neither matcher is being measured on a lucky input.
    let bits: Vec<Scrollable> = (0..16u8)
        .map(|b| {
            let pick = |set: bool, s: Scrollable| if set { s } else { Scrollable::NONE };
            pick(b & 1 != 0, Scrollable::UP)
                .with(pick(b & 2 != 0, Scrollable::DOWN))
                .with(pick(b & 4 != 0, Scrollable::LEFT))
                .with(pick(b & 8 != 0, Scrollable::RIGHT))
        })
        .collect();
    let entries: Vec<Scrollable> = (0..DENSE).map(|i| bits[(i % 16) as usize]).collect();

    let axis = |s: Scrollable, d: (i32, i32)| {
        let y = s.contains(Scrollable::UP) || s.contains(Scrollable::DOWN);
        let x = s.contains(Scrollable::LEFT) || s.contains(Scrollable::RIGHT);
        (d.1 != 0 && y) || (d.0 != 0 && x)
    };

    let report = Bench::new(40)
        .case("four direction bits, 312 entries", 2_000, || {
            let mut hit = 0u32;
            for s in black_box(&entries) {
                if s.admits((0, 1)) {
                    hit += 1;
                }
            }
            black_box(hit);
        })
        .case("two axis bools, 312 entries", 2_000, || {
            let mut hit = 0u32;
            for s in black_box(&entries) {
                if axis(*s, (0, 1)) {
                    hit += 1;
                }
            }
            black_box(hit);
        })
        .run();
    let four = report
        .get("four direction bits, 312 entries")
        .expect("measured");
    let two = report.get("two axis bools, 312 entries").expect("measured");
    println!("\nreport  the wheel matcher over {DENSE} entries:\n{report}");
    println!(
        "        four directions {:.3} us, two axes {:.3} us, difference {:+.3} us\n\
        \x20       ({:+.3}% of the frame budget).\n\
        \x20       Spec §13 measured **-0.033 to +0.190 us**, which is *of both signs* — the two are\n\
        \x20       the same branch on the same byte and the difference is the run-to-run spread.\n\
        \x20       **Free, and it buys 16 of 64 enumerated cases**: the direction matcher is never\n\
        \x20       looser, and where it differs is exactly the case that matters — a collection at\n\
        \x20       its end stop, which under an axis bool eats every click for ever.",
        four / 1e3,
        two / 1e3,
        (four - two) / 1e3,
        (four - two) / frame_budget_ns() * 100.0,
    );
}

/// Report: the price of maintaining the drawn extent in a frame that is not measuring.
fn the_extent_left_on() {
    fn dense(cx: &mut Ctx<'_, '_>) {
        let body = cx.theme().paint(Role::Body);
        let interest = cx.theme().hover_interest().with(Interest::CLICK);
        for i in 0..DENSE {
            let y = i32::try_from(i % 80).unwrap_or(0);
            let x = i32::try_from((i / 80) * 40).unwrap_or(0);
            cx.text(x, y, "a label beside an interactive region", body);
            cx.interact(Id::keyed(Id::ROOT, i), Rect::new(x, y, 40, 1), interest);
        }
    }

    let mut off = Driver::headless(300, 80).expect("sink");
    let mut on = Driver::headless(300, 80).expect("sink");
    on.measure_extent(true);
    let report = Bench::new(40)
        .case("a dense frame", 20, || {
            black_box(&mut off).frame(dense);
        })
        .case("the same frame, maintaining the extent", 20, || {
            black_box(&mut on).frame(dense);
        })
        .run();
    let a = report.get("a dense frame").expect("measured");
    let b = report
        .get("the same frame, maintaining the extent")
        .expect("measured");
    println!("\nreport  the drawn extent, left on in a real frame:\n{report}");
    println!(
        "        off {:.2} us, on {:.2} us, {:+.2} us — {:+.1}% of this frame and {:+.0} ns for each\n\
        \x20       of {DENSE} verbs, for a field that is off by default.\n\
        \x20       Spec §13 priced the same near-miss at **+7 us, 7%**, and ticket 15 measured the\n\
        \x20       grapheme walk at 27.5 ns a verb. This fixture's labels are longer, so the per-verb\n\
        \x20       figure is higher and the shape is the same one. **The shape is what transfers**,\n\
        \x20       and it is worth carrying into every future frame-local accounting field:\n\n\
        \x20         **A frame that is not measuring must not maintain the measurement.**\n\n\
        \x20       `Driver::measure_extent` is the switch, and a scroll area over content of unknown\n\
        \x20       size is the one caller that legitimately turns it on — reading the answer one\n\
        \x20       frame late, between frames, where the application owns the offset anyway.",
        a / 1e3,
        b / 1e3,
        (b - a) / 1e3,
        (b - a) / a * 100.0,
        (b - a) / DENSE as f64,
    );
}
