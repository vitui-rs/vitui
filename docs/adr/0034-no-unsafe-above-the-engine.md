---
status: accepted
date: 2026-08-24
---

# No `unsafe` in any shipped crate above the engine, and the overlay frame pays for it

`vitui-runtime`, `vitui-components` and the `vitui` facade each carry `#![forbid(unsafe_code)]`.
The one `unsafe` that stood in the way — the frame arena, which held an overlay body between the two
passes with its type erased — is deleted. An overlay body is now a `Box<dyn FnMut(&mut Ctx) + 'f>` in
a `Vec` the frame call owns.

**Decided by the architect on 2026-08-24**, in response to
[ADR 0017](0017-the-overlay-is-two-phase.md)'s arena landing as the runtime's first `unsafe`. Carried
out by runtime ticket 21, which is the one ticket on that backlog licensed to amend the map, because
honouring the decision meant changing spec §10, spec §19 and ADR 0017 and not only the code.

## Why

**The attribute is the whole gate, and it is the shape this repository's rule already asks for**: *a
gate is a count, a ratio, an equality or a compile outcome.* There is no test to write and no number
to tune — the crate either compiles or it does not, and `forbid` cannot be lifted by an inner
`allow`. It subsumes the `#![forbid(unsafe_op_in_unsafe_fn)]` each of the three crates used to carry,
which only shaped `unsafe` that was allowed to exist, so that line is removed rather than left beside
it looking like a second rung.

**It deletes the need for a Miri job.** The engine's no-`unsafe` guarantee was compiler-enforced; the
runtime's soundness rested on review plus one drop-counting test, with nothing in CI checking it.
Closing the gap by removing the subject is cheaper than adding the instrument.

**The arena was not an improvisation, which is why this is an ADR and not a bug fix.** ADR 0017 and
spec §10 both specified it — down to the 32-byte figure — five days before the code existed: *"it
drops nothing itself, so a body that owns anything is dropped by a thunk the request carries beside
it"* **is** type erasure, and type erasure of a closure into a byte buffer needs `unsafe`. The word
was never written down, but the design named the mechanism. So the trade is between two things the map
had already accepted, and it is stated as one: **the property *no `unsafe` above the engine* is worth
more than the sentence *the overlay frame allocates nothing*.**

## Consequences

**A frame with n overlay bodies standing allocates n + 1 times**: one `Box` a body, plus the one `Vec`
that holds them. **2** for a dropdown, **5** for a menu + submenu + modal + tooltip, and **0** for
every frame with nothing standing, because an empty `Vec` allocates nothing. Spec §19 carries the
exception with the count in the same sentence, and the gate under it is the **marginal** figure — one
more overlay standing is exactly one more allocation a frame — because that is the number that stays
true as the queue grows, and it still catches a regression that starts allocating per cell or per row.
At roughly 50 ns each this is invisible against a 100 µs frame; what moved is a budget figure, not a
measurement.

**The queue cannot keep its capacity across frames, and the reason is worth knowing before someone
tries to fix it.** A body may capture anything that outlives the frame call, so a stored body is
`+ 'f` — and safe Rust cannot put a `'f`-bounded value inside the thing that is borrowed for `'f`,
which is what both `Frame` and `Driver` are. So the queue is a local of `Driver::frame`, it starts
empty, and the `+ 1` is structural rather than sloppy. It is also exactly why the arena erased its
bodies' types in the first place: an offset into a byte buffer mentions no lifetime.

**Two mechanisms go away with the arena, and one hazard stops existing.** The drop thunk the request
carried beside its call thunk is gone: a `Box` drops what it holds, the `Vec` drops the boxes, and the
queue is dropped when the frame call ends — including the bodies the pass never reached, and including
an unwind, which the hand-rolled discard could not promise. The double buffering is gone with it,
because the hazard it solved does not arise: growing a `Vec<Box<…>>` moves the pointers, not the
closures, so a body that requests another overlay while it is running cannot free itself. `Bodies`
hands a body *out* of its slot to run it, so the `RefCell` is never borrowed across a call.

**Everything else about overlays is unchanged.** This decision moves where a closure lives. The
28 800 exhaustive placements, the 16-round bound, the census-versus-sweep separation,
press-under-modal and barrier-alone-lets-`Tab`-out all pass untouched, and no public signature moved
except `Frame::arena_high_water`, which becomes `Frame::overlay_bodies_boxed`.

**One latent leak went with it, unasked.** `Ctx::measured` builds its own throwaway `Frame`, and the
arena dropped nothing itself: a body requested inside a dry run was never reached by any pass and
never dropped, so anything it owned leaked. The queue drops it because it owns it.

**`vitui-alloc-probe` is the stated exemption**, not a silent one. It implements `GlobalAlloc` for the
counting allocator behind the allocation gates, and that trait cannot be implemented in safe Rust. It
is `publish = false` and no consumer of `vitui` ever compiles it. The engine's own
`#![forbid(unsafe_code)]` is untouched throughout.

## The alternatives, and the one that was refused on purpose

**Command recording, considered.** Run the body inline during the base pass into a recording buffer
rather than a live view, and replay the recording into the layer afterwards: `Ctx` holds
`Target::Live(View) | Target::Record(&mut Vec<Cmd>)` and every draw verb matches on it. Fully safe
**and** zero-allocation in a steady state, because that buffer *can* keep its capacity. Not taken: the
price is a branch in every draw verb including the base pass, text bytes copied into a reusable buffer
or interned, and a recorder that has to reproduce clipping faithfully or an overlay draws differently
from the base pass. **It puts a branch on the runtime's hottest path to remove `unsafe` from a cold
one.** It is the answer if the zero-allocation sentence ever turns out to be load-bearing for a
consumer, and it is written down here so that it does not have to be re-derived.

**Application-side dispatch, rejected.** `driver.overlays(|cx, owner| match owner { … })` stores no
body, so nothing needs erasing: safe, zero-allocation, no arena. Refused because a `select` could no
longer open its own dropdown without the application knowing about it, and this library exists to be
the foundation a component library stands on.

**Drawing into a standalone `Surface` and blitting it in, a dead end checked so nobody checks it
twice.** `Surface`'s public API is `new`, `size` and `root` — there is no blit — and no cell is
readable from outside the engine by design ([ADR 0023](0023-the-cell-is-never-visible-in-the-public-api.md)).
It needs an engine API change that breaks a stated rule, so it was never a candidate.

The two-phase protocol itself is not an alternative and was not revisited: a component cannot open a
layer mid-draw, because `LayerStack::view` holds `&mut` of the stack for the life of the view
(`E0499`, engine architecture ticket 14 R2). That constraint is inherited.
