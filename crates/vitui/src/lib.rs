//! vitui — a fast, layered TUI library for Rust.
//!
//! This crate is the facade. It re-exports the three layers so an application depends on one name:
//!
//! - [`engine`] — cells, surfaces, layers, compositing, damage, the frame writer
//! - [`runtime`] — layout, identity, focus, hit-testing, routing, key maps, theming
//! - [`components`] — windows, panels, charts, lists, trees, forms, pickers
//!
//! A caller who wants only fast layered terminal output can depend on `vitui-engine` alone and never
//! meet a layout type. That is deliberate; see
//! `docs/adr/0002-layout-lives-outside-the-engine.md`.

// **No `unsafe` in any shipped crate above the engine** (ticket 21, ADR 0034). The facade has none to
// forbid and the attribute does not reach through a `pub use`, so this is the crate saying the same
// thing the other three say rather than a guarantee about them: each of the four says it itself.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use vitui_components as components;
pub use vitui_engine as engine;
pub use vitui_runtime as runtime;
