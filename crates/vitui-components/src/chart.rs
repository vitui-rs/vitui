//! **F10 charts**, ~55 entries, expressed by `chart`, `plot`, axes and two memos.
//!
//! The reduction is R4 alone (spec §18), and it is the cleanest of the six: **every warning mark in
//! this family means one thing** — a shape that is not axis-aligned costs sub-cell rasterisation —
//! so the donut, the pie, the radar, the violin, the sankey, the treemap and the globe are all the
//! same rasteriser with a different mapping from data to sub-cells. §13 priced the ladder exactly:
//! 2 / 16 / 256 states a cell for marks, 2 / 9 / 9 for prefixes.
//!
//! This is the family the `constructions` column exists for. §13 measured Unicode against Extended
//! on the rendered surface: **0 cells differ in the chart pane and 882 in the plot pane**, so the
//! third rung is `plot`'s alone.
//!
//! `meter` and `sparkline` declare this family and are homed under F5 — ticket 34 calls them
//! `chart`'s prefix construction at two rungs and `chart` at a small rectangle with no axes.

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["chart", "plot"];
