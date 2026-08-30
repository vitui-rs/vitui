//! **The scroll area: the bar fixpoint, `Σ h` as the extent, and two areas far apart, as numbers.**
//!
//! Components ticket 18. The convention is the runtime's — a file in `examples/` named
//! `<subject>_numbers.rs` that prints the numbers a human reads — and so is the rule about what an
//! example may be: **`cargo test` does not run this file**, and no row of
//! [`vitui_components::gates::REGISTER`] rests on it. Rows that cite it name a `#[test]` in
//! `src/area.rs` beside the citation.
//!
//! # What it prints
//!
//! 1. **The four scenes**, with where each stands and which ticket inverts it.
//! 2. **The bar fixpoint** over 5 475 600 pairs, the hysteresis and the reflow loop.
//! 3. **`Σ h` against the row count**: the reachability, and every counter that does *not* move.
//! 4. **The two areas**, under both damage models, with the amplification factor.
//! 5. **The wrong pairing**, as a count and — separately, as a report — as microseconds.
//! 6. **What does not reproduce**, said out loud rather than engineered away. Three of §9's figures
//!    are on that list and each disagreement says something about the mechanism.
//!
//! # It asserts the shape and not the timings
//!
//! R15's rule, inherited: *a gate is a count, a ratio, an equality or a compile outcome; a timing is
//! a report*. Every `assert!` below is a count or an equality, and each is asserted in `src/` too —
//! `examples/gates_numbers.rs` is the precedent for why that matters: it asserted a register length
//! against a register nine rows longer and passed `cargo test` throughout, because an example is
//! compiled and never evaluated.

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::area::{
    self, ADR_CONTESTED, AH, AW, Bars, Content, DRIFT_REACHES_SEVEN_AT, EXTENT_CELLS, EXTENT_ROWS,
    GAP, Hide, LAST_ROW_IN_CELLS, LAST_ROW_IN_ROWS, OVERLAY_REDAMAGE, PAIRING_ROWS, Pairing,
    REFLOW_FLIPS, REFLOW_FRAMES, REMEMBERED_DRIFT, REMEMBERED_PAIRING_US, REMEMBERED_REDAMAGE,
    ROWS, Responsive, SWEEP_PAIRS, Shown, THUMB_DRIFT, Unit,
};
use vitui_components::gates::Standing;
use vitui_components::scenes::{SCENES, scenes_for};
use vitui_components::{Axis, INVENTORY};

// **The probe, because `allocations` is a total and a total needs something that counts.**
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// How many frames a per-frame figure is averaged over.
const FRAMES: u32 = 8;

/// How many frames the wrong pairing is averaged over. **One**, and that is itself the finding.
const SLOW_FRAMES: u32 = 1;

fn main() {
    println!(
        "The scroll area — {AW}x{AH} for the extent, {}x{} for the two areas, and the four scenes \
         `scroll_area`, `scrollbar` and `sticky` are screens of\n",
        area::W,
        area::H
    );

    scene_list();
    the_bar_fixpoint();
    the_extent();
    the_two_areas();
    let pairing_us = the_wrong_pairing();
    the_shipped_frame();
    o5();
    what_does_not_reproduce(pairing_us);
}

/// 1. The four scenes, and the ticket that inverts each.
fn scene_list() {
    println!("report  the four scenes, and what stands each of them up:");
    println!(
        "  {:>3}  {:<60}  {:<14}  inverted by",
        "#", "scene", "standing"
    );
    let mut evaluated = 0usize;
    for scene in SCENES
        .iter()
        .filter(|s| [17u8, 18, 19, 30].contains(&s.number))
    {
        let (word, ticket) = match scene.standing {
            Standing::Red { inverted_by, .. } => ("red, pinned", inverted_by),
            Standing::Evaluated { .. } => {
                evaluated += 1;
                ("evaluated", "-")
            }
            Standing::Unsubjected { inverted_by } => ("unsubjected", inverted_by),
            Standing::Unreachable { inverted_by, .. } => ("unreachable", inverted_by),
        };
        println!(
            "  {:>3}  {:<60}  {word:<14}  {ticket}",
            scene.number, scene.name
        );
    }
    assert_eq!(
        evaluated, 4,
        "all four were pinned on one fact and components 19 supplied it"
    );
    println!(
        "\n  the subject: {:?}, declared: {:?}\n",
        area::SUBJECTS,
        area::subjects_declared()
    );
}

/// 2. The bar fixpoint, the hysteresis, and the reflow loop.
fn the_bar_fixpoint() {
    let swept = area::sweep();
    println!("gate    the bar fixpoint, over the whole domain rather than a sample:");
    println!(
        "  pairs {:>10}   failures {:>3}   worst passes {:>2}   unneeded bars {:>3}",
        swept.pairs, swept.failures, swept.worst_passes, swept.unneeded
    );
    assert_eq!(swept.pairs, SWEEP_PAIRS);
    assert_eq!(swept.failures, 0);

    let fixpoint = area::fitting(Hide::Fixpoint);
    let incremental = area::fitting(Hide::Incremental);
    println!(
        "\ngate    content that fits with no bars at all, decided two ways ({}x{}):",
        area::FIT_W,
        area::FIT_H
    );
    println!(
        "  {:<14}  bars {:<20}  rows {:>3}  cols {:>3}  last row drawn {}",
        "fixpoint",
        format!("{:?}", fixpoint.shown),
        fixpoint.rows,
        fixpoint.cols,
        fixpoint.last_row_drawn
    );
    println!(
        "  {:<14}  bars {:<20}  rows {:>3}  cols {:>3}  last row drawn {}",
        "incremental",
        format!("{:?}", incremental.shown),
        incremental.rows,
        incremental.cols,
        incremental.last_row_drawn
    );
    println!("  a row and a column, lost permanently, on a screen that looks correct");

    let responsive = Responsive {
        wide: 60,
        tall: 30,
        short: 10,
        natural_w: 40,
    };
    let antitone = Responsive {
        short: 30,
        ..responsive
    };
    println!("\ngate    the reflow loop, over {REFLOW_FRAMES} frames with no input at all:");
    for (what, body, bars) in [
        ("not antitone, reserved", responsive, Bars::Reserved),
        ("not antitone, overlay", responsive, Bars::Overlay),
        ("antitone, reserved", antitone, Bars::Reserved),
    ] {
        let flips = area::flips(&area::oscillates((60, 20), body, REFLOW_FRAMES, bars));
        println!("  {what:<26}  flips {flips:>4}");
    }
    assert_eq!(
        area::flips(&area::oscillates(
            (60, 20),
            responsive,
            REFLOW_FRAMES,
            Bars::Reserved
        )),
        REFLOW_FLIPS
    );
    let boundary = area::decide((60, 20), (60, 20), Bars::Reserved);
    println!(
        "  and the least fixpoint at the boundary: {:?} in {} pass\n",
        boundary.shown, boundary.passes
    );
    assert_eq!(boundary.shown, Shown { v: false, h: false });
}

/// 3. `Σ h` against the row count.
fn the_extent() {
    let content = Content::build(ROWS, true);
    println!(
        "gate    {ROWS} rows, one in {} of them {} cells tall:",
        area::TALL_EVERY,
        area::TALL_H
    );
    println!(
        "  extent in cells {:>9}   extent in rows {:>9}   difference {:>9}",
        content.extent_cells(),
        content.extent_rows(),
        content.extent_cells() - content.extent_rows()
    );
    assert_eq!(content.extent_cells(), EXTENT_CELLS);
    assert_eq!(content.extent_rows(), EXTENT_ROWS);

    let (cells, cells_allocs) = count_allocations(|| area::at_the_end(&content, Unit::Cells));
    let (rows, rows_allocs) = count_allocations(|| area::at_the_end(&content, Unit::Rows));
    println!("\nreport  the two builds at each one's own furthest offset:");
    println!(
        "  {:<8}  {:>10}  {:>8}  {:>7}  {:>8}  {:>7}  {:>9}  {:>10}  {:>8}",
        "unit",
        "offset",
        "last row",
        "writes",
        "distinct",
        "verbs",
        "regions",
        "iterated",
        "allocs"
    );
    for (unit, shape, allocs) in [
        (Unit::Cells, cells, cells_allocs),
        (Unit::Rows, rows, rows_allocs),
    ] {
        println!(
            "  {:<8}  {:>10}  {:>8}  {:>7}  {:>8}  {:>7}  {:>9}  {:>10}  {:>8}",
            unit.word(),
            shape.offset,
            shape.last_row,
            shape.writes,
            shape.distinct,
            shape.verbs,
            shape.regions,
            shape.iterated,
            allocs
        );
    }
    assert_eq!(cells.last_row, LAST_ROW_IN_CELLS);
    assert_eq!(rows.last_row, LAST_ROW_IN_ROWS);
    println!(
        "  the allocation total is the whole `at_the_end` call — the driver attach, the theme \
         rebuild\n  and the `Tally`'s own set of every cell it saw. **The steady-frame zero is a \
         component's\n  claim and it belongs to components 19**; what this column is for here is \
         that the two\n  builds agree on it too, which is one more counter that does not separate \
         them."
    );
    println!(
        "  every counter identical; {} of {ROWS} rows unreachable, which is {}%",
        LAST_ROW_IN_CELLS - LAST_ROW_IN_ROWS,
        100 * (LAST_ROW_IN_CELLS - LAST_ROW_IN_ROWS) / ROWS
    );
    println!(
        "  counters that separate them: {:?}",
        area::counters_that_separate_them(500_000)
    );

    println!("\nreport  the thumb, in a track of {AH} cells:");
    println!(
        "  worst drift {:>3} cells   §9's *up to {REMEMBERED_DRIFT}* is reached at {:.1}% of the \
         reachable range",
        area::thumb_drift(),
        area::drift_reaches(REMEMBERED_DRIFT).unwrap_or(f64::NAN) * 100.0
    );
    assert_eq!(area::thumb_drift(), THUMB_DRIFT);
    println!();
}

/// 4. The two areas, under both damage models.
fn the_two_areas() {
    let reserved = area::steady(Bars::Reserved);
    let overlay = area::steady(Bars::Overlay);
    println!(
        "gate    two scroll areas {}x{} each, {GAP} columns apart, on a {}x{} screen:",
        area::AREA_W,
        area::H,
        area::W,
        area::H
    );
    println!(
        "  {:<12}  {:>12}  {:>14}  {:>14}",
        "bars", "re-damaged", "under spans", "amplification"
    );
    for (what, damage) in [("reserved", reserved), ("overlay", overlay)] {
        println!(
            "  {what:<12}  {:>12}  {:>14}  {:>13.2}x",
            damage.cells,
            damage.span_cells,
            damage.amplification()
        );
    }
    assert_eq!(reserved.cells, 0);
    assert_eq!(overlay.cells, OVERLAY_REDAMAGE);
    println!(
        "  the shipped structure is a per-row bitset and charges {} — 1.00x, exact by construction",
        overlay.cells
    );
    println!(
        "  §9 and ADR 0029 remember {REMEMBERED_REDAMAGE}, which is ~{ADR_CONTESTED} x 8.36 under \
         the model the engine rejected\n"
    );
}

/// 5. The wrong pairing: a count, and separately a timing.
///
/// Returns the microsecond figure so that the closing table prints **the same measurement** rather
/// than a second one of the same quantity — two calls report two different numbers, which is §21's
/// first refinement arriving on a table instead of on a gate.
fn the_wrong_pairing() -> f64 {
    let area_shape = area::pairing_shape(Pairing::Area, PAIRING_ROWS);
    let collection = area::pairing_shape(Pairing::Collection, PAIRING_ROWS);
    println!(
        "gate    the wrong pairing at {PAIRING_ROWS} rows, as the count that is the mechanism:"
    );
    println!(
        "  {:<14}  {:>10}  {:>8}  {:>8}",
        "pairing", "iterated", "verbs", "writes"
    );
    for (kind, shape) in [
        (Pairing::Area, area_shape),
        (Pairing::Collection, collection),
    ] {
        println!(
            "  {:<14}  {:>10}  {:>8}  {:>8}",
            kind.word(),
            shape.iterated,
            shape.verbs,
            shape.writes
        );
    }
    assert_eq!(area_shape.iterated, PAIRING_ROWS);
    assert_eq!(collection.iterated, u64::from(AH));
    assert_eq!(area_shape.writes, collection.writes);

    let slow = area::pairing_cost(Pairing::Area, PAIRING_ROWS, SLOW_FRAMES);
    let fast = area::pairing_cost(Pairing::Collection, PAIRING_ROWS, FRAMES);
    println!("\nreport  and as the timing §9 states, which is not a gate:");
    println!(
        "  {:<14}  {:>12.2} us   {:<14}  {:>12.2} us",
        "scroll_area",
        slow.as_secs_f64() * 1e6,
        "collection",
        fast.as_secs_f64() * 1e6
    );
    println!(
        "  §9 remembers {REMEMBERED_PAIRING_US:.0} us for the same pairing on a screen this ticket \
         does not own\n"
    );
    slow.as_secs_f64() * 1e6
}

/// 6. The shipped frame, `row_at`, and the shape change — components ticket 19's own table.
fn the_shipped_frame() {
    println!(
        "report  the shipped `scroll_area` at {}x{}, both axes, a sticky header, a virtualising \
         body:",
        area::W,
        area::H
    );
    println!(
        "  {:<9}  {:<9}  {:>9}  {:>7}  {:>8}  {:>5}  {:>7}  {:>5}  {:>8}",
        "rows",
        "heights",
        "us/frame",
        "writes",
        "distinct",
        "verbs",
        "regions",
        "stops",
        "iterated"
    );
    let mut shapes = Vec::new();
    for (rows, variable) in [(1_000u64, true), (ROWS, true), (ROWS, false)] {
        let (shape, cost) = area::shipped_frame(rows, variable, FRAMES);
        println!(
            "  {rows:<9}  {:<9}  {:>9.2}  {:>7}  {:>8}  {:>5}  {:>7}  {:>5}  {:>8}",
            if variable { "variable" } else { "uniform" },
            cost.as_secs_f64() * 1e6,
            shape.writes,
            shape.distinct,
            shape.verbs,
            shape.regions,
            shape.stops,
            shape.iterated
        );
        shapes.push(shape);
    }
    assert_eq!(
        (shapes[0].writes, shapes[0].verbs),
        (shapes[1].writes, shapes[1].verbs),
        "a thousand rows and a million are the same frame"
    );
    assert_eq!(
        shapes[1].writes, shapes[1].distinct,
        "the component writes a partition of the screen"
    );
    println!(
        "  the frame is a partition of the whole screen — writes == distinct == {} — and it is \
         the\n  full-screen class, so the budget it is under is 1 ms and not 100 us. The three \
         regions are\n  the area and its two bars: the four bands declare nothing.",
        shapes[1].distinct
    );

    println!(
        "\nreport  `row_at` over {ROWS} rows, which a frame pays once, and the shape change \
         `{}` rows:",
        area::REMOVED
    );
    println!(
        "  row_at   variable {:>9.4} us   uniform {:>9.4} us",
        area::row_at_cost(true, 512).as_secs_f64() * 1e6,
        area::row_at_cost(false, 512).as_secs_f64() * 1e6
    );
    println!(
        "  reshape  variable {:>9.1} us   uniform {:>9.1} us",
        area::shape_change_cost(true, 5).as_secs_f64() * 1e6,
        area::shape_change_cost(false, 5).as_secs_f64() * 1e6
    );
    println!(
        "  the uniform arm has no prefix sum to rebuild and the variable arm does, which is the \
         whole\n  of the difference. The offset clamp is free either way: `max` is recomputed \
         every frame.\n"
    );
}

/// 7. O5's three rows for this family.
fn o5() {
    println!("report  O5, for the three components of F3 scrolling:");
    for component in INVENTORY.iter().filter(|c| area::SUBJECTS.contains(&c.id)) {
        for axis in Axis::ALL {
            if !component.declares(axis) {
                continue;
            }
            let scenes: Vec<u8> = vitui_components::scenes::SCENES
                .iter()
                .filter(|s| {
                    !s.owed
                        && s.covers
                            .iter()
                            .any(|(id, a)| *id == component.id && *a == axis)
                })
                .map(|s| s.number)
                .collect();
            println!(
                "  {:<12}  {:<9}  scenes {:?}",
                component.id,
                axis.name(),
                scenes
            );
        }
    }
    for id in area::SUBJECTS {
        println!(
            "  scenes_for({id:?}) = {:?}",
            scenes_for(id).map(|s| s.number).collect::<Vec<_>>()
        );
    }
    println!();
}

/// 8. What does not reproduce, and what each disagreement says.
fn what_does_not_reproduce(pairing_us: f64) {
    println!("finding what §9 remembers, against what this screen measures:");
    println!(
        "  {:<34}  {:>12}  {:>12}",
        "figure", "remembered", "measured"
    );
    println!(
        "  {:<34}  {:>12}  {:>12}",
        "cells re-damaged, overlay bars", REMEMBERED_REDAMAGE, OVERLAY_REDAMAGE
    );
    println!(
        "  {:<34}  {:>12}  {:>12}",
        "thumb drift, worst", REMEMBERED_DRIFT, THUMB_DRIFT
    );
    println!(
        "  {:<34}  {:>12}  {:>12}",
        "the wrong pairing, us", REMEMBERED_PAIRING_US as u64, pairing_us as u64
    );
    println!(
        "\n  1. {REMEMBERED_REDAMAGE} is a *damage* figure and not a double write. ADR 0029 states \
         both numbers: `~{ADR_CONTESTED}` cells contested, x8.4 under one span per surface row. \
         {ADR_CONTESTED} x 8.36 is {REMEMBERED_REDAMAGE}. The engine measured that model and \
         rejected it — `damage.rs` ships a per-row bitset, 1.00x everywhere — so the measurable \
         quantity is the double write, and this screen's is {OVERLAY_REDAMAGE}."
    );
    println!(
        "  2. The drift is linear in the offset, so *up to {REMEMBERED_DRIFT}* is a point on the \
         range ({:.1}% of it) and not its end. The arithmetic reproduces; the phrase does not.",
        DRIFT_REACHES_SEVEN_AT * 100.0
    );
    println!(
        "  3. {REMEMBERED_PAIRING_US:.0} us is a figure about a screen this ticket does not own, \
         and the gap is the *row* rather than the loop: this body stages a {AW}-column run per \
         row, so the constant is the run's width and the machine is shared with nine other builds \
         besides. What reproduces exactly is the mechanism the runtime's own `scroll.rs` says to \
         measure — {PAIRING_ROWS} rows iterated against {AH} — which is why the gate is on \
         `iterated` and the microseconds are a report."
    );
    println!(
        "\n  and two defects found rather than remembered. The first is `Ctx::scroll_scope`, \
         which\n  translated the content the wrong way ({}) and was invisible at offset 0 — every \
         offset\n  anything above the runtime had used. The second is the instrument's own, and \
         components 19\n  found it: `Tally::distinct` and `Pen` both unioned in the coordinates \
         each verb was\n  *called* in, so a scroll area with a sticky header reported 299 double \
         writes on a frame\n  that has none, and two areas sixty columns apart recorded their \
         bands as one. The repair is\n  `vitui_runtime::Ctx::origin` — runtime architecture issue \
         32 — and it struck one of the two\n  grounds `crate::frame` refused a closure-taking \
         `block` on. The other one stands.",
        area::SETTLED_SCROLL_SCOPE_SIGN
    );
}
