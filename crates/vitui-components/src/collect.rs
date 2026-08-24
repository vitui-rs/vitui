//! **F7 collections**, ~50 entries, expressed by `collection` plus columns, an index or tiles.
//!
//! The reduction is R1 and R5 (spec §18). R5 is this family's own and it is the sharpest of the
//! six: twenty-one of the survey's twenty-two data-grid features are caller state or layout, and
//! the twenty-second — variable row height — is a fourth field on §7's record. **The index is the
//! caller's**, which is what makes R5 a reduction rather than a deferral: an entry in that class
//! needs no library mechanism at all, only a documented shape.
//!
//! Org charts, mind maps and pivot tables are layout research, four families away from a mechanism
//! (§22).

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["collection", "table", "tree", "pagination"];
