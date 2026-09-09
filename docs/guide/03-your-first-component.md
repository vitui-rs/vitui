# 3. Your first component

## The signature

```rust
fn name(cx: &mut Ctx<'_, '_>, area: Rect, …data…) -> Response
```

Four rules, and each of them is load-bearing:

1. **The context first, the rectangle second.** A component is handed where to draw; it never
   decides where it is.
2. **Data by shared reference.** A component does not own what it shows.
3. **Options are a `Default` struct with an `_with` sibling**, never a required builder and never
   fifteen positional arguments.
4. **A `Response` comes back, even from a component that draws and nothing else.**

## The three spellings

Every component in the handbook — and every one in `vitui-components` — exists three times:

```rust
#[track_caller]
pub fn meter(cx: &mut Ctx<'_, '_>, area: Rect, ratio: f32) -> Response {
    meter_with(cx, area, ratio, &MeterOpts::default())
}

#[track_caller]
pub fn meter_with(cx: &mut Ctx<'_, '_>, area: Rect, ratio: f32, opts: &MeterOpts) -> Response {
    meter_into(&mut Direct, cx, area, ratio, opts)
}

#[track_caller]
pub fn meter_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    ratio: f32,
    opts: &MeterOpts,
) -> Response {
    …the only body…
}
```

The first is the ninety-per-cent call. The second is for when you need to say something. The third
takes the writer as a parameter, which is what makes the component **countable** — chapter 12 is
about why that matters and what it catches. Two of the three are one line each, which is what keeps
them from drifting apart.

`#[track_caller]` on all three is not decoration: it is what makes `cx.id()` inside the body mint an
id belonging to the **call site** rather than to the library. Chapter 4 is about what happens when
it is missing.

## The partition rule

> A component handed a rectangle writes every cell of it, exactly once.

Both halves are defects and neither implies the other.

**Writing less than the rectangle** leaves whatever was there before. Not the background — the
*previous content*, which on a screen that re-lays out is the last thing that lived in those cells.
A table whose columns do not fill its band leaves the remainder of every row carrying the previous
column set's data.

**Writing a cell twice** costs you that cell on every frame, for ever, on a screen that is not
moving. A panel that clears the interior it is about to hand over costs 22 200 damaged cells a frame
across three panels.

There is exactly one exception, and it is stated in the return value: a container may *return* the
rectangle it did not write. `panel` returns its interior; `fit` writes one row and returns the rows
below it. If your component does not write a cell, the caller has to be able to name it.

Here is the rule in `meter`: the bar is one row, and the rows either side are this component's to
pad.

```rust
let bar_row = area.y + (i32::from(area.h) - 1) / 2;
for y in area.y..area.bottom() {
    if y != bar_row {
        ink.fill(cx, Rect::new(area.x, y, area.w, 1), " ", track_paint);
    }
}
```

And here is the rule as a test, from `src/main.rs`:

```rust
#[test]
fn a_meter_writes_every_cell_of_its_rectangle_exactly_once() {
    let mut driver = headless(40, 6);
    let mut tally = ink::Tally::default();
    let area = Rect::new(3, 1, 30, 3);
    driver.frame(|cx| {
        meter::meter_into(&mut tally, cx, area, 0.37, &Default::default());
    });
    assert_eq!(tally.distinct(), usize::from(area.w) * usize::from(area.h));
    assert_eq!(tally.excess(), 0, "a cell was painted twice");
}
```

`distinct == area` alone is satisfied by a component that paints the whole rectangle three times.
`excess == 0` alone is satisfied by one that paints half of it. You need both.

## Declare, even when you want nothing

```rust
let id = cx.id();
let resp = cx.interact(id, area, Interest::NONE);
```

`Interest::NONE` means *hit-tested and nothing more*. A component that skips this is missing from
the frame's region index, so a container above it cannot find it and every count taken over the
frame has a hole in it. It costs nothing: `Interest::NONE.tracking()` is `MouseMode::Off`.

## Options

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MeterOpts {
    pub full: Role,
    pub track: Role,
    pub label: Role,
    pub show_percent: bool,
}
```

Public fields, `Copy`, `Default`. Two things follow that are worth knowing before you design one.

**`Copy` and no lifetime is what lets an application write one as a `const`.** A `&str` field costs
you that, which is why per-frame *data* — a title, a caption — is an argument beside the rectangle
and not a field of the options struct.

**A public struct with public fields cannot gain a field compatibly.** Adding one is a breaking
change, so the fields you want are cheapest before your first release and are a major version
afterwards.

## Naming a role, never a colour

```rust
let full_paint = cx.theme().paint(opts.full);
let thumb = cx.theme().glyph(Glyph::Thumb);
```

The palette is the theme's and the glyph repertoire is the terminal's. A component that names a
colour survives a theme change and looks wrong; one that writes `█` directly draws a lozenge on a
terminal without that block. Chapter 8 is the whole of this.

## Formatting without allocating

```rust
let _ = cx.stage(format_args!("{percent:>4}%"));
ink.blit(cx, area.x + i32::from(bar_w), bar_row, label_paint);
```

`stage` formats into a buffer the frame owns and answers the display width — which is what you need
*before* you know where to draw, since a right-aligned number measures first. `blit` writes it.
`cx.label(x, y, args, paint)` is the two in one call.

A `format!` in a draw allocates once a frame for ever, and the frame's allocation gate is a count of
zero.
