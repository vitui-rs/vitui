//! **F9 overlays**, ~35 entries, expressed by `overlay`, layers, placement, scopes and `Trap`.
//!
//! The reduction is R1 and R2 (spec §18): modality is a `bool`, and the eighteen named popup,
//! dialog, drawer, sheet and toast entries are two axes of one component.
//!
//! `select` declares this family and is homed under F6, because §12's finding is that the popup is
//! the *owner's* — `SelectState` is written only by the owner and `PopupState` only by the body,
//! and `&'f mut` is what makes *request the overlay last* a borrow error rather than a comment.

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["overlay"];
