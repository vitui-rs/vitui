---
status: accepted
date: 2026-08-27
---

# A slider's step is an integer index, and there is no range slider

`slider` ships two decisions that are both about what it refuses to store.

**Its keyboard steps an integer index on a grid of `SliderOpts::steps` and divides once**, rather
than adding `1 / steps` to a `f32`. And **there is no range slider**, because two thumbs need one
fact the drag mechanism does not carry — *which* thumb — and the spelling that avoids storing it
moves two values with one gesture.

## The step is an integer because a `f32` step is wrong twice and each half hides the other

`SliderOpts::steps` is a **count** and never a step size. Measured from `0.0` on the default grid of
a hundred, on a 300-cell track:

| | grid | `f32` step |
|---|---|---|
| fifty `Right`s | `0.5` | **`0.4999998`** |
| — the thumb's cell | **150** | **149** |
| a hundred `Right`s | `1.0` | **`0.99999934`** |
| — the thumb's cell | 299 | 299 |
| a hundred up, a hundred down | `0.0` | **`1.4901161e-8`** |

**Neither row is visible where the other is.** In the middle the value looks right to every digit a
status row prints and the thumb is one cell short of centre. At the top the thumb is on exactly the
right cell and the value has never reached its own maximum — so `if v == 1.0` is a branch a slider
driven from the keyboard can never take, and nothing on the screen says so. That is why the value and
the thumb's cell are printed side by side in `mixer`'s status bar, and why the gate asserts both.

`input::defective::float_stepped` is kept runnable, and it is a **component** and not only a
function: `float_stepped_into` is `slider_into` with one field changed, so a reviewer's diff between
the correct build and the refused one is one value.

The cost of the grid is stated rather than hidden: **a keypress quantises a value a drag placed
between two steps.** A drag is *not* quantised — it reports where the pointer is, and snapping it
would make §14's own two fractions, `20/299` and `60/299`, unreachable. So the grid is the
keyboard's unit and not the value's.

## There is no range slider, because the fact it needs is the one §14 measured the absence of

§14's headline is that drag capture needs nothing added to the runtime: the value is
`Response::local` over `Response::rect`, with **no press origin, no stored anchor and no fifth
cross-frame fact**. Two thumbs on one track need a third answer neither field carries, and there are
exactly two ways to get it.

1. **Remember which thumb the press landed nearer.** This works, and it is precisely the fifth
   cross-frame fact — one `bool` that makes the range slider the only component here whose grab has
   state in it.
2. **Derive the boundary from the two values, so nothing is stored.** This is
   `input::defective::Range`, and it does not work: the boundary moves with the values it separates,
   so a pointer crossing it mid-gesture **abandons the thumb it was dragging and yanks the other
   one**. From `(0.2, 0.8)` with the pointer going `0.3 → 0.6`, the low thumb goes to `0.3` and then
   the *high* thumb jumps `0.8 → 0.6` — a move of `0.2` on a thumb nobody touched, in a drag that
   never released.

**Two thumbs as two widgets is not a third answer.** `Response::local` is in the coordinates of the
rectangle it was declared with, so a one-cell thumb's own `local` says nothing about the track; and
two overlapping regions resolve the pointer to the topmost one, so a thumb and its track cannot both
have it.

Which of the two a range slider should cost is left to the ticket that builds one. Neither is a new
*mechanism* — one is a stored `bool` and the other is arithmetic — and what is settled here is that
the shipped `slider` reads two fields and a second thumb is not free. `mixer`'s bottom pair is arm 2
on the screen, so the jump is a drag away rather than a paragraph.

## What the component does reach for, and what it does not

**The bar is `scroll::bar`'s rule reached, not restated.** The verbs are `scroll::stripe` — one
definition of *write `n` cells along an axis*, one place the orientation branch lives — and the order
is thumb-first, so no cell is written twice. `bar` itself is **not** called, for two reasons that are
both counts: a bar has two parts and a slider has three, so `BarOpts`'s single `track` role cannot
express the picture; and a `Span` is three counts of *content cells*, which a slider has none of.
Inventing an extent out of the track's own width to reach a helper is the shape §9's unit rule exists
to refuse. So the *answer* is shared instead, swept against `scroll::thumb` at
`Span { viewport: 1, extent: length, offset: at }` over every track length from 1 to 200.

**`nav::step` is not reached, and the disagreement is a count.** That helper pairs `Up` with `Left`,
because it moves an index into a list and a list's index grows downward. A slider's value grows
*upward*: `Up` is more, at both orientations. The two therefore agree at `Left`, `Right`, `Home` and
`End` and disagree at `Up`, `Down`, `PageUp` and `PageDown` — **4 of the 8 cursor codes**. A slider
that called it would be right for four keys and silently backwards for four, on a screen where the
thumb visibly moves either way. It is components ticket 17's finding from the other side, where a
container could *not* read `←` and `→` through `nav::step` because that helper reads them as `↑`/`↓`.

The two refusals `nav::step` exists for are restated rather than inherited: a release is not a
gesture — on a terminal that reports event types every arrow arrives twice — and a chord is not a
cursor key, which is `keys::is_chord` and the same `CTRL | ALT` mask with Shift deliberately outside
it.

**`Interest::SCROLL` is not declared.** §17's row reads *a slider's value is not an offset and no
wheel event moves it*, and components ticket 20's finding is why the absence has to be gated: a
widget that declares the wheel and consumes nothing is **worse** than one that declares nothing,
because it is the topmost region over its rectangle and an enclosing `scroll_area` never sees the
notch either.

## The value is a fraction, and the unit is the caller's

`slider` takes `&mut f32` in `0.0..=1.0`. It owns no minimum and no maximum, because a component that
did would own a *unit* — and deciding what a number means is the one thing no layer of this library
does for its caller. `mixer` prints decibels under every fader; that conversion is the application's
one line.

The `f32` is also the whole of the state: **4 bytes, and no slot beside it.** `collapsible`'s pair is
4 live and 48 with the slot (ADR 0035); a slider has nowhere to put a fifth fact and that is the
point of the type.

## Consequences

- `INVENTORY`'s `slider` row keeps `tier: Three` and `built: true` with its `MOVED` entry, and the
  `built` column is now **joined against the source**: `every_built_row_is_declared_in_the_module_that_homes_it`,
  19 of 19 and 0 of the 10 unbuilt rows. That gate exists because this row had been `built: true`
  since components ticket 30 with no `slider` anywhere in the crate, and neither existing join could
  see it — a mechanism being built is not the component being built.
- `glyphs` gains `VLine`: a vertical slider's groove is a column, and `HLine` stacked downward is a
  picture of a dashed line.
- `scroll::stripe` and a new `scroll::band` are `pub(crate)`, and `bar`'s own thumb rectangle now
  comes from `band` — one arithmetic rather than two.
- A range slider is **not** on the backlog. Reopen this ADR rather than adding one, and say which of
  the two mechanisms it is paying for.
