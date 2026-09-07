# vitui-runtime

Layout, identity, focus, hit-testing, routing, key maps and theming on top of
[`vitui-engine`](https://docs.rs/vitui-engine).

**Nothing is published. The version is `0.0.0` and there is no stability promise before 0.x.**

## Status

**Implementation-complete**: `data`, `layout`, `layout::text`, `theme` with its fourteen schemes,
`keys`, `ctx`, `id`, `route`, the hit index, `focus`, `sizing`, `work`, `anim`, `overlay` and
`scroll`. The layering is built rather than asserted: the component-facing surface is checked by a
crate that cannot name the engine, so a `use vitui_engine::…` there is an `E0432` and the boundary
is cargo's rather than a reviewer's. The verification register and the scene list are values a test
iterates, and **every instrument in them names a file a test opens**, which is what makes them
registers rather than documents.

**This crate is `#![forbid(unsafe_code)]`, and so are `vitui-components` and the `vitui` facade.**
`overlay` used to carry the only `unsafe` here — a crate-private bump region holding an overlay body
with its type erased — and it is gone: a body is one `Box` in a queue the frame call owns, so a
frame with **n** overlays standing costs **n + 1** allocations and a frame with none costs nothing
at all. That marginal count is gated.

The version is `0.0.0` and there is no stability promise before 0.x, so depend on this today only
if you are willing to follow a surface that may still move.

## The shape, because it is unusual

**There is no scene tree and nothing is retained.** The clip stack is the call stack, the id path is
the closure tree, and what survives one draw is five flat structures rebuilt from the next draw
and nothing else.

`Ctx<'f, 'v>` carries **two** lifetimes deliberately. With one, `child()` shrinks it and an overlay
body capturing a base-pass local compiles — which deletes the mechanism overlays rest on.

A frame consumes at most one routing edge, and there are no per-id inboxes: an event goes to one
place, and where that is can be read off the draw.

## Dependencies

`vitui-engine`, and nothing else. Not "few" — none.

## Documentation

`cargo doc -p vitui-runtime --open`.

## Licence

MIT OR Apache-2.0, at your option.
