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
//! Runs, cells, the size, the generation, and three side tables: clusters, extended styles and the
//! OSC 8 URIs those name. All three are keyed by the handle the cell carries and all three are
//! filled at pack time, so the render thread resolves everything inside the packet and holds
//! nothing that points into an engine table.
//!
//! # The rule every future change to this struct has to pass
//!
//! > **Nothing the render thread compares across frames may be derived from a position.**
//!
//! It is the general form of two decisions this file already rests on, and it is written here rather
//! than in a document because this is the struct that would break it.
//!
//! The first is [`Packet::repaint`]. A handle **is** a position — an index into a table — and the
//! mirror is the one reader that compares handles across frames, so the mark-and-compact sweep that
//! renumbers a table has to tell it (spec §3, [`crate::sweep`]). The flag is the whole of that
//! telling, and it costs one full frame on the render thread at a moment when the engine is already
//! doing topology work.
//!
//! The second is the keying of the three side tables. Writing a *position* into a packed cell — an
//! arena offset instead of the handle — makes an unchanged cell pack differently whenever the
//! frame's damage changes shape, and §7 measured that at **284× in bytes** over five steady frames
//! of a page with nothing changing. Both defects are the same sentence read twice.
//!
//! A **content-keyed** packet, deriving each key from the entry's content so a sweep would be
//! invisible to the mirror, was built and refused (§7): it buys a probability rather than a
//! property, its cost lands on the app thread to save the render thread, and identity is simpler to
//! reason about. The flag is what replaces it.
//!
//! # Why the cluster bytes are copied in rather than pointed at
//!
//! The render thread must hold no handle into an engine table (ADR 0011), so at pack time every
//! handle a cell carries is resolved into a side table **the packet owns**, keyed by the handle
//! rather than written into the cell. The cell is copied byte for byte, which is what keeps a packed
//! cell byte-identical to the surface cell — and that is what makes the equality filter exact.
//! Writing an arena offset into the cell instead would make an unchanged cell compare unequal
//! whenever the frame's damage changes shape, measured at **284x in bytes** on five steady frames.
//!
//! # The dedup is a generation stamp per handle, and the generation is the frame's own
//!
//! ADR 0011 names it: a marker per handle, O(1), against the 166 µs the scan version cost — 31 µs
//! where the scan was 166. [`Marks`] is that marker, one slot per handle rather than a hash probe
//! per cell, and it needs no clearing between frames because a slot stamped with an older
//! generation is already invisible.
//!
//! The generation it is stamped with is [`Packet::generation`], the frame's own, and that is one
//! counter doing both jobs rather than a coincidence worth noting: *was this handle already copied*
//! and *which frame is this* are the same question asked of one number. It counts **packs** rather
//! than submissions, because a frame discarded for a resize consumed one and the next pack of the
//! same packet must not collide with it.
//!
//! The vectors grow to the handle tables' high-water mark and never shrink, which is what makes
//! `pack` allocation-free on warm tables at every density (register entry #6). §3's mark-and-compact
//! sweep is what bounds that mark; a table that could grow without one would grow these with it.

use crate::actuate::Actuation;
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
    clusters: Marks<(u32, u32)>,
    /// Every extended style this packet's cells name: the two colours that are not inline on an
    /// extended word, the underline colour and the hyperlink. Keyed by the handle in bits 51..0.
    exts: Marks<ExtStyle>,
    /// The URI each hyperlink names, in the same arena as the clusters.
    links: Marks<(u32, u32)>,
    /// Which pack filled this packet, and the stamp every side table above is marked with.
    generation: u64,
    /// What this frame has to say to the terminal besides its cells: the mouse tracking level and
    /// the caret.
    ///
    /// It rides the packet because **the render thread owns the write direction** (spec §7) and
    /// these are writes. It is already a delta where a delta is what the wire needs — see
    /// [`Actuation`] — so the render thread still holds no state of its own about either setter, and
    /// ADR 0011's *no application state on the render thread* is undisturbed.
    actuation: Actuation,
    /// Set when a sweep renumbered a table since the last packet: **invalidate the mirror.**
    ///
    /// See the position rule in this module's documentation. It is a property of the *packet* rather
    /// than of the sweep because the sweep and the frame are not the same event: the app thread may
    /// sweep several times, or none, between two frames, and what the render thread has to know is
    /// only whether the handles in the cells it is about to be given still mean what the ones it
    /// last recorded meant.
    repaint: bool,
}

impl Packet {
    pub(crate) fn new() -> Packet {
        Packet {
            runs: Vec::new(),
            cells: Vec::new(),
            size: (0, 0),
            arena: String::new(),
            clusters: Marks::new((0, 0)),
            exts: Marks::new(ExtStyle {
                fg: crate::style::Color::DEFAULT,
                bg: crate::style::Color::DEFAULT,
                ul: crate::style::Color::DEFAULT,
                link: LinkId::NONE,
            }),
            links: Marks::new((0, 0)),
            generation: 0,
            actuation: Actuation::default(),
            repaint: false,
        }
    }

    /// Fill this packet from a frame's damaged runs, reusing whatever capacity it already has.
    ///
    /// The runs are passed in rather than rescanned, because `present` has already scanned them to
    /// know what to composite. `clear` keeps the capacity, which is what makes a steady stream of
    /// frames allocate nothing — and the three side tables are not cleared at all, because
    /// `generation` is what makes last frame's entries invisible.
    ///
    /// `generation` must be higher than any this packet has been packed with before; the app thread
    /// counts packs and never reuses one.
    pub(crate) fn pack(
        &mut self,
        runs: &[Run],
        frame: &Surface,
        tables: &Tables,
        repaint: bool,
        actuation: Actuation,
        generation: u64,
    ) {
        debug_assert!(
            generation > self.generation,
            "a reused generation makes last frame's side tables answer for this one"
        );
        self.repaint = repaint;
        self.actuation = actuation;
        self.generation = generation;
        self.runs.clear();
        self.cells.clear();
        self.arena.clear();
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

    /// Fill this packet from a frame's damaged runs, with nothing to say to the terminal besides
    /// them.
    ///
    /// **The test door, and the reason it exists is that there is exactly one production caller of
    /// [`pack`](Packet::pack).** Everything a frame carries goes in through one call so that nothing
    /// can be forgotten there; a test packing a fixture to look at the cells has no mouse and no
    /// caret, and threading a `Actuation::default()` through forty call sites would say nothing.
    #[cfg(test)]
    pub(crate) fn pack_cells(
        &mut self,
        runs: &[Run],
        frame: &Surface,
        tables: &Tables,
        repaint: bool,
        generation: u64,
    ) {
        self.pack(
            runs,
            frame,
            tables,
            repaint,
            Actuation::default(),
            generation,
        );
    }

    /// Copy the bytes of a cluster handle into this packet, once per frame per handle.
    ///
    /// A scalar handle names its own bytes and needs no table, which is the same property that
    /// keeps the interner empty on a screen of CJK — so the map only ever holds the handles that
    /// point into one.
    fn resolve(&mut self, g: GraphemeId, tables: &Tables) {
        let Some(id) = g.cluster_id() else {
            return;
        };
        let at = id as usize;
        if self.clusters.get(at, self.generation).is_some() {
            return;
        }
        let mut scratch = [0u8; 4];
        let Some(text) = tables.interner.render(g, &mut scratch) else {
            return;
        };
        let start = self.arena.len() as u32;
        self.arena.push_str(text);
        self.clusters
            .set(at, self.generation, (start, self.arena.len() as u32));
    }

    /// Copy one extended style, and the URI it names, into this packet.
    ///
    /// An inline style word carries its own colours and needs no table, which is the same property
    /// that keeps the cluster marks holding only the handles that point into one: under 1% of the
    /// cells on a realistic screen are extended.
    ///
    /// The probe is an indexed load and a compare rather than a hash, which is what the memo
    /// `restyle` uses would have bought and cheaper: cells in a run share a style word, so a
    /// one-entry memo would answer most of these — and a stamp per handle answers *all* of them at
    /// the same cost, including the alternating pair a one-entry memo recognises neither of. See
    /// [`Marks`].
    fn resolve_style(&mut self, style: Style, tables: &Tables) {
        let Some(h) = style.ext_handle() else {
            return;
        };
        let at = h as usize;
        if self.exts.get(at, self.generation).is_some() {
            return;
        }
        let Some(e) = tables.exts.get(h) else {
            return;
        };
        self.exts.set(at, self.generation, e);
        let Some(link) = e.link.index() else {
            return;
        };
        if self.links.get(link, self.generation).is_some() {
            return;
        }
        let Some(uri) = tables.links.uri(e.link) else {
            return;
        };
        let start = self.arena.len() as u32;
        self.arena.push_str(uri);
        self.links
            .set(link, self.generation, (start, self.arena.len() as u32));
    }

    /// The bytes a handle names, for a handle that names a cluster. `None` for a scalar, which
    /// names its own.
    pub(crate) fn cluster(&self, g: GraphemeId) -> Option<&str> {
        let (start, end) = self
            .clusters
            .get(g.cluster_id()? as usize, self.generation)?;
        Some(&self.arena[start as usize..end as usize])
    }

    /// The four channels an extended style word names. `None` for an inline handle, and for a
    /// handle no cell in this packet carried.
    ///
    /// Read by [`serial::channels_of`](crate::serial), which resolves all four: two colours as
    /// `SGR 38`/`48`, the underline colour as `SGR 58`/`59`, and the hyperlink as OSC 8 where the
    /// terminal has it.
    pub(crate) fn ext(&self, handle: u32) -> Option<ExtStyle> {
        self.exts.get(handle as usize, self.generation)
    }

    /// The URI a hyperlink names. `None` for [`LinkId::NONE`].
    pub(crate) fn link(&self, id: LinkId) -> Option<&str> {
        let (start, end) = self.links.get(id.index()?, self.generation)?;
        Some(&self.arena[start as usize..end as usize])
    }

    /// Which pack filled this packet.
    ///
    /// A frame number that counts packs rather than submissions, and the stamp the three side tables
    /// are keyed by. Nothing on the render thread compares it across frames — see the position rule
    /// at the top of this module, of which that is an instance — and it is what a diagnostic names a
    /// frame by.
    #[cfg(test)]
    pub(crate) fn generation(&self) -> u64 {
        self.generation
    }

    /// Whether the mirror has to be thrown away before this packet is read.
    ///
    /// True when a sweep renumbered a table since the last packet went out. The serializer's answer
    /// is to mark every mirror row unknown, which is how a full repaint expresses itself without a
    /// separate mode (§8, ADR 0006).
    pub(crate) fn repaint(&self) -> bool {
        self.repaint
    }

    /// The mouse level and the caret this frame carries.
    pub(crate) fn actuation(&self) -> Actuation {
        self.actuation
    }

    /// Whether this packet carries no **cells**.
    ///
    /// It is deliberately not *whether this packet is worth writing*: a frame that damaged nothing
    /// and moved the caret is empty by this measure and still has bytes to send. The serializer is
    /// where the two are put together, and `Screen::present` is where a packet with neither is never
    /// leased at all.
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

/// A side table keyed by the handle, with a generation stamp per slot instead of a clear.
///
/// One slot per handle, stamped with the generation of the pack that filled it, so *is this handle
/// already in this packet* is an indexed load and one compare — ADR 0011's marker, at O(1) against
/// the 166 µs the scan version cost. Nothing is cleared between frames: a slot stamped with an older
/// generation is invisible, which is the whole trick.
///
/// Generation zero is never a valid stamp, so a freshly grown slot is empty without being written.
#[derive(Debug)]
struct Marks<T: Copy> {
    /// Stamp and value interleaved, so a probe is one cache line rather than two.
    slots: Vec<(u64, T)>,
    /// What a grown slot holds until something stamps it. Never read — the stamp is what says so.
    blank: T,
}

impl<T: Copy> Marks<T> {
    fn new(blank: T) -> Marks<T> {
        Marks {
            slots: Vec::new(),
            blank,
        }
    }

    /// What this handle names in this generation, or `None` if this pack has not copied it in.
    fn get(&self, at: usize, generation: u64) -> Option<T> {
        match self.slots.get(at) {
            Some(&(stamp, value)) if stamp == generation => Some(value),
            _ => None,
        }
    }

    /// Record what this handle names, growing to the handle tables' high-water mark.
    ///
    /// The growth is the one allocation in `pack`, it happens once per table high-water mark rather
    /// than once per frame, and `Vec`'s own geometric growth is what keeps a screen of fresh handles
    /// from being quadratic.
    fn set(&mut self, at: usize, generation: u64, value: T) {
        if at >= self.slots.len() {
            self.slots.resize(at + 1, (0, self.blank));
        }
        self.slots[at] = (generation, value);
    }

    /// How many handles this generation copied in. Linear, and for tests only.
    #[cfg(test)]
    fn len(&self, generation: u64) -> usize {
        self.slots.iter().filter(|(s, _)| *s == generation).count()
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
        p.pack_cells(&runs_of(&frame), &frame, frame.tables(), false, 1);
        assert!(p.is_empty());
        assert!(p.cells().is_empty());
    }

    #[test]
    fn a_packet_carries_the_damaged_cells_and_no_others() {
        let mut p = Packet::new();
        let frame = frame_with_two_spans();
        p.pack_cells(&runs_of(&frame), &frame, frame.tables(), false, 1);
        assert_eq!(p.runs().len(), 2);
        assert_eq!(p.cells().len(), 4, "not the 300 cells of the row");
    }

    #[test]
    fn the_cells_are_concatenated_in_run_order() {
        let mut p = Packet::new();
        let frame = frame_with_two_spans();
        p.pack_cells(&runs_of(&frame), &frame, frame.tables(), false, 1);
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
        p.pack_cells(&runs_of(&frame), &frame, frame.tables(), false, 1);
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
        p.pack_cells(&runs_of(&frame), &frame, frame.tables(), false, 1);
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
        p.pack_cells(&runs_of(&frame), &frame, frame.tables(), false, 1);
        assert_eq!(p.cells().len(), 8);
        assert_eq!(p.exts.len(p.generation), 1);
        assert_eq!(p.links.len(p.generation), 1);
    }

    #[test]
    fn a_packet_of_inline_cells_carries_no_extended_tables() {
        let mut p = Packet::new();
        let frame = frame_with_two_spans();
        p.pack_cells(&runs_of(&frame), &frame, frame.tables(), false, 1);
        assert_eq!(
            p.exts.len(p.generation),
            0,
            "under 1% of cells are extended, and these are not"
        );
        assert_eq!(p.links.len(p.generation), 0);
        assert_eq!(p.ext(0), None);
    }

    #[test]
    fn packing_twice_replaces_rather_than_appends() {
        let mut p = Packet::new();
        let frame = frame_with_two_spans();
        p.pack_cells(&runs_of(&frame), &frame, frame.tables(), false, 1);
        let mut second = Surface::new(300, 4);
        second.root().fill(Rect::new(0, 1, 3, 1), "#", Style::new());
        p.pack_cells(&runs_of(&second), &second, second.tables(), false, 2);
        assert_eq!(p.runs().len(), 1);
        assert_eq!(p.cells().len(), 3);
    }

    #[test]
    fn packing_a_plain_frame_after_an_extended_one_clears_the_side_tables() {
        let mut p = Packet::new();
        let extended = hyperlinked_row();
        p.pack_cells(&runs_of(&extended), &extended, extended.tables(), false, 1);
        assert_eq!(p.exts.len(p.generation), 1);
        let plain = frame_with_two_spans();
        p.pack_cells(&runs_of(&plain), &plain, plain.tables(), false, 2);
        assert_eq!(
            p.exts.len(p.generation),
            0,
            "a stale entry would outlive the cell that named it"
        );
        assert_eq!(p.links.len(p.generation), 0);
    }
}
