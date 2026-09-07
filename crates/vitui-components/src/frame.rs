//! **`block` — border, title and padding ring — `face_paint`, and the helper spec §3 deleted.**
//!
//! Two of spec §3's helpers and the one it struck out live here, and they are the same argument
//! twice. [`block`] returns the rectangle it did not write, so nobody clears what somebody else is
//! about to draw; [`face_paint`] resolves a row's five bits to one paint **before a cell is
//! written**, so nobody lays a restyle over a drawn row; and `frame::focus_ring` is deleted because
//! it is the second of those two written the first way round. See [`Face`] for why the five bits
//! are a struct and not C02's four-armed enum, and [`WhyThereIsNoFocusRing`] for the pair that
//! keeps the deleted item deleted.
//!
//! Spec §3's table gives `block` one job and one shape: *border, title, padding ring* · **draws its
//! frame, returns the rectangle it did not write.** ADR 0026 states the consequence as a number: a
//! `block` that clears what it hands over costs **22 200 damaged cells a frame** across three
//! panels, every frame, for a screen that is not moving.
//!
//! # The interior is returned and not handed to a closure, and that is a measurement
//!
//! The candidate this ticket was written expecting is `block(cx, opts, |cx| …)`: the interior
//! arrives through [`Ctx::child`], is never named, and the crux of runtime architecture issue 22
//! evaporates. It matches ADR 0012's shape — *the clip stack **is** the call stack* — so it reads
//! like the better design rather than a workaround. **It is refused, and on evidence rather than on
//! taste.**
//!
//! - **~~It costs the counter, which is the whole ticket.~~ It did not, and components ticket 19
//!   struck this ground.** The argument was that `Ctx::child` narrows the clip *and* moves the
//!   origin, so one tally over a closure-form panel would union the border's `(0, 0)` with the
//!   interior's first cell — also `(0, 0)` — and report **124 double writes on a panel that has
//!   none**. The 124 were real and they were the *recorder's*: `Tally::distinct` unioned in the
//!   coordinates each verb was called in, and ticket 19 moved it into the frame's root coordinates
//!   (`vitui_runtime::Ctx::origin`, runtime architecture issue 32). The excess is now zero, and
//!   `tests::a_child_context_no_longer_collides_the_border_with_the_interior` is the same
//!   measurement inverted rather than deleted. **A ground that turned out to be an artefact of the
//!   instrument is struck rather than quietly kept**, and the decision does not move, because of
//!   the one below.
//! - **It renames the caller's widgets, and the rule against that is not negotiable.** `CONTEXT.md`,
//!   on identity: *"A container that returns a rectangle preserves its children's identity and one
//!   that takes a closure renames them, with `scope` and `scroll_scope` the deliberate exceptions."*
//!   A closure-taking `block` either roots its body in its own id — ADR 0027's *a container roots
//!   its children inside its own id* — in which case moving a widget into or out of a panel changes
//!   its `Id` and it loses its focus, its grab and its scroll association; or it roots nothing, in
//!   which case the closure buys nothing the caller cannot already write as `cx.child(…)` and
//!   `cx.with_id(…)` itself.
//!
//! So `block` returns, on the identity rule alone. What it returns is a [`Rect`] and not a `Rect`, because `Rect` cannot be
//! named from this package at all — [`vitui_runtime::layout::rect`] is that argument in full, and it is the
//! components-side answer to runtime architecture issue 22.
//!
//! # The 15-cell instance lives here
//!
//! > a `panel` drawing its top border as one run and writing its title over it — **15**
//!
//! It was written by the author of the partition rule, in the corrected build, immediately after
//! writing the corrected chip. **Nothing about a border run crossing five title cells looks like a
//! fill**, and it was found by a count and by nothing else. [`defective::block_over_title`] is that
//! panel, kept as a fixture for the reason `crate::runner::defective` keeps its four: a gate nobody
//! has watched fail is not a gate. The correct arm and the defective one are **one function with one
//! boolean between them**, so the diff a reviewer would have to catch is the diff the register can
//! point at.

use vitui_runtime::{Ctx, Glyph, Role};

use crate::glyphs::elide;
use crate::ink::{Direct, Ink};
use vitui_runtime::Rect;
use vitui_runtime::layout::rect;

/// The cluster a padding run is made of.
const PAD: &str = " ";

/// [`block`]'s options.
///
/// Spec §1's rule 3: a `Default` struct, never a required builder.
///
/// # `focus` is a [`Role`] on this struct and there is no `focus_ring`
///
/// Spec §3 deletes `frame::focus_ring` and says where it went: *a `Role` into `block`, a `Faces`
/// into `press`*. The `Role` is [`BlockOpts::border`] — a focused panel is drawn with
/// [`Role::Focus`] instead of [`Role::Border`], **before** the cell is written. See
/// [`WhyThereIsNoFocusRing`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BlockOpts<'a> {
    /// The title, written into the top run verbatim.
    ///
    /// **Verbatim, and the spaces are the caller's.** A caller wanting `─ Title ─` passes
    /// `" Title "`; `block` adding them would make the written width differ from the width the
    /// caller measured, which is the kind of hidden cell the 15-cell instance is made of.
    pub title: &'a str,
    /// The role the frame is drawn in. **This is where focus lands** — see the type's own note.
    pub border: Role,
    /// The role the title is drawn in.
    pub title_role: Role,
    /// The role the padding ring is drawn in.
    pub pad: Role,
    /// Whether to draw a frame at all. A panel without one still has a padding ring.
    pub bordered: bool,
    /// Whether to apply the theme's density as a padding ring.
    ///
    /// **Density is theme data, it changes rectangles, and this is where that lands**.
    /// [`vitui_runtime::Density::Compact`] pads one cell and `Cosy` two, so the same form fits a
    /// different number of widgets at the two — which is a fact about the screen and is reported
    /// rather than hidden.
    pub padded: bool,
}

impl Default for BlockOpts<'_> {
    fn default() -> BlockOpts<'static> {
        BlockOpts {
            title: "",
            border: Role::Border,
            title_role: Role::Title,
            pad: Role::Body,
            bordered: true,
            padded: true,
        }
    }
}

/// **Draw a frame with a title, and return the interior it did not write.**
///
/// ```
/// use vitui_runtime::Rect;
/// use vitui_components::frame::block;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(40, 10).expect("a sink attaches");
/// driver.frame(|cx| {
///     let interior = block(cx, cx.area(), " panel ");
///     // One cell of frame and the theme's padding ring; the rest is the caller's, and `block`
///     // has not touched it.
///     assert!(interior.w < 40 && interior.h < 10);
/// });
/// ```
pub fn block(cx: &mut Ctx<'_, '_>, area: Rect, title: &str) -> Rect {
    block_with(
        cx,
        area,
        &BlockOpts {
            title,
            ..BlockOpts::default()
        },
    )
}

/// [`block`], with the options spelled out.
pub fn block_with(cx: &mut Ctx<'_, '_>, area: Rect, opts: &BlockOpts<'_>) -> Rect {
    block_into(&mut Direct, cx, area, opts)
}

/// **[`block`], drawing through an [`Ink`] so a counter can see the verbs.**
///
/// The entry point a gate takes. See [`crate::ink`] for why the seam exists rather than a second
/// implementation of this function.
pub fn block_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    opts: &BlockOpts<'_>,
) -> Rect {
    draw(ink, cx, area, opts, true)
}

/// **The four rectangles the frame writes, then the ring, then the interior it returns.**
///
/// `title_split` is the one boolean between the correct panel and the 15-cell one. The top run is
/// written as **two runs with the title between them** when it is true, and as one run with the
/// title laid over it when it is false.
fn draw<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    opts: &BlockOpts<'_>,
    title_split: bool,
) -> Rect {
    let theme = cx.theme();
    let border = theme.paint(opts.border);
    let title_paint = theme.paint(opts.title_role);
    let pad_paint = theme.paint(opts.pad);

    let mut inner = area;
    if opts.bordered && area.w >= 2 && area.h >= 2 {
        let hline = theme.glyph(Glyph::HLine);
        let vline = theme.glyph(Glyph::VLine);
        let x0 = area.x;
        let y0 = area.y;
        let right = area.right() - 1;
        let bottom = area.bottom() - 1;
        // The columns strictly between the two corners. Everything on the top and bottom edges is
        // an offset into this, which is what stops a corner and a run disagreeing about one cell.
        let span = area.w - 2;

        // ── the top edge: corner, run, title, run, corner ────────────────────────────────────────
        ink.text(cx, x0, y0, theme.glyph(Glyph::TopLeft), border);
        let (head, marker, used) = plan_title(cx, opts.title, span);
        if used == 0 {
            ink.run(cx, x0 + 1, y0, hline, span, border);
        } else {
            // One glyph of frame before the title, so a title never touches a corner.
            let lead = 1u16;
            if title_split {
                ink.run(cx, x0 + 1, y0, hline, lead, border);
            } else {
                // **The defect.** One run across the whole span, and the title written over it
                // below: `used` cells written twice, with two different values, in one frame.
                ink.run(cx, x0 + 1, y0, hline, span, border);
            }
            let at = x0 + 1 + i32::from(lead);
            let head_w = vitui_runtime::layout::text::width(head);
            if head_w > 0 {
                ink.text(cx, at, y0, head, title_paint);
            }
            if !marker.is_empty() {
                ink.text(cx, at + i32::from(head_w), y0, marker, title_paint);
            }
            if title_split {
                ink.run(
                    cx,
                    at + i32::from(used),
                    y0,
                    hline,
                    span - lead - used,
                    border,
                );
            }
        }
        ink.text(cx, right, y0, theme.glyph(Glyph::TopRight), border);

        // ── the bottom edge ──────────────────────────────────────────────────────────────────────
        ink.text(cx, x0, bottom, theme.glyph(Glyph::BottomLeft), border);
        ink.run(cx, x0 + 1, bottom, hline, span, border);
        ink.text(cx, right, bottom, theme.glyph(Glyph::BottomRight), border);

        // ── the two sides, which are the rows the edges did not take ─────────────────────────────
        for row in 1..area.h - 1 {
            let y = y0 + i32::from(row);
            ink.text(cx, x0, y, vline, border);
            ink.text(cx, right, y, vline, border);
        }

        inner = rect::inset(area, 1);
    }

    if !opts.padded {
        return inner;
    }

    // ── the padding ring: four bands, and the four are a partition of `inner` minus the body ─────
    //
    // Computed by clamping rather than by `Rect::shrink`, because the interesting case is the one
    // where the ring is wider than what it is padding: a 3-row interior at `Cosy` has no body at
    // all, and the ring is then the whole of `inner` rather than three quarters of it.
    let density = theme.density();
    let (px, py) = (density.pad_x(), density.pad_y());
    let top_h = py.min(inner.h);
    let body_h = inner.h.saturating_sub(py.saturating_mul(2));
    let bottom_h = inner.h - top_h - body_h;
    let left_w = px.min(inner.w);
    let body_w = inner.w.saturating_sub(px.saturating_mul(2));
    let right_w = inner.w - left_w - body_w;

    let body = Rect::new(
        inner.x + i32::from(left_w),
        inner.y + i32::from(top_h),
        body_w,
        body_h,
    );

    fill_rows(
        ink,
        cx,
        Rect::new(inner.x, inner.y, inner.w, top_h),
        pad_paint,
    );
    fill_rows(
        ink,
        cx,
        Rect::new(inner.x, body.bottom(), inner.w, bottom_h),
        pad_paint,
    );
    fill_rows(
        ink,
        cx,
        Rect::new(inner.x, body.y, left_w, body_h),
        pad_paint,
    );
    fill_rows(
        ink,
        cx,
        Rect::new(body.right(), body.y, right_w, body_h),
        pad_paint,
    );

    body
}

/// The title as it will be written: the head, the one-cell marker, and how many cells the two take.
///
/// One glyph of frame is reserved on each side, so the budget is `span - 2`. A span with no room for
/// a title at all reports zero cells, and the caller writes one uninterrupted run.
fn plan_title<'a>(cx: &Ctx<'_, '_>, title: &'a str, span: u16) -> (&'a str, &'static str, u16) {
    if title.is_empty() || span < 3 {
        return ("", "", 0);
    }
    let (head, marker) = elide(cx.theme(), title, span - 2);
    let used =
        vitui_runtime::layout::text::width(head) + vitui_runtime::layout::text::width(marker);
    (head, marker, used)
}

/// Write every row of `cells` as one run of spaces. **Never `Ctx::fill`** — see [`crate::ink`]:
/// `fill` returns `()`, so a filled cell is modelled rather than reported and the pair stops being a
/// comparison between two sources.
fn fill_rows<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, cells: Rect, st: vitui_runtime::Paint) {
    if cells.is_empty() {
        return;
    }
    let x = cells.x;
    for row in 0..cells.h {
        ink.run(cx, x, cells.y + i32::from(row), PAD, cells.w, st);
    }
}

/// **What a row is, as five independent bits.**
///
/// Spec §3 narrowed C02's `selection` to this, and ADR 0026 states the shape: *selection becomes
/// `face_paint`, a `Paint` the row drawer is handed — which is a row signature, `(cx, rect, index,
/// Face)`.*
///
/// # C02's `Sel` enum does not survive, and the reason is that its variants are not exclusive
///
/// A row can be selected **and** hovered. It can be the keyboard cursor **without** being selected
/// — which is what `Ctrl+↓` does, and what every file manager draws. An enum whose arms are
/// `Rest | Hover | Selected | Cursor` cannot say either of those; it named **4** of the states five
/// bits can be in, and there are **32**. The four it named are the four a prototype's screen
/// happened to stand up.
///
/// # Five bytes, and five separate bools rather than a bitset
///
/// `size_of::<Face>() == 5`, asserted rather than assumed
/// (`tests::a_face_is_five_independent_bools_in_five_bytes`). A `u8` of flags would be one byte
/// and would also make `face.selected` a mask-and-compare at every call site, which is where a
/// missing highlight comes from.
///
/// **The trade is deliberate and it is stated here rather than discovered.** Five fields mean a row
/// drawer that binds one and forgets it gets `unused_variable`, which is `warnings = "deny"` at this
/// workspace and therefore a build failure. A *missing highlight* raises nothing at all: the screen
/// looks like a screen, and the row that should have been marked simply is not. An unused binding is
/// a warning; a missing highlight is not, and the cheap failure is the one worth having.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Face {
    /// It is in the selection.
    pub selected: bool,
    /// It is where the keyboard cursor is. **Independent of `selected`** — that independence is the
    /// whole reason this is a struct.
    pub cursor: bool,
    /// It is being acted on: a button down on it, a drag from it, an open disclosure.
    pub active: bool,
    /// The pointer is over it.
    pub hovered: bool,
    /// It cannot be interacted with. **Beats everything**, because a disabled row that looks
    /// selected is a row a user will try to act on.
    pub disabled: bool,
}

impl Face {
    /// A row in none of the five states.
    pub const REST: Face = Face {
        selected: false,
        cursor: false,
        active: false,
        hovered: false,
        disabled: false,
    };

    /// **All thirty-two states**, in the order the five bits count.
    ///
    /// The list [`face_role`]'s precedence is asserted over. It is generated from the bits rather
    /// than written out, because a hand-written list of thirty-two is a list somebody samples — and
    /// *asserted over all 32, not sampled* is criterion 5 in as many words.
    pub const ALL: [Face; 32] = Face::all();

    const fn all() -> [Face; 32] {
        let mut out = [Face::REST; 32];
        let mut bits = 0usize;
        while bits < 32 {
            out[bits] = Face {
                selected: bits & 1 != 0,
                cursor: bits & 2 != 0,
                active: bits & 4 != 0,
                hovered: bits & 8 != 0,
                disabled: bits & 16 != 0,
            };
            bits += 1;
        }
        out
    }

    /// How many of the five bits are set, for a report that wants the states in a readable order.
    pub const fn set(self) -> u32 {
        self.selected as u32
            + self.cursor as u32
            + self.active as u32
            + self.hovered as u32
            + self.disabled as u32
    }
}

/// **The role a [`Face`] resolves to. The one collapse point.**
///
/// The precedence is the decision, and it is stated once, here:
///
/// > **disabled > selected+active > selected > cursor > hovered > base**
///
/// and it is resolved **before a cell is written** — ADR 0026's *a selection or a focus ring is a
/// paint chosen before a cell is written, never a restyle laid over a drawn row*. A restyle used to
/// replace this branch re-damages its range every frame for ever: 26 cells for one focused field,
/// 31 for a selected row, 56 for a ring drawn into its neighbours' cells, 261 for a selected range,
/// and the restyled row is **7 cells different** from the correct one.
///
/// # Why the ladder is `if` and not a match on a tuple
///
/// A `match (selected, cursor, active, hovered, disabled)` is thirty-two arms and each of them is a
/// place the precedence can be written down differently. The ladder is six lines and the order of
/// the lines **is** the rule, which is what makes
/// `tests::the_precedence_holds_over_all_thirty_two_states` a check of one statement rather than
/// of thirty-two.
pub const fn face_role(face: Face) -> Role {
    if face.disabled {
        Role::Disabled
    } else if face.selected && face.active {
        Role::FaceActive
    } else if face.selected {
        Role::Selection
    } else if face.cursor {
        Role::Focus
    } else if face.hovered {
        Role::FaceHover
    } else {
        Role::Body
    }
}

/// **The [`Paint`](vitui_runtime::Paint) a row drawer is handed**, spec §3's own shape for it.
///
/// [`face_role`] with the theme applied, and it is deliberately nothing more: a second precedence
/// ladder for the paint-shaped caller is a second place the rule can be written down differently,
/// which is the whole defect C02's `Sel` enum was an instance of. A caller that needs the role —
/// to hand to [`crate::text::fit_with`]'s two role fields, say — takes `face_role`; a caller that
/// writes cells itself takes this.
///
/// ```
/// use vitui_components::frame::{Face, face_paint, face_role};
/// use vitui_runtime::ctx::Driver;
/// use vitui_runtime::Role;
///
/// let mut driver = Driver::headless(20, 3).expect("a sink attaches");
/// driver.frame(|cx| {
///     // Selected *and* hovered, which C02's enum could not say at all.
///     let both = Face { selected: true, hovered: true, ..Face::REST };
///     assert_eq!(face_role(both), Role::Selection);
///     assert_eq!(face_paint(cx.theme(), both), cx.theme().paint(Role::Selection));
///     // The keyboard cursor without the selection — `Ctrl+↓`, and every file manager.
///     assert_eq!(face_role(Face { cursor: true, ..Face::REST }), Role::Focus);
/// });
/// ```
pub fn face_paint(theme: &vitui_runtime::Theme, face: Face) -> vitui_runtime::Paint {
    theme.paint(face_role(face))
}

/// **The panel that broke the rule its own author had just written down.**
///
/// `pub` for the reason [`crate::runner::defective`] is `pub`: an instrument crate's fixtures are
/// part of the instrument, and a gate validated only against a correct build reports zero for the
/// same reason a broken one would.
pub mod defective {
    use super::{BlockOpts, Ctx, Ink, Rect, draw};

    /// **A top border drawn as one run, with the title written over it.**
    ///
    /// The 15-cell instance of ADR 0026's table, and the only one of that table's five that is not a
    /// fill. Everything else about this panel is correct — the corners, the sides, the bottom edge
    /// and the padding ring are the same cells the correct arm writes, in the same order — so the
    /// only thing that separates the two builds is `writes - distinct`, and the defective one is
    /// *faster*: one run instead of two, one verb fewer per panel.
    pub fn block_over_title<I: Ink>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        opts: &BlockOpts<'_>,
    ) -> Rect {
        draw(ink, cx, area, opts, false)
    }
}

/// **The pair that keeps `frame::focus_ring` deleted, naming both items by path.**
///
/// # Why it is deleted
///
/// ADR 0026's exact rule: *a restyle is free only when the component's own next draw already
/// produces the value the restyle produced.* A deferred hover award satisfies it, because the widget
/// also branches on `Response::hovered`. **A restyle used to *replace* the branch never can** — and
/// that is the entire appeal of a focus-ring helper. Built, it re-damages its range every frame
/// forever: 26 cells for one focused field, 31 for a selected row, 56 for a ring drawn into its
/// neighbours' cells, 261 for a selected range, and the restyled row is **7 cells different** from
/// the correct one, because a restyle repaints the row's sub-widgets.
///
/// So focus is a [`Role`] into [`block`] — [`BlockOpts::border`] — and a `Faces` into `press`, which
/// components ticket 07 builds. **A paint chosen before a cell is written, never a restyle laid over
/// a drawn row.**
///
/// # The twin, naming the protected item by path
///
/// A lone `compile_fail` also passes when the item it names has been *renamed*: `E0432` for *the
/// thing you must not have does not exist* and `E0432` for *the thing you meant moved* are the same
/// diagnostic, and the mechanism cannot tell them apart. So the shape that ships is pinned first —
/// renaming [`block`] or [`BlockOpts`] — `vitui_components::frame::block` and
/// `vitui_components::frame::BlockOpts`, which is how the twin below names them — fails
/// **this** half rather than making the half below pass for the wrong reason.
///
/// ```
/// use vitui_runtime::Rect;
/// use vitui_components::frame::{BlockOpts, block, block_with};
/// use vitui_runtime::ctx::Driver;
/// use vitui_runtime::Role;
///
/// let mut driver = Driver::headless(30, 8).expect("a sink attaches");
/// driver.frame(|cx| {
///     // Focus is this argument. There is nothing else it could be.
///     let opts = BlockOpts { title: " f ", border: Role::Focus, ..BlockOpts::default() };
///     let focused = block_with(cx, cx.area(), &opts);
///     let resting = block(cx, cx.area(), " f ");
///     assert_eq!(focused, resting, "focus changes the paint and never the rectangle");
/// });
/// ```
///
/// # The hostile half
///
/// **Protects:** [`block`], written `vitui_components::frame::block` at the path the twin uses,
/// by way of the item that must not stand beside
/// it.
///
/// ```compile_fail,E0432
/// use vitui_components::frame::focus_ring;
///
/// fn main() {}
/// ```
#[cfg(doc)]
pub struct WhyThereIsNoFocusRing;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counters::Tally;
    use crate::runner::driver_at;
    use std::path::PathBuf;
    use vitui_runtime::Density;
    use vitui_runtime::layout::text::width;

    /// A title fifteen columns wide, which is what makes the defect below **the** 15-cell instance
    /// rather than an instance of the same shape.
    const TITLE: &str = " panel systems ";

    fn tallied(
        w: u16,
        h: u16,
        density: Density,
        f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>),
    ) -> Tally {
        let mut driver = driver_at(w, h, density);
        let mut tally = Tally::new();
        driver.frame(|cx| f(&mut tally, cx));
        tally
    }

    /// **The title is fifteen columns**, asserted rather than assumed, because every number below
    /// is that number.
    #[test]
    fn the_title_is_the_fifteen_cells_the_defect_is_named_after() {
        assert_eq!(width(TITLE), 15);
    }

    /// **`block` writes a partition of its frame and ring, and returns the interior untouched** —
    /// criterion 2, at both densities and at every size that makes the arithmetic degenerate.
    #[test]
    fn block_writes_each_cell_once_and_never_the_interior() {
        for density in [Density::Compact, Density::Cosy] {
            for w in 0u16..14 {
                for h in 0u16..14 {
                    let mut interior = Rect::default();
                    let tally = tallied(w.max(1), h.max(1), density, |tally, cx| {
                        let opts = BlockOpts {
                            title: TITLE,
                            ..BlockOpts::default()
                        };
                        interior = block_into(tally, cx, Rect::new(0, 0, w, h), &opts);
                    });
                    let area = Rect::new(0, 0, w, h);
                    assert_eq!(
                        tally.writes(),
                        tally.distinct(),
                        "{density:?} {w}x{h}: a cell was written twice"
                    );
                    assert_eq!(
                        tally.reported(),
                        tally.writes(),
                        "{density:?} {w}x{h}: a write the engine did not report"
                    );
                    assert_eq!(
                        tally.distinct() + (u64::from(interior.w) * u64::from(interior.h)),
                        (u64::from(area.w) * u64::from(area.h)),
                        "{density:?} {w}x{h}: the frame plus the interior is not the rectangle"
                    );
                    for y in interior.y..interior.bottom() {
                        for x in interior.x..interior.right() {
                            assert!(
                                !tally.touched(x, y),
                                "{density:?} {w}x{h}: `block` wrote ({x}, {y}), which it handed over"
                            );
                        }
                    }
                }
            }
        }
    }

    /// **The gate fires on the 15-cell instance, and says fifteen** — criterion 5.
    ///
    /// Both arms are run on the same panel at the same density, so the only thing between them is
    /// `title_split`. The defective one writes **one verb fewer** and marks the same cells, which is
    /// the whole finding: nothing about a border run crossing a title looks like a fill, and no
    /// counter but this one moves in the direction of the defect.
    #[test]
    fn a_border_run_written_over_its_own_title_is_fifteen_double_writes() {
        let opts = BlockOpts {
            title: TITLE,
            ..BlockOpts::default()
        };
        let correct = tallied(60, 12, Density::Compact, |tally, cx| {
            block_into(tally, cx, cx.area(), &opts);
        });
        let defect = tallied(60, 12, Density::Compact, |tally, cx| {
            defective::block_over_title(tally, cx, cx.area(), &opts);
        });

        assert_eq!(
            correct.writes() - correct.distinct(),
            0,
            "the correct panel writes no cell twice"
        );
        assert_eq!(
            defect.writes() - defect.distinct(),
            u64::from(width(TITLE)),
            "the border run crosses the title in exactly the title's own width"
        );
        assert_eq!(defect.writes() - defect.distinct(), 15);
        assert_eq!(
            defect.distinct(),
            correct.distinct(),
            "the two panels touch the same cells, which is why only the pair separates them"
        );
        assert!(
            defect.verbs() < correct.verbs(),
            "the defective panel is one verb cheaper — {} against {}",
            defect.verbs(),
            correct.verbs()
        );
    }

    /// **One of the two grounds for returning the interior rather than handing it to a closure was
    /// an instrument defect, and components ticket 19 removed it.**
    ///
    /// This test used to assert the opposite of what it asserts now, and the change is worth
    /// reading rather than skipping. The closure form delivers the interior through [`Ctx::child`],
    /// which moves the origin; a single [`Tally`] over the panel then unioned the border's
    /// `(0, 0)` with the interior's first cell — also `(0, 0)` **in the child's coordinates** — and
    /// reported **124 double writes on a panel that has none**. That was ticket 06's first
    /// measurement against the closure form.
    ///
    /// It was never a fact about the closure; it was a fact about the counter. `Tally::distinct`
    /// unioned in the coordinates each verb was called in, and components ticket 19 moved it into
    /// the frame's root coordinates — `vitui_runtime::Ctx::origin`, runtime architecture issue 32.
    /// The excess is now **zero**, and this test is what says so.
    ///
    /// **The decision does not move with it**, and that is the point of keeping the test rather
    /// than deleting it. `block` returns its interior for the *second* reason, which is not a
    /// measurement and cannot be repaired by one: `CONTEXT.md`'s identity rule — *a container that
    /// returns a rectangle preserves its children's identity and one that takes a closure renames
    /// them* — and `Ctx::child` is there so the **caller** can narrow. A ground that turned out to
    /// be an artefact is struck; the one that stands is stated in `block`'s own rustdoc.
    #[test]
    fn a_child_context_no_longer_collides_the_border_with_the_interior() {
        let mut driver = driver_at(60, 12, Density::Compact);
        let mut tally = Tally::new();
        let mut interior = Rect::default();
        let mut frame_only = Tally::new();
        driver.frame(|cx| {
            let opts = BlockOpts {
                title: TITLE,
                ..BlockOpts::default()
            };
            interior = block_into(&mut tally, cx, cx.area(), &opts);
            // What the frame and the ring touched, before the body draws a single cell.
            frame_only = tally.clone();
            // The closure form, spelled out: the caller's body draws through a narrowed context.
            let mut child_cx = cx.child(interior);
            let child = &mut child_cx;
            {
                let paint = child.theme().paint(Role::Body);
                for row in 0..interior.h {
                    Ink::run(&mut tally, child, 0, i32::from(row), PAD, interior.w, paint);
                }
            }
        });
        // A cell of the body lands on a cell the frame wrote whenever its **local** coordinate is
        // also a coordinate the frame wrote at — which is every frame cell inside the child's own
        // rectangle, starting with the top-left corner at `(0, 0)`.
        let phantom: u64 = (0..interior.h)
            .flat_map(|y| (0..interior.w).map(move |x| (x, y)))
            .filter(|(x, y)| frame_only.touched(i32::from(*x), i32::from(*y)))
            .count() as u64;
        let excess = tally.writes() - tally.distinct();
        assert!(
            phantom > 0,
            "the cells that used to collide are the point of this test and there are none of them"
        );
        assert_eq!(
            phantom, 124,
            "the 124 of ticket 06's own note, so this test is the same measurement inverted rather \
             than a different one"
        );
        assert_eq!(
            excess,
            0,
            "the panel has no double write and the counter now says so: {phantom} cells of the {} \
             the body wrote share a *local* coordinate with a cell the frame wrote, and none of \
             them shares a root one",
            (u64::from(interior.w) * u64::from(interior.h))
        );
    }

    /// **Density changes the rectangle, and the change is the padding** — criterion 6's mechanism,
    /// on `block` alone. The screen-level count is `crate::form`'s.
    #[test]
    fn compact_hands_over_a_larger_interior_than_cosy_and_writes_less_ring() {
        let mut compact = Rect::default();
        let compact_tally = tallied(100, 40, Density::Compact, |tally, cx| {
            compact = block_into(tally, cx, cx.area(), &BlockOpts::default());
        });
        let mut cosy = Rect::default();
        let cosy_tally = tallied(100, 40, Density::Cosy, |tally, cx| {
            cosy = block_into(tally, cx, cx.area(), &BlockOpts::default());
        });
        assert_eq!((compact.w, compact.h), (96, 36));
        assert_eq!((cosy.w, cosy.h), (94, 34));
        assert!(
            (u64::from(compact.w) * u64::from(compact.h)) > (u64::from(cosy.w) * u64::from(cosy.h))
        );
        assert!(
            compact_tally.writes() < cosy_tally.writes(),
            "a bigger ring is more cells of ring"
        );
        for tally in [&compact_tally, &cosy_tally] {
            assert_eq!(
                tally.writes(),
                tally.distinct(),
                "neither writes a cell twice"
            );
        }
    }

    /// **A rectangle too small for a frame is still a partition.**
    ///
    /// The degenerate cases are where a partition helper usually stops being one: a one-row
    /// rectangle has no room for two edges, and a `Cosy` ring is wider than a three-row interior.
    /// Both are covered by the sweep above; this one names them so a failure reads as the case it
    /// is.
    #[test]
    fn a_rectangle_with_no_room_for_a_frame_hands_back_what_it_could_not_use() {
        for (w, h) in [(1u16, 1u16), (1, 20), (20, 1), (4, 4), (5, 5)] {
            let mut interior = Rect::default();
            let tally = tallied(w, h, Density::Cosy, |tally, cx| {
                interior = block_into(tally, cx, cx.area(), &BlockOpts::default());
            });
            assert_eq!(
                tally.distinct() + (u64::from(interior.w) * u64::from(interior.h)),
                u64::from(w) * u64::from(h),
                "{w}x{h}: the partition does not cover"
            );
            assert_eq!(tally.writes(), tally.distinct(), "{w}x{h}: written twice");
        }
    }

    /// **`focus_ring` is not declared anywhere in this crate** — criterion 7's source half.
    ///
    /// The `compile_fail` pair on [`WhyThereIsNoFocusRing`] catches the item coming back on the
    /// *public* surface. This catches it coming back as a private one, which is how a deleted helper
    /// actually returns: somebody writes it `pub(crate)` "just for the panel", and the pair stays
    /// green because nothing outside can name it.
    #[test]
    fn no_source_file_in_this_crate_declares_a_focus_ring() {
        let src = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        let mut offenders = Vec::new();
        let mut files = Vec::new();
        collect(&src, &mut files);
        assert!(files.len() > 10, "the scan found no source to scan");
        // **Assembled rather than written**, and it is not a flourish: the first run of this test
        // reported *this line* as an offender, because a scanner looking for a literal contains
        // that literal. A gate that fails on itself is a gate nobody can make pass.
        let declaration = ["fn ", "focus_ring"].concat();
        let marker = ["struct ", "FocusRing"].concat();
        for file in &files {
            let source = std::fs::read_to_string(file).expect("a readable source file");
            for (n, line) in source.lines().enumerate() {
                let line = line.trim();
                // A mention in prose is the point — this module argues about the deleted item at
                // length — so only a declaration counts.
                if line.starts_with("//") {
                    continue;
                }
                if line.contains(&declaration) || line.contains(&marker) {
                    offenders.push(format!("{}:{}", file.display(), n + 1));
                }
            }
        }
        assert!(
            offenders.is_empty(),
            "`frame::focus_ring` is deleted by spec §3 and ADR 0026, and it is declared at {}. A \
             restyle used to replace a branch re-damages its range every frame forever: 26 / 31 / \
             56 / 261 cells, and the restyled row is 7 cells different from the correct one. Focus \
             is `BlockOpts::border` and a `Faces` into `press`",
            offenders.join(", ")
        );
    }

    /// **Five independent bools in five bytes**, and `Face::ALL` really is every one of the
    /// thirty-two — criterion 4.
    ///
    /// The size is asserted because it is the sentence the ticket is written in, and the
    /// distinctness of `ALL` is asserted because a generated list that repeats an entry would make
    /// the exhaustive test below sample without saying so.
    #[test]
    fn a_face_is_five_independent_bools_in_five_bytes() {
        assert_eq!(std::mem::size_of::<Face>(), 5);
        assert_eq!(Face::ALL.len(), 32);
        let distinct: std::collections::BTreeSet<[bool; 5]> = Face::ALL
            .iter()
            .map(|f| [f.selected, f.cursor, f.active, f.hovered, f.disabled])
            .collect();
        assert_eq!(distinct.len(), 32, "`Face::ALL` repeats a state");

        // **The two states C02's four-armed enum could not name**, which is the whole argument for
        // the struct: `Sel::Selected | Sel::Hover | Sel::Cursor | Sel::Rest` has no way to say
        // either, and both are on every file manager's screen.
        assert!(Face::ALL.contains(&Face {
            selected: true,
            hovered: true,
            ..Face::REST
        }));
        assert!(Face::ALL.contains(&Face {
            cursor: true,
            ..Face::REST
        }));
    }

    /// **The precedence, asserted over all thirty-two states and not sampled** — criterion 5.
    ///
    /// The oracle is the rule written as a **predicate over the bits**, independently of the
    /// ladder: for each state, the expected role is the first rung whose condition holds, computed
    /// from a table of `(condition, role)` in precedence order. It shares no line with
    /// [`face_role`], which is the engine's `reference.rs` rule — *share no code with the fast
    /// path* — restated for six lines of `if`.
    ///
    /// Every rung is asserted to be **reached** by at least one state as well, because a ladder
    /// whose third rung is unreachable passes a comparison against an oracle with the same bug.
    #[test]
    fn the_precedence_holds_over_all_thirty_two_states() {
        // disabled > selected+active > selected > cursor > hovered > base.
        type Rung = (fn(Face) -> bool, Role);
        let ladder: [Rung; 5] = [
            (|f: Face| f.disabled, Role::Disabled),
            (|f: Face| f.selected && f.active, Role::FaceActive),
            (|f: Face| f.selected, Role::Selection),
            (|f: Face| f.cursor, Role::Focus),
            (|f: Face| f.hovered, Role::FaceHover),
        ];
        let mut reached = [0usize; 6];
        let mut seen = 0usize;
        for face in Face::ALL {
            let (expected, rung) = ladder
                .iter()
                .enumerate()
                .find(|(_, (holds, _))| holds(face))
                .map_or((Role::Body, 5), |(i, (_, role))| (*role, i));
            assert_eq!(
                face_role(face),
                expected,
                "{face:?}: the precedence is disabled > selected+active > selected > cursor > \
                 hovered > base, and this state resolves the wrong way"
            );
            reached[rung] += 1;
            seen += 1;
        }
        assert_eq!(seen, 32, "the sweep is not over all thirty-two states");
        for (rung, count) in reached.iter().enumerate() {
            assert!(
                *count > 0,
                "rung {rung} of the precedence is reached by none of the thirty-two states, so \
                 the oracle and the ladder could share a defect there and agree"
            );
        }
        // The base case, named: a row in none of the five states is ordinary body text and not a
        // face at all, which is why a list of ten thousand rows costs no face lookup.
        assert_eq!(face_role(Face::REST), Role::Body);
    }

    /// **`face_paint` is `face_role` with the theme applied and nothing else** — criterion 5's
    /// *one collapse point*, over all thirty-two states.
    ///
    /// A second ladder inside `face_paint` would pass every test that only checks the four states a
    /// screen stands up, which is exactly how C02's enum survived as long as it did.
    #[test]
    fn face_paint_resolves_through_the_same_ladder_at_every_state() {
        let mut driver = driver_at(20, 3, Density::Compact);
        driver.frame(|cx| {
            let theme = cx.theme();
            for face in Face::ALL {
                assert_eq!(
                    face_paint(theme, face),
                    theme.paint(face_role(face)),
                    "{face:?}: `face_paint` and `face_role` disagree, so there are two ladders"
                );
            }
        });
    }

    fn collect(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let path = entry.expect("a readable entry").path();
            if path.is_dir() {
                collect(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
}
