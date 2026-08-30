# CLAUDE.md

Guidance for Claude Code working in this repository.

> **The local GitLab moved out of this repo (2026-08-22).** One shared instance serves every repo on
> this machine: <http://gitlab.localhost:8940>, project `repos/vitui`, started with `devkit up`. The
> `infra/` stack here is gone — see `infra/MOVED.md` and `~/Projects/devkit/README.md`.

## What this is

`vitui` is a Rust TUI library: fast, layered terminal rendering, meant to be the foundation a
component library stands on. Version `0.0.0`, unpublished, no stability promise before 0.x.
MSRV **1.88** — `Cargo.toml` is the authority and the `msrv` CI job is what keeps it honest.

**Read these before working, in this order:**

1. The spec for the layer — `.scratch/vitui-{engine,runtime,components}-architecture/spec.md`. All
   three maps are **closed**; the specs are the authority. An `architecture.md` beside a spec is the
   superseded proposal, kept only as the record of what was argued.
2. `CONTEXT.md` — the glossary. Use its terms in code, comments, tickets and commit messages.
3. `docs/adr/` — 52 decisions that are hard to reverse and surprising without context. 0001–0011,
   0022–0025 and 0052 are the engine, 0012–0021 and 0034 the runtime, 0026–0033 and 0035–0051 the
   components.
4. The impl backlog `README.md` for that layer — phase order, blocking edges, and the defects that
   shaped both. **Per-ticket findings are not restated here**: they live in each backlog's
   `research/`, in the tickets' `## Answer` sections, and in the ADRs.

## Where the build is

- **`vitui-engine` — implementation-complete.** 26 impl tickets, 30 verification-register entries,
  none pinned red, ~30k lines. Production readiness added `conform/` — the only instrument that asks
  a real terminal rather than our model of one, and the source of `quirks.rs`'s later entries.
- **`vitui-runtime` — implementation-complete.** 21 tickets. `data`, `layout`, `theme` (fourteen
  schemes), `keys`, `ctx`, `id`, `route`, `focus`, `sizing`, `work`, `anim`, `overlay`, `scroll`.
  Register 41 entries and the 20-scene list, both green. The component-facing crate line is *built*
  rather than counted: `crates/vitui-components/tests/crate_line.rs` cannot name the engine.
- **`vitui-components` — implementation-complete.** All 46 tickets; spec §17's freeze is **29 of 29
  built**, as a value (`INVENTORY`) that tests iterate, with the documentation and verification
  obligations as functions over it. Register 233 rows, 220 evaluated, **1 red and pinned** — row
  112, inverted by runtime architecture 31 — beside 6 unreachable across the crate line (ADR 0023)
  and 6 unsubjected; every scene stood up. Obligations **O1–O4, O6 and O7 are `Met`; O5 is the one
  left**, at **14 of 34 `(component, axis)` pairs**, and is watched panicking, because a query with
  no evidence must fail loudly rather than pass.
- **`vitui-apps` — 18 applications**, one file each in `examples/`. A component ticket ships one, and
  since ticket 45 that is obligation **O7** rather than a habit: `vitui_components::consumer` joins
  the freeze against the import paths here.
- **Active work: `.scratch/vitui-engine-production/`**, un-paused 2026-08-29 (it was paused because
  nothing above the engine could draw a screen; eighteen applications ended that). **09 is the only
  ticket left** and it needs a Windows machine. **04 is resolved** (2026-08-30, the Terminal.app arm,
  four emulator families), **07 is resolved** (the terminal leaves and comes back; ADR 0052), **08 is
  superseded** (the runtime is the caller it wanted, and a better one), **12 is resolved**
  (2026-08-30): `detect::batch`'s first bytes are `?1049h`, so the page is opened by the act of
  asking and Terminal.app 2.15's echo of `+q524742` and seven `p`s lands on a page that is discarded.

## Decisions a session must not re-derive

- **Reactivity stays out.** `vitui-signals` was built to price it and then deleted (architecture
  issue 24): three drivers of one screen differ by 0.21% with 0 of 24 000 cells differing, and the
  signal layer's whole per-frame work is 3.60 ns. ADR 0020 was rewritten with those numbers inline.
  No replacement crate, no `signals` feature, no reactivity module — reopen issue 24 instead.
  `vitui_runtime::data::Memo` is the cache that stays.
- **No `unsafe` in any shipped crate** (ADR 0034). `vitui-alloc-probe` is the one stated exemption.
  Overlay bodies are a queue the frame call owns: **n** overlays standing is exactly **n + 1**
  allocations, and the `+ 1` is forced, because a `'f`-bounded body cannot live inside the thing
  borrowed for `'f`.
- **The component surface speaks `vitui_runtime::Rect`.** The `Cells` stand-in is deleted (components
  issue 17) after runtime issue 22 made engine names reachable *by rule* — `crate::line` gates that
  every engine type the surface names is reachable, **and so is every type needed to construct one**.
  A name a consumer can write but not build is a barrier wearing a re-export's clothes.
- **Nothing holds the focus until an application seats it** (issue 25): `if cx.focused().is_none()`
  inside the draw. A runtime that seats the first stop was refused.
- **`Driver::unhandled` is read *after* the frame**, never before — it is a window onto the same
  queue, valid until the next frame begins. A source scan keeps all five loops in that order.
- **A printable quit key is not always available**: a focused `field` consumes every text-bearing key
  and a focused `collection` eats letters into its type-ahead. Bind `Ctrl+Q` beside `q`.
- **A component may own a time *anchor* and may not own a *clock*** (ADR 0051). `Ctx::now` is the
  only clock; a self-sampling component is invisible to `Driver::pin_clock`, which is the entire test
  regime of this workspace. Stored state may be an anchor, never a phase.
- **Identity**: a container that returns a rectangle preserves its children's identity; one that takes
  a closure renames them (`scope`/`scroll_scope` excepted). A container keying children mints their
  ids arithmetically (`Id::keyed`), because `Ctx::id` is `Location::caller()`.
- **A component names a `Role`, never a colour** (ADR 0018); the one constructor that takes colours is
  on the `Theme` and carries no tier guarantee.
- **Budgets are per class** — typical damage-tracked frame < 100 µs, full-screen 300×80 < 1 ms.
  An over-budget screen is recorded beside the number, never reclassified to buy headroom.
- **The alternate screen is entered exactly once, by whoever speaks to the terminal first** (spec §7,
  production 12). On a real terminal that is `detect::batch`, whose first bytes are `?1049h`: a
  terminal is not obliged to *ignore* a sequence it does not implement — Terminal.app 2.15 prints one
  — and no instrument inside the engine can represent a terminal that does, because a model that
  ignores the unimplemented is a correct model. `actuate::Page` is what stops the negotiation entering
  it a second time; `?1049h` twice restores the shell's cursor to the alternate screen's origin on a
  terminal without xterm's guard. A terminal with no alt screen is probed on its only page and that is
  stated rather than mitigated — §15 puts inline rendering out of scope.
- **Bars are reserved, never overlaid** (ADR 0029) — the parts of a reserved area tile the rectangle
  exactly, which an overlay bar cannot satisfy.

**Open questions — do not "fix" code to match one sentence of a spec without resolving the ticket.**
Twelve stand open, every one of them filed by the layer above the one it lands in.

- **Runtime architecture 26 and its corroboration** — `Ctx::scroll_scope` translates the content the
  wrong way, and the corroboration is C03's inverted sign inside the runtime's own verb. **28** — a
  chord on a shifted character cannot match, because SHIFT is in `INTENT`. **29** — `Response`
  publishes the press as a level and never as an edge. **31** — `Ctx::with_key` inside a scroll
  scope clips the whole window away, and it is what pins register row 112 red. **33** — a
  scroll-into-view is a two-frame gesture and nothing asks for the second frame. **34** — a picture
  cannot ask what the terminal will show: no colour-pair question and no readable wire. **35** — an
  application cannot give its terminal to an editor, because `Screen::suspend` now exists (ADR 0052)
  and `Driver` owns it privately — the third *engine verb behind `Driver`'s private field* after 23
  and 30.
- **Components architecture 19** (does a fold that costs the volume belong to O6), **20** (`tree`
  declares three glyphs it cannot draw), **22** (`Esc` over a plain `collection` is crate-private on
  purpose), **23** (`file_picker`'s popup has no keyboard at all).

**The engine's architecture map has none left.** Issues 20, 21 and 23 read as open for a while and
were closed by production tickets 06, 03 and 02 — do not re-file them.

## Traps this map has met more than once

The verification machinery is architecture, and most of the defects found here were in the
instrument rather than in the code. Each of these has bitten at least twice.

- **A scanner looking for a literal contains that literal.** Assemble needles from fragments.
- **A needle is a join, and a bare name is not one.** `pub fn x(` never matches `pub fn x<T>(` — the
  boundary is `(` **or** `<` — and `keys::text` is not `text::text`, so join through the module the
  freeze homes a component in.
- **`cargo test` does not run an example.** An `assert!` in `examples/*_numbers.rs` is compiled by
  `cargo clippy --all-targets` and evaluated by nobody; several rotted for eight tickets. No register
  row may rest on one.
- **A gate that cannot fail.** Reading a baseline *after* the call it measures compares a value with
  itself; an equality between two derivations of one declaration holds for ever; an equality between
  two things that do not exist holds. Watch every gate failing — and since `warnings = "deny"` turns
  an unused `pub(crate)` into a build failure, empty the body rather than deleting the call.
- **A `compile_fail` needs a twin naming the protected item by path.** The error code beside the fence
  is documentation: rustdoc on stable ignores it.
- **The recorder and the defect share a coordinate system**, so the gate cannot see it. Both recorders
  union in root coordinates for this reason.
- **Counters on the wrong side of the question.** `changed > 0` is green on the exact set it exists to
  catch; every *output* counter is blind to work that produces no output (a fold at 476 ns a point
  drew the identical picture) — pair a work counter with a per-input ceiling.
- **Cumulative ledgers must be read as deltas**, and allocation windows counted **per frame** and
  warmed on the *shape* rather than on two identical frames.
- **A figure quoted from a spec is usually a prototype's screen.** Measure it; assert what reproduces
  as measured, print what does not beside it, and never bend the code to make an old sentence true.
- **`#[track_caller]` forwards into every `#[track_caller]` function it calls** and through nothing
  else — so a private body one frame down mints the caller's id, and two call sites of one component
  are two widgets.

## Workspace

```
crates/vitui-engine       cells, surfaces, layers, compositing, damage, serializer, frame writer
                          └ crossterm behind a seam: raw mode, input, capability detection
crates/vitui-runtime      layout, identity, focus, hit-testing, routing, key maps, theming,
                          overlays, the data contract — no scene tree, no reactivity
crates/vitui-components   windows, panels, charts, lists, trees, forms, pickers — 29 of 29 built
                          └ `gallery`: 29 panels as a value, where §21's rows 7 and 8 are measured
                          └ per component: a doc page with a compiled example (O1), a golden screen
                            per construction under `tests/golden/` (O3, 36 of them, `VITUI_BLESS=1`),
                            a declared keyboard contract for the thirteen that read a key (O4), and
                            an application that imports it (O7)
                          └ `media` is **no row of the freeze at all** — §14's *no v1 component*, so
                            the family ships and `MEMBERS` is empty
crates/vitui              facade re-export — engine, runtime, components
crates/vitui-apps         18 applications, one file each in `examples/`: counter, triage, latency,
                          ledger, explorer, reader, settings, compose, console, theatre, browse,
                          mixer, vitals, roster, gallery, sheet, pipeline, caps.
                          The surface's only consumer, and repeatedly the thing that found the defect
                          the gates could not — a gate exercises a component where its author put it
                          and an application puts it somewhere else
                          └ `caps` draws no frame: attach, read `Capabilities::report`, detach, print.
                            Every gate here is headless, so none can answer *what did my terminal
                            claim* — the question a person holding a broken screen has
                          └ a workspace MEMBER, so CI builds them. Depends on runtime + components
                            and NOT on the `vitui` facade — the facade re-exports the engine, which
                            would make `Rect` nameable here and evaporate the proof
crates/vitui-bench        round-robin minimum-of-N measurement, no deps (publish = false)
crates/vitui-alloc-probe  counting global allocator for the allocation gates (publish = false)
examples/app-template     copy-this-directory starting point, and the home of §11's lint rung
                          └ detached workspace; clippy config does not propagate from a dependency
compare/                  comparative suite: SCENES.md normative, harness.py, run.sh, REPORT.md
                          committed, FINDINGS.md by hand. Nine scenes, five arms, two of them ours
                          └ detached workspace; reports, never gates. No deny.toml, deliberately
conform/                  the only instrument that asks a real terminal: SCENES.md normative, five
                          arms across four examples — Ghostty, Ghostty-via-tmux (the same binary
                          behind `--through-tmux`), tmux, kitty, Terminal.app — one committed
                          REPORT-<arm>.md each, FINDINGS.md by hand
                          └ Terminal.app is the fourth VT lineage and **the arm that disagrees**:
                            four of scene 05's twelve surveyed rows, all four by summing a cluster's
                            code points. Its capture surface carries no style at all, so scene 01 is
                            eleven `cannot ask` rows and scenes 05 and 06 — asked in band — are
                            answered in full
                          └ four scenes, two of them not photographs: 05 asks the emulator's own
                            UAX #11 verdict via CSI 6n (twelve rows are a survey and never fail),
                            06 polls mode 2026 via DECRPM with five compared rows — and a terminal
                            that answers *nothing* is `ModeError::Unanswered` and `cannot express`,
                            never a short batch
                          └ detached workspace, no deny.toml — the third-party thing IS the subject.
                            Live arms are soaks; the gate is `cargo test` over fixtures/
fuzz/                     two libFuzzer targets and the committed corpus that is their gate
                          └ detached workspace: nightly + libfuzzer-sys, which the engine's
                            dependency policy will not have. A loophole, not a permission
scripts/                  the five gates and reports that cannot be a `cargo test`
```

## Commands

```bash
cargo build --workspace
cargo test --workspace -- --test-threads=1  # the real invocation; allocation gates need one thread
cargo test -p vitui-engine name_substring   # single test
cargo clippy --workspace --all-targets
cargo clippy -p vitui-engine --all-targets --features fuzz   # the config `cargo test` misses
cargo fmt --all
cargo doc --workspace --no-deps             # a gate: a broken intra-doc link fails the job
cargo deny check                            # needs `cargo install cargo-deny`
(cd fuzz && cargo deny check)               # detached workspace: its own graph, its own gate
(cd conform && cargo test)                  # the conformance gate, over committed captures
(cd conform && cargo run --example tmux)    # the one live arm that is headless
(cd conform && cargo run --example terminal) # needs an AppleScript grant for Terminal.app
```

Applications (`cargo run -p vitui-apps --example NAME`), each with the key worth pressing; those
marked `--probe` print one headless frame and what it cost:

| app | what it shows |
|---|---|
| `counter` | the first application; `q` to quit |
| `console` | the overlay family; `Ctrl+P` palette, `Ctrl+Q` quit |
| `theatre` | the media family; `4` is the floor of the colour axis (`--probe`) |
| `browse` | the preview pane; `k` then `s` is the memo-key rule (`--probe`) |
| `mixer` | the slider; `x` fifty steps, `f` swaps the arithmetic (`--probe`) |
| `vitals` | the six Tier 2 rows; `g` steps the glyph rung (`--probe`) |
| `roster` | the Tier 2 composites; `Ctrl+G` takes the arrows away (`--probe`) |
| `sheet` | the caller owns the offset; `t` is the tail, on `tiny.csv` (`--probe`) |
| `pipeline` | the anchor; `c` swaps the cadence, 60 wakes against 431 991 (`--probe`) |
| `gallery` | every built component on one screen; `t` is the key (`--probe`, `--matrix`) |
| `caps` | what THIS terminal answered; draws no frame |

Warnings are denied workspace-wide (`[workspace.lints.rust] warnings = "deny"`), so an enum variant
nothing constructs is a build failure rather than a spare part.

There are **no `cargo bench` targets** — criterion was removed and replaced by `vitui-bench`. Timing
lives in examples that print a report, and every gated or reported number has one home in its crate's
`ledger.rs`:

```bash
cargo run --release --example budget -p vitui-engine     # asserts the gates, prints the numbers
cargo run --release --example layout_numbers -p vitui-runtime      # one of sixteen reports
cargo run --example contract_numbers -p vitui-components           # O4
cargo run --release --example gallery_numbers -p vitui-components  # O2, the matrix, what `t` costs
cargo run --release --example volume_numbers -p vitui-components   # O6
scripts/idle-gate.sh 30       # 0.00 user / 0.00 sys over 30 s; thirty is a floor
scripts/observer-gate.sh      # the debug observer is absent from a release binary
scripts/steady-report.sh      # 60 fps for 30 s against 5% of a core
scripts/lint-rung-gate.sh     # the clippy.toml rung fires in an application, not from a dep
scripts/gallery-panic-gate.sh # the terminal is restored before a panic prints, under a pty
scripts/page-order-gate.sh    # nothing precedes `?1049h` on a real pty — register #30
n=1 cargo test -p vitui-engine golden                 # regenerate; review the git diff
VITUI_BLESS=1 cargo test -p vitui-components golden   # the components' screens; refused in CI
```

The fuzz targets are a **soak, never a gate** — the committed corpus replayed by `cargo test` is the
gate; `fuzz/README.md` is the procedure. On this machine `~/.cargo/bin` must come first on `PATH` or
Homebrew's cargo shadows rustup's and toolchain selection is silently ignored:

```bash
RUSTUP_TOOLCHAIN=nightly cargo fuzz run draw_sequence -- -max_total_time=900
```

## Architecture that takes several files to see

**The frame, as a sequence.** `Engine::new(Config)` → `attach()` on the app thread → `(Screen,
WakeHandle)`. Then per frame: add layers and draw into a `View` with three verbs (`text`, `fill`,
`restyle`), which mark damage as they write; `set_mouse` and `set_cursor`; `present()`. `present`
leases a packet, composites the damaged rectangles bottom-up, packs runs plus their cells, submits.
The render thread takes the packet, serialises against its mirror, writes once, returns the packet.
Damage is marked by the verbs and cleared by `present`, and neither is reachable from outside — which
is why nothing above the engine can force a full repaint.

**`Config::clock` is public API, not a test fixture.** Under `Clock::Manual` both halves run inline on
the calling thread before `present` returns — same mailbox, same packet, same bytes — so a test is a
straight-line program. Threading properties (zero wakeups, the packet never superseded, wake-up
latency) cannot be tested in the mode that removes them and live in the threaded mode as counts.

**The app-thread role is enforced by split handles, not a capability token.** `Screen` and `View` are
`!Send` via a private `PhantomData<*const ()>`; `Parker`/`Unparker` and `Producer`/`Consumer` split so
an unreachable method is simply not on the type. The compiler cannot stop the app thread from being
slow — that is a ~44 ns in-loop overrun detector (`perf.rs`) plus a debug-only observer thread. See
ADR 0003 and spec §11.

**The terminal may leave and come back** (ADR 0052), and three cases wearing one name get three
answers: `Screen::suspend`/`resume` for a terminal given up on purpose, a `Wake::Quit` for one that
went away, a fresh `attach` for one that was replaced. The engine installs **no signal handler and
cannot** — and raw mode is `cfmakeraw`, which clears `ISIG`, so `Ctrl+Z` arrives as a key event while
a `Screen` is attached. A resume owes five things, each a screen that looks perfect while something is
dead: the full repaint, `Mailbox::reopen`, `Actuators::renegotiated`, an owed frame, and
`Perf::observe_again`. The supported shape is *suspend, **stop the process**, resume* — the input
thread parks in a blocking `read` that safe Rust cannot cancel.

**The runtime has no scene tree and no retained structure.** The clip stack is the call stack, the id
path is the closure tree, and what survives one draw is five flat structures rebuilt from the next
draw (ADR 0012). `Ctx<'f, 'v>` carries **two** lifetimes deliberately: with one, `child()` shrinks it
and an overlay body capturing a base-pass local compiles, which deletes the mechanism overlays rest
on. A frame consumes at most one routing edge; there are no per-id inboxes (ADR 0016).

**The verification machinery is itself architecture**, and is the part most likely to be misread as
test scaffolding:

- `register.rs` — the spec's properties as a value, each with its instrument and provenance. A
  property may be *pinned red* with the ticket that will invert it.
- `roundtrip.rs` / `testing.rs` — the primary instrument: composite, serialise, replay the bytes
  through the terminal model, assert the replayed screen equals the frame. It stores nothing, and it
  cannot see a defect the serializer and the model share. A golden *byte string* is refused because
  the encoding is exactly the part allowed to change.
- `reference.rs` — the obviously-correct, far-too-slow compositor. Gate #1 is *generated from it*,
  because a hand-written expectation about damage is written by the person who wrote the damage.
- `scenes.rs` — the scenes as a normative list. Every gate runs over every scene; a gate that picks
  its own scenes tests the scenes.
- `golden.rs` — the residue the round trip cannot reach: the composited picture.
- `ledger.rs` — every gated or reported number has exactly one home, with the machine it was measured
  on. The audit that produced it found one threshold copied into nine files.
- `audit.rs` — the public surface as a value, with counts as gates, and the paired compile-fail
  corpus, each twin naming the protected item **by path**.

## Rules that are decisions, not preferences

Violating any of these silently undoes a decision that cost a session to make.

- **The engine does not lay anything out.** Callers bring rectangles. No layout concept may enter
  through the `Surface` API — that is the door this erodes through. See `docs/adr/0002`.
- **The engine never iterates application data.** It offers clipping and offset viewports; culling is
  the caller's job. Invariant: *frame cost is proportional to visible cells, never to data volume.*
- **crossterm is invisible.** Input and terminal mode only, never output, never in a public signature.
  See `docs/adr/0001`.
- **Damage is marked at write time, not derived by diffing.** Prior art measured ratatui's
  full-buffer diff at ~170 µs on 300×80 — already over the budget for a whole frame.
- **A cell holds an interned grapheme-cluster handle, not a `char`** (UAX #29), and no cell, handle or
  style bit is readable from outside the engine (ADR 0023).
- **No traits in the engine's public surface, and `#![forbid(unsafe_code)]`.** The engine has nothing
  to call upward, so the dependency arrow is enforced by there being no arrow.
- **Dependency policy.** Engine: crossterm plus build-script-generated UCD tables. Runtime: nothing.
  Components: case by case. Enforced by `deny.toml` — where `[bans] deny` bans a crate's *presence in
  the graph*, with `wrappers` as the exception list.
- **A gate is a count, a ratio, an equality or a compile outcome. A timing is a report, and a gate
  only at cliff granularity, with the headroom written next to the number.** A gate tuned to the
  measurement is a flaky test that gets disabled within a month.
- **Performance budget** (CI gates, not aspirations): full-screen 300×80 composition < 1 ms; typical
  damage-tracked frame < 100 µs; 60 fps steady state < 5% of a core; zero allocations during frame
  composition; a genuinely idle application costs zero wakeups. A budget figure may not move without
  a new map decision.

## CI

Two runners, and the split is deliberate. **The gate set is `.gitlab-ci.yml`** — five jobs (`test`,
`msrv`, `deny`, `budget`, `idle`) on the shared local GitLab, project `repos/vitui`, started with
`devkit up`.
**`.github/workflows/` holds what a local runner cannot do**: `ci.yml` for the macOS/Linux matrix,
`soak.yml` for the weekly fuzz soak, `compare.yml` for the monthly comparative suite. Both scheduled
workflows *upload* their report and never push one.

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

Build order across the repo is **engine → runtime → components**, and it was never a queue: several
runtime tickets name single engine tickets and ran beside them. The impl backlogs
(`.scratch/vitui-{engine,runtime,components}-impl/`) are **closed**; the active one is the engine's
production-readiness backlog above.

`tickets/` at the repo root is a **separate** surface — the hand-written backlog the `dispatch` skill
consumes — and holds the items that need the finished library. Do not migrate one into the other.
