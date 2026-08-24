//! The component library: windows, panels, charts, lists, trees, forms, pickers.
//!
//! # Status
//!
//! Out of scope for the current effort, which is charting the engine's architecture. Three hostile
//! components — a virtualised 1M-row tree, a form with text input, and a sub-cell-resolution chart —
//! appear in that effort only as design pressure on the engine seam. The component library itself
//! gets its own map afterwards.

// **No `unsafe` in any shipped crate above the engine** (ticket 21, ADR 0034). `forbid` and not
// `deny`, so nothing inside the crate can turn it back on with an `allow`; it subsumes the
// `unsafe_op_in_unsafe_fn` this line used to carry.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
