# CLAUDE.md

Guidance for Claude Code working in this repository.

> **Anything that names a host, a path or a private repository lives in `CLAUDE.local.md`**, which is
> gitignored. This file is the project's and ships with it; that one is the desk's. If a sentence here
> needs a URL or an absolute path to be actionable, it belongs over there.

## What this is

`vitui` is a Rust TUI library: fast, layered terminal rendering, meant to be the foundation a
component library stands on. Version `0.1.0`, published 2026-09-15 — the `0.0.1` under each of the
four names is the placeholder it replaced — and no stability promise before 0.x.
MSRV **1.88** — `Cargo.toml` is the authority and the `msrv` CI job is what keeps it honest.

**Read these before working, in this order:**

1. The spec for the layer — `docs/spec/{engine,runtime,components}.md`. All three maps are
   **closed**; the specs are the authority. The `architecture.md` that sat beside each spec in the
   map directory is the superseded proposal, kept only as the record of what was argued, and it
   stays with the backlogs rather than shipping here.
2. `CONTEXT.md` — the glossary. Use its terms in code, comments, tickets and commit messages.
3. `docs/adr/` — 53 decisions that are hard to reverse and surprising without context. 0001–0011,
   0022–0025 and 0052 are the engine, 0012–0021, 0034 and 0053 the runtime, 0026–0033 and 0035–0051
   the components.
4. The impl backlog `README.md` for that layer — phase order, blocking edges, and the defects that
   shaped both. **Per-ticket findings are not restated here**: they live in each backlog's
   `research/`, in the tickets' `## Answer` sections, and in the ADRs.

## Where the build is

- **`vitui-engine` — implementation-complete.** 26 impl tickets, 31 verification-register entries,
  none pinned red, ~30k lines. Production readiness added `conform/` — the only instrument that asks
  a real terminal rather than our model of one, and the source of `quirks.rs`'s later entries.
- **`vitui-runtime` — implementation-complete.** 21 tickets. `data`, `layout`, `theme` (fourteen
  schemes), `keys`, `ctx`, `id`, `route`, `focus`, `sizing`, `work`, `anim`, `overlay`, `scroll`.
  Register 50 entries and the 20-scene list, both green. The component-facing crate line is *built*
  rather than counted: `crates/vitui-components/tests/crate_line.rs` cannot name the engine.
- **`vitui-components` — implementation-complete.** All 46 tickets; spec §17's freeze is **29 of 29
  built**, as a value (`INVENTORY`) that tests iterate, with the documentation and verification
  obligations as functions over it. Register **238 rows, 233 evaluated** — 237 and 238 are
  production 18's, the two surface gaps that closed as fields; 235 and 236 are
  production 17's, the keyboard boundary; 234 is components
  architecture 20's, *every entry of §16's twenty is drawn on a named line or recorded as undrawn* —
  and
  **no row pinned red** — row 112 was the last, inverted by runtime architecture 31 — beside
  5 unreachable across the crate line (ADR 0023) and **nothing unsubjected**: production 03 took
  `table`'s two and production 04 took `field`'s four, which were the last. **Forty-five of the
  forty-seven scenes stand up, none is red and two have nothing to run over** —
  `scenes.rs`'s own gate names the numbers, and *every scene stood up* was a third stale summary
  sentence of production 01's kind, corrected by 04. That gate's own **name** has now been stale
  four times and corrected four times — production 06 renamed it from three-and-thirty-three, 08
  from thirty-six, 09 from forty and 07 from forty-three — each time with its five register
  citations, which is the recorded policy: a stale name carrying a note saying so stays, one that
  does not is corrected. **`SCENES`' own doc comment was the same defect one line up**, saying
  thirty-six above an array of forty-five while its next paragraph said the count was a summary and
  never the gate; 07 corrected it, because that note is what the gate is and not a licence for the
  sentence beside it to be false. Scenes **34, 35 and 36 are production 05's**,
  all three in `crates/vitui-components/src/window.rs`: `field`'s scrolled, shrunk and wheeled axes,
  which §21 states over one narrow row and the freeze declares four of. Scenes **37 and 38 are
  production 06's** and they are `table`'s last two, in **two** files rather than one — a shrink is a
  screen (`crate::grid`) and a wheel is an offset (`crate::wheel`), which is where the posted notch
  is driven and where `table` is now the **third** subject beside `collection` and `scroll_area`.
  **Two
  inversions there were the runtime's and no components ticket's**: row 112 and row 161, the bytes
  on the wire, by runtime architecture 34. Scenes **39 to 42 are production 08's** and they are the
  overlay family's last four, all four in **one** file (`crate::dropped`) — because 06's two axes
  were decided by two different things and these four are decided by one: **the list is behind a
  layer**, which changes who can see the arithmetic rather than the arithmetic. Scenes **43, 44 and
  45 are production 09's** and they are the scroll family's last three, in **two** files: the two
  screens are decided by one offset (`crate::surround` — the tail past an area's content and the
  bands beside it), and the wheel is a **posted notch**, so it is `crate::wheel::Subject::Pane`, that
  gate's fourth arm. **All three pairs already had an instrument and not one could fail on its own
  axis** — one frame over a state nothing had moved, a band gate played at offset `(0, 0)` where four
  of five spellings agree, and a wheel gate the pane was not a subject of. Scenes **46 and 47 are
  production 07's** and they are `tree`'s last two and the freeze's last two, in **two** files by
  06's rule — a screen is `crate::forest`, where the component's other two already live, and a wheel
  is `crate::wheel::Subject::Tree`, that gate's fifth arm. **They are the only pairs on this backlog
  whose edge was a decision rather than an instrument**: components architecture 20 was open on
  whether `tree` draws the three indent guides its row declared, and a guide column takes cells from
  the label. **All seven obligations are `Met`** — O1–O4, O6 and O7 since the components backlog,
  **O5 since production 07** — which is seven of seven, nine of nine queries, and **every one of the
  thirty-four `(component, axis)` pairs the freeze declares has a scene**: none is bare, and the
  fourteen that production 05 to 09 took were the pairs *without* one. O5's `#[should_panic]` moved
  from *the evidence is empty* to *the evidence is one row wrong*, the sixth and last query to make
  that move, so nothing here is watched on the vacuity arm any more. **Production 10 is resolved**
  (2026-09-04) and it was paperwork rather than a verdict: two of its five criteria were spent by
  07, and what it owned was the summary test's rename to
  `all_nine_obligation_queries_are_met_and_o5_was_the_last_to_turn` with its two register citations
  and this sentence, the two comments citing a test name that no longer exists, and the components
  README's five per-scenes-ticket entries. **It found a third stale count one file over** — the live
  coverage test's own doc heading read *O5 moves from 34 of 34 to 11 of 34*, a sentence no build
  could make true, left when a search-and-replace moved the first number and not the second — and it
  **measured the population the ticket had assumed was two**: a scan of every `` `…tests::name` ``
  citation in prose across the workspace finds **210, of which 15 sites over 14 distinct names point
  at no function anywhere** (13 sites in components, one each in the engine and the runtime) — **17
  sites over 15 names** counting the two this ticket fixed. That is unowned, because none of them is
  a count and a prose citation is invisible to `cargo doc`; it is the shape a gate would close —
  `gates::declares_a_live_test` is already the needle — and no ticket schedules one. **The two this
  ticket was handed were a rename left behind and not a careless citation**, which a review of the
  ticket's own answer established: the name was declared at `scenes.rs:1710` in `2a1ade2`, so the
  unpaid cost is the one the stale-name policy names. **That review corrected three of 10's own
  sentences and the sharpest is a warning about this policy**: 10's first draft read the two arms as
  a *price comparison* — renamed because it had two citations where the kept one has five — and the
  measurement runs the other way. The renamed name had **two** register rows, the kept one has
  **one**, and **the five are a third test's** (`two_scenes_have_nothing_to_run_over_…`, the one
  actually renamed three times). So the citation count separates nothing here: what keeps a name is
  `PINS_SCENE_22`'s reason — *a register row and the scene list are read together* — and what buys a
  rename is a deferral note that already named the ticket. 10 also left *seven `should_panic` tests*
  standing over nine arms in the file it was opened to de-stale.
- **`vitui-apps` — 21 applications**, one file each in `examples/`. A component ticket ships one, and
  since ticket 45 that is obligation **O7** rather than a habit: `vitui_components::consumer` joins
  the freeze against the import paths here.
- **Active work: `.scratch/vitui-production/`** (opened 2026-09-01) — the whole workspace's road to a
  published crate, **twenty-six tickets in nine groups**: the paperwork, the six unsubjected register rows
  (**all six standing**, by production 03 and 04),
  the fourteen hostile axes O5 still owed (**the group is closed**: `field`'s three taken by
  production 05, `table`'s two by 06, the overlay family's four by 08, the scroll family's three by
  09 and **`tree`'s two by 07**, which turned O5; **10 closed the paperwork behind it**), three
  tier-1 terminals nobody had run (**the group is closed on the macOS side**: WezTerm by 11, Alacritty
  by 12 and **iTerm2 by 13**, which was the last one reachable without Windows, with **14** closing
  the group's paperwork behind them),
  the release, and — added 2026-09-07 — **the two surface gaps the three ports found and nothing
  scheduled**: 17, the keyboard boundary asked about twice, and 18, five option fields two of which
  three separate applications reached for. Both were already recorded in
  `crates/vitui-apps/README.md`'s *What they cannot say* table, and recorded is not scheduled — the
  same sentence this backlog was opened with, one surface over.
  **17 is resolved** (2026-09-08) and **half of it was refused because its premise was false**: the
  in-frame route a container needs over a focused `collection` has existed the whole time
  (`Ctx::scope`, register row 236), and what closed the other half was a decision — a group declares
  its axis, `nav::step` reads it (row 235) — which found a live defect nobody had filed, a pager
  answering `↑` for *the previous page* in a help bar. Both traps below are rewritten.
  **It was not on 15's `Blocked by:` line and that judgement was half wrong in fact**: a widened
  visibility was never what it needed, and what it did need — an argument on `nav::step` and
  `nav::cursor` — is a **breaking** change that was free before the publish and would not have been
  after. **18 is resolved** (2026-09-08) and **its compatibility premise was wrong in the same
  direction**: three of its five rows ship as fields — a panel's bottom-border caption, one
  alignment governing both borders, and `Column::justify` — and a public struct with public fields
  cannot gain one compatibly, which `triage`'s `const INNER: PanelOpts` proved by not compiling. So
  18 belonged before the publish too, and both refusals are findings rather than shrugs: a
  per-segment `Role` on `StatusOpts` would not draw the screen it was asked for, and a file icon has
  no spelling at any rung of the repertoire ladder. Rows 237 and 238.
  A seventh group was added the same day and **19 is resolved** (2026-09-07): the shipped
  documentation cited an ADR, a spec section, a ticket, a register row or a backlog path on
  **5,557 lines**, and it is **zero** now — all four publishable crates are `SWEPT` in both
  populations, rustdoc and ordinary comments, held by `crates/vitui/tests/docs.rs` as an equality
  per crate. The rule is *state the fact, not the pointer*; `docs/adr/`, `.scratch/`, this file,
  `.scratch/agents/`, `CONTEXT.md` and `conform/`'s reports were deliberately untouched, because
  those are what a pointer points at — and ticket 26 then took the pointers out of the three that
  ship, since the directory they point into is not in the public repository. **O1 was `Met` over 29 of 29 and did not prevent any of it** — its
  query is *components with zero doc-tests == 0*, so it asks that a page exists and is silent about
  whether it was written for a reader. **Twenty-four of `vitui-components`' fifty-three public
  modules are now `#[doc(hidden)]`**, chosen by scanning what the applications import, so the
  front page is fourteen families and fifteen helpers rather than fifty-three modules with the
  components buried among them. **20 is resolved** (2026-09-08) and **its premise was wrong in the direction
  that mattered**: it was written as *writing an example is a small design exercise per item* over
  3,391 documented items, and the examples were largely already written — 82 fences the compiler
  sees, **not one labelled**, the twelve headings it counted all being *module* headers. A reader
  landing on `collection` found no Examples section not because nobody wrote one but because the
  fence hung off the end of a paragraph. The population is **two derived halves and no hand-written
  list**: *a fence the compiler sees* (`text`, `ignore` and `compile_fail` are not examples), and
  `INVENTORY` itself for *every component shows how to use it* — the freeze, already what every
  obligation joins on, so a component added to it arrives owing an example and a renamed entry point
  fails on the join. **The two are not substitutes and that is watched both ways**: deleting an
  example leaves the labelling gate green with nothing to look at while the freeze gate fails, which
  is O1-and-O2's argument on a new pair. **The ordering rule is one third gated and two thirds left
  to review**, argued from a measurement rather than a taste: of 709 section headings in the shipped
  crates 68 are `# Panics`, 2 `# Errors`, 0 `# Safety` and ~550 are **narrative**, so a gate over
  section order would push 550 headings towards a vocabulary chosen for a different kind of library
  and the pages would get worse to make it green. What is gated has no vocabulary in it — *a page
  opens by saying what the item does*, never a heading or a fence — and **1,948 of 1,948 already
  did**. **`ignore` is gone from the shipped crates**: both instances were `Ctx::focus` and
  `Ctx::focused` avoiding compilation, and making them run found that an id nothing drew is cleared
  at `end` by the vanish rule — so the shipped example of *seat the focus on the first frame*
  asserted the wrong thing for as long as it was never compiled. The example floor moved 1/1/1/9 →
  1/13/28/56, and **a second ratchet caught the two conversions**: `vitui-runtime`'s own
  `register::RUNNABLE_EXAMPLES` moved 62 → 64 while the engine's `audit::RUNNABLE_EXAMPLES` stayed
  at 50, which is two counters agreeing from opposite directions that a heading is not a fence. That
  constant's doc narrated every step — 58, 59, 60, 61 — above a value of **62**; the missing step was
  19's own front-page example, recovered from `git log -L` rather than guessed, and the same
  paragraph called `Driver::suspend`'s block *giving the terminal to `vi`* four tickets after the
  `$EDITOR` arm was removed. It is `no_run` because it **stops this process**.
  **21 is resolved** (2026-09-08) and it is what a re-check of 19 found outside the gate's
  population: the rule held in exactly what `docs.rs` reads — rustdoc lines and ordinary comments —
  and **a citation written as a string literal was invisible to it.** Six sites a user meets were
  fixed in that re-check session: two engine `.expect` messages ending `(spec §3, ADR 0011)`, the
  `fuzz` oracle's *architecture ticket 20*, `order::Use`'s `section` field, `counters::Reading::
  Unreachable`'s `inverted_by` reaching a caller through a panic, and seven prose citations across
  the root and `vitui-apps` READMEs — one carrying a **stale O5 count**, *32 of 34 pairs and both
  are `tree`'s*, four tickets after 07 turned it. **What 21 settled is the rule underneath, as a
  boundary and not an exemption list**: *provenance is data, and a pointer may live only where a
  reader meets it by asking* — which is `#[cfg(test)]` (offered to nobody) or `#[doc(hidden)]` (the
  crate saying *this is not the surface you are offered*), **read off each crate's own `lib.rs`**
  and never listed in the gate. **Half its premise was false**: the runtime's `ledger` is
  `#[cfg(test)]` and always has been, so the asymmetry is two-way rather than three, and what
  reaches its `docs/adr/` path is the fourteen examples that `#[path]`-include the file.
  `#[cfg(test)]` is **priced and refused** for `vitui-components` — forty-seven of its fifty-three
  modules are named from outside `src/`, over thirty examples and six integration tests, and the
  largest instrument is a 9 893-line register, so `#[path]`-inclusion is thirty copies rather than
  the same arrangement written differently. 39 citing lines in offered modules went to zero, ten of
  them `#[expect]`/`#[allow]` reason strings; `Composition::source` became `Composition::stated` with
  the pointer relocated to `composed::EDGE_SOURCES` and joined by `(of, on)` — **the check it
  replaced could not fail**, since `contains('§')` asks whether a field that cannot be absent
  carries a mark; `obligations::Verdict::inverted_by` went the way `counters`' did, a **breaking**
  change to a public `const fn` at 24 call sites and free only before the publish. The gate gained a
  **third population** — `Standing::literals`, an equality at zero over every line that is neither
  rustdoc nor comment in an offered module, with `mod tests` stepped over; un-hiding `gates` as a
  control takes it to exactly 265 — and a needle now matches only where it **starts a word**,
  because `self.scratch.buf` is the runtime frame's own field and carries the backlog directory's
  name six times in `ctx.rs`. **307 citing lines remain in the shipped rlibs and every one is in a
  module its crate hides**, zero in the twenty-nine a reader is offered. `tests/` and `examples/`
  stay out of scope, confirmed rather than deferred.
  An eighth group is **the instrument** and it is one ticket: **22 is resolved** (2026-09-14) and the
  two allocations a hosted runner reported were the **test harness's own**, landing in the first
  window the process opens — the trap above has the mechanism. A ninth is **the presentation**, added
  2026-09-10 after comparing this repository's first screen with ratatui's, Slint's and Bubble Tea's:
  25 shipped the site and 26 took `.scratch/` out of the public history, **23 is resolved**
  (2026-09-14) — forty-nine pictures, each a composited frame written as SVG — and **24 is resolved**
  (2026-09-15). 24's page was built three days before the ticket was closed behind it, and what was
  left when its criteria were finally checked against the tree was the **truth of the figures it had
  moved**: the performance paragraph was the engine ledger's superseded `was` column row for row
  (≈205 µs against a measured ≈149 µs, 107.3 µs at 40 layers against 43.9 at 50), its largest row is
  a third of the total and is still `Machine::Prototype`, and the comparative section was *five
  described scenes* over a suite that has been **nine scenes and five arms** since runtime impl 20 —
  dropping `unchanged`, the only row with a known right answer, and dropping the `vitui-runtime` arm,
  which is the layer a reader is in. Its CPU and latency figures matched no committed revision of
  `compare/REPORT.md`. **A landing page's numbers go stale in a direction nobody checks**, and here
  three of the four findings were against us.
  **02 is resolved** (2026-09-15) and the repository is public: `vitui-rs/vitui`, 292 commits of
  filtered history, `master` alone, no `.scratch/` in any of them and rebuilt from the commit being
  published rather than reused — 26's clone was eight commits stale and predated `site/`. Three of
  its four criteria were already met by the manifest work of 2026-09-11; the fourth is the README's
  *Where the gates run*, which states that the visible workflows are not the gate set. What the
  publish actually required was the operator's environment leaving the tree: `CLAUDE.md`'s
  machine-specific half is now the gitignored `CLAUDE.local.md`, and **a search for `this machine`
  does not match `the machine`**, **a text search does not read a pixel** — the README's first image
  showed a home directory and a prompt four days after the SVG beside it was regenerated — and **a
  demo filesystem is a document**. Nothing gates the three recordings.
  **15 is resolved** (2026-09-15) and the four crates are on crates.io at 0.1.0, each verified
  against the registry copy of the one below it. Three gates stood in front of the upload and only an
  attempt could reach any of them: `cargo publish --dry-run` had never been run and failed on the
  first crate, because the three `publish = false` crates carried a `version` — which is what puts a
  dev-dependency into a published manifest that cannot resolve it; dropping it made them wildcards,
  which `cargo deny` forbids, rightly for a registry dependency and not for a path one; and
  `allow-wildcard-paths` shares five letters with the allowlist a runtime gate refuses, so **a prefix
  standing in for a concept refused the release**. The other three dry-runs are **impossible rather
  than skipped** — cargo resolves before it packages. The Windows marking **may not name a ticket**,
  because the shipped-docs gate forbids it: the crate strings name the event and `docs/status.md`
  names the ticket, reachable from crates.io only because all four crate READMEs had linked
  `../../` out of their own package. Its five components architecture edges — that map's 19, 22, 23,
  24 and 25, not this backlog's numbers — had all resolved by 2026-09-05, and what they decided is
  listed below.
  **Windows is last, as 16**, blocked by the publish — which carries the consequence that no shipped
  string may claim a terminal the conform suite has not asked. **The workspace itself has run there
  since 2026-09-10** — `cargo test --workspace --no-fail-fast` is green on `windows-latest`, and the
  seven failures on the first run were all the instrument — so *Windows has never been run* is false
  as written and 16 stays open on its other five criteria: the conform arms drive emulators that do
  not exist there, and Windows Terminal is the one supported terminal nobody has driven through the
  suite. Everything on it was already true and
  already recorded; what was missing was that nothing scheduled any of it.
- **`.scratch/vitui-engine-production/` is closed** (2026-09-01), un-paused 2026-08-29 (it was paused
  because nothing above the engine could draw a screen; eighteen applications ended that). **09 left
  for the backlog above**; it needs a Windows machine. **04 is resolved** (2026-08-30, the Terminal.app arm,
  four emulator families), **07 is resolved** (the terminal leaves and comes back; ADR 0052), **08 is
  superseded** (the runtime is the caller it wanted, and a better one), **12 is resolved**
  (2026-08-30): `detect::batch`'s first bytes are `?1049h`, so the page is opened by the act of
  asking and Terminal.app 2.15's echo of `+q524742` and seven `p`s lands on a page that is discarded.
  **13 is resolved** (2026-09-01): the reader is never stopped, the refusal is
  `Screen::suspend`'s **first** paragraph rather than its fifth, and register entry 31 is the
  tripwire.

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
- **The press is published as an edge beside the level** (runtime architecture 29, resolved
  2026-08-30). `Response::pressed` is the grab — true on every frame from the press until the
  release — and `Response::press_began` is the frame the button went down. A gesture that means
  *select what is under the pointer the moment it lands* reads the edge; a plain click cannot tell
  the two apart, because `Gesture::Plain` is idempotent, and only a ctrl-click flickers. There is no
  `CollState::press_edge()` any more and nothing keeps a `pressing` bool: the readers hold the
  `Response`.
- **A reveal asks for its own second frame, and `end` is what asks** (runtime architecture 33,
  resolved 2026-08-31). A scroll-into-view is two frames — `Ctx::request_into_view` leaves the
  request and the offset's owner takes it with `take_into_view` on the next one — and nothing asked
  for that next frame, so `End` drew and parked and the list moved on the *following* keystroke.
  `Frame::resolve_into_view` now asks, because it is the one place the **two** producers meet: the
  explicit request and the ring's keyboard pull, which no application spells and where the explicit
  one overwrites it — one gesture, one wake. Asking at `request_into_view` fixes `End` and leaves
  `Tab` lagging. The gate draws **one** frame per arm and reads `WakeLedger::pending()`, with a
  control arm that parks, because every other instrument here drives its own second frame and so
  supplies the thing under test. The cost is one line in the wake census for **every** reveal in the
  process: a body that asks for a reveal every frame — forbidden by name in
  `vitui_components::scroll`, previously invisible to every counter — is now a runaway, and so is a
  held arrow key, which is honest and is not the fault the detector is consulted about.

- **`Ctx::area` moves with a scroll and not with a clip, and `child` is what resets it** (runtime
  architecture 36, resolved 2026-09-01). `Ctx` carries the scroll translation **alone** — `origin`
  accumulates a clip and a scroll together and nothing on the type could tell them apart — and
  `area()` is `Rect::new(-translation.0, -translation.1, w, h)`: **the rectangle in the coordinate
  system the context is drawing in**, which is the rectangle for a clip and the window for a
  scroll. Issue 31 fixed the two verbs that *childed* at `area()` and left the two that *read* it:
  a `clear` inside a scroll scope painted nothing past the first screenful, and a focused `field`'s
  caret was `Some` at offset 0 and **`None` at 100** — asserted on `field` itself and not only on
  `Ctx::caret`, because a gate exercises a component where its author put it and an application puts
  it somewhere else. **`child` resets it to zero** — a child's own coordinates start at
  its own top-left, so a row placed at content row 100 inside a scope scrolled to 100 answers
  `Rect::new(0, 0, 20, 1)`; a translation carried through the clip would have broken that case,
  which was already right. `View::size`'s decision is untouched, which is why answering
  `visible_rows().start` was refused: a child hanging off the top of the screen has a non-zero
  visible start with no scrolling at all. Register entry 48, and `clear` got a number after the
  ticket said it could not have one — a fill that lands nothing damages nothing, so the arm is
  `Presented::submitted` over a pair of frames that paint one rectangle two colours.

- **A tail exists only where the content is smaller than the viewport, and a band scene may not be
  played at the origin** (production 09). `max_offset` is `extent − viewport`, so at any offset the
  clamp admits `offset + viewport ≤ extent` and the range `[extent, offset + viewport)` is **empty**
  — a vertical tail needs a content shorter than the viewport and a horizontal one a content
  narrower than it. What that buys is the reading: **the shrink clamps the window to the origin**, so
  the window moves at the same time as the content stops reaching it. And `scroll::Shared`'s five
  band spellings all draw the same screen at `(0, 0)`, which is where the gate this crate had was
  played — so `crate::surround` plays two offsets and states the axis in **three** readings, because
  the counts separate only the unpinned arm (the two gutters, 12 cells, since a gutter shares
  *neither* offset), the strings separate all three only at a small offset, and the origin is the
  control. `sticky` has **one** body now — `sticky_shaped` — and `defective::arithmetic_band` is
  that body with one arm.

- **The clamp binding and not the label truncating is what makes a tree's narrow axis visible, and
  a chevron is pressed at the column its own rectangle puts it at** (production 07). §7 ends its
  unclamped-indent paragraph *on a shallow tree the same flag is invisible: a statement about the
  scene list*, which reads as a fact about the **depth** — and scene 8 fixes the width at 300 and
  varies only the depth. It is a fact about the **pair**: at depth ten the two builds are the same
  screen at 300 columns *and* at 40, where the label is already cut from 21 columns to 19 and to 17,
  and they diverge at 22 (the depth-11 rows only, 40 cells over 40 of 80 rows) and at 21 (120 over
  all 80, a parent losing its label cell and a leaf losing everything). **No write counter can see
  this axis at any width** — both arms write `w × 80` at every one of the four, because the clip eats
  exactly what the collapse loses — and `verbs` moves *downward* on the defect. And the third
  reading changes **no cell**: `tree` draws the chevron at the indent of the **row** rectangle and
  decides a press from the indent of the rectangle it was **handed**, two calls to `indent_columns`
  agreeing because `collection` gives a row the whole of `area.w`. `min(2d, w − 2)` is one number
  for two widths whenever the clamp binds on neither, so the spellings differ **only** at 21, where
  they are 19 and 20.

- **A component may ask about two colours, and a consumer may configure its own engine** (runtime
  architecture 34, resolved 2026-08-31). `Theme::colours_differ_on_wire(Rgb, Rgb)` is
  `roles_differ_on_wire` asked of two colours instead of two roles — `const`, no index published, so
  ADR 0007 is untouched — and it is what discharges `Theme::custom`'s stated obligation, which had
  no verb behind it for four tickets. And `Clock`, `Output`, `Overrides`, `WidthSource` and
  `InputConfig` are re-exported, which closes issue 22's rule on `Config` itself: *every type needed
  to **construct** one the surface accepts is reachable.* Five names and not three, because
  `Overrides` carries a width source and `Config` carries an input config. **The tier is pinned on
  `Config::overrides` and never on the theme alone** — `Theme::resolve(tier)` narrows the thirteen
  roles and says nothing about what the engine quantises a `custom` paint into. What it bought:
  components register row 161 runs, and a whole-row translation that changes **24 000 of 24 000
  cells** costs **1.25% of a full repaint**, because the engine prices the rows the shift exposed.
  Byte *totals* are reported and never gated — a byte count is one step from a golden byte string,
  and the encoding is exactly the part allowed to change.

- **An engine verb an application needs is forwarded unchanged, and `Driver` is the door** (runtime
  architecture 35, resolved 2026-08-31, spec §4). `Driver::suspend` and `Driver::resume` are the
  **third** instance after 23's `wait` and 30's `permit_slow`, so the shape is now a rule beside
  issue 22's rule about *types*: that one is gated in both directions, this one has no instrument at
  all, and all three were found by writing an application rather than by reading the surface. Both
  bodies are one line — there is no policy to add on this side, and the runtime's own per-frame state
  is untouched because **no frame runs inside a suspension**. `ENGINE_NAMES` is unmoved at
  thirty-five, the first forward here needing no re-export. **How an application stops itself stays
  the application's**: `vitui-runtime` depends on nothing, so `console` shells out to `kill -TSTP`
  — `-TSTP` and not `-STOP`, because `SIGSTOP` cannot be caught by the shell's job control and `fg`
  would not know the process exists. **A suspension does not quiet the reader**, which is the half
  the ticket's brief got wrong and a review caught: `vitui-pty` is an unconditional
  `loop { stdin.read(..) }` that safe Rust cannot cancel, so *stopping* works — `SIGTSTP` stops
  every thread — and *handing the terminal to an interactive child while this process runs* does
  not, because two readers on one tty split the keystrokes. A `Ctrl+E` arm for `$EDITOR` was
  written, reviewed and removed; the engine's own `Screen::suspend` doc says so in advance.

- **The reader is never stopped, and the refusal is the first paragraph** (production 13, resolved
  2026-09-01). `Stdin::read` has no timeout and no cancellation, and the four mechanisms that would
  give it one are all refused: a self-pipe and `poll` and a non-blocking descriptor both need `libc`
  and an `unsafe` block, **crossterm's own cancellable event source cannot carry the capability
  negotiation** — its `Event` has no variant for an unrecognised escape sequence and the whole batch
  is such sequences — and `rustix` would need no `unsafe` here and is refused on dependency policy,
  which is a judgement and is stated as one. So an interactive child in this terminal is a case the
  pair **does not serve** rather than one with a condition on it, and that sentence moved from
  `suspend`'s fifth paragraph to its first, because the fifth is where issue 35 read it as a caveat.
  `resume`'s discard is untouched. **Register entry 31 and `scripts/suspend-reader-gate.sh`**: two
  keystrokes on a real pty, one before the suspension and one during it, because four documents
  claimed this and nothing watched it — and no instrument inside the crate can, since `Tty::open`
  panics under `cfg(test)` and the thread the claim is about is never spawned by a test here. It is
  a **tripwire**: it goes red if the reader ever learns to stop.

- **Three counts sat on the denominator *seven tier-1 terminals* and nothing separated them**
  (production 14). §15's out-of-scope entry said **five have never been run** where one has —
  Windows Terminal, owned by ticket 16 — `quirks.rs` says **six run**, and §10's `legacy_sgr`
  paragraph says **two observed**, which is *correct* and counts a third thing: the **colon SGR
  spelling**, whose scene (`conform/SCENES.md` 03) is proposed and unbuilt. All three now name what
  they count, because two figures differing by four on one denominator are a defect report unless the
  reason is beside them. *Terminals run* is also not *attribute facts answered*: the scene asks
  **eleven per terminal** and **five rows across three of the six arms are `cannot ask`**. **The
  quirk table itself listed seven entries while `lookup` returned eight** — iTerm2 had a section, an
  arm, a fixture and a report and no row — so
  `crate::gates::the_quirk_tables_prose_is_joined_to_its_entries` joins four statements about that
  number: the table's rows, `lookup`'s arms counted **inside that function's body**, the deepest
  `# The nth entry` section, and the refusal heading, which must be one past the last. Over **how
  many** and never **which**: the doc's ordinals are arrival order and `lookup`'s are a precedence
  argument. It is not a register entry, for the reason `the_detector_reaches_for_nothing_in_the_crate`
  is not. **The ConPTY entry is the one row no run here has ever touched** and says so, pointing at
  16. **A fourth count was wrong wherever §10's default is argued for**: four of the eight entries
  force the legacy SGR spelling and **eight sentences across three files said three** — `caps.rs`,
  `serial.rs` and the spec — the sixth entry having arrived without them. §10's argument that the
  colon spelling is the default *is* that number (*a one-way field N entries force*), so it now has
  one home in `quirks.rs` and is the gate's fifth comparison.

- **A `cannot ask` becomes a quirk only when evidence exists outside the capture, and a real
  misbehaviour with no route around it is no entry at all** (production 11). Two rows of `conform/`'s
  WezTerm arm come back bare and **cannot** be promoted the way kitty's two were: a dump cannot tell
  *not stored* from *not serialised*, kitty was settled by its shipped `.so`'s `Cursor` repr, and
  WezTerm has no far side (it is an endpoint) and no second source (its binary's string table holds
  `OVERLINE` as a Unicode character name, so a `grep -c` returns noise). Separately, WezTerm reports
  **mode 2026 reset while it is set** — a real misbehaviour, with three control probes ruling out
  every innocent reading — and it earns nothing either, because **every row of `quirks.rs` is a route
  the serializer can take around a defect** and `Detected::mode` reads `1` and `2` alike as
  *available*, so `sync_output` is true and nothing is degraded. It is the second thing that table
  records as deliberately not an entry. Two parser gaps came out of the same arm and neither grew
  `Dialect`: `ESC ( B` is three bytes (the two-byte fallback left the `B` as content and eleven rows
  read `Bbold`), and **ECMA-48's SGR 21 is *doubly underlined***, not bold-off.

- **A capture surface is established by running the terminal, not by reading what it offers**
  (production 12). The brief said Alacritty had none — no socket, no dump, no AppleScript — and told
  the session to establish that rather than assume it. **`alacritty --ref-test`** writes the `Term`'s
  own grid as JSON when its last window closes, one object per cell, so it is the **strongest**
  capture surface here: no serialiser of the emulator's stands between the cell and the reader, which
  makes kitty's and WezTerm's `cannot ask` structurally impossible and makes this the first arm that
  could be *asked* about a dotted underline. What it costs is stated rather than mitigated — a grid
  is what the terminal **stores** — which is why its two bare rows are argued from `Flags` having no
  bit and from `alacritty -vvv` printing `Term got unhandled attr: BlinkSlow` for one and nothing at
  all for the other. That is `quirks.rs`'s **seventh** entry, recognised by `$ALACRITTY_WINDOW_ID`
  because there is no XTVERSION and `TERM` is `xterm-256color` where no `alacritty` terminfo exists.
  Three shapes in `conform/src/grid.rs` are quotations rather than derivations and the first would
  have been silent: `Storage` is a **ring**, so `inner[0]` is the bottom row and a reader taking the
  array in order hands back every row's text intact and every row in the wrong place.

- **A terminal whose DECRQM never says *set* is a third cause of `AlreadyReset`, and part B then
  measures the reply** (production 12). Alacritty's `Term::report_private_mode` answers mode 2026
  with a **constant** `ModeState::Reset` while synchronised output lives one crate down in `vte`'s
  parser — the flag and the reporter in different layers — so it is the second family after WezTerm
  to report the mode reset while it is set, and it earns no quirk row for WezTerm's reason. What is
  left to measure is the **reply**: a DECRQM written 50 ms into an open block came back at 150, 151
  and 171 ms over three runs against `vte`'s shipped 150 ms `SYNC_UPDATE_TIMEOUT`, which is the
  **first measurement beside a row of the force-flush table** — still not the paint, and the arming
  instant still unobservable from inside. `Bracket::AlreadyReset`'s two documented causes were both
  false here, and its documentation now names the third.

- **A crate's `description` is its README's first sentence, and a gate says so** (production 01,
  resolved 2026-09-01). Three places say what a crate is before a stranger reads a line of its code
  — the manifest's `description`, the README's opening paragraph, the rustdoc's first line — and
  four publishable crates make twelve sentences that nothing had ever compared against anything. The
  facade's three called this library reactive. `crates/vitui/tests/blurb.rs` is the gate and it is
  **the facade's**, because no one of the four can see the other three; the four are derived from
  the workspace members by *does this manifest say `publish = false`*. Its half with teeth is the
  **equality** — a README must open with its manifest's `description` — because a vocabulary scan
  only catches a sentence coming back. **Do not reword one of the twelve without the other two**,
  and note what the gate does *not* reach: a false sentence outside a `## Status` section.

- **An overlay body draws through the `Ink` seam and a `Ctx::overlay` body still cannot capture
  one** (production 08). `input::popup_body` and `files::picker_body` are generic over `I: Ink`, so
  the shipped drawing of a `select`'s popup and a `file_picker`'s listing is what a
  `crate::runner::Pen` sees — but only when a caller invokes those two functions **in the base
  pass**, because a body is `FnMut(&mut Ctx<'f, '_>) + 'f` and a `&mut I` borrowed for the owner's
  call cannot travel into one (spec §1's fifth component). That base-pass call is
  `crate::popup`'s named substitution and it is on **both arms** of every comparison, so it is on
  the side of neither. What it replaced was worse: `popup::popup_cells_into` is *the same two orders
  written where a `Pen` can see them*, which is a copy, and `crate::ink`'s rule is that a gate
  written against a copy tests the copy. It is still there because `FILL_FIRST` is scene 14's number;
  **it is now unnecessary** and a later ticket may retake that figure over the shipped body.

- **A popup's wheel needs two opening frames and `crate::wheel::Subject::ALL` stays at three**
  (production 08). A notch is resolved against the **previous** frame's hit index and a layer's
  entries only enter it once the layer has been placed, so a run over an overlay owner opens with a
  pointer position and two frames where that drive loop opens with one. A wheel gate is therefore
  **not one drive loop**: `crate::obligations`' wheel-pair join is over three subject lists —
  `wheel::Subject::ALL`, `window::WHEELED_SUBJECTS` and `dropped::WHEELED_SUBJECTS` — because a
  notch's cadence is a property of *what consumes it*, and a component reading `Response::scrolled`
  in its own draw, a base-pass subject and a body inside a layer are three cadences.

- **A reserved bar's thumb is a function of the extent and the offset it is handed is a literal
  zero** (production 08). `overlay`'s shell builds `Span { viewport, extent: rows, offset: 0 }`, so a
  popup's bar does **not** move as its list scrolls — and what a tail-cut reference render cannot
  share is the **row count**: the thumb is 9 rows of 24 over 64 rows and 18 over 32, which is 9 cells
  over 9 rows on the **correct** arm, all in one column. A scene over such an oracle compares the
  **interior** (`Canvas::cropped`, so no second `diff` exists) and **reports the excluded number
  beside it**: an exclusion a reader is told about is owed its measurement. **The first record of
  this said *offset* and a review caught it** — the two readings send a session to different places,
  because aligning the two arms' offsets changes nothing and the extent is the only lever.

- **A projection is not a grid, and a quirk's second source may be an absence** (production 13).
  iTerm2's Python API answers `GetBufferRequest` with a `CellStyle` per run of cells — no serialiser
  in the path, which is Alacritty's property — but a `CellStyle` is a **projection** of
  `screen_char_t` rather than the struct, so *the projection dropped it* and *the cell never had it*
  look identical, which is WezTerm's position. The three unanswered rows of scene 01 split **one and
  two** across exactly that line. Overline is the cell's and earns `quirks.rs`'s **eighth** entry,
  recognised by XTVERSION (`iTerm2 3.6.11`) — the evidence is an **absence in two places**: the type
  encoding enumerates fourteen bit fields and eleven spare bits with no overline, and the string
  occurs zero times in the binary. The double and dotted underlines are `cannot ask`, three bits
  projected to a `bool`. **The dotted underline now has all four readings** — spelled wrongly
  (kitty), unspelled (WezTerm), stored as a bit (Alacritty), projected away (iTerm2). Three further
  things the arm found: it is the **only** surface here that tells a cell the terminal *emptied*
  (`U+0000`) from a space somebody wrote (`U+0020`), projected away on purpose with a test asserting
  both halves; a reversed cell's colours come back **already swapped**, so `REVERSED_DEFAULT` is not
  a renderer resolution and a *coloured* reversed cell is refused rather than un-swapped; and it
  splits `vs16` from `keycap` the other way round from Terminal.app, which retires *none of them
  widens a VS16 emoji* and shows `zero-width` was never a summer's symptom. Spec §10's
  `terminal-light` citation was checked and **does not reproduce as a slow terminal**: the whole
  attach in 73, 94 and 80 ms against a 250 ms ceiling.

- **A key release is never routed, and a frame that decides something asks for the frame that draws
  it** (2026-09-07, register entries 49 and 50). Both are one keystroke away from every application
  and both are recorded in full under *Traps* below. In short: `Frame::begin` drops
  `KeyKind::Release` — kitty flag 2 reports both edges and a release is intent for nothing this
  crate routes — and `Frame::resolve_award` calls `wants_another_frame` when the pointer award
  carries a press, a release, a click or a cancelled drag, or when the focus ends the frame
  somewhere other than it started. Do not "simplify" either by moving the filter into the readers or
  the ask into `Driver::frame`: the first is the one place a key enters a frame from both doors, and
  the second is the one place that knows what `end` decided.
- **A panel's two border captions are arguments and its one alignment is a field, and a `Column`
  places its heading and its cells from one declaration** (production 18, resolved 2026-09-08).
  `panel_with(cx, area, title, footer, opts)` — the bottom border's caption is an **argument** beside
  the title, because a caption is per-frame data and an options struct is configuration, and because
  a `&str` on `PanelOpts` costs the type the lifetime-free `Copy` an application uses to write one as
  a `const`. `PanelOpts::justify` is **one** value for both borders and that is measured, not
  preferred: of the four applications that asked, two centre both captions, two start both, and none
  wants them to disagree — a second field buys only the screen nobody drew. `Column::justify` (with
  `Column::aligned`) is the one row a single application carries, and what carries it is the
  asymmetry — a heading placed by the component and cells placed by the caller's closure, with no
  field either could read, which is a wrong screen out of correct code. **Two rows of the same table
  are refused and the refusals are findings**: a per-segment `Role` on `StatusOpts` would not draw
  `mc`'s function bar, because that bar paints two roles *inside* one segment and a segment split in
  two draws a separator between the halves; and a file icon is not a `Glyph`, because a `Glyph` has a
  spelling at every rung and a private-use codepoint has none. Rows 237 and 238.
- **A picture is a frame, and the writer lives where the composited frame is** (production 23).
  `Screen::to_svg` walks the composited surface, resolves each handle through the layer stack's
  tables and each colour through the terminal's capabilities, and writes SVG — a **third** reader of
  the picture the reference compositor produces and the goldens compare, beside them rather than a
  copy of them. It is a shipped verb and not `cfg(test)`, and ADR 0023 is untouched by it: what
  leaves is a picture, never a cell, a handle or a style word. `Driver::to_svg` forwards it, which is
  the **fourth** instance of *an engine verb an application needs is forwarded unchanged*. Three
  properties do not survive and each says why — blink (a picture is one instant), a hyperlink (a
  target is a place to go), conceal (it survives by the glyph not being drawn). A colour the terminal
  never answered for is a **stated** pair rather than a guess, so the same frame is the same file on
  every machine. Forty-nine pictures live in `docs/img/{components,apps}/`: the components' are gated the
  way a golden screen is, and the applications' cannot be — `cargo test` does not run an example — so
  `scripts/shots.sh --check` re-takes all twenty and diffs, in both CI files. **One is excluded and
  the exclusion is joined to its reason**: `latency` draws a measured duration, and a timing is a
  report rather than an equality.
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
- **The freeze's `glyphs` column is what a component draws, and `tree` draws no indent guide**
  (components architecture 20, resolved 2026-09-04). `VLine`, `TeeLeft` and `BottomLeft` stood in
  `INVENTORY`'s `tree` row for four tickets and were drawn by nothing: a guide column at depth *d* is
  a fact about *d* ancestors, and **four** routes to it are refused — a per-row ancestor walk, a
  fifth field wider than §7's eight bytes, `VLine` repeated (wrong under an ended subtree), and one
  `O(n)` reverse pass whose output is that fifth field stored elsewhere and which cannot be cut to
  the viewport, because a visible row's guide is a fact about rows below it. The caller's free
  `flags` bits do not buy it: seeding a forward walk's stack at the first visible row is a backward
  scan to the nearest depth-0 row. `select`'s stepper `ArrowUp` went with them — a popup's gutter is
  `scroll::bar_into`, no caps — and its `ArrowDown` stays, drawn as the chevron. The old tripwire
  joined a **declaration to a declaration**, which is why it could not see this;
  `crate::glyphs::DRAWERS`/`UNDRAWN` partition §16's twenty into **15 drawn on a named line and 5
  not**, the needle assembled from the entry rather than written beside it. The five left are
  `table`'s and are one family (architecture 25).
- **`SHIFT` is intent for a key and not for a character** (ADR 0053, runtime architecture 28, resolved
  2026-08-30). `Chord::typed(c)` compares `keys::TYPED_INTENT`, the five intent bits that are not
  `SHIFT`, and is what to bind on `+`, `?`, `:` or `_` — a US-layout `+` cannot be typed without
  shift, so on a character the bit is *how it was produced* and not intent. `Chord::key` still
  compares all six, so `key('a').shift()` and `Shift+Tab` are unchanged, and `typed(c).shift()` is a
  no-op that is asserted rather than documented. One keystroke has **four** wire spellings and a
  `typed` chord is right about three; the fourth (`CSI 61;2u`) is unreachable by any chord and takes
  an alternate, because the terminal never said which character was produced.
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
- **A scroll scope's offset is a position and `Ctx::scrolled`'s is a translation** (runtime
  architecture 26, resolved 2026-08-30). The negation lives at one call site inside `scroll_scope`;
  `Ctx::scrolled` keeps the engine's rule, *scrolling down by `n` is `scrolled(0, -n)`*. Of the
  fields it moves, `view` and `origin` take `+dy` and `pointer` takes `-dy` — a disagreement that is
  the mechanism and not a bug: the content moves past a pointer that does not, and a press inside a
  scrolled scope lands on the row under it. **At offset 0 every sign agrees**, which is how this
  survived until three components tickets met it from three directions; register row 42 is the first
  scroll gate that draws.
- **Bars are reserved, never overlaid** (ADR 0029) — the parts of a reserved area tile the rectangle
  exactly, which an overlay bar cannot satisfy.

**No architecture question is open on any of the three maps** (2026-09-05). The runtime's closed with
36 on 2026-09-01; the engine's issues 20, 21 and 23 read as open for a while and were closed by
production tickets 06, 03 and 02 — do not re-file them; and the components map's last five all
resolved together. What they decided, because each is a rule a session can undo by accident:

- **19 — an edit that costs the volume is O6's other half, not a seventh obligation.** *The frame is
  flat in `n`* and *the edit is linear with a stated constant* are two **gates joined by an `iff`**:
  a covered row's ceiling has `per_input == 0.0` exactly when it is `Layer::L2`, read both ways, so a
  virtualised row cannot start folding unnoticed and a folding row cannot claim its fold is free.
  Both numbers are **step counts over the fold at a fixed `n`**, never a clock, so *fold less often*
  does not pass them; 16.7 ms is a denominator in `volume_numbers.rs` and a pass mark nowhere. The
  data-volume invariant is the **engine's** and holds there without exception — a component that must
  fold states §13's split instead, and the repository README and `CONTEXT.md` now carry both halves.
- **22 — `Esc` is the container's key until a collection has a selection to clear.** `owns_escape`:
  the component owns it exactly when `apply` would clear something, and declines it otherwise, so a
  `collection` in a modal no longer swallows the one key a dialog must answer. The two-stage
  dismissal falls out. `collection_shaped` stays crate-private — publishing the hook leaves the
  *default* wrong.
- **23 — `file_picker`'s popup has a keyboard.** The focus goes to the **list**; `Enter` answers the
  cursor's file and `Esc` cancels and answers nothing (a picker's `chosen` starts empty, so *answer
  what was there* would say nothing); **the preview pane has no keyboard and will not** — its
  document is replaced by the next arrow press. The two overlay owners share one `&[Bind]` and
  `PICKER_IS_MISSING` is 0.
- **24 — a table's band writes its own slack**, one run a row in `CollOpts::tail`'s Role, which is
  §2's *a component handed a rectangle writes all of it*. What ruled out *the caller owes it* is the
  failure mode: an untouched cell keeps the **previous column set's data**, where §2's six pinned
  panels are background. `COLUMN_RESIDUE` is 0 and `Slack::Unwritten` is the refusal it is measured
  against.
- **25 — the demand column answers *draws*, and `glyphs::DELEGATED` answers *what a caller must
  spell*.** `table`'s row is `Ellipsis` alone; its eleven box entries move, because §6 assigns a
  table's separators to the caller and that caller needs them from the theme. Every ownership gate
  and the collapse gate read the **union**, which keeps the pairwise ASCII figure at 42.
  `Distinction::Guide` is **struck** — its drawing is a tree's indent guide, which 20 established
  cannot exist — so `Distinction::ALL` is **nine** while `Glyph::ALL` stays at twenty.

## Traps this map has met more than once

The verification machinery is architecture, and most of the defects found here were in the
instrument rather than in the code. Each of these has bitten at least twice.

- **A scanner looking for a literal contains that literal.** Assemble needles from fragments, or
  scan a **bounded** region — the function's own body — which is the stronger form and the one this
  crate has twice (`collect`'s column solve, and its `tree`-is-a-collection scan since production
  08). That one had **two** satisfiers and neither was a call: the line holding the needle, and an
  `#[expect]` reason string naming the same function. A gate with two ways to read green on a file
  containing no call at all.
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
- **A gate over a state nothing has moved cannot see a defect that arrives on a change** (production
  04). Rows 15 and 16 were watched over a forward walk of a corpus and over the gestures — both
  *constructions* — while §11's two caret defects both arrive on an **edit**; row 18's gate reads
  *the width being drawn* and compared two indexes neither of which was drawn. Ask where the
  property's own defect arrives, and inspect **there**.
- **A check that re-runs the walk that produced the value cannot fail, however many observations it
  takes** (production 04, found by a review of its own first fix — the *recorder shares a coordinate
  system* trap one shape over). `Text::edit` places a caret by walking cluster steps from a row
  start, so *is this caret on a cluster boundary* is a membership test against that same walk, and
  starting the checker at byte 0 changes nothing — it passes through the row start and continues
  identically. 1 296 inspections of a byte-addressed caret after every edit report **zero**. When a
  property holds by construction, say so and count the producers apart: the observations live where a
  value enters from **outside** the construction — here `set_pos`, and `undo`, which restores a pair
  rather than recomputing one.
- **A `compile_fail` needs a twin naming the protected item by path.** The error code beside the fence
  is documentation: rustdoc on stable ignores it.
- **The recorder and the defect share a coordinate system**, so the gate cannot see it. Both recorders
  union in root coordinates for this reason.
- **Every gate plays the component at the full size of its context**, so two expressions that differ
  only when those two disagree are one expression. `crate::collect::header_row` drew from `x = 0`
  rather than from the rectangle it was given and *every gate passed because every one of them plays
  at `x == 0`*; production 07 met the same shape on the **width** — a tree's chevron is drawn from the
  row rectangle's indent and pressed from the component rectangle's, and `min(2d, w − 2)` is one
  number for two widths whenever the clamp binds on neither. Play the subject **inside** something
  smaller than the screen, at a size where one clamp binds and the other does not.
- **`Pen` records a clipped verb at the column it was *asked* for**, so a verb starting left of its
  clip lands in the recorder's arithmetic shifted by the discarded prefix (ADR 0022's
  clamp-and-discard; `Tally` carries the same caveat). Harmless while the subject writes inside its
  own rectangle — and *every* band refusal is a band writing outside one, so production 09's first
  draft reported **one cell in a gutter the screen does not have**. Write **one cell a verb** where a
  refusal can overrun: per cell a write lands whole or is discarded whole, which is
  `crate::runner::reference`'s arrangement.
- **A body cannot be a ruler on two axes at once.** An oracle built as *the band and the body agree
  about a coordinate*, off one frame, wants the body's top row to carry column marks and its left
  column to carry row marks — and those collide in the top-left cell, which a pinned column spans.
  Every escape is a special case; use the reference render every other scene here uses.
- **Counters on the wrong side of the question.** `changed > 0` is green on the exact set it exists to
  catch; every *output* counter is blind to work that produces no output (a fold at 476 ns a point
  drew the identical picture) — pair a work counter with a per-input ceiling.
  **A consumption counter is the same shape one axis over** (components architecture 22): O4's sweep
  asks *was the key consumed*, and a key consumed to do **nothing** reads exactly like a key consumed
  to do something — `pagination` declared `Escape`, is `Mode::Options` where `apply` ignores
  `Gesture::Nothing`, and the binding was reported answered for the whole life of the crate. Ask what
  the component did with it, not whether it took it.
- **Cumulative ledgers must be read as deltas**, and allocation windows counted **per frame** and
  warmed on the *shape* rather than on two identical frames.
- **An allocation window contains a thread nobody in this workspace started, and `--test-threads=1`
  does not remove it** (production 22). libtest **spawns a thread per test** on every target built
  here, then blocks on the channel it waits on — and that channel's first *blocking* receive
  allocates once per process: the receiving thread's context, the waker's mutex (a boxed pthread
  mutex on macOS, a futex that allocates nothing on Linux, so **three** here and **two** there) and
  the one growth of the waker list. If the parent is descheduled past the child's window opening, it
  lands in the **first** window the process opens — which is how a gate over an iterator with no path
  to an allocator reported two on one hosted run and zero on six. The probe excludes the process's
  first thread from every reading and excludes nothing else; a thread the *subject* started stays in,
  which is what `work_alloc.rs`'s last gate and the engine's handoff gate are about. *Nothing in this
  file starts a thread* is a statement about a file, never about a process.
- **A figure quoted from a spec is usually a prototype's screen.** Measure it; assert what reproduces
  as measured, print what does not beside it, and never bend the code to make an old sentence true.
- **`#[track_caller]` forwards into every `#[track_caller]` function it calls** and through nothing
  else — so a private body one frame down mints the caller's id, and two call sites of one component
  are two widgets. **And the converse is the half an application meets**: *one* call site drawing
  *two* widgets mints **one** id, so a loop over two panels hands the second panel the first one's
  focus, cursor and hover — and the screen looks perfect while the wrong panel answers the keyboard.
  `Ctx::with_key` is the fix and `commander` is where it was found; `spf` has the same shape with
  three panels. A gate never meets this, because a gate draws one subject.
- **`Ctx::next_key` answers only the focused id, and `Ctx::decline` ends the level's turn** — so a
  keyboard sink beside a *focused* `field` is deaf: a dialog's `Enter` and `Esc` are the field's to
  decline and arrive in `Driver::unhandled`, one frame later. It compiles, draws correctly and does
  nothing, which is why all three ports met it.
  **The second half of that trap was itself false and stood in four documents** (2026-09-08):
  *a container drawing a `collection` cannot read what it declined in the same frame, and the one
  in-frame route is `crate::collect::Refusal`, which is `pub(crate)`.* It can. `Ctx::scope`'s
  after-the-body moment is the route — a scope the focus drew inside becomes the routing target when
  its body closes and the decline that shut the queue is spent — and `vitui_components::input::form`
  has been built on it since it was written. What `Refusal` uniquely buys is **first** refusal: the
  keys a collection would otherwise *consume*, which is a `tree`'s fold keys and nothing a container
  can have. The scope reaches what it *declines*. Register row 236.
- **A group declares its axis and `crate::nav::step` reads it** (2026-09-08, register row 235). It
  read all four arrows as one axis until then — *a list's index grows downward, so `←` is `↑`* — and
  a **consumed** key cannot reach the container the list is drawn inside, so `commander` bound
  `Alt+←`/`Alt+→` and `spf` could not bind `hotkeys.toml`'s arrows at all. Both bind what their
  originals bind now. **The axis is a fact about the component and never an option on it**: nothing
  takes an `Axis` from an application, `collect::keyboard` takes it from the component, and there is
  no `CollOpts` field — a caller who could set the other one would be declaring something the
  component cannot honour. **Both values ship and the second was live and wrong**: `pagination` is a
  strip, and it answered `↑` for *the previous page* and declared it in a help bar for the whole life
  of the crate. `tree` is the only component that still reads `←`/`→`, as fold and unfold, and it
  declares them itself. `Home`/`End`/`PageUp`/`PageDown` are both axes'; the deaf `Ctrl` step does
  **not** turn, because `Ctrl+←`/`Ctrl+→` are word motion and `contract::ABSENT` reserves them — so a
  strip has the two ends and no deaf step, and answers 18 spellings where the store declares 22.
  `input::slider` still spells its own pairing (`↑` with `→`, a diagonal across both axes) and is why
  a list can decline an axis at all.
- **`owns_escape`'s defect was live one key over, and `collect::owns` is the rule said of the whole
  vocabulary** (2026-09-07). Components architecture 22 taught `Escape` to decline when `apply`
  would clear nothing — *the component owns it exactly when it would do something* — and left the
  other two keys `from_key` answers with no cursor move behind them: a bare `Space` was
  `Gesture::Toggle` in **every** `Mode` and `Ctrl+A` was `Gesture::All` in every one, where `apply`
  acts on the first in every mode but `Mode::Cursor` and on the second in `Mode::Multi` **alone**.
  So the key was consumed to do nothing, `out.changed` was set for a frame that changed nothing, and
  the container above never saw it. `owns` is now the one predicate and `owns_escape` is its first
  arm; **the three movers are owned unconditionally** and that is not an exception — by the time a
  `Plain` arrives the caller has already moved the cursor. **What it cost was the paperwork the old
  note called a map decision, and it was two numbers**: `Ctrl+A` came off `PAGER_BINDS` (22 → 21)
  and off `SELECT`, because a pager is `Mode::Options` and a popup's list is `Mode::Single` and
  neither answers `Gesture::All` — *select every row* was a help line no press could perform, which
  is exactly the sentence issue 22 wrote about `Escape`. `Space` did **not** move: a pager toggles
  with it. `commander` takes its spaces at the shell prompt now and `cluster` binds k9s's own
  `space` beside the `Ctrl+Space` it had to invent. `spf`'s `Shift+↓` is **not** this defect — the
  cursor moves, so the key did something, and routing `Extend` into its own marks is the
  application's.
- **A dialog that closes does not give the keyboard back, and the vanish rule decides where it
  goes.** The focused widget stopped drawing, so the focus moves to *the nearest surviving entry in
  the previous frame's ring order* — which for a screen with two panels is **the other panel**.
  `commander`'s `F5` on the left panel came back with the right one active and `cluster`'s table went
  deaf after an `Esc`; both look like the application forgot something and neither is visible in a
  gate, because a gate draws one subject and closes no dialogs. The shape of the answer is a standing
  one-`bool` request that outranks *read the focus and believe it*, and it has a second half: while
  that request stands, a value **derived** from the focus must not be adopted, or the application
  reads its own pending move back as the user's.
- **A match on `k.code` is a keyboard that works on a legacy terminal and is half dead on a modern
  one.** ADR 0053's *one keystroke has four wire spellings* is not a curiosity about `+`: a terminal
  speaking the enhanced keyboard protocol sends the **base key and the shift bit**, so `?` arrives
  as `CSI 47;2;63u` (code `/`, SHIFT, text `?`) or `CSI 47;2u` (code `/`, SHIFT, and nothing about
  `?` at all), and `Shift+N` arrives as `CSI 110;2;78u` or `CSI 110;2u`. `cluster` matched
  `Code::Char('?')` and `Code::Char('N')` and therefore opened the *filter* on `?` and sorted
  nothing on `Shift+N` — on every terminal in this repository's own conform suite, while reading
  perfectly on a legacy one. `spf` had the same hole under every capital `hotkeys.toml` binds.
  **`Chord::typed(c)` covers three of the four spellings and the fourth needs an alternate**
  (`Chord::key('/').shift()`), which is a second binding rather than a repair; `KeyMap::match_first`
  takes the first match, so the shifted binding must be bound **before** the unshifted one it shares
  a base key with. `KeyMap` works outside the draw — `match_first` takes a `&Key` — which is what
  makes it usable from the unhandled window. A headless gate cannot see any of this, because a
  headless gate posts the spelling the test author typed.
- **An overlay that appears IS on the screen on its own frame, and the sentence that said otherwise
  named the wrong mechanism** (2026-09-07). What stood here for four documents was *the body draws
  into its granted rectangle and the cells do not reach the terminal; the next real input event of
  any kind brings them*. The pass runs inside the `frame` call, before `end` and before `present`,
  and `present` composites its layer — the frame that adds an overlay writes bytes where an
  identical repaint writes **0**, which is
  `ctx::overlay_tests::an_overlay_that_appears_is_on_the_screen_on_its_own_frame` and is the byte
  count rather than `Presented::submitted`, because damage is marked by the verbs and a boolean
  there cannot fail. **The mechanism was the award**, below. `commander`'s pull-down opened from
  `Response::clicked` looked like a click on nothing because the *click* was a frame late, and
  `press_began` appeared to work because the release was then the event that brought the next frame.
- **Everything `end` decides is drawn on the next frame, and until 2026-09-07 nothing asked for
  it** — `Frame::resolve_award`, register entry 50. The award is resolved *from the index that has
  just drawn* and rotated into `delivered` by the next `begin`; the ring resolves its walk in the
  same place. Every application here parks in `Driver::wait`, so **the press drew nothing, the
  release drew the press, and the click waited for whatever the user did next**, and a `Tab`
  appeared not to switch panels until the next key arrived. Reported from outside as *the release
  fires and not the press*, which is what it looks like from a trackpad. The ask is now beside
  `resolve_into_view`'s (runtime architecture 33, the same shape) and has two producers: the pointer
  award — a press, a release, a click or a cancelled drag — and **the focus ending the frame
  somewhere other than it started**, which covers the ring, a trap's pull, the vanish rule and a
  press that landed on nothing interested. A long press is excluded and says so: it already asks,
  and `a.long_pressed` stays `Some` for every frame a grab stands. **The application half of the
  staleness stands**: a value derived from the focus must be read at the *top* of the draw, because
  everything below it is what the reader sees.
- **A key release is not routed, and it was a keystroke counted twice** (2026-09-07, register entry
  49). `crate::actuate` pushes kitty flag 31 and bit 2 of that is *report event types*, so on
  Ghostty, kitty, WezTerm and iTerm2 every keystroke arrives as two events. Components guard it in
  eight drain loops; **nothing above them did**, so one `j` scrolled two lines in the fifteen of
  twenty-one applications that match on `KeyCode` rather than through a `KeyMap` — and read
  perfectly on a legacy terminal. It is dropped in `Frame::begin`, the one place a key enters a
  frame from the tty and from `Driver::post_key` alike; `Driver::report_key_releases` keeps the wire
  reachable for an application that wants a key-up. **No gate in this workspace could see it**, and
  that is the instrument rather than the coverage: a gate posts the spelling its author typed, and
  `crate::keys::press` builds a press.

## Workspace

```
crates/vitui-engine       cells, surfaces, layers, compositing, damage, serializer, frame writer
                          └ crossterm behind a seam: raw mode, input, capability detection
crates/vitui-runtime      layout, identity, focus, hit-testing, routing, key maps, theming,
                          overlays, the data contract — no scene tree, no reactivity
crates/vitui-components   windows, panels, charts, lists, trees, forms, pickers — 29 of 29 built
                          └ `gallery`: 29 panels as a value, where §21's rows 7 and 8 are measured
                          └ `dropped`: the two overlay owners' four axes, where a popup's interior
                            is drawn in the base pass because a layer body takes no ink
                          └ `forest`: `tree`'s three scenes — the flatten index at depth 59 999, a
                            fold, and §7's partition at 300, 40, 22 and 21 columns, where the clamp
                            binding and not the label truncating is what separates the two indents,
                            and where the chevron's *pressed* column is the rectangle's. Its first
                            two scenes are built by `index_for`, which puts every row at one depth
                            — so no row has a child and the chevron they draw is a **space** on all
                            eighty rows; `nested` is what the third one takes
                          └ `surround`: what an area writes and its body does not — the tail past
                            `[extent, offset + viewport)` and the five bands beside it. The bands
                            are compared at two non-zero offsets with `(0, 0)` as the control; the
                            tail is played *from* `(40, 100)` and read after the shrink has clamped
                            the window to the origin, which is the axis's own arithmetic
                          └ per component: a doc page with a compiled example (O1), a golden screen
                            per construction under `tests/golden/` (O3, 36 of them, `VITUI_BLESS=1`),
                            a declared keyboard contract for the thirteen that read a key (O4), and
                            an application that imports it (O7)
                          └ `media` is **no row of the freeze at all** — §14's *no v1 component*, so
                            the family ships and `MEMBERS` is empty
crates/vitui              facade re-export — engine, runtime, components
crates/vitui-apps         21 applications, one file each in `examples/`: counter, triage, latency,
                          ledger, explorer, reader, settings, compose, console, theatre, browse,
                          mixer, vitals, roster, gallery, sheet, pipeline, caps, commander, cluster,
                          spf.
                          The surface's only consumer, and repeatedly the thing that found the defect
                          the gates could not — a gate exercises a component where its author put it
                          and an application puts it somewhere else
                          └ the last three are **ports of programs people use**: `commander` is GNU
                            Midnight Commander, `cluster` is k9s, `spf` is superfile. A port's shape
                            is not ours to argue with, so what it cannot express is a fact about the
                            surface rather than a taste — and three of them wanting the same missing
                            field is what turns a taste into a gap. All three run on invented data:
                            a real `readdir` or a real `kubectl` puts the interesting failures in
                            the transport instead of in the library under test
                          └ `commander` found the identity trap from the application side: two
                            panels are two calls from **one** source line, `Ctx::id` is
                            `Location::caller()`, and without `Ctx::with_key` the second panel takes
                            the first one's focus, cursor and hover. `spf` has the same shape with
                            three panels
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
conform/                  the only instrument that asks a real terminal: SCENES.md normative, eight
                          arms across seven examples — Ghostty, Ghostty-via-tmux (the same binary
                          behind `--through-tmux`), tmux, kitty, Terminal.app, WezTerm, Alacritty,
                          iTerm2 —
                          one committed REPORT-<arm>.md each, FINDINGS.md by hand
                          └ Terminal.app is the fourth VT lineage and **the arm that disagrees**:
                            four of scene 05's twelve surveyed rows, all four by summing a cluster's
                            code points. Its capture surface carries no style at all, so scene 01 is
                            eleven `cannot ask` rows and scenes 05 and 06 — asked in band — are
                            answered in full
                          └ WezTerm is the fifth family and **the arm that answers wrongly**: mode
                            2026 reported reset while set, which is a real misbehaviour and earns no
                            quirk row, because that table is routes and `Detected::mode` reads `2`
                            as available. Its capture is a classic SGR re-speller — `4:1` back as
                            `4`, `4:2` as `21`, and nothing at all for `4:3`, `4:4`, `4:5`, 53 or 58
                            — so two of scene 01's rows are `cannot ask` that **cannot** become a
                            quirk: no far side, no second source. `wezterm cli` will start a
                            mux-server daemon and photograph its shell, exiting 0
                          └ Alacritty is the sixth family and **the arm whose capture is not an
                            escape stream**: `--ref-test` writes the `Term`'s own grid as JSON, one
                            object per cell, so no serialiser of the emulator's is in the path. It
                            is read by `src/grid.rs` behind `Dialect::AlacrittyGrid`, the fixtures
                            are `.json`, and it is the first arm that could be *asked* about a
                            dotted underline. Two bare rows — blink and overline — that **are** a
                            quirk, because the grid is the evidence and `Flags` and `-vvv` are two
                            more. The capture is the window closing: there is no socket to ask
                            during a run
                          └ iTerm2 is the seventh family and **the arm whose capture surface was
                            chosen rather than found**: AppleScript is Terminal.app's `type="text"`
                            and would have given eleven `cannot ask`, so the arm speaks the Python
                            API's `GetBufferRequest` with `include_styles` — protobuf behind an RFC
                            6455 handshake on a unix socket, hand-rolled because this directory has
                            no dependencies, read by `src/buffer.rs` behind `Dialect::Iterm2Buffer`
                            with `.pb` fixtures. A `CellStyle` is a **projection** of iTerm2's
                            `screen_char_t`, not the struct, so its three unanswered rows split one
                            and two across the two kinds: overline is a quirk entry (the cell has no
                            bit and the binary has no such string), and the double and dotted
                            underlines are `cannot ask` (three bits projected to a `bool`). **The
                            only surface here that tells a cell the terminal emptied from a space
                            somebody wrote** — `U+0000` against `U+0020` — and the reader projects
                            that away on purpose. Needs the API switched on and a grant
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
docs/img/                 the repository's images, and the site stages the tree as it is
                          └ `components/` 29 and `apps/` 20 SVG pictures, each one composited frame
                            written as text rather than as terminal bytes. Regenerated by
                            `scripts/shots.sh`, compared on every pipeline, and reviewed as a diff —
                            which is the whole reason the format is SVG and not PNG
scripts/                  the six gates and reports that cannot be a `cargo test`
site/                     the documentation site: Astro Starlight over this repository's own
                          markdown, served at <https://vitui-rs.github.io> (an organisation page
                          repository, so no base path). Node, not cargo
                          └ **nothing under `docs/` is copied by hand.** `scripts/stage.mjs` reads
                            `docs/guide`, `docs/adr`, `docs/spec` and `CONTEXT.md` on every build and
                            writes them into the content collection with three transformations —
                            the `#` heading becomes the title, a `.md` link becomes a route, a bare
                            `<Left>` outside code is escaped. The staged trees and `public/img` are
                            **git-ignored build output**; edit the home, never the copy
                          └ four data files under `src/generated/`. `inventory.json` and `apps.json`
                            come from `INVENTORY` and `APPS` through
                            `cargo run -p vitui-components --example inventory_json` and
                            `cargo run -p vitui-apps --bin apps_json`, and are **committed** because
                            a Pages runner has node and no cargo — `npm run check` is the ratchet
                            that fails when either stops matching the crate. `benchmarks.json` and
                            `terminals.json` are parsed out of `compare/REPORT.md` and
                            `conform/REPORT-*.md` by node on every build, so they are not committed
                          └ the component page's population **is** the freeze, and a component whose
                            home family has no section on the page is a failed build rather than a
                            missing row. `starlight-links-validator` fails the build on a broken
                            internal link
                          └ the deploy is `.github/workflows/site.yml` and it needs one thing a
                            person does by hand: a deploy key on `vitui-rs/vitui-rs.github.io` and
                            its private half as `PAGES_DEPLOY_KEY` here. Until that secret exists the
                            site builds and uploads an artefact and the deploy step is skipped
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
cargo rustdoc -p vitui-apps --example NAME  # the same lint over an example, which the gate misses
cargo deny check                            # needs `cargo install cargo-deny`
(cd fuzz && cargo deny check)               # detached workspace: its own graph, its own gate
(cd conform && cargo test)                  # the conformance gate, over committed captures
(cd conform && cargo run --example tmux)    # the one live arm that is headless
(cd conform && cargo run --example terminal) # needs an AppleScript grant for Terminal.app
(cd conform && cargo run --example wezterm)  # a window and a control socket; exits 1 on scene 06
(cd conform && cargo run --example alacritty) # a window per scene, closed to take the capture; exits 1 on scene 06
(cd conform && cargo run --example iterm2)   # needs `EnableAPIServer` and a grant; 22/22
(cd site && npm install && npm run dev)     # the documentation site, on localhost:4321
(cd site && npm run build)                  # ./dist; a broken internal link fails it
(cd site && npm run generate)               # re-derive inventory.json and apps.json from the crates
(cd site && npm run check)                  # the ratchet: fails if either has gone stale
```

Applications (`cargo run -p vitui-apps --example NAME`), each with the key worth pressing; those
marked `--probe` print one headless frame and what it cost:

| app | what it shows |
|---|---|
| `counter` | the first application; `q` to quit |
| `console` | the overlay family; `Ctrl+P` palette, `Ctrl+Z` to the shell (`fg` to return), `Ctrl+Q` quit |
| `theatre` | the media family; `4` is the floor of the colour axis (`--probe`) |
| `browse` | the preview pane; `k` then `s` is the memo-key rule (`--probe`) |
| `mixer` | the slider; `x` fifty steps, `f` swaps the arithmetic (`--probe`) |
| `vitals` | the six Tier 2 rows; `g` steps the glyph rung (`--probe`) |
| `roster` | the Tier 2 composites; `Ctrl+G` takes the arrows away (`--probe`) |
| `sheet` | the caller owns the offset; `t` is the tail, on `tiny.csv` (`--probe`) |
| `pipeline` | the anchor; `c` swaps the cadence, 60 wakes against 431 991 (`--probe`) |
| `gallery` | every built component on one screen; `t` is the key (`--probe`, `--matrix`) |
| `caps` | what THIS terminal answered; draws no frame |
| `commander` | Midnight Commander; `Tab` swaps panels, `F5` copies, letters go to the shell prompt |
| `cluster` | k9s; `:deploy` switches resource, `/` filters, `d`/`y`/`l` push a level, `Esc` pops |
| `spf` | superfile; `v` select mode, `Ctrl+C`/`Ctrl+V` starts a process, `Ctrl+R` renames in place |

Warnings are denied workspace-wide (`[workspace.lints.rust] warnings = "deny"`), so an enum variant
nothing constructs is a build failure rather than a spare part.

**`cargo doc` does not document an example**, so the intra-doc lint above is a gate over the four
libraries and over nothing in `examples/`. Four of the twenty-one applications had accumulated a
broken or private-item link that no job in this repository asked about — the same shape as the
`cargo test` does not run an example trap one lint over, and the sweep is the `cargo rustdoc` line
above run per example. All twenty-one are clean as of 2026-09-07; nothing keeps them that way.

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
scripts/suspend-reader-gate.sh # a suspension does not vacate stdin, on a pty — register #31
n=1 cargo test -p vitui-engine golden                 # regenerate; review the git diff
VITUI_BLESS=1 cargo test -p vitui-components golden   # the components' screens; refused in CI
scripts/shots.sh              # the forty-nine pictures: 29 components, 20 applications
scripts/shots.sh --check      # and whether each is still the screen it is a picture of
cargo run -p vitui-apps --example NAME -- --shot   # one application's picture
```

The fuzz targets are a **soak, never a gate** — the committed corpus replayed by `cargo test` is the
gate; `fuzz/README.md` is the procedure. The nightly toolchain must be the one that actually runs —
see `CLAUDE.local.md` if `RUSTUP_TOOLCHAIN` appears to be ignored:

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
`msrv`, `deny`, `budget`, `idle`) on a self-hosted GitLab that is not publicly reachable.
**`.github/workflows/` holds what that runner cannot do**: `ci.yml` for the macOS/Linux/Windows
matrix, `soak.yml` for the weekly fuzz soak, `compare.yml` for the monthly comparative suite, and
`site.yml` for the documentation site. The two scheduled workflows *upload* their report and never
push one.

Every job declares its own image, because the runner has no default one. How to start it, and where
its token is, are in `CLAUDE.local.md`.

**A commit is not the end of a ticket.** Push to the runner and watch every job go green before
reporting the ticket done.

## Working a backlog

**The backlogs are not in this repository as a reader sees it.** `.scratch/` — eight maps, their
tickets, their research and the agent conventions — is ignored here and lives in a private archive
and in the machine's own working copy, both named in `CLAUDE.local.md`. A session on that machine has
all of it; a reader of the public repository has the three specs in `docs/spec/`, the ADRs, and what the
crates say. Cite the fact, never the path.

`.scratch/agents/issue-tracker.md` is the full convention. In short: one ticket per session; the
**frontier** is the lowest-numbered file that is unblocked and unclaimed, and the `Blocked by:` line
is the authority — the number only breaks ties. Claim by setting `Status: claimed` before any work;
resolve by appending an `## Answer` section, setting `Status: resolved`, and adding a one-line
pointer to the map's Decisions-so-far. Research findings go in `research/` beside the issues.
**`issues/` holds only what is open**: a ticket reaching a terminal state moves to the map's
`archive/` keeping its number, so a map with nothing open has no `issues/` directory at all. Nothing
is deleted — the `## Answer` sections are where those findings live, and every document here cites a
ticket by number rather than by path.

Build order across the repo is **engine → runtime → components**, and it was never a queue: several
runtime tickets name single engine tickets and ran beside them. The impl backlogs
(`.scratch/vitui-{engine,runtime,components}-impl/`) are **closed**, and so is
`.scratch/vitui-engine-production/`. **The one active backlog is `.scratch/vitui-production/`** — its
README is the queue and the blocking edges, and a ticket there may name an edge into another backlog
(`components architecture 20`), which is the authority the same way a local number is.

**A `Status:` line carries two vocabularies and they are not in conflict.** A freshly written ticket
says `ready-for-agent` or `ready-for-human`, which is triage. A session working one overwrites it
with `claimed` and then `resolved`, which is the frontier protocol above. `ready-for-human` means
*do not claim this without the thing it needs* — a Windows machine for 16, which is the last one
left carrying it.

`tickets/` at the repo root was a **separate** surface — the hand-written backlog the `dispatch`
skill consumes. Both of its items are spent (the component model became the two architecture specs,
the gallery demo became `gallery`) and the directory is gone; a new hand-written item re-creates it
rather than joining a map's backlog.
