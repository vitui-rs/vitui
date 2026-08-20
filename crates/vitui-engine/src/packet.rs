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
//! Runs, cells, the size, and the **cluster side table**. The tables for extended styles and links
//! arrive with the handles they resolve, at ticket 07.
//!
//! # Why the cluster bytes are copied in rather than pointed at
//!
//! The render thread must hold no handle into an engine table (ADR 0011), so at pack time every
//! handle a cell carries is resolved into a side table **the packet owns**, keyed by the handle
//! rather than written into the cell. The cell is copied byte for byte, which is what keeps a packed
//! cell byte-identical to the surface cell — and that is what makes ticket 14's equality filter
//! exact. Writing an arena offset into the cell instead would make an unchanged cell compare unequal
//! whenever the frame's damage changes shape, measured at **284x in bytes** on five steady frames.
//!
//! The dedup is a `HashMap` reused across frames, which is not the shape ADR 0011 measured: it names
//! a generation-stamped marker per handle, O(1) against the 166 µs the scan version cost. The marker
//! needs a slot per interner entry and a generation counter, and the number that justified it was
//! taken at pack time on a full screen — so it belongs with the ticket that measures pack, not with
//! the one that first puts a cluster in a cell. Recorded here so it is a deferral rather than an
//! omission.

use std::collections::HashMap;

use crate::cell::{Cell, GraphemeId};
use crate::damage::Run;
use crate::intern::Interner;
use crate::surface::Surface;

/// A snapshot in flight: the damaged runs of a frame together with the cells inside them.
#[derive(Debug)]
pub(crate) struct Packet {
    runs: Vec<Run>,
    cells: Vec<Cell>,
    /// So a stale-size packet can be refused on its own.
    size: (u16, u16),
    /// Every cluster this packet's cells name, concatenated. Cleared and refilled each frame, so a
    /// steady stream of frames allocates nothing once it has reached its high-water mark.
    arena: String,
    /// Handle to `(start, end)` in the arena. Keyed by the handle, never by position.
    clusters: HashMap<GraphemeId, (u32, u32)>,
}

impl Packet {
    pub(crate) fn new() -> Packet {
        Packet {
            runs: Vec::new(),
            cells: Vec::new(),
            size: (0, 0),
            arena: String::new(),
            clusters: HashMap::new(),
        }
    }

    /// Fill this packet from a frame's damaged runs, reusing whatever capacity it already has.
    ///
    /// The runs are passed in rather than rescanned, because `present` has already scanned them to
    /// know what to composite. `clear` keeps the capacity, which is what makes a steady stream of
    /// frames allocate nothing.
    pub(crate) fn pack(&mut self, runs: &[Run], frame: &Surface, interner: &Interner) {
        self.runs.clear();
        self.cells.clear();
        self.arena.clear();
        self.clusters.clear();
        self.size = frame.size();
        self.runs.extend_from_slice(runs);
        for r in runs {
            let cells = &frame.row(r.y)[r.lo as usize..=r.hi as usize];
            self.cells.extend_from_slice(cells);
            for cell in cells {
                self.resolve(cell.grapheme, interner);
            }
        }
    }

    /// Copy the bytes of a cluster handle into this packet, once per frame per handle.
    ///
    /// A scalar handle names its own bytes and needs no table, which is the same property that
    /// keeps the interner empty on a screen of CJK — so the map only ever holds the handles that
    /// point into one.
    fn resolve(&mut self, g: GraphemeId, interner: &Interner) {
        if g.cluster_id().is_none() || self.clusters.contains_key(&g) {
            return;
        }
        let mut scratch = [0u8; 4];
        let Some(text) = interner.render(g, &mut scratch) else {
            return;
        };
        let start = self.arena.len() as u32;
        self.arena.push_str(text);
        self.clusters.insert(g, (start, self.arena.len() as u32));
    }

    /// The bytes a handle names, for a handle that names a cluster. `None` for a scalar, which
    /// names its own.
    pub(crate) fn cluster(&self, g: GraphemeId) -> Option<&str> {
        let (start, end) = *self.clusters.get(&g)?;
        Some(&self.arena[start as usize..end as usize])
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
        p.pack(&runs_of(&frame), &frame, frame.interner());
        assert!(p.is_empty());
        assert!(p.cells().is_empty());
    }

    #[test]
    fn a_packet_carries_the_damaged_cells_and_no_others() {
        let mut p = Packet::new();
        let frame = frame_with_two_spans();
        p.pack(&runs_of(&frame), &frame, frame.interner());
        assert_eq!(p.runs().len(), 2);
        assert_eq!(p.cells().len(), 4, "not the 300 cells of the row");
    }

    #[test]
    fn the_cells_are_concatenated_in_run_order() {
        let mut p = Packet::new();
        let frame = frame_with_two_spans();
        p.pack(&runs_of(&frame), &frame, frame.interner());
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
        p.pack(&runs_of(&frame), &frame, frame.interner());
        assert_eq!(p.size(), (300, 80));
    }

    #[test]
    fn packing_twice_replaces_rather_than_appends() {
        let mut p = Packet::new();
        let frame = frame_with_two_spans();
        p.pack(&runs_of(&frame), &frame, frame.interner());
        let mut second = Surface::new(300, 4);
        second.root().fill(Rect::new(0, 1, 3, 1), "#", Style::new());
        p.pack(&runs_of(&second), &second, second.interner());
        assert_eq!(p.runs().len(), 1);
        assert_eq!(p.cells().len(), 3);
    }
}
