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

## Status: stages 0, 1 and 2

**Ghostty 1.3.1 agreed with the engine 11/11 on scene 01, and the run's own capture is now a
fixture.** Nineteen tests, no emulator in the loop for any of them.

```sh
cd conform && cargo test              # the gate: the comparator over committed captures
cd conform && cargo run --example ghostty     # the soak: one real window, and REPORT.md
```

The live arm opens a Ghostty window, drives the engine inside it, photographs the screen and closes
the window again — about a second and a half, and it **takes focus for that second and a half**,
which is inherent to driving a window server and not something the driver can avoid. Whatever is
typed into it while it is up is drained and ignored.

The instrument is one executable with two halves: with no arguments it is the driver, with
`--scene 01` it is the scene, and the driver launches the scene by re-running its own
`current_exe()`. That is not a trick to save a file — it makes the two halves the same build by
construction, where a sibling binary path can silently be yesterday's.

`CONFORM_SAVE_CAPTURE=<path>` writes the raw bytes out. It is **opt-in and never automatic**: a
driver that rewrote its own fixtures on every run would turn the gate into a mirror.

Stages 3 (CPR and the width questions), 4 (a second emulator family) and 5 (mode 2026) are open.
See [ticket 04](../.scratch/vitui-engine-production/issues/04-the-conformance-harness.md).

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
| `tmux-3.7c-wide-no-padding.vt` | that `AB漢CD` comes back with **no padding cell and no continuation marker** — 28 bytes that decide how ticket 06's scene has to be built |
| `ghostty-1.3.1-scene01-attrs.vt` | scene 01 as Ghostty gave it back: all eleven attribute bits, each on its own row, each stopping where its label does; and the OSC 10/11 header this machine's Ghostty leads with, which is the only statement anywhere of what `Colour::Default` actually resolves to |

Raw bytes, as captured. Do not regenerate them to make a test pass: they are evidence, and a fixture
that moves because the code moved is not evidence of anything.

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
- **What the pixels look like.** The `vt` dump is Ghostty's own cell state re-serialised, so it says
  what the terminal *recorded*, not what it *drew*. A terminal that stores an attribute and renders
  nothing agrees here and disagrees on screen.
- **Anything about timing.** Mode 2026 is stage 5, and the expectation is already recorded: the
  AppleScript round trip's jitter is the same order as Alacritty's 150 ms force-flush limit, so the
  sub-200 ms end may be unanswerable on this machine.
