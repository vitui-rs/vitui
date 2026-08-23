# Findings

Hand-written and dated, because a number and what it means are two different artefacts with two
different lifetimes. `REPORT.md` is generated; this is not.

## 2026-08-23 — the second family, and it disagreed

Production ticket 05 asked for the eleven attribute facts on two emulator families. The second family
is tmux 3.7c, and the answer is **not one number**: tmux stores eleven of eleven and forwards ten.

### tmux accepts overline, stores it, hands it back, and never sends it

Three captures of scene 01, all three committed to `fixtures/`:

| path | overline |
|---|---|
| the engine → Ghostty 1.3.1, `write_screen_file:…,vt` | present |
| the engine → tmux 3.7c, `capture-pane -p -e` | present |
| the engine → tmux 3.7c → Ghostty, `write_screen_file:…,vt` | **gone** |

Ten of the eleven bits survive all three paths, so this is one attribute and not a broken route. And
**no engine is needed to reproduce any of it.** A raw `printf '\033[53moverline\033[0m'` into a tmux
pane comes back from `capture-pane -e` as `ESC[5:3m`, and the bytes tmux writes to its own attaching
client — captured with `script -q out tmux attach`, which has neither this engine nor Ghostty in it —
contain **no `53` anywhere at all**. The row arrives with no SGR while its neighbours arrive with
`ESC[5m` and `ESC[9m`.

**The mechanism, and it makes the entry's shape a decision rather than a guess.** tmux 3.0's CHANGES:

> Add support for the overline attribute (SGR 53). The `Smol` capability is needed in
> `terminal-overrides`.

`Smol` is defined in none of `xterm-ghostty`, `xterm-256color`, `tmux-256color` or `screen-256color`
on this machine. `Smulx` **is** defined in `xterm-ghostty`, which is exactly why the underline styles
survive the trip and overline does not. Enabling it closes the loop: with
`set -as terminal-features ",xterm-ghostty:overline"` tmux emits `ESC[53m` and the attribute arrives.

So the quirk has no version boundary — every tmux drops it by default, and before 3.0 there was no
overline output at all. What varies is a configuration, not a release.

**And Ghostty renders SGR 53**, which is the first row of that table. So `xterm-ghostty`'s terminfo
omits a capability the terminal it describes actually has, and tmux believes the terminfo. That is
spec §10's refusal of terminfo arriving as field evidence from a direction nothing planned for: the
refusal was argued as *do not infer capabilities from a name*, and this is the same refusal earning
its keep against a description the terminal's own author ships.

### The instrument's second defect was, again, in the instrument

The first tmux run reported overline as **blink**. tmux never rendered blink there; the parser
invented it — which is word for word the defect stage 1 recorded one level up, arriving again from a
place that had been checked.

`grid.c`'s `grid_string_cells_code`:

```c
if (s[i] < 10)
        xsnprintf(tmp, sizeof tmp, "%d", s[i]);
else
        xsnprintf(tmp, sizeof tmp, "%d:%d", s[i] / 10, s[i] % 10);
```

**A colon in a tmux capture is a divided-by-ten, not a sub-parameter.** tmux numbers its underline
styles 42–45 *so that* the division lands on ECMA-48's `4:2`–`4:5`, and the two readings agree — which
is precisely why a parser that knew nothing about this passed every test stages 0 and 1 had. Overline
is 53 and rides the same branch, so it comes back as `5:3`, and `5:3` in ECMA-48 is blink.

There is no reading of those five bytes that is correct for both formats. So `parse` now takes a
`Dialect` and **has no default**: a caller that cannot say which format it captured cannot be trusted
to have captured either — the same argument as `expected_rows` being a parameter. Both readings are
pinned by tests, so the dialect is asserted rather than assumed.

The generalisation is worth more than the fix. Stage 1's lesson was *the flattening was correct for
every capture stage 0 had*. This one is a level up: **the parser was correct for every capture that
had ever been taken, and a second source of captures is what made it wrong.** An instrument gains a
new failure mode from each arm it grows, and the arm that finds it is the one that was added last.

### And the fold's own trap, caught by writing the test for it

tmux's underline codes are 42–45. **ECMA-48's background colours are 40–47.** So the arm that reads a
folded `4:2` as 42 sits inside the arm that reads a bare `42` as a green background, and in a `match` the
first one written wins — the fold's arm was, so `ESC[42m` came back as a double underline in *both*
dialects. None of the five committed fixtures has a background colour on a row that would have shown it.

The arm is guarded on the fold now (`42..=45 if folded.is_some()`), and there is a test that fails
without the guard. Worth its paragraph because the trap is in the numbering rather than in the code: any
future reading of tmux's colon form lands in the same overlap.

### `attrs_dropped` is populated and nothing reads it

Found while writing the entry the evidence above earned. `Quirks::apply` fills
`caps.private.attrs_dropped`, `Capabilities::report` prints it, and **`serial.rs` never reads the
mask** — while `caps.rs`'s own module docs and spec §10 both say *an unsupported one is dropped
silently at serialise time*.

So the field scene 01 was built to feed is not wired to the thing it claims to control. The tmux entry
is therefore a recorded observation and not yet a behaviour, and both files now say so where a reader
would otherwise take it on trust. Wiring it is production ticket 10 — and it is not a one-line mask,
because `quant::OnTheWire` has to narrow the expectation too, the way it already drops a hyperlink the
terminal cannot express, or the round trip will fail on a bit the engine correctly declined to send.

This is the backlog's opening finding a third time: a declaration that is load-bearing for the silence
around it. The round trip agreeing with itself, an MSRV agreeing with the lints it disabled, and a mask
agreeing with a serializer that never asked.

### Two operational notes

- **A Ghostty window's `command` runs with the GUI application's `PATH`**, which is
  `/usr/bin:/bin:/usr/sbin:/sbin` — Homebrew's `/opt/homebrew/bin` is not on it. The first
  `--through-tmux` run opened a window, failed to exec `tmux`, and reported *the scene never presented
  a frame*: the readiness timeout doing its job and saying nothing about the cause. The path is
  resolved in the driver now, which has the developer's `PATH`, so the refusal names what is missing.
- **A tmux server outlives the client in it.** Closing the window kills the client and leaves the
  session detached, so the arm kills the server explicitly. Without that, a run leaves litter the
  *next* run's socket-collision refusal fires on.

## 2026-08-23 — stage 1, and the instrument found its first defect in itself

Ghostty 1.3.1 agreed with the engine **11/11** on scene 01. Three things are worth more than that
number.

### The dump carries all eleven attributes, and the design note said it carried five

Ticket 04's table recorded Ghostty's `vt` writer as preserving *"attributes 1/3/4/7/9 and underline
colour 58"*. A control probe — raw `printf` into a Ghostty window, no engine anywhere in it — run
before the driver was written says otherwise:

| sent | returned |
|---|---|
| `1` `2` `3` `5` `7` `8` `9` `53` | all eight, unchanged |
| `4` | `4` |
| `4:2` `4:3` `4:4` | `4:2` `4:3` `4:4` |
| `58:2::0:0:255` | `58;2;0;0;255` |

**Dim, blink, conceal, overline and the underline *styles* all survive**, which is what makes an
eleven-boolean assertion answerable at all. Running the control before the instrument is what
separated *the terminal does not render it* from *the dump does not serialise it*; with only the
engine's own arm, an absent overline would have had two explanations and no way to choose.

### `4:2` is one parameter, and the committed parser was reading it as two

The stage 0 parser split an SGR body on `;` **and** `:` into one flat token stream, because that
makes the two colour spellings read through a single path. The control probe's `4:2` came back
through it as SGR 4 followed by SGR 2 — **double underline read as underline plus dim**, an
attribute the terminal never rendered and the instrument invented.

ECMA-48 separates parameters with `;` and a parameter's sub-parameters with `:`, and the parser now
does too; the colour arms accept either spelling by looking for the tail in the two places it can
be. Six tests hold it, and the shape of the mistake is worth keeping in mind: **the flattening was
correct for every case stage 0 had captures for.** It took bytes from a second emulator, produced by
a scene nobody had written yet, to make it wrong.

### The handshake proves a frame exists; it does not prove the window stopped moving

The first live run photographed a screen whose top two rows had scrolled off, and every row of the
report was wrong by two. The scene had presented — the readiness handshake was satisfied — but a
Ghostty window **settles its size after the process inside it starts**, and the frame painted at the
first geometry was reflowed at the second. Two runs on this machine were handed 156×45 and 72×24, so
this is not a rare race.

The fix is not a longer delay. The scene now redraws on every wake and stamps a frame counter, and
the driver waits for that stamp to **stand still** for 500 ms before photographing. An idle vitui
application costs zero wakeups, so a stamp that stops moving is a screen that has stopped moving —
an observed condition, in the same shape as the readiness handshake it extends, rather than a sleep
tuned until it passed.

It also generalises past this arm: **any capture of a window an emulator has just opened is a
capture of a screen that may still be settling**, and the tmux arm is exempt only because
`capture-pane` runs against a pane whose size the harness set.

### Git rewrote the first committed capture, and every test still passed

Found while committing stage 1. `core.autocrlf = input` on this machine, and `git add` rewrote CRLF
to LF on the way into the index: **1949 bytes of what Ghostty actually sent became a 1926-byte
blob.** Twenty-three carriage returns, gone.

Nothing went red. The parser skips CR the way a terminal does, so the fixture stopped being the
bytes a terminal sent and carried on passing every assertion over it — a capture that is evidence,
silently not being the evidence, which is this directory's own failure mode one level up. `*.vt` is
`-text` in `.gitattributes` now, and there is a test asserting the twenty-three are still there,
because an attributes file nobody checks is the same kind of thing as an MSRV nobody compiles.

`-text` and not `binary`: `binary` means `-text -diff` together, and the diff is worth keeping. A
capture that changes must be reviewable *as a diff* — that is the entire mechanism by which
something which reports rather than gates still catches a regression.

## 2026-08-22 — stage 0, and the capture format is the finding

The instrument does not exist yet. What exists is the answer to *what can a dump-based capture see*,
and it is narrower than the design assumed — narrow enough that it changes how ticket 06's scene has
to be built.

### A grid-to-text dump cannot see a double-width glyph's second cell

Sent through tmux 3.7c, `capture-pane -p -e`, pane 20×6:

```
printf 'AB\346\274\242CD\r\n'      →  41 42 e6 bc a2 43 44 0a
                                       A  B  漢          C  D  \n
```

**No padding cell, no continuation marker, nothing.** On screen `漢` occupies columns 2 and 3 and `C`
is at column 4; in the dump `C` follows the glyph immediately. To know that, a reader has to know `漢`
is double-width — **which is the thing under test.**

Ghostty 1.3.1's `write_screen_file:…,vt` was already observed to do the same. Two emulators from two
lineages, one behaviour, and it is not a quirk of either: **it is inherent to re-serialising a cell
grid as text.** So the design's prediction — *ticket 20 will not be settled by the dump* — is
confirmed and generalises from one arm to the whole *category* of arm.

The consequence is a design constraint rather than a disappointment, and it is written into
[production ticket 06](../.scratch/vitui-engine-production/issues/06-the-clip-edge-pair-decided-on-what-was-observed.md):
the scene must put a **unique ASCII sentinel in every cell that should be a continuation**, which
converts *how wide was that glyph* into *is the sentinel still there* — a question a text dump can
answer — or it must use CPR, where the emulator reports the column itself.

### tmux 3.7c parses the colon form, which makes two of tier 1 observed rather than inferred

| sent | returned | |
|---|---|---|
| `ESC[38:2::255:0:0m` | `ESC[38;2;255;0;0m` | exact channels |
| `ESC[1;4;58:2::0:0:255m` | `ESC[1;4m` `ESC[58;2;0;0;255m` | exact channels, attributes split off |
| `ESC[38:5:200m` | `ESC[38;5;200m` | exact index |

Both emulators tested so far **normalise to semicolons on output**, whatever spelling they were sent.
That is what makes the dump a usable measuring channel at all: a terminal that *mis*-parsed the colon
form would return channel values off by one, which is precisely the failure
`serial::emit_color`'s own comment names. Agreement on the normalised output is therefore evidence
about the *parse*, not about the spelling.

With [arch 23](../.scratch/vitui-engine-architecture/issues/23-the-two-sgr-spellings-and-which-one-is-the-default.md)'s
Ghostty observation, **two of tier 1's seven are now observed.** Five remain inference from libvaxis's
three quirk entries. Neither of the two is one of the three terminals those entries name, so nothing
here contradicts them.

### The reset is not one sequence, and a parser that assumes it is will be wrong

tmux closes a colour-only run with `ESC[39m` — default foreground, leaving other attributes alone —
and an attribute-bearing run with `ESC[0m`. Both appear in
`fixtures/tmux-3.7c-attrs-and-colours.vt`. A parser must treat the SGR stream as *state*, not as
paired delimiters. This is the kind of thing that reads as a detail until it silently mis-attributes a
style to the following cell.

### What is not yet known

- **Whether a tmux arm is worth what it costs.** `capture-pane` re-serialises *tmux's* grid, so this
  arm measures tmux's conformance and never the emulator behind it. That is legitimate — tmux is in
  §10's own tier-1 list and `quirks.rs` already cites its 1 s sync-flush limit — but it must never be
  read as a proxy, and `SCENES.md` says so.
- **The empty-capture failure mode has not been exercised.** `screen -X hardcopy` returned rc=0 and a
  zero-byte file during the design investigation, and the refusal that must catch it is unwritten. It
  is the first thing the parser needs, not the last.
- **Everything about the engine.** No engine bytes have been through this path yet: the public API is
  being changed by
  [production ticket 03](../.scratch/vitui-engine-production/issues/03-the-uri-travels-at-the-verb.md)
  as this is written, and a driver built against today's `Restyle` would not compile tomorrow.
