---
status: accepted
date: 2026-08-19
---

# Frame state is rebuilt from the draw

`vitui-runtime` retains no tree and diffs nothing. What it keeps for the length of one frame is
**five flat structures, rebuilt by `Frame::begin` from the draw that follows it**: the hit index,
the focus ring, the overlay request queue, the deadline sink and the key queue. Four id-keyed facts
deliberately outlive the frame, because they are the ones a widget cannot re-declare by drawing:
the pointer grab, the press origin and the focus — released by a sweep when their widget stops
drawing — and the click record, which is not swept.

Every comparable library retains something: a widget tree, a render object graph, a virtual DOM to
diff against. The absence is surprising enough to record, and the consequence below is surprising
even to someone who accepts the absence.

## Why

Four independent answers each removed one candidate for a tree, and none of them was looking for
this decision when it found it. The clip stack **is** the call stack, so the draw tree is never a
value (engine ticket 05). The id path is the **closure** tree, not the draw tree, so a
rectangle-returning split leaves its panes siblings at one depth (runtime tickets 02 × 03). The
layer stack is a sorted `Vec`, and a window inside a window is two entries with different z-order
(engine ticket 06). Intrinsic sizing takes no measure walk, so nothing needs a node to hang a cached
size on (runtime ticket 04).

Rebuilding is also cheaper than the alternative on the numbers that matter: the whole of a realistic
frame's runtime work — 312 hit entries, 67 ring entries, 43 tab stops, routing, id hashing and
duplicate detection — is inside a 33 µs frame against a 100 µs budget, and the structures are
swapped rather than allocated, so a steady frame allocates zero.

## Consequences

**A widget that did not draw cannot be clicked, focused or hovered, even though its cells still look
correct.** This is the whole of the decision and it is silent for exactly one frame, because routing
uses the previous frame's index. Runtime ticket 13 measured it: a frame that redraws one region
declares **1 hit entry against 312 and 0 tab stops against 43** while **23 993 of 24 000 cells stay
correct**.

That is why fine-grained reactivity above this runtime is not a performance question but a
correctness one. A signal graph that redraws only what changed is not slow here — it is asking to
revert the hit index. What a reactivity layer may skip is what `Memo` skips: the *computation*, never
the *declaration*. See [ADR 0020](0020-reactivity-lives-above-the-runtime.md).

The four surviving facts are a closed list, and the argument for adding a fifth has been made and
refused once already: a pending background job is commissioned work whose lifetime is the question's,
not interaction state, and the registry that would make it a fifth fact has no correct setting
(runtime ticket 18).

`Frame::begin` is therefore not skippable. `IdTable` is stamped rather than cleared, so a frame that
has never begun carries stamp 0 over slots stamped 0 and `claim` walks the ring for ever — a hang
rather than a failure. Either `begin` cannot be skipped by construction or `claim` fails rather than
spins; the load check runtime ticket 02 added is the precedent.
