# 1. The three layers

```
  your application
        │
  vitui-components ─── windows, panels, charts, lists, trees, forms, pickers
        │              a consumer of the runtime, with no privileges
  vitui-runtime ────── layout, identity, focus, hit-testing, routing, key maps,
        │              theming, overlays, the data contract
  vitui-engine ─────── cells, surfaces, layers, compositing, damage, the bytes
                       on the wire, input, and the clock deciding when they go
```

The arrow points one way and is enforced by there being no arrow back: the engine has no trait to
call upward and nothing in its public surface names a runtime concept. `vitui` is a facade crate
that re-exports all three; `vitui-components` depends on the runtime **only**, which is what makes
the claim in this guide checkable rather than aspirational.

## What the engine owns

Everything that touches the terminal, and nothing else.

- A **cell** holds an interned grapheme-cluster handle and a style. No cell, handle or style bit is
  readable from outside the engine.
- A **surface** is a grid of cells; a **view** is a borrowed rectangle of one, narrowable and never
  widenable; a **layer** is a rectangle with a z-order.
- **Damage is marked at write time**, by the three verbs that write (`text`, `fill`, `restyle`), and
  cleared by `present`. It is never derived by diffing two buffers — prior art measured a
  full-buffer diff at ~170 µs on a 300×80 screen, which is already over the budget for a whole
  frame.
- Input, capability detection and raw mode. crossterm is behind that seam and never appears in a
  public signature.

Two rules the engine keeps that shape everything above it:

> **The engine lays nothing out.** Callers bring rectangles. No layout concept may enter through the
> surface API.

> **The engine never iterates application data.** It offers clipping and offset viewports; culling
> is the caller's job. Frame cost is proportional to visible cells, never to data volume.

## What the runtime owns

The things a component needs that are not cells: identity, the hit index, the focus ring, key
routing, layout arithmetic, the theme, the overlay queue, deadlines and the data contract.

It has **no scene tree and no retained structure**. The clip stack is the call stack, the id path is
the closure tree, and what survives one draw is a handful of flat structures rebuilt from the next
draw. A widget that did not draw cannot be clicked, focused or hovered — even if its cells are still
on the screen.

It also has **no dependencies at all**, which is worth knowing when you are choosing where to put
something.

## What a component author writes against

The runtime. `Ctx` is the whole of the surface you need, and the components crate is the proof that
it is sufficient: 29 components, none of which can name an engine type.

The engine is reachable — `vitui` re-exports it, and the runtime re-exports the engine names its own
surface uses — and there are two honest reasons to reach for it:

1. **You are writing a different runtime.** The engine is designed to have more than one.
2. **You need a verb the runtime has not forwarded.** Ask for it to be forwarded; the pattern is
   established (`Driver::wait`, `permit_slow`, `suspend`, `resume` are all one-line forwards).

Reaching for it to draw a component is a mistake with a specific consequence: your component now
takes a `View` instead of a `Ctx`, so it has no identity, no focus, no hit-testing and no theme, and
nothing can compose with it.

## Which crate does a new component belong in?

Yours. There is no registry to join and no trait to implement. A component library is a crate that
depends on `vitui-runtime` and exports functions; if it also wants to build on `vitui-components`,
it depends on that too and nothing changes.

The only structural decision is the one chapter 13 is about: what you owe a reader of your crate,
and how you keep the two from drifting apart.
