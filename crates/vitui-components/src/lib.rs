//! The component library: windows, panels, charts, lists, trees, forms and pickers, built on
//! `vitui-runtime`.
//!
//! A component is a **function of a context and a rectangle**. There is no widget trait, no
//! builder, no retained widget tree and nothing to register: you call a function inside a frame,
//! it draws, and it hands back what it did.
//!
//! # Examples
//!
//! ```
//! use vitui_components::structure::panel;
//! use vitui_components::text::text;
//! use vitui_runtime::Rect;
//! use vitui_runtime::ctx::Driver;
//!
//! // A headless driver is the whole of the setup: it attaches to a sink instead of a terminal, so
//! // this runs anywhere, including in a doctest.
//! let mut driver = Driver::headless(40, 10).expect("a sink attaches");
//!
//! driver.frame(|cx| {
//!     // A panel draws its own border and hands back the rectangle inside it, untouched. What a
//!     // component does not write is named in its return value rather than left to be guessed.
//!     let p = panel(cx, cx.area(), " status ");
//!     assert!(p.interior.w < 40 && p.interior.h < 10);
//!
//!     text(cx, Rect::new(p.interior.x, p.interior.y, p.interior.w, 1), "all quiet");
//! });
//! ```
//!
//! Options are plain `Default` structs, so a call takes what you set and nothing else:
//!
//! ```
//! use vitui_components::structure::{PanelOpts, panel_with};
//! use vitui_runtime::ctx::Driver;
//! use vitui_runtime::theme::Role;
//!
//! let mut driver = Driver::headless(30, 6).expect("a sink attaches");
//! driver.frame(|cx| {
//!     // Thirteen roles, and a component names one of them rather than a colour: the theme
//!     // resolves it for the terminal in hand.
//!     let opts = PanelOpts { border: Role::Focus, ..PanelOpts::default() };
//!     panel_with(cx, cx.area(), " focused ", &opts);
//! });
//! ```
//!
//! # The families
//!
//! One module per family, so a reader who knows what they want finds it without a search:
//!
//! - [`structure`] — panels, rules, splits: the furniture everything else sits in
//! - [`text`] — text, chips, paragraphs, and the wrapping behind them
//! - [`collect`] — lists, menus, tables, tabs, radio groups and multi-select, as **one** component
//!   and one `Mode`
//! - [`input`] — fields, selects, checkboxes, sliders, steppers and the popup that a select opens
//! - [`disclose`] — accordions, disclosure rows, and what a collapsed subtree costs
//! - [`nav`] — cursor movement, type-ahead and the stepping a collection does for you
//! - [`scroll`] — scroll areas, bars and the offsets a caller owns
//! - [`overlay`] — dialogs, popovers and toasts, requested during a draw and satisfied after it
//! - [`indicate`] — progress, spinners, badges and status furniture
//! - [`chart`] — sparklines, bars and the plots that fit in cells
//! - [`canvas`] — a cell grid to draw into directly
//! - [`monitor`] — gauges and meters over a live value
//! - [`files`] — the file picker and its preview pane
//! - [`media`] — the family ships, and no v1 component of it does
//!
//! # Two rules worth knowing before you write one
//!
//! **A component writes every cell of the rectangle it is given**, exactly once, and names in its
//! return value the cells it deliberately left alone. An untouched cell keeps whatever the previous
//! frame put there, so "the caller will fill it" is how a screen ends up with the last screen's
//! data in it.
//!
//! **A component names a `Role`, never a colour.** The theme resolves a role to a paint for the
//! terminal in hand, which is what makes a component work on a 16-colour terminal and a truecolor
//! one without knowing which it is on.
//!
//! # Status
//!
//! Implementation-complete: the v1 freeze is **twenty-nine of twenty-nine components built**, and
//! the authority for that sentence is [`INVENTORY`] — a value the tests iterate — rather than a
//! list written out here. There is no stability promise before 0.x. Minimum supported Rust version
//! 1.88.
//!
//! Beside the components, this crate carries the instruments they are checked with — the freeze
//! itself, the obligation queries, the scene list, the golden screens and the counters behind the
//! numbers. They are public because the checks live in other crates and in the examples, they are
//! of no use to an application, and they are the modules not listed under *The families* above.

// **No `unsafe` in any shipped crate above the engine** (ticket 21, ADR 0034). `forbid` and not
// `deny`, so nothing inside the crate can turn it back on with an `allow`; it subsumes the
// `unsafe_op_in_unsafe_fn` this line used to carry.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod accordion;
pub mod app;
pub mod area;
pub mod clusters;
pub mod composed;
pub mod consumer;
pub mod contract;
pub mod counters;
pub mod dense;
pub mod doc;
pub mod document;
pub mod dropped;
pub mod edit;
pub mod forest;
pub mod form;
pub mod frame;
pub mod gallery;
pub mod gates;
pub mod glyphs;
pub mod golden;
pub mod grid;
pub mod ink;
pub mod inventory;
pub mod keys;
pub mod listing;
pub mod memos;
pub mod obligations;
pub mod order;
pub mod picture;
pub mod popup;
pub mod preview;
pub mod runner;
pub mod scenes;
pub mod series;
pub mod state;
pub mod surround;
pub mod volume;
pub mod wheel;
pub mod window;

// **One module per family, and the family is the module** (spec §19). The tree follows the survey's
// fifteen families so that a reader who knows what they want finds it without a search, and
// `INVENTORY`'s `families` column is the join that makes the mapping checkable rather than a naming
// convention. Four of them are empty today and each says why in its own header — F8 navigation, F11
// media, F13 system and F14 terminal-native ship no v1 component of their own, and F15 has no
// module here at all because its twenty-three entries emit no cells and are the runtime's.
//
// **The directory names are a spelling and not a decision.** §19 is explicit that no ticket
// ratified `architecture.md` §2's list and that renaming one reopens nothing. What is settled is
// the join.
pub mod canvas;
pub mod chart;
pub mod collect;
pub mod disclose;
pub mod files;
pub mod indicate;
pub mod input;
pub mod media;
pub mod monitor;
pub mod nav;
pub mod overlay;
pub mod scroll;
pub mod structure;
pub mod text;

mod family;

pub use family::Family;
pub use vitui_runtime::Rect;
// Re-exported at the root because every list, gate and ticket on this backlog names them, and
// `inventory::` in front of each is noise at the one place they are read. The module stays public:
// a reader looking for *why the axes are columns* should land on its documentation.
pub use inventory::{Axis, COMPOSITIONS, Component, INVENTORY, Layer, MOVED, Moved, Tier};
