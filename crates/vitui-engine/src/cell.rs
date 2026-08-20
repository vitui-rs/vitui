//! The cell: sixteen bytes, a grapheme handle and a style word.

use crate::style::Style;

/// What a cell holds instead of a character.
///
/// A cell holds an interned extended-grapheme-cluster handle, never a `char` (spec §3, ADR 0023).
/// The encoding leaves single scalars as their own handle, which is what makes the rule free on the
/// common path — a cell holding `a` *is* `0x61`, with no table and no hash.
///
/// ```text
/// 0x0000_0000..=0x0010_FFFF   a Unicode scalar value, narrow
/// 0x0011_0000..=0x7FFF_FFFE   a cluster id in the intern table, narrow
/// 0x7FFF_FFFF                 EMPTY - a non-opaque content layer's "skip this cell" sentinel
/// bit 31 set                  a cluster id, but the head of a double-width pair
/// 0xFFFF_FFFF                 CONTINUATION - the second half of a double-width pair
/// ```
///
/// Ticket 06 is what mints anything above `0x0010_FFFF`. Until it lands the engine writes single
/// scalars and nothing else, so no `CONTINUATION` is ever produced and §3's pairing invariant holds
/// vacuously rather than by enforcement.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct GraphemeId(u32);

impl GraphemeId {
    /// The blank a cell is born as, and what a repaired half of a pair is blanked to.
    pub(crate) const SPACE: GraphemeId = GraphemeId(b' ' as u32);

    /// A non-opaque content layer's "skip this cell" sentinel. The one value the encoding leaves
    /// free between the narrow cluster ids and the wide ones.
    pub(crate) const EMPTY: GraphemeId = GraphemeId(0x7FFF_FFFF);

    /// The second half of a double-width pair. Ticket 06 is what produces one.
    pub(crate) const CONTINUATION: GraphemeId = GraphemeId(0xFFFF_FFFF);

    /// The handle of a single Unicode scalar, which is the scalar itself.
    pub(crate) const fn scalar(c: char) -> GraphemeId {
        GraphemeId(c as u32)
    }

    /// The scalar this handle *is*, or `None` when it points into a table or is a sentinel.
    pub(crate) const fn as_scalar(self) -> Option<char> {
        char::from_u32(self.0)
    }

    pub(crate) const fn is_continuation(self) -> bool {
        self.0 == GraphemeId::CONTINUATION.0
    }

    pub(crate) const fn is_empty(self) -> bool {
        self.0 == GraphemeId::EMPTY.0
    }
}

/// One addressable position in the grid: what is drawn there, and how it is styled.
///
/// # Layout
///
/// Sixteen bytes, aligned to four, with no padding hole — asserted, because both halves of that
/// sentence are load-bearing. Sixteen bytes is what puts exactly four cells in a cache line, which
/// is why spec §3 says not to pad rows. `_reserved` is not slack to be reclaimed later: without it
/// `#[repr(C)]` over a handle and a style word leaves four bytes of hole, and a hole is four bytes
/// of every cell that no test can see.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Cell {
    pub(crate) grapheme: GraphemeId,
    pub(crate) style: Style,
    _reserved: u32,
}

impl Cell {
    /// A space in the terminal's own colours: what an untouched cell holds.
    pub(crate) const BLANK: Cell = Cell::new(GraphemeId::SPACE, Style::DEFAULT);

    pub(crate) const fn new(grapheme: GraphemeId, style: Style) -> Cell {
        Cell {
            grapheme,
            style,
            _reserved: 0,
        }
    }
}

impl Default for Cell {
    fn default() -> Cell {
        Cell::BLANK
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Color;

    #[test]
    fn a_cell_is_sixteen_bytes_aligned_to_four() {
        assert_eq!(size_of::<Cell>(), 16);
        assert_eq!(align_of::<Cell>(), 4);
    }

    #[test]
    fn a_cell_has_no_padding_hole() {
        // Three fields' worth of bytes adding up to the whole: 4 + 8 + 4. A `#[repr(C)]` struct
        // of a handle and a style word alone would be sixteen bytes too, with four of them a hole.
        assert_eq!(
            size_of::<GraphemeId>() + size_of::<Style>() + size_of::<u32>(),
            size_of::<Cell>()
        );
    }

    #[test]
    fn a_row_of_cells_puts_four_in_a_cache_line() {
        // Spec §3: this is why rows are not padded to a line boundary.
        assert_eq!(64 / size_of::<Cell>(), 4);
    }

    #[test]
    fn a_scalar_is_its_own_handle() {
        assert_eq!(GraphemeId::scalar('a'), GraphemeId(0x61));
        assert_eq!(GraphemeId::scalar('a').as_scalar(), Some('a'));
        assert_eq!(
            GraphemeId::scalar('\u{10FFFF}').as_scalar(),
            Some('\u{10FFFF}')
        );
    }

    #[test]
    fn the_two_sentinels_are_not_scalars() {
        assert_eq!(GraphemeId::EMPTY.as_scalar(), None);
        assert_eq!(GraphemeId::CONTINUATION.as_scalar(), None);
        assert!(GraphemeId::EMPTY.is_empty());
        assert!(GraphemeId::CONTINUATION.is_continuation());
    }

    #[test]
    fn empty_is_the_one_value_the_narrow_range_leaves_free() {
        // The narrow cluster ids run up to 0x7FFF_FFFE, and bit 31 marks a wide head.
        assert_eq!(GraphemeId::EMPTY.0, 0x7FFF_FFFF);
        assert_eq!(GraphemeId::EMPTY.0 & (1 << 31), 0);
    }

    #[test]
    fn a_blank_cell_is_a_space_in_the_terminals_own_colours() {
        assert_eq!(Cell::BLANK.grapheme, GraphemeId::SPACE);
        assert_eq!(Cell::BLANK.style, Style::DEFAULT);
        assert_eq!(Cell::default(), Cell::BLANK);
    }

    #[test]
    fn two_cells_differing_only_in_style_are_not_equal() {
        let a = Cell::new(GraphemeId::scalar('x'), Style::DEFAULT);
        let b = Cell::new(GraphemeId::scalar('x'), Style::new().fg(Color::indexed(1)));
        assert_ne!(a, b);
    }
}
