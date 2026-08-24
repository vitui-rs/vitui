# vitui-components

Windows, panels, charts, lists, trees, forms and pickers built on
[`vitui-runtime`](../vitui-runtime).

**Four of twenty-nine components exist**: `text`, `chip`, `button` and `panel`. Everything else is
instruments — the freeze, the gate register, the scene list, the counters and the screens they are
measured on.

Do not depend on this yet. It is published as part of the workspace and it does not have a component
library in it.

## The shape, and the two places it is not spec §1's

Spec §1's rule is `fn(&mut Ctx, Rect, …) -> Response`, with options in a `Default` struct and an
`f_with` sibling for every `f`. That is what ships, with two substitutions that are stated rather
than silent:

- **`Cells` for `Rect`.** `Rect` is `vitui_engine::Rect`, re-exported by none of the runtime's
  twenty-seven public declarations that name it, and this crate's dependency table is
  `vitui-runtime` and nothing else. `cells.rs` names the four candidate answers and why this is
  the one.
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
```

## Licence

MIT OR Apache-2.0, at your option.
