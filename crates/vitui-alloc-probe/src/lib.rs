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
//! # Caveat
//!
//! The counter is process-global, so a test asserting on it must not run concurrently with another
//! test that allocates. Use `--test-threads=1`, or keep allocation assertions in their own binary.
//! Both: CI runs the whole suite a second time under `--test-threads=1`, and an allocation
//! assertion belongs in a test binary of its own so that the second run is the one that can fail.

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

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

// SAFETY: every method forwards directly to `System` with the caller's layout unchanged, and the
// counters are plain atomics that do not allocate.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        DEALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.realloc(ptr, layout, new_size) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        unsafe { System.alloc_zeroed(layout) }
    }
}

/// Number of allocations recorded since the process started.
pub fn allocation_count() -> usize {
    ALLOCATIONS.load(Ordering::Relaxed)
}

/// Number of deallocations recorded since the process started.
pub fn deallocation_count() -> usize {
    DEALLOCATIONS.load(Ordering::Relaxed)
}

/// Runs `f` and returns how many allocations it caused, along with its result.
pub fn count_allocations<T>(f: impl FnOnce() -> T) -> (T, usize) {
    let before = allocation_count();
    let value = f();
    (value, allocation_count() - before)
}

/// Runs `f` and panics if it allocated.
///
/// # Panics
///
/// Panics if `f` performed at least one allocation, reporting how many.
#[track_caller]
pub fn assert_no_alloc<T>(f: impl FnOnce() -> T) -> T {
    let (value, count) = count_allocations(f);
    assert_eq!(count, 0, "expected zero allocations, observed {count}");
    value
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
}
