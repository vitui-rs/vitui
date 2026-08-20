//! The layer stack, and compositing it into a frame.
//!
//! A layer is a rectangle positioned in the stack with a z-order. A window, a popup, a shadow and a
//! modal dim are all layers; nothing else is. Rows of a virtualised tree are not layers, which is
//! why `n` here is tens and the stack is a sorted `Vec` rather than a tree or a skip list
//! (spec §5).
//!
//! # Scope
//!
//! This is the tracer bullet's half: content layers, painted bottom-up over the damaged runs. The
//! point query, removal and the reordering verbs arrive with ticket 10, the edge repairs at a layer
//! boundary with ticket 11, and the operator layer and its `Mix` with ticket 12.

use crate::cell::{Cell, GraphemeId};
use crate::damage::Run;
use crate::geom::Rect;
use crate::style::Style;
use crate::surface::Surface;
use crate::view::View;

/// A layer's identity, stable for the layer's life.
///
/// Opaque: no public field and no way to construct one from a number. A caller must be able to say
/// *this layer again* without being able to say *entry 7*.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct LayerId(u32);

/// One layer, as the reference compositor and the gates need to see it.
///
/// `cfg(test)` because spec §12's public surface names none of it: a caller cannot read back what
/// is already on screen (ADR 0023), and an oracle is not an exception to that — it is simply
/// inside the crate.
#[cfg(test)]
pub(crate) struct LayerRef<'a> {
    pub(crate) z: i32,
    /// The tie-break among equal `z`. Handed out so the reference compositor can sort for itself.
    pub(crate) seq: u32,
    pub(crate) rect: Rect,
    pub(crate) opaque: bool,
    pub(crate) surface: &'a Surface,
}

struct Layer {
    id: LayerId,
    z: i32,
    /// The tie-break among equal `z`, never mutated — so raising a layer and dropping it back
    /// restores the exact original order.
    seq: u32,
    rect: Rect,
    surface: Surface,
    opaque: bool,
}

/// The layers, in bottom-to-top order.
///
/// There is no public constructor and no `Default`: spec §12 reaches the stack only through
/// [`Screen::layers`](crate::Screen::layers), and a derived `Default` would be a second door.
pub struct LayerStack {
    /// Sorted by `(z, seq)`. The dominant operation is an ordered traversal of the whole stack
    /// every frame, which is what a contiguous `Vec` is best at.
    layers: Vec<Layer>,
    next_id: u32,
    next_seq: u32,
}

impl LayerStack {
    pub(crate) fn new() -> LayerStack {
        LayerStack {
            layers: Vec::new(),
            next_id: 0,
            next_seq: 0,
        }
    }

    /// Add a content layer at `z`, covering `rect`, and allocate its surface.
    ///
    /// `opaque` says whether every cell of the surface paints. A caller who draws only a border
    /// into an opaque layer gets a rectangle of opaque spaces that erases what is underneath —
    /// rounded corners, a tooltip with a transparent gutter and any overlay over a chart all want
    /// `opaque: false`, and pay 4.1x for the skip test (spec §5).
    ///
    /// The new layer damages its whole rectangle, because nothing beneath it has been asked to
    /// repaint what it now covers.
    pub fn add_content(&mut self, z: i32, rect: Rect, opaque: bool) -> LayerId {
        let id = LayerId(self.next_id);
        self.next_id += 1;
        let seq = self.next_seq;
        self.next_seq += 1;

        // A non-opaque layer is born EMPTY rather than blank. The trap spec §5 names is a caller
        // who draws only a border into a popup and gets a rectangle of opaque spaces that erases
        // the window underneath; being born blank is what would spring it.
        let ground = if opaque {
            Cell::BLANK
        } else {
            Cell::new(GraphemeId::EMPTY, Style::DEFAULT)
        };
        let mut surface = Surface::new_filled(rect.w, rect.h, ground);
        surface.damage_mut().mark_all();

        let layer = Layer {
            id,
            z,
            seq,
            rect,
            surface,
            opaque,
        };
        let at = self
            .layers
            .partition_point(|l| (l.z, l.seq) < (layer.z, layer.seq));
        self.layers.insert(at, layer);
        id
    }

    /// A view of a layer's cells. The only way to reach them.
    pub fn view(&mut self, id: LayerId) -> Option<View<'_>> {
        self.layers
            .iter_mut()
            .find(|l| l.id == id)
            .map(|l| l.surface.root())
    }

    /// How many layers are in the stack.
    pub fn len(&self) -> usize {
        self.layers.len()
    }

    /// Whether the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    /// Union every layer's damage into the frame, in the frame's own coordinates.
    pub(crate) fn union_damage_into(&self, frame: &mut Surface) {
        let clip = Rect::new(0, 0, frame.width(), frame.height());
        for layer in &self.layers {
            frame.damage_mut().union_translated(
                layer.surface.damage(),
                layer.rect.x,
                layer.rect.y,
                clip,
            );
        }
    }

    /// Paint the damaged runs of `frame`, bottom-up.
    ///
    /// Cost is damaged area times depth, which is the claim a flattened prefix cache would have
    /// existed to deliver — delivered by damage rectangles instead (ADR 0024).
    pub(crate) fn composite_run(&self, frame: &mut Surface, run: Run) {
        let row = frame.row_mut(run.y);
        let target = &mut row[run.lo as usize..=run.hi as usize];

        // No ground is painted first, and that is a claim rather than an omission: **every damaged
        // cell of the frame lies inside some layer's rectangle.** Damage reaches the frame only
        // through `union_damage_into`, and a layer's own bitset is sized to its own surface, so a
        // translated run cannot leave the rectangle it came from. Two damaged layers standing apart
        // stay two runs, because the bitset is exact — which is the whole reason it was chosen.
        //
        // A ground fill was written here first and survived a mutation check with every test still
        // green, which is what said it was unreachable. It becomes reachable at ticket 10, where a
        // layer can be removed or moved and expose what was under it; that ticket owns the fill and
        // the test that needs it.
        for layer in &self.layers {
            let y = run.y as i32 - layer.rect.y;
            if y < 0 || y >= layer.rect.h as i32 {
                continue;
            }
            let lo = (run.lo as i32).max(layer.rect.x);
            let hi = (run.hi as i32).min(layer.rect.right() - 1);
            if hi < lo {
                continue;
            }
            let src = layer.surface.row(y as u16);
            let src_lo = (lo - layer.rect.x) as usize;
            let src_hi = (hi - layer.rect.x) as usize;
            let dst_lo = (lo - run.lo as i32) as usize;
            let dst_hi = (hi - run.lo as i32) as usize;

            if layer.opaque {
                target[dst_lo..=dst_hi].copy_from_slice(&src[src_lo..=src_hi]);
            } else {
                for (d, s) in target[dst_lo..=dst_hi]
                    .iter_mut()
                    .zip(&src[src_lo..=src_hi])
                {
                    if !s.grapheme.is_empty() {
                        *d = *s;
                    }
                }
            }
        }
    }

    /// Every layer, in **storage order**, each carrying its own `(z, seq)`.
    ///
    /// Deliberately not "bottom-up". Storage order happens to be bottom-up because `add_content`
    /// sorts on insert, and handing that out as an ordering would make the reference compositor
    /// take its stacking order from the fast path — so a defect in that insert would be invisible
    /// to the gate generated from it. The oracle sorts for itself.
    #[cfg(test)]
    pub(crate) fn as_stored(&self) -> impl Iterator<Item = LayerRef<'_>> {
        self.layers.iter().map(|l| LayerRef {
            z: l.z,
            seq: l.seq,
            rect: l.rect,
            opaque: l.opaque,
            surface: &l.surface,
        })
    }

    /// How many cells every layer's damage reports, summed.
    ///
    /// The denominator of gate #3 is the cells the verbs wrote; this is the numerator. Summed per
    /// **layer** and never over the frame's union, because two popups that overlap on screen still
    /// write into two separate surfaces and each surface's bitset is exact about its own — the
    /// union is allowed to be smaller and that is occlusion, not under-reporting.
    #[cfg(test)]
    pub(crate) fn reported_cells(&self) -> usize {
        self.layers.iter().map(|l| l.surface.damaged_cells()).sum()
    }

    /// Clear every layer's damage. `present` owns this; nothing above the engine can reach it.
    pub(crate) fn clear_damage(&mut self) {
        for layer in &mut self.layers {
            layer.surface.damage_mut().clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Style;

    fn composite(stack: &LayerStack, frame: &mut Surface) {
        stack.union_damage_into(frame);
        let mut runs = Vec::new();
        frame.damage().for_each_run(|r| runs.push(r));
        for r in runs {
            stack.composite_run(frame, r);
        }
    }

    fn glyphs(s: &Surface, y: u16) -> String {
        s.row(y)
            .iter()
            .map(|c| c.grapheme.as_scalar().unwrap_or('?'))
            .collect()
    }

    #[test]
    fn a_new_stack_is_empty() {
        let stack = LayerStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn a_content_layer_paints_into_the_frame() {
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(1, 1, 4, 1), true);
        stack.view(id).unwrap().text(0, 0, "abcd", Style::new());
        let mut frame = Surface::new(8, 3);
        composite(&stack, &mut frame);
        assert_eq!(glyphs(&frame, 1), " abcd   ");
    }

    #[test]
    fn a_new_layer_damages_its_whole_rectangle() {
        let mut stack = LayerStack::new();
        stack.add_content(0, Rect::new(2, 1, 3, 2), true);
        let mut frame = Surface::new(8, 4);
        stack.union_damage_into(&mut frame);
        let mut runs = Vec::new();
        frame.damage().for_each_run(|r| runs.push(r));
        assert_eq!(
            runs,
            vec![Run { y: 1, lo: 2, hi: 4 }, Run { y: 2, lo: 2, hi: 4 }]
        );
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
        let mut frame = Surface::new(8, 1);
        composite(&stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "..###...");
    }

    #[test]
    fn layers_added_out_of_z_order_still_paint_in_z_order() {
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
        let mut frame = Surface::new(4, 1);
        composite(&stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "####");
    }

    #[test]
    fn two_layers_with_identical_z_paint_in_insertion_order() {
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
        let mut frame = Surface::new(4, 1);
        composite(&stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "2222");
    }

    #[test]
    fn a_layer_hanging_off_an_edge_is_intersected_not_rejected() {
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(-2, -1, 4, 3), true);
        stack
            .view(id)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 3), "#", Style::new());
        let mut frame = Surface::new(4, 2);
        composite(&stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), "##  ");
        assert_eq!(glyphs(&frame, 1), "##  ");
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
        // Only the middle two cells of the upper layer are written; the rest stay EMPTY.
        stack.view(high).unwrap().text(1, 0, "ab", Style::new());
        let mut frame = Surface::new(4, 1);
        composite(&stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), ".ab.");
    }

    #[test]
    fn an_opaque_layer_drawn_only_at_its_border_erases_what_is_under_it() {
        // The other half of the same decision: `opaque` is a promise the caller makes, and this is
        // what it costs when it is made carelessly. Spec §5 names it as the real degenerate case.
        let mut stack = LayerStack::new();
        let low = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let high = stack.add_content(1, Rect::new(0, 0, 4, 1), true);
        stack
            .view(low)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), ".", Style::new());
        stack.view(high).unwrap().text(1, 0, "ab", Style::new());
        let mut frame = Surface::new(4, 1);
        composite(&stack, &mut frame);
        assert_eq!(glyphs(&frame, 0), " ab ");
    }

    #[test]
    fn view_of_a_layer_this_stack_did_not_mint_is_none() {
        let mut ours = LayerStack::new();
        let mut theirs = LayerStack::new();
        let stranger = theirs.add_content(0, Rect::new(0, 0, 1, 1), true);
        assert!(ours.view(stranger).is_none());
        let id = ours.add_content(0, Rect::new(0, 0, 1, 1), true);
        assert!(ours.view(id).is_some());
    }

    #[test]
    fn two_layers_standing_apart_leave_the_gap_between_them_undamaged() {
        // The invariant `composite_run` rests on: no damaged run reaches past the layers that
        // caused it, so there is no ground to paint.
        let mut stack = LayerStack::new();
        let left = stack.add_content(0, Rect::new(0, 0, 5, 1), true);
        let right = stack.add_content(0, Rect::new(100, 0, 5, 1), true);
        stack
            .view(left)
            .unwrap()
            .fill(Rect::new(0, 0, 5, 1), "L", Style::new());
        stack
            .view(right)
            .unwrap()
            .fill(Rect::new(0, 0, 5, 1), "R", Style::new());

        let mut frame = Surface::new(300, 1);
        stack.union_damage_into(&mut frame);
        let mut runs = Vec::new();
        frame.damage().for_each_run(|r| runs.push(r));
        assert_eq!(
            runs,
            vec![
                Run { y: 0, lo: 0, hi: 4 },
                Run {
                    y: 0,
                    lo: 100,
                    hi: 104
                }
            ],
            "per-row spans would merge these into one run 105 cells wide"
        );

        for r in runs {
            stack.composite_run(&mut frame, r);
        }
        assert!(
            frame.row(0)[5..100].iter().all(|c| *c == Cell::BLANK),
            "the gap was never damaged, so it was never composited"
        );
    }

    #[test]
    fn clearing_damage_leaves_the_cells_alone() {
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        stack
            .view(id)
            .unwrap()
            .fill(Rect::new(0, 0, 4, 1), "#", Style::new());
        stack.clear_damage();
        let mut frame = Surface::new(4, 1);
        stack.union_damage_into(&mut frame);
        assert!(frame.damage().is_empty());
    }
}
