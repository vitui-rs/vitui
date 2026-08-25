//! **The table: the `+` paid in verbs, two cell stores, one editing slot, and identity per cell.**
//!
//! Components ticket 15. The convention is the runtime's — a file in `examples/` named
//! `<subject>_numbers.rs` that prints the numbers a human reads — and so is the rule about what an
//! example may be: **`cargo test` does not run this file**, and no row of
//! [`vitui_components::gates::REGISTER`] rests on it. Every figure below has a `#[test]` in
//! `src/collect.rs` or `src/grid.rs` behind it that asserts the count, the ratio or the equality;
//! what is only here is the microseconds, which are a report (§21).
//!
//! `examples/grid_numbers.rs` is the other half and it is the **screen**: the cells, the verbs, the
//! band's re-damage, the column sweep and the two equalities. This file is the **component**: what
//! it stores, what a gesture costs it, and the three refusals §6 states as measurements.
//!
//! # What it prints
//!
//! 1. **`table = collection + column rectangles`**, as the two counts that make it checkable.
//! 2. **The three bands**, and the pin that may not be elastic.
//! 3. **Cell selection**, per column key against the flattened index, in both directions.
//! 4. **The editing slot**, one position against a map.
//! 5. **Identity**, per cell against per row, with `merges` as the only counter that sees it.
//! 6. **The loop structure §6 refuses while calling it cheaper**, priced.
//! 7. **The frame**, and the budget line it is over.

use std::time::Duration;

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::collect::{
    CELL_COLS, CELL_ROWS, CellSel, Column, Pin, SPAN_BYTES, Solved, TableState, edit_follow_costs,
    header_click_costs, row_click_costs, solve_columns,
};
use vitui_components::grid::{self, Opts};
use vitui_runtime::layout::Constraint;

// **The probe, because `allocations` is a total and a total needs something that counts.**
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// How many frames a per-frame figure is averaged over. Forty, warmed.
const FRAMES: u32 = 40;

/// How many round-robin rounds the loop-structure minimum is taken over. Sixty.
///
/// A minimum and not a mean, and interleaved rather than one arm after the other — see
/// [`vitui_components::grid::band_pass_costs`], which carries the argument and the two runs that
/// forced it.
const ROUNDS: u32 = 60;

fn main() {
    headline();
    bands();
    cell_selection();
    editing();
    identity();
    loop_structure();
    frame();
}

fn headline() {
    let one = grid::frame(Opts::correct(), 1, grid::VOLUMES[2], 0);
    let twelve = grid::frame(Opts::correct(), 12, grid::VOLUMES[2], grid::HOFF);
    println!("\n`table` = `collection` + column rectangles\n");
    println!("  the same rectangle, drawn two ways        cells    verbs");
    println!(
        "  one column (a list, as a table)          {:>6}   {:>6}",
        one.writes, one.verbs
    );
    println!(
        "  twelve columns, two pinned               {:>6}   {:>6}",
        twelve.writes, twelve.verbs
    );
    println!(
        "  ratio                                    {:>6.2}x  {:>6.2}x",
        twelve.writes as f64 / one.writes as f64,
        twelve.verbs as f64 / one.verbs as f64
    );
    println!(
        "\n  §6 remembers 23 030 / 913 -> 23 030 / 2 497 on a screen of two tables and three\n  \
         bars. The cells are identical either way and the verbs are the difference, which is\n  \
         the structure §6 states; the magnitudes are that other screen's."
    );
    println!(
        "\n  two verbs a cell is the floor: {} of {} drawn columns fill themselves exactly and\n  \
         cost one — and no cell costs three.",
        grid::FULL_COLUMNS,
        grid::COLUMNS_DRAWN
    );
}

fn bands() {
    let specs = grid::columns(12);
    let s = solve_columns(grid::W, &specs);
    println!("\nThree bands, and the pin that may not be elastic\n");
    println!(
        "  left pins {:>3} cells   scrolling viewport {:>3}   right pins {:>3}",
        s.left_w, s.view_w, s.right_w
    );
    println!(
        "  the band's content is {} cells — the one width in a table that is not a viewport —\n  \
         so the largest horizontal offset is {}.",
        s.content_w,
        s.max_hoff()
    );

    // The pin's own width wins over a constraint that would have stretched it.
    let elastic = [
        Column::new(0, "pinned", Constraint::Weight(9)).pinned_left(8),
        Column::new(1, "free", Constraint::Weight(1)),
    ];
    let e: Solved = solve_columns(100, &elastic);
    println!(
        "\n  a `Weight(9)` column pinned at 8 solves to {} cells, not to nine tenths of the\n  \
         table: `Pin::width` is the only reader of a pin's width and no `Constraint` reaches it.",
        e.w[0]
    );
    assert_eq!(e.w[0], 8);
    assert_eq!(Pin::Left(8).width(), Some(8));
    assert_eq!(Pin::None.width(), None);
}

fn cell_selection() {
    let (per_column, flat) = header_click_costs(CELL_ROWS, CELL_COLS, 3);
    println!("\nCell selection is a run list per column key\n");
    println!("  one click on a column header, {CELL_ROWS} rows      runs        bytes        µs");
    println!(
        "  a run list per column key             {:>8}   {:>10}   {:>7.2}",
        per_column.0,
        per_column.1,
        per_column.2 as f64 / 1_000.0
    );
    println!(
        "  one list over `row · ncols + col`     {:>8}   {:>10}   {:>7.2}",
        flat.0,
        flat.1,
        flat.2 as f64 / 1_000.0
    );
    println!(
        "  ratio                                 {:>8}x  {:>10}x",
        flat.0 / per_column.0.max(1),
        flat.1 / per_column.1.max(1)
    );
    assert_eq!(per_column.0, 1, "one run, at any length");
    assert_eq!(per_column.1, SPAN_BYTES);
    assert_eq!(flat.0, CELL_ROWS, "a stride of length one is not a run");

    let (mine, theirs) = row_click_costs(CELL_COLS, 17);
    println!(
        "\n  and the other direction, which is the loss: selecting one whole row is {mine} runs\n  \
         here against {theirs} there — bounded by the declared column count, because a table's\n  \
         row count is unbounded and its column count is not."
    );
    assert_eq!((mine, theirs), (CELL_COLS, 1));

    let mut cells = CellSel::new();
    cells.select_column(3, CELL_ROWS);
    println!(
        "  the frame does not distinguish them at all: both answer `contains` through the same\n  \
         `Scan`, and one is {}.",
        cells.contains(999_999, 3)
    );
}

fn editing() {
    let (slot, map) = edit_follow_costs(CELL_ROWS, CELL_ROWS);
    println!("\nIn-cell editing is `Option<(row, col)>` and no wider\n");
    println!(
        "  follow a {CELL_ROWS}-row reorder     one slot {:>10.2} µs      a map per cell {:>10.2} µs",
        slot as f64 / 1_000.0,
        map as f64 / 1_000.0
    );
    println!(
        "  §6 states 0.29 µs, and its reason is the shape rather than the clock: the slot is\n  \
         **one position**, so the cost is one call at any length. The column half needs nothing,\n  \
         because it is a key."
    );
    let mut st = TableState::new();
    st.edit(42, 7);
    assert_eq!(st.editing(), Some((42, 7)));
    st.follow(|row| Some(row + 1));
    assert_eq!(st.editing(), Some((43, 7)), "the column half did not move");
}

fn identity() {
    let (keyed_regions, keyed_merges) = grid::identity_merges(true);
    let (row_regions, row_merges) = grid::identity_merges(false);
    let (inert_regions, inert_merges) = grid::inert_cells();
    println!("\nIdentity is per cell where a cell declares a target\n");
    println!("  two tables on one screen, every visible cell a target   regions   merges");
    println!(
        "  keyed per cell                                          {keyed_regions:>7}   {keyed_merges:>6}"
    );
    println!(
        "  keyed per row                                           {row_regions:>7}   {row_merges:>6}"
    );
    println!(
        "  no target in a cell at all                              {inert_regions:>7}   {inert_merges:>6}"
    );
    println!(
        "\n  {} of {} targets are inert on the row-keyed arm, and the screen renders identically\n  \
         — drawing does not consume an id. §6 remembers 142 merges and 284 of 911 inert on the\n  \
         prototype's screen; the arithmetic is the same and the population is not.",
        row_merges,
        keyed_regions - inert_regions
    );
    assert_eq!(keyed_merges, 0, "the shipped build merges nothing");
    assert!(row_merges > 0, "and the refused one does");
}

fn loop_structure() {
    let (row_shape, band_shape) = grid::band_pass_shapes();
    let (row_us, band_us) = grid::band_pass_costs(ROUNDS);
    println!(
        "\nOne row pass against a pass per band — correct, refused, and not measurably cheaper\n"
    );
    println!("  loop structure        writes   distinct    verbs   min µs");
    println!(
        "  one row pass         {:>7}   {:>8}   {:>6}   {:>7.2}",
        row_shape.writes,
        row_shape.distinct,
        row_shape.verbs,
        micros(row_us)
    );
    println!(
        "  a pass per band      {:>7}   {:>8}   {:>6}   {:>7.2}",
        band_shape.writes,
        band_shape.distinct,
        band_shape.verbs,
        micros(band_us)
    );
    println!(
        "  difference           {:>7}   {:>8}   {:>6}   {:>+7.1}%",
        band_shape.writes as i64 - row_shape.writes as i64,
        band_shape.distinct as i64 - row_shape.distinct as i64,
        band_shape.verbs as i64 - row_shape.verbs as i64,
        (micros(band_us) - micros(row_us)) / micros(row_us) * 100.0
    );
    println!(
        "\n  §6 states **4% cheaper and refused**, and the 4% does not reproduce: over sixty\n  \
         interleaved rounds, minimum of each, the difference lands inside ±1.5% and **the sign\n  \
         flips between runs of the same binary**. The saving is real and so is what pays for it —\n  \
         the band's view is opened once for the whole window instead of once a row, and three row\n  \
         loops and three `Scan::seek`s are bought with it — and on this screen, where the pins\n  \
         are three columns of the eleven drawn, they cancel.\n\n  \
         The refusal is unaffected, and that is the point rather than a consolation: §6 refuses\n  \
         the spelling on an **argument**, not on a timing. The cells are identical, so no counter\n  \
         separates them; what the shipped shape buys is that §5's `O(log k + h)` stays one query\n  \
         instead of becoming `3·(log k + h)`, and that three sets of bounds are not three chances\n  \
         to write one of them differently. A spelling refused on an argument does not become\n  \
         acceptable when the clock stops agreeing with it."
    );
    assert_eq!(row_shape.writes, band_shape.writes);
    assert_eq!(row_shape.distinct, band_shape.distinct);
    assert_eq!(row_shape.verbs, band_shape.verbs);
}

fn frame() {
    let shape = grid::frame(Opts::correct(), 12, grid::VOLUMES[2], grid::HOFF);
    let cost = grid::frame_cost(Opts::correct(), 12, grid::VOLUMES[2], grid::HOFF, FRAMES);
    // **Marginal, because the setup is inside both figures and the difference is the frame.**
    // `grid_numbers.rs` states the arrangement and its reason: a driver, a theme and a column list
    // are allocated once and a total that included them would report the setup.
    let over = |frames| {
        count_allocations(|| {
            let _ = grid::frame_cost(Opts::correct(), 12, grid::VOLUMES[2], grid::HOFF, frames);
        })
        .1
    };
    let (forty, eighty) = (over(FRAMES), over(FRAMES * 2));
    println!("\nThe frame\n");
    println!(
        "  {:.2} µs / 0 marked / {} writes / {} verbs / {} regions / {} stops",
        micros(cost),
        shape.writes,
        shape.verbs,
        shape.regions,
        shape.stops
    );
    println!(
        "  allocations: {forty} over {FRAMES} frames and {eighty} over {}, so **{} for the {FRAMES}\n  \
         frames in between**",
        FRAMES * 2,
        eighty - forty
    );
    assert_eq!(forty, eighty, "a frame of the table allocates nothing");
    println!(
        "\n  §6 states 165.6 µs / 0 marked / 23 030 / 2 497 / 59 / 5 / 0 for its own screen, and\n  \
         **166 µs is over the frame budget while damaging nothing at all**. It is not fixed\n  \
         here: spec §22 files it as one of four instances of a product call that is the user's,\n  \
         and this ticket records it rather than optimising against it."
    );
}

fn micros(d: Duration) -> f64 {
    d.as_nanos() as f64 / 1_000.0
}
