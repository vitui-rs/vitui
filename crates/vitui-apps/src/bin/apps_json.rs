//! The application list, as JSON, for the documentation site's showcase page.
//!
//! [`APPS`] is already the authority — `src/lib.rs` holds one row per file and three tests read it,
//! the rows and the directory being the same set in **both** directions. A showcase page written by
//! hand would be a fourth list nothing joins, and the first application added after it was written
//! would be the one missing from the site.
//!
//! **A binary and not an example, for a reason the gate names**: `examples/` and [`APPS`] must be
//! the same set, so an example that is not an application is a red test. This is a generator, not a
//! demonstration, and it belongs on the other side of that line.
//!
//! ```text
//! cargo run -p vitui-apps --bin apps_json > site/src/generated/apps.json
//! ```

use vitui_apps::APPS;

/// A JSON string literal, on `inventory_json`'s terms and for its reason: the crate takes no
/// dependency and a generator is not where that changes.
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

fn main() {
    let rows: Vec<String> = APPS
        .iter()
        .map(|a| {
            let uses = a
                .uses
                .iter()
                .map(|u| quote(u))
                .collect::<Vec<_>>()
                .join(", ");
            let after = a.after.map_or("null".to_owned(), quote);
            format!(
                "    {{ \"name\": {}, \"what\": {}, \"uses\": [{uses}], \"after\": {after} }}",
                quote(a.name),
                quote(a.what),
            )
        })
        .collect();

    println!("[\n{}\n]", rows.join(",\n"));
}
