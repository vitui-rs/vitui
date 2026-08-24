//! **`work` allocates nothing on a frame, and the probe cannot tell whose frame it was.**
//!
//! Two claims, and the second is the more important one because it is a rule the rest of the backlog
//! inherits.
//!
//! # Why this is its own binary, and why it is a *second* one
//!
//! [`vitui_alloc_probe::CountingAllocator`] is a **process-global** allocator and the count is a
//! process-global counter, so an allocating sibling lands in the number — which is why the
//! workspace's one test command is `cargo test --workspace -- --test-threads=1`. `tests/alloc.rs`
//! beside this file makes the same argument for the layout solver. This is a separate binary rather
//! than more tests in that one because **a worker is a thread**, and the attribution gate below
//! deliberately runs one *inside* a measurement window. Keeping that in its own process keeps it
//! away from every gate that is asserting zero.
//!
//! # A gate with a background job in it runs on a deterministic spawner, or joins before it measures
//!
//! Spec §20 states it as an attribution rule and this file is where it comes from. Every gate here
//! except the last one runs on [`Worker::queueing`], which has no thread at all: the mechanism under
//! test is staleness arithmetic on the app thread, and the only thing a real thread contributes to
//! it is an order. The last one is the demonstration that the rule is necessary.

use std::hint::black_box;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use vitui_alloc_probe::{CountingAllocator, count_allocations, steady};
use vitui_runtime::work::{Drain, Requested, Task, Worker};

#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// **The steady frame: a component asking for the same thing it asked for last frame.**
///
/// This is all but one frame in a thousand, and it is where a `Box` in the wrong place would cost an
/// allocation *per frame at the frame rate*. The job is boxed on the path that actually asks and
/// nowhere else, which is why `request` takes an `impl FnOnce` rather than a `Box<dyn FnOnce>`.
#[test]
fn a_deduplicated_request_and_an_empty_take_allocate_zero() {
    let worker = Worker::queueing();
    let task: Task<u32> = Task::new(&worker);
    steady(|| {
        let asked = task.request(7, |_cancel| 42);
        black_box(asked);
        black_box(task.take());
    });
    assert_eq!(
        task.request(7, |_cancel| 0),
        Requested::Deduped,
        "one question after a thousand frames of asking"
    );
}

/// **The landing frame.** The payload's `Vec` was allocated by the worker; `take` moves it.
#[test]
fn taking_a_landing_allocates_zero() {
    let worker = Worker::queueing();
    let task: Task<Vec<u8>> = Task::new(&worker);

    // The first round is the warm-up, and it is the identical workload rather than a guess at a
    // prefix of it — the same argument `steady` makes, made by hand because the two rounds have to
    // ask different questions.
    for round in 0..2u64 {
        task.request(round, |_cancel| vec![7u8; 1024]);
        assert!(worker.run(round as usize), "the job landed");
        if round == 0 {
            black_box(task.take());
        } else {
            let (landed, allocations) = count_allocations(|| task.take());
            assert_eq!(allocations, 0, "a landing frame allocated");
            assert_eq!(landed.map(|v| v.len()), Some(1024));
        }
    }
}

/// **Dropping a stale landing allocates zero**, which is what nineteen of the twenty frames in the
/// file-browser scene do.
#[test]
fn dropping_a_stale_landing_allocates_zero() {
    let worker = Worker::queueing();
    let task: Task<Vec<u8>> = Task::new(&worker);

    for round in 0..2u64 {
        // Ask, let it land, then ask something else: the landing in the slot is now an answer to a
        // question nobody is asking.
        task.request(round * 2, |_cancel| vec![7u8; 1024]);
        assert!(worker.run((round * 2) as usize), "the job landed");
        task.request(round * 2 + 1, |_cancel| vec![9u8; 1024]);

        if round == 0 {
            black_box(task.take());
        } else {
            let (landed, allocations) = count_allocations(|| task.take());
            assert_eq!(allocations, 0, "dropping a stale landing allocated");
            assert!(landed.is_none(), "and it was dropped rather than shown");
        }
    }
}

/// **A streamed read costs the frame nothing once the screen's buffer is warm.** `drain_into`
/// appends rather than replacing, which is the whole reason it is spelled that way.
#[test]
fn draining_a_streamed_read_into_a_warm_buffer_allocates_zero() {
    let queue: Drain<u32> = Drain::new();
    let mut screen: Vec<u32> = Vec::with_capacity(1024);
    steady(|| {
        queue.extend(0..100);
        let moved = queue.drain_into(&mut screen);
        assert_eq!(moved, 100);
        screen.clear();
    });
}

/// **The rule, as a measurement: the probe counts a worker thread's allocations and calls them the
/// frame's.**
///
/// The window is opened around a frame that allocates nothing, with a worker decoding inside it, and
/// the count is not zero. That is not a defect in the probe — it counts `alloc` calls, not `alloc`
/// calls *by the app thread*, and there is no cheap way for it to know the difference. It is why
/// every other gate in this file runs on a spawner with no thread in it, and why spec §20 states the
/// rule rather than leaving it to be rediscovered.
#[test]
fn the_probe_counts_a_worker_threads_allocations_and_calls_them_the_frames() {
    let go = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicBool::new(false));
    let (start, finished) = (Arc::clone(&go), Arc::clone(&done));

    let worker = std::thread::spawn(move || {
        while !start.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }
        for _row in 0..10_000 {
            black_box(Vec::<u8>::with_capacity(64));
        }
        finished.store(true, Ordering::Release);
    });

    // The "frame": it allocates nothing, and it is the whole of what the window is meant to describe.
    let ((), attributed) = count_allocations(|| {
        go.store(true, Ordering::Release);
        while !done.load(Ordering::Acquire) {
            std::hint::spin_loop();
        }
    });
    worker.join().expect("the worker returned");

    assert!(
        attributed > 0,
        "the probe attributed nothing to a frame with a decoding worker inside its window, \
         so the rule this file exists for has stopped being true"
    );
}
