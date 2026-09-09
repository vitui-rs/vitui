//! **The seam that lets something other than the terminal watch a component draw.**
//!
//! A component that calls `cx.text` directly can be looked at, and cannot be *counted*. That is
//! fine until the first time a screen is slower or dirtier than it should be, at which point the
//! question is always the same — *which verb wrote which cell, and how many times* — and the only
//! honest way to answer it is to have the component write through something the test supplies.
//!
//! So every component here takes its writer as a parameter. There are two implementations:
//! [`Direct`], which forwards to the context and is what ships, and [`Tally`], which forwards *and*
//! records. A component's behaviour does not branch on which one it was given, because the trait
//! has no way to ask.
//!
//! # The recorder unions in root coordinates, and that is not a detail
//!
//! `Ctx::child` moves the origin, so a border drawn at `(0, 0)` of a panel and the first cell of
//! that panel's interior — also `(0, 0)`, one context down — are different cells with the same
//! name. A recorder that unions the coordinates each verb was *called* in reports them as one
//! double write. [`Ctx::origin`] is what the recorder adds to make the two names disagree the way
//! the screen does.
//!
//! The second half of the same trap is that a clipped verb is reported at the column it was
//! **asked** for. A verb that starts left of its clip consumed clusters that were discarded, so a
//! component that may overrun its own rectangle should write one cell at a time where it might —
//! per cell a write lands whole or is discarded whole, and the recorder's arithmetic stays true.

use std::collections::BTreeSet;

use vitui_runtime::{Ctx, Paint, Rect};

/// What a component writes through.
///
/// Two verbs, because two are enough for everything in this handbook. A real library adds
/// `restyle` and a padded run the same way, and for the same reason: the trait grows only where a
/// component needs a verb, never to mirror the context's whole surface.
pub trait Ink {
    /// Write `s` at `(x, y)`. Answers the columns written, which is what the engine reports and is
    /// **not** the columns advanced over when the string was clipped.
    fn text(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, s: &str, st: Paint) -> u16;

    /// Fill `r` with a repeat of `cluster`.
    fn fill(&mut self, cx: &mut Ctx<'_, '_>, r: Rect, cluster: &str, st: Paint);

    /// Write whatever the caller last handed to `Ctx::stage`, at `(x, y)`.
    ///
    /// The pair exists so that a formatted value costs no allocation: `stage` writes into a buffer
    /// the frame owns and this reads it back out. It is a verb of its own here because a verb the
    /// trait cannot see is a verb no counter can see — which is exactly the hole a component
    /// reaches for `format!` to fill.
    fn blit(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, st: Paint) -> u16;
}

/// The writer that ships: straight through to the context, no bookkeeping.
pub struct Direct;

impl Ink for Direct {
    fn text(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, s: &str, st: Paint) -> u16 {
        cx.text(x, y, s, st).cells
    }

    fn fill(&mut self, cx: &mut Ctx<'_, '_>, r: Rect, cluster: &str, st: Paint) {
        cx.fill(r, cluster, st);
    }

    fn blit(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, st: Paint) -> u16 {
        cx.blit(x, y, st).cells
    }
}

/// The writer a test supplies: forwards, and counts.
///
/// `verbs` is calls, `cells` is columns written, and `distinct` is the set of cells touched at
/// least once. The three answer different questions and a gate usually wants two of them: a screen
/// that writes 6 662 cells into 6 662 distinct ones is painting itself once, and the same figure
/// over 3 100 distinct cells is painting half of itself twice.
#[derive(Default)]
pub struct Tally {
    /// How many verbs were called.
    pub verbs: u32,
    /// How many columns those verbs wrote.
    pub cells: u32,
    touched: BTreeSet<(i32, i32)>,
}

impl Tally {
    /// How many cells were written at least once.
    pub fn distinct(&self) -> usize {
        self.touched.len()
    }

    /// Columns written beyond the first time. Zero is *every cell painted exactly once*, which is
    /// the property most component gates are really about.
    pub fn excess(&self) -> u32 {
        self.cells - self.cells.min(self.touched.len() as u32)
    }

    /// Record a run of `n` columns starting at `(x, y)` **in the frame's root coordinates**.
    ///
    /// It does not count the verb: a fill is one verb over `h` runs, and counting a run as a call
    /// would make a component's verb count a function of its rectangle's height.
    fn mark(&mut self, cx: &Ctx<'_, '_>, x: i32, y: i32, n: u16) {
        let (ox, oy) = cx.origin();
        for i in 0..i32::from(n) {
            self.touched.insert((ox + x + i, oy + y));
        }
        self.cells += u32::from(n);
    }
}

impl Ink for Tally {
    fn text(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, s: &str, st: Paint) -> u16 {
        let n = cx.text(x, y, s, st).cells;
        self.verbs += 1;
        self.mark(cx, x, y, n);
        n
    }

    fn fill(&mut self, cx: &mut Ctx<'_, '_>, r: Rect, cluster: &str, st: Paint) {
        cx.fill(r, cluster, st);
        self.verbs += 1;
        for row in 0..i32::from(r.h) {
            self.mark(cx, r.x, r.y + row, r.w);
        }
    }

    fn blit(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, st: Paint) -> u16 {
        let n = cx.blit(x, y, st).cells;
        self.verbs += 1;
        self.mark(cx, x, y, n);
        n
    }
}
