//! **Components ticket 44's report**: O6's seven rows, what each of them costs at a million inputs,
//! and how much of a sixty-hertz frame the fold is.
//!
//! ```text
//! cargo run --release --example volume_numbers -p vitui-components
//! ```
//!
//! # What is here and what is not
//!
//! Nothing below is a gate. The gates are `crate::gates::REGISTER` rows 30 and 227–229, and this
//! file is cited beside them as a [`Report`](vitui_components::gates::Instrument::Report) and never
//! instead of one — R15's refinement 2, which components ticket 20 found a whole crate had been
//! breaking: `cargo test` does not run an example, so an `assert!` here is compiled by
//! `cargo clippy --all-targets` and evaluated by nobody.
//!
//! What the report is *for* is the one thing the register rows cannot carry, because §21 makes it a
//! report by name: **the clock**. O6 is gated on two counts — a growth relation and a per-input
//! ceiling — for the runtime scene 19's reason, that a step count is the same number on every
//! machine where a microsecond figure is three orders apart between a debug binary and a release
//! one. The absolute figure still has to be *somewhere*, with its headroom beside it, and this is
//! where.
//!
//! **16.7 ms is the budget and the fold's share is printed as a fraction of it, never as a pass
//! mark.** A component that cannot hold the relation produces a red row with a number in it;
//! lowering a budget to fit a measurement is the edit §21 forbids by name, and `latency` did it for
//! one commit.
//!
//! Run it with `--release`. In an unoptimised binary the microsecond column is a report about
//! `rustc -O0` and the two counted columns are unchanged, which is the whole argument for counting.

use std::time::Instant;

use vitui_components::chart::raster::{Kind, RUNGS, Range, Raster, Reach, domain_of, geom};
use vitui_components::obligations::{VOLUME_MEASURED, o6};
use vitui_components::volume::{
    Arm, BUDGET_NANOS, COVERED, Covered, GROWTH, VOLUMES, Why, measure, population,
};

/// How many times a run is repeated before the clock is read, so one page fault is not the figure.
///
/// The minimum of the rounds and never the mean — `vitui-bench`'s own arrangement, and the reason is
/// that a maximum is somebody else's process and a mean carries it in.
const ROUNDS: u32 = 5;

fn main() {
    println!("O6 — every component that takes a data volume holds 60 Hz at a million inputs\n");

    println!("the population, derived from the freeze and the memo census");
    println!("  {}\n", population().join(", "));
    println!("  verdict          {:?}\n", o6(VOLUME_MEASURED));

    counts();
    println!();
    clock();
    println!();
    remembered();
    println!();
    defects();
}

/// **The two gated numbers**, per row, at the three volumes O6 is stated at.
fn counts() {
    println!(
        "the shipped arms — work is a step count, and a step count is the same number on every \
         machine"
    );
    println!(
        "  {:<11}  {:<18}  {:>12}  {:>12}  {:>12}  {:>13}  {:>10}",
        "component", "why", VOLUMES[0], VOLUMES[1], VOLUMES[2], "growth/decade", "ceiling"
    );
    for row in COVERED {
        let m = measure(row, Arm::Shipped);
        let g = m.growth();
        let c = m.ceiling(row);
        println!(
            "  {:<11}  {:<18}  {:>12}  {:>12}  {:>12}  {:>6.2} {:>6.2}  {:>10}",
            row.id,
            row.why.word(),
            m.work[0],
            m.work[1],
            m.work[2],
            g[0],
            g[1],
            c[2],
        );
    }
    println!(
        "\n  the relation is <= {GROWTH:.2} a decade and the ceiling is `fixed + per_input * n`. A \
         virtualised"
    );
    println!(
        "  row's ceiling has no `n` in it at all, which is `Layer::L2`'s own sentence as \
         arithmetic."
    );
    for row in COVERED.iter().filter(|r| r.why == Why::FoldsOnEdit) {
        let m = measure(row, Arm::Shipped);
        let measured = m.work[2] as f64 / VOLUMES[2] as f64;
        println!(
            "  {:<11}  {:.4} {} an input against a ceiling of {:.2} — {:.2}x of headroom",
            row.id,
            measured,
            row.unit,
            row.per_input,
            row.per_input / measured,
        );
    }
}

/// **The clock, which is the report.** Absolute, with the sixty-hertz budget as the denominator.
fn clock() {
    println!(
        "the same folds on a clock — a REPORT, and the budget is a denominator and not a mark"
    );
    println!(
        "  {:<11}  {:>14}  {:>14}  {:>10}",
        "component", "us at 1M", "us at 100k", "of 16.7 ms"
    );
    for row in COVERED {
        let big = nanos(row, VOLUMES[2]);
        let small = nanos(row, VOLUMES[1]);
        println!(
            "  {:<11}  {:>14.2}  {:>14.2}  {:>9.1}%",
            row.id,
            big as f64 / 1e3,
            small as f64 / 1e3,
            100.0 * big as f64 / BUDGET_NANOS as f64,
        );
    }
    println!(
        "\n  16.7 ms is 60 Hz. A row over 100% is a red row with a number in it, never a budget \
         that moved."
    );
    println!(
        "  The fixture is outside every bracket: the series, the buffer, the flatten index, the \
         driver"
    );
    println!(
        "  and its warm-up frame. A figure that included `\"a\".repeat(1_000_000)` would be a \
         figure about a String."
    );
}

/// One arm's wall clock at one volume, minimum of [`ROUNDS`].
///
/// The fixture is rebuilt every round and is **outside** every bracket — see
/// [`Reading`](vitui_components::volume::Reading), which is where that split lives.
fn nanos(row: &Covered, n: u64) -> u128 {
    (0..ROUNDS)
        .map(|_| (row.run)(Arm::Shipped, n).nanos)
        .min()
        .unwrap_or(0)
}

/// **The two figures ticket 44 states, measured.**
///
/// Its table is over **two million values** — two series of a million, which is what
/// `chart::raster::reduce_tests` folds and what 476.3 ns a point was taken over — and its criterion
/// is stated as *8.5 ms of 16.7 at 1M*. One of the two reproduces.
fn remembered() {
    let series: Vec<Vec<f32>> = (0..2)
        .map(|s| {
            (0..1_000_000)
                .map(|i| (((i + s * 13) % 97) as f32) / 96.0)
                .collect()
        })
        .collect();
    // `RUNGS[1]` and not the repertoire's own name: §16's count is over the whole package, and a
    // sixth file naming one would be a sixth branch. See `crate::volume`'s `BARS_AT`.
    let g = geom(Kind::Bars, RUNGS[1]);
    let dom = domain_of(&series, Range::Whole).padded();
    let mut r = Raster::empty();
    let mut best = u128::MAX;
    for _ in 0..ROUNDS {
        let started = Instant::now();
        r.build(
            60,
            20,
            Kind::Bars,
            g,
            dom,
            &series,
            Reach::Mapped,
            (0, usize::MAX),
        );
        best = best.min(started.elapsed().as_nanos());
    }
    println!("the two figures ticket 44 states, over its own two million values");
    println!(
        "  the reduced bars fold  {:>10.2} us  against the ticket's 2 720.00 — it reproduces",
        best as f64 / 1e3
    );
    println!(
        "  touched / painted      {:>10} / {:<10}  the points, and the paint that is the raster",
        r.touched(),
        r.painted()
    );
    println!();
    println!(
        "  *8.5 ms of 16.7 at 1M* does NOT reproduce: one series of a million is the `chart` row"
    );
    println!(
        "  above, and 8.5 is its share of the budget as a PERCENTAGE. The eight-point-five has been"
    );
    println!("  read as milliseconds. The ticket's own table is the figure that survives.");
}

/// **The seven deliberate defects, and which half of O6 each one fails.**
fn defects() {
    println!("the deliberate defects — every covered row owes one, or the gate is about nothing");
    println!(
        "  {:<11}  {:>11}  {:>13}  {:>11}  {:<9}",
        "component", "at inputs", "growth/decade", "work", "fails"
    );
    for row in COVERED {
        let m = measure(row, Arm::Defective);
        let g = m.growth();
        let fails = match (m.within_growth(), m.within_ceiling(row)) {
            (true, false) => "ceiling",
            (false, true) => "relation",
            (false, false) => "both",
            (true, true) => "NOTHING",
        };
        println!(
            "  {:<11}  {:>11}  {:>6.2} {:>6.2}  {:>11}  {:<9}",
            row.id, m.volumes[2], g[0], g[1], m.work[2], fails
        );
    }
    println!();
    println!(
        "  `chart`'s is the defect that actually SHIPPED, and it fails the ceiling alone: the whole"
    );
    println!(
        "  column prefix painted once a point is `O(subh)` inside a visit, which is `O(n)` with"
    );
    println!(
        "  `subh` in front of it — so a growth relation reads it as healthy at ten a decade. \
         `plot`'s"
    );
    println!(
        "  is quadratic and is what the relation itself is watched failing on. Neither half would do"
    );
    println!("  on its own, and that is measured rather than argued.");
    println!();
    for row in COVERED {
        println!("  {:<11}  {}", row.id, row.defect);
    }
}
