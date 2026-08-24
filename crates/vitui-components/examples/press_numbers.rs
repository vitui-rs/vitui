//! components ticket 07 — `state::press`, `frame::face_paint` and `scroll::bar`, as numbers.
//!
//! A **report**, not a gate: R15's refinement 2 says a row may cite one and may never rest on one,
//! and `cargo test` does not run a `fn main`. What gates every number below is `src/state.rs`'s
//! test module, `src/frame.rs`'s and `src/scroll.rs`'s, and each figure has its single home in one
//! of the two ledger blocks — `crate::state`'s and `crate::scroll`'s.
//!
//! It prints six things:
//!
//! 1. **The chip**, resting for sixty frames, through the helper and written by hand.
//! 2. **What the other counters say about the same two chips** — which is nothing.
//! 3. **The thirty-two states**, and the six rungs of the precedence they land on.
//! 4. **The two bars**, thumb-first against track-first.
//! 5. **What `marked` would have said**, and why this crate cannot ask it.
//! 6. **What does not reproduce**, said out loud rather than engineered away.

use vitui_components::counters::{Allocations, Counter, Counters, Reading, Tally};
use vitui_components::frame::{Face, face_role};
use vitui_components::runner::{Pen, driver_at};
use vitui_components::scroll;
use vitui_components::state::{self, Chip};
use vitui_runtime::{Density, Role};

fn arms() -> [(&'static str, Chip); 2] {
    [
        ("press", state::chip_pressed as Chip),
        ("by_hand", state::defective::chip_by_hand as Chip),
    ]
}

fn main() {
    println!("components ticket 07 — one role, five bits and a thumb\n");

    // ── 1. the chip ──────────────────────────────────────────────────────────────────────────────
    println!(
        "report  a pointer resting on an {}-cell chip for {} frames:",
        state::CHIP,
        state::FRAMES
    );
    println!(
        "  {:<9}  {:>10}  {:>12}  {:>14}",
        "arm", "first frame", "per frame", "59 frames"
    );
    let mut measured = Vec::new();
    for (name, arm) in arms() {
        let r = state::resting(arm, state::FRAMES);
        println!(
            "  {name:<9}  {:>10}  {:>12}  {:>14}",
            r.first, r.per_frame, r.steady
        );
        measured.push(r);
    }
    println!(
        "\n  The first frame is the same for both and is not re-damage: eight cells that were"
    );
    println!(
        "  nobody's. Everything after it is the helper's — `press` reads `Faces::hover` once,"
    );
    println!("  declares the award from it and returns it, so the restyle writes a background the");
    println!("  chip's own cells already carry and the next draw writes the value back unchanged.");
    println!(
        "  Written by hand the two statements are eight lines apart, the screen is correct on"
    );
    println!("  every frame because the restyle repaints it, and every cell of the face changes");
    println!("  twice a frame for as long as the pointer rests.\n");

    // ── 2. what the other counters say ───────────────────────────────────────────────────────────
    println!("report  the same two chips, in §20's columns:");
    println!(
        "  {:<9}  {:>8}  {:>9}  {:>8}  {:>7}  {:>8}  {:>8}",
        "arm", "writes", "distinct", "excess", "verbs", "regions", "marked"
    );
    for (name, arm) in arms() {
        let mut driver = driver_at(state::W, state::H, Density::Compact);
        let mut pen = Pen::new(state::W, state::H);
        driver.frame(|cx| arm(&mut pen, cx));
        let counters = Counters::of(&driver, pen.tally(), Allocations::over(1, 0));
        let t: &Tally = pen.tally();
        println!(
            "  {name:<9}  {:>8}  {:>9}  {:>8}  {:>7}  {:>8}  {:>8}",
            t.writes(),
            t.distinct(),
            t.writes() - t.distinct(),
            t.verbs(),
            counters
                .regions
                .measured()
                .map_or("-".to_string(), |n| n.to_string()),
            counters
                .marked
                .measured()
                .map_or("-".to_string(), |n| n.to_string()),
        );
    }
    println!("\n  Identical in every column. `writes == distinct` — components ticket 06's whole");
    println!("  gate — is green on both, because neither chip writes a cell twice *in one frame*;");
    println!("  the defect is between two frames, and the only thing that can see it is a surface");
    println!("  that outlives one.\n");

    // ── 3. the thirty-two states ─────────────────────────────────────────────────────────────────
    println!("report  `Face` is five bits, and `face_paint` resolves all 32 through one ladder:");
    println!("  {:<16}  {:>6}  {:<10}", "rung", "states", "role");
    let mut counts: Vec<(Role, usize)> = Vec::new();
    for face in Face::ALL {
        let role = face_role(face);
        match counts.iter_mut().find(|(r, _)| *r == role) {
            Some((_, n)) => *n += 1,
            None => counts.push((role, 1)),
        }
    }
    for (rung, (role, n)) in [
        "disabled",
        "selected+active",
        "selected",
        "cursor",
        "hovered",
        "base",
    ]
    .into_iter()
    .zip(order(&counts))
    {
        println!("  {rung:<16}  {n:>6}  {:<10}", format!("{role:?}"));
    }
    println!(
        "\n  Thirty-two states over six rungs, none of them empty. C02's `Sel` enum named four,"
    );
    println!(
        "  and its variants are not exclusive: a row can be selected *and* hovered, and it can"
    );
    println!("  be the keyboard cursor without being selected — `Ctrl+↓`, and every file manager.");
    println!(
        "  `size_of::<Face>()` is {}.\n",
        std::mem::size_of::<Face>()
    );

    // ── 4. the two bars ──────────────────────────────────────────────────────────────────────────
    println!(
        "report  a two-bar {}x{} screen, thumb-first against track-first:",
        scroll::W,
        scroll::H
    );
    println!(
        "  {:<12}  {:>8}  {:>9}  {:>8}  {:>7}",
        "arm", "writes", "distinct", "excess", "verbs"
    );
    let mut excess = 0u64;
    for (name, track_first) in [("thumb-first", false), ("track-first", true)] {
        let mut driver = driver_at(scroll::W, scroll::H, Density::Compact);
        let mut tally = Tally::new();
        driver.frame(|cx| {
            if track_first {
                scroll::two_bars_track_first(&mut tally, cx);
            } else {
                scroll::two_bars(&mut tally, cx);
            }
        });
        println!(
            "  {name:<12}  {:>8}  {:>9}  {:>8}  {:>7}",
            tally.writes(),
            tally.distinct(),
            tally.writes() - tally.distinct(),
            tally.verbs()
        );
        if track_first {
            excess = tally.writes() - tally.distinct();
        }
    }
    println!(
        "\n  The excess is the two thumbs and nothing else — {} + {} — because the thumb is exactly",
        scroll::THUMB_V,
        scroll::THUMB_H
    );
    println!(
        "  what the groove was written under. The vertical thumb is one cell only because the"
    );
    println!(
        "  clamp saves it: {} cells of viewport over {} of extent is zero.\n",
        scroll::H - 1,
        scroll::ROWS
    );

    // ── 5. what `marked` would have said ─────────────────────────────────────────────────────────
    println!("report  the counter this ticket wanted, and the one it got:");
    for line in [
        "  `marked`      unreachable. `crates/vitui-engine/src/damage.rs` is `pub(crate)` from top",
        "                to bottom and `Presented` carries no count, so no crate above the engine",
        "                can read the engine's damage. `Counters::marked` panics rather than",
        "                answering 0 — a threshold on the wrong side of the question is not a weak",
        "                gate, it is a green one.",
        "  `repaints`    what ships instead. The engine filters a write whose value equals the",
        "                cell's current value, so the quantity that *becomes* damage is a write",
        "                whose value differs from what is already there — and that is computable",
        "                at the verb boundary over a surface this crate keeps itself.",
        "                Exact for the draw verbs. Exact for an award onto a cell already painted",
        "                in the awarded role, which is what a correct widget produces every frame.",
        "                Conservative for an award onto any other cell, and the fixture closes that",
        "                by asking `Theme::roles_differ_on_wire` of the three faces it wears.",
        "  the pointer   cannot be driven from here at all. `Driver::post_mouse` takes a",
        "                `vitui_engine::Mouse`, filed `reachable_as: None`; `Driver::plant` reaches",
        "                the grab, the focus and the click and not the pointer. So the fixture sets",
        "                `Response::hovered` itself and `Pen::end_frame` applies the award where",
        "                `Driver::frame` applies it. Both arms go through both substitutions.",
    ] {
        println!("{line}");
    }
    println!();

    // ── 6. what does not reproduce ───────────────────────────────────────────────────────────────
    println!("report  what does not reproduce, and why:");
    for line in [
        "  ticket 07 / spec §3        here     why",
        "  8 cells for as long as     8        reproduces exactly, and by construction: the number",
        "  the pointer rests                   *is* the chip's width.",
        "  0 through `press`          0        reproduces.",
        "  221 track-first on a       224      does not. The excess is `thumb_v + thumb_h` and both",
        "  two-bar screen                      thumbs are this screen's: 1 + 223 against a",
        "                                      prototype's 221. The *identity* reproduces — the",
        "                                      cells written twice are exactly the thumb — and the",
        "                                      magnitude is a function of one viewport, one extent",
        "                                      and one offset that this ticket does not own. It is",
        "                                      recorded rather than tuned: a fixture aimed at a",
        "                                      remembered number is a fixture that has stopped",
        "                                      measuring anything.",
        "  345 on §9's screen         -        not attempted. §9's screen is a 1M-row area with a",
        "                                      header band and a pinned column, which is four bars",
        "                                      and a `scroll_area` component. None of it exists;",
        "                                      it is scene 9 and components ticket 09 owns it.",
        "  32 states, 5 bytes         32, 5    reproduces exactly.",
    ] {
        println!("{line}");
    }

    // The **shape**, so that a report which has quietly started measuring something smaller fails
    // rather than looking good. R15's rule, and the reason a report is allowed to carry asserts.
    assert_eq!(measured[0].per_frame, state::PRESS_STEADY);
    assert_eq!(measured[1].per_frame, state::HAND_STEADY);
    assert_eq!(excess, scroll::TRACK_FIRST_EXCESS);
    assert_eq!(Face::ALL.len(), 32);
    assert_eq!(counts.iter().map(|(_, n)| n).sum::<usize>(), 32);
    assert!(matches!(
        Reading::Measured(1).get(Counter::Writes),
        1..=u64::MAX
    ));
}

/// The six roles in precedence order, with how many of the thirty-two states land on each.
fn order(counts: &[(Role, usize)]) -> Vec<(Role, usize)> {
    [
        Role::Disabled,
        Role::FaceActive,
        Role::Selection,
        Role::Focus,
        Role::FaceHover,
        Role::Body,
    ]
    .into_iter()
    .map(|role| {
        let n = counts
            .iter()
            .find(|(r, _)| *r == role)
            .map_or(0, |(_, n)| *n);
        assert!(
            n > 0,
            "{role:?} is reached by none of the thirty-two states"
        );
        (role, n)
    })
    .collect()
}
