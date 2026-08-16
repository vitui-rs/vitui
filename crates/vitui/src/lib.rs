//! vitui — a fast, layered, reactive TUI library.
//!
//! This crate is the facade. It re-exports the three layers so an application depends on one name:
//!
//! - [`engine`] — cells, surfaces, layers, compositing, damage, the frame writer
//! - [`runtime`] — scene tree, layout, reactivity, focus, event routing
//! - [`components`] — windows, panels, charts, lists, trees, forms, pickers
//!
//! A caller who wants only fast layered terminal output can depend on `vitui-engine` alone and never
//! meet a layout or reactivity type. That is deliberate; see
//! `docs/adr/0002-layout-lives-outside-the-engine.md`.

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

pub use vitui_components as components;
pub use vitui_engine as engine;
pub use vitui_runtime as runtime;
