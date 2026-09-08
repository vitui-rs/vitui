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
//! It is also the oracle for the first fuzz target, draw sequences against a naive
//! reference compositor, and **that is what the `fuzz` feature is for**: the `cfg` is
//! `any(test, feature = "fuzz")` now, because a fuzz target compiles this crate without `cfg(test)`
//! — `fuzz/` is its own workspace, since `cargo-fuzz` needs nightly and `libfuzzer-sys`. It is on
//! `crate::audit`'s `SOAK_ONLY_MODULES` rather than its `TEST_ONLY_MODULES`, which is what keeps a
//! module in that third state from being silently exempt from the surface scans.
//!
//! What it found there, on its first run: `crate::layer`'s `is_continuation` asked the **layers**
//! whether a `CONTINUATION` still had its head, and a layer extends past the screen while the frame
//! does not — so a pair the frame's own clamp bisected still looked whole, and an operator declined
//! a column that was by then an ordinary space. `repair` below has always had the other reading:
//! it runs a boundary *before* the first column and *after* the last, for exactly this.
//!
//! # The one thing it must not do
//!
//! Share code with the fast path. Every line here is written from the description of
//! painter's algorithm rather than from `LayerStack::composite_run`, including the ground fill,
//! which the fast path does conditionally — it looks for an opaque layer covering the whole run and
//! starts there — where this one does it unconditionally, for every cell, every time. That is the
//! shape of the whole file: the oracle is allowed to be slow and is not allowed to be clever.
//!
//! # Where the line is drawn for an operator layer, and why it is drawn there
//!
//! An operator's **placement** is written here from scratch and its **arithmetic** is not, and that
//! division is deliberate.
//!
//! Placement is the mechanism, and it is where every defect in this area has lived: a shadow one
//! column too wide is invisible while a modal dim one column too wide floods a column of the modal.
//! So the atomic-glyph rule is stated here the other way round from
//! [`recolour`](crate::layer) — that walks glyphs left to right and steps a head over its
//! continuation; this asks of every cell, independently, *where is this glyph's head, and is it
//! inside the rectangle* — and the occlusion, the stacking order and the picture each operator acts
//! on are all recomputed rather than tracked.
//!
//! The arithmetic is [`crate::mix`]'s, shared. `blend` and the colour resolution are a *definition* —
//! what "half way toward black" means, and what `default` resolves to on a terminal that answered —
//! not a mechanism, and two copies of a definition disagree in the last bit with no way to say which
//! copy is right. Interning is shared for a harder reason: an extended style word **is** a handle
//! into this stack's table, so an oracle that minted its own would produce cells that compare
//! unequal for no reason but the handle.
//!
//! # The picture an operator acts on is the one the layers below it make
//!
//! Layers paint bottom-up, so an operator cannot see what paints over it afterwards, and the glyph
//! plane its atomic-glyph decision reads is the composite of the **content layers below it** — seams
//! and all. The fast path gets that for free by running the operator in its turn. This rebuilds that
//! plane per operator, from scratch, which is exactly the kind of thing the oracle is allowed to do.

use crate::caps::Capabilities;
use crate::cell::{Cell, GraphemeId};
use crate::layer::{LayerRef, LayerStack, Paint};
use crate::mix::Mixer;
use crate::surface::Surface;

/// Composite the whole stack into a fresh `w` by `h` surface, bottom-up, one cell at a time.
pub(crate) fn composite(stack: &mut LayerStack, caps: &Capabilities, w: u16, h: u16) -> Surface {
    let (mut layers, tables) = stack.parts();
    // Sorted here rather than taken from the stack's storage order. Spec §5 says the stacking order
    // is `(z, seq)` — z first, insertion order breaking ties, so that raising a layer and dropping
    // it back restores the exact original order — and the oracle applies that rule itself. Reading
    // the fast path's order back would make a defect in `add_content`'s insert invisible to the
    // gate this compositor generates.
    layers.sort_by_key(|l| (l.z, l.seq));

    let mut out = Surface::new(w, h);
    // Which layer each cell of the row came from, or `None` where none covered it. It is what lets
    // `repair` tell a pair the **composite** broke from one that arrived broken, and what says which
    // operators a cell is out of reach of.
    let mut painter = vec![None; w as usize];
    // The glyph plane one operator sees, rebuilt for each of them, and the painter map that comes
    // with it — read by `repair` inside `paint_row` and by nothing here, because which layer painted
    // a cell *below* the operator does not change whether the operator acts on it.
    let mut below: Vec<Cell> = vec![Cell::BLANK; w as usize];
    let mut below_painter = vec![None; w as usize];
    for y in 0..h {
        paint_row(&layers, out.row_mut(y), &mut painter, y);

        for (j, layer) in layers.iter().enumerate() {
            let Paint::Operator(mix) = layer.paint else {
                continue;
            };
            let Some(mixer) = Mixer::new(mix, caps) else {
                continue;
            };
            let row_inside = y as i32 >= layer.rect.y && (y as i32) < layer.rect.bottom();
            if !row_inside || layer.rect.is_empty() {
                continue;
            }
            paint_row(&layers[..j], &mut below, &mut below_painter, y);
            let row = out.row_mut(y);
            for x in 0..w as usize {
                // Anything a content layer above this operator painted is out of its reach: the
                // operator ran first and was overwritten.
                if painter[x].is_some_and(|i| i > j) {
                    continue;
                }
                // **A glyph is atomic and belongs to its head cell: an operator acts on it if and
                // only if the head is inside the rectangle.** One condition, and it is the whole
                // rule — it lets the operator reach one column past a right edge that cut a head,
                // and it keeps it off a glyph whose head lies left of the rectangle.
                let head = if below[x].grapheme.is_continuation() {
                    x as i32 - 1
                } else {
                    x as i32
                };
                if head < layer.rect.x || head >= layer.rect.right() {
                    continue;
                }
                row[x].style = mixer.style(tables, row[x].style);
            }
        }
    }
    out
}

/// One row of the picture `layers` make, and which layer painted each cell of it.
///
/// Content layers only, because an operator changes no glyph — and this is called both for the
/// finished row and for the partial plane an operator sees, which is the same function of a shorter
/// slice of the stack.
fn paint_row(layers: &[LayerRef<'_>], row: &mut [Cell], painter: &mut [Option<usize>], y: u16) {
    for (x, cell) in row.iter_mut().enumerate() {
        *cell = Cell::BLANK;
        painter[x] = None;
        for (i, layer) in layers.iter().enumerate() {
            let Paint::Content { opaque, surface } = layer.paint else {
                continue;
            };
            let lx = x as i32 - layer.rect.x;
            let ly = y as i32 - layer.rect.y;
            if lx < 0 || ly < 0 || lx >= layer.rect.w as i32 || ly >= layer.rect.h as i32 {
                continue;
            }
            let src = surface.row(ly as u16)[lx as usize];
            // An opaque layer paints every cell it covers; a non-opaque one paints only the
            // cells its caller actually wrote, and `EMPTY` is the sentinel for "skip this one"
            // (spec §5). One condition rather than two identical branches, because clippy is
            // right that they were identical.
            if opaque || !src.grapheme.is_empty() {
                *cell = src;
                painter[x] = Some(i);
            }
        }
    }
    repair(row, painter);
}

/// Blank every half of a double-width pair that the **composite** left without its partner.
///
/// The fast path does this as four O(1) fixes per row per layer, at the two seams each paint
/// creates, while the picture is being built. This is the same property stated the other way round:
/// look at the finished picture, ask of every boundary whether the composite is what put those two
/// cells next to each other, and blank whatever does not pair. It is far less clever and cannot miss
/// a seam, which is the division of labour the whole file is written to — the oracle is allowed to
/// be slow and is not allowed to be clever.
///
/// **A boundary between two columns one layer painted at once is that layer's own business.** The
/// composite repairs what compositing broke and nothing else. There used to be exactly one way for a
/// layer to hand it a pair that was already broken — [`View::child`](crate::View::child) could not
/// widen its clip, so a pair the clip bisected kept the half outside it — and this file
/// deliberately did not repair it, because an oracle that quietly did would have made gate #1 fail
/// against the compositor instead of surfacing the open question. That was closed
/// question the other way and the door with it: the drawing verbs' repair is bounded by the surface,
/// so no layer can hand this a broken pair. `crate::fuzz` asserts that of the picture below rather
/// than allowing for it.
///
/// One left-to-right pass is enough. Blanking a head cannot orphan the cell to its right, because it
/// is blanked precisely when that cell is not its continuation; blanking a continuation cannot orphan
/// the cell to its left for the mirror image of the same reason.
fn repair(row: &mut [Cell], painter: &[Option<usize>]) {
    let w = row.len();
    // One boundary per gap between columns, plus the one before the first column and the one after
    // the last: a continuation in column zero and a wide head in the last column are both orphans,
    // and neither has a neighbour to be measured against.
    for b in 0..=w {
        let left = if b == 0 { None } else { painter[b - 1] };
        let right = if b == w { None } else { painter[b] };
        if left.is_some() && left == right {
            continue;
        }
        let head = b > 0 && row[b - 1].grapheme.is_wide_head();
        let cont = b < w && row[b].grapheme.is_continuation();
        let orphan = match (head, cont) {
            (true, false) => b - 1,
            (false, true) => b,
            _ => continue,
        };
        row[orphan] = Cell::new(GraphemeId::SPACE, row[orphan].style);
    }
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
    use crate::testing::terminal;

    fn glyphs(s: &Surface, y: u16) -> String {
        s.row(y)
            .iter()
            .map(|c| c.grapheme.as_scalar().unwrap_or('?'))
            .collect()
    }

    #[test]
    fn an_empty_stack_composites_to_blanks() {
        let mut stack = LayerStack::new();
        let out = composite(&mut stack, &terminal::silent(), 4, 1);
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
        assert_eq!(
            glyphs(&composite(&mut stack, &terminal::silent(), 8, 1), 0),
            "..###..."
        );
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
        assert_eq!(
            glyphs(&composite(&mut stack, &terminal::silent(), 4, 1), 0),
            ".ab."
        );
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
        assert_eq!(
            glyphs(&composite(&mut stack, &terminal::silent(), 4, 1), 0),
            "####"
        );

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
        assert_eq!(
            glyphs(&composite(&mut stack, &terminal::silent(), 4, 1), 0),
            "2222"
        );
    }

    #[test]
    fn a_layer_hanging_off_an_edge_is_intersected_not_rejected() {
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(-2, -1, 4, 3), true);
        stack
            .view(id)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 3), "#", Style::new());
        let out = composite(&mut stack, &terminal::silent(), 4, 2);
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
