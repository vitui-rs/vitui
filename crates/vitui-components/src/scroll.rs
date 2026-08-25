//! **F3 scrolling**, 18 entries, expressed by `scroll_area`, `scrollbar`, `sticky` and
//! `collection` — and [`bar`], the helper every one of them draws.
//!
//! The reduction is R2 and R5 (spec §18); `pull-to-refresh` is the family's one residue entry, a
//! touch gesture with no terminal meaning and nobody who could change that.
//!
//! `collection` declares this family and is homed under F7. Spec §9's C21 is why the distinction
//! is worth the confusion: **a `scroll_area` costs its content and a virtualised `collection` costs
//! its visible window**, and the wrong pairing is 7 907 us at 100 000 rows — seventy-nine budgets.
//!
//! # `bar` draws the thumb first, and that is the whole helper
//!
//! Spec §3's table gives it one job — *the bar every scrollable draws* — and one shape: **thumb,
//! then the track above and below.** It is spec §2's partition rule at the smallest scale it comes
//! in, and it is a helper rather than a paragraph because the other order is what everybody writes:
//! fill the groove, then put the thumb on it. That reads correctly, it is one verb shorter, and it
//! writes every cell of the thumb **twice** — [`TRACK_FIRST_EXCESS`] cells a frame on a two-bar
//! screen, on every frame, whether anything scrolled or not.
//!
//! Nothing about it looks like a fill, which is [`crate::frame`]'s fifteen-cell instance again one
//! helper down: ADR 0026 found the same shape in five places and three of them were not fills.
//!
//! # The unit is content cells, and it comes from one place
//!
//! Spec §9, collecting C05's debt: *the extent, the offset, the thumb and scroll-into-view are all
//! in content cells and all come from one place.* Measured in **rows** instead, on content where
//! one row in eight is three cells tall, the extent reads 1 000 000 where it is 1 250 000 — *time,
//! writes, verbs, marked cells, regions and allocations are all identical*, the area reaches row
//! 799 999 of 999 999, and the thumb is out by up to **7 cells of 69**, smoothly and plausibly. So
//! [`Span`] takes three `u32`s in one unit and [`thumb`] is the only arithmetic that turns them
//! into cells.

use vitui_runtime::{Ctx, Glyph, Role};

use crate::ink::{Direct, Ink};
use vitui_runtime::Rect;

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["scroll_area", "scrollbar", "sticky"];

/// Which way a bar runs.
///
/// Named `Orient` and not `Axis`: [`crate::Axis`] is re-exported at this crate's root and means one
/// of §17's four **hostile axes**, and a reader meeting two `Axis`es in one file has to check which
/// crate each came from — [`crate::text::Justify`]'s naming note, for its reason.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Orient {
    /// Down the right-hand edge. The default, because most content is longer than it is wide.
    #[default]
    Vertical,
    /// Along the bottom edge.
    Horizontal,
}

/// **Where the content is, in content cells.**
///
/// Three numbers in one unit, which is spec §9's rule rather than an ergonomic grouping: an extent
/// in rows against a viewport in cells is a defect that moves no counter at all.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Span {
    /// How many content cells are visible along this axis.
    pub viewport: u32,
    /// How many content cells there are in total. **`Σ h`, never a row count** (spec §9).
    pub extent: u32,
    /// The first visible content cell.
    pub offset: u32,
}

/// [`bar`]'s options.
///
/// Spec §1's rule 3: a `Default` struct, never a required builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BarOpts {
    /// Which way it runs.
    pub orient: Orient,
    /// The role the groove is drawn in.
    pub track: Role,
    /// The role the thumb is drawn in.
    pub thumb: Role,
}

impl Default for BarOpts {
    fn default() -> BarOpts {
        BarOpts {
            orient: Orient::Vertical,
            track: Role::Border,
            thumb: Role::Face,
        }
    }
}

/// **Where the thumb sits in a track of `track` cells: `(start, length)`.**
///
/// The only arithmetic that turns a [`Span`] into cells, which is spec §9's *all come from one
/// place* as a function rather than as a sentence.
///
/// Three cases, and the first two are the ones a bar drawn by hand gets wrong:
///
/// - **content that fits** is a thumb filling the whole track, not an absent bar — the bar is
///   reserved either way (ADR 0029), so the groove has to say *there is nothing below*;
/// - **a thumb is never shorter than one cell**, or a million-row area has no thumb at all;
/// - **the travel is `track - length`**, so the last content cell puts the thumb against the far
///   end exactly rather than one short of it.
pub fn thumb(track: u16, span: Span) -> (u16, u16) {
    if track == 0 {
        return (0, 0);
    }
    if span.extent <= span.viewport || span.viewport == 0 {
        return (0, track);
    }
    let len = (u64::from(track) * u64::from(span.viewport) / u64::from(span.extent)) as u16;
    let len = len.clamp(1, track);
    let travel = track - len;
    let furthest = span.extent - span.viewport;
    let offset = span.offset.min(furthest);
    let start = (u64::from(travel) * u64::from(offset) / u64::from(furthest)) as u16;
    (start.min(travel), len)
}

/// **Draw a scrollbar: the thumb, then the track above and below it. Returns the thumb.**
///
/// The ninety-per-cent spelling: vertical, the theme's border and face roles.
///
/// # It returns the thumb and not a remainder, because a bar has none
///
/// Every other helper on this map returns *the cells it did not write* (spec §2). A bar writes all
/// of its rectangle, so the return value would always be empty — and the one rectangle a caller
/// needs afterwards is the thumb, which is the drag target.
///
/// ```
/// use vitui_runtime::Rect;
/// use vitui_components::scroll::{Span, bar};
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(40, 10).expect("a sink attaches");
/// driver.frame(|cx| {
///     let track = Rect::new(39, 0, 1, 10);
///     let span = Span { viewport: 10, extent: 40, offset: 0 };
///     let thumb = bar(cx, track, span);
///     // A quarter of the content is visible, so a quarter of the track is thumb.
///     assert_eq!((thumb.y, thumb.h), (0, 2));
/// });
/// ```
pub fn bar(cx: &mut Ctx<'_, '_>, area: Rect, span: Span) -> Rect {
    bar_with(cx, area, span, &BarOpts::default())
}

/// [`bar`], with the options spelled out.
pub fn bar_with(cx: &mut Ctx<'_, '_>, area: Rect, span: Span, opts: &BarOpts) -> Rect {
    bar_into(&mut Direct, cx, area, span, opts)
}

/// **[`bar`], drawing through an [`Ink`] so a counter can see the verbs.**
///
/// The entry point a gate takes. See [`crate::ink`] for why the seam exists rather than a second
/// implementation of this function.
pub fn bar_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    span: Span,
    opts: &BarOpts,
) -> Rect {
    draw(ink, cx, area, span, opts, true)
}

/// **The thumb, then the two stretches of track — or, with `thumb_first` false, the defect.**
///
/// One function with one boolean between the correct bar and the track-first one, which is
/// [`crate::frame::draw`]'s arrangement and its reason: the diff a reviewer would have to catch is
/// the diff the register can point at.
fn draw<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    span: Span,
    opts: &BarOpts,
    thumb_first: bool,
) -> Rect {
    let theme = cx.theme();
    let track_paint = theme.paint(opts.track);
    let thumb_paint = theme.paint(opts.thumb);
    let track_glyph = theme.glyph(Glyph::Track);
    let thumb_glyph = theme.glyph(Glyph::Thumb);

    let length = match opts.orient {
        Orient::Vertical => area.h,
        Orient::Horizontal => area.w,
    };
    if area.is_empty() {
        return Rect::new(area.x, area.y, 0, 0);
    }
    let (start, len) = thumb(length, span);
    let thumb_cells = match opts.orient {
        Orient::Vertical => Rect::new(area.x, area.y + i32::from(start), area.w, len),
        Orient::Horizontal => Rect::new(area.x + i32::from(start), area.y, len, area.h),
    };

    if thumb_first {
        stripe(
            ink,
            cx,
            area,
            opts.orient,
            start,
            len,
            thumb_glyph,
            thumb_paint,
        );
        stripe(
            ink,
            cx,
            area,
            opts.orient,
            0,
            start,
            track_glyph,
            track_paint,
        );
        stripe(
            ink,
            cx,
            area,
            opts.orient,
            start + len,
            length - start - len,
            track_glyph,
            track_paint,
        );
    } else {
        // **The defect.** The whole groove, and the thumb laid over it: every cell of the thumb
        // written twice, with two different values, in one frame.
        stripe(
            ink,
            cx,
            area,
            opts.orient,
            0,
            length,
            track_glyph,
            track_paint,
        );
        stripe(
            ink,
            cx,
            area,
            opts.orient,
            start,
            len,
            thumb_glyph,
            thumb_paint,
        );
    }
    thumb_cells
}

/// Write `n` cells of `cluster` along `orient`, starting `from` cells into `area`.
///
/// A vertical bar is one verb a row, because [`Ink::run`] writes one row: the bar is a column and a
/// column is not a run. That is why `verbs` is the counter this helper is *worst* on and the pair is
/// the one it is judged by — §21's *`verbs <= writes`* is a relation for exactly this reason.
#[allow(clippy::too_many_arguments)]
fn stripe<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    orient: Orient,
    from: u16,
    n: u16,
    cluster: &str,
    paint: vitui_runtime::Paint,
) {
    if n == 0 {
        return;
    }
    match orient {
        Orient::Vertical => {
            for i in 0..n {
                ink.run(
                    cx,
                    area.x,
                    area.y + i32::from(from + i),
                    cluster,
                    area.w,
                    paint,
                );
            }
        }
        Orient::Horizontal => {
            for row in 0..area.h {
                ink.run(
                    cx,
                    area.x + i32::from(from),
                    area.y + i32::from(row),
                    cluster,
                    n,
                    paint,
                );
            }
        }
    }
}

/// **The bar everybody writes: the groove, then the thumb on top of it.**
///
/// `pub` for the reason [`crate::frame::defective`] and [`crate::runner::defective`] are: an
/// instrument crate's fixtures are part of the instrument, and a gate validated only against a
/// correct build reports zero for the same reason a broken one would.
pub mod defective {
    use super::{BarOpts, Ctx, Ink, Rect, Span, draw};

    /// **The groove first, then the thumb over it.**
    ///
    /// Everything about this bar is correct — the same cells, the same glyphs, the same paints, the
    /// same thumb in the same place — so the only thing that separates the two builds is
    /// `writes - distinct`, and the defective one is *cheaper*: one verb per stripe instead of two
    /// on the axis that matters, and it reads the way a groove-and-thumb is drawn everywhere else.
    pub fn track_first<I: Ink>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        span: Span,
        opts: &BarOpts,
    ) -> Rect {
        draw(ink, cx, area, span, opts, false)
    }
}

// ── the two-bar screen ───────────────────────────────────────────────────────────────────────────

/// The screen's width. §20 prices every dense screen on this map at 300×80.
pub const W: u16 = 300;
/// The screen's height.
pub const H: u16 = 80;
/// The vertical extent the screen's content has, in content cells. §9's own screen: a million rows.
pub const ROWS: u32 = 1_000_000;
/// The horizontal extent, in content cells. §9's watermark reads `(400, 69)` against its viewport.
pub const COLS: u32 = 400;

// ── the ledger ───────────────────────────────────────────────────────────────────────────────────
//
// **Every number this screen is gated or reported on has one home and it is here** — the runtime's
// ledger rule (`crates/vitui-runtime/src/ledger.rs`), inherited by `crate::form` and
// `crate::state`. Every figure is a count over a deterministic screen, so none carries a machine.

/// Rect the two bars cover: a column of `H - 1` and a row of `W - 1`.
///
/// The corner belongs to neither and is the scroll area's, which is why this is not `H + W`.
pub const BAR_CELLS: u64 = (H - 1) as u64 + (W - 1) as u64;
/// **Rect a track-first pair of bars writes twice, every frame. This screen's own number.**
///
/// It is `thumb_v + thumb_h` and nothing else: the excess *is* the thumb, because the thumb is
/// exactly what the groove was written under. Spec §3 remembers **221** on C02's two-bar screen and
/// **345** on §9's; see [`crate::scroll`]'s test module and
/// `examples/press_numbers.rs` for which halves of those reproduce here and which are another
/// screen's magnitudes.
pub const TRACK_FIRST_EXCESS: u64 = 224;
/// The vertical thumb: one cell, because 79 cells of viewport over a million of extent is a
/// thumb the clamp saves from being zero.
pub const THUMB_V: u16 = 1;
/// The horizontal thumb: `299 * 299 / 400`.
pub const THUMB_H: u16 = 223;

/// The vertical bar's rectangle on the two-bar screen. Reserved, never overlaid (ADR 0029).
pub fn vertical() -> Rect {
    Rect::new(i32::from(W - 1), 0, 1, H - 1)
}

/// The horizontal bar's rectangle.
pub fn horizontal() -> Rect {
    Rect::new(0, i32::from(H - 1), W - 1, 1)
}

/// What the two bars are told about the content.
pub fn spans() -> [(Rect, Span, BarOpts); 2] {
    [
        (
            vertical(),
            Span {
                viewport: u32::from(H - 1),
                extent: ROWS,
                offset: ROWS / 2,
            },
            BarOpts::default(),
        ),
        (
            horizontal(),
            Span {
                viewport: u32::from(W - 1),
                extent: COLS,
                offset: 40,
            },
            BarOpts {
                orient: Orient::Horizontal,
                ..BarOpts::default()
            },
        ),
    ]
}

/// **Both bars of the two-bar screen, drawn correctly.** The arm that ships.
pub fn two_bars<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>) {
    for (area, span, opts) in spans() {
        bar_into(ink, cx, area, span, &opts);
    }
}

/// **Both bars, drawn track-first.** [`TRACK_FIRST_EXCESS`] cells written twice, every frame.
pub fn two_bars_track_first<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>) {
    for (area, span, opts) in spans() {
        defective::track_first(ink, cx, area, span, &opts);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counters::Tally;
    use crate::runner::driver_at;
    use vitui_runtime::Density;
    use vitui_runtime::layout::rect;

    fn tallied(w: u16, h: u16, f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>)) -> Tally {
        let mut driver = driver_at(w, h, Density::Compact);
        let mut tally = Tally::new();
        driver.frame(|cx| f(&mut tally, cx));
        tally
    }

    /// **A bar writes a partition of its rectangle, at every length, offset and orientation** —
    /// spec §2's first equality on the helper it is stated about, and its second restricted to the
    /// bar.
    ///
    /// Swept rather than sampled, because every interesting case is a place the arithmetic runs
    /// out: a track shorter than its thumb, an offset past the end, content that fits, and a bar of
    /// one cell.
    #[test]
    fn a_bar_writes_each_cell_once_at_every_length_and_offset() {
        for orient in [Orient::Vertical, Orient::Horizontal] {
            for length in 0u16..24 {
                for (viewport, extent) in [(0u32, 0u32), (10, 10), (10, 40), (1, 1_000_000)] {
                    for offset in [0u32, 3, 1_000_000] {
                        let opts = BarOpts {
                            orient,
                            ..BarOpts::default()
                        };
                        let area = match orient {
                            Orient::Vertical => Rect::new(0, 0, 1, length),
                            Orient::Horizontal => Rect::new(0, 0, length, 1),
                        };
                        let span = Span {
                            viewport,
                            extent,
                            offset,
                        };
                        let mut thumb_cells = Rect::default();
                        let tally = tallied(length.max(1), length.max(1), |tally, cx| {
                            thumb_cells = bar_into(tally, cx, area, span, &opts);
                        });
                        let what = format!("{orient:?} {length} {viewport}/{extent}@{offset}");
                        assert_eq!(
                            tally.writes(),
                            tally.distinct(),
                            "{what}: a cell was written twice"
                        );
                        assert_eq!(
                            tally.reported(),
                            tally.writes(),
                            "{what}: a write the engine did not report"
                        );
                        assert_eq!(
                            tally.distinct(),
                            (u64::from(area.w) * u64::from(area.h)),
                            "{what}: the bar does not cover its rectangle"
                        );
                        assert_eq!(
                            rect::intersect(thumb_cells, area),
                            thumb_cells,
                            "{what}: the thumb is not inside the bar"
                        );
                    }
                }
            }
        }
    }

    /// **Criterion 6: `writes == distinct` on a two-bar screen — 0, against 221 track-first.**
    ///
    /// Both arms draw the same cells with the same glyphs in the same places, so the only counter
    /// that separates them is the pair. The track-first arm is *cheaper* in verbs, which is the
    /// shape ADR 0026 records four other times: **the defective build is always faster and always
    /// marks less.**
    ///
    /// The excess is asserted to equal the two thumbs, which is what makes it a property of the
    /// mechanism rather than a number read off this screen: the cells written twice **are** the
    /// thumb, because the thumb is exactly what the groove was written under.
    #[test]
    fn a_track_written_under_its_own_thumb_is_two_hundred_and_twenty_four_double_writes() {
        let correct = tallied(W, H, two_bars);
        let defect = tallied(W, H, two_bars_track_first);

        assert_eq!(
            correct.writes() - correct.distinct(),
            0,
            "the correct bars write no cell twice"
        );
        assert_eq!(
            correct.distinct(),
            BAR_CELLS,
            "the two bars, minus the corner"
        );
        assert_eq!(
            defect.writes() - defect.distinct(),
            TRACK_FIRST_EXCESS,
            "the track-first differential moved"
        );
        assert_eq!(
            defect.writes() - defect.distinct(),
            u64::from(THUMB_V) + u64::from(THUMB_H),
            "the excess is the two thumbs, which is what makes it the mechanism's number"
        );
        assert_eq!(
            defect.distinct(),
            correct.distinct(),
            "the two screens touch the same cells, which is why only the pair separates them"
        );
        // **Not even the verb count separates them**, which is stronger than the thing this test
        // was written expecting. Track-first saves a verb on each bar's two track stripes and
        // spends one on the groove it drew whole, and on this screen the two cancel exactly: 82
        // against 82. The vertical bar is one verb a row either way — a column is not a run — so
        // the saving is the horizontal bar's alone, and the extra groove verb is the vertical
        // bar's.
        assert_eq!(
            defect.verbs(),
            correct.verbs(),
            "the two arms have stopped agreeing on verbs. That is not a failure of the gate — it \
             is the note above going stale, and it is worth reading before the number is changed"
        );
        assert!(
            defect.writes() > correct.writes(),
            "the only counter that moves at all is `writes`, and by itself it says the defective \
             screen drew *more* rather than that it drew the same cells twice. The pair is what \
             tells those two apart"
        );
    }

    /// **The two thumbs are what the ledger says they are**, asserted from the arithmetic rather
    /// than from the drawing, so a change to either shows up as a change to both.
    #[test]
    fn the_two_thumbs_are_one_cell_and_two_hundred_and_twenty_three() {
        let [(v, vs, _), (h, hs, _)] = spans();
        assert_eq!(thumb(v.h, vs).1, THUMB_V);
        assert_eq!(thumb(h.w, hs).1, THUMB_H);
        // The clamp is doing the work on the vertical bar: 79 * 79 / 1 000 000 is zero, and a
        // million-row area with no thumb is a bar that says nothing at all.
        assert_eq!(u64::from(H - 1) * u64::from(H - 1) / u64::from(ROWS), 0);
    }

    /// **Content that fits fills the track, and an offset past the end pins the thumb** — the two
    /// ends of [`thumb`], which are where a hand-written bar is wrong in a way nobody notices.
    #[test]
    fn a_thumb_fills_a_track_it_does_not_need_and_pins_at_the_far_end() {
        assert_eq!(
            thumb(
                20,
                Span {
                    viewport: 20,
                    extent: 20,
                    offset: 0
                }
            ),
            (0, 20)
        );
        assert_eq!(
            thumb(
                20,
                Span {
                    viewport: 30,
                    extent: 20,
                    offset: 0
                }
            ),
            (0, 20)
        );
        let span = Span {
            viewport: 10,
            extent: 40,
            offset: 999,
        };
        let (start, len) = thumb(20, span);
        assert_eq!(len, 5);
        assert_eq!(
            start + len,
            20,
            "the last content cell puts the thumb against the far end exactly"
        );
        assert_eq!(thumb(0, span), (0, 0), "a bar with no room writes nothing");
    }
}
