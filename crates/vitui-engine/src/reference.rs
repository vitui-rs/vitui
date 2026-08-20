//! The reference compositor: one cell at a time, no damage, no runs.
//!
//! Obviously correct by inspection, and far too slow to ship — it visits every cell of the screen
//! against every layer of the stack, every frame, whether anything changed or not. That is the
//! point. The fast path must agree with it cell for cell, and **gate #1 is generated from it rather
//! than written by hand**: a hand-written expectation about damage is written by the same person
//! who wrote the damage, and agrees with it for the same reason.
//!
//! # What it is an oracle for
//!
//! - **Gate #1, no damage structure under-reports.** Composite the reference before a frame's verbs
//!   and after them; every cell that differs must lie inside a run the damage structure reported.
//!   Nothing in that sentence mentions how damage is marked, which is why it survives the damage
//!   structure being replaced.
//! - **The composited picture.** The frame the fast path produces must equal this one everywhere,
//!   including the cells outside every damaged run — that is what says damage-tracked compositing
//!   and full compositing are the same function.
//!
//! Spec §14 also names it as the oracle for ticket 25's first fuzz target, draw sequences against a
//! naive reference compositor. That is when it stops being `cfg(test)`.
//!
//! # The one thing it must not do
//!
//! Share code with the fast path. Every line here is written from spec §5's description of
//! painter's algorithm rather than from `LayerStack::composite_run`, including the ground fill the
//! fast path deliberately does not have — the fast path can skip it because every damaged cell lies
//! inside some layer's rectangle, and an oracle that made the same assumption would be unable to
//! notice if it stopped being true.

use crate::cell::Cell;
use crate::layer::LayerStack;
use crate::surface::Surface;

/// Composite the whole stack into a fresh `w` by `h` surface, bottom-up, one cell at a time.
pub(crate) fn composite(stack: &LayerStack, w: u16, h: u16) -> Surface {
    // Sorted here rather than taken from the stack's storage order. Spec §5 says the stacking order
    // is `(z, seq)` — z first, insertion order breaking ties, so that raising a layer and dropping
    // it back restores the exact original order — and the oracle applies that rule itself. Reading
    // the fast path's order back would make a defect in `add_content`'s insert invisible to the
    // gate this compositor generates.
    let mut layers: Vec<_> = stack.as_stored().collect();
    layers.sort_by_key(|l| (l.z, l.seq));

    let mut out = Surface::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let mut cell = Cell::BLANK;
            for layer in &layers {
                let lx = x as i32 - layer.rect.x;
                let ly = y as i32 - layer.rect.y;
                if lx < 0 || ly < 0 || lx >= layer.rect.w as i32 || ly >= layer.rect.h as i32 {
                    continue;
                }
                let src = layer.surface.row(ly as u16)[lx as usize];
                // An opaque layer paints every cell it covers; a non-opaque one paints only the
                // cells its caller actually wrote, and `EMPTY` is the sentinel for "skip this one"
                // (spec §5). One condition rather than two identical branches, because clippy is
                // right that they were identical.
                if layer.opaque || !src.grapheme.is_empty() {
                    cell = src;
                }
            }
            out.row_mut(y)[x as usize] = cell;
        }
    }
    out
}

/// Every cell at which two surfaces of the same size disagree.
pub(crate) fn differences(before: &Surface, after: &Surface) -> Vec<(u16, u16)> {
    let (w, h) = before.size();
    assert_eq!(before.size(), after.size(), "two sizes cannot be compared");
    let mut out = Vec::new();
    for y in 0..h {
        for x in 0..w {
            if before.row(y)[x as usize] != after.row(y)[x as usize] {
                out.push((x, y));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geom::Rect;
    use crate::style::Style;

    fn glyphs(s: &Surface, y: u16) -> String {
        s.row(y)
            .iter()
            .map(|c| c.grapheme.as_scalar().unwrap_or('?'))
            .collect()
    }

    #[test]
    fn an_empty_stack_composites_to_blanks() {
        let stack = LayerStack::new();
        let out = composite(&stack, 4, 1);
        assert_eq!(glyphs(&out, 0), "    ");
    }

    #[test]
    fn a_higher_layer_paints_over_a_lower_one() {
        let mut stack = LayerStack::new();
        let low = stack.add_content(0, Rect::new(0, 0, 8, 1), true);
        let high = stack.add_content(1, Rect::new(2, 0, 3, 1), true);
        stack
            .view(low)
            .unwrap()
            .fill(Rect::new(0, 0, 8, 1), ".", Style::new());
        stack
            .view(high)
            .unwrap()
            .fill(Rect::new(0, 0, 3, 1), "#", Style::new());
        assert_eq!(glyphs(&composite(&stack, 8, 1), 0), "..###...");
    }

    #[test]
    fn a_non_opaque_layer_lets_what_is_under_it_show_through() {
        let mut stack = LayerStack::new();
        let low = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let high = stack.add_content(1, Rect::new(0, 0, 4, 1), false);
        stack
            .view(low)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), ".", Style::new());
        stack.view(high).unwrap().text(1, 0, "ab", Style::new());
        assert_eq!(glyphs(&composite(&stack, 4, 1), 0), ".ab.");
    }

    #[test]
    fn the_oracle_sorts_by_z_then_insertion_order_itself() {
        // Added high-first, so a compositor that trusted storage order and a compositor that
        // sorted would only agree if the sort is real.
        let mut stack = LayerStack::new();
        let high = stack.add_content(5, Rect::new(0, 0, 4, 1), true);
        let low = stack.add_content(-5, Rect::new(0, 0, 4, 1), true);
        stack
            .view(high)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), "#", Style::new());
        stack
            .view(low)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), ".", Style::new());
        assert_eq!(glyphs(&composite(&stack, 4, 1), 0), "####");

        // And `seq` breaks a tie in insertion order, never the other way round.
        let mut stack = LayerStack::new();
        let first = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let second = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        stack
            .view(first)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), "1", Style::new());
        stack
            .view(second)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), "2", Style::new());
        assert_eq!(glyphs(&composite(&stack, 4, 1), 0), "2222");
    }

    #[test]
    fn a_layer_hanging_off_an_edge_is_intersected_not_rejected() {
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(-2, -1, 4, 3), true);
        stack
            .view(id)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 3), "#", Style::new());
        let out = composite(&stack, 4, 2);
        assert_eq!(glyphs(&out, 0), "##  ");
        assert_eq!(glyphs(&out, 1), "##  ");
    }

    #[test]
    fn differences_finds_exactly_the_cells_that_changed() {
        let mut a = Surface::new(4, 2);
        let mut b = Surface::new(4, 2);
        b.root().text(1, 1, "xy", Style::new());
        assert_eq!(differences(&a, &b), vec![(1, 1), (2, 1)]);
        a.root().text(1, 1, "xy", Style::new());
        assert!(differences(&a, &b).is_empty());
    }
}
