//! **The wheel gate, over two shipped components and both axes, as numbers.**
//!
//! Components ticket 20. The convention is the runtime's — a file in `examples/` named
//! `<subject>_numbers.rs` that prints the numbers a human reads — and so is the rule about what an
//! example may be: **`cargo test` does not run this file**, and no row of
//! [`vitui_components::gates::REGISTER`] rests on it. Row 29 *cites* it and names two `#[test]`s
//! beside the citation.
//!
//! # What it prints
//!
//! 1. **The three arms over a virtualised `collection`**, in both directions: where twenty posted
//!    wheel clicks settle, and what a keyboard reveal moves afterwards. Neither column alone is the
//!    gate — `never` passes the first and fails the second, which is what a one-directional spelling
//!    green-lights.
//! 2. **The same three arms over a `scroll_area`, per axis**, which is where the pair that could not
//!    be seen before shows up: **a body dead downward is alive sideways**, and one number for *the
//!    offset* says nothing about either.
//! 3. **What the two removed substitutions were**, because the figures moved when they came out and
//!    a reader deserves to know which ones and why.
//!
//! # It asserts the shape and not the timings
//!
//! R15's rule, inherited: *a gate is a count, a ratio, an equality or a compile outcome; a timing is
//! a report*. There is no timing here at all — every number below is an offset in rows or columns.

use vitui_components::wheel::{
    self, Along, CLICKS, DRAGGED_BACK, MOVED, Play, Reveal, SCROLLED_AWAY, Subject,
};

fn main() {
    println!(
        "The wheel gate — {}x{}, twenty posted clicks, and the reveal that decides whether they \
         move anything\n",
        wheel::W,
        wheel::H
    );

    the_collection();
    the_area_per_axis();
    what_the_substitutions_were();
}

/// 1. The three arms over the shipped `collection`, in both directions.
fn the_collection() {
    println!("report  a virtualised `collection`, a million rows, {CLICKS} clicks down:");
    println!(
        "  {:<18}  {:>8}  {:>14}  {:>9}  {:>16}",
        "reveal", "settled", "after click 20", "requests", "keyboard reveal"
    );
    for reveal in [Reveal::WhenAsked, Reveal::EveryFrame, Reveal::Never] {
        let played = wheel::wheeled(Play::of(Subject::Collection, reveal));
        let keyboard = wheel::revealed(Subject::Collection, reveal, (0, SCROLLED_AWAY));
        println!(
            "  {:<18}  {:>8}  {:>14}  {:>9}  {:>16}",
            reveal.word(),
            played.settled.1,
            played.after_last_click.1,
            played.reveals,
            keyboard.1,
        );
    }
    println!("\n  Both directions, and neither alone is the gate:");
    println!("    the wheel half   `every frame` settles at 0 where the rule settles at {MOVED}.");
    println!("    the reveal half  `never` passes the wheel half and moves the offset 0 when a");
    println!(
        "                     keyboard reveal really asks — so deleting the call is not a fix."
    );
    println!(
        "  The `after click 20` column is 1 and not 0 for the defective arm: a request crosses the\n  \
         frame boundary and is read on the frame after (ADR 0015), so the defect is always one\n  \
         click ahead of its own correction. That one click is the runtime's own documented residue."
    );
    println!(
        "  A frame at offset 0 leaves no request on **any** arm ({}), which is the frame this\n  \
         defect is invisible on and the reason `leaves_no_request` takes an offset.\n",
        wheel::leaves_no_request(Play::of(Subject::Collection, Reveal::EveryFrame), (0, 0))
    );

    assert_eq!(
        wheel::wheeled(Play::of(Subject::Collection, Reveal::WhenAsked)).settled,
        (0, MOVED)
    );
    assert_eq!(
        wheel::wheeled(Play::of(Subject::Collection, Reveal::EveryFrame)).settled,
        (0, DRAGGED_BACK)
    );
    println!("report  a press over a row, {SCROLLED_AWAY} rows into the content:");
    println!(
        "  {:<18}  {:>10}  {:>14}  {:>9}  {:>16}",
        "reveal", "asked first", "asked on press", "selected", "settled"
    );
    for reveal in [Reveal::WhenAsked, Reveal::EveryFrame] {
        let tap = wheel::tapped(Play::of(Subject::Collection, reveal), (0, SCROLLED_AWAY));
        println!(
            "  {:<18}  {:>10}  {:>14}  {:>9}  {:>16}",
            reveal.word(),
            tap.before,
            tap.asked,
            tap.selected,
            tap.settled.1,
        );
    }
    println!(
        "\n  **A press pulls nothing**, because a press already proves the row was on screen — and\n  \
         the row is selected on the same gesture, so this is not a frame where nothing happened.\n  \
         The clause can only be seen at a non-zero offset, and the unconditional arm cannot be\n  \
         watched failing it: by the frame the press edge lands on it has already dragged the\n  \
         viewport all the way back, so the loudest arm is silent for the quietest reason. What it\n  \
         is watched doing is asking on the frame *before* any gesture — the `asked first` column.\n"
    );

    let tap = wheel::tapped(
        Play::of(Subject::Collection, Reveal::WhenAsked),
        (0, SCROLLED_AWAY),
    );
    assert!(!tap.asked);
    assert_eq!(tap.selected, 1);

    assert_eq!(
        wheel::revealed(Subject::Collection, Reveal::Never, (0, SCROLLED_AWAY)),
        (0, 0)
    );
}

/// 2. The same gate over `scroll_area`, per axis. **This is the row that could not be printed
///    before**, because the arithmetic substitution had no second axis to be wrong on.
fn the_area_per_axis() {
    let max = wheel::area_max();
    println!(
        "report  a `scroll_area` over {} x {} content cells (max offset {} x {}), per axis:",
        wheel::EXTENT.0,
        wheel::EXTENT.1,
        max.0,
        max.1
    );
    println!(
        "  {:<18}  {:<9}  {:>18}  {:>18}",
        "reveal", "pulls on", "20 clicks down", "20 clicks across"
    );
    for reveal in [Reveal::WhenAsked, Reveal::EveryFrame, Reveal::Never] {
        for pull in [Along::Rows, Along::Columns] {
            let base = Play::of(Subject::Area, reveal).pulling(pull);
            println!(
                "  {:<18}  {:<9}  {:>18}  {:>18}",
                reveal.word(),
                pull.word(),
                wheel::wheeled(base.along(Along::Rows)).on(Along::Rows),
                wheel::wheeled(base.along(Along::Columns)).on(Along::Columns),
            );
            if reveal != Reveal::EveryFrame {
                // The two conditional arms cannot depend on `pull` at all — the request is either
                // made once or never made — so printing the second row is printing the first twice.
                break;
            }
        }
    }
    println!(
        "\n  **A body dead downward is alive sideways.** The two `every frame` rows are the finding:\n  \
         a body that asks for content row 0 on every frame settles a vertical click at {DRAGGED_BACK}\n  \
         and a horizontal one at {MOVED}, and the transpose does the opposite. `Area::into_view`\n  \
         answers per axis and returns 0 for an axis the rectangle already sits inside, so *the*\n  \
         offset is never the question — one of the two is.\n"
    );

    let pull_rows = Play::of(Subject::Area, Reveal::EveryFrame).pulling(Along::Rows);
    assert_eq!(wheel::wheeled(pull_rows).on(Along::Rows), DRAGGED_BACK);
    assert_eq!(
        wheel::wheeled(pull_rows.along(Along::Columns)).on(Along::Columns),
        MOVED
    );
}

/// 3. What came out of the instrument, because the numbers moved when it did.
fn what_the_substitutions_were() {
    println!("report  what components 20 took out of components 11's instrument:");
    println!("  the click     was the *delta* handed to the arithmetic `Response::scrolled` would");
    println!("                have delivered it to, because `Driver::post_mouse` takes a");
    println!("                `vitui_engine::Mouse` and a `Mouse` needs a `Buttons` and a");
    println!(
        "                `MouseKind` — and **neither of those was in `ENGINE_NAMES` at all**."
    );
    println!("                Runtime architecture issue 22 lifted it, so the click is posted now");
    println!("                and routed through the previous frame's hit index.");
    println!(
        "  the subject   was a row loop written beside the gate, because `collection` did not"
    );
    println!("                exist yet. It is `collection_into` and `scroll_area` now, and the");
    println!("                defective arms are `collect::defective::every_frame` and");
    println!("                `never_reveals` — one value apart from the shipped build, so a");
    println!("                reviewer's diff is one line.");
    println!(
        "\n  Neither substitution was on the side of either arm, which is why the *pinned* figure\n  \
         never moved: `0 against {MOVED}` is what components 11 reported and what the posted click\n  \
         reports. What the substitutions cost was the second subject and the second axis, and those\n  \
         are the rows above that did not exist before.\n"
    );
}
