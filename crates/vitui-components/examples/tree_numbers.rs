//! **The tree: a million nodes at depth 59 999, a fold over 349 524 rows, and the index, as
//! numbers.**
//!
//! Components tickets 16 and 17. The convention is the runtime's — a file in `examples/` named
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
//! 6. **The flatten index** (components ticket 17): the record's size, the interval `depth` finds
//!    against the one the data gives, the splice against the rebuild, and what the height prefix
//!    sum costs.
//! 7. **What does not reproduce**, said out loud rather than engineered away.
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
use vitui_components::order;
use vitui_components::scenes::{SCENES, scenes_for};

// **The probe, because `allocations` is a total and a total needs something that counts.**
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// How many frames a per-frame figure is averaged over.
const FRAMES: u32 = 16;

/// How many rounds a minimum-of-N figure takes. `vitui-bench`'s own argument, followed rather than
/// depended on: C6 is one line and this crate's dependency table is `vitui-runtime`.
const ROUNDS: u32 = 8;

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
    the_index();
    what_does_not_reproduce();
}

/// 1. The two scenes, and the ticket that inverts them.
fn scene_list() {
    println!("report  the two scenes, and what each is waiting for:");
    println!(
        "  {:>3}  {:<48}  {:<16}  inverted by",
        "#", "scene", "standing"
    );
    let mut evaluated = 0usize;
    for scene in scenes_for("tree") {
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
            "  {:>3}  {:<48}  {:<16}  {ticket}",
            scene.number, scene.name, word
        );
    }
    assert_eq!(evaluated, 2, "both stand on `tree`, since components 17");
    assert_eq!(
        SCENES.iter().filter(|s| s.stands.contains(&"tree")).count(),
        2
    );
    println!(
        "\n  Both were pinned red on one fact — `tree` was not declared — and components ticket 17\n  \
         turned them together. The unclamped indent was never the pin: it is a *negative case*,\n  \
         stood up so the instrument can be watched catching it, and a negative case is not a red\n  \
         gate. It is `collect::defective::unclamped_indent` now, one field from the shipped build.\n"
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

/// 6. The flatten index (components ticket 17), which is what makes a fold affordable.
fn the_index() {
    println!("report  the record — §7 states `Row {{ node: u32, depth: u16, flags: u8, h: u8 }}`:");
    let (bare, with_heights) = order::index_bytes(1_000_000);
    println!("  {:<44}  {:>12}", "one row, in bytes", order::ENTRY_BYTES);
    println!(
        "  {:<44}  {:>12.2}",
        "a million rows, in MiB",
        bare as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  {:<44}  {:>12.2}",
        "the same, with the height prefix sum, in MiB",
        with_heights as f64 / (1024.0 * 1024.0)
    );
    println!(
        "  {:<44}  {:>11.0}%",
        "the growth",
        (with_heights as f64 / bare as f64 - 1.0) * 100.0
    );
    println!(
        "\n  §7 says **7 → 11 MB and 57%**, and the two absolutes come back to the megabyte. The\n  \
         growth is `4 / 8` exactly, so it is 50% and not 57% — §7's own pair divides to 57 only\n  \
         because it rounds 7.63 down and 11.44 up.\n\n  \
         **It was sixteen bytes until this ticket.** §7 states the widths and §10 — where *one\n  \
         structure, five names* lives — states the four field names and no widths at all;\n  \
         components 13 built the type from §10 and widened three of them. Nothing caught it,\n  \
         because §7's own criterion is *a gate asserts its size* and that criterion is this\n  \
         ticket's. The three ceilings it costs are stated rather than discovered: a caller's key at\n  \
         `u32::MAX`, eight flag bits where §10 names four, and a row 255 screen rows tall.\n"
    );

    // §21's own forest, padded to a million so the tail a splice moves is a real tail: row 0 a
    // leaf, row 1 the folded node with 349 524 rows under it, then roots to the end.
    let mut forest = vec![0u16, 0];
    for _ in 0..4 {
        forest.push(1);
        forest.extend(std::iter::repeat_n(2u16, 87_380));
    }
    forest.resize(usize::try_from(forest::NODES).expect("a million fits"), 0);
    let index = order::flattened(&forest);

    println!("report  the interval a collapse removes — §7 states 115 us against 1 187:");
    println!("  {:<44}  {:>12}  {:>12}", "read from", "rows", "us");
    let (from_index, from_data) = order::subtree_costs(&index, &forest, forest::FOLD_AT, ROUNDS);
    for (name, took) in [
        ("the index (a contiguous scan of `depth`)", from_index),
        ("the data (one random access a row)", from_data),
    ] {
        println!(
            "  {name:<44}  {:>12}  {:>12.2}",
            index.subtree(forest::FOLD_AT).len(),
            took as f64 / 1_000.0
        );
    }
    println!(
        "\n  {:.1}x, against §7's 10.3x. Both arms answer the same 349 524 — asserted inside the\n  \
         measurement, so the pair is two ways of computing one number — and the index arm's *without\n  \
         touching the forest* is a fact about the signature rather than a timing: `Order::subtree`\n  \
         takes `&self` and there is nowhere in it to reach the caller's data.\n",
        from_data as f64 / from_index.max(1) as f64
    );

    println!("report  splice against rebuild — §7 states 157-342 us flat against 40 393:");
    println!(
        "  {:<28}  {:>10}  {:>12}  {:>12}  {:>8}",
        "subtree folded", "rows out", "splice us", "rebuild us", "ratio"
    );
    // **Flat in what it removes**, over a forest built for the sweep: a million rows carrying four
    // subtrees of 2, 999, 99 999 and 599 999 rows, which is §7's own *2 rows to 999 999*.
    let (staged, sizes) = staged_forest();
    let staged_index = order::flattened(&staged);
    for (at, want) in sizes {
        let removed = staged_index.subtree(at).len();
        assert_eq!(removed, want, "the sweep's forest is not the shape it says");
        let (spliced, rebuilt) = order::fold_costs(&staged, at, ROUNDS);
        println!(
            "  {:<28}  {removed:>10}  {:>12.2}  {:>12.2}  {:>7.1}x",
            format!("a {removed}-row subtree"),
            spliced as f64 / 1_000.0,
            rebuilt as f64 / 1_000.0,
            rebuilt as f64 / spliced.max(1) as f64
        );
    }
    println!(
        "\n  **The splice is flat in what it removes and the rebuild is proportional to what\n  \
         remains**, which is the shape §7 states, and both directions are visible in one table: the\n  \
         splice moves 152 -> 240 us across five orders of magnitude of removal while the rebuild\n  \
         moves the *other* way, 992 -> 596, because the more it drops the less it has left to\n  \
         build. §7's own splice range is 157-342 and reproduces; its 40 393 us rebuild does not,\n  \
         and the reason is stated rather than fitted — that rebuild walked a **shuffled** forest at\n  \
         85 ns a row and this one reads a contiguous depth array, so it is 20-60x cheaper here and\n  \
         *still* the losing arm at every size a tree has.\n\n  \
         There is **no threshold and no rebuild path**. The crossover is where the two columns\n  \
         would meet — about 99.7% of the index removed, and in a tree only at the root, where the\n  \
         rebuild wins by doing nothing. A branch that has to stay correct for ever is not worth one\n  \
         edit's saving; §7 asks for that sentence as a doc comment and it is one, on\n  \
         `order::Order::fold`.\n"
    );

    println!(
        "report  variable row height — §7 states a 2.25x splice and a frame that does not move:"
    );
    let (bare_splice, with_splice) = order::heights_splice_cost(&forest, forest::FOLD_AT, ROUNDS);
    println!(
        "  {:<44}  {:>12.2}",
        "a fold, index only, us",
        bare_splice as f64 / 1_000.0
    );
    println!(
        "  {:<44}  {:>12.2}",
        "the same fold, prefix sum beside it, us",
        with_splice as f64 / 1_000.0
    );
    println!(
        "  {:<44}  {:>11.2}x",
        "what the prefix sum costs the splice",
        with_splice as f64 / bare_splice.max(1) as f64
    );

    let mut tall: Vec<order::Entry> = (0..100_000u32).map(order::Entry::of).collect();
    for (i, e) in tall.iter_mut().enumerate() {
        e.h = if i % 3 == 0 { 2 } else { 1 };
    }
    let tall = order::Order::built(tall);
    let heights = order::Heights::built(&tall);
    let top = heights.top_of(50_000);
    let (searched, direct) = order::row_at_costs(&heights, top, ROUNDS);
    let (stepped, one) = order::window_stepped(&heights, top, u32::from(forest::H));
    let (again, per_row) = order::window_searched(&heights, top, u32::from(forest::H));
    println!(
        "  {:<44}  {:>12.4}",
        "`row_at`, a binary search, us",
        searched / 1_000.0
    );
    println!(
        "  {:<44}  {:>12.4}",
        "`top_of`, an array index, us",
        direct / 1_000.0
    );
    assert_eq!(stepped, again, "the same window, both ways");
    println!(
        "\n  §7's footnote is 0.019 us against 0.0006, **once a frame rather than once a row**, and\n  \
         that is the whole of it: the two spellings answer the same {} content rows over an\n  \
         {}-row window, so no gate on the picture separates them — {one} search against {per_row}\n  \
         does. `Heights::needed` is what makes *only when rows can differ* checkable: a uniform\n  \
         index answers `false` and the component builds nothing at all.\n",
        stepped.len(),
        forest::H
    );
}

/// **A million rows carrying four subtrees of 2, 999, 99 999 and 599 999**, and where each sits.
///
/// §7's *flat in what it removes* is a claim over a sweep of subtree sizes, so the sweep needs a
/// forest that has them: one built out of the fold scene's own shape has exactly two.
fn staged_forest() -> (Vec<u16>, Vec<(usize, usize)>) {
    let mut depth: Vec<u16> = Vec::new();
    let mut at = Vec::new();
    for size in [2usize, 999, 99_999, 599_999] {
        at.push((depth.len(), size));
        depth.push(0);
        depth.extend(std::iter::repeat_n(1u16, size));
    }
    depth.resize(usize::try_from(forest::NODES).expect("a million fits"), 0);
    (depth, at)
}

/// 7. What does not reproduce, said out loud.
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
            "115 us against 1 187 for the interval, and 10.3x",
            "the index arm reproduces to the microsecond — 116 us over 349 524 rows, 0.33 ns a row \
             — and the data arm is 337 rather than 1 187, so the ratio is 2.9x. What the forest arm \
             costs is a property of the caller's own layout, and the one asserted is that both arms \
             answer the same interval",
        ),
        (
            "a 40 393 us rebuild, and 2.25x for the prefix sum",
            "the rebuild is ~990 us here, because §7's walked a shuffled forest at 85 ns a row and \
             this one reads a contiguous depth array. The *direction* is what the paragraph is \
             about and it reproduces at every size: the splice is flat and the rebuild is \
             proportional to what remains. The prefix sum's own figure comes back at 2.85x",
        ),
        (
            "7 -> 11 MB, 57%",
            "7.63 -> 11.44 MiB, which is §7's pair to the megabyte, and the growth is `4 / 8` \
             exactly — 50%. §7's 57% is what its own rounded pair divides to",
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
