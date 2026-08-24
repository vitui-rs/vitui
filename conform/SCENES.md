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
| **Terminal.app** | Terminal.app | plain text only, so glyph-grid scenes and nothing else. Not built |

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
behind a word that means the terminal lacks one. Three rules keep the new cell from becoming an
excuse:

1. **An arm declares its unaskable rows before the run, with the reason.** A limitation discovered
   after seeing the answer is an excuse, not a declaration.
2. **The row is still printed, with what was nonetheless observed.** It leaves the numerator and the
   denominator both; it does not leave the table.
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

The same three rules apply to both new cells, and the third is worth reading twice for `by design`: a
row the engine promised to withhold that arrives anyway means the declaration and `quirks.rs` no
longer describe the same terminal.

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

Its successor — the scene ticket 06 actually needs — puts a unique ASCII sentinel in every cell that
should hold a continuation, so the question becomes *is the sentinel still there*. Not yet written.

## 03 — both SGR spellings, same colour

Any size. Emit a truecolor colour as `38:2::r:g:b` and again as `38;2;r;g;b`, and an indexed one as
`38:5:n` and `38;5;n`. Assertion: the resolved channels are equal and correct.

This is the scene that answers [arch 23](../.scratch/vitui-engine-architecture/issues/23-the-two-sgr-spellings-and-which-one-is-the-default.md)'s
third question for one terminal at a time. Both emulators tested normalise to semicolons on output,
which is what makes the comparison meaningful — agreement on the output is evidence about the parse.
