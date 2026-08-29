//! **The series screen: the raster, the ladder, the three false greens and the axis sweep, as
//! numbers.**
//!
//! Components tickets 27 and 28. The convention is the runtime's — a file in `examples/` named
//! `<subject>_numbers.rs` that prints the numbers a human reads — and so is the rule about what an
//! example may be: **`cargo test` does not run this file.** Rows 68 and 69 of
//! [`vitui_components::gates::REGISTER`] *cite* it, and each names a `#[test]` in `src/series.rs`
//! beside the citation.
//!
//! # What it prints
//!
//! 1. **The two scenes**, with where each stands and which ticket inverts it.
//! 2. **The frame**, at 300x80 and 60x20, at 1 000 / 100 000 / 1 000 000 points — with the
//!    allocation total beside it, because a mean over `n` frames cannot see a frame that allocates
//!    on `n - 1` of them.
//! 3. **The two-size axis**, which is the one this screen is played at two sizes for.
//! 4. **The three false greens**, each with its cell count *and* its speed advantage, because the
//!    speed advantage is the reason each of them gets shipped.
//! 5. **The repertoire ladder**, and the two cell counts that decide which rung `plot` earns.
//! 6. **The axis sweep**, over all 175 712 pairs.
//! 7. **The edit**, which is the one cost the invariant permits to be proportional to the data.
//! 8. **The memo chain and the colour**, which is components ticket 28's half.
//! 9. **What does not reproduce**, said out loud rather than engineered away.
//!
//! # It asserts the shape and not the timings
//!
//! Every `assert!` below is a count or an equality. The timings are printed and nothing rests on
//! them — R15's rule, inherited: *a gate is a count, a ratio, an equality or a compile outcome; a
//! timing is a report.* On this screen that rule earns its keep three times over, because **all
//! three false greens are faster or level and every one of them is wrong.**

use std::time::Duration;

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::chart::raster::KeyMode;
use vitui_components::gates::Standing;
use vitui_components::scenes::{SCENES, scenes_for};
use vitui_components::series::{
    self, AXIS_PAIRS, Build, COMPARED_AT, H, Kind, NARROW_H, NARROW_W, RUNGS, Range, Reach, SERIES,
    Session, VOLUMES, W,
};
use vitui_runtime::{ColorDepth, Role};

// **The probe, because `allocations` is a total and a total needs something that counts.** Installed
// here rather than defaulted to zero inside the library: a figure defaulted to zero is a counter
// that prints `0` when it means *nobody counted*.
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// How many frames a per-frame figure is taken over. **The minimum of eight, warmed**, which is
/// `vitui-bench`'s own rule: a minimum is a measurement of the machine at its least interrupted and
/// a mean is a measurement of everything else that was running.
const FRAMES: u32 = 8;

fn main() {
    println!(
        "The series screen — {W}x{H} and {NARROW_W}x{NARROW_H}, two series, and the two scenes \
         `chart` and `plot` are screens of\n"
    );

    scene_list();
    the_frame();
    the_two_size_axis();
    the_false_greens();
    the_ladder();
    the_axis_sweep();
    the_edit();
    the_chain_and_the_colour();
    what_does_not_reproduce();
}

/// 1. The two scenes, and which ticket inverts each.
fn scene_list() {
    println!("report  the two scenes, and where each stands:");
    println!(
        "  {:>3}  {:<58}  {:<12}  inverted by",
        "#", "scene", "standing"
    );
    let mut stood = 0usize;
    for scene in SCENES.iter().filter(|s| s.stands.contains(&"plot")) {
        let (word, ticket) = match scene.standing {
            Standing::Red { inverted_by, .. } => ("red, pinned", inverted_by),
            Standing::Evaluated { .. } => {
                stood += 1;
                ("evaluated", "-")
            }
            Standing::Unsubjected { inverted_by } => ("unsubjected", inverted_by),
            Standing::Unreachable { inverted_by, .. } => ("unreachable", inverted_by),
        };
        println!(
            "  {:>3}  {:<58}  {word:<12}  {ticket}",
            scene.number, scene.name
        );
    }
    assert_eq!(
        stood, 4,
        "both scenes stand on `chart` and `plot`, which components 28 declared, and the two \
         galleries are the third and fourth since components 40 and 41. **It read 3 for five \
         tickets while the answer was 4** — `cargo test` does not run an example"
    );
    println!(
        "\n  scenes_for(\"chart\") answers {} and scenes_for(\"plot\") answers {}",
        scenes_for("chart").count(),
        scenes_for("plot").count()
    );
    println!("  standing: {:?}\n", series::standing());
}

/// 2. The frame, at both sizes and all three volumes.
fn the_frame() {
    println!("report  the frame costs the rectangle; the edit costs the data:");
    println!(
        "  {:<9}  {:>8}  {:>8}  {:>8}  {:>6}  {:>7}  {:>6}  {:>10}  {:>6}",
        "size", "points", "writes", "distinct", "verbs", "regions", "allocs", "touched", "raster"
    );
    for (size, expect) in [
        ((W, H), series::WRITES),
        ((NARROW_W, NARROW_H), series::NARROW_WRITES),
    ] {
        for points in VOLUMES {
            let shape = series::shape(Build::correct(), size, points);
            // **A total and never a mean.** Eight frames, counted whole: a frame allocating on seven
            // of eight reports 0 as a mean and 7 as a total.
            let mut session = Session::open(Build::correct(), size, points);
            let (_, allocs) = count_allocations(|| {
                for _ in 0..FRAMES {
                    session.frame();
                }
            });
            println!(
                "  {:<9}  {points:>8}  {:>8}  {:>8}  {:>6}  {:>7}  {allocs:>6}  {:>10}  {:>6}",
                format!("{}x{}", size.0, size.1),
                shape.writes,
                shape.distinct,
                shape.verbs,
                shape.regions,
                shape.touched,
                shape.raster_bytes,
            );
            assert_eq!(shape.writes, expect);
            assert_eq!(shape.distinct, expect, "the screen is a partition");
            assert!(shape.verbs <= shape.writes, "verbs <= writes");
            assert_eq!(
                allocs, 0,
                "a steady frame allocated, as a total over {FRAMES}"
            );
        }
        println!(
            "  {:<9}  {:>8}  frame {:?} at 1k, {:?} at 1M — a report, never a gate",
            "",
            "",
            series::cost(Build::correct(), size, VOLUMES[0], FRAMES),
            series::cost(Build::correct(), size, VOLUMES[2], FRAMES),
        );
    }
    println!(
        "\n  a 60x20 raster is {} B at {}, {} and {} points\n",
        2 * 60 * 20,
        VOLUMES[0],
        VOLUMES[1],
        VOLUMES[2]
    );
}

/// 3. The two-size axis: one boolean between the arms, and it only shows at 60x20.
fn the_two_size_axis() {
    println!("report  the legend that does not narrow — green wide, red narrow:");
    let bad = Build {
        narrowing_legend: false,
        ..Build::correct()
    };
    for size in [(W, H), (NARROW_W, NARROW_H)] {
        let good = series::shape(Build::correct(), size, VOLUMES[0]);
        let broken = series::shape(bad, size, VOLUMES[0]);
        let diff = series::split_diff(
            &series::render(Build::correct(), size, VOLUMES[0]),
            &series::render(bad, size, VOLUMES[0]),
            series::whole(size),
        );
        println!(
            "  {:<7}  correct {:>5} writes / {:>4} verbs | defective {:>5} / {:>4}, {} written \
             twice | surface {} cells over {} rows",
            format!("{}x{}", size.0, size.1),
            good.writes,
            good.verbs,
            broken.writes,
            broken.verbs,
            broken.writes - broken.distinct,
            diff.cluster,
            diff.rows,
        );
    }
    println!(
        "  the defective arm costs one verb FEWER at {NARROW_W}x{NARROW_H} and is identical at \
         {W}x{H}: every counter but the pair prefers it\n"
    );
}

/// 4. The three false greens, with the speed advantage that is the reason each of them ships.
fn the_false_greens() {
    println!(
        "report  three false greens, each measured on the rendered surface at {COMPARED_AT} points:"
    );
    let good = series::render(Build::correct(), (W, H), COMPARED_AT);
    let base = series::cost(Build::correct(), (W, H), COMPARED_AT, FRAMES);
    println!(
        "  {:<38}  {:>7}  {:>8}  {:>7}  {:>9}  speed",
        "spelling", "cells", "blanked", "rows", "writes"
    );
    println!(
        "  {:<38}  {:>7}  {:>8}  {:>7}  {:>9}  {:?}",
        "correct",
        0,
        0,
        0,
        series::shape(Build::correct(), (W, H), COMPARED_AT).writes,
        base
    );
    for (name, build) in [
        (
            "culled to the visible index window",
            Build::correct().both(|o| o.reach = Reach::CulledByIndex),
        ),
        (
            "an axis range from a stride sample",
            Build::correct().both(|o| o.range = Range::Sampled(1_000)),
        ),
        (
            "every n-th point instead of the union",
            Build::correct().both(|o| o.reach = Reach::Strided),
        ),
    ] {
        let bad = series::render(build, (W, H), COMPARED_AT);
        let diff = series::split_diff(&good, &bad, series::whole((W, H)));
        let shape = series::shape(build, (W, H), COMPARED_AT);
        let cost = series::cost(build, (W, H), COMPARED_AT, FRAMES);
        println!(
            "  {name:<38}  {:>7}  {:>8}  {:>7}  {:>9}  {:?} ({:.2}x)",
            diff.cluster,
            diff.blanked,
            diff.rows,
            shape.writes,
            cost,
            ratio(cost, base),
        );
        assert!(
            diff.cluster > 0,
            "{name} changed nothing, so the case is inert"
        );
        assert_eq!(shape.writes, series::WRITES, "{name} moved the write count");
    }
    let data = series::Series::build(COMPARED_AT, SERIES);
    let whole = series::domain_of(data.points(), Range::Whole).padded();
    let sampled = series::domain_of(data.points(), Range::Sampled(1_000)).padded();
    println!(
        "  the sampled range's domain is {:.2}..{:.2} against {:.2}..{:.2} — it never saw a spike\n",
        sampled.y0, sampled.y1, whole.y0, whole.y1
    );
}

/// 5. The ladder, and which rung `plot` earns.
fn the_ladder() {
    println!("report  the sub-cell ladder, and which rung is earned by which construction:");
    println!(
        "  {:<14}  {:<18}  {:<18}  {:<18}",
        "construction", "Ascii", "Unicode", "Extended"
    );
    for kind in [Kind::Bars, Kind::Marks] {
        let cells: Vec<String> = RUNGS
            .iter()
            .map(|set| {
                let g = series::geom(kind, *set);
                format!("{}x{}, {:>3} states", g.sx, g.sy, g.states(kind))
            })
            .collect();
        println!(
            "  {:<14}  {:<18}  {:<18}  {:<18}",
            kind.word(),
            cells[0],
            cells[1],
            cells[2]
        );
    }
    let unicode = series::render(Build::correct().at(RUNGS[1]), (W, H), COMPARED_AT);
    let extended = series::render(Build::correct().at(RUNGS[2]), (W, H), COMPARED_AT);
    let ascii = series::render(Build::correct().at(RUNGS[0]), (W, H), COMPARED_AT);
    println!(
        "  Unicode against Extended: {} cells in the CHART pane, {} in the PLOT pane",
        series::split_diff(&unicode, &extended, series::chart_pane((W, H))).cluster,
        series::split_diff(&unicode, &extended, series::plot_pane((W, H))).cluster,
    );
    let ascii_diff = series::split_diff(&ascii, &unicode, series::whole((W, H)));
    println!(
        "  Ascii against Unicode:    {} cells over {} of {H} rows — a different construction",
        ascii_diff.cluster, ascii_diff.rows
    );
    println!(
        "  cells of the plot pane where two series share one: {} — the third rung's price\n",
        series::SHARED_CELLS
    );
}

/// 6. The axis sweep.
fn the_axis_sweep() {
    println!("report  the axis loop, over all {AXIS_PAIRS} viewport x dataset pairs:");
    let tally = series::axis_sweep();
    println!(
        "  auto-scaled to the window, fed back:  converged {} in <= {} passes, OSCILLATES {}",
        tally.converged, tally.max_passes, tally.oscillates
    );
    println!(
        "  last frame's plotting width:          no loop; settles on a gutter that is not a fixed \
         point on {} of {}",
        tally.hysteresis_settled_off, tally.oscillates
    );
    println!(
        "  the whole domain:                     {} pairs, {} oscillations, always {} pass\n",
        tally.whole_pairs, tally.whole_oscillates, tally.whole_max_passes
    );
    assert_eq!(tally.pairs, AXIS_PAIRS);
    assert_eq!(tally.whole_oscillates, 0);
}

/// 7. The edit, which is where the data volume is allowed to appear.
fn the_edit() {
    println!("report  the edit — the one cost the invariant permits to be the data's:");
    for points in VOLUMES {
        let (cost, touched) =
            series::edit_cost(points, 170, 78, Kind::Marks, RUNGS[2], Reach::Mapped);
        println!("  Raster::build over {points:>9} points: {cost:?} over {touched} touched");
    }
    let hit = series::hit_cost(VOLUMES[2], 170, 78, Kind::Marks, RUNGS[2], 100_000);
    let (miss, _) = series::edit_cost(VOLUMES[2], 170, 78, Kind::Marks, RUNGS[2], Reach::Mapped);
    println!(
        "  the same call on a hit:              {hit:?}  — {:.0}x\n",
        ratio(hit, miss)
    );
}

/// 8. The memo chain, the key that is every input, and the colour. Components ticket 28.
fn the_chain_and_the_colour() {
    println!("report  the memo chain, and the key that is every input:");
    println!(
        "  {:<32}  {:>14}  {:>12}  {:>14}",
        "keyed on", "raster folds", "range folds", "cells wrong"
    );
    for mode in [KeyMode::Full, KeyMode::DataRect, KeyMode::DataOnly] {
        let (raster, range) = series::resize_misses(mode);
        println!(
            "  {:<32}  {raster:>14}  {range:>12}  {:>14}",
            mode.word(),
            series::resize_wrong_cells(mode)
        );
    }
    println!(
        "  the narrower key folds FEWER times and is wrong, which is why the detector is the \
         surface and never the counter\n"
    );

    println!("report  colour, and the second axis a paint alone does not have:");
    let sixteen = Build::correct().tier(ColorDepth::Ansi16);
    let theme = sixteen.theme();
    println!(
        "  at sixteen colours, Danger/Warn differ on the wire? {}; Warn/Ok? {}",
        theme.roles_differ_on_wire(Role::Danger, Role::Warn),
        theme.roles_differ_on_wire(Role::Warn, Role::Ok),
    );
    let named = series::render(sixteen, (W, H), COMPARED_AT);
    let roled = series::render(sixteen.both(|o| o.role_series = true), (W, H), COMPARED_AT);
    let diff = series::split_diff(&named, &roled, series::whole((W, H)));
    println!(
        "  role-derived series against `Theme::custom`: {} cells by style, {} by cluster — the \
         signature of a colour-only distinction dying",
        diff.style_only(),
        diff.cluster,
    );
    let with = series::render(sixteen, (W, H), COMPARED_AT);
    let without = series::render(
        sixteen.both(|o| o.threshold_glyph = false),
        (W, H),
        COMPARED_AT,
    );
    let diff = series::split_diff(&with, &without, series::whole((W, H)));
    println!(
        "  a threshold carried by a paint alone against paint + HLine: {} cells over {} row\n",
        diff.cluster, diff.rows
    );
}

/// 9. What does not reproduce, and why.
fn what_does_not_reproduce() {
    println!("report  what does not reproduce, and why:");
    let verbs: Vec<u64> = series::across_volumes(Build::correct(), (W, H))
        .iter()
        .map(|(_, s)| s.verbs)
        .collect();
    println!(
        "  verbs are {verbs:?} against §13's 2 551 / 3 772 / 2 472 — three more at every volume, \
         and the same rise and fall. The three are the chrome going through `frame::block_into` \
         rather than a prototype's own border loop, and they are the same three at all three \
         volumes, so every difference the claim is about is identical"
    );
    println!(
        "  §13's 204 us frame is a prototype's on a different runtime. What reproduces exactly is \
         everything that is a count: 21 872 / 1 136 writes, 2 regions, 2 400 B, 0 / 882, 7 276 \
         over 80 rows, 5 546 / 5 500, 3 431 / 78, 2 953 / 2 544, 25 shared cells, 464 of 175 712"
    );
    println!(
        "  §13's `0 2.5 5 7.5 10 against 0 5 10` needs a 1 / 2 / 2.5 / 5 ladder and this one is \
         1 / 2 / 5, so `nice_step(10, 5)` is 2.0. The finding survives: the loop oscillates on 464 \
         pairs because a narrower window drops the older, larger samples, which is §13's own \
         second sentence"
    );
    println!(
        "  the pane rasters are 40 040 B at 1 000 points and 39 732 B at 100 000 and 1 000 000. \
         The raster's size is the *plotting* rectangle, and the gutter is as wide as the widest \
         tick label — which is data. `the size is the rectangle` holds exactly; `the rectangle is \
         independent of the data` does not"
    );
    println!(
        "  §13 says `roles_differ_on_wire(Danger, Warn)` and `(Warn, Ok)` are BOTH false at \
         sixteen colours. On the shipped Catppuccin Mocha palette the first is true and only the \
         second is false. The palette was deliberately not swapped to make the old number, and the \
         obligation is unchanged: one collapsing pair is enough to owe a second axis, and the 215 \
         cells over 1 row are that axis being drawn"
    );
    println!(
        "  the culled arm is 1.37x faster here against §13's 1.39x, and the memo ratio is over two \
         million against §13's 1 849 000. Both are timings and neither is a gate: the hit is at \
         this clock's resolution floor, so the ratio is arithmetic over a number that cannot be \
         measured more finely from here. What the two share is the only thing that matters — the \
         defect is faster, and the memo is the seam"
    );
}

/// `a / b`, as a ratio a reader can read.
fn ratio(a: Duration, b: Duration) -> f64 {
    b.as_secs_f64() / a.as_secs_f64().max(f64::MIN_POSITIVE)
}
