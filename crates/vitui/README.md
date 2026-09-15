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
the workspace is green on Linux, macOS and Windows. **0.1.0 is the first version with code in it**;
the `0.0.1` beneath it is the placeholder that held the name.

**Green on Windows is a build and a test run, not a terminal.** The conformance suite has driven
seven emulator families and none of them runs there, so Windows Terminal is the one supported
terminal nobody has put a screen in front of and what this library believes about it is inference.
[`docs/status.md`](https://github.com/vitui-rs/vitui/blob/master/docs/status.md) is what has been run, and on what.

## Licence

Licensed under the Apache License, Version 2.0. See [LICENSE](https://github.com/vitui-rs/vitui/blob/master/LICENSE).
