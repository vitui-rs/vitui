---
status: accepted
date: 2026-08-19
---

# The data contract is three types and no trait

`vitui-runtime` has **no traits**, and the one that was proposed is the one that was refused. The
data contract ships as three plain types:

```rust
pub struct Revision(u64);            // from one process-global counter
pub struct Versioned<T>;             // Deref; the only path to &mut is a guard
pub struct Memo<T>;                  // a cached value beside the revision it was computed at
```

A collection is drawn by passing a length and a closure. `Rows`, with `len()` and `revision()` on it,
does not exist.

## Why

The engine has zero traits, so introducing one was a decision — and both of the proposed trait's
methods fail when they are run.

`len()` is already an argument to every collection, and under a trait it becomes a **second source of
truth** the moment a view is filtered. `revision()` is a promise to *report* a number, and what makes
a bump unforgettable is `Drop` on a guard, which is a wrapper rather than a trait.

The proposal's own defence of closures for the cells — *closures cover every shape a trait would* —
is equally true of the length and of the revision. And the only trait shape that really would carry
the O(1) guarantee, one handing out a slice, **cannot be implemented for the struct-of-vectors** every
number on this map rests on (`E0308`).

The guarantee a trait was wanted for is carried by neither shape. One row drawer accepts indexed,
chunked and linked sources with the same 80 verbs and the same hit index, at **19.77 µs, 320.85 µs
and 78.40 ms** — and the *reasonable-looking* chunked source is already 321% of the whole frame
budget. O(1) indexed access is prose either way; the scene list is what keeps it honest.

## Consequences

**The revision counter must be process-global**, and the proposal did not say so. Two per-value
counters both stand at revision 1, so every memo keyed on them is blind to a swap between them —
found by building the wrong one. Under any crate line this stays safe: two copies of the runtime both
hand out revision 1 and can never meet, because `Revision` is nominal (`E0308`).

**Two revisions are compared with `==` and never with `<`.** A revision answers *is this different*,
never *is this newer*. This becomes load-bearing the moment a background worker lands: the newest
answer needs a **generation**, which is a different 8 bytes minted on the app thread, and the first
`<` between two revisions turns a landing into a silent wrong answer.

**The guard's three prices are each a gate rather than an oversight**: the data is exclusive while it
is held (`E0502`), the bump happens on drop rather than on change, and one revision covers the whole
`T`, so touching one column of a table invalidates memos of the columns that did not move. `Deref` is
implemented and `DerefMut` deliberately is not; the wrapper costs 1.005% at the call site. A
content-derived revision was built and refused at 2.59 ms a frame.

**A memo is keyed by where it is stored, consumes no `Id`, and must not join the id sweep.** The
id-keyed alternative was built and pays a full fold every time a tab is switched away and back. So
identity has five consumers, not six.

**The runtime's own share of this is ≈16 ns a frame — 0.016% — and it is simultaneously the largest
risk on the map, because the cost it governs is the caller's.** The chart fold it exists for is
84.91 µs a frame, 85% of the budget, against a 1.28 ns memo hit: **66 301×**. Forgetting to bump is
gated as a test that returns a confidently wrong maximum.

**`Memo` skips the computation and damage skips the output, but nothing skips the composition.** A
frame in which nothing changed still costs 23.58 µs, 23.6% of the budget. That number, not the
sentence it replaces, is what a reactivity layer inherits.
