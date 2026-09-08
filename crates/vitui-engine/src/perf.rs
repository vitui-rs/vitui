//! The unblockable app thread: the in-loop detector, the escape hatch, and the debug observer.
//!
//! > **The compiler cannot stop the app thread from being slow. It can stop anything else from being
//! > the app thread.**
//!
//! Four of the five rungs are elsewhere and cost nothing: `Screen` and `View` are `!Send` by a
//! private `PhantomData<*const ()>`, there is no blocking primitive on the app-thread side at all
//! ([`crate::slot`]), and the lint fragment belongs to the application author
//! (`examples/app-template/`). **This file is the two that cost something**, and the offence they
//! answer to is this crate's rather than Android's:
//!
//! > a **frame-budget overrun by the app thread's iteration, whatever caused it** — not a blocking
//! > syscall.
//!
//! `NetworkOnMainThreadException` picks the syscall and therefore catches a DNS lookup while waving
//! through a `for` loop that takes 400 ms. A frozen interface is a frozen interface, and a pure slow
//! function freezes it identically, so the syscall set survives here as the *lintable subset* of the
//! definition and never as the definition.
//!
//! # Why the two live together
//!
//! The in-loop detector cannot, by construction, see the failure it exists for: **an iteration that
//! never returns never reports.** Only an awake observer can, and being awake is exactly what
//! standing requirement 11 forbids — measured over thirty seconds, no observer costs 10 context
//! switches, a 100 ms poll costs 369 and a 10 ms poll 2 792. CPU is not the objection; *wakeups*
//! are. So the observer is `cfg(debug_assertions)` only and requirement 11 stays a release-build
//! property, which `scripts/observer-gate.sh` checks by looking for the observer's own words in a
//! release binary and finding them in a debug one.
//!
//! # The two sanctions are different, and not because debug is stricter
//!
//! | | who notices | sanction |
//! |---|---|---|
//! | in-loop | the app thread itself, at `present` | debug: **panic** on the first overrun. Release: **warn once**, into a sink the caller supplies |
//! | observer | another thread, at a poll | **restore the terminal, print, abort** — in debug only |
//!
//! A panic is safe *by construction* rather than by luck: the restoration is idempotent,
//! guarded by one atomic, callable from any thread, and runs before the default hook prints. So the
//! in-loop sanction needs no `catch_unwind` and no cooperation.
//!
//! **The observer may not panic**, and that is the whole reason its sanction is spelled differently:
//! a panic on the observer's thread unwinds the observer's stack, which stops nothing — the app
//! thread is still inside the iteration that has not come back, and a *returning* app thread would
//! then paint frames into a terminal somebody had already restored. Printing without restoring puts
//! the message in an alt screen that is about to be discarded. So it is restore, print, abort, in
//! that order, and the abort is the point: there is no state left worth unwinding to.
//!
//! Panicking on a *sustained* overrun rather than the first was rejected: it hides the single 300 ms
//! file read that is exactly the bug being hunted, and it needs a tuned N for which there is no
//! measurement.
//!
//! # Nothing here reaches into the crate
//!
//! Not tidiness — `examples/budget.rs` `#[path]`-includes this file so that the overrun gate can
//! time [`Perf::enter`] and [`Perf::leave`] **in a release build**, which is the only profile the
//! 50 ns figure means anything in, and a single `use crate::` here would drag the whole engine into
//! that binary. The one thing this module cannot do for itself — give the terminal back — arrives as
//! a `fn()` pointer, which is also what keeps it honest: the observer restores through
//! `crate::shutdown`'s process-global site, the same door the panic hook uses, rather than through a
//! `Screen` it could not hold anyway.

use std::cell::{Cell, RefCell};
use std::io::Write;
use std::rc::Rc;
#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(debug_assertions)]
use std::sync::{Arc, Mutex, PoisonError};
use std::time::{Duration, Instant};

/// The rate a threshold falls back to when the application declared none it can be derived from.
///
/// The same number as `Config::DEFAULT_MAX_FRAME_RATE`, and deliberately a second copy of it rather
/// than a reference: this file reaches for nothing in the crate (see the module docs), and the two
/// would have to be compared by hand if they ever disagreed — which
/// `crate::gates::the_detectors_default_threshold_is_the_configs_default_rate` is what does.
pub(crate) const DEFAULT_HZ: f32 = 60.0;

/// How many frame intervals an iteration may be inside before another thread calls it a stall.
///
/// **The observer is not a second overrun detector.** Everything that overruns *and returns* is
/// already reported by the in-loop detector — in debug, by a panic — so the only thing left for the
/// observer is an iteration that has not come back at all, and the limit for that is a different
/// number by two orders of magnitude.
///
/// 64 rather than 100 for two reasons, and both are about being noticed. At the default rate it puts
/// the limit at **1.07 s**, which is past every legitimate frame this crate has ever measured — the
/// worst realistic `wake → submit` (`ledger::realistic_iteration_ns`) leaves 6 400x of
/// headroom against it — and still inside the
/// span a developer stares at a frozen window before reaching for Ctrl-C. And it keeps the limit
/// **caller-configurable through the same knob as the threshold**, so a test can drive the whole
/// sanction in 128 ms rather than in a second.
#[cfg(debug_assertions)]
pub(crate) const STALL_FACTOR: u32 = 64;

/// How many times the observer looks inside one stall limit.
///
/// Eight, so that a stall is reported promptly rather than sampled: at one poll per limit the report
/// arrives anywhere between immediately and one whole limit late, which on a 1.07 s limit is the
/// difference between a diagnostic and a mystery.
#[cfg(debug_assertions)]
const POLL_DIVISOR: u32 = 8;

/// The shortest poll the observer will take, whatever the limit says.
///
/// A limit small enough to ask for less than this is a test's, and a test does not need the observer
/// to be quicker than a millisecond.
#[cfg(debug_assertions)]
pub(crate) const POLL_FLOOR: Duration = Duration::from_millis(1);

/// The longest poll the observer will take, and the number requirement 11 was priced against.
///
/// 100 ms is 369 context switches over thirty seconds, against 10 for a process with no observer at
/// all. That is the cost this rung has, it is why the rung is debug-only, and it is why the ceiling
/// is not 10 ms — which is 2 792.
#[cfg(debug_assertions)]
pub(crate) const POLL_CEILING: Duration = Duration::from_millis(100);

/// Nothing excused, for the arms that have no permit in them.
///
/// `debug_assertions` as well as `test`, for the reason on the test module below: the arms that use
/// it are the debug-only ones, and a `cfg(test)` alone made `cargo test --release` fail to compile.
#[cfg(all(test, debug_assertions))]
const ZERO: Duration = Duration::ZERO;

/// One frame interval at `hz`, or exactly what the caller pinned.
///
/// **The map's < 100 µs is a CI gate on one stage of the engine's own work, not a runtime threshold
/// for the application's whole iteration**, and that distinction is the whole of why this function
/// exists. A full realistic `wake → submit` — drawing, compositing under eight layers, packing —
/// is `ledger::realistic_iteration_ns`, so a 100 µs watchdog fires on entirely legitimate
/// frames. One frame interval is 8.3 ms at 120 Hz, which makes that iteration 1.0% of it: 100x of
/// headroom and no false positives. An overrun then means a **dropped frame**, which is the event a
/// user can actually perceive.
///
/// # An unlimited rate still has a threshold
///
/// `f32::INFINITY` says *do not pace me*, and pacing is not what this number is about — a user's eye
/// does not become more patient because the application stopped asking for a gap. So an unlimited or
/// nonsensical rate falls back to [`DEFAULT_HZ`] rather than switching the detector off, and an
/// application that really wants it off pins `Duration::MAX`.
///
/// Computed in nanoseconds by hand rather than through `Duration::try_from_secs_f64`, which is what
/// `crate::clock::gap_for` uses: the truncation is the conservative direction, and this file may not
/// name that function (see the module docs).
pub(crate) fn threshold_for(hz: f32, pinned: Option<Duration>) -> Duration {
    if let Some(pinned) = pinned {
        return pinned;
    }
    let interval = |hz: f32| Duration::from_nanos((1e9 / f64::from(hz)) as u64);
    if !hz.is_finite() || hz <= 0.0 {
        return interval(DEFAULT_HZ);
    }
    let asked = interval(hz);
    // **A rate whose interval rounds to zero is unlimited, not instantaneous.** `max_frame_rate` is a
    // public `f32`, so `1e10` is reachable — out of somebody's arithmetic as easily as on purpose —
    // and it truncates to a zero threshold, against which *every* iteration overruns and a debug build
    // panics on its first frame. `crate::clock::gap_for` guards the mirror-image edge for the
    // mirror-image reason, and a rate asking for under a nanosecond a frame is indistinguishable from
    // `f32::INFINITY` for every purpose either of them has.
    match asked.is_zero() {
        true => interval(DEFAULT_HZ),
        false => asked,
    }
}

/// The shortest a stall limit may be, however tight a budget the application pinned.
///
/// **The observer's limit is not the frame budget, and this is where that stops being a factor and
/// becomes a floor.** The limit answers *how long may a window be frozen before a human is looking at
/// a hung program*, and that span does not shrink because an application declared a tighter frame
/// budget — pinning 1 ms asks for a stricter *detector*, not for a process that aborts after 64 ms.
///
/// It is also what keeps the abort out of every gate that pins a tight threshold to exercise the
/// in-loop rung. Without it, a test that sleeps 20 ms against a 1 ms budget is 3.2x away from
/// aborting the whole test binary on a loaded runner — a flaky gate in a worse costume than the one
/// this repository already refuses, because the failure it reports is a stall rather than the
/// assertion that actually broke.
///
/// And it is what keeps [`POLL_DIVISOR`]'s promise true after a rate change: the poll is computed
/// once at attach and clamped to 100 ms, so a limit that can never be under a second is always at
/// least ten polls wide however the rate moves afterwards.
#[cfg(debug_assertions)]
pub(crate) const STALL_FLOOR: Duration = Duration::from_secs(1);

/// How long an iteration may be inside before another thread calls it a stall.
#[cfg(debug_assertions)]
fn stall_limit(threshold: Duration) -> Duration {
    threshold.saturating_mul(STALL_FACTOR).max(STALL_FLOOR)
}

/// How often the observer looks, for a given limit.
#[cfg(debug_assertions)]
fn poll_for(limit: Duration) -> Duration {
    (limit / POLL_DIVISOR).clamp(POLL_FLOOR, POLL_CEILING)
}

/// What the iteration cost after everything a permit excused, when that is over budget.
///
/// The one decision the whole in-loop rung makes, as a function of four values and nothing else —
/// which is what lets it be tested at every edge without a `Screen`, a thread or a clock.
///
/// `None` when there is no iteration to judge. **A `present` with no `wait` before it is not an
/// iteration**, and that is not a special case for tests: `Clock::Manual` exists precisely so that a
/// deterministic program can draw and present in a straight line with nothing parking anywhere, and
/// a detector that charged those for the wall-clock time since the last frame would report the
/// programmer's own `sleep` between two asserts.
fn overrun_by(
    entered: Option<Instant>,
    now: Instant,
    excused: Duration,
    threshold: Duration,
) -> Option<Duration> {
    let charged = now
        .saturating_duration_since(entered?)
        .saturating_sub(excused);
    (charged > threshold).then_some(charged)
}

/// A duration a human reads, at three significant figures and never in scientific notation.
fn human(d: Duration) -> String {
    let ns = d.as_nanos();
    match ns {
        0..1_000 => format!("{ns} ns"),
        1_000..1_000_000 => format!("{:.1} µs", d.as_secs_f64() * 1e6),
        1_000_000..1_000_000_000 => format!("{:.1} ms", d.as_secs_f64() * 1e3),
        _ => format!("{:.1} s", d.as_secs_f64()),
    }
}

/// What the in-loop detector says, in one line, whichever profile is going to say it.
///
/// One function for the panic and for the warning, because *they are the same finding* and two
/// copies of this sentence would quietly stop agreeing. It names what a bug report needs and nothing
/// else: the number, the budget it is against, and whether the application had already said it
/// expected this.
fn overrun_message(charged: Duration, threshold: Duration, reason: Option<&str>) -> String {
    let excuse = match reason {
        Some(reason) => format!(", inside a permit for {reason:?}"),
        None => String::new(),
    };
    format!(
        "vitui: the app thread's iteration took {} against a {} frame budget{excuse} — a frame was \
         dropped. Move the work to another thread and post a wake, or declare it with \
         `Screen::permit_slow`.",
        human(charged),
        human(threshold),
    )
}

/// What the observer says before it aborts.
///
/// The entry time is *how long ago* rather than a wall clock, because that is the number that
/// identifies the frame: an application that has been up for four hours does not need to subtract.
#[cfg(debug_assertions)]
fn stall_report(inside: Duration, reason: Option<&str>) -> String {
    let excuse = match reason {
        Some(reason) => format!("permitted as {reason:?}"),
        None => String::from("no permit was held"),
    };
    format!(
        "vitui: the app thread entered an iteration {} ago and has not come back ({excuse}). The \
         terminal has been restored and this process is being aborted, because an app thread that \
         returns after a restoration paints cells onto the user's shell.",
        human(inside),
    )
}

/// The in-loop detector, and the only thing on the app thread that knows how long a frame took.
///
/// # It is not merely unforgettable, it is uncallable
///
/// The split handles are **internal** after the seam ticket: `attach` spawns the threads and
/// `present` owns the sequence, so `Parker`, `Producer` and `Consumer` are not public names and
/// [`Perf::enter`] and [`Perf::leave`] are reachable from exactly two places —
/// [`Screen::wait`](crate::Screen::wait) and [`Screen::present`](crate::Screen::present). A runtime
/// author cannot forget to call them, cannot call them in the wrong order, and cannot call them at
/// all. The enforcement is *strengthened* by the split becoming private, which is the opposite of
/// what hiding a mechanism usually does to it.
///
/// # Why `Cell` and `&self` throughout
///
/// Precedence rule 4 asks for it anyway, and here the compiler insists. `permit_slow` has to be
/// callable from inside the region it excuses, and the region draws — so nothing on this type may
/// hold a `&mut`. The first shape of the escape hatch was `permit_slow(&mut self) -> Permit<'_>` and
/// it did not compile: the guard holds the mutable borrow for its whole lifetime, so `enter` and
/// `leave` are unreachable inside the very region the permit exists for (`E0499`). `Cell` fixes
/// that half. See [`Permit`] for the half it does not fix, which the borrow checker found next.
pub(crate) struct Perf {
    /// When the current iteration entered, or `None` outside one — which is what a `present` with no
    /// `wait` before it is, and it is charged to nothing.
    entered: Cell<Option<Instant>>,
    /// How much of the current iteration a [`Permit`] has already accounted for. Reset at `enter`.
    excused: Cell<Duration>,
    /// When the outermost live [`Permit`] began, or `None` when none is held.
    ///
    /// **A permit still held at `present` has to be accounted for, and the first shape of this did
    /// not do that** — the arithmetic was all in `Drop`, so a guard the caller kept until after the
    /// frame excused nothing at all and the acceptance test *a frame is drawn inside the permitted
    /// region* panicked with the permit's own reason printed in the message. Which is the useful
    /// shape of that bug: the diagnostic named the thing that was supposed to be excusing it.
    ///
    /// So `leave` settles the open span and moves this marker forward, and `Drop` accounts for
    /// whatever is left. A permit that spans two frames is charged to both, each for its own share.
    open: Cell<Option<Instant>>,
    /// How many permits are live. Nesting settles once, at the outermost, rather than once per
    /// guard — which is what keeps two nested permits from excusing their overlap twice.
    depth: Cell<u32>,
    /// One frame interval, or what the caller pinned.
    threshold: Cell<Duration>,
    /// Whether the caller pinned it, in which case a rate change may not move it.
    pinned: bool,
    /// Why the innermost live permit says this region is slow. For the message, and nothing else
    /// reads it — the observer has its own copy, because a `Cell` does not cross a thread.
    reason: Cell<Option<&'static str>>,
    /// Whether the release sanction has been spent. **Once per process, not once per frame.**
    warned: Cell<bool>,
    /// Where the release sanction goes, and `None` is silence rather than stderr: a full-screen
    /// application's stderr is the terminal, and a line printed into the alt screen corrupts the
    /// frame that is on it. An application that wants the diagnostic supplies somewhere for it.
    report: RefCell<Option<Box<dyn Write + Send>>>,
    /// What the observer thread reads. **Absent from a release build entirely** — the field, the
    /// type, the loop and the words it would have printed — which is the observer gate and what
    /// `scripts/observer-gate.sh` reads a binary to check.
    #[cfg(debug_assertions)]
    watch: Arc<Watch>,
}

impl Perf {
    /// A detector for a screen whose application declared `hz`, reporting into `report`.
    ///
    /// An `Rc` because [`Permit`] holds one: see [`Permit`] for why it cannot borrow instead.
    pub(crate) fn new(
        hz: f32,
        pinned: Option<Duration>,
        report: Option<Box<dyn Write + Send>>,
    ) -> Rc<Perf> {
        let threshold = threshold_for(hz, pinned);
        Rc::new(Perf {
            entered: Cell::new(None),
            excused: Cell::new(Duration::ZERO),
            open: Cell::new(None),
            depth: Cell::new(0),
            threshold: Cell::new(threshold),
            pinned: pinned.is_some(),
            reason: Cell::new(None),
            warned: Cell::new(false),
            report: RefCell::new(report),
            #[cfg(debug_assertions)]
            watch: Arc::new(Watch::new(stall_limit(threshold))),
        })
    }

    /// The app thread's iteration has begun. Called by `wait` **after it returns**, and by nothing
    /// else.
    ///
    /// One `Instant::now()` and two `Cell` stores in a release build — the two atomic stores below
    /// are the observer's and do not exist there.
    pub(crate) fn enter(&self) {
        let now = Instant::now();
        self.entered.set(Some(now));
        self.excused.set(Duration::ZERO);
        #[cfg(debug_assertions)]
        self.watch.entered(now);
    }

    /// The iteration is over. Called by `present` on **every** path out of it, and by nothing else.
    ///
    /// Every path, including the ones that submit nothing: a frame refused for a resize, or folded
    /// into the next one because the renderer was busy, still spent the app thread's iteration and
    /// still froze the interface for however long it took.
    pub(crate) fn leave(&self) {
        let Some(entered) = self.entered.take() else {
            // Not an iteration, so not even an `Instant::now()`.
            return;
        };
        let now = Instant::now();
        #[cfg(debug_assertions)]
        self.watch.left();
        // **A permit still held here has already excused what it covered.** Settling it now rather
        // than only at `Drop` is what lets a frame be drawn *inside* a permitted region, which is
        // the whole point of the guard; moving the marker to `now` is what stops the same span being
        // excused again on the next iteration.
        if let Some(at) = self.open.get() {
            self.excuse(now.saturating_duration_since(at));
            self.open.set(Some(now));
        }
        if let Some(charged) =
            overrun_by(Some(entered), now, self.excused.get(), self.threshold.get())
        {
            self.sanction(charged);
        }
    }

    /// A monitor changed under the running program, so the budget did too.
    ///
    /// A **pinned** threshold is not moved. An application that named a number wants that number,
    /// and silently rescaling it on a `set_max_frame_rate` would make the one explicit knob here the
    /// least predictable one.
    pub(crate) fn set_rate(&self, hz: f32) {
        if self.pinned {
            return;
        }
        let threshold = threshold_for(hz, None);
        self.threshold.set(threshold);
        #[cfg(debug_assertions)]
        self.watch.set_limit(stall_limit(threshold));
    }

    /// What a [`Permit`] accounted for, added to whatever earlier permits in this iteration did.
    ///
    /// Nested permits therefore over-excuse rather than under-excuse, and that is the direction to
    /// err in: over-excusing loses a diagnostic, under-excusing panics a debug build on a frame that
    /// did nothing wrong.
    fn excuse(&self, spent: Duration) {
        self.excused.set(self.excused.get().saturating_add(spent));
    }

    /// Both sanctions, because they are one finding.
    ///
    /// The warning goes out **before** the panic rather than instead of it: a panic message reaches
    /// whoever is reading a backtrace, and the caller's sink is what reaches a bug report.
    fn sanction(&self, charged: Duration) {
        // **Before the formatting, not after it, and this is the frame path.** `overrun_message`
        // builds three or four `String`s and `warn_once` throws all but the first away — so a release
        // application with a 300 ms read in its loop would allocate on **every** frame, for ever, to
        // produce a line nobody will ever see. The existing allocation gate cannot catch that:
        // `tests/alloc.rs` runs on `Clock::Manual` and never calls `wait`, so `leave` returns before
        // it reaches here. See [`Perf::will_report`].
        if !self.will_report() {
            return;
        }
        let message = overrun_message(charged, self.threshold.get(), self.reason.get());
        self.warn_once(&message);
        #[cfg(debug_assertions)]
        panic!("{message}");
    }

    /// Whether an overrun found now would produce any output at all.
    ///
    /// The guard that keeps the frame path allocation-free after the first overrun, and its own
    /// function so that it can have its own test — the property is *the second overrun formats
    /// nothing*, and nothing about the sink can say that. Always true in a debug build, where the
    /// first overrun panics and there is therefore never a second.
    fn will_report(&self) -> bool {
        cfg!(debug_assertions) || !self.warned.get()
    }

    /// The release sanction, and *once* is the load-bearing word.
    ///
    /// A frame that overruns usually overruns again — the 300 ms file read is in the loop — so a
    /// diagnostic per frame is a diagnostic nobody reads and an I/O call on the frame path for ever.
    fn warn_once(&self, message: &str) {
        if self.warned.replace(true) {
            return;
        }
        if let Some(sink) = self.report.borrow_mut().as_mut() {
            let _ = writeln!(sink, "{message}");
            let _ = sink.flush();
        }
    }

    /// The current budget, for the gate that compares it with `Config`'s own default.
    #[cfg(test)]
    pub(crate) fn threshold(&self) -> Duration {
        self.threshold.get()
    }

    /// Whether the iteration in progress has already lost its frame — asked without spending the
    /// sanction, which in a debug build is a panic and would take the test with it.
    ///
    /// `debug_assertions` as well as `test`: its only caller is in the debug-only test module, and
    /// the sanction it is asked *instead of* does not exist in a release build either.
    #[cfg(all(test, debug_assertions))]
    fn would_overrun(&self) -> bool {
        overrun_by(
            self.entered.get(),
            Instant::now(),
            self.excused.get(),
            self.threshold.get(),
        )
        .is_some()
    }

    /// Watch this app thread from another thread, and abort if it stops coming back.
    ///
    /// `restore` is `crate::shutdown::restore_current` as a function pointer, which is how this
    /// module reaches the terminal without naming the crate (see the module docs) — and it is the
    /// same door the panic hook uses, so the epilogue is the same epilogue.
    ///
    /// Nothing is spawned if the thread cannot be: a diagnostic that refuses to start the
    /// application it is diagnosing is worse than a missing diagnostic.
    #[cfg(debug_assertions)]
    pub(crate) fn observe(&self, restore: fn()) {
        let watch = Arc::clone(&self.watch);
        let poll = poll_for(stall_limit(self.threshold.get()));
        // **Captured here, on the thread that spawns**, so that a thread which never gets scheduled
        // before the next `stop_observing` still leaves with a generation it cannot match.
        let generation = watch.generation();
        let _ = std::thread::Builder::new()
            .name(String::from("vitui-observer"))
            .spawn(move || observe(&watch, restore, poll, generation));
    }

    /// This session is over; the observer has nothing left to watch.
    ///
    /// **Not joined.** The observer is asleep for up to one poll, and waiting on it would put up to
    /// 100 ms of exit latency into every debug run to tidy up a thread that is about to be reaped
    /// with the process anyway. It holds nothing but an `Arc<Watch>`.
    pub(crate) fn stop_observing(&self) {
        #[cfg(debug_assertions)]
        {
            // **The entry first, then the flag, and the order closes a race that would abort a
            // healthy process.** A program that returns from `wait` and then drops the `Screen`
            // without presenting leaves an iteration open for ever as far as the observer can see —
            // the session is over and nothing is frozen, but the marker still says *inside*. Zeroing
            // it before raising the flag means the worst an observer already mid-poll can find is a
            // stall that had genuinely been open longer than the limit before the drop, and
            // `observe` re-reads the flag once more before it acts on one.
            self.watch.left();
            self.watch.stop();
        }
    }

    /// This session is back; watch it again.
    ///
    /// The counterpart of [`stop_observing`](Perf::stop_observing), and it is a **verb of its own
    /// rather than a second `observe`** for the reason [`Watch::watch_again`] states: the stop flag
    /// lives behind the `Arc` the observer holds, so it outlives the thread that read it.
    ///
    /// A suspend really does have to stop the detector rather than leave it running — the iteration
    /// `wait` opened stays open for as long as somebody else has the terminal, and a user in their
    /// editor is not a frozen interface. What this puts right is that it has to come **back**.
    pub(crate) fn observe_again(&self, restore: fn()) {
        #[cfg(debug_assertions)]
        {
            self.watch.watch_again();
            self.observe(restore);
        }
        #[cfg(not(debug_assertions))]
        let _ = restore;
    }

    /// Whether the detector is still watching this session, for the gate that says a resume brings
    /// it back.
    ///
    /// A door rather than an inference, because the alternative is unwatchable: an observer that
    /// never comes back does nothing at all, and *does nothing* is what a healthy one looks like
    /// from outside for every run in which nothing stalls.
    #[cfg(all(test, debug_assertions))]
    pub(crate) fn is_watching(&self) -> bool {
        !self.watch.stopped()
    }

    /// The observer's half, for the tests that drive both halves from one thread.
    #[cfg(all(test, debug_assertions))]
    fn watch_handle(&self) -> Arc<Watch> {
        Arc::clone(&self.watch)
    }
}

/// **This region is allowed to be slow, and here is why.** 45 ns, measured, against a predicted
/// 4.20 ns — the difference is one `Instant::now()` at each end.
///
/// The 4.20 ns figure was for a guard that only set and cleared a flag, and a flag is not enough:
/// what a permitted region *cost* has to come off the iteration, or a permit taken for one
/// microsecond would excuse the 300 ms after it. So the guard reads the clock when the region opens
/// and again when it settles. It is not on the frame path — an application takes one per cold start,
/// not per frame — so two clock reads there buy a great deal and cost nothing that is measured
/// against a budget.
///
/// First-violation strictness needs an escape or every application's cold-start frame panics in a
/// debug build, and the honest form of the escape is a *declaration*: a string a stall report can
/// print, so that the escape hatch makes the diagnostics better rather than quieter.
///
/// ```
/// # let (screen, _wake) = vitui_engine::Engine::new(vitui_engine::Config {
/// #     output: vitui_engine::Output::Sink(Box::new(Vec::new())),
/// #     ..Default::default()
/// # }).attach().unwrap();
/// let _permit = screen.permit_slow("loading config");
/// ```
///
/// # It does not borrow the `Screen`, and that is the second correction
///
/// The first shape — `permit_slow(&mut self) -> Permit<'_>` — did not compile,
/// because a mutable borrow held for the guard's lifetime makes `enter` and `leave` unreachable
/// inside the very region the permit exists to excuse (`E0499`), and it records the fix as *`Cell`
/// and `&self` throughout*. **That fix is half of one.** With `Cell` inside, `permit_slow(&self)`
/// compiles and a `Permit<'a>` borrowed from `&'a Screen` still cannot have a frame drawn inside it:
/// [`Screen::layers`](crate::Screen::layers) takes `&mut self`, so the shared borrow the guard holds
/// makes drawing `E0502` — the identical defect one letter along, and the one the acceptance test
/// *a frame is drawn inside the permitted region* is about.
///
/// So the guard holds an `Rc<Perf>` and has **no lifetime parameter at all**. `Screen` keeps the
/// detector behind the same `Rc`, `permit_slow` bumps a refcount, and a permitted region borrows the
/// screen for exactly as long as the call takes. The signature is amended here rather than in a
/// document, and the compiler is the reason.
///
/// # `!Send`, and by the `Rc` rather than by a marker
///
/// A permit is the app thread's, so it may not travel — a worker holding one would excuse *this*
/// thread's iteration for reasons that had nothing to do with it. The refusal is `Rc<Perf>`'s own
/// and needs no `PhantomData` beside it, which is worth the sentence because a
/// `Cell<()>` here: the marker changed when the borrow did.
pub struct Permit {
    /// The detector this permit is a declaration to. **When the region began is on the detector
    /// rather than here**, because nesting settles once at the outermost guard and `leave` may
    /// settle it before any guard is dropped at all — a permit that is held across a `present` is
    /// exactly the case this whole type exists for.
    perf: Rc<Perf>,
}

impl Permit {
    /// Declare that what happens from here until this value is dropped is expected to be slow.
    fn new(perf: Rc<Perf>, reason: &'static str) -> Permit {
        if perf.depth.get() == 0 {
            perf.open.set(Some(Instant::now()));
        }
        perf.depth.set(perf.depth.get().saturating_add(1));
        perf.reason.set(Some(reason));
        #[cfg(debug_assertions)]
        perf.watch.note(Some(reason));
        Permit { perf }
    }

    /// The one door, so that `Screen` can mint one without `Permit`'s fields being crate-visible.
    pub(crate) fn mint(perf: &Rc<Perf>, reason: &'static str) -> Permit {
        Permit::new(Rc::clone(perf), reason)
    }
}

impl Drop for Permit {
    /// Only the outermost guard settles, and only what `leave` has not already taken.
    ///
    /// A nested permit's reason is **not** restored to its parent's on the way out — that would need
    /// a stack, and the reason exists to make one diagnostic legible rather than to be an audit
    /// trail. What a nested drop loses is the outer reason on a stall report between the inner drop
    /// and the outer one, which is a window of the caller's own making.
    fn drop(&mut self) {
        let depth = self.perf.depth.get().saturating_sub(1);
        self.perf.depth.set(depth);
        if depth > 0 {
            return;
        }
        if let Some(at) = self.perf.open.take() {
            self.perf.excuse(at.elapsed());
        }
        self.perf.reason.set(None);
        #[cfg(debug_assertions)]
        self.perf.watch.note(None);
    }
}

impl std::fmt::Debug for Permit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Permit")
            .field("reason", &self.perf.reason.get())
            .finish()
    }
}

/// What the observer thread is allowed to know about the app thread.
///
/// **Debug builds only**, and the type's absence from a release binary is the property rather than a
/// consequence of one: the gate is *no observer thread is linked into a release binary*,
/// and the check is that the words [`stall_report`] writes are in a debug binary and not in a
/// release one.
///
/// Everything here is atomic or behind a `Mutex` because two threads read it, and none of it is on
/// the frame path in a release build at all — [`Perf`]'s `watch` field does not exist there.
#[cfg(debug_assertions)]
pub(crate) struct Watch {
    /// What `entered` is counted from. An `Instant` cannot be built from a number, so the atomic
    /// holds an offset from one captured here.
    base: Instant,
    /// Nanoseconds since `base`, **plus one**, or zero outside an iteration. The bias is what makes
    /// *not in an iteration* and *entered at the base instant* two different values.
    entered: AtomicU64,
    /// How long an iteration may be inside before it is a stall.
    limit: AtomicU64,
    /// The innermost live permit's reason. A `Mutex` rather than two atomics holding a pointer and a
    /// length: reassembling a `&'static str` out of a pair that can tear needs `unsafe`, and
    /// refusal 12 is that there is none in this crate.
    reason: Mutex<Option<&'static str>>,
    /// Whether the session is over.
    stop: AtomicBool,
    /// **Which observer is the current one**, and it is what retires the previous one.
    ///
    /// A single `stop` flag cannot do that job, and the reason is a race rather than a subtlety.
    /// `observe` sleeps for up to one poll — 1 to 100 ms — before it reads the flag, so a
    /// suspend and a resume completed inside one poll interval leave the old observer waking to a
    /// flag that has already been cleared. It goes on polling for ever, beside the new one: a
    /// long-lived leaked thread rather than the short-lived one a bare re-`observe` would have
    /// leaked, which is worse in the direction that matters.
    ///
    /// So each observer captures this at spawn and leaves when it stops matching, whatever `stop`
    /// says. Monotone: `stop_observing` bumps it, and only [`Perf::observe`] ever reads it back.
    generation: AtomicU64,
}

#[cfg(debug_assertions)]
impl Watch {
    fn new(limit: Duration) -> Watch {
        Watch {
            base: Instant::now(),
            entered: AtomicU64::new(0),
            limit: AtomicU64::new(nanos(limit)),
            reason: Mutex::new(None),
            stop: AtomicBool::new(false),
            generation: AtomicU64::new(0),
        }
    }

    /// The app thread entered an iteration at `at`.
    fn entered(&self, at: Instant) {
        let since = nanos(at.saturating_duration_since(self.base)).saturating_add(1);
        self.entered.store(since, Ordering::Relaxed);
    }

    /// The app thread came back.
    fn left(&self) {
        self.entered.store(0, Ordering::Relaxed);
    }

    fn set_limit(&self, limit: Duration) {
        self.limit.store(nanos(limit), Ordering::Relaxed);
    }

    fn note(&self, reason: Option<&'static str>) {
        *self.lock() = reason;
    }

    fn stop(&self) {
        // **The bump first, then the flag.** An observer that is mid-poll must find a generation it
        // does not match whichever of the two it reads, and the one it reads first is the flag.
        self.generation.fetch_add(1, Ordering::Relaxed);
        self.stop.store(true, Ordering::Relaxed);
    }

    /// The generation an observer spawned now belongs to.
    fn generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }

    /// Watch again, for a session that is coming back.
    ///
    /// **The flag is what the observer thread reads to know it has nothing left to watch**, and it
    /// is shared through the `Arc` every observer holds — so it is sticky by design and a second
    /// `Perf::observe` on a stopped watch spawns a thread that returns on its first poll. Which is
    /// why [`Screen::resume`](crate::Screen::resume) cannot simply call `observe` again: it would
    /// leave a resumed session with no detector at all, silently, and leak one short-lived thread
    /// per resume for the appearance of one.
    fn watch_again(&self) {
        self.stop.store(false, Ordering::Relaxed);
    }

    /// Whether an observer of this generation is still the current one.
    fn is_current(&self, generation: u64) -> bool {
        self.generation() == generation
    }

    fn stopped(&self) -> bool {
        self.stop.load(Ordering::Relaxed)
    }

    /// How long the app thread has been inside an iteration it has not come back from, and what it
    /// said it was doing — or `None`, which is every ordinary poll.
    fn stalled(&self, now: Instant) -> Option<(Duration, Option<&'static str>)> {
        let entered = self.entered.load(Ordering::Relaxed);
        let entered = self.base + Duration::from_nanos(entered.checked_sub(1)?);
        let inside = now.saturating_duration_since(entered);
        (inside > Duration::from_nanos(self.limit.load(Ordering::Relaxed)))
            .then(|| (inside, *self.lock()))
    }

    /// The reason, for the tests that drive both halves from one thread.
    #[cfg(test)]
    fn reason_now(&self) -> Option<&'static str> {
        *self.lock()
    }

    /// An app thread that panicked while holding this must not stop the observer from reporting the
    /// stall it is about to cause.
    fn lock(&self) -> std::sync::MutexGuard<'_, Option<&'static str>> {
        self.reason.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// A `Duration` as a `u64` of nanoseconds, saturating rather than wrapping.
///
/// 584 years, so the saturation is unreachable in a process with a terminal attached — but
/// `Duration::MAX` is exactly what an application pins to switch the detector off, and `as u64` on
/// that would wrap to a small number and switch it violently back on.
#[cfg(debug_assertions)]
fn nanos(d: Duration) -> u64 {
    u64::try_from(d.as_nanos()).unwrap_or(u64::MAX)
}

/// The observer thread: sleep, look, and either go round again or end the process.
///
/// It never touches the app thread and never takes a lock the app thread holds on the frame path —
/// the reason mutex is only written when a permit is created or dropped.
#[cfg(debug_assertions)]
fn observe(watch: &Watch, restore: fn(), poll: Duration, generation: u64) {
    loop {
        std::thread::sleep(poll);
        // **Two conditions and not one, and the second is what a suspend needs.** `stop` alone is a
        // flag `Screen::resume` clears, and this thread reads it after sleeping for up to a poll —
        // so a suspend and a resume inside one interval would leave this observer alive beside the
        // one the resume spawned, polling for ever, both of them able to abort the process.
        if watch.stopped() || !watch.is_current(generation) {
            return;
        }
        if let Some((inside, reason)) = watch.stalled(Instant::now()) {
            // **Once more, after deciding and before acting.** `stop_observing` runs on the app
            // thread and the decision above is three loads wide, so a session that ended between the
            // first check and this one must not end the process: aborting a program that is already
            // on its way out would turn a clean exit into a crash report.
            if watch.stopped() || !watch.is_current(generation) {
                return;
            }
            // **The order is the sanction.** Restoring and continuing is broken — a returning app
            // thread would paint frames into a restored terminal — and printing before restoring
            // puts the message on an alt screen that is about to be discarded.
            restore();
            let mut err = std::io::stderr().lock();
            let _ = writeln!(err, "{}", stall_report(inside, reason));
            let _ = err.flush();
            std::process::abort();
        }
    }
}

// **`debug_assertions` as well as `test`, and impl 26 is where that was found.**
//
// The observer thread and everything that sizes it — `STALL_FACTOR`, `POLL_FLOOR`, `stall_limit`,
// `Perf::watch_handle` — are `#[cfg(debug_assertions)]`, because requirement 11's zero-wakeup idle is
// a *release-build* property and an observer thread would end it. This module tests them, so it has
// to be gated on the same condition and was not: `cargo test --release -p vitui-engine` failed to
// compile with **twenty-nine errors and two dead-code denials.**
//
// It failed nothing. CI runs `cargo test --workspace`, which is the dev profile, so the one
// configuration nobody built was the one every timing report in this crate names in its own doc
// comment — `cargo test --release … --nocapture`. Impl 26 went to re-measure the ledger against the
// shipped engine and could not run a single one of those commands.
//
// This is a shape met three times, arriving through a third
// door: **a check that is weaker than the gate is not a check** — and here the *gate* was fine and
// the thing nobody ran was the instrument. It is the same argument as `.gitlab-ci.yml`'s
// `cargo clippy -p vitui-engine --features fuzz` line, which exists because a feature-on,
// `cfg(test)`-off build had no build at all. The configurations a workspace does not compile are
// where this class of defect lives, and *release plus tests* is now one fewer of them.
#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::time::Instant;

    /// 60 Hz, the default: one frame interval is 16.6 ms.
    #[test]
    fn the_threshold_is_one_frame_interval_of_the_rate_the_application_declared() {
        assert_eq!(threshold_for(60.0, None), Duration::from_nanos(16_666_666));
        assert_eq!(threshold_for(120.0, None), Duration::from_nanos(8_333_333));
        assert_eq!(threshold_for(300.0, None), Duration::from_nanos(3_333_333));
    }

    #[test]
    fn a_pinned_threshold_overrules_the_rate_and_an_unlimited_rate_falls_back() {
        let pinned = Duration::from_millis(2);
        assert_eq!(threshold_for(60.0, Some(pinned)), pinned);
        assert_eq!(threshold_for(f32::INFINITY, Some(pinned)), pinned);
        // Unlimited is a statement about pacing, not about what a user can perceive.
        assert_eq!(
            threshold_for(f32::INFINITY, None),
            threshold_for(DEFAULT_HZ, None)
        );
        assert_eq!(threshold_for(0.0, None), threshold_for(DEFAULT_HZ, None));
    }

    /// A public `f32` reaches this, and a zero threshold panics a debug build on its first frame.
    #[test]
    fn a_rate_whose_interval_rounds_to_zero_is_unlimited_rather_than_instantaneous() {
        for absurd in [1e10f32, 1e20, f32::MAX] {
            assert_eq!(
                threshold_for(absurd, None),
                threshold_for(DEFAULT_HZ, None),
                "{absurd} produced a threshold no frame can meet"
            );
        }
        // And a rate that is merely very high still gets its own interval.
        assert_eq!(threshold_for(1000.0, None), Duration::from_micros(1_000));
    }

    #[test]
    fn an_iteration_inside_the_threshold_is_not_an_overrun() {
        let entered = Instant::now();
        let threshold = Duration::from_millis(8);
        assert_eq!(
            overrun_by(
                Some(entered),
                entered + Duration::from_millis(7),
                ZERO,
                threshold
            ),
            None
        );
    }

    #[test]
    fn an_iteration_past_the_threshold_reports_by_how_much() {
        let entered = Instant::now();
        let threshold = Duration::from_millis(8);
        assert_eq!(
            overrun_by(
                Some(entered),
                entered + Duration::from_millis(20),
                ZERO,
                threshold
            ),
            Some(Duration::from_millis(20))
        );
    }

    #[test]
    fn a_present_with_no_wait_before_it_is_not_an_iteration_at_all() {
        assert_eq!(
            overrun_by(None, Instant::now(), ZERO, Duration::from_nanos(1)),
            None
        );
    }

    #[test]
    fn what_a_permit_excused_comes_off_the_iteration_rather_than_off_the_detector() {
        let entered = Instant::now();
        let threshold = Duration::from_millis(8);
        let now = entered + Duration::from_millis(20);
        // 14 ms of it was declared slow, so 6 ms is what the frame actually cost.
        assert_eq!(
            overrun_by(Some(entered), now, Duration::from_millis(14), threshold),
            None
        );
        // And one that was not enough leaves the remainder over budget.
        assert_eq!(
            overrun_by(Some(entered), now, Duration::from_millis(5), threshold),
            Some(Duration::from_millis(15))
        );
    }

    #[test]
    fn the_stall_limit_is_far_past_the_threshold_and_moves_with_it() {
        assert_eq!(
            stall_limit(Duration::from_nanos(16_666_666)),
            Duration::from_nanos(16_666_666) * STALL_FACTOR
        );
        assert!(stall_limit(threshold_for(DEFAULT_HZ, None)) > STALL_FLOOR);
    }

    #[test]
    fn the_poll_is_several_polls_inside_the_limit_and_clamped_at_both_ends() {
        // The default: 1.07 s of limit, and the ceiling is what 369 context switches over 30 s was
        // measured at. Every limit a `Screen` can have is at or above the floor, so the ceiling is
        // the answer for all of them — and a second is ten polls, which is what `POLL_DIVISOR`
        // promises.
        assert_eq!(
            poll_for(stall_limit(threshold_for(DEFAULT_HZ, None))),
            POLL_CEILING
        );
        assert_eq!(poll_for(STALL_FLOOR), POLL_CEILING);
        assert!(STALL_FLOOR.div_duration_f64(POLL_CEILING) >= f64::from(POLL_DIVISOR));
        // Below the floor is reachable only by setting a `Watch`'s limit by hand, which is what the
        // gates do so that a stall costs 25 ms instead of a second and a quarter.
        assert_eq!(
            poll_for(Duration::from_millis(128)),
            Duration::from_millis(16)
        );
        assert_eq!(poll_for(Duration::from_micros(1)), POLL_FLOOR);
    }

    #[test]
    fn the_overrun_message_names_the_number_the_budget_and_the_permit() {
        let plain = overrun_message(
            Duration::from_micros(19_500),
            Duration::from_millis(8),
            None,
        );
        assert!(plain.contains("19.5 ms"), "{plain}");
        assert!(
            plain.contains("8.3 ms") || plain.contains("8.0 ms"),
            "{plain}"
        );
        let excused = overrun_message(
            Duration::from_millis(300),
            Duration::from_millis(8),
            Some("loading"),
        );
        assert!(excused.contains("loading"), "{excused}");
    }

    #[test]
    fn the_stall_report_carries_the_entry_time_and_the_permit_reason() {
        let text = stall_report(Duration::from_millis(1_500), Some("loading config"));
        assert!(text.contains("1.5 s"), "{text}");
        assert!(text.contains("loading config"), "{text}");
        let anonymous = stall_report(Duration::from_millis(1_500), None);
        assert!(anonymous.contains("no permit"), "{anonymous}");
    }

    /// A sink a test can read back, standing in for the one an application supplies.
    #[derive(Clone, Default)]
    struct Recorder(std::sync::Arc<std::sync::Mutex<Vec<u8>>>);

    impl Recorder {
        fn text(&self) -> String {
            String::from_utf8(self.0.lock().expect("nothing panicked holding it").clone())
                .expect("the detector writes UTF-8")
        }
    }

    impl std::io::Write for Recorder {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0
                .lock()
                .expect("nothing panicked holding it")
                .extend_from_slice(buf);
            Ok(buf.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    /// Short enough to overrun on purpose without a sleep long enough to slow the suite down.
    const TIGHT: Duration = Duration::from_millis(1);

    fn detector(report: Recorder) -> Rc<Perf> {
        Perf::new(DEFAULT_HZ, Some(TIGHT), Some(Box::new(report)))
    }

    #[test]
    fn an_iteration_inside_the_budget_says_nothing_at_all() {
        let report = Recorder::default();
        let perf = detector(report.clone());
        perf.enter();
        perf.leave();
        assert_eq!(report.text(), "");
    }

    #[test]
    fn a_present_with_no_wait_before_it_is_charged_to_nothing() {
        let report = Recorder::default();
        let perf = detector(report.clone());
        std::thread::sleep(TIGHT * 4);
        perf.leave();
        assert_eq!(report.text(), "");
    }

    /// **Debug: panic on the first overrun.** Safe by construction — the restoration is
    /// idempotent, guarded by one atomic and runs before the default hook prints.
    #[test]
    #[should_panic(expected = "a frame was dropped")]
    fn an_overrun_panics_in_a_debug_build() {
        let perf = detector(Recorder::default());
        perf.enter();
        std::thread::sleep(TIGHT * 4);
        perf.leave();
    }

    /// **Release: warn once**, and *once* is the load-bearing word: a frame that overruns usually
    /// overruns again, and a diagnostic printed sixty times a second is a diagnostic nobody reads.
    #[test]
    fn the_release_sanction_reaches_the_callers_sink_exactly_once() {
        let report = Recorder::default();
        let perf = detector(report.clone());
        perf.warn_once("the first");
        perf.warn_once("the second");
        let text = report.text();
        assert!(text.contains("the first"), "{text}");
        assert!(!text.contains("the second"), "{text}");
    }

    /// **The frame path may not allocate**, and after the first overrun the message is never built.
    ///
    /// A release application with a 300 ms read in its loop overruns on every frame, and formatting a
    /// diagnostic that `warn_once` then discards would put three or four `String`s on the frame path
    /// for ever. `tests/alloc.rs` cannot see this — it runs on `Clock::Manual` and never calls
    /// `wait`, so `leave` returns before the sanction — so the guard is asserted here instead.
    #[test]
    fn the_second_overrun_in_a_release_build_formats_nothing() {
        let perf = detector(Recorder::default());
        assert!(perf.will_report());
        perf.warn_once("spent");
        // In a debug build the first overrun panics, so there is never a second one to guard.
        assert_eq!(perf.will_report(), cfg!(debug_assertions));
    }

    #[test]
    fn with_no_sink_the_release_sanction_says_nothing_rather_than_writing_to_the_alt_screen() {
        let perf = Perf::new(DEFAULT_HZ, Some(TIGHT), None);
        perf.warn_once("nowhere to go");
    }

    /// The case whose first shape was `E0499` and whose second was `E0502`. Here at the level below
    /// `Screen`; `crate::gates::a_frame_may_be_drawn_inside_a_permitted_region` is the same property
    /// through the public API, which is where the borrow was the difficulty.
    #[test]
    fn a_permitted_region_is_excused_and_the_frame_around_it_is_not() {
        let report = Recorder::default();
        let perf = detector(report.clone());
        perf.enter();
        {
            let _permit = Permit::new(Rc::clone(&perf), "loading config");
            std::thread::sleep(TIGHT * 4);
        }
        perf.leave();
        assert_eq!(report.text(), "");
    }

    #[test]
    fn a_permit_that_ended_stops_excusing_the_iteration_it_was_in() {
        let perf = detector(Recorder::default());
        perf.enter();
        drop(Permit::new(
            Rc::clone(&perf),
            "a permit that excused nothing",
        ));
        std::thread::sleep(TIGHT * 4);
        assert!(perf.would_overrun());
    }

    /// **The observer a suspend retired is not the one a resume spawned**, and a flag cannot say
    /// so.
    ///
    /// `Screen::suspend` stops the detector — the iteration `wait` opened stays open for as long as
    /// somebody else has the terminal — and `Screen::resume` starts it again. Between the two, the
    /// previous observer is asleep for up to one poll and has not read anything: if the only signal
    /// were `stop`, it would wake to a flag the resume had already cleared and go on polling for
    /// ever, beside the observer the resume spawned. **Two live observers, both able to abort the
    /// process**, and `Perf::is_watching` reads the flag rather than the thread count, so nothing
    /// would notice.
    ///
    /// The generation is monotone and is bumped by the stop, so the old one cannot match whichever
    /// of the two values it reads first.
    #[test]
    fn an_observer_from_before_a_suspend_is_not_the_one_a_resume_spawned() {
        let perf = detector(Recorder::default());
        let watch = perf.watch_handle();
        let before = watch.generation();
        assert!(watch.is_current(before), "the one `observe` just captured");

        perf.stop_observing();
        assert!(
            !watch.is_current(before),
            "the stop left the sleeping observer able to match, so the resume below makes two"
        );

        watch.watch_again();
        assert!(
            !watch.is_current(before),
            "clearing the flag brought the retired observer back"
        );
        assert!(
            watch.is_current(watch.generation()),
            "and the new one is current"
        );
    }

    /// **And the loop honours it**, which the test above cannot say on its own: a generation nothing
    /// reads is a field.
    ///
    /// The watch is deliberately **not** stopped, so the only thing that can end this loop is the
    /// stale generation.
    ///
    /// **A flag and a deadline rather than a `join` or a `recv`.** A loop that ignored the
    /// generation would never return, so a join would hang the suite and *a hang is worse than a
    /// failure*; and a blocking receive here would be a real entry in
    /// `crate::gates::every_blocking_receive_in_the_crate_is_outside_the_app_threads_loop`'s
    /// allowance list, bought for a test — which is the gate's own subject, from the wrong side. It
    /// is the same spin `the_restoration_stops_the_renderer_before_the_terminal_is_given_back` uses.
    #[test]
    fn an_observer_whose_generation_is_stale_leaves_a_session_that_is_still_running() {
        let watch = Arc::new(Watch::new(TIGHT));
        let stale = watch.generation();
        // Bumped by something that is not a stop, so `stopped()` stays false and the generation is
        // the only reason left to leave.
        watch.generation.fetch_add(1, Ordering::Relaxed);
        assert!(!watch.stopped());

        let left = Arc::new(AtomicBool::new(false));
        let watched = Arc::clone(&watch);
        let reports = Arc::clone(&left);
        std::thread::spawn(move || {
            super::observe(&watched, || {}, POLL_FLOOR, stale);
            reports.store(true, Ordering::Relaxed);
        });
        let deadline = Instant::now() + Duration::from_secs(20);
        while !left.load(Ordering::Relaxed) && Instant::now() < deadline {
            std::hint::spin_loop();
        }
        assert!(
            left.load(Ordering::Relaxed),
            "an observer that is no longer the current one polls for ever, beside the one that \
             replaced it"
        );
    }

    #[test]
    fn the_reason_a_region_is_slow_is_readable_from_another_thread_while_it_is_held() {
        let perf = detector(Recorder::default());
        let watch = perf.watch_handle();
        assert_eq!(watch.reason_now(), None);
        let permit = Permit::new(Rc::clone(&perf), "loading config");
        assert_eq!(watch.reason_now(), Some("loading config"));
        drop(permit);
        assert_eq!(watch.reason_now(), None);
    }

    #[test]
    fn the_observer_sees_nothing_outside_an_iteration() {
        let perf = detector(Recorder::default());
        let watch = perf.watch_handle();
        assert_eq!(watch.stalled(Instant::now()), None);
    }

    /// The floor is what a `Screen` gets; the limit is driven directly here so that the property
    /// costs 25 ms instead of a second and a quarter. Which is exactly the trade the floor exists to
    /// make available: **a tight threshold no longer arms a tight abort**, so a test that wants one
    /// has to ask the `Watch` for it, in one line, where a reader can see it.
    #[test]
    fn an_iteration_inside_the_limit_is_not_a_stall_and_one_past_it_is() {
        let perf = detector(Recorder::default());
        let watch = perf.watch_handle();
        let limit = Duration::from_millis(20);
        watch.set_limit(limit);
        perf.enter();
        assert_eq!(watch.stalled(Instant::now()), None);
        std::thread::sleep(limit + limit / 4);
        let (inside, reason) = watch
            .stalled(Instant::now())
            .expect("the iteration has not come back");
        assert!(inside > limit, "{inside:?} against {limit:?}");
        assert_eq!(reason, None);
    }

    /// **A tight budget does not arm a tight abort.** The whole point of the floor: an application
    /// that pins one millisecond has asked for a stricter detector, not for a process that aborts
    /// after sixty-four of them.
    #[test]
    fn the_stall_limit_never_falls_under_its_floor_however_tight_the_budget() {
        assert_eq!(stall_limit(TIGHT), STALL_FLOOR);
        assert_eq!(stall_limit(Duration::from_nanos(1)), STALL_FLOOR);
        // And above the floor it is still the factor, so the limit stays caller-configurable where
        // configuring it means anything.
        let slow = Duration::from_millis(100);
        assert_eq!(stall_limit(slow), slow * STALL_FACTOR);
    }

    /// **A permit annotates the stall; it does not excuse it.** The in-loop rung is excused because
    /// the application said *this will be slow*; the observer answers a different claim — *this has
    /// not come back at all* — and a permitted region that runs past the limit belongs on a worker
    /// thread, which is the whole thesis.
    #[test]
    fn a_stall_inside_a_permit_still_stalls_and_carries_its_reason() {
        let perf = detector(Recorder::default());
        let watch = perf.watch_handle();
        watch.set_limit(Duration::from_millis(20));
        perf.enter();
        let _permit = Permit::new(Rc::clone(&perf), "loading config");
        std::thread::sleep(Duration::from_millis(40));
        let (_, reason) = watch
            .stalled(Instant::now())
            .expect("a permit is not an exemption from the observer");
        assert_eq!(reason, Some("loading config"));
    }

    #[test]
    fn a_stopped_observer_leaves_and_the_iteration_it_was_watching_is_none_of_its_business() {
        let perf = detector(Recorder::default());
        let watch = perf.watch_handle();
        assert!(!watch.stopped());
        perf.stop_observing();
        assert!(watch.stopped());
    }

    #[test]
    fn a_rate_the_application_changed_moves_the_threshold_and_a_pinned_one_does_not() {
        let derived = Perf::new(DEFAULT_HZ, None, None);
        assert_eq!(derived.threshold(), threshold_for(DEFAULT_HZ, None));
        derived.set_rate(300.0);
        assert_eq!(derived.threshold(), threshold_for(300.0, None));

        let pinned = Perf::new(DEFAULT_HZ, Some(TIGHT), None);
        pinned.set_rate(300.0);
        assert_eq!(pinned.threshold(), TIGHT);
    }
}
