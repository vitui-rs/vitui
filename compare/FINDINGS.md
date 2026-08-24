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

---

# Reading the second run

**2026-08-24, runtime impl ticket 20, Apple M1 Max / macOS 26.5.2 / rustc 1.97.1 / python 3.14.6.**
Arms: `vitui` and `vitui-runtime` at this commit, `ratatui 0.30.2` with `crossterm 0.29.0`,
`textual 8.2.8` on CPython 3.14.6. notcurses still `not built here`.

Runtime impl 20 owed *the runtime's scenes added to engine ticket 26's comparative suite*. What
landed is four scenes and one arm: [`SCENES.md`](SCENES.md) is nine scenes and names the sixteen
runtime scenes it refused, and `arms/vitui-runtime` stands beside `arms/vitui` so that the layer
between them is a subtraction. Everything above this line was written on 2026-08-22 and is not
edited; the numbers it quotes are that run's.

## 9. The suite could not run at all, and had not been able to since it was written

`compare/run.sh` **did not exist**. `README.md` documented three of its flags,
`.github/workflows/compare.yml` invoked it as its Measure step, and `git ls-files compare` had never
carried it. So the monthly workflow's Measure step had been failing since the workflow was committed,
and the only run this suite has ever had is the one the section above describes — driven by hand.

The workflow says of itself, in its own header, *nobody has watched this workflow go green*, and that
sentence turns out to have been the finding rather than the caveat. **A CI job nobody has watched is
not a gate, and a CI job nobody has watched is also not a job**: the missing file was two directory
listings away from the sentence admitting it might be.

`run.sh` now exists, matches the interface `README.md` documented, and is what produced the tables
below. What it fails on is deliberately narrow, and it is the split `Cargo.toml` already argued for:
`--check` fails on a lint in an arm we wrote, and a measurement run fails only when the harness
cannot measure what it claimed to. A missing arm is a row.

## 10. Four arms were drawing four different pictures, and two of them still are

Adding scene 6 — *the application draws, and nothing on the screen is different* — meant replaying
frame 0 of every arm into a text grid and comparing them, which nothing had done before. Three
literals came back wrong:

| | `SCENES.md` says | `vitui` | `ratatui` | `textual` | `notcurses` |
|---|---|---|---|---|---|
| the lorem string | written out since run 1 | ✓ | its own 64 | its own 64 | its own 64 |
| the list label | 19, "stated twice" | ✓ | 20 | 20 | ✓ |
| the dialog's four body rows | written out since run 1 | ✓ | its own four | its own four | its own four |

**Every one of these was correct when the arm was written and was made wrong by the first run.**
`SCENES.md`'s own closing section is a list of eleven under-specifications the first run found, four
of which were *this file did not say which string*; the file was corrected and the arms were not. So
the fix that made the suite honest is what made the arms diverge, and nothing noticed for two days
because **the byte counts did not move**: a different sixty-four characters is the same sixty-four
bytes.

The first two are fixed here, because scenes 6 and 9 are built on them and a scene about two frames
being identical cannot rest on four arms drawing four pictures. **The third is not fixed**, and that
is a scope decision rather than a judgement that it is acceptable: scene 5's dialog body is not
inherited by any new scene, correcting it moves an existing row for a reason unrelated to this
ticket, and the notcurses arm would have to be corrected on a machine that cannot build it. It is a
defect, it is filed here, and the fix is four string constants and a two-column offset in three arms.

The general shape is worth more than the three rows. **A normative file and the code that implements
it drift silently when the drift is invisible in the measurement** — and this suite's measurement is
a byte count, which is exactly the instrument that cannot see a changed letter. The grid replay is
forty lines and found all three in one pass; it is not committed, and the honest recommendation is
that it should be, as a `--check` step rather than a gate.

## 11. `unchanged` is the row the runtime scenes were worth adding for

The four new scenes were picked to say something the first five could not. Three of them do so by
degree; this one does it by having a known right answer.

| `unchanged`, one frame | truecolor | `no-color` |
|---|---:|---:|
| vitui | **0.0** | **0.0** |
| vitui-runtime | **0.0** | **0.0** |
| ratatui | 25.0 | 19.0 |
| textual | 5124.0 | 6066.0 |

`caret` is called *the floor* by the file that defines it and it is not: one cell changes there, so
every arm has something to say and 9.5 against 28.5 is a comparison of two answers. Here nothing
changes, the right answer is zero, and the row is a verdict rather than a comparison.

**ratatui's 25 bytes are unconditional and are not a diff result.** `ESC[39m ESC[49m ESC[59m ESC[0m
ESC[?25l` — reset foreground, background, underline colour and every attribute, then hide the cursor
— written by `Terminal::flush` whether or not the diff produced a single cell. Three kilobytes over
120 frames for a screen nobody touched. It is small and it is a *floor an application cannot get
under*, which is the more interesting property: an idle ratatui application at 60 Hz spends 1.5 kB a
second saying nothing.

**Textual's 5 124 bytes are the same number, to the byte, as its `list-scroll` figure** — 625 358
over 120 frames for both scenes. That is not a coincidence and it is not a bug in the arm: both
scenes repaint forty full-width rows a frame, so both cost forty full-width chops, and **Textual
charges the same for a screen that did not change as for a screen that scrolled.** A refresh means a
repaint; the compositor compares regions, not their contents.

The fair reading, stated because the number invites an unfair one: a Textual application that *knows*
nothing changed emits zero, by not calling `refresh`. This row measures the case where it cannot
know — which is the case every immediate-mode arm here is in on every frame, and is the reason
`SCENES.md` had to write *an arm may not skip the frame* into this scene and no other. The shortcut
is the only one in the suite that leaves no trace in the output, because the output is supposed to be
empty either way.

## 12. What the runtime costs, as a subtraction

The fifth arm exists so that this table is a subtraction and not an argument. Same scenes, same
machine, same run, one layer apart; `arms/vitui` writes the cells that changed and
`arms/vitui-runtime` redraws the whole picture every frame, because that is what code looks like at
each layer.

| one frame, truecolor | engine | runtime | the layer |
|---|---:|---:|---:|
| `caret` | 9.5 | 9.5 | **0** |
| `unchanged` | 0.0 | 0.0 | **0** |
| `fade` | 1394.7 | 1394.7 | **0** |
| `full-repaint` | 96 408 | 96 420 | +12 |
| `scattered` | 82.2 | 114.2 | +32 |
| `filter-shrink` | 76.0 | 108.0 | +32 |
| `status-line` | 22.4 | 52.4 | +30 |
| `list-scroll` | 572.3 | 604.3 | +32 |

**The layer costs about thirty bytes a frame, flat, and it is not the redraw.** An immediate-mode
arm rebuilding 4 800 cells against a damage-marking one writing forty is worth **zero** on the wire —
`unchanged` and `caret` are exact, and `fade` is byte-identical over 120 frames. The equality filter
absorbs the whole of it. That is spec §8's argument arriving from a direction it was not aimed at:
the filter was justified as protection against an *application* that redraws, and the first consumer
it protects is the runtime.

The thirty bytes are one SGR sequence, and their cause is the second finding below. Where the wire
carries an explicit colour they appear once a frame; where it does not, they vanish — at the
`no-color` tier the two arms are **byte-identical on eight of the nine scenes**, because the
serializer narrows the named colours to the terminal's default and the two arms become the same
program. The ninth is `modal-over-list`, where the engine arm draws the spinner bold and this one
does not, which is finding 10's third row wearing another hat.

**The CPU column is where the layer actually shows up**, and only just: **0.23% against 0.27%** of a
core on the caret, where `full-repaint` costs this arm 4 800 `Theme::custom` calls a frame against
zero for the engine arm for twelve bytes of difference. Keystroke to wire is **44 µs against 89 µs
p50**, and the caveat the first run attached to its own latency figure applies twice over here —
neither arm's sample crosses the mailbox, so both are the composite-and-serialise path timed end to
end, with one layer of call in front of the second one. The two figures are the same measurement
taken at two altitudes and neither is what a user would feel.

## 13. The runtime cannot say "the terminal's default colours", and it cost thirty bytes a frame

A `Paint` comes from a `Theme` and cannot be constructed (ADR 0018). A `Theme` is thirteen concrete
colour pairs. The one escape hatch takes `Rgb` and `Rgb`. **There is no argument anywhere in that
surface that means *leave it to the terminal*** — and four of the nine scenes say *in the terminal's
default colours*.

The arm names the two colours the suite had already declared for exactly this problem — scene 5 fixes
them at `rgb(192,192,192)` on `rgb(0,0,0)`, because no arm can read them from inside — so the picture
is the described one and the encoding is not. That is the thirty bytes.

**It is not a `cannot express` and the reasoning matters more than the cell.** It has the shape of
one: a scene asks for something the framework has no word for. It was resolved as an encoding
difference because `SCENES.md` had already ruled that those two colours *are* the terminal's defaults
for measurement. An arm refusing after that ruling would take four rows off the table for a picture
it can draw, and **a missing row reads as a win in whichever direction it is missing.**

Whether the runtime *should* have a default-coloured paint is a map question and is not answered
here. The observation the map would want is that the cost is a constant thirty bytes a frame at the
truecolor tier and nothing at all at `no-color`, and that a thirteen-role theme has no natural place
to put *the one that is not a colour*.

## 14. A cheaper cell because the picture lost an element, and only two arms of the same library caught it

The sharpest thing this run produced, and like the first run's sharpest thing it is not a number in
the table — it is a number that was in the table for an hour.

`Paint` being two colours makes `theme.custom(bg, fg)` look like reverse video, and at the truecolor
tier it *is* the described picture. At the `no-color` tier it is nothing: both colours narrow to the
terminal's default, the swap collapses, and the highlighted row comes out identical to its
neighbours.

| `no-color`, one frame | `arms/vitui` | runtime, swapping colours | runtime, `restyle` |
|---|---:|---:|---:|
| `status-line` | 22.4 | 18.4 | 22.4 |
| `list-scroll` | 572.3 | **40.0** | 572.3 |

**Forty bytes a frame against five hundred and seventy-two, between two arms running the same
compositor.** The scroll had stopped costing anything because the moving highlight had stopped
existing. ratatui and Textual keep theirs at that tier — SGR 7 is an attribute and survives having no
colour — so the cell would have been read against three arms drawing a richer picture.

The fix is to ask for the attribute as an attribute: `Ctx::restyle` with `Repaint::REVERSE`, which is
the engine's third verb surfaced through the runtime, a bit on the cell rather than a pair of
colours. It costs one more verb over the same rectangle and that cost is in the numbers above.

**The rule this suite is built on has a blind spot and this is it.** *A missing row reads as a win*
is written about cells. A missing **element inside a row** reads as a win too, and it has no blank
cell to give it away, no `cannot express` string, and nothing in the harness that could detect it:
the harness checks that two runs are byte-identical, not that the bytes draw the described picture.
What caught it was two arms of the same library disagreeing by 14× on a scene where they had no
business disagreeing at all — which is an accident of this ticket having produced a second arm, and
is not a mechanism.

The mechanism that would catch it is the forty-line grid replay from finding 10. It is the same
instrument, and it now has two findings.

## 15. The other three new scenes, briefly

**`fade`** — 1 200 cells, one colour, moving every frame. It is scene 4's opposite corner and the
pair reads well: at 96 408 bytes `full-repaint` prices a per-cell encoding, at 1 394.7 this prices run
coalescing, and every arm is within 1.5% of every other on the first while Textual is 1.9× on the
second. **It is degenerate at the `no-color` tier for both vitui arms** (0.0) for finding 3's exact
reason — a screen of `=` with no colour is the same screen every frame — and ratatui and Textual pay
1 383 and 2 445 there because they rewrite the glyphs regardless. That asymmetry is the whole content
of the row at that tier and it should not be read as a 1 383× anything.

**`scattered`** — twenty-four cells changing in twelve places, and everything beyond twenty-four
characters is the cost of arriving. 82.2 / 114.2 / 127.2 / 199.0. The spread is the narrowest of any
scene here, which is itself the finding: **cursor addressing is the one thing all four frameworks do
the same way**, because there is only one way to do it. The first run found 582 of ratatui's 705
`list-scroll` bytes were `MoveTo`s and read it as a scroll-region story; with the scroll taken out,
the four arms are within 2.4× and the shape of the column is flat.

**`filter-shrink`** — 76.0 / 108.0 / 110.7 / 291.2, and the row is smaller than it should be for a
reason worth writing down. It was put on the list because nothing else here ever takes anything off
the screen, and *make these cells blank again* is the difference between one erase sequence and a
hundred and twenty spaces. **Not one of the four arms emits an erase sequence.** All four write the
spaces. So the row measures four arms doing the same thing at slightly different prices rather than
the mechanism it was aimed at, and the honest reading is that `ESC[K` is a byte saving nobody in this
comparison is taking — including us, on a scene we wrote.

## What this run still does not say

Everything the first run's list says, unchanged, plus:

- **Nothing about notcurses on the four new scenes.** They are written into `arm.c` and were
  type-checked against a stub header under `-Wall -Wextra -Wpedantic -Werror`; that is not a build
  and it is certainly not a run. The pinned runner owes four cells.
- **Nothing about a component**, still, and now for a sharper reason: `arms/vitui-runtime` takes the
  facade, so `vitui-components` is in its dependency graph and contributes not one line to any
  picture, because it is empty scaffolding. The delta in finding 12 is the runtime's and stops there.
- **Nothing about whether thirty bytes a frame is a price worth paying**, which is a map question
  about a default-coloured paint and is filed rather than answered.

## 16. Two `.pyc` files reached the repository, and the linter put them there

Small, and worth one paragraph because the mechanism is general. `run.sh --check` linted the two
Python files with `python -m compileall`, which does not just parse them — it **writes**
`__pycache__` beside them. Two of those `.pyc` files were then swept into a commit whose subject was
about something else entirely, so a linter run turned into two binary blobs in the history.

`--check` now compiles in memory and writes nothing, `compare/.gitignore` covers the other ways
bytecode appears, and the two files are removed. **A tool that dirties `git status` every time it
runs teaches people not to run it**, which for this directory is the expensive half: the whole
falsifiability argument here is a committed file whose diff a human reads, and a diff with noise in
it is a diff nobody reads.
