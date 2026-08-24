//! **Runtime ticket 18's report: what a reactivity layer costs, and what it can actually skip.**
//!
//! ```text
//! cargo run --release --example signals_numbers -p vitui-signals
//! ```
//!
//! Four numbers, and the rule this backlog applies to all of them: **a gate is a count, a ratio, an
//! equality or a compile outcome; a timing is a report.** Every count below is also asserted in
//! `tests/`; every timing below is printed and asserted nowhere.
//!
//! 1. **The memo is the one thing a graph can skip**, and the ratio is the report's headline.
//! 2. **The signal layer measured alone**, against a 100 µs frame budget.
//! 3. **The same screen, three drivers**, whose spread is the point rather than any one of them.
//! 4. **What each frame declares**, which is the count that decides the question.

use std::hint::black_box;

use vitui_bench::Bench;
use vitui_runtime::ctx::Driver;

#[allow(dead_code)]
#[path = "../src/board.rs"]
mod board;

use board::{Data, Direct, H, Ink, Press, Signals, Tea, W};

/// The frame budget every ratio is against.
const FRAME_NS: f64 = 100_000.0;

fn main() {
    println!("runtime ticket 18 — what reactivity costs above this runtime\n");
    the_memo();
    the_layer_alone();
    the_three_drivers();
    what_a_frame_declares();
}

/// Report: **the aggregate, folded against memoised, over four fold lengths.**
///
/// The one value on this screen worth not recomputing, and the only thing on it a reactivity layer
/// can honestly claim to skip. All three drivers get the hit, because all three use the runtime's
/// `Memo` — `Computed` is that `Memo` with the key filled in, and what it adds over writing the memo
/// at the call site is that the graph assembles the key instead of the author. Ergonomics, and no
/// mechanism at all.
///
/// **It is a sweep and not one number, and that is a finding.** The wayfinder recorded **132×**
/// (164.55 ns against 1.25 ns) and runtime spec §18 carries the figure. Reproduced here at the
/// screen's own 600 rows it is nearer 29×, because a memo hit is a constant and the fold is O(n):
/// **the ratio belongs to the fold's length, not to the mechanism.** What belongs to the mechanism
/// is the count beside it — `recomputes == 1` — and that is the same number at every length.
fn the_memo() {
    println!("report  the derived aggregate, folded against memoised:");
    let mut ratios = Vec::new();
    for rows in [600usize, 1_200, 2_400, 4_800] {
        let data = Data::new(rows);
        let mut memo = vitui_runtime::data::Memo::new();
        let _ = memo.get(data.revision(), || data.max());

        let folded = Bench::new(40)
            .case("folded", 200, || {
                let _ = black_box(black_box(&data).max());
            })
            .run();
        let folded_ns = folded.get("folded").expect("measured");

        let memoised = Bench::new(40)
            .case("memoised", 200, || {
                let d = black_box(&data);
                let _ = black_box(*memo.get(d.revision(), || d.max()));
            })
            .run();
        let memo_ns = memoised.get("memoised").expect("measured");

        assert_eq!(memo.recomputes, 1, "the fold ran once, which is the gate");
        let ratio = folded_ns / memo_ns.max(f64::MIN_POSITIVE);
        ratios.push(ratio);
        println!(
            "\x20       {rows:>5} rows   folded {folded_ns:>9.2} ns   memoised {memo_ns:>7.2} ns   {ratio:>7.1}x"
        );
    }
    println!(
        "\x20       The screen's own length is the first row. Spec §18 records 132x from the\n\
        \x20       wayfinder's screen, and the sweep is why the two do not have to agree: a memo\n\
        \x20       hit is a constant and the fold is O(n), so the ratio is the data's and\n\
        \x20       `recomputes == 1` is the mechanism's. The count is the gate (`tests/drivers.rs`),\n\
        \x20       measured at {:.1}x-{:.1}x here.\n",
        ratios.iter().copied().fold(f64::INFINITY, f64::min),
        ratios.iter().copied().fold(0.0, f64::max),
    );
}

/// Report: **the whole of a signal layer's per-frame work, measured away from a frame.**
///
/// Read three signals into a `Board`, write one back through the diffed setter, ask the graph
/// whether anything moved. That is everything the layer does that the direct driver does not.
fn the_layer_alone() {
    let app = Signals::new();
    let alone = Bench::new(40)
        .case("read three, write one, ask", 2_000, || {
            let a = black_box(&app);
            let b = a.board();
            a.selected.set(b.selected);
            let _ = black_box(a.graph.is_dirty());
        })
        .run();
    let ns = alone.get("read three, write one, ask").expect("measured");

    println!(
        "report  the signal layer, measured alone:\n\
        \x20       read 3, write 1, ask   {ns:>10.2} ns   {:.4}% of a {:.0} us frame budget\n\
        \x20       The wayfinder measured 4.5 ns and 0.0045%. A reactivity layer is not on the\n\
        \x20       frame path; it is above it, and the frame underneath costs what it costs.\n",
        ns / FRAME_NS * 100.0,
        FRAME_NS / 1000.0,
    );
}

/// Report: **the same screen, three drivers**, and the spread is what the number is for.
///
/// The gate beside this report is the equality — 0 of 24 000 cells differ. The timing is here to say
/// that choosing between them is not a performance decision, and the way to read it is the spread
/// rather than the ranking: three drivers inside each other's run-to-run noise are three drivers
/// that cost the same.
fn the_three_drivers() {
    let data = Data::default();

    let mut d_driver = Driver::headless(W, H).expect("sink");
    let mut d_app = Direct::default();
    let mut d_ink = Ink::new();
    let direct = Bench::new(20)
        .case("direct", 5, || {
            let _ = black_box(&mut d_driver).frame(|cx| {
                let _ = d_app.view(cx, &mut d_ink, &data, Press::Quiet);
            });
        })
        .run();
    let direct_ns = direct.get("direct").expect("measured");

    let mut t_driver = Driver::headless(W, H).expect("sink");
    let mut t_app = Tea::default();
    let mut t_ink = Ink::new();
    let tea = Bench::new(20)
        .case("tea", 5, || {
            let _ = black_box(&mut t_driver).frame(|cx| {
                let _ = t_app.view_and_pump(cx, &mut t_ink, &data, Press::Quiet);
            });
        })
        .run();
    let tea_ns = tea.get("tea").expect("measured");

    let mut s_driver = Driver::headless(W, H).expect("sink");
    let mut s_app = Signals::default();
    let mut s_ink = Ink::new();
    let signals = Bench::new(20)
        .case("signals", 5, || {
            let _ = black_box(&mut s_driver).frame(|cx| {
                let _ = s_app.flush(cx, &mut s_ink, &data, Press::Quiet);
            });
        })
        .run();
    let signals_ns = signals.get("signals").expect("measured");

    let spread = [direct_ns, tea_ns, signals_ns];
    let lo = spread.iter().copied().fold(f64::INFINITY, f64::min);
    let hi = spread.iter().copied().fold(0.0, f64::max);

    println!(
        "report  one quiet frame, {W}x{H}, three drivers:\n\
        \x20       direct                 {direct_ns:>10.2} ns\n\
        \x20       TEA                    {tea_ns:>10.2} ns\n\
        \x20       signals                {signals_ns:>10.2} ns\n\
        \x20       spread                 {:>10.2} ns   {:.2}% of the slowest\n\
        \x20       The equality beside this is the gate: 0 of 24 000 cells differ.\n",
        hi - lo,
        (hi - lo) / hi * 100.0,
    );
}

/// Report: **what a frame declares**, which is the count the whole ticket turns on.
///
/// This one is printed because it reads better beside the timings than it does alone, and it is a
/// gate in `tests/declaration.rs` rather than here.
fn what_a_frame_declares() {
    let data = Data::default();
    let mut driver = Driver::headless(W, H).expect("sink");
    let mut app = Direct::default();
    let mut ink = Ink::new();
    let _ = driver.frame(|cx| {
        let _ = app.view(cx, &mut ink, &data, Press::Quiet);
    });
    let whole = (driver.inspect().hits().len(), driver.inspect().stop_count());

    let b = app.board;
    let _ = driver.frame(|cx| board::footer(cx, &mut ink, &b, 0.0));
    let part = (driver.inspect().hits().len(), driver.inspect().stop_count());

    println!(
        "report  what a frame declares:\n\
        \x20       drawn whole            {:>4} hit entries   {:>3} tab stops\n\
        \x20       one region redrawn     {:>4} hit entries   {:>3} tab stops\n\
        \x20       `Frame::begin` rebuilds the hit index, the ring, the overlay queue and the\n\
        \x20       deadline sink from the draw, so a widget that did not draw is in none of them —\n\
        \x20       and the loss is silent for exactly one frame, because routing answers from the\n\
        \x20       previous frame's index. Fine-grained reactivity here is not expensive; it is a\n\
        \x20       request to revert the hit index.\n",
        whole.0, whole.1, part.0, part.1,
    );
}
