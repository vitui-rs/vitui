//! The cell: sixteen bytes, a grapheme handle and a style word.

use crate::style::Style;

/// What a cell holds instead of a character.
///
/// A cell holds an interned extended-grapheme-cluster handle, never a `char`.
/// The encoding leaves single scalars as their own handle, which is what makes the rule free on the
/// common path — a cell holding `a` *is* `0x61`, with no table and no hash.
///
/// **Bit 31 is a flag over the low 31 bits, not a third range**:
///
/// ```text
/// bit 31         clear: one column.  set: two columns, and a CONTINUATION follows.
/// bits 0..=30    0x0000_0000..=0x0010_FFFF   a Unicode scalar value
///                0x0011_0000..=0x7FFF_FFFE   a cluster id in the intern table
///                0x7FFF_FFFF                 EMPTY - a non-opaque content layer's "skip
///                                            this cell" sentinel
///
/// 0xFFFF_FFFF    CONTINUATION - the second half of a double-width pair, which is exactly
///                `bit 31 | EMPTY` and needs no range of its own
/// ```
///
/// Read as three ranges, a wide **scalar** has no representation: `漢` is U+6F22 and is two columns,
/// so it would have to be interned to be a wide head — and then a screen of CJK would intern 12 000
/// times, against the sentence above and against the measurement that a full screen of CJK is
/// *cheaper* than one of Latin. A wide head and its narrow twin are one bit apart.
#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(crate) struct GraphemeId(u32);

impl GraphemeId {
    /// The blank a cell is born as, and what a repaired half of a pair is blanked to.
    pub(crate) const SPACE: GraphemeId = GraphemeId(b' ' as u32);

    /// A non-opaque content layer's "skip this cell" sentinel. The one value the encoding leaves
    /// free between the narrow cluster ids and the wide ones.
    pub(crate) const EMPTY: GraphemeId = GraphemeId(0x7FFF_FFFF);

    /// The second half of a double-width pair, produced by the interner.
    pub(crate) const CONTINUATION: GraphemeId = GraphemeId(0xFFFF_FFFF);

    /// The wide flag, and the first cluster id after the scalars.
    const WIDE: u32 = 1 << 31;
    pub(crate) const FIRST_CLUSTER: u32 = 0x0011_0000;
    /// One past the last cluster id: `EMPTY`'s payload is not one.
    pub(crate) const LAST_CLUSTER: u32 = 0x7FFF_FFFE;

    /// The handle of a single narrow Unicode scalar, which is the scalar itself.
    pub(crate) const fn scalar(c: char) -> GraphemeId {
        GraphemeId(c as u32)
    }

    /// The handle of a single **wide** Unicode scalar: the same scalar, one bit away.
    pub(crate) const fn wide_scalar(c: char) -> GraphemeId {
        GraphemeId(GraphemeId::WIDE | c as u32)
    }

    /// The handle of an interned cluster. `id` is the interner's own index.
    pub(crate) const fn cluster(id: u32, wide: bool) -> GraphemeId {
        let payload = GraphemeId::FIRST_CLUSTER + id;
        GraphemeId(if wide {
            GraphemeId::WIDE | payload
        } else {
            payload
        })
    }

    /// The low 31 bits: what this handle names, with the column count stripped off.
    pub(crate) const fn payload(self) -> u32 {
        self.0 & !GraphemeId::WIDE
    }

    /// The interner index this handle names, or `None` when it names a scalar or a sentinel.
    pub(crate) const fn cluster_id(self) -> Option<u32> {
        let payload = self.payload();
        if payload >= GraphemeId::FIRST_CLUSTER && payload <= GraphemeId::LAST_CLUSTER {
            Some(payload - GraphemeId::FIRST_CLUSTER)
        } else {
            None
        }
    }

    /// The scalar this handle *is*, or `None` when it points into a table or is a sentinel.
    ///
    /// Masks bit 31 first, because a wide scalar is a scalar.
    pub(crate) const fn as_scalar(self) -> Option<char> {
        char::from_u32(self.payload())
    }

    /// The columns this handle occupies: one, or two with a `CONTINUATION` after it.
    ///
    /// A `CONTINUATION` is **one**, not two, and it carries the same guard as
    /// [`is_wide_head`](GraphemeId::is_wide_head) for the same reason: it has the wide bit set
    /// because it is the second half of a pair, not because it occupies two columns of its own.
    /// Every caller today filters continuations out before asking — the serializer does it one line
    /// above the question — so this is latent rather than broken, and a guard on one of two
    /// functions that read the same bit is a trap left for whoever adds the third.
    pub(crate) const fn columns(self) -> u16 {
        if self.is_wide_head() { 2 } else { 1 }
    }

    /// Whether this handle is the head of a double-width pair. `CONTINUATION` is not one, which is
    /// the one case the bare flag test gets wrong.
    pub(crate) const fn is_wide_head(self) -> bool {
        self.0 & GraphemeId::WIDE != 0 && !self.is_continuation()
    }

    pub(crate) const fn is_continuation(self) -> bool {
        self.0 == GraphemeId::CONTINUATION.0
    }

    pub(crate) const fn is_empty(self) -> bool {
        self.0 == GraphemeId::EMPTY.0
    }

    /// The whole handle, wide flag included.
    ///
    /// A key, not a number to do arithmetic on: [`crate::golden`] indexes its legend by it, because
    /// a wide head and its narrow twin are one bit apart and must not share a legend entry.
    #[cfg(test)]
    pub(crate) const fn bits(self) -> u32 {
        self.0
    }
}

/// One addressable position in the grid: what is drawn there, and how it is styled.
///
/// # Layout
///
/// Sixteen bytes, aligned to four, with no padding hole — asserted, because both halves of that
/// sentence are load-bearing. Sixteen bytes is what puts exactly four cells in a cache line, which
/// is why rows are not padded. `_reserved` is not slack to be reclaimed later: without it
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

    /// What the **mirror** holds where it does not know what the terminal is showing.
    ///
    /// `EMPTY` is *there is nothing here*, which is exactly the statement — and it is a value **no
    /// composited frame can hold**, because the frame is opaque and its ground is a blank, while a
    /// non-opaque layer's `EMPTY` cells are skipped rather than copied. That is what makes
    /// this sound rather than convenient: the equality filter's danger is the false *equality*, and a
    /// sentinel no frame cell can equal makes one unreachable with no flag to consult and no branch
    /// to forget. See [`crate::serial::Mirror`].
    pub(crate) const UNKNOWN: Cell = Cell::new(GraphemeId::EMPTY, Style::DEFAULT);

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
        // This is why rows are not padded to a line boundary.
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
    fn a_wide_scalar_is_still_its_own_handle_one_bit_away_from_the_narrow_one() {
        // Bit 31 is a flag over the low 31 bits, not a third range. Read as a range,
        // every cell of a screen of CJK would have to be interned.
        let narrow = GraphemeId::scalar('a');
        let wide = GraphemeId::wide_scalar('漢');
        assert_eq!(wide.0, 0x8000_0000 | 0x6F22);
        assert_eq!(wide.as_scalar(), Some('漢'));
        assert_eq!(wide.columns(), 2);
        assert_eq!(narrow.columns(), 1);
        assert!(wide.is_wide_head());
        assert!(!narrow.is_wide_head());
    }

    #[test]
    fn a_continuation_occupies_one_column_not_two() {
        assert_eq!(GraphemeId::CONTINUATION.columns(), 1);
        assert_eq!(GraphemeId::wide_scalar('漢').columns(), 2);
        assert_eq!(GraphemeId::SPACE.columns(), 1);
    }

    #[test]
    fn continuation_is_exactly_the_wide_flag_over_empty() {
        assert_eq!(
            GraphemeId::CONTINUATION.0,
            0x8000_0000 | GraphemeId::EMPTY.0
        );
        assert!(
            !GraphemeId::CONTINUATION.is_wide_head(),
            "the second half of a pair is not a head"
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
