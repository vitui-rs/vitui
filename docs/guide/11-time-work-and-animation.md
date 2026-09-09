# 11. Time, work and animation

## `Ctx::now` is the only clock

> **A component may own a time *anchor* and may not own a *clock*.**

```rust
let now = cx.now();          // the moment this frame was sampled at
```

Every component in a frame sees the same instant. A component that calls `Instant::now()` itself is
invisible to `Driver::pin_clock`, which is the entire test regime of this workspace: pin the clock,
advance it by a known duration, assert the frame that comes out. A self-sampling component turns
every animation test into a race.

Stored state may be an **anchor** — *when did this start* — and never a phase.

## Asking for another frame

```rust
cx.deadline(at);              // wake the loop at this instant
cx.deadline_for(id, at);      // the same, attributed to a widget
cx.request_frame();           // wake it as soon as the frame clock allows
```

Deadlines are folded into one wake at `end`. An application that is genuinely idle costs **zero**
wakeups, and that is a gate rather than an aspiration — which means a component that asks for a
frame every frame is a runaway, and shows up as one in the wake census.

The runtime asks for the next frame itself in the two places where something it decided at `end`
still has to be drawn: the pointer award, and a reveal request. You do not need to ask on top of
those.

## Animation

`anim` has four shapes and none of them is a scheduler:

- **`Tween<T>`** — a start instant, a duration, a `from` and a `to`, and an `Easing`. `value(now)`
  interpolates, `done(now)` asks whether it is finished, and `wake(now)` answers the instant to ask
  for next.
- **`Steps`** — a start and a period, for a spinner: `index(now)`, `on(now)`, `next_at(now)`.
- **`Spring`** — position and target, for something that settles.
- **`Easing`** — the curve.

They are values a caller keeps. A component is handed one, reads it against `cx.now()`, and asks for
the next frame:

```rust
let phase = tween.value(cx.now());
if let Some(at) = tween.wake(cx.now()) {
    cx.deadline(at);
}
```

Nothing here retains anything, nothing runs on a timer, and an animation that finishes stops asking
for frames — which is what puts the application back to zero wakeups.

## Work that is not drawing

The application thread draws. Everything else happens elsewhere and **posts**.

```rust
let worker = Worker::hire(driver.wake());
let task: Task<Summary> = Task::new(&worker);

// `key` is the caller's own number for *which question this is*. Asking the same
// question again is `Requested::Deduped` and allocates nothing — which is what makes
// it safe to call this every frame, from inside the draw.
match task.request(key, move |cancel| summarise(input, cancel)) {
    Requested::Asked => { /* a spinner starts */ }
    Requested::Deduped => { /* and keeps spinning */ }
}

// …later, in the loop, after `wait` returned `Wake::Posted`:
if let Some(fresh) = task.take() {     // non-blocking; the check IS the take
    latest = Some(fresh);
}
```

- **`Slot<T>`** is the one place a background result lands: `Send + Sync`, so an `Arc` of one goes
  into as many workers as there are kinds of result.
- **`Task<T>`** adds a `Generation` and a key. The generation is what makes a superseded answer
  discardable — two jobs on two threads can land in either order, so `land` keeps the newer — and
  the key is what makes asking idempotent.
- **`Cancel`** is handed to the job and closed the moment a different question is asked. It is a
  bracket, not a promise: the job may already have finished.
- The check *is* the take. A predicate followed by a take is two reads with a worker running between
  them.
- `Worker::queueing()` is the deterministic arm — every question kept, in order, each taken at most
  once — which is what a test drives instead of a thread.

Never call back into the application thread from a worker. There is no lock to take and no channel
to drain in the draw: everything above the engine is single-threaded by construction, and that is
what the split handles enforce — `Screen` and `View` are `!Send`, and the parker/unparker and
producer/consumer halves are separate types so that an unreachable method is simply not on the type.

## A slow region declares itself

A debug build panics on the **first unexcused overrun** of the frame budget. That is a feature: it
catches the frame that got slow, at the moment it got slow, instead of averaging it away.

```rust
let permit = driver.permit_slow("the cold-start frame");
…
drop(permit);
```

The permit excuses what it covers and its reason is printed by whichever diagnostic fires. Building
the first screen is legitimately slower than a frame budget — nothing is warm, no surface is
allocated — so the cold-start frame holds one.

## Budgets

| Class | Budget |
|---|---|
| Full-screen 300×80 composition | < 1 ms |
| Typical damage-tracked frame | < 100 µs |
| 60 fps steady state | < 5% of a core |
| Allocations during frame composition | **zero** |
| A genuinely idle application | **zero wakeups** |

These are per **class**. An over-budget screen is recorded beside the number, never reclassified
into a looser class to buy headroom.

And a rule about how to gate them at all:

> A gate is a count, a ratio, an equality or a compile outcome. A timing is a report, and a gate only
> at cliff granularity, with the headroom written next to the number.

A gate tuned to the measurement is a flaky test that gets disabled within a month.
