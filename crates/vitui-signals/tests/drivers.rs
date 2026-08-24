//! **The same screen three ways, and the equality that is the whole of the boundary's claim.**
//!
//! Runtime ticket 18, runtime spec §18. Four gates, and one count that is a finding rather than a
//! gate:
//!
//! 1. **0 of 24 000 cells differ** between a direct application, a TEA pump with a pure reducer, and
//!    a signal graph — on a runtime none of them modified.
//! 2. **`Memo` recomputes once over eight frames, in all three drivers.** That is hook 4, and it is
//!    the one thing a graph can actually skip.
//! 3. The **undiffed** write never goes quiet: every frame is a change on a screen nobody touched,
//!    against **0** changes for the diffed one over the same run.
//! 4. `role_tag` is a bijection, which is what makes gate 1 an equality about roles rather than about
//!    a hash.
//!
//! And the asymmetry that belongs to the shapes rather than to the runtime: **TEA shows a new value
//! one frame later**, because a view draws the whole screen from one model and the footer is drawn
//! where the reducer has not run yet.
//!
//! # The runtime diff is 0 lines, and here it is a compile outcome
//!
//! The wayfinder prototype had to invert an include tree to be able to say *the runtime is
//! unmodified*. This crate does not: it depends on `vitui-runtime` and can reach nothing but its
//! `pub` surface, so the claim is `cargo build` rather than `git diff`. The manifest half — that
//! nothing in the workspace depends on **this** crate — is `tests/facade.rs`.
//!
//! # The equality is the caller's own ledger, and that is a change from the prototype
//!
//! The prototype read 24 000 cells back out of a prototype surface. **The shipped engine has no such
//! door**: no cell, handle or style bit is readable from outside it (ADR 0023), and this crate does
//! not depend on the engine in the first place. So the comparison is [`Ink`] — what each driver
//! *declared* it wrote, one entry a cell, retained between frames exactly as the engine's surfaces
//! are.
//!
//! That is weaker than a read-back, and it is said out loud rather than implied — the arrangement
//! `vitui-runtime/tests/swap.rs` reached one crate down, for the identical reason and in the same
//! words. What it still catches is the only thing this gate exists to catch: **a reactivity layer
//! quietly changing what is on screen.** Two drivers that paint different strings, in different
//! places, or in different roles are not equal here.

#[allow(dead_code)]
#[path = "../src/board.rs"]
mod board;

use board::{Data, Direct, H, Ink, Mark, Press, Signals, Tea, W, role_tag};
use vitui_runtime::Role;
use vitui_runtime::ctx::Driver;

/// Frames every run draws. Long enough that every driver has stopped asking for another, and fixed
/// rather than "until quiet" so that the three runs are comparable frame by frame.
const FRAMES: usize = 8;

/// The script: a click on a footer button, then a row selected, then the density toggled. Three
/// edges of three different shapes, because a screen that only ever moves an integer would not
/// notice a driver that painted the wrong role.
const SCRIPT: [Press; 3] = [Press::Bump(3), Press::Select(7), Press::Toggle];

/// What one driver's run produced.
struct Run {
    /// Every cell it declared it wrote, after the last frame.
    ink: Ink,
    /// The last frame, one-based, on which it still asked for another. Zero if it never did.
    asked_until: usize,
    /// The first frame, one-based, whose footer read what the final footer reads.
    showed: usize,
}

/// Draw `FRAMES` frames, handing `script` in one press a frame and `Press::Quiet` after it.
fn run(script: &[Press], mut step: impl FnMut(Press, &mut Ink) -> bool) -> Run {
    let mut ink = Ink::new();
    let mut asked = Vec::with_capacity(FRAMES);
    let mut footer = Vec::with_capacity(FRAMES);
    for i in 0..FRAMES {
        let press = script.get(i).copied().unwrap_or(Press::Quiet);
        asked.push(step(press, &mut ink));
        // The footer's first cell carries the hash of the whole footer string, so one mark answers
        // *what does the footer say* without the ledger having to keep the text.
        footer.push(ink.at(0, i32::from(H) - 1).map(|m: Mark| m.source));
    }
    let last = *footer.last().expect("FRAMES is not zero");
    Run {
        asked_until: asked.iter().rposition(|a| *a).map_or(0, |i| i + 1),
        showed: footer.iter().position(|f| *f == last).map_or(0, |i| i + 1),
        ink,
    }
}

fn direct(script: &[Press]) -> (Run, u32) {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    let data = Data::default();
    let mut app = Direct::default();
    let out = run(script, |press, ink| {
        let mut moved = false;
        let _ = driver.frame(|cx| moved = app.view(cx, ink, &data, press));
        moved
    });
    (out, app.max.recomputes)
}

fn tea(script: &[Press]) -> (Run, u32, u32) {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    let data = Data::default();
    let mut app = Tea::default();
    let out = run(script, |press, ink| {
        let mut moved = false;
        let _ = driver.frame(|cx| moved = app.view_and_pump(cx, ink, &data, press));
        moved
    });
    (out, app.max.recomputes, app.applied)
}

fn signals(script: &[Press]) -> (Run, u32, (u32, u32)) {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    let data = Data::default();
    let mut app = Signals::default();
    let out = run(script, |press, ink| {
        let mut moved = false;
        let _ = driver.frame(|cx| moved = app.flush(cx, ink, &data, press));
        moved
    });
    let writes = app.selected.writes();
    (out, app.max.recomputes(), writes)
}

/// **Gate 1, the equality: 0 of 24 000 cells differ.**
///
/// Three drivers, one script, one screen. All three land on the same state from the same edges and
/// paint the same thing — string, place and role — in every one of the 24 000 cells the terminal
/// has.
#[test]
fn the_same_screen_three_ways_differs_in_no_cell_of_twenty_four_thousand() {
    let (a, _) = direct(&SCRIPT);
    let (b, _, _) = tea(&SCRIPT);
    let (c, _, _) = signals(&SCRIPT);

    assert_eq!(
        board::CELLS,
        24_000,
        "the denominator is the dense screen's"
    );
    assert_eq!(
        a.ink.differing(&b.ink),
        0,
        "direct and TEA disagree about the screen"
    );
    assert_eq!(
        a.ink.differing(&c.ink),
        0,
        "direct and the signal graph disagree about the screen"
    );
    assert_eq!(a.ink.agreeing(&c.ink), board::CELLS);

    // **The negative control, and it is not decoration.** An equality over a ledger that recorded
    // nothing also reads zero, so the gate has to be shown capable of saying no: the same driver on
    // a script that never happened paints a different screen, and the count is large.
    let (idle, _) = direct(&[]);
    assert!(
        a.ink.differing(&idle.ink) > 0,
        "the ledger cannot tell two different screens apart, so gate 1 proves nothing"
    );
}

/// **The asymmetry, as a count, and it belongs to the shapes.**
///
/// One press, and the question is *which frame first shows the new value*. The direct driver and the
/// graph both show it on the frame the press arrived; TEA shows it on the next one.
///
/// It is not the queue and it is not the runtime. A view draws the **whole** screen from **one**
/// model, so any part drawn after the reducer would run is still reading the borrow the reducer
/// wants exclusively. The graph escapes it by writing during the frame and reading the graph again
/// below the write, which is what interior mutability buys and what TEA refuses on purpose. Same
/// trade TEA makes over any renderer.
#[test]
fn tea_shows_a_new_value_one_frame_later_and_the_other_two_do_not() {
    let one = [Press::Bump(1)];
    let (a, _) = direct(&one);
    let (b, _, applied) = tea(&one);
    let (c, _, _) = signals(&one);

    assert_eq!(a.showed, 1, "the direct driver writes at the call site");
    assert_eq!(c.showed, 1, "the graph reads again below the write");
    assert_eq!(b.showed, 2, "TEA's footer draws from the pre-message model");
    assert_eq!(applied, 1, "one click is one message");

    // And all three stop asking. A driver that never goes quiet is section 3's failure, not this
    // one's, but the frame it stops on is worth pinning: the press is frame 1, and nothing after it
    // has anything to ask for.
    for (name, r) in [("direct", &a), ("tea", &b), ("signals", &c)] {
        assert!(
            r.asked_until <= 2,
            "{name} was still asking for frames at {}",
            r.asked_until
        );
    }
}

/// **Gate 2, hook 4: the memo recomputes once, in all three drivers.**
///
/// The aggregate is the one value on this screen a reactivity layer could plausibly claim to skip,
/// and all three skip it — because all three use the runtime's `Memo`, keyed on the data's revision.
/// `Computed` is that `Memo` with the key filled in, and this gate is where the two are the same
/// number rather than the same sentence.
#[test]
fn the_aggregate_is_folded_once_over_eight_frames_in_every_driver() {
    let (_, a) = direct(&SCRIPT);
    let (_, b, _) = tea(&SCRIPT);
    let (_, c, _) = signals(&SCRIPT);
    assert_eq!(
        (a, b, c),
        (1, 1, 1),
        "the data never changed; the fold ran once"
    );
}

/// **Gate 3: the write a signal library reaches for first never goes quiet.**
///
/// `Versioned::edit` needs no `PartialEq` and the runtime already ships the guard, so it is the
/// obvious way to build a signal — and it **bumps on drop, not on change**, which is one of the
/// type's three stated prices. A data contract can pay it; a change signal cannot.
///
/// The numbers are per frame and the screen is untouched: the diffed write attempts one and changes
/// none, the undiffed one attempts one and changes one. The consequence is the row below the counts:
/// the undiffed graph asks for a frame on **every** frame, for ever, and it asks through
/// `request_frame()` — so the runtime's own wake ledger sees it, which is what makes a graph that
/// spins a counted defect rather than a wakeup source nobody owns.
#[test]
fn the_undiffed_write_never_goes_quiet_and_the_diffed_one_does() {
    let quiet: [Press; 0] = [];

    let (diffed, _, writes) = signals(&quiet);
    assert_eq!(
        writes,
        (FRAMES as u32, 0),
        "the diffed write attempts one a frame and changes none"
    );
    assert_eq!(
        diffed.asked_until, 1,
        "a graph starts dirty, settles on frame 1, and never asks again"
    );

    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    let data = Data::default();
    let mut app = Signals::default();
    let undiffed = run(&quiet, |press, ink| {
        let mut moved = false;
        let _ = driver.frame(|cx| moved = app.flush_undiffed(cx, ink, &data, press));
        moved
    });
    assert_eq!(
        app.selected.writes(),
        (FRAMES as u32, FRAMES as u32),
        "the undiffed write changes on every frame, because the guard bumps on drop"
    );
    assert_eq!(
        undiffed.asked_until, FRAMES,
        "and it therefore asks for a frame on every frame, for as long as you care to run it"
    );
}

/// **Gate 4: the role tag is a bijection.**
///
/// The ledger compares roles by their position in `Role::ALL`, because `Role::index` is private and
/// the enum offers a consumer no numeric cast. That is exact rather than folded, and this is the
/// assertion that keeps it exact — a collision would make two roles compare *equal* and quietly
/// weaken gate 1 rather than break it.
#[test]
fn every_role_has_its_own_tag() {
    let mut seen = Vec::new();
    for role in Role::ALL {
        let tag = role_tag(role);
        assert!(!seen.contains(&tag), "{role:?} collides at tag {tag}");
        seen.push(tag);
    }
    assert_eq!(seen.len(), Role::ALL.len());
}
