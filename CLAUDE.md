# CLAUDE.md

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

## Working the map

One ticket per session, claimed by setting `Status: claimed` before any work. Tickets live in
`.scratch/vitui-engine-architecture/issues/`; resolution appends an `## Answer` section, sets
`Status: resolved`, and adds a one-line pointer to the map's Decisions-so-far. Research findings go in
`.scratch/vitui-engine-architecture/research/`.

`docs/agents/issue-tracker.md` describes the tracker conventions in full.

## Language

Conversation with the user is in **Russian**. Every artifact — code, comments, tickets, specs, ADRs,
commit messages — is in **English**.
