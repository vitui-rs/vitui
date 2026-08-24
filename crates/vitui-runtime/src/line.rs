//! **The crate line, as a value** — the module map, the visibility count, and the four manifest
//! decisions that had been comments.
//!
//! Runtime ticket 17. Three of that ticket's six acceptance criteria are statements about files
//! rather than about behaviour — the module map ships as written, `deny.toml` states the dependency
//! line, `[bans] allow` is not used, `codegen-units = 1` is gone — and all three were **true and
//! unchecked** when this file was written. That is the same shape as the `crossterm` rule ADR 0001
//! restated four times before runtime ticket 14 made it a `cargo deny` gate: a decision written in a
//! comment is a decision the next edit can undo silently.
//!
//! `#[cfg(test)]`, like `crate::screen` and `crate::keys::corpus` and for the engine's reason
//! (`crates/vitui-engine/src/audit.rs`): **an instrument is not part of the library**, and nothing
//! here is reachable from a component.
//!
//! # What the component-facing surface is, precisely
//!
//! The ticket asks for **0 `pub(crate)`, `pub(super)` or `pub(in …)` items on the component-facing
//! surface**, and the surface has to be defined before that number means anything: there are
//! seventy-eight such declarations inside this crate and every one of them is machinery a component
//! has no business seeing.
//!
//! > The **component-facing surface** is what a crate that depends on `vitui-runtime` and on nothing
//! > else can name: the crate root's re-export list, the fourteen modules it declares `pub mod`, and
//! > the `pub` items inside them.
//!
//! `crates/vitui-components/tests/crate_line.rs` is that crate — components spec §0's constraint C6
//! is what makes it one — and it is where the *build* behind the count is. Two things are gated here
//! that the build cannot see:
//!
//! 1. **No restricted item is in the crate root.** A `pub(crate)` in `lib.rs` is the leak class the
//!    count was about: the root is the seam itself, so an item there is on the surface in the only
//!    sense that matters — a reader of the public API meets it.
//! 2. **No restricted item is re-exported.** `pub use crate::focus::Ring;` is `error[E0365]` and so
//!    cannot happen; a `pub use` of an item that *becomes* restricted is the same error arriving a
//!    commit later, and this is the direction a scan sees before the compiler does.
//!
//! # The row the map does not have
//!
//! The shipped crate declares **fifteen** modules where spec §4's map has fourteen rows plus a
//! `debug` row that is fog: `route` is ticket 11's, spec §7 and ADR 0016 specify it, and §4's table
//! never gained a line for it. [`MODULES`] carries it with `Origin::Added` beside it rather than
//! quietly matching a shorter list, and the discrepancy is filed as architecture issue 21 — **a
//! module is not deleted to make a table come out even.**

/// Where a module row came from.
///
/// Two arms and no third, which is `crate::screen`'s reason for asserting its own shape: a module
/// that is neither in spec §4's map nor attributable to a ticket is drift, and drift with a name for
/// itself stops being visible.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Origin {
    /// A row of spec §4's module map.
    Spec4,
    /// Not in §4's map. The implementation ticket that shipped it, and why.
    Added {
        /// The implementation ticket, as `R NN`.
        by: &'static str,
        /// Why it is there, in the terms the ticket used.
        why: &'static str,
    },
}

/// One row of the module map.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Module {
    /// The module's path, as `lib.rs` declares it.
    pub name: &'static str,
    /// The spec section it answers.
    pub section: &'static str,
    /// Where the row came from.
    pub origin: Origin,
}

/// The module map as shipped: spec §4's fourteen rows and the one it does not have.
///
/// `layout::text`, `theme::registry` and `keys::corpus` are rows of §4's table or of a ticket rather
/// than of `lib.rs` — the crate root declares their parents — so the gate over this value is against
/// the **top-level** names, which is [`top_level`].
pub const MODULES: [Module; 15] = [
    Module {
        name: "anim",
        section: "§16",
        origin: Origin::Spec4,
    },
    Module {
        name: "ctx",
        section: "§1 §3 §6",
        origin: Origin::Spec4,
    },
    Module {
        name: "data",
        section: "§14",
        origin: Origin::Spec4,
    },
    Module {
        name: "focus",
        section: "§8",
        origin: Origin::Spec4,
    },
    Module {
        name: "id",
        section: "§5",
        origin: Origin::Spec4,
    },
    Module {
        name: "keys",
        section: "§9",
        origin: Origin::Spec4,
    },
    Module {
        name: "layout",
        section: "§11",
        origin: Origin::Spec4,
    },
    Module {
        name: "overlay",
        section: "§10",
        origin: Origin::Spec4,
    },
    Module {
        name: "route",
        section: "§7",
        origin: Origin::Added {
            by: "R11",
            why: "spec §7 and ADR 0016 specify the one key queue and the routing edge; §4's table \
                  never gained a row for it",
        },
    },
    Module {
        name: "scroll",
        section: "§13",
        origin: Origin::Spec4,
    },
    Module {
        name: "sizing",
        section: "§12",
        origin: Origin::Spec4,
    },
    Module {
        name: "theme",
        section: "§15",
        origin: Origin::Spec4,
    },
    Module {
        name: "work",
        section: "§17",
        origin: Origin::Spec4,
    },
    // The two rows of §4's map that are not top-level modules, kept here so the table is the whole
    // table and the gate below can say which rows it is not checking against `lib.rs`.
    Module {
        name: "layout::text",
        section: "§11",
        origin: Origin::Spec4,
    },
    Module {
        name: "theme::registry",
        section: "§15",
        origin: Origin::Spec4,
    },
];

/// The names the ticket says must not exist, each because the map corrected the proposal that had
/// them. Spec §4's three corrections plus the one deferral.
///
/// - `input` — there are no per-id inboxes, so it is not a module of its own (ADR 0016).
/// - `hit` — `ctx`'s.
/// - `drag` — deferred to v2.
/// - `Rows` — does not exist; `data` has no trait.
pub const REFUSED: [&str; 4] = ["input", "hit", "drag", "Rows"];

/// An engine name the runtime's **public** surface names, and how a component-facing crate reaches
/// it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EngineName {
    /// The engine's name for it.
    pub name: &'static str,
    /// The path a crate that cannot depend on `vitui-engine` writes, or `None`.
    pub reachable_as: Option<&'static str>,
}

/// Every engine name the runtime's **public** surface names, with the component-facing path or the
/// absence of one.
///
/// **This is ticket 17's finding, as a value.** Components spec §0 states constraint C6 — the
/// components crate depends on `vitui-runtime` and on nothing else — and components spec §1 writes
/// `pub fn button(cx: &mut Ctx, area: Rect, …)`. Both cannot hold while `Rect` is reachable only
/// through `vitui-engine`: a consumer can *hold* one, because `Ctx::area` hands it over and
/// inference carries it, and cannot *write its type*.
///
/// **Runtime spec §4 says three of these are re-exported — `GlyphSet`, `Slot` and `CursorShape` —
/// and `CursorShape` is not.** `Ctx::caret_with` takes one and no path leads to it. Filed as
/// architecture issue 22; not decided here.
///
/// `Style` is on the list because `pub struct Paint(pub(crate) Style)` names it in a public
/// declaration, and it is the one entry nothing is blocked by: the field is restricted, which is the
/// whole design — a component names a role and can never construct a paint.
pub const ENGINE_NAMES: [EngineName; 24] = [
    // The seven with a path. Four of them are one `pub use` in `keys`.
    EngineName {
        name: "GlyphSet",
        reachable_as: Some("vitui_runtime::theme::GlyphSet"),
    },
    EngineName {
        name: "Link",
        reachable_as: Some("vitui_runtime::theme::Link"),
    },
    EngineName {
        name: "Slot",
        reachable_as: Some("vitui_runtime::work::Slot"),
    },
    EngineName {
        name: "Key",
        reachable_as: Some("vitui_runtime::keys::Pressed"),
    },
    EngineName {
        name: "KeyCode",
        reachable_as: Some("vitui_runtime::keys::Code"),
    },
    EngineName {
        name: "KeyKind",
        reachable_as: Some("vitui_runtime::keys::Edge"),
    },
    EngineName {
        name: "KeyText",
        reachable_as: Some("vitui_runtime::keys::Text"),
    },
    // The seventeen with none. Each is named by a public signature; the ones that block a whole
    // family of gates are `Rect`, `Mods`, `Event` and `Mouse` — see this module's documentation and
    // `crates/vitui-components/tests/crate_line.rs`'s table.
    EngineName {
        name: "Rect",
        reachable_as: None,
    },
    EngineName {
        name: "Restyle",
        reachable_as: None,
    },
    EngineName {
        name: "Written",
        reachable_as: None,
    },
    EngineName {
        name: "ColorDepth",
        reachable_as: None,
    },
    EngineName {
        name: "Event",
        reachable_as: None,
    },
    EngineName {
        name: "Mods",
        reachable_as: None,
    },
    EngineName {
        name: "Mouse",
        reachable_as: None,
    },
    EngineName {
        name: "MouseMode",
        reachable_as: None,
    },
    EngineName {
        name: "Capabilities",
        reachable_as: None,
    },
    EngineName {
        name: "Cursor",
        reachable_as: None,
    },
    EngineName {
        name: "CursorShape",
        reachable_as: None,
    },
    EngineName {
        name: "Config",
        reachable_as: None,
    },
    EngineName {
        name: "AttachError",
        reachable_as: None,
    },
    EngineName {
        name: "Presented",
        reachable_as: None,
    },
    EngineName {
        name: "WakeHandle",
        reachable_as: None,
    },
    EngineName {
        name: "Rgb",
        reachable_as: None,
    },
    EngineName {
        name: "Style",
        reachable_as: None,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

    fn src_dir() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"))
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    fn read(path: &Path) -> String {
        std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{} reads: {e}", path.display()))
    }

    fn root() -> String {
        read(&src_dir().join("lib.rs"))
    }

    /// Every `.rs` file under `src/`, relative to it. **Recursive**, because `keys/`, `layout/` and
    /// `theme/` all hold code and a flat walk would be blind to `theme/wire.rs` — which holds four
    /// of the seventy-eight restricted declarations.
    fn modules() -> Vec<String> {
        fn walk(dir: &Path, prefix: &str, out: &mut Vec<String>) {
            for entry in std::fs::read_dir(dir).expect("a readable source directory") {
                let path = entry.expect("a directory entry reads").path();
                let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
                    continue;
                };
                let relative = match prefix.is_empty() {
                    true => name.to_string(),
                    false => format!("{prefix}/{name}"),
                };
                if path.is_dir() {
                    walk(&path, &relative, out);
                } else if name.ends_with(".rs") {
                    out.push(relative);
                }
            }
        }
        let mut out = Vec::new();
        walk(&src_dir(), "", &mut out);
        out.sort();
        assert!(
            out.iter().any(|m| m.contains('/')),
            "the walk found nothing in a subdirectory, so it is not recursing"
        );
        out
    }

    /// The top-level `pub mod` names the crate root declares.
    ///
    /// **`pub mod` only.** `#[cfg(test)] pub mod corpus` and the `#[path]`-included `screen` are
    /// instruments rather than rows of the map, and `mod screen;` is not `pub` — see the module
    /// documentation.
    fn top_level() -> Vec<String> {
        root()
            .lines()
            .filter_map(|l| l.strip_prefix("pub mod "))
            .map(|rest| rest.trim_end_matches(';').to_string())
            .collect()
    }

    /// Every restricted-visibility **declaration**, as `(module, the declaration's line)`.
    ///
    /// Line-anchored on the visibility keyword, so a `pub(crate)` inside a doc comment or a string
    /// is not a declaration — the difference is 90 textual occurrences against 78 items, and the
    /// twelve are prose about the mechanism.
    fn restricted() -> Vec<(String, String)> {
        let mut out = Vec::new();
        for module in modules() {
            for line in read(&src_dir().join(&module)).lines() {
                let s = line.trim_start();
                if s.starts_with("pub(crate)")
                    || s.starts_with("pub(super)")
                    || s.starts_with("pub(in ")
                {
                    out.push((module.clone(), s.to_string()));
                }
            }
        }
        assert!(
            out.len() > 50,
            "only {} restricted declarations were found, which is too few to be this crate",
            out.len()
        );
        out
    }

    /// The leaf names of every `pub use` in the crate, wherever it is written.
    fn re_exported_names() -> BTreeSet<String> {
        let mut names = BTreeSet::new();
        for module in modules() {
            let text = read(&src_dir().join(&module));
            let mut held: Option<String> = None;
            for line in text.lines() {
                let statement = match held.take() {
                    Some(mut open) => {
                        open.push(' ');
                        open.push_str(line.trim());
                        open
                    }
                    None if line.trim_start().starts_with("pub use ") => line.trim().to_string(),
                    None => continue,
                };
                if !(statement.ends_with(';') || statement.ends_with('}')) {
                    held = Some(statement);
                    continue;
                }
                let body = statement
                    .trim_start_matches("pub use ")
                    .trim_end_matches(';')
                    .to_string();
                match body.find('{') {
                    Some(open) => {
                        let close = body.rfind('}').expect("a braced re-export closes");
                        for name in body[open + 1..close].split(',') {
                            let name = name.trim();
                            if !name.is_empty() {
                                names.insert(
                                    name.rsplit(" as ")
                                        .next()
                                        .expect("a name")
                                        .trim()
                                        .to_string(),
                                );
                            }
                        }
                    }
                    None => {
                        let last = body.rsplit("::").next().expect("a path has a last segment");
                        names.insert(
                            last.rsplit(" as ")
                                .next()
                                .expect("a name")
                                .trim()
                                .to_string(),
                        );
                    }
                }
            }
            assert!(held.is_none(), "a `pub use` in {module} never ended");
        }
        assert!(
            names.contains("Slot"),
            "the parse missed `work`'s re-export of the engine's Slot"
        );
        names
    }

    // ── the module map ───────────────────────────────────────────────────────────────────────────

    /// **The map ships as written.** Spec §4's rows, in both directions: a module the crate root
    /// declares and [`MODULES`] does not have is drift, and a row here that the root does not
    /// declare is a table describing a crate that does not exist.
    #[test]
    fn the_crate_root_declares_the_module_map() {
        let declared: BTreeSet<String> = top_level().into_iter().collect();
        let mapped: BTreeSet<String> = MODULES
            .iter()
            .filter(|m| !m.name.contains("::"))
            .map(|m| m.name.to_string())
            .collect();
        assert_eq!(declared, mapped);
        assert_eq!(declared.len(), 13);
    }

    /// **The two nested rows are inside the modules that own them**, which is what makes them rows
    /// rather than crates: `layout::text` over ADR 0005's exports, `theme::registry` over the
    /// shipped set.
    #[test]
    fn the_nested_rows_are_declared_by_their_parents() {
        for row in MODULES.iter().filter(|m| m.name.contains("::")) {
            let (parent, child) = row
                .name
                .split_once("::")
                .expect("a nested row has a parent");
            let text = read(&src_dir().join(format!("{parent}.rs")));
            assert!(
                text.lines()
                    .any(|l| l.trim() == format!("pub mod {child};")),
                "`{}` is a row of the map and `{parent}.rs` does not declare it",
                row.name
            );
        }
    }

    /// **`input`, `hit`, `drag` and `Rows` do not exist.** The four names spec §4 corrected out of
    /// `architecture.md`'s proposal, checked as an absence rather than remembered as one.
    #[test]
    fn the_four_refused_names_are_not_declared_anywhere() {
        for module in modules() {
            let text = read(&src_dir().join(&module));
            for line in text.lines() {
                let s = line.trim_start();
                for refused in REFUSED {
                    for shape in [
                        format!("mod {refused};"),
                        format!("pub mod {refused};"),
                        format!("struct {refused}"),
                        format!("enum {refused}"),
                        format!("trait {refused}"),
                    ] {
                        assert!(
                            !s.starts_with(&shape),
                            "`{refused}` is declared in {module}: {s}"
                        );
                    }
                }
            }
        }
    }

    // ── the visibility count ─────────────────────────────────────────────────────────────────────

    /// **0 restricted items in the crate root.** The root *is* the seam, so an item there is on the
    /// component-facing surface in the one sense that matters: a reader of the public API meets it.
    #[test]
    fn no_restricted_item_is_in_the_crate_root() {
        let in_root: Vec<(String, String)> = restricted()
            .into_iter()
            .filter(|(m, _)| m == "lib.rs")
            .collect();
        assert_eq!(in_root.len(), 0, "{in_root:?}");
    }

    /// **0 restricted items are re-exported.** `pub use` of a less-visible item is `error[E0365]`
    /// today; the scan is what sees the *next* one, where an item becomes restricted under a
    /// re-export that was legal when it was written.
    #[test]
    fn no_restricted_item_is_re_exported() {
        let names = re_exported_names();
        let mut leaked = Vec::new();
        for (module, declaration) in restricted() {
            let after = declaration
                .split_once(')')
                .map(|(_, rest)| rest)
                .unwrap_or_default();
            let Some(name) = after.split_whitespace().find(|w| {
                !matches!(
                    *w,
                    "fn" | "const" | "struct" | "enum" | "type" | "static" | "mod" | "unsafe"
                )
            }) else {
                continue;
            };
            let name = name.trim_end_matches([':', '(', '<', '{', ';', ','].as_ref());
            if !name.is_empty() && names.contains(name) {
                leaked.push(format!("{module}: {name}"));
            }
        }
        assert_eq!(leaked.len(), 0, "{leaked:?}");
    }

    /// **The restricted items are machinery, and there are enough of them for the count to have
    /// been worth checking.** A report rather than a gate on the number: seventy-eight is a property
    /// of how much internal state five flat structures need, and a gate on it would be edited every
    /// time one of them gains a helper.
    #[test]
    fn the_restricted_items_are_inside_the_line() {
        let all = restricted();
        let modules: BTreeSet<&str> = all.iter().map(|(m, _)| m.as_str()).collect();
        println!(
            "{} restricted declarations across {} modules: {:?}",
            all.len(),
            modules.len(),
            modules
        );
        for (module, _) in &all {
            assert_ne!(module, "lib.rs");
        }
    }

    // ── the engine names on the surface ──────────────────────────────────────────────────────────

    /// **What the runtime re-exports from the engine, in both directions.** A `pub use
    /// vitui_engine::…` that [`ENGINE_NAMES`] does not carry is the finding closing itself without
    /// anyone saying so; a reachable row the source does not have is the opposite.
    #[test]
    fn the_engine_names_reachable_through_the_runtime_are_the_documented_seven() {
        let mut found = BTreeSet::new();
        for module in modules() {
            for line in read(&src_dir().join(&module)).lines() {
                let s = line.trim();
                let Some(body) = s.strip_prefix("pub use vitui_engine::") else {
                    continue;
                };
                let body = body.trim_end_matches(';');
                match body.find('{') {
                    Some(open) => {
                        let close = body.rfind('}').expect("a braced re-export closes");
                        for name in body[open + 1..close].split(',') {
                            let name = name.trim();
                            if !name.is_empty() {
                                found.insert(
                                    name.split(" as ")
                                        .next()
                                        .expect("a name")
                                        .trim()
                                        .to_string(),
                                );
                            }
                        }
                    }
                    None => {
                        found.insert(body.trim().to_string());
                    }
                }
            }
        }
        let documented: BTreeSet<String> = ENGINE_NAMES
            .iter()
            .filter(|e| e.reachable_as.is_some())
            .map(|e| e.name.to_string())
            .collect();
        assert_eq!(found, documented);
        assert_eq!(
            ENGINE_NAMES
                .iter()
                .filter(|e| e.reachable_as.is_none())
                .count(),
            17
        );
    }

    // ── the in-binary arm of ticket 17's report ──────────────────────────────────────────────────

    /// **The dense frame, measured from inside the library.** The other arm of
    /// `examples/crate_line_numbers.rs`, and the reason it is a test rather than a second case in
    /// that example: *inside the crate* is a place an example cannot be. The fixture is the same
    /// file — `crate::screen` is `#[path]`-included there and compiled here — so the only difference
    /// between the two numbers is which side of the rlib the drawing verbs are called from.
    ///
    /// **Behind an environment variable, and `n=1 cargo test golden` is the precedent.** A timing in
    /// `cargo test` is not a gate and must not become one: it is off by default, it prints, and it
    /// asserts nothing about the clock.
    ///
    /// ```text
    /// line=1 cargo test --release -p vitui-runtime --lib \
    ///     line::tests::the_frame_measured_inside_the_crate -- --nocapture
    /// ```
    #[test]
    fn the_frame_measured_inside_the_crate() {
        if std::env::var_os("line").is_none() {
            println!("set line=1 to measure; see this test's documentation");
            return;
        }
        let mut driver =
            crate::ctx::Driver::headless(300, 80).expect("a sink cannot fail to attach");
        let mut twice = 0;
        driver.frame(|cx| twice = crate::screen::dense_draw(cx, false));
        assert_eq!(twice, 5902, "the fixture's double-write count has changed");

        let report = vitui_bench::Bench::new(40)
            .case("frame/text-first", 1, || {
                driver.frame(|cx| {
                    std::hint::black_box(crate::screen::dense_draw(cx, true));
                });
            })
            // The small arm, and the one that can see a boundary: see the example's comment.
            .case("layout/32-splits", 100, || {
                std::hint::black_box(crate::screen::screen_frame(300, 80));
            })
            .run();
        println!(
            "in-binary arm, a whole dense frame, minimum of 40 rounds:\n{report}\n\
             lto: {}",
            option_env!("CARGO_PROFILE_RELEASE_LTO").unwrap_or("thin (the manifest's)")
        );
    }

    // ── the four manifest decisions ──────────────────────────────────────────────────────────────

    /// **The runtime takes `vitui-engine` and nothing else**, read out of the manifest rather than
    /// out of `deny.toml`'s comment. `cargo deny` checks the negative half — a banned crate — and
    /// this checks the positive one, which no gate had.
    #[test]
    fn the_runtimes_manifest_names_one_dependency() {
        let manifest = read(&PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/Cargo.toml"
        )));
        assert_eq!(dependencies_of(&manifest), vec!["vitui-engine"]);
    }

    /// **The components crate names the runtime and nothing else** — components spec §0's C6, which
    /// is what makes `crates/vitui-components/tests/crate_line.rs` a component-facing consumer
    /// rather than one more test with the engine in scope.
    #[test]
    fn the_components_manifest_names_only_the_runtime() {
        let manifest = read(&workspace_root().join("crates/vitui-components/Cargo.toml"));
        assert_eq!(dependencies_of(&manifest), vec!["vitui-runtime"]);
    }

    /// The `[dependencies]` table's keys, in order.
    fn dependencies_of(manifest: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut inside = false;
        for line in manifest.lines() {
            let s = line.trim();
            if s.starts_with('[') {
                inside = s == "[dependencies]";
                continue;
            }
            if !inside || s.is_empty() || s.starts_with('#') {
                continue;
            }
            let Some((key, _)) = s.split_once('=') else {
                continue;
            };
            let name = key.split('.').next().expect("a key").trim();
            if !out.iter().any(|n| n == name) {
                out.push(name.to_string());
            }
        }
        out
    }

    /// **`deny.toml` states the line and `[bans] allow` is not used**, with the number beside the
    /// refusal. Ticket 14 wrote 38 against a prototype workspace; the shipped graph is 33 — see the
    /// comment in `deny.toml`, which now carries both and the drift between them, because a number
    /// that moved five without a single `deny.toml` edit is the maintenance cost the refusal is
    /// about.
    #[test]
    fn the_dependency_line_is_a_gate_and_the_allowlist_is_refused() {
        let deny = read(&workspace_root().join("deny.toml"));
        assert!(deny.contains(r#"{ name = "crossterm", wrappers = ["vitui-engine"] }"#));
        assert!(deny.contains(r#"{ name = "vitui-signals", wrappers = [] }"#));
        // **The `[bans]` table only.** `[licenses] allow` is a different key with the same name and
        // is legitimately there — a scan over the whole file fails on it, which is how this gate
        // learned to say which table it means.
        let bans = deny
            .split("[bans]")
            .nth(1)
            .expect("deny.toml declares a bans table")
            .split("\n[")
            .next()
            .expect("the table ends");
        for line in bans.lines() {
            assert!(
                !line.trim_start().starts_with("allow"),
                "an allowlist appeared in [bans]: {line}"
            );
        }
        assert!(
            deny.contains("33 crates"),
            "the measured graph size is not in the comment"
        );
    }

    /// **The release profile: `codegen-units = 1` is gone and `lto = "thin"` stays.** Four crates
    /// are four codegen units whatever the setting says, so it was moot on the shipped workspace and
    /// cost 4.3 µs — 4.3% of the frame — on a single-crate build, which is the shape an application
    /// embedding the runtime can produce. ThinLTO is what earns the inlining back.
    #[test]
    fn the_release_profile_has_no_codegen_units_and_keeps_thin_lto() {
        let manifest = read(&workspace_root().join("Cargo.toml"));
        let profile = manifest
            .split("[profile.release]")
            .nth(1)
            .expect("the workspace declares a release profile");
        assert!(profile.contains(r#"lto = "thin""#));
        for line in profile.lines() {
            assert!(
                !line.trim_start().starts_with("codegen-units"),
                "codegen-units is back in the release profile: {line}"
            );
        }
        // The four numbers the decision rests on, in the comment above the profile.
        for number in ["33.0", "28.7", "28.4", "4.3"] {
            assert!(
                manifest.contains(number),
                "{number} is not beside the profile"
            );
        }
    }
}
