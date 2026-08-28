---
status: accepted
date: 2026-08-28
---

# A keyboard contract is declared data, and *registered* is what the machine answers

Components ticket 38 discharges spec §17's obligation O4 — *a declared keyboard contract, rendered as
help* — and it is the third of the five to turn green, after O1 (ticket 36) and O3 (ticket 37).

O4's gate is an equality: **documented == registered**. One direction is a help bar that lies and the
other is a feature nobody can find, and neither is visible from the other side. Three decisions the
discharge forced are recorded here, and the first is the one everything else hangs off.

## `registered` runs the shipped component; a second reading of the drain loop would agree for ever

`obligations::o4`'s own header already records what it cost to find out that **two empty lists satisfy
an equality**: written over the freeze with both evidence lists empty, O4 returned `Met` over
twenty-nine rows and was the one query of six that read green. *An equality between two things that
do not exist holds.*

The same trap is one level down and it is quieter, because both sides are populated. A `registered`
list derived from `CONTRACTS` and a `documented` list derived from `CONTRACTS` agree about every chord,
for ever, whatever either says about the component. So `Contract::live` **runs the widget**: it posts a
trigger at a headless driver with the component's own id holding the focus, draws a frame, and reads
`Driver::unhandled` — *the keys this frame's batch carried that nobody took*. There is no
per-component knowledge in the observable and no second transcription of a drain loop to drift away
from the first.

A pointer gesture has no such window, because a click is never handed back. Its observable is the
**picture**: the same drive twice, once with the modifiers and once without, compared cell for cell.
A component that never reads `Response::mods` draws the same cells either way, which is the answer for
every row that declares no pointer gesture — and it is an answer the sweep derives rather than one the
declaration asserts.

**`CORPUS` is what makes it an equality rather than a subset.** A hundred and sixty-two triggers —
sixteen codes at five modifier states, twenty-six letters at the three chord states, the printable-text
class and the three modified clicks — so a component that takes a chord nobody declared is caught by the
same sweep that catches a declaration nothing honours.

### The control arm, and why it is a component of the freeze

`Driver::unhandled` answers *nobody took it*, and **the runtime is one of the takers**. `Tab` and
`BackTab` at all five modifier states are the focus walk and never reach a component, so read without a
subtraction all thirteen contracts register ten chords they have never heard of — and the equality would
have been reconciled by *declaring* them, thirteen components each claiming a binding on
`Ctrl+Shift+Tab`.

The control is `button`, and that it is a row of the freeze rather than a fixture is half the decision:
what the control measures is *what a focusable gets for free*, and the sentence it makes checkable is
**`button` is a tab stop that reads no key**. §21's *`Tab` inside a trap* is answered from the other
side by the same number: it is the runtime's key and no component's.

## `Bind::ignores` is spec §3's rule declared, not a convenience

`keys::SIGNIFICANT` is `CTRL | ALT` and **Shift is deliberately not in it**, because a capital is what
Shift is for. A component that filters a key through `keys::is_chord` and then matches on `code`
therefore answers `Shift+X` exactly as it answers `X` — by construction — so the sweep finds two chords
where a help bar should print one line. `Bind::ignores` is that fact, declared: the equality expands a
bind to its trigger *and* its trigger with those modifiers held, and `help` prints the one line.

It is **not a licence to widen a bind until the sweep agrees**. `collection` declares `Up` and
`Shift+Up` as two separate binds, because there the second is a different action; a row that ignored
Shift on the arrows would be claiming the selection never extends. The distinction is per bind and the
gate is the sweep, which sees both spellings either way.

## A contract is read at the configuration where it is whole, and what the other one removes is a number

Two components have a contract that moves with their options, and a sweep at the wrong one reports the
component as broken rather than as configured.

- **`field` is one flag.** At `WrapKind::Ruler` a one-row `input` has no row to step to and its `Enter`
  is the caller's submit, so `Up`, `Down` and `Enter` are declined — ADR 0042 as six spellings
  (`RULER_REMOVES`). The declaration is read at `Words`, where the contract is whole.
- **`collection` is `Mode`, and the direction is the finding.** At `Mode::Single` — the default —
  `apply` answers `Plain` and `Toggle` with the *same call*, so a ctrl-click **is** a plain click, leaves
  the same picture and reads deaf; and it refuses `Extend` outright, so the two extends leave a
  *different* picture and read fluent. **The gesture that disappears is the one that works.** The
  contract is read at `Mode::Multi`, which is what §5 writes its pointer sentence about.
- **`select` is two keyboards rather than two configurations**, and it is the third instance. Shut,
  the owner's drain loop runs; open, *the popup takes the keyboard from its owner* and what answers is
  §5's collection at `Mode::Single` plus the popup's own `Enter` and `Esc`. The contract is the
  **union**, because a help bar is about a component and not about one of its states — three of its
  lines name both meanings, since one spelling does two things across the two seats. The sweep needs a
  **third frame** for this: the handover happens during the frame after the one the focus was planted
  on, so a two-frame drive measures the owner and calls the popup deaf.

The alternative — declaring the union and letting three binds read dead at one configuration — makes the
equality unusable as a gate, because a dead bind would then be normal.

## What this ADR does not decide

**Whether a help bar may show a binding as unavailable.** It is inherited from the runtime spec as a
components question and is still open; `Bind` carries no enabled flag, and nothing in ticket 38 answers
it.

## What it found

Four chord leaks in code that was already green, none of them reachable from register row 5 — *a chord
pressed into every focusable types nothing* — because **that row is a claim about a buffer**. Moving a
caret, opening a list and clearing a selection all type nothing; what these did was *take the key*, so
the application's accelerator never arrived.

| where | the chord | what it did |
|---|---|---|
| `input::field` | `Ctrl+Left`, `Ctrl+Right`, `Ctrl+Backspace`, `Ctrl+Delete`, and the `Alt` set | moved the caret **one cluster** |
| `input::select` | `Ctrl+Down`, `Alt+Enter`, `Ctrl+Space`, … | opened the list |
| `files::file_picker` | the same four keys | opened the list |
| `collect::from_key` | `Ctrl+Esc`, `Alt+Esc`, `Alt+Space` | cleared the selection, toggled a row |

A fifth is in `input::popup_body`'s own `Refusal` — `Ctrl+Enter` committed and `Alt+Esc` dismissed —
and it is the one with nowhere else to go, because that loop runs *inside* a trapless overlay where the
application has no other reader for the key it just lost.

`Ctrl+Left` is the sharpest of them: it is exactly what a user pressing for **word motion** means,
and word motion is `contract::ABSENT`'s one row — the engine exports `graphemes()` and `width_of()` and
no word iterator — so the widget was swallowing the accelerator *and* answering it with a cluster. Every
one is one `keys::is_chord` guard, and `collect::defective::from_key_on_code_alone` keeps the spelling
they replaced runnable.

**And the family turned out not to be one keyboard.** `select`'s popup takes the focus and reads
`Enter` and `Esc`; `file_picker`'s does neither — it draws a plain `collection_into`, seats nothing and
declares no refusal — so an **open `file_picker` can only be used with a mouse**: no arrows, no
`Home`/`End`, no type-ahead, no way to choose a file. **35 spellings against 8 over one family**, on a
screen that renders perfectly. It is not repaired here, because seating a focus and minting a refusal
inside `picker_body` is a component's keyboard being *designed* — three decisions belong to the ticket
that owns `files.rs` — and transcribing `select`'s three lines would be components 32's own finding in
mirror image. `contract::PICKER_IS_MISSING` is 27 and is asserted exactly; components architecture
issue 23 carries the question.

A sixth was found in the gate rather than in the code, and it is ADR 0031's own rule arriving in an
instrument: `Order::built` stamps `Revision::fresh()`, and a collection handed a revision it has not
seen **clears the selection** — so a probe that rebuilt its index inside the draw reported `tree` deaf
to all three pointer gestures, for the same reason a caller who rebuilds its index every frame has no
selection.
