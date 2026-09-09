# 4. Identity

An `Id` is the name a widget keeps between frames. Focus, hit-testing, interest, overlay ownership
and scroll association all key on it. It is derived, never registered, and it **may never be
persisted** — it differs between runs.

## Where an id comes from

```rust
let id = cx.id();
```

That is `Id::at(parent, Location::caller())`: the parent identity, plus `file:line:col` of the call
site. Which call site depends on `#[track_caller]`:

> `#[track_caller]` forwards into every `#[track_caller]` function it calls, and through nothing
> else.

So a component whose three spellings are all `#[track_caller]` mints an id for **the application's
line**, and a private helper one frame further down that is *not* `#[track_caller]` mints one for
its own line — which is what you want, because that helper is one widget however many callers it
has.

## The trap, in both directions

**Two call sites of one component are two widgets.** That is the rule working.

**One call site drawing two widgets mints one id.** That is the trap, and it is the one an
application meets:

```rust
for panel in [left, right] {
    file_panel(cx, panel, …);      // ONE id, minted twice
}
```

Both panels are the same widget. They share a focus, a hover and a cursor. The screen looks
perfect — both panels draw their own contents correctly — and the wrong panel answers the keyboard.
This is not hypothetical: it is how it was found, in a two-panel file manager, and again in a
three-panel one.

The fix is a key:

```rust
for (i, panel) in [left, right].into_iter().enumerate() {
    cx.with_key(i as u64, |cx| file_panel(cx, panel, …));
}
```

`Ctx::with_key` mixes a caller-supplied number into the identity for the duration of the closure.
Use the caller's own stable key — a row id, a node id — and not the loop index, whenever the items
can be reordered: an index-keyed list hands row 3's focus to whatever moves into position 3.

For a container that mints many children arithmetically, `Id::keyed(parent, key)` is the same thing
without a closure.

Here it is as a gate, from the handbook's tests:

```rust
driver.frame(|cx| {
    for i in 0..2 {
        let id = cx.id();
        same.push(id);
        let _ = cx.interact(id, Rect::new(0, i, 20, 1), Interest::NONE);
    }
    for i in 0..2u64 {
        keyed.push(cx.with_key(i, |cx| cx.id()));
    }
});
assert_eq!(same[0], same[1], "one source line mints one id");
assert_ne!(keyed[0], keyed[1], "a key is what separates them");
```

**A gate never meets this on its own**, because a gate draws one subject. It is found by writing an
application.

## Containers and identity

> A container that returns a rectangle preserves its children's identity; one that takes a closure
> renames them.

`panel` returns an interior, so what you draw in it has the identity it would have had anyway.
`Ctx::child(rect)` narrows the rectangle and does **not** rename. `Ctx::scope` and
`Ctx::scroll_scope` are the stated exceptions: they open a scope for the focus ring without renaming
what is inside.

That asymmetry is deliberate. A closure-taking container is a new level of the identity tree, and if
you move a widget from one to another it *is* a different widget. A rectangle-returning container is
a layout decision, and moving a rectangle should not lose a selection.

## What a stale id costs, and the rule that catches it

Frame state is rebuilt from the draw and never diffed, so a widget that stops drawing stops
existing. Four id-keyed facts outlive a frame because drawing cannot re-declare them — the pointer
grab, the press origin, the focus and the click record — and three of them are **swept when their
widget stops drawing**.

That sweep is the *vanish rule*, and it has a consequence that looks like an application bug:

> A dialog that closes does not give the keyboard back. The focused widget stopped drawing, so the
> focus moves to the nearest surviving entry in the previous frame's ring order — which on a screen
> with two panels is **the other panel**.

An application that closes something and wants the focus back has to ask for it, and must do so as
a standing one-frame request that outranks *read the focus and believe it*. While that request
stands, a value derived from the focus must not be adopted, or the application reads its own pending
move back as the user's.

## Testing across two frames

The same trap bites tests. Two `stepper_with(…)` lines in two frame closures are two ids, so a focus
planted after the first frame is cleared by the vanish rule at the end of the second — which reads
as *the key was never routed*. One helper, called from both frames, is the fix:

```rust
// Deliberately NOT `#[track_caller]`: the id is minted at this function's own line,
// which is the same line both times.
fn draw(cx: &mut Ctx<'_, '_>, value: &mut i32, opts: &StepperOpts) -> Response {
    stepper::stepper_with(cx, Rect::new(0, 0, 20, 1), value, opts)
}
```
