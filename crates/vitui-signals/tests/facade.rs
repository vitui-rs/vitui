//! **The crate may exist and be published, and nothing in this workspace may depend on it.**
//!
//! Runtime ticket 18's second acceptance criterion, in both directions. `cargo deny` holds the line
//! for a consumer's dependency graph; this file holds it for *this* repository, where `cargo deny`
//! would not fire until somebody had already written the line and pushed it.
//!
//! # Why the facade does not re-export this crate
//!
//! `vitui` re-exports the engine, the runtime and the components. It does **not** re-export signals,
//! and that is a decision rather than an omission: a facade that re-exported one of two reactivity
//! shapes would have picked a winner between two that measured identical — **0 of 24 000 differing
//! cells**, inside the same noise floor, over the same screen (`tests/drivers.rs`). An application
//! that wants this shape adds one line to its own `Cargo.toml`, which is the visibility the choice
//! deserves.
//!
//! # What is checked, and why a text scan rather than a graph walk
//!
//! Nothing here can ask cargo for the workspace graph — `cargo metadata` is a subprocess and a
//! `cargo test` that shells out to cargo is a gate that fails on somebody's `PATH`. So the manifests
//! are read as text, which is what `vitui-runtime/src/line.rs` does one crate down and for the same
//! reason. The scan is deliberately blunt: **any mention of the name outside a comment, in any
//! dependency table, in any manifest in this workspace, is a failure.**

use std::fs;
use std::path::{Path, PathBuf};

/// The repository root, from this package's manifest directory.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crates/vitui-signals sits two levels below the root")
        .to_path_buf()
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

/// Every `Cargo.toml` of a workspace member, plus the root's.
///
/// The detached workspaces — `fuzz/`, `compare/`, `conform/`, `examples/app-template` — are outside
/// this one by construction and have their own graphs; they are not members and cargo never resolves
/// them here. This gate is about the workspace `deny.toml` covers.
fn manifests() -> Vec<(String, String)> {
    let root = workspace_root();
    let mut out = vec![("Cargo.toml".to_string(), read(&root.join("Cargo.toml")))];
    let crates = root.join("crates");
    let mut names: Vec<_> = fs::read_dir(&crates)
        .expect("crates/ is readable")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| crates.join(n).join("Cargo.toml").is_file())
        .collect();
    names.sort();
    for name in names {
        let path = crates.join(&name).join("Cargo.toml");
        out.push((format!("crates/{name}/Cargo.toml"), read(&path)));
    }
    out
}

/// Lines of a manifest that are not blank and not a comment.
fn code_lines(manifest: &str) -> impl Iterator<Item = &str> {
    manifest
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
}

/// **Direction one: nothing in this workspace depends on `vitui-signals`.**
///
/// Including the facade, including the components crate, including the root's
/// `[workspace.dependencies]` table — whose *absence* of an entry is half the gate, because an entry
/// there exists for no other purpose than to let a member write `.workspace = true`.
#[test]
fn nothing_in_this_workspace_depends_on_vitui_signals() {
    // The two lines in this workspace that are allowed to say the name, and nothing else is. The
    // root's is the `exclude` entry — see `the_crate_is_detached_and_the_ban_is_what_detached_it` —
    // and the crate's own is `[package] name`. A `[workspace.dependencies]` entry would read
    // `vitui-signals = { path = … }` and a member's dependency the same, so both are caught.
    let permitted: &[(&str, &str)] = &[
        (
            "Cargo.toml",
            r#"exclude = ["examples/app-template", "crates/vitui-signals"]"#,
        ),
        (
            "crates/vitui-signals/Cargo.toml",
            r#"name = "vitui-signals""#,
        ),
    ];

    for (path, manifest) in manifests() {
        let allowed = permitted
            .iter()
            .find(|(p, _)| *p == path)
            .map_or("", |(_, line)| *line);
        for line in code_lines(&manifest) {
            if !allowed.is_empty() && line == allowed {
                continue;
            }
            assert!(
                !line.contains("vitui-signals") && !line.contains("vitui_signals"),
                "{path} names vitui-signals outside its own [package] table: {line}"
            );
        }
    }
}

/// **Direction two: the facade re-exports three layers, and signals is not one of them.**
#[test]
fn the_facade_re_exports_three_layers_and_signals_is_not_one() {
    let src = read(&workspace_root().join("crates/vitui/src/lib.rs"));
    for layer in ["vitui_engine", "vitui_runtime", "vitui_components"] {
        assert!(
            src.contains(&format!("pub use {layer} as")),
            "the facade stopped re-exporting {layer}"
        );
    }
    assert!(
        !src.contains("vitui_signals"),
        "the facade re-exports vitui-signals, which is the decision this gate exists to hold"
    );
    // Three `pub use` lines and not four.
    let exports = src
        .lines()
        .filter(|l| l.trim_start().starts_with("pub use "))
        .count();
    assert_eq!(exports, 3, "the facade's re-export count moved");
}

/// **`deny.toml` carries the rule, with an empty `wrappers` list.**
///
/// `wrappers` says *who may depend on it*; empty means nobody. The runtime's own
/// `line::the_dependency_line_is_a_gate_and_the_allowlist_is_refused` asserts the same string from
/// the other side of the workspace, and both are wanted: that one is a crate that cannot see this
/// one, and this one is the crate the rule is about.
#[test]
fn deny_toml_says_the_wrappers_list_is_empty() {
    let deny = read(&workspace_root().join("deny.toml"));
    assert!(
        deny.contains(r#"{ name = "vitui-signals", wrappers = [] }"#),
        "deny.toml no longer bans vitui-signals with an empty wrappers list"
    );
}

/// **The crate is detached from the workspace, and the ban is what detached it.**
///
/// This is the ticket's one structural finding, and it is checked rather than only written down.
/// `cargo deny`'s `[bans] deny` bans a crate's **presence in the graph**; `wrappers` is the exception
/// list, so an empty one means *never present*. A workspace member with nothing depending on it is
/// still present, and the first `cargo deny check` after this crate was added read
/// `error[banned]: crate 'vitui-signals = 0.0.0' is explicitly banned` with no dependent to name.
///
/// So *the crate may exist and be published, and nothing in this workspace may depend on it* is only
/// satisfiable by a crate that is not in the workspace. Detaching it also makes the ban **live**: it
/// now fires the day a member writes the dependency, naming the wrapper, where before it was already
/// failing for the crate merely existing.
///
/// The cost is that `cargo test --workspace` does not reach this file, which is why there is a
/// `signals` job — a crate nobody builds is a crate nobody checks, and the job is the other half of
/// this decision rather than a chore beside it.
#[test]
fn the_crate_is_detached_and_the_ban_is_what_detached_it() {
    let root = read(&workspace_root().join("Cargo.toml"));
    let members: Vec<&str> = root
        .lines()
        .skip_while(|l| l.trim() != "members = [")
        .skip(1)
        .take_while(|l| l.trim() != "]")
        .map(str::trim)
        .collect();
    assert!(
        !members.contains(&r#""crates/vitui-signals","#),
        "vitui-signals is a workspace member again, which fails `cargo deny check` outright"
    );
    assert!(
        root.contains(r#"exclude = ["examples/app-template", "crates/vitui-signals"]"#),
        "the root workspace no longer excludes this crate"
    );

    // The empty table is what detaches it; without it cargo assumes a manifest inside a workspace
    // directory is a member and refuses to build at all.
    let own = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"));
    assert!(
        code_lines(&own).any(|l| l == "[workspace]"),
        "the detaching table is gone"
    );

    // And the job that runs this file, because the exclusion took it out of `--workspace`.
    let ci = read(&workspace_root().join(".gitlab-ci.yml"));
    assert!(
        ci.contains("crates/vitui-signals"),
        "no CI job builds this crate, so none of these gates run"
    );
}

/// **The shipped dependency list is `vitui-runtime` and nothing else.**
///
/// Dev-dependencies are the two instruments the rest of the workspace uses for the same two jobs —
/// `vitui-bench` for a report and `vitui-alloc-probe` for a count — and neither can name an engine
/// type, so neither reopens the refusal. What is checked here is the shipped table: **no
/// `vitui-engine` and no `vitui` anywhere in this manifest**, which is what makes `tests/drivers.rs`
/// honest about why its equality is a ledger rather than a read-back.
#[test]
fn this_crate_depends_on_the_runtime_and_cannot_name_the_engine() {
    let manifest = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"));
    let deps: Vec<&str> = code_lines(&manifest)
        .skip_while(|l| *l != "[dependencies]")
        .skip(1)
        .take_while(|l| !l.starts_with('['))
        .collect();
    assert_eq!(
        deps,
        vec![r#"vitui-runtime = { path = "../vitui-runtime", version = "0.0.0" }"#]
    );

    for line in code_lines(&manifest) {
        assert!(
            !line.contains("vitui-engine") && !line.contains("vitui_engine"),
            "the engine appeared in this crate's manifest: {line}"
        );
        assert!(
            !line.starts_with("vitui.") && !line.starts_with("vitui ="),
            "the facade appeared in this crate's manifest: {line}"
        );
    }
}

/// **The crate is about a hundred and twenty lines, and that is the ticket's whole argument.**
///
/// > Two of its three parts are already in `vitui-runtime`.
///
/// Measured at **112** lines that are neither blank nor a comment, at the commit that shipped it.
/// The gate is `< 200` and not `== 112`, and the reason is this backlog's rule for a number that
/// belongs to the data rather than to the mechanism: **a gate on an exact line count is a gate that
/// is edited rather than fixed**, and it would be edited on the first doc comment anybody rewrites.
/// What the headroom still refuses is the thing worth refusing — a signal library that grew a
/// scheduler, a dependency tracker or an effect queue, none of which fit in ninety spare lines.
///
/// Doc comments are not counted and are the majority of the file, which is the shape this crate is
/// supposed to have: the mechanism is small and the reasons it is small are not.
#[test]
fn the_crate_is_about_a_hundred_and_twenty_lines_of_code() {
    let src = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs"));
    let code = src
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .count();
    assert!(
        code < 200,
        "vitui-signals is {code} lines of code, which is no longer a wrapper over the runtime"
    );
}
