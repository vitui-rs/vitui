---
status: accepted
date: 2026-08-29
---

# A component may own an anchor and may not own a clock

Components ticket 46 builds `spinner`, the twenty-ninth and last row of spec §17's freeze, and the
playhead beside it — the seventh part of the video player's chrome and the one row of
`media::player::PARTS` whose `Needs` was `Clock`. The mechanism was settled by components ticket 42's
prototype, which measured a component that owns a clock against one that owns an anchor; this
records what shipped, and the three things that moved when the prototype's readings were made into
derivations.

## The rule, stated once

> **Stored state may be an anchor, never a phase.**

Spec §8 refused `Collapsing` and `Expanding` because *a transition state has to be stored, which
means the machine can be found halfway between two states with no clock running*. Read as *a
component may not store anything a clock moves* that forbids a spinner outright — and it is not that
rule. `disclose::Collapse` is the proof: it stores a `Tween`, which is two `Instant`s and two values,
**across frames**.

What separates them is what the stored thing *is*:

- an **anchor** is a value the current state is recoverable from at any `now`. There is no state
  *between* two states, because the state **is** a function of `now` — a machine asleep for an hour
  computes the same answer as one drawing at sixty hertz;
- a **phase** is only meaningful relative to a frame that already ran, and a machine holding one can
  be found halfway with no clock running, which is §8's sentence exactly.

So this is not a permission granted to `spinner`. It is the rule `collapsible` and `scroll_area` were
already obeying, said out loud — and a spinner asks for **strictly less** than the collapsible
already ships: `indicate::ANCHOR_BYTES` is **32** against `disclose::TWEEN_BYTES`'s **40**, with no
target, no `from` and nothing to land on.

### The clock is the frame's, and the refusal is a measurement

An anchor is the component's; the clock is not. `Ctx::now` is the frame's own sampled instant, so
everything drawn from it agrees. The other arm — one `Instant::now()` inside a draw — is refused
because **`Driver::pin_clock` is the entire test regime of this workspace and a self-sampling
component is invisible to it**: not only its own gates, but *every screen it appears on* loses the
ability to advance time. Eight advances of a pinned clock move an anchored spinner through 8 distinct
ladder frames, a self-sampling one through **1**, and an accumulating one through 2, with a tweening
`collapsible` reading `[0, 2, 5, 6, 6, 6, 6, 6]` in all three as the control.

It is kept true by a **scan**, because an absence has no expression:
`indicate::tests::no_component_body_in_this_crate_samples_its_own_clock`. The population is a
**function and not a file**, and that is the load-bearing half — fifty-six library-half
`Instant::now()` calls live in this crate's instruments, six of them inside `collect.rs`, which is a
component module. What separates a report from a draw is the **`Ctx`**: a function that takes one has
the frame's clock in its hand. A scan by file would have carried a growing exception list, which is
the shape this crate keeps finding defects behind.

## The two rules a spinner owes, and neither is visible on a surface

**Ask only if the draw put a cell on the screen.** *Undrawn* is already answered by the caller
culling — the same mechanism a closed `collapsible` and an off-viewport row use, since the body is
not called. **Clipped** is the case that is not the same: the component runs, writes nothing, and can
still ask. The two screens are **cell for cell identical** and one of them keeps the terminal awake,
so `indicate::defective::spinner_asking_always_into` is a runnable arm rather than a paragraph.

**The ask names the widget.** `Ctx::deadline` and `Ctx::deadline_for` attribute the *same way* — both
record the caller's line, which `#[track_caller]` makes the application's — so what the id buys is
the **census**. Three spinners on one screen name three widgets through `deadline_for` and **none**
through `deadline`, while both spellings attribute one line correctly; a screen that is awake can
then be asked which widget is keeping it awake.

## What moved when the prototype's readings became derivations

**`constructions` is 3, not 2, and the correction is the rung boundary rather than the count.** Ticket
42 read `4 / 10 / 10` off the frame counts with the braille spinner at `Unicode | Extended`. The
engine's own `GlyphSet` says `Unicode` is *Unicode a normal text font covers* and `Extended` is
*braille, block elements, emoji, powerline* — so a terminal that kept the middle promise would have
rendered tofu, which is the one failure a ladder exists to prevent. The middle rung is the **quadrant
blocks**: an orbiting dot, a different construction from the ASCII rotating line rather than a
re-spelling of it. The counts are `4 / 4 / 10` and the **ladders** are three distinct tables.

That is the second half of the correction: **a count is not a ladder.** Read off the counts the
answer is 2, which is exactly what ticket 42 reported. `indicate::constructions` counts distinct
tables, and `indicate::tests::a_spinner_is_three_constructions_and_the_ladder_is_its_own` asserts the
freeze against it — the derivation every other multi-construction row already had and this one did
not, so nothing in the tree would have noticed the ladder changing shape.

`spinner` is therefore `plot`'s shape and not `meter`'s: braille is what it spends 256 states a cell
on, so `Extended != Unicode` where a horizontal bar's ladder has them equal.

**The cadence figure is sensitive to how 60 Hz is spelled, and only on the arm that is wrong.**
`Cadence::NextCellChange` answers **60** wakes over a two-hour run on a sixty-column bar whatever the
nominal interval is, because the instant the drawn column moves is a function of
`(anchor, duration, width)`. `Cadence::EveryFrame` answers **431 991** at 16 667 µs and 432 017 at
16 666 — its count is a property of the *nominal interval* rather than of the run, which is the
finding from its other side. The ratio is **7 199×**, and it is a **consequence** of the anchor
rather than an optimisation available to all three arms.

**The reading is a delta and never a total.** `WakeLedger`'s counters are cumulative and never reset,
which is the trap ticket 42 recorded against its own first cut: by the second frame every call site
has registered, so a total is saturated and a playhead that never lands measures exactly what a
landed one does.

## `glyphs: &[]` stays empty, and that is §16 working

A `Glyph` is one lookup with no spelling blank. A spinner needs an **ordered set of `n` spellings
that differ from each other**, which is not one lookup and not `n` of them either — cycling
`ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight` at a reader is telling them nothing four times. So
the ladder is the component's own table, exactly as `chart::raster::RUNGS` is `chart`'s and
`media::sub_rows` is the picture's.

**There is no `Distinction` for motion and none is added.** The theme has ten and none of them is
*motion is visible*; `Fade` is about whether a run of intermediate colours survives quantisation. The
motion switch is the caller's `Duration`, and a zero one is static and asks for nothing — §8's own
arrangement one component over, unchanged.

## Consequences

- `INVENTORY` is **29 built of 29** — every row of the freeze — and `MOVED` gains its thirteenth row,
  which is the one that was genuinely at risk rather than merely mis-tiered.
- The four populations that read the `built` column moved to twenty-nine **with no edit to any of
  them**: O1's pages, O2's panels, O3's goldens and O7's applications. That is what those queries
  were written for.
- `media::player::SHIPPED` is **7**, `Needs::Clock` has no row, and the variant stays so the table
  can still record what a part was waiting for.
- Register rows 232 and 233; `EVALUATED` 218 → 220. `crates/vitui-apps/examples/pipeline.rs` is the
  application, and `c` is the key: both cadences draw the identical screen and only the wake counter
  separates them.
- **Five `*_numbers.rs` reports had been panicking since components 41** and the sweep found them —
  components ticket 20's finding for the fourth time: `cargo test` does not run an example, so an
  `assert!` there is compiled by `cargo clippy --all-targets` and evaluated by nobody.
