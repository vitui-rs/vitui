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

use vitui_runtime::{Ctx, Interest, Response, Role};

use crate::frame::{BlockOpts, block_into};
use crate::ink::{Direct, Ink};
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
    use vitui_runtime::layout::text::width;

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
}
