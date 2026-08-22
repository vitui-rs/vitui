# The notcurses arm

Everything this arm decided that [`SCENES.md`](../../SCENES.md) and
[`ARM-CONTRACT.md`](../../ARM-CONTRACT.md) did not, plus the two findings that
contradict the ticket that asked for it.

**Version targeted: notcurses 3.0.17**, the current release and the current tip
of `master` (2026-08-22). Every source citation below is a file and line in that
tag; the arm links `notcurses-core`, not `notcurses`.

---

## The headline: notcurses composites layers, and scene 5's `cannot express` is wrong

The ticket says:

> **notcurses is the honest ceiling, and it does not composite layers.** The
> layered scenes have no notcurses arm and that cell reads **"cannot express"**,
> never blank.

The second half of the first sentence is false. notcurses has planes, they stack,
and it has exactly the operator the ticket says it lacks: **`NCALPHA_BLEND`**,
resolved by the renderer, per cell, over the plane stack.

### The mechanism

`ncpile_render_internal()` (`src/lib/render.c:1496`) walks the pile from
`p->top` downward through `pl->below`. For each plane, `paint()`
(`src/lib/render.c:236`) contributes to the accumulating target cell only while
that cell's alpha is above `NCALPHA_OPAQUE`, and the contribution goes through
`channels_blend()` (`src/lib/internal.h:1342`):

```c
unsigned r = (r1 * *blends + r2) / (*blends + 1);
```

— a running mean, where `r1` is what the planes above have already accumulated
and `r2` is this plane's colour. `channels_blend()` then sets the target's alpha
to *this plane's* alpha, so a `BLEND` plane leaves the cell open and an `OPAQUE`
plane closes it.

So a full-screen plane above the list, whose base cell is `rgb(0,0,0)` with
`NCALPHA_BLEND` on both channels, gives:

1. dim plane reached first, `blends == 0` → target becomes black outright,
   `blends` becomes 1, alpha stays `BLEND`;
2. list plane reached second, `blends == 1` → `(0 * 1 + r2) / 2` = **`r2 / 2`**,
   alpha becomes `OPAQUE`, iteration stops.

Halfway toward black. And glyphs survive it: the dim plane's base cell has
`gcluster == 0`, so at `src/lib/render.c:366` `targc->gcluster` comes out 0,
`crender->p` stays NULL, and the glyph search continues into the plane below.
The caller computes no colour anywhere.

### The measurement

`--scene modal-over-list-blend` is that picture, built from three planes, and it
runs. The list is drawn `fg=rgb(192,192,192)` on `bg=rgb(0,0,0)`, the highlight
row `fg=rgb(0,0,0)` on `bg=rgb(255,255,255)`, the dialog
`fg=rgb(224,224,224)` on `bg=rgb(32,32,48)`. Every distinct colour on the wire
for frame 0:

```
 14  ESC[38;2;96;96;96m      <- 192 dimmed
 14  ESC[48;2;0;0;0m
 12  ESC[38;2;224;224;224m   <- dialog, undimmed
 12  ESC[48;2;32;32;48m
  1  ESC[38;2;0;0;0m
  1  ESC[48;2;127;127;127m   <- 255 dimmed
```

192 → 96. 255 → 127. Exactly halfway, truncating. The dialog's colours come out
untouched. The list's glyphs show through. This is compositing, done by
notcurses' compositor.

### The two candidates the brief asked about, and why they are not it

- **`ncplane_stain()`** (`notcurses.h:2726`) *replaces* the channels of a region.
  It is the "caller pre-computes the dimmed colour" route, it destroys the
  original colours in place, and it is not compositing. Correctly identified.
- **`ncplane_format()`** (`notcurses.h:2715`) sets style bits only. No colour
  involvement at all.

If either of those had been the only route, the `cannot express` cell would have
been right. `NCALPHA_BLEND` is a third thing, and it is a real compositor.

### What this arm nevertheless does

`--scene modal-over-list` **refuses**: exit status **69**, `cannot express` on
stderr, zero bytes on stdout. `SCENES.md` is normative and the harness is being
wired to that behaviour, so the arm conforms to it rather than quietly
contradicting it. The blend scene is a **separate, opt-in scene name** and is
**not** one of the suite's five — the harness must not run it as
`modal-over-list`. It exists so the correction above is demonstrable rather than
asserted.

The decision to flip the report cell from `cannot express` to a number is the
map's, not this arm's. When it is flipped, note the deviation below.

### One deviation in the blend scene, and it is notcurses being honest

The blend scene draws the list in explicit RGB rather than scene 3's terminal
defaults. With no tty notcurses never learns the terminal's real default
foreground — there is no OSC 10 reply to read — so `tcache.fg_default` keeps its
initial `0xff000000` and `channels_blend()` resolves "default" to `rgb(0,0,0)`.
Dimming a default-coloured cell halfway toward black therefore yields black on
black: arithmetically correct, visually nothing. **Scene 5's dim is only
well-defined over cells with explicit colours**, and every arm will hit this,
because "mix the terminal default halfway toward black" has no answer that does
not involve guessing the terminal's palette.

---

## Rendering to a pipe with no tty

notcurses is not merely tty-preferring; it is tty-*seeking*. Getting it onto a
pipe took three decisions, and one of them is the difference between an arm and
a hang.

### 1. The arm sheds its controlling terminal before touching notcurses

`get_tty_fd()` (`src/lib/fd.c:455`) checks whether the output `FILE*` is a tty
and, when it is not, **falls back to `open("/dev/tty")`**. If that succeeds,
notcurses proceeds as though it owned a terminal:

- geometry from `TIOCGWINSZ` on that tty rather than from `COLUMNS`/`LINES`
  (`update_term_dimensions()`, `src/lib/notcurses.c:319`);
- capability queries written to whatever terminal the caller happens to be
  sitting in (`send_initial_queries()`, gated on `ttyfd >= 0`);
- that terminal put into cbreak mode and written to;
- and `interrogate_terminfo()` → `handle_responses()` →
  `inputlayer_get_responses()` **blocks** waiting for a Device Attributes reply.

That last one is not hypothetical. Run this arm with the tty-shedding disabled,
inside `script -q /dev/null` with stdout redirected to a file, and it **hangs
indefinitely** — measured, killed at two minutes. With the shedding in place the
same invocation exits 0, addresses 40 rows (from `LINES=40`, not the pty's size),
and writes **zero bytes to the pty**.

With no controlling terminal, `ttyfd` stays −1 and every one of those paths is
skipped. No queries are sent, no responses awaited, and
`get_default_geometry()` (`src/lib/termdesc.c:73`) fills the size from the
**`LINES` and `COLUMNS` environment variables** — which is precisely the
contract's declared size source, arrived at by accident rather than by design.

`shed_controlling_tty()` does the standard dance: `setsid()` if we are not a
process-group leader (the usual case under a harness that spawns us directly),
otherwise `fork()` and `setsid()` in the child, with the parent relaying the
child's exit status verbatim. `TIOCNOTTY` was considered and rejected — its
behaviour for a non-session-leader varies between kernels and it is not POSIX.

> **Harness consequence, and please read this one.** When the fork branch is
> taken, the process the harness spawned is a relay and the rendering happens in
> its child. CPU time must therefore be measured with `wait4`/`waitpid` plus
> `getrusage(RUSAGE_CHILDREN)`, which accumulates descendant time up the chain,
> **not** by sampling `/proc/<pid>/stat` of the direct child. Python's
> `os.wait4()` and `resource.getrusage(RUSAGE_CHILDREN)` are both fine. A harness
> that spawns the arm with `start_new_session=True` (or `setsid(1)`) avoids the
> fork entirely, because then `setsid()` succeeds on the first try — that is the
> cleanest thing the harness can do, and it costs it nothing.

### 2. Output goes through the ordinary render path, to `stdout`

`ncpile_render_to_buffer()` looks like the right call for a pipe and **is broken
in 3.0.17**. `src/lib/render.c:1597` calls `ncpile_render()` and then
`notcurses_rasterize_inner()` while skipping `postpaint()` — but `postpaint()` is
what sets `crender->s.damaged` and updates `nc->lastframe`, and
`rasterize_core()` (`src/lib/render.c:1114`) emits nothing for an undamaged
cell. Since `engorge_crender_vector()` re-runs `init_rvec()` on every render,
every cell is undamaged, and the returned buffer is essentially empty. It also
hands back a pointer into `nc->rstate.f.buf` after an `fbuf_reset()` that only
zeroes `used`, so the documented "must be freed by the caller" would be a free of
notcurses' own live buffer. Unfixed on `master` as of this writing.

So the arm passes **`stdout`** to `notcurses_core_init()` and uses
`notcurses_render()`. That is the path a real notcurses application takes, all
output goes out through `write(2)` (`fbuf_flush()` in `src/lib/fbuf.h:302`
`fflush`es the `FILE*` and then `write`s), and there is no stdio interleaving to
worry about and no flush to add after a render.

### 3. The options, and why each one is load-bearing

```c
opts.loglevel = NCLOGLEVEL_SILENT;
opts.flags = NCOPTION_SUPPRESS_BANNERS
           | NCOPTION_NO_ALTERNATE_SCREEN
           | NCOPTION_NO_CLEAR_BITMAPS
           | NCOPTION_NO_FONT_CHANGES
           | NCOPTION_NO_WINCH_SIGHANDLER
           | NCOPTION_NO_QUIT_SIGHANDLERS
           | NCOPTION_INHIBIT_SETLOCALE;
opts.flags |= NCOPTION_DRAIN_INPUT;   // every scene except latency
```

- **`loglevel` must be set explicitly.** `NCLOGLEVEL_SILENT` is **−1**, so a
  zeroed `notcurses_options` means `NCLOGLEVEL_PANIC`, and a panic goes to stderr
  in breach of the contract's one-line rule.
- **`SUPPRESS_BANNERS` is mandatory**, not cosmetic: `init_banner()` writes into
  the output fbuf, i.e. to **stdout**, which would be a banner in the middle of
  the measured bytes.
- `NO_ALTERNATE_SCREEN` matches reality — see below.
- `INHIBIT_SETLOCALE` because the arm sets the locale itself.
- `DRAIN_INPUT` for the scenes that never read stdin, so notcurses does not
  accumulate it.
- The harness must **not** set `NOTCURSES_LOGLEVEL`: `set_loglevel_from_env()`
  overrides the option and the logs land on stderr.

### 4. Locale

notcurses refuses to start on a locale that is neither UTF-8 nor ASCII
(`notcurses_early_init()`, `src/lib/notcurses.c:1210`), and an ASCII locale
cannot render scene 1's em dash or scene 5's braille spinner. The arm tries
`setlocale(LC_ALL, "")`, then `C.UTF-8`, then `en_US.UTF-8`, and exits **71**
with a plain message if none of them yields UTF-8. CI needs one of those
available.

---

## `NO_COLOR`: `unsupported`

The string `NO_COLOR` does not occur anywhere in the notcurses 3.0.17 source
tree — case-insensitive grep over the whole tarball, zero hits. It is not that
notcurses ignores the declaration; it has no concept of it, which is the
contract's third answer and not a synonym for either of the first two.

Confirmed on the wire: `full-repaint` at 2 frames is **183 648 bytes with
`NO_COLOR` unset and 183 648 bytes with `NO_COLOR=1`**. Byte-identical.

## Alternate screen: `no`, and not by choice

`notcurses_enter_alternate_screen()` (`src/lib/notcurses.c:31`) returns −1
immediately when `tcache.ttyfd < 0`, which it always is here. The arm passes
`NCOPTION_NO_ALTERNATE_SCREEN` so that notcurses does not additionally emit a
`clear_and_home()` prologue it cannot pair with an `smcup`, but the flag is
recording a fact, not making a choice. **notcurses cannot enter the alternate
screen on a pipe.** The harness's "discard the enter/leave prologue only when it
can see it" rule therefore has nothing to discard for this arm, and the one-off
cost every other arm pays is one this arm did not.

The prologue and epilogue notcurses does emit on stdout, in full, for the record:

```
prologue: ESC[39;49m ESC(B ESC[m ESC[?25l
epilogue: ESC[39;49m ESC(B ESC[m ESC[?1l ESC> ESC[?12l ESC[?25h
```

## `COLORTERM` is missing from the contract, and it is worth 39% of scene 4

`ARM-CONTRACT.md` fixes `TERM=xterm-256color` and says nothing about
`COLORTERM`. But `TERM` alone does not declare 24-bit colour: notcurses'
`query_rgb()` (`src/lib/termdesc.c:177`) wants terminfo's `RGB`/`Tc` — which
`xterm-256color` does not carry — or `COLORTERM` in `{truecolor, 24bit}`.
Without one, `term_fg_rgb8()` (`src/lib/render.c:743`) **quantizes into the
256-colour palette**, so scene 4's ramp is neither the described picture nor a
comparable byte count.

Measured, `full-repaint` at 2 frames:

| `COLORTERM` | bytes | encoding |
|---|---:|---|
| `truecolor` | 183 648 | `38;2;r;g;b` |
| unset-equivalent | 111 708 | `38;5;N`, quantized |

This arm sets `COLORTERM=truecolor` **only if the harness has not set it**, which
is the least bad of the available wrongs. **The real fix belongs in
`ARM-CONTRACT.md`: fix `COLORTERM` for every arm.** ratatui-through-crossterm
emits truecolor unconditionally, Textual consults `COLORTERM`, and notcurses
consults `COLORTERM`, so left unspecified the arms will diverge on the one scene
whose entire purpose is to measure encoding with nothing else in the way.

---

## Where the scene definitions did not fit

Six of these. The ticket predicted the "same scene" definition would break
first; it did.

### 1. "Reverse video" is not expressible in notcurses

notcurses has no reverse-video attribute — `NCSTYLE_*` is
`{ITALIC, UNDERLINE, UNDERCURL, BOLD, STRUCK}` and `notcurses.h:767` says
outright "if you want reverse video, try `ncchannels_reverse()`". And
`ncchannels_reverse()` (`notcurses.h:401`) swaps the two channels wholesale, so
on a pair of *default* channels it swaps default for default and changes
nothing.

Scenes 2 and 3 both call for reverse video over default colours. The only way to
that picture is to name concrete colours, so the arm uses **`rgb(0,0,0)` on
`rgb(255,255,255)`** for every reversed cell.

Consequence for the report: notcurses spends ~30 bytes where an SGR-7 arm spends
2. The rasterizer elides repeated colours across a run, so the real cost is
~28 extra bytes per reversed *run* — one run per frame in scenes 2 and 3, hence
negligible against those rows' totals. But it is a genuine cost of notcurses'
design and not a measurement artifact, and the report should not read it as an
encoding difference.

### 2. The caret is a drawn cell, not the hardware cursor

Scene 1 says "a block caret … alternates between shown and hidden". The hardware
cursor is unavailable: `notcurses_cursor_enable()` writes through `nc->ttyfp`
(fine), but `notcurses_cursor_disable()` writes through
`tty_emit(cinvis, nc->tcache.ttyfd)` (`src/lib/render.c:1774`), and `ttyfd` is
−1 by construction, so the disable fails and emits nothing. notcurses cannot
hide the hardware cursor on a pipe. (This asymmetry looks like a notcurses bug
rather than a policy.)

So the arm draws the caret as a **cell** — the reverse colours above, at row 0
column 21, toggled against the default background. This is also the closer
reading of the scene's own rationale ("one cell of the four thousand eight
hundred changes"), and `SCENES.md` licenses reaching the picture any way the
framework prefers. Still: the report should say which of the two the caret row
measures, because a `ESC[?25l`/`ESC[?25h` toggle is 6 bytes and a cell rewrite is
about 14 to 22.

### 3. The status line's first field must be fixed-width

`SCENES.md` gives the row as a literal and then says the `cpu 12%` and
`3 tasks` fields never change. Those two statements only hold together if the
frame number sits in a fixed-width field: a literal substitution of `frame 119`
for `frame 0` shoves the later fields two columns right, which is both a change
to them and a **fifteenfold** change to the measurement, because the whole
120-cell row would differ every frame instead of two or three cells.

The arm uses a nine-column field one, which reproduces the quoted row exactly at
frame 0 and keeps every later field in the same column at frame 119:

```
 frame 0     elapsed 0.00s    cpu 12%    3 tasks
 frame 119   elapsed 1.98s    cpu 12%    3 tasks
```

`elapsed` needs no such care: `n / 60` for `n ≤ 119` is always four characters,
`0.00` through `1.98`.

**This one needs a ruling in `SCENES.md`, not a per-arm decision.** Two arms
reading it differently would differ by more than an order of magnitude on that
row, and the row would be measuring the reading rather than the framework.

### 4. The list label is 19 characters, not 20

Scene 3 says "a 20-character label `item-NNNNN---------`". The template is
19 characters: `item-` (5) + five digits + nine hyphens. The arm follows the
template, so a row is `NNNNN` + two spaces + 19 = 26 columns. Needs a one-word
fix in `SCENES.md`, either way; the arms must agree.

### 5. Scene 4 does not say which channel takes the colour

"the cell at column *c*, row *r* takes `rgb(c * 2, r * 6, 128)`" — foreground or
background? The arm colours the **foreground** of the `#` and leaves the
background at the terminal default, on the reading that a `#` "coloured from a
ramp" is a coloured glyph. Colouring both would roughly double the row. Needs
specifying.

### 6. Scene 2's lorem is unspecified

`SCENES.md` says "the same 64-character lorem string" without saying which. The
arm uses

```
Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do.
```

exactly 64 ASCII characters. Rows 0..38 never change after frame 0 and every
candidate is the same length in ASCII, so this only affects frame 0's bytes by
nothing at all — but it should still be written down once so arms match.

### And one place the arm is deliberately unfavourable to itself

**`list-scroll` is an upper bound, not notcurses' floor.** This arm redraws all
40 rows every frame and lets notcurses' damage comparison decide what reaches
the wire. notcurses also has a genuine scroll idiom — `ncplane_set_scrolling()`
plus `ncplane_scrollup()` (`notcurses.h:2001`), which the rasterizer supports
through `scroll_lastframe()` and `rasterize_scrolls()` — and on this scene, where
the whole screen scrolls, it would emit one scroll plus one new bottom row plus
two recoloured rows instead of a diff of forty rows. That is most of an order of
magnitude.

It is not implemented here because it could not be validated as carefully as the
rest, and a scroll idiom that silently drew the wrong picture would be worse than
a slow one. **The report must label this row as an upper bound.** A missing row
reads as a win; an un-annotated upper bound reads as a loss, and the suite is
just as dishonest either way.

---

## Was it built? Yes — but not by `make`, and not from a system package

`make` on this machine **fails**, cleanly, with the message `make check-headers`
explains: there is no system notcurses, and installing Homebrew's would have
dragged in ffmpeg for several hundred megabytes that this task had no business
spending.

Instead the arm was compiled and every scene was run against a **notcurses-core
3.0.17 built from the upstream release tarball inside the session scratchpad**,
installing nothing:

```sh
# in a scratch directory, nothing installed
curl -L -O https://github.com/dankamongmen/notcurses/archive/refs/tags/v3.0.17.tar.gz
tar xzf v3.0.17.tar.gz
cmake -S notcurses-3.0.17 -B build -DCMAKE_BUILD_TYPE=Release \
  -DUSE_MULTIMEDIA=none -DBUILD_TESTING=OFF -DUSE_CXX=OFF -DUSE_POC=OFF \
  -DUSE_PANDOC=OFF -DUSE_DOCTEST=OFF -DUSE_STATIC=OFF \
  -DUSE_QRCODEGEN=OFF -DUSE_DEFLATE=OFF
cmake --build build --target notcurses-core -j8
cc -O2 -std=c11 -Wall -Wextra -I notcurses-3.0.17/include -I build/include \
   -o arm arm.c -L build -lnotcurses-core -Wl,-rpath,$PWD/build
```

The one hack that made it configure on macOS: notcurses wants
`tinfo >= 6.1` or `ncursesw >= 6.1` through pkg-config, and macOS ships
ncurses 5.7 with no `.pc` file. A hand-written `ncursesw.pc` in the scratch
`PKG_CONFIG_PATH` pointing at the SDK's `curses.h` and `-lncurses` was enough;
`tiparm()` is present in the system library despite the version number.

**What that means for the numbers below.** Capability detection came from
macOS's terminfo entry for `xterm-256color`, which may differ in small ways from
a Linux ncurses 6.x entry. So the absolute byte counts are indicative, **not**
the CI numbers, and the report's figures must come from the pinned runner. The
*behaviours* are structural and will hold anywhere: the compositing blend, the
geometry source, the `NO_COLOR` no-op, the `COLORTERM` swing, byte-for-byte
determinism, the refusal status, and the hang without the tty shedding.

Verified on this build, 120×40, `TERM=xterm-256color`, `--frames 120`, stdout a
pipe, stdin closed:

| scene | frame 0 | 120 frames | mean/frame | notes |
|---|---:|---:|---:|---|
| `caret` | 128 | 3 210 | 26 | |
| `status-line` | 3 335 | 5 294 | 44 | |
| `list-scroll` | 1 417 | 73 373 | 611 | upper bound, see above |
| `full-repaint` | 91 853 | 11 015 458 | 91 795 | with `COLORTERM=truecolor` |
| `modal-over-list` | — | — | — | **cannot express** (exit 69) |
| `modal-over-list-blend` | 6 188 | 445 555 | 3 712 | not a suite scene |

Also verified: two runs of every scene are **byte-identical**; the geometry
follows `COLUMNS`/`LINES` and not the terminal (40 rows at `LINES=40`, 24 at
`LINES=24`); `latency` reports `key a`, `key Up`, `key b`, `key C-c` correctly as
minimal diffs of the status row (`ESC[6GC-c` for ctrl-C — notcurses normalises a
0x03 byte to `'C'` plus `NCKEY_MOD_CTRL` at `src/lib/in.c:539`); `cpu --seconds
3` finished in 3.01 s wall for 0.02 s user + 0.01 s sys and 4 808 bytes.

The arm also compiles clean (`-Wall -Wextra -Wpedantic -Werror`) against the
3.0.7 headers, so an older distribution will build it.

## What CI should install

The arm links `notcurses-core` and needs nothing from the multimedia half, so
CI wants the **core** development package, which pulls in neither ffmpeg nor
OpenImageIO:

```sh
apt-get install -y libnotcurses-core-dev
```

`libnotcurses-core-dev` ships `usr/include/notcurses/*` **and**
`usr/lib/*/pkgconfig/notcurses-core.pc` (Debian `debian/libnotcurses-core-dev.install`).
`libnotcurses-dev` only adds the multimedia `libnotcurses.so` and its `.pc`, and
is not needed.

Exact versions available, as of 2026-08-22:

| distribution | source version | gives |
|---|---|---|
| Debian sid / forky (testing) | `3.0.17+dfsg-3` | **exactly the targeted version** |
| Ubuntu 26.04 `resolute` | `3.0.17+dfsg-3` | **exactly the targeted version** |
| Ubuntu 25.10 `questing`, 25.04 `plucky` | `3.0.7+dfsg.1-1ubuntu8` | builds; version string differs |
| Ubuntu 24.04 LTS `noble` | `3.0.7+dfsg.1-1ubuntu5` | builds; version string differs |
| Ubuntu 22.04 LTS `jammy` | `3.0.6+dfsg.1-1` | **too old**, the arm's guard rejects < 3.0.7 |
| Debian trixie (13, stable), bookworm (12) | *not packaged* | build from the tarball |

notcurses is **not** in Debian 12 or 13. If the runner is trixie-based, either
pin sid for this one package or build 3.0.17 from the tarball with the cmake
line above — the from-source route is also the only way to pin the exact version
the report names, which the suite wants anyway.

The arm prints whatever `notcurses_version()` reports on its stderr identity
line, so the report records the version that actually ran rather than the one
this file targeted.

## Exit statuses

| status | meaning |
|---:|---|
| 0 | the scene ran |
| 64 | bad arguments (`EX_USAGE`) |
| **69** | **`cannot express`** (`EX_UNAVAILABLE`) — paired with exactly `cannot express\n` on stderr, after the identity line, and **zero** bytes on stdout |
| 70 | a notcurses call failed (`EX_SOFTWARE`) |
| 71 | the environment cannot host this arm (`EX_OSERR`): no UTF-8 locale, or a controlling terminal that could not be shed |

On the refusal the arm still prints its identity line first, so the harness can
record which version refused:

```
arm=notcurses version=3.0.17 no_color=unsupported alt_screen=no
cannot express
```

That is two lines on stderr rather than the contract's one. It is the only
place this arm exceeds that rule, it happens only when there is no frame at all,
and `cannot express` on stderr was the requirement that asked for it.
