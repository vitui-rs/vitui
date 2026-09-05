# Scenes

**Normative.** A scene is a described picture — a size, an initial screen, and a sequence of changes —
and an arm may reach it any way it likes. Nothing here says *verb*, *layer* or *damage*, because the
comparison spans this engine and the emulators it talks to, and those words would decide it in advance.

This list is deliberately **not** `crate::scenes`' twelve. Those are fixed at 300×80, are tuned for the
byte budget and the damage model, and **not one of them contains a CJK glyph, an underline colour or
mode 2026** — the four things this directory exists to ask about. The last of those is scene 06. `compare/` reached the same
conclusion and wrote its own list for the same reason.

**One scene was considered and declined**, and it is recorded here rather than left as a gap: *does
anything the engine sends reach the user's own page*, the property behind production ticket 12's
defect. It is mechanically possible — a marker before `attach`, a marker after the detach, and a row
equality on a text screen, which is the one thing every capture surface here can read. It is declined
because five of the six arms consume the probe silently, so five of its six rows would be green
whether or not the engine is defective, and the one arm that discriminates discriminates on a
*consequence* of the byte order rather than on the byte order. The property is stated where it can
fail on the edit instead: `scripts/page-order-gate.sh`, over a real pty. See `FINDINGS.md`.

## What each arm actually measures

| arm | measures | |
|---|---|---|
| **tmux** | **what tmux stores** | `capture-pane` re-serialises *tmux's* grid, so the emulator behind it never sees the engine's bytes. A legitimate target — tmux is in §10's tier-1 list — and **never a proxy for one.** Headless, and the only arm that can *set* the pane size, which makes it the CI-shaped one |
| **Ghostty** | Ghostty | its own screen dump, over an AppleScript surface. Needs a window server and a macOS automation grant |
| **Ghostty-via-tmux** | **what tmux forwards** | the engine into tmux into Ghostty, photographing Ghostty. Ghostty alone agrees 11/11, so a disagreement here is tmux's. **The only arm that can see this**, because `capture-pane` and tmux's redraw path are different code and `attrs_dropped` is about what is rendered |
| **kitty** | kitty | `kitten @ get-text --ansi` over a unix socket. No automation grant, no clipboard, no z-order — and **the only *emulator* arm that can be handed a size**, in cells, which Ghostty's AppleScript surface cannot do |
| **Terminal.app** | Terminal.app | `contents of selected tab`, over AppleScript, and **plain text only** — the `tab` class has no styled variant. **A fourth VT lineage**, and the first arm here that disagrees with the others. Scene 01 is `cannot ask` on all eleven rows; scenes 05 and 06 are answered **in full**, because a question asked in band needs no capture surface at all. The only *emulator* arm that can be **handed** a size and then insist on it |
| **WezTerm** | WezTerm | `wezterm cli get-text --escapes`, over **this run's own GUI socket**. **A fifth family**, and the second that can be handed a size in cells. Its capture carries style but in a **classic** SGR repertoire — no SGR 53, no SGR 58, and sub-parameters normalised away — so two of scene 01's rows are `cannot ask`; and it is the first arm to answer scene 06 *wrongly* rather than not at all. **The socket is the whole of its setup risk**: `wezterm cli` prefers a background mux server, starts one if none is listening, and photographs that daemon's own shell — exiting 0, with valid JSON |
| **Alacritty** | **what Alacritty stores** | `alacritty --ref-test`, and the `grid.json` it writes when its last window closes. **A sixth family and the first capture here that is not an escape stream**: the `Term`'s grid, one JSON object per cell, with no serialiser of the emulator's in the path — so an underline colour is a field of the cell rather than a sequence that has to survive a re-serialisation, and it is the first arm that could be *asked* about a dotted underline, and the one whose two bare rows are a `quirks.rs` entry rather than a `cannot ask`. What it costs is the tmux arm's caveat arriving on an emulator: a grid is what the terminal **stores**. **The capture is the window closing**, which is the whole of its shape — there is no socket to ask during a run |
| **iTerm2** | iTerm2 | `GetBufferRequest` with `include_styles`, over the Python API's unix socket — protobuf behind an RFC 6455 handshake, and **the second capture here that is not an escape stream**. A `CellStyle` per run of cells is the cell's own fields, so no serialiser of the emulator's is in the path; what it is not is the cell *itself*, which is the difference from a grid — `CellStyle` is a **projection** of `screen_char_t`, and a bit the cell holds and the projection drops is a reading this format has and Alacritty's does not. Its two `cannot ask` rows and its one quirk entry are the two sides of that. **The only surface here that distinguishes a cell the terminal emptied from a space somebody wrote**, and the comparison throws that away on purpose. The second *emulator* arm that can be handed a size and then insist on it |

**Scenes 05 and 06 are answered by a different party than the rows above it**, and only one arm is
affected. Their answers come back **in band**, on the scene's own tty, so they come from the
**innermost** terminal in the path and a capture surface further out cannot change that. For every arm but one
that is the same terminal the photograph measures. For **Ghostty-via-tmux** it is not: the
photograph sees what tmux *forwards*, and an in-band reply never leaves tmux — the fixtures are
**byte-identical** to the plain tmux arm's, device attributes included, which `tests.rs` asserts for
both scenes. Each report heads those scenes' columns with **who answered** rather than with the
arm's title.

A row that does not say which of these it came from is not a result. **The tmux pair is why that
sentence needed a fourth row**: *tmux* and *what tmux does to a terminal downstream of it* gave
different answers on the same scene, and a table with one tmux column could only have printed one of
them.

## Five kinds of non-number, and `compare/` supplied three

`cannot express` is a fact about the emulator. `not run here` is a fact about the run. `FAILED` is a
defect. **A missing row reads as a win, and that is the single easiest way for this directory to become
dishonest** — more so than in `compare/`, because a capture that silently returns nothing looks like
agreement rather than absence.

**`cannot ask` is the fourth, and it is a fact about the *instrument*.** The emulator does the thing
and this suite cannot see it. kitty renders a dotted underline and writes it into a capture as
`CSI 4 : m` — its serialiser has a string for `4:2` and `4:3` and none for the other two — and
ECMA-48 reads an omitted parameter as the default, which for SGR 4 is *single*. A row compared anyway
would report *kitty does not render dotted underlines*, which is false, and would earn a `quirks.rs`
entry the table exists to keep out.

So it is not the same cell as `cannot express`, and collapsing the two would put a real capability
behind a word that means the terminal lacks one. Three rules keep an uncompared row from becoming an
excuse, and they hold for the fifth kind below as well as this one:

1. **An arm declares the rows it will not compare before the run, with the reason**, and declares them
   as a hand-written constant rather than by asking the engine. A limitation discovered after seeing
   the answer is an excuse, not a declaration; one read out of the code under test is not evidence.
2. **The row is still printed, with what was nonetheless observed.** It leaves the numerator and the
   denominator; it does not leave the table.
3. **A declared row that agrees anyway is `STALE` and counts as a failure.** An excuse nobody
   rechecks is the same kind of thing as an MSRV nobody compiles.

Production ticket 04 predicted this cell would be needed and predicted the wrong arm for it: it
expected Terminal.app's plain-text-only capture surface to be the first thing `compare/`'s vocabulary
could not describe. kitty got there first, and with a sharper case — Terminal.app cannot carry *any*
style, where kitty carries ten of the eleven and mis-spells the eleventh.

**WezTerm is the third arm to construct it and the first where it is the *only* verdict available.**
kitty's two bare rows became a `quirks.rs` entry because a second source existed outside the
capture; WezTerm's two have none, so `cannot ask` is not a softer reading of a disagreement here —
it is the strongest thing the evidence supports. That is the cell earning its keep rather than
excusing something: see §01.

**And Alacritty is the arm where the cell was *ruled out* rather than settled for**, which is the
other end of the same argument. Its scene 01 falls two rows short with the same shape as kitty's and
WezTerm's, and neither of those readings is available: the capture is the `Term`'s own grid, so there
is no serialiser to have lost anything, and two second sources outside it say what the grid says.
The three arms together are what make this cell a judgement about *evidence* rather than a house
style — same observation, three verdicts, and the difference is in what could be found outside the
capture each time.

**Terminal.app then arrived and constructed the cell it was predicted for, twelve rows of it — and
also, separately, the first `cannot express` this directory has ever had.** The two land on different
scenes and the pair is what shows they are not one word: scene 01's eleven rows and scene 04's one
reported style are `cannot ask`, because Terminal.app draws bold and `contents` is
`type="text" access="r"`; scene 06's five rows are `cannot express`, because Terminal.app has no
synchronised output at all. One arm, one run, both facts — and a suite with a single word for them
would have reported *this terminal does not do bold* and *we could not see the mode*, each of them
the other's answer.

**The eleven rows are declared from one sentence rather than eleven**, through `Arm::no_style`. Rule
1 above is unaffected — it is still a hand-written declaration in the arm, made before the run, and
rule 3 still applies to every row it covers. What changes is only that a fact about the arm is written
once. The rows still print what was observed, and for a styleless capture that observation is *the
label survived*, which is why a scrolled or mis-sized screen is as loud on this arm as on any other.

**`by design` is the fifth, and it is a fact about *the engine*.** The engine consulted `quirks.rs`
and deliberately did not send the attribute, so a `FAILED` would blame the terminal for a decision of
ours. It is not the same cell as `cannot express` and collapsing the two would hide this repository's
own code behind an emulator's limitation.

It arrived the session after production ticket 10 wired `attrs_dropped` on to the wire, and it
arrived as a surprise: the tmux arm had reported 11/11 while the engine still sent SGR 53, and
reported ten the first time anyone ran it afterwards. **The committed report had gone stale without
anything going red** — which is the mechanism this directory relies on for reporting-not-gating, one
level up from the thing it usually catches.

It costs the instrument the measurement that earned the entry, and that is deliberate rather than a
defect to fix. An arm that asked the engine what to expect would be checking the engine against
itself, which is the arrangement `conform/` exists to break. **The committed fixture is what preserves
the evidence**, captured while the engine still sent the bit, and it is the sharpest reason yet for
the rule that a capture is never regenerated to make something pass.

The third rule above is worth reading twice for this cell: a row the engine promised to withhold that
arrives anyway means the declaration and `quirks.rs` no longer describe the same terminal.

## 01 — the eleven attribute bits, one per row

Eleven rows, at the top left of **whatever size the window is**. `surface configuration` offers a
font size and no rows or columns, and `TIOCSWINSZ` changes what the *program* believes rather than
what the emulator renders, so the size is recorded in `REPORT.md` and never demanded. Two live runs
on the same machine were given 156×45 and 72×24.

Each row lights **exactly one** of the eleven attribute bits of the engine's style word — eight
flags and a three-bit underline field — writes a label with it, and leaves the rest of the row
alone. The three underline rows are single, double and dotted because `4:1`, `4:2` and `4:4` are the
three values with one bit set; curly (`4:3`) and dashed (`4:5`) light two bits each and so cannot be
a per-bit row. They are covered by the committed-fixture tests instead.

Assertion: eleven rows, and a row agrees only when all three of these hold.

1. The label survived, ignoring the padding the engine paints across the rest of the surface.
2. **Every** cluster of the label wears exactly that attribute — an invented attribute is a
   disagreement in the same way a missing one is.
3. The attribute **stopped where the label did**. A reverse block running to the right edge is a
   real defect, and comparing the first cell alone would call it a pass.

Chosen as the first scene because `attrs_dropped` is the field with **no query** — the eleven facts
are in the capability set precisely because nothing can ask for them — and a dump *is* a query for
them. No column arithmetic, no width tables, no timing. It emits `quirks.rs` rows directly, and it
has: **Ghostty 1.3.1 agrees on eleven, tmux 3.7c stores eleven and forwards ten, kitty 0.48.2 renders
nine and cannot be asked about a tenth** — the fourth and fifth entries of `quirks.rs`.

Those two entries are why the live arms no longer report eleven of anything but Ghostty: the engine
withholds what the table says a terminal will not render, so three of the four arms now print a `by
design` cell where they once printed a disagreement. The numbers that earned the entries are in
`FINDINGS.md` and the bytes are in `fixtures/`.

**Read across the arms, never down one.** Overline is present in Ghostty's dump and present in tmux's
grid and absent past tmux, and no single arm could have said which of the three parties lost it. That
is the scene's real assertion: eleven booleans *per path*, compared.

**The fifth family is where reading across stopped being *available*, and the verdict changed with
it.** WezTerm 20240203 renders nine and cannot be asked about two — overline and the dotted
underline — which is kitty's shape one row over, and it earns **no `quirks.rs` entry at all**.
`get-text --escapes` has no spelling for SGR 53 and none for a `4:n` sub-parameter: `CSI 4:1 m`
comes back as bare `4`, `CSI 4:2 m` as ECMA-48's `21`, and `4:3`, `4:4`, `4:5`, `53` and `58` come
back as nothing. A cell carrying curly-plus-bold comes back carrying the bold, so the serialiser
reached a styled cell and dropped one attribute of it — and *that is as far as this instrument
reaches*. There is no far side (WezTerm is an endpoint) and no second source: its shipped binary is
one blob whose string table holds `OVERLINE` as a Unicode character name, so the technique that
settled kitty returns noise. **The difference between a `cannot ask` and a quirk is entirely in what
evidence exists outside the capture**, and this arm is the case that says so.

**The sixth family is where the difference between the two verdicts stopped being an argument.**
Alacritty 0.17.0 renders nine and **stores** neither blink nor overline, and it earns `quirks.rs`
its seventh entry where WezTerm's identical-looking shortfall earned nothing. The reason is entirely
in the capture surface: `--ref-test` writes the `Term`'s grid out as JSON, so *not serialised* is not
one of the readings available — a bare cell is a cell with nothing in it. Two second sources say the
same thing from outside the capture anyway, and they say it about two different mechanisms:
`alacritty_terminal::term::cell::Flags` has no bit for either, and a run under `alacritty -vvv`
prints `Term got unhandled attr: BlinkSlow` for the one `vte` parses and **no `Setting attribute`
line at all** for the one it does not. Parsed and discarded, against never parsed.

**It is also the first arm that could be *asked* about a dotted underline**, and it answers. kitty
renders one and spells it `CSI 4 : m`; WezTerm's capture has no spelling for it either; this grid has
`DOTTED_UNDERLINE` as a bit of its own. The row that has been `cannot ask` on two arms is a plain
agreement here — which is the sharpest available demonstration that those two rows were about the
instrument, exactly as they were declared to be.

**The seventh family is where the two verdicts arrived on one arm, and it is the sharpest statement
of the difference this scene has.** iTerm2 3.6.11 renders ten of the eleven, and its three unanswered
rows split **one and two** across exactly the line the two kinds are drawn on. Its capture is the
Python API's `GetBufferRequest`, which carries a `CellStyle` per run of cells — the cell's fields,
with no serialiser in the path, but a **projection** of iTerm2's `screen_char_t` rather than the
struct. So *the projection dropped it* and *the cell never had it* look identical here, which is
WezTerm's position, and one row is settled each way.

Overline is the cell's. The type encoding the shipped binary compiles in enumerates fourteen bit
fields and eleven spare bits with no overline among them, and the string `overline` occurs **zero**
times in the whole binary — an absence in two places, outside the capture, which is what a
`quirks.rs` entry needs and what WezTerm had none of. The **eighth** entry.

The double and the dotted underline are the projection's. `CellStyle.underline` is a `bool` where the
cell spends three bits on `underlineStyle`, and iTerm2's own help text names `4:3  Curly underline`,
so the terminal renders both and the message says only *underlined*. **That completes the dotted
underline's set**: kitty renders it and spells it `CSI 4 : m`, WezTerm renders it and spells it
nothing, Alacritty stores it as a bit and answers, and iTerm2 renders it and projects three bits down
to one. Four families, four mechanisms, one row — and the three `cannot ask` verdicts are about three
different instruments, which is the whole of what the kind was invented to say.

**And the fifth entry is where reading across stopped being enough.** kitty's conceal and overline
come back bare, which is exactly what tmux's overline looked like from one arm — and there is no
further arm to add, because kitty is an endpoint and has no far side to read from. What settled it was
not another capture but kitty's shipped binary: its `Cursor` repr carries neither attribute, and SGR
is a mutation of the cursor. **A scene can say two things disagree; it cannot always say why**, and
the discipline of running a control probe before the instrument is what keeps the difference
visible.

## 02 — a wide glyph between ASCII sentinels

20×6. `AB漢CD`, and a coloured variant. **The scene exists to be unable to answer the obvious
question**, and it is kept because that is a finding: a grid-to-text dump emits no padding cell for a
double-width glyph, so *what is at column 3* is not askable of it. See `FINDINGS.md`.

Its successor is **scene 04**, below.

## 04 — a pair bisected, and what the terminal does with the orphan

Six rows, each `AB漢CD`: `A` at column 0, `B` at 1, the wide glyph across 2 and 3, `C` at 4, `D` at
5 — and then **one write over one half of it**. Scene 02's successor, and the scene production ticket
06 was blocked on.

**The row is compared as text, which is the whole trick.** Scene 02 established that *what is at
column 3* is not a question a grid-to-text dump can answer, and deriving it would need our own width
tables, which are the thing under test. ASCII sentinels turn it into *is the sentinel still there* —
a string comparison, with no width in the measuring loop.

| row | the write | expected |
|---|---|---|
| `control` | none | `AB漢CD` |
| `over-cont` | `x` at column 3, the continuation | `AB xCD` |
| `over-head` | `x` at column 2, the head | `ABx CD` |
| `over-wide` | `漢` at column 3 | `AB 漢D` |
| `keeps-style` | `x` at column 3, over a red-backed glyph | `AB xCD`, and the cell at 2 **reported** |
| `ruler` | none | `0123456789` |

### The eighth arm reads a blank the other seven cannot spell

iTerm2's capture distinguishes **a cell the terminal emptied** from **a space somebody wrote**: the
first is code zero and the second is U+0020, and the API reports both. Every other surface here
conflates them — an escape-stream re-serialisation writes a space for an empty cell, and a grid stores
one — so this scene's four overwrite rows come back as `AB\0xCD` where the rest of the table reads
`AB xCD`.

The reader projects the NUL down to a space, and the alternative is what justifies it: comparing the
raw row would report *iTerm2 does not blank the head of a bisected pair*, about the one terminal here
that blanks it hardest. The cell is not spaced; it is empty.

**What the projection costs is a loss and is asserted rather than described.** `tests.rs` checks that
the committed capture still holds the NUL and that the parsed row still holds the space, so a future
reader that stopped projecting and a future iTerm2 that stopped distinguishing are both a red test
rather than a quiet change of meaning.

### It is the only scene here that does not drive the engine, and it cannot

Every other scene asks the engine to draw something and compares what came back. This one must not.
The engine's drawing verbs repair a bisected pair **before anything is serialised** — that is
architecture ticket 20's answer — so an engine-driven scene could photograph only the repair. The
question is what a terminal does when it is handed the bytes anyway, which is what the engine's
mirror would have believed had the repair stopped at a `View::child` clip.

The top of this file licenses it in as many words: *a scene is a described picture, and an arm may
reach it any way it likes*. It is also the discipline the kitty arm's conceal row came out of — run
the control before the instrument, or *the terminal does not do it* and *the instrument cannot see
it* have no way to be told apart.

**And it is why this scene keeps its value after the answer, where `--through-tmux`'s overline row
lost its.** An arm that asked the engine what to expect would be checking the engine against itself.
These six rows ask nothing of the engine at all, so wiring the answer took no measurement away.

### What it found, on four arms, 2026-08-23 — and a fifth on 2026-08-30

The four text rows are **unanimous**: every family blanks the orphaned half, in both directions, and
none of them has a clip to consult. That settles architecture ticket 20. Terminal.app 2.15 agreed
with all four of them a week later, which is the first time this answer has been checked against a
terminal that is not a recent reimplementation.

`keeps-style` is not unanimous, and it is the sharper finding. The wide glyph carries a red background
and the cluster written over its continuation does not:

| | what the blanked half wears |
|---|---|
| kitty 0.48.2 | the orphan's own background |
| Ghostty 1.3.1 | nothing — the SGR state in force |
| tmux 3.7c | nothing — the SGR state in force |
| Terminal.app 2.15 | **not askable**, 2026-08-30 — its capture surface carries no style at all |
| WezTerm 20240203 | the orphan's own background, 2026-09-03 |
| Alacritty 0.17.0 | the orphan's own background, 2026-09-03 |

So a repair delegated to the terminal is a repair whose **result differs by terminal**, and no mirror
state could be right on all of them. *The engine may as well repair* becomes *the engine must*.

**The fifth arm made that a split rather than an outlier**, which is worth more than a fifth row.
With four arms the table read *kitty keeps it and everybody askable blanks it*, and one family
against two invites the reading that kitty has a bug. WezTerm 20240203 keeps it too, on a fifth
family and a different codebase — so these are **two designs**, both defensible, and the argument
for repairing in the engine no longer rests on which side is the majority. Its other five text rows
joined the unanimity on the same run.

**The sixth arm made it three against two, and the direction of travel is the finding rather than
the count.** Alacritty 0.17.0 keeps the orphan's background too, on a third codebase and through a
capture surface with no serialiser in it — so the behaviour is read straight out of the cell rather
than out of a re-spelling of one. Every askable arm added since the table had four has landed on the
*keeping* side, which is not a majority argument either: what it says is that *kitty is the odd one
out* was a reading available only while the sample was small, and the engine repairing the pair
itself is right whichever way the next arm falls. Its other five text rows joined the unanimity on
the same run, through the sixth kind of channel.

**The fourth row is a fact about the instrument and is in the table anyway**, which is the rule this
file's second section states: a row left out reads as a win. Terminal.app renders that background —
nothing here says it does not — and no property reachable over AppleScript can report it. Its other
four text rows joined the unanimity on 2026-08-30, so the answer architecture ticket 20 rests on now
has a **fourth VT lineage** behind it and the one row that could not be reached is the one that was
never a comparison.

**That row is reported and never compared**, and it is not a sixth kind of non-number. There is no
single right answer to hold an arm to, and inventing one would give two of three arms a permanent
`FAILED` for something that is not a defect. What gates it is `tests.rs` over the committed captures,
one assertion per terminal — the live arm reports and the fixture holds it still, which is the trade
this directory is built on. A row printed with its observation is still a row; a row left out is the
dishonesty this file opens by naming.

### The instrument's own defect, found by the arm with two parsers in the path

The probe first repainted with `CSI 2 J`. tmux **pushes a cleared screen into the pane's history**, so
ten repaints a second scrolled the picture up through Ghostty's scrollback and the `--through-tmux`
capture came back with the whole scene on it twice, at rows 38 and 76. Six rows of `FAILED` against a
screen that had the right answer on it — twice. `EL` per row touches no history in any of the four
arms and erases exactly the cells the scene is about to write.

It is the third time an arm's first defect has been in the arm rather than in the terminal, and the
third time the arm that found it was the one with the most parsers between the engine and the
capture.

## 03 — both SGR spellings, same colour

Any size. Emit a truecolor colour as `38:2::r:g:b` and again as `38;2;r;g;b`, and an indexed one as
`38:5:n` and `38;5;n`. Assertion: the resolved channels are equal and correct.

This is the scene that answers [arch 23](../.scratch/vitui-engine-architecture/issues/23-the-two-sgr-spellings-and-which-one-is-the-default.md)'s
third question for one terminal at a time. Both emulators tested normalise to semicolons on output,
which is what makes the comparison meaningful — agreement on the output is evidence about the parse.

## 05 — what does this emulator think this cluster is worth

Fifteen clusters, one row, one column, and no picture at all. Written at column 1 with `CSI 6n`
behind each one and `CSI c` behind the batch; the column that comes back is the advance plus one.

**This is the only scene here whose answer does not come back through a photograph**, and that is
the property worth stating first. Scene 02 exists to be unable to answer *what is at column 3*: a
grid-to-text dump emits no padding cell for a double-width glyph, so recovering a column from it
would need a width table, and a width table is the thing under test. A cursor report has no such
problem — the number is the **emulator's own UAX #11 verdict**, reached by the emulator's tables and
reported by the emulator, with nothing of this repository's in the path.

The consequence is larger than the scene: **the capture surface is out of the path entirely**, so
this is the one scene an arm can answer having done nothing but launch the scene — and the one an arm
whose capture surface carries **no style at all** could still answer in full.

What that does *not* say is that an arm needs nothing else. Whatever an arm required in order to open
a terminal it still requires: the Ghostty arm opens, addresses and closes its window over AppleScript,
so its automation grant is in the path for every scene including this one. The claim is about the
capture and about nothing else, and the difference decides which arms this scene unblocks.

### Three rows are compared and twelve are surveyed, and that split is a decision

`ucd.rs` says in as many words that the engine's tables are **authoritative**: it pins three answers
as policy rather than standard — ambiguous width is narrow, a cluster's width is its base's and
never the sum of its code points, VS15 changes a presentation and not a width — and spec §8's
`CHA`-after-non-ASCII rule is what **bounds** a disagreement instead of following it.

So a terminal that answers differently is not misbehaving in any sense this repository acts on.
There is no mechanism that would read such a `quirks.rs` row, and an entry nothing reads is this
backlog's own recurring defect. **A `FAILED` in the survey would be the instrument inventing one.**

What *is* still a defect is the instrument not working, and three rows are held to that: `ascii`,
`ascii-pair` and `cjk`. Their expectations are **hand-written in the scene** and deliberately not
asked of `width_of` — a row that took its number from the engine would be checking the engine
against itself. `ascii-pair` is there because a probe that answered a constant would pass `ascii`.

### The refusals, and they are not the dump's

A capture that raced the paint is a *short screen*. A cursor report that never came is **no reply at
all**, and an instrument that read a missing reply as a width would report a number no terminal ever
said — `screen -X hardcopy`'s zero bytes wearing this scene's clothes. `cursor_reports` therefore
refuses four ways, and they say four different things:

- **no sentinel** — nothing says the terminal finished with the batch. This is what a read that gave
  up early looks like.
- **a count that is not the scene's** — including *the sentinel arrived and no cursor report did*,
  which is a terminal that does not implement DSR and is a different fact from the one above.
- **a reply after the sentinel is not counted**, or a batch that lost an answer could be made up to
  length by a stranger's.
- **a row that moved** — the scene writes every cluster on one row, so two rows means the screen
  scrolled or a cluster wrapped, and no column in that batch is a width.

### What it found, on four arms, 2026-08-29

**Fifteen for fifteen on all four, and the twelve surveyed rows agree with the engine's tables
everywhere.** The corpus is not a soft one: it carries the VS16 and VS15 pair, a ZWJ family, a
regional-indicator flag, a skin-tone modifier, a keycap sequence, a combining acute, a zero-advance
cluster and UAX #11's ambiguous class.

Two of the rows were chosen because `ucd.rs` names them as disagreements, and **neither reproduced
on those four arms**:

| `ucd.rs` says | measured 2026-08-29 | and then 2026-08-30 |
|---|---|---|
| *only 7 of 23 surveyed widen a VS16 emoji correctly* | Ghostty 1.3.1, kitty 0.48.2 and tmux 3.7c all widen it | **Terminal.app 2.15 does not.** The citation reproduces on the fourth family — and on the **fifth**, WezTerm 20240203, 2026-09-03 |
| *kitty sums a ZWJ family emoji to 6 where the answer is 2* | kitty 0.48.2 answers **2** | Terminal.app sums the same cluster to **8**, counting each joiner as a column. The behaviour `ucd.rs` describes, on a terminal it does not name. WezTerm answers **2** |

That paragraph cites a **survey in a research document**, and this is the first thing in this
repository to observe any of it. The kitty row was checked a second time with a raw `printf` control
probe with no vitui code anywhere in its path, because *run the control before the instrument* is
what the kitty arm's conceal row came out of — and because an instrument reporting that a recorded
disagreement has gone away is the one result most worth doubting. The control agreed on all seven
clusters it was given.

**Nothing here changes the decision.** The engine's tables stay authoritative and §8's rule stays
the bound; what moved is that the cited evidence for the disagreement is now known to be stale on
the three families §10 puts in tier 1, at the versions on this machine. The right reading is
`FINDINGS.md`'s: **the survey needs an arm that disagrees**, and the two candidates are a terminal
of a different VT lineage and a locale this suite does not set.

### The fifth arm, 2026-08-30, and the survey stopped being four identical columns

**Terminal.app 2.15 disagrees on four of the twelve surveyed rows**, which is what the paragraph
above asked for and is the reason that arm was built. The count is the least interesting part; the
shape is the finding.

| row | the engine | Terminal.app 2.15 |
|---|---|---|
| `zero-width` (U+200B) | 0 | **1** |
| `vs16` (U+2764 U+FE0F) | 2 | **1** |
| `zwj-family` (three emoji, two ZWJs) | 2 | **8** |
| `skin-tone` (U+1F44D U+1F3FD) | 2 | **4** |

Every one is an emoji-era question, and every one falls the same way: **Terminal.app sums the
cluster's code points where the other three take the base's width.** 8 is 2+1+2+1+2 with the joiners
counted as columns of their own; 4 is 2+2; a zero-width space is a cluster it does not know is
zero-width; and a VS16 pair is a text-presentation heart plus a selector it does not act on. That is
`ucd.rs`'s second pinned policy — *a cluster's width is its base's width, never the sum of its code
points* — named as a policy precisely because terminals exist that do the other thing. This is the
first one this repository has measured.

**It changes no decision and it changes what the survey is worth.** Three columns that agree cannot
distinguish *the terminals agree with our tables* from *the instrument is reading our tables back to
us*; a fourth that disagrees on four rows, through the same code, on the same day, can. The rows stay
surveyed rather than compared, for the reason the split was made: there is no mechanism in this
repository that would read such a `quirks.rs` row, and §8's `CHA`-after-non-ASCII rule is what bounds
the disagreement instead of following it.

### The sixth arm, 2026-09-03, and the two disagreeing columns disagree with each other

**WezTerm 20240203 disagrees on two of the twelve**, and the pair is not a subset of Terminal.app's
four — which is the reading that matters, because two arms failing the same four rows the same way
would be one observation printed twice.

| row | the engine | WezTerm 20240203 | Terminal.app 2.15 |
|---|---|---|---|
| `vs16` (U+2764 U+FE0F) | 2 | **1** | **1** |
| `keycap` (U+0031 U+FE0F U+20E3) | 2 | **1** | 2 |
| `zero-width` (U+200B) | 0 | 0 | **1** |
| `zwj-family` | 2 | 2 | **8** |
| `skin-tone` | 2 | 2 | **4** |

**Two mechanisms, not two instances of one.** Terminal.app sums a cluster's code points; WezTerm
takes the base's width everywhere Terminal.app sums — a ZWJ family at 2, a skin tone at 2, a
zero-width space at 0 — and then **does not let a variation selector widen the base**. VS16 and the
keycap sequence are exactly the two rows of the corpus where a selector is what asks for the second
column, which is why they are the two that differ and nothing else does.

So `ucd.rs`'s headline citation now reproduces on **two of five** arms, which is a population rather
than an outlier and is the sentence the Terminal.app run could not make on its own evidence. And the
keycap row is the first in this survey where the two disagreeing arms disagree with **each other** —
Terminal.app widens it and WezTerm does not. **The decision is untouched** for the reason above:
there is no mechanism here that would read such a `quirks.rs` row.

### The seventh arm, 2026-09-03, and it answers the number the citation quotes

**Alacritty 0.17.0 disagrees on four rows and one of them is `ucd.rs`'s own headline figure.** It
answers **6** for the ZWJ family — which that file attributes to kitty, and which the kitty measured
here answers 2 for. The survey it quotes is about a population, and this is the first capture in this
directory to land on a number the citation names.

**Three of six arms now disagree, and the three disagree by three mechanisms.** That is what the
column was built to find out and it took a third one to see it, because the fourth arm had two
decisions joined in a single behaviour:

| | zero-width space | VS16 pair | ZWJ family | skin tone | keycap |
|---|---|---|---|---|---|
| the engine | 0 | 2 | 2 | 2 | 2 |
| Terminal.app 2.15 | **1** | **1** | **8** | **4** | 2 |
| WezTerm 20240203 | 0 | **1** | 2 | 2 | **1** |
| Alacritty 0.17.0 | 0 | **1** | **6** | **4** | **1** |

Terminal.app **sums** the cluster's code points and costs a zero-width one a column; Alacritty sums
them and costs a zero-width one nothing — so its family is `2+0+2+0+2` where Terminal.app's is the
same sum with the joiners counted. WezTerm does not sum at all: it takes the base's width and then
lets no *variation selector* widen it. **Summing, and what a zero-width scalar is worth, are two
decisions**, and with two disagreeing arms they looked like one.

The three columns agree on exactly one thing besides the seven rows nobody disputes: **none of them
widens a VS16 emoji.** Eight of Alacritty's twelve surveyed rows agree with the engine, which is the
lowest figure this scene has produced and is still not a score.

### The eighth arm, 2026-09-04, and it is the one that breaks the sentence above

**iTerm2 3.6.11 widens a VS16 emoji**, which is what the paragraph directly above had just said no
disagreeing arm does — and it disagrees anyway, on the row beside it. Ten of its twelve surveyed rows
agree; the two that do not are the zero-width space at **1** and the keycap at **1**.

| | zero-width space | VS16 pair | ZWJ family | skin tone | keycap |
|---|---|---|---|---|---|
| the engine | 0 | 2 | 2 | 2 | 2 |
| Terminal.app 2.15 | **1** | **1** | **8** | **4** | 2 |
| WezTerm 20240203 | 0 | **1** | 2 | 2 | **1** |
| Alacritty 0.17.0 | 0 | **1** | **6** | **4** | **1** |
| iTerm2 3.6.11 | **1** | 2 | 2 | 2 | **1** |

**Two readings this file had written down get narrower.**

*Summing and what a zero-width scalar is worth are two decisions* is right, and `zero-width` is not
the row that tells the summers apart — it is not a symptom of summing at all. iTerm2 costs a
zero-width space a column and sums nothing: its family is 2 and its skin tone is 2, both correct. Two
disagreeing arms made that row look like a summer's signature; a third that is not a summer takes the
signature away.

*None of them widens a VS16 emoji* survived three arms and does not survive a fourth. What iTerm2
supplies is the fourth combination of `vs16` and `keycap` — 2 and 1, where Terminal.app is 1 and 2,
WezTerm and Alacritty are 1 and 1, and the agreeing families are 2 and 2. **All four combinations of
two booleans, observed** — the table above is the four, and iTerm2 is the one that completed them. **The two rows are decided by different code in every family that has been
asked**, so neither is a proxy for the other: an ASCII base does not reach iTerm2's VS16 rule and a
default-text emoji base does.

That is what a survey is for, and it is why a `FAILED` here would be the instrument inventing a
defect. Every one of these terminals is entitled to its answer; what the column buys is knowing that
the question has at least three independent parts.

## 06 — mode 2026, asked of the terminal rather than of its documentation

No picture, no capture surface, and the second scene here whose answer comes back in band. It asks
`CSI ? 2026 $ p` on the scene's own tty and reads the DECRQM reply, which means — like scene 05 —
the answer is the **innermost** terminal's and a capture surface further out is not in the path.

### Why it is a scene at all, and what `quirks.rs` says that it does not touch

`quirks.rs` carries a four-row table of the terminals' force-flush limits for mode 2026, and every
row of it has the same provenance: **the implementation, read.** Production ticket 05 asked for
Ghostty's row to be *measured* and it could not be. A force flush is a **rendering** event; nothing
a process inside a terminal can ask reports whether the terminal painted, only a screen capture can,
and this repository's capture is an AppleScript round trip four runs put between 136 ms and 623 ms —
the same order as Alacritty's entire 150 ms limit.

That sentence is still true and **it is not the whole of what mode 2026 is.** Three questions about
it are the terminal's to answer about itself:

- does it **recognise** the mode;
- does its state machine track the `h` and the `l` it was sent;
- and when does it stop reporting the mode as **set**.

None of those is the paint, and the third is the one to be careful about — see below.

### The five compared rows, and why they are compared where scene 05's twelve are surveyed

One batch — ask, set, ask, reset, ask, set, set, ask, reset, ask — with `CSI c` behind it, and the
five answers read positionally.

| row | expected | asks |
|---|---|---|
| `before` | `reset (2)` | whether the terminal recognises the mode, and what it says untouched |
| `while-open` | `set (1)` | whether `CSI ? 2026 h` reached the state machine, or was parsed and thrown away |
| `after-close` | `reset (2)` | whether `CSI ? 2026 l` reached it too |
| `opened-twice` | `set (1)` | whether a second `h` over a set mode is still simply set |
| `closed-once` | `reset (2)` | whether one `l` undoes two `h` — a DEC private mode is not a counter |

**These are DECRPM's own values.** A terminal that reports the mode set after it was asked to reset
it is wrong by the definition of the reply it sent, not by a table this repository chose — which is
exactly the thing scene 05's survey does not have, since `ucd.rs` makes the engine's width tables
authoritative and leaves a terminal nothing to be wrong *against*.

The last row is the one the engine depends on. §8 wraps every frame in a **balanced** pair, so a
terminal that counted the sets would hold a frame back past the close that was sent for it — and
nothing in this repository opens a block twice, which is exactly why nothing here would notice.

A terminal with no synchronised output answers `not recognised (0)` to all five, which is a
legitimate answer and not a defect. Such an arm owes a declaration in advance, with the reason, and
the cell for it is `compare/`'s own `cannot express` — **which `Excluded` has no variant for, because
no arm here needs one.** All four recognise the mode, and a variant nothing constructs is a spare
part this workspace refuses by name. So the path is written down and unexercised: the rows are loud
until an arm that needs the declaration adds it, which is the right way round and is also what would
make it real. `screen` 4.00.03 is the candidate on this machine and has no usable capture surface for
the other three scenes.

### The flag is not the paint, and this scene may never be read as though it were

Part B opens a block, waits, and asks. What it finds is when the terminal stopped reporting the mode
as **set** — the event Ghostty's own source calls *reset the synchronized output flag*. A terminal
could paint without clearing the flag or clear it without painting, and nothing in this suite can
tell those apart. It is a timing besides, so it is **reported and never compared**, and no row of it
moves any numerator here.

### One probe per open, which a control probe had to teach

The obvious instrument opens one block and polls it, which costs one open where this costs seven. It
was written first and it is **wrong on one of the three families**: polling a Ghostty 1.3.1 every
250 ms put the reset before 517 ms, while one probe per open puts it between 879 and 973 — and the
same polling left tmux 3.7c and kitty 0.48.2 on their documented figures. *An instrument that polls
is inside its own measurement*, and the two families it happens not to disturb are exactly what
would have made that invisible. It is the discipline scene 04 and the kitty arm's conceal row came
out of, arriving a third time: **run the control before the instrument.**

So each probe is a fresh open, a wait, one question and a close, and the boundary is bisected for —
seven opens, which is what the arm's readiness timeout is derived from.

### Two clocks, because a bracket built from one of them is unsound in one direction

The terminal processes the question somewhere between the write and the reply, and which end of that
interval is safe depends on the answer. *Still set* at some instant implies still set at every
earlier one, so the **request** is the sound end — a sleep is a floor. *Already reset* implies reset
at every later one, so the **reply** is. Both columns are printed and the bracket takes one end from
each; either clock read for both would report an interval the run does not support.

### What it found, on four arms, 2026-08-29

**Five for five on all four**, and the three families' brackets against the figures `quirks.rs`
reads out of their source trees:

| | documented | observed here |
|---|---|---|
| Ghostty 1.3.1 | 1000 ms | still set at **879 ms**, reset by **973 ms** |
| tmux 3.7c | 1000 ms | still set at **971 ms**, reset by **1064 ms** |
| kitty 0.48.2 | 2000 ms | still set at **1985 ms**, reset by **2085 ms** |
| Ghostty via tmux 3.7c | tmux's | still set at **971 ms**, reset by **1063 ms** — tmux's, as the innermost terminal |

tmux and kitty land on theirs. **Ghostty's bracket sits below its own `sync_reset_ms = 1000`**, and
this scene cannot say why: the instant the terminal armed the timer is not observable from inside,
so a write that reached Ghostty before the scene took its own `Instant::now()` would move the whole
bracket earlier by that much. What is reported is the interval that was observed.

### The fifth arm, 2026-08-30, and the first terminal here that has no answer

**Terminal.app 2.15 answers nothing at all**, and the five rows are this directory's first
`cannot express`. Not `not recognised (0)`, which is DEC's own way of declining a mode it does not
have — nothing: its parser does not take `$` as an intermediate byte, so `CSI ? 2026 $ p` is not a
query it declines but a sequence it never finishes reading, and the `p` lands on the screen as text.

**The sentinel is the whole reason that is an observation rather than a timeout.** A
device-attributes reply cannot be sent before everything ahead of it has been processed, so silence
in front of one is silence the terminal chose. The committed fixture is seven bytes and all seven of
them are that reply.

`vitui_conform::ModeError::Unanswered` is the refusal that says so, and it is **not**
`ModeError::Count`. A short batch is *the terminal lost an answer*, which is a defect in the run; this
is *the terminal has no such report*, which is a fact about the terminal. The line between them is
whether the terminal spoke at all before the sentinel — never whether it spoke in this scene's
grammar — because a capture from another channel has a sentinel and no mode reports too, and
collapsing the two would let a `.cpr` file pass as a terminal without synchronised output.

**Three things this does and does not mean.** `detect.rs` reaches the same conclusion from the same
silence and reports `sync_output false`, so the engine wraps no frame in a mode this terminal does
not have — the engine is right here and this scene is the first outside evidence of it. The
`closed-once` row is the one §8 depends on, and its absence is not a risk: a terminal that counted
the sets would hold a frame past the close sent for it, and a terminal with no mode at all has
nothing to hold. And there is no `quirks.rs` row in this: that table's four entries are force-flush
*limits*, and a terminal without the mode has no limit to record.

### The sixth arm, 2026-09-03, and the first terminal here that answers wrongly

**WezTerm 20240203 answers all five, and two of them are `FAILED`.** It reports mode 2026 as
**reset while the mode is set** — `while-open` and `opened-twice`, the two rows asked immediately
after a `CSI ? 2026 h`. Not `cannot express`: it answers. Not a short batch: five replies and a
sentinel. A genuine disagreement with DECRPM's own definition of the reply it sent, and the first one
this scene has produced on any arm.

**The capture alone could not have said so, and three control probes are what make it a finding.**
Raw `printf` against WezTerm, no engine in any of them:

- `CSI ? 9999 $ p` answers **`0`**, so this terminal does say *not recognised* when it means it —
  the `2` it gives for 2026 is a real *reset* and not a catch-all.
- `CSI ? 2004 $ p` answers `2`, then **`1`** after an `h`, then `2` after an `l`. Its DECRQM tracks
  a mode it implements.
- **It has synchronised output and holds every reply while the mode is set.** A DECRQM about an
  unrelated mode, asked inside an open block, returned nothing for **eight seconds** and arrived the
  instant the block closed. That alone would explain a `2` — a reply computed when the buffer drains
  is a reply about the state at the flush — so the third probe closes it: a mode set *and reset
  again inside the block* came back **`1`**. A reply is computed when the query is **parsed**, so
  the `2` above was given while the mode was set.

**It earns no `quirks.rs` entry, and that is the table's own rule rather than an omission.** Every
row there is a route the serializer can take around a defect. `Detected::mode` reads `1` and `2`
alike as *available* — the question is whether the capability exists, not whether it happens to be
on — so `sync_output` is true for WezTerm, `serial.rs` wraps every frame, and the terminal really
does synchronise. Nothing is degraded and there is nothing to route around. It is the second thing
recorded as *deliberately not an entry*, after Terminal.app's printing of what it cannot parse.

**Part B is unanswerable on this arm, and its rows are not evidence about WezTerm.** The bisection
asks DECRQM *inside* the open block, and a terminal that holds its replies until the close cannot
answer inside one — so every probe is a hole, `flush_bracket` refuses, and the report prints *no
bracket*. Worse, a round's opening batch closes the previous block and flushes what it was holding,
so a probe can read a **leftover** and report it as an answer: the run this arm was built on printed
`reset (2)` at 3000 ms, and that number is a previous round's reply arriving late. The refusal is
doing its job; the rows above it are a log and must not be read as a measurement. Nothing here is
gated — a timing is a report — and WezTerm gets **no force-flush row** in `quirks.rs`, which is
consistent with what was observed: eight seconds with no flush at all.

### The seventh arm, 2026-09-03, and the second terminal to answer wrongly — with the cause readable

**Alacritty 0.17.0 answers all five and two of them are `FAILED`**, which is WezTerm's result on an
unrelated codebase. What is new is not the disagreement but that its cause can be read:
`Term::report_private_mode` answers `NamedPrivateMode::SyncUpdate` with a **constant**
`ModeState::Reset`, while synchronised output is implemented one crate down in `vte`'s parser, which
the `Term` never sees. **The flag and the reporter are in different layers**, and a second family
arriving at the same wrong answer through a mechanism this legible is worth more than either arm
alone: it says the shape is available to anyone who implements the buffering below the state machine.

The same three control probes close the same innocent readings — `CSI ? 9999 $ p` answers `0`, so
its `2` is a real reset; `CSI ? 2004 $ p` tracks correctly through an `h` and an `l`; and it really
does hold a block. It earns **no `quirks.rs` entry**, for the reason WezTerm's earns none.

**Part B measures the reply here, and that is a third fact this scene had no word for.** A bisection
over *when did the terminal stop saying set* has nothing to bisect when the terminal never says set,
so `flush_bracket` returns `AlreadyReset` — whose two documented causes, *the limit is under the
floor* and *the open never took*, are both **false** on this arm. What the figure reports instead is
when the **reply** arrived: a `CSI ? 2026 $ p` written 50 ms into an open block came back at 150,
151 and 171 ms over three runs, so the question sat in the parser's buffer for about a tenth of a
second and came out when the block force-flushed. `vte`'s shipped `SYNC_UPDATE_TIMEOUT` is 150 ms.

That makes it the **first measurement beside a row of `quirks.rs`'s force-flush table**, whose four
rows were all *the implementation, read* — and it is still not the paint, and the arming instant is
still unobservable from inside, for the reason Ghostty's bracket sits below its own figure. The
variant's documentation and the report's sentence both name the third cause now; the arm is what
found it, and the first two were what they said until it ran.

### The eighth arm, 2026-09-04, and the majority side gains a member

**iTerm2 3.6.11 answers 5/5** — reset, set, reset, set, reset — so the two families that answer
wrongly stay two, and the arms that track DECRPM's own definitions become five. That is worth one
line rather than a section, and the section is here because the *silence* it breaks is a pattern:
this scene's last three new arms were a `cannot express`, a wrong answer and a second wrong answer,
and a reader arriving at that run of three would be entitled to wonder whether the scene had stopped
being able to pass.

**Its flag was still set at 3000 ms**, the largest delay this run opened a block for, so iTerm2's
force-flush limit is beyond that or it has none and this run cannot say which. That is a row of the
`quirks.rs` force-flush table with **no** number rather than a wrong one — the table's iTerm2 row is
unpopulated and stays so, because *beyond 3000* is not a limit.

### The refusals, and there are five

A missing reply read as a state would be worse here than in scene 05, because the batch is counted
**positionally**: a lost answer does not blank a row, it reports every later row under the wrong
question. So `mode_reports` refuses on no sentinel, on a count that is not the scene's, on a reply
about a mode nobody asked about, on a state DECRPM does not define, on a private-mode reply whose
two parameters are not two numbers, and on a truncated sequence. A reply after the sentinel is not
counted, for the reason it is not counted in scene 05.

The fifth of those is the one that had to be written rather than left to fall through: a reply this
reader cannot parse, **dropped**, arrives downstream as a short batch — *the terminal lost an answer*
about a terminal that answered every question and spelled one of them in a way this reader does not
know. Two causes, one of them about the terminal and one about the instrument, and a single count
cannot tell them apart.

Part B refuses separately and on its own terms: a probe with no answer is a **hole** and not a
state, a probe that came back *not recognised* or pinned permanently is an answer about the terminal
and not a timing, and a run in which the mode was set at a longer delay than one at which it was
already reset has measured something that is not a timeout. Each of those is printed as what it is
rather than left out.

**Part B has no fixture, and that is the rule rather than an omission.** It is a timing, a timing is
a report, and a report is not gated. Part A is a comparison and its bytes are in `fixtures/`.
