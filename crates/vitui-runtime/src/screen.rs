//! One realistic screen, shared by every report and gate that needs one.
//!
//! **Three copies of this function existed before it was a module** — `tests/alloc.rs`,
//! `examples/layout_numbers.rs` and `examples/text_numbers.rs` — and all three asserted the same two
//! counts, which is what kept them honest. That is a working arrangement and not a good one: the
//! counts catch a copy that *drifts*, and nothing catches three copies being edited three times.
//!
//! Written against the **public** API — `use vitui_runtime::layout::…`, resolved inside the library
//! by the `extern crate self` alias in `lib.rs` — which is the better half of the engine's reason for
//! the same arrangement: a fixture that could reach past the public surface would be measuring
//! something a caller cannot do.
//!
//! `#[path]`-included rather than exported, which is the engine's own arrangement for its scene list
//! (`crates/vitui-engine/src/scenes.rs`) and for the same two reasons: an integration test cannot see
//! an example's private items, an example cannot be imported, and **this is an instrument rather than
//! part of the library**. Putting a benchmark fixture on the public surface would be putting it in
//! front of the runtime's users.
//!
//! ```text
//! #[path = "../src/screen.rs"]
//! mod screen;
//! ```
//!
//! # The shape, and why it is a count
//!
//! A header of three regions with two sub-splits, a twenty-column sidebar holding twenty-four list
//! rows of four columns each, a gutter, a detail pane of three stacked panels, and a footer of six
//! buttons: **thirty-two splits and one hundred and nineteen lanes.**
//!
//! [`screen_frame`] returns those two numbers so that the shape is a *count* rather than a
//! description. Every caller asserts `(32, 119)`, so a report that quietly started measuring a
//! smaller screen fails instead of looking good — which is the same rule §14 applies to a scene list:
//! *the scene list is normative, not an appendix.*

use vitui_engine::Rect;

use vitui_runtime::layout::{
    Col,
    Constraint::{Fixed, Weight},
    Row,
};

/// How many list rows the sidebar holds.
pub const LIST_ROWS: usize = 24;

/// Lay out a whole screen, and return `(splits, lanes)`.
///
/// The counts are the point: see the module comment. Every caller asserts `(32, 119)`.
pub fn screen_frame(w: u16, h: u16) -> (u32, u32) {
    let mut splits = 0;
    let mut lanes = 0;
    let screen = Rect::new(0, 0, w, h);

    let [header, body, footer] = Col::new().split(screen, [Fixed(3), Weight(1), Fixed(1)]);
    splits += 1;
    lanes += 3;

    let [sidebar, _gutter, detail] = Row::new()
        .spacing(1)
        .split(body, [Fixed(20), Fixed(1), Weight(1)]);
    splits += 1;
    lanes += 3;

    let [_top, _mid, _bot] = Col::new()
        .margin(1)
        .split(detail, [Weight(1), Weight(1), Weight(1)]);
    splits += 1;
    lanes += 3;

    let _buttons = Row::new().spacing(2).split(footer, [Weight(1); 6]);
    splits += 1;
    lanes += 6;

    let [left, mid, right] = Row::new().split(header, [Fixed(10), Weight(1), Fixed(12)]);
    splits += 1;
    lanes += 3;
    let _ = Row::new().split(left, [Weight(1), Weight(1)]);
    splits += 1;
    lanes += 2;
    let _ = Row::new().split(right, [Weight(1), Weight(1)]);
    splits += 1;
    lanes += 2;
    let _ = Col::new().split(mid, [Weight(1)]);
    splits += 1;
    lanes += 1;

    let mut rows = [Rect::default(); LIST_ROWS];
    let n = Col::new().split_into(sidebar, &[Weight(1); LIST_ROWS], &mut rows);
    for row in &rows[..n] {
        let _ = Row::new().split(*row, [Fixed(6), Weight(2), Weight(1), Fixed(8)]);
        splits += 1;
        lanes += 4;
    }
    (splits, lanes)
}

/// The rows the sidebar's cells are wrapped from, for the text report.
///
/// Twenty-four rows and **deliberately not twenty-four copies of one string**: a Latin row, a row
/// that wraps four times, a CJK row with nothing to break on, a row of stacked combining marks, an
/// empty row and a run of family emoji all cost different amounts, and a measurement over one of them
/// would be a measurement of that one.
pub const ROWS: [&str; LIST_ROWS] = [
    "a short line",
    "The quick brown fox jumps over the lazy dog and keeps going for a while yet",
    "漢字漢字漢字漢字漢字漢字漢字漢字漢字漢字漢字漢字",
    "e\u{301}quipe \u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467} and a\u{2764}\u{FE0F}b",
    "status: ready",
    "one two three four five six seven eight nine ten eleven twelve thirteen",
    "mixed 漢字 and latin words together in one line that has to wrap somewhere",
    "a\u{300}\u{301}\u{302}\u{303} stacked combining marks, and then some ordinary text after them",
    "/usr/local/share/doc/some-package/examples/a-rather-long-path-name.txt",
    "",
    "  leading and   internal   spaces  ",
    "supercalifragilisticexpialidocious",
    "warning: the value was truncated because it did not fit in the column provided",
    "42",
    "\u{2764}\u{FE0E} text presentation, and \u{2764}\u{FE0F} emoji presentation, side by side",
    "one\ntwo\nthree",
    "The second paragraph is here to make the corpus look like real data rather than a fixture",
    "漢字 mixed with a very long latin run that will definitely need more than one line",
    "id       name                 state     updated",
    "a b c d e f g h i j k l m n o p q r s t u v w x y z",
    "tab\tseparated\tvalues\there",
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor",
    "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}",
    "done",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// **The shape is thirty-two splits and one hundred and nineteen lanes**, and it is asserted here
    /// as well as at every call site — because the whole reason this module exists is that the counts
    /// were the only thing keeping three copies honest, and one copy still needs them.
    #[test]
    fn the_screen_is_thirty_two_splits_and_a_hundred_and_nineteen_lanes() {
        assert_eq!(screen_frame(300, 80), (32, 119));
    }

    /// The shape does not depend on the terminal size, which is what makes it a fixture rather than a
    /// measurement.
    #[test]
    fn the_shape_is_the_same_at_every_terminal_size() {
        for (w, h) in [(300u16, 80u16), (120, 40), (80, 24), (1, 1), (0, 0)] {
            assert_eq!(screen_frame(w, h), (32, 119), "at {w}x{h}");
        }
    }
}
