---
status: accepted
date: 2026-08-19
---

# A memo's key is every input

A memoised value in a component is keyed on **every input it was computed from**, not on the data's
revision alone. The inputs that are not data — the width, the sort, the expansion set, the
repertoire, the theme's revision, the rectangle, the axis policy — are the ones that get forgotten,
and forgetting one is silent.

The same rule reaches the asynchronous layer as a **dedupe key**: an answer carries the identity of
the question it answers, and a component may write it only on an equality against the question it is
currently asking.

## Why

**Six independent arrivals, and every one of them recomputes *less*, runs *faster*, and is wrong on
the rendered surface.**

| keyed on | forgot | the price |
|---|---|---|
| the data | the theme | a paint from the previous palette |
| the data | the sort | a re-sorted table answering with the previous order |
| the data | the expansion set | an expansion edits no application data, so no revision moves |
| the revision | the width | 625 rows drawn where 875 are needed, 69 of 80 rows differ, recomputing 1 against 2 and running twice as fast |
| `(data, tier)` | the theme's revision | hits at half the cost, wrong in 598 characters |
| the revision | `(w, h, kind, sx, sy, series, y0, y1)` | 2 330 cells wrong, recomputing half as often |

**The natural detector points the wrong way.** `Memo`'s `recomputes` counter goes *down* when the key
is wrong, so the instrument the runtime provides for exactly this question reports improvement. The
two detectors that work are the memoised object **recording the input it was built at** and the
**rendered surface**.

**It does not crash, because every real component bounds-checks once**, which turns a stale raster
into a smaller, older picture drawn inside a bigger rectangle, and a stale wrap index into a document
drawn at the previous width. Both are plausible screens.

**The asynchronous form is the same mistake in a layer where no test on the answer can see it.** A
preview pane's question is a *file* and every natural way to name it names a *position*. One re-sort
— one line of application code, no keystroke — moves 197 of 200 positions, and the position-keyed
pane is wrong on **100 of 100 frames afterwards** while asking **1** question in its life: the
question still matches, so nothing posts, so nothing wakes, so no frame corrects it. The same hole
opens with no user in it, when a still-loading directory splices batches into a sorted order: wrong
after 7 of 7 batches. Identity keying is wrong for one frame, the decode's latency.

## Consequences

**A component spells its compound key out at every call site.** `Memo<T>::get` takes one `Revision`
and cannot express any of this; six crates hand-roll a composite key, and the obligation to give the
runtime a keyed memo is recorded against the runtime map rather than worked around here.

**Two memos in a chain are two memos.** A chart's axis range is an aggregate *and* an input to the
raster's key, so a resize invalidates the raster and not the range. Collapsing them loses that.

**A memo carries the theme in its key iff its value is made of paints *or glyphs***, and the key is
the theme's own `Revision`, not the axes — `(data, tier)` is the plausible fix that survives a
palette swap and dies on a repertoire swap.

**A value that can be *maintained* is not a memo.** A wrap index, a flatten index and a fold index are
spliced on the edit and stamp the revision the memo then hits on: 34.34 µs a keystroke against 3 543
held inside a `Memo` at 1 MB. The split is that a memo covers an input the component cannot reconcile
against, and a splice covers an edit it can.

**The asynchronous payload carries eight bytes it would not otherwise need** — the question's
identity — and the pane's write is guarded by an equality against it.
