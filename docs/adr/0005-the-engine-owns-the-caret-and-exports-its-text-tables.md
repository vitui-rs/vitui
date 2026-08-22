---
status: accepted
date: 2026-08-17
---

# The engine owns the caret, and exports the text tables it already runs

Two additions to the engine's public surface, both found by writing a form against it:

- `Screen::set_cursor(Option<Cursor>)` — where the terminal's own caret goes, in screen coordinates,
  applied by `present` after the last write of the frame.
- Grapheme-cluster segmentation and display width, as pure functions over a `&str` the caller brings.

Both look like violations of lines this project has drawn. `set_cursor` is state that a component
owns, kept in the layer that is supposed to only draw. Segmentation is text processing, in a crate
whose invariant is that it never iterates a caller's data. Neither is what it looks like.

## Why the caret is the engine's

The caret is not a widget's property, it is a property of the terminal — one per screen, positioned
after everything else has been written, and it is the only cursor the machine has. A runtime cannot
place it because a runtime does not know when the last write happened; `present` does, and `present`
is the engine's.

The alternative was measured rather than assumed. A component drawing its own caret costs one damaged
cell, which is nothing, and **two wakeups a second for as long as anything has focus** — 5 wakeups in
3 s against 0 for an idle screen, 7 200 an hour. Ticket 09 measured a genuinely idle application at
`0.00 user, 0.00 sys` with zero wakeups and standing requirement 11 makes that a property rather than
a nicety; a text field is not an exotic component, and a design where opening a form ends the idle
state has lost the requirement in the ordinary case.

The terminal's caret blinks in the terminal's process, at the rate the user configured, and is the
only thing an assistive technology or an input method can follow. IME and screen readers are out of
scope for this map — the *hook* they will need is not, and it is this one.

## Why segmentation does not breach ADR 0002

The engine already segments: every `View::text` call walks the caller's string by grapheme cluster
using ticket 02's generated UCD tables, because a cell holds a cluster handle and not a `char`. What
is being added is not new machinery; it is the same walk, exposed.

The invariant it might have breached — *the engine never iterates application data* — is about
collections: the engine is never handed a `Vec` of rows and never loops over one. A `&str` the caller
passed in is not a collection the engine holds; it is an argument, and `text` already iterates it.

What refusing costs is not a smaller engine, it is a second copy of the tables. A runtime that
computes caret columns itself is wrong the moment its Unicode version differs from the engine's — and
wrong immediately with the obvious implementation: the caret column of one family emoji is **5 by the
engine's rule and 10 by `chars().count()`**, five columns of drift on one character. Cursor
desynchronisation is the failure mode ticket 02 exists to prevent, and leaving the tables inside the
crate while every text component needs them recreates it one layer up.

## Consequences

**The engine's public surface grows by two concepts and no signatures change.** Both additions are
new; nothing existing is touched, and `Style`, `View` and `LayerStack` are unaffected.

**`present` gains one job.** After the frame's bytes, it emits the caret position if it moved and
nothing if it did not. A frame that does not move the caret pays nothing.

**The runtime translates coordinates and decides who has focus.** A component reports a caret in its
own coordinates; the runtime knows the layer's rectangle because it brought it, adds them, and calls
`set_cursor` once per frame. Focus is not an engine concept and does not become one.

**Blinking is not configurable through this API.** The terminal blinks, or the user turned it off.
Anything that wants to control the blink is asking for a software caret and is asking to give up
requirement 11's idle; the shape choices in `Cursor` cover the legitimate part of that want.

**Neither addition is a text-shaping API.** Segmentation and width, not bidi, not line breaking, not
shaping. RTL is out of scope for this map and this ADR does not quietly let it in.

## Amended by implementation ticket 21

**"A frame that does not move the caret pays nothing" was nearly right, and the gap is the frame's own
writes.** A frame that emitted any cell has moved the terminal's cursor to wherever its last cell
was, so a visible caret has to be re-placed by that frame whether or not the application moved it —
there is nothing else that will, and the caret would otherwise sit at the end of the last run until
something happened. What survives of the sentence is the byte count rather than the write: the caret
is placed with the serializer's own shortest-move pricing, and in the case that matters — a character
typed into a field, with the caret after it — the move the caret needs is the move the write already
made, and it is **zero bytes**. §8's 29-byte caret frame is 29 bytes with a caret on it.

The exact statement is: a frame that emitted nothing and did not move the caret pays nothing, and
does not exist — `present` refuses it before it leases a packet. A frame that emitted cells pays for
the caret only where the caret is not already where the cursor ended up.

**`DECSCUSR` conflates shape with blink, so "the shape choices in `Cursor` cover the legitimate part
of that want" needed a fourth choice.** There is one escape for the caret's appearance and its
parameters pair each shape with a blink state — 1/3/5 blinking, 2/4/6 steady — so *choose a shape* and
*leave the blink to the terminal* are not two things the wire can say. `CursorShape` therefore names
the three blinking spellings and `Terminal`, which is `Ps = 0`: whatever the person at the terminal
configured, and the default. Steady variants are not offered, because an application asking for one
is overruling an answer that user already gave.
