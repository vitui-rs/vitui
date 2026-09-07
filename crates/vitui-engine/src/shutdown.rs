//! The terminal comes back: one restoration, one atomic, callable from any thread.
//!
//! # Why this is not a method on `Screen`
//!
//! **"Only the render thread writes" narrows to *only the render thread writes frames***.
//! A panic hook runs on whichever thread panicked, and joining the render thread from inside it
//! deadlocks when the render thread is the one that panicked — so restoration cannot be something
//! the app thread does on its way out. It is an **idempotent function guarded by one atomic**, and
//! [`Site`] is that atomic with the two things the function needs beside it.
//!
//! Two details are cheap to omit and expensive to debug, and both are here rather than in a
//! document:
//!
//! - Restoration runs **before** the default panic hook prints. Otherwise the backtrace is painted
//!   into the alt screen and vanishes with it, which is the one moment a developer needs it most.
//! - It hangs on a guard's `Drop` as well as on the hook, so a normal return and a `?` out of `main`
//!   take the same path. That guard is [`Screen`](crate::Screen) itself — §12 can say *Drop
//!   restores* in one line on the type because [`Screen::drop`](crate::Screen) calls
//!   [`Site::restore`] with the renderer's own sink.
//!
//! # What is given back, and why it is more than the alt screen
//!
//! §9's restoration **includes input state**: pop the keyboard enhancement flags, disable mouse
//! tracking, focus reporting and bracketed paste, restore auto-wrap and leave the alt screen. The
//! screen is the half a user can see is broken. **The input state is the half that breaks their
//! shell** — a crashed process that leaves kitty flags pushed goes on eating keystrokes in the shell
//! that outlives it, until somebody knows to run `reset`.
//!
//! The bytes themselves are [`crate::actuate::restoration`]'s, computed here rather than stored,
//! because one of their inputs moves: the mouse level the terminal was **last told** is not the one
//! the negotiation set, and an application that raised it through
//! [`Screen::set_mouse`](crate::Screen::set_mouse) has to have that level reset rather than its
//! floor. [`Site::mouse_is`] is one relaxed store on the submit path — the frame path may not
//! allocate, and re-deriving a `Vec` per frame is exactly the allocation `tests/alloc.rs` refuses.
//!
//! # Two sinks, and why that is not two answers
//!
//! The ordinary path writes the epilogue through the **renderer's** sink, which is the same
//! descriptor every frame went to and which the app thread has just taken back from the render
//! thread by joining it. The panic path has no renderer — it may *be* the render thread's unwind —
//! so it writes through the sink held here.
//!
//! For [`Output::Terminal`](crate::Output) those two are the same thing twice: `std::io::stdout()`
//! is a handle to one process-global, internally locked stream, so a second handle is not a second
//! writer. For a caller-supplied sink there is nothing to duplicate and the default is
//! `std::io::sink()` — a file or a socket is not a terminal and has no modes to give back. The
//! `cfg(test)` door on [`Engine`](crate::engine::Engine) is what lets a headless gate capture the
//! panic path's bytes anyway, and it swaps that one field and nothing else.
//!
//! One consequence is accepted rather than solved: a panic that arrives while the render thread is
//! inside a frame's `write` puts the epilogue between two of that frame's `write` calls, because
//! `Stdout`'s lock is per call and a frame larger than the kernel's buffer is more than one. The
//! alternative is joining the render thread from the hook, which is the deadlock this whole module
//! is shaped around. The visible cost is a torn final frame on a page that is about to be discarded
//! anyway; the thing that must not tear is the restoration itself, and that is one `write_frame` of
//! nineteen to forty-odd bytes.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Once, PoisonError};

use crate::caps::Capabilities;
use crate::handoff::Mailbox;
use crate::input::InputConfig;

/// Everything the terminal is owed back, and the one atomic that decides who gives it.
pub(crate) struct Site {
    /// **The one atomic.** `swap(true)` is the whole protocol: whoever reads `false` performs the
    /// restoration and everybody else returns having done nothing.
    done: AtomicBool,
    /// Whether a tracking mode may be set on the terminal — **monotone, and never cleared**.
    ///
    /// It is a flag and not a level, and that is a correction rather than a simplification. The
    /// level the app thread knows is the one the *next packet* will ask for; the bytes that change
    /// the mode are written later, by the render thread, and there is a window in which either the
    /// old mode or the new one is the one the terminal is actually in. Nothing on the app thread can
    /// close that window, so the restoration resets **all three** tracking modes whenever this is
    /// set, and the only question this has to answer is *did this session ever ask for a mouse at
    /// all*. Never cleared, for the same reason from the other side: an application that lowers the
    /// level back to `Off` has a packet in flight too.
    mouse: AtomicBool,
    /// Whether the render thread may still be handed packets.
    ///
    /// **The restoration stops it, and that is not tidiness.** Without it a panic on any thread
    /// leaves the terminal restored while the render thread goes on taking packets: the epilogue
    /// leaves the alt screen and the next frame paints cells onto the user's shell. It is worse on a
    /// worker thread, where the process does not end at all and so it never stops.
    mailbox: Arc<Mailbox>,
    /// Whether [`Owed::sink`] is somewhere real. Fixed for the life of the site and therefore
    /// readable without the lock, which [`Site::restore`] needs: it has to answer this **before** it
    /// claims the guard.
    has_sink: bool,
    owed: Mutex<Owed>,
}

/// What the restoration is computed from, and where it goes when there is no renderer left.
struct Owed {
    config: InputConfig,
    caps: Capabilities,
    /// `None` over a caller-supplied sink, which is not a terminal and has no second handle.
    sink: Option<Box<dyn Write + Send>>,
}

impl Site {
    /// A site for one session, with the terminal in the state the negotiation left it.
    pub(crate) fn new(
        config: InputConfig,
        caps: Capabilities,
        sink: Option<Box<dyn Write + Send>>,
        mouse_on: bool,
        mailbox: Arc<Mailbox>,
    ) -> Arc<Site> {
        Arc::new(Site {
            done: AtomicBool::new(false),
            mouse: AtomicBool::new(mouse_on),
            mailbox,
            has_sink: sink.is_some(),
            owed: Mutex::new(Owed { config, caps, sink }),
        })
    }

    /// A tracking mode may be set on the terminal. Monotone: see [`Site::mouse`].
    pub(crate) fn mouse_may_be_on(&self, on: bool) {
        if on {
            self.mouse.store(true, Ordering::Relaxed);
        }
    }

    /// Give the terminal back what it is owed, at most once, from whichever thread arrives first.
    ///
    /// `through` is the renderer's sink when the app thread still has it. `None` is the panic path,
    /// and also the ordinary path after a render thread has died holding the sink.
    ///
    /// **The swap happens before the lock**, which is what makes eight threads racing this cost one
    /// lock acquisition rather than eight: the seven losers never touch the mutex.
    ///
    /// # A site with nowhere to write does not claim the guard
    ///
    /// The panic path over a caller-supplied sink has no second handle and writes nothing — and it
    /// must not *say* it restored, because the unwinding `Screen` is still to come and still holds
    /// the renderer's. Burning the one-shot on a write to `std::io::sink()` would leave a recorded
    /// session unterminated, and an `Output::Sink(stdout)` — which is how a caller bypasses
    /// detection — in the alt screen with the kitty flags pushed.
    pub(crate) fn restore(&self, through: Option<&mut (dyn Write + Send)>) -> bool {
        if through.is_none() && !self.has_sink {
            return false;
        }
        if self.done.swap(true, Ordering::AcqRel) {
            return false;
        }
        // **Before the bytes, not after.** A render thread that takes one more packet after the
        // epilogue paints a frame onto the user's shell. This cannot stop a write already in flight
        // — nothing can, short of the join this module exists to avoid — but it is what makes *the
        // last thing this process says is the epilogue* true of every frame that had not started.
        self.mailbox.quit();
        let mut owed = self.lock();
        let bytes = crate::actuate::restoration(
            &owed.config,
            &owed.caps,
            self.mouse.load(Ordering::Relaxed),
        );
        // **No `expect` on either arm, and the reason is where this runs.** The `None` arm is
        // unreachable with no sink — the guard above returned — and a wrong `expect` here would
        // panic *inside the panic hook*, which is an abort rather than a failed test. Verified by
        // removing the guard: the suite stops with `thread panicked while processing panic`.
        match (through, owed.sink.as_mut()) {
            (Some(sink), _) => crate::engine::write_frame(sink, &bytes),
            (None, Some(sink)) => crate::engine::write_frame(&mut **sink, &bytes),
            (None, None) => {}
        }
        true
    }

    /// A panicking thread has already broken one invariant; a poisoned mutex must not stop it
    /// giving the terminal back.
    fn lock(&self) -> MutexGuard<'_, Owed> {
        self.owed.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Whether the restoration has been performed.
    #[cfg(test)]
    pub(crate) fn is_restored(&self) -> bool {
        self.done.load(Ordering::Acquire)
    }
}

/// The site the panic hook can reach, which is the one the most recent `attach` installed.
///
/// A process has one terminal, so this holds one site rather than a list. It is an `Arc` and not a
/// `Weak` on purpose: a panic that arrives while a `Screen` is halfway through its own `Drop` must
/// still find something to restore through, and the guard in [`Site::restore`] is what keeps that
/// from being a second restoration.
static CURRENT: Mutex<Option<Arc<Site>>> = Mutex::new(None);

/// Installs the hook exactly once per process, however many screens are attached.
///
/// Once, because the hook **chains**: it takes whatever hook was installed before it and calls it
/// afterwards. Installing it per `attach` would nest one more layer per screen, and under `cargo
/// test` — where libtest has its own hook and a suite attaches hundreds of times — that is an
/// unbounded chain rather than a leak nobody notices.
static HOOK: Once = Once::new();

/// Make this site the one a panic on any thread restores through.
pub(crate) fn arm(site: &Arc<Site>) {
    install_hook();
    *current() = Some(Arc::clone(site));
}

/// This session is over; a later panic has nothing here to give back.
///
/// Guarded by identity rather than unconditional: a process that attached twice — every test binary
/// in this crate — must not have the second screen's site cleared by the first screen's `Drop`.
pub(crate) fn disarm(site: &Arc<Site>) {
    let mut held = current();
    if is_armed(&held, site) {
        *held = None;
    }
}

/// Whether what is armed is **this** site.
///
/// Its own function so that it can have its own test. Asserting it through [`CURRENT`] would be a
/// test that races every other test in the binary, because every `attach` arms — which is the hazard
/// the three exit gates are child processes to avoid, and it does not become acceptable one file up.
fn is_armed(held: &Option<Arc<Site>>, site: &Arc<Site>) -> bool {
    held.as_ref().is_some_and(|c| Arc::ptr_eq(c, site))
}

fn current() -> MutexGuard<'static, Option<Arc<Site>>> {
    CURRENT.lock().unwrap_or_else(PoisonError::into_inner)
}

/// What the hook does, and what a test that wants to be the hook calls.
///
/// `pub(crate)` since impl 23, for its second caller: the debug observer's sanction is *restore the
/// terminal, print the stall, abort*, and it runs on a thread that holds no `Screen` and could not
/// hold one. It reaches the same process-global site the hook does, so it gives back the same
/// epilogue under the same one-shot guard — an observer that fires while a panic is already
/// unwinding restores once between them.
pub(crate) fn restore_current() {
    // The `Arc` is cloned out and the lock released **before** the write, because the write is a
    // syscall and this lock is reachable from every thread in the process. The clone is what makes
    // that possible at all.
    let site = current().clone();
    if let Some(site) = site {
        site.restore(None);
    }
}

/// Restoration first, then whatever was going to print the panic.
///
/// The order is the whole point: the default hook writes the message and the backtrace, and a
/// terminal still in the alt screen shows them on a page that is discarded a moment later.
fn install_hook() {
    HOOK.call_once(|| {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore_current();
            previous(info);
        }));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::caps::KITTY_ALL;
    use crate::input::MouseMode;
    use crate::testing::Recorder;

    /// Every input protocol answered, which is what makes the epilogue's every arm reachable.
    fn everything() -> Capabilities {
        Capabilities::with_input(true, true, true, true, KITTY_ALL)
    }

    /// An application that asked for all three of the modes that are opt-in.
    fn asking_for_everything() -> InputConfig {
        InputConfig {
            mouse: MouseMode::Buttons,
            focus: true,
            paste: true,
            ..InputConfig::default()
        }
    }

    /// A site over a recorder, with a mailbox of its own so that the quit is observable.
    fn site(config: InputConfig, mouse_on: bool) -> (Arc<Site>, Recorder, Arc<Mailbox>) {
        let sink = Recorder::new();
        let mailbox = Arc::new(Mailbox::new());
        let site = Site::new(
            config,
            everything(),
            Some(Box::new(sink.clone())),
            mouse_on,
            Arc::clone(&mailbox),
        );
        (site, sink, mailbox)
    }

    fn written(sink: &Recorder) -> String {
        let r = sink.handle();
        let r = r.lock().expect("the recorder is never poisoned");
        String::from_utf8_lossy(&r.bytes).replace('\x1b', "^[")
    }

    /// **Spec §7's assertion, in its own words**: eight threads race it and exactly one performs the
    /// restoration.
    ///
    /// Two counts rather than one, and the second is the one that matters: seven threads answering
    /// `false` proves the guard returns the right answer, and one copy of the bytes in the sink
    /// proves it guarded the *write* rather than only the bookkeeping.
    #[test]
    fn eight_threads_race_the_restoration_and_exactly_one_performs_it() {
        const RACERS: usize = 8;

        let (site, sink, _mailbox) = site(asking_for_everything(), true);
        let start = Arc::new(std::sync::Barrier::new(RACERS));
        let performed: Vec<bool> = std::thread::scope(|scope| {
            let handles: Vec<_> = (0..RACERS)
                .map(|_| {
                    let site = Arc::clone(&site);
                    let start = Arc::clone(&start);
                    scope.spawn(move || {
                        start.wait();
                        site.restore(None)
                    })
                })
                .collect();
            handles
                .into_iter()
                .map(|h| h.join().expect("a racer panicked"))
                .collect()
        });

        assert_eq!(
            performed.iter().filter(|p| **p).count(),
            1,
            "{performed:?} — one thread performs it and seven find it done"
        );
        assert!(site.is_restored());

        let seen = written(&sink);
        assert_eq!(
            seen.matches("?1049l").count(),
            1,
            "the alt screen was left once, whatever the bookkeeping said: {seen}"
        );
        assert_eq!(seen, crate::gates::EPILOGUE_ESCAPED, "{seen}");
    }

    /// A ninth call, long after the race, is still nothing at all.
    #[test]
    fn a_restoration_that_has_happened_never_happens_again() {
        let (site, sink, _mailbox) = site(InputConfig::default(), false);

        assert!(site.restore(None));
        let after_one = written(&sink).len();
        for _ in 0..8 {
            assert!(!site.restore(None));
        }
        assert_eq!(
            written(&sink).len(),
            after_one,
            "a second restoration reached the wire"
        );
    }

    /// **Every tracking mode is reset, not the one the app thread believes in.**
    ///
    /// The level the app thread holds is the one the *next packet* will ask for, and the packet that
    /// switches the mode is written later by the render thread — so during that window the terminal
    /// is in one of two modes and nothing here can say which. Resetting only one leaves the other
    /// standing, and a shell that outlives the crash goes on receiving mouse reports.
    #[test]
    fn every_tracking_mode_goes_off_when_the_session_asked_for_a_mouse() {
        let (site, sink, _mailbox) = site(asking_for_everything(), false);
        // The floor was `Buttons`, so the negotiation set 1000 — and then the application raised the
        // level, which is one relaxed store and no idea which of the two the wire has reached.
        site.mouse_may_be_on(true);
        assert!(site.restore(None));

        let seen = written(&sink);
        for mode in ["?1003l", "?1002l", "?1000l", "?1006l"] {
            assert!(seen.contains(mode), "{mode} was left standing: {seen}");
        }
    }

    /// **A session that never asked for a mouse says nothing about one.**
    ///
    /// The other half of the rule above, and the half that keeps it honest: a mode this process did
    /// not set is a mode it may not clear, because the person at the terminal may have set it.
    #[test]
    fn a_session_that_never_asked_for_a_mouse_resets_no_tracking_mode() {
        let (site, sink, _mailbox) = site(InputConfig::default(), false);
        // Monotone in one direction only: `false` never unsets anything, and never sets it either.
        site.mouse_may_be_on(false);
        assert!(site.restore(None));

        let seen = written(&sink);
        for mode in ["1000", "1002", "1003", "1006"] {
            assert!(!seen.contains(mode), "{mode} was reset by nobody: {seen}");
        }
    }

    /// **The render thread is stopped before the terminal is given back.**
    ///
    /// Without this the epilogue leaves the alt screen and the render thread paints the next packet
    /// onto the user's shell — and on a worker thread's panic, where the process does not end, it
    /// goes on doing so. `Screen::drop` sets the same flag a moment later; what this asserts is that
    /// the *restoration* does not depend on the drop having happened.
    #[test]
    fn the_restoration_stops_the_render_thread_before_it_writes_a_byte() {
        let (site, _sink, mailbox) = site(InputConfig::default(), false);
        assert!(!mailbox.quit_requested());
        assert!(site.restore(None));
        assert!(mailbox.quit_requested());
    }

    /// **A site with nowhere to write does not claim the guard.**
    ///
    /// A caller-supplied sink has no second handle, so the panic path writes nothing — and it must
    /// not report that it restored, because the unwinding `Screen` still holds the renderer's sink
    /// and is the one that will terminate the stream.
    #[test]
    fn a_panic_over_a_caller_supplied_sink_leaves_the_drop_still_owing_it() {
        let mailbox = Arc::new(Mailbox::new());
        let site = Site::new(
            asking_for_everything(),
            everything(),
            None,
            true,
            Arc::clone(&mailbox),
        );

        assert!(!site.restore(None), "there was nowhere to write");
        assert!(!site.is_restored(), "and so nothing was owed less");
        assert!(
            !mailbox.quit_requested(),
            "a restoration that did not happen may not stop the renderer either"
        );

        // And the ordinary path, which does have a sink, still gets its epilogue.
        let mut through = Recorder::new();
        assert!(site.restore(Some(&mut through)));
        assert_eq!(written(&through), crate::gates::EPILOGUE_ESCAPED);
    }

    /// `arm` replaces, and `disarm` clears only what it armed.
    ///
    /// The identity guard is not a nicety: a test binary attaches hundreds of times, and an
    /// unconditional `disarm` would leave the live screen's site unreachable from the hook the first
    /// time a dead one was dropped.
    ///
    /// Asserted over [`is_armed`] rather than over [`CURRENT`], because every `attach` in this binary
    /// arms — so a test that read the global would be a test about which of its siblings ran last.
    #[test]
    fn disarming_an_old_site_does_not_unarm_the_current_one() {
        let older = Site::new(
            InputConfig::default(),
            everything(),
            None,
            false,
            Arc::new(Mailbox::new()),
        );
        let newer = Site::new(
            InputConfig::default(),
            everything(),
            None,
            false,
            Arc::new(Mailbox::new()),
        );

        assert!(!is_armed(&None, &older), "nothing is armed");
        assert!(
            !is_armed(&Some(Arc::clone(&newer)), &older),
            "the older screen's Drop would have taken the live screen's site with it"
        );
        assert!(is_armed(&Some(Arc::clone(&older)), &older));
        // Identity and not equality: two sites with the same fields are two sessions.
        assert!(!is_armed(&Some(Arc::clone(&newer)), &Arc::clone(&older)));
    }
}
