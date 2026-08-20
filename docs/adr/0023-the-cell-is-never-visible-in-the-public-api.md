---
status: accepted
date: 2026-08-19
---

# The cell is never visible in the public API

`Cell`, `GraphemeId`, the packed style bits and both handle tables appear in **no public signature**.
Drawing verbs take a `&str` and an opaque `Style`; `restyle` takes a `Restyle` descriptor. There is no
way for a caller to read a cell back out of a surface.

## Why

**It is what makes the interior free to move.** The handle tables were placed a whole ticket later than
the surface API was designed, and the placement — one set per engine, reached through the draw context
rather than stored in a `Surface` — changed no caller-visible signature at all. The same is true of the
style word's bit layout, of the `CONTINUATION` and `EMPTY` sentinels, and of the packet's side tables.
A public `Cell` would have frozen every one of those on the day the first component was written.

It is also what keeps the two closures out of the API that turned out to be wrong. `restyle` taking
`Fn(Style) -> Style` hands the caller a handle it has no table for, so extended styles came back
untouched; the descriptor exists because the cell does not.

## Consequences

**The engine cannot tell a caller what is already on screen, and that is a real capability given up.**
A sub-cell chart packing eight logical pixels into one braille cell must keep its own `w × h` shadow
bitmap, because two dots landing in one cell have to be merged before the cell is written. That costs
one byte per cell of component state, and it is the price of this decision rather than an oversight in
the verb set.

A second consequence lands on the runtime: **the role a cell was painted with is not recoverable from
the cell.** A resolved style is a style, and whatever named it is gone by then. A debug inspector that
wants the role needs the theme to record it, which is a cost on every verb.

The engine's own diffing is unaffected, because the engine does not diff — damage is marked at write
time. The one component that genuinely wanted to diff its own output was measured doing so and it did
not pay: the comparison belongs at pack time, against the mirror, where it rides a scan that is already
reading every damaged cell.
