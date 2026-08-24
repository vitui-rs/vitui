# `conform/` — does a real terminal show what we composited?

**A detached directory, outside the root workspace, and it reports rather than gates.**

Every verification instrument `vitui-engine` has lives inside the crate. `roundtrip.rs` composites a
frame, serialises it, replays the bytes through `term_model.rs` and asserts the replayed screen equals
the frame; `testing.rs` asserts a **three-way** agreement between the frame, `serial::Mirror` and
`TermModel`. **Two of those three are this engine's own code.** Architecture ticket 20 recorded what
that arrangement cannot catch:

> Every gate stays green with §3's pairing invariant **false**, because the serializer and the terminal
> model are wrong in the same direction. *The round trip cannot catch a case where the model and the
> serializer agree with each other and not with the terminal.*

This directory is the missing fourth party. It is
[production ticket 04](../.scratch/vitui-engine-production/issues/04-the-conformance-harness.md).

## Status: stages 0, 1, 2, two scenes and three emulator families

**Four arms, four committed reports, three emulator families, two `quirks.rs` entries and one closed
architecture ticket came out of them.** Thirty-four tests, no emulator in the loop for any of them.

| arm | scene 01 | scene 04 | what its rows are about |
|---|---|---|---|
| `cargo run --example ghostty` | **11/11** | **6/6** | Ghostty 1.3.1's own cell state |
| `cargo run --example tmux` | **10/10**, one `by design` | **6/6** | what tmux 3.7c *stores* — `capture-pane` re-serialises tmux's grid |
| `cargo run --example ghostty -- --through-tmux` | **10/10**, one `by design` | **6/6** | what tmux 3.7c *forwards*, read through Ghostty |
| `cargo run --example kitty` | **8/8**, one `cannot ask`, two `by design` | **6/6** | kitty 0.48.2's own cell state |

**An arm runs every scene or it is not a run**, and one report per arm holds a section for each —
same rule, same reason, as one file per arm: a section that is missing reads as a win. There is
deliberately no flag to run one scene.

**Scene 04 closed [architecture ticket 20](../.scratch/vitui-engine-architecture/issues/20-a-pair-bisected-by-a-child-clip.md)**,
which is the first decision on that map settled by asking a terminal rather than by argument. All four
arms agree that a terminal blanks the orphaned half of a bisected pair itself, in both directions, and
none of them has a clip to consult. **They disagree about what the blanked cell wears** — kitty keeps
the orphan's background, Ghostty and tmux blank to the SGR state in force — which is the finding that
turned *the engine may as well repair* into *the engine must*. See `FINDINGS.md`.

It is also the only scene that does **not** drive the engine, and it cannot: the engine repairs a
bisected pair before it serialises anything, so an engine-driven scene could photograph only the
repair. That is why wiring the answer took no measurement away from it, where the `--through-tmux`
arm's overline row lost its.

tmux's one disagreement was overline, and it took three arms to attribute: tmux accepts SGR 53, stores
it, hands it back when asked, and never puts it on the wire. kitty's two are conceal and overline, and
**no arm could have attributed either** — a dump cannot tell *not stored* from *not serialised*, and a
terminal endpoint has no far side to read from. What settled those is kitty's shipped binary, whose
`Cursor` carries neither attribute. They are `quirks.rs`'s fourth and fifth entries, the only two this
repository gathered rather than inherited — see `FINDINGS.md`, and production tickets 05 and 04.

**Every one of those disagreements is now a `by design` cell, and that is the most important sentence
here.** Production ticket 10 wired `attrs_dropped` on to the wire, so the engine consults the table
and withholds what a terminal will not render — and the arms can no longer take the measurements that
earned the entries. `FAILED` would blame the terminal for a decision of ours. It is not a defect to
fix: an arm that asked the engine what to expect would be checking the engine against itself, which is
the arrangement this directory exists to break. **The committed captures in `fixtures/` are what
preserve the evidence**, taken while the engine still sent those bits, and that is the sharpest reason
yet never to regenerate one to make something pass.

**kitty's fourth number is a different thing again.** `cannot ask` is a fact about the *instrument*,
not about the emulator and not about the engine: kitty renders a dotted underline and writes it into a
capture as `CSI 4 : m`, which ECMA-48 reads as single — so a row compared anyway would have earned a
`quirks.rs` entry describing a misbehaviour that is not happening. `SCENES.md` has all five kinds of
non-number and the three rules that stop the two new cells becoming excuses; one of those rules is a
gate.

```sh
cd conform && cargo test              # the gate: the comparator over committed captures
cd conform && cargo run --example tmux            # the headless soak. No window, no grant
cd conform && cargo run --example kitty           # a window, but no automation grant
cd conform && cargo run --example ghostty         # the soak that needs a window server
cd conform && cargo run --example ghostty -- --through-tmux   # tmux in the middle
```

**One report file per arm**, `REPORT-<arm>.md`, and that is not filing. Two arms writing one
`REPORT.md` means the rows of whichever ran first are gone, and *a missing row reads as a win* is this
directory's own failure mode — the one it is least able to notice.

The Ghostty arm opens a window, drives the engine inside it, photographs the screen and closes the
window again — about a second and a half, and it **takes focus for that second and a half**, which is
inherent to driving a window server and not something the driver can avoid. Whatever is typed into it
while it is up is drained and ignored. **The tmux arm is headless**: no window server, no automation
grant, no focus taken. **The kitty arm sits between them** — it needs a window server and takes focus,
but reaches the terminal over a remote-control socket rather than an automation surface, so there is
no TCC grant, no clipboard and no z-order in the loop, and no `kitty.conf` either. It is also **the
only *emulator* arm that can be handed a size**, in cells; tmux can set a pane size and is not an
emulator, and Ghostty's `surface configuration` offers a font size and nothing else.

Each arm is one executable with two halves: with no arguments it is the driver, with `--scene NN` it
is that scene, and the driver launches the scene by re-running its own `current_exe()`. That is not a
trick to save a file — it makes the two halves the same build by construction, where a sibling binary
path can silently be yesterday's. **A fresh terminal per scene**, rather than a scene switch inside
one: a window that has been written to once is a window whose state is part of the measurement.

**The scene, the readiness handshake and the comparison live in `examples/common.rs`**, shared by the
arms rather than copied into each. That is load-bearing for the second family and paid for itself
again on the third: two copies of scene 01 would make a disagreement between the arms unattributable,
because it could be the software or it could be the drift. An arm brings four things and nothing else
— a way to start the scene, a way to read the screen back, a way to shut down, and an `Arm` describing
itself, **including the rows its capture format cannot ask**, declared before the run.

`CONFORM_SAVE_CAPTURE=<prefix>` writes the raw bytes out, one file per scene —
`fixtures/kitty-0.48.2` becomes `fixtures/kitty-0.48.2-scene04-pairs.vt`. It is **opt-in and never
automatic**: a driver that rewrote its own fixtures on every run would turn the gate into a mirror.
**And it will not overwrite one.** *A capture is never regenerated to make something pass* was a
sentence in three files; it is a branch now. An existing fixture is left alone and said so on stderr
rather than failing the run — adding a scene means running an arm whose other scenes are already
captured.

Stage 3 (CPR and the width questions) is open, and **smaller than it was**: what it was named for was
attacking architecture ticket 20, and scene 04's sentinels did that without a CPR reader. What remains
there is the width questions proper. Stage 5 (mode 2026) is open. Stage 4 has three emulator
families now — Ghostty, kitty and, as a target rather than an emulator, tmux — and what it still owes
is a second **VT lineage**: Terminal.app, glyph-grid scenes only, not built. See
[ticket 04](../.scratch/vitui-engine-production/issues/04-the-conformance-harness.md).

## Why it reports and never gates

`compare/README.md` argued this exact case first:

> **Four external projects' versions cannot gate this repository's pull requests.** A suite that did
> would go red when Textual cut a release, on a commit that touched nothing, and the honest response to
> that would be to stop running it.

Substitute *when Ghostty ships 1.4* and the sentence is unchanged. Three more reasons specific to this
one: it needs a window server, it needs a macOS automation grant that cannot be obtained
non-interactively, and its result depends on the emulator's **version**, which is the thing being
measured. Ghostty's dump format is also **undocumented** — the `plain|html|vt` argument was found by
probing `+validate-config`, so it is unpromised.

Like `compare/` and unlike `fuzz/`, there is **no `deny.toml`**: the third-party thing *is* the subject.

## The part that will gate, and it is the part worth having

**The parser and comparator, as pure functions over the committed captures in `fixtures/`.** Capture
once from a real emulator, commit the bytes, and `cargo test` verifies the comparison logic with no
emulator, no window and no grant in the loop.

This is the same trade `fuzz/` already makes — the committed corpus is the gate, the fuzzer is the soak
— and here the live capture is the soak. It is stage 2, and it is deliberately not last: a comparator
nobody can run is worth less than one that runs on every commit against bytes a terminal really sent.

**The first thing the parser must do is refuse.** During the design investigation
`screen -X hardcopy` returned rc=0 and a **zero-byte file**; written the obvious way, an empty dump
compares equal against a blank region and the row goes green. Zero bytes, or fewer rows than expected,
is `FAILED` and never a match. That is `compare/`'s *a missing row reads as a win* in its most dangerous
form — the missing row hiding inside a green one.

## `fixtures/`

| file | what it pins |
|---|---|
| `tmux-3.7c-attrs-and-colours.vt` | both SGR spellings resolving to the same channels; bold, underline, underline colour, reverse, italic, strikethrough; and the `ESC[39m`-versus-`ESC[0m` reset asymmetry |
| `tmux-3.7c-wide-no-padding.vt` | that `AB漢CD` comes back with **no padding cell and no continuation marker** — 28 bytes that decided how scene 04 had to be built, and it is the reason that scene compares row *text* rather than columns |
| `ghostty-1.3.1-scene01-attrs.vt` | scene 01 as Ghostty gave it back: all eleven attribute bits, each on its own row, each stopping where its label does; and the OSC 10/11 header this machine's Ghostty leads with, which is the only statement anywhere of what `Colour::Default` actually resolves to |
| `tmux-3.7c-scene01-attrs.vt` | the same scene as **tmux's own grid** holds it — all eleven, overline included, spelled `5:3` because tmux writes any two-digit attribute code as `code/10 : code%10` |
| `ghostty-1.3.1-via-tmux-3.7c-scene01-attrs.vt` | the same scene **through** tmux into Ghostty: ten of the eleven, and overline gone. The three files above are one scene down three paths, which is what turns *something is wrong* into *tmux does not forward SGR 53* |
| `kitty-0.48.2-scene01-attrs.vt` | the same scene as kitty holds it: nine of the eleven, conceal and overline bare, and a dotted underline spelled `CSI 4 : m` — the bytes the arm's `cannot ask` declaration rests on. LF-separated where Ghostty's is CRLF, and padded to the full width where tmux's is trimmed to the label |

| `ghostty-1.3.1-scene04-pairs.vt` | scene 04 as Ghostty gave it back: the orphaned half blanked in both directions, and blanked **to the SGR state in force** rather than to the glyph's own red background |
| `kitty-0.48.2-scene04-pairs.vt` | the same scene as kitty holds it: the same four text rows, and the blanked half wearing **the orphan's own background**. The one row on which the three families differ, and the reason the engine may not delegate the repair |
| `tmux-3.7c-scene04-pairs.vt` | the same scene as tmux's own grid holds it — agreeing with Ghostty on all five |
| `ghostty-1.3.1-via-tmux-3.7c-scene04-pairs.vt` | the same scene **through** tmux into Ghostty, agreeing with both. It is also the arm that found the probe's own defect: `CSI 2 J` pushed the picture into tmux's history and the capture came back with the scene on it twice |

Raw bytes, as captured. Do not regenerate them to make a test pass: they are evidence, and a fixture
that moves because the code moved is not evidence of anything. `save_if_asked` now refuses to.

**`.gitattributes` marks `*.vt` as `-text`, and that line is load-bearing.** Without it,
`core.autocrlf = input` rewrote CRLF to LF inside the Ghostty capture on the way into the index —
1949 bytes became 1926, twenty-three carriage returns disappeared, and every test still passed
because the parser skips CR the way a terminal does. A test now asserts the twenty-three are there.
See `FINDINGS.md`.

## The engine is a dev-dependency, and that is on purpose

The library depends on nothing. `vitui-engine` is a **dev**-dependency, so it is linked into the
examples and not into the code the gate runs through — the same rule as `Row` deliberately having no
`cell_at(column)`. A comparator that linked the engine would be checking the engine against itself
again, which is the exact arrangement architecture ticket 20 found and this directory exists to
break.

## What this instrument cannot see

Written down because a limit nobody wrote down becomes a claim.

- **Which column a glyph is in.** Both capture formats emit a double-width glyph with no padding
  cell and no continuation marker, so architecture ticket 20 is not answerable by any dump. It waits
  on production ticket 06's ASCII sentinels or on CPR. See `FINDINGS.md`.
- **Curly and dashed underlines as per-bit rows.** They light two of the three underline bits each;
  the committed fixtures cover them instead.
- **Which of two capture formats it is holding.** It cannot work that out and does not try:
  `parse` takes a `Dialect` and has **no default**. tmux's `capture-pane -e` spells overline `5:3`,
  which is *blink* in ECMA-48, so a parser told nothing invents an attribute — see `FINDINGS.md`.
  A caller that does not know which format it captured cannot be trusted to have captured either.
  kitty was probed for a dialect of its own and had none: every construct it emits is ECMA-48.
- **A dotted or dashed underline, on kitty.** Its serialiser writes both as `CSI 4 : m` and ECMA-48
  reads that as single, so the row is declared `cannot ask` rather than compared — which is not the
  same statement as `cannot express`, and the difference is a `quirks.rs` entry that would have been
  wrong. `CSI 4:0 m` comes back as nothing, so the capture can still say *some* decoration is there.
- **Why two arms disagree, in general.** A scene can say that they do. Whether an absent attribute
  was never stored or merely never serialised is a question a dump has no way to reach, and for tmux
  it took a third arm while for kitty — an endpoint, with no far side — it took the shipped binary.
- **What the pixels look like.** The `vt` dump is Ghostty's own cell state re-serialised, so it says
  what the terminal *recorded*, not what it *drew*. A terminal that stores an attribute and renders
  nothing agrees here and disagrees on screen.
- **Anything about timing.** Mode 2026 is stage 5, and the expectation is already recorded: the
  AppleScript round trip's jitter is the same order as Alacritty's 150 ms force-flush limit, so the
  sub-200 ms end may be unanswerable on this machine.
