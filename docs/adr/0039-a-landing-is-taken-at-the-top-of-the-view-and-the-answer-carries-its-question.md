---
status: accepted
date: 2026-08-27
---

# A landing is taken at the top of the view, and the answer carries its own question

`file_preview_pane` does not take its own landing. The caller calls **`PaneState::land(&task)`**
before anything draws, and the component only reads what is already in the state. And an answer that
lands is checked against **`Preview::shows`** — eight bytes of identity on the payload — before a
cell is written.

Both halves are one decision with one cause: **`Task::take` is destructive and a view has no `&mut`
with which to put back what it took.**

## The tear needs no thread at all

A view that asks for the answer wherever it happens to want it hands the *first* consumer the answer
and the second `None`, **on the same frame**, with nothing having been concurrent. `crate::preview`'s
screen has a status row above the pane and one below it, which is §15's own arrangement, and a
mid-draw take is **20 torn frames of 20 selections against 0**, read off the drawn surface.

It leaves nothing behind. The state ends up correct either way, so a torn frame is invisible to every
counter that reads state and visible only to a screen that prints one value twice.

Making the take a verb on the state rather than a step inside the component is what makes the
shipped shape unwritable-wrong: there is exactly one consumer, and it runs before the frame.
`files::Taken::InsideTheDraw` is kept as the refused spelling so the number stays live.

## The second equality is the pane's, and no test on the answer can reach it

`Task` already drops a landing whose `Generation` is not the newest — that is an answer to a question
**nobody is asking any more**, and it is the runtime's half. The pane's half is the other one: an
answer to a question **nobody ever asked**, which arrives with a perfectly current generation because
the job was started for the right question and computed a different one.

§15 states why an ordering test cannot see it, and the sentence is exact:

> a question that was never asked is invisible to any such test.

An ordering test asserts that answers arrive in the order they were asked for. A wrong answer to the
right question is not out of order. Measured: 1 answered / 1 refused / 0 landings / **0 content
cells** against 1 / 0 / 1 / 2 516.

## What it costs, and what it does not

`Preview` is two methods, and neither is a burden the caller did not already carry: an application
that can decode a file can say which file it decoded, and an application that can decode a file knows
how large the result is. `extent` is the same field §9 needs anyway — a declared content size — and
having it on the *answer* is what makes a preview pane a positive case for bars-reserved rather than
a new question for it.

`land` returns a `Landed`, so a caller that wants to spin a spinner or start a fade has the edge.
There is no `is_pending` beside it, for `Task::take`'s own reason: a predicate followed by a take is
two reads with a worker running between them, so the check *is* the take.

## `file_picker` takes the landing, and that is the rule rather than an exception to it

`picker_body` opens with `body.pane.land(task)`, which reads like the thing refused below. It is not:
the picker **is** the view for its own overlay, and the body is the top of it. The rule is *one
consumer, before anything draws*, not *no component may ever call it*.

What makes that hold rather than merely sound right is a field: `PickerBody::pane` is **private**,
with a read-only `pane()` accessor and no `&mut` twin. A caller cannot reach a second `land` on the
one screen where a second reader of one value is the whole defect.

## What was refused

**A component that takes its own landing.** It is one line shorter at every call site and it makes
the tear unfixable from outside: the caller cannot get the value before the component runs, so any
second reader of it is reading a different version.

**A landing checked only by the runtime.** The generation is necessary and not sufficient, and the
gap between them is a class of defect — a decode handed the wrong path, a cache returning a
neighbour's entry, a batch reply mis-indexed — that no amount of ordering discipline reaches.

**Storing the identity in the pane rather than on the payload.** It would make the equality a
comparison between two things the pane owns, which is a comparison of a value with itself as soon as
the pane's copy is written from the answer.

## Consequences

- A caller owes one line before its view: `st.land(&task)`. Forgetting it is not a crash — it is a
  pane that never shows anything, which the `landings` counter says out loud.
- The application, not the component, decides where in the frame the landing happens. That is the
  same shape `CONTEXT.md` already takes about offsets: the caller owns the sequence.
- `PaneState::refused` is a counter and not a panic, because an answer to an unasked question is a
  defect in somebody else's decode and the pane's job is to keep the screen right while it happens.
