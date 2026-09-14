//! The freeze, as JSON, for a reader who is not a compiler.
//!
//! **The gallery page of the documentation site has to be joined to something, and the only
//! defensible something is [`INVENTORY`].** A hand-written list of twenty-nine is a list that loses
//! one: a component added to the freeze would arrive owing a picture, an example, a golden screen
//! and an application — all four already gated — and a page nobody joined would be the one place it
//! could be missing without anything going red. So the page's population is this value, and this
//! example is the only thing standing between a `const` in a Rust crate and a static-site generator
//! that cannot read one.
//!
//! **It derives, it does not describe.** Every field below is read out of the crate: the columns
//! from the freeze, the spellings a caller writes from the module the freeze homes each row in, and
//! the one-line summary from the component's **own documentation** — the first paragraph of the
//! rustdoc on its entry point, which production ticket 20 gated as *a page opens by saying what the
//! item does*. Nothing here is a second description written for a website, because a second
//! description is one that disagrees with the first within a month.
//!
//! ```text
//! cargo run -p vitui-components --example inventory_json > site/src/generated/inventory.json
//! ```
//!
//! The site's `npm run generate` is that line, and its CI job runs it again and fails on a
//! difference — the arrangement every golden screen in this workspace already uses.

use std::path::PathBuf;

use vitui_components::consumer::spellings;
use vitui_components::doc::doc_comment;
use vitui_components::inventory::{Axis, INVENTORY};

/// The repository root, from this crate's manifest directory.
fn root() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
}

/// A JSON string literal. Six escapes and the control range, which is the whole of the grammar
/// this has to satisfy — no dependency, because the crate takes none and an example is not where
/// that starts.
fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The first paragraph of a doc comment, as one line.
///
/// A summary and not a page: the rest of a component's documentation is what docs.rs is for. A
/// heading or a fence ends it, because a page whose first paragraph is a heading has no summary to
/// take and a wrong one is worse than none.
fn summary(doc: &str) -> Option<String> {
    let mut lines = Vec::new();
    for line in doc.lines() {
        let t = line.trim();
        if t.is_empty() {
            if !lines.is_empty() {
                break;
            }
            continue;
        }
        if t.starts_with('#') || t.starts_with("```") || t.starts_with('>') || t.starts_with('|') {
            break;
        }
        lines.push(t);
    }
    if lines.is_empty() {
        return None;
    }
    Some(lines.join(" "))
}

fn main() {
    let root = root();
    let mut rows = Vec::new();

    for c in INVENTORY {
        let module = c
            .module()
            .expect("every row of the freeze is homed in a module");
        let file = format!("crates/vitui-components/src/{module}.rs");
        let source = std::fs::read_to_string(root.join(&file)).unwrap_or_default();

        let spelled = spellings(&source, c.id);
        let summary = doc_comment(&source, c.id).as_deref().and_then(summary);
        let axes: Vec<&str> = Axis::ALL
            .into_iter()
            .filter(|a| c.declares(*a))
            .map(|a| match a {
                Axis::Scrolled => "scrolled",
                Axis::Shrunk => "shrunk",
                Axis::Wheeled => "wheeled",
                Axis::Narrow => "narrow",
            })
            .collect();

        let fields = [
            format!("\"id\": {}", quote(c.id)),
            format!("\"tier\": {}", c.tier as u8 + 1),
            format!("\"built\": {}", c.built),
            format!("\"layer\": {}", c.layer.rung()),
            format!("\"module\": {}", quote(module)),
            format!("\"file\": {}", quote(&file)),
            format!(
                "\"families\": [{}]",
                c.families
                    .iter()
                    .map(|f| quote(f.name()))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            format!(
                "\"spellings\": [{}]",
                spelled
                    .iter()
                    .map(|s| quote(s))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            format!("\"glyphs\": {}", c.glyphs.len()),
            format!("\"constructions\": {}", c.constructions),
            format!(
                "\"axes\": [{}]",
                axes.iter().map(|a| quote(a)).collect::<Vec<_>>().join(", ")
            ),
            format!(
                "\"summary\": {}",
                summary.as_deref().map_or("null".to_owned(), quote)
            ),
        ];
        rows.push(format!("    {{ {} }}", fields.join(", ")));
    }

    println!("[\n{}\n]", rows.join(",\n"));
}
