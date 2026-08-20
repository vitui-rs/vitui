# Design the component model, and re-examine the crate split before anything is implemented

## Goal

Decide **what a component is** in `vitui`, and **which crate each piece of the answer lives in**,
before a single line of `vitui-runtime` or `vitui-components` is written.

Two halves, and they are one ticket because each answer moves the other. If a component is a plain
function, the crate split can be thin; if it must be a trait with a measure pass, the runtime grows a
node type and the split changes with it.

The engine is not in question. Its abstractions are the lowest in the system, eighteen tickets of a
wayfinder map settled them, and nothing here should reopen one. What *is* in question is everything
above the seam, which was deliberately left as fog while the engine was being decided.

The working analysis this ticket starts from is [`COMPONENT-HIERARCHY.md`](../COMPONENT-HIERARCHY.md)
(2026-08-17). It is a note, not a decision — every claim in it is fair game.

This ticket is likely to produce a **wayfinder map** rather than a single answer, the way the engine
effort did. The map's own Notes already say `vitui-runtime` and `vitui-components` are separate
efforts that get their own maps.

## Acceptance criteria

**A — the component model**

- [ ] A one-sentence definition of a component, and a table running every candidate through it:
      `text`, `fill`, `border`, `bar`, `plot`, `input`, `toggle`, `scrollbar`, `list`, `table`,
      `tree`, `panel`, `split`, `tabs`, `window`, `popup`, `modal`, `form`, `chart`, `picker`,
      `layout`, `grid`, `row`, `cell`, `gradient`, `scroll`.
- [ ] **Is a component a function or a trait?** Decided, with the consequence stated. This is the
      fork: a measure pass forces a trait with two methods and reintroduces a walked structure, which
      is exactly what ticket 05's *the clip stack is the call stack* deleted.
- [ ] **Intrinsic sizing: yes or no.** If no, state what layout can and cannot express. If yes, state
      what it costs in the component signature.
- [ ] **Identity** — where a name that survives between frames comes from. Six mechanisms need one and
      nothing decides it: focus routing, hit-testing, interest declaration, memo invalidation, overlay
      ownership, scroll association. Immediate mode has no mount.
- [ ] **The data contract** (`Rows` or whatever it is called) — the shape a collection component
      demands of application data. It would be the runtime's first trait; the engine has none.
- [ ] **The two-phase overlay protocol**, concretely: how an overlay request is expressed during the
      draw and satisfied after it, and how it is linked to its owner across passes.
- [ ] The component library's own vocabulary, added to `CONTEXT.md`. `cell`, `layer`, `surface`,
      `view`, `run`, `frame` and `damage` are the engine's and cannot be reused — a table needs other
      words for its units.

**B — the crate split**

- [ ] Each crate's responsibility restated, or the split changed, with the reason written down.
- [ ] **Does reactivity belong in `vitui-runtime`?** The map settled that reactivity must be
      replaceable without touching the engine, and ticket 12 proved it by building a TEA runtime and a
      signals runtime on one unmodified seam. If a *specific* reactivity ships inside `vitui-runtime`,
      "replaceable" means "fork the crate", which is not what was decided.
- [ ] **Does layout want to be its own crate?** It is a pure function library over integer
      rectangles with no dependency on anything else in the workspace, and it is independently useful.
      Argue it either way, but argue it.
- [ ] **Can `vitui-components` stay reactivity-agnostic?** It can if a component is a function over
      `(&mut Ctx, Rect, &Data, &State)`. It cannot if it binds to a runtime trait — which is question
      A's fork arriving in the crate graph.
- [ ] The **dependency policy** restated per crate, and `deny.toml` updated to match. The current
      policy says `vitui-runtime: no dependencies` and `vitui-components: case by case`; if the split
      changes, both sentences need rewriting, and "case by case" needs a precedent rather than a
      shrug.
- [ ] Confirmation that nothing in the answer moves the **engine's** boundary. If something does,
      that is a finding and it goes back to the engine map rather than being absorbed quietly.

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
  rendered at `docs/img/component-hierarchy.png`.
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
  - `scroll_area` is a **component** (L3), and `COMPONENT-HIERARCHY.md` §2's "scroll is not a
    component" is corrected in its new §8. It is *not* `list`: a scroll area costs its content,
    a virtualised list costs its visible window. New requirement C21 keeps them apart.
  - Expand/collapse is **one state machine** behind thirteen catalogue entries, so it ships as
    one helper (`collapsible`) rather than thirteen implementations.
  - The graphics-passthrough finding is **upgraded**: it blocks two families at full fidelity,
    with a price (~7 MB/s for full-screen video drawn as cells) and three honest options for
    the engine map. A **background image**, a QR code, a waveform and all video-player chrome
    are buildable today with no engine change.
  - New tickets: runtime R17 (scrolling), R18 (async work); components C13–C16 (scroll area,
    collapsible, media boundary, file preview exemplar). v1 inventory is now 33 components.
