# The ratatui arm — what it took, and what the scenes did not say

Everything below was measured, not recalled. Where a sentence makes a claim about ratatui or
crossterm it names the file it came from, and where it makes a claim about a byte count it is a
count this arm actually produced.

## Versions, pinned exactly

| crate | version | why it is here |
|---|---|---|
| `ratatui` | **0.30.2** | the framework under measurement (current on crates.io, 2026-08-22) |
| `ratatui-crossterm` | **0.1.2** | 0.30 split the backends out of the facade; this is `CrosstermBackend` |
| `crossterm` | **0.29.0** | what `ratatui-crossterm` 0.1.2 selects via its `crossterm_0_29` default |

Reported on stderr as `version=0.30.2`, or `version=0.30.2+scrolling-regions` when the arm is built
with its one optional feature (see *Scroll regions* below).

Measured on: Apple M1 Max (MacBookPro18,2, arm64), macOS 26.5.2, rustc 1.97.1 (Homebrew), release
profile with `lto = "thin"`.

## `no_color=honoured` — measured, and with a caveat worth a line in the report

**Honoured, and by crossterm rather than by ratatui or by this arm.**
`crossterm-0.29.0/src/style/types/colored.rs` has `Colored::ansi_color_disabled()`, memoized behind
a `Once`, and `impl Display for Colored` returns early writing nothing when it is true. It also gets
the [no-color.org](https://no-color.org/) rule right: the variable must be **non-empty**, so
`NO_COLOR=` is correctly ignored. Verified all three ways:

| `NO_COLOR` | `full-repaint`, 5 frames |
|---|---|
| unset | 531 102 |
| `1` | 121 472 |
| `` (empty) | 531 102 |

**The caveat.** `Colored::fmt` writes nothing, but the enclosing `SetForegroundColor` command still
writes its own `ESC [` and `m` — so every suppressed colour becomes a five-byte **`ESC [ ; m`** on
the wire instead of disappearing. On the ramp that is 4 800 empty SGRs a frame: 2 914 822 bytes over
120 frames where the honest floor is about 4 900 a frame. So ratatui's `NO_COLOR` number is neither
"the colours cost this much" nor "the colours are gone" — it is 23% of the coloured number and still
almost entirely SGR.

Worse than a wasted byte: `ESC [ ; m` is an SGR with an empty parameter, which terminals read as
**`0` — reset everything**. `ratatui-crossterm` emits the modifier diff *before* the colours for a
cell, so a cell that both turns reverse-video on and changes colour will have its `ESC [ 7 m`
cancelled by the colour command's `ESC [ ; m` under `NO_COLOR`. It does not bite the five scenes here
(the reverse-video rows never change colour in the same cell), but it means an arm's `NO_COLOR` run
can be a *different picture*, not just a cheaper one.

## `alt_screen=yes`

`ratatui::init()` enters the alternate screen, so the arm does. It is written to the pipe, not to a
tty: `execute!(io::stdout(), EnterAlternateScreen)` before the `BufWriter` the backend owns is
created, `LeaveAlternateScreen` after that writer has been flushed and dropped.

The visible prologue is `ESC [ ? 1049 h` — 8 bytes. The epilogue is `ESC [ ? 1049 l` — 8 bytes —
preceded by `ESC [ ? 25 h` when the last frame left the cursor hidden, because **`Terminal`'s `Drop`
shows the cursor if it hid it**. So the epilogue is 8 or 14 bytes depending on the scene's last
frame, which is worth knowing if the harness strips a fixed prologue length.

`enable_raw_mode` is **deliberately not called**, even though `ratatui::init()` calls it. See the
next section: crossterm implements raw mode against `/dev/tty`, so calling it here would succeed and
reconfigure the developer's real terminal while the arm measures a pipe.

## Writing to a pipe: what it took

Two things, and only the second one was interesting.

**1. The size comes from `COLUMNS`/`LINES`, and `Viewport::Fixed` is the arrangement that imposes
it.** No wrapper backend was needed. `Terminal::with_options` asks `Backend::size()` only for
`Viewport::Fullscreen` and `Viewport::Inline` (`ratatui-core-0.1.2/src/terminal/init.rs:122`), and
`Terminal::autoresize` skips `Fixed` outright (`.../terminal/resize.rs:64`). So
`Viewport::Fixed(Rect::new(0, 0, 120, 40))` never lets ratatui measure anything, and at the origin
and at full size it emits byte-for-byte what `Fullscreen` would — the two differ only in what
`Terminal::clear` does, and this arm never calls it.

**2. `crossterm::terminal::size()` is a trap, and it is exactly the one ARM-CONTRACT warns about.**
It does not fail without a tty. It opens `/dev/tty` and falls back to `STDOUT_FILENO` only if that
fails, so run from an interactive shell with stdout on a pipe it returns **the developer's real
window size** — a silently wrong answer rather than an error. An arm that "takes the size from
crossterm and falls back to `COLUMNS`" would therefore never reach the fallback and would measure a
different terminal on every machine. The contract's sentence about not falling back to 80×24 is not
the whole hazard; the hazard is that the query *succeeds*.

## Byte counts, 120 frames, 120×40, `NO_COLOR` unset

Totals include the 8-byte alt-screen prologue and the 8–14-byte epilogue. Two runs of each produced
identical counts.

| scene | 1-frame run, total | 120 frames | one steady frame |
|---|---|---|---|
| `caret` | 94 | **3 488** | 25 (hide) / 32 (show) |
| `status-line` | 6 115 | **11 647** | 46–47 |
| `list-scroll` | 1 686 | **85 542** | 704 |
| `full-repaint` | 106 238 | **12 745 942** | 106 216 |
| `modal-over-list` | 5 792 | **10 076** | 36 |

The "one steady frame" column is a difference of two runs corrected for the epilogue, not a mean:
`caret`'s two frame kinds cost different amounts and the epilogue length depends on which one ran
last, so a mean would hide both facts.

With `NO_COLOR=1`: `caret` 2 768 · `status-line` 10 927 · `list-scroll` 84 822 · `full-repaint`
2 914 822 · `modal-over-list` 8 963. Note `modal-over-list` under `NO_COLOR` is **not the scene's
picture** — the dim is colour and nothing else, so it vanishes.

`cpu`, 10 seconds at 60 Hz: 600 frames, 10.01 s wall, **0.03 s user + 0.01 s sys = 0.4% of one
core**. `latency` with `a`, `ESC [ A`, `ETX` fed in: 6 241 bytes total, initial screen 6 115 of it,
and the three updates 49, 38 and 39 bytes — a cursor move, `ESC [ 7 m`, the new text, the trailer.
Output was confirmed to reach the pipe **before** stdin closed (fed one byte, waited a second, read
6 150 bytes with the writer still open), so nothing is held back waiting for EOF.

### Two things in those numbers that are about ratatui, not about the scenes

**Every `draw()` pays a 25-byte trailer whether or not anything changed.** `caret`'s odd frames
change nothing at all and still cost exactly 25 bytes: `ESC [ 39 m`, `ESC [ 49 m`, `ESC [ 59 m`,
`ESC [ 0 m`, `ESC [ ? 25 l`. `ESC [ 59 m` is the *underline-colour* reset and comes from ratatui's
default `underline-color` feature; turning that off would save 5 bytes a frame. It is left on because
it is the default, and the arm measures the default. That 25 bytes is ratatui's floor on this suite:
whatever the picture, an unchanged frame is not free.

**The diff is per cell, not per row — and the cursor moves cost more than the text.** `list-scroll`
costs 704 bytes a frame, not forty rewritten rows, because scrolling by one changes only the digits:
`ESC [ 1 ; 5 H 1 ESC [ 1 ; 17 H 1 ESC [ 2 ; 5 H 2 …`. Counted on frame 1: **80 cursor moves, six to
eight bytes each, 582 of the 704 bytes**, to write about 80 characters. The diff will happily pay a
seven-byte `MoveTo` to avoid rewriting two unchanged spaces. This is a much better number than "it
has no scroll region" would suggest, and the report should say so rather than let the 85 KB stand as
if it were forty full rows.

## Scroll regions: the feature exists and changes nothing

`ratatui` 0.30 has an opt-in `scrolling-regions` feature that adds `Backend::scroll_region_up` /
`scroll_region_down`, and `ratatui-crossterm` implements them. **Building the arm with it on produces
byte-identical output on all five scenes** (`list-scroll` 85 542 either way).

The reason is that nothing in the frame path uses them. The only consumers in `ratatui-core` are
`terminal/inline.rs:255,273,286` — `Terminal::insert_before` for the *inline* viewport. The
double-buffer diff has no scroll detection and no API through which an application could say "the
list scrolled by one", so a fullscreen ratatui app cannot reach a scroll region at all. The feature
is kept in `Cargo.toml`, off by default, so that this can be re-checked rather than re-argued.

---

# What did not fit ratatui's model

The ticket predicted the "same scene" definition would break first. It did, in eleven places. Nine
are `SCENES.md` under-specifying the picture; two are ratatui genuinely unable to express it.

## The two that are ratatui's limits

**1 — `modal-over-list` has no layers, so the dim is computed by hand.** Expected, and `SCENES.md`
licenses it. The arm draws the list, runs a `Dim` widget over the whole buffer that rewrites every
cell's colours, then draws the dialog over that. `Clear` is what makes the dialog *undimmed*: it
resets the cells the dim pass just wrote.

The part that was not expected is what the dim costs in the *measurement*. After dimming, **no cell
on the screen is `Color::Reset` any more**, so every one of the 4 800 cells differs from the buffer
ratatui starts from and every one of them is written: scene 5's frame 0 is **5 792 bytes against
scene 3's 1 686** for the same list, and the difference is entirely the trailing spaces that were
free while they were default. The SGR itself is cheap — there are only two styles on the screen, so
crossterm emits the colour pair once per run, not once per cell. A framework that *did* composite a
translucent layer would pay the same, because the terminal has no alpha and the dimmed spaces still
have to be sent. So this does not disadvantage ratatui — but it does mean scene 5's frame-0 number is
set by an arm's *declared default colours* (below), which is a strange thing for a comparison to turn
on.

The reverse-video row forced a second decision. A cell that is reverse-video cannot simply have its
colours halved: the terminal will swap the halved pair, which is not halfway to black. The dim pass
therefore resolves `REVERSED` — swap fg and bg, then halve, then drop the modifier. **There is no way
to say "reverse, dimmed" and mean it**, in ratatui or in the terminal. Any arm that keeps the
modifier and halves the colours draws a different picture, and it will look plausible.

**2 — `caret`'s "block" caret is unreachable.** `SCENES.md` says *a block caret*. ratatui's frame API
carries a cursor *position* and nothing else; crossterm has `SetCursorStyle::SteadyBlock`, but
ratatui never emits it and there is no `Frame` method that would. The arm shows the terminal's
default cursor shape and does not emit a shape escape. Costs nothing (or ~6 bytes once, if an arm
decides to emit `ESC [ 2 SP q`), but the adjective in the scene is not something every arm can honour
and two arms will disagree by that one sequence.

## The nine places `SCENES.md` left the picture open

Ordered by how much the byte count moves if a second arm chooses differently.

**3 — Scene 2's field widths are unstated, and it is worth 10× on that row.** `SCENES.md` fixes the
frame-0 literal and says the later fields "never change", which implies fixed columns but does not
say how wide the first field is. `frame 0` is seven cells and `frame 119` is nine. This arm pads the
first field to twelve — `format!(" {first:<12}elapsed …")`, so `elapsed` starts at column 13 on every
frame — and a frame therefore costs **46 bytes**. An arm that pads to nine is equally compliant with
every sentence in the scene and gets a different number; an arm that lets the tail shift rewrites
about half the line, roughly 500 bytes a frame. **This is the single most consequential omission in
the file.** It should name the columns.

**4 — The 64-character lorem string is not given, only its length.** It sets 39 rows × 64 cells of
frame 0, so it decides scene 2's frame-0 number outright (6 115 bytes here) and nothing after it.
Two arms will pick two strings. `SCENES.md` should paste the literal.

**5 — Scene 3's label length contradicts itself.** The prose says *a 20-character label*; the
illustration `item-NNNNN---------` is **19** characters. The arm followed the prose and pads
`item-NNNNN` with dashes to 20. An arm that copies the literal is one cell narrower on all forty
rows. Pick one.

**6 — Scene 5's dim has no starting colour to be halfway from.** Scene 3 is "the terminal's default
colours", and "mixed halfway toward black" needs a number. **The default is not knowable from inside
the arm** — it is the emulator's, and the emulator is not there. This arm declares fg =
`rgb(192,192,192)` and bg = `rgb(0,0,0)`, so a plain cell dims to `rgb(96,96,96)` on `rgb(0,0,0)` and
the reverse-video row to `rgb(0,0,0)` on `rgb(96,96,96)`. Every arm must declare something and no two
will agree. The scene should state the two triples, or say "dim relative to a declared default and
report the declaration".

**7 — Scene 4 does not say whether the ramp is foreground or background.** "every one of the 4 800
cells holds a `#`, coloured from a 24-bit ramp". The arm colours the **foreground**. A background
ramp costs the same (`48;2;` for `38;2;`); an arm that sets both roughly doubles the row, which is
the ceiling row and the one meant to isolate encoding. Say which.

**8 — Scene 5's dialog: is the title row *in* the border or *under* it?** "a single-line border, a
title row reading `⠋ working`, and four rows of body text" enumerates three things, so the arm gives
the title its own row inside the border — border on screen row 14, title on 15, body on 16..19,
bottom border on 25. ratatui's own idiom is the opposite: `Block::bordered().title("⠋ working")`
paints the title *into* the top border line. Both are 12 rows tall and both cost ~36 bytes a frame,
so this one is cheap, but it is the sort of thing where "the same picture" quietly stops being the
same picture.

**9 — Scene 5's four rows of body text have no content.** Same shape as the lorem string: frame 0
only. The arm invented four lines.

**10 — Scene 5 says nothing about the dialog's other five inner rows.** Twelve rows minus two borders
is ten inner rows; one title plus four body is five. The arm leaves the remaining five blank at the
dialog's own undimmed default, so they read as part of the dialog rather than as a hole in it.

**11 — `latency` does not say what "first output byte" means.** The measured quantity is "the byte
entering the arm's stdin to the arm's first output byte after it". ratatui's `draw` writes the cursor
move first, so the arm's first byte after a keystroke is the `ESC` of `ESC [ 40 ; 2 H`, and the first
byte that changes what the user sees arrives twelve bytes later (`ESC [ 40 ; 2 H` then `ESC [ 7 m`).
A framework that happened to emit its changed cell first would look faster by that much, for no
reason a user could perceive. The definition is fine as
long as every arm is measured the same way, but it is measuring *time to first byte of the reply*,
not *time to first visible change*, and the report should say which.

## Two smaller notes

The caret scene's title contains an em dash (U+2014), so the floor scene has a UAX #11 width
dependency inside it. It is one cell wide in ratatui (`unicode-width`) and the arm's output confirms
it: `ESC [ 1 ; 15 H` writes the dash, and the next write is at column 17, having skipped one space.

Scene 2's `elapsed` field is `n / 60`, derived from the frame index and never from a clock, which is
what makes the run byte-identical twice over. That is the contract's rule and it is easy to violate
by reaching for `Instant` in the one scene that displays a duration.
