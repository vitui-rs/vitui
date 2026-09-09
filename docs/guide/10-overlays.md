# 10. Overlays

An overlay is a layer: a rectangle with a z-order, composited above the base pass. Menus, dropdowns,
tooltips, dialogs and pickers are all one mechanism.

```rust
cx.overlay(id, anchor, OverlayOpts::sized(width, height), move |cx| {
    // this runs in the overlay pass, after the base pass has finished
});
```

## The one rule everything else follows from

> The body runs in a **second pass**, after the base pass has finished and every context in that
> subtree has been dropped.

So the body is `FnMut(&mut Ctx<'f, '_>) + 'f`, and **a `&mut` borrowed for the owner's call cannot
travel into it**. This is not a lifetime puzzle to be solved with a cleverer signature; it is the
mechanism telling you that an owner and its popup are two things with two lifetimes.

The answer is two states:

```rust
pub struct MenuState { pub open: bool }                  // the owner's
pub struct MenuPopup { pub cursor: usize, pub chosen: Option<usize> }  // the body's
```

The application owns both and passes one into each; the popup's is borrowed for `'f`:

```rust
pub fn menu<'f>(
    cx: &mut Ctx<'f, '_>,
    area: Rect,
    label: &str,
    st: &mut MenuState,
    popup: &'f mut MenuPopup,
    items: &'f [&'f str],
) -> Response
```

That signature makes the caller's `ui` take `&'f mut self`, and the first line of `ui` destructures
`self` into disjoint field borrows so that lending one field for `'f` does not swallow the rest:

```rust
fn ui<'f>(&'f mut self, cx: &mut Ctx<'f, '_>) {
    let App { clears, help, volume, list, menu: menu_state, popup, note } = self;
```

**The answer arrives one frame after the click.** That is not a defect to paper over: the frame that
opens a layer is the frame that draws it, and the frame that reads what the user did inside it is
the next one. Written out, it is one line at the top of the owner:

```rust
if popup.chosen.is_some() {
    st.open = false;
}
```

## Placement

```rust
OverlayOpts {
    z: Z::MENU,
    size: (20, 6),
    placement: Placement::BELOW,
    opaque: true,
    scrim: None,
}
```

- **`size` is stated.** There is no measure pass in the overlay path.
- **`placement`** is a `Side` (`Below`, `Above`, `Right`, `Left`) and an `Align` (`Start`, `Center`,
  `End`). It flips to the opposite side when there is no room.
- **`z`** is a band — `Z::MENU`, `Z::MODAL` — and is **ignored for a nested overlay**, whose z counts
  from its parent's layer instead.
- **`opaque`** says whether the layer erases what is under it. `true` for a menu or a dialog; `false`
  for a transparent gutter, rounded corners, or anything drawn over a chart. The engine prices the
  difference at about 4× on the skip test, so it is not a free choice.
- **`scrim`** is what makes an overlay a modal.

The anchor is translated into root coordinates at the point of the request, because by the time the
pass runs, a rectangle in the requesting context's own coordinates would name a cell nobody can find.

## Modality is three things, and the constructor can only do one

`OverlayOpts::modal(w, h, theme)` gives you the band, the centring and the scrim. The other two
thirds have to happen **inside the body**, because they are draw-time acts:

```rust
cx.overlay(id, anchor, OverlayOpts::modal(40, 12, cx.theme()), move |cx| {
    cx.modal_barrier_here();                       // the pointer
    cx.scope(dialog_id, ScopeKind::Trap, |cx| {    // the keyboard
        …
    });
});
```

Without the barrier the pointer still reaches what is underneath. Without the trap, `Tab` walks out
of the dialog.

## Ids inside a body

`Ctx::id` is still `Location::caller()`, and a loop drawing rows from one source line still mints
one id. Inside an overlay body that is the *normal* case rather than the exception:

```rust
let hit = cx.with_key(i as u64, |cx| {
    let rid = cx.id();
    cx.interact(rid, row, Interest::CLICK.with(Interest::HOVER))
});
```

## What an overlay costs

**n overlays standing is exactly n + 1 allocations**, and the `+ 1` is forced: a `'f`-bounded body
cannot live inside the thing borrowed for `'f`, so the queue the frame owns is one allocation of its
own. There is no `unsafe` anywhere in this and no arena that could remove it.

The frame that *adds* an overlay writes bytes where an identical repaint writes zero, because
`present` composites the new layer. An overlay that appears is on the screen on its own frame.

## Drawing an overlay's contents so a test can see them

A body draws through the context it is handed, so a `&mut Ink` borrowed for the owner's call cannot
travel into it either. The way to keep a popup's drawing countable is to make the body's contents a
**function that a base-pass caller can also invoke**:

```rust
fn popup_body<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, area: Rect, items: &[&str]) { … }
```

The real body calls it with `Direct`; a gate calls it directly in the base pass with a counter. The
alternative — writing the same two orders a second time where a counter can see them — is a copy,
and a gate written against a copy tests the copy.
