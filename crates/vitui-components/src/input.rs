//! **F6 input**, ~110 entries — the largest family — expressed by `field`, `button`, `chip`,
//! `select`, `collection` and `slider`.
//!
//! The reductions are R1, R2 and R3 (spec §18). §11's one flag absorbs sixteen named input
//! variants including `textarea`; §5's `Mode` absorbs the radio set and the segmented control;
//! a `Role` absorbs every button variant. The colour wheel, the dial and the font picker are R4 —
//! sub-cell rasterisation, one rasteriser and a different mapping.
//!
//! **Two entries are open rather than reduced**: ctrl-click and shift-click, because `rt::Input`
//! carries no modifier byte on a pointer event. The keyboard half of multi-select is complete; the
//! pointer half is inexpressible, and that is the runtime map's to change (§22).

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &[
    "button", "field", "select", "checkbox", "radio", "switch", "form", "slider",
];
