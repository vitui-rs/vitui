# vitui-runtime

Layout, identity, focus, hit-testing, routing, key maps and theming on top of
[`vitui-engine`](https://docs.rs/vitui-engine).

**Nothing is published. The version is `0.0.0` and there is no stability promise before 0.x.**

## Status

**Implementation-complete** and `#![forbid(unsafe_code)]`: `data`, `layout`, `layout::text`, `theme`
with its fourteen schemes, `keys`, `ctx`, `id`, `route`, the hit index, `focus`, `sizing`, `work`,
`anim`, `overlay` and `scroll`. The layering is built rather than asserted — the component-facing
surface is checked by a crate that cannot name the engine, so a `use vitui_engine::…` there is an
`E0432` and the boundary is cargo's rather than a reviewer's.
[`docs/status.md`](https://github.com/vitui-rs/vitui/blob/master/docs/status.md) has the registers.

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

Licensed under the Apache License, Version 2.0. See [LICENSE](https://github.com/vitui-rs/vitui/blob/master/LICENSE).
