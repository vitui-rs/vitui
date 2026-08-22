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
//! Not here yet, each with the ticket that brings it: input (20, 21).

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

mod caps;
mod cell;
mod clock;
mod damage;
mod detect;
mod engine;
mod exts;
mod geom;
mod handoff;
mod intern;
mod layer;
mod mix;
mod packet;
mod quant;
mod quirks;
mod serial;
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
#[cfg(test)]
mod term_model;
#[cfg(test)]
mod testing;

mod restyle;
mod style;
mod surface;
mod sweep;
mod text;
mod view;

#[cfg(test)]
mod roundtrip;

pub use caps::{Capabilities, ColorDepth, GlyphSet, Overrides, Rgb, WidthSource};
pub use clock::Wake;
pub use engine::{AttachError, Clock, Config, Engine, Output, Presented, Screen, WakeHandle};
pub use exts::LinkId;
pub use geom::Rect;
pub use layer::{LayerId, LayerStack};
pub use mix::Mix;
pub use restyle::Restyle;
pub use style::{Color, Style};
pub use surface::Surface;
pub use text::{graphemes, width_of};
pub use view::{Stop, View, Written};
