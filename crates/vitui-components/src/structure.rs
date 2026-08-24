//! **F2 structure**, ~38 entries, expressed by `panel`, `rule`, `block` and the operator layers.
//!
//! The reduction is R2 (spec §18): a container attribute in CSS is a component here and a component
//! there is an argument here, and an inventory-driven count is structurally blind to both
//! directions. A scrim is a complement and never a fill (§2), and there is no cut-out — a terminal
//! cell has no alpha channel, so a scrim cannot have a hole in it.
//!
//! `status_bar` is homed here rather than under F13 because spec §21's ticket 35 settles what it
//! **is**: the same construction as a sticky header or footer, one rectangle split and one hit
//! entry. What it is used for is F13's; what it is is F2's.

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["panel", "rule", "status_bar"];
