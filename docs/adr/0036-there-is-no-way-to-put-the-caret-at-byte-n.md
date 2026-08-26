---
status: accepted
date: 2026-08-26
---

# There is no way to put the caret at byte N

A caret is a **`(byte, column)` pair** whose fields are private, and the only things that make one are
gestures: `Caret::HOME`, a cluster step, a click's column, `Home`, `End`, and a pair some earlier state
stored. `Text::set_pos` takes a `Caret`; there is no `Text::set_caret(byte)` and `Caret.byte` is not a
field a caller can read or write. The deletion is kept by three `compile_fail` pairs, each with a twin
naming the protected item by path.

`input` and `textarea` are **one component and one flag**, and the flag is a break rule
(`edit::WrapKind`) rather than a width.

## Context

Spec §11's headline is that **every backward question about text is O(prefix) unless an index already
knows the answer, and the index that answers them is the one the wrapping already needs.** `Graphemes`
is a forward iterator, so the boundary *before* an offset can only be found by segmenting forward from
one already known — and which one you have decides the complexity. Measured here: one `Left` at the end
of a pasted megabyte is **6.5–7.6 µs from the caret's own visual row against 31 300–38 400 µs from byte
0**, a factor of **4 800–5 000**, where §11 remembers 9 655. A range and not a figure: it is a
timing and therefore a report (§21, R15), and what is gated beside it is the count — 1 079 316 bytes
segmented against 152.

That argument is about cost. The API consequence is about **correctness**, and it is a deletion:

- An arbitrary byte offset is not a cluster boundary. Over the cluster corpus every off-boundary offset
  is reachable and nothing downstream notices: the column is computed from the caret, the row is looked
  up from it, the selection is measured from it, and **the screen renders exactly as it should**.
- A caret's column cannot be recovered from its byte for free. Restoring a pair is a move; recovering
  one crosses the prefix — **more than 100 000 clusters** on a 200 kB line, which is §11's *0.0007 µs
  against 6 109*.

So a `pub byte` field is `set_caret` with three keystrokes fewer, and both had to go.

## Decision

**`Caret` is opaque.** Two accessors, one constant, and a crate-private constructor. Every gesture that
places a caret is a method on `edit::Text`, and each lands on a boundary by construction.
`edit::defective::at_byte` is the one door a gate takes through, and it is public inside a module named
for what it is — **a defect nobody can build is a gate nobody can watch fire.**

**One flag, and it is a break rule.** `WrapKind::Ruler` breaks every `w` columns with no word breaks
and no newlines; `WrapKind::Words` is a greedy word wrap over hard lines. The near-miss is spelling the
ruler as a `Words` index at a very large width, and it is refused on a measurement rather than on
taste: at `u16::MAX` a megabyte is **one row**, so the caret's row start is byte 0 and `Left` is back
to O(prefix) — the one thing the flag exists to prevent. Measured, 1 row against 500 and byte 0 against
one window back.

**The selection is one anchored range in bytes.** Contiguity is not the argument; the **unit** is. A
run list addresses cluster indices and a buffer addresses bytes, so every gesture pays a prefix walk to
convert: over a 19 090-cluster document, fifty conversions of one selection walk **477 250 clusters against 0**.
`edit::defective::RunList` is that spelling, kept runnable and counted.

**Block selection is not built.** It is a rectangle over *visual* rows, which neither the run list nor
the anchored range expresses, and adding it means a second selection representation beside the one
every gesture, every edit and every undo entry is written in. That belongs to whoever writes the code
editor.

**The undo ring is bounded twice and says when it dropped something.** Entries *and* bytes, because one
large paste is one entry. Crossing either bound sets `truncated`, and that flag is the whole type:
sixteen entries after sixty-four edits runs sixteen steps, **returns `true` every time**, and the
document is not back to the original — an undo that ran out of history and an undo that finished both
answer *yes, I undid something*.

**Coalescing closes a run at a word break, not at a line break.** §11 gives the reason in the same
sentence as the ratio — *undoing a sentence is 414 presses instead of 1 012* — and a run that closed
only at a line would make undoing a sentence **one** press, which is a checkpoint and not a history.
Measured here, 1 050 keystrokes is **224 entries against 1 050**, a factor of **4.69** where §11
remembers 2.44 over its own corpus.

**`Ink` gains a fourth verb, `pad_to`.** A row of text is a partition of its width and never a prefix,
so a row-drawing component writes what the row says and then the space after it. Written as
`Ink::text` then `Ink::run` that is one allocation cheaper *and* it makes `verbs` a counter that
separates two builds by how full their rows happen to be — `run` returns without a verb at a count of
zero, so a row that exactly fills its width costs one verb and a row that does not costs two. On the
memo-key defect a stale index's rows are the whole line truncated, so they fill exactly and **`verbs`
reports the defective build as cheaper**. `pad_to` stages the row and its pad into the frame's own
buffer and blits once: one verb a row, no allocation, and the counter cannot see the defect any more.

## Consequences

**A caret's column is a screen column, re-seated on every row it lands in.** Found by the gate rather
than argued: a forward step that crossed a row boundary and kept accumulating reported column **39 on
a row whose start was 0**, because a greedy break consumes the space it broke at. Both `Right` and
every edit now read the column from the caret's own row start, which the index makes one row rather
than one prefix — §11's headline arriving on the *cheap* direction.

**`field` declares one region and never one per visible cluster.** Measured on a 60×8 field over the
cluster corpus: **1 against 372**, where §11 states 79 against 621. Both magnitudes are a prototype's
screen; what reproduces is the shape, and it is ADR 0027's own defect from the other side — there,
widgets merged into one id and the screen still rendered correctly; here, one widget declares hundreds
of entries and the screen still renders correctly. No counter that reads a cell can see either.

**The frame costs the visible window and never the content**, which is the engine's own invariant
arriving on the component that holds the most data in the freeze. A 300×80 field is **24 000 writes /
80 verbs / 1 region / 0 merges / 0 allocations at 100 kB and at 1 MB**, and the allocation zero holds
over fifty steady frames with the probe installed.

**The frame's µs is text measurement and not the component.** 300×80 of the cluster corpus is
**1 033–1 056 µs** and the same rectangle of ASCII is **428–450** — which is `tree`'s 436 inside the
noise. §11's
*82.4 µs* is a prototype's screen; what this one costs is 24 000 cells of graphemes measured over the
engine's tables, and the ASCII row is what says so.

**The caret is gated on `Ctx::is_focused` and not on `Response::focused`**, and the difference is one
frame: a `Response` carries the focus as it stood when the widget *declared*, so a field just clicked
would place no caret until the frame after. And a screen that seats the focus at the end of its own
draw has no caret on its first frame at all, which is why `document::play_field` plays **two**.

**The wrap index partitions what is drawn and not what is held**, which is the one finding here that
came from a review rather than from a gate. `layout::text::wrap` trims each piece, so a caret placed
by asking which drawn row *contains* its byte is lost by **one trailing space** — `Frame::caret` is
`None`, the terminal cursor disappears, and the screen is otherwise perfect. `Index::row_of` is the
answer for the draw and `Index::row_bound` for the walkers, and the second makes two subtractions a
click sweep found: a hard break is stripped, and a row followed by another stops one cluster short of
that row's start, because that byte is the next row's own column 0.

**A widget that declares the pointer and consumes nothing is worse than one that declares nothing.**
`field` carried `Interest::SCROLL` and read `Response::scrolled` nowhere, so the wheel was dead over a
textarea *and*, because the field is the topmost region over its rectangle, an enclosing `scroll_area`
never saw the notch either. There is no horizontal axis to be wrong on — both break rules wrap to the
rectangle's width — which is components ticket 20's *a body dead downward is alive sideways* answered
by construction rather than by arithmetic.

**A test that calls a `#[track_caller]` component from two call sites is two widgets**, and the caret
is the instrument that notices. Found while writing this ticket's own gates: the second frame minted a
different id, the id the first frame focused had not drawn, the vanish rule cleared the focus, and
`Ctx::caret_with` refused a caret — an absence, on a screen that looked right. ADR 0027's defect
arriving inside the gate for it.
