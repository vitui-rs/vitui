---
status: accepted
date: 2026-08-17
---

# Intent is never dropped; position is

Consecutive `Mouse` events of kind `Move` carrying the same held buttons collapse into the latest one.
Every other input event — key press, repeat, release, button down, button up, wheel, resize, focus,
paste — is delivered, however far behind the application has fallen.

Ticket 09 stated the rule as *frames may be dropped, input may not*. This narrows it: **no event
carrying intent is lost; an intermediate pointer position is not intent.**

## Why

With motion tracking enabled the terminal emits an event per cell the pointer crosses. An application
having a slow frame accumulates a queue in which only the last entry means anything — the earlier
positions describe where the pointer *was*, and nothing acts on that.

Keeping them has a cost with no ceiling. The queue must be able to grow, because ticket 09 refused
backpressure and this ticket refuses it again for a sharper reason: the read direction of the tty also
carries capability replies, so a serializer that stops reading can deadlock itself. An unbounded queue
fed by a source the user can drive at will — waving the mouse — is the one path by which a terminal can
consume an application's memory.

Dropping them has a cost that is bounded and, for the intended use, invisible. What consumes mouse
positions is hover and drag, and both are answered by "where is the pointer now". A hover highlight
that would have lasted zero frames cannot be seen, because the frame clock (ADR 0004) holds events to
the gap anyway — 8.3 ms at 120 Hz.

The alternative placements were considered and are worse. Coalescing in the runtime leaves the engine's
queue growing, which is the actual hazard. Making it an application choice offers a knob whose wrong
setting is an out-of-memory condition.

## Consequences

**A pointer path has gaps, and anything that integrates over the path is wrong by construction.**
Counting how many items the pointer passed over, drawing a freehand stroke, or measuring pointer
distance cannot be built on this event stream. Freehand drawing in a terminal is not a use this engine
is for; the constraint is recorded so that it is met as a design fact rather than as a bug report.

**Enter/leave can skip a widget entirely.** The runtime computes them by diffing the hit test between
frames, so a pointer crossing a button between two frames produces neither. This follows from the
frame clock as much as from coalescing, and it is the same invisibility argument.

**The rule is stated in terms of intent, which makes it checkable.** "Input is never dropped" cannot
survive contact with a motion stream; "no event carrying intent is lost" can be held exactly, and each
variant can be classified once. A future event type is classified by asking whether an application
would act on it, not on how frequent it is.

**Growth from other sources is left alone.** A terminal without bracketed paste delivers a pasted
megabyte as key events; that queue is bounded by the paste and drains itself. Unbounded growth from
anything else means a chronically slow application, which is ticket 18's subject and already has a
detector — it is a symptom, not the disease.
