# Findings

Hand-written and dated, because a number and what it means are two different artefacts with two
different lifetimes. `REPORT.md` is generated; this is not.

## 2026-08-30 — stage 4's second VT lineage, and the survey stopped being four identical columns

The Terminal.app arm — `conform/examples/terminal.rs`, the fifth arm and the fourth emulator family.
The last thing production ticket 04 owed, and the arm this directory has been asking for by name
since 2026-08-29: *what the survey needs is an arm that **disagrees**.*

It disagrees. Four of the twelve surveyed rows of scene 05, and the shape of them is the finding
rather than the count.

### The four rows, and they are one behaviour

| row | the engine | Terminal.app 2.15 |
|---|---|---|
| `zero-width` (U+200B) | 0 | **1** |
| `vs16` (U+2764 U+FE0F) | 2 | **1** |
| `zwj-family` (three emoji, two ZWJs) | 2 | **8** |
| `skin-tone` (U+1F44D U+1F3FD) | 2 | **4** |

Every one is an emoji-era question and every one falls the same way: **Terminal.app sums the
cluster's code points where the other three take the base's width.** 8 is 2+1+2+1+2 with the joiners
counted as columns of their own; 4 is 2+2; a zero-width space is a cluster it does not know is
zero-width; a VS16 pair is a text-presentation heart plus a selector it does not act on. That is
`ucd.rs`'s second pinned policy — *a cluster's width is its base's width, never the sum of its code
points* — and this is the first terminal this repository has measured that does the other thing.

**The headline citation reproduces, on the fourth family.** `ucd.rs` supports *our tables are
authoritative* with a survey of 23 terminals in a research document, whose headline is *only 7 of 23
widen a VS16 emoji correctly*. The 2026-08-29 session measured three families, found all three
widened it, and recorded the citation as not reproducing — on the evidence it had, correctly. What
was actually true is that this suite had been sampling one end of the population: three recent
reimplementations, all of which implement UAX #29 clustering. The terminal Apple has shipped since
NeXT does not, and it answers 1.

So the correction runs the other way from 2026-08-29's. That session's finding was *a citation
nobody re-measured*; this one is **a re-measurement that had not sampled widely enough to see what
the citation was about** — and the earlier entry stays on this page unamended, because it was right
about its own four columns and the record of how a conclusion narrowed is worth more than a tidy
page. `ucd.rs` now carries both dates.

**Nothing changes the decision, and the value is not in the four rows.** The rows stay *surveyed*
rather than compared, for the reason the split was made: there is no mechanism in this repository
that would read such a `quirks.rs` row, and §8's `CHA`-after-non-ASCII rule bounds the disagreement
instead of following it. What changes is what the survey is worth. Three columns that agree cannot
distinguish *the terminals agree with our tables* from *the instrument is reading our tables back to
us*. A fourth that disagrees on four rows, through the same code, on the same day, can — and that is
the entire reason this arm was built.

### The first `cannot express` this directory has ever printed

`SCENES.md` inherited `compare/`'s three kinds of non-number on the day it was written, and in four
arms **no arm ever constructed the first of them**: the three emulator families that came first all
had every capability the scenes ask about. Terminal.app 2.15 has no synchronised output, so scene
06's five rows are the cell the vocabulary was carrying for something.

**It does not answer `not recognised (0)`, which is DEC's own way of declining. It answers nothing.**
Its parser does not take `$` as an intermediate byte, so `CSI ? 2026 $ p` is not a query it declines
but a sequence it never finishes reading — and the `p` lands on the screen as text. The committed
fixture is seven bytes and all seven of them are the device-attributes reply.

That silence had to become a first-class observation, and the shape of it is the finding worth
reading. `mode_reports` refused it as `ModeError::Count { expected: 5, found: 0 }` — *the terminal
lost five answers*, about a terminal that never had one to lose — and a run that reported that would
have failed the arm rather than recording the fact. So there is a `ModeError::Unanswered`, and
**the line between the two is whether the terminal spoke at all, never whether it spoke in this
scene's grammar.** A capture from another channel has a sentinel and no mode reports too: scene 05's
`.cpr` is fifteen cursor reports and a `CSI c`, and read by the looser rule it would have passed as
a terminal without synchronised output. `tests.rs` gates both sides.

**The sentinel is what makes silence an observation rather than a timeout.** A device-attributes
reply cannot be sent before everything ahead of it has been processed. Without it there would be no
way to tell a terminal that declined the question from a read that gave up, and the honest report
would have been *not run here*.

Three things it does and does not mean. `detect.rs` reaches the same conclusion from the same
silence and reports `sync_output false`, so the engine wraps no frame in a mode this terminal does
not have — **this scene is the first outside evidence that the engine is right about that**, where
before it was `detect.rs` believing a reply whose parser it also wrote. The `closed-once` row is the
one §8 depends on and its absence is not a risk: a terminal that counted the sets would hold a frame
past the close sent for it, and a terminal with no mode has nothing to hold. And there is no
`quirks.rs` row in it — that table's four entries are force-flush *limits*, and a terminal without
the mode has no limit to record.

### The engine's own capability probe leaves visible text on this terminal's screen

**The finding that is not about a scene, and it is the most valuable thing the arm turned up.** It
came out of a control probe run before the instrument, which is the fourth time in this directory
that discipline is what separated the terminal from the suite.

Run any vitui application in Terminal.app 2.15 and this is on the screen afterwards:

```text
BEFORE-ATTACH-MARKER
+q524742pppppppvitui capabilities (tty)
```

`detect::batch` sends `DCS + q 524742 ST` for XTGETTCAP `RGB` and seven `CSI ? <mode> $ p` for the
DECRQM block. Terminal.app implements neither, and it does not *ignore* them: it emits the DCS
payload as text and the final `p` of each DECRQM as text. Eight sequences, eight visible artefacts,
and the count is exact — seven `p`s for seven modes.

**It lands on the primary screen and it survives the session.** `detect` runs in `attach` at
`engine.rs:477`, and `begin_session` — whose first bytes are `?1049h` — runs after it. So the bytes
are written to the page the user's shell is on, before the alt screen is entered; `?1049l` on the way
out restores that page with the artefact on it, on the line the prompt was on. Nothing the engine
does afterwards can reach it.

Two reasons this is worth more than its size. It is **exactly** what production ticket 04 exists to
find: every gate in this workspace is the engine's bytes replayed through the engine's own model of a
terminal, and a model that ignores an unknown sequence — which is the correct thing for a model to do
— cannot represent a terminal that prints it. And it is the *first* defect this directory has found
in the engine's **output** rather than in a quirk table, an instrument or a document.

It is not fixed here, and deliberately. The obvious repair is to send the batch after `?1049h`
instead of before, and that is an attach-ordering decision with its own consequences — `negotiation`
is built from `caps`, so the two cannot simply swap. **Production ticket 12** is where it goes.

#### Fixed 2026-08-30, and the scene that would have watched it is **declined** with the reason

Production ticket 12 took the third option: neither function moved, and `detect::batch`'s own first
bytes are now `?1049h`. `actuate::Page` tells the negotiation the page is already ours, so mode 1049
is entered exactly once — a second `?1049h` on a terminal without xterm's *already on the alternate
buffer* guard would save the cursor again and hand the shell back at the alternate screen's origin.
Every artefact now lands on a page that is cleared on the way in, painted over by the first frame,
and discarded by `?1049l` on the way out.

**Terminal.app is also the arm that proves the mitigation works**, and the proof is in the capture
above rather than in a new run. The artefact reached the primary screen *because* `?1049h` afterwards
switched away from it — a terminal that ignored mode 1049 would have left the artefact on the page the
frames were then drawn over, and the capture would look different. So this arm implements the page
switch and prints the probe, which is exactly the pair the fix needs.

The ticket asked for a scene here that watches it, **or the reason there is not one**. This is the
reason, and it is two parts.

*It is mechanically possible.* The judgement would be a row equality on a text screen — the scene
prints one marker, attaches, detaches, prints a second, and every arm's capture surface can read
whether the second line follows the first. That is the one thing even the surface with no style at
all can answer, which is why the ticket named it.

*And it would be four fifths a gate that cannot fail.* An arm runs every scene or it is not a run, and
on Ghostty, kitty, tmux and Ghostty-via-tmux the probe is consumed silently — those four rows are
green with the defect present and green with it absent, which is this workspace's named defect class
rather than a control. Only Terminal.app discriminates, and what it would discriminate on is a
**consequence** of the byte order rather than the byte order itself.

So the property went where it can be stated as a property. `scripts/page-order-gate.sh` runs the
`caps` application under `script(1)` — a real pty, which no test in the engine can have, because
`Tty::open` panics under `cfg(test)` — and asserts that the first escape sequence on the wire is
`?1049h`, entered once and left once. With the defect reintroduced it fails naming the number: **277
bytes reached the user's own page**, which is the whole capability batch. That is one instrument, one
run, no emulator, and it fails on the edit rather than on the emulator's reaction to it.

**What this directory keeps is the finding**, which is the part it was uniquely able to supply: no
instrument inside the engine could have represented a terminal that prints what it cannot parse, and
a control probe on a fourth VT lineage is what found one.

### The user's shell profile was in the measurement path, and it is in three arms' still

`do script` runs the command in a **login shell**, so the user's profile has already run when the
scene starts. On this machine that profile contains `export COLORTERM=truecolor`, unconditionally —
and Terminal.app 2.15 is a 256-colour terminal, which `caps` confirms from its own DA2 and palette.
Inherited, the run would have recorded a truecolor capability belonging to a line in somebody's
`.zshrc`.

So the arm's command line is an explicit `/usr/bin/env` with the eight variables the engine's
detection reads that Terminal.app does not set itself unset, and `TERM`, `TERM_PROGRAM` and
`TERM_PROGRAM_VERSION` left alone because those three are the terminal's own. It is the same argument
as the tmux arm's `-f /dev/null` and kitty's `-o` flags: *the result must be about the terminal and
not about somebody's config.*

**It moves no row in this suite as it stands, and that is luck rather than safety.** No scene reads a
colour the engine chose — scene 01 draws attributes and no colour, scene 04 writes its `SGR 41` as a
raw byte with no engine in the path. **And the other three arms do inherit**: kitty is given this
driver's environment, which has the same variable in it. Nothing is wrong with their captures and
nothing about them is protected either, which is worth knowing before a scene is added that reads a
colour.

### One fact, one declaration, two scenes

Terminal.app's capture surface carries no style at all — `contents` and `history` are
`type="text" access="r"` and the `tab` class has no styled variant. Production ticket 04 read that
out of the `sdef` on 2026-08-23 and recorded the arm as *glyph-grid scenes only*; scene 05 then made
the sentence too small, because an in-band question has no capture surface in its path. What the arm
actually cannot do is scene 01, and one row of scene 04.

Eleven near-identical entries in the arm's `NOT_COMPARED` would have said one thing eleven times,
which is how one of them comes to be worded differently from the other ten — and the report's table
would have carried ninety words per row, which is a table nobody reads. So the fact is declared once,
`Arm::no_style`, and the two scenes that need it consume it: scene 01 feeds it through
`Excluded::CannotAsk` like any other exclusion — **so the `STALE` rule still covers all eleven rows**
— and scene 04's `keeps-style` becomes `Seen::Unreportable` rather than `Seen::Reported`.

That last one is small and it is the honest half. Folded into `Reported`, the row would have printed
*the blanked half wears plain* — a claim about what Terminal.app renders, made by an instrument that
cannot see what Terminal.app renders, in the one row of the scene whose whole value is that the
families answer it differently.

**And the rows still say something.** A styleless capture carries the row's *text*, so scene 01's
eleven rows report *the label survived* and a scrolled or mis-sized screen is as loud on this arm as
on any other. `tests.rs` gates both halves against the committed captures: not one cluster in either
carries a style, and all eleven labels are where the scene put them.

### Two operational facts, both found by doing it

**`id of every window` is cumulative for the life of the Terminal.app process.** A closed window
stays in the list with `visible` false and its name still readable — thirty-five of them after
`close every window` on this machine. The Ghostty arm's equivalent fact is that window *indices* are
z-order and shift between `osascript` calls; this is the same class of surprise one application
along, and neither is written down anywhere but here.

The set difference is still an address, and the reason is worth stating rather than leaving to a
reader: the arm reads the id list **immediately before** it opens the window, so a stale id is in
both snapshots and cancels, and what the difference contains is what appeared between the two calls.
Filtering on `visible` would make the list live and would put *a window that has not finished
appearing* into the refusal's path — a race the cumulative list does not have.

**A `.decrqm` fixture was never covered by `.gitattributes`.** `*.vt` and `*.cpr` are marked `-text`
because `core.autocrlf` silently rewrote 1949 bytes of a Ghostty capture to 1926 with every test
still passing; scene 06 landed a third channel on 2026-08-29 and the rule was not extended to it.
No damage was done — none of the four captures committed in between contains a CR — and *that is
luck rather than safety*, which is the sentence the original rule was written to stop anyone having
to say. Found by the session that added a fifth `.decrqm`, which is one arm's worth of notice.

### The arm that can be handed a size and then insist on it

`number of rows` and `number of columns` are read-write on Terminal.app's `window` class, so this arm
sets them and then reads them back, and a window that came back some other size fails the run. kitty
can *ask* on its command line and report what it got; Ghostty offers a font size and nothing else.

It costs a two-step launch — `do script ""` for an empty window, the size, then `do script "…" in`
that tab — because a size set after the scene has started is a size the engine is told about a moment
too late. **The quiescence handshake is kept regardless**, for the reason the kitty arm gives: a
declaration is not an observation, and the handshake is what observes that the window stopped moving.

### What the arm confirmed rather than found, which is also worth the run

Scene 04's four text rows agreed on a **fourth VT lineage**: Terminal.app blanks the orphaned half in
both directions, exactly as Ghostty, kitty and tmux do. Architecture ticket 20's answer had been
checked only against recent reimplementations, and this is the first terminal to agree with it that
is not one. The `keeps-style` row is `not askable` here and always will be — and it was never a
comparison.

## 2026-08-29 — stage 5, and the instrument was inside its own measurement on one family of three

Scene 06 asks the terminal what it says about **mode 2026**: five DECRQM questions in one batch for
the state machine, and a bracket for when the terminal stops reporting the mode as set. **Five for
five on all four arms**, and three brackets against the four-row table in `quirks.rs`.

That headline is the least interesting sentence here. Four other things came out of it.

### Stage 5 was recorded as *if at all*, and the reason it looked unanswerable was the capture

Ticket 04 recorded this stage last, with the expectation written up front that *AppleScript's
tens-of-milliseconds jitter is the same order as Alacritty's 150 ms limit, so the sub-200 ms end may
be unanswerable on this machine.* `quirks.rs` says the same thing from the other side: production
ticket 05 asked for Ghostty's row to be measured, and it could not be, because **a force flush is a
rendering event and only a screen capture can see one** — and this repository's capture is an
AppleScript round trip four runs put between 136 ms and 623 ms.

Both sentences are about the **capture surface**, and scene 05 had already established that a
question asked in band does not have one in its path. DECRQM is such a question. So the jitter that
made the stage look unanswerable is not in the loop at all, and the resolution available is the
round trip on a local pty — one to three milliseconds on every arm measured.

**What that buys is not the force flush**, and the distinction is the whole of this scene's honesty:
a terminal could paint without clearing the flag or clear it without painting, and nothing here can
tell those apart. What is measured is the event Ghostty's own source calls *reset the synchronized
output flag*. The four rows of `quirks.rs` keep their provenance — *the implementation, read* — and
three of them now have a measurement printed beside them.

### The obvious instrument is wrong on one family, and the other two would have hidden it

The first shape written opens one block and polls it: one open where the shipped one costs seven.
Run as a raw-`printf`-style control probe with no vitui code in the path, polling every 250 ms:

| | polled every 250 ms | one probe per open |
|---|---|---|
| Ghostty 1.3.1 | still set at 262 ms, **reset by 517 ms** | still set at 904 ms, reset by 1002 ms |
| tmux 3.7c | still set at 760 ms, reset by 1007 ms | still set at 904 ms, reset by 1005 ms |
| kitty 0.48.2 | still set at 2013 ms, reset by 2261 ms | still set at 1912 ms, reset by 2010 ms |

**Polling brings Ghostty's reset forward by roughly half a second and leaves the other two where
they were.** An instrument that polls is inside its own measurement, and two families that do not
notice are exactly what would have made that invisible — the polled column, read alone, has two
terminals agreeing with their documentation and one apparently at 500 ms, which reads as a finding
about Ghostty rather than as a defect in the probe.

It is the third time in this directory that *run the control before the instrument* is what
separated the terminal's behaviour from the suite's: the kitty arm's conceal row and scene 04 are
the other two. So the shipped scene is one probe per open, a bisection over seven opens, and the
readiness timeout is derived from that count rather than typed.

### The bracket needs two clocks, and reading one for both ends is unsound in whichever direction

The terminal processes the question somewhere between the write and the reply. *Still set* at some
instant implies still set at every earlier one, so the **request** is the sound end for that answer
— a sleep is a floor, so the question cannot have been asked before it. *Already reset* implies
reset at every later one, so the **reply** is the sound end for that one. The first version recorded
one elapsed per probe, taken after the answer arrived, which pushes `still_set_at` past anything
that was observed.

Both are printed in every report, and `flush_bracket` takes one end from each column. The unit test
that holds it uses a deliberately non-zero round trip, because with the two clocks equal every
assertion would pass under a fold that read one of them for both.

### Ghostty's flag lets go earlier than its own source says, and this scene cannot say why

| | documented | observed by the shipped arm |
|---|---|---|
| Ghostty 1.3.1 | `sync_reset_ms = 1000` in `src/termio/Thread.zig` | still set at **879 ms**, reset by **973 ms** |
| tmux 3.7c | 1 s, its own documentation | still set at **971 ms**, reset by **1064 ms** |
| kitty 0.48.2 | 2000 ms, its own documentation | still set at **1985 ms**, reset by **2085 ms** |
| Ghostty via tmux 3.7c | tmux's | still set at **971 ms**, reset by **1063 ms** |

tmux and kitty land on theirs. Ghostty's whole bracket sits below 1000 ms, and the single-poll
control put it at (904, 1002] — the two overlap, so what both runs support is a reset somewhere
around 900–975 ms.

**The set end of each bracket reproduces exactly and the reset end jitters by a millisecond or
two**, because the first is the delay the bisection asked for and the second is when a reply landed.
The figures above are the runs in the committed reports, which are where these numbers live; a later
run moving the reset end by 1 ms is the reply clock and not the terminal.

**The instant the terminal armed the timer is not observable from inside**, which is the honest end
of this. The scene takes its `Instant::now()` after flushing the `h`, and Ghostty arms whenever its
own read thread gets there; anything spent between those two moves the whole bracket earlier by that
much. No `quirks.rs` row is changed on the strength of it — the table's numbers are what those
projects promise, and this is what one machine observed on one day.

The fourth arm is the one worth reading twice: **Ghostty-via-tmux answers tmux's number**, because a
DECRQM reply, like a cursor report, never leaves the innermost terminal. Its capture is
byte-identical to the plain tmux arm's, which `tests.rs` asserts rather than describes.

### What the scene deliberately does not answer

- **Whether the terminal held the frame back.** That is the paint, and no process inside a terminal
  can see it. Scene 06 says the flag was set; it does not say a frame was withheld.
- **Whether any of this binds.** It does not: the engine opens and closes its block inside one
  `write` — §8's twenty bytes of fixed framing — so a frame cannot approach the smallest of these
  limits. `quirks.rs` says as much where the field is declared, and the numbers are headroom written
  down where a future block spanning two writes will look for them. This scene changes where three
  of them came from, and nothing else.
- **A terminal that does not have the mode.** All four arms recognise it, so the `not recognised (0)`
  path and the `cannot express` declaration that would go with it are written and unexercised —
  `screen` 4.00.03 is the candidate on this machine, and it has no usable capture surface for the
  other scenes.

## 2026-08-29 — stage 3, and the two disagreements `ucd.rs` cites do not reproduce

Scene 05 asks the terminal itself what a cluster is worth: fifteen clusters written at column 1,
`CSI 6n` behind each, `CSI c` behind the batch. **Four arms answered fifteen for fifteen, and the
twelve surveyed rows agree with the engine's tables everywhere.**

That headline is the least interesting sentence here. Five other things came out of the run.

### The claim the scene was built around is stale, and it was a citation rather than a measurement

`crates/vitui-engine/src/ucd.rs`'s module docs are where *our tables are authoritative* is decided,
and the paragraph supports it with two named disagreements. Both were in the corpus on purpose:

| `ucd.rs` says | measured 2026-08-29 |
|---|---|
| *only 7 of 23 surveyed widen a VS16 emoji correctly* | Ghostty 1.3.1, kitty 0.48.2 and tmux 3.7c all widen `U+2764 U+FE0F` to 2 |
| *kitty sums a ZWJ family emoji to 6 where the answer is 2* | kitty 0.48.2 answers **2** |

The sentence cites
`.scratch/vitui-engine-architecture/research/02-grapheme-clustering-and-width.md` — **a survey in a
research document**, gathered from other people's write-ups and never observed here. This is the
first instrument in this repository to look, and on the three families §10 puts in tier 1, at the
versions on this machine, there is nothing left of either disagreement.

**Nothing about the decision moves.** The engine's tables stay authoritative, §8's
`CHA`-after-non-ASCII rule stays the bound, and a terminal agreeing is not a reason to start
following one. What moves is the *evidence*: the paragraph now cites a survey with a date on it and
says which half of it this directory has been able to check.

It is also this backlog's opening shape for the eighth time, with the sign reversed. A false MSRV,
a round trip agreeing with itself, a mask nothing read, a predicate no gate ran, a committed report
nobody re-ran — and now **a citation nobody re-measured**, which was load-bearing not for a silence
but for a paragraph's rhetoric.

### An agreement is the one result worth doubting, so the control was run twice

An instrument reporting that a recorded disagreement has gone away is indistinguishable, from the
outside, from an instrument that is not measuring anything. So the kitty ZWJ row was taken again
with a raw `printf` control probe — a shell script with literal UTF-8 in it, `stty raw -echo`, `dd`,
and **no vitui code anywhere in its path**. Seven clusters, and it agreed with the arm on all seven:
`A` 1, `漢` 2, the ZWJ family **2**, VS16 2, the flag 2, VS15 1, `U+200B` **0**.

*Run the control before the instrument* is the discipline the kitty arm's conceal row came out of,
and it is what separates *the terminal does not do it* from *the instrument cannot see it*. Here it
separated *the disagreement is gone* from *the probe is answering a constant*.

**The control's own first run was wrong, and its defect is worth the paragraph.** It built its
clusters with `printf '\U1F468\U200D…'`, and macOS `/bin/sh` is bash 3.2, whose `printf` does not
implement `\U` — so the terminal was handed the literal 33-character string and answered column 34.
A control that produces a plausible-looking wrong number is worse than no control, and what made it
loud was that 34 is not a width. The clusters are literal UTF-8 in the file now.

### A cursor report never leaves the innermost terminal, and one arm's two halves came apart

The Ghostty-via-tmux arm exists to see what tmux **forwards** — `capture-pane` and tmux's redraw
path are different code, which is how `quirks.rs`'s fourth entry was attributed. Scene 05 cannot use
that: tmux answers `CSI 6n` from its own grid on the pane's pty, and the question never reaches
Ghostty.

The two fixtures are **byte-identical**, device attributes included:
`tmux-3.7c-scene05-widths.cpr` and `ghostty-1.3.1-via-tmux-3.7c-scene05-widths.cpr`. `tests.rs`
asserts it, which turns the sentence into a gate.

So the arm's two channels have **two different subjects for one scene**, and the fix is a heading
rather than an exclusion: the rows are real answers about a real terminal, and what would have been
dishonest is a column headed *Ghostty* over them. `Arm::answers_cpr` is what heads them, and it is
two fields because one of them is a table column and the other is a paragraph.

This is *not* a sixth kind of non-number. `cannot ask` would have been wrong — the question was
asked and answered — and declaring the rows would have made every one of them `STALE`, since a row
excluded that agrees anyway counts as a failure. The rule held; it was the label that was missing.

### The scene needs no capture surface at all, and that is a fact about stage 4

Every other scene here is a picture, and an arm's whole job is to take one. This one's answers arrive
in band, so the **capture surface** is out of its path — Ghostty's `write_screen_file` and its
undocumented `vt` writer, kitty's remote-control socket, `capture-pane`. The arm launches the scene
and does nothing else.

**Not everything an arm needs, and the first draft of this paragraph said otherwise.** Whatever it
took to open a terminal it still takes: the Ghostty arm opens, addresses and closes its window over
AppleScript, so its automation grant is in the path for this scene as much as for the other two. The
claim is about the capture, and stating it wider was the report claiming more than was measured —
which is what this directory is for.

**That changes what Terminal.app's arm is blocked on.** Ticket 04 records it as *plain text only, so
glyph-grid scenes and nothing else*, read out of its own `sdef`: `contents` and `history` are
`type="text" access="r"` and no styled variant exists. That is a fact about the **capture surface**,
and scene 05 has no capture surface in it — so a Terminal.app arm could answer scene 05 **in full**,
scene 04 as text, and only scene 01 not at all. Recorded here rather than acted on; stage 4 is where
it lands.

### A review found four, and the sharpest was a published claim rather than code

**A stale answers file could have been read as this run's measurement.** The three arms deleted the
readiness file before and after a run and left `<ready>.cpr` beside it; both names carry the process
id, so a run whose scene never got as far as asking could have found a **previous** run's answers
under its own name. That is the accident this scene's four refusals exist to prevent, arriving
underneath all four of them — the batch well formed, the count right, the rows one, and the
measurement somebody else's. `common::clear_handshake` removes both halves now, and it is one
function so that the fourth arm cannot be the one that forgets.

**The comment and the report note overclaimed what scene 05 does without.** Both said the automation
grant was out of the path, and for the Ghostty arm that is false: it opens its window with
`osascript`, addresses it by set difference over `terminal_ids`, and closes it with `osascript` —
three Apple Events before the scene runs. What scene 05 does without is the **capture surface**, and
that is the whole of what unblocks a Terminal.app arm anyway. A committed report claiming more than
was measured is the failure this directory exists to prevent, so it is worth more than the two code
defects beside it.

Two smaller ones. `cursor_reports` indexed `replies[0]` after its count check, so a caller asking for
none — a legitimate request, satisfied by a sentinel with nothing behind it — panicked on a
well-formed capture. And the scene's read loop treated an interrupted read as the terminal declining
to answer, which would have surfaced as `NoSentinel`: the instrument blaming the emulator for its own
interruption, one line away from the paragraph warning against exactly that. Its empty-read case also
spun at 100% of a core for the whole deadline on a descriptor that is not a terminal, which is a
defect this repository has already met once under a different name.

### The survey needs an arm that disagrees, and there are two candidates

Four arms with identical answers is a survey with no spread in it, and a table of fifteen ticks
cannot tell *the terminals agree* from *the scene is asking easy questions*. The corpus is not soft —
it carries both variation selectors, a ZWJ family, a regional-indicator pair, a skin-tone modifier, a
keycap sequence, a combining acute, a zero-advance cluster and UAX #11's ambiguous class — so what is
missing is a party rather than a row.

Two candidates, in order of what they would cost:

1. **A different VT lineage.** Terminal.app is on every Mac, is a different lineage, and by the
   paragraph above can answer this scene in full. It is also the arm most likely to disagree: the
   three families measured here are all recent and all implement UAX #29 clustering.
2. **A locale.** `ambiguous` is the one row where a terminal is *entitled* to disagree — UAX #11
   class `A` is width 2 under an East Asian locale and the engine pins it narrow **by policy**. This
   suite sets no locale, so all four arms were asked the question in the configuration least likely
   to produce the answer the policy exists to overrule.

Neither is a defect in what shipped. They are what the second reading of this table needs, and
writing them down is what stops fifteen ticks being read as a result they are not.

## 2026-08-28 — the sixth quirk entry, and it came from a screenshot rather than from this directory

**JetBrains' IDE terminal mis-parses the colon form of SGR 38/48**, and it is the sixth row of
`crates/vitui-engine/src/quirks.rs` — the first that this directory did not produce and could not.

It arrived the way field work usually does. Somebody ran `cargo run -p vitui-apps --example counter`
in RustRover's terminal and the screen was wrong: the title's `C` gone, `Value: ` replaced by sixteen
colons, the panel's left border eight columns in from the edge, runs of `:` down the interior. A panel
and two strings.

**The mechanism is not a lost colour, and that is why it looks like corruption.** The engine writes
truecolour as `SGR 38:2::r:g:b`, the ITU-T T.416 colon form. A parser that handles only the semicolon
form and *ignores* what it cannot read loses the colour and nothing else. A parser that **abandons the
sequence** emits the remainder as text — so the colons and digits of the escape land on the screen as
glyphs, and every glyph after one is pushed sideways. The screen fills with `:` because an SGR
truecolour sequence is mostly colons.

**Three observations, and they are the whole of the evidence:**

| run | result |
|---|---|
| `examples/caps` in RustRover | `legacy_sgr false`, `TERMINAL_EMULATOR=JetBrains-JediTerm`, DA2 `0;10;0`, no XTVERSION, `TERM=xterm-256color` |
| `VITUI_FORCE_LEGACY_SGR=1 … --example counter` in RustRover | correct |
| the same binary in Ghostty 1.3.1, no lever | correct |

Beside them, the bytes `counter` writes were confirmed **identical** to the commit before the report
(`cmp` over two pty captures of two builds), so this is a standing property of that terminal rather
than a regression in anything above it.

**There is no capture and there cannot be one, which is the honest limit of this entry.** Every other
row here rests on a terminal's own screen dump — Ghostty's `+list-fonts`-era dump, tmux's
`capture-pane -e`, kitty's shipped `.so`. JediTerm has no facility to be asked what is on its grid, so
the arm that would belong in `arms/` cannot be written. What replaces it is falsifiability from the
other direction: the lever ships, so one run in that terminal with `VITUI_FORCE_LEGACY_SGR=0` puts the
garbage back, and one with `=1` takes it away.

**Recognised by `$TERMINAL_EMULATOR` starting `JetBrains-`**, which is VSCode's position exactly:
nothing in a query distinguishes it, and spec §10's refusal of terminfo does not reach here because
this is not inferring a capability from a name — it is overriding one that was measured. `starts_with`
rather than an equality, so a reworked terminal shipping under the same key is covered and one
shipping under a different value is not, which is where the observation stops.

**And the entry produced an instrument.** `crates/vitui-apps/examples/caps.rs` is what turned a
screenshot into three lines of evidence: attach, read `Capabilities::report`, detach, print, and draw
no frame at all so that nothing it prints can be a consequence of a component. Every gate in this
workspace is headless; none of them can answer *what did my terminal claim*. That question now has a
command.

## 2026-08-23 — the terminals agree about the repair and disagree about its colour

Scene 04, four arms, and the reason production ticket 06 was blocked on this directory rather than on
a reading of the spec. Architecture ticket 20 had two spec sentences that could not both hold at a
`View::child` clip and no way to choose between them, because the choice turns on *what a real
terminal does with a cluster printed over one half of a double-width glyph* — and that had been
recorded as uncertainty since 2026-08-20.

**The four text rows are unanimous.** `AB漢CD`, then one write over one half of the wide glyph:

| | version | over the continuation | over the head | a wide glyph over the continuation |
|---|---|---|---|---|
| kitty | 0.48.2 | `AB xCD` | `ABx CD` | `AB 漢D` |
| Ghostty | 1.3.1 | `AB xCD` | `ABx CD` | `AB 漢D` |
| tmux | 3.7c | `AB xCD` | `ABx CD` | `AB 漢D` |
| tmux → Ghostty | 3.7c → 1.3.1 | `AB xCD` | `ABx CD` | `AB 漢D` |

Every family blanks the orphaned half itself, in both directions, and **none of them has a clip to
consult**. So a surface holding a wide head with no continuation is a picture no terminal can be made
to show. §3 keeps its invariant; §4 gets a stated exception.

### The row worth the scene is the one that is not unanimous

The wide glyph carries a red background and the cluster written over its continuation does not.
**kitty keeps the orphan's own background. Ghostty and tmux blank to the SGR state in force.**

That was not the question the scene was added for, and it is the better answer. A repair delegated to
the terminal is not merely a repair the mirror does not know about — it is a repair whose *result
differs by terminal*, and there is no single mirror state that could be right on all three. *The
engine may as well repair* becomes **the engine must**, which is an argument no document could have
produced and no single arm could have found. Read across the arms, never down one — for the third
time.

It is **reported and never compared**. There is no right answer to hold an arm to, and inventing one
would have given two of three arms a permanent `FAILED` for something that is not a defect. The
per-terminal fact belongs to a capture, so `tests.rs` asserts it once per fixture.

### The instrument's first defect was in the instrument, for the third time

The probe repainted with `CSI 2 J`. **tmux pushes a cleared screen into the pane's history**, so ten
repaints a second scrolled the picture up through Ghostty's scrollback: the `--through-tmux` capture
came back with the whole scene on it twice, at rows 38 and 76, and every row read `""` — six `FAILED`
against a screen that had the right answer on it twice.

Two things about that are worth more than the fix. It was found by **the arm with two parsers in the
path**, which is the third time that arm has been the one to expose an instrument defect — it has the
most ways to go wrong and therefore the most to say. And the failure was *loud*: the parser's
row-count refusal and six mismatched strings, not a quiet pass. A probe that had painted the whole
screen, as the engine scene does, would have hidden it.

`EL` per row is what the probe does now. It touches no history in any of the four arms and erases
exactly the cells the scene is about to write.

### And a rule that was a comment became something the code will not do

`CONFORM_SAVE_CAPTURE` is a prefix now rather than a filename — there is a capture per scene, and one
name would have kept whichever ran last — and **it will not overwrite an existing fixture.** *A
capture is never regenerated to make something pass* had been a sentence in three files; it is a
branch now. An existing fixture is left alone and said so on stderr, rather than failing the run:
adding a scene means running an arm whose other scenes are already captured, and a refusal there would
make the new capture impossible to take without deleting the old evidence first. Saying nothing is the
other wrong answer — an operator who meant to regenerate would read silence as success.

## 2026-08-23 — wiring the quirk took the measurement away, and the committed report had gone stale

Found by running the *old* arms after building the new one, which was meant to be a five-second check
that a refactor had not broken them.

**`cargo run --example tmux` reported 10/11 where the committed `REPORT-tmux.md` says 11/11**, and
nothing was wrong. Production ticket 10 wired `attrs_dropped` on to the wire the session before: the
engine now consults `quirks.rs`, sees the fourth entry, and does not send SGR 53 to a terminal that
answers XTVERSION `tmux …`. So tmux's grid no longer holds an overline because it was never sent one.

Three things follow, and the third is the one worth the entry.

**A `FAILED` there would blame tmux for a decision of ours.** The row is the quirk table working. So
the report grew a fifth kind of non-number — `by design`, a fact about **the engine** — beside the
fourth this session already added. `cannot express` is the emulator's limit, `cannot ask` is the
instrument's, and `by design` is ours; collapsing any two of the three would hide one party behind
another. Three of the four arms now print one.

**The instrument has lost the measurement that earned the entry.** The `--through-tmux` arm exists to
answer *does tmux forward overline*; it answered no, the answer became a `quirks.rs` row, the row
became a mask, and the mask means the arm can never ask again. That is not a defect to fix here — an
arm that consulted the engine for what to expect would be checking the engine against itself, which
is the arrangement `conform/` exists to break. **What preserves the evidence is the committed
fixture**, captured while the bit was still on the wire, and this is the sharpest reason yet for the
rule that a capture is never regenerated to make something pass. The rule was written against a
lazy fix; it turns out to be load-bearing for the evidence surviving its own consequences.

**And the committed report had been wrong for a session with nothing going red.** That is the exact
mechanism this directory relies on: it reports rather than gates, and *a worsening number arrives as
a review-visible diff*. It only arrives if somebody regenerates the file. Ticket 10 changed what
three of these four arms observe and regenerated none of them — a reasonable omission, since the
ticket was about `quant.rs` and had no reason to think it had moved an instrument. The lesson is not
*run everything*: it is that **a committed report is a claim with a date on it**, and the date is the
last time an arm was run and not the last time the repository changed.

The same shape as this backlog's opening finding, for the fifth time: a declaration that is
load-bearing for the silence around it. A false MSRV disabling lints, a round trip agreeing with
itself, a mask nothing read, a predicate no gate ran — and now a report nobody re-ran.

## 2026-08-23 — the third family, and the two questions it was built to settle both had the same answer

The kitty arm exists. `cargo run --example kitty` opens a window, drives the engine in it, reads the
screen back over `kitten @ get-text --extent screen --ansi` and closes it — **8/10 agreed, one row
unaskable**, and both halves of that sentence are findings.

The two design questions ticket 04 left open were settled **before** the arm was written, by control
probes with no engine in them, and neither answer was the one the ticket predicted.

### `--ansi` is not a third dialect, and the probe is the only reason that can be said

The ticket recorded a leading `ESC[m` per row and `ESC[22;2m` for dim as evidence that kitty might be
a dialect of its own, on the tmux precedent. It is not. Every construct kitty emits is ECMA-48 and
means what ECMA-48 says:

| sent | returned | |
|---|---|---|
| `1` | `22;1` | reset intensity, then bold — legal, and `22` then `1` is what the parser already did |
| `2` | `22;2` | and `1;2` together comes back as `1;2`, so the `22` is a clear of the *other* one |
| `4:2` `4:3` | `4:2` `4:3` | unchanged |
| `21` | `4:2` | **kitty implements ECMA-48's 21 as double underline**, where a meaningful population implements it as bold-off. Nothing here emits 21, so it costs nothing — but it is worth knowing that the two readings are both alive in the field |
| `38;2;10;20;30` | `38:2:10:20:30` | the colon form **without** T.416's empty colour-space id, which the parser already accepts because it skips empties rather than counting them |
| `58:2::0:0:255;4` | `58:2:0:0:255;4` | same, on the other axis |
| `42` | `42` | bare, so the `42..=45` fold guard is not reachable from this arm at all |

So `parse` takes `Dialect::Ecma48` for this arm and the enum does not grow. **A dialect is a
disagreement about meaning, not a difference in style**, and only one of those costs an attribute.

It would have been possible to reach the same conclusion by parsing a capture and observing that both
dialects agree on it — and that would have been luck. tmux numbers its underline styles 42–45 *so
that* the fold lands on `4:2`–`4:5`, so agreement between the two readings is exactly the coincidence
that let one parser get away with being wrong for two stages. The control probe is what makes this a
statement about ECMA-48 rather than about a numbering accident.

### `Through` does not grow a kitty variant, and the reason it was wanted has evaporated

`Through` is *what is inside the window the Ghostty arm photographs*. kitty is a different photograph,
so it is `examples/kitty.rs` — the third arm, and the third `[[example]]` stanza the `Cargo.toml`
comment predicted would cost three lines.

The ticket also wanted a `--through-kitty`, and named its purpose: attributing conceal the way
`--through-tmux` attributed overline. **There is no such arm to build.** tmux has a far side because
tmux is a multiplexer; kitty is an endpoint, and nothing downstream of it can be read. What settled
conceal was not another capture.

### Conceal and overline are settled, and the evidence is a shipped binary

`FINDINGS.md` left conceal open on the possibility that kitty resolves the foreground to the
background at paint time, storing nothing in a flag and rendering correctly. Two observations close
it, and neither is a screen.

**`kitty.fast_data_types.so` on this machine carries kitty's `Cursor` repr**, which is an enumeration
of every formatting attribute the cursor has:

```text
Cursor(x=%u, y=%u, shape=%s, blink=%R, fg=#%08x, bg=#%08x, bold=%R, italic=%R, reverse=%R,
       strikethrough=%R, dim=%R, decoration=%d, decoration_fg=#%08x, text_blink=%R)
```

and the attribute constants beside it are `BOLD ITALIC REVERSE MARK STRIKETHROUGH DECORATION BLINK`,
with `set_attribute` accepting `reverse strike mark bold italic decoration` and rejecting anything
else as *"Unknown cell attribute"*. Three tables, no conceal and no overline in any of them. **SGR is
a mutation of the cursor**, so an attribute the cursor cannot carry is one no cell can hold and no
paint can consult — paint-time resolution needs a flag, and there is none.

**And the control probe agrees from the other side.** `printf '\033[8;31mconcealred'` comes back as
`ESC[31m`: the foreground is stored *unmodified*, so nothing was resolved to the background on the way
in either. That is the hypothesis tested rather than argued away.

So `quirks.rs` has a fifth entry — kitty, recognised by XTVERSION answering `kitty(0.48.2)`, dropping
conceal **and** overline. It is the first entry here whose evidence is a binary rather than a screen,
and the first with a mask of more than one bit, which is a case the serializer had never been given.

### The row the instrument had to refuse to answer, and it is the finding worth the arm

kitty's serialiser writes a **dotted** underline as `CSI 4 : m` — parameter 4 with an empty
sub-parameter — and a dashed one identically. The strings `4:2;` and `4:3;` are in the shipped binary
and neither `4:4;` nor `4:5;` is: a table with a hole in it.

ECMA-48 reads an omitted parameter as the default, and SGR 4's default is 1. So the capture says
*single* for a cell holding *dotted*, and **the comparison would have reported a third dropped
attribute that is not dropped at all.** A `quirks.rs` row would have followed, describing a
misbehaviour that is not happening — which is precisely the failure that table exists to avoid, and
it would have been produced by the instrument built to prevent it.

What separates *not rendered* from *not serialised* here is a control the emulator supplies itself:
**`CSI 4 : 0 m` comes back as nothing at all**, so a `4:` in a capture proves the cell holds a
*non-zero* decoration and only its number was lost. The instrument can see that something is
underlined and cannot see which of two things — which is a fourth kind of non-number, and `compare/`
supplied three.

`cannot express` is a fact about the emulator; this is a fact about the **instrument**. Ticket 04
predicted the cell would be needed and predicted the wrong arm — it expected Terminal.app's
plain-text-only capture surface to be the first thing the vocabulary could not describe. kitty got
there first and with a sharper case: Terminal.app carries no style at all, where kitty carries ten of
the eleven and mis-spells the eleventh.

The rules that stop it being an excuse are in `SCENES.md`, and one of them is a gate: **a row declared
unaskable that agrees anyway is reported `STALE` and counted as a failure.** An excuse nobody rechecks
is the same kind of thing as an MSRV nobody compiles, which is this backlog's opening finding for the
fourth time.

### Two smaller things the run produced

- **kitty is the first *emulator* arm that can be handed a size.** `-o initial_window_width=80c -o
  initial_window_height=24c` is a real geometry in cells and `kitten @ ls` reports back what it got.
  The quiescence handshake is kept anyway: one command-line option is a declaration, and the
  handshake is what *observes* that the window stopped moving. Ghostty's `surface configuration`
  offers a font size and nothing else; tmux can set a pane size and is not an emulator.
- **A launcher that execs is not a launcher that takes a command line**, and the shared helper was
  handing out the wrong one. kitty execs its argv; given `common::scene_command`'s single string it
  looked for a file whose name ends in `--scene 01`, found none, said nothing, and the run failed
  twenty seconds later as *the scene never reported a presented frame*. The helper is `scene_argv`
  now and the two string-taking arms join it themselves, because neither AppleScript's quoting nor
  tmux's is a rule a shared helper could guess.

## 2026-08-23 — kitty, installed for stage 4, and it arrives with two candidate quirk bits

No arm exists yet. These are control probes — raw `printf` into a kitty window, `kitten @ get-text
--extent screen --ansi` back out, **no engine anywhere in it** — run before the instrument, which is the
discipline stage 1 established and the reason the tmux entry could be attributed at all.

### kitty 0.48.2's cell cannot hold an overline, from its own header

`kitty/data-types.h` at `v0.48.2`:

```c
struct { bool bold, italic, reverse, strikethrough, dim, blink;
         uint8_t decoration; color_type fg, bg, decoration_fg; } sgr;
```

**Six flags and the underline field. No overline bit, and no hidden/conceal bit.** The dump agrees:
`overline` and `conceal` come back bare, where `dim` comes back as `ESC[22;2m` and `4:3` as `4:3`.

So kitty is a candidate for **two** `attrs_dropped` bits, and it is one of §10's tier-1 seven. But the
two are not equally settled, and the difference is the whole of stage 1's lesson:

- **Overline is settled.** There is no bit in the cell to store it in, so it cannot be rendered, and no
  further arm can change that.
- **Conceal is not.** All a dump can say is that `get-text` did not serialise it, and *not stored* and
  *not serialised* are exactly the two explanations a single dump cannot choose between. kitty could
  perfectly well implement SGR 8 by resolving the foreground to the background at paint time, which
  stores nothing in an `sgr` flag and renders correctly. **That row waits on an arm that reads the far
  side** — the `--through-kitty` shape, the way `--through-tmux` was what finally attributed overline.

Writing a conceal row into `quirks.rs` on this evidence would be the table inventing a misbehaviour,
which is the failure it exists to avoid.

**Settled the next day, and not by the arm this paragraph expected.** There is no far side to a
terminal endpoint, so the `--through-kitty` shape named above does not exist to be built. What closed
it was the shipped binary — kitty's `Cursor` carries no conceal attribute for a paint to consult — and
a control probe showing `SGR 8 ; 31` comes back as `SGR 31`, foreground unmodified. See the entry at
the top of this file. The reasoning here was right and its proposed instrument was wrong, which is
worth leaving in place rather than editing away.

### `Smol` is missing from kitty's terminfo too, so the tmux entry is not one emulator's bad luck

kitty ships its own `terminfo/x/xterm-kitty` with `Smulx` and `Setulc` and **no `Smol`** — and it is not
installed into the system database at all, so `infocmp -x xterm-kitty` fails outright and has to be read
with `-A /Applications/kitty.app/Contents/Resources/terminfo`.

That is a second, independent confirmation of the mechanism behind the tmux quirk: `Smol` is absent from
**every** terminfo on this machine — `xterm-ghostty`, `xterm-256color`, `tmux-256color`,
`screen-256color` and now `xterm-kitty`. tmux therefore drops overline under kitty as well, and the
entry is right for the common case rather than for one emulator's description.

There is a second-order joke in it worth noticing rather than enjoying: **kitty cannot render overline,
and kitty's terminfo is correct to omit `Smol`.** Ghostty *can*, and its terminfo omits `Smol` anyway.
The same missing capability is accurate in one description and wrong in the other, and tmux cannot tell
them apart — which is the argument for §10's refusal of terminfo in one sentence.

## 2026-08-23 — the second family, and it disagreed

Production ticket 05 asked for the eleven attribute facts on two emulator families. The second family
is tmux 3.7c, and the answer is **not one number**: tmux stores eleven of eleven and forwards ten.

### tmux accepts overline, stores it, hands it back, and never sends it

Three captures of scene 01, all three committed to `fixtures/`:

| path | overline |
|---|---|
| the engine → Ghostty 1.3.1, `write_screen_file:…,vt` | present |
| the engine → tmux 3.7c, `capture-pane -p -e` | present |
| the engine → tmux 3.7c → Ghostty, `write_screen_file:…,vt` | **gone** |

Ten of the eleven bits survive all three paths, so this is one attribute and not a broken route. And
**no engine is needed to reproduce any of it.** A raw `printf '\033[53moverline\033[0m'` into a tmux
pane comes back from `capture-pane -e` as `ESC[5:3m`, and the bytes tmux writes to its own attaching
client — captured with `script -q out tmux attach`, which has neither this engine nor Ghostty in it —
contain **no `53` anywhere at all**. The row arrives with no SGR while its neighbours arrive with
`ESC[5m` and `ESC[9m`.

**The mechanism, and it makes the entry's shape a decision rather than a guess.** tmux 3.0's CHANGES:

> Add support for the overline attribute (SGR 53). The `Smol` capability is needed in
> `terminal-overrides`.

`Smol` is defined in none of `xterm-ghostty`, `xterm-256color`, `tmux-256color` or `screen-256color`
on this machine. `Smulx` **is** defined in `xterm-ghostty`, which is exactly why the underline styles
survive the trip and overline does not. Enabling it closes the loop: with
`set -as terminal-features ",xterm-ghostty:overline"` tmux emits `ESC[53m` and the attribute arrives.

So the quirk has no version boundary — every tmux drops it by default, and before 3.0 there was no
overline output at all. What varies is a configuration, not a release.

**And Ghostty renders SGR 53**, which is the first row of that table. So `xterm-ghostty`'s terminfo
omits a capability the terminal it describes actually has, and tmux believes the terminfo. That is
spec §10's refusal of terminfo arriving as field evidence from a direction nothing planned for: the
refusal was argued as *do not infer capabilities from a name*, and this is the same refusal earning
its keep against a description the terminal's own author ships.

### The instrument's second defect was, again, in the instrument

The first tmux run reported overline as **blink**. tmux never rendered blink there; the parser
invented it — which is word for word the defect stage 1 recorded one level up, arriving again from a
place that had been checked.

`grid.c`'s `grid_string_cells_code`:

```c
if (s[i] < 10)
        xsnprintf(tmp, sizeof tmp, "%d", s[i]);
else
        xsnprintf(tmp, sizeof tmp, "%d:%d", s[i] / 10, s[i] % 10);
```

**A colon in a tmux capture is a divided-by-ten, not a sub-parameter.** tmux numbers its underline
styles 42–45 *so that* the division lands on ECMA-48's `4:2`–`4:5`, and the two readings agree — which
is precisely why a parser that knew nothing about this passed every test stages 0 and 1 had. Overline
is 53 and rides the same branch, so it comes back as `5:3`, and `5:3` in ECMA-48 is blink.

There is no reading of those five bytes that is correct for both formats. So `parse` now takes a
`Dialect` and **has no default**: a caller that cannot say which format it captured cannot be trusted
to have captured either — the same argument as `expected_rows` being a parameter. Both readings are
pinned by tests, so the dialect is asserted rather than assumed.

The generalisation is worth more than the fix. Stage 1's lesson was *the flattening was correct for
every capture stage 0 had*. This one is a level up: **the parser was correct for every capture that
had ever been taken, and a second source of captures is what made it wrong.** An instrument gains a
new failure mode from each arm it grows, and the arm that finds it is the one that was added last.

### And the fold's own trap, caught by writing the test for it

tmux's underline codes are 42–45. **ECMA-48's background colours are 40–47.** So the arm that reads a
folded `4:2` as 42 sits inside the arm that reads a bare `42` as a green background, and in a `match` the
first one written wins — the fold's arm was, so `ESC[42m` came back as a double underline in *both*
dialects. None of the five committed fixtures has a background colour on a row that would have shown it.

The arm is guarded on the fold now (`42..=45 if folded.is_some()`), and there is a test that fails
without the guard. Worth its paragraph because the trap is in the numbering rather than in the code: any
future reading of tmux's colon form lands in the same overlap.

### `attrs_dropped` is populated and nothing reads it

Found while writing the entry the evidence above earned. `Quirks::apply` fills
`caps.private.attrs_dropped`, `Capabilities::report` prints it, and **`serial.rs` never reads the
mask** — while `caps.rs`'s own module docs and spec §10 both say *an unsupported one is dropped
silently at serialise time*.

So the field scene 01 was built to feed is not wired to the thing it claims to control. The tmux entry
is therefore a recorded observation and not yet a behaviour, and both files now say so where a reader
would otherwise take it on trust. Wiring it is production ticket 10 — and it is not a one-line mask,
because `quant::OnTheWire` has to narrow the expectation too, the way it already drops a hyperlink the
terminal cannot express, or the round trip will fail on a bit the engine correctly declined to send.

**Resolved the same day, and the prediction in that paragraph held exactly.** Production ticket 10 put
the mask in `Quantiser` rather than in `serial.rs` — the module whose whole argument is *the one
placement that keeps the equality filter exact* — so one field narrows the wire, the mirror, the gap
pricing and `OnTheWire` at once, and the round trip closes on a tmux-identified terminal. The estimate
was one file short in a way worth recording: `Quantiser::narrows` is the fast path that decides whether
the scroll pre-pass may compare two **slices**, and a truecolor terminal that drops a flag narrows
something — so a mask wired without touching that predicate would have forfeited every scroll on tmux
while every gate stayed green. Nothing in the suite covered it until a gate was written for it.

This is the backlog's opening finding a third time: a declaration that is load-bearing for the silence
around it. The round trip agreeing with itself, an MSRV agreeing with the lints it disabled, and a mask
agreeing with a serializer that never asked.

### Two operational notes

- **A Ghostty window's `command` runs with the GUI application's `PATH`**, which is
  `/usr/bin:/bin:/usr/sbin:/sbin` — Homebrew's `/opt/homebrew/bin` is not on it. The first
  `--through-tmux` run opened a window, failed to exec `tmux`, and reported *the scene never presented
  a frame*: the readiness timeout doing its job and saying nothing about the cause. The path is
  resolved in the driver now, which has the developer's `PATH`, so the refusal names what is missing.
- **A tmux server outlives the client in it.** Closing the window kills the client and leaves the
  session detached, so the arm kills the server explicitly. Without that, a run leaves litter the
  *next* run's socket-collision refusal fires on.

## 2026-08-23 — stage 1, and the instrument found its first defect in itself

Ghostty 1.3.1 agreed with the engine **11/11** on scene 01. Three things are worth more than that
number.

### The dump carries all eleven attributes, and the design note said it carried five

Ticket 04's table recorded Ghostty's `vt` writer as preserving *"attributes 1/3/4/7/9 and underline
colour 58"*. A control probe — raw `printf` into a Ghostty window, no engine anywhere in it — run
before the driver was written says otherwise:

| sent | returned |
|---|---|
| `1` `2` `3` `5` `7` `8` `9` `53` | all eight, unchanged |
| `4` | `4` |
| `4:2` `4:3` `4:4` | `4:2` `4:3` `4:4` |
| `58:2::0:0:255` | `58;2;0;0;255` |

**Dim, blink, conceal, overline and the underline *styles* all survive**, which is what makes an
eleven-boolean assertion answerable at all. Running the control before the instrument is what
separated *the terminal does not render it* from *the dump does not serialise it*; with only the
engine's own arm, an absent overline would have had two explanations and no way to choose.

### `4:2` is one parameter, and the committed parser was reading it as two

The stage 0 parser split an SGR body on `;` **and** `:` into one flat token stream, because that
makes the two colour spellings read through a single path. The control probe's `4:2` came back
through it as SGR 4 followed by SGR 2 — **double underline read as underline plus dim**, an
attribute the terminal never rendered and the instrument invented.

ECMA-48 separates parameters with `;` and a parameter's sub-parameters with `:`, and the parser now
does too; the colour arms accept either spelling by looking for the tail in the two places it can
be. Six tests hold it, and the shape of the mistake is worth keeping in mind: **the flattening was
correct for every case stage 0 had captures for.** It took bytes from a second emulator, produced by
a scene nobody had written yet, to make it wrong.

### The handshake proves a frame exists; it does not prove the window stopped moving

The first live run photographed a screen whose top two rows had scrolled off, and every row of the
report was wrong by two. The scene had presented — the readiness handshake was satisfied — but a
Ghostty window **settles its size after the process inside it starts**, and the frame painted at the
first geometry was reflowed at the second. Two runs on this machine were handed 156×45 and 72×24, so
this is not a rare race.

The fix is not a longer delay. The scene now redraws on every wake and stamps a frame counter, and
the driver waits for that stamp to **stand still** for 500 ms before photographing. An idle vitui
application costs zero wakeups, so a stamp that stops moving is a screen that has stopped moving —
an observed condition, in the same shape as the readiness handshake it extends, rather than a sleep
tuned until it passed.

It also generalises past this arm: **any capture of a window an emulator has just opened is a
capture of a screen that may still be settling**, and the tmux arm is exempt only because
`capture-pane` runs against a pane whose size the harness set.

### Git rewrote the first committed capture, and every test still passed

Found while committing stage 1. `core.autocrlf = input` on this machine, and `git add` rewrote CRLF
to LF on the way into the index: **1949 bytes of what Ghostty actually sent became a 1926-byte
blob.** Twenty-three carriage returns, gone.

Nothing went red. The parser skips CR the way a terminal does, so the fixture stopped being the
bytes a terminal sent and carried on passing every assertion over it — a capture that is evidence,
silently not being the evidence, which is this directory's own failure mode one level up. `*.vt` is
`-text` in `.gitattributes` now, and there is a test asserting the twenty-three are still there,
because an attributes file nobody checks is the same kind of thing as an MSRV nobody compiles.

`-text` and not `binary`: `binary` means `-text -diff` together, and the diff is worth keeping. A
capture that changes must be reviewable *as a diff* — that is the entire mechanism by which
something which reports rather than gates still catches a regression.

## 2026-08-22 — stage 0, and the capture format is the finding

The instrument does not exist yet. What exists is the answer to *what can a dump-based capture see*,
and it is narrower than the design assumed — narrow enough that it changes how ticket 06's scene has
to be built.

### A grid-to-text dump cannot see a double-width glyph's second cell

Sent through tmux 3.7c, `capture-pane -p -e`, pane 20×6:

```
printf 'AB\346\274\242CD\r\n'      →  41 42 e6 bc a2 43 44 0a
                                       A  B  漢          C  D  \n
```

**No padding cell, no continuation marker, nothing.** On screen `漢` occupies columns 2 and 3 and `C`
is at column 4; in the dump `C` follows the glyph immediately. To know that, a reader has to know `漢`
is double-width — **which is the thing under test.**

Ghostty 1.3.1's `write_screen_file:…,vt` was already observed to do the same. Two emulators from two
lineages, one behaviour, and it is not a quirk of either: **it is inherent to re-serialising a cell
grid as text.** So the design's prediction — *ticket 20 will not be settled by the dump* — is
confirmed and generalises from one arm to the whole *category* of arm.

The consequence is a design constraint rather than a disappointment, and it is written into
[production ticket 06](../.scratch/vitui-engine-production/issues/06-the-clip-edge-pair-decided-on-what-was-observed.md):
the scene must put a **unique ASCII sentinel in every cell that should be a continuation**, which
converts *how wide was that glyph* into *is the sentinel still there* — a question a text dump can
answer — or it must use CPR, where the emulator reports the column itself.

### tmux 3.7c parses the colon form, which makes two of tier 1 observed rather than inferred

| sent | returned | |
|---|---|---|
| `ESC[38:2::255:0:0m` | `ESC[38;2;255;0;0m` | exact channels |
| `ESC[1;4;58:2::0:0:255m` | `ESC[1;4m` `ESC[58;2;0;0;255m` | exact channels, attributes split off |
| `ESC[38:5:200m` | `ESC[38;5;200m` | exact index |

Both emulators tested so far **normalise to semicolons on output**, whatever spelling they were sent.
That is what makes the dump a usable measuring channel at all: a terminal that *mis*-parsed the colon
form would return channel values off by one, which is precisely the failure
`serial::emit_color`'s own comment names. Agreement on the normalised output is therefore evidence
about the *parse*, not about the spelling.

With [arch 23](../.scratch/vitui-engine-architecture/issues/23-the-two-sgr-spellings-and-which-one-is-the-default.md)'s
Ghostty observation, **two of tier 1's seven are now observed.** Five remain inference from libvaxis's
three quirk entries. Neither of the two is one of the three terminals those entries name, so nothing
here contradicts them.

### The reset is not one sequence, and a parser that assumes it is will be wrong

tmux closes a colour-only run with `ESC[39m` — default foreground, leaving other attributes alone —
and an attribute-bearing run with `ESC[0m`. Both appear in
`fixtures/tmux-3.7c-attrs-and-colours.vt`. A parser must treat the SGR stream as *state*, not as
paired delimiters. This is the kind of thing that reads as a detail until it silently mis-attributes a
style to the following cell.

### What is not yet known

- **Whether a tmux arm is worth what it costs.** `capture-pane` re-serialises *tmux's* grid, so this
  arm measures tmux's conformance and never the emulator behind it. That is legitimate — tmux is in
  §10's own tier-1 list and `quirks.rs` already cites its 1 s sync-flush limit — but it must never be
  read as a proxy, and `SCENES.md` says so.
- **The empty-capture failure mode has not been exercised.** `screen -X hardcopy` returned rc=0 and a
  zero-byte file during the design investigation, and the refusal that must catch it is unwritten. It
  is the first thing the parser needs, not the last.
- **Everything about the engine.** No engine bytes have been through this path yet: the public API is
  being changed by
  [production ticket 03](../.scratch/vitui-engine-production/issues/03-the-uri-travels-at-the-verb.md)
  as this is written, and a driver built against today's `Restyle` would not compile tomorrow.
