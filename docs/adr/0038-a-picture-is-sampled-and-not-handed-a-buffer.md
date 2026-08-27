---
status: accepted
date: 2026-08-26
---

# A picture is sampled and not handed a buffer

`media::picture` takes a **`Pixels`** — a trait with one method, `fn pixel(&self, x: u16, sub_y: u32)
-> Rgb`, asked in the picture's own coordinates — and never a buffer, a width, a height or a scaling
mode. Decoding, cropping and scaling are the caller's, and they are the caller's because **not one of
them is a fact about a terminal**.

The consequence is the invariant, not a preference: a frame costs the **rectangle** and never the
image. A component handed a `&[Rgb]` with a size would have to decide what to do when the two
disagree, and every answer to that question is a scaling policy this crate has no business holding.

## Context

`CONTEXT.md`'s rule one layer down is *frame cost is proportional to visible cells, never to data
volume*, and ADR 0002's is *the engine never iterates application data*. Spec §14 puts a picture on
the wrong side of both by default: it is the one component whose content **is** its data, so unless
the seam is chosen deliberately the component ends up walking a megabyte to draw eighty rows.

`chart::raster` had already answered the same question the other way round and for the same reason —
the frame costs the rectangle and the fold costs the data, behind a memo — but a chart's data is
*points* and a picture's is *pixels*, and a pixel has no fold. There is nothing to memoise: a picture
reads exactly one sample per sub-cell it draws, every frame, and the number of sub-cells is the
rectangle.

Three shapes were available:

1. **`&[Rgb]` plus a size.** The component then owns scaling: nearest neighbour, box filter,
   letterbox, crop, or a `Fit` enum with all of them. Every one of those is a decision about
   *pictures* rather than about *terminals*, and the enum grows for ever.
2. **An `Image` type this crate owns.** Same problem with a constructor in front of it, plus a second
   rectangle-shaped type in a crate that already deleted one (`Cells`, components architecture issue
   17).
3. **A sampler.** The caller answers *what colour is here*, which is the only question a terminal
   renderer actually has.

## Decision

The third. `Pixels` is one method and a blanket `impl<T: Pixels + ?Sized> Pixels for &T`, and
`media::picture` is `fn(&mut Ctx, Rect, &P) -> Response` with `P: Pixels`.

Two things follow that a reader coming from spec §1 should be told rather than left to discover:

- **§1's `…` is a type parameter here**, and it has to be. `crate::picture::DECLARATIONS` therefore
  scans for `pub fn picture<P: Pixels>(` and not `pub fn picture(` — a scene green on the parenthesis
  needle would have been green **by deleting the type parameter**, which is the one thing about this
  signature that is load-bearing. Components ticket 26 met the same shape on `select`'s two lifetime
  annotations.
- **A translation of the picture is a translation of the source.** The component is never told where
  on the screen it landed, so there is no `offset` option and no need for one: `crate::picture`'s
  scroll trap folds its shift into the sampler, which is what a scroll *is*.

The same argument settles the QR: `media::Modules` is a matrix the caller brings, and this crate
ships **no encoder**. A payload, a version, an error-correction level and a mask are a specification,
and the only thing a terminal library owes a symbol is that its modules are square, two-valued and
positioned — which is `picture::module_aspect` and `picture::readback`.

## Consequences

- **A picture is a pure drawer.** It declares no interactive region, returns `Response::inert`, and
  costs a frame **zero** hit entries. So does every symbol and meter in the family; `player::chrome`
  is the one construction here that interacts.
- **Its census is one `Theme::custom` a cell** and no `fill` is available at any size, because a
  `fill` takes one paint for a rectangle and no two adjacent cells of a photograph share one — 0 of
  47 620, measured. The family is legible as that contrast: picture one a cell, QR 4, barcode 2, the
  audio three 0.
- **The frame is over budget and is recorded rather than optimised against.** 24 000 cells in 24 000
  verbs is §20's full-screen class and it is 1.33–1.50× over the 1 ms in it. The only thing that
  would move it is a verb that can carry two paints, which is an engine change nobody has asked for;
  `media_numbers` prints the figure beside the budget.
- **The id must be taken in the frame that carries `#[track_caller]`.** `Ctx::id` mints from
  `Location::caller()` and the attribute propagates only through functions that carry it, so an id
  taken in a private body one frame down gives **every picture in a program the same id**. Gated in
  both directions by `media::tests::two_pictures_at_two_call_sites_are_two_widgets`, because both
  ways to notice it are quiet: a picture declares no region, so nothing merges, and `Response::id` is
  a value a caller may key on rather than one the runtime checks.
- **A scaling request has one answer and it is a sentence**: write a `Pixels` that scales. That is a
  handful of lines in an application and it keeps the policy where the pictures are.

## Alternatives refused

**A buffer with a `Fit` enum.** Refused on the argument above and on a second one: the enum's arms
are not orthogonal to the colour ladder. A box filter at `ColorDepth::Ansi16` averages two colours
that quantise to the same one, which is work that changes nothing — and a component cannot know
that, because `Theme::custom` gives it no way to ask whether two colours differ on the wire.
`crate::picture::wire_differ` is the contrivance that answers it for the *census*, and it authors a
theme per colour pair; it is not something a draw path can call.

**A `Pixels` that returns a whole row.** It would let a caller hand over a slice and let the
component coalesce runs. Measured, there is nothing to coalesce: **0 of 47 620** adjacent pairs of a
photograph share a value and 3 520 of a gradient's do — 7.4%, which is the smoothest frame anything
could produce still needing its own paint for 92.6% of its adjacencies. A row-shaped method would be
a wider surface for a 7.4% case.

**Homing the family in the freeze.** `INVENTORY` has no media row and this ticket did not add one.
§14's own reduction is *no v1 component*: F11's sixteen survey entries reduce to constructions, and
`file_preview_pane` — the one row naming this family — is homed under F12. So `media::MEMBERS` is
empty and the module ships anyway, with
`inventory::tests::the_module_tree_and_the_families_column_agree` keeping the two statements one
statement.

## Where it was decided

Components ticket 30, `.scratch/vitui-components-impl/issues/30-media.md`. Spec §14, §17, §20.
