//! Views and the drawing verbs.
//!
//! A [`Surface`] owns cells and damage; a borrowed `View` draws into it. A view is an origin, a
//! clip region and a content offset, with no cells of its own — its clip stack **is** the call
//! stack, which is what makes zero allocation structural rather than disciplined, and what makes a
//! child unable to widen its parent (spec §4).
//!
//! # Scope
//!
//! This is the tracer bullet's half: `text` and `fill` at absolute coordinates on the root view.
//! `child`, `scrolled` and the visibility queries arrive with ticket 09; `set` and `restyle` with
//! tickets 06 and 07.

use std::marker::PhantomData;

use crate::cell::{Cell, GraphemeId};
use crate::geom::Rect;
use crate::style::Style;
use crate::surface::Surface;

/// Why a drawing verb stopped.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stop {
    /// The whole string was placed.
    Complete,
    /// The clip region ended before the string did.
    Clipped,
    /// Nothing was placed: the verb fell entirely outside the clip region.
    Offscreen,
}

/// What a drawing verb did.
///
/// The escape hatch is this, not a `Result` (ADR 0022). A caller that needs to know where a verb
/// stopped is told; a caller that does not pays nothing. `bytes` is how much of the string was
/// consumed, which is what a caller that wraps or paginates would otherwise have to re-segment to
/// find out.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Written {
    /// How many cells were written.
    pub cells: u16,
    /// How many bytes of the string were consumed.
    pub bytes: usize,
    /// Why it stopped.
    pub stop: Stop,
}

impl Written {
    /// Nothing was written, because nothing could be.
    pub const NONE: Written = Written {
        cells: 0,
        bytes: 0,
        stop: Stop::Offscreen,
    };
}

/// A borrowed rectangle of a surface.
///
/// # Threading
///
/// `View` is deliberately **not** [`Send`]. Ticket 05 claimed the borrow was enough — "a `View`
/// borrows and therefore cannot be sent anywhere" — and ticket 18 refuted it by test: that holds
/// only against `thread::spawn`, whose `'static` bound was doing the work, while `Surface: Send`
/// implies `&mut Surface: Send` implies `View: Send`, and a `thread::scope` closure that draws
/// through a `View` compiles. The fix is the private zero-sized field below, which is invisible to
/// callers and verified in both directions.
///
/// The narrow price: this forecloses splitting **one** surface across threads. Drawing into
/// *separate* surfaces on separate threads is untouched.
///
/// ```compile_fail,E0277
/// let mut surface = vitui_engine::Surface::new(8, 2);
/// let view = surface.root();
/// std::thread::scope(|s| {
///     s.spawn(move || {
///         let mut view = view;
///         view.fill(vitui_engine::Rect::new(0, 0, 1, 1), "x", vitui_engine::Style::new());
///     });
/// });
/// ```
///
/// ```
/// let mut surface = vitui_engine::Surface::new(8, 2);
/// let mut view: vitui_engine::View<'_> = surface.root();
/// view.fill(vitui_engine::Rect::new(0, 0, 1, 1), "x", vitui_engine::Style::new());
/// ```
pub struct View<'a> {
    surface: &'a mut Surface,
    /// Absolute, in surface coordinates, already intersected with the surface itself.
    clip: Rect,
    /// `View: !Send`. Zero-sized, private, and the whole mechanism.
    _not_send: PhantomData<*const ()>,
}

impl<'a> View<'a> {
    pub(crate) fn root(surface: &'a mut Surface) -> View<'a> {
        let clip = Rect::new(0, 0, surface.width(), surface.height());
        View {
            surface,
            clip,
            _not_send: PhantomData,
        }
    }

    /// The view's size in cells.
    pub fn size(&self) -> (u16, u16) {
        (self.clip.w, self.clip.h)
    }

    /// Write `s` starting at `(x, y)`, in the given style.
    ///
    /// Out-of-bounds writes are discarded silently — no panic, no `Result`, not even a
    /// `debug_assert`. A virtualised component writes far outside a surface as a matter of course,
    /// and that is normal traffic rather than an error (ADR 0022).
    ///
    /// # Scope
    ///
    /// Ticket 03 puts one scalar in one cell. That is the final encoding for a narrow single-scalar
    /// cluster — a cell holding `a` *is* `0x61` — but it is not segmentation: ticket 06 replaces
    /// this walk with the cluster iterator, and with it come display width and the five repair
    /// rules. Until then the engine's own scenes are ASCII.
    pub fn text(&mut self, x: i32, y: i32, s: &str, style: Style) -> Written {
        if s.is_empty() {
            return Written {
                cells: 0,
                bytes: 0,
                stop: Stop::Complete,
            };
        }
        if y < self.clip.y || y >= self.clip.bottom() {
            return Written::NONE;
        }

        let (cells, damage, width) = self.surface.parts_mut();
        let row_start = y as usize * width as usize;
        let mut written = 0u16;
        let mut consumed = 0usize;
        let mut first = i32::MAX;
        let mut last = i32::MIN;
        let mut clipped = false;

        // The column of a scalar is its index in the string, because ticket 03 writes one cell per
        // scalar. Ticket 06 is what makes that an advance by the cluster's width instead.
        for (col, (offset, ch)) in (x..).zip(s.char_indices()) {
            if col >= self.clip.right() {
                clipped = true;
                consumed = offset;
                break;
            }
            if col >= self.clip.x {
                cells[row_start + col as usize] = Cell::new(GraphemeId::scalar(ch), style);
                written += 1;
                first = first.min(col);
                last = last.max(col);
            }
            consumed = offset + ch.len_utf8();
        }

        if written > 0 {
            damage.mark(y, first, last);
        }

        Written {
            cells: written,
            bytes: consumed,
            stop: if written == 0 {
                Stop::Offscreen
            } else if clipped {
                Stop::Clipped
            } else {
                Stop::Complete
            },
        }
    }

    /// Fill `r` with `cluster`, in the given style.
    ///
    /// Seventeen times cheaper than the same cells written with [`text`](Self::text), because it is
    /// a `slice::fill` of a sixteen-byte `Copy` (spec §4). Expressing a fill as repeated text
    /// throws that away.
    pub fn fill(&mut self, r: Rect, cluster: &str, style: Style) {
        let Some(ch) = cluster.chars().next() else {
            return;
        };
        let area = r.intersect(self.clip);
        if area.is_empty() {
            return;
        }
        let cell = Cell::new(GraphemeId::scalar(ch), style);
        let (cells, damage, width) = self.surface.parts_mut();
        for y in area.y..area.bottom() {
            let row_start = y as usize * width as usize;
            let lo = row_start + area.x as usize;
            let hi = row_start + area.right() as usize;
            cells[lo..hi].fill(cell);
            damage.mark(y, area.x, area.right() - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::damage::Run;
    use crate::style::Color;

    fn runs(s: &Surface) -> Vec<Run> {
        let mut out = Vec::new();
        s.damage().for_each_run(|r| out.push(r));
        out
    }

    fn glyphs(s: &Surface, y: u16) -> String {
        s.row(y)
            .iter()
            .map(|c| c.grapheme.as_scalar().unwrap_or('?'))
            .collect()
    }

    #[test]
    fn text_puts_one_scalar_in_one_cell() {
        let mut s = Surface::new(8, 2);
        let w = s.root().text(1, 0, "hi", Style::new());
        assert_eq!(glyphs(&s, 0), " hi     ");
        assert_eq!(w.cells, 2);
        assert_eq!(w.bytes, 2);
        assert_eq!(w.stop, Stop::Complete);
    }

    #[test]
    fn text_marks_damage_once_over_the_span_it_wrote() {
        let mut s = Surface::new(8, 2);
        s.root().text(2, 1, "abc", Style::new());
        assert_eq!(runs(&s), vec![Run { y: 1, lo: 2, hi: 4 }]);
    }

    #[test]
    fn text_carries_its_style_into_every_cell() {
        let st = Style::new().fg(Color::indexed(3)).bold();
        let mut s = Surface::new(8, 1);
        s.root().text(0, 0, "ab", st);
        assert_eq!(s.row(0)[0].style, st);
        assert_eq!(s.row(0)[1].style, st);
        assert_eq!(s.row(0)[2].style, Style::DEFAULT);
    }

    #[test]
    fn text_off_the_bottom_writes_nothing() {
        let mut s = Surface::new(8, 2);
        let w = s.root().text(0, 2, "abc", Style::new());
        assert_eq!(w, Written::NONE);
        assert!(s.damage().is_empty());
    }

    #[test]
    fn text_off_the_top_writes_nothing() {
        let mut s = Surface::new(8, 2);
        assert_eq!(s.root().text(0, -1, "abc", Style::new()), Written::NONE);
    }

    #[test]
    fn text_running_off_the_right_edge_is_clipped_and_says_so() {
        let mut s = Surface::new(4, 1);
        let w = s.root().text(2, 0, "abcd", Style::new());
        assert_eq!(glyphs(&s, 0), "  ab");
        assert_eq!(w.cells, 2);
        assert_eq!(w.bytes, 2, "the caller can resume from byte 2");
        assert_eq!(w.stop, Stop::Clipped);
    }

    #[test]
    fn text_starting_left_of_the_surface_writes_only_what_lands_on_it() {
        let mut s = Surface::new(4, 1);
        let w = s.root().text(-2, 0, "abcdef", Style::new());
        assert_eq!(glyphs(&s, 0), "cdef");
        assert_eq!(w.cells, 4);
        assert_eq!(
            w.bytes, 6,
            "the two columns left of the surface were still consumed"
        );
        assert_eq!(
            w.stop,
            Stop::Complete,
            "every scalar was consumed, so the verb finished even though two were discarded"
        );
        assert_eq!(runs(&s), vec![Run { y: 0, lo: 0, hi: 3 }]);
    }

    #[test]
    fn text_entirely_left_of_the_surface_is_offscreen() {
        let mut s = Surface::new(4, 1);
        let w = s.root().text(-10, 0, "ab", Style::new());
        assert_eq!(w.cells, 0);
        assert_eq!(w.stop, Stop::Offscreen);
        assert!(s.damage().is_empty());
    }

    #[test]
    fn text_entirely_right_of_the_surface_is_offscreen() {
        let mut s = Surface::new(4, 1);
        let w = s.root().text(10, 0, "ab", Style::new());
        assert_eq!(w.stop, Stop::Offscreen);
        assert!(s.damage().is_empty());
    }

    #[test]
    fn an_empty_string_is_complete_and_damages_nothing() {
        let mut s = Surface::new(4, 1);
        let w = s.root().text(0, 0, "", Style::new());
        assert_eq!(w.cells, 0);
        assert_eq!(w.stop, Stop::Complete);
        assert!(s.damage().is_empty());
    }

    #[test]
    fn text_counts_bytes_not_scalars() {
        let mut s = Surface::new(8, 1);
        // Two scalars, three bytes. Ticket 06 is what makes the second one a cluster with a width.
        let w = s.root().text(0, 0, "a\u{e9}", Style::new());
        assert_eq!(w.cells, 2);
        assert_eq!(w.bytes, 3);
    }

    #[test]
    fn fill_covers_its_rectangle_and_nothing_else() {
        let mut s = Surface::new(6, 4);
        s.root().fill(Rect::new(1, 1, 3, 2), "#", Style::new());
        assert_eq!(glyphs(&s, 0), "      ");
        assert_eq!(glyphs(&s, 1), " ###  ");
        assert_eq!(glyphs(&s, 2), " ###  ");
        assert_eq!(glyphs(&s, 3), "      ");
    }

    #[test]
    fn fill_marks_one_run_per_row() {
        let mut s = Surface::new(6, 4);
        s.root().fill(Rect::new(1, 1, 3, 2), "#", Style::new());
        assert_eq!(
            runs(&s),
            vec![Run { y: 1, lo: 1, hi: 3 }, Run { y: 2, lo: 1, hi: 3 }]
        );
    }

    #[test]
    fn fill_clips_to_the_surface() {
        let mut s = Surface::new(4, 2);
        s.root()
            .fill(Rect::new(-2, -1, 100, 100), "#", Style::new());
        assert_eq!(glyphs(&s, 0), "####");
        assert_eq!(glyphs(&s, 1), "####");
    }

    #[test]
    fn fill_with_an_empty_cluster_does_nothing() {
        let mut s = Surface::new(4, 2);
        s.root().fill(Rect::new(0, 0, 4, 2), "", Style::new());
        assert!(s.damage().is_empty());
        assert_eq!(glyphs(&s, 0), "    ");
    }

    #[test]
    fn fill_of_an_empty_rectangle_does_nothing() {
        let mut s = Surface::new(4, 2);
        s.root().fill(Rect::new(0, 0, 0, 2), "#", Style::new());
        s.root().fill(Rect::new(10, 10, 4, 4), "#", Style::new());
        assert!(s.damage().is_empty());
    }

    #[test]
    fn a_root_view_is_the_whole_surface() {
        let mut s = Surface::new(300, 80);
        assert_eq!(s.root().size(), (300, 80));
    }
}
