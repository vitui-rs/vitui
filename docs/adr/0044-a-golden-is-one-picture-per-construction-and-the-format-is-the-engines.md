---
status: accepted
date: 2026-08-27
---

# A golden is one picture per construction, the format is the engine's, and the legend has a ceiling

Components ticket 37 discharges spec §17's obligation O3 — *one golden screen per construction, not
per matrix cell* — and it is the second of the five to turn green, after O1. A golden is **the one
instrument on this map that catches a wrong cell**: O1 catches an API that cannot be called from
outside the crate and O2 catches an inventory that has drifted from what ships, and §17 says in as
many words that neither of them looks at a picture.

Four decisions the discharge forced are recorded here. Three are easy to undo by an edit that looks
like a tidy-up; the fourth is a measurement that will read as an arbitrary size to anybody who has
not met it.

## 1. O3's population is `built`, which is ADR 0043's decision arriving a second time

§17 states O2's second equality over `built` and states **O1's count and O3's count over nothing at
all**, so the reading was owed twice. `spinner` is the one unbuilt row of the twenty-nine — *a
component that owns a clock*, which is components 42's prototype and no ticket on this backlog
ships — and **a golden of a function that does not exist is not a screen anybody can draw**. Asking
for one would put a permanent row in the failing set that nothing here can invert.

*A query stuck red is `Verdict::of`'s vacuity failure in mirror image*: it reads red whatever
happens, so nobody reads it, and the day it goes green for an unrelated reason nobody notices that
either.

The population **moves**. `o3` filters `INVENTORY` by `built` rather than naming twenty-eight rows,
so the day `spinner` ships the query asks about twenty-nine rows and thirty-four screens with no
edit. Today it asks about twenty-eight and thirty-three.

## 2. A screen is one picture and one file, whatever it takes to reach it

The engine's format is one file per *frame*. Two of the twenty-four painters here play more than one
frame — `file_preview_pane` because the answer arrives on a frame after the one that asked for it
(spec §15) — and their goldens are still one file each, because **what O3 counts is a construction
and a cadence is not one**. A second file would be a picture of a pane with nothing in it, which is a
picture but not the thing being counted.

Beside it: `select` and `file_picker` are photographed **shut**, and that is a fact about the overlay
family rather than a choice about those two screens. `Ctx::overlay` takes a `move |cx|` body and no
ink — a body outlives the base pass, so everything it touches is borrowed for the frame (§1's
sentence about the fifth component, components 26's `'f`), and a `&mut I` is not — so **a popup's
interior cannot reach a `Pen` at all**. A golden of an open `select` is four rows of *a cell no verb
wrote* under one row of face: a picture that passes and says nothing. `overlay`'s own screen does
show an interior, because `overlay_into`'s body **does** take the ink; it is a shell and not a layer.

## 3. There is no second format, and a path is what keeps that true

`crates/vitui-engine/src/golden.rs` is `pub(crate)` throughout — a cell, a handle and a style bit
are unreadable from outside the engine (ADR 0023), the same barrier that makes `marked`
`Reading::Unreachable`. So the code cannot be shared. **What must not be forked is the format**, and
`golden::FORMAT_OWNER` names the file by path while `FORMAT_PROPERTIES` carries the owner's four
load-bearing sentences, read back out of that file against a flattened source. A second format
cannot appear unnoticed, because the day one does the sentence it was supposed to inherit is no
longer where this points. `VITUI_BLESS=1` is the owner's command and there is no second switch.

Three things this side has that the owner's does not, each because the surface is different:

- **A cell nobody wrote is `▪` and it is not a blank.** The engine composites over an opaque base,
  so an unpainted cell is a finding; here it is the *subject*, since §2's partition rule is exactly
  the claim that a component leaves none of its rectangle untouched.
- **The style plane names a `Role`.** ADR 0018: a component names a role and never a colour, and a
  `Paint` is opaque here — so the legend resolves each paint against the theme's thirteen roles.
  That is *more* legible than a colour triple: the thing a reviewer wants to know about a component's
  cell is which role it asked for. Paints outside the thirteen are `custom #n`, numbered in
  first-appearance order, because a chart and a picture have two of them and two identical legend
  lines are the one thing a legend exists to prevent.
- **The failure report carries cells and rows** beside the owner's first-differing-line. Not a
  second format — the same file compared twice — and it is the form every defect on this map was
  legible in: *6 662 cells over 80 rows* is a whole screen and *6 662 over 9* is a band.

  **It is a report and never an equality, and that was a review finding in this ticket's own
  instrument.** A plane's key is `GLYPH_KEYS[i]` in first-appearance order, so a plane encodes the
  *pattern* of distinct clusters and not the clusters: two screens whose non-ASCII cells are swapped
  one for one have identical planes. A count over them reports `(0, 0)` for *a bar chart that started
  spelling itself differently at `Extended`*, which is the one regression the equality half exists to
  catch — and on this ticket's own subject it under-reports by nearly half, **12 against 21** on the
  plot's two block rungs. The cell truth is `Canvas::diff`; every equality and every figure runs over
  that, and both numbers are asserted so neither note can go stale.

## 4. The legend has twenty-six keys and braille has 256 states a cell

`plot` at `Extended` is **the one construction on this map whose alphabet can outgrow the format**,
and that is why its screen is 12x4 where every other charting screen is 24x6. The glyph plane refuses
a twenty-seventh key because *more than that on one screen is not reviewable by eye* — the owner's
rule, and not a limit to route around by widening the list. Measured: the same plot needs **41** keys
at 24x6 and **20** at 12x4, while the two rungs below it stay inside at 24x6 (**16** at Unicode,
**0** at Ascii). The ceiling is braille's and not charting's.

**This is the only place on the map where the format decides how big a screen may be**, and it is
gated with both numbers rather than left as a size, so that a change to the rasteriser which pushed
the small screen over fails with a count instead of inside the renderer with *not reviewable by eye*.

## What the discharge measured, against what the map remembers

Two of the three figures do not reproduce and are asserted as measured beside the number they are
not — this map's standing practice:

| | measured here | what the map says |
|---|---|---|
| `plot`, Unicode against Extended | **21 cells over 3 rows** | §17: 882 cells |
| the dense screen at 300x80, Ascii against Unicode | **1 065 cells over 78 of 80 rows** | §16: 7 276 over 80 of 80 |
| every screen at Ascii, clusters outside printable ASCII | **0 of 33** | §16's glyph column, as a picture |

The row count is why the second is worth carrying: **two rows of that screen carry no glyph at all**,
which a cell count alone reports as *the repertoire is everywhere*. And the dense screen keeps
exactly **one** non-ASCII cell at the ASCII rung — the em dash of `dense::HEADER`, `"vitui —
components"` — which is a fixture's own title and no component's glyph. That is §16's *text is
legitimately a component's own content and there is nothing to forbid* arriving as a number: 1 cell
of 24 000, at (6, 0).

## The rung is named in five files and none of them is `src/`

§16's replacement for the type `Paint` was able to be is a count — `GlyphSet::` occurrences in
`vitui-components/src` == 0 — and a screen table that could not say which rung it was at would be a
table with nine of its thirty-three screens missing. So `golden::Rung` is three arms with no
spelling, no table and no ordering claim in it, and the join to the runtime's repertoire is one
`match` in `tests/golden.rs` and a two-line copy of it in `examples/golden_numbers.rs`. The copy is
deliberate: an example is not a test, and importing one from the other would make the report a
dependency of the gate. `tests/glyph_matrix.rs`'s exception list goes three → five.

## Consequences

- `crate::golden` is the format, the screen table and the play; `tests/golden.rs` is the sweep and
  the equalities; `examples/golden_numbers.rs` is the report.
- `obligations::GOLDENS` is filled and `o3` is `Met` over 28. Two of the five obligations are green.
- Register 212 → **215 rows, 199 evaluated**.
- Regenerate with `VITUI_BLESS=1 cargo test -p vitui-components golden` and review the git diff. A
  stale golden is a failure, not a warning, and blessing in CI is refused.
