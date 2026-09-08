//! Damage: a per-row bitset with a damaged-row summary.
//!
//! Marked once per drawing verb, cleared by `present`, scanned in row order into `(y, lo, hi)`
//! runs. Exact by construction — it can never report a cell the frame did not change.
//!
//! # Why a bitset, when the whole field uses rows
//!
//! tcell, ghostty, Zellij, notcurses and termwiz all take the row as the unit and call cell-level
//! tracking more precision than the bookkeeping is worth. That is declined, and the reason is
//! the metric rather than taste: **the metric is overdraw, not marking speed.** Marking happens
//! once per verb and scanning once per frame, and both are nanoseconds. What costs is cells the
//! serializer emits that did not change, because those are bytes on the wire and per-terminal
//! force-flush limits go as low as 150 ms.
//!
//! | scene                         | cells changed | RowSpans   | RowBits   | SpanList |
//! |-------------------------------|---------------|------------|-----------|----------|
//! | blinking cursor               | 1             | 1.00x      | 1.00x     | 1.00x    |
//! | scrolling list                | 24 000        | 1.00x      | 1.00x     | 1.00x    |
//! | twenty stacked popups         | 16 898        | 1.00x      | 1.00x     | 1.00x    |
//! | three dialogs standing apart  | 1 941         | **2.53x**  | 1.00x     | 1.00x    |
//! | sub-cell chart, 400 points    | 396           | **37.07x** | 1.00x     | 4.62x    |
//!
//! The bitset wins by 4.3x on the chart and 2.2x on the dialogs, and it wins **despite emitting
//! five times as many runs** — 393 cursor moves against 79. The cursor moves are cheap; the 14 283
//! unchanged cells the span model carries with them are not. More runs is not worse.
//!
//! The damaged-row summary is what makes an idle frame free: `clear` touches only rows that were
//! damaged, and "is anything damaged" is one scan of a couple of words.
//!
//! **The word is *damaged* and not *dirty*.** `CONTEXT.md` gives this concept the name **Damage**,
//! and *dirty* is the ratatui-lineage synonym for a full-buffer diff — the mechanism this module
//! exists instead of. Four sentences here and one test name used it.

use crate::geom::Rect;

/// One damaged span on one row, inclusive at both ends.
///
/// The unit damage is reported in and the only shape the serializer ever sees. Runs arrive in row
/// order, ascending by column within a row, and that order *is* the order bytes are written in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Run {
    pub(crate) y: u16,
    pub(crate) lo: u16,
    pub(crate) hi: u16,
}

impl Run {
    /// How many cells the run covers.
    pub(crate) const fn len(self) -> usize {
        (self.hi - self.lo) as usize + 1
    }
}

/// A per-row bitset with a damaged-row summary.
#[derive(Clone, Debug)]
pub(crate) struct RowBits {
    width: u16,
    height: u16,
    words_per_row: usize,
    bits: Vec<u64>,
    /// One bit per row: whether that row has anything set. Keeps `clear` and the idle check off
    /// the full bitset.
    summary: Vec<u64>,
}

impl RowBits {
    pub(crate) fn new(width: u16, height: u16) -> RowBits {
        let words_per_row = words_for(width);
        RowBits {
            width,
            height,
            words_per_row,
            bits: vec![0; words_per_row * height as usize],
            summary: vec![0; words_for(height)],
        }
    }

    /// Whether nothing is damaged. This is what the idle path interrogates, so it reads the
    /// summary and never the bitset.
    pub(crate) fn is_empty(&self) -> bool {
        self.summary.iter().all(|&w| w == 0)
    }

    /// Mark `lo..=hi` on row `y` as damaged, clamping to the bitset and discarding what falls
    /// outside it. Coordinates are signed because a verb writing far outside a surface is ordinary
    /// traffic for a virtualised component, not an error.
    pub(crate) fn mark(&mut self, y: i32, lo: i32, hi: i32) {
        if y < 0 || y >= self.height as i32 || hi < lo || self.width == 0 {
            return;
        }
        let last = self.width as i32 - 1;
        if hi < 0 || lo > last {
            return;
        }
        let lo = lo.max(0) as usize;
        let hi = hi.min(last) as usize;
        let y = y as usize;

        let row = &mut self.bits[y * self.words_per_row..(y + 1) * self.words_per_row];
        set_range(row, lo, hi);
        self.summary[y / 64] |= 1 << (y % 64);
    }

    /// Mark every cell. What a resize does, and what the first frame after one does.
    pub(crate) fn mark_all(&mut self) {
        for y in 0..self.height as i32 {
            self.mark(y, 0, self.width as i32 - 1);
        }
    }

    /// Clear every damaged row, and return how many rows were touched.
    ///
    /// The count is the gate: an idle frame's clear must touch zero rows. That is what the summary
    /// buys, and without the count nothing observes it.
    pub(crate) fn clear(&mut self) -> usize {
        let mut touched = 0;
        for (wi, word) in self.summary.iter_mut().enumerate() {
            let mut w = *word;
            while w != 0 {
                let bit = w.trailing_zeros() as usize;
                w &= w - 1;
                let y = wi * 64 + bit;
                self.bits[y * self.words_per_row..(y + 1) * self.words_per_row].fill(0);
                touched += 1;
            }
            *word = 0;
        }
        touched
    }

    /// Call `f` with every run, in row order, ascending by column within a row.
    pub(crate) fn for_each_run(&self, mut f: impl FnMut(Run)) {
        for (wi, &word) in self.summary.iter().enumerate() {
            let mut w = word;
            while w != 0 {
                let bit = w.trailing_zeros() as usize;
                w &= w - 1;
                let y = wi * 64 + bit;
                let row = &self.bits[y * self.words_per_row..(y + 1) * self.words_per_row];
                scan_row(row, self.width, y as u16, &mut f);
            }
        }
    }

    /// Union `src`'s damage into this one, translated by `(dx, dy)` and clipped to `clip`.
    ///
    /// A layer hanging off an edge is clipped, not dropped.
    pub(crate) fn union_translated(&mut self, src: &RowBits, dx: i32, dy: i32, clip: Rect) {
        // `src` and `self` are distinct borrows, so the marking happens inside the scan rather
        // than through a collected `Vec` — this runs on every frame and the frame budget is zero
        // allocations.
        src.for_each_run(|r| {
            let y = r.y as i32 + dy;
            if y < clip.y || y >= clip.bottom() {
                return;
            }
            let lo = (r.lo as i32 + dx).max(clip.x);
            let hi = (r.hi as i32 + dx).min(clip.right() - 1);
            self.mark(y, lo, hi);
        });
    }
}

const fn words_for(bits: u16) -> usize {
    (bits as usize).div_ceil(64)
}

fn set_range(row: &mut [u64], lo: usize, hi: usize) {
    let (first, last) = (lo / 64, hi / 64);
    if first == last {
        row[first] |= mask(lo % 64, hi % 64);
        return;
    }
    row[first] |= mask(lo % 64, 63);
    for w in &mut row[first + 1..last] {
        *w = u64::MAX;
    }
    row[last] |= mask(0, hi % 64);
}

/// Bits `lo..=hi` of one word, both within 0..=63.
const fn mask(lo: usize, hi: usize) -> u64 {
    let width = (hi - lo + 1) as u32;
    if width == 64 {
        u64::MAX
    } else {
        ((1u64 << width) - 1) << lo
    }
}

fn scan_row(row: &[u64], width: u16, y: u16, f: &mut impl FnMut(Run)) {
    let last_col = width.saturating_sub(1);
    let mut open: Option<u32> = None;
    for (wi, &word) in row.iter().enumerate() {
        let base = (wi * 64) as u32;
        if word == 0 {
            if let Some(lo) = open.take() {
                f(Run {
                    y,
                    lo: lo as u16,
                    hi: (base - 1) as u16,
                });
            }
            continue;
        }
        let mut rest = word;
        let mut consumed = 0u32;
        while rest != 0 {
            let zeros = rest.trailing_zeros();
            if zeros > 0 {
                if let Some(lo) = open.take() {
                    f(Run {
                        y,
                        lo: lo as u16,
                        hi: (base + consumed - 1) as u16,
                    });
                }
                rest >>= zeros;
                consumed += zeros;
            }
            let ones = rest.trailing_ones();
            let lo = open.take().unwrap_or(base + consumed);
            if consumed + ones == 64 {
                // The run reaches the end of this word and may continue into the next one, so it
                // stays open rather than being reported here.
                open = Some(lo);
                rest = 0;
                consumed = 64;
            } else {
                f(Run {
                    y,
                    lo: lo as u16,
                    hi: (base + consumed + ones - 1) as u16,
                });
                rest >>= ones;
                consumed += ones;
            }
        }
    }
    if let Some(lo) = open {
        f(Run {
            y,
            lo: lo as u16,
            hi: last_col,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vitui_bench::Bench;

    fn runs(d: &RowBits) -> Vec<Run> {
        let mut out = Vec::new();
        d.for_each_run(|r| out.push(r));
        out
    }

    fn run(y: u16, lo: u16, hi: u16) -> Run {
        Run { y, lo, hi }
    }

    #[test]
    fn a_fresh_bitset_is_empty() {
        assert!(RowBits::new(300, 80).is_empty());
        assert!(runs(&RowBits::new(300, 80)).is_empty());
    }

    #[test]
    fn one_marked_cell_is_one_run() {
        let mut d = RowBits::new(300, 80);
        d.mark(4, 7, 7);
        assert_eq!(runs(&d), vec![run(4, 7, 7)]);
    }

    #[test]
    fn two_spans_on_one_row_stay_two_runs() {
        // Spec §6: two popups 100 columns apart are 180 emitted cells with the bitset and 280 with
        // per-row spans. This is the case that decides it.
        let mut d = RowBits::new(300, 80);
        d.mark(0, 0, 89);
        d.mark(0, 190, 279);
        assert_eq!(runs(&d), vec![run(0, 0, 89), run(0, 190, 279)]);
    }

    #[test]
    fn adjacent_spans_merge_into_one_run() {
        let mut d = RowBits::new(300, 80);
        d.mark(0, 0, 9);
        d.mark(0, 10, 19);
        assert_eq!(runs(&d), vec![run(0, 0, 19)]);
    }

    #[test]
    fn a_run_spanning_a_word_boundary_is_one_run() {
        let mut d = RowBits::new(300, 80);
        d.mark(0, 60, 70);
        assert_eq!(runs(&d), vec![run(0, 60, 70)]);
    }

    #[test]
    fn a_run_covering_a_whole_word_is_one_run() {
        let mut d = RowBits::new(300, 80);
        d.mark(0, 0, 191);
        assert_eq!(runs(&d), vec![run(0, 0, 191)]);
    }

    #[test]
    fn a_full_row_is_one_run() {
        let mut d = RowBits::new(300, 80);
        d.mark(3, 0, 299);
        assert_eq!(runs(&d), vec![run(3, 0, 299)]);
    }

    #[test]
    fn a_row_whose_width_is_a_multiple_of_the_word_is_one_run() {
        let mut d = RowBits::new(128, 4);
        d.mark(1, 0, 127);
        assert_eq!(runs(&d), vec![run(1, 0, 127)]);
    }

    #[test]
    fn runs_arrive_in_row_order() {
        let mut d = RowBits::new(300, 80);
        d.mark(70, 1, 2);
        d.mark(3, 1, 2);
        d.mark(65, 1, 2);
        assert_eq!(
            runs(&d),
            vec![run(3, 1, 2), run(65, 1, 2), run(70, 1, 2)],
            "the summary spans two words, and both must be walked in order"
        );
    }

    #[test]
    fn a_mark_outside_the_surface_is_discarded() {
        let mut d = RowBits::new(10, 4);
        d.mark(-1, 0, 9);
        d.mark(4, 0, 9);
        d.mark(0, 10, 20);
        d.mark(0, -20, -1);
        assert!(d.is_empty());
    }

    #[test]
    fn a_mark_straddling_an_edge_is_clamped() {
        let mut d = RowBits::new(10, 4);
        d.mark(0, -5, 3);
        d.mark(1, 7, 40);
        assert_eq!(runs(&d), vec![run(0, 0, 3), run(1, 7, 9)]);
    }

    #[test]
    fn clear_touches_only_the_rows_that_were_damaged() {
        let mut d = RowBits::new(300, 80);
        d.mark(0, 0, 0);
        d.mark(79, 0, 0);
        assert_eq!(d.clear(), 2);
        assert!(d.is_empty());
    }

    #[test]
    fn an_idle_frames_clear_touches_zero_rows() {
        // The gate spec §6 asks for. Without the summary this would be 80.
        let mut d = RowBits::new(300, 80);
        assert_eq!(d.clear(), 0);
    }

    #[test]
    fn clearing_twice_touches_nothing_the_second_time() {
        let mut d = RowBits::new(300, 80);
        d.mark_all();
        assert_eq!(d.clear(), 80);
        assert_eq!(d.clear(), 0);
    }

    #[test]
    fn mark_all_covers_every_cell() {
        let mut d = RowBits::new(7, 3);
        d.mark_all();
        assert_eq!(runs(&d), vec![run(0, 0, 6), run(1, 0, 6), run(2, 0, 6)]);
    }

    #[test]
    fn a_union_translates_into_the_targets_coordinates() {
        let mut src = RowBits::new(10, 3);
        src.mark(0, 0, 9);
        let mut dst = RowBits::new(80, 24);
        dst.union_translated(&src, 5, 2, Rect::new(0, 0, 80, 24));
        assert_eq!(runs(&dst), vec![run(2, 5, 14)]);
    }

    #[test]
    fn a_union_from_a_layer_hanging_off_an_edge_is_clipped_not_dropped() {
        let mut src = RowBits::new(10, 3);
        src.mark(0, 0, 9);
        src.mark(2, 0, 9);
        let mut dst = RowBits::new(80, 24);
        dst.union_translated(&src, -4, -1, Rect::new(0, 0, 80, 24));
        assert_eq!(
            runs(&dst),
            vec![run(1, 0, 5)],
            "row -1 is dropped, row 1 is clipped"
        );
    }

    #[test]
    fn a_union_of_two_spans_stays_exact() {
        // The claim the bitset is chosen for: OR-ing two layers keeps the gap between them.
        let mut a = RowBits::new(300, 4);
        a.mark(0, 0, 89);
        let mut b = RowBits::new(300, 4);
        b.mark(0, 190, 279);
        let mut dst = RowBits::new(300, 4);
        let clip = Rect::new(0, 0, 300, 4);
        dst.union_translated(&a, 0, 0, clip);
        dst.union_translated(&b, 0, 0, clip);
        let emitted: usize = runs(&dst).iter().map(|r| r.len()).sum();
        assert_eq!(emitted, 180, "per-row spans would emit 280");
    }

    #[test]
    fn a_zero_width_bitset_marks_nothing() {
        let mut d = RowBits::new(0, 4);
        d.mark(0, 0, 0);
        d.mark_all();
        assert!(d.is_empty());
    }

    /// The five costs, printed beside the numbers recorded for them.
    ///
    /// A report, not a gate: the register says a timing is a gate only at cliff granularity, and
    /// none of these is near a cliff. The gates that *are* gates live above — the idle clear's row
    /// count, and the exactness of the union.
    ///
    /// The numbers are only worth reading under `--release`; the test says so rather than leaving
    /// a reader to compare a debug build against a release measurement:
    ///
    /// ```text
    /// cargo test --release -p vitui-engine damage_costs -- --nocapture
    /// ```
    ///
    /// The structure of each case is the original's: mark is once per verb over twenty popups,
    /// scan is the sparse chart's 400 scattered cells, union is twenty layers folded into one
    /// target, and clear follows a full-screen frame.
    #[test]
    fn damage_costs_against_the_spec_table() {
        const W: u16 = 300;
        const H: u16 = 80;

        let mut marked = RowBits::new(W, H);
        let mut scanned = RowBits::new(W, H);
        for i in 0..400 {
            let x = (i * 7) % W as i32;
            scanned.mark((i * 13) % H as i32, x, x);
        }
        let mut full = RowBits::new(W, H);
        let mut unioned = RowBits::new(W, H);
        let layers: Vec<RowBits> = (0..20)
            .map(|_| {
                let mut d = RowBits::new(90, 14);
                d.mark_all();
                d
            })
            .collect();
        let idle = RowBits::new(W, H);
        let clip = Rect::new(0, 0, W, H);

        let report = Bench::new(20)
            .case("mark/twenty-popups", 200, || {
                // Once per drawing verb, not once per cell: twenty popups, fourteen rows each.
                for i in 0..20i32 {
                    for y in 0..14 {
                        marked.mark(y + i, i * 11, i * 11 + 89);
                    }
                }
                marked.clear();
            })
            .case("scan/sparse-chart", 200, || {
                scanned.for_each_run(|r| {
                    std::hint::black_box(r);
                });
            })
            .case("union/twenty-layers", 200, || {
                for (i, layer) in layers.iter().enumerate() {
                    unioned.union_translated(layer, i as i32 * 11, i as i32 * 3, clip);
                }
                unioned.clear();
            })
            .case("clear/after-a-full-screen-frame", 200, || {
                full.mark_all();
                std::hint::black_box(full.clear());
            })
            .case("idle/nothing-damaged", 2_000, || {
                std::hint::black_box(idle.is_empty());
            })
            .run();

        if cfg!(debug_assertions) {
            println!(
                "these numbers are a debug build and are not comparable to the table below; \
                 rerun with --release"
            );
        }
        println!(
            "damage costs, minimum of 20 rounds:\n{report}\n\
             spec §6 recorded, for RowBits: mark 1.56 us, scan 1.94 us, union 3.01 us, \
             clear 608 ns, idle 6.60 ns"
        );
    }
}
