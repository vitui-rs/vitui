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
//! **And the shape that replaced it was wrong in every file here for four tickets**, which is the
//! second thing this crate has found that no gate one crate down could. `Driver::unhandled` is *a
//! window onto the same queue, valid until the next frame begins*, and every loop read it **before**
//! its own frame — so an application acted on the previous frame's window, one wake late. For a
//! single keystroke that means never: measured on the shipped binaries, `reader` did not quit on `q`
//! at all and `explorer` quit a second late, because a `collection`'s type-ahead deadline happened to
//! supply the second wake. Components ticket 22's application found it by being run, and
//! `tests::every_loop_reads_the_unhandled_window_from_the_frame_that_has_just_drawn` is what keeps
//! it fixed.
//!
//! Beside it, the finding that pairs with it: **a printable character cannot be an application's quit
//! key while a `collection` holds the focus**, because a focused collection consumes every
//! text-bearing key into its type-ahead buffer (spec §5). `ledger` and `explorer` bind `Ctrl+Q`, and
//! `Ctrl` is not text.
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
pub const APPS: [App; 15] = [
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
    App {
        name: "reader",
        what: "A build log of 120 000 entries in a scroll area whose bars are reserved and whose \
               four bands are views. `u` measures the extent in rows instead of `Σ h` and `End` \
               then stops a third short of the content with no counter moving; `n` takes the four \
               bands away and the region count does not change; `b` swaps the always-cut gutters \
               for the fixpoint",
        uses: &[
            "scroll::scroll_area_into",
            "scroll::AreaState",
            "scroll::AreaOpts",
            "scroll::Hide",
            "scroll::parts",
            "scroll::Which",
            "counters::Tally",
            "ink::Direct",
            "structure::panel_with",
            "text::fit_with",
            "ctx::Ctx::request_into_view",
            "ctx::Ctx::visible_rows",
            "ctx::Driver::unhandled",
            "ctx::Driver::wait",
        ],
        after: None,
    },
    App {
        name: "settings",
        what: "A settings screen of twelve sections, where every claim of §8 is a key. `w` takes \
               the open height from the drawn extent instead of the sizing function and the \
               section latches at the whole panel with nothing on the screen wrong; `t` stops \
               writing the tail and the old body stays under a correct header; `x` with `k` is the \
               only arm on this screen where the vanish rule answers, because no gesture on a \
               section's own header can reach it",
        uses: &[
            "disclose::collapsible_into",
            "disclose::Collapse",
            "disclose::DiscloseOpts",
            "disclose::Height",
            "disclose::Focus",
            "disclose::Disclosure::used",
            "counters::Tally",
            "ink::Ink",
            "frame::face_paint",
            "input::button_into",
            "structure::panel_with",
            "text::chip_into",
            "text::fit_with",
            "ctx::Ctx::focus",
            "ctx::Driver::unhandled",
            "ctx::Driver::wait",
        ],
        after: None,
    },
    App {
        name: "compose",
        what: "A title and a body that are one component and one flag. The status bar prints the \
               caret's (byte, column) pair, the visual row it is on, the width the wrap index was \
               built at and the undo ring's two bounds — so §11's four gates are visible while a \
               person types, and the fourth of them moves while the terminal is resized. `--mega` \
               pastes 24 000 lines and nothing about the frame changes",
        uses: &[
            "input::field",
            "edit::Text",
            "edit::WrapKind",
            "edit::Caret",
            "edit::Ring",
            "structure::panel_with",
            "text::text_with",
            "layout::rect::split_at_v",
            "ctx::Ctx::focus",
            "ctx::Ctx::is_focused",
            "ctx::Driver::unhandled",
            "ctx::Driver::wait",
        ],
        after: None,
    },
    App {
        name: "console",
        what: "A menu bar, two `select`s and a command palette — one shell and one collection, \
               differing in a `Kind` and a `Placement`. The status bar prints how many layers the \
               census keeps alive, what the open popup was granted against what it wants and what \
               the screen has room for, whether its gutter needs a bar and whether its blur \
               position can see the pointer, so §12's short-screen case is visible while the \
               terminal is resized. Four ways out of a popup, and the third — clicking the widget \
               again — is the one nothing dismissed",
        uses: &[
            "input::select_with",
            "input::SelectState",
            "overlay::PopupState",
            "overlay::overlay_with",
            "overlay::popup_size",
            "overlay::gutter",
            "overlay::Kind",
            "collect::collection",
            "frame::face_paint",
            "structure::panel_with",
            "text::text_with",
            "layout::rect::split_at_v",
            "ctx::Ctx::focus",
            "ctx::Driver::layers_live",
            "ctx::Driver::unhandled",
            "ctx::Driver::wait",
        ],
        after: None,
    },
    App {
        name: "theatre",
        what: "A picture at four colour depths, a QR, a barcode and the audio three, under a \
               video player's chrome. `4` is the one to watch: every other component in the \
               library degrades to a worse drawing of itself and a picture becomes a description \
               of itself, 0 of 23 920 distinctions kept. `g` is the ladder and `b` is what makes \
               it mean something; `p` inverts the pairing and nothing on the screen goes wrong; \
               and the seek bar is drag capture — press jumps, move carries, release moves \
               nothing, from `Response::local` alone",
        uses: &[
            "media::picture_into",
            "media::sub_rows",
            "media::qr_into",
            "media::barcode_into",
            "media::waveform_into",
            "media::spectrum_into",
            "media::vu_meter_into",
            "media::Pixels",
            "media::Modules",
            "media::Census",
            "media::Palette",
            "media::defective::picture_inverted_into",
            "media::defective::picture_braille_into",
            "media::player::chrome_into",
            "media::player::scrub",
            "media::player::Player",
            "media::player::defective::chrome_collecting_into",
            "counters::Tally",
            "ink::Ink",
            "structure::panel_into",
            "text::fit_into",
            "layout::rect::split_at_v",
            "ctx::Driver::set_theme",
            "ctx::Driver::unhandled",
            "ctx::Driver::wait",
        ],
        after: None,
    },
    App {
        name: "browse",
        what: "A file browser whose preview pane really decodes on another thread. `k` then `s` is \
               the one to watch: key the question on the cursor's position instead of on its file, \
               re-sort the listing, and the pane goes on showing the file that used to be at that \
               position for ever — because the question still matches, so nothing posts, so \
               nothing wakes, so no frame corrects it. `l` makes the two status rows disagree with \
               no thread involved; `x` blanks the pane after a shrink and leaves 2 516 cells \
               nobody writes; `b` changes the fold count and not one cell of the screen",
        uses: &[
            "files::file_preview_pane_into",
            "files::PaneState::land",
            "files::PaneShape",
            "files::Preview",
            "files::Question",
            "collect::collection_into",
            "media::picture_into",
            "media::sub_rows",
            "counters::Tally",
            "ink::Ink",
            "order::Rows",
            "frame::face_paint",
            "work::Worker::hire",
            "work::Task",
            "work::Cancel",
            "ctx::Driver::wake",
            "ctx::Driver::unhandled",
            "ctx::Driver::wait",
        ],
        after: None,
    },
    App {
        name: "mixer",
        what: "Eight vertical faders, a horizontal master, and a band whose two thumbs share one \
               track. `x` is the one to press: it steps the focused fader fifty times, and `f` swaps \
               the arithmetic underneath it — the integer grid lands on exactly 0.5 with the thumb \
               on the middle cell, and the `f32` step lands on 0.4999998 with the thumb one cell \
               short, then on 0.99999934 where the thumb is right and the value has never reached \
               its own maximum. Drag the band's low thumb rightwards past the midpoint and the high \
               thumb jumps back to meet the pointer, which is why no range slider ships",
        uses: &[
            "input::slider_into",
            "input::stepped",
            "input::SliderOpts",
            "input::defective::float_stepped",
            "input::defective::Range",
            "structure::panel_into",
            "text::text_into",
            "scroll::Orient",
            "counters::Tally",
            "ink::Ink",
            "keys::press_with",
            "ctx::Ctx::with_key",
            "ctx::Ctx::focused",
            "ctx::Driver::unhandled",
            "ctx::Driver::wait",
        ],
        after: None,
    },
    App {
        name: "vitals",
        what: "A dashboard of six components composed of proved mechanisms: three toggles, two \
               meters, a sparkline over a hundred thousand samples, and four rules. `g` is the one \
               to press — it steps the glyph rung, and three things happen at once: the meter loses \
               its partial cell because a prefix has 8 sub-cells at the block rungs and 1 at ASCII, \
               the checkbox's tick becomes an `x` and the radio's bullet a `*`, and the switch does \
               not change at all, because its state is two words, a side and a face rather than a \
               glyph. `a` pushes one sample and the fold count moves; `Tab` moves the focus and it \
               does not",
        uses: &[
            "input::toggle_into",
            "input::Toggle",
            "input::ToggleOpts",
            "indicate::meter_into",
            "indicate::sparkline_into",
            "indicate::MeterOpts",
            "structure::rule_into",
            "structure::panel_into",
            "chart::Series",
            "chart::raster::PlotState",
            "chart::raster::geom",
            "chart::raster::RUNGS",
            "text::text_into",
            "counters::Tally",
            "ink::Ink",
            "ctx::Ctx::with_key",
            "theme::Themes::set_glyphs",
            "ctx::Driver::set_theme",
            "ctx::Driver::unhandled",
            "ctx::Driver::wait",
        ],
        after: None,
    },
    App {
        name: "roster",
        what: "A staff directory: a form of six labelled fields, a pager over 137 records, and a \
               status bar along the bottom — §18 R3's three compositions on one screen. `Ctrl+G` is \
               the one to press: it takes the form's `Group` scope away, the tab stops go from 2 to \
               7, `Tab` starts walking the fields, and **the arrows stop working entirely**, \
               because a container hears what its children hand back only through a scope and \
               `ScopeKind`'s other two arms are a modal and a code editor. `Ctrl+D` swaps the \
               density and two fields fall off the bottom; `Ctrl+F` makes the bar's segments as \
               wide as they measure, and `Ctrl+←`/`Ctrl+→` then scroll it under its own rectangle. \
               There is no `q` to bind: a focused field consumes every text-bearing key, which is \
               what a field is for",
        uses: &[
            "input::form_into",
            "input::FormOpts",
            "input::FormState",
            "input::form_row_id",
            "collect::pagination_into",
            "collect::PageOpts",
            "collect::CollState",
            "structure::status_bar_into",
            "structure::StatusOpts",
            "structure::Fill",
            "structure::panel_into",
            "text::text_into",
            "edit::Text",
            "counters::Tally",
            "ink::Ink",
            "ctx::Ctx::focused",
            "theme::Themes::set_density",
            "ctx::Driver::set_theme",
            "ctx::Driver::unhandled",
            "ctx::Driver::wait",
        ],
        after: None,
    },
    App {
        name: "gallery",
        what: "**Every built component on one screen**, which is obligation O2 — twenty-eight panels, \
               one per `built` row of the freeze, in the freeze's own order. It is the thinnest \
               application in this directory on purpose: the screen, the panel table and the \
               twenty-eight drawings are `vitui_components::gallery`'s, because spec §21 names two \
               defects to be measured *on the assembled gallery* and both are components tickets \
               whose gate is `cargo test`. This file iterates the table and mints no panel of its \
               own, and a source scan in the library crate says so from the other side. `t` is the \
               key to press — it re-imports and re-resolves the whole theme on a live frame, which is \
               the test of *degradation is resolved at construction* — and `Ctrl+T` is the spelling \
               that still arrives when a focused `field` has eaten the `t`. `Ctrl+G` and `Ctrl+L` \
               walk §16's nine cells, and `Ctrl+L` is where a human watches the traffic light go \
               monochrome. `--probe` prints the budget measured in the gallery and `--matrix` the \
               nine cells; `--panic` exists for `scripts/gallery-panic-gate.sh` and for nothing else",
        uses: &[
            "gallery::Gallery",
            "gallery::Sink",
            "gallery::grid",
            "gallery::pages",
            "gallery::matrix",
            "gallery::traffic_light",
            "gallery::shape",
            "counters::Tally",
            "ink::Direct",
            "ink::Ink",
            "ctx::Driver::attach",
            "ctx::Driver::set_theme",
            "ctx::Driver::unhandled",
            "ctx::Driver::wait",
            "ctx::Driver::wake",
            "work::Worker::hire",
            "gallery::rung_word",
            "gallery::tier_word",
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

    /// **Every name in a `uses` column is a name its own file actually spells.**
    ///
    /// Nothing checked this column until components ticket 39, and the review that found it out
    /// found the sharp instance: the gallery's row listed `gallery::PANELS`, which
    /// `vitui_components::gallery`'s own scan **forbids** that file from spelling — so the column
    /// documented an application doing exactly what a gate one crate over refuses. A stale row here
    /// is worse than a missing one, because the column is what somebody choosing what to read reads.
    ///
    /// The needle is the **last segment**, because a `uses` entry names the item where it is defined
    /// (`ctx::Driver::attach`) and a file spells it where it is called (`Driver::attach`), and a
    /// crate that re-exports at the root would make the module prefix a fiction either way.
    ///
    /// **Two exceptions, each named and each with its reason** — §21's rule, and a list rather than a
    /// count so that a third has to arrive as an argument. Both are `compose`'s, and both are types
    /// the application genuinely exercises without ever writing the name: `edit::Caret` arrives from
    /// `Text::caret()` and `edit::Ring` from the undo verbs, and a row that dropped them would say
    /// the application does not touch the two mechanisms components 24's whole ticket is about.
    #[test]
    fn every_name_in_a_uses_column_is_spelled_by_its_own_file() {
        const REACHED_WITHOUT_BEING_NAMED: [(&str, &str); 2] =
            [("compose", "edit::Caret"), ("compose", "edit::Ring")];
        let mut missing = Vec::new();
        let mut excepted = 0usize;
        for app in APPS {
            let path = examples_dir().join(format!("{}.rs", app.name));
            let source = std::fs::read_to_string(&path).expect("a readable example");
            for entry in app.uses {
                let last = entry.rsplit("::").next().expect("rsplit yields one");
                if source.contains(last) {
                    continue;
                }
                if REACHED_WITHOUT_BEING_NAMED.contains(&(app.name, *entry)) {
                    excepted += 1;
                    continue;
                }
                missing.push(format!("{}: {entry}", app.name));
            }
        }
        assert_eq!(missing, Vec::<String>::new());
        assert_eq!(
            excepted,
            REACHED_WITHOUT_BEING_NAMED.len(),
            "a named exception is no longer needed. Strike the row rather than keep counting to two \
             — a stated exception that has stopped applying is the shape this test exists to catch"
        );

        // **The other direction**, or a scan whose needle has stopped matching reports every column
        // clean. `Sink` is in the gallery's column and in its source; `PANELS` is in neither now.
        let gallery =
            std::fs::read_to_string(examples_dir().join("gallery.rs")).expect("a readable example");
        assert!(gallery.contains("Sink"));
        assert!(
            !gallery.contains("PANELS"),
            "the gallery names the panel table, which `vitui_components::gallery` forbids it"
        );
    }

    /// **Every loop reads `Driver::unhandled` from the frame that has just drawn, and not before
    /// it.**
    ///
    /// Components ticket 22's application found this and **every loop in this crate had it.**
    /// `Driver::unhandled` is documented as *a window onto the same queue, valid until the next frame
    /// begins*, so a loop shaped `wait → unhandled → frame` reads the **previous** frame's window and
    /// acts one wake late — which for a single keystroke means never, because nothing will wake it
    /// again. Measured on the shipped binaries: `reader` did not quit on `q` at all, and `explorer`
    /// quit a second late because a `collection`'s type-ahead deadline happened to supply the second
    /// wake.
    ///
    /// # It is a scan, and the scan counts frames rather than comparing two positions
    ///
    /// A gate over the *behaviour* would need a pty and a real terminal; what is checkable here is
    /// the order, in the source, inside the loop. The slice is from the last `loop {` or `while ` to
    /// the end of the file, and the assertion is that **exactly one** `driver.frame(` stands before
    /// the first `driver.unhandled(`.
    ///
    /// **One and not *at least* one, because the second half of the same mistake is an extra frame
    /// rather than a missing one.** `reader` drew a conditional second frame — the reveal's, which
    /// nothing else asks for — between its frame and this read, and `route::batch_len` folds several
    /// ordinary keys into one batch: `[End, q]` arrives together, `End` requests an into-view, and the
    /// extra frame replaces the queue and takes the undrained `q` with it. A comparison of two
    /// positions cannot see that; a count can.
    ///
    /// Both directions, and three of them: no frame before the read, two frames before it, and the
    /// shape that ships — so a scanner that has quietly stopped finding `driver.frame(` fails instead
    /// of passing.
    #[test]
    fn every_loop_reads_the_unhandled_window_from_the_frame_that_has_just_drawn() {
        let mut checked = 0usize;
        for app in APPS {
            let path = examples_dir().join(format!("{}.rs", app.name));
            let source = std::fs::read_to_string(&path).expect("a readable example");
            if !source.contains("driver.unhandled(") {
                // `counter` and `latency` read their keys through a `KeyMap` and never open this
                // window at all, which is a different arrangement rather than a missing one.
                continue;
            }
            assert!(
                reads_the_window_after_the_frame(&source),
                "`{}` reads `driver.unhandled()` before its own `driver.frame(` inside the loop, \
                 so it acts on the previous frame's window — one wake late, which for a single \
                 keystroke means never",
                app.name
            );
            checked += 1;
        }
        assert_eq!(
            checked, 13,
            "triage, ledger, explorer, reader, settings, compose, console, theatre, browse, mixer, \
             vitals, roster and gallery open the window; counter and latency read their keys \
             through a `KeyMap` instead"
        );

        // **The other directions**, or a scanner that has stopped finding `driver.frame(` reports
        // every loop as correct.
        let none_before = "fn main() {\n    loop {\n        let k = driver.unhandled();\n        \
                           driver.frame(|cx| ui(cx));\n    }\n}";
        assert!(
            !reads_the_window_after_the_frame(none_before),
            "the scan accepts the order it exists to forbid"
        );
        // `reader`'s own shape before this ticket: the reveal frame between the draw and the read.
        let two_before = "fn main() {\n    loop {\n        driver.frame(|cx| ui(cx));\n        \
                           if driver.inspect().into_view().is_some() {\n            \
                           driver.frame(|cx| ui(cx));\n        }\n        let k = \
                           driver.unhandled();\n    }\n}";
        assert!(
            !reads_the_window_after_the_frame(two_before),
            "a second frame before the read replaces the queue this window is onto"
        );
        let right = "fn main() {\n    loop {\n        driver.frame(|cx| ui(cx));\n        \
                     let k = driver.unhandled();\n    }\n}";
        assert!(reads_the_window_after_the_frame(right));
    }

    /// Whether the last loop in `source` draws **exactly once** before it opens the unhandled
    /// window.
    fn reads_the_window_after_the_frame(source: &str) -> bool {
        let loop_at = source
            .rfind("\n    loop {")
            .or_else(|| source.rfind("\n    while "))
            .unwrap_or(0);
        let body = &source[loop_at..];
        let Some(read) = body.find("driver.unhandled(") else {
            // A loop that never opens the window is not a loop this rule can be about.
            return false;
        };
        body[..read].matches("driver.frame(").count() == 1
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
