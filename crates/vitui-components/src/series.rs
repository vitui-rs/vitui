//! **The series screen: two million-point series, at 300x80 and at 60x20, and 175 712 axis pairs.**
//!
//! Components ticket 27. Spec §13, §21. This is the screen `chart`'s and `plot`'s scenes are scenes
//! *of*, and it is the third of these after [`crate::dense`] and [`crate::listing`].
//!
//! | scene | what it decides |
//! |---|---|
//! | two million-point series at 1k / 100k / 1M, at 300x80 and 60x20 | the raster memo; and 60x20, where the overlap is red |
//! | 175 712 axis viewport x dataset pairs | the axis loop oscillates on **464**; from the whole domain, **0** |
//!
//! # A plot cannot satisfy the data-volume invariant the way a collection does
//!
//! [`crate::listing`] satisfies *frame cost is proportional to visible cells* by **never folding**:
//! the window says which rows can be reached and the rest are never touched. A plot must fold,
//! because which of a million points land in the rectangle is not knowable without looking at them.
//! So §13 splits the cost in two instead:
//!
//! > **The frame costs the rectangle. The edit costs the data. There is no third option, and the
//! > memo is the only thing standing between them.**
//!
//! What is folded is the **raster** — one sub-cell bitmask per cell, `w x h` bytes — because that is
//! the smallest object whose *size* is the rectangle and whose *contents* are the data.
//!
//! # 60x20 is the hostile axis, and it is why the scene runs at two sizes
//!
//! The overlap defect is green at 300x80 and red at 60x20. It is one of the four axes established by
//! a defect that passed every gate then in force, and the instrument is [`crate::runner`]'s two-size
//! comparison — the same one [`crate::dense`] uses for the label that does not narrow.
//!
//! # Three false greens, each faster and each wrong on the surface
//!
//! [`Reach::CulledByIndex`] is what a virtualised collection does and it is **correct there**: an
//! index window is a window on rows. Here an index is not an x coordinate, so it draws less, faster,
//! and loses the series. [`Range::Sampled`] is an axis range from a stride sample, which optimises a
//! cost the memo had already removed. [`Reach::Strided`] is every *n*-th point instead of the union,
//! and the union is what makes a column's extrema survive by construction.
//!
//! Each is a field on [`Opts`], so a comparison is **the same screen with one thing changed** — the
//! arrangement [`crate::dense`] and [`crate::listing`] both use, and its reason: a second painter
//! written against the alternative would be a gate testing a copy.
//!
//! # Verbs are not flat and are not gated as flat
//!
//! §21's register carries `verbs <= writes` for this component and says *never verb equality across
//! sizes* in as many words. A run ends where a cell's owner changes, so a plot's verb count tracks
//! the **picture** and is not monotone in *n*. [`Shape`] carries both, and the gate is the relation
//! while the counts are a report.
//!
//! # This module names a glyph repertoire, and that is the finding this ticket carries
//!
//! [`crate::gates::REGISTER`]'s row 26 is *`GlyphSet::` in `vitui-components` == 0*. `CONTEXT.md`
//! states both halves of the collision in two adjacent paragraphs: **Repertoire** — *a component
//! branches on it rather than the engine substituting behind its back* — and **Glyph** — *the
//! sub-cell ladders are the case … a component names no repertoire*. The sub-cell ladder is named in
//! the second sentence as the thing that is a branch, and a branch on the repertoire is a component
//! naming the repertoire.
//!
//! It is settled the way §21's own refinement 3 settles this shape: **name the exception, do not
//! loosen the gate.** [`geom`] is the one branch, this is the only file in the crate that carries
//! one, and the register's scan excepts this file **by name and by count** — a second file, or a
//! different number of occurrences in this one, fails it.
//!
//! # The screen is red until components ticket 28 lands
//!
//! [`standing`] is a [`Verdict`] over two subjects and [`owed_message`] is the sentence that
//! separates *waiting for its subject* from *the code is wrong* — ticket 09's criterion 7, inherited
//! whole. Everything the screen itself can be asked is measured here; what is missing is
//! `pub fn chart(` and `pub fn plot(` in `src/chart.rs`.

use std::fmt::Write as _;
use std::time::{Duration, Instant};

use vitui_runtime::ctx::Driver;
use vitui_runtime::data::Revision;
use vitui_runtime::layout::rect;
use vitui_runtime::theme::CATPPUCCIN_MOCHA;
use vitui_runtime::{
    ColorDepth, Ctx, Density, Glyph, GlyphSet, Id, Interest, Paint, Rect, Response, Rgb, Role,
    Theme,
};

use crate::counters::Tally;
use crate::frame::{BlockOpts, block_into};
use crate::ink::{Direct, Ink};
use crate::obligations::Verdict;
use crate::runner::{Canvas, Diff, Pen};
use crate::text::{FitOpts, fit_into};

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The wide screen's width. §21's own 300x80.
pub const W: u16 = 300;
/// The wide screen's height.
pub const H: u16 = 80;
/// **The narrow screen's width. Sixty**, and it is the hostile half of the two-size scene.
pub const NARROW_W: u16 = 60;
/// The narrow screen's height. Twenty.
pub const NARROW_H: u16 = 20;

/// How many series the screen stands up. **Two**, which is §21's own content and the number that
/// makes a shared cell possible at all — one series never shares a cell with itself.
pub const SERIES: usize = 2;

/// The three data volumes §21's scene 15 states.
pub const VOLUMES: [usize; 3] = [1_000, 100_000, 1_000_000];

/// **Interactive regions the screen declares. Two** — one for the plot and one for the chart.
///
/// The three panes' chrome declares none: it goes through [`block_into`], which draws a frame and
/// returns the interior and registers nothing. `crate::structure::panel_into` would declare a third
/// each, and a panel that is not the subject of a click is a region nothing reads.
pub const REGIONS: usize = 2;

// ── the two constructions, and the sub-cell ladder ───────────────────────────────────────────────

/// The two constructions. **Not two skins of one component**: they differ in what a cell means, in
/// how many states a cell has, and — the part no glyph table can carry — in how many samples they
/// ask the data for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// `chart`: bottom-anchored bars. A cell is a **prefix** of a column, so its states are ordered
    /// and the sub-column axis is unused.
    Bars,
    /// `plot`: arbitrary marks. A cell is a **set** of positions, so its states are a power set and
    /// both sub-axes are live.
    Marks,
}

impl Kind {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Kind::Bars => "chart (bars)",
            Kind::Marks => "plot (marks)",
        }
    }
}

/// Sub-cells per cell, on each axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Geom {
    /// Sub-columns.
    pub sx: u8,
    /// Sub-rows.
    pub sy: u8,
}

impl Geom {
    /// How many bits a cell's mask carries.
    pub const fn bits(self) -> u32 {
        self.sx as u32 * self.sy as u32
    }

    /// **How many distinguishable states a cell has**, which is the number the third repertoire rung
    /// has to be argued from.
    ///
    /// For bars a cell is a prefix, so it is `sy + 1`; for marks it is the power set.
    pub const fn states(self, kind: Kind) -> u32 {
        match kind {
            Kind::Bars => self.sy as u32 + 1,
            Kind::Marks => 1u32 << self.bits(),
        }
    }
}

/// **The branch, stated once — and the only place in `vitui-components` that names a repertoire.**
///
/// `CONTEXT.md` defines the middle rung as *Unicode with box drawing **and block elements***, and
/// the eighth blocks are block elements: U+2581…U+2588, one contiguous run with the half blocks. An
/// operator who promises block elements has promised all of them; no font has the half block and
/// lacks the third. So the bar ladder is **1 / 8 / 8** and the third rung buys a bar chart
/// *nothing*.
///
/// Marks are the other story. A bottom-anchored prefix needs one sub-column; an arbitrary point
/// needs two axes. Block elements give the quadrants — a 2x2 grid, 16 states. Braille gives 2x4, 256
/// states. **That factor of two on the vertical axis is the entire earning of the third rung, and it
/// is `plot`'s and not `chart`'s.**
///
/// # Why this cannot be a `Glyph`, and why it therefore has to name a repertoire
///
/// A glyph is *a lookup with a spelling at every repertoire level, every spelling exactly one cell,
/// and no spelling blank* (`CONTEXT.md`). The same cell here is one of 2, 9, 16 or **256**
/// characters, keyed on a *bitmask*, and the number of samples asked of the data changes with the
/// rung — so the theme's table cannot carry it and the branch is the component's. `CONTEXT.md` says
/// exactly that, in the `Glyph` entry, and in the same breath says *a component names no
/// repertoire*. Both cannot hold here, and the collision is filed rather than decided: see this
/// module's header and `.scratch/vitui-components-impl/issues/28`.
///
/// Nothing else in this crate branches on a repertoire. What a component wants a *spelling* for
/// still goes through `Theme::glyph`, and this screen's chrome does.
pub fn geom(kind: Kind, set: GlyphSet) -> Geom {
    match (kind, set) {
        // At the bottom rung a cell is one sub-cell whatever the construction — `#` or `*`, two
        // states, and no sub-axis at all. One arm and not two, because that is one fact.
        (_, GlyphSet::Ascii) => Geom { sx: 1, sy: 1 },
        // A prefix needs one sub-column, and the eighth blocks are the *middle* rung by
        // `CONTEXT.md`'s own definition of it.
        (Kind::Bars, _) => Geom { sx: 1, sy: 8 },
        // A mark needs two axes. Quadrants are 2x2; braille is 2x4, and that one bit of vertical
        // resolution is the whole earning of the third rung.
        (Kind::Marks, GlyphSet::Unicode) => Geom { sx: 2, sy: 2 },
        (Kind::Marks, _) => Geom { sx: 2, sy: 4 },
    }
}

/// The nine prefixes a bar cell is one of, at the eighth-block rung.
const BAR8: [char; 9] = [
    ' ', '\u{2581}', '\u{2582}', '\u{2583}', '\u{2584}', '\u{2585}', '\u{2586}', '\u{2587}',
    '\u{2588}',
];

/// The sixteen quadrants a mark cell is one of, at the 2x2 rung. Indexed by the raw bitmask, which
/// is what makes this an array rather than a match.
const QUAD: [char; 16] = [
    ' ', '\u{2598}', '\u{259D}', '\u{2580}', '\u{2596}', '\u{258C}', '\u{259E}', '\u{259B}',
    '\u{2597}', '\u{259A}', '\u{2590}', '\u{259C}', '\u{2584}', '\u{2599}', '\u{259F}', '\u{2588}',
];

/// Bit `b = r * sx + c` renumbered into the braille pattern's own dot order.
const BRAILLE_BIT: [u8; 8] = [0, 3, 1, 4, 2, 5, 6, 7];

/// **The cluster a cell's bitmask spells, in the given construction.**
///
/// Two `match`es and a sixteen-entry array, which is §13's own description of the branch. It returns
/// a `char` rather than a `&'static str` because the braille rung is 256 spellings computed from the
/// mask, and a table of 256 static strings would be the private fallback table §16 forbids wearing a
/// different hat.
pub fn cluster(kind: Kind, g: Geom, bits: u8) -> char {
    match kind {
        Kind::Bars => {
            if g.sy == 1 {
                if bits == 0 { ' ' } else { '#' }
            } else {
                BAR8[bits.count_ones() as usize]
            }
        }
        Kind::Marks => match (g.sx, g.sy) {
            (1, 1) => {
                if bits == 0 {
                    ' '
                } else {
                    '*'
                }
            }
            (2, 2) => QUAD[usize::from(bits & 0x0F)],
            _ => {
                let mut pat = 0u32;
                for (b, dot) in BRAILLE_BIT.iter().enumerate() {
                    if bits & (1 << b) != 0 {
                        pat |= 1 << dot;
                    }
                }
                if pat == 0 {
                    ' '
                } else {
                    char::from_u32(0x2800 + pat).unwrap_or(' ')
                }
            }
        },
    }
}

// ── the domain, and where an axis range is allowed to come from ──────────────────────────────────

/// The value range an axis maps onto its sub-rows.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Domain {
    /// The bottom.
    pub y0: f32,
    /// The top.
    pub y1: f32,
}

impl Domain {
    /// A nudge outward, so a maximum lands **inside** the top cell rather than on its edge.
    pub fn padded(self) -> Domain {
        let span = (self.y1 - self.y0).max(f32::EPSILON);
        Domain {
            y0: self.y0,
            y1: self.y1 + span * 0.001,
        }
    }

    /// Its two ends as bits, which is what a memo key can hold. `f32` is not `Eq`.
    pub fn bits(self) -> (u32, u32) {
        (self.y0.to_bits(), self.y1.to_bits())
    }
}

/// **How the axis range is obtained.** Both are folds over the data; only one of them is over *the*
/// data.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Range {
    /// Every point. `O(n)` — on the **edit**, behind the same memo as the raster.
    Whole,
    /// **A stride sample**, the shape everyone reaches for when a fold looks expensive. False green
    /// two: it is an optimisation of a cost the memo had already removed, and it is *slower*.
    Sampled(u32),
}

impl Range {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Range::Whole => "the whole domain",
            Range::Sampled(_) => "a stride sample",
        }
    }
}

/// The extremes of `series`, obtained the way `how` says.
pub fn domain_of(series: &[Vec<f32>], how: Range) -> Domain {
    let step = match how {
        Range::Whole => 1usize,
        Range::Sampled(s) => (s as usize).max(1),
    };
    let (mut lo, mut hi) = (f32::INFINITY, f32::NEG_INFINITY);
    for s in series {
        let mut i = 0usize;
        while i < s.len() {
            let v = s[i];
            if v < lo {
                lo = v;
            }
            if v > hi {
                hi = v;
            }
            i += step;
        }
    }
    if !lo.is_finite() || !hi.is_finite() {
        return Domain { y0: 0.0, y1: 1.0 };
    }
    if (hi - lo).abs() < f32::EPSILON {
        return Domain {
            y0: lo,
            y1: lo + 1.0,
        };
    }
    Domain { y0: lo, y1: hi }
}

// ── the raster ───────────────────────────────────────────────────────────────────────────────────

/// **Which points a build is allowed to look at.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reach {
    /// Every point maps to a column, and the ones outside the domain are discarded **after** the
    /// map. Correct, and `O(n)` on the edit.
    Mapped,
    /// **The list instinct**: cull to the visible *index* window first, then map. Correct for a
    /// virtualised collection — [`crate::listing::Volume::Windowed`] is exactly this — and wrong for
    /// a plot, because an index is not an x coordinate. False green one.
    CulledByIndex,
    /// **Rasterise every *k*-th point**, `k = n / w`. The output has the same shape and the same
    /// frame cost, and the peaks are gone. False green three.
    Strided,
}

impl Reach {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Reach::Mapped => "every point, mapped",
            Reach::CulledByIndex => "culled to the visible index window",
            Reach::Strided => "every n-th point",
        }
    }
}

/// The owner id a cell two or more series reached carries.
pub const SHARED: u8 = 255;
/// The owner id a threshold row carries, so it takes part in the same run splitting as a series.
pub const OWNER_THRESHOLD: u8 = 254;

/// **One cell's worth of the rectangle, plus who owns it.**
///
/// `bits` and `owner` are each `w * h` bytes, so a 60x20 raster is 2 400 B — **at every data
/// volume**, which is the whole claim.
#[derive(Clone, Debug)]
pub struct Raster {
    w: u16,
    h: u16,
    kind: Kind,
    geom: Geom,
    dom: Domain,
    bits: Vec<u8>,
    owner: Vec<u8>,
    touched: u64,
    shared: u32,
}

impl Raster {
    /// A raster over nothing.
    pub fn empty() -> Raster {
        Raster {
            w: 0,
            h: 0,
            kind: Kind::Marks,
            geom: Geom { sx: 1, sy: 1 },
            dom: Domain { y0: 0.0, y1: 1.0 },
            bits: Vec::new(),
            owner: Vec::new(),
            touched: 0,
            shared: 0,
        }
    }

    /// **How many bytes it holds. `2 * w * h`**, and it is the number that does not move with the
    /// data.
    pub fn bytes(&self) -> usize {
        self.bits.len() + self.owner.len()
    }

    /// Its width in cells.
    pub fn w(&self) -> u16 {
        self.w
    }

    /// Its height in cells.
    pub fn h(&self) -> u16 {
        self.h
    }

    /// The domain it was built against.
    pub fn domain(&self) -> Domain {
        self.dom
    }

    /// **Points the build looked at.** The edit's cost as a count — the only place data volume is
    /// allowed to appear.
    pub fn touched(&self) -> u64 {
        self.touched
    }

    /// **Cells two or more series reached.** The number the braille/quadrant colour trade is decided
    /// on: a braille cell has one `Paint` for all eight dots and a quadrant carries a foreground and
    /// a background, so the third rung buys resolution and pays here.
    pub fn shared(&self) -> u32 {
        self.shared
    }

    /// **The defensive read that makes a wrongly-keyed memo invisible.**
    ///
    /// A raster built for one rectangle and read for another is out of bounds, and every real
    /// component has this `if` in it because the alternative is a panic on a resize. With it, a
    /// stale raster draws a *smaller, older* plot inside a bigger rectangle: no panic, no counter,
    /// fewer writes, faster.
    pub fn at(&self, x: u16, y: u16) -> (u8, u8) {
        if x >= self.w || y >= self.h {
            return (0, 0);
        }
        let i = usize::from(y) * usize::from(self.w) + usize::from(x);
        (self.bits[i], self.owner[i])
    }

    fn resize(&mut self, w: u16, h: u16) {
        let n = usize::from(w) * usize::from(h);
        self.bits.clear();
        self.bits.resize(n, 0);
        self.owner.clear();
        self.owner.resize(n, 0);
        self.w = w;
        self.h = h;
    }

    fn put(&mut self, cx: u16, cy: u16, bit: u8, series: u8) {
        let i = usize::from(cy) * usize::from(self.w) + usize::from(cx);
        self.bits[i] |= 1 << bit;
        let o = self.owner[i];
        if o == 0 {
            self.owner[i] = series;
        } else if o != series && o != SHARED {
            self.owner[i] = SHARED;
            self.shared += 1;
        }
    }

    /// **The whole of the data-volume cost, in one function.**
    ///
    /// It runs on a memo miss — an edit, a resize, a zoom or a repertoire swap — and on nothing
    /// else. Every loop over `n` in this module is inside it.
    ///
    /// **It is a union, and a union is idempotent.** A point sets bits; two points in one cell set
    /// the union of their bits. So a column's extrema survive *by construction* and downsampling is
    /// not a technique this component contains.
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        &mut self,
        w: u16,
        h: u16,
        kind: Kind,
        g: Geom,
        dom: Domain,
        series: &[Vec<f32>],
        reach: Reach,
        cull: (usize, usize),
    ) {
        self.resize(w, h);
        self.kind = kind;
        self.geom = g;
        self.dom = dom;
        self.touched = 0;
        self.shared = 0;
        if w == 0 || h == 0 {
            return;
        }
        let subw = u32::from(w) * u32::from(g.sx);
        let subh = u32::from(h) * u32::from(g.sy);
        let span = (dom.y1 - dom.y0).max(f32::EPSILON);

        for (si, s) in series.iter().enumerate() {
            let n = s.len();
            if n == 0 {
                continue;
            }
            let sid = (si as u8) + 1;
            let (lo, hi, step) = match reach {
                Reach::Mapped => (0usize, n, 1usize),
                Reach::CulledByIndex => (cull.0.min(n), cull.1.min(n), 1usize),
                Reach::Strided => (0usize, n, (n / usize::from(w).max(1)).max(1)),
            };
            let mut i = lo;
            while i < hi {
                self.touched += 1;
                let v = s[i];
                // x: the point's position in the series, mapped onto the sub-column axis.
                let sx_pos = ((i as u64 * u64::from(subw)) / (n as u64).max(1)) as u32;
                let sx_pos = sx_pos.min(subw - 1);
                // y: the value, mapped onto the sub-row axis, top down.
                let t = (dom.y1 - v) / span;
                let sy_pos = (t * subh as f32) as i32;
                if sy_pos < 0 || sy_pos >= subh as i32 {
                    // Discarded **after** the map. That ordering is what separates this from the
                    // culled arm, which discards before it.
                    i += step;
                    continue;
                }
                let sy_pos = sy_pos as u32;
                let cx = (sx_pos / u32::from(g.sx)) as u16;
                let c = (sx_pos % u32::from(g.sx)) as u8;
                match kind {
                    Kind::Marks => {
                        let cy = (sy_pos / u32::from(g.sy)) as u16;
                        let r = (sy_pos % u32::from(g.sy)) as u8;
                        self.put(cx, cy, r * g.sx + c, sid);
                    }
                    Kind::Bars => {
                        // A bar is the prefix from the value's sub-row to the bottom. **The union of
                        // two prefixes is the taller one**, so a column's maximum survives without
                        // anybody computing a maximum.
                        let mut sy = sy_pos;
                        while sy < subh {
                            let cy = (sy / u32::from(g.sy)) as u16;
                            let r = (sy % u32::from(g.sy)) as u8;
                            self.put(cx, cy, r * g.sx + c, sid);
                            sy += 1;
                        }
                    }
                }
                i += step;
            }
        }
    }
}

// ── axes: §9's loop in a second place, and here it oscillates ────────────────────────────────────

/// **A 1 / 2 / 5 x 10^k step, and the reason monotonicity fails.**
///
/// Fewer ticks do not give a subset of the labels. Five ticks over 0…10 are `0 2.5 5 7.5 10`; three
/// are `0 5 10`. The five-tick set contains a label one column wider than anything in the three-tick
/// set, and **neither set contains the other** — which is §9's precondition, *a narrower plotting
/// area may not produce a wider label*, failing for ordinary data.
pub fn nice_step(span: f32, target: u16) -> f32 {
    let target = f32::from(target.max(1));
    let raw = (span / target).max(f32::MIN_POSITIVE);
    let mag = 10f32.powf(raw.log10().floor());
    let n = raw / mag;
    let step = if n <= 1.0 {
        1.0
    } else if n <= 2.0 {
        2.0
    } else if n <= 5.0 {
        5.0
    } else {
        10.0
    };
    step * mag
}

/// How many decimals a step needs, so a label is not `0.30000001`.
pub fn decimals(step: f32) -> u32 {
    if step <= 0.0 || !step.is_finite() {
        return 0;
    }
    let d = -step.log10().floor();
    if d <= 0.0 { 0 } else { (d as u32).min(4) }
}

/// **The width of a label, computed rather than formatted**, so the frame path allocates nothing.
pub fn label_width(v: f32, dec: u32) -> u16 {
    let neg = v < 0.0;
    let a = v.abs();
    let int_digits = if a < 1.0 {
        1
    } else {
        u16::try_from(a.log10().floor() as i64 + 1)
            .unwrap_or(1)
            .max(1)
    };
    int_digits + u16::from(neg) + if dec > 0 { 1 + dec as u16 } else { 0 }
}

/// How many ticks a plotting area of this height carries. One every third row, at least two.
pub fn tick_count(plot_h: u16) -> u16 {
    (plot_h / 3).max(2)
}

/// The *i*-th tick's value, from the bottom.
pub fn tick_value(dom: Domain, step: f32, i: u16) -> f32 {
    let first = (dom.y0 / step).ceil() * step;
    first + step * f32::from(i)
}

/// **The gutter a domain needs**: the widest of its tick labels, plus one column of separation.
pub fn gutter(dom: Domain, plot_h: u16) -> u16 {
    let n = tick_count(plot_h);
    let step = nice_step(dom.y1 - dom.y0, n);
    let dec = decimals(step);
    let mut w = 1u16;
    let mut i = 0u16;
    loop {
        let v = tick_value(dom, step, i);
        if v > dom.y1 + step * 0.5 {
            break;
        }
        w = w.max(label_width(v, dec));
        i += 1;
        if i > 64 {
            break;
        }
    }
    w + 1
}

/// **How the gutter is obtained, which is the whole decision.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sizing {
    /// The naive one: recompute from the window the current gutter produces, and feed it back.
    Fixpoint,
    /// **§9's hysteresis form**: compute from *last frame's* plotting width. Never loops, and
    /// settles on a gutter that is not a fixed point of its own rule.
    LastFrame,
    /// **The decision.** The gutter comes from the whole series' range, which is data — so it does
    /// not depend on the rectangle at all and there is no loop to solve.
    ///
    /// > A layout loop is worth solving only when both ends are genuinely layout; when one end is
    /// > data, decouple it.
    WholeDomain,
}

impl Sizing {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Sizing::Fixpoint => "auto-scaled to the window, fed back",
            Sizing::LastFrame => "last frame's plotting width",
            Sizing::WholeDomain => "the whole domain",
        }
    }
}

/// What settling the loop turned out to be.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    /// A fixed point, reached in this many passes.
    Converged(u8),
    /// **A cycle of length two or more.** The screen flips for ever.
    Oscillates(u8),
}

/// How many samples a plotting column shows, for a live series.
pub const PPC: usize = 4;

/// **A live series, with the window domains it answers precomputed.**
///
/// The cache is a memo over a pure function and not a shortcut: `window_domain(plot_w)` depends on
/// the series and on `plot_w` and on nothing else, and the sweep asks for the same few hundred
/// values 175 712 times. Without it the sweep is a scan of the data per pass per pair.
pub struct Live<'a> {
    y: &'a [f32],
    cached: Vec<Domain>,
    whole: Domain,
}

impl<'a> Live<'a> {
    /// A live series whose window domains are precomputed for every plotting width up to `max_w`.
    pub fn over(y: &'a [f32], max_w: u16) -> Live<'a> {
        let mut cached = Vec::with_capacity(usize::from(max_w) + 1);
        for w in 0..=max_w {
            cached.push(Live::scan(y, w));
        }
        let whole = Live::scan(y, u16::MAX);
        Live { y, cached, whole }
    }

    fn scan(y: &[f32], plot_w: u16) -> Domain {
        let want = usize::from(plot_w).saturating_mul(PPC).max(1);
        let lo = y.len().saturating_sub(want);
        let (mut a, mut b) = (f32::INFINITY, f32::NEG_INFINITY);
        for &v in &y[lo..] {
            if v < a {
                a = v;
            }
            if v > b {
                b = v;
            }
        }
        if !a.is_finite() || (b - a).abs() < f32::EPSILON {
            return Domain { y0: 0.0, y1: 1.0 };
        }
        Domain { y0: a, y1: b }
    }

    /// **The auto-scaled range of what a plotting area this wide is showing.**
    pub fn window_domain(&self, plot_w: u16) -> Domain {
        match self.cached.get(usize::from(plot_w)) {
            Some(d) => *d,
            None => Live::scan(self.y, plot_w),
        }
    }

    /// The range of the whole series, which is data and does not depend on the rectangle.
    pub fn whole_domain(&self) -> Domain {
        self.whole
    }
}

/// **Iterate the loop over a `w` by `h` viewport and say which of the three outcomes it is.**
///
/// `last` is the previous frame's gutter, and it is read only by [`Sizing::LastFrame`].
pub fn settle(live: &Live<'_>, w: u16, h: u16, sizing: Sizing, last: u16) -> (u16, Outcome) {
    let plot_h = h.saturating_sub(2).max(1);
    match sizing {
        Sizing::WholeDomain => {
            let g = gutter(live.whole_domain(), plot_h);
            (g.min(w.saturating_sub(1)), Outcome::Converged(1))
        }
        Sizing::LastFrame => {
            let plot_w = w.saturating_sub(last).max(1);
            let g = gutter(live.window_domain(plot_w), plot_h);
            (g.min(w.saturating_sub(1)), Outcome::Converged(1))
        }
        Sizing::Fixpoint => {
            let mut seen = [u16::MAX; 12];
            let mut g = 0u16;
            for pass in 0..12u8 {
                let plot_w = w.saturating_sub(g).max(1);
                let next = gutter(live.window_domain(plot_w), plot_h).min(w.saturating_sub(1));
                if next == g {
                    return (g, Outcome::Converged(pass + 1));
                }
                if let Some(k) = seen.iter().position(|&s| s == next) {
                    return (g, Outcome::Oscillates((pass + 1) - k as u8));
                }
                seen[usize::from(pass)] = next;
                g = next;
            }
            (g, Outcome::Oscillates(12))
        }
    }
}

/// **A dataset built so the recent samples are small decimals and the older ones are large
/// integers.**
///
/// Nothing exotic: it is a latency trace after a deploy, or a queue depth that drained. That shape
/// is what makes *dropping the older samples out of the window* leave `-0.05` where `900` was, which
/// is five columns of label where three were.
pub fn dataset(seed: u32, n: usize) -> Vec<f32> {
    let mut v = Vec::with_capacity(n);
    let tail = 40 + (seed as usize % 400);
    for i in 0..n {
        if i + tail < n {
            let a = ((i as u32).wrapping_mul(2_654_435_761).wrapping_add(seed) >> 8) % 900;
            v.push(100.0 + a as f32);
        } else {
            let a = ((i as u32).wrapping_mul(40_503).wrapping_add(seed) >> 4) % 100;
            v.push(a as f32 / 1000.0 - 0.05);
        }
    }
    v
}

/// How many datasets the sweep runs over. Eight.
pub const AXIS_DATASETS: u32 = 8;
/// How long each of them is.
pub const AXIS_POINTS: usize = 20_000;
/// The narrowest viewport the sweep visits.
pub const AXIS_W: std::ops::RangeInclusive<u16> = 12..=300;
/// The shortest viewport the sweep visits.
pub const AXIS_H: std::ops::RangeInclusive<u16> = 5..=80;

/// **How many viewport x dataset pairs the sweep visits. 175 712** — `8 * 289 * 76`, and §21 states
/// the product rather than the factors.
pub const AXIS_PAIRS: u64 = AXIS_DATASETS as u64 * 289 * 76;

/// What the sweep found, in the three columns §9 named.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct AxisTally {
    /// How many pairs were visited.
    pub pairs: u64,
    /// How many reached a fixed point.
    pub converged: u64,
    /// **How many never do.** The screen flips for ever.
    pub oscillates: u64,
    /// The most passes any converging pair needed.
    pub max_passes: u8,
    /// Converging pairs the hysteresis form disagrees with the fixpoint about.
    pub hysteresis_wrong: u64,
    /// **Oscillating pairs the hysteresis form silently stabilises** — no loop, and a gutter that is
    /// not a fixed point of its own rule. §9's outcome, in this second place.
    pub hysteresis_settled_off: u64,
    /// How many pairs the whole-domain form visited.
    pub whole_pairs: u64,
    /// **How many of them oscillate.** Zero: there is no edge to oscillate in.
    pub whole_oscillates: u64,
    /// How many passes the whole-domain form ever needs. One.
    pub whole_max_passes: u8,
}

/// **Sweep the full [`AXIS_PAIRS`] and report all three forms.**
pub fn axis_sweep() -> AxisTally {
    let mut t = AxisTally::default();
    let sets: Vec<Vec<f32>> = (0..AXIS_DATASETS)
        .map(|s| dataset(s, AXIS_POINTS))
        .collect();
    for ys in &sets {
        let live = Live::over(ys, *AXIS_W.end());
        for w in AXIS_W {
            for h in AXIS_H {
                t.pairs += 1;
                let (g, out) = settle(&live, w, h, Sizing::Fixpoint, 0);
                let settled = hysteresis_steady(&live, w, h);
                match out {
                    Outcome::Converged(p) => {
                        t.converged += 1;
                        t.max_passes = t.max_passes.max(p);
                        if settled != g {
                            t.hysteresis_wrong += 1;
                        }
                    }
                    Outcome::Oscillates(_) => {
                        t.oscillates += 1;
                        // Is what it settled on a fixed point of the rule it is meant to keep?
                        let plot_w = w.saturating_sub(settled).max(1);
                        let want = gutter(live.window_domain(plot_w), h.saturating_sub(2).max(1))
                            .min(w.saturating_sub(1));
                        if want != settled {
                            t.hysteresis_settled_off += 1;
                        }
                    }
                }
                t.whole_pairs += 1;
                let (_, out) = settle(&live, w, h, Sizing::WholeDomain, 0);
                match out {
                    Outcome::Oscillates(_) => t.whole_oscillates += 1,
                    Outcome::Converged(p) => t.whole_max_passes = t.whole_max_passes.max(p),
                }
            }
        }
    }
    t
}

/// The gutter the hysteresis form comes to rest on, run to its own steady state.
fn hysteresis_steady(live: &Live<'_>, w: u16, h: u16) -> u16 {
    let mut last = 0u16;
    for _ in 0..8 {
        let (next, _) = settle(live, w, h, Sizing::LastFrame, last);
        if next == last {
            break;
        }
        last = next;
    }
    last
}

// ── the data ─────────────────────────────────────────────────────────────────────────────────────

/// **The two series the screen stands up, and the revision that keys their memo.**
#[derive(Clone, Debug)]
pub struct Series {
    points: Vec<Vec<f32>>,
    rev: Revision,
}

impl Series {
    /// `series` series of `n` points each, with rare one-sample spikes scattered rather than
    /// periodic.
    ///
    /// **The spikes are the whole reason *every n-th point* is not a downsampling strategy.** At 1M
    /// points into 170 columns the stride is 5 882, and a one-sample spike survives it with
    /// probability one in 5 882. A union keeps every one of them, because an extremum is a set
    /// member and a union is idempotent.
    pub fn build(n: usize, series: usize) -> Series {
        let mut out = Vec::with_capacity(series);
        for s in 0..series {
            let mut v = Vec::with_capacity(n);
            for i in 0..n {
                let t = i as f32 / n.max(1) as f32;
                let base = (t * 12.0 + s as f32).sin() * 20.0 + 50.0 + s as f32 * 7.0;
                let hit = (i as u32)
                    .wrapping_mul(2_654_435_761)
                    .wrapping_add(s as u32 * 7_919)
                    >> 7;
                v.push(if hit % 1_501 == 3 { base + 26.0 } else { base });
            }
            out.push(v);
        }
        Series {
            points: out,
            rev: Revision::fresh(),
        }
    }

    /// The points, per series.
    pub fn points(&self) -> &[Vec<f32>] {
        &self.points
    }

    /// The revision the memo chain is keyed on.
    pub fn revision(&self) -> Revision {
        self.rev
    }

    /// How many points each series holds.
    pub fn len(&self) -> usize {
        self.points.first().map_or(0, Vec::len)
    }

    /// Whether there is nothing to draw.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ── the options: every alternative under measurement is one field ────────────────────────────────

/// **What a pane is, and every variant this ticket has to price is one field of it.**
///
/// [`crate::dense::Build`]'s arrangement and [`crate::listing::Volume`]'s, for the same reason: a
/// second painter written against the alternative is a gate testing a copy.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Opts {
    /// Bars or marks.
    pub kind: Kind,
    /// Which points the raster is allowed to look at.
    pub reach: Reach,
    /// Where the axis range comes from.
    pub range: Range,
    /// The threshold value, if the pane draws one.
    pub threshold: Option<f32>,
    /// **Whether the threshold is carried on the glyph axis as well as on the paint axis.**
    ///
    /// §16's *carried on both axes*, in the place a chart meets it: at sixteen colours `Danger`,
    /// `Warn` and `Ok` need not differ on the wire, so a threshold carried by a paint alone is
    /// invisible and a rule drawn in `HLine` is not.
    pub threshold_glyph: bool,
    /// **Whether series colours are role-derived rather than named.**
    ///
    /// The failure this exists to price: there are thirteen roles, and the three that read as series
    /// colours collapse to one at sixteen colours.
    pub role_series: bool,
}

impl Opts {
    /// The plot pane: marks, every point, the whole domain, a threshold on both axes.
    pub fn plot() -> Opts {
        Opts {
            kind: Kind::Marks,
            reach: Reach::Mapped,
            range: Range::Whole,
            threshold: Some(84.0),
            threshold_glyph: true,
            role_series: false,
        }
    }

    /// The chart pane: the same, as bars.
    pub fn chart() -> Opts {
        Opts {
            kind: Kind::Bars,
            ..Opts::plot()
        }
    }
}

/// **The whole screen as one value**, so a comparison is one word at the call site.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Build {
    /// The repertoire the theme is told about.
    pub glyphs: GlyphSet,
    /// The colour depth the theme is resolved for.
    pub depth: ColorDepth,
    /// The plot pane.
    pub plot: Opts,
    /// The chart pane.
    pub chart: Opts,
    /// **Whether the legend's label narrows through [`fit_into`].**
    ///
    /// The two-size axis as one field: `true` is the correct build and `false` is
    /// [`defective::legend_that_does_not_narrow`]. The two are **indistinguishable at 300x80**,
    /// which is the whole reason the scene is played at two sizes.
    pub narrowing_legend: bool,
}

impl Build {
    /// The correct screen: braille, truecolor, both panes as §13 states them.
    pub fn correct() -> Build {
        Build {
            glyphs: GlyphSet::Extended,
            depth: ColorDepth::TrueColor,
            plot: Opts::plot(),
            chart: Opts::chart(),
            narrowing_legend: true,
        }
    }

    /// The same screen with one thing changed in **both** panes.
    pub fn both(mut self, f: impl Fn(&mut Opts)) -> Build {
        f(&mut self.plot);
        f(&mut self.chart);
        self
    }

    /// The same screen at another repertoire.
    pub fn at(mut self, glyphs: GlyphSet) -> Build {
        self.glyphs = glyphs;
        self
    }

    /// The same screen at another colour depth.
    pub fn tier(mut self, depth: ColorDepth) -> Build {
        self.depth = depth;
        self
    }

    /// The theme this build asks for.
    pub fn theme(self) -> Theme {
        Theme::authored(&CATPPUCCIN_MOCHA, self.glyphs, Density::default()).resolve(self.depth)
    }
}

// ── colour: the one legitimate use of `Theme::custom` ────────────────────────────────────────────

/// **The series palette. Counted, not scattered.**
///
/// This is the only place in `vitui-components` that names a colour, and it exists because there are
/// as many series as the data says and no theme can enumerate them: there are thirteen roles.
/// [`Theme::custom`] is what makes it legal; a `Style` literal would not be, and there are none in
/// this crate.
pub const SERIES_RGB: [(u8, u8, u8); 6] = [
    (0x5f, 0xaf, 0xff),
    (0xff, 0x87, 0x5f),
    (0x87, 0xd7, 0x5f),
    (0xd7, 0x87, 0xff),
    (0xff, 0xd7, 0x5f),
    (0x5f, 0xd7, 0xd7),
];

/// **The paint of series `i`.**
///
/// # `Theme::custom` needs a background and a component cannot read the page's
///
/// It takes two [`Rgb`], and the thirteen roles are `Paint`s rather than colours — there is no
/// `Theme::page()`, no `Role::Page` and no way to ask what the body's background is. So the
/// background is derived from `Theme::is_dark`, which is the one bit about the page a component can
/// read. That is a stated substitution and not a preference: a `Theme::custom_fg` taking one colour,
/// or a readable page colour, would remove it, and neither exists. Filed with components ticket 28.
pub fn series_paint(theme: &Theme, i: usize, role_series: bool) -> Paint {
    if role_series {
        // The failure this arm exists to price: three of the thirteen roles read as series colours
        // and they collapse to one at sixteen colours.
        return theme.paint(match i % 3 {
            0 => Role::Danger,
            1 => Role::Warn,
            _ => Role::Ok,
        });
    }
    let (r, g, b) = SERIES_RGB[i % SERIES_RGB.len()];
    let page = if theme.is_dark() {
        Rgb::new(0, 0, 0)
    } else {
        Rgb::new(0xff, 0xff, 0xff)
    };
    theme.custom(Rgb::new(r, g, b), page)
}

// ── the pane's own state, and the cache the frame reads ──────────────────────────────────────────

/// What a raster was built for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct PaneKey {
    rev: u64,
    w: u16,
    h: u16,
    kind: u8,
    sx: u8,
    sy: u8,
    series: u8,
    y0: u32,
    y1: u32,
    reach: u8,
}

/// **A pane's own state, and the cache lives in it** — `CONTEXT.md`'s rule that a memo is *keyed by
/// where it is stored*, which is why it consumes no `Id`.
///
/// **This is a cache and not yet §13's memo chain.** Components ticket 28 replaces it with
/// [`vitui_runtime::data::Memo`] over a folded key, prices the two narrow keys against it, and turns
/// the range half into a separate memo the raster's chains from. What is here is the smallest thing
/// that makes the screen measurable at a million points: without it every frame is a fold.
#[derive(Debug)]
pub struct PaneState {
    key: Option<PaneKey>,
    dom_key: Option<(u64, u8)>,
    dom: Domain,
    dom_misses: u32,
    raster: Raster,
    misses: u32,
    /// The row buffer the frame spells into, allocated once.
    row: String,
    /// The label buffer the gutter formats into, allocated once.
    label: String,
}

impl PaneState {
    /// A pane that has drawn nothing.
    pub fn new() -> PaneState {
        PaneState {
            key: None,
            dom_key: None,
            dom: Domain { y0: 0.0, y1: 1.0 },
            dom_misses: 0,
            raster: Raster::empty(),
            misses: 0,
            row: String::with_capacity(1024),
            label: String::with_capacity(64),
        }
    }

    /// The raster the last frame drew from.
    pub fn raster(&self) -> &Raster {
        &self.raster
    }

    /// How many times the raster has been rebuilt.
    pub fn misses(&self) -> u32 {
        self.misses
    }

    /// How many times the range has been refolded.
    pub fn range_misses(&self) -> u32 {
        self.dom_misses
    }

    /// The domain the last frame drew against.
    pub fn domain(&self) -> Domain {
        self.dom
    }

    /// **The range half.** Its key is the data revision and the policy; the rectangle is
    /// deliberately not in it, because a resize does not change what the data's extremes are.
    fn range(&mut self, rev: Revision, how: Range, series: &[Vec<f32>]) -> Domain {
        let k = (
            rev.raw(),
            match how {
                Range::Whole => 0u8,
                Range::Sampled(_) => 1,
            },
        );
        if self.dom_key != Some(k) {
            self.dom_key = Some(k);
            self.dom_misses += 1;
            self.dom = domain_of(series, how).padded();
        }
        self.dom
    }

    /// The raster half. Its key is every input, including the domain the range half just answered.
    #[allow(clippy::too_many_arguments)]
    fn refresh(
        &mut self,
        rev: Revision,
        w: u16,
        h: u16,
        kind: Kind,
        g: Geom,
        dom: Domain,
        series: &[Vec<f32>],
        reach: Reach,
        cull: (usize, usize),
    ) {
        let (y0, y1) = dom.bits();
        let key = PaneKey {
            rev: rev.raw(),
            w,
            h,
            kind: kind as u8,
            sx: g.sx,
            sy: g.sy,
            series: series.len() as u8,
            y0,
            y1,
            reach: reach as u8,
        };
        if self.key != Some(key) {
            self.key = Some(key);
            self.misses += 1;
            self.raster.build(w, h, kind, g, dom, series, reach, cull);
        }
    }
}

impl Default for PaneState {
    fn default() -> PaneState {
        PaneState::new()
    }
}

/// **The screen's state: one [`PaneState`] a pane.**
#[derive(Debug, Default)]
pub struct ScreenState {
    /// The plot pane's.
    pub plot: PaneState,
    /// The chart pane's.
    pub chart: PaneState,
    /// **The legend's row buffer, allocated once.**
    ///
    /// It is a field rather than a local of [`legend_into`] because a `String::with_capacity` on the
    /// draw path is **one allocation a frame, for ever** — measured at 8 over 8 frames against a
    /// budget of zero, by `examples/series_numbers.rs`'s total. The same shape as
    /// `player::chrome`'s `Vec<f32>`, which the components backlog records as already fixed and
    /// asks nobody to re-fix.
    pub legend: String,
}

impl ScreenState {
    /// Two panes that have drawn nothing.
    pub fn new() -> ScreenState {
        ScreenState::default()
    }
}

// ── the stand-in painter ─────────────────────────────────────────────────────────────────────────

/// **One pane, drawn from its raster.** The stand-in components ticket 28 replaces.
///
/// The signature is spec §1's `(cx, rect, data, …)` plus the options struct and the pane's own
/// state, drawing through an [`Ink`] so a counter and a recorded surface see the shipped path rather
/// than a copy of it.
///
/// The data arrives as `&Series` and is **never iterated here** — every loop over `n` is inside
/// [`Raster::build`], which runs on a cache miss.
///
/// It writes a **partition** of `area`: the gutter, the axis column, the body and the axis row are
/// disjoint and together they are every cell.
pub fn pane_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    id: Id,
    o: &Opts,
    data: &Series,
    st: &mut PaneState,
) -> Response {
    let response = cx.interact(id, area, Interest::HOVER);
    if area.w < 6 || area.h < 3 {
        return response;
    }

    let g = geom(o.kind, cx.theme().glyphs());
    let dom = st.range(data.revision(), o.range, data.points());
    let plot_h = area.h - 1;
    let gut = gutter(dom, plot_h).min(area.w - 2);
    let plot_w = area.w - gut;
    // What the list instinct would compute: the index window a virtualised collection uses.
    let cull = (0usize, usize::from(plot_h).min(data.len()));
    st.refresh(
        data.revision(),
        plot_w,
        plot_h,
        o.kind,
        g,
        dom,
        data.points(),
        o.reach,
        cull,
    );

    // The chrome's spellings come from the *lookup* half, which is the theme's and not this
    // module's — the sub-cell ladder is the only thing here that is a branch.
    let (vline, hline, corner) = {
        let theme = cx.theme();
        (
            theme.glyph(Glyph::VLine),
            theme.glyph(Glyph::HLine),
            theme.glyph(Glyph::BottomLeft),
        )
    };
    let axis = cx.theme().paint(Role::Border);
    let dim = cx.theme().paint(Role::Dim);
    let threshold_paint = cx.theme().paint(Role::Danger);
    let mut series_paints = [dim; SERIES_RGB.len()];
    for (i, p) in series_paints.iter_mut().enumerate() {
        *p = series_paint(cx.theme(), i, o.role_series);
    }

    let step = nice_step(dom.y1 - dom.y0, tick_count(plot_h));
    let dec = decimals(step) as usize;
    let span = (dom.y1 - dom.y0).max(f32::EPSILON);
    let threshold_row = o.threshold.and_then(|t| {
        let row = (((dom.y1 - t) / span) * f32::from(plot_h)) as i32;
        (row >= 0 && row < i32::from(plot_h)).then_some(row as u16)
    });
    let label_w = gut - 1;

    // **The ticks, resolved to rows once, into a fixed array.** Asking *is there a tick on this row*
    // once per row is `rows x ticks` float divisions a frame for an answer that does not depend on
    // the row.
    let mut tick_row = [u16::MAX; 64];
    let mut tick_at = [0u16; 64];
    let mut ticks = 0usize;
    {
        let mut i = 0u16;
        while ticks < 64 && i <= 128 {
            let v = tick_value(dom, step, i);
            if v > dom.y1 {
                break;
            }
            let row = (((dom.y1 - v) / span) * f32::from(plot_h)) as i32;
            if row >= 0 && row < i32::from(plot_h) {
                tick_row[ticks] = row as u16;
                tick_at[ticks] = i;
                ticks += 1;
            }
            i += 1;
        }
    }

    // ── the gutter: a label where a tick lands, blanks elsewhere, then the axis column ───────────
    for y in 0..plot_h {
        let mut drawn = 0u16;
        if let Some(k) = tick_row[..ticks].iter().position(|&r| r == y) {
            let v = tick_value(dom, step, tick_at[k]);
            st.label.clear();
            let _ = write!(
                st.label,
                "{v:>width$.dec$}",
                width = usize::from(label_w),
                dec = dec
            );
            drawn = ink.text(cx, area.x, area.y + i32::from(y), &st.label, dim);
        }
        if drawn < label_w {
            let _ = ink.run(
                cx,
                area.x + i32::from(drawn),
                area.y + i32::from(y),
                " ",
                label_w - drawn,
                dim,
            );
        }
        let _ = ink.run(
            cx,
            area.x + i32::from(label_w),
            area.y + i32::from(y),
            vline,
            1,
            axis,
        );
    }

    // ── the body: one row at a time, split into runs by who owns the cell ────────────────────────
    let PaneState { raster, row, .. } = st;
    for y in 0..plot_h {
        let mut run_owner = 0u8;
        let mut run_x = 0u16;
        row.clear();
        for x in 0..plot_w {
            let (bits, owner) = raster.at(x, y);
            let (ch, own) = if bits != 0 {
                (cluster(o.kind, g, bits), owner)
            } else if Some(y) == threshold_row {
                // **The threshold's second axis.** A rule the terminal can draw whatever the palette
                // did, which is what §16's *carried on both axes* asks for.
                (
                    if o.threshold_glyph {
                        hline.chars().next().unwrap_or('-')
                    } else {
                        ' '
                    },
                    OWNER_THRESHOLD,
                )
            } else {
                (' ', 0u8)
            };
            if own != run_owner && !row.is_empty() {
                let paint = paint_of(run_owner, &series_paints, dim, threshold_paint);
                let _ = ink.text(
                    cx,
                    area.x + i32::from(gut + run_x),
                    area.y + i32::from(y),
                    row,
                    paint,
                );
                row.clear();
                run_x = x;
            }
            if row.is_empty() {
                run_x = x;
                run_owner = own;
            }
            row.push(ch);
        }
        if !row.is_empty() {
            let paint = paint_of(run_owner, &series_paints, dim, threshold_paint);
            let _ = ink.text(
                cx,
                area.x + i32::from(gut + run_x),
                area.y + i32::from(y),
                row,
                paint,
            );
        }
    }

    // ── the axis row: the blank gutter, the corner, then the rule ────────────────────────────────
    let y = area.y + i32::from(plot_h);
    let _ = ink.run(cx, area.x, y, " ", label_w, dim);
    let _ = ink.run(cx, area.x + i32::from(label_w), y, corner, 1, axis);
    let _ = ink.run(cx, area.x + i32::from(gut), y, hline, plot_w, axis);
    response
}

/// The paint a run of cells owned by `owner` is drawn in.
///
/// **A shared cell can carry only one colour**, which is the third rung's price: a braille cell has
/// one `Paint` for all eight dots where a quadrant carries a foreground *and* a background. The
/// per-cell quadrant fallback that would recover it is named in spec §22 and not built.
fn paint_of(owner: u8, series: &[Paint; 6], dim: Paint, threshold: Paint) -> Paint {
    match owner {
        0 => dim,
        OWNER_THRESHOLD => threshold,
        SHARED => series[0],
        n => series[(usize::from(n) - 1) % series.len()],
    }
}

/// **The legend, and it is a partition of every row it draws.**
///
/// The label goes through [`fit_into`], which is where truncation is decided and where §16's
/// one-cell ellipsis holds. [`defective::legend_that_does_not_narrow`] is the same function with the
/// label written whole and the clip left to decide where it stops — **green at 300x80 and red at
/// 60x20**, which is the two-size axis this screen is played at two sizes for.
pub fn legend_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    n: usize,
    o: &Opts,
    buf: &mut String,
) {
    legend_drawn(ink, cx, area, n, o, buf, true);
}

/// [`legend_into`] and [`defective::legend_that_does_not_narrow`], **one function with one boolean
/// between them**, so a reviewer's diff is one line.
///
/// [`crate::structure::defective::panel_over_title`]'s arrangement and its reason.
#[allow(clippy::too_many_arguments)]
fn legend_drawn<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    n: usize,
    o: &Opts,
    buf: &mut String,
    narrows: bool,
) {
    if area.w < 3 || area.h == 0 {
        return;
    }
    let dim = cx.theme().paint(Role::Dim);
    let bullet = cx.theme().glyph(Glyph::Bullet);
    let mut paints = [dim; SERIES_RGB.len()];
    for (i, p) in paints.iter_mut().enumerate() {
        *p = series_paint(cx.theme(), i, o.role_series);
    }
    let opts = FitOpts {
        role: Role::Dim,
        pad: Role::Dim,
        ..Default::default()
    };
    for i in 0..usize::from(area.h).min(n) {
        let y = area.y + i as i32;
        let _ = ink.run(cx, area.x, y, bullet, 1, paints[i % paints.len()]);
        let _ = ink.run(cx, area.x + 1, y, " ", 1, dim);
        buf.clear();
        let _ = write!(buf, "series {i}");
        let row = Rect::new(area.x + 2, y, area.w - 2, 1);
        if narrows {
            let _ = fit_into(ink, cx, row, buf, &opts);
        } else {
            // **The label written whole, with the clip left to decide where it stops.** At 300x80
            // it fits and this is the correct build; at 60x20 the legend's interior is two columns
            // and the engine's clip is the *pane's* and not the interior's, so the tail lands on
            // the panel's own right border — one cell written twice, per row, for ever.
            let w = ink.text(cx, row.x, y, buf, dim);
            if w < row.w {
                let _ = ink.run(cx, row.x + i32::from(w), y, " ", row.w - w, dim);
            }
        }
    }
}

/// **The spellings ADR 0026 prices, kept because a gate nobody has watched fail is not a gate.**
pub mod defective {
    use super::{Ctx, Ink, Opts, Rect, legend_drawn};

    /// **A legend whose label is written whole and clipped**, which is
    /// [`crate::listing::does_not_narrow`] arriving on a screen whose narrow size is 60x20.
    ///
    /// Identical to [`super::legend_into`] at 300x80, where nothing truncates.
    pub fn legend_that_does_not_narrow<I: Ink>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        n: usize,
        o: &Opts,
        buf: &mut String,
    ) {
        legend_drawn(ink, cx, area, n, o, buf, false);
    }
}

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The three panes' rectangles, cut from `a` the way the screen cuts them.
///
/// **Cut from the context's own rectangle and not from constants**, which is what makes a resize
/// experiment a resize: a screen written against `W` and `H` hands the panes the same rectangle they
/// had before, and a memo-key experiment over it reports a clean bill for a key with no rectangle in
/// it.
pub fn panes(a: Rect) -> (Rect, Rect, Rect) {
    let (wide, rest) = rect::split_at_h(a, a.w * 3 / 5);
    let (mid, right) = rect::split_at_h(rest, rest.w * 3 / 4);
    (wide, mid, right)
}

/// The block a pane's chrome is drawn as. **`padded: false`**, so the interior is the rectangle
/// minus the border and the write counts are arithmetic a reader can do.
fn chrome(title: &str) -> BlockOpts<'_> {
    BlockOpts {
        title,
        bordered: true,
        padded: false,
        ..Default::default()
    }
}

/// **One frame of the whole screen.**
///
/// Three panes: a `plot` of two series, a `chart` of bars over the same data, and a legend. The two
/// components declare one region each and the chrome declares none, so the screen is [`REGIONS`].
pub fn screen_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    build: &Build,
    data: &Series,
    st: &mut ScreenState,
) {
    let (wide, mid, right) = panes(cx.area());
    let interior = block_into(ink, cx, wide, &chrome("plot"));
    if !interior.is_empty() {
        let _ = pane_into(
            ink,
            cx,
            interior,
            Id::named("series.plot"),
            &build.plot,
            data,
            &mut st.plot,
        );
    }
    let interior = block_into(ink, cx, mid, &chrome("chart"));
    if !interior.is_empty() {
        let _ = pane_into(
            ink,
            cx,
            interior,
            Id::named("series.chart"),
            &build.chart,
            data,
            &mut st.chart,
        );
    }
    let interior = block_into(ink, cx, right, &chrome("legend"));
    if !interior.is_empty() {
        if build.narrowing_legend {
            legend_into(ink, cx, interior, SERIES, &build.plot, &mut st.legend);
        } else {
            defective::legend_that_does_not_narrow(
                ink,
                cx,
                interior,
                SERIES,
                &build.plot,
                &mut st.legend,
            );
        }
    }
}

// ── the instruments ──────────────────────────────────────────────────────────────────────────────

/// **What one frame of the screen turned out to be**, returned rather than printed.
///
/// A number only a report prints is a number no gate can read — [`crate::dense::Shape`]'s rule.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Shape {
    /// Columns the engine reported written.
    pub writes: u64,
    /// Distinct cells touched. Equal to [`Shape::writes`] iff the screen is a partition.
    pub distinct: u64,
    /// **Drawing calls made. Reported and never gated as flat** — see this module's header.
    pub verbs: u64,
    /// Entries in the runtime's hit index.
    pub regions: usize,
    /// Entries in the focus ring.
    pub stops: usize,
    /// **Bytes the two rasters hold.** The size that is the rectangle at every data volume.
    pub raster_bytes: usize,
    /// **Points the last raster build looked at.** The edit's cost, as a count.
    pub touched: u64,
    /// How many times the rasters were rebuilt during the measured frames. Zero in a steady frame.
    pub misses: u32,
    /// How many times the ranges were refolded during the measured frames.
    pub range_misses: u32,
}

/// A headless driver carrying `build`'s theme.
fn driver_for(build: Build, w: u16, h: u16) -> Driver {
    let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
    driver.set_theme(build.theme());
    driver
}

/// **A screen that has already drawn its first frame**, so that everything asked of it afterwards
/// is a *steady* frame.
///
/// It exists because the allocation column cannot be filled from inside the library: `vitui-alloc-\
/// probe` is a dev-dependency and a library cannot install a global allocator on a consumer's
/// behalf, so the caller wraps [`Session::frame`] in its own counter. That is
/// [`crate::runner::Run::counters`]'s arrangement, and its reason: *a figure defaulted to zero is a
/// counter that prints `0` when it means nobody counted.*
pub struct Session {
    driver: Driver,
    data: Series,
    build: Build,
    state: ScreenState,
}

impl Session {
    /// Attach a sink, build the data, and draw the warm-up frames.
    ///
    /// **A cold frame is not a frame**: the five frame structures take their allocation on the first
    /// frame that needs one and keep it, and the pane caches take their rasters there.
    ///
    /// # Two frames and not one, and the second one is a measurement
    ///
    /// One warm-up leaves **exactly one allocation** on the next frame and none on any frame after
    /// it — measured `[1, 0, 0, 0, 0, 0, 0, 0, 0, 0]` at both sizes, which is a structure doubling
    /// once rather than a per-frame cost. [`crate::listing::volume_over`] warms with one because one
    /// is enough there; here it is not, and the number is written down rather than absorbed into a
    /// loop count somebody would later shorten.
    pub fn open(build: Build, size: (u16, u16), points: usize) -> Session {
        let mut session = Session {
            driver: driver_for(build, size.0, size.1),
            data: Series::build(points, SERIES),
            build,
            state: ScreenState::new(),
        };
        session.frame();
        session.frame();
        session
    }

    /// **One steady frame, through [`Direct`]** — the path a component takes.
    pub fn frame(&mut self) {
        let Session {
            driver,
            data,
            build,
            state,
        } = self;
        driver.frame(|cx| screen_into(&mut Direct, cx, build, data, state));
    }

    /// **One steady frame, through a [`Tally`]**, and what it turned out to be.
    ///
    /// Separate from [`Session::frame`] because a `Tally` keeps a set of every cell it sees, so a
    /// *timing* taken with one in the loop is a report about the instrument.
    pub fn tallied(&mut self) -> Shape {
        let before = (self.state.plot.misses(), self.state.chart.misses());
        let before_range = (
            self.state.plot.range_misses(),
            self.state.chart.range_misses(),
        );
        let Session {
            driver,
            data,
            build,
            state,
        } = self;
        let mut tally = Tally::new();
        driver.frame(|cx| screen_into(&mut tally, cx, build, data, state));
        let frame = driver.inspect();
        Shape {
            writes: tally.writes(),
            distinct: tally.distinct(),
            verbs: tally.verbs(),
            regions: frame.hits().len(),
            stops: frame.stop_count(),
            raster_bytes: state.plot.raster().bytes() + state.chart.raster().bytes(),
            touched: state.plot.raster().touched() + state.chart.raster().touched(),
            misses: (state.plot.misses() - before.0) + (state.chart.misses() - before.1),
            range_misses: (state.plot.range_misses() - before_range.0)
                + (state.chart.range_misses() - before_range.1),
        }
    }

    /// The two panes' state, for a caller that wants the rasters themselves.
    pub fn state(&self) -> &ScreenState {
        &self.state
    }
}

/// **The screen at one size and one data volume, as a shape.**
pub fn shape(build: Build, size: (u16, u16), points: usize) -> Shape {
    Session::open(build, size, points).tallied()
}

/// **The screen at every one of [`VOLUMES`]**, which is what the equality is asserted over.
pub fn across_volumes(build: Build, size: (u16, u16)) -> Vec<(usize, Shape)> {
    VOLUMES
        .into_iter()
        .map(|n| (n, shape(build, size, n)))
        .collect()
}

/// **The screen as a recorded surface.** The instrument every one of the false greens is caught by.
pub fn render(build: Build, size: (u16, u16), points: usize) -> Canvas {
    let data = Series::build(points, SERIES);
    let mut driver = driver_for(build, size.0, size.1);
    let mut state = ScreenState::new();
    let mut pen = Pen::new(size.0, size.1);
    for _ in 0..2 {
        driver.frame(|cx| screen_into(&mut pen, cx, &build, &data, &mut state));
    }
    pen.into_canvas()
}

/// **What the frame costs a component**, drawn through [`Direct`] rather than through a [`Pen`].
///
/// A report and never a gate. Separate from [`shape`] for [`crate::listing::volume_cost`]'s reason:
/// a `Tally` keeps a set of every cell it sees, so a figure taken with one in the loop is a report
/// about the instrument.
///
/// # Panics
///
/// Panics on zero frames.
pub fn cost(build: Build, size: (u16, u16), points: usize, frames: u32) -> Duration {
    assert!(frames > 0, "a per-frame figure needs a frame");
    let mut session = Session::open(build, size, points);
    let mut best = Duration::MAX;
    for _ in 0..frames {
        let started = Instant::now();
        session.frame();
        best = best.min(started.elapsed());
    }
    best
}

/// **[`Raster::build`] timed on its own, with the points it looked at.**
///
/// This is the cost the invariant *permits* to be proportional to the data, and it is the only such
/// cost in the component. Minimum of three, which is `vitui-bench`'s own rule.
pub fn edit_cost(
    points: usize,
    w: u16,
    h: u16,
    kind: Kind,
    set: GlyphSet,
    reach: Reach,
) -> (Duration, u64) {
    let data = Series::build(points, SERIES);
    let g = geom(kind, set);
    let dom = domain_of(data.points(), Range::Whole).padded();
    let mut raster = Raster::empty();
    let cull = (0usize, usize::from(h).min(data.len()));
    raster.build(w, h, kind, g, dom, data.points(), reach, cull);
    let mut best = Duration::MAX;
    for _ in 0..3 {
        let started = Instant::now();
        raster.build(w, h, kind, g, dom, data.points(), reach, cull);
        best = best.min(started.elapsed());
    }
    (best, raster.touched())
}

/// **The same call on a cache hit**, averaged over `n`. The other half of the ratio.
///
/// # Panics
///
/// Panics on zero iterations.
pub fn hit_cost(points: usize, w: u16, h: u16, kind: Kind, set: GlyphSet, n: u32) -> Duration {
    assert!(n > 0, "a per-call figure needs a call");
    let data = Series::build(points, SERIES);
    let g = geom(kind, set);
    let dom = domain_of(data.points(), Range::Whole).padded();
    let mut st = PaneState::new();
    let cull = (0usize, usize::from(h).min(data.len()));
    let rev = data.revision();
    st.refresh(rev, w, h, kind, g, dom, data.points(), Reach::Mapped, cull);
    let started = Instant::now();
    for _ in 0..n {
        st.refresh(
            std::hint::black_box(rev),
            std::hint::black_box(w),
            h,
            kind,
            g,
            dom,
            data.points(),
            Reach::Mapped,
            cull,
        );
    }
    started.elapsed() / n
}

/// **Three numbers about two surfaces, because a lost series and a lost colour are separable.**
///
/// The middle number alone hides which.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Split {
    /// Cells whose cluster differs.
    pub cluster: usize,
    /// Cells whose cluster **or** paint differs.
    pub any: usize,
    /// Cells the second surface left blank where the first had something.
    pub blanked: usize,
    /// Rows carrying at least one differing cluster.
    pub rows: usize,
    /// The first differing cluster in reading order.
    pub first: Option<(u16, u16)>,
}

impl Split {
    /// **Cells that differ by style and not by cluster.** The signature of a colour-only distinction
    /// dying.
    pub fn style_only(self) -> usize {
        self.any - self.cluster
    }

    /// Whether the two surfaces agree everywhere in the rectangle.
    pub fn clean(self) -> bool {
        self.any == 0
    }

    /// The cluster half as a [`Diff`], which is the form every defect on this map is legible in.
    pub fn as_diff(self, over: (u16, u16)) -> Diff {
        Diff {
            cells: self.cluster,
            rows: self.rows,
            first: self.first,
            over,
        }
    }
}

/// **Compare two surfaces inside one rectangle**, so a defect confined to one pane is not diluted by
/// the pane beside it.
pub fn split_diff(a: &Canvas, b: &Canvas, r: Rect) -> Split {
    let mut out = Split::default();
    for y in r.y..r.bottom() {
        let mut row_differs = false;
        for x in r.x..r.right() {
            let (Ok(x), Ok(y)) = (u16::try_from(x), u16::try_from(y)) else {
                continue;
            };
            let (left, right) = (a.get(x, y), b.get(x, y));
            let text = |c: Option<&crate::runner::Cell>| c.map(|c| c.cluster.clone());
            let paint = |c: Option<&crate::runner::Cell>| c.map(|c| c.paint);
            let cluster_differs = text(left) != text(right);
            if cluster_differs {
                out.cluster += 1;
                row_differs = true;
                if out.first.is_none() {
                    out.first = Some((x, y));
                }
                if right.is_none_or(|c| c.cluster == " " || c.cluster.is_empty()) {
                    out.blanked += 1;
                }
            }
            if cluster_differs || paint(left) != paint(right) {
                out.any += 1;
            }
        }
        if row_differs {
            out.rows += 1;
        }
    }
    out
}

/// The whole screen, as a rectangle.
pub fn whole(size: (u16, u16)) -> Rect {
    Rect::new(0, 0, size.0, size.1)
}

/// **The plot pane's interior**, which is where the third repertoire rung is earned.
pub fn plot_pane(size: (u16, u16)) -> Rect {
    rect::inset(panes(whole(size)).0, 1)
}

/// **The chart pane's interior**, which is where it is not.
pub fn chart_pane(size: (u16, u16)) -> Rect {
    rect::inset(panes(whole(size)).1, 1)
}

// ── the subject, and the scan that says whether it is here ───────────────────────────────────────

/// **The two components these scenes are scenes of, and neither is declared yet.**
pub const SUBJECTS: [&str; 2] = ["chart", "plot"];

/// Where [`SUBJECTS`] belong, as `(module file, the declaration)`.
///
/// The home is the freeze's, joined through [`crate::Family`]: both name `F10Charts`, whose module
/// is `chart.rs`. A component is `fn(&mut Ctx, Rect, …) -> Response` (spec §1, rule 1), so the thing
/// to look for is a public function of the component's own name in its own family's module.
pub const DECLARATIONS: [(&str, &str); 2] =
    [("chart.rs", "pub fn chart("), ("chart.rs", "pub fn plot(")];

/// **Which of [`SUBJECTS`] this crate actually declares. Today: neither.**
///
/// A source scan and not a `use`, for [`crate::dense::subjects_declared`]'s reason: *the item does
/// not exist* has no expression, and a `compile_fail` fence would pass today and pass again the day
/// somebody renames the module. The predicate is `crate::dense::declares`, shared rather than
/// copied — one definition of *a line that is not a comment*.
pub fn subjects_declared() -> Vec<&'static str> {
    let mut out = Vec::new();
    for (subject, (file, declaration)) in SUBJECTS.into_iter().zip(DECLARATIONS) {
        let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).join(file);
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        if crate::dense::declares(&source, declaration) {
            out.push(subject);
        }
    }
    out
}

/// **Whether the screen stands on its subjects, as a verdict rather than as a sentence.**
///
/// `Unmet` over two, inverted by **components 28**. Everything the screen itself can be asked is
/// measured and green; what is missing is the subjects.
pub fn standing() -> Verdict {
    let declared = subjects_declared();
    Verdict::of(
        SUBJECTS.len(),
        SUBJECTS.len() - declared.len(),
        "neither `chart` nor `plot` is declared in this crate, so what stands on the series screen \
         is a stand-in pane and not the components. The screen, its 2 regions, its 21 872 writes at \
         1k / 100k / 1M, its equality at 300x80 and at 60x20, its three false greens and its \
         175 712 axis pairs are measured and green; what is missing is the subject",
        "components 28",
    )
}

/// **The sentence a scene waiting for its subject fails with**, or `None` once both are declared.
///
/// Criterion 7, and it is the distinction the whole ticket rests on: a scene that fails because it
/// is unimplemented and a scene that fails because the code is wrong are **the same failure** unless
/// the message separates them. This one names the subjects, the file they belong in, the
/// declarations to look for, and the ticket — and says in as many words that it is not a defect in
/// the screen.
///
/// It takes the declaration list as an argument for [`crate::dense::owed_message`]'s reason: the day
/// the crate is on the other side of it, the hostile case is still one call away.
pub fn owed_message(declared: &[&str], scene: &str) -> Option<String> {
    if declared.len() == SUBJECTS.len() {
        return None;
    }
    let owed: Vec<String> = SUBJECTS
        .into_iter()
        .zip(DECLARATIONS)
        .filter(|(id, _)| !declared.contains(id))
        .map(|(id, (file, declaration))| format!("`{id}` (`src/{file}`: `{declaration}…)`)"))
        .collect();
    Some(format!(
        "{scene} is not standing, and it is waiting for its subject rather than failing: {} of {} \
         components are undeclared — {}. This is not a defect in the screen. The series screen is \
         drawn, its two regions are counted, its writes are identical at 1 000, 100 000 and \
         1 000 000 points at both sizes, its three false greens are caught on the rendered surface \
         and its axis sweep visits all 175 712 pairs — see `crate::series::tests`. Inverted by \
         `components 28`",
        owed.len(),
        SUBJECTS.len(),
        owed.join(", "),
    ))
}

/// **Fail with the subjects that are missing, the file they belong in, and the ticket.**
///
/// # Panics
///
/// Panics while [`SUBJECTS`] are undeclared, which is **today**. Components ticket 28 inverts it.
pub fn assert_stands_up(scene: &str) {
    if let Some(message) = owed_message(&subjects_declared(), scene) {
        panic!("{message}");
    }
}

// ── the numbers this screen is measured at ───────────────────────────────────────────────────────

/// **Rect a correct frame writes at 300x80, at every one of [`VOLUMES`]. 21 872.**
///
/// It is arithmetic a reader can do: `300 * 80` is 24 000, the legend's interior is `28 * 78` and it
/// draws only [`SERIES`] of those rows, so `24 000 - (2 184 - 56)` is 21 872. §13's own figure,
/// reproduced exactly.
pub const WRITES: u64 = 21_872;

/// **Rect a correct frame writes at 60x20. 1 136**, by the same arithmetic: `1 200 - (72 - 8)`.
pub const NARROW_WRITES: u64 = 1_136;

/// **Cells the legend that does not narrow writes twice at 60x20. Two** — one a row, for ever.
pub const NARROW_DOUBLE_WRITES: u64 = 2;

/// **Cells the two legend arms disagree about at 60x20. Four over two rows.**
pub const NARROW_LEGEND_CELLS: usize = 4;

/// **Cells the two legend arms disagree about at 300x80. None**, which is why the scene needs the
/// second size at all.
pub const WIDE_LEGEND_CELLS: usize = 0;

/// **A 60x20 raster, in bytes. 2 400 — at every data volume**, which is the whole claim.
pub const RASTER_BYTES_60X20: usize = 2 * 60 * 20;

/// The data volume the surface comparisons are taken at.
///
/// **Two hundred thousand and not a million**, and the reason is the instrument rather than the
/// claim: a [`Pen`] records every cell of a 300x80 surface and the equality is over the *picture*,
/// which the volume does not change once it is past the point where every column is occupied. The
/// invariance claim is [`across_volumes`]'s and it is asserted at all three.
pub const COMPARED_AT: usize = 200_000;

/// **Cells the plot pane gains from the third repertoire rung. 882.**
pub const RUNG_PLOT_CELLS: usize = 882;

/// **Cells the chart pane gains from it. None** — a bar chart at Extended is byte-identical to one
/// at Unicode, so the third rung is `plot`'s alone.
pub const RUNG_CHART_CELLS: usize = 0;

/// **Cells ASCII and Unicode disagree about, over the whole screen. 7 276, over 80 of 80 rows** —
/// ADR 0009 literally, a different construction rather than the same one with worse glyphs.
pub const ASCII_CELLS: usize = 7_276;

/// Cells the culled arm gets wrong.
pub const CULLED_CELLS: usize = 5_546;
/// How many of them it leaves blank.
pub const CULLED_BLANKED: usize = 5_500;
/// Cells the sampled-range arm gets wrong.
pub const SAMPLED_CELLS: usize = 3_431;
/// How many rows carry one.
pub const SAMPLED_ROWS: usize = 78;
/// Cells the strided arm gets wrong.
pub const STRIDED_CELLS: usize = 2_953;
/// How many of them it leaves blank.
pub const STRIDED_BLANKED: usize = 2_544;

/// **Cells of the plot pane where two series share one. Twenty-five**, and it is the same at the
/// quadrant rung and at the braille rung — which is what makes the third rung's colour price a
/// price rather than a rounding.
pub const SHARED_CELLS: u32 = 25;

/// **Fixpoint pairs that reach one.** 175 248 of [`AXIS_PAIRS`].
pub const AXIS_CONVERGED: u64 = 175_248;
/// **Fixpoint pairs that never do. 464.**
pub const AXIS_OSCILLATES: u64 = 464;
/// The most passes a converging pair needs. Four.
pub const AXIS_MAX_PASSES: u8 = 4;

#[cfg(test)]
mod tests {
    use super::*;

    /// A 60x20 raster built over `points`, which is §13's own object.
    fn raster_60x20(points: usize) -> Raster {
        let data = Series::build(points, SERIES);
        let g = geom(Kind::Marks, GlyphSet::Extended);
        let dom = domain_of(data.points(), Range::Whole).padded();
        let mut raster = Raster::empty();
        raster.build(
            60,
            20,
            Kind::Marks,
            g,
            dom,
            data.points(),
            Reach::Mapped,
            (0, 20),
        );
        raster
    }

    /// The build with the legend that does not narrow, which is the two-size axis' other arm.
    fn does_not_narrow() -> Build {
        Build {
            narrowing_legend: false,
            ..Build::correct()
        }
    }

    /// **The frame costs the rectangle and not the data, at both sizes — and the verbs do not.**
    ///
    /// The screen with the best excuse not to pass this passes it: three data volumes, one
    /// rectangle, every *count* identical. **Verbs are not one of them and must not be gated as
    /// one**: a run ends where a cell's owner changes, so the verb count tracks the picture and is
    /// **not monotone in `n`** — which is the proof rather than an excuse. What is gated is the bound
    /// the invariant actually claims, `verbs <= writes`, and §21 says *never verb equality across
    /// sizes* in as many words.
    #[test]
    fn the_frame_costs_the_rectangle_and_the_verbs_track_the_picture() {
        for (size, writes) in [((W, H), WRITES), ((NARROW_W, NARROW_H), NARROW_WRITES)] {
            let across = across_volumes(Build::correct(), size);
            assert_eq!(across.len(), VOLUMES.len());
            let mut verbs = Vec::new();
            for (points, shape) in &across {
                assert_eq!(
                    shape.writes, writes,
                    "{}x{} at {points} points",
                    size.0, size.1
                );
                assert_eq!(
                    shape.distinct, writes,
                    "the screen is a partition: no cell written twice, at {points} points"
                );
                assert_eq!(
                    shape.regions, REGIONS,
                    "one region a component and none for the \
                                                   chrome"
                );
                assert_eq!(shape.stops, 0, "neither pane declares a tab stop");
                assert_eq!(
                    (shape.misses, shape.range_misses),
                    (0, 0),
                    "a steady frame refolded the data at {points} points"
                );
                assert!(
                    shape.verbs <= shape.writes,
                    "{} verbs over {} writes",
                    shape.verbs,
                    shape.writes
                );
                verbs.push(shape.verbs);
            }
            // **And the verbs really do move**, or the relation above would be a gate over a
            // constant and `never verb equality` would be a sentence about nothing.
            assert!(
                verbs.iter().any(|v| *v != verbs[0]),
                "{}x{}: the verb count did not move with the data, so the relation is untested",
                size.0,
                size.1
            );
        }
    }

    /// **The verb count is not monotone in `n`**, at the size §13 states it at.
    ///
    /// §13's own 2 551 / 3 772 / 2 472 does not reproduce and the **shape** does: this screen is
    /// **2 554 / 3 775 / 2 475**, three more at every volume and the same rise and fall. The three
    /// are structural and are the same three at every volume — one verb a pane, from the chrome
    /// going through `crate::frame::block_into` rather than through a prototype's own border loop —
    /// so the difference between the volumes, which is what the claim is about, is identical.
    #[test]
    fn the_verb_count_rises_and_falls_with_the_picture_and_not_with_the_volume() {
        let across = across_volumes(Build::correct(), (W, H));
        let verbs: Vec<u64> = across.iter().map(|(_, s)| s.verbs).collect();
        assert_eq!(verbs, vec![2_554, 3_775, 2_475]);
        assert!(
            verbs[1] > verbs[0] && verbs[2] < verbs[0],
            "the count is monotone, which is the shape §13 says it does not have: {verbs:?}"
        );
        // The differences, which are what the claim is about and what reproduces exactly.
        assert_eq!(verbs[1] - verbs[0], 1_221);
        assert_eq!(verbs[0] - verbs[2], 79);
    }

    /// **A 60x20 raster is 2 400 B at 1 000, 100 000 and 1 000 000 points.**
    ///
    /// The size is the rectangle; the contents are the data, and [`Raster::touched`] is where the
    /// volume is allowed to appear.
    #[test]
    fn a_sixty_by_twenty_raster_is_two_thousand_four_hundred_bytes_at_every_volume() {
        let mut touched = Vec::new();
        for points in VOLUMES {
            let raster = raster_60x20(points);
            assert_eq!(raster.bytes(), RASTER_BYTES_60X20, "at {points} points");
            assert_eq!((raster.w(), raster.h()), (60, 20));
            touched.push(raster.touched());
        }
        assert_eq!(
            touched,
            VOLUMES.iter().map(|n| *n as u64 * 2).collect::<Vec<_>>(),
            "the edit's cost is the data's, and it is the only thing here that is"
        );
    }

    /// **The screen's own rasters are the plotting rectangle, and the plotting rectangle moves with
    /// the domain.**
    ///
    /// A finding rather than a caveat: the raster's size is the rectangle *the axis left over*, and
    /// the gutter is as wide as the widest tick label, which is data. At 100 000 and 1 000 000 points
    /// the two agree to the byte; at 1 000 the domain is narrower, the labels are shorter and the
    /// panes each keep one column more. Nothing about the *volume* moves it — [`across_volumes`]'s
    /// write counts are identical at all three — and the honest statement is that *the size is the
    /// rectangle* holds exactly, while *the rectangle is independent of the data* does not.
    #[test]
    fn the_panes_rasters_are_the_plotting_rectangle_and_the_gutter_is_data() {
        let at = |n: usize| shape(Build::correct(), (W, H), n).raster_bytes;
        assert_eq!(at(VOLUMES[1]), at(VOLUMES[2]));
        assert_ne!(at(VOLUMES[0]), at(VOLUMES[1]));
        assert_eq!(at(VOLUMES[0]) - at(VOLUMES[1]), 308);
    }

    /// **The two-size axis, in both directions: green at 300x80 and red at 60x20.**
    ///
    /// The correct legend and the one that writes its label whole are **one function with one
    /// boolean between them**, and at 300x80 they are indistinguishable — identical writes,
    /// identical verbs, zero cells apart. At 60x20 the interior is two columns, the clip belongs to
    /// the pane rather than to the interior, and the tail of every label lands on the panel's own
    /// right border: **two cells written twice, and one verb *fewer* than the correct build.**
    #[test]
    fn the_legend_that_does_not_narrow_is_green_at_three_hundred_and_red_at_sixty() {
        let wide = shape(Build::correct(), (W, H), VOLUMES[0]);
        let wide_bad = shape(does_not_narrow(), (W, H), VOLUMES[0]);
        assert_eq!(wide, wide_bad, "the two arms differ at the wide size");
        assert_eq!(
            split_diff(
                &render(Build::correct(), (W, H), VOLUMES[0]),
                &render(does_not_narrow(), (W, H), VOLUMES[0]),
                whole((W, H)),
            )
            .cluster,
            WIDE_LEGEND_CELLS
        );

        let narrow = shape(Build::correct(), (NARROW_W, NARROW_H), VOLUMES[0]);
        let narrow_bad = shape(does_not_narrow(), (NARROW_W, NARROW_H), VOLUMES[0]);
        assert_eq!(narrow.writes, NARROW_WRITES);
        assert_eq!(
            narrow.distinct, NARROW_WRITES,
            "the correct arm is a partition"
        );
        assert_eq!(
            narrow_bad.writes - narrow_bad.distinct,
            NARROW_DOUBLE_WRITES,
            "the defect is a double write and there is no third counter that sees it"
        );
        assert_eq!(
            narrow_bad.distinct, narrow.distinct,
            "and it touches exactly the cells the correct arm touches, which is why `distinct` \
             cannot see it either"
        );
        assert!(
            narrow_bad.verbs < narrow.verbs,
            "the defective arm did not look cheaper, so the scene's argument is untested: {} \
             against {}",
            narrow_bad.verbs,
            narrow.verbs
        );

        let diff = split_diff(
            &render(Build::correct(), (NARROW_W, NARROW_H), VOLUMES[0]),
            &render(does_not_narrow(), (NARROW_W, NARROW_H), VOLUMES[0]),
            whole((NARROW_W, NARROW_H)),
        );
        assert_eq!((diff.cluster, diff.rows), (NARROW_LEGEND_CELLS, SERIES));
    }

    /// **False green one: culling to the visible index window.**
    ///
    /// It is what a virtualised collection does and it is correct there. Here an index is not an x
    /// coordinate: it draws **the same number of cells**, faster, and loses the series.
    #[test]
    fn culling_to_the_visible_index_window_loses_the_series_and_writes_the_same() {
        let good = render(Build::correct(), (W, H), COMPARED_AT);
        let build = Build::correct().both(|o| o.reach = Reach::CulledByIndex);
        let bad = render(build, (W, H), COMPARED_AT);
        let diff = split_diff(&good, &bad, whole((W, H)));
        assert_eq!(
            (diff.cluster, diff.blanked),
            (CULLED_CELLS, CULLED_BLANKED),
            "{:?}",
            diff
        );
        assert_eq!(
            shape(build, (W, H), COMPARED_AT).writes,
            WRITES,
            "the write count is unmoved, which is why it cannot be the gate"
        );
        assert!(
            shape(build, (W, H), COMPARED_AT).touched < 1_000,
            "the defect did not look cheaper on the one counter that can see it"
        );
    }

    /// **False green two: an axis range from a stride sample.**
    ///
    /// It optimises a cost the memo had already removed, it is *slower* than the fold it replaces,
    /// and the domain it produces stops **26 units short of the data**.
    #[test]
    fn a_sampled_axis_range_is_wrong_and_buys_nothing() {
        let good = render(Build::correct(), (W, H), COMPARED_AT);
        let build = Build::correct().both(|o| o.range = Range::Sampled(1_000));
        let bad = render(build, (W, H), COMPARED_AT);
        let diff = split_diff(&good, &bad, whole((W, H)));
        assert_eq!(
            (diff.cluster, diff.rows),
            (SAMPLED_CELLS, SAMPLED_ROWS),
            "{diff:?}"
        );

        let data = Series::build(COMPARED_AT, SERIES);
        let whole_domain = domain_of(data.points(), Range::Whole).padded();
        let sampled = domain_of(data.points(), Range::Sampled(1_000)).padded();
        assert_eq!(
            (
                format!("{:.2}", whole_domain.y0),
                format!("{:.2}", whole_domain.y1)
            ),
            ("30.00".to_string(), "103.07".to_string())
        );
        assert_eq!(
            (format!("{:.2}", sampled.y0), format!("{:.2}", sampled.y1)),
            ("30.00".to_string(), "77.05".to_string()),
            "the stride happened not to see the spikes, which is the whole of the defect"
        );
        assert!(sampled.y1 < whole_domain.y1);
    }

    /// **False green three: every *n*-th point instead of the union.**
    ///
    /// The output has the same shape and the same frame cost, and the peaks are gone. The union does
    /// not lose them because a union is idempotent and an extremum is a set member.
    #[test]
    fn striding_loses_the_peaks_and_the_union_does_not() {
        let good = render(Build::correct(), (W, H), COMPARED_AT);
        let build = Build::correct().both(|o| o.reach = Reach::Strided);
        let bad = render(build, (W, H), COMPARED_AT);
        let diff = split_diff(&good, &bad, whole((W, H)));
        assert_eq!(
            (diff.cluster, diff.blanked),
            (STRIDED_CELLS, STRIDED_BLANKED),
            "{diff:?}"
        );
        assert_eq!(shape(build, (W, H), COMPARED_AT).writes, WRITES);
    }

    /// **The third rung is the plot's and not the chart's, as a count on the rendered surface.**
    ///
    /// A bar chart at Extended is byte-identical to one at Unicode — **0 cells** — and a plot is not
    /// — **882**. That is C09's handed-over question answered by measurement rather than by argument.
    #[test]
    fn the_third_rung_is_the_plots_and_not_the_charts() {
        let unicode = render(Build::correct().at(GlyphSet::Unicode), (W, H), COMPARED_AT);
        let extended = render(Build::correct().at(GlyphSet::Extended), (W, H), COMPARED_AT);
        assert_eq!(
            split_diff(&unicode, &extended, chart_pane((W, H))).any,
            RUNG_CHART_CELLS,
            "a bar chart gained something from the third rung"
        );
        assert_eq!(
            split_diff(&unicode, &extended, plot_pane((W, H))).cluster,
            RUNG_PLOT_CELLS,
            "a plot gained nothing from the third rung"
        );
    }

    /// **ASCII is a different construction and not the same one with worse glyphs.**
    ///
    /// 7 276 cells over **80 of 80 rows**, which is ADR 0009 literally.
    #[test]
    fn ascii_is_a_different_construction_over_every_row_of_the_screen() {
        let ascii = render(Build::correct().at(GlyphSet::Ascii), (W, H), COMPARED_AT);
        let unicode = render(Build::correct().at(GlyphSet::Unicode), (W, H), COMPARED_AT);
        let diff = split_diff(&ascii, &unicode, whole((W, H)));
        assert_eq!(
            (diff.cluster, diff.rows),
            (ASCII_CELLS, H as usize),
            "{diff:?}"
        );
    }

    /// **The ladders, as the branch states them.** 1 / 8 / 8 for a prefix, 1x1 / 2x2 / 2x4 for a set.
    ///
    /// The vertical resolution doubles at the third rung and the horizontal does not, which is what
    /// makes that rung worth exactly one bit of `y` and nothing else.
    #[test]
    fn the_ladders_are_one_eight_eight_and_one_by_one_two_by_two_two_by_four() {
        let rungs = [GlyphSet::Ascii, GlyphSet::Unicode, GlyphSet::Extended];
        let bars: Vec<(u8, u8)> = rungs
            .iter()
            .map(|set| {
                let g = geom(Kind::Bars, *set);
                (g.sx, g.sy)
            })
            .collect();
        assert_eq!(bars, vec![(1, 1), (1, 8), (1, 8)]);
        let marks: Vec<(u8, u8)> = rungs
            .iter()
            .map(|set| {
                let g = geom(Kind::Marks, *set);
                (g.sx, g.sy)
            })
            .collect();
        assert_eq!(marks, vec![(1, 1), (2, 2), (2, 4)]);

        // And the states, which is the column the third rung has to be argued from: 2 / 9 / 9
        // against 2 / 16 / 256.
        let bar_states: Vec<u32> = rungs
            .iter()
            .map(|s| geom(Kind::Bars, *s).states(Kind::Bars))
            .collect();
        assert_eq!(bar_states, vec![2, 9, 9]);
        let mark_states: Vec<u32> = rungs
            .iter()
            .map(|s| geom(Kind::Marks, *s).states(Kind::Marks))
            .collect();
        assert_eq!(mark_states, vec![2, 16, 256]);
    }

    /// **Every spelling this branch produces is exactly one cell**, which is the rule the theme's
    /// table keeps for a glyph and which a *branch* has to keep for itself.
    #[test]
    fn every_construction_spells_exactly_one_cell() {
        for kind in [Kind::Bars, Kind::Marks] {
            for set in [GlyphSet::Ascii, GlyphSet::Unicode, GlyphSet::Extended] {
                let g = geom(kind, set);
                for bits in 0u16..=255 {
                    let spelled = cluster(kind, g, bits as u8);
                    let mut buf = [0u8; 4];
                    assert_eq!(
                        vitui_runtime::layout::text::width(spelled.encode_utf8(&mut buf)),
                        1,
                        "{kind:?} {set:?} {bits}"
                    );
                }
            }
        }
    }

    /// **The third rung buys resolution and pays in series colour**, and the payment is a count.
    ///
    /// Twenty-five cells of the plot pane carry two series at once. A braille cell has one `Paint`
    /// for all eight dots; a quadrant carries a foreground *and* a background — so the ladder is not
    /// monotone in what it can express. The per-cell quadrant fallback that would recover it is named
    /// in spec §22 and not built.
    #[test]
    fn the_third_rung_costs_colour_where_two_series_share_a_cell() {
        for set in [GlyphSet::Unicode, GlyphSet::Extended] {
            let data = Series::build(COMPARED_AT, SERIES);
            let g = geom(Kind::Marks, set);
            let dom = domain_of(data.points(), Range::Whole).padded();
            let mut raster = Raster::empty();
            raster.build(
                174,
                77,
                Kind::Marks,
                g,
                dom,
                data.points(),
                Reach::Mapped,
                (0, 77),
            );
            assert_eq!(raster.shared(), SHARED_CELLS, "at {set:?}");
        }
    }

    /// **The axis loop, over all 175 712 viewport x dataset pairs.**
    ///
    /// The naive fixpoint converges on 175 248 in at most four passes and **oscillates on 464**. The
    /// hysteresis form never loops and settles on a gutter that is not a fixed point of its own rule
    /// on **all 464** — §9's outcome exactly. Computing the gutter from the whole domain is 175 712
    /// pairs, **0 oscillations, always 1 pass**: there is no edge left to oscillate in.
    #[test]
    fn the_axis_loop_oscillates_on_four_hundred_and_sixty_four_and_the_whole_domain_on_none() {
        let tally = axis_sweep();
        assert_eq!(tally.pairs, AXIS_PAIRS);
        assert_eq!(tally.pairs, 175_712, "the product §21 states");
        assert_eq!(tally.converged, AXIS_CONVERGED);
        assert_eq!(tally.oscillates, AXIS_OSCILLATES);
        assert_eq!(tally.converged + tally.oscillates, tally.pairs);
        assert_eq!(tally.max_passes, AXIS_MAX_PASSES);
        assert_eq!(
            tally.hysteresis_settled_off, AXIS_OSCILLATES,
            "the hysteresis form settles on a non-fixed-point gutter on all of them"
        );
        assert_eq!(tally.whole_pairs, AXIS_PAIRS);
        assert_eq!(tally.whole_oscillates, 0);
        assert_eq!(tally.whole_max_passes, 1);
    }

    /// **The precondition §9 needs is false for ordinary data, and here is the instance.**
    ///
    /// *A narrower plotting area may not produce a wider label* is what would make the loop
    /// converge. It fails here because the window's own contents change: a narrower area shows fewer
    /// samples, dropping the older and larger ones, and what is left is small decimals whose labels
    /// are **wider** than the integers they replaced. Found by looking rather than by construction —
    /// the pair below is the first one the sweep's own datasets produce.
    ///
    /// # §13's illustration is not reachable on a 1 / 2 / 5 ladder, and it is not the mechanism
    ///
    /// §13 offers `0 2.5 5 7.5 10` against `0 5 10` as the reason fewer ticks are not a subset of
    /// more. `nice_step(10, 5)` on a 1 / 2 / 5 x 10^k ladder is **2.0**, not 2.5, so that particular
    /// pair needs a ladder with 2.5 on it. The *finding* survives untouched — the sweep oscillates on
    /// 464 pairs — because the mechanism that produces it is the second sentence of §13's own
    /// paragraph: *dropping older, larger samples can leave `-0.05` where `900` was.*
    #[test]
    fn a_narrower_plotting_area_can_produce_a_wider_label() {
        let ys = dataset(0, AXIS_POINTS);
        let live = Live::over(&ys, *AXIS_W.end());
        let plot_h = 78u16;
        let mut found = None;
        for narrow in 1u16..300 {
            let wide = narrow + 1;
            let a = gutter(live.window_domain(narrow), plot_h);
            let b = gutter(live.window_domain(wide), plot_h);
            if a > b {
                found = Some((narrow, a, wide, b));
                break;
            }
        }
        let (narrow, a, wide, b) = found.expect(
            "no narrowing produced a wider gutter, so the precondition holds and nothing here \
             could oscillate",
        );
        assert!(narrow < wide && a > b, "{narrow} -> {a}, {wide} -> {b}");

        // And the two domains really are different windows on one series, which is what makes this
        // a fact about the data rather than about the arithmetic.
        assert_ne!(
            live.window_domain(narrow).bits(),
            live.window_domain(wide).bits()
        );

        // The ladder, asserted rather than assumed, because §13's illustration needs a rung it does
        // not have.
        assert_eq!(nice_step(10.0, 5), 2.0);
        assert_eq!(nice_step(10.0, 3), 5.0);
    }

    /// **The screen is red because `chart` and `plot` are not declared**, and the verdict says so
    /// over a population of two rather than returning green over nothing.
    #[test]
    fn the_series_screen_is_red_because_chart_and_plot_are_not_declared() {
        assert_eq!(SUBJECTS, ["chart", "plot"]);
        assert_eq!(subjects_declared(), Vec::<&str>::new());
        let verdict = standing();
        assert!(!verdict.met());
        assert!(
            matches!(
                verdict,
                Verdict::Unmet {
                    over: 2,
                    failing: 2,
                    ..
                }
            ),
            "{verdict:?}"
        );
    }

    /// **The waiting message separates *unimplemented* from *wrong*.** Ticket 09's criterion 7.
    ///
    /// It names the subjects, the file, the declarations and the ticket, and says in as many words
    /// that it is not a defect in the screen. The other direction is watched through the declaration
    /// list, so the hostile case stays one call away after ticket 28 lands.
    #[test]
    fn the_waiting_message_separates_unimplemented_from_wrong() {
        let message = owed_message(&[], "scene 15").expect("neither subject is declared");
        assert!(message.contains("waiting for its subject rather than failing"));
        assert!(message.contains("not a defect in the screen"));
        assert!(message.contains("`chart`"));
        assert!(message.contains("`plot`"));
        assert!(message.contains("pub fn chart("));
        assert!(message.contains("components 28"));

        // One of two declared is still a scene that is not standing, and the message says which.
        let half = owed_message(&["chart"], "scene 15").expect("one of two is not standing");
        assert!(half.contains("1 of 2"));
        assert!(!half.contains("`chart` (`src/chart.rs`: `pub fn chart("));

        // And the day both are declared it stops being a failure at all.
        assert!(owed_message(&["chart", "plot"], "scene 15").is_none());
    }

    /// **The scan finds a declaration when there is one and not when it is a comment.**
    ///
    /// Through `crate::dense::declares`, which is one definition of *a line that is not a comment* —
    /// shared rather than copied, because that is the only thing that makes *fires in both
    /// directions* mean anything.
    #[test]
    fn the_subject_scan_finds_a_declaration_when_there_is_one() {
        assert!(crate::dense::declares(
            "pub fn plot(cx: &mut Ctx<'_, '_>) -> Response {",
            "pub fn plot("
        ));
        assert!(!crate::dense::declares(
            "// one day there will be a pub fn plot( here",
            "pub fn plot("
        ));
        assert!(!crate::dense::declares("", "pub fn plot("));
    }

    /// **`assert_stands_up` panics today, and its message is the one above.**
    #[test]
    #[should_panic(expected = "waiting for its subject rather than failing")]
    fn the_screen_is_watched_refusing_to_stand_up() {
        assert_stands_up("scene 15");
    }
}
