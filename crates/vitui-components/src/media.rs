//! **F11 media**, 16 entries, expressed by a picture as `Solid`/`Half`, QR, waveform and player
//! chrome — and **no v1 component**.
//!
//! The reduction is R4 with §14's one correction, which is the correction that matters: **a
//! picture's ladder is a colour ladder, not a glyph ladder**, so the image, the gallery, the
//! lightbox and the avatar degrade along a different axis and reach a floor of *nothing* rather
//! than a floor of *worse*.
//!
//! Terminal-native fidelity, video and camera preview are the residue and the engine map owns them:
//! the engine emits no out-of-band bytes, and §14 splits the reason in two — **a preview pane's
//! picture fails for fidelity and a video's for bandwidth**, 24.6 MB/s at truecolor as a lower
//! bound.
//!
//! `file_preview_pane` declares this family — §18's F12 row says picture fidelity inherits F11 —
//! and is homed under F12.

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &[];
