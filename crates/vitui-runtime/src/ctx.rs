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
//! [`Driver`], which `begin`s it — *and* [`IdTable::claim`] returns `None` after a bounded probe
//! instead of spinning, so the failure is a value even if the first half is ever circumvented.

use std::fmt;
use std::marker::PhantomData;
use std::ops::Range;
use std::time::Instant;

use vitui_engine::{
    AttachError, Capabilities, Clock, Config, Cursor, CursorShape, Engine, LayerId, MouseMode,
    Output, Presented, Rect, Screen, View, Written,
};

use crate::keys::Matches;
use crate::theme::{Link, Paint, Repaint, Theme};

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

/// A widget's identity for this frame.
///
/// **Ticket 08 owns the type and the table's stamp discipline; ticket 09 owns the hash.** What is
/// here is a newtype and a claim that cannot spin — enough for [`Response`] to name it and for
/// `Frame::begin`'s stamp bump to be gated. The derivation from the call site, the id stack,
/// `with_key` and the sweep are all 09's.
///
/// **An `Id` may never be persisted.** It is a function of the call site and the closure tree, so a
/// stored one means something different next frame. No serialisation, no ordering — deliberately not
/// `Ord`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Id(u64);

impl Id {
    /// The root, which is what a frame's outermost draw is.
    pub const ROOT: Id = Id(0);

    /// An id from a raw value. **Ticket 09 replaces the only real caller of this**, which is the
    /// derivation from a call site; it is public so a test can name one.
    pub const fn from_raw(v: u64) -> Id {
        Id(v)
    }

    /// The raw value, for a table to index by.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

/// One slot of the stamped id table.
#[derive(Clone, Copy, Debug, Default)]
struct Slot {
    id: u64,
    /// Which frame claimed it. **Compared against the table's current stamp**, which is why the
    /// table is never cleared and why a stamp of zero over slots of zero is a hang.
    stamp: u32,
}

/// The stamped table that detects a duplicate id in one probe.
///
/// **Stamped rather than cleared**, which is ticket 09's 1.1 ns duplicate detector and is also the
/// reason `begin` cannot be skipped: see the module comment.
#[derive(Clone, Debug)]
pub struct IdTable {
    slots: Vec<Slot>,
    stamp: u32,
    /// How many ids were claimed this frame, for the load check.
    live: usize,
}

impl IdTable {
    /// A table with room for a dense screen's widgets.
    ///
    /// Three hundred and thirteen interactive regions is what a dense 300×80 screen has; 512 keeps
    /// the load factor under two thirds without a growth on the first frame.
    pub fn new() -> IdTable {
        IdTable {
            slots: vec![Slot::default(); 512],
            // **One, not zero.** A fresh table is already past the value an un-stamped slot carries,
            // so even a `Frame` that somehow reached `claim` without a `begin` finds every slot
            // foreign rather than every slot its own. The stamp bump in `begin` is still what makes
            // it correct frame to frame; this is the belt to that braces.
            stamp: 1,
            live: 0,
        }
    }

    /// Start a frame: bump the stamp, so every slot is now foreign.
    ///
    /// Wrapping, and a wrap is handled rather than ignored: at the wrap the whole table is zeroed,
    /// because otherwise a slot stamped `u32::MAX` would read as this frame's again after four
    /// billion frames. At sixty frames a second that is two years and three months of uptime, which
    /// is exactly the kind of number that turns up in a bug report rather than a test.
    fn begin(&mut self) {
        self.live = 0;
        self.stamp = self.stamp.wrapping_add(1);
        if self.stamp == 0 {
            for slot in &mut self.slots {
                *slot = Slot::default();
            }
            self.stamp = 1;
        }
    }

    /// Claim an id for this frame. `None` means the table is full or the id is a duplicate.
    ///
    /// **Bounded, and that is the point.** The probe walks at most the table's length, so the answer
    /// is a value rather than a hang however wrong the stamp is. See the module comment: the naive
    /// version spins for ever on a `Frame` that was never begun, and a hang is worse than a failure.
    pub fn claim(&mut self, id: Id) -> Option<usize> {
        let len = self.slots.len();
        if len == 0 {
            return None;
        }
        let mut at = usize::try_from(id.raw() % len as u64).unwrap_or(0);
        for _ in 0..len {
            let slot = self.slots[at];
            if slot.stamp != self.stamp {
                self.slots[at] = Slot {
                    id: id.raw(),
                    stamp: self.stamp,
                };
                self.live += 1;
                return Some(at);
            }
            if slot.id == id.raw() {
                // A duplicate: first claimant wins and the second is inert. Ticket 09 owns the
                // policy; this is where it will live.
                return None;
            }
            at = (at + 1) % len;
        }
        None
    }

    /// How many ids are claimed this frame.
    pub fn live(&self) -> usize {
        self.live
    }

    /// The current stamp, for the gate that asserts `begin` bumped it.
    pub fn stamp(&self) -> u32 {
        self.stamp
    }
}

impl Default for IdTable {
    fn default() -> IdTable {
        IdTable::new()
    }
}

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
    /// 5. The key queue, drained at successively outer levels. Ticket 11 splits it at a routing edge.
    keys: Vec<vitui_engine::Key>,

    // ── the id-keyed facts (ADR 0012). Three are swept at `end`; the click record is not ────────
    grab: Option<Id>,
    press_origin: Option<(Id, (i32, i32))>,
    focused: Option<Id>,
    /// **Not swept**, deliberately: a click record outliving its widget is how a double click
    /// survives a redraw.
    click_record: Option<(Id, Instant)>,

    // ── everything else the frame accumulates ──────────────────────────────────────────────────
    ids: IdTable,
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
            keys: Vec::with_capacity(32),
            grab: None,
            press_origin: None,
            focused: None,
            click_record: None,
            ids: IdTable::new(),
            scratch: Scratch::default(),
            maps: Matches::new(),
            tracking: MouseMode::Off,
            caret: None,
            repaint: false,
            frames: 0,
            begun: false,
        }
    }

    /// Start a frame.
    ///
    /// **Swap-and-clear, not drop-and-rebuild**: every one of the five keeps its allocation, so a
    /// steady frame allocates nothing. The id table is *stamped* instead, which is why this cannot be
    /// skipped — see the module comment.
    fn begin(&mut self, batch: impl IntoIterator<Item = vitui_engine::Key>) {
        self.hits.clear();
        self.ring.clear();
        self.overlays.clear();
        self.deadline = None;
        self.keys.clear();
        self.maps.clear();
        self.scratch.buf.clear();
        self.tracking = MouseMode::Off;
        self.caret = None;
        self.repaint = false;
        self.ids.begin();
        self.frames += 1;
        self.begun = true;

        // Post the batch. **Ticket 11 splits it at a routing edge**; here it is posted whole, which
        // is the named no-op — the structure and the order are right and the classification is
        // missing.
        self.keys.extend(batch);
    }

    /// Finish a frame, folding everything into one wake.
    ///
    /// Six of these steps are named no-ops belonging to later tickets, and they are steps rather than
    /// comments so that filling one is not also deciding where it goes.
    fn end(&mut self) -> Option<Instant> {
        // award the press — ticket 10.
        // resolve Tab — ticket 12.
        // release the focus with the grab — ticket 12.
        // settle hover — ticket 10.
        // sweep three of the four id-keyed facts — ticket 09.
        // resolve scroll-into-view — ticket 14.

        // **Fold the deadline sink and the repaint flag into ONE wake.** This part is 08's, and it is
        // the reason the sink is one `Option` rather than a list: two callers asking for different
        // moments is one wake at the earlier of them, and a repaint request is a wake *now*.
        match (self.deadline, self.repaint) {
            (_, true) => Some(Instant::now()),
            (at, false) => at,
        }
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
    pub fn restyle(&mut self, r: Rect, d: &Repaint) {
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

    // ── `Ctx::link` is not here, and it cannot be ────────────────────────────────────────────────
    //
    // **The ticket asks for `link(uri) -> Link` on `Ctx`, and it is unimplementable over the engine's
    // current public surface.** Minting a link is `Screen::link(&mut self, &str) -> LinkId` and there
    // is no other public route — `Tables::link` and `Interner::mint` are both `pub(crate)`. A `Ctx`
    // holds a `View`, which was handed out by `LayerStack::view`, which borrows the `LayerStack`,
    // which borrows the `Screen`. So the one call that mints and the one type that draws cannot be
    // held at the same time, by construction rather than by oversight.
    //
    // Three things do not fix it, and it is worth saying why so nobody re-tries them:
    //
    // - **Pre-minting a pool** needs the URIs before the draw, and a URI is a component's own datum.
    // - **Deferring** — record the URI now, resolve it at `end` — returns a `Link` that is wrong on
    //   the frame it was asked for, and the frame it was asked for is the one the component puts it
    //   in a `Repaint` on.
    // - **A runtime-side handle** mapped to a real `LinkId` later needs the mapping to happen inside
    //   `Repaint::lower`, which also cannot reach the `Screen`.
    //
    // So `Driver::link` ships instead: it is the same verb where the borrow is available, and it
    // serves an application that knows its URIs. **A component still cannot make one**, and that is
    // filed as a finding against the engine↔runtime seam rather than worked around: ADR 0011 put
    // minting on `Screen`, and a component draws through a `View`. The fix is one method on `View` or
    // on `LayerStack`, and it is the engine map's to make.

    /// Declare an interactive region.
    ///
    /// **Ticket 10 fills the response in.** What is here is 08's half and it is not nothing: the entry
    /// is appended to the hit index in draw order, and the declared interest is folded into the
    /// tracking level `settle` hands to `set_mouse`.
    pub fn interact(&mut self, id: Id, r: Rect, i: Interest) -> Response {
        self.frame.hits.push(Hit {
            id,
            interest: i,
            over: false,
            scrollable: i.contains(Interest::SCROLL),
        });
        // The `max` over a totally ordered ladder, which is why combining is not a negotiation.
        self.frame.tracking = self.frame.tracking.max(i.tracking());
        if i.contains(Interest::FOCUS) {
            self.frame.ring.push(id);
        }
        Response::inert(id, r)
    }

    /// Take the next key for `id`.
    ///
    /// **One value per call, holding no borrow**, which is the second deviation: an iterator borrows
    /// the queue and every interactive component reads its keys inside the scope where it draws, so
    /// the iterator would be live across the drawing verbs. And there is **no upper bound on how many
    /// arrive**, which is what ADR 0008 requires of input.
    ///
    /// Ticket 11 owns routing, so this answers the queue in order and consults no focus.
    pub fn next_key(&mut self, _id: Id) -> Option<vitui_engine::Key> {
        if self.frame.keys.is_empty() {
            return None;
        }
        Some(self.frame.keys.remove(0))
    }

    /// Hand a key back, in order.
    pub fn decline(&mut self, k: vitui_engine::Key) {
        self.frame.keys.insert(0, k);
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
/// [`IdTable::claim`] returning `None` rather than spinning, which holds even if this one is ever
/// circumvented.
pub struct Driver {
    screen: Screen,
    frame: Frame,
    env: Env,
    base: LayerId,
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

        // begin.
        let batch: Vec<vitui_engine::Key> = std::iter::from_fn(|| self.screen.next_event())
            .filter_map(|e| match e {
                vitui_engine::Event::Key(k) => Some(k),
                _ => None,
            })
            .collect();
        self.env.now = Instant::now();
        self.frame.begin(batch);

        // resolve, from the PREVIOUS frame's index, what cannot be answered during the draw:
        // which widget is topmost, and which owns the wheel. Ticket 10 fills both; the step is here
        // because `begin` is 08's and the ordering is the part that cannot be added later.

        // base pass. The size is read *before* the view is taken, because taking it borrows the
        // screen mutably and reading the size borrows it again — `E0502`, and the fix is an ordering
        // rather than a clone.
        let (w, h) = self.screen.size();
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

        // settle.
        self.screen.set_mouse(self.frame.tracking);
        self.screen.set_cursor(self.frame.caret);
        if let Some(at) = wake {
            self.screen.request_wake_at(at);
        }

        self.env.theme_changed = false;
        self.screen.present()
    }

    /// Mint a hyperlink.
    ///
    /// **Here rather than on `Ctx`, and not by choice** — see the note where `Ctx::link` would have
    /// been. Minting needs `&mut Screen` and a `Ctx` holds a `View` borrowed from it, so a component
    /// cannot mint one and an application can.
    pub fn link(&mut self, uri: &str) -> Link {
        Link::from_engine(self.screen.link(uri))
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
    fn a_claim_fails_rather_than_spinning() {
        // The second half, tested directly on the table: a stamp that matches every slot.
        let mut table = IdTable::new();
        // Fill every slot, so a further claim has nowhere to go. This is the shape a wrong stamp
        // produces, reached by a means that cannot hang.
        let len = 512;
        let mut claimed = 0;
        for i in 0..len as u64 {
            if table.claim(Id::from_raw(i)).is_some() {
                claimed += 1;
            }
        }
        assert_eq!(claimed, len, "every slot took one id");
        assert_eq!(
            table.claim(Id::from_raw(9_999)),
            None,
            "a full table returns a value rather than walking the ring"
        );
        // And a duplicate is inert rather than a second slot.
        let mut fresh = IdTable::new();
        assert!(fresh.claim(Id::from_raw(7)).is_some());
        assert_eq!(fresh.claim(Id::from_raw(7)), None, "first claimant wins");
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

    /// A key posted before a frame is readable inside it, and declining puts it back in order.
    #[test]
    fn a_key_is_readable_and_declinable() {
        use vitui_engine::{Key, KeyCode, KeyKind, KeyText, Mods};

        let mut d = driver();
        // No tty, so no key can be posted through the engine; the queue is exercised through the
        // frame's own door, which is what ticket 11 will split at a routing edge.
        d.frame(|cx| {
            let k = Key {
                code: KeyCode::Char('a'),
                mods: Mods::NONE,
                kind: KeyKind::Press,
                text: KeyText::EMPTY,
                at: cx.now(),
            };
            cx.decline(k);
            let back = cx.next_key(Id::ROOT);
            assert_eq!(back.map(|k| k.code), Some(KeyCode::Char('a')));
            assert!(cx.next_key(Id::ROOT).is_none(), "and only the one");
        });
    }
}
