//! **The instruments `crate::gates::REGISTER` names, run from a crate that cannot name the engine.**
//!
//! Components ticket 03. The register one module over is a value; this file is what makes its rows
//! `Evaluated` rather than prose. Everything here is written the way a component would have to write
//! it — every rectangle comes out of `Ctx::area` and the layout solver, because a package whose
//! dependency list is `vitui-runtime` and nothing else has no other way to obtain one (§19's
//! constraint C6, `tests/crate_line.rs`).
//!
//! # Why the allocation gates are in this binary and not in `tests/budget.rs`
//!
//! [`vitui_alloc_probe::CountingAllocator`] is process-global, so every gate that reads it needs
//! `--test-threads=1` — which `.gitlab-ci.yml`'s `test` job already runs. `budget.rs` prices the two
//! **standing budget counts** a component inherits; this file prices §21's rule that *a mean cannot
//! see anything below `n`*, which is a different claim and needs a fixture that allocates on exactly
//! one frame of two hundred.

use std::hint::black_box;

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::counters::{Allocations, Counter, Counters, Tally};
use vitui_components::frame::block;
use vitui_components::text::{FitOpts, Justify, fit, fit_with};
use vitui_runtime::ctx::{Ctx, Driver};
use vitui_runtime::focus::ScopeKind;
use vitui_runtime::layout::{
    Col,
    Constraint::{Fixed, Weight},
};
use vitui_runtime::{Id, Interest, Role};

#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// The screen §8's accordion was measured on, at the size this crate can drive headlessly.
const W: u16 = 120;
/// See [`W`].
const H: u16 = 40;

/// Twelve sections, which is §8's accordion.
const SECTIONS: u64 = 12;
/// Five focusables in a section's body, so a body that does not cull is visible as a multiple.
const IN_BODY: u64 = 5;

// ── §21's mechanism form of the tab-stop gate ────────────────────────────────────────────────────

/// Twelve closed sections. `cull` is the rule under test: **a body is handed a rectangle and draws
/// inside it** (§8).
///
/// The body's rectangle is `Fixed(0)` tall, which is what *closed* means when the height is an
/// argument (ADR 0014 — the runtime has no measure pass, so a collapsible's height is computed by
/// its caller and handed down).
fn accordion(cx: &mut Ctx<'_, '_>, cull: bool) {
    let band = cx.area();
    let mut sections = [band; SECTIONS as usize];
    let n = Col::new().split_into(band, &[Weight(1); SECTIONS as usize], &mut sections);
    assert_eq!(n, SECTIONS as usize);
    for (i, section) in sections[..n].iter().enumerate() {
        cx.with_key(i as u64, |cx| {
            let [header, body] = Col::new().split(*section, [Fixed(1), Fixed(0)]);
            let head = cx.id();
            let _ = cx.interact(head, header, Interest::CLICK.with(Interest::FOCUS));
            if cull && body.h == 0 {
                return;
            }
            for row in 0..IN_BODY {
                cx.with_key(row, |cx| {
                    let id = cx.id();
                    let _ = cx.interact(id, body, Interest::CLICK.with(Interest::FOCUS));
                });
            }
        });
    }
}

/// **A component drawn into a zero-height rectangle declares no tab stops** — §8's 478-against-70,
/// stated so it can be run against anything.
///
/// # This is the mechanism form, and the geometric form C14 asked for is a different row
///
/// *Tab stops == visible focusables* needs to know what is on screen, and
/// [`the_ring_carries_the_declared_rectangle_and_not_the_visible_one`] is why the ring cannot
/// answer that. The mechanism form needs no geometry at all: it compares a body that culls against
/// one that does not, on the same frame, and the difference is the rule.
///
/// # Both directions, because the failing half is the one that is invisible
///
/// `Ctx::interact` appends a hit entry and a ring entry **before** it looks at the rectangle, so a
/// body drawn at `h = 0` declares every stop it would have declared open and **no golden-cell gate
/// can see it** — the surfaces are identical. §8 measured 478 hit entries against 70 and 423 tab
/// stops against 15. The numbers below are this fixture's twelve sections at five focusables each,
/// and they are the same shape: 72 against 12 either way.
#[test]
fn a_component_drawn_into_a_zero_height_rectangle_declares_no_tab_stops() {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");

    driver.frame(|cx| accordion(cx, true));
    let culled = Counters::of(&driver, &Tally::new(), Allocations::over(1, 0));
    assert_eq!(
        culled.tab_stops.get(Counter::TabStops),
        SECTIONS,
        "a culling body declares its header and nothing else"
    );
    assert_eq!(culled.regions.get(Counter::Regions), SECTIONS);
    assert_eq!(culled.merges.get(Counter::Merges), 0);

    // The other direction. Without it the gate passes on a frame that drew nothing at all.
    driver.frame(|cx| accordion(cx, false));
    let uncut = Counters::of(&driver, &Tally::new(), Allocations::over(1, 0));
    assert_eq!(
        uncut.tab_stops.get(Counter::TabStops),
        SECTIONS * (1 + IN_BODY),
        "a body that does not cull declares every stop it would have declared open"
    );
    assert_eq!(
        uncut.regions.get(Counter::Regions),
        SECTIONS * (1 + IN_BODY)
    );
}

/// **The ring carries the rectangle a widget *declared*, not the one that is on screen** — which is
/// why *tab stops == visible focusables* cannot be written here.
///
/// # The correction this test exists to record
///
/// Spec §21 says the geometric form is impossible because *the ring does not carry geometry* —
/// `RingEnt::rect` is zero unless `Frame::ring_geometry` is on and the entry was declared inside a
/// scroll area. **That is the prototype's type and not the shipped one.** `vitui_runtime::focus::Stop`
/// has a `pub rect` that `Ctx::interact` fills on **every** entry, unconditionally, and this crate
/// can read it: `Rect` cannot be *named* here but a `Stop`'s field can be *read*.
///
/// The conclusion is unchanged and the reason is better. The rectangle recorded is the one passed to
/// `interact`, translated by the enclosing scroll offset and **not intersected with anything** — not
/// with the enclosing `Ctx::child` clip, not with the scroll area's viewport, not with an overlay
/// standing over it. So a stop whose rectangle is entirely outside its own context is declared at
/// full size, which is exactly the case the geometric form would have to catch and exactly the case
/// the ring cannot distinguish.
#[test]
fn the_ring_carries_the_declared_rectangle_and_not_the_visible_one() {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    driver.frame(|cx| {
        let band = cx.area();
        let [top, rest] = Col::new().split(band, [Fixed(2), Weight(1)]);
        // A container two rows tall, handed a body twenty rows tall — the shape a component that
        // forgets to cull produces, and the one the geometric form exists to catch.
        let mut inner = cx.child(top);
        let id = inner.id();
        let _ = inner.interact(id, rest, Interest::FOCUS);
    });

    let ring = driver.inspect().ring();
    assert_eq!(ring.len(), 1);
    assert_eq!(
        ring[0].rect.h,
        H - 2,
        "the ring recorded the declared height, not the two rows the context had"
    );
    assert!(
        ring[0].rect.y >= 2,
        "and it begins below the context that declared it, so nothing about the entry is inside \
         the rectangle the widget was handed"
    );
}

// ── §21's third refinement: name the exception, do not loosen the gate ───────────────────────────

/// Five focusables, optionally with a `Trap` around the last two. A modal is what a standing trap is
/// (`ScopeKind::Trap`).
fn walkable(cx: &mut Ctx<'_, '_>, trapped: bool) {
    let band = cx.area();
    let mut lanes = [band; 5];
    let n = Col::new().split_into(band, &[Weight(1); 5], &mut lanes);
    for (i, lane) in lanes[..3].iter().enumerate() {
        cx.with_key(i as u64, |cx| {
            let id = cx.id();
            let _ = cx.interact(id, *lane, Interest::FOCUS);
        });
    }
    let tail = &lanes[3..n];
    let body = |cx: &mut Ctx<'_, '_>| {
        for (i, lane) in tail.iter().enumerate() {
            cx.with_key(3 + i as u64, |cx| {
                let id = cx.id();
                let _ = cx.interact(id, *lane, Interest::FOCUS);
            });
        }
    };
    if trapped {
        cx.scope(Id::named("modal"), ScopeKind::Trap, body);
    } else {
        body(cx);
    }
}

/// **The walk repeats no id, and reaches every stop unless a trap is standing** — §21's refinement
/// 3, as the conjunction rather than as the loosened gate.
///
/// R08's *a walkthrough visits every tab stop exactly once* fails on exactly one panel of twelve and
/// **correctly**: §12's dialog is open, and a `Trap` is what a modal is — 8 stops declared, 2
/// visited. The wrong fix is to weaken the gate to *no repeats*, which passes on a walk that reaches
/// one stop of eight. `Frame::trap_scopes()` exists so the exception can be **named**, and it is the
/// only thing that can answer it.
///
/// Both halves are asserted in both directions below: the untrapped frame reaches everything, the
/// trapped frame does not and says which scope stopped it, and the loosened spelling is shown
/// passing on the case it must not.
#[test]
fn the_walk_repeats_no_id_and_reaches_every_stop_unless_a_trap_is_standing() {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");

    let walk_of = |driver: &Driver| -> (Vec<Id>, Vec<Id>, Vec<Id>) {
        let frame = driver.inspect();
        (
            frame.tab_walk().collect(),
            frame.stop_ids().collect(),
            frame.trap_scopes().collect(),
        )
    };
    let repeats = |walk: &[Id]| {
        let mut seen = walk.to_vec();
        seen.sort_by_key(|id| id.raw());
        let before = seen.len();
        seen.dedup();
        before - seen.len()
    };

    driver.frame(|cx| walkable(cx, false));
    let (walk, stops, traps) = walk_of(&driver);
    assert_eq!(repeats(&walk), 0, "the walk repeats no id");
    assert!(traps.is_empty(), "no trap is standing");
    assert_eq!(
        walk.len(),
        stops.len(),
        "so it must reach every stop: {} of {}",
        walk.len(),
        stops.len()
    );
    assert_eq!(walk.len(), 5);

    // The exception, named rather than excused.
    driver.frame(|cx| walkable(cx, true));
    let (walk, stops, traps) = walk_of(&driver);
    assert_eq!(repeats(&walk), 0, "the walk still repeats no id");
    assert_eq!(stops.len(), 5, "five stops were declared");
    assert_eq!(
        walk.len(),
        2,
        "and the trap holds the walk to its own range"
    );
    assert_eq!(
        traps,
        vec![Id::named("modal")],
        "the exception has a name, which is the whole of refinement 3"
    );

    // **And the loosened spelling, kept as the negative case.** *No repeats* alone is green on a
    // walk that reaches two stops of five, and would stay green on one that reached zero.
    assert_eq!(repeats(&walk), 0);
    assert!(
        walk.len() < stops.len(),
        "which is exactly the state the loosened gate cannot see"
    );
}

// ── §21's second refinement: a total, never a mean ───────────────────────────────────────────────

/// How many frames the allocation total is taken over. Two hundred is the top of the range the
/// prototypes divided by (§21: `n` between 40 and 200).
const FRAMES: u32 = 200;

/// A steady frame a component can write: a band from `Ctx::area`, lanes from the solver, one
/// interactive region a lane.
fn steady_frame(cx: &mut Ctx<'_, '_>) {
    let band = cx.area();
    let body = cx.theme().paint(Role::Body);
    let mut lanes = [band; 20];
    let n = Col::new().split_into(band, &[Weight(1); 20], &mut lanes);
    for (i, lane) in lanes[..n].iter().enumerate() {
        cx.text(lane.x, lane.y, "a row that does not move", body);
        cx.with_key(i as u64, |cx| {
            let id = cx.id();
            let _ = cx.interact(id, *lane, Interest::CLICK.with(Interest::FOCUS));
        });
    }
}

/// **Zero allocations over two hundred frames, as a total.**
///
/// Warmed with two frames before the window opens, for the reason `vitui_alloc_probe::steady`
/// documents: the five frame structures take their allocation on the first frame that needs one and
/// keep it, so a window containing first touch is measuring the loader.
///
/// The number reported is `total`, and [`vitui_components::counters::Allocations`] has no `mean` for
/// it to be divided by — see [`a_total_sees_one_frame_in_two_hundred_and_a_mean_cannot`].
#[test]
fn a_steady_component_frame_allocates_nothing_as_a_total_over_the_run() {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    driver.frame(steady_frame);
    driver.frame(steady_frame);

    let (_, total) = count_allocations(|| {
        for _ in 0..FRAMES {
            driver.frame(steady_frame);
        }
    });

    let seen = Allocations::over(FRAMES, total as u64);
    assert_eq!(
        seen.total(),
        0,
        "{} allocations over {} frames, and the budget is a total of zero",
        seen.total(),
        seen.frames()
    );
    let counters = Counters::of(&driver, &Tally::new(), seen);
    assert_eq!(counters.allocations.get(Counter::Allocations), 0);
}

/// **A mean cannot see anything below `n`; a total can see one.** §21's refinement 2, measured
/// rather than argued.
///
/// The fixture allocates on **exactly one frame of two hundred** — `player::chrome`'s defect at the
/// amplitude that survives, rather than its actual 40 over 40 — and the two arithmetics disagree:
/// the total is at least one and the integer mean every prototype printed is zero. That is the whole
/// of why the gate above is written as a total, and why `Allocations` exposes no division.
#[test]
fn a_total_sees_one_frame_in_two_hundred_and_a_mean_cannot() {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    // Warmed on the quiet path, exactly as the gate above is: the frame that allocates is the
    // measurement and warming it would be warming the defect away.
    driver.frame(steady_frame);
    driver.frame(steady_frame);

    let (_, total) = count_allocations(|| {
        for n in 0..FRAMES {
            let allocate = n == 137;
            driver.frame(|cx| {
                steady_frame(cx);
                if allocate {
                    // The shape §20 records: a `Vec` collected on the draw path.
                    let owed: Vec<f32> = (0..8).map(|i| i as f32).collect();
                    black_box(owed);
                }
            });
        }
    });

    assert!(
        total >= 1,
        "the total must see the one frame that allocated"
    );
    assert_eq!(
        total as u32 / FRAMES,
        0,
        "and the mean every prototype printed cannot: {total} / {FRAMES}"
    );
}

/// **`alloc_zeroed` is not a blind spot, and this is a test rather than a sentence.**
///
/// > *Twelve of twelve omit a method* is exactly the shape of a claim the next reader files as a
/// > defect. (§21)
///
/// `GlobalAlloc::alloc_zeroed`'s default implementation calls `self.alloc` and zeroes the block, so
/// a counting allocator that overrides only `alloc` still counts a zeroed allocation — and
/// `vitui-alloc-probe` overrides it anyway. **Neither fact is observable from a sentence**, and both
/// give the same observable, which is what this asserts: `vec![0u8; n]` — the standard library's one
/// route to `alloc_zeroed`, via its `IsZero` specialisation — costs exactly one counted allocation,
/// the same as the plain `alloc` beside it.
#[test]
fn a_zeroed_allocation_is_counted_exactly_once_like_any_other() {
    // Warm anything the harness lazily initialises, so the two windows below hold one event each.
    black_box(vec![0u8; 64]);
    black_box(Vec::<u8>::with_capacity(64));

    let (zeroed, via_alloc_zeroed) = count_allocations(|| vec![0u8; 4096]);
    let (plain, via_alloc) = count_allocations(|| Vec::<u8>::with_capacity(4096));
    black_box(zeroed);
    black_box(plain);

    assert_eq!(
        via_alloc_zeroed, 1,
        "a zeroed allocation is counted once, whether the probe overrides `alloc_zeroed` or the \
         default forwards to `self.alloc`"
    );
    assert_eq!(via_alloc, 1, "and so is the ordinary one beside it");
}

// ── The wheel gate's reachable half ──────────────────────────────────────────────────────────────

/// **An offset that moves when nothing asked is a failure** — the second direction of the gate
/// ticket 20 inverts, and for four tickets the only one of the two this crate could run.
///
/// The first direction — *twenty wheel clicks move the offset twenty* — needs a posted `Mouse`,
/// which was `EngineName { name: "Mouse", reachable_as: None }`. **Runtime architecture issue 22
/// lifted that**, so components 20 can now drive the real channel; this half is unchanged because it
/// never needed the pointer. See `crate::gates::REGISTER`'s row for the standing.
///
/// This half needs no pointer at all: `Ctx::request_into_view` is the only door a reveal may
/// take (ADR 0015 — no geometry crosses a frame), so a frame that asks for nothing must leave
/// `Frame::into_view()` empty, and a one-directional gate written the other way round would go green
/// the moment somebody deleted the call entirely.
#[test]
fn a_frame_that_asks_for_no_reveal_moves_no_offset_and_one_that_asks_does() {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");

    let area = Id::named("list");
    driver.frame(|cx| {
        let band = cx.area();
        let [view, _below] = Col::new().split(band, [Fixed(5), Weight(1)]);
        cx.scroll_scope(area, view, (0, 0), (0, 400), |cx| {
            let inner = cx.area();
            let id = cx.id();
            let _ = cx.interact(id, inner, Interest::FOCUS);
        });
    });
    assert!(
        driver.inspect().into_view().is_none(),
        "nothing asked, so nothing may move — this is the direction that catches an unconditional \
         scroll-into-view, which four resolved tickets wrote after reading the rule forbidding it"
    );

    // And the other direction, so that deleting the call is not a way to pass.
    driver.frame(|cx| {
        let band = cx.area();
        let [view, below] = Col::new().split(band, [Fixed(5), Weight(1)]);
        cx.scroll_scope(area, view, (0, 0), (0, 400), |cx| {
            // A rectangle under the viewport, which is what a keyboard cursor walking off the
            // bottom of a list looks like. Every coordinate here came from the solver.
            cx.request_into_view(below);
        });
    });
    let asked = driver
        .inspect()
        .into_view()
        .expect("a reveal that was asked for arrives");
    assert_eq!(asked.area, area);
    assert_ne!(asked.by, (0, 0), "and it asks for a real movement");
}

// ── §21's first refinement, kept as a negative case ──────────────────────────────────────────────

/// **A threshold on the wrong side of the question is not a weak gate, it is a green one.**
///
/// The gallery's theme-swap gate asserted `changed > 0` — *the theme changed and not one cell moved*
/// — and **one cell of 4 800 satisfies it while 3 583 carry the old palette** (§21, ticket 41). The
/// old spelling is kept here as a negative case rather than deleted, because the sentence alone has
/// not stopped anybody: the correct form asserts the **complement**, which is a count over the
/// surface and not a threshold on a delta.
///
/// The surface count itself is unreachable from this crate — see `crate::gates::REGISTER`'s palette
/// row and `vitui_components::counters::sentinel` for the three barriers — so what is asserted here
/// is the arithmetic that made the wrong spelling green, on the map's own recorded numbers.
#[test]
fn changed_greater_than_zero_passes_on_the_exact_set_it_had_to_catch() {
    /// One screen after a swap: how many cells moved, and how many still carry the old palette.
    struct Swap {
        cells: u32,
        changed: u32,
        stale: u32,
    }

    /// The spelling that shipped. **A threshold on the delta.**
    fn the_gate_that_was_there(s: &Swap) -> bool {
        s.changed > 0
    }
    /// The spelling ticket 41 owes. **A count over the surface, on the complement.**
    fn the_gate_it_should_have_been(s: &Swap) -> bool {
        s.stale == 0
    }

    // §21's recorded numbers: one cell of 4 800 moved and 3 583 are wrong for as long as the
    // application runs.
    let broken = Swap {
        cells: 4_800,
        changed: 1,
        stale: 3_583,
    };
    assert!(broken.changed + broken.stale <= broken.cells);
    assert!(
        the_gate_that_was_there(&broken),
        "the shipped spelling is green on the screen it exists to catch, which is the whole of \
         refinement 1"
    );
    assert!(
        !the_gate_it_should_have_been(&broken),
        "and the complement form is red on it"
    );

    // The other direction, so this is a comparison and not a demonstration: on a screen the swap
    // really reached, the two agree.
    let fixed = Swap {
        cells: 4_800,
        changed: 4_800,
        stale: 0,
    };
    assert!(the_gate_that_was_there(&fixed));
    assert!(the_gate_it_should_have_been(&fixed));

    // And the case that separates them the other way round: a swap between two themes that agree
    // about this screen moves nothing and leaves nothing stale. The threshold form fails it and the
    // complement form passes, which is why the two are not orderable by strength — they are gates on
    // different questions, and only one of them is about the defect.
    let identical = Swap {
        cells: 4_800,
        changed: 0,
        stale: 0,
    };
    assert!(!the_gate_that_was_there(&identical));
    assert!(the_gate_it_should_have_been(&identical));
}

// ── components ticket 06: the two partition primitives, against the same budget ──────────────────

/// A steady frame written **through the two helpers** rather than through the runtime's verbs.
///
/// The same claim as [`steady_frame`], one layer up: `fit` and `block` are what every other
/// component in this library will route its writing through, so a per-frame allocation inside either
/// of them is a per-frame allocation in every component at once.
fn steady_helpers(cx: &mut Ctx<'_, '_>) {
    let mut rest = block(cx, cx.area(), " panel systems ");
    for _ in 0..12 {
        rest = fit(cx, rest, "a row that does not move");
    }
    let opts = FitOpts {
        justify: Justify::Middle,
        role: Role::Dim,
        pad: Role::Face,
    };
    for _ in 0..4 {
        rest = fit_with(cx, rest, "an unusually long label that will be cut", &opts);
    }
}

/// **`fit` and `block` allocate nothing on a steady frame, as a total.**
///
/// [`vitui_components::ink::Direct`]'s whole justification is that the shipped path stages into the
/// frame's own reusable buffer — `Ctx::stage` and `Ctx::blit` — instead of materialising a run with
/// `str::repeat`, and a justification with no counter behind it is what §21 spent three refinements
/// on. The instrument path *does* allocate, deliberately, and does not run here.
#[test]
fn the_two_partition_helpers_allocate_nothing_as_a_total_over_the_run() {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    driver.frame(steady_helpers);
    driver.frame(steady_helpers);

    let (_, total) = count_allocations(|| {
        for _ in 0..FRAMES {
            driver.frame(steady_helpers);
        }
    });

    let seen = Allocations::over(FRAMES, total as u64);
    assert_eq!(
        seen.total(),
        0,
        "{} allocations over {} frames of `block` and sixteen `fit`s, and the budget is a total of \
         zero",
        seen.total(),
        seen.frames()
    );
    black_box(&driver);
}

/// **A collection over a million rows allocates nothing in a steady frame**, which is components
/// ticket 12's ninth criterion and the one number in it that is a *gate* rather than a report.
///
/// Warmed for the reason the gate above documents, and over the **shipped** entry point —
/// `collect::collection`, which takes `&mut dyn FnMut` closures precisely so that a component-facing
/// signature needs no monomorphisation and no boxing at a call site.
///
/// The row drawer is the one this crate would write: `Ctx::stage` and `Ctx::blit`, which format into
/// the frame's own reusable buffer. A `format!` per row would be eighty allocations a frame and the
/// gate would say so — which is the half worth having, because *zero allocations during frame
/// composition* is a claim about what a component author can write and not only about what the
/// library does underneath.
///
/// `examples/collection_numbers.rs` prints the same number, and this is where it is asserted: an
/// example is compiled by `cargo clippy --all-targets` and evaluated by nothing, which is register
/// entry 12's own finding one crate down.
#[test]
fn a_collection_over_a_million_rows_allocates_nothing_in_a_steady_frame() {
    use vitui_components::collect::{CollOpts, CollState, collection};
    use vitui_components::frame::{Face, face_paint};
    use vitui_components::order::Rows;
    use vitui_runtime::Rect;

    const ROWS: usize = 1_000_000;

    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    let mut st = CollState::new();
    let opts = CollOpts::default();
    let frame = |driver: &mut Driver, st: &mut CollState| {
        driver.frame(|cx| {
            let area = cx.area();
            let _ = collection(
                cx,
                area,
                st,
                &opts,
                Rows::of(ROWS),
                &mut |_: &str, _: std::ops::Range<usize>| None,
                &mut |cx: &mut Ctx<'_, '_>, r: Rect, i: usize, f: Face| {
                    let paint = face_paint(cx.theme(), f);
                    let _ = cx.stage(format_args!("row {i}"));
                    let written = cx.blit(r.x, r.y, paint);
                    cx.fill(
                        Rect::new(
                            r.x + i32::from(written.cells),
                            r.y,
                            r.w.saturating_sub(written.cells),
                            1,
                        ),
                        " ",
                        paint,
                    );
                },
            );
        });
    };
    frame(&mut driver, &mut st);
    frame(&mut driver, &mut st);

    let (_, total) = count_allocations(|| {
        for _ in 0..FRAMES {
            frame(&mut driver, &mut st);
        }
    });

    let seen = Allocations::over(FRAMES, total as u64);
    assert_eq!(
        seen.total(),
        0,
        "{} allocations over {} frames of a collection at {ROWS} rows, and the budget is a total \
         of zero",
        seen.total(),
        seen.frames()
    );
    // And the frame really drew: a gate over a collection that iterated nothing would report zero
    // for the same reason a correct one does.
    assert_eq!(
        driver.inspect().hits().len(),
        1,
        "one hit entry, and it drew"
    );
    black_box(&driver);
}
