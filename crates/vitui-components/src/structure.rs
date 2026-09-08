//! Panels, rules, splits and status bars — the furniture everything else sits inside.
//!
//! [`panel`] draws a border with a title and hands back the rectangle **inside** it, so a caller
//! never has to work out where the frame ended. [`rule`] is a captioned horizontal line,
//! [`status_bar`] is a row of segments, and [`crate::frame::block`] is the plain filled region a component draws
//! into when it wants no chrome at all.
//!
//! # Examples
//!
//! ```
//! use vitui_components::structure::panel;
//! use vitui_components::text::text;
//! use vitui_runtime::Rect;
//! use vitui_runtime::ctx::Driver;
//!
//! let mut driver = Driver::headless(40, 8).expect("a sink attaches");
//! driver.frame(|cx| {
//!     let p = panel(cx, cx.area(), " logs ");
//!
//!     // The interior is smaller than the panel, and the panel has not written a cell of it.
//!     assert!(p.interior.w < 40 && p.interior.h < 8);
//!     text(cx, Rect::new(p.interior.x, p.interior.y, p.interior.w, 1), "nothing to report");
//! });
//! ```
//!
//! # What a container returns, and why it is not a `Response`
//!
//! [`Panel`] carries the response *and* the interior rectangle. A container owes its caller both
//! halves: what happened to it, and which cells it left alone. A bare `Response` can only say the
//! first, and a caller who has to guess the second draws over the border.
//!
//! # A title is a string, and a border is a role
//!
//! Options are a `Default` struct: [`PanelOpts`] carries the roles for the border, the title and
//! the padding, whether there is a border at all, and what the panel is interested in. Nothing
//! here takes a colour — a component names a [`vitui_runtime::theme::Role`] and the theme
//! resolves it for the terminal in hand.

use vitui_runtime::layout::rect;
use vitui_runtime::layout::text::width;
use vitui_runtime::{Ctx, Glyph, Interest, Response, Role};

use crate::frame::{BlockOpts, block_into};
use crate::ink::{Direct, Ink};
use crate::scroll::{Orient, Shares};
use crate::text::Justify;
use vitui_runtime::Rect;

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["panel", "rule", "status_bar"];

/// [`panel`]'s options.
///
/// A `Default` struct, never a required builder.
///
/// **The title is not here.** It is the panel's data and rule 2 puts data in the argument list — and
/// [`crate::frame::BlockOpts::title`] carrying it as well would be two homes for one string, which
/// is the shape the 15-cell instance is made of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PanelOpts {
    /// The role the frame is drawn in. **This is where focus lands** — a focused panel is drawn in
    /// [`Role::Focus`] *before* its cells are written, never restyled after. See
    /// [`crate::frame::WhyThereIsNoFocusRing`].
    pub border: Role,
    /// The role the title is drawn in.
    pub title_role: Role,
    /// The role the padding ring is drawn in.
    pub pad: Role,
    /// Whether to draw a frame at all. A panel without one still has a padding ring.
    pub bordered: bool,
    /// Whether to apply the theme's density as a padding ring. Density is theme data and it changes
    /// rectangles.
    pub padded: bool,
    /// What the panel declares.
    ///
    /// [`Interest::HOVER`] and **not** `FOCUS`: a panel is not a tab stop, which is what makes
    /// [`crate::dense`]'s screen 338 regions and 333 stops rather than 338 of each.
    pub interest: Interest,
}

impl Default for PanelOpts {
    fn default() -> PanelOpts {
        PanelOpts {
            border: Role::Border,
            title_role: Role::Title,
            pad: Role::Body,
            bordered: true,
            padded: true,
            interest: Interest::HOVER,
        }
    }
}

/// **What [`panel`] answers: what happened to it, and what it did not write.**
///
/// Two fields because rule 4 and *named in its return value* are two obligations and a
/// container owes both. See this module's header for why the closure form that would collapse them
/// into one is refused.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Panel {
    /// What happened to the panel's own region. Rule 4.
    pub response: Response,
    /// **The interior it handed over and did not write.** The second half.
    pub interior: Rect,
}

/// **A frame, a title in its top run, a padding ring, and the interior handed back unwritten.**
///
/// **Hostile axes:** none.
///
/// Its title's truncation is [`crate::text::text`]'s construction and not a second one. Duplicating
/// the flag would buy O5 a second scene for one defect, which is the opposite of what a per-axis
/// scene count is for.
///
/// ```
/// use vitui_runtime::Rect;
/// use vitui_components::structure::panel;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(40, 10).expect("a sink attaches");
/// driver.frame(|cx| {
///     let p = panel(cx, cx.area(), " panel systems ");
///     // The interior is smaller than the panel, and `panel` has not touched a cell of it.
///     assert!(p.interior.w < 40 && p.interior.h < 10);
///     assert!(!p.response.focused);
/// });
/// ```
#[track_caller]
pub fn panel(cx: &mut Ctx<'_, '_>, area: Rect, title: &str) -> Panel {
    panel_with(cx, area, title, &PanelOpts::default())
}

/// [`panel`], with the options spelled out.
#[track_caller]
pub fn panel_with(cx: &mut Ctx<'_, '_>, area: Rect, title: &str, opts: &PanelOpts) -> Panel {
    panel_into(&mut Direct, cx, area, title, opts)
}

/// **[`panel`], drawing through an [`Ink`] so a counter can see the verbs.**
///
/// # The region is declared **before** the body is drawn, and that is not the order a stand-in used
///
/// The hit index is in draw order and the press award is a **reverse** scan of it, so the innermost
/// widget wins. A container declared *after* the widgets it contains is therefore in front of them:
/// the reverse scan reaches the panel first and every click inside it lands on the panel. Ticket
/// 09's stand-in screen declared the panel last — invisibly, because nothing on it was clicked — and
/// this is where that order is fixed, in the component, once.
#[track_caller]
pub fn panel_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    title: &str,
    opts: &PanelOpts,
) -> Panel {
    let id = cx.id();
    let response = cx.interact(id, area, opts.interest);
    let interior = block_into(ink, cx, area, &block_opts(title, opts));
    Panel { response, interior }
}

/// The [`BlockOpts`] a [`PanelOpts`] and a title make. One place, so the two structs cannot drift
/// into disagreeing about what a panel's frame is.
fn block_opts<'a>(title: &'a str, opts: &PanelOpts) -> BlockOpts<'a> {
    BlockOpts {
        title,
        border: opts.border,
        title_role: opts.title_role,
        pad: opts.pad,
        bordered: opts.bordered,
        padded: opts.padded,
    }
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// `rule` — the Tier 2 divider: `fit`'s remainder over one row or one column, plus a `Glyph`
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// [`rule`]'s options.
///
/// A `Default` struct, never a required builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RuleOpts {
    /// Which way it runs. Horizontal by default, because a divider between stacked things is what a
    /// rule is nine times in ten.
    ///
    /// **[`Orient`]'s own default is [`Orient::Vertical`]** and its reason is a scrollbar's — *most
    /// content is longer than it is wide* — which is not a divider's, so this default is spelled out
    /// rather than inherited. [`crate::input::SliderOpts`] made the same choice for the same reason.
    pub orient: Orient,
    /// The role the line is drawn in.
    pub role: Role,
    /// The role the caption and the padding rows are drawn in.
    ///
    /// **Separate from [`RuleOpts::role`] on purpose, and the direction is the opposite of
    /// [`crate::text::FitOpts`]'s.** A rule is not interactive, declares no region and takes no
    /// hover award, so *a label in a second paint is repainted every frame* does not
    /// reach it — there is nothing to repaint it. What a caption in [`Role::Dim`] over a line in
    /// [`Role::Border`] buys is the one thing asked for: the caption and the line are told apart
    /// on the **paint** axis as well as on the content axis.
    pub caption: Role,
    /// Where the caption sits along the line.
    pub justify: Justify,
}

impl Default for RuleOpts {
    fn default() -> RuleOpts {
        RuleOpts {
            orient: Orient::Horizontal,
            role: Role::Border,
            caption: Role::Dim,
            justify: Justify::Start,
        }
    }
}

/// **One row or one column of a [`Glyph`], centred in the rectangle it was handed.**
///
/// **Hostile axes:** none.
///
/// One row or one column of a [`Glyph`], with the caption elided through [`crate::text::fit`] —
/// `text`'s own flag, not a second one here.
///
/// The line is [`Glyph::HLine`] or [`Glyph::VLine`] — the `rule` family, two entries — and every
/// other cell of the rectangle is padding, because the rule is that *a component handed a rectangle
/// writes all of that*.
///
/// ```
/// use vitui_components::structure::rule;
/// use vitui_runtime::Rect;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(20, 3).expect("a sink attaches");
/// driver.frame(|cx| {
///     let resp = rule(cx, Rect::new(0, 0, 20, 3), "");
///     // Rule 4: a `Response` back even from a pure drawer — and it declares no region at all.
///     assert!(!resp.hovered);
/// });
/// // An L0 drawer: nothing to click, nothing to focus.
/// assert_eq!(driver.inspect().hits().len(), 0);
/// ```
#[track_caller]
pub fn rule(cx: &mut Ctx<'_, '_>, area: Rect, caption: &str) -> Response {
    rule_with(cx, area, caption, &RuleOpts::default())
}

/// [`rule`], with the options spelled out.
#[track_caller]
pub fn rule_with(cx: &mut Ctx<'_, '_>, area: Rect, caption: &str, opts: &RuleOpts) -> Response {
    rule_into(&mut Direct, cx, area, caption, opts)
}

/// **[`rule`], drawing through an [`Ink`] so a counter can see the verbs.**
///
/// # `fit`'s remainder, and it is the remainder that is the component
///
/// [`crate::text::fit_into`] writes a row as **four skippable parts** — the lead padding, the head of
/// the text, the one-cell ellipsis and the trail padding — and its own note says why every one of
/// them is skipped when it is empty: *a label that exactly fills its row is one verb and not three*.
/// A rule is that shape with the two padding runs spelled in a glyph instead of a space, so it is
/// written here in the same order and with the same skips, and the caption goes through
/// [`crate::glyphs::elide`] so that the one-cell ellipsis rule holds on a rule as it does on a
/// label.
///
/// A vertical rule cannot reach `fit` at all — `fit` writes rows — so the **caption is the horizontal
/// arm's alone**, and `tests::a_vertical_rule_is_one_column_and_has_no_caption` is where that is
/// asserted rather than left to a reader.
#[track_caller]
pub fn rule_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    caption: &str,
    opts: &RuleOpts,
) -> Response {
    let id = cx.id();
    if area.is_empty() {
        return Response::inert(id, area);
    }
    let pad = cx.theme().paint(opts.caption);
    match opts.orient {
        Orient::Horizontal => {
            // Centred, with the rows either side as padding — `crate::text::face_and_label`'s split,
            // so a rule in a three-row band sits where a chip's label would.
            let (above, rest) = rect::split_at_v(area, area.h.saturating_sub(1) / 2);
            crate::text::pad_rows(ink, cx, above, pad);
            let (band, below) = rect::split_at_v(rest, 1);
            line_into(ink, cx, band, caption, opts);
            crate::text::pad_rows(ink, cx, below, pad);
        }
        Orient::Vertical => {
            let (left, rest) = rect::split_at_h(area, area.w.saturating_sub(1) / 2);
            pad_columns(ink, cx, left, pad);
            let (band, right) = rect::split_at_h(rest, 1);
            line_into(ink, cx, band, "", opts);
            pad_columns(ink, cx, right, pad);
        }
    }
    Response::inert(id, area)
}

/// **The line itself: the lead run, the caption, the trail run.** `fit`'s four parts with a glyph
/// where the padding was.
fn line_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    band: Rect,
    caption: &str,
    opts: &RuleOpts,
) {
    let theme = cx.theme();
    let glyph = theme.glyph(match opts.orient {
        Orient::Horizontal => Glyph::HLine,
        Orient::Vertical => Glyph::VLine,
    });
    let line = theme.paint(opts.role);
    let text = theme.paint(opts.caption);

    // A vertical rule is one column of glyph and nothing else — see [`rule_into`].
    if opts.orient == Orient::Vertical {
        crate::scroll::stripe(ink, cx, band, Orient::Vertical, 0, band.h, glyph, line);
        return;
    }

    let w = band.w;
    // **The spaces around a caption are the caller's**, exactly as [`panel`]'s title's are — its own
    // doctest writes `" panel systems "`. Two reasons and both are counts: a component that spaced
    // its caption for the caller would need a `String` to put the result in, and this one allocates
    // nothing on any path; and the four parts below would stop being `fit`'s four, because a caption
    // that did not fit *with* its spaces would have to decide whether to drop them.
    let (head, marker) = crate::glyphs::elide(theme, caption, w);
    let head_w = width(head);
    let mark_w = width(marker);
    let used = head_w + mark_w;
    debug_assert!(
        used <= w,
        "`elide` returned {used} cells for a {w}-cell row: the one-cell ellipsis rule has moved"
    );
    // **Saturating, which `crate::glyphs::elided_row_into` is for the same quantity.** It is
    // unreachable today — `truncate` is width-bounded and both `Glyph::Ellipsis` spellings are one
    // column — and the day a marker is two columns wide this is a component whose whole contract is
    // a partition wrapping its trailing run.
    let slack = w.saturating_sub(used);
    let lead = match opts.justify {
        Justify::Start => 0,
        Justify::Middle => slack / 2,
        Justify::End => slack,
    };
    let trail = slack - lead;

    // The same four skippable parts, in the same order `fit` writes them in.
    ink.run(cx, band.x, band.y, glyph, lead, line);
    if head_w > 0 {
        ink.text(cx, band.x + i32::from(lead), band.y, head, text);
    }
    if mark_w > 0 {
        ink.text(cx, band.x + i32::from(lead + head_w), band.y, marker, text);
    }
    ink.run(
        cx,
        band.x + i32::from(lead + used),
        band.y,
        glyph,
        trail,
        line,
    );
}

/// Write every column of `cells` as padding. [`crate::text::pad_rows`] on the other axis, and the
/// verb is the same one.
fn pad_columns<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, cells: Rect, st: vitui_runtime::Paint) {
    if cells.is_empty() {
        return;
    }
    for r in 0..cells.h {
        ink.run(cx, cells.x, cells.y + i32::from(r), " ", cells.w, st);
    }
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// `status_bar` — the Tier 2 band: `sticky`'s one construction, an axis argument, one hit entry
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **How wide each segment is.**
///
/// Not a second construction — it decides *rectangles* and nothing about the repertoire, which is
/// what [`crate::Component::constructions`] counts. What it decides is whether the bar's content can
/// be **wider than its band**, and that is the one thing that makes [`StatusOpts::shares`] observable
/// at all: a bar laid out over its own visible width has nothing to scroll.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Fill {
    /// **Split the visible width evenly**, the odd cells going to the leading segments. The default,
    /// and the shape a bar across the bottom of a screen has.
    #[default]
    Even,
    /// **Each segment as wide as it measures**, separators between them. The content is then as wide
    /// as the text, which may be wider than the band — and a bar sharing `x` with the body it is a
    /// bar of scrolls with it.
    Natural,
}

/// [`status_bar`]'s options.
///
/// A `Default` struct, never a required builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct StatusOpts {
    /// **Which offset the band shares.** The axis argument, taken verbatim from
    /// [`crate::scroll::sticky`] — this component adds no fifth band and no fourth value.
    ///
    /// [`Shares::X`] by default: a status bar is a header or a footer, and those are the two of
    /// the four that share the horizontal offset.
    pub shares: Shares,
    /// How wide each segment is. See [`Fill`].
    pub fill: Fill,
    /// The role the segments are drawn in.
    pub role: Role,
    /// The role the padding is drawn in.
    ///
    /// Separate from [`StatusOpts::role`] for [`crate::text::FitOpts`]'s reason: a caller that had
    /// to pass one role would fill the band first to get the second, which is the defect the
    /// partition rule exists to refuse.
    pub pad: Role,
    /// The role the separator between two segments is drawn in.
    pub sep: Role,
    /// Where each segment sits inside its own share.
    pub justify: Justify,
    /// **What the bar declares. One entry for the bar, never one per segment.**
    ///
    /// The rule is the scroll family's, one component over: *one hit entry for all four bands, because a band
    /// that were a second scroll area would win the wheel from the body it is a header of*. A
    /// status bar is the one member of the family that declares anything at all, and what it
    /// declares is **one** region — a segment is a rectangle a caller can resolve out of
    /// [`Response::local`], not a widget.
    ///
    /// [`Interest::SCROLL`] is not in it, and the reason is: a widget that declares
    /// the wheel and consumes nothing is worse than one declaring nothing at all, because it is the
    /// topmost region over its rectangle and the area beneath never sees the notch.
    pub interest: Interest,
}

impl Default for StatusOpts {
    fn default() -> StatusOpts {
        StatusOpts {
            shares: Shares::X,
            fill: Fill::Even,
            role: Role::Dim,
            pad: Role::Body,
            sep: Role::Border,
            justify: Justify::Start,
            interest: Interest::CLICK.with(Interest::HOVER),
        }
    }
}

/// **A band of segments across the bottom of a screen — one hit entry, and a view.**
///
/// **Hostile axes:** none.
///
/// A bar's content is one row derived from its own segments, so it has nothing to be wrong about on
/// either axis: `Shares::Y` and `Shares::Neither` draw the same bar at four different offsets, **0
/// cells of 30x2 apart**.
///
/// What it *is*: **the same construction as a sticky header or a
/// footer**, a rectangle split that shares one of the two offsets and pins the other to zero. So
/// this draws through [`crate::scroll::sticky`] and mints nothing — no fifth band, no second clip,
/// no offset of its own.
///
/// # The clip is the reason, and it is priced one module over
///
/// A segment longer than its share, written by arithmetic into the caller's context instead, lands
/// on whatever is beside it, is overdrawn by that neighbour, and **re-damages those cells on every
/// steady frame for ever** — `crate::scroll::defective::arithmetic_band`. The band is a *view*, so
/// the overrun costs its own cells and nothing else's.
///
/// ```
/// use vitui_components::structure::status_bar;
/// use vitui_runtime::Rect;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(40, 6).expect("a sink attaches");
/// driver.frame(|cx| {
///     let bar = Rect::new(0, 5, 40, 1);
///     let resp = status_bar(cx, bar, &["ready", "utf-8", "ln 1"]);
///     // Rule 4: a `Response` back, and nothing has happened on a frame with no input.
///     assert!(!resp.clicked);
/// });
/// // **One hit entry for three segments**, which is the whole of this component's own criterion.
/// assert_eq!(driver.inspect().hits().len(), 1);
/// ```
#[track_caller]
pub fn status_bar(cx: &mut Ctx<'_, '_>, area: Rect, segments: &[&str]) -> Response {
    status_bar_with(cx, area, segments, (0, 0), &StatusOpts::default())
}

/// [`status_bar`], with the body's offset and the options spelled out.
///
/// `offset` is the offset of the body this is a bar of, handed to [`crate::scroll::sticky`] whole;
/// [`StatusOpts::shares`] decides which half of it survives.
#[track_caller]
pub fn status_bar_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    segments: &[&str],
    offset: (i32, i32),
    opts: &StatusOpts,
) -> Response {
    status_bar_into(&mut Direct, cx, area, segments, offset, opts)
}

/// **[`status_bar`], drawing through an [`Ink`] so a counter can see the verbs.**
///
/// The entry point a gate takes; [`status_bar`] is this with [`Direct`].
///
/// # The axis argument is the band's and not the bar's
///
/// The four bands differ in *which offset they share*, and a bar's content is one row derived from
/// its own segments — so the shared **vertical** offset has nothing to move. That is asserted rather
/// than left to a reader: `tests::the_two_offsets_a_band_can_share_are_not_two_bars` draws the same
/// bar at all three values of [`Shares`] and reports how many cells differ, and only the horizontal
/// one moves anything, and only at [`Fill::Natural`].
///
/// # Every visible cell of the band, exactly once, at every offset
///
/// The segments are laid out in the band's **content** coordinates and the trailing padding is
/// extended to the end of the visible window — so an offset past the end of the text pads rather
/// than leaving the cells nobody wrote, which is what an unbounded scroll area left behind.
#[track_caller]
pub fn status_bar_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    segments: &[&str],
    offset: (i32, i32),
    opts: &StatusOpts,
) -> Response {
    // **The id, taken outside every closure**. `#[track_caller]` all the way down, or two
    // bars in one application are one bar and the second one's `Response` is inert.
    let id = cx.id();
    if area.is_empty() {
        return Response::inert(id, area);
    }
    // **One hit entry, declared before the band opens.** Inside the view it would be declared in the
    // band's own coordinates and a caller resolving a segment out of `Response::local` would be
    // reading a rectangle from one coordinate system against a position from another.
    let resp = cx.interact(id, area, opts.interest);
    let shares = opts.shares;
    let (dx, dy) = shares.of(offset);
    let window = Rect::new(dx, dy, area.w, area.h);
    crate::scroll::sticky(cx, area, shares, offset, |cx| {
        segments_into(ink, cx, window, segments, opts);
    });
    resp
}

/// **The bar's content, in the band's own coordinates**: the segments on the first visible row, the
/// separators between them, and padding everywhere else.
fn segments_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    window: Rect,
    segments: &[&str],
    opts: &StatusOpts,
) {
    let theme = cx.theme();
    let pad = theme.paint(opts.pad);
    let sep_paint = theme.paint(opts.sep);
    let sep_glyph = theme.glyph(Glyph::VLine);

    // The row the segments sit on, and the rows below it. `fit`'s split, so a bar in a three-row
    // band writes its text where a chip's label would — at the top, because a status bar's row is
    // the row it was cut to.
    let (band, below) = rect::split_at_v(window, 1);
    crate::text::pad_rows(ink, cx, below, pad);
    if band.is_empty() {
        return;
    }

    let n = u16::try_from(segments.len()).unwrap_or(u16::MAX);
    if n == 0 {
        ink.run(cx, band.x, band.y, " ", band.w, pad);
        return;
    }
    // Separators sit **between** segments, so there are `n - 1` of them however wide the band is.
    let seps = n - 1;
    let mut x = match opts.fill {
        // Even splits the *visible* window, so the layout starts where the window does.
        Fill::Even => band.x,
        // Natural is laid out from the band's own content origin, which is what a shared offset
        // then scrolls.
        Fill::Natural => 0,
    };
    let room = band.w.saturating_sub(seps);
    for (i, seg) in segments.iter().enumerate() {
        let share = match opts.fill {
            // The odd cells go to the leading segments, which is `Row`'s own rule for a weighted
            // split and is why the shares sum to the band exactly.
            Fill::Even => {
                let i = u16::try_from(i).unwrap_or(u16::MAX);
                room / n + u16::from(i < room % n)
            }
            Fill::Natural => width(seg),
        };
        crate::text::fit_into(
            ink,
            cx,
            Rect::new(x, band.y, share, 1),
            seg,
            &crate::text::FitOpts {
                justify: opts.justify,
                role: opts.role,
                pad: opts.pad,
            },
        );
        x += i32::from(share);
        if i + 1 < segments.len() {
            ink.text(cx, x, band.y, sep_glyph, sep_paint);
            x += 1;
        }
    }
    // **The tail, out to the end of the visible window.** Under `Even` it is empty by construction;
    // under `Natural` it is what stops a bar shorter than its band — or scrolled past its own text —
    // from leaving cells nobody wrote.
    let end = window.x + i32::from(window.w);
    if x < end {
        let w = u16::try_from(end - x).unwrap_or(u16::MAX);
        ink.run(cx, x, band.y, " ", w, pad);
    }
}

/// **The panel written the way the fifth row prices, kept because a gate nobody has watched
/// fail is not a gate.**
///
/// `pub` for the reason [`crate::frame::defective`] is, and it reaches that module rather than
/// re-writing the panel: the correct arm and this one are **one function with one boolean between
/// them**, so the diff a reviewer would have to catch is the diff the register can point at.
pub mod defective {
    use super::{Ctx, Ink, Panel, PanelOpts, Rect, block_opts};

    /// **A panel whose top border is one run with its title written over it.** The 15-cell
    /// instance — the number *is* the title's width — and the only one of the five that is **one
    /// verb cheaper** than the correct build.
    #[track_caller]
    pub fn panel_over_title<I: Ink>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        title: &str,
        opts: &PanelOpts,
    ) -> Panel {
        let id = cx.id();
        let response = cx.interact(id, area, opts.interest);
        let interior =
            crate::frame::defective::block_over_title(ink, cx, area, &block_opts(title, opts));
        Panel { response, interior }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counters::Tally;
    use vitui_runtime::ctx::Driver;

    /// A tally over one frame of `f`, on a `w` by `h` sink.
    fn tallied(
        w: u16,
        h: u16,
        f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>) -> Rect,
    ) -> (Tally, Rect) {
        let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        let mut interior = Rect::new(0, 0, 0, 0);
        driver.frame(|cx| interior = f(&mut tally, cx));
        (tally, interior)
    }

    /// **Criterion 2 and criterion 4: a panel writes its frame exactly once, writes every cell of it
    /// **and** the interior it returns, and its title is part of the top run's partition.**
    ///
    /// The two equalities, restricted the way a container makes them: `writes == distinct` over the
    /// whole rectangle, and `distinct == w × h − interior` — *the cells it does not write are named
    /// in its return value*, so the second equality is over the complement of what came back.
    #[test]
    fn a_panel_writes_its_frame_exactly_once_and_hands_the_interior_over_untouched() {
        for (w, h) in [(4, 4), (10, 5), (20, 3), (40, 10), (2, 2), (1, 8)] {
            for title in [
                "",
                " panel systems ",
                "a title far wider than this panel is",
            ] {
                let opts = PanelOpts::default();
                let (tally, interior) = tallied(w, h, |tally, cx| {
                    panel_into(tally, cx, Rect::new(0, 0, w, h), title, &opts).interior
                });
                let cells = u64::from(w) * u64::from(h);
                assert_eq!(
                    tally.writes(),
                    tally.distinct(),
                    "{w}x{h} `{title}`: {} cells written twice — and the 15-cell instance is what \
                     that looks like when it is a title",
                    tally.writes() - tally.distinct()
                );
                assert_eq!(
                    tally.distinct(),
                    cells - (u64::from(interior.w) * u64::from(interior.h)),
                    "{w}x{h} `{title}`: the panel wrote something other than its own frame and ring"
                );
                // And not one of those cells is in the interior it handed back.
                for y in interior.y..interior.bottom() {
                    for x in interior.x..interior.right() {
                        assert!(
                            !tally.touched(x, y),
                            "{w}x{h} `{title}`: ({x}, {y}) was handed over and then written"
                        );
                    }
                }
            }
        }
    }

    /// **Criterion 4, watched firing: the 15-cell instance.**
    ///
    /// > a `panel` drawing its top border as one run and writing its title over it — **15**
    ///
    /// **The number *is* the title's width**, which is why it reproduces exactly where the other
    /// four rows of the table are magnitudes of a screen. It was measured on `block`;
    /// this is the same fifteen cells on the component, and the defective panel is **one verb
    /// cheaper** — which is the whole reason it survived review in the first place.
    #[test]
    fn a_panel_that_writes_its_title_over_its_border_costs_fifteen_cells() {
        const TITLE: &str = " panel systems ";
        assert_eq!(width(TITLE), 15);
        let opts = PanelOpts::default();

        let (correct, _) = tallied(40, 10, |tally, cx| {
            panel_into(tally, cx, cx.area(), TITLE, &opts).interior
        });
        let (broken, _) = tallied(40, 10, |tally, cx| {
            defective::panel_over_title(tally, cx, cx.area(), TITLE, &opts).interior
        });

        assert_eq!(correct.writes() - correct.distinct(), 0);
        assert_eq!(
            broken.writes() - broken.distinct(),
            u64::from(width(TITLE)),
            "the title's own width, written twice with two different values in one frame"
        );
        assert!(
            broken.verbs() < correct.verbs(),
            "the defect is supposed to be the cheap-looking one: one run instead of two"
        );
        assert_eq!(
            broken.distinct(),
            correct.distinct(),
            "and it covers exactly the same cells, so *no cell never* cannot see it either"
        );
    }

    /// **Criterion 1: a panel is one region and not a tab stop, and it answers with both halves.**
    #[test]
    fn a_panel_is_one_region_and_never_a_tab_stop() {
        let mut driver = Driver::headless(40, 10).expect("a sink attaches");
        let mut stood = None;
        driver.frame(|cx| stood = Some(panel(cx, cx.area(), " panel ")));
        let stood = stood.expect("one frame ran");
        assert_eq!((stood.response.rect.w, stood.response.rect.h), (40, 10));
        assert_eq!(driver.inspect().hits().len(), 1);
        assert_eq!(
            driver.inspect().stop_count(),
            0,
            "a panel is not a tab stop, which is what makes the dense screen 338 regions and 333 \
             stops"
        );
        assert!(stood.interior.w < 40 && stood.interior.h < 10);
    }

    /// **Criterion 7 for `rule`: a partition of the whole rectangle, at both orientations.**
    ///
    /// Swept where the arithmetic runs out — a one-cell rectangle, a caption wider than the line it
    /// sits on, and a rectangle tall enough that the rows either side of the line are somebody's.
    #[test]
    fn a_rule_writes_a_partition_of_its_whole_rectangle() {
        for orient in [Orient::Horizontal, Orient::Vertical] {
            for (w, h) in [
                (1u16, 1u16),
                (2, 1),
                (1, 2),
                (5, 3),
                (30, 1),
                (1, 30),
                (12, 4),
            ] {
                // **And at an origin that is not the screen's**, which is the arm
                // `crate::collect`'s table needed: a component that drew from `x = 0` rather than
                // from the band it was handed passed every gate, because every gate played where
                // the two agree.
                for (ox, oy) in [(0i32, 0i32), (5, 2)] {
                    for caption in ["", " Section ", "a caption far wider than this rule is"] {
                        for justify in [Justify::Start, Justify::Middle, Justify::End] {
                            let opts = RuleOpts {
                                orient,
                                justify,
                                ..RuleOpts::default()
                            };
                            let at = Rect::new(ox, oy, w, h);
                            let (tally, _) = tallied(w + ox as u16, h + oy as u16, |tally, cx| {
                                rule_into(tally, cx, at, caption, &opts);
                                Rect::new(0, 0, 0, 0)
                            });
                            assert_eq!(
                                tally.writes(),
                                tally.distinct(),
                                "{orient:?} {w}x{h}@{ox},{oy} `{caption}` {justify:?}: a cell twice"
                            );
                            assert_eq!(
                                tally.distinct(),
                                u64::from(w) * u64::from(h),
                                "{orient:?} {w}x{h}@{ox},{oy} `{caption}` {justify:?}: not a partition"
                            );
                            assert_eq!(
                                tally.asked(),
                                tally.reported(),
                                "{orient:?} {w}x{h}@{ox},{oy} `{caption}` {justify:?}: a verb left"
                            );
                            for y in oy..oy + i32::from(h) {
                                for x in ox..ox + i32::from(w) {
                                    assert!(
                                        tally.touched(x, y),
                                        "{orient:?} {w}x{h}@{ox},{oy}: ({x}, {y}) unwritten"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// **`fit`'s four skippable parts, on a rule: an empty caption is one verb and not three.**
    ///
    /// [`crate::text::fit_into`]'s own note is the rule being obeyed here — *a label that exactly
    /// fills its row is one verb and not three* — and a rule is that shape with a [`Glyph`] where
    /// the padding was. The three counts are the assertion: an empty caption is one run, a
    /// left-justified caption is two parts and a run, and a centred one is a run either side.
    #[test]
    fn a_rule_is_fits_four_skippable_parts_and_an_empty_caption_is_one_verb() {
        let verbs = |caption: &str, justify: Justify| {
            let (tally, _) = tallied(30, 1, |tally, cx| {
                rule_into(
                    tally,
                    cx,
                    Rect::new(0, 0, 30, 1),
                    caption,
                    &RuleOpts {
                        justify,
                        ..RuleOpts::default()
                    },
                );
                Rect::new(0, 0, 0, 0)
            });
            tally.verbs()
        };
        assert_eq!(verbs("", Justify::Start), 1, "an empty caption is one run");
        assert_eq!(
            verbs(" Section ", Justify::Start),
            2,
            "a caption against the start has nothing before it"
        );
        assert_eq!(
            verbs(" Section ", Justify::Middle),
            3,
            "a centred caption has a run either side"
        );
        assert_eq!(
            verbs(" Section ", Justify::End),
            2,
            "a caption against the end has nothing after it"
        );
        // **And the one-cell ellipsis rule holds on a rule as it does on a label**: the caption
        // goes through `glyphs::elide`, which reserves exactly one cell for the marker.
        //
        // **This block asserted nothing until a review said so.** It drew and dropped the tally,
        // under a comment claiming the rule was checked — which is an earlier pass's one-cell double
        // write, the defect an earlier pass found transcribed a second time, with no gate over it.
        let (elided, _) = tallied(8, 1, |tally, cx| {
            rule_into(
                tally,
                cx,
                Rect::new(0, 0, 8, 1),
                "a caption far wider than this",
                &RuleOpts::default(),
            );
            Rect::new(0, 0, 0, 0)
        });
        assert_eq!(
            (elided.writes(), elided.distinct()),
            (8, 8),
            "a truncated caption wrote the marker's cell twice, which is the one-cell rule going"
        );
        // Two verbs: the head with its marker written as one `text`, then nothing — a caption that
        // fills the row exactly has no glyph run either side of it.
        assert_eq!(elided.verbs(), 2, "the head and the one-cell marker");
    }

    /// **A vertical rule is one column and it has no caption at all.**
    ///
    /// `fit` writes *rows*, so the caption is the horizontal arm's alone. Asserted rather than left
    /// to a reader: a vertical rule handed a caption draws the same picture it draws without one,
    /// and the picture is a column of [`Glyph::VLine`] with padding either side.
    #[test]
    fn a_vertical_rule_is_one_column_and_has_no_caption() {
        let painted = |caption: &str| {
            let mut driver = Driver::headless(5, 3).expect("a sink attaches");
            let mut pen = crate::runner::Pen::over(crate::runner::Canvas::new(5, 3));
            driver.frame(|cx| {
                let area = cx.area();
                rule_into(
                    &mut pen,
                    cx,
                    area,
                    caption,
                    &RuleOpts {
                        orient: Orient::Vertical,
                        ..RuleOpts::default()
                    },
                );
            });
            pen.into_canvas()
        };
        assert_eq!(
            painted("").diff(&painted(" Section ")).cells,
            0,
            "a vertical rule drew its caption"
        );
        let canvas = painted("");
        for y in 0..3u16 {
            assert_eq!(
                canvas.row_text(y).trim_end(),
                "  \u{2502}",
                "row {y} is not one column of `VLine` in the middle"
            );
        }
    }

    /// **Criterion 1 for `rule`: three spellings, one drawer, and it declares nothing.**
    ///
    /// An L0 leaf — the freeze's own `layer` column — so there is no region, no tab stop and no
    /// `Ctx::interact` anywhere in its section. [`crate::composed`]'s scan is the source half; this
    /// is the count.
    #[test]
    fn a_rule_is_a_pure_drawer_and_the_three_spellings_are_one() {
        let mut driver = Driver::headless(30, 3).expect("a sink attaches");
        let mut painted = Vec::new();
        for arm in 0..3u8 {
            let mut pen = crate::runner::Pen::over(crate::runner::Canvas::new(30, 3));
            driver.frame(|cx| {
                let area = cx.area();
                let resp = match arm {
                    0 => rule(cx, area, " Section "),
                    1 => rule_with(cx, area, " Section ", &RuleOpts::default()),
                    _ => rule_into(&mut pen, cx, area, " Section ", &RuleOpts::default()),
                };
                // Rule 4: a `Response` back even from a pure drawer.
                assert_eq!((resp.rect.w, resp.rect.h), (30, 3));
                assert!(!resp.hovered && !resp.focused && !resp.changed);
            });
            if arm == 2 {
                painted.push(pen.into_canvas());
            }
        }
        assert_eq!(driver.inspect().hits().len(), 0, "a rule declared a region");
        assert_eq!(
            driver.inspect().stop_count(),
            0,
            "a rule declared a tab stop"
        );

        // **And the seam is deterministic**, which is all a second `Pen` can say: `Direct` writes
        // into the engine and *nothing reads a cell back*, so there is no surface to
        // compare the shipped path against and two `Pen`s compared is one path compared with
        // itself. Labelled as the equality it is not, this assertion held whatever `rule` did —
        // found by review, and the same shape is in `crate::input`'s toggles and its slider.
        let mut pen = crate::runner::Pen::over(crate::runner::Canvas::new(30, 3));
        driver.frame(|cx| {
            let area = cx.area();
            rule_into(&mut pen, cx, area, " Section ", &RuleOpts::default());
        });
        assert_eq!(
            pen.into_canvas().diff(&painted.remove(0)).cells,
            0,
            "two runs of the seam drew different pictures"
        );

        // **The claim the surface cannot make, made from the source instead**: the three spellings
        // are one body, which is a scan for the two calls that route into it.
        let source =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/structure.rs"))
                .expect("this file");
        let section = crate::composed::section(&source, "// `rule` — §17's Tier 2 divider");
        assert!(!section.is_empty());
        for owed in [
            "rule_with(cx, area, caption, &RuleOpts::default())",
            "rule_into(&mut Direct, cx, area, caption, opts)",
        ] {
            assert!(
                crate::dense::declares(section, owed),
                "`{owed}` is not in the shipped rule, so the three spellings are not one body"
            );
        }
    }

    // ── `status_bar` ─────────────────────────────────────────────────────────────────────────────

    /// A tally over one frame of `f` on a sink two cells larger than `w` by `h` on every side.
    ///
    /// **The margin is the point.** A component drawn at the screen's own edge is clipped by the
    /// screen, so a verb that runs past its rectangle costs nothing a counter can see — and this
    /// component's whole argument is that the band is a *view*, which is only worth asserting where
    /// there is somewhere for an overrun to land. A pager one module over was writing 5 cells into
    /// a 4-cell strip under a sweep that had no margin.
    fn tallied_bar(w: u16, h: u16, f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>)) -> Tally {
        let mut driver = Driver::headless(w + 4, h + 4).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| f(&mut tally, cx));
        tally
    }

    /// **Criterion 2, the partition half: a status bar writes every visible cell of its band exactly
    /// once, at every size, at both fills and at every offset.**
    ///
    /// The offset is what makes this worth sweeping rather than asserting once. A bar laid out in
    /// **content** coordinates can be scrolled past the end of its own text, and the cells the
    /// segments then cannot reach are cells nobody writes — which is the finding
    /// on an unbounded scroll area, arriving here as a trailing run rather than as a hole.
    #[test]
    fn a_status_bar_writes_every_visible_cell_of_its_band_exactly_once() {
        for (w, h) in [(1, 1), (3, 1), (12, 1), (40, 1), (40, 3), (7, 2), (80, 1)] {
            for fill in [Fill::Even, Fill::Natural] {
                for offset in [(0, 0), (3, 0), (40, 0), (0, 5), (400, 9)] {
                    for segments in [
                        &[][..],
                        &["ready"][..],
                        &["ready", "utf-8", "ln 1, col 1"][..],
                        &["a segment far wider than this whole bar is"][..],
                    ] {
                        let opts = StatusOpts {
                            fill,
                            ..StatusOpts::default()
                        };
                        let tally = tallied_bar(w, h, |tally, cx| {
                            status_bar_into(
                                tally,
                                cx,
                                Rect::new(2, 2, w, h),
                                segments,
                                offset,
                                &opts,
                            );
                        });
                        let cells = u64::from(w) * u64::from(h);
                        assert_eq!(
                            tally.writes(),
                            tally.distinct(),
                            "{w}x{h} {fill:?} {offset:?} {segments:?}: {} cells written twice",
                            tally.writes() - tally.distinct()
                        );
                        assert_eq!(
                            tally.distinct(),
                            cells,
                            "{w}x{h} {fill:?} {offset:?} {segments:?}: the band is not covered"
                        );
                    }
                }
            }
        }
    }

    /// **Criterion 2, the region half: one hit entry for the bar, never one per segment.**
    ///
    /// The rule one component over — *one hit entry for all four bands* — and the number that
    /// makes it a gate rather than a sentence is that it does not move with the segment count.
    #[test]
    fn a_status_bar_declares_one_hit_entry_however_many_segments_it_has() {
        for n in [0usize, 1, 3, 12] {
            let labels: Vec<String> = (0..n).map(|i| format!("seg {i}")).collect();
            let segments: Vec<&str> = labels.iter().map(String::as_str).collect();
            let mut driver = Driver::headless(60, 4).expect("a sink cannot fail to attach");
            driver.frame(|cx| {
                status_bar(cx, Rect::new(0, 3, 60, 1), &segments);
            });
            let frame = driver.inspect();
            assert_eq!(
                frame.hits().len(),
                1,
                "{n} segments declared {} regions",
                frame.hits().len()
            );
            // And it is not a tab stop: a status bar is read, not visited.
            assert_eq!(frame.stop_count(), 0);
        }
    }

    /// **The axis argument is the band's and not the bar's**, and this is the number that says so.
    ///
    /// The four bands differ in which offset they share. A status bar's content is one row derived
    /// from its own segments, so the **vertical** share has nothing to move: `Shares::Y` and
    /// `Shares::Neither` draw the same bar at every offset, and only `Shares::X` moves anything —
    /// and only at [`Fill::Natural`], because a bar laid out over its own visible width has nothing
    /// to scroll.
    ///
    /// Both halves are the assertion. A bar that ignored the argument entirely would pass the first
    /// and fail the second; one that transposed its layout would pass the second and fail the first.
    #[test]
    fn the_two_offsets_a_band_can_share_are_not_two_bars() {
        let segments = ["ready", "utf-8", "ln 1, col 1", "spaces: 4", "rust"];
        let drawn = |shares: Shares, fill: Fill, offset: (i32, i32)| -> crate::runner::Canvas {
            let mut driver = Driver::headless(30, 2).expect("a sink cannot fail to attach");
            let mut pen = crate::runner::Pen::new(30, 2);
            driver.frame(|cx| {
                status_bar_into(
                    &mut pen,
                    cx,
                    Rect::new(0, 0, 30, 2),
                    &segments,
                    offset,
                    &StatusOpts {
                        shares,
                        fill,
                        ..StatusOpts::default()
                    },
                );
            });
            pen.end_frame();
            pen.into_canvas()
        };

        for fill in [Fill::Even, Fill::Natural] {
            for offset in [(0, 0), (6, 0), (0, 4), (6, 4)] {
                let pinned = drawn(Shares::Neither, fill, offset);
                let vertical = drawn(Shares::Y, fill, offset);
                assert_eq!(
                    pinned.diff(&vertical).cells,
                    0,
                    "{fill:?} {offset:?}: a shared `y` moved a bar whose content has no rows"
                );
            }
        }

        // And the horizontal share is not decoration: at `Natural` the content is wider than the
        // band, so six columns of offset move it.
        let still = drawn(Shares::X, Fill::Natural, (0, 0));
        let scrolled = drawn(Shares::X, Fill::Natural, (6, 0));
        assert!(
            still.diff(&scrolled).cells > 0,
            "a shared `x` over content wider than the band moved nothing"
        );
        // At `Even` the layout is the window's, so there is nothing to scroll and the two agree.
        assert_eq!(
            drawn(Shares::X, Fill::Even, (0, 0))
                .diff(&drawn(Shares::X, Fill::Even, (6, 0)))
                .cells,
            0
        );
    }

    /// **The band is a view, and that is why an overrunning segment costs its own cells.**
    ///
    /// `crate::scroll::sticky`'s clip, priced: a bar in the middle of a screen whose text is far
    /// wider than its band writes **nothing at all** outside the band, at any offset. Written by
    /// arithmetic into the caller's context instead, the overrun lands on whatever is beside it, is
    /// overdrawn by that neighbour, and re-damages those cells on every steady frame for ever.
    #[test]
    fn a_segment_wider_than_its_band_is_clipped_and_not_written_beside_it() {
        let band = Rect::new(10, 2, 8, 1);
        let mut driver = Driver::headless(40, 6).expect("a sink cannot fail to attach");
        let mut pen = crate::runner::Pen::new(40, 6);
        driver.frame(|cx| {
            status_bar_into(
                &mut pen,
                cx,
                band,
                &["a label very much wider than eight cells", "and another"],
                (0, 0),
                &StatusOpts {
                    fill: Fill::Natural,
                    ..StatusOpts::default()
                },
            );
        });
        pen.end_frame();
        let canvas = pen.into_canvas();
        assert_eq!(canvas.written(), usize::from(band.w) * usize::from(band.h));
        for y in 0..6u16 {
            for x in 0..40u16 {
                let inside = (10..18).contains(&x) && y == 2;
                assert_eq!(
                    canvas.get(x, y).is_some(),
                    inside,
                    "({x}, {y}) is outside the band and was written"
                );
            }
        }
    }

    /// **Two status bars on one screen are two widgets and merge nothing**, on a
    /// component that draws a `#[track_caller]` helper inside its own body.
    ///
    /// `crate::scroll::sticky` is itself `#[track_caller]`, so its `Ctx::id` resolves to the line
    /// inside `status_bar_into` and **both bars' bands mint the same id**. That is harmless only
    /// because a band declares no region and returns an inert `Response`; the count is here so that
    /// the day a band declares one, this fails rather than making the second bar inert on a screen
    /// that renders perfectly. `crate::media`'s chrome shipped exactly that defect twice.
    #[test]
    fn two_status_bars_on_one_screen_are_two_widgets_and_merge_nothing() {
        let mut driver = Driver::headless(40, 4).expect("a sink cannot fail to attach");
        let mut ids = (None, None);
        driver.frame(|cx| {
            ids.0 = Some(status_bar(cx, Rect::new(0, 0, 40, 1), &["one", "two"]).id);
            ids.1 = Some(status_bar(cx, Rect::new(0, 3, 40, 1), &["three", "four"]).id);
        });
        assert_ne!(ids.0, ids.1, "two bars are one bar");
        let frame = driver.inspect();
        assert_eq!(frame.hits().len(), 2, "two bars, two regions");
        assert_eq!(frame.ids().merges(), 0);
    }

    /// **`status_bar` mints no band of its own**, which is criterion 2's other half read off the
    /// source rather than off a screen.
    ///
    /// The scan is `crate::composed`'s, and it runs there over the whole Tier 2 table. What is here
    /// is the one thing that table cannot say: that the call it looks for is the *component's* and
    /// not a second copy of `sticky`'s three lines with the clip left out.
    #[test]
    fn a_status_bar_is_stickys_construction_and_not_a_copy_of_it() {
        let source =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/structure.rs"))
                .expect("this file is here");
        let section = crate::composed::section(
            &source,
            "// `status_bar` — the Tier 2 band: `sticky`'s one construction, an axis argument, one hit entry",
        );
        assert!(!section.is_empty());
        assert!(crate::dense::declares(section, "crate::scroll::sticky("));
        for minted in [
            "cx.child(",
            ".scrolled(",
            "cx.scrollable(",
            "struct BarState",
        ] {
            assert!(
                !crate::dense::declares(section, minted),
                "`status_bar` mints `{minted}`, and it is a band rather than a second clip"
            );
        }
    }
}
