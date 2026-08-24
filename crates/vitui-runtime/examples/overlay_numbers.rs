//! **Runtime ticket 13's report**: what an overlay costs, and **which of the two obligations broke**
//! when a modal stops fitting.
//!
//! ```text
//! cargo run --release --example overlay_numbers -p vitui-runtime
//! ```
//!
//! Spec §10 and §19. The claim this report exists for is a *cliff*: a modal built naively is 117% of
//! the frame budget, and the whole difference is the scrim — an operator layer, which the engine
//! prices at 12.7× a content layer, applied to **every damaged cell on the screen**. Two things bring
//! it back under, and both are contracts on the component library rather than on this crate:
//!
//! 1. **The base pass draws into its own layer, with a damage-driven composite.** `Driver` already
//!    works this way; the degraded arm here forces the shortcut the prototype had — a full-screen
//!    fill every frame — so the number has a name rather than being a hypothesis.
//! 2. **Every component draws its text before its padding.** A component that fills its rectangle and
//!    then draws into it re-damages what it drew, every frame, for identical output.
//!
//! **The gate is a cliff and a ratio; the microseconds are a report.** What is asserted is that the
//! *marginal* cost of a standing modal is under a fifth of the frame budget, and that breaking
//! obligation 1 doubles the frame. Marginal, because that is what *a modal costs 1.19 µs* says: the
//! base pass is priced separately, and on this instrument it is priced high — a headless sink still
//! composites and **serialises** every damaged cell, so a dense screen that redraws 7 488 of its
//! 24 000 cells costs ~92 µs a frame before anything is overlaid on it. Every arm below is a delta
//! against that same base.
//!
//! # Obligation 2 has no timing detector against the shipped engine, and that is a finding
//!
//! The map priced padding-before-text at **10 814 of 24 000 cells** and at the difference between
//! 1.19 µs and 87 µs. Against the shipped engine the second half does not follow, and the reason is
//! structural: damage is a **per-row bitset** (`damage.rs`), so a second write inside a range that is
//! already marked adds **no damaged cell at all**. The scrim therefore composites exactly the same
//! set either way, and what a redundant fill costs is the redundant *writes* — real, small, and
//! nowhere near a cliff. The map's arm was measured against a prototype whose damage was not a
//! bitset.
//!
//! So obligation 2's detector is **the count**, which is gated in
//! `ctx::overlay_tests::padding_before_text_re_damages_cells_and_text_before_padding_re_damages_none`,
//! and its timing is printed here without an assertion on it. A gate with no detector behind it is
//! worse than no gate.

use std::hint::black_box;

use vitui_bench::Bench;
use vitui_engine::{ColorDepth, Rect};
use vitui_runtime::ctx::{Ctx, Driver, Interest};
use vitui_runtime::id::Id;
use vitui_runtime::overlay::{Align, OverlayOpts, Placement, Side, Z, place};
use vitui_runtime::theme::{Role, Theme};

// The one realistic screen, shared with every other report. See `src/screen.rs`.
#[allow(dead_code)]
#[path = "../src/screen.rs"]
mod screen;
use screen::{REGIONS, dense_draw, screen_frame};

/// Spec §19's typical-frame budget, in nanoseconds. **A floor, not a target.**
const FRAME_NS: f64 = 100_000.0;

/// The dense screen, in cells.
const CELLS: f64 = 24_000.0;

/// How the base pass is drawn: obligation 2 and obligation 1, as two booleans.
#[derive(Clone, Copy)]
struct Arm {
    /// `true` draws text before padding — obligation 2 met.
    text_first: bool,
    /// `true` fills the whole screen before drawing anything, which is what a base pass with no
    /// damage-driven composite behind it amounts to — the shortcut obligation 1 forbids.
    full_repaint: bool,
}

const BOTH: Arm = Arm {
    text_first: true,
    full_repaint: false,
};

fn base_pass(cx: &mut Ctx<'_, '_>, arm: Arm) {
    if arm.full_repaint {
        let face = cx.theme().paint(Role::Face);
        cx.fill(cx.area(), " ", face);
    }
    let _ = dense_draw(cx, arm.text_first);
    for region in 0..REGIONS {
        let i = i32::try_from(region).unwrap_or(0);
        cx.interact(
            Id::from_raw(u64::try_from(region).unwrap_or(0)),
            Rect::new((i % 4) * 75, i / 4, 75, 1),
            Interest::CLICK.with(Interest::FOCUS),
        );
    }
}

fn driver() -> Driver {
    let mut d = Driver::headless(300, 80).expect("attaching to a sink cannot fail");
    d.set_theme(Theme::default().resolve(ColorDepth::TrueColor));
    d
}

/// One steady frame with a modal standing, in the given configuration.
///
/// **Eight frames of warm-up before the measurement, and the reason is structural.** The frame that
/// *opens* a modal damages the whole screen once by construction — a new content layer damages its
/// own rectangle, and the scrim's operator exposes everything it covers. What is being priced is the
/// frame after that, which is every frame a dialog is on screen.
fn modal_ns(name: &'static str, arm: Arm) -> f64 {
    let theme = Theme::default().resolve(ColorDepth::TrueColor);
    let dialog = Id::named("dialog");
    let mut d = driver();
    let one = move |d: &mut Driver| {
        d.frame(|cx| {
            base_pass(cx, arm);
            cx.overlay(
                dialog,
                Rect::new(120, 30, 60, 1),
                OverlayOpts::modal(60, 14, &theme),
                move |cx| {
                    cx.modal_barrier_here();
                    let body = cx.theme().paint(Role::Body);
                    for row in 0..14i32 {
                        cx.text(
                            1,
                            row,
                            "the dialog's own content, redrawn every frame",
                            body,
                        );
                    }
                    cx.interact(Id::named("ok"), Rect::new(1, 12, 8, 1), Interest::CLICK);
                },
            );
        });
    };
    for _ in 0..8 {
        one(&mut d);
    }
    let report = Bench::new(40)
        .case(name, 20, || one(black_box(&mut d)))
        .run();
    report.get(name).expect("measured")
}

/// The same screen with no overlay at all.
fn plain_ns() -> f64 {
    let mut d = driver();
    let one = |d: &mut Driver| {
        d.frame(|cx| base_pass(cx, BOTH));
    };
    for _ in 0..8 {
        one(&mut d);
    }
    let report = Bench::new(40)
        .case("no overlay", 20, || one(black_box(&mut d)))
        .run();
    report.get("no overlay").expect("measured")
}

/// The same screen with a dropdown standing: a small content layer and **no scrim**, which is the
/// whole difference between +1.46 µs and 117% of the budget.
fn dropdown_ns() -> f64 {
    let owner = Id::named("dropdown");
    let mut d = driver();
    let one = move |d: &mut Driver| {
        d.frame(|cx| {
            base_pass(cx, BOTH);
            cx.overlay(
                owner,
                Rect::new(10, 3, 12, 1),
                OverlayOpts {
                    z: Z::MENU,
                    ..OverlayOpts::sized(12, 6)
                },
                |cx| {
                    let body = cx.theme().paint(Role::Body);
                    for row in 0..6i32 {
                        cx.text(0, row, "an item", body);
                    }
                },
            );
        });
    };
    for _ in 0..8 {
        one(&mut d);
    }
    let report = Bench::new(40)
        .case("a dropdown", 20, || one(black_box(&mut d)))
        .run();
    report.get("a dropdown").expect("measured")
}

/// Placement, over the exhaustive corpus the gate uses.
fn placement_ns() -> (f64, u32) {
    let bounds = Rect::new(0, 0, 20, 12);
    let sides = [Side::Below, Side::Above, Side::Right, Side::Left];
    let aligns = [Align::Start, Align::Center, Align::End];
    let mut cases = 0u32;
    let sweep = |cases: &mut u32| {
        let mut acc = 0i32;
        for side in sides {
            for align in aligns {
                for ax in 0..10i32 {
                    for ay in 0..10i32 {
                        for w in [1u16, 3, 5, 7, 9, 11] {
                            for h in [1u16, 2, 3, 4] {
                                let r = place(
                                    Rect::new(ax, ay, 3, 1),
                                    (w, h),
                                    bounds,
                                    Placement::new(side, align),
                                );
                                acc += r.x + r.y;
                                *cases += 1;
                            }
                        }
                    }
                }
            }
        }
        black_box(acc);
    };
    sweep(&mut cases);
    let corpus = f64::from(cases);
    let report = Bench::new(20)
        .case("28 800 placements", 1, || {
            let mut ignored = 0u32;
            sweep(&mut ignored);
        })
        .run();
    (
        report.get("28 800 placements").expect("measured") / corpus,
        cases,
    )
}

/// The body queue, over a hundred frames with a dropdown standing.
fn bodies() -> (u32, usize, u64) {
    static ITEMS: [&str; 3] = ["Open", "Save", "Close"];
    let owner = Id::named("dropdown");
    let items: &[&str] = &ITEMS;
    let selected = 1usize;
    let mut d = driver();
    for _ in 0..100 {
        d.frame(|cx| {
            cx.overlay(
                owner,
                Rect::new(4, 4, 10, 1),
                OverlayOpts::sized(10, 3),
                move |cx| {
                    let body = cx.theme().paint(Role::Body);
                    for (row, item) in items.iter().enumerate() {
                        cx.text(0, i32::try_from(row).unwrap_or(0), item, body);
                    }
                    let _ = (selected, owner);
                },
            );
        });
    }
    (
        d.inspect().overlay_bodies_boxed(),
        d.layers_live(),
        d.surface_reallocs(),
    )
}

/// The double-write count, both ways round.
fn double_writes() -> (u32, u32) {
    let mut d = driver();
    let mut pad_first = 0u32;
    let mut text_first = 0u32;
    d.frame(|cx| pad_first = dense_draw(cx, false));
    d.frame(|cx| text_first = dense_draw(cx, true));
    (pad_first, text_first)
}

fn main() {
    let (splits, lanes) = screen_frame(300, 80);
    assert_eq!(
        (splits, lanes),
        (32, 119),
        "the dense screen drifted; every number below is against the wrong shape"
    );

    let plain = plain_ns();
    let dropdown = dropdown_ns();
    let with_both = modal_ns("both obligations met", BOTH);
    let no_text_first = modal_ns(
        "padding before text",
        Arm {
            text_first: false,
            full_repaint: false,
        },
    );
    let no_own_layer = modal_ns(
        "base repainted whole",
        Arm {
            text_first: true,
            full_repaint: true,
        },
    );
    let (per_case, cases) = placement_ns();
    let (boxed, layers, reallocs) = bodies();
    let (pad_first, text_first) = double_writes();

    println!(
        "machine Apple M1 Max, macOS 26.5.2, --release, minimum of 40 rounds, round robin.\n\
        \x20       the dense screen: 300x80 = 24 000 cells, {REGIONS} interactive regions,\n\
        \x20       {splits} splits and {lanes} lanes.\n"
    );

    let share = |ns: f64| ns - plain;
    println!(
        "report  what an overlay adds to a dense frame:\n\
        \x20       base pass, no overlay        {plain:>10.2} ns   {:>6.1}% of the budget\n\
        \x20       + a dropdown standing        {dropdown:>10.2} ns   {:+.2} us, {:+.2}%\n\
        \x20                                                  spec §10 measured **+1.46 us, 1.5%**\n\
        \x20       + a modal, both obligations  {with_both:>10.2} ns   {:+.2} us, {:+.2}%\n\
        \x20                                                  spec §10 measured **1.19 us**\n\
        \x20       The base pass is 92% of the budget on its own, because a headless sink still\n\
        \x20       serialises every damaged cell: 7 488 of 24 000 are redrawn every frame. Every\n\
        \x20       arm here is therefore a **delta** against that same base, which is what *a\n\
        \x20       modal costs 1.19 us* says. Marginal headroom: {:.2} us of {:.0} us.\n",
        100.0 * plain / FRAME_NS,
        share(dropdown) / 1000.0,
        100.0 * share(dropdown) / FRAME_NS,
        share(with_both) / 1000.0,
        100.0 * share(with_both) / FRAME_NS,
        (FRAME_NS / 5.0 - share(with_both)) / 1000.0,
        FRAME_NS / 5.0 / 1000.0
    );

    println!(
        "report  which obligation broke, which is why both degraded arms are measured:\n\
        \x20       both obligations met         {with_both:>10.2} ns   {:>6.1}% of the budget\n\
        \x20       padding drawn before text    {no_text_first:>10.2} ns   {:>6.1}%   {:.2}x   NOT GATED\n\
        \x20       base pass repainted whole    {no_own_layer:>10.2} ns   {:>6.1}%   {:.2}x   gated at 1.5x\n\
        \x20       spec §10 measured **1.19 us with both, 87 us and 117 us with either missing**.\n\
        \x20       Obligation 1 reproduces as a ratio. **Obligation 2 does not, and the reason is\n\
        \x20       structural**: damage is a per-row bitset, so a second write inside a marked range\n\
        \x20       adds no damaged cell and the scrim composites the same set either way. Its\n\
        \x20       detector is the count below, not a stopwatch.\n",
        100.0 * with_both / FRAME_NS,
        100.0 * no_text_first / FRAME_NS,
        no_text_first / with_both,
        100.0 * no_own_layer / FRAME_NS,
        no_own_layer / with_both
    );

    println!(
        "report  the draw-text-before-padding obligation, as a count:\n\
        \x20       padding then text            {pad_first:>10} cells written twice   {:.1}% of 24 000\n\
        \x20       text then padding            {text_first:>10} cells written twice\n\
        \x20       spec §10 measured **10 814 of 24 000 against 0** on its own dense screen; the\n\
        \x20       count above is this fixture's and is asserted as one. **The pair is what belongs\n\
        \x20       to the mechanism**: the picture is identical either way, so the second write buys\n\
        \x20       nothing at all. What it costs is the writes — not extra damage, because a per-row\n\
        \x20       bitset is idempotent — which is why this is the honest form of the obligation.\n\
        \x20       {:.1}% of the frame's cells written for no reason is still worth not doing.\n",
        100.0 * f64::from(pad_first) / CELLS,
        100.0 * f64::from(pad_first) / CELLS
    );

    println!(
        "report  placement, exhaustively:\n\
        \x20       corpus                       {cases:>10} cases   4 sides x 3 aligns x 100 anchors x 24 sizes\n\
        \x20       per case                     {per_case:>10.2} ns      spec §10 measured **0.53 ns**\n\
        \x20       place -> flip -> shift -> clamp, integer arithmetic, no state between calls.\n\
        \x20       The figure includes the six nested loops that generate the corpus, so it is an\n\
        \x20       upper bound on `place` rather than a measurement of it.\n"
    );

    println!(
        "report  the overlay body queue, over 100 frames with a dropdown standing:\n\
        \x20       bodies boxed a frame         {boxed:>10}         spec §10 states **one a request**\n\
        \x20       allocations a frame          {:>10}         n + 1: a `Box` a body, plus the queue\n\
        \x20       layers live                  {layers:>10}\n\
        \x20       surfaces reallocated         {reallocs:>10}         a move is 0, a resize is 1\n\
        \x20       The count is the gate rather than a report and lives in\n\
        \x20       `tests/alloc.rs::one_standing_overlay_costs_one_allocation_a_frame`, where the\n\
        \x20       marginal figure — one allocation a standing overlay — is what is asserted. It was\n\
        \x20       zero until ticket 21 traded the arena for `#![forbid(unsafe_code)]`; spec §19\n\
        \x20       carries the exception.\n",
        boxed + 1
    );

    // ── the gates: one cliff and one ratio ───────────────────────────────────────────────────────
    //
    // **The marginal cost of a standing modal**, against a fifth of the frame budget. A cliff rather
    // than a measurement: the map's 1.19 µs is 1.2% of the budget and this instrument's arm is a few
    // per cent of it, so twenty is far enough away from both to be a regression detector rather than
    // a transcription of today's number.
    let marginal = with_both - plain;
    assert!(
        marginal < FRAME_NS / 5.0,
        "a standing modal added {:.2} us to the frame, against a fifth of the {:.0} us budget",
        marginal / 1000.0,
        FRAME_NS / 1000.0
    );
    // **Obligation 1's detector**, as a ratio. Repainting the base pass whole under a scrim roughly
    // doubles the frame, because the operator layer then has 24 000 damaged cells to work over
    // instead of 7 488. 1.5x is the cliff; the measured value is printed above.
    assert!(
        no_own_layer / with_both > 1.5,
        "a whole-screen repaint under a scrim cost only {:.2}x, so obligation 1 has no detector \
         left: {no_own_layer:.0} against {with_both:.0}",
        no_own_layer / with_both
    );
    // Obligation 2 is deliberately **not** asserted on a timing. See the module comment.
    let _ = no_text_first;
    assert_eq!(text_first, 0, "text before padding writes nothing twice");
    assert!(pad_first > 0, "and padding before text writes plenty twice");
    assert_eq!(cases, 28_800);
    assert_eq!(
        boxed, 1,
        "one body a frame, and a hundred frames did not move it"
    );
    assert_eq!(layers, 1);
    assert_eq!(reallocs, 0);
    println!("gates   all passed.");
}
