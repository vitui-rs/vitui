# The Textual arm — what it measures, and where the scenes did not fit

Read with [`../../SCENES.md`](../../SCENES.md) and
[`../../ARM-CONTRACT.md`](../../ARM-CONTRACT.md). Everything below was observed on this machine,
not inferred from documentation.

## Versions

| | |
|---|---|
| Textual | **8.2.8** (latest on PyPI, 2026-08-22) |
| Python | **CPython 3.14.6** — Homebrew `python@3.14`, `/opt/homebrew/bin/python3.14` |
| rich | 15.0.0 |
| host | macOS 26.0 (Darwin 25.5.0), arm64 |

Textual 8.2.8 installs and runs on 3.14 without complaint, so **no fallback to 3.13 was needed**.
The full transitive closure is pinned in `requirements.txt`; nothing floats. `./setup.sh` builds
`.venv` and is idempotent. `arm.py` carries a `#!/usr/bin/env python3` shebang and re-execs itself
under `.venv/bin/python3` if `textual` is not importable, so both `./arm.py …` and
`.venv/bin/python arm.py …` produce byte-identical output.

## What the bytes on stdout are the bytes *of*

A **real Textual `App`**, started with `App.run_async()`, using Textual's **real `LinuxDriver`** —
real alt-screen prologue, real stylesheet and CSS, real compositor, real `Strip`/`Segment`
encoding, real writer thread, real 60 Hz screen-update timer. This is not the `rich` layer standing
in for Textual, and it is not `HeadlessDriver` (which cannot be used: `App._display` returns early
when `is_headless`, so a headless run emits nothing at all).

Two things about the driver are the arm's, and only two:

1. **The output sink.** `LinuxDriver.__init__` sets `self._file = sys.__stderr__` — *Textual renders
   to stderr, not stdout.* A user who pipes a Textual app's stdout gets an empty file. The contract
   wants the scene on stdout, so the arm's driver subclass sets `_file = sys.stdout`. Nothing else in
   the write path is touched.
2. **The input thread**, see "Textual busy-spins on an EOF stdin pipe" below.

Frame boundaries come from `Pilot.pause()`, whose last act is `screen._on_timer_update()` —
Textual's own frame commit. One call, one composite; measured at 125 composites for 120 frames
(the extra five are startup, see finding 9).

## Getting Textual to render to a pipe with no tty

Easier than expected, once the stderr surprise is out of the way. In order:

- **No tty is needed.** `LinuxDriver` already tolerates a non-tty: its `termios` calls are wrapped,
  `os.isatty(self.fileno)` guards the job-control handling, and `_request_terminal_sync_mode_support`
  returns early when stdin is not a tty. No patching required.
- **Size** comes from `COLUMNS`/`LINES` (the arm overrides `_get_terminal_size`) *and* from
  `run_async(size=(120, 40))`. There is no silent 80×24 path.
- **`run_test()` is the wrong entry point** and was rejected: its `finally` calls
  `app._shutdown()` — which calls `driver.close()`, stopping the writer thread — *before* awaiting
  the app task that runs `stop_application_mode()`. The alt-screen leave, cursor-show and kitty-flag
  reset are enqueued onto a dead thread and silently dropped. `run_async(auto_pilot=…)` orders these
  correctly and gives the same `Pilot`.
- **Textual replaces `sys.stdout` and `sys.stderr`** for the duration of the run
  (`redirect_stdout(self._capture_stdout)`). The contract's one stderr line is therefore written to
  `sys.__stderr__` *before* `run_async` is entered, and the driver's `_file` is captured at
  construction time, which is also before the redirect.
- **Textual busy-spins on an EOF stdin pipe.** `LinuxDriver.run_input_thread` selects on fd 0; at
  EOF the selector reports the fd readable forever, `os.read` returns `b""`, the loop `break`s out of
  the inner processing and immediately goes round again. That is a full core burned, which would have
  made the `cpu` scene meaningless. The byte and CPU scenes therefore park the input thread on the
  exit event; the `latency` scene reimplements `run_input_thread` with the same selector, the same
  `XTermParser` and the same `process_message`, differing only in that an empty read means EOF and
  ends the app.

## `NO_COLOR` — **honoured**, and tested rather than guessed

With `NO_COLOR=1` the stream contains **zero** colour SGR sequences:
`rg -c '38;2;|38;5;'` over the `full-repaint` stream (4 800 truecolor cells per frame otherwise)
returns 0, and every cell comes out as `\x1b[39;49m#\x1b[0m`.

Mechanism, and two things found while establishing it:

- `App.__init__` pops `NO_COLOR` out of the environment and appends a line filter: `NoColor` when
  the app is in native-ANSI mode, `Monochrome` otherwise.
- **The filter is chosen in `App.__init__`, before any theme is applied.** An app that switches to
  an `ansi` theme afterwards still gets `Monochrome`, which *greys* colours rather than removing
  them — the escape sequences stay, they just carry desaturated RGB. This arm passes
  `ansi_color=True` to the constructor so `NoColor` is selected, which actually strips colour. An
  arm author who only set `app.theme` would have reported "honoured" while still emitting 24-bit SGR.
- **`textual.filter.monochrome_style` crashes on a `Segment` whose style is `None`** —
  `AttributeError: 'NoneType' object has no attribute 'color'`, `textual/filter.py:62`. Reproducible
  with any custom `render_line` that returns unstyled segments, plus `NO_COLOR`. Worked around by
  giving every segment an explicit `Style()`. (Textual's own widgets always attach one, which is
  presumably why this has not been hit upstream.)

`no_color` in the stderr line is Textual's own answer (`App.no_color`), not a constant: it reads
`honoured` when `NO_COLOR` is set and Textual noticed, `ignored` if it did not.

Counter-intuitively, `NO_COLOR` makes four of the five scenes **bigger**, because `NoColor` writes an
explicit `\x1b[39;49m` where the default-colour stream needed no SGR at all.

## Alternate screen — **yes**

Prologue, 127 bytes, all of it Textual's own:

```
\x1b[?1049h \x1b[?1000h \x1b[?1003h \x1b[?1015h \x1b[?1006h \x1b[?25l \x1b[?1004h
\x1b[>25u \x1b[?2048$p \x1b[?2004h \x1b[?7l \x1b[?1000h \x1b[?1003h \x1b[?1015h
\x1b[?1006h \x1b]22;default\x07
```

Epilogue, 103 bytes, ending `\x1b[?1049l \x1b[?25h \x1b[?1004l` plus a second mouse-disable.
Mouse reporting is enabled because that is Textual's default (`mouse=True`); it was left on rather
than switched off, since a Textual user pays it.

`\x1b[?2048$p` is a *query*. stdin carries no reply on the picture scenes, so `_sync_available` stays
false and Textual emits **no** synchronised-update markers. On a real terminal it would, and every
frame would gain ~8 bytes. Worth knowing before anyone compares these numbers to a tty capture.

## Measurements

120 frames, 120×40, `TERM=xterm-256color`, stdout a pipe, `NO_COLOR` unset, three runs each.
"tuned" is the default; "naive" is `VITUI_ARM_TEXTUAL_IDIOM=naive` (see finding 1).

| scene | total (tuned) | frame 0 + startup | per frame | total (naive) | per frame (naive) |
|---|---:|---:|---:|---:|---:|
| `caret` | **14 396** | 11 195 | 21 / 29 | 27 744 | 134 |
| `status-line` | **21 252** | 15 372 | 46 | 32 381 | 141 |
| `list-scroll` | **625 358** | 15 372 | 5 124 | 625 358 | 5 124 |
| `full-repaint` | **13 546 622** | 333 108 | 111 036 | 13 546 622 | 111 036 |
| `modal-over-list` | **16 686** | 14 433 | 17 | 22 041 | 62 |

`caret` alternates 21 bytes (caret hidden) and 29 (caret shown, one extra SGR 7 / reset pair).
Totals include the 127-byte prologue and 103-byte epilogue.

**Repeated runs are byte-identical.** Three runs of each of the five scenes produced the same
SHA-256 and the same length. Nothing in the stream carries a clock, a pid or an id; frame *n*'s
picture is a pure function of *n*. No spread to report.

Other conditions, for the record:

| variant | `full-repaint`, 120 frames |
|---|---:|
| truecolor (what the arm reports) | 13 546 622 |
| 8-bit, i.e. `TEXTUAL_COLOR_SYSTEM=auto` with no `COLORTERM` | 9 092 890 |
| `NO_COLOR=1` | 7 651 582 |

`NO_COLOR=1`, all scenes: `caret` 15 785 · `status-line` 24 792 · `list-scroll` 683 186 ·
`full-repaint` 7 651 582 · `modal-over-list` 18 639.

`latency`, measured externally from write-to-stdin to first-byte-on-stdout, over two runs:
`a` 1.0–1.6 ms · `Up` (`\x1b[A`) 0.9–1.2 ms · `C-c` (`\x03`) 2.5–2.6 ms. The status line's first
field becomes `key a` / `key Up` / `key C-c`; the arm exits 0 at EOF. These are this machine's
numbers and are comparable only within one terminal, as the ticket says.

`cpu`, 10 seconds: 10.20 s wall, **0.42 s user + 0.05 s sys**, 600 composites — 60 Hz held exactly,
≈4.7% of one core. Interpreter and Textual startup are inside that figure, which is the point of
"Python counts"; at `--seconds 3` the same startup makes it ≈11%.

## Where the scene definitions did not fit Textual

The ticket predicted the "same scene" definition would break first. It did, in nine places.
Findings 1, 2 and 9 are Textual's model refusing to express what a scene describes; findings 3, 5, 6,
7 and 8 are under-specification in `SCENES.md` that will make arms incomparable until it is
tightened; finding 4 is both at once, and is the one that most needs a ruling.

### 1. Textual's smallest possible update is a *widget*, not a cell — and the licence to "reach the picture any way the framework prefers" is worth 5× a frame here

`ChopsUpdate.render_segments` (`textual/_compositor.py`) renders in *chops*, and a chop is the span
of a line between widget boundaries. For a dirty span `[x1, x2)` inside a chop starting at `x` it
does `strip.crop(0, min(end, x2) - x)` and moves to `x` — **always from the widget's left edge**. A
one-cell change in the middle of a 120-cell widget is therefore never a one-cell update; it is
"everything from the widget's left edge up to the right edge of the change".

Consequence: the only way for Textual to spend one cell on the caret is for the caret to *be a
one-cell widget*. That is what this arm does — row 0 is a 21-cell label widget and a 1-cell caret
widget, so the frame is 21 bytes. Put the same picture in one full-width widget and refresh it, which
is what a `Static` plus a reactive does and therefore what most Textual code costs, and the frame is
134 bytes and the total goes from 14 396 to 27 744.

**This matters for the whole suite.** `SCENES.md`'s licence is a genuine licence, but on `caret` it
does not select between two implementations of one idea — it selects between 21 bytes a frame and
134, a 5× spread that the arm author picks and the scene definition does not constrain. The number
reported here is **Textual hand-tuned**, and the report should say so rather than let it read as
"what Textual costs". Both columns are in the table above for exactly that reason.

### 2. "The terminal's default colours" is not a thing Textual has, outside two themes

Every Textual theme paints `$surface`/`$background`, so the default app fills all 4 800 cells with an
explicit RGB background and the `caret` scene's "rows 1..39 are blank" becomes 4 800 coloured cells.
Only `ansi-dark` and `ansi-light` set `foreground`/`background` to `ansi_default`. The arm selects
`ansi-dark` and passes `ansi_color=True`; blank rows then cost `\x1b[49m` and nothing else.

Note that this has to be done by assigning `app.theme` (the reactive's watcher is what re-applies the
stylesheet); `set_reactive` looks equivalent and silently leaves the themed background in place — a
5 924-versus-5 484-byte difference on the first frame that is easy not to notice.

### 3. The 24-bit ramp is not reachable under the declared `TERM`, and the picture is wrong if you let it slide

`SCENES.md` scene 4 is "a 24-bit ramp", `rgb(c*2, r*6, 128)`. rich (and so Textual) selects **8-bit**
under `TERM=xterm-256color` unless `COLORTERM` says `truecolor`, and the harness sets `TERM` but not
`COLORTERM`. 8-bit does not merely cost fewer bytes — it *quantises the ramp*: columns *c* and *c+1*
differ by 2/255 in red and collapse onto the same palette entry, so adjacent columns become
identical and the arm is drawing a different picture from the one specified.

The arm therefore sets `TEXTUAL_COLOR_SYSTEM=truecolor` — Textual's own documented override — with
`setdefault`, so an explicit harness value still wins. Both numbers are recorded above (13.5 MB
truecolor, 9.1 MB 8-bit).

**`SCENES.md` should say whether the declared tier includes `COLORTERM`.** As written, the tier
decides a 4.5 MB difference on one row *and* whether the row is the specified picture at all, and it
decides it by omission.

### 4. `modal-over-list`: "mixed halfway toward black" cannot be computed over `ansi_default`

Scene 5's base layer is "scene 3's screen" — which is in the terminal's *default* colours — with
every cell "mixed halfway toward black". The arm does not know the terminal's default foreground and
background, and inventing them would be inventing the picture. Textual *can* blend a translucent
`ModalScreen` over a background screen, but only where the colours behind are known RGB.

What the arm does: the list layer carries SGR 2 (`Style(dim=True)`), which is the terminal's own
notion of halfway-toward-black, and the dialog is undimmed. The *layering* is real — a Textual
`ModalScreen` with `background: transparent` pushed over the list screen, `align: center middle`
placing the 40×12 dialog at column 40, row 14, and the dialog's cells coming out of Textual's
compositor over a background screen.

**This row is not comparable until `SCENES.md` picks one.** An arm that mixes RGB (which is what an
engine holding real cell colours would naturally do) emits per-cell truecolor SGR for 4 800 cells;
this arm emits `\x1b[2m`. Same described picture, two byte counts that are not in the same
neighbourhood. This is a second flavour of the notcurses problem: not "cannot express", but "expresses
something that looks the same and costs differently".

### 5. The status line's field widths are unspecified, and the picture moves

`frame 0` → `frame 119` is two characters longer, which pushes `elapsed …`, `cpu 12%` and `3 tasks`
right; at *n*=10 and again at *n*=100 the whole line shifts. `SCENES.md` says only that `frame 0`
becomes `frame n`. This arm substitutes literally, without padding, and refreshes exactly the columns
that differ (which is the whole line on the two shift frames).

An arm that pads to `frame %3d` draws a different picture and pays a smaller, constant update.
`SCENES.md` should pin the width, or state that the shift is intended.

### 6. The list label is 19 characters, not the 20 the prose claims

Scene 3: "a 20-character label `item-NNNNN---------`". The literal is `item-` (5) + 5 digits + 9
hyphens = **19**. The arm uses 20 (`item-NNNNN` plus ten hyphens), because a fixed width is what
makes the row comparable at all, and flags the discrepancy here. One character is 40 rows × 120
frames = 4 800 cells of difference on this row, so this is not cosmetic.

### 7. Three literals the scenes describe but do not name

Recorded so every other arm can copy them exactly:

- the 64-character lorem string:
  `lorem ipsum dolor sit amet consectetur adipiscing elit sed diam.`
- the dialog's four body rows (each ≤ 38 cells, the dialog's interior width):
  `compositing a dialog over the list` / `only the spinner cell moves per frame` /
  `everything behind here is dimmed` / `the list does not scroll`
- where the dialog's *other* five interior rows go. A 40×12 dialog with a single-line border has ten
  interior rows; the title row and four body rows fill five. The arm leaves rows 6..10 blank at the
  bottom.

### 8. "A block caret" has no colour-free spelling in a framework's own vocabulary

Textual's own block cursor is `$block-cursor-background` / `$block-cursor-foreground` — themed
colours, which would contradict "in the terminal's default colours" in the same paragraph. The arm
draws SGR 7 (reverse) on a space, which is the terminal's own block caret. An arm that reaches for
its framework's cursor styling draws a different picture and pays more SGR.

### 9. Textual paints the initial screen four times before it goes idle, and on `caret` that *is* the number

Startup produces five composites, four of which carry bytes: two full-screen layout updates
(5 484 bytes each on `caret`) and two small chop updates. On the `caret` row that is **11 195 of the
14 396 total — 78% of the scene is Textual's boot**, and `SCENES.md` calls this scene "the floor …
the closest thing to a fixed overhead this suite can measure". For Textual, it measures startup.

Nothing is subtracted: the harness discards the mode-setting prologue, which it can see, and frame 0
is a frame. But the report will be read wrongly unless it either carries a separate "first frame"
column or states that frame 0 includes framework startup.

## Deliberate choices a reader should not mistake for accidents

- **The latency number includes Textual's 60 Hz batching.** Textual commits frames on a screen-update
  timer, so a keystroke can wait up to 16.7 ms for its bytes. The arm does *not* force an immediate
  composite from the key handler, even though it could reach into `screen._on_timer_update()` to do
  so, because Textual's own event loop is what its users pay. The measured 1.2–2.6 ms is well inside
  one tick, so in practice the keystroke usually arrives early in a window rather than waiting a full
  one.
- **`cpu` is driven by Textual's own `set_interval(1/60, …)`**, not by a hand-rolled sleep loop, for
  the same reason. It stops on a wall-clock deadline read inside the tick; reading the clock is
  forbidden on the *picture* scenes and is the definition of this one.
- **Mouse reporting stays on** and its sequences are in the prologue, because that is Textual's
  default.
- **Every scene is drawn with `render_line`**, Textual's own idiom for cell-exact content (its
  `DataTable`, `OptionList` and `RichLog` all use it). This is what makes the described pictures
  reachable without a theme's colours leaking in; it is not a bypass of Textual's rendering, which is
  the same `Strip` → `Segment` → compositor path every widget takes.

## Invocation

```sh
./setup.sh
COLUMNS=120 LINES=40 TERM=xterm-256color ./arm.py --scene caret --frames 120 > frames.bin
COLUMNS=120 LINES=40 TERM=xterm-256color ./arm.py --scene cpu --seconds 10 > /dev/null
COLUMNS=120 LINES=40 TERM=xterm-256color ./arm.py --scene latency < keystrokes
VITUI_ARM_TEXTUAL_IDIOM=naive ./arm.py --scene caret --frames 120 | wc -c   # finding 1
```
