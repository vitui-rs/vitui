//! **`layout` allocates nothing, at every arity.**
//!
//! A count, which is what §14's rule wants: *a gate is a count, a ratio, an equality or a compile
//! outcome.* Zero is a count, it is the same number on every machine, and it is the one property of
//! this module that a stopwatch could not have told us.
//!
//! # Why this is its own binary
//!
//! [`vitui_alloc_probe::CountingAllocator`] is a **process-global** allocator and the count is a
//! process-global counter. `cargo test` runs a binary's tests on several threads, so an allocating
//! sibling lands in the number — which is why the workspace's one test command is
//! `cargo test --workspace -- --test-threads=1`, and why this lives beside the library rather than
//! inside it.
//!
//! # What is actually being asserted
//!
//! Not *layout is fast*. **That the solver has no scratch space**, which is a structural claim: the
//! `Max` fixpoint keeps its frozen bit in the caller's output buffer (`out[i].x`), and
//! largest-remainder distribution ranks each lane against the others instead of sorting them. Both
//! of those are choices that would have been easier the other way, and this is the gate that stops
//! either one being quietly undone — a `Vec` for the remainders would pass every other test in this
//! crate.

use vitui_alloc_probe::{CountingAllocator, assert_no_alloc};
use vitui_engine::Rect;
use vitui_runtime::layout::{
    Col, Constraint,
    Constraint::{Fixed, Max, Min, Percent, Ratio, Weight},
    Grid, Row, Stack, rect, solve,
};

#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// `split` at every arity from one to twelve, which is the widest any corpus in this crate
/// generates.
///
/// Twelve separate const generic instantiations, written out rather than looped, because the arity
/// *is* the thing being varied and a loop cannot vary a const generic.
#[test]
fn split_allocates_zero_at_every_arity() {
    let band = Rect::new(0, 0, 300, 80);
    assert_no_alloc(|| {
        let _ = Row::new().split(band, [Weight(1)]);
        let _ = Row::new().split(band, [Fixed(20), Weight(1)]);
        let _ = Row::new().split(band, [Fixed(20), Weight(1), Min(10)]);
        let _ = Row::new().split(band, [Fixed(20), Weight(1), Min(10), Max(30)]);
        let _ = Row::new().split(
            band,
            [Percent(25), Percent(25), Weight(1), Fixed(4), Max(9)],
        );
        let _ = Row::new().split(
            band,
            [
                Ratio(1, 6),
                Ratio(1, 6),
                Weight(1),
                Weight(2),
                Fixed(3),
                Min(2),
            ],
        );
        let _ = Row::new().split(band, [Weight(1); 7]);
        let _ = Row::new().split(band, [Weight(1); 8]);
        let _ = Row::new().split(band, [Weight(1); 9]);
        let _ = Row::new().split(band, [Weight(1); 10]);
        let _ = Row::new().split(band, [Weight(1); 11]);
        let _ = Row::new().split(band, [Weight(1); 12]);
    });
}

/// The dynamic form, which is the one that could plausibly have needed a buffer and does not.
#[test]
fn the_dynamic_split_allocates_zero() {
    let band = Rect::new(0, 0, 300, 80);
    let spec = [
        Fixed(20),
        Weight(1),
        Min(10),
        Max(30),
        Percent(10),
        Ratio(1, 8),
        Weight(3),
    ];
    let mut out = [Rect::default(); 7];
    assert_no_alloc(|| {
        let n = Row::new()
            .spacing(1)
            .margin(1)
            .split_into(band, &spec, &mut out);
        assert_eq!(n, 7);
        let (n, fit) = Col::new().measure_into(band, &spec, &mut out);
        assert_eq!(n, 7);
        assert_eq!(fit.gaps.margin, 0);
    });
}

/// **The `Max` fixpoint driven to its bound**, which is where a scratch buffer would have been most
/// tempting: every round has to know which lanes are frozen, and the frozen set changes as it goes.
#[test]
fn the_cap_fixpoint_allocates_zero_when_it_is_driven_to_its_bound() {
    // Twelve descending ceilings over a band far larger than their sum, so a lane freezes on every
    // round and the fixpoint runs as long as it can.
    let spec: [Constraint; 12] = [
        Max(1),
        Max(2),
        Max(3),
        Max(4),
        Max(5),
        Max(6),
        Max(7),
        Max(8),
        Max(9),
        Max(10),
        Max(11),
        Weight(1),
    ];
    let mut out = [Rect::default(); 12];
    let (_, fit) = solve(4000, &spec, &mut out);
    assert!(fit.rounds >= 2, "the fixpoint did not engage: {fit:?}");
    assert_no_alloc(|| {
        for _ in 0..1_000 {
            let (n, fit) = solve(4000, &spec, &mut out);
            assert_eq!(n, 12);
            assert!(fit.rounds <= 12);
        }
    });
}

/// The rect algebra, `Align`, `Stack` and `Grid`.
///
/// `Grid` is on this list deliberately: it holds two sixty-four-element buffers **on the stack**, and
/// a convenience that quietly heap-allocated would be the easiest way to lose this property without
/// touching the solver at all.
#[test]
fn the_rect_algebra_and_the_grid_allocate_zero() {
    let a = Rect::new(4, 4, 40, 20);
    let b = Rect::new(0, 0, 30, 30);
    assert_no_alloc(|| {
        let _ = rect::inset(a, 2);
        let _ = rect::shrink(a, 1, 2, 3, 4);
        let _ = rect::expand(a, 3);
        let _ = rect::intersect(a, b);
        let _ = rect::clamp_to(a, b);
        let _ = rect::split_at_h(a, 7);
        let _ = rect::split_at_v(a, 7);
        let _ = Stack::center(a, 10, 5);
        let grid = Grid::new(8, 6).spacing(1).margin(1);
        for x in 0..8 {
            for y in 0..6 {
                let _ = grid.cell(a, x, y, 2, 2);
            }
        }
    });
}

/// **The whole realistic screen**, which is the shape the report times: thirty-two splits and one
/// hundred and nineteen lanes.
///
/// The one that matters, because the others each exercise a mechanism and this exercises the
/// composition of them — and a layout pass that allocated once per split would still pass every test
/// above while allocating thirty-two times a frame.
#[test]
fn a_whole_screen_of_layout_allocates_zero() {
    assert_no_alloc(|| {
        for _ in 0..100 {
            let (splits, lanes) = screen_frame(300, 80);
            assert_eq!((splits, lanes), (32, 119));
        }
    });
}

/// A realistic screen: a header, a sidebar, a detail pane, a footer of buttons and a
/// twenty-four-row list with four columns in each row.
///
/// Duplicated between this file and `examples/layout_numbers.rs` on purpose, and it is the one
/// duplication in this ticket. An integration test cannot see an example's private function and an
/// example cannot be imported; the alternative is a third module existing only to be shared, for
/// forty lines. **The two counts it returns are what keeps the copies honest** — both callers assert
/// `(32, 119)`, so a copy that drifted would fail rather than quietly measure something else.
fn screen_frame(w: u16, h: u16) -> (u32, u32) {
    let mut splits = 0;
    let mut lanes = 0;
    let screen = Rect::new(0, 0, w, h);

    let [header, body, footer] = Col::new().split(screen, [Fixed(3), Weight(1), Fixed(1)]);
    splits += 1;
    lanes += 3;

    let [sidebar, _gutter, detail] = Row::new()
        .spacing(1)
        .split(body, [Fixed(20), Fixed(1), Weight(1)]);
    splits += 1;
    lanes += 3;

    let [_top, _mid, _bot] = Col::new()
        .margin(1)
        .split(detail, [Weight(1), Weight(1), Weight(1)]);
    splits += 1;
    lanes += 3;

    let _buttons = Row::new().spacing(2).split(footer, [Weight(1); 6]);
    splits += 1;
    lanes += 6;

    let [left, mid, right] = Row::new().split(header, [Fixed(10), Weight(1), Fixed(12)]);
    splits += 1;
    lanes += 3;
    let _ = Row::new().split(left, [Weight(1), Weight(1)]);
    splits += 1;
    lanes += 2;
    let _ = Row::new().split(right, [Weight(1), Weight(1)]);
    splits += 1;
    lanes += 2;
    let _ = Col::new().split(mid, [Weight(1)]);
    splits += 1;
    lanes += 1;

    let mut rows = [Rect::default(); 24];
    let n = Col::new().split_into(sidebar, &[Weight(1); 24], &mut rows);
    for row in &rows[..n] {
        let _ = Row::new().split(*row, [Fixed(6), Weight(2), Weight(1), Fixed(8)]);
        splits += 1;
        lanes += 4;
    }
    (splits, lanes)
}
