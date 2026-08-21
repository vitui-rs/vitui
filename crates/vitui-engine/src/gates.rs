//! The register's wired entries, driven over the twelve scenes.
//!
//! Six of spec §14's twenty-seven properties are checked here; the rest are in `tests/alloc.rs`
//! (#4), in the paired doctests (#19) and in `examples/budget.rs` (#20, #23, #24, #25), or pinned
//! red in [`crate::register`] with the implementation ticket that inverts them.
//!
//! **Every gate runs over every scene**, never over one scene it likes. A gate that picks its own
//! scenes tests the scenes, which is the failure [`crate::scenes`] exists to describe.
//!
//! One gate here is **not** a register entry and does not run over the scenes: ticket 11's pairing
//! invariant over the composited frame. It is here rather than beside the compositor because it
//! needs the same reference compositor and the same round trip the six above do, and it has a scene
//! of its own because **none of §14's twelve produces the case** — which is exactly why it was
//! filed rather than found.
//!
//! # The birth frame is presented and then left behind
//!
//! Adding a layer damages its whole rectangle, because nothing beneath it has been asked to repaint
//! what it now covers. That is the design and not something to gate against, so every gate below
//! builds, presents once, and starts measuring at the frame after.

use crate::caps::Overrides;
use crate::damage::Run;
use crate::engine::Screen;
use crate::geom::Rect;
use crate::mix::Mix;
use crate::reference;
use crate::register::State;
use crate::restyle::Restyle;
use crate::scenes::{H, Scene, W, scenes, table_two_ways, virtualised_tree};
use crate::serial::Filter;
use crate::style::{Color, Style};
use crate::testing::{Harness, assert_pairing_holds, bisecting_cjk};

/// How many steady-state frames each scene is driven for.
///
/// Three, and the number is a trade rather than a preference: the reference compositor visits
/// 24 000 cells against every layer of the stack twice a frame, which is what makes it an
/// independent oracle and also what makes it too slow to run a hundred frames of in a debug build.
/// Every property here is per-frame, so a fourth frame would repeat the third.
const FRAMES: u32 = 3;

/// A harness with the scene's layers built and its birth frame already presented.
///
/// The harness is [`crate::testing::Harness`], the same one [`crate::roundtrip`] drives, so every
/// `present` below closes the round trip whether or not the gate that called it is about the round
/// trip. That is deliberate: the first draft of this file wrote its own loop and quietly checked
/// one thing fewer — the replayed screen but not the mirror — which is exactly how a normative
/// scene list ends up with weaker coverage than the ad-hoc tests beside it.
fn staged(scene: &mut dyn Scene) -> Harness {
    staged_with(scene, Filter::default())
}

/// The same, serialising under one of the filter configurations that lost.
///
/// The filter is set **before** the birth frame, which costs nothing and says something: every row of
/// a fresh mirror is unknown, so the birth frame is the same bytes under all four configurations, and
/// a harness that had to be told afterwards would have a window where it was measuring two.
fn staged_with(scene: &mut dyn Scene, filter: Filter) -> Harness {
    // The scene says what it needs pinned about the terminal, and both drivers ask. Since impl 17
    // that is truecolor for all twelve — narrowing at truecolor is the identity, and a byte count on
    // a screen whose depth nobody named is a byte count about the depth — plus OSC 8 for the
    // twelfth, whose operator layer is skipped outright at `ColorDepth::None`.
    let mut h = Harness::with_overrides(W, H, scene.overrides())
        .labelled(scene.name())
        .with_filter(filter);
    scene.build(&mut h.screen);
    h.present();
    h
}

/// What [`FRAMES`] steady frames of one scene cost on the wire under one filter configuration.
///
/// Through [`Harness`], so every configuration of the instrument is also driven through the round
/// trip: a variant that skipped a cell it may not have skipped fails as a wrong screen here rather
/// than as a suspiciously small number in a report.
fn steady_bytes(scene: &mut dyn Scene, filter: Filter) -> usize {
    let mut h = staged_with(scene, filter).without_scroll_region();
    let before = h.bytes_written();
    for t in 1..=FRAMES {
        scene.step(&mut h.screen, t);
        h.present();
    }
    h.bytes_written() - before
}

/// Every scene whose mechanism exists today.
fn wired() -> Vec<Box<dyn Scene>> {
    scenes()
        .into_iter()
        .filter(|s| matches!(s.status(), State::Wired { .. }))
        .collect()
}

/// Whether `(x, y)` lies inside one of the reported runs.
fn covered(runs: &[Run], x: u16, y: u16) -> bool {
    runs.iter().any(|r| r.y == y && r.lo <= x && x <= r.hi)
}

// ---------------------------------------------------------------------------------------------
// #1 — no damage structure under-reports.  Generated from the reference compositor.
// ---------------------------------------------------------------------------------------------

/// Gate #1, and the one property on the register that may not be hand-written.
///
/// A hand-written expectation about damage is written by whoever wrote the damage and agrees with
/// it for the same reason. So the expectation is **generated**: composite the whole stack the slow
/// obvious way before a frame's verbs and again after them, and every cell that differs must lie
/// inside a run the damage structure reported. Nothing in that sentence mentions how damage is
/// marked, which is why it survives `RowBits` being replaced.
///
/// The second half is the picture itself: the frame the damage-tracked path produced must equal the
/// reference everywhere, **including outside every damaged run**. That is what says
/// damage-tracked compositing and full compositing are the same function rather than two functions
/// that agree where anyone looked.
#[test]
fn no_damage_structure_under_reports() {
    for mut scene in wired() {
        let name = scene.name();
        let mut h = staged(&mut *scene);
        for t in 1..=FRAMES {
            let before = h.screen.reference();
            let written = scene.step(&mut h.screen, t) as usize;
            let after = h.screen.reference();
            h.present();

            let changed = reference::differences(&before, &after);
            for &(x, y) in &changed {
                assert!(
                    covered(h.screen.runs(), x, y),
                    "{name}, frame {t}: ({x}, {y}) changed and no run reported it"
                );
            }

            // Gate #3 divides by a number the scene declares about itself, and this is the one
            // independent bound on that declaration: a scene cannot change more cells of the
            // screen than its verbs wrote. It does not pin the number — several scenes rewrite
            // cells whose value does not change, which is exactly what the equality filter exists
            // to strip — but it does catch a count that was written down too small, which is the
            // direction that would make an overdraw of 1.00x meaningless.
            assert!(
                changed.len() <= written,
                "{name}, frame {t}: {} cells changed but the scene declares only {written} \
                 written",
                changed.len()
            );

            let frame = h.screen.frame();
            for y in 0..H {
                for x in 0..W {
                    assert_eq!(
                        frame.row(y)[x as usize],
                        after.row(y)[x as usize],
                        "{name}, frame {t}: the damage-tracked frame and the reference \
                         compositor disagree at ({x}, {y})"
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// #2 — runs arrive in serializer order.
// ---------------------------------------------------------------------------------------------

/// Gate #2. The order runs arrive in **is** the order bytes are written in, so a scan that emitted
/// a row twice, or a row out of order, would cost a cursor move per violation on a wire whose
/// force-flush limits go as low as 150 ms.
///
/// Adjacency is part of it: two runs that touch are one run, and a scan that emitted `0..=3` and
/// `4..=7` separately would pass an ordering check while paying for a move between them.
#[test]
fn runs_arrive_in_serializer_order() {
    for mut scene in wired() {
        let name = scene.name();
        let mut h = staged(&mut *scene);
        for t in 1..=FRAMES {
            scene.step(&mut h.screen, t);
            h.present();
            let runs = h.screen.runs();
            for r in runs {
                assert!(r.lo <= r.hi, "{name}, frame {t}: an inverted run {r:?}");
                assert!(
                    r.hi < W,
                    "{name}, frame {t}: a run past the right edge, {r:?}"
                );
                assert!(r.y < H, "{name}, frame {t}: a run past the bottom, {r:?}");
            }
            for pair in runs.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                assert!(
                    (a.y, a.lo) < (b.y, b.lo),
                    "{name}, frame {t}: {b:?} arrived after {a:?}"
                );
                if a.y == b.y {
                    assert!(
                        b.lo > a.hi + 1,
                        "{name}, frame {t}: {a:?} and {b:?} touch and should be one run"
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// #3 — overdraw is 1.00x on every scene.
// ---------------------------------------------------------------------------------------------

/// Gate #3, per scene and never summed.
///
/// Overdraw is the cells the damage structure reported divided by the distinct cells the verbs
/// wrote, and for a per-row bitset it is exactly 1.00x on every scene because the bitset marks
/// exactly what the verbs marked. That is a property of the **mechanism** rather than of the data,
/// which is what spec §14 requires before a number may be an equality rather than a relation — the
/// same gate against per-row spans reports 2.53x on three dialogs and 37.07x on the sparse chart,
/// and those two numbers are about the scenes.
///
/// Kept per scene, because the 27x scroll-detector regression the map found was visible only that
/// way: summed over twelve scenes it would have been a rounding error.
#[test]
fn overdraw_is_one_times_on_every_scene() {
    for mut scene in wired() {
        let name = scene.name();
        let mut h = staged(&mut *scene);
        for t in 1..=FRAMES {
            let written = scene.step(&mut h.screen, t) as usize;
            let reported = h.screen.layers().reported_cells();
            h.present();
            assert_eq!(
                reported,
                written,
                "{name}, frame {t}: overdraw {:.2}x — {reported} cells reported for {written} \
                 written",
                reported as f64 / written as f64
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// #10 — a packed cell is byte-identical to the surface cell.
// ---------------------------------------------------------------------------------------------

/// Gate #10, whose violation is invisible in every ordinary test.
///
/// The screen stays correct when packing quietly widens or reorders a cell — the serializer writes
/// what it was given and the round trip closes. What changes is the wire, by **284x** on the
/// measurement that found it. So the equality is asserted against the surface directly, in the run
/// order the packet claims, and the count is asserted too: a packet carrying more cells than its
/// runs describe would otherwise pass a cell-for-cell walk of its own prefix.
#[test]
fn a_packed_cell_is_byte_identical_to_the_surface_cell() {
    for mut scene in wired() {
        let name = scene.name();
        let mut h = staged(&mut *scene);
        for t in 1..=FRAMES {
            scene.step(&mut h.screen, t);
            h.present();

            let packet = h.screen.packet();
            let frame = h.screen.frame();
            assert_eq!(
                packet.size(),
                h.screen.size(),
                "{name}, frame {t}: the packet carries the wrong size"
            );
            let cells = packet.cells();
            let mut at = 0;
            for r in packet.runs() {
                for x in r.lo..=r.hi {
                    assert_eq!(
                        cells[at],
                        frame.row(r.y)[x as usize],
                        "{name}, frame {t}: the packed cell at ({x}, {}) is not the surface cell",
                        r.y
                    );
                    at += 1;
                }
            }
            assert_eq!(
                at,
                cells.len(),
                "{name}, frame {t}: the packet carries cells no run describes"
            );
            assert_eq!(
                packet.runs(),
                h.screen.runs(),
                "{name}, frame {t}: the packet's runs are not the frame's"
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// #12 — round trip: the replayed screen equals the composited frame.
// ---------------------------------------------------------------------------------------------

/// Gate #12, over all eleven runnable scenes.
///
/// [`crate::roundtrip`] drives the shapes ticket 03 could express one at a time; this drives the
/// normative list. It stores nothing: composite, serialise, replay the bytes through the terminal
/// model, assert the replayed screen equals the frame. A golden byte string would have pinned the
/// encoding, and the encoding is exactly the part ticket 15 has still to change.
#[test]
fn the_round_trip_closes_on_every_scene() {
    for mut scene in wired() {
        let mut h = staged(&mut *scene);
        for t in 1..=FRAMES {
            scene.step(&mut h.screen, t);
            // `Harness::present` *is* the round trip: it replays every fresh byte through the
            // terminal model, refuses a sequence the model does not parse, and asserts the replayed
            // screen and the mirror both equal the composited frame. Every gate in this file calls
            // it, so this test is the one that says so rather than the only place it happens.
            assert!(h.present().submitted, "frame {t} said nothing");
        }
    }
}

// ---------------------------------------------------------------------------------------------
// #21 — wire bytes per scene.
// ---------------------------------------------------------------------------------------------

/// What three steady frames of each scene are allowed to cost on the wire.
///
/// **An upper bound, and the bound is the measurement.** A byte count has no noise — it is a byte
/// count on any machine, which is the whole argument for the register's shape — so there is no
/// headroom to add and nothing to be flaky about. An improvement passes; a regression fails; and
/// the numbers were meant to fall, because tickets 13, 14 and 15 existed to make every one of them
/// smaller. **All three have now run, and §8 is on the wire in full.** The one that gates the
/// *filter's* own arithmetic is [`the_equality_filter_reproduces_spec_8s_table`], which is a relation
/// rather than these counts, and it is measured with the scroll pre-pass **off** so that it stays a
/// statement about the filter.
///
/// Moving a number up is allowed and costs a commit that does three things: states the new number,
/// states the measurement it came from, and replaces the provenance line below. What may **not**
/// move without a new map decision is a budget figure, and none of these is one.
///
/// Provenance: measured by **impl 15** on 2026-08-21, Apple M1 Max, rustc 1.97.1, over
/// [`FRAMES`] steady frames after the birth frame, at 300x80, with the full §8 encoding set —
/// `shortest`, the differential SGR, SGR 58/59, OSC 8 — the equality filter with its gap merge, **and
/// the scroll region.** Nothing about §8 is deferred now. It replaces impl 14's line, which was taken
/// without the scroll region.
///
/// **Four of the twelve fell again, by between 2.2x and 2.6x over the three frames, and eight did not
/// move at all** — the pre-pass emits nothing it has not verified, so a screen with no shift in it
/// pays the probe and nothing else.
///
/// | | three frames | the last frame, which is the only steady scroll |
/// |---|---|---|
/// | `table-as-list-and-bar-chart` | 6 118 → **2 778** | 1 695 → **20**, 84.8x |
/// | `virtualised-tree` | 2 005 → **761** | 643 → **21**, 30.6x |
/// | `scrolling-list-rows-cleared` | 1 925 → **679** | 643 → **20**, 32.1x |
/// | `scrolling-list-label-only` | 1 925 → **679** | 643 → **20**, 32.1x |
///
/// **The three-frame column understates it by more than an order of magnitude, and that is the whole
/// of §8's two-figures problem.** `step(1)` paints eighty labels onto the blank screen the scene's
/// `build` left behind, so the first of the three frames is a second birth frame with no scroll in it
/// and it is most of what the row costs. The per-frame figure is the one the optimisation is about;
/// the three-frame figure is the one this budget gates, because every other row here is counted the
/// same way. Both are printed by [`which_of_spec_14s_twelve_the_scroll_region_reaches`].
///
/// Impl 14's split still holds for the rest of the list. The six that the filter moved were
/// `scrolling-list-rows-cleared` (37.7x), `virtualised-tree` (5.0x), `table-as-list-and-bar-chart`
/// (3.1x), `scrolling-list-label-only` (2.8x), `progress-bar-one-percent` (2.4x) and `caret-blink`
/// (1.4x); the other six are **1.00x under the filter and 1.00x under the scroll region too**, and
/// every one of them is a scene in which every damaged cell genuinely changes every frame.
/// `full-screen-change` and `every-cell-a-distinct-style` say so in their names. `three-dialogs-apart`
/// and `twenty-popups-with-shadows` write a frame counter into their labels *and* cycle a foreground
/// colour, so no cell survives a frame unchanged. `sparse-chart-400-points` moves all four hundred
/// points every frame.
///
/// **And `hyperlinked-page-under-an-animating-operator` is 876 624 bytes still, which closes what impl
/// 13 wrote about it and impl 14 half-answered.** Impl 13's Progress said *impl 14 is where it comes
/// back*; impl 14 found that the text does not change and the **style word does**, because the scene's
/// own subject is a *fading* operator, so every cell of every frame resolves to a different colour.
/// Impl 14 then wrote that *what would actually reduce it is the scroll region or nothing*, and the
/// answer is **nothing**: a fade changes every row, so no row lands where another one was and the
/// probe finds nothing to verify. The row stays `REPORTED_NOT_GATED` in `examples/budget.rs` at the
/// ratio impl 13 measured.
///
/// Two entries are the reason the gate exists at all. **`every-cell-a-distinct-style` is 1.4 MB for
/// three frames** — the adversarial page, which is what set the synchronised-output time limit — and
/// **`sparse-chart-400-points` is 8 009 bytes for 1 200 cells**, which is the ratio a serializer
/// walking the grid instead of the runs would blow up by 284x without changing a pixel.
const WIRE_BUDGET: [(&str, usize); 12] = [
    ("caret-blink", 30),
    ("scrolling-list-label-only", 679),
    ("scrolling-list-rows-cleared", 679),
    ("twenty-popups-with-shadows", 30_354),
    ("three-dialogs-apart", 6_564),
    ("sparse-chart-400-points", 8_009),
    ("progress-bar-one-percent", 138),
    ("virtualised-tree", 761),
    ("table-as-list-and-bar-chart", 2_778),
    ("full-screen-change", 72_519),
    ("every-cell-a-distinct-style", 1_367_431),
    ("hyperlinked-page-under-an-animating-operator", 876_624),
];

/// Gate #21, and the reason it is per scene rather than a total.
///
/// A byte count is a byte count on any machine, which is what makes it the register's favourite
/// shape. Summed across scenes it would hide exactly the regression it exists to catch: the sparse
/// chart is 400 cells against the full screen's 24 000, so a 284x regression on the chart is under
/// 5% of the total.
#[test]
fn wire_bytes_per_scene() {
    // The table covers exactly the wired scenes, checked as a set before a byte is measured. A
    // lookup that panicked on a miss would still let a **removed** scene leave a stale bound
    // behind, and a stale bound is a gate that passes because nothing runs it.
    let mut budgeted: Vec<&str> = WIRE_BUDGET.iter().map(|(n, _)| *n).collect();
    let mut running: Vec<&str> = wired().iter().map(|s| s.name()).collect();
    budgeted.sort_unstable();
    running.sort_unstable();
    assert_eq!(
        budgeted, running,
        "the wire budgets and the scenes that run are not the same set"
    );

    for mut scene in wired() {
        let name = scene.name();
        let mut h = staged(&mut *scene);
        let before = h.bytes_written();
        for t in 1..=FRAMES {
            scene.step(&mut h.screen, t);
            h.present();
        }
        let bytes = h.bytes_written() - before;
        let (_, budget) = WIRE_BUDGET
            .iter()
            .find(|(n, _)| *n == name)
            .expect("checked against the scene list above");
        println!("  {name:<44} {bytes:>9} bytes  (bound {budget})");
        assert!(
            bytes <= *budget,
            "{name}: {bytes} bytes over {FRAMES} frames, bound is {budget}. \
             The bound is impl 14's own measurement, so this is a regression rather than drift."
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Impl 15 — the scroll region, verified before a byte is emitted.
// ---------------------------------------------------------------------------------------------

/// **Gate, and the finding beside it.** §8's two arms of the same list, at the numbers this round
/// trip actually produces.
///
/// §8's table is *label only* refused at 4 305 bytes against *rows cleared, as a widget must* at 180,
/// and calls the difference a performance contract on the component library. **Both halves of that
/// come out differently here, and only one of them is about the mechanism.**
///
/// The ratio is real and larger than §8's, once it is asked of the right frame. §8's *180* and its
/// *1 726 → 60* are the same claim counted over three frames and over one, and neither is the number
/// [`WIRE_BUDGET`] holds: the first steady frame of these scenes is a **second birth frame** — the
/// scene's `build` adds a layer and draws nothing, so `step(1)` paints eighty labels onto a blank
/// screen and no scroll exists yet. So this gate reports the three-frame total that the budget gates
/// and asserts the ratio on **the last frame**, which is the only one of the three that is a steady
/// scroll. See the ticket's own resolution of §8's two figures, filed as a finding against §8.
///
/// What is **not** reproducible on §14's list is the refusal. `scrolling-list-label-only` writes a
/// twenty-column label onto a row that is *blank past it*, so what the frame wants on row `y`
/// genuinely is what the mirror holds on row `y + 1` — the scroll is legitimate and is taken. §8's
/// label-only arm was a list whose rows had content past the label, and that content belongs to the
/// screen row rather than to the item. The two arms differ here in **damage** and not in what the
/// pre-pass may do with them, and both scroll.
///
/// That is filed rather than fixed, for §14's own reason — adding a thirteenth scene to make an arm
/// refuse is measuring the fixture — and the conclusion is executed one level down instead:
/// `crate::roundtrip::a_list_whose_tail_does_not_scroll_with_its_labels_is_refused` is §8's label-only
/// arm as §8 wrote it, and it is refused.
#[test]
fn the_scroll_region_over_spec_8s_two_arms() {
    /// What the pre-pass has to be worth on the one frame of the three that is a steady scroll.
    /// §8's own ratio is 5 160 → 180 counted over three frames, which is 28.7x.
    const WORTH: usize = 20;

    /// Three steady frames of one scene, and the last of them on its own.
    fn arms(scene: &mut dyn Scene, scroll: bool) -> (usize, usize, (usize, usize)) {
        let mut h = staged(scene);
        if !scroll {
            h = h.without_scroll_region();
        }
        let before = h.bytes_written();
        let mut last_at = before;
        for t in 1..=FRAMES {
            last_at = h.bytes_written();
            scene.step(&mut h.screen, t);
            h.present();
        }
        (
            h.bytes_written() - before,
            h.bytes_written() - last_at,
            h.scrolls(),
        )
    }

    println!(
        "\n  {:<32} {:>18} {:>18}   win",
        "scene", "filtered (3f, last)", "+ scroll (3f, last)"
    );
    let mut arm_totals = Vec::new();
    for name in ["scrolling-list-rows-cleared", "scrolling-list-label-only"] {
        let pick = || {
            wired()
                .into_iter()
                .find(|s| s.name() == name)
                .expect("§14's twelve are the wired list")
        };
        let (flat, flat_last, _) = arms(&mut *pick(), false);
        let (total, last, (scrolls, verifies)) = arms(&mut *pick(), true);
        println!(
            "  {name:<32} {:>18} {:>18}   {:.1}x on the last frame  ({scrolls} scrolls, \
             {verifies} verified)",
            format!("{flat}, {flat_last}"),
            format!("{total}, {last}"),
            flat_last as f64 / last.max(1) as f64
        );
        // FRAMES - 1: `step(1)` paints a blank screen and there is no scroll to find in it.
        assert_eq!(
            scrolls,
            FRAMES as usize - 1,
            "{name}: every steady frame of a one-row scroll is a scroll, and the frame that paints \
             the list onto a blank screen is not one"
        );
        assert_eq!(
            verifies,
            FRAMES as usize - 1,
            "{name}: one candidate a frame and never more — §8's 27x regression was verifying every \
             candidate that matched the probe"
        );
        assert!(
            last * WORTH <= flat_last,
            "{name}: the scroll region took the last frame from {flat_last} bytes to {last}, under \
             {WORTH}x. What it buys is a band of rows moved instead of rewritten."
        );
        arm_totals.push((flat, total));
    }

    // **The finding, as a gate.** §8 has the two arms 24x apart after the filter and one of them
    // refused by the pre-pass. Here they are *equal* on both, and that is not a defect: the two arms
    // of a list that is blank past its label produce the same screen and leave the same mirror, so
    // nothing distinguishes them for either mechanism — only for the damage structure, where the
    // cleared arm is 13.7x more span bytes (72 504 against 5 304, printed by
    // `the_equality_filter_reproduces_spec_8s_table`) and 15x more damaged cells. Filed against §8,
    // and what the contract on `Surface` says had to change with it.
    assert_eq!(
        arm_totals[0], arm_totals[1],
        "the two arms differ in damage and in nothing the filter or the pre-pass can see. If they \
         have come apart, one of the two scenes has changed what it draws rather than only how."
    );
}

/// **Report.** Which of §14's twelve the scroll region reaches, and what it is worth on the one frame
/// of the three that is a steady scroll.
///
/// **Four of the twelve, and two of them are a surprise.** The two list arms are what §8 named. The
/// **virtualised tree** and the **table as a list and a bar chart** both scroll too, and neither is
/// on §8's list of what this optimisation is for — a tree that scrolls its rows and a table that
/// scrolls its body are the same shape as a log pane, and the pre-pass finds them without being told.
/// That is the result worth reading here: the mechanism generalises past the scene it was cut for.
///
/// The other eight take nothing, and every one of them is a screen with no shift in it: a caret does
/// not move a row, a progress bar does not, twenty popups do not, and a full-screen churn has no row
/// that lands where another one was. **Not one of them is made worse on the wire** — the pre-pass
/// emits nothing it has not verified — and in time they pay the probe and nothing else: measured
/// against the same run without the pre-pass, `full-screen-change` moves 393.8 µs to 395.4 µs and
/// `twenty-popups-with-shadows` 321.1 to 322.2. What the pre-pass does cost, it costs on the four
/// scenes it *finds* something on, and the figures for those are in
/// `examples/budget.rs`'s `REPORTED_NOT_GATED`.
///
/// A report rather than a gate, for [`WIRE_BUDGET`]'s own reason inverted: the counts *are* gated,
/// there, per scene. What is not gated is which scenes they belong to, because that is a fact about
/// §14's list rather than about the pre-pass.
#[test]
fn which_of_spec_14s_twelve_the_scroll_region_reaches() {
    println!(
        "\n  {:<44} {:>10} {:>10} {:>7}   scrolls",
        "scene", "filtered", "+ scroll", "win"
    );
    for name in wired().iter().map(|s| s.name()) {
        let pick = || {
            wired()
                .into_iter()
                .find(|s| s.name() == name)
                .expect("the name came from this list")
        };
        let last = |scroll: bool| {
            let mut scene = pick();
            let mut h = staged(&mut *scene);
            if !scroll {
                h = h.without_scroll_region();
            }
            let mut at = h.bytes_written();
            for t in 1..=FRAMES {
                at = h.bytes_written();
                scene.step(&mut h.screen, t);
                h.present();
            }
            (h.bytes_written() - at, h.scrolls().0)
        };
        let (flat, _) = last(false);
        let (scrolled, scrolls) = last(true);
        println!(
            "  {name:<44} {flat:>10} {scrolled:>10} {:>6.1}x   {scrolls}",
            flat as f64 / scrolled.max(1) as f64
        );
    }
}

/// **Gate, and it reproduces §8's 27x rather than quoting it.** A screen whose rows repeat matches the
/// probe many times over, and verifying each of them turned a 38 µs frame into 1.03 ms.
///
/// The fixture is two full-width row patterns that swap places every frame, with **one row pinned by a
/// layer the frame never redraws**, so that the shift the band's edge row attests to is not the shift
/// the band actually took. The probe matches at every odd distance in both directions — some eighty
/// candidates — and every one of them is refused: the pinned row is inside the rows a short scroll
/// *moves*, where obligation 1 refuses it after walking half the band, and inside the rows a long one
/// *exposes*, where obligation 2 refuses it because an undamaged row holding content is not one the
/// terminal may erase.
///
/// **Pinned by an undamaged row, and that is what makes the two arms comparable.** The first draft
/// pinned it by redrawing it, and the rejected arm then *found a real scroll* at a distance of 41 —
/// legitimately, because at that distance the pinned row falls among the exposed ones and every
/// exposed row was being repainted. §8 says as much in a clause that is easy to read past: *taking the
/// first match forfeits a scroll that could in principle have been found.* It also emits, which moves
/// the mirror, which makes the two arms diverge in state and the count ratio between them meaningless.
/// So the fixture forfeits nothing and the arms differ only in **work**.
///
/// **Two arms, and the second one is the version that lost.** `Harness::verifying_every_match` is §8's
/// rejected pre-pass, reachable from a test and from nowhere else — the same argument [`Filter`] makes
/// for the three gap rules that lost. Without it this gate could only assert that the shipping
/// version is *fast*, and the first draft did exactly that and was **vacuous**: this fixture's frame is
/// a full-screen change, so the pre-pass is 15 µs of 400 and any headroom that survives a shared
/// runner's noise also survives a forty-candidate loop. §8's 27x was measured on a 38 µs frame. A
/// cliff you cannot construct is a cliff you cannot gate.
///
/// So the property is gated as a **count ratio between the two arms**, which is exact, and the frame
/// time is reported beside it with both arms' numbers. That is this backlog's own rule — *a gate is a
/// count, a ratio, an equality or a compile outcome; a timing is a report* — applied to the one
/// acceptance line that asked for a timing.
///
/// **Measured: 10 candidates against 800, and 410 µs against 967 µs a frame in release.** The ratio is
/// 2.4x rather than §8's 27x and the reason is the denominator, not the mechanism: this fixture
/// rewrites its whole screen every frame, so the frame the pre-pass sits inside is 410 µs where §8's
/// was 38. The numerator is the number worth reading — §8 measured the rejected version at **1.03 ms**
/// and this measures it at **967 µs**, both of which are a full-screen frame budget spent on candidates
/// that were all going to be refused.
///
/// It is not one of §14's twelve and does not belong on that list: it discriminates nothing about
/// damage, compositing or the wire, and exists only because this mechanism has a cliff. Prior art is
/// impl 11's pairing invariant, two sections up, for the same reason.
#[test]
fn a_repeating_rows_screen_verifies_one_candidate_a_frame() {
    /// How many more candidates §8's rejected version must be caught verifying before this gate is
    /// satisfied. It verifies one per *matching* distance and the fixture matches at every odd one,
    /// so the real figure is around forty; the bound is where a version that had quietly gone back to
    /// verifying a handful would still fail.
    const CLIFF: usize = 10;
    /// Frames timed per arm, after the warm-up.
    const SAMPLES: u32 = 8;
    /// The row that does not join in, which is what makes every matching candidate a wrong one.
    const PINNED: u16 = H / 2;

    let patterns: [String; 2] = [
        std::iter::repeat_n('-', W as usize).collect(),
        std::iter::repeat_n('=', W as usize).collect(),
    ];
    let arm = |every: bool| {
        let h = Harness::with_overrides(W, H, Overrides::default()).labelled("repeating-rows");
        let mut h = if every { h.verifying_every_match() } else { h };
        let id = h
            .screen
            .layers()
            .add_content(0, Rect::new(0, 0, W, H), true);
        // The pinned row, on a layer of its own so that the frame below can leave it alone. Painted
        // once: it is damaged on the birth frame and never again, which is what puts it beyond both
        // obligations rather than only one.
        let pin = h
            .screen
            .layers()
            .add_content(1, Rect::new(0, i32::from(PINNED), W, 1), true);
        h.screen
            .layers()
            .view(pin)
            .expect("the layer is still there")
            .text(0, 0, &patterns[0], Style::new());
        let draw = |h: &mut Harness, t: u32| {
            let mut v = h
                .screen
                .layers()
                .view(id)
                .expect("the layer is still there");
            for y in (0..H).filter(|y| *y != PINNED) {
                v.text(
                    0,
                    i32::from(y),
                    &patterns[(u32::from(y) + t) as usize % 2],
                    Style::new(),
                );
            }
        };
        draw(&mut h, 0);
        h.present();
        // Warm: the first steady frame is the one that touches every page of the mirror.
        for t in 1..=2 {
            draw(&mut h, t);
            h.present();
        }
        let at = std::time::Instant::now();
        for t in 3..3 + SAMPLES {
            draw(&mut h, t);
            // `Screen::present` rather than the harness's: the round trip's two full-screen
            // comparisons are an order of magnitude more work than the frame they check, and this
            // arm is a stopwatch. The round trip over the same fixture is the `present` calls above.
            h.screen.present();
        }
        let us = at.elapsed().as_secs_f64() * 1e6 / f64::from(SAMPLES);
        (us, h.scrolls())
    };

    let (ours, (scrolls, verifies)) = arm(false);
    let (theirs, (also_none, every)) = arm(true);
    // One candidate a frame, on every frame after the birth one — a fresh mirror knows nothing, so
    // there is no candidate in the first frame at all.
    let frames = (SAMPLES + 2) as usize;
    println!(
        "\n  a repeating-rows screen, {SAMPLES} frames an arm:\n  \
         one candidate: {verifies} verified over {} frames, {ours:.1} us a frame\n  \
         every match  (§8's rejected version): {every} verified, {theirs:.1} us a frame, \
         {:.2}x\n  Report on the times; the count ratio is the gate.",
        SAMPLES + 3,
        theirs / ours,
    );
    assert_eq!(
        scrolls, 0,
        "the pinned row means no shift is the shift, so nothing may go out"
    );
    assert_eq!(
        also_none, 0,
        "and the rejected version does not find one either — it only pays more to say so, which is \
         what makes this a pure regression rather than a trade"
    );
    assert_eq!(
        verifies, frames,
        "one candidate a frame. This is the property: the probe returns an `Option`, so there is one \
         candidate to verify or none"
    );
    assert!(
        every >= verifies * CLIFF,
        "the arm that verifies every match got through {every} candidates against {verifies}, under \
         {CLIFF}x. That arm is §8's rejected version, and if it is no longer expensive then this \
         fixture has stopped being a repeating-rows screen and the gate is measuring nothing."
    );
}

/// **Gate, absence — with the positive half that makes the absence mean something.** `DECSLRM` is not
/// queried and not used.
///
/// Mode 69 would lift `SU`'s lack of horizontal margins and let a pane scroll without owning the
/// columns beside it. It is deliberately not taken: it is not in tier-1's confirmed set, and the
/// verification the pre-pass rests on would turn an unsupported margin into **silent corruption**
/// rather than into a wasted escape — the terminal would apply `SU` to the whole width while the
/// pre-pass had proved something about a band of it. **Spec §15 is where the question lives**, and
/// nothing in the serializer depends on the answer.
///
/// **The first draft of this gate was two absences and nothing else, which is vacuous** in exactly the
/// shape this backlog has a section about: it was green with the pre-pass switched off, green over an
/// empty wire, and green with `emit_scroll` deleted. An absence is only worth asserting over a wire
/// that had the chance to carry the thing. So the positive half is here: the scrolls the serializer
/// counts and the `SU`/`SD` finals on the wire must agree per scene, and some scene must have put one
/// there — which also gates the counter every other test in this section reads against the bytes it
/// claims to be counting.
#[test]
fn a_scroll_reaches_the_wire_and_never_asks_for_horizontal_margins() {
    let mut total = 0usize;
    for mut scene in wired() {
        let name = scene.name();
        let mut h = staged(&mut *scene);
        for t in 1..=FRAMES {
            scene.step(&mut h.screen, t);
            h.present();
        }
        let wire = h.wire();
        assert!(
            !wire.windows(6).any(|w| w == b"\x1b[?69h"),
            "{name}: something asked for mode 69"
        );
        let finals: Vec<u8> = crate::serial::csi_finals(&wire).collect();
        assert!(
            !finals.contains(&b's'),
            "{name}: something set horizontal margins"
        );
        // The positive half, per scene: what the serializer says it did is what is on the wire.
        let (scrolls, _) = h.scrolls();
        assert_eq!(
            finals.iter().filter(|b| matches!(b, b'S' | b'T')).count(),
            scrolls,
            "{name}: the scroll count and the wire disagree"
        );
        total += scrolls;
    }
    assert!(
        total > 0,
        "no scene put a scroll on the wire at all, so the two absences above were asserted over a \
         wire that never had the chance to carry a margin"
    );
}

// ---------------------------------------------------------------------------------------------
// Impl 14 — what the equality filter is worth, and why it carries no threshold.
// ---------------------------------------------------------------------------------------------

/// **Gate, and the report beside it.** §8's own table, reproduced over the twelve scenes rather than
/// quoted.
///
/// Four columns and one relation. `span` is no filter at all, which is what impl 13 shipped;
/// `strict` compares and never merges a gap; `gap 6` merges any gap of at most six columns, which is
/// the fixed threshold §8 swept; `chosen` is the byte-priced rule that ships.
///
/// The relation is §8's claim about the rule: **it lands within one per cent of the better of the two
/// fixed thresholds on every scene.** That is the shape §14 asks of a number belonging to the data
/// rather than to the mechanism — a ratio, not an equality — and it is what makes this a gate rather
/// than a paragraph. The absolute numbers are the report beside it, and the per-scene bound they are
/// gated against is `WIRE_BUDGET`.
///
/// **Never summed.** The chart is 400 cells against the full screen's 24 000, so a total would hide
/// the very row it exists to show.
#[test]
fn the_equality_filter_reproduces_spec_8s_table() {
    /// How far over the better fixed threshold the byte-priced rule may land, in per cent.
    ///
    /// **§8 says one, and measured here the rule needs none of it**: `chosen` equals
    /// `min(strict, gap 6)` exactly on all twelve scenes, because none of them produces a gap the two
    /// rules disagree about. The one is kept rather than tightened to zero, because overshooting *is*
    /// possible and the direction is written at `SGR_FLOOR`: the floor underestimates the SGR a merged
    /// gap really costs, so a scene with expensive gaps can pay a few bytes for one — §8's own chart
    /// lands 42 bytes over `strict` for exactly that reason. Tightening this to an equality would make
    /// it a gate about §14's scene list rather than about the rule.
    const SLACK_PERCENT: usize = 1;
    const COLUMNS: [(&str, Filter); 4] = [
        ("span", Filter::Off),
        ("strict", Filter::Strict),
        ("gap 6", Filter::Cells(6)),
        ("chosen", Filter::Bytes),
    ];

    let names: Vec<&'static str> = wired().iter().map(|s| s.name()).collect();
    let mut rows = vec![[0usize; COLUMNS.len()]; names.len()];
    for (c, (_, filter)) in COLUMNS.iter().enumerate() {
        // A fresh scene per column, and this is not tidiness. `step` advances the scene's own
        // animation, so four columns measured off one object would be four different frames of it.
        for (i, mut scene) in wired().into_iter().enumerate() {
            rows[i][c] = steady_bytes(&mut *scene, *filter);
        }
    }

    println!(
        "\n  {:<44} {:>9} {:>9} {:>9} {:>9}   win",
        "scene", "span", "strict", "gap 6", "chosen"
    );
    for (i, name) in names.iter().enumerate() {
        let [span, strict, gap6, chosen] = rows[i];
        println!(
            "  {name:<44} {span:>9} {strict:>9} {gap6:>9} {chosen:>9}   {:.2}x",
            span as f64 / chosen.max(1) as f64
        );
    }

    for (i, name) in names.iter().enumerate() {
        let [_, strict, gap6, chosen] = rows[i];
        let better = strict.min(gap6);
        assert!(
            chosen * 100 <= better * (100 + SLACK_PERCENT),
            "{name}: the byte-priced rule spent {chosen} bytes where the better of the two fixed \
             thresholds spent {better}, which is over {SLACK_PERCENT}%. §8's claim is that pricing \
             a gap in bytes lands within that of whichever threshold happens to suit the scene — \
             the point being that no threshold suits them all."
        );
    }
}

/// **Report.** §8's threshold sweep, over §14's twelve — **and the finding is that they are flat.**
///
/// §8 swept a fixed cell-count threshold 0 → 24 and concluded there is no right value for one,
/// because two of its scenes wanted opposite ones. Run over the twelve as this repo writes them the
/// sweep is flat from four columns on, on every single scene, and the byte-priced rule matches every
/// row. So **the twelve do not discriminate on this axis**, and that is a fact about the scene list
/// rather than about the filter: the six that filter at all are label rows where a counter changes,
/// so their gaps are a handful of one-byte columns and every threshold above four merges the same set.
///
/// It is reported rather than fixed, for §14's own reason — a scene list is normative and adding a
/// thirteenth to make a sweep interesting would be measuring the fixture. The conclusion is executed
/// where the mechanism lives instead:
/// `crate::serial::tests::a_fixed_gap_threshold_is_in_the_wrong_unit_and_the_two_fillers_want_opposite_ones`
/// puts the same triangular-gap row through a one-byte and a three-byte cluster and gets §8's two
/// opposite optima out of it. What that test cannot say, and this one can, is that no scene on §14's
/// list is harmed by the rule.
///
/// **The prior art for reporting this rather than burying it is §14's own damage ticket, which had
/// three scenes that discriminated nothing and said so.**
#[test]
fn the_equality_filters_threshold_sweep_is_flat_on_every_scene() {
    const THRESHOLDS: [u16; 7] = [0, 4, 8, 12, 16, 20, 24];
    /// Where the sweep is expected to have stopped moving, in columns. Past this a threshold merges
    /// no gap any of the twelve produces.
    const FLAT_FROM: u16 = 4;

    let names: Vec<&'static str> = wired().iter().map(|s| s.name()).collect();
    let mut rows = vec![Vec::new(); names.len()];
    for n in THRESHOLDS {
        // A fresh scene per point, for the reason the four-column table states: `step` advances the
        // scene's own animation.
        for (i, mut scene) in wired().into_iter().enumerate() {
            rows[i].push(steady_bytes(&mut *scene, Filter::Cells(n)));
        }
    }

    println!("\n  a fixed gap threshold, swept over §14's twelve, columns {THRESHOLDS:?}:");
    for (i, name) in names.iter().enumerate() {
        let counts: Vec<String> = rows[i].iter().map(|b| b.to_string()).collect();
        println!("  {name:<44} {}", counts.join(" → "));
    }

    let flat = THRESHOLDS
        .iter()
        .position(|n| *n >= FLAT_FROM)
        .expect("the sweep reaches the flat point");
    for (i, name) in names.iter().enumerate() {
        let tail = &rows[i][flat..];
        assert!(
            tail.iter().all(|b| *b == tail[0]),
            "{name}: the sweep is no longer flat past {FLAT_FROM} columns — {:?}. That is not a \
             failure of the filter: it means this scene has grown a gap wide enough for a threshold \
             to argue about, and the sweep is now worth reading rather than only worth recording.",
            rows[i]
        );
    }
}

/// **Report.** What the filter costs in time, per damaged cell.
///
/// §8's number is 0.9 ns, from 88.94 µs against 112.83 µs on a full-screen 24 000-cell frame —
/// *under half the estimate, because the comparison rides inside a scan that was already reading
/// every cell*. The arms are the same scene under `Off` and under `Bytes`, so everything but the
/// comparison and the gap merge is identical between them and the difference is attributable.
///
/// **A report and not a gate**, for the register's own reason: a timing is a gate only at cliff
/// granularity. The cliff this is nowhere near is written next to the number — 24 µs against a
/// 16.6 ms frame interval — and §8's argument for *always* is that the price is that ratio while the
/// win is between 1% and 400x.
///
/// `Screen::present` is called directly rather than through the harness: the round trip's two
/// full-screen comparisons are an order of magnitude more work than the frame they check.
#[test]
fn what_the_equality_filter_costs_per_damaged_cell() {
    /// Frames timed after the warm-up, per arm.
    const SAMPLES: u32 = 20;
    /// One frame of a 16.6 ms interval, in microseconds: the number the cost is a fraction of.
    const INTERVAL_US: f64 = 16_600.0;

    let arm = |filter: Filter| {
        let mut scene = wired()
            .into_iter()
            .find(|s| s.name() == "full-screen-change")
            .expect("§14's twelve are the wired list");
        let mut h = staged_with(&mut *scene, filter);
        // Warm: the first steady frame reallocates nothing but does touch every page of the mirror
        // for the first time.
        for t in 1..=2 {
            scene.step(&mut h.screen, t);
            h.screen.present();
        }
        let at = std::time::Instant::now();
        for t in 3..3 + SAMPLES {
            scene.step(&mut h.screen, t);
            h.screen.present();
        }
        at.elapsed().as_secs_f64() * 1e6 / f64::from(SAMPLES)
    };

    let unfiltered = arm(Filter::Off);
    let filtered = arm(Filter::Bytes);
    let cells = f64::from(u32::from(W) * u32::from(H));
    println!(
        "\n  the filter on a full-screen {W}x{H} change, {SAMPLES} frames an arm:\n  \
         {unfiltered:.2} us unfiltered, {filtered:.2} us filtered, {:+.2} us the difference\n  \
         {:+.2} ns a damaged cell (spec §8: 0.9 ns, from 88.94 against 112.83 us)\n  \
         {:.3}% of a 16.6 ms interval. Report, not a gate.",
        filtered - unfiltered,
        (filtered - unfiltered) * 1000.0 / cells,
        (filtered - unfiltered) / INTERVAL_US * 100.0,
    );
}

// ---------------------------------------------------------------------------------------------
// Ticket 11 — the pairing invariant, over the composited frame rather than over one surface.
// ---------------------------------------------------------------------------------------------

/// Ticket 11's gate, and an equality against the reference compositor rather than a hand-written
/// expectation.
///
/// **Every gate in this file stayed green while §3's pairing invariant was false of the frame**,
/// which is the finding worth more than the case that produced it: the serializer emits nothing for
/// a continuation and the terminal model consumes nothing for one, so the round trip cannot see a
/// frame whose halves do not pair — *the model and the serializer are wrong in the same direction*.
/// So the property is asserted of the frame directly, and the picture is asserted against the
/// oracle, which does the same repair the other way round: a whole-screen scan of the finished
/// picture rather than four O(1) fixes per row per layer.
///
/// Twelve rectangles, alternating opaque and non-opaque, overlapping each other and the screen of
/// CJK underneath, plus one hanging off each edge — the case where a pair is bisected by the
/// *frame's* clamp rather than by a layer. Then they all move one column a frame, so every
/// bisection lands on the other parity by the next frame and every layer's own damage has to carry
/// the repair with it.
#[test]
fn the_pairing_invariant_survives_twelve_bisecting_layers_over_cjk() {
    let mut h = Harness::new(W, H).labelled("twelve-bisecting-layers");
    let base = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, W, H), true);
    for y in 0..H {
        let row = bisecting_cjk::row(y, W);
        h.screen
            .layers()
            .view(base)
            .unwrap()
            .text(0, y as i32, &row, Style::new());
    }

    let mut movers = Vec::new();
    for i in 0..bisecting_cjk::BISECTORS {
        let rect = bisecting_cjk::rect(i, W);
        let id = h.screen.layers().add_content(1 + i, rect, i % 2 == 0);
        for y in 0..rect.h {
            h.screen.layers().view(id).unwrap().text(
                0,
                y as i32,
                &bisecting_cjk::row(y, W),
                Style::new(),
            );
        }
        movers.push((id, rect));
    }
    h.present();
    assert_pairing_holds(h.screen.frame());

    for t in 1..=FRAMES {
        let before = h.screen.reference();
        for (id, rect) in &mut movers {
            rect.x += 1;
            h.screen.layers().set_rect(*id, *rect);
            for y in 0..rect.h {
                h.screen.layers().view(*id).unwrap().text(
                    0,
                    y as i32,
                    &bisecting_cjk::row(y, W),
                    Style::new(),
                );
            }
        }
        let after = h.screen.reference();
        h.present();

        assert_pairing_holds(h.screen.frame());
        for &(x, y) in &reference::differences(&before, &after) {
            assert!(
                covered(h.screen.runs(), x, y),
                "frame {t}: ({x}, {y}) changed and no run reported it"
            );
        }
        let frame = h.screen.frame();
        for y in 0..H {
            for x in 0..W {
                assert_eq!(
                    frame.row(y)[x as usize],
                    after.row(y)[x as usize],
                    "frame {t}: the damage-tracked frame and the reference compositor disagree \
                     at ({x}, {y})"
                );
            }
        }
    }

    // The other half, and the one a moving stack hides: the twelve stand still and a single cell
    // walks across them, so every frame is a **one-column run** against a screen full of pairs that
    // layer edges have already bisected. That is where a repair is measured against a cell no layer
    // in the run repainted, and where a repair that reached one column too far would erase a half
    // nothing was going to repaint.
    let marker =
        h.screen
            .layers()
            .add_content(bisecting_cjk::BISECTORS + 1, Rect::new(59, 30, 1, 1), true);
    h.screen
        .layers()
        .view(marker)
        .unwrap()
        .text(0, 0, "X", Style::new());
    // Presented before it is measured, for the reason every gate here stages its birth frame:
    // adding a layer damages its whole rectangle, and a reference taken before that frame is a
    // reference of a picture the frame never held.
    h.present();
    for t in 0..40u32 {
        let before = h.screen.reference();
        h.screen
            .layers()
            .set_rect(marker, Rect::new(60 + t as i32, 30, 1, 1));
        h.screen
            .layers()
            .view(marker)
            .unwrap()
            .text(0, 0, "X", Style::new());
        let after = h.screen.reference();
        h.present();

        assert_pairing_holds(h.screen.frame());
        for &(x, y) in &reference::differences(&before, &after) {
            assert!(
                covered(h.screen.runs(), x, y),
                "marker frame {t}: ({x}, {y}) changed and no run reported it"
            );
        }
        let frame = h.screen.frame();
        for y in 0..H {
            for x in 0..W {
                assert_eq!(
                    frame.row(y)[x as usize],
                    after.row(y)[x as usize],
                    "marker frame {t}: the damage-tracked frame and the reference compositor \
                     disagree at ({x}, {y})"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Ticket 12 — the atomic glyph rule, over the composited frame and against the oracle.
// ---------------------------------------------------------------------------------------------

/// Ticket 12's gate: twelve **operators** bisecting a screen of mixed CJK, walking a column a frame.
///
/// Ticket 11's gate is the same fixture with content layers, and the two are about different
/// failures. A content layer overwrites, so a bisected pair loses a half and the frame stops
/// pairing; an operator recolours, so nothing is orphaned and **the pair simply comes out in two
/// colours** — half a darkened `漢`, which is an artifact the five repair rules cannot see because
/// there is nothing for them to mend.
///
/// So this asserts the property the repair rules do not cover, directly and on every cell:
///
/// > Both halves of a double-width pair carry the same style word.
///
/// Plus the equality against the reference compositor, which states the atomic-glyph rule the other
/// way round — per cell, *where is this glyph's head*, against a picture it rebuilds for each
/// operator rather than tracks. And plus the round trip, because `Harness::present` is what every
/// gate here calls.
///
/// It is the gate's **own** fixture — [`bisecting_cjk`], rows and rects both, shared with ticket 11
/// and with the golden — because a fixture copied to suit a second instrument is one that goes on
/// passing after the first one's rectangles change underneath it.
///
/// # The two ways this gate could quietly test nothing, both closed
///
/// **A headless `Harness` is at `ColorDepth::None`**, because a caller-supplied sink is asked
/// nothing and §10 will not invent a colour for it — and at that depth §5 skips operator layers
/// *outright*, correctly. So the depth is pinned, the way [`crate::golden`] pins it, and the first
/// thing asserted is that the operators moved a cell at all. A gate whose subject never ran reports
/// success for the same reason a scene that draws nothing does.
///
/// **A cell with a default background is left unmixed**, which is spec §5's silent path and is
/// right: no `Overrides` can declare a default background (architecture ticket 22). So the content
/// is drawn in explicit colours, which is the path a real shadow over a themed panel takes and needs
/// no capability at all.
///
/// # And one class of case it still does not reach
///
/// All damage here comes from moving the operators, which exposes their whole rectangles — so a run
/// never begins at an operator's own edge. That is where a differential fuzz against the oracle
/// found two defects, and `layer.rs` carries the three regression tests for them beside its own
/// oracle gate. Said here as well, because a gate that names its fixture as *the* CJK fixture reads
/// as though it covered everything the fixture can express.
#[test]
fn the_atomic_glyph_rule_survives_twelve_bisecting_operators_over_cjk() {
    let pinned = crate::caps::Overrides {
        colors: Some(crate::caps::ColorDepth::TrueColor),
        ..Default::default()
    };
    let mut h = Harness::with_overrides(W, H, pinned).labelled("twelve-bisecting-operators");
    let base = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, W, H), true);
    let panel = Style::new()
        .fg(crate::style::Color::rgb(0xd0, 0xd0, 0xd0))
        .bg(crate::style::Color::rgb(0x30, 0x40, 0x50));
    for y in 0..H {
        let row = bisecting_cjk::row(y, W);
        h.screen
            .layers()
            .view(base)
            .expect("just added")
            .text(0, y as i32, &row, panel);
    }

    let mut movers = Vec::new();
    for i in 0..bisecting_cjk::BISECTORS {
        let rect = bisecting_cjk::rect(i, W);
        // Alternating intensities, so the twelve compound into more than two distinct results and
        // the memo is exercised rather than trivially hit.
        let id = h
            .screen
            .layers()
            .add_operator(1 + i, rect, Mix::darken(24 + 8 * i as u16));
        movers.push((id, rect));
    }
    h.present();
    assert_pairing_holds(h.screen.frame());
    assert_pairs_share_one_style(&h, 0);
    // The operators ran. Without this the whole gate passes at `ColorDepth::None`, where §5 skips
    // them and every assertion below is about a screen nothing recoloured. A floor rather than an
    // exact count, because the number belongs to the fixture's rectangles and would be edited every
    // time one of them moved — but a floor in the thousands cannot be met by an accident.
    let frame = h.screen.frame();
    let moved = (0..H)
        .flat_map(|y| (0..W).map(move |x| (x, y)))
        .filter(|&(x, y)| frame.row(y)[x as usize].style != panel)
        .count();
    assert!(
        moved > 1_000,
        "{moved} cells recoloured; the gate is testing nothing"
    );

    for t in 1..=FRAMES {
        let before = h.screen.reference();
        for (id, rect) in &mut movers {
            rect.x += 1;
            h.screen.layers().set_rect(*id, *rect);
        }
        let after = h.screen.reference();
        h.present();

        assert_pairing_holds(h.screen.frame());
        assert_pairs_share_one_style(&h, t);
        for &(x, y) in &reference::differences(&before, &after) {
            assert!(
                covered(h.screen.runs(), x, y),
                "frame {t}: ({x}, {y}) changed and no run reported it"
            );
        }
        let frame = h.screen.frame();
        for y in 0..H {
            for x in 0..W {
                assert_eq!(
                    frame.row(y)[x as usize],
                    after.row(y)[x as usize],
                    "frame {t}: the damage-tracked frame and the reference compositor disagree \
                     at ({x}, {y})"
                );
            }
        }
    }
}

/// A darkened wide glyph is one colour, not two.
///
/// The invariant the atomic-glyph rule exists for, asserted over the whole frame — and it is
/// **invisible to every other instrument here**, exactly as the pairing invariant was. The round
/// trip cannot see it: the serializer emits the head's SGR and skips the continuation, so a frame
/// whose two halves disagree serialises as though they agreed and replays as though they did too.
fn assert_pairs_share_one_style(h: &Harness, t: u32) {
    let frame = h.screen.frame();
    for y in 0..H {
        let row = frame.row(y);
        for x in 0..W as usize - 1 {
            if row[x].grapheme.is_wide_head() {
                assert_eq!(
                    row[x].style,
                    row[x + 1].style,
                    "frame {t}: half a darkened glyph at ({x}, {y})"
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The list itself.
// ---------------------------------------------------------------------------------------------

/// The scene list is normative, so its shape is asserted rather than assumed.
#[test]
fn the_scene_list_is_spec_14s_twelve() {
    let all = scenes();
    assert_eq!(all.len(), 12, "spec §14's table has twelve rows");

    let mut names: Vec<&str> = all.iter().map(|s| s.name()).collect();
    names.sort_unstable();
    let unique = names.len();
    names.dedup();
    assert_eq!(names.len(), unique, "two scenes share a name");

    for s in &all {
        assert!(
            s.iters() > 0,
            "scene `{}` would be timed over no work",
            s.name()
        );
        assert!(
            !s.decided().is_empty(),
            "scene `{}` does not say what it decided, and a scene that decided nothing is one \
             nobody can argue about removing",
            s.name()
        );
        // The same check the register's own entries get, from the same code: both are the
        // statement "this runs here" or "that ticket lights it", and a second copy of the
        // assertion is a second copy that can be softened.
        s.status()
            .assert_names_a_destination(&format!("scene `{}`", s.name()));
    }

    let red: Vec<&str> = all
        .iter()
        .filter(|s| !matches!(s.status(), State::Wired { .. }))
        .map(|s| s.name())
        .collect();
    assert!(
        red.is_empty(),
        "every scene on §14's normative list is wired since impl 13, which took the last one — the \
         hyperlinked page under an animating operator — off red by putting SGR 58/59 and OSC 8 on \
         the wire. A red scene now is either a mechanism that regressed or a scene added without a \
         measurement behind it: {red:?}"
    );
}

/// Every wired scene must actually damage something on every frame.
///
/// The failure this exists for is silent: a scene that draws nothing passes gate #1 (nothing
/// changed), gate #3 (0 == 0) and gate #12 (both screens are the last frame), and reports a
/// beautiful number in the budget harness. Prior art for it is right here — spec §14's own damage
/// ticket had three scenes that discriminated nothing.
#[test]
fn every_wired_scene_submits_a_frame() {
    for mut scene in wired() {
        let name = scene.name();
        let mut h = staged(&mut *scene);
        for t in 1..=FRAMES {
            let written = scene.step(&mut h.screen, t);
            assert!(written > 0, "{name}, frame {t}: the scene wrote nothing");
            assert!(
                h.present().submitted,
                "{name}, frame {t}: nothing was submitted"
            );
        }
    }
}

/// Gate #20's arms are the same scene at two sizes, and the drawing does not change with the size.
///
/// The ratio the example asserts is only a statement about data volume if the arms differ in
/// nothing else. Here that is checked where it is cheap — same name, same declared write count, and
/// the same picture after a frame — rather than inferred from the timing that is supposed to be the
/// conclusion.
///
/// Both shapes, because §14's row for the table is a pair of numbers "at 1k **and 1M**" and the
/// two scenes fail differently: the tree walks the data once into one layer, the table walks the
/// same rows twice into two.
#[test]
fn gate_20s_arms_are_one_scene_at_two_sizes() {
    for (mut small, mut large) in [
        (virtualised_tree(1_000), virtualised_tree(1_000_000)),
        (table_two_ways(1_000), table_two_ways(1_000_000)),
    ] {
        assert_eq!(small.name(), large.name());
        let mut hs = staged(&mut *small);
        let mut hl = staged(&mut *large);
        // Frame 1 of both draws rows 1..=80, because the offset is `t % (len - H)` and `t` is 1.
        assert_eq!(small.step(&mut hs.screen, 1), large.step(&mut hl.screen, 1));
        hs.present();
        hl.present();
        for y in 0..H {
            for x in 0..W {
                assert_eq!(
                    hs.screen.frame().row(y)[x as usize],
                    hl.screen.frame().row(y)[x as usize],
                    "the 1k and 1M arms of `{}` drew different pictures at ({x}, {y})",
                    small.name()
                );
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// #11 — after a sweep every live cell resolves to the same channels.
// #7  — a settled operator allocates zero; a fading one allocates per distinct extended style.
// ---------------------------------------------------------------------------------------------

/// A screen whose cells are extended, on a tier that has colour, **on the round trip**.
///
/// # Every gate in this section used to be unable to close it, and impl 13 is what ended that
///
/// `Harness::present` closes the round trip on every frame, and until impl 13 an extended cell could
/// not close it: SGR 58/59 and OSC 8 did not reach the wire, so the serializer emitted a hyperlinked,
/// underline-coloured cell in the right colours and dropped the two channels that made it extended.
/// The terminal model then held an *inline* cell where the frame held a handle, and comparing the two
/// compared one screen against a different spelling of it. `crate::testing`'s
/// `screen_without_a_round_trip` was the door around that, and it is **deleted**: its subject was
/// never a capability, it was work this crate had not done.
///
/// > **A door around an instrument is legitimate only while the instrument is incapable, and it must
/// > name the ticket that ends it.**
///
/// # The three axes this pins, and why each of them makes a gate vacuous if it is missing
///
/// **Truecolor**, because §5 skips an operator layer outright at [`ColorDepth::None`], which is what a
/// headless screen is unless something says otherwise. **Explicit colours** on every cell — see
/// [`opaque_ink`] — because a cell with a *default* background is left unmixed on a terminal silent
/// on OSC 11. And **`hyperlinks`**, because these fixtures make a cell extended with a hyperlink
/// deliberately, being the one channel `Mix` never names, and OSC 8 is emitted only where the
/// terminal has it — so without the declaration the round trip would be comparing a frame that
/// carries a link against a replay that could not have read one.
///
/// The third is architecture ticket 22's, and it is the one that was not available to write before:
/// `hyperlinks` is *inferred* rather than detected, and an inference is the one kind of fact a
/// declaration must be able to correct.
fn extended_screen(rows: u16) -> Harness {
    Harness::with_overrides(8, rows, crate::testing::pinned_extended())
}

/// The style every cell of an extended page starts from: explicit on both channels.
fn opaque_ink() -> Style {
    Style::new().fg(Color::indexed(15)).bg(Color::indexed(8))
}

/// A page of `rows` rows, with a **cluster** in the first column of each.
///
/// The cluster is there so the sweep has something in the *other* table to reclaim as well: a
/// fixture that only ever filled the extended-style table would pass with `Interner::compact` never
/// once called.
fn extended_page(screen: &mut Screen, rows: u16) -> crate::layer::LayerId {
    let id = screen
        .layers()
        .add_content(0, Rect::new(0, 0, 8, rows), true);
    let mut view = screen.layers().view(id).expect("just added");
    for y in 0..rows as i32 {
        view.text(0, y, "e\u{301}", opaque_ink());
        view.text(1, y, "abcdefg", opaque_ink());
    }
    id
}

/// Put one extended style on each row, minting one table entry per distinct colour `of` returns.
fn underline_each_row(
    screen: &mut Screen,
    id: crate::layer::LayerId,
    rows: u16,
    of: impl Fn(u16) -> Color,
) {
    let mut view = screen.layers().view(id).expect("the layer is still there");
    for y in 0..rows {
        view.restyle(
            Rect::new(0, y as i32, 8, 1),
            &Restyle {
                ul: Some(of(y)),
                ..Default::default()
            },
        );
    }
}

/// Gate #11, first half: **a sweep may change a cell's bytes and may not change what they say.**
///
/// The oracle is [`Screen::channels`], which resolves every handle before comparing — because
/// comparing `Cell` values would fail on every correct sweep, and comparing rendered text alone
/// would miss the four channels an extended style hides behind one handle.
///
/// The fixture puts four distinct extended styles on four rows and then moves three of the rows onto
/// a fifth, which leaves entries 1, 2 and 3 dead **below** live entry 4. That is the case the sweep
/// has to renumber for, and it is built by restyling rather than by reaching into a table, so it is a
/// case a caller can actually produce.
#[test]
fn a_sweep_preserves_every_live_cells_channels() {
    const ROWS: u16 = 4;
    let mut h = extended_screen(ROWS);
    let id = extended_page(&mut h.screen, ROWS);
    underline_each_row(&mut h.screen, id, ROWS, |y| Color::rgb(y as u8 + 1, 0, 0));
    h.present();
    assert_eq!(
        h.screen.table_lengths(),
        (1, ROWS as usize),
        "one cluster, and one extended style per row"
    );

    // Three of the four rows move onto a fifth style, orphaning entries 1, 2 and 3.
    {
        let mut view = h
            .screen
            .layers()
            .view(id)
            .expect("the layer is still there");
        view.restyle(
            Rect::new(0, 1, 8, ROWS - 1),
            &Restyle {
                ul: Some(Color::rgb(9, 9, 9)),
                ..Default::default()
            },
        );
    }
    h.present();
    assert_eq!(h.screen.table_lengths(), (1, ROWS as usize + 1));

    let before = h.screen.channels();
    let swept = h.screen.sweep_now();
    let after = h.screen.channels();

    assert!(swept.renumbered, "entry 4 had three dead entries below it");
    assert_eq!(swept.freed_exts, 3);
    assert_eq!(swept.live_exts, 2);
    assert_eq!(swept.freed_clusters, 0, "the cluster is still on every row");
    assert_eq!(swept.live_clusters, 1);
    assert_eq!(
        h.screen.table_lengths(),
        (1, 2),
        "the table holds exactly what is live"
    );
    assert_eq!(
        before, after,
        "a sweep changed what a cell says, not only how it is spelled"
    );
}

/// The same equality, over a sweep that reclaims a **cluster** rather than an extended style.
///
/// Two tables are swept and one gate over one of them would leave the other's compaction unexercised
/// — and they are not the same code twice: the interner's index is keyed by bytes it must not rehash,
/// and its handles carry a wide flag beside the id that a rewrite has to preserve. A screen of
/// double-width clusters is what makes both load-bearing.
#[test]
fn a_sweep_preserves_a_wide_clusters_channels_and_its_width() {
    const ROWS: u16 = 4;
    let mut h = extended_screen(ROWS);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 8, ROWS), true);
    // Four distinct wide clusters, one per row: a family emoji and three flags. Each is a ZWJ or
    // regional-indicator sequence, so each is interned, and each is two columns wide.
    const WIDE: [&str; 4] = [
        "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}",
        "\u{1F1EF}\u{1F1F5}",
        "\u{1F1E9}\u{1F1EA}",
        "\u{1F1EB}\u{1F1F7}",
    ];
    {
        let mut view = h.screen.layers().view(id).expect("just added");
        for (y, cluster) in WIDE.iter().enumerate() {
            view.text(0, y as i32, cluster, opaque_ink());
        }
    }
    h.present();
    assert_eq!(h.screen.table_lengths().0, 4, "four interned clusters");

    // Rows 1..3 are overwritten with the cluster row 0 holds, orphaning ids 1, 2 and 3 — which are
    // above the only live one, so this is also the second half's case seen from the interner.
    {
        let mut view = h
            .screen
            .layers()
            .view(id)
            .expect("the layer is still there");
        for y in 1..ROWS as i32 {
            view.text(0, y, WIDE[0], opaque_ink());
        }
    }
    h.present();

    let before = h.screen.channels();
    let swept = h.screen.sweep_now();
    assert_eq!(swept.freed_clusters, 3);
    assert_eq!(swept.live_clusters, 1);
    assert!(
        !swept.renumbered,
        "ids 1..3 were all above the only live one"
    );
    assert_eq!(h.screen.table_lengths().0, 1);
    assert_eq!(before, h.screen.channels());

    // The wide flag survived, which is what says the rewrite rebuilt the handle rather than the id.
    let frame = h.screen.frame();
    for y in 0..ROWS {
        assert!(
            frame.row(y)[0].grapheme.is_wide_head(),
            "row {y} lost its wide head"
        );
        assert!(frame.row(y)[1].grapheme.is_continuation());
    }
}

/// Gate #11, second half: **`renumbered` is false when nothing below a live entry was freed.**
///
/// Spec §3's sentence is *a sweep that frees nothing below a live entry does not renumber at all*,
/// and the reason it is a gate rather than a note is that the cheap case is the common one — a table
/// that grew from the top and lost its top — and getting it wrong is invisible. A sweep that
/// renumbered anyway would still produce a correct screen; it would set `repaint` and cost a mirror,
/// for ever, on every sweep.
///
/// The fixture frees the **top** two entries by moving their rows back onto the bottom two, which the
/// table already holds.
#[test]
fn a_sweep_that_frees_only_the_top_of_a_table_does_not_renumber() {
    const ROWS: u16 = 4;
    let mut h = extended_screen(ROWS);
    let id = extended_page(&mut h.screen, ROWS);
    underline_each_row(&mut h.screen, id, ROWS, |y| Color::rgb(y as u8 + 1, 0, 0));
    h.present();
    assert_eq!(h.screen.table_lengths(), (1, ROWS as usize));

    // Rows 2 and 3 take rows 0 and 1's styles, which the table already holds. Entries 2 and 3 go
    // dead and every survivor keeps its handle.
    underline_each_row(&mut h.screen, id, ROWS, |y| {
        Color::rgb(y as u8 % 2 + 1, 0, 0)
    });
    h.present();
    assert_eq!(
        h.screen.table_lengths(),
        (1, ROWS as usize),
        "nothing new was minted; two entries simply stopped being pointed at"
    );

    let before = h.screen.channels();
    let swept = h.screen.sweep_now();
    assert!(
        !swept.renumbered,
        "the two freed entries were above every live one"
    );
    assert_eq!(swept.freed_exts, 2);
    assert_eq!(swept.live_exts, 2);
    assert_eq!(
        h.screen.table_lengths(),
        (1, 2),
        "and they were still freed"
    );
    assert_eq!(before, h.screen.channels());
}

/// A sweep that renumbers marks **every** mirror row unknown, and one that does not marks none.
///
/// This is the flag itself — `Packet::repaint` (§3, §8, ADR 0006) — and the count is the gate: a full
/// repaint expresses itself as *every row unknown* rather than as a mode, so the number that has to
/// go to zero is the number of rows still claiming to know what the terminal shows.
///
/// The second half matters more than it looks. A sweep that set the flag unconditionally would pass
/// every correctness gate in this repository and cost a mirror on every sweep for ever, which is the
/// same failure shape as an overdraw of 37× — correct, and expensive where nobody is looking.
#[test]
fn a_renumbering_sweep_marks_every_mirror_row_unknown() {
    const ROWS: u16 = 4;
    let mut h = extended_screen(ROWS);
    let id = extended_page(&mut h.screen, ROWS);
    underline_each_row(&mut h.screen, id, ROWS, |y| Color::rgb(y as u8 + 1, 0, 0));
    h.present();
    assert_eq!(
        h.screen.known_rows(),
        ROWS as usize,
        "the birth frame wrote every row whole"
    );

    {
        let mut view = h
            .screen
            .layers()
            .view(id)
            .expect("the layer is still there");
        view.restyle(
            Rect::new(0, 1, 8, ROWS - 1),
            &Restyle {
                ul: Some(Color::rgb(9, 9, 9)),
                ..Default::default()
            },
        );
    }
    h.present();
    assert!(h.screen.sweep_now().renumbered);

    // Nothing is damaged, so this submits nothing — and the flag has to survive it. A flag spent on
    // a frame that never went out is a mirror that is never told.
    assert!(!h.present().submitted);
    assert_eq!(
        h.screen.known_rows(),
        ROWS as usize,
        "an idle present must not spend the flag"
    );

    // The next real frame is the one that carries it. Every row of it is written whole, so the rows
    // are unknown and known again inside one `present` — which is exactly what *costing one full
    // frame on the render thread* means.
    underline_each_row(&mut h.screen, id, ROWS, |_| Color::rgb(1, 2, 3));
    assert!(h.present().submitted);
    assert_eq!(h.screen.known_rows(), ROWS as usize);

    // A narrow frame after a renumbering sweep leaves every cell it did not write unknown, which is
    // the half ADR 0006's *written whole rather than compared* does not reach: the sweep marks no
    // damage, so there is no whole row to write. What impl 14's filter does about it is the reason
    // the granularity is the cell — see `crate::serial::Mirror` and ADR 0006's amendment.
    underline_each_row(&mut h.screen, id, ROWS, |y| Color::rgb(y as u8 + 1, 0, 0));
    h.present();
    {
        let mut view = h
            .screen
            .layers()
            .view(id)
            .expect("the layer is still there");
        view.restyle(
            Rect::new(0, 1, 8, ROWS - 1),
            &Restyle {
                ul: Some(Color::rgb(7, 7, 7)),
                ..Default::default()
            },
        );
    }
    h.present();
    assert!(h.screen.sweep_now().renumbered);
    {
        let mut view = h
            .screen
            .layers()
            .view(id)
            .expect("the layer is still there");
        view.text(3, 0, "z", opaque_ink());
    }
    assert!(h.present().submitted);
    assert_eq!(
        h.screen.known_rows(),
        0,
        "one cell of one row is not one whole row of four"
    );
}

/// A sweep that renumbers nothing may not invalidate a mirror row.
#[test]
fn a_sweep_that_renumbers_nothing_leaves_the_mirror_alone() {
    const ROWS: u16 = 4;
    let mut h = extended_screen(ROWS);
    let id = extended_page(&mut h.screen, ROWS);
    underline_each_row(&mut h.screen, id, ROWS, |y| Color::rgb(y as u8 + 1, 0, 0));
    h.present();
    underline_each_row(&mut h.screen, id, ROWS, |y| {
        Color::rgb(y as u8 % 2 + 1, 0, 0)
    });
    h.present();
    assert_eq!(h.screen.known_rows(), ROWS as usize);

    let swept = h.screen.sweep_now();
    assert!(!swept.renumbered);
    assert_eq!(swept.freed_exts, 2, "it did reclaim, it just did not move");

    // A frame that damages one cell of one row. Were the flag set, every row would go unknown.
    {
        let mut view = h
            .screen
            .layers()
            .view(id)
            .expect("the layer is still there");
        view.text(3, 0, "z", opaque_ink());
    }
    assert!(h.present().submitted);
    assert_eq!(h.screen.known_rows(), ROWS as usize);
}

/// **There is no path by which a sweep runs inside `present`.**
///
/// Spec §13 allows the sweep its cliff — 82.5 µs at one screen, 897 µs at twenty layers — because it
/// stays off the path the 100 µs and 1 ms budgets are taken around, so this is the acceptance line
/// for that sentence rather than a restatement of it. It is a count, and the count is taken while
/// the high-water mark is **already reached** — otherwise the gate would be about the mark not
/// having been hit, which is a different and much weaker claim.
///
/// What it does not claim is that the sweep never runs during a frame at all: `Screen::layers` is
/// also the drawing door, so it does. [`Screen::layers`](crate::Screen) is exact about the
/// difference, and §3 is what asks for the narrower property.
#[test]
fn the_sweep_never_runs_inside_present() {
    let mut h = extended_screen(4);
    let id = extended_page(&mut h.screen, 4);
    h.present();

    // Past the floor: one new distinct extended style per frame until the mark is reached. Two
    // colour channels rather than one, because the floor is 256 entries and one byte would run out
    // of distinct values at exactly the wrong moment.
    let mut distinct = 0u32;
    while !h.screen.sweep_due() {
        distinct += 1;
        assert!(distinct < 4096, "the high-water mark was never reached");
        {
            let mut view = h
                .screen
                .layers()
                .view(id)
                .expect("the layer is still there");
            view.restyle(
                Rect::new(0, 0, 8, 1),
                &Restyle {
                    ul: Some(Color::rgb(distinct as u8, (distinct >> 8) as u8, 1)),
                    ..Default::default()
                },
            );
        }
        h.present();
    }

    // `layers()` is the door, and it has not been opened since the mark was reached.
    let before = h.screen.sweeps();
    for _ in 0..4 {
        h.present();
    }
    assert!(h.screen.sweep_due(), "the mark is still reached");
    assert_eq!(
        h.screen.sweeps(),
        before,
        "`present` swept, and §13's budget for the sweep rests on the promise that it cannot"
    );

    // And the door does open, exactly once, and moves the mark past what is live.
    h.screen.layers();
    assert_eq!(h.screen.sweeps(), before + 1);
    assert!(
        !h.screen.sweep_due(),
        "the high-water mark did not move past the live count"
    );
}

/// Gate #7, the fading half: **a fading operator mints per distinct extended style, never per cell.**
///
/// `tests/alloc.rs` holds the settled half, where the answer is zero. This is the other one, and it
/// is a **count** rather than an allocation window for a reason the register states: the fading case
/// is *supposed* to allocate, so an allocation probe over it can only say "some", where the shape of
/// the growth is the whole property. Spec §3's numbers are 0.8 entries a frame settled against 95.7
/// fading on 96 distinct extended styles — a ratio of a hundred-odd, not of 24 000.
///
/// The operator animates the only way this API allows: **there is no `set_mix`**, so a moving
/// `amount` is a `remove` and an `add_operator`, which is a scene topology change and passes through
/// [`Screen::layers`]. That is not incidental — it is why that door is the right place for the
/// high-water check.
#[test]
fn a_fading_operator_mints_per_distinct_style_and_never_per_cell() {
    const ROWS: u16 = 8;
    const CELLS: usize = 8 * ROWS as usize;
    let mut h = extended_screen(ROWS);
    let id = extended_page(&mut h.screen, ROWS);

    // Four distinct extended styles over sixty-four cells: enough that "per distinct style" and
    // "per cell" are different numbers by a factor of sixteen.
    //
    // They differ by **hyperlink** and not by underline colour, and that is the difference between
    // a gate about a mechanism and a gate about arithmetic. `Mix` blends the underline colour along
    // with the other two, so four underline colours a few units apart collapse into fewer than four
    // after blending — the first draft of this gate measured 3, then 2, and the number it was
    // reporting was `blend`'s truncation rather than the table's growth. A hyperlink is the one
    // channel the mix never touches (§5's *it is not preserved here, it is simply never named*), so
    // four links are four distinct results at **every** amount, by construction.
    let distinct = 4usize;
    let links: Vec<_> = (0..distinct)
        .map(|i| h.screen.link(&format!("https://example.com/{i}")))
        .collect();
    {
        let mut view = h
            .screen
            .layers()
            .view(id)
            .expect("the layer is still there");
        for y in 0..ROWS {
            view.restyle(
                Rect::new(0, y as i32, 8, 1),
                &Restyle {
                    link: Some(links[y as usize % distinct]),
                    ..Default::default()
                },
            );
        }
    }
    h.present();
    let page = h.screen.table_lengths().1;
    assert_eq!(page, distinct, "the page itself holds four extended styles");

    let before_any_operator = h.screen.frame().row(0)[1];
    let mut op =
        h.screen
            .layers()
            .add_operator(1, Rect::new(0, 0, 8, ROWS), Mix::darken(Mix::FULL / 4));
    h.present();
    // **Assert the operator moved a cell before asserting anything about which ones.** A headless
    // screen is at `ColorDepth::None` unless something pins it, and there an operator layer is
    // skipped outright — a fade that touched nothing would report a beautiful zero growth and mean
    // nothing at all.
    assert_ne!(
        h.screen.frame().row(0)[1],
        before_any_operator,
        "the operator changed no cell, so this gate is measuring the depth and not the fade"
    );

    // The settled shape first, because the fading number means nothing without it: the same `amount`
    // again mints nothing at all.
    let settled = h.screen.table_lengths().1;
    h.present();
    assert_eq!(
        h.screen.table_lengths().1,
        settled,
        "a settled operator converges after one frame"
    );

    // Ten frames of a fade, each with an `amount` no earlier frame used.
    let mut minted = Vec::new();
    for step in 1..=10u16 {
        let was = h.screen.table_lengths().1;
        h.screen.layers().remove(op);
        op = h.screen.layers().add_operator(
            1,
            Rect::new(0, 0, 8, ROWS),
            Mix::darken(Mix::FULL / 4 + step * 8),
        );
        h.present();
        minted.push(h.screen.table_lengths().1 - was);
    }

    for (step, &n) in minted.iter().enumerate() {
        assert_eq!(
            n, distinct,
            "fade frame {step}: {n} entries minted for {distinct} distinct extended styles \
             ({minted:?})"
        );
        assert!(
            n < CELLS,
            "fade frame {step}: {n} entries for {CELLS} cells is per cell, not per style"
        );
    }

    // And the growth is what the sweep exists for: it is unbounded in frames and bounded in styles.
    assert_eq!(h.screen.table_lengths().1, page + distinct * 11);
    let swept = h.screen.sweep_now();
    assert_eq!(
        swept.live_exts,
        distinct * 2,
        "the page's four in the layer surface, and the last frame's four mixed results in the \
         frame — the other ten frames' results are pointed at by nothing"
    );
    assert_eq!(swept.freed_exts, distinct * 10);
    assert!(
        swept.renumbered,
        "the page's own four entries are below every freed one"
    );
    assert_eq!(h.screen.table_lengths().1, distinct * 2);
}

// ---------------------------------------------------------------------------------------------
// Impl 17 — quantisation runs before the mirror comparison.
// ---------------------------------------------------------------------------------------------

/// A harness on a terminal of exactly this depth, with the rest of a scene's pins kept.
///
/// **`..scene.overrides()` and not `..Default::default()`**, because the twelfth scene needs OSC 8
/// declared as well and a driver that dropped that pin while varying the depth would be measuring a
/// third thing. See [`crate::scenes::Scene::overrides`].
fn staged_at(scene: &mut dyn Scene, colors: crate::caps::ColorDepth) -> Harness {
    let mut h = Harness::with_overrides(
        W,
        H,
        Overrides {
            colors: Some(colors),
            ..scene.overrides()
        },
    )
    .labelled(scene.name());
    scene.build(&mut h.screen);
    h.present();
    h
}

/// Every colour an SGR sequence in `out` selects, as `(channel, colour)` — where the channel is 38,
/// 48 or 58 and the colour is either an index or `None` for a truecolor triple.
///
/// **A walk and not a split, and the first draft was a split.** Flattening `;` and `:` alike is
/// right — it is what makes the answer the same whether a colour was spelled `38:5:9`, `38;5;9` or
/// `31` — but *then asking whether any number lies in 30..38* is wrong, and wrong in the direction
/// that fails on correct code: `38:5:32` is cube index 32, and its third field is a number in that
/// range. So the selectors are consumed with their payloads, which is the smallest amount of SGR
/// structure that makes the question decidable.
///
/// A truecolor triple's channels are deliberately not returned. Nothing here asks about them: what
/// these gates are about is which *index* a narrowing chose, and an RGB colour chose none.
fn sgr_colours(out: &[u8]) -> Vec<(u32, Option<u32>)> {
    let mut found = Vec::new();
    let mut i = 0;
    while i + 1 < out.len() {
        if out[i] != 0x1b || out[i + 1] != b'[' {
            i += 1;
            continue;
        }
        let start = i + 2;
        let mut end = start;
        while end < out.len() && !(0x40..=0x7e).contains(&out[end]) {
            end += 1;
        }
        if end < out.len() && out[end] == b'm' {
            let mut fields = Vec::new();
            for field in out[start..end].split(|b| *b == b';' || *b == b':') {
                // An empty field is T.416's colour-space id, which is meant to be empty.
                if let Ok(n) = std::str::from_utf8(field).unwrap_or("x").parse::<u32>() {
                    fields.push(n);
                }
            }
            let mut p = 0;
            while p < fields.len() {
                match fields[p] {
                    ch @ (38 | 48 | 58) if p + 1 < fields.len() => {
                        match fields[p + 1] {
                            5 => {
                                found.push((ch, fields.get(p + 2).copied()));
                                p += 3;
                            }
                            2 => {
                                found.push((ch, None));
                                p += 5;
                            }
                            // Neither spelling, which the serializer never emits. Stepping one at a
                            // time is what makes a malformed stream loud rather than silently skipped.
                            _ => p += 1,
                        }
                    }
                    // The short forms, which are the low sixteen and nothing else.
                    n @ (30..=37) => {
                        found.push((38, Some(n - 30)));
                        p += 1;
                    }
                    n @ (90..=97) => {
                        found.push((38, Some(n - 90 + 8)));
                        p += 1;
                    }
                    n @ (40..=47) => {
                        found.push((48, Some(n - 40)));
                        p += 1;
                    }
                    n @ (100..=107) => {
                        found.push((48, Some(n - 100 + 8)));
                        p += 1;
                    }
                    _ => p += 1,
                }
            }
        }
        i = end.max(i + 1);
    }
    found
}

/// **The gate the whole placement exists for**, and the correction the ticket was written around:
///
/// > "Quantising colour saves almost nothing" is right about frame *size* and silent about frame
/// > *membership*.
///
/// Two RGB values that collapse to one index at [`ColorDepth::Ansi16`](crate::ColorDepth) must make
/// the second frame **empty**. If the mirror held the colours the application asked for, the two
/// would compare unequal and 24 000 cells would be re-emitted for no visible change — the equality
/// filter would under-filter by exactly the amount the depth collapses.
///
/// The truecolor arm is what stops this passing for the wrong reason. A fixture that drew nothing,
/// or drew the same colour twice, would satisfy the first half; the same two colours at a depth that
/// can tell them apart have to produce a frame.
#[test]
fn two_colours_a_depth_cannot_tell_apart_are_one_frame_and_not_two() {
    /// Both land on ANSI entry 1, and neither is it.
    const NEAR: [Color; 2] = [Color::rgb(0xcc, 0x02, 0x01), Color::rgb(0xcf, 0x00, 0x03)];

    fn second_frame(colors: crate::caps::ColorDepth) -> usize {
        let mut h = Harness::with_overrides(
            W,
            H,
            Overrides {
                colors: Some(colors),
                ..Overrides::default()
            },
        );
        let id = h
            .screen
            .layers()
            .add_content(0, Rect::new(0, 0, W, H), true);
        let row: String = std::iter::repeat_n('m', W as usize).collect();
        let paint = |h: &mut Harness, c: Color| {
            let mut v = h.screen.layers().view(id).expect("just added");
            for y in 0..H as i32 {
                v.text(0, y, &row, Style::new().fg(c));
            }
        };
        paint(&mut h, NEAR[0]);
        h.present();
        let before = h.bytes_written();
        paint(&mut h, NEAR[1]);
        h.present();
        h.bytes_written() - before
    }

    assert_eq!(
        second_frame(crate::caps::ColorDepth::Ansi16),
        0,
        "a terminal that cannot tell the two apart was sent a second frame"
    );
    assert!(
        second_frame(crate::caps::ColorDepth::TrueColor) > 0,
        "a terminal that can tell them apart was sent nothing, so the arm above is vacuous"
    );
}

/// **Gate: `bytes_16 < bytes_256 < bytes_truecolor`** on the worst-case scene.
///
/// §8's own table is 934 058 / 1 658 570 / 2 417 060 — **fewer** bytes at sixteen colours, because
/// `SGR 31` is two bytes where `38:2::205:0:0` is thirteen.
///
/// > §14's `t_16 > t_256` is an abstract example about quantisation **time**; reading it as bytes
/// > inverts the gate and fails it on correct code.
///
/// A relation and not three equalities, because the numbers belong to the *data* — the scene's own
/// colour walk — and a number that belongs to the data must be a relation or it becomes a gate that
/// is edited rather than fixed ([`crate::register`]'s first refinement).
///
/// It is asked of `every-cell-a-distinct-style` because that scene is the only one on §14's list
/// where every cell carries a colour no other cell carries, so nothing else on the list can
/// distinguish a serializer that narrowed from one that did not.
#[test]
fn fewer_bytes_the_less_colour_the_terminal_has() {
    use crate::caps::ColorDepth::{Ansi16, Indexed256, TrueColor};

    let mut scene = wired()
        .into_iter()
        .find(|s| s.name() == "every-cell-a-distinct-style")
        .expect("the worst case is on the wired list");

    let mut bytes = Vec::new();
    for depth in [Ansi16, Indexed256, TrueColor] {
        let mut h = staged_at(&mut *scene, depth).without_scroll_region();
        let before = h.bytes_written();
        for t in 1..=FRAMES {
            scene.step(&mut h.screen, t);
            h.present();
        }
        let n = h.bytes_written() - before;
        println!("  {:<12} {n:>9} bytes", format!("{depth:?}"));
        bytes.push(n);
    }

    assert!(
        bytes[0] < bytes[1] && bytes[1] < bytes[2],
        "the wire did not get cheaper as the terminal got poorer: {bytes:?}"
    );
}

/// **Gate: at [`ColorDepth::Indexed256`](crate::ColorDepth), no output ever names an index under
/// sixteen as a quantisation target.**
///
/// 16..256 are fixed by specification and identical on every terminal; 0..16 are repainted by the
/// user's theme, so quantising into them is a bet on somebody else's colour scheme. Asked of the
/// **wire** rather than of the function — `crate::quant` has the exhaustive unit gate — because what
/// the register is about is bytes, and a serializer that spelled a cube index with a short form
/// would pass the function's gate and paint the user's theme anyway.
///
/// The fixture carries **only** RGB colours, which is what makes the assertion decidable: no cell
/// asked for a low index, so any low index on the wire was chosen rather than carried.
#[test]
fn at_256_the_wire_never_names_the_users_own_sixteen() {
    let mut scene = wired()
        .into_iter()
        .find(|s| s.name() == "every-cell-a-distinct-style")
        .expect("the worst case is on the wired list");
    let mut h = staged_at(&mut *scene, crate::caps::ColorDepth::Indexed256);
    scene.step(&mut h.screen, 1);
    h.present();

    let bytes = h.wire();
    let colours = sgr_colours(&bytes);
    let mut indexed = 0usize;
    for (channel, index) in &colours {
        let Some(i) = *index else {
            panic!("a truecolor triple reached a terminal that cannot parse one");
        };
        assert!(
            i >= 16,
            "channel {channel} was narrowed to index {i}, which is the user's own theme"
        );
        indexed += 1;
    }
    assert!(
        indexed > 0,
        "nothing on the wire named a colour, so this gate asserted nothing"
    );
}

/// **The other half of ADR 0025's silence rule, asked of a frame rather than of a `Mixer`.**
///
/// Spec §5 leaves a cell with a default background **unmixed** where OSC 11 was silent, because a
/// guessed background inverts a shadow on the opposite theme — wrong in *direction*, where a themed
/// palette is only wrong in degree. `crate::mix` has gated the silent arm since impl 12; the
/// **answered** arm was unreachable from a `Screen` until impl 13 gave [`Overrides`] `default_bg`,
/// so the pair could only be asserted one layer below the frame.
///
/// Both arms here, on one fixture, at one depth: the cell is default-backgrounded, an operator
/// darkens it, and the only difference between the two screens is whether the terminal said what its
/// background is.
#[test]
fn a_default_background_is_mixed_only_where_the_terminal_said_what_it_is() {
    fn painted(default_bg: Option<crate::caps::Rgb>) -> Style {
        let mut h = Harness::with_overrides(
            8,
            2,
            Overrides {
                colors: Some(crate::caps::ColorDepth::TrueColor),
                default_bg,
                ..Overrides::default()
            },
        );
        let id = h
            .screen
            .layers()
            .add_content(0, Rect::new(0, 0, 8, 2), true);
        h.screen
            .layers()
            .view(id)
            .expect("just added")
            // **Explicit foreground, default background.** §5 refuses the whole cell when *either*
            // channel is a default the terminal did not name, so a cell default on both would be
            // left alone in the answered arm too — for a reason that is about the foreground, and
            // this gate is about the background.
            .text(0, 0, "abcdefgh", Style::new().fg(Color::indexed(15)));
        h.screen
            .layers()
            .add_operator(1, Rect::new(0, 0, 8, 2), Mix::darken(Mix::FULL / 2));
        h.present();
        h.screen.frame().row(0)[0].style
    }

    let white = crate::caps::Rgb::new(0xff, 0xff, 0xff);
    let untouched = Style::new().fg(Color::indexed(15));
    assert_eq!(
        painted(None),
        untouched,
        "a guessed background is wrong in direction, so a silent terminal leaves the cell alone"
    );
    let answered = painted(Some(white));
    assert_ne!(
        answered, untouched,
        "the terminal said what its background is and the operator declined to use it"
    );
    assert_eq!(
        answered.background(),
        Color::rgb(0x7f, 0x7f, 0x7f),
        "white, half of the way to black"
    );
}

/// **Gate: the intern-key collapse is eight entries against ninety-six, and not one byte.**
///
/// Spec §10's one narrow exception to *degrade at serialise time*:
///
/// > A channel the terminal **cannot express at all** is dropped from the intern *key* on the app
/// > thread; a channel it expresses **imprecisely** is degraded at serialise time.
///
/// Three arms, because two of them are the same number and the third is what makes either mean
/// anything:
///
/// 1. `hyperlinks: Some(true)` — ninety-six links over ninety-six cells, each with one of eight
///    underline colours, is **ninety-six** distinct extended styles.
/// 2. `Some(false)` — the link leaves the key, and what is left is the eight the underline colours
///    alone distinguish. **One arm alone is not the gate**: a fixture at `false` that reported 8
///    would be indistinguishable from one that never reached the table at all, which is why the
///    positive half below asserts the cells are still extended.
/// 3. `Some(false)` with §7's **rejected** placement — the link kept in the key and dropped at
///    serialise time instead — which is ninety-six entries again and, byte for byte, **the same
///    wire**. Two style words differing only in a channel the serializer will not emit produce an
///    SGR delta with nothing in it, and the emit loop takes back the escape it speculatively
///    opened.
///
/// So the collapse buys a growth bound and costs nothing, which is why it is taken. The one place
/// the two placements could diverge in bytes is a **gap**: `price_gap` charges `SGR_FLOOR` per
/// distinct narrowed word, so a gap spanning cells that differ only by an unemittable link is
/// priced higher under the rejected placement. This fixture damages whole rows and has no gap, and
/// the divergence is recorded here rather than left to be discovered.
#[test]
fn dropping_an_inexpressible_channel_from_the_key_costs_entries_and_no_bytes() {
    /// One cell per link, and eight underline colours cycling through them.
    ///
    /// Eight rather than one because the point of the `false` arm is a number that is neither 96 nor
    /// zero: a fixture whose only extended channel were the link would collapse to *nothing* and
    /// could not tell a table that emptied from a table nobody visited.
    const LINKS: u16 = 96;
    const ULS: u16 = 8;

    fn arm(hyperlinks: bool, rejected_placement: bool) -> (usize, Vec<u8>) {
        let mut h = Harness::with_overrides(
            LINKS,
            2,
            Overrides {
                colors: Some(crate::caps::ColorDepth::TrueColor),
                hyperlinks: Some(hyperlinks),
                ..Overrides::default()
            },
        );
        if rejected_placement {
            h.screen.tables_mut().keep_links_in_key();
        }
        let id = h
            .screen
            .layers()
            .add_content(0, Rect::new(0, 0, LINKS, 2), true);
        let links: Vec<_> = (0..LINKS)
            .map(|i| h.screen.link(&format!("https://example.com/vitui#{i}")))
            .collect();
        {
            let row: String = std::iter::repeat_n('m', LINKS as usize).collect();
            let mut v = h.screen.layers().view(id).expect("just added");
            v.text(0, 0, &row, Style::new().fg(Color::indexed(15)));
            v.text(0, 1, &row, Style::new().fg(Color::indexed(15)));
            for (i, link) in links.iter().enumerate() {
                v.restyle(
                    Rect::new(i as i32, 0, 1, 2),
                    &Restyle {
                        set: Restyle::UNDERLINE_CURLY,
                        ul: Some(Color::rgb((i as u16 % ULS) as u8 + 1, 0, 0)),
                        link: Some(*link),
                        ..Restyle::default()
                    },
                );
            }
        }
        // **Counted before the frame goes out, and that is not tidiness.** The terminal model
        // interns what it reads into this screen's own tables — which is what lets the round trip
        // compare handles at all — so it mints the eight entries an SGR 58 without an OSC 8 names,
        // and the rejected arm's ninety-six would read as a hundred and four. The number this gate
        // is about is what the *drawing verbs* minted.
        let entries = h.screen.table_lengths().1;
        let before = h.bytes_written();
        h.present();
        // The positive half: whatever the counts say, these cells reached a table.
        for x in 0..LINKS {
            assert!(
                h.screen.frame().row(0)[x as usize].style.is_extended(),
                "column {x} is not extended, so the counts below are about nothing"
            );
        }
        let bytes = h.wire()[before..].to_vec();
        (entries, bytes)
    }

    let (expressible, _) = arm(true, false);
    let (collapsed, collapsed_bytes) = arm(false, false);
    let (rejected, rejected_bytes) = arm(false, true);

    assert_eq!(expressible, LINKS as usize, "one entry per distinct link");
    assert_eq!(
        collapsed, ULS as usize,
        "the link left the key and the underline colours are what is left"
    );
    assert_eq!(
        rejected, LINKS as usize,
        "§7's rejected placement keeps the link in the key, so it keeps the entries"
    );
    assert_eq!(
        collapsed_bytes.len(),
        rejected_bytes.len(),
        "the collapse changed the wire, which is the one thing it is not supposed to do"
    );
    assert_eq!(
        collapsed_bytes, rejected_bytes,
        "the same bytes in a different order is still a change"
    );
}

/// **Spec §15's first owed measurement, paid: what narrowing costs per style word**, against the
/// equality filter's 0.9 ns per damaged cell.
///
/// Two numbers, because the mechanism has two costs and one instrument cannot isolate both.
///
/// **The memo's cost, in situ.** `full-screen-change` paints 24 000 cells from **one** style word,
/// and its colour is an index under sixteen — expressible at every depth that has colour — so the
/// wire is byte-identical at `Ansi16` and at `TrueColor` and the whole difference between the two
/// frames is the narrowing. That is asserted rather than assumed, because an isolation that is
/// merely plausible is not one.
///
/// **The narrowing's own cost, per distinct word.** The frame times cannot give it: on
/// `every-cell-a-distinct-style` a poorer terminal narrows 24 000 words *and* writes a ninth of the
/// bytes, so the difference is a sum of the two with opposite signs. So it is timed directly, over
/// the style words of a real frame — read off the frame rather than re-deriving the scene's colour
/// walk, because a second copy of the walk is a second thing to keep in step — through
/// [`Quantiser::style`](crate::quant::Quantiser::style), which is the function the run scan calls.
///
/// A report, not a gate: §14's rule is that a timing is a gate only at cliff granularity with the
/// headroom written beside it, and nothing here is near a cliff.
#[test]
fn what_narrowing_colour_costs_per_style_word() {
    use crate::caps::ColorDepth::{Ansi16, Indexed256, TrueColor};

    const SAMPLES: u32 = 20;
    let cells = f64::from(u32::from(W) * u32::from(H));

    // ---- the memo, in situ, on a scene whose wire does not move with the depth ----------------
    let arm = |depth| {
        let mut scene = wired()
            .into_iter()
            .find(|s| s.name() == "full-screen-change")
            .expect("§14's twelve are the wired list");
        let mut h = staged_at(&mut *scene, depth);
        for t in 1..=2 {
            scene.step(&mut h.screen, t);
            h.present();
        }
        let before = h.bytes_written();
        let at = std::time::Instant::now();
        for t in 3..3 + SAMPLES {
            scene.step(&mut h.screen, t);
            h.screen.present();
        }
        let us = at.elapsed().as_secs_f64() * 1e6 / f64::from(SAMPLES);
        (us, h.bytes_written() - before)
    };
    let (transparent, transparent_bytes) = arm(TrueColor);
    let (narrowed, narrowed_bytes) = arm(Ansi16);
    assert_eq!(
        transparent_bytes, narrowed_bytes,
        "this scene's colours are indices under sixteen, so the two depths spell them the same — \
         and if they ever do not, the difference below stops being the narrowing"
    );

    // ---- the narrowing itself, per distinct style word ----------------------------------------
    let mut scene = wired()
        .into_iter()
        .find(|s| s.name() == "every-cell-a-distinct-style")
        .expect("§14's twelve are the wired list");
    let mut h = staged_at(&mut *scene, TrueColor);
    scene.step(&mut h.screen, 1);
    h.present();
    let words: Vec<Style> = (0..H)
        .flat_map(|y| h.screen.frame().row(y).iter().map(|c| c.style))
        .collect();
    assert_eq!(words.len(), cells as usize);
    let per_word = |depth| {
        let q = crate::quant::Quantiser::for_terminal(&crate::caps::Capabilities::answering(
            depth, None, None,
        ));
        // Warm, then timed: the first pass faults the vector in.
        for w in &words {
            std::hint::black_box(q.style(*w));
        }
        let at = std::time::Instant::now();
        for w in &words {
            std::hint::black_box(q.style(*w));
        }
        at.elapsed().as_secs_f64() * 1e9 / cells
    };

    println!(
        "\n  narrowing colour, {W}x{H}:\n  \
         memoised, one style word for {cells:.0} cells, {SAMPLES} frames an arm:\n    \
         {transparent:.2} us at truecolor, {narrowed:.2} us at sixteen, {:+.2} us the difference\n    \
         {:+.3} ns a cell (the equality filter's own is 0.9 ns, spec §8)\n  \
         unmemoised, {cells:.0} distinct style words:\n    \
         {:.2} ns a word at indexed256, {:.2} ns a word at sixteen\n  \
         Report, not a gate.",
        narrowed - transparent,
        (narrowed - transparent) * 1000.0 / cells,
        per_word(Indexed256),
        per_word(Ansi16),
    );
}

/// **Gate: narrowing is per distinct style word, and never twice per cell.**
///
/// A count, because the failure this is about is invisible to every correctness test and to every
/// byte count: the one-entry memo can answer *one* spelling of a word and store the other, and then
/// a screen of a single colour pays two full narrowings a cell while producing exactly the right
/// bytes. It did, and it cost **1 103 µs against 429 µs** on a 300×80 screen of one RGB colour at
/// sixteen colours — a 2.6x frame, all of it in a function whose output was correct.
///
/// The shape of the defect is structural rather than careless, which is why the counter stays. The
/// run scan narrows a cell to compare it against the mirror; `emit_cell` narrows again so that it is
/// correct whoever called it, because the gap merge sources columns the scan never saw. So every
/// emitted cell asks twice, in two different spellings of the same word, and a memo that recognises
/// one of them is a memo that recognises neither. See [`Serializer::narrow`](crate::serial).
///
/// Both arms, because one is satisfied by a memo that never stores anything:
///
/// - a screen of **one** style word is narrowed **once** a frame;
/// - a screen of 24 000 **distinct** style words is narrowed 24 000 times and not 48 000.
#[test]
fn a_frame_narrows_once_a_distinct_style_word_and_never_twice_a_cell() {
    const CELLS: usize = 300 * 80;
    let ansi16 = Overrides {
        colors: Some(crate::caps::ColorDepth::Ansi16),
        ..Overrides::default()
    };

    // One style word, and every cell emitted every frame: the glyph alternates so the equality
    // filter cannot empty the frame, because a filtered frame narrows in the scan and never in
    // `emit_cell` and would satisfy this arm for the wrong reason.
    let mut h = Harness::with_overrides(W, H, ansi16);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, W, H), true);
    let rows: [String; 2] = [
        std::iter::repeat_n('x', W as usize).collect(),
        std::iter::repeat_n('y', W as usize).collect(),
    ];
    let ink = Style::new().fg(Color::rgb(0x10, 0x40, 0x90));
    for t in 0..3 {
        {
            let mut v = h.screen.layers().view(id).expect("just added");
            for y in 0..H as i32 {
                v.text(0, y, &rows[t % 2], ink);
            }
        }
        let before = h.screen.narrowings();
        h.present();
        let narrowings = h.screen.narrowings() - before;
        assert_eq!(
            narrowings, 1,
            "frame {t}: {narrowings} narrowings for one distinct style word over {CELLS} cells"
        );
    }

    // And the arm that says the memo is one entry deep rather than a table: a screen where no two
    // cells share a word narrows once a cell, which is 24 000 — not the 48 000 the two spellings
    // would cost.
    let mut scene = wired()
        .into_iter()
        .find(|s| s.name() == "every-cell-a-distinct-style")
        .expect("§14's twelve are the wired list");
    let mut h = staged_at(&mut *scene, crate::caps::ColorDepth::Ansi16);
    scene.step(&mut h.screen, 1);
    let before = h.screen.narrowings();
    h.present();
    let narrowings = h.screen.narrowings() - before;
    assert_eq!(
        narrowings, CELLS,
        "{narrowings} narrowings for {CELLS} distinct style words"
    );
}
