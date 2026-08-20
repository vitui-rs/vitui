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
//! Runs, cells, the size, and three side tables: clusters, extended styles and the OSC 8 URIs those
//! name. All three are keyed by the handle the cell carries and all three are filled at pack time,
//! so the render thread resolves everything inside the packet and holds nothing that points into an
//! engine table. `repaint` and `generation` arrive with the sweep (ticket 08) and the mailbox
//! (ticket 18).
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
use crate::exts::{ExtStyle, LinkId};
use crate::style::Style;
use crate::surface::Surface;
use crate::tables::Tables;

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
    /// Every extended style this packet's cells name: the two colours that are not inline on an
    /// extended word, the underline colour and the hyperlink. Keyed by the handle in bits 51..0.
    exts: HashMap<u32, ExtStyle>,
    /// The URI each hyperlink names, in the same arena as the clusters.
    links: HashMap<LinkId, (u32, u32)>,
}

impl Packet {
    pub(crate) fn new() -> Packet {
        Packet {
            runs: Vec::new(),
            cells: Vec::new(),
            size: (0, 0),
            arena: String::new(),
            clusters: HashMap::new(),
            exts: HashMap::new(),
            links: HashMap::new(),
        }
    }

    /// Fill this packet from a frame's damaged runs, reusing whatever capacity it already has.
    ///
    /// The runs are passed in rather than rescanned, because `present` has already scanned them to
    /// know what to composite. `clear` keeps the capacity, which is what makes a steady stream of
    /// frames allocate nothing.
    pub(crate) fn pack(&mut self, runs: &[Run], frame: &Surface, tables: &Tables) {
        self.runs.clear();
        self.cells.clear();
        self.arena.clear();
        self.clusters.clear();
        self.exts.clear();
        self.links.clear();
        self.size = frame.size();
        self.runs.extend_from_slice(runs);
        for r in runs {
            let cells = &frame.row(r.y)[r.lo as usize..=r.hi as usize];
            self.cells.extend_from_slice(cells);
            for cell in cells {
                self.resolve(cell.grapheme, tables);
                self.resolve_style(cell.style, tables);
            }
        }
    }

    /// Copy the bytes of a cluster handle into this packet, once per frame per handle.
    ///
    /// A scalar handle names its own bytes and needs no table, which is the same property that
    /// keeps the interner empty on a screen of CJK — so the map only ever holds the handles that
    /// point into one.
    fn resolve(&mut self, g: GraphemeId, tables: &Tables) {
        if g.cluster_id().is_none() || self.clusters.contains_key(&g) {
            return;
        }
        let mut scratch = [0u8; 4];
        let Some(text) = tables.interner.render(g, &mut scratch) else {
            return;
        };
        let start = self.arena.len() as u32;
        self.arena.push_str(text);
        self.clusters.insert(g, (start, self.arena.len() as u32));
    }

    /// Copy one extended style, and the URI it names, into this packet.
    ///
    /// An inline style word carries its own colours and needs no table, which is the same property
    /// that keeps the cluster map holding only the handles that point into one: under 1% of the
    /// cells on a realistic screen are extended.
    ///
    /// **The memo belongs to ticket 18**, not here. Cells in a run share a style word, so one entry
    /// on the previous word would turn this per-cell hash probe into a per-distinct-style one — the
    /// same trick `restyle` uses — and ADR 0011 names something sharper still, a generation-stamped
    /// marker per handle at O(1) against the 166 µs the scan version cost. Both numbers were taken
    /// at pack time on a full screen, so they belong with the ticket that measures pack. Recorded
    /// here so it is a deferral rather than an omission.
    fn resolve_style(&mut self, style: Style, tables: &Tables) {
        let Some(h) = style.ext_handle() else {
            return;
        };
        if self.exts.contains_key(&h) {
            return;
        }
        let Some(e) = tables.exts.get(h) else {
            return;
        };
        self.exts.insert(h, e);
        if e.link.is_none() || self.links.contains_key(&e.link) {
            return;
        }
        let Some(uri) = tables.links.uri(e.link) else {
            return;
        };
        let start = self.arena.len() as u32;
        self.arena.push_str(uri);
        self.links.insert(e.link, (start, self.arena.len() as u32));
    }

    /// The bytes a handle names, for a handle that names a cluster. `None` for a scalar, which
    /// names its own.
    pub(crate) fn cluster(&self, g: GraphemeId) -> Option<&str> {
        let (start, end) = *self.clusters.get(&g)?;
        Some(&self.arena[start as usize..end as usize])
    }

    /// The four channels an extended style word names. `None` for an inline handle, and for a
    /// handle no cell in this packet carried.
    ///
    /// Read by [`serial::colors_of`](crate::serial) for the two colours it already emits; ticket 13
    /// is what reads the other two, as SGR 58/59 and OSC 8.
    pub(crate) fn ext(&self, handle: u32) -> Option<ExtStyle> {
        self.exts.get(&handle).copied()
    }

    /// The URI a hyperlink names. `None` for [`LinkId::NONE`].
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "ticket 13 emits the OSC 8 this resolves and is the first reader"
        )
    )]
    pub(crate) fn link(&self, id: LinkId) -> Option<&str> {
        let (start, end) = *self.links.get(&id)?;
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
        p.pack(&runs_of(&frame), &frame, frame.tables());
        assert!(p.is_empty());
        assert!(p.cells().is_empty());
    }

    #[test]
    fn a_packet_carries_the_damaged_cells_and_no_others() {
        let mut p = Packet::new();
        let frame = frame_with_two_spans();
        p.pack(&runs_of(&frame), &frame, frame.tables());
        assert_eq!(p.runs().len(), 2);
        assert_eq!(p.cells().len(), 4, "not the 300 cells of the row");
    }

    #[test]
    fn the_cells_are_concatenated_in_run_order() {
        let mut p = Packet::new();
        let frame = frame_with_two_spans();
        p.pack(&runs_of(&frame), &frame, frame.tables());
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
        p.pack(&runs_of(&frame), &frame, frame.tables());
        assert_eq!(p.size(), (300, 80));
    }

    /// A surface whose whole row carries one hyperlink and one underline colour.
    fn hyperlinked_row() -> Surface {
        let mut s = Surface::new(8, 1);
        let link = s.tables_mut().link("https://example.com/");
        s.root().text(0, 0, "abcdefgh", Style::new());
        s.root().restyle(
            crate::geom::Rect::new(0, 0, 8, 1),
            &crate::restyle::Restyle {
                link: Some(link),
                ul: Some(crate::style::Color::rgb(1, 2, 3)),
                ..Default::default()
            },
        );
        s
    }

    #[test]
    fn the_packet_carries_the_extended_style_its_cells_name() {
        // ADR 0011: the render thread holds no handle into an engine table, so every handle a cell
        // carries is resolved at pack time into a side table the packet owns — keyed by the handle,
        // never written into the cell.
        let mut p = Packet::new();
        let frame = hyperlinked_row();
        p.pack(&runs_of(&frame), &frame, frame.tables());
        let handle = p.cells()[0]
            .style
            .ext_handle()
            .expect("the row is extended");
        let e = p.ext(handle).expect("the packet resolved it");
        assert_eq!(e.ul, crate::style::Color::rgb(1, 2, 3));
        assert_eq!(p.link(e.link), Some("https://example.com/"));
    }

    #[test]
    fn one_extended_style_is_copied_once_however_many_cells_carry_it() {
        let mut p = Packet::new();
        let frame = hyperlinked_row();
        p.pack(&runs_of(&frame), &frame, frame.tables());
        assert_eq!(p.cells().len(), 8);
        assert_eq!(p.exts.len(), 1);
        assert_eq!(p.links.len(), 1);
    }

    #[test]
    fn a_packet_of_inline_cells_carries_no_extended_tables() {
        let mut p = Packet::new();
        let frame = frame_with_two_spans();
        p.pack(&runs_of(&frame), &frame, frame.tables());
        assert!(
            p.exts.is_empty(),
            "under 1% of cells are extended, and these are not"
        );
        assert!(p.links.is_empty());
        assert_eq!(p.ext(0), None);
    }

    #[test]
    fn packing_twice_replaces_rather_than_appends() {
        let mut p = Packet::new();
        let frame = frame_with_two_spans();
        p.pack(&runs_of(&frame), &frame, frame.tables());
        let mut second = Surface::new(300, 4);
        second.root().fill(Rect::new(0, 1, 3, 1), "#", Style::new());
        p.pack(&runs_of(&second), &second, second.tables());
        assert_eq!(p.runs().len(), 1);
        assert_eq!(p.cells().len(), 3);
    }

    #[test]
    fn packing_a_plain_frame_after_an_extended_one_clears_the_side_tables() {
        let mut p = Packet::new();
        let extended = hyperlinked_row();
        p.pack(&runs_of(&extended), &extended, extended.tables());
        assert_eq!(p.exts.len(), 1);
        let plain = frame_with_two_spans();
        p.pack(&runs_of(&plain), &plain, plain.tables());
        assert!(
            p.exts.is_empty(),
            "a stale entry would outlive the cell that named it"
        );
        assert!(p.links.is_empty());
    }
}
