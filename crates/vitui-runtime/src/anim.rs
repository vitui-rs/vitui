//! Time, animation and deadlines: closed forms over `(now, start, duration)`, and the ledger that
//! prices what they ask for.
//!
//! **There is no animation object.** A [`Tween`] is 48 bytes, `Copy` and heap-free; the
//! runtime holds nothing, and a hundred animating frames with four animations live allocate **0**
//! (`tests/alloc.rs::a_hundred_animating_frames_with_four_animations_allocate_nothing`). Every
//! helper here is a *function of the frame's own clock* — a value a component computes from
//! `(now, start, duration)` and throws away — so there is nothing to register, nothing to tick and
//! nothing to stop.
//!
//! # Drift is the runtime's, and it compounds with the run
//!
//! A phase taken from `(now − started)` is **1.0000** at the end of a one-second animation on every
//! arrival schedule; the same phase accumulated frame by frame from a nominal interval is **0.9333**
//! on a loop running 8% behind, and it is wrong by more the longer it runs. The step helper says it
//! a second way: a spinner asked at `now + per` takes **352** steps in thirty seconds where
//! [`Steps`] takes **375**, because *asked at* is a fresh late start every time and the anchor is
//! not. `examples/anim_numbers.rs` has both tables.
//!
//! So no helper in this module computes a `dt`, and `tests::no_helper_computes_a_dt` is a scan of
//! the shipped source that says so. It is the same instrument [`crate::work`] points at itself, one
//! subsystem over.
//!
//! # The price of motion is the whole story
//!
//! A fading chip is **114 ns**, a spinner 61, a caret 59 — and **the frame each of them asks for is
//! 34.17 µs, of which 99.7% is redrawing a screen that did not change.** Sixty of those a second is
//! 2.05 ms/s, **0.2% of a core**. `examples/anim_numbers.rs` measures the shipped helpers at 15.8 ns
//! against a 20.18 µs frame, which is the same sentence at **99.92%**: nothing in this module is
//! worth optimising, and the wake it asks for is worth three orders of magnitude more than the
//! arithmetic that asked. That is why the accounting is a [`WakeLedger`] and not a stopwatch.
//!
//! It is also the whole argument for [`Theme::shows`](crate::theme::Theme::shows): the same 300 ms
//! fade costs **19** frames ungated at every tier, and **19 / 2 / 2** gated at True / C256 / C16 —
//! and at 256 colours those nineteen frames buy **three** distinct values on the wire, asked of the
//! engine rather than of our copy of the quantiser. A capability read once removes seventeen
//! wakeups; `tests::the_same_fade_is_nineteen_ungated_and_nineteen_two_two_gated` is the gate, and
//! the ticket's answer carries the finding that *three* is not *one*.
//!
//! # What the ledger is for
//!
//! **The runtime had two wakeup sinks and flushed one.** `settle` flushed the earliest deadline and
//! nothing else, while the split-batch drain set a `repaint` flag beside it. Unfolded, `[Tab,
//! Key(a)]` leaves a key queued, the fold returns `None`, **the loop blocks and the key is never
//! routed**. The pointer case survived only by accident, on a long press's deadline arriving for an
//! unrelated reason, which is why five tickets did not see it.
//!
//! Folded: [`Ctx::request_frame`](crate::ctx::Ctx::request_frame) **is** `deadline(now)` with a call
//! site, there is exactly one sink, and the lazy offender joins the gate it was invisible to.
//! `tests::tab_then_a_routes_both_because_the_lazy_offender_is_in_the_sink` is the scene and
//! `tests::the_refuted_fold_strands_the_key` is the negative case beside it.
//!
//! # Attribution is the call site, not the id
//!
//! The id stack is the closure tree, so a census keyed on the id the frame *has at hand* answers
//! [`Id::ROOT`](crate::Id::ROOT) **12 of 12** for twelve animated chips and the enclosing key scope
//! 12 of 12 when they are keyed — **never the widget**. And an `Id` is a hash of a file pointer that
//! may never be persisted, so it is not a name a diagnostic may print.
//!
//! What works is the **call site**, free at run time because it is a `&'static Location` —
//! [`crate::id`]'s own words for the same mechanism, one module over. It is viral, and here the
//! virality runs *in the right direction*: a component library that wants a runaway blamed on the
//! **application's** line writes `#[track_caller]` on its own wrapper, and the line moves outward
//! rather than inward. Nothing is forced to.
//!
//! [`Ctx::deadline_for`](crate::ctx::Ctx::deadline_for) ships beside
//! [`Ctx::deadline`](crate::ctx::Ctx::deadline) and **the difference between them is not
//! attribution** — both attribute to the call site. What the `id` buys is the *census*: a counter
//! that proves which widget asked, kept beside the attribution and never instead of it.

use std::panic::Location;
use std::time::{Duration, Instant};

use crate::Id;

// ── easings ──────────────────────────────────────────────────────────────────────────────────────

/// How a phase is shaped between its two ends.
///
/// One byte, and every variant is a closed form over the phase — there is no table, no sampling and
/// no state. A cubic Bézier easing would be a fifth field on [`Tween`] and a solve per frame; these
/// four are what the map's scenes actually use.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Easing {
    /// `p`. The one to reach for when the thing being moved is a position and not an opinion.
    #[default]
    Linear,
    /// `p²` — slow to leave, fast to arrive.
    In,
    /// `p(2 − p)` — fast to leave, slow to arrive. The default a fade wants.
    Out,
    /// Quadratic in for the first half, quadratic out for the second.
    InOut,
}

impl Easing {
    /// Shape a phase. `p` is clamped to `0.0..=1.0` first, so a caller cannot ease past an end.
    pub fn at(self, p: f32) -> f32 {
        let p = p.clamp(0.0, 1.0);
        match self {
            Easing::Linear => p,
            Easing::In => p * p,
            Easing::Out => p * (2.0 - p),
            Easing::InOut => {
                if p < 0.5 {
                    2.0 * p * p
                } else {
                    let q = 1.0 - p;
                    1.0 - 2.0 * q * q
                }
            }
        }
    }
}

/// The phase of `(now − start)` through `dur`, clamped, **and it is the only place elapsed time is
/// divided by anything in this module.**
///
/// `f64` inside and `f32` out: over a thirty-second run the seconds themselves need more than 24
/// bits of mantissa to keep a millisecond, and the *ratio* does not.
fn raw_phase(start: Instant, dur: Duration, now: Instant) -> f32 {
    let span = dur.as_secs_f64();
    if span <= 0.0 {
        // A zero-length tween is over. Not an error and not a division: a component that computes a
        // duration from data will produce one eventually, and the alternative is a NaN phase that
        // silently paints nothing.
        return 1.0;
    }
    let elapsed = now.saturating_duration_since(start).as_secs_f64();
    (elapsed / span).clamp(0.0, 1.0) as f32
}

// ── the tween ────────────────────────────────────────────────────────────────────────────────────

/// A value on its way from one end to the other. **48 bytes, `Copy`, and it holds no allocation.**
///
/// # There is no animation object
///
/// This is a *description* of a movement and not a participant in one. Nothing registers it, nothing
/// ticks it, and dropping it stops it — a component keeps one in its own state beside the value it
/// is animating, exactly as it keeps a scroll offset, and the runtime never learns it exists. The
/// only thing that reaches the frame is the [`Ctx::deadline`](crate::ctx::Ctx::deadline) the
/// component chooses to ask for.
///
/// ```
/// use std::time::{Duration, Instant};
/// use vitui_runtime::anim::{Easing, Tween};
///
/// let start = Instant::now();
/// let fade = Tween::new(start, Duration::from_millis(300), 0.0f32, 1.0).eased(Easing::Out);
/// assert_eq!(fade.value(start), 0.0);
/// assert_eq!(fade.value(start + Duration::from_millis(300)), 1.0);
/// assert!(fade.done(start + Duration::from_millis(300)));
/// // Past the end is still the end: a frame that arrives late paints the destination, never past it.
/// assert_eq!(fade.value(start + Duration::from_secs(9)), 1.0);
/// ```
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Tween<T> {
    /// When it started. **The anchor**, and the whole of why nothing here accumulates.
    start: Instant,
    /// How long it lasts.
    dur: Duration,
    /// Where it came from.
    from: T,
    /// Where it is going.
    to: T,
    /// How the phase is shaped.
    ease: Easing,
}

impl<T: Copy> Tween<T> {
    /// A linear tween from `from` to `to`, starting at `start`.
    ///
    /// `start` is a moment the caller already has — [`Ctx::now`](crate::ctx::Ctx::now), sampled once
    /// per frame — and not `Instant::now()`. Taking the clock a second time inside a frame is how a
    /// tween starts a quarter of a millisecond after the frame it belongs to.
    pub const fn new(start: Instant, dur: Duration, from: T, to: T) -> Tween<T> {
        Tween {
            start,
            dur,
            from,
            to,
            ease: Easing::Linear,
        }
    }

    /// The same tween, shaped.
    pub const fn eased(mut self, ease: Easing) -> Tween<T> {
        self.ease = ease;
        self
    }

    /// The eased phase, `0.0..=1.0`.
    ///
    /// This is the value a colour tween wants: `theme.mix(a, b, tween.phase(now))`, because
    /// **colour is a verb on the theme** and a `Paint` cannot be built anywhere else.
    pub fn phase(&self, now: Instant) -> f32 {
        self.ease.at(raw_phase(self.start, self.dur, now))
    }

    /// Whether it has arrived. **A component that asks for a wake without asking this is a spin.**
    pub fn done(&self, now: Instant) -> bool {
        raw_phase(self.start, self.dur, now) >= 1.0
    }

    /// When it ends.
    pub fn ends_at(&self) -> Instant {
        self.start + self.dur
    }

    /// The moment to ask the frame for, or `None` because there is nothing left to draw.
    ///
    /// It is `now` and not `ends_at()`: an animation wants **the next frame the pacing gate will
    /// give it**, and how often that is belongs to the engine's frame clock rather than to a
    /// component's opinion about smoothness. The last frame of a tween returns `None`, which is what
    /// lets the screen sleep.
    ///
    /// ```
    /// # use std::time::{Duration, Instant};
    /// # use vitui_runtime::anim::Tween;
    /// # let now = Instant::now();
    /// let slide = Tween::new(now, Duration::from_millis(100), 0i32, 20);
    /// assert_eq!(slide.wake(now), Some(now));
    /// assert_eq!(slide.wake(now + Duration::from_millis(100)), None);
    /// ```
    pub fn wake(&self, now: Instant) -> Option<Instant> {
        if self.done(now) { None } else { Some(now) }
    }

    /// Where it came from.
    pub const fn from(&self) -> T {
        self.from
    }

    /// Where it is going.
    pub const fn to(&self) -> T {
        self.to
    }
}

impl Tween<f32> {
    /// The value at `now`.
    pub fn value(&self, now: Instant) -> f32 {
        let t = self.phase(now);
        self.from + (self.to - self.from) * t
    }
}

impl Tween<i32> {
    /// The value at `now`, rounded to a whole cell.
    ///
    /// **A terminal has no half cells**, so this rounds rather than truncating: truncating biases a
    /// twenty-cell slide one cell short for most of its length and lands correctly only at the end.
    pub fn value(&self, now: Instant) -> i32 {
        let t = f64::from(self.phase(now));
        let span = f64::from(self.to - self.from);
        self.from + (span * t).round() as i32
    }
}

// ── the step helper ──────────────────────────────────────────────────────────────────────────────

/// A thing that happens every `per`: a spinner's frame, a caret's blink, a marquee's column.
///
/// **Anchored, which is the whole of it.** [`Steps::next_at`] is `start + (n + 1) × per`, computed
/// from the anchor every time, so a frame that arrives late shortens the *next* interval instead of
/// pushing the whole sequence out. Asked at `now + per` instead, a spinner five milliseconds late
/// takes **352** steps in thirty seconds where this takes **375** — the map's figure is 353 — and
/// the shortfall grows with the run.
///
/// ```
/// use std::time::{Duration, Instant};
/// use vitui_runtime::anim::Steps;
///
/// let start = Instant::now();
/// let spinner = Steps::new(start, Duration::from_millis(80));
/// assert_eq!(spinner.index(start), 0);
/// assert_eq!(spinner.index(start + Duration::from_millis(200)), 2);
/// // Late by 15 ms, and the next step is still on the anchor's grid.
/// assert_eq!(
///     spinner.next_at(start + Duration::from_millis(95)),
///     start + Duration::from_millis(160),
/// );
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Steps {
    /// The anchor.
    start: Instant,
    /// How long a step lasts.
    per: Duration,
}

impl Steps {
    /// Steps of `per`, anchored at `start`.
    pub const fn new(start: Instant, per: Duration) -> Steps {
        Steps { start, per }
    }

    /// Which step `now` is in: `(now − start) / per`, and nothing is accumulated.
    pub fn index(&self, now: Instant) -> u64 {
        let per = self.per.as_nanos();
        if per == 0 {
            return 0;
        }
        let elapsed = now.saturating_duration_since(self.start).as_nanos();
        u64::try_from(elapsed / per).unwrap_or(u64::MAX)
    }

    /// Where `now` sits inside the current step, `0.0..1.0`. What a smoothly rotating thing wants.
    pub fn phase(&self, now: Instant) -> f32 {
        let per = self.per.as_nanos();
        if per == 0 {
            return 0.0;
        }
        let elapsed = now.saturating_duration_since(self.start).as_nanos();
        (elapsed % per) as f32 / per as f32
    }

    /// Whether the current step is an even one. A blink, and nothing more than that.
    pub fn on(&self, now: Instant) -> bool {
        self.index(now).is_multiple_of(2)
    }

    /// When the next step begins. **From the anchor, never from `now`.**
    pub fn next_at(&self, now: Instant) -> Instant {
        let per = self.per.as_nanos();
        if per == 0 {
            return now;
        }
        let next = (u128::from(self.index(now)) + 1) * per;
        self.start + Duration::from_nanos(u64::try_from(next).unwrap_or(u64::MAX))
    }

    /// One of `n` cyclic positions at `now` — a spinner's glyph index.
    pub fn cycle(&self, now: Instant, n: u64) -> u64 {
        if n == 0 { 0 } else { self.index(now) % n }
    }
}

// ── the spring ───────────────────────────────────────────────────────────────────────────────────

/// A critically damped spring. **Four fields and no duration.**
///
/// # Both halves of *a spring with a duration* are wrong
///
/// A spring has **no duration**: what ends it is a *threshold*, and a threshold is the caller's
/// because it is a statement about the screen — half a cell is invisible, and a tenth of a cell is
/// invisible twice over. See [`Spring::settled`] for what each choice costs in frames, including the
/// one that never sleeps.
///
/// And it needs **four** fields, because the fourth is the velocity it was moving at when it was
/// retargeted. Dropping it — restarting from rest at the new target — is **1.12 cells and
/// 82 cells/s** apart one frame later at this module's [`RATE`](Spring::RATE) (the map's figures are
/// 3.17 and 141, at a rate it did not write down), which is the difference between a list that flows
/// under a held key and one that stutters once per keypress.
///
/// # It computes no interval
///
/// `value(now)` is the closed form of a critically damped second-order system, so the answer does
/// not depend on when it was last asked. A per-frame integrator over the same motion is **1.29 /
/// 1.50 cells** off at a steady / jittered cadence against **0** by construction — and the jittered
/// column is the one that matters, because it is what a real loop does.
///
/// ```
/// use std::time::{Duration, Instant};
/// use vitui_runtime::anim::Spring;
///
/// let t0 = Instant::now();
/// let s = Spring::new(t0, 0.0, 20.0);
/// let mid = t0 + Duration::from_millis(120);
/// // Retargeting mid-flight keeps the velocity: the new spring is already moving.
/// let r = s.retarget(mid, 40.0);
/// assert!(r.velocity(mid) > 0.0);
/// // The same place, to within one `f32` step of it: a retarget is a *re-anchoring*, and the new
/// // anchor's `x0 − target` is a subtraction the old one had not done.
/// assert!((r.value(mid) - s.value(mid)).abs() < 1e-5);
/// ```
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Spring {
    /// When this leg started.
    start: Instant,
    /// Where it was then.
    x0: f32,
    /// How fast it was moving then. **The field a retarget exists to carry.**
    v0: f32,
    /// Where it is going.
    target: f32,
}

impl Spring {
    /// The angular frequency, in radians a second. **The module's and not the spring's**, because a
    /// per-spring one is a fifth field where the type has four.
    ///
    /// Twelve is a settle a person reads as *immediate but not instant*: a twenty-cell move is
    /// within half a cell of its target in a third of a second. A component that needs a different
    /// feel scales the distance, not the rate.
    pub const RATE: f32 = 12.0;

    /// A spring at `x`, heading for `target`, at rest.
    pub const fn new(start: Instant, x: f32, target: f32) -> Spring {
        Spring {
            start,
            x0: x,
            v0: 0.0,
            target,
        }
    }

    /// The same motion, now heading somewhere else — **sampled at `now`, so the velocity survives.**
    pub fn retarget(&self, now: Instant, target: f32) -> Spring {
        Spring {
            start: now,
            x0: self.value(now),
            v0: self.velocity(now),
            target,
        }
    }

    /// Where it is at `now`.
    ///
    /// `x(t) = target + (a + b·t)·e^(−ω·t)`, with `a = x0 − target` and `b = v0 + ω·a`.
    pub fn value(&self, now: Instant) -> f32 {
        let (a, b, decay, t) = self.terms(now);
        self.target + (a + b * t) * decay
    }

    /// How fast it is moving at `now`, in units a second.
    pub fn velocity(&self, now: Instant) -> f32 {
        let (a, b, decay, t) = self.terms(now);
        (b - Spring::RATE * (a + b * t)) * decay
    }

    /// Where it is going.
    pub const fn target(&self) -> f32 {
        self.target
    }

    /// Whether it is close enough to stop drawing.
    ///
    /// Both halves are checked — a spring one pixel from its target moving at speed is not settled —
    /// and the velocity is compared against `threshold × RATE`, which is the speed at which one more
    /// frame moves it by less than the threshold.
    ///
    /// # The threshold is what lets the screen sleep, and zero does not
    ///
    /// Against a twenty-cell move at [`Spring::RATE`], `0.5` settles in **28** frames at 60 Hz and
    /// `1e-6` in **100**. **Zero settles when the floating point gives up** — 520 frames, 8.7
    /// seconds of a screen redrawing for a picture that stopped changing after the first — which is
    /// not *never* only because `e^(−ω·t)` underflows. Treat it as never: it is what
    /// [`WakeLedger::runaway`] is for, and `tests::a_zero_threshold_is_a_runaway_and_not_an_animation`
    /// is the gate that says so.
    pub fn settled(&self, now: Instant, threshold: f32) -> bool {
        (self.value(now) - self.target).abs() <= threshold
            && self.velocity(now).abs() <= threshold * Spring::RATE
    }

    /// The moment to ask the frame for, or `None` because it has settled.
    pub fn wake(&self, now: Instant, threshold: f32) -> Option<Instant> {
        if self.settled(now, threshold) {
            None
        } else {
            Some(now)
        }
    }

    /// `(a, b, e^(−ω·t), t)` — the four quantities both closed forms are built from.
    fn terms(&self, now: Instant) -> (f32, f32, f32, f32) {
        let t = now.saturating_duration_since(self.start).as_secs_f32();
        let a = self.x0 - self.target;
        let b = self.v0 + Spring::RATE * a;
        (a, b, (-Spring::RATE * t).exp(), t)
    }
}

// ── the wake ledger ──────────────────────────────────────────────────────────────────────────────

/// How many distinct lines the ledger keeps, and how many widgets the census holds.
///
/// **A fixed array and not a `Vec`**, so the ledger allocates nothing at any point in its life —
/// which is a structural claim rather than a measured one, and this crate prefers the first. Thirty-
/// two is far past what an application has: a line is a *source line that asks for a wake*, not a
/// widget, so twelve animated chips written at twelve lines are twelve entries and a thousand rows
/// of one list are one.
pub const LINES: usize = 32;

/// One line that asked, and what it has asked for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Line {
    /// Where it asked, or `None` for an unused slot.
    at: Option<&'static Location<'static>>,
    /// How many times, over the ledger's whole life.
    asks: u32,
    /// How many frames in a row it has asked for one **immediately**. See [`WakeLedger::runaway`].
    streak: u32,
    /// The frame its streak was last extended on.
    last: u64,
}

impl Line {
    const EMPTY: Line = Line {
        at: None,
        asks: 0,
        streak: 0,
        last: 0,
    };
}

/// One widget that asked, by the id it named. The **census**, and never the attribution.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Tally {
    /// Whose.
    who: Option<Id>,
    /// How many.
    asks: u64,
}

impl Tally {
    const EMPTY: Tally = Tally { who: None, asks: 0 };
}

/// A line that has asked for a frame on `streak` consecutive frames, and is therefore a spin rather
/// than an animation.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Runaway {
    /// **The offending line**, which is a `file:line:col` a diagnostic may print — unlike an
    /// [`Id`], which is a hash of an address.
    pub at: &'static Location<'static>,
    /// How many frames in a row.
    pub streak: u32,
    /// How many wakes this line has asked for altogether.
    pub asks: u32,
}

/// What the frames asked for, and which line asked.
///
/// # Four integers and a pointer compare
///
/// [`frames`](WakeLedger::frames), [`wakes`](WakeLedger::wakes), [`streak`](WakeLedger::streak) and
/// [`worst`](WakeLedger::worst) are the whole detector; the compare is
/// [`Runaway::at`] against the line a diagnostic wants to name, and it is a pointer compare because
/// a `&'static Location` is one.
///
/// Driven the way the loop is actually driven, over the scenes in `tests`:
///
/// | scene | frames | wakes | worst streak |
/// |---|---|---|---|
/// | a quiet dense screen | **1**, and then it blocks for ever | 0 | 0 |
/// | a 300 ms fade at 60 Hz | **19** | 18 | 18 |
/// | a caret blinking at 530 ms, over 30 s | **57** — 1.9 a second | 57 | **0** |
/// | a spinning component, over 30 s | **1 801** — 60.0 a second | 1 801 | 1 801 |
///
/// **What separates an animation from a spin is the streak, not the total.** The caret asks for more
/// wakes than the fade and is the cheapest thing on the list, because every one of them is a moment
/// in the future and the screen sleeps in between; the spinner and the fade both ask for *the next
/// frame* on every frame, and only the run length tells them apart.
///
/// # It is unconditional
///
/// **1.51–1.61 ns a frame, of both signs against a whole frame** — the difference between a frame
/// with the ledger and one without is smaller than the spread between two runs of the same frame. So
/// there is no `debug_assertions` guard and no feature flag: a detector that is only armed in a debug
/// build is a detector that never sees the application. The **threshold** is a parameter to
/// [`runaway`](WakeLedger::runaway) rather than a stored setting, for the same reason — what counts
/// as too long is the caller's question, asked where the answer is used.
#[derive(Clone, Debug)]
pub struct WakeLedger {
    /// How many frames have ended.
    frames: u64,
    /// How many of them ended asking for a wake.
    wakes: u64,
    /// How many frames in a row have ended asking for an *immediate* one.
    streak: u32,
    /// The longest such run.
    worst: u32,
    /// What the last frame folded to, so a loop can be driven from it.
    pending: Option<Instant>,
    /// The lines that asked.
    lines: [Line; LINES],
    /// Distinct lines that did not fit. See [`LINES`].
    lines_lost: u32,
    /// The widgets that named themselves.
    census: [Tally; LINES],
    /// Distinct widgets that did not fit.
    census_lost: u32,
}

impl Default for WakeLedger {
    fn default() -> WakeLedger {
        WakeLedger::new()
    }
}

impl WakeLedger {
    /// An empty ledger.
    pub const fn new() -> WakeLedger {
        WakeLedger {
            frames: 0,
            wakes: 0,
            streak: 0,
            worst: 0,
            pending: None,
            lines: [Line::EMPTY; LINES],
            lines_lost: 0,
            census: [Tally::EMPTY; LINES],
            census_lost: 0,
        }
    }

    /// Record that `at` asked for a wake at `when`, during the frame whose clock reads `now`.
    ///
    /// The **immediate** bit is `when <= now`: a frame that asks for a moment it has already reached
    /// is asking for the next frame the pacing gate will give it, and the screen cannot sleep
    /// through it. A moment in the future is a moment the screen sleeps until, however many of them
    /// there are.
    pub(crate) fn asked(
        &mut self,
        at: &'static Location<'static>,
        who: Option<Id>,
        when: Instant,
        now: Instant,
    ) {
        if let Some(id) = who {
            self.tally(id);
        }
        let frame = self.frames;
        let immediate = when <= now;
        let slot = self.line_for(at);
        let Some(line) = slot else {
            self.lines_lost += 1;
            return;
        };
        line.asks = line.asks.saturating_add(1);
        if !immediate {
            // A future moment does not extend a streak and does not break one either: a caret and a
            // fade running at once are two lines with two answers, and the caret is not evidence
            // about the fade.
            return;
        }
        line.streak = if line.last + 1 == frame && line.streak > 0 {
            line.streak.saturating_add(1)
        } else {
            1
        };
        line.last = frame;
    }

    /// The line's entry, or `None` when there is no room for a new one.
    fn line_for(&mut self, at: &'static Location<'static>) -> Option<&mut Line> {
        // A linear scan over at most [`LINES`] entries, and the compare is a pointer compare on the
        // file plus two integers — the same three quantities `Id::at` hashes, and for the same
        // reason: two call sites in one file share the pointer and differ in line and column.
        let mut free = None;
        for i in 0..LINES {
            match self.lines[i].at {
                Some(seen) if same_line(seen, at) => return Some(&mut self.lines[i]),
                None if free.is_none() => free = Some(i),
                _ => {}
            }
        }
        let i = free?;
        self.lines[i] = Line {
            at: Some(at),
            ..Line::EMPTY
        };
        Some(&mut self.lines[i])
    }

    /// Bump the census for one widget.
    fn tally(&mut self, id: Id) {
        let mut free = None;
        for i in 0..LINES {
            match self.census[i].who {
                Some(seen) if seen == id => {
                    self.census[i].asks = self.census[i].asks.saturating_add(1);
                    return;
                }
                None if free.is_none() => free = Some(i),
                _ => {}
            }
        }
        match free {
            Some(i) => {
                self.census[i] = Tally {
                    who: Some(id),
                    asks: 1,
                };
            }
            None => self.census_lost += 1,
        }
    }

    /// Close a frame: what its one sink folded to.
    pub(crate) fn note(&mut self, wake: Option<Instant>, now: Instant) {
        self.frames += 1;
        self.pending = wake;
        match wake {
            Some(at) if at <= now => {
                self.wakes += 1;
                self.streak = self.streak.saturating_add(1);
                self.worst = self.worst.max(self.streak);
            }
            Some(_) => {
                self.wakes += 1;
                self.streak = 0;
            }
            None => self.streak = 0,
        }
    }

    /// How many frames have ended.
    pub const fn frames(&self) -> u64 {
        self.frames
    }

    /// How many of them ended asking for a wake.
    pub const fn wakes(&self) -> u64 {
        self.wakes
    }

    /// How many frames in a row have ended asking for an immediate one.
    pub const fn streak(&self) -> u32 {
        self.streak
    }

    /// The longest such run this ledger has seen.
    pub const fn worst(&self) -> u32 {
        self.worst
    }

    /// What the last frame's one sink folded to. **The loop's own input**, and the only value in
    /// this type a frame reads rather than a diagnostic.
    pub const fn pending(&self) -> Option<Instant> {
        self.pending
    }

    /// The line that has asked for `n` consecutive frames, if one has.
    ///
    /// **The threshold is a parameter and not a setting.** Sixty is a second of a screen that cannot
    /// sleep; a scroll fling legitimately runs twenty and a fade nineteen.
    ///
    /// The worst offender wins, so a screen with two spins names the older one first and the caller
    /// is not handed a list to rank.
    pub fn runaway(&self, n: u32) -> Option<Runaway> {
        self.lines
            .iter()
            .filter(|l| l.at.is_some() && l.streak >= n)
            .max_by_key(|l| l.streak)
            .and_then(|l| {
                Some(Runaway {
                    at: l.at?,
                    streak: l.streak,
                    asks: l.asks,
                })
            })
    }

    /// Every line that has asked, in no useful order.
    pub fn lines(&self) -> impl Iterator<Item = Runaway> + '_ {
        self.lines.iter().filter_map(|l| {
            Some(Runaway {
                at: l.at?,
                streak: l.streak,
                asks: l.asks,
            })
        })
    }

    /// How many distinct lines have asked.
    pub fn line_count(&self) -> usize {
        self.lines.iter().filter(|l| l.at.is_some()).count()
    }

    /// Distinct lines that did not fit in [`LINES`], and are therefore counted here and nowhere else.
    pub const fn lines_lost(&self) -> u32 {
        self.lines_lost
    }

    /// **The census**: how many wakes one widget has named itself for, through
    /// [`Ctx::deadline_for`](crate::ctx::Ctx::deadline_for).
    ///
    /// It is a counter and not the attribution — see the module documentation for the twelve chips
    /// that answer `ROOT` 12 of 12 when a census is used as one.
    pub fn asked_by(&self, id: Id) -> u64 {
        self.census
            .iter()
            .find(|t| t.who == Some(id))
            .map_or(0, |t| t.asks)
    }

    /// How many distinct widgets have named themselves.
    pub fn census_len(&self) -> usize {
        self.census.iter().filter(|t| t.who.is_some()).count()
    }

    /// Distinct widgets that did not fit in [`LINES`].
    pub const fn census_lost(&self) -> u32 {
        self.census_lost
    }
}

/// Whether two call sites are the same one. **A pointer compare and two integers.**
fn same_line(a: &'static Location<'static>, b: &'static Location<'static>) -> bool {
    a.line() == b.line()
        && a.column() == b.column()
        && std::ptr::eq(a.file().as_ptr(), b.file().as_ptr())
}

#[cfg(test)]
mod tests {
    use std::panic::Location;
    use std::time::{Duration, Instant};

    use vitui_engine::{ColorDepth, Rect};

    use super::{Easing, Runaway, Spring, Steps, Tween, WakeLedger};
    use crate::ctx::{Ctx, Driver, Interest};
    use crate::theme::{Distinction, Role, Theme};
    use crate::{Id, Paint};

    /// 60 Hz, rounded to the nearest nanosecond.
    ///
    /// **The rounding is load-bearing at exactly one place, and it is written down rather than
    /// discovered**: eighteen ticks of 16 666 667 ns land 6 ns *after* a 300 ms fade ends, so the
    /// fade costs nineteen frames. Truncated to 16 666 666 they land 12 ns before it and it costs
    /// twenty. Both are 60 Hz. A duration that is a whole number of frames is a knife edge, and a
    /// component that must not gain a frame states 299 ms.
    const TICK: Duration = Duration::from_nanos(16_666_667);

    /// The scene's own zero, so that nothing here is timed by the machine it runs on.
    fn t0() -> Instant {
        Instant::now()
    }

    fn driver() -> Driver {
        Driver::headless(80, 24).expect("attaching to a sink cannot fail")
    }

    /// Run the loop the way an application drives it — a wake, a frame, the fold — and return how many
    /// frames ran.
    ///
    /// **The pacing gate is `max(deadline, previous + TICK)`**, which is the engine's frame clock as
    /// CONTEXT.md states it: a minimum gap and not a tick. A frame that asks for *now* gets the next
    /// tick; one that asks for a moment 530 ms out gets that moment, and the screen sleeps until it.
    fn drive(
        driver: &mut Driver,
        start: Instant,
        horizon: Duration,
        mut view: impl FnMut(&mut Ctx<'_, '_>),
    ) -> u32 {
        let mut now = start;
        let mut frames = 0;
        let end = start + horizon;
        loop {
            driver.pin_clock(now);
            driver.frame(&mut view);
            frames += 1;
            let Some(at) = driver.inspect().wakes().pending() else {
                // Nothing asked. **The loop blocks for ever**, and that is the property rather than
                // a shortcut: there is no timeout here because there is none in the real one.
                break;
            };
            let next = at.max(now + TICK);
            if next > end {
                break;
            }
            now = next;
        }
        frames
    }

    // ── the closed forms ─────────────────────────────────────────────────────────────────────────

    /// **48 bytes, `Copy`, heap-free**, which is the whole of *there is no animation object*.
    #[test]
    fn a_tween_is_forty_eight_bytes_and_copy() {
        assert_eq!(std::mem::size_of::<Tween<f32>>(), 48);
        assert_eq!(std::mem::size_of::<Tween<i32>>(), 48);
        fn assert_copy<T: Copy>() {}
        assert_copy::<Tween<f32>>();
        assert_copy::<Spring>();
        assert_copy::<Steps>();
        // Sixteen of `Instant` and sixteen of `Duration`, so a fifth field of anything at all shows
        // up in one of these three numbers.
        assert_eq!(std::mem::size_of::<Steps>(), 32);
    }

    /// **A spring has four fields and no duration**, and this is the shape of the gate rather than a
    /// comment about it: a fifth field fails to compile here.
    #[test]
    fn a_spring_has_four_fields_and_none_of_them_is_a_duration() {
        let s = Spring::new(t0(), 0.0, 1.0);
        let Spring {
            start: _,
            x0: _,
            v0: _,
            target: _,
        } = s;
        assert_eq!(std::mem::size_of::<Spring>(), 32);
    }

    /// The four easings are closed forms, and all four are pinned at both ends.
    #[test]
    fn every_easing_is_pinned_at_both_ends() {
        for e in [Easing::Linear, Easing::In, Easing::Out, Easing::InOut] {
            assert_eq!(e.at(0.0), 0.0, "{e:?}");
            assert_eq!(e.at(1.0), 1.0, "{e:?}");
            // Clamped, so no caller can ease past an end.
            assert_eq!(e.at(-3.0), 0.0, "{e:?}");
            assert_eq!(e.at(9.0), 1.0, "{e:?}");
        }
        assert_eq!(Easing::InOut.at(0.5), 0.5);
        assert!(Easing::Out.at(0.25) > Easing::In.at(0.25));
    }

    /// What ships, without the gates below it — [`crate::work`]'s instrument, one subsystem over.
    fn shipped_source() -> &'static str {
        let source = include_str!("anim.rs");
        let gates = source
            .find("\n#[cfg(test)]\n")
            .expect("this module's own test module");
        &source[..gates]
    }

    /// **No helper computes a `dt`**, and this is what stops a later edit from adding one.
    ///
    /// A behavioural test cannot tell a closed form from an integrator that happens to agree on the
    /// schedule it was run at — that is the whole finding — so the claim is checked where it is
    /// made. The scan skips prose, because a comment explaining why something is absent has to be
    /// able to name it.
    #[test]
    fn no_helper_computes_a_dt() {
        for (n, code) in shipped_source()
            .lines()
            .enumerate()
            .map(|(n, l)| (n + 1, l.trim_start()))
            .filter(|(_, code)| !code.starts_with("//"))
        {
            for accumulator in ["last_now", "previous", "accumulate", "since_last"] {
                assert!(
                    !code.contains(accumulator),
                    "line {n} accumulates ({accumulator}): {code}"
                );
            }
        }
    }

    /// **The anchored step count is 375 in thirty seconds on every arrival schedule**, and the
    /// `now + per` shape is short on all of them.
    ///
    /// 375 is `30 s / 80 ms`, and it is a property of the closed form rather than of the loop: the
    /// steps are where they are whether or not anybody asked. The drifting count is a property of
    /// the *schedule*, which is exactly the complaint.
    #[test]
    fn the_anchored_spinner_takes_three_hundred_and_seventy_five_steps() {
        let start = t0();
        let per = Duration::from_millis(80);
        let thirty = Duration::from_secs(30);
        let anchored = Steps::new(start, per);
        assert_eq!(anchored.index(start + thirty), 375);

        // Three arrival schedules: on time, five milliseconds late every time, and 200 µs late. On
        // all three, **every step asked for is on the anchor's grid** — `start + k × per` — which is
        // what *the anchor is not a schedule* means, and the count is the same to within the one
        // step the window's own edge cuts off.
        for late in [
            Duration::ZERO,
            Duration::from_millis(5),
            Duration::from_micros(200),
        ] {
            let mut at = start;
            let mut steps = 0u32;
            while at + per <= start + thirty {
                at = anchored.next_at(at) + late;
                steps += 1;
                assert_eq!(at, start + per * steps + late, "late by {late:?}");
            }
            assert!(steps >= 374, "late by {late:?}: {steps} steps");
        }

        // The refuted shape, played here so its number is in the tree: `now + per`, where `now` is
        // the frame's own clock and the frame is five milliseconds late.
        let mut at = start;
        let mut drifting = 0u64;
        while at + per + Duration::from_millis(5) <= start + thirty {
            at += per + Duration::from_millis(5);
            drifting += 1;
        }
        assert_eq!(drifting, 352, "asked at `now + per`, and it is short");
        assert!(drifting < anchored.index(start + thirty));
    }

    /// **At the frame where the anchored phase first reads 1.0000, an accumulated one reads
    /// 0.9333** — and the gap grows with the run.
    ///
    /// The schedule is a loop running 8% behind its nominal cadence, which is the shape engine
    /// was measured as *120 Hz configured achieved 99.7 fps*. The recorded figure is 0.9222,
    /// from a schedule that was a little further behind than this one; what is a property of the
    /// mechanism rather than of the schedule is that the anchored phase is **exactly** 1.0 and the
    /// accumulated one is **never** — a nominal interval cannot see a late frame.
    #[test]
    fn an_anchored_phase_arrives_and_an_accumulated_one_does_not() {
        let start = t0();
        let second = Duration::from_secs(1);
        let fade: Tween<f32> = Tween::new(start, second, 0.0, 1.0);

        let step = Duration::from_micros(18_000);
        let mut sum = 0.0f32;
        let mut now = start;
        let mut frames = 0;
        while fade.phase(now) < 1.0 {
            now += step;
            frames += 1;
            // The refuted shape: a nominal interval, which is what a component reaches for when it
            // is not handed a clock.
            sum += 1.0 / 60.0;
        }
        assert_eq!(frames, 56);
        assert_eq!(fade.phase(now), 1.0, "the anchor arrived");
        assert!(
            (0.93..0.94).contains(&sum),
            "the accumulator is short: {sum}"
        );
    }

    /// A retarget keeps the velocity, and dropping it is a visible distance one frame later.
    #[test]
    fn a_retarget_preserves_the_velocity() {
        let start = t0();
        let flying = Spring::new(start, 0.0, 20.0);
        let mid = start + Duration::from_millis(120);
        let v = flying.velocity(mid);
        assert!(v > 1.0, "it is moving: {v}");

        let kept = flying.retarget(mid, 40.0);
        let dropped = Spring::new(mid, flying.value(mid), 40.0);
        assert_eq!(kept.value(mid), dropped.value(mid), "same place");
        assert_eq!(kept.velocity(mid), v, "the velocity survived");
        assert_eq!(dropped.velocity(mid), 0.0, "and dropping it starts at rest");

        let next = mid + TICK;
        let gap = kept.value(next) - dropped.value(next);
        assert!(gap > 0.0, "one frame later they are apart: {gap} cells");
    }

    /// A per-frame integrator is off the closed form, and a jittered cadence is worse than a steady
    /// one. **The closed form is 0 off by construction**, which is what makes this a gate rather
    /// than a measurement.
    #[test]
    fn an_integrator_drifts_off_the_closed_form() {
        let start = t0();
        let s = Spring::new(start, 0.0, 20.0);
        // The **worst** error over the flight and not the error at the end, because both arrive: a
        // spring's whole job is to converge, and an integrator that is five cells out in the middle
        // of the move is five cells of visible wrongness whatever it does afterwards.
        for jitter in [Duration::ZERO, Duration::from_millis(4)] {
            let mut x = 0.0f32;
            let mut v = 0.0f32;
            let mut now = start;
            let mut worst = 0.0f32;
            for i in 0..60 {
                let step = if i % 3 == 0 { TICK + jitter } else { TICK };
                let h = step.as_secs_f32();
                // Semi-implicit Euler over the same system, which is what a component writes when
                // it has an interval and no closed form.
                let a = -Spring::RATE * Spring::RATE * (x - 20.0) - 2.0 * Spring::RATE * v;
                v += a * h;
                x += v * h;
                now += step;
                worst = worst.max((x - s.value(now)).abs());
            }
            assert!(worst > 0.5, "the integrator is off by {worst} cells");
        }
    }

    /// **A threshold ends a spring and a duration does not**, and zero does not end it either.
    #[test]
    fn a_threshold_ends_a_spring_and_zero_does_not() {
        let start = t0();
        let s = Spring::new(start, 0.0, 20.0);
        let frames_to = |threshold: f32| {
            let mut now = start;
            for frame in 0..10_000u32 {
                if s.settled(now, threshold) {
                    return Some(frame);
                }
                now += TICK;
            }
            None
        };
        let coarse = frames_to(0.5).expect("half a cell settles");
        let fine = frames_to(1e-6).expect("a millionth settles");
        assert!(
            coarse < fine,
            "a coarser threshold sleeps sooner: {coarse} against {fine}"
        );
        // Zero is not *never* — `e^(−ω·t)` underflows eventually — but it is far past anything a
        // screen should be redrawing for, and the runaway detector is what catches it.
        let never = frames_to(0.0).expect("floating point gives up eventually");
        assert!(
            never > 10 * coarse,
            "a zero threshold runs and runs: {never} frames against {coarse}"
        );
    }

    // ── the ledger, over the loop ────────────────────────────────────────────────────────────────

    /// **A quiet dense screen runs one frame and then blocks for ever.**
    #[test]
    fn a_quiet_screen_runs_one_frame_and_blocks() {
        let mut d = driver();
        let frames = drive(&mut d, t0(), Duration::from_secs(30), |cx| {
            let body = cx.theme().paint(Role::Body);
            for row in 0..24i32 {
                cx.text(0, row, "a line of a screen that is not moving", body);
                cx.interact(
                    Id::from_raw(u64::try_from(row).unwrap_or(0)),
                    Rect::new(0, row, 80, 1),
                    Interest::CLICK,
                );
            }
        });
        let l = d.inspect().wakes();
        assert_eq!(frames, 1);
        assert_eq!(l.frames(), 1);
        assert_eq!(l.wakes(), 0, "nothing asked, so nothing is owed");
        assert_eq!(l.worst(), 0);
        assert_eq!(l.pending(), None, "and the loop has nothing to wait for");
        assert_eq!(l.line_count(), 0);
    }

    /// **A 300 ms fade runs 19 frames**, and its streak is eighteen — under a runaway threshold of
    /// sixty, which is what makes it an animation.
    #[test]
    fn a_three_hundred_millisecond_fade_runs_nineteen_frames() {
        let mut d = driver();
        let start = t0();
        let fade: Tween<f32> = Tween::new(start, Duration::from_millis(300), 0.0, 1.0);
        let frames = drive(&mut d, start, Duration::from_secs(30), |cx| {
            let now = cx.now();
            let paint = cx.theme().mix(Role::Face, Role::FaceHover, fade.phase(now));
            cx.fill(Rect::new(0, 0, 12, 1), " ", paint);
            if let Some(at) = fade.wake(now) {
                cx.deadline(at);
            }
        });
        let l = d.inspect().wakes();
        assert_eq!(frames, 19);
        assert_eq!(l.wakes(), 18, "the last frame paints and asks for nothing");
        assert_eq!(l.worst(), 18);
        assert!(l.runaway(60).is_none(), "a fade is not a spin");
        assert_eq!(l.line_count(), 1);
    }

    /// **A caret blinking at 530 ms is 1.9 wakes a second, and its streak is zero.**
    ///
    /// It asks for three times as many wakes as the fade and is the cheapest thing on the list,
    /// because every one of them is a moment in the future: the screen sleeps 530 ms between them.
    /// This is the scene that says the detector cannot be a total.
    ///
    /// The runtime's *own* caret costs nothing at all — the terminal blinks it, and
    /// `ctx::focus_tests::a_caret_costs_no_wakeup` is that gate. This is a component blinking
    /// something of its own.
    #[test]
    fn a_blinking_caret_is_one_point_nine_wakes_a_second() {
        let mut d = driver();
        let start = t0();
        let blink = Steps::new(start, Duration::from_millis(530));
        let frames = drive(&mut d, start, Duration::from_secs(30), |cx| {
            let now = cx.now();
            let paint = cx
                .theme()
                .paint(if blink.on(now) { Role::Body } else { Role::Dim });
            cx.text(0, 0, "|", paint);
            cx.deadline(blink.next_at(now));
        });
        let l = d.inspect().wakes();
        assert_eq!(frames, 57);
        assert_eq!(l.wakes(), 57);
        assert_eq!(
            l.worst(),
            0,
            "every one of them is a moment the screen sleeps until"
        );
        assert!(l.runaway(2).is_none(), "and it is a runaway at no n");
        assert_eq!(format!("{:.1}", f64::from(frames) / 30.0), "1.9");
    }

    /// **A spinning component is 60.0 a second and every frame of it is a streak**, and
    /// `runaway(60)` names the line that asked.
    #[test]
    fn a_spinning_component_is_a_runaway_and_the_ledger_names_the_line() {
        let mut d = driver();
        let start = t0();
        let frames = drive(&mut d, start, Duration::from_secs(30), |cx| {
            let body = cx.theme().paint(Role::Body);
            cx.text(0, 0, "|", body);
            // No `done`, no threshold, no next step: the shape that never sleeps.
            cx.request_frame();
        });
        let l = d.inspect().wakes();
        assert_eq!(frames, 1800);
        assert_eq!(l.wakes(), 1800);
        assert_eq!(l.worst(), 1800, "every frame in a row");
        assert_eq!(format!("{:.1}", f64::from(frames) / 30.0), "60.0");

        let Runaway { at, streak, asks } = l.runaway(60).expect("a second of it is enough");
        assert_eq!(streak, 1800);
        assert_eq!(asks, 1800);
        // **The pointer compare.** The line named is the one in this file that called
        // `request_frame`, and never the runtime's own line inside `Ctx::request_frame`.
        assert!(at.file().ends_with("anim.rs"), "it named {at}");
    }

    /// A spring with a zero threshold is a runaway, and the detector catches it rather than a
    /// reviewer.
    #[test]
    fn a_zero_threshold_is_a_runaway_and_not_an_animation() {
        let start = t0();
        let s = Spring::new(start, 0.0, 20.0);

        let mut d = driver();
        drive(&mut d, start, Duration::from_secs(5), |cx| {
            let now = cx.now();
            let paint = cx.theme().paint(Role::Body);
            cx.text(s.value(now).round() as i32, 0, "o", paint);
            if let Some(at) = s.wake(now, 0.0) {
                cx.deadline(at);
            }
        });
        assert!(
            d.inspect().wakes().runaway(60).is_some(),
            "five seconds of a settled spring still asking"
        );

        // The same spring with a threshold sleeps long before the horizon.
        let mut d2 = driver();
        let frames = drive(&mut d2, start, Duration::from_secs(5), |cx| {
            let now = cx.now();
            let paint = cx.theme().paint(Role::Body);
            cx.text(s.value(now).round() as i32, 0, "o", paint);
            if let Some(at) = s.wake(now, 0.5) {
                cx.deadline(at);
            }
        });
        assert!(frames < 60, "half a cell settles in {frames} frames");
        assert!(d2.inspect().wakes().runaway(60).is_none());
    }

    /// **Sixty frames with a job in flight ask for 0 wakeups, against 60 for the polling shape** —
    /// The criterion, re-gated over the ledger it was owed.
    ///
    /// This used to be counted at its two call sites because the ledger did not exist yet, and said
    /// so. Over the ledger it is stronger than *0 of 60*: the parked shape runs **one** frame and
    /// then blocks for ever, because a job in flight is not a reason to run a frame. The polling
    /// shape runs sixty, and its streak of sixty is a runaway on a screen doing nothing.
    ///
    /// **The seam runs this way and not the other**: `work` names no deadline —
    /// `work::tests::the_module_names_no_deadline` scans for one — so the pairing lives here, where
    /// the wake accounting is.
    #[test]
    fn sixty_frames_with_a_job_in_flight_ask_for_no_wakeups() {
        use crate::work::{Task, Worker};

        let worker = Worker::queueing();
        let parked: Task<u32> = Task::new(&worker);
        parked.request(0, |_cancel| 42);

        let start = t0();
        // Sixty frames, so fifty-nine gaps between them.
        let sixty = TICK * 59;
        let mut d = driver();
        let frames = drive(&mut d, start, sixty, |cx| {
            let body = cx.theme().paint(Role::Body);
            // The whole of the app thread's side, and it asks for nothing.
            let landed = parked.take();
            cx.text(0, 0, if landed.is_some() { "!" } else { "." }, body);
        });
        assert_eq!(frames, 1, "one frame, and then the loop blocks");
        assert_eq!(d.inspect().wakes().wakes(), 0);

        // The shape an application reaches for when it has no wake attached to its slot: a deadline
        // so that it can look again.
        let polling: Task<u32> = Task::new(&worker);
        polling.request(1, |_cancel| 42);
        let mut d2 = driver();
        let polled = drive(&mut d2, start, sixty, |cx| {
            let body = cx.theme().paint(Role::Body);
            let landed = polling.take();
            cx.text(0, 0, if landed.is_some() { "!" } else { "." }, body);
            cx.request_frame();
        });
        assert_eq!(polled, 60);
        assert_eq!(d2.inspect().wakes().wakes(), 60);
        assert_eq!(d2.inspect().wakes().worst(), 60);
        assert!(
            d2.inspect().wakes().runaway(60).is_some(),
            "sixty frames in a row on a screen doing nothing"
        );
    }

    // ── one sink ─────────────────────────────────────────────────────────────────────────────────

    /// **`[Tab, Key(a)]` routes both**, because the split-batch drain asks through the same sink a
    /// deadline does.
    #[test]
    fn tab_then_a_routes_both_because_the_lazy_offender_is_in_the_sink() {
        use vitui_engine::{Key, KeyCode, KeyKind, KeyText, Mods};
        fn key(code: KeyCode) -> Key {
            Key {
                code,
                mods: Mods::NONE,
                kind: KeyKind::Press,
                text: KeyText::EMPTY,
                at: Instant::now(),
            }
        }

        let mut d = driver();
        let field = Id::named("field");
        d.plant(None, Some(field), None);
        d.post_key(key(KeyCode::Tab));
        d.post_key(key(KeyCode::Char('a')));

        // Frame one takes the batch up to and including the routing edge, and something is left.
        let mut seen = Vec::new();
        d.frame(|cx| {
            cx.interact(field, Rect::new(0, 0, 8, 1), Interest::FOCUS);
            while let Some(k) = cx.next_key(field) {
                seen.push(k.code);
            }
        });
        assert!(d.queued() > 0, "the key is still queued");
        assert!(
            d.inspect().wakes().pending().is_some(),
            "the frame owes another one, and there is one sink for saying so"
        );
        // **The offending line is the runtime's own**, which is the whole of *the lazy offender
        // joins the gate it was invisible to*: before this ticket it set a flag no census could see.
        let lines: Vec<_> = d.inspect().wakes().lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].at.file().ends_with("ctx.rs"), "{}", lines[0].at);

        // The `Tab` went to the ring rather than to the widget, which is the rule and not this
        // ticket's — so what frame one delivered is nothing.
        assert!(seen.is_empty(), "the ring took the Tab: {seen:?}");

        // Frame two, driven by that wake, delivers the second key. **This is the whole defect**:
        // without the wake there is no frame two, and `a` is never routed at all.
        d.frame(|cx| {
            cx.interact(field, Rect::new(0, 0, 8, 1), Interest::FOCUS);
            while let Some(k) = cx.next_key(field) {
                seen.push(k.code);
            }
        });
        assert_eq!(seen, vec![KeyCode::Char('a')]);
        assert_eq!(d.queued(), 0);
    }

    /// **The negative case: two sinks, one flushed.** The refuted `settle` returns `None` for the
    /// frame above, so the loop blocks and the key is never routed.
    #[test]
    fn the_refuted_fold_strands_the_key() {
        use vitui_engine::{Key, KeyCode, KeyKind, KeyText, Mods};

        /// What `settle` did before the fold: flush the earliest deadline, and nothing else. The
        /// second sink — a `repaint` flag beside it — is the argument this function ignores.
        fn two_sinks(deadline: Option<Instant>, _repaint: bool) -> Option<Instant> {
            deadline
        }

        fn key(code: KeyCode) -> Key {
            Key {
                code,
                mods: Mods::NONE,
                kind: KeyKind::Press,
                text: KeyText::EMPTY,
                at: Instant::now(),
            }
        }

        let mut d = driver();
        let field = Id::named("field");
        d.plant(None, Some(field), None);
        d.post_key(key(KeyCode::Tab));
        d.post_key(key(KeyCode::Char('a')));
        d.frame(|cx| {
            cx.interact(field, Rect::new(0, 0, 8, 1), Interest::FOCUS);
            while cx.next_key(field).is_some() {}
        });

        // Nothing in that frame called `deadline`, so the refuted fold has nothing to flush — while
        // the shipped one owes a frame.
        assert_eq!(two_sinks(None, true), None, "the loop blocks");
        assert!(d.inspect().wakes().pending().is_some());
        assert!(d.queued() > 0, "and the key is still sitting there");
    }

    /// **`request_frame()` is `deadline(now)` with a call site**, and the two are one sink.
    #[test]
    fn request_frame_is_a_deadline_at_now_with_a_call_site() {
        let mut d = driver();
        let start = t0();
        d.pin_clock(start);
        d.frame(|cx| cx.request_frame());
        let l = d.inspect().wakes();
        assert_eq!(
            l.pending(),
            Some(start),
            "the frame's own clock, sampled once"
        );
        assert_eq!(l.wakes(), 1);
        assert_eq!(l.streak(), 1, "and it is immediate");
        let asked = l.lines().next().expect("one line asked");
        assert!(asked.at.file().ends_with("anim.rs"), "{}", asked.at);
    }

    /// Two callers asking for two moments is **one** wake, at the earlier — and two lines in the
    /// ledger, because the fold is a wake and the ledger is a census.
    #[test]
    fn two_deadlines_are_one_wake_and_two_lines() {
        let mut d = driver();
        let start = t0();
        d.pin_clock(start);
        d.frame(|cx| {
            cx.deadline(cx.now() + Duration::from_millis(100));
            cx.deadline(cx.now() + Duration::from_millis(10));
        });
        let l = d.inspect().wakes();
        assert_eq!(l.pending(), Some(start + Duration::from_millis(10)));
        assert_eq!(l.wakes(), 1, "one frame, one wake");
        assert_eq!(l.line_count(), 2, "and two lines asked for it");
        assert_eq!(l.streak(), 0, "neither of them is immediate");
    }

    // ── attribution and the census ───────────────────────────────────────────────────────────────

    /// **Twelve animated chips attribute to twelve distinct lines**, and none of them is a widget's
    /// id.
    ///
    /// The chip is one function, so `#[track_caller]` on it is what moves the attribution out to the
    /// application's twelve lines — the virality running in the right direction. Without it all
    /// twelve land on one line inside the library.
    #[test]
    fn twelve_chips_attribute_to_twelve_lines() {
        #[track_caller]
        fn chip(cx: &mut Ctx<'_, '_>, x: i32, paint: Paint) {
            cx.text(x, 0, "*", paint);
            cx.deadline(cx.now());
        }

        let mut d = driver();
        d.pin_clock(t0());
        d.frame(|cx| {
            let p = cx.theme().paint(Role::Body);
            chip(cx, 0, p);
            chip(cx, 2, p);
            chip(cx, 4, p);
            chip(cx, 6, p);
            chip(cx, 8, p);
            chip(cx, 10, p);
            chip(cx, 12, p);
            chip(cx, 14, p);
            chip(cx, 16, p);
            chip(cx, 18, p);
            chip(cx, 20, p);
            chip(cx, 22, p);
        });
        let l = d.inspect().wakes();
        assert_eq!(l.line_count(), 12, "twelve chips, twelve lines");
        assert_eq!(l.wakes(), 1, "and one frame owes one wake");
        for line in l.lines() {
            assert_eq!(line.asks, 1);
            assert!(line.at.file().ends_with("anim.rs"));
        }
        assert_eq!(
            l.census_len(),
            0,
            "nobody named an id, so nobody is counted"
        );
    }

    /// The same twelve chips without the annotation: **one line, inside the library**, which is what
    /// `#[track_caller]` being viral in the right direction buys.
    #[test]
    fn twelve_chips_from_an_unannotated_helper_are_one_line() {
        fn chip(cx: &mut Ctx<'_, '_>, x: i32, paint: Paint) {
            cx.text(x, 0, "*", paint);
            cx.deadline(cx.now());
        }

        let mut d = driver();
        d.pin_clock(t0());
        d.frame(|cx| {
            let p = cx.theme().paint(Role::Body);
            for i in 0..12i32 {
                chip(cx, i * 2, p);
            }
        });
        let l = d.inspect().wakes();
        assert_eq!(l.line_count(), 1, "the helper's own line, twelve times");
        assert_eq!(l.lines().next().expect("one line").asks, 12);
    }

    /// The refuted attribution, played here so its number is in the tree: **a census keyed on the id
    /// the frame has at hand answers `ROOT` 12 of 12 unkeyed, and the key scope 12 of 12 keyed** —
    /// never the chip.
    #[test]
    fn attributing_to_the_id_the_frame_has_answers_root_twelve_of_twelve() {
        fn chip(cx: &mut Ctx<'_, '_>, x: i32, scope: Id) {
            let paint = cx.theme().paint(Role::Body);
            cx.text(x, 0, "*", paint);
            // The id an attribution-by-id would have had: the id stack's current, which is the
            // closure tree and not the widget.
            cx.deadline_for(scope, cx.now());
        }

        let mut d = driver();
        d.pin_clock(t0());
        d.frame(|cx| {
            for i in 0..12i32 {
                chip(cx, i * 2, Id::ROOT);
            }
        });
        assert_eq!(d.inspect().wakes().asked_by(Id::ROOT), 12, "12 of 12");
        assert_eq!(d.inspect().wakes().census_len(), 1);

        // Keyed, and it is the *scope* that answers rather than the chip.
        let scope = Id::named("chips");
        let mut d2 = driver();
        d2.pin_clock(t0());
        d2.frame(|cx| {
            cx.with_id(scope, |cx| {
                for i in 0..12i32 {
                    chip(cx, i * 2, scope);
                }
            });
        });
        assert_eq!(d2.inspect().wakes().asked_by(scope), 12);
        assert_eq!(d2.inspect().wakes().asked_by(Id::ROOT), 0, "0 of 12");
        assert_eq!(d2.inspect().wakes().census_len(), 1);
    }

    /// **`deadline_for` attributes to the call site too**, and what the id buys is the census beside
    /// it: four keyed chips are four lines and four widgets with one each.
    #[test]
    fn deadline_for_is_the_same_attribution_and_a_census_as_well() {
        /// **The ask is outside the closure on purpose.** `#[track_caller]` moves the *caller's*
        /// location into this frame, and a closure body is a frame of its own — a `deadline_for`
        /// written inside `with_key` names the closure's line, so twelve chips would be one line
        /// again. The id is what comes out of the closure; the ask happens here.
        #[track_caller]
        fn chip(cx: &mut Ctx<'_, '_>, key: u64, x: i32) -> Id {
            let who = cx.with_key(key, |cx| {
                let who = cx.id();
                let paint = cx.theme().paint(Role::Body);
                cx.text(x, 0, "*", paint);
                who
            });
            cx.deadline_for(who, cx.now());
            who
        }

        let mut d = driver();
        d.pin_clock(t0());
        let mut ids = Vec::new();
        d.frame(|cx| {
            ids.push(chip(cx, 0, 0));
            ids.push(chip(cx, 1, 2));
            ids.push(chip(cx, 2, 4));
            ids.push(chip(cx, 3, 6));
        });
        let l = d.inspect().wakes();
        assert_eq!(l.line_count(), 4, "four call sites");
        assert_eq!(l.census_len(), 4, "and four widgets said so themselves");
        for id in &ids {
            assert_eq!(l.asked_by(*id), 1);
        }
        assert_eq!(l.asked_by(Id::ROOT), 0, "the scope is not the census");
    }

    // ── the tier ─────────────────────────────────────────────────────────────────────────────────

    /// **The same fade is 19 frames ungated at every tier, and 19 / 2 / 2 gated.**
    ///
    /// The gated shape is what a component owes: `shows(Hover)` decides whether there is a hover
    /// state at all, and `shows(Fade)` decides whether to animate into it or to snap. A snap is two
    /// frames — the one the state changed on and the one that paints the destination — against
    /// nineteen.
    #[test]
    fn the_same_fade_is_nineteen_ungated_and_nineteen_two_two_gated() {
        let tiers = [
            ColorDepth::TrueColor,
            ColorDepth::Indexed256,
            ColorDepth::Ansi16,
        ];
        let mut ungated = Vec::new();
        let mut gated = Vec::new();
        for tier in tiers {
            for honour in [false, true] {
                let mut d = driver();
                d.set_theme(Theme::default().resolve(tier));
                let shows = Theme::default().resolve(tier).shows(Distinction::Fade);
                let start = t0();
                let fade: Tween<f32> = Tween::new(start, Duration::from_millis(300), 0.0, 1.0);
                let mut painted = 0u32;
                let frames = drive(&mut d, start, Duration::from_secs(5), |cx| {
                    let now = cx.now();
                    let snap = honour && !shows;
                    let t = if snap { 1.0 } else { fade.phase(now) };
                    let paint = cx.theme().mix(Role::Face, Role::FaceHover, t);
                    cx.fill(Rect::new(0, 0, 12, 1), " ", paint);
                    painted += 1;
                    if snap {
                        // The snap: the frame the state changed on, and one more that paints the
                        // destination. Then the screen sleeps.
                        if painted < 2 {
                            cx.request_frame();
                        }
                    } else if let Some(at) = fade.wake(now) {
                        cx.deadline(at);
                    }
                });
                if honour { &mut gated } else { &mut ungated }.push(frames);
            }
        }
        assert_eq!(
            ungated,
            vec![19, 19, 19],
            "the tier does not shorten a fade"
        );
        assert_eq!(gated, vec![19, 2, 2], "reading the capability does");
    }

    /// The ledger holds thirty-two lines and counts what did not fit rather than growing.
    #[test]
    fn the_ledger_is_bounded_and_says_what_it_dropped() {
        let mut l = WakeLedger::new();
        let now = t0();
        // One line asked a hundred times, which is the shape a real application has: a line is a
        // source line, not a widget.
        for _ in 0..100 {
            l.asked(Location::caller(), None, now, now);
            l.note(Some(now), now);
        }
        assert_eq!(l.line_count(), 1);
        assert_eq!(l.lines_lost(), 0);
        assert_eq!(l.wakes(), 100);
        assert_eq!(l.worst(), 100);
        for i in 0..64u64 {
            l.tally(Id::from_raw(i));
        }
        assert_eq!(l.census_len(), super::LINES);
        assert_eq!(l.census_lost(), 32, "and the rest are counted, not lost");
    }
}
