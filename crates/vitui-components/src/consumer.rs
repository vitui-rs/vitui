//! **O7's evidence as a value: which application exercises which component, joined by path.**
//!
//! > **`crates/vitui-apps` is checked by being a consumer.** Each file in `examples/` is a program
//! > a person would recognise as an application, written with no access to `vitui-engine`, and it
//! > compiles or it does not. (`vitui_apps`'s own header)
//!
//! That claim is made about the *crate* and about no component in it, and this module is the join
//! that makes it a claim about each of the rows this crate declares instead. O7 is the seventh obligation,
//! stated after the map closed and filed on the implementation backlog for [`crate::volume`]'s
//! reason: **the instrument is buildable without reopening anything.**
//!
//! # It is not a nicety, and the argument is four defects
//!
//! | application | what it found | how the gate missed it |
//! |---|---|---|
//! | `counter` | **no loop could be written at all** — `Driver` owned its `Screen` privately and `attach` dropped the `WakeHandle`, so the only shape available was a spin at 100% of a core | every test builds its own `Engine`; none of them writes an application's `while` |
//! | `counter` | **nothing holds the focus until an application says so** — 0 of 5 arrow presses, presenting as *the terminal lost focus* | a one-widget test seats the focus by clicking |
//! | `latency` | **`chart`'s rasteriser painted the whole column prefix for every point** — 952.61 ms against 2.72 at a million | every counter §20 has is an *output* counter, and the cost was in work that produces none |
//! | `ledger` | **a table drew its header one column into the border** when handed a panel's interior | every gate in the crate plays at `x == 0`, where the header's coordinates and the body's agree |
//!
//! Four defects, four different reasons, one shape: **a gate exercises the component where its
//! author put it, and an application puts it somewhere else.** Three of the four were found by a
//! person running the thing, which is the strongest form of the argument and the least
//! reproducible.
//!
//! # The population is *declared*, and that is a finding rather than a convenience
//!
//! [`crate::obligations::o1`] and [`crate::obligations::o3`] are over `built`, because a page or a
//! screen for a function that does not exist is not a thing anybody can write. This one is over the
//! rows this crate **actually declares**, read out of the module that homes each — because the
//! `built` column is a *claim*, and components ticket 33 found it had been lying for two tickets:
//! `slider`'s row read `built: true` and [`crate::input::MEMBERS`] listed the name while **no
//! `slider` existed anywhere in the crate**. The column is joined against the source by
//! `crate::inventory::tests::every_built_row_is_declared_in_the_module_that_homes_it`, so the two
//! readings agree today; asking the source directly is what keeps them agreeing when they next do
//! not.
//!
//! The population **moves**, which is what makes the query measure something: the day `spinner`
//! ships, [`declared`] returns twenty-nine and O7 asks about twenty-nine with no edit here.
//!
//! # The join is by path, and a bare-name scan is wrong twice over
//!
//! [`crate::doc`]'s header records the collision from the other end: there is a `pub fn text(` in
//! `keys.rs` and a `pub fn table(` in `gates.rs`, and **neither of them is a component**. An
//! application that wrote `keys::text(…)` or `gates::table()` would satisfy a scan for the bare
//! name, and `keys.rs`'s own `every_text_bearing_name_is_in_the_freeze` documents the second
//! collision by name and works around it with a `(cx` suffix — which is a heuristic and not a join.
//!
//! So what [`imports`] reads is the **import statement**: `use vitui_components::<module>::<item>`
//! is a `(module, item)` pair, `keys::text` is `("keys", "text")`, and the freeze already homes
//! every row through [`Component::module`] — so `text` is `("text", "text")` and the two never meet.
//! Both collisions are watched *not* counting in
//! `tests::the_two_bare_name_collisions_already_in_the_tree_do_not_count`.
//!
//! **An import is proof of use, and that is this crate's lint configuration rather than an
//! assumption**: `[workspace.lints.rust] warnings = "deny"` makes an unused import a build failure,
//! so an application cannot name a component it does not call.
//!
//! # Three rows share one machine, and the exception is named, counted and necessary
//!
//! Spec §1 gives every component three spellings — the ninety-per-cent one, `_with` and `_into` —
//! and [`crate::input`]'s three toggles share the last two: `checkbox`, `radio` and `switch` are
//! `toggle_with` and `toggle_into` with one field of `ToggleOpts` between them, which
//! `crate::input::tests::the_three_toggles_are_one_machine_and_each_answers_with_a_response` reads
//! out of the source. **`toggle_into` with `Toggle::Radio` *is* `radio`'s ink spelling**, and an
//! application that counts its own cells has no other way to draw one.
//!
//! [`SHARED`] is that exception as three rows with the item that names each, and
//! `tests::every_shared_row_is_necessary_and_there_is_no_fourth` asserts it is still needed — a
//! stated exception that has stopped applying is the shape this crate's exception lists exist to
//! catch.

use std::collections::BTreeSet;

use crate::inventory::{Component, INVENTORY};

/// Where the applications live, relative to the workspace root.
pub const EXAMPLES: &str = "crates/vitui-apps/examples";

/// The crate name an application spells a component through.
///
/// It is a plain literal and not a `concat!`, unlike the needles in [`crate::frame`] and
/// [`crate::dense`]: **this scan reads another crate's files**, so *a scanner looking for a literal
/// contains that literal* — the shape both of those met on their first run — cannot arise here. The
/// hostile fixtures below build their sources from this constant for the same reason the shipped
/// ones are read from disk: one spelling, one place.
pub const CRATE: &str = "vitui_components";

/// **A component whose `_with` and `_into` spellings belong to a machine it shares with its
/// siblings, and the item that names it inside them.**
///
/// See this module's header. Three rows, and the count is the assertion.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shared {
    /// The freeze's id.
    pub id: &'static str,
    /// The shared machine's stem, whose `_with` and `_into` are this row's second and third
    /// spellings.
    pub machine: &'static str,
    /// The type an application imports from the same module to name the row.
    pub kind: &'static str,
    /// The variant of [`Shared::kind`] that names this row and no other.
    pub selector: &'static str,
    /// Why the row has no `_with` and `_into` of its own, in one line.
    pub why: &'static str,
}

/// **The three rows of the freeze that share one machine.** See this module's header.
pub const SHARED: &[Shared] = &[
    Shared {
        id: "checkbox",
        machine: "toggle",
        kind: "Toggle",
        selector: "Toggle::Check",
        why: "a checkbox, a radio and a switch are one body and one field of `ToggleOpts`, so the \
              spelling that takes an `Ink` is `toggle_into` and the thing that names the row is the \
              kind",
    },
    Shared {
        id: "radio",
        machine: "toggle",
        kind: "Toggle",
        selector: "Toggle::Radio",
        why: "`checkbox`'s, and a radio *set* is `collection` at `Mode::Options` rather than this \
              row at all",
    },
    Shared {
        id: "switch",
        machine: "toggle",
        kind: "Toggle",
        selector: "Toggle::Switch",
        why: "`checkbox`'s. Its state is two words, a side and a face — three axes and only one of \
              them the palette — and none of that is a second machine",
    },
];

/// **The spellings of `id` its home module declares**, in spec §1's order.
///
/// The needle is the name and the boundary is either delimiter — components 33's rule, met here for
/// the sixth time on this map. `pub fn file_picker<'f, T>(` and `pub fn file_preview_pane<T, F>(`
/// could never have matched a needle ending in `(`, and a gate green on the parenthesis needle
/// would have been green **by deleting the type parameter**.
///
/// It is `starts_with` rather than `contains`, because what an application can import is a
/// **top-level** item: a `pub fn` indented inside an `impl` or a `mod` is not on the path this join
/// is about.
#[must_use]
pub fn spellings(module_source: &str, id: &str) -> Vec<String> {
    let mut out = Vec::new();
    for suffix in ["", "_with", "_into"] {
        let name = format!("{id}{suffix}");
        let open = format!("pub fn {name}(");
        let generic = format!("pub fn {name}<");
        if module_source
            .lines()
            .any(|l| l.starts_with(&open) || l.starts_with(&generic))
        {
            out.push(name);
        }
    }
    out
}

/// **Every `(module, item)` pair an application imports from this crate.**
///
/// The whole of the join. A `use` statement is a *statement* and not a line — runtime issue 22's
/// finding, whose first scan reported *the engine is unreachable* about a crate re-exporting all of
/// it — so this walks from `use <crate>::` to the `;` that closes it, however many lines that is.
///
/// # `self` is expanded, because a module-qualified call is the same path spelled differently
///
/// `use vitui_components::gallery::{self, Gallery, Sink};` puts `gallery::grid(…)` in the file, and
/// the path is the same one. So a `self` in the brace list adds every `<module>::<item>` the source
/// spells anywhere. It cannot manufacture a hit for a *different* module: the module is the one the
/// `use` named.
#[must_use]
pub fn imports(app_source: &str) -> BTreeSet<(String, String)> {
    let opener = format!("use {CRATE}::");
    let mut pairs = BTreeSet::new();
    let mut selfs: Vec<String> = Vec::new();
    let mut rest = app_source;
    while let Some(at) = rest.find(&opener) {
        let after = &rest[at + opener.len()..];
        let Some(end) = after.find(';') else { break };
        branch(&after[..end], &mut pairs, &mut selfs);
        rest = &after[end..];
    }
    for module in selfs {
        let prefix = format!("{module}::");
        let mut walk = app_source;
        while let Some(hit) = walk.find(&prefix) {
            let after = &walk[hit + prefix.len()..];
            let item: String = after
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !item.is_empty() {
                pairs.insert((module.clone(), item));
            }
            walk = after;
        }
    }
    pairs
}

/// One branch of a `use` statement: `<module>::<items>`, or a top-level `{…, …}` group of branches.
///
/// **The group form is handled although no application uses it today**, and the reason is which way
/// it fails: read as a module named `{collect`, a `use vitui_components::{collect::…, text::…};`
/// drops both components and O7 goes **red** for a component that is drawn. That is the safe
/// direction and it is still a wrong answer, and this crate has met the *needle has quietly stopped
/// matching* shape five times.
fn branch(statement: &str, pairs: &mut BTreeSet<(String, String)>, selfs: &mut Vec<String>) {
    let s = statement.trim();
    if let Some(inner) = s.strip_prefix('{').and_then(|g| g.strip_suffix('}')) {
        for part in split_top_level(inner) {
            branch(part, pairs, selfs);
        }
        return;
    }
    let mut segments = s.splitn(2, "::");
    let Some(module) = segments.next().map(str::trim) else {
        return;
    };
    if module.is_empty() || !module.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
        // `use vitui_components::Rect;` — a root re-export, which is no component's path.
        return;
    }
    for item in segments
        .next()
        .unwrap_or_default()
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
    {
        match item {
            "" => {}
            "self" => selfs.push(module.to_owned()),
            _ => {
                pairs.insert((module.to_owned(), item.to_owned()));
            }
        }
    }
}

/// The comma-separated parts of a brace group, at depth zero.
fn split_top_level(inner: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, c) in inner.char_indices() {
        match c {
            '{' => depth += 1,
            '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&inner[start..]);
    parts
}

/// Whether one application exercises one component, by path.
///
/// Two ways in and no third: the row's own spellings, or — for the three rows of [`SHARED`] — the
/// machine's spelling **and** the kind that names the row, both imported from the same module.
#[must_use]
pub fn exercises(
    app_source: &str,
    pairs: &BTreeSet<(String, String)>,
    module: &str,
    id: &str,
    spellings: &[String],
) -> bool {
    let named = |item: &str| pairs.contains(&(module.to_owned(), item.to_owned()));
    if spellings.iter().any(|s| named(s)) {
        return true;
    }
    let Some(shared) = SHARED.iter().find(|s| s.id == id) else {
        return false;
    };
    let machine = [
        format!("{}_with", shared.machine),
        format!("{}_into", shared.machine),
    ];
    machine.iter().any(|m| named(m)) && named(shared.kind) && app_source.contains(shared.selector)
}

/// **One application, as the join reads it**: its name, its source and the `(module, item)` pairs it
/// imports from this crate.
struct Read {
    name: String,
    source: String,
    pairs: BTreeSet<(String, String)>,
}

/// One row of the freeze and the applications that exercise it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Coverage {
    /// The freeze's id.
    pub id: &'static str,
    /// The module that homes it, which is the first half of every pair the join looks at.
    pub module: &'static str,
    /// Whether this crate declares it at all — [`declared`]'s population, read row by row.
    pub declared: bool,
    /// The applications that exercise it, in the directory's own order.
    pub apps: Vec<String>,
}

/// **The scan: every row of the freeze against every application.**
///
/// `read` takes a path relative to the workspace root, which is [`crate::doc::survey`]'s
/// arrangement and for its reason — the hostile arms hand it a source written by hand, so the
/// negative directions are one call away rather than a file nobody wants in the tree.
#[must_use]
pub fn coverage(read: impl Fn(&str) -> String, apps: &[String]) -> Vec<Coverage> {
    let sources: Vec<Read> = apps
        .iter()
        .map(|name| {
            let source = read(&format!("{EXAMPLES}/{name}.rs"));
            let pairs = imports(&source);
            Read {
                name: name.clone(),
                source,
                pairs,
            }
        })
        .collect();
    INVENTORY
        .iter()
        .map(|c| {
            let module = c
                .module()
                .expect("every row of the freeze is homed in a module");
            let module_source = read(&format!("{}/{module}.rs", crate::doc::SRC));
            let spelt = spellings(&module_source, c.id);
            let apps = sources
                .iter()
                .filter(|a| exercises(&a.source, &a.pairs, module, c.id, &spelt))
                .map(|a| a.name.clone())
                .collect();
            Coverage {
                id: c.id,
                module,
                declared: !spelt.is_empty(),
                apps,
            }
        })
        .collect()
}

/// **The population: every row of the freeze this crate declares.** See this module's header.
#[must_use]
pub fn declared(read: impl Fn(&str) -> String) -> Vec<&'static str> {
    INVENTORY
        .iter()
        .filter(|c| {
            let module = c
                .module()
                .expect("every row of the freeze is homed in a module");
            !spellings(&read(&format!("{}/{module}.rs", crate::doc::SRC)), c.id).is_empty()
        })
        .map(|c| c.id)
        .collect()
}

/// **The evidence, derived**: the ids at least one application exercises, in [`INVENTORY`]'s order.
///
/// [`crate::obligations::APPLIED`] is the same list written out, and
/// `crate::obligations::tests::the_written_list_and_the_scan_agree_about_o7` compares the two — the
/// arrangement `DOC_TESTED` and `crate::doc::doc_tested` already use, for O1's reason: a `const fn`
/// over the scan would make the population and the evidence one expression.
#[must_use]
pub fn applied(read: impl Fn(&str) -> String, apps: &[String]) -> Vec<&'static str> {
    coverage(read, apps)
        .into_iter()
        .filter(|c| !c.apps.is_empty())
        .map(|c| c.id)
        .collect()
}

/// The row of the freeze `id` names, or `None`.
#[must_use]
pub fn row(id: &str) -> Option<&'static Component> {
    INVENTORY.iter().find(|c| c.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn read(relative: &str) -> String {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(relative);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// The applications on disk, in name order. `vitui_apps::APPS` is the same set as a value and
    /// **this crate cannot name it** — the dependency arrow runs the other way — so the directory is
    /// the only reading available here. `vitui_apps::tests::every_row_is_a_file_and_every_file_is_a_row`
    /// is the join that keeps the two the same set.
    fn applications() -> Vec<String> {
        let dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(EXAMPLES);
        let mut out: Vec<String> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .map(|entry| entry.expect("a readable entry").path())
            .filter(|p| p.extension().is_some_and(|e| e == "rs"))
            .map(|p| {
                p.file_stem()
                    .expect("a .rs file has a stem")
                    .to_str()
                    .expect("a utf-8 file name")
                    .to_owned()
            })
            .collect();
        out.sort();
        out
    }

    /// **The two bare-name collisions already in the tree, watched not counting.**
    ///
    /// A test that passes with a heuristic suffix is a test that will pass again when the next
    /// collision has a different shape — `keys.rs`'s own workaround is `(cx`, and `gates::table()`
    /// takes no arguments at all, so that needle would have to grow a second special case rather
    /// than stop needing the first.
    ///
    /// Both directions: the same file with the *component's* module in the path counts.
    #[test]
    fn the_two_bare_name_collisions_already_in_the_tree_do_not_count() {
        let text_module = read(&format!("{}/text.rs", crate::doc::SRC));
        let collect_module = read(&format!("{}/collect.rs", crate::doc::SRC));
        let text_spellings = spellings(&text_module, "text");
        let table_spellings = spellings(&collect_module, "table");
        assert_eq!(text_spellings, ["text", "text_with", "text_into"]);
        assert_eq!(table_spellings, ["table", "table_into"]);

        // `keys::text` is `crate::keys`'s key-name renderer and is no component; `gates::table`
        // prints the register.
        let wrong = format!(
            "use {CRATE}::keys::text;\nuse {CRATE}::gates::table;\nfn main() {{ text(); table(); }}\n"
        );
        let pairs = imports(&wrong);
        assert!(pairs.contains(&("keys".to_owned(), "text".to_owned())));
        assert!(pairs.contains(&("gates".to_owned(), "table".to_owned())));
        assert!(
            !exercises(&wrong, &pairs, "text", "text", &text_spellings),
            "`keys::text` counted as the `text` component"
        );
        assert!(
            !exercises(&wrong, &pairs, "collect", "table", &table_spellings),
            "`gates::table` counted as the `table` component"
        );

        // **And the other direction**, or a join whose needle has stopped matching reports every
        // component uncovered and nothing says which failure it is.
        let right = format!("use {CRATE}::text::text_with;\nuse {CRATE}::collect::table_into;\n");
        let pairs = imports(&right);
        assert!(exercises(&right, &pairs, "text", "text", &text_spellings));
        assert!(exercises(
            &right,
            &pairs,
            "collect",
            "table",
            &table_spellings
        ));
    }

    /// **A `use` is a statement and not a line**, and `self` is a path spelled differently.
    #[test]
    fn the_import_scan_reads_statements_and_expands_a_self() {
        let wrapped = format!(
            "use {CRATE}::files::{{\n    Entry, PickerOpts,\n    file_picker,\n}};\nfn main() {{}}\n"
        );
        let pairs = imports(&wrapped);
        assert!(pairs.contains(&("files".to_owned(), "file_picker".to_owned())));
        assert!(pairs.contains(&("files".to_owned(), "Entry".to_owned())));

        // A `self` in the brace list reaches what the file calls through the module prefix, and
        // reaches nothing under any other module.
        let qualified =
            format!("use {CRATE}::scroll::{{self, Span}};\nfn main() {{ scroll::scrollbar(); }}\n");
        let pairs = imports(&qualified);
        assert!(pairs.contains(&("scroll".to_owned(), "scrollbar".to_owned())));
        assert!(!pairs.contains(&("text".to_owned(), "scrollbar".to_owned())));

        // A root re-export is nobody's module, and is not read as one.
        let root = format!("use {CRATE}::Rect;\nfn main() {{}}\n");
        assert!(imports(&root).is_empty());
    }

    /// **The exception is still necessary, and there is no fourth row.**
    ///
    /// Two halves, because either alone rots: the machine's `_with` and `_into` must exist, and the
    /// row's own must **not** — the day `checkbox_into` ships, this row stops being an exception and
    /// striking it is the edit, not keeping the count at three.
    #[test]
    fn every_shared_row_is_necessary_and_there_is_no_fourth() {
        assert_eq!(SHARED.len(), 3);
        for shared in SHARED {
            let c = row(shared.id).unwrap_or_else(|| panic!("`{}` is not a row", shared.id));
            let module = c.module().expect("homed");
            let source = read(&format!("{}/{module}.rs", crate::doc::SRC));
            assert_eq!(
                spellings(&source, shared.id),
                [shared.id],
                "`{}` declares a `_with` or an `_into` of its own, so the shared machine is no \
                 longer the only way an application can draw one. Strike the row",
                shared.id
            );
            let machine = spellings(&source, shared.machine);
            for owed in [
                format!("{}_with", shared.machine),
                format!("{}_into", shared.machine),
            ] {
                assert!(
                    machine.contains(&owed),
                    "`{owed}` is not declared in `{module}.rs`, so `{}`'s second and third \
                     spellings are not this machine's",
                    shared.id
                );
            }
            assert!(
                shared.selector.starts_with(&format!("{}::", shared.kind)),
                "`{}`'s selector does not name its own kind",
                shared.id
            );
            assert!(!shared.why.is_empty());
        }
        // The three selectors name three different rows, or one import would cover all of them.
        let selectors: BTreeSet<&str> = SHARED.iter().map(|s| s.selector).collect();
        assert_eq!(selectors.len(), SHARED.len());
    }

    /// **The shared machine counts only with the kind that names the row.**
    ///
    /// `toggle_into` alone is three components at once, which is the equality O7 exists to refuse in
    /// miniature: evidence that cannot tell one row from another is evidence for none of them.
    #[test]
    fn the_shared_machine_counts_only_with_the_kind_that_names_the_row() {
        let input = read(&format!("{}/input.rs", crate::doc::SRC));
        let checkbox = spellings(&input, "checkbox");
        let radio = spellings(&input, "radio");
        assert_eq!(checkbox, ["checkbox"]);

        let only_check = format!(
            "use {CRATE}::input::{{Toggle, ToggleOpts, toggle_into}};\n\
             fn main() {{ let k = Toggle::Check; }}\n"
        );
        let pairs = imports(&only_check);
        assert!(exercises(
            &only_check,
            &pairs,
            "input",
            "checkbox",
            &checkbox
        ));
        assert!(
            !exercises(&only_check, &pairs, "input", "radio", &radio),
            "a `Toggle::Check` counted as a radio"
        );

        // The machine without the kind imported is not a path join at all.
        let no_kind = format!("use {CRATE}::input::toggle_into;\nfn main() {{ Toggle::Check; }}\n");
        let pairs = imports(&no_kind);
        assert!(!exercises(&no_kind, &pairs, "input", "checkbox", &checkbox));
    }

    /// **Every row of the freeze that this crate declares is exercised by an application.**
    ///
    /// O7's second equality at the level this file can measure it: the verdict is
    /// [`crate::obligations::o7_everything_declared_has_an_application`] and this is the failing set
    /// it would report, named rather than counted so that a regression says *which component*.
    #[test]
    fn every_declared_component_is_exercised_by_an_application() {
        let apps = applications();
        let coverage = coverage(read, &apps);
        assert_eq!(coverage.len(), INVENTORY.len());
        let owed: Vec<&str> = coverage
            .iter()
            .filter(|c| c.declared && c.apps.is_empty())
            .map(|c| c.id)
            .collect();
        assert!(
            owed.is_empty(),
            "{owed:?} are declared in this crate and no application in `{EXAMPLES}` exercises them. \
             **A component ticket ships an application** — see `vitui_apps`'s header"
        );
        // **The population and the one row outside it.** `spinner` is the row no ticket has built,
        // and it is absent from the population rather than red inside it — a query stuck red is
        // `Verdict::of`'s vacuity failure in mirror image.
        let population: Vec<&str> = coverage
            .iter()
            .filter(|c| c.declared)
            .map(|c| c.id)
            .collect();
        assert_eq!(population.len(), 28);
        assert_eq!(
            coverage
                .iter()
                .filter(|c| !c.declared)
                .map(|c| c.id)
                .collect::<Vec<_>>(),
            vec!["spinner"],
            "the rows this crate does not declare. Components 46 ships `spinner`, and the \
             population moves with it rather than with an edit here"
        );
        assert_eq!(population, declared(read));
    }

    /// **The three rows that were owed when this obligation was built, and the application that
    /// closed them.**
    ///
    /// The count is here so that the next component to arrive without one is a failing test with a
    /// name in it rather than a number that moved. `scrollbar`, `sticky` and `file_picker` were the
    /// three: two of them are what a caller draws when it owns the offset itself — which is
    /// precisely the case `scroll_area` is *not* — and the third had no application at all.
    #[test]
    fn the_three_rows_o7_was_owed_are_covered_by_the_application_that_closed_them() {
        let apps = applications();
        let coverage = coverage(read, &apps);
        for id in ["scrollbar", "sticky", "file_picker"] {
            let row = coverage
                .iter()
                .find(|c| c.id == id)
                .unwrap_or_else(|| panic!("`{id}` is not a row"));
            assert!(
                row.apps.iter().any(|a| a == "sheet"),
                "`{id}` was owed an application and `sheet` is the one that closed it; it now reads \
                 {:?}",
                row.apps
            );
        }
    }
}
