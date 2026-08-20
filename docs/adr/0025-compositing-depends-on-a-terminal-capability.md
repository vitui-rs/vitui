---
status: accepted
date: 2026-08-19
---

# Compositing depends on a terminal capability

The compositor resolves colours to concrete channels before it mixes them: `indexed` through the
palette, `rgb` as itself, and `default` through **the terminal's own default background, queried once
at startup with OSC 11** (and OSC 10 for the foreground). If the terminal does not answer, cells whose
background is `default` are **left unmixed**.

This is the only place in the engine where a rendering decision depends on what the terminal told us.
Every other capability-dependent behaviour lives at serialise time, on the render thread, below the
packet.

## Why

An operator layer applies `Mix { toward, amount }`, and mixing needs channels. `default` has none: it
names whatever the user configured, which may be near-black or near-white and cannot be inferred from
anything else the terminal reports.

**Guessing is not neutral.** The natural guess is a dark theme, and on a light-theme terminal a shadow
over a default background then comes out *lighter* than its surroundings — a shadow drawn backwards.
That is a defect a user reports as "the shadows are wrong" and a developer cannot reproduce.

Refusing to mix is visible too — a shadow clipped to the explicitly-coloured region — but it is an
imperfection rather than an inversion. The rule this instance establishes, and which any future query
should be decided by:

> **Refuse when a guess would be wrong in direction; default when it would be wrong only in degree.**

The companion case is OSC 4, which queries palette entries 0–15 for quantisation at `Ansi16`. Silence
there falls back to the xterm default table, because a themed palette makes the nearest match slightly
off — wrong in degree — while an unthemed terminal would otherwise lose colour entirely. Same rule,
opposite answer, and the two are consistent rather than in tension.

## Consequences

**The startup query batch is load-bearing for rendering and not only for degradation.** OSC 10, OSC 11
and the sixteen OSC 4 queries join the batch ahead of the DA1 sentinel, so they cost no additional
round trip — but a design that dropped the capability handshake to save startup latency would silently
change what shadows look like.

The frame reaching the serializer has colours **already resolved to RGB wherever an operator touched
it**, and the `default` tag is gone from those cells. A 16- or 256-colour downgrade must therefore
quantise from RGB rather than pass the original tag through. This narrows the earlier promise that
"nothing in the cell changes per tier": nothing changes *per tier*, but compositing itself rewrites the
tag.

`Capabilities::default_fg` and `default_bg` are `Option<Rgb>` on the public surface for this reason, and
they pass the test every public field has to pass: someone above can act on them.

At `ColorDepth::None` the whole question disappears, because operator layers are skipped outright — no
colour reaches the wire, so `Mix` provably changes nothing. That is worth 78.2 µs of the 107 µs
worst-case screen, and it is the one place where a degradation tier changes compositing rather than
serialisation.
