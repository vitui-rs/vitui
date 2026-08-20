---
status: accepted
date: 2026-08-19
---

# Scrollbars are reserved, not overlaid

A `scroll_area` reduces its rectangle by the width of the bars it will draw **before** it calls its
body. Bars are never drawn over the body's cells.

The decision of which bars are needed is a **fixpoint over the bar-free rectangle**, computed from
the content extent and the full rectangle — never from last frame's reduced rectangle. And:
**reserved auto-hiding bars require a declared content size.** A measured extent and a hideable
reserved bar are incompatible.

## Why

**An overlay bar re-damages the body's cells every steady frame.** 3 535 cells a frame against 0 for
the reserved twin on the same screen with the same content — ADR 0026, one container up. The rule
"an overlay bar is free where nothing is drawn under it" is exact and measures 0 where it holds, but
it is not a property any component can guarantee of its body, which is what makes the reservation
unconditional rather than a default.

**The amplifier is an engine fact.** Damage is one span per surface row, so an overlay bar costs the
distance to whatever else changed on that row rather than its own column: ×8.4 where two bars contest
~423 cells, and the factor grows with how far apart the areas are.

**The fixpoint cannot oscillate, and that had to be checked rather than argued.** Reserving is
monotone — a bar only removes room, and a smaller viewport can only need more bars — so two monotone
booleans have one least fixpoint. Fed back into itself over **5 475 600 viewport × extent pairs: 0
failures, worst case 3 passes.**

**The monotonicity has a precondition and it is on the body**: *a smaller viewport may not produce a
smaller extent.* A responsive body that breaks it flips the decision **99 times in 99 frames with no
input** — the reflow loop this decision was opened to settle.

**The obvious implementation is the wrong one, and it fails quietly.** Computing from last frame's
*reduced* rectangle — which is how a reader naturally writes it — never loops and never fails to
terminate, but has **hysteresis**: over content that fits with no bars at all it keeps both, losing a
row and a column permanently on a screen that looks correct. That is why the second sentence of the
decision exists.

## Consequences

**A caller that wants auto-hiding bars declares its content size.** The same shape of answer the
chart family reached independently for its axis gutter, and for the same reason: when one end of a
layout loop is data, decouple it rather than solve it.

**Sticky regions are pinned columns transposed, not a rhyme.** A band is a rectangle split that
shares one of the two offsets and pins the other to zero, and **it must be a view** — arithmetic
gives identical writes, identical verbs, identical output and 3 243 cells re-damaged every steady
frame. Header, footer, pinned column and gutter are one construction with an axis argument, and all
four declare **one** hit entry, because a band that were a second scroll area would win the wheel
from the body it is a header of.

**The extent is in content cells, never in rows.** `Σ h` is the extent once rows can differ; measured
in rows, every count in the frame is identical and the area reaches row 799 999 of 999 999 — 20% of
the content unreachable on a screen that looks healthy — with the thumb out by up to 7 cells of 69.

**Inside a popup this decision moves rather than applies.** A popup owns its own viewport, so the
gutter is decided once in 0 passes; what changes is ownership — the bar decision belongs to the
overlay's *body*, because the owner asked for a size and the runtime answered.

**A preview pane is a positive case, not a new question.** A declared content size that is a field of
an asynchronous answer is stale while the next answer decodes (4 000 where 800 is right), which is
survivable precisely because the bars are reserved.

The steady frame is 48.83 µs / 0 marked / 0 allocations over a million rows on both axes, and 1.00×
from 1k to 1M with identical writes and verbs.
