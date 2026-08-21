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
use crate::engine::Screen;
use crate::geom::Rect;
use crate::mix::Mix;
use crate::reference;
use crate::register::State;
use crate::restyle::Restyle;
use crate::scenes::{H, Scene, W, scenes, table_two_ways, virtualised_tree};
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
    // The scene says what it needs pinned about the terminal, and both drivers ask. Eleven of the
    // twelve need nothing; the twelfth is about an operator, and an operator layer is skipped
    // outright at the `ColorDepth::None` a headless screen otherwise has.
    let mut h = Harness::with_overrides(W, H, scene.overrides()).labelled(scene.name());
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

// ---------------------------------------------------------------------------------------------
// #11 — after a sweep every live cell resolves to the same channels.
// #7  — a settled operator allocates zero; a fading one allocates per distinct extended style.
// ---------------------------------------------------------------------------------------------

/// A screen whose cells are extended, on a tier that has colour, with **no round trip**.
///
/// # Why these four gates cannot use the harness, and what is lost
///
/// `Harness::present` closes the round trip on every frame, and **an extended cell cannot close
/// it**: SGR 58/59 and OSC 8 are impl 13's, so the serializer emits a hyperlinked, underline-coloured
/// cell in the right colours and drops the two channels that made it extended (`crate::serial`'s
/// module documentation says so). The terminal model then holds an *inline* cell where the frame
/// holds a handle, and comparing the two compares one screen against a different spelling of it.
///
/// Every gate here is about a **table**, and the table is the thing an extended cell is required
/// for: a sweep with nothing extended in the tables has nothing to reclaim. So the round trip is
/// not skipped for these scenes, it is unavailable to them, and impl 13 is what makes it available.
/// What is lost is stated rather than assumed: nothing below asserts that bytes reached a terminal,
/// only that the tables, the cells and the mirror's own bookkeeping agree.
///
/// Truecolor is pinned because an operator layer is skipped outright at [`ColorDepth::None`] (§5),
/// which is what a headless screen is unless something says otherwise (architecture ticket 22), and
/// the colours are explicit because a cell with a **default** background is left unmixed on a
/// terminal silent on OSC 11. Both traps make the fading gate below measure nothing rather than
/// fail.
fn extended_screen(rows: u16) -> Screen {
    crate::testing::screen_without_a_round_trip(8, rows, crate::testing::pinned_truecolor())
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
    let mut screen = extended_screen(ROWS);
    let id = extended_page(&mut screen, ROWS);
    underline_each_row(&mut screen, id, ROWS, |y| Color::rgb(y as u8 + 1, 0, 0));
    screen.present();
    assert_eq!(
        screen.table_lengths(),
        (1, ROWS as usize),
        "one cluster, and one extended style per row"
    );

    // Three of the four rows move onto a fifth style, orphaning entries 1, 2 and 3.
    {
        let mut view = screen.layers().view(id).expect("the layer is still there");
        view.restyle(
            Rect::new(0, 1, 8, ROWS - 1),
            &Restyle {
                ul: Some(Color::rgb(9, 9, 9)),
                ..Default::default()
            },
        );
    }
    screen.present();
    assert_eq!(screen.table_lengths(), (1, ROWS as usize + 1));

    let before = screen.channels();
    let swept = screen.sweep_now();
    let after = screen.channels();

    assert!(swept.renumbered, "entry 4 had three dead entries below it");
    assert_eq!(swept.freed_exts, 3);
    assert_eq!(swept.live_exts, 2);
    assert_eq!(swept.freed_clusters, 0, "the cluster is still on every row");
    assert_eq!(swept.live_clusters, 1);
    assert_eq!(
        screen.table_lengths(),
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
    let mut screen = extended_screen(ROWS);
    let id = screen
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
        let mut view = screen.layers().view(id).expect("just added");
        for (y, cluster) in WIDE.iter().enumerate() {
            view.text(0, y as i32, cluster, opaque_ink());
        }
    }
    screen.present();
    assert_eq!(screen.table_lengths().0, 4, "four interned clusters");

    // Rows 1..3 are overwritten with the cluster row 0 holds, orphaning ids 1, 2 and 3 — which are
    // above the only live one, so this is also the second half's case seen from the interner.
    {
        let mut view = screen.layers().view(id).expect("the layer is still there");
        for y in 1..ROWS as i32 {
            view.text(0, y, WIDE[0], opaque_ink());
        }
    }
    screen.present();

    let before = screen.channels();
    let swept = screen.sweep_now();
    assert_eq!(swept.freed_clusters, 3);
    assert_eq!(swept.live_clusters, 1);
    assert!(
        !swept.renumbered,
        "ids 1..3 were all above the only live one"
    );
    assert_eq!(screen.table_lengths().0, 1);
    assert_eq!(before, screen.channels());

    // The wide flag survived, which is what says the rewrite rebuilt the handle rather than the id.
    let frame = screen.frame();
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
    let mut screen = extended_screen(ROWS);
    let id = extended_page(&mut screen, ROWS);
    underline_each_row(&mut screen, id, ROWS, |y| Color::rgb(y as u8 + 1, 0, 0));
    screen.present();
    assert_eq!(screen.table_lengths(), (1, ROWS as usize));

    // Rows 2 and 3 take rows 0 and 1's styles, which the table already holds. Entries 2 and 3 go
    // dead and every survivor keeps its handle.
    underline_each_row(&mut screen, id, ROWS, |y| Color::rgb(y as u8 % 2 + 1, 0, 0));
    screen.present();
    assert_eq!(
        screen.table_lengths(),
        (1, ROWS as usize),
        "nothing new was minted; two entries simply stopped being pointed at"
    );

    let before = screen.channels();
    let swept = screen.sweep_now();
    assert!(
        !swept.renumbered,
        "the two freed entries were above every live one"
    );
    assert_eq!(swept.freed_exts, 2);
    assert_eq!(swept.live_exts, 2);
    assert_eq!(screen.table_lengths(), (1, 2), "and they were still freed");
    assert_eq!(before, screen.channels());
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
    let mut screen = extended_screen(ROWS);
    let id = extended_page(&mut screen, ROWS);
    underline_each_row(&mut screen, id, ROWS, |y| Color::rgb(y as u8 + 1, 0, 0));
    screen.present();
    assert_eq!(
        screen.known_rows(),
        ROWS as usize,
        "the birth frame wrote every row whole"
    );

    {
        let mut view = screen.layers().view(id).expect("the layer is still there");
        view.restyle(
            Rect::new(0, 1, 8, ROWS - 1),
            &Restyle {
                ul: Some(Color::rgb(9, 9, 9)),
                ..Default::default()
            },
        );
    }
    screen.present();
    assert!(screen.sweep_now().renumbered);

    // Nothing is damaged, so this submits nothing — and the flag has to survive it. A flag spent on
    // a frame that never went out is a mirror that is never told.
    assert!(!screen.present().submitted);
    assert_eq!(
        screen.known_rows(),
        ROWS as usize,
        "an idle present must not spend the flag"
    );

    // The next real frame is the one that carries it. Every row of it is written whole, so the rows
    // are unknown and known again inside one `present` — which is exactly what *costing one full
    // frame on the render thread* means.
    underline_each_row(&mut screen, id, ROWS, |_| Color::rgb(1, 2, 3));
    assert!(screen.present().submitted);
    assert_eq!(screen.known_rows(), ROWS as usize);

    // A narrow frame after a renumbering sweep leaves the rows it did not write whole unknown, which
    // is the half ADR 0006's *written whole rather than compared* does not reach: the sweep marks no
    // damage, so there is no whole row to write. `crate::serial::Mirror` states what ticket 14's
    // filter has to do about it.
    underline_each_row(&mut screen, id, ROWS, |y| Color::rgb(y as u8 + 1, 0, 0));
    screen.present();
    {
        let mut view = screen.layers().view(id).expect("the layer is still there");
        view.restyle(
            Rect::new(0, 1, 8, ROWS - 1),
            &Restyle {
                ul: Some(Color::rgb(7, 7, 7)),
                ..Default::default()
            },
        );
    }
    screen.present();
    assert!(screen.sweep_now().renumbered);
    {
        let mut view = screen.layers().view(id).expect("the layer is still there");
        view.text(3, 0, "z", opaque_ink());
    }
    assert!(screen.present().submitted);
    assert_eq!(
        screen.known_rows(),
        0,
        "one cell of one row is not one whole row of four"
    );
}

/// A sweep that renumbers nothing may not invalidate a mirror row.
#[test]
fn a_sweep_that_renumbers_nothing_leaves_the_mirror_alone() {
    const ROWS: u16 = 4;
    let mut screen = extended_screen(ROWS);
    let id = extended_page(&mut screen, ROWS);
    underline_each_row(&mut screen, id, ROWS, |y| Color::rgb(y as u8 + 1, 0, 0));
    screen.present();
    underline_each_row(&mut screen, id, ROWS, |y| Color::rgb(y as u8 % 2 + 1, 0, 0));
    screen.present();
    assert_eq!(screen.known_rows(), ROWS as usize);

    let swept = screen.sweep_now();
    assert!(!swept.renumbered);
    assert_eq!(swept.freed_exts, 2, "it did reclaim, it just did not move");

    // A frame that damages one cell of one row. Were the flag set, every row would go unknown.
    {
        let mut view = screen.layers().view(id).expect("the layer is still there");
        view.text(3, 0, "z", opaque_ink());
    }
    assert!(screen.present().submitted);
    assert_eq!(screen.known_rows(), ROWS as usize);
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
    let mut screen = extended_screen(4);
    let id = extended_page(&mut screen, 4);
    screen.present();

    // Past the floor: one new distinct extended style per frame until the mark is reached. Two
    // colour channels rather than one, because the floor is 256 entries and one byte would run out
    // of distinct values at exactly the wrong moment.
    let mut distinct = 0u32;
    while !screen.sweep_due() {
        distinct += 1;
        assert!(distinct < 4096, "the high-water mark was never reached");
        {
            let mut view = screen.layers().view(id).expect("the layer is still there");
            view.restyle(
                Rect::new(0, 0, 8, 1),
                &Restyle {
                    ul: Some(Color::rgb(distinct as u8, (distinct >> 8) as u8, 1)),
                    ..Default::default()
                },
            );
        }
        screen.present();
    }

    // `layers()` is the door, and it has not been opened since the mark was reached.
    let before = screen.sweeps();
    for _ in 0..4 {
        screen.present();
    }
    assert!(screen.sweep_due(), "the mark is still reached");
    assert_eq!(
        screen.sweeps(),
        before,
        "`present` swept, and §13's budget for the sweep rests on the promise that it cannot"
    );

    // And the door does open, exactly once, and moves the mark past what is live.
    screen.layers();
    assert_eq!(screen.sweeps(), before + 1);
    assert!(
        !screen.sweep_due(),
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
    let mut screen = extended_screen(ROWS);
    let id = extended_page(&mut screen, ROWS);

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
        .map(|i| screen.link(&format!("https://example.com/{i}")))
        .collect();
    {
        let mut view = screen.layers().view(id).expect("the layer is still there");
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
    screen.present();
    let page = screen.table_lengths().1;
    assert_eq!(page, distinct, "the page itself holds four extended styles");

    let before_any_operator = screen.frame().row(0)[1];
    let mut op =
        screen
            .layers()
            .add_operator(1, Rect::new(0, 0, 8, ROWS), Mix::darken(Mix::FULL / 4));
    screen.present();
    // **Assert the operator moved a cell before asserting anything about which ones.** A headless
    // screen is at `ColorDepth::None` unless something pins it, and there an operator layer is
    // skipped outright — a fade that touched nothing would report a beautiful zero growth and mean
    // nothing at all.
    assert_ne!(
        screen.frame().row(0)[1],
        before_any_operator,
        "the operator changed no cell, so this gate is measuring the depth and not the fade"
    );

    // The settled shape first, because the fading number means nothing without it: the same `amount`
    // again mints nothing at all.
    let settled = screen.table_lengths().1;
    screen.present();
    assert_eq!(
        screen.table_lengths().1,
        settled,
        "a settled operator converges after one frame"
    );

    // Ten frames of a fade, each with an `amount` no earlier frame used.
    let mut minted = Vec::new();
    for step in 1..=10u16 {
        let was = screen.table_lengths().1;
        screen.layers().remove(op);
        op = screen.layers().add_operator(
            1,
            Rect::new(0, 0, 8, ROWS),
            Mix::darken(Mix::FULL / 4 + step * 8),
        );
        screen.present();
        minted.push(screen.table_lengths().1 - was);
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
    assert_eq!(screen.table_lengths().1, page + distinct * 11);
    let swept = screen.sweep_now();
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
    assert_eq!(screen.table_lengths().1, distinct * 2);
}
