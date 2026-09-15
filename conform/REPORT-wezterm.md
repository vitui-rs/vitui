# The conformance suite — wezterm

**Generated. It reports; it does not block.** One file per arm, because two arms writing one file means the rows of whichever ran first are gone — and *a missing row reads as a win*. **Every scene of `SCENES` is in here** for the same reason, and there is no flag to run one of them.

The version of software this repository does not control cannot gate its pull requests. So this file is committed and regenerated, and a disagreement arrives as a review-visible diff rather than as a red build. The gate is `cargo test` over `fixtures/`.

Read [`SCENES.md`](SCENES.md) first.

- **Arm:** wezterm 20240203-110809-5046fc22, `wezterm cli get-text --escapes`, over this run's own GUI socket
- **These rows are evidence about:** WezTerm. Its own cell state, re-serialised by the emulator that holds it — the same kind of evidence as the Ghostty and kitty arms', gathered over a control socket rather than an AppleScript surface, so no automation grant and no window z-order is in the loop
- **Default colours, from the dump's own OSC 10/11:** fg `cannot express` — this capture format has no OSC 10/11 header, bg `cannot express` — this capture format has no OSC 10/11 header
- **Geometry: asked for and got.** `--config initial_cols=80` and `--config initial_rows=24`, and `wezterm cli list` reported 80x24. **The options come before `start`** — they are top-level, and after the subcommand WezTerm exits with `unexpected argument '--config' found` before opening a window at all. The quiescence handshake is kept anyway: it is what proves the window stopped moving, and one command-line option is not a run
- **Config:** none. `-n` is `--skip-config`, so no `~/.wezterm.lua` is in the result — the tmux arm's `-f /dev/null` equivalent. There is no `remember_window_size` to disable: the two options above are a geometry rather than a request against a remembered one
- **Addressed by socket path, and this is the row worth reading twice.** `WEZTERM_UNIX_SOCKET=~/.local/share/wezterm/gui-sock-<pid>` and `--no-auto-start` on every command. Without them `wezterm cli` prefers the background mux server at `~/.local/share/wezterm/sock`, **starts one with `wezterm-mux-server --daemonize` if none is listening**, and photographs that daemon's default shell pane — exiting 0, with well-formed JSON and a screenful of somebody's prompt. Observed. `--class` does not fix it on macOS: that is a windowing-system class and routes nowhere here
- **Launch to capture:** 872 ms, of which `get-text` itself was 19 ms — reported, never gated. No automation consent dialog, no clipboard, no z-order
- **Trailing blanks: trimmed, like tmux's and unlike kitty's.** Each row ends where its content does, so the check that an attribute stopped where its label did has no padding to look at on this arm — what carries it instead is that WezTerm opens every row with `CSI 0 ; n m`, an explicit reset before the attribute, so a style leaking across a row boundary would be visible as the reset being absent
- **Both SGR spellings were sent, and 38 and 58 answer differently.** A raw `printf` control probe, no engine in it. **38:** `CSI 38;2;255;0;0 m` and `CSI 38:2::255:0:0 m` resolve to the same channel — WezTerm's serialiser emits nothing between two consecutive rows painted with the two spellings, which is a stateful serialiser saying *no change* — and the capture writes truecolour back as `38:2::255:0:0` and an indexed colour as `48;5;21` or the aixterm `91`, so it re-spells rather than echoing. **58:** neither `CSI 58;5;9 m` nor `CSI 58:2::255:0:0 m` comes back at all, in either spelling, so this capture cannot report an underline colour and nothing here says whether WezTerm parsed one. Asked separately because a terminal can want the semicolon form for 58 and never be asked about 38
- **Rows are CRLF-separated**, like Ghostty's dump and unlike kitty's, and the capture ends with a bare `ESC ( B CSI 0 m` after the blank rows. So `core.autocrlf` has something to rewrite in this arm's fixture and `.gitattributes`' `*.vt -text` is what stops it — the same line that lost twenty-three carriage returns out of the Ghostty capture before it existed
- **The two spec §10 claims this terminal carries are not askable here, and that is recorded rather than worked around.** *It ships kitty's keyboard encoding while implementing none of the flag stack*, and *the protocol is off by default*, are both about the keyboard. No scene in `SCENES.md` sends a keystroke — scene 01 is cell state, 04 a bisected pair, 05 an in-band width verdict, 06 mode 2026 — so this run neither confirms nor corrects either. A sixth scene could ask the second one with `CSI ? u`; the first needs a key pressed, and nothing here presses one

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
| overline | `Style::overline` | overline | `get-text --escapes` has no SGR 53 in its repertoire: `CSI 53 m` comes back as nothing, and `CSI 53 ; 1 m` comes back as `CSI 0 ; 1 m` — the bold survives, so the serialiser reached a cell it agrees is styled and wrote none of the overline. A fact about the **capture format**, and one this arm cannot promote to a `quirks.rs` row: WezTerm is an endpoint, so there is no far side to attribute from, and its shipped binary's string table holds `OVERLINE` as a Unicode character name, so the technique that settled kitty's two rows returns noise here — observed: nothing | `cannot ask` |
| under-sgl | `Style::underline` | underline | underline | ✓ |
| under-dbl | `Style::underline_double` | underline:2 | underline:2 | ✓ |
| under-dot | `Style::underline_dotted` | underline:4 | the same repertoire, one axis along. `CSI 4:1 m` comes back as bare `4` and `CSI 4:2 m` as `21`, so sub-parameters are normalised away entirely — and `4:3`, `4:4` and `4:5` come back as nothing at all, where kitty at least emits `CSI 4 : m` and proves a non-zero decoration is there. This capture cannot distinguish a dotted underline from no underline, so a row compared anyway would report *WezTerm does not render dotted underlines*, which is what the `quirks.rs` table exists to keep out — observed: nothing | `cannot ask` |

**9/9 agreed.**

**Not in that denominator: 2 of the scene's 11.** This arm declared before the run that it would not compare them, with the reason printed in the row beside what was nonetheless observed. `cannot express` is a fact about the **emulator** — it has no answer to give, and that is the answer. `cannot ask` is a fact about the **instrument** — the emulator does the thing and this suite cannot see it. `by design` is a fact about **the engine** — it consulted `quirks.rs` and did not send it, so a `FAILED` would blame the terminal for a decision of ours. Only the first of the three is one `compare/` had a word for, which is why `SCENES.md` grew the other two. A row so declared that agrees anyway is reported `STALE` and counted as a failure, so a declaration cannot outlive what earned it.

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

**Answered by: WezTerm** — WezTerm is an endpoint, so nothing sits between the scene's tty and it. It is also the arm where the two channels are furthest apart: the photograph goes through a serialiser with no spelling for SGR 53 or SGR 58 and none for a sub-parameter, and an in-band reply goes through none.

This is the only scene here whose answer does not come back through a photograph. The scene homes the cursor to column 1, writes one cluster, and asks `CSI 6n`; the column that comes back is the **emulator's own UAX #11 verdict**, reached by the emulator's tables and reported by the emulator, with nothing of this repository's in the path. A `CSI c` behind the batch is the sentinel, so the read stops on an observed condition rather than on a delay.

Because the answers arrive in band on the scene's own tty, they come from the **innermost** terminal in the path — which is why the line above names a terminal rather than repeating this arm's title. A capture surface further out cannot change who answered.

### The three rows that are compared

Hand-written expectations, and deliberately **not** asked of the engine: a row that took its number from `width_of` would be checking the engine against itself, which is the arrangement this directory exists to break. They are here to say the probe is measuring an advance at all — a report that answered a constant would pass the first of them.

| row | asks | declared here | WezTerm | |
|---|---|---|---|---|
| ascii | the control. A terminal that disagrees here is not answering about widths at all, and every other row of this table is about the instrument rather than about the emulator | 1 | 1 | ✓ |
| ascii-pair | **the control the control needs.** A probe that reported a constant would pass the row above; two columns is what proves the number moves with what was written | 2 | 2 | ✓ |
| cjk | UAX #11 `W`, unambiguous, and the one wide verdict no terminal in spec §10's tier 1 is known to differ on. Compared rather than surveyed because a terminal that answers 1 here is one this engine's `CHA` rule could not bound | 2 | 2 | ✓ |

**3/3 agreed.**

### The survey

**Reported, never failed, and no row here earns a `quirks.rs` entry.** The engine's tables are authoritative *by decision*: `ucd.rs` says so in as many words, pins three answers as policy rather than standard — ambiguous width is narrow, a cluster's width is its base's and never the sum, VS15 changes a presentation and not a width — and spec §8's `CHA`-after-non-ASCII rule is what **bounds** a disagreement instead of following it. So a terminal that answers differently below is not misbehaving in any sense this repository acts on, there is no mechanism that would read such a quirk row, and a `FAILED` here would be the instrument inventing a defect.

What the paragraph in `ucd.rs` cites for the disagreement is a **survey of 23 terminals in a research document**. This table is the first thing in this repository to observe any of it.

| cluster | code points | asks | the engine | WezTerm | |
|---|---|---|---|---|---|
| `가` | U+AC00 | wide by the same class as the row above, reached through a different block | 2 | 2 | ✓ |
| `Ａ` | U+FF21 | U+FF21, UAX #11 `F` — the class that is wide for being a fullwidth *form* rather than for being East Asian | 2 | 2 | ✓ |
| `☂` | U+2602 | **UAX #11 class `A`, and the engine's answer is a policy rather than a standard**: `ucd.rs` pins ambiguous width as *narrow* by name. This is the row where a terminal running with an East Asian locale is entitled to disagree | 1 | 1 | ✓ |
| `é` | U+0065 U+0301 | *a cluster's width is its base's width, never the sum of its code points* — the second of `ucd.rs`'s three pinned policies. Windows Terminal is recorded in that file as drawing a combining mark at width 1 of its own | 1 | 1 | ✓ |
| `​` | U+200B | a cluster the cursor may not move for at all. The one row whose interesting answer is that nothing happened, which is also the one an instrument reading a missing reply would report by accident | 0 | 0 | ✓ |
| `👍` | U+1F44D | `Emoji_Presentation`, one scalar, no selector — the emoji case with nothing else in it | 2 | 2 | ✓ |
| `❤️` | U+2764 U+FE0F | **the headline of `ucd.rs`'s survey**: *only 7 of 23 surveyed widen a VS16 emoji correctly*. A default-text scalar plus U+FE0F, which forces emoji presentation and with it two columns | 2 | 1 | **differs** |
| `❤︎` | U+2764 U+FE0E | the third pinned policy: **VS15 does not change a width**, only a presentation. The same base as the row above with the other selector, so the two rows read together are what say whether a terminal has selectors at all | 1 | 1 | ✓ |
| `👨‍👩‍👧` | U+1F468 U+200D U+1F469 U+200D U+1F467 | the second named disagreement in `ucd.rs`: *kitty sums a ZWJ family emoji to 6 where the answer is 2*. Three emoji joined by two ZWJs, one cluster | 2 | 2 | ✓ |
| `🇯🇵` | U+1F1EF U+1F1F5 | two regional indicators, which the engine counts as a pair rather than as two clusters — the one place `cluster_width` looks past the base | 2 | 2 | ✓ |
| `👍🏽` | U+1F44D U+1F3FD | a base and a modifier, where the modifier is itself an emoji scalar. The row above answers whether a terminal joins; this one answers whether it modifies | 2 | 2 | ✓ |
| `1️⃣` | U+0031 U+FE0F U+20E3 | an ASCII base carried into emoji presentation by a selector and a combining enclosing keycap — the sequence whose base is one column on its own | 2 | 1 | **differs** |

**10 of 12 agree with the engine's tables.** That number is a fact about this terminal and about the disagreement's size; it is not a score and it is not a denominator anything is held to.

## Scene 06 — mode 2026, asked of the terminal rather than of its documentation

**Answered by: WezTerm** — WezTerm is an endpoint, so nothing sits between the scene's tty and it. It is also the arm where the two channels are furthest apart: the photograph goes through a serialiser with no spelling for SGR 53 or SGR 58 and none for a sub-parameter, and an in-band reply goes through none.

The second scene here whose answer does not come back through a photograph, and the first for a question that is not a width. The scene asks `CSI ? 2026 $ p` on its own tty and the terminal answers in band, so the capture surface, the window server and the automation grant are all out of the path — and the terminal that answers is the **innermost** one, which is why the line above names a terminal rather than repeating this arm's title.

### The state machine, and these five are compared

One batch — ask, set, ask, reset, ask, set, set, ask, reset, ask — with `CSI c` behind it, and the answers read positionally. **The expectations are DECRPM's own**, which is what makes this a comparison where scene 05's twelve rows are a survey: a terminal that reports the mode set after it was asked to reset it is wrong by the definition of the reply it sent, not by a table this repository chose. A terminal with no synchronised output answers `not recognised (0)` throughout, which is a legitimate answer — the arm then owes a `cannot express` declaration, and until it has one the rows are loud. **A terminal may also answer nothing at all**, which Terminal.app 2.15 does: its parser does not take `$` as an intermediate, so this is not a query it declines but one it never finishes reading. The sentinel is what makes that an observation rather than a timeout, and the rows below then read `no reply` against a `cannot express` the arm declared in advance.

| row | asks | declared here | WezTerm | |
|---|---|---|---|---|
| before | whether the terminal recognises the mode at all, and what it says before anything has been done to it. A terminal without synchronised output answers `not recognised (0)` here, which is a legitimate answer and not a defect | reset (2) | reset (2) | ✓ |
| while-open | whether `CSI ? 2026 h` reached the state machine. This is the row that separates a terminal that *has* the mode from one that parses the sequence and throws it away | set (1) | reset (2) | **FAILED** |
| after-close | whether `CSI ? 2026 l` reached it too. A terminal that opens and never closes is one where the engine's own frame framing leaves the mode set for ever | reset (2) | reset (2) | ✓ |
| opened-twice | whether a second `h` over an already-set mode is still simply set | set (1) | reset (2) | **FAILED** |
| closed-once | whether one `l` undoes two `h`. A DEC private mode is not a counter, and a terminal that made it one would hold a frame past the close the engine sent | reset (2) | reset (2) | ✓ |

**3/5 agreed.**

### When the terminal let go of the flag — reported, never failed

**This is the flag and it is not the paint.** A force flush is a *rendering* event and nothing a process inside a terminal can ask reports whether the terminal painted; DECRQM reports a **mode**. What is below is when the terminal stopped reporting the mode as set — the event Ghostty's own source calls *reset the synchronized output flag*. A terminal could paint without clearing the flag or clear it without painting, and nothing here can tell those apart. It is a timing besides, and a timing is a report.

**One probe per open, and the control probe is why.** The polling instrument was written first: it opens one block and asks repeatedly, which costs one open where this costs seven. Polling a Ghostty 1.3.1 every 250 ms brought the reset forward from 1002 ms to under 517 ms, while the same polling left tmux 3.7c at 1007 ms and kitty 0.48.2 at 2261 ms — their documented figures. An instrument that polls is inside its own measurement, and the two families it happens not to disturb are exactly what would have made that invisible. So each row below is a fresh open, a wait, one question and a close, and the boundary between them is halved for. **The two clocks are both printed** because only one of them is sound for each answer: the terminal processed the question somewhere between them, a *set* is evidence back to the request and a *reset* is evidence forward to the reply, so the bracket takes one end from each column.

| open | held open for | answered at | WezTerm |
|---|---|---|---|
| 1 | 50 ms | 5060 ms | no reply |
| 2 | 3000 ms | 3002 ms | reset (2) |

**No bracket:** the probe at 50 ms got no reply, so the sequence has a hole and the event may be on either side of it

Nothing in this section moves the numerator or the denominator above it. A timing is a report, and a gate tuned to one is the flaky test this repository refuses by name.

A row that does not say which arm it came from is not a result, and a *missing* row reads as a win — which is the single easiest way for this directory to become dishonest. Every row of every scene is printed above whether it agreed or not, and a capture with fewer rows than the scene declared never reaches a table: it is refused as `FAILED` by the parser.
