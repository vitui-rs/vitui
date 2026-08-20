---
status: accepted
date: 2026-08-19
---

# Identity is taken outside every closure

A component takes its `Id` **before** it enters any closure, and a container roots its children
inside its own id. In practice: a component that opens a scope, a group or an overlay computes the
id at its own call site and registers with `interact_id`; a collection wraps its row loop in
`cx.with_id(id, …)`; a loop over call sites uses `cx.with_key`.

This is ADR 0013's rule (identity comes from the call site) at the place where the call site stops
being visible.

## Why

**`#[track_caller]` does not cross a closure.** Inside a closure, `Location::caller()` is the line
*in the function that invokes the closure* — so every list in an application derives the same id and
one of them is inert. Components ticket 02 shipped this bug and its own merge counter found it: 266
regions and 1 colliding site against 267 and 0. Neither function was missing the attribute.

**The runtime's `scope` pushes no identity, deliberately**, so a collection that draws rows through a
line inside the component leaves those rows' ids derived from `(screen, key, that line)` — identical
in every collection on the screen. 72 of 160 targets inert; **389 regions and 0 merges against 301
and 72** once the row loop is wrapped. Reproduced with two tables (284 of 911 targets inert) and two
trees (40 merges).

**The failure mode is what makes this an ADR: the screen renders pixel for pixel correctly.** Drawing
does not consume an id. The first screen written for components ticket 01 had **110 of its 338
widgets inert** — six loops with no key, first claimant wins — and a pointer on the thirteenth
toolbar chip lit nothing. Nothing about the picture says so.

**The detector is free and already computed.** Duplicate detection runs every frame at 24.1 ns a
widget, so `merges == 0` costs nothing to assert and caught every instance above, including the two
written after the rule was recorded.

**An `Id` is opaque and cannot be inspected.** It is a hash, so nothing recovers from the value which
scope or owner it was rooted at. Three independent sightings — colliding rows, a refused focus stash,
a submenu sharing its parent's owner — and every workaround on the components map that looks like a
hack is that fact.

## Consequences

**A container's signature is affected.** A component that opens a scope cannot use `interact_named`
inside it — that claims a second time and collides with `cx.id()`. The id is computed outside and
passed in.

**A component with two overlays standing must mint a second id**, and `with_key` is the only verb
that does. Sharing one gets a single layer slot resized: 8 allocations and 564 cells a frame.

**One axis out, the arithmetic differs.** Keying a table per row rather than per cell is cheaper and
correct — until a cell declares a target of its own, and then it is 142 merges. The gate catches it
unchanged.

**A wrapper that forgets `#[track_caller]` merges the widgets inside its own body.** That rule
survives as a special case of this one rather than as a rule of its own.
