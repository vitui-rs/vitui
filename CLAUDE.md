# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> **The local GitLab moved out of this repository (2026-08-22).** One shared instance now serves
> every repo on this machine: <http://gitlab.localhost:8940>, this repo's project is `repos/vitui`,
> and it is started with `devkit up`. The `infra/` stack here is gone — see `infra/MOVED.md` for the
> old-command-to-new-command table, and `~/Projects/devkit/README.md` for the manual.

## What this is

`vitui` is a Rust TUI library: fast, layered terminal rendering, meant to be the foundation a
component library stands on. Version `0.0.0`, unpublished, no stability promise before 0.x. MSRV
1.85.

**Where the build actually is** (keep this paragraph current — it is the first thing a session needs):

- **`vitui-engine` is implementation-complete**: all 26 tickets of `.scratch/vitui-engine-impl/`
  resolved, all 27 entries of the verification register wired with none pinned red, ~30k lines.
  Production readiness added a 28th, `conform/`, which spec §14 had no way to state — and that
  instrument has already earned its keep four times: `quirks.rs`'s fourth entry (tmux accepts SGR 53,
  stores it, and never forwards it); production ticket 10, which found `attrs_dropped` populated,
  printed and read by nothing — now wired at `quant::Quantiser::attrs`, where one field narrows the
  wire, the mirror, the gap pricing and the round trip's expectation together; `quirks.rs`'s fifth
  entry (kitty has no cursor attribute for conceal or overline, settled from the shipped binary
  because no dump can tell *not stored* from *not serialised*); the row it **refused** to answer,
  where a dotted underline kitty renders arrives in the capture as `CSI 4 : m` and comparing it would
  have earned a sixth bit for a misbehaviour that is not happening; and **architecture ticket 20**,
  the only decision on the engine map settled by asking a terminal rather than by argument — scene 04
  asked three families what they do with a cluster printed over one half of a double-width glyph, all
  three blank the orphaned half themselves, and *they disagree about what it wears*, which is what
  made the engine's own repair mandatory rather than merely tidy.
- **`vitui-runtime` is implementation-complete**: all 21 tickets resolved, the last (20, the headroom
  ledger) on 2026-08-24. 05, 12, 15 and 16 landed together on 2026-08-23 and 06, 13 and 14 on
  2026-08-24, then 21 and 17 together, then 18 and 19, then 20; all built in parallel worktrees and
  integrated one pipeline at a time.
  `data`, `layout`, `theme` with its fourteen shipped schemes, `keys`, `ctx`, `id`, `route`, `focus`,
  `sizing`, `work`, `anim`, `overlay` and `scroll` exist, and **the crate line is now built rather
  than counted** — the component-facing surface is checked by a crate that cannot name the engine
  (`crates/vitui-components/tests/crate_line.rs`), which is what the 0-restricted-items count had
  been standing in for. **The register and the scene list are both all-green**
  (tickets 19 and 20): `src/register.rs` is 39 entries and `src/scenes.rs` is spec §20's twenty
  scenes, and what makes them registers rather than documents is that an `Instrument` is a **value
  with a file in it** — a test name, a hostile line that must sit inside a `compile_fail` block, or a
  trimmed attribute — which a test opens the file and checks. That is what turned entry 12 and scene
  19 red, from two sides of one failure: *a number measured in a file nothing evaluates*. Entry 12
  needed a CI line (`examples/frame.rs` now runs in the `budget` job); scene 19 needed a gate that is
  not a timing, and its figure had moved besides — 320.85 → 226.92–237.21 µs — while the step counts
  are exact every run, so it counts steps and gates a growth relation. `State::Red` now carries
  `#[expect(dead_code)]`, so the next red row cannot arrive without deleting that attribute.
  **`src/ledger.rs` is where every gated or reported number in the crate lives** (ticket 20), and
  what it found is that spec §19's headroom table subtracted prototype rows from a prototype base:
  the shipped dense frame is **89.75–89.92 µs here and 91.88 µs on the runner**, not 33, so the
  headroom is **1.11×** rather than ≈67 µs. **The rows do not sum to the frame** — 90.3% of it is the
  engine serialising 7 488 damaged cells and the runtime's own share is 8.71 µs — so the table prints
  both and a failure names a layer. The frame is gated at **2× and not at the budget**, which is the
  engine's own ruling: 1.1× on a runner nobody has measured is a flaky test wearing a budget's
  clothes, and reclassifying the frame into the 1 ms full-screen class to buy headroom is forbidden
  by name.
  **Six of the ten found defects in code that was already green**, which is the argument for a
  consumer over another gate: ticket 15 found `Ctx::hover_style` translating from `rect` rather than
  from an accumulated origin, wrong at every level below the first two; ticket 16 found a stale
  landing able to destroy a fresh one in `Slot::put`, which is where `Generation`'s ordering earns
  its keep; ticket 14 found the wheel riding `Awarded` and so arriving a frame late, wrong for the
  one channel whose reader is inside the draw; ticket 06 found the runtime had two wakeup sinks and
  flushed one; and ticket 21 found the frame arena **leaking** a body requested inside `Ctx::measured`
  — a sizing dry run builds a throwaway `Frame` no pass ever runs over, so a body owning a `String`
  was never dropped, and the one gate that could have caught it only ever exercised the real pass;
  and ticket 19 found `ctx::Frame::tab_walk` carrying **two `compile_fail` fences and no twin**,
  neither of them naming `tab_walk` or `ring` by path — so renaming either would have left both
  halves failing identically and reporting `ok`, `E0599` for *the method you meant moved* and
  `E0599` for *the method you must not have was never built* being the same diagnostic. **The twin
  is the only half of a pair that holds**, and that is a measured fact rather than a preference:
  rustdoc on stable ignores the error code beside `compile_fail`, so ` ```compile_fail,E0599 ` over
  a body whose real error is `E0432` still reports `ok`. The codes are documentation of intent; a
  pair is held to its subject by a twin that names the protected item, and where the pair does not
  sit on that item a `**Protects:**` line says which one it is.
- **`vitui-signals` exists, and it is a detached workspace because the rule taken literally required
  one** (ticket 18, 2026-08-24). 112 lines of code above `vitui-runtime` — `Signal<T>`, `Graph`,
  `Computed<T>` — and the facade deliberately does not re-export it. `deny.toml` has carried
  `{ name = "vitui-signals", wrappers = [] }` since ticket 14, glossed as *the crate may exist and be
  published, and nothing in this workspace may depend on it*; **building it found that sentence is
  stronger than it reads.** `cargo deny`'s `[bans] deny` bans a crate's *presence in the graph* and
  `wrappers` is the exception list, so a member with no dependents at all fails the gate outright —
  which is what the first `cargo deny check` said, with no dependent to name. So the crate carries its
  own `[workspace]` table and the root excludes it, which is also **what makes the ban live**: it now
  fires the day a member writes the dependency instead of already failing. Its gates are folded into
  the `test` job, because `cargo test --workspace` cannot reach them. Three drivers of one screen agree
  in **0 of 24 000 cells**, and a frame that redraws one region declares **1 hit entry against 312 and
  0 tab stops against 43** with **24 000 of 24 000** cells still correct: *fine-grained reactivity here
  is not expensive; it is a request to revert the hit index.*
- **There is no `unsafe` in any shipped crate, at any layer** (ticket 21, 2026-08-24; ADR 0034).
  `vitui-runtime`, `vitui-components` and the `vitui` facade each carry `#![forbid(unsafe_code)]`
  beside the engine's, and the subsumed `#![forbid(unsafe_op_in_unsafe_fn)]` is removed rather than
  left beside it looking like a second rung. `vitui-alloc-probe` is the one stated exemption
  (`GlobalAlloc` cannot be safe; `publish = false`; no consumer of `vitui` ever compiles it).
  **What it cost is a budget figure, and the map moved with the code.** `overlay` used to hold a
  crate-private bump arena — a type-erased overlay body between the two passes, seven `unsafe`
  blocks and two `unsafe fn` — because a `Box<dyn FnMut>` per request per frame is one allocation
  against a budget of zero. A body is now one `Box` in a queue the frame call owns, which costs a
  frame with **n** overlays standing **n + 1** allocations and a frame with none nothing at all.
  The `+ 1` is not slack: a stored body is `+ 'f` and safe Rust cannot put a `'f`-bounded value
  inside the thing borrowed for `'f`, which is what `Frame` and `Driver` both are — so the queue is
  a local of `Driver::frame` rather than a field, and the queue itself is the extra allocation.
  **That is exactly why the arena erased types in the first place**; an offset into a byte buffer
  mentions no lifetime. Spec §19 carries the exception in the same sentence as the count, spec §10
  describes the body queue, ADR 0017 is *partially superseded* through its `status:` field with its
  body untouched, and the gate is the marginal equality — one more overlay standing is exactly one
  more allocation a frame.
- **`vitui-components` has started**: 8 of 43 tickets resolved (2026-08-24). `INVENTORY` is spec
  §17's twenty-nine-row freeze **as a value a test iterates**, with the five documentation and
  verification obligations as functions over it — so *which components must this gate run against* is
  answerable by the machine from here on. All five obligations are `Unmet` and each is watched
  panicking, because a query with no evidence must fail loudly rather than pass: O4 first returned
  **`Met` over 29 rows with no evidence at all**, since *an equality between two things that do not
  exist holds*. **Building the freeze contradicted four figures in the closed map** — the built count
  (19/10, not ADR 0033's thirteen-unbuilt), the count of empty families (five, not §17's two), §1's
  layer rule against §6's own composition, and the `layer` column being uncheckable without stated
  edges. All four are asserted as measured rather than bent to fit.
- **Ticket 05 is the one that reached down into the runtime**, and §16's split says so in as many
  words: the runtime owns the mechanism and the two-count invariant, the ticket owned the entry
  list. So `vitui_runtime::Glyph` went from **7 entries to 20** and `Distinction` from **3 to 10**,
  and the interesting half is that **`Distinction::carried_by` makes ADR 0032's sentence
  arithmetic**: seven of the ten name a glyph pair, three name none, and the three that name none
  are exactly the three narrowing takes away. A declared repertoire is now a real input to the bits,
  so `with_glyphs` re-narrows and the two builders commute — which needed a private `resolved` flag,
  because `ColorDepth::None` had been doing duty as both *nobody has said* and *a terminal with no
  colour* and that only worked while every distinction was carried on the colour axis.
  **The gate is cross-family collapse and not the pairwise version**, and the difference is 0
  against 43 at ASCII: the nine box-drawing entries all spell `+` on purpose — `C(9, 2) == 36`,
  which is §16's *36 of 190* exactly — so the pairwise form fires on every border while the real
  defect is C09's, `Ellipsis` spelled `>` where `>` is an `ArrowRight`. Spelling it back is watched
  firing in five places at once, one of them naming `("tree", ArrowRight, Ellipsis)`.
  **The tier axis is a barrier from that crate** — register row 45, new: `ColorDepth` is
  `reachable_as: None`, so §16's role and distinctions-lost columns are measured in the runtime's
  `theme_numbers` and the repertoire columns in `glyph_numbers`. Neither `1 / 2 / 13 of 78` nor
  `0 / 1 / 2 of 10` reproduces — the shipped palette gives **`0 / 0 / 18`** and **`0 / 1 / 3`** —
  and the palette was deliberately *not* swapped to make them, because the only reason to would have
  been the number.
- **Ticket 06 is the first component-facing code in the workspace**, and it had to settle **runtime
  architecture issue 22** to write a signature at all. Spec §3 states both helpers as sentences about
  a return value — `text::fit` *returns the remainder*, `frame::block` *returns the rectangle it did
  not write* — and neither is writable here: `vitui_engine::Rect` is `reachable_as: None`, 27 of the
  runtime's public declarations name it, and C6 says this crate's dependency list is `vitui-runtime`
  and nothing else. There is no second spelling — Rust has no `typeof`, an `impl Trait` return is
  opaque to its own crate, and no public runtime trait carries `Rect` as an associated type. **So the
  crate names its own rectangle**: `Cells`, four scalars in the coordinates of the `Ctx` they came
  from, its algebra swept against `layout::rect`'s over a corpus, and the two verbs that need a real
  `Rect` building one **by inference** through a private macro — a function would need a return type.
  It is deliberately not called `Rect`, because two of those in one workspace makes every mismatch
  read *expected `Rect`, found `Rect`*.
  **The closure form was refused on two measurements rather than on taste.** `block(cx, opts, |cx| …)`
  delivers the interior through `Ctx::child`, which moves the origin: one `Tally` over such a panel
  reads **124 double writes on a panel that has none**, and split into two tallies the one thing the
  ticket asks to be measurable — *a `block` that clears what it hands over* — lands in two ledgers and
  is checked by nothing. Second, `CONTEXT.md`'s identity rule is not negotiable: *a container that
  returns a rectangle preserves its children's identity and one that takes a closure renames them*.
  `Cells::child` exists so the **caller** can narrow; what a container may not do is narrow on the
  caller's behalf.
  **The gate is watched catching the 15-cell instance.** The correct panel and ADR 0026's defective
  one are **one function with one boolean between them** (`title_split`), so a reviewer's diff is one
  line: the defective arm writes **15 cells twice**, touches exactly the same cells, and costs **one
  verb fewer** — 45 against 46. Every counter but the pair prefers it.
  **One screen at 300×80, drawn four ways**, is where the rest of the numbers come from: `fit`
  against the same order written out by hand is **0 of 24 000 cells apart at 18 912 writes against
  18 912**; the naive form is **40 925 against 0**; a `block` that clears its interior is **16 224**;
  `Compact` against `Cosy` is 18 912 / 171 against 19 206 / 167 with **four widgets** off the bottom.
  Spec §3's remembered 20 804 / 267 against 20 992 / 263 is a **prototype screen this ticket does not
  own** — scenes 1–2, which ticket 09 stands up — so `examples/partition_numbers.rs` prints both
  columns rather than engineering the difference away. Two structural facts do reproduce exactly: the
  region delta is four either way, and the density direction (Cosy writes *more* while standing
  *fewer* widgets) is the same.
  **The seam that makes the gate measure the shipped path is `ink::Ink`.** A component's shape is
  `fn(&mut Ctx, …) -> Response` and nothing has decided a component draws through a wrapper, so the
  helpers cannot take a `Tally` — and a second implementation written against one would be a gate
  testing a copy. One trait, two methods, three implementations: `Direct` for a component (stages
  into the frame's buffer, **0 allocations over 200 frames**), `Tally` and `Pen` for a gate. Nothing
  either helper writes goes through `Ctx::fill`, which returns `()` — so `reported == writes` holds
  and the pair really is two sources compared.
  **`frame::focus_ring` is deleted and two gates keep it deleted**: a `compile_fail` naming it by
  path with a twin naming `block` and `BlockOpts` by path, and a source scan that catches it coming
  back `pub(crate)` — which is how a deleted helper actually returns. The scan's first run reported
  **itself**, because a scanner looking for a literal contains that literal.
- Nothing above the engine can draw a screen yet, so no application exists to run.

Read these before working, in this order:

1. The spec for the layer being worked on — `.scratch/vitui-engine-architecture/spec.md`,
   `.scratch/vitui-runtime-architecture/spec.md`, or
   `.scratch/vitui-components-architecture/spec.md`. All three maps are **closed**; the specs are the
   authority. An `architecture.md` beside a spec is the superseded proposal, kept only as the record
   of what was argued.
2. `CONTEXT.md` — the glossary. Use its terms in code, comments, tickets and commit messages.
3. `docs/adr/` — 34 decisions that are hard to reverse and surprising without context. 0001–0011 and
   0022–0025 are the engine, 0012–0021 and 0034 the runtime, 0026–0033 the components.
4. The impl backlog `README.md` for the layer being worked on — it holds the phase order, the
   blocking edges, and the defects that shaped both.

**Five engine questions reopened the closed map** and are recorded in
`.scratch/vitui-engine-architecture/issues/19`–`23`. **20 is resolved** (2026-08-23, production ticket
06: the repair rules are bounded by the surface and not by the clip, and §4 carries a stated
exception). Two are still open (21, 23): the implementation chose an answer and the tests lock it in,
but the spec still says two things. Do not "fix" the code to match one sentence of the spec without
resolving the ticket.

## Workspace

```
crates/vitui-engine       cells, surfaces, layers, compositing, damage, serializer, frame writer
                          └ crossterm behind a seam: raw mode, input, capability detection
crates/vitui-runtime      layout, identity, focus, hit-testing, routing, key maps, theming,
                          overlays, the data contract — no scene tree, no reactivity
crates/vitui-components   windows, panels, charts, lists, trees, forms, pickers (6 of 43)
                          └ `Cells` is this crate's own rectangle: `vitui_engine::Rect` is
                            unnameable across the crate line, so the two partition primitives
                            return one of these instead (runtime architecture issue 22)
crates/vitui              facade re-export — engine, runtime, components, and deliberately not signals
crates/vitui-signals      a fine-grained signal graph, 112 lines above the runtime
                          └ a detached workspace, and the only one detached by a `deny.toml` rule:
                            `wrappers = []` bans the crate's presence in the graph, not only its
                            dependents, so a member with nothing depending on it fails outright
crates/vitui-bench        round-robin minimum-of-N measurement, no deps (publish = false)
crates/vitui-alloc-probe  counting global allocator for the allocation gates (publish = false)
examples/app-template     copy-this-directory starting point, and the home of spec §11's lint rung
                          └ a detached workspace; its `clippy.toml` is the only way an application
                            gets the disallowed-methods rung, because clippy config does not
                            propagate from a dependency
compare/                  the comparative suite: SCENES.md normative, harness.py the instrument,
                          run.sh the entry point, REPORT.md committed and regenerated, FINDINGS.md
                          written by hand. Nine scenes, five arms — and two of the arms are ours
                          (engine, facade), so the runtime's cost is a subtraction inside the table
                          └ detached workspace; reports, never gates. No deny.toml, deliberately:
                            the third-party dependencies are the subject
conform/                  the only instrument that asks a real terminal rather than our model of one:
                          SCENES.md normative, four arms as examples (Ghostty, Ghostty-via-tmux,
                          tmux, kitty), one committed REPORT-<arm>.md each, FINDINGS.md by hand
                          └ detached workspace, no deny.toml — the third-party thing IS the subject.
                            The live arms are soaks; the gate is `cargo test` over fixtures/, which
                            runs inside the `test` CI job. The engine is a DEV-dependency, so the
                            comparator cannot link the code it is checking.
fuzz/                     two libFuzzer targets and the committed corpus that is their gate
                          └ a detached workspace: cargo-fuzz needs nightly and libfuzzer-sys,
                            which the engine's dependency policy will not have. That is a loophole,
                            not a permission — its own `deny.toml`, its own CI invocation.
scripts/                  the three gates and one report that cannot be a `cargo test`: idle,
                          observer, lint rung, steady state
```

## Commands

```bash
cargo build --workspace
cargo test --workspace -- --test-threads=1  # the real invocation; allocation gates need one thread
cargo test -p vitui-engine name_substring   # single test
cargo clippy --workspace --all-targets
cargo clippy -p vitui-engine --all-targets --features fuzz   # the one config `cargo test` misses
cargo fmt --all
cargo doc --workspace --no-deps             # a gate: a broken intra-doc link fails the job
cargo deny check                            # needs `cargo install cargo-deny`
(cd fuzz && cargo deny check)               # detached workspace: its own graph, its own gate
(cd conform && cargo test)                  # the conformance gate, over committed captures
(cd crates/vitui-signals && cargo test -- --test-threads=1)   # detached: --workspace misses it
(cd conform && cargo run --example tmux)    # the one conformance soak that is headless
(cd conform && cargo run --example kitty)   # a window, but no automation grant and no config file
```

Warnings are denied workspace-wide (`[workspace.lints.rust] warnings = "deny"`), so an enum variant
nothing constructs is a build failure rather than a spare part — several modules say so in a comment
where a reader would otherwise expect a missing arm.

There are **no `cargo bench` targets** — criterion was removed and replaced by `vitui-bench`. Timing
lives in examples that print a report:

```bash
cargo run --release --example budget -p vitui-engine     # asserts the gates, prints the numbers
cargo run --release --example layout_numbers -p vitui-runtime   # one of sixteen *_numbers reports
(cd crates/vitui-signals && cargo run --release --example signals_numbers)
scripts/idle-gate.sh 30       # 0.00 user / 0.00 sys over 30 s; thirty is a floor, not a preference
scripts/observer-gate.sh      # the debug observer is absent from a release binary
scripts/steady-report.sh      # 60 fps for 30 s against 5% of a core
scripts/lint-rung-gate.sh     # the clippy.toml rung fires in an application and not from a dep
n=1 cargo test -p vitui-engine golden        # regenerate the golden frames; review the git diff
```

The fuzz targets are a **soak, never a gate** — the committed corpus replayed by `cargo test` is the
gate. `fuzz/README.md` is the whole procedure. On this machine `~/.cargo/bin` must come first on
`PATH` or Homebrew's cargo shadows rustup's and the toolchain selection is silently ignored:

```bash
RUSTUP_TOOLCHAIN=nightly cargo fuzz run draw_sequence -- -max_total_time=900
```

## Architecture that takes several files to see

**The frame, as a sequence.** `Engine::new(Config)` → `attach()` on the app thread → `(Screen,
WakeHandle)`. Then per frame: add layers and draw into a `View` with three verbs (`text`, `fill`,
`restyle`), which mark damage as they write; `set_mouse` and `set_cursor`; `present()`. `present`
leases a packet, composites the damaged rectangles bottom-up, packs runs plus their cells, submits.
The render thread takes the packet, serialises against its mirror of the screen, writes once, returns
the packet. Damage is marked by the verbs and cleared by `present`, and neither is reachable from
outside — which is why nothing above the engine can force a full repaint.

**`Config::clock` is public API, not a test fixture.** Under `Clock::Manual` both halves of that
sequence run inline on the calling thread before `present` returns — same mailbox, same packet, same
bytes — so a test is a straight-line program. Threading properties (zero wakeups, the packet never
superseded, wake-up latency) are the ones that cannot be tested in the mode that removes them and
live in the threaded mode as counts.

**The app-thread role is enforced by split handles, not a capability token.** `Screen` and `View` are
`!Send` via a private `PhantomData<*const ()>`; `Parker`/`Unparker` and `Producer`/`Consumer` split
so an unreachable method is simply not on the type. The compiler cannot stop the app thread from
being slow — that is a ~44 ns in-loop overrun detector (`perf.rs`) plus a debug-only observer thread,
and the shortfall is written down rather than implied. See ADR 0003 and spec §11.

**The runtime has no scene tree and no retained structure.** The clip stack is the call stack, the id
path is the closure tree, and what survives one draw is five flat structures rebuilt from the next
draw (ADR 0012). `Ctx<'f, 'v>` carries **two** lifetimes deliberately: with one, `child()` shrinks it
and an overlay body capturing a base-pass local compiles, which deletes the mechanism overlays rest
on. A frame consumes at most one routing edge; there are no per-id inboxes (ADR 0016).

**The verification machinery is itself architecture**, and it is the part most likely to be
misunderstood as test scaffolding:

- `register.rs` — spec §14's 27 properties as a value, each with its instrument and provenance, plus
  a 28th the production backlog added for what §14 could not state. A property may be *pinned red*
  with the ticket that will invert it. Currently 28 wired, 0 red.
- `roundtrip.rs` / `testing.rs` — the primary instrument: composite, serialise, replay the bytes
  through the terminal model, assert the replayed screen equals the frame. It stores nothing. All
  four defects the architecture map found were found this way. A golden *byte string* is refused
  because the encoding is exactly the part allowed to change.
- `reference.rs` — the obviously-correct, far-too-slow compositor. Gate #1 is *generated from it*,
  not hand-written, because a hand-written expectation about damage is written by the person who
  wrote the damage.
- `scenes.rs` — the twelve scenes as a normative list. Every gate runs over every scene; a gate that
  picks its own scenes tests the scenes.
- `golden.rs` — the residue, covering only what the round trip cannot reach: the composited picture.
  Regenerated with `n=1`, reviewed as a git diff, and `n=1` in CI is an error.
- `ledger.rs` — **every gated or reported number has exactly one home here**, with the machine it was
  measured on. The audit that produced it found the watchdog threshold copied into nine files.
- `audit.rs` — the public surface as a value, with counts as gates: no public traits, and the paired
  compile-fail corpus — 36 hostile cases and 48 positive twins beside them, each twin naming the
  protected item *by path*, because a lone `compile_fail` also passes when the type has been renamed
  (`E0433` instead of `E0277`, and the mechanism cannot tell those apart).

## Rules that are decisions, not preferences

Violating any of these silently undoes a decision that cost a session to make.

- **The engine does not lay anything out.** Callers bring rectangles. No layout concept may enter
  through the `Surface` API — that is the door this erodes through. See `docs/adr/0002`.
- **The engine never iterates application data.** It offers clipping and offset viewports; culling is
  the caller's job. Invariant: *frame cost is proportional to visible cells, never to data volume.*
- **crossterm is invisible.** Input and terminal mode only, never output, and never in a public
  signature. See `docs/adr/0001`.
- **Damage is marked at write time, not derived by diffing.** Prior art measured ratatui's
  full-buffer diff at ~170 µs on 300×80 — already over the budget for a whole frame.
- **A cell holds an interned grapheme-cluster handle, not a `char`** (UAX #29), and no cell, handle or
  style bit is readable from outside the engine (ADR 0023).
- **No traits in the engine's public surface, and `#![forbid(unsafe_code)]`.** The engine has nothing
  to call upward, so the dependency arrow is enforced by there being no arrow.
- **Dependency policy.** Engine: crossterm plus build-script-generated UCD tables. Runtime: nothing.
  Components: case by case. Enforced by `deny.toml`.
- **A gate is a count, a ratio, an equality or a compile outcome. A timing is a report, and a gate
  only at cliff granularity, with the headroom written next to the number.** A gate tuned to the
  measurement is a flaky test that gets disabled within a month.
- **Performance budget** (CI gates, not aspirations): full-screen 300×80 composition < 1 ms; typical
  damage-tracked frame < 100 µs; 60 fps steady state < 5% of a core; zero allocations during frame
  composition; a genuinely idle application costs zero wakeups. A budget figure may not move without
  a new map decision.

## CI

Two runners, and the split is deliberate. **The gate set is `.gitlab-ci.yml`** — four jobs (`test`,
`deny`, `budget`, `idle`) on a shared local GitLab at <http://gitlab.localhost:8940>, project
`repos/vitui`, started with `devkit up`. **`.github/workflows/` holds what a local runner cannot do**:
`ci.yml` for the macOS/Linux matrix, `soak.yml` for the weekly fuzz soak, and `compare.yml` for the
monthly comparative suite on a pinned runner where every arm builds. Both scheduled workflows
*upload* their report and never push one. (`compare.yml` invoked `compare/run.sh`, which ticket 20
found had never been committed — the job had been failing at its Measure step since the day it was
written, which is what a scheduled workflow nobody watches buys.)

```sh
devkit up                                  # start it (or bring it to spec) — the only mutating verb
devkit down                                # stop it, keeping everything
. ~/.local/state/devkit/env                # GITLAB_HOST, DEVKIT_TOKENS, DEVKIT_GROUP
```

The devkit runner has **no default job image**: it serves every repo, so `default: image:` in
`.gitlab-ci.yml` is required, not decorative. Six concurrent slots, shared with every other repo.

**A commit is not the end of a ticket.** Push to `devkit` and watch every job go green before
reporting the ticket done.

## Working a backlog

`docs/agents/issue-tracker.md` is the full convention. In short: one ticket per session; the
**frontier** is the lowest-numbered file that is unblocked and unclaimed, and the `Blocked by:` line
is the authority — the number only breaks ties. Claim by setting `Status: claimed` before any work;
resolve by appending an `## Answer` section, setting `Status: resolved`, and adding a one-line
pointer to the map's Decisions-so-far. Research findings go in `research/` beside the issues.

**`.scratch/vitui-runtime-impl/` is closed** — all 21 tickets resolved, the last on 2026-08-24.
Build order across the repo is **engine → runtime → components**, and the runtime was *not* a queue
behind the engine: several of its tickets name single engine tickets and ran beside them. The next
backlog is the components' 43 sliced tickets, and the engine's production-readiness backlog
un-pauses now that a consumer exists — see the note below.

**The production-readiness backlog is paused as of 2026-08-23**, with 04, 07, 08 and 09 marked so in
their own files and the reason in `.scratch/vitui-engine-production/README.md`: nothing above the
engine can draw a screen yet, and those tickets get sharper once a real consumer exists rather than
harder. **08 is superseded** — the runtime is the caller it wanted, and a better one, so the small
engine-only application it specifies does not get built. Work returns there when the runtime can put
a frame on a terminal.

`tickets/` at the repo root is a **separate** surface — the hand-written backlog the `dispatch` skill
consumes — and holds the two items that need the finished library. Do not migrate one into the other.
