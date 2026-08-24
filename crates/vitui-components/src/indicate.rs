//! **F5 indicators**, ~46 entries, expressed by `meter`, `chart`, `plot`, `overlay` and deadlines.
//!
//! The reduction is R2 and R4 (spec §18). R2 is the larger half and it is ADR 0018's `Role`:
//! status LEDs, health pills, dot indicators, badge variants, inline messages, banners, alerts and
//! callouts are an **argument**, not a component, and ticket 34 carries a gate over
//! [`crate::INVENTORY`] asserting none of them appears as a row.
//!
//! `chart`, `plot` and `overlay` declare this family and are homed under F10 and F9.

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["meter", "sparkline", "spinner"];
