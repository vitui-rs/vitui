---
status: accepted
date: 2026-08-17
---

# The input model is honest about the terminal

`KeyKind` has three variants — `Press`, `Repeat`, `Release` — and on a terminal that reports only
presses, only `Press` is ever produced. The engine does not synthesise the missing two, and an
application learns from `Capabilities` which it will actually see.

The same rule holds elsewhere in the input path: an ambiguity the terminal did not resolve is passed
up ambiguous, and a state the terminal did not report is not inferred.

## Why

The uniform model is genuinely attractive. Every application would be written once, against a rich
model, and terminals that report less would be filled in by the engine: synthesise a `Release` when
the next press of that key arrives, fold `Repeat` into `Press`. Nothing in the API would ever mention
a tier.

It fails on timestamps, and that is a placement problem rather than a matter of taste. A key was
released at *some* moment between two presses. The engine knows which second, not which millisecond,
and every event carries an `Instant` stamped when it was read. A synthesised release would either
carry a lie or carry the moment of a keystroke it has nothing to do with.

The visible consequence is worse than the internal one. A component drawing a held key — a piano roll,
a game control, a modifier indicator — shows it held until the next keystroke arrives. On an idle form
that is forever. The engine would have converted "this terminal cannot tell you about releases" into
"this key is still down", and the second is a statement about the user that is false.

Three facts make this concrete rather than hypothetical, all from the capability research:

- Without kitty keyboard flag 2, a key release **never arrives**.
- Without it, auto-repeat is **indistinguishable** from a fast series of presses.
- Even with the baseline disambiguation flag, Enter/Tab remain ambiguous with Ctrl+M/Ctrl+I. The kitty
  spec carves them out deliberately, so that `reset` stays typeable after a program crashes with the
  mode set. No amount of engine cleverness removes that ambiguity, and pretending otherwise would put
  a wrong keystroke into an application rather than an absent one.

## Consequences

**An application that wants the rich model must branch, and the branch is cheap.** `Capabilities`
carries eight separate booleans — `key_release`, `key_repeat`, `alternate_keys`, `associated_text`,
`mouse`, `mouse_motion`, `focus_events`, `bracketed_paste` — rather than a tier enum. A tier would read
better and would be false: tmux forwards the `CSI u` encoding while implementing none of the flag
stack, and WezTerm ships the protocol switched off by default. Those are not points on one scale.

**Absence is representable and is not confusable with a value.** A terminal that reports only presses
produces `Press` and nothing else. There is no `Release` with a guessed timestamp for a caller to
mistake for a real one, so a component that handles releases correctly on kitty degrades to "never
sees a release" rather than to "sees wrong ones".

**The same principle decides three smaller questions elsewhere in the input path**, and they are worth
listing so the rule is recognisable next time: a mouse-leave is not synthesised from focus loss,
because pointer position and focus are different things; double-click is not detected by the engine,
because it is policy with a tunable threshold; and an unrecognised escape sequence is dropped and
counted rather than guessed at.

**What is *not* covered by this rule is dropping.** Refusing to invent an event is different from
refusing to discard one, and the engine does discard — see ADR 0008. The line is that the engine may
tell an application less than happened, never something that did not.
