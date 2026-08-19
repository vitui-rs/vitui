---
status: accepted
date: 2026-08-19
---

# The runtime reads a capability and never rewrites a declaration

ADR 0007 says the engine tells the truth about the terminal and never synthesises. The runtime is
where inference is legitimate — it is above the honest layer — and it draws one line inside that
permission:

**The runtime resolves what a capability *means*, and never edits what a component or an application
*declared*.**

Two mechanisms implement it, on two different capabilities, and they arrived one ticket apart on the
same shape.

- `Theme::resolve(tier)` folds the detected colour tier into the theme once, at 4.16 ns. A component
  then reads `theme.hover_interest()` and declares what it declares.
- `Caps::key_tier` sits beside `truecolor`. Key matching **never consults it**.

## Why

The temptation in both cases is to be helpful, and in both cases being helpful is silently wrong.

**Colour.** Quantisation collapses role *pairs*: `Face`/`FaceHover` land on the same index at 256
colours for the stub palette. A screen that declares hover interest raises the whole frame to
`Motion` tracking — every pointer move a wakeup and a whole 32.96 µs frame — for a highlight nobody
can see. The runtime could quietly strip `HOVER` from the declaration. It does not: it resolves the
theme, the component reads it, and a component that ignores the theme is **visibly** wrong rather
than invisibly corrected. Resolving drops the dense screen from tracking 3 to 2 with the same 312
regions and nothing written by the author.

**Keys.** A chord names either a base-layout position or a printed character, and which one is part
of the chord. Below kitty flag 4 the engine's `code` is *inferred from `text`* and reports no bit
saying so, so on a Cyrillic layout **33 of 33** bindings are reachable at flag 4, **28** on a terminal
that falls back to Latin and **14** on one that does not — with **0 wrong at every tier**. The
failure is silence, not misfire. The runtime could rewrite a binding when the tier is poor. It does
not, for the same reason: a rewritten binding fires *something*, and something is worse than nothing
when the author cannot see which.

The two capabilities are also the only two mechanisms on this map that remove wakeups by *reading*
rather than guessing. A 300 ms fade costs 19 frames at every tier ungated and **19 / 2 / 2** gated,
because `Face → FaceHover` has 30 / 1 / 1 distinct values on the wire.

## Consequences

**Silence is the failure mode, and it is asymmetric.** A dead binding and a user who did not press
the key are indistinguishable from inside the process: which legacy terminal we are in is
unobservable, both cases being silence on the wire. So the detector cannot be a runtime check — it is
a **count per (layout, tier)** in the test suite, and a US-layout test suite finds none of it.

**The honest help surface is the tier, not a per-binding flag.** No query in any protocol answers what
key K would print, so a help bar shows `Ctrl+S` for a keycap reading `Ы` and no tier fixes it before
the first press. Whether a component says so at all is the components map's.

**A pair count taken at sixteen colours is a lower bound, not a measurement.** Nothing in the process
can read the operator palette, so the corpus mean moves from 10.87 to 16.36 collapsed pairs once ten
real terminal profiles are applied — five of the ten spell an index twice. The runtime states the
bound and does not pretend to the number.

**A pair can be half animatable and no role list can say so.** `Color::DEFAULT` has no value to fade
towards (ADR 0007 arriving in the colour axis), so `Body → Selection` takes 2 background values across
a whole fade and 32 foreground ones. And showing a hover state is a *different* capability from
showing the animation into it: 338 themes can do the first at C256 and 165 can do the second.

If the engine ever wants to close the key-tier gap, the honest shape is a bit on the key rather than a
tier on the side — but that is engine ticket 10's to reopen with a new argument, not the runtime's to
take over its head.
