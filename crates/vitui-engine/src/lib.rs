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
//!   |   +- the verbs           text - fill, marking damage as they write
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
//! everything stands on (ticket 02) and the tracer bullet through every stage of the sequence above
//! (ticket 03), in the deterministic single-thread mode.
//!
//! Not here yet, each with the ticket that brings it: grapheme clusters and the five repair rules
//! (06), extended styles and `restyle` (07), `child` / `scrolled` / the visibility queries (09),
//! operator layers and the `Mix` (11, 12), the `shortest` cursor encoding and the equality filter
//! (13, 14), the scroll region (15), capability detection (16), the three threads and the frame
//! clock (18, 19), and input (20, 21).

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

// Ticket 06 is what exports `graphemes()` and `width_of()` over these tables; until it lands
// nothing outside the module's own gates calls them.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "ticket 06 exports the public surface over this module"
    )
)]
mod ucd;

mod cell;
mod damage;
mod engine;
mod geom;
mod layer;
mod packet;
mod serial;

// The round trip is the primary instrument, and the model it replays through is engine-internal:
// nothing in spec §12's public surface names it. Ticket 25's fuzz targets are what will need it
// outside `cfg(test)`, and that is when it moves.
#[cfg(test)]
mod term_model;
#[cfg(test)]
mod testing;

mod style;
mod surface;
mod view;

#[cfg(test)]
mod roundtrip;

pub use engine::{AttachError, Clock, Config, Engine, Output, Presented, Screen, WakeHandle};
pub use geom::Rect;
pub use layer::{LayerId, LayerStack};
pub use style::{Color, Style};
pub use surface::Surface;
pub use view::{Stop, View, Written};
