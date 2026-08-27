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

use vitui_runtime::GlyphSet;

use crate::chart::raster::{Geom, Kind, geom};

/// **How many colours a cell carries, whatever glyph it holds. Two.**
///
/// The whole of why §13's ladder does not apply. A plot puts one *bit* per sub-cell and spells the
/// bitmask with a glyph, so a rung that buys more sub-cells buys more resolution; a picture puts a
/// *colour* in every sub-cell, and a cell is a foreground and a background and nothing else. So the
/// sub-cells a rung offers are a **ceiling this number is below** at every rung above the first.
pub const COLOURS_PER_CELL: u8 = 2;

/// **A picture's sub-rows at one repertoire: 1, 2, 2.**
///
/// # It is derived from the bar ladder rather than branching a second time
///
/// [`crate::gates::REGISTER`]'s row 26 is *`GlyphSet::` in `vitui-components` == 0*, with exactly
/// one named exception — [`crate::chart::raster::geom`], held to three lines. A picture needs the
/// same fact that exception already carries, and taking it from there rather than writing a second
/// `match` is what keeps the exception at one. §21's refinement 3 is *name the exception, do not
/// loosen the gate*; a second file would have been a second exception argued from the first one's
/// argument.
///
/// **The derivation is the argument and not a trick.** What the bar ladder's `sy` reports is
/// availability: `crate::chart::raster`'s own note says the eighth blocks and the half blocks are
/// one contiguous run of block elements and *no font has the half block and not the eighth blocks*,
/// so a rung that offers a bar eight sub-rows offers a picture the half block. [`COLOURS_PER_CELL`]
/// is then the ceiling, and 1, 8, 8 becomes **1, 2, 2**.
///
/// > **Braille's 2x4 buys a picture nothing.** `geom(Kind::Marks, ..)` climbs 1 / 4 / 8 bits; this
/// > climbs 1 / 2 / 2 and stops, and `Extended == Unicode` follows from the arithmetic rather than
/// > from a table.
#[must_use]
pub fn sub_rows(set: GlyphSet) -> u8 {
    geom(Kind::Bars, set).sy.min(COLOURS_PER_CELL)
}

/// **The sub-cell geometry a picture is drawn at**, in the shape [`crate::chart::raster`] states
/// one.
///
/// One sub-column at every rung: the half block splits a cell **vertically**, and there is no
/// spelling that splits it horizontally without also splitting it vertically — the quadrants do
/// both, and a quadrant cell still carries two colours, so the second sub-column would be a fourth
/// pixel with nowhere to put its colour.
#[must_use]
pub fn picture_geom(set: GlyphSet) -> Geom {
    Geom {
        sx: 1,
        sy: sub_rows(set),
    }
}
