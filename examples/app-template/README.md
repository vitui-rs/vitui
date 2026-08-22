# A vitui application template — and spec §11's lint rung

Copy this directory. Replace `draw`. Keep `clippy.toml`.

## `clippy.toml` protects vitui, not vitui's users

That sentence is the whole of why this file exists, and it is a measured fact rather than a policy:

> **A library cannot ship this lint to its users.** Clippy reads `clippy.toml` from the crate being
> linted, and there is no stable mechanism for a dependency to inject lints downstream.

Verified with a control rather than assumed. A two-crate probe with `clippy.toml` in the *dependency*
produces **no diagnostic at all**; the same code with the file in the *application* produces
`warning: use of a disallowed method std::thread::sleep`. So `vitui-engine` cannot switch this on for
you, and the copy of it in this repository — wired into CI by `scripts/lint-rung-gate.sh` — protects
this template and vitui's own tree. **It does nothing for an application that has not copied it.**

A dylint driver would reach further and was rejected: an external dependency and a tool nobody
installs cannot be a gate.

Two things are needed, and one alone is inert:

1. `clippy.toml` beside your `Cargo.toml`, with the `disallowed-methods` list.
2. `#![warn(clippy::disallowed_methods)]` in your crate root. Without it the file is read and nothing
   fires.

## What the rung catches, and what it does not

Spec §11 defines the offence as **a frame-budget overrun by the app thread's iteration, whatever
caused it** — not as a blocking syscall. Android's `NetworkOnMainThreadException` picks the syscall
and therefore catches a DNS lookup while waving through a `for` loop that takes 400 ms; a frozen
interface is a frozen interface, and a pure slow function freezes it identically.

So this list is the **lintable subset** of the definition and never the definition:

| rung | mechanism | catches | cost |
|---|---|---|---|
| compile | `Screen` and `View` are `!Send` | the wrong thread waiting, producing or drawing | **0** |
| compile | no blocking primitive on the app-thread side | waiting for a background result | **0** |
| lint | this file, **opt-in by you** | `fs`, `net`, `sleep`, `join`, `recv`, `lock` | 0 at runtime |
| runtime | the in-loop detector, both profiles | any overrun, after the fact | **45 ns/frame** |
| runtime | an observer thread, **debug only** | an iteration that never returns | ~10 wakeups/s |

It is blunt in one more direction, and the bluntness is worth knowing before you fight it: the lint
fires per **crate**, not per thread. A worker function in the same crate that legitimately blocks is
flagged too. Put `#[allow(clippy::disallowed_methods)]` on that function with the reason on the same
line — a sentence worth having to write.

## What the runtime rungs do to you

- **Debug: the first unexcused overrun panics.** Safe by construction — vitui's restoration is
  idempotent, guarded by one atomic, and runs before the default panic hook prints, so the terminal
  comes back and the backtrace is readable.
- **Release: one warning**, into `Config::overrun_report`. `None` is silence, because a full-screen
  application's stderr is the terminal it is drawing on.
- **Debug also watches this thread from another one.** An iteration that has not come back after 64
  frame intervals — 1.07 s at 60 Hz — restores the terminal, prints when the iteration entered and
  what it said it was doing, and aborts. Not a panic: a panic on the observer's thread unwinds the
  wrong stack and stops nothing.
- The threshold is **one frame interval**, from `Config::max_frame_rate`. Pin your own with
  `Config::overrun_threshold`; `Some(Duration::MAX)` switches the detector off.

`Screen::permit_slow("why")` excuses a region and prints its reason in whichever diagnostic fires. It
is a declaration, not a switch — it does not excuse the observer, because *this will be slow* and
*this has not come back at all* are different claims.

## What to do instead of blocking

```rust
let slot = Arc::new(Slot::new());          // Send + Sync
let (worker_slot, wake) = (Arc::clone(&slot), wake.clone());
std::thread::spawn(move || {
    let result = work();                    // whatever this application is for
    worker_slot.put(result);
    wake.post();                            // and then the wake, never before
});
// ... on the app thread, inside the loop:
if let Some(fresh) = slot.take() { /* non-blocking; the check IS the take */ }
```

`thread::spawn` charged to the calling thread is roughly 6–10 µs at p50 and tens of microseconds at
p99; a resident thread plus a channel send is about half a microsecond. Either is nothing once per
keystroke and a meaningful slice of a frame once per frame — which is why the numbers are here and why
a **worker pool is refused**: it is an executor under another name.
