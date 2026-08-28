# Design the component model, and re-examine the crate split before anything is implemented

**Status: resolved — 2026-08-28.** Both halves are answered; this file is a record, not open work.
The ticket was map-shaped, as it predicted, and the two maps it opened have both been reached:
`.scratch/vitui-runtime-architecture/` (2026-08-19, R16 → `spec.md`, ADRs 0012–0021) and
`.scratch/vitui-components-architecture/` (C12 → `spec.md`, ADRs 0026–0033 and 0035–0045). **The
specs are the authority; where this ticket's proposed answers disagree with them, they are wrong** —
one of them is (the data contract, see A below). Nothing here is a work item, and nothing here is
for a dispatched agent to re-derive: the component library is 28 of 29 built with 14 applications
standing on it.

## Goal

Decide **what a component is** in `vitui`, and **which crate each piece of the answer lives in**,
before a single line of `vitui-runtime` or `vitui-components` is written.

Two halves, and they are one ticket because each answer moves the other. If a component is a plain
function, the crate split can be thin; if it must be a trait with a measure pass, the runtime grows a
node type and the split changes with it.

The engine is not in question. Its abstractions are the lowest in the system, eighteen tickets of a
wayfinder map settled them, and nothing here should reopen one. What *is* in question is everything
above the seam, which was deliberately left as fog while the engine was being decided.

The working analysis this ticket started from was a note, `COMPONENT-HIERARCHY.md` (2026-08-17),
written to find out where to dig before there was a spec to dig against. **It has been deleted, and
what replaced it is authoritative**: the runtime's architecture is
[`.scratch/vitui-runtime-architecture/spec.md`](../.scratch/vitui-runtime-architecture/spec.md) and
the components' is [`.scratch/vitui-components-architecture/spec.md`](../.scratch/vitui-components-architecture/spec.md),
with the map beside each. Nothing in the note survived that those two do not say better; where it was
wrong, they say so and say why.

This ticket is likely to produce a **wayfinder map** rather than a single answer, the way the engine
effort did. The map's own Notes already say `vitui-runtime` and `vitui-components` are separate
efforts that get their own maps.

## Acceptance criteria

**All of A and B are answered by the two architecture specs this ticket produced.** Each box cites
where, so a reader arriving cold reads the answer rather than re-deriving it. `RS §n` is
`.scratch/vitui-runtime-architecture/spec.md`, `CS §n` is
`.scratch/vitui-components-architecture/spec.md`.

**A — the component model**

- [x] A one-sentence definition of a component, and a table running every candidate through it:
      `text`, `fill`, `border`, `bar`, `plot`, `input`, `toggle`, `scrollbar`, `list`, `table`,
      `tree`, `panel`, `split`, `tabs`, `window`, `popup`, `modal`, `form`, `chart`, `picker`,
      `layout`, `grid`, `row`, `cell`, `gradient`, `scroll`.
      **CS §1** is the shape, **CS §17** the twenty-nine-row freeze, **CS §18** the coverage
      argument — and §18 is stronger than this box asked for: the candidate list above is 26 names,
      the survey's union catalogue is ~430 entries across 15 families, and §18 accounts for the
      residue by naming a mechanism for every entry rather than by calling it small. `INVENTORY`
      is that freeze as a value a test iterates (components 35).
- [x] **Is a component a function or a trait?** Decided, with the consequence stated. This is the
      fork: a measure pass forces a trait with two methods and reintroduces a walked structure, which
      is exactly what ticket 05's *the clip stack is the call stack* deleted.
      **A function** — **RS §2**. The trait form was *built* rather than argued away, and the
      record matters: same rectangles at **1.03x**, 0 allocations against 2 once it takes the
      caller's buffers. It died of drift and of the dry run, not of cost (RS §12).
- [x] **Intrinsic sizing: yes or no.** If no, state what layout can and cannot express. If yes, state
      what it costs in the component signature.
      **No** — **RS §12**, exit (a). The mechanism is a *sizing function* beside the component,
      taking the same `&data` plus the extent it is about to be given. It takes no draw context, so
      it cannot draw, cannot claim an identity and cannot route, which is the whole of what makes it
      a function rather than a method. Sizing a dialog to its contents is 569 ns.
- [x] **Identity** — where a name that survives between frames comes from. Six mechanisms need one and
      nothing decides it: focus routing, hit-testing, interest declaration, memo invalidation, overlay
      ownership, scroll association. Immediate mode has no mount.
      **Call site plus key, via `#[track_caller]`** — **RS §5**, **CS §4**, ADR 0027. Two
      consequences this ticket could not have predicted and every later one meets: a toggle that
      swaps *which function* draws a widget swaps the widget, and a component taking its id inside a
      private body one frame down gets the *body's* line, because the attribute propagates only
      through functions that carry it.
- [x] **The data contract** (`Rows` or whatever it is called) — the shape a collection component
      demands of application data. It would be the runtime's first trait; the engine has none.
      **There is no trait** — **RS §14**, **ADR 0019**. This overturns the proposal logged below on
      2026-08-17: `Rows` does not exist. `Revision`, `Versioned<T>` and `Memo<T>` ship as plain
      types. The case that was supposed to justify the trait is the one that removed it — `len()`
      becomes a second source of truth the moment a view is filtered, and what makes a revision bump
      unforgettable is `Drop` on a guard rather than a promise to report a number.
- [x] **The two-phase overlay protocol**, concretely: how an overlay request is expressed during the
      draw and satisfied after it, and how it is linked to its owner across passes.
      **RS §10**, **CS §12**, ADR 0017 (its arena half partially superseded by ADR 0034). The
      request is made during the draw and satisfied after it; the two `Ctx` lifetimes are what carry
      the link, and `'f` is the mechanism rather than an annotation — deleting it compiles and
      deletes the protocol, which is why `overlay::OWED_SENTENCE` is checked as a scan of the
      runtime's own file.
- [x] The component library's own vocabulary, added to `CONTEXT.md`. `cell`, `layer`, `surface`,
      `view`, `run`, `frame` and `damage` are the engine's and cannot be reused — a table needs other
      words for its units.
      Done, and `CONTEXT.md` now carries sections of its own for drawing, the loop, layout and
      sizing, scrolling, overlays, data, threads, the terminal and verification.

**B — the crate split**

- [x] Each crate's responsibility restated, or the split changed, with the reason written down.
      **RS §4.** Five modules were proposed as crates during the map: **four refused, one accepted —
      and the accepted one has since been deleted**, so the shipped answer is that none of the five
      is a crate. `crate::line::MODULES` is that table as a value.
- [x] **Does reactivity belong in `vitui-runtime`?** The map settled that reactivity must be
      replaceable without touching the engine, and ticket 12 proved it by building a TEA runtime and a
      signals runtime on one unmodified seam. If a *specific* reactivity ships inside `vitui-runtime`,
      "replaceable" means "fork the crate", which is not what was decided.
      **No** — **RS §18**, ADR 0020. Tested by building the thing and then deleting it:
      `vitui-signals` shipped 2026-08-24 and was removed 2026-08-25 (components architecture issue
      24) on **re-measured** numbers — three drivers of one screen within 0.21% of each other, 0 of
      24 000 cells differing, the signal layer's whole per-frame work 3.60 ns. The memo it wrapped
      (`vitui_runtime::data::Memo`) stays. Reopen issue 24 rather than restoring it.
- [x] **Does layout want to be its own crate?** It is a pure function library over integer
      rectangles with no dependency on anything else in the workspace, and it is independently useful.
      Argue it either way, but argue it.
      **Refused, and the premise was wrong** — **RS §4**: layout needs `vitui_engine::Rect` and ADR
      0005's exports, so it is not dependency-free, and `vitui-layout` is three crates doing two
      crates' work. It stays a module (`layout`, `layout::text`), RS §11.
- [x] **Can `vitui-components` stay reactivity-agnostic?** It can if a component is a function over
      `(&mut Ctx, Rect, &Data, &State)`. It cannot if it binds to a runtime trait — which is question
      A's fork arriving in the crate graph.
      **Yes**, and it is checked rather than claimed: **CS §0's C6** and **CS §19** — this crate
      names `vitui-runtime` and nothing else, enforced by `deny.toml` and gated from a crate that
      cannot name the engine (`crates/vitui-components/tests/crate_line.rs`).
- [x] The **dependency policy** restated per crate, and `deny.toml` updated to match. The current
      policy says `vitui-runtime: no dependencies` and `vitui-components: case by case`; if the split
      changes, both sentences need rewriting, and "case by case" needs a precedent rather than a
      shrug.
      Both sentences were rewritten. `deny.toml` carries the per-crate policy in its own header and
      enforces the engine ban through `[bans] deny` with `wrappers`, and *that mechanism bans a
      crate's presence in the graph* — the finding that forced `vitui-signals` to be detached before
      it could be deleted. `[bans] allow` was refused with a number: 38 crates.
- [x] Confirmation that nothing in the answer moves the **engine's** boundary. If something does,
      that is a finding and it goes back to the engine map rather than being absorbed quietly.
      Confirmed, and the procedure was exercised twice. The engine's own five reopened questions are
      `.scratch/vitui-engine-architecture/issues/19`-`23`, filed rather than absorbed; and the one
      collision this split produced — `vitui_engine::Rect` unnameable across the crate line, which
      made CS §3's *returns the rectangle it did not write* unwritable as Rust — went back as
      **runtime architecture issue 22** and was answered with a rule in both directions, not a list.

**C — the loose end this uncovered**

- [x] `CONTEXT.md` lists **"the scene tree"** among `vitui-runtime`'s responsibilities. That term
      predates ticket 05's finding that the clip stack *is* the call stack. Either the runtime really
      does retain a tree — in which case say what it holds — or the term is a retained-mode leftover
      and `CONTEXT.md` is wrong. It cannot stay ambiguous into the runtime's design.
      **Closed by runtime ticket R14**: it is a leftover, the runtime retains no tree of any kind,
      and the replacement is *frame state* — five flat structures rebuilt from the draw. It did stay
      ambiguous through the runtime's whole design, which is the part worth remembering: nine tickets
      drew through a term each of them had already made false.

## What is already decided and must not be re-derived

Pointers, so the next session reads rather than re-measures. All from
`.scratch/vitui-engine-architecture/`.

- **The engine does not lay out; callers bring rectangles** — ADR 0002.
- **The engine is everything that touches the terminal; the runtime is everything above, including
  the API components see** — ticket 12. A component never holds an engine type.
- **A component never needs ownership or `&mut` of application data** — precedence rule 2. One table,
  shown at once as a list and as a bar chart, with neither owning it.
- **A column split is inexpressible in safe Rust while two views are live** — ticket 12, two E0499s.
  This is why `Screen::split_h`/`split_v` do not exist, and it constrains any container design.
- **A component cannot open its own layer mid-draw** — ticket 14 R2, E0499 against the seam. Overlay
  requests are collected during the draw and satisfied after it.
- **Virtualisation needs O(1) indexed access** — ticket 14 R4: an O(k) row lookup in a 1M-node tree
  costs 9.76 ms a frame, 656× the O(1) form. Expansion must splice, not rebuild.
- **A derived aggregate must be memoised** — ticket 14 R3: a maximum over 1M rows costs 130.23 µs a
  frame against 0.4 ns memoised, more than the whole damage-tracked budget. The invalidation key must
  be something an application cannot silently forget to bump.
- **`visible_rows()` is the virtualisation primitive** — ticket 05: flat at 54 µs across 1k, 100k and
  1M rows; a free per-cell discard is not enough, at 3.6 ns each 1M rows burn 3.68 ms of pure
  rejection.
- **A gradient is a `fill` with a varying style, not a blend mode** — ticket 06. The compositor never
  sees one.
- **The draw context carries** a formatting scratch, the capability tier, the frame's sampled clock, a
  caret sink and the content-to-screen transform — tickets 14 R1 and 10.
- **The runtime samples the clock once per frame; no component assumes an interval** — ticket 14 R6:
  120 Hz configured achieved 99.7 fps, so a fixed `dt` is 17% slow over three seconds.
- **Focus is routing, not enablement; hit-testing, interest and focus are three mechanisms** —
  ticket 10.
- **A border glyph set chosen from `Capabilities::glyphs` is a legitimate choice at the source** —
  ticket 11. The engine never substitutes a cluster behind a component's back.
- **`Mix` clears the extended-style bit, so a shadow crossing a hyperlink deletes it** — ticket 16.
- **Deadlines should be attributable to the component that asked** — ticket 14 R5.

## Non-goals

- Reopening any engine decision. Eighteen tickets, eleven ADRs; if one is wrong, that is a finding
  and it goes back to the engine map with its own ticket.
- Writing the component library. This ticket decides the model; the library is a separate effort.
- Choosing a layout algorithm. Whether layout is `taffy` or a hand-written integer constraint solver
  is a real question and it is **not** this one — this ticket only decides whether layout is a
  component, and which crate it lives in.

## Blocked by

Nothing hard. `.scratch/vitui-engine-architecture/issues/15-write-the-architecture-spec.md` is the
engine map's last open ticket and its destination; the engine's public surface is already frozen by
ticket 12, so this ticket can be worked before, after or alongside it. Doing 15 first means the seam
this design sits on is written down rather than remembered.

## Progress

- 2026-08-17 — opened. Working analysis in `COMPONENT-HIERARCHY.md`, with a component-library graph
  rendered beside it. **Both were deleted on 2026-08-20**, superseded by the two architecture specs;
  the entries below are the record of what they produced, not a pointer to something still there.
  The `component-families` and `runtime-stack` diagrams named further down are a different pair and
  are still here.
- 2026-08-17 — research and two proposals written, ahead of claiming the ticket. The ticket is
  confirmed to be map-shaped: it produced two maps rather than one answer.
  - `.scratch/vitui-components-architecture/research/01-component-library-survey.md` — fourteen
    inventories fetched live, ten families, ~250 components, the measured core (28 components
    present in 9+ inventories), what a terminal deletes, and **three findings for the engine
    map**: bitmap protocols (not blocking — half-block is the answer), OSC 52 clipboard, bell.
  - `.scratch/vitui-components-architecture/requirements.md` — `C1…C20` with a check each.
  - `.scratch/vitui-components-architecture/architecture.md` and
    `.scratch/vitui-runtime-architecture/architecture.md` — proposals, every decision marked
    `[D]`, every inherited fact marked `[F]`.
  - `.scratch/vitui-runtime-architecture/map.md` (R01–R16) and
    `.scratch/vitui-components-architecture/map.md` (C01–C12).
  - Diagrams: `docs/img/component-families.{dot,png}`, `docs/img/runtime-stack.{dot,png}`.
  - Proposed answers to this ticket's acceptance criteria: a component is a **function**, no
    intrinsic sizing; identity is **call site + key** via `#[track_caller]`; the data contract is
    **one trait** (`Rows`) plus closures plus a `Versioned` edit guard; overlays are **requested
    during the draw, satisfied after it, answered next frame**; reactivity is **not** in
    `vitui-runtime`; layout stays a **module** of it for 0.x; `CONTEXT.md`'s "scene tree" is
    **wrong** and a replacement wording is drafted.
- 2026-08-17 (later) — **the catalogue was thin and has been rebuilt.** The first pass reasoned
  from library *widget inventories*, which structurally hide anything expressed as a container
  attribute in retained or CSS frameworks. Five families were missing: scrolling/overflow,
  expand/collapse, media, files and preview, system monitoring. Second sweep added GTK 4,
  Flutter, PrimeNG and yazi as sources plus the *application* vocabulary (btop, k9s, lazygit,
  ranger, mc, zellij). Catalogue is now **15 families, ~430 entries**.
  Consequences that changed a design conclusion, not just a list:
  - `scroll_area` is a **component** (L3), correcting the note's "scroll is not a component". It is
    *not* `list`: a scroll area costs its content, a virtualised list costs its visible window. New
    requirement C21 keeps them apart.
  - Expand/collapse is **one state machine** behind thirteen catalogue entries, so it ships as
    one helper (`collapsible`) rather than thirteen implementations.
  - The graphics-passthrough finding is **upgraded**: it blocks two families at full fidelity,
    with a price (~7 MB/s for full-screen video drawn as cells) and three honest options for
    the engine map. A **background image**, a QR code, a waveform and all video-player chrome
    are buildable today with no engine change.
  - New tickets: runtime R17 (scrolling), R18 (async work); components C13–C16 (scroll area,
    collapsible, media boundary, file preview exemplar). v1 inventory is now 33 components.

- 2026-08-19 — **both maps reached.** The runtime's by R16, the components' by C12. Every box in A
  and B is answered by one of the two specs; the boxes above carry the section that answers each.
- 2026-08-28 — **closed as a record.** The unticked boxes were the live hazard rather than the
  prose: `tickets/` is the surface `dispatch` consumes, the frontier is the lowest-numbered
  unblocked unclaimed file, and this was it — thirteen open boxes inviting an agent to re-derive a
  data contract that ADR 0019 killed on measurements. Deleting the file was considered and
  refused: both maps cite it by path on their line 4 as where they were opened, and a citation to
  a file that does not exist is a defect this repo has already paid for once.
  **One proposal from 2026-08-17 was overturned** and it is the one worth carrying forward: the
  data contract is **not** a trait. `Rows` was proposed here, built as a trait on the runtime map,
  and removed — the two methods that justified it both fail when run (ADR 0019). The other five
  proposals held: a component is a function, no intrinsic sizing, identity is call site plus key,
  overlays are requested during the draw and satisfied after it, reactivity is above the runtime,
  and layout stays a module.
  **What this ticket did not settle, and did not claim to:** the layout algorithm (its own
  non-goal), and the v1 inventory it last logged as 33 components, which the freeze settled at 29.
