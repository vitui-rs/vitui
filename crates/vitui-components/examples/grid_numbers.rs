//! **The grid: verbs as the currency, the band that overruns its pin, and what an equality cannot
//! see.**
//!
//! Components ticket 14. The convention is the runtime's — a file in `examples/` named
//! `<subject>_numbers.rs` that prints the numbers a human reads — and so is the rule about what an
//! example may be: **`cargo test` does not run this file**, and no row of
//! [`vitui_components::gates::REGISTER`] rests on it. Rows 68 and 69 *cite* it and each names two
//! `#[test]`s in `src/grid.rs` beside the citation.
//!
//! # What it prints
//!
//! 1. **The two scenes**, with where each stands and which ticket inverts it.
//! 2. **§6's headline**, measured here and remembered there, side by side — because the magnitudes
//!    belong to a prototype screen this ticket does not own and the *structure* is what reproduces.
//! 3. **The column axis**, at 12, 40, 120 and 240 declared columns, virtualised and clip-only.
//! 4. **The band**, in both spellings and at both offsets, with every counter this crate can read.
//! 5. **The inverted horizontal sign**, which is the defect the equality *does* catch.
//! 6. **O5's table column**, and what `scenes_for("table")` answers.
//! 7. **What does not reproduce**, said out loud rather than engineered away.
//!
//! # It asserts the shape and not the timings
//!
//! R15's rule, inherited: *a gate is a count, a ratio, an equality or a compile outcome; a timing is
//! a report*. Every `assert!` below is a count or an equality, and every microsecond figure is a
//! report — taken in whatever profile this was built in, which for `cargo run --example` is the
//! debug one and is why the figures are two orders of magnitude off §6's.

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::counters::{Allocations, Counter};
use vitui_components::gates::Standing;
use vitui_components::grid::{self, Opts};
use vitui_components::scenes::{SCENES, scenes_for};
use vitui_components::{Axis, INVENTORY};

// **The probe, because `allocations` is a total and a total needs something that counts.** Installed
// here rather than defaulted to zero inside the library: a figure defaulted to zero is a counter
// that prints `0` when it means *nobody counted*.
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// How many frames a per-frame figure is averaged over. Forty, warmed.
const FRAMES: u32 = 40;

fn main() {
    println!(
        "The grid — {}x{}, one table, twelve declared columns, both edges pinned, and a \
         scrolling band {} cells wide in a {}-cell viewport\n",
        grid::W,
        grid::H,
        grid::CONTENT_W,
        grid::VIEW_W
    );

    scene_list();
    the_headline();
    the_column_axis();
    the_band();
    the_inverted_sign();
    o5();
    what_does_not_reproduce();
}

/// 1. The two scenes, and which ticket inverts each.
fn scene_list() {
    println!("report  the two scenes `table` is a screen of:");
    println!(
        "  {:>3}  {:<14}  {:<13}  scene",
        "#", "standing", "inverted by"
    );
    let mut red = 0usize;
    for scene in SCENES.iter().filter(|s| s.stands.contains(&"table")) {
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
            "  {:>3}  {word:<14}  {ticket:<13}  {}",
            scene.number, scene.name
        );
    }
    assert_eq!(red, 2, "both are pinned red");
    println!(
        "\n  Both are waiting for `table` and neither is waiting for anything else. Unlike \
         components\n  ticket 11's five, there is no second reason here: the band defect is a \
         defect of a *spelling*\n  the grid holds as an arm, not of code anything ships.\n"
    );
}

/// 2. §6's headline, measured beside remembered.
fn the_headline() {
    let one = grid::frame(Opts::correct(), 1, grid::VOLUMES[2], 0);
    let twelve = grid::frame(Opts::correct(), 12, grid::VOLUMES[2], grid::HOFF);
    let one_us = grid::frame_cost(Opts::correct(), 1, grid::VOLUMES[2], 0, FRAMES);
    let twelve_us = grid::frame_cost(Opts::correct(), 12, grid::VOLUMES[2], grid::HOFF, FRAMES);

    println!("report  §6's headline: the same screen, the same rectangle, the same cells");
    println!(
        "  {:<34}  {:>8}  {:>8}  {:>10}  {:>10}",
        "drawn as", "cells", "verbs", "us here", "§6"
    );
    println!(
        "  {:<34}  {:>8}  {:>8}  {:>10}  {:>10}",
        "a plain list (§6's screen)", "23 030", "418", "-", "-"
    );
    println!(
        "  {:<34}  {:>8}  {:>8}  {:>10}  {:>10}",
        "a tree (§6's screen)", "23 030", "640", "-", "48.79"
    );
    println!(
        "  {:<34}  {:>8}  {:>8}  {:>10.2}  {:>10}",
        "one column, this screen",
        one.writes,
        one.verbs,
        one_us.as_secs_f64() * 1e6,
        "913 / 71.5"
    );
    println!(
        "  {:<34}  {:>8}  {:>8}  {:>10.2}  {:>10}",
        "twelve columns, this screen",
        twelve.writes,
        twelve.verbs,
        twelve_us.as_secs_f64() * 1e6,
        "2 497 / 165.6"
    );

    assert_eq!(one.writes, twelve.writes, "identical to the cell");
    assert_eq!(one.writes, grid::CELLS);
    assert_eq!(one.verbs, grid::VERBS_ONE);
    assert_eq!(twelve.verbs, grid::VERBS_TWELVE);
    println!(
        "\n  The cells are identical to the cell and the frame costs {:.1}x the verbs. Two verbs \
         a cell is\n  the floor — the text, then the padding after it — and {} of the {} columns \
         drawn are narrower\n  than their own content, so their pad is a zero-length run and \
         `Ink::run` refuses to issue one.\n  No cell costs three, which is the half of §6's \
         sentence that matters.\n",
        twelve.verbs as f64 / one.verbs as f64,
        grid::FULL_COLUMNS,
        grid::COLUMNS_DRAWN
    );

    // **The marginal total, and not the total.** A total that included the driver, the theme and
    // the declaration list would be a number about attaching a terminal; the difference between
    // forty frames and eighty is a number about a frame, and §21's refinement 2 is why it is a
    // total either way — *a mean cannot see anything below n; a total can see one*.
    let (_, forty) = count_allocations(|| {
        grid::frame_cost(Opts::correct(), 12, grid::VOLUMES[2], grid::HOFF, FRAMES)
    });
    let (_, eighty) = count_allocations(|| {
        grid::frame_cost(
            Opts::correct(),
            12,
            grid::VOLUMES[2],
            grid::HOFF,
            FRAMES * 2,
        )
    });
    println!(
        "  allocations: {forty} over {FRAMES} frames and {eighty} over {}, so **{} for the {} \
         frames in between**\n  — the setup is inside both figures and the difference is the \
         frame.\n",
        FRAMES * 2,
        eighty.saturating_sub(forty),
        FRAMES
    );
    assert_eq!(eighty, forty, "a frame of the grid allocates");
}

/// 3. The column axis: `visible_cols` is as load-bearing as `visible_rows`.
fn the_column_axis() {
    println!("report  the column axis, at 12 / 40 / 120 / 240 declared columns:");
    println!(
        "  {:<28}  {:>8}  {:>8}  {:>8}  {:>8}  {:>10}",
        "declared", "cells", "verbs", "asked", "drawn", "us"
    );
    let mut virtualised = Vec::new();
    for declared in grid::DECLARED {
        let shape = grid::frame(Opts::correct(), declared, grid::VOLUMES[2], grid::HOFF);
        let us = grid::frame_cost(
            Opts::correct(),
            declared,
            grid::VOLUMES[2],
            grid::HOFF,
            FRAMES,
        );
        println!(
            "  {:<28}  {:>8}  {:>8}  {:>8}  {:>8}  {:>10.2}",
            format!("{declared}, virtualised"),
            shape.writes,
            shape.verbs,
            shape.asked,
            shape.columns,
            us.as_secs_f64() * 1e6
        );
        virtualised.push(shape);
    }
    for declared in [12usize, 40, 120] {
        let shape = grid::frame(Opts::clip_only(), declared, grid::VOLUMES[2], grid::HOFF);
        let us = grid::frame_cost(
            Opts::clip_only(),
            declared,
            grid::VOLUMES[2],
            grid::HOFF,
            FRAMES,
        );
        println!(
            "  {:<28}  {:>8}  {:>8}  {:>8}  {:>8}  {:>10.2}",
            format!("{declared}, clip-only"),
            shape.writes,
            shape.verbs,
            shape.asked,
            shape.columns,
            us.as_secs_f64() * 1e6
        );
    }
    for shape in &virtualised {
        assert_eq!(*shape, virtualised[0], "the column axis is not flat");
    }
    let clip = grid::frame(Opts::clip_only(), 120, grid::VOLUMES[2], grid::HOFF);
    println!(
        "\n  Flat across all four, and the clip-only spelling at 120 costs {:.1}x the verbs and \
         {:.1}x what it\n  asks for — for **identical writes**, because the engine reports a fully \
         clipped verb as zero\n  columns. §6 measured 3.3x the frame and 7.2x the verbs on its own \
         screen, which capped a frame\n  at 128 drawn columns where this one does not.\n",
        clip.verbs as f64 / virtualised[0].verbs as f64,
        clip.asked as f64 / virtualised[0].asked as f64,
    );

    let volumes = grid::across_volumes(Opts::correct());
    println!("  and the row axis, which a table inherits from a collection:");
    for (rows, shape) in &volumes {
        println!(
            "  {:<28}  {:>8}  {:>8}  {:>8}  {:>8}",
            format!("{rows} rows"),
            shape.writes,
            shape.verbs,
            shape.asked,
            shape.iterated
        );
    }
    println!();
}

/// 4. The band, in both spellings and at both offsets.
fn the_band() {
    let correct = grid::frame(Opts::correct(), 12, grid::VOLUMES[2], grid::HOFF);
    let broken = grid::frame(Opts::arithmetic_band(), 12, grid::VOLUMES[2], grid::HOFF);
    let straddle = grid::frame(
        Opts::arithmetic_band(),
        12,
        grid::VOLUMES[2],
        grid::HOFF_STRADDLE,
    );
    let correct_us = grid::frame_cost(Opts::correct(), 12, grid::VOLUMES[2], grid::HOFF, FRAMES);
    let broken_us = grid::frame_cost(
        Opts::arithmetic_band(),
        12,
        grid::VOLUMES[2],
        grid::HOFF,
        FRAMES,
    );

    println!("report  §6's refused arm: the offset as arithmetic, no view");
    println!(
        "  {:<34}  {:>8}  {:>9}  {:>8}  {:>8}  {:>11}  {:>9}",
        "spelling", "writes", "distinct", "verbs", "asked", "re-damaged", "us"
    );
    for (name, shape, us) in [
        ("a view per row (the rule)", correct, correct_us),
        ("the offset as arithmetic", broken, broken_us),
    ] {
        println!(
            "  {name:<34}  {:>8}  {:>9}  {:>8}  {:>8}  {:>11}  {:>9.2}",
            shape.writes,
            shape.distinct,
            shape.verbs,
            shape.asked,
            shape.re_damaged(),
            us.as_secs_f64() * 1e6
        );
    }
    println!(
        "  {:<34}  {:>8}  {:>9}  {:>8}  {:>8}  {:>11}  {:>9}",
        format!("the same, at hoff {}", grid::HOFF_STRADDLE),
        straddle.writes,
        straddle.distinct,
        straddle.verbs,
        straddle.asked,
        straddle.re_damaged(),
        "-"
    );

    let inert = Allocations::over(1, 0);
    let separating =
        grid::counters_that_separate(Opts::correct(), Opts::arithmetic_band(), inert, inert);
    println!(
        "\n  Of §20's nine counters, {} separates the two arms: {:?}. `marked` is the one §6 \
         measured\n  and it is `Unreachable` from this crate for ever — `vitui-engine/src/damage.rs` \
         is `pub(crate)`\n  and `Presented` carries no cell count.\n",
        separating.len(),
        separating
    );

    println!(
        "  the equality against a reference render, on the same arms:\n    {:<38}  {}",
        "the rule, at a column boundary",
        grid::equality(grid::banded, grid::HOFF)
    );
    println!(
        "    {:<38}  {}",
        "the arithmetic band, same offset",
        grid::equality(grid::arithmetic, grid::HOFF)
    );
    println!(
        "    {:<38}  {}",
        "the arithmetic band, hoff inside a column",
        grid::equality(grid::arithmetic_straddling, grid::HOFF_STRADDLE)
    );
    println!(
        "    {:<38}  {}",
        "the rule, hoff inside a column",
        grid::equality(grid::banded_straddling, grid::HOFF_STRADDLE)
    );
    assert_eq!(broken.re_damaged(), grid::RE_DAMAGED);
    assert!(grid::equality(grid::arithmetic, grid::HOFF).clean());
    println!(
        "\n  **The equality is blind to it and the pair is not.** §6's own clause says why — *the \
         screen is\n  correct, because the pinned band draws afterwards and wins* — so a \
         comparison of the two\n  surfaces reports 0 cells over 0 rows on a build that will \
         re-damage {} cells a frame for ever.\n  What sees it is `writes` against `distinct`, \
         which is §21's register row 6, C02's, filed there\n  as a *report per component*. §6's \
         *no gate left by C01, C02 or C03 sees it* is half true: C02\n  named the counter and \
         nobody was running it.\n\n  The last line is not about the table. At an offset inside a \
         column the band's clip discards a\n  **prefix**, and ADR 0022's clamp-and-discard makes \
         `Pen` — like `Tally` — wrong by exactly that\n  prefix. {} cells over 80 rows, and the \
         painter is not what disagrees. It is the whole reason\n  the scene's offset is a column \
         boundary.\n",
        grid::RE_DAMAGED,
        grid::STRADDLE_RECORDER_CELLS,
    );
}

/// 5. The inverted horizontal sign — the defect the equality does catch.
fn the_inverted_sign() {
    let correct = grid::frame(Opts::correct(), 12, grid::VOLUMES[2], grid::HOFF);
    let broken = grid::frame(Opts::inverted_sign(), 12, grid::VOLUMES[2], grid::HOFF);
    let broken_us = grid::frame_cost(
        Opts::inverted_sign(),
        12,
        grid::VOLUMES[2],
        grid::HOFF,
        FRAMES,
    );
    let correct_us = grid::frame_cost(Opts::correct(), 12, grid::VOLUMES[2], grid::HOFF, FRAMES);

    println!("report  §3.3's refused arm: the horizontal sign inverted");
    println!(
        "  {:<24}  {:>8}  {:>8}  {:>8}  {:>10}",
        "sign", "writes", "verbs", "asked", "us"
    );
    for (name, shape, us) in [
        ("+hoff (the rule)", correct, correct_us),
        ("-hoff", broken, broken_us),
    ] {
        println!(
            "  {name:<24}  {:>8}  {:>8}  {:>8}  {:>10.2}",
            shape.writes,
            shape.verbs,
            shape.asked,
            us.as_secs_f64() * 1e6
        );
    }
    println!(
        "  the equality: {}",
        grid::equality(grid::inverted, grid::HOFF)
    );
    let inert = Allocations::over(1, 0);
    let separating =
        grid::counters_that_separate(Opts::correct(), Opts::inverted_sign(), inert, inert);
    assert!(separating.contains(&Counter::Writes));
    println!(
        "\n  {} writes **fewer**, every verb still issued, and {:?} separate the arms. §3.3 \
         measured\n  4 271 fewer and forty microseconds faster on its own screen. This is the \
         defect the equality\n  under a horizontal offset is for, and the one §21's register row \
         12 names.\n",
        correct.writes - broken.writes,
        separating
    );
}

/// 6. O5's `table` column.
fn o5() {
    println!("report  O5, from the table's side:");
    let component = INVENTORY
        .iter()
        .find(|c| c.id == "table")
        .expect("`table` is in the freeze");
    for axis in Axis::ALL {
        if !component.declares(axis) {
            continue;
        }
        let numbers: Vec<u8> = SCENES
            .iter()
            .filter(|s| s.covers.iter().any(|(id, a)| *id == "table" && *a == axis))
            .map(|s| s.number)
            .collect();
        println!("  {:<10}  scenes {numbers:?}", axis.name());
    }
    let stands: Vec<u8> = scenes_for("table").map(|s| s.number).collect();
    println!("  `scenes_for(\"table\")` answers {stands:?}");
    assert_eq!(stands, vec![7, 30]);
    println!(
        "\n  O5 did not move, and that is correct: §21's row 7 already claimed \
         `(table, scrolled)` and\n  `(table, narrow)`, and scene 30 is a second **instrument** on \
         one axis rather than a second axis.\n  Two of the four axes `table` declares — shrunk and \
         wheeled — still have no scene, and this\n  ticket does not pretend otherwise.\n"
    );
}

/// 7. What does not reproduce.
fn what_does_not_reproduce() {
    println!("report  what does not reproduce, and why:");
    for line in [
        "  §6's 913 -> 2 497 verbs for 23 030 cells is a prototype screen with two tables and three",
        "    bars on one terminal. One table filling the same terminal is 160 -> 1 760 for 24 000",
        "    cells. The **structure** reproduces exactly: identical cells, and the verbs are the",
        "    difference.",
        "",
        "  §6's 345 re-damaged cells is that screen's band reaching past that viewport. Here it is",
        "    560 — seven columns a row over eighty rows — and the seven is the right pin's own width",
        "    because the band's last column reaches 39 past the viewport and the pin is only seven",
        "    wide, so the pin is overwritten whole.",
        "",
        "  §6's *identical writes* does **not** reproduce, and the difference is a fact about the",
        "    prototype's counter rather than about the defect: `proto-c04-app/src/count.rs` folds a",
        "    `fill` in as the rectangle it asked for and a `text` in as what landed after clipping,",
        "    so a padded remainder discarded by the band's clip is counted on both arms and the two",
        "    come out level. `Tally` keeps `asked` and `reported` apart and the arms differ by",
        "    exactly the overrun.",
        "",
        "  §6's *no gate left by C01, C02 or C03 sees it* is half true. The equality is blind and",
        "    the pair `writes == distinct` — C02's row 6 — is not. What §21 records about row 6 is",
        "    that it is a *report per component*, so the honest form of the sentence is that the",
        "    gate existed and nothing was running it.",
        "",
        "  §6's 3.3x the frame and 7.2x the verbs at 120 declared columns is 10.9x the verbs here,",
        "    because that prototype capped a frame at 128 drawn columns and this one does not. The",
        "    direction and the flatness both reproduce exactly.",
        "",
        "  §6's *two verbs a cell is the floor* is refined rather than contradicted: a cell whose",
        "    own text fills its column has no padding and costs one. Two of the eleven columns drawn",
        "    are such cells and both are pins. No cell costs three.",
        "",
        "  And one number that is nobody's remembered figure: `Ctx::scroll_scope` at a nonzero",
        "    offset answers `visible_rows() == -1000..-920` for an offset of a thousand. That is",
        "    C03's inverted scroll sign inside the runtime's own verb, register row 70, filed as",
        "    `.scratch/vitui-runtime-architecture/issues/26`, and it is why every frame here is",
        "    drawn at a vertical offset of zero.",
    ] {
        println!("{line}");
    }
}
