# `vitui-engine` — architecture spec

Status: **settled**. Date: 2026-08-19. Written by engine ticket 15, the destination of
Map: vitui engine architecture.

Nothing here is invention. Every decision below arrived on one of the map's eighteen tickets with a
measurement or a refusal behind it, and the ticket is cited in the margin so the argument can be
re-read rather than re-had. Where consolidation found a question still open, it went back on the map
as fog rather than being answered here — §15 is that list, and it is short.

The spec is done when `/to-tickets` can slice it into implementation work without reopening a
decision.

**Reading order for a newcomer:** `CONTEXT.md` (the glossary — its terms are used here without
re-definition) → this file → `docs/adr/0001`–`0011` and `0022`–`0025` for the decisions that are hard
to reverse → the tickets in `issues/` for the measurements → `research/` for the primary sources.

Every timing in this document is a criterion median or a minimum-of-forty round-robin sample on an
Apple M1 Max (aarch64), release, on a 300×80 grid — 24 000 cells — unless it says otherwise. Which
harness produced a number matters and is stated where it changed a verdict (§14).

---

## 0. What the engine is

`vitui-engine` is everything that touches the terminal: cells, surfaces, layers, compositing, damage,
the bytes on the wire, input, and the frame clock that decides *when* those bytes go out.

It owns **no layout**, **no widget**, **no reactivity**, and it never iterates application data.
"The engine only draws" is the short form; the form that survives an argument about input is the one
above, and it is the boundary ticket 12 settled: *the engine is everything that touches the terminal;
the runtime is everything above it, including the API components see.*

Two readers, and they are not the same person — this is ticket 12's real finding and it dissolves
what looks like a tension between requirements 4 and 10:

- The **runtime author** reads all of this document, holds `Screen`, drives the loop.
- The **component author** reads none of it. They hold the runtime's draw context and never name
  `Screen`, `Wake`, `LayerStack`, a thread, a clock or a damage rectangle.

One invariant governs everything below and is the sentence the engine's cost model is written in:

> **Frame cost is proportional to visible cells, never to data volume.**

It is checked at both ends of the pipeline: a component drawing against `visible_rows()` costs 54 µs
flat at 1 000, 100 000 and 1 000 000 rows (05), and a 1k-row list and a 1M-row list showing the same
80 rows produce identical damage, identical run counts and identical emitted cells (07).

---

## 1. The workspace

```
crates/vitui-engine       cells, surfaces, layers, compositing, damage, frame writer, input
                          └ crossterm behind a seam: raw mode, input, capability detection
crates/vitui-runtime      layout, identity, focus, hit-testing, routing, key maps, theming,
                          overlays, the data contract — no scene tree, no reactivity
crates/vitui-components   windows, panels, charts, lists, trees, forms, pickers
crates/vitui              facade re-export
crates/vitui-alloc-probe  dev-only counting global allocator          (publish = false)
crates/vitui-bench        round-robin minimum-of-N timing harness     (publish = false)
```

Only the first is this spec's subject. The others appear where the seam touches them.

**The dependency policy is a decision and it is enforced rather than reviewed.** `vitui-engine` takes
`crossterm` plus build-script-generated UCD tables and nothing else; `vitui-runtime` takes nothing at
all. `deny.toml` carries it as `{ name = "crossterm", wrappers = ["vitui-engine"] }`, so any other
crate adding crossterm meets `error[banned]` instead of a code review, and as outright bans on
`futures-core`, `futures-util`, `tokio` and `async-std` — the crates that signal an async runtime has
arrived.

Two corrections to the policy as originally written, both from building it:

- **`mio` and `signal-hook` are accepted.** They arrive through crossterm's `events` feature on every
  platform, not through `event-stream` as the policy assumed. `events` *is* the input parser and is
  the entire reason the dependency exists.
- **`criterion` is out, and the reason is not taste** (13). It reaches `futures-core` and
  `futures-util` through plotters → web-sys → js-sys, which `deny.toml` bans by name; `cargo deny
  check` was red on `master` because of it. The engine's dev graph goes from 70 crates to 20.
  `crates/vitui-bench` replaces it and the methodology lives in its rustdoc.

crossterm is used for **input and terminal mode only, never for output**, and appears in no public
signature. Its output layer formats through `fmt::Write` and cannot satisfy the zero-copy budget.
See [ADR 0001](../../docs/adr/0001-crossterm-behind-a-seam.md).

---

## 2. The frame, as a sequence

Everything else in this document hangs on this order.

```
app thread
│
├─ wait() -> Wake            the frame clock gates HERE, not at present — ADR 0004      §7
│                            Wake::{Input, Posted, Deadline, Quit}; Quit is never paced
│
├─ next_event() -> Option<Event>          drain the input queue                          §9
│
├─ layers()                  add / remove / move layers; view(id) to draw into one       §5
│  └─ the verbs              text · set · fill · restyle, marking damage as they write   §4 §6
│
├─ set_mouse(level) · set_cursor(caret) · request_wake_at(when)                          §9 §7
│
└─ present() -> Presented    composite the damaged rectangles bottom-up                  §5
                             pack runs + their cells + the side tables into a lease      §3 §7
                             re-check the size atomic; discard the frame if it moved
                             clear damage; submit into the one-slot mailbox
                                        │
render thread ──────────────────────────┘
   quantise into the run scan · compare against the mirror · verify one scroll candidate
   emit · one write() · update the mirror                                                §8 §10

input thread
   blocking read on the tty · parse · stamp an Instant · post through the WakeHandle     §9
```

Six properties of this sequence are forced rather than chosen.

1. **The clock gates `wait`, not `present`.** Ticket 09 put the pacing gate on the app thread so a
   doomed frame is never *composed*; that is half a solution, because by the time `present` refuses,
   the runtime has already run its layout, its reactivity and every drawing verb. Measured against an
   event storm at 1000 Hz with a 167 µs frame standing in for one iteration of runtime work: gating
   at `present` runs 800.4 iterations a second to show 114.7 frames — **14% useful, 11.4% of a core
   wasted** — against 114.5 for 114.5 and no waste. And at a sparse event rate it *delivers fewer
   frames*, **83.8 fps against 110.8**, because it can only paint at instants when an event happens
   to arrive. The cheaper scheme is also the smoother one, so there is no trade.
   See [ADR 0004](../../docs/adr/0004-the-frame-clock-gates-the-wait.md). (12)
2. **Compositing runs on the app thread, before the handoff.** Layer surfaces never cross the thread
   boundary; only the packed damage does. That is what keeps the render thread free of screen-sized
   application state and makes resize, shutdown and panic small. (06, 09)
3. **Damage is marked by the verbs and cleared by `present`.** The runtime neither declares it nor
   clears it. A forgotten clear produces a frame that repaints for ever, so it is an invariant rather
   than a chore — and it is also why nothing above the engine can force a full repaint. (07, 12)
4. **`present()` is the only exit**, and it owns the whole composite–pack–pace–submit sequence. Ten
   public names disappeared from the seam when it took that job, without a capability being lost. (12)
5. **The mailbox lock is never held across a syscall.** This is requirement 1 reduced to one
   sentence; everything else about the handoff is consequence. The critical section is 47 ns. (09)
6. **A lease is never invalidated; the frame it produced may be discarded.** The authoritative size
   is one packed `AtomicU32` written by the input thread, sampled at frame start and re-checked at
   submit. Writing a 300×80 frame into a terminal that is now 120×40 wraps and scrolls, which is
   worse than a missing frame. (09)

**The engine does not own the loop.** It hands out `wait`, the verbs and `present`, and the runtime
drives. That is what makes a TEA pump and a signal graph both able to sit on it — discharged by
building both against one unmodified seam rather than by argument (12).

---

## 3. The cell, the buffer, and the handle tables

### The cell

**Array-of-structs, 16 bytes, a `u32` grapheme handle and a style packed into exactly one `u64`.** (04)

```rust
#[repr(C)]                  // 16 bytes, align 4, no padding hole
pub struct Cell {
    pub grapheme: GraphemeId,   // u32
    pub style: Style,           // u64
    _reserved: u32,             // keeps the cell 16 bytes and the layout honest
}
```

The style word is the whole finding. It is the serializer's inner comparison, and its cost is decided
by one thing — whether the style fits in a machine word:

| style compare, full-screen scan | time |
|---|---|
| masked `u128`, shift mask | 47.8 µs |
| masked `u128`, AND against a constant | 48.0 µs |
| 12-byte `Style`, derived `PartialEq` | 19.5 µs |
| interned `u32` handle | 5.74 µs |
| **inline `u64` style** | **5.45 µs** |

Two expectations died here and both are worth keeping. **Ghostty's masked-integer trick loses 2.5× on
aarch64**, because a 128-bit value lives in a register pair and masking across the boundary costs
several instructions. And **the interning win was never about interning**: the interned arm's fast
scan is matched by an inline `u64` at zero interning cost, while writing a full screen costs 522 µs
with per-cell style interning against 93 µs, and compositing costs **559 µs against 9.3 µs** — over
half the frame budget, because every composited cell must un-intern, blend and re-intern. That is
ghostty #1873's shape reached by measurement rather than by reading their tracker.

```
bit  63     extended — 0: colours inline; 1: bits 51..0 are a handle into the extended-style table
bits 62..52 attrs (11): bold, dim, italic, reverse, blink, strikethrough, conceal,
                        overline, + 3-bit underline style
bits 51..26 fg  [tag:2][payload:24]   tag: 0 default, 1 indexed, 2 rgb, 3 reserved
bits 25..0  bg  [tag:2][payload:24]
```

Exactly 64 bits with nothing spare. Two things gave, both cheap: **SGR 6 rapid-blink is dropped**
(terminals that implement it fold it into slow blink), and **underline colour and hyperlinks move
behind the `extended` bit**. Keeping an `extras` handle inline instead costs 5.45 → 13.3 µs on the
serializer scan, so the fold pays for itself 2.4×. Because the table is deduplicated, two extended
cells still compare correctly with one `u64` compare, and the interning cost lands only on the <1% of
cells that carry a hyperlink or a coloured underline.

### The grapheme handle

**Bit 31 is a flag over the low 31 bits, not a third range** (19):

```
bit 31         clear: one column.  set: two columns, and a CONTINUATION follows.
bits 0..=30    0x0000_0000..=0x0010_FFFF   a Unicode scalar value
               0x0011_0000..=0x7FFF_FFFE   a cluster id in the intern table
               0x7FFF_FFFF                 EMPTY — a non-opaque content layer's "skip this cell"
                                           sentinel  (06)

0xFFFF_FFFF    CONTINUATION — the second half of a double-width pair, which is exactly
               `bit 31 | EMPTY` and needs no range of its own
```

Read as three ranges, a wide **scalar** has no representation: `漢` is U+6F22 and is two columns, so
it would have to be interned to be a wide head — contradicting the next paragraph for every cell of a
screen of CJK, and contradicting §4's measurement that a full screen of CJK is *cheaper* than Latin,
which is only true if no table is touched. A wide head and its narrow twin are one bit apart.

**A cell holds an interned extended-grapheme-cluster handle, never a `char`.** Forced from two
directions (02): UAX #29 rules GB9c, GB11 and GB12–13 group multiple codepoints into one cluster, and
`unicode-width`'s own rewrite had to introduce a 16-bit context bitfield because per-codepoint width
is insufficient in the presence of VS15/VS16 and ZWJ.

**Interning is identity for single scalars.** A cell holding `a` or `漢` *is* its own handle, with no
table and no hash: 3.6 ns for a scalar, 25.8 ns for a ZWJ family emoji, zero allocations either way.
That is what makes "never a `char`" free on the common path rather than a per-cell tax, and it is
also what makes the packet's cluster arena nearly free (§8).

**Tables are generated by build script from UCD**, as a three-stage trie copied in shape from
`unicode-width`'s generated source: `WIDTH_ROOT` 256 B + `WIDTH_MIDDLE` 1 280 B + `WIDTH_LEAVES`
5 952 B ≈ **7.3 KiB** core, O(1) lookup; ~20–30 KiB of static `.rodata` for all three properties the
engine needs. `unicode-segmentation`'s coarse-index-plus-binary-search hybrid is larger and slower and
is not copied. **Our tables stay authoritative** whatever mode 2027 answers (§10).

### Width, and the five repair rules

The invariant, asserted over three adversarial overwrite passes on a full screen: *a `CONTINUATION`
never appears without a wide head immediately to its left, and a wide head is always followed by a
`CONTINUATION`.* The write verb enforces it by repairing before writing:

1. **A write landing on a `CONTINUATION`** blanks its head at `x-1`, then proceeds. Both cells damaged.
2. **A write landing on a wide head** blanks its continuation at `x+1`, then proceeds. Both damaged.
3. **A blanked half keeps the old cell's style, not the writer's.** The background is what the eye
   notices; a half-erased glyph that also changes colour reads as a bug even when the text is right.
4. **A wide glyph that does not fit** — last column, or the clip region ends — **is written as a
   space.** Never draw half a glyph.
5. **A wide glyph landing across an existing pair repairs at both ends** before writing.

The rules hold at three edges, and the third was found late: a surface edge (04), a *clip* edge (05),
and a *layer* edge at composite time (17, 06). §5 states the layer half, which needs one extra rule
because an operator recolours rather than overwrites.

**The clip edge is the one that had to be settled by asking a terminal**, and architecture ticket 20
is the record. The rules reaching a clip edge means a repair blanking a cell *outside* the clip, which
reads as §4's *a child cannot widen its clip* being false. The two sentences cannot both hold there,
and the implementation chose §4 for a session — leaving the invariant above silently false on any
surface a `View::child` clip bisected. Production ticket 06 closed it the other way, on evidence
rather than on argument:

> `conform/`'s scene 04 prints `AB漢CD` and then one cluster over one half of the wide glyph.
> **kitty 0.48.2, Ghostty 1.3.1 and tmux 3.7c all blank the orphaned half themselves, in both
> directions, and none of them has a clip to consult.** A surface holding a wide head with no
> continuation is a picture no terminal can be made to show, so the mirror would believe a cell the
> screen does not have, damage tracking would never repaint it, and the artifact would stand until
> something else wrote there — which is the corruption this section is about.

The same scene found the sharper half of the argument, which nobody had predicted: the three families
**disagree about what the blanked cell wears**. kitty keeps the orphan's own background; Ghostty and
tmux blank it to the SGR state in force. So a repair left to the terminal is not merely one the mirror
does not know about, it is one whose result differs by terminal, and no mirror state could be right on
all three. Rule 3 above is the engine's answer and it is serialised explicitly, so what the terminal
would have done unaided never arises.

So the invariant is **true as written**, at all four edges, and §4 carries the exception — stated
there rather than here, because it is §4's sentence that now has a boundary on it.

### Memory, and the layouts that lost

| | one 300×80 surface | front + back + a flattened cache |
|---|---|---|
| **chosen (AoS 16 B)** | **375.3 KiB** | 1 125.9 KiB |
| SoA, 12 B/cell | 281.6 KiB | 844.7 KiB |
| 8-byte interned cell | 188.1 KiB | 564.4 KiB |

**SoA measured 3.5% better end to end and does not pay for splitting every verb across two arrays.**
Five-array SoA is the worst of everything measured. A 12-byte packed AoS cell saves 25% of memory and
*loses* on compositing (11.43 against 9.28 µs) because the stride straddles cache lines — cell size
and array splitting are separate effects and that arm exists to prove it. The third column is
vestigial: there is no flattened cache (§5, [ADR 0024](../../docs/adr/0024-no-flattened-layer-cache.md)).

**Row cache-line alignment buys nothing — do not pad rows.** At 16 bytes a 64-byte line holds exactly
four cells, so rows are line-aligned iff the width is a multiple of four; packing at width 300
(aligned) against 297 (every row at a different offset) is **4.845 against 4.847 µs** (09).

### The handle tables

Two tables — the grapheme interner for multi-scalar clusters, and the extended-style table for the
colours that did not fit in the style word. **There is one set per layer stack**, minted by `attach`,
and the drawing verbs reach them through the draw context — which on the engine's side of the seam is
the `View`, so no public signature names one.
See [ADR 0011](../../docs/adr/0011-the-handle-tables-belong-to-the-engine-and-the-packet-carries-copies.md). (16, 19)

One handle space means a handle crossing a surface boundary needs no translation, because there is
nothing to translate it *to* — and that is what keeps compositing a `copy_from_slice`:

| composite, 300×80 | realistic 1% extended | hostile 100% |
|---|---|---|
| **one engine-wide table** (`copy_from_slice`) | **6.16 µs** | **5.79 µs** |
| per-surface, remap per cell | 30.04 µs | 265.62 µs |
| per-surface, remap memoised on the handle | 30.24 µs | 181.74 µs |

**4.9× on a realistic layer and 46× on a hostile one**, with the remap arm charged generously — the
`Remap::build` pass is itself O(source cells) and is excluded. The steelman memo never buys the memcpy
back.

The invariant that replaces the remap, in the words that survive a donated surface (19): ***every
surface in a layer stack speaks that stack's handle space.*** It holds by construction for
`add_content`, which mints the surface, and by renumbering at donation for `add_content_with`, which
does not. Either way it is worth one debug-only assertion on the composite path rather than a runtime
check.

**A surface outside a stack carries its own interner, and usually an empty one** (19).
`Surface::new` and `Surface::root` are public (§4), and a `View` obtained that way has no engine to
reach through; the table stays empty until a multi-scalar cluster is written through that door, and an
empty one holds no allocation. Latin, CJK, box drawing and every single-scalar emoji are their own
handles, so the common off-screen surface never touches it. `Surface` stays `Send`, `'static`, `Arc`-free and free
of interior mutability, which a boxed table is.

**`add_content_with` renumbers once, at donation.** A donated surface whose interner is empty moves in
as-is.
Otherwise the stack walks it once, re-interns each handle above `0x0010_FFFF` into its own table and
rewrites the cell, marking no damage — a new layer already damages its whole rectangle. This is a
scene topology change, where allocation is permitted, and it is never inside a frame. **The cost is
owed rather than measured** (§15): the nearest measured neighbour is the eviction sweep below, the
same shape of work at 58.88 µs for one screen, and expectation is not measurement.

**Every side-table round trip is per style word, not per cell.** Cells in a run are contiguous and
share a `u64`, so a one-entry memo on the previous style word turns a per-cell round trip into a
per-distinct-style one. It is the same trick three times — `restyle`, `Mix`, pack — and it is what
keeps ticket 04's 559 µs unreachable:

| `restyle`, full screen | plain | realistic 1% | linked 100% |
|---|---|---|---|
| inline mask that skips extended cells (*wrong*) | 13.02 µs | 14.22 µs | 16.30 µs |
| correct, per cell | 24.51 µs | 28.42 µs | 289.19 µs |
| **correct, memoised** | **12.65 µs** | **17.47 µs** | **75.49 µs** |

The memo is **free where nothing is extended** — 12.65 against 13.02 for the mask that does less.
Two fast paths were built beside it and both refused: a per-cell branch (it breaks the mask loop's
vectorisation) and a surface-level "contains no extended cell" gate (indistinguishable from the memo
alone). The memo is the whole implementation.

**Two silent wrong answers are deleted rather than documented.** `Style::with_bg` on an extended style
returns it unchanged, so a selection highlights everything except the hyperlink (14's H3);
`Style::with_fg_bg` keeps bits 62..52 and rewrites the rest, so it clears bit 63 and overwrites the
52-bit handle with colours — **a shadow falling across a hyperlink deletes the hyperlink**, with no
error and nothing to see. Both methods are **removed**, and the compiler is the right place for that
error. What replaces them is a verb that owns the table:

```rust
pub enum Link<'a> { None, Uri(&'a str) }

pub struct Restyle<'a> {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub set: u16,                  // attributes to add
    pub clear: u16,                // attributes to remove
    pub ul: Option<Color>,         // Some(DEFAULT) clears the underline colour
    pub link: Option<Link<'a>>,    // Some(Link::None) clears the hyperlink
}
```

`restyle` takes this descriptor and not `impl Fn(Style) -> Style`, because a closure receives a handle
it has no table for — which is exactly how the ticket-05 prototype ended up returning extended styles
untouched. The contract, written down rather than implied: **`restyle` rewrites what the descriptor
names and preserves every channel it does not, on either side of the extended bit.** A descriptor that
clears both extended channels puts the cell back inline — *extended is a cost, not a state* — which is
a live pressure valve on table growth. `Style` stays `Copy`, self-contained and free of any table.

**The URI travels at the drawing verb, and `LinkId` is internal** (21). `Restyle::link` names a URI;
the `View` interns it into whatever handle space it draws into, which is exactly what `text` has
always done with a grapheme cluster — and it was the *only* difference between the two halves of one
table set. `Link::None` clears the hyperlink, which is what the field's `Some(0)` once meant and
then `Some(LinkId::NONE)` after it.

Ticket 21 was asked whether a standalone `Surface` should gain a mint of its own, because it can
carry a hyperlinked cell and cannot mint the id one needs. **Every answer that adds a second mint
makes a worse failure reachable than the one it fixes.** `Links::mint` numbers from `uris.len() + 1`,
so the first id in *every* table is 1: a stack-minted id landing in range of a non-empty donor table
is silently rewritten at donation to **the donor's URI at that slot** — not a deleted hyperlink and
not a preserved one, but a *different* one, which is the class of §5's shipped prototype and of the
`Style::with_fg_bg` removal, and harder to see than either. Closing that honestly needs a handle-space
stamp inside `LinkId`: four bytes to eight, `ExtStyle` sixteen to twenty-four, and a rehash of the one
table that can grow without bound — machinery bought to *detect* an error the rest of this crate
answers by making the value unobtainable.

Three things fall out, and none of them is the question that was asked:

- **The donation walk becomes total.** Every `LinkId` in a donated surface's cells was minted by that
  surface's own table, so `remapped`'s not-in-range arm is unreachable by construction and is an
  `expect` beside the grapheme one.
- **The public surface loses the one handle it had**, so §12's refusal 11 — *no cells, no grapheme
  handles, no style bits* — holds with no exception beside it, and ADR 0023 has one fewer to argue.
- **The runtime's `Ctx::link` becomes unnecessary rather than unblocked.** It was specified by the
  runtime and recorded as unimplementable: minting needed `&mut Screen`, and a `Ctx` holds a `View`
  borrowed from it. `Repaint` carries the URI and `lower` hands it to `Restyle`.

The price, stated: `Restyle` gains a lifetime, so §12's *one lifetime in the whole surface* became
*two* — and elision covers every call site, because a descriptor is built inline; and **one hash probe
per verb call that names a URI** — not per cell and not per style word, since it resolves once above
`restyle`'s memo. Unmeasured, and it is §15's seventh owed figure.

### Growth, eviction, and the one thing a sweep breaks

**The clusters are not the problem and the styles are, and the difference is who writes them.** A
cluster handle comes from text the application supplied. An extended style handle comes from a drawing
verb *and from the compositor*, because a `Mix` over an extended cell produces a style that has never
existed. Measured on 300×80 of hyperlinked text (96 distinct extended styles) under one operator:

| | entries created | per frame |
|---|---|---|
| a settled modal dim, 120 frames | 96 | 0.8 |
| a fade in, 120 frames | 11 484 | 95.7 |

**A static operator converges after one frame.** The leak needs an operator whose `amount` is
changing: a 300 ms fade costs about 1 700 entries, a permanently pulsing dim over a hyperlinked page
about 5 700 a second, and that is the case eviction exists for.

**Eviction is a mark-and-compact sweep**, run where allocation is already permitted — a scene topology
change, or a high-water mark on the table — and **never inside a frame**. It walks the live surfaces
once with one marker slot per table entry, rebuilds each table in table order, rewrites the handles,
and **marks no damage**, because the cells still say the same thing. One screen is 58.88 µs; twenty
layers is 1.17 ms.

Compacting in table order matters: a sweep that frees nothing *below* a live entry does not renumber
at all. When it does renumber, exactly one thing breaks — the mirror, which is the only reader that
compares handle identity across frames — and the answer is one flag, `Packet::repaint`, costing one
full frame on the render thread at a moment when the engine is already doing topology work.

**The "second reader" this was feared for does not exist**, because a packet holds no handle. The app
thread may sweep while the render thread is inside a 200 ms `write`.

The general rule, and every future change to the packet has to pass it:

> **Nothing the render thread compares across frames may be derived from a position.**

---

## 4. Surfaces, views and the drawing verbs

**A `Surface` owns cells and damage; a borrowed `View` draws into it; three verbs carry everything.** (05)

```rust
pub struct Surface { /* cells, damage */ }          // Send. No Arc, no interior mutability, no lifetime.
impl Surface {
    pub fn new(w: u16, h: u16) -> Surface;
    pub fn size(&self) -> (u16, u16);
    pub fn root(&mut self) -> View<'_>;             // the only way to get a View
}

pub struct View<'a> { /* origin, clip rect, content offset, size — no cells */ }
impl<'a> View<'a> {
    pub fn child(&mut self, r: Rect) -> View<'_>;           // clip intersects; can only shrink
    pub fn scrolled(&mut self, dx: i32, dy: i32) -> View<'_>;

    pub fn size(&self) -> (u16, u16);
    pub fn visible_rows(&self) -> Range<i32>;               // the culling query
    pub fn visible_cols(&self) -> Range<i32>;

    pub fn text(&mut self, x: i32, y: i32, s: &str, style: Style) -> Written;
    pub fn set(&mut self, x: i32, y: i32, cluster: &str, style: Style) -> Written;
    pub fn fill(&mut self, r: Rect, cluster: &str, style: Style);
    pub fn restyle(&mut self, r: Rect, d: &Restyle);
}

pub struct Written { pub cells: u16, pub bytes: usize, pub stop: Stop }
pub enum Stop { Complete, Clipped, Offscreen }
```

### Why a view rather than a painter

Two genuinely different shapes were written and both compile against an identical write path, so the
benchmark compares shapes and not implementations. **They cost the same** — 54.1 against 54.3 µs on
full-screen text, 54.5 against 56.3 at nesting depth 8 — so the choice rests entirely on what each
makes *impossible*:

1. **Zero allocation is structural, not disciplined.** A `View`'s clip stack **is** the call stack, so
   there is nothing to allocate. The painter's is a `Vec` that allocates at construction and **again,
   mid-frame, whenever nesting outgrows the capacity someone guessed at.**
2. **A child cannot widen its clip.** A view asking for a 400×400 rectangle inside a 6×2 parent
   touches exactly the parent's 12 cells. In the painter shape a callee holds the whole surface and a
   `pop_region` it can call more times than it pushed, which is a component painting over its caller's
   chrome. For a library other people write components against, a seam defended by convention is not
   defended.

   **The one exception, and it is stated rather than left false** (architecture ticket 20): §3's
   repair rules 1, 2 and 5 are bounded by the *surface*, not by the clip, so a write that orphans
   half of a double-width pair blanks the other half even when that half is outside the clip. The
   argument above survives it, because what §4 forbids is a caller *directing* a write outside its
   clip and this is not one:

   - the only reachable outcome is the surface's **ground**, keeping the cell's own style (rule 3).
     There is no way to put content through it;
   - it reaches exactly **one column**, and only the column adjacent to a cell the caller
     legitimately wrote;
   - it fires only when that column holds the other half of a pair the caller's own write has
     already destroyed.

   So the licence a one-column `child` buys is one cell of ground in a column the parent's glyph no
   longer occupies anyway. **The seam is defended by what the operation can express rather than by
   convention**, which is this clause's own standard. `restyle` is where the sentence still bites
   unchanged: restyling a bisected pair whole *would* be a caller-directed write outside the clip and
   nothing forces it — an unrestyled pair is still a pair — so that verb shrinks its span instead.
   §5 gives the same answer one level up, where a layer's repair has always reached outside the
   layer's own rectangle.
3. **Forgetting to pop is not expressible.** Scope does it.
4. **The ergonomic argument for the painter is smaller than expected.** Lifetime elision means a
   component signature carries no lifetime in either shape.

Kept on the record because it may return: the painter shape is trivially object-safe, so `dyn Painter`
would give a recording painter, a null painter and a test double for free. Nothing in the engine wants
that today.

### Three verbs, not four and not six

**`text`, `fill`, `restyle`.** `set` is `text` with a one-cluster string — the same call, kept only
because `set(x, y, "▀", st)` reads better at a chart's call site. Lines and boxes are **not**
primitives: a box is four `fill`s on degenerate rectangles plus four `set`s, and it does not appear in
a profile. Each of the three earns its place on a number:

| verb, same 24 000 cells | time |
|---|---|
| `text` — 80 verbs of 300 cells | 150.4 µs |
| `text` — 24 000 verbs of 1 cell | **290.6 µs** |
| `text` — full screen of CJK | 138.5 µs |
| `fill` | **8.79 µs** |
| `restyle` | 12.65 µs |

- **`fill` against `text` on the same cells is 17×.** A `fill` is a `slice::fill` of a 16-byte `Copy`
  with two edge repairs; expressing it as repeated text throws that away.
- **`restyle` against redrawing: moving a selection bar one row costs 205 ns against 1.36 µs, 6.6×.**
  Without it, changing a background means re-segmenting UTF-8 and re-interning every cluster on the
  row. This verb was not in the ticket's list; the measurement is why it is in the answer.
- **A full screen of CJK is not a tax** — 138.5 against 150.4 µs, cheaper, because half as many
  clusters cover the same columns. **This did not reproduce, and the shipped number is here rather
  than left to be rediscovered** (impl 06): 258 µs against 136 for Latin, **1.90x**, on the same
  24 000 columns. The mechanism the claim rests on is intact and gated by equality — a wide scalar is
  its own handle and a full row of CJK interns nothing — so what is left is segmentation itself, at
  **14.3 ns a code point against 3.1 ns** for one that takes the ASCII fast path. Two three-stage
  trie walks per code point, `BREAK` and `AUX`, are the substance of that; merging them into one
  8-bit property table is the candidate and it belongs to §3's tables rather than to the verbs. Both
  numbers are inside the 1 ms full-screen budget, and neither is a damage-tracked frame.

**The granularity rule, which constrains every component ever written against this API: 80 verbs of
300 cells cost 150.4 µs; 24 000 verbs of one cell cost 290.6 µs for the same cells — 1.93×.** The
verbs are span-shaped by default and a component that walks cell by cell is choosing to pay double.
The rule is about **cells per verb, not verbs per frame**: a sparse braille chart lighting 200 cells
sees no penalty at all, because per-cell and per-run verbs touch nearly the same small number of cells
and the overhead has nothing to amortise over (14).

Two verbs were built specifically to be refused, and are recorded so that "we priced it" is
distinguishable from "we did not think of it" (14):

- **`fill_with(rect, cluster, |x, y| style)`** — a gradient in one verb. **2.5× faster on a 40-cell
  progress bar and 8% *slower* on a full screen**, for an absolute stake of at most 1.8 µs. The
  ergonomic win it was meant to deliver belongs to the runtime, where `gradient_bar` is six lines
  wrapping a loop of one-column `fill`s.
- **A write-time equality filter** (`Surface::set_eq_filter`) — free in time, 14.49 against 15.05 µs,
  and **defeated by clear-then-draw**: immediate mode blanks before it draws, so the filter sees
  blank-over-text and then text-over-blank, both changes. The same idea at pack time is worth 3–30×
  and is where it lives (§8).

### Zero-allocation text

`&str` reaches cells with no intermediate allocation, asserted rather than assumed. The segmenter is a
**borrowing iterator yielding `&str` slices into the caller's buffer**, with an ASCII fast path; never
a `Vec<&str>`. A whole frame — a clear, a scrolled viewport, 80 rows of text, a `restyle`, a ZWJ family
emoji, a box-drawn popup and CJK — allocates **0 times**.

### Clipping, viewports, and why a free discard is not enough

The clip is one absolute rectangle per view, intersected on `child()`. A verb tests the row once and
then works against a column range — one compare per verb for the reject, not one per cell. An offset
viewport is `scrolled(dx, dy)`: content coordinates in, absolute out, one add per verb. libvaxis's
accumulate-only-negative-offsets trick is not needed once the clip is an absolute rectangle, because
the intersection does the same work and also survives a child whose parent is scrolled.

| discard | time |
|---|---|
| 24 000 text verbs, all rejected | 87.0 µs — **3.6 ns each** |
| 24 000 text verbs, all accepted | 12.97 ms — 540 ns each |
| 300 columns written + 300 clipped away on every row | 150.5 µs — same as writing 300 |

**A discarded write is 149× cheaper than an accepted one, and that is categorically insufficient.** At
3.6 ns each, a component drawing all 1 000 000 rows of a tree burns **3.68 ms in pure rejection —
3.7× the entire frame budget — before formatting a single string** (29.9 ms with the formatting). The
primitive that makes the tree affordable is the query:

```rust
for i in v.visible_rows() { v.text(0, i, &row(i), st); }
```

which is **54 µs at 1 000, at 100 000 and at 1 000 000 rows — flat to three significant figures.**

`visible_rows()` does not breach [ADR 0002](../../docs/adr/0002-layout-lives-outside-the-engine.md).
It measures nothing, sizes nothing and asks nothing of the caller's data: it reports which of the
coordinates the caller already chose fall inside the window the caller already brought.

### Error handling: clamp and discard, one rule, everywhere

See [ADR 0022](../../docs/adr/0022-drawing-verbs-clamp-and-discard.md).

- **Out of bounds** — discarded silently, `Written::NONE`. No panic, no `Result`, not even a
  `debug_assert`: virtualised components write far outside a surface as a matter of course, and that
  is normal traffic rather than an error.
- **Invalid UTF-8 — the error class is deleted rather than handled.** The verbs take `&str`, so it
  cannot be constructed. This is why the seam takes `&str` and not `&[u8]`, and it is deliberately
  asymmetric with the input boundary, which owns bytes (§9): *at the drawing verbs the engine may
  demand well-formed input from its caller; at the input boundary it may demand nothing.*
- **A write landing mid-glyph** — the five repair rules of §3, unchanged.
- **The escape hatch is `Written`, not `Result`.** A caller that needs to know where a verb stopped is
  told — `cells`, `bytes` consumed, and why — at no cost to the caller that does not. `bytes` is an
  addition to libvaxis's `PrintResult`: a caller that wraps or paginates would otherwise segment the
  string a second time.

### The cell is never visible

`Cell`, `GraphemeId` and the packed style bits appear **nowhere** in a public signature. Verbs take
`&str` and an opaque `Style`. See [ADR 0023](../../docs/adr/0023-the-cell-is-never-visible-in-the-public-api.md).

That is what let the handle tables be placed without touching a caller, and it costs the engine one
capability, stated here rather than left to be discovered: **a caller cannot read back what is already
on screen.** A sub-cell chart therefore keeps its own `w × h` shadow bitmap, at one byte per cell of
component state, because two dots landing in one cell have to be merged before the cell is written
(14). That is the right refusal and the cost is recorded, not contested.

### Ownership, threading, and the claim that was refuted

A caller **leases** a surface, draws into it, and submits it. `Surface` is `Send` and contains no
`Arc`, no interior mutability and no lifetime, which is what lets the app thread prepare frame N+1
while the render thread paints frame N. `Surface` staying `Send` is deliberate: drawing a heavy
off-screen surface on a worker and donating it through `add_content_with` is legitimate.

**That is the second door to a `View`, and it is why the interner is per stack rather than per
engine** (19). A worker cannot hold `&mut` to the app thread's interner — not by convention, by the
borrow checker — so a surface drawn outside a stack interns into its own table and `add_content_with`
renumbers it into the stack's, once, at donation (§3). The alternative was to delete `Surface::root`
and with it off-thread drawing, the reason `Surface` is `Send` at all, and `add_content_with`'s stated
purpose; it is recorded in ticket 19 as still
available at the price of one deprecation, should the donation pass measure badly at implementation
ticket 10.

> **Ticket 05's threading claim was wrong when written, and ticket 18 refuted it by test.** "`View`
> borrows and therefore cannot be sent anywhere" holds only against `thread::spawn`, whose `'static`
> bound was doing the work; `Surface: Send` implies `&mut Surface: Send` implies `View: Send`, and a
> `thread::scope` closure that draws through a `View` compiles. The fix is a private
> `PhantomData<*const ()>` field in `View`, zero-sized, invisible to callers, verified in both
> directions.

The narrow price: `View: !Send` forecloses splitting **one** surface across threads. Drawing into
*separate* surfaces on separate threads is untouched.

### `split_h` / `split_v` — closed as not needed, with the price recorded

The case never materialised: in a single-threaded immediate-mode pass no scenario was found where two
siblings must be held at the same *moment* rather than one after another, and `child()` already gives
each a clip it cannot widen. None of the five hostile components missed it. Refusing it is recorded
with what building it would have cost (12):

1. A **row-band** split is expressible safely — but only after redesigning the hot type. Every write
   verb needs the cell buffer, the per-row damage *and* the grapheme interner at once, so `View` would
   stop holding `&mut Surface` and start holding disaggregated borrows with the interner behind
   interior mutability.
2. A **column-band** split is **not expressible in safe Rust at all.** Cells are row-major, both panes
   share every row, and safe Rust splits a slice only along its own axis. It needs the crate's first
   `unsafe`, in its hottest code. Asserted rather than argued: two `E0499`s against the real `Surface`.
3. **Sibling content layers** work today and cost nothing per frame, but inflate `n` in the layer
   stack from "windows, popups, shadows" to "every pane" — which contradicts the glossary's *nothing
   else is a layer* and turns a structure chosen for a dozen entries into one holding a widget tree.
4. A scoped sequential `panes(&[r], |i, v|)` is safe and cheap and expresses only what `child()`
   already expresses, one call less clearly.

### `&mut` on the verbs, kept, with the cost of removing it recorded

The map's precedence rule 4 prefers neither ownership nor a mutable borrow anywhere either can be
avoided, and yields where avoiding it costs materially. Here it costs materially. Verbs on `&self`
need the cells behind `UnsafeCell`, which means two views over overlapping regions can race on the
same cell with nothing to stop them — a compile-time guarantee replaced by an unchecked contract, in
the hottest code in the engine, in a crate that contains no `unsafe` at all. `RefCell` puts a branch
and a panic path on each of the 24 000 writes a full-screen frame makes. `&mut` stays.

### The caret, and the text tables

Two additions, both found by writing a form against the seam, both additive, and neither moving
ADR 0002's line. See [ADR 0005](../../docs/adr/0005-the-engine-owns-the-caret-and-exports-its-text-tables.md). (14)

**`Screen::set_cursor(Option<Cursor>)`** — position and shape, in screen coordinates, applied by
`present` after the frame's last write, which is the only moment at which it is correct and a moment
only the engine has. Blinking is the terminal's. The runtime translates from layer coordinates, which
it can, because it brought the layer's rectangle.

The alternative is a software caret, and it was measured: one `restyle` of one cell, toggled, costing
**two wakeups a second for as long as anything has focus** — 7 200 an hour on a screen where nothing
is happening. Ticket 09's measured idle and standing requirement 11's "where it can idle it must idle
completely" do not survive a text field, and a form is not an exotic component. The terminal's own
caret blinks for free, in the terminal's process, at the user's configured rate, and is the only thing
a screen reader or an IME can follow.

**Grapheme segmentation and display width as functions over a caller's `&str`** — a borrowing iterator
yielding `(&str, width)`, and `width_of(&str)`. Both iterate *the caller's string*, exactly as `text`
does, and iterate no application data. The cost of refusing: either the runtime ships a second copy of
the UCD tables — and the two disagree the day their Unicode versions differ — or every text component
is wrong on emoji. Concretely, the caret column of a family emoji is **5 by the engine's rule and 10
by a plausible `chars().count()`**: five columns of desynchronisation on one character.

---

## 5. Layers and compositing

**Two layer kinds and one operator.** (06)

```rust
pub struct LayerId(u32);

pub enum LayerKind {
    /// Carries cells. `opaque == false` allows `EMPTY` cells, skipped at composite time.
    Content { surface: Surface, opaque: bool },
    /// No cells: a rectangle and a transformation of whatever is already there.
    Operator(Mix),
}

pub struct Layer { id: LayerId, z: i32, seq: u32, rect: Rect, kind: LayerKind }

/// The only operator. Darken is Mix toward black, lifting is Mix toward white,
/// tint is Mix toward anything, and a fade is `amount` moving across frames.
pub struct Mix { toward: Color, amount: u16 /* 0..=256; 0 is the identity and is skipped */ }
```

A **content layer** carries cells and paints `Replace`; an **operator layer** has no cells, only a
rectangle and a `Mix`. Shadows, liftings, modal dimming, tints and fades are all `Mix`.

The map's three-item list (`Replace`, `Darken`, `Blend(f32)`) collapsed to two mechanisms. `Replace`
is not a blend mode, it is what a content layer does. **`Blend(f32)` as alpha-over is rejected**: a
terminal cell has no alpha, so blending two layers' *glyphs* is not a thing — one of them has to win —
and "a semi-transparent popup" would hand that choice to the compositor, which has no basis for making
it. **A gradient is not an operator either**: it is a `fill` with a varying style, already fully
expressed by §4's verbs, and the compositor never sees it.

**Painter's algorithm is available to us, and that is a dividend of the cell decision.** Textual states
plainly that painter's cannot be used, because it composites *segments* — variable-width runs of styled
text. That constraint belongs to their representation, not to terminals: a fixed-width cell grid with
an explicit `CONTINUATION` sentinel makes depth-order painting sound again (17).

### There is no flattened layer cache

See [ADR 0024](../../docs/adr/0024-no-flattened-layer-cache.md). Issue 03 named the flattened
prefix cache as the one optimisation with no prior art anywhere, ours to invent, carrying real design
risk. **It is closed by refusing to invent it.**

| stack depth, full-screen damage | full-screen content layers | popups, each with its shadow |
|---|---|---|
| 1 | 6.28 µs | 5.03 µs (2 layers) |
| 3 | 20.2 µs | 15.2 µs (6 layers) |
| 20 | 133.5 µs | **107.3 µs (40 layers)** |
| 50 | 372.3 µs | 264.5 µs (100 layers) |

| damaged area, 20 popups + shadows (40 layers) | time |
|---|---|
| one cell (a blinking cursor) | **171 ns** |
| one row of 300 cells | 2.02 µs |
| one popup, 90×14 | 16.7 µs |
| the whole screen | 100.7 µs |

**Cost is damaged area × depth**, and that is the claim the cache would have existed to deliver,
delivered by damage rectangles instead. A full-screen composite is not a "typical damage-tracked
frame"; it is budgeted at 1 ms, where 107 µs is 11%, and even 100 layers stays inside it at 264.5 µs.

This is **not** issue 03's anti-pattern. Its warning was against compositing the whole stack from
scratch with **no dirty-plane list** — notcurses, Zellij and termwiz fail because they have no damage,
not because they have no cache. Damage rectangles *are* the dirty-plane list. The prefix cache is
refused separately and for its own reason: reusing layers `0..k` requires the flattened result of
*that* prefix, so full generality is one 375 KiB buffer per prefix, and any cheaper variant only works
while the split point is stable — which is exactly when the frame was already cheap.

Recorded so nobody re-derives it: **the grilling session refused the cache on a linear extrapolation
that was wrong by 1.8×** (twenty layers ≈ 60 µs, measured 107.3). The decision stands on the correct
budget. This is the second time on this map a plausible extrapolation did not survive measurement;
treat the pattern as established.

### Colour resolution, and the capability the compositor depends on

`Mix` needs channels, so colours resolve at composite time: `default` → the terminal's real colour,
`indexed` → the palette, `rgb` → itself. **The terminal's real default background is queried once at
startup with OSC 11.** See [ADR 0025](../../docs/adr/0025-compositing-depends-on-a-terminal-capability.md).

This is the only place in the engine where a rendering decision depends on what the terminal told us,
and it has to be stated rather than left implicit. Guessing is not neutral: the guess is a dark theme,
and on a light-theme terminal a shadow over a default background comes out *lighter* than its
surroundings — a shadow drawn backwards.

**If the terminal stays silent on OSC 11, cells with a default background are left unmixed.** A shadow
clipped to the explicitly-coloured area is a visible imperfection; an inverted shadow is a bug.

`Mix` must go through the handle table, memoised (§3). That is a correctness requirement — the shipped
prototype deleted a hyperlink under a shadow — and it is *cheaper* afterwards, not dearer:

| full-screen `Mix` | plain | realistic 1% | linked 100% |
|---|---|---|---|
| the shipped prototype — *and it deletes the hyperlink* | 65.29 µs | 66.29 µs | 57.80 µs |
| correct, per cell | 79.47 µs | 86.28 µs | 646.87 µs |
| **correct, memoised** | **20.38 µs** | **28.46 µs** | **200.22 µs** |

**3.2× faster than the form ticket 06 shipped, on ordinary content**, because the shipped form does
the mix arithmetic per cell and the memo does it once per distinct style.

Two costs worth keeping visible. **`Mix` costs 14% over a plain `Darken`** — 78.2 against 68.5 µs
in ticket 06's unmemoised form — and it is taken, because one operator instead of three buys lifting,
tint and fade for free. And **an operator layer is 12.7× more expensive than a content layer**
(78.2 against 6.14 µs for a full-screen blit): it read-modify-writes every cell and resolves two
colours per cell, where a content layer is a `copy_from_slice`. A full-screen modal dim is therefore
the most expensive single thing the compositor can be asked to do, and it is a frame-pacing
consideration and not only a visual one.

**`Mix` darkens foreground and background by the same factor.** Darkening only the background *raises*
contrast, making shadowed text more prominent than unshadowed text — the opposite of what a shadow
means, and it would break modal dimming outright, whose whole job is to push the background away.

**Operators compound and are therefore not idempotent**: two overlapping shadows at 0.5 leave the
overlap at 0.25, which is visually right and means the order of operators among themselves matters,
not only their order relative to content.

### A shadow, and why no region arithmetic exists

A shadow is an ordinary operator layer whose rectangle is offset from its window's, sitting at a `z`
just below it. **The window occludes its own shadow by painting over it** — no region subtraction, no
L-shapes, no rectangle-minus-rectangle. The same trick covers modal dimming: the scrim goes beneath
the modal. The whole four-rectangle apparatus is replaced by putting the operator one `z` below the
thing that occludes it.

The darkening done under an opaque window and then overwritten is wasted work, and it is bounded:
6.14 µs of blit against 78.2 µs of operator. If it ever matters the fix is to skip damage rectangles
fully occluded by an opaque layer above, which is an optimisation and not a model change.

Convenience on top of the mechanism: a constructor taking an offset and an intensity and emitting the
operator at the right `z`. Mechanism first, convenience second.

### The wide-glyph hazard at a layer edge, in two halves

A layer edge can bisect a double-width glyph in the layer beneath it — §3's corruption bug reappearing
at composite time (17). It splits along the content/operator line and the halves get different rules.

**A content layer overwrites, so it is §3's five repair rules moved from write time to composite
time.** Four O(1) fixes per row: the copied content's own halves, a wide head left orphaned outside
the left edge, and a continuation left orphaned outside the right. The pairing invariant survives
twelve bisecting layers over a screen of mixed CJK.

**An operator recolours rather than overwrites, and half a darkened 漢 is an artifact the repair rules
do not cover.** One rule replaces a per-operator snap policy:

> **A glyph is atomic and belongs to its head cell: an operator acts on it if and only if the head is
> inside the rectangle.**

A left edge landing on a continuation leaves that glyph alone; a right edge cutting a head mixes the
whole glyph, reaching one column past the rectangle. This needs no knowledge of the operator's intent,
which is what makes it better than snapping — a shadow one column wider is invisible, but a modal dim
snapped outward would flood a column of the modal itself.

**Repair damages cells outside the layer's own rectangle**, and ticket 07 found a live bug proving it:
the prototype's `blit` repaired a bisected glyph and marked damage only over the layer's own columns,
so the terminal would have kept showing the half that was just blanked. The damage structure has to be
able to express this, and the bitset does.

### Non-rectangular content, and the `opaque` flag

The real degenerate case is not a "fully transparent layer" — that concept died with alpha-over — but
the trap beneath it: a caller who creates a 90×14 popup surface and draws only its border gets a
rectangle of **opaque spaces that erases the window underneath**. Rounded corners, a tooltip with a
transparent gutter and any overlay over a chart all hit it.

So a content layer carries `opaque`. Opaque layers go through `copy_from_slice`; non-opaque layers
test each cell against the `EMPTY` sentinel and skip it — **6.14 µs against 25.25 µs, 4.1×**. Making
every layer pay the skip path would have quadrupled the cost of the cheapest thing the compositor
does. `EMPTY` is `0x7FFF_FFFF`, the single value §3's grapheme encoding leaves free.

### The stack: a sorted `Vec`

`z: i32` explicit, `LayerId` stable for the layer's life, `seq` the tie-break among equal `z` and never
mutated — so raising a layer and dropping it back restores the exact original order.

| | n = 20 | n = 200 |
|---|---|---|
| ordered traversal of the whole stack | 5.05 ns | 59.3 ns |
| raise a layer to the top | 60.7 ns | 447 ns |
| `topmost_at` point query | 13.5 ns | 81.0 ns |

A tree was rejected because a `z` local to a parent *is* a scene, and the scene lives above the engine
(ADR 0002). A skip list was rejected for three reasons: **n is tens** — a layer is a window, popup,
shadow or dim, and rows of a virtualised tree are not layers; **the dominant operation is an ordered
traversal of the whole stack every frame**, which is what a contiguous `Vec` is best at and a skip list
worst at; and **a skip list's real modern advantage is lock-free concurrent ordered access, and there
is no concurrency here** — plus a node allocation per insert and an RNG for level assignment, which is
non-determinism where this map wants a property a test can assert. If a `Vec` ever stops being enough
the answer is `BTreeMap<(z, seq), Layer>`.

### The point query

`LayerStack::topmost_at(x, y) -> Option<LayerId>` — a plain reverse linear scan. **Operator layers are
not hittable**: a shadow is not a thing you click, so a click near a modal's edge cannot resolve to it.

The division of labour is the one `visible_rows()` already established: **the engine owns the query,
the runtime owns the index.** A spatial index is warranted where n is thousands, and there n is
*widgets*, which is a runtime concept — Textual is the direct evidence, using a spatial grid because a
per-widget region check stops scaling past ~1 000 widgets. Two orders of magnitude separate their n
from ours.

**The query must return a layer and never a widget.** `hit_test` returning anything widget-shaped is
the door ADR 0002 exists to keep shut.

### Degenerate cases

- **A layer larger than the target, or at negative coordinates** — intersected, not rejected.
- **A shadow falling off the edge** — the same intersection.
- **Two layers with identical `z`** — insertion order, stable, unaffected by any later reordering.
- **An operator with `amount == 0`** — skipped entirely, and it marks no damage. The identity never
  reaches a cell.
- **A zero-area layer** — the same.
- **An operator over default-background cells with the terminal silent on OSC 11** — left alone.
- **At `ColorDepth::None`, operator layers are skipped outright**, because `Mix` provably changes no
  byte on the wire. Worth **78.2 µs of the 107 µs worst screen** (§10).

---

## 6. Damage

**A per-row bitset with a dirty-row summary word, marked once per drawing verb, scanned in row order
into `(y, lo, hi)` runs.** Exact by construction: it can never report a cell the frame did not
change. (07)

### The metric is overdraw, not marking speed

The obvious ranking — how fast a structure marks and scans — is wrong. Marking happens once per
*verb*, not per cell, and per-cell marking was measured to buy 5–7% and nothing more. Scanning happens
once per frame over 80 rows. Both are nanoseconds.

What costs is **overdraw**: cells the serializer emits that did not change. Those are bytes on the
wire, and per-terminal force-flush limits go as low as 150 ms (01). **The wire is the scarce resource;
the CPU is the tie-break.**

| scene | cells changed | RowSpans | **RowBits** | SpanList |
|---|---|---|---|---|
| blinking cursor | 1 | 1.00× | 1.00× | 1.00× |
| scrolling list | 24 000 | 1.00× | 1.00× | 1.00× |
| twenty stacked popups | 16 898 | 1.00× | 1.00× | 1.00× |
| three dialogs standing apart | 1 941 | **2.53×** | **1.00×** | 1.00× |
| sub-cell chart, 400 points | 396 | **37.07×** | **1.00×** | 4.62× |

Estimated wire bytes: three dialogs 5 168 / **2 341** / 2 341; sub-cell chart 15 311 / **3 540** /
4 141. **The bitset wins by 4.3× on the chart and 2.2× on the dialogs, and it wins despite emitting
five times as many runs** — 393 cursor moves against 79. The cursor moves are cheap; the 14 283
unchanged cells the span model carries with it are not.

### The scenes the ticket named do not discriminate, and that is the finding

The ticket named three — a blinking cursor, a scrolling list, twenty stacked animated popups — and
**all three score 1.00× on every candidate.** Twenty popups 90 columns wide on a 300-column screen
overlap into a single interval per row, so there is nothing for a finer structure to save.

The two scenes that separate the structures are ones the ticket did not name and both are ordinary: a
status bar updating only its two ends across three dialogs, and a sub-cell chart, which is already one
of the map's hostile components. **A damage model validated only against its own scene list would have
picked per-row spans and been wrong by 37× on a component the map had already committed to
supporting.** That is why §14 makes the scene list normative rather than an appendix.

### The costs

| worst case per frame, on the scene where each is worst | RowSpans | **RowBits** | SpanList |
|---|---|---|---|
| mark (20 popups) | 519 ns | 1.56 µs | 1.98 µs |
| scan (sparse chart) | 10.8 ns | 1.94 µs | 163 ns |
| union of 20 layers | 431 ns | 3.01 µs | 1.10 µs |
| clear after a full-screen frame | 127 ns | 608 ns | — |
| **idle frame** | 3.97 ns | **6.60 ns** | — |
| bytes for a 300×80 surface | 320 B | 3 216 B | 1 440 B |

**About 7 µs in the worst frame measured against a 100 µs budget**, and 3.2 KiB against a surface's
375 KiB. Against §4's 150 µs full-screen draw and §5's 100 µs full-screen composite it is noise.

**An idle frame costs 6.6 ns and clears nothing** — the dirty-row summary means `clear` touches only
rows that were dirty. That is what the zero-wakeup idle needs from this layer, because "is anything
damaged" is what the condvar path interrogates.

### This goes against converged prior art, deliberately

Issue 03's survey was unambiguous: tcell, ghostty, Zellij, notcurses and termwiz all converge on the
row as the unit of damage, and call cell-level tracking more precision than the bookkeeping is worth.
We choose the bookkeeping they declined. The justification is the scene list, not taste: those projects
are terminal emulators and general toolkits, and this map has committed to a sub-cell-resolution chart
as a pressure case — exactly the shape where per-row granularity costs 37×. The bookkeeping they
measured as not worth it costs us 7 µs a frame.

**`SpanList<K=4>` is the arm to reject and it loses on every axis at once**: 4.62× on the chart, the
worst marking cost of the three, and by far the most intricate implementation. More complexity, more
CPU, worse output. And **per-row spans are not merely lossy, they are lossy without warning** — 1.00×
on three of five scenes, including two of the three originally named. A structure that is perfect until
it is 37× wrong is worse than one that is consistently mediocre, because nothing in ordinary testing
reveals the cliff.

**More runs is not worse.** The intuition that a serializer wants few long runs is wrong when the
alternative is long runs full of unchanged cells.

### Composition across layers

Two operations, both required by the compositor:

- **Translate** a layer's damage by that layer's offset into target coordinates, clipped to the target
  — a layer hanging off an edge is clipped, not dropped.
- **Union** across layers. A bitset unions with an `OR` and stays exact. Two 90-column popups on one
  row 100 columns apart are 180 emitted cells with the bitset and **280** with per-row spans.

**An operator layer has no damage of its own.** It holds no surface to ask. It contributes damage in
exactly two situations: when the union already covers part of its rectangle, and when the operator
itself changes — a fading shadow — which damages its whole rectangle. Neither requires the damage
structure to know what a layer is.

### The two terminal workarounds that are deleted rather than implemented

ratatui carries two bug-driven workarounds: VS16 emoji do not reliably clear their trailing cell, and
a wide→narrow shrink can leave the old style painted on the vacated columns. **Both are handled here
and cost nothing extra, because the write verb already damages both columns of a pair it breaks.**
Two special cases in the emit loop become zero. Do not port them.

### Resize, and invisibility

**Resize damages everything and is simply correct, not clever**: mark the whole screen, clear every
structure, reallocate the surfaces. There is no cache to invalidate, because §5 refused the only one
there would have been.

**The caller never sees a damage type.** Damage is not a parameter, not a return value and not a
concept a component author holds: the verbs mark it, the compositor unions it, `present` clears it.
That is deliberate — it is the one part of the engine that has to be invisible to be correct.

---

## 7. Threads, the handoff, and the frame clock

### Three threads, and the write direction is owned

| thread | blocks in | owns |
|---|---|---|
| **app** | one condvar | the layer stack, the canonical composite, the handle tables, the packet pool |
| **render** | the mailbox condvar | the terminal's **write** direction, exclusively, and the mirror |
| **input** | `read` on the tty | the terminal's **read** direction; writes the authoritative size |

**No timer thread.** An animation is a `wait_timeout` on a condvar the app thread already owns. A timer
thread would add a hop, a wakeup and a scheduling dependency, and buys nothing.

The two directions of a tty do not serialise against each other, so no lock mediates input and output.
Terminal setup — raw mode, alt screen, and §10's capability queries, which are the only place the
engine both writes *and* reads — happens entirely before either thread is spawned, where no
concurrency exists yet.

**And in that order, which the implementation got wrong for eight months.** `?1049h` is the first
byte this engine sends to a terminal and the capability batch is sent *inside* the page it opens —
production ticket 12, and it is the one place this sentence's list was load-bearing rather than
descriptive. The batch went out first because `actuate::negotiation` is built **from** the answers
and cannot be written before they are in; the repair is that the batch's own first bytes are
`?1049h`, and the negotiation is told the page is already ours. What that buys is that a terminal
which *prints* a sequence it does not implement — Terminal.app 2.15 prints the XTGETTCAP payload and
the final byte of each DECRQM — leaves its artefacts on a page discarded by `?1049l` on the way out
rather than on the user's own. The number of bytes this
engine sends to the user's own screen is **zero**, and that is register entry 30: a count, at offset
zero, over the two functions that are the whole of what goes out before a frame exists.

Three things this does not decide, stated rather than left as fog:

- **A terminal with no alternate screen is probed on its only page.** That is this defect with the
  mitigation removed, and it is not a regression — the same terminal ignored the same `?1049h` a
  moment later, in the negotiation, so its own page was always the one being written to. There is no
  fallback to design, because §15 puts inline, non-alt-screen rendering out of scope.
- **The page is owed back before there is a `Screen` to owe it.** An `attach` that ends in
  `AttachError::NoAnswer` has entered the alternate screen and armed no restoration site, so `Tty`'s
  own `Drop` gives it back — the same boundary it already draws for mode 2027, *whatever detection
  took, detection returns* — and hands the debt over the instant the site is armed. Both writing it
  and not writing it are defects, one leaving an empty alternate screen and the other restoring the
  user's shell cursor twice.
- **An erase of the *primary* screen was the other candidate and it is refused.** `CSI 2 J` there
  destroys the user's scrollback view, and the engine cannot know how many cells an artefact occupied
  because it cannot know which sequences a given terminal will print. **An erase of our own page is a
  different proposal and is taken**, because the questions go out after `?1049h` has cleared it: the
  artefact is at the alternate screen's home position, the mirror is unknown everywhere, and a cell
  no layer covers is never damaged and never written — so an application whose layers do not tile the
  screen would keep it for the session. Four bytes on the arm where the batch opened the page, and
  the extent nobody can know does not have to be known.

### The mailbox

`Mutex<Shared>` plus two condvars: one for "a packet landed", one for "the renderer is free". `Shared`
holds the slot, the free list and the ready flag, so **one synchronisation primitive exists in the
whole design**, not one per concern.

The mutex is not a compromise, it is free: hybrid pacing requires a condvar, `std`'s condvar requires a
mutex, so the mutex exists whatever the slot looks like. Putting the slot under it costs nothing.
Measured: the critical section on submit is **47 ns** over 5 000 submits, and a full uncontended
`lease → submit → take → finish` (four lock acquisitions) is **80 ns**.

The alternatives were rejected on `unsafe`, not on speed:

- **A seqlock cannot be written in safe Rust at all.** Its reader reads while the writer writes, which
  is a data race by construction.
- **A lock-free newest-only slot** needs `AtomicPtr` and a manual `Box::from_raw` on the take path.
- **A channel of owned frames** queues frames we would only drop, and an unbounded queue means
  unbounded buffers.
- **Double or triple buffering** is buffer age under another name; see below.

**The honest cost of not being lock-free**, stated rather than buried: if the app thread is preempted
while holding the lock, the render thread waits a quantum. Nothing cheaper than `unsafe` prevents
that, §2's invariant does not cover it, and it is accepted.

### What crosses: a packet, not a grid

```rust
struct Packet {
    runs:  Vec<Run>,       // (y, lo, hi), row order — §6's output, verbatim
    cells: Vec<Cell>,      // the damaged cells only, concatenated in run order, copied verbatim
    clusters: …,           // keyed by grapheme handle, bytes in the packet's own arena
    exts:     …,           // keyed by extended-style handle: fg, bg, underline colour, link id
    links:    …,           // OSC 8 URIs, in the same arena
    size: (u16, u16),      // so a stale-size packet can be refused on its own
    repaint: bool,         // set after a sweep renumbered a table — invalidate the mirror
    generation: u64,
}
```

**The render thread holds no application state and no handle at all.** Every handle is resolved at pack
time into a side table the packet owns, **keyed by the handle** — never written into the cell — and the
cell is copied verbatim. Dedup inside the packet is a generation-stamped marker per handle, so it is
O(1); the scan version cost 166 µs where the marker costs 31.

That keying is a correctness requirement rather than a refinement, and the defect it fixes was invisible
to reading:

> Writing a *position* into the packed cell makes an unchanged cell pack differently whenever the
> frame's damage changes shape. Two frames whose damage differs but whose content does not therefore
> disagree about cells that did not change, and §8's equality filter re-emits them. Over five steady
> frames of a page with one cluster cell in a hundred, nothing changing: **28 360 bytes and 1 170 cells
> emitted against 100 bytes and none — 284×.** Keying by the handle is also **1.6×–2.6× faster**,
> because the run copy becomes a `copy_from_slice` again.

| pack, 300×80, on the app thread | plain | realistic 1% | linked 100% | hostile 100% |
|---|---|---|---|---|
| position-rewriting | 50.00 µs | 51.05 µs | 49.95 µs | 98.99 µs |
| **keyed** | **21.99 µs** | **28.13 µs** | **31.02 µs** | **38.42 µs** |

A **content-keyed** packet — deriving the key from the entry's content so a sweep is invisible to the
mirror — was built, fused into the pack loop, and **refused** on three grounds in this order: it buys a
*probability* (≈10⁻⁸ at 52 bits for 10⁴ live extended styles, but the failure it buys is a cell
silently never repainted, replacing a full repaint that happens roughly never); its 31 µs on the
adversarial page lands on the **app** thread, which is the scarce one, to save the render thread, which
has slack; and identity is simpler to reason about. It is on the prototype branch with its number
attached if a future ticket wants it.

### Why a packet rather than handing the grid over

The alternative was ownership transfer with a recycled pool: no copy at all, at the price of *buffer
age* — a recycled buffer holds a stale generation, so the app must repaint the union of the last N
frames' damage.

**Cost does not discriminate**, and the correction is recorded rather than dropped: the area multiplier
is real (1.01×–2.00×) but the absolute worst case is **0.4 µs against a 100 µs budget**. That is the
third ticket running where cost did not decide — two ownership shapes at 54.1 against 54.3 µs (05),
three damage structures at 1.00× on the ticket's own scenes (07).

What decided it is the serializer's walk:

| scene | packet (contiguous) | grid indexed by runs | × |
|---|---|---|---|
| cursor | 1.11 ns | 1.88 ns | 1.69 |
| scrolling list | 7.86 µs | 7.80 µs | 0.99 |
| twenty popups | 5.62 µs | 6.32 µs | 1.12 |
| **sparse chart (584 runs)** | **179.6 ns** | **875.7 ns** | **4.88** |
| full screen | 7.98 µs | 7.87 µs | 0.99 |

On contiguous full-screen damage the two are identical, as they must be. On scattered runs the packet
walks 9.4 KB contiguously where the grid jumps a row stride per run across 384 KB. **That scene is the
sub-cell chart, which had already decided §6 at 37.07×** — two independent tickets settled by one
component, which is why it is a normative CI scene rather than a curiosity.

Packing is proportional to damage: **4.76 ns** for a cursor, 2.29 µs for the sparse chart, 5.66 µs for
a whole screen, against **6.24–7.24 µs** to memcpy the grid regardless of what changed. For a typical
frame the packet is three orders of magnitude cheaper than the copy it replaces.

### Backpressure: there is none, and the drop path is unreachable

**The app thread never waits for a slot, ever.** But dropping intermediate frames is implemented as
**never composing them**: the app composites only after the render thread has signalled that it is
free, and until then damage accumulates in §6's structure, which is already the coalescing mechanism
and costs 6.6 ns to interrogate. Coalescing therefore happens *before* the 107 µs composite rather than
after it — on a 200 ms ssh frame the app pays for one composite, not for the twenty it would have
thrown away.

The consequence is stronger than intended: with one producer and the ready gate, **the slot is always
empty at submit, so a packet can never be superseded** — asserted over 10 000 cycles with the counter
at zero. The frame-union hazard raised during the grilling, where a dropped frame takes its damage runs
with it and corrupts the screen, cannot occur, because no frame is ever dropped at the mailbox. The
supersede path exists in the prototype only so the counter can prove it stays at zero.

This also fixes the pool at **two packets**, and two is provably enough rather than empirically enough:
one being filled, one in the renderer's hands.

### The frame clock

`Config::max_frame_rate` in hertz — set by the application, never discovered by the engine, because a
tty cannot report a refresh rate. `Screen::set_max_frame_rate` covers a monitor changing under a
running program.

**It is a minimum gap, not a tick.** The first damage after a quiet period paints immediately;
everything arriving inside the gap coalesces and paints once, at the end. The gate costs 2.16 ns and it
sits on `wait` (§2, ADR 0004).

Why not a fixed-rate ticker, in one line: a ticker costs CPU while idle and adds up to half a frame of
latency to the first keystroke; leading edge has neither, and idle cost is a standing requirement rather
than a preference.

Four properties, each asserted rather than assumed:

- **Leading edge survives the move to `wait`**: a wake after a quiet period returns in **250 ns p50 /
  1.04 µs p99 / 16.75 µs max** over 2 000 samples.
- **Idle is zero, not small.** Three threads parked for three seconds: `3.005 s elapsed · app waits 1,
  wakes 0 · render wakeups 0 · frames painted 0`, and `3.00 real, 0.00 user, 0.00 sys, 1 voluntary
  context switch`. A 120 Hz ticker would have woken 360 times.
- **Quit is never paced.** At a 1 Hz ceiling, checking the clock before the quit flag hangs shutdown
  for a second. `Wake::Quit` is checked first, with a test. Writing the test is what found it.
- **The ceiling is achieved from below.** `wait_timeout` overshoots on macOS — a hold measures
  10.18 ms p50 against an 8.333 ms gap — so 120 Hz configured yields about 110 fps under load.
  Undershooting a ceiling is safe by construction; compensating is an implementation choice, and the
  number is here so nobody spends a day rediscovering it.

**Animations register a deadline and the engine has no animation concept.** `request_wake_at(when)`
keeps the earliest outstanding deadline; deregistration is not renewing it. With no deadline registered
the wait is indefinite, which is the whole of the idle guarantee. Timelines, easing and interpolation
are the runtime's.

One wake source multiplexes everything that can wake the app thread — an input event, the renderer
going free, a deadline, and a post from another thread:

```rust
pub enum Wake { Input, Posted, Deadline, Quit }
```

`Wake` says *why we woke*; `Event` says *what happened*. That is why a resize is an `Event` and not a
`Wake` variant: reacting to it means re-laying rectangles.

Wake-up latency, app submit → render holding the packet, with the render thread genuinely parked
(n = 5 000): **p50 4.58 µs · p90 6.96 · p99 16.4 · p99.9 30.3 · max 71.5 µs**. **This does not spend
the 100 µs frame budget** — that budget is app-thread CPU work and the app thread has already returned;
this is wall-clock delay on the way to the wire, and it is OS scheduler latency rather than our code.
Against a 16.6 ms gap the median is 0.03% of a frame.

**False sharing across the handoff is real and irrelevant**: 5.09× at 20 M contended writes, and the
handoff touches those words about 60 times a second. No padding in the mailbox.

### Resize, shutdown and panic

**Resize.** The authoritative size is one packed `AtomicU32`, written by the input thread. A leased
surface never changes size under a drawing caller — it does not know the terminal exists. At submit the
app compares once more, one atomic load; if the size moved, the packet goes straight back to the pool
and a full repaint is scheduled. Cost: one load per frame, one discarded composite per resize.

**Panic.** "Only the render thread writes" narrows to *only the render thread writes frames*. A panic
hook runs on whichever thread panicked, and joining the render thread from inside it deadlocks when the
render thread is the one that panicked — so restoration is an **idempotent function guarded by one
atomic, callable from any thread**, asserted with eight threads racing it and exactly one performing
the restoration. Two details are cheap to omit and expensive to debug:

- Restoration must run **before** the default panic hook prints, or the backtrace is painted into the
  alt screen and vanishes with it.
- It hangs on a guard's `Drop` as well as on the hook, so a normal return and a `?` out of `main` take
  the same path.

**Restoration includes input state, not only the alt screen** (§9): pop the keyboard enhancement flags,
disable mouse tracking, focus reporting and bracketed paste, and restore auto-wrap (§8). A crashed
process that leaves kitty flags pushed breaks the user's shell until they know to run `reset`.

**Shutdown.** The render thread **is** joined — it is either parked on a condvar or inside a bounded
`write`, both finite. The input thread is **never** joined: it sits in a blocking `read` with nothing
to wake it short of a signal, so it dies with the process. The last frame is not flushed on quit;
showing state that is already stale buys nothing and costs exit latency.

### The terminal leaves and comes back

Three ways it happens, and they are not the same shape. §15 filed all three as fog; production ticket
07 is where each got an answer, and **only one of them is a pair of verbs**. Register entry 29.

**1. The application gives the terminal up on purpose** — Ctrl-Z, or a child that does not want the
keyboard: a formatter, a build, anything reading from a pipe. `Screen::suspend` writes the epilogue
and leaves raw mode; `Screen::resume` writes the negotiation, takes raw mode back, and schedules a
full repaint. The `Screen` keeps everything in between: the layers, their cells, the interned
clusters, the registered deadlines, and the caller's own mouse level and caret.

**The supported shape is *suspend, stop the process, resume*, and the reason is the reader.** The
thread that reads the terminal is parked in a blocking `read` on standard input and **nothing in safe
Rust cancels one**, so a suspension does not vacate stdin — register entry 31, and a pty gate rather
than a sentence since production ticket 13. A `SIGTSTP` stops every thread of the process and the
reader with it; an application that suspends and keeps *running* to spawn a child in the same window
is competing with that child for every byte, and the kernel gives each byte to whichever reader it
schedules. **So an interactive child in the same terminal is refused rather than supported**: it is
not a caveat on case 1, it is a case this pair does not serve, and the way to serve it is to stop the
process while the child runs or to give the child its own standard input. Production ticket 13 priced
the four ways the read could be made cancellable — a self-pipe and `poll`, crossterm's own event
source, a non-blocking descriptor, `rustix` — and refused all four; ADR 0052 carries the reasoning.

What is decidable is what happens to the bytes the reader did take: a resume **throws away everything
the user typed** during the suspension and keeps everything the terminal *became* — a resize, because
`next_event` is the only thing that applies one to the surfaces and `present` refuses to composite
while the two disagree, and focus, because it is a level rather than something somebody typed.

Seven consequences, each of which is a defect if it is left out:

- **A resume is a repaint of every cell.** Re-entering the alternate screen is a *cleared page*, so
  the mirror on the other side of it may not be trusted for a single cell — the same `repaint` a
  resize schedules, one step stronger, because a resize keeps the terminal's cells and this does not.
- **The frame after a resume is owed and nobody else will ask for it.** An application whose next line
  is `wait()` has no input, no deadline and no post to be woken by, so the resume records the debt
  itself. It needs both halves of what `wait` releases on: the render thread that the suspend joined
  set *free* to false on its way out.
- **The actuators are told what the negotiation said, not what the caller wants.** The negotiation
  sets the *declared floor* and hides the caret, so an application that had raised the mouse to
  `Motion` comes back to a terminal at `Buttons` — and the frame after the resume is what carries the
  difference. Without it the mouse is dead on a screen that repainted perfectly.
- **Nothing goes out in between.** On the deterministic clock the renderer is inline, so a `present`
  between the two verbs writes straight onto whatever has the terminal. It answers `submitted: false`
  and keeps the damage, so a frame is deferred rather than lost.
- **A session whose render thread panicked stays suspended.** Both verbs reach for the renderer's
  sink, and a panicked render thread took it: `suspend` has nothing to write the epilogue through and
  `resume` has nothing to hand a packet to. Answering with silence is right rather than merely safe —
  the panic hook has already given the terminal back and a `Wake::Quit` is on its way, and §12's
  refusal 7 already says a dead renderer is indistinguishable from a permanently busy one.
- **What the user typed at the other program is dropped, and what the terminal became is kept.** The
  paragraph above is why there is anything to drop; this is the rule for it. Delivering it would put
  an editor's whole session into the application as keystrokes, on a screen that has just repainted
  and believes it has the keyboard.
- **The in-loop overrun detector stops, and comes back.** §11's detector aborts the process when an
  iteration stays open past the threshold, and the iteration `wait` opened is open for as long as the
  editor runs — so a suspend must stop it. It must also *restart* it, and that is the half that is
  invisible: the stop flag lives behind the `Arc` every observer thread holds, so a second `observe`
  spawns a thread that reads it and returns, leaving a resumed session with no detector at all in
  exactly the debug builds that exist to have one. Nothing would say so, because a detector that does
  nothing is what a healthy one looks like. **A flag cannot express it either**: an observer sleeps
  for up to one poll before reading anything, so a suspend and a resume completed inside that
  interval leave it waking to a flag already cleared, alive beside the new one and equally able to
  abort the process. The watch carries a monotone **generation** that the stop bumps, and an observer
  leaves when it stops matching the one it was spawned with.

**Ctrl-Z is a key and never a signal, and that is what makes this the application's business rather
than a signal handler's.** Raw mode is `cfmakeraw`, which clears `ISIG` — measured on this machine on
2026-08-29 through a pty with crossterm 0.29: `ISIG` is true before `enable_raw_mode` and false after.
So for as long as a `Screen` is attached the byte `0x1a` reaches the parser as `Ctrl+z` and no
`SIGTSTP` is generated at all, and what an application does about it is three lines on the app thread:
suspend, stop the process, resume. **The engine installs no signal handler and cannot** — `sigaction`
is not reachable from safe Rust, std has no signal API, and this crate's dependency policy is
crossterm plus generated UCD tables (ADR 0001). That is a constraint rather than a position, and it
happens to be the right answer anyway: the process that stops is the one that knows what it was doing.

An **external** `SIGTSTP` — `kill -TSTP`, or a job-control stop of a process that is not in raw mode —
is not covered and cannot be. The default action stops the process where it stands, with no unwinding
and nothing to run, so the shell that takes the foreground gets it inside the alternate screen. Stated
rather than mitigated: the mitigation is a signal handler, and there is nowhere to put one.

**2. The terminal goes away underneath the process** — a dropped `ssh` connection, a closed window, a
multiplexer detaching. Nothing can be written, because the descriptor is gone: `write_frame` discards
the error and the frame path has no `Result` in it by design (§4's clamp-and-discard, ADR 0022), so
`present` goes on answering `submitted: true` for ever. **What the engine owes is to say so.** The
input thread's channel closes when its blocking `read` returns end-of-file **or an error** — both,
because a closed pty master presents as end-of-file on macOS and as `EIO` on Linux, so a signal that
fired on only one of them would be right on one of the two platforms this workspace builds for — and
`Tty::open` hands a
reader over only when standard input *and* standard output are both a terminal — so that end-of-file
is the pty's far side closing and nothing else. It raises `Wake::Quit`, which is the same spelling
`WakeSource::renderer_gone` reaches for one file over and for the same reason: an application that
handles quit already does the right thing, and one that does not was going to hang either way. **A
hang is worse than a failure.**

**3. A different terminal on the same process** — a reconnected `ssh` session, a multiplexer client
attaching from somewhere else. `Capabilities` is sampled once and immutable for the life of a `Screen`
(§10), and that does not change here: the answer is to drop the `Screen` and `attach` again, which is
supported and gated. What it costs is the whole session — the layers, their cells, the interned
clusters and the mirror all go with the `Screen` — plus a detection round trip, whose ceiling is
`detect::CEILING` at 250 ms with per-terminal force-flush limits as low as 150 ms recorded in
`quirks.rs`. **A resume is deliberately not that**: it re-declares and never re-detects, which is
right for a terminal that is the same terminal and wrong for one that was replaced, and the two cases
are told apart by the caller because nothing on this side can tell them apart at all.

---

## 8. Bytes on the wire

**Twenty-nine bytes of framing, one `write`, one `u64` compare per run — and a mirror in front of the
emit loop, which is what buys everything else.** (08)

The emit loop itself is issue 03's converged prior art with §3's packed style substituted for four
scalars. What is new is what sits in front of it.

### The mirror

The render thread keeps one grid recording **what the terminal is currently showing**, updated as bytes
are emitted. See [ADR 0006](../../docs/adr/0006-the-render-thread-mirrors-what-the-terminal-shows.md).

This reverses ticket 09's "the render thread holds no screen-sized state at all". The reason behind
that sentence — that resize, shutdown and panic stay small — survives, because the mirror holds no
*application* state and no handle; the sentence does not. The mirror is what the equality filter
compares against and what a scroll is proved against. A row that cannot be trusted — at startup, and
after a resize — is an **unknown row**, written whole rather than compared, which is how a full repaint
expresses itself without a separate mode. `Packet::repaint` marks every row unknown (§3).

### The serializer has no knobs

The prototype is `Options`-shaped only so the variants could be measured. **The engine ships exactly
one configuration, and none of it is a choice an application makes:**

| | |
|---|---|
| cursor encoding | `shortest` — `CUP`, `CHA`, `CUF`, `CR`, `CR`+`LF`s, priced by digit count, no table |
| equality filter | on, always, with the gap merge priced in bytes |
| scroll region | on, one candidate, verified against the mirror before a byte is emitted |
| synchronised output | DEC mode 2026 where the terminal has it |
| auto-wrap | **off for the lifetime of the alt screen** |
| SGR at frame start | reset — tcell's "make no style assumptions", four bytes |

No `Options` type reaches the public surface. Everything wire-specific lives **below** the packet.

### Auto-wrap off, and the two workarounds it deletes

Auto-wrap is turned off **once**, on entering the alt screen, not once per frame, and **it must be
restored on leaving** — which is why this is a decision the spec states rather than an implementation
detail.

- It is worth **10 bytes of every frame**, which matters only because a caret blink is a 29-byte frame:
  `?2026h` + `0m` + `?2026l` is 20 bytes of fixed framing and the caret's own change is 9.
- **Both of cellbuf's bug-driven workarounds stop having a subject.** With no wrap there is no
  pending-wrap state to track and no bottom-right corner that scrolls. The prototype models both
  anyway and tests both with auto-wrap on and off, so the claim is executed rather than asserted.

The cost of refusing auto-wrap is that a run ending at the right margin can no longer flow into the next
row's run without a move. Measured on the only scene where that happens at scale — a full-screen change
across 80 rows — it is **zero**: 73 290 bytes either way, because damage already splits at row
boundaries.

### The emit loop

```rust
let mut style = u64::MAX;              // never equal to a real style, forces a first emit
for (i, &c) in run_cells.iter().enumerate() {
    if c.g == CONTINUATION { continue; }        // the head already painted both columns
    let x = run.lo + i as u16;
    move_to(x, run.y);                          // a no-op when the cursor is already there
    if c.style != style { emit_sgr_delta(style, c.style); style = c.style; }
    emit_grapheme(c);
}
```

- **The column comes from the index, not from an advance counter.** That is what makes skipping a
  `CONTINUATION` free of bookkeeping, and the first version got it wrong in the one case where a run
  begins on a continuation cell.
- **SGR 22 resets bold *and* dim together.** There is no un-bold that leaves dim standing, so removing
  either means emitting 22 and reapplying the other. A naive per-attribute on/off loop silently drops
  dim whenever bold turns off. Pinned by a test.
- **SGR 21 is never emitted.** ECMA-48 assigns it "doubly underlined" and a meaningful population of
  terminals implements it as "bold off". Double underline is `4:2`.
- **A differential SGR is worth it and costs nothing to decide**, because the decision is one `u64`
  compare and the decomposition is off the hot path by construction. A realistic full-screen frame
  emits **one** SGR sequence for 24 000 cells.
- Extended styles emit SGR `58:2::r:g:b` / `59` and OSC 8 from `packet.exts` and `packet.links`. **The
  underline colour's spelling is its own axis and not `legacy_sgr`'s**: `Underlines::ConPty` selects
  `58;2;r;g;b`, is reached only from the quirk table, and is set independently — so all four
  combinations of the two axes exist, and §14's round trip drives every one (§10).

### Cursor movement: the first bullet, and it barely matters

| scene | absolute `CUP` | natural | **shortest** | moves |
|---|---|---|---|---|
| caret blink | 88 | 88 | 88 | 1 |
| progress +1% | 147 | 147 | 147 | 1 |
| list scrolled, label only | 4 313 | 4 313 | **4 305** | 80 |
| list scrolled, rows cleared | 5 168 | 5 168 | **5 160** | 80 |
| sub-cell chart moving | 5 471 | 5 471 | **4 623** | 113 |
| three dialogs apart | 1 230 | 1 230 | **1 203** | 8 |
| full-screen change | 77 438 | 73 290 | 73 290 | 80 |
| worst case | 2 417 060 | 2 417 060 | 2 417 060 | 80 |

**§6 was right that the cursor strategy matters more than it would have, and still it is worth 0.2% to
15%.** The largest win is the chart, the scene with the most runs, and even there the win is not the
*encoding*: absolute and natural tie because a chart's runs are never contiguous, and `shortest` gets
its 848 bytes almost entirely from choosing `CUF` and `CHA` over `CUP`.

**One rule is added by §10 and it is a correctness rule rather than a byte-count one: after a run
containing any non-ASCII cluster, the next move on that row is `CHA` rather than `CUF`.** A width
disagreement is *permanent* once a mirror exists — the mirror records what was intended, so an
overpainted neighbour is never re-emitted — and `CUF` is relative and compounds the error along the
row. One boolean carried through a scan that is already running.

**Issue 03's tension between "fewest bytes" and "fewest sequences" dissolves rather than resolves.**
Every choice here reduces both: the filter removes cells *and* escapes, the gap merge removes escapes at
the cost of cells the terminal was going to parse as a run anyway, and `shortest`'s `CR`+`LF` picks
one-byte controls over a parameterised CSI. The kitty #6030 finding never comes due. It would have, for
cellbuf's per-move search over five encodings including content overwrite, which is the version that is
refused — along with its `ICH`/`DCH` line editing, which the filter subsumes.

### The equality filter

Placed **inside the run scan**, comparing each damaged cell against the mirror. Real bytes, four frames,
first excluded:

| scene | span (no filter) | strict | gap 6 cells | **chosen** | win |
|---|---|---|---|---|---|
| caret blink | 267 | 88 | 88 | **88** | 3.03× |
| progress +1% | 4 527 | 147 | 147 | **147** | 30.8× |
| list scrolled, label only | 7 932 | 4 483 | 4 305 | **4 305** | 1.84× |
| list scrolled, rows cleared | 73 356 | 5 290 | 5 160 | **180** | **408×** |
| sub-cell chart moving | 21 843 | 4 581 | 5 276 | **4 623** | 4.72× |
| three dialogs apart | 10 926 | 1 491 | 1 203 | **1 203** | 9.08× |
| full-screen change | 74 250 | 80 650 | 73 290 | **50 746** | 1.46× |
| worst case | 2 417 060 | 2 417 060 | 2 417 060 | 2 417 060 | 1.00× |

**A fixed gap threshold is in the wrong unit and there is no right value for one.** A cell is one byte
of ASCII, **three of braille** and four of an emoji, so a six-cell threshold on the chart spends
eighteen bytes to save six. A sweep shows it: the chart goes 4 581 → 4 735 → 5 135 → 5 276 → 5 507 →
5 909 → 7 611 as the threshold climbs 0 → 24, monotonically worse, while the dialogs go 1 491 → 1 203
and then flat. **The two scenes want opposite thresholds, so there is no threshold.**

What replaces it needs no tuning: **price the gap in bytes** — the clusters' UTF-8 lengths plus a floor
for any style change inside — against the digit-counted cost of the cheapest encoding of the move it
would avoid. It lands within 1% of the better of the two fixed thresholds on every scene.
Underestimating the SGR is deliberately the safe direction: it only ever makes the serializer prefer
the move.

**The same mirror pays for a second win.** §6 produces genuinely separate runs on one row, and the gap
between them is not in the packet — but it *is* in the mirror. So two runs on a row can be bridged by
repainting the untouched gap out of the mirror, priced by the same rule. That is where the dialogs'
1 491 → 1 203 comes from.

**Cost: 0.9 ns per damaged cell** — 88.94 against 112.83 µs on a full-screen 24 000-cell frame, under
half the estimate, because the comparison rides inside a scan that was already reading every cell.
Memory is one cell buffer, 384 KiB at 300×80.

**Run it always. Damage area does not predict whether it pays, and the intuition is exactly inverted:**
a full-screen change damages 24 000 cells and the filter buys 1%, while a cleared-row list scroll
damages the same 24 000 cells and the filter buys 14× — and then the scroll region takes it to 408×.
An area-based heuristic would switch the filter off precisely where it is worth most. The price of
"always" is +24 µs on the render thread on a full-screen churn frame, 0.14% of a 16.6 ms interval.

### The scroll region, and the reasoning that nearly shipped a corruption bug

**In scope and taken.** A cleared-row list scroll goes from **1 726 bytes to 60** — one `DECSTBM`, one
`SU`, one `DECSTBM` reset, and twenty cells. That is `less`, `tail -f`, a file manager and every log
pane.

It is a pre-pass: emit `DECSTBM` + `SU`/`SD`, shift the mirror exactly as the terminal will, then run
the ordinary filtered emit loop over the residue. **The exposed rows are recorded in the mirror as blank
rather than as unknown**, and that is exact rather than optimistic: a terminal erases what it exposes
with the *current* background, and the frame has already reset SGR to default at that point, so
background-colour-erase and erase-to-default agree. That is the one place a scroll optimisation usually
goes wrong and it is closed by ordering, not by a capability query.

> **The reasoning that was wrong, because it is the attractive one.** The first version was
> *speculative*: guess cheaply, because the filter behind it compares against the mirror and repairs
> whatever the guess got wrong. That is false. The filter can only emit cells the packet carries, and
> the packet carries damaged cells only. `SU` moves every column of every row in the region — including
> the columns this frame never touched, which are exactly the ones no later pass can put back. A screen
> with two text verbs and a one-column gap between them loses that column. The round trip caught it
> within a minute.

So the guess is **verified before a byte is emitted**, against two obligations:

1. Every row the scroll *moves* must already hold, in the mirror, what this frame wants on the row it
   lands on — undamaged columns included.
2. Every row the scroll *exposes* must be blank in every column this frame does not repaint.

Obligation 2 is checked first, because it is a handful of rows against the whole band and it is the one
that actually fails. Cost when the probe matches and the verify fails: **+0.3 µs**. Cost when it
succeeds: **+21 µs** to remove 5 100 bytes from the wire, which on a 4 MB/s link is 1.3 ms of
transmission — a 60× trade.

Two limits, both measured rather than reasoned:

- **One candidate is verified, not every candidate that matches the probe.** A screen whose rows repeat
  — an alternating pattern, a ruled table — matches many times over, and verifying each turned a 38 µs
  frame into **1.03 ms**, a 27× regression that only appeared because numbers were kept per scene.
  Taking the first match forfeits a scroll that could in principle have been found; that is not worth
  27×.
- **`SU` has no horizontal margins**, so a pane that is not the full width can only use this when the
  columns it does not own are blank in both frames. `DECSLRM` (mode 69) would lift that and is
  deliberately not used: it is not in tier-1's confirmed set, and the verification above turns an
  unsupported margin into silent corruption rather than a wasted escape. §15 keeps the question open.

**And the finding that decides who can use it: a component must repaint whole rows.**

| the same list, scrolled one row | damaged cells | span bytes | filtered | + scroll region |
|---|---|---|---|---|
| label only | 2 038 | 7 932 | 4 305 | 4 305 (refused) |
| rows cleared, as a widget must | 24 000 | 73 356 | 5 160 | **180** |

Clearing the rows makes the unfiltered case **9× worse** and the final case **24× better**, and costs
the app thread 34 µs of packing instead of 1.9 µs. This is the same fact from the other side as
clear-then-draw defeating the write-time filter (§4). **The idiom is not a wart to be optimised away;
it is load-bearing**, and it is a performance contract on the component library rather than a style
preference.

### Colour emission

Default is `39`/`49`; indices under 16 use `30`–`37` / `90`–`97` and their background forms rather than
the parameterised one. **The parameterised forms exist twice over and this section does not choose
between them**: ITU-T T.416's colon spelling is `38:5:n` / `38:2::r:g:b`, with the empty colour-space
id the standard puts there, and xterm's pre-ITU-T semicolon spelling is `38;5;n` / `38;2;r;g;b`.
**Which one ships is §10's and not this section's** — it is the default of a capability, moved by
`Overrides::legacy_sgr`, `VITUI_FORCE_LEGACY_SGR` and four quirk entries. Quantisation resolves from
§3's packed tag, on the render thread, **before** the mirror comparison (§10).

**Every byte table in this section was measured on the semicolon spelling, because the prototype
emitted it** (arch 23). The correction is exactly countable and narrower than it looks: the two
spellings of an *indexed* colour are the same length — `38:5:200` and `38;5;200` are both eight bytes
— so **only a truecolor colour costs the extra byte**, and only a row whose scene paints truecolor
moves at all. Every number a gate reads has been re-measured against the shipped serializer and lives
in `crate::gates::WIRE_BUDGET`; the tables below are kept as the evidence for the *rulings* they were
taken for, and every one of those rulings is a within-row comparison at a constant spelling. The one
row a decision rests on is restated rather than annotated — see *One write per frame* below.

| scene | truecolor | 256 | 16 |
|---|---|---|---|
| three dialogs apart | 1 203 | 1 161 | 1 128 (−6%) |
| sub-cell chart | 4 623 | 4 599 | 4 581 (−1%) |
| full-screen change | 50 746 | 50 722 | 50 704 (−0.1%) |
| worst case | 2 417 060 | 1 658 570 | 934 058 (−61%) |

**Quantising colour saves almost nothing on frame size**, because style-run coalescing has already
collected the win: one SGR covers a run, so shortening the sequence saves bytes proportional to *runs*,
not to cells. `--no-color` is a **compatibility** feature, not a bandwidth one, and must not be defended
as an optimisation. What quantisation *does* change is frame **membership** — see §10.

### One write per frame, and the worst case

**One `write` per frame on every scene, including an 800 KB one** — 30 frames, 30 writes, measured
through a real socket pair. The partial-write loop is exercised separately and deterministically by a
sink that takes seven bytes at a time and returns `WouldBlock` every fifth call: 795 writes and 159
retries reassemble into a byte stream that replays to the correct screen. That is the honest way to test
it, because whether a real pipe fragments a write is the kernel's business.

**The frame is never split on purpose.** A synchronised-output block spanning two `write` calls is still
one block to the terminal; a frame split into two blocks tears. The loop is only about the kernel's
buffer being smaller than the frame. Partial writes block nothing but the render thread — §2's invariant
is what makes a 200 ms `write` on ssh harmless.

| | bytes | SGR | serialize | on a 4 MB/s link |
|---|---|---|---|---|
| every cell a distinct style | **841 002** | 24 000 | — | **201 ms/frame** |
| a realistic full-screen frame | 24 430 | **1** | 116 µs | 6.1 ms/frame |

**Restated by arch 23, and it is the one row in this section where that was necessary rather than
cosmetic** — a decision rests on it, so an annotation would not do. The prototype's figures were 805 657
bytes and 33.6 a cell; `serial::tests::the_worst_case_frame` measures **841 002 bytes, 24 000 SGR and
35.0 a cell** on the shipped engine, which is **4.4% low** and is arch 23's estimate turned into a
measurement. This fixture is the only one charged the colon form's extra byte *twice* per cell — a
distinct foreground **and** background — so the spelling is nearly the whole of the gap. Two arithmetic
slips in the prototype's own prose go with it: 805 657 bytes at 4 MB/s is 201 ms and 1.3×, not "nearly
twice", and the realistic row is 6.1 ms rather than 5.0. The serialize column is dropped rather than
restated, because the 925 µs was a release-build prototype figure and the shipped test runs in debug.

**The synchronised-output ceiling that binds is the time limit, not the byte limit**, and the ruling is
unchanged by the new number: 841 KB is 0.40× of Alacritty's 2 MiB, and 201 ms is 1.3× its 150 ms — so a
worst-case frame over a slow link *will* be force-flushed and *will* tear. The test asserts that
inequality rather than printing it, so the ruling is gated and not merely recorded. Nothing mitigates it
and nothing needs to: the frame is 35.0 bytes per cell against 1.0 for a realistic one, and a screen
where every cell has a unique colour is a test fixture. Recorded so the tearing is a known consequence
rather than a bug report.

### Whose budget this is

Two tickets wrote that essentially the whole 100 µs incremental budget was the serializer's to spend.
That is off by a thread. **The serializer runs on the render thread, so it spends none of the app
thread's budget**; what bounds it is the frame interval, and the worst realistic frame is 116 µs against
16.6 ms — 0.7%. The app thread's share is the pack step alone, at **1.4 ns per cell**: 53 ns for a caret
blink, 2.86 µs for three dialogs, **34.5 µs** for the row-clearing list scroll that unlocks the scroll
region.

### The backend seam is the packet, not the byte stream

Everything in this section — SGR deltas, cursor encodings, `DECSTBM`, synchronised output, the mirror —
lives **below** the packet. A different backend, a GPU one included, is another consumer of
`runs: &[Run]` plus `cells: &[Cell]`, and damage runs are exactly what such a backend wants in order to
avoid re-uploading a whole screen. No design budget is spent on this and no capability is added for it;
the only obligation it creates is the one already in force — **nothing below the packet leaks above
it.**

### The instrument is part of the design

Before any optimisation there is a **`TermModel`**: a miniature terminal that parses the serializer's own
bytes back into a grid — `CUP`, `CHA`, `CUF`, `CR`, `LF`, `SGR`, `SGR 58/59`, OSC 8, `DECSTBM`, `SU`,
`SD`, `DECAWM` and the pending-wrap state. Every scene is replayed and the resulting screen asserted
equal to the frame that was composited.

*This read "under nine option sets" until ticket 22.* The nine were the prototype's **serializer
knobs** — the shape it was given only so the variants could be measured — and the engine ships one
configuration with no `Options` type on the public surface, so the phrase no longer names anything.
What a scene varies now is the **declared tier** it is replayed at, through its own `Scene::overrides`,
over the seven axes §10 makes declarable. The phrase had also been cited in §10 as the reason headless
must pin every axis, which was the wrong reason for a true sentence; §10 now carries the right one.

This is not test scaffolding, it is why the ticket reached an answer. **Both times the serializer was
wrong, the round trip is what said so, and in both cases the reasoning that produced the bug was
comfortable.** A serializer is a program whose output is only checkable by a program that reads it, and
building that program is a third of the work. §14 makes it the primary instrument.

---

## 9. Input

**The engine reports what the terminal reports, and refuses to invent the rest.** (10)

```rust
pub enum Event {
    Key(Key),
    Mouse(Mouse),
    Paste(Paste),
    Resize(u16, u16),
    FocusGained,
    FocusLost,
}

pub struct Key {
    pub code: KeyCode,          // the base layout — where the key is, not what it printed
    pub mods: Mods,             // eight bits: shift alt ctrl super hyper meta caps num
    pub kind: KeyKind,          // Press | Repeat | Release
    pub text: KeyText,          // what it produced: a grapheme cluster, inline, possibly empty
    pub at: Instant,            // stamped by the input thread, at read time
}

pub struct Mouse {
    pub x: u16, pub y: u16,     // screen cells, and nothing resolved
    pub kind: MouseKind,        // Down(Button) | Up(Button) | Move | Wheel(Delta)
    pub buttons: Buttons,       // what is held down, which is what makes a Move a drag
    pub mods: Mods,
    pub at: Instant,
}

pub struct Paste { /* owns bytes */ }
impl Paste {
    fn bytes(&self) -> &[u8];
    fn text(&self) -> Cow<'_, str>;     // lossy, and it says so
    fn truncated(&self) -> bool;
}

pub enum MouseMode { Off, Buttons, Drag, Motion }    // totally ordered, deliberately
pub struct InputConfig { mouse: MouseMode, focus: bool, paste: bool, paste_limit: usize }
```

`Event` **owns everything**, because `next_event(&mut self) -> Option<Event>` was already fixed by the
seam and a returned value cannot borrow the queue it came from. The only real cost is paste, which is
the one variant that allocates and the one that is rare.

### The keyboard, and the uniform model that was refused

Three facts decide this and none is ours to fix (01): without kitty flag 2 a **key release never
arrives at all**; without it **auto-repeat is indistinguishable** from a fast series of presses; and even
at kitty baseline **Enter/Tab stay ambiguous** with Ctrl+M/Ctrl+I, because the spec carves them out on
purpose so that `reset` remains typeable after a program crashes with the mode set.

The tempting design papers over the first two: synthesise a `Release` when the next press arrives, fold
`Repeat` into `Press`, and let every application be written once. **It is refused**, and the reason is
placement rather than principle — a synthesised release has no honest timestamp. The key was released at
*some* moment between two presses and the engine knows which second, not which millisecond. A component
drawing a held key would show it held until the next keystroke, which on an idle form is forever.
See [ADR 0007](../../docs/adr/0007-the-input-model-is-honest-about-the-terminal.md).

So `KeyKind` is `Press | Repeat | Release`, an application learns from `Capabilities` which of them it
will actually see, and a terminal that reports only presses produces only `Press`.

- **`code` is the base layout, `text` is what was produced.** A shortcut is about where the key is; text
  is about what it printed. Reporting only one collapses `Ctrl+Shift+5` on a non-US layout into something
  the application cannot bind. kitty flag 4 supplies both; where it is absent, `text` is what the
  terminal sent and `code` is inferred from it.
- **`text` is a grapheme cluster, never a `char`, and it is inline.** §3's constraint applies here as
  much as in a cell: kitty flag 16 reports the *codepoints* a key would produce, which is a sequence.
  Inline rather than `String` because a keystroke must not allocate.
- **All eight modifiers, including caps and num lock.** Dropping hyper and meta is free now and
  impossible later, and lock state is what separates `Ctrl+C` from `Ctrl+Shift+C` on layouts where the
  distinction is real. Lock state is keyboard *state* and is never part of a chord.

### Intent, and the one thing that is dropped

Ticket 09 wrote *input may not be dropped*. That rule is narrowed rather than kept, and the narrowing is
an ADR because it is the exact opposite of what the surrounding design says everywhere else:

> **Consecutive mouse moves coalesce.** An intermediate pointer position carries no intent — it says
> only where the pointer was on the way — and an unbounded queue fed by a waving hand is the one path by
> which a terminal can eat an application's memory. A press, a release, a wheel turn, a keystroke and a
> resize all express something the user meant and are never dropped.

See [ADR 0008](../../docs/adr/0008-intent-is-never-dropped-position-is.md). Refusing to read the tty
instead would be worse than it looks, because the same channel carries the capability replies.

**The queue grows, and nothing caps it.** A terminal without bracketed paste delivers a pasted megabyte
as key events, so growth is real; it is bounded by the paste and drains itself. Unbounded growth means a
chronically slow application, which is §11's subject and already has a detector. Dropping oldest events
would repeal the rule for a case that resolves in seconds.

### The mouse

`MouseMode` is `Off < Buttons < Drag < Motion` — modes 1000, 1002, 1003 — and the ordering is a design
decision rather than an accident of numbering: each strictly contains the one below, so **combining what
several components want is a `max`, not a set union**. SGR encoding (1006) is always on when the mouse
is on, because without it coordinates stop at column 223 and a 300-column screen exceeds that.

**`Screen::set_mouse(MouseMode)` is the engine's entire actuator, and it carries an obligation the spec
states rather than implies: it is idempotent, and free when the value has not changed.** That is not
politeness, it is load-bearing — the runtime calls it after every frame, and a naive implementation
writes an escape sequence per frame for ever.

The mechanism above it is the runtime's and is pinned here because the engine's obligation is meaningless
without it. **There is no mount**: a component is a function, its state travels beside it, and nothing
holds a registry of live components. So the declaration rides the draw, and it is the *same call* as the
hit region. Three properties follow, and the third is why the shape won:

- **A widget scrolled out of view does not draw, so does not declare, so does not pay.** The same culling
  that keeps frame cost proportional to visible cells keeps input cost proportional to visible widgets,
  for free, with no second mechanism.
- **Interest and location are inseparable**, because they are one call. There are no two things to keep
  in agreement.
- **Resolution is two-level.** An event finds its layer through the engine (`topmost_at`, n is layers,
  13.5 ns at twenty) and its widget through the runtime (n is widgets, which is why the index is theirs).

**`Config::input` is a floor, not a setting.** The union is known only after a draw, so the frame that
first paints a hover-wanting modal did not yet have tracking on. An application that knows it wants the
mouse says so once and has no blind frame; one that does not, pays nothing.

**A terminal without a mouse takes `set_mouse` silently.** Clamp-and-discard everywhere, never a
`Result` — here for its own reason: an application that did not ask `capabilities()` will not handle an
error usefully, and one that did already knows.

### Three permanent consequences, recorded rather than mitigated

- **`mouse-enter` / `mouse-leave` do not exist on the wire** and cannot be engine events: the terminal
  sends positions, and turning a position into "entered this field" needs the widget index. The runtime
  diffs this frame's hit against last frame's — the same shape as §8's mirror.
- **The pointer can cross a widget entirely between two frames and generate no enter/leave at all**,
  because moves coalesce and the frame clock holds them to the gap. Invisible by construction, since a
  highlight lasting zero frames cannot be seen.
- **No protocol has a "the mouse left the terminal" event**, so a hover highlight can stick when the
  pointer leaves the window. Synthesising one from focus loss was considered and refused for the same
  reason as the synthetic key release: focus and pointer position are different things, and gluing them
  makes the highlight flicker on alt-tab. *This is a claim about other people's software and is listed
  in §15 as unverified.*

**Hover answers one frame late**, because draw order *is* z-order within a layer and the topmost claimant
is known only when the frame is finished. Answering immediately from rectangle containment would light
up two overlapping widgets at once, which is a visible defect exactly where overlap is the point.

**Double-click detection is not the engine's.** It is policy with a tunable threshold and it belongs where
hit-testing belongs. **Every event carries `at: Instant`, stamped by the input thread at read time**,
because the app thread cannot recover it: between the read and the handler sit the frame clock's hold
(7.3 ms at 120 Hz) and the application's own slowness. The runtime's per-frame clock sample is the right
clock for animation and the wrong one for a double click, where two events inside a single coalesced wake
would be indistinguishable.

### Paste

`Paste` **owns bytes**, not a `String`, and offers `text() -> Cow<str>` beside `bytes()`.

Pasted bytes are not guaranteed to be UTF-8 under any answer: a file in latin-1, a binary, a multi-byte
sequence cut by a read boundary. A `String` forces the engine to resolve that silently, and silently
means either a panic or corruption. The asymmetry with §4 is deliberate: **at the drawing verbs the
engine may demand well-formed input from its caller; at the input boundary it may demand nothing** — a
paste is composed by the world, not by our API.

- **A paste has a configurable ceiling**, and exceeding it sets `truncated()`. Silent truncation is data
  loss the user will attribute to the application, so the flag belongs on the event and not in a
  diagnostic.
- **An unterminated paste is not rescued by a timer.** If the closing marker never comes, the channel is
  already broken and the application is about to learn that anyway; a timeout here would be a timer the
  idle budget dislikes, bought to paper over a dead connection.

### Negotiation, and everything else

- **Keyboard enhancement flags are requested together, never cherry-picked**, per kitty's own instruction
  that implementing a subset makes no sense. They cost nothing in idle.
- **Mouse tracking, focus reporting (mode 1004) and bracketed paste (mode 2004) are requested only if the
  application declared them**, because each converts an idle application into a woken one — focus
  reporting on every alt-tab, motion tracking on every pointer move. This is standing requirement 11
  reaching into startup. Focus reporting in particular is **opt-in and off by default**.
- **The DA1 sentinel answers "did the negotiation take" without a stall**, so nothing in the input path
  needs a timeout of its own.
- **Unrecognised escape sequences are dropped, counted, and the last one kept for diagnostics.** There is
  nothing to hand upward, but silent discard is the defect class that costs a day: "Shift+F5 does
  nothing" with no thread to pull. Two fields buy the thread.
- **Restoration includes input state** (§7).

### What `Capabilities` must answer about input

Eight **separate booleans and not a tier**: `key_release` · `key_repeat` · `alternate_keys` ·
`associated_text` · `mouse` · `mouse_motion` · `focus_events` · `bracketed_paste`.

A `KeyboardTier::{Legacy, Disambiguate, Full}` would read better and would lie: tmux forwards the `CSI u`
*encoding* while implementing none of the flag stack, and WezTerm ships the protocol switched off by
default. Those are not points on one scale. Exposing raw kitty flags would be crossterm leaking through a
different door. See [ADR 0010](../../docs/adr/0010-detected-axes-are-flat-declared-axes-are-ordered.md).

---

## 10. Capabilities and degradation

**The engine degrades presentation and never content.** See
[ADR 0009](../../docs/adr/0009-the-engine-degrades-presentation-never-content.md). (11)

Colour is quantised and an unsupported attribute is dropped, both **silently, at serialise time**. A
glyph is **never** touched, and the component branches. That is not a gap between two tickets; it is
one rule with two consequences, and ADR 0007 is the same rule a third time, on input.

**The mechanism underneath the asymmetry is detectability.** Colour is measured — XTGETTCAP `RGB`, DA,
OSC 11, OSC 4. **Glyph repertoire cannot be detected at all**: no query asks whether `U+28FF` is in the
font, the terminal accepts braille and draws tofu, and none of the detection machinery sees it. So
`--ascii` is not a discovery, it is an **operator's promise**, and a component branching on it is the
only place that promise can be kept.

**A third case sits between the two and had gone unnamed until ticket 22**: **OSC 8 has no query
either**, and unlike the font it is not something an operator was ever asked to promise, so
`hyperlinks` is *inferred* — matched against an allow-list of terminal identities. An inference is the
engine guessing, which ADR 0025's rule governs (*refuse when a guess would be wrong in direction*), and
the direction here is asymmetric: a wrong `false` costs a hyperlink that is not offered, a wrong `true`
costs a component branching on a capability it does not have. **An inferred axis must therefore be
declarable**, because a declaration is the only thing that can correct it — there is no second query,
and a quirk-table entry is a new release.

From which falls a second rule with teeth in it:

> **A detected axis must be flat booleans; a declared axis may be an ordered ladder.** The world does
> not sort; a promise is downward-closed by whoever makes it.

See [ADR 0010](../../docs/adr/0010-detected-axes-are-flat-declared-axes-are-ordered.md). `ColorDepth`
is ordered *and* detected because the wire formats genuinely nest.

### The shape

```rust
// --- what is true, read after attach, immutable for the life of the Screen ---
pub struct Capabilities {
    pub colors: ColorDepth,              // detected — a ladder, and an honest one
    pub glyphs: GlyphSet,                // declared — never detected
    pub default_fg: Option<Rgb>,         // OSC 10 ⎫ None is §5's silent path
    pub default_bg: Option<Rgb>,         // OSC 11 ⎭
    pub hyperlinks: bool,                // OSC 8 — *inferred*: there is no query (22)
    pub grapheme_clusters: bool,         // mode 2027
    pub key_release: bool,      pub key_repeat: bool,
    pub alternate_keys: bool,   pub associated_text: bool,
    pub mouse: bool,            pub mouse_motion: bool,
    pub focus_events: bool,     pub bracketed_paste: bool,
}
impl Capabilities { pub fn report(&self) -> String; }   // diagnostics, not a field per fact

pub enum ColorDepth { None, Ansi16, Indexed256, TrueColor }
pub enum GlyphSet   { Ascii, Unicode, Extended }

// --- what was asked for, set before attach ---
pub struct Overrides {                   // every field PINS; None means "let detection decide"
    pub colors: Option<ColorDepth>,
    pub glyphs: Option<GlyphSet>,
    pub default_fg: Option<Rgb>,         // OSC 10 ⎫ Some pins an answer; silence is not
    pub default_bg: Option<Rgb>,         // OSC 11 ⎭ declarable, because it is the default
    pub hyperlinks: Option<bool>,        // OSC 8 — the one way to correct an inference
    pub legacy_sgr: Option<bool>,        // SGR 38/48 — the colon form ships; this pins the semicolon one
    pub width: Option<WidthSource>,
}
impl Overrides { pub fn plain() -> Self; }   // Some(ColorDepth::None), Some(Ascii) — `--ascii --no-color`
```

**Seven fields, and the set is a rule rather than a list** (22). `Overrides` is not a subset of
`Capabilities` and never was — `legacy_sgr` and `width` pin *private* facts — so the shape it had to
be argued away from was never *fourteen facts mirrored as fourteen `Option`s*. The line is:

> **An axis belongs on `Overrides` iff (a) nothing measured it — nothing can, or nothing did and the
> engine inferred it — or (b) the engine's own output depends on it *and* the value is one the person
> at the terminal knows.**

That is the write-side twin of the read-side test below. `Capabilities` asks *can someone above act on
it*; `Overrides` asks *can someone below be told it, and does telling it change what the engine
produces*. Two questions, two field sets.

**`hyperlinks` is the case that forced the rule, and it is a third category the shape above had been
hiding.** ADR 0010 splits *detected* from *declared*; **OSC 8 has no query**, so `hyperlinks` is
neither — it is **inferred**, the engine guessing on the world's behalf from what XTVERSION reported.
An inference is the one kind of fact a declaration must be able to correct, because there is no second
query to ask more carefully and a quirk-table entry is a new release. The shipped inference's residual
failure is named where it happens: VTE implements OSC 8, answers no XTVERSION, and gets a wrong
`false`.

**`default_fg` and `default_bg` are on `Overrides` by (b), and they are the only fields there whose
reader is the compositor** rather than the serializer (ADR 0025). Silence is not declarable and does
not need to be: it is what a caller-supplied sink gets by default and what every unanswered terminal
falls through to, so the arm that needed a door was the answered one, and one `Option` reaches it. The
consequence, stated rather than discovered: an application cannot force a terminal that *did* answer
OSC 11 to be treated as silent, and nothing wants to.

**`legacy_sgr` is the axis this spec disagreed with itself about, and this is where its default lives.**
The parameterised forms of SGR 38, 48 and 58 exist twice over: ITU-T T.416's colon spelling,
`38:2::r:g:b` and `38:5:n`, and xterm's pre-ITU-T semicolon spelling, `38;2;r;g;b` and `38;5;n`. **The
colon spelling is what a terminal with no quirk entry and no declaration receives** — `legacy_sgr` is
`false` at level 7, and `Overrides::legacy_sgr`, `VITUI_FORCE_LEGACY_SGR` and the four level-5 quirk
entries are the only things that move it — ConPTY, Termux, VSCode and JetBrains' IDE terminal, and it
read *three* until production ticket 14 counted them against `lookup`, the JetBrains entry having
arrived after the sentence. **This is the only sentence in the spec that states the
default**: §8 describes both spellings' bytes and names neither as shipped, because §8's tables were
taken on the prototype, which emitted semicolons.

The argument that settles it is not a reading of either section — it is the type. `Quirks::legacy_sgr`
is a `bool`, not an `Option<bool>`, and `Quirks::apply` reads `if self.legacy_sgr { … = true }`: **the
table can only ever turn the flag on.** A one-way override is coherent in exactly one direction, and
§8's reading would need the four entries to force a value their terminals already have, through a field
that cannot express the opposite. `Underlines` is the same shape one parameter along — `#[default]
Standard`, and `apply` writes only `ConPty`. **SGR 58 is therefore a separate axis**: `underlines` is
reached only from the quirk table, because a terminal can want the semicolon form for 58 and never be
asked about 38.

**Two of tier 1's seven have been observed to parse the colon spelling, and that is a narrower count
than any other on this denominator.** Ghostty 1.3.1 was sent
`ESC[38:2::255:0:0m` and `ESC[1;4;58:2::0:0:255m` and returned the exact channel values for both — the
colon form parsed correctly for SGR 38 *and* 58 — on macOS 26.5.2, 2026-08-22. **tmux 3.7c** was sent the
same two and returned the same channels, on 2026-08-22, which is why the tmux quirk entry added by
production ticket 05 sets `attrs_dropped` and pointedly not `legacy_sgr`. The remaining five are still
inference from libvaxis's three quirk entries, and §15's per-terminal quirk table is where a wrong
answer lands.

**Two, and not the six §15 counts, because these are two different questions on one set of
terminals.** Six of the seven have been driven through `conform/`'s scene 01, which asks about the
**eleven attribute bits**; this axis is the **colon SGR spelling**, and the scene that would ask a
terminal about it — `conform/SCENES.md`'s scene 03 — is proposed and unbuilt. So the two figures may
not be read across, and neither may be taken for *how many terminals have been run*. Production
ticket 14 is where the three were separated and named.

**Nine of the fourteen are excluded, and so is every private fact but the two that already had a field —
each by the rule rather than case by case.** The eight input
facts fail (b) twice: nothing on the composite→pack→serialise path reads one, and *a declaration cannot
make an event arrive* — `key_release: Some(true)` on a terminal that sends no releases would leave a
component drawing a key it believes still held, which is exactly the defect
[ADR 0007](../../docs/adr/0007-the-input-model-is-honest-about-the-terminal.md) refused the uniform
keyboard model for. `grapheme_clusters` fails (b) because the sentence beside it in the block above is
load-bearing: *our tables stay authoritative either way*, so nothing in the engine reads it and a
component reading it is not the engine. Of the private facts, `decslrm` fails (b) because §15 says in as
many words that nothing in the shipped serializer depends on the answer, and `palette` fails it because
silence there has a *defined* answer — the xterm table — so both arms already run and a declaration
would move values rather than reach a path. `sync_output`, `underlines`, `attrs_dropped` and
`sync_flush` fail (b)'s second half: nobody at the terminal can name them, and their arms are reached
from inside the crate through `assemble` (see *the two doors*).

**Any of them gains a field the moment one of the two sentences becomes true of it**, which is a visible
event in a diff and not a judgement call. That is the stopping line: not a count, a query over the code.

**The word *tier* does not survive on the read side.** `Config::tier: TierRequest` becomes
`Config::overrides: Overrides`. A tier was defined as "a named combination of capabilities a scene can
be rendered against", and after this shape no such combination is read by anybody — **fourteen separate
facts** are, two of them ordered. The named combination survives only as `Overrides::plain()`, because
`--ascii --no-color` genuinely travel together and a constructor is the right size for that.

**Every public field had to pass one test: someone above can act on it.** What that test excludes is the
finding. Synchronised output, `DECSLRM`, legacy SGR, the ConPTY underline-colour escape form, the
force-flush limits and the 16-entry palette are all *known* and all *private*, surfaced only through
`report()`.

**The eleven attribute bits are not on `Capabilities` at all, because they are not detectable either.**
There is no query for "do you render italic": DECRQM does not cover SGR, and XTGETTCAP was deliberately
narrowed to the single `RGB` capability precisely because terminfo descriptions are what detection
refused. Eleven attribute booleans would be eleven inventions. Attributes live in the **quirk table**,
and an unsupported one is dropped silently at serialise time. Nothing above needs to ask, because an
absent attribute still draws the correct text — that is the whole content-versus-presentation line. The
one monochrome mechanism a component will actually lean on is `reverse`, and it works everywhere.

### Detection: query the live pty, never terminfo

terminfo describes what `$TERM` claims, not what the terminal on the other end implements, and it cannot
express mode 2026 or kitty-keyboard levels at all. libvaxis is the production proof the other road
works; tcell is the counterexample and shows its cost — suffix matching, name patterns, a "best guess"
256-colour default, and a hardcoded known-terminal profile table. (01, 17)

The probe set:

- **DA1** (`CSI c`) — not for identification, but as a **universal sentinel**. Virtually everything
  answers it, including tmux.
- **XTVERSION** (`CSI > 0 q`) — the primary identity probe where available: kitty, WezTerm, foot,
  Ghostty. **DA2** as the fallback.
- **DECRQM** (`CSI ? Ps $ p`) — for any DEC-private-mode question.
- **XTGETTCAP** — narrowly, for the `RGB` truecolor capability only.
- **OSC 10 / OSC 11** — the terminal's real default foreground and background (§5).
- **OSC 4 × 16** — palette entries 0–15, needed for quantisation at `Ansi16`.
- **DECSET 2026** (synchronised output) and **DECSET 2027** (grapheme-cluster width), requested once.

**The timeout problem is solved by the sentinel, not by tuning a number.** Fire the real queries with a
trailing DA1 and react to DA1's *arrival* rather than to a clock; a numeric ceiling of 100–300 ms exists
only for the case where there is no terminal at all. Tuning the numeric timeout alone is fragile — a
real bug (`terminal-light` against iTerm2) shows it. The seventeen queries the
degradation model added — OSC 4 × 16 and mode 2027 — cost no extra round trip, because they sit in the
same batch ahead of the sentinel.

**That citation was checked against the terminal it names, 2026-09-04** (production ticket 13), and it
is recorded here whichever way it went — a citation is not an observation, and the last time this
repository looked at one of its own with an instrument the surveyed claim failed to reproduce on three
families and then reproduced on the fourth. **It does not reproduce as a slow terminal.** `vitui-apps`'
`caps` run on iTerm2 3.6.11's own tty, three times, completed the whole attach — seventeen queries,
sixteen palette entries, both OSC colour probes, five DECRQM modes, the kitty keyboard flags and the
DA1 sentinel — in **73, 94 and 80 ms**, against an idle ceiling of 250 ms. Every answer arrived, in
order, ahead of the sentinel: the report came back with an identity (`iTerm2 3.6.11`, from XTVERSION),
a full palette and `sync_output true`, none of which a run cut short at the sentinel would have carried.

**The rule the citation supports is untouched and the citation is narrower than it reads.** What
`terminal-light` demonstrates is that a *fixed* numeric timeout is fragile, and the machine, the load
and the emulator's own startup are all in that number; nothing about it required iTerm2 to be slow, and
this measurement says only that iTerm2 3.6.11 on this machine is not. A sentinel is unaffected by which
of those is true, which is the whole reason it is the mechanism.

**`$COLORTERM` is a cheap fast path and nothing more.** It is an unstandardised convention, lost across
ssh and sudo and clobbered by tmux and screen; corroborate with a live XTGETTCAP `RGB` query and default
to 256 colours when genuinely ambiguous.

**Synchronised output is available across all of tier-1** — kitty, Ghostty, WezTerm, Alacritty ≥ 0.13.0,
iTerm2, tmux, Windows Terminal — and it does eliminate tearing when frames are wrapped correctly. Every
implementation force-flushes if a block stays open too long and the limits differ by an order of
magnitude: Alacritty 150 ms / 2 MiB, kitty 2000 ms, tmux 1 s. §8 states where that binds.

### Precedence, stated once

Highest wins:

1. **The explicit API** — `Config::overrides`. This is where an application's own `--ascii` and
   `--no-color` land; **the engine parses no argv.**
2. **`VITUI_*` environment** — `VITUI_GLYPHS`, `VITUI_FORCE_COLOR`, `VITUI_FORCE_LEGACY_SGR`,
   `VITUI_FORCE_WCWIDTH`, `VITUI_HYPERLINKS`, `VITUI_DEFAULT_FG`, `VITUI_DEFAULT_BG`. When detection is
   wrong in the field, a user needs a lever that does not require a new release. **Every `Overrides`
   field has a twin here and that is a rule, not a coincidence** (22): level 2 exists to be the same
   lever without a release, so the two sets grow together. The last three are not symmetry for its own
   sake — `VITUI_HYPERLINKS` is the answer to the inference's known residual (a VTE user's wrong
   `false`, which otherwise waits for a released quirk-table entry), and the two colours are the field
   case §5 had been carrying silently: a terminal that does not answer OSC 11 gives its user shadows
   clipped to the explicitly-coloured area for ever, and the value that fixes it is the one that user
   typed into their own terminal profile.
3. **`NO_COLOR`** — any non-empty value.
4. **`TERM=dumb`, or stdout is not a tty.**
5. **The quirk table, applied *after* detection.** Some terminals answer the query correctly and then
   misbehave: ConPTY, Termux and VSCode are forced to legacy SGR, and ConPTY gets its own
   underline-colour escape form. This is also where the attribute facts live.
6. **Live detection.**
7. **Conservative defaults.**

Only the 1–2 pair was contested. Environment above API would let a stale variable silently override a
flag the user just typed. The mechanism that removes the conflict is the shape already chosen: **every
forcing field is an `Option`, defaulting to `None`, and the environment fills only the `None`s.** A
`Some` is never overridden. `NO_COLOR` keeps its intent under this order — an explicit request through
the application's own flag is a *more* specific expression of the same user's wish than the variable is.

### Colour quantisation: placement first, algorithm second

**Quantisation runs on the render thread, inside the run scan, *before* the comparison with the
mirror.** Three things force it: quantisation is wire-specific and everything wire-specific lives below
the packet; a mirror holding colours the terminal was never sent is not a mirror; and the app thread's
budget is the scarce one while the render thread has slack.

What falls out is a correction nobody was looking for:

> **§8's "quantising colour saves almost nothing" is right about frame *size* and silent about frame
> *membership*.** If the mirror held canonical colours, two RGB values that quantise to the same index
> would compare *unequal* and be re-emitted for no visible change — the equality filter would
> under-filter by exactly the amount quantisation collapses. Quantising first makes the filter exact
> with respect to the wire, and an animated gradient on a 16-colour terminal nearly disappears from it.
> The saving is real and it is confined to *changing* colour.

Cost is per **style word**, not per cell: cells in a run share a `u64`, so a one-entry memo covers nearly
everything, and it covers an extended style's underline colour with no extra mechanism because an
extended word resolves to exactly one entry in `packet.exts`. *(The memo's cost against the filter's
0.9 ns per damaged cell is one of §15's owed measurements.)*

The algorithm is **weighted integer distance in RGB**. CIELAB does not earn a dependency the policy
forbids anyway: the choice is among fixed points, where perceptual ordering rarely changes the winner.

- **At `Indexed256`, indices 0–15 are never a quantisation target** — only the 6×6×6 cube and the 24
  greys. 16–255 are fixed by specification and identical on every terminal; 0–15 are repainted by the
  user's theme. Quantising into the cube is deterministic; quantising into 0–15 is a bet on someone
  else's colour scheme.
- **At `Ansi16` there is no such escape, which is what makes OSC 4 worth its queries.** On silence, the
  xterm default table is used.

**The OSC 11 / OSC 4 silence asymmetry is one rule, not an inconsistency**, and it is the rule for any
future query:

- **OSC 11 silence → refuse to mix.** A guessed background inverts a shadow on the opposite theme.
  Wrong *direction*.
- **OSC 4 silence → use the standard table.** A themed palette makes the nearest match slightly off.
  Wrong *degree*.

> Refuse when a guess would be wrong in direction; default when it would be wrong only in degree.

### Contrast preservation is refused, and the reason is mechanical

A shadow that quantises into its own background disappears. The fix everyone reaches for is
context-aware quantisation — nudge the darker of two adjacent colours so they stay distinct — and it
cannot be had here. Context-dependent quantisation means two cells carrying an *identical* `u64` can
require different SGR, so a style run has to break. Style-run coalescing is the mechanism that already
collected the entire wire win, so making quantisation contextual would turn §8's −6% into a byte
*increase*, and would put a spatial concept into a serializer that has none and whose runs arrive one
row at a time.

The consequence is stated rather than mitigated: **at 16 colours a shadow over a similar background
disappears.** That is not a defect to be patched at serialise time; it is precisely the case ADR 0009
hands upward.

### Where a lifting goes when there is no colour

**At `ColorDepth::None`, compositing skips operator layers entirely** — `Mix` provably changes no byte
on the wire. Worth **78.2 µs of §5's 107 µs worst screen**. At `Ansi16` the effect is *often* nil but
not provably so, and there the operator runs.

What expresses lifting instead is not the engine's call. But the shape of the branch is worth writing
down because it is the least obvious appearance of the rule:

> **A character shadow is a content layer, not an operator layer.** It has its own rectangle, its own
> `z`, and it *occludes* what lies beneath instead of tinting it.

So the component's branch is not "pass a different parameter", it is "build a different layer stack" —
exactly as the braille chart's branch is not "substitute a glyph" but "become an area chart". A
degradation model that made this look like a parameter would have been wrong in the same way a
substitution table is wrong.

### `--ascii`: the engine's behaviour is zero, and that is the point

The engine emits no glyphs of its own. §4 leaves it three verbs and no box-drawing primitive — boxes are
`fill`s — so there is nothing for the engine to substitute even if it wanted to. The prohibition costs no
discipline; it is a property of the surface that already exists.

`GlyphSet` has three levels because a real component has three strategies: **braille is 256 states per
cell, block elements are 8 and bottom-anchored, ASCII is 2** — so a line chart is inexpressible in block
elements and the component becomes an area chart, and inexpressible again in ASCII and becomes something
else. The ladder was read off a real chart, not chosen because three felt right.

**The default is `Extended`.** `Unicode` is the safer default — braille and emoji are exactly what fonts
lack, and the failure is tofu — but it would ship the flagship capability switched off, and requirement 6
says *UTF-8 first, with `--ascii` as the degradation path*. Since nothing can be queried, the lever is
one environment variable, and that is the trade taken.

One rule has to be written down or the content/presentation line will be read too widely:

> **Choosing a glyph before drawing is legitimate; replacing one after it is drawn is forbidden.**

A border character set the runtime hands a component according to `glyphs` is a choice at the source and
is fine. The same table applied by the engine to cells a component already wrote is the thing that turns
a chart into noise while every budget stays green.

### The width disagreement, and why it is permanent

Only **7 of 23 surveyed terminals widen a VS16 emoji correctly**; kitty sums a ZWJ family emoji to width
6 where the correct answer is 2; Windows Terminal and cmd.exe treat combining marks as width 1 rather
than 0. This is not a tail case — it is the common path for any application displaying user-supplied
text. Our own UAX #29 / #11 tables stay authoritative (02).

Once the mirror exists, a disagreement stops being transient. If the terminal draws a cluster wider than
our tables assumed, it overpaints the neighbouring cell; **the mirror records what we intended, not what
happened**, so the equality filter will never re-emit that cell. The corruption is permanent until that
cell's content changes.

Three options were weighed. Marking a row containing a contentious cluster as an **unknown row** was
rejected because it does not converge — the next whole-row repaint contains the same emoji. Doing nothing
was rejected because the fix is nearly free. What is taken:

- **Mode 2027 is requested once at startup, non-blocking**, and reported as `grapheme_clusters`. It is
  implemented by kitty, Windows Terminal (as the *default* since v1.22), contour, wezterm, foot and Rio;
  Alacritty is confirmed absent. Our tables remain authoritative either way.
- **After a run that contained any non-ASCII cluster, the next move on that row is emitted as `CHA`
  rather than `CUF`** (§8). It converts a compounding error into a bounded one.
- **The residual is recorded as permanent and known**: one visually wrong cell next to a mis-measured
  cluster, persisting until its content changes.

### Capabilities are sampled once and never change

`Capabilities` is built during `attach` and is **immutable for the life of the `Screen`**, so
`Screen::capabilities()` stays `&self` and no component ever handles a tier changing between frames.

The price is explicit: **a terminal that changed underneath the process — a reconnected ssh session, a
SIGTSTP/SIGCONT cycle — cannot be re-detected without a fresh `attach`.** That is §7's *the terminal
leaves and comes back*, not a degradation question, and production ticket 07 settled it there: a
`Screen::resume` re-declares and never re-detects, and a terminal that was **replaced** rather than
briefly borrowed costs a fresh `attach` on a dropped `Screen`. Both halves are gated.

### The two doors, and the one that stops at the crate boundary

Two things other than a live terminal can produce a `Capabilities`. They sit at different depths, and
reading them as competing doors into one type is what made ticket 22's answer space look smaller than it
is:

- **`Overrides` puts a `Screen` on a tier.** It is the *only* door that does, and therefore the only
  door that reaches a frame, the wire, a golden, one of §14's twelve scenes, or any test outside the
  crate.
- **A synthetic `Detected` put through `assemble` hands a `&Capabilities` to a function that takes
  one.** It lives *below* `Screen`, inside the crate, and cannot reach a frame. It is not a second door
  into the type, because it goes through the same resolution a real tty's replies do and therefore
  cannot express a `Capabilities` detection could not have produced.

The second is what covers the axes the rule above excludes — `sync_output`, `underlines`,
`attrs_dropped`, the palette — and it is legitimate for exactly them. It is **not** an alternative to a
field for anything the rule admits, and the reason is a hard boundary rather than a preference:
[ADR 0023](../../docs/adr/0023-the-cell-is-never-visible-in-the-public-api.md) keeps cells off the public
surface, so the gates whose only observable is bytes live in an integration test, which is another crate. **An axis a gate outside the crate has to pin has one door and
only one.**

### Headless, and a non-tty

**Headless is a fully *declared* tier, not the lowest one**: detection switched off, **every declarable
axis pinned** through `Overrides`, output into a caller-supplied sink. Then any tier a caller can name
is testable, truecolor included, which is what a floor tier could never have provided. What is not
declarable is either something a headless run could not exercise if it were — an event that has to
arrive — or a private fact nobody at the terminal can name, reached through `assemble` from inside the
crate.

*This sentence read "every axis pinned" until ticket 22, and it over-promised because of a
mis-citation*: it justified itself with §14's *nine option sets*, which were ticket 08's prototype
**serializer knobs** and not tiers at all. The engine ships one configuration and no `Options` type
reaches the public surface, so the clause cited a matrix that no longer exists to justify a promise
about a different axis. The reason above is the real one.

**A non-tty stdout is a separate thing and is not headless**: it is a terminal nothing is known about. No
queries are fired — there is nobody to answer, and the DA1 sentinel would burn the whole 100–300 ms
ceiling for nothing — so everything falls through to declared defaults: `colors: None` (escape codes in a
log are noise), `glyphs: Extended` (nothing lowered it, and lowering it unasked would be an invention),
mode 2026 off, no input.

### One narrow exception to "degrade at serialise time"

A channel the terminal **cannot express at all** is dropped from the intern *key* on the app thread; a
channel it expresses **imprecisely** is degraded at serialise time on the render thread. Colour is always
the second kind, including an extended style's underline colour.

The one case measured is a hyperlink on a terminal without OSC 8, and the result is a negative one: both
placements produce **byte-identical frames at 88 071 bytes**, because two style words differing only in a
channel the serializer will not emit produce an SGR delta with nothing in it, and the emit loop already
withdraws the escape it speculatively opened. What the collapse is worth is **8 table entries against
96** — a growth bound, not a frame cost — and it is free, so it is taken. This is *not* the
quantise-before-compare finding repeating: that one changed frame membership; this one does not.

**8 against 96 is a two-arm measurement, and `Overrides::hyperlinks` is what gives it two arms** (22).
The 96 needs a screen on which OSC 8 is expressible and the 8 needs one on which it is not; every test
is headless, so before the field there was one arm and a bound with nothing to bound against. The same
sentence is why the collapse's *other* consequence has to be pinned deliberately rather than inherited:
a scene whose subject is table growth — §14's twelfth — measures 8 entries instead of 96 the moment the
collapse lands, unless it declares the axis the way it already declares the depth.

---

## 11. Enforcing the unblockable app thread

> **The compiler cannot stop the app thread from being slow. It can stop anything else from being the
> app thread.** (18)

That inversion is the whole of standing requirement 9's answer. The offence is defined as **a frame-budget
overrun by the app thread's iteration, whatever caused it** — not as "a blocking syscall". Android's
`NetworkOnMainThreadException` picks the syscall and therefore catches network while waving through a
`for` loop that takes 400 ms; a frozen interface is a frozen interface, and a pure slow function freezes
it identically. The syscall set survives as the *lintable subset* of that definition, not as the
definition.

| rung | mechanism | catches | cost |
|---|---|---|---|
| compile | split handles, `!Send` halves | wrong thread waiting, producing, drawing | **0** |
| compile | no blocking primitive on the app-thread side | waiting on a background result | **0** |
| lint | a shipped `clippy.toml` fragment, **opt-in by the application author** | `fs`, `net`, `sleep`, `join`, `recv`, `lock` | 0 at runtime |
| runtime | in-loop `Perf`, both profiles | any overrun, after the fact | **42.6 ns/frame** |
| runtime | observer thread, **debug only** | an iteration that never returns | 369 ctx switches / 30 s |

### Split handles, not a threaded token

See [ADR 0003](../../docs/adr/0003-split-handles-not-a-capability-token.md). Each of ticket 09's two
primitives became two types, so that an unreachable method is simply **not on the type**:

| primitive | app-thread half | the other half |
|---|---|---|
| the wake source | `Parker` — `!Send`, owns `wait` and `request_wake_at` | `Unparker` — `Send + Sync + Clone`, owns `post` and `quit` |
| the mailbox | `Producer` — `!Send`, owns `lease` and `submit` | `Consumer` — `Send`, `!Sync`, owns `take` and `finish` |

**A capability token was measured as a verb parameter and is free** — a realistic frame is 152.77 µs
plain and 151.09 µs with a `&Ui` carried into every verb, a difference in the token's favour. It was
still rejected *in that shape*: it buys nothing over a private `PhantomData<*const ()>` field, and it
taxes every call site and every component signature, which requirement 10 forbids. It survives only as
that private field and as what `attach` mints.

**`Engine` is `Send`, built before any thread exists; `Engine::attach()` consumes it on whichever thread
is going to be the app thread**, minting the `!Send` halves there. `attach` is the only mint, so no
public constructor and no `unsafe` exists — and the app thread need not be `main`, because the `Send`
builder travels and the `!Send` handles are created at the destination.

**After ticket 12 the split is internal.** `attach` spawns the render and input threads and `present`
owns the whole sequence, so `Parker`, `Producer`, `Consumer` and `Ui` are not public names at all; what
remains on the public surface is `Screen` (`!Send`) and `WakeHandle` (`Send + Sync + Clone`, two verbs:
`post`, `quit`). The enforcement is *strengthened* by that: `Perf::enter`/`leave` are not merely
unforgettable, they are uncallable, because `wait` and `present` are the only things that call them.

### The ticket's real yield: two holes under a headline invariant

Ticket 09's *"with one producer and the ready-gate, the slot is always empty at submit, so a packet can
never be superseded"*, asserted over 10 000 cycles, is a claim about behaviour **with one producer**, and
nothing made one producer true. Two tests were written and **both passed, which means both were holes**:

- **A worker thread could `lease` and `submit`.** The supersede counter was defended by convention only.
- **A worker thread could call `wait()` and consume a wake the app thread never saw.** The thief received
  the event. That is exactly the silent freeze this rung exists to make unrepresentable: no CPU, no log,
  no wakeup, and the app thread waiting for something already gone.

The wake source was in fact *self-contradictory* as one type — the posting verb must be `Sync` to be
callable from a worker and `wait` must not be — which is the structural reason the split is not merely
tidier. After it, six negative cases fail to compile (five `E0277` on `*const ()`, one on `Cell<()>`).
**Splitting costs nothing measurable.**

### The lint rung, and its measured ceiling

**A library cannot ship this lint to its users.** Verified with a control rather than assumed: a
two-crate probe where `clippy.toml` sits in the dependency produces no diagnostic, and the same code with
the file in the application produces `warning: use of a disallowed method std::thread::sleep`.
`clippy.toml` is read from the crate being linted, and there is no stable mechanism for a dependency to
inject lints downstream.

**So the spec must say plainly that this rung is the application author's, not the engine's.** What ships
is a ready `clippy.toml` fragment with `disallowed-methods = ["std::fs::*", "std::net::*",
"std::thread::sleep", "std::thread::JoinHandle::join", …]` plus the `#![warn(clippy::disallowed_methods)]`
line, in the docs and in a template example, and wired into vitui's own CI — which protects vitui, not
its users. A dylint driver was rejected: an external dependency and a tool nobody installs cannot be a
gate.

### Detection: in loop first, an observer only in debug

`Perf::enter` after `wait` returns, `Perf::leave` after the frame is submitted. Both live on `!Send`
types, so the measured interval is provably the app thread's, and the measurement is automatic rather
than something the runtime must remember to call.

- `Instant::now()` **19.42 ns**, `elapsed()` **25.05 ns**, `enter` + `leave` **42.64 ns**.
- Against the *cheapest possible* iteration — a one-cell cursor move at 28.61 ns — the detector is 2.4×,
  which is the honest worst framing. In absolute terms it is 40.8 ns, or **2.5 µs per second at 60 fps**.
- **It stays in release.** 0.04% of the frame budget is worth a diagnostic that reaches a user's bug
  report.

The in-loop detector cannot, by construction, see the failure it exists for: an iteration that never
returns never reports. Only an awake observer can, and the idle guarantee is what that costs — measured
over 30 s: no observer 10 context switches, a 100 ms poll 369, a 10 ms poll 2 792. **CPU is not the
objection, wakeups are**: at 100 ms the CPU cost is below `/usr/bin/time`'s resolution at 3 s, but it
converts zero wakeups into ~12 a second for ever, and standing requirement 11 names that property
literally. **The observer is `cfg(debug_assertions)` only**, so requirement 11 stays a release-build
property.

### The threshold is one frame, and 100 µs would have been a bug

**The map's < 100 µs is a CI gate on one stage of the engine's own work, not a runtime threshold for the
application's whole iteration.** A full realistic `wake → submit` — drawing, compositing under 8 layers,
packing — is **166.76 µs**. A 100 µs watchdog would fire on entirely legitimate frames.

The threshold is **one frame interval**, the same number as `Config::max_frame_rate`: 8.3 ms at 120 Hz,
3.3 ms at 300 Hz. An overrun then means a **dropped frame**, which is the event a user can actually
perceive, and 166.76 µs is 1.0% of it — 100× of headroom and no false positives. Caller-configurable.

### The sanction, and the defect the compiler found in the escape hatch

**Debug: panic on the first overrun. Release: warn once, into a sink the caller supplies.** A panic is
safe here by construction: restoration is idempotent, guarded by one atomic, callable from any thread,
and runs before the default hook prints (§7). Panicking on a *sustained* overrun instead was rejected —
it hides the single 300 ms file read that is exactly the bug being hunted, and it needs a tuned N for
which there is no measurement.

First-violation strictness requires an escape, or every application's cold-start frame panics in debug:

```rust
let _g = screen.permit_slow("loading config");   // 4.20 ns
```

**The first shape of this did not compile**, and the correction is recorded rather than quietly fixed:
`permit_slow(&mut self) -> Permit<'_>` holds a mutable borrow for the guard's whole lifetime, so `enter`
and `leave` are unreachable inside the very region the permit exists to excuse — `E0499`. The fix is
`Cell` and `&self` throughout, which costs nothing and is what precedence rule 4 asks for anyway.

**The observer's sanction in debug is different**, because panicking an observer unwinds the wrong stack
and does not stop the app thread: **restore the terminal, print the stall with the iteration's entry time
and its permit reason if any, and abort.** Restoring and continuing is broken — a returning app thread
would then paint frames into a restored terminal — and printing without restoring puts the message in an
alt screen where it may never be seen.

### What is offered instead of blocking

`WakeHandle` and a `Slot<T>` whose only accessor is a non-blocking `take`. **There is no `recv`, no
`wait`, no `Future`, and no completion returned by anything on the app-thread side** — that absence is
the second compile-time rung, and it is checkable by reading the API. "No backpressure, no way to wait
for a frame to be painted" generalises to background work unchanged.

A `Worker::spawn` sugar over `std::thread` belongs to the **runtime**, not the engine, and the numbers
are documented with it so nobody puts it in a draw path:

| shape | p50 | p90 | p99 |
|---|---|---|---|
| `thread::spawn`, **charged to the app thread** | **9.50 µs** | 21.75 µs | 30.46 µs |
| spawn → work → wake → take, end to end | 28.75 µs | 48.50 µs | 67.83 µs |
| resident thread + channel send | **2.83 µs** | 5.79 µs | 9.21 µs |

9.5 µs is nothing once per keystroke and 10% of the frame budget once per frame; the API's shape decides
which one gets written, which is why the number is in the docs. **A worker pool is refused**: it is an
executor under another name.

### What is conceded, in as many words

- **`thread::sleep`, a slow pure function, or a 10-second `for` loop.** No type prevents them. The
  in-loop detector reports them *after* the frame is lost, and the lint catches only `sleep`, only if the
  application author opted in.
- **An iteration that never returns.** Invisible to the engine entirely; only the debug observer sees it,
  so a release build hangs silently. This is the one place Android is stricter, and it is stricter because
  it can afford a permanently awake system service.
- **A component author who blocks inside `draw`.** They hold a `View`, `View` is `!Send`, and none of that
  stops `std::fs::read`. The overrun is attributed to the frame, not to the component.
- **Deliberate misuse from inside the render loop** — out of scope by the map's precedence rule 5.

A hostile component **cannot block the app thread by drawing** — every verb is bounded by the clip, and
`visible_rows` bounds the loop. It can block it by *computing*: 130 µs of unmemoised aggregate over 1M
rows, 9.76 ms of O(k) tree walking. Both trip the detector, and both are the runtime's to prevent.

### What propagates upward, without deciding anything

`wait() -> Wake` plus "work goes elsewhere and posts" is neutral between a TEA loop and a signals graph,
which was the point of refusing the executor-mediated option. One constraint does propagate: **the
boundary at which a background result enters the reactive layer is a single `Wake::Posted`, never a
callback on a worker thread** — now unrepresentable rather than merely discouraged, since nothing `!Send`
travels.

---

## 12. The public API — and what is deliberately absent

**Twenty-one public types and about sixty-three functions. No traits.** The surface is read by the runtime
author; a component author reads none of it. (12, amended by 05, 10, 11, 14, 16, 21)

> **Those two counts are ticket 12's and were never true of the block below them**, which declares
> forty-one types. What the surface actually is lives in one place and is gated there — the `built`
> row of `crates/vitui-engine/src/audit.rs`'s module doc — and the deltas this chapter has accrued
> are at the end of it, under *`prototypes/seam.rs` is four tickets behind*. **No traits** is the one
> clause of this paragraph that is still exactly true, and its checkable form is narrower than it
> reads: see `audit::tests::no_trait_of_this_crates_own_is_behind_a_dyn`.

```rust
// --- lifecycle ---------------------------------------------------------------------------------
pub struct Config { packets, max_frame_rate, resolver, overrides, input, clock }   // Default + struct update
pub struct Engine;                        // Send. Built before any thread exists.
impl Engine { fn new(Config) -> Engine; fn attach(self) -> Result<(Screen, WakeHandle), AttachError>; }

pub struct Screen;                        // !Send. The app thread's whole world. Drop restores.
impl Screen {
    fn wait(&mut self) -> Wake;                       // paced — ADR 0004
    fn next_event(&mut self) -> Option<Event>;
    fn layers(&mut self) -> &mut LayerStack;
    fn present(&mut self) -> Presented;
    fn size(&self) -> (u16, u16);
    fn request_wake_at(&self, Instant);
    fn set_max_frame_rate(&mut self, f32);
    fn set_mouse(&mut self, MouseMode);               // idempotent, free when unchanged
    fn set_cursor(&mut self, Option<Cursor>);         // applied by present, after the last write
    fn permit_slow(&self, &'static str) -> Permit<'_>;
    fn capabilities(&self) -> &Capabilities;
}

pub enum Wake { Input, Posted, Deadline, Quit }
pub struct Presented { submitted: bool, coalesced: u32, discarded_for_resize: bool }
pub struct WakeHandle;                    // Send + Sync + Clone. Two verbs: post(), quit().
pub struct Slot<T>;                       // put() on a worker, non-blocking take() on the app thread
pub struct Cursor { x, y, shape }
pub enum Clock { System, Manual }

// --- drawing -----------------------------------------------------------------------------------
pub struct Surface;   fn new, size, root
pub struct View<'a>;  fn size, child, scrolled, visible_rows, visible_cols, text, set, fill, restyle
pub struct Rect; pub struct Style; pub struct Color; pub struct Restyle<'a>;
pub enum Link<'a> { None, Uri(&'a str) };        // the URI at the verb; no handle is public (21)
pub struct Written; pub enum Stop;

// --- text (ADR 0005) ---------------------------------------------------------------------------
pub fn graphemes(&str) -> impl Iterator<Item = (&str, u16)>;
pub fn width_of(&str) -> u16;

// --- compositing -------------------------------------------------------------------------------
pub struct LayerStack;
impl LayerStack {
    fn add_content(&mut self, z, Rect, opaque) -> LayerId;                 // allocates the surface
    fn add_content_with(&mut self, z, Rect, opaque, Surface) -> LayerId;   // donated, e.g. from a worker
    fn add_operator(&mut self, z, Rect, Mix) -> LayerId;
    fn remove, set_z, set_rect;
    fn view(&mut self, LayerId) -> Option<View<'_>>;                       // the only way to a layer's cells
    fn topmost_at(&self, x, y) -> Option<LayerId>;
    fn len, is_empty;
}
pub struct LayerId; pub struct Mix { toward, amount }; pub struct Resolver;

// --- input (§9) --------------------------------------------------------------------------------
pub enum Event; pub struct Key; pub struct Mouse; pub struct Paste;
pub enum KeyCode; pub enum KeyKind; pub struct Mods; pub struct Buttons;
pub enum MouseKind; pub enum MouseMode; pub struct InputConfig;

// --- capabilities (§10) ------------------------------------------------------------------------
pub struct Capabilities; pub struct Overrides; pub struct Rgb;
pub enum ColorDepth; pub enum GlyphSet; pub enum WidthSource;

pub mod prelude { Color, Config, Engine, LayerId, Rect, Screen, Style, View, Wake }
```

### What the runtime brings

- **A rectangle and a `z` for every layer, every frame.** No default z, no automatic stacking, no sizing.
- **The loop.** `wait` → handle → draw → `present`. The engine offers the pieces and paces them; it does
  not run them.
- **Culling.** Deciding not to draw what `visible_rows()` excludes is the caller's job.
- **Time, and the whole animation model.** The runtime samples `Instant::now()` once per frame, puts it
  in its draw context, accumulates the deadlines its components ask for, and flushes the earliest through
  `request_wake_at` once, after drawing. Easing, timelines and interpolation never touch the engine.
- **The combined tracking level**, as a `max` over what the frame's components declared, pushed down
  through `set_mouse`.
- **The caret's screen position**, translated from layer coordinates, which the runtime can do because it
  brought the layer's rectangle.
- **Everything a component sees.**
- **Damage: nothing.**

### What the engine refuses — each checkable by reading the API alone

There is no type to look for. That is the test.

1. **No layout.** No constraint, no flex, no measure, no auto-size (ADR 0002).
2. **No widget.** Nothing is drawn for you and nothing can be registered to be drawn.
3. **No reactivity.** No signal, no observer, no subscription.
4. **No iteration of application data.** Coordinates and `&str` go in; nothing of yours is held.
5. **No trait — zero of them.** The engine has nothing to call upward, so the dependency arrow is enforced
   by there being no arrow. `dyn Painter` stays a negative result.
6. **No alpha and no per-layer opacity.**
7. **No blocking primitive on the app thread and no completion anywhere.** `Slot::take` is non-blocking;
   the word `recv` does not appear.
8. **No executor and no thread pool.**
9. **No clock and no scheduler.** `request_wake_at` is a deadline sink. *This one was nearly lost* —
   `elapsed()` and `wake_in()` on `View` were proposed, worked, and were refused: a clock and a scheduler
   on the drawing type is to time what ADR 0002 forbids for layout. On inspection the engine needs
   *nothing*, because the runtime owns the loop.
10. **No display query.** The refresh rate arrives as configuration or not at all.
11. **No cells, no grapheme handles, no style bits** (ADR 0023).
12. **No `unsafe`.**

Also absent, and priced rather than overlooked: `split_h`/`split_v` (§4), `fill_with` (§4), a write-time
equality filter (§4), a flattened layer cache (§5), a serializer `Options` type (§8), and a worker pool
(§11).

### The threading contract

| crosses a thread boundary | stays on the app thread |
|---|---|
| `Engine` (`Send`) — built before any thread exists | `Screen` (`!Send`) |
| `WakeHandle` (`Send + Sync + Clone`) | `View<'a>` (`!Send`, by a private `PhantomData`) |
| `Surface` (`Send`) — **deliberately** | `Permit<'a>` (`!Send`) |
| `Slot<T>` (`Send + Sync`) | `LayerStack` (reachable only through `Screen`) |

Asserted by the compiler, not by this table: the negative claims are paired `compile_fail` doctests
(§14), and they produce `E0277` for the `Send` cases and `E0499` for the two borrow cases.

### Precedence rule 4, audited verb by verb

- **`&self` wherever a call only reads**: `Screen::size`, `request_wake_at`, `permit_slow`,
  `capabilities`; `View::size`, `visible_rows`, `visible_cols`; `LayerStack::topmost_at`, `len`,
  `is_empty`; `Slot::put`, `take`; `WakeHandle::post`, `quit`; every method on `Rect`, `Style`, `Color`,
  `Mix`.
- **`&mut self` that is load-bearing rather than a concession**: `Screen::layers`, `present`, `wait`.
  `layers` hands out a `View`; if `present` took `&self`, a half-drawn frame could be submitted *while*
  that view was still alive. The exclusivity **is** the guarantee.
- **`&mut self` assessed and kept**: the write verbs, with the cost of removing it recorded in §4.
- **Moves**: `Engine::attach`, `Slot::put`, `add_content_with`. Ownership transfer is the mechanism by
  which the app thread never waits, not a cost.
- **Two lifetimes in the whole surface**, `View<'a>` and `Restyle<'a>`, and elision keeps both out of
  component signatures. The second is ticket 21's: `Restyle::link` names a URI rather than a handle,
  and a descriptor is built inline at every call site — the lifetime reaches a signature only where a
  helper takes `&Restyle<'_>`.

### Naming

- **`Screen`** for the app-thread object. `Terminal` was rejected because the glossary already uses
  *backend* for the seam the terminal library lives behind, and a central public type called `Terminal`
  two lines away invites exactly the confusion the crossterm-invisibility rule exists to prevent. `Ui`
  was rejected because an engine that refuses to know what a widget is should not name its central type
  after the thing it refuses to know. The one weak spot is written down rather than hidden:
  `screen.wait()` reads oddly.
- **`WakeHandle`** rather than `Waker`, which is half the `Future` contract in a crate that bans futures,
  and rather than `Unparker`, which was orphaned once `Parker` stopped being public.
- **`Wake::Input` / `Posted` / `Deadline` / `Quit`.** `Input` names the source thread, `Posted` names the
  verb that caused it, and `Event` is freed for *what happened*.
- **`Presented { submitted, .. }`** — `submitted`, never `painted`. The distinction is refusal 7.

### What did not survive

A capability token in a signature; `elapsed()`/`wake_in()` on `View`; `Screen` split into a
terminal-facing and a compositing-facing type (cleaner on a diagram, and it pays with two handles the
runtime must hold in the right order when both are `!Send`, both live on the app thread, and `present`
has to know the clock that lives with `wait` regardless); a component holding `&mut View` directly;
`std::io::Error` for `attach` ("the terminal never answered the capability query" is not an I/O failure,
and saying so in a signature is a lie a reader has to unlearn); and a builder for `Config` — every knob
visible in one place is worth more to "the engine is comprehensible on its own" than a chain of setters.

### `prototypes/seam.rs` is four tickets behind

The compiling skeleton ticket 12 produced is the API chapter's source text and it has not been updated
since. Whoever implements the engine inherits these deltas, and this list is the complete one:

- ADR 0005: `Screen::set_cursor` plus `Cursor`; `graphemes()` and `width_of()`.
- Ticket 10: the six `Event` variants and their payload types, `InputConfig`, `Screen::set_mouse`.
- Ticket 11: `Config::tier: TierRequest` → `Config::overrides: Overrides`; `Capabilities`, `ColorDepth`,
  `GlyphSet`, `Rgb`, `WidthSource`.
- Ticket 16: `restyle` takes `&Restyle` rather than `impl Fn(Style) -> Style`; `Style::with_bg` and
  `Style::with_fg_bg` are **removed**.
- Ticket 13: `Config::clock: Clock`.
- Impl 01: `Config::packets` and `Config::resolver` are **removed**, and `pub struct Resolver;` with
  them — the pool is fixed at two *provably* (§7), so a knob for it may only ever hold one value, and
  `Resolver` is a name this chapter declares that appears nowhere else in the architecture. Both
  absences are **gated** rather than remembered, in `audit::REFUSED_NAMES`, each with a positive twin
  naming a surviving item by path. `Config` gains `output: Output`, `size: (u16, u16)`,
  `overrun_threshold: Option<Duration>` and `overrun_report: Option<Box<dyn Write + Send>>` — where
  the bytes go, and the two knobs §11's in-loop detector is configured through.
- Impl 11: `Screen::input_diagnostics`, which is how §9's dropped-intent counters are read.
- Architecture 21: `Screen::link` and `LinkId` are **removed** and `Link<'a>` joins the listing, so
  the type count does not move and the named functions go from forty-two to forty-one.
- Production 07: `Screen::suspend` and `Screen::resume` (ADR 0052). Two verbs on a type already
  listed, neither returning anything, so the type count does not move.
- Runtime architecture 28: `KeyText::of` (ADR 0053). A constructor on a type already listed.

**This list has been the incomplete one twice**, and both times the reason was the same: a ticket
moved the surface, recorded the move in its own `## Answer`, and left the sentence above claiming
completeness. The counts in this chapter's opening paragraph were never true of the block beneath it
either — twenty-one against a block declaring forty-one — and the reconciliation is
`crates/vitui-engine/src/audit.rs`'s module doc, whose `built` row is joined to the surface itself by
`audit::tests::the_counts_are_the_ones_the_audit_recorded`. **Read the audit's table, not this
paragraph, for what the surface is**; read this chapter for what it may and may not contain.

The handle tables are engine-internal and **no public signature names a handle**, which is what ticket
05's "the API question is closed in advance" was promising. Nothing else moves.

---

## 13. The performance budget

These are the map's standing requirements. They are gates, not aspirations, and none of them may move
without a new map decision (§14).

- **Full-screen composition of 300×80 (~24 000 cells): < 1 ms.**
- **Typical damage-tracked frame: < 100 µs.**
- **Steady-state 60 fps animation: < 5% of one core.**
- **Zero allocations in frame composition.** Allocation is permitted when the scene topology changes,
  not while painting. Arenas are reset, not freed.
- **A genuinely idle application costs zero CPU**: no polling loop, no timer tick, no wakeup. The render
  thread blocks; the input thread blocks; nothing spins.
- **A scene holding 1M elements must paint in the same time as one holding 1k.**
- **Comparative, not only absolute**: the same scenes measured against ratatui, Textual and notcurses.

### The ledger

Everything the app thread pays for, on the worst *realistic* frame measured on this map:

| step | cost | where |
|---|---|---|
| component drawing, five hostile components | ~45 µs | 14 |
| compositing, worst measured full screen at 40 layers | 107.3 µs | 06 |
| packing, full screen with the equality filter's shadow | ~53 µs | 14, 08 |
| damage marking and scanning, worst frame | ~7 µs | 07 |
| the mailbox critical section | 47 ns | 09 |
| the overrun detector | 42.6 ns | 18 |
| **total** | **≈ 205 µs against 1 ms** | |

**Roughly 5× of headroom**, and it is what keeps parallel compositing out of scope: rows are independent
and the stack would split into horizontal bands, but the price is a thread pool that requirement 11
dislikes and `rayon`, which the dependency policy forbids outright. Writing one by hand on `std` for a
107 µs worst case already inside budget is not a trade. The only scene that could ever want it is a
full-screen operator layer — 78.2 µs of that 107 — and it is in budget too, and it is skipped entirely
at `ColorDepth::None`.

Frames that are deliberately outside the *incremental* budget, each a cliff by construction:

| frame | cost | why it is allowed |
|---|---|---|
| a full-screen composite at 40 layers | 107.3 µs | it is a full-screen composition, budgeted at 1 ms |
| a full-screen `Mix` over entirely hyperlinked content | 200.22 µs | the adversarial page, against 1 ms |
| a row-clearing list scroll | 34.5 µs of packing | it buys 24× on the wire and unlocks the scroll region |
| a handle-table sweep over twenty layers | 1.17 ms | runs where allocation is already permitted, never in a frame |
| a legitimate `wake → submit` iteration | 166.76 µs | which is why the overrun threshold is a frame interval, not 100 µs |

Two costs land **outside** the engine and are the runtime's, recorded because they are the ones that
actually break a frame: an unmemoised aggregate over 1M rows is **130.23 µs against a 100 µs budget**
(0.4 ns memoised), and an O(k) row lookup in a 1M-node tree is **9.76 ms**, 656× the O(1) form. The
engine's invariant protects the engine.

### How it is enforced

Not as a list of benchmarks. §14 states the register; the shape of it is:

- **Gates are counts, ratios, equalities and compile outcomes.** Nineteen of the twenty-seven gated
  properties are, and none of them is flaky on a shared runner.
- **A timing gate sits at the *budget*, never at the measurement**, and detects a cliff. Every timing
  cliff this map actually met was 4.88×, 27×, 37× or 284×. A gate tuned to the measurement is a flaky
  test that gets disabled within a month, which is the real failure mode.
- **Comparative results report into a committed file rather than blocking.** Four external projects'
  versions cannot gate our pull requests, and a worsening number arriving as a review-visible diff is
  what makes requirement 11 falsifiable.
- **Every gate number carries a provenance comment** naming the ticket it came from and the machine it
  was measured on.

---

## 14. Verification

### The register

> **A gate is a count, a ratio, an equality or a compile outcome. A timing is a report, and is a gate
> only at cliff granularity, with the headroom written next to the number.**

Almost every defect this map found was catchable without a stopwatch, and the gates say so. The 37.07×
overdraw cliff is a ratio. The 27× scroll-detector regression was visible only because numbers were kept
per scene. The 284× wire regression is a byte count. Zero wakeups is a count. Byte-identical packed cells
is an equality. A 10% timing regression is not detectable on a shared runner, and a gate that claims to
detect one is a flaky test wearing a budget's clothes.

Three refinements arrived after the engine's own ticket closed, from the runtime and components maps
working the same register, and they are folded in here rather than left as folklore:

1. **A gate is an equality only when the number is a property of the mechanism**, not of the data. A
   number that belongs to the data must be a relation (`t_16 > t_256`, `quadratic / linear >= 100`) or it
   becomes a gate that is edited rather than fixed.
2. **A report may not be load-bearing for a gate.** Negative cases kept honest only by a `size_of` line
   in a benchmark that nobody had decided and `cargo test` compiled by accident are not gated at all.
3. **The budget catches a cliff; a growth ratio catches a slope**, and neither substitutes for the other.
   A quadratic duplicate scan costs a dense screen 1.19× and walks straight through a 100 µs gate, while
   the same defect across a 4× size step is 9.2× against 3.96×.

### Two harnesses, and the split is stated rather than discovered

**Determinism is single-threaded before it is clock-injected.** A fake clock makes *time* reproducible; it
does not make *interleaving* reproducible, and every invariant that matters here is about interleaving.

> **In the deterministic mode there is one thread.** `present()` composites, packs, serialises and writes
> into the caller's sink inline on the calling thread; `wait()` returns the next queued wake instead of
> parking.

A test is then a straight-line program: draw, present, assert on the sink, advance the clock, repeat. No
condvar, no join, no timeout, no flake.

`Config::clock: Clock` is **public API, not a test fixture** — a concrete enum, `System` or `Manual`,
never a trait, because the seam has no traits in it and a trait object here would be the first. An
application author testing their own UI needs the same determinism the engine's tests need, which is the
same argument that made headless a *declared* tier rather than the floor.

**`std::time::Instant` has no constructor from a number** — `from_nanos`, `From<Duration>` and `default`
all fail to compile — so the manual clock is **one captured `Instant` plus an offset**. That is why no
public signature changes, and it is worth a sentence here so nobody later proposes a `vitui::Timestamp`
newtype to "make testing possible". `base + Duration` is exact, monotone and identical across threads.

**The cost, stated rather than discovered later: the properties that are about threads cannot be tested
in the mode that removes them.** Zero wakeups, the 47 ns critical section, the packet never superseded,
wake-up latency — all need the real three-thread path and are measured as counts over a real run. The
deterministic mode is for content and sequencing; the threaded mode is for the threading invariants.

Pinning the *capabilities* makes a frame's content deterministic; pinning the *threading* makes the
sequence of frames deterministic; only the two together make a golden frame reproducible.

### The round trip is the primary instrument; goldens are the residue

**All four defects caught so far were caught by the round trip, which stores nothing.** Composite a
frame, serialise it, replay the bytes through §8's terminal model, assert the replayed screen equals the
frame. Each scene is replayed at the tier its own `Scene::overrides` pins — see §8 on why this is no
longer *nine option sets*, and §10 on which axes a scene may pin. There is no file to review, nothing to
bless, and no maintenance. A golden byte string would have pinned the encoding, and the encoding is
exactly the part that is allowed to change.

So a golden's job is only what the round trip cannot reach: **the composited picture itself.** Format —
two planes and a legend, one file per frame, fixed-width, **one file column per terminal column**:

```
scene: three-dialogs  frame: 2  size: 24x6  tier: truecolor/extended

glyph
.......................
.+---------+...........
.|.Save?...|..+------+.
.|.[Y].[N].|..|.·A···|.
.+---------+..+------+.
.......................

style
aaaaaaaaaaaaaaaaaaaaaaa
abbbbbbbbbbaaaaaaaaaaaa
abcccccccbbaaabbbbbbbba
abcddcbcddcbaaabceeeeba
abbbbbbbbbbaaabbbbbbbba
aaaaaaaaaaaaaaaaaaaaaaa

legend
  a  default
  b  fg=#c0c0c0
  c  fg=#ffffff bold
  d  fg=#000000 bg=#c0c0c0
  e  fg=#ffffff underline=curly:#ff0000 link=https://example.invalid/1
  ·  U+1F468 U+200D U+1F469 (width 2, continuation follows)
```

- **Two planes, not one.** A cell carries a cluster and a style word and they change independently; a
  single rendering can only make one of them diff legibly.
- **One file column per terminal column, always.** That is what makes a diff's column position mean
  something, and it is why the glyph plane cannot simply print clusters.
- **ASCII renders as itself; everything else gets a legend character.** The picture survives for the
  content that is mostly ASCII, and the cells that would break alignment are exactly the ones nobody can
  review by eye anyway. The continuation cell of a double-width cluster is a reserved marker, so a width
  bug shows up as a shifted row rather than as nothing.
- **One file per frame**, under `tests/golden/<scene>/<nn>.txt`. Inline expectations read better for
  three lines and destroy a test file at 300×80.
- **Blessing is `VITUI_BLESS=1`**, fifty lines of our own code, not a dependency. It rewrites the file in
  place; the review is the git diff. A golden a human cannot read in a diff will never be updated
  correctly, and one nobody can regenerate will be deleted.
- **The header line is part of the assertion.** A golden taken at the wrong tier is a golden that passes
  for the wrong reason.

### Negative cases: paired `compile_fail` doctests, and nothing else

The must-fail-to-compile shell step does not work, and this was executed rather than argued.

- **The negative direction is a false green.** Deleting all six negative cases and leaving an empty
  `#[cfg(…)]` module — that is, putting the holes back — still fails to compile, so a shell step
  asserting failure still passes. Two independent causes, both from the repository's own configuration
  rather than from the test: an unregistered custom `cfg` promoted by `-D warnings`, and imports left
  unused once the module is empty. Neither has anything to do with `Send`.
- **The positive direction does not compile at all under CI's flags**, so the ordinary test step would be
  red too.
- **`RUSTFLAGS="--cfg …"` replaces `-D warnings`, it does not compose with it**, so such a step silently
  drops the deny-warnings policy for its own compilation.

**Ticket 18's recorded must-fail command must not be copied into CI.** Its conclusions stand; the command
does not.

What ships instead rides inside `cargo test` with no `RUSTFLAGS`, no second job and no dependency:

```rust
/// The app-thread half. Must never travel.
///
/// ```compile_fail,E0277
/// let s = vitui_engine::Engine::new(Default::default()).attach().unwrap().0;
/// std::thread::spawn(move || s.size());
/// ```
///
/// ```
/// let s = vitui_engine::Engine::new(Default::default()).attach().unwrap().0;
/// assert_eq!(s.size(), (80, 24));
/// ```
```

Three properties of that shape decide how every case must be written:

- **The pair is the unit, and neither half is a gate alone.** Deleting the hostile line is caught by the
  first half; **renaming the item it protects is caught only by the second.** A rename makes the negative
  case fail for `E0433` instead of `E0277`, which the mechanism cannot distinguish, and it reports `ok`.
- **The positive twin must name the protected item by path.** A twin that merely exercises the mechanism
  survives the rename — measured on the runtime's corpus, **11 of 57** cases were written that way,
  including the one that would otherwise have been missed.
- **The error-code annotation is documentation, not an assertion.** `compile_fail,E0308` on a snippet
  whose actual error is E0277 **passes** on stable. Write the code for the reader; do not believe it.

**The limit, stated rather than papered over:** doctests compile as an external crate, so this mechanism
reaches only the public surface — which, after ticket 12 removed ten names from it, is exactly the set of
misuses a *user* could commit. An internal misuse is caught by the engine's own compilation. If an
internal negative case is ever genuinely needed it costs a `#[doc(hidden)] pub`, which is a design
admission and should read as one.

A second effect is worth naming: the negative case now sits in the rustdoc of the type it protects, which
is where a runtime author reads it. §11's whole thesis is that an unreachable method is simply not on the
type; the explanation of *why* now sits on the type too, which makes it API documentation rather than
test scaffolding.

### The harness

**`criterion` is not the harness**, and the reason is not taste: it reaches `futures-core` and
`futures-util` through plotters → web-sys → js-sys, which `deny.toml` bans by name as the signal that an
async runtime arrived (§1). `crates/vitui-bench` replaces it — the round-robin minimum-of-N loop written
once, no dependencies, with the methodology in its rustdoc.

One implementation detail that is correctness rather than taste: **the minimum is kept as the whole timed
batch and divided in floating point at report time.** Dividing a `Duration` truncates to whole
nanoseconds, and this project routinely quotes 0.9 ns per damaged cell and 1.4 ns per packed cell —
figures a `Duration / iters` harness reports as zero.

A gate is `Report::assert_under(case, budget)` inside an example, not a threshold in a dashboard. The
number lives in the repository next to a comment naming the ticket it came from.

### CI, and the part of this chapter that was stale

The register above was written by ticket 13 and its CI half did not survive contact. **Three jobs on this
lineage had never been green, and each was found by a different map:**

- **`cargo test --workspace`, the workflow's first test step, had never passed.** The allocation probe is
  a process-global counter and `cargo test` runs a binary's test functions on several threads, so an
  allocating sibling lands in the number. It failed on a **different binary each run** and with a
  **different count each run**. Ticket 13 wrote the comment explaining exactly this above the step and
  left the step in.
- **`cargo clippy --workspace --all-targets` had never passed under the workflow's own
  `RUSTFLAGS: -D warnings`** — unregistered `cfg(protoNN_negative)` names and the unused imports ticket 13
  named in writing and prescribed the fix for. The fix was never applied.
- **`cargo deny` had never passed at all** — path dependencies written without a `version`, which
  `wildcards = "deny"` refuses, and the first crate on the lineage to open a real terminal firing ADR
  0001's `wrappers = ["vitui-engine"]`.

The pattern is three for three and is worth stating as a rule rather than an anecdote:

> **A CI job nobody has watched go green is not a gate, whatever it checks.**

What the engine's workflow must therefore say:

- **One test command, serialised: `cargo test --workspace -- --test-threads=1`, and the plain step is
  deleted rather than moved.** Serialising costs ~9.4 s of in-binary time against a ~10.9 s wall clock on
  the measured lineage; there is nothing to buy back. The one command also runs the doctests, which is
  where the negative gates live.
- **Every job carries `timeout-minutes`**, because `cargo test` has no per-test timeout and a hang is
  worse than a failure.
- **If any custom `cfg` ever ships, register it** — `[lints.rust] unexpected_cfgs = { level = "warn",
  check-cfg = ["cfg(…)"] }` in the crate manifest. The build script the UCD tables require is the other
  place to do it.
- **The idle gate is a two-runner job, not a matrix one** — `/usr/bin/time -l` on macOS, `-v` on GNU,
  neither on Windows — and it needs **30 s per run**, because at 3 s the difference between an observer
  and none is below the tool's resolution.
- **The comparative suite is a separate workflow** on a pinned runner with pinned versions, and it
  reports rather than gates.

### The gate list

**Gate** fails the build. **Test** is an ordinary assertion that happens to be cheap. **Report** prints a
number a human reads.

| # | Property | Kind | Source |
|---|---|---|---|
| 1 | No damage structure under-reports | gate, property | 07 |
| 2 | Runs arrive in serializer order | gate | 07 |
| 3 | Overdraw is 1.00× on every scene | gate, ratio, per scene | 07 |
| 4 | Zero allocations over 1 000 compose cycles | gate, count | 04, 09 |
| 5 | Zero allocations over 1 000 lease–pack–submit–take–finish | gate, count | 09 |
| 6 | `pack` allocates zero on warm tables, at every density | gate, count | 16 |
| 7 | A settled operator allocates zero; a fading one allocates per distinct extended style and never per cell | gate, shape | 16 |
| 8 | The pool does not starve at 10 000 cycles | gate, count | 09 |
| 9 | A packet is never superseded | gate, count | 09 |
| 10 | A packed cell is byte-identical to the surface cell | gate, equality | 16 |
| 11 | After a sweep every live cell resolves to the same channels; `renumbered` is false when nothing below a live entry was freed | gate, equality | 16 |
| 12 | Round trip: replayed screen == composited frame | gate, equality | 08, 16 |
| 13 | A presses-only terminal never yields `Release` or `Repeat` | gate | 10, ADR 0007 |
| 14 | Motion floods collapse to one per wake; press floods do not collapse | gate | 10, ADR 0008 |
| 15 | Panic restore pops keyboard flags and disables mouse, focus reporting and bracketed paste | gate | 10, 01 |
| 16 | Parser survives five adversarial splits | test | 10 |
| 17 | Idle: zero wakeups, `0.00 user 0.00 sys` | gate, count, **30 s minimum** | 09, 18 |
| 18 | No observer thread linked into a release binary | gate | 18 |
| 19 | The negative cases do not compile | gate, paired doctests | 18, 13 |
| 20 | 1M elements cost the same as 1k | gate, ratio in a stated band | 05, 14 |
| 21 | Wire bytes per scene | gate, count | 08, 16 |
| 22 | Overrun detector under 50 ns | gate, timing | 18 |
| 23 | Full-screen 300×80 under 1 ms | gate, timing at the budget | budget |
| 24 | Typical damage-tracked frame under 100 µs | gate, timing at the budget | budget |
| 25 | 60 fps steady state under 5% of a core | report | budget |
| 26 | Wake-up latency p50 / p99 / max | report | 09 |
| 27 | Comparative suite | report, committed file | 11, 13 |

Two entries a future session will be tempted to demote. **#9 is the backpressure decision**, not a health
check: if it ever trips, the pacing gate has moved off the app thread and frames are being composed to be
thrown away. **#10's violation is invisible in every ordinary test** — the screen is correct and the wire
is 284× fatter.

One entry has an **attribution window** and it is a general rule: the allocation probe counts `alloc`
calls, not `alloc` calls *by the app thread*, so a window opened around a frame while a worker is running
attributes the worker's growth to the frame. An allocation gate with a background job in it runs on a
deterministic spawner, or joins before it measures.

**Two entries have a subject that can be skipped, and a gate whose subject can be skipped must prove
the subject ran** (22). #7's operator is skipped outright at `ColorDepth::None` and its `Mix` reaches no
table at all unless the cells it touches are *extended* — so the gate pins both the depth and, once
§10's intern-key collapse lands, `hyperlinks`, and carries a positive half asserting the mechanism
moved something. #12 is the same shape one layer along: a round trip over a cell whose extended channel
the serializer will not emit compares one screen against a different spelling of it, so the hyperlinked
arm and the collapsed arm are two gates and each declares its own axis. Both of these were green over a
skipped subject at least once, which is why the rule is here rather than in a comment.

### The scene list is normative

Not an appendix of "things we test". **Three scenes that score identically on every candidate validate
the wrong design while reporting success**, and that is exactly what happened to the damage model. The
list lives in `crates/vitui-engine/examples/budget.rs` so a regression can be pointed at it.

| scene | what it decided |
|---|---|
| a blinking caret | 171 ns composite at 40 layers; the 29-byte frame that makes framing visible |
| a scrolling list, label only | span damage against the filter, 1.84× |
| a scrolling list, rows cleared | the scroll region: 1 726 → 60 bytes; 34.5 µs of packing |
| twenty stacked popups with shadows | 107.3 µs full-screen composite at 40 layers |
| three dialogs standing apart | per-row spans 2.53×; the gap merge, 1 491 → 1 203 bytes |
| **a sub-cell chart, 400 points** | overdraw **37.07×** (07), the serializer walk **4.88×** (09), the 166.76 µs iteration that set the watchdog threshold (18) |
| a progress bar advancing 1% | 100 cells rewritten, one changed: 30.8× on the filter |
| a virtualised tree, 1k / 100k / 1M rows | the data-volume invariant, flat at 54 µs and at 14.4–15.0 µs with component code |
| one table as a list and a bar chart | precedence rule 2; 22.35 / 22.97 µs at 1k and 1M |
| a full-screen change | the filter's floor, 1.46×; `natural` worth 5.4% and `shortest` nothing |
| every cell a distinct style | 805 657 bytes, 286 ms on a 4 MB/s link — the sync-output time limit |
| **a hyperlinked page under an animating operator** | table growth, the sweep, `repaint`, and the memo, all at once |

Three rules govern it:

- **Numbers are kept per scene and never summed.** The 27× scroll-detector regression was visible only
  that way.
- **A scene is removed only by a ticket that names the property it can no longer distinguish.** Twenty
  popups discriminates nothing today and stays anyway, because "discriminates nothing today" is a
  statement about the current candidates.
- **The sparse sub-cell chart is not optional.** It has decided three tickets.

The last row is the only shape that grows a handle table without bound, and it decides table lifetime the
way the chart decided damage. **It was the one row with no measurement behind it until implementation
ticket 08 paid it** — 96 entries created over 120 settled frames against 11 520 over 120 fading ones, the
sweep at 82.5 µs for one screen and 897 µs for twenty layers — and those numbers were taken *before*
§10's intern-key collapse existed. They do not survive it unless the scene declares `hyperlinks` the way
it already declares truecolor (22): a page whose links are dropped from the intern key mints nothing, and
the row would report 8 entries where it exists to report 96. **The scene needs two pins, and neither is
optional.**

### Regression policy

**What is a failure.** A gate is a number, a count or an equality committed to the repository, in
`assert_under` or in an `assert_eq!`, with a comment naming the ticket it came from and the machine it was
measured on. Nothing is a gate because a dashboard went red.

**What is noise.** Counts, ratios and equalities have none — a byte count is a byte count on any machine,
and that is the whole argument for the register's shape. Timings have plenty, and the harness already does
what can be done: minimum of forty, round-robin between variants, so a background job is spread across the
arms rather than landing inside one. What remains is the runner. Therefore fine-grained timing comparisons
run on a developer's machine, through `cargo run --release --example *_numbers`, and their output goes into
a ticket.

**Who may move a number.** Anyone, in a commit that does three things: states the new number, states the
measurement it came from, and replaces the provenance comment next to it. A gate number without a
provenance line is already broken, whoever moves it. **The one class of number that may not move without a
new map decision is a budget figure** — the 1 ms, the 100 µs, the zero allocations, the zero wakeups.

### The comparative suite

Requirement 11 is a claim about other people's software, so an absolute number cannot check it. The rules
are what was actually missing:

- **A scene is defined by what the user sees, never by what a framework does.** The specification is a
  terminal size, a described initial screen and a described sequence of changes. Any definition in terms
  of "the same widget" or "the same redraw" smuggles one framework's model into the comparison, and
  immediate-mode against retained-mode is precisely where that would bite.
- **Two measured quantities: bytes on the wire, and end-to-end keystroke-to-wire latency**, plus process
  CPU time over a fixed scene, which is what "never profligate" actually names. Bytes are exact and
  machine-independent. Latency is end-to-end because our wake-up interval is OS scheduler latency and the
  synchronous renderers have no such interval at all; comparing it to nothing is not a comparison.
- **Bytes are captured with stdout on a pipe**, which is the declared tier, and is why the comparison is
  possible without four terminal emulators. Every arm is forced to the same declared capability set
  through the environment, and **what each arm did with that declaration is reported**, because a
  framework that ignores `NO_COLOR` is not thereby faster.
- **Latency needs one terminal, named and pinned**, and the number is comparable only within it.
- **Python counts.** Textual's interpreter overhead is what its users pay. The interpreter version is
  pinned and stated.
- **notcurses is the honest ceiling, and it does not composite layers.** The layered scenes have no
  notcurses arm and that cell reads **"cannot express"**, never blank. A missing row reads as a win, and
  this is the single easiest way for the suite to become dishonest.
- **It reports; it does not block**, into a committed file.

### Fuzzing

`cargo-fuzz` needs nightly and `libfuzzer-sys`. The `fuzz/` directory is its own workspace, so its
dependencies never enter `cargo deny`'s graph — **that is a loophole, not a permission**, and `fuzz/` gets
its own `cargo deny` invocation.

A fuzzer cannot be a pull-request gate: at sixty seconds it finds nothing, at an hour it is a flaky test.
So the polarity inverts:

> **Every crash is minimised and committed as an ordinary unit test, and the committed corpus replayed as
> an ordinary test is the gate. The fuzzer itself is a scheduled soak.**

Two targets. **Draw sequences against a naive reference compositor** — one cell at a time, no damage, no
runs, obviously correct and far too slow to ship; the fast one must agree cell for cell, and this is also
where gate #1 is generated rather than hand-written. And **byte streams into the input parser**, where the
oracle is weaker — no panic, every byte consumed, no unbounded growth — but which is the one surface that
parses input the engine did not produce.

---

## 15. What this spec does not decide

Each of these is fog with a named shape, not an oversight.

- **Terminal lifecycle — struck 2026-08-29, and what is left of it is one sentence.** The whole entry
  is decided and lives in §7's *the terminal leaves and comes back*: three cases, three different
  answers, register entry 29. The leased surface under a resize, the SIGWINCH race and panic
  restoration graduated first; production ticket 07 took the other three. `Capabilities` stays
  immutable and a resume re-declares rather than re-detects, a terminal that was replaced costs a
  fresh `attach`, and an external `SIGTSTP` is stated as uncovered because the engine can install no
  signal handler (safe Rust has no `sigaction` and the dependency policy has no crate that does).
  **The one thing still open is smaller than the entry was**: an application that suspends itself must
  stop itself, and there is no way to raise a signal from safe Rust either — so the three lines the
  spec describes end with a `libc` call, a `Command::new("kill")` or a crate, all of which are the
  *application's* dependency policy and none of which this spec has an opinion about.
- **Whether `DECSLRM` (mode 69, left/right margins) is worth querying.** Without it `SU` moves every
  column of its region, so a pane that is not the full width can only use the scroll region when the
  columns it does not own are blank in both frames. With it, any pane could scroll — and a one-row list
  scroll is 1 726 bytes against 60. It is not in tier-1's confirmed set, and an unsupported margin would
  turn a verified optimisation into silent corruption. **Nothing in §8 depends on the answer.**
- **Feature flags, MSRV and the workspace's release policy.** The crate is designed as a public one, with
  a deliberate prelude and documented surface, and **no stability promise before 0.x**.
- **Windows conhost specifics** beyond the general degradation path. Windows Terminal gets the full
  feature set through crossterm; legacy conhost gets the degraded path, which is the same machinery
  `--no-color` and `--ascii` need anyway.
- **The per-terminal quirk table is not fully enumerated, and this entry is now narrower than it was
  twice over.**
  The layer must exist and populating it is field work, which is what this said and why. `conform/`
  did that field work four times: a fourth entry, **tmux — overline is accepted, stored and never
  forwarded** (engine production ticket 05), the first here recognised by a *query* rather than by an
  environment variable; a fifth, **kitty — conceal and overline have no attribute to be stored in**,
  settled by the shipped binary rather than by a capture; a seventh, **Alacritty — blink and overline
  have no bit** (production ticket 12); and an eighth, **iTerm2 — overline has no bit in the cell**
  (production ticket 13). Eight entries, four of them this repository's.

  **Two counts say how far that goes, and they count different things.** *Terminals run*: **one of
  §10's seven tier-1 terminals has never been run, and it is Windows Terminal** — kitty, Ghostty,
  WezTerm, Alacritty, tmux and iTerm2 have all been driven through `conform/`'s attribute scene, and
  **production ticket 16** is what owns the seventh. *Attribute facts*: the scene asks **eleven per
  terminal**, so a terminal run is not eleven facts answered — Windows Terminal's eleven remain
  inference from libvaxis's three entries, and five rows across three of the six arms that did run
  come back `cannot ask`, which is a limit of that arm's capture surface rather than of the terminal.
  **The ConPTY entry is the one no run here has ever touched**: it is libvaxis's, inherited whole,
  and ticket 16 is where it is confirmed, corrected or removed.
- **The sweep's high-water policy is not tuned.** The mechanism is measured; `2×` the live count at the
  last sweep with a floor is a starting value and nothing discriminates between candidates.
- **The URI table is not swept.** Link ids are few and nothing measured suggests it matters. Recorded
  so it is a decision rather than an omission. Its *reason* used to be *an application holds handles to
  them*, and ticket 21 ended that: nothing outside the crate holds one, so the table could join the two
  the sweep already touches whenever anyone wants it to. Not taken there, and the decision stands on
  the measurement rather than on the reason it lost.

Seven measurements are **owed**, each with its payer named:

1. **The quantisation memo's cost**, against the equality filter's 0.9 ns per damaged cell. Whoever
   implements the serializer.
2. **What the `CHA`-after-non-ASCII rule costs in bytes**, against the 848 bytes `shortest` won on the
   chart. The instrument exists: §8's terminal model, taught to simulate a terminal that disagrees about
   width. Whoever implements the serializer.
3. **The golden-frame format has never been diffed in anger.** Two planes plus a legend is reasoned from
   what a reviewer sees, not from a review that happened.
4. **The reference compositor's slowness is assumed.** If the naive one is within 2× the differential
   fuzz is worth less than it looks.
5. **The comparative suite has never been run.** Expect the "same scene" definition to break first.
6. **What `add_content_with` costs when it renumbers a donated surface's handles** (19). O(cells) with a
   hash lookup on the cells that carry a cluster, once, at a topology change; the eviction sweep's
   58.88 µs for one screen is the same shape of work and is the only reason to expect it is cheap.
   Whoever implements the layer stack — implementation ticket 10.
7. **What one intern probe at the verb costs** (21). `Restyle::link` names a URI, so every verb call
   that names a hyperlink hashes it once — above `restyle`'s memo, so once per call and not per cell or
   per style word. Measured against the full-screen linked `restyle` that already costs 75.49 µs, and
   payable by whoever moves `Restyle`.

Two claims are **about other people's software and are not vouched for** (10): that no mouse protocol
reports the pointer leaving the terminal, and that an image cannot arrive through bracketed paste. The
first belongs to whoever builds hover in the runtime; the second is immaterial, because `Paste` owns its
bytes regardless.

And one debt this map declines to pay because it belongs to whoever writes the engine: **the gate list has
twenty-seven entries and nothing to run them against.** Every one is written from a prototype's
measurement, and a prototype's measurement is a claim about a design, not about the code that will exist.

Out of scope entirely, ruled while charting and unchanged:

- **The component library** — the five hostile components appear here only as design pressure. What they
  *found* is in scope and recorded; what they *are* belongs to the components map.
- **Image protocols** — sixel, kitty graphics, iTerm2. The `Surface` seam must not *preclude* them, which
  is checked on paper, and §8's "the backend seam is the packet" is the same guarantee from the other
  side. None is designed here.
- **IME and complex text input** — out for the engine; the runtime will need it for forms.
- **RTL and bidirectional text.** Grapheme clustering and CJK width are *in* scope — without them the
  cursor desynchronises — but bidi reordering is not.
- **Accessibility and screen readers** — explicitly acknowledged as not solved rather than silently
  omitted. §4's caret decision is the one thing on this map that helps.
- **Inline (non-alt-screen) rendering** — shadows and transitions have nowhere to land without an owned
  screen. If it is ever wanted it is a separate effort, and probably a simpler frontend over the same
  `Surface`.
- **Parallel compositing** — ruled out by measurement, not while charting. See §13's ledger.

---

## Appendix A — ADRs this spec rests on

| ADR | Decision | § |
|---|---|---|
| [0001](../../docs/adr/0001-crossterm-behind-a-seam.md) | crossterm behind a seam | §1 |
| [0002](../../docs/adr/0002-layout-lives-outside-the-engine.md) | Layout lives outside the engine | §0 §4 §5 |
| [0003](../../docs/adr/0003-split-handles-not-a-capability-token.md) | Split handles, not a capability token | §11 |
| [0004](../../docs/adr/0004-the-frame-clock-gates-the-wait.md) | The frame clock gates the wait | §2 §7 |
| [0005](../../docs/adr/0005-the-engine-owns-the-caret-and-exports-its-text-tables.md) | The engine owns the caret and exports its text tables | §4 |
| [0006](../../docs/adr/0006-the-render-thread-mirrors-what-the-terminal-shows.md) | The render thread mirrors what the terminal shows | §8 |
| [0007](../../docs/adr/0007-the-input-model-is-honest-about-the-terminal.md) | The input model is honest about the terminal | §9 |
| [0008](../../docs/adr/0008-intent-is-never-dropped-position-is.md) | Intent is never dropped, position is | §9 |
| [0009](../../docs/adr/0009-the-engine-degrades-presentation-never-content.md) | The engine degrades presentation, never content | §10 |
| [0010](../../docs/adr/0010-detected-axes-are-flat-declared-axes-are-ordered.md) | Detected axes are flat, declared axes are ordered | §9 §10 |
| [0011](../../docs/adr/0011-the-handle-tables-belong-to-the-engine-and-the-packet-carries-copies.md) | The handle tables belong to the engine and the packet carries copies | §3 §7 |
| [0022](../../docs/adr/0022-drawing-verbs-clamp-and-discard.md) | Drawing verbs clamp and discard | §4 |
| [0023](../../docs/adr/0023-the-cell-is-never-visible-in-the-public-api.md) | The cell is never visible in the public API | §3 §4 §12 |
| [0024](../../docs/adr/0024-no-flattened-layer-cache.md) | There is no flattened layer cache | §5 |
| [0025](../../docs/adr/0025-compositing-depends-on-a-terminal-capability.md) | Compositing depends on a terminal capability | §5 §10 |

`0012`–`0021` belong to the runtime map and are not this spec's, but two of them read back onto the
engine and are worth knowing: **0015** (no geometry crosses a frame) is the runtime's form of §7's
"nothing the render thread compares across frames may be derived from a position", and **0021** (the
runtime reads a capability and never rewrites a declaration) is ADR 0009 seen from above.

## Appendix B — where each decision was taken

| § | Ticket |
|---|---|
| §1 | 12 the seam, 13 verification |
| §2 | 09 render thread, 12 the seam |
| §3 | 02 grapheme clustering, 04 cell representation, 16 handle tables |
| §4 | 05 surface API, 14 pressure test, 16 handle tables, 18 unblockable |
| §5 | 06 layer stack, 17 prior art: engines, 16 handle tables |
| §6 | 03 prior art: damage, 07 damage model |
| §7 | 09 render thread, 12 the seam, 16 handle tables |
| §8 | 08 frame serializer, 01 capability detection, 14 pressure test |
| §9 | 10 input pipeline, 01 capability detection |
| §10 | 11 degradation model, 01 capability detection, 02 grapheme clustering |
| §11 | 18 unblockable, 12 the seam |
| §12 | 12 the seam, amended by 05, 10, 11, 13, 14, 16 |
| §13 | the map's standing requirements, 14 pressure test, 13 verification |
| §14 | 13 verification, with the CI half corrected by runtime R15 and components C11 |
