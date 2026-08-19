---
status: accepted
date: 2026-08-19
---

# No geometry crosses a frame

The hit index carries **no rectangles**. An entry is `(id, interest, over, scrollable)` — 16 bytes,
in draw order — and containment is decided *during* the draw, in the widget's own coordinates, from
the pointer that travelled down the `Ctx`. Pointer outcomes are then awarded at the **end of the
frame that drew**, from the index that has just been built.

The rule is not "the runtime holds no geometry". It is: **geometry is needed inside a frame and never
across one.**

## Why

`architecture.md` §6 proposed `(id, layer, rect, interest)` scanned in reverse on the next frame, and
named a one-frame lag as the price — attributing it to z-order overlap, which is the case an author
notices. That attribution was wrong, and the two cases it missed are worse than the one it named.

An `over` bit is computed against the pointer *as it was when that frame drew*. So nothing at all is
hovered on the frame the pointer crosses between two widgets that do not even touch. And a press
arriving where no frame has drawn — **every press at `Buttons` tracking, since that level sends no
motion events** — reaches nothing.

Awarding at the end of the frame closes both. The press is awarded from this frame's index, with
`begin`'s answer demoted to a guess that buys in-frame feedback and can never become a wrong click.
Hover-as-a-style is settled at the end for **37.5 ns**, which also closes the z-order lag on the
*first* frame of an overlap. What is left one frame old is hover that changes content or size, and
that is the whole residue.

Dropping the rect is also what makes the index cheap: 16 bytes against 32, **312 entries** on a dense
300×80 screen — identical over 2 000 and 1 000 000 rows — and a reverse scan of all of them at
**142.3 ns, 0.14% of the frame budget**.

## Consequences

**A row with two targets must declare per target.** An entry carrying no geometry cannot separate
them. That is the mechanical trigger behind 312 regions where an earlier prototype counted 4, and it
is a component-authoring rule rather than a runtime cost.

**Modality is an ordering, not a membership.** The barrier is one `Option<usize>` into a `Vec` that
is already in draw order: everything past it is inside the modal's scope by position alone, and no
entry carries a scope of its own. `Option` rather than `usize`, because `0` meant both "no modal" and
"a modal over nothing". This is also what forces a nested overlay's z-order to count from its
parent's layer — sorted into its own band it would land below the barrier that exists to protect it.

**The rule admits exactly one rectangle, and it obeys the rule rather than breaking it.** The focus
ring's entries carry a rectangle in the enclosing scroll area's *content* coordinates, read at the
end of the same frame — in the same breath as the press award — so that a `Tab` onto a row below the
fold can be scrolled into view in one frame instead of two. What crosses the frame boundary is
`IntoView`: 16 bytes naming an area and an offset, and no `Rect`.

**One channel cannot obey it, and that is documented rather than fixed.** Wheel chaining is resolved
from the previous frame's index, because the offset is read *during* the draw by the widget that owns
it. The residue is one wheel click, at each end stop and on an area's first frame.

A held grab is exclusive and the wheel is withheld while it is held; without exclusivity a splitter
drag lights every button it crosses.
