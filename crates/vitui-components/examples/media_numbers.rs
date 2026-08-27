//! **The picture screen: the colour ladder, the custom census, the two traps and the distinctions
//! the colour axis takes away, as numbers.**
//!
//! Components ticket 29. The convention is the runtime's — a file in `examples/` named
//! `<subject>_numbers.rs` that prints the numbers a human reads — and so is the rule about what an
//! example may be: **`cargo test` does not run this file.** The rows of
//! [`vitui_components::gates::REGISTER`] this ticket adds *cite* it, and each names a `#[test]` in
//! `src/picture.rs` beside the citation.
//!
//! # What it prints
//!
//! 1. **The scene**, where it stands, and which ticket inverts it.
//! 2. **The frame** — 24 000 cells, 24 000 verbs, 24 000 customs, no region, no `fill`.
//! 3. **The ladder**, beside the plot's, which is where `Extended == Unicode` comes from.
//! 4. **The distinction census** at four tiers and two sources, which is the colour axis.
//! 5. **The two traps**, each with the counter that sees it and the counter that does not.
//! 6. **What this crate cannot ask**, with the item each answer needs.
//! 7. **What does not reproduce**, said out loud rather than engineered away.
//!
//! # It asserts the shape and not the timings
//!
//! Every `assert!` below is a count, a ratio or an equality. §14's own headline figure — *261–357 µs
//! of draw against a 100 µs budget* — is a timing and is printed beside what this screen measures,
//! and the finding underneath it is printed too: **the customs are not the cost.**

use std::time::{Duration, Instant};

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::chart::raster::RUNGS;
use vitui_components::gates::Standing;
use vitui_components::picture::{
    self, ADJACENT_EQUAL, ADJACENT_EQUAL_GRADIENT, BAR_LADDER, Build, CELL_ASPECT_MEASURED,
    CELL_ASPECT_NOMINAL, CELLS, CUSTOMS, DEPTHS, H, HORIZONTAL_PAIRS, LADDER, Ladder, MARK_BITS,
    Modules, Pairing, Palette, QR_CUSTOMS, QR_MODULE_COUNT, Source, W,
};
use vitui_components::scenes::{SCENES, scenes_for};
use vitui_runtime::ColorDepth;

// **The probe, because `allocations` is a total and a total needs something that counts.**
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// How many frames a per-frame figure is taken over. The minimum of eight, warmed.
const FRAMES: u32 = 8;

fn main() {
    println!("The picture screen — {W}x{H}, {CELLS} cells, every one of them outside the theme\n");
    scene_list();
    the_frame();
    the_ladder();
    the_colour_axis();
    the_two_traps();
    what_this_crate_cannot_ask();
    what_does_not_reproduce();
}

/// 1. The scene, and which ticket inverts it.
fn scene_list() {
    println!("report  the scene, and where it stands:");
    println!(
        "  {:>3}  {:<50}  {:<12}  inverted by",
        "#", "scene", "standing"
    );
    let mut red = 0usize;
    for scene in SCENES.iter().filter(|s| s.stands.contains(&"picture")) {
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
            "  {:>3}  {:<50}  {word:<12}  {ticket}",
            scene.number, scene.name
        );
    }
    assert_eq!(red, 1, "the picture scene is red until components 30");
    println!(
        "\n  scenes_for(\"picture\") answers {} and scenes_for(\"qr\") answers {}",
        scenes_for("picture").count(),
        scenes_for("qr").count()
    );
    println!("  standing: {:?}\n", picture::standing());
}

/// 2. The frame.
fn the_frame() {
    println!("report  a picture is one verb a cell, and there is nothing else available:");
    println!(
        "  {:<26}  {:>8}  {:>8}  {:>7}  {:>8}  {:>7}  {:>6}  {:>7}",
        "build", "writes", "distinct", "verbs", "customs", "regions", "allocs", "adj eq"
    );
    let arms = [
        ("photograph, Extended", Build::correct()),
        ("photograph, Ascii", Build::correct().at(RUNGS[0])),
        ("gradient, Extended", Build::correct().of(Source::Gradient)),
        (
            "the thirteen roles",
            Build {
                palette: Palette::Roles,
                ..Build::correct()
            },
        ),
    ];
    for (label, build) in arms {
        let shape = picture::shape(build);
        // **A total and never a mean**: a frame allocating on seven of eight reports 0 as a mean.
        // The session is opened *outside* the window, because `Driver::headless` attaches an engine
        // and allocates its surfaces — counted inside, that is 64 allocations a frame that belong
        // to the attach and not to the draw.
        let mut session = picture::Session::open(build);
        let (_, allocs) = count_allocations(|| {
            for _ in 0..FRAMES {
                session.frame();
            }
        });
        let adjacent = picture::adjacent_equal(&picture::render(build));
        println!(
            "  {label:<26}  {:>8}  {:>8}  {:>7}  {:>8}  {:>7}  {:>6}  {:>7}",
            shape.writes,
            shape.distinct,
            shape.verbs,
            shape.census.customs,
            shape.regions,
            allocs,
            adjacent
        );
        assert_eq!(shape.writes, shape.distinct, "a cell written twice");
    }
    assert_eq!(
        picture::adjacent_equal(&picture::render(Build::correct())),
        ADJACENT_EQUAL
    );

    // **The finding under §14's headline.** The customs are 24 000 against 0 and the two frames are
    // inside each other's noise, taken round-robin so that a machine warming up cannot be read as a
    // difference between the arms.
    let custom = Build::correct();
    let roles = Build {
        palette: Palette::Roles,
        ..custom
    };
    let (a, b) = round_robin(custom, roles, 200);
    println!(
        "\n  24 000 customs {:?} against 0 customs {:?} — {:.1}%. **The customs are not the cost**; \
         the 24 000 verbs are. What the census is evidence for is structural: a cell outside the \
         theme is a cell no `fill` and no run can reach",
        a,
        b,
        (a.as_secs_f64() / b.as_secs_f64() - 1.0) * 100.0
    );
    println!(
        "  §14: 261–357 µs of draw against a 100 µs budget, zero allocations. Here: {a:?}, and the \
         allocation total is the gate\n"
    );
}

/// 3. The ladder.
fn the_ladder() {
    println!("report  a picture's ladder is a colour ladder, so it stops at two:");
    println!(
        "  {:<12}  {:>10}  {:>10}  {:>10}",
        "rung", "picture sy", "bar sy", "mark bits"
    );
    for (i, set) in RUNGS.into_iter().enumerate() {
        println!(
            "  {:<12}  {:>10}  {:>10}  {:>10}",
            format!("{set:?}"),
            LADDER[i],
            BAR_LADDER[i],
            MARK_BITS[i]
        );
    }
    let correct = Build::correct();
    let same = picture::render(correct.at(RUNGS[1]))
        .diff(&picture::render(correct.at(RUNGS[2])))
        .clean();
    let braille = Build {
        ladder: Ladder::Braille,
        ..correct
    };
    let braille_same = picture::render(braille.at(RUNGS[1]))
        .diff(&picture::render(braille.at(RUNGS[2])))
        .clean();
    assert!(same && !braille_same);
    println!(
        "\n  Extended == Unicode for a picture: {same}. For the ladder it is not drawn on: \
         {braille_same}. Braille's 2x4 buys a picture nothing, because a cell carries exactly two \
         colours whatever glyph it holds\n"
    );
}

/// 4. The colour axis.
fn the_colour_axis() {
    println!(
        "report  the axis with a floor of nothing — horizontal distinctions of {HORIZONTAL_PAIRS}:"
    );
    println!(
        "  {:<12}  {:>10}  {:>8}  |  {:>10}  {:>8}",
        "tier", "photo kept", "lost", "gradient", "lost"
    );
    for tier in DEPTHS.into_iter().chain(std::iter::once(ColorDepth::None)) {
        let photo = picture::distinctions(Build::correct().tier(tier));
        let gradient = picture::distinctions(Build::correct().of(Source::Gradient).tier(tier));
        println!(
            "  {:<12}  {:>10}  {:>7.2}%  |  {:>10}  {:>7.2}%",
            format!("{tier:?}"),
            photo.kept,
            photo.lost_percent(),
            gradient.kept,
            gradient.lost_percent()
        );
    }
    assert_eq!(
        picture::distinctions(Build::correct().tier(ColorDepth::None)).kept,
        0,
        "the floor is not a floor"
    );
    println!(
        "\n  Every other component degrades to a worse drawing of itself. A picture at \
         `ColorDepth::None` has 0 of {HORIZONTAL_PAIRS} — a description of itself, with nothing to \
         re-pair the lost distinctions with\n"
    );
}

/// 5. The two traps.
fn the_two_traps() {
    println!("report  the two traps, and the counter each one is invisible to:");
    let shifted = picture::cells_changed_by_shift(Build::correct(), 1);
    let still = picture::cells_changed_by_shift(Build::correct(), 0);
    let frames = picture::repaints_over(Build::correct(), 4);
    println!(
        "  a translation by one whole cell row changes {shifted} of {CELLS} cells; a still picture \
         changes {still}"
    );
    println!("  a still picture's first four frames repaint {frames:?}");
    assert_eq!(shifted, u64::from(CELLS));
    assert_eq!(frames, vec![u64::from(CELLS), 0, 0, 0]);

    let modules = Modules::v1();
    let area = picture::qr_area();
    let correct = Build::correct();
    let inverted = Build {
        pairing: Pairing::Inverted,
        ..correct
    };
    let flat = correct.at(RUNGS[0]);
    let good = picture::readback(
        &picture::render_qr(correct, area, &modules),
        area,
        correct,
        &modules,
    );
    let bad = picture::readback(
        &picture::render_qr(inverted, area, &modules),
        area,
        inverted,
        &modules,
    );
    let one_per_cell = picture::readback(
        &picture::render_qr(flat, area, &modules),
        area,
        flat,
        &modules,
    );
    println!(
        "\n  {:<28}  {:>10}  {:>14}  {:>14}",
        "QR build", "modules", "read back wrong", "module aspect"
    );
    for (label, wrong, per_cell) in [
        ("two modules a cell", good, 2u16),
        ("the pairing inverted", bad, 2),
        ("one module a cell", one_per_cell, 1),
    ] {
        println!(
            "  {label:<28}  {QR_MODULE_COUNT:>10}  {wrong:>14}  {:>7.2} / {:.2}",
            picture::module_aspect(per_cell, CELL_ASPECT_NOMINAL),
            picture::module_aspect(per_cell, CELL_ASPECT_MEASURED)
        );
    }
    assert_eq!(good, 0);
    assert!(bad > 0);
    assert_eq!(
        one_per_cell, 0,
        "the readback is the wrong instrument for the aspect, and that is why they are two gates"
    );
    println!(
        "\n  A QR spends {QR_CUSTOMS} customs against a picture's {CUSTOMS} — not because it has too \
         many distinctions but because it has exactly two and they must be *those* two. The \
         readback catches the pairing and is blind to the aspect; the aspect is arithmetic and is \
         blind to the pairing\n"
    );
}

/// 6. What this crate cannot ask.
fn what_this_crate_cannot_ask() {
    println!("report  what this crate cannot ask, and the item each answer needs:");
    println!(
        "  the wire.  §14 prices a picture at 37.5 B/cell, 900 KB a frame, 27.0 MB/s at 30 fps, \
         and a whole-row translation at 5 885 bytes. **No crate above the engine can read a byte \
         the engine wrote.** `vitui_runtime::Config` is reachable; `Clock`, `Output` and \
         `Overrides` are not in `vitui_runtime::line::ENGINE_NAMES` at all, so the only headless \
         door is `Driver::headless`, whose sink is a `Vec` nobody can reach. Needs a driver that \
         hands its sink back, or those three names re-exported"
    );
    println!(
        "  the quantiser.  `Theme::custom`'s own documentation says the caller *owes a branch on \
         the terminal's own capabilities* and offers no verb to ask with: `roles_differ_on_wire` \
         compares two of the thirteen roles, and a picture's cells are outside the theme by \
         construction. `picture::wire_differ` is the way through — `Roles::from_palette` maps \
         base08 and base0A verbatim onto Danger and Warn — and it is one theme construction a \
         colour pair. Needs `Theme::colours_differ_on_wire(Rgb, Rgb)`"
    );
    println!("  Both are filed as runtime architecture issue 34.\n");
}

/// 7. What does not reproduce.
fn what_does_not_reproduce() {
    println!("report  what does not reproduce, said out loud:");
    println!(
        "  §14's 50.7% of horizontal distinctions gone at sixteen colours is a prototype's picture. \
         This screen's photograph loses 35.84% and its gradient loses 99.27% of the same {HORIZONTAL_PAIRS} \
         adjacencies at the same rung — 2.8x apart. **On the distinction axis the source decides a \
         great deal**, which is the opposite direction from §14's B/cell claim that it moves the \
         wire by 9%: adjacent samples of noise are far apart in colour space and survive a \
         quantiser, adjacent samples of a ramp are one step apart and do not"
    );
    println!(
        "  §14's QR aspect pair, 0.97 against 0.50, cannot both come from one cell. Two modules a \
         cell is exactly twice one at every cell aspect, so 0.97 needs a cell of 0.485 and 0.50 \
         needs one of 0.500. The pair is a nominal cell's floor beside a measured cell's ceiling; \
         both columns are printed above and the gate is the relation"
    );
    // **The overrun is written next to the number**, which is this crate's own precedent — the
    // `table` ticket recorded *166 µs over the budget while damaging nothing* rather than saying
    // which class it was near. A picture is over the budget, and by how much is the report.
    let frame = picture::cost(Build::correct(), FRAMES);
    let over = frame.as_secs_f64() / 0.001;
    println!(
        "  §14's 261–357 µs is a prototype's screen, and its *against a 100 µs budget* is the wrong \
         class besides. This one writes every cell of 300x80 in one verb each, which is §20's \
         full-screen class — and it is **over that budget too**: {frame:?} against 1 ms, {over:.2}x. \
         Recorded rather than optimised against, because what a picture costs is one verb a cell \
         and the only thing that would move it is a verb that can carry two paints. The dense \
         screen writes the same 24 000 cells in 171 verbs and a twelve-column table in 160"
    );
    println!(
        "  the gradient's {ADJACENT_EQUAL_GRADIENT} equal adjacencies of {} are 7.4%. §14 says no \
         `fill` is available at any size and the photograph's {ADJACENT_EQUAL} says so exactly; the \
         smoothest frame anything could produce still needs its own paint for 92.6% of them",
        picture::ADJACENCIES
    );
}

/// The two arms interleaved, minimum of each. A machine warming up is not a difference between
/// arms — `crate::series` learned that with a 4% that turned out to be noise with a sign.
fn round_robin(a: Build, b: Build, rounds: u32) -> (Duration, Duration) {
    let (mut one, mut two) = (picture::Session::open(a), picture::Session::open(b));
    let (mut best_a, mut best_b) = (Duration::MAX, Duration::MAX);
    for _ in 0..rounds {
        for (session, best) in [(&mut one, &mut best_a), (&mut two, &mut best_b)] {
            let started = Instant::now();
            session.frame();
            *best = (*best).min(started.elapsed());
        }
    }
    (best_a, best_b)
}
