# The conformance suite — kitty

**Generated. It reports; it does not block.** One file per arm, because two arms writing one file means the rows of whichever ran first are gone — and *a missing row reads as a win*.

The version of software this repository does not control cannot gate its pull requests. So this file is committed and regenerated, and a disagreement arrives as a review-visible diff rather than as a red build. The gate is `cargo test` over `fixtures/`.

Read [`SCENES.md`](SCENES.md) first.

- **Arm:** kitty 0.48.2, `kitten @ get-text --extent screen --ansi`, over a unix socket
- **These rows are evidence about:** kitty. Its own cell state, re-serialised by the emulator that holds it — the same kind of evidence as the Ghostty arm's and gathered over a remote-control socket rather than an AppleScript surface, so no automation grant and no window z-order is in the loop
- **Surface:** 80x24 cells, as the scene reported it
- **Default colours, from the dump's own OSC 10/11:** fg `cannot express` — this capture format has no OSC 10/11 header, bg `cannot express` — this capture format has no OSC 10/11 header
- **Geometry: asked for and got.** `-o initial_window_width=80c -o initial_window_height=24c`, and `kitten @ ls` reported 80x24. **The first *emulator* arm that can be handed a size** — Ghostty's `surface configuration` offers a font size and nothing else. The quiescence handshake is kept anyway: it is what proves the window stopped moving, and one command-line option is not a run
- **Config:** none. `--listen-on` and `-o allow_remote_control=yes` are given on the command line, so there is no `kitty.conf` in the result — the tmux arm's `-f /dev/null` equivalent, for free. `-o remember_window_size=no` is the other half: without it kitty restores the size of the last window the user dragged
- **Launch to capture:** 988 ms, of which `get-text` itself was 19 ms — reported, never gated. No automation consent dialog, no clipboard, no z-order
- **Trailing blanks: kept, and that is the opposite of the tmux arm.** Every row the engine painted comes back at its full width, so the check that an attribute stopped where its label did has real padding to look at here. kitty closes each label with an explicit off-code — `22`, `23`, `24`, `27`, `29` — rather than a reset, which is what makes that check answerable at all. The one exception is the final row, which arrives as a bare `CSI m` with no cells where Ghostty's dump gives it painted; the parser drops a trailing blank row after counting the rows, so nothing turns on it
- **A never-written cell is not a painted blank**, and only the first is trimmed. A probe screen kitty had never had written to came back with its unpainted rows absent entirely — which is what an empty capture looks like from this arm, and why the refusal is the first thing it does rather than the last
- **Rows are LF-separated**, where Ghostty's dump is CRLF. So `core.autocrlf` has nothing to rewrite in this arm's fixture — which is luck rather than safety, and `.gitattributes` still marks `*.vt` as `-text`

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
| conceal | `Style::conceal` | conceal | `quirks.rs`'s fifth entry, earned by this arm's own first run and by kitty's shipped binary: its `Cursor` carries no conceal attribute, so SGR 8 has nothing to set. The engine now withholds it, and `fixtures/kitty-0.48.2-scene01-attrs.vt` is the capture taken while it still sent it — observed: nothing | `by design` |
| overline | `Style::overline` | overline | the same entry's second bit. kitty's `Cursor` carries no overline attribute either, and a raw-`printf` control probe with no engine in it says the same thing independently — observed: nothing | `by design` |
| under-sgl | `Style::underline` | underline | underline | ✓ |
| under-dbl | `Style::underline_double` | underline:2 | underline:2 | ✓ |
| under-dot | `Style::underline_dotted` | underline:4 | kitty's serialiser has a string for `4:2` and `4:3` and none for `4:4` or `4:5`, so a dotted underline is written into the capture as `CSI 4 : m` — an empty sub-parameter, which ECMA-48 reads as SGR 4's default of *single*. `CSI 4:0 m` comes back as nothing at all, so the `4:` here does prove the cell holds a non-zero decoration; what is lost is which one. A fact about the **capture format**, not about what kitty renders — observed: underline | `cannot ask` |

**8/8 agreed.**

**Not in that denominator: 3 of the scene's 11.** This arm declared before the run that it would not compare them, with the reason printed in the row beside what was nonetheless observed. `cannot ask` is a fact about the **instrument** — the emulator does the thing and this suite cannot see it. `by design` is a fact about **the engine** — it consulted `quirks.rs` and did not send it, so a `FAILED` would blame the terminal for a decision of ours. `compare/`'s three kinds of non-number have a word for neither, which is why `SCENES.md` grew two more. A row so declared that agrees anyway is reported `STALE` and counted as a failure, so a declaration cannot outlive what earned it.

A row that does not say which arm it came from is not a result, and a *missing* row reads as a win — which is the single easiest way for this directory to become dishonest. Every row of the scene is printed above whether it agreed or not, and a capture with fewer rows than the scene declared never reaches this table: it is refused as `FAILED` by the parser.
