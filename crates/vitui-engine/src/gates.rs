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
use crate::clock::Wake;
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

            let frame = h.screen.frame();
            let size = h.screen.size();
            let runs = h.screen.runs();
            h.screen
                .with_packet(|packet| {
                    assert_eq!(
                        packet.size(),
                        size,
                        "{name}, frame {t}: the packet carries the wrong size"
                    );
                    let cells = packet.cells();
                    let mut at = 0;
                    for r in packet.runs() {
                        for x in r.lo..=r.hi {
                            assert_eq!(
                                cells[at],
                                frame.row(r.y)[x as usize],
                                "{name}, frame {t}: the packed cell at ({x}, {}) is not the surface \
                                 cell",
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
                        runs,
                        "{name}, frame {t}: the packet's runs are not the frame's"
                    );
                })
                .expect("the frame was packed on this thread");
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

/// **A cell-less frame is still a frame the flag can land on, and it must not swallow it.**
///
/// The defect this is about was real for one commit and would have been invisible to every other gate
/// here. `repaint` is taken out of the `Screen`'s latch at pack time, so a packet carrying it has
/// spent it — and impl 21 made a packet with **no cells** reachable for the first time: a caret that
/// moved with nothing else changing. The serializer read the flag after its cell-less early return,
/// so that frame dropped it, and the mirror went on trusting handles a sweep had already moved.
///
/// Every existing gate misses it because before impl 21 an empty packet could not be produced: an
/// idle `present` returns before it leases one.
#[test]
fn a_renumbering_sweep_reaches_the_mirror_through_a_frame_with_no_cells() {
    const ROWS: u16 = 4;
    let mut h = extended_screen(ROWS);
    let id = extended_page(&mut h.screen, ROWS);
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
                ul: Some(Color::rgb(9, 9, 9)),
                ..Default::default()
            },
        );
    }
    h.present();
    assert_eq!(h.screen.known_rows(), ROWS as usize);
    assert!(h.screen.sweep_now().renumbered);

    // Nothing is damaged. The caret moved, so a frame goes out — and it is a frame with no cells in
    // it at all, which is the one the flag now has to survive.
    //
    // Presented through the `Screen` rather than the harness on purpose: the round trip catches this
    // too, one frame later and from four levels down a stack, and *the mirror was never told* is the
    // thing that went wrong. The round trip still gets its say at the bottom.
    h.screen.set_cursor(Some(crate::actuate::Cursor {
        x: 2,
        y: 1,
        shape: crate::actuate::CursorShape::Terminal,
    }));
    assert!(h.screen.present().submitted);
    assert_eq!(
        h.screen.known_rows(),
        0,
        "the flag was spent on a cell-less frame and the mirror was never told"
    );

    // And the screen the terminal is left showing is still the frame's, which is the assertion the
    // flag exists for rather than a restatement of the one above.
    underline_each_row(&mut h.screen, id, ROWS, |_| Color::rgb(4, 5, 6));
    assert!(h.present().submitted);
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

// ---------------------------------------------------------------------------------------------
// #8, #9 — the pool does not starve, and a packet is never superseded.
// ---------------------------------------------------------------------------------------------

/// A sink that takes everything and keeps a count.
///
/// A [`Recorder`](crate::testing::Recorder) would grow a `Vec` by ten thousand frames, and a gate
/// whose window contains that is measuring its own instrument.
#[derive(Clone, Default)]
struct Counting(std::sync::Arc<std::sync::atomic::AtomicUsize>);

impl std::io::Write for Counting {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0
            .fetch_add(buf.len(), std::sync::atomic::Ordering::Relaxed);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Counting {
    fn bytes(&self) -> usize {
        self.0.load(std::sync::atomic::Ordering::Relaxed)
    }
}

/// A screen on the **real three-thread path**: `Clock::System`, so `attach` spawns a render thread
/// that owns the write direction and the mirror.
///
/// Everything else in this file is deterministic on purpose. These three gates cannot be: they are
/// about the handoff, and §14's own note says so — *the properties that are about threads cannot be
/// tested in the mode that removes them.*
fn threaded(sink: Box<dyn std::io::Write + Send>) -> Screen {
    threaded_at(f32::INFINITY, sink).0
}

/// The same, with a ceiling, and keeping the [`crate::WakeHandle`] the pacing gates need.
///
/// `threaded` throws the handle away and pins the rate at unlimited, because every gate that came
/// before ticket 19 is about the handoff and would otherwise be measuring a gap it never reads. The
/// clock gates are the ones that want both.
fn threaded_at(
    hz: f32,
    sink: Box<dyn std::io::Write + Send>,
) -> (Screen, crate::engine::WakeHandle) {
    crate::engine::Engine::new(crate::engine::Config {
        size: (W, H),
        output: crate::engine::Output::Sink(sink),
        clock: crate::engine::Clock::System,
        max_frame_rate: hz,
        overrides: crate::testing::pinned_truecolor(),
        overrun_threshold: None,
        overrun_report: None,
        input: crate::input::InputConfig::default(),
    })
    .attach()
    .expect("attaching to a sink cannot fail")
}

/// Wait until the render thread has finished `want` frames, or give up and let the caller's
/// assertion say what was short.
///
/// A spin rather than a condvar, because `finish` signals nothing: the app thread is released at
/// *take*, which is what makes the pool two, and nothing in the design waits for a write to end.
/// Adding a notification for a test's benefit would put a wakeup in the frame path that ticket 19's
/// idle gate would then have to explain.
fn drained(screen: &Screen, want: u64) -> (u32, u32, u64) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    loop {
        let counts = screen.handoff_counts();
        if counts.2 >= want || std::time::Instant::now() > deadline {
            return counts;
        }
        std::hint::spin_loop();
    }
}

/// **Gates #9 and #8, and #9 is the backpressure decision rather than a health check.**
///
/// > With one producer and the ready gate, the slot is always empty at submit, so a packet can never
/// > be superseded — which fixes the pool at two packets, provably rather than empirically.
///
/// If the superseded counter ever leaves zero, the pacing gate has moved off the app thread and
/// frames are being composed in order to be thrown away. That is not a leak and not a crash: the
/// screen stays correct and the engine quietly does 107 µs of work per frame for nothing, which is
/// exactly the class of defect a count catches and a stopwatch does not.
///
/// Ten thousand cycles, on one cell a frame, because what is being counted is the handoff and not
/// the composite: a full-screen frame would spend a minute of CI to exercise the same four lock
/// acquisitions ten thousand times.
#[test]
fn a_packet_is_never_superseded_and_the_pool_of_two_does_not_starve() {
    const CYCLES: u64 = 10_000;
    let sink = Counting::default();
    let mut screen = threaded(Box::new(sink.clone()));
    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);

    let mut submitted = 0u64;
    for i in 0..CYCLES {
        {
            let mut view = screen.layers().view(id).expect("the layer was just added");
            // Alternating, so the equality filter has something to emit and the renderer has
            // something to write. A steady glyph would make every frame after the first zero bytes,
            // and then the pool would be cycling with no write to overlap.
            view.text(
                0,
                (i % u64::from(H)) as i32,
                if i % 2 == 0 { "x" } else { "y" },
                Style::new(),
            );
        }
        loop {
            if screen.present().submitted {
                submitted += 1;
                break;
            }
            // The only reason a frame with damage in it does not submit: the renderer has not taken
            // the last one. This is where ticket 19's `wait` goes.
            screen.wait_for_renderer();
        }
    }
    assert_eq!(submitted, CYCLES, "every frame was eventually handed on");

    let (superseded, starved, painted) = drained(&screen, CYCLES);
    assert_eq!(
        superseded, 0,
        "a packet was superseded, so frames are being composed to be thrown away"
    );
    assert_eq!(
        starved,
        0,
        "the pool of {} starved, which the ready gate is supposed to make unreachable",
        crate::handoff::POOL
    );
    assert_eq!(painted, CYCLES, "the render thread wrote every frame");
    assert!(sink.bytes() > 0, "nothing reached the wire at all");
}

/// **The threaded path writes the byte the deterministic path writes, and this is what says so.**
///
/// Ticket 18's own constraint is that nothing it adds may change a byte the deterministic mode
/// asserts, and the twelve scenes are what assert those bytes. So the same scene is driven through
/// both paths and the two recordings are compared whole — prologue, frames and all.
///
/// It is exact rather than approximate because the two paths share the serializer, the mirror and
/// the packet pool: what differs is which thread runs the renderer. A frame is waited for before the
/// next one is drawn, so no frame coalesces into another — coalescing is a different property and
/// [`a_busy_renderer_coalesces_rather_than_dropping_a_frame`] is where it is checked.
#[test]
fn the_threaded_path_writes_the_bytes_the_deterministic_path_writes() {
    for mut scene in wired() {
        let name = scene.name();

        let inline = {
            let mut h = Harness::with_overrides(W, H, scene.overrides());
            scene.build(&mut h.screen);
            h.present();
            for t in 1..=FRAMES {
                scene.step(&mut h.screen, t);
                h.present();
            }
            h.bytes()
        };

        let recorder = crate::testing::Recorder::new();
        let recording = recorder.handle();
        let threaded = {
            let (mut screen, _wake) = crate::engine::Engine::new(crate::engine::Config {
                size: (W, H),
                output: crate::engine::Output::Sink(Box::new(recorder)),
                clock: crate::engine::Clock::System,
                max_frame_rate: f32::INFINITY,
                overrides: scene.overrides(),
                overrun_threshold: None,
                overrun_report: None,
                input: crate::input::InputConfig::default(),
            })
            .attach()
            .expect("attaching to a sink cannot fail");
            scene.build(&mut screen);
            /// One frame, waited for: the next one is not drawn until the renderer has taken this
            /// one, so nothing coalesces and the byte streams stay comparable frame for frame.
            fn present(screen: &mut Screen) {
                while !screen.present().submitted {
                    screen.wait_for_renderer();
                }
            }
            let mut frames = 0;
            present(&mut screen);
            frames += 1;
            for t in 1..=FRAMES {
                scene.step(&mut screen, t);
                present(&mut screen);
                frames += 1;
            }
            let (_, _, painted) = drained(&screen, frames);
            assert_eq!(painted, frames, "{name}: the render thread fell behind");
            // Read before the `Screen` is dropped: the epilogue goes out on the way past, and the
            // inline arm's is not in its recording either.
            recording
                .lock()
                .expect("the recorder is never poisoned")
                .bytes
                .clone()
        };

        assert_eq!(
            threaded, inline,
            "{name}: the render thread wrote different bytes from the inline round"
        );
    }
}

/// **The backpressure decision, from the other side: a frame that would have been dropped is never
/// composed.**
///
/// The renderer is held inside a write by a sink that blocks on a channel, so *the render thread is
/// busy* is a state this test puts it in rather than one it waits to observe. What is then asserted
/// is the whole of §7's answer to backpressure:
///
/// - `present` returns without compositing, and says `submitted: false`;
/// - the damage is **still there** afterwards, which is what makes coalescing free — §6's structure
///   is already the coalescing mechanism and costs 6.6 ns to interrogate;
/// - the frame that eventually submits reports how many were folded into it, and clears the count.
///
/// The pool is what makes the second frame submit at all: the renderer holds one packet while the
/// app fills the other, which is exactly *one being filled, one in the renderer's hands*.
#[test]
fn a_busy_renderer_coalesces_rather_than_dropping_a_frame() {
    /// A sink that will not take a frame until the test lets it.
    ///
    /// One `recv` per write, and a dropped sender is what releases it for ever — which is how the
    /// epilogue gets out at the end without the test having to count the writes it did not make.
    struct Gated(std::sync::mpsc::Receiver<()>);

    impl std::io::Write for Gated {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let _ = self.0.recv();
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let (tx, rx) = std::sync::mpsc::channel();
    // The prologue, which `attach` writes on this thread before any second thread exists. Gating it
    // would deadlock the constructor.
    tx.send(()).expect("the receiver is alive");
    let mut screen = threaded(Box::new(Gated(rx)));
    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
    let row: String = std::iter::repeat_n('m', W as usize).collect();
    let paint = |screen: &mut Screen, y: i32, style: Style| {
        let mut view = screen.layers().view(id).expect("the layer was just added");
        view.text(0, y, &row, style);
    };

    // Frame one goes in and the renderer takes it: `wait_for_renderer` is what makes that an
    // ordering rather than a hope. It is now blocked inside the write.
    paint(&mut screen, 0, Style::new());
    assert!(screen.present().submitted);
    screen.wait_for_renderer();

    // Frame two fills the second packet of the pool and lands in the slot. The renderer is still
    // inside frame one's write, so it has not taken it.
    paint(&mut screen, 1, Style::new());
    let second = screen.present();
    assert!(
        second.submitted,
        "the pool's second packet was there to fill"
    );
    assert_eq!(second.coalesced, 0);

    // Frames three and four are not composited at all.
    for expected in 1..=2 {
        paint(&mut screen, 2, Style::new().bold());
        let folded = screen.present();
        assert!(
            !folded.submitted,
            "a frame was composed for a renderer that had not asked for one"
        );
        assert_eq!(folded.coalesced, expected);
        assert!(
            !screen.frame().damage().is_empty(),
            "the folded frame's damage was thrown away rather than accumulated"
        );
    }

    // Let both writes through, and the frame that follows carries the two that were folded.
    tx.send(()).expect("the receiver is alive");
    tx.send(()).expect("the receiver is alive");
    drained(&screen, 2);
    screen.wait_for_renderer();
    let caught_up = screen.present();
    assert!(caught_up.submitted);
    assert_eq!(
        caught_up.coalesced, 2,
        "the frame that paints reports what was folded into it"
    );
    assert_eq!(
        screen.present().coalesced,
        0,
        "the count is cleared by the frame that carried it"
    );
    drop(tx);
}

// ---------------------------------------------------------------------------------------------
// Resize: sampled at frame start, re-checked at submit.
// ---------------------------------------------------------------------------------------------

/// **A lease is never invalidated; the frame it produced may be discarded** (spec §2's sixth
/// invariant).
///
/// Writing a 300x80 frame into a terminal that is now 120x40 wraps and scrolls, which is worse than
/// a missing frame — so the frame goes back to the pool unsent. Four things are asserted, and the
/// last two are what make it a discard rather than a loss:
///
/// - `Presented::discarded_for_resize` says so;
/// - not one byte went out;
/// - the damage is still marked, so the next frame composites it again;
/// - a full repaint is scheduled, because the mirror is describing a terminal that has reflowed.
///
/// And then it happens **once**: the next frame samples the size the last one was refused for, so a
/// terminal that resizes and stays resized costs one composite rather than every composite after it.
#[test]
fn a_frame_whose_size_moved_under_it_is_discarded_and_repaints() {
    let mut h = Harness::truecolor(W, H);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, W, H), true);
    let row: String = std::iter::repeat_n('m', W as usize).collect();
    {
        let mut view = h.screen.layers().view(id).expect("just added");
        view.text(0, 0, &row, Style::new());
    }
    h.present();

    let before = h.bytes_written();
    {
        let mut view = h.screen.layers().view(id).expect("still there");
        view.text(0, 1, &row, Style::new().bold());
    }
    h.screen.resize_during_next_frame(120, 40);
    let discarded = h.screen.present();

    assert!(
        discarded.discarded_for_resize,
        "the frame was written into a terminal of a different size"
    );
    assert!(!discarded.submitted);
    assert_eq!(
        h.bytes_written(),
        before,
        "a frame composited for a screen that no longer exists reached the wire"
    );
    assert!(
        !h.screen.frame().damage().is_empty(),
        "the discarded frame's damage went with it"
    );
    assert!(
        h.screen.repaint_pending(),
        "the mirror was left describing a terminal that has reflowed"
    );
    assert_eq!(h.screen.terminal_size(), (120, 40));

    // **And it keeps refusing until the application catches up**, which is a correction impl 20
    // made and could not have made before it: the surfaces are still 80x24 and the terminal is
    // 120x40, so the next composite is a frame for a screen that does not exist either. The gate
    // used to assert that this one submitted — right when nothing delivered a resize *event*, so
    // nothing would ever have moved the surfaces and an engine that kept refusing would go blank
    // for good. `Screen::next_event` is what moves them now, and an application gets one call to
    // do it.
    let still = h.screen.present();
    assert!(
        !still.submitted,
        "an 80x24 composite for a 120x40 terminal reached the wire"
    );
    assert!(still.discarded_for_resize);
    assert_eq!(h.bytes_written(), before);

    // The application drains the resize — which is what the harness's own `resize` stands in for,
    // surfaces and terminal model together — and the frame after that is an ordinary frame.
    h.resize(120, 40);
    let next = h.screen.present();
    assert!(
        next.submitted,
        "the frame after the application caught up is not discarded"
    );
    assert!(!next.discarded_for_resize);
    assert!(h.bytes_written() > before);
}

/// **A resize observed between two frames is refused before the composite, not after it.**
///
/// The other half of the gate above, and the one the review found missing. `present` samples the
/// authoritative size at frame start and re-checks it at submit, which catches a resize that lands
/// *inside* the frame and cannot catch one that landed before it — the sample already equals the
/// stored size and the re-check agrees with itself. Before impl 20 nothing wrote that size in a
/// release build, so the hole was unreachable; the input thread writes it on every read now, and an
/// application that calls `present` without draining its events first walks straight into it.
///
/// The concrete failure is a shrink: 80x24 stored, the terminal becomes 40x10, and an 80-column
/// frame written into a 40-column terminal wraps every row and scrolls the screen.
#[test]
fn a_resize_the_application_has_not_drained_is_refused_before_the_composite() {
    let mut h = Harness::truecolor(W, H);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, W, H), true);
    let row: String = std::iter::repeat_n('m', W as usize).collect();
    {
        let mut view = h.screen.layers().view(id).expect("just added");
        view.text(0, 0, &row, Style::new());
    }
    h.present();

    // The input thread's whole write, without the app thread hearing about it yet.
    let before = h.bytes_written();
    h.screen.observe_terminal_size(40, 10);
    h.screen.inject(crate::input::Event::Resize(40, 10));
    {
        let mut view = h.screen.layers().view(id).expect("still there");
        view.text(0, 1, &row, Style::new().bold());
    }

    let refused = h.screen.present();
    assert!(
        !refused.submitted,
        "a 300-column frame reached a 40-column terminal"
    );
    assert!(refused.discarded_for_resize);
    assert_eq!(h.bytes_written(), before);

    // **The damage is still in the layer stack**, because the refusal is before `take_damage_into`:
    // the frame that does run sees exactly what this one would have. `next_event` is the application
    // catching up, and `Harness::resize` is that plus the terminal model the round trip replays
    // through — the size it is given is the one already stored, so it moves only what is behind.
    assert_eq!(
        h.screen.next_event(),
        Some(crate::input::Event::Resize(40, 10))
    );
    h.resize(40, 10);
    let next = h.screen.present();
    assert!(next.submitted, "the frame the application caught up for");
    assert!(h.bytes_written() > before);
}

/// The generation is what the three side tables are stamped with, so a repeated one would make last
/// frame's clusters answer for this frame's handles.
///
/// It counts **packs** and not submissions, which is the whole of why a discarded frame consumes
/// one: the packet it was packed into goes back to the pool with its marks still stamped.
#[test]
fn every_pack_stamps_a_generation_of_its_own() {
    let mut h = Harness::truecolor(8, 2);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 8, 2), true);
    let mut seen = Vec::new();
    for t in 0..6u32 {
        {
            let mut view = h.screen.layers().view(id).expect("just added");
            view.text(0, 0, if t % 2 == 0 { "ab" } else { "cd" }, Style::new());
        }
        if t == 3 {
            h.screen.resize_during_next_frame(4, 2);
            assert!(h.screen.present().discarded_for_resize);
            h.screen.resize(4, 2);
            continue;
        }
        h.present();
        seen.push(
            h.screen
                .with_packet(|p| p.generation())
                .expect("the frame was packed on this thread"),
        );
    }
    let mut sorted = seen.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(
        sorted.len(),
        seen.len(),
        "a generation was reused: {seen:?}"
    );
    assert!(
        seen.windows(2).all(|w| w[0] < w[1]),
        "generations must increase, not merely differ: {seen:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// Reports: what the handoff costs, what `pack` costs, and the walk the packet was chosen for.
// ---------------------------------------------------------------------------------------------

/// Report: the mailbox's critical section, against §7's **47 ns** on submit and **80 ns** for a full
/// uncontended `lease → submit → take → finish`.
///
/// # Why the submit figure is a difference rather than a direct reading
///
/// A 47 ns critical section cannot be bracketed by two `Instant::now()` calls: on this machine the
/// pair costs about as much as the thing between them, so the number would be mostly clock. What is
/// measured instead is two cycles that differ by exactly two lock acquisitions — the full round
/// against `lease → give_back`, which is the discard path and takes two — and the difference divided
/// by two is what one acquisition of the mailbox costs with its critical section inside it.
///
/// Round-robin through [`Bench`](vitui_bench::Bench), because the whole value here is a difference
/// between two arms and a background job that lands inside one of them would be the difference.
#[test]
fn what_the_handoff_costs() {
    use crate::handoff::{Lease, Mailbox};

    const ITERS: u32 = 5_000;
    let full = Mailbox::new();
    let half = Mailbox::new();
    let report = vitui_bench::Bench::new(24)
        .case("lease-submit-take-finish", ITERS, || {
            let Lease::Ready(packet) = full.lease() else {
                unreachable!("the renderer is this thread and it is never busy")
            };
            full.submit(packet);
            let packet = full.take().expect("the slot was just filled");
            full.finish(packet);
        })
        .case("lease-give-back", ITERS, || {
            let Lease::Ready(packet) = half.lease() else {
                unreachable!("nothing else holds a packet")
            };
            half.give_back(packet);
        })
        .run();

    let cycle = report
        .get("lease-submit-take-finish")
        .expect("the case ran");
    let discard = report.get("lease-give-back").expect("the case ran");
    println!(
        "\n  the mailbox, uncontended, {ITERS} iterations a sample:\n    \
         {cycle:.1} ns  lease -> submit -> take -> finish, four acquisitions (spec §7: 80 ns)\n    \
         {discard:.1} ns  lease -> give_back, two acquisitions (the resize discard path)\n    \
         {:.1} ns  one acquisition with its critical section (spec §7: 47 ns on submit)\n  \
         Report, not a gate: §14 allows a timing as a gate only at cliff granularity, and this is a \
         difference of two nanosecond-scale arms.",
        (cycle - discard) / 2.0,
    );
    assert!(
        cycle > discard,
        "four lock acquisitions cost less than two, so this is measuring the clock"
    );
}

/// The four densities of §7's `pack` table, built through the drawing verbs.
///
/// The names are §7's own. What varies is how many of the 24 000 cells carry a handle the packet has
/// to resolve into a side table — none, one in a hundred, all of them naming one entry, and all of
/// them naming a distinct entry, which is the adversarial page.
///
/// **Two passes, and the second one is what is measured over.** The first mints every handle so the
/// tables are warm, and the second changes every cell so the whole screen is damaged with the
/// density's handles still on it. One pass would have measured a `pack` over whatever the birth frame
/// happened to leave damaged, and a density applied *before* a full-screen `text` would have measured
/// a plain screen four times — `text` writes the style word it is given, so it takes an extended cell
/// back to inline.
fn packing_density(which: &str) -> Screen {
    let mut h = Harness::with_overrides(
        W,
        H,
        Overrides {
            hyperlinks: Some(true),
            ..crate::testing::pinned_truecolor()
        },
    );
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, W, H), true);
    let row: String = std::iter::repeat_n('m', W as usize).collect();
    let link = if which == "linked 100%" {
        Some(h.screen.link("https://example.com/vitui"))
    } else {
        None
    };
    for pass in 0..2u32 {
        // A different ink each pass, so every one of the 24 000 cells changes and the second pass
        // damages the whole screen.
        let ink = if pass == 0 {
            opaque_ink()
        } else {
            opaque_ink().bold()
        };
        let mut view = h.screen.layers().view(id).expect("just added");
        for y in 0..H as i32 {
            view.text(0, y, &row, ink);
        }
        match which {
            "plain" => {}
            // One cell in a hundred carries a cluster and an underline colour: §7's *realistic*.
            "realistic 1%" => {
                for y in (0..H as i32).step_by(4) {
                    for x in (0..W as i32).step_by(25) {
                        view.text(x, y, "e\u{301}", ink);
                        view.restyle(
                            Rect::new(x, y, 1, 1),
                            &Restyle {
                                ul: Some(Color::rgb(1, 2, 3)),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
            // Every cell hyperlinked, and one table entry between them.
            "linked 100%" => view.restyle(
                Rect::new(0, 0, W, H),
                &Restyle {
                    link,
                    ..Default::default()
                },
            ),
            // Every cell a distinct extended style, which is a table of thousands and a probe per
            // cell that cannot be answered from the one before it.
            "hostile 100%" => {
                for y in 0..H {
                    for x in 0..W {
                        let n = u32::from(y) * u32::from(W) + u32::from(x);
                        view.restyle(
                            Rect::new(x as i32, y as i32, 1, 1),
                            &Restyle {
                                ul: Some(Color::rgb((n >> 16) as u8, (n >> 8) as u8, n as u8)),
                                ..Default::default()
                            },
                        );
                    }
                }
            }
            other => unreachable!("no such density: {other}"),
        }
        h.present();
    }
    h.screen
}

/// Report: what `pack` costs on the app thread at §7's four densities.
///
/// > | pack, 300x80, on the app thread | plain | realistic 1% | linked 100% | hostile 100% |
/// > |---|---|---|---|---|
/// > | position-rewriting | 50.00 µs | 51.05 µs | 49.95 µs | 98.99 µs |
/// > | **keyed** | **21.99 µs** | **28.13 µs** | **31.02 µs** | **38.42 µs** |
///
/// Only the keyed row is reproducible here, because the position-rewriting packet is the one that
/// was refused and there is nothing left of it to run — its cost is recorded above and its *defect*
/// is what gate #10 keeps out. What the four numbers below are for is the shape: `pack` is
/// proportional to the damage and to the handles inside it, and the adversarial page is under twice
/// the plain one rather than an order of magnitude over it.
///
/// **This is the app thread's frame budget**, which is the scarce one — the render thread has 16.6 ms
/// and nothing else to do — so a change that made the render thread cheaper by making this dearer
/// would be a regression this report is where anyone would see.
///
/// §7's figures are from a release build, so read this one from a release build too:
///
/// ```text
/// cargo test --release -p vitui-engine what_pack_costs -- --nocapture
/// ```
#[test]
fn what_pack_costs_at_four_densities() {
    const DENSITIES: [&str; 4] = ["plain", "realistic 1%", "linked 100%", "hostile 100%"];

    let mut out = String::new();
    for density in DENSITIES {
        let screen = packing_density(density);
        let mut packet = crate::packet::Packet::new();
        let mut generation = 0u64;
        // A pack of the whole screen per iteration, and the tables are warm: every handle the cells
        // name already exists, which is the state register entry #6 is about.
        let mut once = || {
            generation += 1;
            packet.pack_cells(
                screen.runs(),
                screen.frame(),
                screen.tables(),
                false,
                generation,
            );
            std::hint::black_box(packet.cells().len());
        };
        // Warm the marker vectors to the tables' high-water mark before timing, because their one
        // growth is the allocation the density gate is about and it happens once.
        once();
        let report = vitui_bench::Bench::new(16).case(density, 4, once).run();
        let us = report.get(density).expect("the case ran") / 1e3;
        out.push_str(&format!("    {us:>7.2} us  {density}\n"));
    }
    println!(
        "\n  pack, {W}x{H}, whole screen damaged, on the app thread:\n{out}  \
              spec §7's keyed row: 21.99 / 28.13 / 31.02 / 38.42 us.\n  Report, not a gate."
    );
}

/// Report: the walk the packet was chosen for, contiguous against indexed by runs.
///
/// > | scene | packet (contiguous) | grid indexed by runs | x |
/// > |---|---|---|---|
/// > | **sparse chart (584 runs)** | **179.6 ns** | **875.7 ns** | **4.88** |
///
/// This is what decided §7, because *cost did not discriminate* on the copy: the buffer-age area
/// multiplier of the ownership-transfer alternative is real but its absolute worst case is 0.4 µs
/// against a 100 µs budget. What separates the two is the **serializer's** walk afterwards, and on
/// scattered runs the packet walks its cells contiguously where a grid jumps a row stride per run
/// across 384 KB.
///
/// Both arms read the same cells in the same order and do the same arithmetic on them; the only
/// difference is where those cells live. Round-robin, because the ratio is the whole point and it is
/// only meaningful if both arms saw the same interference.
#[test]
fn what_the_serializers_walk_costs_contiguous_against_indexed() {
    let mut rows = String::new();
    for mut scene in wired() {
        let name = scene.name();
        let mut h = staged(&mut *scene);
        scene.step(&mut h.screen, 1);
        h.present();
        let screen = &h.screen;
        let runs = screen.runs();
        let frame = screen.frame();
        let cells: Vec<crate::cell::Cell> = screen
            .with_packet(|p| p.cells().to_vec())
            .expect("the frame was packed on this thread");
        assert!(!cells.is_empty(), "{name}: nothing was packed to walk");

        let report = vitui_bench::Bench::new(24)
            .case("contiguous", 64, || {
                let mut acc = 0u32;
                for c in &cells {
                    acc ^= c.grapheme.bits();
                }
                std::hint::black_box(acc);
            })
            .case("indexed", 64, || {
                let mut acc = 0u32;
                for r in runs {
                    for c in &frame.row(r.y)[r.lo as usize..=r.hi as usize] {
                        acc ^= c.grapheme.bits();
                    }
                }
                std::hint::black_box(acc);
            })
            .run();
        let contiguous = report.get("contiguous").expect("ran");
        let indexed = report.get("indexed").expect("ran");
        rows.push_str(&format!(
            "    {name:<44} {contiguous:>9.1} ns {indexed:>9.1} ns  {:>5.2}x  ({} runs, {} cells)\n",
            indexed / contiguous,
            runs.len(),
            cells.len(),
        ));
    }
    println!(
        "\n  the serializer's walk, per scene: packet (contiguous) against grid indexed by runs\n\
         {rows}  spec §7: 4.88x on the sparse chart, 0.99x-1.12x on the contiguous scenes.\n  \
         Report, not a gate."
    );
}

/// **A resize invalidates the mirror by the flag and never by the size delta.**
///
/// The review found this one and it is the position rule again, one level up: the render thread
/// rebuilds its serializer when a packet's size differs from the one it holds, and a size that leaves
/// and comes back is a size that never differs. 300x80 to 120x40 to 300x80, with both events handled
/// before the next frame, hands the renderer a packet of the size it already has — and the mirror
/// still describes a screen that has reflowed twice. The frame damages every cell, and then the
/// equality filter suppresses exactly the ones that match the pre-reflow mirror.
///
/// The flag is about the *event* and has no such hole, which is why it is what `resize` sets. Both
/// halves are asserted: the latch, and the byte count of the frame after a round trip through another
/// size.
#[test]
fn a_resize_invalidates_the_mirror_even_when_the_size_comes_back() {
    let mut h = Harness::truecolor(8, 2);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 8, 2), true);
    {
        let mut view = h.screen.layers().view(id).expect("just added");
        view.text(0, 0, "abcdefgh", Style::new());
    }
    h.present();
    assert!(
        !h.screen.repaint_pending(),
        "nothing has swept and nothing has resized"
    );

    // Away and back, so nothing about the *size* has changed by the time the next frame is packed.
    h.screen.resize(4, 2);
    h.screen.resize(8, 2);
    assert!(
        h.screen.repaint_pending(),
        "a resize that ends where it started still reflowed the terminal twice"
    );

    // And the frame that follows writes the whole screen rather than filtering against a mirror
    // that describes what the terminal showed before either resize.
    let before = h.bytes_written();
    {
        let mut view = h.screen.layers().view(id).expect("still there");
        view.text(0, 0, "abcdefgh", Style::new());
    }
    h.screen.present();
    assert!(
        h.bytes_written() > before,
        "the identical content was filtered against a mirror the terminal has left behind"
    );
}

/// **The app thread reads the authoritative size and never writes it.**
///
/// `resize` used to write it, on the reasoning that the app was agreeing with what the input thread
/// had already recorded. The review found the case where that is false: a second resize can land while
/// the application is still draining the first event, and then the write puts the *older* size back —
/// so the next frame samples it, agrees with itself at submit, and writes a 120x40 frame into an 80x24
/// terminal. That is the wrap-and-scroll §2's sixth invariant exists to prevent, with the evidence
/// erased by the thread that was supposed to read it.
#[test]
fn resizing_the_surfaces_does_not_write_the_authoritative_size() {
    let mut h = Harness::truecolor(W, H);
    assert_eq!(h.screen.terminal_size(), (W, H));
    h.screen.resize(120, 40);
    assert_eq!(
        h.screen.terminal_size(),
        (W, H),
        "the app thread wrote a size only the input thread can know"
    );
    assert_eq!(h.screen.size(), (120, 40), "the surfaces did resize");
}

/// **A dead render thread must not park the app thread for ever.**
///
/// The deadlock the review found is exact: the renderer takes packet one and dies inside its write,
/// packet two is submitted into the slot behind it, and `ready` is now false with nobody left to
/// clear it. `wait_until_free` then blocks on a condvar nobody will signal — `quit` is set from
/// `Screen::drop`, and `drop` cannot run while the app thread is parked. **A hang is worse than a
/// failure.**
///
/// The escape is a `Drop` guard on the render thread, so an unwind takes the same path as a return.
/// What it does *not* do is make the renderer look free: that would let the app submit into a slot
/// nobody empties, which moves register entry #9's counter and destroys the meaning of the gate this
/// file opens with.
#[test]
fn a_render_thread_that_panics_releases_the_app_thread_rather_than_parking_it() {
    /// A sink that dies the first time it is asked to take a frame, once armed.
    struct Fatal(std::sync::Arc<std::sync::atomic::AtomicBool>);

    impl std::io::Write for Fatal {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            assert!(
                !self.0.load(std::sync::atomic::Ordering::Relaxed),
                "this sink is supposed to die"
            );
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let armed = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    // Armed after `attach`, because the prologue goes out on this thread before the render thread
    // exists and a sink that died there would take the constructor with it.
    let mut screen = threaded(Box::new(Fatal(std::sync::Arc::clone(&armed))));
    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
    let paint = |screen: &mut Screen, y: i32| {
        let mut view = screen.layers().view(id).expect("just added");
        view.text(0, y, "the last frame", Style::new());
    };

    // A backtrace in the middle of a passing suite reads like a failure, and this panic is the
    // subject rather than a surprise.
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    armed.store(true, std::sync::atomic::Ordering::Relaxed);

    // Frame one is taken — which is what frees the renderer — and then dies in the write.
    paint(&mut screen, 0);
    assert!(screen.present().submitted);
    screen.wait_for_renderer();

    // Frame two lands in the slot behind a renderer that is not coming back for it, so `ready` is
    // false and this is the wait that used to be for ever.
    paint(&mut screen, 1);
    assert!(screen.present().submitted, "the slot was empty");
    let at = std::time::Instant::now();
    screen.wait_for_renderer();
    std::panic::set_hook(hook);
    assert!(
        at.elapsed() < std::time::Duration::from_secs(5),
        "the app thread was parked on a condvar nobody was going to signal"
    );
    assert!(
        screen.renderer_is_gone(),
        "the wait returned, so either the renderer took the packet or it is gone — and it is gone"
    );
    assert_eq!(
        screen.handoff_counts().0,
        0,
        "a dead renderer must not look free, or every frame after is composed to be dropped"
    );
}

/// The packet accessor answers by **identity, and admits when there is nothing to answer with**.
///
/// Two defects, both found by the review and both the position rule one level down. The index this
/// held was invalidated by the next `lease`, because `finish` pushes at the tail and `lease` pops
/// from it; and index zero was *valid* before anything had ever been packed, so the accessor handed
/// back a packet nobody had filled and `Harness::present` read a `repaint` flag off it. A screen that
/// presents nothing is the case that reaches it, and there is one in `crate::roundtrip`.
///
/// What it answers with is the last packet **returned to the pool**, not the last one painted, and
/// that is forced rather than chosen: the pool reuses buffers, so a discarded frame packs over the
/// bytes of the painted one before it — in the same `Box`. There is no last-painted packet left to
/// hold on to, and a name claiming otherwise would describe a copy that does not exist.
#[test]
fn the_packet_accessor_answers_by_identity_or_not_at_all() {
    let mut h = Harness::truecolor(8, 2);
    assert_eq!(
        h.screen.with_packet(|p| p.generation()),
        None,
        "no frame has packed anything, and a packet nobody filled is not an answer"
    );

    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 8, 2), true);
    {
        let mut view = h.screen.layers().view(id).expect("just added");
        view.text(0, 0, "ab", Style::new());
    }
    h.present();
    assert_eq!(h.screen.with_packet(|p| p.generation()), Some(1));

    // A second frame leases from the tail of the free list, which is where the first one was pushed
    // back. An index recorded by `finish` was out of bounds by this point.
    {
        let mut view = h.screen.layers().view(id).expect("still there");
        view.text(0, 0, "cd", Style::new());
    }
    h.present();
    assert_eq!(
        h.screen.with_packet(|p| p.generation()),
        Some(2),
        "the accessor lost the packet the lease moved"
    );

    // And a discarded frame is what the pool most recently got back, because it packed over the same
    // buffer the painted frame used.
    {
        let mut view = h.screen.layers().view(id).expect("still there");
        view.text(0, 0, "efgh", Style::new());
    }
    h.screen.resize_during_next_frame(4, 2);
    assert!(h.screen.present().discarded_for_resize);
    assert_eq!(
        h.screen.with_packet(|p| p.generation()),
        Some(3),
        "a discarded frame still consumed a generation and still packed over the buffer"
    );
}

// ---------------------------------------------------------------------------------------------
// The frame clock, and an idle that is zero (ticket 19).
//
// The gate is on `wait` rather than on `present` — ADR 0004 — so every gate below drives `wait`
// and reads what it answered. Two of them are counts and the rest are reports, which is §14's own
// split: a wake-up latency is a distribution on a shared runner and cannot be a gate, while zero
// wakeups over an idle window is a number that does not move.
// ---------------------------------------------------------------------------------------------

/// **Register entry #17, as a count, and it is the standing requirement rather than a health check.**
///
/// Three threads exist and the app thread is the one that had no parking point until this ticket:
/// the render thread has parked on the mailbox since impl 18, and an idle process still turned in
/// whatever loop its caller wrote. So an idle measured before `wait` was a measurement of the caller.
///
/// Four numbers, and the second is the one that matters:
///
/// - the app thread entered its blocking wait **once**;
/// - it came back **zero** times before something actually happened — a 120 Hz ticker would put 18
///   in this number over the window below, and 3 600 over the thirty seconds the process-level half
///   of this entry runs for;
/// - the render thread's wait came back zero times;
/// - no frame was painted.
///
/// The window is short on purpose. This is the *count* half, and a count does not need thirty
/// seconds to be zero; `examples/idle.rs` under `/usr/bin/time` is the half that needs the wall
/// clock, because `0.00 user 0.00 sys` is below the tool's resolution at three seconds.
///
/// The counters are read from **another thread**, which is forced rather than chosen: the thread
/// under test is parked inside the wait being measured, and a thread cannot assert about a park it
/// is in. `Arc<WakeSource>` is `Send + Sync` and travels where `Screen` deliberately cannot.
#[test]
fn an_idle_application_parks_once_and_wakes_for_nothing() {
    let (mut screen, wake) = threaded_at(120.0, Box::new(Counting::default()));
    let source = wake.source();
    let quiet = std::time::Duration::from_millis(150);
    let observer = std::thread::spawn(move || {
        std::thread::sleep(quiet);
        let counts = source.park_counts();
        wake.quit();
        counts
    });

    let began = std::time::Instant::now();
    assert_eq!(screen.wait(), Wake::Quit);
    let (parks, wakeups) = observer.join().expect("the observer does not panic");

    assert!(
        began.elapsed() >= quiet,
        "the wait came back before anything had happened"
    );
    assert_eq!(parks, 1, "an indefinite park re-entered the wait");
    assert_eq!(
        wakeups, 0,
        "something woke the app thread while it was idle: a 120 Hz ticker would put 18 here"
    );
    assert_eq!(
        screen.render_wakeups(),
        0,
        "the render thread woke with nothing in the slot"
    );
    assert_eq!(
        screen.handoff_counts().2,
        0,
        "an idle application painted a frame"
    );
}

/// **Quit is checked before the clock, and the fixture is the one that found the defect.**
///
/// A 1 Hz ceiling and a frame just submitted, so the gap has 999 ms left in it. Checking the clock
/// first holds the shutdown for all of it — shutdown latency becoming a function of the refresh rate,
/// which is exactly backwards.
///
/// It is a **gate** rather than a report even though it reads a stopwatch, because the cliff is three
/// orders of magnitude wide: the bound below is 200 ms against a second, and no runner is slow enough
/// to make that ambiguous.
#[test]
fn quit_is_never_paced_by_the_frame_clock() {
    let (mut screen, wake) = threaded_at(1.0, Box::new(Counting::default()));
    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
    {
        let mut view = screen.layers().view(id).expect("the layer was just added");
        view.text(0, 0, "paced", Style::new());
    }
    assert!(screen.present().submitted);
    assert!(
        screen.next_frame_allowed().is_some(),
        "a 1 Hz ceiling did not arm the gap"
    );

    wake.quit();
    let began = std::time::Instant::now();
    assert_eq!(screen.wait(), Wake::Quit);
    assert!(
        began.elapsed() < std::time::Duration::from_millis(200),
        "shutdown waited out a second of frame gap"
    );
}

/// **A minimum gap, not a tick**, and both halves are asserted: the leading edge is immediate and
/// everything inside the gap is held to the end of it.
///
/// The first `wait` is the leading edge — no frame has gone out, so nothing is paced and a post
/// returns at once. The second is the gap: a post arriving immediately after a frame is held until
/// the instant the clock allows the next one.
#[test]
fn the_first_wake_after_a_quiet_period_is_immediate_and_the_next_one_is_held() {
    const HZ: f32 = 25.0;
    let (mut screen, wake) = threaded_at(HZ, Box::new(Counting::default()));

    wake.post();
    let began = std::time::Instant::now();
    assert_eq!(screen.wait(), Wake::Posted);
    assert!(
        began.elapsed() < std::time::Duration::from_millis(10),
        "the leading edge was paced: {:?}",
        began.elapsed()
    );

    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
    {
        let mut view = screen.layers().view(id).expect("the layer was just added");
        view.text(0, 0, "paced", Style::new());
    }
    assert!(screen.present().submitted);
    let allowed = screen
        .next_frame_allowed()
        .expect("a submitted frame arms the gap");

    wake.post();
    assert_eq!(screen.wait(), Wake::Posted);
    assert!(
        std::time::Instant::now() >= allowed,
        "a frame was released {:?} before the clock allowed one",
        allowed - std::time::Instant::now()
    );
}

/// A deadline is what ends an indefinite wait, and **the earliest one is the one that is kept**.
///
/// The later deadline is registered first so that keeping it would be the passing answer — a test
/// that registers them in the other order passes whether the comparison is there or not.
#[test]
fn the_earliest_deadline_ends_the_wait_and_a_later_one_does_not_replace_it() {
    let (mut screen, _wake) = threaded_at(f32::INFINITY, Box::new(Counting::default()));
    let began = std::time::Instant::now();
    screen.request_wake_at(began + std::time::Duration::from_secs(30));
    screen.request_wake_at(began + std::time::Duration::from_millis(30));
    assert_eq!(screen.wait(), Wake::Deadline);
    assert!(began.elapsed() >= std::time::Duration::from_millis(30));
    assert!(
        began.elapsed() < std::time::Duration::from_secs(5),
        "the later deadline replaced the earlier one"
    );
}

/// **The frame a busy renderer refused is owed, and `wait` is what pays it.**
///
/// This is the case with no `Wake` variant of its own, and the reason it cannot be ignored. A slow
/// link, and the user stops typing exactly while the renderer is inside a write: `present` refused,
/// the damage is still in §6's structure, and **nothing else is going to happen**. Without the owed
/// frame the app thread parks for ever and the last keystroke's echo never reaches the terminal —
/// unbounded, not merely late.
///
/// Three things are asserted, and the middle one is what makes it a debt rather than a poll:
///
/// - `present` refusing leaves a frame owed;
/// - the wait does **not** return while the renderer is still holding the packet, because a frame
///   released then is a frame `present` would only refuse again;
/// - when the renderer takes it, the wait returns and the frame that follows submits.
///
/// See `crate::clock` for why the answer is [`Wake::Deadline`] and not a fifth variant.
#[test]
fn a_frame_a_busy_renderer_refused_is_owed_and_the_wait_pays_it() {
    /// A sink that will not take a frame until the test lets it. One `recv` per write.
    struct Gated(std::sync::mpsc::Receiver<()>);

    impl std::io::Write for Gated {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let _ = self.0.recv();
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let (tx, rx) = std::sync::mpsc::channel();
    // The prologue, written by `attach` before any second thread exists.
    tx.send(()).expect("the receiver is alive");
    let (mut screen, _wake) = threaded_at(f32::INFINITY, Box::new(Gated(rx)));
    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
    let row: String = std::iter::repeat_n('m', W as usize).collect();
    let paint = |screen: &mut Screen, y: i32, style: Style| {
        let mut view = screen.layers().view(id).expect("the layer was just added");
        view.text(0, y, &row, style);
    };

    // Frame one is taken and the renderer is now blocked inside its write. Frame two fills the pool's
    // second packet and lands in the slot, which the renderer has not taken.
    paint(&mut screen, 0, Style::new());
    assert!(screen.present().submitted);
    screen.wait_for_renderer();
    paint(&mut screen, 1, Style::new());
    assert!(screen.present().submitted);

    // Frame three is not composited at all, and it is owed.
    paint(&mut screen, 2, Style::new().bold());
    let folded = screen.present();
    assert!(!folded.submitted);
    assert!(
        screen.owes_a_frame(),
        "a refused frame left nothing for `wait` to come back for"
    );

    // The wait does not return while the renderer holds the packet. Asserted as an absence, with a
    // deadline as the only other way out: a wait that answered `Deadline` because the owed frame was
    // released early would come back before the deadline and the elapsed time is what says which.
    let held = std::time::Duration::from_millis(80);
    let began = std::time::Instant::now();
    screen.request_wake_at(began + held);
    assert_eq!(screen.wait(), Wake::Deadline);
    assert!(
        began.elapsed() >= held,
        "the owed frame was released while the renderer still held the packet"
    );

    // Let the writes through. Now the take frees the renderer, and the owed frame is what the wait
    // comes back for — with no deadline registered, so nothing else could have released it.
    tx.send(()).expect("the receiver is alive");
    tx.send(()).expect("the receiver is alive");
    assert_eq!(screen.wait(), Wake::Deadline);
    assert!(!screen.owes_a_frame(), "the debt was reported twice");
    let caught_up = screen.present();
    assert!(
        caught_up.submitted,
        "the frame the renderer was too busy for never went out"
    );
    assert_eq!(caught_up.coalesced, 1);
    drop(tx);
}

/// A frame discarded because the terminal resized under it is owed exactly as a refused one is.
///
/// The renderer is free — the packet went back to the pool rather than into the slot — so this is the
/// arm of the debt that does not wait for anything, and the whole screen is marked damaged behind it.
#[test]
fn a_frame_discarded_for_a_resize_is_owed_too() {
    let mut h = Harness::truecolor(W, H);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, W, H), true);
    {
        let mut view = h
            .screen
            .layers()
            .view(id)
            .expect("the layer was just added");
        view.text(0, 0, "resized under", Style::new());
    }
    h.screen.resize_during_next_frame(W / 2, H / 2);
    assert!(h.screen.present().discarded_for_resize);
    assert!(
        h.screen.owes_a_frame(),
        "a discarded frame left nothing for `wait` to come back for"
    );
    assert_eq!(
        h.screen.wait(),
        Wake::Deadline,
        "the owed frame did not release the wait"
    );
}

/// **`Wake::Input` is a reason of its own, and it outranks a post.**
///
/// What is gated here is the **multiplexing** rather than the parser: an input event and a post are
/// two reasons, the wait reports them one at a time, and each is consumed by the return that carries
/// it. The parser and the thread that raises this in a release build are impl 20's and are gated
/// separately — `crate::reader::tests` for the raise, `crate::gates` above for the events.
///
/// It was filed before either existed, because the alternative was a `Wake` variant nothing had ever
/// produced, which is indistinguishable from one that was decided against.
#[test]
fn an_input_event_is_a_reason_of_its_own_and_is_reported_before_a_post() {
    let (mut screen, wake) = threaded_at(f32::INFINITY, Box::new(Counting::default()));
    wake.post();
    wake.input();
    assert_eq!(screen.wait(), Wake::Input);
    assert_eq!(screen.wait(), Wake::Posted);

    // And each is consumed: a third wait would park for ever, so the absence is asserted by giving
    // it something else to come back for and checking that it is what comes back.
    screen.request_wake_at(std::time::Instant::now());
    assert_eq!(screen.wait(), Wake::Deadline);
}

/// **Register entry #26, and it is a report because a scheduler latency is not ours to gate.**
///
/// App submit to the render thread holding the packet, with the app thread **genuinely parked** —
/// which is what impl 18 could not do and this ticket can. §7's figures, for comparison:
/// p50 4.58 µs · p90 6.96 · p99 16.4 · p99.9 30.3 · max 71.5 µs over 5 000 samples.
///
/// **This does not spend the 100 µs frame budget**, and the distinction is worth keeping rather than
/// letting a reader assume the worst: that budget is app-thread CPU work and the app thread has
/// already returned. This is wall-clock delay on the way to the wire, and it is the OS scheduler's
/// rather than ours. Against a 16.6 ms gap the median is a few hundredths of a percent of a frame.
///
/// The park is real and it is what makes the number the right one: each iteration submits, then
/// registers a short deadline and calls `wait`, so the render thread's take happens while this
/// thread is inside a condvar rather than inside a spin.
#[test]
fn what_a_wake_up_costs_with_the_app_thread_parked() {
    const SAMPLES: usize = 512;
    let (mut screen, _wake) = threaded_at(f32::INFINITY, Box::new(Counting::default()));
    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
    screen.reset_wake_latencies();

    for i in 0..SAMPLES {
        {
            let mut view = screen.layers().view(id).expect("the layer was just added");
            view.text(
                0,
                (i % usize::from(H)) as i32,
                if i % 2 == 0 { "x" } else { "y" },
                Style::new(),
            );
        }
        while !screen.present().submitted {
            screen.wait();
        }
        // Parked, for as long as it takes the render thread to be scheduled and take the packet. The
        // deadline is what gets this thread back out; the sample is stamped on the other one.
        screen.request_wake_at(std::time::Instant::now() + std::time::Duration::from_micros(300));
        screen.wait();
    }

    let mut samples = screen.wake_latencies();
    assert!(
        samples.len() >= SAMPLES / 2,
        "only {} of {SAMPLES} submits were observed by the render thread",
        samples.len()
    );
    samples.sort_unstable();
    let at = |q: f64| samples[((samples.len() - 1) as f64 * q) as usize];
    println!(
        "wake-up latency, app submit -> render holding the packet, app thread parked (n = {}): \
         p50 {:?} p90 {:?} p99 {:?} max {:?} \
         -- scheduler latency on the way to the wire, NOT frame budget",
        samples.len(),
        at(0.50),
        at(0.90),
        at(0.99),
        samples[samples.len() - 1],
    );
}

/// **Leading-edge latency after a quiet period**, which is the property the move to `wait` most
/// threatened, in **two arms** — and the second arm is a correction to how §7's figure reads.
///
/// §7 reports 250 ns p50 / 1.04 µs p99 / 16.75 µs max over 2 000 samples. That is the **unparked**
/// leading edge: a reason that was already pending when `wait` was called, which returns without
/// touching the condvar at all. It is the number that says *the gate itself is free*, and it is
/// reproduced here as the first arm.
///
/// It is **not** what a keystroke after a genuinely idle application costs. That one is a cross-thread
/// condvar wakeup and therefore a scheduler hop, and it lands at the same order as §7's own handoff
/// figure of 4.58 µs p50 — measured here as the second arm, with the app thread parked in the
/// indefinite wait the idle guarantee is about. **Neither number is wrong; they are two quantities**,
/// and printing only the first would advertise 250 ns for something that costs twenty times it.
///
/// Both are leading edges in the sense that matters: no frame is ever submitted here, so the clock
/// never arms and nothing is ever held to a gap.
#[test]
fn what_a_leading_edge_costs_after_a_quiet_period() {
    const SAMPLES: usize = 256;
    let (mut screen, wake) = threaded_at(120.0, Box::new(Counting::default()));

    // Arm one: the reason is already pending, so `wait` never parks. This is the gate's own cost.
    let mut unparked = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        wake.post();
        let began = std::time::Instant::now();
        assert_eq!(screen.wait(), Wake::Posted);
        unparked.push(began.elapsed());
    }
    unparked.sort_unstable();
    let pick = |s: &[std::time::Duration], q: f64| s[((s.len() - 1) as f64 * q) as usize];
    println!(
        "leading edge, nothing to park for -- the gate's own cost (n = {}): p50 {:?} p99 {:?} max {:?}",
        unparked.len(),
        pick(&unparked, 0.50),
        pick(&unparked, 0.99),
        unparked[unparked.len() - 1],
    );

    // **One post outstanding at a time, and it is a handshake rather than a sleep.**
    //
    // `POSTED` is one bit, so two posts inside one park fold into one return — and the first draft
    // relied on a 200 µs sleep to keep them apart. That is not a slow test, it is a **hanging** one:
    // the app thread only has to be delayed past 200 µs between a return and the next park, which is
    // inside one scheduler quantum and above this gate's own measured 46 µs max, and then there are
    // fewer returns than samples and the last `wait` parks for ever. A hang is worse than a failure,
    // and a hang in a report is fifteen minutes of CI saying nothing.
    //
    // So the app thread says *I am about to park* and the poster posts exactly once for it. The sleep
    // stays, because its job is the other half — letting the park actually happen, so the sample is a
    // cross-thread wakeup rather than arm one again — but nothing depends on it any more.
    let (ready_tx, ready_rx) = std::sync::mpsc::channel::<()>();
    let (stamp_tx, stamp_rx) = std::sync::mpsc::channel::<std::time::Instant>();
    let poster = std::thread::spawn(move || {
        while ready_rx.recv().is_ok() {
            std::thread::sleep(std::time::Duration::from_micros(200));
            // Stamped immediately before the post and sent immediately after it. The other order
            // puts a channel send inside the interval, which is what the first draft measured.
            let at = std::time::Instant::now();
            wake.post();
            let _ = stamp_tx.send(at);
        }
    });

    let mut samples = Vec::with_capacity(SAMPLES);
    for _ in 0..SAMPLES {
        ready_tx.send(()).expect("the poster is alive");
        assert_eq!(screen.wait(), Wake::Posted);
        let woke = std::time::Instant::now();
        let posted = stamp_rx.recv().expect("the poster sends after it posts");
        samples.push(woke.saturating_duration_since(posted));
    }
    // Dropping the sender is what ends the poster's loop, so it needs no count of its own and cannot
    // be one post out of step with this one.
    drop(ready_tx);
    poster.join().expect("the poster does not panic");

    samples.sort_unstable();
    println!(
        "leading edge, post -> wait returns with the app thread parked (n = {}): \
         p50 {:?} p99 {:?} max {:?} \
         -- a cross-thread condvar wakeup, so the same order as the 4.58 us handoff and NOT the \
         250 ns above. The tail is the handshake's: two threads ping-pong through channels to keep \
         one post outstanding, so the p99 is scheduler contention this arm creates and not something \
         a real post pays",
        samples.len(),
        pick(&samples, 0.50),
        pick(&samples, 0.99),
        samples[samples.len() - 1],
    );
}

/// **The ceiling is achieved from below**, and the number is here so nobody spends a day
/// rediscovering it.
///
/// `wait_timeout` overshoots: §7 measured a hold at 10.18 ms p50 against an 8.333 ms gap on macOS, so
/// a configured 120 Hz yields about 110 fps under load. Undershooting a ceiling is safe by
/// construction — a ceiling achieved from below is still a ceiling — so the only assertion is the one
/// that would catch the clock being absent altogether.
///
/// A storm rather than a sparse rate, because a sparse rate measures the storm and not the clock: at
/// 200 Hz of events, gating at `present` delivers 83.8 fps against 110.8 for the same ceiling, and
/// that comparison is ADR 0004's rather than this report's.
#[test]
fn the_achieved_rate_lands_under_the_configured_ceiling() {
    const HZ: f32 = 120.0;
    const WINDOW: std::time::Duration = std::time::Duration::from_millis(500);
    let (mut screen, wake) = threaded_at(HZ, Box::new(Counting::default()));
    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
    let storming = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));

    let flag = std::sync::Arc::clone(&storming);
    let storm = std::thread::spawn(move || {
        while flag.load(std::sync::atomic::Ordering::Relaxed) {
            wake.post();
            std::thread::sleep(std::time::Duration::from_micros(500));
        }
        // The app thread is inside the gap when the flag drops; one last post lets it out.
        wake.post();
    });

    let began = std::time::Instant::now();
    let mut frames = 0u32;
    let mut iterations = 0u32;
    while began.elapsed() < WINDOW {
        screen.wait();
        iterations += 1;
        {
            let mut view = screen.layers().view(id).expect("the layer was just added");
            view.text(
                0,
                (frames % u32::from(H)) as i32,
                if frames % 2 == 0 { "x" } else { "y" },
                Style::new(),
            );
        }
        if screen.present().submitted {
            frames += 1;
        }
    }
    let elapsed = began.elapsed();
    storming.store(false, std::sync::atomic::Ordering::Relaxed);
    storm.join().expect("the storm does not panic");

    let fps = f64::from(frames) / elapsed.as_secs_f64();
    println!(
        "achieved rate against a configured {HZ} Hz: {fps:.1} fps over {elapsed:?}, \
         {iterations} iterations for {frames} frames \
         -- wait_timeout overshoots (10.18 ms p50 against an 8.333 ms gap on macOS), so a ceiling is \
         approached from below"
    );
    assert!(
        fps <= f64::from(HZ) * 1.5,
        "{fps:.1} fps against a {HZ} Hz ceiling: the clock is not gating anything"
    );
    assert!(
        frames > 0,
        "the clock held every frame of the window, which is not a ceiling"
    );
}

/// **Nothing anywhere queries a display**, which is §12's refusal 10 and an absence rather than a
/// behaviour.
///
/// A tty cannot report a refresh rate, so `Config::max_frame_rate` is the application's to set and
/// `Screen::set_max_frame_rate` is how it changes one. An absence cannot be asserted by calling
/// anything, so this reads the crate's own source: the platform APIs a future contributor would reach
/// for are named, and none of them may appear.
///
/// It is a cheap gate and a real one. `deny.toml` already forbids the dependency that would make one
/// of these easy, and this catches the other route — a raw `extern "C"` block, or an `ioctl` bolted
/// onto `crate::detect` beside the one that legitimately asks for the size in cells.
#[test]
fn nothing_anywhere_queries_a_display_for_a_refresh_rate() {
    /// What a display query would have to name, whichever platform it was written for.
    const FORBIDDEN: &[&str] = &[
        "CGDisplay",
        "NSScreen",
        "CVDisplayLink",
        "XRRGetScreenInfo",
        "XRRGetScreenResources",
        "randr",
        "DwmGetCompositionTimingInfo",
        "EnumDisplaySettings",
        "GetDeviceCaps",
        "VREFRESH",
        "refresh_rate",
        "refreshRate",
    ];
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rust_files(&root, &mut files);
    assert!(
        files.len() > 20,
        "the source walk found {} files, so it is not walking the crate",
        files.len()
    );
    for path in files {
        // This file, and only this file: it is the one whose purpose is to name them, and it is
        // `cfg(test)` — nothing it says reaches a release binary. Excluded by name rather than by
        // excluding every `cfg(test)` module, because that list is a thing to forget to update and
        // this is one line that fails loudly if the file is renamed.
        if path.ends_with("gates.rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("the crate's own source is readable");
        for needle in FORBIDDEN {
            assert!(
                !text.contains(needle),
                "{} names `{needle}`: the engine never asks a display anything, and the frame rate \
                 is the application's to set (spec §12's refusal 10)",
                path.display()
            );
        }
    }
}

/// Every `.rs` file under `root`, recursively.
#[cfg(test)]
fn collect_rust_files(root: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let entries = std::fs::read_dir(root).expect("the crate's own source is readable");
    for entry in entries {
        let path = entry.expect("the crate's own source is readable").path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            out.push(path);
        }
    }
}

/// `set_max_frame_rate` moves the gap, and the deterministic clock refuses to be given one.
///
/// The second half is the one worth a gate: a `Clock::Manual` screen that could be paced would make
/// the deterministic mode reproducible in its interleaving and not in its timing, which is half of
/// what it is for.
#[test]
fn setting_the_frame_rate_moves_the_gap_and_never_paces_the_deterministic_clock() {
    let (mut screen, _wake) = threaded_at(1.0, Box::new(Counting::default()));
    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
    let paint = |screen: &mut Screen, y: i32| {
        let mut view = screen.layers().view(id).expect("the layer was just added");
        view.text(0, y, "hz", Style::new());
    };
    paint(&mut screen, 0);
    assert!(screen.present().submitted);
    let slow = screen
        .next_frame_allowed()
        .expect("a 1 Hz ceiling arms the gap");

    screen.set_max_frame_rate(f32::INFINITY);
    paint(&mut screen, 1);
    screen.wait_for_renderer();
    assert!(screen.present().submitted);
    assert_eq!(
        screen.next_frame_allowed(),
        None,
        "an unlimited rate left a gap armed"
    );
    assert!(
        slow > std::time::Instant::now(),
        "the 1 Hz gap was not a second"
    );

    let mut h = Harness::truecolor(8, 2);
    h.screen.set_max_frame_rate(1.0);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 8, 2), true);
    {
        let mut view = h.screen.layers().view(id).expect("just added");
        view.text(0, 0, "manual", Style::new());
    }
    assert!(h.screen.present().submitted);
    assert_eq!(
        h.screen.next_frame_allowed(),
        None,
        "the deterministic clock accepted a ceiling"
    );
}

/// **Register entry #13** — a presses-only terminal never yields `Release` or `Repeat`.
///
/// The absence is what is being asserted, so the gate has to be able to fail: the second half feeds
/// the *same* keys from a terminal that has kitty flag 2 and asserts that both kinds do arrive. A
/// gate that only ever asserts an absence passes identically against a parser that produces nothing
/// at all, and spec §14's rule about vacuous gates is what that sentence is.
///
/// The presses-only corpus is every shape a legacy terminal has: bare ASCII, a control chord, a meta
/// prefix, both cursor-key introducers, the tilde block, `CSI Z`, an SGR mouse press *and its
/// release*, focus in and out, and a paste. **The mouse release is deliberately in there** — a
/// button coming up is a `MouseKind::Up` and must never be mistaken for a `KeyKind::Release`, which
/// is exactly the confusion a "synthesise the missing half" implementation makes.
#[test]
fn a_presses_only_terminal_never_yields_release_or_repeat() {
    const LEGACY: &[u8] = b"a\x03\x1bb\x1b[A\x1bOB\x1b[3~\x1b[Z\x1b[<0;10;5M\x1b[<0;10;5m\
                            \x1b[I\x1b[O\x1b[200~pasted\x1b[201~\x1b[97;3u\x1b[1;5A";
    let events = crate::input_tests::parse(LEGACY);
    assert_eq!(
        events.len(),
        14,
        "nine keys, a mouse press and its release, focus in and out, and one paste: {events:?}"
    );

    let mut keys = 0;
    for event in &events {
        if let crate::input::Event::Key(key) = event {
            keys += 1;
            assert_eq!(
                key.kind,
                crate::input::KeyKind::Press,
                "{key:?} was synthesised out of a stream that reports only presses"
            );
        }
    }
    assert_eq!(keys, 9, "the corpus must be mostly keys: {keys}");

    // And the same keys from a terminal that reports event types, so that the absence above is a
    // property of the wire rather than of this parser.
    let enhanced = crate::input_tests::parse(b"\x1b[97;1:1u\x1b[97;1:2u\x1b[97;1:3u");
    let kinds: Vec<_> = enhanced
        .iter()
        .map(|event| match event {
            crate::input::Event::Key(key) => key.kind,
            other => panic!("unexpected {other:?}"),
        })
        .collect();
    assert_eq!(
        kinds,
        vec![
            crate::input::KeyKind::Press,
            crate::input::KeyKind::Repeat,
            crate::input::KeyKind::Release,
        ],
        "flag 2 is what makes the other two reachable at all"
    );
}

/// **Register entry #14** — motion floods collapse to one per wake; press floods do not.
///
/// One read is one wake, which is what the reader thread does: parse the whole read, then post.
/// **The asymmetry is asserted in the same test on purpose** (ADR 0008): a position on the way
/// carries no intent and the newest one supersedes it, and every other event expresses something the
/// user meant and is never dropped. Two tests would let one of them be deleted without the other
/// noticing.
#[test]
fn motion_floods_collapse_and_press_floods_do_not() {
    const FLOOD: usize = 1_000;

    let mut moves = Vec::new();
    for i in 0..FLOOD {
        // A hand waving across a 300-column screen, one SGR motion report per cell.
        moves.extend_from_slice(format!("\x1b[<35;{};10M", i % 300 + 1).as_bytes());
    }
    let queue = crate::input::Queue::new();
    for event in crate::input_tests::parse(&moves) {
        queue.push(event);
    }
    assert_eq!(
        queue.len(),
        1,
        "{FLOOD} intermediate positions are one position"
    );

    let presses = vec![b'a'; FLOOD];
    let queue = crate::input::Queue::new();
    for event in crate::input_tests::parse(&presses) {
        queue.push(event);
    }
    assert_eq!(
        queue.len(),
        FLOOD,
        "a keystroke is intent and is never dropped"
    );

    // The wheel is the case that looks like motion and is not: turning it five notches means five
    // notches, and coalescing them would make a scroll bar move a fifth as far.
    let mut wheel = Vec::new();
    for _ in 0..5 {
        wheel.extend_from_slice(b"\x1b[<65;10;10M");
    }
    let queue = crate::input::Queue::new();
    for event in crate::input_tests::parse(&wheel) {
        queue.push(event);
    }
    assert_eq!(queue.len(), 5, "a wheel notch is intent");
}

/// **Register entry #16** — the parser survives five adversarial splits.
///
/// Five sequences, each cut at **every** internal byte boundary, each half delivered as its own read
/// with its own end-of-read. The reassembled events must equal the events the whole sequence
/// produces.
///
/// # The one index where the answer legitimately differs
///
/// A cut after byte one leaves a read whose entire content is `ESC`, and a bare `ESC` at the end of
/// a read is the Escape key — see `crate::input::parse`'s module documentation for the two
/// alternatives and what each costs. That is asserted here rather than skipped, and the assertion is
/// two-sided: index one differs *and every other index does not*. A parser that quietly re-flushed
/// somewhere else would fail the second half.
#[test]
fn the_parser_survives_five_adversarial_splits() {
    const CORPUS: [&[u8]; 5] = [
        // A kitty key with a modifier and associated text.
        b"\x1b[97;2;65u",
        // An SGR mouse press.
        b"\x1b[<0;10;5M",
        // A bracketed paste, whose terminator is itself an escape sequence.
        b"\x1b[200~hi\x1b[201~",
        // A legacy modified arrow.
        b"\x1b[1;5A",
        // A three-byte scalar, which has no `ESC` in it at all.
        "漢".as_bytes(),
    ];

    // Every comparison below is made at one instant, because `at` is the real clock and two parses
    // of the same bytes differ in it by construction. See `crate::input_tests::stamped`.
    let epoch = std::time::Instant::now();
    for whole in CORPUS {
        let expected = crate::input_tests::stamped(crate::input_tests::parse(whole), epoch);
        assert!(
            !expected.is_empty(),
            "the corpus entry must parse whole: {whole:?}"
        );

        for cut in 1..whole.len() {
            let split = crate::input_tests::stamped(
                crate::input_tests::parse_with(1 << 20, &[&whole[..cut], &whole[cut..]]),
                epoch,
            );
            let escape_flush = cut == 1 && whole[0] == 0x1b;
            if escape_flush {
                assert_ne!(
                    split, expected,
                    "a read that is nothing but ESC is the Escape key, and this cut must show it"
                );
                match split.first() {
                    Some(crate::input::Event::Key(key)) => assert_eq!(
                        key.code,
                        crate::input::KeyCode::Escape,
                        "the documented flush produces Escape and nothing else"
                    ),
                    other => panic!("expected Escape first, got {other:?}"),
                }
            } else {
                assert_eq!(split, expected, "{whole:?} cut at {cut} did not reassemble",);
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// `set_mouse`, the negotiation and the caret (ticket 21).
//
// Two counts and one absence. Both counts are about the same obligation read from two ends —
// *idempotent, and free when the value has not changed* — and the absence is ADR 0005's: the
// terminal blinks the caret, so nothing here does.
// ---------------------------------------------------------------------------------------------

/// A terminal that can do every input protocol, so a gate about what was *asked for* is not
/// quietly a gate about what could be. See [`crate::Capabilities::with_input`].
#[cfg(test)]
fn every_input_protocol() -> crate::caps::Capabilities {
    crate::caps::Capabilities::with_input(true, true, true, true, crate::caps::KITTY_ALL)
}

/// **Gate, count: `set_mouse` with an unchanged value across a thousand frames writes zero bytes.**
///
/// The obligation the spec states rather than implies, and it is load-bearing rather than polite:
/// *the runtime calls this after every frame*, because the level is a `max` over what the frame's
/// components declared and there is nowhere else to take it. A naive actuator writes an escape
/// sequence per frame for ever.
///
/// The gate is stronger than the obligation, and the difference is the design: an unchanged setter
/// does not merely emit nothing, **it does not cause a frame**. The delta is computed on the app
/// thread, before `present` leases a packet, so a thousand unchanged calls cost a thousand
/// comparisons and no composite, no pack, no serialise and no write.
#[test]
fn a_thousand_unchanged_set_mouse_calls_write_nothing_and_cause_no_frame() {
    const FRAMES: usize = 1_000;
    let mut h = Harness::declaring(
        40,
        4,
        every_input_protocol(),
        crate::input::InputConfig {
            mouse: crate::input::MouseMode::Buttons,
            ..crate::input::InputConfig::default()
        },
    );
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 40, 4), true);
    h.screen
        .layers()
        .view(id)
        .unwrap()
        .text(0, 0, "a field", Style::new());
    h.present();
    // The floor is already on the wire, so the birth frame said nothing about it either.
    assert_eq!(h.terminal().modes(), vec![1000, 1006], "the declared floor");
    let settled = h.bytes_written();

    for _ in 0..FRAMES {
        // Exactly what a runtime does: the `max` over this frame's components, every frame.
        h.screen.set_mouse(crate::input::MouseMode::Buttons);
        assert!(!h.screen.present().submitted);
    }
    assert_eq!(
        h.bytes_written() - settled,
        0,
        "{FRAMES} unchanged `set_mouse` calls reached the wire"
    );
    assert_eq!(h.terminal().modes(), vec![1000, 1006], "and nothing moved");

    // And a real change still goes out, which is what says the zero above is not a broken actuator.
    h.screen.set_mouse(crate::input::MouseMode::Motion);
    assert!(h.screen.present().submitted);
    h.present();
    assert_eq!(h.terminal().modes(), vec![1003, 1006]);
}

/// **Gate: spec §8's 29-byte caret frame, with a caret on it.**
///
/// The number was producible from impl 13 onwards and was a claim about a frame with no caret in it:
/// `?2026h` + `0m` + `?2026l` is twenty bytes of fixed framing and an eight-byte `CUP` plus one
/// ASCII cell is the other nine. What this adds is the caret, and the property is that **it costs
/// nothing** — a character typed into a field leaves the terminal's cursor exactly where the caret
/// belongs, so the move the caret needs is the move the write already made.
///
/// That is not a coincidence to be grateful for. It is why the caret is placed with the serializer's
/// own `shortest` rather than with an unconditional `CUP`, and why the *shape* and the *visibility*
/// are tracked separately from the position: a frame that re-stated either would be 40 bytes, and
/// the caret is the most frequent frame there is.
#[test]
fn a_caret_frame_is_twenty_nine_bytes() {
    // Mode 2026 is the twenty bytes of framing, and no `Overrides` can name it — see
    // `Capabilities::on_the_wire`.
    let syncing = crate::caps::Capabilities::on_the_wire(
        crate::caps::ColorDepth::TrueColor,
        true,
        crate::quirks::Underlines::Standard,
        false,
    );
    let mut h = Harness::declaring(80, 24, syncing, crate::input::InputConfig::default());
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 80, 24), true);
    h.screen.set_cursor(Some(crate::actuate::Cursor {
        x: 40,
        y: 12,
        shape: crate::actuate::CursorShape::Terminal,
    }));
    // The birth frame paints the whole screen and shows the caret. Everything measured is after it.
    h.present();
    assert_eq!(h.terminal().caret(), Some((40, 12)));
    let settled = h.bytes_written();

    // One character typed into the field, and the caret follows it — which is where the terminal's
    // cursor already is.
    h.screen
        .layers()
        .view(id)
        .unwrap()
        .text(40, 12, "x", Style::new());
    h.screen.set_cursor(Some(crate::actuate::Cursor {
        x: 41,
        y: 12,
        shape: crate::actuate::CursorShape::Terminal,
    }));
    h.present();
    let frame = h.bytes_written() - settled;
    assert_eq!(
        frame,
        29,
        "§8's caret frame, with the caret on it: {}",
        String::from_utf8_lossy(&h.wire()[settled..]).replace('\x1b', "^[")
    );
    assert_eq!(
        h.terminal().caret(),
        Some((41, 12)),
        "and it is where it belongs"
    );
}

/// **Gate, count and absence: nothing in this crate blinks a caret.**
///
/// ADR 0005 measured the alternative: one `restyle` of one cell, toggled, is **two wakeups a second
/// for as long as anything has focus** — 7 200 an hour on a screen where nothing is happening. Ticket
/// 19's measured idle (`30.01 s real, 0.00 user, 0.00 sys, 0 voluntary context switches`) does not
/// survive that, and a text field is not an exotic component.
///
/// The count is the property and the scan is the insurance. A software caret cannot be invisible to
/// the count — it has to damage a cell, and damage produces a frame — so a thousand `present` calls
/// over a screen with a visible caret and nothing changing submit **nothing** and write **nothing**.
/// The scan then names what such an implementation would have to call itself, which catches the
/// version of it that is written and not yet wired.
#[test]
fn nothing_anywhere_blinks_a_caret_in_software() {
    let mut h = Harness::new(40, 4);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 40, 4), true);
    h.screen
        .layers()
        .view(id)
        .unwrap()
        .text(0, 0, "a focused field", Style::new());
    h.screen.set_cursor(Some(crate::actuate::Cursor {
        x: 15,
        y: 0,
        shape: crate::actuate::CursorShape::Bar,
    }));
    h.present();
    let settled = h.bytes_written();

    for _ in 0..1_000 {
        assert!(
            !h.screen.present().submitted,
            "a frame was produced with a caret on screen and nothing changing"
        );
    }
    assert_eq!(h.bytes_written() - settled, 0);
    assert_eq!(h.terminal().caret(), Some((15, 0)), "and it is still there");

    /// What a software caret would have to name, whatever it was called.
    const FORBIDDEN: &[&str] = &[
        "caret_blink",
        "blink_caret",
        "cursor_blink",
        "blink_phase",
        "caret_phase",
        "BLINK_RATE",
        "BLINK_INTERVAL",
        "BLINK_PERIOD",
        "toggle_caret",
    ];
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    collect_rust_files(&root, &mut files);
    assert!(
        files.len() > 20,
        "the source walk found {} files, so it is not walking the crate",
        files.len()
    );
    for path in files {
        // This file names them, and it is `cfg(test)`. See the display gate above for why the
        // exclusion is by name.
        if path.ends_with("gates.rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("the crate's own source is readable");
        for needle in FORBIDDEN {
            assert!(
                !text.contains(needle),
                "{} names `{needle}`: the terminal blinks the caret, in its own process, at the \
                 user's rate (ADR 0005)",
                path.display()
            );
        }
    }
}

/// **Ticket 19's idle gate, with a focused field on screen.**
///
/// The claim ADR 0005 rests on is that the caret costs nothing at rest, and ticket 19's gate was
/// written on a screen with no caret. This is the same measurement with one: the app thread enters
/// its wait once, comes back zero times, and the render thread's wait comes back zero times — with a
/// visible caret sitting on the screen for the whole window.
///
/// It is a separate gate rather than an extra assertion inside ticket 19's, because it is a claim
/// about a different thing: that one is about the clock, this one is about the caret, and a gate that
/// fails should say which.
#[test]
fn an_idle_application_with_a_caret_on_screen_still_wakes_for_nothing() {
    let (mut screen, wake) = threaded_at(120.0, Box::new(Counting::default()));
    let id = screen.layers().add_content(0, Rect::new(0, 0, 40, 4), true);
    screen
        .layers()
        .view(id)
        .unwrap()
        .text(0, 0, "a focused field", Style::new());
    screen.set_cursor(Some(crate::actuate::Cursor {
        x: 15,
        y: 0,
        shape: crate::actuate::CursorShape::Bar,
    }));
    assert!(screen.present().submitted);
    // The caret is on the wire before the window opens, so what is measured is a caret at rest
    // rather than a caret arriving.
    drained(&screen, 1);
    // **A baseline rather than a zero**, and the difference is the birth frame. Ticket 19's gate
    // presents nothing, so its render thread has never parked and its count is zero absolutely; this
    // one hands over a frame first, and whether that costs a park at all is a race between the two
    // threads — a render thread that finds the packet already in the slot never waited for it. What
    // is asserted is that the count does not move while nothing is happening.
    let render_wakeups = screen.render_wakeups();

    let source = wake.source();
    let quiet = std::time::Duration::from_millis(150);
    let observer = std::thread::spawn(move || {
        std::thread::sleep(quiet);
        let counts = source.park_counts();
        wake.quit();
        counts
    });

    let began = std::time::Instant::now();
    assert_eq!(screen.wait(), Wake::Quit);
    let (parks, wakeups) = observer.join().expect("the observer does not panic");

    assert!(
        began.elapsed() >= quiet,
        "the wait came back before anything had happened"
    );
    assert_eq!(parks, 1, "an indefinite park re-entered the wait");
    assert_eq!(
        wakeups, 0,
        "a caret on screen woke the app thread: a software one would put 18 here at two blinks a \
         second, and 7 200 in an hour"
    );
    assert_eq!(
        screen.render_wakeups(),
        render_wakeups,
        "the render thread woke with nothing in the slot"
    );
    assert_eq!(
        screen.handoff_counts().2,
        1,
        "one frame, and it is the birth frame"
    );
}

// ---------------------------------------------------------------------------------------------
// Shutdown: three exits, one restoration, and the frame that is deliberately not written.
//
// Spec §7's *resize, shutdown and panic*, and the property register's entry #15. The three exits
// below are three **child processes** rather than three in-process cases, and that is forced rather
// than chosen: the panic hook is process-global and installed once, so an in-process gate about it
// would be a gate about which test armed `crate::shutdown`'s site last — which under `cargo test`
// is whichever test the scheduler happened to run beside it. A child process has exactly one
// screen, one hook and one exit.
//
// It also buys the ordering the property is really about. The child's stdout and stderr are two
// handles on **one open file**, so what the parent reads back is the two streams interleaved in
// write order — and *the backtrace prints outside the alt screen* is exactly a statement about that
// order. A pty would show the same order for the same reason; this crate's dependency policy has no
// way to open one (`deny.toml`), and `.gitlab-ci.yml` runs the whole suite a second time under
// `script` so that this gate is executed with a real terminal on the process's other end anyway.
// ---------------------------------------------------------------------------------------------

/// Turns this test binary into the child of one of the three exit gates, and says which exit.
const EXIT_MODE: &str = "VITUI_SHUTDOWN_EXIT";

/// What the panicking child says, so that the parent can find it in the stream.
const CHILD_PANIC: &str = "the child is going down on purpose";

/// Everything the epilogue has to give back, as bytes, in the order [`crate::actuate::restoration`]
/// writes them.
///
/// One definition for all three exits, because *they all take the same path* is the property and
/// three copies of this string could quietly stop saying so.
pub(crate) const EPILOGUE: &str = "\x1b[?2004l\x1b[?1004l\x1b[?1003l\x1b[?1002l\x1b[?1000l\x1b[?1006l\x1b[<u\x1b[?25h\x1b[?7h\x1b[?1049l";

/// The same, with the escapes made readable — for the failure messages, and for
/// [`crate::shutdown`]'s unit tests, which assert the same sequence one level down.
pub(crate) const EPILOGUE_ESCAPED: &str =
    "^[[?2004l^[[?1004l^[[?1003l^[[?1002l^[[?1000l^[[?1006l^[[<u^[[?25h^[[?7h^[[?1049l";

/// The same, on a screen that declared nothing and was answered nothing.
///
/// A caller-supplied sink is asked no questions, so all eight input facts are false and the six
/// modes above are neither set nor reset — **which is the property and not a shortcut**: a mode this
/// session did not take is a mode it may not give back, because `CSI ? 1004 l` sent to a terminal
/// whose user had focus reporting on for their own reasons turns it off for them. What is left is
/// the three every session takes unconditionally.
const EPILOGUE_PLAIN: &str = "\x1b[?25h\x1b[?7h\x1b[?1049l";

/// Run this same test binary again as a child, and give back everything it wrote — **stdout and
/// stderr in one stream, in write order**.
fn child_output(test: &str, mode: &str) -> String {
    use std::process::{Command, Stdio};

    let path =
        std::env::temp_dir().join(format!("vitui-shutdown-{}-{mode}.out", std::process::id()));
    let file = std::fs::File::create(&path).expect("a file in the temp directory");
    // **`try_clone`, not a second `open`.** Two handles on one open file share one offset, which is
    // what makes the interleaving below the child's write order rather than two races to byte zero.
    let merged = file.try_clone().expect("the same open file, twice");
    let status = Command::new(std::env::current_exe().expect("the test binary's own path"))
        .args(["--exact", test, "--nocapture", "--test-threads=1"])
        .env(EXIT_MODE, mode)
        // libtest's capture would otherwise swallow the panic message, which is half of what the
        // panic gate is reading.
        .env("RUST_BACKTRACE", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::from(file))
        .stderr(Stdio::from(merged))
        .status()
        .expect("the test binary is runnable");
    let out = std::fs::read_to_string(&path).expect("what the child wrote");
    let _ = std::fs::remove_file(&path);
    assert!(
        out.contains("running 1 test"),
        "the child exited {status:?} without running `{test}` — the name in the parent and the name \
         of the test have to agree, and nothing else checks that: {out}"
    );
    out
}

/// The child of all three gates: take the terminal, paint one frame, and leave by the named exit.
///
/// Every optional input mode is declared **and** answered for, so that the epilogue's every arm is
/// on the wire rather than skipped as inapplicable. The mouse is then raised above its floor,
/// because the level the terminal has to be given back is the one it was **last told** and not the
/// one the negotiation set.
fn exit_through(mode: &str) {
    let engine = crate::engine::Engine::new(crate::engine::Config {
        size: (20, 4),
        output: crate::engine::Output::Sink(Box::new(std::io::stdout())),
        clock: crate::engine::Clock::Manual,
        max_frame_rate: f32::INFINITY,
        overrides: Overrides::default(),
        overrun_threshold: None,
        overrun_report: None,
        input: crate::input::InputConfig {
            mouse: crate::input::MouseMode::Buttons,
            focus: true,
            paste: true,
            ..crate::input::InputConfig::default()
        },
    });
    // The frames go to this child's standard output and so does the panic path's restoration, which
    // is what puts both in one stream for the parent to read an order out of.
    let (mut screen, _wake) = engine
        .attach_restoring_into(Some(every_input_protocol()), Box::new(std::io::stdout()))
        .expect("attaching to a sink cannot fail");
    screen.set_mouse(crate::input::MouseMode::Motion);
    let id = screen.layers().add_content(0, Rect::new(0, 0, 20, 4), true);
    screen
        .layers()
        .view(id)
        .expect("the layer was just added")
        .text(0, 0, "on the alt screen", Style::new());
    assert!(screen.present().submitted);

    match mode {
        // The application's `main` returns, and the guard on the type is what restores.
        "return" => {}
        // A `?` propagates out from under a live `Screen`, which is the same guard reached by the
        // other of the two ordinary exits.
        "question" => {
            fn out_of_main(screen: crate::engine::Screen) -> Result<(), std::fmt::Error> {
                // Held across the `?`, so that what drops it is the unwinding of this frame rather
                // than a line somebody remembered to write.
                let _screen = screen;
                Err(std::fmt::Error)?;
                unreachable!("the `?` above always propagates")
            }
            assert!(out_of_main(screen).is_err());
        }
        // And the one that is not a return at all.
        "panic" => panic!("{CHILD_PANIC}"),
        other => unreachable!("no such exit: {other}"),
    }
}

/// **A normal return restores the terminal**, and the guard is the type.
#[test]
fn a_normal_return_restores_the_terminal() {
    if std::env::var(EXIT_MODE).is_ok() {
        return exit_through("return");
    }
    let out = child_output("gates::a_normal_return_restores_the_terminal", "return");
    assert_restored_exactly_once(&out);
}

/// **A `?` out of `main` takes the same path**, because it drops the same value.
///
/// It is the exit that is easiest to have working by accident and easiest to break: an epilogue
/// written by a method somebody has to call is an epilogue a `?` walks straight past.
#[test]
fn a_question_mark_out_of_main_restores_the_terminal() {
    if std::env::var(EXIT_MODE).is_ok() {
        return exit_through("question");
    }
    let out = child_output(
        "gates::a_question_mark_out_of_main_restores_the_terminal",
        "question",
    );
    assert_restored_exactly_once(&out);
}

/// **Register entry #15, and it is the word *panic* that this entry turns on.**
///
/// > Panic restore pops keyboard flags and disables mouse, focus reporting and bracketed paste.
///
/// Three things are asserted and the second is the one that costs a debugging session to find out
/// about the hard way:
///
/// - the whole epilogue is on the wire, **captured** rather than inspected — the kitty pop, all
///   three input modes, the caret, auto-wrap and the alt screen;
/// - it is there **before** the panic message, because restoration that runs after the default hook
///   paints the backtrace into a page the terminal is about to discard;
/// - it is there **once**, although both the hook and the unwinding `Screen`'s `Drop` ran. That is
///   the atomic, observed from the outside.
#[test]
fn a_panic_restores_the_terminal_before_the_backtrace_prints() {
    if std::env::var(EXIT_MODE).is_ok() {
        return exit_through("panic");
    }
    let out = child_output(
        "gates::a_panic_restores_the_terminal_before_the_backtrace_prints",
        "panic",
    );
    assert_restored_exactly_once(&out);

    let restored = out.find(EPILOGUE).expect("asserted just above");
    let printed = out
        .find(CHILD_PANIC)
        .unwrap_or_else(|| panic!("the child never printed its panic: {}", escaped(&out)));
    assert!(
        restored < printed,
        "the backtrace was printed into the alt screen, at {printed} against a restoration at \
         {restored}: {}",
        escaped(&out)
    );
}

/// What every exit owes the terminal, and it owes it exactly once.
fn assert_restored_exactly_once(out: &str) {
    assert_eq!(
        out.matches("\x1b[?1049h").count(),
        1,
        "the alt screen was entered {} times: {}",
        out.matches("\x1b[?1049h").count(),
        escaped(out)
    );
    assert!(
        out.contains(EPILOGUE),
        "the epilogue is not on the wire in one piece: {}",
        escaped(out)
    );
    assert_eq!(
        out.matches("\x1b[?1049l").count(),
        1,
        "the restoration ran more than once: {}",
        escaped(out)
    );
}

/// Escapes made visible, because a failure message full of raw `ESC` reprograms the reader's own
/// terminal instead of telling them anything.
fn escaped(out: &str) -> String {
    out.replace('\x1b', "^[")
}

/// **The render thread is joined; the input thread is not** (spec §7).
///
/// The join is not observable as a join, so it is observed as its consequence: the renderer — and
/// the sink inside it — comes back to the app thread, and the epilogue is written from **this**
/// thread while the frames were written from another. Nothing but a join produces that, and a
/// `Screen::drop` that skipped it would either write the epilogue through a sink two threads hold or
/// find no sink at all.
///
/// The input thread's asymmetry is not gated and cannot usefully be: it sits in a blocking `read`
/// with nothing to wake it short of a signal, so it dies with the process, and a headless screen
/// does not have one at all. The reason is a comment where the second join would go, in
/// `Screen::reclaim_renderer`.
#[test]
fn the_render_thread_is_joined_and_the_app_thread_writes_the_epilogue() {
    /// Every write, with the thread that made it.
    type Writes = std::sync::Arc<std::sync::Mutex<Vec<(std::thread::ThreadId, Vec<u8>)>>>;

    #[derive(Clone)]
    struct WhoWrote(Writes);

    impl std::io::Write for WhoWrote {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0
                .lock()
                .expect("never poisoned")
                .push((std::thread::current().id(), buf.to_vec()));
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    let log = WhoWrote(std::sync::Arc::new(std::sync::Mutex::new(Vec::new())));
    let seen = log.clone();
    let here = std::thread::current().id();

    let mut screen = threaded(Box::new(log));
    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
    screen
        .layers()
        .view(id)
        .expect("the layer was just added")
        .text(0, 0, "one frame", Style::new());
    assert!(screen.present().submitted);
    drained(&screen, 1);
    drop(screen);

    let writes = seen.0.lock().expect("never poisoned");
    let (prologue_thread, _) = writes.first().expect("the prologue is a write");
    assert_eq!(
        *prologue_thread, here,
        "the prologue goes out before a second thread exists"
    );
    assert!(
        writes.iter().any(|(who, _)| *who != here),
        "no frame was written by a render thread, so this gate is about nothing"
    );
    let (epilogue_thread, epilogue) = writes.last().expect("the epilogue is a write");
    assert_eq!(
        String::from_utf8_lossy(epilogue).replace('\x1b', "^["),
        escaped(EPILOGUE_PLAIN)
    );
    assert_eq!(
        *epilogue_thread, here,
        "the epilogue was written by a thread that should have been joined by then"
    );
}

/// A sink that records every write and takes none of them until the test lets go.
///
/// One token per write, and **a dropped sender releases it for ever** — which is how the epilogue
/// gets out at the end without a test having to count the writes it did not make.
struct GatedLog {
    gate: std::sync::mpsc::Receiver<()>,
    writes: std::sync::Arc<std::sync::Mutex<Vec<Vec<u8>>>>,
}

impl std::io::Write for GatedLog {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let _ = self.gate.recv();
        self.writes
            .lock()
            .expect("never poisoned")
            .push(buf.to_vec());
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// **The last frame is not flushed on quit** (spec §7), as a count of writes.
///
/// Showing state that is already stale buys nothing and costs exit latency, so a packet still in the
/// slot when `quit` arrives goes nowhere. Two frames are submitted and the renderer is held inside
/// the first one's write for the whole of it, so the second is sitting in the slot at the moment
/// `Screen::drop` sets `quit` — and `Mailbox::take` answers `None` before it answers with a packet.
///
/// # Why this is an ordering and not a sleep
///
/// The gate opens strictly **after** `quit` is set, because the thread that opens it spins on
/// [`Mailbox::quit_requested`](crate::handoff::Mailbox) rather than on a clock. A sleep here would
/// be a flake with a budget's clothes on: the whole property is *which of two things happened
/// first*.
#[test]
fn the_last_frame_is_not_flushed_on_quit() {
    let (tx, gate) = std::sync::mpsc::channel();
    // The prologue, which `attach` writes on this thread before a second one exists. Gating it would
    // deadlock the constructor.
    tx.send(()).expect("the receiver is alive");
    let writes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let mut screen = threaded(Box::new(GatedLog {
        gate,
        writes: std::sync::Arc::clone(&writes),
    }));
    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
    let paint = |screen: &mut Screen, text: &str| {
        screen
            .layers()
            .view(id)
            .expect("the layer was just added")
            .text(0, 0, text, Style::new());
    };

    // Frame one is taken and the renderer is now inside its write, holding the gate shut.
    paint(&mut screen, "first");
    assert!(screen.present().submitted);
    screen.wait_for_renderer();

    // Frame two fills the pool's other packet and lands in the slot. Nothing will ever take it.
    paint(&mut screen, "second");
    assert!(
        screen.present().submitted,
        "the pool's second packet was there to fill"
    );

    let mailbox = screen.mailbox_handle();
    let releaser = std::thread::spawn(move || {
        while !mailbox.quit_requested() {
            std::hint::spin_loop();
        }
        drop(tx);
    });
    drop(screen);
    releaser.join().expect("the releaser cannot panic");

    let writes = writes.lock().expect("never poisoned");
    assert_eq!(
        writes.len(),
        3,
        "the prologue, one frame and the epilogue — and a fourth write is the stale frame: {:?}",
        writes
            .iter()
            .map(|w| escaped(&String::from_utf8_lossy(w)))
            .collect::<Vec<_>>()
    );
    let all = writes.concat();
    let seen = String::from_utf8_lossy(&all);
    assert!(seen.contains("first"), "the frame that was taken: {seen:?}");
    assert!(
        !seen.contains("second"),
        "the last frame was flushed on quit: {seen:?}"
    );
    assert!(seen.ends_with(EPILOGUE_PLAIN), "{}", escaped(&seen));
}

/// **The restoration stops the render thread, so nothing is painted after the terminal is given
/// back.**
///
/// This is the failure the panic path invites and `Screen::drop` does not cover: the hook runs on
/// whichever thread panicked, and until this ticket the only thing that set `quit` was the drop —
/// which comes *after* the hook, and on a worker thread's panic may never come at all, because a
/// panic off the main thread does not end the process. In between, the epilogue leaves the alt
/// screen and the render thread takes the next packet and paints cells onto the user's shell.
///
/// Two frames are submitted with the renderer held inside the first one's write, so the second is in
/// the slot when the restoration happens. It is then never written — and the gate is an ordering
/// rather than a sleep, because the sink's gate opens strictly after the restoration has returned.
#[test]
fn the_restoration_stops_the_renderer_before_the_terminal_is_given_back() {
    let (tx, gate) = std::sync::mpsc::channel();
    tx.send(()).expect("the receiver is alive");
    let writes = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let epilogue = crate::testing::Recorder::new();

    let (mut screen, _wake) = crate::engine::Engine::new(crate::engine::Config {
        size: (W, H),
        output: crate::engine::Output::Sink(Box::new(GatedLog {
            gate,
            writes: std::sync::Arc::clone(&writes),
        })),
        clock: crate::engine::Clock::System,
        max_frame_rate: f32::INFINITY,
        overrides: crate::testing::pinned_truecolor(),
        overrun_threshold: None,
        overrun_report: None,
        input: crate::input::InputConfig::default(),
    })
    // The panic path's sink, which over a caller-supplied output would otherwise not exist.
    .attach_restoring_into(None, Box::new(epilogue.clone()))
    .expect("attaching to a sink cannot fail");

    let id = screen.layers().add_content(0, Rect::new(0, 0, W, H), true);
    let paint = |screen: &mut Screen, text: &str| {
        screen
            .layers()
            .view(id)
            .expect("the layer was just added")
            .text(0, 0, text, Style::new());
    };

    paint(&mut screen, "first");
    assert!(screen.present().submitted);
    screen.wait_for_renderer();
    paint(&mut screen, "second");
    assert!(screen.present().submitted, "and it lands in the slot");

    assert!(screen.restore_as_a_panic_would());
    assert_eq!(
        String::from_utf8_lossy(&epilogue.handle().lock().expect("never poisoned").bytes)
            .replace('\x1b', "^["),
        escaped(EPILOGUE_PLAIN),
        "the panic path wrote the epilogue through its own sink"
    );

    // Only now can the renderer move at all, and by then it has already been told to stop. It
    // finishes the write it is inside and then leaves, which is the **observable** consequence and
    // the one this gate turns on: a renderer that was not told would take the packet in the slot and
    // paint it, and would then still be parked on `take` when the deadline expired.
    drop(tx);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(20);
    while !screen.renderer_is_gone() && std::time::Instant::now() < deadline {
        std::hint::spin_loop();
    }
    assert!(
        screen.renderer_is_gone(),
        "the restoration left the render thread running with a packet in the slot"
    );
    drop(screen);

    let writes = writes.lock().expect("never poisoned");
    let seen = String::from_utf8_lossy(&writes.concat()).into_owned();
    assert!(seen.contains("first"), "the frame that was taken: {seen:?}");
    assert!(
        !seen.contains("second"),
        "a frame was painted onto the user's shell after the terminal was restored: {seen:?}"
    );
    assert!(
        !seen.contains("\x1b[?1049l"),
        "the epilogue went out twice, once through each sink: {}",
        escaped(&seen)
    );
}

// ---------------------------------------------------------------------------------------------
// The unblockable app thread: the two runtime rungs, and the negative corpus's runtime companions.
//
// Spec §11, and register entries #18 and #22. Four of the five rungs cost nothing and are checked by
// the compiler — the paired `compile_fail` doctests on `Screen`, `View` and `Permit`, and the absence
// of any blocking accessor on `Slot` — so what is left here is what runs: the in-loop detector, the
// escape hatch, and the observer thread whose sanction ends the process.
//
// **No gate here asserts a scheduler property.** A test that spends 166.76 us and then insists the
// operating system gave the thread back inside 8.3 ms is a flaky test wearing a budget's clothes,
// which is exactly what §14 refuses. So the numbers live in `crate::perf`'s unit tests, where the
// instants are arguments rather than measurements, and what is driven through the public API here is
// the *wiring*: that the iteration begins at `wait` and ends at `present`, on every path out of it,
// with a threshold taken from the rate the application declared.
// ---------------------------------------------------------------------------------------------

/// A headless screen on the threaded clock, at `hz`, with the budget pinned to `threshold`.
///
/// The threaded clock rather than `Clock::Manual` for one reason: `wait` is where the iteration
/// begins, `Manual` is the mode in which a deterministic program does not call it, and a gate about
/// the detector that never entered an iteration would be a gate about nothing.
///
/// **A tight `threshold` here does not arm a tight abort**, and the review is what made that true.
/// The observer's limit was the threshold times sixty-four with no floor, so a gate pinning 1 ms and
/// sleeping 20 ms inside its iteration sat 3.2x away from `std::process::abort` on a loaded runner —
/// which would have taken all seven hundred tests with it and reported a stall instead of the
/// assertion that actually broke. `crate::perf::STALL_FLOOR` is the fix: every limit a `Screen` can
/// have is at least a second, whatever budget the caller pinned.
fn detected_screen(
    hz: f32,
    threshold: Option<std::time::Duration>,
) -> (Screen, crate::engine::WakeHandle) {
    crate::engine::Engine::new(crate::engine::Config {
        size: (40, 8),
        output: crate::engine::Output::Sink(Box::new(std::io::sink())),
        clock: crate::engine::Clock::System,
        max_frame_rate: hz,
        overrides: Overrides::default(),
        overrun_threshold: threshold,
        overrun_report: None,
        input: crate::input::InputConfig::default(),
    })
    .attach()
    .expect("attaching to a sink cannot fail")
}

/// **The threshold is one frame interval of the rate the application declared** — never the map's
/// 100 us, which is a CI gate on one stage of the engine's own work.
///
/// The number itself is `crate::perf`'s to check, against instants that are arguments; what is
/// checked here is that a `Screen` at 120 Hz really carries 8.333 ms and that
/// `set_max_frame_rate` moves it, because those are two wires and either could be missing.
#[test]
fn a_screens_budget_is_one_frame_interval_of_its_own_rate() {
    let (mut screen, _wake) = detected_screen(120.0, None);
    assert_eq!(
        screen.overrun_threshold(),
        std::time::Duration::from_nanos(8_333_333)
    );
    screen.set_max_frame_rate(300.0);
    assert_eq!(
        screen.overrun_threshold(),
        std::time::Duration::from_nanos(3_333_333)
    );
}

/// **A pinned threshold is the application's number and a rate change does not rescale it.**
#[test]
fn a_pinned_budget_survives_a_monitor_changing_under_the_program() {
    let pinned = std::time::Duration::from_millis(2);
    let (mut screen, _wake) = detected_screen(120.0, Some(pinned));
    screen.set_max_frame_rate(300.0);
    assert_eq!(screen.overrun_threshold(), pinned);
}

/// **The default budget is `Config`'s default rate**, and this is the only thing that says so.
///
/// `crate::perf::DEFAULT_HZ` is a deliberate second copy of `Config::DEFAULT_MAX_FRAME_RATE` — that
/// module reaches for nothing in the crate, because `examples/budget.rs` `#[path]`-includes it to
/// time the detector in a release build — so the two can drift, and nothing but an equality would
/// notice.
#[test]
fn the_detectors_default_budget_is_the_configs_default_rate() {
    let (screen, _wake) = detected_screen(crate::engine::Config::default().max_frame_rate, None);
    let (defaulted, _wake) = detected_screen(crate::perf::DEFAULT_HZ, None);
    assert_eq!(screen.overrun_threshold(), defaulted.overrun_threshold());
}

/// **The iteration begins when `wait` returns and ends when `present` does**, and an overrun between
/// them panics a debug build on the first one.
///
/// The sleep is twenty times the pinned budget, so there is no scheduler question in either
/// direction: no runner is slow enough to make this pass and none is fast enough to make it fail.
#[test]
#[should_panic(expected = "a frame was dropped")]
fn an_overrun_between_wait_and_present_panics_a_debug_build() {
    let (mut screen, wake) = detected_screen(120.0, Some(std::time::Duration::from_millis(1)));
    wake.post();
    assert_eq!(screen.wait(), Wake::Posted);
    std::thread::sleep(std::time::Duration::from_millis(20));
    screen.present();
}

/// **Every path out of `present` ends the iteration**, including the ones that submit nothing.
///
/// The body has six early returns, and a diagnostic that missed one would go quiet on exactly the
/// frames worth reporting: an idle `present` here is the cheapest of them, and it still has to leave.
/// The gate is that the *second* iteration is charged from its own `wait` rather than from the first
/// one's — if `leave` had not run, the entry instant would still be the older one and this would
/// panic.
#[test]
fn an_idle_present_still_ends_the_iteration_it_was_in() {
    let (mut screen, wake) = detected_screen(120.0, Some(std::time::Duration::from_millis(50)));
    wake.post();
    assert_eq!(screen.wait(), Wake::Posted);
    assert!(!screen.present().submitted, "nothing was drawn");
    std::thread::sleep(std::time::Duration::from_millis(30));
    wake.post();
    assert_eq!(screen.wait(), Wake::Posted);
    // Thirty of the fifty milliseconds were spent outside any iteration. A detector that had not
    // left the first one would charge them to this one and panic.
    assert!(!screen.present().submitted);
}

/// **A `present` with no `wait` before it is not an iteration at all**, which is what makes
/// `Clock::Manual` usable as a straight-line program — and what makes every other gate in this file
/// safe to run beside a debugger.
#[test]
fn the_detector_is_silent_without_a_wait() {
    let mut h = Harness::new(8, 2);
    std::thread::sleep(std::time::Duration::from_millis(5));
    h.screen
        .layers()
        .add_content(0, Rect::new(0, 0, 8, 2), true);
    h.present();
}

/// **A frame may be drawn inside a permitted region.** The case whose first shape was `E0499` and
/// whose second was `E0502`.
///
/// Spec §11 records the `&mut self` shape failing and the fix as *`Cell` and `&self` throughout*.
/// That is half of one: with `Cell` inside, a `Permit<'a>` borrowed out of `&'a Screen` still cannot
/// have a frame drawn inside it, because `Screen::layers` takes `&mut self`. This test is what a
/// borrowing guard cannot pass, which is why the guard holds an `Rc` and has no lifetime — see
/// `crate::perf::Permit`.
///
/// The budget is pinned tight and the region sleeps well past it, so the assertion is not *this was
/// fast* but *this was excused*.
#[test]
fn a_frame_is_drawn_inside_a_permitted_region_and_the_region_is_excused() {
    let (mut screen, wake) = detected_screen(120.0, Some(std::time::Duration::from_millis(1)));
    wake.post();
    assert_eq!(screen.wait(), Wake::Posted);
    let permit = screen.permit_slow("the cold-start frame");
    std::thread::sleep(std::time::Duration::from_millis(20));
    let id = screen.layers().add_content(0, Rect::new(0, 0, 8, 1), true);
    screen
        .layers()
        .view(id)
        .expect("the layer was just added")
        .text(0, 0, "cold", Style::new());
    assert!(screen.present().submitted);
    drop(permit);
}

/// **What a permit excused comes off the iteration and not off the detector.**
///
/// The frame around the permitted region is still charged, and it is the half of the escape hatch
/// that makes it an escape rather than a switch: a permit taken for a 1 us region does not buy the
/// 300 ms after it.
#[test]
#[should_panic(expected = "a frame was dropped")]
fn a_permit_that_ended_does_not_excuse_what_came_after_it() {
    let (mut screen, wake) = detected_screen(120.0, Some(std::time::Duration::from_millis(1)));
    wake.post();
    assert_eq!(screen.wait(), Wake::Posted);
    drop(screen.permit_slow("a region that cost nothing"));
    std::thread::sleep(std::time::Duration::from_millis(20));
    screen.present();
}

/// **The split is internal**: `Parker`, `Producer`, `Consumer` and `Ui` are not public names.
///
/// ADR 0003 split each of two primitives into an app-thread half and a `Send` half, and the seam
/// ticket then made all four private — which *strengthens* the enforcement rather than hiding it,
/// because `Perf::enter` and `Perf::leave` stop being merely unforgettable and become uncallable.
/// What is left on the public surface is `Screen`, `WakeHandle`, `Slot` and `Permit`, and the four
/// names below being absent from `lib.rs`'s re-exports is the only thing that says so.
#[test]
fn the_split_handles_are_not_public_names() {
    let lib = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"))
        .expect("the crate root is beside this file");
    // **Whole statements, not first lines.** `lib.rs` already has a re-export spanning four lines —
    // `pub use input::{ … }` — so a `Parker` added on a continuation line would be invisible to a
    // filter over line starts, which is what the first draft of this gate was.
    let mut exports: Vec<String> = Vec::new();
    let mut open: Option<String> = None;
    for line in lib.lines() {
        let statement = match open.take() {
            Some(mut held) => {
                held.push(' ');
                held.push_str(line.trim());
                held
            }
            None if line.starts_with("pub use ") || line.starts_with("pub mod ") => {
                line.trim().to_string()
            }
            None => continue,
        };
        match statement.ends_with(';') {
            true => exports.push(statement),
            false => open = Some(statement),
        }
    }
    assert!(
        open.is_none(),
        "a re-export in lib.rs never ended in a semicolon, so this gate stopped reading early"
    );
    // Nine at the time of writing, and the count is here so that a gate reading an empty list fails
    // rather than passes. It is a floor and not an equality: adding a re-export is ordinary.
    assert!(
        exports.len() >= 9,
        "only {} re-exports were parsed out of lib.rs, which is too few to be the whole list",
        exports.len()
    );
    for orphan in ["Parker", "Unparker", "Producer", "Consumer", "Ui"] {
        for line in &exports {
            assert!(
                !line.contains(orphan),
                "`{orphan}` is on the public surface again: {line}"
            );
        }
    }
    // And the positive twin, because a list of absences passes for any reason including the file
    // having been renamed out from under it.
    for present in ["Screen", "WakeHandle", "Slot", "Permit"] {
        assert!(
            exports.iter().any(|line| line.contains(present)),
            "`{present}` is not re-exported, so this test is checking a file that moved"
        );
    }
}

/// **Refusal 7, over the source rather than in prose: no blocking receive is inside the app thread's
/// loop.**
///
/// Spec §12 spells this *the word `recv` does not appear*, and taken literally it is neither true nor
/// desirable — and the two ways it is false are not the same way, which is why the allowance below
/// carries a reason per file rather than a count. Two are on **other threads**: the render thread
/// waits on a one-message channel for the renderer `attach` hands it, and the input thread's whole
/// life is a receive on the reader's channel. One is on the app thread and **outside the loop**:
/// capability detection reads the terminal's answers with a deadline, at `attach`, before a render
/// thread exists and before anything has drawn.
///
/// So the property is *no blocking receive is inside the app thread's iteration*, and a new one
/// anywhere else fails the build. The `Slot` this ticket added is what makes that liveable: a
/// background result reaches the app thread through a non-blocking `take` and a `WakeHandle`, so an
/// application never needs the call it cannot make.
#[test]
fn every_blocking_receive_in_the_crate_is_outside_the_app_threads_loop() {
    /// Where a blocking receive may appear, and why that one is not inside a frame.
    const ALLOWED: [(&str, &str); 4] = [
        (
            "reader.rs",
            "another thread: the input thread's whole life, and it owns the read direction",
        ),
        (
            "engine.rs",
            "another thread: the render thread waiting for the renderer `attach` hands it after the \
             spawn succeeded",
        ),
        (
            "detect.rs",
            "the app thread, and outside the loop: §10's query batch is read back with a deadline \
             at `attach`, before a render thread exists and before anything has drawn",
        ),
        (
            "gates.rs",
            "tests: sinks and orderings, on threads standing in for the renderer",
        ),
    ];
    /// Every blocking spelling, not just the bare one. `recv_timeout` is what `clippy.toml` names as
    /// the offence, and `recv_deadline` is the same call with the argument the other way round.
    const SPELLINGS: [&str; 3] = [".recv(", ".recv_timeout(", ".recv_deadline("];
    /// **Recursive, because `src/` has subdirectories and both of them hold code.** `src/input/` and
    /// `src/ucd/` exist, and a flat `read_dir` would have made this gate pass for a blocking receive
    /// added to the input parser — the one file where a reader would most expect to find one.
    fn walk(dir: &std::path::Path, found: &mut Vec<(String, usize)>) {
        for entry in std::fs::read_dir(dir).expect("a readable source directory") {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                walk(&path, found);
                continue;
            }
            if path.extension().is_none_or(|e| e != "rs") {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("a readable source file");
            let count: usize = SPELLINGS.iter().map(|s| source.matches(s).count()).sum();
            if count > 0 {
                let name = path
                    .file_name()
                    .expect("a file with an extension has a name")
                    .to_string_lossy()
                    .into_owned();
                found.push((name, count));
            }
        }
    }

    let dir = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
    let mut found = Vec::new();
    walk(dir, &mut found);
    let mut offenders = Vec::new();
    let mut seen = Vec::new();
    for (name, count) in found {
        match ALLOWED.iter().any(|(file, _)| *file == name) {
            true => seen.push(name),
            false => offenders.push(format!("{name} has {count}")),
        }
    }
    assert!(
        offenders.is_empty(),
        "a blocking receive appeared somewhere the allowance does not name: {}. Spec §12's refusal \
         7 is that nothing inside the app thread's iteration waits for a result; `Slot::take` and \
         `WakeHandle::post` are what it uses instead. If the new one really is outside the loop or on \
         another thread, add it to `ALLOWED` **with that reason** — the reason is the gate",
        offenders.join(", ")
    );
    // The positive twin: the two files that are allowed one still have one. Otherwise this test
    // passes on the day somebody deletes the render thread's channel and leaves the allowance behind.
    for (file, _) in ALLOWED {
        assert!(
            seen.iter().any(|s| s == file),
            "`{file}` is on the allowance and has no `recv` in it — the allowance is stale"
        );
    }
}

/// **The detector reaches for nothing in the crate**, which is what lets `examples/budget.rs`
/// `#[path]`-include it and time register entry #22 in a release build.
///
/// The same rule `crate::input` carries for `tests/alloc.rs`, and it looks like tidiness in both
/// places and is not: one `use crate::` here would drag the whole engine into that binary. The one
/// thing the module cannot do for itself — give the terminal back — arrives as a `fn()` pointer.
#[test]
fn the_detector_reaches_for_nothing_in_the_crate() {
    let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/perf.rs"))
        .expect("the detector is beside this file");
    for (number, line) in source.lines().enumerate() {
        let code = line.trim_start();
        if code.starts_with("//") {
            continue;
        }
        assert!(
            !code.contains("crate::"),
            "crates/vitui-engine/src/perf.rs:{} names the crate: {code}",
            number + 1
        );
    }
    // And it declares no **file-backed** child module, because a `#[path]`-included file resolves
    // its children beside itself rather than under it. An inline `mod` needs no file and is fine,
    // which is why the unit tests below it are one.
    for (number, line) in source.lines().enumerate() {
        let code = line.trim_start();
        assert!(
            !(code.starts_with("mod ") && code.ends_with(';')),
            "crates/vitui-engine/src/perf.rs:{} declares a file-backed child module, which a \
             `#[path]` include cannot resolve: {code}",
            number + 1
        );
    }
}

/// Turns this test binary into the child of the observer gate.
const STALL_MODE: &str = "VITUI_OBSERVER_STALL";

/// What the child declares its stalled region to be, so the parent can find it in the report.
const STALL_REASON: &str = "a stall on purpose";

/// The exit the child takes if the observer never fires, so that *nothing happened* is a failure
/// with a name rather than a hung job.
const NEVER_FIRED: i32 = 17;

/// **Gate: the observer's sanction is restore, print, abort — in that order.**
///
/// A child process, for the same reason the three exit gates are one: the panic hook and the
/// restoration site are process-global, and this gate ends the process on purpose. It also buys the
/// ordering the property is really about — the child's stdout and stderr are two handles on **one
/// open file**, so what comes back is the two streams interleaved in write order, and *the terminal
/// was given back before the message was printed* is exactly a statement about that order.
///
/// Why the order is the sanction, in one line each. Restoring and continuing is broken: a returning
/// app thread would paint frames into a terminal somebody else had restored. Printing before
/// restoring puts the message on an alt screen that is discarded a moment later. And panicking
/// instead of aborting unwinds the observer's own stack, which stops nothing at all — the app thread
/// is still inside the iteration that has not come back.
///
/// The child waits past `crate::perf::STALL_FLOOR`, so the whole gate costs a test-binary startup and
/// a little over a second. It is not shortened by pinning a tight budget, and that is the floor doing
/// its job rather than an inconvenience: a limit that could be driven down to 128 ms by a `Config`
/// field is a limit every other gate in this file would have been arming by accident.
#[test]
fn the_observers_sanction_restores_the_terminal_then_prints_the_stall_then_aborts() {
    use std::process::{Command, Stdio};

    if std::env::var(STALL_MODE).is_ok() {
        return stall_on_purpose();
    }
    let path = std::env::temp_dir().join(format!("vitui-observer-{}.out", std::process::id()));
    let file = std::fs::File::create(&path).expect("a file in the temp directory");
    let merged = file.try_clone().expect("the same open file, twice");
    let status = Command::new(std::env::current_exe().expect("the test binary's own path"))
        .args([
            "--exact",
            "gates::the_observers_sanction_restores_the_terminal_then_prints_the_stall_then_aborts",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(STALL_MODE, "1")
        .env("RUST_BACKTRACE", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::from(file))
        .stderr(Stdio::from(merged))
        .status()
        .expect("the test binary is runnable");
    let out = std::fs::read_to_string(&path).expect("what the child wrote");
    let _ = std::fs::remove_file(&path);

    assert_ne!(
        status.code(),
        Some(NEVER_FIRED),
        "the app thread stalled for five seconds and the observer never noticed: {}",
        escaped(&out)
    );
    assert_ne!(
        status.code(),
        Some(0),
        "the child returned normally, so nothing aborted: {}",
        escaped(&out)
    );
    let restored = out
        .find(EPILOGUE)
        .unwrap_or_else(|| panic!("the terminal was never given back: {}", escaped(&out)));
    let printed = out
        .find("has not come back")
        .unwrap_or_else(|| panic!("the stall was never reported: {}", escaped(&out)));
    assert!(
        restored < printed,
        "the stall was printed into the alt screen, before the restoration: {}",
        escaped(&out)
    );
    assert!(
        out[printed..].contains(STALL_REASON),
        "the report does not name the permit the stalled region was held under: {}",
        escaped(&out)
    );
    // The entry time, which is the number that identifies the frame — and it is *before* the phrase
    // above rather than after it, so the slice starts at the restoration. The limit is one second and
    // the poll is 100 ms, so the number is just over a second and is printed in seconds.
    assert!(
        out[restored..].contains(" s ago"),
        "the report does not say when the iteration entered: {}",
        escaped(&out)
    );
}

/// The child: take the terminal, paint a frame, enter an iteration, and never come back.
fn stall_on_purpose() {
    let engine = crate::engine::Engine::new(crate::engine::Config {
        size: (20, 4),
        output: crate::engine::Output::Sink(Box::new(std::io::stdout())),
        clock: crate::engine::Clock::System,
        max_frame_rate: f32::INFINITY,
        overrides: Overrides::default(),
        // Pinned tight so that nothing here is at the mercy of the default budget, and it does
        // **not** shorten the stall limit — `crate::perf::STALL_FLOOR` is a second whatever this
        // says. The numbers this gate is about are the order of three events, not the size of the
        // limit.
        overrun_threshold: Some(std::time::Duration::from_millis(2)),
        overrun_report: None,
        input: crate::input::InputConfig {
            mouse: crate::input::MouseMode::Buttons,
            focus: true,
            paste: true,
            ..crate::input::InputConfig::default()
        },
    });
    let (mut screen, wake) = engine
        .attach_restoring_into(Some(every_input_protocol()), Box::new(std::io::stdout()))
        .expect("attaching to a sink cannot fail");
    screen.set_mouse(crate::input::MouseMode::Motion);
    let id = screen.layers().add_content(0, Rect::new(0, 0, 20, 4), true);
    screen
        .layers()
        .view(id)
        .expect("the layer was just added")
        .text(0, 0, "on the alt screen", Style::new());
    // No `wait` before it, so this frame is not an iteration and the in-loop detector is silent.
    assert!(screen.present().submitted);

    wake.post();
    assert_eq!(screen.wait(), Wake::Posted);
    // A permit **annotates** the stall and does not excuse it: the in-loop rung is excused because
    // the application said *this will be slow*, and the observer answers a different claim.
    let _permit = screen.permit_slow(STALL_REASON);
    // Past the floor and its poll, and then some. The observer fires at a little over a second; five
    // is the ceiling at which *nothing happened* becomes a diagnosis rather than a hung job.
    std::thread::sleep(std::time::Duration::from_secs(5));
    // Unreachable unless the observer is missing, and then it is the diagnosis rather than a hang.
    std::process::exit(NEVER_FIRED);
}
