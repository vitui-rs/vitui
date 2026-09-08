//! Overlays: the two-phase protocol, the body queue, placement and the scrim.
//!
//! **Request during the draw, satisfy after it, answer
//! next frame.** The constraint is inherited and not re-derived here: a component cannot open a
//! layer mid-draw, because `LayerStack::view` holds `&mut` of the stack for the life of the view
//! (`E0499`). So [`Ctx::overlay`](crate::Ctx::overlay) queues a
//! request and [`Driver::frame`](crate::Driver::frame) runs the bodies after the base pass, in
//! `(z, seq)` order, in rounds bounded at [`OVERLAY_ROUNDS`](crate::ctx::OVERLAY_ROUNDS).
//!
//! # The owner id is handed over, and that is what makes the failure impossible
//!
//! `cx.overlay(area, opts, body)` — the shape the proposal had — cannot work. A component carries
//! `#[track_caller]` and the attribute reaches *into* the body, so an id derived inside the request
//! is the caller's location, which is the id the owner claimed one line earlier. The collision
//! policy then makes the second claimant inert and **the overlay is silently dropped**.
//!
//! The verb therefore takes the owner as an argument, and there is no second spelling that derives
//! one: the failure is removed rather than documented. The owner id does two jobs at once —
//!
//! - it **keys the layer's lifecycle** across frames, so a layer requested again is reused; and
//! - it **roots the overlay's own id stack**, which is what makes an overlay a different *place*
//!   for identity and not only for geometry. Rooted at [`Id::ROOT`](crate::Id::ROOT) instead, a body
//!   drawn inline and the same body drawn in an overlay produce the same ids — the failing test that
//!   designed this, kept as
//!   `overlay_tests::a_body_drawn_inline_and_in_an_overlay_produces_different_ids`.
//!
//! # Where a body lives between the two passes, and what it costs
//!
//! A body outlives the base pass and is run after every base-pass draw context has been dropped, so
//! it has to be *stored*. It is stored as a `Box`, one per request per frame, in the crate-private
//! `Bodies` queue that [`Driver::frame`](crate::Driver::frame) owns for the length of the frame
//! call. The count is reported by
//! [`Frame::overlay_bodies_boxed`](crate::ctx::Frame::overlay_bodies_boxed).
//!
//! **That replaces a bump region that allocated nothing, and the trade was decided rather than
//! discovered.** The arena held the bodies' bytes with their types erased, which needs `unsafe`; the
//! property *no `unsafe` in any shipped crate above the engine* is worth more than the sentence *the
//! overlay frame allocates nothing*: **n** overlays standing cost **n + 1** allocations, and a
//! frame with none costs nothing at all.
//!
//! **The queue cannot keep its capacity across frames, and the `'f` bound is the reason.** A body may
//! capture anything that outlives the frame call, so a stored body is `+ 'f` — and safe Rust cannot
//! put a `'f`-bounded value inside the thing that is borrowed for `'f`, which is what a `Frame` and a
//! [`Driver`](crate::Driver) both are. So the queue is a local of the frame call, it starts empty, and
//! a frame with **n** bodies costs **n + 1** allocations: one `Box` a body, plus the one `Vec` that
//! holds them. A frame with no overlay standing allocates nothing at all, because an empty `Vec` does
//! not.
//!
//! **Nothing drops a body by hand.** A `Box` drops what it holds, the `Vec` drops the boxes, and the
//! queue is dropped when the frame call ends — including the bodies the pass never reached, and
//! including an unwind. Both of the arena's own mechanisms are gone with it: the drop thunk the
//! request carried beside the call thunk, and the double buffering that stopped a body reallocating
//! the region it was executing out of. A `Vec<Box<…>>` that grows moves the pointers, not the
//! closures.
//!
//! # Placement is four steps in one order
//!
//! [`place`] is integer arithmetic: **place, flip, shift, clamp**. Flipping is conditional on the
//! other side having *more* room, so a tie keeps the side that was asked for; clamping is last and
//! **never resizes**. Verified exhaustively over 28 800 cases — see
//! `tests::placement_is_exhaustive_over_twenty_eight_thousand_eight_hundred_cases`.
//!
//! # A modal is an overlay plus a scrim plus a `Trap`
//!
//! Three mechanisms, none of them new here:
//!
//! - the overlay, from this module;
//! - the [`Scrim`], an *operator* layer, because a terminal cell has no alpha — it transforms what is
//!   already there rather than covering it;
//! - [`ScopeKind::Trap`](crate::ScopeKind::Trap), for the keyboard.
//!
//! and [`Ctx::modal_barrier_here`](crate::Ctx::modal_barrier_here) for the pointer.
//! **The barrier stops the pointer and only the pointer**: a barrier with no trap around it lets
//! `Tab` walk straight out of the modal, which is gated rather than assumed.
//!
//! # The obligation this module puts on everything above it
//!
//! A modal built naively costs **116.62 µs — 117% of the frame budget** — and the whole difference
//! is the scrim, priced at 3.41 ns a cell over 24 000. Two things fix it and **both are contracts on
//! the component library, not on this module**:
//!
//! 1. **The base pass draws into its own layer, with a damage-driven composite.** That is the
//!    engine's own model and [`Driver`](crate::Driver) already works this way — the prototype the
//!    number came from had shortcut it.
//! 2. **Every component draws its text before its padding.** A component that fills its rectangle
//!    and *then* draws into it writes the cells under its label **twice**, for an identical picture,
//!    every frame, for as long as it is on screen. The map measured **10 814 of 24 000 cells against
//!    0** on the dense screen.
//!
//! `examples/overlay_numbers.rs` measures all three configurations, so a regression says *which*
//! obligation broke rather than only that one did.
//!
//! **One half of the map's second obligation does not survive the shipped engine, and it is written
//! down here rather than quietly dropped.** The map also priced padding-before-text at the difference
//! between 1.19 µs and 87 µs. The engine's damage is a **per-row bitset**, so a second write inside a
//! range that is already marked adds *no damaged cell*: the scrim composites the same set either way,
//! and what the redundant fill costs is the redundant writes rather than redundant compositing. The
//! map's arm was measured against a prototype whose damage was not a bitset. The obligation stands —
//! a quarter of the frame's cells written for nothing is worth not doing — but **its detector is the
//! count, not a stopwatch**, and the count is what is gated.

use std::cell::RefCell;

use vitui_engine::{Mix, Rect};

use crate::ctx::Ctx;
use crate::id::Id;
use crate::theme::Theme;

// ── z bands ──────────────────────────────────────────────────────────────────────────────────────

/// The z bands overlays are sorted into.
///
/// An empty enum with associated constants rather than a module of loose `const`s, so that `Z::MENU`
/// is one name a reader can follow and nothing can construct a `Z`.
///
/// The base layer is at **0** and every band is above it. The gaps are wide enough that an
/// application can place its own layer between two of them without renumbering anything here.
pub enum Z {}

impl Z {
    /// The base layer, which is the one the base pass draws into.
    pub const BASE: i32 = 0;
    /// A dropdown, a menu, a completion list.
    pub const MENU: i32 = 1_000;
    /// A modal dialog, and the scrim that goes under it at `MODAL - 1`.
    pub const MODAL: i32 = 2_000;
    /// A tooltip, which is above everything because it explains everything.
    pub const TOOLTIP: i32 = 3_000;
    /// **How far a nested overlay sits above its parent's layer**, and it is a step rather than a
    /// band on purpose.
    ///
    /// A nested overlay's z counts from the layer it was requested *from*, not from its own kind's
    /// band — or a dropdown inside a modal, sorted into `MENU`, would land **below** the barrier
    /// that exists to protect it. See
    /// `overlay_tests::a_dropdown_inside_a_modal_draws_above_the_barrier`.
    pub const NESTED: i32 = 10;
}

// ── placement ────────────────────────────────────────────────────────────────────────────────────

/// Which side of its anchor an overlay is asked for.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Side {
    /// Under the anchor. What a dropdown asks for.
    #[default]
    Below,
    /// Over it.
    Above,
    /// To its right. What a submenu asks for.
    Right,
    /// To its left.
    Left,
}

impl Side {
    /// The side a flip goes to.
    pub const fn opposite(self) -> Side {
        match self {
            Side::Below => Side::Above,
            Side::Above => Side::Below,
            Side::Right => Side::Left,
            Side::Left => Side::Right,
        }
    }

    /// Whether this side stacks vertically, which is the axis the primary steps along.
    const fn is_vertical(self) -> bool {
        matches!(self, Side::Below | Side::Above)
    }
}

/// How an overlay lines up with its anchor **across** the side it was placed on.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Align {
    /// Left edges together, or top edges for a horizontal side.
    #[default]
    Start,
    /// Centres together.
    Center,
    /// Right edges together, or bottom edges.
    End,
}

/// Where an overlay's rectangle lands against its anchor.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Placement {
    /// Which side of the anchor.
    pub side: Side,
    /// How it lines up across that side.
    pub align: Align,
}

impl Placement {
    /// A placement.
    pub const fn new(side: Side, align: Align) -> Placement {
        Placement { side, align }
    }

    /// Under the anchor, left edges together. What a dropdown wants.
    pub const BELOW: Placement = Placement::new(Side::Below, Align::Start);
    /// To the right, top edges together. What a submenu wants.
    pub const RIGHT: Placement = Placement::new(Side::Right, Align::Start);
    /// Under the anchor, centred on it. What a tooltip wants.
    pub const UNDER: Placement = Placement::new(Side::Below, Align::Center);
}

/// Place `size` against `anchor` inside `bounds`, as four steps in one order.
///
/// Every rectangle is in the coordinates of the frame's root, and every number is an integer: there
/// are no floats anywhere in this function and no state between calls.
///
/// 1. **place** — the primary axis from [`Placement::side`], the cross axis from
///    [`Placement::align`].
/// 2. **flip** — to the opposite side, and **only when the opposite side has more room**. A tie
///    keeps the side that was asked for, which is what makes a dropdown that fits exactly stay
///    where the author put it instead of jumping on a one-cell resize.
/// 3. **shift** — slide along the *cross* axis until it is inside `bounds`.
/// 4. **clamp** — both axes, last, and **never a resize**. An overlay bigger than the screen hangs
///    off the far edge at its stated size rather than being silently shrunk to fit, because a
///    component that was told it has forty columns and got twelve draws a lie.
///
/// # Examples
///
/// ```
/// use vitui_engine::Rect;
/// use vitui_runtime::overlay::{Align, Placement, Side, place};
///
/// let screen = Rect::new(0, 0, 80, 24);
/// let anchor = Rect::new(10, 20, 8, 1);
/// // There are three rows under the anchor and twenty above it, so a six-row menu flips up.
/// let r = place(anchor, (8, 6), screen, Placement::new(Side::Below, Align::Start));
/// assert_eq!(r, Rect::new(10, 14, 8, 6));
/// ```
#[must_use]
pub fn place(anchor: Rect, size: (u16, u16), bounds: Rect, p: Placement) -> Rect {
    let (w, h) = (i32::from(size.0), i32::from(size.1));
    let vertical = p.side.is_vertical();

    // ── 1. place ────────────────────────────────────────────────────────────────────────────────
    // The room on each side of the anchor, inside the bounds, along the primary axis.
    let (before, after) = if vertical {
        (
            (anchor.y - bounds.y).max(0),
            (bounds.bottom() - anchor.bottom()).max(0),
        )
    } else {
        (
            (anchor.x - bounds.x).max(0),
            (bounds.right() - anchor.right()).max(0),
        )
    };
    let (asked, other) = match p.side {
        Side::Below | Side::Right => (after, before),
        Side::Above | Side::Left => (before, after),
    };

    // ── 2. flip ─────────────────────────────────────────────────────────────────────────────────
    // **`other > asked`, not `other >= asked`.** That single character is the tie rule.
    let need = if vertical { h } else { w };
    let side = if asked < need && other > asked {
        p.side.opposite()
    } else {
        p.side
    };

    let primary = match side {
        Side::Below => anchor.bottom(),
        Side::Above => anchor.y - h,
        Side::Right => anchor.right(),
        Side::Left => anchor.x - w,
    };

    // The cross axis, from the alignment. Integer division truncates toward zero, which for an
    // overlay wider than its anchor puts the extra half-cell on the right rather than the left —
    // stated because it is the kind of thing a reader would otherwise assume was rounding.
    let (cross_start, cross_span, cross_size) = if vertical {
        (anchor.x, i32::from(anchor.w), w)
    } else {
        (anchor.y, i32::from(anchor.h), h)
    };
    let mut cross = match p.align {
        Align::Start => cross_start,
        Align::Center => cross_start + (cross_span - cross_size) / 2,
        Align::End => cross_start + cross_span - cross_size,
    };

    // ── 3. shift ────────────────────────────────────────────────────────────────────────────────
    let (cross_lo, cross_len) = if vertical {
        (bounds.x, i32::from(bounds.w))
    } else {
        (bounds.y, i32::from(bounds.h))
    };
    cross = slide(cross, cross_size, cross_lo, cross_len);

    // ── 4. clamp ────────────────────────────────────────────────────────────────────────────────
    // Applied to both axes, so the order in this function is the order in the sentence. The cross
    // axis is already inside after the shift, so its clamp is an identity — kept rather than
    // special-cased, because a reader checking the four steps against the four steps should find
    // four of them.
    let (prim_lo, prim_len) = if vertical {
        (bounds.y, i32::from(bounds.h))
    } else {
        (bounds.x, i32::from(bounds.w))
    };
    let primary = slide(primary, need, prim_lo, prim_len);
    cross = slide(cross, cross_size, cross_lo, cross_len);

    if vertical {
        Rect::new(cross, primary, size.0, size.1)
    } else {
        Rect::new(primary, cross, size.0, size.1)
    }
}

/// Move `at` so that `[at, at + len)` sits inside `[lo, lo + span)`, **without changing `len`**.
///
/// The far edge first and the near edge second, so that something larger than the span ends up
/// flush with the near edge and hangs off the far one — which is the only arrangement in which the
/// caller's first row is on screen.
const fn slide(at: i32, len: i32, lo: i32, span: i32) -> i32 {
    let hi = lo + span;
    let at = if at + len > hi { hi - len } else { at };
    if at < lo { lo } else { at }
}

// ── the scrim ────────────────────────────────────────────────────────────────────────────────────

/// The darkening under a modal: an **operator** layer, because a terminal cell has no alpha.
///
/// It transforms what is already on screen rather than covering it, which is why it is proportional
/// to the *screen* and not to the overlay it belongs to — and therefore the expensive half of a
/// modal. See the module comment for the two obligations that make it affordable.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Scrim {
    mix: Mix,
}

impl Scrim {
    /// How far a scrim moves the page by default, out of [`Mix::FULL`]. **96 of 256**, which leaves
    /// the page legible underneath rather than blanking it.
    pub const DEFAULT_AMOUNT: u16 = 96;

    /// A scrim that **darkens**, which is the shape a dark page wants.
    ///
    /// # This is the one structurally dark-assuming thing on the runtime's map
    ///
    /// A shadow is a move toward black, and the engine ships [`Mix::lift`] beside [`Mix::darken`]
    /// precisely because the other direction exists. Nothing in the engine or in this crate makes
    /// *darker* the correct direction for a light page — a light page under a darkening scrim goes
    /// grey, which reads as dimmed, but so does a dark page under a lift, and the map recorded the
    /// question rather than settling it.
    ///
    /// **[`Theme::is_dark`] is the reader.** [`Scrim::for_theme`] is the only caller of it in this
    /// crate, and it is there so that a caller who does not want to think about the direction does
    /// not have to. A caller who *does* want to name a direction calls this or [`Scrim::lifting`].
    pub const fn shadow(amount: u16) -> Scrim {
        Scrim {
            mix: Mix::darken(amount),
        }
    }

    /// A scrim that **lifts**: the same mechanism, the other direction.
    pub const fn lifting(amount: u16) -> Scrim {
        Scrim {
            mix: Mix::lift(amount),
        }
    }

    /// The scrim a theme asks for, at [`DEFAULT_AMOUNT`](Scrim::DEFAULT_AMOUNT).
    ///
    /// **The one reader of [`Theme::is_dark`]**, which is what makes the direction a branch rather
    /// than a constant. See [`Scrim::shadow`] for what is settled here and what is not.
    #[must_use]
    pub const fn for_theme(theme: &Theme) -> Scrim {
        if theme.is_dark() {
            Scrim::shadow(Scrim::DEFAULT_AMOUNT)
        } else {
            Scrim::lifting(Scrim::DEFAULT_AMOUNT)
        }
    }

    /// The operator this scrim is, for the layer stack.
    pub(crate) const fn mix(self) -> Mix {
        self.mix
    }
}

// ── the request ──────────────────────────────────────────────────────────────────────────────────

/// What an overlay is asked for, beyond its owner and its anchor.
///
/// **There is no `Default`, and the missing one is the size.** A component's size is
/// stated or comes from a sizing function, because there is no measure pass — so a default size
/// would be a zero-area overlay that draws nothing, silently, which is the exact failure mode this
/// ticket exists to remove one instance of. [`OverlayOpts::sized`] is the base expression the
/// struct-update syntax in the spec's example uses:
///
/// ```
/// use vitui_runtime::overlay::{OverlayOpts, Placement, Z};
///
/// let opts = OverlayOpts {
///     z: Z::MENU,
///     placement: Placement::BELOW,
///     ..OverlayOpts::sized(20, 6)
/// };
/// assert_eq!(opts.size, (20, 6));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct OverlayOpts {
    /// The band. See [`Z`]. **Ignored for a nested overlay**, whose z counts from its parent's
    /// layer instead — see [`Z::NESTED`].
    pub z: i32,
    /// How big, in cells. There is no measure pass, so this is stated.
    pub size: (u16, u16),
    /// Where it lands against its anchor.
    pub placement: Placement,
    /// Whether the layer erases what is underneath it.
    ///
    /// `true` for a menu or a dialog; `false` for a layer with a transparent gutter, rounded
    /// corners, or anything drawn over a chart. The engine prices the difference at 4.1× for the
    /// skip test, so this is not a free choice.
    pub opaque: bool,
    /// The scrim under it, if it has one. `Some` is what makes an overlay a modal.
    pub scrim: Option<Scrim>,
}

impl OverlayOpts {
    /// A menu-band, opaque, below-and-start-aligned overlay of this size, with no scrim.
    #[must_use]
    pub const fn sized(w: u16, h: u16) -> OverlayOpts {
        OverlayOpts {
            z: Z::MENU,
            size: (w, h),
            placement: Placement::BELOW,
            opaque: true,
            scrim: None,
        }
    }

    /// A modal of this size: the [`Z::MODAL`] band, centred on its anchor, with the theme's scrim.
    ///
    /// The other two thirds of a modal are not here and cannot be: the caller opens a
    /// [`Trap`](crate::ScopeKind::Trap) for the keyboard and calls
    /// [`Ctx::modal_barrier_here`](crate::Ctx::modal_barrier_here) for the pointer, both from inside
    /// the body. A constructor that could do either would have to run during the draw.
    #[must_use]
    pub const fn modal(w: u16, h: u16, theme: &Theme) -> OverlayOpts {
        OverlayOpts {
            z: Z::MODAL,
            size: (w, h),
            placement: Placement::new(Side::Below, Align::Center),
            opaque: true,
            scrim: Some(Scrim::for_theme(theme)),
        }
    }
}

/// A queued overlay: everything the second pass needs, and the handle of the body beside it.
///
/// `Copy`, so the pass can lift one out of the round's `Vec` and still reach `&mut self` for the
/// screen and the frame. That is why the body is named by a `BodyAt` rather than carried here: a
/// `Box` is not `Copy`, and the request has to be liftable.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OverlayRequest {
    pub(crate) owner: Id,
    /// **In the coordinates of the frame's root**, translated at the request. A rectangle in the
    /// requesting context's own coordinates would mean nothing by the time the pass runs, which is
    /// the same reason [`Ctx::hover_style`](crate::Ctx::hover_style) translates.
    pub(crate) anchor: Rect,
    pub(crate) opts: OverlayOpts,
    /// The resolved band: `opts.z` at the base pass, the parent's z plus [`Z::NESTED`] inside
    /// another overlay.
    pub(crate) z: i32,
    /// Insertion order, which is the tie-break inside a band.
    pub(crate) seq: u32,
    pub(crate) body: BodyAt,
}

// ── the body queue ───────────────────────────────────────────────────────────────────────────────

/// An overlay body, boxed. **One allocation, and the whole of what storing a body costs.**
///
/// `FnMut` and not `FnOnce` because that is the request verb's bound: the pass calls a body at most
/// once, and the looser bound is the one that does not force a caller to give up a body it may want
/// to keep. See the module comment for why the `+ 'f` is what stops this living in a `Frame`.
type Body<'f> = Box<dyn FnMut(&mut Ctx<'f, '_>) + 'f>;

/// Which queued body a request names.
///
/// An index and not the body itself, so that [`OverlayRequest`] stays `Copy` and the pass can lift one
/// out of the round's `Vec` while it holds `&mut` of the screen and the frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct BodyAt(usize);

/// The overlay bodies queued during one frame call.
///
/// See the module comment for why this is a local of the frame call rather than a field of a `Frame`,
/// and for the allocation count that follows from it.
///
/// # A body is taken out to be run, never borrowed in place
///
/// [`Bodies::take`] moves the `Box` out of its slot, so the `RefCell` is borrowed across a push and a
/// take and never across a call. A body that requests another overlay while it is running — a
/// submenu, a tooltip over a menu item — therefore pushes into this same queue with nothing borrowed
/// and nothing to alias. That is what the arena needed two buffers for.
///
/// It is also what makes *dropped exactly once* structural rather than reviewed: a slot that has been
/// taken holds `None`, and everything still holding a `Box` is dropped with the queue.
pub(crate) struct Bodies<'f> {
    slots: RefCell<Vec<Option<Body<'f>>>>,
}

impl<'f> Bodies<'f> {
    /// An empty queue. **Allocates nothing**, which is what keeps a frame with no overlay at zero.
    pub(crate) const fn new() -> Bodies<'f> {
        Bodies {
            slots: RefCell::new(Vec::new()),
        }
    }

    /// Box a body and return the handle its request carries.
    pub(crate) fn push<F>(&self, body: F) -> BodyAt
    where
        F: FnMut(&mut Ctx<'f, '_>) + 'f,
    {
        let mut slots = self.slots.borrow_mut();
        let at = slots.len();
        slots.push(Some(Box::new(body)));
        BodyAt(at)
    }

    /// Take a body out to be run, or to be dropped without running.
    ///
    /// `None` for a handle whose body has already been taken. The pass cannot produce that — one
    /// request names one body and the queue is drained once — so it is a skip rather than a panic:
    /// there is no screen output a panic here would protect.
    pub(crate) fn take(&self, at: BodyAt) -> Option<Body<'f>> {
        self.slots.borrow_mut().get_mut(at.0)?.take()
    }

    /// How many bodies were boxed this frame. **The reported number, and one allocation each.**
    pub(crate) fn count(&self) -> usize {
        self.slots.borrow().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Placement, exhaustively: 28 800 cases.**
    ///
    /// Four sides × three alignments × a hundred anchor positions × twenty-four overlay sizes, over
    /// a fixed twenty-by-twelve screen. The domain is stated as a product so that the count is a
    /// property of the loop rather than a number somebody typed, and the count is asserted.
    ///
    /// Two invariants hold over every one of them: **the size never changes**, and an overlay that
    /// fits at all lands **entirely inside** the bounds.
    #[test]
    fn placement_is_exhaustive_over_twenty_eight_thousand_eight_hundred_cases() {
        let bounds = Rect::new(0, 0, 20, 12);
        let sides = [Side::Below, Side::Above, Side::Right, Side::Left];
        let aligns = [Align::Start, Align::Center, Align::End];
        let widths = [1u16, 3, 5, 7, 9, 11];
        let heights = [1u16, 2, 3, 4];

        let mut cases = 0u32;
        for side in sides {
            for align in aligns {
                for ax in 0..10i32 {
                    for ay in 0..10i32 {
                        let anchor = Rect::new(ax, ay, 3, 1);
                        for w in widths {
                            for h in heights {
                                let p = Placement::new(side, align);
                                let r = place(anchor, (w, h), bounds, p);
                                cases += 1;
                                assert_eq!(
                                    (r.w, r.h),
                                    (w, h),
                                    "clamping resized {anchor:?} {w}x{h} {p:?}"
                                );
                                if w <= bounds.w && h <= bounds.h {
                                    assert!(
                                        r.x >= bounds.x
                                            && r.y >= bounds.y
                                            && r.right() <= bounds.right()
                                            && r.bottom() <= bounds.bottom(),
                                        "{r:?} escaped {bounds:?} from {anchor:?} {p:?}"
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        assert_eq!(cases, 28_800, "the corpus is 4 x 3 x 10 x 10 x 6 x 4");
    }

    /// **A tie keeps the side that was asked for**, and that is `>` rather than `>=` in one line of
    /// [`place`].
    ///
    /// Six rows above the anchor and six below, and a six-row menu asked for below stays below. Give
    /// the other side one more row and it flips.
    #[test]
    fn a_tie_keeps_the_side_that_was_asked_for() {
        let bounds = Rect::new(0, 0, 20, 13);
        let anchor = Rect::new(0, 6, 3, 1);
        let below = place(anchor, (3, 6), bounds, Placement::BELOW);
        assert_eq!(below.y, 7, "six above and six below is a tie, so it stays");

        let lower = Rect::new(0, 7, 3, 1);
        let flipped = place(lower, (3, 6), bounds, Placement::BELOW);
        assert_eq!(flipped.y, 1, "seven above against five below flips it up");
    }

    /// **Clamping is last and never resizes.** An overlay taller than the screen keeps its height
    /// and hangs off the bottom, flush with the top.
    #[test]
    fn clamping_never_resizes() {
        let bounds = Rect::new(0, 0, 20, 12);
        let r = place(Rect::new(4, 4, 3, 1), (40, 40), bounds, Placement::BELOW);
        assert_eq!((r.w, r.h), (40, 40));
        assert_eq!(
            (r.x, r.y),
            (0, 0),
            "flush with the near edge, over the far one"
        );
    }

    /// The four steps are in one order, and the shift is the one that is easy to lose: an overlay
    /// aligned to the start of an anchor near the right edge slides left instead of flipping.
    #[test]
    fn the_shift_moves_across_and_the_flip_moves_along() {
        let bounds = Rect::new(0, 0, 20, 12);
        let anchor = Rect::new(17, 2, 3, 1);
        let r = place(anchor, (10, 3), bounds, Placement::BELOW);
        assert_eq!(r.y, 3, "still below: the primary axis had room");
        assert_eq!(r.x, 10, "and it slid left until it fitted");
    }

    /// Alignment across the side, all three of them, with the truncation stated.
    #[test]
    fn alignment_lines_up_across_the_side() {
        let bounds = Rect::new(0, 0, 40, 12);
        let anchor = Rect::new(10, 2, 4, 1);
        let at = |align| place(anchor, (7, 2), bounds, Placement::new(Side::Below, align)).x;
        assert_eq!(at(Align::Start), 10);
        // (4 - 7) / 2 truncates toward zero, so the extra cell is on the right.
        assert_eq!(at(Align::Center), 9);
        assert_eq!(at(Align::End), 7);
    }

    /// A horizontal side puts the primary on x and the cross on y, which is the whole of what
    /// `is_vertical` decides.
    #[test]
    fn a_submenu_goes_right_and_flips_left() {
        let bounds = Rect::new(0, 0, 20, 12);
        let r = place(Rect::new(2, 3, 4, 1), (6, 4), bounds, Placement::RIGHT);
        assert_eq!(r, Rect::new(6, 3, 6, 4));
        let squeezed = place(Rect::new(15, 3, 4, 1), (6, 4), bounds, Placement::RIGHT);
        assert_eq!(squeezed, Rect::new(9, 3, 6, 4), "flipped to the left");
    }

    /// **A body the pass never reaches is still dropped, exactly once.**
    ///
    /// This is the property the arena's drop thunk existed for, and the reason it survives the thunk:
    /// the mechanism changed from *the request carries a way to drop its body* to *the queue owns the
    /// box*, and a mechanism that changes is exactly when a gate earns its keep. Taken out and
    /// dropped, or left in a slot and dropped with the queue — one drop either way, and neither is a
    /// number anybody typed.
    #[test]
    fn a_body_is_dropped_exactly_once_whether_or_not_it_ran() {
        use std::cell::Cell;
        use std::rc::Rc;

        struct Counts(Rc<Cell<u32>>);
        impl Drop for Counts {
            fn drop(&mut self) {
                self.0.set(self.0.get() + 1);
            }
        }

        let drops = Rc::new(Cell::new(0));
        let taken = {
            let bodies = Bodies::new();
            let owned = Counts(Rc::clone(&drops));
            let at = bodies.push(move |_cx: &mut Ctx<'_, '_>| {
                let _ = &owned;
            });
            let left = Counts(Rc::clone(&drops));
            bodies.push(move |_cx: &mut Ctx<'_, '_>| {
                let _ = &left;
            });
            assert_eq!(
                drops.get(),
                0,
                "a queued body is not dropped by queueing it"
            );
            drop(bodies.take(at));
            assert_eq!(drops.get(), 1, "taking one out and dropping it is one drop");
            assert_eq!(bodies.count(), 2, "and both were boxed");
            at
        };
        assert_eq!(
            drops.get(),
            2,
            "the one the pass never reached went with the queue"
        );
        let _ = taken;
    }

    /// **One request names one body, and a handle answers once.**
    ///
    /// The slot holds `None` afterwards, which is what makes *dropped exactly once* a property of the
    /// container rather than of the pass being careful.
    #[test]
    fn a_handle_hands_its_body_out_once() {
        let bodies = Bodies::new();
        let state = (0u64, 0u64, 0u64, 0u64);
        let at = bodies.push(move |_cx: &mut Ctx<'_, '_>| {
            let _ = state;
        });
        assert!(bodies.take(at).is_some(), "the first take gets the body");
        assert!(bodies.take(at).is_none(), "and the second gets nothing");
    }

    /// `Z::NESTED` is a step above the parent, not a band of its own — which is the whole reason a
    /// dropdown inside a modal can be above the barrier.
    #[test]
    fn the_bands_are_ordered_and_nested_is_a_step() {
        const {
            assert!(Z::BASE < Z::MENU && Z::MENU < Z::MODAL && Z::MODAL < Z::TOOLTIP);
            assert!(Z::NESTED > 0);
            assert!(
                Z::MODAL + Z::NESTED < Z::TOOLTIP,
                "a nested overlay stays under the band above its parent's"
            );
        }
    }

    /// A scrim's direction is the theme's, and `Theme::is_dark` is what decides it.
    #[test]
    fn the_scrim_reads_the_theme_for_its_direction() {
        let dark = Theme::default();
        assert!(dark.is_dark());
        assert_eq!(
            Scrim::for_theme(&dark),
            Scrim::shadow(Scrim::DEFAULT_AMOUNT)
        );
        assert_ne!(
            Scrim::shadow(Scrim::DEFAULT_AMOUNT),
            Scrim::lifting(Scrim::DEFAULT_AMOUNT),
            "the two directions are two operators"
        );
    }
}
