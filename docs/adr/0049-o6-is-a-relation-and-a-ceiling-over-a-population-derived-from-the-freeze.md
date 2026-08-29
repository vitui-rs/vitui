---
status: accepted
date: 2026-08-29
---

# O6 is a relation *and* a ceiling, over a population derived from the freeze

Components ticket 44 adds a **sixth obligation** beside spec §17's five: *every component that takes
a data volume draws a frame inside a 16.7 ms budget at 1 000 000 inputs.* It is stated after the
architecture map closed, and it is an implementation ticket rather than an architecture issue
because the instrument is buildable without reopening anything.

Three decisions are recorded here: what is gated, over which rows, and where the arms that are
watched failing live.

## The hole, stated once

> This crate can prove a frame's **output** is flat in the data volume. It cannot prove its **work**
> is.

`chart`'s rasteriser painted the whole column prefix for **every point**. A bar is a prefix and the
union of two prefixes is the taller one, so the picture was correct — and at a million points onto
four hundred sub-columns the same cells were painted thousands of times over: **952.61 ms against
2.72**, 476.3 ns a point against 1.4.

Every gate in this workspace was green on it, and each for a different and good reason. The round
trip replays the serialised bytes through the terminal model and compares them against the frame
that produced them, and both spellings produce the identical frame. The flat-in-`n` gates —
`writes`, `verbs`, `regions`, `stops` identical at 1k / 100k / 1M — are all **output** counters, and
the defect was entirely in work that produces none. `Raster::touched` was **right**: 2 000 000 both
ways, because the cost was `O(subh)` *inside* a visit it counts as one. The budget example measures
the engine, and the fold sits above it.

It was found by a user pressing `+` on `crates/vitui-apps/examples/latency.rs`.

## The gate is two counts, and neither half would do on its own

`crate::volume` gates each covered row on both of:

1. **a growth relation** — `work(n) / work(n / 10) <= 12`, at three volumes a decade apart;
2. **a ceiling, affine in the volume** — `work(n) <= fixed + per_input * n`.

Neither is redundant, and this is measured rather than argued
(`volume::tests::neither_criterion_would_do_on_its_own`):

- **The defect that actually shipped passes the relation.** `O(subh)` a point is `O(n)` with `subh`
  in front of it, so `naive_bars` grows by **10.26 and 10.02** a decade — the shipped fold's own
  figures — and fails only the ceiling, by twenty-five times.
- **A ceiling alone is met by any constant chosen large enough**, which is §21's first refinement by
  name: *a threshold on the wrong side of the question is not a weak gate, it is a green one*. Under
  a ceiling raised until it admits the quadratic arm, the relation still refuses it at **94.47 and
  99.41** a decade.

**Both are counts and neither is a clock.** This is the runtime's scene 19 one crate down, and for
that scene's stated reason: *a step count is the same number on every machine*, where a microsecond
figure is three orders apart between a debug binary and a release one and would be three different
orders somewhere else. The absolute figure is a **report** with the 16.7 ms budget as a denominator
and never as a pass mark, in `examples/volume_numbers.rs`. Lowering a budget to fit a measurement is
the edit §21 forbids by name; a component that cannot hold the relation produces a red row with a
number in it.

**A virtualised row's ceiling has no `n` in it at all.** `Layer::L2` reads *it costs its visible
window and never the data volume*, and `per_input == 0.0` is that sentence as arithmetic rather than
as a rounding of it. The `iff` is gated in both directions: a folding row with a slope of zero would
be a row claiming its fold is free.

## The population is derived, and the derivation found a row the ticket did not name

ADR 0033's rule is that every obligation is a query over `INVENTORY`. O1 and O3 are over `built`,
because a page or a screen for a function that does not exist is not a thing anybody can write. O6
is over **rows that take a volume**, and both halves of that come off values that already exist:

- `Layer::L2` is already the column that says *it costs its visible window and never the data
  volume*;
- `crate::memos` already answers *which rows fold on the edit*, because a `Spelling::Folded` memo is
  a value keyed on a data revision and that is what folding on the edit is.

Ticket 44's own parenthesis names six rows — the four `L2` rows plus `chart` and `plot`. **The
derivation answers seven.** `sparkline` holds `PlotState`'s two folded memos as well: it is `chart`'s
body with the chrome deleted (components 34) and it folds a million points through the same
`Raster`. That is the criterion working rather than failing — *derived, so a component added later
joins without anyone remembering* — and `sparkline` shipped ten tickets after the sentence that
missed it.

**The comparison is against an ordered list and not a subset**, because O6 is the one obligation of
the seven whose population can go quietly *smaller*: a row that stops declaring `Layer::L2` leaves
the population without leaving a failing set, and an ordered equality is what fails first.

## Every covered row owes a deliberate defect, and building them found a missing arm

*A gate nobody has watched fail is not a gate*, and for O6 that is seven arms rather than one. Six of
the seven are §21's *the instrument separates a correct build from a defective one*; `chart`'s is the
defect that actually shipped, kept runnable since the reduce.

Building the four row-shape arms found that **`collect::defective::whole_content` existed for one
caller of three**. `table` and `tree` *are* `collection` plus a rectangle split and a flatten index,
so they inherit **the single most expensive mistake available above this runtime** — and neither
could be asked to make it, because `table_with` and `tree_with` both called `collection_into`, which
hardcodes `Shape::Virtualised`. `collection_shaped` threads the one field, and
`table_whole_content` and `tree_whole_content` are that arm on the other two.

Two counters were added to shipped types, and both are the counter the defect was invisible to:

- `Raster::painted`, beside `Raster::touched`. The pair is what `crate::volume` reads for the three
  folding rows, and the reason the pair is needed is the whole of this ADR's first section.
- `edit::step_left_counted`, with `step_left` forwarding to it. The runtime's scene 19 arrangement —
  `chunked.get(i, &mut hops)` — and one loop rather than two, because a counted copy of a walk is a
  copy.

## Where an arm runs is what a debug binary can hold

Every shipped arm is measured at **10 000 / 100 000 / 1 000 000**, which is the volume O6 is stated
at. The defective arms are measured lower, per row: `naive_bars` at a million points is eighty
million paints in an unoptimised binary and `domain_per_point` at a million is `10^12` reads. **A
defect you can afford to run is a defect you can watch**, and both criteria are ratios and affine
bounds — neither has a volume baked into it — so an arm watched failing three decades down is
watched failing the same two numbers.

## Consequences

- Register rows **227, 228 and 229**, and row 30's gate becomes *O1-O6*. Two hundred and sixteen of
  two hundred and twenty-nine evaluated.
- `crate::obligations` carries seven queries, `VOLUME_MEASURED` is O6's evidence and
  `crate::volume::met` is what holds it honest.
- `examples/gates_numbers.rs` prints all seven verdicts, which until now took six files to read.
- **Two figures of ticket 44's own were checked.** Its table — the reduced bars fold at two million
  values — reproduces at **2 787.67 µs against 2 720.00**, with `touched` at exactly 2 000 000. Its
  criterion *8.5 ms of 16.7 at 1M* does **not**: one series of a million is **1.42 ms**, which is
  **8.5% of 16.7 ms**. The eight-point-five is a percentage that has been read as milliseconds, and
  the headroom is 11.7x rather than 1.96x. Asserted as measured rather than bent to fit.
