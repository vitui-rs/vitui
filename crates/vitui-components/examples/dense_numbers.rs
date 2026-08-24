//! components ticket 09 — the dense screen, the same screen drawn two ways, and the narrow axis.
//!
//! A **report**, not a gate: R15's refinement 2 says a row may cite one and may never rest on one,
//! and `cargo test` does not run a `fn main`. What gates every number below is `src/dense.rs`'s test
//! module, and each figure has its single home in that file's ledger block.
//!
//! It prints six things:
//!
//! 1. **The metric row**, in §21's own column order, for every arm of the screen.
//! 2. **ADR 0026's five re-damage instances**, each standing on the screen, against a correct arm's
//!    zero.
//! 3. **The equality**, at 300×80 and at 120×40, with what each arm covered beside it.
//! 4. **The narrow axis** — the one defect that is green at the wide size and red at the narrow one.
//! 5. **What `marked` would have said**, and why this crate cannot ask it.
//! 6. **What does not reproduce**, said out loud rather than engineered away.

use vitui_alloc_probe::{CountingAllocator, count_allocations};
use vitui_components::counters::{Allocations, Counter, Reading};
use vitui_components::dense::{self, Arm};
use vitui_components::runner::{Painter, compare_at};
use vitui_components::scenes;
use vitui_runtime::Density;

// **The probe, because `allocations` is a total and a total needs something that counts.** Installed
// here rather than defaulted to zero inside the library: a figure defaulted to zero is a counter
// that prints `0` when it means *nobody counted*.
#[global_allocator]
static PROBE: CountingAllocator = CountingAllocator;

/// Every arm, with the name the report prints it under.
const ARMS: [(&str, Arm); 9] = [
    ("correct", Arm::Correct),
    ("naive twin", Arm::Naive),
    ("clear/frame", Arm::ClearsEveryFrame),
    ("chip fill", Arm::ChipFillsItsFace),
    ("chip wide", Arm::ChipDoesNotNarrow),
    ("label wide", Arm::LabelDoesNotNarrow),
    ("border/title", Arm::BorderOverTheTitle),
    ("scrim around", Arm::ScrimAroundTheDialog),
    ("scrim under", Arm::ScrimUnderTheDialog),
];

fn main() {
    println!("components ticket 09 — one screen, 338 regions, and five instances of one rule\n");

    // ── 1. the metric row ────────────────────────────────────────────────────────────────────────
    println!(
        "report  the dense screen at {}x{}, `Compact`, in §21's column order:",
        dense::W,
        dense::H
    );
    println!("{}", vitui_components::runner::metric_heading());
    for (name, arm) in ARMS {
        let (_, allocated) = count_allocations(|| dense::row(arm, "", Allocations::over(1, 0)));
        // The allocation total is the whole play's — `driver_at` attaches its own driver inside it —
        // and its microsecond figure is a **first** frame in whatever profile this was built in.
        // Both are reports. The steady-frame zero is `tests/gates.rs`'s, which warms the workload.
        let row = dense::row(arm, "", Allocations::over(1, allocated as u64));
        println!("{:<52}  {}", name, row.columns());
    }
    println!();
    println!("  `us` is a **report** and gated by nothing: three of the four hostile axes on this");
    println!("  map were *faster*, so a microsecond figure beside a defect is evidence about the");
    println!(
        "  defect's disguise and never about its presence. Every count beside it is exact and"
    );
    println!("  has one home in `crate::dense`'s ledger.");
    println!(
        "  `allocations` is the **whole play's** — `driver_at` attaches its own driver inside"
    );
    println!("  the measured region and `Pen` records a `String` a cell, which is an instrument");
    println!("  allocating and not a component. The steady-frame budget is `tests/gates.rs`'s,");
    println!("  which warms the identical workload before it opens the window.\n");

    // ── 2. the five instances ────────────────────────────────────────────────────────────────────
    println!("report  ADR 0026's five re-damage instances, each standing on this screen:");
    println!("  {:<46}  {:>9}  {:>9}", "where", "§2 says", "here");
    let mut measured = Vec::new();
    for (label, arm, remembered) in [
        (
            "the application's own `cx.clear(body)` a frame",
            Arm::ClearsEveryFrame,
            6_662u64,
        ),
        (
            "`chip` filling its face before its label",
            Arm::ChipFillsItsFace,
            2_648,
        ),
        ("a chip that does not narrow", Arm::ChipDoesNotNarrow, 432),
    ] {
        let seen = dense::steady(arm, 4);
        println!("  {label:<46}  {remembered:>9}  {:>9}", seen.per_frame);
        measured.push(seen.per_frame);
    }
    let under = dense::modal_steady(true, 4);
    let around = dense::modal_steady(false, 4);
    println!(
        "  {:<46}  {:>9}  {:>9}",
        "a modal's scrim filled *under* the dialog", 229, under.per_frame
    );
    let border = dense::steady(Arm::BorderOverTheTitle, 4);
    println!(
        "  {:<46}  {:>9}  {:>9}",
        "a `panel` border run written over its title",
        15,
        border.per_frame / u64::from(dense::PANELS)
    );
    println!(
        "  {:<46}  {:>9}  {:>9}",
        "  ... the same, over the screen's three panels", "-", border.per_frame
    );
    println!();
    println!(
        "  and the correct arm: {} cells, on every steady frame, for ever.",
        dense::steady(Arm::Correct, 4).per_frame
    );
    println!(
        "  A scrim drawn as the four rectangles *around* the dialog: {}.",
        around.per_frame
    );
    println!();
    println!("  **The five do not add up, and that is the point of counting distinct cells.** The");
    println!("  naive twin is three of them at once and costs the largest, not the sum: a screen");
    println!("  clear already changes every cell a face fill changes. A report that totalled the");
    println!("  table would be double counting.\n");

    // ── 3. the equality ──────────────────────────────────────────────────────────────────────────
    println!("report  the same screen drawn naive and correct, at two sizes:");
    println!(
        "  {:<12}  {:<34}  {:>10}  {:>10}",
        "size", "the runner reports", "correct", "naive"
    );
    for (size, cells) in [
        ((dense::W, dense::H), dense::SCREEN),
        (dense::NARROW, dense::NARROW_SCREEN),
    ] {
        let diff = dense::equality(size);
        let correct = dense::play(
            Arm::Correct,
            &[dense::widgets(dense::REQUESTED).resized(size.0, size.1)],
        );
        let naive = dense::play(
            Arm::Naive,
            &[dense::widgets(dense::REQUESTED).resized(size.0, size.1)],
        );
        println!(
            "  {:<12}  {:<34}  {:>10}  {:>10}",
            format!("{}x{}", size.0, size.1),
            diff.to_string(),
            correct.canvas().written(),
            naive.canvas().written()
        );
        assert!(diff.clean());
        assert_eq!(correct.canvas().written() as u64, cells);
    }
    println!();
    println!("  Neither arm is clean by drawing nothing: both cover every cell of the screen, and");
    println!("  the correct one writes each of them exactly once. A build that merely drew less");
    println!("  would score 0 re-damaged cells by drawing nothing, and three separate defects on");
    println!("  this map did exactly that — which is the whole reason the twin is kept.\n");

    // ── 4. the narrow axis ───────────────────────────────────────────────────────────────────────
    println!("report  the narrow axis: green at the wide size, red at the narrow one.");
    for (size, label) in [((dense::W, dense::H), "300x80"), (dense::NARROW, "120x40")] {
        let diff = compare_at(
            Density::Compact,
            dense::correct as Painter,
            dense::label_does_not_narrow as Painter,
            &[dense::widgets(dense::REQUESTED).resized(size.0, size.1)],
        );
        println!("  a label that does not narrow, {label:<8}  {diff}");
    }
    let wide = dense::shape(Arm::Correct, (dense::W, dense::H), dense::REQUESTED);
    let narrow = dense::shape(Arm::Correct, dense::NARROW, dense::REQUESTED);
    println!("\n  {:<26}  {:>8}  {:>8}", "", "300x80", "120x40");
    for (name, a, b) in [
        ("interactive regions", wide.regions, narrow.regions),
        ("tab stops", wide.stops, narrow.stops),
        ("widgets standing", wide.visible, narrow.visible),
        ("widgets dropped", wide.dropped, narrow.dropped),
    ] {
        println!("  {name:<26}  {a:>8}  {b:>8}");
    }
    println!();
    println!(
        "  At 300 columns a label column is 74 cells and all eight labels fit, so the arm that"
    );
    println!(
        "  does not truncate is **identical to the correct one**. At 120 it is 14 cells, five"
    );
    println!(
        "  of the eight overrun, and the same arm writes into the chip beside it and into the"
    );
    println!("  next panel's border. One size cannot see this; that is what two are for.\n");

    // ── 5. what `marked` would have said ─────────────────────────────────────────────────────────
    println!("report  the counter this ticket wanted, and the one it got:");
    for line in [
        "  `marked`      unreachable. `crates/vitui-engine/src/damage.rs` is `pub(crate)` from top",
        "                to bottom and `Presented` carries no count of cells, so no crate above the",
        "                engine can read the engine's damage. `Counters::marked` panics rather than",
        "                answering 0 — a threshold on the wrong side of the question is not a weak",
        "                gate, it is a green one.",
        "  `repaints`    what ships instead, and it is the **input** to the number `marked` is the",
        "                output of. The engine drops a write whose value equals the resident value,",
        "                so what becomes damage is a write whose value differs from what is already",
        "                there — computable at the verb boundary over a surface this crate keeps.",
        "                Carried across frames by `Pen::over`: a fresh `Pen` a frame would report",
        "                every cell as a first paint for ever, which is re-damage with the relation",
        "                removed.",
        "  the modal     is drawn on its own layer over a base pass that has not changed, and this",
        "                instrument has one pass. A `Ctx::child` moves the origin, so an overlay",
        "                body's writes would land in a coordinate system the base pass's `Pen` also",
        "                uses — `crate::frame` measures that collision at 124 false double writes.",
        "                So the first frame draws the screen and the modal and every frame after it",
        "                draws the modal alone. **Both scrim arms go through the substitution.**",
    ] {
        println!("{line}");
    }
    println!();

    // ── 6. what does not reproduce ───────────────────────────────────────────────────────────────
    println!("report  what does not reproduce, and why:");
    for line in [
        "  spec §2 / ADR 0026         here     why",
        "  338 regions at 300x80      338      reproduces, and it is arithmetic rather than a",
        "                                      target: 2 + 3 x (1 + 74 + 37), where 74 is what a",
        "                                      `Compact` ring leaves of a 78-row band.",
        "  0 of 24 000 cells apart,   0 / 0    reproduces at both sizes. 24 000 and 4 800 cells,",
        "  at 300x80 and 120x40                every one of them written exactly once by the",
        "                                      correct arm.",
        "  15 for a border run over   15       reproduces exactly, and by construction: the number",
        "  its own title                       *is* the title's width. Ticket 06 measured the same",
        "                                      15 as `writes - distinct` on one panel; this is the",
        "                                      same defect as re-damage on a screen, and the screen",
        "                                      carries three of them (45).",
        "  6 662 for `cx.clear` every 9 024    does not. The quantity is *cells whose steady value",
        "  frame                               is not a space painted `Body`* — how much of a screen",
        "                                      is ink. 37.6% here against C01's 27.8%. The relation",
        "                                      to the correct arm's 0 reproduces, and so does its",
        "                                      being the largest of the five.",
        "  2 648 for a chip filling   1 095    does not, and it is arithmetic: 3 x (18 x (2+3+4+11)",
        "  its face                            + 2 + 3) over the four values a row cycles through.",
        "                                      Both factors — how many chips, how wide their labels",
        "                                      — are this screen's rather than the rule's. It was",
        "                                      1 149 until components 10: a chip's label wears its",
        "                                      own face, so the one space inside `degraded, r…` now",
        "                                      carries what the fill wrote and the engine drops it.",
        "                                      18 rows x 1 space x 3 panels = 54.",
        "  432 for a chip that does   324      does not. 54 of 222 chips overrun 6 columns each, and",
        "  not narrow                          `54 x 6` is the whole number. The *identity*",
        "                                      reproduces: the cells re-damaged are exactly the ones",
        "                                      the chip wrote into its neighbour's rectangle.",
        "  229 for a scrim under the  600      does not, and it is **larger**: nothing in this",
        "  dialog                              dialog is painted in the scrim's role, so all 600 of",
        "                                      its cells differ from what the scrim wrote. §2's 229",
        "                                      says 371 of C01's 600 already carried it. The",
        "                                      relation §2 states — against 0 for a scrim drawn",
        "                                      around the dialog — reproduces exactly.",
        "  600 fewer writes for the   600      reproduces, and it is stated as a construction rather",
        "  scrim drawn around                  than a measurement: §2's figure *is* the dialog's cell",
        "                                      count, so the dialog is 60x10.",
        "  214.58 / 84.96 us          -        not compared. A timing is a report (R15) and these",
        "                                      were taken on a machine and a build profile this",
        "                                      ticket does not have. The microsecond column above is",
        "                                      printed and gated by nothing.",
        "  20 784 writes for the      24 000   does not, and the direction is the interesting half:",
        "  correct build                       this screen writes *more* than C01's because it is a",
        "                                      full partition of its rectangle and C01's left a tail",
        "                                      nobody wrote. `crate::form` keeps the other shape on",
        "                                      purpose, so register row 7's *no cell never* has a",
        "                                      screen to stand on.",
    ] {
        println!("{line}");
    }
    println!();

    // ── the scene list, with the three rows this ticket moved ────────────────────────────────────
    println!("report  §21's scene list, and the three rows this ticket stood up:");
    let printed = scenes::report(&[]);
    for line in printed.lines().take(3) {
        println!("{line}");
    }
    println!("  ... (twenty-five more) ...");
    for line in printed.lines() {
        if line.contains("the narrow axis over the dense screen") {
            println!("{line}");
        }
    }
    println!();
    println!("  Stood up, and components ticket 10 is what moved them. They were `Red` for one");
    println!("  ticket: the screen was drawn and measured out of `fit`, `block` and `press`, and");
    println!("  what was missing was the subject — `text`, `chip`, `button` and `panel`. A scene");
    println!("  that fails because it is unimplemented and one that fails because the code is");
    println!("  wrong are the same failure unless the message separates them, and");
    println!("  `dense::owed_message` is still the sentence that separates them.");

    // The **shape**, so that a report which has quietly started measuring something smaller fails
    // rather than looking good. R15's rule, and the reason a report is allowed to carry asserts.
    //
    // **No second copy of a count this crate already owns.** Components ticket 08 left
    // `gates_numbers.rs` asserting `REGISTER.len() == 47` against a register that had reached 56,
    // green throughout, because an example is compiled and never evaluated. Every figure below reads
    // the ledger constant rather than restating it.
    assert_eq!(measured[0], dense::CLEARED_EVERY_FRAME);
    assert_eq!(measured[1], dense::CHIP_FILLED_FACE);
    assert_eq!(measured[2], dense::CHIP_NOT_NARROWED);
    assert_eq!(under.per_frame, dense::SCRIM_UNDER);
    assert_eq!(around.per_frame, 0);
    assert_eq!(border.per_frame, dense::BORDER_OVER_TITLE_SCREEN);
    assert_eq!(wide.regions, dense::REGIONS);
    assert_eq!(narrow.regions, dense::NARROW_REGIONS);
    assert!(
        dense::standing().met(),
        "the three scenes stand on four declared components since components 10"
    );
    assert!(matches!(
        Reading::Measured(1).get(Counter::Writes),
        1..=u64::MAX
    ));
}
