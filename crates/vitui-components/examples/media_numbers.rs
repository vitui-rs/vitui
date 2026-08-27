//! **The media family: the colour ladder, the custom census across every construction, the two
//! traps, the distinctions the colour axis takes away, and the chrome's six parts of ten.**
//!
//! Components tickets 29 and 30. The convention is the runtime's — a file in `examples/` named
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
//! 6. **The family's census**, one row a construction — 24 000, 4, 2, 0, 0, 0 — which is what §14
//!    states as a *contrast* and what makes F11 legible as a count.
//! 7. **The chrome**, ten parts and the mechanism each of the four that do not ship is waiting for,
//!    with the grab's three phases beside it.
//! 8. **What this crate cannot ask**, with the item each answer needs.
//! 9. **What does not reproduce**, said out loud rather than engineered away.
//!
//! # It asserts the shape and not the timings
//!
//! Every `assert!` below is a count, a ratio or an equality. §14's own headline figure — *261–357 µs
//! of draw against a 100 µs budget* — is a timing and is printed beside what this screen measures,
//! and the finding underneath it is printed too: **the customs are not the cost.**

use std::time::{Duration, Instant};

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::chart::raster::RUNGS;
use vitui_components::counters::Tally;
use vitui_components::gates::Standing;
use vitui_components::ink::Direct;
use vitui_components::media::player::{Chapter, Needs, PARTS, Player, SHIPPED, chrome_into, scrub};
use vitui_components::media::{
    BARCODE_CUSTOMS, Census, Palette, QR_CUSTOMS, barcode_into, picture_into, spectrum_into,
    vu_meter_into, waveform_into,
};
use vitui_components::picture::{
    self, ADJACENT_EQUAL, ADJACENT_EQUAL_GRADIENT, BAR_LADDER, Build, CELL_ASPECT_MEASURED,
    CELL_ASPECT_NOMINAL, CELLS, CUSTOMS, DEPTHS, H, HORIZONTAL_PAIRS, LADDER, Ladder, MARK_BITS,
    Pairing, QR_MODULE_COUNT, Source, W, v1_symbol,
};
use vitui_components::scenes::{SCENES, scenes_for};
use vitui_runtime::ctx::Driver;
use vitui_runtime::{Button, Buttons, ColorDepth, Mods, Mouse, MouseKind, Rect};

// **The probe, because `allocations` is a total and a total needs something that counts.**
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// How many frames a per-frame figure is taken over. The minimum of eight, warmed.
const FRAMES: u32 = 8;

/// One construction's draw, in the shape a census row needs it: an ink, a context and a census.
///
/// A `type` rather than the tuple written out, which is clippy's own request and is also what makes
/// the six rows of the census table read as six rows of one thing.
type Draw<'a> = dyn Fn(&mut Tally, &mut vitui_runtime::Ctx<'_, '_>, &mut Census) + 'a;

fn main() {
    println!("The picture screen — {W}x{H}, {CELLS} cells, every one of them outside the theme\n");
    scene_list();
    the_frame();
    the_ladder();
    the_colour_axis();
    the_two_traps();
    the_family_census();
    the_chrome();
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
    assert_eq!(
        red, 0,
        "the picture scene has stood on `picture` and `qr` since components 30"
    );
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

    let modules = v1_symbol();
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

/// 6. The family's census, one row a construction — the contrast §14 states.
fn the_family_census() {
    println!("report  what each construction spends on `Theme::custom`, and it is the contrast:");
    println!(
        "  {:<24}  {:>8}  {:>7}  {:>8}  {:>7}  why",
        "construction", "customs", "roles", "writes", "verbs"
    );

    // One rectangle, so the numbers are comparable. Small, because what is being compared is the
    // census and not the screen.
    const CW: u16 = 40;
    const CH: u16 = 10;
    let cells = u64::from(CW) * u64::from(CH);

    let bars: Vec<bool> = (0..CW).map(|i| i % 3 != 0).collect();
    let samples: Vec<f32> = (0..4_000)
        .map(|i| {
            let t = i as f32 / 4_000.0;
            (1.0 - t).powf(0.4) * (t * 240.0).sin()
        })
        .collect();
    let bins: Vec<f32> = (0..CW)
        .map(|i| 1.0 - f32::from(i) / f32::from(CW))
        .collect();
    let modules = v1_symbol();

    let rows: [(&str, &str, &Draw<'_>); 6] = [
        (
            "picture",
            "every cell is a pixel, and a pixel is outside the theme",
            &|ink, cx, census| {
                let src = picture::Sampled::of(Build::correct());
                picture_into(ink, cx, cx.area(), &src, &Default::default(), census);
            },
        ),
        (
            "qr",
            "two colours that must be *those* two, and a cell carries a pair",
            &|ink, cx, census| {
                let _ = vitui_components::media::qr_into(ink, cx, cx.area(), &modules, census);
            },
        ),
        (
            "barcode",
            "the same specification with no vertical structure inside a cell",
            &|ink, cx, census| {
                barcode_into(ink, cx, cx.area(), &bars, census);
            },
        ),
        ("waveform", "its colours are roles", &|ink, cx, census| {
            waveform_into(ink, cx, cx.area(), &samples, census);
        }),
        ("spectrum", "its colours are roles", &|ink, cx, census| {
            spectrum_into(ink, cx, cx.area(), &bins, census);
        }),
        (
            "vu meter",
            "three roles and a threshold",
            &|ink, cx, census| {
                vu_meter_into(ink, cx, cx.area(), &bins, census);
            },
        ),
    ];

    let mut spent = Vec::new();
    for (label, why, draw) in rows {
        let mut driver = Driver::headless(CW, CH).expect("a sink attaches");
        let mut census = Census::default();
        let mut tally = Tally::new();
        driver.frame(|cx| draw(&mut Tally::new(), cx, &mut Census::default()));
        driver.frame(|cx| draw(&mut tally, cx, &mut census));
        println!(
            "  {label:<24}  {:>8}  {:>7}  {:>8}  {:>7}  {why}",
            census.customs,
            census.roles,
            tally.writes(),
            tally.verbs()
        );
        spent.push((label, census.customs));
    }

    assert_eq!(spent[0].1, cells, "a picture spends one custom a cell");
    assert_eq!(spent[1].1, QR_CUSTOMS);
    assert_eq!(spent[2].1, BARCODE_CUSTOMS);
    for (label, customs) in &spent[3..] {
        assert_eq!(*customs, 0, "{label} spent a custom");
    }
    println!(
        "\n  §14 states the split rather than the figures: a picture spends one a cell, a waveform \
         **zero** because its colours are roles, and a QR **four** — not because it has too many \
         distinctions but because it has exactly two and they must be *those* two, which no theme \
         can promise. A barcode's **two** is this ticket's own subtraction: it carries no \
         information across a cell's own height, so two of a QR's four cell states are unreachable — \
         which is the same fact that gives it runs where a picture has none\n"
    );
}

/// 7. The chrome: ten parts, six shipped, and the grab's three phases.
fn the_chrome() {
    println!("report  the video player's chrome, and what each part is waiting for:");
    println!("  {:<28}  needs", "part");
    for part in PARTS {
        let needs = match part.needs {
            Needs::Nothing => "nothing — it ships",
            Needs::DragCapture => "the grab — measured below; the component is `slider` (30 -> 33)",
            Needs::Clock => "a component that owns a clock (components 42)",
            Needs::Passthrough => "the engine's out-of-band graphics (survey §6.1)",
        };
        println!("  {:<28}  {needs}", part.name);
    }
    assert_eq!(
        PARTS.iter().filter(|p| p.needs == Needs::Nothing).count(),
        SHIPPED
    );
    println!(
        "\n  {SHIPPED} of {} ship. The survey's ✅ is a claim about the engine, and the engine is \
         not what the chrome was waiting for",
        PARTS.len()
    );

    // The grab, over the shipped chrome with a posted pointer. §14's own fractions.
    const TW: u16 = 300;
    const TH: u16 = 4;
    const TRACK_ROW: u16 = 1;
    let at = |x: u16, kind: MouseKind| Mouse {
        x,
        y: TRACK_ROW,
        kind,
        buttons: Buttons::NONE,
        mods: Mods::NONE,
        at: Instant::now(),
    };
    let mut driver = Driver::headless(TW, TH).expect("a sink attaches");
    let mut player = Player::new(
        3_672.0,
        vec!["01 - engine, cells and layers".to_owned()],
        vec![Chapter {
            at: 0.42,
            name: "the seam".to_owned(),
        }],
    );
    let step = |driver: &mut Driver, player: &mut Player| {
        let mut answer = None;
        driver.frame(|cx| {
            let track = chrome_into(
                &mut Direct,
                cx,
                Rect::new(0, 0, TW, TH),
                player,
                &mut Census::default(),
            );
            answer = scrub(&track);
        });
        if let Some(v) = answer {
            player.position = v;
        }
        answer
    };

    println!("\n  {:<34}  {:>10}  {:>10}", "phase", "value", "§14");
    driver.post_mouse(at(20, MouseKind::Move));
    let hover = step(&mut driver, &mut player);
    println!(
        "  {:<34}  {:>10}  {:>10}",
        "the pointer arrives", "none", "-"
    );
    assert!(hover.is_none());

    driver.post_mouse(at(20, MouseKind::Down(Button::Left)));
    // Two frames: the grab is awarded at `end` from the index that has just drawn.
    step(&mut driver, &mut player);
    let pressed = step(&mut driver, &mut player).expect("the grab is held");
    println!(
        "  {:<34}  {pressed:>10.4}  {:>10.4}",
        "the press jumps (20/299)",
        20.0 / 299.0
    );

    driver.post_mouse(at(60, MouseKind::Move));
    let moved = step(&mut driver, &mut player).expect("still held");
    println!(
        "  {:<34}  {moved:>10.4}  {:>10.4}",
        "the move carries (60/299)",
        60.0 / 299.0
    );

    driver.post_mouse(at(60, MouseKind::Up(Button::Left)));
    step(&mut driver, &mut player);
    let released = step(&mut driver, &mut player);
    println!(
        "  {:<34}  {:>10}  {:>10}",
        "the release moves nothing", "none", "-"
    );
    assert!(released.is_none());
    assert!((pressed - 20.0 / 299.0).abs() < 1e-6);
    assert!((moved - 60.0 / 299.0).abs() < 1e-6);
    assert!((player.position - moved).abs() < f32::EPSILON);
    println!(
        "\n  Every one of those three is `Response::local` over `Response::rect`: no press origin, \
         no stored anchor, no fifth cross-frame fact. **`slider` leaves Tier 3 on this**, and what \
         is left for components 33 is the thumb, the keyboard, the step and the orientation\n"
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
