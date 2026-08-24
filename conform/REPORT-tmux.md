# The conformance suite — tmux

**Generated. It reports; it does not block.** One file per arm, because two arms writing one file means the rows of whichever ran first are gone — and *a missing row reads as a win*.

The version of software this repository does not control cannot gate its pull requests. So this file is committed and regenerated, and a disagreement arrives as a review-visible diff rather than as a red build. The gate is `cargo test` over `fixtures/`.

Read [`SCENES.md`](SCENES.md) first.

- **Arm:** tmux 3.7c, `capture-pane -p -e`
- **These rows are evidence about:** **tmux**, and never the emulator behind it. `capture-pane` re-serialises tmux's own grid, so the engine's bytes were parsed and stored by tmux and handed back by tmux. A legitimate target — tmux is in spec §10's tier-1 list — and never a proxy for the terminal it is running inside
- **Surface:** 80x24 cells, as the scene reported it
- **Default colours, from the dump's own OSC 10/11:** fg `cannot express` — this capture format has no OSC 10/11 header, bg `cannot express` — this capture format has no OSC 10/11 header
- **Geometry: asked for and got.** `new-session -x 80 -y 24`, which is the one thing this arm can do that the Ghostty arm cannot. The size above is what the *scene* reported, so the two disagreeing would be visible here
- **Config:** `-f /dev/null`, so this is tmux's own defaults and not a user's `~/.tmux.conf`. `default-terminal` was `tmux-256color`
- **Launch to capture:** 583 ms, of which `capture-pane` itself was 6 ms — reported, never gated. Headless: no window server, no automation grant, no focus taken
- **Trailing blanks:** `capture-pane` trims the default-styled ones the engine painted, where Ghostty's `vt` dump keeps them. It does **not** trim a *styled* blank, so an attribute leaking past its label is still counted as the disagreement it is

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
| overline | `Style::overline` | overline | overline | ✓ |
| under-sgl | `Style::underline` | underline | underline | ✓ |
| under-dbl | `Style::underline_double` | underline:2 | underline:2 | ✓ |
| under-dot | `Style::underline_dotted` | underline:4 | underline:4 | ✓ |

**11/11 agreed.**

A row that does not say which arm it came from is not a result, and a *missing* row reads as a win — which is the single easiest way for this directory to become dishonest. Every row of the scene is printed above whether it agreed or not, and a capture with fewer rows than the scene declared never reaches this table: it is refused as `FAILED` by the parser.
