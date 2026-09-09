# 2. The frame

## The loop

```rust
let mut driver = Driver::attach(Default::default(), *Themes::standard().theme())?;

// The first frame is drawn before the first park: parking with nothing pending is
// indefinite by design, and a screen that appears on the first keystroke is a bug.
driver.frame(|cx| app.ui(cx));

loop {
    match driver.wait() {
        Wake::Quit => break,
        Wake::Input | Wake::Posted | Wake::Deadline => {}
    }
    driver.frame(|cx| app.ui(cx));

    // After the frame, never before.
    if driver.unhandled().iter().any(|k| quit.match_first(k) == Some(QUIT)) {
        break;
    }
}
```

That is the whole of it. `wait` is the **only** blocking call on the application thread. There is no
event handler, no update step and no render step: `ui` runs top to bottom, once, and the screen is a
pure function of the application's fields.

A wake is not an event. One `wait` return can carry a hundred keystrokes, and the frame clock gates
it — the first damage after a quiet period returns at once, everything inside the gap coalesces into
one return at the end of it, and a genuinely idle application costs **zero** wakeups.

## What happens inside `frame`

```
 begin      the id table is stamped, the key batch is split, the pointer is placed
 base pass  your closure runs: layout, draws, declarations, key drains
 overlay    every body queued by `Ctx::overlay` runs, innermost band last
 end        the award is resolved, the focus ring walks, the vanish rule fires,
            hover settles into a style, deadlines are folded into one wake
 present    the damaged rectangles are composited bottom-up and handed to the writer
```

Three consequences you will meet:

**Everything `end` decides is drawn on the next frame.** The pointer award — press, release, click,
cancelled drag — and the focus ring's walk are resolved *after* your closure has run, and rotated in
by the next `begin`. The runtime asks for that next frame itself, so an application parked in `wait`
still sees the press on the frame it happened; but a value you *derive* from the focus must be read
at the **top** of your draw, because everything below it is what the user sees.

**An overlay body runs after the base pass has finished.** Every context in that subtree has been
dropped by then. This is why an overlay body cannot borrow the state its owner is holding — see
chapter 10.

**A frame consumes at most one routing edge.** A burst of keystrokes is several frames rather than
one, which is why holding an arrow key counts smoothly instead of jumping. There are no per-id
inboxes: one queue, drained by whoever has the focus.

## `Ctx`, and its two lifetimes

```rust
pub fn ui<'f>(&'f mut self, cx: &mut Ctx<'f, '_>) { … }
```

`Ctx<'f, 'v>` carries two lifetimes deliberately. `'f` is the frame; `'v` is the borrow of the view
underneath it. With only one, `child()` would shrink the frame lifetime and an overlay body that
captures a base-pass local would stop compiling — which deletes the mechanism overlays rest on.

Most component signatures never mention either: `cx: &mut Ctx<'_, '_>` is what you write until you
queue an overlay body, at which point you need `'f` to say what the body may borrow.

## Manual mode: the loop as a straight-line program

`Config::clock` is public API, not a test fixture. Under `Clock::Manual` both halves of the engine
run inline on the calling thread before `present` returns — same mailbox, same packet, same bytes —
so a test is a straight-line program with no threads in it.

`Driver::headless(w, h)` is the door most component tests take: a driver over a sink, at truecolor,
with `post_key` and `post_mouse` to drive it and `inspect()` to read the frame back.

```rust
let mut driver = Driver::headless(80, 24).expect("a sink attaches");
driver.frame(|cx| { /* draw */ });
let frame = driver.inspect();
assert_eq!(frame.hits().len(), 4);
```

Properties that only exist because of threading — zero wakeups, a packet never superseded, wake-up
latency — cannot be tested in the mode that removes them, and live in the threaded mode as counts.

## Where your code must not be

The application thread draws. Anything else happens somewhere else and **posts**: a worker computes,
leaves the result in a `Slot`, and calls `WakeHandle::post`. The application learns about it as a
`Wake::Posted` and picks it up with a non-blocking `take` — never as a callback on the worker's own
thread, which is what keeps everything above the engine single-threaded by construction.

A debug build panics on the first unexcused overrun of the frame budget. A region that is
legitimately going to be slow declares itself with `Driver::permit_slow("the cold-start frame")`,
whose reason is what a stall report prints.
