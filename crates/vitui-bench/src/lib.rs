//! Round-robin, minimum-of-N measurement, with no dependencies.
//!
//! This crate exists because `criterion` was removed, and it was removed for two independent
//! reasons that arrived from opposite directions: it brings a dependency tree this workspace's
//! policy will not have, and its statistics answer a question — *how long does this take on a
//! quiet machine* — that no gate here asks. What the gates ask is a **ratio between two
//! variants measured under the same interference**, which is the two sections below.
//!
//! # Why the minimum and not the mean
//!
//! An observed duration is the true duration plus interference, and interference is non-negative.
//! On the machine this project is being designed on, the load average wandered between 6 and 130
//! across the map, so a confidence interval computed from that sample describes the neighbours
//! rather than the code. The minimum is the sample least contaminated by them. Tickets 08, 14 and
//! 16 each reached this conclusion separately and each hand-rolled the same loop; this crate is
//! that loop, written once.
//!
//! # Why round-robin and not one case at a time
//!
//! A ratio between two variants is only meaningful if both saw the same interference. Running
//! `A × 40` and then `B × 40` lets a background job land entirely inside one of them. Running
//! `A, B, A, B, …` spreads any such event across both. The absolute numbers are still worth what
//! the machine is worth; the ratio survives.
//!
//! # Usage
//!
//! ```
//! use vitui_bench::Bench;
//!
//! let report = Bench::new(8)
//!     .case("sum_fold", 1_000, || {
//!         std::hint::black_box((0u64..64).sum::<u64>());
//!     })
//!     .case("sum_loop", 1_000, || {
//!         let mut acc = 0u64;
//!         for i in 0..64 {
//!             acc += i;
//!         }
//!         std::hint::black_box(acc);
//!     })
//!     .run();
//!
//! assert_eq!(report.len(), 2);
//! assert!(report.get("sum_fold").is_some());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt::Write as _;
use std::time::{Duration, Instant};

type Case<'a> = (&'a str, u32, Box<dyn FnMut() + 'a>);

/// A set of cases measured against each other under the same interference.
pub struct Bench<'a> {
    rounds: u32,
    cases: Vec<Case<'a>>,
}

impl<'a> Bench<'a> {
    /// A new bench that will run every case `rounds` times.
    ///
    /// Forty is what the map's tickets settled on. Fewer than about eight makes the minimum a
    /// coin toss on a loaded machine.
    pub fn new(rounds: u32) -> Self {
        assert!(rounds > 0, "a bench with no rounds measures nothing");
        Bench {
            rounds,
            cases: Vec::new(),
        }
    }

    /// Add a case. `iters` is how many times the closure's work is repeated inside one timed
    /// sample; the reported duration is per iteration.
    ///
    /// Cases are run in insertion order within every round, so a case that must not be advantaged
    /// by a warm cache should not be added first.
    pub fn case(mut self, name: &'a str, iters: u32, f: impl FnMut() + 'a) -> Self {
        assert!(iters > 0, "case `{name}` would be timed over no work");
        self.cases.push((name, iters, Box::new(f)));
        self
    }

    /// Run every case in every round and keep the minimum per-iteration duration for each.
    ///
    /// The minimum is kept as the whole batch and divided in floating point at report time.
    /// Dividing a `Duration` truncates to whole nanoseconds, and this project quotes figures like
    /// 0.9 ns per damaged cell and 1.4 ns per packed cell, which that truncation would report as
    /// zero.
    pub fn run(mut self) -> Report {
        let mut best: Vec<Duration> = vec![Duration::MAX; self.cases.len()];

        // One untimed round first: every case pays its own first-touch costs before any case is
        // measured, so the first entry in the list is not charged for warming the others' data.
        for (_, iters, f) in self.cases.iter_mut() {
            for _ in 0..*iters {
                f();
            }
        }

        for _ in 0..self.rounds {
            for (i, (_, iters, f)) in self.cases.iter_mut().enumerate() {
                let start = Instant::now();
                for _ in 0..*iters {
                    f();
                }
                let batch = start.elapsed();
                if batch < best[i] {
                    best[i] = batch;
                }
            }
        }

        Report {
            rows: self
                .cases
                .iter()
                .zip(best)
                .map(|((name, iters, _), batch)| Row {
                    name: (*name).to_owned(),
                    per_iter_nanos: batch.as_secs_f64() * 1e9 / f64::from(*iters),
                })
                .collect(),
        }
    }
}

struct Row {
    name: String,
    per_iter_nanos: f64,
}

/// The minimum per-iteration cost of each case, in the order the cases were added.
pub struct Report {
    rows: Vec<Row>,
}

impl Report {
    /// How many cases were measured.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether nothing was measured.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// The minimum for one case, in nanoseconds per iteration.
    pub fn get(&self, name: &str) -> Option<f64> {
        self.rows
            .iter()
            .find(|r| r.name == name)
            .map(|r| r.per_iter_nanos)
    }

    /// Every case, in insertion order, as nanoseconds per iteration.
    pub fn rows(&self) -> impl Iterator<Item = (&str, f64)> {
        self.rows
            .iter()
            .map(|r| (r.name.as_str(), r.per_iter_nanos))
    }

    /// Fail if a case is at or above `budget`.
    ///
    /// This is the form a performance **gate** takes. A gate names one case and one number; a
    /// report that prints many numbers and asserts none of them is not a gate.
    #[track_caller]
    pub fn assert_under(&self, name: &str, budget: Duration) {
        let measured = self
            .get(name)
            .unwrap_or_else(|| panic!("no case named `{name}` was measured"));
        let budget_nanos = budget.as_secs_f64() * 1e9;
        assert!(
            measured < budget_nanos,
            "`{name}` measured {measured:.2} ns, budget is {budget_nanos:.2} ns"
        );
    }
}

impl std::fmt::Display for Report {
    /// A table, with each row's ratio against the fastest case.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fastest = self
            .rows
            .iter()
            .map(|r| r.per_iter_nanos)
            .fold(f64::INFINITY, f64::min);
        let width = self.rows.iter().map(|r| r.name.len()).max().unwrap_or(0);

        let mut out = String::new();
        for row in &self.rows {
            let _ = write!(
                out,
                "  {:<width$}  {:>12.2} ns",
                row.name, row.per_iter_nanos
            );
            // A ratio against a zero baseline says nothing, and a case that measures zero has
            // been optimised away rather than made fast.
            if fastest > 0.0 {
                let _ = write!(out, "  {:>6.2}x", row.per_iter_nanos / fastest);
            }
            let _ = writeln!(out);
        }
        f.write_str(&out)
    }
}
