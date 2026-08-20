//! The packet: what crosses from the app thread to the render thread.
//!
//! Damaged runs, and the cells inside them, and nothing else — never the grid. Spec §7 measured
//! both: on contiguous full-screen damage the two are identical, as they must be, but on the sparse
//! sub-cell chart the packet walks 9.4 KB contiguously where a grid indexed by runs jumps a row
//! stride per run across 384 KB, and that is 4.88x. Packing is proportional to damage — 4.76 ns for
//! a caret against 6.24 us to memcpy the grid regardless of what changed.
//!
//! # Scope
//!
//! Runs, cells and the size. The side tables that make a packet self-contained — clusters, extended
//! styles, links — arrive with the handles they resolve, at tickets 06 and 07.

use crate::cell::Cell;
use crate::damage::Run;
use crate::surface::Surface;

/// A snapshot in flight: the damaged runs of a frame together with the cells inside them.
#[derive(Debug)]
pub(crate) struct Packet {
    runs: Vec<Run>,
    cells: Vec<Cell>,
    /// So a stale-size packet can be refused on its own.
    size: (u16, u16),
}

impl Packet {
    pub(crate) fn new() -> Packet {
        Packet {
            runs: Vec::new(),
            cells: Vec::new(),
            size: (0, 0),
        }
    }

    /// Fill this packet from a frame's damaged runs, reusing whatever capacity it already has.
    ///
    /// The runs are passed in rather than rescanned, because `present` has already scanned them to
    /// know what to composite. `clear` keeps the capacity, which is what makes a steady stream of
    /// frames allocate nothing.
    pub(crate) fn pack(&mut self, runs: &[Run], frame: &Surface) {
        self.runs.clear();
        self.cells.clear();
        self.size = frame.size();
        self.runs.extend_from_slice(runs);
        for r in runs {
            self.cells
                .extend_from_slice(&frame.row(r.y)[r.lo as usize..=r.hi as usize]);
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.runs.is_empty()
    }

    pub(crate) fn runs(&self) -> &[Run] {
        &self.runs
    }

    pub(crate) fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub(crate) fn size(&self) -> (u16, u16) {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Rect;
    use crate::style::Style;

    fn runs_of(s: &Surface) -> Vec<Run> {
        let mut out = Vec::new();
        s.damage().for_each_run(|r| out.push(r));
        out
    }

    fn frame_with_two_spans() -> Surface {
        let mut s = Surface::new(300, 4);
        s.root().text(0, 0, "ab", Style::new());
        s.root().text(200, 0, "cd", Style::new());
        s
    }

    #[test]
    fn a_packet_from_an_undamaged_frame_is_empty() {
        let mut p = Packet::new();
        let frame = Surface::new(8, 2);
        p.pack(&runs_of(&frame), &frame);
        assert!(p.is_empty());
        assert!(p.cells().is_empty());
    }

    #[test]
    fn a_packet_carries_the_damaged_cells_and_no_others() {
        let mut p = Packet::new();
        let frame = frame_with_two_spans();
        p.pack(&runs_of(&frame), &frame);
        assert_eq!(p.runs().len(), 2);
        assert_eq!(p.cells().len(), 4, "not the 300 cells of the row");
    }

    #[test]
    fn the_cells_are_concatenated_in_run_order() {
        let mut p = Packet::new();
        let frame = frame_with_two_spans();
        p.pack(&runs_of(&frame), &frame);
        let glyphs: String = p
            .cells()
            .iter()
            .map(|c| c.grapheme.as_scalar().unwrap())
            .collect();
        assert_eq!(glyphs, "abcd");
    }

    #[test]
    fn the_packet_carries_the_size_it_was_packed_at() {
        let mut p = Packet::new();
        let frame = Surface::new(300, 80);
        p.pack(&runs_of(&frame), &frame);
        assert_eq!(p.size(), (300, 80));
    }

    #[test]
    fn packing_twice_replaces_rather_than_appends() {
        let mut p = Packet::new();
        let frame = frame_with_two_spans();
        p.pack(&runs_of(&frame), &frame);
        let mut second = Surface::new(300, 4);
        second.root().fill(Rect::new(0, 1, 3, 1), "#", Style::new());
        p.pack(&runs_of(&second), &second);
        assert_eq!(p.runs().len(), 1);
        assert_eq!(p.cells().len(), 3);
    }
}
