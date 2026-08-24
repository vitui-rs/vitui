//! **F1 text**, ~48 entries, expressed by `text`, `chip`, `field` and the [`fit`] helper.
//!
//! The reduction is R1 and R5 (spec §18): markdown needs its own wrap pass and is §22's, and there
//! is no bidi — a stated non-goal with the engine's tables named as the reason.
//!
//! `field` declares this family and is **not** homed here: its first family is F6, which is where
//! the caret, the wrap index and the keyboard live. Spelled out because `text` and `field` sharing
//! `layout::text` is the reason a reader would expect otherwise.
//!
//! # `fit` is the partition primitive
//!
//! Spec §3's table gives it one job — *truncation, alignment, padding* — and one shape: **text, then
//! the remainder; there is no verb on it that fills first.** That sentence is the whole helper. A
//! label centred in a rectangle is three writes that touch every cell of its row exactly once — the
//! lead, the text, the trail — and never a fill followed by a draw, which is R07's original defect
//! and 26 of 48 cells on a steady dropdown frame (ADR 0026).
//!
//! **Routing through it costs nothing against the discipline**, and that is the measurement spec
//! §3's table owed rather than a claim about ergonomics: `crate::form` renders one screen twice, once
//! through `fit` and once with the same order written out by hand, and compares them cell for cell.
//! The helper is not a convenience over hand-written correctness — it is the only form of it anybody
//! keeps.

use vitui_runtime::layout::text as measure;
use vitui_runtime::{Ctx, Role};

use crate::cells::Cells;
use crate::glyphs::elide;
use crate::ink::{Direct, Ink};

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["text", "chip"];

/// The cluster a padding run is made of. One cell, single width, and the only thing `fit` writes
/// that is not the caller's text.
const PAD: &str = " ";

/// Where the text sits in the row it is fitted into.
///
/// Named `Justify` and not `Align` or `Fit`: [`vitui_runtime::layout::Align`] and
/// [`vitui_runtime::layout::Fit`] both exist and mean something else, and a component author reading
/// two `Align`s in one call has to check which crate each came from.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Justify {
    /// Against the left edge, the padding after it. The default.
    #[default]
    Start,
    /// Centred, with the odd cell going to the trailing side — `(w - used) / 2` leads.
    Middle,
    /// Against the right edge, the padding before it.
    End,
}

/// [`fit`]'s options.
///
/// Spec §1's rule 3: **options are a `Default` struct, never a required builder**, and every helper
/// `f` has a sibling `f_with` that takes one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FitOpts {
    /// Where the text sits in its row.
    pub justify: Justify,
    /// The role the text is painted in.
    pub role: Role,
    /// The role the padding is painted in.
    ///
    /// **Separate from `role` on purpose.** A chip's label and a chip's face are one rectangle and
    /// two paints, and a caller that had to pass one role would fill the face first to get the
    /// second — which is R07's defect arriving through the helper that exists to prevent it.
    pub pad: Role,
}

impl Default for FitOpts {
    fn default() -> FitOpts {
        FitOpts {
            justify: Justify::Start,
            role: Role::Body,
            pad: Role::Body,
        }
    }
}

/// **Write `s` into the first row of `area`, and return the rows below it.**
///
/// The ninety-per-cent spelling: left-justified, [`Role::Body`] for both halves.
///
/// ```
/// use vitui_components::cells::Cells;
/// use vitui_components::text::fit;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(20, 3).expect("a sink attaches");
/// driver.frame(|cx| {
///     let area = Cells::of(cx);
///     let rest = fit(cx, area, "one");
///     let rest = fit(cx, rest, "two");
///     // Two rows written, one row returned — and the caller owns it.
///     assert_eq!((rest.y(), rest.h()), (2, 1));
/// });
/// ```
pub fn fit(cx: &mut Ctx<'_, '_>, area: Cells, s: &str) -> Cells {
    fit_with(cx, area, s, &FitOpts::default())
}

/// [`fit`], with the options spelled out.
pub fn fit_with(cx: &mut Ctx<'_, '_>, area: Cells, s: &str, opts: &FitOpts) -> Cells {
    fit_into(&mut Direct, cx, area, s, opts)
}

/// **[`fit`], drawing through an [`Ink`] so a counter can see the verbs.**
///
/// The entry point a gate takes; `fit` is this with [`Direct`]. There is no second implementation —
/// [`crate::ink`]'s header is why, and it is the same argument `crate::counters` makes one file
/// over: a gate written against a copy of the code tests the copy.
///
/// # What it writes, and in what order
///
/// The first row of `area`, left to right and never twice: the lead padding, the head of the text,
/// the one-cell ellipsis if there is one, and the trail padding. Every one of the four is skipped
/// when it is empty, which is what keeps `verbs` honest — a label that exactly fills its row is one
/// verb and not three.
///
/// # The one-cell ellipsis is enforced here
///
/// `fit` is where truncation is decided, so it is where §16's rule has to hold: [`elide`] reserves
/// **one** cell for the marker and `Theme::glyph` guarantees every spelling is one cell wide.
/// A three-cell `...` where one was reserved has `writes`, `verbs` and `marked` identical either
/// way — the defect has no signature at any counter and only the surface disagrees — which is why
/// the rule lives in a call rather than in a comment.
pub fn fit_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Cells,
    s: &str,
    opts: &FitOpts,
) -> Cells {
    let (band, rest) = area.split_at_v(1);
    if band.is_empty() {
        return rest;
    }
    // `theme()` borrows the environment and not the context, so the paints outlive the verbs below.
    let theme = cx.theme();
    let text_paint = theme.paint(opts.role);
    let pad_paint = theme.paint(opts.pad);

    let w = band.w();
    let (head, marker) = elide(theme, s, w);
    let head_w = measure::width(head);
    let mark_w = measure::width(marker);
    let used = head_w + mark_w;
    debug_assert!(
        used <= w,
        "`elide` returned {used} cells for a {w}-cell row: the one-cell ellipsis rule has moved"
    );
    let slack = w - used;
    let lead = match opts.justify {
        Justify::Start => 0,
        Justify::Middle => slack / 2,
        Justify::End => slack,
    };
    let trail = slack - lead;

    let y = i32::from(band.y());
    let x = i32::from(band.x());
    ink.run(cx, x, y, PAD, lead, pad_paint);
    if head_w > 0 {
        ink.text(cx, x + i32::from(lead), y, head, text_paint);
    }
    if mark_w > 0 {
        ink.text(cx, x + i32::from(lead + head_w), y, marker, text_paint);
    }
    ink.run(cx, x + i32::from(lead + used), y, PAD, trail, pad_paint);
    rest
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counters::Tally;
    use vitui_runtime::ctx::Driver;

    /// A tally over one frame of `f`, on a `w` by `h` sink.
    fn tallied(w: u16, h: u16, f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>)) -> Tally {
        let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| f(&mut tally, cx));
        tally
    }

    /// **`fit` writes its row exactly once and nothing else** — spec §2's first equality, on the
    /// helper it is stated about, and the second equality restricted to the band.
    ///
    /// Swept over every width and every justification, because the interesting cases are the ones
    /// where the arithmetic runs out: a row narrower than its text, a row exactly as wide, and a row
    /// one cell wide where the ellipsis has nowhere to go.
    #[test]
    fn fit_writes_a_partition_of_its_row_at_every_width_and_justification() {
        for justify in [Justify::Start, Justify::Middle, Justify::End] {
            for w in 0u16..24 {
                for label in ["", "a", "hello", "a rather longer label than fits"] {
                    let opts = FitOpts {
                        justify,
                        ..FitOpts::default()
                    };
                    let tally = tallied(w.max(1), 3, |tally, cx| {
                        let area = Cells::at(0, 0, w, 1);
                        let rest = fit_into(tally, cx, area, label, &opts);
                        assert!(rest.is_empty(), "a one-row area leaves no remainder");
                    });
                    assert_eq!(
                        tally.writes(),
                        tally.distinct(),
                        "{justify:?} `{label}` at {w}: a cell was written twice"
                    );
                    assert_eq!(
                        tally.distinct(),
                        u64::from(w),
                        "{justify:?} `{label}` at {w}: the row is not covered"
                    );
                    assert_eq!(
                        tally.reported(),
                        tally.writes(),
                        "{justify:?} `{label}` at {w}: a write the engine did not report"
                    );
                }
            }
        }
    }

    /// **The remainder is the rest of the rectangle, and it is untouched.**
    ///
    /// The first half of criterion 1: `fit` returns what it did not write, and a chain of them
    /// partitions a rectangle into rows.
    #[test]
    fn fit_returns_the_rows_it_did_not_write() {
        let tally = tallied(12, 5, |tally, cx| {
            let area = Cells::of(cx);
            let opts = FitOpts::default();
            let mut rest = area;
            for line in ["one", "two"] {
                rest = fit_into(tally, cx, rest, line, &opts);
            }
            assert_eq!((rest.x(), rest.y(), rest.w(), rest.h()), (0, 2, 12, 3));
        });
        assert_eq!(tally.writes(), 24, "two rows of twelve");
        assert_eq!(tally.distinct(), 24);
        for y in 2..5 {
            for x in 0..12 {
                assert!(
                    !tally.touched(x, y),
                    "({x}, {y}) is the caller's and was written"
                );
            }
        }
    }

    /// **A chain that runs off the bottom keeps returning empty rectangles in the right place**, so
    /// a caller with more content than room writes nothing outside its own rectangle.
    #[test]
    fn a_chain_that_runs_out_of_rows_writes_nothing_more() {
        let tally = tallied(8, 2, |tally, cx| {
            let opts = FitOpts::default();
            let mut rest = Cells::of(cx);
            for i in 0..6 {
                rest = fit_into(tally, cx, rest, &format!("row {i}"), &opts);
                if i >= 2 {
                    assert!(rest.is_empty());
                    assert_eq!(
                        rest.y(),
                        2,
                        "the empty remainder stays where the split left it"
                    );
                }
            }
        });
        assert_eq!(
            tally.writes(),
            16,
            "two rows of eight and not one cell more"
        );
        // Two verbs a row — `row 0` is five columns and three of padding trail it — and **four
        // calls past the bottom add none**, which is the half this test is about.
        assert_eq!(
            tally.verbs(),
            4,
            "a call onto an empty rectangle is not a verb"
        );
    }

    /// **The one-cell ellipsis, enforced where truncation is decided** — criterion 8.
    ///
    /// Two claims, and the second is the one ticket 05 could not make from `glyphs` alone: the
    /// marker is one cell, *and* the head plus the marker never exceed the row. A three-cell marker
    /// would leave `writes`, `verbs` and `distinct` unmoved inside a narrowed context and only the
    /// surface would disagree.
    #[test]
    fn a_truncated_label_ends_in_exactly_one_cell_of_ellipsis() {
        let mut driver = Driver::headless(40, 1).expect("a sink cannot fail to attach");
        driver.frame(|cx| {
            let theme = cx.theme();
            for w in 1u16..20 {
                let (head, marker) = elide(theme, "a label that is far too long", w);
                assert_eq!(
                    measure::width(marker),
                    1,
                    "the marker is not one cell at {w}"
                );
                assert!(
                    measure::width(head) + measure::width(marker) <= w,
                    "head plus marker overruns a {w}-cell row"
                );
            }
            // Nothing is cut, so nothing is marked.
            let (head, marker) = elide(theme, "short", 20);
            assert_eq!((head, marker), ("short", ""));
        });
    }

    /// **Centring puts the odd cell on the trailing side**, which is a decision and not an accident:
    /// a column of centred labels of mixed parity has to agree with itself, and `(w - used) / 2`
    /// truncating is what makes it.
    #[test]
    fn centring_leads_with_the_floor_and_trails_with_the_remainder() {
        let tally = tallied(9, 1, |tally, cx| {
            let opts = FitOpts {
                justify: Justify::Middle,
                ..FitOpts::default()
            };
            fit_into(tally, cx, Cells::at(0, 0, 9, 1), "abcd", &opts);
        });
        assert_eq!(tally.writes(), 9);
        assert_eq!(tally.verbs(), 3, "lead, text, trail");
        // (9 - 4) / 2 == 2 leading, so the text occupies columns 2..6 and three cells trail.
        for x in 0..9 {
            assert!(tally.touched(x, 0));
        }
    }
}
