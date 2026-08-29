//! **O6 — every component that takes a data volume holds sixty hertz at a million inputs**, as a
//! query over [`crate::INVENTORY`] and an instrument that runs the shipped components.
//!
//! Spec §17 states its obligations as queries over the freeze (ADR 0033); components ticket 44
//! states this one **after the map closed**, and it is an implementation ticket rather than an
//! architecture issue because the *instrument* is buildable without reopening anything. ADR 0049 is
//! the decision and §17's table carries the row.
//!
//! # The hole this closes, stated once
//!
//! > This crate can prove a frame's **output** is flat in the data volume. It cannot prove its
//! > **work** is.
//!
//! `chart`'s rasteriser painted the whole column prefix for **every point**. A bar is a prefix and
//! the union of two prefixes is the taller one, so the picture was correct — and at a million
//! points onto four hundred sub-columns the same cells were painted thousands of times over:
//! **952.61 ms against 2.72**, 476.3 ns a point against 1.4. Every gate in this workspace was green
//! on it, and each for a different and good reason:
//!
//! - the **round trip** replays the serialised bytes through the terminal model and compares them
//!   against the frame that produced them, and both spellings produce the identical frame;
//! - the **flat-in-n gates** — `writes`, `verbs`, `regions`, `stops` identical at 1k / 100k / 1M —
//!   are all *output* counters, and the defect was entirely in work that produces none;
//! - [`crate::chart::raster::Raster::touched`] was **right**: 2 000 000 both ways. The cost was
//!   `O(subh)` *inside* each visit, which none of spec §20's nine counters expresses;
//! - the budget example measures the engine, and the fold sits above it.
//!
//! It was found by a user pressing `+` on `crates/vitui-apps/examples/latency.rs`.
//!
//! # What is gated, and why neither half would do on its own
//!
//! **A count, twice.** [`Measured::work`] is a step count — the runtime's scene 19 arrangement one
//! crate down, and for that scene's stated reason: *a step count is the same number on every
//! machine*, where a microsecond figure is three orders apart between a debug binary and a release
//! one and would be three different orders somewhere else. The absolute frame time is a **report**
//! with its headroom beside it, in `examples/volume_numbers.rs`, and it is never a pass mark.
//!
//! 1. **A growth relation.** `work(n) / work(n / 10) <= `[`GROWTH`], at three volumes a decade
//!    apart. Superlinear work fails; a faster machine passes unchanged.
//! 2. **A ceiling, affine in the volume.** `work(n) <= fixed + per_input * n`.
//!
//! **A relation alone is blind to a constant** — a fold that is linear at 4 µs a point is linear
//! and useless, and the defect that actually shipped was *linear*: `O(subh)` a point is `O(n)` with
//! `subh` in front of it, so its growth relation is the shipped fold's exactly. **A ceiling alone is
//! met by any constant chosen large enough**, which is a threshold on the wrong side of the
//! question and §21's first refinement by name. So O6 is both, and
//! [`crate::chart::raster::defective`] carries one arm for each half: `naive_bars` fails the ceiling
//! with a textbook growth relation, and `domain_per_point` fails the relation.
//!
//! # The population is derived, and it grew by one row while being derived
//!
//! [`population`] is not a list. It is *rows that take a volume*, read off the freeze:
//! [`Layer::L2`] is already the column that says **it costs its visible window and never the data
//! volume**, and beside it the rows that **fold on the edit**, which [`crate::memos`] already
//! answers — a [`Spelling::Folded`] memo is a value keyed on a data revision, which is what folding
//! on the edit *is*.
//!
//! Ticket 44 names six rows in its own parenthesis: the four `L2` rows and `chart` and `plot`. The
//! derivation answers **seven**, because `sparkline` holds
//! [`crate::chart::raster::PlotState`]'s two memos as well — it is `chart`'s body with the chrome
//! deleted (components 34) and it folds a million points through the same `Raster`. That is the
//! criterion working rather than failing: *derived, so a component added later joins without anyone
//! remembering*, and `sparkline` shipped ten tickets after the sentence that missed it.
//!
//! `field`'s two spellings are the other half of the same reading. Its volume is text and its
//! [`WrapKind::Ruler`] index exists so that **a boundary is never more than one window behind the
//! caret**; `crate::edit`'s memo is hand-rolled, so it is `L2` that puts it here and not the memo
//! census. The two derivations agree about it and neither is redundant.
//!
//! # Every covered row owes a deliberate defect, or the gate is a claim about nothing
//!
//! Six of the seven arms are §21's *the instrument separates a correct build from a defective one*,
//! and the seventh is the defect that shipped. Four are the row shape — **the single most expensive
//! mistake available above this runtime** — and building them found that
//! [`crate::collect::defective::whole_content`] existed for **one caller of three**: `table` and
//! `tree` *are* `collection` plus a rectangle split and a flatten index, and neither could be asked
//! to make the mistake it inherits. [`crate::collect::defective::table_whole_content`] and
//! [`crate::collect::defective::tree_whole_content`] are that arm on the other two, one field each.
//!
//! # Where an arm runs is what a debug binary can hold, and the two criteria are scale-free
//!
//! [`Covered::volumes`] is `10 000 / 100 000 / 1 000 000` on every shipped arm — ticket 44's own
//! figure, and the volume the obligation is stated at. [`Covered::defect_volumes`] is smaller,
//! per row, and that is the whole of the difference: `naive_bars` at a million points is eighty
//! million paints in an unoptimised binary and `domain_per_point` at a million is `10^12` reads.
//! **A defect you can afford to run is a defect you can watch**, and both criteria are ratios and
//! affine bounds — neither has a volume baked into it — so an arm watched failing three decades
//! down is watched failing the same two numbers.
//!
//! [`Layer::L2`]: crate::inventory::Layer::L2
//! [`Spelling::Folded`]: crate::memos::Spelling::Folded
//! [`WrapKind::Ruler`]: crate::edit::WrapKind::Ruler

use std::ops::Range;
use std::time::Instant;

use vitui_runtime::theme::GlyphSet;
use vitui_runtime::{Ctx, Interest, Rect, Role};

use crate::chart::raster::{self, Domain, Geom, Kind, RUNGS, Raster, Reach, geom};
use crate::collect::{
    CollOpts, CollState, Column, Node, TableOpts, TableState, TreeOpts, TreeState, collection_into,
    defective as coll_defective, table_into, tree_into,
};
use crate::edit::{Caret, Text, WrapKind};
use crate::ink::{Direct, Ink};
use crate::inventory::{INVENTORY, Layer};
use crate::memos::{MEMOS, Spelling};
use crate::order::{Entry, Order, Rows};

/// **The three volumes O6 is stated at.** Ticket 44's own figure, and a decade apart because
/// [`GROWTH`] is a ratio between neighbours.
pub const VOLUMES: [u64; 3] = [10_000, 100_000, 1_000_000];

/// **What one decade of volume may cost.**
///
/// Near ten, and above it rather than at it: ten is what linear work costs between two volumes a
/// decade apart, and a component whose fixed cost is a large share of the smaller volume reads
/// *under* ten rather than over. Twelve is the headroom, written here once.
///
/// A quadratic arm reads a hundred, so the gate has two decimal orders of daylight and is not tuned
/// to any measurement — R15's rule about a gate at cliff granularity, on a ratio rather than on a
/// clock.
pub const GROWTH: f64 = 12.0;

/// **The frame budget sixty hertz is, in nanoseconds.** A **report** figure and never a gate.
///
/// It appears in `examples/volume_numbers.rs` as the denominator of a headroom fraction and nowhere
/// else. The gates on this page are two counts, for the reason this module exists.
pub const BUDGET_NANOS: u64 = 16_667_000;

/// The screen every drawn arm is measured on. §21's own 300x80.
pub const W: u16 = 300;
/// See [`W`].
pub const H: u16 = 80;

/// The rectangle a `chart`, a `plot` or a `sparkline` is rasterised into here.
const PANE: (u16, u16) = (60, 20);
/// The rectangle a `sparkline` is rasterised into: `chart`'s body with the chrome deleted.
const STRIP: (u16, u16) = (20, 5);
/// **The rung the two bar folds are measured at, and the one the marks fold is.**
///
/// `RUNGS[1]` and `RUNGS[2]` rather than the repertoire's own names: **a file in `src/` may not
/// spell a `GlyphSet`** (§16, and `crate::gates`' scan holds the one exception to three lines).
/// `chart::raster::RUNGS` is the door, and this is the fourth caller it was made public for.
const BARS_AT: GlyphSet = RUNGS[1];
/// See [`BARS_AT`]. The braille rung, where a cell's states are a power set.
const MARKS_AT: GlyphSet = RUNGS[2];

/// The width a `field` is measured at, which is what a `Ruler` row is worth.
const FIELD_W: u16 = 80;
/// How many columns the table declares. `crate::grid`'s twelve, so the two screens agree.
const COLUMNS: usize = 12;

/// Which build of a covered component is being measured.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Arm {
    /// The one that ships.
    Shipped,
    /// The deliberate defect, named in [`Covered::defect`].
    Defective,
}

impl Arm {
    /// Both, which is what a sweep iterates.
    pub const ALL: [Arm; 2] = [Arm::Shipped, Arm::Defective];

    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Arm::Shipped => "shipped",
            Arm::Defective => "defective",
        }
    }
}

/// **Why a row of the freeze is covered by O6**, which is also the shape of the ceiling it is held
/// to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Why {
    /// [`Layer::L2`] — *it costs its visible window and never the data
    /// volume*. Its ceiling has **no `n` in it at all**, which is that sentence as arithmetic.
    Virtualised,
    /// It keeps a memo whose key is a data revision, so an edit costs a pass over the data. Its
    /// ceiling is affine and the slope is what a point may cost.
    FoldsOnEdit,
}

impl Why {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Why::Virtualised => "virtualised",
            Why::FoldsOnEdit => "folds on the edit",
        }
    }
}

/// **What one run of one arm at one volume cost**: a step count, and the clock beside it.
///
/// Two numbers from one setup, and the split is where the bracket goes. **The fixture is built
/// outside it** — the series, the buffer, the flatten index, the headless driver and its warm-up
/// frame — because a figure that includes `"a".repeat(1_000_000)` is a figure about a `String`.
/// `crate::listing::volume_over` makes the same split one file over and for the same reason.
///
/// [`Reading::steps`] is what O6 is gated on and [`Reading::nanos`] is what
/// `examples/volume_numbers.rs` prints. A gate is a count, a ratio, an equality or a compile
/// outcome; a timing is a report.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Reading {
    /// The work, in [`Covered::unit`]s. **The gate.**
    pub steps: u64,
    /// How long the measured region took, in nanoseconds. **A report and never a gate.**
    pub nanos: u128,
}

/// One covered row: what its work is, what it may cost, and the defect it is separated from.
#[derive(Clone, Copy)]
pub struct Covered {
    /// The row of [`INVENTORY`] it is.
    pub id: &'static str,
    /// Why the derivation reached it.
    pub why: Why,
    /// What one step of [`Covered::run`] is, in words.
    pub unit: &'static str,
    /// **The part of the work that is not the volume's.** A visible window, a raster, a row.
    ///
    /// Every one of the seven is written as **four times** what the mechanism costs — eighty rows,
    /// nine hundred and sixty cells, one window of clusters, a raster's own sub-cells — so that the
    /// bound is a *cliff* and not the current measurement. R15's rule, on a ceiling rather than on a
    /// clock: a gate tuned to what was measured is a flaky test that gets disabled within a month,
    /// and every defective arm on this page misses it by three to three thousand times.
    pub fixed: u64,
    /// **What one input may cost above that.** Zero for every [`Why::Virtualised`] row, which is the
    /// claim rather than a rounding of it.
    pub per_input: f64,
    /// Where the shipped arm is measured. [`VOLUMES`] on every row.
    pub volumes: [u64; 3],
    /// Where the defective arm is measured. See this module's header: it is what a debug binary can
    /// hold, and both criteria are scale-free.
    pub defect_volumes: [u64; 3],
    /// The deliberate defect, in one line ending in what it costs.
    pub defect: &'static str,
    /// **The measurement.** Runs the component and answers a step count with a clock beside it.
    pub run: fn(Arm, u64) -> Reading,
}

/// **The seven covered rows.** Ordered as [`INVENTORY`] orders them, so a row inserted into the
/// freeze in the middle is a failing test here rather than a tidy append.
pub const COVERED: &[Covered] = &[
    Covered {
        id: "field",
        why: Why::Virtualised,
        unit: "clusters walked by one `Left` at the end of the buffer",
        fixed: 4 * FIELD_W as u64,
        per_input: 0.0,
        volumes: VOLUMES,
        defect_volumes: VOLUMES,
        defect: "`edit::defective::blind_left` — the caret moved from the only boundary a build \
                 with no index has, which is byte 0: one cluster against the buffer",
        run: field_run,
    },
    Covered {
        id: "collection",
        why: Why::Virtualised,
        unit: "row-drawer calls in one frame",
        fixed: 4 * H as u64,
        per_input: 0.0,
        volumes: VOLUMES,
        defect_volumes: [1_000, 10_000, 100_000],
        defect: "`collect::defective::whole_content` — the content iterated and the clip left to \
                 reject the rest, which writes exactly what the window writes",
        run: collection_run,
    },
    Covered {
        id: "table",
        why: Why::Virtualised,
        unit: "cell-drawer calls in one frame",
        fixed: 4 * H as u64 * COLUMNS as u64,
        per_input: 0.0,
        volumes: VOLUMES,
        defect_volumes: [1_000, 10_000, 100_000],
        defect: "`collect::defective::table_whole_content` — `whole_content` on the component \
                 built on it, which until this ticket had no arm at all",
        run: table_run,
    },
    Covered {
        id: "tree",
        why: Why::Virtualised,
        unit: "row-drawer calls in one frame",
        fixed: 4 * H as u64,
        per_input: 0.0,
        volumes: VOLUMES,
        defect_volumes: [1_000, 10_000, 100_000],
        defect: "`collect::defective::tree_whole_content` — the flatten index iterated whole, \
                 which is a different quantity from `unclamped_indent`'s depth",
        run: tree_run,
    },
    Covered {
        id: "chart",
        why: Why::FoldsOnEdit,
        unit: "points visited plus cells painted, in one fold",
        fixed: 4 * PANE.0 as u64 * PANE.1 as u64 * 8,
        per_input: 2.0,
        volumes: VOLUMES,
        defect_volumes: [300, 3_000, 30_000],
        defect: "`chart::raster::defective::naive_bars` — the whole column prefix painted once a \
                 point, `O(subh)` inside a visit `touched` counts as one",
        run: chart_run,
    },
    Covered {
        id: "plot",
        why: Why::FoldsOnEdit,
        unit: "points visited plus cells painted plus values read, in one fold",
        fixed: 4 * PANE.0 as u64 * PANE.1 as u64 * 8,
        per_input: 3.0,
        volumes: VOLUMES,
        defect_volumes: [30, 300, 3_000],
        defect: "`chart::raster::defective::domain_per_point` — the domain recomputed from the \
                 whole series at every point, which draws the identical raster",
        run: plot_run,
    },
    Covered {
        id: "sparkline",
        why: Why::FoldsOnEdit,
        unit: "points visited plus cells painted, in one fold",
        fixed: 4 * STRIP.0 as u64 * STRIP.1 as u64 * 8,
        per_input: 2.0,
        volumes: VOLUMES,
        defect_volumes: [300, 3_000, 30_000],
        defect: "`chart::raster::defective::naive_bars` at the strip's own geometry — one fold, \
                 two callers, and the second geometry is what says the defect is the fold's",
        run: sparkline_run,
    },
];

/// **The population, derived from the freeze and from the memo census.**
///
/// *Rows that take a volume*: [`Layer::L2`], plus every row that holds
/// a memo whose key is a data revision. See this module's header for why the second half is
/// [`Spelling::Folded`] and for the row the derivation found that ticket 44's own parenthesis did
/// not name.
///
/// In [`INVENTORY`]'s order, so [`COVERED`] can be compared against it as an ordered list.
pub fn population() -> Vec<&'static str> {
    INVENTORY
        .iter()
        .filter(|c| {
            c.layer == Layer::L2
                || MEMOS
                    .iter()
                    .any(|m| m.spelling == Spelling::Folded && m.holders.contains(&c.id))
        })
        .map(|c| c.id)
        .collect()
}

/// The covered row `id` is, if it is one.
pub fn covered(id: &str) -> Option<&'static Covered> {
    COVERED.iter().find(|c| c.id == id)
}

/// **What one arm of one component cost, at three volumes.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Measured {
    /// The row of the freeze.
    pub id: &'static str,
    /// Which build.
    pub arm: Arm,
    /// Where it was measured.
    pub volumes: [u64; 3],
    /// What it cost, in [`Covered::unit`]s.
    pub work: [u64; 3],
}

impl Measured {
    /// **The two growth ratios**, neighbour over neighbour.
    ///
    /// A flat arm reads `1.0` and a linear one reads the decade. A zero denominator answers `0.0`
    /// rather than an infinity: an arm that did no work at the smaller volume has not shown growth,
    /// and [`Measured::holds`] refuses it separately.
    pub fn growth(&self) -> [f64; 2] {
        let r = |a: u64, b: u64| {
            if a == 0 { 0.0 } else { b as f64 / a as f64 }
        };
        [r(self.work[0], self.work[1]), r(self.work[1], self.work[2])]
    }

    /// **The ceiling at each volume**, `fixed + per_input * n`.
    pub fn ceiling(&self, row: &Covered) -> [u64; 3] {
        let at = |n: u64| row.fixed + (row.per_input * n as f64) as u64;
        [
            at(self.volumes[0]),
            at(self.volumes[1]),
            at(self.volumes[2]),
        ]
    }

    /// Whether every volume sits under its ceiling.
    pub fn within_ceiling(&self, row: &Covered) -> bool {
        let ceiling = self.ceiling(row);
        (0..3).all(|i| self.work[i] <= ceiling[i])
    }

    /// Whether neither decade grew by more than [`GROWTH`].
    pub fn within_growth(&self) -> bool {
        self.growth().iter().all(|g| *g <= GROWTH)
    }

    /// **Whether it answers O6.** Both halves, and neither implies the other.
    ///
    /// **An arm that did no work at a volume has not been measured there**, and it is refused before
    /// either bound is looked at — [`crate::obligations::Verdict::of`]'s vacuity refusal one file
    /// over, on a measurement rather than on a population. A ratio between two zeroes is not a
    /// growth of one, and a zero is under every ceiling.
    pub fn holds(&self, row: &Covered) -> bool {
        self.work.iter().all(|w| *w > 0) && self.within_ceiling(row) && self.within_growth()
    }

    /// What is wrong, in one line, or `None`.
    ///
    /// Names the criterion rather than the number alone, because the two halves are watched failing
    /// separately and a report that said only *too dear* could not tell them apart.
    pub fn why_not(&self, row: &Covered) -> Option<String> {
        if let Some(i) = (0..3).find(|i| self.work[*i] == 0) {
            return Some(format!(
                "{} did no work at {} inputs, so nothing was measured there",
                self.id, self.volumes[i]
            ));
        }
        if !self.within_ceiling(row) {
            let c = self.ceiling(row);
            let i = (0..3).find(|i| self.work[*i] > c[*i]).unwrap_or(2);
            return Some(format!(
                "{} exceeds the ceiling: {} {} at {} inputs, against {}",
                self.id, self.work[i], row.unit, self.volumes[i], c[i]
            ));
        }
        if !self.within_growth() {
            let g = self.growth();
            return Some(format!(
                "{} grows by {:.2} and {:.2} a decade, against {GROWTH:.2}",
                self.id, g[0], g[1]
            ));
        }
        None
    }
}

/// **Run one arm of one covered row at its three volumes.**
pub fn measure(row: &Covered, arm: Arm) -> Measured {
    let volumes = match arm {
        Arm::Shipped => row.volumes,
        Arm::Defective => row.defect_volumes,
    };
    Measured {
        id: row.id,
        arm,
        volumes,
        work: [
            (row.run)(arm, volumes[0]).steps,
            (row.run)(arm, volumes[1]).steps,
            (row.run)(arm, volumes[2]).steps,
        ],
    }
}

/// **Every shipped arm, measured.** What the report prints and what [`met`] reads.
pub fn shipped() -> Vec<Measured> {
    COVERED
        .iter()
        .map(|row| measure(row, Arm::Shipped))
        .collect()
}

/// **The ids whose shipped arm answers O6 with a number inside both bounds.**
///
/// [`crate::obligations::o6`] takes this as its evidence, and
/// `crate::obligations::VOLUME_MEASURED` is the written-out list it is joined against — two sources
/// rather than one read twice, which is the arrangement every other obligation's evidence already
/// uses.
pub fn met() -> Vec<&'static str> {
    COVERED
        .iter()
        .filter(|row| measure(row, Arm::Shipped).holds(row))
        .map(|row| row.id)
        .collect()
}

// ── the seven measurements ───────────────────────────────────────────────────────────────────────

/// A ramp with a sawtooth in it, so a bar fold has a top to find in every sub-column and a mark fold
/// has points on both sides of the domain.
fn series_of(n: u64) -> Vec<Vec<f32>> {
    let n = usize::try_from(n).unwrap_or(usize::MAX);
    vec![(0..n).map(|i| ((i % 97) as f32) / 96.0).collect()]
}

/// Time `f`, and hand back what it counted.
fn timed(f: impl FnOnce() -> u64) -> Reading {
    let started = Instant::now();
    let steps = f();
    Reading {
        steps,
        nanos: started.elapsed().as_nanos(),
    }
}

/// The bar fold's own work: [`Raster::touched`] plus [`Raster::painted`].
///
/// **The two together and not either alone.** `touched` was 2 000 000 both ways across the reduce
/// and `painted` alone would report a windowed component as free; the sum is *what the fold did*.
///
/// The series, the geometry and the domain are built **outside** the bracket: they are the caller's
/// data and the caller's rectangle, and the fold is what O6 is about.
fn raster_run(kind: Kind, size: (u16, u16), set: GlyphSet, arm: Arm, n: u64) -> Reading {
    let series = series_of(n);
    let g: Geom = geom(kind, set);
    let dom: Domain = raster::domain_of(&series, raster::Range::Whole).padded();
    let mut r = Raster::empty();
    timed(|| {
        match arm {
            Arm::Shipped => r.build(
                size.0,
                size.1,
                kind,
                g,
                dom,
                &series,
                Reach::Mapped,
                (0, usize::MAX),
            ),
            Arm::Defective => {
                raster::defective::naive_bars(&mut r, size.0, size.1, g, dom, &series);
            }
        }
        r.touched() + r.painted()
    })
}

/// `chart` — bars into a pane, at the Unicode rung.
fn chart_run(arm: Arm, n: u64) -> Reading {
    raster_run(Kind::Bars, PANE, BARS_AT, arm, n)
}

/// `sparkline` — the same fold at the strip's geometry, which is what says the cost is the fold's
/// and not the pane's.
fn sparkline_run(arm: Arm, n: u64) -> Reading {
    raster_run(Kind::Bars, STRIP, BARS_AT, arm, n)
}

/// `plot` — marks into a pane at the braille rung, where a cell's states are a power set.
///
/// Its defective arm is the quadratic one, and the reads it makes are added to the fold's own work
/// because they are work the fold did: a `Raster` has no field for a cost its build did not pay, and
/// inventing one would put the defect's bookkeeping in the shipped type.
///
/// **The domain is inside the bracket on both arms**, which is the one departure from
/// [`raster_run`] and is the point of this row: the shipped chain asks for it once and memoises it
/// on the data's revision, and the defective one asks inside the point loop.
fn plot_run(arm: Arm, n: u64) -> Reading {
    let series = series_of(n);
    let g = geom(Kind::Marks, MARKS_AT);
    let mut r = Raster::empty();
    timed(|| match arm {
        Arm::Shipped => {
            let dom = raster::domain_of(&series, raster::Range::Whole).padded();
            r.build(
                PANE.0,
                PANE.1,
                Kind::Marks,
                g,
                dom,
                &series,
                Reach::Mapped,
                (0, usize::MAX),
            );
            r.touched() + r.painted()
        }
        Arm::Defective => {
            let scanned = raster::defective::domain_per_point(
                &mut r,
                PANE.0,
                PANE.1,
                Kind::Marks,
                g,
                &series,
            );
            r.touched() + r.painted() + scanned
        }
    })
}

/// `field` — one `Left` at the end of an `n`-byte buffer.
///
/// **The component's own index decides where the walk starts**, which is the whole of §11: a
/// [`WrapKind::Ruler`] index exists so that a boundary is never more than one window behind the
/// caret. The defective arm starts from the only boundary a build with no index has.
///
/// The buffer and the index are built outside the bracket. §11's own figure is *one `Left` at the
/// end of a pasted megabyte*, so the paste is not the measurement.
fn field_run(arm: Arm, n: u64) -> Reading {
    let n = usize::try_from(n).unwrap_or(usize::MAX);
    let mut st = Text::of("a".repeat(n), WrapKind::Ruler);
    st.set_pos(Caret::of(n, 0));
    let caret = st.caret();
    let start = {
        let index = st.index(FIELD_W);
        let row = index.row_of(caret.byte());
        index.row_start(row)
    };
    let text = st.text();
    timed(|| {
        let mut steps = 0u64;
        let _ = match arm {
            Arm::Shipped => {
                crate::edit::step_left_counted(text, caret, Caret::of(start, 0), &mut steps)
            }
            Arm::Defective => crate::edit::defective::blind_left_counted(text, caret, &mut steps),
        };
        steps
    })
}

/// The row a drawn arm writes: a label and its trailing pad, so the row is a partition of its width.
fn write_row(ink: &mut Direct, cx: &mut Ctx<'_, '_>, r: Rect, w: u16) {
    let paint = cx.theme().paint(Role::Body);
    let written = ink.text(cx, r.x, r.y, "a row that does not move", paint);
    let _ = ink.run(
        cx,
        r.x + i32::from(written),
        r.y,
        " ",
        w.saturating_sub(written),
        paint,
    );
}

/// **A headless driver at [`W`] by [`H`], one warm-up frame, then one measured frame.**
///
/// Attached outside the bracket and warmed inside it once, which is `crate::listing::volume_over`'s
/// arrangement and its reason: the frame structures take their allocation on the first frame that
/// needs one and keep it, so a cold frame is not a frame — and an attach inside the clock is a
/// figure about `Driver::headless`.
///
/// **Through [`Direct`] and not through a `Tally`**, for `crate::listing::volume_cost`'s reason: a
/// tally keeps a `BTreeSet` of every cell it sees, so a figure taken with one in the loop is a
/// figure about the instrument. The counter here is the drawer's own `u64`, so the seam costs
/// nothing and the component drawn is the shipped one.
fn drawn(mut draw: impl FnMut(&mut Ctx<'_, '_>, &mut Direct) -> u64) -> Reading {
    let mut driver = crate::runner::driver_at(W, H, vitui_runtime::theme::Density::default());
    let mut ink = Direct;
    driver.frame(|cx| {
        let _ = draw(cx, &mut ink);
    });
    let started = Instant::now();
    let mut steps = 0u64;
    driver.frame(|cx| steps = draw(cx, &mut ink));
    Reading {
        steps,
        nanos: started.elapsed().as_nanos(),
    }
}

/// `collection` — row-drawer calls in one frame over `n` rows.
fn collection_run(arm: Arm, n: u64) -> Reading {
    let rows = Rows::of(usize::try_from(n).unwrap_or(usize::MAX));
    drawn(move |cx, ink| {
        let view = cx.area();
        let opts = CollOpts::default();
        let mut st = CollState::new();
        let find = |_: &str, _: Range<usize>| None;
        let mut iterated = 0u64;
        {
            let row = |ink: &mut Direct, cx: &mut Ctx<'_, '_>, r: Rect, i: usize, _f| {
                iterated += 1;
                cx.with_key(i as u64, |cx| {
                    let id = cx.id();
                    let _ = cx.interact(id, r, Interest::CLICK);
                    write_row(ink, cx, r, W);
                });
            };
            let _ = match arm {
                Arm::Shipped => collection_into(ink, cx, view, &mut st, &opts, rows, find, row),
                Arm::Defective => {
                    coll_defective::whole_content(ink, cx, view, &mut st, &opts, rows, find, row)
                }
            };
        }
        iterated
    })
}

/// `table` — cell-drawer calls in one frame over `n` rows and [`COLUMNS`] columns.
fn table_run(arm: Arm, n: u64) -> Reading {
    let cols: Vec<Column> = crate::grid::columns(COLUMNS);
    let rows = Rows::of(usize::try_from(n).unwrap_or(usize::MAX));
    drawn(move |cx, ink| {
        let view = cx.area();
        let opts = TableOpts::default();
        let mut st = TableState::new();
        let find = |_: &str, _: Range<usize>| None;
        let mut iterated = 0u64;
        {
            let cell = |ink: &mut Direct, cx: &mut Ctx<'_, '_>, r: Rect, _c, _f| {
                iterated += 1;
                write_row(ink, cx, r, r.w);
            };
            let _ = match arm {
                Arm::Shipped => table_into(ink, cx, view, &mut st, &opts, &cols, rows, find, cell),
                Arm::Defective => coll_defective::table_whole_content(
                    ink, cx, view, &mut st, &opts, &cols, rows, find, cell,
                ),
            };
        }
        iterated
    })
}

/// `tree` — row-drawer calls in one frame over an `n`-node flatten index.
///
/// **The index is built outside the frame**, which is what materialising is: spec §10 prices doing
/// it inside one at 211 frame budgets, and what O6 is asking about here is the row loop.
fn tree_run(arm: Arm, n: u64) -> Reading {
    let index = Order::built(
        (0..usize::try_from(n).unwrap_or(usize::MAX))
            .map(|i| Entry::of(u32::try_from(i).unwrap_or(u32::MAX)).at_depth(1))
            .collect(),
    );
    drawn(move |cx, ink| {
        let view = cx.area();
        let opts = TreeOpts::default();
        let mut st = TreeState::new();
        let find = |_: &str, _: Range<usize>| None;
        let mut iterated = 0u64;
        {
            let row = |ink: &mut Direct, cx: &mut Ctx<'_, '_>, r: Rect, _n: Node, _f| {
                iterated += 1;
                if r.w > 0 {
                    write_row(ink, cx, r, r.w);
                }
            };
            let _ = match arm {
                Arm::Shipped => tree_into(ink, cx, view, &mut st, &opts, &index, find, row),
                Arm::Defective => coll_defective::tree_whole_content(
                    ink, cx, view, &mut st, &opts, &index, find, row,
                ),
            };
        }
        iterated
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::obligations::{VOLUME_MEASURED, o6};

    /// **The population is derived, and [`COVERED`] is exactly it, in the freeze's order.**
    ///
    /// Written the other way round — a hand-written list of seven with a query that reads it — the
    /// two would be one expression and the equality would hold for ever. This is
    /// [`crate::obligations`]'s own arrangement: the query's population comes off `INVENTORY` and
    /// the evidence is written out beside it.
    #[test]
    fn the_covered_rows_are_the_derived_population_in_the_freezes_order() {
        let ids: Vec<&str> = COVERED.iter().map(|c| c.id).collect();
        assert_eq!(
            ids,
            population(),
            "a row that takes a volume has no measurement, or a measurement names a row that does \
             not take one"
        );
        assert_eq!(COVERED.len(), 7);
        for row in COVERED {
            assert!(
                INVENTORY.iter().any(|c| c.id == row.id),
                "{} is not a row of the freeze",
                row.id
            );
        }
    }

    /// **The derivation reaches a row ticket 44's own parenthesis does not name, and that is the
    /// criterion working.**
    ///
    /// The ticket says *`Layer::L2`, plus the rows that fold on the edit (`chart`, `plot`)* — six.
    /// The derivation answers seven: `sparkline` holds `PlotState`'s two folded memos as well,
    /// because it is `chart`'s body with the chrome deleted (components 34) and it folds a million
    /// points through the same `Raster`. *Derived, so a component added later joins without anyone
    /// remembering* — and `sparkline` shipped ten tickets after the sentence that missed it.
    #[test]
    fn the_derivation_reaches_sparkline_and_a_written_out_six_would_not_have() {
        let ticket = ["field", "collection", "table", "tree", "chart", "plot"];
        let derived = population();
        for id in ticket {
            assert!(
                derived.contains(&id),
                "the ticket names {id} and the derivation misses it"
            );
        }
        assert_eq!(
            derived
                .iter()
                .filter(|id| !ticket.contains(id))
                .collect::<Vec<_>>(),
            vec![&"sparkline"],
        );
        // And it is reached by the memo half rather than by the layer half, which is the join that
        // did the work: `sparkline` is `Layer::L1`.
        let row = INVENTORY
            .iter()
            .find(|c| c.id == "sparkline")
            .expect("a row");
        assert_ne!(row.layer, Layer::L2);
        assert!(
            MEMOS
                .iter()
                .any(|m| m.spelling == Spelling::Folded && m.holders.contains(&"sparkline"))
        );
    }

    /// **A virtualised row's ceiling has no `n` in it at all**, which is
    /// [`Layer::L2`](crate::inventory::Layer::L2)'s own sentence as arithmetic rather than as a
    /// rounding of it.
    ///
    /// The `iff` matters in both directions: a folding row with a slope of zero would be a row
    /// claiming its fold is free, and a virtualised row with a slope at all is a row that has
    /// quietly conceded the volume.
    #[test]
    fn a_virtualised_rows_ceiling_has_no_n_in_it_and_a_folding_rows_does() {
        for row in COVERED {
            assert_eq!(
                row.why == Why::Virtualised,
                row.per_input == 0.0,
                "{} is {} and its slope is {}",
                row.id,
                row.why.word(),
                row.per_input
            );
        }
        assert_eq!(
            COVERED.iter().filter(|r| r.why == Why::Virtualised).count(),
            4
        );
    }

    /// **Every volume set is three volumes a decade apart**, because [`GROWTH`] is a ratio
    /// between neighbours: over volumes that were not a decade apart the relation would be gating
    /// the spacing rather than the growth.
    #[test]
    fn every_volume_set_is_a_decade_apart() {
        for row in COVERED {
            for set in [row.volumes, row.defect_volumes] {
                assert_eq!(set[1], set[0] * 10, "{}", row.id);
                assert_eq!(set[2], set[1] * 10, "{}", row.id);
            }
            assert_eq!(
                row.volumes, VOLUMES,
                "{} is not measured where O6 is stated",
                row.id
            );
        }
    }

    /// **The seven shipped arms, with their exact counts.**
    ///
    /// Exact, because every one of them is a count and a count is the same number on every machine
    /// — the runtime's scene 19's whole reason for counting steps. Each is derivable, which is what
    /// makes it a gate rather than a snapshot:
    ///
    /// - `field` is [`FIELD_W`]: a `Ruler` row is exactly one window, so the boundary the caret
    ///   walks back to is exactly eighty clusters behind it, **at every volume**;
    /// - `collection` and `tree` are [`H`]: one drawer call a visible row;
    /// - `table` is `80 x 9` — the nine columns of twelve that three hundred columns admit;
    /// - `chart` is `n + 9 600`, the second term being the bars painted into 60 sub-columns of 160
    ///   sub-rows;
    /// - `plot` is `2n` less the points that fall outside the padded domain;
    /// - `sparkline` is `n + 800`, the same fold at the strip's geometry.
    #[test]
    fn the_seven_shipped_arms_cost_what_they_cost() {
        let at = |id: &str| {
            let row = covered(id).expect("a covered row");
            measure(row, Arm::Shipped).work
        };
        assert_eq!(at("field"), [80, 80, 80]);
        assert_eq!(at("collection"), [80, 80, 80]);
        assert_eq!(at("table"), [720, 720, 720]);
        assert_eq!(at("tree"), [80, 80, 80]);
        assert_eq!(at("chart"), [19_600, 109_600, 1_009_600]);
        assert_eq!(at("plot"), [19_896, 198_969, 1_989_690]);
        assert_eq!(at("sparkline"), [10_800, 100_800, 1_000_800]);
    }

    /// **Every covered row answers O6**, on both criteria, at a million inputs.
    #[test]
    fn every_covered_row_holds_both_bounds_at_a_million() {
        for row in COVERED {
            let m = measure(row, Arm::Shipped);
            assert!(m.holds(row), "{}", m.why_not(row).unwrap_or_default());
        }
        assert_eq!(met().len(), COVERED.len());
        assert_eq!(met(), VOLUME_MEASURED.to_vec());
    }

    /// **Every covered row is watched failing, and the failure names a criterion.**
    ///
    /// Six of the seven arms are §21's *the instrument separates a correct build from a defective
    /// one*; the seventh — `chart`'s — is the defect that actually shipped.
    #[test]
    fn every_covered_row_is_watched_failing_on_a_deliberate_defect() {
        for row in COVERED {
            let m = measure(row, Arm::Defective);
            assert!(
                !m.holds(row),
                "{}'s defective arm answers O6, so the gate is a claim about a defect nobody has \
                 expressed: {}",
                row.id,
                row.defect
            );
            let why = m.why_not(row).expect("an unmet arm says what is wrong");
            assert!(why.contains("ceiling") || why.contains("grows"), "{why}");
        }
    }

    /// **Neither criterion would do on its own, measured rather than argued.**
    ///
    /// The defect that *actually shipped* — the whole column prefix painted once a point — is
    /// `O(subh)` inside a visit, which is `O(n)` with `subh` in front of it. **Its growth relation
    /// is the shipped fold's**, ten a decade, so a relation alone reads it as healthy; what
    /// disqualifies it is the per-input ceiling, by twenty-five times.
    ///
    /// And a ceiling alone is met by any constant chosen large enough, which is §21's first
    /// refinement by name — *a threshold on the wrong side of the question is not a weak gate, it is
    /// a green one*. So the second arm is quadratic: under a ceiling raised until it admits the
    /// arm, the relation still refuses it at a hundred a decade.
    #[test]
    fn neither_criterion_would_do_on_its_own() {
        let chart = covered("chart").expect("a covered row");
        let shipped_defect = measure(chart, Arm::Defective);
        assert!(
            shipped_defect.within_growth(),
            "the defect that shipped grows by {:?} a decade, and a relation alone calls that \
             healthy",
            shipped_defect.growth()
        );
        assert!(!shipped_defect.within_ceiling(chart));

        let plot = covered("plot").expect("a covered row");
        let quadratic = measure(plot, Arm::Defective);
        // The ceiling raised until it admits the arm, which is what *any constant chosen large
        // enough* means. The relation is unmoved, because a relation has no constant in it.
        let generous = Covered {
            fixed: u32::MAX as u64,
            ..*plot
        };
        assert!(quadratic.within_ceiling(&generous));
        assert!(!quadratic.within_growth());
        assert!(
            quadratic.growth().iter().all(|g| *g > 90.0),
            "{:?}",
            quadratic.growth()
        );
    }

    /// **An arm that did no work has not been measured**, whatever the two bounds say about it.
    ///
    /// A ratio between two zeroes is not a growth of one and a zero is under every ceiling, so the
    /// refusal is before either — [`crate::obligations::Verdict::of`]'s vacuity refusal one file
    /// over, on a measurement rather than on a population.
    #[test]
    fn an_arm_that_did_no_work_does_not_hold() {
        let row = covered("collection").expect("a covered row");
        let nothing = Measured {
            id: "collection",
            arm: Arm::Shipped,
            volumes: VOLUMES,
            work: [0, 0, 0],
        };
        assert!(nothing.within_ceiling(row));
        assert!(nothing.within_growth());
        assert!(!nothing.holds(row));
        assert!(
            nothing
                .why_not(row)
                .expect("a reason")
                .contains("did no work")
        );
    }

    /// **O6 is met over the seven**, and it is the query rather than this file that says so.
    #[test]
    fn o6_is_met_over_the_seven() {
        assert_eq!(
            o6(&met()),
            crate::obligations::Verdict::Met { over: 7 },
            "O6"
        );
    }
}
