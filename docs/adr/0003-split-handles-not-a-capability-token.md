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

## Amended by the implementation, 2026-08-22 (engine impl 18, 19, 23)

**All four halves are now internal, and the enforcement is stronger for it.** `attach` spawns the
render and input threads and `present` owns the whole sequence, so `Parker`, `Unparker`, `Producer`,
`Consumer` and the `Ui` token are not public names at all: what remains on the public surface is
`Screen` (`!Send`), `WakeHandle` (`Send + Sync + Clone`, `post` and `quit`), `Slot<T>`
(`Send + Sync`) and `Permit` (`!Send`). *Consequences*' list of what crosses threads should be read
against those four rather than against the prototype's. `Perf::enter` and `Perf::leave` stopped being
merely unforgettable and became **uncallable**, because `wait` and `present` are the only things that
can reach them.

**The two holes are closed in the shipped crate and each is a paired `compile_fail` doctest on
`Screen`**, rather than in the prototype this ADR was written against: a worker cannot `present` (the
lease and the submit), and a worker cannot `wait` (the stolen wake). The third door they would have
gone through instead — `layers()` — is closed the same way.

One thing this ADR's own reasoning did not reach. *A private zero-sized field delivers the identical
guarantee invisibly* is true of `View` and `Screen`, and it is **not** true of a guard that has to be
held across a draw: `Permit<'a>` borrowed out of `&'a Screen` makes `Screen::layers` unreachable
(`E0502`) inside the very region the permit exists to excuse. `Permit` therefore holds an `Rc` and has
no lifetime, and its `!Send`-ness is the `Rc`'s rather than a marker's. The general form is that
**immobility by borrow and immobility by marker are not interchangeable when the value is a guard.**

## Amended by production ticket 12, 2026-08-30 — `attach`'s order, and the byte before the first byte

This ADR's account of `attach` is *it spawns the render and input threads*, and the whole of its
argument rests on **terminal setup finishing before either exists**. That was true and it was not
enough: setup is also an *order*, and the order was wrong.

`Engine::attach` wrote §10's capability batch and entered the alternate screen afterwards, because
`actuate::negotiation` is built from the answers and cannot be written before they arrive. On a
terminal that ignores what it does not implement — which is what the standard asks for, and what this
crate's own terminal model does — that is invisible. **Terminal.app 2.15 prints instead**: the
XTGETTCAP payload comes back as `+q524742` and the final byte of each of the seven DECRQMs as a `p`,
so every application built on this engine left eight artefacts on the line the user's shell prompt
was on, and `?1049l` on the way out restored that page unchanged.

The repair keeps every claim above intact and adds one word to the order: `detect::batch`'s first
bytes are `?1049h`, so **the alternate screen is entered before the first question**, and
`actuate::Page` tells the negotiation the page is already ours, so that mode 1049 is entered exactly
once and the page the batch printed onto is erased once. A second `?1049h` is not free — xterm guards against re-entering the alternate buffer and a
terminal without that guard would save the cursor again, then hand the user's shell back at the
alternate screen's origin.

Two consequences belong to this ADR rather than to §10, because both are about *what exists when*:

- **The renderer's sink does not exist until after detection**, which is what makes *nothing precedes
  `?1049h`* structural rather than remembered. There is no third writer for a byte to escape
  through, and register entry 30 is the count that says so.
- **The alternate screen is owed back before there is a `Screen` to owe it.** `crate::shutdown` is
  armed after detection succeeds, so between the batch and the arming the only thing that can give
  the page back is `Tty`'s `Drop` — the same boundary it already draws for mode 2027. It hands the
  debt over at the arming, because a page given back twice restores the user's cursor twice.

*Split handles are not what enforces this.* Nothing about the order is expressible in a type: it is a
sequence inside one function, on one thread, and a count over the bytes it produces is the only
instrument that can hold it.
