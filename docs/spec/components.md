# `vitui-components` — architecture spec

Status: **settled**. Date: 2026-08-19. Written by components ticket C12, the destination of
Map: vitui components architecture.

This document replaces `architecture.md`, which was a proposal written so the
map's tickets had something to attack. Its §1 is confirmed as written, its §3 is rewritten (one
helper deleted, three added, two kept for a different reason than the one recorded), its §5 coverage
table is superseded by §18 here, and its §2 module tree is confirmed only in its *rule*. The
proposal is kept unedited as the record of what was argued. Where it and this spec disagree, this
spec is right and the ticket named in the margin is why.

Nothing here is invention. Every decision below arrived on a ticket with a number behind it, and the
ticket is cited so the argument can be re-read rather than re-had. Where a question was left open it
appears in §22 rather than being answered here. The spec is done when `/to-tickets` can slice it into
implementation work without reopening a decision.

**Reading order for a newcomer:** `CONTEXT.md` (the glossary — its terms are used here without
re-definition) → `../vitui-runtime-architecture/spec.md`
(this crate's whole surface) → this file → `docs/adr/0026`–`0032` for the decisions that are hard to
reverse → the tickets in `issues/` for the measurements.

---

## 0. What the crate is

`vitui-components` is the widget library. It owns twenty-nine components in three tiers (§17),
eight shared helpers (§3), a glyph and distinction catalogue that the runtime's theme resolves
(§16), and an inventory value that every documentation and verification obligation is a query over
(§17, §21).

It owns **no drawing primitive** (the engine's), **no layout, identity, focus, routing, theme or
worker machinery** (the runtime's), and **no reactivity** (nobody's — ADR 0020). It depends on
`vitui-runtime` and on nothing else; it does not depend on `vitui-engine`, which is how C6 is
checked (`Cargo.toml`, plus `deny.toml`'s wrapper list — §21).

One sentence governs the whole document and it is not about widgets:

> **We will not implement every component that exists. Nothing that exists may be impossible.**

The positive half is §17, twenty-nine components frozen on what was built. The negative half is §18,
and it is the chapter neither the engine's spec nor the runtime's had to write: fifteen families and
~430 named entries surveyed live, against twenty-nine shipped, with the mechanism named for every
entry that is not one of the twenty-nine.

The second-order finding of the whole map is what §2 exists for. Twelve prototypes, each written by
someone who had read the rule the previous one wrote down, broke it anyway — and in every case the
defective build was **faster** and **marked fewer cells** than the correct one. This crate's
architecture is therefore not a taxonomy of widgets. It is a small number of rules with a counter
behind each, because the rules with only prose behind them were all broken.

---

## 1. The shape of every component

`architecture.md` §1's four rules are confirmed by C01 as written, on five components written by a
*user* — a crate depending on the runtime and on nothing else, assembled into a 300×80 screen with
**338 interactive regions**.

```rust
pub fn button(cx: &mut Ctx, area: Rect, label: &str) -> Response;
pub fn checkbox(cx: &mut Ctx, area: Rect, label: &str, on: &mut bool) -> Response;
pub fn field(cx: &mut Ctx, area: Rect, text: &mut String, st: &mut FieldState) -> Response;
pub fn collection(cx: &mut Ctx, area: Rect, len: usize, st: &mut CollState,
                  row: impl FnMut(&mut Ctx, Rect, usize, Face)) -> Response;
```

1. **Function first.** `fn(&mut Ctx, Rect, …) -> Response`. No trait to implement, nothing to
   register, no lifecycle. (C5; the runtime's ADR 0014 is why there can be no second pass to
   implement one for.)
2. **Data by shared reference or by closure; state by `&mut` beside it.** A `&mut bool` is the
   widget's own value, never application data; a collection never takes the collection. (C7)
3. **Options are a `Default` struct, never a required builder.** Every component `f` has a sibling
   `f_with(cx, area, …, &FOpts)`. (C14 of the requirements)
4. **Return `Response`**, even from a pure drawer.

**What the shape costs at a call site, measured rather than asserted.** Four of the five components
carry no lifetime annotation at all. The fifth opens an overlay, and `'f` costs it **two
annotations** — `menu_button<'f>(cx: &mut Ctx<'f, '_>, …, items: &'f [&'f str], …)`. Written without
them, rustc says *lifetime may not live long enough* and *consider introducing a named lifetime
parameter*, which is the fix. The row closure needed no destructuring, no turbofish and no explicit
HRTB. Nothing in the five needed anything the runtime does not publish. (C01)

**One diagnostic path is a trap and the doc owes a sentence.** On an overlay body that captures its
owner's state by `&mut`, rustc's own `help:` line — *add explicit lifetime `'f` to the type of `st`*
— **compiles**. What then fails is the caller, with `E0503: cannot use st.open because it was
mutably borrowed`, naming the caller's own read one level away from the mistake and never mentioning
the overlay. `Ctx::overlay`'s documentation owes: *the body answers through the inbox; a `&'f mut`
capture compiles and costs you the state for the rest of the frame.* (C01)

**A component is one of six layers and may depend only on lower ones** (`COMPONENT-HIERARCHY.md`
§4): `L0` pure drawer · `L1` one piece of state · `L2` virtualised · `L3` container · `L4` owns a
layer · `L5` compound. The layer is recorded per entry in `INVENTORY` (§17), which is what makes the
DAG checkable rather than a review habit.

---

## 2. The partition rule

This is the crate's central rule, it has three tickets and eleven measurements behind it, and it is
[ADR 0026](../../docs/adr/0026-a-component-writes-a-partition-of-its-rectangle.md).

> **Every component and every helper writes a partition of its rectangle.** Each cell it is
> responsible for is written **exactly once**, by exactly one of its branches, and the cells it does
> not write are **named in its return value**.

The rule arrived in three widenings, each from a different ticket, each because the previous
statement was too narrow to catch what came next.

**Widening 1 — from fills to writes (C01).** The runtime map handed this map *a component that fills
its rectangle before it draws into it re-damages what it drew* (R07: 26 of 48 cells on a steady
dropdown frame; 10 814 of 24 000 on a dense screen). Built, the rule turned out not to be about
fills. It is **no cell may be written twice with different values in one frame**, and C01 found it
in five places, three of which are not fills:

| where | cells re-damaged every steady frame |
|---|---|
| the application's own `cx.clear(body)` at the top of every frame | **6 662** |
| `chip` filling its face before drawing its label — R07's original | **2 648** |
| a chip that does not narrow, whose label runs into its sibling's rectangle | **432** |
| a modal's scrim filled *under* the dialog rather than around it | **229** |
| a `panel` drawing its top border as one run and writing its title over it | **15** |

**The 15-cell one is the finding.** It was written by the author of this ticket, in the corrected
build, immediately after writing the corrected chip; nothing about a border run crossing five title
cells looks like a fill. It was found by a count and by nothing else.

**Widening 2 — from cells to the rectangle (C02).** *No cell twice* is a partition's first half.
Stated as a partition, the container case follows without argument: **`block` draws its frame, its
title-split top run and its padding ring, and returns the interior unwritten.** A `block` that
clears what it hands over costs **22 200 damaged cells a frame** across three panels. The four
non-fill instances above stop being four mistakes and become one.

**Widening 3 — the second half has a name and had no instrument (C11).** *No cell never* — every
cell of the rectangle written at least once — was counted by nothing on either map, because a cell
nobody writes keeps what was already there and what was already there is almost always right. The
detector is a **sentinel**: stamp a `Theme::custom` style over the **base** layer between frames,
draw one more, count the cells still carrying it. It must be the base layer, because the engine's
equality filter makes a cell rewritten with its previous value indistinguishable from one never
written.

Run over the assembled gallery: **9 956 cells of 53 280 (18.7%), in six panels of twelve**, and they
are four different mechanisms.

| panel | unwritten | what owns it |
|---|---|---|
| C07 | **4 189** — the whole body | the **caller's**: the gallery fills its panel background once, guarded by a `painted` flag, which is C01's rule doing exactly what it was written to do |
| C16 | 2 159 | the directory rows' tails and the preview body's |
| C13 | 1 799, columns 67–118 | a viewport 118 columns wide over 60 columns of content |
| C15 | 1 286 | the picture, the QR symbol and the waveform each centred in a wider rectangle |
| C08 | 496, columns 103–118 | the legend pane |
| C09 | 27 | the five-column gap between two half-panels |

**The two halves meet at the same fill and neither one alone resolves it.** C07's panel is the
sharpest case: filling every frame is the double-write defect, and not filling is the unwritten
defect, and the resolution is that the *rectangle* and the *content* are told apart — the owner of a
rectangle writes all of it; a component handed a rectangle writes all of that. Stated as an
equality in both directions:

```
writes == distinct cells touched            (no cell twice)
distinct cells touched == area.w * area.h   (no cell never)
```

The first is C02's gate and every prototype prints it (0 for a correct build, **43 941** for the
naive one, read as a differential with the equality filter off). The second is C11's sentinel gate
and it is **red on six panels**, pinned in its failing state with its exact failing set, because
C10 split the defect three ways and gave this map the rule and the detector while the code belongs
to the implementation backlog.

**What the rule is worth, on one screen, three times.**

| | naive | correct |
|---|---|---|
| C01, 338 regions | 214.58 µs / 69 647 writes / **6 662 damaged** | 84.96 µs / 20 784 / **0** |
| C01, the frame one chip is hovered | **6 662 damaged** | **8** — 833× |
| C02, 267 regions | 131.29 µs / 67 941 / **23 098** | 69.33 µs / 20 804 / **0** |
| C02, the frame one chip is hovered | **23 098** | **8** |

**And the two screens are the same screen: 0 of 24 000 cells differ**, at 300×80 and at 120×40. That
equality is what stops the damage count being gamed, and it is not decoration — a "correct" build
that merely drew less would score 0 marked cells by drawing nothing, which is exactly what three
separate defects on this map did (§21).

Earning the equality cost one rule of its own: **the correct build clears once, on its first frame
and on a resize**, because a screen whose gaps are never painted is not the same screen. Same call,
`cx.clear(theme.body)`; the difference between once and every frame is 6 662 damaged cells.

### Why the rule needs a counter and not a paragraph

Three amplifiers make the violation invisible and the violator cheap.

- ~~Damage is one span per surface row (C13, C16) … ×8.4 … 300 cells re-composited every frame
  forever.~~ **Withdrawn: the engine measured per-row spans and rejected them.** What ships is a
  **per-row bitset** with a dirty-row summary word (engine §6), chosen precisely because spans are
  *"lossy without warning"* — two popups 100 columns apart are 180 emitted cells with the bitset
  against 280 with spans. Under the shipped structure this amplifier does not exist: C16's panel
  costs **four cells, not 300**, and the two-bars case is ≈1.0× rather than ×8.4.
  **ADR 0029 survives untouched**, because its primary number — 3 535 re-damaged where the body draws
  *under* the bar — is a **double write**, and a double write costs the same under any damage
  structure. It is the amplifier that was structure-dependent, not the rule.
- **The defective build is faster and marks less.** C05's stale tail: **2.3× faster, marking 226×
  fewer cells**. C14's, one layer up: **1.19× faster marking 2.42× less**. C08's culled plot: 1.39×
  faster with the write count not moving at all.
- **A restyle looks free and is not.** §3 of the proposal said focus is shown by "a restyle, never a
  redraw". Built, it re-damages its range every frame forever: **26 cells** for one focused field,
  **31** for a selected row, **56** for a ring drawn into its neighbours' cells, **261** for a
  selected range (C03) — and C03's restyled row is **7 cells different** from the correct one,
  because a restyle repaints the row's sub-widgets. The exact rule:

  > A restyle is free only when the component's own next draw already produces the value the
  > restyle produced.

  R05's deferred hover award satisfies it, because the widget also branches on `Response::hovered`.
  A restyle used to *replace* the branch never can. That is why `focus_ring` is deleted and
  selection is a paint (§3).

**The scrim is the most expensive thing a component can draw and drawing it at all is the mistake.**
Filled as cells under the dialog: **+54.17 µs on every frame the modal is up** — 63% of the whole
correct frame — and 229 damaged cells for a dialog that is not moving. Drawn as the four rectangles
*around* the dialog: **0 damaged cells** and 600 fewer writes. Drawn as the engine's operator layer:
neither. (C01, owned by C07 — §12.)

---

## 3. The helpers

`architecture.md` §3 proposed six. **Five survive, one is deleted, three are added, and two of the
five survive for a different reason than the one written down.** The reason the table exists is not
consistency: it is that three of these helpers are the only place a rule with no compiler behind it
can be kept. (C02)

| helper | what it decides once | shape |
|---|---|---|
| `text::fit` | truncation, alignment, padding | **the partition primitive**: text, then the remainder — there is no verb that fills first |
| `frame::block` | border, title, padding ring | draws its frame, **returns the rectangle it did not write** |
| `state::press` | the face drawn **and** the face awarded | one role, so the two cannot disagree |
| `frame::face_paint` | selected / cursor / active / hovered / disabled | **a `Paint` the row drawer is handed** (C03 narrowed C02's `selection`) |
| `scroll::bar` | the bar every scrollable draws | thumb, then the track above and below |
| `keys::text` | what a text widget accepts | **a chord is not text, and a capital is not a chord** |
| `nav::cursor` | what a `Group` moves with | arrows, `Home`/`End`, `PageUp`/`PageDown`, type-ahead with a deadline |
| `frame::focus_ring` | — | **deleted.** A `Role` into `block`, a `Faces` into `press` |

**The two partition helpers return `vitui_runtime::Rect`, which is what these two rows have said all
along** (architecture issue 17). For eleven tickets they did not, and the reason was not a design
choice: `Rect` is `vitui_engine::Rect`, §0's constraint C6 says this crate depends on
`vitui-runtime` and nothing else, and the runtime re-exported it nowhere — so *returns the rectangle
it did not write* was **not writable as Rust in this package**, and the crate named its own
rectangle, `Cells`, with the algebra of `layout::rect` duplicated inside it and a corpus sweep
holding the copy honest.

Runtime architecture issue 22 settled that the runtime re-exports every engine type its public
surface names, so `vitui_runtime::Rect` is a path a component may write; issue 17 then deleted
`Cells`. **C6 is untouched** — the dependency table is still one crate — and what went with the type
is the second rectangle in one workspace, the duplicated algebra, and the sweep that existed only
because the duplication did. One thing got structurally simpler rather than merely shorter: the hover
award used to reach the runtime through **two** hops, `Cells::hover_style` and then `ink.rs`, the
first of which existed *only* because a component could not name a `Rect` to pass. It is now one.

**Routing through the helper costs nothing against the discipline.** `fit` against the same order
written out by hand: **0 of 24 000 cells differ, 20 804 writes against 20 804.** That is the
measurement this table owed — the helper is not a convenience over hand-written correctness, it is
the only form of it anybody keeps.

**`press` exists for a disagreement, not for a state machine.** The `PressState` the proposal names
was built and its field was never read: `Response` already carries `hovered`, `pressed`, `released`,
`clicked`, `double_clicked`, `long_pressed`, and there is no cross-frame fact left. What the helper
is for is that the face a widget *draws* and the face it asks to be *awarded* are two statements
with nothing connecting them; written by hand they are correct on every frame and re-damage **8
cells for as long as the pointer rests on the chip**. (C02)

**`scroll::bar` draws the thumb first.** Track-first is **221 cells a frame** for a two-bar screen
(C02), **345** on C13's. Same rule, one helper down.

**`keys::text` is the components crate's own predicate and it is not the runtime's.** R12's
`Mods::SIGNIFICANT` includes `SHIFT`, correctly, for the question R12 asks. A text widget's question
is different, and the difference is one line: `Mods::significant() = CTRL | ALT`, Shift deliberately
excluded. Measured through a focused field: `Ctrl+S, h, i` gives `"value 0shi"` matching on `code`
alone and `"value 0hi"` declining chords; declining on `intent()` instead gives `"i"` where `"Hi"`
is right. In a `textarea` (C06): matching on `code` alone is **2 bytes gained and 0 keys reaching
the application**; `next_text_key` is **1 byte gained and 1 key reaching it**. This is the
components-side half of R12's registry, and without it every accelerator in the application is
swallowed by whatever text widget holds the focus. (C01, C02, C06)

**`nav::cursor`'s placement is the decision, not its contents.** A collection opens its own
`Group` scope, so *a list is one tab stop* is true of every list: **266 tab stops become 69** on
C02's screen. Type-ahead needs one `cx.deadline_for`, or the buffer expires at the next keypress —
the keypress whose meaning depends on it. (C02)

**Density is theme data, it changes rectangles, and `block` is where that lands.** `Compact` against
`Cosy`: 20 804 writes and 267 regions against 20 992 and 263 — four widgets fall off the bottom of
the form because the padding is real — **and neither writes a cell twice.** (C02)

---

## 4. Identity at a call site

The runtime settled that identity comes from the call site and may never be persisted (ADR 0013).
This map found the three places where a call site stops being visible, each silently, each on a
screen that renders perfectly. It is
[ADR 0027](../../docs/adr/0027-identity-is-taken-outside-every-closure.md).

> **A component takes its id outside every closure, and a container roots its children inside its
> own id.**

**The silent defect at a call site is identity, not the fill order.** The first screen written for
C01 had **110 of its 338 widgets inert**: six loops over one call site each, no `cx.with_key`, first
claimant wins, and the other 110 claim nothing. The screen renders pixel for pixel correctly —
drawing does not consume an id — and a pointer on the thirteenth toolbar chip lights **nothing**.
The runtime spec says *a loop needs a key* in one line of its §5, and that line does more work than
any other line in the document.

**`#[track_caller]` does not cross a closure, and `scope` is a closure.** C02 shipped this bug and
its own merge counter found it: `list` opens a `Group`, the body runs inside the scope's closure,
and `Location::caller()` inside the body is the line *in `list`* that invokes the closure — so every
list in the application is one list and one of them is inert (**266 regions and 1 colliding site
against 267 and 0**). Neither function was missing the attribute. This supersedes C01's narrower
wrapper rule rather than repeating it.

**`scope` roots no identity, deliberately, so a collection's rows collide across collections.**
C03's per-row-hit build produced **72 merges — 72 of 160 targets inert**. R08's `scope` pushes no
id, so rows drawn through a line inside the component derive their ids from `(screen, key, that
line)`, identical in every collection on the screen. `cx.with_id(id, …)` around the row loop is the
fix and it is one call: **389 regions and 0 merges against 301 and 72**. It is latent rather than
absent in a correct build, because the caller's row closures sit at different source lines — and it
becomes live the moment a target moves inside the component. Reproduced one ticket later with two
tables on screen: **284 of 911 targets inert** (C04), and **40 merges** with two trees (C05).

**One axis out, the same defect has a different arithmetic.** Keying a table per row rather than per
cell is 1 µs cheaper and correct — until a cell declares a target, and then it is **142 merges**.
(C04)

**The detector is free and already computed.** `merges == 0` is a count the runtime makes every
frame at 24.1 ns a widget. It caught every instance above, including the two that were written after
the rule.

**The id is opaque, and every workaround on this map that looks like a hack is that fact.** R07 O6
roots an overlay's stack at its owner; an `Id` is a hash, so nothing recovers the rooting from the
value. Third independent sighting: C03's colliding rows, C05's refused `Stash`, C07's submenu. A
component with two overlays standing must **mint** a second id, and `with_key` is the only verb that
does — shared, the two get **one slot resized, 8 allocations and 564 cells a frame** (C07).

---
## 5. `collection` — one component, one `Mode`

`list`, option list, menu, multi-select, tabs, radio group and segmented control are **one component
and one `Mode`**, and the whole difference between a radio group and a file manager is **thirteen
match arms**. Together with §6 and §7 this is
[ADR 0028](../../docs/adr/0028-the-collection-is-one-component.md). (C03, C10)

### Three facts, not one

The store is the **cursor** (`lead`), the **anchor**, and **what is selected**. A menu keeps only the
first; options and single-select keep the third at one element; multi-select uses all three.

**Selection is a sorted, disjoint run list**, and the property that decides it is that *select-all
is one run whatever the length* while every other gesture adds at most one — so the store is
proportional to the number of gestures and never to the number of rows.

| gesture, 1M rows | run list | bitset | `Vec<usize>` | `HashSet` |
|---|---|---|---|---|
| `Ctrl+A` | **0.08 µs / 16 B** | 1.71 µs / 125 KB | 164 µs / 8 MB | **23 438 µs / 16.5 MB** |

The `HashSet` figure is one keystroke at 234× the whole frame budget. And the wrong store is not
only a gesture cost: a `Vec<usize>` select-all over 1M costs **8 221 µs on a steady frame — 82× the
budget — but only once the list is scrolled.** At the top of the same selection it is 69 µs and
looks correct.

The row loop reads the runs through a **scan cursor** seeked once and advanced in lockstep with the
visible window: `O(log k + h)`, not `O(h · log k)`.

### The row signature is `(cx, rect, index, Face)`

C02's `Sel` enum does not survive: its variants are not exclusive. A row can be selected *and*
hovered, and it can be the keyboard cursor without being selected — which is what `Ctrl+↓` does and
what every file manager draws. The enum names 4 of the 32 states five independent bits can be in.

`Face { selected, cursor, active, hovered, disabled }` — **5 bytes** — is handed to the row drawer,
and `face_paint` is the one collapse point. Its precedence is the decision: **disabled >
selected+active > selected > cursor > hovered > base**. C02's rule is kept exactly — the paint is
resolved before a cell is written — and the trade is worth stating: the parameter costs the row
drawer one binding, and an unused binding is a warning while a missing highlight is not.

### One hit entry per collection

`Response::local` is computed inside `interact_id` from *this frame's* pointer, unlike `hovered` — so
resolving the row by arithmetic (`offset + local.1`) gives per-row hover with **no frame lag and no
per-row index entry**. Declaring per row is strictly worse on both counts: **389 regions against
229**, for a frame-old answer. R05's rule is unchanged for everything else: a row with a target of
its own declares it, per target, for visible rows only.

The consequence reaches three later tickets. A closed popup requested at `h = 0` costs four hit
entries and not four hundred (C07) precisely because a collection declares one entry however many
rows it has; a table declares **59 regions against 1 195** for one entry per visible cell (C04); a
text widget declares **79 against 621** for one per visible cluster (C06).

### Per-row state is one slot, never a map

C01's open item, answered as a rule that can be checked from a type:

> **Per-row state is either derived from the row's data, or it is one slot on the collection naming
> the row that has it.**

`CollState` is **208 bytes** — offset, selection, type-ahead buffer, and `editing: Option<usize>`,
because only one row can hold an inline editor. Nothing keyed by row index may exist, and the
runtime already enforces the other half: a row that did not draw cannot be clicked, focused or
hovered (ADR 0012), so state for an undrawn row is state nothing can reach.

### Type-ahead is the caller's search behind a budget

It is the one navigation gesture that cannot be answered from the visible window. One `z` into a
focused million-row list: **70.79 µs bounded against 1 507.83 µs unbounded**; the search alone
**6.33 µs against 1 334.46 µs — 211×**.

### The frame

**63.9 µs / 0 marked / 20 914 writes / 229 regions / 31 stops / 0 allocations**, and **identical at
1 000, 100 000 and 1 000 000 rows** — 63.87 / 63.87 / 64.08 µs, same writes, same regions. The frame
the selection moves is **50 cells marked**, identical at 1k and 1M. A pathological selection — 1 000
ctrl-clicks on alternate rows — is 1 000 runs, 16 000 B, and a **64.92 µs** frame against 63.87 with
nothing selected.

### `Escape` is the container's key until the collection has a selection to clear

Architecture issue 22, resolved 2026-09-05, and it is the one place §5's keyboard was widened after
the map closed.

`from_key` reads `Esc` as `Gesture::Nothing` and the drain loop consumed it unconditionally, so a
plain `collection` inside `cx.overlay` at `OverlayOpts::modal` swallowed the one key a dialog is
expected to answer — found by running `crates/vitui-apps/examples/console.rs`, whose palette had to
close on a chord. **The narrowing is the answer and not the hook:** publishing `collection_shaped`
would have made the expected behaviour something every application has to know about, and
`Refusal`'s own documentation refuses it for a reason that stands.

The rule is `owns_escape`: **the collection owns `Esc` exactly when `apply` would clear something.**
Two of the four modes ignore `Gesture::Nothing` at all — `Mode::Cursor` never selects anything and
`Mode::Options` is *exactly one, and it can never become zero* — and in the other two it is a no-op
over an empty selection. So the key is declined unless there is a selection to lose, and the
behaviour that falls out is the one every file manager has: **the first `Esc` drops the selection,
the second closes the dialog.**

`MODES_THAT_CLEAR` is the written-down half and is joined to `apply` by a probe over `Mode::ALL` —
written down rather than derived at the call site because deriving it there means cloning a
`Selection` on every press, which this crate counts.

**It found a key that had done nothing since the crate was written.** `pagination` is `Mode::Options`
and declared `Escape` in O4's contract, so the sweep reported it answered — because a key that is
*consumed* reads as answered whatever it did with it. A pager's `Esc` could never clear anything. It
moves out of the pager's prefix into the listing's group, `PAGER_BINDS` is **22**, and `REGISTERED`'s
pager row goes from 30 spellings to **28**. Two of the thirty were a key that did nothing.

The sweep itself had to change shape, and the shape was already in the file: a conditional binding
has to be driven in the state its condition holds in, which is why `select`'s probe opens the popup
before asking what `Esc` does. The four collection-backed contracts are now swept **twice**, with a
selection and without, and answered by the **OR** — which adds `Escape` and nothing else.

### The gesture that is inexpressible, and it is the runtime's

`rt::Input` is `Move`, `Down`, `Up`, `Wheel` and `Key`, and **only `Key` carries a modifier byte**.
Ctrl-click and shift-click — the two gestures every multi-select in the survey is built out of — are
therefore inexpressible above this runtime, and what an application is forced to substitute is the
modifier state of the last key event, which is wrong across R06's batched drain. **`Mode::Multi`'s
pointer gestures are keyboard-only until a runtime map reopens.** The keyboard half is complete and
measured: `Ctrl+A`, `Space`, `Shift+↑/↓`, `Ctrl+↑/↓` all route through `apply`. Recorded three times
(C03, C04, C05) and filed in §22.

---

## 6. `table` = `collection` + column rectangles

True, and **the `+` is paid in verbs, not in cells** — a currency nothing on either map had been
counting. (C04)

### The headline

The same screen, the same rectangle, the same **23 030 cells**:

| drawn as | verbs | µs |
|---|---|---|
| a plain list | 418 | — |
| a tree | 640 | 48.79 |
| a twelve-column table | **2 497** | 165.6 |
| one column | 913 | 71.5 |

Damage is marked once per verb, and a table cuts each row into twelve short runs where a list writes
one long one — then C02's partition makes each column **two** verbs, text then padding, because the
alternative is a cell written twice. **Two verbs a cell is the floor and there is no cheaper correct
shape.** A full-screen twelve-column table draws in **166 µs while damaging nothing at all**, which
is the draw and not the composite, and §20 owns what that means.

### Columns

A caller-side rect split through R03's solver, run once a frame over the declared columns, touching
no row. **`visible_cols` is as load-bearing as `visible_rows`**: letting the clip discard what does
not fit costs **3.3× the frame and 7.2× the verbs** at 120 declared columns (548 µs, 18 049 verbs)
for an identical screen.

### Pinning is three bands, and the band forces a clip

A pinned column may not be elastic: a pin claims a share of the *table* and a scrolling lane a share
of the *content*, and `max(viewport, Σ minima)` gives them two different denominators. Written the
tempting way — the offset as arithmetic, no view — the scrolling band's edge column overwrites the
pinned band: **identical writes, identical verbs, identical time, and 345 cells re-damaged on every
steady frame forever.** No gate left by C01, C02 or C03 sees it.

The band view is opened **per row inside one row pass**, not once per band. A pass per band measures
4% *cheaper* and is refused, because three loops share nothing but the author writing the same
bounds three times and cannot share §5's lockstep scan cursor.

### Cell selection is a run list per column key

C03's two candidates are not equivalent, because a table's row count is unbounded and its column
count is not. Under a flattened `row·ncols + col`, one click on a column header at a million rows is
**1 000 000 runs, 16 MB and 62 535 µs**, against **1 run, 16 B and 0.08 µs** — 780 000×, and it is
C03's rejected `HashSet` arriving as a different type. The per-column store's only losses
(select-all, select-one-row: 40 runs against 1) are bounded by the declared column count. The frame
does not distinguish them at all — both are amortised O(1) a cell — so the store is decided entirely
by what a gesture costs.

### In-cell editing

`Option<(row, col)>` and no wider — **0.29 µs to follow a million-row sort, because it is one
position.** The row half is revalidated against the order's revision (§10), which is a field beside
the slot rather than a widening of it. The column half needs nothing, because it is a **key** and not
a position: hide and reorder move positions, and **nothing stored may be keyed on one**.

### The twenty-two grid features

Twenty-one of the survey's twenty-two data-grid features are state or layout, each assigned with its
mechanism in `research/05-table.md`. The one that is neither cleanly is
**variable row height** — master-detail rows, group rows, a wrapped cell. It is not a drawing
question either, so the map's claim survives; what it is, is a layout question with a
data-proportional storage consequence, and C05 closed it as a **fourth field, not a fourth
structure** (§7).

### The frame

**165.6 µs / 0 marked / 23 030 writes / 2 497 verbs / 59 regions / 5 stops / 0 allocations**, flat at
1k / 100k / 1M rows **and** flat from 40 to 240 declared columns, with identical writes and verbs.

---

### A row is a partition of its rectangle on both axes, and the band writes its own slack

Architecture issue 24, resolved 2026-09-05, filed by production 06 while standing `table`'s shrink
axis up.

`solve_columns` answers `content_w = max(Σ minima, view_w)` and `table_with` draws the **columns**
inside each band. Nothing drew the band. With every scrolling column `Fixed` and narrower than the
viewport the remainder of every row was **untouched** — not painted with a space, which is
`crate::runner::Canvas`'s own distinction, so those cells kept whatever was already there.
**21 760 of 24 000** at three narrow columns, against **0** at `grid::columns(12)`, which overflows;
and **66 cells of every row** in `crates/vitui-apps/examples/ledger.rs` at three hundred columns,
with none at eighty, a hundred and twenty or two hundred.

**The answer is §2's and not a free choice**: *a component handed a rectangle writes all of it.*
`table_with` writes one run a row from the last column's right edge to the band's, in
`CollOpts::tail`'s Role — which is `collection`'s tail one axis over, down to the paint, and is what
makes it a repair rather than a new drawing. The header takes the same run for the same reason.

What decided it against *the caller owes it, and an all-`Fixed` list is a caller error* was the
**failure mode** rather than the rule. §2's second half is already pinned red on six panels of the
gallery, and every one of those is a **background** gap — a picture centred in a wider rectangle, a
viewport wider than its content. This one is not: an untouched cell keeps what was there, so the
first frame after a column set narrows keeps the wider set's **data** on the screen. That is §17's
`shrunk` axis with columns in the sentence, and a caller cannot be given the job of not showing
stale data it never wrote.

**The premise that either answer moves a normative figure turned out to be false**, and it is false
for the same reason the defect survived twelve tickets: every column list on `crate::grid`'s screen
exists to put *horizontal overflow* on it, so scene 7 never had any slack to write. `VERBS_TWELVE`,
`VERBS_ONE` and §6's 913 → 2 497 are unmoved. The figure that moved is `grid::COLUMN_RESIDUE`, from
21 760 to **0** — a tripwire that said in its own failure message that failing meant the defect had
been repaired, and did.

The repair has a second arm rather than an assertion: `TableShape`'s `Slack::Unwritten` is the
shipped table with one field changed, and it is what `COLUMN_RESIDUE_WAS` is measured on. Unlike the
stale tail, the band wrote its slack on **no** arm before this, so there was nothing to compare
against until the refusal was written.

`ledger.rs` is untouched and is now correct as it stands, which is the other half of the answer:
declaring twelve fixed columns is a legitimate thing for a caller to do.

## 7. `tree` = `collection` + a flatten index

Holds. The `+` costs **two verbs a row** — the indent run and the chevron cell, 222 verbs over 111
rows — and the index is the caller's, but it is **not C04's `Vec<u32>`**, and the difference is what
makes a fold affordable. (C05)

### The record, and why `depth` is in it

```rust
struct Row { node: u32, depth: u16, flags: u8, h: u8 }   // 8 bytes
```

The `depth` is not there for the indent. **It is there so a collapse can find the interval it
removes without touching the forest** — from the index that is a contiguous scan, from the data it is
one random access per removed row: **115 µs against 1 187 at 349 524 rows, 10.3×**. C04's order is
this record with three fields unused, which is the honest form of *three names for one mechanism is
one too many* (§10).

### Splice, always, with no threshold

**A collapse by splice is flat in what it removes** (157–342 µs at every subtree size from 2 rows to
999 999); **a rebuild is proportional to what remains** (40 393 µs at 52% folded). The rebuild walks
a shuffled forest at 85 ns a row and the splice moves records at 0.26 ns, so the crossover is at
about **99.7% of the index removed** — in a tree, only at the root, where it wins by doing nothing.
An expand by splice is proportional to what it inserts and beats a rebuild by `total / inserted`:
1.15× at the root, **700× at a 340-row subtree**.

### Variable row height is a fourth field

`Row::h` is a byte the record already had; the prefix sum is a `Vec<u32>` built in the same pass and
only when rows can differ. The index grows 57% (7 → 11 MB at a million rows), a splice costs
**2.25×** (244 → 549 µs) because the tail it has already moved must be re-accumulated, and **the
frame does not move at all** (48.88 against 48.79, identical writes and verbs). One footnote to the
map's Notes: **the index is O(1) per row and O(log n) per viewport**, because the content row at
screen `y` becomes a binary search — 0.019 µs against 0.0006, once a frame rather than once a row.

### A fold is an interval, which is why C04's answer does not transfer

C03 said a collapse's effect on a selection was C05's question; C04 said it was already answered.
**Both are partly right, and the difference neither names is that a sort is a permutation and a fold
is an interval.** A subtree is contiguous in pre-order display coordinates, so the edit is
`Splice { at, removed, inserted }` and the component *performed* it. Under an interval edit a sorted
run list transforms in **O(runs) and cannot shatter**: 1 000 000 rows, 349 524 removed, a half-tree
contiguous selection is **297 180 runs / 24 338 µs / 8 550 KB under a permutation, against 1 run /
0.04 µs / 0 KB under a splice — 608 000×.**

So the division, and it is §10's:

> **A splice is reconciled exactly and stamps the revision it produced. Anything else that moves the
> index leaves the revision behind and gets the permutation answer on the next frame.**

All three real policies are O(runs) and free — `Clear` 0.00 µs, `Drop` 0.08 µs, `Stash` 0.04 µs — so
the choice is behavioural: **`Drop` by default, `Stash` as the caller's opt-in** (the parked runs are
bounded by runs, not rows: a half-tree fold parks **one `Run`, 16 bytes**, and the round trip is
exact). **`Clear` is refused for a splice**, because it throws away the runs that were nowhere near
the fold for no saving at all.

### The frame

**48.79 µs / 0 marked / 23 030 writes / 640 verbs / 59 regions / 5 stops / 0 allocations**, flat at
1k / 100k / 1M nodes and unchanged at **depth 59 999**.

**An unclamped indent makes a row's cost proportional to its depth** — 3 812 970 cells asked for
against 23 030, **165×** — and it measures **18% faster**, because the clip eats them and the label
rectangle collapses. On a shallow tree the same flag is invisible: a statement about the scene list
(§21), not about the gate.

---

## 8. `collapsible` — one state machine

Accordion, tree node, code folding and inplace edit are one machine, and **the split is not between
the four components; it is between what their collapsed content *is*.** (C14)

`Collapse` is `open: bool`, a height, and an optional tween: **5 B of live state, 72 B with a tween
slot**, per section and **never per row of content**.

| the collapsed content is | who collapses it | example |
|---|---|---|
| rows in a caller-owned index | the caller, on request | tree node, code folding |
| a region of components | the component, on the frame | accordion, panel minimise |
| both | both | inplace edit |

### There is no transition state

No `Collapsing` and no `Expanding`. A transition state has to be *stored*, which means the machine
can be found halfway between two states with no clock running — the shape R11 refused when it
refused an animation object. `set` flips `open` **at the instant the gesture lands** and starts a
`Tween<u16>` from the current height, so `open` is never ambiguous and only the height moves. A
200 ms collapse is **14 frames to quiet, 46.00 µs worst, 1 789 cells, 0 allocations**, and the tween
asks for its own frames through `cx.deadline`.

**An animated fold is refused**: the removed rows would have to still be in the index while they
shrink, and the splice is an edit a frame may not perform. **A fold steps; a region animates.**

### "A component may only ask" is the *index's* rule

C05 forced it twice — a `Vec::splice` allocates, and taking `&mut` while the draw holds it shared is
`E0502`. Both reasons belong to the index. A region collapsible has neither: a click on an accordion
header goes **6 open → 5 open on the same frame, 46.79 µs, 0 allocations**, against the same gesture
on a fold index at **130.96 µs on data the draw holds shared**. So the rule is narrower than it was
written: *a component may not perform an edit; a collapse of a region is not one.*

### Closed content is **not drawn**, and the cells are not why

Twelve closed sections, body drawn into an `h = 0` rectangle instead of skipped: **the two surfaces
are identical**, and

| | µs | cells asked for | hit entries | tab stops |
|---|---|---|---|---|
| body skipped | **24.38** | **23 034** | **70** | **15** |
| body drawn at `h = 0` | 49.25 | 56 298 | **478** | **423** |

`interact` pushes a hit entry and a ring entry *before* it looks at the rectangle, so 408 of those
stops are invisible and **no golden-cell gate can see it**. The same defect at lower amplitude runs
for the length of every transition: halfway down a collapse, a body that does not cull declares
**273 ring entries against 247**. The rule is one line: **a body is handed a rectangle and draws
inside it.** C07 measured the same shape in the overlay family — a closed popup requested at `h = 0`
renders the identical 80 rows and declares **321/318 against 317/316** — small there only because a
collection declares one entry (§5).

### The height is an argument

The runtime has no measure pass (ADR 0014). A **sizing function** beside the body opens a section at
**22 cells, 0 rows wrong, settled in 2 frames**; the drawn extent opens it at **83 cells, 8 rows
wrong, settled in 3**, because with no answer yet the only thing to do is hand the body the whole
viewport and read how far it reached. The watermark also costs **+11.5%** of the frame (48.75 against
43.71 µs) — R17 priced it at 7% and it is off by default for that reason.

### Focus, and the anchor

**A collapsible closed by a gesture on its own header focuses that header** — which is what a
focusable widget does on a click anyway — and then R08's vanish rule never fires: **0 ring probes
against 405**. Left to the vanish rule, the focus lands on the *next* surviving entry, one section
too far, while the section the user acted on is still on screen one row above.

**R08's 150× cannot be reproduced by an accordion**, and that is structural rather than a correction:
the stamped table saves 235 against 265 probes here, because an accordion's **headers survive every
collapse** — even exclusive mode, removing 204 of 273 ring entries at once, resolves on the second
candidate. R08's number needs a ring range that vanishes with nothing after it.

**`Stash` does not generalise to a region.** C05 could stash a selection because a selection is
positions in an index the component was handed. A region collapsible is handed a *closure*, and the
ids inside it are minted at that closure's own call sites, so the component has nothing to key a
stash on — the caller can, and does, by capturing `Frame::focus`. **`Drop` is the component's answer;
`Stash` belongs to whoever owns the content's identity.**

### Folding differs from a tree in the anchor

A fold set is keyed on a **line number**, not on a node id, and every edit above it moves it. Insert
ten lines at line 24 of a 200 000-line document holding 4 167 closed folds: left alone, **4 166 of
4 167 folds sit on a line that opens no block** — nothing throws, and the screen is plausible with
4 166 folds hiding the wrong lines. Reanchored, **0 of 4 167**, and the reanchor is a
`partition_point` plus a walk: **1.04 µs**, against the 72.83 µs the document edit itself costs.
Splice-always holds here too: **129.42 µs against 2 977.71** to rebuild.

### Inplace needs no prefix sum

One open detail row maps a content row to a data row in two comparisons and a subtraction; C05's
`ytop` is the answer when *many* rows are tall, not when one is. Five thousand open rows through a
sorted store with a running sum costs **49.17 µs against 46.21** — O(log k), still flat in the
million. Scrolling the open row out of the window keeps the **state** and loses the **focus**, which
is ADR 0012 reaching a component rather than a defect.

### The frame

| the accordion, 300×80 | µs | marked | writes | verbs | regions | stops | allocs |
|---|---|---|---|---|---|---|---|
| twelve sections closed | 24.38 | 0 | 23 034 | 230 | 70 | 15 | **0** |
| six open (content 72 = viewport) | 43.71 | 0 | 22 938 | 829 | 274 | 219 | **0** |
| twelve open (content 132) | 45.71 | 0 | 24 340 | 909 | 303 | 248 | **0** |

Twelve open sections cost **1.05×** six open rather than 2×, because a section outside the viewport
is not drawn and therefore declares nothing.

---
## 9. `scroll_area`, `scrollbar` and `sticky`

**Bars are reserved. The rectangle is reduced by the component before it calls the body, and the
reason is damage rather than layout.** It is
[ADR 0029](../../docs/adr/0029-scrollbars-are-reserved-not-overlaid.md). (C13)

An overlay bar is free exactly where the body does not draw under it — **0 cells** — and costs
**3 535 cells re-damaged on every steady frame** where it does, against **0** for the reserved twin
on the same screen with the same content. "Free where nothing is drawn under it" is exact and is not
a property any component can guarantee of its body, which is what makes the rule unconditional.

### The decision is a fixpoint and it cannot oscillate

Reserving is **monotone**: a bar only removes room, and a smaller viewport can only need more bars.
Two monotone booleans have one least fixpoint — checked by feeding each answer back into itself over
**5 475 600 viewport × extent pairs: 0 failures, worst case 3 passes.**

The monotonicity has a precondition, and it is on the **body**: *a smaller viewport may not produce
a smaller extent.* A responsive body that breaks it flips the decision **99 times in 99 frames with
no input**. Computed the way a reader writes it — from last frame's *reduced* rectangle — there is no
oscillation and no non-termination, but **hysteresis**: over content that fits with no bars at all
it keeps both, losing a row and a column permanently on a screen that looks correct. Hence the second
half:

> **Reserved auto-hiding bars require a declared content size.** A measured extent and a hideable
> reserved bar are incompatible, because the measurement is taken inside the rectangle the decision
> produced.

### A band is one construction, and sticky regions are pinning transposed

The same mechanism as C04's pinned columns, and the test is that C04's clip finding reproduces on the
other axis with no change of shape: a body drawn by arithmetic instead of into a view has **identical
writes, identical verbs, identical output and 3 243 cells re-damaged every steady frame.**

> **A band is a rectangle split that shares one of the two offsets and pins the other to zero, and it
> must be a view.** The header shares `x`; the pinned column shares `y`; the footer shares `x`; the
> gutter shares neither.

One construction with an axis argument, not four components — and **one hit entry for all four**,
because a band that were a second scroll area would win the wheel from the body it is a header of
(R17 §5).

### `Σ h` is the extent, and measuring it in rows is a false green

C05's debt, collected. 1M rows, one in eight three cells tall: the extent is **1 250 000**, not
1 000 000. Measured in rows, **time, writes, verbs, marked cells, regions and allocations are all
identical** and the area reaches **row 799 999 of 999 999** — 20% of the content unreachable, on a
screen that looks perfectly healthy. The same unit error reaches the thumb, out by up to **7 cells
of 69**, smoothly and plausibly. **The extent, the offset, the thumb and scroll-into-view are all in
content cells and all come from one place.**

### C21, restated where it belongs

A `scroll_area` costs its **content**; a virtualised `collection` costs its **visible window**. The
wrong pairing at the component level is **7 907 µs at 100 000 rows** — 79 budgets. Every shipped
scrollable of unbounded data is a `collection`.

**Virtualisation is per axis, and so is the watermark's blindness.** Over a body that virtualises
rows and not columns, the watermark reads `(400, 69)` against a `(246, 69)` viewport: the area is
dead downward — **twenty wheel clicks move the offset 0** — and alive sideways.

### The frame

| | |
|---|---|
| steady, 1M rows, 300×80, both axes | **48.83 µs / 0 marked / 33 890 writes / 1 453 verbs / 59 regions / 0 allocs** |
| 1k → 1M | **1.00×**, identical writes and verbs |
| `row_at` over 1M, variable / uniform | 0.0187 / 0.0007 µs, once a frame; the frame does not move |
| a shape change, 349 524 rows removed | 58.8 µs uniform, 394.2 µs variable |
| an offset past the end, on the next frame | clamps 999 931 → 99 931, 196 marked |

### The seam with §8, measured and assigned

A shape change costs 58.8 / 394.2 µs; the offset clamp is **free**, because `max` is recomputed every
frame. **The tail is not**: `[extent, offset + viewport)` is inside the rectangle, and the component
that owns the rectangle must write it (§2). Whoever integrates `collapsible` with a scroll area owns
that line. What is *not* decided is whether a collapsible inside a scroll area may learn its content
height one frame late under the extent shape — §22.

---

## 10. The order, the index and the memo

Six tickets arrived independently at one structure and one rule, and the sixth arrived with it in a
different layer entirely. This section is what they share; it is
[ADR 0030](../../docs/adr/0030-a-memo-key-is-every-input.md) and half of
[ADR 0031](../../docs/adr/0031-a-collection-stores-positions-in-an-order-the-caller-owns.md).

### One structure, five names

`table`'s sort/filter order, `tree`'s flatten index, `textarea`'s wrap index, `collapsible`'s fold
index and `table`'s variable-row-height prefix sum are **one structure**: a caller-owned, O(1)-indexed
materialisation of a display order, built on the edit and spliced rather than rebuilt.

It is forced, not chosen. A virtualised collection needs O(1) access to the *k*-th visible row;
neither a comparator, a predicate, a tree walk nor a wrap rule provides one; so the order must be
**materialised**, and materialising it is proportional to the data — **21 158 µs at a million rows,
211 frame budgets**, if it is done in a frame. Moving it from the component to the caller changes
none of that. **What makes it affordable is R09's memo, on the edit**, and what makes an *edit*
affordable is a splice:

| | splice | rebuild |
|---|---|---|
| `tree`, a collapse (C05) | 157–342 µs, flat in what is removed | 40 393 µs at 52% folded |
| `textarea`, a keystroke at 1 MB (C06) | **34.34 µs** | 3 216 µs |
| folding, an insert at line 24 (C14) | 129.42 µs | 2 977.71 µs |

### The order is the caller's and a component may only ask

Expansion joins sort and filter as caller state, which pays R02's debt without a rule: **no `Id` is
anywhere near it**, because the index is keyed on the caller's node ids — exactly the keys that may
be written to disk. The component holds a one-slot request drained by the caller after the draw,
forced twice over: a `Vec::splice` allocates, and taking `&mut` to the index while the draw holds it
shared is `E0502`. **The edit is three things in one order: splice the index, reconcile the
positions, stamp the revision.**

> **A gesture that changes the shape of a collection is an edit in R09's sense.**

### Everything a collection stores is a position in an order the caller can replace

The selection runs are positions, the `editing` slot is a position, `offset` and `lead` are positions
— and a sort changes no data and no length, so **nothing inside the component can notice**. The frame
after a sort draws a perfectly correct table with the wrong rows selected and the editor open on the
wrong row.

The only thing that makes it noticeable is **a revision on the length**: `Rows { len, rev }`, one
`u64` compared once a frame. Reconciling then costs what the edit costs:

| edit | selection | cost |
|---|---|---|
| a permutation, half a table selected, 1M rows | 1 run, 16 B → **268 023 runs, 4.29 MB** | **22 941 µs** |
| a splice, half a tree selected, 349 524 removed | 1 run → **1 run** | **0.04 µs** |
| any edit, select-all | invariant — `[0, len)` under any permutation of that length | O(1) |
| any edit, the editing slot | one position | 0.29 µs |

So **`Clear` is the default for a permutation** and `Remap` is the caller's opt-in with a stated
bound (the remap is the caller's function, because only the caller has both orders); **`Drop` is the
default for a splice**, `Stash` on request, `Clear` refused (§7).

### A memo's key is every input, and the inputs that are not data are the ones that get forgotten

Six independent arrivals, and each one **recomputes less, runs faster, and is wrong on the rendered
surface**:

| keyed on | forgot | what it costs |
|---|---|---|
| the data (R10) | the theme | a paint from the previous palette |
| the data (C04) | the sort | a re-sorted table answering with the previous order |
| the data (C05) | the expansion set | an expansion edits no application data, so the revision does not move |
| the revision (C06) | the width | **625 rows drawn where 875 are needed; 69 of 80 rows differ** after a 300→120 resize — while recomputing 1 against 2 and running **twice as fast** |
| `(data, tier)` (C09) | the theme's revision | hits at **half the cost**, wrong in **598 characters** |
| the revision (C08) | `(w, h, kind, sx, sy, series, y0, y1)` | **2 330 cells wrong**, recomputing half as often |

**R09's `recomputes` counter points the wrong way**, so the natural detector is useless. The
detectors that work are two: the memoised object **recording the input it was built at** (C06's index
records its width), and the **rendered surface** (C08, C09).

The axis range in a chart is an aggregate *and* an input to the raster's key, so the two memos are a
**chain** and not one memo: a resize invalidates the raster and not the range. (C08)

**R09's `Memo<T>::get` takes one `Revision` and cannot express any of this.** Six crates on this map
hand-roll a compound key, and a value that can be *maintained* has nowhere to say so — held inside
`Memo`, C06's wrap index is **3 543 µs a keystroke at 1 MB against 34 spliced**. The components-side
decision is that the key is spelled out at every call site until the runtime map reopens; the
obligation is R09's and is filed in §22.

### The sixth arrival is a dedupe key, not a memo

A preview pane's question is a **file**, and every natural way to name it names a **position**. R18
made the ask idempotent in a key and did not say what the key is; the one thing in the caller's hand
is the selected row. One re-sort — one line of application code, no keystroke — moves **197 of 200
positions**, and the position-keyed pane is wrong on **100 of 100 frames afterwards** and asks **1**
question in its life. Both arms ask on every frame (103 either way); only the spawn count differs,
because the question still matches, so nothing posts, so nothing wakes, so no frame corrects it.

**This is R18 §1's steady wrong state through a door R18 never opened**: R18's defect was an answer
accepted out of order and its fix is a test on the *answer*; a question that was never asked is
invisible to any such test. The same hole opens with **no user in it** — a directory still being
listed splices batches into a sorted order and the cursor's file changes under it: wrong after **7 of
7 batches**. Identity keying is wrong for exactly **1** frame, the decode's latency.

> **An asynchronous answer carries the identity of the question it answers, and the pane may write
> only on an equality against it** — `shows() == dir.at(pos).ino`, 8 bytes on the payload.

(C16. The general rule this is the sixth instance of is the same one: a key that names a position
instead of a thing is a key that forgot an input.)

---

## 11. `field` — `input` and `textarea` are one component

**Every backward question about text is O(prefix) unless an index already knows the answer, and the
index that answers them is the one the wrapping already needs.** (C06)

### The caret is a `(byte, column)` pair moved in cluster steps

Not a byte offset whose column is computed where it is needed. The column is carried alongside and
moved by the width of the cluster crossed. `display_width(&s[..caret])` is correct on every cluster
and is **2 988 µs a frame at 1 MB against 80.12** — thirty budgets, on a screen where nothing
happened.

A `char` caret is the other failure and it is *faster per step*: **217 steps to cross 166 clusters,
51 of them landing inside one**, and **15 of 217 insertions at those positions change the buffer's
display width by something other than what was typed** — the cluster came apart.

The API consequence is a deletion: **there is no such thing as "put the caret at byte N".** A caret
is placed by a gesture — a click's column, a cluster step, `Home`, or a pair some earlier state
stored — and each of those lands on a boundary by construction. An arbitrary offset does not, and
everything downstream preserves the bad caret faithfully. Undo restores the **pair**: `set_pos(byte,
col)` is **0.0007 µs** against `set_caret(byte)`'s **6 109 µs** on a 1 MB line.

### One component and one flag

`Graphemes` is a forward iterator, so the boundary *before* an offset can only be found by segmenting
forward from one already known — and which one you have decides the complexity. `Left` at the end of
a pasted megabyte is **3 161.68 µs with no index against 0.327 µs with one — 9 655×**. A `textarea`
has that index already; an `input` keeps the **same** index with a different break rule
(`WrapKind::Ruler`: hard breaks every `w` columns, no word breaks, no newlines), whose only job is
that a boundary is never more than one window behind the caret. §5's `Mode`, in a fourth family.

### The selection is one anchored range

A text selection is contiguous by construction — no inventory in the survey has a
ctrl-click-a-cluster gesture — so §5's run list would always be exactly one run. That is not the cost
argument. **The cost argument is the unit**: a run list addresses cluster indices and a buffer
addresses bytes, so every gesture pays a prefix walk. Fifty select-half-and-replace edits on 200 kB:
**30.78 µs and 100 allocations against 428.68 µs and 901.**

### The undo ring holds edits, is bounded twice, coalesces, and reports truncation

An entry is `(at, removed, inserted)` plus the caret pair and anchor the edit started from. Over
1 012 keystrokes the delta history is **40 756 B regardless of document size**; snapshots are
**1 012 575 322 B and 169 773 µs** on a 1 MB document. Coalescing is a **2.4× entry and byte count**
and not a time win; its reason is that undoing a sentence is 414 presses instead of 1 012. The frame
does not pay: ring at cap (256 entries, 25 600 B) **160.67 µs / 0 allocs**, ring emptied **160.71 µs
/ 0 allocs**, same document, same screen.

**The bound is where the silent defect lives.** A ring of 16 entries after 64 edits runs sixteen
steps, returns `true` every time, and the document is **not** back to the original — indistinguishable
from success unless the ring says so. So crossing the bound sets `truncated`, and there are **two**
bounds rather than one, entries *and* bytes, because one large paste is one entry.

### The wrap index

A `Vec<u32>` of visual-row starts plus a sentinel — §7's record with one field instead of four,
because a visual row carries no depth and no flags. Keyed on `(revision, width)` and spliced (§10).
The revision must also not move when the caret does: **200 recomputes at 306.51 µs/key against 1 at
1.587**.

**A greedy word wrap's break depends on text *after* it.** A row is closed at the last space before
it overflows, and that point is past the row's start — so an insertion at an offset *greater* than a
break can move that break. A splice restarted at `row_of(at)` therefore keeps a break the edit
invalidated: it agreed with a rebuild **499 times in 500** and the screen looked right. **Restarting
one row earlier is provably enough**, and the defect was caught by an equality against a rebuild and
by nothing else.

### The frame

**82.4 µs / 0 marked / 19 634 writes / 631 verbs / 79 regions / 0 merges / 0 allocations**, flat at
100 kB and 1 MB. Every number and every gate runs on clusters that are not one code point: a
combining acute, two marks on one base, a ZWJ family, a skin-tone modifier, `U+FE0F`, a
regional-indicator pair, CJK.

### Four gates, none of them visible on the rendered screen

The caret is always on a cluster boundary · the caret's column equals the engine's tables over the
same prefix · a spliced index equals a rebuilt one · the index's recorded width equals the width
being drawn.

---
## 12. `select` and the overlay family

**R07's two-phase protocol carried a whole family without one change to it, and every problem was at
a seam.** The parts are §5's collection, §9's reserved bar, §8's not-drawn rule and R07's request
queue; all four held. (C07)

### The family is three kinds on two axes, not six components and not "modal"

- **Axis A — does it declare anything.** A tooltip and a toast are overlays made of cells and
  nothing else. One that declares *and* covers its anchor takes its own hover: **99 flips in 99
  frames.**
- **Axis B — is its owner guaranteed to be drawing.** The census is over the **request**, so a
  dialog owned by the menu row that opened it lives **3 of 8 frames**.

Dropdown, menu, submenu and context menu are **one** mechanism; tooltip and toast are another;
dialog and command palette are a third, needing a host owner, a barrier and a trap. **Modality is one
`bool` on the request and forces no construction.**

### The rectangle is three parties, and the component never hears what it was granted

The **anchor** is the component's, in its own coordinates, during its own call. The **size** is the
component's too, from a **sizing function** beside it — `CONTEXT.md`'s shape, no draw context. The
**placement** is the runtime's `place()`.

**The size may not come from the drawn extent**: a popup has no frame before the one it opens on, so
its extent there is 0 and stays 0 — granted `(20, 0)` against `(20, 8)`. `CONTEXT.md`'s "one frame
old" is survivable for a scroll area and fatal for an overlay.

Because the owner asked for a size and the runtime answered, **§9's bar decision moves into the
body**. Owner-side, on a screen too short, **1 of 4 rows is unreachable with no bar**. §9's fixpoint
does not arise at all: a popup owns its own viewport, so the gutter is decided once, in **0 passes
against ≤ 3**.

### One owner is one layer

The layer is keyed and censused on the **request**. An overlay that must outlive its trigger takes
`owner: Id` as an **argument** — the same shape R07 forced on `cx.overlay`, for a different reason.
A component with two overlays standing must mint a second id with `with_key` (§4); shared, the two
get **one slot resized, 8 allocations and 564 cells a frame**.

### Dismissal is `Esc`, choosing, the owner ceasing to request, and blur — qualified by a position

`focus_left` is what an outside click already produces, because R05's award sets `focus = winner`
and `winner` is `None` when the press lands on nothing. But **`begin` hands out an optimistic focus
a frame before the body can speak**, so blur is qualified by a **position** the body reports through
one hit entry over the popup's whole rectangle with `Interest::HOVER` only — never by a press. The
alternative, a catcher layer, costs **386 912 layer bytes and swallows a click**, against 2 592 bytes
and two clicks for blur.

On the way out the owner refocuses **itself** — one id it already has, so no id belonging to anybody
else is named, and it is not §8's `Stash`. A modal additionally needs R08's `Trap`, and **without one
six `Tab`s leave it**.

### Hit-testing across layers needs nothing new

Overlay entries append after the base pass and win by draw order. An operator layer (scrim, shadow)
has no cells and declares nothing, so a click on a shadow reaches the widget beneath. The barrier
stops the pointer and only the pointer.

### The popup's state is two structs, one writer each

`SelectState` is the owner's, written only by the owner from its own input and the inbox;
`PopupState` is the body's, borrowed `&'f mut` and written only by the body. §7's goal — nothing has
two writers — is kept, the choice comes home through the inbox, and **`&'f mut` makes "request the
overlay last" a borrow error** rather than a comment. §7's literal `Copy`-only body moves the offset
**0 in 20 wheel clicks**.

### The frame

300×80, 312 chips, two `select`s, a menu bar; minimum of 60 steady frames.

| | µs | marked | regions | stops | content layers | allocs |
|---|---|---|---|---|---|---|
| nothing open | **33.96** | 0 | 317 | 316 | 0 | **0** |
| one select open | 35.29 | 0 | 319 | 317 | 2 | 0 |
| menu + submenu | 35.12 | 0 | 323 | 322 | 4 | 0 |
| modal dialog + scrim | 34.83 | 0 | 319 | 318 | 3 | 0 |
| the popup drawn fill-first | 36.58 | **107** | 319 | 317 | 2 | 0 |

An open dropdown is **+1.33 µs**, a menu with its submenu **+1.17**, a modal **+0.75**, its scrim
**+0.12**. **The scrim's real price is the frame it appears on: 40.50 → 138.04 µs, 25 080 cells, over
the whole budget** — once per opening, and §20 owns it.

### What building it corrected (components 26, ADR 0037)

**The menu delta above is a per-row menu, and §5 forbids one.** The table's *323 / 322* for a menu
with its submenu is **+6/+6** — three rows twice, each a target of its own. §5 collapses a menu into a
`Mode` of `collection`, and a collection declares **one** hit entry however many rows it has, so each
level of the shipped menu is a dropdown's delta: one entry for its rows and one for its blur position.
**+4/+2.** It is the same subtraction this section already prices for a dropdown's own option list;
reproducing the six would mean declaring per row on purpose. The other four rows of the table
reproduce exactly. `allocations` is `n + 1` for `n` bodies standing (ADR 0034 deleted the arena) and
`content layers` is 1 / 2 / 1 rather than 2 / 4 / 3, because every request in the prototype carried a
shadow layer `OverlayOpts` has no field for.

**The blur's position is `Response::local` and not `Response::hovered`.** This section says *a
position* and the distinction is load-bearing: `hovered` is `hover_guess`, resolved from the
**previous** frame's index, so on the frame the layer is placed — the frame the optimistic focus
arrives on — it reads false and the popup dismisses itself. `local` is this frame's containment.

**A dialog has no blur position at all**, which is narrower than Axis A. A dialog declares its buttons;
what it has no use for is an entry over its own rectangle, because its barrier already withholds the
pointer from everything outside it and a modal is not dismissed by blur.

**A nested overlay's state arrives as an `Option` the body takes.** An overlay body is `FnMut` and so
cannot move a capture, and a nested request has to move a `&'f mut` into the inner closure; a reborrow
is shorter than `'f` and fails the `+ 'f` bound. `Option::take` mutates the capture instead, which is
the one spelling that compiles.

**A component cannot share an overlay owner without sharing its own identity.** A `select`'s id *is*
its overlay's owner, so two `select`s under one id are one layer, one widget and **3 regions against
6** — the second request inert *and* the second widget's hit entry merged away. There is no spelling
that shares the layer and not the widget.

**The scan that decides whether this section's scene has a subject had the wrong needle**, and §1 said
so before the component existed: it looked for `pub fn select(`, and §1 records that *the fifth
component opens an overlay, and `'f` costs it two annotations*. The shipped signature is
`pub fn select<'f>(`.

---

## 13. `chart` and `plot`

**A plot cannot satisfy the data-volume invariant the way a collection does, so it splits the cost in
two.** `list`, `table` and `tree` satisfy *frame cost is proportional to visible cells* by **never
folding**; a plot must fold, because which of a million points land in the rectangle is not knowable
without looking. (C08)

> **The frame costs the rectangle. The edit costs the data. There is no third option, and the memo is
> the only thing standing between them.**

What is memoised is the **raster** — one sub-cell bitmask per cell, `w × h` bytes — because that is
the smallest object whose *size* is the rectangle and whose *contents* are the data. A 60×20 raster
is **2 400 B at every data volume**. `Raster::build` is **4 881 µs over 1M points against 2.64 ns on
a hit — 1 849 000×.**

**The rasteriser is a union, and a union is idempotent**, so a column's extrema survive by
construction and **downsampling is not a technique this component contains**. Three false greens
prove the point from the other side: culling to the visible index window (**5 546 cells wrong, 5 500
blanked, 1.39× faster, writes unmoved at 21 872**), an axis range from a stride sample (**3 431 cells
over 78 rows**, domain 30.00–77.05 against 30.00–103.07, and *slower*), and every *n*-th point
instead of the union (**2 953 cells, 2 544 blanked**).

### Verbs are not flat and must not be gated as flat

One correction to how §§5–7 stated flatness. A run ends where a cell's owner changes, so a plot's
verb count tracks the **picture**: 2 551 at 1k, 3 772 at 100k, **2 472 at 1M** — not monotone in *n*.
The gate is the bound the invariant actually claims: **`verbs ≤ writes`**, never verb equality across
sizes.

### The third repertoire rung is `plot`'s, not `chart`'s

| construction | Ascii | Unicode | Extended |
|---|---|---|---|
| `chart` — bars; a cell is a **prefix** | 1×1, 2 states | 1×8, 9 states | 1×8, 9 states |
| `plot` — marks; a cell is a **set** | 1×1, 2 states | 2×2, 16 states | 2×4, 256 states |

On the rendered surface, Unicode against Extended: **0 cells differ in the chart pane, 882 in the
plot pane.** So a bar chart at Extended is byte-identical to one at Unicode, and the third rung is
earned by `plot` alone, buying exactly one bit of vertical resolution. Ascii against Unicode is
**7 276 cells over 80 of 80 rows** — ADR 0009 literally, a different construction rather than the
same one with worse glyphs.

This corrects R10's `theme::subrows`, which puts the half blocks at Unicode and the eighth blocks at
Extended: **they are the same Unicode block, U+2580…U+2588**, and `CONTEXT.md` defines the middle
rung as *Unicode with box drawing and block elements*. An operator who promises block elements has
promised all of them. Against R10's table the bar chart is **4 706 cells and 77 rows wrong at
Unicode and 0 at Extended** — wrong only where nobody looks. The freeze does not wait on this:
§17's `constructions` column is derived from `CONTEXT.md`, so `chart` is 2 and `plot` is 3 whatever
the runtime's table says.

**The ladder is not monotone in what it can express.** A braille cell has **one `Paint` for all eight
dots**; a quadrant carries a foreground *and* a background. The third rung buys resolution and pays
in series colour, on the **25** cells of this screen where two series share one. The per-cell quadrant
fallback that would recover it is named and not built (§22).

**Why this can never be a `Glyph`**: the same cell is one of 2, 9, 16 or 256 characters, keyed on a
*bitmask*, and the number of samples asked of the data changes with the rung. §16 owns the lookup;
the branch is two `match`es and a 16-entry array in the component crate.

### Axes: §9's loop in a second place, and here it oscillates

`gutter → plotting width → what a live series shows → the auto-scaled range → the tick values → the
widest label → gutter`. §9's precondition becomes *a narrower plotting area may not produce a wider
label*, and it is **false for ordinary data**: a 1/2/5×10^k step means fewer ticks are not a subset
of more ticks (`0 2.5 5 7.5 10` against `0 5 10`), and dropping older, larger samples can leave
`-0.05` where `900` was.

Over **175 712 viewport × dataset pairs**: the fixpoint converges on 175 248 in ≤ 4 passes and
**oscillates on 464**. The hysteresis form never loops and settles on a **non-fixed-point gutter on
all 464** — §9's hysteresis exactly. Computing the gutter from the **whole domain** is **175 712
pairs, 0 oscillations, always 1 pass.**

> **A layout loop is worth solving only when both ends are genuinely layout; when one end is data,
> decouple it.**

§9's could not be decoupled, §12's never arose, this one can. A caller who insists on a
window-scaled axis must **declare the range** — the same shape of answer as reserved bars.

### Colour

Series colours are the **one legitimate use of `Theme::custom`**: the crate names them in one
six-entry table and contains zero `Style` literals. At C16, `roles_differ_on_wire(Danger, Warn)` and
`(Warn, Ok)` are both false, so role-derived series arrive as one colour — **5 594 cells differ by
style and 0 by cluster**, the signature of a colour-only distinction dying. A threshold carried by a
paint alone against paint-plus-`HLine` at C16 is **215 cells over 1 row**: §16's *carried on both
axes*, in the place a chart meets it.

### The frame

**204 µs / 0 marked / 21 872 writes / 2 regions / 0 allocations, identical at 1k, 100k and 1M.** At
60×20: **13.8 µs / 1 136 writes**, likewise flat. 204 µs is over the frame budget while damaging
nothing — §20.

---

## 14. Media

**The survey drew the line between the chrome and the picture. Measured, it falls in three places at
once, and the graphics decision is only one of them.** (C15)

### A picture's ladder is a colour ladder, and §13's does not apply

A plot puts one **bit** per sub-cell and spells the bitmask with a glyph; a picture puts a **colour**
in every sub-cell, and a cell carries exactly two colours whatever glyph it holds. So braille's 2×4
buys a picture **nothing**: two sub-rows wherever the half blocks exist, one everywhere else, and
**`Extended == Unicode`**, gated.

Two consequences the survey's ⚠️ hides:

- **A picture at `GlyphSet::Ascii` is still a picture** — a space with a background colour, full
  colour depth, one sub-row.
- **The axis with a floor of nothing is the colour axis.** Every other component degrades to a worse
  drawing of itself; a picture with no colour degrades to a *description* of itself. **50.7% of its
  horizontal distinctions are gone at sixteen colours**, with nothing to re-pair them with.

Three constructions, so §17's `constructions: 1..=3` holds — reached along the other axis.

### A picture is the first caller whose every cell is outside the theme

**24 000 `Theme::custom` calls a frame, 24 000 verbs, and no `fill` available at any size** — a
`fill` takes one `Paint` for a rectangle and a picture's rectangle has as many paints as it has
cells. **261–357 µs of draw against a 100 µs budget, zero allocations.** The contrast that makes the
media family legible as a count: a **waveform spends zero** customs, because its colours are roles;
a **QR spends four** — not because it has too many distinctions but because it has exactly two and
they must be *those* two, which no theme can promise.

### The video number, corrected by 3.4–3.7×

~10 bytes a cell is what a **style run** costs, and a picture is the input on which run coalescing
and the SGR delta both stop working. Measured through engine ticket 08's own emit loop with
`TermModel` parsing the bytes back:

| tier | B/cell | a 300×80 frame | at 30 fps |
|---|---|---|---|
| truecolor | **37.5** | 900 KB | **27.0 MB/s** |
| 256 | — | — | 15.9 MB/s |
| 16 | — | — | 8.4 MB/s |

The survey's ~7 MB/s is roughly right at sixteen colours and nowhere else. **B/cell barely moves with
the source** — 34.2 for a smooth gradient against 37.5 for a noisy photograph, 9% apart, gated —
because coalescing is broken by *any* picture; what the source decides is only how many cells change
between frames, and for video that is all of them. **The refusal therefore rests on the lower bound
of 24.6 MB/s**, the smoothest frame a camera could produce, and is better founded than the argument
written for it.

**But a static picture emits 900 134 bytes and then 0, 0, 0.** So the two families in the graphics
question do not fail for one reason: **a preview pane's picture fails for fidelity, a video's for
bandwidth**, and only the second is a bandwidth question at all. Gated trap: a picture translated by
a whole cell row is a **scroll**, and the engine's pre-pass emits **5 885 bytes** for it — a
fiftieth of the truth, gated so the comfortable number cannot come back.

### The background image is right about the engine and looking at the wrong side of the seam

No engine change is needed: `LayerKind::Content { opaque }` and the `EMPTY`-skipping blit already
exist. **`rt::overlay::LayerHost` cannot express it, in three places** — there is no slot below the
base pass, `blit_rows` is unconditionally opaque, and a surface cannot be cleared to transparent.

Drawn where a component can actually put it — first in the base pass — the wallpaper and the UI
overwrite each other's cells every frame: a **still** screen re-damages **24 000 cells against
1 076** and costs **407 µs against 14**. This is §2's rule in a form **reordering cannot fix**, since
a wallpaper must be drawn first by definition. Built as the survey described it, on the engine's own
`composite`, the still-screen damage is the application's own, unchanged, and the frame is **13.7×
cheaper**.

And the number the survey's ✅ is really a claim about: an application shows **20 093 of 24 000 cells
(83.7%)** of its wallpaper with every panel interior open, and **0** once each panel fills its
interior. **A wallpaper is not a feature a component provides; it is a discipline the whole screen
keeps.**

### The chrome is six parts of ten, and drag capture works

**Drag capture is measured here for the first time on either map** and it needs nothing added to the
runtime: press jumps to where it landed (`20/299 = 0.0669`), the move carries it
(`60/299 = 0.2007`), the release ends the scrub and moves nothing — computed from `Response::local`
alone, no press origin, no fifth cross-frame fact. **`slider` leaves Tier 3** (§17). What stays open
for the other two call sites is §5's missing modifier byte, not the grab. The playhead is the other
Tier 3 mechanism: a component that owns a clock.

**QR, barcode, waveform, spectrum and VU meter are simply true**, and one of them for a reason the
survey does not give: **a QR module must be square and a cell is not**, so the half block makes the
symbol *correct* rather than prettier. One module per cell is a third kind of degradation the matrix
has no column for — **not worse, invalid** (aspect 0.97 against 0.50), gated, with a readback of the
drawn cells against the module matrix because `▀` is the upper half and the pairing is one character
from inverted. The audio half adds no mechanism and no construction: §13's 1 / 8 / 8 ladder verbatim,
zero customs, zero allocations.

---

## 15. The file preview pane, and F12

**The five pieces the survey lists all work, and every defect is at a seam between two of them.** The
identity half is §10; this section is the rest. (C16)

### The offset belongs to neither side

Four spellings, four different defects, on a 4 000-row file, an 800-row file and a 74-row viewport:

| the offset is | what happens |
|---|---|
| unclamped | **the body draws nothing at all** — 1 650 writes against 4 166, because a landing *is* a shrink, from another thread for the first time |
| clamped and kept | the new file opens at **row 726**, its last page |
| reset on the **request** | scrolls **the previous file, still on screen**, to its top for the length of the decode |
| reset on the **landing** | right on arrival, forgets on return — the trade rather than a defect |
| a per-file map | right in both directions, and **nothing releases it**: 1M entries, **14 857 142 bytes** against one slot's 4, because R02's sweep releases what an `Id` stopped drawing and **a file is not an `Id`** |

Reset on the landing is what ships; the per-file map is refused on the release argument.

### §9's precondition is a field of the answer

A declared content size is a property of the *file*, so while the next file decodes the pane's extent
is the previous file's — **4 000 where 800 is right**. That makes a preview pane a **positive case for
bars-reserved** rather than a new question for it.

### Requirement 9, as a count and not a microsecond

**0 decode units on the app thread against 5 076** for the same decode called from the view, with the
answer still arriving. R18 §7's unenforced ordering reproduces in a screen with a status row on each
side of the pane: **20 torn frames of 20 selections** taken inside the draw against **0** taken at the
top of the view.

**R02's sweep is refused twice, over two different things and for two different reasons.** A pending
job registry-swept costs **10 spawns and 10 decodes** over ten tab switches against **1 and 1** with
the slot in application state; a **derived value** swept the same way costs **10 full folds against
1**. A job's lifetime is the *question's*, a memo's is the *data's*, and neither is the widget's.

**R09's `Edit` bumps on drop, and a landing that did not happen is still a drop.** Written without
the branch, the pane's revision advances every frame and the highlighter re-folds the whole file —
**119 computations against 20 landings** — on two screens that are cell-identical.

### `Worker` is `Send`, and G10 named the wrong auto trait

| | `Send` | `Sync` |
|---|---|---|
| `Cell<u64>`, `RefCell<u64>` | **true** | false |
| `Arc<dyn Spawner>` | false | false |
| `work::Task<T>` | false | false |
| `work::Worker<T>` — what a pane holds | **true** | false |

`Cell` and `RefCell` are `Send`; what they remove is `Sync`. What makes a `Task` `!Send` is its
`Arc<dyn Spawner>`, held for an unrelated reason, and `Spawner` carries no `Send` bound — add one and
the case flips with nothing about the design having moved. **The property that is true of both, that
`Cell`/`RefCell` genuinely buy, and that the design needs, is `!Sync`**: the app thread owns the
staleness arithmetic, so a job may never hold a *reference* to the task or worker that spawned it.
G10 is rewritten on `!Sync` over both types and **G10b** added beside it — a compiling case asserting
that `Cell`, `RefCell` and `Worker` *are* `Send`.

### The picture, and what "free once static" does not survive

The preview body is **198×74 = 14 652 cells, 61.0% of the screen** — one `Theme::custom` per cell of
the *pane*, **0 marked on a steady frame**, the whole pane on every landing frame, **238 µs against a
100 µs budget**. A pane is not a cheap version of §14's screen; it is most of one.

Twenty selections through a directory of photographs at 37.5 B/cell: **1 picture and 536 KB** when
the key repeats faster than the decode, **20 pictures and 10.7 MB — 18.3 MB/s at a 30 ms repeat**
when it does not. That is **R18 §5's crossover read from the wire**: the regime in which cancellation
saves nothing is exactly the regime in which every picture is drawn, so the bytes and the wasted CPU
peak together. A preview pane is **not blocked on bandwidth and not free either — a burst per landing
bounded by a key repeat**.

### The frame

Steady at 1 000, 100 000 and 1 000 000 entries: **3 557 writes / 197 verbs / 0 marked / 0 allocs,
identical**, with a worker in the frame and a landing in its history. 1M with a photograph selected:
**17 782 writes / 14 833 verbs / 14 652 customs / 0 marked / 0 allocs / 237 µs**.

**`file_preview_pane` is built with 23 gates, and `file_picker` is `collection` + `overlay` + this
pane with no mechanism in it that is new** — which is what moves both out of §17's at-risk tier.

---

### The picker's popup has a keyboard, and the pane deliberately does not

Architecture issue 23, resolved 2026-09-05. The overlay family's two owners were **one drawing and
two keyboards** for eight tickets: `select`'s popup takes the focus from its owner and reads `Enter`
and `Esc` through a `Refusal`, and `file_picker`'s did neither — so an **open picker could only be
used with a mouse**, with no arrows, no `Home`/`End`, no type-ahead and no way to choose a file. It
rendered perfectly, which is why every gate here and in `crate::preview` was green on it: all of them
drive the picker with a pointer or assert about the pane.

Three decisions, and none of them is `select`'s answer copied:

1. **The keyboard goes to the list.** The pane is not a candidate — its document is a function of the
   list's cursor, so it has nothing of its own to answer — and `body.inside` is already defined as
   *the list has the focus*, which the owner's blur clause reads. Seating anywhere else would leave
   `inside` false while the body held the keyboard and shut the popup under the user.
2. **`Enter` answers the cursor's file; `Esc` cancels and answers nothing.** `select`'s `Esc` answers
   the *previously chosen* index, which it can because a `SelectState` always has one. A
   `PickerState::chosen` is an `Option<u64>` that starts empty, so *answer what was already there*
   would be `None` — indistinguishable from *nothing decided yet*, which is the state the owner uses
   to decide whether to close. Cancelling is therefore its own slot on the body, and leaving `chosen`
   untouched **is** the restore.
3. **The pane's scroll keys are not the picker's, and the pane has no keyboard at all.** Its document
   is replaced by the next arrow press, so a key that scrolled it would be operating on something
   about to vanish; giving it one needs a focus switch inside a popup where §12 forbids `Tab` from
   leaving, which is a new mechanism in the overlay family rather than a picker decision; and
   `PageUp`/`PageDown` are §5's and belong to the list, so splitting them would make `file_picker`
   the only component where they mean something else. The pane is scrolled with the pointer, and this
   is a limitation stated rather than a gap.

**Declaring the keyboard found the pointer half nobody had named.** The picker answered a press on
`Response::clicked` where `select` answers `press_began` — a row selects on the press, so a
release-driven reader arrives a frame late and a drive that ends at the press never reaches it at
all. `Shift+Click` and `Ctrl+Shift+Click` were 2 of the 27, and no keyboard question would have found
them: it took declaring the other 25 for the sweep to disagree in a way that pointed here.

The two owners now share **one `&[Bind]`** — `OVERLAY_OWNER` is gone — and `PICKER_IS_MISSING` is
kept at **0**, because the gate that reads it is what would catch them coming apart again. The gate
is not a screen: `dropped::tests::an_open_picker_answers_the_keyboard_and_the_arm_that_shipped_answered_none_of_it`
plays the shipped body against `files::defective::a_popup_with_no_keyboard`, which is the same
function with one field changed.

**And the body owed a frame it was not asking for.** An answer goes home through the inbox and the
owner reads it at the top of the *next* frame, so `picker_body` calls `Ctx::request_frame` when either
slot fills — `popup_body`'s own finding, which this body did not have. It was latent on the pointer
path because a pointer usually moves again; on `Enter` it would have been certain.

## 16. Theme, glyphs and degradation

R10 closed the colour half of the no-literals rule with a type; **the glyph half was never closed,
and the count is the asymmetry — `GlyphSet::` appears 24 times across four component crates and
`ColorTier` 0 times.** The colour axis is resolved before a component runs and nobody names it; the
glyph axis was not, so four component crates each shipped a private `mod missing` — six glyph
literals and a `match` on `GlyphSet`, **byte-identical in all four**. By the time C11 counted, it was
**nine crates of twelve**. This section is
[ADR 0032](../../docs/adr/0032-a-distinction-survives-only-if-it-is-carried-on-both-axes.md). (C09,
C11)

### What a `Glyph` is, stated as two counts

> **A `Glyph` is a lookup with a spelling at every level, every spelling exactly one cell, and no
> spelling blank.** Anything failing either rule is not a `Glyph` — it is a **branch**.

Both are counts over `Glyph::ALL × 3`, and both are **0**. Absence is not representable, which is
what makes a blank fallback a defect rather than a policy: ADR 0009 says a component that cannot
express itself **builds something different**, and drawing nothing is not that.

**The table goes from 7 entries to 20**, closing every recorded debt: `ArrowUp`, `ArrowDown`,
`ArrowLeft`, `ArrowRight` (§9's steppers and §7's disclosure markers are **one family**, and entering
them twice under two names would be a collapse the pair gate catches); the four box corners and
`Ellipsis` (§3); the four tees and `Cross` (§6). `tree` needs no new entry — its chevron pair *is*
`ArrowDown`/`ArrowRight` and its indent guides are `VLine`, `TeeLeft`, `BottomLeft`.

> **The last clause was never true of the shipped crate, and the sentence stays with this note
> rather than being rewritten** (ADR 0033; components architecture 20, 2026-09-04). `tree` draws no
> indent guide and §7 forbids every route to a correct one, so `INVENTORY` lost the three entries
> and the freeze's `glyphs` column is *what a component draws*. This paragraph's other allocation —
> the four tees and `Cross` to §6, meaning `table` — is false in the same way and is still open;
> `crate::glyphs::UNDRAWN` holds it and components architecture 25 owns it. See §22.

**The lookup table has two rows, not three.** Entries whose Unicode and Extended spellings differ:
**0**. That is not an omission — everything that would distinguish the top two rungs changes the
construction and the number of samples asked of the data, so it is a branch. §13 earns the third rung
on its own.

### Degradation is measured on signals, and resolved rather than branched

A **signal** is `(Option<Glyph>, Role)` — what actually reaches a cell.

> **A distinction survives the whole matrix iff it is carried on both axes.**

The theme narrows each declared `Distinction` to **one bit at construction**, from the palette as it
arrives at the terminal and the repertoire as it was declared; a component branches on the bool and
names neither `GlyphSet` nor `ColorTier`. This generalises R10 rather than sitting beside it —
`shows(Hover)` **is** R10's `hover_distinct`, asserted at all nine cells — and resolved against
branched render **cell-identical screens** at all nine, so the choice is cost and vocabulary, never
correctness.

| | µs a frame | per decision |
|---|---|---|
| resolved | **93.29** | `shows` **0.524 ns** |
| the component deciding per draw | 95.25 (+1.96, 2.1%) | `signals_differ` 8.22 ns — 15.7× |

`Theme::with_glyphs().resolve()` including all nine distinctions is **291 ns, once per theme, off the
frame path**; `theme.glyph` is 0.661 ns. **The frame delta is small and is not the argument.** The
argument is that a component which branches must *name the axis*, and naming it is what produced 24
occurrences and nine divergent copies of one table.

**No probe exists and none may**: a repertoire is declared, never detected (ADR 0010), so
`Theme::with_glyphs` is the operator's promise arriving rather than a capability question. Nothing
goes near crossterm and no public signature moves (ADR 0001).

### The matrix is a count, not nine screenshots

| glyphs | tier | roles / 78 | glyphs / 190 | signals / 37 128 | distinctions lost / 10 |
|---|---|---|---|---|---|
| Extended | True / C256 / C16 | 1 / 2 / 13 | 0 / 0 / 0 | 21 / 42 / 273 | 0 / 1 / 2 |
| Unicode | True / C256 / C16 | 1 / 2 / 13 | 0 / 0 / 0 | 21 / 42 / 273 | 0 / 1 / 2 |
| Ascii | True / C256 / C16 | 1 / 2 / 13 | **36 / 36 / 36** | 561 / 654 / **1 677** | 0 / 1 / 2 |

R10's `1 / 2 / 13 of 78` reproduces through the generalised machinery, so the baseline is visibly the
same one. The glyph axis collapses in one place and cleanly — **36 of 190 at ASCII**, the nine
box-drawing entries all spelling `+`.

**The number a component acts on is the last column, and it is far smaller than the pair count** — 2
of 9 against 13 of 78 — because most role pairs are never asked to be told apart. What the pair
count under-reports is *which*, and the second survivor is new: **`Danger`, `Warn` and `Ok` all
quantise to bright white at sixteen colours, so a traffic light is monochrome at C16.** Everything
glyph-bearing — stepper, disclosure, guide, separator, thumb — survives all nine cells, because the
glyph carries it. That is the argument for filling the table.

### Three defects with no signature, all found on the rendered surface

**The blank fallback moves every counter the wrong way.**

| | µs | marked | writes | allocs |
|---|---|---|---|---|
| Extended · complete table | 95.08 | 0 | 23 402 | 0 |
| Extended · what four crates had | **94.79** | 0 | **23 325** | 0 |
| ASCII · complete table | 92.33 | 0 | 23 402 | 0 |
| ASCII · what four crates had | 92.38 | 0 | **23 325** | 0 |

No slower, **77 fewer cells written**, same damage, same allocations. An absent entry returns `""`, a
verb handed an empty run writes nothing and marks nothing, so **absence is cheaper than presence at
every level of the stack and the defect has no signature.** The only detector is the surface: 135
cells and 77 of 80 rows at Extended, **545 cells and 80 of 80 rows** at ASCII.

**A collapse inside one family is fine; across families it is C09's defect.** The shadow table's
ASCII ellipsis was `>`, which is exactly an ASCII `ArrowRight`, so **468 truncated labels ended in
the collapsed-node marker** — and `tree` draws both. Corrected, the ASCII ellipsis is `~`. The gate
this implies is **cross-family collapse == 0**, not the pairwise version §17 first wrote: run
literally, the pairwise form fires on `panel`, `table` and everything else with a border, because the
four box corners are all `+` at ASCII **on purpose**. A corner collapsing onto a corner loses
nothing.

**A three-cell `...` where one cell was reserved moves 468 cells on the surface over 78 rows, with
`writes / verbs / marked` identical at 23 402 / 2 163 / 0 either way.** The two extra cells fall
outside the narrowed context and the clip discards them silently, so the reader gets a hard cut that
looks deliberate; outside a narrowed context the same three overrun the neighbour. Enforceable only
on the table, which is the one-cell rule's whole reason.

### Two rules that leave this section

**R20's memo rule gains one word**: *a memo carries the theme in its key iff its value is made of
paints* → **or glyphs**, and the key is the theme's own `Revision`, not the axes. Keyed `(data,
tier)` — the plausible fix, which survives a palette swap — a repertoire swap **hits at half the cost
and is wrong in 598 characters** (§10).

**`Theme::glyph` returning `&'static str` cannot be closed by a type the way `Paint` was**, because
text is legitimately a component's own content and there is nothing to forbid. What replaces the type
is a count: **`GlyphSet::` occurrences in `vitui-components` == 0**, and no private `mod missing`
(§21).

---

## 17. The v1 inventory

**Twenty-nine components in three tiers, and the tier is part of the freeze.** (C10, C11)

### The inherited count was wrong twice

The survey's §3 lists are 28 and 5, **but they intersect** — `scrollbar` is in both — so the union is
**32, not 33**: a sum presented as a union, repeated in the survey, the map's Notes, the requirements
(C2) and the ticket. And the "present in nine or more of fourteen inventories" rule, re-run against
the only population that can carry it, with a deliberately generous synonym set so the count is an
*upper* bound, puts **21 of the 28 below nine** — `tooltip` 2, `form` 2, `scrollbar` 2, `sparkline`
3, `pagination` 3, `file picker` 4. Three of the fourteen carry no component list at all by the
survey's own text: Material names a taxonomy, ECharts names series types, Qt is written as a delta
over JavaFX.

**The set is not shown to be wrong; its stated justification is not reproducible.** It survives as
product intent, and **the freeze is re-founded on what was built.** Requirement C2's "**33**" and its
check are superseded by this section.

### The freeze

| tier | n | components |
|---|---|---|
| **1 — built, each the subject of a resolved ticket** | 16 | `text` `panel` `chip` `button` `field` `collection` `table` `tree` `select` `overlay` `scroll_area` `scrollbar` `sticky` `collapsible` `chart` `plot` |
| **2 — composed of proved mechanisms** | 9 | `checkbox` `radio` `switch` `meter` `sparkline` `rule` `status_bar` `pagination` `form` |
| **3 — at risk, each naming an unmeasured mechanism and an owner** | 4 | `slider` `spinner` `file_picker` `file_preview_pane` |

**Two of the four have since moved and the table is where that gets recorded.** `slider`'s mechanism
was **drag capture**, and §14 built it — `slider` leaves Tier 3. `file_picker` and
`file_preview_pane` name C16 as their owner, and §15 built both with 23 gates. What remained
genuinely at risk was **`spinner`**, whose mechanism is *a component that owns a clock*, prototyped
nowhere on this map (§22) — **and components ticket 42 prototyped it, so no row of this tier is at
risk any more.** The answer is *a component may own an anchor and may not own a clock*, and it also
corrects this section's own `constructions` for that row from 1 to **2**: the ladder is `4 / 10 / 10`
frames at Ascii / Unicode / Extended, so the frame count moves with the rung and §16's *one cell,
never blank* — both true of the measured ladder — decides nothing.

That disagreement is why the freeze carries a **`built` column beside the tier**, gated against it:
C10 froze two rows at Tier 3 and C16 resolved afterwards having built them, and prose had nowhere for
a later ticket to write.

**Six collapses, five measured and one a judgement**: accordion → `collapsible` (§8), tabs →
`collection` and menu → `collection` (§5), textarea → `field` (§11), tooltip → `overlay` (§12), and
**gauge → `meter`, which is a judgement rather than a measurement** — the only one in the freeze that
is, and it is marked as such.

**Three additions no count produces**, and they are what make this a decision rather than an
arithmetic: **`chip`** (built three times, the subject of three defects), **`plot`** (§13: the chart
pane is 0 cells different at Extended, the plot pane differs in 882) and **`sticky`** (§9). None
appears in any inventory's core list.

### The inventory is a value, not a paragraph

This is the load-bearing half of the section and it is
[ADR 0033](../../docs/adr/0033-the-inventory-is-a-value-and-every-obligation-is-a-query-over-it.md).

> Every obligation this map has stated as a sentence has been broken by someone who had read it.

`CONTEXT.md` forbids the unconditional scroll-into-view and four prototypes did it; C02 decided every
helper writes a partition and four collections left a tail outside it; C01's author wrote a 15-cell
double write straight after writing the rule against it; C10 wrote *the inventory is a value, not a
paragraph* and then resolved as a research document, so its five obligations had nothing to be
enumerated against until C11 wrote the value.

```rust
pub struct Component {
    pub id: &'static str,
    pub tier: Tier,
    pub built: bool,
    pub layer: Layer,              // L0..L5
    pub families: &'static [Family],
    pub glyphs: &'static [Glyph],  // the per-component demand set
    pub constructions: u8,         // 1..=3
    pub can_shrink: bool,          // the hostile axes — C10's O5
    pub owns_offset: bool,
    pub scrolled: bool,
    pub narrow: bool,
}

pub const INVENTORY: &[Component];
```

**The obligations are queries over it**, and each is a gate rather than a sentence:

| | obligation | gate |
|---|---|---|
| O1 | a rustdoc page with a compiled example | compile outcome — `deny(missing_docs)` + `cargo test --doc`; and **components with 0 doc-tests == 0** |
| O2 | a panel in the gallery binary | **two** equalities: nothing shown may be absent from the freeze, and everything `built` must have a panel |
| O3 | one golden screen **per construction**, not per matrix cell | count `goldens == constructions`, **and** an equality: screens declared identical must be identical |
| O4 | a declared keyboard contract, rendered as help | equality (documented == R12-registered), plus §21's walkthrough and *a chord types nothing* |
| O5 | **a scene per hostile axis** | count — scenes ≥ 1 for every component whose axis flag is set |
| O6 | **sixty hertz at a million inputs**, for every row that takes a data volume | a growth relation **and** a per-input ceiling, both counts (ADR 0049) |
| O7 | **an application outside this crate imports it**, in `crates/vitui-apps/examples/` | **two** equalities, the shape O2 has: nothing an application exercises may be absent from the freeze, and everything this crate declares must be exercised by one |

**O6 is stated after this map closed** (components ticket 44) and it is here because it is a query
over the same value. Its population is the one that is **derived** rather than written out — the
`Layer::L2` column plus the rows whose memo is keyed on a data revision — and the derivation answers
**seven** where the ticket that asked for it named six: `sparkline` folds a million points through
`chart`'s own `Raster`. What it closes is a hole this map states in one sentence: *this crate can
prove a frame's output is flat in the data volume and it cannot prove its work is.* `chart`'s
rasteriser painted the whole column prefix for every point — **952.61 ms against 2.72** — and every
gate in the workspace was green on it, because all of them count output and the cost was `O(subh)`
inside a visit `touched` counts as one.

**O5 is worth more than the other four together.** The four axes are `scrolled`, `shrunk`, `wheeled`
and `narrow`, and **each was established by a defect that passed every gate then in force and looked
healthier**: the inverted scroll drew nothing at 12.21 µs against 62.96 and was found three times
independently; the stale tail is 71 of 80 rows with the defective build 2.3× faster marking 226× less;
twenty wheel clicks moved the offset 0 against 16; §13's overlap is green at 300×80 and red at 60×20.
Three of the four were caught only by an equality against a reference render. **C11 can write a
perfect gate and still not know which twenty-nine components to run it against — an inventory is what
makes a gate enumerable**, and that is why the axes are columns.

**O1 and O2 both, and neither substitutes**: O1 catches an API that cannot be called from outside the
crate (C01's actual question), O2 catches an inventory that has drifted from what ships. Neither
catches a wrong cell — that is O3 and O5. **And none of the five catches a cell drawn too many
times**, which is O6.

**O6 is two numbers and neither half would do on its own**, measured rather than argued. A relation
alone reads the defect that shipped as healthy, because `O(subh)` a point is linear with `subh` in
front of it — 10.26 and 10.02 a decade, the shipped fold's own figures. A ceiling alone is met by any
constant chosen large enough, which is §21's first refinement by name; under a ceiling raised until
it admits a quadratic arm the relation still refuses it at 94.47 a decade. **Both are counts and
neither is a clock**: a step count is the same number on every machine, and 16.7 ms lives in
`examples/volume_numbers.rs` as a denominator and never as a pass mark.

**O7 is the seventh and it arrived the same way**, after this section froze and on the implementation
backlog rather than as an architecture issue, for O6's stated reason: the instrument is buildable
without reopening anything. It is two equalities because O2's shape is the one it needs — a component
an application exercises but the freeze does not declare, and a component the freeze declares that no
application has ever imported, are two different drifts and neither implies the other. So the seven
obligations are **nine queries**, which is the number `crate::obligations` enumerates.

**And O6 is one obligation and not two** (architecture issue 19, resolved 2026-09-05). The issue asks
whether *the frame is flat in `n`* and *the edit is linear in `n` with a stated constant* are one rule
or two, and the answer is that they are two **gates joined by an `iff`** rather than two obligations.
A covered row's ceiling has `per_input == 0.0` exactly when it is `Layer::L2` — that is the column's
own sentence written as arithmetic — and the join is read in **both** directions, so a virtualised row
cannot start folding unnoticed and a folding row cannot claim its fold is free. Two obligations over
two populations cannot state a rule that binds one population against the other, which is what makes
the merge the stronger arrangement rather than the cheaper one.

The escape the issue fears is real and is closed somewhere else. A gate shaped *the frame is under
16.7 ms at a million* **can** be satisfied by folding less often — a coarser memo key, a debounce, a
lower rate, each of them a defect this map already has a name for — and that is the gate the issue
argues against. O6 as built is not it. Both of its numbers are **step counts measured over the fold at
a fixed `n`**, so how often the fold runs is not an input to either, and 16.7 ms never appears as a
pass mark. The issue was written against the obligation's *sentence*; the instrument had already
answered it, and the answer is that the two questions are one obligation's two halves.

The population is a **derived query and not a new column**, which is the other reason O6 could be
stated after this section froze: `Layer::L2` ∪ the rows holding a `Spelling::Folded` memo. It answers
**seven** where ticket 44's own parenthesis named six.

**The invariant one layer down is the engine's, and saying so is issue 19's one edit.** *Frame cost is
proportional to visible cells, never to data volume* holds in `vitui-engine` without exception,
because the engine never iterates application data at all — and it is quoted in twenty-odd places
across this workspace, including the repository README, where it is the first sentence a stranger
reads and is stated of the library rather than of the engine. A component that must fold does not
satisfy it and does not claim to; §13 states the split it satisfies instead. `CONTEXT.md`'s glossary
entry and the repository README now carry both halves, and `vitui/tests/blurb.rs` joins the README's
paragraph to `volume::population()` being non-empty, so the sentence and the mechanism fail together
in either direction. **The engine's own README is untouched**: it states the invariant under *it never
iterates your data*, which is the scope that makes it true.

### The demand column answers *draws*, and a second list answers *what a caller must spell*

Architecture issue 25, resolved 2026-09-05, filed by issue 20's own new join.

20 settled the column's verb — **draws** — and struck four entries no component drew. Five were left:
`TeeTop`, `TeeBottom`, `TeeLeft`, `TeeRight` and `Cross`, §16's box junctions, demanded by `table`
and drawn by no line of the crate. `panel` draws a border; `table` draws **no rule at all**, because
§6 puts its column separators in the caller's cells. So no component here draws two rules that meet.

**They are not the same claim as `tree`'s, and one fact decides it.** A tree's indent guide cannot be
drawn *by anyone*: a guide column at depth *d* is a fact about *d* ancestors and §7 refuses every
route to it, so those entries were owed to nobody and striking them cost nothing. A table's
separators **can** be drawn, by exactly the caller §6 assigns them to, in the cell drawer it already
has — and that caller needs them from the *theme*, because one that hard-codes `│` and `┼` has a
table that breaks at `Repertoire::Ascii`. That is what `Glyph` is for.

So the column is **split rather than overloaded**, which is 25's third option taken deliberately
where 20 refused to invent it silently:

- `Component::glyphs` — what this component **draws**. `table`'s row is `Ellipsis` alone; eleven
  entries left it, the five junctions and the six border glyphs.
- `crate::glyphs::DELEGATED` — what this component's **caller must be able to spell**. All eleven,
  all `table`'s.

Every gate that asks *is this entry owned* reads the union, and no entry of `Glyph::ALL` is unowned.
**The collapse gate reads the union too**, and that is not bookkeeping: a caller's separators and the
table's own ellipsis land on one screen, which is exactly what *within-component cross-family
collapse* asks about. The pairwise ASCII figure stays at **42** — six on `panel`, thirty-six on
`table` — where reading the drawn set alone would have taken it to six by moving entries between
lists.

**The gate that could not see any of this is no longer vacuous.** *Every distinction with a glyph
carrier has a component drawing both halves* read the freeze's column, so *draws* meant *declares*,
and `Distinction::Guide` passed on a declaration nobody honoured — carried first by `tree` and then,
when 20 struck `tree`'s row, by `table`, neither of which drew either half. With the column split it
reads *draws* and means it.

**And `Distinction::Guide` is struck**, which is the last of 25's question — *whether a distinction
with no drawer should be a distinction*. It is not, and this one is the clearest possible case: its
own definition is *a tree's indent guide says this row has a sibling below rather than merely there
is depth here*, and architecture 20 established that this library does not draw one and cannot. A
distinction is a claim about what a narrowed repertoire can still tell apart **on a screen somebody
draws**. `Distinction::ALL` is **nine** and six of the nine name a glyph pair.

The two denominators moving apart is the finding rather than an inconsistency: **the table stayed at
twenty**. A glyph is what a theme can spell; a distinction is what a screen can still tell apart.
`DELEGATED` is deliberately not read by the distinction gate — whether a caller's screen keeps a bit
alive is a question about a screen this crate does not draw, and answering it from a list would be
the vacuity again with one more indirection.

### The glyph catalogue is a per-component demand column

Not a global list. Two gates come off it: **`GlyphSet::` occurrences in `vitui-components` == 0**
(with no private `mod missing`), and **within-component cross-family collapse == 0 at every rung** —
which is the defect §16's global 36-of-190 cannot see, because `tree` draws both `Ellipsis` and
`ArrowRight` and ASCII spells both `>`.

The `constructions: 1..=3` column is how the freeze **stops depending on `theme::subrows`** rather
than waiting for it: block elements are the Unicode rung by `CONTEXT.md`, so `chart` and `meter` are
2 and only `plot` is 3, whatever the runtime's table says.

### Fifteen families, each with a v1 expression or a named home

Two — F11 media and F12 files — have no v1 *component* and both are covered by a resolved ticket
(§14, §15). The rest is §18.

---
## 18. Coverage — the claim of C1

This is the chapter neither the engine's spec nor the runtime's had to write, and it is the whole
reason the map opened with a survey rather than with a widget list.

> **We will not implement every component that exists. Nothing that exists may be impossible.**

A spec that lists twenty-nine components without saying why the rest of the catalogue is expressible
has not made the argument. This section makes it, in the only form that can be checked: not a claim
that the residue is small, but a **mechanism named for every entry that is not one of the
twenty-nine**, and a residue enumerated by name rather than by adjective.

### The arithmetic, corrected

| | |
|---|---|
| sources fetched live | **17** |
| of those, carrying an enumerable component inventory | **14** — Material names a taxonomy, ECharts names series types, Qt is written as a delta over JavaFX (C10) |
| families in the union catalogue | **15** |
| named entries across them | **~430** |
| shipped in v1 | **29** (§17) |

The map's own header line — *ten families, ~250 components* — is **stale**, and so is the survey's
*33*; both are superseded by the numbers above (C10). The residue this section must account for is
therefore **~400 entries**, not ~217.

### The six reductions

Every entry in the catalogue that is not one of the twenty-nine falls into one of six classes.
The classes are not a taxonomy invented here — each is a decision taken on a ticket, and each is
cited.

**R1 — It is one of the twenty-nine under another name.** The survey counts names, and libraries
name the same mechanism differently. §5's `Mode` is thirteen match arms and it absorbs
`list · option list · selection list · checklist · listbox · menu · submenu · context menu ·
tiered menu · panel menu · tabs · content switcher · radio set · segmented control · button group ·
multi-select · pick list · transfer · order list · recent items · favourites · search results ·
command palette body · quick switcher body · which-key body`. §11's flag absorbs
`text input · password · masked · pattern · search input · OTP · tag input · mention · autocomplete
input · textarea · auto-growing textarea · editable label · inplace editor · float label ·
character-limited input · unit / currency / percentage input`. §8's machine absorbs the whole of F4
— twenty-one entries — for the reason F4 exists as a family: *every one of these is the same state
machine.* §12's two axes absorb `popup · dropdown · popover · popconfirm · tooltip · modal ·
non-modal · alert · confirm · prompt · file / colour / about / shortcuts dialog · drawer · sheet ·
toast stack · notification stack · splash · block-UI`.
**A collapse under R1 is only legitimate when a ticket measured it** — §17 records six, five measured
and one (gauge → `meter`) marked as a judgement.

**R2 — It is an argument, not a component.** A `Role` (ADR 0018) absorbs
`primary / secondary / ghost / danger button · link button · command link · status LED · health
pill · dot indicator · badge variants · inline message · banner · alert · callout`. An axis argument
absorbs §9's four bands (`sticky header · footer · group header · pinned rows · pinned columns ·
gutter`). A `bool` absorbs modality (§12). A gradient is a **`fill` with a varying style** — not a
blend mode and not a component (engine ticket 06), which is requirement 10's own example:
`bar(cx, area, 0.62, Gradient::horizontal(RED, GREEN))` is one call and one argument change.
**The direction runs both ways and that is the whole point of families F3 and F4**: what is a
container *attribute* in CSS or in a retained framework is a *component* here, and what is a
*component* in those inventories (a button variant) is an argument here. An inventory-driven count
is structurally blind to both directions.

**R3 — It is a composition of shipped components with no new mechanism.** `file_picker` is
`collection` + `overlay` + the preview pane (§15, and the sentence *no mechanism in it that is new*
is C16's, measured). `command palette` is `collection` + `overlay` + the caller's filter (§10's
order). `process table` is `table` + `plot` + a memo (§6, §13 — C04's screen *is* a million-row
process table). `form` is `field` + `nav::cursor` + the focus ring the draw builds (R08). `wizard`,
`stepper`, `dual pane`, `Miller columns`, `master-detail`, `property grid`, `JSON tree`, `key-value
viewer`, `descriptions`, `column chooser`, `bulk-action bar`, `page-size selector`, `saved views`,
`export menu` are each two or three of the twenty-nine and a rect split. **R3 is the class that
requires the most care**, because *composition without a new mechanism* is exactly the claim that
turns out to be false when it is false — which is why §17 keeps a `built` column and why C16 was a
ticket rather than an assertion.

**R4 — It is a construction question at the declared repertoire, and ADR 0009 already governs it.**
Every ⚠️ in F10 means one thing: *a shape that is not axis-aligned costs sub-cell rasterisation.* §13
priced that ladder exactly — 2 / 16 / 256 states a cell for marks, 2 / 9 / 9 for prefixes — and
`donut · radial gauge · pie · rose · radar · violin · bubble · sankey · parallel coordinates ·
sunburst · graph · contour · surface · treemap · geo map · globe` are all the same rasteriser with a
different mapping from data to sub-cells. §14 added the one correction that matters: **a picture's
ladder is a colour ladder, not a glyph ladder**, so `image · gallery · lightbox · thumbnail strip ·
compare · avatar · colour wheel · gradient strip · icon set · emoji picker` degrade along a different
axis and reach a floor of *nothing* rather than a floor of *worse*. A component that cannot express
itself at a rung **builds something different**; it never substitutes content.

**R5 — It is caller state plus the one index of §10.** Twenty-one of the survey's twenty-two
data-grid features are state or layout, assigned individually in
`research/05-table.md`; the twenty-second, variable row height, is a fourth
*field* on §7's record. `infinite scroll · server-side row model · row grouping with aggregates ·
pivot · tree data · multi-column sort · filter row · quick filter · column resize / reorder / hide /
groups · minimap · scroll spy · follow-tail · timeline · gantt · kanban · heatmap grid · calendar
heatmap · paginator · diff · blame gutter · hex dump · markdown document · syntax-highlighted code`
are the same shape: a caller-owned, O(1)-indexed materialisation built on the edit and spliced. **The
architectural fact that makes this a reduction rather than a deferral** is that the index is the
*caller's* — so an entry in this class needs no library mechanism at all, only a documented shape.

**R6 — It belongs to the runtime, or the terminal deleted it.** All twenty-three entries of F15 are
the runtime's by `COMPONENT-HIERARCHY.md`'s T2 test: **they emit no cells** (key maps, focus scopes,
clipboard, virtualiser, router, timers, theme provider, i18n, error boundary, inspector). Bubbles
ships two of them as components and Ant ships three; that is a packaging choice, not a disagreement
about what they are. And the survey's §4 enumerates what a terminal deletes with the fact that
deletes it: pixel-precise anything, alpha (a cell has no alpha channel, so a scrim cannot have a hole
in it), sub-cell layout, hover with no pointer, cross-process drag and drop, bidi and complex
shaping, screen readers. **"Missing" is never mistaken for "not yet done" because the fact is written
beside it.**

### Family by family

| Family | entries | expressed by | the reduction, and its class |
|---|---|---|---|
| F1 text | ~48 | `text`, `chip`, `field`, `fit` | R1, R5. Markdown needs its own wrap pass (§22); no bidi (non-goal) |
| F2 structure | ~38 | `panel`, `rule`, `block`, operator layers | R2. A scrim is a complement, never a fill (§2); no cut-out (no alpha) |
| F3 scrolling | 18 | `scroll_area`, `scrollbar`, `sticky`, `collection` | R2, R5. `pull-to-refresh` is ⛔ — a touch gesture |
| F4 disclosure | 21 | `collapsible` | R1 — one machine, and the family exists to say so |
| F5 indicators | ~46 | `meter`, `chart`, `plot`, `overlay`, deadlines | R2, R4 |
| F6 input | ~110 | `field`, `button`, `chip`, `select`, `collection`, `slider` | R1, R2, R3. Colour wheel, dial and font picker are R4; ctrl-click and shift-click are **open** |
| F7 collections | ~50 | `collection` + columns / an index / tiles | R1, R5. Org chart, mind map and pivot are layout research (§22) |
| F8 navigation | ~35 | `collection` + `overlay` | R1, R3. Dock is v2 |
| F9 overlays | ~35 | `overlay`, layers, placement, scopes, `Trap` | R1, R2 |
| F10 charts | ~55 | `chart`, `plot`, axes, two memos | R4 — every ⚠️ is one rasteriser and one mapping |
| F11 media | 16 | picture as `Solid`/`Half`, QR, waveform, player chrome | R4. **Terminal-native fidelity and camera/video are ⛔** (§14) |
| F12 files | ~35 | `collection` + `scroll_area` + worker + the preview pane | R3. Picture fidelity inherits F11 |
| F13 system | ~30 | `meter`, `table`, `plot`, a log `collection` | R3, R5. Topology and service maps are layout research |
| F14 terminal-native | ~24 | `canvas` at three rungs, `pty` | R4. **OSC 52 and the bell are ⛔**; OSC 8 and cursor shape are ✅ |
| F15 behavioural | ~23 | **not here** — `vitui-runtime` | R6, by the T2 test |

### The residue, by name

Everything that is not expressible, with the fact that makes it so. There is no fourth category: an
entry is shipped, reduced with the reduction named, or listed here.

| | why | who could change it |
|---|---|---|
| **graphics passthrough** — sixel, kitty, iTerm2; terminal-native image fidelity; video; camera preview | the engine emits no out-of-band bytes. §14 splits the reason in two: **a preview pane's picture fails for fidelity, a video's for bandwidth** (24.6 MB/s lower bound at truecolor) | the engine map, survey §6.1 |
| **OSC 52 clipboard**, **bell / visual flash** | out-of-band sequences the engine does not emit | the engine map |
| **pull-to-refresh** | a touch gesture with no terminal meaning | nobody |
| **alpha** — a scrim with a hole, a translucent panel | a terminal cell has no alpha channel; a scrim is a colour operator | nobody |
| **bidi, complex shaping, IME preedit, block (column) selection** | stated non-goals with reasons; the engine's tables give clusters and widths, not a shaper | a later map, deliberately |
| **ctrl-click and shift-click** | `rt::Input` carries **no modifier byte on a pointer event** (C03, C04, C05). Multi-select's *pointer* gestures are keyboard-only; the keyboard half is complete | the runtime map — §22 |
| **a background image below the base pass** | the engine can express it and `rt::overlay::LayerHost` cannot, in three places (§14). Built on the engine's own `composite` it is 13.7× cheaper than any component-level placement | the runtime map — §22 |
| **word motion** (`Ctrl+Left`/`Right`) | needs UAX #29's word-boundary half, which the engine's export does not offer | the engine map, beside R03's UAX #14 near-miss |

**Four of those eight are the same ⛔ the survey named on day one**, and the other four were found by
building. That ratio is the honest summary of this chapter: the survey's coverage table was right
about what the terminal refuses and blind to what a seam refuses.

### How the claim is checked rather than asserted

C1's check was *the coverage table is complete and every ⚠️/⛔ row has a reason*, which is a review
of a document. It is superseded by three things that run:

1. **C3, proof by exemplar per family — the hardest member, not the easiest.** **Eleven** of the
   twelve named exemplars were built and measured: `table` (§6), `scroll_area` with both axes (§9),
   `collapsible` (§8), `select` (§12), `chart` (§13), `modal` (§12), `textarea` (§11), `file preview
   pane` (§15), `video player chrome` (§14, six parts of ten), `process table` (§6), and
   **`command palette`** — components ticket 43 (2026-08-28),
   `research/43-the-command-palette.md`.
   **One is not**: `pty` (§22). Recorded as one, not waved through.

   **R3 holds for the palette and R5's own sentence does not.** Eleven mechanisms, none of them new —
   and the prototype crate depends on `vitui-runtime` and `vitui-components` and nothing else, which
   is what makes *a caller can assemble it* a fact rather than a claim. What the ticket found instead
   is on the other side: **a filter is R5's index edited at keystroke rate**, its removal is `runs`
   intervals rather than one, `runs` grows linearly with the corpus (161 / 1 608 / 16 072 / 160 714),
   and every `Vec::splice` memmoves the tail — so *built on the edit and **spliced*** is
   `O(n × runs)` against a rescan's `O(n)`. It wins at a thousand candidates (0.36×) and loses at a
   million (**48.47×**), crossing between ten and thirty thousand. **That is a correction to R5's
   wording and not a new mechanism**: both spellings are the caller's `order`, and the palette draws
   identically over either. R5 is right about the edits it was written about — a fold, a sort, a row
   arriving — and a filter is the one member of the class edited at keystroke rate.

   **And the ticket added a column §18 does not have: four obligations the caller acquires**, each of
   which draws a *perfect screen* when it is left out — reset the cursor (the revision clamps rather
   than resets it), keep it in view (**0 of 40** frames off screen against **30 of 40**), key the
   children under the container's own id (`#[track_caller]` forwards the container's caller location
   to every `#[track_caller]` function it calls, so the list mints the palette's own id and the merge
   makes it inert: **3 regions and a press that lands against 2 and a press that does nothing**, the
   two screens **0 cells apart of 2 436**), and say `CollState::reconciled` (**3 selected rows kept
   against 0**). *A composition with no new mechanism can still be a composition nobody gets right
   twice.*
2. **§17's `INVENTORY`**, against which every obligation is enumerable — including O5, whose whole
   argument is that a gate nobody can enumerate is not a gate.
3. **§21's scene list**, which is normative: a scene is removed only by a ticket naming the property
   it can no longer distinguish.

---

## 19. The module tree, the crate line and features

**A module is a family.** The module tree follows the fifteen families of the survey so that a reader
who knows what they want finds it without a search, and `INVENTORY`'s `families` column is the join
that makes the mapping checkable rather than a naming convention. **The rule that matters is that a
module may only depend on lower layers**, which keeps the library graph a DAG and is checkable from
the `layer` column.

**What is settled here is the rule and not the spelling.** `architecture.md` §2 lists sixteen module
directories under the pre-correction "ten families", and **no ticket ratified the directory names**.
They are a starting point for `/to-tickets`, not a decision, and renaming one reopens nothing.

**The crate line.** `vitui-components` depends on `vitui-runtime` and on nothing else. It does not
depend on `vitui-engine` — that is how C6 is checked, and `deny.toml` enforces the other half through
ADR 0001's wrapper list. Two facts from C11 belong here because they are the crate line as it
actually behaves:

- **`cargo deny` had been red since C02** — 13 `error[wildcard]` and 1 `error[banned]`. Every
  component prototype wrote `{ path = "../proto-r14-rt" }` with no `version`, which `wildcards =
  "deny"` refuses. Both fixed rather than excepted; the gallery binary joins the wrapper list with
  its reason beside it, because it uses crossterm for raw mode, the alternate screen and input while
  its bytes go out through the runtime's surface — which is ADR 0001's own sentence rather than an
  exemption from it.
- **No components crate named `vitui-alloc-probe`** — twelve prototypes each hand-rolled a counting
  global allocator. The crate that exists for this is the one to use (§21).

**Feature flags are not decided.** `architecture.md` proposes `default = ["core"]` plus eight
features; no ticket ratified it, and §17's **tiers** are the shipping unit the freeze actually
produced. Whether tiers and features are the same partition is §22.

---

## 20. The performance budget

Inherited unchanged from the engine map, with the component's own work included: **full-screen 300×80
composition < 1 ms; typical damage-tracked frame < 100 µs; zero allocations during frame
composition.** C10 (requirements C10, C11, C13) is checked per component and not per application.

### Steady frames, every one of them 0 marked and 0 allocations

| screen | µs | writes | verbs | regions | flat over |
|---|---|---|---|---|---|
| §12 the overlay screen, 312 chips, nothing open | **33.96** | — | — | 317 | — |
| §8 twelve sections closed | 24.38 | 23 034 | 230 | 70 | — |
| §7 a million-node forest | 48.79 | 23 030 | 640 | 59 | 1k / 100k / 1M, depth 59 999 |
| §9 a 1M-row area, both axes | 48.83 | 33 890 | 1 453 | 59 | 1k → 1M, 1.00× |
| §5 four collections | 63.90 | 20 914 | — | 229 | 1k / 100k / 1M |
| §3 the helper screen, 267 regions | 69.33 | 20 804 | — | 267 | 1k → 1M |
| §11 twenty fields and a 1 MB textarea | 82.40 | 19 634 | 631 | 79 | 100 kB, 1 MB |
| §2 the call-site screen, 338 regions | 84.96 | 20 784 | — | 338 | 1k / 100k / 1M |
| §16 at all nine matrix cells | 93.29 | 23 402 | 2 163 | — | nine cells |
| §6 a twelve-column million-row table | **165.6** | 23 030 | 2 497 | 59 | 1k→1M, 40→240 columns |
| §13 two million-point series | **204** | 21 872 | 2 472–3 772 | 2 | 1k / 100k / 1M |
| §15 a preview pane with a photograph | **237** | 17 782 | 14 833 | — | 1k / 100k / 1M |
| §14 a full-screen picture | **261–357** | 24 000 | 24 000 | — | — |

### The four frames that are over budget while damaging nothing, and it is a product call

`table` 166 µs · `plot` 204 µs · a preview pane with a photograph 237 µs · a full-screen picture
261–357 µs. **All four are the draw and not the composite**, all four mark zero cells, and the
budget line has never been priced against a screen that is *one component*. C04 and C05 called this
C10's; C10 called it **a product call for the user**, and C11 recorded that it now has four instances
rather than one. It is filed in §22 unanswered, because answering it is not this map's to do.

### The transition frames, each a cliff by construction

| frame | cost | why it is allowed |
|---|---|---|
| a modal opening over the dense screen (§12) | **138.04 µs** | 25 080 cells once per opening; the steady state is +0.75 µs |
| a filled scrim, every frame it is up (§2) | +54.17 µs | **not** allowed — this is the defect, and the complement is 0 |
| the first frame, and every resize (§2) | one `cx.clear` | the alternative is 6 662 damaged cells a frame |
| a landing frame in a preview pane (§15) | the whole pane | one per selection, bounded by the key repeat |

### The edits, which are not frames

`Raster::build` over 1M points **4 881 µs** · a sort materialised over 1M rows **21 158 µs** · a
selection remapped under a permutation **22 941 µs** · a tree collapse **157–342 µs** · a fold
reanchored **1.04 µs** · a wrap splice at 1 MB **34.34 µs**. Every one is on the edit and behind a
memo or a splice (§10), and **where a 4 881 µs rasterisation runs is R18's question, not this map's**
(§22).

### Allocations

Zero on a steady frame, at every size, in every prototype — with one real exception found only when
the figure was computed as a **total** rather than an integer mean (§21): `player::chrome` collected
a `Vec<f32>` on the draw path, **40 allocations over 40 frames**, against a budget of zero. It is a
field now, derived at construction, and the shape it replaced is kept as a negative case.

---
## 21. Verification

The register is engine ticket 13's split with R15's two refinements, and C11 found the same thing one
map down that R15 found on the runtime's: **the corpus was not being run.** (C11)

| | |
|---|---|
| gates the twelve component tickets produced | **18** |
| of those, evaluated by anything `cargo test` runs | **2** — §9's bar fixpoint and R18's G10 doctest |
| asserted inside one prototype's own binary, on one screen, its author's | **6** |
| a number printed by a `fn main()`, which `cargo test` does not run, or absent | **10** |
| after C11 | **11 evaluated / 3 in-prototype / 4 report-only / 0 absent** — six moved from *nothing evaluates this* |
| `*_numbers.rs` examples on this lineage before C11 | **0**, against the runtime's 19 |

**The register is data, not prose** — `proto_c11_gates::register::REGISTER`, one row per gate with
its kind, its owner, where it stood at the branch point and where it stands now, so the delta is a
number a test asserts. The reason is §17's: every obligation stated as a sentence on this map has
been broken by someone who had read it.

### What a gate is

R15's rules are inherited unchanged: **a gate is a count, a ratio, an equality or a compile outcome;
a timing is a report** · **a gate is an equality only when the number is a property of the mechanism**
· **a report may not be load-bearing for a gate** · **a positive twin must name the protected item by
path.** This map adds three of its own, each from a gate that was green and wrong.

1. **A threshold on the wrong side of the question is not a weak gate, it is a green one.** The
   gallery's theme-swap gate asserted `changed > 0` — *the theme changed and not one cell moved* —
   and **one cell of 4 800 satisfies it** while 3 583 carry the old palette.
2. **A mean cannot see anything below n; a total can see one.** Every prototype reported
   `allocs / n` with n = 40–200, so a frame allocating on n−1 of n frames reports 0. Run as a total,
   eleven panels are 0 and one is 40 over 40 (§20).
3. **Name the exception; do not loosen the gate.** R08's *a walkthrough visits every tab stop exactly
   once* fails on exactly one panel of twelve, and **correctly**: §12's dialog is open and a `Trap` is
   what a modal is (8 stops declared, 2 visited). The gate becomes the conjunction R08 meant — *the
   walk repeats no id, and reaches every stop unless a trap is standing* — with `Frame::trap_scopes()`
   added so the exception can be named.

And one gate that **cannot be written as C14 asked for it**, which is a result rather than a gap:
*tab stops == visible focusables* needs geometry the ring does not carry (`RingEnt::rect` is zero
unless `Frame::ring_geometry` is on *and* the entry was declared inside a scroll area — R08 kept
geometry out deliberately, R17 put it back for one case). Its **mechanism form** needs no geometry
and is green: *a component drawn into a zero-height rectangle declares no tab stops*, which is §8's
478-against-70 stated so it can be run against anything.

### The register

| gate | kind | owner | standing |
|---|---|---|---|
| cells marked on a steady frame == 0 | count | C01 — every component | report per component; assembled form below |
| the same screen drawn both ways is equal cell for cell | equality | C01 | the gate that keeps the first honest |
| duplicate merges a frame == 0 | count | C01 | green; caught four identity defects |
| writes flat 1k → 1M | count | C01 | report per component |
| a chord pressed into every focusable types nothing | count | C01, C06 | green |
| `writes == distinct cells touched` (no cell twice) | equality | C02 | report per component |
| **every cell of the rectangle written at least once** (no cell never) | count, sentinel | **C11** | **red on six panels, pinned with its failing set** |
| **no cell keeps the previous palette a frame after a swap** | count | **C11 / R20 §3** | **red on six panels, pinned** |
| regions identical at 1k and 1M | equality | C03 | green |
| `size_of::<CollState>()` independent of length | count | C03 | green |
| writes and verbs identical across declared column counts | equality | C04 | green |
| the same table under a horizontal offset, against itself | equality | C04 | green |
| an interval edit on the selection store against a bit vector | equality | C05 | green |
| a fold/unfold round trip, and `splice == rebuild` | equality | C05 | green |
| the caret is always on a cluster boundary | equality | C06 | green |
| the caret's column equals the engine's tables over the prefix | equality | C06 | green |
| a spliced wrap index equals a rebuilt one | equality | C06 | green |
| the index's recorded width equals the width being drawn | equality | C06 | green |
| `verbs <= writes` | relation | C08 | green — never verb equality across sizes |
| the bar fixpoint over 5 475 600 pairs | count | C13 | green — one of the two that were being run |
| the surface after a collapse equals a freshly built one | equality | C14 | green |
| a closed body declares no ring entry and no hit entry | count + equality | C14 | green |
| a fold anchored on a position is reconciled | count | C14 | green |
| the inplace map round-trips | equality | C14 | green |
| `open` is never ambiguous mid-transition | invariant | C14 | green |
| `GlyphSet::` in `vitui-components` == 0, no private `mod missing` | count | C09, C10 | **red, pinned** — nine crates carry one |
| within-component **cross-family** glyph collapse == 0 at every rung | count | C10, corrected by C11 | green over the whole freeze |
| a too-long label at ASCII does not end in `ArrowRight` | equality on the drawn cell | C09 | the end-to-end form; the source count is the symptom |
| twenty wheel clicks move the offset twenty | count | C07, C10 | **red, pinned in both directions** |
| O1–O5, as queries over `INVENTORY` | mixed | C10 | green; O2 is two equalities |
| `Task` and `Worker` are `!Sync`; `Cell`, `RefCell`, `Worker` **are** `Send` | compile outcome, paired | C16 (G10, G10b) | green |
| zero allocations in a steady frame, as a **total** | count | all | green on eleven panels, one fixed |

**Three gates are red and are pinned in their failing state rather than fixed or ignored.** Each
asserts its exact failing set, fires in both directions, and says what to invert when it is fixed;
the code that fixes them belongs to the implementation backlog, not to this map (C10 split each
defect three ways and gave this map the rule and the detector). **Four are report-only** — `writes == distinct`, `marked == 0`,
`merges == 0` and *writes flat 1k→1M* are printed by nine to twelve binaries and asserted by two or
three, and the honest statement is that the per-component forms remain reports until the crates
exist. Their assembled-screen forms are the two sentinel gates above.

**One green gate should not be banked.** *The surface after a shrink equals a freshly built one*,
written against a terminal resize, passes on all twelve panels — because `Gallery::resize` allocates
a new `Surface` and the residue has nowhere to survive. **That spelling tests the resize path and not
the defect**, which is content shrinking inside a rectangle that does not move.

### One test command, and the instruments

`cargo test --workspace -- --test-threads=1`, serialised because the allocation probe's counter is
process-global. The probe is **`vitui-alloc-probe`**, not a hand-rolled counting allocator per crate
(§19). `alloc_zeroed` is not a blind spot — the default calls `self.alloc`, checked in a test rather
than asserted in prose, because *twelve of twelve omit a method* is exactly the shape of a claim the
next reader files as a defect.

**A CI job nobody has watched go green is not a gate, whatever it checks.** Three for three: engine
ticket 13 left a `cargo test --workspace` step that had never passed, R15 found a `clippy` step that
had never passed under its own `-D warnings`, and C11 found a `cargo deny` job that had never passed
at all (§19).

### The scene list is normative

A scene is removed only by a ticket naming the property it can no longer distinguish. Three of these
exist because a defect survived every gate then in force by not being on any screen anybody had
built.

| scene | what it decided |
|---|---|
| the dense screen, 300×80, 267–338 regions | the partition rule: 214.58 µs / 6 662 damaged against 84.96 / 0 |
| the same screen drawn naive and correct | 0 of 24 000 cells apart — what keeps the damage count honest |
| four collections at 1k / 100k / 1M | one store, one `Mode`; 63.87 / 63.87 / 64.08 µs |
| **a scrolled collection** | the inverted scroll sign: 12.21 µs / 3 058 writes against 62.96 / 20 418, and *faster* |
| **a collection shorter than its viewport** | the stale tail: 71 of 80 rows, the defective build 2.3× faster marking 226× less |
| **twenty wheel clicks over a collection** | the unconditional scroll-into-view: 0 against 16, in four resolved tickets' code |
| a twelve-column 1M-row table, pinned both edges, horizontal overflow | verbs as the currency: 913 → 2 497; the band's 345 re-damaged cells |
| a million-node forest at depth 59 999 | the flatten index; 640 verbs against 418 and 2 497 |
| a fold and an unfold over 349 524 rows | splice against permutation: 1 run / 0.04 µs against 297 180 / 24 338 |
| a 200 000-line document with 4 167 folds and an insert above them | the anchor: 4 166 of 4 167 wrong, reanchored for 1.04 µs |
| an accordion of twelve sections, 0 / 6 / 12 open | closed content is not drawn: 478 hit entries against 70 |
| twenty fields and a 1 MB pasted textarea | the caret pair and the index: 9 655× on one `Left` |
| a 300 → 120 resize with a wrap memo | the memo key: 625 rows drawn where 875 are needed |
| a select, a menu with a submenu, a modal and a scrim | the family, and the 138.04 µs opening frame |
| two million-point series at 1k / 100k / 1M, at 300×80 and 60×20 | the raster memo, 1 849 000×; and 60×20, where C08's overlap is red |
| 175 712 axis viewport × dataset pairs | the axis loop oscillates on 464; from the whole domain, 0 |
| 5 475 600 viewport × extent pairs | the bar fixpoint: 0 failures, ≤ 3 passes |
| a 1M-row scroll area with one row in eight three cells tall | `Σ h` is the extent: row 799 999 of 999 999, every count identical |
| **two scroll areas far apart with overlay bars** | damage is one span per row: ×8.4 amplification (owed) |
| the nine cells of the repertoire × tier matrix | 21 → 1 677 signal pairs; 2 distinctions of 9 lost |
| a repertoire swap with a memoised glyph prefix | the theme's revision in the key: 598 wrong characters |
| a full-screen picture at three colour depths | 24 000 customs; 50.7% of horizontal distinctions gone at C16 |
| twenty selections through a directory of photographs | R18 §5's crossover: 1 picture / 536 KB against 20 / 10.7 MB |
| a directory streamed in seven batches under a sort | the dedupe key: wrong after 7 of 7 batches, with no user in it |
| a re-sort under a preview pane, 200 files | 197 of 200 positions move; wrong on 100 of 100 frames |
| **the assembled gallery, twelve panels, 53 280 cells** | 9 956 cells nobody writes; the swap excess equal to it on five of six |
| a theme swap over the gallery | R20 §3 has no caller; six panels keep the old palette permanently |

**Every defect below is invisible on the screen of the ticket that owns the mechanism and visible on
the screen where the components meet.** That is the argument for the last two rows, and it is why the
gallery is a gate rather than a demo.

---

## 22. What this spec does not decide

Each of these is fog with a named shape. Nothing here was answered by this ticket, because answering
a question a ticket closed too early is how a spec stops being a record.

**Owed by the runtime map, each with the number that will make it worth reopening.**

- **A modifier byte on a pointer event.** `rt::Input` carries none, so ctrl-click and shift-click are
  inexpressible and `Mode::Multi`'s pointer gestures are keyboard-only (§5, §18). Recorded by C03,
  C04 and C05 and never filed.
- **`Memo<T>` taking a key rather than a `Revision`, and a value that can be *maintained*.** Six
  crates hand-roll a compound key (§10); held inside `Memo`, C06's wrap index is 3 543 µs a keystroke
  against 34 spliced.
- **A layer slot below the base pass**, with a non-opaque content blit in `LayerHost` and a way to
  clear a surface to transparent. §14's background image needs all three and needs **no engine change
  at all**.
- **`theme::subrows` puts the eighth blocks at the wrong rung** (§13). The freeze does not wait on it;
  the runtime's own table is wrong in 4 706 cells where nobody looks.
- **§16 allocates the four tees and `Cross` to `table` and nothing draws them.** The paragraph that
  takes the table to twenty hands every entry an owner, and the join that was supposed to keep that
  honest — *every entry has a demander* — compares a declaration with a declaration, so a row
  demanding an entry it draws nothing of satisfies it. Components architecture 20 struck the four it
  could settle (`tree`'s three guides on §7's argument, `select`'s stepper `ArrowUp` because a
  popup's gutter is `bar_into`) and built `crate::glyphs::DRAWERS`/`UNDRAWN`, which joins each entry
  to a **line**: 15 drawn, 5 not. The five are the box junctions and a junction is where two rules
  meet — `panel` draws a border, `table` draws no rule at all — so this is one missing construction
  rather than five stale entries, and whether `table` should draw its own rules is a verb budget
  against §6's `913 → 2 497`. Components architecture 25.
- **Where a 4 881 µs rasterisation runs** — an edit, not a frame; R18's.
- **A reverse cluster step** (`prev_cluster(&str, i)`) beside `Graphemes`, and **UAX #29's
  word-boundary half** for `Ctrl+Left`/`Right`. Neither is asked for; both are the engine's tables
  (ADR 0005), and the second should be settled with R03's UAX #14 near-miss rather than separately.

**Product calls, not architecture.**

- **The frame budget against a screen that is one component.** Four instances now: 166 / 204 / 237 /
  261–357 µs against 100, all damaging nothing (§20). C04 and C05 called it C10's; C10 called it the
  user's; it is still the user's.
- **Whether a click on a scrollbar track pages or jumps.** The geometry is exact and reversible and
  R05 already owns the grab.
- **Whether a help bar may show a binding as unavailable** — inherited from the runtime spec's §21 as
  a components question, and still open.

**Mechanisms named with an owner and not prototyped.**

- ~~**A component that owns a clock.**~~ **Prototyped and answered**, components implementation
  ticket 42 (2026-08-28).
  A component may own an **anchor** and may not own a **clock**: *stored state may be an anchor,
  never a phase*, which is the rule §8 and §9 were already obeying rather than a permission granted
  here. `Driver::pin_clock` advanced eight ladder steps and the self-sampling arm showed **1 of the
  anchored arm's 8** distinct frames — it never left the one it started on —
  with a tween beside it as the control, which is the refusal as a count. Implementation is
  components ticket 46.
- **The per-cell quadrant fallback for colliding series at Extended** — counted at 25 cells, not
  built (§13).
- **`pty`'s process model.** It borders on requirement 9 and may need its own ticket; it is one of
  the two C3 exemplars that were not built (§18).
- **Markdown rendering** (its own wrap pass) and **the syntax-highlighting seam**.
- ~~**`command palette`**, the other unbuilt C3 exemplar.~~ **Prototyped and answered**, components
  ticket 43 (2026-08-28) —
  `research/43-the-command-palette.md`.
  **R3 holds**: eleven mechanisms and none of them new, so §18's exemplar count is eleven of twelve
  and `pty` is the one left. The finding is R5's rather than R3's — *spliced* is the wrong index
  spelling at keystroke rate, **48.47×** slower at a million candidates than the rescan it is supposed
  to beat — and beside it four obligations a caller acquires that render perfectly when they are
  missing. **Nothing ships**: a palette is a composition, which is what R3 means.

**Left open inside a resolved ticket.**

- **A collapsible inside a scroll area**: under the extent shape the container learns its content
  height one frame late. Neither §8's nor §9's alone (C14).
- **Nested collapsibles**, and whether a body that is never called keeps a stale height under
  `HeightFrom::Extent` (C14).
- **`Opts::scope_identity`** is exercised by no measurement; C14 recorded one observation and refused
  to call it a result.
- **Org charts, mind maps, pivot tables, topology and service maps** — layout research, four families
  away from a mechanism.
- **Block (column) selection**: a rectangle over visual rows, which neither §5's runs nor §11's range
  expresses. It belongs to whoever writes the code editor (C06).

**Decided elsewhere and not reopened here.** The module directory names and the feature-flag
partition (§19); the tier boundary between what ships and what waits (§17 froze it and a product call
can move it).

---

## Appendix A — ADRs this spec rests on

Inherited from the engine map: **0001** crossterm behind a seam · **0005** the engine owns the caret
and exports its text tables · **0009** the engine degrades presentation, never content · **0010**
detected axes are flat, declared axes are ordered · **0022** drawing verbs clamp and discard ·
**0023** the cell is never visible in the public API.

Inherited from the runtime map: **0012** frame state is rebuilt from the draw · **0013** identity
comes from the call site · **0014** there is no measure pass · **0015** no geometry crosses a frame ·
**0017** the overlay is two-phase · **0018** a component names a role and can never construct a
paint · **0019** the data contract is three types and no trait · **0020** reactivity lives above the
runtime · **0021** the runtime reads a capability and never rewrites a declaration.

Added by this map:

| ADR | Decision | § |
|---|---|---|
| [0026](../../docs/adr/0026-a-component-writes-a-partition-of-its-rectangle.md) | A component writes a partition of its rectangle | §2 §3 |
| [0027](../../docs/adr/0027-identity-is-taken-outside-every-closure.md) | Identity is taken outside every closure | §4 |
| [0028](../../docs/adr/0028-the-collection-is-one-component.md) | The collection is one component; table and tree are it plus an index | §5 §6 §7 |
| [0029](../../docs/adr/0029-scrollbars-are-reserved-not-overlaid.md) | Scrollbars are reserved, not overlaid | §9 |
| [0030](../../docs/adr/0030-a-memo-key-is-every-input.md) | A memo's key is every input | §10 |
| [0031](../../docs/adr/0031-a-collection-stores-positions-in-an-order-the-caller-owns.md) | A collection stores positions in an order the caller owns | §7 §10 |
| [0032](../../docs/adr/0032-a-distinction-survives-only-if-it-is-carried-on-both-axes.md) | A distinction survives only if it is carried on both axes | §13 §16 |
| [0033](../../docs/adr/0033-the-inventory-is-a-value-and-every-obligation-is-a-query-over-it.md) | The inventory is a value and every obligation is a query over it | §17 §21 |

**Weighed and refused**, so the next reader does not re-derive them:

- **Closed content is not drawn** (§8) — surprising, and its violation is invisible to a golden-cell
  gate, but it is cheap to fix wherever it is found and it reverses nothing. It has a gate; it does
  not need an ADR.
- **A chord is not text** (§3) — one comparison, one line. It cost R12 six tickets to notice and
  nothing to fix.
- **The scrim is the complement, never a fill** (§2, §12) — an instance of 0026, not a decision of
  its own.
- **`focus_ring` is deleted** (§3) — a consequence of 0026's restyle rule.
- **One owner is one layer** (§12) — a consequence of ADR 0017 plus 0027's opaque `Id`.
- **A picture's ladder is a colour ladder** (§14) — a construction fact with a number, not a fork;
  ADR 0009 already governs it.
- **Verbs, not cells, are what a table costs** (§6) — a measurement, not a decision.
- **Drag capture needs nothing new** (§14) — a question closed by building, which is a finding.

## Appendix B — where each decision was taken

| § | Ticket |
|---|---|
| §1 §2 | C01 the call-site shape |
| §2 §3 | C02 the shared helpers |
| §4 | C01, C02, C03 list and selection, C04 table |
| §5 | C03 list and selection |
| §6 | C04 table |
| §7 §10 | C05 tree |
| §8 | C14 collapsible |
| §9 | C13 scroll area and scrollbar |
| §10 | C04, C05, C06 input and textarea, C08 chart and plot, C09 theme, C16 file preview pane |
| §11 | C06 input and textarea |
| §12 | C07 select and overlays |
| §13 | C08 chart and plot |
| §14 | C15 media |
| §15 | C16 file preview pane |
| §16 | C09 theme and the degradation matrix |
| §17 | C10 the v1 inventory, C11 verification |
| §18 | research/01 the component library survey, C10, and every ticket that built an exemplar |
| §19 §21 | C11 verification |
| §20 | every ticket; the four over-budget frames are C04, C08, C15, C16 |
