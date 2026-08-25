//! **The forest: a million nodes at depth 59 999, and a fold over 349 524 rows, as numbers.**
//!
//! Components ticket 16. The convention is the runtime's — a file in `examples/` named
//! `<subject>_numbers.rs` that prints the numbers a human reads — and so is the rule about what an
//! example may be: **`cargo test` does not run this file**, and no row of
//! [`vitui_components::gates::REGISTER`] rests on it. The rows that *cite* it each name a `#[test]`
//! in `src/forest.rs` beside the citation.
//!
//! # What it prints
//!
//! 1. **The two scenes**, with where each stands and which ticket inverts it.
//! 2. **The frame**, flat at 1k / 100k / 1M nodes and at depth 10 and 59 999, with §7's remembered
//!    row beside it.
//! 3. **The verb comparison**, one rectangle drawn three ways, with §6's remembered 418 / 640 /
//!    2 497 beside it and the decomposition that makes the difference a screen rather than a
//!    mechanism.
//! 4. **The trap**, in both directions: what the unclamped indent asks for, what every other counter
//!    says about it, and what it costs on the clock.
//! 5. **The fold**, splice against permutation, with the round trip on both spellings of the splice.
//! 6. **What does not reproduce**, said out loud rather than engineered away.
//!
//! # It asserts the shape and not the timings
//!
//! R15's rule, inherited: *a gate is a count, a ratio, an equality or a compile outcome; a timing is
//! a report*. The timings below are printed and one of them is the whole point — **the defective arm
//! is faster** — which is exactly why no gate is written on one.

use std::time::Duration;

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::forest::{
    self, DEEP, Drawn, Flat, Forest, Indent, Plan, Policy, REMEMBERED_VERBS, Runs, SELECTED,
    SHALLOW, Straddle, VERBS_A_ROW,
};
use vitui_components::gates::Standing;
use vitui_components::scenes::{SCENES, scenes_for};

// **The probe, because `allocations` is a total and a total needs something that counts.**
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// How many frames a per-frame figure is averaged over.
const FRAMES: u32 = 16;

fn main() {
    println!(
        "The forest — {}x{}, one tree, {} rows a window, and the two scenes `tree` is a screen of\n",
        forest::W,
        forest::H,
        forest::H
    );

    scene_list();
    the_frame();
    the_verb_comparison();
    the_trap();
    the_fold();
    what_does_not_reproduce();
}

/// 1. The two scenes, and the ticket that inverts them.
fn scene_list() {
    println!("report  the two scenes, and what each is waiting for:");
    println!(
        "  {:>3}  {:<48}  {:<16}  inverted by",
        "#", "scene", "standing"
    );
    let mut red = 0usize;
    for scene in scenes_for("tree") {
        let (word, ticket) = match scene.standing {
            Standing::Red { inverted_by, .. } => {
                red += 1;
                ("red, pinned", inverted_by)
            }
            Standing::Evaluated { .. } => ("evaluated", "-"),
            Standing::Unsubjected { inverted_by } => ("unsubjected", inverted_by),
            Standing::Unreachable { inverted_by, .. } => ("unreachable", inverted_by),
        };
        println!(
            "  {:>3}  {:<48}  {:<16}  {ticket}",
            scene.number, scene.name, word
        );
    }
    assert_eq!(red, 2, "both are pinned red");
    assert_eq!(
        SCENES.iter().filter(|s| s.stands.contains(&"tree")).count(),
        2
    );
    println!(
        "\n  Both are waiting for the same thing, so there is one pin and not two. The unclamped\n  \
         indent is a *negative case* stood up so the instrument can be watched catching it, and a\n  \
         negative case is not a red gate.\n"
    );
}

/// 2. The frame, flat across volume and across depth.
fn the_frame() {
    println!(
        "report  the frame — §7 states 48.79 us / 0 marked / 23 030 writes / 640 verbs / 59 regions / 5 stops:"
    );
    println!(
        "  {:<34}  {:>9}  {:>9}  {:>7}  {:>8}  {:>6}  {:>5}  {:>10}",
        "at", "writes", "asked", "verbs", "distinct", "region", "stops", "us/frame"
    );

    for (rows, shape) in forest::across_volumes(Plan::tree_at(SHALLOW)) {
        row(
            &format!("{rows} nodes, depth {SHALLOW}"),
            shape,
            forest::frame_cost(Plan::tree_at(SHALLOW).over(rows), FRAMES),
        );
    }
    let deep = Plan::tree_at(DEEP);
    row(
        &format!("1 000 000 nodes, depth {DEEP}"),
        forest::frame(deep),
        forest::frame_cost(deep, FRAMES),
    );

    println!(
        "\n  marked is `unreachable` and is never printed as 0 (`crate::counters`). Allocations over\n  \
         {FRAMES} frames, as a total and never as a mean: {}. The driver is warmed **outside** the\n  \
         bracket, because a total that counts attaching a terminal cannot see zero.\n",
        allocations(deep)
    );
}

/// One line of the frame table.
fn row(label: &str, shape: forest::Shape, cost: Duration) {
    println!(
        "  {label:<34}  {:>9}  {:>9}  {:>7}  {:>8}  {:>6}  {:>5}  {:>10.2}",
        shape.writes,
        shape.asked,
        shape.verbs,
        shape.distinct,
        shape.regions,
        shape.stops,
        cost.as_secs_f64() * 1e6
    );
}

/// How many allocations [`FRAMES`] frames of `plan` cost, as a total, with the driver warmed
/// outside the bracket.
fn allocations(plan: Plan) -> u64 {
    let mut frames = forest::Frames::warmed(plan);
    let (_, allocs) = count_allocations(|| {
        for _ in 0..FRAMES {
            frames.draw();
        }
    });
    allocs as u64
}

/// 3. One rectangle, drawn three ways, and verbs as the currency.
fn the_verb_comparison() {
    let (list, tree, table) = forest::verbs_by_drawing(SHALLOW);
    let plan = Plan::tree_at(SHALLOW);
    println!("report  §6's headline — the same rectangle, the same cells, drawn three ways:");
    println!(
        "  {:<24}  {:>7}  {:>11}  {:>12}  {:>10}  {:>22}",
        "drawn as", "verbs", "a row here", "§6 remembers", "us/frame", "§6 minus the furniture"
    );
    for (name, drawn, measured, a_row, remembered) in [
        (
            "a plain list",
            Drawn::List,
            list,
            VERBS_A_ROW.0,
            REMEMBERED_VERBS.0,
        ),
        (
            "a tree",
            Drawn::Tree,
            tree,
            VERBS_A_ROW.1,
            REMEMBERED_VERBS.1,
        ),
        (
            "a twelve-column table",
            Drawn::Table,
            table,
            VERBS_A_ROW.2,
            REMEMBERED_VERBS.2,
        ),
    ] {
        println!(
            "  {name:<24}  {measured:>7}  {a_row:>11}  {remembered:>12}  {:>10.2}  {:>22}",
            forest::frame_cost(plan.as_drawn(drawn), FRAMES).as_secs_f64() * 1e6,
            format!("{} over 111 rows", remembered as i64 - 196)
        );
    }
    assert_eq!(tree - list, 2 * u64::from(forest::H));
    println!(
        "\n  §7: *the `+` costs two verbs a row — the indent run and the chevron cell, 222 verbs over\n  \
         111 rows*. 640 - 418 is 222 and 222 / 111 is 2, so the remembered pair decomposes as\n  \
         111 x 2 + 196 and 111 x 4 + 196 — one screen's furniture, subtracted once. **The per-row\n  \
         figure reproduces exactly and the absolute one is a screen this ticket does not own.**\n\n  \
         **And the us column is the finding beside it.** §6 prices the three drawings in time as\n  \
         well as in verbs — 48.79 for a tree against 165.6 for a twelve-column table, 3.4x for the\n  \
         same 23 030 cells. Here **twelve times the verbs costs the same frame**: 160, 320 and\n  \
         1 920 verbs over one rectangle land within 1% of each other. On this runtime the verb\n  \
         count is a *structural* currency and not a timing one — which is R15's rule arriving from\n  \
         the other side: **a gate is a count, and a timing is a report.** A gate written on the\n  \
         timing here would report the tree and the table as one construction.\n"
    );
}

/// 4. The trap, in both directions.
fn the_trap() {
    let (clamped, broken) = forest::the_numbers(DEEP);
    let clamped_cost = forest::frame_cost(Plan::tree_at(DEEP), FRAMES);
    let broken_cost = forest::frame_cost(Plan::tree_at(DEEP).unclamped(), FRAMES);

    println!("report  the trap — one flag, at depth {DEEP}:");
    println!(
        "  {:<14}  {:>10}  {:>10}  {:>9}  {:>7}  {:>8}  {:>6}  {:>10}",
        "indent", "asked", "tallied", "writes", "verbs", "distinct", "region", "us/frame"
    );
    for (kind, shape, cost) in [
        (Indent::Clamped, clamped, clamped_cost),
        (Indent::Unclamped, broken, broken_cost),
    ] {
        println!(
            "  {:<14}  {:>10}  {:>10}  {:>9}  {:>7}  {:>8}  {:>6}  {:>10.2}",
            kind.word(),
            shape.asked,
            shape.tallied_ask,
            shape.writes,
            shape.verbs,
            shape.distinct,
            shape.regions,
            cost.as_secs_f64() * 1e6
        );
    }

    let faster = if broken_cost < clamped_cost {
        format!(
            "{:.1}% FASTER",
            (1.0 - broken_cost.as_secs_f64() / clamped_cost.as_secs_f64()) * 100.0
        )
    } else {
        format!(
            "{:.1}% slower",
            (broken_cost.as_secs_f64() / clamped_cost.as_secs_f64() - 1.0) * 100.0
        )
    };
    println!(
        "\n  The defective arm is {faster} here; §7 remembers 18% faster. **The direction reproduces\n  \
         every run and the magnitude does not, and the difference makes the scene stronger rather\n  \
         than weaker**: on this screen both arms write the same 300 cells a row through the same\n  \
         verb, so what the collapse saves is two short verbs and not a label render — the clock is\n  \
         *less* able to see this defect here than it was on the prototype. It writes the same cells,\n  \
         touches the same cells, declares the same regions and the same stops, and makes {} verbs\n  \
         against {}. **Cells asked for is the only counter that moves**, and the tally's own\n  \
         version of it saturates at 65 535 a verb — see `crate::forest`'s header.",
        broken.verbs, clamped.verbs
    );

    let (shallow, shallow_broken) = forest::the_numbers(SHALLOW);
    println!(
        "  At depth {SHALLOW} the same flag is invisible: {} against {} asked, {} against {} verbs.\n  \
         **That is a statement about the scene list, not about the gate** — removing the deep scene\n  \
         removes the ability to distinguish the two builds at all.\n",
        shallow.asked, shallow_broken.asked, shallow.verbs, shallow_broken.verbs
    );
}

/// 5. The fold: splice against permutation, and the round trip.
fn the_fold() {
    let forest_ = Forest::folded();
    let (spliced, permuted, splice_took, permutation_took) = forest::reconciled(&forest_);

    println!(
        "report  scene 9 — a fold over {} rows of a million:",
        forest::FOLD_ROWS
    );
    println!(
        "  {:<34}  {:>10}  {:>10}  {:>12}  {:>12}",
        "reconciled as", "runs after", "rows after", "KB", "us"
    );
    for (name, r, took) in [
        ("an interval (the fold performed it)", spliced, splice_took),
        (
            "a permutation (the caller re-sorted)",
            permuted,
            permutation_took,
        ),
    ] {
        println!(
            "  {name:<34}  {:>10}  {:>10}  {:>12.1}  {:>12.2}",
            r.runs,
            r.rows,
            r.bytes as f64 / 1024.0,
            took.as_secs_f64() * 1e6
        );
    }
    println!(
        "\n  §21 remembers 1 run / 0.04 us / 0 KB against 297 180 / 24 338 us / 8 549.7 KB. **The 1\n  \
         and the 0 KB reproduce exactly; the shattered count does not and cannot** — it is a\n  \
         property of that forest's key shuffle, not of the mechanism. A contiguous half of a\n  \
         million rows through a uniform shuffle breaks into about 250 000 runs, and this one breaks\n  \
         into {}.\n",
        permuted.runs
    );

    println!("report  the fold/unfold round trip, on both spellings of the splice:");
    println!("  {:<44}  {:>12}", "splice", "rows back");
    for straddle in [Straddle::Split, Straddle::Forget] {
        let word = match straddle {
            Straddle::Split => "splits a run straddling the edit (the rule)",
            Straddle::Forget => "shifts `start >= at` only (what shipped)",
        };
        println!(
            "  {word:<44}  {:>12}",
            forest::round_trip(&forest_, straddle)
        );
    }

    let mut flat = Flat::expanded(&forest_);
    let mut selection = Runs::of(0, SELECTED);
    let edit = flat.fold(forest::FOLD_AT);
    let parked = selection.splice(edit, Policy::Stash, Straddle::Split);
    println!(
        "\n  {SELECTED} selected, {} kept under `Drop`, {} parked in {} `Run` of {} bytes under\n  \
         `Stash`. The defective splice comes back with {} rows selected, one run before and one run\n  \
         after, a timing that does not move and a screen that is perfectly correct — which is why\n  \
         the equality is the only instrument that reports it.\n",
        selection.rows(),
        parked.rows(),
        parked.count(),
        std::mem::size_of::<forest::Run>() * parked.count(),
        forest::ROUND_TRIP_WRONG,
    );
}

/// 6. What does not reproduce, said out loud.
fn what_does_not_reproduce() {
    println!("report  what does not reproduce, and why:");
    for (figure, why) in [
        (
            "3 812 970 cells asked for, 165x",
            "the prototype's screen is two tree panels of 111 rows plus a menu bar, a toolbar and \
             a status bar; this one is one tree filling one terminal. 9 599 840 against 24 000, \
             399.99x — and its own 3 812 970 carries `depth * 2 as u16`, which truncates 119 998 \
             to 54 462",
        ),
        (
            "418 / 640 / 2 497 verbs",
            "the same screen. The per-row figures — 2, 4 and the `+ 2` between them — reproduce \
             exactly, and the remembered pair decomposes as 111 rows plus 196 verbs of furniture",
        ),
        (
            "23 030 writes / 59 regions / 5 stops",
            "likewise: 24 000 / 81 / 1 here, one tree over one terminal rather than two panels \
             inside a chrome",
        ),
        (
            "297 180 runs after a permutation",
            "a property of that forest's key shuffle. What is asserted is the side of the cliff — \
             one run against hundreds of thousands",
        ),
        (
            "18% faster",
            "a timing, and it is a report on both maps. The direction is what the scene is about \
             and the direction is asserted nowhere: the gate is the ask",
        ),
        (
            "unchanged at depth 59 999",
            "true of writes, asked, distinct, regions, stops and rows iterated, and not of verbs: \
             the trailing pad has nothing to pad. The prototype's own table says 634 against 640",
        ),
        (
            "48.79 / 165.6 us for a tree against a twelve-column table",
            "3.4x there and 1.00x here. One rectangle drawn with 160, 320 and 1 920 verbs costs \
             the same frame on this runtime — so the verb count is what §6 says it is, a currency \
             for the frame's *structure*, and it is not one for the frame's time",
        ),
    ] {
        println!("  {figure}\n    {why}\n");
    }
}
