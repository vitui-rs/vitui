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
