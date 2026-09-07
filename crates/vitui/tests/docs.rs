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
}

/// **The ratchet.** Both numbers were measured, never chosen, and the whole point of the pair is
/// that neither can move without a person editing this table and seeing the other one.
const STANDING: &[Standing] = &[
    Standing {
        dir: "vitui",
        citations: 0,
        examples: 1,
    },
    Standing {
        dir: "vitui-engine",
        citations: 876,
        examples: 1,
    },
    Standing {
        dir: "vitui-runtime",
        citations: 312,
        examples: 1,
    },
    Standing {
        dir: "vitui-components",
        citations: 2683,
        examples: 1,
    },
];

/// The crates whose sweep is finished. A finished crate's budget is zero and stays zero, which is
/// the difference between a ratchet and a treadmill.
const SWEPT: &[&str] = &["vitui"];

/// Citation vocabulary that is a plain substring: a decision-record number, a section mark, a
/// register row, a path into the backlog, an obligation letter, a map's own name.
fn plain_needles() -> Vec<String> {
    vec![
        needle("adr", " 0"),
        needle("docs", "/adr"),
        needle(".scr", "atch"),
        needle("register ", "entry"),
        needle("register ", "row"),
        needle("components arch", "itecture"),
        needle("runtime arch", "itecture"),
        needle("engine arch", "itecture"),
        needle("obliga", "tion"),
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
fn measure(dir: &str) -> (usize, usize, Vec<String>) {
    let src = root().join("crates").join(dir).join("src");
    let mut citations = 0;
    let mut examples = 0;
    let mut worst: Vec<String> = Vec::new();
    for file in sources(&src) {
        let text =
            std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
        let short = file
            .strip_prefix(root())
            .unwrap_or(&file)
            .display()
            .to_string();
        for (n, line) in text.lines().enumerate() {
            let Some(doc) = doc_text(line) else { continue };
            if opens_examples(&doc) {
                examples += 1;
            }
            if cites(&doc) {
                citations += 1;
                if worst.len() < 12 {
                    worst.push(format!("{short}:{}", n + 1));
                }
            }
        }
    }
    (citations, examples, worst)
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
        let (citations, _, worst) = measure(standing.dir);
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
            "{dir} is listed as swept and still carries a budget of {}",
            standing.citations
        );
    }
}

/// **The example floor.** Deleting prose satisfies the equality above; it does not satisfy this.
/// Stated as an equality for the same reason the citations are: a floor the crate has already
/// climbed past is a number nobody can read.
#[test]
fn the_examples_a_caller_can_run_never_get_fewer() {
    let mut wrong: Vec<String> = Vec::new();
    for standing in STANDING {
        let (_, examples, _) = measure(standing.dir);
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
