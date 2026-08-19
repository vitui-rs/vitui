---
status: accepted
date: 2026-08-19
---

# A component names a role and can never construct a paint

Every drawing verb on `Ctx` takes a `Paint`, and the only way to obtain one is to ask a `Theme` for
a `Role`. `Paint` is a newtype over the engine's `Style` whose field is private to the theme module;
an application constructs a theme from **sixteen colours** (`Roles::from_palette`) and the pairing
into thirteen paints happens inside the crate.

**A role names a paint, not a colour.**

## Why

"No literals" started as a lint and could not survive as one. A component handed `surface` and
`on_surface` as *colours* must pair them, and pairing is `Style::pack` — the literal the same rule
forbids. Making the pairing the theme's job is what makes the rule enforceable, and making `Paint` a
type is what makes it enforced by the compiler rather than by review.

The cost was measured before it was chosen: **1.002×** and **8 bytes against 8**, with **zero**
component signature changes across all 56 gates that existed at the time.

Four compile outcomes hold the door shut, and the one that would have been missed is `restyle`: six
verbs take a style as a parameter and are closed by their signature, while the seventh takes a
*function over styles* and can return one it invented.

The door has been re-opened once already, by a later ticket and not by an application:
`Theme::imported(&'static [Style; 13], …)` plus a public `Style::pack` plus `Box::leak` is three lines
to an arbitrary `Paint`, and it compiled from outside the crate. That is why the constructor is
`Roles` — sixteen colours in — and why the negative case is gated in both directions rather than
asserted once.

## Consequences

**An application can ship its own theme and still cannot invent a paint.** That is the whole point of
the `Roles` shape: the authored surface is a palette, and the mapping onto roles is code with a gate
on it.

**Degradation is a pair property, and the component reads it.** Quantisation happens at serialise
time and no engine change is proposed — but it collapses role *pairs*: 1 / 2 / 13 of 78 pairs are
indistinguishable at True / C256 / C16 for the stub palette, and over 338 real schemes the shipped
mapping gives 0.08 / 0.26 / 10.87. `Theme::resolve(tier)` folds the tier in once, at 4.16 ns, so a
component reading `theme.hover_interest()` on a 256-colour terminal stops asking for `Motion`
tracking — every pointer move a wakeup and a whole 32.96 µs frame — for a highlight nobody can see.

**The runtime never edits a declared interest.** It resolves the theme and the component reads it, so
a component that ignores the theme is *visibly* wrong rather than invisibly corrected. That is
ADR 0007's rule one layer up, and [ADR 0021](0021-the-runtime-reads-a-capability-and-never-rewrites-a-declaration.md)
generalises it.

**A colour tween is a verb on the theme.** `impl Lerp for Paint` and `Paint::derive` both fail
(`E0423`, `E0624`), so interpolation is `Theme::mix(a, b, t)` at 7.58 ns, and a component still names
two roles and never a colour.

**A theme is heap-free**, because the obvious `Vec<Style>` palette allocates on the frame a swap is
visible: 0 allocations across two swaps and two full frames, a swap at 14.30 ns, and an imported
palette in `.rodata` at 104 B.

**One cost is real and is not recoverable:** the role a cell was painted with cannot be read back
from the cell, because a `Paint` *is* a `Style` and the role is gone by then. A debug inspector that
wants roles needs the theme to record them, which is a cost on every verb.

Found by accident and undetected for nine tickets: `Dim` and `Border` were one paint under two names.
The pair count is what found it, and it is a gate.
