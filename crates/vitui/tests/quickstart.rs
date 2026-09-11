//! **The README's quickstart is an application that compiles, and this is what makes that true.**
//!
//! Production ticket 24 asks the first screen for a runnable example and then says the thing that
//! matters about it: it must be *compiled by something*, because a README code block is the one
//! kind of Rust in a repository that nothing builds. A second, unchecked copy of an application is
//! a copy that goes wrong at the first rename and stays wrong until a stranger tries it.
//!
//! So the block is not a copy. It is `crates/vitui-apps/examples/counter.rs` with its comments
//! removed, and `cargo test` is where the two are compared. The example itself is a target in this
//! workspace, so the code in the README is compiled on every build of `vitui-apps` — by being the
//! same text rather than by being quoted.
//!
//! **This is the arrangement `blurb.rs` uses and for its reason**: two things derived from each
//! other, with an equality between them, hold for ever. The alternative — a doc-test carrying a
//! third copy — would compile something, but not the thing on the page.
//!
//! The example is a port of ratatui's counter tutorial, which is why it is the one quoted: its
//! shape is not ours to argue with, so whatever it cannot express here is a fact about this surface
//! rather than a taste.

use std::path::PathBuf;

/// The repository root, from this crate's manifest directory.
fn root() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

/// The example, with every comment line taken out and the runs of blank lines that leaves collapsed
/// to one.
///
/// Comment lines and not trailing comments: a `//` inside a string literal is not a comment, and a
/// stripper that looked anywhere but the start of a line would have to know that. Nothing in the
/// example carries a trailing comment, and a gate that would break if one arrived is a gate that
/// says so here rather than being clever.
fn without_comments(source: &str) -> String {
    let kept: Vec<&str> = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .map(|line| line.trim_end())
        .collect();

    let mut out = String::with_capacity(source.len());
    let mut blank = false;
    for line in kept {
        if line.is_empty() {
            if !blank && !out.is_empty() {
                out.push('\n');
            }
            blank = true;
        } else {
            out.push_str(line);
            out.push('\n');
            blank = false;
        }
    }
    out.trim_end().to_owned() + "\n"
}

/// The first fenced `rust` block in the README, fence lines excluded.
fn quickstart(readme: &str) -> String {
    let mut lines = readme.lines().skip_while(|l| l.trim_end() != "```rust");
    lines.next().expect("the README has no ```rust block");
    let body: Vec<&str> = lines.take_while(|l| l.trim_end() != "```").collect();
    assert!(!body.is_empty(), "the README's rust block is empty");
    body.join("\n") + "\n"
}

/// The README's quickstart is the `counter` example and not a paraphrase of it.
///
/// Failing here means one of two things and the diff says which: the example moved and the page did
/// not, or the page was edited directly. Either way the fix is to regenerate the block from the
/// file rather than to reconcile them by hand.
#[test]
fn the_readme_quickstart_is_the_counter_example_with_its_comments_removed() {
    let readme = std::fs::read_to_string(root().join("README.md")).expect("repository README");
    let example = std::fs::read_to_string(root().join("crates/vitui-apps/examples/counter.rs"))
        .expect("the counter example");

    let shown = quickstart(&readme);
    let compiled = without_comments(&example);

    assert_eq!(
        shown, compiled,
        "the README's quickstart has drifted from crates/vitui-apps/examples/counter.rs — \
         regenerate the block from the file"
    );
}

/// The block is the whole application, which is what makes *runnable* a claim rather than a word.
#[test]
fn the_quickstart_is_a_whole_program() {
    let readme = std::fs::read_to_string(root().join("README.md")).expect("repository README");
    let shown = quickstart(&readme);
    for needed in ["fn main()", "use vitui_components::", "use vitui_runtime::"] {
        assert!(
            shown.contains(needed),
            "the README's quickstart has no `{needed}`, so it is an excerpt and not a program"
        );
    }
}
