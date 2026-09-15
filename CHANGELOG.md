# Changelog

Notable changes to `vitui`, `vitui-engine`, `vitui-runtime` and `vitui-components`. The four are
versioned and released together: they are one library split along the boundaries it is enforced at,
and a reader who has one of them at 0.1.0 has the other three at 0.1.0.

There is **no stability promise before 0.x**. A minor bump may change any public signature, and the
changelog is where that is written down rather than a place where compatibility is claimed.

## 0.1.0 — 2026-09-15

The first release with a library in it. `0.0.1` on crates.io is a placeholder that holds the four
names and contains no code; nothing depends on it and nothing should.

### The library

- **`vitui-engine`** — cells, surfaces, layers, compositing, damage tracking, input and the bytes on
  the wire. `#![forbid(unsafe_code)]`. Damage is marked at write time by the drawing verbs rather
  than derived by diffing two buffers, which is what makes a frame's cost proportional to the cells
  that changed. The engine lays nothing out and never iterates application data.
- **`vitui-runtime`** — layout, identity, focus, hit-testing, routing, key maps, theming with
  fourteen schemes, overlays, scrolling, timers and workers. Depends on the engine and on nothing
  else.
- **`vitui-components`** — 29 components, frozen as a value the tests iterate rather than a list in
  prose: text, panel, chip, button, field, collection, table, tree, select, overlay, scroll area,
  scrollbar, sticky, collapsible, chart, plot, slider, file picker, file preview pane, checkbox,
  radio, switch, meter, sparkline, rule, status bar, pagination, form and spinner.
- **`vitui`** — the facade, so an application names one dependency and one version. An application
  that wants only fast layered output can depend on `vitui-engine` alone and never meet a layout
  type.

### What a caller should know before depending on this

- **Windows Terminal is unverified.** The workspace builds and its whole test suite passes on
  Windows, but the conformance suite that asks a real emulator what it does with the engine's bytes
  drives seven families on macOS and none on Windows. What this library believes about Windows
  Terminal's attribute handling is inference, and it is the one supported terminal nobody has put a
  frame through.
- **A `file_picker`'s preview pane has no keyboard** and is scrolled with the pointer.
- **A `table` whose declared columns are narrower than its band** leaves the remainder of each row to
  the caller; the component writes the part of the band no column claims and no more.
- **Box-junction glyphs are vocabulary a caller spells**, not something the components draw for you.
- **The first frame of a session writes every cell** of a screen the alternate-screen switch has
  already blanked — about 4 900 bytes on an 80×60 terminal, once. This is a decision about what the
  engine may assume of a terminal rather than an oversight.

### Requirements

Rust 1.88, edition 2024. The MSRV is measured rather than inherited from the edition and is verified
by a CI job pinned to it.
