//! Surfaces: a rectangular grid of cells, and the damage that says which of them changed.

use crate::cell::{Cell, GraphemeId};
use crate::damage::RowBits;
use crate::geom::Rect;
use crate::intern::Interner;
use crate::view::View;

/// A rectangular grid of cells that can be drawn into.
///
/// The engine's central primitive. A layer usually owns one; a caller constructs one directly only
/// to draw somewhere off-screen. A surface holds cells and damage and nothing else — in particular
/// it does not hold the handle tables, which is what lets one layer be composited into another as a
/// plain copy (spec §3).
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
pub struct Surface {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
    damage: RowBits,
    /// What an untouched cell of this surface holds: a blank for an opaque surface, `EMPTY` for a
    /// non-opaque one. A repair blanks half a pair back to *this*, not to a space — blanking to a
    /// space inside a non-opaque layer punches exactly the hole `opaque: false` exists to prevent
    /// (spec §5), in a cell the caller never asked for.
    ground: GraphemeId,
    /// The handle space of a surface **outside** a layer stack, and nothing else.
    ///
    /// `Surface::new` and `Surface::root` are public, and a `View` from that door has no engine to
    /// reach through — so the standalone door brings its own table (spec §3, ticket 19). A surface
    /// the stack minted never touches this one: `LayerStack::view` hands the *stack's* interner to
    /// the `View`, which is what keeps one handle space per stack and compositing a
    /// `copy_from_slice`. An untouched table holds no allocation, so the field costs a layer surface
    /// nothing but its width.
    interner: Interner,
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
            interner: Interner::new(),
        }
    }

    /// The surface's size in cells.
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// A view of the whole surface, drawing into this surface's own handle space.
    ///
    /// The standalone door. A surface reached this way is not in a layer stack, so it interns into
    /// the table it carries; `add_content_with` renumbers those handles into the stack's when the
    /// surface is donated (ticket 10). The fields are taken apart here rather than passed as
    /// `&mut self` because the verbs need the cells, the damage **and** the interner at once, and
    /// they come from one struct.
    pub fn root(&mut self) -> View<'_> {
        let clip = Rect::new(0, 0, self.width, self.height);
        View::new(
            &mut self.cells,
            &mut self.damage,
            self.width,
            clip,
            self.ground,
            &mut self.interner,
        )
    }

    /// A view of the whole surface, drawing into a handle space that is not this surface's.
    ///
    /// What [`LayerStack::view`](crate::LayerStack::view) hands out: one table per stack, so a
    /// handle crossing a surface boundary inside the stack needs no translation (ADR 0011).
    pub(crate) fn draw<'a>(&'a mut self, interner: &'a mut Interner) -> View<'a> {
        let clip = Rect::new(0, 0, self.width, self.height);
        View::new(
            &mut self.cells,
            &mut self.damage,
            self.width,
            clip,
            self.ground,
            interner,
        )
    }

    /// A view of part of this surface, in its own handle space.
    ///
    /// The mechanism `View::child` will be built on at ticket 09, reached early because the repair
    /// rules hold at a *clip* edge as well as a surface edge (spec §3) and there is otherwise no way
    /// to produce one.
    #[cfg(test)]
    pub(crate) fn clipped(&mut self, clip: Rect) -> View<'_> {
        let clip = clip.intersect(Rect::new(0, 0, self.width, self.height));
        View::new(
            &mut self.cells,
            &mut self.damage,
            self.width,
            clip,
            self.ground,
            &mut self.interner,
        )
    }

    /// This surface's own handle space. Empty unless something was drawn through
    /// [`Surface::root`](Surface::root).
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "ticket 10's `add_content_with` is the first caller outside the gates"
        )
    )]
    pub(crate) fn interner(&self) -> &Interner {
        &self.interner
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
