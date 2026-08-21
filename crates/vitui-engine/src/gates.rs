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

use crate::damage::Run;
use crate::geom::Rect;
use crate::mix::Mix;
use crate::reference;
use crate::register::State;
use crate::scenes::{H, Scene, W, scenes, table_two_ways, virtualised_tree};
use crate::style::Style;
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
    let mut h = Harness::new(W, H).labelled(scene.name());
    scene.build(&mut h.screen);
    h.present();
    h
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
/// encoding, and the encoding is exactly the part tickets 13, 14 and 15 are going to change.
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
/// the numbers are meant to fall, because tickets 13, 14 and 15 exist to make every one of them
/// smaller.
///
/// Moving a number up is allowed and costs a commit that does three things: states the new number,
/// states the measurement it came from, and replaces the provenance line below. What may **not**
/// move without a new map decision is a budget figure, and none of these is one.
///
/// Provenance: measured by **impl 04** on 2026-08-20, Apple M1 Max, rustc 1.97.1, over
/// [`FRAMES`] steady frames after the birth frame, at 300x80, with no equality filter (impl 14),
/// no `shortest` cursor encoding (impl 13) and no scroll region (impl 15) in the serializer yet.
///
/// Two of these are the reason the gate exists. **`every-cell-a-distinct-style` is 1.3 MB for three
/// frames** — the adversarial page, which is what set the synchronised-output time limit — and
/// **`sparse-chart-400-points` is 11 417 bytes for 1 200 cells**, which is the ratio a serializer
/// walking the grid instead of the runs would blow up by 284x without changing a pixel.
const WIRE_BUDGET: [(&str, usize); 11] = [
    ("caret-blink", 43),
    ("scrolling-list-label-only", 6_465),
    ("scrolling-list-rows-cleared", 73_665),
    ("twenty-popups-with-shadows", 30_648),
    ("three-dialogs-apart", 6_870),
    ("sparse-chart-400-points", 11_417),
    ("progress-bar-one-percent", 336),
    ("virtualised-tree", 11_265),
    ("table-as-list-and-bar-chart", 20_598),
    ("full-screen-change", 73_680),
    ("every-cell-a-distinct-style", 1_296_592),
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
             The bound is impl 04's own measurement, so this is a regression rather than drift."
        );
    }
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

    let red = all
        .iter()
        .filter(|s| !matches!(s.status(), State::Wired { .. }))
        .count();
    assert_eq!(
        red, 1,
        "one scene has no measurement behind it — the hyperlinked page under an animating \
         operator — and spec §14 says so. A second red scene is either a mechanism that regressed \
         or a scene that was added without one."
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
