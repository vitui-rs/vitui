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

A TEA pump is a `while` loop over `Ctx` and needs no crate at all. A signal graph is
`vitui-signals` — accepted as a crate, deliberately kept **out of the facade**, and enforced in both
directions by `deny.toml`.

## Why

The map requires reactivity to be replaceable without touching the engine, and a specific reactivity
shipped inside `vitui-runtime` would make "replaceable" mean "fork the crate".

It was proved rather than asserted. The same screen written three ways — a direct application, a TEA
pump with a pure reducer, and a signal graph of `Rc<RefCell<Versioned<T>>>` — produces **0 of 24 000
differing cells** on a runtime none of them modified, and the non-modification is a compile outcome
rather than a claim: the reactivity module `#[path]`s the runtime as its *child*, so the diff over
every prototype module is **0 lines** and both layers see nothing but `pub`.

The facade excludes `vitui-signals` because the two shapes measured identical — all three drivers are
inside the ±0.5 µs noise floor of one another, and the signal layer alone is 4.5 ns, 0.0045%. A
facade that shipped one of them would be picking a winner on no evidence.

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

**What a graph may skip is what `Memo` skips** — the computation, 132× — and `Computed<T>` turns out
to be `Memo` with the key filled in.

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

**`vitui-signals` is a detached workspace, and that is this decision taken literally rather than a
build convenience.** Runtime ticket 18 built the crate and found that *nothing in this workspace may
depend on it* is stronger than it reads: `cargo deny`'s `[bans] deny` bans a crate's **presence in
the graph**, with `wrappers` as the exception list, so a workspace member with nothing depending on
it is banned all the same — `error[banned]: crate 'vitui-signals = 0.0.0' is explicitly banned`, with
no dependent to name. An empty `wrappers` list is therefore satisfiable only by a crate outside the
workspace, which is what `crates/vitui-signals`'s own `[workspace]` table and the root's `exclude`
make it. It also makes the rule **live**: the ban now fires the day a member writes the dependency,
naming the wrapper, where before it was already failing for the crate merely existing. The cost is
that `cargo test --workspace` does not reach its gates, which is why they have a CI invocation of
their own — a crate nobody builds is a crate nobody checks.
