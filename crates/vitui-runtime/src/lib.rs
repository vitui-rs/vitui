//! Everything between the engine and a component: the scene tree, layout, reactivity, focus,
//! hit-testing and event routing.
//!
//! Replaceable in principle — a different runtime should be able to sit on the same engine. That
//! claim is tested by the engine↔runtime seam ticket, not assumed.
//!
//! # Status
//!
//! Nothing is implemented, and nothing here is decided. Layout, reactivity and focus are all still
//! fog on the wayfinder map at `.scratch/vitui-engine-architecture/map.md`; this crate exists so the
//! seam has somewhere to point.

#![forbid(unsafe_op_in_unsafe_fn)]
#![warn(missing_docs)]
