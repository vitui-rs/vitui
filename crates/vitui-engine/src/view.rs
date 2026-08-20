//! Views and the drawing verbs.
//!
//! A [`Surface`] owns cells and damage; a borrowed `View` draws into it. A view is an origin, a
//! clip region and a content offset, with no cells of its own — its clip stack **is** the call
//! stack, which is what makes zero allocation structural rather than disciplined, and what makes a
//! child unable to widen its parent (spec §4).
//!
//! # Scope
//!
//! `text`, `set`, `fill` and `restyle` at absolute coordinates on the root view, over extended
//! grapheme clusters. `child`, `scrolled` and the visibility queries arrive with ticket 09.

use std::marker::PhantomData;

use crate::cell::{Cell, GraphemeId};
use crate::damage::RowBits;
use crate::geom::Rect;
use crate::restyle::{Restyle, apply};
use crate::style::Style;
use crate::tables::Tables;
use crate::ucd;

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
    /// How many **columns** were written.
    ///
    /// Columns rather than clusters, because a screen of CJK covers 300 columns with 150 clusters
    /// and a caller measuring how far a verb got wants the former.
    ///
    /// It counts columns *written*, not columns *advanced over*: a verb starting left of the clip
    /// consumed clusters that were discarded, so `x + cells` is not the next free column in that
    /// case. That is ADR 0022's clamp-and-discard showing through — the verb reports what it did,
    /// and a caller that needs to resume mid-string has [`bytes`](Written::bytes).
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
/// # What it holds
///
/// The cells, the damage and the handle tables, taken apart rather than reached through a
/// `&mut Surface`. Every write verb needs all three at once and they do not come from one owner:
/// a surface in a layer stack draws into the **stack's** handle space, while a standalone surface
/// draws into its own (spec §3, ticket 19).
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
    cells: &'a mut [Cell],
    damage: &'a mut RowBits,
    /// The owning surface's width, which is the row stride and the clip's outer bound.
    stride: u16,
    /// Absolute, in surface coordinates, already intersected with the surface itself.
    clip: Rect,
    /// What an untouched cell of the surface holds; see [`Row::blank`].
    ground: GraphemeId,
    tables: &'a mut Tables,
    /// `View: !Send`. Zero-sized, private, and the whole mechanism.
    _not_send: PhantomData<*const ()>,
}

/// One row of a view, with the columns the caller is allowed to touch.
///
/// The repair rules reach one column either side of what is being written, and that reach stops at
/// the clip: a view may not widen itself (spec §4), so a pair bisected by the clip edge keeps the
/// half that is outside. With a root view the clip is the surface and the question does not arise;
/// ticket 09 brings `child`, which is what makes it arise, and ticket 11 moves the same rules to
/// composite time where a layer edge *does* reach outside.
struct Row<'r> {
    cells: &'r mut [Cell],
    lo: u16,
    hi: u16,
    /// What a blanked half goes back to: a space, or `EMPTY` in a non-opaque layer.
    ground: GraphemeId,
}

impl<'r> Row<'r> {
    /// The row `y` holds, clipped to what a view may touch. `None` when `y` is outside.
    ///
    /// A free constructor rather than a method on `View`, because a verb needs the row and the
    /// tables at the same time and they are two of the view's fields.
    fn of(
        cells: &'r mut [Cell],
        stride: u16,
        clip: Rect,
        ground: GraphemeId,
        y: i32,
    ) -> Option<Row<'r>> {
        // A clip with no columns has no `hi`, and a verb that reached one would write at `lo`
        // anyway — the left-edge half of rule 4 does exactly that. Zero-sized surfaces are legal
        // (spec §4) and a verb against one is discarded, not a panic (ADR 0022).
        if clip.is_empty() || y < clip.y || y >= clip.bottom() {
            return None;
        }
        let start = y as usize * stride as usize;
        Some(Row {
            cells: &mut cells[start..start + stride as usize],
            lo: clip.x as u16,
            hi: (clip.right() - 1) as u16,
            ground,
        })
    }

    /// Blank one half of a pair, **keeping the cell's own style** (rule 3).
    ///
    /// The background is what the eye notices; a half-erased glyph that also changes colour reads as
    /// a bug even when the text is right.
    ///
    /// It goes back to the surface's **ground**, not to a space. In a non-opaque layer those are
    /// different cells and the difference is the whole of `opaque: false`: this is a cell the caller
    /// never wrote, orphaned by a write next to it, and leaving an opaque space there erases what is
    /// underneath. Rule 4's space is the other case — there the caller *did* ask for that column.
    fn blank(&mut self, x: u16) {
        let cell = &mut self.cells[x as usize];
        *cell = Cell::new(self.ground, cell.style);
    }

    /// Rules 1 and 2: make column `x` safe to write into, and say which columns changed.
    ///
    /// A write landing on a `CONTINUATION` blanks its head at `x - 1`; a write landing on a wide
    /// head blanks its continuation at `x + 1`. Both halves are damaged, which is what deletes
    /// ratatui's two bug-driven workarounds rather than porting them (spec §6).
    fn repair(&mut self, x: u16) -> (u16, u16) {
        let g = self.cells[x as usize].grapheme;
        if g.is_continuation() && x > self.lo {
            self.blank(x - 1);
            return (x - 1, x);
        }
        if g.is_wide_head() && x < self.hi {
            self.blank(x + 1);
            return (x, x + 1);
        }
        (x, x)
    }

    /// Widen `lo..=hi` so that no double-width pair is bisected, or shrink it where the clip
    /// forbids widening. `None` when nothing is left to touch.
    ///
    /// The two edges are independent and each has the same two answers: reach the other half if it
    /// is inside the clip, and give up this half if it is not. Giving up is what keeps a view from
    /// widening itself — the head of a pair the clip starts inside belongs to whoever owns the
    /// cells to the left.
    fn whole_pairs(&self, mut lo: u16, mut hi: u16) -> Option<(u16, u16)> {
        if self.cells[lo as usize].grapheme.is_continuation() {
            if lo > self.lo {
                lo -= 1;
            } else {
                lo += 1;
            }
        }
        if hi >= lo && self.cells[hi as usize].grapheme.is_wide_head() {
            if hi < self.hi {
                hi += 1;
            } else if hi == 0 {
                return None;
            } else {
                hi -= 1;
            }
        }
        if hi < lo { None } else { Some((lo, hi)) }
    }

    /// Write one cell, repairing whatever it lands on first.
    fn put(&mut self, x: u16, cell: Cell) -> (u16, u16) {
        let (lo, hi) = self.repair(x);
        self.cells[x as usize] = cell;
        (lo, hi)
    }
}

impl<'a> View<'a> {
    pub(crate) fn new(
        cells: &'a mut [Cell],
        damage: &'a mut RowBits,
        stride: u16,
        clip: Rect,
        ground: GraphemeId,
        tables: &'a mut Tables,
    ) -> View<'a> {
        View {
            cells,
            damage,
            stride,
            clip,
            ground,
            tables,
            _not_send: PhantomData,
        }
    }

    /// The view's size in cells.
    pub fn size(&self) -> (u16, u16) {
        (self.clip.w, self.clip.h)
    }

    /// Write `s` starting at `(x, y)`, in the given style.
    ///
    /// The string is segmented into extended grapheme clusters (UAX #29) and each is placed in one
    /// cell, advancing by the columns it occupies. A cluster occupying no column — a lone combining
    /// mark, a format character — is stepped over rather than placed, because spec §3's width table
    /// says nothing advances.
    ///
    /// Out-of-bounds writes are discarded silently — no panic, no `Result`, not even a
    /// `debug_assert`. A virtualised component writes far outside a surface as a matter of course,
    /// and that is normal traffic rather than an error (ADR 0022).
    ///
    /// # The five repair rules
    ///
    /// A double-width glyph can never be cut in half by a write (spec §3). Landing on a
    /// `CONTINUATION` blanks its head; landing on a wide head blanks its continuation; a blanked
    /// half keeps the **old** cell's style; a wide glyph that does not fit is written as a space,
    /// never as half a glyph; and a wide glyph landing across an existing pair repairs at both ends
    /// before it writes.
    pub fn text(&mut self, x: i32, y: i32, s: &str, style: Style) -> Written {
        if s.is_empty() {
            return Written {
                cells: 0,
                bytes: 0,
                stop: Stop::Complete,
            };
        }
        let View {
            cells,
            damage,
            stride,
            clip,
            ground,
            tables,
            ..
        } = self;
        let (left, right) = (clip.x, clip.right());
        let Some(mut row) = Row::of(cells, *stride, *clip, *ground, y) else {
            return Written::NONE;
        };

        let mut col = x;
        let mut written = 0u16;
        let mut consumed = 0usize;
        let mut offset = 0usize;
        let mut first = i32::MAX;
        let mut last = i32::MIN;
        let mut clipped = false;
        // Whether anything was refused for being outside the clip, as opposed to for occupying no
        // column. `Stop::Offscreen` is a statement about the clip and must not be returned for a
        // string that never left it.
        let mut discarded = false;

        for cluster in ucd::clusters(s) {
            if col >= right {
                clipped = true;
                discarded = true;
                consumed = offset;
                break;
            }
            let Some(g) = tables.interner.handle(cluster) else {
                // No column, so no cell — and the bytes are still consumed.
                offset += cluster.len();
                consumed = offset;
                continue;
            };
            let width = g.columns() as i32;

            // Rule 4 at the right edge: a wide glyph the clip has no room for is written as a
            // space, never as half a glyph. The cluster is **not** consumed, so a caller that wraps
            // resumes with it, and there is nothing after it that could fit.
            if width == 2 && col + 1 >= right {
                let (lo, hi) = row.put(col as u16, Cell::new(GraphemeId::SPACE, style));
                written += 1;
                first = first.min(lo as i32);
                last = last.max(hi as i32);
                clipped = true;
                consumed = offset;
                break;
            }

            // Rule 4 at the left edge, which does **not** end the verb: the half that landed becomes
            // a space and the rest of the string carries on into the clip. Ending here would drop
            // everything after a glyph the viewport happened to bisect, which is ordinary traffic
            // for a horizontally scrolled component rather than an edge case.
            if width == 2 && col < left && col + 1 == left {
                let (lo, hi) = row.put(left as u16, Cell::new(GraphemeId::SPACE, style));
                written += 1;
                first = first.min(lo as i32);
                last = last.max(hi as i32);
                col += width;
                offset += cluster.len();
                consumed = offset;
                continue;
            }

            if col < left {
                discarded = true;
            } else {
                let at = col as u16;
                // Rule 5: a wide glyph landing across an existing pair repairs at both ends.
                let (lo, hi) = row.put(at, Cell::new(g, style));
                first = first.min(lo as i32);
                last = last.max(hi as i32);
                if width == 2 {
                    let (_, hi) = row.put(at + 1, Cell::new(GraphemeId::CONTINUATION, style));
                    last = last.max(hi as i32);
                }
                written += width as u16;
            }
            col += width;
            offset += cluster.len();
            consumed = offset;
        }

        if first <= last {
            damage.mark(y, first, last);
        }

        Written {
            cells: written,
            bytes: consumed,
            stop: if written == 0 && discarded {
                Stop::Offscreen
            } else if clipped {
                Stop::Clipped
            } else {
                Stop::Complete
            },
        }
    }

    /// Write one cluster at `(x, y)`. The same call as [`text`](Self::text).
    ///
    /// Kept because `set(x, y, "▀", st)` reads better at a chart's call site than the same string
    /// passed to a verb named for prose (spec §4).
    pub fn set(&mut self, x: i32, y: i32, cluster: &str, style: Style) -> Written {
        self.text(x, y, cluster, style)
    }

    /// Fill `r` with `cluster`, in the given style.
    ///
    /// Seventeen times cheaper than the same cells written with [`text`](Self::text), because a
    /// narrow cluster is a `slice::fill` of a sixteen-byte `Copy` with two edge repairs (spec §4).
    /// Expressing a fill as repeated text throws that away.
    ///
    /// A **wide** cluster fills in pairs, and an odd last column gets a space rather than half a
    /// glyph — rule 4 again, at the right edge of the rectangle.
    pub fn fill(&mut self, r: Rect, cluster: &str, style: Style) {
        let Some(first) = ucd::clusters(cluster).next() else {
            return;
        };
        let View {
            cells,
            damage,
            stride,
            clip,
            ground,
            tables,
            ..
        } = self;
        let area = r.intersect(*clip);
        if area.is_empty() {
            return;
        }
        let Some(g) = tables.interner.handle(first) else {
            return;
        };
        let cell = Cell::new(g, style);
        let (lo, hi) = (area.x as u16, (area.right() - 1) as u16);

        for y in area.y..area.bottom() {
            let mut row =
                Row::of(cells, *stride, *clip, *ground, y).expect("the area is in the clip");
            // The two edge repairs, before anything is written: a pair bisected by either edge of
            // the rectangle loses its other half here rather than being left in halves.
            let (dlo, _) = row.repair(lo);
            let (_, dhi) = row.repair(hi);

            if g.columns() == 1 {
                row.cells[lo as usize..=hi as usize].fill(cell);
            } else {
                let mut x = lo;
                while x < hi {
                    row.cells[x as usize] = cell;
                    row.cells[x as usize + 1] = Cell::new(GraphemeId::CONTINUATION, style);
                    x += 2;
                }
                if x == hi {
                    row.cells[hi as usize] = Cell::new(GraphemeId::SPACE, style);
                }
            }
            damage.mark(y, dlo as i32, dhi as i32);
        }
    }

    /// Change how the cells in `r` are painted, without touching what is drawn there.
    ///
    /// Moving a selection bar one row costs **205 ns against 1.36 µs** for redrawing the same row,
    /// 6.6x (spec §4). Without this verb, changing a background means re-segmenting UTF-8 and
    /// re-interning every cluster on the row to write back the text that was already there.
    ///
    /// # What it promises
    ///
    /// **It rewrites what the descriptor names and preserves every channel it does not, on either
    /// side of the extended bit** — so a shadow falling across a hyperlink darkens it instead of
    /// deleting it. A descriptor that clears both extended channels puts the cell back inline:
    /// *extended is a cost, not a state*.
    ///
    /// # The memo, which is the whole implementation
    ///
    /// Cells in a run are contiguous and share a `u64`, so one entry remembering the previous
    /// style word turns a per-cell table round trip into a per-*distinct-style* one: 12.65 µs for a
    /// full screen against 24.51 µs unmemoised, and 75.49 against 289.19 on a screen where every
    /// cell is hyperlinked. **It is free where nothing is extended** — 12.65 against the 13.02 µs
    /// of an inline mask that skips extended cells and is therefore wrong.
    ///
    /// Two fast paths were built beside it and both were refused, so that nobody re-derives them:
    /// a per-cell branch taking the mask on inline cells (13.10 / 17.43 / 90.00 µs — it loses,
    /// because the branch breaks the mask loop's vectorisation) and a surface-level *contains no
    /// extended cell* gate (12.38 / 17.44 / 75.61 µs — indistinguishable from the memo alone).
    ///
    /// # Double-width pairs are restyled as units
    ///
    /// A continuation carries its head's style and nothing else: the serializer emits the head and
    /// skips the continuation, so the terminal paints both columns from one SGR. Restyling one half
    /// of a pair would leave the frame saying something the wire cannot express. A pair bisected by
    /// the rectangle is therefore restyled whole — and left alone when its other half is outside
    /// the clip, because a view may not widen itself (spec §4).
    pub fn restyle(&mut self, r: Rect, d: &Restyle) {
        let View {
            cells,
            damage,
            stride,
            clip,
            ground,
            tables,
            ..
        } = self;
        let area = r.intersect(*clip);
        if area.is_empty() {
            return;
        }
        // One entry, on the *input* style word. Runs of equal style are what a drawing verb
        // produces, so this hits on nearly every cell and misses once per distinct style.
        let mut memo: Option<(Style, Style)> = None;

        for y in area.y..area.bottom() {
            let row = Row::of(cells, *stride, *clip, *ground, y).expect("the area is in the clip");
            let Some((lo, hi)) = row.whole_pairs(area.x as u16, (area.right() - 1) as u16) else {
                continue;
            };
            for cell in &mut row.cells[lo as usize..=hi as usize] {
                let old = cell.style;
                cell.style = match memo {
                    Some((was, now)) if was == old => now,
                    _ => {
                        let now = apply(tables, d, old);
                        memo = Some((old, now));
                        now
                    }
                };
            }
            damage.mark(y, lo as i32, hi as i32);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::damage::Run;
    use crate::layer::LayerStack;
    use crate::style::Color;
    use crate::surface::Surface;

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

    /// What a row would look like on a terminal: every cell's cluster, continuations dropped.
    fn rendered(s: &Surface, y: u16) -> String {
        let mut out = String::new();
        for c in s.row(y) {
            if c.grapheme.is_continuation() {
                continue;
            }
            out.push_str(
                &s.tables()
                    .interner
                    .resolve(c.grapheme)
                    .unwrap_or_else(|| "?".to_owned()),
            );
        }
        out
    }

    #[test]
    fn text_puts_one_cluster_in_one_cell_however_many_scalars_it_has() {
        let mut s = Surface::new(8, 1);
        // Two clusters, three scalars, four bytes: `e` with a combining acute, then `x`.
        let w = s.root().text(0, 0, "e\u{301}x", Style::new());
        assert_eq!(w.cells, 2, "two columns, not three scalars");
        assert_eq!(w.bytes, 4);
        assert_eq!(rendered(&s, 0), "e\u{301}x      ");
    }

    #[test]
    fn a_wide_cluster_takes_two_columns_and_the_second_is_a_continuation() {
        let mut s = Surface::new(8, 1);
        let w = s.root().text(0, 0, "漢字", Style::new());
        assert_eq!(w.cells, 4, "two clusters, four columns");
        assert!(s.row(0)[0].grapheme.is_wide_head());
        assert!(s.row(0)[1].grapheme.is_continuation());
        assert!(s.row(0)[2].grapheme.is_wide_head());
        assert!(s.row(0)[3].grapheme.is_continuation());
        assert_eq!(rendered(&s, 0), "漢字    ");
    }

    #[test]
    fn a_full_row_of_cjk_interns_nothing() {
        // Spec §4: a full screen of CJK is *cheaper* than one of Latin, which is only true if no
        // table is touched. A wide scalar is still its own handle (§3, ticket 19).
        let mut s = Surface::new(300, 1);
        let row: String = std::iter::repeat_n('漢', 150).collect();
        s.root().text(0, 0, &row, Style::new());
        assert!(s.tables().interner.is_empty());
    }

    /// The invariant of spec §3, over a whole surface: a `CONTINUATION` never appears without a wide
    /// head immediately to its left, and a wide head is always followed by a `CONTINUATION`.
    fn assert_pairing_holds(s: &Surface) {
        let (w, h) = s.size();
        for y in 0..h {
            let row = s.row(y);
            for x in 0..w as usize {
                let g = row[x].grapheme;
                if g.is_continuation() {
                    assert!(x > 0, "a continuation in column 0 at row {y}");
                    assert!(
                        row[x - 1].grapheme.is_wide_head(),
                        "a continuation at ({x}, {y}) with no wide head to its left"
                    );
                }
                if g.is_wide_head() {
                    assert!(
                        x + 1 < w as usize,
                        "a wide head in the last column at row {y}"
                    );
                    assert!(
                        row[x + 1].grapheme.is_continuation(),
                        "a wide head at ({x}, {y}) with no continuation after it"
                    );
                }
            }
        }
    }

    #[test]
    fn rule_1_a_write_landing_on_a_continuation_blanks_its_head_and_damages_both() {
        let mut s = Surface::new(8, 1);
        s.root().text(2, 0, "漢", Style::new());
        s.damage_mut().clear();
        s.root().text(3, 0, "x", Style::new());
        assert_eq!(rendered(&s, 0), "   x    ", "the head at 2 was blanked");
        assert_eq!(runs(&s), vec![Run { y: 0, lo: 2, hi: 3 }], "both damaged");
        assert_pairing_holds(&s);
    }

    #[test]
    fn rule_2_a_write_landing_on_a_wide_head_blanks_its_continuation_and_damages_both() {
        let mut s = Surface::new(8, 1);
        s.root().text(2, 0, "漢", Style::new());
        s.damage_mut().clear();
        s.root().text(2, 0, "x", Style::new());
        assert_eq!(
            rendered(&s, 0),
            "  x     ",
            "the continuation at 3 was blanked"
        );
        assert_eq!(runs(&s), vec![Run { y: 0, lo: 2, hi: 3 }], "both damaged");
        assert_pairing_holds(&s);
    }

    #[test]
    fn rule_3_the_blanked_half_keeps_the_old_style_not_the_writers() {
        // The background is what the eye notices: a half-erased glyph that also changes colour reads
        // as a bug even when the text is right.
        let old = Style::new().bg(Color::indexed(4));
        let writer = Style::new().bg(Color::indexed(1));
        let mut s = Surface::new(8, 1);
        s.root().text(2, 0, "漢", old);
        s.root().text(3, 0, "x", writer);
        assert_eq!(
            s.row(0)[2].style,
            old,
            "the blanked head kept the old style"
        );
        assert_eq!(s.row(0)[3].style, writer);
    }

    #[test]
    fn rule_4_a_wide_glyph_that_does_not_fit_the_surface_is_written_as_a_space() {
        let mut s = Surface::new(4, 1);
        let w = s.root().text(3, 0, "漢字", Style::new());
        assert_eq!(rendered(&s, 0), "    ", "a space, never half a glyph");
        assert_eq!(w.cells, 1);
        assert_eq!(
            w.bytes, 0,
            "the caller resumes with the cluster that did not fit"
        );
        assert_eq!(w.stop, Stop::Clipped);
        assert_pairing_holds(&s);
    }

    #[test]
    fn rule_4_a_wide_glyph_that_does_not_fit_the_clip_is_written_as_a_space() {
        // A clip narrower than the surface. Ticket 09 is what replaces this hand-built view with
        // `child`; the rule it exercises is the same one at the same edge.
        let mut s = Surface::new(8, 1);
        let w = s
            .clipped(Rect::new(1, 0, 3, 1))
            .text(3, 0, "漢", Style::new());
        assert_eq!(w.cells, 1);
        assert_eq!(w.stop, Stop::Clipped);
        assert_eq!(
            rendered(&s, 0),
            "        ",
            "a space at column 3, not a head"
        );
        assert!(!s.row(0)[3].grapheme.is_wide_head());
        assert!(
            !s.row(0)[4].grapheme.is_continuation(),
            "column 4 is outside the clip and was not touched"
        );
    }

    #[test]
    fn rule_4_a_wide_glyph_straddling_the_left_edge_is_written_as_a_space() {
        let mut s = Surface::new(4, 1);
        let w = s.root().text(-1, 0, "漢xy", Style::new());
        assert_eq!(
            rendered(&s, 0),
            " xy ",
            "the half that landed became a space, and the rest carried on"
        );
        assert_eq!(w.cells, 3);
        assert_eq!(w.bytes, 5, "every cluster was consumed");
        assert_eq!(
            w.stop,
            Stop::Complete,
            "a glyph bisected by the left edge does not end the verb — everything after it fits"
        );
    }

    #[test]
    fn rule_5_a_wide_glyph_landing_across_a_pair_repairs_at_both_ends() {
        let mut s = Surface::new(8, 1);
        s.root().text(0, 0, "漢字", Style::new());
        s.damage_mut().clear();
        // Lands on the continuation of the first pair and the head of the second.
        s.root().text(1, 0, "漢", Style::new());
        assert_eq!(rendered(&s, 0), " 漢     ", "both neighbours became spaces");
        assert_eq!(runs(&s), vec![Run { y: 0, lo: 0, hi: 3 }]);
        assert_pairing_holds(&s);
    }

    #[test]
    fn three_adversarial_overwrite_passes_of_mixed_cjk_keep_every_pair_whole() {
        // The gate spec §3 asks for. Deterministic rather than random: a property that fails one
        // time in ten is a property nobody fixes.
        let mut s = Surface::new(300, 80);
        let alphabet = [
            "a",
            "漢",
            "e\u{301}",
            "字",
            " ",
            "\u{1F468}\u{200D}\u{1F469}",
        ];
        let mut seed = 0x2545_F491u32;
        let mut next = move || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed
        };
        for _ in 0..3 {
            for y in 0..80i32 {
                let mut x = (next() % 300) as i32 - 4;
                while x < 300 {
                    let word = alphabet[(next() % alphabet.len() as u32) as usize];
                    s.root().text(x, y, word, Style::new());
                    x += 1 + (next() % 5) as i32;
                }
            }
            assert_pairing_holds(&s);
        }
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
    fn cells_counts_columns_written_not_columns_advanced_over() {
        // The half of the contract that is easy to misread: a verb clipped on the left discarded
        // columns it still walked past, so `x + cells` is not where the next verb goes.
        let mut s = Surface::new(8, 1);
        let w = s.root().text(-4, 0, "漢字漢字", Style::new());
        assert_eq!(w.cells, 4, "four columns landed; four were discarded");
        assert_eq!(w.stop, Stop::Complete);
        assert_eq!(rendered(&s, 0), "漢字    ");
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
    fn a_string_that_occupies_no_columns_is_complete_rather_than_offscreen() {
        // `Stop::Offscreen` says the verb fell outside the clip. A lone combining acute never left
        // it — it simply has no column to be placed in — and a caller that reads `Offscreen` as
        // "stop drawing this row" would abandon a row it should have kept.
        let mut s = Surface::new(8, 1);
        let w = s.root().text(0, 0, "\u{301}", Style::new());
        assert_eq!(w.cells, 0);
        assert_eq!(w.bytes, 2, "consumed, just not placed");
        assert_eq!(w.stop, Stop::Complete);
        assert!(s.damage().is_empty());
    }

    #[test]
    fn a_repair_inside_a_non_opaque_layer_restores_transparency_not_a_space() {
        // `opaque: false` exists so an overlay does not erase what is under it (spec §5). A repair
        // that blanks half a pair to an opaque space punches exactly the hole the flag was added to
        // prevent — and the caller never asked for that cell at all.
        let mut stack = LayerStack::new();
        let low = stack.add_content(0, Rect::new(0, 0, 6, 1), true);
        let high = stack.add_content(1, Rect::new(0, 0, 6, 1), false);
        stack
            .view(low)
            .unwrap()
            .fill(Rect::new(0, 0, 6, 1), ".", Style::new());
        let mut v = stack.view(high).unwrap();
        v.text(0, 0, "漢", Style::new());
        v.text(0, 0, "x", Style::new());

        let mut frame = Surface::new(6, 1);
        stack.union_damage_into(&mut frame);
        let mut runs = Vec::new();
        frame.damage().for_each_run(|r| runs.push(r));
        for r in runs {
            stack.composite_run(&mut frame, r);
        }
        assert_eq!(
            glyphs(&frame, 0),
            "x.....",
            "the continuation's cell went back to transparent, not to an opaque space"
        );
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
    fn a_zero_width_surface_discards_every_verb_instead_of_panicking() {
        // ADR 0022: out of bounds is discarded silently, and a zero-sized surface is legal
        // (`surface::tests::a_zero_sized_surface_is_legal_and_holds_nothing`). The left-edge half of
        // rule 4 wrote its space at the clip's own left column without asking whether the clip had
        // one.
        let mut s = Surface::new(0, 1);
        assert_eq!(s.root().text(-1, 0, "漢", Style::new()), Written::NONE);
        assert_eq!(s.root().text(0, 0, "x", Style::new()), Written::NONE);
        s.root().fill(Rect::new(0, 0, 4, 1), "漢", Style::new());
        assert!(s.damage().is_empty());
    }

    #[test]
    fn a_root_view_is_the_whole_surface() {
        let mut s = Surface::new(300, 80);
        assert_eq!(s.root().size(), (300, 80));
    }

    // ---- restyle -----------------------------------------------------------------------------

    fn styles(s: &Surface, y: u16) -> Vec<Style> {
        s.row(y).iter().map(|c| c.style).collect()
    }

    #[test]
    fn restyle_changes_how_cells_are_painted_and_not_what_is_drawn() {
        let mut s = Surface::new(8, 1);
        s.root().text(0, 0, "abcdefgh", Style::new());
        s.damage_mut().clear();
        s.root().restyle(
            Rect::new(2, 0, 3, 1),
            &Restyle {
                bg: Some(Color::indexed(4)),
                ..Default::default()
            },
        );
        assert_eq!(glyphs(&s, 0), "abcdefgh", "nothing moved");
        let want = Style::new().bg(Color::indexed(4));
        assert_eq!(
            styles(&s, 0),
            vec![
                Style::new(),
                Style::new(),
                want,
                want,
                want,
                Style::new(),
                Style::new(),
                Style::new()
            ]
        );
    }

    #[test]
    fn restyle_marks_damage_over_the_span_it_touched_and_no_further() {
        let mut s = Surface::new(8, 2);
        s.root().fill(Rect::new(0, 0, 8, 2), ".", Style::new());
        s.damage_mut().clear();
        s.root().restyle(
            Rect::new(2, 1, 3, 1),
            &Restyle {
                set: Restyle::BOLD,
                ..Default::default()
            },
        );
        assert_eq!(runs(&s), vec![Run { y: 1, lo: 2, hi: 4 }]);
    }

    #[test]
    fn restyle_is_clamped_to_the_clip_rather_than_refused() {
        // ADR 0022: a verb reaching outside is ordinary traffic for a virtualised component.
        let mut s = Surface::new(4, 1);
        s.root().restyle(
            Rect::new(-100, 0, 1000, 1),
            &Restyle {
                set: Restyle::BOLD,
                ..Default::default()
            },
        );
        assert_eq!(runs(&s), vec![Run { y: 0, lo: 0, hi: 3 }]);
        assert!(styles(&s, 0).iter().all(|st| *st == Style::new().bold()));
    }

    #[test]
    fn restyle_entirely_outside_the_clip_touches_nothing() {
        let mut s = Surface::new(4, 1);
        s.root().restyle(
            Rect::new(100, 0, 4, 1),
            &Restyle {
                set: Restyle::BOLD,
                ..Default::default()
            },
        );
        assert!(s.damage().is_empty());
    }

    #[test]
    fn restyle_takes_a_bisected_pair_whole_at_both_edges() {
        // A continuation carries its head's style, because the serializer emits the head and skips
        // the continuation. Half a restyled pair would be a frame saying something the wire cannot.
        let mut s = Surface::new(8, 1);
        s.root().text(0, 0, "漢字漢字", Style::new());
        s.damage_mut().clear();
        s.root().restyle(
            Rect::new(1, 0, 4, 1),
            &Restyle {
                set: Restyle::BOLD,
                ..Default::default()
            },
        );
        let bold = Style::new().bold();
        assert_eq!(
            styles(&s, 0),
            vec![
                bold,
                bold,
                bold,
                bold,
                bold,
                bold,
                Style::new(),
                Style::new()
            ],
            "the pair at 0..1 and the pair at 4..5 both came in whole"
        );
        assert_eq!(runs(&s), vec![Run { y: 0, lo: 0, hi: 5 }]);
    }

    #[test]
    fn restyle_leaves_a_pair_alone_when_the_clip_forbids_reaching_its_other_half() {
        // A view may not widen itself (spec §4): the head at column 1 belongs to whoever owns the
        // cells left of the clip, so the continuation at column 2 is left as it is rather than
        // restyled into a pair that disagrees with itself.
        let mut s = Surface::new(8, 1);
        s.root().text(0, 0, "a漢b", Style::new());
        s.damage_mut().clear();
        s.clipped(Rect::new(2, 0, 2, 1)).restyle(
            Rect::new(0, 0, 8, 1),
            &Restyle {
                set: Restyle::BOLD,
                ..Default::default()
            },
        );
        assert_eq!(s.row(0)[1].style, Style::new(), "the head, out of reach");
        assert_eq!(s.row(0)[2].style, Style::new(), "so its continuation too");
        assert_eq!(
            s.row(0)[3].style,
            Style::new().bold(),
            "and `b` is restyled"
        );
        assert_eq!(runs(&s), vec![Run { y: 0, lo: 3, hi: 3 }]);
    }

    #[test]
    fn restyle_over_one_column_that_is_a_stranded_continuation_touches_nothing() {
        let mut s = Surface::new(8, 1);
        s.root().text(0, 0, "漢", Style::new());
        s.damage_mut().clear();
        s.clipped(Rect::new(1, 0, 1, 1)).restyle(
            Rect::new(1, 0, 1, 1),
            &Restyle {
                set: Restyle::BOLD,
                ..Default::default()
            },
        );
        assert!(s.damage().is_empty());
        assert_eq!(s.row(0)[1].style, Style::new());
    }

    #[test]
    fn restyle_carries_a_hyperlink_across_a_shadow_falling_over_it() {
        // The whole reason this verb exists rather than a `Style` method: `with_fg_bg` would have
        // cleared bit 63 and overwritten the handle, deleting the hyperlink with nothing to see.
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(0, 0, 4, 1), true);
        let link = stack.tables_mut().link("https://example.com/");
        {
            let mut v = stack.view(id).unwrap();
            v.text(0, 0, "abcd", Style::new());
            v.restyle(
                Rect::new(0, 0, 4, 1),
                &Restyle {
                    link: Some(link),
                    ..Default::default()
                },
            );
            v.restyle(
                Rect::new(1, 0, 2, 1),
                &Restyle {
                    bg: Some(Color::indexed(0)),
                    ..Default::default()
                },
            );
        }
        let mut frame = Surface::new(4, 1);
        stack.union_damage_into(&mut frame);
        let mut all = Vec::new();
        frame.damage().for_each_run(|r| all.push(r));
        for r in all {
            stack.composite_run(&mut frame, r);
        }
        for x in 0..4usize {
            let handle = frame.row(0)[x].style.ext_handle().expect("still extended");
            let e = stack.tables().exts.get(handle).unwrap();
            assert_eq!(e.link, link, "column {x} kept its hyperlink");
        }
        assert_eq!(
            stack
                .tables()
                .exts
                .get(frame.row(0)[1].style.ext_handle().unwrap())
                .unwrap()
                .bg,
            Color::indexed(0),
            "and the shadow still landed"
        );
    }

    #[test]
    fn restyle_mints_one_table_entry_per_distinct_style_and_not_one_per_cell() {
        // The memo, as a count rather than as a stopwatch: a full row shares one style word, so the
        // table is reached once however many cells the verb covers.
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(0, 0, 300, 1), true);
        let link = stack.tables_mut().link("https://example.com/");
        {
            let mut v = stack.view(id).unwrap();
            v.fill(Rect::new(0, 0, 300, 1), ".", Style::new());
            v.restyle(
                Rect::new(0, 0, 300, 1),
                &Restyle {
                    link: Some(link),
                    ..Default::default()
                },
            );
        }
        assert_eq!(stack.tables().exts.len(), 1);
    }

    #[test]
    fn restyling_the_same_way_twice_mints_nothing_the_second_time() {
        // What makes a settled operator converge after one frame.
        let mut stack = LayerStack::new();
        let id = stack.add_content(0, Rect::new(0, 0, 8, 1), true);
        let link = stack.tables_mut().link("https://example.com/");
        let d = Restyle {
            link: Some(link),
            ..Default::default()
        };
        for _ in 0..10 {
            stack.view(id).unwrap().restyle(Rect::new(0, 0, 8, 1), &d);
        }
        assert_eq!(stack.tables().exts.len(), 1);
    }

    #[test]
    fn a_restyle_that_names_nothing_extended_never_reaches_the_table() {
        let mut s = Surface::new(300, 1);
        s.root().fill(Rect::new(0, 0, 300, 1), ".", Style::new());
        s.root().restyle(
            Rect::new(0, 0, 300, 1),
            &Restyle {
                bg: Some(Color::indexed(4)),
                set: Restyle::BOLD,
                ..Default::default()
            },
        );
        assert!(
            s.tables().exts.is_empty(),
            "extended is a cost, not a state"
        );
    }
}
