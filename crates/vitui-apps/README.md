# vitui-apps

Applications written against the component surface, one file each in `examples/`.

```sh
cargo run -p vitui-apps --example counter
```

## Why this is a crate and not a folder of snippets

Every other crate here is checked by gates it ships itself. This one is checked by **being a
consumer**. Its dependency list is `vitui-runtime` and `vitui-components` — *not* the `vitui`
facade, because the facade re-exports the engine and that would make `vitui::engine::Rect` nameable
here, evaporating the claim without a line changing. A program in this directory that could only be
written by reaching past the runtime is a program saying the component surface is unfinished, and it
would say so at compile time.

It is a workspace member so that `cargo clippy --workspace --all-targets` and the `test` job build
it. `compare/run.sh` is the reason that matters: it invoked a script nobody had committed, and the
job failed at its Measure step from the day it was written. **A consumer nobody builds is a consumer
nobody checks.**

`publish = false`. What ships to crates.io is `vitui`.

## What writing the first one found

Architecture issue 23. `Screen::wait` is the app thread's only blocking call, `Driver` owned its
`Screen` privately, and `Driver::attach` bound the `WakeHandle` it was given as `_wake` and let it
fall. So the only loop an application could write was

```rust
while !app.exit { driver.frame(|cx| app.ui(cx)); }
```

— a spin at 100% of a core. `present` coalesces, so the screen looked right and the whole symptom
was in the CPU. Worse, `Worker::hire(WakeHandle)` was **public and uninvokable**: the whole of spec
§17 was reachable from a test that builds its own `Engine` and from nowhere else.

`Driver::wait` and `Driver::wake` forward both, unchanged. Measured on the shipped `counter`
binary, parked for five seconds:

```
real 5.01
user 0.00
sys  0.00
```

which is `scripts/idle-gate.sh`'s budget met by an application rather than by the engine under test.

## The list is a value

`src/lib.rs` holds `APPS`, one row per file, and three tests read it: the rows and the directory
must be the same set in **both** directions, every row must say what it is and what it exercises,
and **no application may name the engine** — the last read off the source rather than off the
manifest, because a rule enforced only by a manifest is a rule the next manifest edit undoes
silently.

## The apps

| example | what | after |
|---|---|---|
| `counter` | A bordered panel, a centred value, `Left`/`Right`/`q` | [ratatui's counter-app tutorial](https://ratatui.rs/tutorials/counter-app/basic-app/) |
| `triage` | Mail triage over 200 000 messages that do not exist. **Four collections, one component, four values of `Mode`** — a view filter, a folder list, a multi-select and a menu — with the cursor, the anchor and the span list printed along the bottom | — |

```sh
cargo run -p vitui-apps --example triage
```

`triage` is the second and it is **not** a port, because there is nothing to port: what it shows is
ADR 0028's claim that `list`, option list, menu, multi-select, tabs, radio group and segmented
control are one component and one `Mode`, and no other library's tutorial has an equivalent because
no other library makes the claim. The status bar is the point — `Ctrl+A` over two hundred thousand
messages reads **`selected 200000 in 1 span(s), 16 B`**, and every other gesture adds at most one
span.

Switching the **View** filter is an edit in §10's sense: the message list holds *positions in the
filtered order*, and the caller stamps a fresh revision so the component compares one `u64` once a
frame and clears them. Watch `rev` move in the status bar.

Three things it cannot say, each recorded in the file rather than worked around: a **horizontal**
segmented control shares `Selection` and `apply` and lays itself out, because `collection`
virtualises rows; a collection cannot be *given* an id, so the focus is seated from the `Response`
the draw returned; and `Ctx::with_id` cannot be called inside a scroll scope, so ADR 0027's *wrap
the row loop* is written as *wrap the whole component*.

**Ports rather than inventions, where a port is available.** A tutorial's shape is not ours to argue
with, so what it cannot express here is a fact about this surface instead of a taste. `counter`
records two: `PanelOpts` has no title alignment, so the title is left-anchored where ratatui centres
it, and there is no bottom-border title, so the instruction line sits on an interior row. Neither is
worked around in the file — the point of a port is to show the gap.
