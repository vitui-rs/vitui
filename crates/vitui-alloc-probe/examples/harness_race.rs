//! **The flake this probe's attribution rule exists for, reproduced on demand.**
//!
//! `a_column_of_wrapped_rows_allocates_zero` reported two allocations on one hosted Linux runner and
//! zero on five others. The code it measures cannot reach an allocator, and the two were the test
//! harness's: `--test-threads=1` does not stop the harness spawning a thread per test, and the
//! parent then blocks on an mpsc channel waiting for it. The **first blocking receive on that
//! channel allocates** — the receiving thread's channel context, the waker's mutex and the one
//! growth of the waker list — once per process, and if the parent is descheduled long enough to
//! reach it after the child has opened a window, that cost lands inside the child's number.
//!
//! # Why this is an example and not a test
//!
//! The interfering thread has to be *the process's first thread*, because that is the one every
//! reading here excludes. Inside a test binary that thread belongs to the harness and is blocked in
//! the receive this is about, so no test can drive it. A `main` can: this one is the parent, and the
//! thread it spawns is the subject, which is the same arrangement the harness has.
//!
//! It is run by CI beside the test command rather than left to be remembered, because a `cargo test`
//! does not run an example and a gate nothing runs is not a gate.
//!
//! # What it asserts
//!
//! Two things, and the first is what makes the second mean anything:
//!
//! 1. the parent's receive really does allocate inside the window — otherwise the platform has
//!    changed underneath the reproduction and the reading below proves nothing;
//! 2. none of it is attributed to the subject.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use vitui_alloc_probe::{CountingAllocator, Observed, observe};

#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator::new();

/// Raised by the subject once its window is open, and what makes this causal rather than timed: the
/// parent does not reach its receive until there is a window for it to land in.
static WINDOW_IS_OPEN: AtomicBool = AtomicBool::new(false);

/// How long the window stays open after that. The parent has to get from a spin loop into
/// `Receiver::recv` inside it, which is microseconds of work; this is five orders of magnitude of
/// margin and is not a budget.
const WINDOW: Duration = Duration::from_millis(200);

fn main() {
    let (tx, rx) = mpsc::channel::<Observed>();

    let subject = thread::spawn(move || {
        // The subject allocates nothing at all, which is the point: every number this window
        // reports came from somewhere else in the process.
        let ((), observed) = observe(|| {
            WINDOW_IS_OPEN.store(true, Ordering::Release);
            thread::sleep(WINDOW);
        });
        tx.send(observed).expect("the parent is waiting on this");
    });

    while !WINDOW_IS_OPEN.load(Ordering::Acquire) {
        std::hint::spin_loop();
    }
    // The parent's first blocking receive, taken with the subject's window open. Nothing has been
    // sent yet, so this cannot take the fast path that skips the allocations.
    let observed = rx.recv().expect("the subject sends before it returns");
    subject.join().expect("the subject returned");

    println!("the window was open for {WINDOW:?} and the subject allocated nothing in it");
    println!("  {:>3} allocations by the whole process", observed.process);
    println!(
        "  {:>3} on the process's first thread, which is the one this parent is",
        observed.first_thread
    );
    println!(
        "  {:>3} attributed to the subject, which is what a gate asserts on",
        observed.attributed
    );

    assert!(
        observed.first_thread > 0,
        "the parent's first blocking receive allocated nothing inside the window, so this no \
         longer reproduces what it was written for: the standard library's channel has changed and \
         the reading below has stopped proving anything"
    );
    assert_eq!(
        observed.attributed, 0,
        "a window attributed the parent's allocations to the subject, which is the defect this \
         crate's attribution rule removes"
    );
    println!("the parent's allocations are outside the subject's number");
}
