# 9. State, lists and scrolling

## Where state lives

There are three kinds and only one of them is yours:

| Kind | Example | Who holds it |
|---|---|---|
| **Data** | the rows, the document, the file tree | the application |
| **Component state** | selection, scroll offset, expanded set, cursor | **the caller**, passed in by `&mut` |
| **Frame state** | hit index, focus ring, overlay queue, key queue | the runtime, rebuilt every frame |

A component holds nothing, because there is nowhere to hold it: the runtime keeps no retained
structure, so a value that must survive a frame belongs to whoever is calling.

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PickerState {
    pub cursor: usize,
    pub offset: i32,
}
```

Two numbers, `Copy`, `Default`, public fields. That shape is deliberate: an application can put it
in a struct, restore it from a config file, or keep two of them for two panels. A component that
owned it could do none of those.

## The invariant a list exists to keep

> **Frame cost is proportional to visible cells, never to data volume.**

The loop runs over `offset..offset + h` and **never over the data**:

```rust
for i in 0..i32::from(area.h) {
    let index = st.offset + i;
    let Some(label) = usize::try_from(index).ok().and_then(|k| items.get(k)) else {
        // Past the end of the data. It is still this component's rectangle,
        // so it is still this component's to paint.
        ink.fill(cx, row, " ", row_paint);
        continue;
    };
    …
}
```

And as a gate:

```rust
let hundred: Vec<&str> = (0..100).map(|_| "host").collect();
let many: Vec<&str> = (0..100_000).map(|_| "host").collect();
assert_eq!(cost(&hundred), cost(&many));
```

Note what that gate is careful about: **both fill the viewport**. Ten rows in a twelve-row window is
a different *screen* — two of its rows are the empty tail, one verb each instead of two — and a gate
comparing those would be measuring the tail rather than the data volume. The first draft of this
test compared ten against a hundred thousand and failed for that reason, which is the instrument
being wrong rather than the code.

It follows that a component may only ask the data questions it can answer in `O(1)`. *What is the
k-th visible row* and *how tall is row k* are the two, and a caller that sorts, filters, folds or
wraps owes an **index** that answers them — materialised once when the data changes, spliced rather
than rebuilt, and never recomputed in a frame. Materialising a million-row order costs about 200
frame budgets; moving that work from the component to the caller does not make it cheaper, but it
does make it happen once.

`data::Versioned<T>`, `data::Revision` and `data::Memo<T>` are the runtime's answer to *when did this
change*: a memo's key is every input, and a stale index is noticeable because its revision does not
match.

## The offset is the caller's

Nothing in the runtime stores a scroll position. A component is handed one, moves it, and hands it
back.

```rust
let max = (items.len() as i32 - rows).max(0);   // extent − viewport, floored at zero
st.offset = st.offset.clamp(0, max);

let resp = cx.scrollable(id, area, opts.interest, Scrollable::between((0, st.offset), (0, max)));

st.offset = (st.offset + resp.scrolled.1).clamp(0, max);
```

Three things there are decisions:

**`max` is a bound on the offset, not the content height.** They are the same quantity, so they are
computed in one place. It also means a *tail* — content that does not reach the bottom of the
viewport — exists only where the content is smaller than the viewport, because at any offset the
clamp admits `offset + viewport ≤ extent`.

**The wheel's arithmetic is `offset + resp.scrolled.1`.** A negation there scrolls the wrong way at
every offset except zero, where all four possible spellings agree — which is why it survives a
screenshot review.

**`Scrollable::between` declares which way it can still move**, not which axis it has. A list at its
bottom that reports the y axis as movable eats every downward notch and the pane around it stops
scrolling; the signature is what makes that hard to write by accident.

## Two ways to scroll

**The component owns the offset** — the handbook's `picker`. It does the arithmetic itself, so a
reveal lands on the same frame:

```rust
let cursor = st.cursor as i32;
if cursor < st.offset {
    st.offset = cursor;
} else if cursor >= st.offset + rows {
    st.offset = cursor - rows + 1;
}
```

**A scope owns the offset** — `cx.scroll_scope(id, view, offset, max, |cx| …)`. Everything inside
draws in content coordinates and the runtime clips. A child that wants to be revealed calls
`cx.request_into_view(rect)`, and the offset's owner takes the answer with `cx.take_into_view(id)`
on the **next** frame, as a *delta in content cells* — a delta composes with whatever the owner did
to its own state in between, where an absolute computed against last frame's content would silently
overwrite it. The runtime asks for that next frame itself, so a reveal is not stuck waiting for the
user's next keystroke.

Two things about a scroll scope that are easy to get wrong:

- **`Ctx::area` moves with a scroll and not with a clip.** It answers *the rectangle in the
  coordinate system the context is drawing in* — which is the rectangle for a clip and the window
  for a scroll. `Ctx::child` resets it to zero, because a child's coordinates start at its own
  top-left.
- **A scroll offset is a position; `Ctx::scrolled`'s argument is a translation.** Of the fields a
  scroll moves, the view and the origin take `+dy` and the pointer takes `−dy` — the content moves
  past a pointer that does not, so a press inside a scrolled scope lands on the row under it. At
  offset 0 every sign agrees, which is how a sign error survives.

## The wheel's cadence is a property of the consumer

A notch is resolved against the previous frame's hit index. So:

- a component reading `Response::scrolled` in its own draw sees it on the next frame;
- a body inside an overlay layer needs **two** opening frames, because the layer's entries only
  enter the index once the layer has been placed.

A wheel test is therefore not one drive loop, and a gate written as one will pass over a subject it
never actually drove.
