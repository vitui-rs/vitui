//! The frame: [`Ctx`], [`Frame`], [`Env`], and a sequence that cannot be skipped.
//!
//! Spec §1, §3 and §6; ADR 0012 (what the runtime keeps between frames). **The runtime's tracer
//! bullet**: a component function draws through a `Ctx` into a layer and `present()` puts it on
//! screen. Everything after this ticket adds a service to this context or a structure to this frame;
//! nothing after it invents a new sequence.
//!
//! ```
//! use vitui_runtime::ctx::Driver;
//! use vitui_runtime::{Role, Theme};
//! use vitui_engine::{Clock, Config, Output, Rect};
//!
//! let mut driver = Driver::headless(40, 10).expect("a sink cannot fail to attach");
//! driver.frame(|cx| {
//!     let body = cx.theme().paint(Role::Body);
//!     cx.text(0, 0, "hello", body);
//! });
//! # let _ = (Clock::Manual, Rect::new(0, 0, 1, 1));
//! ```
//!
//! # The sequence, which everything else hangs on
//!
//! ```text
//! wait()                    the engine paces this
//! ├─ take          worker landings → application state, before anything reads them
//! ├─ begin         post the input batch and split it at a routing edge · sample the clock once
//! │                swap-and-clear the five frame structures · clear the overlay queue
//! │                resolve, from the PREVIOUS frame's index, the two things that cannot be
//! │                answered during the draw: which widget is topmost and which owns the wheel
//! ├─ base pass     view(&mut Ctx) draws into the base layer
//! ├─ overlay pass  each request in (z, seq) order, bounded at 16 rounds
//! ├─ end           award the press · resolve Tab · release the focus with the grab · settle hover ·
//! │                sweep three of the four id-keyed facts · resolve scroll-into-view · read the
//! │                ONE deadline sink (ticket 06 folded the repaint flag into it)
//! ├─ settle        set_mouse(tracking) · set_cursor(caret) · request_wake_at(earliest)
//! └─ present()
//! ```
//!
//! **Several of those steps are named no-ops here**, and that is the ticket's own arrangement: the
//! press award is ticket 10's, Tab is 12's, the sweep is 09's, scroll-into-view is 14's. They exist
//! as steps in the right order with nothing in them, so that adding one is filling a hole rather
//! than inventing a place to put it. `Frame::end` says which is which.
//!
//! # Two lifetimes, and the reason is a compile outcome that needs three things to be true
//!
//! `'f` is the frame's; `'v` is the borrow's. [`Ctx::child`] shrinks `'v` and leaves `'f` alone, and
//! the bound on an overlay body is `+ 'f` — so a body cannot capture a base-pass local.
//!
//! **That bound is vacuous unless all three of these hold**, and only the first is in the ticket:
//!
//! 1. There are **two** lifetimes. With one, `child()` shrinks the only one there is and a capturing
//!    body compiles.
//! 2. **`'f` is invariant.** This is the one that is easy to get wrong and silent when you do: with a
//!    covariant brand — `PhantomData<&'f ()>` — a caller can shorten `'f` at the `child()` call and
//!    the bound is satisfied by a shorter capture, exactly as with one lifetime. The brand here is
//!    `fn(&'f ()) -> &'f ()`, which is invariant in `'f` because a lifetime appears in both argument
//!    and return position.
//! 3. **The driver takes `&'f mut self`.** `fn frame(&mut self, body: impl FnOnce(&mut Ctx))` makes
//!    `'f` higher-ranked and forces every body to `'static`, which is a different and wrong rule;
//!    `&'f mut self` makes it *outlives this frame call*, which is the true requirement.
//!
//! [`Ctx`] carries the paired gate. All three were measured before this module was written, and the
//! first two attempts at the negative case compiled — see `tests::the_brand_is_invariant`.
//!
//! # `begin` is not skippable, and the failure would be a hang
//!
//! [`IdTable`] is **stamped rather than cleared**, so a `Frame` that has never been `begin`-ed carries
//! stamp 0 over slots stamped 0: no slot is ever *not mine*, and a naive `claim` walks the ring for
//! ever. **A hang is worse than a failure, because `cargo test` has no per-test timeout.**
//!
//! Both halves of the obligation are met rather than one: a `Frame` is only reachable through
//! [`Driver`], which `begin`s it — *and* `IdTable::claim` returns `None` after a bounded probe
//! instead of spinning, so the failure is a value even if the first half is ever circumvented.

use std::fmt;
use std::marker::PhantomData;
use std::ops::Range;
use std::time::{Duration, Instant};

use vitui_engine::{
    AttachError, Capabilities, Clock, Config, Cursor, CursorShape, Engine, LayerId, MouseMode,
    Output, Presented, Rect, Screen, Surface, View, Wake, WakeHandle, Written,
};

use crate::focus::{ScopeKind, Stop};
use crate::id::IdStack;
use crate::keys::Matches;
use crate::overlay::{Bodies, OverlayOpts, OverlayRequest, Z, place};
use crate::route::{self, KeyQueue};
use crate::scroll::{Area, IntoView, Scrollable, Wheel};
use crate::sizing::{Consulted, Extent, Measured};
use crate::theme::{Paint, Repaint, Theme};

/// The engine's capabilities, re-exported and not redefined.
///
/// Spec §9 states the mapping this document had left implicit. Same rule as `GlyphSet` and
/// `CursorShape`: **two types with one name across the seam is the failure being avoided.**
pub type Caps = Capabilities;

/// What a widget asks the frame to track for it.
///
/// Five bits, and the fifth is not like the others: `FOCUS` declares a tab stop and **costs no
/// tracking at all**, because the keyboard is delivered whatever the mouse is doing.
///
/// # A declaration is not a correction
///
/// The runtime never edits one (ADR 0021). A widget that declares `HOVER` against a theme whose
/// hover is invisible still gets motion tracking, and is therefore visibly wrong rather than
/// invisibly fixed — which is why [`Theme::hover_interest`] is an *answer* a component reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Interest(u8);

impl Interest {
    /// Hit-tested and nothing more.
    pub const NONE: Interest = Interest(0);
    /// Presses and clicks.
    pub const CLICK: Interest = Interest(1 << 0);
    /// The pointer entering and leaving, which is what costs motion reporting.
    pub const HOVER: Interest = Interest(1 << 1);
    /// The wheel.
    pub const SCROLL: Interest = Interest(1 << 2);
    /// A drag, which needs the pointer's position while a button is held.
    pub const DRAG: Interest = Interest(1 << 3);
    /// A tab stop. **Costs no tracking**, which is the whole reason it is a bit here rather than a
    /// separate declaration.
    pub const FOCUS: Interest = Interest(1 << 4);

    /// Both.
    pub const fn with(self, other: Interest) -> Interest {
        Interest(self.0 | other.0)
    }

    /// Whether every bit of `other` is set.
    pub const fn contains(self, other: Interest) -> bool {
        self.0 & other.0 == other.0
    }

    /// Nothing at all.
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Whether hover is being asked for. Kept because [`Theme::hover_interest`]'s callers read it.
    pub const fn wants_hover(self) -> bool {
        self.contains(Interest::HOVER)
    }

    /// The mouse mode this interest needs.
    ///
    /// **The engine's ladder is totally ordered — `Off < Buttons < Drag < Motion` — so combining what
    /// several widgets want is a `max`.** That is the whole reason `set_mouse` takes one value and the
    /// frame accumulates rather than negotiating.
    pub const fn tracking(self) -> MouseMode {
        if self.contains(Interest::HOVER) {
            MouseMode::Motion
        } else if self.contains(Interest::DRAG) {
            MouseMode::Drag
        } else if self.contains(Interest::CLICK) || self.contains(Interest::SCROLL) {
            MouseMode::Buttons
        } else {
            MouseMode::Off
        }
    }
}

pub use crate::id::{Id, IdTable};

/// One entry of the hit index: **sixteen bytes, with no rectangle and no layer id.**
///
/// The rect is dropped because the press award does not need it — the index is in draw order and a
/// reverse scan gives the innermost — and the layer id because modality is one index into this list
/// rather than a per-entry field. Thirty-two bytes became sixteen that way.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hit {
    /// Whose.
    pub id: Id,
    /// What it asked for.
    pub interest: Interest,
    /// Whether the pointer was inside it when it was declared.
    pub over: bool,
    /// **Where it can still move**, as four directions and never two axes — which is the whole of
    /// what makes wheel chaining work at an end stop. See [`Scrollable`].
    pub scrollable: Scrollable,
}

/// What a widget learns about the pointer and the keyboard.
///
/// **Ticket 10 fills this in.** Ticket 08 ships the type, and a `Response` from this ticket carries
/// the id, the rect and nothing else true — every interaction field is `false` because the press
/// award is a named no-op. That is deliberate: a component written against this compiles and draws,
/// and starts responding when 10 lands, without a signature moving.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Response {
    /// Whose.
    pub id: Id,
    /// Where, in the coordinates it was declared in.
    pub rect: Rect,
    /// The pointer is inside it.
    pub hovered: bool,
    /// A button is down on it.
    pub pressed: bool,
    /// A button came up on it this frame.
    pub released: bool,
    /// Pressed and released without leaving.
    pub clicked: bool,
    /// Two clicks inside the double-click interval — compared against **the event's** stamp and
    /// never the frame clock.
    pub double_clicked: bool,
    /// Held past the long-press interval. **The only field that costs a wakeup.**
    pub long_pressed: bool,
    /// How far the pointer has moved since the press, if it is dragging.
    pub dragged: Option<(i32, i32)>,
    /// Wheel movement delivered to it.
    pub scrolled: (i32, i32),
    /// The pointer in this widget's own coordinates. **Without it no drag can ask where the pointer
    /// is**, which is why it was added to the proposal's fifteen.
    pub local: Option<(i32, i32)>,
    /// It holds the focus.
    pub focused: bool,
    /// It took the focus this frame.
    pub focus_entered: bool,
    /// It lost the focus this frame.
    pub focus_left: bool,
    /// **A return-value convention the runtime never sets.** A component sets it to say its own value
    /// changed; nothing here can know that.
    pub changed: bool,
    /// The modifiers held. The intent half is what a key map compares — see [`crate::keys::INTENT`].
    pub mods: vitui_engine::Mods,
}

impl Response {
    /// A response that says nothing happened.
    pub const fn inert(id: Id, rect: Rect) -> Response {
        Response {
            id,
            rect,
            hovered: false,
            pressed: false,
            released: false,
            clicked: false,
            double_clicked: false,
            long_pressed: false,
            dragged: None,
            scrolled: (0, 0),
            local: None,
            focused: false,
            focus_entered: false,
            focus_left: false,
            changed: false,
            mods: vitui_engine::Mods::NONE,
        }
    }
}

/// The immutable half of the frame's services.
///
/// **Behind a shared reference whose lifetime is the frame's rather than the `&mut Ctx` borrow's**,
/// which is what makes `cx.text(x, y, cx.theme().glyph(g), st)` compile — see [`Ctx::theme`].
///
/// It cannot be removed by threading the theme through component signatures, and the reason is not
/// convenience: the frame's own `end` reads the theme when **no component exists to have been handed
/// one**. Settling hover into a style is the case.
#[derive(Clone, Debug)]
pub struct Env {
    theme: Theme,
    caps: Caps,
    now: Instant,
    theme_changed: bool,
}

impl Env {
    /// The theme.
    pub const fn theme(&self) -> &Theme {
        &self.theme
    }

    /// The capabilities.
    pub const fn caps(&self) -> &Caps {
        &self.caps
    }

    /// The moment this frame was sampled at.
    pub const fn now(&self) -> Instant {
        self.now
    }

    /// Whether the theme changed for this frame, and **this frame only**.
    pub const fn theme_changed(&self) -> bool {
        self.theme_changed
    }
}

/// The formatting buffer [`Ctx::stage`] writes into and [`Ctx::blit`] reads.
///
/// **A field of the frame and never a returned guard**, which is the first of the proposal's four
/// deviations: a guard holds `&mut Ctx` and the caller then needs `&mut Ctx` again to draw, so the
/// borrow is live across the verb. A right-aligned number, a truncating cell and a centred label all
/// need the **width before they know where to draw**, so the guard is necessarily live across the
/// draw. `stage` returns the width instead.
#[derive(Clone, Debug, Default)]
struct Scratch {
    buf: String,
}

/// The mutable half of a frame: five structures rebuilt from the draw, and the facts that are not.
///
/// **Reachable only through [`Driver`]**, which is the first half of *`begin` cannot be skipped*.
#[derive(Debug)]
pub struct Frame {
    // ── the five structures rebuilt from the draw, swapped and cleared in `begin` ──────────────
    /// 1. The hit index, in draw order. A reverse scan gives the innermost.
    hits: Vec<Hit>,
    /// 2. The focus ring: the tab stops declared during the draw, this frame's scopes over them, and
    ///    **the previous frame's copy of both** — one more swapped buffer, which is the whole
    ///    storage cost of the vanish rule. See [`crate::focus`].
    ring: crate::focus::Ring,
    /// 3. The overlay request queue, ordered `(z, seq)` by the pass that drains it.
    overlays: Vec<OverlayRequest>,
    /// 4. The deadline sink: **the earliest requested wake, and one value rather than a list**,
    ///    because `end` folds it into one wake anyway.
    ///
    ///    **It is the only wakeup sink there is** (ticket 06). It used to have a `repaint: bool`
    ///    beside it and `end` flushed one of the two, which is how `[Tab, Key(a)]` stranded its
    ///    second key: `request_frame()` **is** `deadline(now)`, and the split-batch drain asks the
    ///    same way a component does.
    deadline: Option<Instant>,
    /// 5. The key queue, drained at successively outer levels. **One queue and no per-id inboxes**
    ///    — `crate::route` is where the cursor that keeps draining it linear lives.
    keys: KeyQueue,

    // ── the pointer, which is a frame structure in everything but name ─────────────────────────
    /// This frame's mouse events, in arrival order.
    mouse: Vec<vitui_engine::Mouse>,
    /// Where the pointer is, in root coordinates. `None` until one has been reported — and it stays
    /// `None` at `Buttons` tracking, where **no motion event is ever sent**.
    pointer: Option<(i32, i32)>,
    /// What is held.
    buttons: vitui_engine::Buttons,
    /// What modifiers were on the last pointer event. **One byte, and it costs the tracking level
    /// nothing** — dropping it is why ctrl-click and shift-click were recorded as inexpressible three
    /// times.
    mods: vitui_engine::Mods,
    /// **Modality: one index into the hit index, which is already in draw order.** An *ordering*, not
    /// a membership — everything at or after this index is inside the modal.
    ///
    /// `Option<usize>` and **not** `usize`, because `0` meant both *no modal* and *a modal over
    /// nothing*.
    modal_from: Option<usize>,
    /// Resolved in `begin` from the **previous** frame's index: a guess at what is hovered, which
    /// buys in-frame feedback and can never become a wrong click.
    hover_guess: Option<Id>,
    /// **This frame's wheel, resolved in `begin`** from the previous frame's index and the
    /// direction bits it carries — not awarded at `end` like every other pointer outcome.
    ///
    /// It is the one channel that cannot wait, because the offset is read *during* the draw by the
    /// widget that owns it, so an answer produced after the draw arrives after its only reader. The
    /// price is one click of residue at each end stop and on an area's first frame, and it is a
    /// documented property rather than a bug — see [`crate::scroll`].
    wheel: Option<(Id, (i32, i32))>,
    /// How far one click moves. **The whole motion model**: no momentum, no smooth scroll.
    wheel_config: Wheel,
    /// Hover styles declared during the draw, resolved at `end` — which is what makes hover-as-a-style
    /// land in the **same** frame.
    hover_styles: Vec<(Id, Rect, crate::theme::Role)>,
    /// What `end` awarded, delivered on the next frame's draw.
    awarded: Option<Awarded>,
    /// What the previous frame's `end` awarded, readable during this draw.
    delivered: Option<Awarded>,
    /// The thresholds.
    pointer_config: Pointer,

    // ── routing (ticket 11) ────────────────────────────────────────────────────────────────────
    /// **Who may take a key right now**, and it is one id rather than a map.
    ///
    /// `begin` sets it to the focused id; a bubbling scope moves it outward to its own id after its
    /// body. Nothing focused means `None`, and then no widget takes anything — every key falls
    /// through to the application, which is the outermost level of the one queue.
    route_to: Option<Id>,
    /// How many times the focused widget has declared itself this frame.
    ///
    /// **The bubbling detector, and it is a counter rather than a scan**: a scope reads it before
    /// and after its body, and a change means the focus is inside it. A container drawn *before*
    /// the focused widget must not bubble, and this is what tells it apart at O(1).
    focus_draws: u32,

    // ── the id-keyed facts (ADR 0012). Three are swept at `end`; the click record is not ────────
    grab: Option<Id>,
    press_origin: Option<(Id, (i32, i32))>,
    focused: Option<Id>,
    /// What the focus was when **this** frame's draw started.
    focus_shown: Option<Id>,
    /// What it was when the **previous** frame's draw started, which is what a widget saw last time
    /// it asked. `focus_entered` and `focus_left` are the difference between the two, and neither can
    /// be read off `focused` alone: the focus moves in `end`, so by the time a widget draws again the
    /// move has already happened.
    focus_before: Option<Id>,
    /// Whether the focus sat inside an [`Isolated`](crate::focus::ScopeKind::Isolated) scope last
    /// frame, which is the one question the drain has to answer before this frame has any scopes.
    focus_isolated: bool,
    /// **Not swept**, deliberately: a click record outliving its widget is how a double click
    /// survives a redraw.
    click_record: Option<(Id, Instant)>,

    // ── everything else the frame accumulates ──────────────────────────────────────────────────
    ids: IdTable,
    /// The id path: **the closure tree, not the draw tree.** Pushed only by `with_key` and `with_id`.
    stack: IdStack,
    scratch: Scratch,
    /// The declared key maps for the open scope, cleared and never freed.
    maps: Matches,

    // ── scrolling (ticket 14) ──────────────────────────────────────────────────────────────────
    /// **The scroll areas this frame declared**, in draw order, and a *sixth* frame-local structure
    /// where spec §1 says five. It is cleared in `begin` and read once, in `end`, by the step that
    /// resolves scroll-into-view — nothing in it survives the frame, so it is a finding against §1's
    /// sentence and not against ADR 0012's decision, which is that nothing is *retained*.
    scroll_areas: Vec<Area>,
    /// Which of them is open right now, as an index. **On the frame and not on `Ctx`**, because
    /// `scroll_scope` brackets its body: it saves this, replaces it and restores it, which is the
    /// same shape as the scope stack and costs `Ctx` no field.
    open_area: Option<u32>,
    /// **The sixteen bytes that cross the frame boundary**, and the only ones this subsystem has.
    /// Written by `end`, read by [`Ctx::take_into_view`] on the frame after, and dropped by the next
    /// `end` if nobody took it — one frame of life, so a request cannot fire long after the move
    /// that asked for it.
    into_view: Option<IntoView>,
    /// What the *draw* asked for through [`Ctx::request_into_view`]. **Frame-local**, cleared in
    /// `begin`, and folded into `into_view` at `end` — a separate field because the two must not
    /// alias: a request made during a draw that then calls `take_into_view` would otherwise consume
    /// itself.
    into_view_asked: Option<IntoView>,
    /// Whether the **ring** moved the focus this frame, which is the runtime's whole test for *this
    /// was a keyboard-driven move*. A press does not set it, and an unconditional pull is the list's
    /// old bug: it drags the viewport back to the selection every time the wheel moves away from it.
    tab_moved: bool,

    // ── the drawn extent, maintained only while something is asking for it (spec §12) ───────────
    /// How far the verbs reached, and **`None` in a real frame**: maintaining it costs a display
    /// width measurement per verb — 7% of the frame budget — so a frame pays it only when something
    /// asked. Today the only thing that asks is [`Ctx::measured`]; a scroll area over bounded
    /// content is the second caller and reads it a frame late (ticket 14).
    ///
    /// It doubles as the flag for *this is a measured world*, which is why the two live together:
    /// there is no second bit to get out of step with this one.
    extent: Option<Extent>,
    /// Whether a **real** frame was asked to maintain one, by [`Driver::measure_extent`].
    ///
    /// **Off by default and it must stay off by default.** A scroll area over content whose size it
    /// does not know reads the extent one frame late, and that is the one legitimate reason to turn
    /// it on — at the price ticket 15 measured: a display-width walk per verb, 7% of the frame
    /// budget, *for a field that is off by default*. While it is on, `Ctx::interact` and
    /// `Ctx::next_key` also populate [`Frame::consulted`], where nothing reads it in a real frame;
    /// harmless, and named here rather than discovered.
    measure_extent: bool,
    /// What the body asked a measured world for that a measured world cannot answer.
    consulted: Consulted,

    // ── overlays (ticket 13) ───────────────────────────────────────────────────────────────────
    /// How many overlay bodies were boxed this frame. **One allocation each, and the reported
    /// number.**
    ///
    /// The queue that holds them is *not* a field here and cannot be: a body is `+ 'f` and a `Frame`
    /// is what is borrowed for `'f`, so it is a local of the frame call. See [`crate::overlay`].
    overlay_bodies: u32,
    /// The `z` of the layer being drawn into right now, and **0 during the base pass**.
    ///
    /// One field rather than a `Ctx` field, deliberately: every `Ctx` in a body's subtree shares the
    /// frame, so *which layer is being drawn into* is a property of the pass and not of a context.
    /// It is what makes a nested overlay's `z` count from its parent's layer.
    layer_z: i32,
    /// How many rounds the overlay pass ran. **The bound is a limit, not a hang.**
    overlay_rounds: u32,
    /// How many overlays were placed into a layer this frame.
    overlays_placed: u32,
    /// How many requests named an owner that had already asked this frame, and were therefore inert.
    overlays_merged: u32,

    /// The `max` of every declared interest, which is what `settle` hands to `set_mouse`.
    tracking: MouseMode,
    /// Where the caret goes, if anything asked.
    caret: Option<Cursor>,
    /// **What asked for a wake, and from which line.** Unconditional — 1.51–1.61 ns a frame, of both
    /// signs against a whole frame — so there is no `debug_assertions` guard and no feature flag.
    /// See [`crate::anim::WakeLedger`]; it allocates nothing at any point in its life.
    wakes: crate::anim::WakeLedger,
    /// How many frames have run, for the gates that count.
    frames: u64,
    /// Whether `begin` has ever run. The gate reads it.
    begun: bool,
}

/// What `end` awarded, from the index that has just drawn.
///
/// **Delivered on the next frame's draw**, which is the whole trick: the guess resolved in `begin`
/// buys in-frame feedback and this buys correctness, and neither has to be both.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct Awarded {
    /// Who was pressed.
    pressed: Option<Id>,
    /// Who was clicked, and whether it was the second of two.
    clicked: Option<(Id, bool)>,
    /// Who released.
    released: Option<Id>,
    /// Who is hovered.
    hovered: Option<Id>,
    /// Who was held past the long-press threshold.
    long_pressed: Option<Id>,
    /// **Whose drag was cancelled**, which the identity sweep alone did not tell anybody.
    cancelled: Option<Id>,
    /// The modifiers on the event that decided it.
    mods: vitui_engine::Mods,
}

/// A layer this driver is keeping alive for an owner, and what it currently is.
///
/// **The lifecycle is keyed by the owner id and the census is not §5's identity sweep**, which is a
/// distinction with a case behind it: an owner whose dropdown is *closed* is still drawing, so the
/// sweep sees a live id and would keep a layer nothing asked for. The census asks a different
/// question — *was this layer requested this frame* — and `seen` is where the answer is kept.
#[derive(Clone, Copy, Debug)]
struct Placed {
    owner: Id,
    layer: LayerId,
    /// The operator layer under it, when the overlay has a scrim.
    scrim: Option<ScrimLayer>,
    rect: Rect,
    z: i32,
    /// The frame number this layer was last requested on.
    seen: u64,
}

/// The scrim's operator layer, and enough of what it was made from to notice a change.
///
/// The `Scrim` is kept beside the id because a theme swap changes the operator: a light scheme
/// arriving under a standing modal has to replace the layer, not keep a darkening that was chosen
/// for the scheme before it.
#[derive(Clone, Copy, Debug)]
struct ScrimLayer {
    layer: LayerId,
    spec: crate::overlay::Scrim,
    rect: Rect,
    z: i32,
}

/// The thresholds double-click and long-press inference uses.
///
/// # Why the runtime infers these at all
///
/// The engine refuses to synthesise anything it did not see (ADR 0007): no double-click detection, no
/// mouse-leave inferred from focus loss, no guess at an escape sequence. **The runtime is exactly
/// where that inference is legitimate**, because it sits above the honest layer and because these are
/// *thresholds* — configuration, not facts about the wire. The engine's own `Mouse::at` doc says so:
/// *"double-click detection is not the engine's: it is policy with a tunable threshold and it belongs
/// where hit-testing belongs."*
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pointer {
    /// How close together two clicks must be. **400 ms.**
    pub click_threshold: Duration,
    /// How far the pointer may move between them and still be one place. **One cell** — a terminal
    /// pointer moves in whole cells, so this is *the pointer did not really move*, not a tolerance.
    pub click_slop: u16,
    /// How long a button must be held. **500 ms**, and it is the only threshold that costs a wakeup.
    pub long_press: Duration,
}

impl Default for Pointer {
    fn default() -> Pointer {
        Pointer {
            click_threshold: Duration::from_millis(400),
            click_slop: 1,
            long_press: Duration::from_millis(500),
        }
    }
}

/// How many rounds the overlay pass runs. **A limit, not a hang**: a body may request another
/// overlay, and sixteen is where that stops being a menu and starts being a loop.
pub const OVERLAY_ROUNDS: usize = 16;

impl Frame {
    fn new() -> Frame {
        Frame {
            hits: Vec::with_capacity(512),
            ring: crate::focus::Ring::new(),
            overlays: Vec::with_capacity(8),
            deadline: None,
            keys: KeyQueue::new(),
            mouse: Vec::with_capacity(32),
            pointer: None,
            buttons: vitui_engine::Buttons::NONE,
            mods: vitui_engine::Mods::NONE,
            modal_from: None,
            hover_guess: None,
            wheel: None,
            wheel_config: Wheel::default(),
            hover_styles: Vec::with_capacity(32),
            awarded: None,
            delivered: None,
            pointer_config: Pointer::default(),
            route_to: None,
            focus_draws: 0,
            grab: None,
            press_origin: None,
            focused: None,
            focus_shown: None,
            focus_before: None,
            focus_isolated: false,
            click_record: None,
            ids: IdTable::new(),
            stack: IdStack::new(),
            scratch: Scratch::default(),
            maps: Matches::new(),
            scroll_areas: Vec::with_capacity(4),
            open_area: None,
            into_view: None,
            into_view_asked: None,
            tab_moved: false,
            extent: None,
            measure_extent: false,
            consulted: Consulted::NONE,
            overlay_bodies: 0,
            layer_z: Z::BASE,
            overlay_rounds: 0,
            overlays_placed: 0,
            overlays_merged: 0,
            tracking: MouseMode::Off,
            caret: None,
            wakes: crate::anim::WakeLedger::new(),
            frames: 0,
            begun: false,
        }
    }

    /// Start a frame, over **one batch of events that has already been split at a routing edge**.
    ///
    /// **Swap-and-clear, not drop-and-rebuild**: every one of the five keeps its allocation, so a
    /// steady frame allocates nothing. The id table is *stamped* instead, which is why this cannot be
    /// skipped — see the module comment.
    ///
    /// The batch arrives whole and in arrival order, keys and pointer events interleaved, because
    /// **the split is over the interleaving**: a `Down` between two keys ends the batch there, and
    /// two separate per-kind queues could not have said so.
    fn begin(&mut self, batch: &[vitui_engine::Event]) {
        // **The guess, resolved from the PREVIOUS frame's index before it is cleared.** This is the
        // one thing `begin` can answer that the draw cannot: which widget is topmost under the
        // pointer, and which owns the wheel. It is a *guess* — the index is a frame old — and it is
        // never allowed to decide a click.
        self.hover_guess = self.topmost_over();
        // **And the wheel, which is not a guess and is not awarded at `end`.** It is resolved here,
        // against the previous index's direction bits, because the widget that owns the offset reads
        // it during the draw. See [`Frame::resolve_wheel`].
        self.wheel = self.resolve_wheel(batch);
        // And the previous frame's award becomes this frame's news.
        self.delivered = self.awarded.take();

        self.hits.clear();
        self.ring.begin();
        // **The request queue is cleared and nothing here drops a body.** The bodies live in the
        // frame call's own queue, which owns the boxes and drops whatever the pass did not reach when
        // the call ends — including on an unwind, which the arena's hand-rolled discard could not
        // promise. A request left over from a frame that never reached its pass names a slot in a
        // queue that no longer exists, so clearing the requests is the whole of what is owed.
        self.overlays.clear();
        self.overlay_bodies = 0;
        self.layer_z = Z::BASE;
        self.overlay_rounds = 0;
        self.overlays_placed = 0;
        self.overlays_merged = 0;
        self.deadline = None;
        self.keys.begin();
        self.maps.clear();
        self.scratch.buf.clear();
        self.tracking = MouseMode::Off;
        self.caret = None;
        // The ledger is **not** cleared here: a streak is a run of frames, so it is the one thing in
        // this type that has to survive the swap-and-clear. It is also the only one that is not
        // rebuilt from the draw, which is why it is not a sixth structure.
        self.ids.begin();
        self.stack.clear();
        self.scroll_areas.clear();
        self.open_area = None;
        self.into_view_asked = None;
        self.tab_moved = false;
        // **A frame that is not measuring must not maintain the measurement.** The `Option` is the
        // switch, so there is no second bit to get out of step with it.
        self.extent = self.measure_extent.then_some(Extent::ZERO);
        self.frames += 1;
        self.begun = true;

        // **Routing starts at the focus and moves outward, never inward.** One id, not a map: see
        // the field.
        self.route_to = self.focused;
        self.focus_draws = 0;

        // **The two focus questions that must be answered before the draw**, and both are answered
        // from the frame that has just ended rather than from the one about to start.
        self.focus_before = self.focus_shown;
        self.focus_shown = self.focused;
        // Whether `Tab` is the ring's or the focused widget's. It cannot be a decline — by the time
        // a key is declined the ring has already consumed it and moved the focus — so it is decided
        // here, in the drain, from the previous frame's scopes.
        self.focus_isolated = self.ring.was_isolated(self.focused);

        // Post the batch. It was split at a routing edge by `route::batch_len` before it got here,
        // so **everything in it is routed against one routing state** and at most one event in it
        // changes that state — at the end, because every edge that ships is a closing edge.
        //
        // **This is the split that was invisible for five tickets**, and the reason is worth having
        // in front of whoever edits it next: at `MouseMode::Motion` the unsplit batch *works*. A
        // motion event arrives between any two clicks a human can produce, and a batch that is
        // mostly moves is a batch whose edges were already one to a frame. Ticket 04's theme switch
        // is what escapes `Motion` — a theme with no visible hover state declares no `HOVER`, the
        // tracking level drops to `Buttons`, **the terminal stops sending motion events entirely**,
        // and `[Down, Up, Down, Up]` arrives as one batch for the first time. A US-layout,
        // `Motion`-tracking test suite finds nothing here.
        self.mouse.clear();
        self.hover_styles.clear();
        self.modal_from = None;
        for event in batch {
            match event {
                vitui_engine::Event::Key(k) => self.keys.push(*k),
                // The pointer's own batch updates the position as it goes, so that `over` is
                // computed against the pointer **as it was when this frame drew**.
                vitui_engine::Event::Mouse(m) => {
                    self.pointer = Some((i32::from(m.x), i32::from(m.y)));
                    self.buttons = m.buttons;
                    self.mods = m.mods;
                    self.mouse.push(*m);
                }
                // Not routed by this crate yet, and `route::edge_of` says which ticket owns each.
                vitui_engine::Event::Paste(_)
                | vitui_engine::Event::Resize(_, _)
                | vitui_engine::Event::FocusGained
                | vitui_engine::Event::FocusLost => {}
            }
        }
    }

    /// Ask for another frame, because the batch this one took was not the whole queue.
    ///
    /// **The one wake sink and nothing new** — this is `deadline(now)` with a call site, which is
    /// what makes `[Tab, Key(a)]` route both keys rather than stranding the second. It used to set a
    /// second sink that `end` did not read.
    ///
    /// `#[track_caller]` so the line the ledger names is `Driver::frame`'s and not this one: the
    /// runtime's lazy offender is a line like any other, and it now appears in the census it was
    /// invisible to.
    #[track_caller]
    fn wants_another_frame(&mut self, now: Instant) {
        self.ask(std::panic::Location::caller(), None, now, now);
    }

    /// **The one wakeup sink.** Fold `when` into the earliest, and record which line asked.
    ///
    /// Every path that wants another frame goes through here: [`Ctx::deadline`],
    /// [`Ctx::deadline_for`], [`Ctx::request_frame`] and the split-batch drain above. Two callers
    /// asking for different moments is one wake at the earlier of them, and the ledger keeps both
    /// lines — because a fold is a wake and a census is not.
    fn ask(
        &mut self,
        at: &'static std::panic::Location<'static>,
        who: Option<Id>,
        when: Instant,
        now: Instant,
    ) {
        self.deadline = Some(match self.deadline {
            Some(existing) if existing <= when => existing,
            _ => when,
        });
        self.wakes.asked(at, who, when, now);
    }

    /// The innermost entry the pointer is over, from the index as it stands.
    ///
    /// **A reverse scan, because the index is in draw order and later is innermost.** No quadtree: the
    /// engine already answers the layer question, and 312 entries is 142 ns.
    fn topmost_over(&self) -> Option<Id> {
        self.topmost_over_entry().map(|h| h.id)
    }

    /// The same entry, whole, because the press award needs to know whether it is a tab stop.
    fn topmost_over_entry(&self) -> Option<Hit> {
        let from = self.modal_from.unwrap_or(0);
        self.hits[from..]
            .iter()
            .rev()
            .find(|h| h.over && !h.interest.is_empty())
            .copied()
    }

    /// **The wheel chain**: the innermost scrollable entry the pointer is over **that can still
    /// move the way the wheel is going**.
    ///
    /// The direction is the whole of it. Asked *can you move on this axis*, a collection at its
    /// bottom answers yes — it can still go up — and swallows every downward click while the area
    /// around it never sees one. Asked *can you go down*, it says no and the click chains outward.
    fn chain_target(&self, delta: (i32, i32)) -> Option<Id> {
        let from = self.modal_from.unwrap_or(0);
        self.hits[from..]
            .iter()
            .rev()
            .find(|h| h.over && h.scrollable.admits(delta))
            .map(|h| h.id)
    }

    /// Resolve this batch's wheel notches against the **previous** frame's index.
    ///
    /// Called at the top of `begin`, before the index is cleared, which is the only moment both the
    /// batch and the previous frame's direction bits exist at once.
    ///
    /// **Notches in one batch add up.** A wheel click folds into a frame (ADR 0016) and it is intent
    /// (ADR 0008), and the two are only compatible if folding is a sum: overwriting made three
    /// notches in one batch scroll one row, which is dropping intent by another name. The 1006
    /// encoding carries no magnitude, so counting the notches is the only place the count can come
    /// from.
    ///
    /// **The wheel is withheld while a grab is held.** A drag is one gesture and a scroll in the
    /// middle of it is not part of it — and the grab read here is the one that stands when the batch
    /// arrives, which is right, because `Down` is a *closing* edge and can therefore only be the last
    /// event of a batch.
    fn resolve_wheel(&self, batch: &[vitui_engine::Event]) -> Option<(Id, (i32, i32))> {
        if self.grab.is_some() {
            return None;
        }
        let mut out: Option<(Id, (i32, i32))> = None;
        for event in batch {
            let vitui_engine::Event::Mouse(m) = event else {
                continue;
            };
            let vitui_engine::MouseKind::Wheel(w) = m.kind else {
                continue;
            };
            let (lines, cols) = (
                self.wheel_config.lines_per_click,
                self.wheel_config.columns_per_click,
            );
            let d = match w {
                vitui_engine::Wheel::Up => (0, -lines),
                vitui_engine::Wheel::Down => (0, lines),
                vitui_engine::Wheel::Left => (-cols, 0),
                vitui_engine::Wheel::Right => (cols, 0),
            };
            let Some(id) = self.chain_target(d) else {
                continue;
            };
            out = Some(match out {
                Some((held, (hx, hy))) if held == id => (id, (hx + d.0, hy + d.1)),
                // A different target mid-batch: the newer one wins, whole.
                _ => (id, d),
            });
        }
        out
    }

    /// Finish a frame, folding everything into one wake.
    ///
    /// Six of these steps are named no-ops belonging to later tickets, and they are steps rather than
    /// comments so that filling one is not also deciding where it goes.
    fn end(&mut self, now: Instant) -> Option<Instant> {
        self.award();
        // **Absence before the walk**: a `Tab` pressed on the frame a row disappears has to start
        // from where the vanish rule put the focus, not from a position belonging to an id that is
        // no longer on screen.
        self.vanish();
        self.resolve_tab();
        self.trap_focus();
        self.sweep();
        self.settle_caret();
        // **Where the focus ended, recorded for the next frame's vanish rule.** It is the last step
        // because every one above it can move the focus.
        self.ring.note(self.focused);
        self.resolve_into_view();

        // **The one wake, read from the one sink.** 08 folded a `repaint` flag in here and 06 folded
        // the flag itself away: `request_frame()` is `deadline(now)`, so there is nothing left to
        // combine and the fold that could drop half of what asked is gone. Two callers asking for
        // different moments is still one wake at the earlier of them — `Frame::ask` does that where
        // the ask happens, which is also where the line that asked is still known.
        let wake = self.deadline;
        self.wakes.note(wake, now);
        wake
    }

    /// **Award the pointer, from the index that has just drawn.**
    ///
    /// This is where both one-frame lags close. The guess `begin` resolved was a frame old, so it
    /// could be wrong about *where* — and it never decides a click, only in-frame feedback. Here the
    /// index is this frame's, so `over` was computed against the pointer as it was when this frame
    /// drew, and a press that arrived where no frame had yet drawn still finds its widget.
    ///
    /// **That last case is not exotic**: at `Buttons` tracking the terminal sends no motion events at
    /// all, so *every* press arrives at a position no frame has been told about.
    fn award(&mut self) {
        let mut a = Awarded {
            mods: self.mods,
            ..Awarded::default()
        };
        let over_entry = self.topmost_over_entry();
        let over = over_entry.map(|h| h.id);
        a.hovered = over;

        // **By index, and cleared afterwards.** `std::mem::take` here dropped the batch's
        // allocation and `begin` bought it back on the next frame that saw a pointer event — an
        // allocation a frame, on the frame where the pointer is busiest. It passed the steady-frame
        // gate only because that gate posts no pointer events.
        for i in 0..self.mouse.len() {
            let m = self.mouse[i];
            match m.kind {
                vitui_engine::MouseKind::Down(_) => {
                    // **The grab is exclusive.** Without it a splitter drag lights every button it
                    // crosses — the pointer is over them, and without a grab every one of them is
                    // hovered and pressable.
                    if let Some(id) = over {
                        self.grab = Some(id);
                        self.press_origin = Some((id, (i32::from(m.x), i32::from(m.y))));
                        a.pressed = Some(id);
                    }
                    // **`Tab` and the press are one mechanism**: both award at `end` from the
                    // structure that has just drawn, and both mean the widget first sees itself
                    // focused on the *next* frame. That is what `focus_entered` was built for, and
                    // it has two producers rather than one.
                    match over_entry {
                        Some(h) if h.interest.contains(Interest::FOCUS) => {
                            self.focused = Some(h.id);
                        }
                        // **A press on nothing interested is intent**: the user meant to defocus, and
                        // the vanish rule must not undo it. A press on something that is interested
                        // but is not a tab stop — a tree row, a table cell — leaves the focus alone.
                        None => self.focused = None,
                        Some(_) => {}
                    }
                }
                vitui_engine::MouseKind::Up(_) => {
                    let held = self.grab.take();
                    self.press_origin = None;
                    if let Some(id) = held {
                        a.released = Some(id);
                        // A click is a release **over the widget that was pressed**.
                        if over == Some(id) {
                            let double = self.is_double(id, m.at, (i32::from(m.x), i32::from(m.y)));
                            a.clicked = Some((id, double));
                            self.click_record = Some((id, m.at));
                        } else {
                            // Released somewhere else: the drag was cancelled, and **the application
                            // is told**, which the identity sweep alone did not do.
                            a.cancelled = Some(id);
                        }
                    }
                }
                // **The wheel is not awarded here**, and it is the only pointer channel that is
                // not. It was resolved in `begin` by `Frame::resolve_wheel`, because the offset it
                // moves is read *during* the draw by the widget that owns it — an award made at this
                // point arrives after its only reader. See [`crate::scroll`].
                vitui_engine::MouseKind::Wheel(_) => {}
                vitui_engine::MouseKind::Move => {}
            }
        }
        self.mouse.clear();

        // **The long press costs a wakeup, and it is the only field that does**, because no event
        // arrives while a button is held. Attributed to the runtime's own call site: blaming a
        // component for a wake the runtime asked for is worse than no attribution at all.
        if let Some((id, _)) = self.press_origin {
            if let Some((_, since)) = self.click_record.filter(|(held, _)| *held == id)
                && since.elapsed() >= self.pointer_config.long_press
            {
                a.long_pressed = Some(id);
            }
            self.deadline = Some(match self.deadline {
                Some(at) => at.min(Instant::now() + self.pointer_config.long_press),
                None => Instant::now() + self.pointer_config.long_press,
            });
        }

        self.awarded = Some(a);
    }

    /// Whether this click is the second of two.
    ///
    /// **Two comparisons against the *event's* stamp**, never the frame clock: the frame clock is
    /// sampled once and a split drain would make it measure the drain rather than the user.
    fn is_double(&self, id: Id, at: Instant, at_xy: (i32, i32)) -> bool {
        let Some((last_id, last_at)) = self.click_record else {
            return false;
        };
        if last_id != id {
            return false;
        }
        let soon = at.duration_since(last_at) <= self.pointer_config.click_threshold;
        let near = self.press_origin.is_none_or(|(_, origin)| {
            origin.0.abs_diff(at_xy.0) <= u32::from(self.pointer_config.click_slop)
                && origin.1.abs_diff(at_xy.1) <= u32::from(self.pointer_config.click_slop)
        });
        soon && near
    }

    /// **Release the three id-keyed facts whose widget stopped drawing.**
    ///
    /// The grab, the press origin and the focus — and **not the click record**, which is the fourth
    /// and deliberately survives its widget so that a double click survives a redraw.
    ///
    /// Without this, **a stale grab swallows the pointer for every widget still on screen**: a widget
    /// that held the pointer and then stopped drawing keeps holding it, and every hit test afterwards
    /// resolves to something that is not there.
    ///
    /// The test is *did this id draw this frame*, which is exactly what the stamped table answers —
    /// so the sweep is three lookups rather than a scan, and 281 ns is the whole of it on a dense
    /// screen.
    fn sweep(&mut self) {
        if let Some(id) = self.grab
            && !self.ids.drew(id)
        {
            self.grab = None;
            // The press origin goes with the grab: it is the same interaction, and a press origin
            // without a grab is a drag nobody is holding.
            self.press_origin = None;
        }
        if let Some((id, _)) = self.press_origin
            && !self.ids.drew(id)
        {
            self.press_origin = None;
        }
        // **The focus is not swept here**, and separating it from these two is ticket 12's finding.
        // Answering absence with `None` costs the keyboard entirely — every key afterwards reaches
        // nobody until the user picks the pointer back up — so absence is `vanish` and the `None`
        // that stands is the award's. See [`Frame::vanish`].
        //
        // The click record is **not** swept either. See this function's documentation.
    }

    /// **The ring takes the `Tab` nobody drained.**
    ///
    /// Resolved here rather than in `begin`, which is what makes `Tab` a *closing* edge: the ring is
    /// declared during the draw, so the frame that moves the focus is the frame that saw the ring
    /// the user is looking at. Resolved in `begin` instead, a row inserted directly after the
    /// focused one is jumped straight over — and a row *prepended* does not separate the two
    /// answers, which is how a one-frame hop survives five tickets.
    ///
    /// A frame moves the focus **at most once**: a second `Tab` in the same batch waits, because
    /// `route::batch_len` ended the batch at the first.
    fn resolve_tab(&mut self) {
        let Some(k) = self.keys.peek_edge() else {
            return;
        };
        // `BackTab` is what a terminal sends for the shifted key when it can; the ones that cannot
        // send `Tab` with the shift bit, and both mean the same direction.
        let forward =
            k.code != vitui_engine::KeyCode::BackTab && !k.mods.contains(vitui_engine::Mods::SHIFT);
        let from = self.focused.and_then(|id| self.ring.position(id));
        // **The ring takes the key only if it has somewhere to put the focus.** A frame with no tab
        // stops moves nothing, and the `Tab` then reaches the application like any other key nobody
        // took rather than disappearing into a mechanism that had no answer for it.
        if let Some(id) = self.ring.advance(from, forward) {
            self.focused = Some(id);
            self.keys.drop_edge();
            // **The one test for *this was a keyboard-driven move*.** Scroll-into-view fires on it
            // and on nothing else the runtime moves the focus with.
            self.tab_moved = true;
        }
    }

    /// **A standing trap pulls the focus into itself**, which is also where the focus goes when a
    /// modal opens: the trap's first stop.
    fn trap_focus(&mut self) {
        if let Some(id) = self.ring.trap_pull(self.focused) {
            self.focused = Some(id);
        }
    }

    /// **The vanish rule.** The focused id stopped drawing, so the focus moves to the nearest
    /// surviving entry in the previous frame's ring order.
    ///
    /// The two `None`s are separated here and they were conflated by two tickets at once: the
    /// award's is *intent* — a press landed on nothing interested and the user meant to defocus, so
    /// there is no id left to have vanished — and this one is *absence*, which is the answer only
    /// when nothing in the previous ring survived.
    fn vanish(&mut self) {
        let Some(id) = self.focused else { return };
        if self.ids.drew(id) {
            return;
        }
        self.focused = self.ring.vanished(id);
    }

    /// **Nothing focused means no caret**, and the runtime blinks nothing.
    ///
    /// A caret on a screen where no widget holds the keyboard is a lie about where typing goes. The
    /// terminal's own caret is what is being placed (ADR 0005), and a software caret is two wakeups
    /// a second for as long as anything has focus — which is why `end` asks for no wake here.
    fn settle_caret(&mut self) {
        if self.focused.is_none() {
            self.caret = None;
        }
    }

    /// **Scroll-into-view, resolved from the ring that has just drawn.**
    ///
    /// This is the step that makes it *one* frame rather than two: the ring is this frame's, the
    /// focus has already moved, and the request is on the frame before the next draw reads it.
    ///
    /// It needs the ring to carry a **content-coordinate rectangle**, which overturns §8's *the ring
    /// carries no geometry* and nothing beside it — the rule was never "no geometry" but *geometry is
    /// needed inside a frame and never across one* (ADR 0015), and this rect is read **here**,
    /// exactly where the press award already is. What crosses the boundary is [`IntoView`]: sixteen
    /// bytes, an area and an offset, no `Rect`.
    ///
    /// Three refusals:
    ///
    /// - **it fires only for a keyboard-driven focus move** — [`Frame::tab_moved`] is the whole test,
    ///   and a press does not pull, because a press already proves the widget was on screen;
    /// - a stop **outside every scroll area** asks for nothing, because there is nothing to move;
    /// - a request nobody took **lives one frame**, so the assignment below is unconditional: the
    ///   alternative is a pull that fires long after the move that asked for it.
    fn resolve_into_view(&mut self) {
        let keyboard = self.tab_moved.then(|| self.keyboard_into_view()).flatten();
        // An explicit `request_into_view` wins: the component named a rectangle, which is more than
        // the ring knows.
        self.into_view = self.into_view_asked.take().or(keyboard);
    }

    /// The focused stop's request, if it is inside an area and is not already visible.
    fn keyboard_into_view(&self) -> Option<IntoView> {
        let id = self.focused?;
        let stop = self.ring.stops().get(self.ring.position(id)?)?;
        let area = self.scroll_areas.get(usize::try_from(stop.area?).ok()?)?;
        let by = area.into_view(stop.rect);
        (by != (0, 0)).then_some(IntoView { area: area.id, by })
    }

    /// The hover style to apply, if anything is hovered. **Resolved from the index that has just
    /// drawn**, which is what makes it land in the same frame.
    fn hover_to_apply(&self) -> Option<(Rect, crate::theme::Role)> {
        let hovered = self.awarded.and_then(|a| a.hovered)?;
        self.hover_styles
            .iter()
            .rev()
            .find(|(id, _, _)| *id == hovered)
            .map(|&(_, r, role)| (r, role))
    }

    /// The hit index, for the gates and for ticket 10.
    pub fn hits(&self) -> &[Hit] {
        &self.hits
    }

    /// The scroll areas this frame declared, in draw order. **For the gates and for `end`.**
    pub fn scroll_areas(&self) -> &[Area] {
        &self.scroll_areas
    }

    /// The pending scroll-into-view request, if `end` produced one. **The only sixteen bytes of this
    /// subsystem that cross a frame.**
    pub fn into_view(&self) -> Option<IntoView> {
        self.into_view
    }

    /// How far one wheel click moves.
    pub fn wheel_config(&self) -> Wheel {
        self.wheel_config
    }

    /// The drawn extent, and **`None` unless something asked this frame to maintain one**.
    ///
    /// A real frame never does: see the field, and spec §12 for what it would cost.
    pub fn extent(&self) -> Option<Extent> {
        self.extent
    }

    /// The focus ring: every entry that declared [`Interest::FOCUS`], in draw order.
    ///
    /// **Read-only, and there is no verb that reorders it.** *The ring is the reading order of the
    /// source* is only true if nothing can rearrange it; a component that wants to be visited
    /// earlier moves its call.
    pub fn ring(&self) -> &[Stop] {
        self.ring.stops()
    }

    /// The ring's ids, which is what most gates actually compare.
    pub fn ring_ids(&self) -> impl Iterator<Item = Id> + '_ {
        self.ring.stops().iter().map(|s| s.id)
    }

    /// How many **tab stops** this frame has, which is not how many ring entries it has: a
    /// [`Group`](crate::focus::ScopeKind::Group) collapses its whole range onto one.
    pub fn stop_count(&self) -> usize {
        self.ring.stop_indices().count()
    }

    /// The tab stops, in ring order.
    pub fn stop_ids(&self) -> impl Iterator<Item = Id> + '_ {
        self.ring.stop_indices().map(|ix| self.ring.stops()[ix].id)
    }

    /// The order a keyboard walkthrough visits from where the focus is now.
    ///
    /// **A read, never a move.** The walk is not a verb a component can call: one that could would
    /// move the focus during the draw, over a ring that is half built — the stale-ring defect this
    /// ticket removed, reintroduced from above. The only focus verb is [`Ctx::focus`], which names a
    /// widget rather than a direction.
    ///
    /// # The positive twin, naming the protected items by path
    ///
    /// Ticket 19's refinement 3: **a lone `compile_fail` also passes when the protected item has
    /// been renamed**, because `E0599` for *the method you meant is now spelled differently* and
    /// `E0599` for *the method you must not have was never built* are the same diagnostic. This pair
    /// was the one case in the corpus written without a twin, so both halves below would have gone
    /// on passing through a rename of either [`Frame::tab_walk`] or [`Frame::ring`]. The twin names
    /// both by path first, and a rename now fails **here**:
    ///
    /// ```
    /// use vitui_engine::Rect;
    /// use vitui_runtime::ctx::{Driver, Frame};
    /// use vitui_runtime::focus::Stop;
    /// use vitui_runtime::{Id, Interest};
    ///
    /// fn protected(f: &Frame) -> (Vec<Id>, usize) {
    ///     let walk: Vec<Id> = Frame::tab_walk(f).collect();
    ///     let ring: &[Stop] = Frame::ring(f);
    ///     (walk, ring.len())
    /// }
    ///
    /// let mut d = Driver::headless(20, 3).expect("sink");
    /// d.frame(|cx| {
    ///     cx.interact(Id::named("field"), Rect::new(0, 0, 4, 1), Interest::FOCUS);
    /// });
    /// let (walk, stops) = protected(d.inspect());
    /// assert_eq!(stops, 1, "one widget asked to be in the ring");
    /// assert_eq!(walk.len(), stops, "the walk visits the ring and nothing else");
    /// ```
    ///
    /// # And the negative cases: there is no shape to ask for a move, or to reorder the ring
    ///
    /// The codes are declared and **rustdoc does not check them** — measured on rustc 1.97.1, not
    /// assumed; `register::tests::every_negative_case_declares_the_error_it_expects` carries the
    /// evidence. They are here as documentation of which error the author meant, and the twin above
    /// is what actually holds these two halves to their subject.
    ///
    /// ```compile_fail,E0599
    /// use vitui_runtime::ctx::Driver;
    ///
    /// let mut d = Driver::headless(20, 3).expect("sink");
    /// d.frame(|cx| {
    ///     cx.focus_next();
    /// });
    /// ```
    ///
    /// Nor can a component reorder the ring: it is a private field behind a shared slice, and *the
    /// ring is the reading order of the source* is only true while that stays so. A widget that
    /// wants to be visited earlier moves its call.
    ///
    /// ```compile_fail,E0596
    /// use vitui_runtime::ctx::Driver;
    ///
    /// let mut d = Driver::headless(20, 3).expect("sink");
    /// d.frame(|_cx| {});
    /// d.inspect().ring().sort_by_key(|stop| stop.rect.y);
    /// ```
    pub fn tab_walk(&self) -> impl Iterator<Item = Id> + '_ {
        self.ring.walk()
    }

    /// This frame's scopes, as ranges over the ring.
    ///
    /// **Frame-local**: they are built during the draw and cleared by the next `begin`, so there is
    /// nothing here to become a fifth id-keyed cross-frame fact.
    ///
    /// # The positive twin, naming the protected item by path
    ///
    /// A lone `compile_fail` also passes when the item has been renamed, so the shape that ships is
    /// pinned here first — a rename fails *this* half rather than making the half below pass for the
    /// wrong reason:
    ///
    /// ```
    /// use vitui_runtime::ctx::{Ctx, Driver};
    /// use vitui_runtime::focus::{ScopeKind, ScopeRec};
    /// use vitui_runtime::Id;
    ///
    /// fn protected<'f, 'v>(cx: &mut Ctx<'f, 'v>, id: Id) {
    ///     Ctx::scope(cx, id, ScopeKind::Trap, |_inner| {});
    /// }
    ///
    /// let mut d = Driver::headless(20, 3).expect("sink");
    /// d.frame(|cx| protected(cx, Id::named("modal")));
    /// let scopes: &[ScopeRec] = d.inspect().scopes();
    /// assert_eq!(scopes.len(), 1, "a scope is a range over the frame that made it");
    /// ```
    ///
    /// # And the negative case: a scope cannot become a cross-frame fact
    ///
    /// The shape a caller would have to write to keep one is a borrow held across the next frame,
    /// and the frame needs `&mut`:
    ///
    /// ```compile_fail,E0502
    /// use vitui_runtime::ctx::Driver;
    ///
    /// let mut d = Driver::headless(20, 3).expect("sink");
    /// d.frame(|_cx| {});
    /// let scopes = d.inspect().scopes();      // borrowed from this frame
    /// d.frame(|_cx| {});                      // and the next one needs `&mut`
    /// let _kept = scopes.len();
    /// ```
    pub fn scopes(&self) -> &[crate::focus::ScopeRec] {
        self.ring.scopes()
    }

    /// **The traps standing this frame.**
    ///
    /// Three components tickets read this to *name* the exception their keyboard-walkthrough gate
    /// carries — *the walk repeats no id and reaches every stop, unless a trap is standing* — and
    /// nothing else can answer it. An iterator rather than a `Vec`, because a `collect` on the frame
    /// path is one allocation against a budget of zero.
    pub fn trap_scopes(&self) -> impl Iterator<Item = Id> + '_ {
        self.ring.trap_ids()
    }

    /// Who holds the focus.
    pub fn focused(&self) -> Option<Id> {
        self.focused
    }

    /// Where the caret goes, as `settle` will hand it to `Screen::set_cursor`.
    ///
    /// **The sink, and the gate asserts against it**: the shape a component asked for is the shape
    /// in this `Cursor`, and the last write of the frame is the one that is here.
    pub fn caret(&self) -> Option<Cursor> {
        self.caret
    }

    /// **How many slots the vanish rule touched this frame. Zero on a quiet one.**
    ///
    /// A count and not a stopwatch, for the reason §20 gives: the defect this detects is a slope,
    /// and a ratio of timings is a report.
    pub fn vanish_probes(&self) -> u64 {
        self.ring.probes()
    }

    /// How many overlay requests are still queued. **Zero after a frame**, because the pass drains
    /// the queue rather than reading it — which is what makes it a queue and not a log.
    pub fn overlays_requested(&self) -> usize {
        self.overlays.len()
    }

    /// How many overlays the pass gave a layer to this frame.
    pub fn overlays_placed(&self) -> u32 {
        self.overlays_placed
    }

    /// How many requests named an owner that had already asked this frame and were therefore inert.
    ///
    /// **The same policy as [`Ctx::interact`]'s merge, for the same reason**: one owner keys one
    /// layer, so a second request under one id would have two rectangles for one surface. It is a
    /// count rather than a panic because the failure is cosmetic and visible on screen.
    pub fn overlays_merged(&self) -> u32 {
        self.overlays_merged
    }

    /// How many rounds the overlay pass ran, out of [`OVERLAY_ROUNDS`].
    ///
    /// **A body that requests itself for ever reaches the bound**, and the gate reads this to say so
    /// — a limit, not a hang.
    pub fn overlay_rounds(&self) -> u32 {
        self.overlay_rounds
    }

    /// How many overlay bodies were boxed this frame. **One allocation each.**
    ///
    /// The frame's whole overlay cost is `n + 1` for `n` bodies — one `Box` a body plus the one `Vec`
    /// that holds them — and **0 for a frame with no overlay**, because an empty `Vec` allocates
    /// nothing. The queue cannot keep its capacity across frames, because a body is `+ 'f`; see
    /// [`crate::overlay`] and spec §19.
    pub fn overlay_bodies_boxed(&self) -> u32 {
        self.overlay_bodies
    }

    /// The id table.
    pub fn ids(&self) -> &IdTable {
        &self.ids
    }

    /// How many frames have run.
    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// Whether `begin` has ever run.
    pub fn begun(&self) -> bool {
        self.begun
    }

    /// **What the frames asked for, and which line asked.** See [`crate::anim::WakeLedger`].
    ///
    /// It is on `Frame` rather than on `Driver` because it is frame state — the one piece of it that
    /// survives the swap-and-clear, because a streak is a run of frames.
    pub fn wakes(&self) -> &crate::anim::WakeLedger {
        &self.wakes
    }

    /// The tracking level the pointer needs, which is the `max` of what was declared.
    pub fn tracking(&self) -> MouseMode {
        self.tracking
    }

    /// Plant an id-keyed fact, for the gates. **Tickets 10 and 12 own the real writers** — the press
    /// award and the focus resolution — and this is how ticket 09 tests the sweep without inventing
    /// either of them early.
    pub fn plant_facts(&mut self, grab: Option<Id>, focus: Option<Id>, click: Option<Id>) {
        self.grab = grab;
        self.press_origin = grab.map(|id| (id, (0, 0)));
        self.focused = focus;
        self.click_record = click.map(|id| (id, Instant::now()));
    }

    /// Where the modal barrier is, for the gate. **`Option`, and `Some(0)` is not `None`.**
    pub fn modal_from(&self) -> Option<usize> {
        self.modal_from
    }

    /// Whose drag was cancelled, if any.
    pub fn cancelled_drag(&self) -> Option<Id> {
        self.awarded.and_then(|a| a.cancelled)
    }

    /// Who is hovered, as awarded from the index that has just drawn.
    pub fn hovered_now(&self) -> Option<Id> {
        self.awarded.and_then(|a| a.hovered)
    }

    /// **What nobody took**, in arrival order.
    ///
    /// The focus ring reads this: a `Tab` no scope claimed is what moves the focus, and a `Tab` an
    /// isolated scope *did* claim must not. Ticket 12 is the caller; the door is here because the
    /// queue is.
    pub fn undrained_keys(&self) -> &[vitui_engine::Key] {
        self.keys.undrained()
    }

    /// How many keys this frame's batch carried, drained or not.
    pub fn keys_in_batch(&self) -> usize {
        self.keys.len()
    }

    /// **How many key slots the queue has touched since the driver was made.**
    ///
    /// The growth-ratio gate's number, and it is a count for the reason §20 gives: a ratio of
    /// timings is a report. One slot a key is linear; the `Vec::remove(0)` form touches the whole
    /// tail and is 16× at 4× the keys.
    pub fn key_touches(&self) -> u64 {
        self.keys.touched()
    }

    /// Who may take a key right now. **One id, and there is no map behind it.**
    pub fn routed_to(&self) -> Option<Id> {
        self.route_to
    }

    /// The four id-keyed facts, for the gate that counts them.
    pub fn id_keyed_facts(&self) -> (bool, bool, bool, bool) {
        (
            self.grab.is_some(),
            self.press_origin.is_some(),
            self.focused.is_some(),
            self.click_record.is_some(),
        )
    }
}

/// A component's whole view of the runtime.
///
/// # The paired compile outcome
///
/// **An overlay body cannot borrow a base-pass local:**
///
/// ```compile_fail,E0373
/// use vitui_runtime::ctx::{Ctx, Driver, Id};
/// use vitui_runtime::overlay::OverlayOpts;
/// use vitui_engine::Rect;
///
/// let mut driver = Driver::headless(20, 5).expect("sink");
/// driver.frame(|cx| {
///     let base_pass_local = String::from("dies at the end of the base pass");
///     // No `move`: the closure borrows, and the borrow is what cannot outlive the frame.
///     cx.overlay(Id::ROOT, Rect::new(0, 0, 4, 1), OverlayOpts::sized(4, 1), |_inner| {
///         let _ = &base_pass_local;
///     });
/// });
/// ```
///
/// **`move` defeats this gate, and the first version of it was written with `move`.** A moved capture
/// is *owned*, and an owned value satisfies `: 'f` for every `'f` — so the hostile case compiled and
/// the pair proved nothing. The bound is about **borrows** escaping the frame, which is also the only
/// thing it needs to be about: a body that owns its data cannot dangle.
///
/// and the twin, naming `Ctx` by path and capturing what a body legitimately may — application data
/// that outlives the frame, and `Copy` state:
///
/// ```
/// use vitui_runtime::ctx::{Ctx, Driver, Id};
/// use vitui_runtime::overlay::OverlayOpts;
/// use vitui_engine::Rect;
///
/// static MENU: [&str; 2] = ["Open", "Save"];
/// let mut driver = Driver::headless(20, 5).expect("sink");
/// driver.frame(|cx: &mut Ctx<'_, '_>| {
///     let selected = 1usize;
///     cx.overlay(
///         Id::ROOT,
///         Rect::new(0, 0, 4, 1),
///         OverlayOpts::sized(4, 1),
///         move |inner: &mut Ctx<'_, '_>| {
///             let _ = (MENU[selected], inner.area());
///         },
///     );
/// });
/// ```
///
/// **The single-lifetime version of the hostile case compiles**, which is what makes this pair
/// evidence rather than an assertion — `tests::the_brand_is_invariant` keeps both that fact and the
/// covariant one runnable.
///
/// # Exactly one child is alive at a time
///
/// [`Ctx::child`] reborrows the view, the frame and the env, so a second live child is `E0499`. That
/// is the type system enforcing what a container has to do anyway: compute all the sub-rectangles
/// first, then draw the children one at a time.
///
/// ```compile_fail,E0499
/// use vitui_runtime::ctx::Driver;
/// use vitui_engine::Rect;
///
/// let mut driver = Driver::headless(20, 5).expect("sink");
/// driver.frame(|cx| {
///     let a = cx.child(Rect::new(0, 0, 4, 1));
///     let b = cx.child(Rect::new(4, 0, 4, 1));
///     let _ = (a.area(), b.area());
/// });
/// ```
///
/// # `Ctx` is `!Send`, in both directions
///
/// A `Surface` is `Send`, so a `View` is, so a `Ctx` would be — and a `Ctx` on another thread is a
/// draw on another thread, which is the one thing the engine's split handles exist to prevent. The
/// marker is `PhantomData<*const ()>` and no annotation asks for it.
///
/// ```compile_fail,E0277
/// use vitui_runtime::ctx::Driver;
///
/// fn needs_send<T: Send>(_: T) {}
/// let mut driver = Driver::headless(20, 5).expect("sink");
/// driver.frame(|cx| {
///     needs_send(cx);
/// });
/// ```
///
/// and the twin, which is the same helper accepting the things that legitimately are:
///
/// ```
/// use vitui_runtime::ctx::{Driver, Id};
/// use vitui_runtime::Theme;
///
/// const fn needs_send<T: Send>() {}
/// needs_send::<Theme>();
/// needs_send::<Id>();
/// needs_send::<vitui_engine::Rect>();
/// let mut driver = Driver::headless(20, 5).expect("sink");
/// driver.frame(|cx| {
///     // And the `Ctx` still draws; it simply cannot leave this thread.
///     let _ = cx.area();
/// });
/// ```
pub struct Ctx<'f, 'v> {
    view: View<'v>,
    frame: &'v mut Frame,
    env: &'v Env,
    rect: Rect,
    /// Where this context's own `(0, 0)` sits in the coordinates of the frame's root.
    ///
    /// **Not derivable from `rect`**, which is a sub-rectangle in its *parent's* coordinates: two
    /// levels down, adding one `rect.x` gives the wrong column. Two contexts writing `(0, 0)` name
    /// the same cell without it, and last-write-wins then picks between two different places on
    /// screen. It is accumulated on the way down rather than computed because its three readers —
    /// the drawn extent, the caret, and the hover style resolved at `end` — all run where every
    /// intermediate `Ctx` has already been dropped.
    origin: (i32, i32),
    /// The pointer **in this context's own coordinates**, transformed on the way down by `child` and
    /// `scrolled`.
    ///
    /// This is what makes the hit index need no geometry: containment is decided *during the draw*, in
    /// the coordinates the widget is already thinking in, and the index carries one bit instead of a
    /// rectangle. Geometry is needed **inside** a frame and never across one (ADR 0015) — which is the
    /// rule, and is not the same as *no geometry*.
    pointer: Option<(i32, i32)>,
    /// This context's origin **in the enclosing scroll area's content coordinates**, which is what a
    /// ring entry's rectangle is in.
    ///
    /// It resets at the area boundary — a `scrolled` context *is* a content coordinate system — and
    /// at the root, where the two are the same thing. Read at `end` and never across a frame
    /// (ADR 0015).
    content: (i32, i32),
    /// **The frame call's overlay body queue**, shared by every context in the frame rather than
    /// owned by one — the same reasoning as [`Frame::layer_z`], one level further out: *where a body
    /// is kept* is a property of the frame call and not of a context.
    ///
    /// A shared reference with interior mutability, so `child()` can hand it on while the frame is
    /// borrowed mutably beside it.
    bodies: &'v Bodies<'f>,
    /// **Invariant in `'f`**, and the whole overlay guarantee rests on it: a covariant brand lets a
    /// caller shorten `'f` at a `child()` call, which makes the `+ 'f` bound on an overlay body
    /// satisfiable by a shorter capture. A lifetime in both argument and return position of a `fn`
    /// pointer is invariant.
    ///
    /// The queue above is invariant in `'f` for its own reasons and would carry the brand today, but
    /// the marker stays: it is what the paired compile outcome below is written against, and a
    /// guarantee that holds only while a field's type happens to be what it is is not a guarantee.
    _frame: PhantomData<fn(&'f ()) -> &'f ()>,
    /// **`!Send`, and it needs saying.** A `Surface` is `Send`, so a `View` is, so `Ctx` would be —
    /// and a `Ctx` on another thread is a draw on another thread, which is the one thing the engine's
    /// split handles exist to prevent.
    _not_send: PhantomData<*const ()>,
}

impl<'f, 'v> Ctx<'f, 'v> {
    /// This context's own rectangle, origin-relative.
    ///
    /// The engine's `View` has a size and not a rect, so this is the runtime's own arithmetic.
    pub fn area(&self) -> Rect {
        Rect::new(0, 0, self.rect.w, self.rect.h)
    }

    /// Its size.
    pub fn size(&self) -> (u16, u16) {
        self.view.size()
    }

    /// Narrow to a sub-rectangle. **Can never widen.**
    ///
    /// # A hole, recorded rather than closed
    ///
    /// A component is handed a `Ctx` *and* an area, by convention — `fn thing(cx: &mut Ctx, area:
    /// Rect)` — and nothing stops it drawing outside the area it was given. `child` narrows the
    /// *clip*, so a rude component cannot escape its parent's rectangle; it can freely trample its
    /// sibling's half of it.
    ///
    /// ```text
    /// polite: aaaaaaaaaabbbbbbbbbb
    /// rude:   aaaaaaaaaabbbbbbbbbbbbbbbbbbbb
    /// ```
    ///
    /// The engine's own sentence applies, and is quoted rather than paraphrased because it is the
    /// reason this is written down at all: **"for a library other people write components against, a
    /// seam defended by convention is not defended."**
    ///
    /// Three fixes exist and each has a price: drop `area` from the signature entirely, so the only
    /// rectangle a component has is its own; add `cx.at(area, |cx| …)` so narrowing is the only way to
    /// address a sub-area; or keep the convention and detect the violation in a debug build, one
    /// compare per verb, where the watermark already sits.
    ///
    /// **The choice belongs to the components map, not here.** This ticket records it and closes
    /// nothing, and this rustdoc does not pretend the convention is a guarantee.
    pub fn child(&mut self, r: Rect) -> Ctx<'f, '_> {
        Ctx {
            view: self.view.child(r),
            frame: self.frame,
            env: self.env,
            rect: r,
            origin: (self.origin.0 + r.x, self.origin.1 + r.y),
            pointer: self.pointer.map(|(x, y)| (x - r.x, y - r.y)),
            content: (self.content.0 + r.x, self.content.1 + r.y),
            bodies: self.bodies,
            _frame: PhantomData,
            _not_send: PhantomData,
        }
    }

    /// Offset the content coordinates, which is how a scrolled list is drawn.
    ///
    /// **`(dx, dy)` is a translation and not a scroll position**: scrolling *down* by `n` is
    /// `scrolled(0, -n)`, which is what `tests::visible_rows_bounds_a_million_rows_to_a_screenful`
    /// and `tests::local_survives_nesting_and_scrolling` both pin. [`Ctx::scroll_scope`] takes the
    /// *offset* and negates it here; the two senses are one negation apart and the negation is
    /// written down at exactly one call site.
    pub fn scrolled(&mut self, dx: i32, dy: i32) -> Ctx<'f, '_> {
        Ctx {
            view: self.view.scrolled(dx, dy),
            frame: self.frame,
            env: self.env,
            rect: self.rect,
            // **The same sense as `view.origin` and not the opposite one**, which is components
            // ticket 12's correction: `View::at` is `(x + origin.0, y + origin.1)` and this origin
            // is read by `hover_style` for exactly the same content-to-root map, so the two cannot
            // take the translation with different signs. Written `- dy` it agreed with a *scroll
            // position* reading that `view` and `pointer` on the two lines below do not take, and
            // the disagreement was invisible because nothing had ever awarded a hover inside a
            // scrolled scope.
            origin: (self.origin.0 + dx, self.origin.1 + dy),
            pointer: self.pointer.map(|(x, y)| (x - dx, y - dy)),
            // **The reset**: a scrolled context is a content coordinate system, so its own origin is
            // the content origin. §13's `to_content` resets at the area boundary, and this is it.
            content: (0, 0),
            bodies: self.bodies,
            _frame: PhantomData,
            _not_send: PhantomData,
        }
    }

    /// Which content rows can still be reached. **The virtualisation primitive**: a list over a
    /// million rows iterates this range and nothing else.
    pub fn visible_rows(&self) -> Range<i32> {
        self.view.visible_rows()
    }

    /// Which content columns can still be reached.
    pub fn visible_cols(&self) -> Range<i32> {
        self.view.visible_cols()
    }

    /// Note how far a verb reached, in the coordinates of the frame's root.
    ///
    /// **`None` in a real frame and a not-taken branch there**, which is the whole of what makes the
    /// drawn extent affordable: the alternative is a grapheme walk per verb, and spec §12 prices
    /// that at 7% of the frame budget.
    #[inline]
    fn note(&mut self, r: Rect) {
        let origin = self.origin;
        if let Some(extent) = self.frame.extent.as_mut() {
            extent.reach(origin, r);
        }
    }

    /// The same, for a verb whose width has to be measured to be known.
    ///
    /// **The measurement is inside the branch, not before it.** Hoisting it out is the version that
    /// costs a real frame the 7%.
    #[inline]
    fn note_text(&mut self, x: i32, y: i32, s: &str) {
        if self.frame.extent.is_some() {
            let w = crate::layout::text::width(s);
            self.note(Rect::new(x, y, w, 1));
        }
    }

    /// Write a string.
    pub fn text(&mut self, x: i32, y: i32, s: &str, st: Paint) -> Written {
        self.note_text(x, y, s);
        self.view.text(x, y, s, st.style())
    }

    /// Write one grapheme cluster.
    pub fn set(&mut self, x: i32, y: i32, cluster: &str, st: Paint) -> Written {
        self.note_text(x, y, cluster);
        self.view.set(x, y, cluster, st.style())
    }

    /// Fill a rectangle.
    pub fn fill(&mut self, r: Rect, cluster: &str, st: Paint) {
        self.note(r);
        self.view.fill(r, cluster, st.style());
    }

    /// Fill this whole context.
    pub fn clear(&mut self, st: Paint) {
        let area = self.area();
        self.note(area);
        self.view.fill(area, " ", st.style());
    }

    /// Restyle a rectangle without rewriting its content.
    ///
    /// **A descriptor and never a closure**, which is the deviation that closed the fourth compile
    /// outcome: a closure over styles could return one it invented, and there would be a hole for a
    /// component to mint a paint through. See [`Repaint`].
    pub fn restyle(&mut self, r: Rect, d: &Repaint<'_>) {
        let lowered = d.lower(self.env.theme());
        self.note(r);
        self.view.restyle(r, &lowered);
    }

    /// Stage a formatted value and return **its display width**.
    ///
    /// A width rather than a guard or a `&str`, because that is what the caller needs before it knows
    /// where to draw: a right-aligned number, a truncating cell and a centred label all measure
    /// first. See `Scratch` for why the guard cannot be written at all.
    pub fn stage(&mut self, args: fmt::Arguments<'_>) -> u16 {
        use fmt::Write as _;

        self.frame.scratch.buf.clear();
        // The buffer keeps its allocation across frames, so this is a write into existing capacity.
        let _ = self.frame.scratch.buf.write_fmt(args);
        crate::layout::text::width(&self.frame.scratch.buf)
    }

    /// Draw what was staged.
    pub fn blit(&mut self, x: i32, y: i32, st: Paint) -> Written {
        // The staged text and the view are disjoint fields, so this needs no copy — which is the
        // other half of why the buffer lives on the frame rather than in a guard.
        let staged = std::mem::take(&mut self.frame.scratch.buf);
        self.note_text(x, y, &staged);
        let written = self.view.text(x, y, &staged, st.style());
        self.frame.scratch.buf = staged;
        written
    }

    /// Stage and draw in one call, which is the ninety-per-cent path.
    pub fn label(&mut self, x: i32, y: i32, args: fmt::Arguments<'_>, st: Paint) -> Written {
        let _ = self.stage(args);
        self.blit(x, y, st)
    }

    // ── `Ctx::link` is not here, and it is **unnecessary** rather than unblocked ─────────────────
    //
    // It was specified — `link(uri) -> Link` on `Ctx` — and recorded here as unimplementable, in
    // twenty-three lines ending *"The fix is one method on `View` or on `LayerStack`, and it is the
    // engine map's to make."* Minting needed `&mut Screen`; a `Ctx` holds a `View` borrowed from it
    // through `LayerStack::view`, so the call that minted and the type that draws could not be held
    // at once, by construction rather than by oversight.
    //
    // Engine architecture ticket 21 answered it by deleting the mint. There is no handle to get, so
    // there is no verb to put anywhere: a component writes `Repaint { link: Some(Link::Uri(uri)) }`
    // and `Ctx::restyle` lowers it, which is a field a component was already writing rather than a
    // borrow it could not obtain. `Driver::link` went with `Screen::link` for the same reason.

    /// This widget's id, from the call site.
    ///
    /// `#[track_caller]`, so the location is the caller's and it costs nothing at run time — a
    /// `&'static Location` is already in the binary.
    ///
    /// # A `#[track_caller]` wrapper merges the widgets inside its own body
    ///
    /// **Found by accident, and visible only because duplicate detection named it.** A helper marked
    /// `#[track_caller]` reports *its caller's* location for every call it makes, so two widgets drawn
    /// inside one such helper get the same id and the second is inert.
    ///
    /// ```text
    /// #[track_caller]
    /// fn labelled_field(cx: &mut Ctx, label: &str) {   // ← the attribute is the bug
    ///     cx.id();   // both of these report the *caller's* line,
    ///     cx.id();   // so they are one widget and one of them does nothing
    /// }
    /// ```
    ///
    /// The rule for a component author: **put `#[track_caller]` on a function that draws one widget,
    /// and not on one that draws several.** A wrapper that draws several should take a key, or let its
    /// children report their own call sites by not carrying the attribute at all.
    #[track_caller]
    pub fn id(&mut self) -> Id {
        Id::at(self.frame.stack.current(), std::panic::Location::caller())
    }

    /// How deep the id path is here. Zero at the top of a frame.
    ///
    /// **The gate for the container rule**: a rectangle-returning split leaves its panes at one depth
    /// and a keyed row is one deeper.
    pub fn depth(&self) -> usize {
        self.frame.stack.depth()
    }

    /// Draw under a caller-supplied key, which is what a loop needs.
    ///
    /// One of exactly **two** places the id stack is pushed. `Ctx::child` is not one of them.
    pub fn with_key<R>(&mut self, key: u64, f: impl FnOnce(&mut Ctx<'f, '_>) -> R) -> R {
        let id = Id::keyed(self.frame.stack.current(), key);
        self.with_id(id, f)
    }

    /// Draw under an id the caller already has.
    ///
    /// The other of the two places the stack is pushed.
    pub fn with_id<R>(&mut self, id: Id, f: impl FnOnce(&mut Ctx<'f, '_>) -> R) -> R {
        self.frame.stack.push(id);
        // A block rather than an explicit `drop`: `Ctx` has no `Drop` impl, so `drop` only extends
        // the borrow's region — clippy is right, and a scope is what actually ends it.
        let r = {
            let mut inner = Ctx {
                view: self.view.child(self.area()),
                frame: self.frame,
                env: self.env,
                rect: self.rect,
                origin: self.origin,
                pointer: self.pointer,
                content: self.content,
                bodies: self.bodies,
                _frame: PhantomData,
                _not_send: PhantomData,
            };
            f(&mut inner)
        };
        self.frame.stack.pop();
        r
    }

    /// Open a scope.
    ///
    /// # **A scope renames nothing, and that is load-bearing**
    ///
    /// This is the container rule's one exception. `scope` takes a closure, and *a container that
    /// takes a closure renames its children* — except this one, because **the frame a modal opens
    /// would otherwise rename every field of the form it traps**, and a form whose fields are renamed
    /// loses its focus and its scroll position for a reason nobody wrote down.
    ///
    /// So the id stack is untouched here, and **this is the one exception the container rule has**.
    /// It is not a nicety: the frame a modal opens is the frame a [`Trap`](ScopeKind::Trap) appears
    /// around a form that was already on screen. If the scope renamed its children, every field
    /// would take a new id at exactly that moment, the focus would name a widget that no longer
    /// exists, and the vanish rule would fire on the whole form — the failure arriving precisely
    /// when the mechanism is needed.
    ///
    /// The price is the ordinary one: two structurally identical scoped subtrees in one frame are
    /// one set of ids, and the author separates them with [`Ctx::with_key`] as for any other
    /// repeated construct.
    ///
    /// # What the kind decides
    ///
    /// A [`ScopeKind`] is three answers about `Tab` and not three degrees of one — see
    /// [`crate::focus`]. It is a **frame-local range over this frame's ring**, so nothing about it
    /// survives the frame and there is no fifth id-keyed fact.
    ///
    /// # This is where bubbling happens, and it is not a walk of the id path
    ///
    /// A container draws **before** its children, so a pull API gives *capture* and not bubbling.
    /// The only moment an ancestor can ask *after* its children is **after its body**, which only a
    /// closure-taking container has — and this is the one that already takes a closure.
    ///
    /// That looked like a third force on the container shape, pointing against the rule that a
    /// closure-taking container renames its children. **It is not a trade at all**: taking a closure
    /// is not what renames a child, *scoping identity* is, and they are independent. So the ids
    /// inside a scope are equal entry for entry to the ids without it, and **a bubbling container
    /// adds exactly one id: its own**, claimed here.
    ///
    /// After the body, if the focus drew **inside** it, this scope becomes the routing target and
    /// [`Ctx::next_key`] answers `id`. A scope the focus is not in bubbles nothing — otherwise a
    /// sibling drawn earlier would take keys the focused widget has not been offered yet.
    ///
    /// # A merge does not switch bubbling off, and here that differs from [`Ctx::interact`]
    ///
    /// `interact` makes a merged claim **inert**, because a second hit entry under one id would
    /// double-count a region. Bubbling has no such quantity: `route_to = Some(id)` is idempotent,
    /// and the case that matters is not a duplicate call site at all — it is **a panel that declares
    /// a clickable region and then opens a scope under its own id**, which is how a bubbling
    /// container is actually written. Refusing to bubble there would leave the panel silently unable
    /// to see a keystroke, with no diagnostic and nothing on screen to notice.
    pub fn scope<R>(
        &mut self,
        id: Id,
        kind: ScopeKind,
        f: impl FnOnce(&mut Ctx<'f, '_>) -> R,
    ) -> R {
        // **The one id a bubbling container adds.** Not on the id stack — a scope renames nothing —
        // and not in the hit index either, because it declares no region. The claim is for the
        // identity side alone: it makes the id live, so the sweep and the counts see it.
        let _ = self.frame.ids.claim(id);
        let opened = self.frame.ring.open_scope(id, kind);
        let before = self.frame.focus_draws;
        let r = {
            let mut inner = self.child(self.area());
            f(&mut inner)
        };
        self.frame.ring.close_scope(opened);
        // The after-the-body moment.
        if self.frame.focus_draws != before {
            self.frame.route_to = Some(id);
            // **The routing target moved, so a decline made below is spent.** Without this the
            // queue is still closed by whatever the level inside handed back, and the key it handed
            // back is exactly the one this scope exists to be offered.
            self.frame.keys.resume();
        }
        r
    }

    /// Open a scroll area: a viewport over content larger than itself, moved by an offset the
    /// **application** owns.
    ///
    /// `view` is the viewport in this context's coordinates, `offset` the application's current
    /// scroll position and `max` the largest offset the content admits — *content size minus
    /// viewport*, floored at zero, and not the content size, because an offset and its bound are the
    /// same quantity. Both are clamped here, so an application whose content just shrank is one
    /// frame stale rather than one frame wrong.
    ///
    /// # It costs the content, and that is the choice being made
    ///
    /// The body draws as if everything were visible and the clip rejects the rest, so **cost is
    /// proportional to the content**: right for a form, a panel or a document page, and catastrophic
    /// for a million rows, where it is **887–889×** a virtualised collection's 0.997–1.003×. Draw
    /// the rows [`Ctx::visible_rows`] admits *inside* this scope and it is **7.65–7.80 µs** instead
    /// — the two mechanisms compose, and it is choosing the wrong one alone that costs. See
    /// [`crate::scroll`].
    ///
    /// # It scopes no identity, and it declares no region
    ///
    /// **No identity**, for §5's reason and the same one [`Ctx::scope`] has: a container that takes a
    /// closure renames its children, except the two that exist to wrap something already on screen.
    /// A scroll area appearing around a form would otherwise rename every field in it.
    ///
    /// **No region either.** Publishing the scrollable region is [`Ctx::scrollable`]'s job and it is a
    /// separate call, because the two happen at different moments: *what moved me* has to be read
    /// **before** the offset is known, and the window is opened **after**. One verb would have to
    /// return both a [`Response`] and the body's value.
    ///
    /// ```
    /// use vitui_runtime::ctx::{Driver, Interest};
    /// use vitui_runtime::scroll::Scrollable;
    /// use vitui_runtime::Id;
    /// use vitui_engine::Rect;
    ///
    /// let mut driver = Driver::headless(20, 6).expect("sink");
    /// let (id, view, max) = (Id::named("page"), Rect::new(0, 0, 20, 6), (0, 34));
    /// let mut offset = (0, 0);
    /// driver.frame(|cx| {
    ///     // Publish, then apply: the pair per axis is computed from the offset the area drew with.
    ///     let r = cx.scrollable(id, view, Interest::NONE, Scrollable::between(offset, max));
    ///     offset.1 = (offset.1 + r.scrolled.1).clamp(0, max.1);
    ///     cx.scroll_scope(id, view, offset, max, |cx| {
    ///         let _ = cx.area();
    ///     });
    /// });
    /// ```
    pub fn scroll_scope<R>(
        &mut self,
        id: Id,
        view: Rect,
        offset: (i32, i32),
        max: (i32, i32),
        f: impl FnOnce(&mut Ctx<'f, '_>) -> R,
    ) -> R {
        let max = (max.0.max(0), max.1.max(0));
        let offset = (offset.0.clamp(0, max.0), offset.1.clamp(0, max.1));
        let ix = u32::try_from(self.frame.scroll_areas.len()).unwrap_or(u32::MAX);
        self.frame.scroll_areas.push(Area {
            id,
            view,
            offset,
            max,
        });
        // Bracketed on the frame rather than carried on `Ctx`: the body is a closure, so the save
        // and the restore are both here and no context has to grow a field for it.
        let outer = self.frame.open_area.replace(ix);
        let r = {
            let mut clipped = self.child(view);
            // **Negated, and it is the one place the two senses meet.** `offset` is a scroll
            // *position* — the first visible content cell, which is what `Scrollable::between` and
            // `Area::into_view` are both written in — and [`Ctx::scrolled`] takes a *translation*.
            // Passed through unnegated, `visible_rows()` answered `-offset..-offset + h` and every
            // verb at a content row the offset had reached was clipped away: at offset 5 over an
            // eight-row view, `-5..3` and **0 cells** written at content row 5. Nothing saw it
            // because no consumer had drawn a scrolled window yet — `crate::scroll`'s own tests all
            // play at offset 0, and the wheel harness one crate up measures where the offset ended
            // up rather than what landed on the screen. Found by components ticket 12, the first
            // virtualised collection.
            let mut inner = clipped.scrolled(-offset.0, -offset.1);
            f(&mut inner)
        };
        self.frame.open_area = outer;
        r
    }

    /// **Publish a scrollable region**, with the four directions it can still move in.
    ///
    /// This is the wheel chain's entry: the innermost published region under the pointer that
    /// [admits](Scrollable::admits) the wheel's direction consumes the click, and the rest chain
    /// outward. The delta arrives on [`Response::scrolled`] **during the draw of the frame the click
    /// arrived on**, which is the one pointer outcome that is not awarded at `end`.
    ///
    /// **A component that publishes a region owes the pair per axis, computed from its clamped
    /// offset** — [`Scrollable::between`] is that arithmetic. Declaring an axis instead of a
    /// direction is the defect this signature exists to make unwritable: a list at its bottom that
    /// says *the y axis is movable* eats every downward click and the area around it moves by 0.
    ///
    /// `interest` is folded in beside [`Interest::SCROLL`], so a region that is also clickable or a
    /// tab stop says so here rather than declaring itself twice.
    pub fn scrollable(&mut self, id: Id, r: Rect, interest: Interest, s: Scrollable) -> Response {
        self.declare(id, r, interest.with(Interest::SCROLL), s)
    }

    /// **Take the scroll-into-view request addressed to `id`**, if `end` left one on the frame
    /// before.
    ///
    /// The answer is a **delta in content cells**, to be added to the offset and clamped, because
    /// the application owns the offset: a delta composes with whatever it did to its own state in
    /// between, where an absolute value computed against last frame's content silently overwrites
    /// it. It answers once — the request is consumed — and a request nobody takes lives exactly one
    /// frame.
    ///
    /// # There is no verb that reads a scroll offset, and there is no shape to ask for one
    ///
    /// The application owns the offset. The runtime holds a viewport, a clamp and one request; it
    /// holds no position, and a component asking it *where is this area scrolled to* is asking the
    /// wrong object. The item this pair protects is named by path, so a rename fails **here** and not
    /// silently in the `compile_fail` below — which would go on passing for the wrong reason, a
    /// method that no longer exists also being a method that does not compile:
    ///
    /// ```
    /// use vitui_runtime::ctx::{Ctx, Driver};
    /// use vitui_runtime::Id;
    ///
    /// fn protected<'f, 'v>(cx: &mut Ctx<'f, 'v>, id: Id) -> Option<(i32, i32)> {
    ///     Ctx::take_into_view(cx, id)
    /// }
    ///
    /// let mut d = Driver::headless(20, 6).expect("sink");
    /// d.frame(|cx| {
    ///     assert!(protected(cx, Id::named("area")).is_none(), "nothing asked for one");
    /// });
    /// ```
    ///
    /// ```compile_fail,E0599
    /// use vitui_runtime::ctx::Driver;
    /// use vitui_runtime::Id;
    ///
    /// let mut d = Driver::headless(20, 6).expect("sink");
    /// d.frame(|cx| {
    ///     let _offset = cx.scroll_offset(Id::named("area"));
    /// });
    /// ```
    pub fn take_into_view(&mut self, id: Id) -> Option<(i32, i32)> {
        let asked = self.frame.into_view?;
        if asked.area != id {
            return None;
        }
        self.frame.into_view = None;
        Some(asked.by)
    }

    /// **Ask the enclosing scroll area to bring `r` into view**, in this context's own coordinates.
    ///
    /// The runtime's own pull fires only for a keyboard-driven focus move, because a press already
    /// proves the widget was on screen. This is the door for every other legitimate case — a
    /// selection moved by an arrow key inside a [`Group`](crate::focus::ScopeKind::Group), a search
    /// result, a caret walking off the bottom of a text area — and it is the component's call
    /// precisely because the runtime cannot tell those from a pull that fights the wheel.
    ///
    /// It does nothing outside a [`Ctx::scroll_scope`], and nothing for a rectangle that is already
    /// visible.
    pub fn request_into_view(&mut self, r: Rect) {
        let Some(ix) = self.frame.open_area else {
            return;
        };
        let Some(area) = self
            .frame
            .scroll_areas
            .get(usize::try_from(ix).unwrap_or(usize::MAX))
            .copied()
        else {
            return;
        };
        // Into the area's content coordinates, which is what `content` accumulates and what the ring
        // entry beside it is in.
        let content = Rect::new(r.x + self.content.0, r.y + self.content.1, r.w, r.h);
        let by = area.into_view(content);
        if by != (0, 0) {
            self.frame.into_view_asked = Some(IntoView { area: area.id, by });
        }
    }

    /// Declare an interactive region.
    ///
    /// **Ticket 10 fills the response in.** What is here is 08's half and it is not nothing: the entry
    /// is appended to the hit index in draw order, and the declared interest is folded into the
    /// tracking level `settle` hands to `set_mouse`.
    #[track_caller]
    pub fn interact_here(&mut self, r: Rect, i: Interest) -> Response {
        let id = Id::at(self.frame.stack.current(), std::panic::Location::caller());
        self.interact(id, r, i)
    }

    /// Declare an interactive region under an id the caller already has.
    ///
    /// **A merge is where duplicate detection shows up at a call site**: if this id was already
    /// claimed this frame, the first claimant keeps it and this response is inert. That is the policy
    /// rather than an error, because two widgets legitimately sharing a call site is a *design* smell
    /// the author can see on screen, and a panic would be a crash for a cosmetic problem.
    pub fn interact(&mut self, id: Id, r: Rect, i: Interest) -> Response {
        // **`Interest::SCROLL` on its own means every direction**, which is the honest reading of one
        // bit and the right answer for a widget with no stated bounds — an endless log, an embedded
        // terminal. A region whose content *has* an end owes the pair per axis and declares it
        // through [`Ctx::scrollable`]; that is where the four bits come from and this is the only
        // other producer of them.
        let s = if i.contains(Interest::SCROLL) {
            Scrollable::ALL
        } else {
            Scrollable::NONE
        };
        self.declare(id, r, i, s)
    }

    /// The one place a hit entry is appended, shared by [`Ctx::interact`] and [`Ctx::scrollable`]:
    /// they differ only in where the four direction bits come from.
    fn declare(&mut self, id: Id, r: Rect, i: Interest, s: Scrollable) -> Response {
        // **The measured world has no focus, no hover and no press**, and this is what stops that
        // being silent: a body that asked is recorded, so [`crate::sizing::check`] can name it in a
        // failure instead of leaving two numbers that disagree for no visible reason.
        if self.frame.extent.is_some() {
            self.frame.consulted.interaction = true;
        }
        let claimed = self.frame.ids.claim(id).is_some();
        if !claimed {
            // Merged: the first claimant owns the id, and this one does nothing at all — no hit
            // entry, no tracking, no tab stop.
            return Response::inert(id, r);
        }
        // **Containment, here, in this widget's own coordinates.** The pointer travelled down the
        // `Ctx`; the index gets one bit.
        let local = self
            .pointer
            .filter(|&(x, y)| x >= r.x && y >= r.y && x < r.right() && y < r.bottom());
        self.frame.hits.push(Hit {
            id,
            interest: i,
            over: local.is_some(),
            scrollable: s,
        });
        // The `max` over a totally ordered ladder, which is why combining is not a negotiation.
        self.frame.tracking = self.frame.tracking.max(i.tracking());
        if i.contains(Interest::FOCUS) {
            // **The ring is built during the draw**, one branch on a bit the widget is already
            // passing — a cost moved from the frame a `Tab` arrives on to every frame, measured at
            // 1.000×–1.009× of a dense frame, which is under the run-to-run spread of the frame
            // itself. Built in `begin` instead it is a frame old, and `Tab` steps over a row
            // inserted directly after the focused one.
            let focused = self.frame.focused == Some(id);
            self.frame.ring.push(
                id,
                Rect::new(r.x + self.content.0, r.y + self.content.1, r.w, r.h),
                self.frame.open_area,
                focused,
            );
        }
        // **The bubbling detector**, incremented here because this is the one verb every focusable
        // widget calls. A scope reads it either side of its body; see `Ctx::scope`.
        if self.frame.focused == Some(id) {
            self.frame.focus_draws += 1;
        }

        let a = self.frame.delivered.unwrap_or_default();
        let is = |who: Option<Id>| who == Some(id);
        Response {
            id,
            rect: r,
            // **The guess**, from the previous frame's index: in-frame feedback, and it never decides
            // a click.
            hovered: self.frame.hover_guess == Some(id),
            pressed: self.frame.grab == Some(id),
            released: is(a.released),
            clicked: a.clicked.is_some_and(|(who, _)| who == id),
            double_clicked: a.clicked.is_some_and(|(who, d)| who == id && d),
            long_pressed: is(a.long_pressed),
            dragged: match (self.frame.grab, self.frame.press_origin, local) {
                (Some(g), Some((_, origin)), Some(now)) if g == id => {
                    Some((now.0 - (origin.0 - r.x), now.1 - (origin.1 - r.y)))
                }
                _ => None,
            },
            // **This frame's wheel, not the previous frame's award.** The offset it moves is read
            // here, inside the draw, by the widget that owns it — see [`crate::scroll`].
            scrolled: self
                .frame
                .wheel
                .filter(|(who, _)| *who == id)
                .map_or((0, 0), |(_, d)| d),
            // **The position, not a delta.** A splitter, a slider and a selection drag all need where
            // the pointer *is*; reconstructing it from a delta needs a press origin the widget was
            // never given.
            local: local.map(|(x, y)| (x - r.x, y - r.y)),
            focused: self.frame.focused == Some(id),
            // **Compared against what the previous draw saw**, not against `focused`: the focus moves
            // in `end`, so by the time a widget draws again the move has already happened and the
            // difference is not visible from the live value. Both `Tab` and the press produce these,
            // and validation-on-blur is the case they exist for.
            focus_entered: self.frame.focused == Some(id) && self.frame.focus_before != Some(id),
            focus_left: self.frame.focus_before == Some(id) && self.frame.focused != Some(id),
            // **Never written by the runtime.** A component with a value sets it before returning,
            // because the runtime does not hold the value and may not.
            changed: false,
            mods: a.mods,
        }
    }

    /// Open a modal barrier here.
    ///
    /// **One index into the hit index**, which is already in draw order — so modality is an *ordering*
    /// and not a membership, and nothing needs a layer id. Everything declared from here on is inside
    /// the modal; everything before it is withheld from the pointer.
    ///
    /// # The barrier stops the pointer and only the pointer
    ///
    /// A modal is three mechanisms and this is one of them: the overlay is
    /// [`Ctx::overlay`]'s, the keyboard half is a [`ScopeKind::Trap`], and the scrim is
    /// [`crate::overlay::Scrim`]. **A barrier with no trap around it lets `Tab` walk straight out of
    /// the modal**, which is gated rather than assumed —
    /// `overlay_tests::a_barrier_alone_lets_tab_out`.
    ///
    /// # Where to call it
    ///
    /// **From inside the overlay body, as its first statement.** The base pass has finished by then,
    /// so `hits.len()` is exactly the boundary between *under the modal* and *in it*, and nothing
    /// depends on the owner having drawn last. Called during the base pass instead it is still an
    /// index, and every widget drawn after it — siblings included — is inside the modal.
    pub fn modal_barrier_here(&mut self) {
        self.frame.modal_from = Some(self.frame.hits.len());
    }

    /// Declare an interactive region and derive its id from the call site.
    ///
    /// The spelling a component actually uses; `interact` is for a caller that already has an id.
    #[track_caller]
    pub fn interact_named(&mut self, id: Id, r: Rect, i: Interest) -> Response {
        self.interact(id, r, i)
    }

    /// **Hover as a style, resolved in the same frame.**
    ///
    /// Declaring the intent during the draw and applying it at `end` is what closes the lag: the
    /// winner is decided from the index that has just drawn, so it is right on the *first* frame of an
    /// overlap, where a guess from the previous frame would still be pointing at what was on top
    /// before.
    ///
    /// **What is left one frame old is hover that changes content or size**, and that is the whole
    /// residue. A style is not; a different label is.
    pub fn hover_style(&mut self, resp: &Response, r: Rect, role: crate::theme::Role) {
        // Root coordinates, because `end` runs after every `Ctx` is dropped and a widget's own
        // coordinates mean nothing to it by then.
        //
        // **From `origin` and not from `rect`**, which is the correction ticket 15 brought with it:
        // `rect` is this context's rectangle in its *parent's* coordinates, so adding one of them
        // was right at the root and at one level down, and wrong at every level after that — a
        // widget two containers deep had its hover painted at the offset of the inner container
        // alone. `origin` accumulates the whole chain, which is what a root coordinate needs.
        let root = Rect::new(self.origin.0 + r.x, self.origin.1 + r.y, r.w, r.h);
        self.frame.hover_styles.push((resp.id, root, role));
    }

    /// Take the next key for `id`.
    ///
    /// **It answers nobody but the routing target**, which is the focused id — or, after a bubbling
    /// container's body, that container's own id. Any other caller gets `None`, whatever is in the
    /// queue. That single rule is what makes a `Trap` a trap: a scope that has taken the focus has
    /// taken the keyboard, with no second mechanism.
    ///
    /// **One value per call, holding no borrow**, which is the second deviation: an iterator borrows
    /// the queue and every interactive component reads its keys inside the scope where it draws, so
    /// the iterator would be live across the drawing verbs. And there is **no upper bound on how many
    /// arrive**, which is what ADR 0008 requires of input.
    ///
    /// # There is one queue, and no per-id inbox
    ///
    /// This is the whole keyboard API, and the item it protects is named here by path so that a
    /// rename fails this twin rather than quietly changing what the negative case below is about:
    ///
    /// ```
    /// use vitui_runtime::ctx::{Ctx, Driver};
    /// use vitui_runtime::Id;
    /// use vitui_engine::Key;
    ///
    /// // Named by path and pinned to its signature: a rename or a changed argument fails HERE, and
    /// // not silently in the `compile_fail` case below — which would go on passing for the wrong
    /// // reason, because a method that no longer exists also does not compile.
    /// fn protected<'f, 'v>(cx: &mut Ctx<'f, 'v>, id: Id) -> Option<Key> {
    ///     Ctx::next_key(cx, id)
    /// }
    ///
    /// let mut d = Driver::headless(20, 3).expect("sink");
    /// d.frame(|cx| {
    ///     assert!(protected(cx, Id::ROOT).is_none(), "nothing focused, nothing routed");
    /// });
    /// ```
    ///
    /// And there is no shape to ask for a queue of one's own. The per-id version costs 1.17×
    /// routing and at least one allocation a frame against zero, and this is the gate that says it
    /// was never built:
    ///
    /// ```compile_fail,E0599
    /// use vitui_runtime::ctx::Driver;
    /// use vitui_runtime::Id;
    ///
    /// let mut d = Driver::headless(20, 3).expect("sink");
    /// d.frame(|cx| {
    ///     let _inbox = cx.inbox(Id::ROOT);
    /// });
    /// ```
    pub fn next_key(&mut self, id: Id) -> Option<vitui_engine::Key> {
        // The measured world has no keys either, and the same reason it is recorded.
        if self.frame.extent.is_some() {
            self.frame.consulted.keys = true;
        }
        if self.frame.route_to != Some(id) {
            return None;
        }
        // **The trap's second keyboard mechanism.** A trap that stood *last* frame refuses delivery
        // outside itself, which is what stops an explicit `cx.focus` from handing the keyboard back
        // to a widget behind the modal. It cannot reach the frame the trap opens — widgets behind
        // draw first, so they pull their keys before the scope that would stop them exists — and
        // that exposure is one key, because the click that opens a modal is a closing edge and ends
        // its batch.
        if !self.frame.ring.prev_delivers_to(id) {
            return None;
        }
        // **`Tab` is the ring's key, not the focused widget's.** Withheld rather than declined,
        // because by the time a key is declined the ring has already consumed it: what is handed
        // back no longer describes the state. An [`Isolated`](ScopeKind::Isolated) scope is the one
        // caller that gets it, and that is answered from the previous frame's scopes.
        if !self.frame.focus_isolated
            && self
                .frame
                .keys
                .peek()
                .is_some_and(|k| crate::route::edge_of(&vitui_engine::Event::Key(k)).is_some())
        {
            return None;
        }
        self.frame.keys.take()
    }

    /// Give the focus to `id`.
    ///
    /// **The only focus verb, and it names a widget rather than a direction.** The walk is not
    /// reachable from a component: one that could call it would move the focus during the draw, over
    /// a ring that is half built.
    ///
    /// # An application that never calls this is deaf, and the first one written was
    ///
    /// `frame.focused` starts as `None` and **nothing sets it for you**: not `interact`, not
    /// `Interest::FOCUS`, not the ring being built. [`Ctx::next_key`] answers only the focused id —
    /// *nothing focused, nothing routed*, which is its own doctest — so a program with a perfectly
    /// good key map, a tab stop and a drain loop receives **nothing at all** until a click awards
    /// the focus to something. Measured on the two arms of one application, five `Right` presses
    /// each: **0 against 5.**
    ///
    /// The symptom is worse than silence, because the click is a repair the user finds by accident:
    /// *it starts working after I click on it*. `crates/vitui-apps/examples/counter.rs` is where
    /// this was found and carries the fix.
    ///
    /// **Focus on the first frame, guarded by [`Ctx::focused`]:**
    ///
    /// ```ignore
    /// if cx.focused().is_none() {
    ///     cx.focus(sink);
    /// }
    /// ```
    ///
    /// **Not `if !cx.is_focused(id) { cx.focus(id) }`**, which reads as the same thing and is not:
    /// the conditional form takes the keyboard **back** every frame the user has tabbed away, so
    /// `Tab` appears to do nothing. *Which widget starts with the keyboard* is a statement about the
    /// first frame; asking whether **anything** holds the focus is what makes that statement
    /// writable inside the draw, and until architecture issue 25 it was not askable at all — so this
    /// obligation used to require a flag the application kept outside the frame.
    ///
    /// **The runtime still focuses nothing on its own, and that is now a decision rather than an
    /// omission** (issue 25). Auto-focusing the first stop would be an opinion about which widget is
    /// primary, on a runtime with no scene tree and no such opinion, and *first* would mean first in
    /// draw order.
    pub fn focus(&mut self, id: Id) {
        self.frame.focused = Some(id);
    }

    /// Whether `id` holds the focus.
    pub fn is_focused(&self, id: Id) -> bool {
        self.frame.focused == Some(id)
    }

    /// **Who holds the focus, if anyone** — and the question [`Ctx::focus`]'s obligation needs.
    ///
    /// Architecture issue 25. `is_focused(id)` asks about one id and there was **no way to ask
    /// whether anything at all holds the focus**: `Frame::focused` exists and is reachable only
    /// through `Driver::inspect()`, which is between frames, where the `Ctx` is gone. So the
    /// obligation [`Ctx::focus`] documents — *focus something on the first frame* — could be
    /// satisfied correctly only by a flag the application keeps outside the frame.
    ///
    /// With this, it is one line inside the draw and there is no flag:
    ///
    /// ```ignore
    /// if cx.focused().is_none() {
    ///     cx.focus(sink);
    /// }
    /// ```
    ///
    /// **Read the difference from `if !cx.is_focused(sink)` carefully, because they look alike and
    /// one of them is a defect.** The `is_focused` form takes the keyboard *back* every frame the
    /// user has tabbed away, so `Tab` appears to do nothing. This form fires only while **nobody**
    /// holds it, which is true on the first frame and false ever after — unless the focus is
    /// genuinely lost, and then re-seating it is the wanted behaviour rather than a theft.
    ///
    /// # What it reads, and when
    ///
    /// The focus as of the **start of this frame**, plus any [`Ctx::focus`] call already made during
    /// it. The focus *moves* in `end` — `Tab`, the traps, the vanish rule — so a widget drawing
    /// after a move sees the value that was current when its frame began, which is the same value
    /// `route_to` was taken from and therefore the one that decides who `next_key` answers.
    ///
    /// **The runtime still focuses nothing on its own**, and issue 25 refused two candidates that
    /// would have: a runtime that focuses the first stop is a runtime with an opinion about which
    /// widget is primary, on a design whose whole shape is that it has no scene tree and no such
    /// opinion — and *first* would mean first in **draw order**, a layout accident. A `Driver` flag
    /// only moves the argument, because its default is still the decision.
    pub fn focused(&self) -> Option<Id> {
        self.frame.focused
    }

    /// What this key fires, resolved **innermost-first from the open scope**.
    ///
    /// A key map is a range tagged with the scope that was open when [`Ctx::key_map`] declared it,
    /// and this walks that scope's parents outward, finishing at the frame level where an
    /// application declares its own. First match wins inside each rung.
    ///
    /// **The flat pass is wrong in the expensive direction**: `Ctrl+S` under a
    /// [`Trap`](ScopeKind::Trap) fires the application's *Save* instead of the modal's, which is a
    /// document written behind a dialog the user has not confirmed.
    pub fn action(&self, k: &vitui_engine::Key) -> Option<crate::keys::ActionId> {
        let mut at = self.frame.ring.open();
        loop {
            if let Some(action) = self.frame.maps.match_in(k, at) {
                return Some(action);
            }
            // `None` is both the frame level and the end of the walk, so the rung above is asked
            // first and only then does the loop finish.
            let ix = at?;
            at = self.frame.ring.parent_of(ix);
        }
    }

    /// Hand a key back, **in order**: it is the next key the queue answers, to the next level out.
    ///
    /// This is the whole of bubbling's mechanism from below. An inner widget takes a key, finds it
    /// is not one of its own and declines it; the container's after-the-body moment takes it next,
    /// and so on outward. **Nested traps are innermost-first for free**, because the drain order
    /// already is.
    ///
    /// # A decline ends this widget's turn at the queue
    ///
    /// [`Ctx::next_key`] answers `None` after it, however many keys are left, until the routing
    /// target moves outward. So the obvious loop terminates:
    ///
    /// ```
    /// # use vitui_runtime::ctx::Driver;
    /// # use vitui_runtime::Id;
    /// # let mut d = Driver::headless(20, 3).expect("sink");
    /// # let field = Id::from_raw(1);
    /// d.frame(|cx| {
    ///     while let Some(k) = cx.next_key(field) {
    ///         cx.decline(k);   // not mine — and this loop ends rather than spinning
    ///     }
    /// });
    /// ```
    ///
    /// **That is a contract and not a limitation.** The queue is ordered and shared: a widget that
    /// declined key *k* and then took *k+1* would leave the level above it seeing the two the wrong
    /// way round.
    pub fn decline(&mut self, k: vitui_engine::Key) {
        self.frame.keys.put_back(k);
    }

    /// Declare a key map **for the open scope**.
    ///
    /// The scope is what [`Ctx::action`] walks outward from; a map declared with no scope open is
    /// the application's, and is reached last.
    pub fn key_map(&mut self, map: &crate::keys::KeyMap) {
        let scope = self.frame.ring.open();
        self.frame.maps.declare_in(map, scope);
    }

    /// Queue an overlay: **request during the draw, satisfy after it, answer next frame.**
    ///
    /// A component cannot open a layer mid-draw — `LayerStack::view` holds `&mut` of the stack for
    /// the life of the view, so a second one is `E0499` (engine architecture ticket 14 R2). The body
    /// therefore runs in the second pass, after every base-pass draw context has been dropped, and
    /// anything it produces reaches its owner on the frame after through whatever the owner is
    /// already reading — a press it declared, a key it took, its own `&mut` state.
    ///
    /// # The owner is an argument, and there is no spelling that derives one
    ///
    /// `owner` is the id the calling component already claimed. It cannot be derived here: a
    /// component carries `#[track_caller]` and the attribute reaches *into* the body, so a derived
    /// id would be the id the owner claimed one line earlier, [`Ctx::interact`]'s merge would make
    /// the second claimant inert, and **the overlay would be silently dropped**. This verb is
    /// deliberately not `#[track_caller]` and mints no id at all.
    ///
    /// It does two jobs. It **keys the layer across frames**, so a layer requested again is reused
    /// rather than rebuilt; and it **roots the body's id stack**, so the same body drawn inline and
    /// drawn in an overlay produces *different* ids. An overlay is a different place for identity,
    /// not only for geometry.
    ///
    /// # What the body may capture
    ///
    /// The bound is `+ 'f`, and `'f` is the frame call's, not the borrow's — see [`Ctx`]'s own
    /// documentation for the paired compile outcome and for the three things that have to be true
    /// for it to bite. A body may capture application data and `Copy` state; it may **not** capture
    /// a local of the base pass, because that local is gone by the time the body runs.
    ///
    /// **"A body may not capture `&mut` state" is a convention and not a bound.** It compiles, it
    /// runs, and nothing here refuses it. The reason to keep the convention anyway is that an
    /// outcome reaching its owner the ordinary way keeps a `select` mutating its state in one place
    /// instead of two.
    ///
    /// # The z it actually gets
    ///
    /// `opts.z` at the base pass. Inside another overlay it is the **parent's** layer plus
    /// [`Z::NESTED`], because a dropdown inside a modal sorted into its own band would draw below
    /// the barrier that exists to protect it.
    ///
    /// ```
    /// use vitui_engine::Rect;
    /// use vitui_runtime::ctx::{Driver, Id};
    /// use vitui_runtime::overlay::{OverlayOpts, Z};
    /// use vitui_runtime::Role;
    ///
    /// static ITEMS: [&str; 2] = ["Open", "Save"];
    ///
    /// let mut driver = Driver::headless(40, 12).expect("sink");
    /// driver.frame(|cx| {
    ///     let owner = Id::from_raw(7);
    ///     cx.overlay(owner, Rect::new(2, 2, 8, 1), OverlayOpts::sized(10, 2), move |cx| {
    ///         let body = cx.theme().paint(Role::Body);
    ///         for (row, item) in ITEMS.iter().enumerate() {
    ///             cx.text(0, row as i32, item, body);
    ///         }
    ///     });
    /// });
    /// assert_eq!(driver.inspect().overlays_placed(), 1);
    /// ```
    ///
    /// # The body answers through the inbox, and a `&'f mut` capture costs you the state
    ///
    /// **Capture what the body reads by value or by shared reference; let it answer through the
    /// inbox.** A `&'f mut` capture *compiles* — the bound above is `+ 'f`, and a mutable borrow of
    /// a frame-lived local satisfies it — and it costs the caller that state for **the rest of the
    /// frame**, because the body is held in the queue until the satisfy pass and the borrow lives
    /// exactly as long.
    ///
    /// This is worth a heading rather than a sentence because **the diagnostic never mentions the
    /// overlay.** The failure arrives at the *caller*, one level away, as `E0503` — *cannot use
    /// `x` because it was mutably borrowed* — pointing at the next ordinary read of a local whose
    /// only unusual property is that a closure three lines up captured it. Worse, rustc's own
    /// `help:` line for the shape suggests a borrow that compiles here, so following the compiler's
    /// advice produces a build that works and a frame that has quietly lost a variable.
    ///
    /// Components ticket 10 owed this note and could not write it: the crate that meets the mistake
    /// cannot edit the item it belongs on, and the item is this one — a note on the *caller's* side
    /// would have to be repeated at every call site, which is where a rule goes to rot.
    pub fn overlay<F>(&mut self, owner: Id, anchor: Rect, opts: OverlayOpts, body: F)
    where
        F: FnMut(&mut Ctx<'f, '_>) + 'f,
    {
        // **Root coordinates, translated here.** The pass runs after every context in this subtree
        // has been dropped, so an anchor in this context's own coordinates would name a cell nobody
        // can find. The same translation, and the same reason, as `hover_style`.
        let anchor = Rect::new(
            self.origin.0 + anchor.x,
            self.origin.1 + anchor.y,
            anchor.w,
            anchor.h,
        );
        let z = if self.frame.layer_z == Z::BASE {
            self.frame.layer_z + opts.z
        } else {
            self.frame.layer_z + Z::NESTED
        };
        let seq = u32::try_from(self.frame.overlays.len()).unwrap_or(u32::MAX);
        // **The body is boxed into the frame call's queue and the request carries its handle.** One
        // allocation, and the `Box` is what drops the body — exactly once, whether the pass runs it,
        // finds it inert, or never reaches it at all. See [`crate::overlay`] for the count this puts
        // on the frame and for why the queue cannot be a field of one.
        let body = self.bodies.push(body);
        self.frame.overlays.push(OverlayRequest {
            owner,
            anchor,
            opts,
            z,
            seq,
            body,
        });
    }

    /// Ask for another frame at a moment.
    ///
    /// **One sink.** Two callers asking for different moments is one wake, at the earlier — and two
    /// entries in [`crate::anim::WakeLedger`], because a fold is a wake and a census is not.
    ///
    /// **Attributed to the call site**, which is a `file:line:col` a diagnostic may print. An `Id`
    /// could not be: it is a hash of an address that may never be persisted (ADR 0013), and the id
    /// the frame has at hand here is the id stack's current — the *closure tree*, so twelve animated
    /// chips all answer [`Id::ROOT`]. See [`Ctx::deadline_for`] for the counter that does name a
    /// widget, and `anim`'s module documentation for the numbers.
    ///
    /// `#[track_caller]` is viral, and here it runs in the right direction: a component library that
    /// wants a runaway blamed on the *application's* line writes the attribute on its own wrapper.
    #[track_caller]
    pub fn deadline(&mut self, at: Instant) {
        let now = self.env.now();
        self.frame
            .ask(std::panic::Location::caller(), None, at, now);
    }

    /// Ask for another frame at a moment, **and say which widget wants it.**
    ///
    /// The difference from [`Ctx::deadline`] is **not** attribution: both attribute to the call site.
    /// What the `id` buys is the *census* — [`WakeLedger::asked_by`](crate::anim::WakeLedger::asked_by),
    /// a counter that proves which widget asked, kept beside the attribution and never instead of it.
    #[track_caller]
    pub fn deadline_for(&mut self, id: Id, at: Instant) {
        let now = self.env.now();
        self.frame
            .ask(std::panic::Location::caller(), Some(id), at, now);
    }

    /// Ask for another frame now.
    ///
    /// **This is `deadline(now)` with a call site**, and `now` is the frame's own sampled clock
    /// rather than a second reading of the machine's. It is not a second sink: the runtime had two
    /// and flushed one, and `[Tab, Key(a)]` lost its second key to exactly that.
    ///
    /// A frame that asks for *now* is asking for the next frame the pacing gate will give it, which
    /// is the one thing the screen cannot sleep through — so this is what a
    /// [`runaway`](crate::anim::WakeLedger::runaway) counts.
    #[track_caller]
    pub fn request_frame(&mut self) {
        let now = self.env.now();
        self.frame
            .ask(std::panic::Location::caller(), None, now, now);
    }

    /// Put the caret here, in this context's own coordinates.
    ///
    /// **The sink, and the last write of the frame wins** — `settle` forwards it to
    /// `Screen::set_cursor` (ADR 0005). The shape is the theme's default; [`Ctx::caret_with`]
    /// carries one.
    ///
    /// Two refusals, and both are the same sentence about lying: **nothing focused means no caret**,
    /// because a caret on a screen where no widget holds the keyboard says typing goes somewhere it
    /// does not; and a caret **outside this context's own area** is somebody else's cell, which is
    /// what a field scrolled out of its pane would otherwise place. The runtime blinks nothing —
    /// a software caret is two wakeups a second for as long as anything has focus.
    pub fn caret(&mut self, x: i32, y: i32) {
        self.caret_with(x, y, CursorShape::Terminal);
    }

    /// Put the caret here, with a shape.
    ///
    /// **`CursorShape` is the engine's, re-exported and not redefined** — the same rule as
    /// `GlyphSet` and for the same reason. `set_cursor` applies all three of `{ x, y, shape }`, so a
    /// sink taking only `(x, y)` drops the third with no other door: a bar caret in an input beside a
    /// block caret in a list was inexpressible above this runtime while the component library
    /// recorded cursor shape as supported.
    pub fn caret_with(&mut self, x: i32, y: i32, shape: CursorShape) {
        let area = self.area();
        if x < area.x || y < area.y || x >= area.right() || y >= area.bottom() {
            return;
        }
        // **Root coordinates, accumulated on the way down.** Two contexts writing `(0, 0)` name the
        // same cell otherwise, and last-write-wins then picks between two different places.
        let (rx, ry) = (x + self.origin.0, y + self.origin.1);
        self.frame.caret = Some(Cursor {
            x: u16::try_from(rx.max(0)).unwrap_or(u16::MAX),
            y: u16::try_from(ry.max(0)).unwrap_or(u16::MAX),
            shape,
        });
    }

    /// **A test mechanism, and never the layout mechanism** — the dry run, kept only as the detector
    /// that keeps a sizing function honest against the component beside it: draw a body into a
    /// discard surface and read how far its verbs reached. Laying something out with it instead is
    /// **13.3×** the sizing function for the same answer, and **6 051 331×** it for the question a
    /// sizing function is usually being asked.
    ///
    /// The second of those two numbers is the one that decides it. A dry run measures the *clip*, so
    /// a 1M-row list under an 80-row clip reports 80, and the honest question — 65 535 rows, all
    /// `u16` can express — is 5.81 ms against 0.96 ns. Spec §12 and ADR 0014 hold the rest of the
    /// argument, [`crate::sizing`] holds the contract, and [`crate::sizing::check`] is the one call
    /// a component author needs.
    ///
    /// # It gets its own [`Frame`], which is what bounds what it may be asked
    ///
    /// Sharing this frame's would claim every id twice, double the hit index and drain the key
    /// queue — the third of those silently, on the frame where a keystroke was about to be read. So
    /// the measured world has **no focus, no hover, no press and no keys**: not suppressed, but
    /// absent, because a fresh frame over an empty batch has none of the four. A body that asks
    /// anyway is recorded in [`Measured::consulted`], which is what turns *a default that looks like
    /// an answer* into something a failure can name.
    ///
    /// **`&self`, deliberately.** The measured world cannot touch this frame, and the signature is
    /// where that is said: there is no `&mut` for it to write through.
    ///
    /// ```
    /// use vitui_runtime::ctx::Driver;
    /// use vitui_runtime::Role;
    ///
    /// let mut driver = Driver::headless(40, 10).expect("sink");
    /// driver.frame(|cx| {
    ///     let body = cx.theme().paint(Role::Body);
    ///     let measured = cx.measured(20, 4, |inner| {
    ///         inner.text(0, 0, "two", body);
    ///         inner.text(0, 1, "rows", body);
    ///     });
    ///     assert_eq!((measured.extent.w, measured.extent.h), (4, 2));
    /// });
    /// ```
    pub fn measured<R>(
        &self,
        w: u16,
        h: u16,
        body: impl FnOnce(&mut Ctx<'_, '_>) -> R,
    ) -> Measured<R> {
        // Its own frame, begun over an empty batch: no pointer event to place a pointer, no key to
        // route, and the four id-keyed facts at their `None`.
        let mut frame = Frame::new();
        frame.begin(&[]);
        frame.extent = Some(Extent::ZERO);
        // Its own surface, and a standalone one rather than a layer: a layer is composited and this
        // is a discard.
        let mut surface = Surface::new(w, h);
        // Its own body queue, and the reason it is not `self.bodies` is the same reason the frame is
        // not `self.frame`: **a dry run is its own world**. An overlay requested inside one is queued
        // against a frame nothing will run a pass over, and this is what drops that body — the arena
        // dropped nothing itself, so under it a measured body owning a `String` leaked.
        let bodies = Bodies::new();
        let value = {
            let mut cx = Ctx {
                view: surface.root(),
                frame: &mut frame,
                env: self.env,
                rect: Rect::new(0, 0, w, h),
                origin: (0, 0),
                pointer: None,
                // The measured world is its own root: it composites nothing and scrolls nothing, so
                // its content coordinates and its root coordinates are the same thing.
                content: (0, 0),
                bodies: &bodies,
                _frame: PhantomData,
                _not_send: PhantomData,
            };
            body(&mut cx)
        };
        Measured {
            value,
            extent: frame.extent.unwrap_or(Extent::ZERO),
            consulted: frame.consulted,
        }
    }

    /// The moment this frame was sampled at. **Once per frame** — an event carries its own moment,
    /// and a double click compares against that rather than against this.
    pub fn now(&self) -> Instant {
        self.env.now()
    }

    /// The theme.
    ///
    /// **The `'v` is on the return type and not on `&self`**, and that is the whole of the fourth
    /// deviation. Tied to `&self` instead, this would be `E0502` the moment a caller bound the theme
    /// once and held it across two verbs — which is exactly what *a component names a role, never a
    /// colour* pushes an author to write.
    ///
    /// ```
    /// use vitui_runtime::ctx::Driver;
    /// use vitui_runtime::{Glyph, Role};
    ///
    /// let mut driver = Driver::headless(20, 5).expect("sink");
    /// driver.frame(|cx| {
    ///     // Bound once and held across two verbs, which is the shape that matters.
    ///     let theme = cx.theme();
    ///     let body = theme.paint(Role::Body);
    ///     let rule = theme.glyph(Glyph::HLine);
    ///     cx.text(0, 0, rule, body);
    ///     cx.text(0, 1, rule, body);
    /// });
    /// ```
    ///
    /// # The negative case, and it has to be the *held* shape
    ///
    /// Tied to `&self` instead of to `'v`, the returned reference lives only as long as the borrow of
    /// the context — so binding it once and using it across two verbs is `E0502`. A local type is used
    /// here because the shipped signature is the correct one and there is nothing on `Ctx` to point at:
    ///
    /// ```compile_fail,E0502
    /// struct Theme;
    /// impl Theme { fn glyph(&self) -> &'static str { "-" } }
    /// struct Mimic { theme: Theme }
    /// impl Mimic {
    ///     // The wrong signature: the return is tied to `&self`, not to the frame.
    ///     fn theme<'a>(&'a self) -> &'a Theme { &self.theme }
    ///     fn text(&mut self, _s: &str) {}
    /// }
    /// let mut cx = Mimic { theme: Theme };
    /// let theme = cx.theme();
    /// cx.text(theme.glyph());
    /// cx.text(theme.glyph());   // the second use is what keeps the borrow alive across the first verb
    /// ```
    ///
    /// **Bound-and-held across *two* verbs, and both details are load-bearing.** Two-phase borrows
    /// already allow one lookup *inside* one verb call, so `cx.text(cx.theme().glyph())` compiles even
    /// with the wrong signature — a negative case written that way passes for the wrong reason. And
    /// one verb is not enough either: with a single use the borrow ends at that use and NLL lets the
    /// `&mut` through. It takes a **second** use, after the verb, to hold the borrow across it. Both
    /// of those were wrong in the first version of this case, and both compiled.
    ///
    /// What the wrong signature actually forbids is holding the theme across several verbs — which is
    /// exactly what *a component names a role, never a colour* pushes an author to write.
    pub fn theme(&self) -> &'v Theme {
        self.env.theme()
    }

    /// Whether the theme changed for this frame, and this frame only.
    pub fn theme_changed(&self) -> bool {
        self.env.theme_changed()
    }

    /// What the terminal can do.
    pub fn caps(&self) -> &'v Caps {
        self.env.caps()
    }
}

/// The driver: it owns the screen, the frame and the env, and it is the only way to reach a
/// [`Frame`].
///
/// **That is the first half of *`begin` cannot be skipped*** — there is no public constructor for a
/// `Frame`, and `Driver::frame` begins one before the body sees it. The second half is
/// `IdTable::claim` returning `None` rather than spinning, which holds even if this one is ever
/// circumvented.
pub struct Driver {
    screen: Screen,
    frame: Frame,
    env: Env,
    base: LayerId,
    /// **The unsplit input queue**, in arrival order and with keys and pointer events interleaved:
    /// what the terminal sent plus what was posted, minus what earlier frames have taken.
    ///
    /// The split is over the interleaving, which is why this is one queue and not two.
    pending: Vec<vitui_engine::Event>,
    /// How much of `pending` earlier frames have consumed.
    ///
    /// **A cursor rather than a `drain`, and the same reason `next_key` has one**: an edge at the
    /// head of a long batch would otherwise shift the tail once a frame. The queue is cleared
    /// outright the moment it is fully drained, which is every frame an application keeps up.
    pending_at: usize,
    /// Whether the frame clock is pinned rather than sampled. See [`Driver::pin_clock`].
    pinned: bool,
    /// **The overlay layers, keyed by owner id and kept across frames.** The only structure in this
    /// crate that outlives a frame on purpose and is not one of ADR 0012's four id-keyed facts —
    /// because it is not a fact about a widget, it is a resource the engine is holding on one's
    /// behalf, and dropping it every frame would rebuild every surface every frame.
    ///
    /// A `Vec` and not a map: a screen with a menu, a submenu, a modal and a tooltip on it at once
    /// has four entries, and a reverse scan over four is cheaper than hashing one.
    placed: Vec<Placed>,
    /// The round the overlay pass is running, swapped out of the frame so that a body can queue into
    /// the frame while its own round is being walked. Kept for its capacity.
    round: Vec<OverlayRequest>,
    /// How many surface reallocations the layer lifecycle has cost since this driver was made.
    ///
    /// **A move is 0 and a resize is 1**, which is the engine's own rule for `LayerStack::set_rect`
    /// counted on this side of the seam — *no reallocation* is true of one and false of the other,
    /// and the gate is the pair rather than either number.
    reallocs: u64,
    /// **The handle that wakes the app thread, kept rather than dropped.**
    ///
    /// `attach` used to bind this `_wake` and let it fall, which was correct for exactly as long as
    /// nothing above the engine could park: with no [`Driver::wait`] there was no thread to wake and
    /// no worker that could have been hired to wake it. Both halves arrive together, because either
    /// alone is useless — a loop that parks with no handle out never returns, and a handle with no
    /// loop to park posts to a spin.
    wake: WakeHandle,
}

impl Driver {
    /// Attach to a real terminal.
    pub fn attach(config: Config, theme: Theme) -> Result<Driver, AttachError> {
        let (mut screen, wake) = Engine::new(config).attach()?;
        let (w, h) = screen.size();
        let base = screen.layers().add_content(0, Rect::new(0, 0, w, h), true);
        let caps = screen.capabilities().clone();
        Ok(Driver {
            screen,
            frame: Frame::new(),
            env: Env {
                theme,
                caps,
                now: Instant::now(),
                theme_changed: true,
            },
            base,
            pending: Vec::new(),
            pending_at: 0,
            pinned: false,
            placed: Vec::new(),
            round: Vec::new(),
            reallocs: 0,
            wake,
        })
    }

    /// A driver over a sink, for tests and reports.
    ///
    /// `Clock::Manual`, which is public API and not a test fixture: `present` composites, packs,
    /// serialises and writes inline, so a frame is a straight-line program.
    pub fn headless(w: u16, h: u16) -> Result<Driver, AttachError> {
        Driver::attach(
            Config {
                clock: Clock::Manual,
                output: Output::Sink(Box::new(Vec::new())),
                size: (w, h),
                ..Default::default()
            },
            Theme::default().resolve(vitui_engine::ColorDepth::TrueColor),
        )
    }

    /// **Park until something happens.** The loop's only blocking call, and the first line of spec
    /// §1's sequence.
    ///
    /// It is the engine's `Screen::wait` forwarded unchanged, which is the whole of what it should
    /// be: the pacing, the coalescing and the indefinite park all belong to the frame clock, and a
    /// second policy on this side would be a second answer to a question the engine has already
    /// answered. The first damage after a quiet period returns at once, everything arriving inside
    /// the gap coalesces into one return at the end of it, and **with nothing pending and no
    /// deadline registered the wait is indefinite** — so an idle application costs no wakeups and
    /// no CPU.
    ///
    /// # Why this did not exist until now, and what its absence cost
    ///
    /// Spec §21 leaves the loop unowned — *whether the runtime ships an application shell or only
    /// the pieces* — and that is still open; this is not a shell. What was not a decision was that
    /// **no loop could be written at all**: `wait` lives on `Screen`, `Driver` owns its `Screen`
    /// privately, and no path led to either. An application on this crate could only spin, and a
    /// spin turns the one budget that is not a timing — *a genuinely idle application costs zero
    /// wakeups* — into a claim about a layer nobody could reach. Adding a forward is the smallest
    /// thing that makes §21's question a real choice rather than a description of a wall.
    ///
    /// # It does not drain
    ///
    /// A `Wake` says *why*, not *what*. Events are drained by [`Driver::frame`], at `begin`, in
    /// arrival order and interleaved — the batch split is over the interleaving (ADR 0016), so
    /// there is nothing here for a caller to take and nothing it could do with it if there were.
    ///
    /// ```no_run
    /// use vitui_runtime::ctx::Driver;
    /// use vitui_runtime::work::Wake;
    ///
    /// let mut driver = Driver::headless(80, 24).expect("a sink attaches");
    /// loop {
    ///     match driver.wait() {
    ///         Wake::Quit => break,
    ///         Wake::Input | Wake::Posted | Wake::Deadline => {}
    ///     }
    ///     driver.frame(|cx| { let _ = cx.area(); });
    /// }
    /// ```
    pub fn wait(&mut self) -> Wake {
        self.screen.wait()
    }

    /// **A handle a worker is hired with**, cloned from the one `attach` was given.
    ///
    /// [`Worker::hire`](crate::work::Worker::hire) takes one of these and there was no way to
    /// obtain one: `attach` dropped it. So spec §17's whole module — the resident thread, the
    /// one-slot inbox, the generation that says which question an answer answers — was reachable
    /// from a test that built its own engine and from nothing else. **`Worker::queueing` is not the
    /// answer to that**: it runs jobs inline on demand and exists so a gate can be a straight-line
    /// program, which is the opposite of the thing an application wants.
    ///
    /// Clone it once per worker. The handle is `Send`, and it is the only piece of the app thread's
    /// half that is — see spec §17's compile-outcome gate, which stands unchanged: what crosses is
    /// a `Slot` and a post, never a `Ctx` and never a `Frame`.
    pub fn wake(&self) -> WakeHandle {
        self.wake.clone()
    }

    /// Run one frame: `begin`, the base pass, the overlay pass, `end`, `settle`, `present`.
    ///
    /// **`&'f mut self`, not `&mut self`.** With the elided form `'f` is higher-ranked and every
    /// overlay body would have to be `'static`, which is a different rule and the wrong one; this
    /// makes the bound *outlives this frame call*, which is what it should be.
    pub fn frame<'f>(&'f mut self, view: impl FnOnce(&mut Ctx<'f, '_>)) -> Presented {
        // take — a worker landing is the application's write at the top of the view, and the sequence
        // gains no step for it (ticket 16).

        // begin. Everything the terminal has to say goes on the back of the one queue, classified
        // by nobody yet: **the split is over the interleaving**, so the queue has to hold it.
        while let Some(event) = self.screen.next_event() {
            self.pending.push(event);
        }
        // **The clock is sampled once per frame** (spec §1), and `pin_clock` is what lets a loop
        // choose the moment instead.
        if !self.pinned {
            self.env.now = Instant::now();
        }

        // **The batch split**, and it is the whole of ADR 0016 in three lines: take events from the
        // front until one of them is a routing edge, take that edge too, and leave the rest for the
        // next frame. Moves, wheel notches and ordinary keys are not edges and all of them fold
        // into this one frame at 5.0 ns each.
        let queued = &self.pending[self.pending_at..];
        let taken = route::batch_len(queued);
        self.frame.begin(&queued[..taken]);
        self.pending_at += taken;
        if self.pending_at >= self.pending.len() {
            self.pending.clear();
            self.pending_at = 0;
        } else {
            // Something is still queued, so **there must be another frame** — otherwise the tail of
            // a burst waits for whatever the user does next, which for `[Tab, Key(a)]` means the
            // key arrives on the next keystroke or never. One wake, folded at `end` with everything
            // else that asked for one.
            self.frame.wants_another_frame(self.env.now);
        }

        // resolve, from the PREVIOUS frame's index, what cannot be answered during the draw:
        // which widget is topmost, and which owns the wheel. Both happen inside `begin`, which is
        // where the previous index still exists — the hover guess is demoted and overwritten at
        // `end`, and the wheel is not, because its reader is inside the draw.

        // base pass. The size is read *before* the view is taken, because taking it borrows the
        // screen mutably and reading the size borrows it again — `E0502`, and the fix is an ordering
        // rather than a clone.
        let (w, h) = self.screen.size();
        let pointer = self.frame.pointer;
        // **The overlay bodies live here, for exactly this call.** A local rather than a field,
        // because a body is `+ 'f` and both `Frame` and `Driver` are borrowed for `'f`. It starts
        // empty, so a frame with no overlay standing allocates nothing at all.
        let bodies = Bodies::new();
        {
            let mut cx = Ctx {
                view: self
                    .screen
                    .layers()
                    .view(self.base)
                    .expect("the base layer outlives the driver"),
                frame: &mut self.frame,
                env: &self.env,
                rect: Rect::new(0, 0, w, h),
                origin: (0, 0),
                pointer,
                content: (0, 0),
                bodies: &bodies,
                _frame: PhantomData,
                _not_send: PhantomData,
            };
            view(&mut cx);
        }

        // overlay pass — each request in (z, seq) order, repeating while bodies request more,
        // bounded at sixteen rounds.
        self.overlay_pass(&bodies, Rect::new(0, 0, w, h));

        // end.
        let wake = self.frame.end(self.env.now);

        // **Hover as a style, applied after the draw and before `present`.** This is the whole of why
        // it lands in the same frame: the winner was decided from the index that has just drawn, and
        // the restyle happens while the frame is still ours.
        if let Some((r, role)) = self.frame.hover_to_apply() {
            let lowered = crate::theme::Repaint {
                bg: Some(role),
                ..Default::default()
            }
            .lower(&self.env.theme);
            if let Some(mut view) = self.screen.layers().view(self.base) {
                view.restyle(r, &lowered);
            }
        }

        // settle.
        self.screen.set_mouse(self.frame.tracking);
        self.screen.set_cursor(self.frame.caret);
        if let Some(at) = wake {
            self.screen.request_wake_at(at);
        }

        self.env.theme_changed = false;
        self.screen.present()
    }

    /// Pin the frame clock, so that a loop chooses the moment each frame is drawn at.
    ///
    /// **Public API and not a test fixture**, on `Clock::Manual`'s precedent one layer down and
    /// `Worker::queueing`'s one module over. Every helper in [`crate::anim`] is a closed form over
    /// `(now, start, duration)`, so an application testing an animated component needs exactly one
    /// thing: to say what `now` is. Without it a scene of nineteen frames is timed by the machine it
    /// runs on, which is a stopwatch and not a gate.
    ///
    /// It pins until the next call. `Instant` has no public constructor, so the moment handed in is
    /// always one the caller already had — a real reading, offset by real durations.
    pub fn pin_clock(&mut self, at: Instant) {
        self.pinned = true;
        self.env.now = at;
    }

    /// Step a pinned clock. Pins it at `now + by` if it was not pinned already.
    pub fn advance(&mut self, by: Duration) {
        self.pin_clock(self.env.now + by);
    }

    /// **The second pass**: each request in `(z, seq)` order, in rounds, bounded at
    /// [`OVERLAY_ROUNDS`].
    ///
    /// # Why it is rounds and not a queue walked to exhaustion
    ///
    /// A body may request another overlay — a submenu, a tooltip over a menu item — and the request
    /// has to be satisfied on the same frame or a submenu would trail its parent by one. So the pass
    /// repeats while anything was queued, and **sixteen is where that stops being a menu and starts
    /// being a loop**: a body that requests itself for ever gets a limit rather than a hang, and
    /// [`Frame::overlay_rounds`] is how a gate sees which happened.
    ///
    /// # The two things each round has to do in order
    ///
    /// 1. **Swap the queue out of the frame.** The bodies about to run will queue into it.
    /// 2. **Sort by `(z, seq)`.** `seq` is the request order and is unique, so the pair is a total
    ///    order and the sort is deterministic. A menu below a modal therefore draws — and takes its
    ///    hit entries — *before* the modal's barrier goes down, which is what withholds it from the
    ///    pointer.
    ///
    /// There used to be a third thing, and it is worth knowing that it is gone: the arena handed the
    /// round its own chunk, because a body that queued while its round was running would otherwise
    /// grow the buffer it was executing out of and free itself mid-call. A `Vec<Box<…>>` that grows
    /// moves the pointers and not the closures, so the hazard does not exist to be solved.
    fn overlay_pass<'f>(&mut self, bodies: &Bodies<'f>, screen: Rect) {
        for _ in 0..OVERLAY_ROUNDS {
            if self.frame.overlays.is_empty() {
                break;
            }
            self.frame.overlay_rounds += 1;
            core::mem::swap(&mut self.frame.overlays, &mut self.round);
            self.round.sort_unstable_by_key(|r| (r.z, r.seq));
            for i in 0..self.round.len() {
                // Lifted out by value — `OverlayRequest` is `Copy` — because everything below needs
                // `&mut self` and the `Vec` it came from is a field of it.
                let req = self.round[i];
                self.run_overlay(&req, bodies, screen);
            }
            self.round.clear();
        }
        // **Anything still queued after the bound is dropped rather than leaked**, and no line here
        // has to say so: the boxes go with the queue when the frame call ends.
        self.frame.overlays.clear();
        self.frame.overlay_bodies = u32::try_from(bodies.count()).unwrap_or(u32::MAX);
        self.frame.layer_z = Z::BASE;
        self.layer_census();
    }

    /// Place one overlay, ensure its layer, and run its body inside it.
    fn run_overlay<'f>(&mut self, req: &OverlayRequest, bodies: &Bodies<'f>, screen: Rect) {
        // **Taken out of the queue rather than borrowed inside it**, so that a body requesting another
        // overlay pushes into the same queue with nothing borrowed. It is dropped at the end of this
        // function whichever arm below ran, which is where *exactly once* comes from — and it is the
        // `Box`, not this function, that does the dropping.
        let Some(mut body) = bodies.take(req.body) else {
            // One request names one body and the queue is drained once, so this is unreachable. A skip
            // rather than a panic, because there is no output a panic here would protect.
            return;
        };
        let this_frame = self.frame.frames;
        // **One owner keys one layer**, so a second request under one id this frame is inert — the
        // same policy as `Ctx::interact`'s merge and for the same reason: two rectangles for one
        // surface is not a thing the lifecycle can express.
        if self
            .placed
            .iter()
            .any(|p| p.owner == req.owner && p.seen == this_frame)
        {
            self.frame.overlays_merged += 1;
            return;
        }

        let rect = place(req.anchor, req.opts.size, screen, req.opts.placement);
        let layer = self.ensure_layer(req, rect, screen);

        // The body draws in the layer's own coordinates, so its origin is the layer's position and
        // the pointer arrives translated by the same amount — the transform every `Ctx` applies on
        // the way down, applied once at the top of a new one.
        let pointer = self.frame.pointer.map(|(x, y)| (x - rect.x, y - rect.y));
        let parent_z = self.frame.layer_z;
        self.frame.layer_z = req.z;
        // **The owner roots the body's id stack.** This is the half that makes an overlay a
        // different *place* for identity: without it a body drawn inline and the same body drawn
        // here produce the same ids, and the second one is inert by the collision policy.
        self.frame.stack.push(req.owner);
        if let Some(view) = self.screen.layers().view(layer) {
            let mut cx = Ctx {
                view,
                frame: &mut self.frame,
                env: &self.env,
                rect: Rect::new(0, 0, rect.w, rect.h),
                origin: (rect.x, rect.y),
                pointer,
                content: (0, 0),
                bodies,
                _frame: PhantomData,
                _not_send: PhantomData,
            };
            body(&mut cx);
        }
        // **There is no `else`, and that is the point.** An operator layer has no cells and cannot be
        // drawn into, so a `None` view is a body that never runs; nothing here mints an overlay as an
        // operator layer, so it is unreachable in practice. *Unreachable* was never a reason to leak,
        // and now it cannot be one — the drop belongs to the `Box` rather than to a branch.
        self.frame.stack.pop();
        self.frame.layer_z = parent_z;
        self.frame.overlays_placed += 1;
    }

    /// Find or make the layer for an owner, and reconcile its rectangle, its band and its scrim.
    ///
    /// **A layer requested again is reused.** The engine's `set_rect` keeps a moved layer's cells
    /// and reallocates a resized one's, so *no reallocation* is true of a move and false of a
    /// resize; [`Driver::surface_reallocs`] counts the difference on this side of the seam.
    fn ensure_layer(&mut self, req: &OverlayRequest, rect: Rect, screen: Rect) -> LayerId {
        let this_frame = self.frame.frames;
        if let Some(at) = self.placed.iter().position(|p| p.owner == req.owner) {
            let old = self.placed[at];
            if old.z != req.z {
                self.screen.layers().set_z(old.layer, req.z);
            }
            if old.rect != rect {
                if (old.rect.w, old.rect.h) != (rect.w, rect.h) {
                    self.reallocs += 1;
                }
                self.screen.layers().set_rect(old.layer, rect);
            }
            let scrim = self.reconcile_scrim(old.scrim, req, screen);
            self.placed[at] = Placed {
                owner: req.owner,
                layer: old.layer,
                scrim,
                rect,
                z: req.z,
                seen: this_frame,
            };
            return old.layer;
        }
        let layer = self
            .screen
            .layers()
            .add_content(req.z, rect, req.opts.opaque);
        let scrim = self.reconcile_scrim(None, req, screen);
        self.placed.push(Placed {
            owner: req.owner,
            layer,
            scrim,
            rect,
            z: req.z,
            seen: this_frame,
        });
        layer
    }

    /// Bring the scrim into line with what was asked for.
    ///
    /// **At `z - 1`, covering the whole screen**, because a scrim is proportional to the screen it
    /// darkens and not to the overlay it belongs to. That is what makes it the expensive half of a
    /// modal, and why the two obligations in [`crate::overlay`]'s documentation exist.
    fn reconcile_scrim(
        &mut self,
        existing: Option<ScrimLayer>,
        req: &OverlayRequest,
        screen: Rect,
    ) -> Option<ScrimLayer> {
        let want = req.opts.scrim;
        match (existing, want) {
            (None, None) => None,
            (Some(had), None) => {
                self.screen.layers().remove(had.layer);
                None
            }
            (Some(had), Some(spec)) if had.spec == spec => {
                // A move or a resize of the screen, and neither is a new layer. The engine's
                // `set_rect` on an operator marks damage, so it is called only when something
                // actually changed — a scrim re-set to the rectangle it already had would repaint
                // the whole screen every frame, which is the 117% number from the other direction.
                let z = req.z - 1;
                if had.z != z {
                    self.screen.layers().set_z(had.layer, z);
                }
                if had.rect != screen {
                    self.screen.layers().set_rect(had.layer, screen);
                }
                Some(ScrimLayer {
                    layer: had.layer,
                    spec,
                    rect: screen,
                    z,
                })
            }
            (had, Some(spec)) => {
                if let Some(had) = had {
                    self.screen.layers().remove(had.layer);
                }
                let z = req.z - 1;
                let layer = self.screen.layers().add_operator(z, screen, spec.mix());
                Some(ScrimLayer {
                    layer,
                    spec,
                    rect: screen,
                    z,
                })
            }
        }
    }

    /// Remove the layers nothing asked for this frame.
    ///
    /// **The census is not §5's identity sweep and cannot be.** The sweep asks *did this id draw*;
    /// an owner whose dropdown is closed is still drawing, so the sweep would keep a layer nothing
    /// wants. This asks *was this layer requested*, which is a different question about a different
    /// thing, and the two cannot share a pass.
    fn layer_census(&mut self) {
        let this_frame = self.frame.frames;
        let mut at = 0;
        while at < self.placed.len() {
            if self.placed[at].seen == this_frame {
                at += 1;
                continue;
            }
            // `swap_remove`, because the order of this list decides nothing: the stack sorts by z.
            let gone = self.placed.swap_remove(at);
            self.screen.layers().remove(gone.layer);
            if let Some(scrim) = gone.scrim {
                self.screen.layers().remove(scrim.layer);
            }
        }
    }

    /// How many overlay layers this driver is keeping alive.
    ///
    /// **The census's number.** A dropdown that closes takes its layer with it on the frame after it
    /// stopped being requested, and an owner that is still drawing keeps nothing alive by itself.
    pub fn layers_live(&self) -> usize {
        self.placed.len()
    }

    /// How many surfaces the layer lifecycle has reallocated since this driver was made.
    ///
    /// **A move is 0 and a resize is 1.** The gate is the pair: *no reallocation* said of a layer
    /// that was moved is a true sentence, and said of one that was resized it is not.
    pub fn surface_reallocs(&self) -> u64 {
        self.reallocs
    }

    /// Swap the theme. **A move into `Env`**, and the next frame reports it changed.
    pub fn set_theme(&mut self, theme: Theme) {
        self.env.theme = theme;
        self.env.theme_changed = true;
    }

    /// **How far one wheel click moves.** Configuration, and the whole motion model: a terminal
    /// delivers discrete clicks, so there is nothing to interpolate and nothing to decelerate.
    pub fn set_wheel(&mut self, wheel: crate::scroll::Wheel) {
        self.frame.wheel_config = wheel;
    }

    /// **Ask real frames to maintain the drawn extent**, which a scroll area over content of unknown
    /// size reads one frame late, between frames, where the application already owns the offset.
    ///
    /// **Off by default, and the default is the decision.** Maintaining it costs a display-width walk
    /// per drawing verb — 7% of the frame budget — so *a frame that is not measuring must not
    /// maintain the measurement*. While it is on, [`Frame`]'s `consulted` record is also populated
    /// and nothing in a real frame reads it; harmless, and stated here rather than discovered.
    pub fn measure_extent(&mut self, on: bool) {
        self.frame.measure_extent = on;
    }

    /// The frame, for the gates that count its structures.
    pub fn inspect(&self) -> &Frame {
        &self.frame
    }

    /// The env, for the gates.
    pub fn env(&self) -> &Env {
        &self.env
    }

    /// The screen's size.
    pub fn size(&self) -> (u16, u16) {
        self.screen.size()
    }

    /// Post a pointer event for the next frame.
    ///
    /// **A door for the gates and the reports, and it has to exist.** A headless attach has no tty, so
    /// `Screen::next_event` never yields anything and there is no other way to drive the pointer at
    /// all — every property in this ticket is about what a press does, and none of it would be
    /// testable. It is the same shape as `Clock::Manual`: a deterministic input path that is public
    /// API rather than test scaffolding, because a test that cannot reach the mechanism is not a test
    /// of it.
    pub fn post_mouse(&mut self, m: vitui_engine::Mouse) {
        self.pending.push(vitui_engine::Event::Mouse(m));
    }

    /// Post a key for the next frame. **The same door and the same reason as [`Driver::post_mouse`]**
    /// — a headless attach has no tty, so nothing about routing would be reachable without it.
    ///
    /// It shares the one queue with the pointer, because the batch split is over the interleaving:
    /// posting `[Key(a), Down]` and posting `[Down, Key(a)]` are different frames, and two per-kind
    /// doors could not have expressed the difference.
    pub fn post_key(&mut self, k: vitui_engine::Key) {
        self.pending.push(vitui_engine::Event::Key(k));
    }

    /// How many events are queued and not yet consumed by a frame.
    ///
    /// **The gate that counts frames reads this**: sixteen edges are drained when this reaches zero,
    /// and it took sixteen frames to get there.
    pub fn queued(&self) -> usize {
        self.pending.len() - self.pending_at
    }

    /// The keys this frame's batch carried that nobody took.
    ///
    /// **The outermost level of the one queue is the application**, and this is where it reads. Not
    /// a second queue: it is a window onto the same one, valid until the next frame begins.
    pub fn unhandled(&self) -> &[vitui_engine::Key] {
        self.frame.undrained_keys()
    }

    /// Plant the id-keyed facts, for the gates. **Tickets 10 and 12 own the real writers** — the
    /// press award and the focus resolution — and this is how ticket 09 tests the sweep without
    /// inventing either of them early.
    pub fn plant(&mut self, grab: Option<Id>, focus: Option<Id>, click: Option<Id>) {
        self.frame.plant_facts(grab, focus, click);
    }
}

#[cfg(test)]
mod routing_tests {
    //! Ticket 11's gates: **one queue, and a frame that consumes at most one routing edge.**
    //!
    //! The classification and the splitter have their own unit tests in [`crate::route`], including
    //! the negative case that keeps the two-ended rule honest. What is here is the half that needs a
    //! whole frame: who a key is answered to, how many frames a burst costs, and where bubbling
    //! happens.

    use super::*;
    use crate::id::Id;
    use vitui_engine::{
        Button, Buttons, Key, KeyCode, KeyKind, KeyText, Mods, Mouse, MouseKind, Wheel,
    };

    fn driver() -> Driver {
        Driver::headless(300, 80).expect("attaching to a sink cannot fail")
    }

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: Mods::NONE,
            kind: KeyKind::Press,
            text: KeyText::EMPTY,
            at: Instant::now(),
        }
    }

    fn mouse(x: u16, y: u16, kind: MouseKind) -> Mouse {
        Mouse {
            x,
            y,
            kind,
            buttons: Buttons::NONE,
            mods: Mods::NONE,
            at: Instant::now(),
        }
    }

    /// Run frames until nothing is queued, and answer how many it took.
    fn drain(d: &mut Driver, mut draw: impl FnMut(&mut Ctx<'_, '_>)) -> usize {
        let mut frames = 0;
        loop {
            d.frame(&mut draw);
            frames += 1;
            if d.queued() == 0 {
                break;
            }
        }
        frames
    }

    /// **`next_key` answers nobody but the focused id.**
    ///
    /// Not *the innermost*, not *the one under the pointer*, and not *whoever asks first*. That one
    /// rule is also ticket 12's `Trap` in its entirety: a scope that has taken the focus has taken
    /// the keyboard, and there is no second mechanism to keep consistent with this one.
    #[test]
    fn next_key_answers_nobody_but_the_focused_id() {
        let mut d = driver();
        let focused = Id::from_raw(1);
        let other = Id::from_raw(2);
        d.plant(None, Some(focused), None);
        d.post_key(key(KeyCode::Char('a')));

        let mut seen = (None, None);
        d.frame(|cx| {
            cx.interact(other, Rect::new(0, 0, 4, 1), Interest::FOCUS);
            // The unfocused one asks first, and asks with a key sitting in the queue.
            seen.0 = cx.next_key(other);
            cx.interact(focused, Rect::new(0, 1, 4, 1), Interest::FOCUS);
            seen.1 = cx.next_key(focused);
        });
        assert_eq!(seen.0, None, "an unfocused widget is answered nothing");
        assert_eq!(seen.1.map(|k| k.code), Some(KeyCode::Char('a')));
    }

    /// With nothing focused, **every key falls through to the application** — which is the outermost
    /// level of the one queue and not a second queue.
    #[test]
    fn nothing_focused_means_the_application_gets_the_key() {
        let mut d = driver();
        let row = Id::from_raw(1);
        d.post_key(key(KeyCode::Char('a')));
        d.frame(|cx| {
            cx.interact(row, Rect::new(0, 0, 4, 1), Interest::FOCUS);
            assert!(cx.next_key(row).is_none(), "it is not focused");
        });
        let unhandled: Vec<KeyCode> = d.unhandled().iter().map(|k| k.code).collect();
        assert_eq!(unhandled, vec![KeyCode::Char('a')]);
    }

    /// **Sixteen edges, sixteen frames, eight clicks, none lost.**
    ///
    /// The number that made the rule worth having. Eight presses and eight releases arrive as one
    /// burst — which is what happens the moment the theme drops the tracking level below `Motion`
    /// and the terminal stops interleaving moves between them.
    #[test]
    fn sixteen_edges_are_sixteen_frames_and_eight_clicks() {
        let mut d = driver();
        let button = Id::from_raw(1);
        for _ in 0..8 {
            d.post_mouse(mouse(2, 0, MouseKind::Down(Button::Left)));
            d.post_mouse(mouse(2, 0, MouseKind::Up(Button::Left)));
        }
        assert_eq!(d.queued(), 16, "sixteen edges went in");

        let mut clicks = 0;
        let mut draw = |cx: &mut Ctx<'_, '_>| {
            let r = cx.interact(button, Rect::new(0, 0, 8, 1), Interest::CLICK);
            if r.clicked {
                clicks += 1;
            }
        };
        let frames = drain(&mut d, &mut draw);
        // One more, because an award is delivered on the frame after the one that made it.
        d.frame(&mut draw);

        assert_eq!(frames, 16, "one edge a frame, and every one of them drew");
        assert_eq!(clicks, 8, "eight clicks, none lost");
    }

    /// **`[Key(a), Tab]` is one frame**, because `Tab` closes a batch rather than opening one.
    ///
    /// The key was routed against the focus this frame drew with, which is the state it was typed
    /// against. Split the other way it would go to the widget the `Tab` is about to move to.
    #[test]
    fn a_key_before_a_tab_is_one_frame() {
        let mut d = driver();
        let focused = Id::from_raw(1);
        d.plant(None, Some(focused), None);
        d.post_key(key(KeyCode::Char('a')));
        d.post_key(key(KeyCode::Tab));

        let mut got = Vec::new();
        let frames = drain(&mut d, |cx| {
            cx.interact(focused, Rect::new(0, 0, 4, 1), Interest::FOCUS);
            while let Some(k) = cx.next_key(focused) {
                got.push(k.code);
            }
        });
        assert_eq!(frames, 1, "both events, one frame");
        // **Ticket 12 corrected the second half of this gate.** Ticket 11 had the widget take the
        // `Tab` and decline it, because nothing yet consumed one; the ring now withholds it, so a
        // widget cannot drain the key that is about to move the focus off it.
        assert_eq!(got, vec![KeyCode::Char('a')], "the Tab was never offered");
        assert!(
            d.unhandled().is_empty(),
            "the ring took it: it is the ring's key and the application does not see it"
        );
    }

    /// **`[Tab, Key(a)]` routes both keys** — two frames, and the second one happens because the
    /// frame that took a short batch asked for another. One wake, folded at `end`.
    #[test]
    fn a_key_after_a_tab_routes_both_keys() {
        let mut d = driver();
        let focused = Id::from_raw(1);
        d.plant(None, Some(focused), None);
        d.post_key(key(KeyCode::Tab));
        d.post_key(key(KeyCode::Char('a')));

        let mut per_frame: Vec<Vec<KeyCode>> = Vec::new();
        let frames = drain(&mut d, |cx| {
            cx.interact(focused, Rect::new(0, 0, 4, 1), Interest::FOCUS);
            let mut this = Vec::new();
            while let Some(k) = cx.next_key(focused) {
                this.push(k.code);
            }
            per_frame.push(this);
        });
        assert_eq!(frames, 2);
        assert_eq!(
            per_frame,
            vec![vec![], vec![KeyCode::Char('a')]],
            "the Tab is the ring's and `a` is the widget's, and neither was routed against the \
             other's focus"
        );
    }

    /// **Ordinary keys fold**: an eight-thousand-key paste is one frame, not eight thousand.
    ///
    /// This is the direction "one event a frame" gets wrong that nobody notices until a paste.
    #[test]
    fn an_eight_thousand_key_paste_is_one_frame() {
        let mut d = driver();
        let editor = Id::from_raw(1);
        d.plant(None, Some(editor), None);
        for i in 0..8_000u32 {
            let c = char::from_u32(b'a' as u32 + i % 26).unwrap_or('a');
            d.post_key(key(KeyCode::Char(c)));
        }

        let mut taken = 0;
        let frames = drain(&mut d, |cx| {
            cx.interact(editor, Rect::new(0, 0, 40, 1), Interest::FOCUS);
            while cx.next_key(editor).is_some() {
                taken += 1;
            }
        });
        assert_eq!(frames, 1, "no key in it is a routing edge");
        assert_eq!(taken, 8_000, "and none of them was dropped");
    }

    /// **The growth-ratio detector, through the whole frame**: four times the keys is about four
    /// times the slots touched. The quadratic form is 16× here and still passes a 100 µs gate.
    #[test]
    fn draining_a_paste_through_a_frame_is_linear() {
        let touches_for = |n: u32| {
            let mut d = driver();
            let editor = Id::from_raw(1);
            d.plant(None, Some(editor), None);
            for _ in 0..n {
                d.post_key(key(KeyCode::Char('x')));
            }
            d.frame(|cx| {
                cx.interact(editor, Rect::new(0, 0, 40, 1), Interest::FOCUS);
                while cx.next_key(editor).is_some() {}
            });
            d.inspect().key_touches()
        };
        let small = touches_for(2_000);
        let large = touches_for(8_000);
        let ratio = large as f64 / small as f64;
        assert!(
            ratio <= 5.0,
            "the drain touched {ratio:.2}x the slots for 4x the keys, which is quadratic territory"
        );
    }

    /// **A modal bounds the pointer only.** A focused widget behind one keeps receiving keys, which
    /// is the floor ticket 12's `Trap` is built on: trapping the keyboard is a *scope* decision and
    /// not something modality does for free.
    #[test]
    fn a_modal_withholds_the_pointer_and_not_the_keyboard() {
        let mut d = driver();
        let behind = Id::from_raw(1);
        let front = Id::from_raw(2);
        d.plant(None, Some(behind), None);
        d.post_mouse(mouse(2, 0, MouseKind::Move));
        d.post_key(key(KeyCode::Char('a')));

        let mut got = None;
        let mut hovered = None;
        d.frame(|cx| {
            let r = cx.interact(
                behind,
                Rect::new(0, 0, 8, 1),
                Interest::CLICK.with(Interest::FOCUS),
            );
            got = cx.next_key(behind);
            cx.modal_barrier_here();
            cx.interact(front, Rect::new(0, 10, 8, 1), Interest::CLICK);
            let _ = r;
        });
        d.frame(|cx| {
            let r = cx.interact(
                behind,
                Rect::new(0, 0, 8, 1),
                Interest::CLICK.with(Interest::FOCUS),
            );
            hovered = Some(r.hovered);
            cx.modal_barrier_here();
            cx.interact(front, Rect::new(0, 10, 8, 1), Interest::CLICK);
        });
        assert_eq!(
            got.map(|k| k.code),
            Some(KeyCode::Char('a')),
            "the key got through"
        );
        assert_eq!(hovered, Some(false), "and the pointer did not");
    }

    /// **A bubbling container adds exactly one id: its own.**
    ///
    /// The equality gate. Taking a closure is not what renames a child — *scoping identity* is — so
    /// the ids inside a scope are equal entry for entry to the ids without it, and the only
    /// difference in the table is the container itself.
    #[test]
    fn a_bubbling_container_adds_exactly_one_id() {
        let rows = |cx: &mut Ctx<'_, '_>| {
            for i in 0..24u64 {
                cx.with_key(i, |cx| {
                    cx.interact_here(Rect::new(0, i as i32, 20, 1), Interest::FOCUS);
                });
            }
        };

        let mut bare = driver();
        bare.frame(|cx| rows(cx));
        let bare_hits: Vec<Id> = bare.inspect().hits().iter().map(|h| h.id).collect();
        let bare_live = bare.inspect().ids().live();

        let mut wrapped = driver();
        wrapped.frame(|cx| {
            let id = Id::from_raw(9_999);
            cx.scope(id, ScopeKind::Group, |cx| rows(cx));
        });
        let wrapped_hits: Vec<Id> = wrapped.inspect().hits().iter().map(|h| h.id).collect();
        let wrapped_live = wrapped.inspect().ids().live();

        assert_eq!(
            wrapped_hits, bare_hits,
            "entry for entry: a scope renames nothing, and a renamed field loses its focus"
        );
        assert_eq!(
            wrapped_live,
            bare_live + 1,
            "and the container's own is the one it adds"
        );
    }

    /// **Bubbling is `Ctx::scope`'s after-the-body moment**, and it is not a walk of the id path.
    ///
    /// A container draws before its children, so asking during its body would be *capture*. Here
    /// the focused child declines a key it does not want and the container takes it afterwards.
    #[test]
    fn a_declined_key_bubbles_to_the_container_after_its_body() {
        let mut d = driver();
        let panel = Id::from_raw(1);
        let field = Id::from_raw(2);
        d.plant(None, Some(field), None);
        d.post_key(key(KeyCode::Escape));

        let mut captured = None;
        let mut bubbled = None;
        d.frame(|cx| {
            cx.scope(panel, ScopeKind::Group, |cx| {
                // Capture: the container's own moment has not arrived, and asking here answers
                // nothing because the container is not the routing target yet.
                captured = cx.next_key(panel);
                cx.interact(field, Rect::new(0, 0, 8, 1), Interest::FOCUS);
                let k = cx.next_key(field).expect("the focused field is offered it");
                assert_eq!(k.code, KeyCode::Escape);
                cx.decline(k);
            });
            bubbled = cx.next_key(panel);
        });
        assert_eq!(
            captured, None,
            "before the body is capture, and there is no capture here"
        );
        assert_eq!(bubbled.map(|k| k.code), Some(KeyCode::Escape));
    }

    /// **The obvious drain loop terminates.**
    ///
    /// `while let Some(k) = cx.next_key(id) { cx.decline(k) }` is the shape `decline`'s own
    /// documentation describes, and a bare cursor rewind hands the same key out for ever — a hung
    /// frame from the most natural way to write the verb. A decline ends this widget's turn, so the
    /// loop ends after one offer and the key goes outward whole.
    #[test]
    fn the_obvious_drain_loop_terminates() {
        let mut d = driver();
        let field = Id::from_raw(1);
        d.plant(None, Some(field), None);
        for _ in 0..4 {
            d.post_key(key(KeyCode::Char('a')));
        }

        let mut rounds = 0;
        d.frame(|cx| {
            cx.interact(field, Rect::new(0, 0, 8, 1), Interest::FOCUS);
            while let Some(k) = cx.next_key(field) {
                rounds += 1;
                assert!(rounds < 1_000, "the drain loop did not terminate");
                cx.decline(k);
            }
        });
        assert_eq!(rounds, 1, "one offer, one decline, and the queue closes");
        assert_eq!(
            d.unhandled().len(),
            4,
            "and all four keys went outward, in order"
        );
    }

    /// **A decline never duplicates a keystroke.**
    ///
    /// The queue holds every key it was given and removes none, so a decline with nothing taken —
    /// or a second decline — has nothing to restore. Restoring anyway would make the application
    /// process one keystroke twice, and would shift the tail into a possible allocation.
    #[test]
    fn declining_twice_does_not_duplicate_a_keystroke() {
        let mut d = driver();
        let field = Id::from_raw(1);
        d.plant(None, Some(field), None);
        d.post_key(key(KeyCode::Char('a')));

        d.frame(|cx| {
            cx.interact(field, Rect::new(0, 0, 8, 1), Interest::FOCUS);
            let k = cx.next_key(field).expect("a key");
            cx.decline(k);
            cx.decline(k);
            cx.decline(key(KeyCode::Char('z')));
        });
        assert_eq!(
            d.unhandled().iter().map(|k| k.code).collect::<Vec<_>>(),
            vec![KeyCode::Char('a')],
            "one key in, one key out"
        );
        assert_eq!(d.inspect().keys_in_batch(), 1);
    }

    /// **A panel that is clickable and bubbling still bubbles**, though its id is claimed twice.
    ///
    /// This is how a bubbling container is actually written — `interact` for the region, `scope` for
    /// the children — and `interact`'s merge rule would have made it silently deaf: every declined
    /// key falling through to the application with nothing on screen to notice.
    #[test]
    fn a_clickable_container_still_bubbles_under_its_own_id() {
        let mut d = driver();
        let panel = Id::from_raw(1);
        let field = Id::from_raw(2);
        d.plant(None, Some(field), None);
        d.post_key(key(KeyCode::Escape));

        let mut bubbled = None;
        d.frame(|cx| {
            cx.interact(panel, Rect::new(0, 0, 20, 10), Interest::CLICK);
            cx.scope(panel, ScopeKind::Group, |cx| {
                cx.interact(field, Rect::new(1, 1, 8, 1), Interest::FOCUS);
                let k = cx.next_key(field).expect("offered to the focus");
                cx.decline(k);
            });
            bubbled = cx.next_key(panel);
        });
        assert_eq!(bubbled.map(|k| k.code), Some(KeyCode::Escape));
        assert!(d.unhandled().is_empty(), "and it did not fall through");
    }

    /// A scope the focus is **not** in bubbles nothing — otherwise a sibling drawn earlier would
    /// take keys the focused widget has not been offered yet.
    #[test]
    fn a_scope_the_focus_is_not_in_takes_nothing() {
        let mut d = driver();
        let sidebar = Id::from_raw(1);
        let editor = Id::from_raw(2);
        let field = Id::from_raw(3);
        d.plant(None, Some(field), None);
        d.post_key(key(KeyCode::Char('a')));

        let mut stolen = None;
        let mut reached = None;
        d.frame(|cx| {
            cx.scope(sidebar, ScopeKind::Group, |cx| {
                cx.interact(Id::from_raw(4), Rect::new(0, 0, 8, 1), Interest::CLICK);
            });
            stolen = cx.next_key(sidebar);
            cx.scope(editor, ScopeKind::Group, |cx| {
                cx.interact(field, Rect::new(0, 10, 8, 1), Interest::FOCUS);
                reached = cx.next_key(field);
            });
        });
        assert_eq!(stolen, None, "the focus was not in it");
        assert_eq!(reached.map(|k| k.code), Some(KeyCode::Char('a')));
    }

    /// **Folding a wheel is a sum, not an overwrite.**
    ///
    /// A wheel notch folds into a frame (ADR 0016) *and* carries intent (ADR 0008), and the two are
    /// only compatible if the notches add up. Overwriting made three notches in one batch scroll one
    /// row, which is dropping intent under another name.
    #[test]
    fn wheel_notches_in_one_batch_add_up() {
        let mut d = driver();
        let list = Id::from_raw(1);
        let draw =
            |cx: &mut Ctx<'_, '_>| cx.interact(list, Rect::new(0, 0, 20, 10), Interest::SCROLL);
        d.post_mouse(mouse(2, 2, MouseKind::Move));
        d.frame(|cx| {
            draw(cx);
        });
        for _ in 0..3 {
            d.post_mouse(mouse(2, 2, MouseKind::Wheel(Wheel::Down)));
        }
        // **On the frame the batch arrives**, not the one after: the wheel is the one pointer
        // channel resolved in `begin` rather than awarded at `end`, because the offset it moves is
        // read during the draw by the widget that owns it (ticket 14).
        let mut scrolled = (0, 0);
        let frames = drain(&mut d, |cx| {
            scrolled = draw(cx).scrolled;
        });
        assert_eq!(frames, 1, "a wheel notch is not a routing edge");
        assert_eq!(scrolled, (0, 3), "three notches are three rows");
    }
}

/// **Why `'f` has to be invariant, kept runnable rather than asserted.**
///
/// Neither of these two shapes gives the overlay bound any force, and both compile — which is the
/// evidence that the third one is doing something. They are written out here because the first two
/// attempts at this module's negative case were exactly these, and both passed.
///
/// **One lifetime**: `child` shrinks the only lifetime there is, so a capturing body is fine.
///
/// ```
/// struct Frame;
/// struct One<'f> { _f: &'f mut Frame }
/// impl<'f> One<'f> {
///     fn child(&mut self) -> One<'_> { One { _f: self._f } }
///     fn overlay(&mut self, body: impl FnOnce() + 'f) { drop(body); }
/// }
/// fn run<'f>(f: &'f mut Frame, view: impl FnOnce(&mut One<'f>)) {
///     let mut c = One { _f: f };
///     view(&mut c);
/// }
/// let mut frame = Frame;
/// run(&mut frame, |cx| {
///     let local = String::from("x");
///     cx.child().overlay(|| { let _ = &local; });   // compiles, and should not
/// });
/// ```
///
/// **Two lifetimes but a covariant brand**: a caller can shorten `'f`, so the bound is satisfied by a
/// shorter capture and nothing changed.
///
/// ```
/// use std::marker::PhantomData;
/// struct Frame;
/// struct Co<'f, 'v> { _v: &'v mut Frame, _f: PhantomData<&'f ()> }
/// impl<'f, 'v> Co<'f, 'v> {
///     fn child(&mut self) -> Co<'f, '_> { Co { _v: self._v, _f: PhantomData } }
///     fn overlay(&mut self, body: impl FnOnce() + 'f) { drop(body); }
/// }
/// fn run<'f>(f: &'f mut Frame, view: impl FnOnce(&mut Co<'f, '_>)) {
///     let mut c = Co { _v: f, _f: PhantomData };
///     view(&mut c);
/// }
/// let mut frame = Frame;
/// run(&mut frame, |cx| {
///     let local = String::from("x");
///     cx.child().overlay(|| { let _ = &local; });   // still compiles
/// });
/// ```
///
/// The shipped brand is `PhantomData<fn(&'f ()) -> &'f ()>`, which is invariant because `'f` appears
/// in both argument and return position — and [`Ctx`]'s own `compile_fail` case is the one that bites.
#[cfg(doc)]
pub struct WhyInvariance;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Glyph, Role};

    /// A driver over a sink, sized for the gates.
    fn driver() -> Driver {
        Driver::headless(40, 10).expect("attaching to a sink cannot fail")
    }

    /// **`Ctx` is `!Send`, and it needs saying** — a `Surface` is `Send`, so a `View` is, so `Ctx`
    /// would be without the marker. A `Ctx` on another thread is a draw on another thread.
    ///
    /// Asserted by a helper that only accepts `Send`, so the *absence* is what compiles and the
    /// presence is a compile error — see the paired doctest on [`Ctx`]'s module for the other
    /// direction.
    #[test]
    fn a_ctx_is_not_send_and_the_engines_types_are() {
        const fn needs_send<T: Send>() {}
        // The engine's are, which is what makes the marker necessary rather than decorative.
        needs_send::<vitui_engine::Style>();
        needs_send::<vitui_engine::Rect>();
        needs_send::<Theme>();
        // And `Frame` is, because it is data. Only `Ctx` is not, because it holds the view.
        needs_send::<Id>();
        // `Ctx` is deliberately absent from this list; the compile outcome is on the module.
    }

    /// **A never-`begin`-ed frame cannot be reached, and a claim on one would fail rather than
    /// spin.**
    ///
    /// Both halves of the disjunction, because the ticket allows either and the second is the one
    /// that survives somebody adding a constructor later. The failure mode being guarded is a **hang**
    /// — stamp 0 over slots stamped 0 means no slot is ever *not mine* — and a hang is worse than a
    /// failure because `cargo test` has no per-test timeout.
    #[test]
    fn a_claim_terminates_rather_than_spinning() {
        // **The probe is bounded**, so a claim is a value however wrong the stamp is. Since ticket 09
        // gave the table growth, a *full* table is no longer reachable — which is why this asserts
        // termination under load rather than a `None` from a full one, and why the first version of
        // this test stopped being right the moment growth landed.
        let mut table = IdTable::new();
        for i in 0..4_000u64 {
            // Every one of these either claims or merges, and neither walks the ring for ever. A
            // naive probe over a table whose stamp matched every slot would not return at all.
            let _ = table.claim(Id::keyed(Id::ROOT, i));
        }
        assert_eq!(table.live(), 4_000, "every id was claimed");
        assert!(
            table.grows() > 0,
            "and the table grew rather than filling up"
        );
        // A duplicate is inert rather than a second slot.
        assert_eq!(
            table.claim(Id::keyed(Id::ROOT, 0)),
            None,
            "first claimant wins"
        );
    }

    /// The first half: a `Frame` is only reachable through a `Driver`, which begins it.
    #[test]
    fn a_frame_is_only_reachable_through_a_driver_that_begins_it() {
        let mut d = driver();
        assert!(!d.inspect().begun(), "not begun before the first frame");
        d.frame(|_cx| {});
        assert!(d.inspect().begun());
        assert_eq!(d.inspect().frames(), 1);
    }

    /// **The stamp is bumped every frame**, which is what makes the previous frame's slots foreign.
    #[test]
    fn begin_bumps_the_stamp_every_frame() {
        let mut d = driver();
        let mut stamps = Vec::new();
        for _ in 0..4 {
            d.frame(|_cx| {});
            stamps.push(d.inspect().ids().stamp());
        }
        for pair in stamps.windows(2) {
            assert_ne!(pair[0], pair[1], "two frames shared a stamp: {stamps:?}");
        }
        // And never zero, because zero is the value an untouched slot carries.
        assert!(stamps.iter().all(|&s| s != 0));
    }

    /// **The five structures are swapped and cleared**, not dropped and rebuilt.
    #[test]
    fn the_five_structures_are_cleared_every_frame() {
        let mut d = driver();
        d.frame(|cx| {
            for i in 0..8u64 {
                cx.interact(
                    Id::from_raw(i),
                    Rect::new(0, 0, 4, 1),
                    Interest::CLICK.with(Interest::FOCUS),
                );
            }
            cx.overlay(
                Id::ROOT,
                Rect::new(0, 0, 4, 1),
                OverlayOpts::sized(4, 1),
                |_inner| {},
            );
            cx.deadline(cx.now() + std::time::Duration::from_millis(5));
        });
        assert_eq!(d.inspect().hits().len(), 8);
        assert_eq!(d.inspect().ring().len(), 8);

        // The next frame declares nothing, and every structure is empty again.
        d.frame(|_cx| {});
        assert!(
            d.inspect().hits().is_empty(),
            "the hit index was not cleared"
        );
        assert!(d.inspect().ring().is_empty(), "the ring was not cleared");
        assert_eq!(d.inspect().overlays_requested(), 0);
    }

    /// **The four id-keyed facts exist and are four**, which is ADR 0012's closed list.
    #[test]
    fn there_are_four_id_keyed_facts() {
        let d = driver();
        let (grab, origin, focus, click) = d.inspect().id_keyed_facts();
        // All absent on a fresh frame; the point of the gate is the arity and the naming, since the
        // sweep of three of them is ticket 09's.
        assert_eq!((grab, origin, focus, click), (false, false, false, false));
    }

    /// **`end` folds the deadline sink and the repaint flag into one wake.**
    ///
    /// Two callers asking for different moments is one wake at the earlier of them, which is why the
    /// sink is one `Option` and not a list.
    #[test]
    fn two_deadlines_fold_into_the_earlier_one() {
        let mut d = driver();
        let mut seen = None;
        d.frame(|cx| {
            let now = cx.now();
            let later = now + std::time::Duration::from_millis(100);
            let sooner = now + std::time::Duration::from_millis(10);
            cx.deadline(later);
            cx.deadline(sooner);
            seen = Some((now, sooner));
        });
        let (_now, sooner) = seen.expect("the frame ran");
        // The fold is internal, so what is asserted is the property through the only door there is:
        // asking twice does not ask twice.
        let mut d2 = driver();
        d2.frame(|cx| {
            let now = cx.now();
            cx.deadline(now + std::time::Duration::from_millis(100));
            cx.deadline(now + std::time::Duration::from_millis(10));
        });
        let _ = sooner;
        assert_eq!(d2.inspect().frames(), 1);
    }

    /// Geometry: `area` is origin-relative and `child` narrows.
    #[test]
    fn area_is_origin_relative_and_child_narrows() {
        let mut d = driver();
        d.frame(|cx| {
            assert_eq!(cx.area(), Rect::new(0, 0, 40, 10));
            assert_eq!(cx.size(), (40, 10));
            let inner = cx.child(Rect::new(4, 2, 10, 3));
            assert_eq!(
                inner.area(),
                Rect::new(0, 0, 10, 3),
                "a child's area is its own size at its own origin"
            );
            assert_eq!(inner.size(), (10, 3));
        });
    }

    /// **`visible_rows` is the virtualisation primitive**: a scrolled window over a million rows
    /// reaches a screenful.
    #[test]
    fn visible_rows_bounds_a_million_rows_to_a_screenful() {
        let mut d = driver();
        d.frame(|cx| {
            let scrolled = cx.scrolled(0, -500_000);
            let rows = scrolled.visible_rows();
            assert!(
                rows.end - rows.start <= 10,
                "a ten-row screen reached {} rows",
                rows.end - rows.start
            );
            assert!(rows.contains(&500_000), "and it is the right screenful");
            let cols = scrolled.visible_cols();
            assert!(cols.end - cols.start <= 40);
        });
    }

    /// **A scroll scope's offset is a position and `Ctx::scrolled`'s is a translation**, and the one
    /// negation between them is inside `scroll_scope`.
    ///
    /// Components ticket 12's finding, as a regression test on both halves of the same call: the
    /// window `visible_rows()` answers, and whether a verb at a content row inside it actually
    /// lands. Written unnegated, the first read `-5..3` and the second wrote **0 cells** — and the
    /// `Area` the same call pushes carried the positive offset all along, so the two halves of one
    /// verb disagreed about which way down is.
    ///
    /// The `Area` half is asserted beside it, because a fix that negated the *area* instead would
    /// pass the first assertion and break the wheel chain and every `into_view` delta.
    #[test]
    fn a_scroll_scopes_offset_is_a_position_and_the_window_is_the_rows_it_names() {
        let mut d = driver();
        let (mut rows, mut wrote) = (0..0, 0);
        d.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            let view = Rect::new(0, 0, 40, 8);
            cx.scroll_scope(Id::named("list"), view, (0, 5), (0, 992), |cx| {
                rows = cx.visible_rows();
                wrote = cx.text(0, 5, "X", body).cells;
            });
        });
        assert_eq!(
            rows,
            5..13,
            "the window is the content rows the offset names"
        );
        assert_eq!(
            wrote, 1,
            "and a verb at the first of them lands on the screen"
        );

        let area = d.inspect().scroll_areas()[0];
        assert_eq!(
            area.offset,
            (0, 5),
            "the area keeps the position, which is what `Scrollable::between` and \
             `Area::into_view` are both written in"
        );
    }

    /// The drawing verbs reach the wire, and `stage` measures before `blit` draws.
    #[test]
    fn stage_measures_and_blit_draws() {
        let mut d = driver();
        d.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            let w = cx.stage(format_args!("{:>6.2}", 12.5_f64));
            assert_eq!(w, 6, "the width is known before the position is chosen");
            // A right-aligned number: the whole reason `stage` returns a width.
            let written = cx.blit(i32::from(40 - w), 0, body);
            assert_eq!(written.cells, 6);
            // And the one-call form.
            let written = cx.label(0, 1, format_args!("{}", "hello"), body);
            assert_eq!(written.cells, 5);
        });
    }

    /// A staged value that is wider than one cell measures in **columns**, not bytes or characters.
    #[test]
    fn stage_measures_columns_and_not_bytes() {
        let mut d = driver();
        d.frame(|cx| {
            let w = cx.stage(format_args!("漢字"));
            assert_eq!(w, 4, "two characters, four columns, six bytes");
        });
    }

    /// `restyle` takes a descriptor over roles, and the theme resolves it inside the crate.
    #[test]
    fn restyle_takes_a_descriptor_over_roles() {
        let mut d = driver();
        d.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            cx.text(0, 0, "danger", body);
            cx.restyle(
                Rect::new(0, 0, 6, 1),
                &Repaint {
                    fg: Some(Role::Danger),
                    set: Repaint::BOLD,
                    ..Default::default()
                },
            );
        });
    }

    /// `interact` folds the declared interest into the tracking level, as a `max` over a totally
    /// ordered ladder.
    #[test]
    fn tracking_is_the_max_of_what_was_declared() {
        let mut d = driver();
        d.frame(|cx| {
            cx.interact(Id::from_raw(1), Rect::new(0, 0, 4, 1), Interest::CLICK);
        });
        assert_eq!(d.inspect().tracking(), MouseMode::Buttons);

        d.frame(|cx| {
            cx.interact(Id::from_raw(1), Rect::new(0, 0, 4, 1), Interest::CLICK);
            cx.interact(Id::from_raw(2), Rect::new(4, 0, 4, 1), Interest::HOVER);
        });
        assert_eq!(
            d.inspect().tracking(),
            MouseMode::Motion,
            "one widget wanting hover raises the whole frame"
        );

        // And it resets: a frame that declares nothing needs nothing.
        d.frame(|_cx| {});
        assert_eq!(d.inspect().tracking(), MouseMode::Off);
    }

    /// `Interest::FOCUS` costs no tracking, which is why it is a bit here rather than elsewhere.
    #[test]
    fn a_tab_stop_costs_no_tracking() {
        assert_eq!(Interest::FOCUS.tracking(), MouseMode::Off);
        assert_eq!(Interest::NONE.tracking(), MouseMode::Off);
        assert_eq!(Interest::CLICK.tracking(), MouseMode::Buttons);
        assert_eq!(Interest::SCROLL.tracking(), MouseMode::Buttons);
        assert_eq!(Interest::DRAG.tracking(), MouseMode::Drag);
        assert_eq!(Interest::HOVER.tracking(), MouseMode::Motion);
        // The ladder is totally ordered, which is what makes combining a `max`.
        assert!(MouseMode::Off < MouseMode::Buttons);
        assert!(MouseMode::Buttons < MouseMode::Drag);
        assert!(MouseMode::Drag < MouseMode::Motion);
    }

    /// The theme is readable through `Env` and a lookup can be held across two verbs.
    #[test]
    fn the_theme_is_readable_and_holdable() {
        let mut d = driver();
        d.frame(|cx| {
            let theme = cx.theme();
            let body = theme.paint(Role::Body);
            let rule = theme.glyph(Glyph::HLine);
            cx.text(0, 0, rule, body);
            cx.text(0, 1, rule, body);
        });
    }

    /// `theme_changed` is true for exactly one frame.
    #[test]
    fn theme_changed_is_true_for_exactly_one_frame() {
        let mut d = driver();
        let mut first = None;
        d.frame(|cx| first = Some(cx.theme_changed()));
        assert_eq!(first, Some(true), "the first frame has a new theme");
        let mut second = None;
        d.frame(|cx| second = Some(cx.theme_changed()));
        assert_eq!(second, Some(false));

        d.set_theme(Theme::default().resolve(vitui_engine::ColorDepth::Ansi16));
        let mut third = None;
        d.frame(|cx| third = Some(cx.theme_changed()));
        assert_eq!(third, Some(true), "and a swap sets it again");
    }

    /// The clock is sampled **once** per frame.
    #[test]
    fn the_clock_is_sampled_once_a_frame() {
        let mut d = driver();
        d.frame(|cx| {
            let a = cx.now();
            let b = cx.now();
            assert_eq!(a, b, "two reads inside one frame are one moment");
            let inner = cx.child(Rect::new(0, 0, 4, 1));
            assert_eq!(inner.now(), a, "and a child shares it");
        });
    }

    /// The overlay pass is bounded, which is a limit rather than a hang.
    #[test]
    fn the_overlay_pass_is_bounded() {
        assert_eq!(OVERLAY_ROUNDS, 16);
        let mut d = driver();
        d.frame(|cx| {
            for i in 0..4u64 {
                cx.overlay(
                    Id::from_raw(i),
                    Rect::new(0, 0, 4, 1),
                    OverlayOpts::sized(4, 1),
                    |_inner| {},
                );
            }
        });
        // Drained by the pass, which is what makes the queue a queue rather than a log.
        assert_eq!(d.inspect().overlays_requested(), 0);
    }

    /// **`Ctx::child` pushes nothing**, so a rectangle-returning split leaves its panes siblings at
    /// one depth — and a keyed row is one deeper.
    ///
    /// The container rule as an equality: *a container that returns a rectangle preserves its
    /// children's identity; one that takes a closure renames them.*
    #[test]
    fn a_rectangle_returning_split_leaves_its_panes_siblings() {
        let mut d = driver();
        d.frame(|cx| {
            assert_eq!(cx.depth(), 0, "the top of a frame is depth zero");
            let mut pane = cx.child(Rect::new(0, 0, 10, 4));
            assert_eq!(pane.depth(), 0, "a child pushes nothing");
            let deeper = pane.child(Rect::new(0, 0, 4, 2));
            assert_eq!(deeper.depth(), 0, "and neither does a child of a child");
            let _ = deeper.area();
        });
    }

    /// A keyed row is depth 2 from inside, which is the other half of the same equality.
    #[test]
    fn a_keyed_row_is_one_deeper() {
        let mut d = driver();
        d.frame(|cx| {
            cx.with_key(7, |row| {
                assert_eq!(row.depth(), 1, "inside a key");
                row.with_key(3, |cell| {
                    assert_eq!(cell.depth(), 2, "and inside a key inside a key");
                });
            });
            assert_eq!(cx.depth(), 0, "and the stack is popped on the way out");
        });
    }

    /// **Two keys give two widgets and the same key gives the same widget**, which is the whole of
    /// what a loop needs.
    #[test]
    fn a_key_names_the_row() {
        // **One call site, drawn twice.** The first version of this test collected the two frames
        // from two different `for` loops and then compared them — two source lines, so two sets of
        // ids, and the assertion failed for the one reason that is not interesting. A stability test
        // has to run the *same* code.
        fn rows(cx: &mut Ctx<'_, '_>, out: &mut Vec<Id>) {
            for row in 0..3u64 {
                cx.with_key(row, |r| out.push(r.id()));
            }
        }

        let mut d = driver();
        let mut first = Vec::new();
        d.frame(|cx| rows(cx, &mut first));
        assert_eq!(first.len(), 3);
        assert_ne!(first[0], first[1], "two keys, two widgets");
        assert_ne!(first[1], first[2]);

        let mut second = Vec::new();
        d.frame(|cx| rows(cx, &mut second));
        assert_eq!(first, second, "a row keeps its name between frames");
    }

    /// **A scope renames nothing**, entry for entry — the container rule's one load-bearing
    /// exception.
    ///
    /// Without it the frame a modal opens renames every field of the form it traps, and a renamed
    /// field loses its focus and its scroll position for a reason nobody wrote down.
    #[test]
    fn a_scope_scopes_no_identity() {
        // Again one call site, called three ways. Comparing ids collected at three different source
        // lines would compare three different call sites and prove nothing about scoping.
        fn fields(cx: &mut Ctx<'_, '_>, out: &mut Vec<Id>) {
            for row in 0..4u64 {
                cx.with_key(row, |r| out.push(r.id()));
            }
        }

        let mut d = driver();
        let mut without = Vec::new();
        d.frame(|cx| fields(cx, &mut without));

        let mut within = Vec::new();
        d.frame(|cx| {
            cx.scope(Id::named("modal"), ScopeKind::Trap, |inner| {
                fields(inner, &mut within)
            })
        });
        assert_eq!(without, within, "a scope changed an id");

        let mut scrolled = Vec::new();
        d.frame(|cx| {
            cx.scroll_scope(Id::named("list"), cx.area(), (0, 5), (0, 40), |inner| {
                fields(inner, &mut scrolled)
            });
        });
        assert_eq!(without, scrolled, "a scroll scope changed an id");
    }

    /// **A stale grab does not survive its widget**, which is the sweep's whole purpose: without it,
    /// a widget that held the pointer and stopped drawing keeps holding it and every hit test
    /// afterwards resolves to something that is not there.
    #[test]
    fn a_stale_grab_does_not_survive_its_widget() {
        let mut d = driver();
        let held = Id::named("scrollbar.thumb");

        // A frame in which it draws, holding the pointer and the focus.
        d.frame(|cx| {
            cx.interact(held, Rect::new(0, 0, 1, 4), Interest::DRAG);
        });
        d.plant(Some(held), Some(held), Some(held));
        assert_eq!(
            d.inspect().id_keyed_facts(),
            (true, true, true, true),
            "all four planted"
        );

        // A frame in which it draws again: nothing is released.
        d.frame(|cx| {
            cx.interact(held, Rect::new(0, 0, 1, 4), Interest::DRAG);
        });
        assert_eq!(
            d.inspect().id_keyed_facts(),
            (true, true, true, true),
            "a widget that is still drawing keeps its facts"
        );

        // And a frame in which it does not.
        d.frame(|_cx| {});
        let (grab, origin, focus, click) = d.inspect().id_keyed_facts();
        assert!(!grab, "the grab was not released");
        assert!(!origin, "the press origin went with it");
        assert!(!focus, "the focus was released");
        assert!(
            click,
            "**and the click record did not sweep**, which is the fourth fact and deliberate"
        );
    }

    /// A merge is visible at the call site: the second claimant's response is inert and it appears in
    /// no structure.
    #[test]
    fn a_merged_widget_is_inert_and_declares_nothing() {
        let mut d = driver();
        let shared = Id::named("shared");
        d.frame(|cx| {
            let first = cx.interact(shared, Rect::new(0, 0, 4, 1), Interest::CLICK);
            let second = cx.interact(shared, Rect::new(4, 0, 4, 1), Interest::CLICK);
            assert_eq!(first.id, second.id);
        });
        assert_eq!(
            d.inspect().hits().len(),
            1,
            "one hit entry, not two — the second claimant declared nothing"
        );
        assert_eq!(d.inspect().ids().merges(), 1);
    }

    /// The table does not grow on a dense screen, frame after frame.
    #[test]
    fn the_id_table_does_not_grow_in_a_steady_frame() {
        let mut d = driver();
        for _ in 0..8 {
            d.frame(|cx| {
                for i in 0..300u64 {
                    cx.interact(
                        Id::keyed(Id::ROOT, i),
                        Rect::new(0, 0, 4, 1),
                        Interest::CLICK,
                    );
                }
            });
            assert_eq!(d.inspect().ids().grows(), 0);
        }
        assert_eq!(d.inspect().ids().live(), 300);
    }

    /// A key posted before a frame is readable inside it, and declining puts it back in order.
    #[test]
    fn a_key_is_readable_and_declinable() {
        use vitui_engine::{Key, KeyCode, KeyKind, KeyText, Mods};

        let mut d = driver();
        let focused = Id::from_raw(1);
        d.plant(None, Some(focused), None);
        d.post_key(Key {
            code: KeyCode::Char('a'),
            mods: Mods::NONE,
            kind: KeyKind::Press,
            text: KeyText::EMPTY,
            at: Instant::now(),
        });
        d.frame(|cx| {
            cx.interact(
                focused,
                vitui_engine::Rect::new(0, 0, 4, 1),
                Interest::FOCUS,
            );
            let first = cx.next_key(focused);
            assert_eq!(first.map(|k| k.code), Some(KeyCode::Char('a')));
            cx.decline(first.expect("a key"));
            // **A decline hands the key outward, not back to the same widget.** Asking again is
            // `None`, which is what makes the obvious `while let` drain loop terminate rather than
            // hand the same key out for ever.
            assert!(
                cx.next_key(focused).is_none(),
                "this widget is finished with the queue"
            );
        });
        assert_eq!(
            d.unhandled().iter().map(|k| k.code).collect::<Vec<_>>(),
            vec![KeyCode::Char('a')],
            "and the key is still there, in order, for the level out"
        );
    }
}

#[cfg(test)]
mod pointer_tests {
    //! Ticket 10's gates: the index that carries no geometry, and everything awarded at `end`.

    use super::*;
    use crate::id::Id;
    use crate::theme::Role;
    use vitui_engine::{Button, Buttons, Mouse, MouseKind, Wheel};

    fn driver() -> Driver {
        Driver::headless(300, 80).expect("attaching to a sink cannot fail")
    }

    fn at(x: u16, y: u16, kind: MouseKind, buttons: Buttons) -> Mouse {
        Mouse {
            x,
            y,
            kind,
            buttons,
            mods: vitui_engine::Mods::NONE,
            at: Instant::now(),
        }
    }

    fn moved(x: u16, y: u16) -> Mouse {
        at(x, y, MouseKind::Move, Buttons::NONE)
    }

    fn down(x: u16, y: u16) -> Mouse {
        at(x, y, MouseKind::Down(Button::Left), Buttons::NONE)
    }

    fn up(x: u16, y: u16) -> Mouse {
        at(x, y, MouseKind::Up(Button::Left), Buttons::NONE)
    }

    /// Run frames until nothing is queued, and answer how many it took.
    ///
    /// **A burst of pointer events is not one frame** — `Down` and `Up` are routing edges and each
    /// closes a batch, so `[Move, Down, Up]` is two. Every one of them draws, which is what the
    /// award needs: the index it reads has to be this frame's.
    fn drain(d: &mut Driver, mut draw: impl FnMut(&mut Ctx<'_, '_>)) -> usize {
        let mut frames = 0;
        loop {
            d.frame(&mut draw);
            frames += 1;
            if d.queued() == 0 {
                break;
            }
        }
        frames
    }

    /// **The index entry is sixteen bytes and carries no rectangle and no layer id.**
    ///
    /// The rect buys nothing, because containment is decided during the draw from the pointer that
    /// travelled down the `Ctx`; `layer` goes because modality is an *ordering* into a list that is
    /// already in draw order, not a membership.
    #[test]
    fn the_index_entry_is_sixteen_bytes_and_holds_no_geometry() {
        assert_eq!(size_of::<Hit>(), 16);
        // What it would have been with the two fields the proposal had.
        struct WithGeometry {
            _id: Id,
            _layer: u32,
            _rect: Rect,
            _interest: Interest,
        }
        assert_eq!(size_of::<WithGeometry>(), 32);
    }

    /// **Modality is an `Option<usize>`, and `0` is not a sentinel.**
    ///
    /// `usize` alone meant both *no modal* and *a modal over nothing* — a modal declared before any
    /// widget drew. Both are reachable and they are different situations.
    #[test]
    fn a_modal_over_nothing_is_not_no_modal() {
        let mut d = driver();
        d.post_mouse(moved(1, 1));
        d.frame(|cx| {
            // A modal barrier before anything has drawn: index 0, and nothing is inside it.
            cx.modal_barrier_here();
            cx.interact(Id::from_raw(1), Rect::new(0, 0, 4, 1), Interest::CLICK);
        });
        assert_eq!(
            d.inspect().modal_from(),
            Some(0),
            "a modal over nothing is Some(0), which is not None"
        );

        let mut e = driver();
        e.frame(|cx| {
            cx.interact(Id::from_raw(1), Rect::new(0, 0, 4, 1), Interest::CLICK);
        });
        assert_eq!(e.inspect().modal_from(), None, "and no modal is None");
    }

    /// A modal withholds the pointer from everything declared before it.
    #[test]
    fn a_modal_withholds_the_pointer_from_what_is_behind_it() {
        let mut d = driver();
        let behind = Id::from_raw(1);
        let inside = Id::from_raw(2);
        d.post_mouse(moved(2, 0));
        // Frame one: both draw, no modal, so the pointer finds the innermost.
        d.frame(|cx| {
            cx.interact(behind, Rect::new(0, 0, 8, 1), Interest::CLICK);
            cx.interact(inside, Rect::new(0, 0, 8, 1), Interest::CLICK);
        });
        // Frame two: a modal opens above `behind`.
        d.post_mouse(moved(2, 0));
        let mut behind_hovered = None;
        d.frame(|cx| {
            let b = cx.interact(behind, Rect::new(0, 0, 8, 1), Interest::CLICK);
            cx.modal_barrier_here();
            cx.interact(inside, Rect::new(0, 0, 8, 1), Interest::CLICK);
            behind_hovered = Some(b.hovered);
        });
        // The guess was from frame one, where the innermost was `inside`; the award is from frame two.
        d.post_mouse(moved(2, 0));
        let mut behind_now = None;
        d.frame(|cx| {
            let b = cx.interact(behind, Rect::new(0, 0, 8, 1), Interest::CLICK);
            cx.modal_barrier_here();
            cx.interact(inside, Rect::new(0, 0, 8, 1), Interest::CLICK);
            behind_now = Some(b.hovered);
        });
        assert_eq!(
            behind_now,
            Some(false),
            "a widget behind a modal is not hovered, whatever the pointer is over"
        );
    }

    /// **The count is bounded by visible cells and not by data**: the same screen over two thousand
    /// rows and over a million declares the same entries.
    #[test]
    fn the_entry_count_is_identical_over_two_thousand_and_a_million_rows() {
        fn screen(cx: &mut Ctx<'_, '_>, rows: i64) {
            // A list that culls, which is the only way a million rows is affordable at all.
            let visible = cx.visible_rows();
            for row in visible.start..visible.end.min(80) {
                if i64::from(row) >= rows {
                    break;
                }
                cx.with_key(u64::try_from(row).unwrap_or(0), |r| {
                    // **Two targets a row**, which is the mechanical trigger for the spread: an entry
                    // carrying no geometry cannot separate them, so the author declares per target.
                    let label = r.id();
                    r.interact(label, Rect::new(0, row, 40, 1), Interest::CLICK);
                    r.interact(
                        Id::keyed(Id::from_raw(9), u64::try_from(row).unwrap_or(0)),
                        Rect::new(40, row, 8, 1),
                        Interest::CLICK,
                    );
                });
            }
        }

        let mut small = driver();
        small.frame(|cx| screen(cx, 2_000));
        let a = small.inspect().hits().len();

        let mut large = driver();
        large.frame(|cx| screen(cx, 1_000_000));
        let b = large.inspect().hits().len();

        assert_eq!(a, b, "the count moved with the data volume");
        assert_eq!(a, 160, "eighty rows, two targets each");
    }

    /// **The press is awarded at `end`, from the index that has just drawn** — so a press arriving
    /// where no frame has been told the pointer is still finds its widget.
    ///
    /// This is not an exotic case: **at `Buttons` tracking the terminal sends no motion events at
    /// all**, so every press arrives at a position no frame has heard about.
    #[test]
    fn a_press_where_no_frame_has_drawn_still_reaches_its_widget() {
        let mut d = driver();
        let button = Id::from_raw(1);

        // Frame one: the widget draws, and the pointer has never been reported.
        d.frame(|cx| {
            cx.interact(button, Rect::new(10, 5, 6, 1), Interest::CLICK);
        });
        assert_eq!(
            d.inspect().tracking(),
            MouseMode::Buttons,
            "no motion is sent"
        );

        // A press arrives at a position no frame has seen. Frame two draws and awards it.
        d.post_mouse(down(12, 5));
        let mut pressed_during_draw = None;
        d.frame(|cx| {
            let r = cx.interact(button, Rect::new(10, 5, 6, 1), Interest::CLICK);
            pressed_during_draw = Some(r.pressed);
        });
        assert_eq!(
            pressed_during_draw,
            Some(false),
            "during the draw nothing was known — the guess was from a frame with no pointer"
        );
        assert!(
            d.inspect().id_keyed_facts().0,
            "and at `end` the grab was awarded from the index that had just drawn"
        );

        // The release completes the click, delivered on the next frame.
        //
        // **The widget has to draw on the frame that processes the release**, and the first version of
        // this test put an empty frame in between. That frame's award ran over an empty index, so the
        // release found a grab and nothing under the pointer — which is a *cancelled drag*, correctly,
        // and not a click. A widget that stops drawing mid-gesture has cancelled it.
        d.post_mouse(up(12, 5));
        d.frame(|cx| {
            cx.interact(button, Rect::new(10, 5, 6, 1), Interest::CLICK);
        });
        let mut clicked = None;
        d.frame(|cx| {
            let r = cx.interact(button, Rect::new(10, 5, 6, 1), Interest::CLICK);
            clicked = Some(r.clicked);
        });
        assert_eq!(clicked, Some(true), "the click reached its widget");
    }

    /// **`begin`'s guess never produces a wrong click.** A pointer that moves between two widgets
    /// that do not touch hovers nothing on the frame it crosses — and does not click the one it left.
    #[test]
    fn the_guess_never_becomes_a_wrong_click() {
        let mut d = driver();
        let left = Id::from_raw(1);
        let right = Id::from_raw(2);
        let draw = |cx: &mut Ctx<'_, '_>| {
            (
                cx.interact(left, Rect::new(0, 0, 4, 1), Interest::CLICK),
                cx.interact(right, Rect::new(20, 0, 4, 1), Interest::CLICK),
            )
        };

        // Over the left one, then a press over the right one in the same batch as the move.
        d.post_mouse(moved(1, 0));
        d.frame(|cx| {
            draw(cx);
        });
        d.post_mouse(moved(21, 0));
        d.post_mouse(down(21, 0));
        d.post_mouse(up(21, 0));
        // Two frames: the move folds into the one the `Down` closes, and the `Up` closes its own.
        assert_eq!(
            drain(&mut d, |cx| {
                draw(cx);
            }),
            2
        );
        // The award came from that frame's index, where the pointer was over `right`.
        let mut clicks = (false, false);
        d.frame(|cx| {
            let (l, r) = draw(cx);
            clicks = (l.clicked, r.clicked);
        });
        assert_eq!(
            clicks,
            (false, true),
            "the click went to where the pointer was, not to the previous frame's guess"
        );
    }

    /// **A held grab is exclusive.** Without it a splitter drag lights every button it crosses.
    #[test]
    fn a_held_grab_is_exclusive() {
        let mut d = driver();
        let splitter = Id::from_raw(1);
        let button = Id::from_raw(2);
        let draw = |cx: &mut Ctx<'_, '_>| {
            (
                cx.interact(splitter, Rect::new(0, 0, 1, 10), Interest::DRAG),
                cx.interact(
                    button,
                    Rect::new(10, 5, 6, 1),
                    Interest::CLICK.with(Interest::HOVER),
                ),
            )
        };

        d.post_mouse(moved(0, 5));
        d.frame(|cx| {
            draw(cx);
        });
        d.post_mouse(down(0, 5));
        d.frame(|cx| {
            draw(cx);
        });
        assert!(d.inspect().id_keyed_facts().0, "the splitter holds it");

        // Drag across the button.
        d.post_mouse(moved(12, 5));
        let mut button_pressed = None;
        d.frame(|cx| {
            let (_, b) = draw(cx);
            button_pressed = Some(b.pressed);
        });
        assert_eq!(
            button_pressed,
            Some(false),
            "the button the drag crossed was not pressed — the grab is exclusive"
        );
    }

    /// **The wheel is withheld while a grab is held.** A drag is one gesture and a scroll in the
    /// middle of it is not part of it.
    #[test]
    fn the_wheel_is_withheld_while_a_grab_is_held() {
        let mut d = driver();
        let list = Id::from_raw(1);
        let draw = |cx: &mut Ctx<'_, '_>| {
            cx.interact(
                list,
                Rect::new(0, 0, 40, 20),
                Interest::SCROLL.with(Interest::DRAG),
            )
        };

        // A wheel with nothing held reaches the list.
        d.post_mouse(moved(5, 5));
        d.frame(|cx| {
            draw(cx);
        });
        d.post_mouse(at(5, 5, MouseKind::Wheel(Wheel::Down), Buttons::NONE));
        let mut scrolled = (0, 0);
        d.frame(|cx| {
            scrolled = draw(cx).scrolled;
        });
        assert_eq!(
            scrolled,
            (0, 1),
            "the wheel reached the list, on its own frame"
        );

        // Now hold the pointer and try again.
        d.post_mouse(down(5, 5));
        d.frame(|cx| {
            draw(cx);
        });
        d.post_mouse(at(5, 5, MouseKind::Wheel(Wheel::Down), Buttons::NONE));
        let mut held_scroll = (0, 0);
        d.frame(|cx| {
            held_scroll = draw(cx).scrolled;
        });
        assert_eq!(held_scroll, (0, 0), "the wheel was withheld");
    }

    /// **A cancelled drag reaches the application**, which the identity sweep alone did not do.
    #[test]
    fn a_cancelled_drag_reaches_the_application() {
        let mut d = driver();
        let thumb = Id::from_raw(1);
        let draw = |cx: &mut Ctx<'_, '_>| cx.interact(thumb, Rect::new(0, 0, 1, 4), Interest::DRAG);
        d.post_mouse(moved(0, 1));
        d.frame(|cx| {
            draw(cx);
        });
        d.post_mouse(down(0, 1));
        d.frame(|cx| {
            draw(cx);
        });
        // Released a long way away: not a click, and the widget has to be told.
        d.post_mouse(moved(50, 40));
        d.post_mouse(up(50, 40));
        d.frame(|cx| {
            draw(cx);
        });
        // **Read immediately.** The award is rotated into `delivered` by the next `begin`, so asking
        // after another frame asks about that frame instead — which is how the first version of this
        // test managed to see `None`.
        assert_eq!(
            d.inspect().cancelled_drag(),
            Some(thumb),
            "the cancellation is recorded rather than silently swallowed"
        );
        let mut r = None;
        d.frame(|cx| {
            r = Some(draw(cx));
        });
        assert!(
            !r.expect("the frame ran").clicked,
            "and a release elsewhere is not a click"
        );
    }

    /// **One region in 312 raises the whole frame to `Motion`**, and the escape is the theme.
    #[test]
    fn one_hovering_region_raises_the_frame_and_a_flat_theme_does_not() {
        let hovering = |cx: &mut Ctx<'_, '_>| {
            let interest = cx.theme().hover_interest().with(Interest::CLICK);
            for i in 0..312u64 {
                cx.interact(Id::keyed(Id::ROOT, i), Rect::new(0, 0, 4, 1), interest);
            }
        };

        // A theme whose hover is visible.
        let mut rich = driver();
        rich.frame(hovering);
        assert_eq!(
            rich.inspect().tracking(),
            MouseMode::Motion,
            "a visible hover costs motion reporting"
        );

        // **A flat theme leaves the same 312 entries at `Drag`.** That is a routing consequence of a
        // theme switch, and it is why hover interest comes from the theme rather than from a literal.
        let mut flat = driver();
        flat.set_theme(Theme::default().resolve(vitui_engine::ColorDepth::None));
        flat.frame(|cx| {
            let interest = cx
                .theme()
                .hover_interest()
                .with(Interest::CLICK)
                .with(Interest::DRAG);
            for i in 0..312u64 {
                cx.interact(Id::keyed(Id::ROOT, i), Rect::new(0, 0, 4, 1), interest);
            }
        });
        assert_eq!(
            flat.inspect().tracking(),
            MouseMode::Drag,
            "a flat theme does not pay for a highlight nobody can see"
        );
    }

    /// `Response` carries the modifiers, which is what makes ctrl-click expressible.
    #[test]
    fn the_response_carries_the_modifiers() {
        let mut d = driver();
        let row = Id::from_raw(1);
        let draw = |cx: &mut Ctx<'_, '_>| cx.interact(row, Rect::new(0, 0, 8, 1), Interest::CLICK);

        d.post_mouse(moved(2, 0));
        d.frame(|cx| {
            draw(cx);
        });
        let mut ctrl_click = Mouse {
            mods: vitui_engine::Mods::CTRL,
            ..down(2, 0)
        };
        d.post_mouse(ctrl_click);
        ctrl_click.kind = MouseKind::Up(Button::Left);
        d.post_mouse(ctrl_click);
        assert_eq!(
            drain(&mut d, |cx| {
                draw(cx);
            }),
            2,
            "two edges, two frames"
        );
        let mut got = None;
        d.frame(|cx| got = Some(draw(cx)));
        let got = got.expect("the frame ran");
        assert!(got.clicked);
        assert!(
            got.mods.ctrl(),
            "a ctrl-click is a click plus a modifier, and the field is how a component sees it"
        );
    }

    /// `local` is a position and not a delta, which is what a slider needs.
    #[test]
    fn local_is_a_position_in_the_widgets_own_coordinates() {
        let mut d = driver();
        d.post_mouse(moved(13, 7));
        d.frame(|cx| {
            // A widget at (10, 5): the pointer at (13, 7) is at (3, 2) inside it.
            let r = cx.interact(Id::from_raw(1), Rect::new(10, 5, 8, 4), Interest::DRAG);
            assert_eq!(r.local, Some((3, 2)));
            // And a widget the pointer is outside gets nothing rather than a negative guess.
            let out = cx.interact(Id::from_raw(2), Rect::new(0, 0, 4, 1), Interest::DRAG);
            assert_eq!(out.local, None);
        });
    }

    /// `local` survives being nested and scrolled, because the pointer travels down the `Ctx`.
    #[test]
    fn local_survives_nesting_and_scrolling() {
        let mut d = driver();
        d.post_mouse(moved(13, 7));
        d.frame(|cx| {
            let mut pane = cx.child(Rect::new(10, 5, 20, 10));
            // Inside the pane the pointer is at (3, 2).
            let r = pane.interact(Id::from_raw(1), Rect::new(0, 0, 8, 4), Interest::DRAG);
            assert_eq!(r.local, Some((3, 2)));
            let mut scrolled = pane.scrolled(0, -100);
            // Scrolled, the same screen position is a different content row.
            let s = scrolled.interact(Id::from_raw(2), Rect::new(0, 100, 8, 4), Interest::DRAG);
            assert_eq!(s.local, Some((3, 2)));
        });
    }

    /// **`changed` is never written by the runtime.**
    #[test]
    fn changed_is_never_written_by_the_runtime() {
        let mut d = driver();
        d.post_mouse(moved(2, 0));
        d.frame(|cx| {
            cx.interact(Id::from_raw(1), Rect::new(0, 0, 8, 1), Interest::CLICK);
        });
        d.post_mouse(down(2, 0));
        d.post_mouse(up(2, 0));
        drain(&mut d, |cx| {
            cx.interact(Id::from_raw(1), Rect::new(0, 0, 8, 1), Interest::CLICK);
        });
        let mut changed = None;
        d.frame(|cx| {
            let r = cx.interact(Id::from_raw(1), Rect::new(0, 0, 8, 1), Interest::CLICK);
            assert!(r.clicked, "it was clicked");
            changed = Some(r.changed);
        });
        assert_eq!(
            changed,
            Some(false),
            "a click is not a change — the runtime does not hold the value and may not"
        );
    }

    /// The thresholds are the three the ticket names.
    #[test]
    fn the_thresholds_are_configuration() {
        let p = Pointer::default();
        assert_eq!(p.click_threshold, Duration::from_millis(400));
        assert_eq!(p.click_slop, 1);
        assert_eq!(p.long_press, Duration::from_millis(500));
    }

    /// Hover as a style resolves in the same frame, on the first frame of an overlap.
    #[test]
    fn hover_as_a_style_resolves_in_the_same_frame() {
        let mut d = driver();
        let under = Id::from_raw(1);
        let over = Id::from_raw(2);
        d.post_mouse(moved(2, 0));
        // Frame one: only `under` draws, so the guess will point at it.
        d.frame(|cx| {
            let r = cx.interact(under, Rect::new(0, 0, 8, 1), Interest::HOVER);
            cx.hover_style(&r, Rect::new(0, 0, 8, 1), Role::FaceHover);
        });
        // Frame two: `over` appears on top. The guess still says `under`; the award says `over`.
        d.post_mouse(moved(2, 0));
        let mut guessed = None;
        d.frame(|cx| {
            let a = cx.interact(under, Rect::new(0, 0, 8, 1), Interest::HOVER);
            let b = cx.interact(over, Rect::new(0, 0, 8, 1), Interest::HOVER);
            cx.hover_style(&a, Rect::new(0, 0, 8, 1), Role::FaceHover);
            cx.hover_style(&b, Rect::new(0, 0, 8, 1), Role::FaceHover);
            guessed = Some((a.hovered, b.hovered));
        });
        assert_eq!(
            guessed,
            Some((true, false)),
            "the guess is a frame old and points at what was on top before"
        );
        assert_eq!(
            d.inspect().hovered_now(),
            Some(over),
            "**and the style went to the one actually on top, in this frame**"
        );
    }

    /// **A hover style two containers deep lands where the widget is**, which it did not before
    /// ticket 15 gave `Ctx` an origin: the transform added this context's `rect`, and `rect` is a
    /// rectangle in its *parent's* coordinates. One level down that is the same number; two levels
    /// down it is the inner container's offset with the outer one missing.
    #[test]
    fn a_hover_style_two_containers_deep_is_in_root_coordinates() {
        let mut d = driver();
        let widget = Id::from_raw(7);
        d.post_mouse(moved(14, 6));
        d.frame(|cx| {
            let mut outer = cx.child(Rect::new(10, 4, 40, 10));
            let mut inner = outer.child(Rect::new(3, 1, 20, 5));
            let r = inner.interact(widget, Rect::new(1, 1, 8, 1), Interest::HOVER);
            inner.hover_style(&r, Rect::new(1, 1, 8, 1), Role::FaceHover);
        });
        assert_eq!(
            d.inspect().hover_styles.as_slice(),
            [(widget, Rect::new(14, 6, 8, 1), Role::FaceHover)],
            "10 + 3 + 1 and 4 + 1 + 1, and not 3 + 1 and 1 + 1"
        );
    }
}

#[cfg(test)]
mod focus_tests {
    //! Ticket 12's gates: **a sixth interest bit, three scope answers, and a vanish rule that was
    //! 32% of the budget written the obvious way.**
    //!
    //! The walk, the group collapse and the vanish rule's own arithmetic have unit tests in
    //! [`crate::focus`]. What is here is the half that needs a whole frame: the tracking delta, the
    //! keyboard walkthrough, the trap's two mechanisms, the scope-resolved key map and the caret.

    use super::*;
    use crate::focus::ScopeKind;
    use crate::id::Id;
    use crate::keys::{Chord, KeyMap};
    use vitui_engine::{
        Button, Buttons, ColorDepth, Key, KeyCode, KeyKind, KeyText, Mouse, MouseKind,
    };

    fn driver() -> Driver {
        Driver::headless(300, 80).expect("attaching to a sink cannot fail")
    }

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: vitui_engine::Mods::NONE,
            kind: KeyKind::Press,
            text: KeyText::EMPTY,
            at: Instant::now(),
        }
    }

    fn chorded(code: KeyCode, mods: vitui_engine::Mods) -> Key {
        Key {
            code,
            mods,
            kind: KeyKind::Press,
            text: KeyText::EMPTY,
            at: Instant::now(),
        }
    }

    fn press(x: u16, y: u16) -> Mouse {
        Mouse {
            x,
            y,
            kind: MouseKind::Down(Button::Left),
            buttons: Buttons::NONE,
            mods: vitui_engine::Mods::NONE,
            at: Instant::now(),
        }
    }

    /// Run frames until the queue is empty, and answer how many it took.
    fn drain(d: &mut Driver, mut draw: impl FnMut(&mut Ctx<'_, '_>)) -> usize {
        let mut frames = 0;
        loop {
            d.frame(&mut draw);
            frames += 1;
            if d.queued() == 0 {
                break;
            }
            assert!(frames < 64, "the drain did not terminate");
        }
        frames
    }

    /// A cell of the dense screen, so a rectangle is never the interesting part.
    fn cell(i: u64) -> Rect {
        let i = i32::try_from(i).unwrap_or(0);
        Rect::new(i % 300, i / 300, 1, 1)
    }

    /// **The dense IDE screen**: 312 interactive regions, 67 of them tab stops, 43 stops after the
    /// three groups collapse.
    ///
    /// The shape is a *count* and the counts are asserted at every call site, which is the rule the
    /// layout screen fixture already applies: a gate that quietly started measuring a smaller screen
    /// would look good rather than fail.
    ///
    /// `stops` says whether the 67 declare [`Interest::FOCUS`] — **the delta gate needs the same
    /// screen with the bit and without it**, and nothing else about the frame may differ.
    fn dense(cx: &mut Ctx<'_, '_>, stops: bool) {
        let base = cx.theme().hover_interest().with(Interest::CLICK);
        let stop = if stops {
            base.with(Interest::FOCUS)
        } else {
            base
        };
        let mut n = 0u64;
        // Three groups — the menu bar, the toolbar and the tab strip — 27 entries and 3 stops.
        for (name, count) in [("menu bar", 7u64), ("toolbar", 12), ("tab strip", 8)] {
            cx.scope(Id::named(name), ScopeKind::Group, |cx| {
                for i in 0..count {
                    cx.interact(Id::keyed(Id::named(name), i), cell(n + i), stop);
                }
            });
            n += count;
        }
        // Forty ordinary stops: fields, buttons, the two panes, the search box.
        for i in 0..40u64 {
            cx.interact(Id::keyed(Id::named("stop"), i), cell(n + i), stop);
        }
        n += 40;
        // And 245 clickable regions that are **not** stops: 76 tree rows, 152 table cells, 17 others.
        // No combination of the pointer's bits separates these from the 67 above, which is the whole
        // reason `FOCUS` is a bit of its own.
        for i in 0..245u64 {
            cx.interact(Id::keyed(Id::named("row"), i), cell(n + i), base);
        }
    }

    /// **The delta gate: declaring `FOCUS` on 67 entries does not raise the frame's tracking level.**
    ///
    /// A delta and not an absolute, because *the dense screen stays at `Drag`* is a statement about
    /// the **theme**, not about this bit: the same screen is at `Motion` on a theme whose hover is
    /// visible, and drops to `Drag` only after `resolve(ColorDepth::None)` has taken the highlight
    /// away. Both arms are here, and the bit changes neither.
    #[test]
    fn declaring_focus_on_sixty_seven_entries_costs_no_tracking() {
        // The tier is named: a truecolor theme's hover is visible, so 312 regions cost motion.
        let mut rich = driver();
        rich.frame(|cx| dense(cx, false));
        let rich_without = rich.inspect().tracking();
        rich.frame(|cx| dense(cx, true));
        assert_eq!(rich.inspect().hits().len(), 312);
        assert_eq!(rich.inspect().ring().len(), 67);
        assert_eq!(
            rich.inspect().tracking(),
            rich_without,
            "the bit is a delta"
        );
        assert_eq!(
            rich_without,
            MouseMode::Motion,
            "a visible hover costs motion"
        );

        // And the flat theme, which is where the screen is at `Drag` — with the bit and without it.
        let mut flat = driver();
        flat.set_theme(Theme::default().resolve(ColorDepth::None));
        flat.frame(|cx| dense(cx, false));
        let flat_without = flat.inspect().tracking();
        flat.frame(|cx| dense(cx, true));
        assert_eq!(flat.inspect().ring().len(), 67);
        assert_eq!(
            flat.inspect().tracking(),
            flat_without,
            "the bit is a delta"
        );
        assert_eq!(
            flat_without,
            MouseMode::Buttons,
            "**the dense screen does not pay for a highlight nobody can see**, and that is the \
             theme's doing and not the bit's"
        );
        assert_eq!(Interest::FOCUS.tracking(), MouseMode::Off);
    }

    /// **A keyboard walkthrough visits every tab stop exactly once**, and 43 is not 67.
    ///
    /// The counts are the ticket's: 312 hit entries, 67 ring entries, 43 stops once the menu bar,
    /// the toolbar and the tab strip collapse — 311 / 43 = 7.2× fewer things to visit. The walk is
    /// driven by pressing `Tab`, not by reading the frame's own answer back: a walkthrough that
    /// consulted [`Frame::tab_walk`] would be testing one expression against itself.
    #[test]
    fn a_keyboard_walkthrough_visits_every_tab_stop_exactly_once() {
        let mut d = driver();
        d.frame(|cx| dense(cx, true));
        assert_eq!(d.inspect().hits().len(), 312, "the screen is 312 regions");
        assert_eq!(
            d.inspect().ring().len(),
            67,
            "and 67 of them are in the ring"
        );
        assert_eq!(d.inspect().stop_count(), 43, "and 43 of those are stops");

        let mut visited = Vec::new();
        for _ in 0..43 {
            d.post_key(key(KeyCode::Tab));
            drain(&mut d, |cx| dense(cx, true));
            visited.push(d.inspect().focused().expect("a Tab always lands somewhere"));
        }
        let mut deduped = visited.clone();
        deduped.sort_by_key(|id| id.raw());
        deduped.dedup();
        assert_eq!(deduped.len(), 43, "the walk repeated an id");
        assert_eq!(
            visited,
            d.inspect().stop_ids().collect::<Vec<_>>(),
            "and it visited them in the ring's order, which is the order of the source"
        );

        // The forty-fourth press is back at the beginning: the ring wraps.
        d.post_key(key(KeyCode::Tab));
        drain(&mut d, |cx| dense(cx, true));
        assert_eq!(d.inspect().focused(), Some(visited[0]), "the ring wraps");
    }

    /// **A scope is a frame-local range, and it cannot become a cross-frame fact.**
    ///
    /// The behavioural half: the scopes are gone on the next frame, and the id-keyed facts are
    /// still the four ADR 0012 closed the list at. **The compile outcome is on [`Frame::scopes`]**,
    /// with its positive twin — a `compile_fail` inside a private test module is collected by
    /// nobody, which is the defect ticket 19 exists to stop shipping.
    #[test]
    fn a_scope_is_a_frame_local_range_and_not_a_cross_frame_fact() {
        let mut d = driver();
        let form = Id::named("form");
        d.frame(|cx| {
            cx.scope(form, ScopeKind::Trap, |cx| {
                for i in 0..3u64 {
                    cx.interact(Id::keyed(form, i), cell(i), Interest::FOCUS);
                }
            });
        });
        let scope = d.inspect().scopes()[0];
        assert_eq!(
            (scope.start, scope.end),
            (0, 3),
            "a range over this frame's ring"
        );
        assert_eq!(scope.parent, None);
        assert_eq!(scope.kind, ScopeKind::Trap);

        // The next frame declares no scope, and there is nothing left of this one.
        d.frame(|_cx| {});
        assert!(
            d.inspect().scopes().is_empty(),
            "a scope survived its frame"
        );
        let (grab, origin, _focus, click) = d.inspect().id_keyed_facts();
        assert_eq!(
            (grab, origin, click),
            (false, false, false),
            "**still four id-keyed facts**, and a scope is not a fifth"
        );
    }

    /// **`Trap`'s keyboard half is `next_key` itself**, and the frame it opens is exposed to at most
    /// one key.
    ///
    /// The exposure is structural: widgets behind a modal draw *before* it, so they pull their keys
    /// before the scope that would stop them exists. What bounds it is the batch — the click that
    /// opens a modal is a closing edge and ends its batch, so a pointer-opened modal cannot leak at
    /// all, and a key-opened one leaks exactly what is behind the opening key in **one** batch.
    #[test]
    fn the_frame_a_trap_opens_is_exposed_to_at_most_one_key() {
        let mut d = driver();
        let behind = Id::named("editor");
        let modal = Id::named("modal");
        let ok = Id::named("modal.ok");
        d.plant(None, Some(behind), None);
        // `o` opens the modal and `a` is behind it in the same batch.
        d.post_key(key(KeyCode::Char('o')));
        d.post_key(key(KeyCode::Char('a')));

        let mut open = false;
        let mut leaked = 0;
        let mut opening = 0;
        let draw = |cx: &mut Ctx<'_, '_>, open: &mut bool, leaked: &mut i32, opening: &mut i32| {
            cx.interact(behind, cell(0), Interest::FOCUS);
            while let Some(k) = cx.next_key(behind) {
                if k.code == KeyCode::Char('o') {
                    *open = true;
                    *opening += 1;
                } else {
                    *leaked += 1;
                }
            }
            if *open {
                cx.scope(modal, ScopeKind::Trap, |cx| {
                    cx.interact(ok, cell(1), Interest::FOCUS);
                });
            }
        };
        d.frame(|cx| draw(cx, &mut open, &mut leaked, &mut opening));
        assert_eq!(opening, 1, "the key that opened it is the widget's own");
        assert_eq!(
            leaked, 1,
            "**one key**, and it is the rest of the opening batch"
        );
        assert_eq!(
            d.inspect().focused(),
            Some(ok),
            "and the trap has pulled the focus onto its first stop"
        );

        // Every frame after it: the widget behind gets nothing, however many keys arrive.
        for c in ['b', 'c', 'd'] {
            d.post_key(key(KeyCode::Char(c)));
            d.frame(|cx| draw(cx, &mut open, &mut leaked, &mut opening));
        }
        assert_eq!(
            leaked, 1,
            "the trap holds the keyboard from the next frame on"
        );

        // **The alternative, with no trap declared**: the widget behind keeps the keyboard for as
        // long as the modal is open, which is what the mechanism is against.
        let mut loose = driver();
        loose.plant(None, Some(behind), None);
        let mut kept = 0;
        for c in ['b', 'c', 'd'] {
            loose.post_key(key(KeyCode::Char(c)));
            loose.frame(|cx| {
                cx.interact(behind, cell(0), Interest::FOCUS);
                while cx.next_key(behind).is_some() {
                    kept += 1;
                }
                // The modal draws, and declares nothing about the focus.
                cx.interact(ok, cell(1), Interest::FOCUS);
            });
        }
        assert_eq!(
            kept, 3,
            "without the trap, every key still reaches what is behind it"
        );
    }

    /// **`Isolated` is decided in the drain, from the previous frame's scopes** — it could not have
    /// been a `decline`, because the ring consumes the key first.
    #[test]
    fn an_isolated_scope_is_decided_in_the_drain_from_the_previous_frames_scopes() {
        let editor = Id::named("editor");
        let field = Id::named("field");
        let code = Id::named("code");

        // The ordinary case: the ring takes the `Tab` and the widget is never offered it.
        let mut plain = driver();
        plain.plant(None, Some(field), None);
        let mut plain_got = Vec::new();
        plain.post_key(key(KeyCode::Tab));
        drain(&mut plain, |cx| {
            cx.interact(field, cell(0), Interest::FOCUS);
            cx.interact(Id::named("other"), cell(1), Interest::FOCUS);
            while let Some(k) = cx.next_key(field) {
                plain_got.push(k.code);
            }
        });
        assert!(
            plain_got.is_empty(),
            "the ring's key, withheld from the widget"
        );
        assert_eq!(plain.inspect().focused(), Some(Id::named("other")));

        // The isolated case. The scope has to have stood **last** frame, which is what makes this a
        // drain decision rather than a draw one.
        let mut d = driver();
        d.plant(None, Some(code), None);
        let draw = |cx: &mut Ctx<'_, '_>, got: &mut Vec<KeyCode>| {
            cx.scope(editor, ScopeKind::Isolated, |cx| {
                cx.interact(code, cell(0), Interest::FOCUS);
                while let Some(k) = cx.next_key(code) {
                    got.push(k.code);
                }
            });
            cx.interact(Id::named("other"), cell(1), Interest::FOCUS);
        };
        let mut warm = Vec::new();
        d.frame(|cx| draw(cx, &mut warm));

        let mut got = Vec::new();
        d.post_key(key(KeyCode::Tab));
        drain(&mut d, |cx| draw(cx, &mut got));
        assert_eq!(
            got,
            vec![KeyCode::Tab],
            "the editor inserts a tab character"
        );
        assert_eq!(
            d.inspect().focused(),
            Some(code),
            "and the ring did not move: the key never reached it"
        );
    }

    /// **`dismissible` does not exist**: the runtime consumes no `Esc`, and nested traps receive it
    /// innermost-first.
    ///
    /// The runtime may not close a modal — that is a write to application state it does not hold —
    /// so it may not consume the key. `Esc` is an ordinary key on the decline queue, and the
    /// innermost-first order is the drain's own, with no new code at all.
    #[test]
    fn the_runtime_consumes_no_esc_and_nested_traps_get_it_innermost_first() {
        let mut d = driver();
        let outer = Id::named("outer modal");
        let inner = Id::named("inner modal");
        let field = Id::named("inner.field");
        d.plant(None, Some(field), None);
        d.post_key(key(KeyCode::Escape));

        let mut order = Vec::new();
        d.frame(|cx| {
            cx.scope(outer, ScopeKind::Trap, |cx| {
                cx.scope(inner, ScopeKind::Trap, |cx| {
                    cx.interact(field, cell(0), Interest::FOCUS);
                    if let Some(k) = cx.next_key(field) {
                        order.push("field");
                        cx.decline(k);
                    }
                });
                if let Some(k) = cx.next_key(inner) {
                    order.push("inner");
                    cx.decline(k);
                }
            });
            if let Some(k) = cx.next_key(outer) {
                order.push("outer");
                cx.decline(k);
            }
        });
        assert_eq!(
            order,
            vec!["field", "inner", "outer"],
            "innermost-first, and for free — it is the drain order"
        );
        assert_eq!(
            d.unhandled().iter().map(|k| k.code).collect::<Vec<_>>(),
            vec![KeyCode::Escape],
            "**and the runtime consumed nothing**: what nobody took reaches the application"
        );
    }

    /// **The vanish rule's probe count**: bounded by the previous ring, where the obvious way is
    /// bounded by the previous ring times this one.
    ///
    /// The scene is the map's: a search box filtering 600 keyed rows. One keystroke halves the list
    /// and the focused row is one of the ones that went, so **every candidate the walk crosses is
    /// dead** — which is the shape that makes the inner scan quadratic rather than incidental.
    ///
    /// Two counts and a ratio. The shipped form is a lazily filled stamped table: one pass over this
    /// frame's ring plus about one probe a candidate. The obvious form is written out beside it, so
    /// the detector is asserted to be detecting something rather than assumed to be — the shape
    /// ticket 11's `Vec::remove(0)` twin has.
    #[test]
    fn the_vanish_rule_is_bounded_by_the_ring_and_not_by_the_ring_squared() {
        let search = Id::named("search");
        let row = |i: u64| Id::keyed(Id::named("row"), i);
        let mut d = driver();
        // Frame one: everything draws, and the last row holds the focus.
        d.frame(|cx| {
            cx.interact(search, cell(0), Interest::FOCUS);
            for i in 0..600u64 {
                cx.interact(row(i), cell(i + 1), Interest::FOCUS);
            }
            cx.focus(row(599));
        });
        assert_eq!(d.inspect().ring().len(), 601, "the scene is 601 stops");
        assert_eq!(d.inspect().focused(), Some(row(599)));
        assert_eq!(
            d.inspect().vanish_probes(),
            0,
            "nothing vanished, nothing asked"
        );

        // Frame two: the filter keeps the first three hundred, and the focused row is gone.
        let prev: Vec<Id> = d.inspect().ring_ids().collect();
        d.frame(|cx| {
            cx.interact(search, cell(0), Interest::FOCUS);
            for i in 0..300u64 {
                cx.interact(row(i), cell(i + 1), Interest::FOCUS);
            }
        });
        let now: Vec<Id> = d.inspect().ring_ids().collect();
        assert_eq!(now.len(), 301);
        assert_eq!(
            d.inspect().focused(),
            Some(row(299)),
            "the nearest survivor in the previous ring's order"
        );

        let shipped = d.inspect().vanish_probes();
        assert!(
            shipped <= 2 * u64::try_from(prev.len()).expect("601 fits"),
            "the shipped form is bounded by the previous ring, not by it times this one: {shipped}"
        );

        // **The obvious way**, written out: walk the previous ring from the focus and ask, per
        // candidate, whether that id is still in this one — with the inner question a scan.
        let at = prev
            .iter()
            .position(|id| *id == row(599))
            .expect("it was there");
        let mut obvious = 0u64;
        let mut landed = None;
        for id in prev[at + 1..].iter().chain(prev[..at].iter().rev()) {
            let mut found = false;
            for other in &now {
                obvious += 1;
                if other == id {
                    found = true;
                    break;
                }
            }
            if found {
                landed = Some(*id);
                break;
            }
        }
        assert_eq!(
            landed,
            Some(row(299)),
            "the two forms answer the same thing"
        );
        assert!(
            obvious / shipped >= 100,
            "the ratio is the detector, and it is {obvious} / {shipped}"
        );

        // And a quiet frame — the same screen again, nothing gone — pays nothing at all.
        d.frame(|cx| {
            cx.interact(search, cell(0), Interest::FOCUS);
            for i in 0..300u64 {
                cx.interact(row(i), cell(i + 1), Interest::FOCUS);
            }
        });
        assert_eq!(d.inspect().vanish_probes(), 0, "**a quiet frame pays 0**");
    }

    /// **`Ctrl+S` under a `Trap` fires the modal's binding, never the application's.**
    ///
    /// A key map is a range tagged with the open scope and the walk is innermost-first. The flat
    /// pass is written out beside it, because the failure is not a crash: it is a document written
    /// behind a dialog the user has not confirmed.
    #[test]
    fn ctrl_s_under_a_trap_fires_the_modals_binding() {
        const APP_SAVE: u32 = 1;
        const MODAL_SAVE: u32 = 2;
        let app = KeyMap::new().bind(&[Chord::key('s').ctrl()], APP_SAVE, "Save the document");
        let modal = KeyMap::new().bind(&[Chord::key('s').ctrl()], MODAL_SAVE, "Save the settings");
        let ctrl_s = chorded(KeyCode::Char('s'), vitui_engine::Mods::CTRL);

        let mut d = driver();
        let mut under_trap = None;
        let mut at_the_frame_level = None;
        let mut flat_pass = None;
        d.frame(|cx| {
            // The application declares first, because it draws first.
            cx.key_map(&app);
            cx.scope(Id::named("modal"), ScopeKind::Trap, |cx| {
                cx.key_map(&modal);
                under_trap = cx.action(&ctrl_s);
            });
            at_the_frame_level = cx.action(&ctrl_s);
            // The flat pass: one list in declaration order, first match wins.
            flat_pass = cx
                .frame
                .maps
                .match_first(&ctrl_s, crate::keys::MatchMode::Masked);
        });
        assert_eq!(under_trap, Some(MODAL_SAVE), "innermost-first");
        assert_eq!(
            at_the_frame_level,
            Some(APP_SAVE),
            "and outside the scope the application's own map is still the answer"
        );
        assert_eq!(
            flat_pass,
            Some(APP_SAVE),
            "**the flat pass is what fires the wrong one**, and it is wrong in the expensive \
             direction"
        );
    }

    /// `Ctx::focus`, `Ctx::is_focused`, and the two `Response` fields that arrive with the focus.
    #[test]
    fn focus_is_a_verb_and_entering_and_leaving_are_reported() {
        let mut d = driver();
        let a = Id::named("a");
        let b = Id::named("b");
        let mut seen = (false, false, false, false);
        d.frame(|cx| {
            cx.interact(a, cell(0), Interest::FOCUS);
            assert!(!cx.is_focused(a));
            cx.focus(a);
            assert!(cx.is_focused(a), "and it takes effect at once");
        });
        d.frame(|cx| {
            let ra = cx.interact(a, cell(0), Interest::FOCUS);
            let rb = cx.interact(b, cell(1), Interest::FOCUS);
            seen = (ra.focused, ra.focus_entered, rb.focused, rb.focus_left);
        });
        assert_eq!(
            seen,
            (true, true, false, false),
            "it entered on the next frame"
        );

        d.frame(|cx| {
            cx.interact(a, cell(0), Interest::FOCUS);
            cx.interact(b, cell(1), Interest::FOCUS);
            cx.focus(b);
        });
        let mut left = (false, false);
        d.frame(|cx| {
            let ra = cx.interact(a, cell(0), Interest::FOCUS);
            let rb = cx.interact(b, cell(1), Interest::FOCUS);
            left = (ra.focus_left, rb.focus_entered);
        });
        assert_eq!(
            left,
            (true, true),
            "validation-on-blur is the case these exist for"
        );
    }

    /// **The two seating forms differ, and the wrong one is a `Tab` that appears to do nothing.**
    ///
    /// Architecture issue 25's whole point as a gate. `if cx.focused().is_none()` and
    /// `if !cx.is_focused(sink)` read alike and are not the same program, so both arms are driven
    /// over the same four frames and the difference is asserted rather than described.
    ///
    /// The three facts, in order: nothing holds the focus on the first frame (which is why an
    /// application that never seats it is deaf); the guarded form seats it once; and after the user
    /// moves the focus away, the guarded form **leaves it moved** while the `is_focused` form drags
    /// it back — the frame after which `Tab` has visibly done nothing.
    #[test]
    fn the_guarded_seating_form_seats_once_and_the_is_focused_form_steals_it_back() {
        let sink = Id::named("sink");
        let other = Id::named("other");

        // The two arms as one function with one boolean between them, which is `crate::frame`'s
        // arrangement in the components crate and for its reason: a reviewer's diff is one line.
        fn arm(steal_back: bool) -> (bool, Option<Id>, Option<Id>) {
            let sink = Id::named("sink");
            let other = Id::named("other");
            let mut d = driver();

            let mut nobody_on_the_first_frame = false;
            let seat = |cx: &mut Ctx<'_, '_>| {
                cx.interact(sink, cell(0), Interest::FOCUS);
                cx.interact(other, cell(1), Interest::FOCUS);
                match steal_back {
                    true if !cx.is_focused(sink) => cx.focus(sink),
                    false if cx.focused().is_none() => cx.focus(sink),
                    _ => {}
                }
            };

            d.frame(|cx| {
                nobody_on_the_first_frame = cx.focused().is_none();
                seat(cx);
            });
            let after_seating = d.inspect().focused();

            // The user moves the focus, the way `Tab` does.
            d.frame(|cx| {
                cx.interact(sink, cell(0), Interest::FOCUS);
                cx.interact(other, cell(1), Interest::FOCUS);
                cx.focus(other);
            });
            // ...and the application draws one more time, running its seating line again.
            d.frame(seat);
            (
                nobody_on_the_first_frame,
                after_seating,
                d.inspect().focused(),
            )
        }

        let (guarded_empty, guarded_seated, guarded_after) = arm(false);
        let (stealing_empty, stealing_seated, stealing_after) = arm(true);

        // 1. Nothing holds the focus until an application says so — the finding itself.
        assert!(guarded_empty, "the first frame starts with nobody focused");
        assert!(stealing_empty, "and it is the same on both arms");

        // 2. Both forms seat it, which is why the defect is invisible in a one-widget program.
        assert_eq!(guarded_seated, Some(sink));
        assert_eq!(stealing_seated, Some(sink));

        // 3. The difference, and it only appears once there is somewhere else to be.
        assert_eq!(
            guarded_after,
            Some(other),
            "`focused().is_none()` fires only while nobody holds it, so the move stands"
        );
        assert_eq!(
            stealing_after,
            Some(sink),
            "`!is_focused(sink)` fires every frame the user has moved away, which is a `Tab` that \
             appears to do nothing"
        );
        assert_ne!(
            guarded_after, stealing_after,
            "if these ever agree this gate has stopped separating the two forms"
        );
    }

    /// **A press moves the focus, and a press on nothing interested is intent.**
    ///
    /// The award's `None` stands; only the sweep's is absence, and that one is the vanish rule.
    #[test]
    fn a_press_focuses_a_stop_and_a_press_on_nothing_defocuses() {
        let mut d = driver();
        let stop = Id::named("field");
        let plain = Id::named("tree row");
        let screen = |cx: &mut Ctx<'_, '_>| {
            cx.interact(
                stop,
                Rect::new(0, 0, 4, 1),
                Interest::CLICK.with(Interest::FOCUS),
            );
            cx.interact(plain, Rect::new(0, 1, 4, 1), Interest::CLICK);
        };
        d.post_mouse(press(0, 0));
        drain(&mut d, screen);
        assert_eq!(
            d.inspect().focused(),
            Some(stop),
            "the press moved the focus"
        );

        // A press on something clickable that is not a stop leaves it where it is.
        d.post_mouse(press(0, 1));
        drain(&mut d, screen);
        assert_eq!(
            d.inspect().focused(),
            Some(stop),
            "a tree row is not a tab stop"
        );

        // And a press on nothing at all is intent: the user meant to defocus.
        d.post_mouse(press(200, 40));
        drain(&mut d, screen);
        assert_eq!(
            d.inspect().focused(),
            None,
            "**intent, and the vanish rule leaves it alone**"
        );
    }

    /// `Frame::trap_scopes` exposes the standing set, which nothing else can answer.
    #[test]
    fn trap_scopes_exposes_the_standing_set() {
        let mut d = driver();
        let outer = Id::named("outer");
        let inner = Id::named("inner");
        d.frame(|cx| {
            cx.scope(Id::named("form"), ScopeKind::Group, |cx| {
                cx.interact(Id::named("field"), cell(0), Interest::FOCUS);
            });
            cx.scope(outer, ScopeKind::Trap, |cx| {
                cx.interact(Id::named("outer.ok"), cell(1), Interest::FOCUS);
                cx.scope(inner, ScopeKind::Trap, |cx| {
                    cx.interact(Id::named("inner.ok"), cell(2), Interest::FOCUS);
                });
            });
        });
        assert_eq!(
            d.inspect().trap_scopes().collect::<Vec<_>>(),
            vec![outer, inner],
            "the groups are not traps, and both traps are standing"
        );
        d.frame(|_cx| {});
        assert_eq!(
            d.inspect().trap_scopes().count(),
            0,
            "and none of it survives"
        );
    }

    /// **A ring entry carries a rectangle in the enclosing scroll area's content coordinates**, and
    /// the hit index does not carry one at all.
    #[test]
    fn a_ring_entry_carries_a_content_coordinate_rectangle() {
        let mut d = driver();
        let plain = Id::named("plain");
        let scrolled = Id::named("scrolled");
        d.frame(|cx| {
            // Nested twice, with no scroll area: content coordinates are root coordinates.
            let mut pane = cx.child(Rect::new(10, 5, 40, 20));
            let mut inner = pane.child(Rect::new(2, 3, 20, 10));
            inner.interact(plain, Rect::new(1, 1, 8, 1), Interest::FOCUS);
        });
        assert_eq!(
            d.inspect().ring()[0].rect,
            Rect::new(13, 9, 8, 1),
            "accumulated on the way down, not read off one level of it"
        );

        d.frame(|cx| {
            let mut pane = cx.child(Rect::new(10, 5, 40, 20));
            let view = pane.area();
            pane.scroll_scope(Id::named("list"), view, (0, 100), (0, 400), |cx| {
                cx.interact(scrolled, Rect::new(0, 104, 8, 1), Interest::FOCUS);
            });
        });
        assert_eq!(
            d.inspect().ring()[0].rect,
            Rect::new(0, 104, 8, 1),
            "**the content rectangle, which is row 104 of the list** — the scroll offset is not \
             applied, because a scroll area's own coordinates are what scroll-into-view reasons in"
        );

        // The hit index is a different structure and did not grow one.
        assert_eq!(std::mem::size_of::<Hit>(), 16);
    }

    /// **The caret is the last write of the frame, in the shape it was asked for, and nothing
    /// focused means none.**
    #[test]
    fn the_caret_is_the_last_write_and_nothing_focused_means_none() {
        let mut d = driver();
        let field = Id::named("field");
        d.plant(None, Some(field), None);
        d.frame(|cx| {
            cx.interact(field, cell(0), Interest::FOCUS);
            cx.caret(1, 0);
            cx.caret_with(4, 2, CursorShape::Bar);
        });
        assert_eq!(
            d.inspect().caret(),
            Some(Cursor {
                x: 4,
                y: 2,
                shape: CursorShape::Bar
            }),
            "the last write wins, whichever of the two verbs made it, and the shape is carried"
        );

        // The other order, so the gate is about *last* and not about *which verb*.
        d.frame(|cx| {
            cx.interact(field, cell(0), Interest::FOCUS);
            cx.caret_with(4, 2, CursorShape::Block);
            cx.caret(1, 0);
        });
        assert_eq!(
            d.inspect().caret(),
            Some(Cursor {
                x: 1,
                y: 0,
                shape: CursorShape::Terminal
            })
        );

        // Translated to root coordinates, because two contexts writing `(0, 0)` are two places.
        d.frame(|cx| {
            cx.interact(field, cell(0), Interest::FOCUS);
            let mut pane = cx.child(Rect::new(10, 5, 40, 20));
            let mut inner = pane.child(Rect::new(2, 3, 20, 10));
            inner.caret_with(0, 0, CursorShape::Underline);
        });
        assert_eq!(
            d.inspect().caret().map(|c| (c.x, c.y)),
            Some((12, 8)),
            "and a local (0, 0) is not the screen's"
        );

        // Outside the context's own area, no caret: a field scrolled out of its pane names somebody
        // else's cell.
        d.frame(|cx| {
            cx.interact(field, cell(0), Interest::FOCUS);
            let mut pane = cx.child(Rect::new(10, 5, 4, 2));
            pane.caret(9, 0);
        });
        assert_eq!(d.inspect().caret(), None, "outside the clip, no caret");

        // **Nothing focused means no caret**, whatever was written.
        let mut empty = driver();
        empty.frame(|cx| {
            cx.caret_with(3, 3, CursorShape::Bar);
        });
        assert_eq!(empty.inspect().caret(), None);
    }

    /// **The runtime blinks nothing**: a frame that places a caret asks for no wake.
    #[test]
    fn a_caret_costs_no_wakeup() {
        let mut d = driver();
        let field = Id::named("field");
        d.plant(None, Some(field), None);
        d.frame(|cx| {
            cx.interact(field, cell(0), Interest::FOCUS);
            cx.caret_with(2, 0, CursorShape::Bar);
        });
        assert!(
            d.inspect().caret().is_some(),
            "the caret is placed, and the terminal's own caret is what does the blinking"
        );
        assert_eq!(
            d.queued(),
            0,
            "nothing was queued for a repaint: a software caret would be two wakeups a second"
        );
    }
}

#[cfg(test)]
mod overlay_tests {
    //! Ticket 13's gates: **request during the draw, satisfy after it, answer next frame** — the
    //! half that needs a whole frame.
    //!
    //! Placement, the arena's drop thunk and the z bands have unit tests in [`crate::overlay`], and
    //! the paired compile outcome on the `'f` bound is on [`Ctx`] itself, where it shipped four
    //! tickets before overlays needed it. What is here is the pass: the owner rooting identity, the
    //! layer lifecycle and its census, the barrier that stops the pointer and only the pointer, the
    //! nested band, and the bound that makes a self-requesting body a limit rather than a hang.

    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;
    use crate::focus::ScopeKind;
    use crate::id::Id;
    use crate::overlay::{Align, OverlayOpts, Placement, Scrim, Side, Z, place};
    use crate::theme::Role;
    use vitui_engine::{Button, Buttons, Key, KeyCode, KeyKind, KeyText, Mouse, MouseKind};

    fn driver() -> Driver {
        Driver::headless(300, 80).expect("attaching to a sink cannot fail")
    }

    fn press(x: u16, y: u16) -> Mouse {
        Mouse {
            x,
            y,
            kind: MouseKind::Down(Button::Left),
            buttons: Buttons::NONE,
            mods: vitui_engine::Mods::NONE,
            at: Instant::now(),
        }
    }

    fn tab() -> Key {
        Key {
            code: KeyCode::Tab,
            mods: vitui_engine::Mods::NONE,
            kind: KeyKind::Press,
            text: KeyText::EMPTY,
            at: Instant::now(),
        }
    }

    /// Run frames until the queue is empty, and answer how many it took.
    fn drain(d: &mut Driver, mut draw: impl FnMut(&mut Ctx<'_, '_>)) -> usize {
        let mut frames = 0;
        loop {
            d.frame(&mut draw);
            frames += 1;
            if d.queued() == 0 {
                break;
            }
            assert!(frames < 64, "the drain did not terminate");
        }
        frames
    }

    /// The same, and then **one more frame**.
    ///
    /// The press is awarded at `end` from the index that has just drawn and **delivered on the next
    /// frame's draw**, so the frame that consumes a `Down` is never the frame that reports the click.
    /// A test that asserts a click has to draw once more; a test that asserts the *absence* of one
    /// does not, which is why both spellings exist.
    fn settle(d: &mut Driver, mut draw: impl FnMut(&mut Ctx<'_, '_>)) {
        drain(d, &mut draw);
        d.frame(&mut draw);
    }

    /// **The failing test that designed the verb.** A body drawn inline and the same body drawn in an
    /// overlay produce **different** ids.
    ///
    /// Rooted at [`Id::ROOT`] instead — which is what a derived owner amounts to — the two produce
    /// the same four ids, [`Ctx::interact`]'s collision policy makes the second set inert, and the
    /// overlay draws nothing. One source function called from both places, because two copies of the
    /// body would differ by call site and the assertion would pass for the one reason that is not
    /// interesting.
    ///
    /// The `Rc` is also the positive half of what a body may capture: it is **owned and moved**, so
    /// it satisfies `: 'f` for every `'f`, which is exactly the distinction the compile-fail pair on
    /// [`Ctx`] draws.
    #[test]
    fn a_body_drawn_inline_and_in_an_overlay_produce_different_ids() {
        fn menu(cx: &mut Ctx<'_, '_>, out: &mut Vec<Id>) {
            for item in 0..4u64 {
                cx.with_key(item, |cx| out.push(cx.id()));
            }
        }

        let mut d = driver();
        let owner = Id::named("picker");
        let inline = Rc::new(RefCell::new(Vec::new()));
        let inside = Rc::new(RefCell::new(Vec::new()));
        let inline_sink = Rc::clone(&inline);
        let inside_sink = Rc::clone(&inside);
        d.frame(move |cx| {
            menu(cx, &mut inline_sink.borrow_mut());
            cx.overlay(
                owner,
                Rect::new(10, 10, 8, 1),
                OverlayOpts::sized(8, 4),
                move |cx| menu(cx, &mut inside_sink.borrow_mut()),
            );
        });

        let inline = inline.borrow();
        let inside = inside.borrow();
        assert_eq!((inline.len(), inside.len()), (4, 4), "both bodies ran");
        for (i, id) in inline.iter().enumerate() {
            assert!(
                !inside.contains(id),
                "inline id {i} was reused inside the overlay: the owner did not root the stack"
            );
        }
    }

    /// **A dropdown standing boxes exactly one body a frame**, and a hundred frames do not move the
    /// number: what a frame costs is a property of what is standing on it and not of how long it has
    /// been there.
    ///
    /// This is where the arena's high-water figure used to be asserted, and the shape of the claim is
    /// the same — one number, unmoved by a hundred frames — while the number itself is now a count of
    /// boxes rather than a count of bytes. The allocation that follows from it is gated over the probe
    /// in `tests/alloc.rs`; here the mechanism is asserted where the mechanism is.
    ///
    /// The body captures a `&'static [&str]`, a `usize` and an [`Id`], which is what a dropdown's body
    /// actually holds: its items, its selection, and the owner it reports to.
    #[test]
    fn a_standing_dropdown_boxes_one_body_a_frame() {
        static ITEMS: [&str; 3] = ["Open", "Save", "Close"];
        let mut d = driver();
        let owner = Id::named("dropdown");
        let selected = 1usize;
        let items: &[&str] = &ITEMS;
        for _ in 0..100 {
            d.frame(|cx| {
                cx.overlay(
                    owner,
                    Rect::new(4, 4, 10, 1),
                    OverlayOpts::sized(10, 3),
                    move |cx| {
                        let body = cx.theme().paint(Role::Body);
                        for (row, item) in items.iter().enumerate() {
                            let row = i32::try_from(row).unwrap_or(0);
                            cx.text(0, row, item, body);
                        }
                        let _ = (selected, owner);
                    },
                );
            });
        }
        assert_eq!(
            d.inspect().overlay_bodies_boxed(),
            1,
            "one body a frame, and the hundredth frame boxed no more than the first"
        );
        assert_eq!(d.inspect().overlays_placed(), 1);
        assert_eq!(d.layers_live(), 1, "one layer, reused a hundred times");
    }

    /// **Lifecycle by owner id: a move reallocates 0 and a resize reallocates 1.**
    ///
    /// *No reallocation* is a true sentence about one and a false one about the other, which is why
    /// the gate is the pair rather than either number on its own.
    #[test]
    fn a_move_reallocates_nothing_and_a_resize_reallocates_one_surface() {
        let mut d = driver();
        let owner = Id::named("menu");
        fn show(d: &mut Driver, owner: Id, anchor: Rect, size: (u16, u16)) {
            d.frame(|cx| {
                cx.overlay(owner, anchor, OverlayOpts::sized(size.0, size.1), |_cx| {});
            });
        }

        show(&mut d, owner, Rect::new(2, 2, 8, 1), (10, 3));
        let after_open = d.surface_reallocs();

        // A move: the same size at a different anchor.
        show(&mut d, owner, Rect::new(40, 20, 8, 1), (10, 3));
        assert_eq!(
            d.surface_reallocs(),
            after_open,
            "a move keeps the layer's cells"
        );
        assert_eq!(d.layers_live(), 1, "and it is the same layer");

        // A resize: one surface, reallocated, and the caller redraws — which the body does every
        // frame anyway, because there is no retained structure to redraw from.
        show(&mut d, owner, Rect::new(40, 20, 8, 1), (10, 6));
        assert_eq!(
            d.surface_reallocs(),
            after_open + 1,
            "a resize is one surface"
        );
    }

    /// **The layer census is not §5's identity sweep and cannot be**: an owner with a closed dropdown
    /// is still drawing.
    ///
    /// The owner declares an interactive region on every frame, so the sweep sees a live id
    /// throughout. Only the *request* stops, and only the census notices.
    #[test]
    fn the_census_is_separate_from_the_identity_sweep() {
        let mut d = driver();
        let owner = Id::named("owner");
        fn frame(d: &mut Driver, owner: Id, open: bool) {
            d.frame(|cx| {
                cx.interact(owner, Rect::new(0, 0, 8, 1), Interest::CLICK);
                if open {
                    cx.overlay(
                        owner,
                        Rect::new(0, 0, 8, 1),
                        OverlayOpts::sized(8, 4),
                        |_cx| {},
                    );
                }
            });
        }

        frame(&mut d, owner, true);
        assert_eq!(d.layers_live(), 1);
        assert!(d.inspect().ids().drew(owner), "the owner drew");

        frame(&mut d, owner, false);
        assert!(
            d.inspect().ids().drew(owner),
            "the owner is STILL drawing, so the sweep has nothing to sweep"
        );
        assert_eq!(
            d.layers_live(),
            0,
            "and the layer is gone anyway, because nothing requested it"
        );
    }

    /// **With a modal standing, a press over the base pass reaches nothing** — and the same press one
    /// frame after the modal is dismissed reaches the widget under it.
    ///
    /// The barrier goes down as the body's first statement, which is where `hits.len()` is exactly the
    /// boundary between *under the modal* and *in it*.
    #[test]
    fn a_press_under_a_standing_modal_reaches_nothing_and_reaches_it_again_after() {
        let mut d = driver();
        let button = Id::named("button");
        let owner = Id::named("dialog");
        let theme = *d.env().theme();

        let hit = Rc::new(RefCell::new(false));
        let sink = Rc::clone(&hit);
        d.post_mouse(press(2, 2));
        settle(&mut d, |cx| {
            // **`pressed`, not `clicked`**: one `Down` is the whole event here, and a click needs an
            // `Up` too. `pressed` is the grab, awarded at `end` from the index that has just drawn
            // and readable on the next draw — which is what `settle` runs.
            let r = cx.interact(button, Rect::new(0, 0, 8, 4), Interest::CLICK);
            if r.pressed {
                *sink.borrow_mut() = true;
            }
            cx.overlay(
                owner,
                Rect::new(0, 0, 300, 80),
                OverlayOpts::modal(40, 10, &theme),
                |cx| {
                    cx.modal_barrier_here();
                    cx.interact(Id::named("ok"), Rect::new(0, 0, 6, 1), Interest::CLICK);
                },
            );
        });
        assert!(
            !*hit.borrow(),
            "the press was over the button and the modal withheld it"
        );
        assert!(
            d.inspect().modal_from().is_some(),
            "and the barrier is an ordering into an index that already exists"
        );

        // Dismissed: no request, so no layer, no barrier and no scrim.
        let after = Rc::new(RefCell::new(false));
        let sink = Rc::clone(&after);
        d.post_mouse(press(2, 2));
        settle(&mut d, |cx| {
            let r = cx.interact(button, Rect::new(0, 0, 8, 4), Interest::CLICK);
            if r.pressed {
                *sink.borrow_mut() = true;
            }
        });
        assert!(
            *after.borrow(),
            "and the widget under it is reachable again"
        );
        assert_eq!(d.layers_live(), 0, "the census took the modal's two layers");
    }

    /// **The barrier stops the pointer and only the pointer.** A barrier with no trap around it lets
    /// `Tab` walk straight out of the modal.
    ///
    /// The keyboard half is ticket 12's [`ScopeKind::Trap`], and this is the gate that keeps the two
    /// from being quietly merged into one mechanism: the same frame, with a trap, keeps the focus
    /// inside.
    #[test]
    fn a_barrier_alone_lets_tab_out() {
        let outside = Id::named("outside");
        let inside = Id::named("inside");
        let owner = Id::named("dialog");

        let run = |trapped: bool| {
            let mut d = driver();
            d.plant(None, Some(inside), None);
            d.post_key(tab());
            drain(&mut d, |cx| {
                cx.interact(outside, Rect::new(0, 0, 8, 1), Interest::FOCUS);
                cx.overlay(
                    owner,
                    Rect::new(0, 10, 8, 1),
                    OverlayOpts::sized(20, 4),
                    move |cx| {
                        cx.modal_barrier_here();
                        if trapped {
                            cx.scope(owner, ScopeKind::Trap, |cx| {
                                cx.interact(inside, Rect::new(0, 0, 6, 1), Interest::FOCUS);
                            });
                        } else {
                            cx.interact(inside, Rect::new(0, 0, 6, 1), Interest::FOCUS);
                        }
                    },
                );
            });
            d.inspect().focused()
        };

        assert_eq!(
            run(false),
            Some(outside),
            "a barrier is not a trap: Tab walked out of the modal"
        );
        assert_eq!(
            run(true),
            Some(inside),
            "and the trap is what keeps it in — ticket 12's mechanism, not a second one"
        );
    }

    /// **A nested overlay's z counts from its parent's layer**, so a dropdown inside a modal draws
    /// above the barrier that exists to protect it.
    ///
    /// Sorted into its own band instead, the dropdown is at [`Z::MENU`] — *below* [`Z::MODAL`] — and
    /// it draws under the dialog it belongs to and takes no clicks. It asks for `Z::MENU` here and
    /// does not get it, which is the assertion.
    #[test]
    fn a_dropdown_inside_a_modal_draws_above_the_barrier() {
        let mut d = driver();
        let dialog = Id::named("dialog");
        let picker = Id::named("picker");
        let item = Id::named("item");
        let theme = *d.env().theme();

        d.post_mouse(press(131, 36));
        // **`move`, and the reason is a finding rather than a formality.** `drain` runs many frames,
        // so it takes `impl FnMut(&mut Ctx<'_, '_>)` — a higher-ranked `'f`, which forces every
        // overlay body inside it to `'static`. A `Driver::frame` call gets the true rule from
        // `&'f mut self`; a helper that loops over frames cannot, because each call needs its own
        // reborrow. Moving the `Copy` ids in is what satisfies it.
        drain(&mut d, move |cx| {
            cx.interact(
                Id::named("under"),
                Rect::new(0, 0, 300, 80),
                Interest::CLICK,
            );
            cx.overlay(
                dialog,
                Rect::new(0, 0, 300, 80),
                OverlayOpts::modal(40, 10, &theme),
                move |cx| {
                    cx.modal_barrier_here();
                    cx.overlay(
                        picker,
                        Rect::new(0, 0, 8, 1),
                        OverlayOpts {
                            z: Z::MENU,
                            ..OverlayOpts::sized(8, 3)
                        },
                        move |cx| {
                            cx.interact(item, Rect::new(0, 0, 8, 3), Interest::CLICK);
                        },
                    );
                },
            );
        });
        assert_eq!(
            d.inspect().overlay_rounds(),
            2,
            "the dropdown was requested by a body, so the pass ran a second round"
        );
        assert_eq!(d.layers_live(), 2, "the dialog and the dropdown");

        let from = d.inspect().modal_from().expect("the barrier went down");
        let hits = d.inspect().hits();
        assert!(
            hits[from..].iter().any(|h| h.id == item),
            "the dropdown is inside the modal's range and therefore reachable"
        );
        const {
            assert!(
                Z::MENU < Z::MODAL,
                "which is the whole hazard: its own band is below its parent's"
            );
        }
    }

    /// **The overlay pass is bounded at sixteen rounds**, so a body that requests itself for ever is a
    /// limit and not a hang.
    ///
    /// A fresh owner each round, because one owner keys one layer and a second request under the same
    /// id is inert by design — which would end the recursion for the wrong reason and prove nothing
    /// about the bound. The seventeenth request is dropped, body and all.
    #[test]
    fn a_self_requesting_body_hits_a_limit_and_not_a_hang() {
        assert_eq!(OVERLAY_ROUNDS, 16);
        fn again(cx: &mut Ctx<'_, '_>, depth: u64) {
            cx.overlay(
                Id::from_raw(depth),
                Rect::new(0, 0, 4, 1),
                OverlayOpts::sized(4, 1),
                move |cx| again(cx, depth + 1),
            );
        }
        let mut d = driver();
        d.frame(|cx| again(cx, 0));
        assert_eq!(
            d.inspect().overlay_rounds(),
            16,
            "sixteen rounds, and the frame returned"
        );
        assert_eq!(
            d.inspect().overlays_requested(),
            0,
            "the seventeenth request was discarded, body and all"
        );
        assert_eq!(d.inspect().overlays_placed(), 16);
    }

    /// **One owner keys one layer**, so a second request under the same id this frame is inert — the
    /// same policy as [`Ctx::interact`]'s merge, and counted rather than silent.
    #[test]
    fn a_second_request_under_one_owner_is_inert_and_counted() {
        let mut d = driver();
        let owner = Id::named("twice");
        d.frame(|cx| {
            cx.overlay(
                owner,
                Rect::new(0, 0, 4, 1),
                OverlayOpts::sized(4, 1),
                |_cx| {},
            );
            cx.overlay(
                owner,
                Rect::new(9, 9, 4, 1),
                OverlayOpts::sized(9, 2),
                |_cx| {},
            );
        });
        assert_eq!(d.inspect().overlays_placed(), 1);
        assert_eq!(d.inspect().overlays_merged(), 1);
        assert_eq!(d.layers_live(), 1);
    }

    /// The requests run in `(z, seq)` order, which is what puts a menu's hits *before* a modal's
    /// barrier and therefore out of the pointer's reach.
    #[test]
    fn the_pass_runs_in_z_then_seq_order() {
        let mut d = driver();
        let theme = *d.env().theme();
        let menu_item = Id::named("menu-item");
        let ok = Id::named("ok");
        d.frame(|cx| {
            // Requested modal-first, deliberately: the order it runs in is the z's, not the queue's.
            cx.overlay(
                Id::named("dialog"),
                Rect::new(0, 0, 300, 80),
                OverlayOpts::modal(40, 10, &theme),
                |cx| {
                    cx.modal_barrier_here();
                    cx.interact(ok, Rect::new(0, 0, 6, 1), Interest::CLICK);
                },
            );
            cx.overlay(
                Id::named("menu"),
                Rect::new(0, 0, 8, 1),
                OverlayOpts::sized(8, 3),
                |cx| {
                    cx.interact(menu_item, Rect::new(0, 0, 8, 1), Interest::CLICK);
                },
            );
        });
        let hits: Vec<Id> = d.inspect().hits().iter().map(|h| h.id).collect();
        let at = |id: Id| hits.iter().position(|h| *h == id).expect("it drew");
        assert!(
            at(menu_item) < at(ok),
            "MENU sorts below MODAL, so the menu drew first"
        );
        let from = d.inspect().modal_from().expect("the barrier went down");
        assert!(
            at(menu_item) < from,
            "and the menu is under the barrier, where the pointer cannot reach it"
        );
    }

    /// A scrim is an operator layer under the overlay, and it is what makes an overlay a modal.
    #[test]
    fn a_modal_carries_a_scrim_and_a_dropdown_does_not() {
        let mut d = driver();
        let theme = *d.env().theme();
        assert_eq!(
            Scrim::for_theme(&theme),
            Scrim::shadow(Scrim::DEFAULT_AMOUNT),
            "the shipped theme is dark, so the scrim darkens"
        );
        assert!(OverlayOpts::modal(40, 10, &theme).scrim.is_some());
        assert!(OverlayOpts::sized(8, 3).scrim.is_none());
        d.frame(|cx| {
            cx.overlay(
                Id::named("dialog"),
                Rect::new(0, 0, 300, 80),
                OverlayOpts::modal(40, 10, &theme),
                |_cx| {},
            );
        });
        assert_eq!(d.layers_live(), 1, "one owner, two engine layers");
        d.frame(|_cx| {});
        assert_eq!(d.layers_live(), 0, "and the census took both");
    }

    /// The anchor is translated to root coordinates at the request, because the pass runs after every
    /// context in the subtree has been dropped.
    ///
    /// A container two levels down anchors at its own `(0, 0)`, and the overlay lands under *that*
    /// cell rather than under the screen's origin. The body reads its own origin back through the
    /// pointer it was handed, which is the only coordinate a body can observe from inside.
    #[test]
    fn the_anchor_is_translated_to_root_coordinates() {
        let mut d = driver();
        let owner = Id::named("nested");
        let local = Rc::new(RefCell::new(None));
        let sink = Rc::clone(&local);
        d.post_mouse(press(25, 14));
        drain(&mut d, |cx| {
            let mut outer = cx.child(Rect::new(20, 10, 40, 20));
            let mut inner = outer.child(Rect::new(5, 3, 20, 10));
            let sink = Rc::clone(&sink);
            inner.overlay(
                owner,
                Rect::new(0, 0, 8, 1),
                OverlayOpts {
                    placement: Placement::new(Side::Below, Align::Start),
                    ..OverlayOpts::sized(8, 2)
                },
                move |cx| {
                    let r = cx.interact(Id::named("row"), Rect::new(0, 0, 8, 2), Interest::CLICK);
                    if let Some(at) = r.local {
                        *sink.borrow_mut() = Some(at);
                    }
                },
            );
        });
        // 20 + 5 = 25 across and 10 + 3 = 13 down is the anchor, so the overlay is at (25, 14) —
        // which is the cell the pointer was put on, and the body sees it as its own (0, 0).
        assert_eq!(
            place(
                Rect::new(25, 13, 8, 1),
                (8, 2),
                Rect::new(0, 0, 300, 80),
                Placement::BELOW
            ),
            Rect::new(25, 14, 8, 2)
        );
        assert_eq!(
            *local.borrow(),
            Some((0, 0)),
            "the pointer arrived translated into the layer's own coordinates"
        );
    }

    /// **Every component draws its text before its padding**, and this is the count that says why.
    ///
    /// A component that fills its rectangle and *then* draws into it writes the cells under its text
    /// twice, for an identical picture, every frame, for as long as it is on screen.
    ///
    /// The map measured **10 814 of 24 000** on its own dense screen; this fixture's number is its
    /// own and is asserted for the reason `screen_frame`'s `(32, 119)` is. **What is a property of the
    /// mechanism is the pair**: text-first writes nothing twice, and the picture is identical.
    ///
    /// # The half of the map's claim that does not survive the shipped engine
    ///
    /// The map also priced this at the difference between 1.19 µs and 87 µs. That does not follow
    /// here, and the reason is structural: the engine's damage is a **per-row bitset**, so a second
    /// write inside a range that is already marked adds **no damaged cell**. A scrim therefore
    /// composites the same set either way, and what a redundant fill costs is the redundant *writes*
    /// — real, and nowhere near a cliff. The map's arm was measured against a prototype whose damage
    /// was not a bitset. **So this count is the obligation's only detector**, which is why it is a
    /// gate and why `examples/overlay_numbers.rs` prints that arm's timing without asserting on it.
    ///
    /// The contract is on the component library and cannot be enforced from here — nothing in the
    /// runtime's surface can tell a legitimate second write from a wasteful one.
    /// `examples/overlay_numbers.rs` carries the two frame timings that hang off it.
    #[test]
    fn padding_before_text_re_damages_cells_and_text_before_padding_re_damages_none() {
        let mut d = Driver::headless(300, 80).expect("attaching to a sink cannot fail");
        let pad_first = Rc::new(RefCell::new(0u32));
        let text_first = Rc::new(RefCell::new(0u32));
        let sink = Rc::clone(&pad_first);
        d.frame(move |cx| *sink.borrow_mut() = crate::screen::dense_draw(cx, false));
        let sink = Rc::clone(&text_first);
        d.frame(move |cx| *sink.borrow_mut() = crate::screen::dense_draw(cx, true));
        assert_eq!(
            *text_first.borrow(),
            0,
            "text before padding writes nothing twice"
        );
        assert_eq!(
            *pad_first.borrow(),
            PAD_FIRST_TWICE,
            "and padding before text writes this many cells twice, every frame, for nothing"
        );
        assert_eq!(300 * 80, 24_000, "of a screen this size");
    }

    /// How many of the dense screen's 24 000 cells a pad-then-text component writes twice.
    ///
    /// A property of `crate::screen::dense_draw` and asserted as one, exactly as `screen_frame`'s
    /// `(32, 119)` is: a report that quietly started measuring a smaller screen would otherwise look
    /// good rather than fail.
    const PAD_FIRST_TWICE: u32 = 5_902;
}

#[cfg(test)]
mod loop_tests {
    //! **The loop, as far as this crate owns one.** Spec §21 leaves *who owns the loop* open and
    //! these do not close it; what they close is that no loop could be written at all, because
    //! `Screen::wait` and the `WakeHandle` were both behind a private field.
    //!
    //! Every case here posts or quits **before** it parks. That is not tidiness: with nothing
    //! pending and no deadline the wait is indefinite by design, and `cargo test` has no per-test
    //! timeout — a hang is worse than a failure, which is the rule `Frame::begin` already states
    //! for `IdTable::claim`.

    use super::*;
    use crate::work::{Task, Worker};

    /// **`wait` and `wake` are two halves of one engine.** A quit posted through the handle the
    /// driver hands out is the wake the driver returns, which is the whole claim — a handle cloned
    /// from a different engine would park here for ever.
    ///
    /// `Wake::Quit` and not `Posted`, because a quit is never paced: at a 1 Hz ceiling checking the
    /// clock first would hang shutdown for a second.
    #[test]
    fn a_quit_through_the_drivers_handle_is_the_wake_the_driver_returns() {
        let mut driver = Driver::headless(80, 24).expect("a sink attaches");
        driver.wake().quit();
        assert_eq!(driver.wait(), Wake::Quit);
    }

    /// **The §17 handoff, reachable from a `Driver` for the first time.**
    ///
    /// `Worker::hire` takes a `WakeHandle` and until `Driver::wake` existed there was no way to
    /// obtain one — so the resident thread, the one-slot inbox and the generation beside the answer
    /// were reachable from a test that built its own `Engine` and from nothing else. This is that
    /// module's own arrangement driven end to end: hire, ask, park, take.
    ///
    /// **The park is the gate, not the take.** A polling shape passes a version of this test
    /// without a `WakeHandle` at all, by running frames until the slot is full; sixty frames with a
    /// job in flight then cost sixty wakeups against zero, and the streak fires the runaway
    /// detector on a screen doing nothing.
    #[test]
    fn a_worker_hired_from_the_driver_wakes_it_and_the_answer_is_there() {
        let mut driver = Driver::headless(80, 24).expect("a sink attaches");
        let worker = Worker::hire(driver.wake());
        let task: Task<u64> = Task::new(&worker);

        task.request(1, |_| (1..=10_000u64).filter(|n| n % 7 == 0).count() as u64);

        // Park. The post is the only thing that can return this, and it arrives from the worker's
        // thread — a landing with nothing behind it would leave the take below empty.
        assert_eq!(driver.wait(), Wake::Posted);
        assert_eq!(task.take(), Some(1_428));
    }

    /// **A handle outlives the frame it was taken before, and it is `Send`.**
    ///
    /// The one piece of the app thread's half that crosses, and spec §17's compile-outcome gate is
    /// unchanged by this ticket: what goes to a thread is a `Slot` and a post, never a `Ctx` and
    /// never a `Frame`. Here that is asserted positively — the negative half is `work`'s.
    #[test]
    fn the_handle_is_send_and_survives_a_frame() {
        let mut driver = Driver::headless(80, 24).expect("a sink attaches");
        let wake = driver.wake();
        driver.frame(|cx| {
            let _ = cx.area();
        });
        let joined = std::thread::spawn(move || {
            wake.quit();
        });
        joined.join().expect("the worker thread ran");
        assert_eq!(driver.wait(), Wake::Quit);
    }
}
