//! **F4 disclosure**, 21 entries, expressed by `collapsible` alone.
//!
//! The reduction is R1, and **the family exists in order to say so**: spec §18's own sentence is
//! *every one of these is the same state machine*, which is the shape of claim that has to be
//! measured rather than asserted. §8 measured it.

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["collapsible"];
