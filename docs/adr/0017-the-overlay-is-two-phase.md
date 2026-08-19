---
status: accepted
date: 2026-08-19
---

# The overlay is two-phase, and its owner id is handed over

A component cannot open a layer mid-draw: `LayerStack::view` holds `&mut` of the stack for the life
of the view, so the second call is `E0499` (engine ticket 14 R2). Overlays are therefore
**requested during the draw and satisfied after it**, with the outcome reaching the owner on the
frame after.

The request verb takes the owner's identity as an argument:

```rust
cx.overlay(owner_id, anchor, OverlayOpts { z, placement, .. }, |cx| { … });
```

## Why

The `E0499` is inherited and not re-argued. What is decided here is the two sentences
`architecture.md` §7 spent least space on, and each of them decides whether the protocol fits at all.

**The owner id cannot be derived.** Components carry `#[track_caller]` and the attribute propagates
*into* the body, so an id derived inside `select` is `select`'s caller's location — which is the id
`select` already claimed one line earlier. The collision policy from
[ADR 0013](0013-identity-comes-from-the-call-site.md) then makes the second claimant inert and the
overlay is silently dropped.

**The `'f` bound is not a bound with one lifetime.** `Ctx<'f>` lets `child()` shrink the bound, so a
body capturing a base-pass local compiles — and that local is gone by the time the body runs.
`Ctx<'f, 'v>` separates the frame's lifetime from the borrow's, and the pair is a compile outcome in
both directions. The driver's signature decides it: `impl FnOnce(&mut Ctx)` forces every body to
`'static`, while `&'f mut self` makes it "outlives this frame call".

The owner id also **roots the overlay's own id stack**. Rooted at `ROOT_ID` instead, a body drawn
inline and the same body drawn in an overlay produce the same four ids. An overlay is a different
*place* for identity, not only for geometry.

## Consequences

**`'f` is viral into any component that opens an overlay.** It costs nothing anywhere else — the
change landed with no component signature altered — and this is the price, stated rather than
discovered.

**A body may not capture a base-pass local; it may capture application data and `Copy` state.** The
compiler says so at the call site, in one error. "A body may not capture `&mut` state" is a
*convention* rather than a bound — it compiles and runs — and the reason to keep it is that an
outcome delivered through the queue keeps a `select` mutating its state in one place.

**The frame arena allocates nothing in a steady state**: one chunk, 32 bytes at high water, zero
allocations across a hundred frames with a dropdown standing. It is reset rather than freed and drops
nothing itself, so a body that owns anything is dropped by a thunk the request carries beside it.

**Layer lifecycle is keyed by the owner id, and the census is not the id sweep.** An owner with a
closed dropdown is still drawing, so the two mechanisms cannot share a pass. "No reallocation" is
true of a move (0) and false of a resize (+1 surface).

**A nested overlay's z counts from its parent's layer**, not from its own band, or a dropdown inside
a modal draws below the barrier that exists to protect it.

**A modal made the frame budget visible, and the fix is an obligation on everything above.** Built
naively a modal costs **116.62 µs — 117% of the budget** — and the whole difference is the scrim at
3.41 ns a cell over 24 000. Two things bring it to **1.19 µs**: the base pass must draw into **its own
layer** with a damage-driven composite (the engine's own model, which the prototype had shortcut),
**and** every component must draw its text before its padding, because a component that fills its
rectangle and then draws into it re-damages what it drew, every frame, for identical output —
**10 814 of 24 000 cells against 0**. With either missing, 87 µs or 117 µs.

Two hazards ride along: `Mix` clears the extended-style bit, so a shadow crossing a hyperlink deletes
it (engine ticket 16, confirmed against the engine's own code); and the scrim's direction assumes a
dark theme, which is why `Theme::is_dark` has a reader.

The second pass runs in rounds while bodies request more overlays, bounded at 16. A body that
requests itself for ever is a recorded limit, not a hang.
