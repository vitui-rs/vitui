---
status: accepted
date: 2026-08-28
---

# A swap is measured against an oracle, and the memo the rule is about had to be built

Components ticket 41 inverts register row 8 — *no cell keeps the previous palette a frame after a
swap* — which is the last of spec §21's two pinned reds and the second the gallery is measured on.
Three decisions are recorded here: what the detector is, why neither the delta nor its complement
could have been it, and where the memo ADR 0030's rule is about lives now that the census has found
there is not one.

## The detector is a second arm played at the destination theme

`gallery::swap_on` plays two galleries. **The swapped arm** is what a terminal does: one surface,
three frames at the old theme, the key, one frame more. **The reference arm** is the same gallery,
on the same page, driven the same four frames, with the destination theme in place from the first —
the screen the swap was *supposed* to produce. `Swap::stale` is the cells the two disagree about,
and the gate is `stale == 0`.

It is the engine's own arrangement one crate up. `reference.rs` is the obviously-correct,
far-too-slow compositor, and gate #1 is **generated from it** rather than hand-written, *because a
hand-written expectation about damage is written by the person who wrote the damage*. The same
argument holds here one layer out: an expectation about what a swap should have reached, written by
hand, is written by whoever decided what it reaches.

**The reference arm is always `Keying::Derived`**, whatever the arm under test is. The oracle is
*the screen the destination theme produces*; a memo is a thing the screen under test does. Giving
the reference the same memo would compare a stale picture with a picture that never had the chance
to go stale, and both arms would agree — the vacuity this whole ticket is about, arriving one level
in.

## Neither the delta nor its complement could have been the gate

§21's first refinement is *a threshold on the wrong side of the question is not a weak gate, it is a
green one*, and this row is its own example. The spelling that shipped asserted `changed > 0`, and
**one cell of 4 800 satisfies it while 3 583 carry the old palette**.

The complement is not the repair. Measured on this screen:

| axis | `kept` | and nothing is wrong |
|---|---|---|
| a scheme change | 0 of 24 000 | every paint on the screen moved |
| a repertoire change | **17 884 of 24 000** | a rung moves the cells drawn from the theme's glyph table and no others; every letter of every label is `kept` and right |
| a colour-depth change | **23 990 of 24 000** | `Theme::resolve` returns the same `Paint` at every depth, so the colour axis moves nothing on any canvas (ADR 0018) |

*Both numbers are the delta read from its two ends.* Neither can tell **the swap reached nothing**
from **the swap had nothing to reach**, and on two of the three axes those two readings are the same
number. `changed` and `kept` are printed by `examples/gallery_numbers.rs` and gated by nothing.

## ADR 0023 did not have to be reopened, for row 7's reason one ticket earlier

The row carried a barrier citing *the cell is never visible in the public API*, on the ground that a
surface count needs a readback. It does not: the count is read off `crate::runner::Pen`, which is
where the partition rule's other half has always been read, and where two arms of a swap can be
compared cell for cell without any crate above the engine reading an engine cell. The barrier is
struck rather than lifted.

## No shipped memo in this crate holds a paint or a cluster, so the memo was built

`crate::memos` is ADR 0030's rule as a population — *a memo carries the theme in its key iff its
value is made of paints or glyphs*, one row per memo, joined to `INVENTORY` because ticket 41 asks
the enumeration to go through the freeze rather than through a grep. Five rows, and the shape of the
list is the finding:

- `chart::raster::PlotState::raster` — a sub-cell bit grid and an owner byte per cell.
- `chart::raster::PlotState::range` — an axis domain: two floats.
- `edit::Text::index` — a wrap index: byte offsets and row starts.
- `preview::Screen::highlighter` — a fold count.
- `gallery::Panels::slots` — **the verbs a tile drew**, and it is this ticket's.

Every cluster and every paint on every panel is derived from the theme in front of the frame that
draws it. So the rule was true here **vacuously**, which is `Verdict::of`'s vacuity failure in the
shape this map keeps meeting — *an equality between two things that do not exist holds* — and a gate
resting on the census would have been green for ever.

**So the memo is built as an axis**, the way `Remainder` is: `gallery::Keying` with a shipped arm
(`Derived`, no memo at all), the rule's arm (`Themed`, keyed with `Theme::memo_key`) and the defect
(`DataAndTier`). It is what an application author reaches for first — a panel's drawing is expensive
and its data did not move, so keep the verbs and replay them — and it is the only thing on this
screen that can make row 8's gate fail.

Measured, over the same twenty-eight call sites:

| arm | scheme | repertoire | colour depth |
|---|---|---|---|
| `Themed`, 300x80 | 0 stale | 0 | 0 |
| `DataAndTier`, 300x80 | **18 156 stale** | **6 509** | 0 |
| `DataAndTier`, 100x30 page 3 | **861 of 3 000, with `changed` reading 2 159** | 568 | 0 |

`Themed` is held to hitting — 56 of 112 tile draws at 300x80, which is frames two and three, with
the swap frame a miss — because *a memo that never hits cannot be stale*, and `stale == 0` over one
would be a statement about a memo nobody used.

**The tier column is the argument for the key.** `(data, tier)` is right about the one axis that
moves nothing anyway and blind to the two that move everything. An enumeration of axes is a key the
next reader forgets one of; the theme's own `Revision` is the one key that cannot be short.

## The 598-character form was already built, and it was owed to this row

`crate::glyphs::gutter` and the two tests in `tests/glyph_matrix.rs` are components ticket 05's:
a memo over a 200-row tree gutter, keyed `(data, tier)`, hitting after a repertoire swap and handing
back **598 wrong characters**. It is the same defect over a value that really is made of glyphs, at
a corpus small enough to count by hand, and it is cited on row 8 rather than rebuilt.

## Consequences

- Register 223 → **226 rows, 213 evaluated**; red 2 → **1**, and the one left is another crate's.
- `changed` and `kept` survive as reports. Deleting them would lose the record of what the row's
  failing set was measured with.
- `Keying` is a public axis on a public screen. `Derived` is the default and every other gate in the
  crate runs through it.
- **The census's completeness check found a scan that could not see two thirds of the crate.**
  `crate::order` claimed *no memo in this crate's library half takes a bare `Revision`* and checked
  it with one non-recursive `read_dir` over `src/`, while `src/chart/raster.rs` built two of them.
  It is `crate::inventory`'s own recorded defect arriving a second time in a different file. Both
  halves are repaired: the recursion, and the claim restated to what is true — `Memo::get` takes one
  `Revision`, so a compound key is a **fold** into eight bytes, which is the runtime's own documented
  door, and `Keyed` is for the key that cannot be folded.
- **The allocation window for `t` is round the press and not round the frame**, and that is an
  instrument's property rather than an application's: `Driver::headless` moves a `Vec<u8>` into the
  engine and never drains it, so a themed frame — a full-screen repaint — feeds a buffer that only
  grows. Measured over 6 000 presses at 100x30 it reallocates at **92, 275, 641, 1 372, 2 836 and
  5 762**, exact doubling, inside `vitui_engine::engine::write_frame`. Runtime architecture issue 34
  as a number. What is gated is the press: **0 allocations over 168 of them**, which wraps every
  scheme, every repertoire and every depth.
