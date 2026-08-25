//! Applications written against the component surface, and the list of them as a value.
//!
//! # What this crate is for
//!
//! Every other crate in this workspace is checked by gates it ships itself. This one is checked by
//! **being a consumer**: each file in `examples/` is a program a person would recognise as an
//! application, written with no access to `vitui-engine`, and it compiles or it does not. Six of
//! the runtime's ten found defects were found by writing a consumer rather than another gate, and
//! architecture issue 23 was found by writing the first file here — `Driver` owned its `Screen`
//! privately and `attach` dropped the `WakeHandle`, so **no loop could be written at all** and the
//! only shape available was a spin at 100% of a core.
//!
//! The dependency list is `vitui-runtime` and `vitui-components`. Not the `vitui` facade, and the
//! difference is the whole proof: the facade re-exports the engine **entire**, so depending on it
//! would put `vitui::engine::Surface`, `View`, `Screen`, `Engine` and `LayerStack` in reach here, and
//! the claim *the component surface is sufficient* would evaporate without a line changing.
//!
//! **`Rect` used to be the example in that sentence and is no longer**, because runtime architecture
//! issue 22 made it `vitui_runtime::Rect`. That does not weaken the proof and it is worth saying why:
//! the runtime re-exports the engine's **vocabulary** — the types its own public surface names, plus
//! what is needed to construct one it accepts — under a rule gated in both directions. The facade
//! re-exports the engine's **machinery** as well, and an application that reaches a `View` is an
//! application saying the component surface is not finished. The scanner below is unaffected either
//! way: it looks for the crate name, and neither re-export puts that in a source file.
//!
//! # There is nothing to call in this library
//!
//! [`APPS`] is the list, and the tests below are the only thing that reads it. An application is a
//! file, not a function — `cargo run -p vitui-apps --example counter` is how you meet one.
//!
//! It is a value rather than a paragraph for the reason this repository gives everywhere else: a
//! list in prose is a list the next edit can contradict silently. Here that is not hypothetical —
//! the row carries the file name, a test opens it, and a second test walks `examples/` and asserts
//! the two sets are equal, so an application added without a row and a row without an application
//! both fail.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// What an application in `examples/` demonstrates, in one line.
///
/// The column exists so that a reader picking one to open does not have to open all of them, and so
/// that *what is not yet demonstrated* is answerable by looking rather than by remembering.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct App {
    /// The example's file stem, which is also its `--example` argument.
    pub name: &'static str,
    /// What it is, for somebody choosing what to read.
    pub what: &'static str,
    /// The parts of the surface it exercises. Names as a component author writes them.
    pub uses: &'static [&'static str],
    /// Where the shape came from, when it came from somewhere. An application copied from a known
    /// tutorial is worth more than one invented here: **its shape is not ours to argue with**, so
    /// what it cannot express is a fact about the surface rather than a taste.
    pub after: Option<&'static str>,
}

/// Every application in `examples/`.
///
/// The first is a port rather than an invention on purpose — see [`App::after`]. The second is not,
/// and the reason is that there is nothing to port: what it demonstrates is *one component and one
/// `Mode`*, and no other library's tutorial has an equivalent because no other library makes the
/// claim.
pub const APPS: [App; 5] = [
    App {
        name: "counter",
        what: "A bordered panel, a centred value, and Left/Right/q. The smallest program anybody \
               calls a TUI application",
        uses: &[
            "structure::panel_with",
            "text::text_with",
            "layout::rect::split_at_v",
            "keys::KeyMap",
            "ctx::Driver::wait",
        ],
        after: Some("https://ratatui.rs/tutorials/counter-app/basic-app/"),
    },
    App {
        name: "triage",
        what: "Mail triage over 200 000 messages that do not exist. Four collections on one \
               screen, one component and four values of `Mode` — a tab strip, a folder list, a \
               multi-select and a menu — with the cursor, the anchor and the span list printed \
               along the bottom, so the store is something you watch rather than something you \
               are told",
        uses: &[
            "collect::collection",
            "collect::Mode",
            "collect::CollState",
            "frame::face_paint",
            "structure::panel_with",
            "text::fit_with",
            "layout::Row",
            "layout::Col",
            "ctx::Ctx::focused",
            "ctx::Driver::wait",
        ],
        after: None,
    },
    App {
        name: "latency",
        what: "A live p50/p99 latency monitor with an SLO rule and a throughput chart. The volume, \
               the repertoire, the colour depth and the threshold's second axis are all under a \
               key, because each of them is a claim you have to watch to believe",
        uses: &[
            "chart::plot_with",
            "chart::chart_with",
            "chart::Series::push",
            "chart::raster::PlotState",
            "chart::raster::RUNGS",
            "structure::panel_with",
            "text::text_with",
            "layout::rect::split_at_h",
            "ctx::Ctx::deadline",
            "ctx::Driver::set_theme",
            "ctx::Driver::wait",
        ],
        after: None,
    },
    App {
        name: "ledger",
        what: "A general ledger of a million entries in twelve columns, both edges pinned. The \
               verb counter is on the screen and `Alt+1`/`Alt+2` move it by an order of magnitude \
               without moving the cell count; the footer sums a cell selection whose whole-column \
               gesture is one span at any length; and `Alt+s` against `Alt+S` is the editing \
               slot's revalidation with and without the caller carrying the positions across",
        uses: &[
            "collect::table_into",
            "collect::TableState",
            "collect::TableOpts",
            "collect::Cell",
            "collect::CellSel",
            "collect::solve_columns",
            "collect::visible_columns",
            "counters::Tally",
            "ink::Direct",
            "frame::face_paint",
            "order::Rows",
            "structure::panel_with",
            "text::fit_with",
            "layout::Col",
            "ctx::Ctx::focused",
            "ctx::Driver::unhandled",
            "ctx::Driver::wait",
        ],
        after: None,
    },
    App {
        name: "explorer",
        what: "A workspace of 258 313 nodes over a caller-owned flatten index. `←`/`→` fold and \
               unfold, and what the component does is **ask** — the splice, the reconciliation and \
               the revision all happen after the draw, in the application, because the index is \
               the caller's. `d` swaps the clamped indent for §7's unclamped one and the counters \
               along the bottom show every one of them preferring the defect except the ask",
        uses: &[
            "collect::tree_into",
            "collect::TreeState",
            "collect::TreeOpts",
            "collect::Node",
            "collect::defective::unclamped_indent",
            "order::Order::fold",
            "order::Order::unfold",
            "order::Asked::drain",
            "order::reconcile_splice",
            "counters::Tally",
            "ink::Ink",
            "frame::face_paint",
            "structure::panel_with",
            "text::fit_with",
            "layout::rect::split_at_v",
            "ctx::Ctx::focused",
            "ctx::Driver::wait",
        ],
        after: None,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn examples_dir() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples"))
    }

    /// **The list and the directory are the same set, in both directions.**
    ///
    /// The direction that is easy to forget is the second: a row whose file has been renamed still
    /// reads correctly, and a `cargo run --example` that fails is the only thing that would have
    /// caught it.
    #[test]
    fn every_row_is_a_file_and_every_file_is_a_row() {
        let listed: BTreeSet<&str> = APPS.iter().map(|a| a.name).collect();
        assert_eq!(listed.len(), APPS.len(), "two rows share a name");

        let mut on_disk = BTreeSet::new();
        for entry in std::fs::read_dir(examples_dir()).expect("examples/ exists") {
            let path = entry.expect("a readable entry").path();
            if path.extension().is_some_and(|e| e == "rs") {
                on_disk.insert(
                    path.file_stem()
                        .expect("a .rs file has a stem")
                        .to_str()
                        .expect("a utf-8 file name")
                        .to_owned(),
                );
            }
        }
        let on_disk: BTreeSet<&str> = on_disk.iter().map(String::as_str).collect();
        assert_eq!(listed, on_disk);
    }

    /// **Every row says something, and a port names where it came from.**
    #[test]
    fn every_row_carries_a_description_and_the_surface_it_exercises() {
        for app in APPS {
            assert!(!app.what.is_empty(), "{} says nothing", app.name);
            assert!(!app.uses.is_empty(), "{} exercises nothing", app.name);
        }
    }

    /// **No application names the engine.**
    ///
    /// The manifest says so — `[dependencies]` is two lines and neither is `vitui-engine` — and
    /// this is the same statement read off the source, which is the direction that catches a
    /// `vitui_engine::` arriving through a path dependency somebody added while debugging. It is
    /// `line.rs`'s arrangement one layer up, and its reason: a rule enforced only by a manifest is
    /// a rule the next manifest edit undoes silently.
    #[test]
    fn no_application_names_the_engine() {
        for app in APPS {
            let path = examples_dir().join(format!("{}.rs", app.name));
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("{} reads: {e}", path.display()));
            for (n, line) in source.lines().enumerate() {
                // The scanner's own mention is the one false positive, and it is this file rather
                // than that one — `components/src/frame.rs`'s deleted-helper scan reported itself
                // on its first run for exactly this shape, so the needle is spelled apart.
                assert!(
                    !line.contains(concat!("vitui", "_engine")),
                    "{}:{} names the engine: {}",
                    path.display(),
                    n + 1,
                    line.trim()
                );
            }
        }
    }
}
