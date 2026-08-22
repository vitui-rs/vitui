//! The vitui rendering engine.
//!
//! Turns drawing calls into bytes on the terminal, as fast as possible. It owns cells, surfaces,
//! layers, compositing, damage tracking and the frame writer.
//!
//! It does **not** lay anything out, does not know what a widget is, and never iterates application
//! data. See `docs/adr/0002-layout-lives-outside-the-engine.md`.
//!
//! # Invariant
//!
//! Frame cost is proportional to visible cells, never to data volume.
//!
//! # The frame, as a sequence
//!
//! ```text
//! Engine::new(Config) -> attach() -> (Screen, WakeHandle)
//!   |                                 raw mode, the capability queries, the prologue -- and only
//!   |                                 then the render thread, where no concurrency existed yet
//!   |
//!   +- layers()                add layers; view(id) to draw into one
//!   |   +- the verbs           text - fill - restyle, marking damage as they write
//!   |
//!   +- set_mouse(level)        what the frame's components asked the pointer for, as a `max`
//!   +- set_cursor(caret)       where the caret goes, applied after the frame's last write
//!   |
//!   +- present() -> Presented  the app thread:
//!                                lease a packet, or fold this frame into the next one
//!                                composite the damaged rectangles bottom-up
//!                                pack runs and their cells into the packet
//!                                submit, clear damage, return
//!                              the render thread:
//!                                take the packet, serialise against the mirror,
//!                                one write into the sink, give the packet back
//! ```
//!
//! On `Clock::Manual` both halves run on the calling thread, in that order, before `present`
//! returns — same mailbox, same packet, same bytes. That is spec §14's deterministic mode and it is
//! public API rather than test scaffolding.
//!
//! **The engine does not own the loop.** It hands out the verbs and `present`, and the runtime
//! drives. Damage is marked by the verbs and cleared by `present`, and neither is reachable from
//! outside — which is also why nothing above the engine can force a full repaint.
//!
//! # Status
//!
//! The architecture is decided — `.scratch/vitui-engine-architecture/spec.md` — and the
//! implementation backlog is `.scratch/vitui-engine-impl/`. What exists so far is the Unicode layer
//! everything stands on (ticket 02), the tracer bullet through every stage of the sequence above
//! (ticket 03) in the deterministic single-thread mode, the instruments that keep both honest
//! (ticket 04) — spec §14's twelve scenes as a normative list, a reference compositor that generates
//! the damage gate rather than agreeing with it, and all twenty-seven register entries either wired
//! or pinned red against the ticket that lights them — grapheme clusters in cells (ticket 06): the
//! interner, the five repair rules, and [`graphemes`] and [`width_of`] over the same tables the
//! verbs segment with — the extended-style bit with the verb that owns it (ticket 07):
//! [`Restyle`], [`LinkId`] and the two side tables the packet now carries — and the clip, the
//! viewport and the visibility query (ticket 09): [`View::child`], [`View::scrolled`],
//! [`View::visible_rows`] and [`View::visible_cols`], which are what make a component's rectangle
//! inescapable and a 1M-row tree cost what a 1k-row one costs — and the whole of the layer stack
//! (ticket 10): [`LayerStack::add_content_with`] with the renumbering that lets a surface drawn on
//! a worker be donated, [`LayerStack::remove`], [`LayerStack::set_z`], [`LayerStack::set_rect`] and
//! [`LayerStack::topmost_at`], the query that answers with a layer and never a widget — and the
//! composite of the damaged runs themselves (ticket 11): the opaque `copy_from_slice` against the
//! `EMPTY` skip, the four O(1) repairs per row that close the wide-glyph corruption bug at its
//! third and last edge, the damage a repair leaves outside the layer that caused it, and a resize
//! that repaints rather than patches — and what the terminal on the other end can do (ticket 16):
//! [`Capabilities`] and [`Overrides`], a query batch fired at the live pty behind one DA1 sentinel
//! rather than a terminfo lookup, and spec §10's seven levels of precedence resolved once at
//! [`Engine::attach`] and immutable thereafter — and the residue the round trip cannot reach
//! (ticket 05): the two-plane golden frame with its legend and `VITUI_BLESS=1`, over three of §14's
//! twelve scenes plus the one fixture that draws a wide cluster, whose format lost an argument with a
//! real diff and came back with a row-number gutter and a cell count on every legend entry — and the
//! operator layer (ticket 12): [`Mix`] as the only operator, colour resolved at composite time
//! against what the terminal answered, the memo that makes the correct form 3.2x cheaper than the
//! prototype that deleted a hyperlink, and the atomic-glyph rule that gives a darkened `漢` one
//! colour rather than two — and the bound on the one table that can grow without one (ticket 08):
//! the mark-and-compact sweep, run at [`Screen::layers`] on a high-water mark and never inside a
//! frame, the mirror's **unknown** state and the `repaint` flag that is the only thing a renumbering
//! breaks, and §14's twelfth scene with the numbers it was put on the list for — 96 entries created
//! over 120 settled frames against 11 520 over 120 fading ones, bounded thereafter by the
//! sweep — and **the whole of §8** (tickets 13, 14 and 15): `shortest`, the
//! differential SGR, the two extended channels, synchronised output, **the equality filter with
//! the gap merge that needs no threshold** — which takes six of §14's twelve scenes down by between
//! 1.4x and 37.7x, leaves the six in which every damaged cell genuinely changes exactly where they
//! were, and made one decision on the way: *unknown* is per **cell** rather than per row, because per
//! row left eleven of the twelve scenes unfilterable for ever — and **the scroll region, verified
//! before a byte is emitted**, which takes a steady frame of a scrolling list from 643 bytes to 20,
//! reaches four of the twelve rather than the two §8 named, and verifies exactly one candidate
//! because verifying every one that matched the probe was a 27x regression.
//!
//! Colour is **narrowed to what the terminal can express, inside the run scan and before the mirror
//! comparison** — which is right about frame *size* and was silent about frame *membership*: a
//! mirror holding colours the terminal was never sent under-filters by exactly the amount the depth
//! collapses, so an animated gradient on a 16-colour terminal re-emits every frame for no visible
//! change. The wire gets cheaper as the terminal gets poorer, indices under sixteen are never a
//! quantisation *target* because they are the user's own theme, and contrast preservation is refused
//! because a context-aware choice would break the style run that collected the whole win.
//!
//! And **the three threads** (ticket 18): one mailbox — a `Mutex<Shared>` and two condvars, so one
//! synchronisation primitive exists in the whole design — a pool of exactly two packets, and a
//! render thread that owns the write direction and the mirror and holds no application state and no
//! handle at all. What crosses is a packet keyed by the handle, never by a position: writing an
//! arena offset into a packed cell makes an unchanged cell pack differently whenever the damage
//! changes shape, which §7 measured at **284x in bytes** over five steady frames of a page with
//! nothing changing. There is **no backpressure and the drop path is unreachable**, because
//! dropping an intermediate frame is implemented as never composing it — the app composites only
//! after the renderer has signalled it is free, and damage coalesces in the structure that already
//! does that for 6.6 ns. The consequence is stronger than intended: the slot is always empty at
//! submit, so a packet can never be superseded, which fixes the pool at two provably rather than
//! empirically. `Config::clock` chooses the path and **the deterministic mode still asserts every
//! byte it did**, which is a gate rather than a claim: the same scene through both paths is compared
//! recording against recording.
//!
//! And **the frame clock, on the one gate that does not waste a core** (ticket 19): [`Screen::wait`]
//! is the app thread's only blocking call, and the clock gates *it* rather than `present` (ADR 0004)
//! — gating at `present` runs 800.4 iterations a second to show 114.7 frames, 14% useful, where
//! gating at `wait` runs 114.5 for 114.5 and at a *sparse* event rate delivers **more** frames, not
//! fewer, because it can wake at the gap boundary rather than only when an event happens to arrive.
//! It is a **minimum gap and not a tick**: the first damage after a quiet period returns
//! immediately, everything inside the gap coalesces into one return at the end of it, and with no
//! deadline registered the wait is indefinite — **`30.01 s real, 0.00 user, 0.00 sys, 0 voluntary
//! context switches`** over thirty idle seconds, where a 120 Hz ticker would have woken 3 600 times.
//! [`Wake::Quit`] is checked before the clock, because at a 1 Hz ceiling checking it after hangs
//! shutdown for a second. `request_wake_at` keeps the earliest deadline and deregistration is simply
//! not renewing it; timelines and easing are the runtime's, and **nothing anywhere asks a display
//! what its refresh rate is** — the application says.
//!
//! One thing the settled architecture did not have a name for turned up here and is worth the
//! sentence: §7 has the wake source multiplex *the renderer going free* and §12 has four `Wake`
//! variants, and those are not the same four. It is not cosmetic — a user who stops typing while the
//! renderer is inside a 200 ms write loses that keystroke's echo for ever, because `present` refused
//! and nothing else is going to happen. So the frame is **owed**, `wait` releases when the renderer
//! takes the packet, and it answers [`Wake::Deadline`]: what released the app thread really is the
//! clock, and a fifth variant is a public surface this backlog has not decided. See `crate::clock`.
//!
//! And **input** (ticket 20): the six event variants, the parser behind the seam, and the two
//! synthesised conveniences that everyone reaches for and that are **deliberately absent**. Three
//! facts decide the keyboard and none is ours to fix — without kitty flag 2 a key release never
//! arrives at all, without it auto-repeat is indistinguishable from a fast series of presses, and
//! even at kitty baseline Enter and Tab stay ambiguous with Ctrl+M and Ctrl+I because the spec
//! carves them out so that `reset` stays typeable after a crash. Papering over the first two is
//! refused for **placement rather than principle** (ADR 0007): a synthesised release has no honest
//! timestamp, so a component drawing a held key would show it held until the next keystroke, which
//! on an idle form is forever. [`KeyCode`] is the base layout and [`KeyText`] is what the key
//! printed — an inline grapheme cluster, never a `char`, because kitty flag 16 reports the
//! *codepoints* a key would produce and because a keystroke may not allocate, which the counting
//! allocator asserts over the whole path rather than at one type. **Intent is never dropped;
//! position is** (ADR 0008): 1 000 pointer positions are one event and 1 000 keystrokes are 1 000,
//! gated in one test so the asymmetry cannot be half deleted. [`Paste`] owns **bytes** and not a
//! `String`, because pasted bytes are not guaranteed to be UTF-8 under any answer — the asymmetry
//! with the drawing verbs is deliberate: *at the drawing verbs the engine may demand well-formed
//! input from its caller; at the input boundary it may demand nothing.*
//!
//! The parser is **ours rather than crossterm's**, and that is a finding rather than a preference:
//! `crossterm::event::read` ships welded to a reader that opens `/dev/tty` and registers `SIGWINCH`
//! on one poll, and this crate already owns the only reader the terminal has — detection needs a
//! deadline on the read, and a second reader steals bytes from the first. So ADR 0001's title is
//! amended and what crossterm is left doing is raw mode, the tty test and the size. One consequence
//! is owed and stated rather than hidden: **a resize is observed when the next byte arrives**, and a
//! terminal resized while the application is completely idle is not noticed until the user touches
//! something. Closing it needs either a signal handler, which the dependency policy does not have,
//! or DEC mode 2048 requested at startup. **Ticket 21's negotiation did not take it**, and the
//! reason is that requesting the mode is the cheap half: the report arrives as a `CSI 48 ; … t`
//! nothing parses, so it would land in [`InputDiagnostics`] as an unrecognised sequence per resize,
//! and §10's batch has no DECRQM question that would say whether the request took. That is a
//! detection axis, a parser arm and a `Capabilities` field, and none of the three belonged to a
//! ticket about two setters.
//!
//! And **the two setters, the negotiation and the caret** (ticket 21). Everything the engine says to
//! the terminal that is not a cell is [`crate::actuate`], and all of it is bytes with no reply — the
//! questions were asked and answered at `attach`, before its first byte goes out.
//!
//! [`MouseMode`] is `Off < Buttons < Drag < Motion` and the ordering is a design decision rather than
//! an accident of numbering: each level strictly contains the one below, so **combining what several
//! components want is a `max` and not a set union**, and [`Screen::set_mouse`] receives one level per
//! frame with no idea how many components it came from. It is **idempotent and free when unchanged**,
//! and free in the strong sense the obligation needs rather than the one it states: the delta is
//! computed on the app thread before a packet is leased, so a thousand unchanged calls — which is
//! what a runtime makes, because it calls this after every frame — cost a thousand comparisons and no
//! composite, no pack, no serialise and no write. [`Config::input`] is the **floor** under it, since
//! the union is known only after a draw and the frame that first paints a hover-wanting modal did not
//! yet have tracking on. SGR encoding is on whenever the mouse is on, because without it a press
//! stops being reportable past column 223 and the budget is written against 300 columns.
//!
//! [`Screen::set_cursor`] is the caret, and **there is no software caret anywhere** (ADR 0005): one
//! `restyle` of one cell, toggled, is two wakeups a second for as long as anything has focus — 7 200
//! an hour on a screen where nothing is happening — so ticket 19's measured idle would not survive a
//! text field, and a form is not an exotic component. The terminal's own caret blinks in the
//! terminal's process at the user's rate, and is the only one a screen reader or an IME can follow.
//! It is applied **after the frame's last write**, which is the only moment at which it is correct
//! and a moment only the engine has, and it is placed with the serializer's own `shortest` rather
//! than an unconditional `CUP` — so a character typed into a field leaves the cursor exactly where
//! the caret belongs and **§8's 29-byte caret frame is 29 bytes with a caret on it**. Position, shape
//! and visibility are tracked apart for the same reason: a frame that re-stated either would be 40.
//!
//! One thing this ticket had to reach for and one thing it refused. The reach: arch 22 refused the
//! eight input facts an `Overrides` field, on the grounds that *a declaration cannot make an event
//! arrive* and that nothing on the output path reads one — and the second half stopped being true
//! here, because the actuator and the negotiation read four of them. The field is still refused; the
//! arms come from **inside** the crate, which is the door arch 22 named for `sync_output`. The
//! refusal: `DECSCUSR` conflates shape with blink, so [`CursorShape`] offers the blinking spellings
//! and `Terminal`, which is the user's own configuration and the default.
//!
//! And **the terminal comes back** (ticket 22). The alt screen is entered as the session's first
//! byte and left as its last, and everything between is given back in the order it was taken — but
//! the sequence is the easy half. *Only the render thread writes* narrows to **only the render
//! thread writes frames**: a panic hook runs on whichever thread panicked, and joining the render
//! thread from inside one deadlocks when the render thread is the one that died. So restoration is
//! an **idempotent function guarded by one atomic, callable from any thread** — eight threads race
//! it and exactly one performs it — reached from the hook and from `Screen`'s own `Drop`, which is
//! why a normal return and a `?` out of `main` need no ceremony and why a panic mid-frame needs no
//! `catch_unwind`. It runs **before** the default hook prints, because a backtrace painted into the
//! alt screen is discarded with the page a moment later, and the gate is a child process whose two
//! output streams share one open file so that *before* is a fact about the bytes rather than a
//! claim. **Restoration is the input state and not only the screen**: the kitty flags are popped,
//! mouse tracking, focus reporting and bracketed paste go off, and auto-wrap comes back before the
//! page does — a crashed process that leaves the flags pushed breaks the shell that outlives it,
//! and a shell whose line editor cannot wrap overwrites its own prompt. **All three tracking modes
//! go off, not the one this side believes in**, because the level the app thread holds is the one the
//! next packet will ask for and the mode the terminal is actually in is one of two until that packet
//! is written; a session that never asked for a mouse still says nothing about one. And the
//! restoration **stops the render thread** before its first byte, because a renderer that takes one
//! more packet after the alt screen is gone paints cells onto the user's shell — which on a worker
//! thread's panic, where the process does not end, it would go on doing. On the way out the render
//! thread **is**
//! joined — it is parked on a condvar or inside a bounded `write`, both finite — the input thread
//! never is, and *the last frame is not flushed*: a packet still in the slot when `quit` arrives is
//! left there, which is one write fewer on the wire and no exit latency spent showing state that is
//! already stale.
//!
//! And **the app thread that nothing but itself can stop** (ticket 23). The inversion is the whole
//! of it — *the compiler cannot stop the app thread from being slow; it can stop anything else from
//! being the app thread* — and the offence is a **frame-budget overrun by the app thread's iteration,
//! whatever caused it**, never a blocking syscall: `NetworkOnMainThreadException` picks the syscall
//! and therefore catches a DNS lookup while waving through a `for` loop that takes 400 ms. Four of
//! the five rungs cost nothing — [`Screen`] and [`View`] are `!Send`, there is no blocking primitive
//! on the app-thread side at all, and the lint fragment is the application author's
//! (`examples/app-template/`, which ships it with the sentence that it protects vitui and **not**
//! vitui's users, because `clippy.toml` is read from the crate being linted and no stable mechanism
//! lets a dependency inject lints downstream). The two that cost something are `crate::perf`: an
//! in-loop detector at **45 ns a frame that stays in release**, and a debug-only observer thread.
//!
//! The threshold is **one frame interval, and 100 µs would have been a bug** — the map's < 100 µs is
//! a CI gate on one stage of the engine's own work, and a full realistic `wake → submit` is
//! 166.76 µs, so that watchdog fires on entirely legitimate frames. Debug panics on the first
//! overrun, which is safe by construction because ticket 22's restoration is idempotent and runs
//! before the default hook; release warns **once**, into a sink the caller supplies, and `None` is
//! silence rather than stderr because a full-screen application's stderr is the terminal it is
//! drawing on.
//!
//! Two things this ticket found rather than built. **Spec §11's fix for the escape hatch is half of
//! one**: `permit_slow(&mut self)` was `E0499`, and *`Cell` and `&self` throughout* leaves a
//! `Permit<'a>` borrowed out of `&'a Screen`, which makes drawing inside the permitted region
//! `E0502` — the identical defect one letter along. So [`Permit`] holds an `Rc` and has no lifetime,
//! and the marker the ticket predicted for its negative case (`Cell<()>`) is `Rc<Perf>`. And **a
//! permit still held at `present` has to be accounted for**: the first shape did all the arithmetic
//! in `Drop`, so a guard kept until after the frame excused nothing at all — and said so in a
//! diagnostic that printed the permit's own reason.
//!
//! The two sanctions are deliberately spelled differently, and it is not that debug is stricter.
//! **The observer may not panic**: a panic on its thread unwinds its own stack and stops nothing, and
//! a *returning* app thread would then paint frames into a terminal somebody had restored. So it
//! restores, prints when the iteration entered and what it said it was doing, and aborts — in that
//! order, gated by a child process whose two streams share one open file. A permit **annotates** the
//! stall and does not excuse it, because *this will be slow* and *this has not come back at all* are
//! different claims. And the observer is `cfg(debug_assertions)` only, because **CPU is not the
//! objection and wakeups are**: a 100 ms poll converts zero wakeups into about ten a second for ever,
//! which is the one property standing requirement 11 names literally. Register entry #18 is that
//! absence, checked in the one place absent code is visible — the observer's own words are in a debug
//! binary and not in a release one.
//!
//! What is offered instead of blocking is [`WakeHandle`] and [`Slot`], whose only accessor is a
//! non-blocking `take`. **There is no `recv`, no `wait`, no `Future` and no completion returned by
//! anything on the app-thread side**, and the two `recv`s that do exist in this crate are the render
//! thread's and the input thread's — and detection's deadline read is on the app thread but outside the loop.
//! *No blocking receive is inside the app thread's iteration* is the honest form of §12's refusal 7,
//! and `crate::gates` keeps it true over the source rather than in prose. A worker pool
//! is refused: it is an executor under another name, and the numbers that stop anybody reaching for
//! one are beside [`Slot`] itself.
//!
//! Not here yet: the public surface and its negative corpus (24), fuzzing (25), and the budget ledger
//! and comparative suite (26).

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

// `crate::scenes` and `crate::register` are `#[path]`-included by `examples/budget.rs` as well as
// compiled here, so that the twelve scenes and the twenty-seven register entries have exactly one
// definition. They are written against the public API — `use vitui_engine::…` — and this alias is
// what makes that resolve inside the library too. Spec §14's rule for the comparative suite is that
// a scene is defined by what the user sees rather than by what a framework does, and a scene that
// could reach past the public surface would be defined by what this framework does.
extern crate self as vitui_engine;

mod ucd;

mod actuate;
mod caps;
mod cell;
mod clock;
mod damage;
mod detect;
mod engine;
mod exts;
mod geom;
mod handoff;
mod input;
mod intern;
mod layer;
mod mix;
mod packet;
mod perf;
mod quant;
mod quirks;
mod reader;
mod serial;
mod shutdown;
mod tables;

// The scene list, the reference compositor and the register. All three are the instruments spec
// §14 asks for rather than parts of the engine, and none of them is on the public surface: a
// caller cannot read back what is already on screen (ADR 0023), so an oracle over cells lives
// inside the crate. Ticket 25's fuzz targets are what will need `reference` outside `cfg(test)`.
#[cfg(test)]
mod gates;
#[cfg(test)]
mod golden;
#[cfg(test)]
mod reference;
#[cfg(test)]
mod register;
#[cfg(test)]
mod scenes;

// The round trip is the primary instrument, and the model it replays through is engine-internal:
// nothing in spec §12's public surface names it. Ticket 25's fuzz targets are what will need it
// outside `cfg(test)`, and that is when it moves.
// `crate::input`'s unit tests, declared here rather than inside `input.rs` — which is where they
// would go, and where they were. `tests/alloc.rs` `#[path]`-includes `input.rs` so that the
// keystroke path can be measured under the counting allocator, and a `mod tests` inside it would put
// all of them in that binary, running beside an allocation window. See
// `alloc::a_keystroke_allocates_nothing`, which says the same thing from the other end.
#[cfg(test)]
#[path = "input/tests.rs"]
mod input_tests;

#[cfg(test)]
mod term_model;
#[cfg(test)]
mod testing;

mod restyle;
mod slot;
mod style;
mod surface;
mod sweep;
mod text;
mod view;

#[cfg(test)]
mod roundtrip;

pub use actuate::{Cursor, CursorShape};
pub use caps::{Capabilities, ColorDepth, GlyphSet, Overrides, Rgb, WidthSource};
pub use clock::Wake;
pub use engine::{AttachError, Clock, Config, Engine, Output, Presented, Screen, WakeHandle};
pub use exts::LinkId;
pub use geom::Rect;
pub use input::{
    Button, Buttons, Event, InputConfig, InputDiagnostics, Key, KeyCode, KeyKind, KeyText, Keypad,
    Media, Modifier, Mods, Mouse, MouseKind, MouseMode, Paste, Wheel,
};
pub use layer::{LayerId, LayerStack};
pub use mix::Mix;
pub use perf::Permit;
pub use restyle::Restyle;
pub use slot::Slot;
pub use style::{Color, Style};
pub use surface::Surface;
pub use text::{graphemes, width_of};
pub use view::{Stop, View, Written};
