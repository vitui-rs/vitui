---
status: accepted
date: 2026-08-16
---

# crossterm is the terminal backend, used for input only, behind a seam

`vitui-engine` uses `crossterm` for raw-mode handling, input reading and platform abstraction, but
writes its own bytes for output and does not expose `crossterm` in any public type. A future reader
will notice the dependency and wonder why half of it is unused — this records that the split is
deliberate.

## Considered options

**Build the terminal layer from scratch on `libc`.** Genuinely attractive: the strict dependency floor
for a terminal library is `libc` regardless (`std` exposes neither termios nor `ioctl`), the output
path would be written by hand anyway, and dropping legacy Windows conhost in favour of VT-only
Windows Terminal removes the largest single chunk of platform work. Rejected because the terminal
*input* parser is where the multi-month bug tail lives — `ESC` as a key versus `ESC` as a sequence
prefix is disambiguated only by timing and breaks on a high-latency ssh link, and beyond that sit
three incompatible mouse encodings, bracketed paste, kitty keyboard negotiation, UTF-8 split across
`read()` boundaries, and escape sequences arriving in fragments. None of that work advances any of the
project's seven requirements.

**Build on `ratatui`.** Rejected on architecture, not effort: ratatui is immediate-mode and repaints
the whole buffer every frame, while the layer stack, shadows, animations and damage tracking this
engine is built around require a retained model. We would be fighting its design rather than using it.

**`termwiz`.** A better fit than ratatui — its `Surface`/`Change` model is closer to what we want — but
a smaller ecosystem and no decisive advantage over writing the compositor ourselves, which we intend
to do regardless.

## Consequences

crossterm's output layer — `queue!`, `Print`, `SetForegroundColor`, `Stylize` — is **not used**. It
formats through `fmt::Write` and cannot satisfy the zero-allocation, single-`write`-per-frame budget.
The engine serialises its own bytes into a reusable buffer.

The `event-stream` feature is **not enabled**. It pulls `futures-core`, which would tie the engine to
an async runtime; the map requires it to stay runtime-agnostic. Input is read with `poll`/`read` on a
dedicated thread instead.

`derive-more` is dropped from crossterm's defaults. It only adds trait derives to crossterm types we
do not expose, and it drags a proc-macro chain (syn, quote, convert_case, unicode-segmentation) into
the build graph. Without it the engine's entire transitive tree is 13 crates of platform plumbing.

**`mio` and `signal-hook` arrive regardless**, through the `events` feature — crossterm 0.29 declares
`events = [dep:mio, dep:signal-hook, dep:signal-hook-mio]`, on every platform. This was assumed
avoidable when the decision was taken, and it is not: `events` *is* the input parser, which is the
entire reason for the dependency. They are crossterm's polling implementation rather than an async
runtime, and they are accepted. `deny.toml` therefore bans `futures-core`, `tokio` and `async-std` —
the real signals of runtime lock-in — and deliberately does not ban `mio`.

Because crossterm appears in no public signature, replacing it later — with a hand-written backend, or
with termwiz — is a contained change behind the backend trait rather than a breaking API change. This
is what makes the decision reversible enough to take now, and it is load-bearing: do not let a
crossterm type leak into a public API.

Windows is supported through crossterm as a consequence rather than a goal. Windows Terminal gets the
full feature set; legacy conhost receives the degraded path that `--no-color` and `--ascii` require
anyway.
