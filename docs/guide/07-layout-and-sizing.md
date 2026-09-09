# 7. Layout and sizing

Layout is arithmetic on rectangles. It allocates nothing, retains nothing, and there is no measure
walk: a `Row` is a `Copy` value holding two numbers, and a split is a pure function of it.

## Bands

```rust
let [header, body, footer] = Col::new().split(
    screen,
    [Constraint::Fixed(1), Constraint::Weight(1), Constraint::Fixed(1)],
);

let [left, right] = Row::new()
    .spacing(2)
    .margin(1)
    .split(body, [Constraint::Weight(1), Constraint::Weight(1)]);
```

The array literal carries the arity, so there is no turbofish and no way for the count to disagree
with itself. `split_into(band, &spec, &mut out)` is the dynamic-arity form.

`Row` and `Col` are two types rather than one with an axis field, because an axis that is a *type*
cannot be passed the wrong way round.

## Constraints

| Constraint | Meaning | The part that surprises people |
|---|---|---|
| `Fixed(n)` | exactly `n` cells, taken first | over-subscribed lanes are satisfied **greedily in declaration order**, not shrunk proportionally: `[Fixed(15), Fixed(15), Fixed(15)]` on twenty columns is `[15, 5, 0]` |
| `Min(v)` | a weighted lane with a floor | it is a `Weight(1)` lane seeded at `v`, and costs no fixpoint |
| `Max(v)` | a weighted lane with a ceiling | **this is the one that costs a fixpoint**, because clamping one lane hands its surplus to the others |
| `Percent(p)` | `p`% of the **whole band** | not of what the rigid lanes left: `Percent(50)` beside `Fixed(20)` on eighty columns is forty, not thirty |
| `Ratio(a, b)` | `a`/`b` of the whole band | `b == 0` claims nothing rather than dividing by zero |
| `Weight(w)` | a share of what the rigid lanes left | `Weight(0)` claims nothing |

## What a split could not pay for

`measure_into` is the only way to see a `Fit`, and it answers two things a caller sometimes has to
know:

- **`slack`** — cells inside the band no lane claimed. Always **one trailing remainder at the end of
  the band**, never a hole between lanes.
- **`gaps`** — the furniture the band actually paid for. A zero-width band asked for
  `.spacing(2).margin(1)` reports `Gaps::default()`, because there is nowhere to put a margin. A
  caller checking that its lanes tile the band must compare against these numbers and not against
  the ones it asked for.

## Rectangles and grids

`layout::rect` is the operator set: `inset`, `shrink`, `expand`, `intersect`, `clamp_to`,
`split_at_v`, `split_at_h`. `layout::Align::place(area, w, h)` positions a box of a known size, and
`Stack::center` is the common case.

`Grid::new(cols, rows).cell(band, x, y, span_x, span_y)` is **a convenience over `Row` and `Col`,
not a fourth primitive** — two splits with `Weight(1)` lanes, adding no arithmetic of its own. A span
is the union of adjacent cells and takes in the spacing between them. It is documented as a
convenience so that nobody looks for grid-specific behaviour that is not there.

## Text

`layout::text` is where measurement lives:

- `width(s)` — display columns, which is not `s.len()` and not `s.chars().count()`.
- `truncate(s, w)` — the prefix that fits in `w` columns, cut on a **cluster** boundary.
- `wrap(s, w)` and `wrap_height(s, w)`.

Truncation is a cluster question, not a byte or a `char` question: a cell holds an interned
grapheme cluster, and a double-width glyph occupies two cells. Reserve the ellipsis column *before*
you truncate, as the handbook's `pill` does:

```rust
let head = measure::truncate(label, room);
let elided = head.len() < label.len();
let head = if elided {
    measure::truncate(label, room.saturating_sub(1))
} else {
    head
};
```

## Asking a body how big it wants to be

```rust
let measured = cx.measured(20, 4, |inner| {
    inner.text(0, 0, "two", body);
    inner.text(0, 1, "rows", body);
});
assert_eq!((measured.extent.w, measured.extent.h), (4, 2));
```

`Ctx::measured` runs a body against **its own frame and its own surface**, both discarded. It is a
dry run: nothing it declares reaches the real frame, no overlay it queues will ever run, and the
pointer is nowhere.

It is deliberately not a measure *pass*. There is no two-phase layout here — a component is handed a
rectangle and works inside it — and `measured` exists for the cases where a caller genuinely has to
know an extent before it can choose one, at the price of drawing the thing twice.

`sizing::auto_fit(cells)` is the cheaper answer for the common case of *how wide is the widest of
these strings*.

## Claiming and drawing must agree

`sizing::check(widths, claim, draw)` runs a component's own width claim against what it actually
drew, across a range of widths, and reports the disagreements. It exists because the two are written
by the same person on different days, and a component that claims 20 and draws 21 is a component
that overruns its neighbour at exactly one terminal size.

## The trap every layout gate meets

> Every gate plays the component at the full size of its context, so two expressions that differ
> only when those two disagree are one expression.

A component that drew from `x = 0` instead of from the rectangle it was given passed every test in
the crate, because every test played it at `x == 0`. The same shape exists on the width: a clamp
written `min(2 * depth, w - 2)` is one number for two different widths whenever the clamp binds on
neither.

Play the subject **inside** something smaller than the screen, at a size where one clamp binds and
the other does not.
