# 6. The keyboard

## One queue, drained by the focus

```rust
while let Some(key) = cx.next_key(id) {
    match action_of(&key) {
        Some(action) if owns(action, *value, opts) => { … }
        _ => cx.decline(key),
    }
}
```

`cx.next_key(id)` answers **only the focused id**. There are no per-id inboxes: one queue, and the
widget holding the keyboard drains it. `cx.decline(key)` hands one back and **ends the level's
turn** — the key is available to whoever is above, on the next frame or in
`Driver::unhandled` after this one.

Two consequences that have cost real time:

**A keyboard sink beside a focused text field is deaf.** A focused field consumes every text-bearing
key, so a dialog's `Enter` and `Esc` are the field's to decline and arrive in `Driver::unhandled` one
frame later. The code compiles, draws correctly and does nothing.

**A container can still reach what its child declined.** `Ctx::scope`'s after-the-body moment is the
route: when a scope the focus drew inside closes, it becomes the routing target and the decline that
shut the queue is spent. What a scope cannot reach is what the child *consumed* — a tree's fold keys,
say. Consumption is final; declining is not.

## Bindings are a value

```rust
pub const BINDINGS: &[Binding] = &[
    Binding::new(
        &[Chord::new(Code::Left), Chord::key('h'), Chord::typed('-')],
        DECREMENT,
        "Decrease",
    ),
    Binding::new(
        &[Chord::new(Code::Right), Chord::key('l'), Chord::typed('+')],
        INCREMENT,
        "Increase",
    ),
    Binding::new(&[Chord::new(Code::Home)], TO_MIN, "Minimum"),
    Binding::new(&[Chord::new(Code::End)], TO_MAX, "Maximum"),
];
```

One list. The drain loop reads it, the help bar renders from it, and a test sweeps it. A `match` on
a key code inside the draw is a second copy that no gate can read and that goes stale the first time
a chord is added.

Rendering help from the declaration rather than typing it out beside it is what keeps a help bar
from lying:

```rust
pub fn help() -> String {
    let mut out = String::new();
    for b in BINDINGS {
        if !out.is_empty() { out.push_str("   "); }
        write_help(&mut out, b);
    }
    out
}
```

Build it **outside** the draw. It allocates, and a frame allocates nothing.

An application composes bindings with `KeyMap`, declares them for the frame with `cx.key_map(&map)`,
and asks `cx.action(&key)`. `KeyMap::match_first` also works outside the draw, which is what makes it
usable from the unhandled window.

## One keystroke, four wire spellings

This is the part that breaks code which reads perfectly in review.

A terminal speaking the enhanced keyboard protocol reports **the base key and the shift bit**, not
the character. So on a US layout:

| What the user pressed | What may arrive |
|---|---|
| `+` | code `=` with `SHIFT`, text `+` — or code `=` with `SHIFT` and no text at all |
| `?` | code `/` with `SHIFT`, text `?` — or code `/` with `SHIFT` and nothing about `?` |
| `Shift+N` | code `n` with `SHIFT`, text `N` — or code `n` with `SHIFT` and no text |

A `match` on `k.code == Char('?')` therefore works on a legacy terminal and is **half dead on a
modern one**. Two real ports had exactly this: `?` opened the wrong panel and `Shift+N` sorted
nothing, on every terminal in this repository's conformance suite.

The two chord spellings answer different questions:

- **`Chord::new(code)` / `Chord::key(c)`** — *where the key is*. Compares the code and all six
  intent modifiers. This is what a shortcut wants: `Ctrl+S` is the same physical key on every
  layout.
- **`Chord::typed(c)`** — *what the key produced*. Compares the text and the five intent modifiers
  that are **not** `SHIFT`, because a character that needed shift already says so. This is what to
  bind on `+`, `?`, `:` and `_`.

`Chord::typed` covers three of the four spellings. The fourth — the terminal reporting the base key
and the shift bit and saying nothing about the character — is unreachable by any chord and needs an
**alternate**: `Chord::key('/').shift()` beside `Chord::typed('?')`. `KeyMap::match_first` takes the
first match, so bind the shifted alternate **before** the unshifted binding it shares a base key
with.

Two further refusals inside `Chord::matches` that you should know about:

- **A release never matches.** At kitty flag 2 a terminal reports both edges; a map that matched both
  would fire every binding twice. The runtime drops releases in `Frame::begin` for the same reason —
  `Driver::report_key_releases` keeps the wire reachable if you genuinely want a key-up.
- **A repeat does match.** Below the kitty protocol a held key arrives as a stream of presses, so
  matching repeats is what makes a held key behave the same at both tiers. The cost is that a held
  `Ctrl+S` fires repeatedly, and debouncing is the application's, because only it knows which
  actions should not repeat.

## Own a key only when you would do something with it

```rust
pub fn owns(action: ActionId, value: i32, opts: &StepperOpts) -> bool {
    match action {
        DECREMENT | TO_MIN => value > opts.min,
        INCREMENT | TO_MAX => value < opts.max,
        _ => false,
    }
}
```

A key consumed to do nothing is worse than a key not taken. The component has told the container
*handled*, and the dialog that wanted to close, or the pager that wanted to page, never sees it —
with no diagnostic, because a consumption counter cannot tell a key consumed to do something from a
key consumed to do nothing.

Three shapes of this have been live in shipped code: a list swallowing `Esc` when it had no
selection to clear, a `Space` bound in a mode where the gesture behind it did nothing, and a help
bar advertising *select all* in a mode where no press could perform it.

The gate is easy once the predicate exists:

```rust
let taken = driver.unhandled().is_empty();
assert_eq!(taken, expect_taken, "starting at {start}");
```

## A group declares its own axis

A vertical list reads `↑`/`↓`; a horizontal strip reads `←`/`→`. This is a fact about the component,
never an option on it — a caller who could set the other one would be declaring something the
component cannot honour. `Home`, `End`, `PageUp` and `PageDown` belong to both.

The reason it matters is that a **consumed** key cannot reach the container the component is drawn
inside. A vertical list that reads all four arrows leaves a container with no arrow keys to bind,
which is how a file manager ends up binding `Alt+←` for *go up*.

`Ctrl+←` and `Ctrl+→` are word motion and are reserved; do not bind them for stepping.
