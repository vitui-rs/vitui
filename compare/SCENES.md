# The scenes, and they are defined by what the user sees

This file is **normative**. An arm implements these nine scenes; the harness measures them; the
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

## Nine scenes, and the last four came from a layer up

**Scenes 1–5 were sliced against the engine and scenes 6–9 against the runtime** (runtime impl
ticket 20). The two backlogs described *the same suite* — same rules, same four external projects,
same pinned runner, same committed file — so this is one list with four rows added to it and not a
second suite standing beside the first.

What the four add is stated once here, because the rule that governs them is the rule that governs
the first five and it is easy to lose: **a runtime scene is admitted only when it survives being
written down as a picture.** The runtime's own normative list
(`crates/vitui-runtime/src/scenes.rs`) is twenty scenes and sixteen of them do not survive that —
not because they are unimportant, but because what they decide is invisible on a wire. *Duplicate
detection's slope*, *the chain stops at the innermost that can move*, *cancel is a bracket*: each
of those is a real property with a real number, and every one of them has to name a mechanism to be
stated at all. A scene that cannot be described without saying *widget*, *route* or *memo* is a
scene this file may not carry, and porting it anyway would have smuggled one framework's model into
a comparison that spans four.

The four below are the ones where a runtime property and a picture are the same sentence:

| here | from the runtime's list | the picture |
|---|---|---|
| 6 — `unchanged` | 16, sixty frames with a job in flight | the application draws and the screen does not change |
| 7 — `fade` | 8, a 300 ms fade at each colour tier | one region, one colour, moving every frame |
| 8 — `scattered` | 1, the dense IDE screen | twelve small changes in twelve places |
| 9 — `filter-shrink` | 6, a search box filtering 600 rows | rows leaving the screen rather than being overwritten |

**And the sixteen that were rejected are named here rather than left as an absence**, on this file's
own rule that a missing row reads as a win. Runtime scenes 3, 9, 10, 12, 17 and 19 (keyed widgets,
wheel chaining, nested scroll areas, two scrolling mechanisms, a flat imported theme, a chunked data
source) each need a mechanism word to be stated and are therefore inexpressible here, not merely
unmeasured. Scenes 7 and 18 (an 8 000-key paste, a Cyrillic layout at three key tiers) are about
what reaches an application from the keyboard, which this suite measures in exactly one place and
not in bytes — see `latency` below. Scenes 2, 11, 13, 14, 15 and 20 are expressible and **draw a
picture one of the nine already draws**: 2 is `list-scroll`'s window onto a long list, 11 is
that window with the highlight driving it, 13, 14 and 15 differ from `status-line` only in what
decides the new text, and 20 — a theme swap over the whole screen — is `full-repaint` with the
glyphs held still, which is what scene 4 already is. *A scene is on this list because it
discriminates*, and a second copy of a row discriminates nothing.

## The nine scenes

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

### 6 — `unchanged`

**Initial screen.** Scene 2's initial screen, exactly: rows 0..38 hold `line NN  ` followed by the
64-character lorem string; row 39 is the status line across the full width in reverse video, padded
to 120 columns, reading

```
 frame 0     elapsed 0.00s    cpu 12%    3 tasks
```

**The change.** **There is none.** On frame *n* the application draws that screen again, status line
included — the first field still reads `frame 0` and the second still reads `elapsed 0.00s`. Frame
*n*'s picture is frame 0's picture for every *n*.

**An arm may not skip the frame.** The loop draws the picture and presents it 120 times, exactly as
it does for the other eight; what an arm's own machinery decides to put on the wire is the
measurement, and an arm that stops calling its renderer is measuring its `if` statement. This is the
one scene where that has to be said, because it is the one scene where the shortcut is invisible.

*Why it is on the list.* **`caret` is called the floor and it is not.** One cell of the four
thousand eight hundred changes there, so an arm has *something* to say, and 9.5 bytes against 28.5
is a comparison of two answers. Here the right answer is a known constant — **zero** — which makes
this the only row in the table with an absolute against which every cell can be read, and the only
one where a number is not a comparison but a verdict. Every arm in this suite claims a mechanism
that reaches it: a double-buffered diff, a damage set marked at write time, a per-widget chop. The
row is whether the claim survives an application that keeps drawing.

It is the runtime's scene 16 — sixty frames with a job in flight, gated at **0 wakeups against 60
polled** — and the runtime's own ledger prices this frame at **23.58 µs** with the note *nothing
skips the composition*. That figure is a CPU cost for a frame that puts nothing on the wire, and it
is deliberately outside the budget as a cliff with a detector. This row is the wire half of the same
sentence, and it is the half the other three frameworks can be asked about.

### 7 — `fade`

**Initial screen.** The screen is blank in the terminal's default colours except for a panel **60
columns wide and 20 rows tall whose top-left cell is column 30, row 10** — so it occupies columns
30..89 and rows 10..29. Every one of the panel's 1 200 cells holds the character `=`. On frame 0 the
panel's foreground is `rgb(0, 0, 0)` and its background is `rgb(0, 0, 0)`, so the panel is not
visible on frame 0 and the screen looks blank.

**The change.** The panel fades up. On frame *n* every one of its cells takes the foreground
`rgb(v, v, v)` where **`v = 2n`**, capped at 255 — `rgb(0,0,0)` on frame 0, `rgb(238,238,238)` on
frame 119. The background stays `rgb(0, 0, 0)` on every frame, the glyph stays `=` in every cell,
and nothing outside the panel is ever written.

*Why it is on the list.* It is scene 4's opposite corner and the pair is the point. Scene 4 changes
every cell's colour to a **different** colour across the **whole** screen, which prices a per-cell
encoding with nothing else in the way. This changes 1 200 cells to the **same** colour inside
**one region**, which prices the two things scene 4 cannot see: whether damage stays inside the
rectangle that moved, and whether an arm can say *these cells, this colour* once instead of 1 200
times. The wire's floor for this picture is twenty cursor moves, one colour and 1 200 characters.

It is also the only scene here whose change is not a change of **content** — no glyph anywhere on
the screen is ever different — which is the shape every animation has and is the runtime's scene 8,
where the same fade is **19 wakeups ungated and 19 / 2 / 2 gated** across three colour tiers. That
scene's finding is that at 256 colours and below the intermediate steps collapse onto the endpoints,
so most of a fade is not visible and the frames that draw it are wasted; the two declared tiers here
are the crude version of the same question, and the `no-color` row is what a fade costs when there
is nothing to fade.

### 8 — `scattered`

**Initial screen.** Twelve panels in a grid four wide and three tall, in the terminal's default
colours. Panel *k*, for *k* from 0 to 11, has its top-left cell at **column `30 * (k mod 4)`** and
**row `13 * (k div 4)`** — so the four columns of panels begin at 0, 30, 60 and 90, and the three
rows of them at 0, 13 and 26. Each panel is 30 columns wide and 13 rows tall, which fills rows
0..38; row 39 is blank.

Panel *k*'s **first** row reads, from the panel's own first column, `panel NN` — where `NN` is *k*,
two digits, zero-padded — and its **third** row (two rows below the first) reads `count NN`, where
`NN` is again *k*, two digits, zero-padded. Every other cell of the screen is a space.

**The change.** On frame *n*, panel *k*'s `count` field reads `(k + n) mod 100`, two digits,
zero-padded. Nothing else on the screen differs between any two frames. **Twenty-four cells change,
in twelve places.**

*Why it is on the list.* Every other scene changes **one contiguous thing**. `caret` changes a cell,
`status-line` a run at the head of one row, `list-scroll` and `full-repaint` a rectangle,
`modal-over-list` a cluster. This one changes twelve, and what it prices is the cost of **arriving**
at a changed cell rather than of writing it: twenty-four characters of content, and everything else
on that row is addressing.

The first run found this by accident and filed it as an aside — **582 of ratatui's 705 `list-scroll`
bytes were cursor motion**, eighty `MoveTo`s to write eighty characters — on a row whose subject was
a scroll region. Here it is the subject. It is the runtime's scene 1, the dense screen with **312
interactive regions and 43 tab stops** in one frame, reduced to the part a byte count can see: a
real screen has many small things on it, and a frame that touches a few of them touches them in
different places.

### 9 — `filter-shrink`

**Initial screen.** Rows 0..38 hold the first 39 rows of scene 3's list: row *r* shows list index
*r* in scene 3's exact format — `NNNNN`, two spaces, then the 19-character label
`item-NNNNN---------` — padded with spaces to 120 columns, in the terminal's default colours. There
is **no highlighted row** in this scene. Row 39 is a query line across the full width in reverse
video, padded to 120 columns:

```
 search: 39 matches
```

**The change.** The query narrows and widens again. On frame *n* let **`m = 39 - (n mod 39)`** — so
39 matches on frame 0, 38 on frame 1, down to 1 on frame 38, and 39 again on frame 39. Screen rows
0..*m*-1 hold list indices 0..*m*-1 in the same format; **screen rows *m*..38 are blank** — every
cell a space, in the terminal's default colours. Row 39 reads ` search: MM matches` with `MM` the
value of *m* in two digits, zero-padded, padded with spaces to 120 columns.

*Why it is on the list.* **Nothing else here ever takes anything off the screen.** Every one of the
other eight is a rewrite: `list-scroll` pads every row out to 120 columns whatever was there,
`full-repaint` overwrites all 4 800 cells, the dialog in `modal-over-list` never closes, and
`unchanged` writes the same thing twice. A search box that narrows its own results is the ordinary
case where the screen has fewer rows on it than it had a frame ago, and this is the one row where an
arm's answer to *make these cells blank again* is visible — which on this wire is the difference
between one erase sequence and a hundred and twenty spaces.

It is the runtime's scene 6, the search box over 600 keyed rows, whose finding is the price of a row
that **stops being drawn**: **90 002 probes against 601** for the same frame, and a quiet frame
paying 0. That is a count of work inside a runtime; this is the same event priced in bytes, and the
two are worth reading together because they are the same widget disappearing.

## The latency scene, which is its own thing

`latency` is not one of the nine and is not measured in bytes.

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
