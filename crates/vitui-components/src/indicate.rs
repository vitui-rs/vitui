//! **F5 indicators**, ~46 entries, expressed by `meter`, `chart`, `plot`, `overlay` and deadlines.
//!
//! The reduction is R2 and R4 (spec §18). R2 is the larger half and it is ADR 0018's `Role`:
//! status LEDs, health pills, dot indicators, badge variants, inline messages, banners, alerts and
//! callouts are an **argument**, not a component, and [`ROLE_VARIANTS`] is that list as a value with
//! `tests::no_role_variant_is_a_component_row` as the gate over [`crate::INVENTORY`].
//!
//! `chart`, `plot` and `overlay` declare this family and are homed under F10 and F9.
//!
//! # The two components here are one sentence each, and each sentence is a call into `chart`
//!
//! Components ticket 34 states them and `crate::INVENTORY`'s `COMPOSITIONS` carries both edges:
//!
//! - [`meter`] is **`chart`'s prefix construction at 2 rungs** — the ladder comes from
//!   [`geom`]`(Kind::Bars, …)` and nothing here decides how many sub-cells a rung has;
//! - [`sparkline`] is **`chart` at a small rectangle, no axes, no gutter, no axis loop** — the same
//!   raster memo and the same body loop, with the chrome deleted rather than reimplemented.
//!
//! # The prefix ladder is shared and the prefix *spelling* is not, and that is a fact about Unicode
//!
//! `chart`'s bars grow **upward**, so its partial cell is one of the lower eighth blocks —
//! U+2581…U+2588, one contiguous run, and [`crate::chart::raster::cluster`] is the lookup.
//! A horizontal [`meter`] grows **rightward**, and the left eighth blocks are a *different*
//! contiguous run — U+258F…U+2588, running the other way. So [`meter`]'s vertical arm **reaches**
//! `cluster` and its horizontal arm cannot, and `LEFT8` — this module's own private run — is what
//! the horizontal arm spells instead.
//!
//! That is a spelling and not a ladder: how many sub-cells a rung offers is
//! [`geom`]`(Kind::Bars, set).sy` in both arms, which is the number `constructions: 2` is derived
//! from, and `tests::the_two_prefix_runs_are_one_ladder_and_two_spellings` sweeps the two runs
//! against each other at every eighth.

use vitui_runtime::{Ctx, Glyph, Rect, Response, Role};

use crate::chart::raster::{KeyMode, Kind, PlotState, Range, Reach, geom};
use crate::chart::{Body, SERIES_RGB, Series, body_into, series_paint};
use crate::ink::{Direct, Ink};
use crate::scroll::{Orient, stripe};

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["meter", "sparkline", "spinner"];

/// **The role variants §18's R2 absorbs, as a value.**
///
/// > `primary / secondary / ghost / danger`, link buttons, status LEDs, health pills, dot
/// > indicators, badge variants, inline messages, banners, alerts and callouts — all of them are an
/// > argument, not a component.
///
/// The list is [`crate::collect::ABSORBED`]'s arrangement for the same reason: a collapse recorded
/// only in prose is a collapse a later ticket undoes by adding a row, and
/// `tests::no_role_variant_is_a_component_row` is what makes undoing it a failing test instead of a
/// review comment.
///
/// **Spelled as component ids would be spelled**, because that is the form the drift takes: nobody
/// adds a row called *ghost*, they add one called `ghost_button`.
pub const ROLE_VARIANTS: [&str; 18] = [
    "primary_button",
    "secondary_button",
    "ghost_button",
    "danger_button",
    "link_button",
    "icon_button",
    "status_led",
    "health_pill",
    "dot_indicator",
    "badge",
    "tag",
    "pill",
    "inline_message",
    "banner",
    "alert",
    "callout",
    "toast",
    "notice",
];

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// `meter` — `chart`'s prefix construction at two rungs
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **The nine prefixes a horizontal meter cell is one of, at the eighth-block rung.**
///
/// U+258F LEFT ONE EIGHTH BLOCK up to U+2588 FULL BLOCK, indexed by eighths filled. It is
/// [`crate::chart::raster`]'s `BAR8` on the other axis, and the two are swept against each other by
/// `tests::the_two_prefix_runs_are_one_ladder_and_two_spellings`.
///
/// **Index 8 is `█`, which is also what `Theme::glyph(Glyph::Thumb)` spells at this rung** — so a
/// meter's full cells and its topped-up partial cell agree without either being told about the
/// other.
const LEFT8: [char; 9] = [
    ' ', '\u{258F}', '\u{258E}', '\u{258D}', '\u{258C}', '\u{258B}', '\u{258A}', '\u{2589}',
    '\u{2588}',
];

/// [`meter`]'s options.
///
/// Spec §1's rule 3: a `Default` struct, never a required builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MeterOpts {
    /// Which way it fills. Horizontal by default — a meter is a bar beside a label far more often
    /// than it is a column, and [`Orient`]'s own default is a scrollbar's reason rather than this
    /// one. [`crate::input::SliderOpts`] spells its default out for the same reason.
    pub orient: Orient,
    /// The stretch that is filled.
    pub done: Role,
    /// The stretch that is not.
    pub rest: Role,
}

impl Default for MeterOpts {
    fn default() -> MeterOpts {
        MeterOpts {
            orient: Orient::Horizontal,
            done: Role::Ok,
            rest: Role::Dim,
        }
    }
}

/// **A fraction of a track, filled to the sub-cell where the rung has sub-cells.**
///
/// **Hostile axes:** none.
///
/// It writes every cell of the rectangle it was handed, from a fraction, and holds nothing across
/// frames. The ladder is the repertoire's and not the width's.
///
/// `value` is a fraction, `0.0..=1.0`. **It is not a range**, which is spec §9's unit rule and
/// [`crate::input::slider`]'s own sentence: the one thing no layer of this library does for its
/// caller is decide what a number means.
///
/// **`gauge` collapses into this row, and that collapse is a judgement rather than a measurement** —
/// the only one of §17's six that is. `crate::INVENTORY`'s own doc comment carries the marking and
/// `crate::inventory::tests::the_six_collapses_are_recorded_and_the_judgement_is_marked` reads it.
///
/// ```
/// use vitui_components::indicate::meter;
/// use vitui_runtime::Rect;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(20, 1).expect("a sink attaches");
/// driver.frame(|cx| {
///     let resp = meter(cx, Rect::new(0, 0, 20, 1), 0.5);
///     // Rule 4: a `Response` back even from a pure drawer, and it declares no region.
///     assert!(!resp.hovered);
/// });
/// assert_eq!(driver.inspect().hits().len(), 0);
/// ```
#[track_caller]
pub fn meter(cx: &mut Ctx<'_, '_>, area: Rect, value: f32) -> Response {
    meter_with(cx, area, value, &MeterOpts::default())
}

/// [`meter`], with the options spelled out.
#[track_caller]
pub fn meter_with(cx: &mut Ctx<'_, '_>, area: Rect, value: f32, opts: &MeterOpts) -> Response {
    meter_into(&mut Direct, cx, area, value, opts)
}

/// **[`meter`], drawing through an [`Ink`] so a counter can see every cell.**
///
/// The entry point a gate takes; [`meter`] is this with [`Direct`].
#[track_caller]
pub fn meter_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    value: f32,
    opts: &MeterOpts,
) -> Response {
    let id = cx.id();
    if area.is_empty() {
        return Response::inert(id, area);
    }
    let length = match opts.orient {
        Orient::Vertical => area.h,
        Orient::Horizontal => area.w,
    };
    // **The ladder is `chart`'s and this line is the whole of the composition edge.** One sub-cell
    // at ASCII, eight from the Unicode rung up — which is `CONTEXT.md`'s *Unicode with box drawing
    // and block elements*, so an operator who has promised block elements has promised all of them
    // and there is no third thing for a meter to build.
    let sub = u32::from(geom(Kind::Bars, cx.theme().glyphs()).sy).max(1);
    let (full, part) = filled(value, length, sub);

    let theme = cx.theme();
    let done = theme.paint(opts.done);
    let rest = theme.paint(opts.rest);
    let thumb = theme.glyph(Glyph::Thumb);
    let track = theme.glyph(Glyph::Track);
    // The partial cell's spelling, and the axis is what decides which contiguous run it comes from.
    // See this module's header: the ladder is shared and the spelling cannot be.
    let mut buf = [0u8; 4];
    let partial: &str = partial_cell(opts.orient, sub, part, &mut buf);

    // **Track order**, so everything below is orientation-free: a vertical meter fills from the
    // bottom, so its filled stretch is the *last* `full` cells of the column.
    let (from_full, from_part) = match opts.orient {
        Orient::Horizontal => (0, full),
        Orient::Vertical => (length - full, length - full - u16::from(part > 0)),
    };
    stripe(ink, cx, area, opts.orient, from_full, full, thumb, done);
    if part > 0 {
        stripe(ink, cx, area, opts.orient, from_part, 1, partial, done);
    }
    let used = full + u16::from(part > 0);
    let (from_rest, n_rest) = match opts.orient {
        Orient::Horizontal => (used, length - used),
        Orient::Vertical => (0, length - used),
    };
    stripe(ink, cx, area, opts.orient, from_rest, n_rest, track, rest);
    Response::inert(id, area)
}

/// **How many whole cells are filled, and how many eighths of the one after them.**
///
/// The arithmetic is done in **sub-cells and never in floats after the first line**, which is
/// [`crate::input::stepped`]'s finding on the other axis: a value accumulated in `f32` is wrong in
/// its last digits and wrong on the screen at different moments, and the two defects hide each
/// other.
fn filled(value: f32, length: u16, sub: u32) -> (u16, u32) {
    let value = if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let total = u32::from(length) * sub;
    let filled = (value * total as f32).round() as u32;
    let filled = filled.min(total);
    ((filled / sub) as u16, filled % sub)
}

/// The cluster a partial cell is spelled with, on the axis the meter runs along.
///
/// A `&mut [u8; 4]` because [`Ink::run`] takes a `&str` and a `char` has to live somewhere: the
/// component allocates nothing, which is what `crate::gates`' row for it asserts as a total.
fn partial_cell(orient: Orient, sub: u32, part: u32, buf: &mut [u8; 4]) -> &str {
    if part == 0 || sub <= 1 {
        return " ";
    }
    let ch = match orient {
        // **Reached, not transcribed.** `chart`'s prefix grows upward and so does a vertical
        // meter's, so the spelling is the same lookup — a mask with `part` bits set is what
        // `cluster` counts.
        Orient::Vertical => {
            crate::chart::raster::cluster(Kind::Bars, geom_for(sub), ((1u16 << part) - 1) as u8)
        }
        Orient::Horizontal => LEFT8[part.min(8) as usize],
    };
    ch.encode_utf8(buf)
}

/// The [`crate::chart::raster::Geom`] a sub-cell count came from, rebuilt so that
/// [`crate::chart::raster::cluster`] can be called with it.
///
/// It is not a second ladder: `sub` came out of [`geom`] one call earlier and this puts it back into
/// the shape that function's own consumer takes.
fn geom_for(sub: u32) -> crate::chart::raster::Geom {
    crate::chart::raster::Geom {
        sx: 1,
        sy: sub.min(255) as u8,
    }
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// `sparkline` — `chart` at a small rectangle: no axes, no gutter, no axis loop
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// [`sparkline`]'s options.
///
/// Spec §1's rule 3: a `Default` struct, never a required builder. **There is no `threshold` field
/// and no `declared` field**, and both absences are the component's own claim: a sparkline that took
/// a threshold would need a row to draw it on, and a row it draws chrome on is a chart.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct SparkOpts {
    /// Whether the series colours are role-derived rather than named — [`crate::chart::Opts`]'s own
    /// field, carried through so a sparkline in a sixteen-colour terminal has the same answer a
    /// chart does.
    ///
    /// **`false` is the default and it is `Default`'s own**, which is why this struct derives the
    /// impl where every other options struct in this crate writes one: there is exactly one field
    /// and its default is the type's, so a hand-written impl would be a place for the two to drift.
    pub role_series: bool,
}

/// **A chart with no chrome: every cell of the rectangle is body.**
///
/// **Hostile axes:** none.
///
/// Every cell of the rectangle is body, and the fold is the caller's [`PlotState`] — so there is no
/// chrome whose construction could change with the width and no offset to be wrong about.
///
/// The memo is the caller's [`PlotState`], exactly as [`crate::chart::chart`]'s is — rule 2 puts the
/// data behind a shared reference and the state in a `&mut` beside it.
///
/// ```
/// use vitui_components::chart::{Series, raster::PlotState};
/// use vitui_components::indicate::sparkline;
/// use vitui_runtime::Rect;
/// use vitui_runtime::ctx::Driver;
///
/// let data = Series::build(1_000, 1);
/// let mut state = PlotState::new();
/// let mut driver = Driver::headless(20, 2).expect("a sink attaches");
/// driver.frame(|cx| { let _ = sparkline(cx, Rect::new(0, 0, 20, 2), &data, &mut state); });
/// // The raster is the whole rectangle: no gutter, no axis row.
/// assert_eq!((state.raster().w(), state.raster().h()), (20, 2));
/// assert_eq!(state.misses(), 1);
/// ```
#[track_caller]
pub fn sparkline(cx: &mut Ctx<'_, '_>, area: Rect, data: &Series, st: &mut PlotState) -> Response {
    sparkline_with(cx, area, data, st, &SparkOpts::default())
}

/// [`sparkline`], with the options spelled out.
#[track_caller]
pub fn sparkline_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    data: &Series,
    st: &mut PlotState,
    opts: &SparkOpts,
) -> Response {
    sparkline_into(&mut Direct, cx, area, data, st, opts)
}

/// **[`sparkline`], drawing through an [`Ink`] so a counter can see every cell.**
///
/// # What is *not* here, and each absence is one of ticket 34's criteria
///
/// No gutter, because a gutter is a label column and there is nothing to label. No tick values, no
/// [`crate::chart::axes::nice_step`], no [`crate::chart::axes::tick_count`]. No axis row and no
/// corner. **No loop to decouple**: `chart`'s gutter is computed from the whole domain precisely so
/// that the width does not depend on the rectangle, and a component with no gutter cannot have that
/// coupling in the first place.
///
/// What *is* here is the whole of `chart`'s body — the same raster memo, the same fold, the same
/// run-splitting loop, reached through `chart`'s own crate-private `body_into` rather than copied.
#[track_caller]
pub fn sparkline_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    data: &Series,
    st: &mut PlotState,
    opts: &SparkOpts,
) -> Response {
    let id = cx.id();
    if area.is_empty() {
        return Response::inert(id, area);
    }
    let g = geom(Kind::Bars, cx.theme().glyphs());
    // The range memo is `chart`'s and the domain is the whole series', for `chart`'s own reason:
    // an axis scaled to the window is an axis whose width depends on the rectangle.
    let dom = st.range(data.revision(), Range::Whole, data.points());
    let mut series_paints = [cx.theme().paint(Role::Dim); SERIES_RGB.len()];
    for (i, p) in series_paints.iter_mut().enumerate() {
        *p = series_paint(cx.theme(), i, opts.role_series);
    }
    let dim = cx.theme().paint(Role::Dim);
    st.refresh_keyed(
        KeyMode::Full,
        data.revision(),
        area.w,
        area.h,
        Kind::Bars,
        g,
        dom,
        data.points(),
        Reach::Mapped,
        (0, data.len()),
    );
    let body = Body {
        kind: Kind::Bars,
        geom: g,
        series: series_paints,
        dim,
        threshold: None,
    };
    let (raster, row) = st.raster_and_row();
    body_into(ink, cx, area, raster, row, &body);
    Response::inert(id, area)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::raster::{RUNGS, cluster};
    use crate::chart::{Opts, chart_into};
    use crate::counters::Tally;
    use crate::runner::{Canvas, Pen};
    use crate::{INVENTORY, Tier};
    use vitui_runtime::ColorDepth;
    use vitui_runtime::ctx::Driver;
    use vitui_runtime::theme::{CATPPUCCIN_MOCHA, Density, Theme};

    /// A tally over one frame of `f`, on a `w` by `h` sink.
    fn tallied(w: u16, h: u16, f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>)) -> Tally {
        let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| f(&mut tally, cx));
        tally
    }

    /// A driver whose theme is authored at [`RUNGS`]`[rung]`.
    ///
    /// **The rung is an index and not a named repertoire**, which is `chart::raster::RUNGS`'s own
    /// purpose: naming one here would make this module the second file in the crate that spells a
    /// repertoire, and the exception §16 states is worth exactly one file.
    fn driver_at(w: u16, h: u16, rung: usize) -> Driver {
        let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
        driver.set_theme(
            Theme::authored(&CATPPUCCIN_MOCHA, RUNGS[rung], Density::default())
                .resolve(ColorDepth::TrueColor),
        );
        driver
    }

    /// **The ladder is one and the spellings are two, and the second half is a fact about Unicode.**
    ///
    /// `chart`'s prefix grows upward and a horizontal meter's grows rightward, and the eighth blocks
    /// are **two** contiguous runs rather than one — U+2581…U+2588 climbing and U+258F…U+2588 filling
    /// from the left. So the vertical arm *reaches* [`cluster`] and the horizontal arm cannot.
    ///
    /// Three assertions, because the interesting claim is the disagreement: the two runs are the
    /// same length, they agree at the ends (nothing filled and everything filled), and they agree
    /// **nowhere** in between. A meter that had transcribed `chart`'s table onto the wrong axis
    /// would draw a bar that grows upward inside a row, and every counter in this crate would
    /// report it as correct.
    #[test]
    fn the_two_prefix_runs_are_one_ladder_and_two_spellings() {
        let g = crate::chart::raster::Geom { sx: 1, sy: 8 };
        let upward: Vec<char> = (0..=8u32)
            .map(|k| cluster(Kind::Bars, g, ((1u16 << k) - 1) as u8))
            .collect();
        assert_eq!(upward.len(), LEFT8.len());
        assert_eq!(upward[0], LEFT8[0], "nothing filled is nothing either way");
        assert_eq!(upward[8], LEFT8[8], "everything filled is `█` either way");
        let agree = (1..8).filter(|&k| upward[k] == LEFT8[k]).count();
        assert_eq!(
            agree, 0,
            "the two runs agree at {agree} of the seven partial eighths, and they are supposed to \
             agree at none"
        );
        // And the whole cells the two arms share come out of the theme rather than out of either
        // run: `Glyph::Thumb` is `█` at this rung, which is `LEFT8[8]`.
        let theme = Theme::authored(&CATPPUCCIN_MOCHA, RUNGS[1], Density::default())
            .resolve(ColorDepth::TrueColor);
        assert_eq!(theme.glyph(Glyph::Thumb), LEFT8[8].to_string());
    }

    /// **Criterion 3: `meter` is 2 constructions, and the ladder is `chart`'s rather than its own.**
    ///
    /// Block elements are the *Unicode* rung by `CONTEXT.md`'s own definition — *Unicode with box
    /// drawing and block elements* — so an operator who has promised block elements has promised all
    /// of them and the top two rungs are one construction. `1 / 8 / 8` is three readings and two
    /// values, which is the number §17's column carries.
    #[test]
    fn a_meter_is_two_constructions_and_the_ladder_is_charts() {
        let ladders: Vec<u8> = RUNGS.iter().map(|&set| geom(Kind::Bars, set).sy).collect();
        assert_eq!(ladders, vec![1, 8, 8]);
        let distinct: std::collections::BTreeSet<u8> = ladders.iter().copied().collect();
        assert_eq!(distinct.len(), 2);
        let row = INVENTORY
            .iter()
            .find(|c| c.id == "meter")
            .expect("the freeze has a `meter`");
        assert_eq!(row.constructions as usize, distinct.len());
        assert_eq!(row.tier, Tier::Two);
        // And the same derivation gives the `sparkline` row its 2, which §17 does not state — see
        // that row's own comment.
        let spark = INVENTORY
            .iter()
            .find(|c| c.id == "sparkline")
            .expect("the freeze has a `sparkline`");
        assert_eq!(spark.constructions as usize, distinct.len());

        // The two constructions really are two pictures, drawn rather than derived.
        let painted = |rung: usize| {
            let mut driver = driver_at(8, 1, rung);
            let mut pen = Pen::over(Canvas::new(8, 1));
            driver.frame(|cx| {
                let area = cx.area();
                meter_into(&mut pen, cx, area, 0.44, &MeterOpts::default());
            });
            pen.into_canvas()
        };
        assert!(painted(0).diff(&painted(1)).cells > 0);
        assert_eq!(
            painted(1).diff(&painted(2)).cells,
            0,
            "the third rung buys a prefix nothing, which is `chart`'s own finding"
        );
    }

    /// **Criterion 7 for `meter`: a partition of the whole rectangle at both orientations.**
    ///
    /// Swept where the arithmetic runs out — one cell, one row, one column, a value that is `NaN` or
    /// outside `0..=1`, and both ends of the track.
    #[test]
    fn a_meter_writes_a_partition_of_its_whole_rectangle() {
        for orient in [Orient::Horizontal, Orient::Vertical] {
            for (w, h) in [
                (1u16, 1u16),
                (2, 1),
                (1, 2),
                (3, 3),
                (20, 1),
                (1, 20),
                (12, 4),
            ] {
                // **And at an origin that is not the screen's** — the arm `crate::collect`'s table
                // needed, where a component that drew from `x = 0` instead of from the band it was
                // handed passed every gate, because every gate played where the two agree.
                for (ox, oy) in [(0i32, 0i32), (5, 2)] {
                    for value in [0.0f32, 0.001, 0.4, 0.5, 0.999, 1.0, f32::NAN, -2.0, 7.0] {
                        let opts = MeterOpts {
                            orient,
                            ..MeterOpts::default()
                        };
                        let at = Rect::new(ox, oy, w, h);
                        let (sw, sh) = (w + ox as u16, h + oy as u16);
                        let tally = tallied(sw, sh, |tally, cx| {
                            meter_into(tally, cx, at, value, &opts);
                        });
                        assert_eq!(
                            tally.writes(),
                            tally.distinct(),
                            "{orient:?} {w}x{h}@{ox},{oy} at {value}: a cell written twice"
                        );
                        assert_eq!(
                            tally.distinct(),
                            u64::from(w) * u64::from(h),
                            "{orient:?} {w}x{h}@{ox},{oy} at {value}: not its own track"
                        );
                        assert_eq!(
                            tally.asked(),
                            tally.reported(),
                            "{orient:?} {w}x{h}@{ox},{oy} at {value}: a verb left"
                        );
                        for y in oy..oy + i32::from(h) {
                            for x in ox..ox + i32::from(w) {
                                assert!(
                                    tally.touched(x, y),
                                    "{orient:?} {w}x{h}@{ox},{oy}: ({x}, {y}) unwritten"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    /// **The fill is counted in sub-cells and the count is monotone, exact at both ends and exact at
    /// every eighth.**
    ///
    /// `crate::input::stepped`'s finding on the other axis: an accumulated `f32` is wrong in its last
    /// digits and wrong on the screen at different moments, and the two defects hide each other. Here
    /// the value arrives once and everything after it is integers, so *filled to k eighths* and
    /// *`k / (length × sub)`* are the same number by construction.
    #[test]
    fn the_fill_is_sub_cells_and_it_is_exact_at_every_eighth() {
        const LENGTH: u16 = 20;
        const SUB: u32 = 8;
        let total = u32::from(LENGTH) * SUB;
        for k in 0..=total {
            let (full, part) = filled(k as f32 / total as f32, LENGTH, SUB);
            assert_eq!(
                u32::from(full) * SUB + part,
                k,
                "{k} of {total} eighths came back as {full} cells and {part} eighths"
            );
        }
        // The ends, and the two values a caller can hand in that are not fractions at all.
        assert_eq!(filled(0.0, LENGTH, SUB), (0, 0));
        assert_eq!(filled(1.0, LENGTH, SUB), (LENGTH, 0));
        assert_eq!(filled(f32::NAN, LENGTH, SUB), (0, 0));
        assert_eq!(filled(7.0, LENGTH, SUB), (LENGTH, 0));
        assert_eq!(filled(-2.0, LENGTH, SUB), (0, 0));
        // And the bottom rung has no eighths at all, so the same fraction rounds to whole cells.
        assert_eq!(filled(0.44, LENGTH, 1), (9, 0));
    }

    /// **Criterion 5: a sparkline's raster is the whole rectangle and a chart's is not.**
    ///
    /// The measurable form of *no axes, no gutter, no axis loop*. `chart` at 20x5 reserves a gutter
    /// whose width comes from the domain and an axis row underneath, so its raster is smaller than
    /// its rectangle on both axes; a sparkline's is the rectangle. The subtraction is the claim, and
    /// [`crate::composed`]'s scan is the other half — a component that had the machinery and did not
    /// happen to use it would pass this and fail that.
    #[test]
    fn a_sparkline_is_the_whole_rectangle_and_a_chart_is_not() {
        let data = Series::build(500, 1);
        let mut spark_state = PlotState::new();
        let mut chart_state = PlotState::new();
        let mut driver = Driver::headless(20, 5).expect("a sink attaches");
        driver.frame(|cx| {
            let area = cx.area();
            sparkline(cx, area, &data, &mut spark_state);
        });
        driver.frame(|cx| {
            let area = cx.area();
            chart_into(
                &mut Direct,
                cx,
                area,
                &data,
                &mut chart_state,
                &Opts::chart(),
            );
        });
        assert_eq!(
            (spark_state.raster().w(), spark_state.raster().h()),
            (20, 5)
        );
        let (cw, ch) = (chart_state.raster().w(), chart_state.raster().h());
        assert!(
            cw < 20 && ch < 5,
            "a chart reserved no chrome: {cw}x{ch} of 20x5"
        );
        assert_eq!(ch, 4, "one axis row");
        // The gutter is what the difference on the other axis is, and it is the chart's own.
        assert!(20 - cw >= 2, "the gutter is {} columns", 20 - cw);
    }

    /// **Criterion 7 for `sparkline`, and `CONTEXT.md`'s invariant beside it.**
    ///
    /// A partition of the rectangle, the memo folded exactly once however many frames run, and the
    /// frame identical at a thousand points and at a million — *frame cost is proportional to
    /// visible cells, never to data volume*.
    #[test]
    fn a_sparkline_writes_a_partition_and_folds_once_however_much_data_there_is() {
        for (w, h) in [(1u16, 1u16), (4, 1), (20, 2), (30, 4)] {
            // And at an origin that is not the screen's — see the meter's own sweep.
            for (ox, oy) in [(0i32, 0i32), (5, 2)] {
                let data = Series::build(1_000, 1);
                let mut st = PlotState::new();
                let at = Rect::new(ox, oy, w, h);
                let tally = tallied(w + ox as u16, h + oy as u16, |tally, cx| {
                    sparkline_into(tally, cx, at, &data, &mut st, &SparkOpts::default());
                });
                assert_eq!(
                    tally.writes(),
                    tally.distinct(),
                    "{w}x{h}@{ox},{oy}: a cell twice"
                );
                assert_eq!(
                    tally.distinct(),
                    u64::from(w) * u64::from(h),
                    "{w}x{h}@{ox},{oy}"
                );
                assert_eq!(
                    tally.asked(),
                    tally.reported(),
                    "{w}x{h}@{ox},{oy}: a verb left"
                );
                for y in oy..oy + i32::from(h) {
                    for x in ox..ox + i32::from(w) {
                        assert!(
                            tally.touched(x, y),
                            "{w}x{h}@{ox},{oy}: ({x}, {y}) unwritten"
                        );
                    }
                }
            }
        }

        // One fold over twenty frames, and the same verbs and writes at three orders of magnitude.
        let mut counted = Vec::new();
        for n in [1_000usize, 100_000, 1_000_000] {
            let data = Series::build(n, 1);
            let mut st = PlotState::new();
            let mut driver = Driver::headless(30, 4).expect("a sink attaches");
            let mut tally = Tally::new();
            for _ in 0..20 {
                tally = Tally::new();
                driver.frame(|cx| {
                    let area = cx.area();
                    sparkline_into(&mut tally, cx, area, &data, &mut st, &SparkOpts::default());
                });
            }
            assert_eq!(st.misses(), 1, "{n} points folded {} times", st.misses());
            counted.push((tally.writes(), tally.verbs()));
        }
        // **The writes are flat and the verbs are not**, and that is §21's own sentence rather than
        // a shortfall: *`verbs <= writes`, never verb equality across sizes*. A verb here is a
        // **run**, and a run ends where the cell's owner changes — which is a property of where the
        // data's empty cells fall and not of how many points there are. Measured on this fixture:
        // **16 / 13 / 11** verbs at a thousand, a hundred thousand and a million points, over the
        // same **120** writes. A gate that had asserted the verbs equal would have been asserting a
        // fact about `Series::build`, and asserting them *monotone* would be asserting one about
        // this particular fixture's shape — so the relation is the gate and the three figures are
        // the report.
        let writes: Vec<u64> = counted.iter().map(|c| c.0).collect();
        assert_eq!(
            writes,
            vec![120, 120, 120],
            "30 x 4 cells, every one written"
        );
        for (w, v) in &counted {
            assert!(v <= w, "{v} verbs over {w} writes");
        }
        assert!(
            counted.iter().all(|c| c.1 > 0 && c.1 < 120),
            "every size splits the rows into runs: {counted:?}"
        );
    }

    /// **The R2 collapse as a gate: no role variant is a component row.**
    ///
    /// > `primary / secondary / ghost / danger`, link buttons, status LEDs, health pills, dot
    /// > indicators, badge variants, inline messages, banners, alerts and callouts — all of them are
    /// > an argument, not a component.
    ///
    /// Both halves, because a list nothing is compared against is a list: the eighteen names are
    /// absent from the freeze, and the freeze is not empty.
    #[test]
    fn no_role_variant_is_a_component_row() {
        assert_eq!(INVENTORY.len(), 29);
        for variant in ROLE_VARIANTS {
            assert!(
                !INVENTORY.iter().any(|c| c.id == variant),
                "`{variant}` is a row of the freeze, and §18's R2 says it is a `Role`"
            );
        }
        // **Watched firing**, because a scan over a list that happens to miss is not a gate: the one
        // name that *is* a row differs from a variant by the argument it would have been.
        assert!(INVENTORY.iter().any(|c| c.id == "button"));
        assert!(ROLE_VARIANTS.contains(&"primary_button"));
        assert_eq!(
            ROLE_VARIANTS
                .iter()
                .filter(|v| INVENTORY.iter().any(|c| c.id == **v))
                .count(),
            0
        );
    }
}
