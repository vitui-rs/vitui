//! **The scroll area's screens: the bar fixpoint, `Σ h` as the extent, and two areas far apart.**
//!
//! Components ticket 18. Spec §9, §21, ADR 0029. This is what `scroll_area`, `scrollbar` and
//! `sticky` are scenes *of*, and it exists for [`crate::listing`]'s reason one family over: **each
//! of the four hostile axes was established by a defect that passed every gate then in force and
//! looked healthier than the correct build.**
//!
//! | scene | what it decides |
//! |---|---|
//! | 5 475 600 viewport x extent pairs | the bar fixpoint: **0 failures, at most 3 passes** — and the spelling a reader writes has hysteresis instead |
//! | a 1M-row scroll area, one row in eight three cells tall | `Σ h` is the extent: **row 799 999 of 999 999**, and **no counter separates the two builds** |
//! | two scroll areas far apart with overlay bars | a bar over a drawing body: **[`OVERLAY_REDAMAGE`] cells against 0** |
//! | a `scroll_area` over unbounded data | the wrong pairing: **[`PAIRING_ROWS`] rows iterated against [`AH`]** |
//!
//! # The three numbers this screen refuses to reproduce, and why each one matters
//!
//! §21's rule is that a figure is asserted **as measured** and the difference explained. Three of
//! §9's are measured here and none of the three comes out at the remembered value. They are not
//! three sloppy figures: each disagreement says something about the mechanism.
//!
//! **1. §9's `3 535` is a damage figure and not a double-write count.** ADR 0029 states both
//! numbers in the same document and they are about different quantities — *3 535 cells a frame
//! against 0 for the reserved twin*, and then, three paragraphs down, *the amplifier is an engine
//! fact: damage is one span per surface row … ×8.4 where two bars contest ~423 cells.* `423 × 8.36`
//! **is** 3 535. So the double write is the ~423, and the 3 535 is what the span model made of it.
//! The engine measured that model and **rejected it**: `crates/vitui-engine/src/damage.rs` ships a
//! per-row **bitset** whose own table prices `RowSpans` at 2.53× on three dialogs standing apart and
//! 37.07× on a sub-cell chart, against `RowBits` at **1.00× everywhere** — *exact by construction —
//! it can never report a cell the frame did not change.* This screen measures the double write,
//! which is [`OVERLAY_REDAMAGE`], and reports both models beside it ([`Damage`]).
//!
//! **2. The thumb is out by more than 7 of 69, and 7 is the half-range figure.** §9 says *up to 7
//! cells of 69*. The drift is `travel · offset · (1/furthest_rows − 1/furthest_cells)`, which is
//! **linear in the offset**: it reaches 7 at the middle of the reachable range and
//! [`THUMB_DRIFT`] at its end. `up to` is the phrase that does not survive, not the arithmetic —
//! see [`thumb_drift`].
//!
//! **3. The wrong pairing does not cost 7 907 µs here, and the count it is diagnosed by does not
//! move either.** §9 prices it at 100 000 rows on a screen this ticket does not own; what reproduces
//! exactly is the *mechanism* — [`PAIRING_ROWS`] rows iterated against [`AH`] — which is
//! `crates/vitui-runtime/src/scroll.rs`'s own rule: **count what each body iterated rather than
//! timing it, because the count is the mechanism and the timing is the weather.** The µs figure is
//! printed by `examples/area_numbers.rs` beside §9's, and neither is a gate.
//!
//! # Four instruments, because the three scenes ask three different questions
//!
//! 1. **The fixpoint is arithmetic and takes no `Ctx`.** ADR 0002: the engine lays nothing out, so
//!    the question is *who computes the reduced rectangle and when*, and the answer is *the
//!    component, before it calls the body, from integers the caller already owns.* [`decide`] is a
//!    sizing function in `CONTEXT.md`'s sense — nothing in section 1 draws.
//! 2. **The extent scene draws a real frame**, measured with a [`Tally`] and the frame's own hit
//!    index. It cannot go through [`Pen`], for [`crate::listing`]'s reason: a scrolled context is a
//!    content coordinate system, so a body drawn at content coordinates writes far outside a
//!    [`Canvas`] sixty-nine rows tall. It also cannot go through [`Ctx::scroll_scope`] yet, and
//!    that is the fourth finding — see [`SCROLL_SCOPE_TRANSLATES_THE_WRONG_WAY`].
//! 3. **The two-areas scene goes through [`Pen`]**, because re-damage is a relation between two
//!    frames and only a surface that survives one can hold it — `Pen::over`, ticket 07's
//!    arrangement, and the reason the arms are played over four frames rather than one.
//! 4. **The wrong pairing goes through both**, because its gate is a count and its figure is a
//!    timing, and §20's rule keeps those apart.
//!
//! # The screen is red, and it is red for one reason rather than two
//!
//! [`crate::listing`] keeps two reasons apart because one of its five scenes is pinned on a defect
//! rather than on a missing subject. **All four of these are waiting for their subject**:
//! `scroll_area`, `scrollbar` and `sticky` are undeclared, so what stands on this screen is a
//! stand-in row loop and two rectangles. [`standing`] is the verdict, [`subjects_declared`] opens
//! the file the freeze homes the three in, and [`owed_message`] is the sentence that separates
//! *waiting for its subject* from *the code is wrong*. Inverted by **components 19**.
//!
//! **Ticket 20's wheel gate is not asked here and that is deliberate.** §9's watermark line — *the
//! area is dead downward, twenty wheel clicks move the offset 0, and alive sideways* — is the same
//! defect [`crate::listing`] pins as scene 6, and a second copy of a pinned gate is a second thing
//! to invert.
//!
//! [`Canvas`]: crate::runner::Canvas
//! [`Ctx::scroll_scope`]: vitui_runtime::Ctx::scroll_scope
//! [`Pen`]: crate::runner::Pen
//! [`Tally`]: crate::counters::Tally

use std::time::{Duration, Instant};

use vitui_runtime::{Ctx, Density, Id, Interest, Rect, Role, Scrollable};

use crate::counters::{Counter, Tally};
use crate::ink::{Direct, Ink};
use crate::obligations::Verdict;
use crate::runner::{Canvas, Pen, driver_at};
use crate::scroll::{self, Orient, Span};

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// 1. The bar decision — scene 17
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// Whether a bar takes room from the body or floats over it.
///
/// **[`Bars::Overlay`] is a negative case and never an option a component offers** (ADR 0029, and
/// components ticket 19's first criterion in as many words). It exists here so the rule can be
/// measured rather than repeated: an overlay bar is free exactly where the body does not draw under
/// it, which is not a property any component can guarantee of its body.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bars {
    /// The bar gets a rectangle of its own, cut off the viewport **before** the body is called.
    Reserved,
    /// The bar is drawn over the body's last column and last row, after the body has drawn.
    Overlay,
}

/// How the *presence* of a bar is decided.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hide {
    /// The gutter is always cut, whether or not there is anything to scroll. No decision, no loop.
    Never,
    /// **The answer.** Decide from the *bar-free* rectangle every frame, and iterate to a fixpoint.
    Fixpoint,
    /// The shape a reader writes: decide from *last frame's reduced* rectangle. Measured because it
    /// is the one with hysteresis.
    Incremental,
}

impl Hide {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Hide::Never => "never",
            Hide::Fixpoint => "fixpoint",
            Hide::Incremental => "incremental",
        }
    }
}

/// The pair of decisions.
///
/// `v` is the vertical bar, which takes a **column**; `h` is the horizontal bar, which takes a
/// **row**. The axis a bar *reports* and the axis it *costs* are perpendicular, which is the whole
/// reason the two decisions are coupled at all and the whole reason the fixpoint exists.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Shown {
    /// The vertical bar, down the right-hand edge.
    pub v: bool,
    /// The horizontal bar, along the bottom edge.
    pub h: bool,
}

/// One decision, with the evidence the gate needs beside it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Decision {
    /// What is shown.
    pub shown: Shown,
    /// **How many passes the loop took, including the one that changed nothing.** §9's gate is
    /// *worst case 3 passes*, and the third is the pass that proves it is a fixpoint.
    pub passes: u8,
}

/// The largest number of passes [`decide`] is allowed to take. §9: **worst case 3 passes.**
pub const MAX_PASSES: u8 = 3;

/// The viewport widths the sweep visits: `1..40`.
pub const SWEEP_VIEWPORTS: u16 = 39;
/// The extents the sweep visits on each axis: `0..60`.
pub const SWEEP_EXTENTS: u32 = 60;
/// **The domain, and it is §9's own figure**: `39 · 39 · 60 · 60`.
pub const SWEEP_PAIRS: u64 = (SWEEP_VIEWPORTS as u64)
    * (SWEEP_VIEWPORTS as u64)
    * (SWEEP_EXTENTS as u64)
    * (SWEEP_EXTENTS as u64);

/// **The fixpoint, computed from the bar-free rectangle.**
///
/// The termination argument is one sentence and it is worth writing down because it is the whole
/// answer to *can this oscillate*: **reserving is monotone.** A bar only ever *removes* room, and a
/// smaller viewport can only make an overflow more likely, so the iteration moves each of two
/// booleans from `false` to `true` and never back — two monotone booleans, therefore one least
/// fixpoint and at most two changes plus the pass that proves it.
///
/// The `|` below is the mechanism and not a convenience: written as `=` the loop is no longer
/// monotone by construction and the argument above becomes a claim about the reasoning rather than
/// about the code.
///
/// The precondition that makes the monotonicity real is a property of the **body**, not of this
/// function: *a smaller viewport may not produce a smaller extent.* [`oscillates`] is the case
/// where a body breaks it, and it is not hypothetical — a list that stacks two fields per row when
/// it is narrow breaks it on purpose.
pub fn decide(free: (u16, u16), extent: (u32, u32), bars: Bars) -> Decision {
    let mut shown = Shown::default();
    let mut passes = 0u8;
    loop {
        let (w, h) = reduced(free, shown, bars);
        let next = Shown {
            v: shown.v || extent.1 > u32::from(h),
            h: shown.h || extent.0 > u32::from(w),
        };
        passes = passes.saturating_add(1);
        if next == shown {
            return Decision { shown, passes };
        }
        shown = next;
        // Monotone, so this is unreachable at three; it is here so that a future edit that breaks
        // the monotonicity returns a `passes` the gate can fail on rather than hanging.
        if passes >= 8 {
            return Decision { shown, passes };
        }
    }
}

/// **The spelling a reader writes, and it has hysteresis.**
///
/// One pass, from *last frame's* reduced rectangle. It is not wrong on any single frame — it is
/// wrong about **history**: once both bars are up, the rectangle they left behind keeps them up over
/// content that would fit with neither. There is no oscillation and no non-termination; the bars
/// simply never come back down, and nothing in a frame's counts moves when they should have.
///
/// This is the second half of ADR 0029: **reserved auto-hiding bars require a declared content
/// size**, because a measured extent is taken inside the rectangle the decision produced.
pub fn decide_incremental(
    free: (u16, u16),
    extent: (u32, u32),
    prev: Shown,
    bars: Bars,
) -> Decision {
    let (w, h) = reduced(free, prev, bars);
    Decision {
        shown: Shown {
            v: extent.1 > u32::from(h),
            h: extent.0 > u32::from(w),
        },
        passes: 1,
    }
}

/// The viewport left after a decision. An overlay bar reserves nothing, which is why it cannot
/// oscillate and why it is not the answer.
pub fn reduced(free: (u16, u16), shown: Shown, bars: Bars) -> (u16, u16) {
    match bars {
        Bars::Overlay => free,
        Bars::Reserved => (
            free.0.saturating_sub(u16::from(shown.v)),
            free.1.saturating_sub(u16::from(shown.h)),
        ),
    }
}

/// What the sweep found. A value rather than three assertions, so `examples/area_numbers.rs` prints
/// the same numbers the gate reads.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sweep {
    /// How many `(viewport, extent)` pairs were visited. [`SWEEP_PAIRS`].
    pub pairs: u64,
    /// **How many answers were not fixpoints.** §9: **0**.
    pub failures: u64,
    /// **The worst pass count over the whole domain.** §9: **3**.
    pub worst_passes: u8,
    /// How many of the pairs showed a bar that the reduced rectangle did not need. The *least*
    /// fixpoint half, which the round-trip check alone cannot see.
    pub unneeded: u64,
}

/// **Feed each answer back into itself, over every one of [`SWEEP_PAIRS`] pairs.**
///
/// The check is the definition: the viewport a decision leaves must ask for exactly the bars that
/// decision already shows. A sample would answer a different question — every interesting case here
/// is a place the arithmetic runs out, and they are all at boundaries.
pub fn sweep() -> Sweep {
    let mut out = Sweep {
        pairs: 0,
        failures: 0,
        worst_passes: 0,
        unneeded: 0,
    };
    for w in 1..=SWEEP_VIEWPORTS {
        for h in 1..=SWEEP_VIEWPORTS {
            for ex in 0..SWEEP_EXTENTS {
                for ey in 0..SWEEP_EXTENTS {
                    let free = (w, h);
                    let decision = decide(free, (ex, ey), Bars::Reserved);
                    let (rw, rh) = reduced(free, decision.shown, Bars::Reserved);
                    let again = Shown {
                        v: ey > u32::from(rh),
                        h: ex > u32::from(rw),
                    };
                    out.pairs += 1;
                    if again != decision.shown {
                        out.failures += 1;
                    }
                    if (decision.shown.v && !again.v) || (decision.shown.h && !again.h) {
                        out.unneeded += 1;
                    }
                    out.worst_passes = out.worst_passes.max(decision.passes);
                }
            }
        }
    }
    out
}

/// **The body that breaks the precondition**, kept as a value so the reflow loop is a measurement.
///
/// A responsive body: `tall` cells of extent while it has at least `wide` columns, `short` below
/// that. Not a contrivance — a list that drops a column and stacks two fields per row when it is
/// narrow is the commonest responsive behaviour there is, and a chart that drops its legend is
/// another. Its extent is **not antitone in the viewport**, so [`decide`]'s monotonicity argument
/// does not apply to it and the fixpoint is taken over a function that has none.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Responsive {
    /// The width at or above which the body lays out tall.
    pub wide: u16,
    /// The extent it has when it is wide enough.
    pub tall: u32,
    /// The extent it has when it is not. **Below `tall` is what breaks the precondition.**
    pub short: u32,
    /// The horizontal extent, which does not depend on the viewport.
    pub natural_w: u32,
}

impl Responsive {
    /// The extent this body reports in a viewport `viewport_w` columns wide.
    pub fn extent(&self, viewport_w: u16) -> (u32, u32) {
        (
            self.natural_w,
            if viewport_w >= self.wide {
                self.tall
            } else {
                self.short
            },
        )
    }
}

/// **The reflow loop**: run the fixpoint against a responsive body, frame after frame.
///
/// The extent is measured at the width the *previous* frame left, which is what a watermark is.
/// Declared content has no such dependency, which is exactly the difference ADR 0029's second
/// sentence is about.
pub fn oscillates(free: (u16, u16), body: Responsive, frames: usize, bars: Bars) -> Vec<Shown> {
    let mut out = Vec::with_capacity(frames);
    let mut w = free.0;
    for _ in 0..frames {
        let extent = body.extent(w);
        let decision = decide(free, extent, bars);
        w = reduced(free, decision.shown, bars).0;
        out.push(decision.shown);
    }
    out
}

/// How many times the decision changed across a run. **Zero on a steady frame** is the whole check.
pub fn flips(seq: &[Shown]) -> usize {
    seq.windows(2).filter(|w| w[0] != w[1]).count()
}

/// How many frames the reflow loop is run for. §9: **99 flips in 99 frames**, which is 100 frames
/// and the 99 windows between them.
pub const REFLOW_FRAMES: usize = 100;
/// §9's own figure: **the decision flips 99 times in 99 frames with no input.**
pub const REFLOW_FLIPS: usize = REFLOW_FRAMES - 1;

// ── the hysteresis screen ────────────────────────────────────────────────────────────────────────

/// The width of the screen the hysteresis is visible on.
pub const FIT_W: u16 = 20;
/// Its height. Twenty rows of content in a twenty-row rectangle: it fits with **no bars at all**,
/// which is the only shape the two spellings disagree about.
pub const FIT_H: u16 = 20;

/// What one frame of the fitting screen turned out to be.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Fitting {
    /// The bars the spelling decided to show.
    pub shown: Shown,
    /// **How many of the twenty content rows the body could draw.**
    pub rows: u16,
    /// How many columns of each row it could draw.
    pub cols: u16,
    /// Whether the last content row reached the screen at all.
    pub last_row_drawn: bool,
}

/// **Content that fits with no bars, decided both ways.**
///
/// The whole of the hysteresis, as a shape rather than as a sentence: the fixpoint shows neither bar
/// and draws all twenty rows; the incremental spelling, started from a frame where both bars were
/// up, keeps both and draws nineteen rows of nineteen columns — **for ever**, on a screen that looks
/// entirely correct.
pub fn fitting(hide: Hide) -> Fitting {
    let free = (FIT_W, FIT_H);
    let extent = (u32::from(FIT_W), u32::from(FIT_H));
    let shown = match hide {
        Hide::Never => Shown { v: true, h: true },
        Hide::Fixpoint => decide(free, extent, Bars::Reserved).shown,
        // Started from the frame before, in which the content was longer and both bars were up.
        // That is the only input the incremental spelling has, and it is the whole defect.
        Hide::Incremental => {
            decide_incremental(free, extent, Shown { v: true, h: true }, Bars::Reserved).shown
        }
    };
    let (cols, rows) = reduced(free, shown, Bars::Reserved);
    Fitting {
        shown,
        rows,
        cols,
        last_row_drawn: rows >= FIT_H,
    }
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// 2. `Σ h` is the extent — scene 18
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// The area's width. **§9's own watermark line**: the watermark reads `(400, 69)` against a
/// `(246, 69)` viewport.
pub const AW: u16 = 246;
/// The area's height, in content cells. **Sixty-nine**, which is where *7 cells of 69* comes from
/// as well: the bar's track is the viewport.
pub const AH: u16 = 69;

/// How many rows the content holds. §9's own screen.
pub const ROWS: u64 = 1_000_000;
/// One row in this many is taller than one cell. C05's own mix, kept so the two prototypes'
/// numbers stay comparable.
pub const TALL_EVERY: u64 = 8;
/// How tall a tall row is, in cells.
pub const TALL_H: u32 = 3;

/// **The extent, in content cells: `Σ h`.** One row in eight three cells tall over a million rows.
pub const EXTENT_CELLS: u32 = 1_250_000;
/// **The extent a reader writes first**, and the defect §9 had to price: the row count, used as if
/// it were a length in cells.
pub const EXTENT_ROWS: u32 = 1_000_000;

/// The last content row a correct build can reach. **999 999 — the last one.**
pub const LAST_ROW_IN_CELLS: u64 = 999_999;
/// **The last content row the row-measured spelling can reach: 799 999.** Twenty per cent of the
/// content is unreachable, on a screen that looks perfectly healthy.
pub const LAST_ROW_IN_ROWS: u64 = 799_999;

/// **`Ctx::scroll_scope` translates the content the wrong way, and this screen works around it.**
///
/// Filed as runtime architecture issue 26. At offset 5 in a six-row viewport the scope reports
/// `visible_rows() == -5..1` and a write at content row 5 lands **0 cells** while one at content
/// row 0 lands five: `Ctx::scroll_scope` hands `+offset` to `View::scrolled`, and the engine's
/// documented convention is the other sign — *a viewport scrolled `n` rows down is
/// `scrolled(0, -n)`* (`crates/vitui-engine/src/view.rs`).
///
/// **It is invisible at offset 0**, which is every offset the runtime's own tests and
/// [`crate::listing`]'s three volumes use, and it is why this ticket is the first thing in the
/// workspace to meet it: nothing above the runtime had scrolled a `scroll_scope` yet.
///
/// Until it is settled [`draw_into`] applies the offset itself, inside a `Ctx::child` of the same
/// rectangle — which is what a working scope would have produced. **The substitution is on both
/// arms**, so it is on the side of neither, and what separates them is the only thing under test:
/// which unit the extent was measured in.
/// `tests::the_runtime_still_translates_a_scrolled_scope_the_wrong_way` is a live reproduction, so
/// the day the runtime is fixed this fails and the workaround comes out rather than staying for
/// ever.
pub const SCROLL_SCOPE_TRANSLATES_THE_WRONG_WAY: &str = "runtime architecture issue 26";

/// The offsets the equality is swept over. Every one of them is reachable in **both** builds, which
/// is what makes the comparison a comparison: at an offset only one of them admits, the two frames
/// differ because the content differs and the unit error is invisible under it.
pub const SWEPT_OFFSETS: [i32; 8] = [0, 1, 7, 999, 100_000, 500_000, 999_930, 999_931];

/// One content row: a height in cells.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Row {
    /// How many cells tall it is.
    pub h: u8,
}

/// **The caller's content index**, which is where an extent lives (§9, ADR 0002).
///
/// `ytop` is the prefix sum of the heights, `rows + 1` entries, built **only** when a row can be
/// taller than one cell — C05's rule, kept: it is four bytes a row and a uniform list does not pay
/// them.
#[derive(Clone, Debug)]
pub struct Content {
    rows: Vec<Row>,
    ytop: Vec<u32>,
    variable: bool,
}

impl Content {
    /// `n` rows. `variable` makes one row in [`TALL_EVERY`] [`TALL_H`] cells tall.
    pub fn build(n: u64, variable: bool) -> Content {
        let rows: Vec<Row> = (0..n)
            .map(|i| Row {
                h: if variable && i % TALL_EVERY == 0 {
                    TALL_H as u8
                } else {
                    1
                },
            })
            .collect();
        let mut ytop = Vec::new();
        if variable {
            ytop.reserve_exact(rows.len() + 1);
            let mut acc = 0u32;
            ytop.push(0);
            for row in &rows {
                acc += u32::from(row.h);
                ytop.push(acc);
            }
        }
        Content {
            rows,
            ytop,
            variable,
        }
    }

    /// How many rows it holds.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether it holds no rows at all.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// **The extent, in content cells: `Σ h`.** The row count exactly when every row is one tall.
    pub fn extent_cells(&self) -> u32 {
        if self.variable {
            self.ytop.last().copied().unwrap_or(0)
        } else {
            u32::try_from(self.rows.len()).unwrap_or(u32::MAX)
        }
    }

    /// **The extent measured in rows** — the unit error, as a function so it can be handed to the
    /// same code the correct one is.
    pub fn extent_rows(&self) -> u32 {
        u32::try_from(self.rows.len()).unwrap_or(u32::MAX)
    }

    /// The extent under a [`Unit`]. **The one place the two builds differ.**
    pub fn extent(&self, unit: Unit) -> u32 {
        match unit {
            Unit::Cells => self.extent_cells(),
            Unit::Rows => self.extent_rows(),
        }
    }

    /// The top of row `i`, in content cells.
    pub fn top_of(&self, i: usize) -> u32 {
        if self.variable {
            self.ytop[i.min(self.ytop.len() - 1)]
        } else {
            u32::try_from(i).unwrap_or(u32::MAX)
        }
    }

    /// **The first row whose span contains content cell `y`.** A binary search, once a frame.
    ///
    /// Uniform rows make it arithmetic; the point of keeping both behind one name is that the
    /// caller cannot tell which it got, so the `O(log n)` is not a second code path in the
    /// component.
    pub fn row_at(&self, y: u32) -> usize {
        if self.rows.is_empty() {
            return 0;
        }
        if !self.variable {
            return (y as usize).min(self.rows.len() - 1);
        }
        self.ytop
            .partition_point(|&top| top <= y)
            .saturating_sub(1)
            .min(self.rows.len() - 1)
    }

    /// The half-open row range that intersects the cell window `[y, y + h)`. **Once a frame.**
    pub fn rows_in(&self, y: u32, h: u16) -> (usize, usize) {
        if self.rows.is_empty() || h == 0 {
            return (0, 0);
        }
        let lo = self.row_at(y);
        let end = y.saturating_add(u32::from(h));
        let hi = if self.variable {
            self.ytop.partition_point(|&top| top < end)
        } else {
            (end as usize).min(self.rows.len())
        };
        (lo, hi.clamp(lo + 1, self.rows.len()))
    }
}

/// **Which unit the extent is measured in.** The whole of the defect, as one enum.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unit {
    /// `Σ h`, in content cells. **The answer** (§9, ADR 0029).
    Cells,
    /// The row count, used as if it were a length in cells.
    Rows,
}

impl Unit {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Unit::Cells => "cells",
            Unit::Rows => "rows",
        }
    }
}

/// **What one frame of the scroll area turned out to be**, returned rather than printed.
///
/// [`crate::listing::Shape`]'s arrangement and its reason: a number only a report prints is a number
/// no gate can read.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Shape {
    /// Columns the engine reported written.
    pub writes: u64,
    /// Distinct cells touched.
    pub distinct: u64,
    /// Drawing calls made.
    pub verbs: u64,
    /// Entries in the runtime's hit index.
    pub regions: usize,
    /// Entries in the focus ring.
    pub stops: usize,
    /// **Rows the body actually iterated.** The mechanism, where the three counters above are the
    /// symptom.
    pub iterated: u64,
    /// The offset the runtime clamped to. **The one field the unit error moves**, and it moves it
    /// only at the end of the content.
    pub offset: i32,
    /// **The last content row the frame could reach**, which is what the scene asserts on.
    pub last_row: u64,
}

/// The largest offset a content of `extent` cells admits in an [`AH`]-cell viewport.
pub fn max_offset(extent: u32) -> i32 {
    i32::try_from(extent.saturating_sub(u32::from(AH))).unwrap_or(i32::MAX)
}

/// **Draw one frame of the scroll area**, in the unit its extent was measured in.
///
/// Generic over [`Ink`] so a [`Tally`] and a [`Direct`] measure the same drawing path rather than a
/// copy of it — [`crate::ink`]'s whole argument.
///
/// The body draws rows at their true `Σ h` positions under **both** units. That is the point: the
/// unit error is in the extent alone, so at any offset both builds can reach the frame is
/// identical, and what differs is only which offsets exist.
pub fn draw_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    content: &Content,
    unit: Unit,
    offset: i32,
) -> (u64, u64) {
    let id = Id::named("scroll_area");
    let view = cx.area();
    let extent = content.extent(unit);
    let max = (0, max_offset(extent));
    let offset = (0, offset.clamp(0, max.1));
    let body = cx.theme().paint(Role::Body);

    let _ = cx.scrollable(
        id,
        view,
        Interest::CLICK.with(Interest::FOCUS),
        Scrollable::between(offset, max),
    );

    let mut iterated = 0u64;
    let top = u32::try_from(offset.1).unwrap_or(0);
    let (lo, hi) = content.rows_in(top, AH);
    // **The one substitution, and it is on both arms.** See [`SCROLL_SCOPE_TRANSLATES_THE_WRONG_WAY`]:
    // the offset is applied here rather than by `Ctx::scroll_scope`, which is what a working scope
    // would have produced and is what the day the runtime issue is settled deletes.
    {
        let mut scope = cx.child(view);
        for i in lo..hi {
            iterated += 1;
            scope.with_key(i as u64, |cx| {
                let row = cx.id();
                let y = i32::try_from(content.top_of(i)).unwrap_or(i32::MAX)
                    - i32::try_from(top).unwrap_or(0);
                let h = u16::from(content.rows[i].h);
                let _ = cx.interact(row, Rect::new(0, y, AW, h), Interest::CLICK);
                for dy in 0..i32::from(h) {
                    let _ = ink.run(cx, 0, y + dy, "-", AW, body);
                }
            });
        }
    }
    // The last content row this frame could reach: the row holding the last cell of the window.
    let last_cell = top.saturating_add(u32::from(AH)).saturating_sub(1);
    let last_row = u64::try_from(content.row_at(last_cell.min(content.extent_cells() - 1)))
        .unwrap_or(u64::MAX);
    (iterated, last_row)
}

/// **The scroll area at one unit and one offset, as a shape.**
///
/// One warm-up frame and one measured one, because the frame structures take their allocation on the
/// first frame that needs one and keep it — a cold frame is not a frame.
pub fn frame_at(content: &Content, unit: Unit, offset: i32) -> Shape {
    let mut driver = driver_at(AW, AH, Density::default());
    let mut warm = Tally::new();
    driver.frame(|cx| {
        let _ = draw_into(&mut warm, cx, content, unit, offset);
    });

    let mut tally = Tally::new();
    let mut reach = (0u64, 0u64);
    driver.frame(|cx| reach = draw_into(&mut tally, cx, content, unit, offset));
    let frame = driver.inspect();
    Shape {
        writes: tally.writes(),
        distinct: tally.distinct(),
        verbs: tally.verbs(),
        regions: frame.hits().len(),
        stops: frame.stop_count(),
        iterated: reach.0,
        offset: offset.clamp(0, max_offset(content.extent(unit))),
        last_row: reach.1,
    }
}

/// **The frame at the furthest offset each unit admits** — the two ends of the scene.
pub fn at_the_end(content: &Content, unit: Unit) -> Shape {
    frame_at(content, unit, i32::MAX)
}

/// **Which of §20's counters separate the two builds at a common offset. It is the empty list.**
///
/// [`crate::listing::counters_that_separate_them`]'s shape, and it is the sharper half of this
/// scene: the reachability differs and *nothing a frame counts does*. `marked` is not in the walk
/// because [`crate::counters::Counters::marked`] is unreachable from this crate — which is itself
/// the point, since a build that marks less is what two of the four axes were established by.
pub fn counters_that_separate_them(offset: i32) -> Vec<Counter> {
    let content = Content::build(ROWS, true);
    let cells = frame_at(&content, Unit::Cells, offset);
    let rows = frame_at(&content, Unit::Rows, offset);
    let mut out = Vec::new();
    if cells.writes != rows.writes {
        out.push(Counter::Writes);
    }
    if cells.verbs != rows.verbs {
        out.push(Counter::Verbs);
    }
    if cells.regions != rows.regions {
        out.push(Counter::Regions);
    }
    if cells.stops != rows.stops {
        out.push(Counter::TabStops);
    }
    out
}

// ── the thumb, which is the same unit error one helper down ──────────────────────────────────────

/// **The worst drift between a thumb sized from `Σ h` and one sized from the row count**, in cells
/// of a track [`AH`] long, over the offsets the defective build can actually reach.
///
/// It is linear in the offset — `travel · offset · (1/furthest_rows − 1/furthest_cells)` — so *up
/// to 7* is the figure at the middle of the range and this is the figure at its end. See the module
/// header.
pub fn thumb_drift() -> u16 {
    let content = Content::build(ROWS, true);
    let furthest = max_offset(content.extent_rows());
    let mut worst = 0u16;
    let step = (furthest / 200).max(1);
    let mut offset = 0i32;
    while offset <= furthest {
        worst = worst.max(drift_at(&content, offset));
        offset += step;
    }
    worst.max(drift_at(&content, furthest))
}

/// **How far into the reachable range the drift first reaches `cells`**, as a fraction of it, or
/// `None` if it never does.
///
/// The instrument behind the module header's second finding. §9 states the drift as *up to 7 cells
/// of 69*; the drift is linear in the offset, so *7* is a point on the range and not its end, and
/// this is the point. Scanned rather than solved because the quantity is a difference of two floors
/// and the crossing is where the two floors part company, not where the reals do.
pub fn drift_reaches(cells: u16) -> Option<f64> {
    let content = Content::build(ROWS, true);
    let furthest = max_offset(content.extent_rows());
    let step = (furthest / 4_000).max(1);
    let mut offset = 0i32;
    while offset <= furthest {
        if drift_at(&content, offset) >= cells {
            return Some(f64::from(offset) / f64::from(furthest));
        }
        offset += step;
    }
    None
}

fn drift_at(content: &Content, offset: i32) -> u16 {
    let at = |extent: u32| {
        scroll::thumb(
            AH,
            Span {
                viewport: u32::from(AH),
                extent,
                offset: u32::try_from(offset).unwrap_or(0),
            },
        )
        .0
    };
    at(content.extent_cells()).abs_diff(at(content.extent_rows()))
}

/// §9's remembered drift: **up to 7 cells of 69**.
pub const REMEMBERED_DRIFT: u16 = 7;
/// **The drift this screen measures at the end of the reachable range.** See the module header for
/// why it is not [`REMEMBERED_DRIFT`], and `examples/area_numbers.rs` for both columns.
pub const THUMB_DRIFT: u16 = 14;
/// **Where §9's seven cells actually sit on the range**: a little short of the halfway mark, at
/// **45.6%** of the offsets the defective build can reach. Gated to a hundredth either side rather
/// than exactly, because it is a crossing between two floors and not a property of the mechanism.
pub const DRIFT_REACHES_SEVEN_AT: f64 = 0.456;

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// 3. Two scroll areas far apart, with overlay bars — scene 19
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// The screen's width. §20 prices every dense screen on this map at 300x80, and
/// [`crate::scroll::W`] already ships the constant.
pub const W: u16 = scroll::W;
/// The screen's height.
pub const H: u16 = scroll::H;
/// Each area's width. **Symmetric on purpose**: the total is then exactly twice one area's, which
/// makes the figure a property of *an area whose body draws under its bar* rather than of a layout.
pub const AREA_W: u16 = 120;
/// **How far apart the two areas are, in columns.** The word in §21's row is *far apart*, and this
/// is what it is spelled as: sixty columns of screen that neither area touches. It is also the term
/// the span model's factor grows with, which is why it is a named constant rather than a
/// subtraction — see [`Damage`].
pub const GAP: u16 = W - 2 * AREA_W;

/// The sticky header each area carries, in rows. **One**, because `sticky` has to be on this screen
/// for [`crate::scenes::scenes_for`] to answer for it, and because a scroll area with a pinned
/// header is the shape §9 describes. What this screen does **not** measure is §9's band-drawn-by-
/// arithmetic figure — that is components ticket 19's own criterion, and a second copy of a number
/// is a second thing to keep in step.
pub const BAND_H: u16 = 1;

/// How many steady frames the arms are played over. Four: one to paint the surface, and three in
/// which nothing about the content has changed.
pub const STEADY_FRAMES: u32 = 4;

/// **Cells the two areas' bars and bodies contest**, and the figure ADR 0029 is about.
///
/// `2 · (H + AREA_W − 1)`: a vertical bar of [`H`] cells and a horizontal bar of [`AREA_W`], sharing
/// their corner, per area. It is asserted from that arithmetic as well as measured, which is what
/// makes it a property of the mechanism rather than a number read off a screen.
pub const OVERLAY_REDAMAGE: u64 = 2 * (H as u64 + AREA_W as u64 - 1);

/// **What the reserved twin re-damages: nothing.**
pub const RESERVED_REDAMAGE: u64 = 0;

/// §9's and ADR 0029's remembered figure, which this screen does not reproduce and which the module
/// header explains: **3 535 is `~423 × 8.36`, the span model's picture of the double write.**
pub const REMEMBERED_REDAMAGE: u64 = 3_535;
/// ADR 0029's own count of the cells the span figure was taken over: **~423**. [`OVERLAY_REDAMAGE`]
/// is this screen's, and the two agree to within six per cent on two rectangles that are not the
/// same two rectangles.
pub const ADR_CONTESTED: u64 = 423;

/// **What a frame re-damaged, under both damage models.**
///
/// The engine ships the first and measured the second before rejecting it
/// (`crates/vitui-engine/src/damage.rs`), so the pair is the honest way to report a number §9 states
/// under the model that lost.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Damage {
    /// **Distinct cells whose value a verb changed.** The shipped engine's unit: a per-row bitset,
    /// *exact by construction — it can never report a cell the frame did not change.*
    pub cells: u64,
    /// **The same frame under one span per surface row**, from the leftmost changed cell of a row
    /// to the rightmost. The model §2 assumed and the engine rejected.
    pub span_cells: u64,
}

impl Damage {
    /// **The amplification factor: what the span model charges for what the bitset charges once.**
    ///
    /// `1.00` for the bitset against itself, by construction. On this screen the span model is far
    /// above ADR 0029's ×8.4, and that is the ADR's own sentence rather than a contradiction of it:
    /// *the factor grows with how far apart the areas are*, and [`GAP`] is sixty columns.
    pub fn amplification(self) -> f64 {
        if self.cells == 0 {
            return 1.0;
        }
        self.span_cells as f64 / self.cells as f64
    }
}

/// Where the left-hand area sits.
pub fn left() -> Rect {
    Rect::new(0, 0, AREA_W, H)
}

/// Where the right-hand area sits. [`GAP`] columns of untouched screen between them.
pub fn right() -> Rect {
    Rect::new(i32::from(AREA_W + GAP), 0, AREA_W, H)
}

/// **One scroll area: a sticky header, a body, and two bars — reserved or overlaid.**
///
/// The two arms differ in **one** value, the [`Bars`] argument, which is `crate::frame`'s private
/// `draw` and `crate::scroll`'s arrangement for its reason: the diff a reviewer would have to catch
/// is the diff the register can point at.
pub fn area_into<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, rect: Rect, bars: Bars) {
    let body_paint = cx.theme().paint(Role::Body);
    let band_paint = cx.theme().paint(Role::Dim);

    // Both bars are shown: the content is a million rows and four hundred columns, so the fixpoint
    // reaches `{ v: true, h: true }` in one pass and the decision is not what this screen is about.
    let shown = Shown { v: true, h: true };
    let (cols, rows) = reduced((rect.w, rect.h), shown, bars);

    // The band shares `x` and pins `y` to zero — §9's construction — and it is a view, so the body
    // below it cannot write into it whichever way the bars went.
    let band = Rect::new(rect.x, rect.y, cols, BAND_H);
    let body = Rect::new(
        rect.x,
        rect.y + i32::from(BAND_H),
        cols,
        rows.saturating_sub(BAND_H),
    );

    ink.run(cx, band.x, band.y, "=", band.w, band_paint);
    for dy in 0..i32::from(body.h) {
        ink.run(cx, body.x, body.y + dy, ".", body.w, body_paint);
    }

    // The bars, at the rectangle's own edges either way. Under `Reserved` the body stopped short of
    // them; under `Overlay` it did not, and that difference **is** the scene.
    let track = cx.theme().paint(Role::Border);
    let thumb = cx.theme().paint(Role::Face);
    bar_into(
        ink,
        cx,
        Rect::new(rect.right() - 1, rect.y, 1, rect.h - 1),
        Orient::Vertical,
        track,
        thumb,
    );
    bar_into(
        ink,
        cx,
        Rect::new(rect.x, rect.bottom() - 1, rect.w, 1),
        Orient::Horizontal,
        track,
        thumb,
    );
}

/// A bar, thumb first, drawn through the [`Ink`] the screen is being measured on.
///
/// It is [`crate::scroll::bar`]'s shape rather than a call to it, because `bar` draws through a
/// `Ctx` and this screen has to hand every verb to one instrument. The thumb geometry is
/// [`crate::scroll::thumb`]'s, which is the shipped arithmetic.
fn bar_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    rect: Rect,
    orient: Orient,
    track: vitui_runtime::Paint,
    thumb: vitui_runtime::Paint,
) {
    let length = match orient {
        Orient::Vertical => rect.h,
        Orient::Horizontal => rect.w,
    };
    let span = Span {
        viewport: u32::from(length),
        extent: EXTENT_CELLS,
        offset: 0,
    };
    let (start, len) = scroll::thumb(length, span);
    let stripe = |ink: &mut I, cx: &mut Ctx<'_, '_>, from: u16, n: u16, cluster: &str, paint| {
        if n == 0 {
            return;
        }
        match orient {
            Orient::Vertical => {
                for i in 0..n {
                    ink.run(
                        cx,
                        rect.x,
                        rect.y + i32::from(from + i),
                        cluster,
                        rect.w,
                        paint,
                    );
                }
            }
            Orient::Horizontal => {
                for row in 0..rect.h {
                    ink.run(
                        cx,
                        rect.x + i32::from(from),
                        rect.y + i32::from(row),
                        cluster,
                        n,
                        paint,
                    );
                }
            }
        }
    };
    stripe(ink, cx, start, len, "#", thumb);
    stripe(ink, cx, 0, start, "|", track);
    stripe(ink, cx, start + len, length - start - len, "|", track);
}

/// **Both areas, far apart, drawn one way.**
pub fn two_areas_into<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, bars: Bars) {
    area_into(ink, cx, left(), bars);
    area_into(ink, cx, right(), bars);
}

/// **Play the two-areas screen for [`STEADY_FRAMES`] frames and report the last one's damage.**
///
/// The surface persists across frames — `Pen::over` — because re-damage is a relation between two
/// frames and a fresh surface each frame reports every cell as a first paint for ever. The first
/// frame is the one that paints the screen and its damage is thrown away; what is returned is the
/// **steady** frame's, which is the frame ADR 0029 prices.
pub fn steady(bars: Bars) -> Damage {
    let mut driver = driver_at(W, H, Density::default());
    let mut canvas = Canvas::new(W, H);
    let mut damage = Damage {
        cells: 0,
        span_cells: 0,
    };
    for _ in 0..STEADY_FRAMES {
        let mut pen = Pen::over(canvas);
        driver.frame(|cx| two_areas_into(&mut pen, cx, bars));
        pen.end_frame();
        canvas = pen.into_canvas();
        damage = Damage {
            cells: canvas.repaints(),
            span_cells: canvas
                .repainted_spans()
                .iter()
                .map(|(_, lo, hi)| u64::from(hi - lo) + 1)
                .sum(),
        };
        let _ = canvas.take_repaints();
    }
    damage
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// 4. The wrong pairing — the negative case §9 states as C21
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **Which of the two scrollables the caller reached for.**
///
/// > A `scroll_area` costs its **content**; a virtualised `collection` costs its **visible window**.
/// > Every shipped scrollable of unbounded data is a `collection`. (§9)
///
/// Both compile and both look right on a thousand rows, which is what makes it *the single most
/// expensive mistake available above this runtime* (`crates/vitui-runtime/src/scroll.rs`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pairing {
    /// A `scroll_area`: the whole content, clipped. Right for a form, and the negative case here.
    Area,
    /// A virtualised `collection`: the window and nothing else.
    Collection,
}

impl Pairing {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Pairing::Area => "scroll_area",
            Pairing::Collection => "collection",
        }
    }
}

/// The volume §9 prices the wrong pairing at.
pub const PAIRING_ROWS: u64 = 100_000;
/// §9's remembered cost for it: **7 907 µs — seventy-nine budgets.** A report, never a gate.
pub const REMEMBERED_PAIRING_US: f64 = 7_907.0;

/// **One frame of the wrong pairing, as a shape.** Uniform rows, so the row count and `Σ h` agree
/// and the only thing under test is which rows the body walks.
pub fn pairing_shape(pairing: Pairing, rows: u64) -> Shape {
    let content = Content::build(rows, false);
    let mut driver = driver_at(AW, AH, Density::default());
    let mut warm = Tally::new();
    driver.frame(|cx| {
        let _ = pairing_into(&mut warm, cx, &content, pairing);
    });
    let mut tally = Tally::new();
    let mut iterated = 0u64;
    driver.frame(|cx| iterated = pairing_into(&mut tally, cx, &content, pairing));
    let frame = driver.inspect();
    Shape {
        writes: tally.writes(),
        distinct: tally.distinct(),
        verbs: tally.verbs(),
        regions: frame.hits().len(),
        stops: frame.stop_count(),
        iterated,
        offset: 0,
        last_row: 0,
    }
}

/// **What the frame costs**, drawn through [`Direct`] rather than through a [`Tally`].
///
/// Separate from [`pairing_shape`] for [`crate::listing::volume_cost`]'s reason and it is worth the
/// second function: a `Tally` keeps a set of every cell it sees, so a figure taken with one in the
/// loop is a report about the instrument. A report and never a gate.
///
/// # Panics
///
/// Panics on zero frames. A per-frame figure over no frames is a division by zero dressed as a
/// measurement.
pub fn pairing_cost(pairing: Pairing, rows: u64, frames: u32) -> Duration {
    assert!(frames > 0, "a per-frame figure needs a frame");
    let content = Content::build(rows, false);
    let mut driver = driver_at(AW, AH, Density::default());
    let mut ink = Direct;
    driver.frame(|cx| {
        let _ = pairing_into(&mut ink, cx, &content, pairing);
    });
    let started = Instant::now();
    for _ in 0..frames {
        driver.frame(|cx| {
            let _ = pairing_into(&mut ink, cx, &content, pairing);
        });
    }
    started.elapsed() / frames
}

fn pairing_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    content: &Content,
    pairing: Pairing,
) -> u64 {
    let id = Id::named("pairing");
    let view = cx.area();
    let max = (0, max_offset(content.extent_cells()));
    let body = cx.theme().paint(Role::Body);
    let _ = cx.scrollable(id, view, Interest::CLICK, Scrollable::between((0, 0), max));
    let mut iterated = 0u64;
    cx.scroll_scope(id, view, (0, 0), max, |cx| {
        let window = cx.visible_rows();
        let walk = match pairing {
            Pairing::Area => 0..i32::try_from(content.len()).unwrap_or(i32::MAX),
            Pairing::Collection => window,
        };
        for i in walk {
            iterated += 1;
            let _ = ink.run(cx, 0, i, ".", AW, body);
        }
    });
    iterated
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// The subject, and the scan that says whether it is here
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **The three components these scenes are scenes of, and none of them is declared yet.**
pub const SUBJECTS: [&str; 3] = ["scroll_area", "scrollbar", "sticky"];

/// Where [`SUBJECTS`] are declared, as `(module file, the declaration)`.
///
/// The home is the freeze's, joined through [`crate::Family`]: all three name `F3Scrolling`, whose
/// module is `scroll.rs`. A component is `fn(&mut Ctx, Rect, …) -> Response` (spec §1, rule 1), so
/// the thing to look for is a public function of the component's own name in its own family's
/// module.
///
/// **`scrollbar` is the component and [`crate::scroll::bar`] is the helper**, which is why the
/// needle is `pub fn scrollbar(` and not `pub fn bar(`: components ticket 07 shipped the helper and
/// a scan that accepted it would report this screen as standing on a component nobody has written.
pub const DECLARATIONS: [(&str, &str); 3] = [
    ("scroll.rs", "pub fn scroll_area("),
    ("scroll.rs", "pub fn scrollbar("),
    ("scroll.rs", "pub fn sticky("),
];

/// **Which of [`SUBJECTS`] this crate actually declares. Today: none.**
///
/// A source scan and not a `use`, for [`crate::dense::subjects_declared`]'s reason: *the item does
/// not exist* has no expression, and a `compile_fail` fence would pass today and pass again the day
/// somebody renames the module. The predicate is `crate::dense::declares` and it is shared rather
/// than copied — one definition of *a line that is not a comment*.
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

/// **Whether these screens stand on their subjects, as a verdict rather than as a sentence.**
///
/// `Unmet` over three, inverted by **components 19**. Everything the screens themselves can be asked
/// is measured — the sweep over 5 475 600 pairs, the hysteresis, the reflow loop, the reachability
/// at both units, the thumb drift, the two arms of the two-areas screen and the pairing count — and
/// what is missing is the subject.
pub fn standing() -> Verdict {
    let declared = subjects_declared();
    Verdict::of(
        SUBJECTS.len(),
        SUBJECTS.len() - declared.len(),
        "`scroll_area`, `scrollbar` and `sticky` are not declared in this crate, so what stands on \
         these screens is a stand-in row loop, a bar drawn from `scroll::thumb`'s own arithmetic \
         and two rectangles. The sweep over 5 475 600 viewport x extent pairs, the hysteresis that \
         keeps both bars over content that fits, the 99 flips in 99 frames, row 799 999 of 999 999 \
         with no counter separating the two builds, the thumb drift and the two-areas re-damage \
         are all measured and green; what is missing is the subject",
        "components 19",
    )
}

/// **The sentence a scene waiting for its subject fails with**, or `None` once they are declared.
///
/// # This is criterion 7, and it is the distinction the whole ticket rests on
///
/// A scene that fails because it is unimplemented and a scene that fails because the code is wrong
/// are **the same failure** unless the message separates them. This one names the subjects, the file
/// they belong in, the declarations to look for, and the ticket — and says in as many words that it
/// is not a defect in the screen.
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
         components are undeclared — {}. This is not a defect in the screen. The bar decision is a \
         fixpoint over all 5 475 600 pairs in at most 3 passes, the incremental spelling keeps both \
         bars over content that fits, a body that is not antitone flips the decision 99 times in 99 \
         frames, the row-measured extent reaches row 799 999 of 999 999 with no counter separating \
         it from the correct build, and an overlay bar re-damages {} cells a frame against 0 — see \
         `crate::area::tests`. Inverted by `components 19`",
        owed.len(),
        SUBJECTS.len(),
        owed.join(", "),
        OVERLAY_REDAMAGE,
    ))
}

/// **Fail with the subjects that are missing, the file they belong in, and the ticket.**
///
/// # Panics
///
/// Panics while [`SUBJECTS`] are undeclared, which is **today**. Components ticket 19 inverts it.
pub fn assert_stands_up(scene: &str) {
    if let Some(message) = owed_message(&subjects_declared(), scene) {
        panic!("{message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── scene 17: the bar fixpoint ───────────────────────────────────────────────────────────────

    /// **Criterion 2: 5 475 600 viewport x extent pairs, 0 failures, worst case 3 passes.**
    ///
    /// A count and not a sample. Every interesting case in this arithmetic is a place it runs out —
    /// a viewport one cell wide, an extent exactly equal to the viewport, an extent one larger — and
    /// they are all at boundaries a sample walks past.
    #[test]
    fn the_bar_decision_is_a_fixpoint_over_five_million_pairs() {
        let swept = sweep();
        assert_eq!(swept.pairs, SWEEP_PAIRS, "the domain is not §9's");
        assert_eq!(swept.pairs, 5_475_600);
        assert_eq!(
            swept.failures, 0,
            "a decision whose own reduced rectangle asks for different bars"
        );
        assert_eq!(
            swept.worst_passes, MAX_PASSES,
            "§9's figure is *worst case 3 passes*, and the third is the pass that proves it is a \
             fixpoint. A worst case of 2 means the loop stopped checking"
        );
        assert_eq!(
            swept.unneeded, 0,
            "the decision is the *least* fixpoint: a bar that is not needed is not shown"
        );
    }

    /// **The least fixpoint at the boundary**, which is the only place the two fixpoints differ.
    #[test]
    fn a_bar_that_is_not_needed_is_not_shown() {
        let exact = decide((60, 20), (60, 20), Bars::Reserved);
        assert_eq!(exact.shown, Shown { v: false, h: false });
        assert_eq!(exact.passes, 1);
        // One more cell of content on either axis, and the coupling shows: the vertical bar takes a
        // column, which is what makes the horizontal one necessary.
        let one_more = decide((60, 20), (60, 21), Bars::Reserved);
        assert_eq!(one_more.shown, Shown { v: true, h: true });
        assert_eq!(one_more.passes, MAX_PASSES);
    }

    /// **The hysteresis, and what it costs on a screen that looks correct.**
    ///
    /// Fired in both directions: the fixpoint shows neither bar over content that fits and draws all
    /// twenty rows of twenty columns; the incremental spelling, handed the frame before in which
    /// both bars were up, keeps both — a row and a column lost permanently, with nothing in the
    /// frame's counts saying so.
    #[test]
    fn the_incremental_spelling_keeps_both_bars_over_content_that_fits() {
        let fixpoint = fitting(Hide::Fixpoint);
        let incremental = fitting(Hide::Incremental);

        assert_eq!(fixpoint.shown, Shown { v: false, h: false });
        assert_eq!((fixpoint.rows, fixpoint.cols), (FIT_H, FIT_W));
        assert!(fixpoint.last_row_drawn);

        assert_eq!(
            incremental.shown,
            Shown { v: true, h: true },
            "the spelling a reader writes has no way to come back down"
        );
        assert_eq!((incremental.rows, incremental.cols), (FIT_H - 1, FIT_W - 1));
        assert!(
            !incremental.last_row_drawn,
            "a row is lost permanently, and the screen looks entirely correct"
        );

        // And the same input from a cold start is right, which is what makes it hysteresis rather
        // than a wrong answer: the defect is in the history and not in the frame.
        let cold = decide_incremental(
            (FIT_W, FIT_H),
            (u32::from(FIT_W), u32::from(FIT_H)),
            Shown::default(),
            Bars::Reserved,
        );
        assert_eq!(cold.shown, Shown { v: false, h: false });
    }

    /// **Criterion 3: a body that breaks monotonicity flips the decision 99 times in 99 frames with
    /// no input.**
    ///
    /// Three arms, because a one-armed version says nothing: the responsive body flips under
    /// reserved bars, does not flip under overlay bars — which reserve nothing and therefore cannot
    /// close the loop — and an antitone body does not flip at all.
    #[test]
    fn a_body_that_is_not_antitone_flips_the_decision_every_frame() {
        let responsive = Responsive {
            wide: 60,
            tall: 30,
            short: 10,
            natural_w: 40,
        };
        let antitone = Responsive {
            short: 30,
            ..responsive
        };
        assert_eq!(
            flips(&oscillates(
                (60, 20),
                responsive,
                REFLOW_FRAMES,
                Bars::Reserved
            )),
            REFLOW_FLIPS,
            "the reflow loop ADR 0029 was opened to settle"
        );
        assert_eq!(
            flips(&oscillates(
                (60, 20),
                responsive,
                REFLOW_FRAMES,
                Bars::Overlay
            )),
            0,
            "an overlay bar reserves nothing, so the loop has no edge to close"
        );
        assert_eq!(
            flips(&oscillates(
                (60, 20),
                antitone,
                REFLOW_FRAMES,
                Bars::Reserved
            )),
            0,
            "the precondition holding is what makes the fixpoint's argument apply"
        );
    }

    /// Scene 17 is waiting for its subject, and the sentence says which failure it is.
    #[test]
    #[should_panic(expected = "waiting for its subject rather than failing")]
    fn the_bar_fixpoint_is_waiting_for_its_subject() {
        assert_stands_up("scene 17, the bar fixpoint over 5 475 600 viewport x extent pairs");
    }

    // ── scene 18: `Σ h` is the extent ────────────────────────────────────────────────────────────

    /// **`Σ h` is 1 250 000 where the row count is 1 000 000**, and the arithmetic says why.
    #[test]
    fn the_extent_is_the_sum_of_the_heights_and_not_the_row_count() {
        let content = Content::build(ROWS, true);
        assert_eq!(content.len() as u64, ROWS);
        assert_eq!(content.extent_cells(), EXTENT_CELLS);
        assert_eq!(content.extent_rows(), EXTENT_ROWS);
        // One row in eight is three cells tall, so the average is 1.25 and the extent is a quarter
        // longer. Asserted as a relation, so the constant cannot drift away from the fixture.
        assert_eq!(
            u64::from(EXTENT_CELLS),
            ROWS + (ROWS / TALL_EVERY) * (u64::from(TALL_H) - 1)
        );
    }

    /// **Criterion 4: the last reachable content row is 999 999, and the row-measured spelling
    /// reaches 799 999.**
    ///
    /// The assertion is on **reachability** and not on counts, because no count moves — which is the
    /// next test.
    #[test]
    fn the_row_measured_extent_cannot_reach_the_last_row() {
        let content = Content::build(ROWS, true);

        let cells = at_the_end(&content, Unit::Cells);
        let rows = at_the_end(&content, Unit::Rows);

        assert_eq!(cells.last_row, LAST_ROW_IN_CELLS, "999 999, the last row");
        assert_eq!(
            rows.last_row, LAST_ROW_IN_ROWS,
            "the row-measured spelling stops at 799 999, and 20% of the content is unreachable"
        );
        assert_eq!(cells.offset, max_offset(EXTENT_CELLS));
        assert_eq!(rows.offset, max_offset(EXTENT_ROWS));
        // The gap is the unit error itself: exactly the cells the tall rows added.
        assert_eq!(
            u64::from(EXTENT_CELLS - EXTENT_ROWS),
            (ROWS / TALL_EVERY) * (u64::from(TALL_H) - 1)
        );
        assert_eq!(
            (LAST_ROW_IN_CELLS - LAST_ROW_IN_ROWS) * 5,
            ROWS,
            "a fifth of the content is what the row spelling cannot reach"
        );
    }

    /// **No counter separates the two builds at an offset they both admit — the list is empty.**
    ///
    /// [`crate::listing::counters_that_separate_them`]'s shape, and it is what makes this a false
    /// green rather than a bug: time, writes, verbs, regions, tab stops and allocations are all
    /// identical, and the screen looks perfectly healthy.
    #[test]
    fn nothing_a_frame_counts_separates_a_cell_extent_from_a_row_extent() {
        let content = Content::build(ROWS, true);
        for offset in SWEPT_OFFSETS {
            let cells = frame_at(&content, Unit::Cells, offset);
            let rows = frame_at(&content, Unit::Rows, offset);
            assert_eq!(
                Shape { ..cells },
                Shape { ..rows },
                "the two builds differ at offset {offset}, which they must not: the unit error is \
                 in the extent and the extent is not drawn"
            );
        }
        assert_eq!(
            counters_that_separate_them(500_000),
            Vec::new(),
            "a counter that separates them would mean this defect is not the false green §9 says \
             it is, and the scene would need rewriting rather than the code"
        );

        // **And at each build's own furthest offset, which is the sharper form.** §9: *measured in
        // rows, time, writes, verbs, marked cells, regions and allocations are all identical.* The
        // two frames are at two different places in the content and every counter agrees to the
        // unit; the only fields that move are the offset the runtime clamped to and the row it
        // reaches, and neither is a counter.
        let cells = at_the_end(&content, Unit::Cells);
        let rows = at_the_end(&content, Unit::Rows);
        assert_ne!(cells.offset, rows.offset);
        assert_ne!(cells.last_row, rows.last_row);
        assert_eq!(
            Shape {
                offset: 0,
                last_row: 0,
                ..cells
            },
            Shape {
                offset: 0,
                last_row: 0,
                ..rows
            },
            "the two builds are at two different ends of the content and no counter says so"
        );
        assert_eq!(
            cells.writes,
            u64::from(AW) * u64::from(AH),
            "the body writes a partition of its viewport, which is what makes `writes` blind here: \
             it is the viewport's area at every offset either build can reach"
        );
    }

    /// **Criterion 5: the thumb is asserted in content cells, and the row-measured spelling drifts.**
    ///
    /// §9 remembers *up to 7 of 69* and this screen measures [`THUMB_DRIFT`] at the end of the
    /// reachable range; the two are the same arithmetic at two points of it, and the half-range
    /// figure **is** 7. Both are asserted so neither can be quietly replaced by the other.
    #[test]
    fn the_thumb_and_the_extent_share_a_unit() {
        assert_eq!(
            thumb_drift(),
            THUMB_DRIFT,
            "the drift at the end of the range the defective build can reach"
        );
        let seven = drift_reaches(REMEMBERED_DRIFT).expect("the drift passes seven cells");
        assert!(
            (seven - DRIFT_REACHES_SEVEN_AT).abs() < 0.01,
            "§9's *up to 7 cells of 69* is a point on the range and not its end, and it is at \
             {seven:.3} of it. The figure that moved is `up to`, not the arithmetic"
        );
        assert_eq!(
            drift_reaches(0),
            Some(0.0),
            "the drift starts at nothing, which is what makes it plausible"
        );
        assert!(
            thumb_drift() > REMEMBERED_DRIFT,
            "the drift is linear in the offset, so the worst case cannot be the half-range figure"
        );
        // And a correct build has none of it, whatever the offset.
        let content = Content::build(ROWS, true);
        for offset in SWEPT_OFFSETS {
            let both = scroll::thumb(
                AH,
                Span {
                    viewport: u32::from(AH),
                    extent: content.extent_cells(),
                    offset: u32::try_from(offset).unwrap_or(0),
                },
            );
            assert!(
                both.0 + both.1 <= AH,
                "the thumb left the track at {offset}"
            );
        }
    }

    /// The binary search answers the same thing the prefix sum does, at every boundary of it.
    #[test]
    fn the_row_at_a_cell_is_the_row_whose_span_contains_it() {
        let content = Content::build(200, true);
        for i in 0..content.len() {
            let top = content.top_of(i);
            let h = u32::from(content.rows[i].h);
            for dy in 0..h {
                assert_eq!(
                    content.row_at(top + dy),
                    i,
                    "cell {} is not in row {i}",
                    top + dy
                );
            }
        }
        // And the range a window claims is the range the rows actually cover.
        for y in 0..content.extent_cells() {
            let (lo, hi) = content.rows_in(y, AH);
            assert!(content.top_of(lo) <= y);
            assert!(lo < hi);
            let end = y + u32::from(AH);
            assert!(
                hi == content.len() || content.top_of(hi) >= end,
                "a row inside the window was left out at {y}"
            );
        }
    }

    /// **The runtime defect this screen works around, reproduced so the workaround cannot outlive
    /// it.**
    ///
    /// [`SCROLL_SCOPE_TRANSLATES_THE_WRONG_WAY`]. The day `Ctx::scroll_scope` hands `-offset` to
    /// `View::scrolled` this test fails, which is the point: a substitution nobody is watching is a
    /// substitution that becomes the design.
    #[test]
    fn the_runtime_still_translates_a_scrolled_scope_the_wrong_way() {
        use vitui_runtime::ctx::Driver;
        use vitui_runtime::scroll::Scrollable;

        let mut driver = Driver::headless(20, 6).expect("a sink attaches");
        let mut seen = (0..0, 0u16, 0u16);
        driver.frame(|cx| {
            let id = Id::named("probe");
            let view = Rect::new(0, 0, 20, 6);
            let _ = cx.scrollable(
                id,
                view,
                Interest::NONE,
                Scrollable::between((0, 5), (0, 34)),
            );
            cx.scroll_scope(id, view, (0, 5), (0, 34), |cx| {
                let paint = cx.theme().paint(Role::Body);
                let rows = cx.visible_rows();
                let at_five = cx.text(0, 5, "hello", paint).cells;
                let at_zero = cx.text(0, 0, "hello", paint).cells;
                seen = (rows, at_five, at_zero);
            });
        });
        assert_eq!(
            seen.0,
            -5..1,
            "`visible_rows` inside a scope at offset 5 answers with negative content rows. If this \
             now reads `5..11`, {SCROLL_SCOPE_TRANSLATES_THE_WRONG_WAY} has been settled and \
             `crate::area::draw_into`'s substitution must come out"
        );
        assert_eq!(
            seen.1, 0,
            "content row 5 is the top row and it writes nothing"
        );
        assert_eq!(
            seen.2, 5,
            "content row 0 writes, and it is five rows off screen"
        );
    }

    /// Scene 18 is waiting for its subject.
    #[test]
    #[should_panic(expected = "waiting for its subject rather than failing")]
    fn the_extent_scene_is_waiting_for_its_subject() {
        assert_stands_up("scene 18, a 1M-row scroll area with one row in eight three cells tall");
    }

    // ── scene 19: two areas far apart ────────────────────────────────────────────────────────────

    /// **Criterion 6: a bar over a drawing body re-damages [`OVERLAY_REDAMAGE`] cells every steady
    /// frame, against 0 for the reserved twin on the same screen with the same content.**
    ///
    /// The two arms are one function with one value between them, so a reviewer's diff is one
    /// argument. The count is asserted from the arithmetic as well as measured, which is what makes
    /// it a property of the mechanism rather than a number read off this screen: the cells written
    /// twice **are** the bars' footprint, because the bars are exactly what the body was written
    /// under.
    #[test]
    fn an_overlay_bar_re_damages_what_the_body_draws_under_it() {
        let reserved = steady(Bars::Reserved);
        let overlay = steady(Bars::Overlay);

        assert_eq!(
            reserved.cells, RESERVED_REDAMAGE,
            "the reserved twin re-damages nothing, which is the whole of ADR 0029"
        );
        assert_eq!(overlay.cells, OVERLAY_REDAMAGE);
        assert_eq!(
            overlay.cells,
            2 * (u64::from(H) + u64::from(AREA_W) - 1),
            "the excess is the two bars' footprint, minus the corner they share, twice"
        );
        // The symmetric split is what makes the figure one area's doubled.
        assert_eq!(overlay.cells % 2, 0);
        // And a reserved frame re-damages nothing under the span model either: zero cells changed
        // is zero spans, whichever way they are counted.
        assert_eq!(reserved.span_cells, 0);
    }

    /// **The amplification factor, reported as whatever the shipped damage structure gives.**
    ///
    /// §21's row for this scene says *damage is one span per row: ×8.4 amplification (owed)*, and
    /// the engine measured that model and rejected it. The bitset is **1.00× by construction** —
    /// `crates/vitui-engine/src/damage.rs`: *exact by construction, it can never report a cell the
    /// frame did not change* — so the number the row asks for cannot be measured on the engine that
    /// ships, and what is reported instead is what the span model *would* have charged on this
    /// screen. It is far above ×8.4, which is ADR 0029's own sentence rather than a contradiction of
    /// it: *the factor grows with how far apart the areas are*, and [`GAP`] is sixty columns.
    #[test]
    fn the_amplification_is_one_under_the_structure_that_ships() {
        let overlay = steady(Bars::Overlay);
        let bitset = Damage {
            cells: overlay.cells,
            span_cells: overlay.cells,
        };
        assert!(
            (bitset.amplification() - 1.0).abs() < f64::EPSILON,
            "the shipped bitset is exact by construction"
        );
        assert!(
            overlay.amplification() > 8.4,
            "the span model on two areas {GAP} columns apart: {:.2}x over {} cells",
            overlay.amplification(),
            overlay.cells
        );
        // The two damage models agree about nothing except which cells changed, which is the
        // finding: §9's 3 535 is the span picture of a double write of about {ADR_CONTESTED}.
        assert!(overlay.span_cells > overlay.cells);
        assert!(
            REMEMBERED_REDAMAGE.abs_diff(ADR_CONTESTED * 8) < ADR_CONTESTED,
            "3 535 is ~423 times the ×8.4 ADR 0029 states three paragraphs below it. If that stops \
             holding, the reading this module is built on is wrong and the header has to be rewritten"
        );
    }

    /// **The two-areas screen is the one this ticket collects a debt on, and the debt is the
    /// number.**
    ///
    /// §21 marks the row `(owed)` and states its figure as *×8.4 amplification*. The figure is not
    /// measurable on the shipped engine and the double write is, so what this test pins is the
    /// relation between the two: the remembered 3 535 is within one contested-cell count of ADR
    /// 0029's own `~423 × 8.4`, and this screen's own double write is within a tenth of the 423.
    #[test]
    fn the_owed_figure_is_the_span_model_and_the_double_write_is_the_measurable_one() {
        let overlay = steady(Bars::Overlay);
        let ratio = overlay.cells as f64 / ADR_CONTESTED as f64;
        assert!(
            (0.9..1.1).contains(&ratio),
            "this screen's double write is {} against ADR 0029's ~{ADR_CONTESTED}, a ratio of \
             {ratio:.3}. The two rectangles are not the same two rectangles and the magnitude is \
             the claim",
            overlay.cells
        );
        assert!(
            overlay.cells * 4 < REMEMBERED_REDAMAGE,
            "if the double write ever reaches the remembered 3 535, the reading in this module's \
             header is wrong: 3 535 would then be a double-write count after all"
        );
    }

    /// Scene 19 is waiting for its subject.
    #[test]
    #[should_panic(expected = "waiting for its subject rather than failing")]
    fn the_two_areas_scene_is_waiting_for_its_subject() {
        assert_stands_up("scene 19, two scroll areas far apart with overlay bars");
    }

    // ── the wrong pairing ────────────────────────────────────────────────────────────────────────

    /// **Criterion 7: the wrong pairing, as a count rather than as a microsecond figure.**
    ///
    /// A `scroll_area` iterates its content and a virtualised `collection` iterates its window, and
    /// **the two frames draw the same cells** — the engine reports a fully clipped verb as zero
    /// columns, so `writes` is identical and only the walk moves. That is
    /// [`crate::listing`]'s finding arriving on the other component of the pair, and it is why the
    /// gate here is `iterated` rather than `writes`.
    #[test]
    fn a_scroll_area_costs_its_content_and_a_collection_costs_its_window() {
        let area = pairing_shape(Pairing::Area, PAIRING_ROWS);
        let collection = pairing_shape(Pairing::Collection, PAIRING_ROWS);

        assert_eq!(
            area.iterated, PAIRING_ROWS,
            "the wrong pairing walks the whole content"
        );
        assert_eq!(
            collection.iterated,
            u64::from(AH),
            "a virtualised collection walks the window and nothing else"
        );
        assert_eq!(
            area.writes, collection.writes,
            "the counter a reader reaches for first does not move, which is why §9 states the \
             pairing as a rule rather than as a gate on writes"
        );
        assert!(
            area.verbs > collection.verbs,
            "the calls are made and thrown away by the clip"
        );
    }

    /// Scene 30 is waiting for its subject.
    #[test]
    #[should_panic(expected = "waiting for its subject rather than failing")]
    fn the_wrong_pairing_is_waiting_for_its_subject() {
        assert_stands_up("scene 30, a scroll_area over unbounded data");
    }

    // ── the standing, in both directions ─────────────────────────────────────────────────────────

    /// **The screens are red, and the verdict says over how many.**
    #[test]
    fn the_screens_are_waiting_for_three_subjects() {
        assert_eq!(subjects_declared(), Vec::<&str>::new());
        let verdict = standing();
        assert!(!verdict.met());
        let Verdict::Unmet {
            over,
            failing,
            inverted_by,
            ..
        } = verdict
        else {
            unreachable!("`standing` returned `Met` over an undeclared subject")
        };
        assert_eq!((over, failing), (SUBJECTS.len(), SUBJECTS.len()));
        assert_eq!(inverted_by, "components 19");
    }

    /// **The waiting message still says which failure it is**, fired from the other side.
    ///
    /// Handed a full declaration list it returns `None`, which is the day components 19 lands;
    /// handed a partial one it names exactly what is missing. A message that could only ever be
    /// produced is a message nobody has watched stop.
    #[test]
    fn the_waiting_message_says_which_failure_it_is_and_stops_when_it_should() {
        assert_eq!(owed_message(&SUBJECTS, "scene 17"), None);
        let message = owed_message(&["scroll_area"], "scene 19").expect("two are missing");
        assert!(
            message.contains("2 of 3 components are undeclared"),
            "{message}"
        );
        assert!(message.contains("`scrollbar`"), "{message}");
        assert!(message.contains("`sticky`"), "{message}");
        assert!(!message.contains("`scroll_area` ("), "{message}");
        assert!(message.contains("components 19"), "{message}");
        assert!(
            message.contains("This is not a defect in the screen"),
            "{message}"
        );
    }

    /// **The declaration scan reads a declaration and not a mention**, fired both ways.
    ///
    /// The needle for `scrollbar` is `pub fn scrollbar(` and this crate already ships
    /// [`crate::scroll::bar`], so a scan matching the helper would report a component nobody has
    /// written. Watched refusing exactly that.
    #[test]
    fn the_scan_does_not_mistake_the_helper_for_the_component() {
        let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/scroll.rs"))
            .expect("the family module");
        assert!(crate::dense::declares(&source, "pub fn bar("));
        for (_, declaration) in DECLARATIONS {
            assert!(
                !crate::dense::declares(&source, declaration),
                "`{declaration}` is declared and this module still says it is not"
            );
        }
        // And a comment naming one does not satisfy it, which is `dense::declares`'s own rule.
        assert!(!crate::dense::declares(
            "// pub fn scroll_area(cx: &mut Ctx)",
            "pub fn scroll_area("
        ));
    }
}
