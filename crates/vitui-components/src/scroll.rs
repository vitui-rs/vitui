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

use vitui_runtime::{Ctx, Glyph, Id, Interest, Rect, Response, Role, Scrollable};

use crate::ink::{Direct, Ink};

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
    use super::{BarOpts, Ctx, Ink, Rect, Response, Shares, Span, draw};

    /// **A band drawn by arithmetic instead of into a view.**
    ///
    /// One `Ctx::child` apart from [`super::sticky`] and identical in every other respect: the same
    /// cells at the same coordinates through the same verbs, and the same origin — `child` narrows
    /// the clip and `scrolled` moves the origin, and this keeps the second and drops the first. So
    /// **a band whose body writes past its own edge overruns onto whatever is beside it**, that
    /// neighbour draws afterwards and wins, the picture is identical, and the overrun is re-damaged
    /// on every steady frame for ever.
    ///
    /// It is spec §6's pinned-column finding on the other axis, which is what spec §9 asks to be
    /// shown: *a body drawn by arithmetic instead of into a view has identical writes, identical
    /// verbs, identical output and cells re-damaged every steady frame.*
    pub fn arithmetic_band(
        cx: &mut Ctx<'_, '_>,
        band: Rect,
        shares: Shares,
        offset: (i32, i32),
        body: impl FnOnce(&mut Ctx<'_, '_>),
    ) -> Response {
        let id = cx.id();
        if band.is_empty() {
            return Response::inert(id, band);
        }
        let (dx, dy) = shares.of(offset);
        let mut view = cx.scrolled(band.x - dx, band.y - dy);
        body(&mut view);
        Response::inert(id, band)
    }

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

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// The decision: **bars are reserved**, and the answer is a fixpoint
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **Which bars stand.** `v` takes a **column**, `h` takes a **row**.
///
/// The axis a bar *reports* and the axis it *costs* are perpendicular, which is the whole reason
/// the two decisions are coupled and the whole reason [`decide`] iterates at all.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Shown {
    /// The vertical bar, down the right-hand edge. It costs a column.
    pub v: bool,
    /// The horizontal bar, along the bottom edge. It costs a row.
    pub h: bool,
}

/// One decision, with the evidence a gate needs beside it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Decision {
    /// What stands.
    pub shown: Shown,
    /// **How many passes the loop took, including the one that changed nothing.** Spec §9's gate is
    /// *worst case 3 passes*, and the third is the pass that proves it is a fixpoint.
    pub passes: u8,
}

/// The largest number of passes [`decide`] may take. Spec §9: **worst case 3 passes.**
pub const MAX_PASSES: u8 = 3;

/// **How the *presence* of a bar is decided.**
///
/// # `WhenItFits` requires a declared content size, and that is a rule about the caller
///
/// ADR 0029's second half: *reserved auto-hiding bars require a declared content size, because the
/// measurement is taken inside the rectangle the decision produced.* This crate keeps that
/// unwritable rather than documented — [`scroll_area`] takes its extent as an **argument**, so
/// there is no closure to hand it, no `measure` hook and nothing that could see the reduced
/// rectangle before answering. A body that wants its own size measured has
/// [`Ctx::measured`](vitui_runtime::Ctx::measured) and owes the caller the number *before* the
/// frame reaches the area.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Hide {
    /// **The default.** Both gutters are cut whether or not there is anything to scroll: no
    /// decision, no loop, and a screen whose furniture does not move when its content changes.
    #[default]
    Never,
    /// Cut a gutter only for an axis that overflows the rectangle the other gutter left behind.
    WhenItFits,
}

impl Hide {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Hide::Never => "never",
            Hide::WhenItFits => "when it fits",
        }
    }
}

/// **The viewport a decision leaves behind**: a bar only ever *removes* room.
pub fn reserved(free: (u16, u16), shown: Shown) -> (u16, u16) {
    (
        free.0.saturating_sub(u16::from(shown.v)),
        free.1.saturating_sub(u16::from(shown.h)),
    )
}

/// **Which bars stand over `extent` in a rectangle of `free` cells — the fixpoint.**
///
/// The termination argument is one sentence, and it is the whole answer to *can this oscillate*:
/// **reserving is monotone.** A bar only removes room and a smaller viewport can only make an
/// overflow more likely, so the iteration moves two booleans from `false` to `true` and never back
/// — two monotone booleans, therefore one least fixpoint, at most two changes plus the pass that
/// proves it. [`MAX_PASSES`].
///
/// The `||` below is the mechanism and not a convenience: written `=` the loop is no longer
/// monotone by construction and the paragraph above becomes a claim about the reasoning rather
/// than about the code.
///
/// The precondition that makes the monotonicity real is a property of the **body**, not of this
/// function: *a smaller viewport may not produce a smaller extent.* A body that breaks it — a list
/// that stacks two fields per row when it is narrow — flips the decision every frame with no
/// input, which is why [`Hide::WhenItFits`] is not the default and why the extent is declared.
///
/// ```
/// use vitui_components::scroll::{Hide, Shown, decide};
///
/// // Ninety-nine rows in eighty: the vertical bar stands, which costs a column — and the content
/// // is exactly eighty wide, so losing that column raises the horizontal bar as well.
/// let d = decide((80, 24), (80, 99), Hide::WhenItFits);
/// assert_eq!(d.shown, Shown { v: true, h: true });
/// assert_eq!(d.passes, 3);
/// ```
pub fn decide(free: (u16, u16), extent: (u32, u32), hide: Hide) -> Decision {
    if let Hide::Never = hide {
        return Decision {
            shown: Shown { v: true, h: true },
            passes: 0,
        };
    }
    let mut shown = Shown::default();
    let mut passes = 0u8;
    loop {
        let (w, h) = reserved(free, shown);
        let next = Shown {
            v: shown.v || extent.1 > u32::from(h),
            h: shown.h || extent.0 > u32::from(w),
        };
        passes = passes.saturating_add(1);
        if next == shown {
            return Decision { shown, passes };
        }
        shown = next;
        // Monotone, so this is unreachable at three. It is here so that an edit which breaks the
        // monotonicity returns a `passes` the gate can fail on rather than hanging.
        if passes >= 8 {
            return Decision { shown, passes };
        }
    }
}

/// **The largest offset a content of `extent` cells admits in a `view` viewport**, per axis.
///
/// An offset and its bound are the same quantity in the same unit — **content cells** — and mixing
/// the two is the off-by-a-viewport that makes the last screenful unreachable.
pub fn max_offset(extent: (u32, u32), view: (u16, u16)) -> (i32, i32) {
    (
        i32::try_from(extent.0.saturating_sub(u32::from(view.0))).unwrap_or(i32::MAX),
        i32::try_from(extent.1.saturating_sub(u32::from(view.1))).unwrap_or(i32::MAX),
    )
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// `sticky` — a band is one construction with an axis argument
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **Which of the body's two offsets a band shares.** Spec §9's axis argument.
///
/// > A band is a rectangle split that shares one of the two offsets and pins the other to zero, and
/// > it must be a view. The header shares `x`; the pinned column shares `y`; the footer shares `x`;
/// > the gutter shares neither.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Shares {
    /// The horizontal offset, with the vertical pinned to zero — a **header** or a **footer**.
    X,
    /// The vertical offset, with the horizontal pinned to zero — a **pinned column**.
    Y,
    /// Neither — a **gutter**, which is where a header and a pinned column meet.
    Neither,
}

impl Shares {
    /// The translation this band applies to `offset`. The pinned axis is a literal zero.
    pub const fn of(self, offset: (i32, i32)) -> (i32, i32) {
        match self {
            Shares::X => (offset.0, 0),
            Shares::Y => (0, offset.1),
            Shares::Neither => (0, 0),
        }
    }

    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Shares::X => "x",
            Shares::Y => "y",
            Shares::Neither => "neither",
        }
    }
}

/// **Which of spec §9's four bands this is.**
///
/// Four names and **one** construction: they differ in the rectangle they were cut from and in
/// [`Shares`], and in nothing else. Four components here would be four places for the wheel rule
/// below to be got wrong.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Which {
    /// Cut off the top. Shares `x`.
    Header,
    /// Cut off the bottom. Shares `x`.
    Footer,
    /// Cut off the left, between the header and the footer. Shares `y`.
    Pinned,
    /// Where a horizontal band and the pinned column meet. Shares neither.
    Gutter,
}

impl Which {
    /// Which offset this band shares. **The table is spec §9's own sentence.**
    pub const fn shares(self) -> Shares {
        match self {
            Which::Header | Which::Footer => Shares::X,
            Which::Pinned => Shares::Y,
            Which::Gutter => Shares::Neither,
        }
    }

    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Which::Header => "header",
            Which::Footer => "footer",
            Which::Pinned => "pinned",
            Which::Gutter => "gutter",
        }
    }
}

/// One standing band, as it is handed to a band drawer.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Band {
    /// Which of the four it is.
    pub which: Which,
    /// Where it is, in the coordinates of the [`Ctx`] the area was drawn on.
    pub rect: Rect,
}

impl Band {
    /// Which offset it shares. [`Which::shares`].
    pub const fn shares(self) -> Shares {
        self.which.shares()
    }
}

/// **Draw a band: a rectangle split that shares one of the two offsets, pins the other to zero, and
/// is a view.**
///
/// This is the one construction spec §9 states, with [`Shares`] as its axis argument. `scroll_area`
/// calls it for all four of §9's bands and nothing else in this crate opens a band of its own.
///
/// # The coordinates the body draws in
///
/// **Content coordinates on the shared axis, and zero-based inside the band on the pinned one.** A
/// header at `Shares::X` over a body at horizontal offset 40 draws its first visible column at
/// `x == 40` and its only row at `y == 0`; a pinned column at `Shares::Y` over a body at vertical
/// offset 900 draws content row 900 at `y == 900` and its first column at `x == 0`. That is what
/// makes a title and the cells under it unable to disagree about where a column is: they are the
/// same number.
///
/// # It declares nothing, and that is spec §9's rule rather than an omission
///
/// > One construction with an axis argument, not four components — and **one hit entry for all
/// > four**, because a band that were a second scroll area would win the wheel from the body it is
/// > a header of.
///
/// So the [`Response`] is inert and its `rect` is the band. Whatever the body declares inside is
/// the caller's, and a frame with four bands standing has exactly as many hit entries as the same
/// frame with none.
///
/// ```
/// use vitui_components::scroll::{Shares, sticky};
/// use vitui_runtime::{Rect, ctx::Driver};
///
/// let mut driver = Driver::headless(20, 6).expect("a sink attaches");
/// driver.frame(|cx| {
///     let band = Rect::new(0, 0, 20, 1);
///     let paint = cx.theme().paint(vitui_runtime::Role::Title);
///     // The body is at horizontal offset 4, so the header's first visible column is content
///     // column 4 — and it is written at `x == 4`.
///     let resp = sticky(cx, band, Shares::X, (4, 900), |cx| {
///         assert_eq!(cx.text(4, 0, "abc", paint).cells, 3);
///         // The vertical offset is pinned to zero: the band's own row is row zero.
///         assert_eq!(cx.text(4, 900, "abc", paint).cells, 0);
///     });
///     assert_eq!(resp.rect, band);
/// });
/// ```
#[track_caller]
pub fn sticky(
    cx: &mut Ctx<'_, '_>,
    band: Rect,
    shares: Shares,
    offset: (i32, i32),
    body: impl FnOnce(&mut Ctx<'_, '_>),
) -> Response {
    let id = cx.id();
    if band.is_empty() {
        return Response::inert(id, band);
    }
    let (dx, dy) = shares.of(offset);
    // **The clip is the whole point.** `Ctx::child` narrows and translates; `Ctx::scrolled` puts
    // the shared axis back into content coordinates and leaves the pinned one at the band's own
    // origin. Drawn by arithmetic into the caller's context instead, a band's overrun lands on
    // whatever is beside it, is overdrawn by that neighbour, and re-damages those cells on every
    // steady frame for ever — `defective::arithmetic_band`, and it is spec §6's pinned-column
    // finding on the other axis.
    let mut clipped = cx.child(band);
    let mut view = clipped.scrolled(-dx, -dy);
    body(&mut view);
    Response::inert(id, band)
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// `scrollbar` — the component, where [`bar`] is the helper
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// [`scrollbar`]'s options.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ScrollbarOpts {
    /// How the groove and the thumb are drawn. [`bar`]'s own options.
    pub bar: BarOpts,
    /// **A stepper at each end**, drawn in the four arrow glyphs the freeze declares for this
    /// component (spec §17). Dropped when the bar is shorter than [`CAPS`] + 1 cells, because a
    /// bar with no track left is two buttons and a lie.
    pub caps: bool,
    /// The role the steppers are drawn in.
    pub cap: Role,
}

/// How many cells a pair of steppers costs. One at each end.
pub const CAPS: u16 = 2;

impl Default for ScrollbarOpts {
    fn default() -> ScrollbarOpts {
        ScrollbarOpts {
            bar: BarOpts::default(),
            caps: true,
            cap: Role::Border,
        }
    }
}

/// **The scrollbar: two steppers and the track between them, over one [`Span`].**
///
/// [`bar`] is the *helper* this draws through — spec §3's *the bar every scrollable draws* — and
/// this is the **component**: it declares a hit entry, so the thumb is a drag target and the
/// steppers are click targets, and it returns a [`Response`] like everything else on this map.
///
/// **It does not own an offset** (spec §17's freeze says so in its own column, and §9 says why):
/// a bar that moved an offset of its own would be a second scroll area, and a second scroll area
/// inside one wins the wheel from the body it is a bar of. What it publishes is `Interest::CLICK`
/// and `Interest::HOVER` and **never** `Interest::SCROLL`, so a wheel click over the bar chains
/// outward to the area.
///
/// ```
/// use vitui_components::scroll::{Span, scrollbar};
/// use vitui_runtime::{Rect, ctx::Driver};
///
/// let mut driver = Driver::headless(40, 10).expect("a sink attaches");
/// driver.frame(|cx| {
///     let span = Span { viewport: 10, extent: 40, offset: 0 };
///     let resp = scrollbar(cx, Rect::new(39, 0, 1, 10), span);
///     assert_eq!(resp.rect, Rect::new(39, 0, 1, 10));
/// });
/// ```
#[track_caller]
pub fn scrollbar(cx: &mut Ctx<'_, '_>, area: Rect, span: Span) -> Response {
    scrollbar_with(cx, area, span, &ScrollbarOpts::default())
}

/// [`scrollbar`], with the options spelled out.
#[track_caller]
pub fn scrollbar_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    span: Span,
    opts: &ScrollbarOpts,
) -> Response {
    let id = cx.id();
    scrollbar_into(&mut Direct, cx, id, area, span, opts)
}

/// **[`scrollbar`], drawing through an [`Ink`] and under an id its caller minted.**
///
/// The entry point a gate takes, and the one [`scroll_area`] takes: an area draws two of these and
/// they cannot both take `Location::caller()`, so the id is a parameter here and only here. See
/// [`crate::ink`] for why the seam exists rather than a second implementation.
pub fn scrollbar_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    id: Id,
    area: Rect,
    span: Span,
    opts: &ScrollbarOpts,
) -> Response {
    if area.is_empty() {
        return Response::inert(id, area);
    }
    // **Never `Interest::SCROLL`.** See the note on [`scrollbar`].
    let resp = cx.interact(id, area, Interest::CLICK.with(Interest::HOVER));

    let length = match opts.bar.orient {
        Orient::Vertical => area.h,
        Orient::Horizontal => area.w,
    };
    let caps = opts.caps && length > CAPS;
    // **The steppers, and they are the four arrow glyphs spec §17's freeze declares for this
    // component.** `tree` had to file an issue for declaring three it could not draw; a bar's two
    // ends are where the other four belong, and drawing them here rather than in [`bar`] leaves
    // ticket 07's helper — and every number measured on it — untouched.
    let track = if caps {
        let theme = cx.theme();
        let paint = theme.paint(opts.cap);
        let (lo, hi) = match opts.bar.orient {
            Orient::Vertical => (Glyph::ArrowUp, Glyph::ArrowDown),
            Orient::Horizontal => (Glyph::ArrowLeft, Glyph::ArrowRight),
        };
        let (lo, hi) = (theme.glyph(lo), theme.glyph(hi));
        match opts.bar.orient {
            Orient::Vertical => {
                ink.run(cx, area.x, area.y, lo, area.w, paint);
                ink.run(cx, area.x, area.bottom() - 1, hi, area.w, paint);
                Rect::new(area.x, area.y + 1, area.w, length - CAPS)
            }
            // **Every row of the cap column, and not just the first.** A `scrollbar` is public and
            // takes an arbitrary rectangle: a horizontal bar two rows tall would otherwise leave
            // rows `1..h` of its two end columns unwritten, which is spec §2's partition rule
            // broken in the one place `scroll_area` never looks — `parts` only ever hands it a bar
            // one row tall.
            Orient::Horizontal => {
                for row in 0..i32::from(area.h) {
                    ink.run(cx, area.x, area.y + row, lo, 1, paint);
                    ink.run(cx, area.right() - 1, area.y + row, hi, 1, paint);
                }
                Rect::new(area.x + 1, area.y, length - CAPS, area.h)
            }
        }
    } else {
        area
    };
    let _ = bar_into(ink, cx, track, span, &opts.bar);
    resp
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// `scroll_area` — the rectangle is reduced before the body is called
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **Everything a [`scroll_area`] stores: an offset in content cells, per axis.**
///
/// One pair of `i32`s and nothing else. Spec §9: *the extent, the offset, the thumb and
/// scroll-into-view are all in content cells and all come from one place* — this is that place for
/// the second of the four, and [`Span`] is it for the other three.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct AreaState {
    /// The first visible content cell, per axis. **Content cells, never rows.**
    pub offset: (i32, i32),
}

/// [`scroll_area`]'s options.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AreaOpts {
    /// How the presence of a bar is decided. [`Hide::Never`] by default.
    pub hide: Hide,
    /// How the two bars are drawn.
    pub bar: ScrollbarOpts,
    /// The role the **tail** is written in — the cells of the rectangle the content does not reach.
    pub tail: Role,
    /// How many rows of sticky header to cut off the top.
    pub header: u16,
    /// How many rows of sticky footer to cut off the bottom.
    pub footer: u16,
    /// How many columns of pinned column to cut off the left.
    pub pinned: u16,
    /// What the area is interested in besides the wheel. `Interest::SCROLL` is folded in whatever
    /// this says.
    pub interest: Interest,
}

impl Default for AreaOpts {
    fn default() -> AreaOpts {
        AreaOpts {
            hide: Hide::Never,
            bar: ScrollbarOpts::default(),
            tail: Role::Body,
            header: 0,
            footer: 0,
            pinned: 0,
            interest: Interest::CLICK.with(Interest::HOVER),
        }
    }
}

/// **Where a scroll area's parts go, as arithmetic.**
///
/// A value rather than five returns, and public rather than private, because a caller that needs
/// to know how wide the body came out — to size the content it is about to declare — must be able
/// to ask *the same function the component asks*. Two spellings of this split is the defect this
/// exists to make unwritable.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Parts {
    /// Which bars stand.
    pub shown: Shown,
    /// How many passes [`decide`] took. [`MAX_PASSES`].
    pub passes: u8,
    /// **The body's viewport**, both bars and all four bands removed. This is the rectangle the
    /// scroll scope is opened on.
    pub view: Rect,
    /// The vertical bar, or empty.
    pub vbar: Rect,
    /// The horizontal bar, or empty.
    pub hbar: Rect,
    /// Where the two bars meet, or empty.
    pub corner: Rect,
    /// The sticky header, or empty.
    pub header: Rect,
    /// The sticky footer, or empty.
    pub footer: Rect,
    /// The pinned column, or empty.
    pub pinned: Rect,
    /// Where the header and the pinned column meet, or empty.
    pub gutter_top: Rect,
    /// Where the footer and the pinned column meet, or empty.
    pub gutter_bottom: Rect,
}

impl Parts {
    /// **The bands that stand, in draw order.** Empty rectangles are not bands.
    pub fn bands(&self) -> impl Iterator<Item = Band> + '_ {
        [
            (Which::Gutter, self.gutter_top),
            (Which::Header, self.header),
            (Which::Pinned, self.pinned),
            (Which::Gutter, self.gutter_bottom),
            (Which::Footer, self.footer),
        ]
        .into_iter()
        .filter(|(_, r)| !r.is_empty())
        .map(|(which, rect)| Band { which, rect })
    }

    /// The largest offset this viewport admits over `extent`, per axis.
    pub fn max_offset(&self, extent: (u32, u32)) -> (i32, i32) {
        max_offset(extent, (self.view.w, self.view.h))
    }
}

/// **The split, and the whole of ADR 0029 as arithmetic: the rectangle is reduced before the body
/// is called.**
///
/// The bands are cut first because their sizes are the caller's and do not depend on the decision;
/// the bars are decided over what is left, which is the only coupled pair and is why [`decide`]
/// iterates.
pub fn parts(rect: Rect, extent: (u32, u32), opts: &AreaOpts) -> Parts {
    let bands_w = opts.pinned.min(rect.w);
    let bands_h = opts.header.saturating_add(opts.footer).min(rect.h);
    let free = (rect.w - bands_w, rect.h - bands_h);
    let d = decide(free, extent, opts.hide);
    let v = u16::from(d.shown.v).min(free.0);
    let h = u16::from(d.shown.h).min(free.1);

    let inner = Rect::new(rect.x, rect.y, rect.w - v, rect.h - h);
    let head_h = opts.header.min(inner.h);
    let foot_h = opts.footer.min(inner.h - head_h);
    let pin_w = opts.pinned.min(inner.w);
    let mid_y = inner.y + i32::from(head_h);
    let mid_h = inner.h - head_h - foot_h;
    let foot_y = inner.bottom() - i32::from(foot_h);
    let body_x = inner.x + i32::from(pin_w);
    let body_w = inner.w - pin_w;

    Parts {
        shown: d.shown,
        passes: d.passes,
        view: Rect::new(body_x, mid_y, body_w, mid_h),
        vbar: Rect::new(rect.right() - i32::from(v), rect.y, v, rect.h - h),
        hbar: Rect::new(rect.x, rect.bottom() - i32::from(h), rect.w - v, h),
        corner: Rect::new(
            rect.right() - i32::from(v),
            rect.bottom() - i32::from(h),
            v,
            h,
        ),
        header: Rect::new(body_x, inner.y, body_w, head_h),
        footer: Rect::new(body_x, foot_y, body_w, foot_h),
        pinned: Rect::new(inner.x, mid_y, pin_w, mid_h),
        gutter_top: Rect::new(inner.x, inner.y, pin_w, head_h),
        gutter_bottom: Rect::new(inner.x, foot_y, pin_w, foot_h),
    }
}

/// **Why a reserved auto-hiding bar takes a declared content size and there is no way to measure
/// one.**
///
/// ADR 0029's second half, kept unwritable rather than written down: *a measured extent and a
/// hideable reserved bar are incompatible, because the measurement is taken inside the rectangle
/// the decision produced.* [`scroll_area`] takes its extent as an **argument** — a pair of numbers
/// the caller already owns before the frame reaches the area — so there is no closure to ask, no
/// `measure` hook, and nothing that could see the reduced rectangle before answering.
///
/// The alternative is not hypothetical and it is not far-fetched: an area that asked its body for a
/// size would take that size **after** [`decide`] had already removed a column and a row for the
/// bars, so the answer would be a size measured in the viewport the answer produced. Over content
/// that fits with no bars at all, the reader's own spelling — decide from *last frame's* reduced
/// rectangle — keeps both bars for ever, losing a row and a column permanently on a screen that
/// looks correct (`crate::area::fitting`, 19 of 20 either way), and a body whose extent is not
/// antitone in its viewport flips the decision 99 times in 99 frames with no input.
///
/// # The twin, naming the protected items by path
///
/// A lone `compile_fail` also passes when the item it names has been *renamed*: `E0425` for *the
/// thing you must not have does not exist* and `E0425` for *the thing you meant moved* are the
/// same diagnostic. So the shape that ships is pinned first.
///
/// ```
/// let _ = vitui_components::scroll::Hide::WhenItFits;
/// let _ = vitui_components::scroll::Hide::Never;
/// let _ = vitui_components::scroll::scroll_area;
/// let _ = vitui_components::scroll::parts;
/// ```
///
/// # The hostile half
///
/// **Protects:** [`Hide::WhenItFits`] and [`scroll_area`], written
/// `vitui_components::scroll::Hide::WhenItFits` and `vitui_components::scroll::scroll_area` at the
/// paths the twin uses, by way of the item that must not stand beside them.
///
/// ```compile_fail,E0425
/// fn main() {
///     let _ = vitui_components::scroll::scroll_area_measured;
/// }
/// ```
#[cfg(doc)]
pub struct WhyAnAutoHidingBarNeedsADeclaredExtent;

/// **The scroll area: bars are reserved, and the rectangle is reduced before the body is called.**
///
/// # Bars are reserved. There is no overlay option
///
/// ADR 0029, and it is unconditional. An overlay bar is free exactly where the body does not draw
/// under it — **0 cells** — and costs cells re-damaged on every steady frame where it does, against
/// **0** for the reserved twin on the same screen with the same content. ADR 0029 prices that at
/// **3 535 cells a frame**; `crate::area` measures the double write underneath it at
/// [`crate::area::OVERLAY_REDAMAGE`] and says at length why the two are different quantities. What
/// makes the rule unconditional rather than a default is the *exactly*: **free where nothing is
/// drawn under it is not a property any component can guarantee of its body.**
///
/// So there is no `Bars` argument here, no `overlay: bool` on [`AreaOpts`], and
/// [`crate::area::Bars::Overlay`] exists only as the negative case a scene is played on.
///
/// # A `scroll_area` costs its content
///
/// Spec §9's C21: a `scroll_area` costs its **content** and a virtualised `collection` costs its
/// **visible window**, and the wrong pairing is the single most expensive mistake available above
/// this runtime. **Every shipped scrollable of unbounded data is a [`crate::collect::collection`]**
/// — this is for a form, a preview, a picture, a document whose size the caller already knows. A
/// body that means to virtualise reads [`Ctx::visible_rows`](vitui_runtime::Ctx::visible_rows) and
/// iterates that and nothing else.
///
/// # The unit is content cells, everywhere
///
/// `extent` is `Σ h` and never a row count. Spec §9 prices the substitution at **row 799 999 of
/// 999 999** over a million rows one in eight of which is three cells tall — a fifth of the content
/// unreachable, with *time, writes, verbs, marked cells, regions and allocations all identical*.
/// See [`crate::area::Unit`].
///
/// ```
/// use vitui_components::scroll::{AreaState, scroll_area};
/// use vitui_runtime::{Rect, Role, ctx::Driver};
///
/// let mut driver = Driver::headless(20, 6).expect("a sink attaches");
/// let mut st = AreaState::default();
/// st.offset = (0, 4);
/// driver.frame(|cx| {
///     scroll_area(cx, Rect::new(0, 0, 20, 6), &mut st, (20, 100), &mut |cx| {
///         let paint = cx.theme().paint(Role::Body);
///         // Content coordinates: the offset put content row 4 on the viewport's top row.
///         assert_eq!(cx.visible_rows().start, 4);
///         assert_eq!(cx.text(0, 4, "hello", paint).cells, 5);
///     });
/// });
/// ```
#[track_caller]
pub fn scroll_area(
    cx: &mut Ctx<'_, '_>,
    rect: Rect,
    st: &mut AreaState,
    extent: (u32, u32),
    body: &mut dyn FnMut(&mut Ctx<'_, '_>),
) -> Response {
    scroll_area_with(
        cx,
        rect,
        st,
        &AreaOpts::default(),
        extent,
        &mut |_, _| {},
        body,
    )
}

/// [`scroll_area`], with the options and the band drawer spelled out.
///
/// `band` is called once per standing band, **inside its view**, with the coordinates
/// [`sticky`] documents. It is called for all four of spec §9's bands and the caller tells them
/// apart by [`Band::which`].
#[track_caller]
pub fn scroll_area_with(
    cx: &mut Ctx<'_, '_>,
    rect: Rect,
    st: &mut AreaState,
    opts: &AreaOpts,
    extent: (u32, u32),
    band: &mut dyn FnMut(&mut Ctx<'_, '_>, Band),
    body: &mut dyn FnMut(&mut Ctx<'_, '_>),
) -> Response {
    let id = cx.id();
    scroll_area_into(
        &mut Direct,
        cx,
        id,
        rect,
        st,
        opts,
        extent,
        |_ink, cx, b| band(cx, b),
        |_ink, cx| body(cx),
    )
}

/// **[`scroll_area`], drawing through an [`Ink`] and under an id its caller minted.**
///
/// The entry point a gate takes. See [`crate::ink`] for why the seam exists rather than a second
/// implementation of the component written against a `Tally`.
#[expect(
    clippy::too_many_arguments,
    reason = "the component's own six — a context, an id, a rectangle, the state, the options and \
              the declared extent — plus the two drawers and the `Ink` seam's writer. Folding the \
              first six into a parameter struct would invent a type that exists only to satisfy a \
              lint, and folding the two drawers into one would hand a band drawer to a body that \
              has no band"
)]
pub fn scroll_area_into<I, B, D>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    id: Id,
    rect: Rect,
    st: &mut AreaState,
    opts: &AreaOpts,
    extent: (u32, u32),
    mut band: B,
    body: D,
) -> Response
where
    I: Ink,
    B: FnMut(&mut I, &mut Ctx<'_, '_>, Band),
    D: FnOnce(&mut I, &mut Ctx<'_, '_>),
{
    // **The rectangle is reduced here, before anything is drawn and before the body is called.**
    let p = parts(rect, extent, opts);
    let max = p.max_offset(extent);

    // **The reveal the frame before asked for**, applied by the widget that owns the offset. A
    // delta and not a position, because the offset may have moved in between — and it is in the
    // same content cells the extent is, which is the fourth of spec §9's four sites.
    if let Some((dx, dy)) = cx.take_into_view(id) {
        st.offset = (st.offset.0 + dx, st.offset.1 + dy);
    }
    st.offset = (st.offset.0.clamp(0, max.0), st.offset.1.clamp(0, max.1));

    // **One hit entry, on the whole rectangle**, so a wheel click over a bar or a band still
    // reaches the body. A component that publishes a scrollable region owes the pair per axis,
    // computed from its clamped offset.
    let mut resp = cx.scrollable(
        id,
        rect,
        opts.interest.with(Interest::SCROLL),
        Scrollable::between(st.offset, max),
    );
    // **This frame's wheel, read inside the draw by the widget that owns the offset.**
    st.offset = (
        (st.offset.0 + resp.scrolled.0).clamp(0, max.0),
        (st.offset.1 + resp.scrolled.1).clamp(0, max.1),
    );

    // **The bands, and every one of them is a view.** One construction, four configurations —
    // and not one of them declares anything, so the four share the area's single hit entry.
    for b in p.bands() {
        sticky(cx, b.rect, b.shares(), st.offset, |cx| {
            band(&mut *ink, cx, b)
        });
    }

    // **The body, at content coordinates.** `Ctx::with_id` is outside the scope rather than around
    // it, for `crate::collect`'s reason: inside, it re-childs the view at `self.area()`, which is
    // the *content's* rectangle, and at any offset past the first screenful the two do not overlap.
    let tail = cx.theme().paint(opts.tail);
    cx.with_id(id, |cx| {
        cx.scroll_scope(id, p.view, st.offset, max, |cx| {
            body(&mut *ink, cx);
            // **The tail is the area's own, and every cell of it is written** (spec §2). The range
            // `[extent, offset + viewport)` is inside the rectangle, and the component that owns
            // the rectangle must write it — spec §9 assigns that line by name. It is not free and
            // the offset clamp is: `max` is recomputed every frame, and a shrunk extent leaves
            // cells no body will ever draw.
            tail_into(&mut *ink, cx, p.view, extent, st.offset, tail);
        });
    });

    // **The bars, and their span is the body's viewport against the declared extent.** The third of
    // spec §9's four sites, and it reads the same two numbers the first two did.
    if !p.vbar.is_empty() {
        let mut o = opts.bar;
        o.bar.orient = Orient::Vertical;
        scrollbar_into(
            &mut *ink,
            cx,
            Id::keyed(id, VBAR),
            p.vbar,
            Span {
                viewport: u32::from(p.view.h),
                extent: extent.1,
                offset: u32::try_from(st.offset.1).unwrap_or(0),
            },
            &o,
        );
    }
    if !p.hbar.is_empty() {
        let mut o = opts.bar;
        o.bar.orient = Orient::Horizontal;
        scrollbar_into(
            &mut *ink,
            cx,
            Id::keyed(id, HBAR),
            p.hbar,
            Span {
                viewport: u32::from(p.view.w),
                extent: extent.0,
                offset: u32::try_from(st.offset.0).unwrap_or(0),
            },
            &o,
        );
    }
    if !p.corner.is_empty() {
        let theme = cx.theme();
        let paint = theme.paint(opts.bar.bar.track);
        let glyph = theme.glyph(Glyph::Track);
        ink.run(cx, p.corner.x, p.corner.y, glyph, p.corner.w, paint);
    }

    resp.rect = rect;
    resp
}

/// **The key the vertical bar's id is minted under**, so an area's two bars are two widgets.
///
/// # Why it is a sentinel and not `0`
///
/// The bars are keyed under the **area's own id**, and so is anything a caller keys inside it: a
/// body that writes `Id::keyed(id, row)` for its rows — which is what `table` does and what
/// `crate::area`'s screens do — would collide with `Id::keyed(id, 0)` on its first row. Two widgets
/// with one id is a **merge**, which the runtime counts and nothing else reports: the hit index
/// takes both entries and the focus, the grab and the scroll association go to whichever the table
/// kept. So the two keys are values no row index reaches, and
/// `tests::four_bands_standing_declare_no_hit_entry_of_their_own` asserts `merges == 0` over a
/// frame that keys a row at 0 and at 1 beside them.
const VBAR: u64 = u64::MAX;
/// The key the horizontal bar's id is minted under. See [`VBAR`].
const HBAR: u64 = u64::MAX - 1;

/// **`[extent, offset + viewport)` on both axes, written by the owner of the rectangle.**
///
/// Spec §2's partition rule at the one place a body cannot reach: the body draws what the content
/// admits and this is the rest of the rectangle. Leaving it out is what a scroll area whose extent
/// has just shrunk looks like — the old rows still on the screen, under a correct offset.
fn tail_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    view: Rect,
    extent: (u32, u32),
    offset: (i32, i32),
    paint: vitui_runtime::Paint,
) {
    let x0 = offset.0;
    let x1 = offset.0 + i32::from(view.w);
    let y1 = offset.1 + i32::from(view.h);
    let ex = i32::try_from(extent.0).unwrap_or(i32::MAX);
    let ey = i32::try_from(extent.1).unwrap_or(i32::MAX);
    for y in offset.1..y1 {
        let from = if y >= ey { x0 } else { ex.max(x0) };
        if from >= x1 {
            continue;
        }
        let n = u16::try_from(x1 - from).unwrap_or(u16::MAX);
        ink.run(cx, from, y, " ", n, paint);
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

    // ── components ticket 19: the three components ───────────────────────────────────────────────

    /// A screen big enough to hold a scroll area with all four bands standing.
    const AW: u16 = 60;
    /// Its height.
    const AH: u16 = 20;

    /// The options every one of these tests starts from: four bands and both bars.
    fn banded() -> AreaOpts {
        AreaOpts {
            hide: Hide::Never,
            header: 1,
            footer: 1,
            pinned: 4,
            ..AreaOpts::default()
        }
    }

    /// **Criterion 1: bars are reserved, so the parts tile the rectangle exactly.**
    ///
    /// A partition and not a picture: every cell of the rectangle belongs to exactly one part, at
    /// every size a rectangle comes in. An overlay bar cannot satisfy this at all — the cells under
    /// it belong to two — which is ADR 0029 as arithmetic rather than as a measurement.
    #[test]
    fn the_parts_of_a_reserved_area_tile_its_rectangle_exactly() {
        for w in 1u16..24 {
            for h in 1u16..24 {
                for opts in [AreaOpts::default(), banded()] {
                    for extent in [(0u32, 0u32), (5, 5), (400, 1_000_000)] {
                        let rect = Rect::new(3, 2, w, h);
                        let p = parts(rect, extent, &opts);
                        let mut seen = vec![0u8; usize::from(w) * usize::from(h)];
                        let mut mark = |r: Rect| {
                            for y in r.y..r.bottom() {
                                for x in r.x..r.right() {
                                    let (dx, dy) = (x - rect.x, y - rect.y);
                                    assert!(
                                        dx >= 0
                                            && dy >= 0
                                            && dx < i32::from(w)
                                            && dy < i32::from(h),
                                        "a part left the rectangle at {w}x{h}"
                                    );
                                    seen[dy as usize * usize::from(w) + dx as usize] += 1;
                                }
                            }
                        };
                        mark(p.view);
                        mark(p.vbar);
                        mark(p.hbar);
                        mark(p.corner);
                        for band in p.bands() {
                            mark(band.rect);
                        }
                        assert!(
                            seen.iter().all(|n| *n == 1),
                            "the parts do not tile {w}x{h} at extent {extent:?}: {:?}",
                            seen.iter().filter(|n| **n != 1).count()
                        );
                        assert!(p.passes <= MAX_PASSES);
                    }
                }
            }
        }
    }

    /// **A `scrollbar` writes a partition of its rectangle, at every orientation, size and offset.**
    ///
    /// Spec §2's first equality on the component rather than on [`bar`], and it is swept because
    /// the interesting cases are all places the arithmetic runs out: a bar shorter than its two
    /// steppers, a bar of one cell, and — the one this test was written for — **a horizontal bar
    /// more than one row tall**, which `scroll_area` never produces and a caller can ask for at
    /// any time, since `scrollbar` is public and takes an arbitrary rectangle.
    #[test]
    fn a_scrollbar_writes_each_cell_of_its_rectangle_once() {
        for orient in [Orient::Vertical, Orient::Horizontal] {
            for caps in [false, true] {
                for w in 1u16..6 {
                    for h in 1u16..6 {
                        for offset in [0u32, 1, 40, 999] {
                            let opts = ScrollbarOpts {
                                bar: BarOpts {
                                    orient,
                                    ..BarOpts::default()
                                },
                                caps,
                                ..ScrollbarOpts::default()
                            };
                            let mut driver = driver_at(8, 8, Density::default());
                            let mut tally = Tally::new();
                            let area = Rect::new(1, 1, w, h);
                            driver.frame(|cx| {
                                let id = cx.id();
                                scrollbar_into(
                                    &mut tally,
                                    cx,
                                    id,
                                    area,
                                    Span {
                                        viewport: 4,
                                        extent: 1_000,
                                        offset,
                                    },
                                    &opts,
                                );
                            });
                            let cells = u64::from(w) * u64::from(h);
                            assert_eq!(
                                (tally.writes(), tally.distinct()),
                                (cells, cells),
                                "{}x{w}x{h} caps={caps} offset={offset} is not a partition",
                                match orient {
                                    Orient::Vertical => "v",
                                    Orient::Horizontal => "h",
                                }
                            );
                        }
                    }
                }
            }
        }
    }

    /// **Criterion 4, first half: one construction, and the four bands are its four configurations.**
    ///
    /// The table is spec §9's own sentence — *the header shares `x`; the pinned column shares `y`;
    /// the footer shares `x`; the gutter shares neither* — asserted rather than paraphrased, and
    /// the source scan beside it is what keeps a second construction from being written next to it.
    #[test]
    fn a_band_is_one_construction_with_an_axis_argument() {
        assert_eq!(Which::Header.shares(), Shares::X);
        assert_eq!(Which::Footer.shares(), Shares::X);
        assert_eq!(Which::Pinned.shares(), Shares::Y);
        assert_eq!(Which::Gutter.shares(), Shares::Neither);
        assert_eq!(
            Shares::X.of((7, 9)),
            (7, 0),
            "the pinned axis is a literal zero"
        );
        assert_eq!(Shares::Y.of((7, 9)), (0, 9));
        assert_eq!(Shares::Neither.of((7, 9)), (0, 0));

        // **All four stand at once**, so the four configurations are exercised and not merely
        // declared: a `Which` nothing constructs would satisfy the table above and draw nothing.
        let p = parts(Rect::new(0, 0, AW, AH), (400, 1_000), &banded());
        let mut which: Vec<Which> = p.bands().map(|b| b.which).collect();
        which.sort_by_key(|w| w.word());
        which.dedup();
        assert_eq!(
            which,
            vec![Which::Footer, Which::Gutter, Which::Header, Which::Pinned],
            "all four of §9's bands stand on one area"
        );

        // **And there is exactly one of them in this crate.** A band is `child` and then `scrolled`
        // over one rectangle; the scan counts the second half, which is the half that cannot be
        // written by accident. `collect.rs`'s three column bands are the one other place and they
        // are deliberately *not* this construction — their coordinates stay the caller's, because a
        // translated band makes `Tally::distinct` meaningless across the row (components 15).
        let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/scroll.rs"))
            .expect("this module");
        let opens = source
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            // The scan's own line contains the needle, which is `crate::frame`'s first run
            // reporting itself. Dropped by name rather than by counting one extra.
            .filter(|l| l.contains(".scrolled(") && !l.contains("l.contains"))
            .count();
        assert_eq!(
            opens, 2,
            "one band construction and the negative case beside it; a third would be a second \
             spelling of §9's one"
        );
    }

    /// **Criterion 4, second half: the four bands share one hit entry, because they declare none.**
    ///
    /// Spec §9: *a band that were a second scroll area would win the wheel from the body it is a
    /// header of.* The gate is that the hit count does not depend on how many bands stand — the
    /// same frame with four bands and with none declares the same regions.
    #[test]
    fn four_bands_standing_declare_no_hit_entry_of_their_own() {
        let count = |opts: &AreaOpts| {
            let mut driver = driver_at(AW, AH, Density::default());
            let mut st = AreaState::default();
            let id = vitui_runtime::Id::named("area");
            driver.frame(|cx| {
                let paint = cx.theme().paint(Role::Body);
                scroll_area_into(
                    &mut Direct,
                    cx,
                    id,
                    Rect::new(0, 0, AW, AH),
                    &mut st,
                    opts,
                    (400, 1_000),
                    |_ink, cx, _band| {
                        let _ = cx.text(0, 0, "b", paint);
                    },
                    |_ink, cx| {
                        // **Two rows keyed under the area's own id, at 0 and at 1**, which is what
                        // a body that keys per row writes and is exactly where the bars' keys
                        // would have collided. See `VBAR`.
                        for row in 0..2u64 {
                            let rid = vitui_runtime::Id::keyed(id, row);
                            let _ =
                                cx.interact(rid, Rect::new(0, row as i32, 4, 1), Interest::CLICK);
                            let _ = cx.text(0, row as i32, ".", paint);
                        }
                    },
                );
            });
            let frame = driver.inspect();
            assert_eq!(
                frame.ids().merges(),
                0,
                "two widgets share an id: the bars' keys reach a row index"
            );
            frame.hits().len()
        };
        let bare = count(&AreaOpts {
            hide: Hide::Never,
            ..AreaOpts::default()
        });
        let four = count(&banded());
        assert_eq!(
            bare, four,
            "a band declared a hit entry, and a band that declares one is a second scroll area"
        );
        assert_eq!(
            bare, 5,
            "the area, its two bars and the body's two rows, and nothing else: the bands are drawn \
             and not declared"
        );
    }

    /// **Criterion 5: every band is a view, and the arithmetic spelling is stood up beside it.**
    ///
    /// The two arms are one `Ctx::child` apart and write the same arguments at the same call sites.
    /// The band's body deliberately overruns its own rectangle by [`OVERRUN`] rows; the view clips
    /// them and the arithmetic spelling writes them onto the body below, which draws afterwards and
    /// wins — so the **picture is identical** and the cost is invisible in everything but the
    /// damage.
    #[test]
    fn a_band_is_a_view_and_the_arithmetic_spelling_re_damages_what_is_under_it() {
        /// How many rows past its own edge the band's body writes.
        const OVERRUN: u16 = 2;
        const BW: u16 = 40;
        const BH: u16 = 10;

        let play = |arithmetic: bool| {
            let mut driver = driver_at(BW, BH, Density::default());
            let mut canvas = crate::runner::Canvas::new(BW, BH);
            let mut damage = 0u64;
            let mut shape = (0u64, 0u64);
            for _ in 0..3 {
                let mut pen = crate::runner::Pen::over(canvas);
                driver.frame(|cx| {
                    let theme = cx.theme();
                    let head = theme.paint(Role::Title);
                    let body = theme.paint(Role::Body);
                    let band = Rect::new(0, 0, BW, 1);
                    let draw = |cx: &mut Ctx<'_, '_>, pen: &mut crate::runner::Pen| {
                        for dy in 0..i32::from(1 + OVERRUN) {
                            pen.text(cx, 0, dy, &"=".repeat(usize::from(BW)), head);
                        }
                    };
                    if arithmetic {
                        defective::arithmetic_band(cx, band, Shares::X, (0, 0), |cx| {
                            draw(cx, &mut pen);
                        });
                    } else {
                        sticky(cx, band, Shares::X, (0, 0), |cx| {
                            draw(cx, &mut pen);
                        });
                    }
                    // The body under it, which draws afterwards and wins.
                    for y in 1..i32::from(BH) {
                        pen.text(cx, 0, y, &".".repeat(usize::from(BW)), body);
                    }
                });
                pen.end_frame();
                shape = (pen.tally().writes(), pen.tally().verbs());
                canvas = pen.into_canvas();
                damage = canvas.repaints();
                let _ = canvas.take_repaints();
            }
            (damage, shape)
        };

        let (view_damage, view_shape) = play(false);
        let (arith_damage, arith_shape) = play(true);
        assert_eq!(
            view_shape.1, arith_shape.1,
            "identical verbs, which is what makes the defect invisible"
        );
        assert_eq!(
            view_damage, 0,
            "a band that is a view re-damages nothing: its overrun never reached the surface"
        );
        assert_eq!(
            arith_damage,
            u64::from(OVERRUN) * u64::from(BW),
            "the arithmetic band's overrun is re-damaged on every steady frame, for ever"
        );
        assert!(
            arith_shape.0 > view_shape.0,
            "the clip is what makes the writes differ, and it is the only thing that does"
        );
    }

    /// **Criterion 11: the tail is written by the owner of the rectangle.**
    ///
    /// `[extent, offset + viewport)` on both axes. The body draws what the content admits; the rest
    /// of the rectangle is the component's, and a shrunk extent that left it unwritten is the old
    /// rows still on the screen under a correct offset.
    #[test]
    fn a_shrunk_extent_leaves_no_unwritten_tail() {
        const TW: u16 = 20;
        const TH: u16 = 8;
        for extent in [(0u32, 0u32), (3, 2), (TW as u32, TH as u32 - 1), (40, 40)] {
            let mut driver = driver_at(TW, TH, Density::default());
            let mut st = AreaState::default();
            let mut pen = crate::runner::Pen::new(TW, TH);
            let opts = AreaOpts {
                hide: Hide::WhenItFits,
                ..AreaOpts::default()
            };
            let p = parts(Rect::new(0, 0, TW, TH), extent, &opts);
            driver.frame(|cx| {
                let id = cx.id();
                let paint = cx.theme().paint(Role::Body);
                scroll_area_into(
                    &mut pen,
                    cx,
                    id,
                    Rect::new(0, 0, TW, TH),
                    &mut st,
                    &opts,
                    extent,
                    |_ink, _cx, _b| {},
                    |ink, cx| {
                        // Only what the content admits, which is the body's whole contract.
                        for y in 0..i32::try_from(extent.1).unwrap_or(0) {
                            let n = u16::try_from(extent.0).unwrap_or(u16::MAX);
                            ink.run(cx, 0, y, "#", n, paint);
                        }
                    },
                );
            });
            for y in p.view.y..p.view.bottom() {
                for x in p.view.x..p.view.right() {
                    assert!(
                        pen.canvas().get(x as u16, y as u16).is_some(),
                        "cell ({x}, {y}) of the viewport is unwritten at extent {extent:?}: the \
                         tail belongs to the component that owns the rectangle"
                    );
                }
            }
        }
    }

    /// **Criterion 6: the extent, the offset, the thumb and the reveal are one unit.**
    ///
    /// Content where one row in eight is three cells tall: the row count and `Σ h` differ by a
    /// quarter, and all four sites answer in cells. The fourth site is the reveal, which arrives as
    /// a **delta in content cells** and is applied to the offset without a conversion — a reveal in
    /// rows would land a quarter short at the bottom of the content.
    #[test]
    fn the_extent_the_offset_the_thumb_and_the_reveal_are_all_content_cells() {
        const ROWS: u32 = 800;
        const CELLS: u32 = ROWS + ROWS / 8 * 2;
        const VIEW: u16 = 20;

        // 1 and 2: the extent bounds the offset, in cells.
        assert_eq!(
            max_offset((0, CELLS), (0, VIEW)),
            (0, i32::try_from(CELLS - u32::from(VIEW)).expect("fits"))
        );
        // 3: the thumb, from the same two numbers plus the viewport.
        let at_end = thumb(
            VIEW,
            Span {
                viewport: u32::from(VIEW),
                extent: CELLS,
                offset: CELLS - u32::from(VIEW),
            },
        );
        assert_eq!(
            at_end.0 + at_end.1,
            VIEW,
            "the last content cell puts the thumb against the far end exactly"
        );
        // **Half way down, which is where the drift is visible.** At either end both spellings pin
        // the thumb, which is why *up to 7 of 69* is a figure from the middle of the range and not
        // from its end — see `crate::area::thumb_drift`.
        let middle = CELLS / 2;
        let in_cells = thumb(
            VIEW,
            Span {
                viewport: u32::from(VIEW),
                extent: CELLS,
                offset: middle,
            },
        );
        let in_rows = thumb(
            VIEW,
            Span {
                viewport: u32::from(VIEW),
                extent: ROWS,
                offset: middle,
            },
        );
        assert_ne!(
            in_rows, in_cells,
            "a thumb sized from the row count is a different thumb, smoothly and plausibly"
        );

        // 4: the reveal, as a delta in the same unit, applied by the component.
        let mut driver = driver_at(VIEW, VIEW, Density::default());
        let mut st = AreaState::default();
        let target = i32::try_from(CELLS - 1).expect("fits");
        for frame in 0..2 {
            driver.frame(|cx| {
                let paint = cx.theme().paint(Role::Body);
                scroll_area(
                    cx,
                    Rect::new(0, 0, VIEW, VIEW),
                    &mut st,
                    (u32::from(VIEW), CELLS),
                    &mut |cx| {
                        if frame == 0 {
                            cx.request_into_view(Rect::new(0, target, 1, 1));
                        }
                        let _ = cx.text(0, cx.visible_rows().start, ".", paint);
                    },
                );
            });
        }
        let view = parts(
            Rect::new(0, 0, VIEW, VIEW),
            (u32::from(VIEW), CELLS),
            &AreaOpts::default(),
        )
        .view;
        assert_eq!(
            st.offset.1,
            i32::try_from(CELLS - u32::from(view.h)).expect("fits"),
            "the reveal reached the last content cell, which a reveal measured in rows cannot"
        );
    }

    /// **Criterion 9: an offset past the end clamps on the next frame, and the clamp is free.**
    ///
    /// `max` is recomputed every frame from the extent the caller declared, so an offset the
    /// content no longer admits costs nothing to repair — which is what makes a shape change
    /// expensive and the clamp not.
    #[test]
    fn an_offset_past_the_end_clamps_on_the_next_frame() {
        const VW: u16 = 30;
        const VH: u16 = 10;
        let mut driver = driver_at(VW, VH, Density::default());
        let mut st = AreaState {
            offset: (0, 999_931),
        };
        driver.frame(|cx| {
            let paint = cx.theme().paint(Role::Body);
            scroll_area(
                cx,
                Rect::new(0, 0, VW, VH),
                &mut st,
                (u32::from(VW), 100_000),
                &mut |cx| {
                    let _ = cx.text(0, cx.visible_rows().start, ".", paint);
                },
            );
        });
        // The viewport is a row short of the rectangle, because the horizontal bar is reserved:
        // `Hide::Never` is the default and `scroll_area` cuts both gutters whatever the content is.
        let view = parts(
            Rect::new(0, 0, VW, VH),
            (u32::from(VW), 100_000),
            &AreaOpts::default(),
        )
        .view;
        assert_eq!(view.h, VH - 1);
        assert_eq!(
            st.offset.1,
            100_000 - i32::from(view.h),
            "the clamp is the extent minus the viewport and it is taken every frame"
        );
    }

    /// **Criterion 7: the frame does not grow with the content.**
    ///
    /// A body that virtualises writes its viewport and nothing else, so a thousand rows and a
    /// million are the same frame — identical writes and identical verbs. This is the counter form
    /// of the budget's own invariant: *frame cost is proportional to visible cells, never to data
    /// volume.*
    #[test]
    fn a_thousand_rows_and_a_million_are_the_same_frame() {
        let shape = |rows: u32| {
            let mut driver = driver_at(AW, AH, Density::default());
            let mut st = AreaState::default();
            let mut tally = Tally::new();
            // One warm-up frame: the frame structures take their allocation on the first frame
            // that needs one and keep it.
            for _ in 0..2 {
                tally = Tally::new();
                driver.frame(|cx| {
                    let id = cx.id();
                    let paint = cx.theme().paint(Role::Body);
                    scroll_area_into(
                        &mut tally,
                        cx,
                        id,
                        Rect::new(0, 0, AW, AH),
                        &mut st,
                        &banded(),
                        (400, rows),
                        |ink, cx, band| {
                            ink.run(cx, 0, 0, "=", band.rect.w, paint);
                        },
                        |ink, cx| {
                            let window = cx.visible_rows();
                            for y in window {
                                ink.run(cx, 0, y, ".", AW, paint);
                            }
                        },
                    );
                });
            }
            (tally.writes(), tally.verbs())
        };
        assert_eq!(
            shape(1_000),
            shape(1_000_000),
            "a thousand rows and a million are the same frame, which is what virtualising means"
        );
    }
}
