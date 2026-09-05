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
  Register 48 entries and the 20-scene list, both green. The component-facing crate line is *built*
  rather than counted: `crates/vitui-components/tests/crate_line.rs` cannot name the engine.
- **`vitui-components` — implementation-complete.** All 46 tickets; spec §17's freeze is **29 of 29
  built**, as a value (`INVENTORY`) that tests iterate, with the documentation and verification
  obligations as functions over it. Register **234 rows, 229 evaluated** — 234 is components
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
- **`vitui-apps` — 18 applications**, one file each in `examples/`. A component ticket ships one, and
  since ticket 45 that is obligation **O7** rather than a habit: `vitui_components::consumer` joins
  the freeze against the import paths here.
- **Active work: `.scratch/vitui-production/`** (opened 2026-09-01) — the whole workspace's road to a
  published crate, sixteen tickets in five groups: the paperwork, the six unsubjected register rows
  (**all six standing**, by production 03 and 04),
  the fourteen hostile axes O5 still owed (**the group is closed**: `field`'s three taken by
  production 05, `table`'s two by 06, the overlay family's four by 08, the scroll family's three by
  09 and **`tree`'s two by 07**, which turned O5; **10 closed the paperwork behind it**), three
  tier-1 terminals nobody had run (**the group is closed on this machine**: WezTerm by 11, Alacritty
  by 12 and **iTerm2 by 13**, which was the last one reachable without Windows),
  and the release.
  **Two of the sixteen can start today, 02 and 14** — 02 is `ready-for-human` (it needs a public
  repository) and **14 is the frontier**, unblocked by 13.
  **Windows is last, as 16**, blocked by the publish — which carries the consequence that no shipped
  string may claim a terminal the conform suite has not asked. Everything on it was already true and
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

**Open questions — do not "fix" code to match one sentence of a spec without resolving the ticket.**
Five stand open, all of them the components map's and all filed by the layer above the one they land
in. **The runtime's architecture map has none left**: 36, the only one that map ever filed against
itself, resolved 2026-09-01.

- **Components architecture 19** (does a fold that costs the volume belong to O6), **22** (`Esc` over
  a plain `collection` is crate-private on purpose), **23** (`file_picker`'s popup has no keyboard at
  all), **24** (a `table` whose columns do not fill the band leaves the remainder **unwritten** — 66
  cells of every row in `examples/ledger.rs` at three hundred columns, and none at eighty; filed by
  production 06, whose scene 37 is the same axis on the row side), **25** (five entries of §16's
  twenty are demanded by `table` and drawn by nothing — the box junctions, filed by 20's own new
  join). **20 is resolved** (2026-09-04) and production **07 is resolved with it** — the narrow
  scene it blocked reads the partition 20 settled, and it filed no new question.

**The engine's architecture map has none left.** Issues 20, 21 and 23 read as open for a while and
were closed by production tickets 06, 03 and 02 — do not re-file them.

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
(cd conform && cargo run --example wezterm)  # a window and a control socket; exits 1 on scene 06
(cd conform && cargo run --example alacritty) # a window per scene, closed to take the capture; exits 1 on scene 06
(cd conform && cargo run --example iterm2)   # needs `EnableAPIServer` and a grant; 22/22
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
scripts/suspend-reader-gate.sh # a suspension does not vacate stdin, on a pty — register #31
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
(`.scratch/vitui-{engine,runtime,components}-impl/`) are **closed**, and so is
`.scratch/vitui-engine-production/`. **The one active backlog is `.scratch/vitui-production/`** — its
README is the queue and the blocking edges, and a ticket there may name an edge into another backlog
(`components architecture 20`), which is the authority the same way a local number is.

**A `Status:` line carries two vocabularies and they are not in conflict.** A freshly written ticket
says `ready-for-agent` or `ready-for-human`, which is triage. A session working one overwrites it
with `claimed` and then `resolved`, which is the frontier protocol above. `ready-for-human` means
*do not claim this without the thing it needs* — a public repository for 02, a Windows machine
for 16.

`tickets/` at the repo root is a **separate** surface — the hand-written backlog the `dispatch` skill
consumes — and holds the items that need the finished library. Do not migrate one into the other.
