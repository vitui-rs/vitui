//! The component library: windows, panels, charts, lists, trees, forms, pickers.
//!
//! # Status
//!
//! Being built one ticket at a time from `.scratch/vitui-components-architecture/spec.md`, whose
//! map is closed; the backlog is `.scratch/vitui-components-impl/`, forty-three tickets. **No
//! component is written yet.** What exists is the instruments the other forty are enumerable
//! against, and every one of them is a value rather than a paragraph:
//!
//! - [`INVENTORY`] — spec §17's v1 freeze as a value: **twenty-nine components in three tiers**,
//!   eleven columns each, with [`MOVED`] and [`COMPOSITIONS`] beside it. Not a paragraph, because
//!   *every obligation this map has stated as a sentence has been broken by someone who had read
//!   it* (ADR 0033).
//! - [`obligations`] — §17's five obligations as queries over the freeze, each returning a count or
//!   an equality. **Not one of them can be met yet, and every one of them says so out loud** rather
//!   than returning green over an empty population.
//! - [`gates`] — §21's register: **forty gates as rows, fourteen of them evaluated**, four pinned
//!   red with their failing sets, six unreachable across the crate line with what would have to
//!   become public, and sixteen with nothing yet to run over. An instrument is a value with a file
//!   in it, so a row that has stopped running turns the register red here.
//! - [`counters`] — §20's nine per-frame counters, **eight of which this crate can read**. The
//!   ninth, `marked`, panics rather than answering `0`, and so does the sentinel probe.
//! - The module tree, one module per family (§19), joined to the freeze by
//!   [`Component::families`].
//!
//! # The two rules a reader of this crate needs first
//!
//! **The crate line is `vitui-runtime` and nothing else** — constraint C6, and it is checked by a
//! test in the runtime that reads this package's `[dependencies]` table. It does not depend on
//! `vitui-engine`, which is why a component-facing crate cannot *name* an engine type even where it
//! can hold one.
//!
//! **Every component and every helper writes a partition of its rectangle** (§2, ADR 0026). Each
//! cell it is responsible for is written exactly once, and the cells it does not write are named in
//! its return value. The rule arrived in three widenings and the third came from a defect its own
//! author wrote fifteen cells after recording the rule.

// **No `unsafe` in any shipped crate above the engine** (ticket 21, ADR 0034). `forbid` and not
// `deny`, so nothing inside the crate can turn it back on with an `allow`; it subsumes the
// `unsafe_op_in_unsafe_fn` this line used to carry.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod counters;
pub mod gates;
pub mod inventory;
pub mod obligations;

// **One module per family, and the family is the module** (spec §19). The tree follows the survey's
// fifteen families so that a reader who knows what they want finds it without a search, and
// `INVENTORY`'s `families` column is the join that makes the mapping checkable rather than a naming
// convention. Four of them are empty today and each says why in its own header — F8 navigation, F11
// media, F13 system and F14 terminal-native ship no v1 component of their own, and F15 has no
// module here at all because its twenty-three entries emit no cells and are the runtime's.
//
// **The directory names are a spelling and not a decision.** §19 is explicit that no ticket
// ratified `architecture.md` §2's list and that renaming one reopens nothing. What is settled is
// the join.
pub mod canvas;
pub mod chart;
pub mod collect;
pub mod disclose;
pub mod files;
pub mod indicate;
pub mod input;
pub mod media;
pub mod monitor;
pub mod nav;
pub mod overlay;
pub mod scroll;
pub mod structure;
pub mod text;

mod family;

pub use family::Family;
// Re-exported at the root because every list, gate and ticket on this backlog names them, and
// `inventory::` in front of each is noise at the one place they are read. The module stays public:
// a reader looking for *why the axes are columns* should land on its documentation.
pub use inventory::{Axis, COMPOSITIONS, Component, INVENTORY, Layer, MOVED, Moved, Tier};
