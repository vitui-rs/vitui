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

## Status: every stage, four scenes and six emulator families

**Seven arms, seven committed reports, six emulator families, three `quirks.rs` entries, one closed
architecture ticket, a citation that reproduces on three families by three mechanisms, two parser
defects and one defect in the engine's own output came out of them.** Ninety-seven tests, no emulator
in the loop for any of them.

| arm | scene 01 | scene 04 | scene 05 | scene 06 | what its rows are about |
|---|---|---|---|---|---|
| `cargo run --example ghostty` | **11/11** | **6/6** | **3/3**, and 12 of 12 surveyed | **5/5**, flag reset 879–973 ms | Ghostty 1.3.1's own cell state |
| `cargo run --example tmux` | **10/10**, one `by design` | **6/6** | **3/3**, and 12 of 12 surveyed | **5/5**, flag reset 971–1064 ms | what tmux 3.7c *stores* — `capture-pane` re-serialises tmux's grid |
| `cargo run --example ghostty -- --through-tmux` | **10/10**, one `by design` | **6/6** | **3/3**, and 12 of 12 surveyed — **tmux's, not Ghostty's** | **5/5**, 971–1063 ms — **tmux's, not Ghostty's** | what tmux 3.7c *forwards*, read through Ghostty |
| `cargo run --example kitty` | **8/8**, one `cannot ask`, two `by design` | **6/6** | **3/3**, and 12 of 12 surveyed | **5/5**, flag reset 1985–2085 ms | kitty 0.48.2's own cell state |
| `cargo run --example terminal` | **0/0**, eleven `cannot ask` | **6/6** | **3/3**, and **8 of 12** surveyed | **0/0**, five `cannot express` | Terminal.app 2.15's screen as plain text, and its own in-band answers |
| `cargo run --example wezterm` | **9/9**, two `cannot ask` | **6/6** | **3/3**, and **10 of 12** surveyed | **3/5**, no bracket | WezTerm 20240203's own cell state, and the first arm to answer scene 06 **wrongly** |
| `cargo run --example alacritty` | **9/9**, two `by design` | **6/6** | **3/3**, and **8 of 12** surveyed | **3/5**, reply held to 150 ms | what Alacritty 0.17.0 *stores* — `--ref-test` serialises the `Term`'s own grid, so no serialiser of the emulator's is in the path |

**An arm runs every scene or it is not a run**, and one report per arm holds a section for each —
same rule, same reason, as one file per arm: a section that is missing reads as a win. There is
deliberately no flag to run one scene.

**Scene 04 closed [architecture ticket 20](../.scratch/vitui-engine-architecture/issues/20-a-pair-bisected-by-a-child-clip.md)**,
which is the first decision on that map settled by asking a terminal rather than by argument. All four
arms agree that a terminal blanks the orphaned half of a bisected pair itself, in both directions, and
none of them has a clip to consult. **They disagree about what the blanked cell wears** — kitty,
WezTerm and Alacritty keep the orphan's background, Ghostty and tmux blank to the SGR state in force
— which is the finding that turned *the engine may as well repair* into *the engine must*. **Three
against two**, and the direction is the finding rather than the count: every askable arm added since
the table had four has landed on the keeping side, so *kitty has a bug* was a reading available only
while the sample was small. See `FINDINGS.md`.

It is also the only scene that does **not** drive the engine, and it cannot: the engine repairs a
bisected pair before it serialises anything, so an engine-driven scene could photograph only the
repair. That is why wiring the answer took no measurement away from it, where the `--through-tmux`
arm's overline row lost its.

**Scene 05 is the only one whose answer does not come back through a photograph**, and that is worth
as much as its numbers. It writes fifteen clusters at column 1 with `CSI 6n` behind each and `CSI c`
behind the batch; the column that comes back is the **emulator's own UAX #11 verdict**, with none of
this repository's tables in the path. **The capture surface is out of it entirely** — Ghostty's
`write_screen_file` and its undocumented `vt` writer, kitty's remote-control socket, `capture-pane` —
so the arm launches the scene and does nothing else. Whatever an arm needed in order to *open* a
terminal it still needs: the Ghostty arm's automation grant is in the path here as much as anywhere,
and the claim is about the capture and about nothing else. It is why the scene-05 column above is
headed by **who answered** rather than by the arm, and why the `--through-tmux` row says tmux: a
cursor report never leaves the innermost terminal, and that arm's two fixtures are byte-identical to
the plain tmux arm's.

**Scene 06 is the second one built on that property, and the first for a question that is not a
width.** It asks `CSI ? 2026 $ p` and reads what the terminal says about mode 2026 — five rows
compared against DECRPM's own definitions, and a bracket for when the terminal stops reporting the
mode as **set**. `quirks.rs` carries a four-row table of the force-flush limits and every row of it
is *the implementation, read*, because a force flush is a **rendering** event that nothing inside
the terminal can observe. **The flag is not the paint** and this scene never claims otherwise; what
it adds is that three of those numbers now have a measurement beside them. tmux and kitty land on
theirs; **Ghostty's sits below its own `sync_reset_ms = 1000`.**

Its finding is about the instrument. The obvious shape — open one block, poll it — is **wrong on one
of the three families**: polling Ghostty every 250 ms put the reset before 517 ms where one probe
per open puts it between 879 and 973, while the same polling left tmux and kitty on their documented
figures. *An instrument that polls is inside its own measurement*, and the two families it happens
not to disturb are what would have made that invisible. Third time in this directory that running
the control before the instrument is what separated the terminal's behaviour from the suite's.

Three of its fifteen rows are compared against hand-written numbers and twelve are **surveyed**. The
survey never fails, because `ucd.rs` decides that the engine's tables are authoritative and §8's
`CHA`-after-non-ASCII rule bounds the disagreement rather than following it — so a `FAILED` there
would be the instrument inventing a defect. **What it found is that the two disagreements `ucd.rs`
cites do not reproduce**: those three families widen a VS16 emoji, and kitty 0.48.2 answers 2 for a ZWJ
family where that paragraph records 6. The citation is a survey in a research document; this is the
first thing here to look. `ucd.rs` now says so, the decision is untouched, and `FINDINGS.md` records
what the survey needs next — an arm that disagrees.

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
cd conform && cargo run --example terminal        # a window, an automation grant, and no style
cd conform && cargo run --example wezterm         # a window, a control socket, and no grant
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

**The Terminal.app arm is the fourth family and the one that disagrees.** It needs a window server
and an automation grant like the Ghostty arm, and its capture surface carries **no style at all** —
`contents` is `type="text" access="r"` and the `tab` class has no styled variant — so scene 01 is
eleven `cannot ask` rows and scene 04's reported style is unreportable. What it *can* do is the
reason it exists: scenes 05 and 06 are answered in band on the scene's own tty, where no capture
surface is in the path, and it is the first arm here to disagree with the other three about anything.
It is also the only arm that can be **handed** a size and then insist on it — `number of rows` and
`number of columns` are read-write on its `window` class, where kitty's geometry is a request
reported back.

**The WezTerm arm is the fifth family and the one whose setup risk is the interesting part.** Like
kitty's it reaches the terminal over a control socket, so there is no TCC grant and no z-order in
the loop, and like kitty's it can be handed a size in cells — `--config initial_cols=80`, before
the `start` subcommand, because after it WezTerm exits with `unexpected argument` before opening a
window. `-n` is `--skip-config`, the tmux arm's `-f /dev/null` for free.

What is unlike every other arm is **who answers**. `wezterm cli` prefers a background *mux server*
at `~/.local/share/wezterm/sock` over the GUI this run started, and **starts one with
`wezterm-mux-server --daemonize` if none is listening** — so a run written the obvious way exits 0,
returns well-formed JSON, and photographs a daemon's default shell. Observed. The arm addresses its
own child's `gui-sock-<pid>` through `WEZTERM_UNIX_SOCKET` and passes `--no-auto-start` on every
command, and `--class` is not the answer: on macOS it is a windowing-system class and routes
nowhere. **It is the sharpest instance of *a missing row reads as a win* in this directory**, because
the wrong version succeeds.

**The Alacritty arm is the sixth family and the only one whose capture is not an escape stream.**
`alacritty --ref-test` writes the `Term`'s grid to `./grid.json` when its last window closes, one
JSON object per cell — so there is no serialiser of the emulator's between the cell and the reader,
which is the property kitty's and WezTerm's `cannot ask` rows exist because their arms lack. It is
read by `src/grid.rs` behind `Dialect::AlacrittyGrid`, and the fixtures are `.json` for the reason
`.cpr` and `.decrqm` are named apart.

What is unlike every other arm is **when** the capture happens: there is no socket to ask during a
run, and the file is written from the one branch of the event loop where the last window has closed.
So the arm stops the scene and waits for the process to exit, and it stops it with `SIGSTOP` and
then `SIGKILL` rather than asking it to finish — a clean exit would leave the alternate screen and
hand back a grid of the shell. Three more things it needs, each found by running it: a private
`HOME`, because Alacritty `chdir`s there on macOS before writing `./grid.json`; `scrolling.history=0`,
because the ref test materialises the whole scrollback and the first probe wrote 111 MB for a
four-row screen; and a **built** environment, which on this arm is load-bearing rather than
hygienic — an inherited `TERM_PROGRAM=ghostty` makes the engine inside the Alacritty window detect
Ghostty.

Its capture carries style, in a **classic** SGR repertoire — sub-parameters normalised away (`4:1`
comes back as bare `4`, `4:2` as `21`), and no spelling at all for `4:3`, `4:4`, `4:5`, SGR 53 or
SGR 58. Two of scene 01's rows are therefore `cannot ask`, and unlike kitty's they cannot be
promoted to a `quirks.rs` row: there is no far side and no second source. It is also the first arm
to answer scene 06 **wrongly** — mode 2026 reported reset while set, with three control probes
separating that from every innocent reading, and still no quirk entry, because `Detected::mode`
reads `2` as *available* and nothing is degraded.

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
`fixtures/kitty-0.48.2` becomes `fixtures/kitty-0.48.2-scene04-pairs.vt`. **The extension says which
channel it came through**: a `.vt` is a screen and reads through `parse`, a `.cpr` is a terminal's own
answers and reads through `cursor_reports`, and handing either to the other produces a refusal rather
than a wrong number — which `tests.rs` asserts by name. It is **opt-in and never
automatic**: a driver that rewrote its own fixtures on every run would turn the gate into a mirror.
**And it will not overwrite one.** *A capture is never regenerated to make something pass* was a
sentence in three files; it is a branch now. An existing fixture is left alone and said so on stderr
rather than failing the run — adding a scene means running an arm whose other scenes are already
captured.

Stage 3 (CPR and the width questions) landed 2026-08-29 as scene 05, and **stage 5 (mode 2026) the
same day as scene 06** — which was recorded as *if at all*, on an expectation that the AppleScript
jitter made it unanswerable. It did not need the capture surface: the terminal answers DECRQM in
band, so the whole of that jitter is out of the path.

**Stage 4 closed on 2026-08-30 with the Terminal.app arm, and ticket 04 with it.** Ticket 04 had
recorded that arm as *glyph-grid scenes only* from its `sdef`, and scene 05 had already made the
sentence too small: *plain text only* is a fact about the **capture surface**, and an in-band
question has none in its path. So the arm answers scene 05 in full, scene 06 in full, scene 04 as
text, and only scene 01 not at all — and it is the arm the survey needed. Three families that agree
cannot say whether they are agreeing with the engine's tables or reflecting them; **Terminal.app 2.15
disagrees on four of scene 05's twelve surveyed rows**, all four by summing a cluster's code points
where the others take the base's width. `ucd.rs`'s headline citation reproduces on it — and again on
WezTerm 20240203 and again on Alacritty 0.17.0 (both 2026-09-03), by two further mechanisms, which
is what makes it a population rather than an outlier. **Alacritty answers 6 for a ZWJ family, the
figure that citation attributes to kitty**, and the kitty measured here answers 2. See
[ticket 04](../.scratch/vitui-engine-production/issues/04-the-conformance-harness.md) and
`FINDINGS.md`, 2026-08-30.

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
| `wezterm-20240203-110809-5046fc22-scene01-attrs.vt` | the same scene as **WezTerm's own grid** hands it back: nine of the eleven, overline and the dotted underline bare, the double underline spelled **`21`** rather than `4:2`, and every row led by **`ESC ( B`**. Those last two are the bytes the parser grew two arms for, and no other fixture here contains either — `tests.rs` asserts both facts, because a parser arm added for a capture and then tested only synthetically leaves *WezTerm writes it this way* resting on a sentence |
| `wezterm-20240203-110809-5046fc22-scene04-pairs.vt` | the same scene as WezTerm holds it: the four text rows agreeing with all four other families, and the blanked half wearing **the orphan's own background** — the fifth family, landing on kitty's side and turning that row into a two-two split |
| `wezterm-20240203-110809-5046fc22-scene05-widths.cpr` | the same fifteen as WezTerm answered them. **Two of the twelve surveyed rows disagree and they are not Terminal.app's four**: a VS16 pair at **1** and a keycap sequence at **1**, where every summing case — ZWJ family, skin tone, zero-width space — matches the engine exactly. Terminal.app sums code points; this terminal takes the base's width and then lets no **variation selector** widen it |
| `wezterm-20240203-110809-5046fc22-scene06-sync.decrqm` | **five answers and every one of them `reset`**, two of them given while the mode was set. The first capture here that is a *wrong* answer rather than a missing one, and the fixture the three control probes in `SCENES.md` §06 exist to interpret |
| `alacritty-0.17.0-scene01-attrs.json` | the same scene as **Alacritty's own grid** holds it, and the first fixture here that is not an escape stream: nine of the eleven, **blink and overline with no flag at all**, and a dotted underline reported correctly — the row two other arms declared `cannot ask`. Captured while the engine still sent both missing bits, which is what makes it the evidence for `quirks.rs`'s seventh entry; the live arm reads `by design` now and this file is why it may not be regenerated |
| `alacritty-0.17.0-scene04-pairs.json` | the same scene as Alacritty holds it: the four text rows agreeing with all five other families, the blanked half wearing **the orphan's own background** — the third family on that side — and a `WIDE_CHAR_SPACER` in the bytes, which is the cell the reader drops so this arm's rows are comparable with the five that emit no padding cell |
| `alacritty-0.17.0-scene05-widths.cpr` | the same fifteen as Alacritty answered them. **Four of the twelve surveyed rows disagree and they are neither Terminal.app's four nor WezTerm's two**: a ZWJ family at **6** — the figure `ucd.rs` attributes to kitty — a skin tone at **4**, a VS16 pair at **1** and a keycap at **1**, with the zero-width space at **0**. It sums the code points like Terminal.app and costs a zero-width one nothing, which is the row that tells the two summers apart |
| `alacritty-0.17.0-scene06-sync.decrqm` | **five answers and every one of them `reset`**, two given while the mode was set — the second capture of that misbehaviour, on an unrelated codebase, where `Term::report_private_mode` answers this mode with a constant |
| `terminal-2.15-scene01-attrs.vt` | the same scene as **Terminal.app's AppleScript surface** hands it back: eleven labels and **not one attribute anywhere**, because `contents` is `type="text"`. The bytes the arm's eleven `cannot ask` rows rest on, and the reason the declaration is gated rather than only stated — no cluster in this file carries a style, and all eleven labels are where the scene put them |

| `ghostty-1.3.1-scene04-pairs.vt` | scene 04 as Ghostty gave it back: the orphaned half blanked in both directions, and blanked **to the SGR state in force** rather than to the glyph's own red background |
| `kitty-0.48.2-scene04-pairs.vt` | the same scene as kitty holds it: the same four text rows, and the blanked half wearing **the orphan's own background**. The one row on which the three families differ, and the reason the engine may not delegate the repair |
| `tmux-3.7c-scene04-pairs.vt` | the same scene as tmux's own grid holds it — agreeing with Ghostty on all five |
| `ghostty-1.3.1-via-tmux-3.7c-scene04-pairs.vt` | the same scene **through** tmux into Ghostty, agreeing with both. It is also the arm that found the probe's own defect: `CSI 2 J` pushed the picture into tmux's history and the capture came back with the scene on it twice |
| `terminal-2.15-scene04-pairs.vt` | the same scene on a **fourth VT lineage**, agreeing with all three on the four text rows — the first terminal to agree with architecture ticket 20's answer that is not a recent reimplementation. The `keeps-style` row is `not askable` here and always will be |
| `ghostty-1.3.1-scene05-widths.cpr` | scene 05 as Ghostty answered it — fifteen `CSI 6n` replies and the device-attributes sentinel behind them. **Not a screen**: these are the terminal's own answers, and the file is what a width measurement with none of our tables in it looks like |
| `kitty-0.48.2-scene05-widths.cpr` | the same fifteen as kitty answered them, including **2** for a ZWJ family emoji where `ucd.rs` records kitty summing it to 6 |
| `tmux-3.7c-scene05-widths.cpr` | the same fifteen as tmux answered them, and a `?1;2;4c` sentinel — a VT100 with AVO |
| `terminal-2.15-scene05-widths.cpr` | **the file that made the survey a survey.** Four of the twelve surveyed rows disagree with the other three arms, all four by summing a cluster's code points: a ZWJ family at **8**, a skin tone at **4**, a zero-width space at **1**, and a VS16 pair at **1** — which is `ucd.rs`'s headline citation reproducing on the fourth family after it did not reproduce on the first three |
| `ghostty-1.3.1-via-tmux-3.7c-scene05-widths.cpr` | **byte-identical to the file above**, which is the evidence that a cursor report never leaves the innermost terminal. The two arms' *screen* captures are two serialisations of two grids; their reply captures are one terminal answering twice |
| `ghostty-1.3.1-scene06-sync.decrqm` | scene 06 as Ghostty answered it — five DECRQM replies about mode 2026 and the device-attributes sentinel behind them. The evidence that the mode `serial.rs` wraps every frame in is one this terminal has, and that its state machine tracks both the `h` and the `l` |
| `kitty-0.48.2-scene06-sync.decrqm` | the same five as kitty answered them |
| `tmux-3.7c-scene06-sync.decrqm` | the same five as tmux answered them |
| `terminal-2.15-scene06-sync.decrqm` | **seven bytes, and all seven of them are the sentinel.** Terminal.app 2.15 answered none of the five, because it has no synchronised output and its parser does not take `$` as an intermediate. The device-attributes reply is what makes that an observation rather than a timeout, and `ModeError::Unanswered` is the refusal that says *this terminal has no such report* where `ModeError::Count` would have said *this run lost five answers* |
| `ghostty-1.3.1-via-tmux-3.7c-scene06-sync.decrqm` | **byte-identical to the file above**, for scene 05's reason: a DECRQM reply, like a cursor report, never leaves the innermost terminal. **Part B has no fixture on any arm** — it is a timing, a timing is a report, and a report is not gated |

Raw bytes, as captured. Do not regenerate them to make a test pass: they are evidence, and a fixture
that moves because the code moved is not evidence of anything. `save_if_asked` now refuses to.

**`.gitattributes` marks `*.vt` and `*.cpr` as `-text`, and that line is load-bearing.** Without it,
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

- **Which column a glyph is in, *from a dump*.** Both capture formats emit a double-width glyph
  with no padding cell and no continuation marker, so architecture ticket 20 was not answerable by
  any dump — production ticket 06's ASCII sentinels are what settled it. **Scene 05 lifts the
  general form of this limit and lifts it only there**: a cursor report gives the emulator's own
  column with no width table in the path, but it can only ever answer *what did that cluster
  advance*, never *what is at column 3*. See `FINDINGS.md`.
- **A width the terminal renders but does not advance for.** A cursor report is the emulator's
  arithmetic and not its glyph cache, so a terminal that advances two columns and paints one is a
  terminal this scene calls correct — the same gap as the `vt` dump's *recorded, not drawn*, one
  axis over.
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
- ~~**A disagreement, so far.**~~ **Answered 2026-08-30.** Four arms had answered scene 05
  identically, so its table had no spread in it and fifteen ticks could not distinguish *the
  terminals agree* from *the questions are easy*. `FINDINGS.md` named two candidates for a column
  that would disagree; the first of them — a different VT lineage — was built, and Terminal.app 2.15
  disagrees on four of the twelve surveyed rows. **The second candidate is still open**: a locale
  this suite does not set, which is the one row (`ambiguous`) where a terminal is *entitled* to
  disagree and the engine answers by policy.
- **Anything about timing.** Mode 2026 is stage 5, and the expectation is already recorded: the
  AppleScript round trip's jitter is the same order as Alacritty's 150 ms force-flush limit, so the
  sub-200 ms end may be unanswerable on this machine.
- **What a terminal does with a sequence it does not implement — until an arm looks.** No scene here
  asks that, and it is where the sharpest finding of 2026-08-30 came from: a *control probe* showed
  Terminal.app 2.15 printing the payload of the engine's own `DCS + q` and the final `p` of each of
  its seven DECRQM queries as visible text, on the **primary** screen, before `?1049h` is sent. Every
  gate in this workspace replays the engine's bytes through the engine's own model of a terminal, and
  a model that ignores an unknown sequence — which is correct for a model — cannot represent one that
  prints it. Production ticket 12.
