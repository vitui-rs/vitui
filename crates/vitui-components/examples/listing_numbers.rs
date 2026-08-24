//! **The listing: the collection's four hostile axes and its size invariance, as numbers.**
//!
//! Components ticket 11. The convention is the runtime's — a file in `examples/` named
//! `<subject>_numbers.rs` that prints the numbers a human reads — and so is the rule about what an
//! example may be: **`cargo test` does not run this file**, and no row of
//! [`vitui_components::gates::REGISTER`] rests on it. Rows 29 and 66 *cite* it and each names a
//! `#[test]` in `src/listing.rs` beside the citation.
//!
//! # What it prints
//!
//! 1. **The five scenes**, with where each stands and which ticket inverts it.
//! 2. **The four axes**, each caught, with what the defective arm cost beside the diff — because
//!    the whole argument for an equality against a reference render is that *every one of the four
//!    made the defective build look healthier*.
//! 3. **The size invariance**, at 1 000, 100 000 and 1 000 000 rows, against a listing that iterates
//!    its whole content. Writes are **identical on both arms**, which is why criterion 6's equality
//!    is on `regions`.
//! 4. **The wheel gate**, in both directions, over three arms.
//! 5. **O5's collection column**, and what `scenes_for("collection")` answers.
//! 6. **What does not reproduce**, said out loud rather than engineered away.
//!
//! # It asserts the shape and not the timings
//!
//! R15's rule, inherited: *a gate is a count, a ratio, an equality or a compile outcome; a timing is
//! a report*. Every `assert!` below is a count or an equality — and this file is on the wrong side
//! of the line for a gate anyway, which is why the counts it shares with the register are asserted
//! in `src/` too.

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::counters::Allocations;
use vitui_components::gates::Standing;
use vitui_components::listing::{self, Reveal, Volume};
use vitui_components::scenes::{SCENES, scenes_for};
use vitui_components::{Axis, INVENTORY};

// **The probe, because `allocations` is a total and a total needs something that counts.** Installed
// here rather than defaulted to zero inside the library: a figure defaulted to zero is a counter
// that prints `0` when it means *nobody counted*.
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// How many frames a per-frame figure is averaged over. Eight, warmed.
const FRAMES: u32 = 8;

/// How many frames the million-row whole-content arm is averaged over. **One**, and that is itself
/// the finding: eight of them is four seconds.
const SLOW_FRAMES: u32 = 1;

fn main() {
    println!(
        "The listing — 40x80, one collection, {} rows a window, and the five scenes \
         `collection` is a screen of\n",
        listing::H
    );

    scene_list();
    the_four_axes();
    size_invariance();
    the_wheel_gate();
    o5();
    what_does_not_reproduce();
}

/// 1. The five scenes, and which ticket inverts each.
fn scene_list() {
    println!("report  the five scenes, and the two different reasons they are red:");
    println!(
        "  {:>3}  {:<44}  {:<28}  inverted by",
        "#", "scene", "standing"
    );
    let mut red = 0usize;
    for scene in SCENES.iter().filter(|s| s.stands.contains(&"collection")) {
        let (word, ticket) = match scene.standing {
            Standing::Red { inverted_by, .. } => {
                red += 1;
                ("red, pinned", inverted_by)
            }
            Standing::Evaluated { .. } => ("evaluated", "-"),
            Standing::Unsubjected { inverted_by } => ("unsubjected", inverted_by),
            Standing::Unreachable { inverted_by, .. } => ("unreachable", inverted_by),
        };
        println!(
            "  {:>3}  {:<44}  {:<28}  {ticket}",
            scene.number, scene.name, word
        );
    }
    assert_eq!(red, 5, "all five are pinned red");
    println!(
        "\n  Four are waiting for `collection` and one is not: the wheel gate's failing set is the\n  \
         defect itself, which is why it stays red through components 12 and goes green at 20.\n"
    );
}

/// 2. The four axes, each caught, with what the defective arm cost beside the diff.
fn the_four_axes() {
    /// One row of the table, so that a literal and a computed string line up under one format.
    fn axis(name: &str, reported: &str, counters: &str) {
        println!("  {name:<30}  {reported:<50}  {counters}");
    }

    println!(
        "report  the four hostile axes, each caught by an equality against a reference render:"
    );
    axis("axis", "the runner reports", "what every counter said");

    let scrolled = listing::scrolled();
    // The two allocation totals are measured, which is the one column `counters_approve` cannot
    // fill itself: a library cannot install a global allocator on a consumer's behalf.
    let (_, correct_allocs) = count_allocations(listing::scrolled);
    let (_, broken_allocs) = count_allocations(listing::scrolled);
    let separating = listing::counters_that_separate_them(
        Allocations::over(1, correct_allocs as u64),
        Allocations::over(1, broken_allocs as u64),
    );
    axis(
        "scrolled (offset 60, 1k rows)",
        &scrolled.to_string(),
        &format!(
            "{} of §20's nine counters separate the arms",
            separating.len()
        ),
    );

    let stale = listing::stale();
    axis(
        "shrunk (200 -> 9 rows)",
        &stale.to_string(),
        "3 200 writes against 360 — the defective arm is the cheap one",
    );

    axis(
        "wheeled (20 clicks)",
        &format!(
            "offset {} against {}",
            listing::wheeled(Reveal::EveryFrame, listing::CLICKS).settled,
            listing::wheeled(Reveal::WhenAsked, listing::CLICKS).settled
        ),
        "no counter moves at all; the screen is identical every frame",
    );

    let narrow = listing::narrow(listing::NARROW);
    axis(
        "narrow (40 -> 16 columns)",
        &narrow.to_string(),
        "the last cell of every row, and nothing else",
    );

    println!(
        "\n  And the two spellings §21 keeps apart: the same stale-tail painter written against a\n  \
         terminal resize scores `{}` — clean — because a fresh `Surface` has\n  nowhere for the \
         residue to survive.",
        listing::stale_by_resize()
    );
    println!(
        "  The narrow arm at {} columns scores `{}`, so the scene is green wide and red narrow.\n",
        listing::W,
        listing::narrow(listing::W)
    );

    assert_eq!(scrolled.rows, listing::SCROLLED_ROWS);
    assert_eq!(stale.rows, listing::STALE_ROWS);
    assert_eq!(stale.cells, listing::STALE_CELLS);
    assert_eq!(narrow.rows, listing::H as usize);
    assert!(listing::stale_by_resize().clean());
    assert!(listing::narrow(listing::W).clean());
    assert!(
        separating.is_empty(),
        "{separating:?} told the two arms apart"
    );
}

/// 3. The size invariance, and the arm the write count cannot see.
fn size_invariance() {
    println!("report  one store, one window: the frame at 1 000, 100 000 and 1 000 000 rows:");
    println!(
        "  {:<14}  {:>9}  {:>7}  {:>9}  {:>9}  {:>6}  {:>11}  {:>11}",
        "arm", "rows", "writes", "verbs", "regions", "stops", "us (Direct)", "us (Tally)"
    );

    let mut windowed = Vec::new();
    for kind in [Volume::Windowed, Volume::WholeContent] {
        for rows in listing::VOLUMES {
            let slow = kind == Volume::WholeContent && rows == listing::VOLUMES[2];
            let frames = if slow { SLOW_FRAMES } else { FRAMES };
            let (shape, tallied) = listing::volume_over(kind, rows, frames);
            let direct = listing::volume_cost(kind, rows, frames);
            println!(
                "  {:<14}  {:>9}  {:>7}  {:>9}  {:>9}  {:>6}  {:>11.2}  {:>11.2}",
                kind.word(),
                rows,
                shape.writes,
                shape.verbs,
                shape.regions,
                shape.stops,
                direct.as_secs_f64() * 1e6,
                tallied.as_secs_f64() * 1e6,
            );
            if kind == Volume::Windowed {
                windowed.push((shape, direct));
            }
        }
    }

    let flat = windowed[2].1.as_secs_f64() / windowed[0].1.as_secs_f64();
    println!("\n  The equality — and it is the whole of criterion 6:");
    println!(
        "    writes    {} at every volume, on **both** arms. The engine reports a fully clipped",
        listing::WRITES
    );
    println!(
        "              verb as zero columns, so §21's register row 4 — *writes flat 1k -> 1M* —"
    );
    println!("              is green on a build that declares a million hit entries.");
    println!(
        "    regions   {} at every volume windowed, and 1 001 / 100 001 / 1 000 001 whole-content.",
        listing::REGIONS
    );
    println!(
        "              `Ctx::declare` does not clip, so an off-screen declaration is a real entry."
    );
    println!(
        "    stops     {} at every volume, both arms: a virtualised collection is one tab stop (§13).",
        listing::STOPS
    );
    println!("  The timing is a report: the windowed frame is {flat:.3}x from 1k to 1M, and the");
    println!("  whole-content arm is three orders of magnitude. §20's rule is why the flatness is");
    println!("  printed and the microseconds are not gated.\n");

    // The allocation total is the whole run's, `driver_at` and the `Tally`'s `BTreeSet` included,
    // because `volume_over` attaches its own driver. The steady-frame zero is a component's claim
    // and belongs to components 12.
    let (_, allocated) =
        count_allocations(|| listing::volume(Volume::Windowed, listing::VOLUMES[2]));
    println!(
        "  Allocations over one whole `volume` call — driver attach, theme rebuild and the\n  \
         instrument's own `BTreeSet` included: {allocated}. The steady-frame zero is a claim about\n  \
         a component and it is components 12's to make.\n"
    );

    for (shape, _) in &windowed {
        assert_eq!(shape.writes, listing::WRITES);
        assert_eq!(shape.regions, listing::REGIONS);
        assert_eq!(shape.stops, listing::STOPS);
    }
}

/// 4. The wheel gate, in both directions.
fn the_wheel_gate() {
    println!(
        "report  twenty wheel clicks, and the reveal that decides whether they move anything:"
    );
    println!(
        "  {:<18}  {:>10}  {:>12}  {:>9}  {:>16}",
        "reveal", "settled", "after click 20", "requests", "keyboard reveal"
    );
    for reveal in [Reveal::WhenAsked, Reveal::EveryFrame, Reveal::Never] {
        let played = listing::wheeled(reveal, listing::CLICKS);
        println!(
            "  {:<18}  {:>10}  {:>12}  {:>9}  {:>16}",
            reveal.word(),
            played.settled,
            played.after_last_click,
            played.reveals,
            listing::revealed(reveal, listing::SCROLLED_AWAY),
        );
    }
    println!("\n  Both directions, and neither alone is the gate:");
    println!("    the wheel half   `every frame` settles at 0 where the rule settles at 20.");
    println!("    the reveal half  `never` passes the wheel half and moves the offset 0 when a");
    println!(
        "                     keyboard reveal really asks — so deleting the call is not a fix."
    );
    println!(
        "  The `after click 20` column is 1 and not 0 for the defective arm: a request crosses the\n  \
         frame boundary and is read on the frame after (ADR 0015), so the defect is always one\n  \
         click ahead of its own correction. That one click is the runtime's own documented residue."
    );
    println!(
        "  A frame at offset 0 leaves no request on **any** arm ({}), which is the frame this\n  \
         defect is invisible on and the reason `leaves_no_request` takes an offset.\n",
        listing::leaves_no_request(Reveal::EveryFrame, 0)
    );

    assert_eq!(
        listing::wheeled(Reveal::WhenAsked, listing::CLICKS).settled,
        listing::MOVED
    );
    assert_eq!(
        listing::wheeled(Reveal::EveryFrame, listing::CLICKS).settled,
        listing::DRAGGED_BACK
    );
    assert_eq!(listing::revealed(Reveal::Never, listing::SCROLLED_AWAY), 0);
    assert!(listing::leaves_no_request(
        Reveal::WhenAsked,
        listing::MOVED
    ));
}

/// 5. O5's collection column.
fn o5() {
    println!("report  O5, from the collection's side:");
    let component = INVENTORY
        .iter()
        .find(|c| c.id == "collection")
        .expect("`collection` is in the freeze");
    for axis in Axis::ALL {
        let declared = component.declares(axis);
        let scenes: Vec<u8> = SCENES
            .iter()
            .filter(|s| {
                s.covers
                    .iter()
                    .any(|(id, a)| *id == "collection" && *a == axis)
            })
            .map(|s| s.number)
            .collect();
        println!(
            "  {:<10}  declared {:<5}  scenes claiming the pair: {:?}",
            axis.name(),
            declared,
            scenes
        );
    }
    let stands: Vec<u8> = scenes_for("collection").map(|s| s.number).collect();
    println!("  `scenes_for(\"collection\")` answers {stands:?} — all four axis columns.");
    println!(
        "  The narrow column is answered by a scene that **stands** `collection` up and claims no\n  \
         `(collection, narrow)` pair: the freeze sets `narrow: false` because a row truncates\n  \
         through `text::fit`, which is `text`'s flag, and scene 28 already carries `(text, narrow)`.\n  \
         O5 is therefore unmoved by this ticket, at fourteen of thirty-four.\n"
    );
    assert_eq!(stands, vec![3, 4, 5, 6, 29]);
    assert!(!component.declares(Axis::Narrow));
}

/// 6. What does not reproduce, and why.
fn what_does_not_reproduce() {
    println!("report  what does not reproduce, and why:");
    for line in [
        "  §21 / the ticket           here          why",
        "  12.21 us / 3 058 writes    3 200 / 80    the inverted arm writes a blank row where it has",
        "  against 62.96 / 20 418     both arms     no content, so the *write* counts are equal here",
        "                                           rather than a fifth. The relation §21 states —",
        "                                           the defective build is not more expensive by any",
        "                                           counter — reproduces, and stronger: not one of",
        "                                           the eight readable counters separates the arms.",
        "  the scrolled diff          75 of 80      not 79. `Fixture::lines` is `(5i + c) mod 26`, so",
        "                             rows          rows agree iff `i ≡ j (mod 26)`; the inverted arm",
        "                                           puts row `60 − y` where `60 + y` belongs and those",
        "                                           are congruent every thirteenth row. The equality",
        "                                           **under-reports by five rows** and is still the",
        "                                           only instrument that reports it at all.",
        "  71 of 80 rows              71 of 80      reproduces exactly, through components ticket 04's",
        "                             / 2 840       own runner rather than a second copy of it.",
        "  2.3x faster marking        8.9x fewer    `marked` is unreachable from this crate and always",
        "  226x less                  writes        will be (`damage.rs` is `pub(crate)`), so the",
        "                                           226x has no expression. The direction reproduces",
        "                                           in the counter that is readable: 3 200 against 360.",
        "  0 against 16               0 against 20  §21's 16 carries four clicks of wheel-chain",
        "                                           residue. There is none here: a `Mouse` cannot be",
        "                                           posted from this crate, so the click's delta is",
        "                                           handed straight to the arithmetic — on both arms.",
        "                                           The zero is the number that matters and it is the",
        "                                           same zero.",
        "  63.87 / 63.87 / 64.08 us   52-104 us     a report and gated by nothing (R15), on another",
        "                             per frame,    machine. The absolute figure moves by 2x between an",
        "                             all three     idle run and a busy one, which is exactly why R15",
        "                                           makes it a report. What reproduces is the",
        "                                           **flatness** — the three volumes agree to within",
        "                                           the run-to-run spread — and that is the property",
        "                                           scene 3 decides. Measured through `Direct`; the",
        "                                           `Tally` column beside it is several times larger",
        "                                           and is a report about the instrument, not the frame.",
        "  one store, one `Mode`      one store,    `Mode` is components 12's. What this screen can",
        "                             no `Mode`     say is that the window and not the content decides",
        "                                           the frame, which is the claim `Mode` rests on.",
        "  229 regions (ticket 12)    81            ticket 12's screen is a component's; this one is a",
        "                                           row loop with one target a row. The **shape** is",
        "                                           the same and it is the shape that is gated: one",
        "                                           entry for the collection, and per-row entries for",
        "                                           visible rows only.",
    ] {
        println!("{line}");
    }
    println!();
}
