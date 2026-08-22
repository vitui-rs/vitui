# The scenes, and they are defined by what the user sees

This file is **normative**. An arm implements these five scenes; the harness measures them; the
report ([`REPORT.md`](REPORT.md)) is a table over them.

> **A scene is defined by what the user sees, never by what a framework does.**

That rule is the whole reason this file is written the way it is, and it is not a style preference.
Every definition below is a terminal size, a described initial screen, and a described sequence of
changes. Nothing here says *widget*, *redraw*, *diff*, *layer*, *re-render* or *component*, because
each of those words is one framework's model, and the comparison spans immediate-mode and
retained-mode designs. A scene phrased as "the same widget updated the same way" would have decided
the result before the first byte was measured.

The corollary is a licence rather than a restriction: **an arm may reach the described screen any way
its framework prefers.** If ratatui's idiom is to rebuild the whole frame and let its diff sort it
out, that is ratatui's arm; if ours is to write a hundred cells and mark them, that is ours. What is
compared is the bytes each one puts on the wire and the CPU each one spends to reach the *same
picture*.

## The terminal, fixed for every scene

**120 columns × 40 rows.** There is no tty — stdout is a pipe (see [`README.md`](README.md)) — so an
arm takes the size from `COLUMNS` and `LINES`, and defaults to 120×40 when they are unset. An arm
that queries the terminal for its size and gets an error must not fall back to 80×24 silently; the
harness sets both variables on every run.

**120 frames**, for every scene, unless the harness is told otherwise with `--frames`. Frame 0 is the
initial screen; frames 1..119 are the described change applied once each.

## The five scenes

### 1 — `caret`

**Initial screen.** Row 0 reads `vitui compare — caret` in the terminal's default colours. A block
caret sits in row 0 immediately after the final `t`. Rows 1..39 are blank.

**The change.** The caret alternates between shown and hidden. Nothing else on the screen differs
between any two frames.

*Why it is on the list.* It is the floor. One cell of the four thousand eight hundred changes, and
what an arm spends to say so is the closest thing to a fixed overhead this suite can measure.

### 2 — `status-line`

**Initial screen.** Rows 0..38 hold static text: row *n* reads `line NN  ` — two digits,
zero-padded — followed by this exact 64-character string, in default colours:

```
lorem ipsum dolor sit amet consectetur adipiscing elit sed do ei
```

Row 39 is a status line across the full width, in reverse video, padded with spaces to 120 columns:

```
 frame 0     elapsed 0.00s    cpu 12%    3 tasks
```

**The change.** On frame *n*, `frame 0` becomes `frame n` and `elapsed 0.00s` becomes the elapsed
figure `n / 60` to two decimals. The `cpu 12%` and `3 tasks` fields never change, and rows 0..38
never change.

**The field widths are normative, and they are worth about ten times this row.** `frame ` is followed
by the number left-aligned in **6 columns**, so `frame 0` and `frame 119` occupy the same width and
nothing after them moves. `elapsed ` is followed by the figure in **6 columns** including the `s`. An
arm that lets the tail shift pays roughly 500 bytes a frame instead of 46 — a difference in the arm's
*formatting* that would have been read as a difference in its renderer.

*Why it is on the list.* §8's dialogs had a live status bar, ours did not, and the twelve scenes
therefore could not discriminate a gap threshold at all. This is that scene, in the one suite where
another framework's answer to it is visible.

### 3 — `list-scroll`

**Initial screen.** Forty rows of a ten-thousand-row list. Row *r* of the screen shows list index
*r*, formatted `NNNNN  ` — five digits, zero-padded — followed by the label `item-NNNNN---------`,
which is **19 characters**: the literal `item-`, the five-digit index, and nine `-`. Each row is
padded with spaces to 120 columns. List index 12 is drawn in reverse video; every other row is in
default colours.

**The change.** The window advances by one list index per frame: on frame *n* the screen's top row is
list index *n*, and the reverse-video row is list index `12 + n`, so the highlight stays on the same
screen row and follows the same list item.

*Why it is on the list.* This is where a scroll region either exists or does not, and the difference
is 30× to 85× of wire on our own measurements. An arm with no scroll region is not thereby wrong; it
is thereby expensive, and that is the number.

### 4 — `full-repaint`

**Initial screen.** Every one of the 4 800 cells holds a `#` whose **foreground** is set from a
24-bit ramp: the cell at column *c*, row *r* takes `rgb(c * 2, r * 6, 128)`. The background is left
at the terminal's default. Foreground only — an arm that sets both doubles this row, and this row is
the ceiling, so doubling it would misprice the one scene that has nothing else in it.

**The change.** The ramp shifts one column left per frame — on frame *n* the cell at column *c* takes
the colour column `(c + n) % 120` had on frame 0. Every cell differs from the frame before it.

*Why it is on the list.* The ceiling. No filter, no diff and no scroll region can save anything here,
so this row is where an arm's *encoding* is measured with nothing else in the way — and it is the one
row on which a framework that has no diff at all is not penalised.

### 5 — `modal-over-list`

**Initial screen.** Scene 3's screen, at its frame 0, with two things over it:

1. Every cell of the list is **dimmed** — its foreground and background mixed halfway toward black.
   **The colours the dim starts from have to be declared, because no arm can know them from inside.**
   A terminal's default foreground and background are not reportable, so this scene fixes them:
   foreground `rgb(192, 192, 192)`, background `rgb(0, 0, 0)`, and halfway toward black is therefore
   `rgb(96, 96, 96)` on `rgb(0, 0, 0)`. Without this line no two arms produce the same picture, and
   one of them produces no picture at all: an operator that mixes toward black over an unknown ground
   is a no-op, so our own arm's dim silently did nothing until the two defaults were declared.
2. A 40×12 dialog is centred (top-left at column 40, row 14), **undimmed**. Its rows, exactly:
   row 0 is `┌` + 38 `─` + `┐`; row 1 is `│`, a space, the spinner glyph, ` working`, spaces to
   column 39, `│`; rows 2 and 7..10 are `│` + 38 spaces + `│`; rows 3..6 carry body text starting at
   column 2 of the dialog:

   ```
   Copying 4 of 12 files.
   This dialog is a layer, and the
   list behind it is another one.
   Nothing below has been rewritten.
   ```

   Row 11 is `└` + 38 `─` + `┘`. **The title is a row of its own, under the border and not in it** —
   which is not the idiom of every framework, and is specified here because "a title row" reads as
   `Block::title` to a ratatui author and as a separate line to ours, and those are different
   pictures.

**The change.** The spinner glyph in the title cycles `⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏` one step per frame. The
list behind the dialog does not scroll, and the dim does not change.

*Why it is on the list.* It is the layered scene, and it is the one that produces a **`cannot
express`** cell rather than a number.

**The claim that notcurses cannot composite layers is withdrawn.** Impl 26 built notcurses-core
3.0.17 and demonstrated the opposite: `NCALPHA_BLEND`, resolved by the renderer over the plane stack,
lands a plane below at exactly half without the caller computing a single dimmed colour. So the
`modal-over-list` cell against notcurses is **not** a statement about what notcurses can do — the arm
refuses that scene because `SCENES.md` is normative and the correction is a map decision, and the real
picture is an opt-in scene `modal-over-list-blend` beside it. The evidence is filed against
architecture ticket 13, whose line 574 is the sentence that was wrong. **A cell claiming a competitor
cannot do something it can would have been the exact dishonesty the missing-row rule exists to
prevent, committed by that rule.**

**And the dim has a second ambiguity this file has not resolved.** Two arms drew the described picture
by two mechanisms: ours through an operator layer over declared default colours, Textual through
`SGR 2` on the list layer under a transparent modal. Both are the picture; the bytes are not
comparable, because one is a colour per cell and the other is one attribute. This row is therefore
reported **with that caveat rather than as a clean comparison**, and picking one mechanism would be
picking a framework — which is the thing the scene rules exist to refuse. **A missing row reads as a win, and that is the single easiest way for this
suite to become dishonest** — so an arm that cannot reach this picture is written down as unable to
reach it, never left blank and never quietly dropped from the mean.

## The latency scene, which is its own thing

`latency` is not one of the five and is not measured in bytes.

**Initial screen.** Scene 2's screen.

**The interaction.** A single keystroke arrives on stdin. The status line's first field becomes
`key <name>` — `key a`, `key Up`, `key C-c`. What is measured is the interval from the byte entering
the arm's stdin to the arm's **first output byte after it**: keystroke to wire.

**First output byte means the first byte, not the first byte a human would see.** An arm whose first
byte is a cursor move has already reacted; whether the visible change is twelve bytes behind it is a
property of its encoding and belongs in the byte column, not in this one. Measuring to the first
*visible* byte would charge one arm twice for the same encoding choice.

Keystroke-to-wire, not keystroke-to-photon. The emulator's own paint is not in the number and cannot
be from a harness. This is the honest form of the quantity, and it is the one that is comparable at
all: our own wake-up interval is scheduler latency, and a synchronous renderer has no such interval,
which is exactly why an absolute per-frame figure would compare two different things.

## The CPU scene, which is also its own thing

`cpu` is scene 1 driven at 60 frames a second for a fixed ten seconds of wall time, with process user
and system time read afterwards. Fixed wall time rather than a fixed frame count, because what is
being compared is what an arm costs a laptop while a caret blinks — and an arm that cannot keep 60 Hz
should show up as slow rather than as having taken longer to finish.

## What broke in these definitions, and what it cost

**§15's fifth owed measurement was *the suite has never been run*, and the ticket predicted what
would happen: expect the "same scene" definition to break first.** It did, in eleven places, and
every one of them was found by an arm rather than by reading. They are recorded here rather than
quietly fixed, because the pattern in them is the useful part.

**Two were genuine framework limits, and they are the suite working as intended:**

1. ratatui has no layer compositing, so scene 5's dim is a pass over its buffer with the reverse-video
   row resolved by hand. That is legitimate — the scene is a picture — and it is the difference the
   `modal-over-list` row exists to price.
2. A **block** caret is unreachable in ratatui: its `Frame` carries a cursor *position* and never a
   shape. So scene 1's caret is a position in that arm and a position plus a shape in ours.

**Nine were this file under-specifying, and the costs were not small:**

3. Scene 2's field widths were unstated and are worth **about ten times that row** — 46 bytes a frame
   against roughly 500, depending on whether the tail shifts. Now normative above.
4. The 64-character lorem string was described by its length and not given, and it decides scene 2's
   frame 0 outright. Now written out.
5. Scene 3 contradicted itself: the prose said a 20-character label and the example `item-NNNNN---------`
   is 19. Now 19, stated twice so the next reader does not have to choose.
6. Scene 5's dim had **no colour to be halfway from**. This is the worst of the nine, because it does
   not produce disagreement — it produces a no-op. Our own arm's dim did nothing at all until the two
   defaults were declared, and the row was byte-identical across both colour tiers, which looked like
   a finding about the equality filter and was a missing sentence in this file.
7. Scene 4 did not say foreground or background, and an arm that set both would have doubled the
   ceiling row.
8. Scene 5's title row: in the border or under it. Two idioms, two pictures.
9. Scene 5's body text and its blank rows were unspecified.
10. `latency` did not define *first output byte*, and the first byte is twelve bytes ahead of the
    first visible one in at least one arm.
11. And one that is about the harness rather than the scenes: **`crossterm::terminal::size()` opens
    `/dev/tty`**, so with stdout on a pipe it does not fail — it succeeds and returns the size of the
    developer's own window. An arm written to "fall back to `COLUMNS` when the size query fails"
    never reaches the fallback, and measures whatever window the operator happened to have open.
    The contract's *there is no other source of the size* is therefore a rule an arm has to be
    written to obey, not one the pipe enforces.

**The shape of all nine is the same, and it is the one the ticket warned about from the other
direction.** A scene defined by what the user sees is the right rule, and it is not
self-enforcing: *what the user sees* includes every column position, every literal string and every
colour the eye would notice, and a definition that leaves one of them to the arm has silently
delegated part of the measurement to the thing being measured.
