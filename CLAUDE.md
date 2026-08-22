# CLAUDE.md

> **The local GitLab moved out of this repository (2026-08-22).** One shared instance now serves
> every repo on this machine: <http://gitlab.localhost:8940>, this repo's project is `repos/vitui`,
> and it is started with `devkit up`. The `infra/` stack here is gone — see `infra/MOVED.md` for the
> old-command-to-new-command table, and `~/Projects/devkit/README.md` for the manual.

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`vitui` is a Rust TUI library: fast, layered, reactive terminal rendering, meant to be the foundation
a component library stands on. **Nothing is implemented yet.** The workspace is scaffolding; the
architecture is being decided one ticket at a time on a wayfinder map.

Read these before working, in this order:

1. `.scratch/vitui-engine-architecture/map.md` — the live map. Its **Notes** section holds the settled
   decisions that constrain everything; do not reopen them without a new decision.
2. `CONTEXT.md` — the glossary. Use its terms in code, comments, tickets and commit messages.
3. `docs/adr/` — decisions that are hard to reverse and surprising without context.
4. `.scratch/vitui-runtime-architecture/spec.md` — the runtime's settled architecture, if the work is
   above the engine. Its map is closed; `architecture.md` beside it is the superseded proposal and is
   kept only as the record of what was argued.

## Workspace

```
crates/vitui-engine       cells, surfaces, layers, compositing, damage, frame writer
                          └ crossterm behind a seam: raw mode, input, capability detection
crates/vitui-runtime      layout, identity, focus, hit-testing, routing, key maps, theming,
                          overlays, the data contract — no scene tree, no reactivity
crates/vitui-components   windows, panels, charts, lists, trees, forms, pickers
crates/vitui              facade re-export
crates/vitui-alloc-probe  dev-only counting global allocator (publish = false)
fuzz/                     two libFuzzer targets and the committed corpus that is their gate
                          └ a **detached workspace**: cargo-fuzz needs nightly and libfuzzer-sys,
                            which the engine's dependency policy will not have. That is a loophole,
                            not a permission — it has its own `deny.toml` and its own CI invocation.
```

## Commands

```bash
cargo build --workspace
cargo test --workspace
cargo test -p vitui-engine name_substring   # single test
cargo test --workspace -- --test-threads=1  # required for allocation assertions
cargo clippy --workspace --all-targets
cargo fmt --all
cargo bench --workspace                     # criterion; -- --test to just check it runs
cargo deny check                            # needs `cargo install cargo-deny`
cargo clippy -p vitui-engine --all-targets --features fuzz   # the one configuration `cargo test` misses
(cd fuzz && cargo deny check)               # `fuzz/` is a detached workspace: its own graph, its own gate
```

The fuzz targets are a **soak, never a gate** — the committed corpus replayed by `cargo test` is the
gate. They need nightly and `cargo-fuzz`, and `fuzz/README.md` is the whole procedure:

```bash
RUSTUP_TOOLCHAIN=nightly cargo fuzz run draw_sequence -- -max_total_time=900
```

## Rules that are decisions, not preferences

Violating any of these silently undoes a decision that cost a session to make.

- **The engine does not lay anything out.** Callers bring rectangles. No layout concept may enter
  through the `Surface` API — that is the door this erodes through. See `docs/adr/0002`.
- **The engine never iterates application data.** It offers clipping and offset viewports; culling is
  the caller's job. Invariant: *frame cost is proportional to visible cells, never to data volume.*
- **crossterm is invisible.** It is used for input and terminal mode only, never for output, and must
  not appear in any public signature. See `docs/adr/0001`.
- **Damage is marked at write time, not derived by diffing.** Prior art measured ratatui's full-buffer
  diff at ~170 µs on 300×80 — already over the budget for a whole frame.
- **A cell holds an interned grapheme-cluster handle, not a `char`.** Forced by UAX #29.
- **Dependency policy.** Engine: crossterm plus generated UCD tables. Runtime: nothing. Enforced by
  `deny.toml`.
- **Performance budget** (CI gates, not aspirations): full-screen 300×80 composition < 1 ms; typical
  damage-tracked frame < 100 µs; zero allocations during frame composition.

## Local CI

The `.gitlab-ci.yml` gates run on a **shared local GitLab** at <http://gitlab.localhost:8940>,
project `repos/vitui`. The instance is not in this repository — it lives in `~/Projects/devkit` and
is shared with every other repo on this machine.

```sh
devkit up                                  # start it (or bring it to spec) — the only mutating verb
devkit down                                # stop it, keeping everything
. ~/.local/state/devkit/env                # GITLAB_HOST, DEVKIT_TOKENS, DEVKIT_GROUP
```

Its runner has **no default job image**: it serves every repo, so `.gitlab-ci.yml` must name its own
(`default: image:` is required, not decorative). Six concurrent slots, shared with every other repo.
`~/Projects/devkit/README.md` is the manual. This replaced a per-repo GitLab that used to live in
`infra/` here.

## Working the map

One ticket per session, claimed by setting `Status: claimed` before any work. Tickets live in
`.scratch/vitui-engine-architecture/issues/`; resolution appends an `## Answer` section, sets
`Status: resolved`, and adds a one-line pointer to the map's Decisions-so-far. Research findings go in
`.scratch/vitui-engine-architecture/research/`.

`docs/agents/issue-tracker.md` describes the tracker conventions in full.
