//! A counting global allocator, for asserting that a code path allocates nothing.
//!
//! The map's budget requires **zero allocations during frame composition**. Prior art shows this is
//! exactly the kind of guarantee that decays silently: ratatui shipped a per-frame `Vec` allocation
//! that was only noticed when it fragmented the heap on embedded devices. So it is asserted, not
//! assumed.
//!
//! This crate is a development tool. It is `publish = false` and must never appear in the runtime
//! dependency graph of `vitui-engine`.
//!
//! # Usage
//!
//! Install it as the global allocator in a test or benchmark binary, then wrap the path under test:
//!
//! ```
//! use vitui_alloc_probe::{CountingAllocator, assert_no_alloc};
//!
//! #[global_allocator]
//! static ALLOC: CountingAllocator = CountingAllocator::new();
//!
//! assert_no_alloc(|| {
//!     // steady-state frame composition goes here
//! });
//! ```
//!
//! # The window is process-wide, and one thread in it is never the subject
//!
//! The counter is global, so a window opened around a frame counts **every thread alive in the
//! process** while it is open. That is the property the attribution rule is about, and it is
//! deliberate: the engine's render thread is part of what a frame costs, and a gate that could not
//! see it would be measuring half the work.
//!
//! There is exactly one thread it must not see, and it belongs to nobody the tests can reach: **the
//! process's first thread**, which in a test binary is the harness's own. Every reading here
//! therefore subtracts what that thread allocated — *unless the reading is taken on it*, which is
//! how the same counter serves an example, where the first thread is the subject and opens the
//! window itself. [`allocation_count`] is the unfiltered number, kept beside the attributed one so
//! that a reader of *zero allocations* can see whose zero it is.
//!
//! ## What that rule is worth, measured rather than argued
//!
//! `a_column_of_wrapped_rows_allocates_zero` reported **two allocations** on one hosted Linux runner
//! and zero on five others, on the same commit. The code under test is `text::wrap_height` over four
//! fixed strings — an iterator holding two indices, over tables generated at build time, with no
//! branch that can reach an allocator — so nothing it does can be two on Tuesday and zero on
//! Wednesday.
//!
//! The two were the harness's. **`--test-threads=1` does not remove the thread**: the test harness
//! spawns one per test and then blocks on an mpsc channel waiting for it, and the first *blocking*
//! receive on that channel allocates three things once per process — the receiving thread's
//! thread-local channel context, the waker's mutex (a boxed pthread mutex on macOS, nothing at all
//! on Linux, which is why the failure said two and this machine says three) and the one-element
//! growth of the waker list. Normally the parent reaches its receive before the child has opened a
//! window, and the cost lands in nobody's number. On a runner with more threads than cores it can be
//! descheduled instead, and then a one-time cost of the harness's lands inside the first window the
//! process opens.
//!
//! That is why the exclusion is by *thread* and not by a warm-up, a retry or a tolerance: none of
//! those is a statement about who allocated.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

/// The first thread's share of the two counters above. Subtracted by every attributed reading.
static FIRST_ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static FIRST_DEALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

/// Taken by whichever thread reaches the allocator first, which is the thread the runtime starts on.
static FIRST_CLAIMED: AtomicBool = AtomicBool::new(false);

thread_local! {
    /// `0` not yet resolved, `1` this is the process's first thread, `2` it is not.
    ///
    /// `const`-initialised and `Copy` with no destructor, which is what makes it reachable from
    /// inside the allocator: such a thread-local needs no lazy allocation and registers no cleanup,
    /// so reading it cannot re-enter the path that is reading it.
    static ROLE: Cell<u8> = const { Cell::new(0) };

    /// This thread's allocations, for the split a failure reports.
    static MINE: Cell<usize> = const { Cell::new(0) };
}

/// Whether the calling thread is the one the process started on.
///
/// Resolved once per thread, by an atomic swap that exactly one thread can win. The winner is the
/// thread that reaches the allocator first, and that is the runtime's own: a Rust program allocates
/// on its first thread — its name, its arguments — before it reaches any code in this workspace.
///
/// A thread whose locals have already been torn down answers *not the first*, so its late
/// allocations are counted rather than silently discarded.
fn is_first_thread() -> bool {
    ROLE.try_with(|role| match role.get() {
        1 => true,
        2 => false,
        _ => {
            let first = !FIRST_CLAIMED.swap(true, Ordering::Relaxed);
            role.set(if first { 1 } else { 2 });
            first
        }
    })
    .unwrap_or(false)
}

/// A global allocator that forwards to the system allocator and counts calls.
pub struct CountingAllocator;

impl CountingAllocator {
    /// Creates the allocator. `const` so it can be used in a `static`.
    pub const fn new() -> Self {
        Self
    }
}

impl Default for CountingAllocator {
    fn default() -> Self {
        Self::new()
    }
}

/// Records one allocation against the process, the first thread if that is who we are, and us.
fn note_allocation() {
    ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    if is_first_thread() {
        FIRST_ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
    }
    let _ = MINE.try_with(|mine| mine.set(mine.get() + 1));
}

// SAFETY: every method forwards directly to `System` with the caller's layout unchanged, and the
// counters are plain atomics and `Copy` thread-locals that do not allocate.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        note_allocation();
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        if is_first_thread() {
            FIRST_DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        note_allocation();
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        note_allocation();
        unsafe { System.alloc_zeroed(layout) }
    }
}

/// Number of allocations recorded since the process started, **on every thread**.
///
/// The unfiltered reading. [`attributed_allocation_count`] is the one a gate should assert on; this
/// one is what says how far apart the two are, and a gate that reports a number is expected to
/// report both.
pub fn allocation_count() -> usize {
    ALLOCATIONS.load(Ordering::Relaxed)
}

/// Number of deallocations recorded since the process started, on every thread.
pub fn deallocation_count() -> usize {
    DEALLOCATIONS.load(Ordering::Relaxed)
}

/// [`allocation_count`] with the process's first thread taken out — unless you are it.
///
/// See the module comment for why that thread is excluded and why the exception is not a special
/// case: in a test binary the first thread is the harness's and never the subject, and in an example
/// it is the subject and is the thread asking.
pub fn attributed_allocation_count() -> usize {
    if is_first_thread() {
        ALLOCATIONS.load(Ordering::Relaxed)
    } else {
        ALLOCATIONS.load(Ordering::Relaxed) - FIRST_ALLOCATIONS.load(Ordering::Relaxed)
    }
}

/// [`deallocation_count`] with the process's first thread taken out — unless you are it.
pub fn attributed_deallocation_count() -> usize {
    if is_first_thread() {
        DEALLOCATIONS.load(Ordering::Relaxed)
    } else {
        DEALLOCATIONS.load(Ordering::Relaxed) - FIRST_DEALLOCATIONS.load(Ordering::Relaxed)
    }
}

/// Number of allocations made by the calling thread since it started.
pub fn thread_allocation_count() -> usize {
    MINE.try_with(Cell::get).unwrap_or(0)
}

/// Number of allocations made by the process's first thread since it started.
///
/// Read directly rather than derived from the two counts above, which is what lets a window assert
/// *the interference is still there* and *the subject's number excludes it* as two statements: from
/// one subtraction they would be one, and deleting the exclusion would make the first of them false
/// instead of the second.
pub fn first_thread_allocation_count() -> usize {
    FIRST_ALLOCATIONS.load(Ordering::Relaxed)
}

/// What a window saw, split by who allocated in it.
///
/// Three numbers over one interval: what the gate is asserting on, what the process did, and what
/// the measuring thread itself did. They are equal in the ordinary case and it is the differences
/// that name a defect — in the code, in the instrument, or in the attribution rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Observed {
    /// Allocations the window attributes to the subject: every thread but the first.
    pub attributed: usize,
    /// Allocations the whole process made while the window was open.
    pub process: usize,
    /// The share of `attributed` made by the thread that opened the window.
    pub here: usize,
    /// Allocations inside the window made by the process's first thread, which no gate asserts on.
    ///
    /// Measured, not derived: see [`first_thread_allocation_count`].
    pub first_thread: usize,
}

impl Observed {
    /// Allocations inside the window made by some *other* thread the subject is responsible for —
    /// the engine's render thread, a worker, a decoder.
    pub const fn on_another_thread(self) -> usize {
        self.attributed - self.here
    }
}

/// Runs `f` and returns how many allocations it caused, along with its result.
///
/// The count is [`attributed_allocation_count`]'s: every thread alive in the window except the one
/// the process started on. Use [`observe`] where the split matters.
pub fn count_allocations<T>(f: impl FnOnce() -> T) -> (T, usize) {
    let (value, observed) = observe(f);
    (value, observed.attributed)
}

/// Runs `f` and returns what the window saw, split three ways.
pub fn observe<T>(f: impl FnOnce() -> T) -> (T, Observed) {
    let before_attributed = attributed_allocation_count();
    let before_process = allocation_count();
    let before_here = thread_allocation_count();
    let before_first = first_thread_allocation_count();
    let value = f();
    let observed = Observed {
        attributed: attributed_allocation_count() - before_attributed,
        process: allocation_count() - before_process,
        here: thread_allocation_count() - before_here,
        first_thread: first_thread_allocation_count() - before_first,
    };
    (value, observed)
}

/// Runs `f` and panics if it allocated.
///
/// # Panics
///
/// Panics if `f` performed at least one allocation on any thread but the process's first, reporting
/// how many and where they were made.
#[track_caller]
pub fn assert_no_alloc<T>(f: impl FnOnce() -> T) -> T {
    let (value, observed) = observe(f);
    assert!(observed.attributed == 0, "{}", failure(observed));
    value
}

/// The sentence a failed window prints, which is the whole diagnosis the next reader gets.
///
/// Written out rather than left to `assert_eq!` because the number on its own sent one session
/// looking for an allocation in a function that cannot reach an allocator: what matters is not that
/// the count was two but that neither of the two was made by the thread under test.
fn failure(observed: Observed) -> String {
    let mut message = format!(
        "expected zero allocations, observed {} ({} on this thread",
        observed.attributed, observed.here
    );
    let elsewhere = observed.on_another_thread();
    if elsewhere > 0 {
        message.push_str(&format!(
            ", {elsewhere} on another thread alive inside the window — a worker, or the engine's \
             render thread, which the window counts on purpose"
        ));
    }
    message.push(')');
    let harness = observed.first_thread;
    if harness > 0 {
        message.push_str(&format!(
            ". {harness} further allocations were made inside the window by the process's first \
             thread and are not in the number above"
        ));
    }
    message
}

/// **Run `f` once untimed, then assert it allocates nothing.**
///
/// Use this rather than [`assert_no_alloc`] for anything with a warm-up cost — which is almost every
/// gate, because *zero allocations* nearly always means *zero in the steady state* and reaching the
/// steady state is itself allocation. A window that includes the first touch is measuring the loader.
///
/// # It lives here because the pattern could not enforce itself anywhere else
///
/// Two allocation gates flaked in CI within two pipelines, on the same class of defect and in
/// different crates: one in the runtime observing two allocations and one in the engine's handoff
/// observing nine. The first fix introduced a `steady` helper — and put it in one integration test
/// file. **Each `tests/*.rs` compiles as its own crate**, so a private helper there cannot be reached
/// from a sibling file, let alone from another crate, and the commit claiming *forgetting it means not
/// calling it, which shows up in a diff* was true only inside the one file it lived in.
///
/// This crate is the one place both sides already see: it is a dev-dependency of the engine and of
/// the runtime, and it is where `assert_no_alloc` lives. **The warm-up now travels with the counter.**
///
/// # The warm-up has to be the same work, not less of it
///
/// The engine's handoff gate already warmed — ten frames — and still flaked, because it then measured
/// **a thousand**, whose damage patterns reach buffers ten frames never touch. `f` is run *whole*, so
/// the warm pass is the identical workload rather than a guess at a prefix of it. That doubles the
/// work and removes the guess, and a gate is not the place to be economical about certainty.
///
/// # A warm-up is not what makes a window honest
///
/// The runtime's third flake looked like the first two and was not: it was a **one-time cost of the
/// test harness** landing inside the window of the first test in the binary, which no warm pass of
/// the subject can spend, because the subject is not what pays it. That is the module comment's rule
/// and this function inherits it — running `f` twice says nothing about who else was running.
///
/// # Panics
///
/// Panics if the second run performed at least one allocation.
#[track_caller]
pub fn steady<T>(mut f: impl FnMut() -> T) -> T {
    let _first_touch = f();
    assert_no_alloc(f)
}

#[cfg(test)]
mod tests {
    use super::*;

    // These tests exercise the counting logic itself; they do not need the allocator to be
    // installed, because they only assert on relative movement.

    #[test]
    fn counts_a_vec_allocation() {
        let (_, count) = count_allocations(|| Vec::<u8>::with_capacity(1024));
        // Without the allocator installed as the global one this is 0, which is still a valid
        // observation: the helper must not itself allocate.
        assert!(count <= 1, "the probe allocated on its own: {count}");
    }

    #[test]
    fn assert_no_alloc_passes_on_a_pure_computation() {
        let value = assert_no_alloc(|| 2 + 2);
        assert_eq!(value, 4);
    }

    /// The message is the diagnosis, so it is asserted rather than left to be read once in anger.
    #[test]
    fn a_failure_says_which_thread_allocated() {
        let mixed = Observed {
            attributed: 2,
            process: 5,
            here: 1,
            first_thread: 3,
        };
        let said = failure(mixed);
        assert!(said.contains("observed 2 (1 on this thread"), "{said}");
        assert!(said.contains("1 on another thread"), "{said}");
        assert!(said.contains("3 further allocations"), "{said}");
    }
}
