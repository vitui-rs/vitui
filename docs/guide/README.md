# Building components on vitui

This is the manual for writing your own components against `vitui-engine` and `vitui-runtime`.

`vitui-components` is not privileged. It is one consumer of the runtime's surface, written under the
same rules as anything you write, and every component in it is a plain function you could have
written yourself. If something here needed a name only that crate exports, this guide would be
wrong — so the worked example deliberately depends on `vitui-runtime` and nothing else.

## The claim, in one paragraph

A component is **a function that takes a draw context and a rectangle, writes every cell of that
rectangle exactly once, and returns what happened**. It owns no state, registers nothing, is not an
object, and does not exist between frames. Everything that has to survive a frame belongs to
whoever is calling. That single decision is what removes the retained tree, the diffing pass, the
lifecycle, and the class of bug where the screen and the model disagree.

## Reading order

| # | Chapter | What it settles |
|---|---|---|
| 1 | [The three layers](01-the-three-layers.md) | What each crate owns, and which one you should be writing against |
| 2 | [The frame](02-the-frame.md) | The loop, the passes inside one frame, and where your code runs |
| 3 | [Your first component](03-your-first-component.md) | The signature, the three spellings, the partition rule |
| 4 | [Identity](04-identity.md) | Where an id comes from, and the two ways a loop breaks one |
| 5 | [Interaction](05-interaction.md) | Interest, `Response`, hover, press, drag, focus |
| 6 | [The keyboard](06-the-keyboard.md) | Chords, contracts, declining, and the four wire spellings of one keystroke |
| 7 | [Layout and sizing](07-layout-and-sizing.md) | Constraints, bands, grids, and asking a body how big it wants to be |
| 8 | [Theming](08-theming.md) | Roles, glyphs, density, and asking whether the terminal can show a difference |
| 9 | [State, lists and scrolling](09-state-lists-and-scrolling.md) | Caller-owned state, virtualisation, offsets, the wheel |
| 10 | [Overlays](10-overlays.md) | Layers, placement, modality, and the two-state rule |
| 11 | [Time, work and animation](11-time-work-and-animation.md) | Clocks, deadlines, workers, and why a component may not sample time |
| 12 | [Verifying a component](12-verifying-a-component.md) | Counters, reference renders, golden screens, hostile axes |
| 13 | [Shipping a component library](13-shipping-a-component-library.md) | Obligations, lints, dependency policy, documentation |

## The worked example

Everything in this guide is quoted from code that compiles, in
[`examples/component-handbook/`](../../examples/component-handbook). It is a detached crate — copy
the directory and start replacing things.

```sh
cd examples/component-handbook
cargo run              # the application
cargo run -- --probe   # one headless frame, drawn through a counter, numbers printed
cargo test             # four gates, one per rule the guide claims
```

Five components are built there and each one carries a chapter's worth of the argument:

| Component | File | What it is there to show |
|---|---|---|
| `meter` | `src/meter.rs` | Drawing, roles, glyphs, the partition rule |
| `pill` | `src/pill.rs` | Interest, `Response`, one paint resolved before any cell is written |
| `stepper` | `src/stepper.rs` | A declared keyboard contract, and declining what you cannot act on |
| `picker` | `src/picker.rs` | Caller-owned offset, virtualisation, the wheel |
| `menu` | `src/menu.rs` | An overlay body, and the two states an overlay owner needs |

`src/ink.rs` is the seam that makes all of them countable, and `src/main.rs` is the loop.

## Vocabulary

[`CONTEXT.md`](../../CONTEXT.md) is the glossary and the words in it are used here exactly as it
defines them. [`docs/adr/`](../adr) carries the arguments behind the decisions this guide states as
rules — when a rule here looks arbitrary, the ADR is where the measurement is.
