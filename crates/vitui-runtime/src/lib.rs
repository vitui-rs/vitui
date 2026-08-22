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

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]

pub mod data;
pub mod layout;

// Re-exported at the root as well as in the module, because the four are named constantly and
// `data::` in front of every one of them is noise at a call site. The module stays public: a reader
// looking for *why there is no trait* should land on its documentation, not on four scattered types.
pub use data::{Edit, Memo, Revision, Versioned};
