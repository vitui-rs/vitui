---
status: accepted
date: 2026-08-19
---

# The inventory is a value, and every obligation is a query over it

The v1 freeze ships as `vitui_components::INVENTORY: &[Component]` — a compiled value carrying, per
component, its `id`, `tier`, whether it is `built`, its layer, its families, its glyph demand set,
its number of constructions, and one flag per hostile axis (`can_shrink`, `owns_offset`, `scrolled`,
`narrow`).

Every documentation and verification obligation is a **query over it**, not a sentence in a
document: a doc page with a compiled example, a panel in the gallery, one golden screen per
construction, a declared keyboard contract, and **at least one scene per hostile axis a component
declares**.

## Why

**Every obligation this project has stated as a sentence has been broken by someone who had read
it.** `CONTEXT.md` forbids an unconditional scroll-into-view and four prototypes wrote one; the
partition rule was decided and four collections left a tail outside it; the author of the
double-write rule wrote a 15-cell double write immediately after recording it; and the ticket that
wrote *the inventory is a value, not a paragraph* then resolved as a research document, leaving its
five obligations with nothing to be enumerated against.

**A gate is only enumerable against a list.** A perfect gate is worthless if nobody knows which
twenty-nine components to run it against — which is why the hostile axes are *columns* and not
prose. The four axes each exist because a defect passed every gate then in force **and looked
healthier**: an inverted scroll drew nothing at 12.21 µs against 62.96 and was found three times
independently; a stale tail left 71 of 80 rows on screen with the defective build 2.3× faster marking
226× fewer cells; twenty wheel clicks moved the offset 0 against 16; an overlap was green at 300×80
and red at 60×20. Three of the four were caught only by an equality against a reference render.

**A table holds a disagreement that prose cannot.** The freeze put `file_picker` and
`file_preview_pane` at Tier 3 naming a ticket as their owner; that ticket then resolved having built
both, and its own line says so. Neither row moved, because a paragraph has nowhere for a later ticket
to write. The `built` column is gated against the tier, so the disagreement is an assertion with both
names in it rather than a contradiction between two documents.

**The gallery equality is two gates, not one.** Nothing shown may be absent from the freeze, and
everything `built` must have a panel — over `built` rather than over all twenty-nine, because
thirteen entries have nothing to show. Finding that out grew the gallery from ten panels to twelve.

**The glyph demand set has to be per component, because the global count is blind to the defect that
matters.** A global "36 of 190 pairs collapse at ASCII" is expected and harmless; what it cannot see
is that one component draws both `Ellipsis` and `ArrowRight` and ASCII spelled both `>`.

**The `constructions` column makes the freeze independent of an unsettled runtime table.** Block
elements are the Unicode rung by the glossary's own definition, so `chart` is 2 and `plot` is 3
whatever the runtime's sub-row table currently says.

## Consequences

**A component is added by adding a row**, and the gates then demand its doc test, its gallery panel,
its golden screens and its hostile-axis scenes without anyone remembering to ask.

**The two documentation obligations do not substitute for each other**: a doc page catches an API
that cannot be called from outside the crate, a gallery panel catches an inventory that has drifted
from what ships, and neither catches a wrong cell — that is the golden screens and the hostile-axis
scenes.

**Goldens are per construction, not per matrix cell.** Nine screenshots per component is nine times
the maintenance for a claim the count in the theme's matrix already makes, and screens declared
identical are asserted identical instead.

**The freeze is twenty-nine components in three tiers, and the tier ships with it.** Six collapses
(five measured, one marked as a judgement) and three additions no frequency count produces. The
frequency rule the set inherited does not reproduce — 21 of its 28 fall below the threshold under a
generous synonym set — so the set survives as product intent while the freeze rests on what was
built. A product call can move the tier boundary; it cannot move the column.
