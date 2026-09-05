---
status: accepted
date: 2026-08-29
---

# O7 is a join by path, over a population read out of the source

Components ticket 45 adds a **seventh obligation** beside spec §17's five and ADR 0049's sixth:
*every component this crate declares is exercised by at least one application in
`crates/vitui-apps/examples/`.* Like O6 it is stated after the architecture map closed, and like O6
it is an implementation ticket rather than an architecture issue for one reason: **the instrument is
buildable without reopening anything.**

Four decisions are recorded here: why the obligation exists at all, how the join is made, which rows
it is asked about, and what one hand-written list is allowed to go on being.

## The hole, stated once

> A gate exercises the component where its author put it, and an application puts it somewhere else.

`crates/vitui-apps/src/lib.rs` already carried the claim — *this crate is checked by being a
consumer* — and there was nothing that made it true of any particular component. The argument for
closing that gap is four defects, not a preference:

| application | what it found | how the gate missed it |
|---|---|---|
| `counter` | **no loop could be written at all** — `Driver` owned its `Screen` privately and `attach` dropped the `WakeHandle`, so the only shape available was a spin at 100% of a core (runtime architecture issue 23) | every test builds its own `Engine`; none of them writes an application's `while` |
| `counter` | **nothing holds the focus until an application says so** — 0 of 5 arrow presses, presenting as *the terminal lost focus* (architecture issue 25) | a one-widget test seats the focus by clicking |
| `latency` | **`chart`'s rasteriser painted the whole column prefix for every point** — 952.61 ms against 2.72 at a million (ADR 0049's whole subject) | every counter §20 has is an *output* counter, and the cost was in work that produces none |
| `ledger` | **a table drew its header one column into the border** when handed a panel's interior (register row 113) | every gate in the crate plays at `x == 0`, where the header's coordinates and the body's agree |

Four defects, four different reasons, one shape. **Three of the four were found by a person running
the thing**, which is the strongest form of the argument and the least reproducible.

## The join is by path, and a bare-name scan is wrong twice over

Both false positives were already in the tree when the obligation was written:
`crates/vitui-components/src/keys.rs` declares a `pub fn text(` that is not the `text` component, and
`gates::table()` prints the gate register. `keys.rs`'s own
`every_text_bearing_name_is_in_the_freeze` documents the second by name and works around it with a
`(cx` suffix — **a heuristic, not a join**, and one that will pass again when the next collision has
a different shape.

So `crate::consumer::imports` reads the **import statement**. `use vitui_components::<module>::<item>`
is a `(module, item)` pair; the freeze already homes every row through `Component::module`; and
`keys::text` is `("keys", "text")` where the component is `("text", "text")`. The two never meet, and
both collisions are watched *not* counting.

Two things make this a real reading of *what an application draws* rather than a spelling check.
First, `[workspace.lints.rust] warnings = "deny"` makes an unused import a build failure — **an
import is proof of use**. Second, a `use` is a *statement* and not a line, which is runtime issue
22's finding: its first scan reported *the engine is unreachable* about a crate re-exporting all of
it.

### Three rows share one machine, and the exception is named, counted and necessary

Spec §1 gives every component three spellings, and `input`'s three toggles share the last two:
`checkbox`, `radio` and `switch` are `toggle_with` and `toggle_into` with one field of `ToggleOpts`
between them. **`toggle_into` with `Toggle::Radio` *is* `radio`'s ink spelling**, and an application
that counts its own cells has no other way to draw one. `crate::consumer::SHARED` is that exception
as three rows with the item that names each, and its own test asserts it is *still necessary*: the
day `checkbox_into` ships, striking the row is the edit, not keeping the count at three.

The disambiguator is the kind, which is why the reverse direction of the `uses` check skips the
machine: one path for three rows cannot say *this entry implies this component*.

## The population is read out of the source, and that is the third reading of one question

O1 and O3 are asked over `built`, because a page or a screen for a function that does not exist is
not a thing anybody can write. O6 derives its population from the `Layer::L2` column and the memo
census. **O7 opens each row's home module and looks for the declaration**, and the reason is
components ticket 33: the `built` column read `true` for `slider` through two tickets with **no
`slider` anywhere in the crate**, and the two joins that look as though they should have caught it
were each blind for a stated reason.

The population **moves**. `spinner` is the one row this crate does not declare, so it is absent from
the population rather than red inside it — *a query stuck red is `Verdict::of`'s vacuity failure in
mirror image*, and the day components 46 ships the ladder O7 asks about twenty-nine with no edit.

## `App::uses` stays, and stops being load-bearing

The column is a hand-written list of strings, and a list a human maintains is exactly what this
register exists to replace: it would go stale in the direction that reads as green, and it is what
somebody choosing which application to open reads. Components 39's review found the sharp instance —
the gallery's row listed `gallery::PANELS`, which `crate::gallery`'s own scan **forbids** that file
from spelling, so the column documented an application doing exactly what a gate one crate over
refuses, unchecked for fifteen applications.

It is not deleted, because the prose is worth having. It is **joined**:
`vitui_apps::tests::the_uses_column_agrees_with_the_scan` runs both directions over forty-nine
`(application, component)` pairs, and the count is asserted because two empty lists agree about
everything.

## What it was owed, and what closed it

Three rows: `scrollbar`, `sticky` and `file_picker`. The first two are what a caller assembles when
it **owns the offset itself** — which is precisely the case `scroll_area` is not — and until this
ticket nothing in this workspace had called either of them from outside the component that homes
them. `crates/vitui-apps/examples/sheet.rs` is the application, and what it makes visible is the
four things `scroll_area` does for nothing: the clamp, the tail, the wheel and the reveal. The last
of those it **cannot** buy back — `Ctx::request_into_view` addresses the widget that owns the
offset, and here that is an application — and the picker on the same screen reproduces components
architecture issue 23 by being pressed: an open picker had no keyboard at all. (Issue 23 resolved
2026-09-05 and it has one now; this application is where a person found that it did not, which is
the claim this ADR is making about consumers.)

## Consequences

- `crate::consumer` is the seventh obligation's instrument, and the only one whose evidence is in
  another crate.
- Register rows 230 and 231; row 30's gate reads *O1-O7*; `EVALUATED` 216 → 218.
- `crates/vitui-apps/src/lib.rs`'s header states the rule in one sentence: **a component ticket
  ships an application.**
- Eight of the nine obligation queries are met. O5 is the one left, and §17 says it is worth more
  than the other four together.
