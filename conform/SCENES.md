# Scenes

**Normative.** A scene is a described picture — a size, an initial screen, and a sequence of changes —
and an arm may reach it any way it likes. Nothing here says *verb*, *layer* or *damage*, because the
comparison spans this engine and the emulators it talks to, and those words would decide it in advance.

This list is deliberately **not** `crate::scenes`' twelve. Those are fixed at 300×80, are tuned for the
byte budget and the damage model, and **not one of them contains a CJK glyph, an underline colour or
mode 2026** — the four things this directory exists to ask about. `compare/` reached the same
conclusion and wrote its own list for the same reason.

## What each arm actually measures

| arm | measures | |
|---|---|---|
| **tmux** | **what tmux stores** | `capture-pane` re-serialises *tmux's* grid, so the emulator behind it never sees the engine's bytes. A legitimate target — tmux is in §10's tier-1 list — and **never a proxy for one.** Headless, and the only arm that can *set* the pane size, which makes it the CI-shaped one |
| **Ghostty** | Ghostty | its own screen dump, over an AppleScript surface. Needs a window server and a macOS automation grant |
| **Ghostty-via-tmux** | **what tmux forwards** | the engine into tmux into Ghostty, photographing Ghostty. Ghostty alone agrees 11/11, so a disagreement here is tmux's. **The only arm that can see this**, because `capture-pane` and tmux's redraw path are different code and `attrs_dropped` is about what is rendered |
| **kitty** | kitty | `kitten @ get-text --ansi` over a unix socket. No automation grant, no clipboard, no z-order — and **the only *emulator* arm that can be handed a size**, in cells, which Ghostty's AppleScript surface cannot do |
| **Terminal.app** | Terminal.app | plain text only, so glyph-grid scenes and nothing else. Not built — **and scene 05 changes what that sentence excludes**, because a cursor report needs no capture surface at all |

**Scene 05 is answered by a different party than the rows above it**, and only one arm is affected.
Its answers come back **in band**, on the scene's own tty, so they come from the **innermost**
terminal in the path and a capture surface further out cannot change that. For every arm but one
that is the same terminal the photograph measures. For **Ghostty-via-tmux** it is not: the
photograph sees what tmux *forwards*, and the cursor reports never leave tmux — the two fixtures are
**byte-identical** to the plain tmux arm's, device attributes included, which `tests.rs` asserts.
Each report heads that scene's columns with **who answered** rather than with the arm's title.

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

### What it found, on four arms, 2026-08-23

The four text rows are **unanimous**: every family blanks the orphaned half, in both directions, and
none of them has a clip to consult. That settles architecture ticket 20.

`keeps-style` is not unanimous, and it is the sharper finding. The wide glyph carries a red background
and the cluster written over its continuation does not:

| | what the blanked half wears |
|---|---|
| kitty 0.48.2 | the orphan's own background |
| Ghostty 1.3.1 | nothing — the SGR state in force |
| tmux 3.7c | nothing — the SGR state in force |

So a repair delegated to the terminal is a repair whose **result differs by terminal**, and no mirror
state could be right on all three. *The engine may as well repair* becomes *the engine must*.

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

Two of the rows were chosen because `ucd.rs` names them as disagreements, and **neither
reproduces**:

| `ucd.rs` says | measured here |
|---|---|
| *only 7 of 23 surveyed widen a VS16 emoji correctly* | Ghostty 1.3.1, kitty 0.48.2 and tmux 3.7c all widen it |
| *kitty sums a ZWJ family emoji to 6 where the answer is 2* | kitty 0.48.2 answers **2** |

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
