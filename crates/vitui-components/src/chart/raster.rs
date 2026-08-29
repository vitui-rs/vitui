//! **The rasteriser and its memo chain — the whole of §13's answer to the data-volume invariant.**
//!
//! Components ticket 28, spec §13. A collection satisfies *frame cost is proportional to visible
//! cells, never to data volume* by **never folding**: the window says which rows can be reached and
//! the rest are never touched. **A plot cannot do that.** Every one of a million points may land
//! inside the rectangle, and which ones do is not knowable without looking at them. So the invariant
//! is satisfied in a different shape:
//!
//! > **The frame costs the rectangle. The edit costs the data. There is no third option, and the
//! > memo is the only thing standing between them.**
//!
//! What is memoised is not *the maximum* and not *the buckets*. It is the **raster** — one sub-cell
//! bitmask per cell of the rectangle, `w x h` bytes — because that is the smallest object whose
//! *size* is the rectangle and whose *contents* are the data. A 60x20 raster is **2 400 B at every
//! data volume**.
//!
//! # Two properties carry more weight than the size
//!
//! **It is a union, and a union is idempotent.** A point sets bits; two points in one cell set the
//! union of their bits. So a column's extrema survive *by construction*, and **downsampling is not a
//! technique this component contains** — there is no `downsample`, no `stride_sample` and no
//! `every_nth` in this crate, and `crate::series::tests` names each absent spelling by path.
//!
//! **Its key is every input.** The rectangle's width and height are inputs because they *are* the
//! output's shape; the domain is an input because it is the map from values to sub-rows; and the
//! sub-cell geometry is an input because it comes from the repertoire and changes the number of
//! samples asked of the data.
//!
//! # The memo is a chain and not one memo
//!
//! The axis range is an aggregate *and* an input to the raster's key, so [`PlotState`] holds two
//! [`Memo`]s: the range's key is `(revision, policy)` and the raster's is
//! `(w, h, kind, sx, sy, series, y0, y1)` folded together with the revision. **A resize invalidates
//! the raster and not the range**, which is the whole reason they are two.
//!
//! [`Memo`]: vitui_runtime::data::Memo

use vitui_runtime::GlyphSet;
use vitui_runtime::data::{Memo, Revision};

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

/// **The three rungs, in order.**
///
/// It lives here because this file is the one the repertoire scans except, and the exception is
/// worth exactly one file: a test that sweeps the ladder, a report that prints it and a caller that
/// wants the richest rung all need to *name* one, and every one of them naming one directly would
/// turn a stated exception into a spreading one. `RUNGS[2]` is what a caller writes instead.
pub const RUNGS: [GlyphSet; 3] = [GlyphSet::Ascii, GlyphSet::Unicode, GlyphSet::Extended];

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
    /// **Cells the build painted**, which is the work `touched` cannot see.
    ///
    /// See [`Raster::painted`]. It is a field beside `touched` and not derived from it because the
    /// two answer different questions and the defect this counter exists for moves only one of them.
    painted: u64,
    shared: u32,
    /// **Per sub-column, the topmost sub-row a bar reaches**, reused across folds and across series.
    ///
    /// Scratch and not state: it is filled and consumed inside one `build`, and it is a field only
    /// so that a fold allocates nothing. `u32::MAX` is *no bar in this sub-column*.
    tops: Vec<u32>,
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
            painted: 0,
            shared: 0,
            tops: Vec::new(),
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

    /// The construction it was built in.
    pub fn kind(&self) -> Kind {
        self.kind
    }

    /// The sub-cell geometry it was built at.
    pub fn geometry(&self) -> Geom {
        self.geom
    }

    /// **Points the build looked at.** The edit's cost as a count — the only place data volume is
    /// allowed to appear.
    pub fn touched(&self) -> u64 {
        self.touched
    }

    /// **Cells the build painted.** The edit's cost as a count on the axis `touched` is blind to.
    ///
    /// `touched` is *points the build looked at* and it was **2 000 000 both ways** across the
    /// reduce — the prefix-per-point spelling visited each point exactly once and then did
    /// `O(subh)` work inside the visit. None of spec §20's nine counters expresses work that
    /// produces no output, and the picture never differed, so the whole of **952.61 ms against
    /// 2.72** was invisible to every gate in this workspace. This is the counter that sees it, and
    /// [`crate::volume`] is what reads it.
    pub fn painted(&self) -> u64 {
        self.painted
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
        self.painted += 1;
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
        self.painted = 0;
        self.shared = 0;
        if w == 0 || h == 0 {
            return;
        }
        let subw = u32::from(w) * u32::from(g.sx);
        let subh = u32::from(h) * u32::from(g.sy);
        if kind == Kind::Bars {
            self.tops.clear();
            self.tops.resize(subw as usize, u32::MAX);
        }
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
                        // **A bar is a prefix, and the union of two prefixes is the taller one** —
                        // so a sub-column needs the topmost sub-row any of its points reached, and
                        // nothing else. Recorded here and painted once below.
                        //
                        // It used to paint the whole prefix per point, which is the same picture
                        // and `O(subh)` a point: at a million points onto four hundred sub-columns
                        // the same cells were painted thousands of times over. **952.61 ms against
                        // the marks arm's 6.00 ms on the same data** — 476.3 ns a point against
                        // 3.0. The union being idempotent is what made it *correct*, and it is also
                        // what made the cost invisible to every gate: the picture never differed.
                        let slot = &mut self.tops[sx_pos as usize];
                        if sy_pos < *slot {
                            *slot = sy_pos;
                        }
                    }
                }
                i += step;
            }

            // **The paint, once a sub-column rather than once a point.** `put` is idempotent and
            // the series loop is outer in both spellings, so `bits`, `owner` and `shared` come out
            // identical — which is what `the_reduced_bars_fold_is_the_same_raster` asserts.
            if kind == Kind::Bars {
                for sx_pos in 0..subw {
                    let top = self.tops[sx_pos as usize];
                    if top == u32::MAX {
                        continue;
                    }
                    self.tops[sx_pos as usize] = u32::MAX;
                    let cx = (sx_pos / u32::from(g.sx)) as u16;
                    let c = (sx_pos % u32::from(g.sx)) as u8;
                    let mut sy = top;
                    while sy < subh {
                        let cy = (sy / u32::from(g.sy)) as u16;
                        let r = (sy % u32::from(g.sy)) as u8;
                        self.put(cx, cy, r * g.sx + c, sid);
                        sy += 1;
                    }
                }
            }
        }
    }
}

// ── the memo chain, and the key that is every input ──────────────────────────────────────────────

/// **What the raster's memo is keyed on**, and the two narrow spellings that look right.
///
/// Every arm below draws the same picture on the frame it is built on. What separates them is what
/// happens *next*: a resize, a zoom, a repertoire swap. Each narrower key recomputes **less often**
/// and is wrong, which is why the miss counter is reported and is never the detector — the surface
/// is.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum KeyMode {
    /// **Every input.** The rectangle, the construction, the sub-cell geometry, the series count and
    /// the domain, folded with the data's revision.
    #[default]
    Full,
    /// **The data revision and nothing else**, which is the shape a `Memo<T>` invites when its `get`
    /// takes one `Revision`. It survives a resize, a zoom and a repertoire swap, and it is wrong
    /// after every one of them.
    DataOnly,
    /// **The data and the rectangle**, and still not enough: it survives a repertoire swap and a
    /// change of domain. Kept so the failure is visibly *gradual* rather than binary.
    DataRect,
}

impl KeyMode {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            KeyMode::Full => "every input",
            KeyMode::DataOnly => "the data revision alone",
            KeyMode::DataRect => "the data and the rectangle",
        }
    }
}

/// FNV-1a over the key's bytes.
///
/// **A fold and not a hash table.** `Memo::get` takes one `Revision` and the runtime's own
/// documentation says what to do about it — *a caller composing a key out of two revisions folds
/// them into one and this is the door* — so the whole key becomes eight bytes and
/// [`Revision::from_raw`] carries them. `Revision::UNKNOWN` is 0 and means *memoise nothing*, so a
/// fold that lands on zero is nudged: the alternative is a plot that silently refolds a million
/// points every frame, once in 2^64.
fn fold(parts: &[u64]) -> Revision {
    let mut h = 0xcbf2_9ce4_8422_2325u64;
    for part in parts {
        for byte in part.to_le_bytes() {
            h ^= u64::from(byte);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    Revision::from_raw(if h == 0 { 1 } else { h })
}

/// **A plot's own state, and the memo chain lives in it** — `CONTEXT.md`'s rule that a memo is
/// *keyed by where it is stored*, which is why it consumes no `Id`.
///
/// # It is a chain and not one memo
///
/// The axis range is an aggregate *and* an input to the raster's key. Folding the two together
/// would make a resize refold the data; keeping them apart makes a resize invalidate the raster and
/// **not** the range, which is what [`PlotState::range_misses`] against [`PlotState::misses`]
/// reports and what `crate::series::tests` gates.
///
/// # The two buffers are fields because the draw path allocates nothing
///
/// A `String::with_capacity` on the draw path is one allocation a frame, for ever. Both were
/// measured doing exactly that and both are fields now.
#[derive(Debug)]
pub struct PlotState {
    range: Memo<Domain>,
    raster: Memo<Raster>,
    blank: Raster,
    /// The row buffer the frame spells into, allocated once.
    pub(crate) row: String,
    /// The label buffer the gutter formats into, allocated once.
    pub(crate) label: String,
}

impl PlotState {
    /// A plot that has drawn nothing.
    pub fn new() -> PlotState {
        PlotState {
            range: Memo::new(),
            raster: Memo::new(),
            blank: Raster::empty(),
            row: String::with_capacity(1024),
            label: String::with_capacity(64),
        }
    }

    /// **The raster the last frame drew from**, or an empty one before the first.
    pub fn raster(&self) -> &Raster {
        self.raster.peek().unwrap_or(&self.blank)
    }

    /// **How many times the raster has been folded.** A report and never the detector: a narrower
    /// key folds *fewer* times and is wrong, so this counter points the wrong way on exactly the
    /// class of defect it looks like it would catch.
    pub fn misses(&self) -> u32 {
        self.raster.recomputes
    }

    /// How many times the range has been folded. See [`PlotState::misses`].
    pub fn range_misses(&self) -> u32 {
        self.range.recomputes
    }

    /// **The raster and the row buffer, borrowed at once.**
    ///
    /// The draw loop needs the memo's value and the buffer it spells into in the same statement, and
    /// they are separate fields precisely so it can have both. Returning the pair is the one place
    /// that split is visible.
    pub fn raster_and_row(&mut self) -> (&Raster, &mut String) {
        let PlotState {
            raster, blank, row, ..
        } = self;
        (raster.peek().unwrap_or(blank), row)
    }

    /// **How much room the row buffer holds**, so that *the draw path allocates nothing* is a
    /// property a test can ask about rather than a comment.
    pub fn row_capacity(&self) -> usize {
        self.row.capacity()
    }

    /// [], for the gutter's label buffer.
    pub fn label_capacity(&self) -> usize {
        self.label.capacity()
    }

    /// The domain the last frame drew against.
    pub fn domain(&self) -> Domain {
        self.range
            .peek()
            .copied()
            .unwrap_or(Domain { y0: 0.0, y1: 1.0 })
    }

    /// **The range half of the chain.**
    ///
    /// Its key is the data revision and the policy. **The rectangle is deliberately not in it**,
    /// because a resize does not change what the data's extremes are — which is the whole reason
    /// this is a second memo rather than nine more bytes in the first one's key.
    pub fn range(&mut self, rev: Revision, how: Range, series: &[Vec<f32>]) -> Domain {
        let policy = match how {
            Range::Whole => 0u64,
            Range::Sampled(n) => 1 | (u64::from(n) << 8),
        };
        *self.range.get(fold(&[rev.raw(), policy]), || {
            domain_of(series, how).padded()
        })
    }

    /// **The raster half.** Its key is every input, including the domain the range half answered.
    ///
    /// # Panics
    ///
    /// Never. A miss builds the raster and a hit returns it; there is no third outcome.
    #[allow(clippy::too_many_arguments)]
    pub fn refresh(
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
        self.refresh_keyed(KeyMode::Full, rev, w, h, kind, g, dom, series, reach, cull);
    }

    /// [`PlotState::refresh`], with the key narrowed on purpose. The three negative cases.
    #[allow(clippy::too_many_arguments)]
    pub fn refresh_keyed(
        &mut self,
        mode: KeyMode,
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
        let key = match mode {
            KeyMode::DataOnly => fold(&[rev.raw()]),
            KeyMode::DataRect => fold(&[rev.raw(), u64::from(w), u64::from(h)]),
            KeyMode::Full => fold(&[
                rev.raw(),
                u64::from(w),
                u64::from(h),
                kind as u64,
                u64::from(g.sx),
                u64::from(g.sy),
                series.len() as u64,
                u64::from(y0),
                u64::from(y1),
                reach as u64,
            ]),
        };
        self.raster.get(key, || {
            let mut raster = Raster::empty();
            raster.build(w, h, kind, g, dom, series, reach, cull);
            raster
        });
    }
}

impl Default for PlotState {
    fn default() -> PlotState {
        PlotState::new()
    }
}

/// **The two spellings of the fold that draw the right picture and do too much work**, kept
/// runnable because [`crate::volume`]'s whole subject is a cost no output counter can see.
///
/// Each is watched failing O6 in `crate::volume::tests`, and the two fail *different halves of it*,
/// which is why there are two rather than one:
///
/// - [`defective::naive_bars`] is **linear with a constant of `subh`**, so its growth relation is
///   the shipped fold's — ten, at volumes a decade apart — and what disqualifies it is the
///   per-point ceiling. It is the defect that actually shipped.
/// - [`defective::domain_per_point`] is **quadratic**, so it is what the relation itself is watched
///   failing on. A per-input ceiling alone would be met by any constant chosen large enough; a
///   relation alone is blind to a constant. O6 is both, and these are the two arms that say so.
pub mod defective {
    use super::{Domain, Geom, Kind, Raster};

    /// **The spelling this module shipped until the reduce: the whole prefix, once a point.**
    ///
    /// Kept because the claim being made about the reduce is an *equality* between two
    /// implementations and an equality needs both sides — `reduce_tests` is that half — and because
    /// the claim O6 makes about it is a **cost**, which needs it to be runnable from a gate.
    ///
    /// It is the old inner loop verbatim: **952.61 ms against 2.72 ms** on two million values,
    /// 476.3 ns a point against 1.4, and *the same raster*, cell for cell and owner for owner.
    pub fn naive_bars(r: &mut Raster, w: u16, h: u16, g: Geom, dom: Domain, series: &[Vec<f32>]) {
        r.resize(w, h);
        r.kind = Kind::Bars;
        r.geom = g;
        r.dom = dom;
        r.touched = 0;
        r.painted = 0;
        r.shared = 0;
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
            for (i, &v) in s.iter().enumerate() {
                r.touched += 1;
                let sx_pos =
                    (((i as u64 * u64::from(subw)) / (n as u64).max(1)) as u32).min(subw - 1);
                let t = (dom.y1 - v) / span;
                let sy_pos = (t * subh as f32) as i32;
                if sy_pos < 0 || sy_pos >= subh as i32 {
                    continue;
                }
                let mut sy = sy_pos as u32;
                let cx = (sx_pos / u32::from(g.sx)) as u16;
                let c = (sx_pos % u32::from(g.sx)) as u8;
                while sy < subh {
                    let cy = (sy / u32::from(g.sy)) as u16;
                    let rr = (sy % u32::from(g.sy)) as u8;
                    r.put(cx, cy, rr * g.sx + c, sid);
                    sy += 1;
                }
            }
        }
    }

    /// **The domain recomputed from the whole series at every point.**
    ///
    /// The shipped chain asks [`super::domain_of`] once and memoises it on the data's revision
    /// ([`super::PlotState::range`]); this asks it inside the point loop, which is the shape a
    /// caller writes when the domain is a local rather than a memo. It draws **the identical
    /// raster** — the domain of a series does not depend on which point of it you are standing on —
    /// so every counter this crate has, `touched` and `painted` included, reads exactly what the
    /// shipped fold reads.
    ///
    /// What separates it is the reads it makes of the data, which is what `scanned` counts. `n`
    /// points times `n` values is `O(n^2)`: at volumes a decade apart the growth relation is **a
    /// hundred against ten**, and it is the only arm on this map that fails that half of O6.
    ///
    /// Returns the reads rather than storing them, because a raster has no field for a cost its own
    /// build did not pay and inventing one would put the defect's bookkeeping in the shipped type.
    pub fn domain_per_point(
        r: &mut Raster,
        w: u16,
        h: u16,
        kind: Kind,
        g: Geom,
        series: &[Vec<f32>],
    ) -> u64 {
        let mut scanned = 0u64;
        let mut dom = Domain { y0: 0.0, y1: 1.0 };
        for s in series {
            for _ in 0..s.len() {
                dom = super::domain_of(series, super::Range::Whole);
                scanned += series.iter().map(|t| t.len() as u64).sum::<u64>();
            }
        }
        r.build(
            w,
            h,
            kind,
            g,
            dom,
            series,
            super::Reach::Mapped,
            (0, usize::MAX),
        );
        scanned
    }
}

#[cfg(test)]
mod reduce_tests {
    use super::defective::naive_bars;
    use super::{Domain, Kind, Raster, Reach, geom};
    use vitui_runtime::theme::GlyphSet;

    /// **The reduce paints the same raster, cell for cell, owner for owner.**
    ///
    /// A bar is a prefix and the union of two prefixes is the taller one, so a sub-column needs only
    /// the topmost sub-row any of its points reached. Painting the whole prefix per point reaches
    /// the same union — which is why the old spelling was *correct* — at `O(subh)` a point:
    /// **952.61 ms against 2.72 ms** on two million values, 476.3 ns a point against 1.4.
    ///
    /// The cost was invisible to every gate this crate has, and that is the part worth keeping:
    /// nothing here reads a clock, the round trip compares a replayed screen against the frame that
    /// produced it, and both spellings produce the identical frame. **An equality between two
    /// implementations is what caught it, and only because someone ran the application.**
    ///
    /// Asserted on all four counters, over a mixed-sign series that lands points above and below
    /// the domain so the discard path is exercised on both arms, and at two geometries so the
    /// sub-cell arithmetic is not tested at `sx = sy = 1` alone.
    #[test]
    fn the_reduced_bars_fold_is_the_same_raster() {
        let n = 4_000usize;
        let series: Vec<Vec<f32>> = vec![
            (0..n)
                .map(|i| 50.0 + 40.0 * (i as f32 * 0.01).sin())
                .collect(),
            (0..n)
                .map(|i| 20.0 + 90.0 * (i as f32 * 0.017).cos())
                .collect(),
        ];
        let dom = Domain { y0: 0.0, y1: 100.0 };
        for set in [GlyphSet::Ascii, GlyphSet::Unicode, GlyphSet::Extended] {
            let g = geom(Kind::Bars, set);
            for (w, h) in [(60u16, 20u16), (37, 11)] {
                let mut shipped = Raster::empty();
                shipped.build(w, h, Kind::Bars, g, dom, &series, Reach::Mapped, (0, n));
                let mut naive = Raster::empty();
                naive_bars(&mut naive, w, h, g, dom, &series);
                assert_eq!(shipped.bits, naive.bits, "{set:?} {w}x{h}: the cells");
                assert_eq!(shipped.owner, naive.owner, "{set:?} {w}x{h}: the owners");
                assert_eq!(
                    shipped.shared, naive.shared,
                    "{set:?} {w}x{h}: the shared count"
                );
                assert_eq!(
                    shipped.touched, naive.touched,
                    "{set:?} {w}x{h}: the points visited"
                );
            }
        }
    }
}
