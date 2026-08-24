# vitui-runtime

Layout, identity, focus, hit-testing, routing, key maps and theming on top of
[`vitui-engine`](../vitui-engine).

**Nothing is published. The version is `0.0.0` and there is no stability promise before 0.x.**

## Status: 16 of 20 tickets, and this crate is under construction

`data`, `layout`, `layout::text`, `theme` with its standard set, `keys`, `ctx`, `id`, `route`, the
hit index, `focus`, `sizing`, `work`, `anim`, `overlay` and `scroll` exist. **The crate line, the
facade's signals, and the verification ledger do not.** Nothing above the engine can draw a full
screen yet.

**This crate is `#![forbid(unsafe_code)]`, and so are `vitui-components` and the `vitui` facade**
(`docs/adr/0034`). `overlay` used to carry the crate's only `unsafe` — a crate-private bump region
holding an overlay body with its type erased — and it is gone: a body is one `Box` in a queue the
frame call owns, which costs a frame with **n** overlays standing **n + 1** allocations and a frame
with none nothing at all. Spec §19 carries the exception; the marginal count is the gate.

Depend on this today only if you intend to follow its development.

## The shape, because it is unusual

**There is no scene tree and nothing is retained.** The clip stack is the call stack, the id path is
the closure tree, and what survives one draw is five flat structures rebuilt from the next draw
(`docs/adr/0012`).

`Ctx<'f, 'v>` carries **two** lifetimes deliberately. With one, `child()` shrinks it and an overlay
body capturing a base-pass local compiles — which deletes the mechanism overlays rest on.

A frame consumes at most one routing edge, and there are no per-id inboxes (`docs/adr/0016`).

## Dependencies

`vitui-engine`, and nothing else. Not "few" — none.

## Documentation

`cargo doc -p vitui-runtime --open`, and the repository's [`README.md`](../../README.md).

## Licence

MIT OR Apache-2.0, at your option.
