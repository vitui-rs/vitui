---
status: accepted
date: 2026-08-17
---

# The engine degrades presentation, never content

Colour and text attributes are degraded silently, at serialisation, and no caller is told. A glyph is
never touched: on a terminal or in a configuration where a cluster cannot be shown, the engine emits
it unchanged and the component decides — from `Capabilities` — to draw something else instead.

The line is content against presentation. A glyph *is* what was drawn; a colour is how it looks.

## Why

The question "where does degradation happen — at draw time or at serialise time?" was the ticket's
declared central decision, and it arrived already answered twice, in opposite directions.

Colour was settled at serialise time by three tickets. Ticket 04 packs a two-bit colour tag into the
`u64` style and resolves the downgrade when the SGR is emitted, so nothing in a cell changes per tier.
Ticket 08 built that. Ticket 06 added the condition that after a `Mix` the colours are already RGB and
the `default` and `indexed` tags are gone, so the downgrade is a quantisation rather than a tag being
passed through.

Glyphs were settled in the other direction by ticket 14. Braille carries 256 states per cell, block
elements carry 8 and are anchored to the bottom, ASCII carries 2. A line chart is not *expressible* in
block elements — the component has to become an area chart — and it is expressible in nothing at all
under ASCII. An engine that quietly replaced `U+28xx` with `?` would turn a chart into noise while
every performance budget stayed green.

Reading those two as a gap to be closed is the mistake. They are one rule. Colour has a
meaning-preserving fallback and a glyph does not, because the character carries the information
itself. The same rule already governs input, where ADR 0007 refuses to synthesise a key release: an
event is content, and an invented one is a statement about the user that is false.

There is a mechanism under the rule, and it is not a coincidence. Colour is *detected* — XTGETTCAP
`RGB`, DA, OSC 11, OSC 4. Glyph repertoire **cannot be detected at all**: no query asks whether a font
contains `U+28FF`, and the terminal will accept braille and draw tofu without a word. So the axis the
engine may act on is the one it can measure, and the axis it must hand upward is the one it cannot.
See ADR 0010 for what that does to the shape of the types.

## Consequences

**Colour quantisation runs on the render thread, inside the run scan, before the comparison with the
mirror.** It is wire-specific, and everything wire-specific lives below ticket 09's `Packet`. This
also makes ADR 0006's equality filter exact with respect to the wire: a mirror holding colours the
terminal was never sent would compare unequal on two values that quantise to the same index, and
re-emit cells for no visible change.

**Contrast-preserving quantisation is not available**, and the reason is mechanical rather than
aesthetic. It would make two cells with an identical `u64` style require different SGR, so a style run
would break — and run coalescing is what already collected the wire win. At 16 colours a shadow over a
similar background therefore disappears, which is a case handed upward under this ADR rather than
patched under it.

**Unsupported text attributes are dropped rather than reported.** They are not detectable either —
DECRQM does not cover SGR, and ticket 01 narrowed XTGETTCAP to `RGB` precisely to avoid terminfo
descriptions — so they live in the per-terminal quirk table and never reach `Capabilities`. Nothing
above would act on them: an absent attribute still draws the correct text.

**A degraded lifting is a different layer stack, not a different parameter.** At `ColorDepth::None` an
operator layer's `Mix` provably does nothing and compositing skips it, worth 78.2 µs of ticket 06's
107 µs worst screen. A character shadow that replaces it is a *content* layer — its own rectangle, its
own `z`, occluding rather than tinting. The branch is the component's, and it is the same shape as the
chart's.

**Choosing a glyph before drawing is legitimate; replacing one after it is drawn is forbidden.** A
border character set handed to a component according to `Capabilities::glyphs` is a choice at the
source. The same table applied by the engine to cells a component already wrote is the thing this ADR
exists to prevent. The prohibition costs no discipline in practice: ticket 05 left the engine three
verbs and no box-drawing primitive, so it has no glyphs of its own to substitute.
