---
status: accepted
date: 2026-08-19
---

# A collection stores positions in an order the caller owns

The display order of a collection — a sort, a filter, a tree's flatten, a document's wrap, a fold set
— is **materialised by the caller**, built on the edit, and spliced rather than rebuilt. The
component holds only **positions in it**: the selection runs, the cursor, the anchor, the offset and
the editing slot.

Because those are positions in an order the caller can replace between two frames, the order carries
a **revision on its length**, and the component reconciles on it:

- an edit the component asked for and the caller performed as a **splice** is reconciled exactly, and
  stamps the revision it produced — policy `Drop` by default, `Stash` on request, `Clear` refused;
- **any other movement of the order is a permutation** — policy `Clear` by default, `Remap` as the
  caller's opt-in with a stated bound, the remap being the caller's function because only the caller
  has both orders.

A component may not perform the edit. It asks; the caller drains the request after the draw.

## Why

**The materialisation is forced, not chosen.** A virtualised collection needs O(1) access to the
*k*-th visible row, and neither a comparator, a predicate, a tree walk nor a wrap rule provides one.
Rebuilt in a frame it is 21 158 µs at a million rows — 211 frame budgets — and moving it from the
component to the caller changes none of that. What makes it affordable is a memo on the edit, and
what makes the edit affordable is a splice: 157–342 µs against 40 393 for a tree collapse, 34.34 µs
against 3 216 for a keystroke at 1 MB, 129.42 against 2 977.71 for a document insert above 4 167
folds.

**The failure is a perfectly correct frame.** A sort changes no data and no length, so nothing inside
the component can notice: the frame after it draws a correct table with the wrong rows selected and
the editor open on the wrong row. Only a revision on the length makes it noticeable, and it is one
`u64` compared once a frame.

**A sort is a permutation and a fold is an interval, which is why one answer does not cover both.** A
subtree is contiguous in pre-order display coordinates, so under a splice a sorted run list transforms
in O(runs) and **cannot shatter**: a half-tree selection is 1 run and 0.04 µs under a splice against
**297 180 runs, 24 338 µs and 8 550 KB** under a permutation of the same edit. Select-all escapes
both at O(1), not by special-casing but because `[0, len)` is invariant under any permutation of that
length. A half-table selection under a real permutation is 1 run and 16 bytes becoming 268 023 runs,
4.29 MB and 22 941 µs — which is what makes `Clear` the honest default there.

**"A component may only ask" is the index's rule and is forced twice over**: a `Vec::splice`
allocates, against a zero-allocation frame budget, and taking `&mut` to the index while the draw
holds it shared is `E0502`. It is narrower than it was first written — a collapse of a *region* is
not an edit, and goes 6 open → 5 open on the same frame at 0 allocations.

**Caller ownership pays an identity debt for free.** The index is keyed on the caller's node ids —
exactly the keys that may be written to disk — so **no `Id` is anywhere near it**, and ADR 0013's
"an `Id` may never be persisted" is satisfied by construction rather than by a rule.

## Consequences

**One structure, five names.** A table's order, a tree's flatten index, a wrap index, a fold index
and a variable-row-height prefix sum are `{ node, depth, flags, h }` with fields unused. Building
five is the mistake; three names for one mechanism is already one too many.

**A gesture that changes the shape of a collection is an edit in the data contract's sense**, so it
stamps a revision and invalidates the memos keyed on it.

**Stored state is never keyed on a position that a hide or a reorder moves.** A table's editing slot
is `(row, col)` where the row is a position revalidated against the revision and the column is a
**key**.

**`Stash` does not generalise to a region.** A selection is positions in an index the component was
handed; a region collapsible is handed a closure whose ids are minted at its own call sites, so the
component has nothing to key a stash on. `Drop` is the component's answer and `Stash` belongs to
whoever owns the content's identity.

**A fold set anchored on a line number must be reanchored on every edit above it.** Ten lines inserted
at line 24 of a 200 000-line document leaves 4 166 of 4 167 folds hiding the wrong lines, with nothing
thrown and a plausible screen; the reanchor is a `partition_point` and a walk, 1.04 µs against the
72.83 µs the edit itself costs.
