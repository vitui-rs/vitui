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

## Status: stages 0 and 2

**No driver and no report yet, and the parser is already gated.** What exists is `FINDINGS.md`,
`SCENES.md`, two committed captures, and the parser and comparator over them — eleven tests, no
emulator in the loop. The captures changed the plan before a harness was written, which is the point
of doing stage 0 first, and the parser was built next because it is the half that does not depend on
the engine's public API, which ticket 03 is rewriting.

What is left for a live run: the driver, which needs the settled API, and the OSC 10/11 header reader.

```sh
cd conform && cargo test        # a detached workspace: its own graph, its own gate
```

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

Raw bytes, as captured. Do not regenerate them to make a test pass: they are evidence, and a fixture
that moves because the code moved is not evidence of anything.
