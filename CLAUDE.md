# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

> **The local GitLab moved out of this repository (2026-08-22).** One shared instance now serves
> every repo on this machine: <http://gitlab.localhost:8940>, this repo's project is `repos/vitui`,
> and it is started with `devkit up`. The `infra/` stack here is gone — see `infra/MOVED.md` for the
> old-command-to-new-command table, and `~/Projects/devkit/README.md` for the manual.

## What this is

`vitui` is a Rust TUI library: fast, layered terminal rendering, meant to be the foundation a
component library stands on. Version `0.0.0`, unpublished, no stability promise before 0.x. MSRV
**1.88** — `Cargo.toml` is the authority and the `msrv` job in `.gitlab-ci.yml` is what keeps it
honest. It said 1.85 here for three releases after let-chains moved it.

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
  have earned a sixth bit for a misbehaviour that is not happening; `quirks.rs`'s **sixth** entry (JetBrains' IDE terminal abandons the ITU-T colon form of SGR 38/48
  and emits the remainder as text, so a screen fills with `:` and every glyph after one is pushed
  sideways — the first entry this repository got from a user's screenshot rather than from an
  instrument, and the one that cannot have a capture because JediTerm cannot be asked what is on its
  grid); and **architecture ticket 20**,
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
- **`vitui-signals` was built, and it is deleted** (ticket 18 built it 2026-08-24; architecture
  issue 24 removed it 2026-08-25, on the user's decision after the numbers were **re-measured rather
  than recalled**). *Reactivity stays out* stands and is not reopened — what went is the crate built
  to prove it. Three drivers of one screen measure **19 558.40 / 19 591.80 / 19 550.00 ns**, a spread
  of **41.80 ns — 0.21% of the slowest** — with **0 of 24 000 differing cells**, and the signal
  layer's whole per-frame work is **3.60 ns**. That is ergonomics over one runtime hook plus a cache
  over another, and the cache is `vitui_runtime::data::Memo`, which stays. **The 132× in the old
  spec and ADR was on the wrong denominator**: it is one derived value at one fold length — 31.8× at
  600 rows, 248.1× at 4 800 — and at the screen's own 600 rows the memo saves 44.79 ns against a
  19 558 ns frame, **0.23%**. ADR 0020 is **rewritten and not deleted**, carrying those numbers
  **inline**, because a result whose instrument no longer exists is an assertion again. Two findings
  survive the crate as facts about the tools: `cargo deny`'s `[bans] deny` bans a crate's *presence
  in the graph* with `wrappers` as the exception list — so a member with no dependents fails outright,
  which is why that crate had to be detached — and `request_frame()` is unreachable from a TEA
  `update` because every `Ctx` is gone by then. The still-true half of the old bullet: a frame that
  redraws one region declares **1 hit entry against 312 and 0 tab stops against 43** with 24 000 of
  24 000 cells correct — *fine-grained reactivity here is not expensive; it is a request to revert
  the hit index.* **No replacement crate, no `signals` feature, no reactivity module** — reopen
  issue 24 rather than restoring it.
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
- **`vitui-components` has started**: 40 of 45 tickets resolved (the last on 2026-08-28). `INVENTORY` is spec
  §17's twenty-nine-row freeze **as a value a test iterates**, with the five documentation and
  verification obligations as functions over it — so *which components must this gate run against* is
  answerable by the machine from here on. **O1 is met since ticket 36, O3 since 37, O4 since 38 and
  both halves of O2 since 39**, and the one left — O5, the one §17 says is worth more than the other
  four together — is `Unmet` and watched panicking, because a query with no evidence must fail loudly rather than pass: O4
  itself first returned **`Met` over 29 rows with no evidence at all**, since *an equality between two
  things that do not exist holds*. **Building the freeze contradicted four figures in the closed map** — the built count
  (19/10, not ADR 0033's thirteen-unbuilt), the count of empty families (five, not §17's two), §1's
  layer rule against §6's own composition, and the `layer` column being uncheckable without stated
  edges. All four are asserted as measured rather than bent to fit.
- **O2 is green — the gallery is twenty-eight panels, and the screen is a value in the library
  because two of §21's gates are measured on it** (components ticket 39, 2026-08-28; ADR 0046).
  `crates/vitui-components/src/gallery.rs` is the screen — `PANELS`, twenty-eight rows over the
  freeze's `built` column in the freeze's own order, each with the function that draws the shipped
  component — and `crates/vitui-apps/examples/gallery.rs` is the loop, the keys and the reports. This
  is also `tickets/002`, the user's own requirement, closed with it.
  **Where it lives is forced rather than chosen.** §21 names two defects to be measured *on the
  assembled gallery* — the sentinel and the palette swap, register rows 7 and 8 — both are components
  tickets, and a components gate is `cargo test`: **a screen only an application can draw is a screen
  no gate can measure**. The direction that opens — an application listing its own panels — is closed
  by a negative scan the application cannot satisfy by deleting anything.
  **Row 45's barrier had decayed into the near-miss runtime issue 22 warned about**: it read
  `Unreachable { needs: "ColorDepth" }` and named **the issue that lifted the barrier** as its own
  inverter, so the citation had been passing while meaning the opposite. Unreachable is 7 → **6**.
  **The colour axis of §16's matrix is not observable on a screen at all, and that is the finding.**
  `Theme::resolve` returns the **same `Paint`** for all thirteen roles at all four depths — measured —
  because quantisation is the engine's and happens before the mirror, which is ADR 0018 working rather
  than a hole. So the repertoire axis is **58 / 82 / 110** clusters read off the surface, the colour
  axis is **66 / 9 / 0** collapsed role pairs of 78 and **3 / 2 / 0** distinctions lost of 10 read off
  the theme, and the screen's own paint count reads **ten in all nine cells** — a column that moves on
  neither axis, printed beside them because *a counter on the wrong side of the question is not a weak
  gate, it is a green one*. **On the surface it is a zero**: a colour-depth change moves 10 cells at
  300x80 and 5 at 100x30 and **0 of them on any panel**, the movers being the two chrome rows printing
  the depth's own name — and that number had to exclude **both** chrome rows rather than one, because
  at 100x30 the status bar is elided before it reaches the depth and excluding the heading alone read
  0 there and 5 at 300x80. *One size agreeing with a law the other breaks is how a gate over one size
  stays green.* The matrix also needed the **whole gallery** rather than one page: read on
  page one alone the repertoire axis reports `Extended == Unicode`, true of that page and false of the
  screen, because `plot` is the one braille construction and it is on page two.
  **Three figures do not reproduce.** *`Danger`, `Warn` and `Ok` all quantise to bright white at
  sixteen colours* is true of **8 of the 14** shipped schemes and of **14 of 14** at
  `ColorDepth::None` — the right claim about the wrong rung. `t` costs **~70 ns** against the
  criterion's 291, and `with_glyphs` before `resolve` costs **~550**, because a declared repertoire is
  a real input to the ten distinction bits. Both are asserted as measured; the palette was
  deliberately not swapped to make the old sentence true.
  **`'f` costs the table one field**, and it is components 26's answer to the same borrow: two of the
  twenty-eight own an overlay and take `&'f mut` of what their popup body captures, a `fn` pointer's
  elided lifetimes are fresh and unrelated, and tying the bag's borrow to `'f` makes the *first*
  panel's borrow last the frame — so the two arrive as a pair of `Option`s a panel **takes**, where a
  second take panics because a panel drawn twice is two widgets under one id.
  **The wrapper-list criterion is met by the exception being unnecessary**: the gallery names no
  crossterm — `Driver::attach` owns raw mode, the alternate screen, the input and the restoration —
  and a gate asserts `deny.toml`'s line still reads exactly
  `{ name = "crossterm", wrappers = ["vitui-engine"] }`, because the gallery is the reason it did not
  move.
  **The budget is measured in the gallery**: **~390 µs at 300x80** against the 1 ms full-screen class
  and **~79 µs at 100x30** against the 100 µs typical frame, `writes == distinct` and `merges == 0` at
  every size, and **zero allocations on every page**. Three of §20's four over-budget
  single-component screens are on it, which is what makes those two sentences consistent: the budget
  is per class. The allocation window found one defect — two preview drawers spelling their row labels
  with `format!`, **4 a frame on page three and zero on every other page** — which is components 30's
  `player::chrome` finding a second time.
  **A review then found six more, and three are a finding rather than a fix.** A **resize** is the one
  event that changes the page count without going through a key, and an index past the end yields an
  empty range rather than a clamp — *0 panels · page 4/1* over a grid nobody writes, standing until the
  next keystroke. `changed > 0` was the weak gate this module's own header warns about, on the axis the
  module is about; the reading is `changed_on_a_panel`, and it had to exclude **both** chrome rows,
  because at 100x30 the status bar is elided before it reaches the depth and excluding the heading
  alone read 0 there and 5 at 300x80. And the pty gate's needle was an **ERE in which `?` is a
  quantifier**, so `'\[?1049l'` is *optional bracket then `1049l`* and matched any capture containing
  that substring — the gate whose whole subject is byte-exactness. Three more were smaller: `shape`
  never answered the queued work, the clear bypassed the caller's ink (so `Ctrl+O`'s counters were
  blind to `w × h` writes on the one frame a reader turns them on for), and the gate sat in the `idle`
  CI job whose cache has only ever held release artifacts of the engine. **And one was in
  `crates/vitui-apps/src/lib.rs`'s own table**: the gallery's `uses` column listed `gallery::PANELS`,
  which `crate::gallery`'s scan forbids that file from spelling — a column documenting an application
  doing exactly what a gate one crate over refuses, unchecked for fifteen applications.
  **Four things the gates caught while being written**: the tiles are laid out by boundary arithmetic
  and not by multiplication (which leaves the column past `n * (w / n)` to nobody and is
  indistinguishable at every width that divides — swept 20x8 to 119x29); each tile is keyed with
  `Ctx::with_key`, because one call to `panel_into` in one loop is one `Location::caller()`; the pty
  gate compares two byte **offsets** rather than looking for the epilogue, because a capture where the
  backtrace came first contains both strings; and a file here may not spell `GlyphSet::`, so the
  repertoire axis goes through `chart::raster::RUNGS` — the third caller that const was made public
  for. Register 218 → **223 rows, 208 evaluated**.
  **Rows 7 and 8 were left red on purpose and row 7 is green since components 40** (below):
  `gallery::shape` and `gallery::swap` print both numbers as measured here — 10 252 unwritten cells of
  24 000 at 300x80 on a **steady** frame, and a swap that keeps 0 of 13 748 paints, 64.6% of clusters
  under a rung change and 99.9% under a tier change — so components 40 and 41 start from a figure
  taken on this screen. Reading one axis for the other is how a swap gate goes green. **`unwritten` is
  a steady frame's**, and it has to be: `crate::app::Clears` writes every cell on the first frame and
  on a resize, so *cells nobody ever wrote* is zero on any screen that clears and says nothing about
  any component.

- **Register row 7 is green, and the detector spec §2 prescribes could not be built at all**
  (components ticket 40, 2026-08-28; ADR 0047). *Every cell of the rectangle written at least once* —
  **0 of 24 000 at 300x80, 0 of 3 000 at 100x30, 0 at eight sizes on every page** — the second of the
  three red rows whose failing set was a **defect** rather than a missing subject, and the longest
  pinned. Register 208 → **209 evaluated of 223**, red 3 → **2**.
  **The sentinel is a reading of a recorded surface and not a probe of the screen.** §2 asks for a
  stamp on the base layer and a count of survivors; two of `counters::sentinel`'s three barriers were
  gone and the third is **ADR 0023**, which is a decision. It did not need lifting, and the argument is
  the pair: `writes == distinct` has always been read off `Tally`, whose union has been in root
  coordinates since components 19, so **the second half is that same union against the area** — read on
  the screen the pair would be an equality between two different instruments. The recorder is also the
  **stricter** one, because a verb that skips the caller's `Ink` makes the number *larger* and fails
  loudly, where a surface probe counts the engine's own clear and passes quietly. That defect is on
  this map twice: components 15's `ledger` drew a table cell with `cx.text` rather than through the
  ink it was handed — 78 cells and 10 verbs on a frame that wrote 1 560, on a screen that looked
  correct — and components 39's gallery cleared through `Direct`.
  **Three mechanisms and a fourth that is no component's**, which is §2's sentence as four numbers:
  `panel` hands back `Frame::interior` (**294**), `collapsible` hands back `Disclosure::used`
  (**432**), a `scrollbar` is three columns of a wider tile because *this caller* narrowed it
  (**630**), and two slots of a six-by-five grid hold no panel at all (**1 600**). The first two were
  named in a return value and the gallery discarded it. `gallery::Remainder::LeftAlone` keeps all four
  runnable on the same twenty-eight call sites and the gate is watched leaving **2 956 cells at 300x80
  and 145 at 100x30**, attributed per tile rather than as a total — and **at 80x24 it leaves 0**, which
  is why the sweep is a sweep.
  **Twenty-six of the twenty-eight already wrote every cell of any rectangle they were handed, and the
  two that did not are the two overlay owners** — `select` and `file_picker`, **576 cells of a 48x13
  interior each**, exactly the two whose body is in another layer (§12). **Their remainder could not be
  named**: §2's third clause is *the cells it does not write are named in its return value* and both
  return the runtime's `Response`, so writing them is the only reachable answer rather than the chosen
  one. The paint is the face's, because `press_into` declares the hover award over the whole rectangle
  — a rectangle a component paints one row of and awards all of is one whose hover repaints cells
  nobody wrote.
  **The per-component form runs over the shipped call site and over the constructions**, which is
  §21's own account of why the row survived: *the per-component forms were report-only across nine to
  twelve binaries*. `gallery::shot` at five rectangles for each of the twenty-eight, and
  `tests/golden.rs` over the **thirty-three** constructions at all three rungs — the second was
  already true and asserted by nothing, and `golden::UNWRITTEN` being `▪` rather than a blank is what
  makes it a gate rather than a coincidence.
  **Row 8 did not move with row 7, and §21 pins the two in one sentence** — *the swap excess equal to
  it on five of six*. `swap` carries **one surface** across the change and the first frame clears, so a
  cell nobody writes on a *steady* frame is still a cell somebody wrote once: it is inside `written`
  and counts as `kept`. A rung change at 100x30 keeps **2 005 of 3 000** with row 7 red and with it
  green, both arms run. What moved is **1 248 cells at 300x80 in the other direction**, because twelve
  panels are handed the whole of their tile now instead of its first row and `meter` and `slider`
  **fill** what they are handed with glyphs a repertoire change moves.
  **`crate::form`'s unwritten tail stays, as the exception §21's third refinement asks to be named.**
  The form is a **fixture**: its two helpers each write a partition of what they were handed and
  `block` *returns* the interior it did not write. The tail is what buys two of ticket 06's three
  measurements, and a gate run over every screen in this crate would have to delete them to go green.
  **Scene 26 is stood up and its own `inverted_by` had named components 39** — the screen has existed
  since 39 and the scene went on reading *not played: `components 39` builds the subject*, a citation
  decayed into naming a resolved ticket. `covers` stays empty (O5 unmoved) and `stands` is every built
  row of the freeze, which is the answer `scenes_for("checkbox")` had none of before; **five reports
  asserted counts that moved with it**, found by running all twenty-six, which is components 20's
  *`cargo test` does not run an example* for the third time.
  Scenes **30 stood up, 3 unsubjected, 0 red**. The screen writes 24 000 cells at 300x80 against
  13 748 before, `writes == distinct` and `merges == 0` at every size and page, zero allocations, and
  the frame is still inside §20's full-screen class.

- **O4 is green — thirteen keyboard contracts as declared data, and *registered* is what the machine
  answers** (components ticket 38, 2026-08-28; ADR 0045). `crate::contract` is the value:
  a `Contract` per component that reads a key, a `Bind` list, and a `live` function pointer that
  **runs the shipped widget**. `crate::obligations::o4` is `Met` over **13** — the union of the two
  lists and not the freeze, because sixteen rows read no key at all — and the chord-for-chord
  equality is register row 216. **The equality could not have been between two readings of one
  declaration**, which is `obligations`'s own header one level down: a `registered` list derived from
  `CONTRACTS` agrees with a `documented` list derived from `CONTRACTS` for ever. So `registered` is a
  sweep of **162 triggers** posted at a headless driver with the widget's id holding the focus,
  answered by `Driver::unhandled` — *the keys this frame's batch carried that nobody took* — read
  after the frame. A click has no such window, so its observable is the **picture**: the same drive
  with and without the modifiers, compared through `Canvas::diff`.
  **The control arm is the mechanism and a finding at once.** `unhandled` says *nobody took it* and
  **the runtime is one of the takers**: `Tab` and `BackTab` at all five modifier states are the focus
  walk, **10 of 162**, and without subtracting them all thirteen contracts register ten chords they
  have never heard of — an equality that would have been reconciled by *declaring* them. The control
  is `button`, a row of the freeze and not a fixture, so what it makes checkable is *a tab stop that
  reads no key*. That is §21's *`Tab` inside a trap* from the other side: it is the runtime's key and
  no component's.
  **Five chord leaks in code that was already green, and register row 5 could not see one of them** —
  *a chord pressed into every focusable types nothing* is a claim about a **buffer**, and moving a
  caret, opening a list and clearing a selection all type nothing. `field` answered `Ctrl+Left` and
  the whole cursor and edit set; `select` and `file_picker` opened on `Ctrl+Down` and `Alt+Enter`;
  `collect::from_key` cleared a selection on `Ctrl+Esc` and toggled a row on `Alt+Space`; and
  `input::popup_body`'s own refusal committed on `Ctrl+Enter` — **the one with nowhere else to go**,
  because that loop runs *inside* a trapless overlay where the application has no other reader. **The
  sharpest is this ticket's own subject**: `Ctrl+Left` is what a user pressing for word motion means,
  word motion is `contract::ABSENT`'s one row — the engine exports `graphemes()` and `width_of()` and
  no word iterator — so the widget was swallowing the accelerator *and* answering it with a cluster.
  Each repair is one `keys::is_chord` guard, `collect::defective::from_key_on_code_alone` keeps the
  replaced spelling runnable, and **none of the four broke a single existing test**.
  **`Bind::ignores` is spec §3's rule declared rather than a convenience**: `SIGNIFICANT` is
  `CTRL | ALT` and Shift is deliberately not in it, so a component that filters through `is_chord`
  and matches on `code` answers `Shift+X` as it answers `X` — two chords where a help bar prints one
  line. It is per bind: `collection` declares `Up` and `Shift+Up` as **two**, because there the
  second is a different action.
  **A contract is read where it is whole and what the other configuration removes is a number.**
  `field` at `WrapKind::Ruler` loses `Up`, `Down` and `Enter` — ADR 0042 as **6 spellings**;
  `collection` at `Mode::Single` keeps **2 of 3** pointer gestures, and *the gesture that disappears
  is the one that works*, because a ctrl-click **is** a plain click there while the two extends the
  mode refuses leave a different picture. **`select` is the third instance and it is two keyboards
  rather than two configurations**: shut, the owner's loop runs; open, *the popup takes the keyboard
  from its owner*, and the contract is the **union** of the two seats — which the sweep needs a third
  frame to reach, because the handover happens during the frame after the plant.
  **And the family is not one keyboard: `file_picker`'s popup has none at all.** It draws a plain
  `collection_into`, seats no focus and declares no refusal, so an open picker has no arrows, no
  `Home`/`End`, no type-ahead and **no way to choose a file** — mouse only, on a screen that renders
  perfectly. **35 spellings against 8 over one family**, which is components 32's *two copies of a
  drawing that has already been wrong once is one copy too many* with the keyboard as the half that
  stayed transcribed. Not repaired here — seating a focus and minting a refusal inside `picker_body`
  is a component's keyboard being *designed* — so it is **components architecture issue 23**, held by
  `PICKER_IS_MISSING = 27` asserted exactly with the owner's four asserted equal.
  **A fifth defect was in the gate**, and it is ADR 0031 arriving in an instrument: `Order::built`
  stamps `Revision::fresh()` and a collection handed an unseen revision **clears the selection**, so a
  probe rebuilding its index inside the draw reported `tree` deaf to all three pointer gestures. Two
  more fixture facts came the same way — the click seeds at column **6**, because at column 1 a
  `tree` row of depth 1 is indent and chevron and the press toggles a fold; and every probe's labels
  begin with `a`, `m` and `z`, because **a collection's type-ahead consumes a letter only when a row
  starts with it**.
  The walkthrough and *a chord types nothing* were already green and are cited rather than rebuilt
  (rows 34, 88 and 5). **Nothing here decides whether a binding may render as unavailable.**
  Register 215 → **218 rows, 202 evaluated**.
- **O3 is green — thirty-three screens, one per construction, in the engine's format and not a second
  one** (components ticket 37, 2026-08-27; ADR 0044). `crate::golden` is the format, the screen table
  and the play; `tests/golden.rs` is the three-rung sweep and the equalities;
  `examples/golden_numbers.rs` is the report. **A golden is the one instrument on this map that
  catches a wrong cell** — §17's own reason O1 and O2 do not substitute — and the engine's
  `golden.rs` is `pub(crate)` throughout (ADR 0023), so the *code* cannot be shared and what must not
  be forked is named by path in `golden::FORMAT_OWNER`, with the owner's four load-bearing sentences
  read back out of that file against a flattened source. `VITUI_BLESS=1` is the owner's command;
  blessing in CI is refused.
  **The population is `built`, which is ticket 36's finding a second time**: §17 states O3's count
  over nothing at all, `spinner` is the one unbuilt row, and *a golden of a function that does not
  exist is not a screen anybody can draw* — so asking for one pins a row nothing here can invert,
  which is `Verdict::of`'s vacuity failure in mirror image. 33 over 28 rows; 34 over the freeze; the
  population moves on its own.
  **The finding is the legend's ceiling and it is a fact about braille.** Twenty-six keys, a
  twenty-seventh refused as *not reviewable by eye*, and a `plot` at `Extended` is 256 states a cell:
  **41** keys at 24x6 and **20** at 12x4, against **16** at Unicode and **0** at Ascii on the same
  rectangle. **The one place on this map where the *format* decides how big a screen may be**, gated
  with both numbers so a rasteriser change fails with a count rather than inside the renderer.
  **Two of the three figures do not reproduce**: `plot`'s two block rungs are **21 cells over 3 rows**
  where §17 says 882, and the dense screen at 300×80 between Ascii and Unicode is **1 065 cells over
  78 of 80 rows** where §16 says 7 276 over 80 of 80 — and *the two rows that do not move are the
  finding inside it*, since a cell count alone reports the same screen as *the repertoire is
  everywhere*. What does reproduce is a rule nobody had counted: **0 of 33 screens keep a cluster
  outside printable ASCII at the ASCII rung**, and the dense screen keeps exactly **one** — the em
  dash of `dense::HEADER`, a fixture's own title and no component's glyph, which is §16's *text is
  legitimately a component's own content* as 1 cell of 24 000.
  **Three departures from the owner's format, each because the surface is different**: a cell nobody
  wrote is `▪` and not a blank (§2's partition rule is the subject here, where the engine's unpainted
  cell is a finding); the style plane names a **`Role`**, because ADR 0018 says a component names a
  role and never a colour, with paints outside the thirteen printed as `custom #n` so two identical
  legend lines cannot happen; and the failure report carries **cells and rows** beside the owner's
  first-differing-line.
  **`select` and `file_picker` are photographed shut**, and that is the overlay family rather than a
  choice about two screens: `Ctx::overlay`'s body is `move |cx|` and takes no ink — a body is
  borrowed for the frame and a `&mut I` is not — so **a popup's interior cannot reach a `Pen` at
  all**, and a golden of an open `select` is four rows of *a cell no verb wrote* under one row of
  face. `overlay`'s own screen does show an interior, because `overlay_into` is a shell and not a
  layer. Register 209 → **215 rows, 199 evaluated**; the rung is named in **five** files and none of
  them is `src/`.
  **A review then found this ticket's own equality unable to see its own subject.** A plane's key is
  first-appearance order, so a plane encodes the **pattern** of distinct clusters and not the
  clusters — two screens whose non-ASCII cells are swapped one for one have *identical planes*, and
  a count over them answers `(0, 0)` for the one regression the equality exists to catch. On the
  plot's own two block rungs it under-reports **12 against 21**. Every equality and every figure runs
  over `Canvas::diff` now, with both numbers asserted; `divergence` stays for the **file**, where
  there is no canvas and the line comparison has already caught the legend. Four more beside it,
  each latent: a multi-frame shot that ended its frames once, so an award declared on frame one would
  land on frame two's cells; a row width taken from two of the four planes; a negative arm asserting
  a property of its own string literal rather than running the scan; and `describe_paint` taking the
  **first** matching role, so the thirteen paints are now asserted pairwise distinct — the runtime's
  `Roles::pick` exists because a stub palette once had `Dim` and `Border` both on `indexed(8)`.

- **`field` is the fourteenth component, and §11's API consequence is a deletion three `compile_fail`
  pairs keep deleted** (components ticket 24, 2026-08-26; ADR 0036). `input` and `textarea` are **one
  type and one flag** — `edit::WrapKind`, a break rule — and the near-miss is refused on a
  measurement: a ruler spelled as a `Words` index at `u16::MAX` is **1 row against 500** over a
  megabyte, so the caret's row start is byte 0 and `Left` is back to O(prefix), which is the one thing
  the flag exists to prevent. **`Caret`'s fields are private and there is no `set_caret(byte)`**,
  because an arbitrary offset is not a cluster boundary and everything downstream preserves the bad
  caret faithfully while the screen renders exactly as it should. One `Left` at the end of a pasted
  megabyte is **6.5–7.6 µs against 31 300–38 400 — 4 800–5 000×**, where §11 remembers 9 655.
  **The gate found a real defect in the machine, and it is §11's headline arriving on the *cheap*
  direction**: a forward cluster step that crossed a row boundary and kept accumulating reported
  column **39 on a row whose start was 0**, because a greedy break consumes the space it broke at. A
  caret's column is a *screen* column, so `Right` and every edit now re-read it from the caret's own
  row start — one row rather than one prefix, which is what the index is for.
  **`Ink` gained a fourth verb and the reason is a counter on the wrong side of the question.** A row
  is a partition of its width, and written as `text` then `run` the pad makes `verbs` separate two
  builds by how full their rows happen to be — `run` returns without a verb at zero, so a row that
  fills exactly costs one verb and a row that does not costs two. On the memo-key defect a stale
  index's rows are the whole line truncated, so **`verbs` reports the defective build as cheaper**.
  `Ink::pad_to` stages the row and its pad and blits once: one verb a row, no allocation, and the
  counter is blind again — which is what register row 82 asserts of all four gates.
  **Three of §11's figures do not reproduce and are asserted as measured.** The regions are **1
  against 372** where §11 says 79 against 621 (both magnitudes are a prototype's screen; the shape is
  one entry for the widget). Coalescing is **4.69×** where §11 says 2.44× — and the rule underneath
  it is the **word** break rather than the line break, because §11 gives the reason in the same
  sentence as the ratio and a run closing only at a line makes undoing a sentence *one* press, which
  is a checkpoint and not a history. And the frame at 300×80 is **1 033–1 056 µs of clusters against
  428–450 of ASCII** — `tree`'s 436 inside the noise — so §11's 82.4 µs is a prototype's screen and what this
  one costs is text measurement over the engine's tables. The ASCII row is printed beside it rather
  than engineered away.
  **What is flat is what was claimed**: 24 000 writes / 80 verbs / 1 region / 0 merges / 0
  allocations at 100 kB and at 1 MB, with the allocation zero held over fifty steady frames with the
  probe installed. The undo ring is bounded on **entries and bytes** — one large paste is one entry —
  and sixteen undos that each answered `true` leave the document unrestored, which is
  indistinguishable from success unless `truncated` says so.
  **The scenes went green together**, the same shape 14→15, 16→17, 18→19 and 21→22 had, because they
  were pinned on one fact and it was the subject: `crate::document`'s two screens and the cluster
  corpus now draw *through* `input::field_into`, and the caret and the wrap index moved out of the
  scene file into `crate::edit`. Register 135 → **143 rows, 127 evaluated**; scenes **1 red, 24 stood
  up**, and the one still red is the overlay family's.
  **Two runtime cadences the gates had to be written around, and both were found by a gate being
  wrong first.** The caret is gated on `Ctx::is_focused` and not on `Response::focused`, which is one
  frame apart — a `Response` carries the focus as it stood when the widget *declared*. And a screen
  that seats the focus at the end of its own draw has **no caret on its first frame at all**, so
  `document::play_field` plays two. Beside them: **a test that calls a `#[track_caller]` component
  from two call sites is two widgets**, and the caret is the instrument that notices — the second
  frame minted a different id, the id the first frame focused had not drawn, the vanish rule cleared
  the focus, and the caret was simply *absent* on a screen that looked right. ADR 0027's defect
  arriving inside the gate for it.
  **A review then found two gaps the gates did not cover, and both are register rows.** The wrap
  index partitions what is **drawn** and not what is held — `layout::text::wrap` trims each piece —
  so a caret placed by asking which drawn row *contains* its byte is lost by **one trailing space**,
  with the terminal cursor gone and the screen otherwise perfect; on a contiguous `Ruler` index the
  same fact fails the other way and the caret jumps to the start of the row it just left. `End` on
  `"hi "` landed at byte 2 and `Ctrl+A` selected `"hi"` for the same reason. And **`field` declared
  `Interest::SCROLL` and consumed nothing** — ticket 20's defect class on a component that gate has
  no subject for, and worse than declaring nothing at all, because the field is the topmost region
  over its rectangle. The new gate then caught the repair's **own sign error**.
  **The application is `compose`**, and its own finding is that **`q` cannot be a quit key here at
  all** — a focused `field` consumes every text-bearing key into its buffer, which is what it is for.
  `ledger` and `explorer` bind `Ctrl+Q` beside `q`; there is no `q` to bind beside here, because
  typing one is the point. Its status bar prints the `(byte, column)` pair, the visual row, the width
  the index was built at and the ring's two bounds, so §11's four gates are visible while a person
  types and the fourth of them moves while the terminal is resized.

- **`select` and `overlay` are the fifteenth and sixteenth components, the scene list has no red row
  left, and three of §12's seven columns are replaced by findings** (components ticket 26,
  2026-08-26; ADR 0037). The family is **`overlay::FAMILY`** — §12's table as a value a test
  iterates, **three kinds on two axes over eight entries** with no fourth on either side — and
  `overlay` is the **one shell** every one of them draws inside: the blur position, the reserved bar,
  the barrier, the trap, and **not one cell of the interior**, swept as an exact tiling from 1×1 to
  17×17 under five extents. **Modality is one `bool` on the request and forces no construction.** The
  two axes are not one: Axis A separates exactly `Kind::Transient` and Axis B exactly `Kind::Dialog`,
  so neither column is derivable from the other — and Axis A became a fact about the **kind** rather
  than a warning about placement, because a declaring overlay flips **99 in 100 frames** over a
  rectangle covering its anchor and a `Kind::Transient` flips **1** over the same rectangle. *The
  rectangle is not the fix.*
  **The size is a sizing function's and the drawn extent is a fixpoint at zero.** `popup_size` takes
  the data and the room and no draw context; `select` widens it to at least its own rectangle, which
  is the component's job because a sizing function never sees the anchor. Granted over two frames on
  a 12-row and a 3-row screen: **(20, 4)/(20, 3)** for the rule, **(20, 4)/(20, 4)** sized to the
  content, **(20, 0)/(20, 0)** from the drawn extent — two frames and not one, because the last is a
  *fixpoint* and one cold frame at zero is indistinguishable from a warm-up. **The gutter is decided
  in the body in 0 passes against §9's 3**, because a popup's content is as wide as the viewport it
  was granted and there is no second axis to couple through; sized to the content instead, **1 of 4**
  rows is reachable by nothing, since `place` clamps a position and **never a size**.
  **Three of §12's seven columns do not reproduce**, and the third is the finding: `allocations` is
  `n + 1` (ADR 0034 deleted the arena), `content layers` is 1/2/1 against 2/4/3 (a shadow layer
  `OverlayOpts` has no field for), and **the menu delta is +4/+2 where §12 says +6/+6** — §12's six is
  three rows twice, *each a target of its own*, and §5 collapses a menu into a `Mode` of `collection`
  where a collection declares **one** hit entry however many rows it has. The other four rows
  reproduce exactly.
  **Four findings at seams, every one found by writing it.** The blur's position is `Response::local`
  and **not** `Response::hovered` — `hovered` is a *previous*-frame guess and reads false on the frame
  the layer is placed, which is the frame the optimistic focus arrives on, so a popup dismisses itself
  out from under a pointer standing on it. **A dialog has no blur position at all**, which is narrower
  than Axis A: its barrier already withholds the pointer. **A nested overlay's state has to arrive as
  an `Option` the body takes**, because an overlay body is `FnMut` and cannot move a capture while a
  reborrow is shorter than `'f`. And **a component cannot share an overlay owner without sharing its
  own identity** — two `select`s under one id are **3 regions against 6**, the second request inert
  *and* the second widget merged away, so a second overlay mints.
  **The scan the red scene carried could never have matched**: `DECLARATIONS` read `pub fn select(`
  and spec §1 already says *the fifth component opens an overlay, and `'f` costs it two annotations*.
  A scene green on the old needle would have been green by deleting the lifetime. `&'f mut` is the
  mechanism rather than an annotation, and the pair's diagnostic was **read off the compiler**:
  `E0502 … argument requires that popup is borrowed for '1`, with **no mention of the overlay
  anywhere**. The sentence §1 says `Ctx::overlay` owes is written on the runtime's own item, and the
  instrument is a **scan of that file** against a flattened source — a doc comment is not an item, and
  in the shipped file one phrase is broken across a line by the formatter.
  Register 143 → **155 rows, 140 evaluated**; scenes **0 red, 25 stood up** — row 89 and scene 14 were
  the last pinned pair, and `scenes::line` is where the shape of a red row's own report stayed
  reachable from a test after the list ran out of red rows.
  **The application is `console`, and it earned most of this ticket's findings.** It met two of the
  diagnostics §12 predicts in itself plus a third: the palette's answer as a local is `E0503` at the
  app's own read, `sort_popup.granted()` after the handover is `E0502` at the call site, and
  `self.status(cx, …)` after a `select` is `E0502` on a method call three lines from a popup nobody was
  thinking about. **And it found four defects in code that was already green, none of them reachable
  from any gate here, because every other instrument posts its keys at the *owner*.** The popup never
  took the keyboard, so the **arrows were dead** and a shut widget eating them looks exactly like a
  popup that works; **`Response::focus_left` on the owner stops being the blur signal** the moment the
  popup takes the keyboard, because the owner no longer holds the focus and has none to lose, so a
  blur is `seated && !inside && !over` and `Blur::PositionAlone` is the arm that forgets the middle
  clause (**cursor 0 of 4**); **a body that answers through the inbox owes the frame that delivers
  it**, or the choice lands on whatever wake happens next, which for the keystroke that made it means
  never — `Driver::unhandled`'s own finding (C25) one layer over; and **a shut `select` consumed
  `Esc`**, leaving an application whose quit key is `Esc` with none. Two more are the app's own: its
  loop needed the crate's `if !unhandled.is_empty() { continue; }` or `Ctrl+P` was invisible until the
  next keystroke, and the palette's **header is a row of the popup** while `popup_size` sizes a *list*,
  so the last command was off the bottom. **One is a question, filed as components architecture issue
  22**: `Esc` cannot close a modal whose body is a plain `collection`, because §5 claims it as
  `Gesture::Nothing` and `collection_chorded`'s first refusal is crate-private *on purpose* — so a
  component here can own `Esc` over a collection and an application cannot. `console` owns `Ctrl+P`
  instead. Its status line prints what the popup was granted against what it wants and what the screen
  has room for, so §12's short-screen case moves while a person resizes the terminal: at 100×12,
  fourteen options are granted **24×12** and the bar stands.
  **A review then found three more, none reachable from a gate that existed**, and the sharpest is
  §12's own sentence: *on the way out the owner refocuses itself* was **missing**, and the gate that
  asserts it **passed** — with one widget on the screen the vanish rule picks the owner by itself, so
  the defect only shows where there is somewhere else to go. The application has two selects, and `Esc`
  sent the keyboard to the other one. The label was padded to
  the whole width and the ellipsis written **over the pad's last cell**, so one cell of every
  *truncated* `select` was written twice — `glyphs::elide` has already reserved the marker's cell, which
  is what makes the mistake easy, and §2 had no partition gate over `select` at all. And **a popup
  dismissed by a blur could not be reopened**: what the body reports survives the dismissal, so the
  clause fired on the frame after the reopening — the latch had to move to the **owner**, because
  `SelectState::open` is the only thing that knows an opening has begun.

- **The picture scene was red on purpose and components 30 stood it up** (components ticket 29,
  2026-08-26; runtime architecture issue **34**). The screen is
  `crates/vitui-components/src/picture.rs`, and for one ticket what it was played over was a
  **stand-in painter**: `INVENTORY` has no media row at all, which is §14's own *no v1 component*, so
  the subjects are read out of `src/media.rs`. Read the bullet below it for what inverting it cost;
  what follows here is the screen's own findings, every one of which now reproduces **through the
  component**.
  **A picture's ladder is derived rather than branched, and that keeps register row 26's exception at
  one file.** `crate::media::sub_rows` is `geom(Kind::Bars, set).sy.min(COLOURS_PER_CELL)` — **1 / 2
  / 2** out of the bar ladder's **1 / 8 / 8** — and the derivation is the argument: the eighth blocks
  and the half blocks are one contiguous run of block elements, so a rung that offers a bar eight
  sub-rows offers a picture the half block, and two colours a cell is the ceiling. **`Extended ==
  Unicode` fires in both directions** — the same screen on the *mark* ladder, 1 / 4 / 8 bits, differs
  at the top two rungs.
  **The custom census is real and it is not a cost**, which is the finding under §14's headline:
  24 000 customs / 24 000 verbs / 24 000 writes / 0 regions / 0 fills, **0 of 47 620 adjacent cell
  pairs of a photograph sharing a value** — and the same 24 000 cells drawn from the thirteen roles
  spend **zero** customs and take **the same frame**, 1.384 ms against 1.408 over two hundred
  interleaved rounds. The census is evidence about *structure* — a cell outside the theme is a cell
  no run and no `fill` can reach — and not about the clock.
  **Three of §14's figures do not reproduce.** *50.7% of distinctions gone at sixteen colours* is a
  prototype's picture: this photograph loses **35.84%** and this gradient **99.27%** of the same
  23 920 adjacencies — **2.8×**, and **on the distinction axis the source decides a great deal**,
  which is the opposite direction from §14's B/cell claim that it moves the wire by 9%. *0.97 against
  0.50* **cannot both come from one cell**, since two modules a cell is exactly twice one at every
  cell aspect — a nominal cell's floor beside a measured cell's ceiling, both printed, the gate the
  relation. And *261–357 µs against a 100 µs budget* is a prototype's screen and the wrong class
  besides: this one is **1.37–1.50 ms**, one verb a cell, which is §20's full-screen class and
  **1.37–1.50× over it** — printed beside the number rather than optimised against. **What reproduces exactly is the floor**: at `ColorDepth::None` a picture
  keeps **0 of 23 920**, which is *degrades to a description of itself* as a number and which no
  other screen in this crate can produce.
  **The QR readback models the terminal, and the first version of it did not**: decoding a cell's two
  modules from its **paint alone** passed the inverted build **0 of 441**, because the inversion
  writes the same cells with the same four paints one character apart. Decoded as a terminal would it
  is **219 of 441** — and **one module a cell reads back perfectly and is still not a QR**, so the
  readback is blind to the aspect and the aspect is blind to the pairing.
  **Two of the ticket's criteria are bytes, and no crate above the engine can read one** (runtime
  issue 34). `Driver::headless` moves a `Vec` into the engine and never returns it, and `Output`,
  `Clock` and `Overrides` are **not in `ENGINE_NAMES` at all** — issue 22's construction rule
  arriving on `Config` itself, in its weaker form, because `Config` has a `Default` and a consumer
  can build one it cannot configure. The reachable form is gated instead: **24 000 of 24 000 cells
  changed by a translation of one whole row** and `[24 000, 0, 0, 0]` over a still picture's first
  four frames. **The colour half is reachable only through a contrivance and the contrivance works**,
  which is why its row is `Evaluated`: `Theme::custom` says the caller owes a branch on the
  terminal's capabilities and the only wire question on the surface is over the thirteen **roles**,
  so `picture::wire_differ` authors a theme per colour pair onto `Danger` and `Warn`, which
  `Roles::from_palette` takes verbatim from `base08` and `base0A` — about 24 000 theme constructions
  for one screen. Register 155 → **162 rows, 145 evaluated**; scenes **1 red, 25 stood up** — and components 30 took that last red row, leaving **168 rows, 152 evaluated** and **0 red, 26 stood up**.
  **A review then found nine, and three are the shape this crate keeps meeting** — the recorder and
  the defect share a coordinate system, so the gate cannot see it. `qr_into` read its rectangle as an
  *origin* and `Pen` records at root coordinates with no clip, so a symbol painted past its rectangle
  read back **0 of 441 wrong**; `adjacent_equal` counted two cells nobody wrote as sharing a paint
  (`None == None`), which over a symbol in a corner is **47 250 of 47 620** — a blank screen reading
  as maximally fillable; and `readback` cast a negative `i32` origin with `as u16` and reported
  **441 of 441 wrong** for a symbol drawn perfectly. Beside them, `Source::Gradient` **masked** its
  green channel where it should have saturated — `sub_y = 159` is `0x00ff80` and 160 is `0x000080`,
  reachable from `shift` and from the braille arm and invisible because both play over the
  photograph — and `distinctions` answered **23 920 kept at truecolor** for a `Palette::Roles` screen
  that collapses onto thirteen paints.

- **The media family ships, six of a video player's ten chrome parts with it, and drag capture is
  two fields** (components ticket 30, 2026-08-26; ADR 0038). `crates/vitui-components/src/media.rs` is the
  family — `picture`, `qr`, `barcode`, `waveform`, `spectrum`, `vu_meter`, all of them pure drawers
  with **no hit entry between them** — and `src/media/player.rs` is the chrome. **`MEMBERS` stays
  empty and that is the deliverable**: `INVENTORY` has no media row, so the module ships the family
  and the join says so from both sides.
  **Every figure the scene was pinned with reproduced through the component** — 24 000 writes /
  verbs / customs, 0 regions, 0 adjacent equal pairs, the ladder 1 / 2 / 2, 23 920 / 23 899 /
  15 347 / 0 distinctions, 219 of 441 modules wrong under an inverted pairing — which is what a
  stand-in written from the component's own construction is supposed to buy and is not usually
  checked.
  **The scan's needle could never have matched, and it is components 26's finding a second time.**
  `DECLARATIONS` read `pub fn picture(` and the shipped declaration is **`pub fn picture<P:
  Pixels>(`**: §1's `…` is a *type parameter* here and has to be, because a picture handed a buffer
  makes the frame cost the **image** and a sampler makes it cost the **rectangle**. A scene green on
  the parenthesis needle would have been green *by deleting the type parameter*.
  **The census is a contrast rather than four numbers, and the barcode's two is a subtraction**:
  picture one a cell, QR **4**, barcode **2**, the audio three **0**. A barcode carries no
  information across a cell's own *height*, so two of a QR's four cell states are unreachable —
  `COLOURS_PER_CELL` read from the other end, and the same fact that gives it **runs where a picture
  has none**. All three constructions whose colours are outside the theme share one `Theme::custom`
  calling line, so `chart`'s one-palette-in-one-place gate reads `[("chart.rs", 1), ("media.rs", 1)]`
  with its argument untouched.
  **Three constructions, and the count is a union rather than a product**: two glyph spellings and
  two palettes are four combinations and three constructions, because the description does not move
  with the repertoire — its paints are the theme's and its cells read one pixel each.
  **Drag capture works and it is `Response::local` over `Response::rect` and nothing else.** No press
  origin, no stored anchor, no *was I dragging last frame*; played through a **posted** pointer over
  the shipped chrome it is `20/299 = 0.0669`, `60/299 = 0.2007` and unchanged on release. **`slider`
  leaves Tier 3** and what is left for ticket 33 is the thumb, the keyboard, the step and the
  orientation. The cadence had to be written around and it is the usual two frames — the grab is
  awarded at `end` — so a gate playing one frame a phase would have measured the cadence and called
  it the mechanism.
  **The negative case fires on the mean and not on the total, measured in one run.** `chrome`'s
  marks are a field; `defective::chrome_collecting_into` is the shape they replaced and it allocates
  on the frames that draw the **chapter list**, which a short screen has no room for. Over 200 frames
  of which 40 are tall: shipped **0**, collected **40**, `allocs / n` **0**.
  **Three near-misses in the instruments.** A row about allocations drafted as *allocates* walked
  past a filter looking for *allocation* — and neither a wider needle (`alloc` also matches the row
  about the *allocator*) nor a `for` in place of the `any` (which fires on row 36, about the probe)
  is the answer, so the row was spelled to be caught and **the count is the assertion**. Every drawer
  here has a subject **smaller than its rectangle** and the easy arm is a `continue`, so all six
  fill — and a QR's remainder is the **quiet zone its own specification asks for**, the one place on
  this map where the partition rule and the subject's own standard are the same sentence. And
  `Stamp` saturated on the minutes alone, so 4 369 minutes read `99:00`.
  **A review then found a fourth, and it is `Ctx::id`'s documented trap with the half nobody
  states.** `picture_into` and `qr_into` carry `#[track_caller]` and took their id inside a plain
  private body one frame down — and the attribute propagates only through functions that carry it —
  so **every picture in a program had the same id**, whatever the call site. Both ways to notice it
  are quiet: a picture declares no region, so nothing merges and nothing is lost on the screen, and
  `Response::id` is a value a caller may key on rather than one the runtime checks. `barcode` was
  correct beside them for the only reason that matters, which made the diagnosis a comparison rather
  than a guess.
  **A review then found the same defect one file over, on the one construction that interacts.**
  `chrome` did not root its children inside its own id — ADR 0027's rule — so two chromes returned
  **one `Response::id`** and **6 merges**, and this one is *not* quiet: `Ctx::interact` makes a merged
  claim **inert**, so a second player's seek bar takes no press, no drag and no hover and its buttons
  never click on a screen that renders perfectly. It needs both halves — `#[track_caller]` decides
  whose id, `cx.with_id` decides what the keyed children hang from. Two more were the application's,
  and both are shapes this ticket had already written down: its QR band was capped at half the side
  column, so the extent clamp silently ate the bottom of the symbol at every ordinary terminal size
  (it is now what the symbol needs, with **got / wanted** on the status line), and it collected the
  barcode's pattern on the draw path, which is the exact shape the negative case is kept *for*.
  **What is not built is recorded and not attempted**: `media::WhyThereIsNoWallpaper` carries the
  three `LayerHost` gaps, the still screen's **24 000 cells re-damaged against 1 076** and the
  **13.7×**, and no component-level placement ships. Register 162 → **168 rows, 152 evaluated**;
  scenes **0 red, 26 stood up**.
  **The application is `theatre`**, and `--probe` is its own answer to a screen whose subject is a
  count: one headless frame at 100×30 prints **3 006 writes over 3 000 distinct** — every cell of the
  terminal, the six overwrites being the chapter marks and the thumb — 890 customs, and **7 regions:
  the panel, five transport buttons and the track, not one of them a picture, a symbol or a meter**.
  `4` is the key to press: every other component in the library degrades to a worse drawing of itself
  and a picture becomes a **description** of itself.

- **`file_preview_pane` and `file_picker` are the seventeenth and eighteenth components, the scene
  list has no red row left, and §15's *unclamped* is not a spelling of the offset at all**
  (components ticket 32, 2026-08-27; ADR 0039). `crates/vitui-components/src/files.rs` is the
  module; the three preview-pane screens draw through it, so scenes 23, 24 and 25 went green
  together — the last three, and they left the way they arrived, because they were pinned on one
  fact and it was the subject. **Every figure they were pinned with reproduced through the
  component**: 100 of 100 frames against 0 at 103 questions either way with `Worker::asked` 1
  against 2, 7 of 7 batches against 0 with a longest run of 1, 1 picture against 20 at 18 315 000
  B/s, the five offset spellings each producing their own defect, 14 857 142 bytes against a slot's
  4, and 20 torn frames of 20.
  **§15's *unclamped* is on the extent axis and not on the offset's, and it costs two things rather
  than one.** `scroll_area` clamps against the extent it is handed on **every** frame and the clamp
  is free, so an offset cannot be left unclamped; the only way to *the body draws nothing* is a pane
  that declares no bound — *a landing is a shrink*, and what shrinks is the extent. And an unbounded
  extent leaves the area **no tail**, so the cells the body cannot write are cells nobody writes:
  the shrink frame is **21 484 of 24 000** and the 2 516 nobody writes is the document's own. §15's
  own pair is 4 166 against 1 650 on a screen that did not partition — two different screens, and
  **the gap is 2 516 in both**, which is what makes `LINE_COLUMNS = 34` derived rather than chosen.
  **§15's 198 x 74 is the pane's *viewport* and not its rectangle**, and read the other way its own
  numbers stop agreeing: a positive case for bars-reserved costs one gutter an axis, so a 198 x 74
  rectangle hands its body 197 x 73 and neither `74 x 34 = 2 516` nor **14 652 customs** comes out.
  The rectangle is 199 x 75.
  **A landing is taken at the top of the view and an answer carries its own question** (ADR 0039),
  and both halves are one decision: `Task::take` is destructive and *a view has no `&mut` with which
  to put back what it took*. `PaneState::land` is a verb on the state rather than a step inside the
  component, so the shipped shape has exactly one consumer; and `Preview::shows` is §15's eight
  bytes, catching **an answer to a question nobody ever asked** — current generation, wrong payload
  — at 1 refused / 0 landings / **0 content cells**. *A question that was never asked is not out of
  order*, so no test on the answer's arrival can reach it.
  **R02's sweep is refused twice and R09's `Edit` once.** Ten tab switches cost **1 spawn, 1 decode
  and 1 fold against 10 of each**, because nothing here owns the job or the memo; and *a landing that
  did not happen is still a drop* — a guard taken outside the landing branch advances the revision
  every frame, so the highlighter re-folds **119 times against 20** on two screens that write the
  same 2 880 000 cells.
  **The needle problem, met a fourth time and answered by the ticket that owns the parameter list.**
  `pub fn file_preview_pane(` could never have matched `pub fn file_preview_pane<T, F>(`; components
  31 deliberately left the parenthesis rather than dictate this ticket's signature, so this one
  writes the needle and adds `PANE_OWES` — three fragments each deletable while leaving the
  declaration scan green and each a mechanism §15 assigns away from the widget. **G10 was already
  built**: `gates` row 31 is green and its own note already records that §21 is wrong about `Worker`,
  which is `Sync`; what §15 adds is G10b's `Cell`/`RefCell` sentence, and that is the new pair.
  **Three findings in the instruments, each the gate being wrong first.** A process-global decode
  counter read **100 286** units because tests run in parallel — the requirement is *0 on the app
  thread*, so the counter has to be one the app thread can read about itself. A highlighter with no
  document folds nothing, or the run's first frame is a real fold and §15's pair comes out 21 against
  120. And **a screen may not spell `Theme::custom`** — routed through `crate::media::picture_into`,
  one strip a row with each strip doing its own offsetting, it is §14's stated exception rather than
  a third palette, and the customs come out at **14 652**, derivable from 198 x 74 rather than
  measured.
  **`file_picker` is checked twice**: a source scan over its own body for the four mechanisms it may
  not mint, and a subtraction over what it declares — **6 regions as 1 + 1 + 1 + 3** — plus a shut
  face that partitions its row at every width from 1 to 40. Its `decode` and `line` are function
  pointers because a job must be `Send + 'static` and a body is `FnMut`. **The shut face is one
  drawing now**: the picker's was `select`'s transcribed, and that drawing is where components 26's
  one-cell double write lived, so it is `glyphs::elided_row_into` and both components spend their
  own prefix before calling it — two copies of a drawing that has already been wrong once is one
  copy too many.
  Register 175 → **181 rows, 165 evaluated**; scenes **0 red, 29 stood up**.
  **The application is `browse`**, and `k` then `s` is the one to watch: nothing moves, and the pane
  goes on showing the file that used to be at that position for ever. Its `--probe` prints one
  headless frame at 120x30 — 4 800 writes over 4 800 distinct, 4 regions, 1 spawn, 1 landing, 0
  refused.

- **The preview pane's three screens run and all three are red, and §15's own wire sentence is two
  readings of one number** (components ticket 31, 2026-08-26). `crates/vitui-components/src/preview.rs`
  is the screen; `src/files.rs` declares neither `file_preview_pane` nor `file_picker`, so scenes 23,
  24 and 25 go `Unsubjected` → **`Red`** with their figures as the failing set and components 32 as
  the inverter. It is scene 22's move one ticket earlier, and **the list had no red row in between**,
  which is the stretch `scenes::line` was kept alive for.
  **The memo-key rule arrives through the one door that is not a gesture.** A preview pane's question
  is a *file* and every natural way to name it names a *position*: one re-sort — one line of
  application code, no keystroke — and the position-keyed pane is wrong on **100 of 100 frames**
  against 0, with `Task::request` called **103 times either way** and `Worker::asked` the only
  counter that moves, **1 against 2**, *because the question still matches, so nothing posts, so
  nothing wakes, so no frame corrects it*. The **197 of 200** is a property of the data rather than a
  stated permutation — the movable positions rotated by one, with `File::size` each file's rank in
  that order — so the re-sort really is a sort. `frames` and `requests` are two counters incremented
  in two places, because one counter read twice is a spelling.
  **Three of §15's sentences are read rather than reproduced, and each reading is the finding.** Its
  *identity keying is wrong for exactly 1 frame* is a **run** and not a total — **1 against 21** over
  7 wrong frames against 21 — because a total of one would need six of the seven batches not to move
  the cursor's file. Its *1 650 writes against 4 166* is a screen that **did not partition its
  rectangle**: a component writes every cell of its own, so this one writes **24 000 either way** and
  both `writes` and `distinct` are blind to the offset defect while `content_writes` is not — and
  **4 166 − 1 650 = 2 516 = 74 × 34**, which is where the document's width is derived from rather
  than chosen. And its three wire figures are **two readings of one number**: 14 652 cells at 37.5
  B/cell is 549 450 bytes = **536.57 KiB**, which §15 prints as *536 KB* by truncating; twenty of
  them is 10 989 000 bytes = **10.99 MB**, which is exactly what **18.3 MB/s over 600 ms** requires —
  and the *10.7 MB* printed beside that rate is the truncated 536 read as decimal kB. The two that
  agree are gated and the third is recorded.
  **The five offset spellings each produce their own column**, which is why §15's row is a table: 0
  content cells against 2 516, row **726**, the *previous* file scrolled to its top while it is still
  on screen, right on arrival and 0 on return, and right in both directions. The map is refused at
  **14 857 142 bytes against a slot's 4** — §15's own number from the arithmetic that produces it —
  with the shipped `HashMap` at **35 651 584** and **the relation gated rather than that figure**,
  because a count of the standard library's own allocation is a gate on somebody else's
  implementation.
  **The tear needs no thread at all.** `Task::take` is destructive, so a view that asks for the
  answer where it happens to want it takes it in the *first* consumer and leaves the second — three
  quarters of a screen further down — looking at the value that was there before; a view has no
  `&mut` to put back what it took. **20 torn frames of 20 against 0**, read off the drawn surface,
  and the state ends up correct either way, which is why a torn frame leaves nothing behind.
  **The needle problem, met a third time and answered differently.** A longer needle would be this
  ticket dictating ticket 32's parameter list, so the existence scan stays `pub fn file_preview_pane(`
  and the load-bearing half is a **negative** scan: §15 refuses R02's sweep on a job's lifetime —
  *neither is the widget's* — so `preview::mints_its_own_task` looks for `Task::new(` inside
  `files.rs`, which cannot be satisfied by deleting anything and is watched over two sources rather
  than over one empty file. **`covers` is empty on all three deliberately**: a scene waiting for its
  subject is not yet evidence of anything. Register 168 → **175 rows, 158 evaluated**; scenes **3
  red, 26 stood up**.

- **`collapsible` is the thirteenth component, and three of §8's figures are replaced by findings
  rather than reproduced** (components ticket 22, 2026-08-26; ADR 0035). It is **one machine and
  three configurations** — `disclose::SPLIT` is §8's three-row table as a value and
  `disclose::Collapses` is the join, three rows and three arms with no fourth on either side — and
  `crate::accordion` now draws *through* it, so **scenes 10 and 11 went green together**, the same
  shape 14→15, 16→17 and 18→19 had, because they were pinned on one fact and it was the subject.
  Every ticket-21 figure reproduces through the component: 13 hit entries closed against 421 at
  `h = 0`, **408 on the hit index and the ring at once**, two surfaces 0 cells over 0 rows apart, the
  mid-transition 26, 4 166 of 4 167 folds against 0.
  **§8's byte pair cannot both be a `size_of` of one type**, and that is the half worth keeping: a
  `Collapse` is **4** bytes live and **48** with the slot, because an `Option<Tween<u16>>` field costs
  its forty bytes **empty** — a record that is 5 B without a tween is a record whose tween lives
  somewhere else, and nothing here has anywhere to put one. Five is what a third `u16` would cost and
  there is no third: `Tween::to` is the target while a tween runs and the height is it afterwards.
  *Never per row of content* does hold — 4 167 folds cost 16 668 bytes of line numbers, twelve
  sections 576.
  **The watermark latches rather than settling, and that is ADR 0029's second sentence on the height
  axis.** §8 prices the drawn extent at *83 cells, 8 rows wrong, 3 frames* against *22, 0, 2*; those
  are a prototype's **body**. What reproduces is *a measured extent is taken inside the rectangle the
  decision produced* — over a body that fills what it is handed the height sticks at **19 rows where
  4 are right, permanently**, and over one that draws only its content the two arms are
  **indistinguishable**, which is what makes the rule unconditional rather than a default. The +11.5%
  is read as a count: the dry run goes through the caller's ink, so the watermark arm makes the body's
  verbs twice.
  **§8's *0 ring probes against 405* is unreachable from any self-close gesture**, and the three
  reasons are three mechanisms: a click on a focusable header is *awarded* the focus; a click on one
  that is **not** a tab stop **defocuses**, because a press landing on nothing interested is read as
  intent and there is then no id to have vanished; and `Enter` needs the header to hold the focus
  already. The arm that pays is a collapse nobody clicked for — **0 probes against 15** over a
  collapse-all with and without the caller capturing `Frame::focus` — and what focusing the header
  buys is **the keyboard**, not the probe count. Beside it, *14 frames to quiet* is a cadence wearing
  a count's clothes (12 at sixty hertz, 24 at a hundred and twenty), so the gate is the relation and
  `Driver::pin_clock` is what makes it one.
  **§8's zero allocations reproduce, and the warming discipline is the finding underneath**: every
  other allocation window in the workspace warms with two identical frames, and a transition hands
  the body a **different rectangle** every frame — so warmed that way it reads **1 over 12**, which is
  amortised zero and the shape `Allocations` refuses to average away; warmed on the *shape* it is
  **0**. Register row 32's fifth instrument, and the first non-steady frame that row has been asked
  about.
  **There is no third state and the scan that keeps it out reported the module it defends** — a
  scanner looking for a literal contains that literal, which is `crate::frame`'s own first run — so
  the two needles are assembled from fragments. The surface equality is a **new** register row and not
  row 21, which is row 41's standing to row 2's: a `Pen` carried *across* frames sees residue in the
  cells this crate wrote, and the arms are compared with the focus seated and no pointer anywhere,
  because a click leaves them **one cell of hover paint** apart. Criterion 14 is that instrument with
  one field flipped: a caller writing its tail from the height it had *before* the collapse leaves
  **240 cells over 4 rows** of the old body under a correct header.
  Register 129 → **135 rows, 118 evaluated** (rows 24 and 25 inverted — the last two `Unsubjected`
  rows of §21's own table); scenes **4 red, 21 stood up**. §22's fog is left alone by construction:
  `settings` owns its own offset rather than putting a collapsible inside a `scroll_area`, because
  pressing `w` there would answer *may it learn its content height one frame late* by accident, in an
  application.

- **Every application in the workspace read the unhandled key window one frame too early, and
  `reader`'s `q` did not work at all** (found by components ticket 22's application, 2026-08-26; map
  decision C25). `Driver::unhandled` is *a window onto the same queue, valid until the next frame
  begins*, and all five loops that read the keyboard read it **before** their own frame — so an
  application acted on the previous frame's window, one wake late, which for a single keystroke means
  **never**. Measured on the shipped binaries: `reader` hung past two minutes on `q` with stdin held
  open, and `explorer` quit at **1 309 ms** rather than 298 because a `collection`'s type-ahead
  deadline happened to supply the second wake. Five loops fixed, and a **source scan** keeps them
  fixed — watched rejecting the order it forbids, because a gate over the behaviour would need a pty.
  **Six defects were caught between the first green run and the commit, five of them in the
  application**, all at coordinates the gates never use: three instances of one mistake — the child's
  coordinates are its own, so a cull bound and a rectangle origin in the interior's *root* space culled
  the first three sections of a three-row-inset panel and indented the rest; `heads` indexed by draw
  order against an `opened` indexed by section, so exclusive mode focused the wrong header — the one
  thing that pair exists to show; `content` counted twice a drawn section; `status` zeroing the one
  counter it prints. **The sixth is the second half of the finding itself**: `reader`'s reveal frame
  sat *between* the draw and the read, and a frame re-`begin`s the key queue, so `[End, q]` in one
  batch lost the `q` — the scan compared two positions and could not see it, and it **counts** now.
  Beside it: **a printable character cannot be an application's quit key while a `collection` holds
  the focus**, since a focused collection consumes every text-bearing key into its type-ahead buffer
  (§5). `ledger` and `explorer` bind `Ctrl+Q` beside `q` — `ledger`'s own established arrangement for
  a binding a terminal or a component can take away — and `triage`'s check already ignored the
  modifiers, so its own comment's *would have eaten it* is *does*. All five quit in under 300 ms on
  `Ctrl+Q`.

- **The wheel gate is green, and inverting it found the pair spec §21 had no way to state**
  (components ticket 20, 2026-08-25). It was the sharpest of the four pinned reds and the only one
  whose failing set was the **defect** rather than a missing subject, which is why it stayed red for
  nine tickets. **Criterion 1 was already met** — the shipped `collection` has asked for a reveal only
  from `Handled::reveal` since ticket 12, and every `request_into_view` in the workspace is behind a
  condition. What kept the row red was that **the gate could not be written over the code**: two
  stand-ins, each honest when written. The click was a *delta* handed to the arithmetic
  `Response::scrolled` would have delivered it to, because *a `Mouse` needs a `Buttons` and a
  `MouseKind` and neither was in `ENGINE_NAMES` at all* (runtime issue 22 lifted it); the subject was
  a row loop written beside the gate, because `collection` did not exist yet.
  `crates/vitui-components/src/wheel.rs` is that instrument with both removed — a posted `Notch`
  routed through the previous frame's hit index, played over `collection_into` and `scroll_area`
  against `collect::defective::every_frame` and `never_reveals`.
  **Neither stand-in was on the side of either arm, so the pinned figure did not move** — `0 against
  20` either way. **What they had cost was the second subject and the second axis**: §21 carries the
  wheel as one row over one component, which is exactly what is writable while a click is a delta
  added to an offset, because *a delta added to an offset has no second axis to be wrong on*. With a
  real notch, **a body dead downward is alive sideways** — asking for content row 0 every frame
  settles a vertical click at **0** and a horizontal one at **20**, and the transpose does the
  opposite, because `Area::into_view` answers per axis and returns 0 for an axis the rectangle already
  sits inside. **A gate reading one number calls the first of those healthy.** `scroll_area` had
  declared `owns_offset` since ticket 01 with no scene claiming the axis; **scene 33** is that scene
  and O5 goes 18 → **17** unmet of 34.
  **The rule's second clause is asserted for the first time** — *a press already proves the widget was
  on screen* — and the unconditional arm **cannot be watched failing it**: by the frame the press edge
  lands on it has already dragged the viewport back, so `Area::into_view` answers `(0, 0)` and the
  loudest arm is silent for the quietest reason. It is watched asking on the frame *before* any
  gesture instead. Register 128 → **129 rows, 110 evaluated**; scenes 32 → **33**, **6 red, 19 stood
  up**. Three runtime cadences the instrument had to be written around, each found by the gate being
  wrong first: the focus is seated from `Response::id` a frame before the key it enables, a press is
  two frames because the grab is awarded at `end`, and **`WhenAsked` for an area is state across
  frames** — a *once* flag inside the body closure is `false` again next frame, so the correct arm
  asked every frame and measured the defect under the correct arm's name.
  **A review then found the gate blind in the one place the three-arm design exists for**, and it is
  the ticket's own subject arriving in the ticket's own instrument. The chord was posted behind
  `if reveal != Reveal::Never`, so `revealed(Collection, Never, …) == (0, 0)` — the whole evidence
  that deleting the call loses the keyboard — was produced by the **absent gesture** and not by the
  absent reveal. Measured: with the guard in, repairing `never_reveals` to behave exactly like the
  shipped component left all five tests green.
  **And running every report found three that had been panicking, one for eight tickets.**
  `listing_numbers` asserted `red == 5` and ticket 12 turned four of those green without touching it;
  `keys_numbers` asserted `REACHABLE_STATES == 8` where runtime issue 22 made it 256; `scene_numbers`
  carried two standing counts from before ticket 12. **`cargo test` does not run an example** — it is
  compiled by `cargo clippy --all-targets` and evaluated by nothing, which is runtime ticket 20's
  *a number measured in a file nothing evaluates* one crate over and the reason `examples/frame.rs`
  needed a CI line. All twenty-one `*_numbers.rs` ran to completion then, and **none of them is a
  gate** — the convention that no register row rests on one is untouched. **It rotted again by
  components 32 and a review during 36 found it**: `scene_numbers` had been asserting `covered == 17`
  since this ticket while the answer was 20. A sweep of all twenty-six is the check, and it is not a
  gate either.

- **`scroll_area`, `scrollbar` and `sticky` are the tenth, eleventh and twelfth components, and the
  ticket found that both of this crate's recorders were in the wrong coordinate system** (components
  ticket 19, 2026-08-25). **Bars are reserved and there is no overlay option** (ADR 0029), checked as
  arithmetic rather than as a measurement: the parts of a reserved area — the body's viewport, the
  two bars, the corner and the four bands — **tile the rectangle exactly** at every size from 1×1 to
  23×23, under both `Hide` spellings and three extents, and an overlay bar cannot satisfy that
  equality at all because the cells under it belong to two parts. `sticky` is §9's **one** band
  construction with `Which` as its four configurations — header and footer share `x`, the pinned
  column shares `y`, the gutter shares neither — and **none of them declares anything**, so a frame
  with four bands standing declares the same **3** regions as the same frame with none. `Hide::
  WhenItFits` takes a *declared* extent because there is nowhere to put a closure that could measure
  one, kept unwritable by a `compile_fail` pair rather than by a paragraph.
  **The finding is one crate down and it is one components ticket 15 had already written down as a
  constraint.** `Tally::distinct` and `Pen` recorded a verb where it was *called*: a scroll area with
  a sticky header reported **299 double writes on a frame that has none**, two areas sixty columns
  apart recorded their bands as one (**119 re-damaged against 0**), and a `tree` narrowed to `x == 5`
  recorded its first cell at column 0 under a test that checked the wrong edge and passed for it. §6
  says *a translated band makes `distinct` meaningless* — **as a rule about what a component may
  do**, and it is a defect in the recorder. `Ctx::origin` is runtime architecture issue 32, additive,
  the same arithmetic `hover_style`, `overlay`'s anchor and `caret` already did privately. **Nothing
  drawn at the root moved**, so three tests changed and each was inverted rather than deleted — and
  **one of the three was an argument**: `crate::frame`'s first ground for refusing a closure-taking
  `block`, *124 double writes on a panel that has none*, is **struck**. The decision stands on
  `CONTEXT.md`'s identity rule, which is not a measurement and cannot be repaired by one.
  **Three of §9's five figures are timings and none reproduces.** The frame is **438–500 µs at
  24 000 writes / 163 verbs / 3 regions / 0 allocations** against 48.83 / 33 890 / 1 453 / 59: this
  screen writes **every cell of 300×80**, so `writes == distinct == 24 000` is a partition of the
  whole terminal and the budget is the 1 ms class — where `tree` (436) and `table` (545) are. **The
  regions are the finding inside that row**: 59 is a screen with per-row hit entries, and a
  `scroll_area`'s body is the caller's and declares none. The shape change reproduces within 2.5% on
  the arm with a prefix sum to rebuild (403.8 against 394.2) and is four times cheaper on the arm
  without one (14.7 against 58.8). **1k → 1M is identical writes and identical verbs**, which is the
  row that matters. Register 121 → **128 rows, 108 evaluated**; scenes **7 red, 17 stood up**, all
  four of ticket 18's going green together because they were pinned on one fact and it was the
  subject.
  **Four defects were caught between the first green run and the commit**, all at coordinates the
  gates never use: the two bars were keyed at `Id::keyed(id, 0)` and `(id, 1)`, which is what a body
  keying per row writes for its first two rows — so an area **merged two widgets into one id**, and
  the gate is watched catching it at 1 merge against 0; a horizontal `scrollbar` taller than one row
  left both stepper columns unwritten below their first cell, which `scroll_area` cannot produce and
  a caller can ask for; and the application both blanked its own body at any horizontal offset past a
  screenful (padding from `Written::cells`, which inside a scrolled scope is *what survived the
  clip*) and rebuilt the panel's interior by hand as the screen inset by two where `frame::draw`
  insets by three under `Cosy`.
  **The application is `reader`** — a build log of 120 000 entries, one in eight three rows tall —
  and `u` is §9's unit rule on a screen: measured in rows, `End` reaches **entry 95 999 of 119 999**
  and calls it the end, with the thumb a plausible size in a plausible place and every counter the
  same at both ends of the content. Its own finding is runtime issue **33**: `Ctx::request_into_view`
  never asks for the frame that applies it, so a reveal lands **one keystroke late** in any loop that
  parks on `Driver::wait` — `collection`, `table`, `tree` and `accordion` all carry it, and **a gate
  drives its own frames**, so the missing wake is invisible by construction.

- **`tree` is the ninth component, and it found that §7's record had been the wrong width since
  components 13** (components ticket 17, 2026-08-25). It is `collection` plus a flatten index and the
  sentence is *checkable*: `tree_with` calls `collection_chorded`, and
  `size_of::<TreeState>() == size_of::<CollState>() + size_of::<Asked>()` — a tree **is** a
  collection and one slot. The `+` is two verbs a row, the indent run and the chevron cell, measured
  as `tree - list == 2 × rows` exactly.
  **The record.** §7 states `Row { node: u32, depth: u16, flags: u8, h: u8 }` **with the widths**;
  §10 — where *one structure, five names* lives — states the four field *names* and no widths at
  all. Components 13 built the type from §10 and widened three of them, and nothing caught it
  because §7's own criterion is *a gate asserts its size* and that criterion is ticket 17's: **a
  number only one ticket is instructed to assert is unasserted until that ticket runs.** Narrowed,
  and it is what makes §7's memory table reproduce — **7.63 → 11.44 MiB at a million rows** against
  15.26 → 19.07, with the growth `4 / 8` exactly (50%, where §7's rounded pair divides to 57). The
  alternative was `tree` minting a second record, which is the thing ADR 0031 exists to refuse.
  Three ceilings are documented rather than discovered: a key at `u32::MAX`, eight flag bits, a row
  255 rows tall. Components architecture issue 21, map decision C21.
  **`←` and `→` cannot be read before or after `collection` — only inside it.** `Ctx::decline` hands
  a key back **and ends the level's turn at the queue**, so a container draining first leaves
  `collection` nothing and one draining after finds the queue closed. And `nav::step` reads `←`/`→`
  as `↑`/`↓`, which are exactly the fold keys. The hook is `collect::Refusal`, a parameter of the
  one drain loop — named `Refusal` because `vitui_runtime::Chord` already owns the other word.
  **The scenes went green together**, because they were pinned on one fact, and every ticket-16
  figure reproduces *through the shipped component*: 24 000 cells, 81 regions, 1 stop, 2/4/24 verbs a
  row, 9 599 840 asked against 24 000 with every other counter preferring the defect, and the tally's
  65 535-a-verb saturation. §7's *115 µs against 1 187* comes back as **116 against 337**, its
  157–342 µs splice as **152–240 flat across five orders of magnitude of removal**, its 2.25× prefix
  sum as **2.85×**, and its 40 393 µs rebuild as **~990** — the last because §7's walked a shuffled
  forest at 85 ns a row and this one reads a contiguous depth array, and it is *still* the losing arm
  at every size a tree has. Register 113 → **121 rows, 101 evaluated**; scenes **11 red, 13 stood
  up**.
  **The frame is 436 µs and the component's own share is inside the noise**: the same rectangle
  drawn as a plain list through a hand-written row loop is 435.60 and through `tree` is 436.29 —
  0.16% — while a twelve-column table over the same 24 000 cells is 426.74. Twelve times the verbs
  is the same frame, which is §6's *verbs are a currency for structure and not for time* from the
  other side. 24 000 cells is the full-screen class, so it is inside the 1 ms budget.
  **Two things had to be settled to keep *only the ask moves* true** once a component owned the row
  loop: the row drawer is called with an **empty** rectangle when the indent has eaten the row rather
  than not called (not calling it makes *rows iterated* a second counter that separates the arms),
  and `indent_columns` is the single decision both the component and the screen read, pinned to the
  component by `label.x == indent + 1`.
  **A review of the first commit found the application's keyboard dead**, and the cause is this
  ticket's own finding one level up: `explorer` read its five keys with a second `cx.next_key(id)`
  loop after the component returned, and `collection` had already declined the first key it did not
  own — which closes the level's queue. **A container gets its keys inside `collection`'s one drain
  loop and an application gets them after the frame, from `driver.unhandled()`; there is no third
  place.** Beside it: a toggle that swaps *which function* draws a widget swaps the widget, because
  `Ctx::id` mints from `Location::caller()` — so `d` re-seats the focus, and `v` does not have to,
  because `impl<T: Ink + ?Sized> Ink for &mut T` makes the ink erasable and the call site one.
  **The application is `explorer`**, 258 313 nodes over a caller-owned index, and its finding is the
  glyph column: `INVENTORY` declares `VLine`, `TeeLeft` and `BottomLeft` for `tree` and the component
  draws none of them, because a guide column at depth *d* is a fact about *d* **ancestors** and the
  flatten index does not hold that. Filed as components architecture issue 20 with the three
  available answers rather than faked with a `VLine` repeat.
- **`table` is the eighth component and the second one that reached down into another crate's
  defect** (components ticket 15, 2026-08-25). It is `collection` plus a column rect split and the
  sentence is *checkable*: `table_with` calls `collection_into`, and *no second selection store, no
  second scan cursor, no second `Mode`* is read out of the file. **Three of §6's remembered figures
  did not reproduce.** The headline's and identity's magnitudes are the prototype screen's, which
  ticket 14 had already established — the structure is exact (24 000 cells either way, **160 verbs
  against 1 600**) and **640 of 800 targets are inert** on the row-keyed arm, §4's arithmetic over a
  different population. And *a pass per band measures 4% cheaper* is **inside ±1.5% with its sign
  flipping between runs of the same binary** — which only became visible after the instrument was
  rewritten from `A × 40, B × 40` to sixty interleaved rounds and a minimum. The refusal stands
  untouched, and that is the point rather than a consolation: §6 refuses that spelling on an
  **argument** — three loops cannot share §5's lockstep scan cursor — and *a spelling refused on an
  argument does not become acceptable when the clock stops agreeing with it*.
  **The finding is a defect one crate down.** `Ctx::with_key` inside a scroll scope draws **0 cells
  of 8 at an offset of 100 and 8 of 8 at zero**: `Ctx::with_id` re-childs the view at `self.area()`,
  which is `Rect::new(0, 0, w, h)` in the *current* coordinate system, and inside a scroll scope that
  origin is the **content's**. `collection` had worked around it without naming it — its `with_id` is
  outside the scope — and `table` cannot take the same workaround, because a cell's key varies with
  the row. Nor can it translate the band to make `with_key` work, because both recorders union in the
  coordinates of the `Ctx` the verb was called on and a translated band makes `distinct` meaningless.
  So a cell's id is minted with `Id::keyed` and handed down on `collect::Cell::id`, which is spec
  §4's *every workaround that looks like a hack is the id being opaque* one axis over. Filed as
  `.scratch/vitui-runtime-architecture/issues/31` and pinned as **components register row 112**, the
  second row on that register whose subject is another crate. Register 108 → **112 rows, 91
  evaluated**; scenes 7 and 31 green with row 78; the frame is **545 µs against a 544 µs stand-in**,
  so drawing through `collection` costs ~2%, and 166 µs over the budget while damaging nothing is
  recorded rather than optimised against.
  **Its application found a second defect, in code that was already green.**
  `crates/vitui-apps/examples/ledger.rs` is the first thing to put a table inside anything, and
  `header_row` drew its bands from `x = 0` rather than from `head.x` — so a table handed a panel's
  interior put its header one column into the border, and **every gate in the crate passed**,
  because every one of them plays at `x == 0` where the header's coordinates and the body's agree.
  They do not come from the same place: the body draws inside `Ctx::scroll_scope`, which childs at
  the body's rectangle. Register row 113, gated on the reported double write at a non-zero origin —
  276 of 2 304 right against 288 wrong. The app then made the mirror-image mistake in itself, a cell
  drawer writing with `cx.text` instead of through the ink it was handed: **78 cells and 10 verbs on
  a frame that wrote 1 560**, on a screen that looked correct.
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
- **Ticket 06 is the first component-facing code in the workspace**, and it had to work **around**
  runtime architecture issue 22 to write a signature at all — issue 22 has since been settled the
  other way, so read the paragraph below as the record of why `Cells` existed rather than as the
  current state — components architecture issue 17 deleted it, and the helpers return
  `vitui_runtime::Rect`. Spec §3 states both helpers as sentences about
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
  `Ctx::child` is there so the **caller** can narrow; what a container may not do is narrow on the
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
- **The engine's vocabulary crossed the crate line** (runtime architecture issue 22, 2026-08-24).
  The component-facing surface named **seventeen** engine types no crate above the runtime could
  spell — `vitui_engine::Rect` in twenty-seven public declarations — and the fix is a rule rather
  than a list, gated in both directions at `crate::line`: *every engine type this crate's public
  surface names is reachable through this crate, and so is every type needed to **construct** one
  that the surface accepts.* **The second clause is the finding, and `listing.rs` wrote it before
  the rule existed**: a `Mouse` needs a `Buttons` and a `MouseKind`, and *neither of those is in
  `ENGINE_NAMES` at all* — so a barrier against `Mouse` survived every check that looked at `Mouse`.
  *A name a consumer can write but not build is a barrier wearing a re-export's clothes.*
  `ENGINE_NAMES` is 25 → **29 rows, every one with a path**; twenty sit at the crate root because no
  runtime module owns them, `Wheel` arrives as `Notch` because `scroll::Wheel` is a *configuration*,
  and `Restyle`/`Style` are re-exported despite not being on the surface — striking a row to make a
  gate come out even is what `route` is the precedent against. **Amending C6 was refused** and
  `deny.toml` is unchanged: a re-export keeps *the runtime is replaceable on the same engine* a
  claim about one crate. Three defects in the instrument itself: `Capabilities` was **already**
  reachable as `ctx::Caps` and the scan could not see a `pub type`; the scan read lines and a
  `pub use` is a *statement*, so its first run reported *the engine is unreachable* about a crate
  re-exporting all of it; and §4's *three names are re-exported* sentence was **a third true**.
  **The near-miss is the part to remember**: three `Instrument::Barrier` citations point at
  `name: "Rgb",`-style lines that **still exist** — only the line beneath changed — so all three
  would have gone on passing while meaning the opposite. What caught it was a *second* gate on the
  same fact, asserting the line beneath the name, whose failure message named its own procedure.
  Downstream, `counters::sentinel` loses the first of three barriers and `keys::REACHABLE_STATES`
  goes **8 → 256**; **no row went green**, because none of the defects they name was a re-export.
  `Cells` was left a module with no stated reason, filed as components architecture issue 17 rather
  than deleted from a runtime session — and resolved the next day.
- **An application runs.** `crates/vitui-apps` is new (2026-08-24, at the user's request, on no map):
  the applications as `examples/`, one file each, the first a port of ratatui's counter-app tutorial.
  It depends on `vitui-runtime` and `vitui-components` and **not** on the `vitui` facade, which is
  what makes every file in it a standing proof that the component-facing surface is sufficient — the
  facade re-exports the engine, so depending on it would make `Rect` nameable and evaporate the claim
  without a line changing. A test reads the source rather than the manifest for the same reason.
- **`Cells` is deleted, and the component surface speaks the runtime's `Rect`** (components
  architecture issue 17, 2026-08-25). For eleven tickets this crate named its own rectangle because
  spec §3's *returns the rectangle it did not write* **was not writable as Rust here**:
  `vitui_engine::Rect` was unnameable across the crate line. Runtime issue 22 fixed the cause, so the
  stand-in goes — 23.6 KB, the algebra duplicating `layout::rect` operator for operator, and the
  corpus sweep that existed **only** to hold the duplicate honest. ~250 sites across 21 files; **C6
  untouched**, the dependency table is still one crate. The distinction was *checked rather than
  assumed*: `Ctx::area` is `Rect::new(0, 0, w, h)`, so the coordinate systems were identical, and the
  one real difference — `Cells`'s `u16` origin against `Rect`'s `i32` — argues the same way, because
  the engine's verbs take i32 and the code already wrote `i32::from(band.y())` before every draw. The
  migration **removed** conversions. **One structural win**: the hover award used to reach the runtime
  through *two* hops, the first existing only because a component could not name a `Rect` to pass;
  `ink.rs` is now the only file that spells it, and the gate that asserted two asserts one — the
  single failure out of 181, failing for the right reason. **One defect introduced and caught before
  it shipped**: `Cells::hover_style` guarded on `is_empty` where `Ctx::hover_style` pushes
  unconditionally, so a mechanical rewrite dropped the guard and an empty rectangle would have become
  an entry that paints nothing and still counts. Restored at `Ink::award`, named in a comment.
- **Nothing holds the focus until an application says so** (architecture issue 25, 2026-08-25).
  The first application drew correctly, parked at 0.00/0.00 and **ignored the keyboard**:
  `Frame::focused` starts `None` and nothing sets it — not `interact`, not `Interest::FOCUS`, not the
  ring — while `next_key` answers nobody but the focused id. **0 against 5** over five `Right`
  presses. The click is an *accidental repair*, which is why it presents as *the terminal lost focus*.
  Settled additively: **`Ctx::focused() -> Option<Id>`**, so `if cx.focused().is_none() { cx.focus(sink) }`
  is writable inside the draw — `Ctx` could ask `is_focused(id)` and had no way to ask whether
  *anything* held the focus, so the obligation had needed a flag outside the frame. **A runtime that
  seats the first stop was refused**: it is an opinion about which widget is primary on a runtime with
  no scene tree, *first* means **draw order**, and it collides with the vanish rule — a closing modal
  would hand the keyboard to whatever draws first. A `Driver` flag only relocates the argument. The
  gate is that the two seating forms are **different programs**: both seat correctly on the first
  frame, so a one-widget program cannot tell them apart, and after the user tabs away
  `!is_focused(sink)` drags the keyboard back — a `Tab` that appears to do nothing. Register 40 → 41,
  spec §8 gained the section it never had, and `counter.rs` lost its `focused: bool`.
- **Writing it found that no loop could be written at all** — runtime architecture issue 23, now
  resolved. §1's frame sequence opens with `wait()`; `Screen::wait` is the app thread's only
  blocking call; `Driver` owned its `Screen` privately and `attach` bound the `WakeHandle` as `_wake`
  and dropped it. So the only shape available was `while !exit { driver.frame(…) }`, a spin at 100%
  of a core that **looks correct** because `present` coalesces. Worse: `Worker::hire(WakeHandle)` was
  **public and uninvokable**, so the whole of spec §17 — resident thread, one-slot inbox, the eight
  bytes of generation — was reachable from a test that builds its own `Engine` and from nowhere else.
  `Driver::wait` and `Driver::wake` forward both, unchanged and with no policy on this side; **§21's
  question of who owns the loop is untouched**, and what is settled is that the first of its three
  homes is possible rather than described. `line.rs`: 24 engine names → 25, reachable 7 → 9,
  unreachable 17 → 16. `register.rs`: 39 → 40, and entry 40 is the first row whose source is an
  *architecture issue* rather than an `R NN`, because the implementation backlog was already closed —
  the destination gate was widened to accept `issue NN` rather than given a citation to a file that
  does not exist. The measurement that matters is on the shipped binary, parked five seconds:
  **0.00 user / 0.00 sys**, which is `scripts/idle-gate.sh`'s budget met by an application rather
  than by the engine under test.

- **`slider` is the nineteenth component, and the ticket found that the `built` column had claimed it
  for two tickets with nothing able to see that** (components ticket 33, 2026-08-27; ADR 0040).
  `INVENTORY`'s row read `built: true` and `input::MEMBERS` listed the name from ticket 30 onwards, on
  the strength of §14's *`slider` leaves Tier 3* — and **no `slider` existed anywhere in the crate**.
  Neither join that looks as though it should have caught it could: the module-tree join compares a
  module's `MEMBERS` against the `families` **column** and never against the source, so a name in both
  places agrees with itself, and the tier/built gate accepts any row appearing in `MOVED` — which for
  `slider` recorded something true, because the *mechanism* had been built. **A mechanism being built
  is not the component being built.** Register row 187 is the join, 19 of 19 with the reverse direction
  counted too, and **the needle problem arrives a fifth time and is answered as a rule**: a join over
  twenty-nine rows cannot dictate nineteen signatures, so the needle is the *name* and the boundary is
  **either delimiter** — `pub fn <id>(` or `pub fn <id><`.
  **Everything §14 measured reproduces through the component.** `grab` is `Response::local` over
  `Response::rect` and nothing else, swept against `media::player::scrub` rather than transcribed;
  played from a posted pointer it is **20/299**, **60/299** and unchanged on release. The vertical arm
  is **inverted and not transposed** — up is more — and the transpose agrees at one row of eleven.
  **The step is an integer index and the `f32` step is wrong twice with each half hiding the other**
  (ADR 0040). On a 300-cell track from zero: fifty `Right`s land on **0.5 / cell 150** against
  **0.4999998 / cell 149**, and a hundred on **1.0** against **0.99999934 / cell 299** — the *right*
  cell, and a value that never reaches its own maximum, so `if v == 1.0` is a branch a keyboard-driven
  slider can never take and nothing on the screen says so. A drag is deliberately **not** quantised,
  because snapping it would make §14's own two fractions unreachable: the grid is the keyboard's unit
  and not the value's.
  **There is no range slider, and the refusal is a measurement.** Two thumbs need *which* thumb, which
  is the fifth cross-frame fact §14's headline is that drag capture does not need; the spelling that
  stores nothing derives the boundary from the two values it separates, so a pointer crossing it
  mid-gesture **abandons the thumb it was dragging and yanks the other one** — `0.8 → 0.6` on a thumb
  nobody touched, in a drag that never released. Two thumbs as two *widgets* is not a third answer: a
  one-cell thumb's own `local` says nothing about the track.
  **`scroll::stripe` is reached and `scroll::bar` is refused on two counts** — a bar has two parts and
  a slider has three, so `BarOpts`'s one `track` role cannot express it, and a `Span` is content cells
  which a slider has none of — so the *answer* is shared instead, swept against `scroll::thumb` over
  every track length from 1 to 200. **`nav::step` is not reached and the disagreement is 4 of 8 cursor
  codes**: it pairs `Up` with `Left` because a list's index grows downward, and a slider's value grows
  upward. Components 17's finding from the other side.
  **Three findings in the instruments, each the gate being wrong first.** The steady figure is a
  **sequence** and not a total — `[40, 1, 0, …]`, because `Response::hovered` is resolved from the
  previous frame's index, so read as one total over 59 frames it is `1` and looks like a defect. The
  allocation window read **1 over 60 frames** until it warmed the path it prices, the run driving the
  keyboard and the warm-up not. And **a test calling the component from two call sites is two
  widgets**, with the *focus* as the instrument that notices: the press focuses the first id, the next
  frame draws the second, and the vanish rule clears it — failing three mechanisms from its cause.
  **A self-review then found a fourth, in the shipped code**: `Response::changed` was compared against
  the value as *sanitised* rather than as it arrived, so a caller handing in `1.5` got
  `changed == false` for ever while the component rewrote the value under it — and `NaN` is what makes
  it load-bearing, because `NAN.clamp(0.0, 1.0)` is `NaN` and `NaN != NaN`, so a kept non-finite value
  reports a change on every frame for ever.
  Register 181 → **190 rows, 174 evaluated**; `glyphs` gained `VLine`, because a vertical groove is a
  column.
  **The application is `mixer`**, and `x` is the key to press — fifty steps in one keystroke, with `f`
  swapping the arithmetic underneath. It found two defects of its own in coordinates no gate here
  uses: a three-cell track in a seven-cell column left **four columns a channel nobody writes**, and
  the tail was computed from `CHANNELS.len()` rather than from the channels that fitted, so below 61
  columns the skipped columns were written by nobody. Threading an `Ink` through the draw turned the
  argument into a number — `--probe` prints **3 000 writes over 3 000 distinct of 3 000 cells**, 12
  regions and 11 tab stops. **`q` is a quit key here and it is the first application on this map where
  that is true**: a focused `slider` takes cursor keys and nothing else, where a `field` consumes every
  text-bearing key and a `collection` eats it into a type-ahead buffer.

- **The last three Tier 2 rows ship, `pagination` is `collection`'s store rather than its row loop,
  and a `field` had been eating every cursor key it could not act on** (components ticket 35,
  2026-08-27; ADR 0042). `status_bar` is in `crates/vitui-components/src/structure.rs`, `pagination`
  in `src/collect.rs` and `form` in `src/input.rs`, so the freeze is **28 built of 29** — every row
  but `spinner`, whose mechanism is *a component that owns a clock* — `MOVED` goes nine → twelve, and
  `crate::composed`'s Tier 2 claim goes six rows → **nine** with nothing left on the unbuilt side.
  **The row loop is an axis and a paginator is the other one.** `table` is `collection` plus a column
  split and `tree` plus a flatten index, and both *call* `collection_into`; a pager cannot, because
  the row loop hands its drawer `Rect::new(0, y, area.w, 1)` and asks `Ctx::visible_rows` which rows
  are reachable — both the vertical axis by construction. **A transpose is not a rectangle split, it
  is a different `Ctx`**, so what `pagination` reaches is the half of `collection` with no axis at
  all: the store (`CollState`, its `offset` read as the first visible *page*), the thirteen arms of
  `apply`, and the **one drain loop**. The gate is the equality — the same key at the same length
  lands a pager and a `collection` at `Mode::Options` on the same index — and `nav::step` reading
  `←`/`→` as `↑`/`↓` is components 17's collision arriving as exactly the right answer.
  **The sweep was vacuous when it was written**, which is components 33's finding a second time: the
  first frame and the drive loop were two call sites, so `Ctx::id` minted two ids, the planted focus
  named a widget no later frame declared, **not one key was delivered**, and both arms stood still at
  zero for all thirty-nine rows.
  **The axis argument is the band's and not the bar's.** `status_bar` draws through `scroll::sticky`
  and mints nothing, and the clip is what that buys — a segment written by arithmetic lands on its
  neighbour and re-damages those cells for ever. But a bar's content is one row derived from its own
  segments, so `Shares::Y` and `Shares::Neither` draw the same bar at every offset (**0 cells of
  30×2 apart** at four of them) and only `Shares::X` moves anything, and only at `Fill::Natural`.
  **A container that moves the focus inside itself mints its children's ids.** `Ctx::id` is
  `Location::caller()`, so a form cannot ask what id row 7 would have — and it must know **before**
  it draws, because the key that names a row is handed back by the focused field at `Ctx::scope`'s
  after-the-body moment. So the ids are arithmetic — `Id::keyed(form_id, row)` through
  `input::field_keyed` — which is `collect::Cell::id`'s answer one component over, and
  `input::form_row_id` is public because architecture issue 25 says the *application* seats the first
  focus. **`FormState` is `size_of::<TypeAhead>()` and nothing more**, which is how *no new
  mechanism* stops being a claim: the cursor is the focus, read back with `Ctx::is_focused`.
  **A form is one tab stop and the runtime's vocabulary is what says so** — `ScopeKind` has three
  arms and the other two are a modal and a code editor, so a form that is not a `Group` is a form
  with **no `nav::cursor` at all**: 6 ring entries / 1 stop grouped, 6 / 6 flat, and `Down` moves the
  focus on exactly one of the two. Its type-ahead is unreachable while a field holds the keyboard,
  which is §3's own sentence from the other side.
  **The defect is in code that was already green**: a `field` consumed `Up`/`Down` even where a
  one-row `input` has no row to step to — components 20's *declared and consumed nothing* on the
  keyboard axis — so every `form` above it had no arrows, on a screen that rendered perfectly. The
  answer is the caret's own position rather than a flag on the kind.
  **Two more the gates found in themselves.** The allocation window caught `pagination` spelling its
  page labels with `to_string()` — **9 240 over 60 frames** at 137 pages, with the writes, the verbs
  and the picture identical either way; it is `Digits` on the stack now and the negative case is
  `input::defective::form_collecting_labels`, the record-shaped form, which pays **60 over 60**
  because `nav::cursor` takes `&[&str]`. And the partition sweeps drew at the screen's own edge,
  where the screen clipped every overrun: given two cells of margin the pager was writing **5 cells
  into a 4-cell strip** and **2 into a 1-cell one**. The recorder and the defect sharing a coordinate
  system, for the third time on this map.
  **And a third finding, in the column rather than in the code**: §17's `glyphs` column is what
  §16's within-component collapse gate runs over, so a component drawing a marker its column does not
  declare is a gate running over less than the component draws. `crate::composed` is the one place
  carrying a component's own **section**, so the join is possible there — *a Tier 2 row whose section
  reaches an eliding call declares `Ellipsis`* — and it found `rule` under-declaring since ticket 34
  beside `status_bar`, `form` and `pagination`. The three toggles are excluded **by construction**:
  one machine, one section, and `switch`'s empty column is a finding rather than a hole. 20 of the 29
  rows drew a glyph; **21** do.
  Register 200 → **209 rows, 193 evaluated**.
  **The application is `roster`**, and `Ctrl+G` is the key to press: it takes the form's `Group`
  away, the stops go 2 → 7, `Tab` starts walking the fields and the arrows die. `--probe` prints
  **3 000 writes over 3 000 distinct** at 100×30, 9 regions and 2 tab stops, and sweeps 320 sizes for
  the partition. **There is no `q` to bind** — a focused field consumes every text-bearing key, which
  is `compose`'s finding for the second time.

- **O1 is green — the first of the five obligations to turn — and its population is `built`, which
  is a finding rather than a convenience** (components ticket 36, 2026-08-27; ADR 0043).
  `crate::doc` is O1's evidence as a value: a page per built component, **located** by the freeze's
  own `families` column rather than listed, with a scan that opens each file and reports what is in
  it. `DOC_TESTED` is twenty-eight ids and `crate::doc::doc_tested` derives the same list from the
  files; the crate compiles under `#![deny(missing_docs)]`. It had stood at zero for **thirty-five
  tickets, twenty-five of which shipped a component entitled to add a row to it and none of which
  did** — `obligations`'s own *a list filled by whichever ticket happened to write a doctest is a
  list nobody audits*, holding for twenty-five tickets and then needing one to discharge it.
  **§17 states O2's second equality over `built` and states O1's count over nothing at all**, so the
  reading was owed. `spinner` is the one unbuilt row and a doc page for a function that does not
  exist is not a page anybody can write, so asking for one puts a permanent row in the failing set
  **no ticket on this backlog can invert** — *a query stuck red is `Verdict::of`'s vacuity failure in
  mirror image*: it reads red whatever happens, so nobody reads it. The population **moves**, so the
  day `spinner` ships the query asks about twenty-nine with no edit.
  **The axis criterion cannot be met by mentioning the axes you have, and the number is why.**
  **Thirteen of the twenty-eight declare none at all**, so a page that names the axes it has is
  silent on nearly half the freeze and silence is indistinguishable from a page that forgot. Every
  page carries one line — the freeze's own words in the freeze's own order, or `none` — and the gate
  is an equality **in both directions**: a page listing an axis its row does not set fails as loudly
  as one omitting an axis its row does. A scan for the words anywhere in the prose was refused
  because it passes on *it does not narrow* and fails on a page explaining why an axis does not
  apply — two mistakes with opposite signs that one scan cannot separate.
  **The needle problem, met a fifth time, and the file is the freeze's and not the name's.** There is
  a `pub fn text(` in `document.rs`, a `pub fn chip(` in `state.rs` and a `pub fn table(` in
  `gates.rs`, and **none of them is a component**; `Component::module()` is what disambiguates, so
  the join that finds a page is §19's own. The boundary is `(` **or** `<`, because
  `pub fn file_preview_pane<T, F>(` could never have matched the parenthesis.
  **A `compile_fail` fence is the opposite of evidence for O1**, and the report prints the census
  rather than concluding from a zero: 28 fences on the twenty-eight pages, 28 of them run, and the
  eleven hostile fences in the ten files the pages live in sit on `WhyThereIsNo…` items of their own — *the rule
  is right and this crate is not the evidence for it*.
  **The gate was wrong first, three times.** `running_examples` tracked *am I inside a running
  fence* rather than *am I inside a fence*, so the line closing a `compile_fail` body read as the
  opening of a running one — and **the crate is on the wrong side of that arm to notice**, since
  every page here carries a running fence anyway. The negative arm of the `deny(missing_docs)` scan
  first pointed at `doc.rs`, which contains the literal three lines up: *a scanner looking for a
  literal contains that literal*, for the third time on this map. And a review found the third **in
  the clause the whole obligation rests on**: `calls_itself` was a substring search for `<id>(`, and
  **`Ctx::text` exists and every drawing doctest in this crate calls it** — so `text`'s page could
  drop its own call, draw with `cx.text(…)`, and go on holding with O1 `Met` and nothing proving the
  API is callable from outside the crate. The collision is **already live** in `file_picker`'s
  example. The call has to be a *free* one, and the same review found the `E0503` needle was five
  characters over a seven-thousand-line file: it is `one level away, as e0503` now, the same kind of
  thing as the other three phrases.
  **And the review found `scene_numbers` panicking, which is ticket 20's finding in the file that
  finding was about.** Its `covered` assertion had said **17** since components 20 and components 32
  took it to 20 — *`cargo test` does not run an example*, so an `assert!` there is compiled by
  `cargo clippy --all-targets` and evaluated by nobody. **O5's own report had been broken for four
  tickets** while `unmet(o5(AXIS_SCENES)) == (34, 14)` — the gate — stayed green throughout.
  **`deny` and `warn` are the same gate here, measured rather than assumed** — `warnings = "deny"`
  already turns it into an error and an undocumented `pub fn` failed the build under both spellings —
  and `deny` ships because the two agree only while that table exists. **There is no sixth CI job**:
  `cargo test --doc` is spelled `cargo test --workspace`, whose own comment already says the
  doctests are where the negative gates live.
  **`overlay::OWED_SENTENCE` gained a fourth phrase**, because O1 asks for the sentence *with the
  `E0503` trap named* and the two halves are not the same claim: three phrases are the mechanism and
  `E0503` is the string a reader searches for at the moment they meet it, **since the diagnostic
  never mentions the overlay**. Register 209 → **212 rows, 196 evaluated** — rows 210, 211 and 212,
  because **row 30 is `Kind::Count` and O1 is two gates**, which is what §21's own `mixed` in the
  kind column was hiding.

- **The six Tier 2 rows ship, Tier 2's claim is a value a scan runs over, and `switch`'s empty glyph
  column is the finding rather than the hole** (components ticket 34, 2026-08-27; ADR 0041).
  `crates/vitui-components/src/composed.rs` is the claim — six rows, each naming what its component
  **reaches** and what it may not **mint** — and `crate::indicate` went from a nine-line stub to the
  F5 module homing `meter` and `sparkline`. The freeze is **25 built of 29**, and `MOVED` goes three
  → nine.
  **A scan for absences alone goes green when the section is deleted**, so every row carries `uses`
  beside `mints` and the negative case has **three** arms: a section that is *gone* fails separately
  from one that is present and wrong. The third section terminator is a `defective` module and it is
  load-bearing — every one of them in this crate is a *deliberate* collection of the spellings its
  component refuses, so a scan that ran into one would report the refusals as the component's own.
  **The mark is one glyph and one absence.** The backlog says *a `Glyph` pair*; the freeze's column
  says `Tick`, `Bullet` and **nothing**, and the *off* half cannot be a glyph at all because
  `CONTEXT.md` defines one as a lookup with **no spelling blank**. So `switch`'s empty column is a
  fact: measured at all three rungs, a checkbox and a radio differ in exactly **1** cell between on
  and off and that cell came out of the theme's table, while a switch differs in **3** and none of
  them did — its state is two words, the side its knob sits on and the face. **Three axes, one of
  them the palette**, and the one row of twenty-nine whose state survives ASCII *and* no colour.
  **`meter` shares `chart`'s ladder and cannot share its spelling, and that is a fact about
  Unicode.** `geom(Kind::Bars, set).sy` is `1 / 8 / 8` — where `constructions: 2` comes from, and the
  only thing `indicate` asks `chart` for. `chart`'s bars grow **upward**, so its partial cell is
  U+2581…U+2588; a horizontal meter grows **rightward**, which is U+258F…U+2588, a *different*
  contiguous run going the other way. They agree at both ends and at **0 of the 7 partial eighths**,
  so the **vertical** arm reaches `chart::raster::cluster` and the horizontal one reads the module's
  own run. **The disagreement is the assertion**: a meter that transcribed the table onto the wrong
  axis would draw a bar growing upward inside a row and every counter here would call it correct.
  **`sparkline` is `chart`'s body with the chrome deleted rather than copied** — `chart::body_into`
  was extracted so that *no axes, no gutter, no axis loop* is a shared call rather than a claim. Its
  raster is **20 x 5 of 20 x 5** where a chart's is 4 rows and a gutter; it folds **1** time over 20
  frames; and its **120 writes are flat** at 1k, 100k and 1M points while its **verbs are 16 / 13 /
  11** — a run ends where a cell's owner changes, which is the *data's* property, so §21's
  *`verbs <= writes`, never verb equality across sizes* is the gate and the three figures are the
  report.
  **The allocation window found the one defect no other gate here could**: `rule` spaced its caption
  with a `format!` — **2 a frame, 120 over 60** — and the picture is identical either way. The spaces
  are the caller's now, exactly as `panel`'s title's are, and the four skippable parts come out at
  **1 / 2 / 3 / 2** verbs.
  **`Mode::Radio` does not exist**: §5 shipped it as `Mode::Options`, and a mode called `Radio` would
  be §5's thirteen match arms wearing one component's name. Recorded, not renamed. And **`MOVED`
  growing to nine is a finding about the gate rather than about the rows** — it reads any non-Tier-1
  row as claiming *not built*, which is right for Tier 3 and wrong for Tier 2, whose definition says
  nothing about whether anybody has written it.
  Register 190 → **200 rows, 184 evaluated**.
  **A review then found nine, and two of them are gates that could not fail.** *The three spellings
  are one draw* **compared a drawing with itself** in three components — arms 0 and 1 draw through
  `Direct` and only the `_into` arm's canvas was captured — and it **cannot** be written as a surface
  comparison at all, because `Direct` writes into the engine and nothing reads a cell back (ADR
  0023); what ships is the determinism check it is, plus a source scan for the calls that route the
  named spellings into one body. The shape came from components 33, so `slider`'s is repaired beside
  it. The **ellipsis half** of the `rule` verb test drew and dropped its tally under a comment
  claiming §16's one-cell marker rule was checked. Beside them: the application **never seated a
  focus**, so `Space` and `Enter` were dead until `Tab` — architecture issue 25's symptom, and no
  gate here can see it because every gate posts keys at an id it focused itself; and the
  **sparkline got a zero-row rectangle** at a size the guard admitted, at a terminal **17** rows tall
  rather than the 13 arithmetic predicts, because a `PanelOpts` is bordered *and* padded.
  **The application is `vitals`**, and `g` is the key to press: it steps the glyph rung and the
  meter, the checkbox and the switch each answer §16 differently on one screen. `--probe` prints
  **3 000 writes over 3 000 distinct** at 100x30, **4 regions** — the panel and the three toggles,
  with two meters, a sparkline and four rules declaring nothing between them — and **1 fold** over a
  hundred thousand points. `q` is a quit key here, the second application on this map where that is
  true.

Read these before working, in this order:

1. The spec for the layer being worked on — `.scratch/vitui-engine-architecture/spec.md`,
   `.scratch/vitui-runtime-architecture/spec.md`, or
   `.scratch/vitui-components-architecture/spec.md`. All three maps are **closed**; the specs are the
   authority. An `architecture.md` beside a spec is the superseded proposal, kept only as the record
   of what was argued.
2. `CONTEXT.md` — the glossary. Use its terms in code, comments, tickets and commit messages.
3. `docs/adr/` — 47 decisions that are hard to reverse and surprising without context. 0001–0011 and
   0022–0025 are the engine, 0012–0021 and 0034 the runtime, 0026–0033 and 0035–0047 the components.
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
crates/vitui-components   windows, panels, charts, lists, trees, forms, pickers (28 of 29 built)
                          └ and `gallery`, the assembled screen: 28 panels as a value the
                            application iterates, which is where O2's two equalities are measured
                            (ticket 39), and §21's row 7 — every cell of the rectangle written at
                            least once, green since ticket 40, with row 8 the one still red
                          └ every built one carries a doc page with a compiled example (O1, ticket 36)
                          └ and, for the thirteen that read a key, a declared keyboard contract whose
                            help is rendered from it and whose other half is a sweep that runs the
                            component (O4, ticket 38)
                          └ and a golden screen per construction under `tests/golden/` — 33 of them,
                            in the engine's format, blessed with `VITUI_BLESS=1` (O3, ticket 37)
                          └ plus `media`, which is **no row of the freeze at all** — §14's own *no
                            v1 component*, so the family ships and `MEMBERS` is empty
                          └ the partition primitives return `vitui_runtime::Rect`. This crate used
                            to name its own rectangle (`Cells`) because `vitui_engine::Rect` was
                            unnameable across the crate line; runtime issue 22 re-exported it and
                            components issue 17 deleted the stand-in
crates/vitui              facade re-export — engine, runtime, components
crates/vitui-apps         the applications, one file each in `examples/` — 16: `counter`, `triage`,
                          `latency`, `ledger`, `explorer`, `reader`, `settings`, `compose`, `console`,
                          `theatre`, `browse`, `mixer`, `vitals`, `roster`, `gallery`, `caps`. **A component ticket ships one**: the surface's
                          └ `caps` is the odd one and draws no frame: attach, read
                            `Capabilities::report`, detach, print. Every gate here is headless, so
                            none of them can answer *what did my terminal claim* — which is the
                            question a person holding a broken screen has, and the one that produced
                            `quirks.rs`'s sixth entry
                          only consumer, and four times now the thing that found the defect its gates
                          could not
                          └ a workspace MEMBER, so CI builds them: a consumer nobody builds is a
                            consumer nobody checks (`compare/run.sh` is the precedent). Depends on
                            runtime + components and NOT on the `vitui` facade — the facade
                            re-exports the engine, which would make `Rect` nameable here and
                            evaporate the proof that the component surface is sufficient
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
scripts/                  the four gates and one report that cannot be a `cargo test`: idle,
                          observer, lint rung, the gallery's panic-under-a-pty, steady state
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
cargo run -p vitui-apps --example counter   # the first real application; q to quit
cargo run -p vitui-apps --example console   # the overlay family; Ctrl+P palette, Ctrl+Q quit
cargo run -p vitui-apps --example theatre   # the media family; 4 is the floor of the colour axis
cargo run -p vitui-apps --example theatre -- --probe   # one headless frame, and what it cost
cargo run -p vitui-apps --example browse    # the preview pane; k then s is the memo-key rule
cargo run -p vitui-apps --example browse -- --probe
cargo run -p vitui-apps --example mixer     # the slider; x fifty steps, f swaps the arithmetic
cargo run -p vitui-apps --example mixer -- --probe
cargo run -p vitui-apps --example vitals    # the six Tier 2 rows; g steps the glyph rung
cargo run -p vitui-apps --example vitals -- --probe
cargo run -p vitui-apps --example roster    # the three Tier 2 composites; Ctrl+G takes the arrows away
cargo run -p vitui-apps --example roster -- --probe
cargo run -p vitui-apps --example gallery   # every built component on one screen; t is the key
cargo run -p vitui-apps --example gallery -- --probe    # the budget, measured in the gallery
cargo run -p vitui-apps --example gallery -- --matrix   # §16's nine cells, as counts
cargo run -p vitui-apps --example caps      # what THIS terminal answered; draws no frame
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
cargo run --example contract_numbers -p vitui-components   # O4: what each component declares and answers
cargo run --release --example gallery_numbers -p vitui-components   # O2: the screen, the matrix, what `t` costs
scripts/idle-gate.sh 30       # 0.00 user / 0.00 sys over 30 s; thirty is a floor, not a preference
scripts/observer-gate.sh      # the debug observer is absent from a release binary
scripts/steady-report.sh      # 60 fps for 30 s against 5% of a core
scripts/lint-rung-gate.sh     # the clippy.toml rung fires in an application and not from a dep
scripts/gallery-panic-gate.sh # the gallery restores the terminal before a panic prints, under a pty
n=1 cargo test -p vitui-engine golden        # regenerate the golden frames; review the git diff
VITUI_BLESS=1 cargo test -p vitui-components golden   # the components' 33 screens; same, and refused in CI
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
