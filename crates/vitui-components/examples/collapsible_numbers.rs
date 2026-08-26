//! **The accordion and the fold set: the two scenes `collapsible` is a screen of, as numbers.**
//!
//! Components tickets 21 and 22. The convention is the runtime's — a file in `examples/` named
//! `<subject>_numbers.rs` that prints the numbers a human reads — and so is the rule about what an
//! example may be: **`cargo test` does not run this file**, and no row of
//! [`vitui_components::gates::REGISTER`] rests on it alone. The rows that cite it each name a
//! `#[test]` in `src/accordion.rs` beside the citation, because
//! `examples/gates_numbers.rs` once asserted `REGISTER.len() == 47` against a register that had
//! reached 56 and passed `cargo test` throughout.
//!
//! # What it prints
//!
//! 1. **The two scenes**, with where each stands — both `Evaluated` since components ticket 22, and
//!    through the subject: `accordion::draw_into` calls `disclose::collapsible_into`.
//! 2. **The accordion at 0 / 6 / 12 open**, measured, with §8's own column beside it — because the
//!    two are not the same screen and printing one of them would be a claim rather than a report.
//! 3. **Closed against `h = 0`**, which is the scene: identical surfaces, 408 more entries on the
//!    hit index and 408 more on the ring, and which of §20's nine counters can see it.
//! 4. **The transition**, section by section of the height, where the amplitude is 26.
//! 5. **The collapse**, frame by frame at sixty hertz, and why §8's *14 frames* is a cadence.
//! 6. **The height**, both of §8's spellings, and §9's own precondition on the other axis.
//! 7. **The focus**, and the one arm where R08's vanish rule answers at all.
//! 8. **The inplace map**, both spellings, round-tripped over a thousand rows.
//! 5. **The fold anchor**, 4 166 of 4 167 against 0 of 4 167, with the reanchor's cost beside the
//!    document edit's.
//! 6. **O5's `collapsible` column**, and what `scenes_for("collapsible")` answers.
//! 7. **What does not reproduce**, said out loud rather than engineered away.
//!
//! # It asserts the shape and not the timings
//!
//! R15's rule, inherited: *a gate is a count, a ratio, an equality or a compile outcome; a timing is
//! a report*. Every `assert!` below is a count or an equality.

use std::time::Duration;

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::accordion::{
    self, Anchor, BODY_ROWS, Body, CHROME_ENTRIES, CHROME_STOPS, Keep, Screen, admitted,
};
use vitui_components::counters::Allocations;
use vitui_components::disclose::{self, DiscloseOpts, Height, Inplace};
use vitui_components::gates::Standing;
use vitui_components::ink::Direct;
use vitui_components::scenes::{SCENES, scenes_for};
use vitui_components::{Axis, INVENTORY};

// **The probe, because `allocations` is a total and a total needs something that counts.** Installed
// here rather than defaulted to zero inside the library: a figure defaulted to zero is a counter
// that prints `0` when it means *nobody counted*.
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// How many single frames a per-frame figure is the **minimum** over. See `accordion::cost`: a mean
/// over eight read 1 621 µs for a frame that does strictly less work than one that read 555.
const FRAMES: u32 = 64;

/// How many frames the allocation total is a total *over*. §19's rule: a mean cannot see anything
/// below `n`, so the figure is a total and the frame count is printed beside it.
const ALLOC_FRAMES: u32 = 200;

/// How many repeats the two one-shot fold figures are a minimum over.
const REPEATS: u32 = 5;

/// How many frames the driver is warmed for before an allocation window opens. **Four**, because the
/// five frame structures take their allocation on the first frame that needs one and keep it.
const WARM: u32 = 4;

fn main() {
    println!(
        "The accordion — {}x{}, twelve sections, {} cells of body each, and the fold set beside \
         it\n",
        accordion::W,
        accordion::H,
        accordion::BODY_ROWS
    );

    scene_list();
    the_machine();
    the_frame();
    closed_against_zero_height();
    the_transition();
    the_collapse();
    the_height();
    the_focus();
    the_inplace_map();
    the_fold_anchor();
    o5();
    what_does_not_reproduce();
}

/// 2. **The machine**: §8's three-row split, and what a `Collapse` costs.
fn the_machine() {
    println!("report  §8's split — one machine, three configurations:");
    println!(
        "  {:<30}  {:<28}  {:<16}  examples",
        "the collapsed content is", "who collapses it", "spelling"
    );
    for row in disclose::SPLIT {
        println!(
            "  {:<30}  {:<28}  {:<16}  {}",
            row.content,
            row.who.word(),
            row.who.collapses().word(),
            row.examples.join(", ")
        );
    }
    println!(
        "\n  a `Collapse` costs {} bytes live, {} for the tween slot and {} of padding = {} — \
         against §8's {} and {}",
        disclose::LIVE_BYTES,
        disclose::TWEEN_BYTES,
        disclose::PADDING_BYTES,
        disclose::COLLAPSE_BYTES,
        disclose::SPEC_LIVE_BYTES,
        disclose::SPEC_TWEEN_BYTES,
    );
    println!(
        "  and the slot costs its {} bytes empty, which is why the two cannot both be a size_of \
         of one type",
        disclose::TWEEN_BYTES
    );
    println!(
        "  never per row of content: {} folds cost {} bytes of line numbers and twelve sections \
         cost {}\n",
        accordion::FOLDS,
        size_of::<u32>() * accordion::FOLDS,
        disclose::COLLAPSE_BYTES * accordion::SECTIONS,
    );
    assert_eq!(disclose::SPLIT.len(), 3);
    assert_ne!(disclose::LIVE_BYTES, disclose::SPEC_LIVE_BYTES);
}

/// 5. **The collapse**, frame by frame at sixty hertz, and §8's *14 frames* beside it.
fn the_collapse() {
    let run = accordion::a_collapse(Duration::from_millis(200), SIXTY_HERTZ);
    println!(
        "report  a 200 ms collapse at 60 Hz: {} frames to quiet against §8's {}, worst {:.2} us \
         against {:.2}, {} cells against {}",
        run.frames,
        accordion::SPEC_TRANSITION_FRAMES,
        run.worst.as_secs_f64() * 1e6,
        accordion::SPEC_TRANSITION_US,
        run.cells,
        accordion::SPEC_TRANSITION_CELLS,
    );
    println!("  the heights, one a frame: {:?}", run.heights);
    // **The count is a cadence and the relation is the gate.** Halving the step doubles the frames.
    let twice = accordion::a_collapse(Duration::from_millis(200), SIXTY_HERTZ / 2);
    println!(
        "  at 120 Hz it is {} frames, which is what makes §8's number a cadence rather than a \
         count\n",
        twice.frames
    );
    assert_eq!(twice.frames, run.frames * 2);
    assert_eq!(run.heights.last(), Some(&0));

    // **The allocation total over the transition's own frames**, with the driver warm — §8 claims
    // zero for the collapse and the driver's own attachment is not part of it. A total and not a
    // mean, for `crate::counters::Allocations`'s reason: a mean over `n` cannot see anything below
    // `n`, so a frame that allocates once in twelve reports 0.
    let mut live = accordion::Live::new(1, DiscloseOpts::default());
    let mut ink = Direct;
    for _ in 0..WARM {
        live.frame(&mut ink);
    }
    // **A whole collapse as the warm-up, and then the one that is measured.** Four steady frames are
    // not enough: every height between `BODY_ROWS` and zero is a rectangle the frame's structures
    // have not been asked for yet, and the first of them takes its allocation and keeps it. Warming
    // on the *shape* rather than on the frame count is what makes the window a claim about the
    // steady state — `crate::runner`'s rule, one collapse wide.
    live.collapse(0, Duration::from_millis(200));
    while live.animating() > 0 {
        live.advance(SIXTY_HERTZ);
        live.frame(&mut ink);
    }
    live.reopen(0, BODY_ROWS);
    live.frame(&mut ink);
    live.collapse(0, Duration::from_millis(200));
    let (frames, allocs) = count_allocations(|| {
        let mut n = 0u32;
        while live.animating() > 0 {
            live.advance(SIXTY_HERTZ);
            live.frame(&mut ink);
            n += 1;
        }
        n
    });
    println!(
        "  allocations over the {frames} frames of the collapse, driver warm: {allocs} — a TOTAL \
         and not a mean, and §8 claims 0\n"
    );
    assert_eq!(frames, run.frames, "the same collapse, driven by hand");
}

/// The cadence the collapse is reported at. Sixty hertz, `scripts/steady-report.sh`'s own rate.
const SIXTY_HERTZ: Duration = Duration::from_micros(16_667);

/// 6. **The height**: a sizing function against a measured extent, which is §9's finding on the
///    other axis.
fn the_height() {
    println!("report  §8's two spellings of the open height:");
    println!("  {:<22}  {:<44}  rows drawn", "spelling", "what it is");
    println!(
        "  {:<22}  {:<44}  {}",
        Height::Sized.word(),
        "the argument beside the body",
        disclose::CONTENT_ROWS
    );
    println!(
        "  {:<22}  {:<44}  {}  <- latched",
        Height::Watermark.word(),
        "measured inside the rectangle it produced",
        disclose::WATERMARK_LATCHED_ROWS
    );
    println!(
        "\n  §8 states {} cells / {} rows wrong / 3 frames for the watermark against {} / 0 / 2 \
         for the sizing function, and those are a prototype's *body*. What reproduces is §9's own \
         precondition: over a body that fills what it is handed the measurement never comes down, \
         and over one that draws only its content the two arms are indistinguishable.",
        disclose::SPEC_WATERMARK_CELLS,
        disclose::SPEC_WATERMARK_ROWS_WRONG,
        disclose::SPEC_SIZED_CELLS,
    );
    println!(
        "  §8 prices it at +{:.1}% of the frame; here it is one extra pass over the body, which a \
         verb count reads.\n",
        disclose::SPEC_WATERMARK_OVERHEAD * 100.0
    );
    const { assert!(disclose::WATERMARK_LATCHED_ROWS > disclose::CONTENT_ROWS) };
}

/// 7. **The focus**, and the one arm where the vanish rule answers at all.
fn the_focus() {
    let (kept, on_a_header) = accordion::collapse_all(Keep::TheHeader);
    let (dropped, landed) = accordion::collapse_all(Keep::Nothing);
    println!(
        "report  §8's 0 ring probes against {}:",
        disclose::SPEC_VANISH_PROBES
    );
    println!(
        "  {:<34}  {:>7}  focus lands on a header",
        "a collapse with no gesture", "probes"
    );
    println!("  {:<34}  {kept:>7}  {on_a_header}", Keep::TheHeader.word());
    println!("  {:<34}  {dropped:>7}  {landed}", Keep::Nothing.word());
    println!(
        "\n  and no *gesture* reaches the rule at all: a click on a focusable header is awarded \
         the focus, a click on one that is not a tab stop **defocuses**, and `Enter` needs the \
         header to hold the focus already. What the rule buys is the keyboard, not the probe \
         count.\n"
    );
    assert_eq!(kept, 0);
    assert!(dropped > 0);
}

/// 8. **The inplace map**, and why one open row needs no prefix sum.
fn the_inplace_map() {
    let one = Inplace::one(3, 4);
    let many = Inplace::many((0..1_000).step_by(7).map(|r| (r, 3)));
    let round_trips = (0..1_000)
        .filter(|&d| one.data_row(one.content_row(d)) == Some(d))
        .count();
    let many_trips = (0..1_000)
        .filter(|&d| many.data_row(many.content_row(d)) == Some(d))
        .count();
    println!("report  the inplace map, both spellings:");
    println!(
        "  one open detail row:   {round_trips} of 1 000 round trip, {} open rows stored",
        one.len()
    );
    println!(
        "  many open rows:        {many_trips} of 1 000 round trip, {} open rows stored",
        many.len()
    );
    println!(
        "  the detail region maps to no data row: {:?} at content row 5\n",
        one.data_row(5)
    );
    assert_eq!(round_trips, 1_000);
    assert_eq!(many_trips, 1_000);
}

/// 1. The two scenes, and the ticket that inverts each.
fn scene_list() {
    println!("report  the two scenes, and what stands each up:");
    println!(
        "  {:>3}  {:<58}  {:<14}  inverted by",
        "#", "scene", "standing"
    );
    let mut red = 0usize;
    for scene in SCENES.iter().filter(|s| s.stands.contains(&"collapsible")) {
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
            "  {:>3}  {:<58}  {word:<14}  {ticket}",
            scene.number, scene.name
        );
    }
    assert_eq!(red, 0, "both scenes stand since components 22");
    println!(
        "\n  the verdict over the one subject: {:?}",
        accordion::standing()
    );
    // **The sentence is still live and still readable**, which is why `owed_message` takes a
    // declaration list rather than reading the crate: a message no test and no report can reach is a
    // message that rots.
    println!(
        "  what a reader would see if the subject went away:\n    {}\n",
        accordion::owed_message(&[], "the accordion of twelve sections")
            .expect("an empty declaration list is a scene that is not standing")
    );
}

/// 3. The frame at 0 / 6 / 12 open, measured, with §8's own column beside it.
fn the_frame() {
    println!("report  §8's frame table, measured here and remembered there:");
    println!(
        "  {:<40}  {:>9}  {:>7}  {:>6}  {:>7}  {:>6}",
        "the accordion, 300 columns", "us", "writes", "verbs", "entries", "stops"
    );
    let rows: [(&str, Screen, Option<SpecRow>); 4] = [
        (
            "twelve closed, 80 rows",
            Screen::correct(0),
            Some((
                accordion::SPEC_CLOSED_ENTRIES,
                accordion::SPEC_CLOSED_STOPS,
                accordion::SPEC_CLOSED_US,
            )),
        ),
        (
            "six open (content 72), 80 rows",
            Screen::correct(6),
            Some((
                accordion::SPEC_SIX_ENTRIES,
                accordion::SPEC_SIX_STOPS,
                accordion::SPEC_SIX_US,
            )),
        ),
        // §8's row is the one below; this one is here because the *cost* ratio §8 states —
        // 1.05x six open rather than 2x — is a statement about one viewport, and six open is
        // measured at eighty rows above.
        (
            "twelve open (content 132), 80 rows",
            Screen::correct(12),
            None,
        ),
        (
            "twelve open (content 132), 74 rows",
            Screen::correct(12).on_the_prototypes_viewport(),
            Some((
                accordion::SPEC_TWELVE_ENTRIES,
                accordion::SPEC_TWELVE_STOPS,
                accordion::SPEC_TWELVE_US,
            )),
        ),
    ];
    let mut costs = Vec::new();
    for (name, screen, spec) in rows {
        let shape = accordion::shape(screen);
        let us = micros(accordion::cost(screen, FRAMES));
        costs.push(us);
        let tail = match spec {
            Some((entries, stops, spec_us)) => {
                assert_eq!(shape.entries + CHROME_ENTRIES, entries);
                assert_eq!(shape.stops + CHROME_STOPS, stops);
                format!(
                    "(§8: {spec_us:>6.2} us, {entries} entries, {stops} stops; here + chrome: \
                     {} / {})",
                    shape.entries + CHROME_ENTRIES,
                    shape.stops + CHROME_STOPS
                )
            }
            None => String::from("(§8 states no row at this viewport)"),
        };
        println!(
            "  {name:<40}  {us:>9.2}  {:>7}  {:>6}  {:>7}  {:>6}   {tail}",
            shape.writes, shape.verbs, shape.entries, shape.stops,
        );
    }
    println!(
        "  the chrome §8's screen carried and this crate cannot build: {CHROME_ENTRIES} entries, \
         {CHROME_STOPS} stops"
    );
    println!(
        "  fitted from ONE row — twelve closed, {} entries against §8's {} — and it predicts the \
         other three to the unit, the mid-transition pair included",
        accordion::CLOSED_ENTRIES,
        accordion::SPEC_CLOSED_ENTRIES
    );
    println!(
        "  twelve open against six, both at eighty rows: {:.2}x   (§8: {:.2}x, and its point is \
         *not 2x*)\n",
        costs[2] / costs[1],
        accordion::SPEC_TWELVE_US / accordion::SPEC_SIX_US
    );
}

/// 3. The scene: closed against `h = 0`, on identical surfaces.
fn closed_against_zero_height() {
    let correct = accordion::shape(Screen::correct(0));
    let drawn = accordion::shape(Screen::zero_rect());
    let quiet = accordion::shape(Screen::zero_rect_declaring());

    println!("report  closed content is not drawn, and the surfaces are identical:");
    println!(
        "  {:<44}  {:>7}  {:>7}  {:>7}  {:>6}  {:>6}",
        "arm", "asked", "writes", "verbs", "entr.", "stops"
    );
    for (name, shape) in [
        ("body skipped (the rule)", correct),
        ("body drawn at h = 0, no cull", drawn),
        ("body declares every row, paints admitted", quiet),
    ] {
        println!(
            "  {name:<44}  {:>7}  {:>7}  {:>7}  {:>6}  {:>6}",
            shape.asked, shape.writes, shape.verbs, shape.entries, shape.stops
        );
    }
    println!(
        "  §8, on its own screen:                          {} / {} cells asked for, \
         {} / {} entries, {} / {} stops",
        accordion::SPEC_CLOSED_CELLS,
        accordion::SPEC_ZERO_CELLS,
        accordion::SPEC_CLOSED_ENTRIES,
        accordion::SPEC_ZERO_ENTRIES,
        accordion::SPEC_CLOSED_STOPS,
        accordion::SPEC_ZERO_STOPS
    );
    println!(
        "  the surfaces:  h = 0 → {}   ·   declares only → {}",
        accordion::surfaces_are_identical(Screen::zero_rect()),
        accordion::surfaces_are_identical(Screen::zero_rect_declaring())
    );
    println!(
        "  the excess:    {} hit entries and {} ring entries, and the two are one subtraction",
        drawn.entries - correct.entries,
        drawn.stops - correct.stops
    );

    // Which of §20's nine can see it, on each spelling. The allocation column is a **measured**
    // total on both arms rather than the same value handed in twice — a figure defaulted to zero is
    // a counter that prints `0` when it means nobody counted, and one measured on only one arm is a
    // separation the instrument invented.
    let correct_allocations = steady_allocations(Screen::correct(0));
    for (name, screen) in [
        ("drawn at h = 0", Screen::zero_rect()),
        ("declares only", Screen::zero_rect_declaring()),
    ] {
        let separating = accordion::counters_that_separate_them(
            screen,
            correct_allocations,
            steady_allocations(screen),
        );
        let words: Vec<&str> = separating.iter().map(|c| c.word()).collect();
        println!(
            "  counters that separate the arms ({name:<14}): {} of §20's nine — {}",
            separating.len(),
            words.join(", ")
        );
    }
    println!(
        "  the allocation column separates on a difference of ONE over {ALLOC_FRAMES} frames — \
         see the three totals below. That is amortised zero on every arm and not a detector; the \
         gate in `src/accordion.rs` holds both arms to the same total, so the list there is the \
         mechanism's and this one is this machine's."
    );

    // The standing budget, as a total over 200 frames rather than a mean.
    for (name, screen) in [
        ("twelve closed", Screen::correct(0)),
        ("six open", Screen::correct(6)),
        ("drawn at h = 0", Screen::zero_rect()),
    ] {
        println!(
            "  allocations over {ALLOC_FRAMES} steady frames ({name:<14}), as a TOTAL: {}",
            steady_allocations(screen).total()
        );
    }
    println!();
}

/// **What [`ALLOC_FRAMES`] steady frames of `s` allocate, as a total.**
///
/// The driver is attached and one frame drawn **before** the probe starts counting: attaching a
/// terminal and building a theme allocate, and a total that carried them would be a report about
/// `Driver::headless`. A total and not a mean, because a mean cannot see anything below `n`.
fn steady_allocations(s: Screen) -> Allocations {
    let mut driver = accordion::driver_for(s);
    driver.frame(|cx| accordion::draw_into(&mut Direct, cx, s));
    let (_, allocations) = count_allocations(|| {
        for _ in 0..ALLOC_FRAMES {
            driver.frame(|cx| accordion::draw_into(&mut Direct, cx, s));
        }
    });
    Allocations::over(ALLOC_FRAMES, allocations as u64)
}

/// 4. The transition, height by height.
fn the_transition() {
    println!("report  halfway down a collapse — a body that culls against one that does not:");
    println!(
        "  {:>4}  {:>10}  {:>10}  {:>8}",
        "h", "culls", "does not", "excess"
    );
    for h in 0..=accordion::BODY_ROWS {
        let mut screen = Screen::correct(1);
        screen.collapsing = Some(h);
        let obedient = accordion::shape(screen);
        screen.body = Body::Uncut;
        let rude = accordion::shape(screen);
        println!(
            "  {h:>4}  {:>10}  {:>10}  {:>8}   (a body {h} rows tall admits {} widgets)",
            obedient.stops,
            rude.stops,
            rude.stops - obedient.stops,
            admitted(h)
        );
    }
    let obedient = accordion::shape(Screen::collapsing(Body::Culls));
    let rude = accordion::shape(Screen::collapsing(Body::Uncut));
    println!(
        "  §8's frame — six open, the last of them {} rows tall: {} against {} ring entries here, \
         {} against {} with the chrome, §8's own {} against {}",
        accordion::MID_HEIGHT,
        obedient.ring,
        rude.ring,
        obedient.ring + CHROME_ENTRIES,
        rude.ring + CHROME_ENTRIES,
        accordion::SPEC_MID_CULLED,
        accordion::SPEC_MID_UNCULLED
    );
    println!(
        "  and the surfaces there: {}\n",
        accordion::transition_surfaces_are_identical()
    );
}

/// 5. The fold anchor.
fn the_fold_anchor() {
    println!(
        "report  a {}-line document, a block every {}, {} lines long, every {}th block closed:",
        accordion::DOC_LINES,
        accordion::BLOCK_EVERY,
        accordion::BLOCK_LEN,
        accordion::CLOSE_EVERY
    );
    for anchor in [Anchor::Raw, Anchor::Shifted] {
        let (wrong, folds) = accordion::misanchored_after_the_edit(anchor);
        println!(
            "  {:<12}  {wrong} of {folds} closed folds now sit on a line that opens no block",
            anchor.word()
        );
    }
    let (reanchor, edit) = accordion::reanchor_cost(REPEATS);
    println!(
        "  the reanchor {:.2} us, the document edit it sits beside {:.2} us   \
         (§8: {:.2} against {:.2})",
        micros(reanchor),
        micros(edit),
        accordion::SPEC_REANCHOR_US,
        accordion::SPEC_EDIT_US
    );
    println!("  nothing throws on either arm, and the screen is plausible on both\n");
}

/// 6. O5's `collapsible` column.
fn o5() {
    let component = INVENTORY
        .iter()
        .find(|c| c.id == "collapsible")
        .expect("the freeze homes `collapsible`");
    let axes: Vec<&str> = Axis::ALL
        .into_iter()
        .filter(|a| component.declares(*a))
        .map(Axis::name)
        .collect();
    println!(
        "report  O5 — `collapsible` declares {} hostile {}: {}",
        axes.len(),
        if axes.len() == 1 { "axis" } else { "axes" },
        axes.join(", ")
    );
    for scene in scenes_for("collapsible") {
        let covers: Vec<&str> = scene
            .covers
            .iter()
            .filter(|(id, _)| *id == "collapsible")
            .map(|(_, a)| a.name())
            .collect();
        println!(
            "  scene {:>2}  {:<58}  covers: {}",
            scene.number,
            scene.name,
            if covers.is_empty() {
                String::from("-")
            } else {
                covers.join(", ")
            }
        );
    }
    println!(
        "  O5 does not move: it asks about axes, not about components, and a red scene is not \
         evidence\n"
    );
}

/// 7. What does not reproduce, and why.
fn what_does_not_reproduce() {
    println!("report  what does not reproduce, said out loud rather than engineered away:");
    println!(
        "  · the cell columns. §8's {} against {} cells asked for is a screen with a menu bar, a \
         toolbar of sixteen chips and thirty-two status chips on it; this one has none of those, \
         and `menu_bar` is components 35's while `field` is 24's. What reproduces is the \
         direction and the equality under it — `asked` grows, `writes` does not move, and the \
         surfaces are identical.",
        accordion::SPEC_CLOSED_CELLS,
        accordion::SPEC_ZERO_CELLS
    );
    println!(
        "  · the µs, and they are out by an order of magnitude rather than a little. §8's 24.38 \
         for a screen that wrote 23 034 cells is about 1 ns a written cell; this engine is about \
         19, so a frame that writes a partition of a 300x80 terminal costs a few hundred. Damage \
         is marked at write time and never derived by diffing, so a steady frame that rewrites \
         its rectangle damages every cell of it — and §8's own table claims `marked 0` on the \
         same frames, which is the one column of §20 this crate cannot read at all. A timing is a \
         report and no gate here rests on one; what reproduces is the direction, and the *not 2x* \
         the twelve-open row is about."
    );
    println!(
        "  · the reanchor's own figure. §8 says {:.2} us and this machine is faster; what is \
         asserted is the count, 4 166 of 4 167 against 0 of 4 167, which is exact.",
        accordion::SPEC_REANCHOR_US
    );
    println!(
        "  · `verbs` separates the `h = 0` spelling and is blind to the one beside it, which §8 \
         does not record. A gate written on the cheapest counter that happened to work would be \
         green on a body that declares every row and paints the admitted ones."
    );
    println!(
        "  · `marked` prints `unreachable` and never `0`: `damage.rs` is `pub(crate)` and \
         `Presented` has no cell count."
    );
    println!(
        "  · §8's byte pair, {} live against {} with a tween slot. The live half is `open: bool` \
         and a `u16` — {} — and five is what a third `u16` would cost, of which there is none: \
         `Tween::to` is the target while a tween runs and the height is it afterwards. And the \
         pair cannot both be a size_of of one type, because an `Option<Tween<u16>>` field costs \
         its {} bytes empty — a record that is 5 B without a tween is a record whose tween lives \
         somewhere else, and nothing here has anywhere to put one.",
        disclose::SPEC_LIVE_BYTES,
        disclose::SPEC_TWEEN_BYTES,
        disclose::LIVE_BYTES,
        disclose::TWEEN_BYTES,
    );
    println!(
        "  · the watermark's {} cells / {} rows wrong / 3 frames. Those are a prototype's *body*, \
         and they are replaced rather than reproduced: §9's own precondition says a measured \
         extent is taken inside the rectangle the decision produced, so over a body that fills \
         what it is handed it latches at {} rows where {} are right, permanently — and over one \
         that draws only its content the two arms are indistinguishable, which is what makes the \
         rule unconditional rather than a preference.",
        disclose::SPEC_WATERMARK_CELLS,
        disclose::SPEC_WATERMARK_ROWS_WRONG,
        disclose::WATERMARK_LATCHED_ROWS,
        disclose::CONTENT_ROWS,
    );
    println!(
        "  · §8's {} ring probes. Not reachable from a header gesture at all — the three self-close \
         gestures each leave the focus off the body before the vanish rule looks, for three \
         different reasons — so the number belongs to a collapse nobody clicked for, and the figure \
         printed above is this screen's ring rather than the prototype's.",
        disclose::SPEC_VANISH_PROBES,
    );
    println!(
        "  · §8's {} frames to quiet. A cadence and not a count: sixty hertz over two hundred \
         milliseconds is twelve and a hundred and twenty is twenty-four, and the relation — the \
         tween is quiet exactly when the clock reaches `start + dur`, and the screen sleeps after — \
         is what is gated.\n",
        accordion::SPEC_TRANSITION_FRAMES,
    );
}

/// **What §8 remembers about one row of its frame table**: hit entries, tab stops, and the µs.
///
/// A named type rather than the tuple it is, because the tuple is three numbers of two kinds and a
/// reader of `the_frame` has no way to tell the first two apart at the call site.
type SpecRow = (usize, usize, f64);

/// A `Duration` in microseconds, which is the unit §8 states every one of its figures in.
fn micros(d: Duration) -> f64 {
    d.as_secs_f64() * 1e6
}
