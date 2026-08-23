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
| **tmux** | **tmux** | `capture-pane` re-serialises *tmux's* grid, so the emulator behind it never sees the engine's bytes. A legitimate target — tmux is in §10's tier-1 list — and **never a proxy for one.** Headless, which makes it the only CI-shaped arm |
| **Ghostty** | Ghostty | its own screen dump, over an AppleScript surface. Needs a window server and a macOS automation grant |
| **Terminal.app** | Terminal.app | plain text only, so glyph-grid scenes and nothing else |

A row that does not say which of these it came from is not a result.

## The three kinds of non-number, inherited from `compare/`

`cannot express` is a fact about the emulator. `not run here` is a fact about the run. `FAILED` is a
defect. **A missing row reads as a win, and that is the single easiest way for this directory to become
dishonest** — more so than in `compare/`, because a capture that silently returns nothing looks like
agreement rather than absence.

## 01 — the eleven attributes, one per row

12×12. Each row sets one of the eleven attribute facts `Capabilities::attrs_dropped` covers, writes a
label, and resets. Assertion: eleven booleans — did the dump carry each attribute back.

Chosen as the first scene because `attrs_dropped` is the field with **no query** — the eleven facts are
in the capability set precisely because nothing can ask for them — and a dump *is* a query for them.
No column arithmetic, no width tables, no timing. It emits `quirks.rs` rows directly.

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
