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
//! [`LayerStack::topmost_at`], the query that answers with a layer and never a widget.
//!
//! Not here yet, each with the ticket that brings it:
//! the edge repairs at a layer boundary (11), what a [`Mix`] does to a cell (12), the `shortest`
//! cursor encoding and the equality filter (13, 14), the scroll region (15), capability detection
//! (16), the three threads and the frame clock (18, 19), and input (20, 21).

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

mod cell;
mod damage;
mod engine;
mod exts;
mod geom;
mod intern;
mod layer;
mod packet;
mod serial;
mod tables;

// The scene list, the reference compositor and the register. All three are the instruments spec
// §14 asks for rather than parts of the engine, and none of them is on the public surface: a
// caller cannot read back what is already on screen (ADR 0023), so an oracle over cells lives
// inside the crate. Ticket 25's fuzz targets are what will need `reference` outside `cfg(test)`.
#[cfg(test)]
mod gates;
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
mod text;
mod view;

#[cfg(test)]
mod roundtrip;

pub use engine::{AttachError, Clock, Config, Engine, Output, Presented, Screen, WakeHandle};
pub use exts::LinkId;
pub use geom::Rect;
pub use layer::{LayerId, LayerStack, Mix};
pub use restyle::Restyle;
pub use style::{Color, Style};
pub use surface::Surface;
pub use text::{graphemes, width_of};
pub use view::{Stop, View, Written};
