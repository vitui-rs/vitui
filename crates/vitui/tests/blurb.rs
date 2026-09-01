//! **What the four publishable crates say they are, as a gate.**
//!
//! A crate says what it is in three places before a stranger reads a line of its code: the
//! manifest's `description`, which is the crates.io tagline; the opening paragraph of the
//! `README.md`, which is the crates.io front page; and the first line of the crate's rustdoc, which
//! is the docs.rs front page. Four publishable crates, so twelve sentences — and until production
//! ticket 01 nothing in this repository had ever compared one of them against what shipped.
//!
//! The defect that opened the ticket was not an absence. The facade's tagline described this
//! library as having the mechanism ADR 0020 priced at 0.21% across three drivers and deleted, and
//! it had said so since before the crate that would have implemented it was removed. An absent
//! `repository` key at least says nothing; a description is read as a claim.
//!
//! **This is the facade's test and not a crate's own**, because no one of the four can see the
//! other three: `vitui` is the only member that depends on all of them, and reading a sibling's
//! manifest from inside that sibling would gate each crate against itself. It reads files and
//! links against nothing, so it costs the published surface nothing.
//!
//! # The needles are assembled from fragments
//!
//! A scanner looking for a literal contains that literal, and this file is a document about
//! sentences that may not be shipped — written whole, the needles would make it the first thing
//! that fails its own scan. Every one of them is built by [`needle`] from two halves.
//!
//! # What each gate can and cannot see
//!
//! The vocabulary scans are the drift half: they catch a sentence coming *back*, not a sentence
//! going wrong in a new way. The equality is the half with teeth — the description and the README's
//! opening sentence are two files maintained by different hands, and requiring them to agree means
//! correcting one without the other is a build failure rather than a discrepancy nobody reads.
//!
//! The vocabulary in [`unfinished`] is the vocabulary of the drift that was **found**, not a
//! permanent claim that these crates can never be unfinished again. A crate that genuinely stops
//! being implementation-complete changes that list in a ticket that says why, the way a scene is
//! removed only by a ticket naming the property it can no longer distinguish. **Eleven of the
//! sixteen needles are quoted from a sentence this repository actually shipped**; the five that are
//! not are marked `prospective` where they are written, because a needle with no provenance is a
//! guess about how the next drift will be phrased and should be readable as one.
//!
//! # What is scanned, and what is not
//!
//! The twelve sentences, and each publishable crate's status section — the README's and, where the
//! crate has one, the rustdoc's — plus the repository README's, which two of the crate READMEs link
//! to by name as the authority on what shipped. **Not every paragraph of every page.** A false
//! sentence written outside a status section is out of this gate's reach, and the components
//! README's *do not depend on this yet* was exactly that until this ticket gave that page a status
//! section for it to live in.

use std::path::PathBuf;

/// The repository root, from this crate's manifest directory.
fn root() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

/// A needle, from two halves that are meaningless apart.
fn needle(head: &str, tail: &str) -> String {
    format!("{head}{tail}")
}

/// The mechanisms this workspace priced and deleted. A shipped sentence naming one of these as
/// something the library *has* is the defect production ticket 01 was opened by.
fn deleted() -> Vec<String> {
    vec![
        needle("reac", "tive"),
        // Prospective, all three: the tagline used the adjective, and these are the other three
        // ways the same claim comes back.
        needle("reac", "tivity"),
        needle("vitui-", "signals"),
        needle("signal ", "layer"),
    ]
}

/// The vocabulary of a crate under construction, which none of these four is. See the module
/// header on why this list is allowed to change and what changing it costs.
fn unfinished() -> Vec<String> {
    vec![
        needle("under con", "struction"),
        // Prospective: no page here ever said it. Kept because it is the word a half-written crate
        // reaches for and the cost of carrying it is a string.
        needle("scaff", "olding"),
        needle("half ", "built"),
        needle("crate is em", "pty"),
        // The repository README's table said it in a cell rather than in a sentence.
        needle("| em", "pty"),
        needle("do not depend", " on this yet"),
        needle("being built one", " ticket at a time"),
        needle("what exists", " so far"),
    ]
}

/// The four backlogs that are closed. A status section pointing a reader at one of them is telling
/// them to follow work that finished.
fn closed_backlogs() -> Vec<String> {
    vec![
        needle("vitui-engine-", "impl"),
        needle("vitui-runtime-", "impl"),
        needle("vitui-components-", "impl"),
        // Prospective: this one closed without ever being named on a shipped page.
        needle("vitui-engine-", "production"),
    ]
}

/// One publishable crate and the three sentences it opens with.
struct Blurb {
    name: String,
    /// The manifest's `description`.
    description: String,
    /// The `README.md`'s opening paragraph, after the title.
    opening: String,
    /// The first line of the crate's rustdoc.
    first_line: String,
    /// The `README.md`'s `## Status` section, heading included.
    status: Option<String>,
    /// The rustdoc's `# Status` section, where the crate has one.
    doc_status: Option<String>,
}

/// Markdown emphasis and links carry no claim, and a scan that does not strip them is a scan a
/// pair of asterisks defeats: the sentence that opened this ticket's sibling read
/// `is **em` + `pty**`, which no needle for the bare word would ever have matched.
fn plain(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut chars = markdown.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '*' | '`' | '~' => {}
            // `[text](url)` keeps the text and drops the target.
            '[' => {
                for t in chars.by_ref() {
                    if t == ']' {
                        break;
                    }
                    if !matches!(t, '*' | '`' | '~') {
                        out.push(t);
                    }
                }
                if chars.peek() == Some(&'(') {
                    for t in chars.by_ref() {
                        if t == ')' {
                            break;
                        }
                    }
                }
            }
            _ => out.push(c),
        }
    }
    out
}

/// The value between the first pair of quotes on a `key = "value"` line.
fn quoted(line: &str) -> String {
    let mut parts = line.splitn(3, '"');
    parts.next();
    parts.next().unwrap_or_default().to_owned()
}

/// The paragraph after the title: the first run of non-blank lines below the `# ` heading, joined
/// into one line the way a renderer joins it.
fn opening_paragraph(readme: &str) -> String {
    let mut lines = readme.lines().skip_while(|l| !l.starts_with("# "));
    lines.next();
    let para: Vec<&str> = lines
        .skip_while(|l| l.trim().is_empty())
        .take_while(|l| !l.trim().is_empty())
        .collect();
    plain(&para.join(" "))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// A heading and everything under it, up to the next heading at the same level **or shallower**.
///
/// The terminator is the level and not the text, and the difference is the whole section: with the
/// heading itself as the terminator the engine's `# Status` ran to the end of a three-hundred-line
/// module header and a scan over it read `EMPTY`, a constant in a sentence about the mirror, as a
/// crate calling itself unfinished. A gate that reads the wrong region is a gate about a different
/// document.
fn section(text: &str, marker: &str) -> Option<String> {
    let level = marker.chars().take_while(|c| *c == '#').count();
    let mut lines = text.lines().skip_while(|l| !heading(l, marker));
    let head = lines.next()?;
    let body: Vec<&str> = lines.take_while(|l| !closes(l, level)).collect();
    Some(plain(&format!("{head}\n{}", body.join("\n"))))
}

/// A heading line opening exactly this section, in markdown or in rustdoc. `## Status:` counts —
/// the runtime's README put the whole claim in the heading — and `### Status` does not.
fn heading(line: &str, marker: &str) -> bool {
    let line = bare(line);
    line.starts_with(marker) && !line[marker.len()..].starts_with('#')
}

/// A heading at this level or shallower, which is what ends a section at this level.
fn closes(line: &str, level: usize) -> bool {
    let line = bare(line);
    let hashes = line.chars().take_while(|c| *c == '#').count();
    hashes > 0 && hashes <= level && line[hashes..].starts_with(' ')
}

/// A line with its rustdoc marker and indentation taken off.
fn bare(line: &str) -> &str {
    line.trim_start().trim_start_matches("//!").trim_start()
}

/// The workspace members that will reach crates.io: every member whose manifest does not say
/// `publish = false`. Derived rather than listed, so a fifth publishable crate arrives inside this
/// gate instead of beside it.
fn blurbs() -> Vec<Blurb> {
    let root = root();
    let workspace = std::fs::read_to_string(root.join("Cargo.toml")).expect("workspace manifest");
    let members: Vec<String> = workspace
        .lines()
        .skip_while(|l| !l.starts_with("members = ["))
        .skip(1)
        .take_while(|l| !l.starts_with(']'))
        .map(quoted)
        .filter(|m| !m.is_empty())
        .collect();
    assert!(
        members.len() > 4,
        "the members list did not parse: {members:?}"
    );

    let mut out = Vec::new();
    for member in members {
        let dir = root.join(&member);
        let manifest =
            std::fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_else(|_| panic!("{member}"));
        if manifest
            .lines()
            .any(|l| l.starts_with("publish") && l.contains("false"))
        {
            continue;
        }
        let readme = std::fs::read_to_string(dir.join("README.md"))
            .unwrap_or_else(|_| panic!("{member} is publishable and has no README.md"));
        let lib = std::fs::read_to_string(dir.join("src/lib.rs"))
            .unwrap_or_else(|_| panic!("{member} has no src/lib.rs"));
        let description = manifest
            .lines()
            .find(|l| l.starts_with("description = "))
            .map(quoted)
            .unwrap_or_default();
        let first_line = lib
            .lines()
            .find(|l| l.starts_with("//!") && !l.trim_end_matches('!').trim().is_empty())
            .map(|l| plain(l.trim_start_matches("//!").trim()))
            .unwrap_or_default();
        out.push(Blurb {
            name: member.rsplit('/').next().unwrap_or(&member).to_owned(),
            description,
            opening: opening_paragraph(&readme),
            first_line,
            status: section(&readme, "## Status"),
            doc_status: section(&lib, "# Status"),
        });
    }
    out
}

impl Blurb {
    /// The three sentences, each with the place it is read from.
    fn three(&self) -> [(&'static str, &str); 3] {
        [
            ("the crates.io tagline", &self.description),
            ("the README's opening paragraph", &self.opening),
            ("the rustdoc's first line", &self.first_line),
        ]
    }
}

/// Twelve sentences, and every publishable crate carries a status section a reader can find.
#[test]
fn every_publishable_crate_says_what_it_is_in_three_places() {
    let blurbs = blurbs();
    assert_eq!(
        blurbs.len(),
        4,
        "four crates publish: {:?}",
        blurbs.iter().map(|b| &b.name).collect::<Vec<_>>()
    );
    let mut sentences = 0;
    for blurb in &blurbs {
        for (place, text) in blurb.three() {
            assert!(!text.is_empty(), "{} has no {place}", blurb.name);
            sentences += 1;
        }
        assert!(
            blurb.status.is_some(),
            "{}'s README has no `## Status` section, so what it ships is discovered rather than \
             stated",
            blurb.name
        );
    }
    assert_eq!(sentences, 12);
}

/// No shipped sentence names a mechanism this workspace deleted.
#[test]
fn no_blurb_claims_a_mechanism_this_workspace_deleted() {
    for blurb in blurbs() {
        for (place, text) in blurb.three() {
            let lower = text.to_lowercase();
            for banned in deleted() {
                assert!(
                    !lower.contains(&banned),
                    "{}: {place} claims a mechanism this workspace priced and deleted (ADR 0020)",
                    blurb.name
                );
            }
        }
    }
}

/// The tagline and the front page are two files, and they must agree. This is the half of the
/// audit that is not a vocabulary: correcting one without the other fails the build.
#[test]
fn the_manifest_description_opens_the_readme() {
    for blurb in blurbs() {
        let expected = format!("{}.", blurb.description);
        assert!(
            blurb.opening.starts_with(&expected),
            "{}: the README's opening paragraph does not open with the manifest's description.\n  \
             description: {expected}\n  README:      {}",
            blurb.name,
            blurb.opening
        );
    }
}

/// A status section that describes a finished crate as unfinished, or points a reader at a backlog
/// that closed, is the same defect as the tagline: a sentence that was true once.
#[test]
fn no_status_section_describes_a_finished_crate_as_unfinished() {
    let mut sections: Vec<(String, String)> = Vec::new();
    for blurb in blurbs() {
        if let Some(status) = blurb.status {
            sections.push((format!("{}'s README", blurb.name), status));
        }
        if let Some(status) = blurb.doc_status {
            sections.push((format!("{}'s rustdoc", blurb.name), status));
        }
    }
    let repository = std::fs::read_to_string(root().join("README.md")).expect("repository README");
    sections.push((
        "the repository README".to_owned(),
        section(&repository, "## Status").expect("the repository README has a `## Status` section"),
    ));
    assert!(
        sections.len() >= 7,
        "four READMEs, the pages that carry a rustdoc status, and the repository README: {:?}",
        sections.iter().map(|(w, _)| w).collect::<Vec<_>>()
    );

    for (where_, text) in sections {
        let lower = text.to_lowercase();
        for banned in unfinished() {
            assert!(
                !lower.contains(&banned),
                "{where_} describes a crate that is implementation-complete as unfinished"
            );
        }
        for banned in closed_backlogs() {
            assert!(
                !lower.contains(&banned),
                "{where_} points a reader at a backlog that is closed"
            );
        }
    }
}
