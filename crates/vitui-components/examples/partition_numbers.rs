//! components ticket 06 — `text::fit`, `frame::block` and the partition rule, as numbers.
//!
//! A **report**, not a gate: R15's refinement 2 says a row may cite one and may never rest on one,
//! and `cargo test` does not run a `fn main`. What gates every number below is `src/form.rs`'s test
//! module, `src/frame.rs`'s and `src/text.rs`'s, and `crate::form`'s ledger block is where each one
//! has its single home.
//!
//! It prints five things:
//!
//! 1. **The form**, at both densities, with what falls off the bottom.
//! 2. **The four arms** — routed, hand-written, clearing, naive — in §20's counter columns.
//! 3. **The equality**: `fit` against the same order written out, cell for cell.
//! 4. **The 15-cell instance**, watched firing.
//! 5. **What does not reproduce**, said out loud rather than engineered away.

use vitui_components::counters::Tally;
use vitui_components::form;
use vitui_components::frame::{BlockOpts, block_into, defective};
use vitui_components::runner::{Painter, compare_at, driver_at, play_at};
use vitui_runtime::Density;
use vitui_runtime::layout::text::width;

const DENSITIES: [Density; 2] = [Density::Compact, Density::Cosy];

fn arms() -> [(&'static str, Painter); 4] {
    [
        ("routed", form::routed as Painter),
        ("by_hand", form::by_hand as Painter),
        ("clearing", form::clearing as Painter),
        ("naive", form::naive as Painter),
    ]
}

fn main() {
    println!("components ticket 06 — the two partition primitives\n");

    // ── 1. the form ──────────────────────────────────────────────────────────────────────────────
    println!(
        "report  the form: {}x{}, {} panels, {} widgets asked for",
        form::W,
        form::H,
        form::PANELS,
        2 * form::LONG + form::SHORT
    );
    println!(
        "  {:<9}  {:>8}  {:>8}  {:>8}  {:>8}  {:>12}",
        "density", "visible", "dropped", "regions", "handed", "unwritten"
    );
    for density in DENSITIES {
        let shape = form::shape(density);
        let unwritten = form::SCREEN
            - play_at(density, form::routed, &form::steps())
                .tally()
                .distinct();
        println!(
            "  {:<9}  {:>8}  {:>8}  {:>8}  {:>8}  {:>12}",
            format!("{density:?}"),
            shape.visible,
            shape.dropped,
            shape.regions,
            shape.handed_over,
            unwritten
        );
    }
    println!(
        "\n  Four widgets fall off between the two, and it is arithmetic rather than a target:"
    );
    println!(
        "  `pad_y` is 1 at Compact and 2 at Cosy, two of the three panels overflow, and each of"
    );
    println!("  those two loses a row at the top and a row at the bottom.\n");

    // ── 2. the four arms ─────────────────────────────────────────────────────────────────────────
    println!("report  the four arms, in §20's columns. `marked` is unreachable here (ADR 0023):");
    println!(
        "  {:<9}  {:<9}  {:>8}  {:>9}  {:>8}  {:>7}  {:>9}",
        "density", "arm", "writes", "distinct", "excess", "verbs", "reported"
    );
    for density in DENSITIES {
        for (name, paint) in arms() {
            let run = play_at(density, paint, &form::steps());
            let t = run.tally();
            println!(
                "  {:<9}  {:<9}  {:>8}  {:>9}  {:>8}  {:>7}  {:>9}",
                format!("{density:?}"),
                name,
                t.writes(),
                t.distinct(),
                t.writes() - t.distinct(),
                t.verbs(),
                t.reported()
            );
        }
    }
    println!(
        "\n  Every arm's `reported` equals its `writes`, which is what makes the pair a comparison"
    );
    println!("  between two sources rather than a model against itself: nothing here goes through");
    println!("  `Ctx::fill`, because `Ctx::fill` returns `()` and has nothing to report.\n");
    println!(
        "  **The defective arms touch more cells than the correct one.** 24 000 of 24 000 against"
    );
    println!(
        "  18 912 — so register row 7's metric, *no cell never*, scores them as the healthier"
    );
    println!("  build. Only the pair separates them.\n");

    // ── 3. the equality ──────────────────────────────────────────────────────────────────────────
    println!("report  `fit` against the same order written by hand:");
    for density in DENSITIES {
        let diff = compare_at(density, form::routed, form::by_hand, &form::steps());
        let routed = play_at(density, form::routed, &form::steps());
        let hand = play_at(density, form::by_hand, &form::steps());
        println!(
            "  {:<9}  {diff}, at {} writes against {}",
            format!("{density:?}"),
            routed.tally().writes(),
            hand.tally().writes()
        );
    }
    println!(
        "\n  Spec §3 remembers 0 of 24 000 at 20 804 against 20 804, on the dense screen scene 1"
    );
    println!(
        "  stands up and components ticket 09 owns. The equality reproduces; the magnitude is"
    );
    println!("  a different screen's.\n");

    // ── 4. the 15-cell instance ──────────────────────────────────────────────────────────────────
    println!("report  the 15-cell instance, watched firing:");
    let opts = BlockOpts {
        title: form::TITLE,
        ..BlockOpts::default()
    };
    let mut correct = Tally::new();
    let mut broken = Tally::new();
    let mut driver = driver_at(60, 12, Density::Compact);
    driver.frame(|cx| {
        block_into(&mut correct, cx, cx.area(), &opts);
    });
    let mut driver = driver_at(60, 12, Density::Compact);
    driver.frame(|cx| {
        defective::block_over_title(&mut broken, cx, cx.area(), &opts);
    });
    println!(
        "  {:<10}  {:>8}  {:>9}  {:>8}  {:>7}",
        "panel", "writes", "distinct", "excess", "verbs"
    );
    for (name, t) in [("correct", &correct), ("defective", &broken)] {
        println!(
            "  {:<10}  {:>8}  {:>9}  {:>8}  {:>7}",
            name,
            t.writes(),
            t.distinct(),
            t.writes() - t.distinct(),
            t.verbs()
        );
    }
    println!(
        "\n  The title `{}` is {} columns and the excess is {}. The defective panel touches the",
        form::TITLE,
        width(form::TITLE),
        broken.writes() - broken.distinct()
    );
    println!(
        "  same cells as the correct one and costs one verb fewer — so it is *cheaper*, and the"
    );
    println!(
        "  only counter that disapproves is the pair. Nothing about a border run crossing its"
    );
    println!("  own title looks like a fill.\n");

    // ── 5. what does not reproduce ───────────────────────────────────────────────────────────────
    println!("report  what does not reproduce, and why:");
    for line in [
        "  ticket 06 / spec §3          here          why",
        "  0 of 24 000 at 20 804 vs     0 of 24 000   reproduces. The write count is this form's,",
        "  20 804                       at 18 912     not the prototype's dense screen — that screen",
        "                                             is scenes 1-2 and components ticket 09 owns it.",
        "  267 regions vs 263           171 vs 167    the delta is 4 either way, which is the number",
        "                                             that is a property of `pad_y` and the panel",
        "                                             count. The magnitude is the other screen's.",
        "  20 804 writes vs 20 992      18 912 vs     the *direction* reproduces: Cosy writes more",
        "                               19 206        while standing fewer widgets, because a thicker",
        "                                             ring is more cells of ring and the rows the",
        "                                             lost widgets would have taken were nobody's.",
        "  four widgets fall off        four          reproduces exactly, and by construction.",
        "  43 941 naive against 0       40 925        reproduces in shape. Three of ADR 0026's five",
        "                                             instances are on this screen; the sizes of the",
        "                                             panels are not the prototype's.",
        "  22 200 damaged across        21 312 handed the ADR's quantity is the interior; the gate's",
        "  three panels                 16 224 twice  is the collision, which is smaller by the third",
        "                                             panel's unwritten tail. Both are recorded.",
    ] {
        println!("{line}");
    }

    // The **shape**, so that a report which has quietly started measuring something smaller fails
    // rather than looking good. R15's rule, and the reason a report is allowed to carry asserts.
    assert_eq!(width(form::TITLE), 15);
    assert_eq!(broken.writes() - broken.distinct(), 15);
    assert_eq!(correct.writes() - correct.distinct(), 0);
    assert_eq!(form::shape(Density::Compact).regions, form::COMPACT_REGIONS);
    assert_eq!(
        form::shape(Density::Cosy).dropped - form::shape(Density::Compact).dropped,
        form::DROPPED_BY_COSY
    );
}
