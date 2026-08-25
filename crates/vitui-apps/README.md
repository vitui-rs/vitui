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
| `latency` | A live p50/p99 latency monitor, an SLO rule and a throughput chart. `g` repertoire · `c` colour depth · `t` threshold axis · `space` pause · `+`/`-` points · `q` quit | — |

```sh
cargo run -p vitui-apps --example latency
```

**`latency` is an invention, and it is the first one here.** A port was not available: nothing
published makes the four things a charting component is *for* visible at once, and three of the four
are only visible in a program that is **running**.

- **`+` and `-` move the series between 1 000, 100 000 and 1 000 000 points.** The readouts show the
  write count and the frame time unmoved and the *fold* time moving by three orders of magnitude.
  Two screenshots side by side cannot show that, because the interesting part is that one number
  moved and the other did not.
- **`g` cycles the repertoire.** The bar chart is byte-identical at Unicode and Extended — block
  elements are the middle rung by `CONTEXT.md`'s own definition — and the plot is not, because
  braille buys exactly one bit of vertical resolution. At ASCII both become a *different
  construction* rather than a worse-looking one.
- **`c` cycles the colour depth and `t` takes the SLO rule off the glyph axis.** At sixteen colours a
  threshold carried by a paint alone stops being distinguishable and nobody is told. That is §16's
  *carried on both axes*, and watching it disappear is the only way to believe it.
- **The readouts count the two memos of the chain separately.** A resize moves the raster's fold
  count and not the range's; a new sample moves both; a still frame moves neither.

What writing it found: **`Theme::custom` needs a background colour and a component cannot read the
page's.** There is no `Theme::page()`, no `Role::Page` and no accessor for a role's background, so
`vitui_components::chart::series_paint` derives one from `Theme::is_dark` — the single bit about the
page that *is* readable. It is right at both ends of the ladder and a guess in the middle, and it is
recorded in the component rather than worked around in the application.

**Ports rather than inventions, where a port is available.** A tutorial's shape is not ours to argue
with, so what it cannot express here is a fact about this surface instead of a taste. `counter`
records two: `PanelOpts` has no title alignment, so the title is left-anchored where ratatui centres
it, and there is no bottom-border title, so the instruction line sits on an interior row. Neither is
worked around in the file — the point of a port is to show the gap.
