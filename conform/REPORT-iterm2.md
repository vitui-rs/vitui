# The conformance suite — iTerm2

**Generated. It reports; it does not block.** One file per arm, because two arms writing one file means the rows of whichever ran first are gone — and *a missing row reads as a win*. **Every scene of `SCENES` is in here** for the same reason, and there is no flag to run one of them.

The version of software this repository does not control cannot gate its pull requests. So this file is committed and regenerated, and a disagreement arrives as a review-visible diff rather than as a red build. The gate is `cargo test` over `fixtures/`.

Read [`SCENES.md`](SCENES.md) first.

- **Arm:** iTerm2 3.6.11, `GetBufferRequest` with `include_styles`, over the Python API's unix socket
- **These rows are evidence about:** iTerm2 — **what iTerm2 stores**, read as its own cells rather than as an escape stream it re-serialised. The second capture surface here of that kind and the first that is a *projection* of the cell rather than the cell itself
- **Default colours, from the dump's own OSC 10/11:** fg `cannot express` — this capture format has no OSC 10/11 header, bg `cannot express` — this capture format has no OSC 10/11 header
- **The capture surface was chosen, and the loser is worth naming.** `iTerm2.sdef` offers `contents` and `text` on the `session` class as `type="text"`, with no styled variant anywhere on it — Terminal.app's surface exactly, and it would have made all eleven rows of scene 01 `cannot ask`. The Python API's `GetBufferRequest` takes an `include_styles` flag and answers with one `CellStyle` per run of cells, so **eight of the eleven are answerable**. This arm speaks the API and uses AppleScript for the window's life, its geometry and the cookie
- **Geometry: set, and then insisted on.** `columns` and `rows` are read-write on the `session` class, so this arm assigns 80x24 and reads it back, and a session that came back some other size fails the run. The second *emulator* arm that can — Terminal.app is the other, and this one sets the size **after** the command starts rather than before, because `create window … command` execs the scene directly where `do script` types into a login shell. The quiescence handshake is what observes the reflow
- **Environment: built, not inherited — and `COLORTERM` is deliberately left alone.** The command line unsets the 8 variables a *different* terminal or a previous run could have left behind. It does **not** unset `COLORTERM`, which the Terminal.app arm does: there the value comes from a login shell's profile, and here it comes from iTerm2 itself — the string and the value `truecolor` are both in the shipped binary, `launchctl getenv COLORTERM` is empty, and no shell runs between the terminal and the scene. Unsetting a terminal's own true statement about itself would be the lie the other arm avoids by leaving `TERM_PROGRAM` alone
- **Launch to capture:** 8418 ms, of which the API round trip alone was 165 ms — reported, never gated. The round trip is a cookie request over AppleScript, a websocket handshake on a unix socket, one frame out and one frame back
- **Surface, as the scene reported it: 80x24**, against a session this arm set to 80x24. The two are printed side by side because they are different measurements — one is what the API says the session is, the other is what `TIOCGWINSZ` told the process inside it, and an arm that printed one of them twice would not notice them coming apart
- **A never-written cell is not reported at all**, which is the kitty arm's finding reaching a third surface. A row's `code_points_per_cell` covers the cells that were written and stops: `bold` on an eighty-column screen comes back as four cells where the engine's own alt-screen paint comes back as eighty, because the engine paints every cell and a raw-byte probe writes six short rows. Nothing in these scenes turns on it — scene 04 compares with `trim_end` — and nothing here pads a short row, because the scene's declared row count is the refusal
- **The alt screen is what the buffer reads**, so scene 01's capture is the engine's page and not the shell's — and there is no shell here at all, because `create window … command` execs the scene. Scene 04 writes raw bytes to the primary screen with no alt-screen switch, and erases each of its rows before writing it

## Scene 01 — the eleven attribute bits, one per row

Surface: **80x24** cells, as the scene reported it.

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
| overline | `Style::overline` | overline | `quirks.rs`'s eighth entry, earned by this arm's first run and by iTerm2's own binary. `screen_char_t` — the cell struct, readable from the shipped binary's Objective-C type encoding — enumerates `bold faint italic blink underline image strikethrough underlineStyle0 invisible inverse guarded virtualPlaceholder rtlStatus underlineStyle1`, and there is no bit for an overline; the string `overline` appears **zero** times in the binary. The engine now withholds SGR 53, and `fixtures/iterm2-3.6.11-scene01-attrs.pb` is the capture taken while it still sent it — observed: nothing | `by design` |
| under-sgl | `Style::underline` | underline | underline | ✓ |
| under-dbl | `Style::underline_double` | underline:2 | `CellStyle.underline` is a `bool`. iTerm2's cell carries a three-bit `underlineStyle` and its own help text names `4:3  Curly underline`, so the terminal renders a double underline and the API says only *underlined*. A fact about the **capture format** and not about what iTerm2 draws — kitty's `CSI 4 : m` reached through a completely different surface, which is what makes the pair worth reading together — observed: underline | `cannot ask` |
| under-dot | `Style::underline_dotted` | underline:4 | the same `bool`, and the same three-bit field behind it. This is the row Alacritty's grid could answer and neither kitty's serialiser nor this projection can, so the suite now has all three readings of one question: rendered and spelled, rendered and unspelled, and stored as a bit of its own — observed: underline | `cannot ask` |

**8/8 agreed.**

**Not in that denominator: 3 of the scene's 11.** This arm declared before the run that it would not compare them, with the reason printed in the row beside what was nonetheless observed. `cannot express` is a fact about the **emulator** — it has no answer to give, and that is the answer. `cannot ask` is a fact about the **instrument** — the emulator does the thing and this suite cannot see it. `by design` is a fact about **the engine** — it consulted `quirks.rs` and did not send it, so a `FAILED` would blame the terminal for a decision of ours. Only the first of the three is one `compare/` had a word for, which is why `SCENES.md` grew the other two. A row so declared that agrees anyway is reported `STALE` and counted as a failure, so a declaration cannot outlive what earned it.

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
| keeps-style | **what does the blanked half wear, and the askable families split two-two.** The wide glyph carries a red background and the `x` does not. kitty 0.48.2 and WezTerm 20240203 keep the orphan's own background; Ghostty 1.3.1 and tmux 3.7c blank it to the SGR state in force; Terminal.app 2.15 cannot be asked. Reported, never compared — there is no single right answer to hold an arm to, and this row's value is that sentence rather than a tick | `"AB xCD"` | `"AB xCD"` — and cluster 2, the blanked half, wears **bg Indexed(1)** | ✓ |
| ruler | the row that makes a mis-sized or reflowed capture loud. This scene reports no surface size — a raw-byte probe has none to report — so a ruler that is short, wrapped or absent is what stands in for the handshake | `"0123456789"` | `"0123456789"` | ✓ |

**6/6 agreed.**

## Scene 05 — what does this emulator think this cluster is worth

**Answered by: iTerm2** — iTerm2 is an endpoint, so nothing sits between the scene's tty and it. The two channels are further apart here than on any arm but Terminal.app's: the buffer is a message iTerm2 composed about its own memory, and an in-band reply is the terminal answering a question on the wire.

This is the only scene here whose answer does not come back through a photograph. The scene homes the cursor to column 1, writes one cluster, and asks `CSI 6n`; the column that comes back is the **emulator's own UAX #11 verdict**, reached by the emulator's tables and reported by the emulator, with nothing of this repository's in the path. A `CSI c` behind the batch is the sentinel, so the read stops on an observed condition rather than on a delay.

Because the answers arrive in band on the scene's own tty, they come from the **innermost** terminal in the path — which is why the line above names a terminal rather than repeating this arm's title. A capture surface further out cannot change who answered.

### The three rows that are compared

Hand-written expectations, and deliberately **not** asked of the engine: a row that took its number from `width_of` would be checking the engine against itself, which is the arrangement this directory exists to break. They are here to say the probe is measuring an advance at all — a report that answered a constant would pass the first of them.

| row | asks | declared here | iTerm2 | |
|---|---|---|---|---|
| ascii | the control. A terminal that disagrees here is not answering about widths at all, and every other row of this table is about the instrument rather than about the emulator | 1 | 1 | ✓ |
| ascii-pair | **the control the control needs.** A probe that reported a constant would pass the row above; two columns is what proves the number moves with what was written | 2 | 2 | ✓ |
| cjk | UAX #11 `W`, unambiguous, and the one wide verdict no terminal in spec §10's tier 1 is known to differ on. Compared rather than surveyed because a terminal that answers 1 here is one this engine's `CHA` rule could not bound | 2 | 2 | ✓ |

**3/3 agreed.**

### The survey

**Reported, never failed, and no row here earns a `quirks.rs` entry.** The engine's tables are authoritative *by decision*: `ucd.rs` says so in as many words, pins three answers as policy rather than standard — ambiguous width is narrow, a cluster's width is its base's and never the sum, VS15 changes a presentation and not a width — and spec §8's `CHA`-after-non-ASCII rule is what **bounds** a disagreement instead of following it. So a terminal that answers differently below is not misbehaving in any sense this repository acts on, there is no mechanism that would read such a quirk row, and a `FAILED` here would be the instrument inventing a defect.

What the paragraph in `ucd.rs` cites for the disagreement is a **survey of 23 terminals in a research document**. This table is the first thing in this repository to observe any of it.

| cluster | code points | asks | the engine | iTerm2 | |
|---|---|---|---|---|---|
| `가` | U+AC00 | wide by the same class as the row above, reached through a different block | 2 | 2 | ✓ |
| `Ａ` | U+FF21 | U+FF21, UAX #11 `F` — the class that is wide for being a fullwidth *form* rather than for being East Asian | 2 | 2 | ✓ |
| `☂` | U+2602 | **UAX #11 class `A`, and the engine's answer is a policy rather than a standard**: `ucd.rs` pins ambiguous width as *narrow* by name. This is the row where a terminal running with an East Asian locale is entitled to disagree | 1 | 1 | ✓ |
| `é` | U+0065 U+0301 | *a cluster's width is its base's width, never the sum of its code points* — the second of `ucd.rs`'s three pinned policies. Windows Terminal is recorded in that file as drawing a combining mark at width 1 of its own | 1 | 1 | ✓ |
| `​` | U+200B | a cluster the cursor may not move for at all. The one row whose interesting answer is that nothing happened, which is also the one an instrument reading a missing reply would report by accident | 0 | 1 | **differs** |
| `👍` | U+1F44D | `Emoji_Presentation`, one scalar, no selector — the emoji case with nothing else in it | 2 | 2 | ✓ |
| `❤️` | U+2764 U+FE0F | **the headline of `ucd.rs`'s survey**: *only 7 of 23 surveyed widen a VS16 emoji correctly*. A default-text scalar plus U+FE0F, which forces emoji presentation and with it two columns | 2 | 2 | ✓ |
| `❤︎` | U+2764 U+FE0E | the third pinned policy: **VS15 does not change a width**, only a presentation. The same base as the row above with the other selector, so the two rows read together are what say whether a terminal has selectors at all | 1 | 1 | ✓ |
| `👨‍👩‍👧` | U+1F468 U+200D U+1F469 U+200D U+1F467 | the second named disagreement in `ucd.rs`: *kitty sums a ZWJ family emoji to 6 where the answer is 2*. Three emoji joined by two ZWJs, one cluster | 2 | 2 | ✓ |
| `🇯🇵` | U+1F1EF U+1F1F5 | two regional indicators, which the engine counts as a pair rather than as two clusters — the one place `cluster_width` looks past the base | 2 | 2 | ✓ |
| `👍🏽` | U+1F44D U+1F3FD | a base and a modifier, where the modifier is itself an emoji scalar. The row above answers whether a terminal joins; this one answers whether it modifies | 2 | 2 | ✓ |
| `1️⃣` | U+0031 U+FE0F U+20E3 | an ASCII base carried into emoji presentation by a selector and a combining enclosing keycap — the sequence whose base is one column on its own | 2 | 1 | **differs** |

**10 of 12 agree with the engine's tables.** That number is a fact about this terminal and about the disagreement's size; it is not a score and it is not a denominator anything is held to.

## Scene 06 — mode 2026, asked of the terminal rather than of its documentation

**Answered by: iTerm2** — iTerm2 is an endpoint, so nothing sits between the scene's tty and it. The two channels are further apart here than on any arm but Terminal.app's: the buffer is a message iTerm2 composed about its own memory, and an in-band reply is the terminal answering a question on the wire.

The second scene here whose answer does not come back through a photograph, and the first for a question that is not a width. The scene asks `CSI ? 2026 $ p` on its own tty and the terminal answers in band, so the capture surface, the window server and the automation grant are all out of the path — and the terminal that answers is the **innermost** one, which is why the line above names a terminal rather than repeating this arm's title.

### The state machine, and these five are compared

One batch — ask, set, ask, reset, ask, set, set, ask, reset, ask — with `CSI c` behind it, and the answers read positionally. **The expectations are DECRPM's own**, which is what makes this a comparison where scene 05's twelve rows are a survey: a terminal that reports the mode set after it was asked to reset it is wrong by the definition of the reply it sent, not by a table this repository chose. A terminal with no synchronised output answers `not recognised (0)` throughout, which is a legitimate answer — the arm then owes a `cannot express` declaration, and until it has one the rows are loud. **A terminal may also answer nothing at all**, which Terminal.app 2.15 does: its parser does not take `$` as an intermediate, so this is not a query it declines but one it never finishes reading. The sentinel is what makes that an observation rather than a timeout, and the rows below then read `no reply` against a `cannot express` the arm declared in advance.

| row | asks | declared here | iTerm2 | |
|---|---|---|---|---|
| before | whether the terminal recognises the mode at all, and what it says before anything has been done to it. A terminal without synchronised output answers `not recognised (0)` here, which is a legitimate answer and not a defect | reset (2) | reset (2) | ✓ |
| while-open | whether `CSI ? 2026 h` reached the state machine. This is the row that separates a terminal that *has* the mode from one that parses the sequence and throws it away | set (1) | set (1) | ✓ |
| after-close | whether `CSI ? 2026 l` reached it too. A terminal that opens and never closes is one where the engine's own frame framing leaves the mode set for ever | reset (2) | reset (2) | ✓ |
| opened-twice | whether a second `h` over an already-set mode is still simply set | set (1) | set (1) | ✓ |
| closed-once | whether one `l` undoes two `h`. A DEC private mode is not a counter, and a terminal that made it one would hold a frame past the close the engine sent | reset (2) | reset (2) | ✓ |

**5/5 agreed.**

### When the terminal let go of the flag — reported, never failed

**This is the flag and it is not the paint.** A force flush is a *rendering* event and nothing a process inside a terminal can ask reports whether the terminal painted; DECRQM reports a **mode**. What is below is when the terminal stopped reporting the mode as set — the event Ghostty's own source calls *reset the synchronized output flag*. A terminal could paint without clearing the flag or clear it without painting, and nothing here can tell those apart. It is a timing besides, and a timing is a report.

**One probe per open, and the control probe is why.** The polling instrument was written first: it opens one block and asks repeatedly, which costs one open where this costs seven. Polling a Ghostty 1.3.1 every 250 ms brought the reset forward from 1002 ms to under 517 ms, while the same polling left tmux 3.7c at 1007 ms and kitty 0.48.2 at 2261 ms — their documented figures. An instrument that polls is inside its own measurement, and the two families it happens not to disturb are exactly what would have made that invisible. So each row below is a fresh open, a wait, one question and a close, and the boundary between them is halved for. **The two clocks are both printed** because only one of them is sound for each answer: the terminal processed the question somewhere between them, a *set* is evidence back to the request and a *reset* is evidence forward to the reply, so the bracket takes one end from each column.

| open | held open for | answered at | iTerm2 |
|---|---|---|---|
| 1 | 50 ms | 51 ms | set (1) |
| 2 | 3000 ms | 3011 ms | set (1) |

**Still set at 3000 ms**, which is the largest delay this run opened a block for. Either this terminal's limit is beyond that or it has none, and this run cannot say which.

Nothing in this section moves the numerator or the denominator above it. A timing is a report, and a gate tuned to one is the flaky test this repository refuses by name.

A row that does not say which arm it came from is not a result, and a *missing* row reads as a win — which is the single easiest way for this directory to become dishonest. Every row of every scene is printed above whether it agreed or not, and a capture with fewer rows than the scene declared never reaches a table: it is refused as `FAILED` by the parser.
