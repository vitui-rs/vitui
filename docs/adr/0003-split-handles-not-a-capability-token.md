---
status: accepted
date: 2026-08-17
---

# The app-thread role is enforced by splitting handles, not by a capability token

Each primitive that the app thread shares with another thread is presented as **two types**, not one.
`Parker` waits and is `!Send`; `Unparker` only posts and is `Send + Sync + Clone`. `Producer` leases
and submits and is `!Send`; `Consumer` takes and finishes, is `Send`, and is neither `Sync` nor
`Clone`. `Engine` is `Send`, is built before any thread exists, and `Engine::attach` consumes it on
whichever thread is going to be the app thread — minting the `!Send` halves and the `Ui` token there.

No capability token appears in any public signature. A method the caller's thread may not use is
simply not on the type it holds.

This is surprising twice over. A reader expects one `WakeSource` with four methods, and a reader who
knows the requirement ("the UI thread cannot be blocked, enforced rather than documented") expects a
`!Send` token threaded through the API, which is the design the originating ticket proposed.

## Why

**A token was built and measured first, and it is free.** Carried into every drawing verb, ticket 05's
headline frame drew in 151.09 µs against 152.77 µs without it — noise, in the token's favour. Cost was
never the objection.

The objection is that a token in a signature is a tax paid at every call site for a guarantee the type
system will give for nothing. `text(x, y, s, style, &ui)` fails the standing ergonomics requirement,
and a private zero-sized field in `View` delivers the identical guarantee invisibly.

**And a token does not prevent blocking anyway.** It prevents a *thread* from holding a capability.
Once that is understood, the enforcement worth having is not "you may not block" — no type expresses
that — but "you may not be the app thread". Splitting handles states exactly that, with no vocabulary
added for the caller to learn.

The decisive argument is that one of the primitives is **self-contradictory as a single type**. Its
posting method must be callable from a background thread, so the type must be `Sync`. Its waiting
method must not be callable from a background thread, so the type must not be `Sync`. No amount of
documentation reconciles that; two handles do.

The alternative was tested against reality rather than assumed. Before the split, two tests were
written against the pre-existing prototype: a worker thread leasing and submitting a frame, and a
worker thread consuming the wake the app thread was waiting for. **Both passed.** The invariant that
the frame slot can never be superseded rested on there being one producer, and nothing had made one
producer true. Both are now compile errors.

## Consequences

The threading contract is readable off the public API, which is the property that makes it checkable:
`Engine`, `Surface`, `Unparker` and `Consumer` cross threads. `View`, `Parker`, `Producer`, the layer
stack and the token do not. There is no blocking primitive and no completion value anywhere on the
app thread's side of the seam — background work posts, and the app thread discovers the result on its
next iteration.

**The app thread need not be the process's first thread.** This falls out of `Engine` being `Send`
while its products are not, and it is the reason the split needs no `unsafe` constructor: the token is
minted at the destination rather than transported to it.

A background thread's entire vocabulary is one verb. That is a constraint on the reactive layer above
— a background result arrives as a single wake, never as a callback running on the worker — and it is
now unrepresentable rather than discouraged.

`Waker` is not available as a name. In `std::task::Waker` it is half the `Future` contract, and this
project bans futures outright; borrowing `Parker`/`Unparker` from `thread::park`/`unpark` costs
nothing and misleads nobody.

The narrow price, paid knowingly: with `View` unsendable, one surface cannot be split across threads,
so sibling views stay single-threaded. Drawing into *separate* surfaces on separate threads is
unaffected, because `Surface` remains `Send` deliberately.

Splitting is free at runtime. Both halves hold the same shared state behind the same `Arc`; only
reachability differs. What changed is which mistakes compile.
