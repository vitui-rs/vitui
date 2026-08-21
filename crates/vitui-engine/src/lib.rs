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
//!   |
//!   +- layers()                add layers; view(id) to draw into one
//!   |   +- the verbs           text - fill - restyle, marking damage as they write
//!   |
//!   +- present() -> Presented  composite the damaged rectangles bottom-up
//!                              pack runs and their cells into a packet
//!                              serialise against the mirror, one write into the sink
//!                              clear damage
//! ```
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
//! frame, the mirror's **unknown row** and the `repaint` flag that is the only thing a renumbering
//! breaks, and §14's twelfth scene with the numbers it was put on the list for — 96 entries created
//! over 120 settled frames against 11 520 over 120 fading ones, bounded thereafter by the sweep.
//!
//! Not here yet, each with the ticket that brings it:
//! the `shortest`
//! cursor encoding and the equality filter (13, 14), the scroll region (15), quantisation before
//! the mirror (17), the three threads and the frame clock (18, 19), and input (20, 21).

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
mod damage;
mod detect;
mod engine;
mod exts;
mod geom;
mod intern;
mod layer;
mod mix;
mod packet;
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
