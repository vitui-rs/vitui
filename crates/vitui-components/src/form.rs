//! **One screen, drawn three ways**, which is what turns the rule from a paragraph into a
//! number.
//!
//! Three measurements are owed here and none of them can be taken on a helper in
//! isolation: *routing through `fit` costs nothing against the discipline* is an equality between
//! two whole screens; *the naive form is 43 941 against 0* is a differential between two whole
//! screens; and *`Compact` against `Cosy`* is the same form at two densities. So the form is here,
//! beside the helpers, and the three arms are three `crate::runner::Painter`s over it.
//!
//! # What the form is
//!
//! 300×80. A header row, a footer row, and three panels side by side across the 78 rows between
//! them. Every panel is a [`crate::frame::block`] with the same fifteen-column title; every row of
//! every panel's interior is a label and a chip, and the chip is the row's interactive region.
//!
//! **Two of the three panels overflow and the third does not**, and both halves are load-bearing:
//!
//! - The two that overflow are what makes the density measurement mean something. `Compact` pads one
//!   cell and `Cosy` two, so a `Cosy` panel is **two rows shorter** and two of its widgets
//!   fall off the bottom. Two panels, two rows each: **four widgets**, which is the recorded number
//!   reports beside `20 804 / 267` against `20 992 / 263`. It is a consequence of `pad_y` and of the
//!   panel count, not a tuned figure.
//! - The one that does not overflow is what leaves an **unwritten tail** — *no cell never*, register
//!   row 7, arriving on a screen instead of in a sentence. It is also what makes the two densities'
//!   write counts differ at all: a form whose every interior row is written covers 24 000 cells at
//!   both densities and the comparison says nothing.
//!
//!   **Row 7 is green since components 40 and this tail is the named exception, not a hole.**
//!   own third refinement is *name the exception; do not loosen the gate*, and the exception is that
//!   this form is a **fixture** rather than a component: the two helpers it is built from each write
//!   a partition of the rectangle they were handed and `block` *returns* the interior it did not
//!   write, which is the rule working. What does not write the tail is the screen — this file — and
//!   it is what buys two of the three measurements. A gate run over every screen in this
//!   crate would have to delete them to go green, which is the trade the refinement exists to
//!   refuse. The rule is checked where it is a rule: per component in `tests/golden.rs` over the
//!   thirty-three constructions, and over the assembled gallery in `crate::gallery`.
//!
//! # The three arms
//!
//! | arm | what it is |
//! |---|---|
//! | [`routed`] | every row through [`crate::text::fit`], every panel through [`crate::frame::block`] |
//! | [`by_hand`] | the same order, with `fit`'s truncation, justification and padding **written out** |
//! | [`naive`] | a screen clear, a panel fill under every interior, and a face filled under every label |
//!
//! `by_hand` keeps `block` and open-codes only `fit`, because that is the claim the table
//! makes: *`fit` against the same order written by hand*. Its truncation is `layout::text::truncate`
//! and a `Glyph::Ellipsis` written out at the call site rather than [`crate::glyphs::elide`], so the
//! equality is between two implementations and not between one implementation and itself.
//!
//! **`naive` is kept and never deleted.** It is the reference the correct build is proved cheaper
//! than, and deleting it deletes the argument — the rule, arriving here for the third time.

use vitui_runtime::layout::text as measure;
use vitui_runtime::{Ctx, Density, Glyph, Id, Interest, Paint, Role};

use crate::frame::{BlockOpts, block_into};
use crate::ink::Ink;
use crate::runner::{Fixture, Pen, Run, play_at};
use crate::text::{FitOpts, Justify, fit_into};
use vitui_runtime::Rect;
use vitui_runtime::layout::rect;

/// The screen's width. Every dense screen here is priced at 300×80.
pub const W: u16 = 300;
/// The screen's height.
pub const H: u16 = 80;
/// How many panels stand side by side.
pub const PANELS: usize = 3;
/// The title every panel carries. **Fifteen columns**, which is the 15-cell instance's own width.
pub const TITLE: &str = " panel systems ";
/// How many widgets the two overflowing panels ask for. More than either density can fit.
pub const LONG: usize = 96;
/// How many the third panel asks for. Fewer than either density can fit, so its tail is unwritten.
pub const SHORT: usize = 20;
/// How wide a chip is.
pub const CHIP: u16 = 12;

/// How many cells the screen holds. 300 × 80.
pub const SCREEN: u64 = W as u64 * H as u64;

// ── the ledger ───────────────────────────────────────────────────────────────────────────────────
//
// **Every number this form is gated or reported on has exactly one home and it is here**, which is
// the runtime's ledger rule (`crates/vitui-runtime/src/ledger.rs`, an earlier pass) inherited unchanged.
// The audit that produced that file found one watchdog threshold copied into nine files.
//
// Every figure below is a **count**, not a timing, so it carries no machine: the form is
// deterministic, the labels are static, and the same numbers come out of a debug build and a
// release one. The rule is satisfied in the strong direction — *a gate is a count, a ratio, an
// equality or a compile outcome* — and there is no headroom to write beside a count that is exact.
//
// **Each is recorded beside the figure the design and an earlier pass remember**, and where they differ the
// difference is stated rather than engineered away. Those figures were taken on a prototype screen
// this ticket does not own: the 300×80 dense screen with 267–338 regions is scene 1 and 2 of
// `crate::scenes`, and an earlier pass stands it up. What reproduces here is every *direction*
// and every *structural* number; the magnitudes are a different screen's.

/// Rect the form writes at [`Density::Compact`]. **The recorded figure is 20 804, on another screen.**
pub const COMPACT_WRITES: u64 = 18_912;
/// Rect the form writes at [`Density::Cosy`]. **The recorded figure is 20 992.**
///
/// It is *more* than [`COMPACT_WRITES`] and fewer widgets are standing, which is the shape of
/// own pair and is not a paradox: a `Cosy` ring is two cells thicker on every side of every panel,
/// and the rows the lost widgets would have written were never written by anybody — they are part
/// of the third panel's unwritten tail either way.
pub const COSY_WRITES: u64 = 19_206;
/// Interactive regions at `Compact`. **The recorded figure is 267.**
pub const COMPACT_REGIONS: usize = 171;
/// Interactive regions at `Cosy`. **The recorded figure is 263** — four fewer, exactly as here.
pub const COSY_REGIONS: usize = 167;
/// **How many widgets fall off the bottom when the padding grows.** *four widgets*.
///
/// Two panels overflow and each loses two rows, so the number is `2 × (pad_y(Cosy) − pad_y(Compact))
/// × 2` and not a figure that was aimed at.
pub const DROPPED_BY_COSY: usize = 4;
/// `writes - distinct` for the naive arm at `Compact`. **The recorded figure is 43 941.**
///
/// Read as a differential with the equality filter off, which is the only way the number means
/// anything: the engine drops a rewrite of an identical value, so a build measured through the
/// filter scores its own defect as free.
pub const COMPACT_NAIVE_EXCESS: u64 = 40_925;
/// The same at `Cosy`.
pub const COSY_NAIVE_EXCESS: u64 = 40_191;
/// **What a `block` that clears what it hands over costs, at `Compact`** — cells written twice, on
/// every frame, whether anything moved or not.
///
/// **The recorded figure is 22 200 across three panels.** This form has three panels and hands over
/// [`COMPACT_HANDED_OVER`] cells; the double-write count is smaller than that by the third panel's
/// unwritten tail, because a cell the content never writes is cleared once and not twice. Both
/// numbers are here rather than one, since the ADR's figure is the *interior* and the gate's figure
/// is the *collision*.
pub const COMPACT_CLEARING_EXCESS: u64 = 16_224;
/// The same at `Cosy`.
pub const COSY_CLEARING_EXCESS: u64 = 15_510;
/// Rect the three `block` calls hand over at `Compact`.
///
/// **The 22 200-cell figure is this quantity**: a `block` that cleared what it hands over
/// would re-damage exactly this many cells on every frame, moving or not.
pub const COMPACT_HANDED_OVER: u64 = 21_312;
/// The same at `Cosy`.
pub const COSY_HANDED_OVER: u64 = 20_304;

/// The labels the rows cycle through. **Static, and the fixture's content is deliberately unused**:
/// this scene measures a construction, and a label whose length varied with the step would move the
/// write count for a reason that is not the construction.
const LABELS: [&str; 8] = [
    "throughput",
    "queue depth",
    "retries",
    "an unusually long metric name that will not fit",
    "latency p99",
    "errors",
    "connections",
    "backlog",
];

/// What a chip says.
const VALUES: [&str; 4] = ["on", "off", "auto", "degraded"];

/// Which of the three ways the form is drawn.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Style {
    Routed,
    ByHand,
    Naive,
    Clearing,
}

/// **What the form turned out to be at the density it was drawn at.**
///
/// Returned rather than printed, because criterion 6 asks for the widget count that falls off the
/// bottom to be *reported rather than hidden* — and a number that only a report prints is a number
/// no gate can read.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Shape {
    /// How many widgets the form asked for, across all three panels.
    pub requested: usize,
    /// How many were drawn.
    pub visible: usize,
    /// How many fell off the bottom because the padding is real.
    pub dropped: usize,
    /// How many cells the three `block` calls handed over unwritten.
    ///
    /// **The 22 200-cell figure's own quantity**: a `block` that cleared what it hands over would
    /// re-damage exactly this many cells on every frame, whether anything moved or not.
    pub handed_over: u64,
    /// How many interactive regions the form declared.
    pub regions: usize,
}

/// **Every row through the helpers.** The arm that ships.
pub fn routed(pen: &mut Pen, cx: &mut Ctx<'_, '_>, _fx: &Fixture) {
    draw(pen, cx, Style::Routed);
}

/// **The same order, with `fit`'s arithmetic written out at the call site.**
pub fn by_hand(pen: &mut Pen, cx: &mut Ctx<'_, '_>, _fx: &Fixture) {
    draw(pen, cx, Style::ByHand);
}

/// **The correct form, with one line added: every panel clears what `block` handed it.**
///
/// Criterion 2's measurement, isolated. It is priced at **22 200 damaged cells a frame across
/// three panels**, and it is the one instance of the five that a reviewer cannot see: the clear is
/// correct-looking, it is one line, and it makes the panel's own background reliably right. What it
/// costs is every interior cell the content then writes again, on every frame, moving or not.
pub fn clearing(pen: &mut Pen, cx: &mut Ctx<'_, '_>, _fx: &Fixture) {
    draw(pen, cx, Style::Clearing);
}

/// **A clear, a fill under every panel, and a face filled under every label.**
///
/// Three of the five instances at once, and every counter but the pair moves the wrong way:
/// it is not slower to *write*, it marks more, and it looks like an ordinary screen.
pub fn naive(pen: &mut Pen, cx: &mut Ctx<'_, '_>, _fx: &Fixture) {
    draw(pen, cx, Style::Naive);
}

/// One step, at the size the form is defined at. The form supplies its own content.
pub fn steps() -> Vec<Fixture> {
    vec![Fixture::of(W, H, Vec::new())]
}

/// Play one arm at one density.
pub fn play(density: Density, paint: crate::runner::Painter) -> Run {
    play_at(density, paint, &steps())
}

/// **The form's shape at a density**, measured by drawing it.
///
/// A frame is run rather than the arithmetic repeated, because a `Shape` computed beside the drawing
/// code is a second implementation of the layout and would agree with a wrong one.
pub fn shape(density: Density) -> Shape {
    let mut driver = crate::runner::driver_at(W, H, density);
    let mut shape = Shape::default();
    let mut pen = Pen::new(W, H);
    driver.frame(|cx| {
        shape = draw(&mut pen, cx, Style::Routed);
    });
    shape.regions = driver.inspect().hits().len();
    shape
}

/// The form itself. One function, three arms, so that what differs between them is the thing under
/// test and not the layout.
fn draw<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, style: Style) -> Shape {
    let theme = cx.theme();
    let body = theme.paint(Role::Body);
    let face = theme.paint(Role::Face);
    let dim = theme.paint(Role::Dim);

    let screen = cx.area();
    let mut shape = Shape::default();

    // **The screen clear that is a defect.** The largest instance — 6 662 cells a frame —
    // and the correct build's answer is *once, on the first frame and on a resize*, which is a
    // property of the application loop and not of this frame.
    if style == Style::Naive {
        wash(ink, cx, screen, body);
    }

    let (header, rest) = rect::split_at_v(screen, 1);
    let (band, footer) = rect::split_at_v(rest, rest.h - 1);
    let title_row = how(Justify::Start, Role::Title, Role::Title);
    let footer_row = how(Justify::End, Role::Dim, Role::Dim);
    let heading_row = how(Justify::Start, Role::Title, Role::Body);
    let label_row = how(Justify::Start, Role::Body, Role::Body);
    let chip_row = how(Justify::Middle, Role::Dim, Role::Face);
    row(ink, cx, style, header, "vitui — components", &title_row);
    row(ink, cx, style, footer, "ready", &footer_row);
    shape.regions += 2;
    let _ = cx.with_key(u64::MAX, |cx| {
        let id = cx.id();
        cx.interact(id, header, Interest::CLICK)
    });
    let _ = cx.with_key(u64::MAX - 1, |cx| {
        let id = cx.id();
        cx.interact(id, footer, Interest::CLICK)
    });

    let mut left = band;
    for panel in 0..PANELS {
        let (slot, next) = rect::split_at_h(left, band.w / PANELS as u16);
        left = next;
        let asked = if panel + 1 == PANELS { SHORT } else { LONG };
        shape.requested += asked;

        let opts = BlockOpts {
            title: TITLE,
            border: if panel == 0 {
                Role::Focus
            } else {
                Role::Border
            },
            ..BlockOpts::default()
        };
        let interior = block_into(ink, cx, slot, &opts);
        shape.handed_over += u64::from(interior.w) * u64::from(interior.h);

        // **A panel that clears what `block` handed it.** The 22 200-cell instance, one panel at a
        // time, and it is invisible to everything except the pair.
        if matches!(style, Style::Naive | Style::Clearing) {
            wash(ink, cx, interior, body);
        }

        let heading = row(ink, cx, style, interior, "metric", &heading_row);
        let mut rows = heading;
        let fits = usize::from(rows.h);
        let visible = asked.min(fits);
        shape.visible += visible;
        shape.dropped += asked - visible;

        cx.with_key(panel as u64, |cx| {
            for i in 0..visible {
                let (line, below) = rect::split_at_v(rows, 1);
                rows = below;
                let (label, chip) = rect::split_at_h(line, line.w.saturating_sub(CHIP));
                row(ink, cx, style, label, LABELS[i % LABELS.len()], &label_row);
                // **The chip: a face and a label, and the face is never filled first.** `fit`'s two
                // roles are the whole mechanism — the padding carries the face and the text carries
                // the foreground, so the row is one partition and not a fill under a draw.
                if style == Style::Naive {
                    wash(ink, cx, chip, face);
                    let value = VALUES[i % VALUES.len()];
                    let at = chip.x + (i32::from(chip.w) - i32::from(measure::width(value))) / 2;
                    ink.text(cx, at, chip.y, value, dim);
                } else {
                    row(ink, cx, style, chip, VALUES[i % VALUES.len()], &chip_row);
                }
                cx.with_key(i as u64, |cx| {
                    let id: Id = cx.id();
                    cx.interact(id, chip, Interest::CLICK.with(Interest::FOCUS));
                });
            }
        });
        shape.regions += visible + 1;
        let _ = cx.with_key(1_000 + panel as u64, |cx| {
            let id = cx.id();
            cx.interact(id, slot, Interest::HOVER)
        });
    }

    shape
}

/// The three roles a row is drawn with, as one value. A struct so the function below stays inside
/// clippy's argument budget, and a `const fn` so each call site reads as a constant.
const fn how(justify: Justify, role: Role, pad: Role) -> FitOpts {
    FitOpts { justify, role, pad }
}

/// One row, drawn the way `style` says. The single place the three arms diverge for text.
fn row<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    style: Style,
    area: Rect,
    s: &str,
    opts: &FitOpts,
) -> Rect {
    match style {
        Style::Routed | Style::Naive | Style::Clearing => fit_into(ink, cx, area, s, opts),
        Style::ByHand => hand_written(ink, cx, area, s, opts),
    }
}

/// **`fit`, written out at the call site**, which is what a component author writes when there is no
/// helper — truncation, a reserved cell for the marker, the justification arithmetic, and the two
/// padding runs.
///
/// It shares no line with [`fit_into`] and reaches [`crate::glyphs::elide`] not at all: the
/// truncation is `layout::text::truncate` and the marker is `Theme::glyph(Glyph::Ellipsis)` fetched
/// here. That is the engine's `reference.rs` rule — *share no code with the fast path* — restated
/// for a helper instead of for a compositor.
fn hand_written<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    s: &str,
    opts: &FitOpts,
) -> Rect {
    let (band, rest) = rect::split_at_v(area, 1);
    if band.is_empty() {
        return rest;
    }
    let theme = cx.theme();
    let text_paint = theme.paint(opts.role);
    let pad_paint = theme.paint(opts.pad);
    let w = band.w;

    let (head, marker) = if measure::width(s) <= w {
        (s, "")
    } else if w == 0 {
        ("", "")
    } else {
        (measure::truncate(s, w - 1), theme.glyph(Glyph::Ellipsis))
    };
    let head_w = measure::width(head);
    let mark_w = measure::width(marker);
    let used = head_w + mark_w;
    let slack = w - used;
    let lead = match opts.justify {
        Justify::Start => 0,
        Justify::Middle => slack / 2,
        Justify::End => slack,
    };
    let trail = slack - lead;

    let x = band.x;
    let y = band.y;
    if lead > 0 {
        ink.run(cx, x, y, " ", lead, pad_paint);
    }
    if head_w > 0 {
        ink.text(cx, x + i32::from(lead), y, head, text_paint);
    }
    if mark_w > 0 {
        ink.text(cx, x + i32::from(lead + head_w), y, marker, text_paint);
    }
    if trail > 0 {
        ink.run(cx, x + i32::from(lead + used), y, " ", trail, pad_paint);
    }
    rest
}

/// Write every cell of `cells` as a space. **The verb the naive arm is made of**, and the one no
/// correct arm calls on a rectangle it is about to hand over.
fn wash<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, cells: Rect, st: Paint) {
    if cells.is_empty() {
        return;
    }
    let x = cells.x;
    for r in 0..cells.h {
        ink.run(cx, x, cells.y + i32::from(r), " ", cells.w, st);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counters::{Allocations, Counter};
    use crate::runner::{Painter, compare_at};

    /// The three arms, so a sweep does not have to spell them.
    fn arms() -> [(&'static str, Painter); 3] {
        [
            ("routed", routed as Painter),
            ("by_hand", by_hand as Painter),
            ("naive", naive as Painter),
        ]
    }

    fn writes_and_distinct(density: Density, paint: Painter) -> (u64, u64) {
        let run = play(density, paint);
        (run.tally().writes(), run.tally().distinct())
    }

    /// **Criterion 3: routing through `fit` costs nothing against the discipline.**
    ///
    /// The measurement the helper table owed. One screen, rendered once through
    /// [`crate::text::fit`] and once with `fit`'s truncation, justification and padding written out
    /// at the call site, compared **cell for cell** — and the write counts compared beside it,
    /// because a build that merely drew less would score zero differing cells by drawing nothing,
    /// which is what three separate defects on this map did.
    ///
    /// Shipped: **0 of 24 000 cells differ at 18 912 writes against 18 912** at `Compact`, and
    /// 19 206 against 19 206 at `Cosy`. The recorded figures are 0 of 24 000 at 20 804 against 20 804, on
    /// the dense screen one module over.
    #[test]
    fn routing_through_fit_is_zero_cells_different_at_equal_write_counts() {
        for (density, expected) in [
            (Density::Compact, COMPACT_WRITES),
            (Density::Cosy, COSY_WRITES),
        ] {
            let (routed_writes, routed_distinct) = writes_and_distinct(density, routed);
            let (hand_writes, hand_distinct) = writes_and_distinct(density, by_hand);
            assert_eq!(
                routed_writes, hand_writes,
                "{density:?}: the helper and the hand-written order write different numbers of \
                 cells"
            );
            assert_eq!(routed_distinct, hand_distinct);
            assert_eq!(
                routed_writes, expected,
                "{density:?}: the form's write count moved. It is a count and not a timing, so it \
                 moved because the construction did"
            );
            assert!(
                routed_writes > 0 && hand_writes > 0,
                "{density:?}: an arm that drew nothing agrees with everything"
            );

            let diff = compare_at(density, routed, by_hand, &steps());
            assert!(
                diff.clean(),
                "{density:?}: {diff}. The helper is not a convenience over hand-written \
                 correctness — it is the only form of it anybody keeps, and that claim is this \
                 equality"
            );
        }
    }

    /// **Criterion 4: `writes == distinct cells touched`, as a gate over both helpers.**
    ///
    /// The form is built out of nothing but `fit` and `block`, so the equality holding over the
    /// whole screen is the equality holding over both of them together — which is the half a
    /// per-helper test cannot reach, because the interesting double writes are *between* two
    /// branches that are each correct alone.
    ///
    /// `reported == writes` is asserted first. Without it both sides of the pair would be this
    /// crate's own arithmetic and the equality would be checking a model against itself
    /// (`crate::counters::Tally::reported`).
    #[test]
    fn neither_helper_writes_a_cell_twice_at_either_density() {
        for density in [Density::Compact, Density::Cosy] {
            for (name, paint) in arms().into_iter().take(2) {
                let run = play(density, paint);
                let tally = run.tally();
                assert_eq!(
                    tally.reported(),
                    tally.writes(),
                    "{density:?} {name}: a write the engine did not report"
                );
                assert_eq!(
                    tally.writes(),
                    tally.distinct(),
                    "{density:?} {name}: {} cells written twice",
                    tally.writes() - tally.distinct()
                );
                assert_eq!(
                    tally.asked(),
                    tally.reported(),
                    "{density:?} {name}: a verb left its own rectangle and the clip discarded it"
                );
            }
        }
    }

    /// **Criterion 4's other half: the differential, watched separating.**
    ///
    /// A gate nobody has watched fail is not a gate, and *both* directions matter here — the naive
    /// arm exists so the correct arm's `0` means something. Every counter but this one moves the
    /// wrong way: the naive screen touches **more** cells (24 000 of 24 000 against 18 912), which
    /// is the metric looking *healthier* on the defective build.
    #[test]
    fn the_naive_arm_writes_forty_thousand_cells_it_had_already_written() {
        for (density, excess) in [
            (Density::Compact, COMPACT_NAIVE_EXCESS),
            (Density::Cosy, COSY_NAIVE_EXCESS),
        ] {
            let (naive_writes, naive_distinct) = writes_and_distinct(density, naive);
            let (good_writes, good_distinct) = writes_and_distinct(density, routed);
            assert_eq!(good_writes - good_distinct, 0);
            assert_eq!(
                naive_writes - naive_distinct,
                excess,
                "{density:?}: the naive differential moved"
            );
            assert_eq!(
                naive_distinct, SCREEN,
                "{density:?}: the naive arm covers the screen, which is the counter that makes it \
                 look better than the correct one"
            );
            assert!(
                naive_distinct > good_distinct,
                "{density:?}: the defective build touches more cells — {naive_distinct} against \
                 {good_distinct} — so *no cell never* scores it as the healthier one"
            );
        }
    }

    /// **Criterion 2: a `block` that clears what it hands over, priced.**
    ///
    /// One line added to the correct form — `wash(interior)` right after `block` returns it — and
    /// the screen is pixel for pixel identical. Nothing about it looks wrong: it is the line an
    /// author writes to make a panel's background reliably correct, and it makes the panel's
    /// background reliably correct. What it costs is **16 224 cells written twice every frame** at
    /// `Compact` and 15 510 at `Cosy`, on a screen that is not moving.
    ///
    /// The recorded figure is **22 200 damaged cells a frame across three panels**. The quantity that
    /// figure is about is the interior itself — [`COMPACT_HANDED_OVER`], 21 312 here — and the
    /// collision count is smaller by the third panel's unwritten tail, because a cell the content
    /// never writes is cleared once rather than twice. Both are asserted, so neither can drift into
    /// standing for the other.
    #[test]
    fn a_block_that_clears_what_it_hands_over_writes_sixteen_thousand_cells_twice() {
        for (density, excess, handed) in [
            (
                Density::Compact,
                COMPACT_CLEARING_EXCESS,
                COMPACT_HANDED_OVER,
            ),
            (Density::Cosy, COSY_CLEARING_EXCESS, COSY_HANDED_OVER),
        ] {
            let (writes, distinct) = writes_and_distinct(density, clearing);
            assert_eq!(
                writes - distinct,
                excess,
                "{density:?}: the price of clearing what `block` hands over moved"
            );
            let unwritten = SCREEN - play(density, routed).tally().distinct();
            assert_eq!(
                excess + unwritten,
                handed,
                "{density:?}: the collision count plus the tail is the interior, which is what \
                 makes the two numbers one fact"
            );
            assert_eq!(
                distinct, SCREEN,
                "{density:?}: clearing the interiors also covers the screen — the defective build \
                 scores better on *no cell never* than the correct one"
            );
            // **The two halves of the rule meeting on one screen**, which is the thing the ADR
            // says neither half resolves alone: the clearing build differs from the correct one on
            // exactly the cells nobody wrote, and nowhere else. Filling every frame is the
            // double-write defect; not filling is the unwritten one. The 16 224 cells above are the
            // first and these 5 088 are the second, on the same form, at the same moment.
            let diff = compare_at(density, routed, clearing, &steps());
            assert_eq!(
                diff.cells as u64, unwritten,
                "{density:?}: {diff}. The clearing build should differ from the correct one on the \
                 unwritten tail and on nothing else — every cell either build wrote, it wrote the \
                 same value into"
            );
        }
    }

    /// **Criterion 6: the same form at two densities, and what falls off the bottom is reported.**
    ///
    /// Neither writes a cell twice — that is the test above, over both densities — and the numbers
    /// here are the two stated as a pair: `Compact` stands more regions and writes
    /// fewer cells than `Cosy`, and four widgets do not fit.
    ///
    /// **The four is arithmetic and not a target.** `pad_y` is 1 at `Compact` and 2 at `Cosy`, two
    /// of the three panels ask for more widgets than fit, and each of those two loses one row at the
    /// top and one at the bottom.
    #[test]
    fn compact_and_cosy_are_the_same_form_and_four_widgets_fall_off_the_bottom() {
        let compact = shape(Density::Compact);
        let cosy = shape(Density::Cosy);

        assert_eq!(compact.regions, COMPACT_REGIONS);
        assert_eq!(cosy.regions, COSY_REGIONS);
        assert_eq!(
            compact.regions - cosy.regions,
            DROPPED_BY_COSY,
            "a widget that does not fit is a region that is not declared"
        );
        assert_eq!(
            cosy.dropped - compact.dropped,
            DROPPED_BY_COSY,
            "four widgets fall off the bottom because the padding is real"
        );
        assert_eq!(compact.visible - cosy.visible, DROPPED_BY_COSY);
        assert_eq!(
            (compact.requested, cosy.requested),
            (2 * LONG + SHORT, 2 * LONG + SHORT),
            "both densities are asked for the same form"
        );
        assert!(
            compact.dropped > 0,
            "a form that fits at both densities measures nothing about density"
        );

        assert_eq!(compact.handed_over, COMPACT_HANDED_OVER);
        assert_eq!(cosy.handed_over, COSY_HANDED_OVER);
        assert!(
            compact.handed_over > cosy.handed_over,
            "a thicker ring is a smaller interior"
        );

        let (compact_writes, _) = writes_and_distinct(Density::Compact, routed);
        let (cosy_writes, _) = writes_and_distinct(Density::Cosy, routed);
        assert_eq!((compact_writes, cosy_writes), (COMPACT_WRITES, COSY_WRITES));
        assert!(
            cosy_writes > compact_writes,
            "the pair spec §3 states runs this way round: {cosy_writes} against {compact_writes}"
        );
    }

    /// **The form leaves a tail nobody writes, and says how big it is.**
    ///
    /// The sentinel gate — *every cell of the rectangle written at least once* — was pinned red at
    /// 9 956 cells of 53 280 over six panels of twelve when this was written, with the note that its
    /// detector was unreachable from this crate. **Components 40 inverted it, and this
    /// tail is deliberately still here**: the count is read off `crate::runner::Pen` — the same
    /// recorder `distinct` below comes from — and the row's population is *components* and the
    /// assembled gallery, not every screen this crate can draw. This form is a fixture, and its tail
    /// is what buys the density pair and the naive differential; see this module's header for why
    /// that is a named exception rather than a gap.
    ///
    /// So what this asserts is unchanged and its subject is not: the third panel asks for fewer
    /// widgets than fit, the rows below them are written by nobody, and the model knows it.
    #[test]
    fn the_short_panels_tail_is_written_by_nobody_and_the_count_says_so() {
        for (density, writes) in [
            (Density::Compact, COMPACT_WRITES),
            (Density::Cosy, COSY_WRITES),
        ] {
            let run = play(density, routed);
            let unwritten = SCREEN - run.tally().distinct();
            assert_eq!(run.tally().distinct(), writes);
            assert!(
                unwritten > 0,
                "{density:?}: the form covers its screen, so the tail two of ticket 06's three \
                 measurements rest on is gone"
            );
            assert_eq!(
                unwritten,
                SCREEN - writes,
                "{density:?}: {unwritten} cells of {SCREEN} keep whatever was there before — which \
                 is what makes them invisible until a theme changes"
            );
        }
    }

    /// **No two widgets on this screen share an id** — the free detector, on the first
    /// screen this crate has drawn.
    ///
    /// 110 of 338 widgets were inert on the first screen written here, and the
    /// screen rendered pixel for pixel correctly, which is why this is asserted and not eyeballed.
    /// The form takes its ids **outside every closure** and keys both loops.
    #[test]
    fn the_form_declares_no_colliding_ids() {
        for density in [Density::Compact, Density::Cosy] {
            let run = play(density, routed);
            let counters = run.counters(Allocations::over(1, 0));
            assert_eq!(
                counters.get(Counter::Merges).get(Counter::Merges),
                0,
                "{density:?}: a widget claimed an id another widget already held, and the screen \
                 still renders correctly"
            );
        }
    }
}
