# 5. Interaction

## Declaring

```rust
let id = cx.id();
let resp = cx.interact(id, area, Interest::CLICK.with(Interest::HOVER).with(Interest::FOCUS));
```

`Interest` is five bits, and the fifth is not like the others:

| Bit | What it asks for | What it costs |
|---|---|---|
| `NONE` | hit-tested and nothing more | nothing |
| `CLICK` | presses and clicks | button reporting |
| `HOVER` | the pointer entering and leaving | **motion reporting** |
| `DRAG` | the pointer's position while a button is held | drag reporting |
| `FOCUS` | a tab stop | **nothing** — the keyboard is delivered whatever the mouse is doing |

The engine's mouse ladder is totally ordered — `Off < Buttons < Drag < Motion` — so combining what
several widgets want is a `max`, which is why the frame accumulates rather than negotiating.

**A declaration is not a correction.** The runtime never edits one: a widget that declares `HOVER`
against a theme whose hover is invisible still gets motion tracking, and is therefore visibly wrong
rather than invisibly fixed. `Theme::hover_interest()` is an *answer a component reads*, not
something done to it.

**Declare once.** A second hit entry under one id makes the claim inert, because it would
double-count the region. A component drawn inside a row that has already declared its own region
should take the `Response` rather than declare a second one — which is what the `_drawn` spelling in
the handbook's `pill` is for.

## What comes back

`Response` carries the id, the rectangle, and every fact the frame resolved about it. The ones you
will use:

| Field | Edge or level | Notes |
|---|---|---|
| `hovered` | level | resolved only for a region that asked for the pointer |
| `pressed` | **level** | true on every frame from the press until the release |
| `press_began` | edge | the frame the button went down |
| `released`, `clicked`, `double_clicked` | edge | `double_clicked` compares the *event's* stamp, never the frame clock |
| `long_pressed` | level | the only field that costs a wakeup |
| `dragged` | `Option<(i32, i32)>` | how far since the press |
| `local` | `Option<(i32, i32)>` | the pointer in **this widget's** coordinates |
| `scrolled` | `(i32, i32)` | wheel delivered to it |
| `focused`, `focus_entered`, `focus_left` | level, edge, edge | |
| `changed` | **yours** | the runtime never sets it; a component sets it to say its own value moved |

**The press is published as an edge beside the level.** A gesture that means *select what is under
the pointer the moment it lands* reads `press_began`; a plain click cannot tell the two apart,
because it is idempotent, and only a modified click flickers. Nothing keeps a `pressing` bool — the
readers hold the `Response`.

`local` is what lets a component declare **one** rectangle and still know where inside it the
pointer landed. The handbook's stepper uses exactly that instead of declaring two arrow regions:

```rust
if resp.clicked && let Some((lx, _)) = resp.local {
    let on_right = lx >= i32::from(area.w) / 2;
    …
}
```

## Resolving the facts into one paint

The tempting shape is to draw and then paint a state over the top. It works, and it costs the cell
twice on every frame the pointer is anywhere near. Resolve first:

```rust
pub fn face_of(resp: &Response, faces: &Faces) -> Role {
    if resp.pressed {
        faces.active
    } else if resp.hovered {
        faces.hover
    } else {
        faces.rest
    }
}
```

The order is the decision: pressed beats hovered, because a pointer that is pressing is also
hovering and the press is the more specific fact.

Then ask whether the paint you chose will actually show:

```rust
let paint_says_focus = resp.focused && cx.theme().roles_differ_on_wire(base, opts.faces.focus);
let face = if paint_says_focus { opts.faces.focus } else { base };
let mark_focus = resp.focused && !paint_says_focus;
```

On a monochrome theme, or at a tier where two roles quantise to the same wire colour, a focused pill
drawn in the focus role *is* a pill drawn in the face role and the user cannot tell which one has
the keyboard. `mark_focus` puts a glyph there instead. This is not a fallback for old terminals: the
same branch is taken under a monochrome theme on a brand-new one.

## Focus

- `cx.focus(id)` seats it. `cx.is_focused(id)` asks about one widget; `cx.focused()` asks whether
  *anything* holds it.
- **Nothing holds the focus until an application seats it.** The idiom is
  `if cx.focused().is_none() { cx.focus(id) }`, inside the draw. `!cx.is_focused(id)` is a different
  program: it takes the keyboard back every frame the user has tabbed away, so `Tab` appears to do
  nothing.
- `Tab` is the ring's and the runtime answers it — a widget never sees the key that is about to move
  the focus off it.
- A `Trap` scope confines the ring, which is how a modal keeps the keyboard.

## Hit-testing, in order

The hit index is in draw order, sixteen bytes an entry, with no rectangle and no layer id: the press
award scans it in reverse and takes the innermost, and modality is one index into the list rather
than a per-entry field. `Ctx::modal_barrier_here()` is what plants that index.

An overlay's entries only enter the index once the layer has been placed, which is why a wheel over
a popup needs two opening frames: a notch is resolved against the **previous** frame's hit index.
