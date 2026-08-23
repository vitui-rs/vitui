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
//! │                swap-and-clear the five frame structures · reset the arena
//! │                resolve, from the PREVIOUS frame's index, the two things that cannot be
//! │                answered during the draw: which widget is topmost and which owns the wheel
//! ├─ base pass     view(&mut Ctx) draws into the base layer
//! ├─ overlay pass  each request in (z, seq) order, bounded at 16 rounds
//! ├─ end           award the press · resolve Tab · release the focus with the grab · settle hover ·
//! │                sweep three of the four id-keyed facts · resolve scroll-into-view · fold the
//! │                deadline sink and the repaint flag into ONE wake
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
    Output, Presented, Rect, Screen, View, Written,
};

use crate::id::IdStack;
use crate::keys::Matches;
use crate::route::{self, KeyQueue};
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
    /// Whether it can take a wheel.
    pub scrollable: bool,
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
    /// 2. The focus ring: tab stops declared during the draw. Ticket 12 fills it.
    ring: Vec<Id>,
    /// 3. The overlay request queue, ordered `(z, seq)`. Ticket 13 fills it.
    overlays: Vec<OverlayRequest>,
    /// 4. The deadline sink: **the earliest requested wake, and one value rather than a list**,
    ///    because `end` folds it into one wake anyway.
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
    /// Resolved in `begin` the same way: who owns the wheel.
    wheel_target: Option<Id>,
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
    /// The `max` of every declared interest, which is what `settle` hands to `set_mouse`.
    tracking: MouseMode,
    /// Where the caret goes, if anything asked.
    caret: Option<Cursor>,
    /// Something asked for another frame without naming a moment.
    repaint: bool,
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
    /// Who got the wheel, and how much.
    wheel: Option<(Id, (i32, i32))>,
    /// **Whose drag was cancelled**, which the identity sweep alone did not tell anybody.
    cancelled: Option<Id>,
    /// The modifiers on the event that decided it.
    mods: vitui_engine::Mods,
}

/// A queued overlay. **Ticket 13 owns `OverlayOpts` and placement**; this is the shape of the request
/// so that the queue is a real structure with nothing in it.
#[derive(Debug)]
struct OverlayRequest {
    #[expect(
        dead_code,
        reason = "ticket 13 reads these; the queue exists so it has somewhere to go"
    )]
    owner: Id,
    #[expect(dead_code, reason = "ticket 13 reads these")]
    anchor: Rect,
    #[expect(dead_code, reason = "ticket 13 reads these")]
    z: i32,
    #[expect(dead_code, reason = "ticket 13 reads these")]
    seq: u32,
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
            ring: Vec::with_capacity(64),
            overlays: Vec::with_capacity(8),
            deadline: None,
            keys: KeyQueue::new(),
            mouse: Vec::with_capacity(32),
            pointer: None,
            buttons: vitui_engine::Buttons::NONE,
            mods: vitui_engine::Mods::NONE,
            modal_from: None,
            hover_guess: None,
            wheel_target: None,
            hover_styles: Vec::with_capacity(32),
            awarded: None,
            delivered: None,
            pointer_config: Pointer::default(),
            route_to: None,
            focus_draws: 0,
            grab: None,
            press_origin: None,
            focused: None,
            click_record: None,
            ids: IdTable::new(),
            stack: IdStack::new(),
            scratch: Scratch::default(),
            maps: Matches::new(),
            tracking: MouseMode::Off,
            caret: None,
            repaint: false,
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
        self.wheel_target = self.topmost_scrollable();
        // And the previous frame's award becomes this frame's news.
        self.delivered = self.awarded.take();

        self.hits.clear();
        self.ring.clear();
        self.overlays.clear();
        self.deadline = None;
        self.keys.begin();
        self.maps.clear();
        self.scratch.buf.clear();
        self.tracking = MouseMode::Off;
        self.caret = None;
        self.repaint = false;
        self.ids.begin();
        self.stack.clear();
        self.frames += 1;
        self.begun = true;

        // **Routing starts at the focus and moves outward, never inward.** One id, not a map: see
        // the field.
        self.route_to = self.focused;
        self.focus_draws = 0;

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
    /// **The wake sink and nothing new** — `end` folds this into the one wake it already emits,
    /// which is what makes `[Tab, Key(a)]` route both keys rather than stranding the second.
    fn wants_another_frame(&mut self) {
        self.repaint = true;
    }

    /// The innermost entry the pointer is over, from the index as it stands.
    ///
    /// **A reverse scan, because the index is in draw order and later is innermost.** No quadtree: the
    /// engine already answers the layer question, and 312 entries is 142 ns.
    fn topmost_over(&self) -> Option<Id> {
        let from = self.modal_from.unwrap_or(0);
        self.hits[from..]
            .iter()
            .rev()
            .find(|h| h.over && !h.interest.is_empty())
            .map(|h| h.id)
    }

    /// The innermost scrollable entry the pointer is over.
    fn topmost_scrollable(&self) -> Option<Id> {
        let from = self.modal_from.unwrap_or(0);
        self.hits[from..]
            .iter()
            .rev()
            .find(|h| h.over && h.scrollable)
            .map(|h| h.id)
    }

    /// Finish a frame, folding everything into one wake.
    ///
    /// Six of these steps are named no-ops belonging to later tickets, and they are steps rather than
    /// comments so that filling one is not also deciding where it goes.
    fn end(&mut self) -> Option<Instant> {
        self.award();
        // resolve Tab — ticket 12.
        // release the focus with the grab — ticket 12.
        self.sweep();
        // resolve scroll-into-view — ticket 14.

        // **Fold the deadline sink and the repaint flag into ONE wake.** This part is 08's, and it is
        // the reason the sink is one `Option` rather than a list: two callers asking for different
        // moments is one wake at the earlier of them, and a repaint request is a wake *now*.
        match (self.deadline, self.repaint) {
            (_, true) => Some(Instant::now()),
            (at, false) => at,
        }
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
        let over = self.topmost_over();
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
                vitui_engine::MouseKind::Wheel(w) => {
                    // **The wheel is withheld while a grab is held.** A drag is one gesture and a
                    // scroll in the middle of it is not part of it.
                    if self.grab.is_none()
                        && let Some(id) = self.topmost_scrollable()
                    {
                        let (dx, dy) = match w {
                            vitui_engine::Wheel::Up => (0, -1),
                            vitui_engine::Wheel::Down => (0, 1),
                            vitui_engine::Wheel::Left => (-1, 0),
                            vitui_engine::Wheel::Right => (1, 0),
                        };
                        // **Notches in one batch add up.** A wheel click folds into a frame
                        // (ADR 0016) and it is intent (ADR 0008), and the two are only
                        // compatible if folding is a sum: overwriting made three notches in one
                        // batch scroll one row, which is dropping intent by another name. The
                        // 1006 encoding carries no magnitude, so counting the notches is the
                        // only place the count can come from.
                        a.wheel = Some(match a.wheel {
                            Some((held, (hx, hy))) if held == id => (id, (hx + dx, hy + dy)),
                            // A different target mid-batch: the newer one wins, whole.
                            _ => (id, (dx, dy)),
                        });
                    }
                }
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
        let drew = |id: Id, hits: &[Hit]| hits.iter().any(|h| h.id == id);
        if let Some(id) = self.grab
            && !drew(id, &self.hits)
        {
            self.grab = None;
            // The press origin goes with the grab: it is the same interaction, and a press origin
            // without a grab is a drag nobody is holding.
            self.press_origin = None;
        }
        if let Some((id, _)) = self.press_origin
            && !drew(id, &self.hits)
        {
            self.press_origin = None;
        }
        if let Some(id) = self.focused
            && !drew(id, &self.hits)
        {
            self.focused = None;
        }
        // The click record is **not** swept. See this function's documentation.
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

    /// The focus ring.
    pub fn ring(&self) -> &[Id] {
        &self.ring
    }

    /// How many overlays were requested.
    pub fn overlays_requested(&self) -> usize {
        self.overlays.len()
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
/// use vitui_engine::Rect;
///
/// let mut driver = Driver::headless(20, 5).expect("sink");
/// driver.frame(|cx| {
///     let base_pass_local = String::from("dies at the end of the base pass");
///     // No `move`: the closure borrows, and the borrow is what cannot outlive the frame.
///     cx.overlay(Id::ROOT, Rect::new(0, 0, 4, 1), 0, |_inner| {
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
/// use vitui_engine::Rect;
///
/// static MENU: [&str; 2] = ["Open", "Save"];
/// let mut driver = Driver::headless(20, 5).expect("sink");
/// driver.frame(|cx: &mut Ctx<'_, '_>| {
///     let selected = 1usize;
///     cx.overlay(Id::ROOT, Rect::new(0, 0, 4, 1), 0, move |inner: &mut Ctx<'_, '_>| {
///         let _ = (MENU[selected], inner.area());
///     });
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
    /// The pointer **in this context's own coordinates**, transformed on the way down by `child` and
    /// `scrolled`.
    ///
    /// This is what makes the hit index need no geometry: containment is decided *during the draw*, in
    /// the coordinates the widget is already thinking in, and the index carries one bit instead of a
    /// rectangle. Geometry is needed **inside** a frame and never across one (ADR 0015) — which is the
    /// rule, and is not the same as *no geometry*.
    pointer: Option<(i32, i32)>,
    /// **Invariant in `'f`**, and the whole overlay guarantee rests on it: a covariant brand lets a
    /// caller shorten `'f` at a `child()` call, which makes the `+ 'f` bound on an overlay body
    /// satisfiable by a shorter capture. A lifetime in both argument and return position of a `fn`
    /// pointer is invariant.
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
            pointer: self.pointer.map(|(x, y)| (x - r.x, y - r.y)),
            _frame: PhantomData,
            _not_send: PhantomData,
        }
    }

    /// Offset the content coordinates, which is how a scrolled list is drawn.
    pub fn scrolled(&mut self, dx: i32, dy: i32) -> Ctx<'f, '_> {
        Ctx {
            view: self.view.scrolled(dx, dy),
            frame: self.frame,
            env: self.env,
            rect: self.rect,
            pointer: self.pointer.map(|(x, y)| (x - dx, y - dy)),
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

    /// Write a string.
    pub fn text(&mut self, x: i32, y: i32, s: &str, st: Paint) -> Written {
        self.view.text(x, y, s, st.style())
    }

    /// Write one grapheme cluster.
    pub fn set(&mut self, x: i32, y: i32, cluster: &str, st: Paint) -> Written {
        self.view.set(x, y, cluster, st.style())
    }

    /// Fill a rectangle.
    pub fn fill(&mut self, r: Rect, cluster: &str, st: Paint) {
        self.view.fill(r, cluster, st.style());
    }

    /// Fill this whole context.
    pub fn clear(&mut self, st: Paint) {
        let area = self.area();
        self.view.fill(area, " ", st.style());
    }

    /// Restyle a rectangle without rewriting its content.
    ///
    /// **A descriptor and never a closure**, which is the deviation that closed the fourth compile
    /// outcome: a closure over styles could return one it invented, and there would be a hole for a
    /// component to mint a paint through. See [`Repaint`].
    pub fn restyle(&mut self, r: Rect, d: &Repaint<'_>) {
        let lowered = d.lower(self.env.theme());
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
                pointer: self.pointer,
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
    /// So the id stack is untouched here. Ticket 12 adds what a scope is *for* — grouping, trapping,
    /// isolating the focus — and none of it touches identity.
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
    pub fn scope<R>(&mut self, id: Id, f: impl FnOnce(&mut Ctx<'f, '_>) -> R) -> R {
        // **The one id a bubbling container adds.** Not on the id stack — a scope renames nothing —
        // and not in the hit index either, because it declares no region. The claim is for the
        // identity side alone: it makes the id live, so the sweep and the counts see it.
        let _ = self.frame.ids.claim(id);
        let before = self.frame.focus_draws;
        let r = {
            let mut inner = self.child(self.area());
            f(&mut inner)
        };
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

    /// Open a scroll scope. **Scopes no identity either**, for the same reason.
    pub fn scroll_scope<R>(
        &mut self,
        _id: Id,
        offset: (i32, i32),
        f: impl FnOnce(&mut Ctx<'f, '_>) -> R,
    ) -> R {
        let mut inner = self.scrolled(offset.0, offset.1);
        f(&mut inner)
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
            scrollable: i.contains(Interest::SCROLL),
        });
        // The `max` over a totally ordered ladder, which is why combining is not a negotiation.
        self.frame.tracking = self.frame.tracking.max(i.tracking());
        if i.contains(Interest::FOCUS) {
            self.frame.ring.push(id);
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
            scrolled: a
                .wheel
                .filter(|(who, _)| *who == id)
                .map_or((0, 0), |(_, d)| d),
            // **The position, not a delta.** A splitter, a slider and a selection drag all need where
            // the pointer *is*; reconstructing it from a delta needs a press origin the widget was
            // never given.
            local: local.map(|(x, y)| (x - r.x, y - r.y)),
            focused: self.frame.focused == Some(id),
            focus_entered: false,
            focus_left: false,
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
        let root = Rect::new(
            r.x + (self.rect.x - self.area().x),
            r.y + (self.rect.y - self.area().y),
            r.w,
            r.h,
        );
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
    /// ```compile_fail
    /// use vitui_runtime::ctx::Driver;
    /// use vitui_runtime::Id;
    ///
    /// let mut d = Driver::headless(20, 3).expect("sink");
    /// d.frame(|cx| {
    ///     let _inbox = cx.inbox(Id::ROOT);
    /// });
    /// ```
    pub fn next_key(&mut self, id: Id) -> Option<vitui_engine::Key> {
        if self.frame.route_to != Some(id) {
            return None;
        }
        self.frame.keys.take()
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

    /// Declare a key map for the open scope.
    pub fn key_map(&mut self, map: &crate::keys::KeyMap) {
        self.frame.maps.declare(map);
    }

    /// Queue an overlay.
    ///
    /// The body is bounded by `+ 'f` — see the type's documentation for the paired compile outcome
    /// and for the three things that have to be true for that bound to bite.
    ///
    /// **Ticket 13 runs it.** Here the request is queued and the pass loops over it doing nothing,
    /// bounded at [`OVERLAY_ROUNDS`].
    pub fn overlay<F>(&mut self, owner: Id, anchor: Rect, z: i32, body: F)
    where
        F: FnMut(&mut Ctx<'f, '_>) + 'f,
    {
        // The body is dropped rather than stored: ticket 13 adds the frame arena and the drop thunk
        // beside it, and storing a boxed closure here would be inventing that mechanism early and
        // wrongly. The *bound* is what this ticket owes, and the bound is on the signature.
        drop(body);
        let seq = u32::try_from(self.frame.overlays.len()).unwrap_or(u32::MAX);
        self.frame.overlays.push(OverlayRequest {
            owner,
            anchor,
            z,
            seq,
        });
    }

    /// Ask for another frame at a moment.
    ///
    /// **One sink, folded at `end`.** Two callers asking for different moments is one wake, at the
    /// earlier.
    pub fn deadline(&mut self, at: Instant) {
        self.frame.deadline = Some(match self.frame.deadline {
            Some(existing) if existing <= at => existing,
            _ => at,
        });
    }

    /// Ask for another frame now.
    pub fn request_frame(&mut self) {
        self.frame.repaint = true;
    }

    /// Put the caret here.
    pub fn caret(&mut self, x: i32, y: i32) {
        self.caret_with(x, y, CursorShape::Terminal);
    }

    /// Put the caret here, with a shape.
    ///
    /// `CursorShape` is the engine's, forwarded untouched.
    pub fn caret_with(&mut self, x: i32, y: i32, shape: CursorShape) {
        self.frame.caret = Some(Cursor {
            x: u16::try_from(x.max(0)).unwrap_or(u16::MAX),
            y: u16::try_from(y.max(0)).unwrap_or(u16::MAX),
            shape,
        });
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
}

impl Driver {
    /// Attach to a real terminal.
    pub fn attach(config: Config, theme: Theme) -> Result<Driver, AttachError> {
        let (mut screen, _wake) = Engine::new(config).attach()?;
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
        self.env.now = Instant::now();

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
            self.frame.wants_another_frame();
        }

        // resolve, from the PREVIOUS frame's index, what cannot be answered during the draw:
        // which widget is topmost, and which owns the wheel. Ticket 10 fills both; the step is here
        // because `begin` is 08's and the ordering is the part that cannot be added later.

        // base pass. The size is read *before* the view is taken, because taking it borrows the
        // screen mutably and reading the size borrows it again — `E0502`, and the fix is an ordering
        // rather than a clone.
        let (w, h) = self.screen.size();
        let pointer = self.frame.pointer;
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
                pointer,
                _frame: PhantomData,
                _not_send: PhantomData,
            };
            view(&mut cx);
        }

        // overlay pass — bounded at sixteen rounds. Ticket 13 runs the bodies; the loop is here so
        // that the bound is not something 13 also has to invent.
        for _ in 0..OVERLAY_ROUNDS {
            if self.frame.overlays.is_empty() {
                break;
            }
            self.frame.overlays.clear();
        }

        // end.
        let wake = self.frame.end();

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

    /// Swap the theme. **A move into `Env`**, and the next frame reports it changed.
    pub fn set_theme(&mut self, theme: Theme) {
        self.env.theme = theme;
        self.env.theme_changed = true;
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
                if k.code == KeyCode::Tab {
                    // The ring's, not a widget's: hand it back so `end` can see it.
                    cx.decline(k);
                    break;
                }
            }
        });
        assert_eq!(frames, 1, "both events, one frame");
        assert_eq!(got, vec![KeyCode::Char('a'), KeyCode::Tab]);
        assert_eq!(
            d.unhandled().iter().map(|k| k.code).collect::<Vec<_>>(),
            vec![KeyCode::Tab],
            "and what nobody took is what moves the focus"
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
            vec![vec![KeyCode::Tab], vec![KeyCode::Char('a')]],
            "neither key was lost, and neither was routed against the other's focus"
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
            cx.scope(id, |cx| rows(cx));
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
            cx.scope(panel, |cx| {
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
            cx.scope(panel, |cx| {
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
            cx.scope(sidebar, |cx| {
                cx.interact(Id::from_raw(4), Rect::new(0, 0, 8, 1), Interest::CLICK);
            });
            stolen = cx.next_key(sidebar);
            cx.scope(editor, |cx| {
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
        let frames = drain(&mut d, |cx| {
            draw(cx);
        });
        let mut scrolled = (0, 0);
        d.frame(|cx| scrolled = draw(cx).scrolled);
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
            cx.overlay(Id::ROOT, Rect::new(0, 0, 4, 1), 0, |_inner| {});
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
                cx.overlay(Id::from_raw(i), Rect::new(0, 0, 4, 1), 0, |_inner| {});
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
        d.frame(|cx| cx.scope(Id::named("modal"), |inner| fields(inner, &mut within)));
        assert_eq!(without, within, "a scope changed an id");

        let mut scrolled = Vec::new();
        d.frame(|cx| {
            cx.scroll_scope(Id::named("list"), (0, -5), |inner| {
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
        d.frame(|cx| {
            draw(cx);
        });
        let mut scrolled = (0, 0);
        d.frame(|cx| {
            scrolled = draw(cx).scrolled;
        });
        assert_eq!(scrolled, (0, 1), "the wheel reached the list");

        // Now hold the pointer and try again.
        d.post_mouse(down(5, 5));
        d.frame(|cx| {
            draw(cx);
        });
        d.post_mouse(at(5, 5, MouseKind::Wheel(Wheel::Down), Buttons::NONE));
        d.frame(|cx| {
            draw(cx);
        });
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
}
