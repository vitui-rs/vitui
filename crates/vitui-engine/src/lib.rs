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
//! # Status
//!
//! Nothing is public yet. The architecture is decided — `.scratch/vitui-engine-architecture/spec.md`
//! — and the implementation backlog is `.scratch/vitui-engine-impl/`. What exists so far is the
//! Unicode layer everything above it stands on.

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
