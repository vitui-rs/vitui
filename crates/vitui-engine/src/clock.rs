//! The frame clock, and the one wake source it gates.
//!
//! # The clock gates `wait`, not `present`
//!
//! `docs/adr/0004-the-frame-clock-gates-the-wait.md`, and it is the decision the rest of this module
//! is shaped by. Putting the gate on the app thread so a doomed frame is never
//! *composed* is half a solution: by the time `present` refuses, the runtime has already run its
//! layout, its reactivity and every drawing verb for a frame nobody will see. Measured against an
//! event storm at 1000 Hz with a 167 µs frame standing in for one iteration of runtime work: gating
//! at `present` runs 800.4 iterations a second to show 114.7 frames — **14% useful, 11.4% of a core
//! wasted** — against 114.5 for 114.5 and no waste. And at a sparse event rate it *delivers fewer
//! frames*, **83.8 fps against 110.8**, because it can only paint at instants when an event happens
//! to arrive. **The cheaper scheme is also the smoother one, so there is no trade.**
//!
//! Both gates exist. [`crate::Screen::present`] still refuses a frame the renderer has not asked
//! for, and that is the backstop for a runtime with an external loop that never calls `wait`; this
//! one is the effective one.
//!
//! # It is a minimum gap, not a tick
//!
//! The first damage after a quiet period paints immediately; everything arriving inside the gap
//! coalesces and returns once, at the end of it. A fixed-rate ticker costs CPU while idle and adds
//! up to half a frame of latency to the first keystroke — leading edge has neither, and **idle cost
//! is a standing requirement rather than a preference**.
//!
//! With nothing pending and no deadline registered the wait is **indefinite**, which is the whole of
//! the idle guarantee: no timer exists anywhere in this crate, so there is nothing to wake for.
//!
//! # One wake source, and why `wait` is the only thing that may park
//!
//! There is one mutex and one condvar here, multiplexing everything that can wake the app thread: an
//! input event, an external post, a deadline, and **the renderer going free with a frame owed**. That
//! last one is why the mailbox's own *free* condvar is not what `wait` parks on — one thread cannot
//! park on two condvars, so the render thread signals *this* source when it takes a packet.
//!
//! # The owed frame answers `Wake::Deadline`, and that is a decision rather than a shortcut
//!
//! Spec §7 names four things this source multiplexes and §12 names four `Wake` variants, and they
//! are not the same four: *the renderer going free* has no variant of its own. Ignoring it is not
//! available — the case is real and it loses a frame outright. A slow link, the user stops typing
//! exactly while the renderer is inside a 200 ms write: `present` refused, the damage is still in
//! the structure, nothing else is going to happen, and the last keystroke's echo never reaches the
//! terminal. Unbounded, not merely late.
//!
//! So the fourth reason exists, it is [`Reason::Owed`], and it surfaces as [`Wake::Deadline`]. Three
//! things make that the right spelling rather than a lie of convenience:
//!
//! 1. **What released the app thread really is the clock.** An owed frame is one the *pacing gate*
//!    deferred; it is released when the gap has elapsed **and** the renderer is free, and the gap is
//!    the same gap every other return here is held to.
//! 2. **`Deadline` is the variant whose meaning survives it.** Arch ticket 12 maps `Deadline` onto
//!    *re-view, and the animation phase falls out of `elapsed()`* — which is correct for an owed
//!    frame, because time has genuinely passed. `Posted` maps onto *a background result landed*, and
//!    would send the runtime looking for one that does not exist.
//! 3. **A fifth variant is a public surface this backlog has not decided.** The four are settled,
//!    and every non-`Quit` variant already means the same thing to the caller: drain, draw, present.
//!
//! Recorded because it may return: if the runtime ever needs to tell an owed frame from a deadline —
//! to skip re-running reactivity, say — the answer is a fifth variant and it costs one enum.

use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

/// Why the app thread woke.
///
/// `Wake` says *why we woke*; `Event` says *what happened*. That is why a resize is an `Event` and
/// not a variant here: reacting to it means re-laying rectangles.
///
/// ```
/// use vitui_engine::Wake;
/// assert_ne!(Wake::Quit, Wake::Input);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Wake {
    /// The input thread parsed something. Drain it with `next_event`.
    Input,
    /// [`WakeHandle::post`](crate::WakeHandle::post) was called from somewhere.
    Posted,
    /// A deadline registered through [`Screen::request_wake_at`](crate::Screen::request_wake_at)
    /// has arrived — or the frame clock has released a frame it was holding. See the module
    /// documentation for why those two share a variant.
    Deadline,
    /// [`WakeHandle::quit`](crate::WakeHandle::quit) was called, **or the terminal went away**.
    /// Never paced.
    ///
    /// The second reason is not a second variant, and that is a decision rather than an economy:
    /// when the input thread's read end closes, the terminal this process was drawing on
    /// is gone. Every write after that is discarded — the frame path has no `Result` in it — so
    /// `present` would go on answering `submitted: true` for ever while an application parked in
    /// [`Screen::wait`](crate::Screen::wait) waited on a keyboard that cannot send another byte. An
    /// application that handles quit already does the right thing here, and one that does not was
    /// going to hang either way.
    Quit,
}

/// What `wait` found, before it is narrowed to the four public spellings.
///
/// Four reasons, four variants, and the mapping is not a bijection: see the module documentation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Reason {
    Input,
    Posted,
    /// A registered deadline has arrived.
    Deadline,
    /// A frame the renderer was too busy to take, and the renderer is free now.
    Owed,
}

impl Reason {
    /// The public spelling.
    fn wake(self) -> Wake {
        match self {
            Reason::Input => Wake::Input,
            Reason::Posted => Wake::Posted,
            Reason::Deadline | Reason::Owed => Wake::Deadline,
        }
    }
}

/// The input thread parsed something.
pub(crate) const INPUT: u32 = 1 << 0;
/// Somebody called `post`.
const POSTED: u32 = 1 << 1;
/// Somebody called `quit`.
pub(crate) const QUIT: u32 = 1 << 2;

/// Everything the wake source holds, under its one lock.
///
/// **The flags are under the mutex rather than in an atomic beside it**, and the reason is the
/// classic lost-wakeup: an atomic `fetch_or` that lands between the waiter's check and its
/// `Condvar::wait` is a notification nobody receives, and the waiter then parks with work pending —
/// which is exactly the silent freeze this layer exists to make unrepresentable. A `post` from a
/// worker takes a lock; it happens about as often as a background job finishes.
#[derive(Debug)]
struct Wakes {
    /// `INPUT | POSTED | QUIT`.
    flags: u32,
    /// The earliest deadline registered, and **one slot rather than a set** — see
    /// [`WakeSource::request_wake_at`], which is where a later instant is dropped and why that is a
    /// sink rather than a loss. *Deregistration is simply not renewing it.*
    deadline: Option<Instant>,
    /// A frame `present` composed nothing for because the renderer had not taken the last packet, or
    /// discarded because the terminal resized under it. Cleared by the frame that submits and by the
    /// `wait` that releases it.
    owed: bool,
    /// Whether the render thread has taken every packet the app submitted.
    ///
    /// A **level** and not an edge, which is what makes the owed frame race-free: `present` sets
    /// `owed` after the mailbox has authoritatively said *busy*, so the renderer has not taken the
    /// packet yet and the `mark_renderer_free` that follows cannot have been missed.
    ///
    /// The invariant that makes it safe to read this instead of the mailbox: **whenever the mailbox
    /// says ready, this is true.** It is lowered before a submit and raised after every take, and a
    /// take is the only thing that makes the mailbox ready. `Screen::present` is where the ordering
    /// that guarantees it lives.
    free: bool,
    /// How many times the app thread has entered a blocking wait. Register entry #17.
    ///
    /// A plain field with a `cfg(test)` accessor, which is the shape `crate::handoff`'s counters
    /// already have: what has to exist in a release build is the *park*, and two increments inside a
    /// critical section that is about to block are not a cost anybody can find.
    parks: u64,
    /// How many times a blocking wait has returned — a notification or a timeout, spurious wakeups
    /// included. **Zero over an idle window is the gate**, and it is a count rather than an
    /// adjective.
    wakeups: u64,
}

/// The one thing that can wake the app thread.
#[derive(Debug)]
pub(crate) struct WakeSource {
    state: Mutex<Wakes>,
    ready: Condvar,
}

impl WakeSource {
    /// Nothing pending, no deadline, no frame owed, and the renderer free.
    pub(crate) fn new() -> WakeSource {
        WakeSource {
            state: Mutex::new(Wakes {
                flags: 0,
                deadline: None,
                owed: false,
                free: true,
                parks: 0,
                wakeups: 0,
            }),
            ready: Condvar::new(),
        }
    }

    /// Block until there is a reason to run a frame **and** the clock allows one.
    ///
    /// `allowed` is the earliest instant at which a frame may be composed — `None` when nothing is
    /// pacing. A reason arriving before it is held rather than returned, and the flags are left set
    /// so that everything else arriving inside the gap folds into the same return.
    ///
    /// **Quit is checked before the clock.** Writing the test is what found it: at a 1 Hz ceiling,
    /// checking the clock first hangs shutdown for a second, and shutdown latency must not be a
    /// function of the refresh rate.
    pub(crate) fn wait(&self, allowed: Option<Instant>) -> Wake {
        let mut state = self.lock();
        loop {
            if state.flags & QUIT != 0 {
                state.flags &= !QUIT;
                return Wake::Quit;
            }
            let now = Instant::now();
            match state.reason(now) {
                Some(reason) => match allowed {
                    // Inside the gap. Hold to the end of it, keeping every flag: this is the
                    // coalescing, and it is why a keystroke 1 ms after a paint is deliberately
                    // 7.3 ms late at 120 Hz.
                    Some(at) if at > now => state = self.hold(state, at),
                    _ => {
                        state.consume(reason, now);
                        return reason.wake();
                    }
                },
                // **Indefinite when there is no deadline**, which is the whole of the idle
                // guarantee. There is no timer anywhere in this crate to make it otherwise.
                None => {
                    state = match state.deadline {
                        Some(at) => self.hold(state, at),
                        None => self.park(state),
                    };
                }
            }
        }
    }

    /// Park with no timeout.
    fn park<'a>(&'a self, mut state: MutexGuard<'a, Wakes>) -> MutexGuard<'a, Wakes> {
        state.parks += 1;
        let mut state = self
            .ready
            .wait(state)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.wakeups += 1;
        state
    }

    /// Park until `at`, or until somebody signals.
    ///
    /// **`wait_timeout` overshoots**, and the number is here rather than in a bug report: a hold
    /// measures 10.18 ms p50 against an 8.333 ms gap on macOS, so 120 Hz configured yields about
    /// 110 fps under load. Undershooting a ceiling is safe by construction — a ceiling achieved from
    /// below is still a ceiling — and compensating is an implementation choice nobody has needed.
    fn hold<'a>(&'a self, mut state: MutexGuard<'a, Wakes>, at: Instant) -> MutexGuard<'a, Wakes> {
        let now = Instant::now();
        if at <= now {
            return state;
        }
        state.parks += 1;
        let (mut state, _) = self
            .ready
            .wait_timeout(state, at - now)
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.wakeups += 1;
        state
    }

    /// The input thread parsed something. Raised by `crate::reader::run`, once per read that
    /// produced anything — never once per event, which would be a wake per keystroke of a paste.
    pub(crate) fn input(&self) {
        self.raise(INPUT);
    }

    /// Somebody wants a frame run.
    pub(crate) fn post(&self) {
        self.raise(POSTED);
    }

    /// Somebody wants the application to stop.
    pub(crate) fn quit(&self) {
        self.raise(QUIT);
    }

    fn raise(&self, bit: u32) {
        self.lock().flags |= bit;
        self.ready.notify_one();
    }

    /// Keep the earliest deadline, and notify only when it moved earlier.
    ///
    /// **One slot and not a set, and a later instant is dropped rather than queued.** *Outstanding*
    /// in the spec's phrasing reads as though several could be held at once, and they cannot: two
    /// components registering `+8 ms` and `+500 ms` in one frame leave only the 8 ms, and after it
    /// fires there is nothing registered at all.
    ///
    /// That is correct rather than lossy **because the caller re-registers every frame.** The
    /// runtime's settle keeps its own set of deadlines and flushes `request_wake_at(earliest)` once
    /// per frame, so the 500 ms one is re-supplied on the frame the 8 ms one bought — and *deadline
    /// deregistration is simply not renewing it* is the same statement read from the other side. A
    /// caller that registers once and never again gets one wake, which is what it asked for.
    ///
    /// The engine holding the set instead would be a scheduler, and the refusal 9 is that there
    /// isn't one: this is a deadline **sink**.
    ///
    /// A later deadline also changes nothing a parked thread has to know about — it is already waiting
    /// for the earlier one, and waking it to say so would be a wakeup with no work behind it.
    pub(crate) fn request_wake_at(&self, when: Instant) {
        let mut state = self.lock();
        if state.deadline.is_none_or(|held| when < held) {
            state.deadline = Some(when);
            drop(state);
            self.ready.notify_one();
        }
    }

    /// `present` composed nothing, and the damage is still owed a frame.
    pub(crate) fn owe_frame(&self) {
        self.lock().owed = true;
        self.ready.notify_one();
    }

    /// A packet is **about** to go into the slot: the renderer has not taken it, and nothing is owed.
    ///
    /// Called *before* `Mailbox::submit` rather than after, and the order is load-bearing — see
    /// `Screen::present`. Submitting first lets the render thread take the packet and raise `free`
    /// before this thread runs again, and then this call writes `false` over a renderer that is free.
    pub(crate) fn frame_submitted(&self) {
        let mut state = self.lock();
        state.owed = false;
        state.free = false;
    }

    /// The renderer has taken the packet, so the app may compose again.
    ///
    /// Called from the render thread on the threaded path and from `present` itself on the
    /// deterministic one, which is the same instant in both: **the take is what frees the
    /// renderer**, not the write.
    pub(crate) fn mark_renderer_free(&self) {
        self.lock().free = true;
        self.ready.notify_one();
    }

    /// The render thread is not coming back, so the debt is **cancelled** rather than released.
    ///
    /// The wording is exact because the alternative is worse and looks better. Releasing it —
    /// `free = true`, `owed` kept — would answer `Wake::Deadline`, the application would run a frame,
    /// the mailbox would refuse it because `ready` is false for ever, `present` would owe it again,
    /// and the loop would turn once per frame gap against a sink that no longer exists. **A spin is a
    /// hang that costs a core.**
    ///
    /// So the app thread does park after this, and that is the honest state: nothing can be painted
    /// and nothing is going to happen. It is not a new hang — **a dead renderer is indistinguishable
    /// from a permanently busy one through the public surface** (the refusal 7), and a permanently
    /// busy one parks the app thread in exactly the same place. What this call buys is that the park
    /// is not *waiting on the renderer*: any `post` or `quit` gets the app thread out, and `present`
    /// answers `submitted: false` for ever after. Telling the application is a `Wake::Quit` and a
    /// restoration rather than a fifth spelling invented here — and the restoration is
    /// [`crate::shutdown`]'s, which the render thread's own unwind has already performed by the time
    /// this is read: the panic hook runs on whichever thread panicked.
    pub(crate) fn renderer_gone(&self) {
        let mut state = self.lock();
        state.owed = false;
        state.free = false;
        drop(state);
        self.ready.notify_all();
    }

    /// A render thread exists again, after [`renderer_gone`](WakeSource::renderer_gone) said one
    /// did not.
    ///
    /// The one caller is [`Screen::resume`](crate::Screen::resume). `renderer_gone` leaves `free`
    /// false on purpose — a dead renderer must not look like one that can take a packet — and a
    /// suspended session reaches that state deliberately rather than by a panic, so the flag has to
    /// be put back or the first owed frame after a resume waits for a renderer that is standing
    /// there free.
    pub(crate) fn renderer_back(&self) {
        let mut state = self.lock();
        state.free = true;
        drop(state);
        self.ready.notify_one();
    }

    /// The raised flags, for the test that says `post` and `quit` are recorded separately.
    #[cfg(test)]
    pub(crate) fn pending(&self) -> u32 {
        self.lock().flags
    }

    /// How many blocking waits the app thread has entered, and how many have returned.
    ///
    /// **Register entry #17 is the second number over an idle window**, and it is a count rather
    /// than an adjective: a 120 Hz ticker would put 3600 in it over 30 seconds.
    #[cfg(test)]
    pub(crate) fn park_counts(&self) -> (u64, u64) {
        let state = self.lock();
        (state.parks, state.wakeups)
    }

    /// Whether a frame is owed, for the gate that says the owed frame is what releases the wait.
    #[cfg(test)]
    pub(crate) fn owes_a_frame(&self) -> bool {
        self.lock().owed
    }

    /// Whether the renderer can take a packet, for the gate that says a resumed session can be
    /// woken by the frame it owes.
    ///
    /// The pair is the unit: `owed` alone does not release [`wait`](WakeSource::wait) and neither
    /// does `free`, so a gate reading one of them is a gate that passes on half a mechanism.
    #[cfg(test)]
    pub(crate) fn renderer_is_free(&self) -> bool {
        self.lock().free
    }

    /// A poisoned wake source is a thread that panicked inside a critical section that runs no
    /// caller's code, so the state behind the lock is intact whoever died holding it. Recovering the
    /// guard is what keeps a panic on one thread from becoming a second, unrelated panic here.
    fn lock(&self) -> MutexGuard<'_, Wakes> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Wakes {
    /// The highest-priority reason to run a frame, or `None` to park.
    ///
    /// The order is `Input` before `Posted` before a deadline before an owed frame, and it is
    /// deliberately not load-bearing: every one of them leads the caller to the same three
    /// statements — drain, draw, present. What *is* load-bearing is that `QUIT` is checked before
    /// this function is ever called.
    fn reason(&self, now: Instant) -> Option<Reason> {
        if self.flags & INPUT != 0 {
            return Some(Reason::Input);
        }
        if self.flags & POSTED != 0 {
            return Some(Reason::Posted);
        }
        if self.deadline.is_some_and(|at| at <= now) {
            return Some(Reason::Deadline);
        }
        if self.owed && self.free {
            return Some(Reason::Owed);
        }
        None
    }

    /// Take exactly the reason being returned, and nothing else.
    ///
    /// The exactness is a defect the first draft had: clearing `deadline` on the way out of an
    /// **owed** frame throws away a deadline the caller registered and has not reached yet, and the
    /// animation it belongs to then stops until something unrelated happens.
    fn consume(&mut self, reason: Reason, now: Instant) {
        match reason {
            Reason::Input => self.flags &= !INPUT,
            Reason::Posted => self.flags &= !POSTED,
            Reason::Deadline => {
                debug_assert!(self.deadline.is_some_and(|at| at <= now));
                self.deadline = None;
            }
            Reason::Owed => self.owed = false,
        }
    }
}

/// The minimum gap between two frames.
///
/// **In hertz, set by the application and never discovered**, because a tty cannot report a refresh
/// rate and no display query exists anywhere in this crate (the refusal 10). The application
/// discovers it from the platform and says so;
/// [`Screen::set_max_frame_rate`](crate::Screen::set_max_frame_rate) covers a monitor changing under
/// a running program.
#[derive(Clone, Copy, Debug)]
pub(crate) struct FrameClock {
    /// Whether this clock paces at all, which is [`Clock`](crate::Clock)'s to say and immutable for
    /// the life of the screen. Distinct from `gap` being `None`, which is an unlimited *rate* and is
    /// the application's to change.
    paced: bool,
    /// `None` when nothing is paced — an unlimited rate, or the deterministic clock.
    gap: Option<Duration>,
    /// When the last frame was submitted. `None` before the first, which is what makes the leading
    /// edge immediate rather than one gap late.
    last: Option<Instant>,
}

impl FrameClock {
    /// The gap for `hz`, or no pacing at all when `paced` is false.
    ///
    /// **[`Clock::Manual`](crate::Clock) is not paced**, and that is the promise `Manual` was
    /// carrying rather than a convenience: *time only moves when the caller moves it*. A
    /// deterministic mode that contained a real sleep would be reproducible in its interleaving and
    /// not in its timing, which is half of what it is for. Registered deadlines still hold on that
    /// path — an `Instant` the caller chose is the caller's own clock, not this one's.
    pub(crate) fn new(hz: f32, paced: bool) -> FrameClock {
        FrameClock {
            paced,
            gap: if paced { gap_for(hz) } else { None },
            last: None,
        }
    }

    /// A monitor changed under the running program.
    ///
    /// **Ignored on the deterministic clock**, which is what keeps `set_max_frame_rate` from turning
    /// a reproducible path into a timed one. Ignored rather than refused, because the signature has
    /// nothing to refuse with and the call is not a mistake — an application may reasonably set its
    /// ceiling at startup and choose its clock somewhere else.
    pub(crate) fn set_rate(&mut self, hz: f32) {
        if self.paced {
            self.gap = gap_for(hz);
        }
    }

    /// A frame went into the slot.
    ///
    /// The clock is read here rather than by the caller, so that an unpaced screen — the
    /// deterministic path, or an unlimited rate — pays nothing at all per frame rather than one
    /// `Instant::now()` it will never compare against.
    pub(crate) fn submitted(&mut self) {
        if self.gap.is_some() {
            self.last = Some(Instant::now());
        }
    }

    /// The earliest instant at which the next frame may be composed.
    pub(crate) fn next_allowed(&self) -> Option<Instant> {
        Some(self.last? + self.gap?)
    }
}

/// The longest gap a rate can ask for.
///
/// **This is a panic guard and not a policy**, and the panic is reachable from a public field.
/// `Duration::from_secs_f64` panics above about 1.8e19 seconds, and a subnormal `f32` rate — 1e-40 Hz
/// out of somebody's arithmetic — asks for 1e40 of them. Clamping also keeps `Instant + gap` inside
/// what an `Instant` can hold, which is the second place the same number would have overflowed.
///
/// An hour, because a rate below one frame an hour is indistinguishable from never for anything that
/// has a terminal attached, and because clamping to `Duration::MAX` would move the overflow into the
/// addition instead of removing it.
const LONGEST_GAP: Duration = Duration::from_secs(3600);

/// The gap `hz` asks for, or `None` for no pacing.
///
/// **A non-finite or non-positive rate is unlimited rather than never**, and the choice is
/// deliberate: 0 Hz read literally means *never paint*, which is a frozen application, and a zero
/// arriving out of a caller's own arithmetic should not be the way to produce one. `f32::INFINITY`
/// is the honest spelling of unlimited and lands in the same arm.
fn gap_for(hz: f32) -> Option<Duration> {
    if !hz.is_finite() || hz <= 0.0 {
        return None;
    }
    Some(
        Duration::try_from_secs_f64(1.0 / f64::from(hz))
            .unwrap_or(LONGEST_GAP)
            .min(LONGEST_GAP),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A gap that is long enough to observe and short enough not to slow the suite down.
    const SLOW: f32 = 50.0;

    #[test]
    fn a_rate_becomes_a_gap_and_an_unlimited_rate_becomes_none() {
        assert_eq!(gap_for(60.0), Some(Duration::from_secs_f64(1.0 / 60.0)));
        assert_eq!(gap_for(1.0), Some(Duration::from_secs(1)));
        assert_eq!(gap_for(f32::INFINITY), None);
        assert_eq!(gap_for(0.0), None);
        assert_eq!(gap_for(-1.0), None);
        assert_eq!(gap_for(f32::NAN), None);
    }

    /// **A rate small enough to overflow a `Duration` is clamped rather than fatal**, and the panic it
    /// guards is reachable from a public field: `Duration::from_secs_f64` gives up above about
    /// 1.8e19 seconds and a subnormal `f32` asks for 1e40 of them. The clamp also keeps
    /// `Instant + gap` inside what an `Instant` can hold, which is where the same number would have
    /// overflowed a second time.
    #[test]
    fn a_rate_too_small_to_be_a_duration_is_clamped_rather_than_fatal() {
        assert_eq!(gap_for(1e-40), Some(LONGEST_GAP));
        assert_eq!(gap_for(f32::MIN_POSITIVE), Some(LONGEST_GAP));
        // And the clamp is not merely returned: it has to survive the addition the wait does.
        let mut clock = FrameClock::new(1e-40, true);
        clock.submitted();
        assert!(clock.next_allowed().is_some());
    }

    /// The other end: a rate so high the gap rounds to nothing paces nothing, and does not divide by
    /// anything it should not.
    #[test]
    fn a_rate_too_high_to_have_a_gap_paces_nothing() {
        let mut clock = FrameClock::new(f32::MAX, true);
        clock.submitted();
        let allowed = clock.next_allowed().expect("a finite rate arms the gap");
        assert!(
            allowed <= Instant::now(),
            "a gap that rounds to nothing held a frame"
        );
    }

    /// **The leading edge is what the first frame gets**, and it is a property of `last` being
    /// `None` rather than of a special case anywhere.
    #[test]
    fn nothing_is_paced_before_the_first_frame() {
        let clock = FrameClock::new(SLOW, true);
        assert_eq!(clock.next_allowed(), None);
    }

    #[test]
    fn a_submitted_frame_puts_the_next_one_a_gap_away() {
        let mut clock = FrameClock::new(SLOW, true);
        let before = Instant::now();
        clock.submitted();
        let allowed = clock
            .next_allowed()
            .expect("a paced clock has painted once");
        assert!(allowed >= before + Duration::from_millis(20));
        assert!(allowed <= Instant::now() + Duration::from_millis(20));
    }

    /// The deterministic path has no gap and `set_max_frame_rate` may not give it one.
    #[test]
    fn the_deterministic_clock_stays_unpaced_however_it_is_configured() {
        let mut clock = FrameClock::new(120.0, false);
        clock.submitted();
        assert_eq!(clock.next_allowed(), None);
        clock.set_rate(1.0);
        clock.submitted();
        assert_eq!(clock.next_allowed(), None);
    }

    #[test]
    fn an_unlimited_rate_can_be_narrowed_and_widened_again() {
        let mut clock = FrameClock::new(f32::INFINITY, true);
        clock.submitted();
        assert_eq!(clock.next_allowed(), None);
        clock.set_rate(SLOW);
        clock.submitted();
        assert!(clock.next_allowed().is_some());
        clock.set_rate(f32::INFINITY);
        assert_eq!(clock.next_allowed(), None);
    }

    /// **Quit is checked before the clock**, and the fixture is the one from the spec: a 1 Hz
    /// ceiling, a frame just submitted, and a quit that must not wait out the second.
    #[test]
    fn quit_is_never_paced() {
        let source = WakeSource::new();
        source.quit();
        let began = Instant::now();
        assert_eq!(
            source.wait(Some(began + Duration::from_secs(1))),
            Wake::Quit
        );
        assert!(
            began.elapsed() < Duration::from_millis(200),
            "shutdown waited out the frame gap"
        );
    }

    /// Quit is consumed, exactly like every other reason: a second `wait` does not answer with a
    /// shutdown nobody asked for twice.
    #[test]
    fn quit_is_consumed_by_the_wait_that_reports_it() {
        let source = WakeSource::new();
        source.quit();
        assert_eq!(source.wait(None), Wake::Quit);
        assert_eq!(source.pending(), 0);
    }

    #[test]
    fn a_post_and_an_input_are_separate_reasons_and_input_is_reported_first() {
        let source = WakeSource::new();
        source.post();
        source.input();
        assert_eq!(source.wait(None), Wake::Input);
        assert_eq!(source.wait(None), Wake::Posted);
        assert_eq!(source.pending(), 0);
    }

    /// A deadline that has already arrived is a reason, and it is taken by the wait that reports it.
    #[test]
    fn an_arrived_deadline_is_reported_once() {
        let source = WakeSource::new();
        source.request_wake_at(Instant::now());
        assert_eq!(source.wait(None), Wake::Deadline);
        assert_eq!(source.pending(), 0);
        // And nothing is left to fire: a second wait would park for ever, so the absence is asserted
        // through the reason function rather than by waiting.
        assert!(source.lock().reason(Instant::now()).is_none());
    }

    /// **The earliest deadline wins, and a later one does not overwrite it.**
    #[test]
    fn the_earliest_deadline_is_the_one_that_is_kept() {
        let source = WakeSource::new();
        let now = Instant::now();
        source.request_wake_at(now + Duration::from_secs(60));
        source.request_wake_at(now);
        assert_eq!(source.wait(None), Wake::Deadline);
    }

    /// The gap holds a reason that arrived too early, and the hold ends at the gap rather than at
    /// the reason.
    #[test]
    fn a_reason_arriving_inside_the_gap_is_held_to_the_end_of_it() {
        let source = WakeSource::new();
        source.post();
        let began = Instant::now();
        let allowed = began + Duration::from_millis(60);
        assert_eq!(source.wait(Some(allowed)), Wake::Posted);
        assert!(
            Instant::now() >= allowed,
            "the gap released a frame before it was allowed"
        );
    }

    /// **A gap already elapsed is no gap at all**: the return is immediate and nothing parks.
    #[test]
    fn a_reason_arriving_after_the_gap_returns_at_once() {
        let source = WakeSource::new();
        source.post();
        assert_eq!(
            source.wait(Some(Instant::now() - Duration::from_secs(1))),
            Wake::Posted
        );
        assert_eq!(source.park_counts(), (0, 0), "an immediate return parked");
    }

    /// The owed frame: `present` refused, the renderer is still holding the packet, and the wait
    /// releases when it lets go.
    #[test]
    fn an_owed_frame_waits_for_the_renderer_and_then_answers_deadline() {
        let source = std::sync::Arc::new(WakeSource::new());
        source.frame_submitted();
        source.owe_frame();
        let renderer = std::sync::Arc::clone(&source);
        let thread = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            renderer.mark_renderer_free();
        });
        assert_eq!(source.wait(None), Wake::Deadline);
        assert!(!source.owes_a_frame(), "the owed frame was reported twice");
        thread.join().expect("the renderer thread does not panic");
    }

    /// **A frame owed while the renderer is busy is not a reason yet**, which is what keeps `wait`
    /// from returning a frame `present` would only refuse again.
    #[test]
    fn an_owed_frame_is_not_a_reason_while_the_renderer_holds_the_packet() {
        let source = WakeSource::new();
        source.frame_submitted();
        source.owe_frame();
        assert!(source.lock().reason(Instant::now()).is_none());
    }

    /// Submitting clears the debt. The frame that went out is the frame that was owed.
    #[test]
    fn submitting_a_frame_clears_the_one_that_was_owed() {
        let source = WakeSource::new();
        source.owe_frame();
        source.frame_submitted();
        assert!(!source.owes_a_frame());
    }

    /// **An owed frame does not eat a deadline that has not arrived.** The first draft cleared both
    /// and stopped an animation dead.
    #[test]
    fn releasing_an_owed_frame_leaves_a_future_deadline_registered() {
        let source = WakeSource::new();
        let later = Instant::now() + Duration::from_secs(60);
        source.request_wake_at(later);
        source.owe_frame();
        assert_eq!(source.wait(None), Wake::Deadline);
        assert_eq!(
            source.lock().deadline,
            Some(later),
            "the owed frame consumed a deadline that had not arrived"
        );
    }

    /// A renderer that died **cancels** the debt rather than releasing it, so the app thread parks
    /// and a `quit` still gets it out.
    ///
    /// Releasing it instead would spin at the frame gap against a sink that no longer exists, which
    /// is a hang that costs a core. What is asserted here is both halves: the wait does not come
    /// back on the death itself, and it does come back on the quit that follows — the elapsed time is
    /// what tells those two apart.
    #[test]
    fn a_renderer_that_left_does_not_leave_an_owed_frame_parked() {
        let source = std::sync::Arc::new(WakeSource::new());
        source.frame_submitted();
        source.owe_frame();
        let dying = std::sync::Arc::clone(&source);
        let began = Instant::now();
        let thread = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            dying.renderer_gone();
            // Nothing is owed and nothing is free, so `wait` needs a reason of its own to come back
            // — a shutdown. Here it is the quit that proves the park ended.
            std::thread::sleep(Duration::from_millis(40));
            dying.quit();
        });
        assert_eq!(source.wait(None), Wake::Quit);
        assert!(
            began.elapsed() >= Duration::from_millis(60),
            "the death released the wait instead of cancelling the debt, which is a spin"
        );
        thread.join().expect("the dying thread does not panic");
    }

    /// **The idle guarantee, as a count.** Nothing pending, no deadline: one park, and not one
    /// wakeup until something actually happens.
    #[test]
    fn an_idle_wait_parks_once_and_wakes_for_nothing() {
        let source = std::sync::Arc::new(WakeSource::new());
        let watcher = std::sync::Arc::clone(&source);
        let observed = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            let counts = watcher.park_counts();
            watcher.quit();
            counts
        });
        assert_eq!(source.wait(None), Wake::Quit);
        let (parks, wakeups) = observed.join().expect("the watcher does not panic");
        assert_eq!(parks, 1, "an indefinite park re-entered the wait");
        assert_eq!(
            wakeups, 0,
            "something woke the app thread while it was idle: a 120 Hz ticker would put 18 here"
        );
    }
}
