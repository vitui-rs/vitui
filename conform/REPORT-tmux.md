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
- **Launch to capture:** 592 ms, of which `capture-pane` itself was 5 ms — reported, never gated. Headless: no window server, no automation grant, no focus taken
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
| overline | `Style::overline` | overline | `quirks.rs`'s fourth entry: tmux accepts SGR 53, stores it, hands it back to `capture-pane` and never forwards it, so the engine stopped sending it when production ticket 10 wired the mask. This arm reported 11/11 while it still did, and `fixtures/tmux-3.7c-scene01-attrs.vt` is that capture — observed: nothing | `by design` |
| under-sgl | `Style::underline` | underline | underline | ✓ |
| under-dbl | `Style::underline_double` | underline:2 | underline:2 | ✓ |
| under-dot | `Style::underline_dotted` | underline:4 | underline:4 | ✓ |

**10/10 agreed.**

**Not in that denominator: 1 of the scene's 11.** This arm declared before the run that it would not compare them, with the reason printed in the row beside what was nonetheless observed. `cannot ask` is a fact about the **instrument** — the emulator does the thing and this suite cannot see it. `by design` is a fact about **the engine** — it consulted `quirks.rs` and did not send it, so a `FAILED` would blame the terminal for a decision of ours. `compare/`'s three kinds of non-number have a word for neither, which is why `SCENES.md` grew two more. A row so declared that agrees anyway is reported `STALE` and counted as a failure, so a declaration cannot outlive what earned it.

A row that does not say which arm it came from is not a result, and a *missing* row reads as a win — which is the single easiest way for this directory to become dishonest. Every row of the scene is printed above whether it agreed or not, and a capture with fewer rows than the scene declared never reaches this table: it is refused as `FAILED` by the parser.
