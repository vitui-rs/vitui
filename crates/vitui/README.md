# vitui

A fast, layered TUI library for Rust. **This crate is the facade** — it re-exports
[`vitui-engine`](https://docs.rs/vitui-engine), [`vitui-runtime`](https://docs.rs/vitui-runtime)
and [`vitui-components`](https://docs.rs/vitui-components) and contains no logic of its own.

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

All three layers are implementation-complete, the v1 freeze is **29 of 29 components built**, and
the workspace is green on Linux, macOS and Windows — but the library is **not released**: crates.io
carries a `0.0.1` placeholder and `0.1.0` is the first version with code in it.
[`docs/status.md`](../../docs/status.md) is what has been run, and on what.

## Licence

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE).
