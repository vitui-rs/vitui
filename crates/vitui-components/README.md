# vitui-components

Windows, panels, charts, lists, trees, forms and pickers built on
[`vitui-runtime`](../vitui-runtime).

**Nothing is published. The version is `0.0.0` and there is no stability promise before 0.x.**

## Status

**Implementation-complete: 46 tickets, and spec §17's v1 freeze is 29 of 29 components built** — as
a value, `INVENTORY`, that the tests iterate rather than a list a reader is asked to trust. Its
seven documentation and verification obligations are functions over that value: O1, O2, O3, O4, O6
and O7 are met, and O5 — *every component stands up under every hostile axis it can meet* — is the
one left, at 14 of 34 `(component, axis)` pairs. A query with no evidence behind it panics here
rather than returning green over an empty population.

§21's register is 233 rows, 222 of them evaluated and **none pinned red**, beside 5 that are
unreachable across the crate line (`docs/adr/0023`) and 6 with nothing yet to run over. Thirty-three
scenes stand behind them, each with the size it is played at and the property it decides; three
exist because a defect survived every gate then in force by not being on any screen anybody had
built.

The `media` family ships with no members, and that is spec §14 rather than an omission: it is the
one family with no v1 component.

## The shape, and the one place it is not spec §1's

Spec §1's rule is `fn(&mut Ctx, Rect, …) -> Response`, with options in a `Default` struct and an
`f_with` sibling for every `f`. That is what ships, with one substitution that is stated rather
than silent and one that was made and then withdrawn:

- ~~**`Cells` for `Rect`.**~~ **Withdrawn**, and it is the one deviation that ended. `Rect` was
  `vitui_engine::Rect`, re-exported by none of the runtime's twenty-seven public declarations that
  named it, while this crate's dependency table is `vitui-runtime` and nothing else — so the crate
  named its own rectangle. Runtime architecture issue 22 made the runtime re-export every engine
  type its public surface names, and components architecture issue 17 deleted `Cells`: the helpers
  return `vitui_runtime::Rect`, which is what spec §2 and §3 said all along.
- **`Panel` for `Response`, on `panel` alone.** A container owes both §1's rule 4 and §2's *the
  cells it does not write are named in its return value*, and a bare `Response` cannot state the
  second. The closure form that would collapse the two is refused on two measurements — see
  `frame.rs`.

## The rule the whole crate is built on

**Every component and every helper writes a partition of its rectangle** (spec §2, ADR 0026). Each
cell it is responsible for is written exactly once, and the cells it does not write are named in its
return value. It is not a style preference: the five ways of breaking it cost 9 024, 1 095, 324, 600
and 15 re-damaged cells a frame on a screen that is not moving, and each of those five is a running
arm of `dense.rs` rather than a row in a table.

Beside it is the one rule an application owns rather than a component: **the screen clears once, on
its first frame and on a resize**, which is `app::Clears`.

## Reports

```sh
cargo run --release --example primitive_numbers -p vitui-components   # the four, and clearing once
cargo run --release --example dense_numbers -p vitui-components       # the 338-region screen
cargo run --release --example listing_numbers -p vitui-components     # the collection's four axes
```

## Licence

MIT OR Apache-2.0, at your option.
