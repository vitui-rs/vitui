//! Surfaces: a rectangular grid of cells, and the damage that says which of them changed.

use crate::cell::{Cell, GraphemeId};
use crate::damage::RowBits;
use crate::geom::Rect;
use crate::tables::Tables;
use crate::view::View;

/// A rectangular grid of cells that can be drawn into.
///
/// The engine's central primitive. A layer usually owns one; a caller constructs one directly only
/// to draw somewhere off-screen. What a surface holds is cells and damage: the handle tables belong
/// to the layer stack, and that is what lets one layer be composited into another as a plain copy.
/// The `tables` field below is the standalone door's exception and is empty for every
/// surface the stack minted.
///
/// # Threading
///
/// `Surface` is [`Send`], and holds no `Arc`, no interior mutability and no lifetime. That is
/// deliberate rather than incidental: drawing a heavy off-screen surface on a worker and donating
/// it to the layer stack is legitimate, and it is what lets the app thread prepare frame N+1 while
/// the render thread paints frame N.
///
/// Each half of that sentence is checked by the compiler rather than by reading: `Sync` fails if a
/// field has interior mutability, and `'static` fails if the type gains a lifetime.
///
/// ```
/// fn assert_send_sync_static<T: Send + Sync + 'static>() {}
/// assert_send_sync_static::<vitui_engine::Surface>();
/// ```
///
/// # There is no `set_eq_filter`, and the reason is `clear`-then-draw
///
/// A write-time equality filter — *do not mark damage for a write whose value is already there* —
/// is free in time and was measured and refused. **It is defeated by the idiom every
/// component library uses:** immediate mode blanks a region before it draws into it, so the filter
/// sees blank-over-text and then text-over-blank, and both of those are changes. It strips nothing on
/// the frames that matter.
///
/// The same idea one layer down, at pack time against the mirror, is worth between 1.4x and 37.7x on
/// spec §14's twelve — see `crate::serial`. That is not the same optimisation implemented better;
/// it is a different one, because by then the two writes have already collapsed into the one cell the
/// frame ends up holding.
///
/// # `clear`-then-draw is load-bearing, and what it buys is not what §8 thought
///
/// The consequence runs the other way too: **a component that repaints whole rows is not committing a
/// wart to be optimised away.** §8 priced it as *9x worse unfiltered and 24x better filtered*, and
/// measured over spec §14's own two arms of one scrolling list — one that rewrites a twenty-column
/// label, one that blanks the whole row first — the trade is this instead:
///
/// | the same list, scrolled one row | damaged cells | span bytes | filtered | + scroll region |
/// |---|---|---|---|---|
/// | label only | 1 600 | 5 304 | 1 925 | **679** |
/// | rows cleared, as a widget must | 24 000 | 72 504 | 1 925 | **679** |
///
/// So the cost is real and the benefit is not where §8 put it. Clearing the rows is **15x the damaged
/// cells and 13.7x the span bytes**, and on the wire it is worth **nothing at all** — the equality
/// filter and the scroll region both take the two arms to the same byte. The read for a component
/// author is therefore not *clear your rows and win 24x*:
///
/// - **On the wire, either idiom is fine** for a list whose rows are blank past what it draws. The
///   filter compares against what the terminal is showing, and the scroll region proves a shift
///   against it; neither can see how the damage was marked.
/// - **What clearing buys is that the guarantee holds unconditionally.** The row a scroll exposes is
///   one the terminal *erases*, so every column of it the component does not repaint has to be a
///   column the component wanted blank. A component that repaints its whole rectangle satisfies that
///   always; one that repaints a label inside it satisfies it only while the rest of the row happens
///   to be blank, and the frame it stops being blank is the frame the scroll silently stops being
///   available. `crate::roundtrip::two_text_verbs_with_a_one_column_gap_do_not_take_the_scroll_path`
///   is that frame.
/// - **What it costs is app-thread time**, which is the thread under budget pressure — 34 µs of
///   packing against 1.9 µs in §8's measurement — while the render thread has 16.6 ms and nothing
///   else to do with it.
///
/// The numbers above are produced rather than quoted: `crate::gates::the_scroll_region_over_spec_8s_two_arms`
/// and `crate::gates::the_equality_filter_reproduces_spec_8s_table` print all four columns, and the
/// first of them gates the two arms being *equal* so that this table cannot go stale quietly.
pub struct Surface {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
    damage: RowBits,
    /// What an untouched cell of this surface holds: a blank for an opaque surface, `EMPTY` for a
    /// non-opaque one. A repair blanks half a pair back to *this*, not to a space — blanking to a
    /// space inside a non-opaque layer punches exactly the hole `opaque: false` exists to prevent,
    /// in a cell the caller never asked for.
    ground: GraphemeId,
    /// The handle space of a surface **outside** a layer stack, and nothing else.
    ///
    /// `Surface::new` and `Surface::root` are public, and a `View` from that door has no engine to
    /// reach through — so the standalone door brings its own tables. A surface
    /// the stack minted never touches these: `LayerStack::view` hands the *stack's* tables to the
    /// `View`, which is what keeps one handle space per stack and compositing a `copy_from_slice`.
    /// An untouched table holds no allocation, so the field costs a layer surface nothing but its
    /// width.
    tables: Tables,
}

impl std::fmt::Debug for Surface {
    /// Dimensions only. A derived `Debug` would print every `Cell` — its grapheme handle and its
    /// packed style bits — through `{:?}`, which is the read-back
    /// `docs/adr/0023-the-cell-is-never-visible-in-the-public-api.md` forecloses: a caller cannot
    /// read back what is already on screen. The formatter is not an exception to that.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Surface {{ {}x{} }}", self.width, self.height)
    }
}

impl Surface {
    /// A surface of blank cells, `w` wide and `h` tall, with nothing damaged.
    pub fn new(w: u16, h: u16) -> Surface {
        Surface::new_filled(w, h, Cell::BLANK)
    }

    /// A surface every cell of which starts as `ground`. A non-opaque layer is born `EMPTY`.
    pub(crate) fn new_filled(w: u16, h: u16, ground: Cell) -> Surface {
        Surface {
            width: w,
            height: h,
            cells: vec![ground; w as usize * h as usize],
            damage: RowBits::new(w, h),
            ground: ground.grapheme,
            tables: Tables::new(),
        }
    }

    /// The surface's size in cells.
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// A view of the whole surface, drawing into this surface's own handle space.
    ///
    /// The standalone door. A surface reached this way is not in a layer stack, so it interns into
    /// the tables it carries; `add_content_with` renumbers those handles into the stack's when the
    /// surface is donated. The fields are taken apart here rather than passed as
    /// `&mut self` because the verbs need the cells, the damage **and** the tables at once, and
    /// they come from one struct.
    pub fn root(&mut self) -> View<'_> {
        let clip = Rect::new(0, 0, self.width, self.height);
        View::new(
            &mut self.cells,
            &mut self.damage,
            self.width,
            clip,
            self.ground,
            &mut self.tables,
        )
    }

    /// A view of the whole surface, drawing into a handle space that is not this surface's.
    ///
    /// What [`LayerStack::view`](crate::LayerStack::view) hands out: one set of tables per stack,
    /// so a handle crossing a surface boundary inside the stack needs no translation.
    pub(crate) fn draw<'a>(&'a mut self, tables: &'a mut Tables) -> View<'a> {
        let clip = Rect::new(0, 0, self.width, self.height);
        View::new(
            &mut self.cells,
            &mut self.damage,
            self.width,
            clip,
            self.ground,
            tables,
        )
    }

    /// This surface's own handle space. Empty unless something was drawn through
    /// [`Surface::root`](Surface::root).
    pub(crate) fn tables(&self) -> &Tables {
        &self.tables
    }

    /// This surface's own handle space, to mint into or to hand back empty.
    ///
    /// [`LayerStack::add_content_with`](crate::LayerStack::add_content_with) empties it once the
    /// surface's handles have been renumbered into the stack's space: the surface then speaks that
    /// space and its own table is unreachable, so keeping it would be holding the donor's clusters
    /// alive for the life of the layer.
    pub(crate) fn tables_mut(&mut self) -> &mut Tables {
        &mut self.tables
    }

    /// What an untouched cell of this surface holds, and what a repaired half of a pair goes back
    /// to: a blank for an opaque surface, `EMPTY` for a non-opaque one.
    pub(crate) fn ground(&self) -> GraphemeId {
        self.ground
    }

    pub(crate) fn width(&self) -> u16 {
        self.width
    }

    pub(crate) fn height(&self) -> u16 {
        self.height
    }

    pub(crate) fn row(&self, y: u16) -> &[Cell] {
        let start = y as usize * self.width as usize;
        &self.cells[start..start + self.width as usize]
    }

    pub(crate) fn row_mut(&mut self, y: u16) -> &mut [Cell] {
        let start = y as usize * self.width as usize;
        &mut self.cells[start..start + self.width as usize]
    }

    /// How many cells this surface's damage reports.
    ///
    /// Gate #3's numerator, and it lives here because the sum is over this surface's own bitset:
    /// the layer stack adds them up, it does not compute them.
    #[cfg(test)]
    pub(crate) fn damaged_cells(&self) -> usize {
        let mut total = 0;
        self.damage.for_each_run(|r| total += r.len());
        total
    }

    pub(crate) fn damage(&self) -> &RowBits {
        &self.damage
    }

    pub(crate) fn damage_mut(&mut self) -> &mut RowBits {
        &mut self.damage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn a_new_surface_is_blank_and_undamaged() {
        let s = Surface::new(8, 3);
        assert_eq!(s.size(), (8, 3));
        assert!(s.damage().is_empty());
        assert!(s.row(0).iter().all(|c| *c == Cell::BLANK));
    }

    #[test]
    fn every_row_is_the_surfaces_width() {
        let s = Surface::new(300, 80);
        for y in 0..80 {
            assert_eq!(s.row(y).len(), 300);
        }
    }

    #[test]
    fn rows_are_not_padded_to_a_cache_line() {
        // Spec §3 measured padding at 4.845 against 4.847 us and refused it, so a row of 297 cells
        // is 297 cells and the next row starts immediately after.
        let s = Surface::new(297, 4);
        assert_eq!(s.cells.len(), 297 * 4);
    }

    #[test]
    fn a_zero_sized_surface_is_legal_and_holds_nothing() {
        let s = Surface::new(0, 0);
        assert_eq!(s.size(), (0, 0));
        assert!(s.damage().is_empty());
    }
}
