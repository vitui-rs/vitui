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
| **Terminal.app** | Terminal.app | plain text only, so glyph-grid scenes and nothing else |

A row that does not say which of these it came from is not a result. **The tmux pair is why that
sentence needed a fourth row**: *tmux* and *what tmux does to a terminal downstream of it* gave
different answers on the same scene, and a table with one tmux column could only have printed one of
them.

## The three kinds of non-number, inherited from `compare/`

`cannot express` is a fact about the emulator. `not run here` is a fact about the run. `FAILED` is a
defect. **A missing row reads as a win, and that is the single easiest way for this directory to become
dishonest** — more so than in `compare/`, because a capture that silently returns nothing looks like
agreement rather than absence.

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
has: **Ghostty 1.3.1 agrees on eleven, tmux 3.7c stores eleven and forwards ten**, and the missing one
is `quirks.rs`'s fourth entry.

**Read across the three arms, never down one.** Overline is present in Ghostty's dump and present in
tmux's grid and absent past tmux, and no single arm could have said which of the three parties lost
it. That is the scene's real assertion: eleven booleans *per path*, compared.

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
