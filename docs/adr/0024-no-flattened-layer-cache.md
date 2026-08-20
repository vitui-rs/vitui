---
status: accepted
date: 2026-08-19
---

# There is no flattened layer cache

The compositor rebuilds each damaged rectangle bottom-up, every frame. Layers `0..k` are not cached in
flattened form, at any granularity, and no invalidation machinery exists because there is nothing to
invalidate.

## Why

This is surprising because the map set the opposite expectation. The prior-art survey found that **no
project caches a flattened layer stack** — notcurses, termwiz and Zellij all recompute the entire
composite from scratch — and concluded that the optimisation was therefore ours to invent, carrying
real design risk. **The decision is to refuse to invent it.**

The claim the cache would have delivered is that cost tracks change rather than stack depth. Damage
rectangles deliver it already, because *damage rectangles are the dirty-plane list those projects
lacked*. Their failure is the absence of damage, not the absence of a cache. Measured with twenty
popups and their shadows — forty layers:

| damaged area | time |
|---|---|
| one cell | 171 ns |
| one row of 300 cells | 2.02 µs |
| one popup, 90×14 | 16.7 µs |
| the whole screen | 100.7 µs |

Cost is damaged area × depth. A full-screen composite at that depth is **107.3 µs against the 1 ms
full-screen budget**, and even a hundred layers stays inside it at 264.5 µs.

The cache is refused separately and on its own terms as well. Reusing layers `0..k` requires the
flattened result of *that* prefix, so full generality is one 375 KiB buffer per prefix; every cheaper
variant works only while the split point is stable, which is exactly the case where the frame was
already cheap.

## Consequences

There is nothing to invalidate on resize, so resize is *mark everything, reallocate, repaint* and is
simply correct rather than clever. There is no cache to reason about on a layer reorder, a z change or
a rectangle move. The layer stack can stay a sorted `Vec` rather than a structure shaped by cache
invalidation.

The memory that a front buffer, a back buffer and a flattened cache would have cost — 1 126 KiB at
300×80 — is not spent.

**What this depends on, and what would reopen it**: the damage model must stay exact. A structure that
over-reports damage converts every frame into a bigger composite, and the per-row alternative was
measured at 37× overdraw on a sub-cell chart. The cache is the wrong answer to that problem; the right
one is not to lose the damage.

Recorded as a methodology note because it happened twice on this map: the cache was first refused on a
**linear extrapolation from three layers to twenty that was wrong by 1.8×** — 60 µs estimated, 107.3
measured, over the figure it was being compared against. The decision survived being measured; the
argument for it did not, and the argument above is the one that stands.
