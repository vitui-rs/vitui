---
status: accepted
date: 2026-08-19
---

# A component writes a partition of its rectangle

Every component and every shared helper writes a **partition** of the rectangle it is handed. Each
cell it is responsible for is written **exactly once**, by exactly one of its branches, and the cells
it does not write are **named in its return value**.

Two equalities state it, and both are gates:

```
writes == distinct cells touched              no cell twice
distinct cells touched == area.w * area.h     no cell never
```

Concretely: `text::fit` draws the text and then pads the remainder — there is no verb that fills
first. `frame::block` draws its frame, its title-split top run and its padding ring, and **returns
the interior unwritten**. A scrim is the four rectangles *around* the dialog, never a fill beneath
it. A selection or a focus ring is a **paint chosen before a cell is written**, never a restyle laid
over a drawn row.

## Why

**The engine marks damage at write time, so a cell written twice with two different values inside one
frame is a change, and the equality filter is doing exactly its job when it says so.** The runtime
map handed this map the fill-then-draw case (26 of 48 cells on a steady dropdown frame; 10 814 of
24 000 on a dense screen). Built, it turned out not to be about fills at all: components ticket 01
found it in five places, three of which are not fills — an application-level clear (6 662 cells a
frame), a border run crossing its own title (15), a leaf overlapping its sibling (432), a scrim under
a dialog (229).

**The 15-cell instance is why this is an ADR and not a style note.** It was written by the author of
the rule, in the corrected build, immediately after writing the corrected chip. Nothing about a
border run looks like a fill. It was found by a count and by nothing else.

**The price, measured three times on one screen.** Naive against correct: 214.58 µs / 69 647 writes /
6 662 damaged against 84.96 / 20 784 / 0; and on the ordinary frame — the pointer moves onto one chip
— **6 662 damaged cells against 8, 833×**. Independently on a second screen: 131.29 µs / 23 098
against 69.33 / 0. **The two screens are identical, 0 of 24 000 cells apart**, which is what stops
the damage count being gamed.

**The second half needed an instrument, and without one it was live in half the library.** *No cell
never* was counted by nothing on either map, because a cell nobody writes keeps what was already
there. Stamping a sentinel style over the base layer between frames finds **9 956 cells of 53 280
(18.7%) in six panels of twelve** — and the same six panels keep the previous palette permanently
after a theme swap, the swap excess **equal to the unwritten count on five of the six**. One rule,
two symptoms, and the symptom only shows on the one frame a theme changes.

**A restyle is free only when the component's own next draw already produces the value the restyle
produced.** A deferred hover award satisfies this because the widget also branches on
`Response::hovered`. A restyle used to *replace* the branch — the entire appeal of a focus-ring or
selection helper — never does: 26 / 31 / 56 / 261 cells re-damaged every frame forever, and the
restyled row is also 7 cells *different* from the correct one.

## Consequences

**`frame::focus_ring` does not exist.** It becomes a `Role` argument into `block` and a `Faces` into
`press`. Selection becomes `face_paint`, a `Paint` the row drawer is handed — which is a row
signature, `(cx, rect, index, Face)`.

**Routing through a helper costs nothing against the discipline** — `fit` against the same order
written by hand is 0 of 24 000 cells apart with identical write counts — so the helper is not a
convenience over hand-written correctness, it is the only form of it anybody keeps.

**The rectangle and the content are told apart.** The owner of a rectangle writes all of it; a
component handed a rectangle writes all of that. A container whose content shrinks below its
viewport must write the tail: four components shipped without it, and the defective build is 2.3×
faster marking 226× fewer cells.

**The defective build is always faster and always marks less.** Every counter moves the wrong way, so
the rule cannot be enforced by a budget or by a damage threshold. It is enforced by two equalities,
one of which is currently red on six panels and pinned in its failing state.

**A screen still clears once** — on its first frame and on a resize — because a screen whose gaps are
never painted is not the same screen. Once, not every frame; the difference is 6 662 damaged cells.
