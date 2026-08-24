# The arm contract

Every arm is a program the harness can run. This file is what the harness relies on; it is short on
purpose, and everything it does not say is the arm's own business — including which framework idiom it
reaches the picture with (see [`SCENES.md`](SCENES.md)).

## Invocation

```
<arm> --scene caret|status-line|list-scroll|full-repaint|modal-over-list --frames N
<arm> --scene unchanged|fade|scattered|filter-shrink --frames N
<arm> --scene latency
<arm> --scene cpu --seconds S
```

- The nine picture scenes write `N` frames to **stdout** and exit 0. Frame 0 is the initial screen.
  The second line is the four `SCENES.md` gained from the runtime backlog; they take the same two
  arguments and are separated only because one line of nine scene names is unreadable.
- `latency` writes the initial screen, then reads stdin byte by byte until EOF; on each keystroke it
  updates the status line's first field to `key <name>` and flushes. It never buffers a reply.
- `cpu` drives scene `caret` at 60 frames a second for `S` seconds of wall time, then exits 0.

## What the harness guarantees the arm

- **stdout is a pipe, never a tty.** An arm that refuses to run without a tty has failed the
  contract, and the report says so rather than omitting it.
- `COLUMNS=120` and `LINES=40` are set. There is no other source of the size.
- `TERM=xterm-256color`, and **`COLORTERM=truecolor` on the truecolor tier**. The second was missing
  from the first version of this file and it is worth **39% of scene 4**: `xterm-256color` does not
  declare 24-bit colour, so notcurses' `query_rgb` and rich's colour-system probe both quantise the
  ramp to `38;5;N` and draw a *different picture* — 111 708 bytes against 183 648 for the same two
  frames. Two arms found this independently. An arm may not set it itself; the harness sets it, and an
  arm that quantises anyway reports so in its `NOTES.md`.
- `NO_COLOR` unset on the truecolor tier and set on the `no-color` one.
- **Its own session.** The harness spawns every arm with `setsid`, so `/dev/tty` is unreachable. That
  is load-bearing rather than tidy: notcurses opens it, takes the geometry from `TIOCGWINSZ`, and then
  blocks for ever waiting for a Device Attributes reply a pipe will never carry — demonstrated, and
  killed at two minutes. crossterm's `terminal::size()` does the same thing more quietly, by
  *succeeding* and returning the operator's real window size, so an arm written to fall back to
  `COLUMNS` on failure never reaches its fallback.
- stdin is a pipe. For the nine picture scenes it is closed immediately; nothing is sent.

## What the arm owes the harness

**One line on stderr before the first frame**, and nothing else on stderr:

```
arm=<name> version=<exact version> no_color=<honoured|ignored|unsupported> alt_screen=<yes|no>
```

`no_color` is the field the suite's honesty rests on. **A framework that ignores `NO_COLOR` is not
thereby faster** — it is emitting fewer SGR sequences than it was asked to and more than the tier
allows, and the byte count means something different for it than for an arm that honoured the
declaration. `unsupported` is for a framework that has no such concept, which is a third answer and
not a synonym for either of the first two.

`alt_screen` matters for the same reason: an arm that never enters the alternate screen has skipped a
one-off cost every other arm paid, and the harness discards the enter/leave prologue only when it can
see it.

## What the arm must not do

- Exit 0 on a picture it cannot draw. **The status for *I cannot reach this picture* is 69**
  (`EX_UNAVAILABLE`), with `cannot express` on stderr and no bytes at all on stdout. Exiting 0 with
  fewer bytes is the one failure mode this whole suite is built to prevent, because a missing row
  reads as a win.
- Write to stdout anything that is not the scene — no banner, no timing, no summary.
- Sleep between frames on the nine picture scenes. Those are byte measurements; wall time on them is
  not compared and a sleep only makes the CPU column meaningless.
- Read the clock to decide what to draw. Frame *n*'s picture is a function of *n*, so two runs of the
  same arm produce byte-identical output. The harness asserts this.
