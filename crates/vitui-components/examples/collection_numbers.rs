//! **`collection`: one component, one `Mode`, and every number spec §5 states.**
//!
//! Components ticket 12. The convention is the runtime's — a file in `examples/` named
//! `<subject>_numbers.rs` that prints the numbers a human reads — and so is the rule about what an
//! example may be: **`cargo test` does not run this file**, and no row of
//! [`vitui_components::gates::REGISTER`] rests on it. Rows 68–74 *cite* it and each names a
//! `#[test]` in `src/collect.rs` beside the citation.
//!
//! # What it prints
//!
//! 1. **The component**: four modes, thirteen arms, seven absorbed names, and `size_of::<CollState>()`.
//! 2. **The four stores**, one `Ctrl+A` at a million rows each — the table §5 leads with, measured
//!    rather than remembered. One of them is a keystroke at 234× the whole frame budget.
//! 3. **The frame**, at 1 000 / 100 000 / 1 000 000 rows, against the same screen drawn as a scroll
//!    area. Writes are **identical on both arms**, which is why the equality is on `regions`.
//! 4. **The scan cursor**, as a count of `partition_point` calls rather than as a timing.
//! 5. **Type-ahead**, bounded against unbounded, as a ratio and as microseconds beside it.
//! 6. **What does not reproduce**, said out loud rather than engineered away.
//!
//! # It asserts the shape and not the timings
//!
//! R15's rule, inherited: *a gate is a count, a ratio, an equality or a compile outcome; a timing is
//! a report*. Every `assert!` below is a count or an equality — and this file is on the wrong side
//! of the line for a gate anyway, which is why the counts it shares with the register are asserted
//! in `src/` too.

use std::time::Instant;

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::collect::{
    self, ARMS, COLL_STATE_BYTES, CollState, Mode, SEARCH_BUDGET, SPAN_BYTES,
    SPEC_COLL_STATE_BYTES, Selection, VOLUMES, stores,
};
use vitui_components::listing::{self, Volume};

// **The probe, because `allocations` is a total and a total needs something that counts.**
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// How many frames a per-frame figure is averaged over. Eight, warmed.
const FRAMES: u32 = 8;

/// How many rounds a gesture is timed over, minimum-of-N. The `HashSet` arm is 23 ms a round.
const ROUNDS: u32 = 5;

fn main() {
    println!(
        "`collection` — one component, one `Mode`, {ARMS} match arms, at {}x{}\n",
        listing::W,
        listing::H
    );

    the_component();
    the_stores();
    the_frame();
    the_scan();
    the_type_ahead();
    the_selection_move();
    what_does_not_reproduce();
}

/// **The component, as counts.**
fn the_component() {
    println!("== the component ==\n");
    println!("  modes                     {}", Mode::ALL.len());
    println!(
        "  match arms in `apply`     {} (read out of `src/collect.rs`, not declared)",
        collect::arms_in_apply()
    );
    println!(
        "  names absorbed            {}  {}",
        collect::ABSORBED.len(),
        collect::ABSORBED.join(", ")
    );
    println!("  size_of::<CollState>()    {COLL_STATE_BYTES} B  (§5 says {SPEC_COLL_STATE_BYTES})");
    println!("  size_of::<Span>()         {SPAN_BYTES} B\n");

    for mode in Mode::ALL {
        let mut sel = Selection::new();
        collect::apply(mode, &mut sel, 1_000, collect::Gesture::Plain(4));
        collect::apply(mode, &mut sel, 1_000, collect::Gesture::Extend(9));
        collect::apply(mode, &mut sel, 1_000, collect::Gesture::All);
        println!(
            "  {:<9} after plain(4), extend(9), all: {} selected in {} span(s), lead {}",
            mode.word(),
            sel.count(),
            sel.span_count(),
            sel.lead
        );
    }
    println!();
    assert_eq!(collect::arms_in_apply(), ARMS);
}

/// **The four stores, one `Ctrl+A` at a million rows each.**
fn the_stores() {
    println!("== `Ctrl+A` at {} rows ==\n", VOLUMES[2]);
    println!("  store         time        bytes");

    let len = VOLUMES[2];
    let mut best = u128::MAX;
    let mut spans = Selection::new();
    for _ in 0..ROUNDS {
        let at = Instant::now();
        spans.select_all(len);
        best = best.min(at.elapsed().as_nanos());
    }
    println!(
        "  {:<12}  {:>7.2} us  {:>10} B   <- the shipped one, and it is {} span",
        stores::Store::Spans.word(),
        best as f64 / 1000.0,
        spans.bytes(),
        spans.span_count()
    );

    for store in [
        stores::Store::Bitset,
        stores::Store::VecIndices,
        stores::Store::HashSet,
    ] {
        let mut alt = stores::Alt::new();
        let mut best = u128::MAX;
        for _ in 0..ROUNDS {
            let mut fresh = stores::Alt::new();
            let at = Instant::now();
            fresh.select_all(store, len);
            best = best.min(at.elapsed().as_nanos());
            alt = fresh;
        }
        println!(
            "  {:<12}  {:>7.2} us  {:>10} B",
            store.word(),
            best as f64 / 1000.0,
            alt.bytes(store)
        );
        assert!(alt.contains(store, len / 2), "the store really holds them");
    }
    println!(
        "\n  §5's table: 0.08 us / 16 B  ·  1.71 / 125 KB  ·  164 / 8 MB  ·  23 438 / 16.5 MB.\n  \
         The magnitudes are this machine's; the *shape* is the finding — one span at any length\n  \
         against three stores whose cost is the row count.\n"
    );

    // The pathological selection: a thousand ctrl-clicks on alternate rows.
    let path = collect::alternating(1_000);
    println!(
        "  1 000 ctrl-clicks on alternate rows: {} spans, {} B — proportional to the gestures\n",
        path.span_count(),
        path.bytes()
    );
    assert_eq!(path.span_count(), 1_000);
}

/// **The frame, at three volumes, on both arms.**
fn the_frame() {
    println!("== the frame, {}x{} ==\n", listing::W, listing::H);
    println!("  rows        writes  distinct  verbs  regions  stops  iterated     us/frame");

    let mut first = None;
    for rows in listing::VOLUMES {
        let (shape, _tallied) = listing::volume_over(Volume::Windowed, rows, FRAMES);
        // **`volume_cost` and not `volume_over`'s own duration.** A `Tally` keeps a `BTreeSet` of
        // every cell it sees, so a figure taken with one in the loop is a report about the
        // instrument — 250-odd microseconds against the shipped path's. The two run the identical
        // `draw_into`, which is `crate::ink`'s whole argument.
        let per_frame = listing::volume_cost(Volume::Windowed, rows, FRAMES);
        println!(
            "  {:>9}  {:>6}  {:>8}  {:>5}  {:>7}  {:>5}  {:>8}  {:>11.2}",
            rows,
            shape.writes,
            shape.distinct,
            shape.verbs,
            shape.regions,
            shape.stops,
            shape.iterated,
            per_frame.as_secs_f64() * 1e6
        );
        let seen = *first.get_or_insert(shape);
        assert_eq!(seen, shape, "the frame moved with the data volume");
    }
    println!(
        "\n  §5's frame: 63.9 us / 0 marked / 20 914 writes / 229 regions / 31 stops. That is a\n  \
         *screen* of many components at 300x80, and this is one collection at {}x{} — so the\n  \
         magnitudes are a different measurement and the invariance is the same one.\n",
        listing::W,
        listing::H
    );

    // The other arm, and the one number that cannot see it.
    let windowed = listing::volume(Volume::Windowed, listing::VOLUMES[0]);
    let whole = listing::volume(Volume::WholeContent, listing::VOLUMES[0]);
    println!(
        "  the same screen as a scroll area at {} rows: writes {} against {}, regions {} against {}",
        listing::VOLUMES[0],
        whole.writes,
        windowed.writes,
        whole.regions,
        windowed.regions
    );
    println!(
        "  -> `writes` cannot see it at all, because the engine reports a fully clipped verb as\n  \
         zero columns. The equality is on `regions`.\n"
    );
    assert_eq!(whole.writes, windowed.writes);

    // **Allocations over the shipped path, with the driver warmed outside the counted region.**
    // The five frame structures take their allocation on the first frame that needs one and keep
    // it, so a cold frame is not a frame — the same warming `tests/budget.rs` does.
    let mut driver = vitui_runtime::ctx::Driver::headless(listing::W, listing::H)
        .expect("a sink cannot fail to attach");
    let mut st = CollState::new();
    let opts = vitui_components::collect::CollOpts::default();
    let draw = |driver: &mut vitui_runtime::ctx::Driver, st: &mut CollState| {
        driver.frame(|cx| {
            let area = cx.area();
            let _ = collect::collection(
                cx,
                area,
                st,
                &opts,
                vitui_components::order::Rows::of(VOLUMES[2]),
                &mut |_: &str, _: std::ops::Range<usize>| None,
                &mut |cx: &mut vitui_runtime::Ctx<'_, '_>,
                      r: vitui_runtime::Rect,
                      i: usize,
                      f: vitui_components::frame::Face| {
                    let paint = vitui_components::frame::face_paint(cx.theme(), f);
                    cx.text(r.x, r.y, collect::label(i), paint);
                },
            );
        });
    };
    for _ in 0..4 {
        draw(&mut driver, &mut st);
    }
    let (_, allocations) = count_allocations(|| {
        for _ in 0..FRAMES {
            draw(&mut driver, &mut st);
        }
    });
    println!(
        "  allocations over {FRAMES} warmed frames of the shipped path: {allocations}  \
         (the standing budget is zero)\n"
    );
    assert_eq!(allocations, 0, "the frame allocates");
}

/// **The scan cursor, as a count.**
fn the_scan() {
    println!("== the scan cursor ==\n");
    println!("  window   partition_point calls: scan   naive");
    let sel = collect::alternating(1_000);
    for h in [10usize, 40, 80, 400] {
        let (scan, naive) = collect::seek_and_probes(&sel, 0..h);
        println!("  {h:>6}                          {scan:>5}   {naive:>5}");
        assert_eq!((scan, naive), (1, h));
    }
    println!("\n  O(log k + h) against O(h · log k), as a count rather than as a timing.\n");
}

/// **Type-ahead, bounded against unbounded.**
fn the_type_ahead() {
    println!("== one `z` into a focused {}-row list ==\n", VOLUMES[2]);
    let (bounded, unbounded) = collect::type_ahead_cost(VOLUMES[2], ROUNDS);
    let rows_bounded = collect::search_range(0, VOLUMES[2], SEARCH_BUDGET).len();
    let rows_unbounded = collect::search_range(0, VOLUMES[2], 0).len();
    println!("  budget                 {SEARCH_BUDGET} rows");
    println!(
        "  rows looked at         {rows_bounded} against {rows_unbounded}  ({}x)",
        rows_unbounded / rows_bounded
    );
    println!(
        "  the search alone       {:.2} us against {:.2} us  ({:.1}x)",
        bounded as f64 / 1000.0,
        unbounded as f64 / 1000.0,
        unbounded as f64 / bounded.max(1) as f64
    );
    println!(
        "\n  §5: 70.79 us bounded against 1 507.83 unbounded; the search alone 6.33 against\n  \
         1 334.46 — 211x. The **ratio** is the property of the bound and the microseconds are the\n  \
         weather; 211 implies a budget of about 4 740 rows, and this one is {SEARCH_BUDGET}.\n"
    );
}

/// **What the frame the selection moves re-damages.**
fn the_selection_move() {
    println!("== the frame the selection moves ==\n");
    for rows in [1_000usize, VOLUMES[2]] {
        let (first, moved) = collect::selection_move_repaints(rows, 3, 4);
        println!("  {rows:>9} rows: first frame {first:>5} cells, the move {moved:>4} cells");
    }
    println!(
        "\n  §5 says **50 cells marked, identical at 1k and 1M**. `marked` is unreachable from this\n  \
         crate (the engine's damage module is `pub(crate)` top to bottom), so what is counted is\n  \
         cells a verb *changed* — components ticket 07's substitution. The identity across volumes\n  \
         is what reproduces.\n"
    );
}

/// **What does not reproduce, said out loud.**
fn what_does_not_reproduce() {
    println!("== what does not reproduce ==\n");
    println!(
        "  1. `size_of::<CollState>()` is {COLL_STATE_BYTES} B and §5 says {SPEC_COLL_STATE_BYTES}.\n     \
         The 208 is C03's prototype struct, which carried the three refused stores beside the\n     \
         shipped one and a discriminant to choose between them. Reaching it from here would mean\n     \
         adding a field for the number. What §5 gates is the invariance, and that is exact."
    );
    println!(
        "  2. The frame is one collection at {}x{}; §5's 63.9 us / 20 914 writes / 229 regions /\n     \
         31 stops is a *screen* of many components at 300x80, which this ticket does not own.\n     \
         The invariance across 1k / 100k / 1M is exact.",
        listing::W,
        listing::H
    );
    println!(
        "  3. The type-ahead budget is {SEARCH_BUDGET} rows and §5's 211x implies about 4 740.\n     \
         The bound is a stated number with a stated reason (fifty screens of rows); tuning it to\n     \
         211 would be a constant chosen to reproduce a ratio."
    );
    println!(
        "  4. `marked` is unreachable and stays that way, so the 50-cell figure is measured as\n     \
         cells a verb changed rather than as engine damage."
    );
    println!(
        "\n  And one that is not a number at all: **`CONTEXT.md` spends the word `Span` on a\n  \
         contiguous interval of selected indices, and `crate::scroll::Span` (components 07) spends\n  \
         it on the scrollbar's viewport/extent/offset triple.** Nothing is renamed from this\n  \
         ticket; `collect.rs` never imports the other one, so no line reads *expected `Span`,\n  \
         found `Span`*.\n"
    );

    let mut st = CollState::new();
    st.editing = Some(0);
    assert_eq!(size_of_val(&st), COLL_STATE_BYTES);
}
