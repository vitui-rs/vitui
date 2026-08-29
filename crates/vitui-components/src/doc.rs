//! **O1's evidence as a value: one documentation page per built component, and a scan that opens
//! the file rather than trusting the list.**
//!
//! > **O1 catches an API that cannot be called from outside the crate** — which was C01's actual
//! > question — and O2 catches an inventory that has drifted from what ships. Neither catches a
//! > wrong cell; that is O3 and O5. (spec §17)
//!
//! O1 is stated as two gates that do not substitute for each other, and this module carries both.
//! The first is a **compile outcome** — `#![deny(missing_docs)]` on this crate, plus the doctests
//! run in CI, where `cargo test --doc` is spelled `cargo test --workspace` in the one serialised
//! invocation the `test` job makes. The second is a **count**: *components with 0 doc-tests == 0*,
//! asked over the **built** rows of [`INVENTORY`] rather than as a grep, and answered by
//! [`crate::obligations::o1`] over [`crate::obligations::DOC_TESTED`] — see that function for why
//! the population is `built` and what asking the other question would have cost.
//!
//! # A hand-written evidence list is a claim, so this module derives one beside it
//!
//! [`crate::obligations::DOC_TESTED`] is written out, for the reason [`crate::obligations::AXIS_SCENES`]
//! is: a `const fn` over the freeze would make the population and the evidence one expression, and
//! an equality between two things derived from each other holds. What holds it honest is
//! [`survey`], which opens each page's file, finds the declaration, takes the doc comment above it
//! and reports what is *in* it — and `tests::the_written_list_and_the_scan_agree` compares the two.
//!
//! # A `compile_fail` fence is the opposite of evidence for O1
//!
//! This crate carries fifteen of them and they are the negative gates, not the examples. A page
//! whose only fence is hostile has proved that a spelling does **not** compile, which is exactly
//! not the claim O1 makes. So [`running_examples`] counts only a fence that opens a doctest rustdoc
//! will *run* — ` ``` ` and ` ```rust ` and nothing else — and the fifteen are invisible to it.
//!
//! # The needle is the name and the boundary is either delimiter
//!
//! Components 33's rule, and this is the fifth place on the map that has met it: a join over
//! twenty-eight rows cannot dictate twenty-eight signatures. Two of the twenty-eight are generic —
//! `pub fn file_preview_pane<T, F>(` and `pub fn file_picker<'f, T>(` — so a needle ending in `(`
//! could never have matched either, and a gate green on the parenthesis needle would have been
//! green *by deleting the type parameter*. That is components 30's finding on `picture`, which is
//! `pub fn picture<P: Pixels>(` and no row of the freeze at all. The boundary here is `(` **or**
//! `<`.
//!
//! **The name alone is ambiguous across the crate and the freeze is what disambiguates it.** There
//! is a `pub fn text(` in `document.rs` and another in `keys.rs`, a `pub fn chip(` in `state.rs`
//! and a `pub fn table(` in `gates.rs`. None of them is a component, and nothing here has to know
//! that: the file is [`crate::inventory::Component::module`]'s, which is the first entry of the freeze's own
//! `families` column, so the join that finds the page is the same join §19 already gates the module
//! tree with.
//!
//! # Why the axes are a line and not a paragraph
//!
//! O1's fifth criterion is *every component's page states its hostile axes, so a reader knows which
//! of the four apply before they hit one* — and **fourteen of the twenty-nine built components
//! declare none at all**, so a page that simply mentions the axes it has is silent on nearly half
//! the freeze, and silence is indistinguishable from a page that forgot. That is the vacuity
//! [`crate::obligations::Verdict::of`] refuses one file over, arriving on the documentation axis.
//!
//! So every page carries [`AXES_LINE`] and the gate is an **equality against the freeze**, in both
//! directions: a page that lists an axis its row does not set fails exactly as loudly as one that
//! omits an axis its row does. A page with no axis says `none`, out loud, in the same place.

use crate::inventory::{Axis, INVENTORY};

/// The directory every page lives in, relative to the workspace root.
pub const SRC: &str = "crates/vitui-components/src";

/// **The line every component's page carries**, and the marker [`stated_axes`] reads.
///
/// It is a line rather than a sentence because a scan for a claim inside prose is a scan that
/// passes on the day somebody writes *it does not narrow*. What follows the marker on that line is
/// the whole of the claim: a comma-separated list of the freeze's own words in the freeze's own
/// order, backticked, or the word `none`. Everything explaining it goes on the lines beneath.
pub const AXES_LINE: &str = "**Hostile axes:**";

/// The word a page with no hostile axis states, so that *none* and *nothing written* are different
/// answers.
pub const NONE: &str = "none";

/// One built component's documentation page — **located rather than listed**.
///
/// See this module's header: the file is [`crate::inventory::Component::module`]'s and the item is the id, so a page
/// cannot be pointed at the wrong file by a typo in a table nobody reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Page {
    /// The freeze's id, which is also the function a caller writes.
    pub id: &'static str,
    /// The file it is declared in, relative to the workspace root.
    pub file: String,
    /// The axes the freeze sets for it, in [`Axis::ALL`]'s order.
    pub axes: Vec<Axis>,
}

impl Page {
    /// The axes as the words the page must state, or `["none"]`'s empty vector.
    #[must_use]
    pub fn axis_words(&self) -> Vec<&'static str> {
        self.axes.iter().map(|a| a.name()).collect()
    }
}

/// **The population: every row of the freeze that is built.**
///
/// Over `built` rather than over all twenty-nine, which is
/// [`crate::obligations::o2_everything_built_has_a_panel`]'s population and its reason too. See
/// [`crate::obligations::o1`] for why the difference is one row and what asking the other question
/// would have cost.
#[must_use]
pub fn pages() -> Vec<Page> {
    INVENTORY
        .iter()
        .filter(|c| c.built)
        .map(|c| Page {
            id: c.id,
            file: format!(
                "{SRC}/{}.rs",
                c.module().expect("every built row is homed in a module")
            ),
            axes: Axis::ALL.into_iter().filter(|a| c.declares(*a)).collect(),
        })
        .collect()
}

/// **The doc comment immediately above `pub fn <id>(` or `pub fn <id><`**, attributes stepped over.
///
/// `None` when there is no such declaration, or when the lines between it and the nearest doc
/// comment are not attributes — which is what *the item has no documentation at all* looks like
/// from here, because the walk then runs into the previous item's closing brace.
#[must_use]
pub fn doc_comment(source: &str, id: &str) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    let open = format!("pub fn {id}(");
    let generic = format!("pub fn {id}<");
    let decl = lines
        .iter()
        .position(|l| l.starts_with(&open) || l.starts_with(&generic))?;
    let mut i = decl;
    // Step over the attribute block. `#[expect(…)]` here is several lines long with a `reason =`
    // string in it, so this cannot be *lines beginning with `#`* — what it can be is *lines that do
    // not end an item*, which is exactly what separates an attribute from the code above it.
    while i > 0 && !lines[i - 1].trim_start().starts_with("///") {
        let above = lines[i - 1].trim_end();
        if above.is_empty() || above.ends_with('{') || above.ends_with('}') || above.ends_with(';')
        {
            return None;
        }
        i -= 1;
    }
    let end = i;
    let mut start = end;
    while start > 0 && lines[start - 1].trim_start().starts_with("///") {
        start -= 1;
    }
    if start == end {
        return None;
    }
    Some(
        lines[start..end]
            .iter()
            .map(|l| l.trim_start().trim_start_matches("///").trim_start())
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

/// **The body of every fence rustdoc will run**, one `String` per block.
///
/// A fence whose info string is anything but empty or `rust` is skipped, which is what keeps this
/// crate's fifteen `compile_fail` bodies out of O1's evidence — see this module's header.
///
/// # One walk, because two spellings of one walk is a defect waiting to be found
///
/// [`running_examples`] and [`running_example_text`] are the count and the text of this, and neither
/// re-walks the page: **a fence closes whatever it opened, running or not**, and that is exactly
/// what the first version of the count got wrong — with only a *running* flag, the line that closes
/// a `compile_fail` body has an empty info string and reads as the opening of a running one. The
/// crate is on the wrong side of that arm to notice, because every page here carries a running fence
/// anyway, so it is watched over a page written by hand. A second copy of the state machine is a
/// second place for that to come back.
#[must_use]
pub fn running_blocks(doc: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut open: Option<(bool, String)> = None;
    for line in doc.lines() {
        let t = line.trim();
        if let Some(info) = t.strip_prefix("```") {
            match open.take() {
                Some((running, body)) => {
                    if running {
                        blocks.push(body);
                    }
                }
                None => open = Some((matches!(info.trim(), "" | "rust"), String::new())),
            }
            continue;
        }
        if let Some((_, body)) = open.as_mut() {
            body.push_str(t);
            body.push('\n');
        }
    }
    blocks
}

/// **The text of every fence rustdoc will run**, concatenated. See [`running_blocks`].
#[must_use]
pub fn running_example_text(doc: &str) -> String {
    running_blocks(doc).concat()
}

/// How many fences on the page open a doctest rustdoc will run. See [`running_blocks`].
#[must_use]
pub fn running_examples(doc: &str) -> usize {
    running_blocks(doc).len()
}

/// **Whether an example calls `id` as a free function**, rather than merely containing its name.
///
/// # The needle `text(` matches `cx.text(`, and that is not a hypothetical
///
/// A substring search for `<id>(` was this scan's first spelling and it is wrong for the one page
/// where it matters most. [`vitui_runtime::ctx::Ctx::text`] exists and **every drawing doctest in
/// this crate calls it**, so `text`'s page could drop its `text(cx, …)` call, draw with
/// `cx.text(…)` instead, and go on holding — with O1 `Met` and nothing proving the API is callable
/// from outside the crate, which is O1's whole question. The collision is already live: the
/// `file_picker` example contains a `cx.text(` inside its `line` function.
///
/// So the call has to be a **free** one: the character before the name may not be `.`, and may not
/// be part of an identifier or a path. That rules out `cx.text(`, `self.text(` and
/// `some::other::text(` while leaving `text(cx, …)`, `let r = text(…)` and `(text(…))` alone.
#[must_use]
pub fn calls_free_function(body: &str, id: &str) -> bool {
    let needle = format!("{id}(");
    body.match_indices(&needle).any(|(at, _)| {
        at == 0
            || !matches!(
                body.as_bytes()[at - 1],
                b'.' | b':' | b'_' | b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9'
            )
    })
}

/// **The axes the page states**, as the words it wrote them in.
///
/// `None` when the page carries no [`AXES_LINE`] at all, which is a different failure from stating
/// the wrong set and is reported as one. An empty vector is the page saying [`NONE`] out loud.
#[must_use]
pub fn stated_axes(doc: &str) -> Option<Vec<String>> {
    let line = doc.lines().find(|l| l.contains(AXES_LINE))?;
    let rest = line.split_once(AXES_LINE)?.1.trim().trim_end_matches('.');
    if rest == NONE {
        return Some(Vec::new());
    }
    Some(
        rest.split(',')
            .map(|w| w.trim().trim_matches('`').to_string())
            .collect(),
    )
}

/// **What a page's scan found.**
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    /// The id, so a failure names the component rather than the index.
    pub id: &'static str,
    /// Whether the declaration is in the file the freeze's `families` column points at.
    pub declared: bool,
    /// How many fences on it rustdoc will run.
    pub examples: usize,
    /// Whether an example reaches the crate the way a consumer has to — through
    /// `vitui_components::` and not through `crate::`.
    ///
    /// **This is a belt over a barrier the mechanism already provides**, and it is worth saying so
    /// rather than letting the clause look load-bearing: a doctest compiles as its *own* crate, so
    /// `crate::input::checkbox` does not resolve to anything and an example written that way fails
    /// to compile before it reaches here. What the clause is really for is the reader — an example
    /// is the one part of a page a person copies, and it has to be copyable.
    pub through_the_public_surface: bool,
    /// Whether an example calls the component the page is about, as a **free** function.
    ///
    /// **The load-bearing clause of the three**, and the only one nothing else catches: a page can
    /// carry a perfectly good compiled example of something *else* — its options struct, a helper,
    /// the state type — and O1's claim is about the component. See [`calls_free_function`] for why
    /// it is not a substring search, and for the collision that is already live in this crate.
    pub calls_itself: bool,
    /// Whether any example names the engine, which a consumer of this crate cannot.
    ///
    /// A belt, like [`Found::through_the_public_surface`]: C6 makes this crate's dependency list
    /// `vitui-runtime` and nothing else, so `vitui_engine` is not a direct dependency and rustdoc
    /// passes no `--extern` for it. An example naming it fails to compile. It is asserted anyway
    /// because the thing being claimed — *nothing the runtime does not publish* — is a property of
    /// the **example**, and a barrier that is currently supplied by a manifest is a barrier one
    /// manifest edit away from being gone.
    pub names_the_engine: bool,
    /// The axes the page states, `None` when it states none at all.
    pub stated: Option<Vec<String>>,
}

impl Found {
    /// Whether the page's [`AXES_LINE`] states the freeze's own set, in the freeze's own order.
    ///
    /// The equality is two-directional on purpose: a page that lists an axis its row does not set
    /// fails exactly as loudly as one that omits an axis its row does. See this module's header.
    #[must_use]
    pub fn agrees_about_axes(&self, axes: &[&str]) -> bool {
        self.stated.as_deref().is_some_and(|s| s == axes)
    }

    /// Whether the page discharges O1 for its component, given the axes the freeze sets.
    #[must_use]
    pub fn holds(&self, axes: &[&str]) -> bool {
        self.declared
            && self.examples > 0
            && self.through_the_public_surface
            && self.calls_itself
            && !self.names_the_engine
            && self.agrees_about_axes(axes)
    }
}

/// **Run every page's scan.** The value a gate and a report both read.
#[must_use]
pub fn survey(read: impl Fn(&str) -> String) -> Vec<Found> {
    pages()
        .into_iter()
        .map(|page| {
            let source = read(&page.file);
            let Some(doc) = doc_comment(&source, page.id) else {
                return Found {
                    id: page.id,
                    declared: false,
                    examples: 0,
                    through_the_public_surface: false,
                    calls_itself: false,
                    names_the_engine: false,
                    stated: None,
                };
            };
            let body = running_example_text(&doc);
            Found {
                id: page.id,
                declared: true,
                examples: running_examples(&doc),
                through_the_public_surface: body.contains("vitui_components::"),
                calls_itself: calls_free_function(&body, page.id),
                names_the_engine: body.contains("vitui_engine"),
                stated: stated_axes(&doc),
            }
        })
        .collect()
}

/// **The ids whose page discharges O1**, derived from the scan.
///
/// This is the list [`crate::obligations::DOC_TESTED`] is compared against. See this module's
/// header for why there are two of them.
#[must_use]
pub fn doc_tested(read: impl Fn(&str) -> String) -> Vec<&'static str> {
    let axes: Vec<Vec<&'static str>> = pages().iter().map(Page::axis_words).collect();
    survey(read)
        .into_iter()
        .zip(axes)
        .filter(|(f, a)| f.holds(a))
        .map(|(f, _)| f.id)
        .collect()
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// the refused spellings
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **One spelling this library refuses, and the page that says so.**
///
/// O1's seventh criterion: *each refused spelling has a doc line saying it is refused and why, so
/// the next reader reads rather than re-derives*. Six of them were named on the ticket and all six
/// were already written — by the ticket that made the refusal, which is the right place and the
/// place nothing was watching. A refusal is a **doc comment**, so no `use`, no `compile_fail` and
/// no type can see it go: the item it defends compiles identically the day the paragraph is
/// deleted, and the next reader re-derives the argument from scratch and usually gets it wrong.
#[derive(Clone, Copy, Debug)]
pub struct Refusal {
    /// What is refused, in the ticket's own words.
    pub what: &'static str,
    /// The file the argument is written in, relative to the workspace root.
    pub file: &'static str,
    /// A phrase from the paragraph, matched against [`crate::overlay::flattened`] so that a reflow
    /// is not a failure.
    pub needle: &'static str,
}

/// **The six refusals O1 owes a doc line**, each with the file it is written in.
///
/// Five carry a `WhyThereIsNo…` item beside the paragraph, which is this crate's shape for a
/// refusal that also has a compile outcome to show. The sixth — the animated fold — has none, and
/// that is not an omission: what it refuses is a *stored third state*, and there is no type to
/// point a `compile_fail` at because the whole claim is that the type does not exist. It is held by
/// a source scan in [`crate::disclose`] instead, and by this line.
pub const REFUSED: &[Refusal] = &[
    Refusal {
        what: "no animated fold",
        file: "crates/vitui-components/src/disclose.rs",
        needle: "an animated fold is refused",
    },
    Refusal {
        what: "no region `Stash`",
        file: "crates/vitui-components/src/disclose.rs",
        needle: "WhyThereIsNoRegionStash",
    },
    Refusal {
        what: "no arbitrary-byte caret",
        file: "crates/vitui-components/src/input.rs",
        needle: "WhyThereIsNoWayToPutTheCaretAtByteN",
    },
    Refusal {
        what: "no `Sel` enum",
        file: "crates/vitui-components/src/collect.rs",
        needle: "five independent bits and no Sel enum",
    },
    Refusal {
        what: "no downsampling in `plot`",
        file: "crates/vitui-components/src/chart.rs",
        needle: "WhyThereIsNoDownsample",
    },
    Refusal {
        what: "no overlay bars",
        file: "crates/vitui-components/src/scroll.rs",
        needle: "Bars are reserved. There is no overlay option",
    },
];

/// **The refusals whose doc line is missing.** Empty is the answer O1 wants.
#[must_use]
pub fn refusals_missing(read: impl Fn(&str) -> String) -> Vec<&'static str> {
    REFUSED
        .iter()
        .filter(|r| {
            let source = crate::overlay::flattened(&read(r.file));
            !source.contains(&crate::overlay::flattened(r.needle))
        })
        .map(|r| r.what)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obligations::{DOC_TESTED, o1};
    use std::path::PathBuf;

    fn read(relative: &str) -> String {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(relative);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// **Criterion 3: components with 0 doc-tests == 0.**
    ///
    /// The count, over the population the freeze says is built. Twenty-eight of twenty-nine, and
    /// the twenty-ninth is `spinner` — see [`o1`] for why the population is `built` and what asking
    /// the other question would have cost.
    #[test]
    fn every_built_component_carries_a_page_with_a_compiled_example() {
        let pages = pages();
        assert_eq!(
            pages.len(),
            29,
            "the built population, which is every row of the freeze"
        );
        let axes: Vec<Vec<&str>> = pages.iter().map(Page::axis_words).collect();
        let found = survey(read);
        let bad: Vec<&Found> = found
            .iter()
            .zip(&axes)
            .filter(|(f, a)| !f.holds(a))
            .map(|(f, _)| f)
            .collect();
        assert!(
            bad.is_empty(),
            "these pages do not discharge O1: {bad:#?}. A page owes a declaration in the file its \
             `families` column points at, a fence rustdoc runs, a call to the component through \
             `vitui_components::`, and the freeze's own axis set on its `{AXES_LINE}` line"
        );
    }

    /// **Criterion 5, in both directions: the page's axes are the freeze's axes.**
    ///
    /// Thirteen of the twenty-eight declare none, so the equality is what makes the line worth
    /// writing — see this module's header. Watched failing both ways: a page that has lost the line
    /// and a page that claims an axis its row does not set.
    #[test]
    fn a_page_states_the_axes_its_row_sets_and_no_others() {
        for (page, found) in pages().iter().zip(survey(read)) {
            let stated: Option<Vec<&str>> = found
                .stated
                .as_ref()
                .map(|s| s.iter().map(String::as_str).collect());
            assert_eq!(
                stated,
                Some(page.axis_words()),
                "`{}`'s page and the freeze disagree about its hostile axes",
                page.id
            );
        }
        // The two directions, over a page written by hand rather than over the crate.
        let none = format!("**Summary.**\n\n{AXES_LINE} {NONE}.\n");
        let one = format!("**Summary.**\n\n{AXES_LINE} `narrow`.\n");
        assert_eq!(stated_axes(&none), Some(Vec::new()));
        assert_eq!(stated_axes(&one), Some(vec!["narrow".to_string()]));
        assert_eq!(stated_axes("**Summary.** No line at all.\n"), None);
        assert_ne!(stated_axes(&none), stated_axes(&one));
        // Fourteen of the twenty-nine, which is the number that makes the equality load-bearing: a
        // page that merely mentions the axes it has is silent on nearly half the freeze, and
        // silence is indistinguishable from a page that forgot. `spinner` is the fourteenth — it
        // declares none of the four, which is the freeze's own reading and not this page's.
        assert_eq!(pages().iter().filter(|p| p.axes.is_empty()).count(), 14);
    }

    /// **The written list and the scan agree**, which is what stops either from being a claim.
    ///
    /// [`crate::obligations::AXIS_SCENES`]'s arrangement, one obligation over: the population and
    /// the evidence are two values, so a component that stops carrying an example fails here rather
    /// than agreeing with itself.
    #[test]
    fn the_written_list_and_the_scan_agree() {
        assert_eq!(
            DOC_TESTED.to_vec(),
            doc_tested(read),
            "`DOC_TESTED` and the scan of the pages have drifted"
        );
        assert!(o1(DOC_TESTED).met(), "O1");
    }

    /// **A `compile_fail` fence is not evidence, and this is the arm that says so.**
    ///
    /// Fifteen of this crate's fences are hostile and none of them may count. Written over a page
    /// by hand, because the crate is on the side of the argument where every page also carries a
    /// running fence.
    #[test]
    fn a_hostile_fence_is_invisible_to_the_count() {
        let hostile = "**Summary.**\n\n```compile_fail\nnot_a_thing();\n```\n";
        assert_eq!(running_examples(hostile), 0);
        assert_eq!(running_example_text(hostile), "");
        let both = format!("{hostile}\n```\nlet x = 1;\n```\n");
        assert_eq!(running_examples(&both), 1);
        assert_eq!(running_example_text(&both), "let x = 1;\n");
        for skipped in ["ignore", "text", "no_run", "compile_fail,E0499"] {
            let page = format!("```{skipped}\nlet x = 1;\n```\n");
            assert_eq!(running_examples(&page), 0, "{skipped}");
        }
    }

    /// **The doc comment is the one above the declaration, and an undocumented item has none.**
    ///
    /// The attribute walk is what this is about: three of the twenty-eight sit under a multi-line
    /// `#[expect(…)]` with a `reason =` string in it, so *lines beginning with `#`* is the rule
    /// that does not work and *lines that do not end an item* is the rule that does.
    #[test]
    fn the_walk_steps_over_attributes_and_stops_at_the_previous_item() {
        let documented = "/// One.\n/// Two.\n#[track_caller]\n#[expect(\n    lint,\n    reason = \
                          \"why\"\n)]\npub fn thing() -> u8 {\n";
        assert_eq!(
            doc_comment(documented, "thing").as_deref(),
            Some("One.\nTwo.")
        );
        let bare = "fn other() {}\n\npub fn thing() -> u8 {\n";
        assert_eq!(doc_comment(bare, "thing"), None);
        let elsewhere = "/// One.\npub fn other() -> u8 {\n";
        assert_eq!(doc_comment(elsewhere, "thing"), None);
        // The generic delimiter, which is components 33's rule and the fifth place it applies.
        let generic = "/// One.\npub fn thing<T>(t: T) {\n";
        assert_eq!(doc_comment(generic, "thing").as_deref(), Some("One."));
    }

    /// **Criterion 1 and criterion 2: the crate denies the lint, and CI runs the doctests.**
    ///
    /// O1's other half is a **compile outcome**, and neither of its two parts is a thing this
    /// process can observe about itself — a test cannot ask whether the crate it is linked into was
    /// compiled under a lint level, and it cannot watch a pipeline. What it can do is read the two
    /// files that decide, which is `crate::line`'s arrangement for exactly this shape.
    ///
    /// # `deny` and `warn` are the same gate here, and that is measured rather than assumed
    ///
    /// The workspace carries `[workspace.lints.rust] warnings = "deny"`, so a `missing_docs`
    /// warning is already an error and the crate compiled identically under either spelling — an
    /// undocumented `pub fn` added to `canvas.rs` failed the build both ways, with the same
    /// diagnostic. **`deny` is still the spelling that ships**, because the two are the same gate
    /// only while that table exists: written `warn`, this crate stops being documented the day
    /// somebody relaxes a lint table three files up, and nothing here would say so.
    ///
    /// # There is no sixth CI job, and `cargo test --doc` is spelled `cargo test --workspace`
    ///
    /// `.gitlab-ci.yml`'s note above its jobs says a sixth job means this pipeline alone can
    /// saturate a runner shared with every other repo on the machine, and the `test` job's own
    /// comment already records that its one serialised invocation *also runs the doctests, and the
    /// doctests are where the negative gates live*. A `cargo test --doc` line beside it would run
    /// the same seventy-six doctests a second time for a duplicate green.
    #[test]
    fn the_crate_denies_missing_docs_and_the_pipeline_runs_the_doctests() {
        let lib = read("crates/vitui-components/src/lib.rs");
        assert!(
            lib.contains("#![deny(missing_docs)]"),
            "O1's first half is a compile outcome and this is the line that makes it one"
        );
        let ci = read(".gitlab-ci.yml");
        assert!(
            ci.contains("cargo test --workspace -- --test-threads=1"),
            "the one invocation that runs this crate's doctests"
        );
        // **Watched failing, over a file that is not the one that decides** — and the first
        // spelling of this line pointed at *this* file, which contains the literal three lines up.
        // A scanner looking for a literal contains that literal: `crate::frame`'s first run and
        // `crate::disclose`'s two assembled needles, met here for the third time.
        assert!(!read("crates/vitui-components/src/canvas.rs").contains("#![deny(missing_docs)]"));
    }

    /// **Criterion 7: every refused spelling still has its doc line.**
    ///
    /// Watched failing, because a scan over six needles that all happen to be present is a scan
    /// nobody has seen work.
    #[test]
    fn every_refused_spelling_says_it_is_refused() {
        assert_eq!(refusals_missing(read), Vec::<&str>::new());
        assert_eq!(refusals_missing(|_| String::new()).len(), REFUSED.len());
        // And the argument is not written twice: each refusal names one file, and the file it names
        // is one of this crate's own.
        for r in REFUSED {
            assert!(r.file.starts_with(SRC), "{}", r.file);
        }
    }

    /// **A method call is not the component, and `text` is the page that proves it matters.**
    ///
    /// [`calls_free_function`]'s own arm. `Ctx::text` exists and every drawing doctest in this crate
    /// calls it, so a substring search for `text(` — which is what this scan first did — reads
    /// `true` for a page whose example never calls the component at all.
    #[test]
    fn a_method_call_of_the_same_name_is_not_a_call_to_the_component() {
        assert!(calls_free_function(
            "let r = text(cx, area, \"hi\");",
            "text"
        ));
        assert!(calls_free_function("text(cx, area, \"hi\");", "text"));
        assert!(calls_free_function(
            "assert!((text(cx, a, \"hi\")).changed);",
            "text"
        ));
        assert!(!calls_free_function(
            "let _ = cx.text(0, 0, \"hi\", paint);",
            "text"
        ));
        assert!(!calls_free_function("crate::document::text();", "text"));
        assert!(!calls_free_function("let _ = self.text(0);", "text"));
        assert!(!calls_free_function("subtext(cx);", "text"));
        // And the shipped pages are on the right side of it, `file_picker`'s included — its example
        // carries a `cx.text(` inside the `line` function it hands over, which is the live
        // collision this arm is about.
        let files = read("crates/vitui-components/src/files.rs");
        let doc = doc_comment(&files, "file_picker").expect("the page");
        let body = running_example_text(&doc);
        assert!(
            body.contains("cx.text("),
            "the collision is still in the example"
        );
        assert!(calls_free_function(&body, "file_picker"));
        assert!(!calls_free_function(&body, "text"));
    }

    /// **The scan is watched failing on a page that is missing each of the four things it owes.**
    ///
    /// A gate nobody has watched fail is not a gate (§21), and this one has five clauses that all
    /// read `true` on the shipped crate.
    #[test]
    fn a_page_missing_any_one_of_the_four_does_not_hold() {
        let whole = Found {
            id: "text",
            declared: true,
            examples: 1,
            through_the_public_surface: true,
            calls_itself: true,
            names_the_engine: false,
            stated: Some(vec!["narrow".to_string()]),
        };
        assert!(whole.holds(&["narrow"]));
        assert!(!whole.holds(&["scrolled"]), "the wrong axis set");
        assert!(
            !Found {
                examples: 0,
                ..whole.clone()
            }
            .holds(&["narrow"])
        );
        assert!(
            !Found {
                declared: false,
                ..whole.clone()
            }
            .holds(&["narrow"])
        );
        assert!(
            !Found {
                through_the_public_surface: false,
                ..whole.clone()
            }
            .holds(&["narrow"]),
            "an example that reaches the crate as `crate::` proves nothing about a consumer"
        );
        assert!(
            !Found {
                calls_itself: false,
                ..whole.clone()
            }
            .holds(&["narrow"])
        );
        assert!(
            !Found {
                names_the_engine: true,
                ..whole.clone()
            }
            .holds(&["narrow"]),
            "the engine is not nameable from a consumer of this crate, so an example that names it \
             is an example nobody can copy"
        );
        assert!(
            !Found {
                stated: None,
                ..whole
            }
            .holds(&["narrow"])
        );
    }

    /// **The file a page is looked for in is the freeze's, and the name alone would not do.**
    ///
    /// Three of this crate's own function names collide with a component's outside that component's
    /// module. None of them is reachable from here, and this is the assertion that says the join is
    /// what makes that true rather than luck.
    #[test]
    fn the_freeze_and_not_the_name_decides_which_file_a_page_is_in() {
        let by_id: Vec<(&str, String)> = pages().into_iter().map(|p| (p.id, p.file)).collect();
        for (id, file) in &by_id {
            assert!(file.starts_with(SRC), "{id}: {file}");
            assert!(!read(file).is_empty(), "{id}: {file}");
        }
        let of = |id: &str| by_id.iter().find(|(i, _)| *i == id).map(|(_, f)| f.clone());
        assert_eq!(
            of("text").as_deref(),
            Some("crates/vitui-components/src/text.rs")
        );
        assert_eq!(
            of("chip").as_deref(),
            Some("crates/vitui-components/src/text.rs")
        );
        assert_eq!(
            of("table").as_deref(),
            Some("crates/vitui-components/src/collect.rs")
        );
        // The three homonyms, each in a file no page points at.
        for (needle, file) in [
            ("pub fn text(", "crates/vitui-components/src/document.rs"),
            ("pub fn chip(", "crates/vitui-components/src/state.rs"),
            ("pub fn table(", "crates/vitui-components/src/gates.rs"),
        ] {
            assert!(read(file).contains(needle), "{file}");
            assert!(!by_id.iter().any(|(_, f)| f == file), "{file}");
        }
    }
}
