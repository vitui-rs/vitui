//! **F1 text**, ~48 entries, expressed by `text`, `chip`, `field` and the `fit` helper.
//!
//! The reduction is R1 and R5 (spec §18): markdown needs its own wrap pass and is §22's, and there
//! is no bidi — a stated non-goal with the engine's tables named as the reason.
//!
//! `field` declares this family and is **not** homed here: its first family is F6, which is where
//! the caret, the wrap index and the keyboard live. Spelled out because `text` and `field` sharing
//! `layout::text` is the reason a reader would expect otherwise.

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["text", "chip"];
