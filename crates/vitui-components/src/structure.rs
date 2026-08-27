//! **F2 structure**, ~38 entries, expressed by `panel`, `rule`, `block` and the operator layers.
//!
//! The reduction is R2 (spec §18): a container attribute in CSS is a component here and a component
//! there is an argument here, and an inventory-driven count is structurally blind to both
//! directions. A scrim is a complement and never a fill (§2), and there is no cut-out — a terminal
//! cell has no alpha channel, so a scrim cannot have a hole in it.
//!
//! `status_bar` is homed here rather than under F13 because spec §21's ticket 35 settles what it
//! **is**: the same construction as a sticky header or footer, one rectangle split and one hit
//! entry. What it is used for is F13's; what it is is F2's.

//! # `panel` is the one of the four primitives that does not write all of its rectangle
//!
//! Spec §2 states the rule in one sentence with two halves — *each cell it is responsible for is
//! written exactly once, and the cells it does not write are **named in its return value***. For
//! `text`, `chip` and `button` the second half is empty: they were handed a rectangle and they write
//! every cell of it. A panel is a container, so the second half is the whole point, and it is why
//! [`panel`] returns a [`Panel`] rather than a bare `Response`.
//!
//! **That is the second stated substitution on spec §1's shape and it is not a loophole.** The first
//! is `Rect` for `Rect` ([`vitui_runtime::layout::rect`]); this one is *rule 4 with the container's rectangle
//! beside the `Response`*, because §2's own sentence is unstatable in a bare `Response` and the
//! alternative that would make it statable — handing the interior to a closure — is **refused on two
//! measurements** in [`crate::frame`]'s header: it collides the border with the interior in the one
//! counter this whole rule is measured by (124 false double writes), and it renames the caller's
//! widgets, which `CONTEXT.md`'s identity rule forbids. So the interior comes back, and the field it
//! comes back in is named.

use vitui_runtime::layout::rect;
use vitui_runtime::layout::text::width;
use vitui_runtime::{Ctx, Glyph, Interest, Response, Role};

use crate::frame::{BlockOpts, block_into};
use crate::ink::{Direct, Ink};
use crate::scroll::Orient;
use crate::text::Justify;
use vitui_runtime::Rect;

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["panel", "rule", "status_bar"];

/// [`panel`]'s options.
///
/// Spec §1's rule 3: a `Default` struct, never a required builder.
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
    /// rectangles (spec §3).
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
/// Two fields because §1's rule 4 and §2's *named in its return value* are two obligations and a
/// container owes both. See this module's header for why the closure form that would collapse them
/// into one is refused.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Panel {
    /// What happened to the panel's own region. Rule 4.
    pub response: Response,
    /// **The interior it handed over and did not write.** §2's second half.
    pub interior: Rect,
}

/// **A frame, a title in its top run, a padding ring, and the interior handed back unwritten.**
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
// `rule` — §17's Tier 2 divider: `fit`'s remainder over one row or one column, plus a `Glyph`
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// [`rule`]'s options.
///
/// Spec §1's rule 3: a `Default` struct, never a required builder.
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
    /// hover award, so ADR 0026's *a label in a second paint is repainted every frame* does not
    /// reach it — there is nothing to repaint it. What a caption in [`Role::Dim`] over a line in
    /// [`Role::Border`] buys is the one thing §16 asks for: the caption and the line are told apart
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
/// The line is [`Glyph::HLine`] or [`Glyph::VLine`] — §16's `rule` family, two entries — and every
/// other cell of the rectangle is padding, because §2's rule is that *a component handed a rectangle
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
/// [`crate::glyphs::elide`] so that §16's one-cell ellipsis rule holds on a rule as it does on a
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
    let slack = w - used;
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

/// **The panel written the way ADR 0026's fifth row prices, kept because a gate nobody has watched
/// fail is not a gate.**
///
/// `pub` for the reason [`crate::frame::defective`] is, and it reaches that module rather than
/// re-writing the panel: the correct arm and this one are **one function with one boolean between
/// them**, so the diff a reviewer would have to catch is the diff the register can point at.
pub mod defective {
    use super::{Ctx, Ink, Panel, PanelOpts, Rect, block_opts};

    /// **A panel whose top border is one run with its title written over it.** ADR 0026's 15-cell
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
    /// §2's two equalities, restricted the way a container makes them: `writes == distinct` over the
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
    /// four rows of ADR 0026's table are magnitudes of a screen. Ticket 06 measured it on `block`;
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
                                "{orient:?} {w}x{h}@{ox},{oy} `{caption}` {justify:?}: not a                                  partition"
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
        // **And §16's one-cell ellipsis rule holds on a rule as it does on a label**: the caption
        // goes through `glyphs::elide`, which reserves exactly one cell for the marker.
        let (_, _) = tallied(30, 1, |tally, cx| {
            rule_into(
                tally,
                cx,
                Rect::new(0, 0, 8, 1),
                "a caption far wider than this",
                &RuleOpts::default(),
            );
            Rect::new(0, 0, 0, 0)
        });
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
        assert_eq!(driver.inspect().stop_count(), 0, "a rule is a tab stop");

        let mut pen = crate::runner::Pen::over(crate::runner::Canvas::new(30, 3));
        driver.frame(|cx| {
            let area = cx.area();
            rule_into(&mut pen, cx, area, " Section ", &RuleOpts::default());
        });
        assert_eq!(pen.into_canvas().diff(&painted.remove(0)).cells, 0);
    }
}
