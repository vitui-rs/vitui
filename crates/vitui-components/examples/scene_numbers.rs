//! **The scene list and what the reference-render runner reports over it.**
//!
//! Components ticket 04. The convention is the runtime's — a file in `examples/` named
//! `<subject>_numbers.rs` that prints the numbers a human reads — and so is the rule about what an
//! example may be: **`cargo test` does not run this file**, and no row of
//! [`vitui_components::gates::REGISTER`] rests on it. Four rows *cite* the runner and every one of
//! them names a `#[test]` in `src/runner.rs` beside the citation.
//!
//! # What it prints, and why each half is here
//!
//! 1. **The scene list, one line a scene, in §21's column order.** Twenty-six of the twenty-seven
//!    print the ticket that will build their subject rather than a row of zeros — see
//!    [`vitui_components::scenes::report`], which argues that at length. The twenty-seventh is a
//!    rehearsal over a fixture and says so.
//! 2. **The four hostile axes, each caught.** *n cells over m rows*, beside what the defective build
//!    cost — because the whole argument for an equality against a reference render is that **every
//!    one of the four made the defective build look healthier**, and a report that printed only the
//!    diff would leave the reader to take that on trust.
//! 3. **O5's coverage**, twelve of thirty-four, with the twenty-two bare pairs named.
//!
//! # It asserts the shape and not the timings
//!
//! R15's rule, inherited: *a gate is a count, a ratio, an equality or a compile outcome; a timing is
//! a report*. Every `assert!` below is a count — and this file is on the wrong side of the line for
//! a gate anyway, which is why the counts it shares with the register are asserted in `src/` too.

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::counters::Allocations;
use vitui_components::runner::{
    Fixture, at_two_sizes, compare, defective, play, reference, rows_at_a_time,
};
use vitui_components::scenes::{self, SCENES, Scene, Size};
use vitui_components::{Axis, INVENTORY};

// **The probe, because `allocations` is a total and a total needs something that counts.** Installed
// here rather than defaulted to zero inside the library: a figure defaulted to zero is a counter
// that prints `0` when it means *nobody counted*.
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// The fixture every rehearsal is played on. Forty columns by eighty rows, which is where §21's
/// *71 of 80 rows* comes from.
const W: u16 = 40;
/// See [`W`].
const H: u16 = 80;

fn main() {
    println!("§21's scene list — twenty-seven scenes, and the spec's table is the authority\n");

    // One rehearsal, over a fixture, so that the format the other twenty-six will print in is
    // visible rather than described.
    let scrolled = Fixture::lines(W, 12, 1_000).scrolled_to(60);
    let (run, allocated) =
        count_allocations(|| play(rows_at_a_time, std::slice::from_ref(&scrolled)));
    let row = run.row(
        "a scrolled collection",
        Allocations::over(1, allocated as u64),
    );

    println!("{}", scenes::report(&[(4, row)]));
    println!("The rehearsal's allocation total is the whole run's, `Driver::headless` included,");
    println!(
        "because `play` attaches its own driver, and its microsecond figure is a first frame."
    );
    println!(
        "**The steady-frame zero is `tests/budget.rs`'s**, which warms the identical workload"
    );
    println!("before it opens the window: the five frame structures take their allocation on the");
    println!("first frame that needs one and keep it.");

    println!("\nThe four hostile axes, each caught by an equality against a reference render.");
    println!("Every one of them made the defective build look *healthier*, which is why no");
    println!("counter in the stack disapproved of any of them.\n");
    println!(
        "{:<38}  {:<50}  {:>8}  {:>9}",
        "axis", "the runner reports", "correct", "defective"
    );

    // 1. The inverted scroll sign. It draws nothing, so it is fast.
    let diff = compare(
        reference,
        defective::inverted_scroll,
        std::slice::from_ref(&scrolled),
    );
    let correct = play(rows_at_a_time, std::slice::from_ref(&scrolled));
    let broken = play(defective::inverted_scroll, std::slice::from_ref(&scrolled));
    println!(
        "{:<38}  {:<50}  {:>8}  {:>9}",
        "scrolled - the inverted sign",
        diff.to_string(),
        correct.tally().writes(),
        broken.tally().writes()
    );
    assert_eq!(diff.rows, 11);

    // 2. The stale tail. Two frames, one rectangle, less content.
    let full = Fixture::lines(W, H, 200);
    let shrunk = full.shrunk_to(9);
    let diff = compare(
        reference,
        defective::stale_tail,
        &[full.clone(), shrunk.clone()],
    );
    let correct = play(rows_at_a_time, std::slice::from_ref(&shrunk));
    let broken = play(defective::stale_tail, std::slice::from_ref(&shrunk));
    println!(
        "{:<38}  {:<50}  {:>8}  {:>9}",
        "shrunk - the stale tail",
        diff.to_string(),
        correct.tally().writes(),
        broken.tally().writes()
    );
    assert_eq!(diff.rows, 71, "§21's own number: 71 of 80 rows");

    // 3. Twenty wheel clicks that move nothing.
    let base = Fixture::lines(W, 12, 400);
    let clicks: Vec<Fixture> = (0..=20).map(|c| base.scrolled_to(c * 3)).collect();
    let diff = compare(reference, defective::unmoved_wheel, &clicks);
    let correct = play(rows_at_a_time, &clicks);
    let broken = play(defective::unmoved_wheel, &clicks);
    println!(
        "{:<38}  {:<50}  {:>8}  {:>9}",
        "wheeled - twenty clicks, zero moved",
        diff.to_string(),
        correct.tally().writes(),
        broken.tally().writes()
    );
    assert_eq!(diff.rows, 12);

    // 4. Narrow: green at the wide size and red at the narrow one, which is the whole shape of the
    //    axis and cannot be seen at one size.
    let narrow_fixture = Fixture::lines(60, 20, 200);
    let wide = compare(
        reference,
        defective::narrow_drops_a_column,
        &[narrow_fixture.resized(60, 20)],
    );
    let narrow = compare(
        reference,
        defective::narrow_drops_a_column,
        &[narrow_fixture.resized(16, 8)],
    );
    println!("{:<38}  {}", "narrow - green at 60x20", wide);
    println!("{:<38}  {}", "narrow - red at 16x8", narrow);
    assert!(wide.clean());
    assert_eq!(narrow.rows, 8);

    let pair = at_two_sizes(&narrow_fixture, (60, 20), (16, 8), rows_at_a_time);
    println!(
        "\nThe content equality across two sizes, which is what keeps the damage count honest."
    );
    println!(
        "  {} of {} rows differ, {} cells written wide and {} narrow.",
        pair.rows_differing, pair.rows_compared, pair.wide_written, pair.narrow_written
    );
    println!(
        "  A build that merely drew less would score 0 differing rows by drawing nothing, and"
    );
    println!("  three separate defects on the map did exactly that.");
    assert_eq!(pair.rows_differing, 0);

    println!("\nO5 — a scene per hostile axis, by enumeration over `INVENTORY`.\n");
    let coverage = scenes::coverage();
    let covered = coverage.iter().filter(|(_, _, s)| !s.is_empty()).count();
    println!(
        "  {covered} of {} declared (component, axis) pairs have a scene in §21's own list.",
        coverage.len()
    );
    println!(
        "  The other {} are the per-component scenes tickets':",
        coverage.len() - covered
    );
    for (id, axis, numbers) in &coverage {
        if numbers.is_empty() {
            println!("    {id:<20} {}", axis.name());
        }
    }
    assert_eq!(covered, 12);
    assert_eq!(coverage.len(), 34);
    assert_eq!(
        INVENTORY
            .iter()
            .flat_map(|c| Axis::ALL.into_iter().filter(|a| c.declares(*a)))
            .count(),
        34
    );

    println!("\nShapes, which is what this file asserts:\n");
    println!("  {:<52}{}", "scenes", SCENES.len());
    println!(
        "  {:<52}{} / {} / {} / {} / {}",
        "at a size / two / a domain / cells / unstated",
        count(|s| matches!(s.size, Size::Screen { .. })),
        count(|s| matches!(s.size, Size::Two { .. })),
        count(|s| matches!(s.size, Size::Domain { .. })),
        count(|s| matches!(s.size, Size::Cells { .. })),
        count(|s| matches!(s.size, Size::Unstated)),
    );
    println!("  {:<52}{}", "marked `(owed)` by §21", count(|s| s.owed));
    println!(
        "  {:<52}{}",
        "from a defect that survived every gate",
        count(|s| s.from_a_survived_defect)
    );
    println!(
        "  {:<52}{}",
        "rehearsed over a fixture by `src/runner.rs`",
        count(|s| !s.rehearsed_by.is_empty())
    );
    assert_eq!(SCENES.len(), 27, "the scene list's shape has changed");
    assert_eq!(count(|s| s.owed), 1);
    assert_eq!(count(|s| s.from_a_survived_defect), 3);
    assert_eq!(count(|s| !s.rehearsed_by.is_empty()), 5);
}

fn count(f: impl Fn(&Scene) -> bool) -> usize {
    SCENES.iter().filter(|s| f(s)).count()
}
