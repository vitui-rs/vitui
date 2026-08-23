# The conformance suite — Ghostty-via-tmux

**Generated. It reports; it does not block.** One file per arm, because two arms writing one file means the rows of whichever ran first are gone — and *a missing row reads as a win*.

The version of software this repository does not control cannot gate its pull requests. So this file is committed and regenerated, and a disagreement arrives as a review-visible diff rather than as a red build. The gate is `cargo test` over `fixtures/`.

Read [`SCENES.md`](SCENES.md) first.

- **Arm:** Ghostty-via-tmux 1.3.1, `write_screen_file:…,vt`
- **These rows are evidence about:** **tmux's forwarding**, read through Ghostty. tmux parses the engine's bytes and Ghostty parses tmux's, and Ghostty alone agrees 11/11 — so a disagreement here is what tmux passed on. This is the only instrument here that can see it: `capture-pane` and tmux's redraw path are different code, and `attrs_dropped` is about what is rendered
- **Surface:** 36x11 cells, as the scene reported it
- **Default colours, from the dump's own OSC 10/11:** fg `#eaeaea`, bg `#000000`
- **Launch to capture:** 1111 ms, of which the capture round trip alone was 136 ms — reported, never gated. It is AppleScript round trips and a window opening
- **Geometry:** not ours to set. `surface configuration` offers a font size and no rows or columns, so the scene draws at the top left of whatever it is given
- **Two parsers in series.** The engine detected *tmux's* capabilities, not Ghostty's, so a disagreement has two candidate causes — what the engine chose to emit, and what tmux passed on. The row tells them apart: an attribute the engine never sent is missing, where one tmux mangled arrives wrong

## Scene 01 — the eleven attribute bits, one per row

Each row lights exactly one of the style word's eleven attribute bits. A row agrees only when the dump reports that attribute **and nothing else** — an invented attribute is a disagreement in the same way a missing one is.

| bit | verb | expected | observed | |
|---|---|---|---|---|
| bold | `Style::bold` | bold | bold | ✓ |
| dim | `Style::dim` | dim | dim | ✓ |
| italic | `Style::italic` | italic | italic | ✓ |
| reverse | `Style::reverse` | reverse | reverse | ✓ |
| blink | `Style::blink` | blink | blink | ✓ |
| strikethru | `Style::strikethrough` | strike | strike | ✓ |
| conceal | `Style::conceal` | conceal | conceal | ✓ |
| overline | `Style::overline` | overline | nothing | **FAILED** |
| under-sgl | `Style::underline` | underline | underline | ✓ |
| under-dbl | `Style::underline_double` | underline:2 | underline:2 | ✓ |
| under-dot | `Style::underline_dotted` | underline:4 | underline:4 | ✓ |

**10/11 agreed.**

A row that does not say which arm it came from is not a result, and a *missing* row reads as a win — which is the single easiest way for this directory to become dishonest. Every row of the scene is printed above whether it agreed or not, and a capture with fewer rows than the scene declared never reaches this table: it is refused as `FAILED` by the parser.
