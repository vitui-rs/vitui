---
status: accepted
date: 2026-08-17
---

# Detected axes are flat booleans; declared axes may be ordered

`Capabilities` answers the eight input questions as eight independent booleans and refuses a keyboard
tier. It answers the glyph question with an ordered three-level `GlyphSet`. Both are correct, and the
rule that separates them is who decides the axis:

- A **detected** axis is a set of independent facts about the world, and the world does not sort. It
  is exposed flat.
- A **declared** axis is a promise made by whoever declares it, and a promise can be downward-closed
  by construction. It may be exposed as a ladder.

`ColorDepth { None, Ansi16, Indexed256, TrueColor }` is detected *and* ordered, and it is the case
that proves the rule rather than breaking it: the ladder is a property of the wire formats, which
genuinely nest — a terminal that emits `38;2;r;g;b` emits `38;5;n` and `30`–`37` as well.

## Why

Ticket 10 found the negative case with evidence. `KeyboardTier::{Legacy, Disambiguate, Full}` reads
better than eight booleans and is false: tmux forwards the `CSI u` *encoding* while implementing none
of the kitty flag stack, and WezTerm ships the protocol switched off by default. Those are not points
on one scale, and an application told "Legacy" would be told something untrue about two terminals in
the tier-1 set.

Ticket 14 arrived at the same requirement from the other side — a braille chart cannot be
glyph-substituted into an ASCII one, so `Capabilities` has to be something a component *branches* on.
Two independent arrivals at "a component branches on this" was the strongest signal the degradation
ticket had about the shape.

What was missing was why the glyph axis is then allowed to be a ladder when the keyboard axis is not,
since both are branched on. The answer is that nobody detects the glyph axis. There is no query for
"is `U+28FF` in the font" — the terminal accepts braille and draws tofu in silence. `--ascii` is
therefore an operator's promise about their own font, and a promise is downward-closed because the
person making it says so. `GlyphSet { Ascii, Unicode, Extended }` has three levels because a real
component has three strategies: braille is 256 states per cell, block elements are 8 and anchored to
the bottom, ASCII is 2.

## Consequences

**Every public field on `Capabilities` must pass one test: someone above can act on it.** Synchronised
output, `DECSLRM`, legacy SGR, the ConPTY underline-colour escape form, the terminals' force-flush
limits and the 16-entry palette are all known and all private, reachable only through a `report()`
string for diagnostics. Fourteen public facts remain — six about output, ticket 10's eight about
input.

**The word "tier" does not survive on the read side.** It described a named combination of
capabilities, and after this rule no such combination is read by anyone. `Config::tier: TierRequest`
becomes `Config::overrides: Overrides`, whose every field is an `Option` that lowers or pins what
detection found. The one named combination that survives is `Overrides::plain()` — colours off, glyphs
ASCII — because `--ascii --no-color` genuinely travel together, and a constructor is the right size
for that.

**A declared default can be optimistic where a detected one cannot.** `GlyphSet` defaults to
`Extended` rather than the safer `Unicode`: requirement 6 says UTF-8 first with `--ascii` as the
degradation path, nothing can be queried, and the lever is one environment variable. A detected axis
gets no such benefit of the doubt — an unanswered query means the capability is absent.

**The precedence order is where declarations are collected, and it is stated once**: the explicit API,
then `VITUI_*` environment variables, then `NO_COLOR`, then `TERM=dumb` or a non-tty, then the
per-terminal quirk table applied *after* detection, then live detection, then conservative defaults.
The API sits above the environment, and the conflict that would otherwise create is removed by the
`Option` shape: the environment fills only fields left `None`, and never overrides a `Some`.

## Amendment, 2026-08-21 — a third category, and the write-side test

Amended by [architecture ticket 22](../../.scratch/vitui-engine-architecture/issues/22-headless-cannot-declare-a-hyperlink.md).
The decision above stands unchanged; what it did not have is a name for a fact that is neither detected
nor declared, and a test for the *write* side to match the one it gives the read side.

**A third category: inferred.** `Capabilities::hyperlinks` was written as though it were detected. **OSC
8 has no query** — none of DA1, XTVERSION, DA2, DECRQM, XTGETTCAP, OSC 10/11 or OSC 4 asks it and none
could — and it is not something an operator was ever asked to promise either, the way the font is. So it
is *inferred*: matched against an allow-list of terminal identities. The first implementation of that
inference read *answered XTVERSION* as *implements OSC 8*, which is wrong because XTVERSION is xterm's
own and xterm has no OSC 8, and the corrected form still ships a known residual — VTE implements OSC 8,
answers no XTVERSION, and gets a wrong `false`.

An inference is the engine guessing on the world's behalf, which
[ADR 0025](0025-compositing-depends-on-a-terminal-capability.md) governs. What follows is one rule:

> **An inferred axis must be declarable**, because a declaration is the only thing that can correct it.
> There is no second query to ask more carefully, and a quirk-table entry is a new release.

**The write-side test.** This ADR's consequence — *every public field on `Capabilities` must pass one
test: someone above can act on it* — decides the read side, and `Overrides` was left to be a subset of
whatever that produced. It is not one and never was: `legacy_sgr` and `width` pin **private** facts, so
the type was never *fourteen facts mirrored as fourteen `Option`s* and the stopping line was never a
count. The test is:

> **An axis belongs on `Overrides` iff (a) nothing measured it — nothing can, or nothing did and the
> engine inferred it — or (b) the engine's own output depends on it *and* the value is one the person at
> the terminal knows.**

Seven fields satisfy it: `colors`, `glyphs`, `default_fg`, `default_bg`, `hyperlinks`, `legacy_sgr`,
`width`. The eight input facts fail (b) twice, and the second time re-admits a defect this repository
already decided against: nothing on the output path reads an input fact, and *a declaration cannot make
an event arrive*, so `key_release: Some(true)` on a terminal that sends no releases would leave a
component drawing a key it believes still held — the exact failure
[ADR 0007](0007-the-input-model-is-honest-about-the-terminal.md) refused the uniform keyboard model for.
`grapheme_clusters` fails (b) because the engine's own tables stay authoritative either way, so nothing
in the engine reads it; a component reading it is not the engine.

The two tests are deliberately different questions, and the fields moving apart is the answer rather than
a drift to be corrected: **`Capabilities` is what someone above can act on; `Overrides` is what someone
below can be told.**
