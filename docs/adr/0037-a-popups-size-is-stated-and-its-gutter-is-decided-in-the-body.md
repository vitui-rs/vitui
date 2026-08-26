---
status: accepted
date: 2026-08-26
---

# A popup's size is stated, and its gutter is decided in the body

A popup's **size** comes from a sizing function beside the component — `overlay::popup_size`, which
takes the data and the room and no draw context — and never from what the popup drew. Its **gutter**
is decided inside the body, from the size the runtime granted, by `overlay::gutter`, in **zero
passes**. Neither decision is available to the owner, and that is the point: the owner asks and the
runtime answers, so the owner does not know what it got.

The whole family is **three kinds on two axes** (`overlay::FAMILY`) over §12's eight entries, and
**modality is one `bool` on the request** — `OverlayOpts::scrim` being `Some` — which forces no
construction at all.

## Context

Spec §12 states the rectangle as **three parties**: the anchor is the component's, in its own
coordinates, during its own call; the size is the component's too, from a sizing function; the
placement is the runtime's `place`. Two of the three have somewhere obvious to live. The size does
not, and both of the obvious answers are wrong in ways nothing on the screen shows.

**A drawn extent cannot be the size.** A popup has no frame before the one it opens on, so the extent
it can read is zero; granted zero rows it draws nothing; having drawn nothing its extent is zero
again. Measured over two frames on both a 12-row and a 3-row screen: granted **(20, 0)**, on every
frame, for ever, where the sizing function grants (20, 4) and (20, 3). One frame is not enough to see
it — a single cold frame at zero looks like a warm-up. `CONTEXT.md`'s *one frame old* is survivable
for a scroll area and fatal for an overlay.

**Sizing to the content instead of to the room loses a row, and only one.** `place` clamps a position
and **never a size** — an overlay bigger than its bounds is flush with the near edge and hangs off the
far one, because a component told it has forty columns and given twelve draws a lie. So a popup that
asks for four rows on a three-row screen gets four rows, one of them off the bottom; its body then
sees four rows for four options, decides *no bar*, and the fourth row is reachable by nothing. **1 of
4**, with the screen otherwise perfect and every counter agreeing.

**§9's fixpoint does not arise here at all.** §9's bar decision iterates because the axis a bar
*reports* and the axis it *costs* are perpendicular: raising the vertical bar takes a column, which
can raise the horizontal one, which takes a row. A popup's content is as wide as the viewport it was
granted, because its labels truncate to it — so reserving a column narrows the labels and changes no
row count, and there is no second axis to couple through. Measured against §9's own `decide` over the
same two numbers: **0 passes against 3**.

## Decision

**Four rules, and each of them is a place rather than a policy.**

1. **The size is a sizing function's.** `overlay::popup_size(options, within) -> (u16, u16)`, taking
   the data the component takes plus the room it is about to be given. No draw context: it cannot
   draw, cannot claim an identity and cannot route, which is the whole of what makes it a function
   rather than a method on a trait. `select` widens the result to at least its own rectangle — a
   dropdown narrower than the widget it dropped from reads as a different widget — and that widening
   is the *component's*, because a sizing function never sees the anchor.
2. **The gutter is the body's.** `overlay::gutter(granted, rows)`, one comparison, `passes: 0`. It is
   in the body because the body is the only thing that knows what was granted.
3. **The shell is one function and the kinds are its branches.** `overlay::overlay` declares the blur
   position, reserves the bar, puts down the barrier, opens the trap, and writes **not one cell of
   the interior** — the interior and the bar tile the rectangle exactly, swept over every size from
   1×1 to 17×17 under five extents.
4. **Two structs, one writer each.** `input::SelectState` is the owner's, written only by the owner
   from its own input and the inbox; `overlay::PopupState` is the body's, borrowed `&'f mut` and
   written only by the body. The choice comes home through the inbox.

**`&'f mut` is the mechanism and not an annotation.** It makes *request the overlay last* a borrow
error rather than a comment: the queue holds the body until the satisfy pass, so the borrow lives
exactly that long, and a caller that reads its own popup state after handing it over fails with
`E0502` — pointing at the caller's own read and **never mentioning the overlay**. That is spec §1's
trap, met one crate up from where §1 recorded it, and it is kept by a `compile_fail` pair whose twin
names `select` and `PopupState` by path.

**Blur is qualified by a position, and the position is `Response::local`.** Not `Response::hovered`:
`hovered` is `hover_guess`, resolved from the **previous** frame's hit index, and the frame that
matters is the one the layer was placed on — the frame `begin`'s optimistic focus arrives on. A blur
qualified on `hovered` reads false there and dismisses the popup out from under a pointer standing on
it. `local` is this frame's containment in the widget's own coordinates and is exact on the popup's
first frame.

**The barrier and the trap are two verbs.** Each is gated with the other standing: a press at a widget
under a modal that still has its trap does not reach the base pass, and with the barrier removed it
does; six `Tab`s at a modal that still has its barrier do not leave, and with the trap removed they do.

**A dialog has no blur position**, and that is narrower than Axis A. A dialog declares plenty — its
buttons — and a hit entry over its own rectangle would be one nothing can read, because its barrier
already withholds the pointer from everything outside it and a modal is not dismissed by blur.

## Consequences

**§12's menu delta does not reproduce, and §5 is why.** §12 records a menu with its submenu as
**+6/+6** regions and stops — three rows twice, each a target of its own. §5 collapses a menu into a
`Mode` of `collection`, and a collection declares **one** hit entry however many rows it has, so each
level is a dropdown's delta: one entry for its rows and one for its blur position. **+4/+2.** It is
the same subtraction `popup::PER_ROW_ENTRIES` prices for a dropdown, arriving on the construction
§12's own prototype spent per row. Reproducing the six would mean declaring per row on purpose, which
is the defect `popup::Config::PerRow` exists to be.

**A component cannot share an overlay owner without sharing its own identity.** A `select`'s id *is*
its overlay's owner, and an `Id` is a hash — nothing recovers the rooting from the value. Two
`select`s under one id are one layer, one widget and **3 regions against 6**: the second request is
inert *and* the second widget's hit entry is merged away. There is no spelling that shares the layer
and not the widget, which is why a second overlay from one component mints (`popup::SUBMENU_OWNER`,
`input::CATCHER_KEY`) rather than reusing.

**A nested overlay's state has to arrive as an `Option` the body takes.** An overlay body is `FnMut`,
so it **cannot move a capture** — and a nested request has to move a `&'f mut` into the inner closure.
A reborrow is shorter than `'f` and fails the `+ 'f` bound. `Option::take` mutates the capture instead
of moving out of it, which is the one spelling that compiles, and the state itself stays in the
caller's own storage across frames. `popup::Held::submenu` is that, with the reason on the field.

**The scan that decides whether a scene has its subject had the wrong needle**, and spec §1 said so
before the component existed. It read `pub fn select(`; §1 records that *the fifth component opens an
overlay, and `'f` costs it two annotations*, so the shipped signature is `pub fn select<'f>(` and the
scan answered *undeclared* about a component that was right there. A scene that had gone green on the
old needle would have gone green by deleting the lifetime, which is a different component.

**A doc sentence owed on another crate's item is checked by opening that crate's file.** §1 says
`Ctx::overlay`'s documentation owes *the body answers through the inbox; a `&'f mut` capture compiles
and costs you the state for the rest of the frame*, and components ticket 10 could not write it. It is
written, and the instrument is a scan of `crates/vitui-runtime/src/ctx.rs` — a doc comment is not an
item, so nothing a compiler can be asked about changes when it is deleted, and a `compile_fail` cannot
see one either. The phrases are matched against a *flattened* source, because in the shipped file one
of them is broken across a line by the formatter.

**§12's catcher-layer bytes are recorded and not reproduced.** 386 912 against 2 592 is a layer's cell
count times a cell width, and the engine publishes no cell width to anything above it (ADR 0023). What
is measurable is the ratio in cells — a catcher covers the screen and a blur position covers the popup
— and the behaviour: a catcher **swallows** the press it exists to report, so dismissing costs a second
click.

**The unreachable-row figure is arithmetic and says so.** No cell of a composited surface is readable
from outside the engine, so *the row was drawn where the screen is not* has no observable form at this
layer. What is arithmetic over `place`, `popup_size`, `gutter` and `CollState::max_offset` is which
rows a keyboard can put on screen; its two behavioural halves — the granted height and whether a bar
stands — are asserted beside it and both move between the arms.
