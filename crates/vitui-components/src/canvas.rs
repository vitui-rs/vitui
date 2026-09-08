//! **F14 terminal-native**, ~24 entries, expressed by `canvas` at three rungs and `pty`, and **no
//! v1 component**.
//!
//! The reduction is R4: the braille, block and sextant canvases are the ladder under
//! another name. OSC 52 and the bell are the residue and the engine map owns them — out-of-band
//! sequences the engine does not emit; OSC 8 hyperlinks and the cursor shape are expressible.
//!
//! `pty` is one of the two exemplars that were **not** built. Its process model borders on
//! requirement 9 and may need its own ticket, and recording it as two rather than waving it
//! through is the accounting.

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &[];
