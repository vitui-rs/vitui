//! Layout: pure functions over integer rectangles, one denominator, and no gaps.
//!
//! Spec §11. **No solver state, no allocation, no floats in the result** — and no floats anywhere,
//! because the one place a float would be natural is the one place it cannot be trusted: a
//! proportional claim that rounds differently on two lanes is a hole in the screen.
//!
//! ```
//! use vitui_engine::Rect;
//! use vitui_runtime::layout::{Constraint::*, Row};
//!
//! let band = Rect::new(0, 0, 80, 24);
//! let [side, gutter, main] = Row::new().spacing(1).split(band, [Fixed(20), Fixed(1), Weight(1)]);
//! assert_eq!(side.w + gutter.w + main.w + 2, 80);
//! ```
//!
//! # The three things this module is, in the order they matter
//!
//! **One: it never leaves a gap.** Every lane starts where the last one ended, by construction
//! rather than by arithmetic — `place` walks a cursor and writes, so contiguity is not a property
//! that can round wrong. What arithmetic decides is only how long each lane is.
//!
//! **Two: the proportional family shares one denominator.** `Percent` and `Ratio` are resolved as
//! *one group over one total*, with largest-remainder distribution, and not lane by lane. Lane by
//! lane, **two 50% lanes on 101 columns round to 50 and 50** and the hundred-and-first column is
//! nobody's. Here they are 51 and 50.
//!
//! **Three: it clamps and discards, and never returns a `Result`.** Every degenerate input in this
//! module is reachable **by dragging a terminal edge** — a zero-width band, one column, more lanes
//! than columns, a `Fixed` larger than the band — so an error type would mean handling an error on
//! every resize, in every component, for a condition that has an obvious right answer. [`Fit`]
//! reports what was discarded instead, and the ergonomic [`Row::split`] drops it.
//!
//! # The naive scheme was replaced on a measurement, not an argument
//!
//! The obvious way to divide a band — hand out the fixed lanes, give each weighted lane
//! `free * weight / total` truncated, and let the last lane absorb the error — loses at least one
//! column in **10 957 of 36 503 comparable specs, which is 30.0%**, against **0** for the scheme
//! here. The smallest case that shows it is three lanes:
//! `[Weight(1), Weight(1), Fixed(3)]` over ten columns is 3/3/3 naive and 4/3/3 here.
//! `tests::the_naive_scheme_leaves_gaps_and_this_one_does_not` is that sweep.
//!
//! # Rounding is *up*, and the over-claim is bounded rather than absent
//!
//! Each proportional claim is put on a `2^40` scale with `div_ceil`. Rounded **down**, three
//! `Ratio(1, 3)` lanes claim 99 columns of 100 and the hundredth is nobody's — the same hole as the
//! per-lane rounding above, arriving through the denominator instead. Rounding up over-claims by at
//! most `lanes × avail / 2^40`, which at 64 lanes and a 65 535-cell band is **under 4×10⁻⁶ of a
//! column**: it cannot round to a whole cell, so it cannot move a boundary.

// Text measurement, which is what layout actually costs. A submodule rather than a module of its
// own: measuring a wrapped paragraph is a layout question, and the two are used together. Its
// documentation is in `layout/text.rs` — a `///` here as well as the `//!` there makes rustdoc
// resolve that file's intra-doc links in *this* scope, where none of its items exist.
pub mod text;

use vitui_engine::Rect;

/// Which way a band is divided. Private: [`Row`] and [`Col`] are the public form, and an axis
/// parameter on a public function would be a third way to say the same thing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Axis {
    X,
    Y,
}

/// The scale proportional claims are put on before they are summed.
///
/// `2^40` and not `2^32`: the product `claim_sum * avail` must not overflow a `u64`, and the claim
/// sum is at most `lanes * 2^40`. At 64 lanes and a 65 535-cell band that product is `2^40 × 64 ×
/// 65535 ≈ 2^62`, which fits with two bits to spare. It also has to be large enough that the
/// per-lane round-up cannot reach a whole cell — see the module comment.
const SCALE: u64 = 1 << 40;

/// A ceiling that means *no ceiling*. `u32::MAX` rather than an `Option`, because it is compared
/// against in the hot loop and `None` would be a branch in the middle of the fixpoint.
const UNBOUNDED: u32 = u32::MAX;

/// The most lanes [`Grid`] will divide an axis into.
///
/// A grid takes its arity at run time and this module allocates nothing, so the buffer is on the
/// stack and has a size. Sixty-four is the same bound the over-claim analysis uses, and a grid with
/// more than sixty-four columns on a terminal is not a layout.
const GRID_MAX: usize = 64;

/// What a lane asks for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Constraint {
    /// Exactly this many cells, taken before anything else.
    ///
    /// Over-subscribed `Fixed` lanes are satisfied **greedily in declaration order** rather than
    /// shrunk proportionally: a caller who put the important column first should still see it whole.
    /// `[Fixed(15), Fixed(15), Fixed(15)]` on twenty columns is `[15, 5, 0]`.
    Fixed(u16),
    /// A weighted lane that never falls below this many cells. **Only a floor** — it is a
    /// `Weight(1)` lane seeded at `v`, and it costs no fixpoint.
    Min(u16),
    /// A weighted lane that never rises above this many cells. **This is the one that costs a
    /// fixpoint**, because clamping one lane hands its surplus to the others, which may then need
    /// clamping too. Bounded at one round per lane; see [`solve`].
    Max(u16),
    /// `p` per cent of the whole band. `p` is clamped to 100.
    ///
    /// Of the *whole band*, not of what the rigid lanes left: `Percent(50)` beside `Fixed(20)` on
    /// eighty columns is forty, not thirty.
    Percent(u8),
    /// `a`/`b` of the whole band. `a` is clamped to `b`, and `b == 0` claims nothing rather than
    /// dividing by zero.
    Ratio(u16, u16),
    /// A share of what the rigid lanes left, proportional to this number. `Weight(0)` claims
    /// nothing, and a band of nothing but zero weights is all [`Fit::slack`].
    Weight(u16),
}

impl Constraint {
    /// `(numerator, denominator)` over the **whole band**, for the one group that shares a
    /// denominator.
    #[inline]
    const fn proportional(self) -> Option<(u64, u64)> {
        match self {
            // Written out rather than `min`, because `Ord::min` is not a const trait yet and
            // these helpers are worth keeping const.
            Constraint::Percent(p) => Some((if p > 100 { 100 } else { p } as u64, 100)),
            Constraint::Ratio(_, 0) => None,
            Constraint::Ratio(a, b) => Some((if a > b { b } else { a } as u64, b as u64)),
            _ => None,
        }
    }

    /// `(weight, floor, ceiling)`.
    ///
    /// `Min` and `Max` are **weight-one lanes** with a floor and a ceiling respectively, which is
    /// what keeps them out of the proportional group and out of `Fixed`'s greedy pass.
    #[inline]
    const fn weighted(self) -> Option<(u64, u32, u32)> {
        match self {
            Constraint::Weight(w) => Some((w as u64, 0, UNBOUNDED)),
            Constraint::Min(v) => Some((1, v as u32, UNBOUNDED)),
            Constraint::Max(v) => Some((1, 0, v as u32)),
            _ => None,
        }
    }

    /// The cells this lane takes before anything else.
    #[inline]
    const fn fixed(self) -> Option<u32> {
        match self {
            Constraint::Fixed(v) => Some(v as u32),
            _ => None,
        }
    }
}

/// The furniture a band actually paid for, which is not always what was asked for.
///
/// **This is where a band too small to pay for its own furniture becomes visible instead of
/// silent.** A caller checking that its lanes tile the band must compare against these numbers and
/// not against the ones it requested: a zero-width band asked for `.spacing(2).margin(1)` reports
/// `Gaps::default()`, because there is nothing to put a margin in.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Gaps {
    /// Cells between two adjacent lanes.
    pub spacing: u16,
    /// Cells inside the band's edge, on all four sides.
    pub margin: u16,
}

/// What a split discarded, and what it could not fill.
///
/// Returned by [`Row::measure_into`]; [`Row::split`] and [`Row::split_into`] drop it, which is the
/// ergonomic default and is why this type is small enough to be `Copy`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Fit {
    /// Cells inside the band no lane claimed.
    ///
    /// **Always one trailing remainder at the end of the band, never a hole between lanes** — only
    /// a spec with no elastic lane can produce it. `[Max(10), Max(10)]` on a hundred columns is
    /// `slack == 80`; `[Weight(0), Weight(0), Weight(0)]` on forty is `slack == 40`.
    pub slack: u32,
    /// Cells the spec asked for beyond the band, and did not get.
    pub overflow: u32,
    /// How many rounds [`Constraint::Max`]'s fixpoint took. Zero when no ceiling engaged, and never
    /// more than the number of lanes.
    pub rounds: u32,
    /// The furniture actually applied.
    pub gaps: Gaps,
}

/// Divide `avail` cells among `spec`, writing each lane's extent into `out`.
///
/// **One-dimensional, and `Rect` is the carrier rather than the result.** `out[i].w` is the lane's
/// extent along whichever axis the caller is splitting; `x`, `y` and `h` are zero on return.
/// [`Row`] and [`Col`] are what turn extents into rectangles, and they are what a caller should
/// normally use — this is public because a caller doing its own placement should not have to
/// reimplement the arithmetic.
///
/// Returns `(lanes written, fit)`. **A spec longer than the buffer is discarded from the end**, and
/// the returned count is what was written: `spec.len().min(out.len())`.
///
/// # Where the fixpoint's frozen bit lives
///
/// In `out[i].x`, and that is why there is no scratch parameter and no allocation. The output buffer
/// is already exactly one slot per lane, it is already `&mut`, and `x` is meaningless until `place`
/// runs — so the `Max` fixpoint marks its frozen lanes there and `place_none` clears them before
/// returning. A caller of `solve` alone therefore sees `x == 0`; a caller who saw `x == 1` would be
/// looking at a bug.
///
/// # Order of resolution
///
/// 1. **`Fixed`**, greedily in declaration order.
/// 2. **`Percent` and `Ratio`** as one group over one denominator, claiming against the *whole*
///    band and then capped by what is left.
/// 3. **`Weight`, `Min` and `Max`** over the remainder, with the fixpoint.
///
/// # Why `Max` terminates in one round per lane
///
/// Every round that does not break freezes at least one lane, and a frozen lane never moves again —
/// it has become a `Fixed` lane. So `fit.rounds <= n`, and that is asserted rather than reasoned
/// about in `tests::the_cap_fixpoint_is_bounded_at_one_round_per_lane` over fifty thousand random
/// specs.
///
/// **The reset at the top of each round is load-bearing and was once missing.** Without it,
/// `[Max(5), Weight(1), Weight(1)]` over thirty columns gave `[5, 3, 2]` — ten columns of a
/// thirty-column band, silently — and every invariant except exact coverage still held. Each round
/// must redistribute the *whole* remainder among the unfrozen lanes, not top up what they already
/// had.
pub fn solve(avail: u32, spec: &[Constraint], out: &mut [Rect]) -> (usize, Fit) {
    let n = spec.len().min(out.len());
    let mut fit = Fit::default();
    if n == 0 {
        fit.slack = avail;
        return (0, fit);
    }
    for slot in out[..n].iter_mut() {
        *slot = Rect::default();
    }

    let mut remaining = avail;

    // Stage 0 — `Fixed`, greedy in declaration order.
    for i in 0..n {
        if let Some(want) = spec[i].fixed() {
            let got = want.min(remaining);
            out[i].w = clamp_u16(got);
            fit.overflow += want - got;
            remaining -= got;
        }
    }

    // Stage 1 — the proportional family, as one group over one denominator.
    let claim = |c: Constraint| c.proportional().map(|(a, b)| (a * SCALE).div_ceil(b));
    let claim_sum: u64 = spec[..n].iter().copied().filter_map(claim).sum();
    if claim_sum > 0 {
        // One division for the whole group, against `avail` rather than against what stage 0 left:
        // `Percent(50)` means half the band.
        let wanted = clamp_u32(claim_sum * u64::from(avail) / SCALE);
        let pool = wanted.min(remaining);
        fit.overflow += wanted - pool;

        let floor_of = |w: u64| clamp_u32(u64::from(pool) * w / claim_sum);
        let remainder_of = |w: u64| (u64::from(pool) * w) % claim_sum;
        let floor_sum: u32 = spec[..n]
            .iter()
            .copied()
            .filter_map(claim)
            .map(floor_of)
            .sum();
        let deficit = pool - floor_sum;

        for i in 0..n {
            if let Some(w) = claim(spec[i]) {
                let bump = takes_a_bump(i, remainder_of(w), deficit, n, |j| {
                    claim(spec[j]).map(remainder_of)
                });
                let got = floor_of(w) + u32::from(bump);
                out[i].w = clamp_u16(got);
                remaining -= got;
            }
        }
    }

    // Stage 2 — `Weight`, `Min` and `Max`, with the fixpoint.
    let floor_total: u64 = spec[..n]
        .iter()
        .copied()
        .filter_map(Constraint::weighted)
        .map(|(_, lo, _)| u64::from(lo))
        .sum();
    if floor_total > u64::from(remaining) {
        // The floors alone do not fit. Handed out greedily in declaration order, exactly like
        // `Fixed` and for the same reason: a floor that cannot be honoured is a `Fixed` lane that
        // cannot be honoured.
        for i in 0..n {
            if let Some((_, lo, _)) = spec[i].weighted() {
                let got = lo.min(remaining);
                out[i].w = clamp_u16(got);
                fit.overflow += lo - got;
                remaining -= got;
            }
        }
        fit.slack = 0;
        place_none(out, n);
        return (n, fit);
    }

    for i in 0..n {
        if let Some((weight, lo, hi)) = spec[i].weighted() {
            out[i].w = clamp_u16(lo);
            // A zero-weight lane, or one whose ceiling is already at its floor, starts frozen: it
            // can never take a share, so leaving it in the pool would only cost rounds.
            out[i].x = i32::from(weight == 0 || hi <= lo);
        }
    }

    let pool = remaining;
    loop {
        for i in 0..n {
            if let Some((_, lo, _)) = spec[i].weighted() {
                if out[i].x == 0 {
                    out[i].w = clamp_u16(lo);
                }
            }
        }
        let used: u32 = (0..n)
            .filter(|&i| spec[i].weighted().is_some())
            .map(|i| u32::from(out[i].w))
            .sum();
        let left = pool.saturating_sub(used);
        let total_weight: u64 = (0..n)
            .filter(|&i| out[i].x == 0)
            .filter_map(|i| spec[i].weighted().map(|(w, _, _)| w))
            .sum();
        if left == 0 || total_weight == 0 {
            fit.slack = left;
            break;
        }

        let floor_of = |w: u64| clamp_u32(u64::from(left) * w / total_weight);
        let remainder_of = |w: u64| (u64::from(left) * w) % total_weight;
        let unfrozen = |j: usize| {
            (out[j].x == 0)
                .then(|| spec[j].weighted().map(|(w, _, _)| w))
                .flatten()
        };
        let floor_sum: u32 = (0..n).filter_map(unfrozen).map(floor_of).sum();
        let deficit = left - floor_sum;

        for i in 0..n {
            // The extent is computed in a block that borrows `out` immutably and then written after
            // that borrow has ended. Hoisting the closure out of the loop is what the first draft
            // did, and it held a shared borrow across the write (`E0506`) — the rank has to see
            // every lane's frozen bit, which lives in the same buffer being written.
            let extent = {
                let unfrozen = |j: usize| {
                    (out[j].x == 0)
                        .then(|| spec[j].weighted().map(|(w, _, _)| w))
                        .flatten()
                };
                unfrozen(i).map(|w| {
                    let (_, lo, _) = spec[i].weighted().expect("unfrozen implies weighted");
                    let bump = takes_a_bump(i, remainder_of(w), deficit, n, |j| {
                        unfrozen(j).map(remainder_of)
                    });
                    clamp_u16(lo + floor_of(w) + u32::from(bump))
                })
            };
            if let Some(extent) = extent {
                out[i].w = extent;
            }
        }

        let mut froze_something = false;
        for i in 0..n {
            if out[i].x != 0 {
                continue;
            }
            if let Some((_, _, hi)) = spec[i].weighted() {
                if u32::from(out[i].w) > hi {
                    out[i].w = clamp_u16(hi);
                    out[i].x = 1;
                    froze_something = true;
                }
            }
        }
        fit.rounds += 1;
        if !froze_something {
            fit.slack = 0;
            break;
        }
    }

    place_none(out, n);
    (n, fit)
}

/// Whether lane `i` takes one of the `deficit` spare cells, under largest-remainder distribution.
///
/// **An O(n²) rank rather than a sort, and that is what buys the zero allocation.** Sorting the
/// remainders needs somewhere to put them; ranking each lane against the others needs nothing. In
/// practice it is linear, because `deficit` is almost always 0 or 1 and the early return above is
/// then the whole function: measured 30.60 ns at two lanes and 498.75 ns at sixty-four, which is
/// 16× for 32× the lanes.
///
/// **Ties go to the lower index**, so the spare column lands on the left: three equal lanes over a
/// hundred columns are 34/33/33 and not 33/33/34.
fn takes_a_bump(
    i: usize,
    rem_i: u64,
    deficit: u32,
    n: usize,
    rem: impl Fn(usize) -> Option<u64>,
) -> bool {
    if deficit == 0 {
        return false;
    }
    let mut ahead = 0u32;
    for j in 0..n {
        if j == i {
            continue;
        }
        let Some(rem_j) = rem(j) else { continue };
        if rem_j > rem_i || (rem_j == rem_i && j < i) {
            ahead += 1;
        }
    }
    ahead < deficit
}

/// Clear the frozen bits the fixpoint left in `x`.
fn place_none(out: &mut [Rect], n: usize) {
    for slot in out[..n].iter_mut() {
        slot.x = 0;
    }
}

/// Turn extents into rectangles by walking a cursor.
///
/// **This is where gap-freeness comes from, and it is structural rather than arithmetic.** The
/// cursor advances by `extent + spacing` for **every** lane including a zero-extent one, so a lane
/// that collapsed to nothing does not move its neighbours and no rounding decision can open a hole.
fn place(band: Rect, axis: Axis, gaps: Gaps, out: &mut [Rect], n: usize) {
    let margin = i32::from(gaps.margin);
    let spacing = i32::from(gaps.spacing);
    let cross = match axis {
        Axis::X => band.h.saturating_sub(gaps.margin.saturating_mul(2)),
        Axis::Y => band.w.saturating_sub(gaps.margin.saturating_mul(2)),
    };
    let mut cursor = match axis {
        Axis::X => band.x + margin,
        Axis::Y => band.y + margin,
    };
    for slot in out[..n].iter_mut() {
        let extent = slot.w;
        *slot = match axis {
            Axis::X => Rect::new(cursor, band.y + margin, extent, cross),
            Axis::Y => Rect::new(band.x + margin, cursor, cross, extent),
        };
        cursor += i32::from(extent) + spacing;
    }
}

/// What furniture a band can actually pay for, and how many cells are left for lanes.
///
/// **Decided before any constraint resolves, and that is a decision rather than an accident.** If
/// the furniture were computed from the lanes' outcomes, a lane collapsing to zero would move its
/// neighbours — the spacing beside it would vanish and the whole band would reflow for a reason the
/// caller did not ask for. Margins are clamped first, because a margin is about the band and spacing
/// is about what is left of it.
fn effective(band: Rect, axis: Axis, gaps: Gaps, n: usize) -> (Gaps, u32) {
    let len = u32::from(match axis {
        Axis::X => band.w,
        Axis::Y => band.h,
    });
    let margin = u32::from(gaps.margin).min(len / 2);
    let after_margins = len - margin * 2;
    let slots = u32::try_from(n.saturating_sub(1)).unwrap_or(u32::MAX);
    // `checked_div` rather than an `if`: one lane has no slots to space, and clippy is right that
    // the guard and the division are the same question asked twice.
    let spacing = after_margins
        .checked_div(slots)
        .map_or(0, |per| u32::from(gaps.spacing).min(per));
    let avail = after_margins - spacing * slots;
    (
        Gaps {
            spacing: clamp_u16(spacing),
            margin: clamp_u16(margin),
        },
        avail,
    )
}

#[inline]
fn clamp_u16(v: u32) -> u16 {
    u16::try_from(v).unwrap_or(u16::MAX)
}

#[inline]
fn clamp_u32(v: u64) -> u32 {
    u32::try_from(v).unwrap_or(u32::MAX)
}

/// Generate [`Row`] and [`Col`], which differ only in an axis.
///
/// A macro rather than one type with an axis field: the axis is known at the call site, `Row` and
/// `Col` read better than `Band::new(Axis::X)`, and an axis that is a *type* cannot be passed the
/// wrong way round by a caller who meant the other one.
macro_rules! band {
    ($name:ident, $axis:expr, $what:literal, $lanes:literal) => {
        #[doc = concat!("A ", $what, " band divided into ", $lanes, ".")]
        ///
        /// Cheap and `Copy`: it holds nothing but the furniture, and the split is a pure function of
        /// it. Nothing is retained between frames, because there is nothing to retain.
        #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
        pub struct $name(Gaps);

        impl $name {
            /// No spacing, no margin.
            pub const fn new() -> Self {
                $name(Gaps {
                    spacing: 0,
                    margin: 0,
                })
            }

            /// Cells between two adjacent lanes.
            pub const fn spacing(mut self, n: u16) -> Self {
                self.0.spacing = n;
                self
            }

            /// Cells inside the band's edge, on all four sides.
            pub const fn margin(mut self, n: u16) -> Self {
                self.0.margin = n;
                self
            }

            /// Split into exactly `N` lanes.
            ///
            /// **No turbofish**: the array literal carries the arity, so
            /// `split(band, [Fixed(20), Weight(1)])` needs no `::<2>` and cannot disagree with
            /// itself about how many lanes there are.
            ///
            /// [`Fit`] is dropped. Use [`Self::measure_into`] when it matters.
            pub fn split<const N: usize>(self, band: Rect, spec: [Constraint; N]) -> [Rect; N] {
                let mut out = [Rect::default(); N];
                let _ = self.measure_into(band, &spec, &mut out);
                out
            }

            /// Split into as many lanes as fit in `out`, returning how many were written.
            pub fn split_into(self, band: Rect, spec: &[Constraint], out: &mut [Rect]) -> usize {
                self.measure_into(band, spec, out).0
            }

            /// Split, and say what was discarded.
            ///
            /// The only public way to see a [`Fit`].
            pub fn measure_into(
                self,
                band: Rect,
                spec: &[Constraint],
                out: &mut [Rect],
            ) -> (usize, Fit) {
                let n = spec.len().min(out.len());
                let (gaps, avail) = effective(band, $axis, self.0, n);
                let (n, mut fit) = solve(avail, spec, out);
                fit.gaps = gaps;
                place(band, $axis, gaps, out, n);
                (n, fit)
            }
        }
    };
}

band!(Row, Axis::X, "horizontal", "columns");
band!(Col, Axis::Y, "vertical", "rows");

/// A grid, which is **a convenience over [`Row`] and [`Col`] and not a fourth primitive**.
///
/// Two splits compose into it and it adds no arithmetic of its own: a [`Col`] down the band and a
/// [`Row`] across the chosen row, both with `Weight(1)` lanes. It exists because writing those two
/// splits out at every call site is noise, and it is documented as a convenience so that nobody
/// looks for grid-specific behaviour that is not there.
///
/// ```
/// use vitui_engine::Rect;
/// use vitui_runtime::layout::Grid;
///
/// let band = Rect::new(0, 0, 30, 10);
/// let grid = Grid::new(3, 2);
/// assert_eq!(grid.cell(band, 0, 0, 1, 1), Rect::new(0, 0, 10, 5));
/// // A span is the union of adjacent cells, and it takes in the spacing between them.
/// assert_eq!(grid.cell(band, 0, 0, 2, 1), Rect::new(0, 0, 20, 5));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Grid {
    cols: u16,
    rows: u16,
    gaps: Gaps,
}

impl Grid {
    /// A grid of equal cells. **Both counts are clamped to sixty-four**: a grid takes its arity
    /// at run time and this module allocates nothing, so its buffer is on the stack and has a size.
    /// A grid with more than sixty-four columns on a terminal is not a layout.
    pub const fn new(cols: u16, rows: u16) -> Grid {
        Grid {
            cols,
            rows,
            gaps: Gaps {
                spacing: 0,
                margin: 0,
            },
        }
    }

    /// Cells between adjacent cells, in both directions.
    pub const fn spacing(mut self, n: u16) -> Grid {
        self.gaps.spacing = n;
        self
    }

    /// Cells inside the grid's outer edge.
    pub const fn margin(mut self, n: u16) -> Grid {
        self.gaps.margin = n;
        self
    }

    /// The rectangle covering `span_x` × `span_y` cells starting at column `x`, row `y`.
    ///
    /// A span is the **union of adjacent cells including the spacing between them**, which is what
    /// makes a spanning cell look like one cell rather than like two with a gap down the middle.
    /// Out-of-range coordinates give an empty rectangle: this is the clamp-and-discard rule, and a
    /// grid coordinate is as reachable by a resize as anything else here.
    pub fn cell(self, band: Rect, x: u16, y: u16, span_x: u16, span_y: u16) -> Rect {
        let cols = usize::from(self.cols).min(GRID_MAX);
        let rows = usize::from(self.rows).min(GRID_MAX);
        if cols == 0 || rows == 0 || usize::from(x) >= cols || usize::from(y) >= rows {
            return Rect::default();
        }
        let mut row_buf = [Rect::default(); GRID_MAX];
        let mut col_buf = [Rect::default(); GRID_MAX];
        let spec = [Constraint::Weight(1); GRID_MAX];

        let n_rows = Col::new()
            .spacing(self.gaps.spacing)
            .margin(self.gaps.margin)
            .split_into(band, &spec[..rows], &mut row_buf);
        let first_row = row_buf[usize::from(y)];
        let last_row = row_buf[(usize::from(y) + usize::from(span_y).max(1) - 1).min(n_rows - 1)];
        // The union, which is what takes in the spacing.
        let strip = Rect::new(
            band.x,
            first_row.y,
            band.w,
            clamp_u16(u32::try_from(last_row.bottom() - first_row.y).unwrap_or(0)),
        );

        // The inner split gets no margin: the outer one already applied it, and applying it twice
        // would inset every cell instead of the grid.
        let n_cols = Row::new()
            .spacing(self.gaps.spacing)
            .margin(self.gaps.margin)
            .split_into(band, &spec[..cols], &mut col_buf);
        let first_col = col_buf[usize::from(x)];
        let last_col = col_buf[(usize::from(x) + usize::from(span_x).max(1) - 1).min(n_cols - 1)];
        Rect::new(
            first_col.x,
            strip.y,
            clamp_u16(u32::try_from(last_col.right() - first_col.x).unwrap_or(0)),
            strip.h,
        )
    }
}

/// The rect algebra. Pure, total, and saturating everywhere.
///
/// Every function here clamps rather than panicking, for the module's standing reason: a rectangle
/// in a terminal is whatever the last resize made it, and an inset larger than the rectangle is a
/// normal Tuesday.
pub mod rect {
    use super::{Rect, clamp_u16};

    /// Shrink by `n` on all four sides.
    ///
    /// The origin moves by `n` even when the size collapses, so `inset(Rect::new(0, 0, 3, 3), 9)` is
    /// `Rect::new(9, 9, 0, 0)`: an empty rectangle *where the inset asked for one*, rather than an
    /// empty rectangle somewhere else.
    pub fn inset(a: Rect, n: u16) -> Rect {
        shrink(a, n, n, n, n)
    }

    /// Shrink by a different amount on each side, in the order left, top, right, bottom.
    pub fn shrink(a: Rect, l: u16, t: u16, r: u16, b: u16) -> Rect {
        Rect::new(
            a.x + i32::from(l),
            a.y + i32::from(t),
            a.w.saturating_sub(l.saturating_add(r)),
            a.h.saturating_sub(t.saturating_add(b)),
        )
    }

    /// Grow by `n` on all four sides.
    pub fn expand(a: Rect, n: u16) -> Rect {
        Rect::new(
            a.x - i32::from(n),
            a.y - i32::from(n),
            a.w.saturating_add(n.saturating_mul(2)),
            a.h.saturating_add(n.saturating_mul(2)),
        )
    }

    /// The overlap of two rectangles, empty when they do not overlap.
    ///
    /// Delegates to the engine's own [`Rect::intersect`] rather than reimplementing it: two
    /// intersections that disagree by one cell is exactly the class of defect this module exists to
    /// avoid.
    pub fn intersect(a: Rect, b: Rect) -> Rect {
        a.intersect(b)
    }

    /// Move `a` the shortest distance that puts it inside `b`, **keeping its size**.
    ///
    /// Not the same as [`intersect`], and the difference is what a popup wants: a dialog dragged off
    /// the screen should come back whole, not be cropped. `clamp_to(Rect::new(20, 20, 4, 4),
    /// Rect::new(0, 0, 10, 10))` is `Rect::new(6, 6, 4, 4)`.
    pub fn clamp_to(a: Rect, b: Rect) -> Rect {
        let x = a.x.min(b.right() - i32::from(a.w)).max(b.x);
        let y = a.y.min(b.bottom() - i32::from(a.h)).max(b.y);
        Rect::new(x, y, a.w.min(b.w), a.h.min(b.h))
    }

    /// Cut `n` rows off the top: `(top, rest)`.
    ///
    /// `n` is clamped, so cutting nine rows off a three-row rectangle gives the whole thing and an
    /// empty remainder — and the remainder is positioned **after** the whole thing, so a caller that
    /// keeps splitting the rest keeps getting empty rectangles in the right place.
    pub fn split_at_v(a: Rect, n: u16) -> (Rect, Rect) {
        let cut = n.min(a.h);
        (
            Rect::new(a.x, a.y, a.w, cut),
            Rect::new(a.x, a.y + i32::from(cut), a.w, a.h - cut),
        )
    }

    /// Cut `n` columns off the left: `(left, rest)`.
    pub fn split_at_h(a: Rect, n: u16) -> (Rect, Rect) {
        let cut = n.min(a.w);
        (
            Rect::new(a.x, a.y, cut, a.h),
            Rect::new(a.x + i32::from(cut), a.y, a.w - cut, a.h),
        )
    }

    /// A rectangle of `w` by `h` at `a`'s origin, clamped to `a`. Used by `Align`.
    pub(super) fn sized(a: Rect, w: u16, h: u16) -> (u16, u16) {
        (w.min(a.w), h.min(a.h))
    }

    /// The centre offset for a `len`-long thing in a `span`-long space.
    pub(super) fn middle(span: u16, len: u16) -> i32 {
        i32::from(clamp_u16(u32::from(span.saturating_sub(len)) / 2))
    }
}

/// Where to put a fixed-size thing inside a band.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align {
    /// Top left corner.
    TopLeft,
    /// Centred horizontally, at the top.
    Top,
    /// Top right corner.
    TopRight,
    /// Centred vertically, at the left.
    Left,
    /// Centred both ways.
    Center,
    /// Centred vertically, at the right.
    Right,
    /// Bottom left corner.
    BottomLeft,
    /// Centred horizontally, at the bottom.
    Bottom,
    /// Bottom right corner.
    BottomRight,
}

impl Align {
    /// A `w` by `h` rectangle placed inside `a`.
    ///
    /// The size is clamped to `a` first, so a dialog larger than the screen is the screen rather
    /// than something hanging off the edge of it.
    pub fn place(self, a: Rect, w: u16, h: u16) -> Rect {
        let (w, h) = rect::sized(a, w, h);
        let (dx, dy) = match self {
            Align::TopLeft => (0, 0),
            Align::Top => (rect::middle(a.w, w), 0),
            Align::TopRight => (i32::from(a.w - w), 0),
            Align::Left => (0, rect::middle(a.h, h)),
            Align::Center => (rect::middle(a.w, w), rect::middle(a.h, h)),
            Align::Right => (i32::from(a.w - w), rect::middle(a.h, h)),
            Align::BottomLeft => (0, i32::from(a.h - h)),
            Align::Bottom => (rect::middle(a.w, w), i32::from(a.h - h)),
            Align::BottomRight => (i32::from(a.w - w), i32::from(a.h - h)),
        };
        Rect::new(a.x + dx, a.y + dy, w, h)
    }
}

/// Centring, which is what an overlay wants nine times out of ten.
///
/// A separate name rather than a tenth `Align` variant, because `Stack::center(band, w, h)` is what
/// the call site means and `Align::Center.place(band, w, h)` is how it is spelled. They are the same
/// function and the equality is asserted.
pub struct Stack;

impl Stack {
    /// `w` by `h`, centred in `a`.
    pub fn center(a: Rect, w: u16, h: u16) -> Rect {
        Align::Center.place(a, w, h)
    }
}

#[cfg(test)]
mod tests {
    use super::Constraint::{Fixed, Max, Min, Percent, Ratio, Weight};
    use super::*;

    /// SplitMix64, written out because this crate takes no dependencies and a sweep needs a
    /// reproducible stream. Seeded per corpus so that a failure is replayable from the test name.
    struct Rng(u64);

    impl Rng {
        const fn new(seed: u64) -> Rng {
            Rng(seed)
        }

        fn next_u64(&mut self) -> u64 {
            self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
            let mut z = self.0;
            z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
            z ^ (z >> 31)
        }

        fn below(&mut self, n: u64) -> u64 {
            if n == 0 { 0 } else { self.next_u64() % n }
        }

        fn range(&mut self, lo: u64, hi: u64) -> u64 {
            lo + self.below(hi - lo + 1)
        }
    }

    /// The widest spec any corpus generates, and the size of every scratch buffer here.
    const C: usize = 12;

    /// **What "gap-free" means, in three parts**, and the return value is the slack.
    ///
    /// 1. **Contiguity**: every lane starts exactly where the previous one ended, plus the spacing.
    /// 2. **Containment**: every lane is inside the band's margins.
    /// 3. **One trailing remainder**: whatever is left is at the end, which is what makes the
    ///    difference between slack and a hole checkable at all.
    ///
    /// Compared against `fit.gaps` and never against the requested furniture — a band too small to
    /// pay for its own spacing applies none, and a checker that used the request would report every
    /// small band as broken.
    fn assert_tiles(band: Rect, lanes: &[Rect], gaps: Gaps, axis: Axis, what: &str) -> i32 {
        let (start, end, spacing) = match axis {
            Axis::X => (
                band.x + i32::from(gaps.margin),
                band.right() - i32::from(gaps.margin),
                i32::from(gaps.spacing),
            ),
            Axis::Y => (
                band.y + i32::from(gaps.margin),
                band.bottom() - i32::from(gaps.margin),
                i32::from(gaps.spacing),
            ),
        };
        let mut cursor = start;
        for (i, lane) in lanes.iter().enumerate() {
            let (near, far) = match axis {
                Axis::X => (lane.x, lane.right()),
                Axis::Y => (lane.y, lane.bottom()),
            };
            assert_eq!(
                near, cursor,
                "{what}: lane {i} does not start where {} ended",
                i
            );
            assert!(
                far <= end.max(start),
                "{what}: lane {i} runs to {far}, past {end}"
            );
            cursor = far + spacing;
        }
        let last = lanes.last().map_or(start, |l| match axis {
            Axis::X => l.right(),
            Axis::Y => l.bottom(),
        });
        (end - last).max(0)
    }

    /// **The naive scheme, preserved so that its replacement is a measurement rather than a
    /// preference.**
    ///
    /// Hand out the fixed lanes, give each weighted lane `free * weight / total` truncated, and let
    /// the last lane absorb the error. This is the obvious implementation and it is what the
    /// architecture proposed; it loses a column in thirty per cent of realistic specs.
    fn naive_columns(avail: u32, spec: &[Constraint], out: &mut [u32]) {
        let total_fixed: u32 = spec.iter().copied().filter_map(Constraint::fixed).sum();
        let free = avail.saturating_sub(total_fixed);
        let total_weight: u32 = spec
            .iter()
            .filter_map(|c| match c {
                Weight(w) => Some(u32::from(*w)),
                _ => None,
            })
            .sum();
        for (i, c) in spec.iter().enumerate() {
            out[i] = match c {
                Fixed(v) => u32::from(*v),
                Weight(w) if total_weight > 0 => free * u32::from(*w) / total_weight,
                _ => 0,
            };
        }
        // **"The last lane absorbs the error", and that sentence is the whole defect.** It absorbs it
        // when the last lane is the one with a weight — and does nothing at all when the last lane is
        // `Fixed`, which is where a status bar, a scrollbar or a right-hand gutter goes. There is
        // nowhere else the error can be put: adding it to a `Fixed` lane would break the one
        // constraint that is supposed to be exact.
        //
        // The first version of this helper absorbed at the last *weighted* lane instead, which closed
        // every gap and reported the naive scheme as flawless over all 36 503 specs. A model of the
        // thing being compared against has to be able to lose.
        if let Some(Weight(w)) = spec.last() {
            if *w > 0 && total_weight > 0 {
                let used: u32 = out[..spec.len()].iter().sum();
                out[spec.len() - 1] += avail.saturating_sub(used);
            }
        }
    }

    // ---------------------------------------------------------------------------------------------
    // The equalities: one test per number the spec commits to.
    // ---------------------------------------------------------------------------------------------

    /// Three thirds of a hundred columns are a hundred, not ninety-nine.
    ///
    /// **This is the case that decides the rounding direction.** With `div_ceil` off the `2^40`
    /// scale, three `Ratio(1, 3)` lanes claim the whole band; rounded down they claim ninety-nine and
    /// the hundredth column belongs to nobody.
    #[test]
    fn three_thirds_of_a_hundred_columns_sum_to_a_hundred() {
        let out = Row::new().split(
            Rect::new(0, 0, 100, 1),
            [Ratio(1, 3), Ratio(1, 3), Ratio(1, 3)],
        );
        assert_eq!(out.map(|r| r.w), [34, 33, 33]);
        assert_eq!(out.iter().map(|r| u32::from(r.w)).sum::<u32>(), 100);
    }

    /// The same for weights, and the spare column goes to the left.
    #[test]
    fn three_equal_weights_over_a_hundred_columns_are_34_33_33() {
        let out = Row::new().split(Rect::new(0, 0, 100, 1), [Weight(1), Weight(1), Weight(1)]);
        assert_eq!(out.map(|r| r.w), [34, 33, 33]);
    }

    /// **Two 50% lanes on 101 columns are 51 and 50**, which is the exact gap a per-lane denominator
    /// leaves: lane by lane, each rounds to 50 and the hundred-and-first column is nobody's.
    #[test]
    fn two_fifty_per_cent_lanes_on_101_columns_are_51_and_50() {
        let out = Row::new().split(Rect::new(0, 0, 101, 1), [Percent(50), Percent(50)]);
        assert_eq!(out.map(|r| r.w), [51, 50]);
        assert_eq!(out.iter().map(|r| u32::from(r.w)).sum::<u32>(), 101);
    }

    /// A `Max` lane hands its surplus back, and the fixpoint is what makes that happen.
    ///
    /// **The number to watch is 13, not 5.** The first version of the fixpoint did not reset the
    /// unfrozen lanes each round and produced `[5, 3, 2]` — ten columns of a thirty-column band, with
    /// every invariant except exact coverage still holding.
    #[test]
    fn a_capped_lane_hands_its_surplus_to_the_others() {
        let out = Row::new().split(Rect::new(0, 0, 30, 1), [Max(5), Weight(1), Weight(1)]);
        assert_eq!(out.map(|r| r.w), [5, 13, 12]);
    }

    /// `Min` is a floor and nothing else: it takes its floor and then its share.
    #[test]
    fn a_floor_takes_its_floor_and_then_its_share() {
        let out = Row::new().split(Rect::new(0, 0, 30, 1), [Min(20), Weight(1)]);
        assert_eq!(out.map(|r| r.w), [25, 5]);
    }

    /// `Percent` is of the whole band, not of what the rigid lanes left.
    #[test]
    fn a_percentage_is_of_the_whole_band() {
        let out = Row::new().split(Rect::new(0, 0, 80, 1), [Fixed(20), Percent(50)]);
        assert_eq!(out.map(|r| r.w), [20, 40]);
    }

    /// Over-subscribed `Fixed` lanes are greedy in declaration order, and the overflow is reported.
    #[test]
    fn over_subscribed_fixed_lanes_are_greedy_and_the_overflow_is_reported() {
        let mut out = [Rect::default(); 3];
        let (n, fit) = Row::new().measure_into(
            Rect::new(0, 0, 20, 1),
            &[Fixed(15), Fixed(15), Fixed(15)],
            &mut out,
        );
        assert_eq!(n, 3);
        assert_eq!(out.map(|r| r.w), [15, 5, 0]);
        assert_eq!(fit.overflow, 25);
        assert_eq!(fit.slack, 0);
    }

    /// A band nothing elastic claims reports its remainder as slack, at the end and never as a hole.
    #[test]
    fn slack_is_a_trailing_remainder_and_never_a_hole() {
        let mut out = [Rect::default(); 2];
        let band = Rect::new(0, 0, 100, 1);
        let (n, fit) = Row::new().measure_into(band, &[Max(10), Max(10)], &mut out);
        assert_eq!(out.map(|r| r.w), [10, 10]);
        assert_eq!(fit.slack, 80);
        assert_eq!(
            assert_tiles(band, &out[..n], fit.gaps, Axis::X, "two caps"),
            80
        );
    }

    /// Weights that sum to zero claim nothing, and the whole band is slack.
    #[test]
    fn weights_summing_to_zero_claim_nothing() {
        let mut out = [Rect::default(); 3];
        let (_, fit) = Row::new().measure_into(
            Rect::new(0, 0, 40, 1),
            &[Weight(0), Weight(0), Weight(0)],
            &mut out,
        );
        assert_eq!(out.map(|r| r.w), [0, 0, 0]);
        assert_eq!(fit.slack, 40);
    }

    /// A zero-extent lane still advances the cursor, so its spacing is where it should be.
    #[test]
    fn a_zero_extent_lane_still_pays_its_spacing() {
        let out = Row::new()
            .spacing(2)
            .split(Rect::new(0, 0, 20, 1), [Fixed(0), Weight(1)]);
        assert_eq!(out[0], Rect::new(0, 0, 0, 1));
        assert_eq!(out[1], Rect::new(2, 0, 18, 1));
    }

    /// A `Col` divides the other axis and keeps the cross extent.
    #[test]
    fn a_column_divides_the_other_axis() {
        let out = Col::new().split(Rect::new(3, 7, 40, 9), [Fixed(2), Weight(1)]);
        assert_eq!(out[0], Rect::new(3, 7, 40, 2));
        assert_eq!(out[1], Rect::new(3, 9, 40, 7));
    }

    /// A margin insets both axes.
    #[test]
    fn a_margin_insets_both_axes() {
        let out = Row::new()
            .margin(1)
            .split(Rect::new(0, 0, 10, 6), [Weight(1)]);
        assert_eq!(out[0], Rect::new(1, 1, 8, 4));
    }

    // ---------------------------------------------------------------------------------------------
    // The degenerate inputs, every one of which is reachable by dragging a terminal edge.
    // ---------------------------------------------------------------------------------------------

    /// A zero-width band gives empty lanes and **no furniture at all**, which is the visible form of
    /// *this band cannot pay for its own margin*.
    #[test]
    fn a_zero_width_band_gives_empty_lanes_and_no_furniture() {
        let mut out = [Rect::default(); 3];
        let band = Rect::new(4, 4, 0, 0);
        let (n, fit) = Row::new().spacing(2).margin(1).measure_into(
            band,
            &[Fixed(10), Weight(1), Percent(50)],
            &mut out,
        );
        assert_eq!(n, 3);
        for lane in &out {
            assert_eq!((lane.w, lane.h), (0, 0));
        }
        assert_eq!(
            fit.gaps,
            Gaps::default(),
            "no margin and no spacing were paid"
        );
        assert_eq!(
            assert_tiles(band, &out[..n], fit.gaps, Axis::X, "zero band"),
            0
        );
    }

    /// The same on the other axis.
    #[test]
    fn a_zero_height_band_gives_empty_rows() {
        let mut out = [Rect::default(); 3];
        let band = Rect::new(0, 0, 40, 0);
        let (n, fit) = Col::new().spacing(2).margin(1).measure_into(
            band,
            &[Fixed(10), Weight(1), Percent(50)],
            &mut out,
        );
        assert_eq!(n, 3);
        for lane in &out {
            assert_eq!(lane.h, 0);
        }
        assert_eq!(fit.gaps, Gaps::default());
    }

    /// One column, and two lanes that each want everything.
    #[test]
    fn one_column_between_two_lanes_that_each_want_everything() {
        let out = Row::new().split(Rect::new(0, 0, 1, 1), [Fixed(u16::MAX), Fixed(u16::MAX)]);
        assert_eq!(out.map(|r| r.w), [1, 0]);
    }

    /// More lanes than columns is still a tiling.
    #[test]
    fn more_lanes_than_columns_is_still_a_tiling() {
        let mut out = [Rect::default(); 5];
        let band = Rect::new(0, 0, 3, 1);
        let (n, fit) = Row::new().measure_into(band, &[Fixed(2); 5], &mut out);
        assert_eq!(out.map(|r| r.w), [2, 1, 0, 0, 0]);
        assert_eq!(
            assert_tiles(band, &out[..n], fit.gaps, Axis::X, "five in three"),
            0
        );
    }

    /// A spec longer than the buffer is discarded **from the end**, and the count says how many
    /// arrived.
    #[test]
    fn a_spec_longer_than_the_buffer_is_discarded_from_the_end() {
        let mut out = [Rect::default(); 2];
        let (n, _) = Row::new().measure_into(Rect::new(0, 0, 30, 1), &[Weight(1); 5], &mut out);
        assert_eq!(n, 2);
        assert_eq!(out.map(|r| r.w), [15, 15]);
    }

    /// A zero denominator claims nothing rather than dividing by zero.
    #[test]
    fn a_zero_denominator_claims_nothing() {
        let out = Row::new().split(Rect::new(0, 0, 30, 1), [Ratio(1, 0), Weight(1)]);
        assert_eq!(out.map(|r| r.w), [0, 30]);
    }

    /// Over-claims are clamped rather than rejected.
    #[test]
    fn over_claims_are_clamped() {
        assert_eq!(
            Row::new().split(Rect::new(0, 0, 30, 1), [Percent(250)])[0].w,
            30
        );
        assert_eq!(
            Row::new().split(Rect::new(0, 0, 30, 1), [Ratio(9, 2)])[0].w,
            30
        );
    }

    /// Furniture larger than the band leaves every lane empty.
    #[test]
    fn furniture_larger_than_the_band_leaves_every_lane_empty() {
        let mut out = [Rect::default(); 3];
        let (n, _) = Row::new().spacing(9).margin(9).measure_into(
            Rect::new(0, 0, 6, 6),
            &[Weight(1); 3],
            &mut out,
        );
        assert_eq!(n, 3);
        for lane in &out {
            assert!(lane.is_empty(), "{lane:?} should be empty");
        }
    }

    /// Zero lanes is not an error.
    #[test]
    fn zero_lanes_is_not_an_error() {
        let out: [Rect; 0] = Row::new().split(Rect::new(0, 0, 10, 10), []);
        assert_eq!(out.len(), 0);
        let (n, fit) = solve(10, &[], &mut []);
        assert_eq!(n, 0);
        assert_eq!(fit.slack, 10);
    }

    // ---------------------------------------------------------------------------------------------
    // The rect algebra, `Align` and `Stack`.
    // ---------------------------------------------------------------------------------------------

    /// An inset larger than the rectangle collapses it **where the inset asked for it**.
    #[test]
    fn an_inset_larger_than_the_rectangle_collapses_it_in_place() {
        assert_eq!(rect::inset(Rect::new(0, 0, 3, 3), 9), Rect::new(9, 9, 0, 0));
    }

    /// `clamp_to` moves rather than crops, which is what a dialog dragged off the screen wants.
    #[test]
    fn clamp_to_moves_rather_than_crops() {
        let inside = rect::clamp_to(Rect::new(20, 20, 4, 4), Rect::new(0, 0, 10, 10));
        assert_eq!(inside, Rect::new(6, 6, 4, 4));
        // And it is not `intersect`, which would have cropped it to nothing.
        assert!(rect::intersect(Rect::new(20, 20, 4, 4), Rect::new(0, 0, 10, 10)).is_empty());
    }

    /// A cut larger than the rectangle takes all of it and leaves an empty remainder **after** it.
    #[test]
    fn a_cut_larger_than_the_rectangle_leaves_an_empty_remainder_after_it() {
        assert_eq!(
            rect::split_at_v(Rect::new(0, 0, 4, 3), 9),
            (Rect::new(0, 0, 4, 3), Rect::new(0, 3, 4, 0))
        );
        assert_eq!(
            rect::split_at_h(Rect::new(0, 0, 4, 3), 9),
            (Rect::new(0, 0, 4, 3), Rect::new(4, 0, 0, 3))
        );
    }

    /// `expand` and `shrink` are inverses on a rectangle large enough to survive the round trip.
    #[test]
    fn expand_and_inset_are_inverses_when_there_is_room() {
        let a = Rect::new(5, 5, 10, 10);
        assert_eq!(rect::inset(rect::expand(a, 2), 2), a);
    }

    /// Every `Align` corner and edge.
    #[test]
    fn align_places_all_nine_positions() {
        let a = Rect::new(0, 0, 10, 10);
        assert_eq!(Align::TopLeft.place(a, 4, 4), Rect::new(0, 0, 4, 4));
        assert_eq!(Align::Top.place(a, 4, 4), Rect::new(3, 0, 4, 4));
        assert_eq!(Align::TopRight.place(a, 4, 4), Rect::new(6, 0, 4, 4));
        assert_eq!(Align::Left.place(a, 4, 4), Rect::new(0, 3, 4, 4));
        assert_eq!(Align::Center.place(a, 4, 4), Rect::new(3, 3, 4, 4));
        assert_eq!(Align::Right.place(a, 4, 4), Rect::new(6, 3, 4, 4));
        assert_eq!(Align::BottomLeft.place(a, 4, 4), Rect::new(0, 6, 4, 4));
        assert_eq!(Align::Bottom.place(a, 4, 4), Rect::new(3, 6, 4, 4));
        assert_eq!(Align::BottomRight.place(a, 4, 4), Rect::new(6, 6, 4, 4));
    }

    /// Something larger than the band is the band.
    #[test]
    fn a_thing_larger_than_the_band_is_the_band() {
        let band = Rect::new(2, 3, 8, 4);
        assert_eq!(Stack::center(band, 40, 40), band);
    }

    /// `Stack::center` is `Align::Center`, asserted rather than claimed.
    #[test]
    fn stack_center_is_align_center() {
        let band = Rect::new(1, 2, 11, 7);
        for (w, h) in [(1, 1), (4, 4), (11, 7), (20, 20)] {
            assert_eq!(Stack::center(band, w, h), Align::Center.place(band, w, h));
        }
    }

    /// A grid is two splits, and a span takes in the spacing between the cells it covers.
    #[test]
    fn a_grid_is_two_splits_and_a_span_takes_in_the_spacing() {
        let band = Rect::new(0, 0, 32, 10);
        let grid = Grid::new(3, 2).spacing(1);
        let a = grid.cell(band, 0, 0, 1, 1);
        let b = grid.cell(band, 1, 0, 1, 1);
        assert_eq!(b.x, a.right() + 1, "one cell of spacing between them");
        let span = grid.cell(band, 0, 0, 2, 1);
        assert_eq!(span.x, a.x);
        assert_eq!(span.right(), b.right(), "the span covers both and the gap");
        // Out of range is empty, not a panic.
        assert!(grid.cell(band, 9, 0, 1, 1).is_empty());
        assert!(Grid::new(0, 0).cell(band, 0, 0, 1, 1).is_empty());
    }

    // ---------------------------------------------------------------------------------------------
    // The sweeps.
    // ---------------------------------------------------------------------------------------------

    /// **Zero gaps over 2 406 exhaustive cases**: six weight-only specs at every width from zero to
    /// four hundred.
    ///
    /// Exhaustive rather than random on the one corpus where exhaustiveness is cheap and where the
    /// property is strongest — a weight-only spec has no excuse for slack, so `slack == 0` is an
    /// equality here rather than a bound.
    #[test]
    fn zero_gaps_over_2406_exhaustive_weight_only_cases() {
        let specs: [&[Constraint]; 6] = [
            &[Weight(1)],
            &[Weight(1), Weight(1)],
            &[Weight(1), Weight(1), Weight(1)],
            &[Weight(3), Weight(1)],
            &[Weight(7), Weight(5), Weight(3), Weight(1)],
            &[Weight(1); 9],
        ];
        let mut out = [Rect::default(); C];
        let mut checked = 0usize;
        for spec in specs {
            for w in 0..=400u16 {
                let band = Rect::new(0, 0, w, 5);
                let (n, fit) = Row::new().measure_into(band, spec, &mut out);
                let slack = assert_tiles(band, &out[..n], fit.gaps, Axis::X, "exhaustive");
                assert_eq!(slack, 0, "spec {spec:?} at width {w} left {slack} columns");
                assert_eq!(fit.slack, 0);
                checked += 1;
            }
        }
        assert_eq!(checked, 6 * 401, "the corpus is 2 406 cases");
    }

    /// **Zero gaps over 200 000 random cases**, across three constraint alphabets, with random
    /// spacing and margin.
    ///
    /// Contiguity and containment are asserted on every case. **Exact coverage is asserted only on
    /// the fillable subset** — a spec with no elastic lane genuinely cannot fill its band, and
    /// demanding it would make this test wrong rather than strict.
    #[test]
    fn zero_gaps_over_200_000_random_cases() {
        let mut r = Rng::new(0x5EED_0003);
        let mut out = [Rect::default(); C];
        let mut fillable = 0usize;
        for case in 0..200_000u32 {
            let n = usize::try_from(r.range(1, C as u64)).expect("in range");
            let mut spec = [Weight(1); C];
            for lane in spec.iter_mut().take(n) {
                *lane = match case % 3 {
                    0 => Weight(u16::try_from(r.range(0, 8)).expect("small")),
                    1 => {
                        if r.below(2) == 0 {
                            Fixed(u16::try_from(r.below(40)).expect("small"))
                        } else {
                            Weight(u16::try_from(r.range(1, 8)).expect("small"))
                        }
                    }
                    _ => match r.below(6) {
                        0 => Fixed(u16::try_from(r.below(40)).expect("small")),
                        1 => Min(u16::try_from(r.below(30)).expect("small")),
                        2 => Max(u16::try_from(r.below(30)).expect("small")),
                        3 => Percent(u8::try_from(r.below(120)).expect("small")),
                        4 => Ratio(
                            u16::try_from(r.below(8)).expect("small"),
                            u16::try_from(r.below(8)).expect("small"),
                        ),
                        _ => Weight(u16::try_from(r.below(8)).expect("small")),
                    },
                };
            }
            let w = u16::try_from(r.below(320)).expect("small");
            let spacing = u16::try_from(r.below(4)).expect("small");
            let margin = u16::try_from(r.below(3)).expect("small");
            let band = Rect::new(
                i32::try_from(r.below(50)).expect("small"),
                i32::try_from(r.below(50)).expect("small"),
                w,
                9,
            );
            let (lanes, fit) =
                Row::new()
                    .spacing(spacing)
                    .margin(margin)
                    .measure_into(band, &spec[..n], &mut out);
            let slack = assert_tiles(band, &out[..lanes], fit.gaps, Axis::X, "random");

            let floors: u32 = spec[..n]
                .iter()
                .copied()
                .filter_map(|c| c.weighted().map(|(_, lo, _)| lo))
                .sum();
            let elastic = spec[..n]
                .iter()
                .any(|c| matches!(c, Weight(v) if *v > 0) || matches!(c, Min(_)));
            let furniture =
                u32::from(margin) * 2 + u32::from(spacing) * u32::try_from(n - 1).expect("small");
            if elastic && u32::from(w) >= furniture + floors {
                fillable += 1;
                assert_eq!(slack, 0, "case {case}: spec {:?} on {w}", &spec[..n]);
            }
        }
        assert!(
            fillable > 50_000,
            "only {fillable} of 200 000 cases were fillable, so this swept nothing"
        );
    }

    /// **The measurement that replaced the naive scheme: 10 957 gaps in 36 503 comparable specs
    /// against 0.**
    ///
    /// Comparable means *expressible in the naive scheme and not over-subscribed* — at least one
    /// non-zero weight, and the fixed lanes fitting. Anything else is not a fair comparison, since
    /// the naive scheme has nothing to say about `Percent` or `Max`.
    #[test]
    fn the_naive_scheme_leaves_gaps_and_this_one_does_not() {
        let mut r = Rng::new(0x5EED_0003);
        let mut out = [Rect::default(); C];
        let mut naive = [0u32; C];
        let mut comparable = 0usize;
        let mut naive_gaps = 0usize;
        let mut ours_gaps = 0usize;

        for _ in 0..50_000u32 {
            let n = usize::try_from(r.range(1, C as u64)).expect("in range");
            let mut spec = [Weight(1); C];
            for lane in spec.iter_mut().take(n) {
                *lane = if r.below(2) == 0 {
                    Fixed(u16::try_from(r.below(40)).expect("small"))
                } else {
                    Weight(u16::try_from(r.range(1, 8)).expect("small"))
                };
            }
            let w = u32::try_from(r.range(1, 320)).expect("small");
            let total_fixed: u32 = spec[..n]
                .iter()
                .copied()
                .filter_map(Constraint::fixed)
                .sum();
            let has_weight = spec[..n].iter().any(|c| matches!(c, Weight(v) if *v > 0));
            if !has_weight || total_fixed > w {
                continue;
            }
            comparable += 1;

            naive_columns(w, &spec[..n], &mut naive);
            let naive_end: u32 = naive[..n].iter().sum();
            if naive_end < w {
                naive_gaps += 1;
            }

            let (lanes, _) = solve(w, &spec[..n], &mut out);
            let ours_end: u32 = out[..lanes].iter().map(|r| u32::from(r.w)).sum();
            if ours_end < w {
                ours_gaps += 1;
            }
        }

        println!(
            "the naive scheme: {naive_gaps} gaps in {comparable} comparable specs \
             ({:.1}%), against {ours_gaps}",
            naive_gaps as f64 / comparable as f64 * 100.0
        );
        assert!(comparable > 10_000, "only {comparable} comparable specs");
        assert_eq!(ours_gaps, 0, "this scheme left a gap in {ours_gaps} specs");
        assert!(
            naive_gaps > comparable / 5,
            "the naive scheme left only {naive_gaps} gaps in {comparable} specs, so this test is no \
             longer measuring what it was written to measure"
        );
        // The minimal case, pinned so a reader does not have to run the sweep to see the shape.
        naive_columns(10, &[Weight(1), Weight(1), Fixed(3)], &mut naive);
        assert_eq!(naive[..3], [3, 3, 3], "naive loses a column");
        let out = Row::new().split(Rect::new(0, 0, 10, 1), [Weight(1), Weight(1), Fixed(3)]);
        assert_eq!(out.map(|r| r.w), [4, 3, 3]);
    }

    /// **The proportional group against exact rational arithmetic**, over a hundred thousand cases.
    ///
    /// `u128` over the product of the denominators is the oracle: what the group *should* claim, with
    /// no rounding anywhere. The `2^40` scale must land on that number, or on `avail` when the claim
    /// exceeds the band.
    #[test]
    fn the_proportional_group_matches_exact_rational_arithmetic() {
        let mut r = Rng::new(0x1234_0003);
        let mut out = [Rect::default(); C];
        let mut nontrivial = 0usize;
        for _ in 0..100_000u32 {
            let n = usize::try_from(r.range(1, 4)).expect("in range");
            let mut spec = [Weight(1); C];
            for lane in spec.iter_mut().take(n) {
                *lane = if r.below(2) == 0 {
                    Percent(u8::try_from(r.below(101)).expect("small"))
                } else {
                    let b = u16::try_from(r.range(1, 255)).expect("small");
                    Ratio(u16::try_from(r.below(u64::from(b) + 1)).expect("small"), b)
                };
            }
            let avail = u32::try_from(r.below(400)).expect("small");
            let (lanes, _) = solve(avail, &spec[..n], &mut out);
            let got: u32 = out[..lanes].iter().map(|r| u32::from(r.w)).sum();

            // The oracle: sum the rationals exactly over the product of the denominators.
            let den: u128 = spec[..n]
                .iter()
                .copied()
                .filter_map(Constraint::proportional)
                .map(|(_, b)| u128::from(b))
                .product();
            let num: u128 = spec[..n]
                .iter()
                .copied()
                .filter_map(Constraint::proportional)
                .map(|(a, b)| u128::from(a) * (den / u128::from(b)))
                .sum();
            let exact = u32::try_from(num * u128::from(avail) / den).unwrap_or(u32::MAX);
            assert_eq!(
                got,
                exact.min(avail),
                "spec {:?} on {avail}: exact {exact}",
                &spec[..n]
            );
            if got > 0 && got < avail {
                nontrivial += 1;
            }
        }
        assert!(nontrivial > 20_000, "only {nontrivial} non-trivial cases");
    }

    /// **`Max`'s fixpoint is bounded at one round per lane.**
    ///
    /// Every round that does not break freezes at least one lane and a frozen lane never moves
    /// again, so the bound is structural. Fifty thousand random cap-heavy specs is the check.
    #[test]
    fn the_cap_fixpoint_is_bounded_at_one_round_per_lane() {
        let mut r = Rng::new(0xC0FF_EE03);
        let mut out = [Rect::default(); C];
        let mut worst = 0u32;
        let mut capped = 0usize;
        for _ in 0..50_000u32 {
            let n = usize::try_from(r.range(1, C as u64)).expect("in range");
            let mut spec = [Weight(1); C];
            for lane in spec.iter_mut().take(n) {
                *lane = match r.below(3) {
                    0 => Max(u16::try_from(r.below(30)).expect("small")),
                    1 => Min(u16::try_from(r.below(30)).expect("small")),
                    _ => Weight(u16::try_from(r.range(1, 8)).expect("small")),
                };
            }
            let avail = u32::try_from(r.below(200)).expect("small");
            let (_, fit) = solve(avail, &spec[..n], &mut out);
            assert!(
                fit.rounds <= u32::try_from(n).expect("small"),
                "{} rounds for {n} lanes: {:?} on {avail}",
                fit.rounds,
                &spec[..n]
            );
            worst = worst.max(fit.rounds);
            if fit.rounds > 1 {
                capped += 1;
            }
        }
        assert!(
            worst >= 2,
            "no case ever needed a second round, so nothing was tested"
        );
        assert!(capped > 100, "only {capped} cases engaged a ceiling");
    }

    /// **Every degenerate arrival, on both axes, at every tiny size.** The only assertion is that it
    /// arrives: no panic, no overflow, no divide by zero.
    ///
    /// Three hundred and sixty thousand cases, and the point is that there is nothing to assert
    /// beyond arrival — a layout that returns rectangles for a two-cell terminal has already done
    /// the only thing it can.
    #[test]
    fn every_degenerate_arrival_on_both_axes() {
        let mut r = Rng::new(0xDEAD_0003);
        let mut out = [Rect::default(); C];
        let mut arrived = 0usize;
        for _ in 0..20_000u32 {
            let n = usize::try_from(r.range(1, C as u64)).expect("in range");
            let mut spec = [Weight(1); C];
            for lane in spec.iter_mut().take(n) {
                *lane = match r.below(6) {
                    0 => Fixed(u16::try_from(r.below(40)).expect("small")),
                    1 => Min(u16::try_from(r.below(30)).expect("small")),
                    2 => Max(u16::try_from(r.below(30)).expect("small")),
                    3 => Percent(u8::try_from(r.below(120)).expect("small")),
                    4 => Ratio(
                        u16::try_from(r.below(8)).expect("small"),
                        u16::try_from(r.below(8)).expect("small"),
                    ),
                    _ => Weight(u16::try_from(r.below(8)).expect("small")),
                };
            }
            let spacing = u16::try_from(r.below(3)).expect("small");
            let margin = u16::try_from(r.below(3)).expect("small");
            for w in 0..=8u16 {
                let band = Rect::new(0, 0, w, w);
                let (rows, rfit) = Row::new().spacing(spacing).margin(margin).measure_into(
                    band,
                    &spec[..n],
                    &mut out,
                );
                assert_tiles(band, &out[..rows], rfit.gaps, Axis::X, "degenerate row");
                let (cols, cfit) = Col::new().spacing(spacing).margin(margin).measure_into(
                    band,
                    &spec[..n],
                    &mut out,
                );
                assert_tiles(band, &out[..cols], cfit.gaps, Axis::Y, "degenerate col");
                arrived += 2;
            }
        }
        assert_eq!(arrived, 20_000 * 9 * 2, "360 000 arrivals");
    }

    /// `solve` leaves no frozen bit behind: a caller reading `x` sees zero.
    #[test]
    fn solve_clears_the_frozen_bits_it_used() {
        let mut out = [Rect::default(); 3];
        let (n, fit) = solve(30, &[Max(5), Weight(1), Weight(1)], &mut out);
        assert!(fit.rounds >= 1, "the fixpoint did engage");
        for slot in &out[..n] {
            assert_eq!(slot.x, 0, "a frozen bit survived into the result");
            assert_eq!(slot.y, 0);
            assert_eq!(slot.h, 0);
        }
    }
}
