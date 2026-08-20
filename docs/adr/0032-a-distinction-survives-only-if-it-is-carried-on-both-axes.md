---
status: accepted
date: 2026-08-19
---

# A distinction survives only if it is carried on both axes

Degradation is measured on **signals** — `(Option<Glyph>, Role)`, what actually reaches a cell — and
a distinction a component draws survives the whole repertoire × colour-depth matrix **iff it is
carried on both axes**: a glyph difference *and* a paint difference.

Degradation is **resolved, never branched**. The theme narrows each declared distinction to one bit
at construction, from the palette as it arrives at the terminal and the repertoire as it was
declared. A component branches on the bool and names neither `GlyphSet` nor `ColorTier`.

And the line between the two mechanisms is two counts. **A `Glyph` is a lookup with a spelling at
every level, every spelling exactly one cell, and no spelling blank.** Anything failing either is not
a `Glyph` — it is a **branch**, and a branch changes the construction.

## Why

**The colour half of the no-literals rule was closed by a type and the glyph half was never closed,
and the count is the asymmetry**: `GlyphSet::` appeared 24 times across four component crates while
`ColorTier` appeared 0 times. A component crate cannot add an entry to `Glyph`, so four crates — nine
by the time anyone counted — shipped a private `mod missing`: six glyph literals and a `match` on
`GlyphSet`, **byte-identical in all of them**. That is exactly the failure a shared theme exists to
prevent, and two components doing it will disagree.

**They already had.** The shadow table spelled the ASCII ellipsis `>`, which is precisely an ASCII
`ArrowRight`, so **468 truncated labels ended in the collapsed-node marker** — and `tree` draws
both. The correction to the real table reached none of the nine private copies.

**A missing entry has no signature at any level of the stack.** An absent entry returns `""`, a verb
handed an empty run writes nothing and marks nothing, so the incomplete table is **77 cells lighter,
no slower, with identical damage and identical allocations**. The only detector is the rendered
surface: 135 cells and 77 of 80 rows at Extended, **545 cells and 80 of 80 rows** at ASCII.

**Absence is not representable, and that is the design rather than an omission.** ADR 0009 says a
component that cannot express itself builds *something different*; drawing nothing is not that.

**The matrix is a count, not nine screenshots.** Across the nine cells: role pairs collapsing 1 / 2 /
13 of 78, glyph pairs 0 / 0 / **36 of 190** (at ASCII, the nine box-drawing entries all spelling
`+`), signal pairs 21 → 1 677 of 37 128 — and the number a component acts on, **distinctions lost
0 / 1 / 2 of 10**. Everything glyph-bearing survives all nine cells, because the glyph carries it.
The second lost distinction is the one worth naming: **`Danger`, `Warn` and `Ok` all quantise to
bright white at sixteen colours, so a traffic light is monochrome.**

**Resolving costs 15.7× less per decision than branching, on a fact that cannot change during a
frame** — `shows` 0.524 ns against `signals_differ` 8.22 ns, 93.29 µs against 95.25 a frame — and
the two render **cell-identical screens at all nine cells**. So the frame delta is not the argument.
The argument is that a component which branches must *name the axis*, and naming it is what produced
the 24 occurrences and nine divergent copies.

## Consequences

**The glyph table is 20 entries, not 7**: the four arrow ends (one family serving both a scrollbar's
steppers and a disclosure marker — entering them twice under two names would be a collapse), the four
box corners, `Ellipsis`, the four tees and `Cross`.

**The lookup table has two rows, not three.** Entries whose Unicode and Extended spellings differ:
**0**. Everything that would distinguish the top two rungs changes the construction and the number of
samples asked of the data, so it is a branch — which is how `plot` earns the third rung on its own
while `chart` is byte-identical at Unicode and Extended.

**A collapse inside one glyph family is legitimate; across families it is the defect.** All four box
corners are `+` at ASCII on purpose, and a gate written pairwise fires on every border. The gate is
**cross-family collapse == 0**.

**No probe exists and none may.** A repertoire is declared, never detected (ADR 0010), so
`Theme::with_glyphs` is the operator's promise arriving rather than a capability question. No engine
change is implied and no public signature moves.

**`Theme::glyph` returning `&'static str` cannot be closed by a type the way `Paint` was**, because
text is legitimately a component's own content and there is nothing to forbid. What replaces the type
is a count: **`GlyphSet::` occurrences in `vitui-components` == 0**, with no private `mod missing`.

**A memo carries the theme in its key iff its value is made of paints *or glyphs*** — the extra word
this decision adds to the runtime's rule, and it costs 598 wrong characters when it is missing.
