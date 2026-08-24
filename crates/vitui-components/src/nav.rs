//! **F8 navigation**, ~35 entries, expressed by `collection` + `overlay`, and **no v1 component of
//! its own**.
//!
//! The reduction is R1 and R3 (spec §18): §5's `Mode` absorbs tabs, the content switcher, the menu
//! bar, the submenu and the command palette body, and §12's two axes absorb the popup that carries
//! them. The dock is v2.
//!
//! **An empty module is a claim and it is checked.** [`MEMBERS`] being empty is counted by
//! `inventory::tests::the_module_tree_and_the_families_column_agree`, which runs the join in both
//! directions. The one entry this family owes a ticket rather than a reduction is `command
//! palette` — R3 over
//! §5 and §12 with **no mechanism named as new**, which spec §18 says needs a ticket rather than an
//! assertion. It is one of the two §18 exemplars that were not built.

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &[];
