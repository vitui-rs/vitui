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
- **`vitui-components` has started**: 28 of 45 tickets resolved (the last on 2026-08-26). `INVENTORY` is spec
  §17's twenty-nine-row freeze **as a value a test iterates**, with the five documentation and
  verification obligations as functions over it — so *which components must this gate run against* is
  answerable by the machine from here on. All five obligations are `Unmet` and each is watched
  panicking, because a query with no evidence must fail loudly rather than pass: O4 first returned
  **`Met` over 29 rows with no evidence at all**, since *an equality between two things that do not
  exist holds*. **Building the freeze contradicted four figures in the closed map** — the built count
  (19/10, not ADR 0033's thirteen-unbuilt), the count of empty families (five, not §17's two), §1's
  layer rule against §6's own composition, and the `layer` column being uncheckable without stated
  edges. All four are asserted as measured rather than bent to fit.
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
  needed a CI line. All twenty-one `*_numbers.rs` run to completion now, and **none of them is a
  gate** — the convention that no register row rests on one is untouched.

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

Read these before working, in this order:

1. The spec for the layer being worked on — `.scratch/vitui-engine-architecture/spec.md`,
   `.scratch/vitui-runtime-architecture/spec.md`, or
   `.scratch/vitui-components-architecture/spec.md`. All three maps are **closed**; the specs are the
   authority. An `architecture.md` beside a spec is the superseded proposal, kept only as the record
   of what was argued.
2. `CONTEXT.md` — the glossary. Use its terms in code, comments, tickets and commit messages.
3. `docs/adr/` — 37 decisions that are hard to reverse and surprising without context. 0001–0011 and
   0022–0025 are the engine, 0012–0021 and 0034 the runtime, 0026–0033 and 0035–0037 the components.
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
crates/vitui-components   windows, panels, charts, lists, trees, forms, pickers (16 of 29 built)
                          └ the partition primitives return `vitui_runtime::Rect`. This crate used
                            to name its own rectangle (`Cells`) because `vitui_engine::Rect` was
                            unnameable across the crate line; runtime issue 22 re-exported it and
                            components issue 17 deleted the stand-in
crates/vitui              facade re-export — engine, runtime, components
crates/vitui-apps         the applications, one file each in `examples/` — 9: `counter`, `triage`,
                          `latency`, `ledger`, `explorer`, `reader`, `settings`, `compose`, `console`. **A component ticket ships one**: the surface's only
                          consumer, and three times now the thing that found the defect its gates could not
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
cargo run -p vitui-apps --example counter   # the first real application; q to quit
cargo run -p vitui-apps --example console   # the overlay family; Ctrl+P palette, Ctrl+Q quit
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
