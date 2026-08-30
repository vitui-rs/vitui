---
status: accepted
date: 2026-08-30
---

# `SHIFT` is intent for a key and is not intent for a character

`Chord::key('+')` never fired on a terminal that reports modifiers, and every application binding
`?` for help or `:` for a command line was walking into the same wall. The decision is that
**`On::Typed` compares five modifiers and `On::BaseLayout` compares six**, and the sixth is `SHIFT`.

## The two things `SHIFT` means

`keys::INTENT` is the six engine modifier bits that are intent rather than keyboard state, and
matching consults it through `Mods::chord()`. That mask is correct for a named key: `Shift+Tab` is a
different binding from `Tab`, and both `keys::SIGNIFICANT` and `nav::step`'s *shift passes* rule are
built on it being correct.

It is wrong for a character. `+`, `?`, `:` and `_` **cannot be typed on a US layout without shift**,
so on a character the bit is not a modifier the author asked for — it is *how the character was
produced*, and the author asking for the character has already asked for it. `Chord` carried
`Mods::NONE` for `Chord::key('+')`, the terminal reported `SHIFT`, and the two were unequal before
anything looked at which key it was.

`On::BaseLayout` / `On::Typed` was already in the type and is a **different axis**: it says *which
half of the key* is compared, not *which modifiers matter*. The decision joins the two axes at one
point — `On::Typed` **and** a `KeyCode::Char` — and nowhere else.

## Why not the one-line version

Masking `SHIFT` wherever a chord's code is a character is one line and was refused. It makes
`Chord::key('a').shift()` silently equal to `Chord::key('a')`, which is a binding an author writes on
purpose and can no longer write. `shift_still_means_shift_on_a_base_layout_chord` is the gate that
fails on that repair, and it is in register row 43 beside the positive one for that reason: the row
is *what fires* and *what still does not*, because a fix for a silence is exactly the kind that
over-reaches.

The narrow decision has its own cost and it is stated rather than discovered:
**`Chord::typed(c).shift()` is `Chord::typed(c)`**. It is acceptable where the broad one was not
because of reachability — nobody has a reason to write it, and the author who wants shift to mean
something still has `key`. `shift_on_a_typed_chord_is_a_no_op_and_is_recorded_as_one` asserts the
equality so that the day it stops holding, it says so.

## One keystroke has four spellings and a chord can be right about three

| what the terminal sends | `code` | `mods` | `text` | what matches |
|---|---|---|---|---|
| the legacy byte | `+` | — | `+` | `typed('+')`, `key('+')` |
| `CSI 43;2u` | `+` | SHIFT | `+` | `typed('+')` |
| `CSI 61;2;43u` | `=` | SHIFT | `+` | `typed('+')` |
| `CSI 61;2u` | `=` | SHIFT | `=` | `key('=').shift()` |

**The fourth is not a matching defect and is not repairable here.** At kitty flag 1 alone the
terminal reports the *unshifted* key and sends no associated text, so nothing in the process knows a
`+` was produced. A runtime that inferred one would be reading a keyboard layout ADR 0021 refuses to
read. It stays an alternate the author binds — one, where the workaround in `latency.rs` cost two of
the three `MAX_ALTS` slots for one logical key.

## What it cost the layer below

`On::Typed` had **no positive test in its own crate**, and could not have one: `KeyText` had no
public constructor, so there was no way to build the key a typed chord matches. The recorded reason
for the privacy was that *nothing outside should be able to forge a key whose `code` and `text`
disagree* — and that reason was never true of this engine. A terminal at kitty flag 4 puts the base
layout in `code` while flag 16 puts what the key produced in `text`; `CSI 61;2;43u` is `Char('=')`
with `"+"`, and the two disagreeing **is the design**. What the private fields actually protect is
the overflow sentinel, which a `char` cannot reach.

So `KeyText::of(char)` is public — total by arithmetic, four bytes into twenty-four, no `Option` and
no failure mode. No `&str` constructor joins it: a multi-scalar cluster is a dead key, a conjunct or
an IME commit, none of which any binding compares against, so the only thing one could build is text
nothing can match, and it would carry the overflow case this one does not have.

This is the **first item on the engine's public surface added by a ticket from the layer above**, and
the engine's own audit had no vocabulary for it: `everything_outside_spec_12s_block_names_the_ticket_that_added_it`
admitted `impl NN` and `production NN` and nothing else. The prefix list was widened rather than the
citation bent, for the third time and the same reason — *a destination gate that admits only one
backlog is a gate that asks the next ticket to lie about where it came from*. It will not be the
last: this crate is implementation-complete and its consumers are not.

## Consequences

- `keys::TYPED_INTENT` joins `keys::INTENT` as a named mask. Two names for two masks, which is what
  two meanings of one bit costs.
- `Chord::typed` is the constructor to reach for on a character shift produces, and its doc says so.
  `Chord::key` on such a character matches only the legacy spelling.
- `MatchMode::Equality` is untouched. It exists to *lose* bindings — it is how *first match wins is
  not an equality* stays runnable — and a bit exempted from it would blunt the one gate it is there
  to fail.
- Register row 43, spec §9 gains *six modifiers are intent for a key, and five are intent for a
  character*, and the engine's public function count moves to one hundred and seven.

## Status

Accepted. Runtime architecture issue 28, the third defect on that map with one shape: **correct on
the configuration everything was tested on and wrong on the capable one**, after the scroll-scope
sign (issue 26) and the greedy wrap (issue 27). Invisible on a legacy terminal, which is why it
shipped, and why a US-layout test suite on a terminal at flag 1 finds none of the family.
