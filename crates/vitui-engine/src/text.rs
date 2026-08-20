//! Grapheme segmentation and display width, over a caller's `&str`.
//!
//! Two functions, both public, both iterating **the caller's string** and no application data —
//! exactly as [`View::text`](crate::View::text) does, which is why they do not reopen ADR 0002.
//! See `docs/adr/0005-the-engine-owns-the-caret-and-exports-its-text-tables.md`.
//!
//! The cost of refusing them is the reason they exist: either the runtime ships a second copy of the
//! UCD tables — and the two disagree the day their Unicode versions differ — or every text component
//! is wrong on emoji.

use crate::ucd;

/// The extended grapheme clusters of `s`, each with the columns it occupies.
///
/// A borrowing iterator over slices of the caller's string. Never a `Vec<&str>`, which is what makes
/// zero-allocation text structural rather than disciplined (spec §4).
///
/// ```
/// let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
/// let clusters: Vec<_> = vitui_engine::graphemes(family).collect();
/// assert_eq!(clusters, vec![(family, 2)]);
/// ```
///
/// The caret column of a string is the running sum, and it is the number a text component needs:
///
/// ```
/// let s = "ab\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}cd";
/// let caret: u16 = vitui_engine::graphemes(s).map(|(_, w)| w).sum();
/// assert_eq!(caret, 6, "two, then the family emoji's two, then two");
/// assert_eq!(s.chars().count(), 9, "what a plausible caret would have said instead");
/// ```
pub fn graphemes(s: &str) -> impl Iterator<Item = (&str, u16)> {
    ucd::clusters(s).map(|c| (c, ucd::cluster_width(c)))
}

/// The columns `s` occupies when written to a terminal.
///
/// ```
/// assert_eq!(vitui_engine::width_of("hello"), 5);
/// assert_eq!(vitui_engine::width_of("漢字"), 4);
/// assert_eq!(vitui_engine::width_of("e\u{301}"), 1, "a combining acute is not a column");
/// ```
///
/// A string wider than `u16::MAX` columns saturates rather than overflowing. The return type is
/// spec §12's and no terminal has 65 536 columns, so the answer is already meaningless there — but
/// the clamp is what ADR 0022 asks for everywhere else, and a plain `sum` would panic in a debug
/// build on an input a caller is allowed to hand us.
pub fn width_of(s: &str) -> u16 {
    ucd::clusters(s)
        .map(ucd::cluster_width)
        .fold(0u16, u16::saturating_add)
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_string_wider_than_a_u16_saturates_rather_than_overflowing() {
        let wide: String = std::iter::repeat_n('漢', 40_000).collect();
        assert_eq!(vitui_engine::width_of(&wide), u16::MAX);
    }

    #[test]
    fn a_zwj_family_emoji_is_one_cluster_of_width_two() {
        let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
        assert_eq!(vitui_engine::width_of(family), 2);
        assert_eq!(vitui_engine::graphemes(family).count(), 1);
    }
}
