# Every data-bearing component holds 60 Hz at a million rows, and a gate says so

## Goal

**60 Hz at 1 000 000 inputs, for every component that takes a volume.** Not as a number in a
report nobody evaluates, but as a standing obligation over `INVENTORY` — a sixth one beside
O1–O5, queried the way they are and red until every row answers.

This is a requirement, not an aspiration, and it exists because the library missed it once
already in shipped code with every gate green.

## What went wrong, and why nothing caught it

`chart`'s rasteriser painted **the whole column prefix for every point**. A bar *is* a prefix and
the union of two prefixes is the taller one, so the picture was correct — and at a million points
onto four hundred sub-columns the same cells were painted thousands of times over.

| arm | before | after | per point |
|---|---|---|---|
| `Bars` | **952.61 ms** | **2.72 ms** | 476.3 → 1.4 ns |
| `Marks` | 6.00 ms | 5.48 ms | 3.0 → 2.7 ns |

**Every gate in this crate was green throughout, and each one for a good reason:**

- The **round trip** replays the serialised bytes through a terminal model and compares against
  the frame that produced them. Both spellings produce the identical frame, so it cannot see it.
- The **flat-in-n gates** — `writes`, `verbs`, `regions`, `stops` identical at 1k / 100k / 1M —
  are all *output* counters. The defect was entirely in work that produces no output: the same
  cells, written again.
- **`touched`** counts points visited, and it was correct: 2 000 000 both ways. The cost was
  `O(subh)` *inside* each visit, which no counter in `§20`'s nine expresses.
- The **budget example** measures the engine, and the fold is above it.

So the shape of what is missing is precise: **this crate can prove a frame's output is flat in
the data volume, and cannot prove its work is.** That is the hole this ticket closes.

It was found by a user pressing `+` on a demo application.

## Acceptance criteria

- [ ] **O6 exists in `obligations.rs`, queried over `INVENTORY` and not over a hand-written list.**
      The population is *every component that takes a data volume* — `Layer::L2` is already the
      column that says *it costs its visible window and never the data volume*, plus the rows that
      fold on the edit (`chart`, `plot`). Which rows those are is derived, so a component added
      later joins the obligation without anyone remembering to add it.
- [ ] **An empty population is `Unmet`.** `Verdict::of`'s existing rule, for ticket 01's reason:
      an equality between two things that do not exist holds.
- [ ] **The gate is a growth relation, not a timing.** *A timing is a report, and a gate only at
      cliff granularity* — so what is asserted is `cost(1M) / cost(100k) ≤ k` for a stated `k`
      close to 10, and `cost(100k) / cost(10k) ≤ k`. Superlinear work fails it; a machine twice
      as fast passes it unchanged. **The runtime's scene 19 is the precedent** — it counts steps
      and gates a growth relation for exactly this reason.
- [ ] **The absolute number is reported beside the relation, with its headroom**, in a
      `*_numbers.rs` this lineage already has. 60 Hz is 16.7 ms; the fold's share is the figure
      that matters and it is stated as a fraction of the budget, not as a pass mark.
- [ ] **A per-point ceiling, because the relation alone is blind to a constant.** A fold that is
      linear at 4 µs a point is linear and useless. The ceiling is stated per component and
      derived from its budget share at 1M.
- [ ] **The gate is watched failing.** The old prefix-per-point loop is already kept as a
      test-only twin in `chart::raster::reduce_tests`; O6 must be watched red against it, or
      against an equivalent deliberate superlinearity, on every component it covers.
- [ ] **`chart` and `plot` are green under it on merit** — 8.5 ms of a 16.7 ms budget at 1M,
      measured 2026-08-25 — and every other covered row is either green or red with a named
      failing set. **Red is an acceptable outcome for this ticket**; unmeasured is not.
- [ ] **A register row**, so the standing is a value rather than a paragraph, and O6 appears in
      `examples/gates_numbers.rs` beside the other five.

## What this ticket may not do

- **It may not lower a budget to fit a measurement.** `latency` declared `max_frame_rate: 10.0`
  for one commit to survive a slow frame, which is precisely the edit spec §21 forbids. If a
  component cannot hold the relation, the ticket's output is a red row with a number in it.
- **It may not reach for downsampling.** `chart::WhyThereIsNoDownsample` is a decision: a stride
  sample loses the spike that is the reason anybody is looking. The bars fix was a *reduction* —
  the same points, visited once, aggregated before painting — and that is the shape available.
- **It may not weaken the flat-in-n output gates**, which remain correct and remain necessary.
  O6 is a second instrument on a second axis, the way `crate::dense`'s narrow axis needed two.

## Open question for whoever takes it

**Whether O6 belongs to the components at all.** *Frame cost is proportional to visible cells,
never to data volume* is the engine's stated invariant, and a component that folds on the edit is
outside it by construction — `chart` costs the volume on the edit and the rectangle on the frame,
which is what its own title bar claims. If that is the rule, then O6 is really two obligations:
*the frame is flat in n* (already gated, output counters) and *the edit is linear in n with a
stated constant* (this ticket). Deciding that is a spec §17 change and belongs in an issue against
the map, not inside the ticket.

## Progress

Not started. The evidence above is from components ticket 28's `chart` and the session of
2026-08-25; the `Bars` reduction and its equality gate are on `master`, and the per-arm figures
are reproducible with a fold harness over two series of a million points.
