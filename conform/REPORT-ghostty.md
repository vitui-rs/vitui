# The conformance suite — Ghostty

**Generated. It reports; it does not block.** One file per arm, because two arms writing one file means the rows of whichever ran first are gone — and *a missing row reads as a win*. **Every scene of `SCENES` is in here** for the same reason, and there is no flag to run one of them.

The version of software this repository does not control cannot gate its pull requests. So this file is committed and regenerated, and a disagreement arrives as a review-visible diff rather than as a red build. The gate is `cargo test` over `fixtures/`.

Read [`SCENES.md`](SCENES.md) first.

- **Arm:** Ghostty 1.3.1, `write_screen_file:…,vt`
- **These rows are evidence about:** Ghostty. Its own cell state, re-serialised by the emulator that holds it
- **Default colours, from the dump's own OSC 10/11:** fg `#eaeaea`, bg `#000000`
- **Launch to capture:** 2542 ms, of which the capture round trip alone was 893 ms — reported, never gated. It is AppleScript round trips and a window opening
- **Geometry:** not ours to set. `surface configuration` offers a font size and no rows or columns, so the scene draws at the top left of whatever it is given

## Scene 01 — the eleven attribute bits, one per row

Surface: **156x41** cells, as the scene reported it.

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

## Scene 04 — a pair bisected, and what the terminal does with the orphan

Surface: **raw**. This scene is raw bytes rather than the engine, so it has no surface size to report — see below.

**The only scene here that does not drive the engine, and it cannot.** The engine's drawing verbs repair a bisected pair before anything is serialised, so an engine-driven scene could photograph only the repair. The question is what a terminal does when it is handed the bytes anyway — which is what the engine's mirror would have believed had the repair stopped at a `View::child` clip, and is architecture ticket 20's whole subject.

Every row is `AB漢CD` — `A` at column 0, `B` at 1, the wide glyph across 2 and 3, `C` at 4, `D` at 5 — and then one write over one half of it. **The row is compared as text**: a grid-to-text dump emits no padding cell for a double-width glyph, so *what is at column 3* is not askable of it, and ASCII sentinels turn the question into one that is.

| row | asks | expected | observed | |
|---|---|---|---|---|
| control | nothing is overwritten, so a disagreement here says the other five rows are about the capture rather than about the terminal | `"AB漢CD"` | `"AB漢CD"` | ✓ |
| over-cont | **the ticket's own case.** A narrow cluster lands on the continuation at column 3. Does the terminal blank the head at column 2, or leave a wide head with nothing after it? | `"AB xCD"` | `"AB xCD"` | ✓ |
| over-head | the mirror image: a narrow cluster lands on the head at column 2. Does the terminal blank the continuation at column 3? | `"ABx CD"` | `"ABx CD"` | ✓ |
| over-wide | rule 5 — a wide cluster landing across an existing pair, which orphans a half at each end at once | `"AB 漢D"` | `"AB 漢D"` | ✓ |
| keeps-style | **what does the blanked half wear, and the three families disagree.** The wide glyph carries a red background and the `x` does not. kitty 0.48.2 keeps the orphan's own background; Ghostty 1.3.1 and tmux 3.7c blank it to the SGR state in force. Reported, never compared — there is no single right answer to hold an arm to, and this row's value is that sentence rather than a tick | `"AB xCD"` | `"AB xCD"` — and cluster 2, the blanked half, wears **nothing** | ✓ |
| ruler | the row that makes a mis-sized or reflowed capture loud. This scene reports no surface size — a raw-byte probe has none to report — so a ruler that is short, wrapped or absent is what stands in for the handshake | `"0123456789"` | `"0123456789"` | ✓ |

**6/6 agreed.**

A row that does not say which arm it came from is not a result, and a *missing* row reads as a win — which is the single easiest way for this directory to become dishonest. Every row of every scene is printed above whether it agreed or not, and a capture with fewer rows than the scene declared never reaches a table: it is refused as `FAILED` by the parser.
