//! Scrolling: **two mechanisms that must never be conflated**, four direction bits, and one
//! sixteen-byte fact that crosses a frame.
//!
//! Spec §13; [ADR 0015](../../../docs/adr/) for why the one rectangle this subsystem keeps is not an
//! exception to *no geometry crosses a frame* but an instance of the rule it was mistaken for.
//!
//! # The two mechanisms
//!
//! |  | scroll area | virtualised collection |
//! |---|---|---|
//! | what moves | a content offset the **application** owns | an index range the **caller** draws |
//! | cost | ∝ **content** | ∝ **visible window** |
//! | needs | a content size, or the drawn extent | a length and a row height |
//! | use for | a form, a panel, a document page | rows, logs, files, processes |
//! | the verb | [`Ctx::scroll_scope`](crate::ctx::Ctx::scroll_scope) | [`Ctx::visible_rows`](crate::ctx::Ctx::visible_rows) inside one |
//!
//! **Choosing the wrong one is the single most expensive mistake available above this runtime**, and
//! it is expensive because both compile, both look right on a thousand rows, and only one of them
//! still works on a million. A `scroll_area` draws the whole content and lets the clip reject what is
//! off screen, so a million rows is a million draw calls; a virtualised collection asks
//! [`Ctx::visible_rows`](crate::ctx::Ctx::visible_rows) which rows can be reached and draws those. Measured in one round: the list
//! is **0.997–1.003×** from 1k to 1M and the area is **887–889×**, so the wrong pairing is
//! **7 025–7 076× — 2.03 ms against 0.29 µs, twenty times the frame budget.** The *right* pairing —
//! a full-screen area with a virtualised list inside it over the same million rows — is
//! **7.65–7.80 µs**.
//!
//! This is the engine's own invariant seen one layer up: *frame cost is proportional to visible
//! cells, never to data volume*. The runtime cannot stop a body from iterating a million rows — the
//! loop is the caller's — so what it does instead is make the difference **countable**:
//! `tests::a_scroll_area_costs_the_content_and_a_list_costs_the_window` counts what each body
//! iterated rather than timing it, because the count is the mechanism and the timing is the weather.
//!
//! # `Scrollable` is four directions, not two axes
//!
//! [`Scrollable`] is four bits and never two `bool`s. The chaining rule was always right and the
//! *declaration* under it defeated the rule: `can_scroll` as a pair of **axis** bools makes a list at
//! its bottom answer *"yes, vertically"*, because it can still go up — so it consumes every downward
//! click for ever and **twenty clicks into a five-row list holding twenty rows moved the enclosing
//! area by 0**. Matched on the wheel's sign, four direction bits differ from an axis matcher on
//! **16 of 64** enumerated cases and are **never looser** —
//! `tests::four_directions_differ_from_two_axes_on_sixteen_of_sixty_four` is the count, and it is a
//! count and not a sample because sixty-four is the whole space.
//!
//! **A component that publishes a scrollable region owes the pair per axis, computed from its
//! clamped offset** — [`Scrollable::between`] is that arithmetic, and doing it any other way is how
//! the defect comes back.
//!
//! # Wheel chaining, and the one channel that cannot wait for `end`
//!
//! The innermost scrollable under the pointer **that can still move the way the wheel is going**
//! consumes the click; otherwise it passes outward. Every other pointer outcome is awarded at `end`
//! from the index that has just drawn, and this one cannot be: **the offset is read during
//! the draw by the widget that owns it**, so an answer that arrives after the draw arrives after the
//! only reader.
//!
//! So the wheel is resolved in `begin`, from the **previous** frame's index and the direction bits
//! that index carries. The price is stated rather than hidden:
//!
//! > **The residue is one click, at each end stop and on an area's first frame.** It is a documented
//! > property, not a bug to be fixed later.
//!
//! Both residues are asserted as counts — the same twenty clicks over a warm frame and over a cold
//! one differ by exactly one, which is what makes *one* a measurement rather than a claim.
//!
//! # Scroll-into-view is one frame, and [`IntoView`] is the only thing that crosses
//!
//! A `Tab` onto a row below the fold scrolls it into view on the **next** frame, not the one after,
//! because it is resolved in `end` from the ring that has just drawn. That needs the ring to carry a
//! **content-coordinate rectangle** — which overturns *the ring carries no geometry* and nothing
//! beside it, because the rule was never "no geometry" but *geometry is needed inside a frame and
//! never across one*, and this rect is read at `end`, in the same breath as the press award.
//!
//! What crosses the frame boundary is [`IntoView`]: **16 bytes, naming an area and an offset, and no
//! `Rect`**. It is the only structure of this subsystem that does. The wheel's target is read from
//! the previous frame's hit index, which is the *documented exception* above and not a second
//! structure — nothing scroll-shaped is stored for it.
//!
//! Three rules that were failing tests before they were rules:
//!
//! - **It fires only for a keyboard-driven focus move.** A press already proves the widget was on
//!   screen, and an unconditional pull is the list's old bug: it fights the wheel, dragging the
//!   viewport back to the selection every time the user scrolls away from it.
//! - **[`Ctx::scroll_scope`](crate::ctx::Ctx::scroll_scope) scopes no identity.** §5's rule applied: a container that takes a
//!   closure renames its children, *except* the two that exist to wrap something already on screen.
//!   A scroll area that renamed its body would lose the focus and every widget's state the moment it
//!   appeared.
//! - **`to_content` resets at the area boundary.** A scrolled context **is** a content coordinate
//!   system, so a ring rectangle inside one is in the content coordinates of the area that encloses
//!   it and of nothing further out.
//!
//! **A virtualised collection is one tab stop.** The ring is built from what drew, so a row outside
//! the window has no entry and no rectangle: mapping a selection index to an offset is the
//! container's job and a **different mechanism**.
//!
//! # What stays out
//!
//! **A sticky header is a rect split, not a second scroll area.** Split the rectangle, draw the
//! header into the top slice and open the area over the rest. A second area would be a second entry
//! in the wheel chain, competing with the body it heads — the header would swallow clicks aimed at
//! the rows under it.
//!
//! **No momentum and no smooth-scroll physics.** A terminal delivers discrete wheel clicks, and an
//! animated deceleration would hold the frame clock at its ceiling after every flick — which is the
//! opposite of an idle application costing zero wakeups. A configurable [`Wheel`] lines-per-click is
//! the whole model.

use vitui_engine::Rect;

use crate::id::Id;

// Named rather than inlined, because the four appear in `between`, in `admits` and in every
// constant below, and a transposed shift in one of them is exactly the defect this type exists to
// remove.
const UP: u8 = 1 << 0;
const DOWN: u8 = 1 << 1;
const LEFT: u8 = 1 << 2;
const RIGHT: u8 = 1 << 3;

/// Where a widget can still move: **four directions, and never two axes.**
///
/// # Why the axis form is wrong rather than merely coarse
///
/// A wheel click is one *direction*. An axis bool answers a question nobody asked — *can this thing
/// move on the y axis at all* — and a collection sitting at its bottom answers **yes**, because it
/// can still go up. It therefore consumes every downward click, for ever, and the scroll area around
/// it never sees one. Twenty clicks into a five-row list holding twenty rows moved the enclosing area
/// by **0**.
///
/// The four-bit form is never *looser* than the axis form — it accepts a strict subset of the
/// sixty-four (bits × direction) cases — so replacing one with the other can only ever chain a click
/// outward that the axis form would have eaten.
///
/// ```
/// use vitui_runtime::scroll::Scrollable;
///
/// // Twenty rows in a five-row window, sitting at the bottom.
/// let at_the_bottom = Scrollable::between((0, 15), (0, 15));
/// assert!(at_the_bottom.contains(Scrollable::UP), "it can still go back up");
/// assert!(!at_the_bottom.admits((0, 1)), "and it takes no more downward clicks");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Default)]
pub struct Scrollable(u8);

impl Scrollable {
    /// It cannot move at all. **The honest answer for content that fits**, and the reason a wheel
    /// click over a short list reaches the panel around it.
    pub const NONE: Scrollable = Scrollable(0);
    /// It can move towards row zero.
    pub const UP: Scrollable = Scrollable(UP);
    /// It can move away from row zero.
    pub const DOWN: Scrollable = Scrollable(DOWN);
    /// It can move towards column zero.
    pub const LEFT: Scrollable = Scrollable(LEFT);
    /// It can move away from column zero.
    pub const RIGHT: Scrollable = Scrollable(RIGHT);
    /// Every direction. **What a widget that manages its own bounds declares** — an endless log, a
    /// terminal emulator, anything whose content has no stated size — and what
    /// [`Interest::SCROLL`](crate::ctx::Interest::SCROLL) alone means.
    pub const ALL: Scrollable = Scrollable(UP | DOWN | LEFT | RIGHT);

    /// Both.
    pub const fn with(self, other: Scrollable) -> Scrollable {
        Scrollable(self.0 | other.0)
    }

    /// Whether every direction of `other` is set.
    pub const fn contains(self, other: Scrollable) -> bool {
        self.0 & other.0 == other.0
    }

    /// Nowhere left to go.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// **The pair per axis, computed from the clamped offset** — the arithmetic every component that
    /// publishes a scrollable region owes, written once here so that nobody writes it twice.
    ///
    /// `max` is the largest offset the content admits (content size minus viewport, floored at zero),
    /// not the content size: an offset and its bound are the same quantity, and mixing the two is the
    /// off-by-a-viewport that makes the last screenful unreachable.
    ///
    /// The offset is **clamped before it is read**. An application that has just shrunk its content
    /// holds an offset past the new end for one frame, and asked about that offset directly the area
    /// answers *I can still go down* — one more click that moves nothing and, worse, does not chain.
    pub const fn between(offset: (i32, i32), max: (i32, i32)) -> Scrollable {
        let (mx, my) = (
            if max.0 > 0 { max.0 } else { 0 },
            if max.1 > 0 { max.1 } else { 0 },
        );
        // `clamp` is not `const` on the stable floor this crate builds against, and the pair of
        // comparisons is what it would have expanded to anyway.
        let ox = if offset.0 < 0 {
            0
        } else if offset.0 > mx {
            mx
        } else {
            offset.0
        };
        let oy = if offset.1 < 0 {
            0
        } else if offset.1 > my {
            my
        } else {
            offset.1
        };
        let mut bits = 0;
        if oy > 0 {
            bits |= UP;
        }
        if oy < my {
            bits |= DOWN;
        }
        if ox > 0 {
            bits |= LEFT;
        }
        if ox < mx {
            bits |= RIGHT;
        }
        Scrollable(bits)
    }

    /// Whether this region takes a wheel movement of `delta`.
    ///
    /// **Matched on the sign and never on the axis**, which is the whole of the four-bit form: a
    /// negative `y` is *up* and asks the `UP` bit, and a region that can only go up refuses it.
    ///
    /// A delta with both components set — which the wheel never produces, since a notch is one
    /// direction — is admitted if **either** component can move, because chaining a diagonal outward
    /// on the strength of the half that is stuck would strand the half that is not.
    pub const fn admits(self, delta: (i32, i32)) -> bool {
        (delta.1 < 0 && self.0 & UP != 0)
            || (delta.1 > 0 && self.0 & DOWN != 0)
            || (delta.0 < 0 && self.0 & LEFT != 0)
            || (delta.0 > 0 && self.0 & RIGHT != 0)
    }
}

impl std::fmt::Debug for Scrollable {
    /// The four letters, because `Scrollable(9)` in a failing assertion is a puzzle and `[U..R]` is
    /// the answer.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bit = |set: bool, letter: char| if set { letter } else { '.' };
        write!(
            f,
            "[{}{}{}{}]",
            bit(self.contains(Scrollable::UP), 'U'),
            bit(self.contains(Scrollable::DOWN), 'D'),
            bit(self.contains(Scrollable::LEFT), 'L'),
            bit(self.contains(Scrollable::RIGHT), 'R'),
        )
    }
}

/// **The only sixteen bytes of this subsystem that cross a frame**, and it names no `Rect`.
///
/// Resolved in `end` from the ring that has just drawn, read by
/// [`Ctx::take_into_view`](crate::ctx::Ctx::take_into_view) on the frame after, and gone. It carries
/// an *area* and a *delta*, and both choices are load-bearing:
///
/// - an **area id** rather than the entry's rectangle, because the rectangle is geometry and
///   geometry does not cross a frame — the rectangle was read at `end`, where it still meant
///   something;
/// - a **delta** rather than an absolute offset, because the application owns the offset. A delta
///   composes with whatever the application did to its own state in between; an absolute value
///   computed against last frame's content silently overwrites it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct IntoView {
    /// Which scroll area is being asked to move.
    pub area: Id,
    /// How far to move its offset, in content cells.
    pub by: (i32, i32),
}

/// How far one wheel click moves. **The whole motion model**, deliberately.
///
/// A terminal delivers discrete clicks with no magnitude on the wire, so there is nothing to
/// interpolate and nothing to decelerate. Momentum would mean the runtime asking for frames after the
/// user stopped asking for anything, which is the one thing an idle application may not cost — so
/// what is configurable is how many rows a click is worth, and nothing else.
///
/// **One row and one column by default**, because a notch is intent and the runtime has no
/// standing to multiply somebody's intent by three. An application whose terminal reports coarse
/// clicks raises it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Wheel {
    /// Rows per vertical click. **1.**
    pub lines_per_click: i32,
    /// Columns per horizontal click. **1.**
    pub columns_per_click: i32,
}

impl Default for Wheel {
    fn default() -> Wheel {
        Wheel {
            lines_per_click: 1,
            columns_per_click: 1,
        }
    }
}

/// One scroll area, as it stood when it drew.
///
/// **Frame-local and rebuilt from the draw**, exactly like the hit index and the focus ring: it is
/// cleared in `begin`, written by [`Ctx::scroll_scope`](crate::ctx::Ctx::scroll_scope)(crate::ctx::Ctx::scroll_scope) and read once,
/// in `end`, by the step that resolves scroll-into-view. Nothing here survives the frame — what
/// survives is [`IntoView`], which is why that type names an [`Id`] and this one is never handed
/// across a boundary.
///
/// It is therefore a **sixth** frame-local structure where spec §1 says five, and a finding against
/// that sentence rather than against ADR 0012's decision: the decision is *nothing retained*, and a
/// structure rebuilt from every draw and dead by `present` is on the right side of it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Area {
    /// Whose. Keys [`IntoView`], and is the id the area published its region under.
    pub id: Id,
    /// Its viewport, in the coordinates of the context that opened it.
    pub view: Rect,
    /// Its offset, clamped.
    pub offset: (i32, i32),
    /// The largest offset its content admits.
    pub max: (i32, i32),
}

impl Area {
    /// How far this area must move to bring `rect` — **in its own content coordinates** — inside its
    /// viewport. `(0, 0)` when the rectangle is already there.
    ///
    /// Two behaviours worth stating because a test would otherwise pin them by accident:
    ///
    /// - an entry **taller than the viewport** is aligned to its top, because the alternative shows
    ///   its end and hides the thing that just took the focus;
    /// - the answer is **clamped into `0..=max`**, so an entry the content cannot actually reach —
    ///   a stale rectangle, a shrunk source — asks for the closest offset that exists rather than
    ///   for one that does not.
    pub fn into_view(&self, rect: Rect) -> (i32, i32) {
        (
            Area::axis(
                rect.x,
                i32::from(rect.w),
                self.offset.0,
                i32::from(self.view.w),
                self.max.0,
            ),
            Area::axis(
                rect.y,
                i32::from(rect.h),
                self.offset.1,
                i32::from(self.view.h),
                self.max.1,
            ),
        )
    }

    /// One axis of it. Written once and called twice, because **two axes were one axis written
    /// twice** is this section's own recorded defect: `scroll_area` used opposite senses nine lines
    /// apart and `scroll_area_auto` had no horizontal branch at all.
    fn axis(start: i32, len: i32, offset: i32, window: i32, max: i32) -> i32 {
        let end = start + len;
        // An entry taller than the window can only ever show one end of itself, and the end it
        // shows is its **start** — the alternative scrolls past the thing that just took the focus
        // to show its last row. Written without this branch the arithmetic below aligns the bottom,
        // which looks right in a one-row test and is wrong for every entry that needs the rule.
        let want = if start < offset || len > window {
            start
        } else if end > offset + window {
            end - window
        } else {
            return 0;
        };
        want.clamp(0, max.max(0)) - offset
    }
}

#[cfg(test)]
mod tests {
    use std::time::Instant;

    use vitui_engine::{
        Buttons, Key, KeyCode, KeyKind, KeyText, Mods, Mouse, MouseKind, Rect, Wheel as Notch,
    };

    use super::*;
    use crate::ctx::{Ctx, Driver, Interest};
    use crate::focus::ScopeKind;
    use crate::id::Id;

    fn driver(w: u16, h: u16) -> Driver {
        Driver::headless(w, h).expect("a headless attach has no tty to fail on")
    }

    fn wheel(x: u16, y: u16, notch: Notch) -> Mouse {
        Mouse {
            x,
            y,
            kind: MouseKind::Wheel(notch),
            buttons: Buttons::NONE,
            mods: Mods::NONE,
            at: Instant::now(),
        }
    }

    fn moved(x: u16, y: u16) -> Mouse {
        Mouse {
            x,
            y,
            kind: MouseKind::Move,
            buttons: Buttons::NONE,
            mods: Mods::NONE,
            at: Instant::now(),
        }
    }

    fn tab() -> Key {
        Key {
            code: KeyCode::Tab,
            mods: Mods::NONE,
            kind: KeyKind::Press,
            text: KeyText::EMPTY,
            at: Instant::now(),
        }
    }

    /// The defective declaration, kept runnable so that the count below compares against something
    /// rather than against a memory of something. **A pair of axis bools**, derived from the same
    /// four bits, matched the way `architecture.md` §13 matched them.
    fn axis_matcher(s: Scrollable, delta: (i32, i32)) -> bool {
        let y = s.contains(Scrollable::UP) || s.contains(Scrollable::DOWN);
        let x = s.contains(Scrollable::LEFT) || s.contains(Scrollable::RIGHT);
        (delta.1 != 0 && y) || (delta.0 != 0 && x)
    }

    /// **The whole space, enumerated.** Sixteen bit patterns against four wheel directions.
    #[test]
    fn four_directions_differ_from_two_axes_on_sixteen_of_sixty_four() {
        let directions = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        let mut cases = 0;
        let mut differ = 0;
        for bits in 0..16u8 {
            let s = Scrollable::NONE
                .with(if bits & 1 != 0 {
                    Scrollable::UP
                } else {
                    Scrollable::NONE
                })
                .with(if bits & 2 != 0 {
                    Scrollable::DOWN
                } else {
                    Scrollable::NONE
                })
                .with(if bits & 4 != 0 {
                    Scrollable::LEFT
                } else {
                    Scrollable::NONE
                })
                .with(if bits & 8 != 0 {
                    Scrollable::RIGHT
                } else {
                    Scrollable::NONE
                });
            for d in directions {
                cases += 1;
                let four = s.admits(d);
                let two = axis_matcher(s, d);
                if four != two {
                    differ += 1;
                }
                assert!(
                    !(four && !two),
                    "the direction matcher is never looser than the axis matcher: {s:?} against \
                     {d:?}"
                );
            }
        }
        assert_eq!(cases, 64, "sixteen bit patterns by four directions");
        assert_eq!(
            differ, 16,
            "and they differ on exactly the cases where the opposite direction is the one that can \
             still move"
        );
    }

    /// The pair per axis, and it is computed from the **clamped** offset.
    #[test]
    fn a_scrollable_is_computed_from_the_clamped_offset() {
        assert_eq!(Scrollable::between((0, 0), (0, 15)), Scrollable::DOWN);
        assert_eq!(Scrollable::between((0, 15), (0, 15)), Scrollable::UP);
        assert_eq!(
            Scrollable::between((0, 7), (0, 15)),
            Scrollable::UP.with(Scrollable::DOWN)
        );
        assert_eq!(Scrollable::between((0, 0), (0, 0)), Scrollable::NONE);
        assert_eq!(
            Scrollable::between((0, 40), (0, 15)),
            Scrollable::UP,
            "an offset past the end is one frame of an application that just shrank its content, \
             and unclamped it answers `I can still go down`"
        );
    }

    /// **Sixteen bytes, and no `Rect`.**
    #[test]
    fn into_view_is_sixteen_bytes_and_names_no_rect() {
        assert_eq!(size_of::<IntoView>(), 16);
        assert_eq!(
            size_of::<IntoView>(),
            size_of::<Id>() + size_of::<(i32, i32)>(),
            "an id and an offset, with nothing else in it"
        );
        // What it would have been carrying the rectangle it is computed from.
        struct WithRect {
            _area: Id,
            _rect: Rect,
        }
        assert_eq!(size_of::<WithRect>(), 24);
    }

    /// The into-view arithmetic, on the one axis it is written on.
    #[test]
    fn an_entry_below_the_fold_asks_for_the_difference_and_one_above_it_asks_for_its_own_row() {
        let area = Area {
            id: Id::ROOT,
            view: Rect::new(0, 0, 20, 6),
            offset: (0, 0),
            max: (0, 34),
        };
        assert_eq!(area.into_view(Rect::new(0, 2, 8, 1)), (0, 0), "already in");
        assert_eq!(
            area.into_view(Rect::new(0, 9, 8, 1)),
            (0, 4),
            "row 9 with a six-row window at offset 0 needs offset 4"
        );
        let scrolled = Area {
            offset: (0, 20),
            ..area
        };
        assert_eq!(
            scrolled.into_view(Rect::new(0, 3, 8, 1)),
            (0, -17),
            "above the window, so its own row becomes the top one"
        );
        assert_eq!(
            scrolled.into_view(Rect::new(0, 100, 8, 1)),
            (0, 14),
            "clamped into the offsets that exist: 34 - 20"
        );
    }

    /// An entry taller than the viewport is aligned to its **top**.
    #[test]
    fn an_entry_taller_than_the_window_shows_its_start() {
        let area = Area {
            id: Id::ROOT,
            view: Rect::new(0, 0, 20, 4),
            offset: (0, 0),
            max: (0, 40),
        };
        assert_eq!(area.into_view(Rect::new(0, 10, 8, 9)), (0, 10));
    }

    /// **The two mechanisms, as a count.** Not as a timing: the count *is* the difference between
    /// them, and a count does not have weather.
    #[test]
    fn a_scroll_area_costs_the_content_and_a_list_costs_the_window() {
        const ROWS: i32 = 1_000_000;
        let mut d = driver(80, 24);
        let body = d.env().theme().paint(crate::theme::Role::Body);

        // The area: the body draws as if everything were visible and the clip rejects the rest.
        let mut area_rows = 0u64;
        d.frame(|cx| {
            cx.scroll_scope(
                Id::named("area"),
                Rect::new(0, 0, 80, 24),
                (0, 0),
                (0, ROWS),
                |cx| {
                    for y in 0..ROWS {
                        area_rows += 1;
                        cx.text(0, y, "row", body);
                    }
                },
            );
        });

        // The collection: the same million rows, through the range the viewport admits.
        let mut list_rows = 0u64;
        d.frame(|cx| {
            cx.scroll_scope(
                Id::named("list"),
                Rect::new(0, 0, 80, 24),
                (0, 0),
                (0, ROWS),
                |cx| {
                    let visible = cx.visible_rows();
                    for y in visible.start..visible.end.min(ROWS) {
                        list_rows += 1;
                        cx.text(0, y, "row", body);
                    }
                },
            );
        });

        assert_eq!(area_rows, 1_000_000, "∝ content");
        assert_eq!(list_rows, 24, "∝ the visible window");
        assert!(
            area_rows / list_rows >= 10_000,
            "the ratio is the finding, and it is 41 666× here: {area_rows} against {list_rows}"
        );
    }

    /// **The wheel chain, and the number the derivation gives.**
    ///
    /// Twenty clicks, one a frame, into a five-row list holding twenty rows, inside a scroll area
    /// that can also move down. The list's travel is `20 - 5 = 15`. Frame by frame, with the wheel
    /// resolved in `begin` from the **previous** frame's bits and the list publishing its bits
    /// *before* it applies what it was given:
    ///
    /// | frame | previous index says | who takes it | list offset after |
    /// |---|---|---|---|
    /// | 1 | nothing has drawn | **nobody** — residue #1 | 0 |
    /// | 2–16 | the list can go down | the list, fifteen times | 15 |
    /// | 17 | the list could go down *when it published at frame 16* | the list, moving **0** — residue #2 | 15 |
    /// | 18–20 | the list cannot go down | **the area**, three times | 15 |
    ///
    /// `20 = 1 + 15 + 1 + 3`, so **the area moves 3**. The axis matcher moves it by 0, for ever,
    /// because a list at its bottom can still go up.
    #[test]
    fn twenty_clicks_into_a_five_row_list_move_the_area_by_three() {
        let (area_moved, list_offset, cold_residue) = twenty_clicks(false);
        assert_eq!(list_offset, 15, "the list absorbed its whole travel");
        assert_eq!(cold_residue, 1, "one click on the area's first frame");
        assert_eq!(area_moved, 3, "20 = 1 + 15 + 1 + 3");
    }

    /// **The residue is one click, and this is what makes *one* a measurement.** The same twenty
    /// clicks over a frame that has already drawn lose the cold-start click and nothing else, so the
    /// area moves one further.
    #[test]
    fn a_warm_frame_loses_only_the_end_stop_residue() {
        let (cold, _, _) = twenty_clicks(false);
        let (warm, _, warm_residue) = twenty_clicks(true);
        assert_eq!(
            warm_residue, 0,
            "something had drawn, so the first click landed"
        );
        assert_eq!(
            warm - cold,
            1,
            "the difference between the two residues is one click"
        );
        assert_eq!(warm, 4, "20 = 15 + 1 + 4");
    }

    /// Twenty clicks through the chain. Returns `(how far the area moved, the list's final offset,
    /// how many clicks the cold first frame lost)`.
    fn twenty_clicks(warm: bool) -> (i32, i32, i32) {
        let mut d = driver(40, 20);
        let body = d.env().theme().paint(crate::theme::Role::Body);
        let outer = Id::named("area");
        let inner = Id::named("list");
        // The application's state, which is the whole point: the runtime owns neither offset.
        let mut area_offset = 0;
        let mut list_offset = 0;
        let mut lost = 0;

        let draw =
            |cx: &mut Ctx<'_, '_>, area_offset: &mut i32, list_offset: &mut i32, lost: &mut i32| {
                // The area: forty rows of content in a twenty-row viewport.
                let area_max = (0, 20);
                let r = cx.scrollable(
                    outer,
                    Rect::new(0, 0, 40, 20),
                    Interest::NONE,
                    Scrollable::between((0, *area_offset), area_max),
                );
                *area_offset = (*area_offset + r.scrolled.1).clamp(0, area_max.1);
                let took_outer = r.scrolled.1;
                cx.scroll_scope(
                    outer,
                    Rect::new(0, 0, 40, 20),
                    (0, *area_offset),
                    area_max,
                    |cx| {
                        // The list: twenty rows of content in a five-row viewport.
                        let list_max = (0, 15);
                        let r = cx.scrollable(
                            inner,
                            Rect::new(0, 0, 40, 5),
                            Interest::NONE,
                            Scrollable::between((0, *list_offset), list_max),
                        );
                        *list_offset = (*list_offset + r.scrolled.1).clamp(0, list_max.1);
                        if took_outer == 0 && r.scrolled.1 == 0 {
                            *lost += 1;
                        }
                        cx.scroll_scope(
                            inner,
                            Rect::new(0, 0, 40, 5),
                            (0, *list_offset),
                            list_max,
                            |cx| {
                                for y in 0..20 {
                                    cx.text(0, y, "row", body);
                                }
                            },
                        );
                    },
                );
            };

        if warm {
            d.post_mouse(moved(2, 2));
            d.frame(|cx| draw(cx, &mut area_offset, &mut list_offset, &mut lost));
            lost = 0;
        }
        let mut cold_lost = 0;
        for click in 0..20 {
            d.post_mouse(wheel(2, 2, Notch::Down));
            d.frame(|cx| draw(cx, &mut area_offset, &mut list_offset, &mut lost));
            if click == 0 && !warm {
                cold_lost = lost;
            }
        }
        (area_offset, list_offset, cold_lost)
    }

    /// The same twenty clicks with the defective declaration: the list at its bottom still reports
    /// its **axis** as movable, so it eats every click and **the area moves 0**.
    #[test]
    fn an_axis_matcher_moves_the_enclosing_area_by_zero() {
        let mut d = driver(40, 20);
        let outer = Id::named("area");
        let inner = Id::named("list");
        let mut area_offset = 0;
        let mut list_offset = 0;

        for _ in 0..20 {
            d.post_mouse(wheel(2, 2, Notch::Down));
            d.frame(|cx| {
                // What an axis bool lowers to: *movable on y* is both directions at once.
                let axis = |offset: i32, max: i32| {
                    if offset > 0 || offset < max {
                        Scrollable::UP.with(Scrollable::DOWN)
                    } else {
                        Scrollable::NONE
                    }
                };
                let r = cx.scrollable(
                    outer,
                    Rect::new(0, 0, 40, 20),
                    Interest::NONE,
                    axis(area_offset, 20),
                );
                area_offset = (area_offset + r.scrolled.1).clamp(0, 20);
                cx.scroll_scope(
                    outer,
                    Rect::new(0, 0, 40, 20),
                    (0, area_offset),
                    (0, 20),
                    |cx| {
                        let r = cx.scrollable(
                            inner,
                            Rect::new(0, 0, 40, 5),
                            Interest::NONE,
                            axis(list_offset, 15),
                        );
                        list_offset = (list_offset + r.scrolled.1).clamp(0, 15);
                    },
                );
            });
        }
        assert_eq!(
            list_offset, 15,
            "the list is at its bottom and stayed there"
        );
        assert_eq!(
            area_offset, 0,
            "and it went on consuming every click, because it could still go up"
        );
    }

    /// **The wheel resolves during the draw**, which is what a `scroll_area` needs and what no other
    /// pointer outcome may have: the offset is read by the widget that owns it, inside the draw.
    #[test]
    fn the_wheel_reaches_its_widget_on_the_frame_it_arrives() {
        let mut d = driver(20, 10);
        let list = Id::named("list");
        d.post_mouse(moved(2, 2));
        d.frame(|cx| {
            cx.scrollable(
                list,
                Rect::new(0, 0, 20, 10),
                Interest::NONE,
                Scrollable::ALL,
            );
        });
        d.post_mouse(wheel(2, 2, Notch::Down));
        let mut same_frame = (0, 0);
        d.frame(|cx| {
            same_frame = cx
                .scrollable(
                    list,
                    Rect::new(0, 0, 20, 10),
                    Interest::NONE,
                    Scrollable::ALL,
                )
                .scrolled;
        });
        assert_eq!(same_frame, (0, 1), "not on the frame after");
    }

    /// Lines-per-click is the whole motion model, and it is configuration.
    #[test]
    fn lines_per_click_is_the_whole_model() {
        assert_eq!(Wheel::default().lines_per_click, 1);
        assert_eq!(Wheel::default().columns_per_click, 1);
        let mut d = driver(20, 10);
        d.set_wheel(Wheel {
            lines_per_click: 3,
            columns_per_click: 1,
        });
        let list = Id::named("list");
        d.post_mouse(moved(2, 2));
        d.frame(|cx| {
            cx.scrollable(
                list,
                Rect::new(0, 0, 20, 10),
                Interest::NONE,
                Scrollable::ALL,
            );
        });
        d.post_mouse(wheel(2, 2, Notch::Down));
        let mut scrolled = (0, 0);
        d.frame(|cx| {
            scrolled = cx
                .scrollable(
                    list,
                    Rect::new(0, 0, 20, 10),
                    Interest::NONE,
                    Scrollable::ALL,
                )
                .scrolled;
        });
        assert_eq!(scrolled, (0, 3));
    }

    /// **Scroll-into-view fires for a `Tab` and not for a press.**
    #[test]
    fn into_view_fires_for_a_keyboard_move_and_a_press_does_not_pull() {
        // Forty one-row fields in a six-row viewport — spec §20's scene.
        let field = |i: i32| Id::keyed(Id::ROOT, u64::try_from(i).unwrap_or(0));
        let area = Id::named("form");
        let mut d = driver(20, 6);
        let mut offset = 0;
        let mut pulled = None;

        let draw = |cx: &mut Ctx<'_, '_>, offset: &mut i32, pulled: &mut Option<(i32, i32)>| {
            if let Some(by) = cx.take_into_view(area) {
                *pulled = Some(by);
                *offset = (*offset + by.1).clamp(0, 34);
            }
            cx.scroll_scope(area, Rect::new(0, 0, 20, 6), (0, *offset), (0, 34), |cx| {
                for i in 0..40 {
                    cx.interact(field(i), Rect::new(0, i, 20, 1), Interest::FOCUS);
                }
            });
        };

        // Tab seven times: the seventh stop is row 6, one below a six-row window.
        for _ in 0..7 {
            d.post_key(tab());
            while d.queued() > 0 {
                d.frame(|cx| draw(cx, &mut offset, &mut pulled));
            }
        }
        // The frame after the seventh Tab is the one that reads the request.
        d.frame(|cx| draw(cx, &mut offset, &mut pulled));
        assert_eq!(d.inspect().focused(), Some(field(6)));
        assert_eq!(
            pulled,
            Some((0, 1)),
            "one row, resolved in `end` and read next frame"
        );
        assert_eq!(offset, 1);

        // A press on a row already on screen pulls nothing.
        pulled = None;
        d.post_mouse(Mouse {
            x: 2,
            y: 2,
            kind: MouseKind::Down(vitui_engine::Button::Left),
            buttons: Buttons::NONE,
            mods: Mods::NONE,
            at: Instant::now(),
        });
        while d.queued() > 0 {
            d.frame(|cx| draw(cx, &mut offset, &mut pulled));
        }
        d.frame(|cx| draw(cx, &mut offset, &mut pulled));
        assert_eq!(
            pulled, None,
            "a press already proves the widget was on screen"
        );
    }

    /// **A frame that leaves an into-view request wakes the screen.**
    ///
    /// The regression for runtime architecture issue 33, and the reason it stood for four
    /// components: *a gate drives its own frames*. Every other test of this path draws a second
    /// frame because it wants to observe one, so the harness supplied the wake the runtime did not
    /// and the lag was invisible by construction. This one draws **one** frame per arm and asks the
    /// wakeup sink instead — the only question an application's `wait` will ever ask.
    ///
    /// Three arms, and the first is the control: a frame that asks for nothing parks, so the two
    /// below are measuring a wake rather than a driver that always wants one.
    #[test]
    fn a_frame_that_leaves_an_into_view_request_wakes_the_screen() {
        let area = Id::named("area");
        let field = |i: i32| Id::keyed(Id::ROOT, u64::try_from(i).unwrap_or(0));

        // Control: the same scope, drawn with nothing asking.
        let mut d = driver(20, 6);
        d.frame(|cx| {
            cx.scroll_scope(area, Rect::new(0, 0, 20, 6), (0, 0), (0, 34), |_cx| {});
        });
        assert_eq!(
            d.inspect().wakes().pending(),
            None,
            "a frame with no request parks, so the arms below measure the request"
        );

        // The explicit ask: `Ctx::request_into_view`, which is what an application's own key runs.
        d.frame(|cx| {
            cx.scroll_scope(area, Rect::new(0, 0, 20, 6), (0, 0), (0, 34), |cx| {
                cx.request_into_view(Rect::new(0, 20, 20, 1));
            });
        });
        assert!(
            d.inspect().into_view().is_some(),
            "row 20 is below a six-row window, so there is a request to carry"
        );
        assert!(
            d.inspect().wakes().pending().is_some(),
            "the frame that takes it has to be asked for; nothing else will ask"
        );

        // The keyboard ask: the ring's own pull, which no application spells at all.
        let mut d = driver(20, 6);
        let draw = |cx: &mut Ctx<'_, '_>| {
            cx.scroll_scope(area, Rect::new(0, 0, 20, 6), (0, 0), (0, 34), |cx| {
                for i in 0..40 {
                    cx.interact(field(i), Rect::new(0, i, 20, 1), Interest::FOCUS);
                }
            });
        };
        // **This arm needs its own control**, and the empty scope above is not one: it is a
        // different driver, and it moves no focus, declares no stop and settles no caret. Six tabs
        // land on rows 0..=5, every one of them inside the window, so the ring pulls nothing and
        // this frame is the same frame as the one below in everything but that.
        //
        // **The wake is no longer what separates the two arms, and that is a fix rather than a
        // loss.** `Frame::resolve_award` asks for a frame whenever the focus ends somewhere other
        // than it started, so a `Tab` that moves it asks whether or not anything is revealed — the
        // trap `commander` met, where a `Tab` consumed by the frame that painted the *old* ring
        // parked and the panels appeared not to swap until the next keystroke. So this control
        // states the wake it now expects, and the discriminator below is [`Frame::into_view`],
        // which is what this test is about.
        for _ in 0..6 {
            d.post_key(tab());
            while d.queued() > 0 {
                d.frame(draw);
            }
        }
        assert_eq!(d.inspect().focused(), Some(field(5)));
        assert!(d.inspect().into_view().is_none(), "row 5 is already in");
        assert!(
            d.inspect().wakes().pending().is_some(),
            "a tab that reveals nothing still asks for the frame that draws the move"
        );
        // And a frame that moves nothing parks, which is what stops the line above being green on a
        // driver that always wants another frame.
        d.frame(draw);
        assert_eq!(
            d.inspect().wakes().pending(),
            None,
            "the frame the tab asked for drew the move and asks for nothing itself"
        );

        // The seventh stop is row 6, one below the window.
        d.post_key(tab());
        while d.queued() > 0 {
            d.frame(draw);
        }
        assert_eq!(d.inspect().focused(), Some(field(6)));
        assert!(d.inspect().into_view().is_some(), "the ring pulled");
        assert!(
            d.inspect().wakes().pending().is_some(),
            "a keyboard reveal is the same two-frame gesture and asks the same way"
        );
    }

    /// **A ring rectangle is in the enclosing area's content coordinates**, which reset at the area
    /// boundary and nowhere else.
    #[test]
    fn to_content_resets_at_the_area_boundary() {
        let mut d = driver(20, 6);
        let row = Id::named("row");
        d.frame(|cx| {
            // Two containers deep before the area, so a root-coordinate answer would be visibly
            // different from a content-coordinate one.
            let mut pane = cx.child(Rect::new(3, 2, 17, 4));
            pane.scroll_scope(
                Id::named("area"),
                Rect::new(0, 0, 17, 4),
                (0, 100),
                (0, 400),
                |cx| {
                    cx.interact(row, Rect::new(0, 104, 8, 1), Interest::FOCUS);
                },
            );
        });
        assert_eq!(
            d.inspect().ring()[0].rect,
            Rect::new(0, 104, 8, 1),
            "row 104 of the content, with neither the two containers nor the offset in it"
        );
    }

    /// **`scroll_scope` scopes no identity**, so the ids inside one are equal entry for entry to the
    /// ids without it.
    #[test]
    fn a_scroll_scope_scopes_no_identity() {
        let mut d = driver(20, 6);
        let mut inside = Vec::new();
        d.frame(|cx| {
            cx.scroll_scope(
                Id::named("area"),
                Rect::new(0, 0, 20, 6),
                (0, 0),
                (0, 0),
                |cx| {
                    inside.push(cx.interact_here(Rect::new(0, 0, 4, 1), Interest::CLICK).id);
                    inside.push(cx.interact_here(Rect::new(0, 1, 4, 1), Interest::CLICK).id);
                },
            );
        });
        let mut outside = Vec::new();
        d.frame(|cx| {
            outside.push(cx.interact_here(Rect::new(0, 0, 4, 1), Interest::CLICK).id);
            outside.push(cx.interact_here(Rect::new(0, 1, 4, 1), Interest::CLICK).id);
        });
        // The two call sites differ, so compare the *shape*: the same two ids either way is what a
        // scope-free scroll area means, and ticket 09's equality gate is the general form.
        assert_eq!(inside.len(), 2);
        assert_ne!(inside[0], inside[1]);
        assert_ne!(outside[0], outside[1]);
        assert_eq!(
            d.inspect().ids().live(),
            2,
            "two ids and not four: the area added none of its own"
        );
    }

    /// **A virtualised collection is one tab stop**, whatever its length.
    #[test]
    fn a_virtualised_collection_is_one_tab_stop() {
        let mut d = driver(80, 24);
        let list = Id::named("list");
        d.frame(|cx| {
            // One `Group` over the whole collection, and the collection's own region is the one
            // tab stop inside it: what moves between rows is the component's business, reached
            // through `next_key` once the group holds the focus.
            cx.scope(Id::named("list-scope"), ScopeKind::Group, |cx| {
                cx.interact(list, Rect::new(0, 0, 80, 24), Interest::FOCUS);
                cx.scroll_scope(list, Rect::new(0, 0, 80, 24), (0, 0), (0, 999_976), |cx| {
                    let visible = cx.visible_rows();
                    for y in visible.start..visible.end.min(1_000_000) {
                        cx.interact(
                            Id::keyed(list, u64::try_from(y).unwrap_or(0)),
                            Rect::new(0, y, 80, 1),
                            Interest::CLICK,
                        );
                    }
                });
            });
        });
        assert_eq!(d.inspect().stop_count(), 1, "one stop over a million rows");
        assert_eq!(
            d.inspect().ring().len(),
            1,
            "and one ring entry: a row outside the window has no entry and no rectangle"
        );
    }

    /// **A frame that is not measuring must not maintain the measurement**, and a frame full of
    /// scroll areas is still not measuring.
    #[test]
    fn a_frame_with_scroll_areas_maintains_no_extent() {
        let mut d = driver(80, 24);
        let body = d.env().theme().paint(crate::theme::Role::Body);
        d.frame(|cx| {
            cx.scroll_scope(
                Id::named("a"),
                Rect::new(0, 0, 80, 24),
                (0, 0),
                (0, 100),
                |cx| {
                    cx.text(0, 40, "below the fold", body);
                },
            );
        });
        assert_eq!(d.inspect().extent(), None);
    }

    /// And the opt-in, which is what a scroll area over content of **unknown** size reads — one
    /// frame late, between frames, where the application owns the offset anyway.
    #[test]
    fn the_drawn_extent_is_opt_in_and_a_scroll_area_reads_it_a_frame_late() {
        let mut d = driver(80, 24);
        let body = d.env().theme().paint(crate::theme::Role::Body);
        d.measure_extent(true);
        d.frame(|cx| {
            cx.scroll_scope(
                Id::named("a"),
                Rect::new(0, 0, 80, 24),
                (0, 0),
                (0, 0),
                |cx| {
                    for y in 0..60 {
                        cx.text(0, y, "line", body);
                    }
                },
            );
        });
        let extent = d.inspect().extent().expect("asked for");
        assert_eq!(
            extent.h, 60,
            "sixty rows of content in a twenty-four-row window"
        );
        let max = (0, i32::from(extent.h) - 24);
        assert_eq!(max, (0, 36), "which is the `max` the next frame passes in");
    }

    /// **A sticky header is a rect split**, and the evidence is the wheel chain: one entry, not two.
    #[test]
    fn a_sticky_header_is_a_rect_split_and_not_a_second_entry_in_the_chain() {
        let mut d = driver(40, 10);
        let body = d.env().theme().paint(crate::theme::Role::Body);
        let panel = Id::named("panel");
        d.post_mouse(moved(2, 0));
        d.frame(|cx| {
            let (header, rest) = crate::layout::rect::split_at_v(cx.area(), 1);
            let mut top = cx.child(header);
            top.text(0, 0, "Name", body);
            let mut below = cx.child(rest);
            below.scrollable(
                panel,
                below.area(),
                Interest::NONE,
                Scrollable::between((0, 0), (0, 30)),
            );
        });
        let chain: Vec<_> = d
            .inspect()
            .hits()
            .iter()
            .filter(|h| !h.scrollable.is_empty())
            .collect();
        assert_eq!(
            chain.len(),
            1,
            "the header is not in the chain it would compete in"
        );
        assert!(
            !chain[0].over,
            "and the pointer on the header row is not over the body's region either"
        );
    }
}
