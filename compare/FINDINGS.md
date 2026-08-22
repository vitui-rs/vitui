# Reading the first run

**2026-08-22, impl ticket 26, Apple M1 Max / macOS 26.5.2 / rustc 1.97.1.** Arms: `vitui` at this
commit, `ratatui 0.30.2` with `crossterm 0.29.0`.

[`REPORT.md`](REPORT.md) is regenerated on every run. This file is written by hand and dated, because
a number and what it means have different lifetimes.

**§15's fifth owed measurement was "the suite has never been run".** It has now, and the ticket's
prediction — *expect the "same scene" definition to break first* — was right, in eleven places. Those
are recorded at the bottom of [`SCENES.md`](SCENES.md), because they are findings about the method and
this file is about the numbers.

## 1. The total reversed a row, and the marginal cost is the table to read

The first version of this harness reported one number a scene: bytes over 120 frames. On `caret` it
said **6 073 for us against 3 488 for ratatui** — we are 74% fatter — and that number was correct and
the opposite of what is happening:

| `caret` | vitui | ratatui |
|---|---|---|
| total, 120 frames | 6 073 | 3 488 |
| **one more frame** | **9.5** | **28.5** |

A total folds the session prologue, the first paint and the 119 frames after it into one figure. Our
steady caret is **three times cheaper**; our *first frame* is 4 900 bytes where ratatui's is a few
hundred, and that swamps everything else at 120 frames.

**So the report leads with the marginal cost and keeps the total underneath it.** The generalisation is
worth more than the row: any per-scene total in a benchmark over *n* frames is a weighted mean of a
one-off and a steady state, and it inverts whenever the one-off is large and *n* is small. This suite
had *n* = 120 and got an inverted row on the cheapest scene on the list.

## 2. Our first frame paints 4 800 cells the terminal has already blanked

This is the finding behind §1 and it is about the engine, not the suite.

Entering the alternate screen gives a blank screen. ratatui's first frame therefore writes only the
cells that are not blank. Ours composites a full-screen opaque content layer and writes **every one of
the 4 800 cells**, blanks included — 120 rows of spaces with a carriage return each, about 4 900 bytes
for a screen the terminal was already showing.

It is not a defect, exactly: nothing has told the engine the alternate screen starts blank, and an
engine that assumed it would be wrong on a terminal that does not honour `?1049h` cleanly. But it is
**a real cost, paid once per session, with a plausible answer** — the mirror could start in a
known-blank state rather than a known-nothing state, which is a claim about `?1049h` and about the
quirk table, not about the compositor.

**Not fixed here.** It is a decision about what the engine may assume of a terminal, which is §15's
territory and not this ticket's, and it is the kind of assumption ADR 0006's mirror exists to make
carefully. Raised, with the number.

## 3. `NO_COLOR` costs ratatui 590× and us nothing, and that is a bug in the shape of a win

| `full-repaint`, one frame | truecolor | `no-color` |
|---|---|---|
| vitui | 96 408 | **0** |
| ratatui | 106 216 | **24 290** |

Both arms honour `NO_COLOR` — which was the surprise, since the suite's own rule exists for arms that
ignore it. What differs is *how*. crossterm 0.29 suppresses the colour but still writes the enclosing
`SetForegroundColor`, so every suppressed colour becomes a five-byte **`ESC [ ; m`** — an SGR
*reset*, not a no-op. Twenty-four thousand of them a frame.

**And it can change the picture**, which is worse than the bytes: an `ESC [ ; m` clobbers a `REVERSED`
attribute set earlier in the same cell, so a `no-color` run of scenes 2, 3 and 5 is not the same
screen as a truecolor run with the colour taken out. That is upstream's, it is reported here rather
than worked around, and it is exactly the case the suite's honesty rule was written for from the other
direction: *a framework that ignores `NO_COLOR` is not thereby faster* — and one that honours it
imperfectly is not thereby slower on purpose.

**Our own 0 needs its own caveat and it is not a triumph.** At the `no-color` tier scene 4's described
change is *not observable at all* — a screen of `#` with no colour is the same screen on every frame —
so there is nothing for the equality filter to emit and it emits nothing. The right reading is that
**scene 4 is degenerate at that tier**, and the row belongs to the filter being correct rather than to
the wire being fast. A suite that quoted `0 bytes` as a colour-blind rendering result would be lying
by arithmetic.

## 4. Where we are ahead, and by how much

Every marginal figure at the truecolor tier:

| scene | vitui | ratatui | |
|---|---|---|---|
| `caret` | 9.5 | 28.5 | 3.0× |
| `status-line` | 22.4 | 46.5 | 2.1× |
| `list-scroll` | 572.3 | 704.7 | 1.2× |
| `full-repaint` | 96 408 | 106 216 | 1.10× |
| `modal-over-list` | 19.0 | 36.0 | 1.9× |

The shape is the one the architecture predicts and it is worth stating as a prediction that held:
**the advantage is largest where the change is smallest.** A caret is 3.0×; a screen where every cell
changes is 1.10×, and on that row the only thing being compared is the encoder, because no filter,
diff or scroll region can save a byte. §8's argument was that marked damage beats a full-buffer diff
on the frames applications actually produce, and the gradient across this column is that argument's
signature.

**`list-scroll` at 1.2× is the row that is not a win and should be.** 572 bytes a frame for a
one-row scroll of a forty-row list is far off what a scroll region can do — our own measurements have
it at 30× to 85× on a steady scroll — and the reason is that `SCENES.md` moves the reverse-video
highlight *with* the list, so every frame changes forty rows' worth of content rather than scrolling
thirty-nine of them intact. The scene, as written, denies both arms the scroll region. **A better
version of this scene exists and is not this one**, and the honest thing is to say so rather than to
present 1.2× as the ceiling on a mechanism the scene does not reach. Also: ratatui's 704 bytes are
**582 bytes of cursor motion** — eighty `MoveTo`s to write eighty characters — which is the same
observation from the other side.

## 5. CPU and latency

| | vitui | ratatui |
|---|---|---|
| CPU, caret at 60 Hz | 0.16% of a core | 0.51% |
| keystroke to wire, µs p50 / p99 | 33 / 56 | 176 / 272 |

The CPU column is a 3.2× and it is measured over the cheapest scene there is, so it is a
fixed-overhead comparison and not a rendering one: what it prices is three threads, a clock and a
mailbox against a synchronous loop, and the three threads win because two of them are parked.

**The latency figure is the one to be most careful about.** 33 µs against 176 µs is a 5.3×, and this
engine's *whole reason* for having a wake-up interval is the thing that should have made it lose here.
It does not, and the honest explanation is that the arm's `latency` mode reads stdin on the app thread
and calls `present` inline, so the sample does not cross the mailbox at all — it is the engine's
composite-and-serialise path timed end to end, which is fast. A real application's keystroke crosses
from the input thread, and §7 puts that hop at 4.58 µs p50 with a parked app thread. **So the number
is real and it is not the number a user would feel**; the missing 4.58 µs is small next to 33, but the
arrangement it is missing from is the one the architecture is actually about. A future run should drive
this through the input thread, which needs a pty.

## What this run does not say

- **Nothing about notcurses**, the honest ceiling, which is not built on this machine: Homebrew's
  formula pulls ffmpeg. Its cells read `not built here`, which is a fact about the run and is not the
  same cell as `cannot express`. The pinned Linux runner installs `libnotcurses-dev`.
- **Nothing about Textual**, for the same class of reason.
- **Nothing about Windows, or about any terminal but one.** Latency is comparable only within one
  terminal and this is one.
- **Nothing about a component library**, ours or anyone's. Every arm here is written directly against
  a rendering layer, which is the only level at which the four are comparable at all, and it is not
  the level at which anybody writes an application.

## 6. The suite's own fairness rule was built on a false fact about notcurses

This is the sharpest thing the first run produced, and it is not a number.

Architecture ticket 13, line 574, and the ticket that sliced it:

> **notcurses is the honest ceiling**, and it does not composite layers. The layered scenes therefore
> have no notcurses arm, and that cell in the table reads **"cannot express"**, not blank. A missing
> row reads as a win, and this is the single easiest way for the suite to become dishonest.

**notcurses composites layers.** It has `NCALPHA_BLEND`, resolved by the renderer over the plane
stack: `ncpile_render_internal` walks the pile top to bottom, `paint` contributes while alpha is above
`NCALPHA_OPAQUE`, and the contribution goes through `channels_blend` as a running mean. A full-screen
plane above the list with a black base cell and `NCALPHA_BLEND` lands the list at exactly half.

Built from the 3.0.17 release tarball and run, not inferred. Frame 0 of the layered scene emits
`38;2;96;96;96` for a 192 foreground, `48;2;127;127;127` for a 255 background, and
`38;2;224;224;224` for the dialog above the blend — untouched. **The caller computed no dimmed colour
anywhere.** The two calls that look like the answer and are not are `ncplane_stain`, which *replaces*
channels and is the pre-compute route, and `ncplane_format`, which is style bits; presumably one of
those is how the original claim was reached.

**The rule ate itself.** *A missing row reads as a win* was written to stop the suite flattering us,
and following it to the letter would have printed a cell asserting that a competitor cannot do
something it can — in the one place in the table with no external check on it. A dishonest cell
produced by the honesty rule.

**What was done, and what deliberately was not.** The arm still refuses `--scene modal-over-list` and
exits 69, because `SCENES.md` is normative and the backlog does not reopen decisions. The real picture
is a separate opt-in scene, `modal-over-list-blend`, so the correction is demonstrable rather than
asserted. The evidence is appended to architecture ticket 13. **Flipping the cell is that map's call**,
and what it needs is a decision about what the layered row is *for* — and on either answer notcurses
belongs in it with a number.

## 7. Textual, and where the arm's own idiom decides the number

Textual 8.2.8 on CPython 3.14.6, a real `App` through the real `LinuxDriver`, with only the output
sink swapped — because **Textual renders to stderr**, so a user piping a Textual app's stdout gets an
empty file.

| | vitui | ratatui | textual |
|---|---|---|---|
| `caret`, one frame | 9.5 | 28.5 | 25.0 |
| `status-line`, one frame | 22.4 | 46.5 | 47.5 |
| `list-scroll`, one frame | 572 | 705 | 5 124 |
| `full-repaint`, one frame | 96 408 | 106 216 | 111 036 |
| CPU, caret at 60 Hz | 0.16% | 0.51% | ≈4.7% |
| keystroke to wire, p50 | 33 µs | 176 µs | 0.9–2.6 ms |

Three things about this column need saying, and only the first is about Textual being slower.

**`list-scroll` at 5 124 bytes a frame is 9× the next arm**, and it is the row where the difference is
architectural rather than incidental: Textual's smallest update is a **widget**, not a cell, and
`ChopsUpdate` renders from a widget's left edge to the right edge of the change. Forty rows changing
means forty full-width strips.

**The arm's idiom moved the caret row by 5×**, and this is a fairness problem the scene rules do not
solve. A one-cell caret needs a one-cell *widget*: written that way it is 21 bytes a frame, and written
the ordinary `Static.update()` way it is 134. Both are Textual, both draw the described picture, and
`SCENES.md` cannot constrain the choice without smuggling in a model. The tuned figure is what the
table carries and `VITUI_ARM_TEXTUAL_IDIOM=naive` reproduces the other, which is the most honest
arrangement available: **report the better one and ship the lever that shows the spread.**

**And 78% of Textual's `caret` total is startup** — 11 195 bytes of 14 396, because it paints the
initial screen four times before going idle. That is the strongest independent argument for §1's
marginal-cost table: the scene this file calls *the floor* measures Textual's boot, and a total would
have reported boot as though it were the floor.

Two upstream defects found on the way, reported and not worked around: `filter.monochrome_style`
raises on a `Segment` whose style is `None`; and Textual **busy-spins a core** when stdin is an EOF
pipe, which would have wrecked the CPU number had the arm not parked the input thread.

## 8. `COLORTERM` was missing from the contract and was worth 39% of a scene

Found independently by two arms. `TERM=xterm-256color` does **not** declare 24-bit colour, so
notcurses' `query_rgb` and rich's colour-system probe both quantise scene 4's ramp to `38;5;N` — which
is not a cheaper encoding of the same picture, it is **a different picture**. Measured on the notcurses
arm: 111 708 bytes against 183 648 for the same two frames.

The harness sets `COLORTERM=truecolor` on the truecolor tier and `ARM-CONTRACT.md` now says so. The
general shape is the ninth instance of the same lesson: *the declared tier is only declared to the
extent it is written down*, and every part of it an arm has to guess is a part of the measurement
delegated to the thing being measured.
