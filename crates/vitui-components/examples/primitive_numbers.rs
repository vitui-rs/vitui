//! components ticket 10 — `text`, `chip`, `button`, `panel`, and clearing once, as numbers.
//!
//! A **report**, not a gate: R15's refinement 2 says a row may cite one and may never rest on one,
//! and `cargo test` does not run a `fn main`. What gates every number below is `src/text.rs`'s test
//! module, `src/input.rs`'s, `src/structure.rs`'s, `src/app.rs`'s and `src/dense.rs`'s, and
//! `crate::dense`'s ledger block is where each figure has its single home.
//!
//! It prints six things:
//!
//! 1. **The four primitives**, each writing a partition of its own rectangle.
//! 2. **The dense screen** in §20's counter columns, correct against the naive twin.
//! 3. **The five re-damage instances**, each watched firing on this screen.
//! 4. **Clearing once**, against clearing every frame and against not clearing at all.
//! 5. **The frame one chip is hovered**: 8 cells, and what the ratio is on this screen.
//! 6. **What does not reproduce**, said out loud rather than engineered away.

use vitui_components::app::Clears;
use vitui_components::counters::{Allocations, Tally};
use vitui_components::dense::{self, Arm};
use vitui_components::input::{ButtonOpts, button_into};
use vitui_components::runner::{Canvas, Pen, driver_at, metric_heading};
use vitui_components::structure::{PanelOpts, panel_into};
use vitui_components::text::{ChipOpts, TextOpts, chip_drawn, chip_into, text_into};
use vitui_runtime::Density;
use vitui_runtime::Rect;
use vitui_runtime::ctx::Ctx;

/// The rectangle every primitive is measured on: wide enough that nothing truncates, one row.
const CELL: (u16, u16) = (24, 1);

/// One frame of `f` on a `CELL`-sized sink, tallied.
fn tallied(f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>)) -> Tally {
    let mut driver = driver_at(CELL.0, CELL.1 + 6, Density::Compact);
    let mut tally = Tally::new();
    driver.frame(|cx| f(&mut tally, cx));
    tally
}

fn main() {
    println!("components ticket 10 — the four primitives\n");

    // ── 1. the four primitives ───────────────────────────────────────────────────────────────────
    println!(
        "report  each primitive on a {}x{} rectangle, `Compact`",
        CELL.0, CELL.1
    );
    println!(
        "  {:<9}  {:>7}  {:>9}  {:>6}  {:>9}  {:>9}",
        "component", "writes", "distinct", "verbs", "own cells", "returned"
    );
    let area = Rect::new(0, 0, CELL.0, CELL.1);
    let rows: [(&str, Tally, u64); 4] = [
        (
            "text",
            tallied(|tally, cx| {
                text_into(tally, cx, area, "throughput, req/s", &TextOpts::default());
            }),
            0,
        ),
        (
            "chip",
            tallied(|tally, cx| {
                chip_into(tally, cx, area, "degraded", &ChipOpts::default());
            }),
            0,
        ),
        (
            "button",
            tallied(|tally, cx| {
                button_into(tally, cx, area, "reset", &ButtonOpts::default());
            }),
            0,
        ),
        {
            let mut handed = 0u64;
            let tally = tallied(|tally, cx| {
                let panel = panel_into(
                    tally,
                    cx,
                    Rect::new(0, 0, CELL.0, CELL.1 + 6),
                    " panel systems ",
                    &PanelOpts::default(),
                );
                handed = u64::from(panel.interior.w) * u64::from(panel.interior.h);
            });
            ("panel", tally, handed)
        },
    ];
    for (name, tally, returned) in &rows {
        let owned = if *returned == 0 {
            u64::from(area.w) * u64::from(area.h)
        } else {
            u64::from(CELL.0) * u64::from(CELL.1 + 6) - returned
        };
        println!(
            "  {name:<9}  {:>7}  {:>9}  {:>6}  {owned:>9}  {returned:>9}",
            tally.writes(),
            tally.distinct(),
            tally.verbs(),
        );
    }
    println!(
        "  every row: writes == distinct == the cells it owns. `panel` is the one that returns\n  \
         something, because §2's *the cells it does not write are named in its return value* is a\n  \
         statement only a container has to make.\n"
    );

    // ── 2. the dense screen ──────────────────────────────────────────────────────────────────────
    println!(
        "report  the dense screen, {}x{}, `Compact`",
        dense::W,
        dense::H
    );
    println!("{}", metric_heading());
    for (name, arm) in [("correct", Arm::Correct), ("naive", Arm::Naive)] {
        println!("{}", dense::row(arm, name, Allocations::over(1, 0)).line());
    }
    let shape = dense::shape(Arm::Correct, (dense::W, dense::H), dense::REQUESTED);
    println!(
        "  {} regions declared, {} in the runtime's hit index, {} tab stops, {} cells handed over\n",
        shape.declared, shape.regions, shape.stops, shape.handed_over
    );

    // ── 3. the five instances ────────────────────────────────────────────────────────────────────
    println!("report  ADR 0026's five re-damage instances, per steady frame:");
    println!("  {:<34}  {:>8}  {:>8}", "instance", "cells", "correct");
    for (what, arm, expected) in [
        (
            "`cx.clear(body)` every frame",
            Arm::ClearsEveryFrame,
            dense::CLEARED_EVERY_FRAME,
        ),
        (
            "a chip filling its face",
            Arm::ChipFillsItsFace,
            dense::CHIP_FILLED_FACE,
        ),
        (
            "a chip that does not narrow",
            Arm::ChipDoesNotNarrow,
            dense::CHIP_NOT_NARROWED,
        ),
        (
            "a border run over its title, x3 panels",
            Arm::BorderOverTheTitle,
            dense::BORDER_OVER_TITLE_SCREEN,
        ),
    ] {
        let seen = dense::steady(arm, 3);
        assert_eq!(seen.per_frame, expected);
        println!("  {what:<34}  {:>8}  {:>8}", seen.per_frame, 0);
    }
    let scrim = dense::modal_steady(true, 3);
    assert_eq!(scrim.per_frame, dense::SCRIM_UNDER);
    println!(
        "  {:<34}  {:>8}  {:>8}",
        "a scrim filled under the dialog", scrim.per_frame, 0
    );
    println!(
        "  the naive twin is {} and not their sum: re-damage counts distinct cells, and a screen\n  \
         clear already changes every cell the other two change.\n",
        dense::NAIVE_TWIN
    );

    // ── 4. clearing once ─────────────────────────────────────────────────────────────────────────
    println!("report  the clear, three spellings:");
    println!(
        "  {:<22}  {:>8}  {:>10}  {:>28}",
        "spelling", "clears", "re-damage", "what it costs"
    );
    let once = dense::steady_clearing(4);
    println!(
        "  {:<22}  {:>8}  {:>10}  {:>28}",
        "once, and on resize", 1, once.per_frame, "nothing"
    );
    println!(
        "  {:<22}  {:>8}  {:>10}  {:>28}",
        "every frame",
        4,
        dense::CLEARED_EVERY_FRAME,
        "the largest of the five"
    );
    println!(
        "  {:<22}  {:>8}  {:>10}  {:>28}",
        "never", 0, 0, "the gaps, and no counter sees it"
    );
    // The resize, which needs two drivers: `runner::play` refuses a step that moves the rectangle.
    let mut clears = Clears::new();
    let mut counted = 0u32;
    for (w, h) in [(80u16, 24u16), (80, 24), (80, 24), (120, 40), (120, 40)] {
        let mut driver = driver_at(w, h, Density::Compact);
        driver.frame(|cx| {
            counted += u32::from(clears.frame_into(&mut Tally::new(), cx));
        });
    }
    println!(
        "  five frames over two sizes: {counted} clears. The `never` row is why the equality \
         against\n  the naive twin is the gate and the damage count is not.\n"
    );

    // ── 5. the hovered frame ─────────────────────────────────────────────────────────────────────
    let chip = vitui_components::state::chip();
    let mut driver = driver_at(
        vitui_components::state::W,
        vitui_components::state::H,
        Density::Compact,
    );
    let mut canvas = Canvas::new(vitui_components::state::W, vitui_components::state::H);
    let mut hovered_frame = 0u64;
    for hovered in [false, false, true] {
        let mut pen = Pen::over(canvas);
        driver.frame(|cx| {
            let opts = ChipOpts::default();
            let id = cx.id();
            let mut resp = cx.interact(id, chip, opts.interest);
            resp.hovered = hovered;
            chip_drawn(
                &mut pen,
                cx,
                chip,
                vitui_components::state::LABEL,
                &resp,
                &opts,
            );
        });
        pen.end_frame();
        canvas = pen.into_canvas();
        hovered_frame = canvas.take_repaints();
    }
    println!("report  the frame one chip is hovered:");
    println!("  correct: {hovered_frame} cells — the chip's own width, and nothing else");
    println!(
        "  clearing every frame: {} cells on this screen, which is {}x",
        dense::CLEARED_EVERY_FRAME,
        dense::CLEARED_EVERY_FRAME / hovered_frame
    );
    println!("  §2 states 6 662 against 8, which is 833x on C01's screen.\n");

    // ── 6. what does not reproduce ───────────────────────────────────────────────────────────────
    println!("report  what does not reproduce, and why:");
    for line in [
        "  §1 / §2                      here          why",
        "  `fn(&mut Ctx, Rect, …)`      `Rect` for   `Rect` is `vitui_engine::Rect` and C6 says",
        "                               `Rect`        the dependency table is the runtime alone. The",
        "                                             rule is obeyed with this crate's own rectangle",
        "                                             in its place — see `vitui_runtime::layout::rect`.",
        "  rule 4, `-> Response`        `-> Panel`    only for `panel`. §2's *the cells it does not*",
        "                               for `panel`   *write are named in its return value* is not",
        "                                             writable in a bare `Response`, and the closure",
        "                                             form that would make it writable is refused on",
        "                                             two measurements in `crate::frame`.",
        "  84.96 us / 20 784 writes     see the row   the microseconds are a report and gated by",
        "                               above         nothing (R15). The write count is 24 000 here,",
        "                                             deliberately: this screen is a full partition",
        "                                             of its rectangle and C01's was not. 20 784 is",
        "                                             a screen with an unwritten tail.",
        "  6 662 damaged                9 024         the quantity is *cells whose steady value is",
        "                                             not a space painted `Body`* — how much of a",
        "                                             screen is ink. 37.6% here against C01's 27.8%.",
        "  2 648 for a filled face      1 095         and it was 1 149 one ticket ago: a chip's label",
        "                                             wears its own face, so the space inside",
        "                                             `degraded, r…` stopped differing from the fill.",
        "                                             18 rows x 1 space x 3 panels = 54.",
        "  432 for a chip that does     324           54 of 222 chips overrun 6 columns each. The",
        "  not narrow                                 *identity* reproduces exactly.",
        "  229 for a scrim under        600           stronger, not looser: nothing in this dialog is",
        "                                             painted in the scrim's role, so the difference",
        "                                             is total. §2's 229 says 371 of 600 already",
        "                                             carried what the scrim wrote.",
        "  15 for a border over its     15            reproduces exactly per panel; the screen has",
        "  own title                     (45 / 3)     three of them. The number *is* the title's width.",
        "  8 on the hovered frame       8             reproduces exactly. The number *is* the chip's",
        "                                             width. The ratio does not: 1 128x here, because",
        "                                             the other column is this screen's 9 024.",
    ] {
        println!("{line}");
    }

    // The **shape**, so that a report which has quietly started measuring something smaller fails
    // rather than looking good. R15's rule, and the reason a report is allowed to carry asserts.
    assert_eq!(hovered_frame, 8);
    assert_eq!(counted, 2, "one clear a size");
    assert_eq!(once.per_frame, 0);
    assert_eq!(shape.regions, dense::REGIONS);
    for (name, tally, returned) in &rows {
        assert_eq!(
            tally.writes(),
            tally.distinct(),
            "{name} wrote a cell twice"
        );
        assert_eq!(*returned == 0, *name != "panel");
    }
}
