---
status: accepted
date: 2026-08-19
---

# Reactivity lives above the runtime

`vitui-runtime` ships the frame loop and four hooks, and no reactivity model:

1. `request_frame()`
2. `Wake::Posted` + `Slot<T>` + `Task<T>`
3. deadlines
4. `Memo`

A TEA pump is a `while` loop over `Ctx` and needs no crate at all. A signal graph is an application's
own dependency; **this workspace ships none** — see the Update below.

## Why

The map requires reactivity to be replaceable without touching the engine, and a specific reactivity
shipped inside `vitui-runtime` would make "replaceable" mean "fork the crate".

It was proved rather than asserted. The same screen written three ways — a direct application, a TEA
pump with a pure reducer, and a signal graph of `Rc<RefCell<Versioned<T>>>` — produces **0 of 24 000
differing cells** on a runtime none of them modified, and the non-modification is a compile outcome
rather than a claim: the reactivity module `#[path]`s the runtime as its *child*, so the diff over
every prototype module is **0 lines** and both layers see nothing but `pub`.

**The numbers are here rather than in the crate that produced them**, because that crate has since
been deleted and a result whose instrument does not exist is an assertion again. One quiet frame,
300×80, `--release` on the M1 Max:

| driver | ns |
|---|---|
| direct | 19 558.40 |
| TEA | 19 591.80 |
| signals | 19 550.00 |
| **spread** | **41.80 — 0.21% of the slowest** |

Three drivers of one screen are indistinguishable, with **0 of 24 000 differing cells** beside it,
and the signal layer's whole per-frame work — read 3, write 1, ask — is **3.60 ns**. Neither shape is
in the facade, and on this evidence neither should be: shipping one would be picking a winner on a
0.21% spread.

## Consequences

**"Re-running the view is the propagation" is a constraint, not a convenience.** A frame that draws
less than the whole screen *declares* less than the whole screen: the hit index, the focus ring, the
overlay queue and the deadline sink are rebuilt from the draw
([ADR 0012](0012-frame-state-is-rebuilt-from-the-draw.md)). A frame redrawing one region declares
**1 hit entry against 312 and 0 tab stops against 43** while 23 993 of 24 000 cells stay correct, and
the loss is silent for exactly one frame because routing uses the previous frame's index.

**Fine-grained reactivity here is therefore not expensive; it is a request to revert the hit index.**
The expected barrier was the type system and there is none — a stored `Box<dyn Fn(&mut Ctx)>`
compiles, runs and passes. The barrier is a count.

**What a graph may skip is what `Memo` skips** — the computation — and `Computed<T>` turns out to be
`Memo` with the key filled in. **The ratio belongs to the fold's length, not to the mechanism**, and
the 132× this ADR used to quote was one derived value at one length:

| rows | folded | memoised | ratio |
|---|---|---|---|
| 600 | 46.25 ns | 1.46 ns | 31.8× |
| 4 800 | 413.12 ns | 1.67 ns | 248.1× |

At the screen's own 600 rows the memo saves **44.79 ns against a 19 558 ns frame — 0.23%**, and
against the 89.9 µs dense frame, 0.05%. `Memo` is in `vitui_runtime::data` and stays; it is the part
that measured a real win.

**`request_frame()` is on `Ctx`, which TEA cannot reach.** `update` runs after the view by definition
and every `Ctx` is gone by then. This costs no runtime change and has two available fixes, both
belonging to the loop — and the loop is the one piece nothing owns yet.

**Both layers converge, from opposite directions, on copy-and-diff**, because a component writes its
state during the draw and the revision guard bumps on drop rather than on change. The obvious signal —
a `Versioned::edit` guard — never goes quiet: 403 writes of which 403 "changed", 400 frames capped.
The wakeup ledger catches it from outside, streak 403, with the census naming the line.

What makes the diff free is that component state is small, owned and comparable — **a components-map
obligation, not the runtime's**. A component shipping state that is not `Copy + PartialEq` narrows
what can be built above this runtime without the runtime changing at all, and it cannot be enforced
from here.

One asymmetry belongs to the shapes rather than to the runtime: TEA shows a new value one frame later
than a direct write.

## Update — 2026-08-25, `vitui-signals` deleted

**The decision stands and is not reopened; the crate that proved it is gone.** Runtime architecture
issue 24, decided by the user after the numbers above were **re-measured rather than recalled**. The
crate was 112 lines of `Signal<T>`, `Graph` and `Computed<T>`, and what it was is ergonomics over one
runtime hook plus a cache over another — and the cache, `vitui_runtime::data::Memo`, was already in
the runtime.

**What is lost, knowingly.** *Reactivity lives above the runtime* was **proved**: the same screen
written three ways over a runtime none of them modified, the non-modification a compile outcome
rather than a claim. It is now a result whose instrument does not exist, which is why the numbers are
inline above rather than cited — *a decision written in a comment is a decision the next edit can
undo silently*, and the same is true of one written in a deleted crate.

**Two findings the crate discovered and had no other home for**, kept because they are facts about
the tools rather than about it:

- `cargo deny`'s `[bans] deny` bans a crate's **presence in the graph**, with `wrappers` as the
  exception list — so a workspace member with nothing depending on it is banned all the same:
  `error[banned]: crate 'vitui-signals = 0.0.0' is explicitly banned`, with **no dependent to name**.
  An empty `wrappers` list is satisfiable only by a crate outside the workspace, which is why that
  crate carried its own `[workspace]` table and the root excluded it. `deny.toml` keeps this beside
  the `vitui-engine` entry, whose non-empty list is the contrast.
- `request_frame()` is unreachable from a TEA `update`, because every `Ctx` is gone by then. The
  mechanism half closed with issue 23
  — there is a loop to put the fix in now — but this is still why the hook is shaped as it is.

**Not in scope, and not to be re-added by a later reading:** no replacement crate, no `signals`
feature, no reactivity module in the runtime. The four hooks above are the surface. A reader who
disagrees should reopen issue 24 rather than quietly restoring the crate.
