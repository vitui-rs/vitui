//! **Runtime ticket 09's reports**: what duplicate detection costs, what the alternative costs, and
//! what the sweep costs.
//!
//! ```text
//! cargo run --release --example id_numbers -p vitui-runtime
//! ```
//!
//! Three numbers, and the middle one is the argument:
//!
//! 1. **A claim is about a nanosecond**, which is what makes duplicate detection part of the design
//!    rather than a debug aid — it runs at every widget on every frame.
//! 2. **Growth is 3.95× for 4× the widgets against the quadratic 13.20×.** `Vec::contains` is the
//!    obvious implementation and it is O(n²); the table was **chosen with the number, not the
//!    argument**, and this is the number.
//! 3. **The sweep is 281 ns**, which buys not having a stale grab swallow the pointer.

use std::hint::black_box;

use vitui_bench::Bench;
use vitui_engine::Rect;
use vitui_runtime::ctx::{Driver, Interest};
use vitui_runtime::id::Id;

/// The frame budget the ratios are against.
const FRAME_NS: f64 = 100_000.0;

/// A dense screen's interactive regions.
const DENSE: u64 = 313;

fn main() {
    println!("runtime ticket 09 — what identity costs\n");
    what_a_claim_costs();
    the_growth_that_chose_the_table();
    what_the_sweep_costs();
}

/// Report: deriving an id, and declaring a dense screen's worth of widgets.
///
/// **Measured through `Ctx::interact`, because `IdTable::claim` is `pub(crate)` and should be** — a
/// component that could claim directly would bypass the hit index and the tracking fold. So the arm
/// below is the real path: derive, claim, append an entry, fold the interest. That is more than a
/// claim and the report says so rather than attributing the whole figure to the table.
fn what_a_claim_costs() {
    let mut driver = Driver::headless(300, 80).expect("sink");
    // Warm, for the first-touch reason the allocation gates give.
    driver.frame(|cx| {
        for i in 0..DENSE {
            cx.interact(
                Id::keyed(Id::ROOT, i),
                Rect::new(0, 0, 4, 1),
                Interest::CLICK,
            );
        }
    });

    let report = Bench::new(40)
        .case("derive an id from a call site", 10_000, || {
            black_box(Id::at(
                black_box(Id::ROOT),
                black_box(std::panic::Location::caller()),
            ));
        })
        .case("derive a keyed id", 10_000, || {
            black_box(Id::keyed(black_box(Id::ROOT), black_box(7)));
        })
        .case("declare a dense screen", 20, || {
            black_box(&mut driver).frame(|cx| {
                for i in 0..DENSE {
                    cx.interact(
                        Id::keyed(Id::ROOT, i),
                        Rect::new(0, 0, 4, 1),
                        Interest::CLICK,
                    );
                }
            });
        })
        .run();
    println!("report  a claim, minimum of 40 rounds:\n{report}");
    let dense = report.get("declare a dense screen").expect("measured");
    println!(
        "        a whole frame declaring {DENSE} widgets: {:.2} us, {:.2}% of a {:.0} us frame\n\
        \x20       per widget {:.2} ns, and that is derive + claim + hit entry + tracking fold —\n\
        \x20       spec §5's 1.1 ns is the claim alone, and 24.1 ns is its figure for identity plus\n\
        \x20       hit registration together, which is the comparable one.\n",
        dense / 1e3,
        dense / FRAME_NS * 100.0,
        FRAME_NS / 1e3,
        dense / DENSE as f64,
    );
}

/// Report and gate: growth against the quadratic alternative.
fn the_growth_that_chose_the_table() {
    /// The obvious implementation, kept so the comparison is a measurement.
    fn contains_claim(seen: &mut Vec<u64>, id: Id) -> bool {
        if seen.contains(&id.raw()) {
            return false;
        }
        seen.push(id.raw());
        true
    }

    let table_at = |n: u64| -> f64 {
        let mut driver = Driver::headless(300, 80).expect("sink");
        driver.frame(|cx| {
            for i in 0..n {
                cx.interact(
                    Id::keyed(Id::ROOT, i),
                    Rect::new(0, 0, 4, 1),
                    Interest::NONE,
                );
            }
        });
        let report = Bench::new(20)
            .case("t", 20, || {
                black_box(&mut driver).frame(|cx| {
                    for i in 0..n {
                        cx.interact(
                            Id::keyed(Id::ROOT, i),
                            Rect::new(0, 0, 4, 1),
                            Interest::NONE,
                        );
                    }
                });
            })
            .run();
        report.get("t").expect("measured")
    };
    let vec_at = |n: u64| -> f64 {
        let report = Bench::new(20)
            .case("v", 20, || {
                let mut seen: Vec<u64> = Vec::new();
                for i in 0..n {
                    black_box(contains_claim(&mut seen, Id::keyed(Id::ROOT, i)));
                }
            })
            .run();
        report.get("v").expect("measured")
    };

    let (t200, t800) = (table_at(200), table_at(800));
    let (v200, v800) = (vec_at(200), vec_at(800));
    let table_ratio = t800 / t200;
    let vec_ratio = v800 / v200;

    println!("report  growth from 200 to 800 widgets — 4x the widgets:");
    println!(
        "          stamped table   {:>9.2} ns -> {:>9.2} ns   {:>6.2}x",
        t200, t800, table_ratio
    );
    println!(
        "          Vec::contains   {:>9.2} ns -> {:>9.2} ns   {:>6.2}x",
        v200, v800, vec_ratio
    );
    println!(
        "        spec §5 measured 3.95x against 13.20x, and 1.1 ns against 22.5 ns a widget.\n\
        \x20       Per widget here: {:.2} ns against {:.2} ns at 800.\n\
        \x20       **This is what chose the table.** The argument for `Vec::contains` is that a\n\
        \x20       screen has a few hundred widgets and a linear scan of a few hundred is nothing —\n\
        \x20       and it is nothing *per widget*, which is the mistake: it is a scan per widget, so\n\
        \x20       it is quadratic per frame.",
        t800 / 800.0,
        v800 / 800.0
    );
    // **A ratio, so it is a gate.** The bound is generous because the number belongs to the load
    // factor and the hash rather than to a machine; what it catches is a change of complexity class.
    assert!(
        table_ratio <= 6.0,
        "the table grew {table_ratio:.2}x for 4x the widgets, which is quadratic territory"
    );
    assert!(
        vec_ratio > table_ratio,
        "the quadratic alternative is no longer worse, so this comparison has stopped measuring \
         what it was written to measure"
    );
    println!(
        "        gate: table growth <= 6x, and worse for the alternative. Measured {table_ratio:.2}x against {vec_ratio:.2}x.\n"
    );
}

/// Report: the sweep, on a dense screen.
fn what_the_sweep_costs() {
    let mut driver = Driver::headless(300, 80).expect("sink");
    let held = Id::named("scrollbar.thumb");

    // A dense frame that draws the holder, so the sweep has the full hit index to search.
    let dense_frame = |driver: &mut Driver| {
        driver.frame(|cx| {
            cx.interact(held, Rect::new(0, 0, 1, 4), Interest::DRAG);
            for i in 0..DENSE {
                cx.interact(
                    Id::keyed(Id::ROOT, i),
                    Rect::new(0, 0, 4, 1),
                    Interest::CLICK,
                );
            }
        });
    };
    dense_frame(&mut driver);
    driver.plant(Some(held), Some(held), Some(held));

    let report = Bench::new(20)
        .case("a dense frame, holder drawing", 20, || {
            dense_frame(black_box(&mut driver));
        })
        .run();
    println!("report  the sweep, inside a dense frame:\n{report}");
    let frame = report
        .get("a dense frame, holder drawing")
        .expect("measured");
    println!(
        "        the whole frame is {:.2} us for {} interactive regions, {:.2}% of a {:.0} us\n\
        \x20       budget. The sweep is three lookups against the hit index inside it — spec §5\n\
        \x20       measured **281 ns** — and it is not separable here, because a frame that does not\n\
        \x20       draw the holder is a different frame rather than the same one without the sweep.\n\
        \x20       What is gated instead is the behaviour, in `ctx::tests`: a stale grab does not\n\
        \x20       survive its widget, and the click record does.",
        frame / 1e3,
        DENSE + 1,
        frame / FRAME_NS * 100.0,
        FRAME_NS / 1e3,
    );
    // And the fact that matters, printed rather than only asserted.
    driver.frame(|_cx| {});
    let (grab, origin, focus, click) = driver.inspect().id_keyed_facts();
    println!(
        "\n        after one frame in which the holder did not draw:\n\
        \x20         grab {grab}   press origin {origin}   focus {focus}   click record {click}\n\
        \x20       Three released, one kept. **The click record is the fourth id-keyed fact and it\n\
        \x20       deliberately does not sweep**, so a double click survives a redraw."
    );
}
