//! A fine-grained signal graph for `vitui` — **the shape the runtime deliberately does not ship**.
//!
//! Reactivity lives above the runtime (ADR 0020, runtime spec §18), and this crate is what that
//! sentence looks like when it is built rather than asserted. It is about a hundred and twenty lines
//! of code, and the reason it is that short is the whole finding:
//!
//! > **Two of its three parts are already in `vitui-runtime`.** [`Revision`] is the process-global
//! > sequence, [`Versioned`] is the bump-on-drop guard, and [`Memo`] is the cache. What this crate
//! > adds is *shared ownership* and *a dirty flag*, in that order of importance.
//!
//! # Re-running the view is the propagation
//!
//! Not as a convenience of immediate mode, but as a constraint:
//!
//! > **A frame that draws less than the whole screen *declares* less than the whole screen.**
//!
//! A signal graph's whole promise is that a write reaches only the widgets that read it. The
//! expected barrier was the type system, and there is none — a stored `Box<dyn Fn(&mut Ctx)>`
//! compiles, runs and passes, and `tests/declaration.rs` keeps that positive case beside its cost.
//! The real barrier is a count. `Frame::begin` rebuilds the hit index, the focus ring, the overlay
//! queue and the deadline sink **from the draw**, so a frame that redraws one region declares
//! **1 hit entry against 312 and 0 tab stops against 43** while the picture on the screen stays
//! almost entirely correct — and the loss is silent for exactly one frame, because routing answers
//! from the *previous* frame's index.
//!
//! **Fine-grained reactivity here is not expensive; it is a request to revert the hit index.** That
//! is runtime spec §6, §7 and §8, three closed tickets, and no amount of graph design gets round it.
//!
//! # The four hooks
//!
//! The runtime offers reactivity four names and nothing else. They are **four, not three**: shipping
//! `Slot<T>` without the generation beside it ships the defect runtime spec §17 spent a ticket
//! closing — a `Slot` keeps the newest landing and nothing kept the newest *question*.
//!
//! | # | hook | where it is | what this crate does with it |
//! |---|---|---|---|
//! | 1 | `Ctx::request_frame()` | on `Ctx`, so **during** the draw | [`Graph::settle`] |
//! | 2 | `Wake::Posted` + `Slot<T>` + `Task<T>` | `vitui_runtime::work` | nothing — see *`!Send` by construction* |
//! | 3 | deadlines — `Ctx::deadline`, `Ctx::deadline_for` | on `Ctx` | nothing; a graph is not a clock |
//! | 4 | [`Memo`] | `vitui_runtime::data` | [`Computed`] |
//!
//! **`Memo` is the one a graph can actually skip, and the number is 132×** (1.25 ns against
//! 164.55 ns for the aggregate the canonical screen puts in its footer). Everything else a
//! subscription would skip is in the table above this paragraph: the hit index, the ring, the
//! overlay queue and the deadline sink, all of them rebuilt from the draw on purpose.
//!
//! **[`Computed`] turns out to be `Memo` with the key filled in.** The cache is a `Memo<T>`, the key
//! is the inputs' `Revision`, and the invalidation is the same process-global counter. What it adds
//! over writing the memo at the call site is that the graph assembles the key instead of the author.
//! That is real ergonomics and no mechanism at all, and it is said here rather than implied.
//!
//! # `request_frame()` from a TEA `update` is out of reach, and it is the loop's problem
//!
//! Hook 1 is a method on `Ctx`. That is fine for a graph, whose writes happen during the draw. It is
//! not fine for TEA: `update` runs *after* the view by definition, and **every `Ctx` has been
//! dropped by then** — which is the runtime's own test for whether something belongs to it,
//! arriving at the hook instead of at the state.
//!
//! It costs **no runtime change**, and it has two fixes, *both of them the application loop's*:
//! drain the queue inside the frame body where the context is still alive, or let the loop refuse to
//! sleep while its own queue is non-empty. `tests/drivers.rs` builds the first, because it is the
//! one that puts the wakeup where the runtime's wake ledger can see it.
//!
//! **There is no loop to put either fix in.** Who owns the loop — whether the runtime ships an
//! application shell or only the pieces — is runtime spec §21's largest open question, and this
//! crate points at it rather than inventing a loop to close it. Nothing here is blocked by that.
//!
//! # `!Send` by construction, and nothing asked for it
//!
//! [`Signal`] holds an `Rc`, so the whole graph is app-thread-only with no bound written anywhere.
//! A worker cannot be handed the graph, and the only representable shape is the one the engine seam
//! already offers: put a value in a `Slot`, `post`, and let the app thread write the signal when it
//! wakes. That is hook 2, and this crate uses none of it — there is nothing to add.
//!
//! # Copy-and-diff, and the obligation this crate cannot enforce
//!
//! A component writes its state **during the draw** — a wheel notch, an arrow key and a click on a
//! row all land inside the component before the caller sees a `Response`. So a driver that does not
//! hold `&mut` to its state at draw time hands the component a scratch copy and turns the difference
//! into a `set`. TEA and a signal graph converge on the identical mechanism from opposite
//! directions.
//!
//! What makes the diff free is **not the runtime**: it is that component state is small, owned and
//! comparable. That is the *components map's* obligation (its constraint C8), it cannot be enforced
//! from here, and it is load-bearing:
//!
//! > **A component that ships state which is not `Copy + PartialEq` narrows what can be built above
//! > the runtime — without the runtime changing at all.** [`Signal::set`] needs `PartialEq` to be a
//! > change signal, and a component whose state is not comparable puts both reactive shapes off this
//! > runtime.
//!
//! The runtime's own [`Versioned::edit`] cannot stand in for the diff, and this is one of its three
//! stated prices: **it bumps on drop, not on change.** A data contract can pay that; a change signal
//! cannot. [`Signal::edit_undiffed`] is that shape kept deliberately, with the count beside it —
//! **403 writes of which 403 "changed"**, on a screen nobody touched.
//!
//! # This crate is not in the facade, and that is a decision
//!
//! `vitui` re-exports the engine, the runtime and the components — **not** signals. A facade that
//! re-exported one of two reactivity shapes would have picked a winner between two that measured
//! identical, and an application that wants this one adds a line to its own `Cargo.toml`, which is
//! the visibility the choice deserves. The rule is `deny.toml`'s
//! `{ name = "vitui-signals", wrappers = [] }`, and `tests/facade.rs` holds it in both directions.
//!
//! **This crate is a detached workspace, and that is the rule taken literally rather than a
//! convenience.** `cargo deny`'s `[bans] deny` bans a crate's *presence in the graph*; `wrappers` is
//! the exception list, so an empty one means *never present*. A workspace member with nothing
//! depending on it is still present, and the first `cargo deny check` after this crate existed read
//! `error[banned]: crate 'vitui-signals = 0.0.0' is explicitly banned` with no dependent to name. So
//! *the crate may exist and be published, and nothing in this workspace may depend on it* is only
//! satisfiable by a crate outside the workspace — and detaching makes the ban **live**, because it
//! now fires the day a member writes the dependency instead of already failing. Its gates run from
//! their own CI invocation, because `cargo test --workspace` does not reach them.
//!
//! [`Revision`]: vitui_runtime::data::Revision
//! [`Versioned`]: vitui_runtime::data::Versioned
//! [`Versioned::edit`]: vitui_runtime::data::Versioned::edit
//! [`Memo`]: vitui_runtime::data::Memo

// **No `unsafe` in any shipped crate above the engine** (runtime ticket 21, ADR 0034). `forbid` and
// not `deny`, so nothing inside the crate can turn it back on with an `allow`. A signal graph is
// where a `Cell`-based library reaches for `UnsafeCell` first; `RefCell` and `Cell` are what it gets.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// `crate::board` is `#[path]`-included by every gate and by the report as well as compiled here, so
// the one screen has exactly one definition. It is written against the **public** API —
// `use vitui_signals::…` — and this alias is what makes that resolve inside the library too. The
// runtime does the same for its screen and gives the better of the two reasons: **a fixture that
// could reach past the public surface would be measuring something a caller cannot do.**
extern crate self as vitui_signals;

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use vitui_runtime::Ctx;
use vitui_runtime::data::{Memo, Revision, Versioned};

/// A value the graph owns, shared by handle.
///
/// `Rc<RefCell<Versioned<T>>>` plus two `Rc<Cell<_>>`, and the [`Versioned`]
/// inside is not decoration: the revision, the process-global sequence and the bump-on-drop guard
/// are exactly what a signal needs and all three already exist one crate down.
///
/// Cloning a `Signal` clones the handle, never the value. Every clone shares one dirty flag with the
/// [`Graph`] that minted it — which is what makes a write anywhere ask for a frame.
pub struct Signal<T> {
    cell: Rc<RefCell<Versioned<T>>>,
    dirty: Rc<Cell<bool>>,
    writes: Rc<Cell<(u32, u32)>>,
}

impl<T> Clone for Signal<T> {
    fn clone(&self) -> Signal<T> {
        Signal {
            cell: Rc::clone(&self.cell),
            dirty: Rc::clone(&self.dirty),
            writes: Rc::clone(&self.writes),
        }
    }
}

impl<T> Signal<T> {
    /// Read the value under a closure, without cloning it.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        f(self.cell.borrow().get())
    }

    /// The revision the value currently has — the half of a [`Computed`] key this signal supplies.
    pub fn revision(&self) -> Revision {
        self.cell.borrow().revision()
    }

    /// How many writes were **attempted**, and how many of them **changed** the value.
    ///
    /// A count rather than a stopwatch, so that the difference between [`Signal::set`] and
    /// [`Signal::edit_undiffed`] is a gate.
    pub fn writes(&self) -> (u32, u32) {
        self.writes.get()
    }
}

impl<T: Clone> Signal<T> {
    /// The value, cloned.
    pub fn get(&self) -> T {
        self.cell.borrow().get().clone()
    }
}

impl<T: PartialEq> Signal<T> {
    /// **The write, diffed** — and the whole change signal is the `if`.
    ///
    /// `PartialEq` is required because [`Versioned`] bumps on drop
    /// rather than on change, so an undiffed write is not a change signal. See
    /// [`Signal::edit_undiffed`] for what that costs, measured.
    pub fn set(&self, v: T) {
        let (attempted, changed) = self.writes.get();
        if *self.cell.borrow().get() == v {
            self.writes.set((attempted + 1, changed));
            return;
        }
        *self.cell.borrow_mut().edit() = v;
        self.dirty.set(true);
        self.writes.set((attempted + 1, changed + 1));
    }
}

impl<T> Signal<T> {
    /// **The write, undiffed** — the shape a signal library reaches for first, kept so that what it
    /// costs is a number rather than a warning.
    ///
    /// It is the shape that needs no `PartialEq`: hand the component a
    /// [`Versioned::edit`](vitui_runtime::data::Versioned::edit) guard and let it write. Run it and
    /// the screen never sleeps — **403 writes of which 403 "changed"** on a screen nobody touched,
    /// because the guard bumps **on drop, not on change**. `tests/drivers.rs` is where that count
    /// lives.
    pub fn edit_undiffed(&self, f: impl FnOnce(&mut T)) {
        f(&mut self.cell.borrow_mut().edit());
        self.dirty.set(true);
        let (attempted, changed) = self.writes.get();
        self.writes.set((attempted + 1, changed + 1));
    }
}

/// The graph: one dirty flag, and the signals that share it.
///
/// **One flag rather than a set, and the reason is not economy.** There is nothing finer for a set
/// to be a set *of*: a dirty widget cannot be redrawn on its own without the frame declaring less
/// than the whole screen, which is the count in this crate's module documentation.
pub struct Graph {
    dirty: Rc<Cell<bool>>,
}

impl Default for Graph {
    fn default() -> Graph {
        Graph::new()
    }
}

impl Graph {
    /// A graph, dirty — because nothing has been drawn yet.
    pub fn new() -> Graph {
        Graph {
            dirty: Rc::new(Cell::new(true)),
        }
    }

    /// Mint a signal on this graph.
    pub fn signal<T>(&self, value: T) -> Signal<T> {
        Signal {
            cell: Rc::new(RefCell::new(Versioned::new(value))),
            dirty: Rc::clone(&self.dirty),
            writes: Rc::new(Cell::new((0, 0))),
        }
    }

    /// Whether any signal on this graph has changed since the last [`Graph::settle`].
    pub fn is_dirty(&self) -> bool {
        self.dirty.get()
    }

    /// **Hook 1, at the only place it is reachable from: inside the draw.**
    ///
    /// Clears the flag and asks for another frame if anything moved. Returns whether it asked.
    ///
    /// Called from the frame body, because that is where a `Ctx` exists — and because asking through
    /// this door is what puts the wakeup in front of the runtime's wake ledger. A graph that drove
    /// its own loop instead would be a wakeup source nothing counts, which is the defect the runtime
    /// closed one layer down, arriving one layer up in a new dress.
    pub fn settle(&self, cx: &mut Ctx<'_, '_>) -> bool {
        if !self.dirty.get() {
            return false;
        }
        self.dirty.set(false);
        cx.request_frame();
        true
    }
}

/// A derived value, cached until one of its inputs moves.
///
/// **It is [`Memo`] with the key filled in**, and this type says so
/// rather than claiming a mechanism: the cache is a `Memo<T>`, the key is a `Revision`, and the
/// invalidation is the runtime's process-global counter. Fold two inputs' revisions with
/// [`Revision::from_raw`](vitui_runtime::data::Revision::from_raw) and hand the result over —
/// assembling that key is the whole of what a graph does for the author here.
///
/// The `RefCell` is the only difference from writing the memo at the call site: a `Memo` needs
/// `&mut` to fill, and a graph is read through shared handles.
pub struct Computed<T> {
    memo: RefCell<Memo<T>>,
}

impl<T> Default for Computed<T> {
    fn default() -> Computed<T> {
        Computed {
            memo: RefCell::new(Memo::new()),
        }
    }
}

impl<T: Clone> Computed<T> {
    /// The cached value, computing it if `on` is not the revision it was computed at.
    ///
    /// [`Revision::UNKNOWN`](vitui_runtime::data::Revision::UNKNOWN) never hits — *memoise nothing*
    /// is inherited unchanged, and so is its price.
    pub fn get(&self, on: Revision, compute: impl FnOnce() -> T) -> T {
        self.memo.borrow_mut().get(on, compute).clone()
    }

    /// How many times the closure has run. The 132× is this counter staying at 1.
    pub fn recomputes(&self) -> u32 {
        self.memo.borrow().recomputes
    }
}

// **One screen, shared by every gate and both halves of the report.** `#[path]`-included by its
// callers rather than exported, which is `vitui-runtime`'s arrangement for `src/screen.rs` and the
// engine's for its scene list, and for the same reason: an instrument is not part of the library.
// Compiled under `cfg(test)` here so the crate's own suite checks the fixture's counts; the tests and
// the example include the file directly.
//
// `allow(dead_code)` rather than `expect`, and the reason the runtime gives for the same line: each
// *caller* uses a different part of this file — one gate wants the ledger, another only the
// declaration half, the report neither — and a lint that fires on the half you are not looking at
// teaches people to delete the other half.
#[cfg(test)]
#[allow(dead_code)]
#[path = "board.rs"]
mod board;
