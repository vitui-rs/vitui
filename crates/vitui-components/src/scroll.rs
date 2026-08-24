//! **F3 scrolling**, 18 entries, expressed by `scroll_area`, `scrollbar`, `sticky` and
//! `collection`.
//!
//! The reduction is R2 and R5 (spec §18); `pull-to-refresh` is the family's one residue entry, a
//! touch gesture with no terminal meaning and nobody who could change that.
//!
//! `collection` declares this family and is homed under F7. Spec §9's C21 is why the distinction
//! is worth the confusion: **a `scroll_area` costs its content and a virtualised `collection` costs
//! its visible window**, and the wrong pairing is 7 907 us at 100 000 rows — seventy-nine budgets.

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["scroll_area", "scrollbar", "sticky"];
