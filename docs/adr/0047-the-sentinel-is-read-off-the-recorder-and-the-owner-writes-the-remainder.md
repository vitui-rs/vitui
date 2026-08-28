---
status: accepted
date: 2026-08-28
---

# The sentinel is read off the recorder, and the owner writes the remainder its component named

Components ticket 40 inverts register row 7 — *every cell of the rectangle written at least once*,
the second half of spec §2's partition rule (ADR 0026). It had been pinned red with its exact failing
set since ticket 03, longer than any other row on this map bar one, and its prescribed detector was
never built. Two decisions are recorded here: what the detector is, and who writes the cells a
drawing does not.

## The detector is `crate::runner::Pen`, and ADR 0023 did not have to be reopened

Spec §2 prescribes a **sentinel**: stamp a paint no `Role` can produce over the **base** layer between
frames, draw one more, count the cells still carrying it. It has to be the base layer, because the
engine filters a write whose value equals the cell's current value, so downstream of that filter a
cell rewritten with what it already held is indistinguishable from one never written.

`crate::counters::sentinel` named three barriers. Runtime architecture issue 22 lifted the first
(`Rgb` is nameable, so `Theme::custom` is callable here); the second was never one; the third is
**ADR 0023 — the cell is never visible in the public API**, which holds against `vitui-engine`'s own
callers as well as this crate's. So the prescribed instrument cannot be written, and the ticket's
first job was to decide whether that is a barrier to lift or a question asked in the wrong place.

**It is the wrong place, and the pair is the argument.** The rule is two halves of one equality:

```
writes == distinct cells touched            (no cell twice)
distinct cells touched == area.w * area.h   (no cell never)
```

The first half has been read off `counters::Tally` since the first component ticket, and `distinct`
has unioned **in the coordinates of the frame's root** since ticket 19 (runtime architecture issue
32). The second half is that same union compared against the area. Read on the composited surface it
would be an equality between two different instruments — which is exactly the mistake §21's own
refinements are about.

And the recorder is the **stricter** of the two, in the direction that matters:

- A verb that does not go through the caller's `Ink` is invisible to the recorder, so it
  **under**-counts what was written and the gate fails loudly. Components 39 shipped precisely that
  defect — the gallery's clear bypassing the caller's ink — and this is the gate that catches it.
- A surface probe counts the engine's own clear as a write and passes quietly. It cannot tell *the
  component wrote this cell* from *something painted over it first*, which is the whole subject.

So `sentinel` takes a `Canvas` and answers a count. What was lost by not lifting the barrier is
nothing this rule needs; what would have been lost by lifting it is ADR 0023.

## The owner writes the remainder, and the return value is where it is named

§2 states the rule in three clauses: a component writes every cell of the rectangle it is handed,
each exactly once, **and the cells it does not write are named in its return value**. On the
assembled gallery the failing set is four numbers and three mechanisms, all of them the third clause:

| at 300x80 | cells | named by |
|---|---|---|
| `panel` | 294 | `Frame::interior` — a `block` returns the rectangle it did not write (§3) |
| `collapsible` | 432 | `Disclosure::used` — *every row below is the caller's, and this is how it is named* |
| `scrollbar` | 630 | nothing: the **caller** narrowed a wider tile to a three-column bar |
| two slots with no panel | 1 600 | nothing: the grid is `cols x rows` and twenty-eight panels do not fill six by five |

The last two are the owner's own arithmetic and not a component's omission, which is the distinction
ADR 0026's C07 case turned on: *filling every frame is the double-write defect and not filling is the
unwritten defect, and the resolution is that the rectangle and the content are told apart*. A blanket
fill would trade one half of the rule for the other; both halves are asserted at once, at every size
and on every page.

**Twenty-six of the twenty-eight components already met the first clause**, measured before anything
was changed: handed a rectangle taller and wider than its content, every row of the freeze wrote all
of it except `select` and `file_picker` — 576 cells of a 48x13 interior each. Those two are exactly
the two whose body is in another layer (§12), and their remainder **could not be named**: both return
the runtime's `Response`, which has no field for a rectangle. Writing it is therefore the only
reachable answer rather than the chosen one, and the paint is the face's, because `press_into`
declares the hover award over the whole of the rectangle — a rectangle a component paints one row of
and awards all of is a rectangle whose hover repaints cells nobody wrote.

## What this costs and what it does not

The gallery hands each panel the whole of its tile's interior and the twelve that used to be handed
one row are handed thirteen. `chip`, `button` and the three toggles centre themselves in it; `meter`
and `slider` **fill** it, which is a fatter drawing than before and a true one — a meter is as tall as
its rectangle. At 300x80 the screen wrote **13 748** of its 24 000 cells before this ticket and writes
all 24 000 now, and the arm that leaves every remainder alone is 21 044 writes and 1 871 verbs against
1 933. The frame stays inside §20's full-screen class, `writes == distinct` and `merges == 0` at every
size and on every page, and the allocation total stays zero.

**Row 8 did not move with it, and §21 pins the two rows in one sentence** — *the swap excess equal to
it on five of six*. On this screen they are independent, and the reason is components 39's own: `swap`
carries **one surface** across the change and the first frame clears, so a cell nobody writes on a
steady frame is still a cell somebody wrote once. It is inside `written` and it counts as `kept`. At
100x30 a rung change keeps 2 005 of 3 000 with row 7 red and with it green, and both arms are run.

`crate::form`'s third panel still leaves an unwritten tail, and that is the **named exception** §21's
third refinement asks for rather than a hole: the form is a fixture, its two helpers each write a
partition of what they were handed, and the tail is what buys two of ticket 06's three measurements.
A gate run over every screen in this crate would have to delete them to go green.

## Alternatives rejected

- **Lift ADR 0023 with a survivor count on the engine.** `counters::sentinel`'s own header proposed
  it — *a count is not a readback, so ADR 0023 survives it*. It would have been a new engine surface
  and a new runtime door for a number this crate already has, measured on the wrong side of the
  equality's other half.
- **A `painted` flag on the panel's background.** ADR 0026's C07 case, and the reason row 7 and row 6
  are one rule: guarded, it leaves the body unwritten; unguarded, it re-damages everything the body
  draws.
- **Let the two overlay owners keep their one row and have the caller narrow.** It works for the
  gallery and leaves the rule unmet for every other caller, with nothing in either signature saying
  so.
