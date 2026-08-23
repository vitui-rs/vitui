# vitui

A fast, layered TUI library for Rust. **This crate is the facade** — it re-exports
[`vitui-engine`](../vitui-engine), [`vitui-runtime`](../vitui-runtime) and
[`vitui-components`](../vitui-components) and contains no logic of its own.

**Nothing is published. The version is `0.0.0` and there is no stability promise before 0.x.**

## Which crate you actually want

| you want | depend on |
|---|---|
| fast layered terminal output, bringing your own layout | `vitui-engine` alone — it is the only one that is implementation-complete |
| layout, focus, routing and theming as well | `vitui-runtime` (9 of 20 tickets in) |
| widgets | `vitui-components` (**empty**) |
| all three, one dependency line | this crate |

The engine is usable on its own and never reaches upward, so taking only it is a supported choice
rather than a workaround.

## Status

Read the [repository README](../../README.md)'s status table before depending on any of these. In
short: the engine is complete and has been compared against one real terminal emulator once, the
runtime is half built, and the components crate is empty.

## Licence

MIT OR Apache-2.0, at your option.
