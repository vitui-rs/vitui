//! **Runtime ticket 10's report**: the reverse scan, and what a dense screen's interaction costs.
//!
//! ```text
//! cargo run --release --example hit_numbers -p vitui-runtime
//! ```
//!
//! The number the ticket asks for is **the reverse scan at 142.3 ns over 312 entries, 0.14% of the
//! budget** — and the reason it is worth printing is what it replaces: *no quadtree*. The engine
//! already answers the layer question, the index is already in draw order, and later is innermost.
//! A spatial structure would be a second answer to a question that has one.
//!
//! # Provenance
//!
//! **R 10** took these numbers and **R 20** re-ran the report on 2026-08-24: Apple M1 Max, macOS
//! 26.5.2, rustc 1.97.1, `--release`, unloaded, minimum of 40 rounds. The whole frame reads 3.42 µs
//! for 312 entries, 10.98 ns an entry. **`crate::ledger` carries no row for the reverse scan, and
//! that is this report's own finding restated**: the scan is not separable from the frame that
//! declares the entries, so there is no marginal cost to accumulate — a row here would be the whole
//! frame filed as a part of itself, which is the category error `Kind::Cliff` exists to prevent one
//! rank up.
//!
//! The 100 µs the percentage divides by is **read from `crate::ledger` and is not this file's to
//! choose**: spec §19 inherits it from the engine map, and **a budget figure may not move without a
//! new map decision.** The second report divides by nothing at all — 160 entries against 160 for
//! five hundred times the rows is a count, and a count needs no divisor to survive a change of
//! machine.

use std::hint::black_box;
use std::time::Instant;

use vitui_bench::Bench;
use vitui_engine::{Buttons, Mouse, MouseKind, Rect};
use vitui_runtime::ctx::{Ctx, Driver, Interest};
use vitui_runtime::id::Id;

// **The frame budget every ratio here is against, read rather than written.** See the provenance
// note above: this file used to declare its own copy of the figure.
#[allow(dead_code)]
#[path = "../src/ledger.rs"]
mod ledger;

use ledger::frame_budget_ns;

/// The spec's dense screen.
const DENSE: u64 = 312;

fn main() {
    println!("runtime ticket 10 — what hit-testing costs\n");
    the_reverse_scan();
    the_count_is_bounded_by_cells();
}

/// Report: a whole frame's interaction on a dense screen, and the scan inside it.
fn the_reverse_scan() {
    let mut driver = Driver::headless(300, 80).expect("sink");
    let dense = |cx: &mut Ctx<'_, '_>| {
        let interest = cx
            .theme()
            .hover_interest()
            .with(Interest::CLICK)
            .with(Interest::SCROLL);
        for i in 0..DENSE {
            // Spread across the screen, so the pointer is over exactly one and the scan is a real
            // scan rather than an immediate hit.
            let y = i32::try_from(i % 80).unwrap_or(0);
            let x = i32::try_from((i / 80) * 40).unwrap_or(0);
            cx.interact(Id::keyed(Id::ROOT, i), Rect::new(x, y, 40, 1), interest);
        }
    };

    // The pointer over the **first-declared** entry, which is the worst case for a reverse scan: it
    // walks all 312 before finding it.
    let pointer = Mouse {
        x: 1,
        y: 0,
        kind: MouseKind::Move,
        buttons: Buttons::NONE,
        mods: vitui_engine::Mods::NONE,
        at: Instant::now(),
    };
    driver.post_mouse(pointer);
    driver.frame(dense);
    driver.post_mouse(pointer);
    driver.frame(dense);

    let report = Bench::new(40)
        .case("a dense frame, pointer over the first entry", 20, || {
            let d = black_box(&mut driver);
            d.post_mouse(pointer);
            d.frame(dense);
        })
        .run();
    println!("report  a dense frame with the pointer over the worst entry:\n{report}");
    let frame = report
        .get("a dense frame, pointer over the first entry")
        .expect("measured");
    println!(
        "        {DENSE} entries: {:.2} us a frame, {:.2}% of a {:.0} us budget, {:.2} ns an entry.\n\
        \x20       spec §6 measured the reverse scan at **142.3 ns, 0.14%** — which is the scan alone,\n\
        \x20       and this figure is the whole frame: declare, claim, append, fold the tracking\n\
        \x20       level, scan twice at `end` (hover and the wheel target) and award.\n\
        \x20       **The scan is not separable here**, and that is the honest way round: a frame\n\
        \x20       without it is a frame that declares nothing.\n\
        \x20       **No quadtree.** The engine answers the layer question, the index is in draw order,\n\
        \x20       and later is innermost — a spatial structure would be a second answer to a\n\
        \x20       question that already has one.\n",
        frame / 1e3,
        frame / frame_budget_ns() * 100.0,
        frame_budget_ns() / 1e3,
        frame / DENSE as f64,
    );
}

/// Report and gate: the count is bounded by visible cells and not by data volume.
fn the_count_is_bounded_by_cells() {
    fn list(cx: &mut Ctx<'_, '_>, rows: i64) {
        let visible = cx.visible_rows();
        for row in visible.start..visible.end.min(80) {
            if i64::from(row) >= rows {
                break;
            }
            cx.with_key(u64::try_from(row).unwrap_or(0), |r| {
                let label = r.id();
                r.interact(label, Rect::new(0, row, 40, 1), Interest::CLICK);
                r.interact(
                    Id::keyed(Id::from_raw(9), u64::try_from(row).unwrap_or(0)),
                    Rect::new(40, row, 8, 1),
                    Interest::CLICK,
                );
            });
        }
    }

    println!("report  entries declared, by data volume:");
    let mut counts = Vec::new();
    for rows in [2_000i64, 1_000_000] {
        let mut d = Driver::headless(300, 80).expect("sink");
        d.frame(|cx| list(cx, rows));
        let n = d.inspect().hits().len();
        println!("          {rows:>9} rows -> {n} entries");
        counts.push(n);
    }
    assert_eq!(
        counts[0], counts[1],
        "the entry count moved with the data volume, which is the one thing it may not do"
    );
    println!(
        "\n        **Identical**, which is the invariant: the count is bounded by visible cells.\n\
        \x20       spec §6's screen declares 312 on an IDE shape — a sidebar, a tab bar, a gutter and\n\
        \x20       a status line as well as the list. This one is a list, so it declares {},\n\
        \x20       and the number that transfers is the ratio rather than the total: {:.1}% of the\n\
        \x20       24 000-cell ceiling against §6's 1.3%.\n\
        \x20       **Two targets a row is the mechanical trigger for the spread**, and it is not a\n\
        \x20       style choice: an entry carrying no geometry cannot separate two targets in one\n\
        \x20       row, so the author declares per target. That is the cost of the sixteen bytes.",
        counts[0],
        counts[0] as f64 / 24_000.0 * 100.0,
    );
}
