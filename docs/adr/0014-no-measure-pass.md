---
status: accepted
date: 2026-08-19
---

# There is no measure pass

A component is `fn(&mut Ctx, Rect, …) -> Response`. Layout is `Rect → [Rect]`, computed by plain
functions over integers before anything is drawn. There is no `measure` trait, no layout node, no
two-pass walk and no retained size cache.

A container that must size to its contents calls a **sizing function**: a plain function beside the
component, taking the same `&data` plus the width or height it is about to be given, and returning
integers. It takes no draw context, so it cannot draw, cannot claim an identity and cannot route.

Every comparable toolkit — Flutter, Yoga, JavaFX, Compose — has a measure phase. The absence is
surprising, and both alternatives were built and priced before it was decided.

## Why

There are two exits from *no measure pass*, and the weaker one is the trap: it argues against a measure
pass by arguing against a *trait*. A component is already a function and a function can be called
twice, so a **dry run** into a discard surface gives intrinsic sizing with no trait, no node and no
retained tree. That is the opponent this decision actually had to beat, and both are priced below.

**The trait form does not die of cost** — same rectangles at 1.03×, and 0 allocations against 2 once
it takes the caller's buffers. It dies of `E0499`: a fit-container holds every child alive across its
measure loop, so two children writing through one borrow cannot coexist. It *compiles* on disjoint
fields, which is why it is easy to miss. Its second death is that `measure` has no answer for a
virtualised child: honest is `u16::MAX`, a rectangle nothing can draw and still 15× short of a
million rows.

**The dry run dies of three numbers and one design fact.** It is a second whole frame (dense screen
2.07×, +34.89 µs, against 85.79–569.00 ns for the sizing functions — 13.3× for the same answer). It
measures the *clip*, not the content: a 1M-row list under an 80-row clip reports 80, and the honest
question costs 5.81 ms against 0.96 ns. A frame has side effects that now happen twice — a log tail
reaches `tick = 2` where the author wrote 1. And it needs its own `Frame`, or it claims every id
twice, doubles the hit index and drains the key queue.

## Consequences

**Sizing a dialog to its contents is a function call, not a property.** That call is cheap — 569 ns
— and the survey's containers that need sizing need it for text (measurable) or for a list (`len ×
row_height`, arithmetic).

**The real cost is drift, and it is silent in both directions.** A sizing function and its component
are two expressions of one layout with nothing holding them together. Runtime ticket 01's
`detail_height` disagreed with its own component at **221 of 229 widths**, including the 78 every
number in that ticket was measured at, and three tickets did not notice.

Two things follow, and they are obligations rather than advice. The component **lays itself out from
the arithmetic its sizing function publishes**, so there is one expression and not two. And the
**dry run survives as the test** that keeps them equal — draw into a discard surface, read the drawn
extent, compare with what the sizing function claimed, across a width sweep. Every component that
publishes a sizing function owes that gate. `Ctx::measured` is that mechanism and is scoped to it; it
is never the layout mechanism, and the measured world has no focus, no hover, no press and no keys,
which limits what it may be asked.

The one expensive sizing function is not a sizing question: column auto-fit is 17.60 µs at 1k rows
and 17.69 ms at 1M, is a derived aggregate, and memoised is 1.73 ns. This decision keeps that fold
visible at the call site where a measure pass would have hidden it inside a container.
