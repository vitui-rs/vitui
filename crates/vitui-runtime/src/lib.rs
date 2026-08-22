//! Everything between the engine and a component: layout, identity, focus, hit-testing, routing,
//! key maps, theming, overlays and the data contract.
//!
//! There is **no scene tree** and no retained structure of any kind — the clip stack is the call
//! stack and the id path is the closure tree. What the runtime keeps for one draw and rebuilds on
//! the next is five flat structures; `CONTEXT.md` calls them the frame state. Reactivity is not
//! here either: it is an application's, and it is above this crate rather than inside it.
//!
//! Replaceable in principle — a different runtime should be able to sit on the same engine. That
//! claim is tested by the engine↔runtime seam ticket, not assumed.
//!
//! # Status
//!
//! Being built one ticket at a time from `.scratch/vitui-runtime-architecture/spec.md`, whose map is
//! closed; the backlog is `.scratch/vitui-runtime-impl/`. What exists so far:
//!
//! - [`data`] — `Revision`, `Versioned`, `Edit`, `Memo`. Spec §14, ADR 0019. **`std` only**: it
//!   reaches for neither the engine nor the frame, which is why it is first.
//! - [`layout`] — the constraint solver and the rect algebra. Spec §11. Pure functions over integer
//!   rectangles: no solver state, no allocation, and no floats anywhere.
//! - [`theme`] — `Paint`, `Role`, `Roles`, `Theme`, `Repaint`, `Glyph`, `Distinction`, `Density`.
//!   Spec §3 and §10; ADR 0018, 0021, 0010. A component names a role and can never construct a
//!   paint.
//! - [`keys`] — `Chord`, `Binding`, `KeyMap`, sequences and help. Spec §9. Chords stored inline
//!   because `&'static [Chord]` cannot be written at a call site, and matching on the intent half of
//!   eight modifier bits.
//! - [`ctx`] — `Ctx<'f, 'v>`, `Frame`, `Env`, `Response`, `Interest`, `Driver`. Spec §1, §3, §6;
//!   ADR 0012. Five flat structures rebuilt from the draw, four id-keyed facts, and a `begin` that
//!   cannot be skipped.
//! - [`id`] — `Id`, `IdTable`, the id stack. Spec §5; ADR 0013. The call site is the source, FNV-1a
//!   with no finalizer, and three of the four id-keyed facts swept when a widget stops drawing.

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

// `crate::screen` is `#[path]`-included by `tests/alloc.rs` and by both reports as well as compiled
// here, so the one realistic screen has exactly one definition. It is written against the public API
// — `use vitui_runtime::…` — and this alias is what makes that resolve inside the library too. The
// engine does the same for its scene list and gives the same second reason, which is the better one:
// **a fixture that could reach past the public surface would be measuring something a caller cannot
// do.**
extern crate self as vitui_runtime;

pub mod ctx;
pub mod data;
pub mod id;
pub mod keys;
pub mod layout;
pub mod theme;

// **One realistic screen, shared by the reports and the allocation gates.** `#[path]`-included by its
// callers rather than exported, which is the engine's arrangement for its scene list and for the same
// reason: an instrument is not part of the library. Compiled under `cfg(test)` here so the crate's own
// suite checks its two counts; the examples include the file directly.
// `allow(dead_code)` rather than `expect`, and the same reason the engine's `budget.rs` gives for its
// own included modules: each *caller* uses a different part of this file — the crate's suite checks the
// counts, one example wraps `ROWS`, another only splits — and a lint that fires on the half you are not
// looking at teaches people to delete the other half.
#[cfg(test)]
#[allow(dead_code)]
#[path = "screen.rs"]
mod screen;

// Re-exported at the root as well as in the module, because the four are named constantly and
// `data::` in front of every one of them is noise at a call site. The module stays public: a reader
// looking for *why there is no trait* should land on its documentation, not on four scattered types.
pub use ctx::{Ctx, Driver, Interest, Response};
pub use data::{Edit, Memo, Revision, Versioned};
pub use id::{Id, IdTable};
pub use keys::{ActionId, Binding, Chord, Chords, KeyMap, Match, MatchMode, On};
pub use theme::{Density, Distinction, Glyph, GlyphSet, Paint, Repaint, Role, Roles, Theme};
