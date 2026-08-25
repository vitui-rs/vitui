//! **Axes, and spec §9's layout loop in a second place — where it oscillates.**
//!
//! Components ticket 28, spec §13. The loop is real and it is short: the y-axis gutter takes columns
//! from the plotting area; the plotting area's width decides how much of a live series is on screen;
//! what is on screen decides the auto-scaled range; the range decides the tick values; the tick
//! values decide the widest label; the widest label is the gutter.
//!
//! §9's precondition becomes *a narrower plotting area may not produce a wider label*, and unlike
//! §9's it is **false for ordinary data**: dropping the older, larger samples out of the window can
//! leave a range whose nice ticks are `-0.05 … 0.05`, and `-0.05` is five columns where `900` was
//! three.
//!
//! > **A layout loop is worth solving only when both ends are genuinely layout; when one end is
//! > data, decouple it.**
//!
//! So the fix is not a better fixpoint. It is to **cut the edge**: [`Sizing::WholeDomain`] computes
//! the gutter from the whole series' range, which is data, so it does not depend on the rectangle at
//! all and there is no loop to solve. Over [`AXIS_PAIRS`] viewport x dataset pairs the naive form
//! oscillates on **464**, the hysteresis form settles on a gutter that is not a fixed point of its
//! own rule on **all 464**, and the whole-domain form is **0 oscillations, always 1 pass**.
//!
//! **A caller who insists on a window-scaled axis must declare the range** — the same shape of
//! answer as reserved bars. That is [`crate::chart::Opts::range`] and it is not this module's
//! business: nothing here reads a window unless it is asked to.

use super::raster::Domain;

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
