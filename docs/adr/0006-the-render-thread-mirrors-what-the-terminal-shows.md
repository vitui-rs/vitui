---
status: accepted
date: 2026-08-17
---

# The render thread mirrors what the terminal shows

The frame serializer holds one screen-sized buffer of `Cell` — 384 KiB at 300×80 — recording what the
terminal is currently showing. It is the only screen-sized state anywhere on the render thread, and it
is written by the serializer as a side effect of emitting.

It is called a **mirror** and not a shadow, because `CONTEXT.md` already spends "shadow" on the layer
that darkens what lies beneath it — and ticket 14 spent it a third time on a sub-cell component's own
bitmap. Three meanings for one word in one crate is one too many.

This directly contradicts a property ticket 09 established and put in writing: *the render thread
holds no screen-sized state at all, which is what makes resize, shutdown and panic small.*

## Why

Two of the three wins in ticket 08 are the same buffer seen twice.

**The equality filter.** Damage says which cells a frame *wrote*; the mirror says which of them
actually *changed*. Ticket 14 measured the gap and handed it over: a caret blink damages forty cells
and changes one; a progress bar advancing one percent rewrites a hundred and changes one. With real
bytes on the wire the filter is worth 3.0x on a caret blink, 30.8x on a progress tick, 9.1x on three
dialogs with a live status bar, and 4.7x on a sub-cell chart. It costs 0.9 ns per damaged cell,
because the comparison rides inside a scan that was already reading every cell to coalesce style runs.

**The scroll region.** `DECSTBM` + `SU` turns a one-row list scroll from 1 726 bytes into 60. Deciding
that a scroll happened requires comparing this frame against the last one, and *proving* it requires
comparing whole rows — including columns this frame never touched. Both are the mirror.

Neither can live on the app thread instead. The comparison itself is affordable there, but the app
thread is the one under budget pressure — ticket 14 measured its worst realistic frame at ~205 µs
against a 1 ms ceiling and a 100 µs damage-tracked one — while the render thread has 16.6 ms of wall
clock and nothing else to do with it. The whole serializer, filter and scroll pre-pass included, is
116 µs on the worst realistic frame: 0.7% of a frame interval.

## Consequences

**Ticket 09's reason for the property survives; the property does not.** What that sentence was
protecting was the smallness of resize, shutdown and panic, and the mirror does not threaten any of
them. Resize reallocates it and marks every row unknown, which forces the full repaint a resize needs
anyway. Shutdown and panic are memory being dropped. The invariant that actually carries ticket 09's
design — *the mailbox lock is never held across a syscall* — is untouched, because the mirror is
private to the render thread and never crosses the boundary.

**The mirror is authoritative about the terminal, not about the frame.** It is updated when bytes are
emitted, so anything that writes to the tty behind the serializer's back desynchronises it. Two rules
follow and both are cheap: rows are marked *unknown* at startup and after a resize, and a row marked
unknown is emitted whole rather than filtered.

**A scroll must be proved, not guessed.** It is tempting to let the filter behind the pre-pass repair
a wrong guess, and that is wrong: the filter can only emit cells the packet carries, and the packet
carries damaged cells only. `SU` moves every column of the region, including untouched ones. So the
guess is verified against the whole band before a byte is emitted — and one candidate is verified, not
every candidate that matches the probe, because a screen with repeating rows turned a 38 µs frame into
1.03 ms.

**Memory is 384 KiB against a surface's 375 KiB.** It doubles what the engine holds per screen. That
is the price and it is stated here so it is not rediscovered as a regression.

## Amendment — 2026-08-21, engine impl 14

**Unknown is per cell, not per row.** This document said *rows are marked unknown at startup and after
a resize, and a row marked unknown is emitted whole rather than filtered*, and engine impl 08 built
exactly that: a flag per row, set when one frame had written every column of it. Impl 14 built the
filter the flag exists for, measured it over spec §14's twelve scenes, and found the filter **worth
nothing at all on eleven of them** — a row at three hundred columns is almost never written whole by
one frame, because a dialog is sixty columns wide and a list draws its rows and not the gutter beside
them, so every row stayed unknown for ever.

Nothing above changes except the granularity. The property is the same sentence with a smaller subject:
*never compare against what the mirror does not know*, and a cell becomes known when the serializer
emits it.

It costs no bitset and no branch, because the unknown state is a **value** rather than a flag: a cell
whose grapheme is the `EMPTY` sentinel, which no composited frame cell can hold — the frame is opaque
so its ground is a blank, and a non-opaque layer's `EMPTY` cells are skipped rather than copied. So the
comparison the filter already makes is false wherever the mirror does not know, and the false
*equality* this document warns about is unreachable by construction rather than by remembering to
consult a flag.

Two consequences: invalidating the whole mirror is now a 384 KiB fill rather than an eighty-byte one,
which is a fraction of the one full frame spec §3 already prices a renumbering sweep at; and *row*
survives as a question asked of the cells, for the scroll region, which records an exposed row whole.

## Amendment — 2026-08-21, engine impl 15

**The scroll region is not the reader the amendment above predicted, and it is the reader of something
else.** That amendment kept `is_known` as a row query on the strength of two callers, one of them *the
scroll region, which records an exposed row whole*. Impl 15 built it and half of that is right: an
exposed row **is** recorded whole, in `Mirror::scroll`, because `SU` has no horizontal margins and the
terminal therefore erases every column of what it exposes. What is wrong is that the scroll region
**asks** the row question. It does not, and it must not:

- **Obligation 1**, over the rows a scroll moves, needs per-cell knowledge for the same reason the
  filter does. What the frame wants in an undamaged column is what the mirror holds there, so the
  question is asked through `known_cell` per column, fused into the comparison the obligation was
  already making. A row query over the band would be a second pass over a screen for a fact the first
  pass already carries.
- **Obligation 2**, over the rows a scroll exposes, would be *made wrong* by it. `is_known` over the
  row demands that the columns the frame **repaints** be known too, and the obligation does not: the
  terminal is about to erase them and the frame is about to write them. On the scene this optimisation
  exists for, the frame repaints every column of the exposed row — so a row query there forfeits
  exactly the scroll it is guarding. That is the amendment above arriving a second time, from the
  other side: *the row is the wrong granularity for a screen no single frame writes whole.*

So the row question survives with a different shipping reader, and it is a better one because it is an
**invariant rather than a precondition**. After a verified scroll the mirror knows *every row of the
band*: obligation 1 held over every column of every row moved, so each destination row inherits
knowledge rather than a hole, and each exposed row is a blank the terminal wrote. That is what keeps
the *next* frame's filter sound, it is not trivially true — a narrower obligation 1 would break it
without breaking any other test — and it is asked as a `debug_assert!` in `Serializer::emit_scroll`.
`Screen::known_rows` remains the other reader, gating `Packet::repaint`.

Nothing above changes. The mirror is still per-cell, unknown is still a value rather than a flag, and
the sentence this document is named for is untouched.

**And the ordering the scroll rests on is now checked rather than argued.** The paragraph above says a
terminal erases what it exposes with the *current* background and that the frame has already reset SGR
by then, so background-colour-erase and erase-to-default agree. The terminal model erases with the
style it is actually holding, so a serializer that ever emitted `SU` under a live SGR would produce a
cell the mirror records as blank and the model records as coloured, and the round trip would fail. The
argument did not need a capability query; it does need an instrument, and it has one.
