//! **The two standing budget counts, run for the first time by a crate that cannot name the engine.**
//!
//! Runtime ticket 20. Spec §20's budget carries two figures that are counts rather than timings —
//! *zero allocations during frame composition* and *a screen with no animation produces zero
//! wakeups* — and the ticket asks for both **on the real crate line**. Neither had ever been there:
//!
//! | count | where it lived | which line that is |
//! |---|---|---|
//! | zero allocations | `crates/vitui-runtime/tests/alloc.rs` | across the *runtime's* line, and `vitui-engine` is a dependency of that package — eleven of its gates open with `use vitui_engine::Rect` |
//! | zero wakeups | `anim.rs`'s `#[cfg(test)] mod tests` | **inside the library**, where a gate reaches `pub(crate)` and there is no line at all |
//!
//! So the two properties every component inherits were asserted by callers no component can be. This
//! file is the third place, and the only one where the compiler is holding the line: the
//! `vitui-components` package depends on `vitui-runtime` **and nothing else** — components spec §0's
//! constraint C6, gated by `line::tests::the_components_manifest_names_only_the_runtime` — so a
//! `use vitui_engine::…` here is `error[E0432]: unresolved import` before any count is taken.
//!
//! # What that costs the fixtures, and it is the point rather than a tax
//!
//! Every rectangle below comes out of [`Ctx::area`] and the layout solver, because a component-facing
//! crate has no other way to obtain one — `Rect` is named by twenty-seven of the runtime's public
//! declarations and reachable through none of them (`crate::line::ENGINE_NAMES`, architecture issue
//! 22). The runtime's own steady-frame gate writes `Rect::new(0, row, 80, 1)` twenty-four times, and
//! that is the shape this file cannot use — which means the frame being priced here is the frame a
//! component would actually draw, band-derived and solver-fed, rather than one with the rectangles
//! written in by hand.
//!
//! # Why one binary
//!
//! [`vitui_alloc_probe::CountingAllocator`] is a **process-global** allocator over a process-global
//! counter, so a sibling test that allocates lands in the number. That is what
//! `cargo test --workspace -- --test-threads=1` is for, and the `test` job in `.gitlab-ci.yml`
//! already runs exactly that line — these gates need no new invocation and no new job.
//!
//! Nothing here starts a thread, so spec §20's attribution-window rule (*an allocation gate with a
//! background job in it runs on a deterministic spawner, or joins before it measures*) costs this
//! file nothing.

use std::time::{Duration, Instant};

use vitui_alloc_probe::{CountingAllocator, steady};
use vitui_runtime::anim::{Easing, Tween};
use vitui_runtime::ctx::{Ctx, Driver};
use vitui_runtime::layout::{
    Col,
    Constraint::{Fixed, Weight},
};
use vitui_runtime::{Id, Interest, Role};

#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// The screen both gates draw on, which is `anim.rs`'s and `alloc.rs`'s so that a number that moves
/// is comparable with the one it moved from.
const W: u16 = 80;
/// See [`W`].
const H: u16 = 24;

/// Twenty-two lanes, which is what [`H`] leaves once a header row and a footer row are taken.
const LANES: usize = 22;

/// A list frame: a header, twenty-two rows each with a formatted cell and an interactive region, and
/// a footer.
///
/// **Written the way a component has to write it.** The band is [`Ctx::area`], the lanes are the
/// solver's, and the row rectangle handed to `interact` is one of them — no rectangle in this
/// function is constructed, because none can be.
fn list_frame(cx: &mut Ctx<'_, '_>) {
    let band = cx.area();
    let body = cx.theme().paint(Role::Body);
    let [header, rows, footer] = Col::new().split(band, [Fixed(1), Weight(1), Fixed(1)]);

    cx.text(header.x, header.y, "a header that does not move", body);

    let mut lanes = [rows; LANES];
    let n = Col::new().split_into(rows, &[Weight(1); LANES], &mut lanes);
    assert_eq!(n, LANES);
    for (i, lane) in lanes[..n].iter().enumerate() {
        let row = i as u64;
        cx.label(
            lane.x,
            lane.y,
            format_args!("row {row:>3}  {:>8.2}", row as f64 * 1.5),
            body,
        );
        cx.interact(
            Id::keyed(Id::named("row"), row),
            *lane,
            Interest::CLICK.with(Interest::FOCUS),
        );
    }

    cx.text(footer.x, footer.y, "a footer that does not move", body);
}

/// **A steady frame allocates nothing, from a crate that cannot name the engine.**
///
/// # Why it is here and not one crate down
///
/// `crates/vitui-runtime/tests/alloc.rs` asserts the same zero and cannot assert it about a
/// *component's* frame: its fixture hands `interact` a `Rect::new(0, row, 80, 1)`, and a component
/// has no `Rect::new`. What a component has instead is a band from [`Ctx::area`] and the layout
/// solver, and that path was outside every allocation window in the workspace — a solver call per
/// frame, a `split_into` over a caller-owned buffer, and twenty-two rectangles carried by inference
/// rather than by a name. Zero here is a claim about the frame a component can write; zero one crate
/// down is a claim about a frame it cannot.
///
/// Warmed with the identical workload before the window opens, for the reason [`steady`] documents:
/// the five frame structures take their allocation on the first frame that needs one and keep it, so
/// a window containing first touch is measuring the loader rather than the steady state.
#[test]
fn a_steady_frame_allocates_nothing_across_the_component_line() {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");

    let one_frame = |driver: &mut Driver| driver.frame(list_frame);
    one_frame(&mut driver);
    one_frame(&mut driver);

    steady(|| {
        for _ in 0..50 {
            one_frame(&mut driver);
        }
    });

    assert_eq!(
        driver.inspect().hits().len(),
        LANES,
        "and it drew what it claimed, on the last of those fifty frames"
    );
    assert_eq!(driver.inspect().ring().len(), LANES);
}

/// **A screen with no animation asks for no wakeups**, driven the way spec §1's loop drives it.
///
/// # Why it is here and not inside `anim.rs`
///
/// `anim::tests::a_quiet_screen_runs_one_frame_and_blocks` is compiled **inside the library**, which
/// is the one place in this workspace where a gate can reach a `pub(crate)` item — seventy-eight of
/// them exist. So it proved the ledger's arithmetic and not the consumer's access: until this file,
/// nothing had established that a component can *see* the zero it is required to produce. It can —
/// [`Driver::inspect`] hands back a `Frame`, `Frame::wakes` a `&WakeLedger`, and `frames`, `wakes`,
/// `worst`, `pending` and `line_count` are all `pub` on it. That is the finding, and it is a positive
/// one: the wake ledger is the *only* one of the runtime's five frame structures whose detector is
/// fully readable from across the component line.
///
/// # The loop, and why the assertion is `frames == 1`
///
/// The pacing gate is `max(deadline, previous + TICK)` — a minimum gap and not a tick — so a frame
/// that asks for nothing ends the loop outright. **The horizon is thirty seconds and the loop leaves
/// after one frame**, which is a stronger statement than a count of zero over a fixed number of
/// turns: there is no timeout in [`drive`] because there is none in the real loop.
///
/// # The zero is a measurement and not a vacuum
///
/// A quiet frame that asked for nothing because the instrument was not wired would pass the first
/// half of this test and every assertion in it. So the same driver, the same fixture and the same
/// loop run a second time with one 300 ms fade added, and the count has to move: nineteen frames and
/// eighteen wakes from one line, which is `anim.rs`'s scene table read from outside the crate.
#[test]
fn a_screen_with_no_animation_asks_for_no_wakeups_across_the_component_line() {
    let mut quiet = Driver::headless(W, H).expect("a sink cannot fail to attach");
    let frames = drive(&mut quiet, Instant::now(), Duration::from_secs(30), |cx| {
        list_frame(cx);
    });
    let ledger = quiet.inspect().wakes();
    assert_eq!(frames, 1, "one frame, and then the loop blocks for ever");
    assert_eq!(ledger.frames(), 1);
    assert_eq!(ledger.wakes(), 0, "nothing asked, so nothing is owed");
    assert_eq!(ledger.worst(), 0);
    assert_eq!(
        ledger.pending(),
        None,
        "and the loop has nothing to wait for"
    );
    assert_eq!(ledger.line_count(), 0, "no line asked");

    // The same everything, plus one fade. A gate that cannot tell a quiet screen from a broken
    // ledger is not a gate.
    let mut fading = Driver::headless(W, H).expect("a sink cannot fail to attach");
    let start = Instant::now();
    let fade: Tween<f32> =
        Tween::new(start, Duration::from_millis(300), 0.0, 1.0).eased(Easing::Out);
    let animated = drive(&mut fading, start, Duration::from_secs(30), |cx| {
        list_frame(cx);
        let now = cx.now();
        if let Some(at) = fade.wake(now) {
            cx.deadline(at);
        }
    });
    let ledger = fading.inspect().wakes();
    assert_eq!(animated, 19, "a 300 ms fade at 60 Hz is nineteen frames");
    assert_eq!(
        ledger.wakes(),
        18,
        "the last frame paints and asks for nothing"
    );
    assert_eq!(
        ledger.line_count(),
        1,
        "one call site asked, eighteen times"
    );
    assert!(ledger.runaway(60).is_none(), "a fade is not a spin");
}

/// 60 Hz, rounded to the nearest nanosecond — `anim.rs`'s constant, and the rounding is load-bearing:
/// eighteen ticks of 16 666 667 ns land 6 ns *after* a 300 ms fade ends, so the fade costs nineteen
/// frames rather than twenty. That is why the number above is 19.
const TICK: Duration = Duration::from_nanos(16_666_667);

/// Drive the loop the way spec §1 drives it — a wake, a frame, the fold — and return how many frames
/// ran.
///
/// A copy of `anim.rs`'s harness, and deliberately a copy: it is a `#[cfg(test)]` helper inside the
/// library, so a consumer cannot borrow it, and **the loop being writable from out here is part of
/// what this file checks**. Everything it touches — [`Driver::pin_clock`], [`Driver::frame`],
/// `inspect().wakes().pending()` — is on the component-facing surface.
fn drive(
    driver: &mut Driver,
    start: Instant,
    horizon: Duration,
    mut view: impl FnMut(&mut Ctx<'_, '_>),
) -> u32 {
    let mut now = start;
    let mut frames = 0;
    let end = start + horizon;
    loop {
        driver.pin_clock(now);
        driver.frame(&mut view);
        frames += 1;
        let Some(at) = driver.inspect().wakes().pending() else {
            // Nothing asked. **The loop blocks for ever**, which is the property rather than a
            // shortcut: there is no timeout here because there is none in the real one.
            break;
        };
        let next = at.max(now + TICK);
        if next > end {
            break;
        }
        now = next;
    }
    frames
}

/// **Components ticket 10, criterion 9: a steady frame of the dense screen allocates nothing, and
/// the figure is a total.**
///
/// # It is a total and this type has no `mean`
///
/// > A mean cannot see anything below `n`; a total can see one. (§21, refinement 2)
///
/// Every component prototype reported `allocs / n` with `n` between 40 and 200, so a frame
/// allocating on `n − 1` of `n` frames reported **0**. [`count_allocations`] hands back a total and
/// the assertion below is on the total, over **fifty** frames of a screen with 338 interactive
/// regions on it — so one allocation on one of those fifty fails it.
///
/// # Why this frame and not `list_frame`
///
/// The frame above is twenty-two lanes and one formatted cell each. This one is spec §2's own
/// screen: three panels, 222 chips, 111 buttons, 338 regions, 333 tab stops and 24 000 cells written
/// exactly once — and it is drawn through `text`, `chip`, `button` and `panel` rather than through
/// their construction. *Zero allocations during frame composition* is a budget every component
/// inherits, and until components ticket 10 there was no frame of components to inherit it on.
///
/// The path it puts under the window is the one that matters: `crate::ink::Direct::run` builds every
/// padding band, border run and label through `Ctx::stage` and `Ctx::blit`, which format into the
/// frame's own reusable buffer. The `Tally` and `Pen` implementations beside it materialise the same
/// string with `str::repeat`, and **an instrument allocating is not a defect** — which is exactly
/// why the gate has to run with `Direct` and cannot be a counter's report.
#[test]
fn a_steady_frame_of_the_dense_screen_allocates_nothing_as_a_total() {
    use vitui_alloc_probe::count_allocations;
    use vitui_components::counters::Allocations;
    use vitui_components::dense::{self, Arm};
    use vitui_components::ink::Direct;
    use vitui_components::runner::driver_at;
    use vitui_runtime::Density;

    let mut driver = driver_at(dense::W, dense::H, Density::Compact);
    let mut shape = None;
    let mut one_frame = |driver: &mut Driver| {
        driver.frame(|cx| {
            shape = Some(dense::draw_into(
                &mut Direct,
                cx,
                Arm::Correct,
                dense::REQUESTED,
            ));
        });
    };
    // Warmed with the identical workload before the window opens: the five frame structures take
    // their allocation on the first frame that needs one and keep it, so a window containing first
    // touch measures the loader rather than the steady state.
    one_frame(&mut driver);
    one_frame(&mut driver);

    const FRAMES: u32 = 50;
    let (_, total) = count_allocations(|| {
        for _ in 0..FRAMES {
            one_frame(&mut driver);
        }
    });
    let measured = Allocations::over(FRAMES, total as u64);
    assert_eq!(
        measured.total(),
        0,
        "{} allocations over {} frames of the dense screen. A mean would have reported 0 for any \
         total below {}",
        measured.total(),
        measured.frames(),
        measured.frames()
    );

    // And it drew what it claimed on the last of those fifty frames, so the zero is not a zero over
    // a frame that quietly stopped drawing.
    let shape = shape.expect("fifty frames ran");
    assert_eq!(shape.declared, dense::REGIONS);
    assert_eq!(driver.inspect().hits().len(), dense::REGIONS);
    assert_eq!(driver.inspect().stop_count(), 333);
}

/// **A two-hundred-millisecond collapse allocates nothing, over the frames of the transition
/// itself.**
///
/// Components ticket 22, and spec §8's own figure: *14 frames to quiet, 46.00 µs worst, 1 789 cells,
/// 0 allocations*. The frame count is a cadence and the µs and the cells belong to another screen —
/// `examples/collapsible_numbers.rs` prints all three beside §8's — but the **zero** is a count and a
/// count is a gate.
///
/// # The warm-up has to be a whole collapse and not four frames, and that is the finding
///
/// Every other allocation window in this workspace warms with *two identical frames*, because the
/// five frame structures take their allocation on the first frame that needs one and keep it. A
/// transition breaks that rule: each frame of it hands the body a **different rectangle**, so a
/// height nothing has drawn yet is a first touch inside the window. Warmed with four steady frames
/// this reads **1 allocation over 12 frames** — amortised zero, and exactly the shape
/// `crate::counters::Allocations` refuses to average away. Warmed on the *shape* — one whole collapse
/// and then the one that is measured — it is zero.
#[test]
fn a_two_hundred_millisecond_collapse_allocates_nothing_over_its_own_frames() {
    use vitui_alloc_probe::count_allocations;
    use vitui_components::accordion::{BODY_ROWS, Live};
    use vitui_components::disclose::DiscloseOpts;
    use vitui_components::ink::Direct;

    /// Sixty hertz, which is `scripts/steady-report.sh`'s own rate.
    const STEP: Duration = Duration::from_micros(16_667);
    /// §8's own collapse.
    const DUR: Duration = Duration::from_millis(200);

    let mut live = Live::new(1, DiscloseOpts::default());
    let mut ink = Direct;
    for _ in 0..2 {
        live.frame(&mut ink);
    }
    // The warm-up collapse: every rectangle the measured one will ask for, asked for once.
    live.collapse(0, DUR);
    while live.animating() > 0 {
        live.advance(STEP);
        live.frame(&mut ink);
    }
    live.reopen(0, BODY_ROWS);
    live.frame(&mut ink);

    live.collapse(0, DUR);
    let (frames, total) = count_allocations(|| {
        let mut n = 0u32;
        while live.animating() > 0 {
            live.advance(STEP);
            live.frame(&mut ink);
            n += 1;
        }
        n
    });

    assert_eq!(
        total, 0,
        "a collapse allocated {total} times over {frames} frames, and §8's figure is zero"
    );
    assert_eq!(
        frames, 12,
        "two hundred milliseconds at sixty hertz. §8 states fourteen at a cadence it does not state, \
         so the count is this cadence's and the relation is what `crate::disclose` gates"
    );
    assert_eq!(live.open(), 0, "and it arrived");
    assert_eq!(live.heights()[0], 0);
}

/// **A `field` over a megabyte allocates nothing in a steady frame**, which is components ticket
/// 24's own zero and §11's last frame column.
///
/// # It is warmed on the *shape* and not on two identical frames
///
/// Every other allocation window in this workspace warms with two identical frames, and that is
/// enough where the widget is handed the same rectangle every time. A field is: the caret does not
/// move on a steady frame, the index is keyed on `(revision, width)` and neither moved, and the row
/// loop reads the same rows. So the ordinary warming is the right one here — what it must **not**
/// do is build the index inside the window, because a `textarea` has that index already (§11) and
/// timing its construction would price the wrapping against the frame.
///
/// # The buffer is a megabyte and the window is twenty-four rows
///
/// Which is the whole claim: the frame costs the visible window and never the content, so the same
/// zero holds at twelve bytes and at a million. `crate::input::field_tests` asserts the counters
/// are equal at 100 kB and 1 MB; this asserts the ninth counter, which a library cannot read.
#[test]
fn a_steady_field_over_a_megabyte_allocates_nothing() {
    use vitui_components::edit::{Text, WrapKind};
    use vitui_components::input::field;
    use vitui_runtime::Rect;

    let mut st = Text::of(
        "the quick brown fox jumps over the lazy dog\n".repeat(24_000),
        WrapKind::Words,
    );
    assert!(st.text().len() > 1_000_000, "the buffer is not a megabyte");
    // The index the wrapping already needs, built before the window opens.
    let _ = st.index(W);

    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    let area = Rect::new(0, 0, W, H);
    let one_frame = |driver: &mut Driver, st: &mut Text| {
        driver.frame(|cx| {
            field(cx, area, st);
        });
    };
    one_frame(&mut driver, &mut st);
    one_frame(&mut driver, &mut st);

    steady(|| {
        for _ in 0..50 {
            one_frame(&mut driver, &mut st);
        }
    });

    assert_eq!(
        driver.inspect().hits().len(),
        1,
        "one hit entry for the widget, on the last of those fifty frames"
    );
}

/// **A full-screen picture allocates nothing, and it is the one screen where every cell is a
/// `Theme::custom`.**
///
/// Components ticket 29. Spec §14 states the allocation figure in the same sentence as the draw:
/// *24 000 `Theme::custom` calls a frame, 24 000 verbs, and no `fill` available at any size —
/// 261–357 µs of draw against a 100 µs budget, **zero allocations***. The µs are a timing and a
/// report; the zero is a count and a gate, and this is the only screen in the workspace that puts
/// 24 000 style constructions inside a window.
///
/// # Why the session is opened outside the window
///
/// `Driver::headless` attaches an engine and allocates its surfaces. Opened *inside* the window,
/// this reads **64 allocations a frame** — which is the attach, not the draw, and which is exactly
/// the shape `crate::counters::Allocations` refuses to average away in the other direction.
/// `vitui_components::picture::Session` is where the warm-up lives, so the thing being priced is a
/// steady frame and the number is not a report about the constructor.
#[test]
fn a_steady_frame_of_a_full_screen_picture_allocates_nothing_as_a_total() {
    use vitui_components::picture::{self, Build, Source};

    for build in [
        Build::correct(),
        Build::correct().of(Source::Gradient),
        // The Ascii rung, which draws a space with a background colour and is still 24 000 customs.
        Build::correct().at(vitui_components::chart::raster::RUNGS[0]),
    ] {
        let mut session = picture::Session::open(build);
        steady(|| {
            for _ in 0..50 {
                session.frame();
            }
        });
    }

    // And it drew what it claimed: 24 000 cells in 24 000 verbs with 24 000 customs, so the zero is
    // not a zero over a frame that quietly stopped drawing.
    let shape = picture::shape(Build::correct());
    assert_eq!(shape.writes, picture::WRITES);
    assert_eq!(shape.verbs, picture::VERBS);
    assert_eq!(shape.census.customs, picture::CUSTOMS);
}

/// **The player's chrome allocates nothing on a steady frame, and the shape it replaced allocates a
/// total no integer mean would show.**
///
/// Components ticket 30. Spec §21 records the defect this gate exists downstream of:
/// `player::chrome` collected a `Vec<f32>` of chapter positions on the draw path — **40 allocations
/// over 40 frames against a budget of zero**, *found only when the figure was computed as a total
/// rather than an integer mean*. The ticket's instruction is **do not fix it again; keep the shape
/// it replaced as a negative case**, and
/// [`vitui_components::media::player::defective::chrome_collecting_into`] is that case.
///
/// # The mean is what missed it, and this is why
///
/// The collected arm allocates on the frames that draw the **chapter list** — the band above the
/// transport block, which a short screen does not have room for. So over a run that is mostly short
/// the total is the defect and the arithmetic mean is a fraction the integer division throws away.
/// Both numbers are taken here, from the same run, so the argument is a pair of measured figures
/// rather than a sentence about the past.
#[test]
fn the_chrome_allocates_nothing_and_the_shape_it_replaced_allocates_a_total_no_mean_would_show() {
    use vitui_alloc_probe::count_allocations;
    use vitui_components::ink::Direct;
    use vitui_components::media::Census;
    use vitui_components::media::player::{Chapter, Player, chrome_into, defective};
    use vitui_runtime::Rect;

    /// How many frames the run is. Every one of them draws; only [`TALL`] of them are tall enough
    /// for the chapter list.
    const FRAMES: usize = 200;
    /// How many of [`FRAMES`] are tall. §21's own numerator.
    const TALL: usize = 40;

    let player = || {
        Player::new(
            3_672.0,
            vec![
                "01 - engine, cells and layers".to_owned(),
                "02 - the runtime seam".to_owned(),
            ],
            vec![
                Chapter {
                    at: 0.0,
                    name: "cold open".to_owned(),
                },
                Chapter {
                    at: 0.42,
                    name: "the seam".to_owned(),
                },
            ],
        )
    };

    // The two heights. The short one is exactly the transport block, so the lists band is empty.
    let short = Rect::new(0, 0, W, 4);
    let tall = Rect::new(0, 0, W, H);

    let mut driver = Driver::headless(W, H).expect("a sink attaches");
    let mut correct = player();
    let mut collecting = player();

    // Warm both arms: the frame structures take their allocation on the first frame that needs one,
    // and the two heights are two shapes.
    for area in [short, tall] {
        driver.frame(|cx| {
            chrome_into(&mut Direct, cx, area, &mut correct, &mut Census::default());
        });
        driver.frame(|cx| {
            defective::chrome_collecting_into(
                &mut Direct,
                cx,
                area,
                &mut collecting,
                &mut Census::default(),
            );
        });
    }

    let run = |driver: &mut Driver, arm: &mut dyn FnMut(&mut Driver, Rect)| {
        count_allocations(|| {
            for frame in 0..FRAMES {
                arm(driver, if frame < TALL { tall } else { short });
            }
        })
        .1
    };

    let shipped = run(&mut driver, &mut |driver: &mut Driver, area: Rect| {
        driver.frame(|cx| {
            chrome_into(&mut Direct, cx, area, &mut correct, &mut Census::default());
        });
    });
    assert_eq!(
        shipped, 0,
        "the chrome allocated {shipped} times over {FRAMES} frames"
    );

    let collected = run(&mut driver, &mut |driver: &mut Driver, area: Rect| {
        driver.frame(|cx| {
            defective::chrome_collecting_into(
                &mut Direct,
                cx,
                area,
                &mut collecting,
                &mut Census::default(),
            );
        });
    });
    assert_eq!(
        collected, TALL,
        "the collected arm is one `Vec<f32>` per frame that draws the chapter list"
    );
    // **And this is the figure that missed it.** `allocs / n` is the shape every prototype's report
    // used, and integer division over a run this shape reads zero for a defect that is really there.
    assert_eq!(
        collected / FRAMES,
        0,
        "an integer mean would have shown the defect, so the total is not what caught it"
    );
}

/// **The audio half adds no allocation and no construction: the bar ladder verbatim, and zero as a
/// total.**
///
/// Components ticket 30. §14: *the audio half adds no mechanism and no construction — §13's
/// 1 / 8 / 8 ladder verbatim, zero customs, zero allocations.* The customs are counted by
/// `vitui_components::media`'s own census; the zero here is the allocation half, and it is a total
/// over fifty frames of all three at once so that a construction allocating on some frames and not
/// others cannot average itself away.
///
/// The symbols are in the same window, because *zero* is the claim for the whole family and running
/// each alone would let one of them hide behind the warm-up of the next.
#[test]
fn a_steady_frame_of_the_media_family_allocates_nothing_as_a_total() {
    use vitui_components::ink::Direct;
    use vitui_components::media::{
        Census, Modules, Pixels, barcode_into, picture_into, qr_into, spectrum_into, vu_meter_into,
        waveform_into,
    };
    use vitui_runtime::{Rect, Rgb};

    struct Ramp;
    impl Pixels for Ramp {
        fn pixel(&self, x: u16, sub_y: u32) -> Rgb {
            Rgb::new((x % 251) as u8, (sub_y % 241) as u8, 0x80)
        }
    }

    let modules = Modules::new(21, (0..21 * 21).map(|i: u32| i.is_multiple_of(3)).collect());
    let bars: Vec<bool> = (0..W).map(|i| !i.is_multiple_of(5)).collect();
    let samples: Vec<f32> = (0..48_000)
        .map(|i| ((i as f32) / 137.0).sin() * (1.0 - i as f32 / 48_000.0))
        .collect();
    let bins: Vec<f32> = (0..W).map(|i| f32::from(i) / f32::from(W)).collect();

    let mut driver = Driver::headless(W, H).expect("a sink attaches");
    let band = |row: u16, h: u16| Rect::new(0, i32::from(row), W, h);
    let one_frame = |driver: &mut Driver| {
        driver.frame(|cx| {
            let mut census = Census::default();
            picture_into(
                &mut Direct,
                cx,
                band(0, 8),
                &Ramp,
                &Default::default(),
                &mut census,
            );
            qr_into(&mut Direct, cx, band(8, 5), &modules, &mut census);
            barcode_into(&mut Direct, cx, band(13, 2), &bars, &mut census);
            waveform_into(&mut Direct, cx, band(15, 4), &samples, &mut census);
            spectrum_into(&mut Direct, cx, band(19, 3), &bins, &mut census);
            vu_meter_into(&mut Direct, cx, band(22, 2), &bins, &mut census);
        });
    };

    one_frame(&mut driver);
    one_frame(&mut driver);
    steady(|| {
        for _ in 0..50 {
            one_frame(&mut driver);
        }
    });

    // And nothing in that window declared a region, which is the other half of what these six are:
    // pure drawers.
    assert_eq!(driver.inspect().hits().len(), 0);
}

/// **The preview pane's steady frame allocates nothing as a total**, with a worker in the frame and
/// a landing in its history.
///
/// Components ticket 32's criterion 8, and it could not be measured from inside `crate::preview`:
/// the counting allocator is `vitui-alloc-probe`, installed in this binary, which is register row
/// 32's arrangement.
///
/// **It is the component and not the screen.** `crate::preview::Screen` formats a `String` for
/// every list row and every document line and allocates
/// `crate::preview::SCREEN_ALLOCATIONS_A_FRAME` a frame — which is what an instrument does, and
/// `crate::ink` says so out loud: *an instrument allocating is not a defect.* §20's budget is the
/// component's, so the window goes over `files::file_preview_pane_into` with a line drawer that
/// stages rather than formats.
///
/// **The total and not the mean**, for §21 refinement 2's reason: a mean cannot see anything below
/// `n` and a total can see one. What is under the window is a component holding an asynchronous
/// answer — `Task::request` is called on every one of the fifty frames and boxes its job on none of
/// them, because the key does not change and the deduplicated call returns before it looks at it.
#[test]
fn a_steady_preview_frame_allocates_nothing_as_a_total() {
    use vitui_alloc_probe::count_allocations;
    use vitui_components::counters::Allocations;
    use vitui_components::preview::Alone;

    let mut alone = Alone::warmed();

    const FRAMES: u32 = 50;
    let (_, total) = count_allocations(|| {
        for _ in 0..FRAMES {
            alone.frame();
        }
    });
    let measured = Allocations::over(FRAMES, total as u64);
    assert_eq!(
        measured.total(),
        0,
        "{} allocations over {} frames of `file_preview_pane`. A mean would have reported 0 for \
         any total below {}",
        measured.total(),
        measured.frames(),
        measured.frames()
    );
}

/// **A slider allocates nothing, at either orientation and through both of its stepping arms.**
///
/// Components ticket 33. Spec §20's *zero allocations during frame composition*, and the figure is a
/// **total** rather than an integer mean — §21's own refinement 2, and the reason it is stated that
/// way is `crate::media::player`'s defect: a chrome that collected a `Vec` on the frames tall enough
/// to draw a chapter list allocated 40 times over 200 frames and reported `allocs / n == 0`.
///
/// A slider has no such conditional path — it is three runs and a face — so the interesting half is
/// that **the run drives the keyboard**, which is where a component that built a `String` for a
/// value, or collected its steps, would pay. Both arms are in one window, because *zero* is the claim
/// for the component and running each alone lets one hide behind the warm-up of the next.
#[test]
fn a_steady_frame_of_a_slider_allocates_nothing_as_a_total() {
    use vitui_alloc_probe::count_allocations;
    use vitui_components::ink::Direct;
    use vitui_components::input::{SliderOpts, defective, slider_into, slider_with};
    use vitui_components::scroll::Orient;
    use vitui_runtime::Mods;
    use vitui_runtime::keys::Code;

    /// How many frames the run is.
    const FRAMES: usize = 60;

    let horizontal = SliderOpts::default();
    let vertical = SliderOpts {
        orient: Orient::Vertical,
        ..SliderOpts::default()
    };
    let mut driver = Driver::headless(W, H).expect("a sink attaches");
    let mut a = 0.25f32;
    let mut b = 0.75f32;
    let mut c = 0.5f32;

    // One frame of each shape, to warm every structure the frame takes its own allocation for. Two
    // orientations and two stepping arms are four shapes and one call site each, which is what keeps
    // them four widgets rather than one.
    let warm_or_measure = |driver: &mut Driver, a: &mut f32, b: &mut f32, c: &mut f32| {
        driver.frame(|cx| {
            let area = cx.area();
            let (left, right) = split_h(area, area.w / 2);
            slider_with(cx, top_row(left), a, &horizontal);
            slider_into(&mut Direct, cx, right, b, &vertical);
            defective::float_stepped_into(&mut Direct, cx, bottom_row(left), c, &horizontal);
        });
    };
    // **And the warm-up posts a key**, because the frame's key queue takes its own first allocation
    // on the first key that reaches it. Warmed by drawing alone, this window read **1 over 60
    // frames** — a queue's `Vec` growing once, attributed to the slider. It is components ticket 22's
    // warming discipline stated as a rule: *the window must warm the path it prices*, and a warm-up
    // that draws but never presses is not warming the drain loop at all.
    for _ in 0..2 {
        driver.post_key(vitui_components::keys::press_with(Code::Right, Mods::NONE));
        warm_or_measure(&mut driver, &mut a, &mut b, &mut c);
    }

    let total = count_allocations(|| {
        for frame in 0..FRAMES {
            // **The keyboard is driven inside the window**, because the arrow is the path a value
            // that formatted itself would allocate on. Nothing holds the focus, so the keys reach no
            // widget — which is the honest arrangement: what is being priced is the drain loop and
            // the draw, and seating a focus would price the runtime's award as well.
            driver.post_key(vitui_components::keys::press_with(
                if frame % 2 == 0 {
                    Code::Right
                } else {
                    Code::Left
                },
                Mods::NONE,
            ));
            warm_or_measure(&mut driver, &mut a, &mut b, &mut c);
        }
    })
    .1;
    assert_eq!(
        total, 0,
        "three sliders allocated {total} times over {FRAMES} frames"
    );
}

/// **The six Tier 2 components allocate nothing, as a total over sixty frames.**
///
/// Components ticket 34, criterion 7. Spec §20's *zero allocations during frame composition*, and
/// the figure is a **total** rather than an integer mean — §21's refinement 2, and the reason it is
/// stated that way is `crate::media::player`'s defect: a chrome that collected a `Vec` on the frames
/// tall enough to draw a chapter list allocated 40 times over 200 frames and reported
/// `allocs / n == 0`.
///
/// **All six in one window**, because *zero* is the claim for the tier and running each alone lets
/// one hide behind the next one's warm-up. The two that could plausibly pay are the two that are
/// `chart`: a sparkline's raster is the caller's `PlotState` and its row buffer is the same field
/// `chart` reuses, so a frame that re-folded or re-allocated the row would show here and nowhere
/// else.
///
/// **The window warms the path it prices** — components ticket 22's discipline, met for the fourth
/// time on this map: the warm-up posts a key, because the frame's key queue takes its own first
/// allocation on the first key that reaches it, and a toggle reads the keyboard.
#[test]
fn a_steady_frame_of_the_six_tier_two_components_allocates_nothing_as_a_total() {
    use vitui_alloc_probe::count_allocations;
    use vitui_components::chart::{Series, raster::PlotState};
    use vitui_components::counters::Allocations;
    use vitui_components::indicate::{MeterOpts, SparkOpts, meter_with, sparkline_with};
    use vitui_components::input::{Toggle, ToggleOpts, toggle_with};
    use vitui_components::scroll::Orient;
    use vitui_components::structure::{RuleOpts, rule_with};
    use vitui_components::text::Justify;
    use vitui_runtime::Mods;
    use vitui_runtime::keys::Code;

    /// How many frames the run is.
    const FRAMES: usize = 60;

    let data = Series::build(10_000, 1);
    let mut spark = PlotState::new();
    let mut driver = Driver::headless(W, H).expect("a sink attaches");
    let (mut check, mut radio, mut switch) = (true, false, true);
    let horizontal = MeterOpts::default();
    let vertical = MeterOpts {
        orient: Orient::Vertical,
        ..MeterOpts::default()
    };

    let once = |driver: &mut Driver,
                check: &mut bool,
                radio: &mut bool,
                switch: &mut bool,
                spark: &mut PlotState| {
        driver.frame(|cx| {
            let area = cx.area();
            let (left, right) = split_h(area, area.w / 2);
            toggle_with(cx, top_row(left), "wrap", check, &ToggleOpts::default());
            toggle_with(
                cx,
                bottom_row(left),
                "one",
                radio,
                &ToggleOpts {
                    kind: Toggle::Radio,
                    ..ToggleOpts::default()
                },
            );
            toggle_with(
                cx,
                top_row(right),
                "dark",
                switch,
                &ToggleOpts {
                    kind: Toggle::Switch,
                    ..ToggleOpts::default()
                },
            );
            meter_with(cx, bottom_row(right), 0.4375, &horizontal);
            let (_, rest) = vitui_runtime::layout::rect::split_at_v(area, 4);
            let (bar, body) = vitui_runtime::layout::rect::split_at_h(rest, 4);
            meter_with(cx, bar, 0.62, &vertical);
            let (line, chart) = vitui_runtime::layout::rect::split_at_v(body, 1);
            rule_with(
                cx,
                line,
                " load ",
                &RuleOpts {
                    justify: Justify::Middle,
                    ..RuleOpts::default()
                },
            );
            sparkline_with(cx, chart, &data, spark, &SparkOpts::default());
        });
    };

    // Two warm frames, and each of them posts a key: a warm-up that draws but never presses is not
    // warming the drain loop a toggle reads its `Space` out of.
    for _ in 0..2 {
        driver.post_key(vitui_components::keys::press_with(Code::Tab, Mods::NONE));
        once(&mut driver, &mut check, &mut radio, &mut switch, &mut spark);
    }

    let total = count_allocations(|| {
        for _ in 0..FRAMES {
            driver.post_key(vitui_components::keys::press_with(Code::Tab, Mods::NONE));
            once(&mut driver, &mut check, &mut radio, &mut switch, &mut spark);
        }
    })
    .1;
    let measured = Allocations::over(FRAMES as u32, total as u64);
    assert_eq!(
        measured.total(),
        0,
        "the six Tier 2 components allocated {} times over {} frames. A mean would have reported 0 \
         for any total below {}",
        measured.total(),
        measured.frames(),
        measured.frames()
    );
    // And the memo really did run: one fold, however many frames.
    assert_eq!(spark.misses(), 1);
}

/// **The three Tier 2 composites allocate nothing, as a total over sixty frames — and the shape one
/// of them replaced allocates once a frame.**
///
/// Components ticket 35, criterion 8. Spec §20's *zero allocations during frame composition*, as a
/// **total** rather than an integer mean (§21's refinement 2).
///
/// **The negative case is the point of this one.** [`crate::nav::cursor`] takes `&[&str]`, so a form
/// written over a slice of *records* — a label and a `Text` in one struct, which reads better and is
/// what a reviewer expects — has to build that slice every frame. The picture is **identical**
/// either way, the writes and the verbs are identical either way, and the only instrument in this
/// workspace that can tell the two apart is this window. Measured in one run: shipped **0**,
/// record-shaped **60 over 60**.
///
/// **The window warms the path it prices** — components ticket 22's discipline, met for the fifth
/// time here: the warm-up posts a key, because a form drains the keyboard and the frame's key queue
/// takes its own first allocation on the first key that reaches it.
#[test]
fn the_three_tier_two_composites_allocate_nothing_and_the_record_shaped_form_allocates_a_frame() {
    use vitui_alloc_probe::count_allocations;
    use vitui_components::collect::{CollState, PageOpts, pagination_with};
    use vitui_components::counters::Allocations;
    use vitui_components::edit::Text;
    use vitui_components::input::{FormOpts, FormState, defective, form_with};
    use vitui_components::structure::{StatusOpts, status_bar_with};
    use vitui_runtime::Mods;
    use vitui_runtime::keys::Code;

    /// How many frames the run is.
    const FRAMES: usize = 60;
    const LABELS: [&str; 4] = ["name", "email", "role", "team"];

    let mut driver = Driver::headless(W, H).expect("a sink attaches");
    let mut form_state = FormState::new();
    let mut pages = CollState::new();
    let mut texts = [Text::input(), Text::input(), Text::input(), Text::input()];
    let bar = StatusOpts::default();
    let page = PageOpts::default();
    let shape = FormOpts::default();

    let once = |driver: &mut Driver,
                form_state: &mut FormState,
                pages: &mut CollState,
                texts: &mut [Text],
                collected: bool| {
        driver.frame(|cx| {
            let area = cx.area();
            let (rows, rest) = vitui_runtime::layout::rect::split_at_v(area, 4);
            let (strip, tail) = vitui_runtime::layout::rect::split_at_v(rest, 1);
            match collected {
                true => defective::form_collecting_labels(
                    &mut vitui_components::ink::Direct,
                    cx,
                    rows,
                    form_state,
                    &LABELS,
                    texts,
                    &shape,
                ),
                false => form_with(cx, rows, form_state, &LABELS, texts, &shape),
            };
            pagination_with(cx, strip, pages, 137, &page);
            status_bar_with(
                cx,
                vitui_runtime::layout::rect::split_at_v(tail, 1).0,
                &["ready", "utf-8", "ln 1"],
                (0, 0),
                &bar,
            );
        });
    };

    // Two warm frames each, and both post a key: a warm-up that draws but never presses is not
    // warming the drain loop a form reads its arrows out of.
    for collected in [false, true] {
        for _ in 0..2 {
            driver.post_key(vitui_components::keys::press_with(Code::Tab, Mods::NONE));
            once(
                &mut driver,
                &mut form_state,
                &mut pages,
                &mut texts,
                collected,
            );
        }
    }

    let shipped = count_allocations(|| {
        for _ in 0..FRAMES {
            driver.post_key(vitui_components::keys::press_with(Code::Tab, Mods::NONE));
            once(&mut driver, &mut form_state, &mut pages, &mut texts, false);
        }
    })
    .1;
    let record_shaped = count_allocations(|| {
        for _ in 0..FRAMES {
            driver.post_key(vitui_components::keys::press_with(Code::Tab, Mods::NONE));
            once(&mut driver, &mut form_state, &mut pages, &mut texts, true);
        }
    })
    .1;

    let measured = Allocations::over(FRAMES as u32, shipped as u64);
    assert_eq!(
        measured.total(),
        0,
        "the three Tier 2 composites allocated {} times over {} frames. A mean would have reported \
         0 for any total below {}",
        measured.total(),
        measured.frames(),
        measured.frames()
    );
    // **And the refused shape is watched paying**, in the same run and on the same screen, so the
    // zero above is a measurement rather than a warm-up.
    assert_eq!(
        record_shaped, FRAMES,
        "the record-shaped form allocated {record_shaped} times over {FRAMES} frames, and the \
         claim is exactly one a frame — the `Vec<&str>` `nav::cursor`'s signature forces"
    );
}

/// **The assembled gallery allocates nothing on a steady frame, as a total.**
///
/// Components ticket 39. Every other allocation gate in this file prices one component or one
/// composite; this one prices **twenty-eight of them at once**, which is the criterion §20 states as
/// *the performance budget holds in the gallery, not only in isolated harnesses*.
///
/// # Two things had to be written around, and both are the screen rather than the budget
///
/// **The status line is the caller's `String` and is passed in by reference.** `Gallery::ui_into`
/// takes the note as a `&str`, so the loop below hands it a `&'static str` and the frame path
/// formats nothing it did not already have — an application that cloned its status text per frame
/// would land in this number, which is why the application destructures instead.
///
/// **The window is warmed on the shape it prices.** Six of the twenty-eight keep a memo — the two
/// charts, the sparkline, the wrap index, the flatten index and the preview — and every one of them
/// takes its allocation on the first frame that needs it. Components 22 measured what warming on the
/// wrong shape costs: **1 over 12**, which is amortised zero and exactly what `Allocations`'s
/// missing `mean` exists to refuse.
///
/// **The chrome is staged, not formatted.** The heading and the status bar's left segment are two
/// `String`s the gallery keeps and rewrites with `write!` after `clear()`, which is what makes the
/// total below cover the *whole* frame rather than only the tiles: a `format!` per frame is two
/// allocations, and a gate that excluded the chrome would be a gate on a screen nobody draws.
#[test]
fn a_steady_frame_of_the_gallery_allocates_nothing_as_a_total() {
    use vitui_alloc_probe::count_allocations;
    use vitui_components::counters::Allocations;
    use vitui_components::gallery::{Gallery, Sink};
    use vitui_components::ink::Direct;
    use vitui_components::runner::driver_at;
    use vitui_runtime::Density;
    use vitui_runtime::work::Worker;

    let (w, h) = (100u16, 30u16);
    let mut driver = driver_at(w, h, Density::default());
    let mut gallery = Gallery::new(Worker::queueing());
    driver.set_theme(*gallery.theme());
    let one = |driver: &mut Driver, gallery: &mut Gallery| {
        gallery.bag.answer_queued();
        let mut sink: Sink<'_> = &mut Direct;
        driver.frame(|cx| gallery.ui_into(&mut sink, cx, "steady"));
    };
    // Warmed with the identical workload, twice, before the window opens.
    one(&mut driver, &mut gallery);
    one(&mut driver, &mut gallery);

    const FRAMES: u32 = 50;
    let (_, total) = count_allocations(|| {
        for _ in 0..FRAMES {
            one(&mut driver, &mut gallery);
        }
    });
    let measured = Allocations::over(FRAMES, total as u64);
    assert_eq!(
        measured.total(),
        0,
        "twenty-eight components on one screen allocated {} times over {} frames. A mean would \
         have reported 0 for any total below {}",
        measured.total(),
        measured.frames(),
        measured.frames()
    );

    // **And every page**, because the panels that keep a memo are not all on page one and a window
    // over one page prices twelve of the twenty-eight.
    for _ in 0..vitui_components::gallery::pages(w, h) {
        gallery.next_page(w, h);
        one(&mut driver, &mut gallery);
        one(&mut driver, &mut gallery);
        let (_, total) = count_allocations(|| {
            for _ in 0..FRAMES {
                one(&mut driver, &mut gallery);
            }
        });
        assert_eq!(
            Allocations::over(FRAMES, total as u64).total(),
            0,
            "page {} of the gallery allocates on a steady frame",
            gallery.paging(w, h).0 + 1
        );
    }
}

/// The top row of `area`, which is the shape a horizontal slider is drawn in.
fn top_row(area: vitui_runtime::Rect) -> vitui_runtime::Rect {
    vitui_runtime::layout::rect::split_at_v(area, 1).0
}

/// The row under it, so the two horizontal sliders are two rectangles rather than one.
fn bottom_row(area: vitui_runtime::Rect) -> vitui_runtime::Rect {
    let (_, rest) = vitui_runtime::layout::rect::split_at_v(area, 1);
    vitui_runtime::layout::rect::split_at_v(rest, 1).0
}

/// `area` cut in two at `at` columns.
fn split_h(area: vitui_runtime::Rect, at: u16) -> (vitui_runtime::Rect, vitui_runtime::Rect) {
    vitui_runtime::layout::rect::split_at_h(area, at)
}
