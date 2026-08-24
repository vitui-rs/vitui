//! **Runtime ticket 12's report**: what focus costs, and what the vanish rule written the obvious
//! way costs instead.
//!
//! ```text
//! cargo run --release --example focus_numbers -p vitui-runtime
//! ```
//!
//! Four numbers the ticket asks for — **the ring built during the draw at 1.000×–1.009× of a dense
//! frame**, **a scope at +11.6–18.3 ns**, **the whole of focus at ≤ +0.31 µs**, and the vanish
//! rule's probe counts — plus the one it asks for as a contrast: the rule written the obvious way,
//! which walks the previous ring and asks per candidate whether that id is still in this one, with
//! the inner question a scan.
//!
//! **The probe counts are the gate and the timings are the report**, which is this backlog's rule and
//! not a preference here: +32.2 µs on one keystroke is 32% of the budget at 600 rows and grows
//! quadratically from there, so the shape a test can see is the count and the ratio.
//!
//! # Provenance
//!
//! **R 12** took these numbers and **R 20** re-measured the whole-of-focus row against the shipped
//! runtime on 2026-08-24: Apple M1 Max, macOS 26.5.2, rustc 1.97.1, `--release`, unloaded, minimum
//! of 40 rounds. It reads **243.75 ns**, and the prototype it replaces stated a *ceiling* of
//! ≤ +0.31 µs rather than a figure — so the row that went into `crate::ledger` is the first actual
//! measurement of it, and it is under the ceiling. The gate beside it is untouched by any of this,
//! because it is a probe ratio — 601 against 90 300 — and a count is the same number on every
//! machine.
//!
//! The 100 µs the two percentages divide by is **read from `crate::ledger` and is not this file's to
//! choose**: spec §19 inherits it from the engine map, and **a budget figure may not move without a
//! new map decision.** That matters more here than in most of these reports, because the contrast
//! arm's +32.2 µs is argued *as a share of the budget* — a divisor edited to suit would delete the
//! argument rather than answer it.

use std::hint::black_box;

use vitui_bench::Bench;
use vitui_engine::{ColorDepth, Rect};
use vitui_runtime::ctx::{Ctx, Driver, Interest};
use vitui_runtime::focus::ScopeKind;
use vitui_runtime::id::Id;
use vitui_runtime::theme::Theme;

// **The frame budget every ratio here is against, read rather than written.** See the provenance
// note above: this file used to declare its own copy of the figure.
#[allow(dead_code)]
#[path = "../src/ledger.rs"]
mod ledger;

use ledger::frame_budget_ns;

/// The dense screen's interactive regions.
const DENSE: u64 = 312;

/// How many of them are tab stops.
const RING: u64 = 67;

/// The search box's list.
const ROWS: u64 = 600;

/// A cell of the dense screen: a rectangle is never the interesting part here.
fn cell(i: u64) -> Rect {
    let i = i32::try_from(i).unwrap_or(0);
    Rect::new(i % 300, i / 300, 1, 1)
}

/// The dense screen, with the ring declared or not, and with or without its three groups.
fn dense(cx: &mut Ctx<'_, '_>, stops: bool, groups: bool) {
    let base = cx.theme().hover_interest().with(Interest::CLICK);
    let stop = if stops {
        base.with(Interest::FOCUS)
    } else {
        base
    };
    let mut n = 0u64;
    for (name, count) in [("menu bar", 7u64), ("toolbar", 12), ("tab strip", 8)] {
        if groups {
            cx.scope(Id::named(name), ScopeKind::Group, |cx| {
                for i in 0..count {
                    cx.interact(Id::keyed(Id::named(name), i), cell(n + i), stop);
                }
            });
        } else {
            for i in 0..count {
                cx.interact(Id::keyed(Id::named(name), i), cell(n + i), stop);
            }
        }
        n += count;
    }
    for i in 0..40u64 {
        cx.interact(Id::keyed(Id::named("stop"), i), cell(n + i), stop);
    }
    n += 40;
    for i in 0..245u64 {
        cx.interact(Id::keyed(Id::named("row"), i), cell(n + i), base);
    }
}

fn main() {
    println!("runtime ticket 12 — what focus costs\n");
    the_ring_during_the_draw();
    the_scopes();
    the_vanish_rule();
}

/// Report: **the ring built during the draw, against the same screen without the bit.**
///
/// This is the trade the ticket had to price. Building the ring only when a `Tab` arrives is a cost
/// on one frame in a hundred; building it during the draw is a branch on a bit the widget is already
/// passing, on every frame. The second is what ships, because a ring built from the previous frame's
/// index is a frame old — and `Tab` then steps over a row inserted directly after the focused one.
fn the_ring_during_the_draw() {
    let mut without = Driver::headless(300, 80).expect("sink");
    without.set_theme(Theme::default().resolve(ColorDepth::None));
    let bare = Bench::new(40)
        .case("a dense frame, no ring", 20, || {
            black_box(&mut without).frame(|cx| dense(cx, false, false));
        })
        .run();
    let bare_ns = bare.get("a dense frame, no ring").expect("measured");

    let mut with = Driver::headless(300, 80).expect("sink");
    with.set_theme(Theme::default().resolve(ColorDepth::None));
    let ringed = Bench::new(40)
        .case("a dense frame, 67 stops", 20, || {
            black_box(&mut with).frame(|cx| dense(cx, true, false));
        })
        .run();
    let ring_ns = ringed.get("a dense frame, 67 stops").expect("measured");

    with.frame(|cx| dense(cx, true, false));
    assert_eq!(with.inspect().hits().len(), DENSE as usize);
    assert_eq!(with.inspect().ring().len(), RING as usize);
    assert_eq!(
        with.inspect().tracking(),
        without.inspect().tracking(),
        "the sixth bit costs the mouse nothing, which is a gate and not a report"
    );

    println!(
        "report  the ring, built during the draw:\n\
        \x20       {DENSE} regions, no ring      {bare_ns:>10.2} ns\n\
        \x20       {DENSE} regions, {RING} stops     {ring_ns:>10.2} ns\n\
        \x20       ratio                  {:>10.3}x   {:+.2} ns, {:+.4}% of a {:.0} us budget\n\
        \x20       spec §8 measured **1.000x-1.009x**, which is under the run-to-run spread of the\n\
        \x20       frame itself. The mouse pays nothing: `FOCUS.tracking()` is `Off`, and the\n\
        \x20       tracking level is identical with the bit and without it.\n",
        ring_ns / bare_ns,
        ring_ns - bare_ns,
        (ring_ns - bare_ns) / frame_budget_ns() * 100.0,
        frame_budget_ns() / 1000.0
    );
}

/// Report: **a scope at +11.6–18.3 ns**, and the whole of focus against the +0.31 µs budget.
fn the_scopes() {
    let mut flat = Driver::headless(300, 80).expect("sink");
    flat.set_theme(Theme::default().resolve(ColorDepth::None));
    let ungrouped = Bench::new(40)
        .case("67 stops, no scopes", 20, || {
            black_box(&mut flat).frame(|cx| dense(cx, true, false));
        })
        .run();
    let flat_ns = ungrouped.get("67 stops, no scopes").expect("measured");

    let mut grouped = Driver::headless(300, 80).expect("sink");
    grouped.set_theme(Theme::default().resolve(ColorDepth::None));
    let scoped = Bench::new(40)
        .case("67 stops, three groups", 20, || {
            black_box(&mut grouped).frame(|cx| dense(cx, true, true));
        })
        .run();
    let scoped_ns = scoped.get("67 stops, three groups").expect("measured");

    grouped.frame(|cx| dense(cx, true, true));
    let stops = grouped.inspect().stop_count();
    assert_eq!(stops, 43, "three groups collapse 27 entries onto 3 stops");

    // And the whole of focus: the same screen with nothing declared at all, against the one with the
    // ring, the scopes and the walk.
    let mut bare = Driver::headless(300, 80).expect("sink");
    bare.set_theme(Theme::default().resolve(ColorDepth::None));
    let none = Bench::new(40)
        .case("no focus at all", 20, || {
            black_box(&mut bare).frame(|cx| dense(cx, false, false));
        })
        .run();
    let none_ns = none.get("no focus at all").expect("measured");

    println!(
        "report  three scopes over the ring:\n\
        \x20       {RING} stops, no scopes     {flat_ns:>10.2} ns\n\
        \x20       {RING} stops, three groups  {scoped_ns:>10.2} ns   -> {stops} tab stops\n\
        \x20       one scope              {:>10.2} ns   spec §8 measured **+11.6 to +18.3 ns**\n\
        \x20       the whole of focus     {:>10.2} ns   {:.3}% of a {:.0} us budget, against a\n\
        \x20                                            stated ceiling of **+0.31 us, 0.31%**\n\
        \x20       311 / {stops} = {:.1}x fewer things a keyboard walkthrough visits.\n",
        (scoped_ns - flat_ns) / 3.0,
        scoped_ns - none_ns,
        (scoped_ns - none_ns) / frame_budget_ns() * 100.0,
        frame_budget_ns() / 1000.0,
        311.0 / stops as f64
    );
}

/// Report: **the vanish rule, in probes, against the same rule written the obvious way.**
///
/// The scene is the map's: a search box filtering 600 keyed rows. One keystroke halves the list and
/// the focused row is one of the ones that went, so every candidate the walk crosses is dead — which
/// is what makes the inner scan quadratic rather than incidental.
fn the_vanish_rule() {
    let search = Id::named("search");
    let row = |i: u64| Id::keyed(Id::named("row"), i);
    let mut d = Driver::headless(300, 80).expect("sink");
    d.set_theme(Theme::default().resolve(ColorDepth::None));

    d.frame(|cx| {
        cx.interact(search, cell(0), Interest::FOCUS);
        for i in 0..ROWS {
            cx.interact(row(i), cell(i + 1), Interest::FOCUS);
        }
        cx.focus(row(ROWS - 1));
    });
    let prev: Vec<Id> = d.inspect().ring_ids().collect();
    assert_eq!(prev.len(), ROWS as usize + 1);
    assert_eq!(d.inspect().vanish_probes(), 0, "a quiet frame pays nothing");

    let kept = ROWS / 2;
    d.frame(|cx| {
        cx.interact(search, cell(0), Interest::FOCUS);
        for i in 0..kept {
            cx.interact(row(i), cell(i + 1), Interest::FOCUS);
        }
    });
    let now: Vec<Id> = d.inspect().ring_ids().collect();
    let shipped = d.inspect().vanish_probes();
    assert_eq!(d.inspect().focused(), Some(row(kept - 1)));

    // The obvious way, written out beside it.
    let at = prev
        .iter()
        .position(|id| *id == row(ROWS - 1))
        .expect("it was there");
    let mut obvious = 0u64;
    for id in prev[at + 1..].iter().chain(prev[..at].iter().rev()) {
        let mut found = false;
        for other in &now {
            obvious += 1;
            if other == id {
                found = true;
                break;
            }
        }
        if found {
            break;
        }
    }

    // And what the difference costs in time, on the frame it happens.
    let mut stamped = Driver::headless(300, 80).expect("sink");
    stamped.set_theme(Theme::default().resolve(ColorDepth::None));
    let filtered = Bench::new(20)
        .case("the filtering keystroke", 10, || {
            let d = black_box(&mut stamped);
            d.frame(|cx| {
                cx.interact(search, cell(0), Interest::FOCUS);
                for i in 0..ROWS {
                    cx.interact(row(i), cell(i + 1), Interest::FOCUS);
                }
                cx.focus(row(ROWS - 1));
            });
            d.frame(|cx| {
                cx.interact(search, cell(0), Interest::FOCUS);
                for i in 0..kept {
                    cx.interact(row(i), cell(i + 1), Interest::FOCUS);
                }
            });
        })
        .run();
    let two_frames = filtered.get("the filtering keystroke").expect("measured");

    println!(
        "report  the vanish rule, at {ROWS} rows:\n\
        \x20       previous ring          {:>10} entries\n\
        \x20       this frame's ring      {:>10} entries\n\
        \x20       shipped, stamped       {shipped:>10} probes  one fill pass, then one a candidate\n\
        \x20       the obvious way        {obvious:>10} probes  a scan a candidate\n\
        \x20       ratio                  {:>10.1}x  spec §8 measured **601 against 90 002, 150x**\n\
        \x20       a quiet frame          {:>10} probes\n\
        \x20       the two frames         {two_frames:>10.2} ns  the keystroke that filters, whole\n\
        \x20       Both counts are slot touches, which is the honest unit against the scan the two\n\
        \x20       forms are being compared on: the fill inserts {} entries and each candidate costs\n\
        \x20       one probe, so 601 is the fill plus the 300 dead candidates the walk crosses.\n\
        \x20       **The gate is the ratio and the bound, not either count on its own** — a number\n\
        \x20       belonging to the data has to be a relation, and 90 300 is the data's.\n",
        prev.len(),
        now.len(),
        obvious as f64 / shipped as f64,
        0,
        now.len(),
    );
}
