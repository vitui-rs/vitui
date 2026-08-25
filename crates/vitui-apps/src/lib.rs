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
//! difference is the whole proof: the facade re-exports the engine, so depending on it would make
//! `vitui::engine::Rect` nameable here and the claim *the component surface is sufficient* would
//! evaporate without a line changing.
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
/// One row so far, and the first one is a port rather than an invention on purpose — see
/// [`App::after`].
pub const APPS: [App; 1] = [App {
    name: "counter",
    what: "A bordered panel, a centred value, and Left/Right/q. The smallest program anybody \
           calls a TUI application",
    uses: &[
        "structure::panel_with",
        "text::text_with",
        "cells::Cells",
        "keys::KeyMap",
        "ctx::Driver::wait",
    ],
    after: Some("https://ratatui.rs/tutorials/counter-app/basic-app/"),
}];

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
