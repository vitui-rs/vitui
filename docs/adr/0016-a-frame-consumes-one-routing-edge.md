---
status: accepted
date: 2026-08-19
---

# A frame consumes at most one routing edge

A frame routes against **one** routing state. Events that change nothing routing reads — pointer
moves, wheel clicks, ordinary keys — fold freely into a single frame at 5.0 ns each. An event whose
effect the frame *reads* is a **routing edge**, and a frame consumes at most one of them.

Which end of the batch an edge may sit at is decided by **when its effect is read**, not by what kind
of event it is. An edge resolved after the draw **closes** a batch; an edge resolved before the draw
would **open** one.

## Why

"One event per frame" is the rule an immediate-mode loop is usually written with, and it is wrong in
both directions. It drops throughput on input that costs nothing to fold — an 8 000-key paste becomes
8 000 frames — and it does not actually protect the case that needs protecting, because two edges in
one batch are misrouted whether or not the rest of the batch was split off with them.

ADR 0008 already says which events may collapse. What was missing is the classification of the rest,
and it is not intrinsic to the event: `Down`/`Up` resolve in `end`, after the draw, so every event
before them in the batch was routed against the state the frame actually drew with, and they may
therefore be the **last** event a frame consumes. `Tab` resolved in `begin` would be the mirror image
and may be only the **first**. Getting that backwards still misroutes `[Key(a), Tab]` — the key goes
to the widget the `Tab` is about to move to.

Measured with the rule applied: sixteen edges, sixteen frames, **eight clicks, none lost**.

**The defect was invisible for five tickets because at `Motion` tracking the unsplit batch works.**
Escaping `Motion` — which the theme decides, by declaring whether a hover state is visible at all —
is what makes it reachable.

## Consequences

**Every routing edge is now a closing edge.** Runtime ticket 08 moved the `Tab` resolution out of
`begin` and into `end`, by declaring the focus ring during the draw at 1.000×–1.009× of the frame —
its own noise floor. So `[Key(a), Tab]` is one frame, and nothing about focus crosses a frame.

The rule is kept even though only one of its two cases is now reachable, because the rule is what
makes the classification checkable, and the negative case is a gate. A future edge resolved before
the draw is expressible; it is not expressible by accident.

**There are no per-id inboxes.** Bubbling, accelerators and "unhandled reaches the application" are
one queue with one byte per key, drained at successively outer levels. The literal per-id version
costs 1.17× routing and at least one allocation a frame against zero, and its pointer targets come
from last frame's index — the stale answer [ADR 0015](0015-no-geometry-crosses-a-frame.md) exists to
remove.

**Bubbling is not a walk of the id path.** A container draws before its children, so a pull API gives
capture; the only moment an ancestor can ask *after* them is after its body, which only a
closure-taking container has. `Ctx::scope` supplies that moment, and a bubbling container adds
exactly one id — its own. Bubbling costs 2.99 ns a key a level, over an id path that is 1 at the top
of a realistic screen and 2 inside a keyed row.

**Draining is where quadratics hide.** `next_key` written the obvious way is O(n²): an 8 000-key
paste is 11.75 ms against 53.79 µs, fixed with one `usize`. It is a slope, not a cliff, so the
100 µs budget gate does not catch it and the growth-ratio gate does.

Budget: routing a realistic batch on a realistic screen is **221 ns, 0.221%**.
