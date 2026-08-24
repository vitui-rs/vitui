//! **The barrier is a count, and the type system is not it.**
//!
//! Runtime ticket 18, runtime spec §18. A signal graph's whole promise is that a write reaches only
//! the widgets that read it. The expected barrier was the type system — `Ctx<'f, 'v>` is invariant
//! in `'f`, an overlay body is `+ 'f`, an `impl FnOnce(&mut Ctx)` forces `'static` — so a stored
//! `Box<dyn Fn(&mut Ctx))>` looked like the thing that would refuse.
//!
//! **It does not refuse.** Elision gives `for<'f, 'v>`, the effect is callable from inside any frame,
//! and the rectangle it captures to scope itself is `Copy` and costs nothing to store. The positive
//! case in this file compiles, runs and passes, and it is kept **with its count beside it**, which is
//! the only honest way to ship a shape that works and must not be used.
//!
//! The barrier is what the frame *declares*:
//!
//! | | full draw | one region redrawn |
//! |---|---|---|
//! | hit entries | **312** | **1** |
//! | tab stops | **43** | **0** |
//! | cells still correct | 24 000 of 24 000 | see [`the_picture_survives_and_the_frame_does_not`] |
//!
//! `Frame::begin` rebuilds the hit index, the focus ring, the overlay queue and the deadline sink
//! **from the draw**, so a widget that did not draw is in none of them. And **the loss is silent for
//! exactly one frame**, because routing answers from the *previous* frame's index: the first partial
//! frame still answers the pointer, and the second answers nothing at all.
//!
//! > So a subscription that redraws one region is asking the runtime to keep the other 311 entries
//! > from last frame's geometry, permanently. **Fine-grained reactivity here is not expensive; it is
//! > a request to revert the hit index** — runtime spec §6, §7 and §8, three closed decisions, and
//! > no compiler could have said so.

#[allow(dead_code)]
#[path = "../src/board.rs"]
mod board;

use board::{Board, Data, H, Ink, Press, W};
use vitui_runtime::Ctx;
use vitui_runtime::ctx::Driver;

/// Draw the whole screen once into a fresh driver, and hand back what it declared.
fn full() -> (Driver, Ink, Board) {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    let data = Data::default();
    let mut app = board::Direct::default();
    let mut ink = Ink::new();
    let _ = driver.frame(|cx| {
        let _ = app.view(cx, &mut ink, &data, Press::Quiet);
    });
    (driver, ink, app.board)
}

/// **The count, in both directions.**
///
/// The full draw declares 312 hit entries and 43 tab stops — 40 loose ones plus one for each of
/// three groups, which is 27 entries collapsed onto 3. The next frame redraws the footer alone, and
/// declares 1 and 0.
#[test]
fn a_frame_that_redraws_one_region_declares_one_hit_entry_against_312() {
    let (mut driver, mut ink, mut app_board) = full();
    assert_eq!(driver.inspect().hits().len(), board::REGIONS);
    assert_eq!(driver.inspect().stop_count(), board::STOPS);
    assert_eq!((board::REGIONS, board::STOPS), (312, 43));

    // The subscription fires: `count` moved, and the only region that reads it is the footer. So the
    // frame draws the footer and nothing else — which is exactly what a fine-grained graph is for.
    app_board.count += 1;
    let _ = driver.frame(|cx| {
        board::footer(cx, &mut ink, &app_board, 0.0);
    });

    assert_eq!(
        driver.inspect().hits().len(),
        1,
        "the footer declared one interactive region and the other 311 are gone"
    );
    assert_eq!(
        driver.inspect().stop_count(),
        0,
        "and every tab stop with them"
    );
}

/// **The picture survives and the frame does not**, which is the sentence the numbers above are for.
///
/// The partial frame is *correct about the screen*: the ledger is retained between frames because
/// the engine's surfaces are, so every cell the partial frame did not touch still holds what the
/// full frame put there — and the footer, which is the only thing that moved, was redrawn. The
/// comparison is against a full redraw of the same state.
///
/// **24 000 of 24 000**, on this screen. The wayfinder prototype measured 23 993 of 24 000 on its
/// own screen, where the region redrawn clipped a neighbour's label; the difference is the fixture
/// and not the mechanism, and the mechanism is the same either way: **the cells are fine and the
/// declarations are gone.**
#[test]
fn the_picture_survives_and_the_frame_does_not() {
    let (mut driver, mut partial, mut app_board) = full();

    app_board.count += 1;
    let _ = driver.frame(|cx| {
        board::footer(cx, &mut partial, &app_board, 0.0);
    });

    // What the same state would have looked like drawn whole.
    let mut whole_driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    let data = Data::default();
    let mut whole = Ink::new();
    let mut app = board::Direct::default();
    app.board = app_board;
    let _ = whole_driver.frame(|cx| {
        let _ = app.view(cx, &mut whole, &data, Press::Quiet);
    });
    // The full driver folds the aggregate; the partial frame above was handed 0.0 because a
    // subscription on `count` has no reason to touch the data's memo. Redraw the footer with the
    // aggregate the whole frame used, so that the comparison is about *what the partial frame
    // skipped* rather than about an argument it was passed.
    let max = *app.max.peek().expect("the whole frame folded it");
    let _ = driver.frame(|cx| {
        board::footer(cx, &mut partial, &app_board, max);
    });

    assert_eq!(
        partial.agreeing(&whole),
        board::CELLS,
        "the partial frame left the screen wrong somewhere"
    );
    assert_eq!(board::CELLS, 24_000);

    // …and the frame that produced that perfect picture declares nothing.
    assert_eq!(driver.inspect().hits().len(), 1);
    assert_eq!(driver.inspect().stop_count(), 0);
}

/// **The loss is silent for exactly one frame.**
///
/// Routing answers from the *previous* frame's index, so the first partial frame still holds a full
/// index to answer from and the second does not. Here that is a count over two frames rather than a
/// claim about a pointer this crate cannot post: after one partial frame the index holds 1 entry, and
/// after two it still holds 1 — the 312 that answered the first one are gone, and nothing rebuilt
/// them.
#[test]
fn the_index_is_gone_after_one_partial_frame_and_stays_gone() {
    let (mut driver, mut ink, app_board) = full();
    let mut counts = Vec::new();
    for _ in 0..3 {
        let _ = driver.frame(|cx| {
            board::footer(cx, &mut ink, &app_board, 0.0);
        });
        counts.push(driver.inspect().hits().len());
    }
    assert_eq!(counts, vec![1, 1, 1]);
}

/// **The positive case: a stored `Box<dyn Fn(&mut Ctx)>` compiles, runs and passes.**
///
/// This is the shape a fine-grained subscription is built out of — an effect captured once and
/// re-run when its inputs move — and there is no type-level reason it cannot exist. Elision gives
/// `for<'f, 'v>`, so the closure is callable from inside *any* frame, and what it captures to scope
/// itself is `Copy`.
///
/// It is kept because a barrier that is a count has to be shown not to be a compile error, and
/// **the count is beside it**: run every effect and the screen declares 312; run one and it declares
/// one. Nothing here recommends the shape.
#[test]
fn a_stored_effect_compiles_and_runs_and_the_count_is_what_refuses() {
    type Effect = Box<dyn Fn(&mut Ctx<'_, '_>, &mut Ink)>;

    let effects: Vec<Effect> = vec![
        Box::new(|cx, ink| board::declare(cx, ink, false)),
        Box::new(|cx, ink| board::footer(cx, ink, &Board::default(), 0.0)),
    ];

    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    let mut ink = Ink::new();

    // All of them: the screen declares what a whole draw declares.
    let _ = driver.frame(|cx| {
        for effect in &effects {
            effect(cx, &mut ink);
        }
    });
    assert_eq!(driver.inspect().hits().len(), board::REGIONS);
    assert_eq!(driver.inspect().stop_count(), board::STOPS);

    // One of them — which is the whole promise of a subscription, and the whole cost of it.
    let _ = driver.frame(|cx| {
        effects[1](cx, &mut ink);
    });
    assert_eq!(driver.inspect().hits().len(), 1);
    assert_eq!(driver.inspect().stop_count(), 0);
}
