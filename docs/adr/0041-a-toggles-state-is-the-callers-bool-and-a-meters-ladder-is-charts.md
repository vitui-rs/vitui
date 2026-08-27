---
status: accepted
date: 2026-08-27
---

# A toggle's state is the caller's `bool`, and a meter shares `chart`'s ladder but not its spelling

Components ticket 34 ships six Tier 2 components — `checkbox`, `radio`, `switch`, `meter`,
`sparkline` and `rule` — and Tier 2's whole claim is that each is *composed of mechanisms a Tier 1
ticket already measured*. Three of the six decisions that claim forced are recorded here, because
each is easy to undo by a later edit that looks like an improvement.

## The three toggles are one machine and three configurations

`checkbox`, `radio` and `switch` are `input::toggle_into` with one field of `ToggleOpts` between
them. Nothing here holds state at all: a toggle's whole cross-frame fact is the caller's
`&mut bool`, which spec §1's rule 2 names in the same sentence it writes the signature in — *a
`&mut bool` is the widget's own value, never application data*.

**A radio *set* is not this row.** It is `collect::collection` at `Mode::Options` — *exactly one, and
it can never become zero* — which is §5's collapse. `INVENTORY`'s `radio` row carries no composition
edge for that reason: the row is the widget and the set is a different call, and a component here
that minted a second selection store would be undoing the collapse that makes the freeze
twenty-nine components rather than thirty-five.

The backlog spells that mode `Mode::Radio` and §5 shipped it as `Mode::Options`. **The disagreement
is recorded rather than resolved by renaming**: a mode called `Radio` would be §5's thirteen match
arms wearing one component's name.

## The mark is one glyph and one absence, and the switch has neither

The backlog's sentence reads *`state::press` plus a `Glyph` pair plus a `Role`*, and the freeze's
`glyphs` column — a value, and therefore the authority — gives `checkbox` exactly `Glyph::Tick`,
`radio` exactly `Glyph::Bullet`, and `switch` **nothing at all**.

There is no pair to draw, because the *off* half of a toggle cannot be a glyph: `CONTEXT.md` defines
a glyph as a lookup with **no spelling blank**, and off is blank. So each mark is one glyph and one
absence, and the absence is painted in the face like every other cell of the widget.

**`switch`'s empty column is a fact rather than a hole**, and it is the one place §16's axis question
has a third answer. A checkbox and a radio carry their state on the **glyph** axis — `✓` becomes `x`
and `•` becomes `*` at `GlyphSet::Ascii`, both still one cell, both still present — and differ in
exactly **one** cell between on and off at every rung. A switch carries its state on **three** axes
and only one of them is the palette: the two words of `ToggleOpts::words`, the side its knob sits on,
and the face. It differs in **three** cells at every rung, and none of them came out of a glyph
table.

## A meter shares `chart`'s prefix ladder and cannot share its spelling

`meter` is *`chart`'s prefix construction at 2 rungs*, and the half that is shared is the **ladder**:
`chart::raster::geom(Kind::Bars, set).sy` is `1 / 8 / 8`, which is where the freeze's
`constructions: 2` comes from. Nothing in `indicate` decides how many sub-cells a rung offers, and
nothing there names a repertoire.

The half that cannot be shared is the **spelling**, and the reason is a fact about Unicode rather
than about this crate. `chart`'s bars grow **upward**, so its partial cell is one of U+2581…U+2588 —
one contiguous run, and `chart::raster::cluster` is the lookup. A horizontal meter grows
**rightward**, and the left eighth blocks are a *different* contiguous run: U+258F…U+2588, running
the other way. The two agree at nothing filled and at everything filled and at **0 of the 7 partial
eighths** in between.

So a meter's **vertical** arm reaches `cluster` and its horizontal arm reads `indicate::LEFT8`. A
component that had transcribed `chart`'s table onto the wrong axis would draw a bar growing upward
inside a row, and every counter in this crate would report it correct — which is why the
disagreement is the assertion (register row 193) rather than the agreement.

## The spaces around a `rule`'s caption are the caller's

`rule` is `text::fit`'s four skippable parts with a `Glyph` where the padding was — the lead run, the
head of the caption, the one-cell ellipsis and the trail run, each skipped when it is empty, which is
what keeps `verbs` at **1 / 2 / 3 / 2** for an empty, a leading, a centred and a trailing caption.

The caller writes its own spaces — `rule(cx, at, " Section ")`, exactly as `panel(cx, at, " panel
systems ")` does — and that is priced rather than preferred. The spelling that spaced them inside the
component needed a `String`, which is **2 allocations a frame and 120 over 60**; register row 200's
window caught it on its first run, and no other gate here could, because the picture is identical
either way.

## What this costs, and the one thing it does not buy

Six rows of the freeze move from `built: false` to `built: true`, and all six need an entry in
`INVENTORY::MOVED`, because `nothing_is_built_at_a_tier_that_says_otherwise_without_the_row_saying_so`
reads any row that is not Tier 1 as claiming *not built*. That reading is right for Tier 3 — *at
risk, naming an unmeasured mechanism* — and wrong for Tier 2, whose definition is *composed of proved
mechanisms* and says nothing about whether anybody has written it yet.

**Recorded rather than bent.** Promoting six rows into Tier 1 would rewrite §17's freeze table, which
is a value a test counts, and the `MOVED` mechanism already exists for exactly this disagreement.
When ticket 35 lands, the list will be nine of nine, at which point it is *every row that is not Tier
1 and is built* — which is what the column already says.
