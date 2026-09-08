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

`Screen::wait` is the app thread's only blocking call, and `Driver` owned its
`Screen` privately, and `Driver::attach` bound the `WakeHandle` it was given as `_wake` and let it
fall. So the only loop an application could write was

```rust
while !app.exit { driver.frame(|cx| app.ui(cx)); }
```

— a spin at 100% of a core. `present` coalesces, so the screen looked right and the whole symptom
was in the CPU. Worse, `Worker::hire(WakeHandle)` was **public and uninvokable**: the whole of spec
the resident worker thread was reachable from a test that builds its own `Engine` and from nowhere else.

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
The claim that `list`, option list, menu, multi-select, tabs, radio group and segmented
control are one component and one `Mode`, and no other library's tutorial has an equivalent because
no other library makes the claim. The status bar is the point — `Ctrl+A` over two hundred thousand
messages reads **`selected 200000 in 1 span(s), 16 B`**, and every other gesture adds at most one
span.

Switching the **View** filter is an edit in the data contract's sense: the message list holds *positions in the
filtered order*, and the caller stamps a fresh revision so the component compares one `u64` once a
frame and clears them. Watch `rev` move in the status bar.

Three things it cannot say, each recorded in the file rather than worked around: a **horizontal**
segmented control shares `Selection` and `apply` and lays itself out, because `collection`
virtualises rows; a collection cannot be *given* an id, so the focus is seated from the `Response`
the draw returned; and `Ctx::with_id` cannot be called inside a scroll scope, so *wrap
the row loop* is written as *wrap the whole component*.

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
  threshold carried by a paint alone stops being distinguishable and nobody is told. That is the
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

## Three ports of programs people use

`commander`, `cluster` and `spf` are Midnight Commander, k9s and superfile. They are **ports**
rather than inventions for `counter`'s reason, one scale up: a program somebody uses every day has
a shape that is not ours to argue with, so what it cannot express here is a fact about this surface
instead of a taste — and a gap three separate applications reach for is a gap rather than a
preference.

```sh
cargo run -p vitui-apps --example commander   # mc: two panels, F1..F10, a shell prompt
cargo run -p vitui-apps --example cluster     # k9s: a view stack, `:` and `/`, logs and YAML
cargo run -p vitui-apps --example spf         # superfile: a sidebar, up to three panels, a footer
```

**All three run on invented data, deliberately.** A real `readdir` or a real `kubectl` would put
the interesting failures in the transport rather than in the library under test, and everything is
hashed out of its own name so two runs agree and an edit is a *deliberate* change rather than a
different random draw.

### What they found

- **`commander` found the identity trap from the application side.** Two panels are two calls to
  one function from **one** source line; `Ctx::id` is `Location::caller()`; without `Ctx::with_key`
  the second panel takes the first one's focus, cursor and hover, and the screen looks fine while
  the wrong panel answers the keyboard. `spf` has the same shape with three panels. `CLAUDE.md`
  records `#[track_caller]`'s reach as a trap; this is the first time an application has met it.
- **A dialog cannot answer its caller, and neither can a widget that is not focused.**
  `Ctx::next_key` answers *only the focused id*, so a keyboard sink beside a focused `field` is
  deaf: every `Enter` and `Esc` in these three arrives through `Driver::unhandled` instead. Both
  facts are documented and both are easy to get wrong the first time, because the wrong version
  compiles and draws correctly.
- **A band the application composes is a rectangle the application owes in full.** The
  seventh row of `cluster`'s namespace column has no namespace in it, and left unwritten it kept
  whatever the previous frame put there — a pod's `Running`, in the middle of the header.
- **`owns_escape`'s defect was live one key over, and it is fixed.** `collect::from_key` answered a
  bare `Space` with `Gesture::Toggle` and `Ctrl+A` with `Gesture::All` in *every* `Mode`, where
  `apply` acts on the first in every mode but `Mode::Cursor` and on the second in `Mode::Multi`
  alone — so the key was consumed to do nothing, `out.changed` was set for a frame that changed
  nothing, and the container above never saw it. `commander` could not type a space at its shell
  prompt and `cluster` had moved k9s's `space` mark to `Ctrl+Space` for the life of the port.
  `collect::owns` is `owns_escape`'s own rule said of the whole vocabulary — the component owns a
  key exactly when `apply` would do something with it, and declines it otherwise — and the three
  cursor movers are owned unconditionally, because by the time a `Plain` arrives the caller has
  already moved the cursor. `commander`'s prompt takes its spaces now and `cluster` binds k9s's own `space` beside the
  `Ctrl+Space` it had to invent. What it cost was two numbers: `Ctrl+A` came off `PAGER_BINDS`
  (22 → 21) and off `SELECT`, because a pager is `Mode::Options` and a popup's list is
  `Mode::Single` and neither answers `Gesture::All` — *select every row* was a help line no press
  could perform. `Space` did **not** move: a pager toggles with it. `spf`'s `Shift+↓` is **not**
  this defect — the cursor moves, so the key did something, and routing `Extend` into its own marks
  is the application's.
- **A dialog that closes does not give the keyboard back.** The focused widget stopped drawing, so
  the vanish rule moves the focus to *the nearest surviving entry in the previous frame's ring
  order* — which is never where the application wants it. `commander`'s `F5` on the left panel came
  back with the **right** panel active, and `cluster`'s table went deaf after `Esc`. All three now
  carry a one-`bool` standing request and re-seat.
- **A match arm on a bare `Code::Char('q')` also catches `Ctrl+Q`.** `spf`'s help closed on `q` and
  therefore swallowed the application's quit chord — a finding one level up
  (*`Esc` and `Space` are keys and not chords*), met by an application instead of a component. All
  three now answer the quit chord before anything else claims the keyboard, so the way out never
  depends on which dialog is up.
- **An application that hangs inside `Driver::frame` looks exactly like a rendering bug**, and that
  is worth knowing before you go looking at the compositor. `commander` and `spf` both deleted an
  entry by pointing it at *itself* — neat, because no listing scans for it — which turned the walk
  up the parents into an infinite loop the moment anything asked for its path. The screen froze on
  the last good frame, the process stayed alive, nothing panicked, and the visible symptom was *a
  dialog that will not close*. Both now use a `dead` flag and both bound the walk, so a future
  mistake degrades into a wrong path instead of a frame that never returns.
- **A match on `k.code` is a keyboard that works on a legacy terminal and is half dead on a modern
  one.** A terminal speaking the enhanced keyboard protocol sends the base key and the shift bit, so
  `?` is `CSI 47;2;63u` or `CSI 47;2u` and `Shift+N` is `CSI 110;2;78u` or `CSI 110;2u` — and
  `Code::Char('?')` sees neither. `cluster` opened its *filter* on `?`; `spf` lost every capital
  `hotkeys.toml` binds. Both now go through a `KeyMap` of `Chord::typed(c)` plus the
  `Chord::key(base).shift()` alternate, resolved with `KeyMap::match_first` from the
  unhandled window. `commander` was unaffected and that is instructive: its only character route is
  `keys::text`, which reads what the terminal *says was produced*.
- **An overlay that appears IS on the screen on its own frame, and the sentence that stood here
  named the wrong mechanism.** `Ctx::overlay`'s body is entered on the frame the overlay is added,
  with its granted rectangle, and the pass runs inside the `frame` call — before `end` and before
  `present`, which composites its layer. The frame that adds an overlay writes bytes where an
  identical repaint writes **0**, which is
  `ctx::overlay_tests::an_overlay_that_appears_is_on_the_screen_on_its_own_frame`, and it is the
  byte count rather than `Presented::submitted` because damage is marked by the verbs and a boolean
  there cannot fail. What `commander`'s menu bar actually found was **the award**, which is the
  bullet below: a pull-down opened from `Response::clicked` looked like a click on nothing because
  the *click* was a frame late, and `Response::press_began` appeared to work because the release was
  then the event that brought the next frame.
- **Everything `end` decides is drawn on the next frame, and nothing used to ask
  for it.** The award is resolved from the index that has just drawn and rotated into `delivered` by
  the next `begin`; the ring resolves its walk in the same place. Every application here parks in
  `Driver::wait`, so the press drew nothing, the release drew the press, the click waited for
  whatever the user did next, and `commander`'s `Tab` looked like it did nothing at all until the
  next keystroke. `Frame::resolve_award` now asks, beside `resolve_into_view`'s ask for the same
  reason, over two producers: **the pointer award** — a press, a release, a click or a cancelled
  drag — and **the focus ending the frame somewhere other than it started**, which covers the ring,
  a trap's pull, the vanish rule and a press that landed on nothing interested. So no application
  here compares `Driver::inspect().focused()` across frames any more. **The application half of the
  staleness stands**: read anything derived from the focus at the *top* of the draw, not the
  bottom, because everything below it is what the reader sees.

### What they cannot say, in one list

The same missing field turned up three times, which is what makes it worth stating here rather
than in three file headers:

| gap | who wanted it |
|---|---|
| `PanelOpts` has no bottom-border title or border-info items | `counter` (instructions), `commander` (free space), `spf` (`sort · Browser · 3/24`) |
| `PanelOpts` has no title alignment | `counter`, `commander`, `cluster` |
| `StatusOpts` has no per-segment role | `commander`'s function bar paints `1Help` in one colour where `mc` paints two |
| `Column` carries no justification, so a header and its cells can disagree | `cluster`'s right-aligned `RESTARTS`, `CPU`, `MEM` |
| the theme's twenty glyphs include no file icon | `spf`, which is a nerd-font application upstream |
| `←`/`→` are `crate::nav::step`'s, so a container cannot bind them over a collection | `commander`'s menu bar (took `Alt+←`/`Alt+→`), `spf`'s open/parent |
| `crate::collect::Refusal` is `pub(crate)`, so a container's own keys over a `collection` have no in-frame route | `commander`'s pull-downs, `spf`'s sort menu — both read them from `Driver::unhandled` instead |

None of them is worked around in the files. The point of a port is to show the gap.
