---
status: accepted
date: 2026-08-16
---

# Layout lives outside the engine

`vitui-engine` does not lay anything out. Callers bring rectangles; the engine draws into them,
composites the layer stack and writes bytes. Layout, along with the scene tree, reactivity, focus and
hit-testing, lives in `vitui-runtime`. (Two of those five are corrected by the amendment below.)
Every comparable library — ratatui, termwiz, notcurses — bundles some form of layout with rendering,
so the absence is surprising enough to record.

## Why

The engine has one job and one measurable success criterion: put changed cells on the terminal, fast,
against a written budget. Layout has no such budget and answers to entirely different pressures —
integer cell grids versus fractional flexbox arithmetic, breakpoints, content measurement, RTL. Mixing
the two produces a module where neither concern can be reasoned about alone, and makes the engine's
performance claims untestable in isolation.

Keeping the seam here also makes the reactivity choice replaceable. Layout and reactivity are deeply
entangled — a signal changing means a subtree may need re-measuring — so if layout were in the engine,
reactivity would follow it in, and the engine could no longer be used or understood on its own.

## Consequences

The engine's cost model is stated in terms it can actually control: **frame cost is proportional to
visible cells, never to data volume.** It offers clipping and offset viewports as the primitives that
make culling cheap, but it never iterates application data — a component drawing 50 rows of a
million-row tree is the component's own achievement, and one that draws all million is the component's
own fault.

`vitui-runtime` must supply a rectangle for every layer, every frame. There is no engine-side default,
no automatic sizing, and no notion of a widget asking for space.

A caller who wants only fast layered terminal output can depend on `vitui-engine` alone and never meet
a layout type. This is the concrete form of the "clear abstractions" requirement, and it is checkable:
if understanding the engine requires reading the runtime, the seam has moved and this decision has
been violated.

The reverse also holds — no layout concept may enter through the `Surface` API. That door is the one
this decision is most likely to be eroded through.

## Amendment — 2026-08-18, runtime ticket 14

The decision stands unchanged; two words in its opening paragraph do not, and they are left in place
rather than edited because they were true of what was known in August 2026 and the record is worth
more than the tidiness.

- **"the scene tree"** names nothing. There is no tree of nodes in `vitui-runtime` or anywhere else:
  the clip stack is the call stack, the id path is the closure tree, the layer stack is a sorted
  `Vec`, and intrinsic sizing takes no measure walk. `CONTEXT.md` carries the correction and the
  four answers behind it under *Scene tree — considered and refused*.
- **"reactivity"** does not live in `vitui-runtime` either. It lives above it, in the application:
  the runtime ships the loop and three hooks and nothing else, which runtime ticket 13 discharged by
  building a TEA pump and a signal graph on one unmodified runtime.

Neither correction touches this ADR's subject. The engine still lays nothing out, and the reason it
must not — that layout and reactivity are entangled and would drag each other in — is if anything
stronger now that reactivity has been measured to be outside the runtime as well.
