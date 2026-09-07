//! vitui — a fast, layered TUI library for Rust.
//!
//! This crate is the facade: it re-exports the three layers, so an application depends on one name
//! and one version.
//!
//! - [`engine`] — cells, surfaces, layers, compositing, damage tracking, the frame writer
//! - [`runtime`] — layout, identity, focus, hit-testing, key routing, theming, overlays
//! - [`components`] — windows, panels, charts, lists, trees, forms, pickers
//!
//! # Examples
//!
//! A frame, start to finish. The clock is manual, so `present` does the render pass inline on the
//! calling thread and the program is a straight line — which is what makes this runnable here
//! rather than a sketch:
//!
//! ```
//! use vitui::engine::{Clock, Color, Config, Engine, Output, Rect, Style};
//!
//! let engine = Engine::new(Config {
//!     clock: Clock::Manual,
//!     output: Output::Sink(Box::new(Vec::new())),
//!     size: (40, 6),
//!     ..Config::default()
//! });
//! let (mut screen, _wake) = engine.attach()?;
//!
//! let panel = screen.layers().add_content(0, Rect::new(0, 0, 40, 6), true);
//! let mut view = screen.layers().view(panel).expect("the layer was just added");
//! view.fill(Rect::new(0, 0, 40, 6), " ", Style::new().bg(Color::rgb(16, 16, 24)));
//! view.text(2, 2, "hello", Style::new().fg(Color::rgb(220, 220, 220)).bold());
//! drop(view);
//!
//! // The three verbs above marked their own damage as they wrote, so this frame has something to
//! // send. A second `present` with nothing drawn between the two reports `submitted == false`.
//! assert!(screen.present().submitted);
//! # Ok::<(), vitui::engine::AttachError>(())
//! ```
//!
//! # Which layer you want
//!
//! An application that wants fast layered terminal output and brings its own layout can depend on
//! `vitui-engine` alone and never meet a layout type: the engine has nothing to call upward, so
//! taking only it is a supported choice rather than a workaround. The engine lays nothing out and
//! never iterates your data — it offers clipping and offset viewports, and culling is yours — which
//! is what keeps a frame's cost proportional to the cells on screen rather than to the rows behind
//! them.
//!
//! Add `vitui-runtime` for layout, focus, hit-testing, key maps and themes, and
//! `vitui-components` for the built widgets. Depending on this crate is the same set of code with
//! one dependency line instead of three.

// The facade has no code of its own to forbid `unsafe` in, and the attribute does not reach through
// a `pub use`, so this is the crate saying for itself what the other three say for themselves.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use vitui_components as components;
pub use vitui_engine as engine;
pub use vitui_runtime as runtime;
