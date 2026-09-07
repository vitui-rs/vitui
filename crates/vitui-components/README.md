# vitui-components

Windows, panels, charts, lists, trees, forms and pickers built on
[`vitui-runtime`](https://docs.rs/vitui-runtime).

**Nothing is published. The version is `0.0.0` and there is no stability promise before 0.x.**

## Status

**Implementation-complete: the v1 freeze is 29 of 29 components built** — as a value, `INVENTORY`,
that the tests iterate, rather than a list a reader is asked to trust. All seven documentation and
verification obligations are functions over that value and all seven are met, the last being *every
component stands up under every hostile axis it can meet*, at 34 of 34 `(component, axis)` pairs. A
query with no evidence behind it panics here rather than returning green over an empty population.

The verification register is 234 rows, 229 of them evaluated and **none pinned red**, beside 5 that
are unreachable from a crate that cannot name the engine. Forty-seven scenes stand behind them,
forty-five of which stand up and two of which have nothing to run over, each with the size it is
played at and the property it decides; three of them exist because a defect survived every gate then
in force by not being on any screen anybody had built.

The `media` family ships with no members, and that is deliberate rather than an omission: it is the
one family with no v1 component.

## The shape

A component is `fn(&mut Ctx, Rect, …) -> Response`, with options in a `Default` struct and an
`f_with` sibling for every `f`. There is no widget trait, no builder and nothing to register.

One substitution is stated rather than silent: **`panel` returns a `Panel` and not a `Response`**,
because a container owes the caller the rectangle inside it as well as what happened, and a bare
`Response` cannot say the first. The closure form that would have collapsed the two is refused on
two measurements, which are recorded in `frame.rs`.

## The rule the whole crate is built on

**Every component and every helper writes a partition of its rectangle.** Each cell it is
responsible for is written exactly once, and the cells it does not write are named in its return
value. It is not a style preference: the five ways of breaking it cost 9,024, 1,095, 324, 600 and 15
re-damaged cells a frame on a screen that is not moving, and each of those five is a running arm of
`dense.rs` rather than a row in a table.

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
