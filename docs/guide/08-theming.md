# 8. Theming

> **A component names a `Role`, never a colour.**

## The thirteen roles

`Body`, `Title`, `Dim`, `Disabled`, `Border`, `Face`, `FaceHover`, `FaceActive`, `Selection`,
`Focus`, `Danger`, `Warn`, `Ok`.

```rust
let paint = cx.theme().paint(Role::Selection);
cx.text(x, y, label, paint);
```

`Paint` is what a verb takes. It is opaque: a component cannot take one apart, which is what keeps a
component from developing an opinion about the palette.

Two escape hatches exist and both are named:

- `Theme::mix(a, b, t)` blends two roles, for a gradient or a settling animation.
- `Theme::custom(fg, bg)` takes colours. It is on the **theme**, not on a component, and it carries
  **no tier guarantee** — you get exactly what you asked for and the terminal may quantise it into
  something indistinguishable from the cell beside it. `Theme::colours_differ_on_wire(a, b)` is how
  you discharge that obligation yourself.

## The twenty glyphs

`ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`, `VLine`, `HLine`, the four corners, the four
tees, `Cross`, `Bullet`, `Tick`, `Thumb`, `Track`, `Ellipsis`.

```rust
let thumb = cx.theme().glyph(Glyph::Thumb);   // &'static str
```

The set is a **repertoire ladder**: the same call answers a box-drawing character on a terminal that
has one and an ASCII stand-in on one that does not. Writing `│` directly into a `text` call is the
mistake this exists to prevent — and a private-use codepoint (a file-type icon from a patched font)
has no rung at all, which is why such things are not glyphs and have to be the caller's strings.

`GlyphSet` selects the rung; `Theme::with_glyphs` rebuilds a theme against another one.

## Density

`Density::pad_x()`, `pad_y()` and `gap()` are the three numbers a component asks for when it wants
padding that a compact theme can take away. Ask; do not hard-code a `1`.

## What the terminal can actually show

This is the part that separates a component that degrades from one that merely looks wrong on a
lesser terminal.

```rust
cx.theme().shows(Distinction::Hover)                // will a hover be visible at all?
cx.theme().roles_differ_on_wire(Role::Face, Role::Focus)   // will these two look different?
```

`Distinction` names the nine visual differences a component might rely on — `Hover`, `Fade`,
`Status`, `Stepper`, `Disclosure`, `Separator`, `Thumb`, `Truncation`, `Marker` — and `shows`
answers whether this theme, resolved for this terminal's colour tier, can express it.

`roles_differ_on_wire` is the sharper question: *will these two roles produce different bytes*. It is
`const`, it publishes no index, and it is what the handbook's `pill` asks before it decides between
a paint and a marker glyph:

```rust
let paint_says_focus = resp.focused && cx.theme().roles_differ_on_wire(base, opts.faces.focus);
```

The alternative — assuming colour works — produces a component that is *correct* and *unusable* on a
monochrome theme, and no test that reads cells will notice, because the cells are exactly what the
component intended.

## Tiers

`Theme::resolve(tier)` narrows the thirteen roles to what a colour depth can express.
**The tier is pinned on the engine's configuration and never on the theme alone**: a theme resolved
for truecolor and an engine quantising to 256 colours is two different answers to one question.

```rust
let mut themes = Themes::standard();      // fourteen schemes, four of them light
themes.set_tier(caps.colors);             // told once at start-up, and again if re-detected
driver.set_theme(*themes.theme());
```

A registry that has not been told about the terminal claims **no distinction at all**, which is the
conservative answer rather than a placeholder.

## Changing the theme

`Driver::set_theme` swaps it between frames. It cannot be swapped during one: a component may not
write to the frame's environment, so a theme picker *returns a selection* and the loop applies it.

`cx.theme_changed()` is true for the frame the swap landed on, and only that frame. A component that
caches anything derived from the theme keys that cache on `Theme::revision()`, or on
`Theme::memo_key(data_revision)`, which folds the theme's revision and the data's into one number.

## Bars are reserved, never overlaid

A scrollbar, a status bar or a gutter **takes cells out of the rectangle** and the parts tile the
rectangle exactly. An overlaid bar cannot satisfy that, and the failure mode is a cell owned by two
drawers that disagree about what is in it.
