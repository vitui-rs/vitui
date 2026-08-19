---
status: accepted
date: 2026-08-19
---

# Identity comes from the call site, and an `Id` may never be persisted

A widget's identity is derived from where it is written:

```
Id = fnv(parent_id, file_ptr, line, col)
```

`#[track_caller]` on `Ctx::id`, `Ctx::interact` and every interactive component supplies the caller's
`&'static Location`. A loop adds a key — `cx.with_key(row.id, …)` — because one call site produces N
widgets. `named()` is a `const` caller-level escape and is available, not mandatory.

**An `Id` differs between two runs of the same binary and may never be persisted.** The file pointer
is an address. Nothing may write one to disk, send one over a wire, or compare one against a stored
value.

## Why

Six mechanisms need a name that survives between frames — focus, hit-testing, interest declaration,
overlay ownership, scroll association, and originally memo keys — and immediate mode supplies no
mount to hang one on. The alternatives were each refused with a reason: a slot index breaks the
moment a conditional widget appears above it, which is the classic React-key bug and is worse here
because it silently transfers focus and scroll between unrelated widgets; a mandatory user-supplied
string taxes every call site.

The call site is free at runtime and was checked rather than assumed: **0 collisions in 2 880 000
enumerated ids**. A splitmix64 finalizer was built on a sound argument and **measured backwards** —
1.000 against 1.370 probes per insert, because line and column are dense and FNV maps them
injectively.

The persistence rule is not a style preference. It was found by asking whether the `&'static str`
file pointer is stable, which runtime ticket 01 had asserted; it is stable *within* a run and is an
address *between* them.

## Consequences

**Duplicate detection is part of the design, not a debug aid.** The collision that actually happens
is not a hash collision — it is a *merge*, at probability 1, when two widgets legitimately share a
call site. The policy is first-claimant-wins with the second inert, which is why an overlay's owner
id is handed over rather than derived ([ADR 0017](0017-the-overlay-is-two-phase.md)). Detection is a
stamped open-addressed table at **1.1 ns a widget**, growing **3.95×** for 4× the widgets; the
`Vec::contains` it replaced was 22.5 ns and **13.20×**.

**A container's shape is readable from its signature.** A container that returns a rectangle
preserves its children's identity; one that takes a closure renames them. There is exactly one
exception and it is load-bearing: `Ctx::scope` takes a closure and must **not** rename, or the frame
a modal opens renames every field of the form it traps. `scroll_scope` is the same.

**A `#[track_caller]` wrapper merges the widgets inside its own body**, because the attribute is
viral and silent. A component library that wraps components in helpers must know this; it is visible
only because duplicate detection names it.

Three cross-frame facts survive a widget that stops drawing — grab, press origin, focus — and a
**281 ns** sweep releases them. Without it a stale grab swallows the pointer for every widget still
on screen.

A debug inspector prints the id path so a legitimate identity change — a container gained above a
widget — can be seen rather than guessed. It is the closure tree, not the draw tree.
