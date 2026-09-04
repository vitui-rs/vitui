//! **The wheel gate, over five shipped components and both axes, as numbers.**
//!
//! Components ticket 20, production tickets 06, 09 and 07. **The opening sentence said *three* for
//! two tickets after the fourth and fifth subjects landed** — `cargo test` does not run this file,
//! so a false sentence here is compiled by `cargo clippy --all-targets` and read by nobody but a
//! human. Production 07 corrected it while adding the fifth.
//!
//! The convention is the runtime's — a file in `examples/` named
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
//! 3. **The same three arms over a `table`**, which is production 06's, and every column of it is
//!    asserted **equal to the collection's** rather than to a repeated constant — a table's row
//!    axis is a collection's, and this is where that stops being a sentence in spec §6.
//! 4. **The same three arms over a `tree`**, which is production 07's, asserted equal to the
//!    collection's the same way — and with the **press** clause beside them, because §7 adds a
//!    second pointer gesture on the chevron column and a press over the middle of the screen must
//!    therefore land exactly where it did before.
//! 5. **What the two removed substitutions were**, because the figures moved when they came out and
//!    a reader deserves to know which ones and why.
//!
//! # `Subject::Pane` has no section here and that is stated rather than hidden
//!
//! Production 09's fourth subject is `file_preview_pane` and its numbers are the area's arm for arm
//! — `crate::wheel`'s own `twenty_posted_clicks_move_a_panes_offset_twenty_and_the_numbers_are_\
//! the_areas` is the assertion. Adding a section for it is a report ticket's edit and not this
//! one's; what production 07 owed was that the sentence above stop saying three.
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
    the_table();
    the_tree();
    what_the_substitutions_were();
}

/// 3. The same three arms over the shipped `table`. **Production 06**, and the row that says how
///    much of spec §6's opening sentence is true.
fn the_table() {
    println!(
        "report  a `table` at three columns, a million rows, {CLICKS} clicks down, beside the \
         `collection`:"
    );
    println!(
        "  {:<18}  {:>8}  {:>12}  {:>9}  {:>16}  {:>10}",
        "reveal", "settled", "collection", "requests", "keyboard reveal", "agree"
    );
    for reveal in [Reveal::WhenAsked, Reveal::EveryFrame, Reveal::Never] {
        let table = wheel::wheeled(Play::of(Subject::Table, reveal));
        let coll = wheel::wheeled(Play::of(Subject::Collection, reveal));
        let keyboard = wheel::revealed(Subject::Table, reveal, (0, SCROLLED_AWAY));
        let agree = table.settled == coll.settled
            && table.reveals == coll.reveals
            && keyboard == wheel::revealed(Subject::Collection, reveal, (0, SCROLLED_AWAY));
        println!(
            "  {:<18}  {:>8}  {:>12}  {:>9}  {:>16}  {:>10}",
            reveal.word(),
            table.settled.1,
            coll.settled.1,
            table.reveals,
            keyboard.1,
            agree,
        );
        assert!(
            agree,
            "`table` and `collection` disagree on the {} arm",
            reveal.word()
        );
    }
    println!(
        "\n  **The `agree` column is the finding and the other columns are how it is checked.**\n  \
         `table = collection + column rectangles` says the row axis, the wheel, the keyboard and\n  \
         the reveal are all reached by calling `collection`, and a column split that had grown a\n  \
         second offset or a reveal of its own would show up as a `false` here rather than as a\n  \
         number a reader has to compare by eye. The wheel notch reaches **one** axis: a table's\n  \
         horizontal offset is the caller's and is `examples/grid_numbers.rs`'s question.\n"
    );
}

/// 4. The same three arms over the shipped `tree`. **Production 07**, and the row that says how
///    much of spec §7's opening sentence is true.
fn the_tree() {
    println!(
        "report  a `tree` over a nested index, a million rows, {CLICKS} clicks down, beside the \
         `collection`:"
    );
    println!(
        "  {:<18}  {:>8}  {:>12}  {:>9}  {:>16}  {:>8}  {:>10}",
        "reveal", "settled", "collection", "requests", "keyboard reveal", "press", "agree"
    );
    for reveal in [Reveal::WhenAsked, Reveal::EveryFrame, Reveal::Never] {
        let tree = wheel::wheeled(Play::of(Subject::Tree, reveal));
        let coll = wheel::wheeled(Play::of(Subject::Collection, reveal));
        let keyboard = wheel::revealed(Subject::Tree, reveal, (0, SCROLLED_AWAY));
        let tap = wheel::tapped(Play::of(Subject::Tree, reveal), (0, SCROLLED_AWAY));
        let press_agrees =
            tap == wheel::tapped(Play::of(Subject::Collection, reveal), (0, SCROLLED_AWAY));
        let agree = tree.settled == coll.settled
            && tree.after_last_click == coll.after_last_click
            && tree.reveals == coll.reveals
            && keyboard == wheel::revealed(Subject::Collection, reveal, (0, SCROLLED_AWAY))
            && press_agrees;
        println!(
            "  {:<18}  {:>8}  {:>12}  {:>9}  {:>16}  {:>8}  {:>10}",
            reveal.word(),
            tree.settled.1,
            coll.settled.1,
            tree.reveals,
            keyboard.1,
            tap.selected,
            agree,
        );
        assert!(
            agree,
            "`tree` and `collection` disagree on the {} arm",
            reveal.word()
        );
    }
    println!();
    println!(
        "  **The `agree` column is the finding and the other columns are how it is checked**,"
    );
    println!("  which is `the_table` one component up. §7 states the claim in more words than §6");
    println!("  does — *no second selection store, no second scan cursor, no second `Mode`, no");
    println!("  second offset and no second press edge* — so two verbs a row and a one-slot fold");
    println!("  request cost the row axis nothing, and a build where they did would read `false`.");
    println!();
    println!("  **The `press` column is what the table arm did not have to check.** A tree adds a");
    println!("  second pointer gesture on the chevron column, so *a press still selects a row and");
    println!(
        "  still refuses to pull the viewport* is a question rather than an inheritance. It is"
    );
    println!("  over the middle of the screen, nowhere near the chevron; the chevron's own column");
    println!("  is scene 46's and is `examples/tree_numbers.rs`'s third table.");
    println!();
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

/// 5. What came out of the instrument, because the numbers moved when it did.
///
/// **It read `3.` while `the_table` also read `3.`**, and had done since production 06 added that
/// section without renumbering this one. Corrected by production 07 while adding the fifth subject
/// — `cargo test` does not run this file, so nothing but a reader was ever going to catch it.
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
