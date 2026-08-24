//! **F12 files**, ~35 entries, expressed by `collection` + `scroll_area` + a worker + the preview
//! pane.
//!
//! The reduction is R3 (spec §18), and R3 is the class that requires the most care: *composition
//! with no new mechanism* is exactly the claim that turns out to be false when it is false, which
//! is why C16 was a ticket rather than an assertion. It resolved having built both rows —
//! `file_preview_pane` with 23 gates — and every defect it found was **at a seam between two of
//! the five pieces**, not inside one.

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["file_picker", "file_preview_pane"];
