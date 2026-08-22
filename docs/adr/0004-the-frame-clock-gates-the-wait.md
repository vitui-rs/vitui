---
status: accepted
date: 2026-08-17
---

# The frame clock gates `wait`, not only `present`

`Screen::wait()` does not return as soon as something happens. It returns as soon as something has
happened **and** the frame clock allows a new frame — the first wake after a quiet period
immediately, and everything arriving inside the minimum gap folded into one return at the end of it.

`Screen::present()` still refuses a frame that arrives too early. Both gates exist; the one on `wait`
is the one that matters.

This is surprising. A reader expects `wait` to mean "block until something happens" and expects the
rate limiting to live where the frames do. A keystroke arriving 1 ms after a paint is deliberately
held for the remainder of the gap — 7.3 ms at a 120 Hz ceiling — before the runtime is even told
about it.

## Why

Ticket 09 put the pacing gate on the app thread so that a frame which would be dropped is never
composed. That is correct and it is not enough: by the time `present` refuses, the runtime has
already run its layout, its reactivity and every drawing verb for a frame nobody will see. The waste
did not go away, it moved one storey up — into the layer this crate exists to serve.

Measured, with an event storm at 1000 Hz and one iteration of runtime work standing in at ticket 05's
realistic frame of 167 µs:

| scheme | ceiling | iterations/s | frames/s | useful | wasted CPU |
|---|---|---|---|---|---|
| gate at `present` | 120 Hz | 800.4 | 114.7 | **14%** | 11.4% of a core |
| gate at `wait` | 120 Hz | 114.5 | 114.5 | **100%** | 0 |
| gate at `present` | 300 Hz | 776.3 | 262.8 | 34% | 8.5% of a core |
| gate at `wait` | 300 Hz | 270.3 | 270.3 | 100% | 0 |

The second finding was not expected and is the stronger one. At a *sparse* event rate — 200 Hz, a
fast key repeat plus a busy background job — gating at `present` does not merely waste work, it
**delivers fewer frames**: 83.8 fps against 110.8 for the same ceiling. Gating at `present` can only
paint at instants when an event happens to arrive, so the achieved rate is quantised by the event
pattern rather than by the clock. Gating at `wait` wakes at the gap boundary and is not.

So the scheme that does less work also shows more frames. There is no trade to weigh there.

## Consequences

**A runtime cannot produce 1000 frames a second, however it is written.** That is not advice in the
documentation; it is where the sleep is. A runtime with its own external loop that never calls `wait`
is still caught by the gate on `present` — the outer defence is the effective one, the inner one is
the backstop.

**The two properties this could have broken were asserted, not assumed.** It remains a minimum gap
and not a tick: a wake after a quiet period returns in **250 ns p50, 1.04 µs p99** (n = 2000), while
one arriving inside the gap is held to the gap. And an idle application is still indefinitely parked
— measured at `3.00 real, 0.00 user, 0.00 sys` over three seconds with **one park and zero wakeups**,
where a 120 Hz ticker would have woken 360 times. Standing requirement 11 survives the mechanism that
most threatened it.

**Quit is never paced.** Writing the test found it: shutdown latency must not be a function of the
refresh rate, so `Wake::Quit` is checked before the clock. At a 1 Hz ceiling the difference is a
second of hang.

**The ceiling is a ceiling, and the achieved rate lands under it.** `wait_timeout` overshoots on
macOS — a hold measures 10.18 ms p50 against an 8.333 ms gap — so 120 Hz configured produces about
110 fps in a storm. Undershooting a ceiling is safe by construction; the number is recorded here so
nobody spends a day discovering it, and compensating for the overshoot is an implementation choice,
not an architectural one.

**Input latency is bounded by one frame interval, by design.** 8.3 ms at 120 Hz, 3.3 ms at 300 Hz.
The delay is only observable through a repaint, which was going to cost the same wait anyway.

The knob is `Config::max_frame_rate`, in hertz, because hertz is what a platform reports. The engine
never asks the hardware — a tty cannot answer — so the application discovers it and sets it, and
`Screen::set_max_frame_rate` handles a monitor changing under a running program. The same number sets
ticket 18's frame-budget overrun threshold, which is one frame interval.

## Amendment, 2026-08-21 — implemented, and the fifth wake reason

Implementation ticket 19 built this. The decision stands unchanged and every number in it was
reproduced; three things the prototype could not have found are recorded here because they are what a
reader of the code will hit.

**The renderer going free is a fifth wake reason and there are four `Wake` variants.** Spec §7 has
the wake source multiplex *an input event, the renderer going free, a deadline, and an external
post*; §12 fixes `Wake` at `Input | Posted | Deadline | Quit`. Those are not the same four, and the
gap is load-bearing rather than cosmetic: when `present` refuses a frame because the renderer has not
taken the last packet, the damage stays in the damage structure and **nothing else is guaranteed to
ask for a frame again.** On a slow link, a user who stops typing while the renderer is inside a
200 ms write loses that keystroke's echo for good — unbounded, not merely late.

So the frame is *owed*, the wait releases when the renderer takes the packet, and it answers
`Wake::Deadline`. Three reasons that is the right spelling and not a convenience: what released the
app thread really is the clock, since an owed frame is one the pacing gate deferred and is released
against the same gap; the runtime's mapping for `Deadline` is *re-view, and the phase falls out of
`elapsed()`*, which is correct because time has genuinely passed, where `Posted` would send it looking
for a background result that does not exist; and a fifth variant is a public surface this backlog has
not decided. **If the runtime ever needs to tell the two apart — to skip re-running reactivity, say —
the answer is that fifth variant and it costs one enum.**

**The mailbox's *free* condvar is not what `wait` parks on.** One thread cannot park on two condvars,
so the wake source owns the park and the render thread signals it from `take`. `Mailbox::wait_until_free`
stayed where it was and is now blocked on only by the gates, where *do not go on until the renderer has
taken that frame* is a question about the handoff rather than about a wake.

**250 ns is the unparked leading edge, and it is not what a keystroke costs.** The figure above is the
gate's own cost when a reason is already pending, so `wait` returns without touching the condvar —
reproduced at 83 ns p50 in a debug build. A genuinely idle app thread woken by a post from another
thread pays a scheduler hop instead, measured at **4.3 µs p50 / 24 µs p99 / 46 µs max** over 256
samples — the same order as this crate's own 4.58 µs handoff figure, and unavoidable. Two quantities,
both reported, because printing only the first advertises 250 ns for something twenty times dearer.

**`Clock::Manual` is not paced at all**, which is the promise that variant was already carrying —
*time only moves when the caller moves it*, so the deterministic mode is reproducible in its timing as
well as in its interleaving. `set_max_frame_rate` is **ignored** there: not rejected, because §12's
signature has nothing to reject with and setting a ceiling at startup while choosing the clock
elsewhere is a reasonable thing for an application to do. Registered deadlines still hold: an `Instant`
the caller chose is the caller's own clock.

**`request_wake_at` is one slot and not a set**, and *outstanding* in §7's phrasing reads as though it
were several. Two components registering `+8 ms` and `+500 ms` in one frame leave only the 8 ms, and
once it fires nothing is registered. That is a sink rather than a loss because the caller re-registers
every frame — the runtime keeps its own set and flushes the earliest of it once per settle — and an
engine holding the set would be the scheduler §12's refusal 9 says there isn't. Written down because
the phrasing invites the opposite reading.

**Idle, measured at the full thirty seconds** rather than three: `30.01 real, 0.00 user, 0.00 sys, 0
voluntary context switches`, where a 120 Hz ticker would have woken 3 600 times. Three seconds was
below the tool's resolution, which is why the gate is `scripts/idle-gate.sh` around a release binary
rather than a test — libtest's harness is in the same process, and so is cargo.
