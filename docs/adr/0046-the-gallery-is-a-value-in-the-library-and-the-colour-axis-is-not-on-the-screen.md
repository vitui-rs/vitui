---
status: accepted
date: 2026-08-28
---

# The gallery is a value in the library, and the colour axis is not on the screen

Components ticket 39 discharges spec §17's obligation O2 — *a panel in the gallery binary* — and it is
the fourth of the five to turn green, after O1 (ticket 36), O3 (ticket 37) and O4 (ticket 38). It is
also the implementation of `tickets/002`, the user's own requirement for *one binary that shows every
shipped component and switches themes on `t`*.

Three decisions are recorded here. The first is where the screen lives, which decided everything else;
the second is what `'f` costs a table of function pointers; the third is a fact about the colour axis
that the ticket's own criterion states the other way round.

## The screen is a value in `vitui-components`, and the application is a driver

O2's gate is two equalities over a panel list, and the obvious home for a gallery is the application
crate — that is where the other fourteen applications live, and `crates/vitui-apps` exists precisely
so that every one of them is a standing proof that the component-facing surface is sufficient.

**It cannot live there, and the reason is not tidiness.** Spec §21 names two defects to be measured
**on the assembled gallery** and pins them as register rows 7 and 8:

> the assembled gallery, twelve panels, 53 280 cells — 9 956 cells nobody writes
> a theme swap over the gallery — six panels keep the old palette permanently

Both are components tickets — 40 and 41 — and a components gate is `cargo test`. `crates/vitui-apps`
is a crate of `examples/`, so nothing in `vitui-components` can reach into it: **a screen only an
application can draw is a screen no gate can measure.** So `crate::gallery` carries the panel table,
the twenty-eight drawings, the tile grid and the state, and `crates/vitui-apps/examples/gallery.rs`
is the loop, the keys and the two reports.

That split is the same one `crate::golden` already makes for O3 and `crate::contract` for O4, and it
buys the same thing: the evidence is a value a test iterates rather than a screen somebody looked at.

**The direction it opens is closed by a negative scan.** An application free to list its own panels
drifts from the freeze where no equality over the table is looking, so
`crates/vitui-apps/examples/gallery.rs` may not spell `Panel {`, a second `PANELS`, or any component's
own call — a scan for what a file must **not** contain, which cannot be satisfied by deleting
anything. `crate::preview::mints_its_own_task` is the precedent.

**The wrapper-list criterion is met by not needing the exception.** The ticket asks for the gallery to
join ADR 0001's wrapper list *because it uses crossterm for raw mode, the alternate screen and input*.
It uses none of that: `Driver::attach` enters raw mode and the alternate screen, reads the input, and
restores both — the panic hook included. `deny.toml`'s `{ name = "crossterm", wrappers =
["vitui-engine"] }` is unchanged, and a gate asserts that the line is still exactly that, because the
gallery is the reason it did not have to move.

## `'f` costs the table one field, and `Option::take` is the only way to pay it

The panel table is twenty-eight `fn` pointers over one state bag. Two of the twenty-eight own an
overlay — `input::select` and `files::file_picker` — and both take `&'f mut` of the state their popup
body captures, because a body outlives the base pass (spec §1's sentence about the fifth component).

A `fn` pointer cannot hand that over. Its elided lifetimes are fresh and unrelated, so a reborrow out
of the bag is shorter than `'f` whatever the caller writes; and tying the bag's borrow to `'f` in the
signature makes the **first** panel's borrow last the whole frame, so the second one does not compile.

So the two `'f`-bound borrows are split off the bag once, at the top of the view, and arrive as a pair
of `Option`s a panel **takes**. That is components 26's own answer to the same borrow — *a nested
overlay's state has to arrive as an `Option` the body takes, because an overlay body is `FnMut` and
cannot move a capture while a reborrow is shorter than `'f`* — and here the `take` is load-bearing
twice: it is the one operation that yields a `'f` borrow through a shorter one, and a second take
panics, which is correct because a panel drawn twice in one frame is two widgets under one id.

## `Theme::resolve` returns the same paint at every depth, so the colour axis is not on the screen

§16's matrix is *a count, not nine screenshots*, and register row 45 filed it `Unreachable { needs:
"`ColorDepth`" }` with runtime architecture issue 22 as its inverter — **the issue that lifted the
barrier**. `crates/vitui-runtime/src/line.rs` files `ColorDepth` as `reachable_as:
Some("vitui_runtime::ColorDepth")`, so the row had been passing while meaning the opposite: exactly the
decayed `Barrier` citation that issue's own header warns a `Barrier` invites.

Both axes are values this crate can hold now, and the nine cells are taken over the assembled gallery.
The repertoire axis is on the screen: 58, 82 and 110 distinct clusters at Ascii, Unicode and Extended.
**The colour axis is not, and it cannot be.** `Theme::resolve` returns the **same `Paint`** for all
thirteen roles at all four depths — measured, over `Themes::standard()` — because quantisation is the
engine's and happens before the mirror. A component is handed the palette's own colour whatever the
terminal can show, which is ADR 0018 (*a component names a role and never a colour*) working rather
than a hole in it.

So the colour half of the matrix is `Theme::roles_differ_on_wire` and `Theme::shows`: 66, 9 and 0
collapsed role pairs of 78, and 3, 2 and 0 distinctions lost of 10 (**of 9** since components
architecture 25 struck `Distinction::Guide`; none of the three figures was carried by it). The
screen's own paint count is
reported beside them and reads **ten in all nine cells** — a column that moves on neither axis, which
is §21's refinement 1 as a table: *a counter on the wrong side of the question is not a weak gate, it
is a green one.*

**On the surface it is a zero, and that is the sharpest form of it.** A colour-depth change over the
assembled gallery moves **10 cells at 300×80 and 5 at 100×30, and 0 of them on any panel**: the cells
that move are the heading and the status bar printing the depth's own name. `Swap::changed_on_a_panel`
is that number, and it had to exclude **both** chrome rows rather than one — at 100×30 the status
bar's left segment is elided before it reaches the depth, so excluding the heading alone read 0 there
and 5 at 300×80, which reads as a panel moving. *One size agreeing with a law the other breaks is how
a gate over one size stays green.*

**The criterion's own sentence does not reproduce.** *`Danger`, `Warn` and `Ok` all quantise to bright
white at sixteen colours* is true of **eight of the fourteen** shipped schemes — six keep the three
distinct — and of **fourteen of fourteen** at `ColorDepth::None`. It is the right claim about the
wrong rung, and it is asserted as measured rather than bent to fit; the palette was deliberately not
swapped to make the old sentence true, which is the same choice components ticket 05 made about C09's
figures.

## What follows from these

- `crate::gallery::PANELS` is twenty-eight rows over the freeze's `built` column, so the population
  moves on its own the day `spinner` ships. O1 and O3 both chose `built` for the same reason and O2's
  second equality was already written over it in the spec.
- `crate::obligations::PANELS` stays a written-out list, compared against `gallery::panel_ids()`. Two
  lists a test compares is `crate::doc`'s arrangement for O1 and `crate::contract`'s for O4: an
  equality between two things derived from each other holds for ever.
- Register rows 7 and 8 stay red. This ticket delivers the screen; `gallery::shape` and
  `gallery::swap` print both numbers as measured on it — 10 252 unwritten cells of 24 000 at 300×80,
  and a swap that keeps 0 of 13 748 paints, 64.6% of clusters under a rung change and 99.9% under a
  tier change — so components 40 and 41 start from a figure taken here rather than from a prototype's.
- The budget is measured **in** the gallery: ~390 µs at 300×80 against the 1 ms full-screen class,
  ~79 µs at 100×30 against the 100 µs typical frame, and **zero allocations on every page**. The last
  found one defect — two preview drawers spelling their row labels with `format!`, 4 allocations a
  frame on page three and zero on every other page — which is components 30's `player::chrome`
  finding a second time.
- `t` costs ~70 ns, which is the whole argument for keeping the key: if *resolved at construction*
  quietly meant *at start-up*, it would stutter. The figure that does not reproduce is
  `with_glyphs` before `resolve` — ~550 ns against the criterion's 291 — because a declared repertoire
  is a real input to the ten distinction bits (ADR 0032) and re-narrowing them is the work.
- A file in this crate may not spell `GlyphSet::`, and the gallery is the third caller
  `chart::raster::RUNGS` was made public for: *a test that sweeps the ladder, a report that prints it
  and a caller that wants the richest rung* are all this module at once.
