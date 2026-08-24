# vitui-signals

A fine-grained signal graph for [`vitui`](../vitui) — **deliberately outside the facade**.

`vitui` re-exports the engine, the runtime and the components. It does not re-export this crate, and
that is a decision rather than an omission: a facade that re-exported one of two reactivity shapes
would have picked a winner between two that measured identical. Add it to your own `Cargo.toml`,
which is the visibility the choice deserves.

It is also a **detached workspace**, which is that same decision taken literally: `deny.toml` bans
the crate with an empty `wrappers` list, and `cargo deny` bans a crate's *presence in the graph*
rather than only its dependents — so a workspace member with nothing depending on it fails the gate
outright. Detaching is the only shape in which the rule is a sentence a tool can hold, and it makes
the ban fire the day somebody writes the dependency instead of already failing. Its own manifest
carries the whole finding.

```toml
[dependencies]
vitui = "0.0.0"
vitui-signals = "0.0.0"
```

## What it is

About a hundred and twenty lines of code, and the reason it is that short is the point: **two of its
three parts are already in `vitui-runtime`.** `Revision` is the process-global sequence, `Versioned`
is the bump-on-drop guard, `Memo` is the cache. What this crate adds is shared ownership and a dirty
flag — `Signal<T>`, `Graph`, and `Computed<T>`, which is `Memo` with the key filled in.

## What it cannot do, and it is not a limitation of this crate

> **Re-running the view is the propagation** — not as a convenience of immediate mode, but because a
> frame that draws less than the whole screen *declares* less than the whole screen.

A subscription that redraws one region declares **1 hit entry against 312 and 0 tab stops against
43**, while every cell on the screen stays correct — and the loss is silent for exactly one frame,
because routing answers from the previous frame's index. `Frame::begin` rebuilds the hit index, the
focus ring, the overlay queue and the deadline sink from the draw.

**Fine-grained reactivity here is not expensive; it is a request to revert the hit index.**

So this crate is ergonomics over one runtime hook and a cache over another. It is worth having, and
it is not a mechanism. See `tests/declaration.rs`, which keeps the stored-closure shape that
*compiles and passes* beside the count that refuses it.

## Status

Runtime ticket 18. `vitui` itself is `0.0.0` and unpublished; nothing here is stable.
