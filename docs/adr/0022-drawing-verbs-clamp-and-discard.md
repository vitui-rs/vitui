---
status: accepted
date: 2026-08-19
---

# Drawing verbs clamp and discard

A drawing verb never returns a `Result` and never panics. A write that falls outside the surface or
outside the clip region is **discarded silently** — no error, no `debug_assert`, `Written::NONE`. A
write that falls partly outside is **clamped**: it draws what fits and reports where it stopped. A
write that would leave half a double-width glyph writes a space instead.

Invalid UTF-8 is not handled either, because the verbs take `&str` and the error class therefore
cannot be constructed.

A caller that needs to know where a verb stopped reads the `Written` it returns — `cells`, `bytes`
consumed, and a `Stop` of `Complete`, `Clipped` or `Offscreen`.

## Why

**An out-of-bounds write is normal traffic, not an error.** A virtualised component draws against
`View::visible_rows()`, but any component that scrolls, truncates or nests writes past its own edge as
a matter of course, and the map's central invariant depends on that being cheap: a discarded write
costs 3.6 ns against 540 ns for an accepted one. Making it fallible would put a branch and an unused
`Result` at every one of the 24 000 write sites a full-screen frame makes, in exchange for telling the
caller something it usually already knows.

Every comparable API does one of the two things this one refuses. libvaxis discards silently and is
the independent convergence; the rest either panic on out-of-bounds or return a fallible write. That
is what makes the rule surprising enough to record rather than leave implicit.

**Taking `&str` rather than `&[u8]` deletes an error class instead of handling it.** There is no
"invalid UTF-8" path in the engine's drawing half, because the type system removed the input that
would reach it.

## Consequences

The rule is uniform and there is no second rule to remember: out of bounds, past the clip, mid-glyph,
a wide glyph that does not fit, a zero-area rectangle, a mouse mode a terminal does not support — all
of them clamp or discard.

`Written` is the escape hatch, and it is free to a caller that ignores it. `bytes` is on it
deliberately: a caller that wraps or paginates would otherwise segment the string a second time to
work out how far the verb consumed.

**The asymmetry with the input boundary is deliberate and is stated rather than discovered.** At the
drawing verbs the engine may demand well-formed input from its caller; at the input boundary it may
demand nothing, because a paste is composed by the world and not by our API. `Paste` therefore owns
bytes and offers a lossy `text()`, which looks inconsistent with this decision and is not: the caller
of a verb is a program, and the source of an event is a person with a clipboard.

What this costs: a component with a genuine bug — writing to the wrong coordinates — gets no
diagnostic from the engine, and its cells simply do not appear. The golden-frame tests are where that
shows up, which is one of the reasons they exist.
