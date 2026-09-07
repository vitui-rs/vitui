//! Rectangles, in absolute cell coordinates.

/// A rectangle of cells: an origin and a size.
///
/// The origin is signed because a layer may hang off the top or left edge of the screen — such a
/// layer is intersected, never rejected. The size is unsigned because a rectangle with a
/// negative extent is not a thing the engine has to have an opinion about.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Rect {
    /// Column of the left edge, inclusive.
    pub x: i32,
    /// Row of the top edge, inclusive.
    pub y: i32,
    /// Width in cells.
    pub w: u16,
    /// Height in cells.
    pub h: u16,
}

impl Rect {
    /// A rectangle at `(x, y)`, `w` cells wide and `h` cells tall.
    pub const fn new(x: i32, y: i32, w: u16, h: u16) -> Rect {
        Rect { x, y, w, h }
    }

    /// The column one past the right edge.
    ///
    /// Saturating, so a rectangle placed at the far end of the coordinate space stays a rectangle
    /// rather than wrapping into one that starts to the left of where it was put.
    pub const fn right(self) -> i32 {
        self.x.saturating_add(self.w as i32)
    }

    /// The row one past the bottom edge. Saturating, for the reason `right` is.
    pub const fn bottom(self) -> i32 {
        self.y.saturating_add(self.h as i32)
    }

    /// Whether the rectangle covers no cells.
    pub const fn is_empty(self) -> bool {
        self.w == 0 || self.h == 0
    }

    /// The overlap of two rectangles, empty when they do not overlap.
    pub fn intersect(self, other: Rect) -> Rect {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        if right <= x || bottom <= y {
            return Rect::new(x, y, 0, 0);
        }
        // Both differences are positive and bounded by the smaller of two `u16` extents.
        Rect::new(x, y, (right - x) as u16, (bottom - y) as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersect_of_overlapping_rects_is_the_overlap() {
        let a = Rect::new(0, 0, 10, 10);
        let b = Rect::new(5, 5, 10, 10);
        assert_eq!(a.intersect(b), Rect::new(5, 5, 5, 5));
    }

    #[test]
    fn intersect_of_disjoint_rects_is_empty() {
        let a = Rect::new(0, 0, 4, 4);
        let b = Rect::new(10, 10, 4, 4);
        assert!(a.intersect(b).is_empty());
    }

    #[test]
    fn a_child_cannot_widen_its_parent() {
        // Spec §4: a view asking for 400x400 inside a 6x2 parent touches exactly the parent's cells.
        let parent = Rect::new(0, 0, 6, 2);
        let asked = Rect::new(0, 0, 400, 400);
        assert_eq!(parent.intersect(asked), parent);
    }

    #[test]
    fn a_rect_hanging_off_the_left_edge_is_clipped_not_rejected() {
        let screen = Rect::new(0, 0, 80, 24);
        let layer = Rect::new(-5, -2, 10, 10);
        assert_eq!(screen.intersect(layer), Rect::new(0, 0, 5, 8));
    }

    #[test]
    fn right_saturates_rather_than_wrapping() {
        assert_eq!(Rect::new(i32::MAX - 1, 0, 10, 1).right(), i32::MAX);
    }
}
