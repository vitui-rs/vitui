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
//! Nothing is implemented. The architecture is decided one ticket at a time on the wayfinder map at
//! `.scratch/vitui-runtime-architecture/map.md`; this crate exists so the seam has somewhere to
//! point, and its prototypes are the evidence behind those decisions.

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
