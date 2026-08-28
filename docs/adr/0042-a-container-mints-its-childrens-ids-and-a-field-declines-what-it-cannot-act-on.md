---
status: accepted
date: 2026-08-27
---

# A container mints its children's ids, and a field declines a cursor key it cannot act on

Components ticket 35 ships the last three Tier 2 rows — `status_bar`, `pagination` and `form` — and
Tier 2's claim is that each is *composed of mechanisms a Tier 1 ticket already measured*. Spec §18
calls R3, the composition class, **the class that requires the most care**, because *composition
without a new mechanism is exactly the claim that turns out to be false when it is false*. Three
decisions that claim forced are recorded here; each is easy to undo by a later edit that looks like
an improvement, and one of them is a repair to a component that was already green.

## A container that moves the focus inside itself has to mint its children's ids

`Ctx::id` mints from `Location::caller()` (ADR 0013), so there is no way to ask *what id would row 7
have* — the only way to find out is to draw row 7 and read `Response::id` back. A form cannot: the
key that moves its cursor is handed back by the focused **field** and arrives at `Ctx::scope`'s
after-the-body moment, which is one row too late to be the row the key names.

So `form_into` mints them: every row is `Id::keyed(form_id, row)`, hung under the form's own id,
which is ADR 0027's rule and is exactly `collect::Cell::id`'s answer one component over — spec §4's
*every workaround that looks like a hack is the id being opaque*. `input::field_keyed` is the
crate-private seam that takes one; `field_into` is that with `cx.id()`, so the shipped signature does
not change.

The alternatives were both refused on measurements rather than on taste. **Storing the ids** is a
`Vec<Id>` keyed by row — ADR 0028's *per-row state is one slot, never a map*, and an allocation on
the draw path. **Seating the focus next frame** costs a wake nothing produces: `Ctx::focus` already
takes effect at the next frame's routing, so a form that recorded the row instead would be one whole
frame later on the *screen* and would need a `request_frame` to get there at all.

One consequence is public and is stated on it: `input::form_row_id` is `pub`, because spec §8's
*nothing holds the focus until an application says so* (architecture issue 25) means a program
opening on a form has to be able to name its first field. It takes a `Response::id` read back from
the frame that just drew, so identity still comes from the call site and nothing is persisted.

## A field declines a cursor key it could not act on

This is the defect the ticket found in code that was already green, and it is components ticket 20's
*declared and consumed nothing* arriving on the keyboard axis.

§11's one flag makes `input` and `textarea` one component, so `field` reads `Up` and `Down` as a
caret row step — and a one-row `input`, which is most fields anybody writes, has no row to step to.
It consumed the key anyway. Nothing about the field looks wrong: the caret is where it was, the
screen is correct, and every counter agrees. What it costs is one layer up — a `form` is a `Group`
whose `nav::cursor` **never sees an arrow**, so a documented key does nothing and the only gate that
can see it reads *the focus did not move* rather than *a cell is wrong*.

The answer is the caret's own position and not a flag on the kind: `moved = st.caret() != before`.
The same line is right at the top of a textarea, at the bottom of one, and in a single-row input, and
a rule written on `WrapKind` would have to be re-derived at each of them.

## The labels arrive as their own slice, because `nav::cursor` takes one

`form` takes `labels: &[&str]` and `texts: &mut [Text]` as two parallel slices rather than one slice
of records. The record shape reads better and is what a reviewer expects; it is refused on a count.
`crate::nav::cursor` takes `&[&str]`, so a form over records has to **build** that slice every frame,
and a `Vec<&str>` a frame is one allocation a frame against spec §20's budget of zero. The picture,
the writes and the verbs are identical either way, so the allocation window is the only instrument in
the workspace that can tell the two apart — which is why the refused shape ships as
`input::defective::form_collecting_labels` and is watched paying **60 over 60** in the same run that
measures the shipped 0.

The same discipline caught a defect in `pagination` on that gate's first run: a page's label spelled
`(i + 1).to_string()` is one allocation a page a frame, **9 240 over 60 frames** at 137 pages. It is
`collect::Digits` on the stack now, and the type-ahead searches those rather than a built slice.

## What is *not* decided here

Whether a form should be one tab stop is spec §3's, not this ticket's: *`nav::cursor`'s placement is
the decision, not its contents.* `FormOpts::group` defaults to `true` for that reason, and the
ungrouped arm is kept and measured because it is not merely a form without a scope — a container
hears what its children hand back **only** through a scope's after-the-body moment, and `ScopeKind`
has three arms of which the other two are a modal's `Trap` and the code editor's `Isolated`. A form
that is not one tab stop is a form with no `nav::cursor` at all, and that is a fact about the runtime
rather than a preference.
