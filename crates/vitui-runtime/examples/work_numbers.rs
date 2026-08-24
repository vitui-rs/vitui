//! **Runtime ticket 16's report**: what the handoff costs, and where cancellation stops paying.
//!
//! ```text
//! cargo run --release --example work_numbers -p vitui-runtime
//! ```
//!
//! # What is reported and what is asserted
//!
//! Reported, because they are timings: `Worker::ask` against `std::thread::spawn`, the deduplicated
//! `request` a component makes every frame, the empty `take`, and the wall time of a streamed read
//! through the two shapes.
//!
//! Asserted, because they are counts that belong to the mechanism rather than to this machine:
//!
//! - the sweep is **twenty** selections and a decode is **210** units, so a ratio printed against
//!   them means something;
//! - the arm that ignores the flag does **20.0×** the necessary work at every interval, exactly,
//!   because none of its twenty jobs stops early — that is arithmetic, not a measurement;
//! - a streamed read through a slot of the whole copies **505 000** elements against a queue's
//!   **10 000**.
//!
//! # The crossover is the finding, and it is why this is a report rather than three gates
//!
//! *Cancellation saves nothing while the decode is faster than the user.* At a 30 ms key repeat every
//! job has finished before it is superseded, so there is nothing to cancel and nothing to replace;
//! the twentyfold waste is intrinsic and the only cure is not to ask. What the resident worker saves
//! at 1 ms it saves **in the inbox**, before the job exists. The two mechanisms win in opposite
//! conditions and neither subsumes the other — which is a shape, and a shape is what a report is for.
//!
//! # Provenance
//!
//! **R 16** took these numbers and **R 20** re-ran the report on 2026-08-24: Apple M1 Max, macOS
//! 26.5.2, rustc 1.97.1, `--release`, unloaded, minimum of 40 rounds. The handoff's parts read
//! `Worker::ask` 71.0 ns, `Task::request` deduplicated 2.3 ns, `Slot::put + take` 18.4 ns.
//!
//! **`crate::ledger`'s async row is still the prototype's −0.18 µs, and it is the one unreproduced
//! row worth flagging** — because it is negative. This file measures the handoff's parts and no
//! frame delta at all, so nothing here has confirmed that the mechanism gives a frame time back;
//! until something does, that row is *buying* the ledger headroom it has not earned. An unreproduced
//! cost overstates the crate's own share and is safe; an unreproduced saving understates it and is
//! not, which is why `ledger::tests::the_table_says_how_many_rows_are_still_the_prototypes` names
//! this one by hand rather than counting to three.
//!
//! Nothing in this file divides by the frame budget, and that is deliberate rather than an
//! oversight: the crossover this report exists to show is between a key repeat and a decode, so its
//! denominators are milliseconds of human timing and counts of units — 20 selections, 210 units,
//! 505 000 elements against 10 000 — none of which a frame budget makes more legible.

use std::hint::black_box;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};

use vitui_bench::Bench;
use vitui_engine::{Clock, Config, Engine, Output, Slot, WakeHandle};
use vitui_runtime::work::{Drain, Task, Worker};

/// An arrow-key sweep down a directory.
const SELECTIONS: u64 = 20;
/// One decode, in units a job can stop between.
const UNITS: u64 = 210;
/// About how long a decode takes, which is what the intervals below are compared against.
const DECODE_MS: u64 = 5;
/// The three key-repeat rates: faster than the decode, about the decode, slower than the decode.
const INTERVALS_MS: [u64; 3] = [1, 10, 30];

/// The directory read: 10 000 rows in 100 batches.
const ROWS: usize = 10_000;
const BATCHES: usize = 100;

fn main() {
    let (_screen, wake) = sink_engine();

    println!("runtime ticket 16 — the handoff, and where cancellation stops paying\n");
    handoff(&wake);
    crossover(&wake);
    streamed();
}

/// What it costs to ask, to look, and to spawn instead.
fn handoff(wake: &WakeHandle) {
    let worker = Worker::hire(wake.clone());
    let task: Task<u32> = Task::new(&worker);
    // One question asked, so every `request` below is the deduplicated frame a component actually
    // makes — the shape that runs sixty times a second for ever.
    task.request(1, |_cancel| 42);

    let outbox: Slot<u32> = Slot::new();
    // **The question carries a payload, and the first version of this report did not.** A closure
    // that captures nothing boxes without allocating, so the measured `ask` was 21.5 ns — a fifth of
    // the real number, and a comparison against a thread that flattered the shipped shape by four
    // hundred per cent. A question is a path or an index; this one is the smallest honest stand-in.
    let question = [1u64, 2, 3, 4];

    let report = Bench::new(9)
        .case("Worker::ask", 200, || {
            worker.ask(move || {
                black_box(question);
            });
        })
        .case("thread::spawn", 20, || {
            // The handle is dropped, which detaches: this is the cost of *starting* a thread and
            // nothing else, which is the number the resident worker is being compared against.
            drop(std::thread::spawn(move || {
                black_box(question);
            }));
        })
        .case("Task::request, deduplicated", 500, || {
            black_box(task.request(1, |_cancel| 42));
        })
        .case("Task::take, empty", 500, || {
            black_box(task.take());
        })
        .case("Slot::put + take", 500, || {
            outbox.put(7);
            black_box(outbox.take());
        })
        .run();

    println!("the handoff");
    for (name, ns) in report.rows() {
        println!("  {name:<28} {ns:>9.1} ns");
    }
    if let (Some(ask), Some(spawn)) = (report.get("Worker::ask"), report.get("thread::spawn")) {
        println!(
            "  a resident worker's inbox against a thread: {:.0}x",
            spawn / ask
        );
        println!(
            "  over a {SELECTIONS}-key sweep: {:.1} us of handoff against {:.1} us of thread creation",
            ask * SELECTIONS as f64 / 1000.0,
            spawn * SELECTIONS as f64 / 1000.0
        );
    }
    println!();
}

/// The crossover: units of work performed over a twenty-key sweep, three arms, three intervals.
///
/// **Units rather than microseconds**, because the claim is a ratio against the necessary work and a
/// ratio of counts is the same number on every machine. The unit itself is a busy wait, so a decode
/// is about `DECODE_MS` however fast the machine is — which is what puts the three intervals on
/// either side of it.
fn crossover(wake: &WakeHandle) {
    println!("cancellation, as a multiple of the one decode anybody wanted ({UNITS} units)");
    println!("  interval   polled   ignored   resident worker");
    for ms in INTERVALS_MS {
        let interval = Duration::from_millis(ms);
        let polled = per_job_threads(interval, true);
        let ignored = per_job_threads(interval, false);
        let resident = resident_worker(wake, interval);

        assert_eq!(
            ignored,
            SELECTIONS * UNITS,
            "the arm that never looks at the flag does every unit of every job, by construction"
        );
        println!(
            "  {ms:>5} ms   {:>5.1}x   {:>6.1}x   {:>10.1}x",
            polled as f64 / UNITS as f64,
            ignored as f64 / UNITS as f64,
            resident as f64 / UNITS as f64
        );
    }
    println!(
        "  cancellation saves nothing while the decode (~{DECODE_MS} ms) is faster than the user\n"
    );
}

/// The shape an application writes first: a thread per selection, with a flag it may or may not
/// poll.
fn per_job_threads(interval: Duration, polls: bool) -> u64 {
    let done = Arc::new(AtomicU64::new(0));
    let mut cancel: Option<Arc<AtomicBool>> = None;
    let mut threads = Vec::new();

    for _selection in 0..SELECTIONS {
        if let Some(previous) = cancel.take() {
            previous.store(true, Ordering::Relaxed);
        }
        let flag = Arc::new(AtomicBool::new(false));
        cancel = Some(Arc::clone(&flag));
        let counted = Arc::clone(&done);
        threads.push(std::thread::spawn(move || {
            for _unit in 0..UNITS {
                unit();
                counted.fetch_add(1, Ordering::Relaxed);
                if polls && flag.load(Ordering::Relaxed) {
                    break;
                }
            }
        }));
        std::thread::sleep(interval);
    }
    for thread in threads {
        thread.join().expect("a decode returned");
    }
    done.load(Ordering::Relaxed)
}

/// The shipped shape: one resident thread, one question at a time, and the flag as well.
fn resident_worker(wake: &WakeHandle, interval: Duration) -> u64 {
    let done = Arc::new(AtomicU64::new(0));
    let worker = Worker::hire(wake.clone());
    let task: Task<u64> = Task::new(&worker);

    for selection in 0..SELECTIONS {
        let counted = Arc::clone(&done);
        task.request(selection, move |cancel| {
            let mut units = 0;
            for _unit in 0..UNITS {
                unit();
                units += 1;
                counted.fetch_add(1, Ordering::Relaxed);
                if cancel.cancelled() {
                    break;
                }
            }
            units
        });
        std::thread::sleep(interval);
    }
    // Dropping the worker joins, so every unit it was ever going to do has been counted.
    drop(worker);
    done.load(Ordering::Relaxed)
}

/// One unit of a decode: a busy wait, so the decode is the same length wherever this runs.
fn unit() {
    let until = Instant::now() + Duration::from_micros(DECODE_MS * 1000 / UNITS);
    while Instant::now() < until {
        std::hint::spin_loop();
    }
}

/// The directory read, through the three shapes.
fn streamed() {
    // The counts first, because they are the claim; the timings are what they cost.
    let mut cumulative = 0usize;
    let mut whole = Vec::new();
    for batch in 0..BATCHES {
        whole.extend(batch * (ROWS / BATCHES)..(batch + 1) * (ROWS / BATCHES));
        cumulative += whole.len();
    }
    assert_eq!(cumulative, 505_000, "the cumulative shape's element count");

    let queue: Drain<usize> = Drain::new();
    let mut screen = Vec::with_capacity(ROWS);
    let report = Bench::new(7)
        .case("slot of the whole", 5, || {
            let outbox: Slot<Vec<usize>> = Slot::new();
            let mut accumulated = Vec::new();
            for batch in 0..BATCHES {
                accumulated.extend(batch * (ROWS / BATCHES)..(batch + 1) * (ROWS / BATCHES));
                outbox.put(accumulated.clone());
            }
            black_box(outbox.take());
        })
        .case("Drain", 5, || {
            for batch in 0..BATCHES {
                queue.extend(batch * (ROWS / BATCHES)..(batch + 1) * (ROWS / BATCHES));
            }
            queue.drain_into(&mut screen);
            screen.clear();
        })
        .run();

    println!("a directory read of {ROWS} rows in {BATCHES} batches");
    println!(
        "  elements copied: {cumulative} cumulative against {ROWS} through a queue — {:.1}x",
        cumulative as f64 / ROWS as f64
    );
    for (name, ns) in report.rows() {
        println!("  {name:<28} {:>9.1} us", ns / 1000.0);
    }
    println!(
        "  and a slot of deltas loses 9 000 of the 10 000, which is the gate, not this report"
    );
}

/// A screen over a sink, only for the [`WakeHandle`] a resident worker needs.
fn sink_engine() -> (vitui_engine::Screen, WakeHandle) {
    Engine::new(Config {
        size: (80, 24),
        output: Output::Sink(Box::new(Vec::new())),
        clock: Clock::System,
        ..Default::default()
    })
    .attach()
    .expect("attaching to a sink cannot fail")
}
