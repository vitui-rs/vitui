# vitui

A fast, layered TUI library for Rust. **This crate is the facade** — it re-exports
[`vitui-engine`](../vitui-engine), [`vitui-runtime`](../vitui-runtime) and
[`vitui-components`](../vitui-components) and contains no logic of its own.

**Nothing is published. The version is `0.0.0` and there is no stability promise before 0.x.**

## Which crate you actually want

| you want | depend on |
|---|---|
| fast layered terminal output, bringing your own layout | `vitui-engine` alone |
| layout, focus, routing and theming as well | `vitui-runtime` |
| the twenty-nine components | `vitui-components` |
| all three, one dependency line | this crate |

The engine is usable on its own and never reaches upward, so taking only it is a supported choice
rather than a workaround. `vitui-components` pulls the other two in behind it, which is why the
one-line answer and the three-line answer differ only in what you can name.

## Status

All three layers are implementation-complete: the engine's 26 tickets, the runtime's 21 and the
components crate's 46, with the v1 freeze at **29 of 29 components built**. What a prospective user
should know before depending on any of them is in the [repository README](../../README.md)'s status
section — in short, this workspace has been run on four terminal emulator families and **never on
Windows**.

## Licence

MIT OR Apache-2.0, at your option.
