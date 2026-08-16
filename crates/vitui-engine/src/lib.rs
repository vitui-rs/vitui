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
//! Nothing is implemented. The architecture is being decided on the wayfinder map at
//! `.scratch/vitui-engine-architecture/map.md`; the next decision is the cell and buffer
//! representation.

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
