//! **What the shipped documentation may not say, as a gate, plus the floor under what it must.**
//!
//! A reader who opens this library on docs.rs has never read this workspace's backlog, will never
//! open its decision records, and cannot resolve a section number, a ticket number, a register row
//! or a path into a scratch directory. For that reader a citation is not shorthand: it is a dead
//! end where a fact should have been. The rule this file enforces is therefore **state the fact,
//! not the pointer** — whatever a citation stood in for is either something the caller needs, in
//! which case the sentence says it, or something only this workspace needs, in which case it lives
//! in the decision records and not in the crate.
//!
//! # The two halves
//!
//! **The citation count is an equality, per crate, against a declared budget.** Not an upper bound:
//! a count that drifts *down* without the budget moving is how a number in this repository goes
//! stale, and a stale budget is a lie about how much prose is left. So an improvement fails the
//! build until the budget records it, and a regression fails it too. The target is zero for all
//! four crates and every sweep lowers a number here.
//!
//! **The example count is a floor.** It may grow freely and may not shrink, because the second half
//! of the rule is that a public item a caller constructs or calls carries an example that compiles.
//! A crate can satisfy the first half by deleting prose; the floor is what stops that being an
//! improvement.
//!
//! # The needles are assembled from fragments
//!
//! A scanner looking for a literal contains that literal — the trap this workspace has met more
//! often than any other. Every needle below is built by [`needle`] from two halves that mean
//! nothing apart, so this file does not match itself if the scan is ever pointed at it.
//!
//! # What is scanned, and what is not
//!
//! Every `.rs` file under each publishable crate's `src/`, and within those files only the lines
//! that carry a rustdoc marker. That is the docs.rs surface plus the internal doc comments that sit
//! beside it, which is the population the rule is about. Ordinary comments are out of this gate's
//! reach and are swept by hand; so is anything under `tests/`, including this file.

use std::path::{Path, PathBuf};

/// The repository root, from this crate's manifest directory.
fn root() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

/// A needle, from two halves that are meaningless apart.
fn needle(head: &str, tail: &str) -> String {
    format!("{head}{tail}")
}

/// One crate's standing, in the two numbers this file gates.
struct Standing {
    /// The directory name under `crates/`.
    dir: &'static str,
    /// Rustdoc lines carrying at least one citation. Exact, and every sweep lowers it.
    citations: usize,
    /// Rustdoc example headings. A floor: it may grow, and may not shrink.
    examples: usize,
    /// Ordinary `//` comment lines carrying a citation. Exact, like the rustdoc count.
    ///
    /// A separate number rather than one total, because the two populations answer different
    /// questions: a rustdoc line is what a stranger reads on docs.rs, and a comment is what the
    /// next person to edit the file reads. Both are held, and the first was swept first.
    comments: usize,
}

/// **The ratchet.** Both numbers were measured, never chosen, and the whole point of the pair is
/// that neither can move without a person editing this table and seeing the other one.
const STANDING: &[Standing] = &[
    Standing {
        dir: "vitui",
        citations: 0,
        examples: 1,
        comments: 0,
    },
    Standing {
        dir: "vitui-engine",
        citations: 0,
        examples: 13,
        comments: 0,
    },
    Standing {
        dir: "vitui-runtime",
        citations: 0,
        examples: 28,
        comments: 0,
    },
    Standing {
        dir: "vitui-components",
        citations: 0,
        examples: 56,
        comments: 0,
    },
];

/// The crates whose sweep is finished, in **both** populations: no rustdoc line and no ordinary
/// comment points at a document the reader does not have.
///
/// A finished crate's budgets are zero and stay zero, which is the difference between a ratchet and
/// a treadmill. All four are on it now, in both populations.
const SWEPT: &[&str] = &["vitui", "vitui-engine", "vitui-runtime", "vitui-components"];

/// Citation vocabulary that is a plain substring: a decision-record number, a section mark, a
/// register row, a path into the backlog, an obligation letter, a map's own name.
fn plain_needles() -> Vec<String> {
    vec![
        needle("adr", " 0"),
        needle("docs", "/adr"),
        needle(".scr", "atch"),
        needle("components arch", "itecture"),
        needle("runtime arch", "itecture"),
        needle("engine arch", "itecture"),
        needle("\u{a7}", ""),
    ]
}

/// Citation vocabulary that is a word followed by a number, where the bare word is ordinary
/// English and the pair is a pointer: a ticket, an issue, a scene, a backlog's own numbering.
fn numbered_needles() -> Vec<String> {
    vec![
        needle("tick", "et "),
        needle("iss", "ue "),
        needle("sce", "ne "),
        needle("produc", "tion "),
        // The word alone is ordinary English — "both halves of the obligation are met". What points
        // at a document the reader does not have is the numbered one.
        needle("obligation ", "o"),
        // `a register entry` is this workspace's own vocabulary and reads as English; `register
        // entry 12` is a pointer at a row of a table the reader has no copy of.
        needle("register ent", "ry "),
        needle("register r", "ow "),
    ]
}

/// The text of one rustdoc line, marker and indentation removed, lowercased.
fn doc_text(line: &str) -> Option<String> {
    let line = line.trim_start();
    let rest = line
        .strip_prefix("///")
        .or_else(|| line.strip_prefix("//!"))?;
    Some(rest.trim().to_lowercase())
}

/// The text of one ordinary `//` comment line — never a rustdoc one.
fn comment_text(line: &str) -> Option<String> {
    let line = line.trim_start();
    if line.starts_with("///") || line.starts_with("//!") {
        return None;
    }
    Some(line.strip_prefix("//")?.trim().to_lowercase())
}

/// Whether a rustdoc line points at something a reader outside this repository cannot open.
fn cites(text: &str) -> bool {
    if plain_needles().iter().any(|n| text.contains(n.as_str())) {
        return true;
    }
    numbered_needles().iter().any(|n| {
        text.match_indices(n.as_str()).any(|(at, _)| {
            text[at + n.len()..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
        })
    })
}

/// The one crate where a scene number is not a citation.
///
/// `vitui-components` **owns** the scene list: it is `crate::scenes::SCENES`, its rows are
/// numbered, constants are named for those numbers (`PINS_SCENE_43`), and the section banners in
/// the screens that implement them read `── scene 17: the bar fixpoint ──`. A number that resolves
/// inside the crate the reader is already in is a local label, not a pointer at a document they
/// have no copy of — and where a name reads better than the number it is used anyway
/// (`crate::window`'s *scrolled screen*, `crate::dropped`'s *shrunk listing*).
///
/// Everywhere else — the engine, the runtime, the applications, `conform/` — a scene number points
/// at a list the reader does not have, and the needle stands.
const SCENE_NUMBERS_ARE_LOCAL: &str = "vitui-components";

/// The same line with its scene numbers taken out, for the one file that owns them.
fn scene_free(text: &str) -> String {
    let needle = needle("sce", "ne ");
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(needle.as_str()) {
        out.push_str(&rest[..at]);
        rest = &rest[at + needle.len()..];
        let digits = rest.len() - rest.trim_start_matches(|c: char| c.is_ascii_digit()).len();
        rest = &rest[digits..];
    }
    out.push_str(rest);
    out
}

/// An example heading, in either of rustdoc's two spellings.
fn opens_examples(text: &str) -> bool {
    text == "# examples" || text == "# example"
}

/// Every `.rs` file under a directory, in a stable order.
fn sources(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(at) = stack.pop() {
        let entries = std::fs::read_dir(&at).unwrap_or_else(|e| panic!("{}: {e}", at.display()));
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

/// One crate's two numbers as they are now, with the citing lines themselves for the failure
/// message — a count that cannot say *where* sends the next session to grep for it.
fn measure(dir: &str) -> (usize, usize, usize, Vec<String>) {
    let src = root().join("crates").join(dir).join("src");
    let mut citations = 0;
    let mut examples = 0;
    let mut comments = 0;
    let mut worst: Vec<String> = Vec::new();
    for file in sources(&src) {
        let text =
            std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        let short = file
            .strip_prefix(root())
            .unwrap_or(&file)
            .display()
            .to_string();
        let local_scenes = dir == SCENE_NUMBERS_ARE_LOCAL;
        for (n, line) in text.lines().enumerate() {
            if let Some(ordinary) = comment_text(line) {
                let ordinary = match local_scenes {
                    true => scene_free(&ordinary),
                    false => ordinary,
                };
                if cites(&ordinary) {
                    comments += 1;
                }
                continue;
            }
            let Some(doc) = doc_text(line) else { continue };
            if opens_examples(&doc) {
                examples += 1;
            }
            let doc = match local_scenes {
                true => scene_free(&doc),
                false => doc,
            };
            if cites(&doc) {
                citations += 1;
                if worst.len() < 12 {
                    worst.push(format!("{short}:{}", n + 1));
                }
            }
        }
    }
    (citations, examples, comments, worst)
}

/// The table covers the four crates that publish, and no more.
#[test]
fn the_ratchet_covers_every_publishable_crate() {
    let workspace = std::fs::read_to_string(root().join("Cargo.toml")).expect("workspace manifest");
    let members: Vec<String> = workspace
        .lines()
        .skip_while(|l| !l.starts_with("members = ["))
        .skip(1)
        .take_while(|l| !l.starts_with(']'))
        .filter_map(|l| l.split('"').nth(1).map(str::to_owned))
        .collect();
    let mut publishable: Vec<String> = Vec::new();
    for member in members {
        let manifest = std::fs::read_to_string(root().join(&member).join("Cargo.toml"))
            .unwrap_or_else(|e| panic!("{member}: {e}"));
        if !manifest
            .lines()
            .any(|l| l.starts_with("publish") && l.contains("false"))
        {
            publishable.push(member.rsplit('/').next().unwrap_or(&member).to_owned());
        }
    }
    publishable.sort();
    let mut gated: Vec<String> = STANDING.iter().map(|s| s.dir.to_owned()).collect();
    gated.sort();
    assert_eq!(
        gated, publishable,
        "a crate that publishes and is not in this table ships undocumented prose nobody counts"
    );
}

/// **The citation count, as an equality.** One run reports every crate, because a gate that stops
/// at the first failure makes the next session run it four times to learn where it stands.
#[test]
fn no_shipped_doc_comment_points_at_a_document_the_reader_does_not_have() {
    let mut wrong: Vec<String> = Vec::new();
    for standing in STANDING {
        let (citations, _, _, worst) = measure(standing.dir);
        if citations != standing.citations {
            wrong.push(format!(
                "{}: {citations} citing rustdoc lines against a table that says {}{}",
                standing.dir,
                standing.citations,
                if worst.is_empty() {
                    String::new()
                } else {
                    format!(" — first lines {worst:?}")
                }
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "a rustdoc line may not point at a document the reader does not have. Lower a number in \
         this table when prose leaves; raise nothing.\n  {}",
        wrong.join("\n  ")
    );
}

/// A swept crate's budget is zero, so finishing a crate is a thing this file can state.
#[test]
fn a_swept_crate_carries_no_citation_budget_at_all() {
    for dir in SWEPT {
        let standing = STANDING
            .iter()
            .find(|s| s.dir == *dir)
            .unwrap_or_else(|| panic!("{dir} is swept and is not in the table"));
        assert_eq!(
            standing.citations, 0,
            "{dir} is listed as swept and still carries a rustdoc budget of {}",
            standing.citations
        );
        assert_eq!(
            standing.comments, 0,
            "{dir} is listed as swept and still carries a comment budget of {}",
            standing.comments
        );
    }
}

/// **The comment count, as an equality.** A comment is not on docs.rs and is held anyway: the
/// person it is written for is the next one to edit the file, and a pointer into a document they
/// have no copy of wastes their time in exactly the way it wastes a stranger's.
#[test]
fn no_ordinary_comment_points_at_a_document_the_reader_does_not_have() {
    let mut wrong: Vec<String> = Vec::new();
    for standing in STANDING {
        let (_, _, comments, _) = measure(standing.dir);
        if comments != standing.comments {
            wrong.push(format!(
                "{}: {comments} citing comment lines against a table that says {}",
                standing.dir, standing.comments
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "an ordinary comment may not point at a document the reader does not have either. Lower a \
         number in this table when prose leaves; raise nothing.\n  {}",
        wrong.join("\n  ")
    );
}

/// **The example floor.** Deleting prose satisfies the equality above; it does not satisfy this.
/// Stated as an equality for the same reason the citations are: a floor the crate has already
/// climbed past is a number nobody can read.
#[test]
fn the_examples_a_caller_can_run_never_get_fewer() {
    let mut wrong: Vec<String> = Vec::new();
    for standing in STANDING {
        let (_, examples, _, _) = measure(standing.dir);
        if examples < standing.examples {
            wrong.push(format!(
                "{}: {examples} example headings, down from {}",
                standing.dir, standing.examples
            ));
        } else if examples > standing.examples {
            wrong.push(format!(
                "{}: {examples} example headings against a floor of {} — raise the floor in the \
                 edit that adds the example",
                standing.dir, standing.examples
            ));
        }
    }
    assert!(
        wrong.is_empty(),
        "an example a caller can run is the half of this rule that cannot be satisfied by \
         deletion.\n  {}",
        wrong.join("\n  ")
    );
}

// ── the other half of the rule: an example a caller can run ──────────────────────────────────────
//
// The half above is a prohibition — a doc line may not point at a document the reader does not
// have. Satisfying it by deleting prose is the failure mode the example floor exists to catch, and
// a floor is a count: it cannot say *which* items are documented, only how many headings there
// are. The three gates below say which.
//
// They are derived and not declared. Two of them read every rustdoc block in the four crates — one
// asks that an example the compiler sees is labelled, the other that a page opens by saying what
// the item does. The third reads `INVENTORY`, which is the freeze itself and already the population
// every other obligation joins on. None carries a hand-written list of items, because a
// hand-written population is a second declaration of something the crate already states, and an
// equality between two derivations of one declaration holds for ever.
//
// The first and the third are not substitutes, and the difference is worth keeping: deleting an
// example outright leaves the labelling gate green, because it has nothing left to look at, and
// fails the freeze gate. That is O1-and-O2's argument on a new pair, and it was watched both ways
// rather than reasoned about.

/// Whether a fence tag names a block the compiler sees.
///
/// `text` is an illustration, `ignore` is not compiled at all, and `compile_fail` is a gate whose
/// whole point is that it does not build. Everything else — the bare tag, `rust`, `no_run`,
/// `should_panic` — reaches the compiler, and reaching the compiler is what makes an example a
/// promise rather than a picture of one.
fn compiles(tag: &str) -> bool {
    let tag = tag.trim();
    tag.is_empty()
        || tag == "rust"
        || tag
            .split(',')
            .any(|t| matches!(t.trim(), "no_run" | "should_panic"))
}

/// What one rustdoc block says about its example.
#[derive(Default, Clone, Copy)]
struct Block {
    /// The item is a free function at the top level of its module, rather than a method.
    ///
    /// The freeze's id is *the function a caller writes*, and a builder constructor or an accessor
    /// may share that name — `ChartOpts::chart()` and `field(&self)` both do — so the join that
    /// finds a component's page has to say which of the two it means.
    free: bool,
    /// It carries at least one fence the compiler sees.
    running: bool,
    /// It carries an examples heading, **outside** every fence — a heading inside a `text` block is
    /// a picture of a heading.
    labelled: bool,
    /// Its first line is a description, rather than a section heading or a code fence.
    opens_with_prose: bool,
    /// Its block had a first line at all, so [`Block::opens_with_prose`] means something.
    has_prose: bool,
}

/// Every documented `pub fn` in one file, with what its block says.
///
/// Items inside a top-level `#[cfg(test)]` module are skipped whole: they are not compiled into a
/// shipped crate, so a fence there belongs to a gate and not to a caller.
fn documented_items(text: &str) -> Vec<(usize, String, Block)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let (mut block, mut have_doc, mut open) = (Block::default(), false, false);
    let mut attr = 0i32;
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("#[cfg(test)]") {
            while i < lines.len() && !lines[i].starts_with('}') {
                i += 1;
            }
            i += 1;
            block = Block::default();
            have_doc = false;
            open = false;
            continue;
        }
        if let Some(doc) = doc_text(line) {
            have_doc = true;
            if let Some(tag) = doc.strip_prefix("```") {
                match open {
                    true => open = false,
                    false => {
                        open = true;
                        if compiles(tag) {
                            block.running = true;
                        }
                    }
                }
            } else if !open && opens_examples(&doc) {
                block.labelled = true;
            }
            if !block.has_prose && !doc.is_empty() {
                block.has_prose = true;
                block.opens_with_prose = !doc.starts_with('#') && !doc.starts_with("```");
            }
            i += 1;
            continue;
        }
        let trimmed = line.trim();
        if have_doc && (attr > 0 || trimmed.starts_with("#[")) {
            attr += trimmed.matches('[').count() as i32 - trimmed.matches(']').count() as i32;
            attr = attr.max(0);
            i += 1;
            continue;
        }
        if have_doc && let Some(name) = item_name(trimmed) {
            block.free = !line.starts_with(char::is_whitespace);
            out.push((i + 1, name, block));
        }
        if !trimmed.is_empty() {
            block = Block::default();
            have_doc = false;
            open = false;
        }
        i += 1;
    }
    out
}

/// The name of a public function, from the line that declares it.
fn item_name(trimmed: &str) -> Option<String> {
    let rest = trimmed
        .strip_prefix("pub fn ")
        .or_else(|| trimmed.strip_prefix("pub const fn "))?;
    let end = rest.find(['(', '<']).unwrap_or(rest.len()).min(rest.len());
    Some(rest[..end].trim().to_owned())
}

/// The modules a crate hides from rustdoc, read off its own `lib.rs`.
///
/// A `#[doc(hidden)]` module is the crate saying this is not the surface a reader is offered, and
/// `vitui-components` says it of twenty-four: the scene screens, the registers, the defective arms
/// a gate plays against a correct one. A fence in one of those is a gate's reference — often a
/// deliberately *wrong* build — so labelling it as an example would be false on the one page where
/// it would be read. Derived rather than listed, so hiding a module exempts it in the same edit.
fn hidden_modules(dir: &str) -> Vec<String> {
    let lib = std::fs::read_to_string(root().join("crates").join(dir).join("src/lib.rs"))
        .unwrap_or_default();
    let mut out = Vec::new();
    let mut marked = false;
    for line in lib.lines() {
        let t = line.trim();
        if t == "#[doc(hidden)]" {
            marked = true;
            continue;
        }
        if let Some(rest) = t.strip_prefix("pub mod ") {
            if marked {
                out.push(rest.trim_end_matches(';').trim().to_owned());
            }
            marked = false;
            continue;
        }
        if !t.is_empty() && !t.starts_with("//") {
            marked = false;
        }
    }
    out
}

/// **Every example the compiler sees is labelled.**
///
/// An unlabelled fence still compiles and still runs, so nothing else in this workspace can see the
/// difference — and on the page a reader lands on, an unlabelled fence is a code block hanging off
/// the end of a paragraph rather than a section they can scroll to. The rule is *documentation
/// shows how to use the library*, and a section heading is what makes the showing findable.
#[test]
fn every_example_the_compiler_sees_is_labelled() {
    let mut bare: Vec<String> = Vec::new();
    let mut seen = 0usize;
    for standing in STANDING {
        let src = root().join("crates").join(standing.dir).join("src");
        let hidden = hidden_modules(standing.dir);
        for file in sources(&src) {
            let rel = file.strip_prefix(&src).unwrap_or(&file).to_path_buf();
            let top = rel
                .components()
                .next()
                .map(|c| {
                    c.as_os_str()
                        .to_string_lossy()
                        .trim_end_matches(".rs")
                        .to_owned()
                })
                .unwrap_or_default();
            if hidden.contains(&top) {
                continue;
            }
            let text = std::fs::read_to_string(&file)
                .unwrap_or_else(|e| panic!("{}: {e}", file.display()));
            let short = file
                .strip_prefix(root())
                .unwrap_or(&file)
                .display()
                .to_string();
            for (line, name, block) in documented_items(&text) {
                if !block.running {
                    continue;
                }
                seen += 1;
                if !block.labelled {
                    bare.push(format!("{short}:{line} `{name}`"));
                }
            }
        }
    }
    assert!(
        seen > 50,
        "only {seen} compiled examples found across four crates — this gate is reading nothing"
    );
    assert!(
        bare.is_empty(),
        "an example the compiler sees carries a heading, so a reader can find it on the page.\n  {}",
        bare.join("\n  ")
    );
}

/// **Every component the freeze declares carries an example a caller can run.**
///
/// The population is [`INVENTORY`](vitui::components::inventory::INVENTORY) itself — the freeze,
/// which already names what a caller draws, homes each row in a module and is what every
/// obligation in `vitui-components` joins on. Nothing is listed here twice: a component added to
/// the freeze arrives in this gate owing an example, and one whose entry point is renamed fails on
/// the join rather than going quietly absent.
#[test]
fn every_component_the_freeze_declares_shows_how_to_use_it() {
    use vitui::components::inventory::INVENTORY;

    let mut wrong: Vec<String> = Vec::new();
    let mut over = 0usize;
    for c in INVENTORY.iter().filter(|c| c.built) {
        over += 1;
        let module = c.module().expect("every built row is homed in a module");
        let path = root()
            .join("crates/vitui-components/src")
            .join(format!("{module}.rs"));
        let text =
            std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let found: Vec<Block> = documented_items(&text)
            .into_iter()
            .filter(|(_, name, b)| name == c.id && b.free)
            .map(|(_, _, b)| b)
            .collect();
        match found.as_slice() {
            [b] if b.running && b.labelled => {}
            [b] => wrong.push(format!(
                "{module}::{} — compiled example: {}, labelled: {}",
                c.id, b.running, b.labelled
            )),
            [] => wrong.push(format!(
                "{module}::{} — the freeze homes it here and no documented `pub fn` of that name is",
                c.id
            )),
            many => wrong.push(format!(
                "{module}::{} — {} documented functions of that name, so the join is ambiguous",
                c.id,
                many.len()
            )),
        }
    }
    assert_eq!(
        over, 29,
        "the freeze declares 29 built components and this gate saw {over}"
    );
    assert!(
        wrong.is_empty(),
        "a component a caller draws shows how to draw it.\n  {}",
        wrong.join("\n  ")
    );
}

/// **A page opens by saying what the item does.**
///
/// *What it does, what the caller must guarantee, what it costs* is the order these pages are
/// written in, and only the first third of it is gateable. The other two thirds are left to review
/// on purpose, and the reason is a measurement: of 709 section headings in the three shipped
/// crates, 68 are `# Panics` and 2 are `# Errors` — the rest are **narrative**, in the crate's own
/// words rather than in rustdoc's fixed vocabulary. A gate over section order would therefore not
/// be checking that a page answers the three questions; it would be pushing 550 headings towards a
/// vocabulary chosen for a different kind of library, and the pages would get worse to make the
/// gate green.
///
/// What is left is the part with no vocabulary in it. An item whose page opens with a heading or a
/// code block has buried the sentence a reader came for under the first thing the author felt like
/// explaining. All 1 948 documented functions in the three crates already open with a description,
/// so this is a property the code has rather than a target it is moving towards — which is the only
/// kind of thing worth an equality.
#[test]
fn a_page_opens_by_saying_what_the_item_does() {
    let mut buried: Vec<String> = Vec::new();
    let mut seen = 0usize;
    for standing in STANDING {
        let src = root().join("crates").join(standing.dir).join("src");
        for file in sources(&src) {
            let text = std::fs::read_to_string(&file)
                .unwrap_or_else(|e| panic!("{}: {e}", file.display()));
            let short = file
                .strip_prefix(root())
                .unwrap_or(&file)
                .display()
                .to_string();
            for (line, name, block) in documented_items(&text) {
                if !block.has_prose {
                    continue;
                }
                seen += 1;
                if !block.opens_with_prose {
                    buried.push(format!("{short}:{line} `{name}`"));
                }
            }
        }
    }
    assert!(
        seen > 1_000,
        "only {seen} documented functions found across four crates — this gate is reading nothing"
    );
    assert!(
        buried.is_empty(),
        "a page opens with what the item does, not with a section or a code block.\n  {}",
        buried.join("\n  ")
    );
}
