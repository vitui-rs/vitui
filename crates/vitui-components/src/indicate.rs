//! Meters, sparklines, spinners and badges — the small components that report a value.
//!
//! [`meter`] is a fraction as a bar, [`sparkline`] is a series in one row of cells, [`spinner`] is
//! a running indicator, and the badge shapes are in [`crate::text`] as chips. All of them degrade
//! with the terminal rather than assuming a glyph is available.
//!
//! # Examples
//!
//! ```
//! use vitui_components::indicate::{MeterOpts, meter, meter_with};
//! use vitui_runtime::Rect;
//! use vitui_runtime::ctx::Driver;
//!
//! let mut driver = Driver::headless(20, 3).expect("a sink attaches");
//! driver.frame(|cx| {
//!     // A fraction, never a range: 0.0 is empty and 1.0 is full.
//!     meter(cx, Rect::new(0, 0, 20, 1), 0.42);
//!
//!     let opts = MeterOpts::default();
//!     meter_with(cx, Rect::new(0, 1, 20, 1), 1.0, &opts);
//! });
//! ```
//!
//! # `value` is a fraction, and that is not a simplification
//!
//! A meter takes `0.0..=1.0` and never a range, because a component that took `(value, min, max)`
//! would be doing the caller's arithmetic in the caller's units and then rounding it into cells.
//! Convert once, at the call site, where the units are known.
//!
//! # Three ladders, and the terminal decides which one you get
//!
//! A meter, a sparkline and a spinner each have an ASCII rung, a Unicode rung and an extended rung,
//! and the theme's glyph set picks between them. That is why a bar looks like eighths on one
//! terminal and like `#` on another, with no branch in the calling code.

use std::time::{Duration, Instant};

use vitui_runtime::anim::Steps;
use vitui_runtime::{Ctx, Glyph, GlyphSet, Rect, Response, Role};

use crate::chart::raster::{KeyMode, Kind, PlotState, RUNGS, Range, Reach, geom};
use crate::chart::{Body, SERIES_RGB, Series, body_into, series_paint};
use crate::ink::{Direct, Ink};
use crate::scroll::{Orient, stripe};

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["meter", "sparkline", "spinner"];

/// **The role variants R2 absorbs, as a value.**
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
/// A `Default` struct, never a required builder.
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
/// `value` is a fraction, `0.0..=1.0`. **It is not a range**, which is the unit rule and
/// [`crate::input::slider`]'s own sentence: the one thing no layer of this library does for its
/// caller is decide what a number means.
///
/// **`gauge` collapses into this row, and that collapse is a judgement rather than a measurement** —
/// the only one of the six that is. `crate::INVENTORY`'s own doc comment carries the marking and
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
/// A `Default` struct, never a required builder. **There is no `threshold` field
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
/// # What is *not* here, and each absence is deliberate
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

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// `spinner` — the twenty-ninth row: an anchor, a ladder of its own, and no clock
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// **The spinner's frame set at each of the three rungs**, in [`RUNGS`]'s order.
///
/// It is the component's own table and not a [`Glyph`], and that is the rule working rather than
/// a hole. A `Glyph` is *one lookup with no spelling blank* — one meaning with one spelling per
/// rung; a spinner needs an **ordered set of `n` spellings that differ from each other**, which is
/// not one lookup and not `n` of them either: cycling `ArrowUp`, `ArrowDown`, `ArrowLeft`,
/// `ArrowRight` at a reader is telling them nothing four times. So the ladder lives here, exactly as
/// [`crate::chart::raster::RUNGS`] is `chart`'s and [`crate::media::sub_rows`] is the picture's.
///
/// # Three ladders, and the middle one is a correction to the prototype
///
/// The prototype measured `4 / 10 / 10` with the braille spinner at **`Unicode | Extended`**,
/// and that puts braille at the wrong rung: the engine's own `GlyphSet` says `Unicode` is *Unicode a
/// normal text font covers* and `Extended` is *braille, block elements, emoji, powerline*. A
/// terminal that promised the middle rung and got braille renders tofu, which is the one failure the
/// ladder exists to prevent.
///
/// So the middle rung is the **quadrant blocks**, which are block elements and are a four-position
/// orbit — a different construction from the ASCII rotating line rather than a re-spelling of it —
/// and braille stays at the top. The counts are `4 / 4 / 10` and the **ladders** are three distinct
/// tables, which is what [`constructions`] counts and what makes `spinner`'s row a **3**. That is
/// `plot`'s shape one family over and not `meter`'s: braille is what a spinner spends its 256 states
/// a cell on, so `Extended != Unicode` here where a horizontal bar's ladder has them equal.
pub const LADDERS: [&[&str]; 3] = [
    // `|/-\`, and the backslash is one reason this is a table and none of the twenty `Glyph`
    // meanings. Two states a cell, so a rotating line is what ASCII has.
    &["|", "/", "-", "\\"],
    // U+2596..U+259D, the quadrant blocks: an orbiting dot. Four positions, block elements, and
    // nothing here is braille.
    &["\u{2596}", "\u{2598}", "\u{259D}", "\u{2597}"],
    // The ten-frame braille spinner. Eight dots rotating in pairs, which is what 256 states a cell
    // buys and what neither rung below can spell.
    &[
        "\u{280b}", "\u{2819}", "\u{2839}", "\u{2838}", "\u{283c}", "\u{2834}", "\u{2826}",
        "\u{2827}", "\u{2807}", "\u{280f}",
    ],
];

/// **The ladder at a declared repertoire.**
///
/// The rung is found by position in [`RUNGS`] rather than by naming a `GlyphSet` variant, because a
/// component file in this crate may not spell one —
/// `crate::gates::tests::no_component_here_names_a_glyph_set_and_none_has_a_private_missing_table`
/// is the scan, and `RUNGS` is the public const that exists so a caller that needs to name a rung
/// has one place to do it from.
#[must_use]
pub fn ladder(set: GlyphSet) -> &'static [&'static str] {
    let rung = RUNGS
        .iter()
        .position(|r| *r == set)
        .unwrap_or(RUNGS.len() - 1);
    LADDERS[rung]
}

/// **How many distinct ladders there are: three.** [`crate::INVENTORY`]'s `constructions` for
/// `spinner`, derived from the shipped table rather than asserted in a comment.
///
/// `chart`, `plot`, `meter` and `sparkline` each derive theirs the same way, and until components
/// this table shipped, `spinner`'s was the one literal in that column that nothing
/// checked — so a ladder changing shape would have moved no number anywhere.
#[must_use]
pub fn constructions() -> usize {
    let mut seen: Vec<&'static [&'static str]> = Vec::new();
    for set in RUNGS {
        let l = ladder(set);
        if !seen.contains(&l) {
            seen.push(l);
        }
    }
    seen.len()
}

/// **What a spinner keeps across frames: an anchor, and nothing a clock has to move.**
///
/// # Stored state may be an anchor, never a phase
///
/// `Collapsing` and `Expanding` are refused because *a transition state has to be stored, which
/// means the machine can be found halfway between two states with no clock running*. Read as *a
/// component may not store anything a clock moves* that would forbid a spinner, and it is not that
/// rule — [`crate::disclose::Collapse`] is the proof, since it stores a `Tween` across frames.
///
/// What separates the two is what the stored thing **is**:
///
/// - an **anchor** is a value the current state is recoverable from at any `now`. There is no state
///   *between* two states, because the state **is** a function of `now`, so a machine that has been
///   asleep for an hour computes the same answer as one that has been drawing at sixty hertz;
/// - a **phase** is only meaningful relative to a frame that already ran, and a machine holding one
///   *can* be found halfway with no clock running — which is the sentence exactly.
///
/// A spinner asks for **strictly less** than the collapsible already ships: [`ANCHOR_BYTES`] against
/// [`crate::disclose::TWEEN_BYTES`], with no target, no `from` and nothing to land on. Components
/// the prototype is where it was measured and the decision is recorded beside it.
///
/// # And the clock is the frame's
///
/// The anchor is the component's; the **clock** is not. `Ctx::now` is the frame's own sampled
/// instant, and every component drawn from it agrees about what time it is —
/// `tests::no_component_body_in_this_crate_samples_its_own_clock` is the scan that says no body here
/// reaches for `Instant::now()` instead. The refusal is a measurement rather than a taste:
/// `Driver::pin_clock` is the entire test regime of this workspace, and a component that samples its
/// own clock is invisible to it, so *every screen it appears on* loses the ability to advance time.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct SpinState {
    /// The anchor. `None` when it is not spinning — **which is also how it stops**: there is nothing
    /// to land on and no tween to run out, so a stopped spinner asks for nothing on the very next
    /// frame.
    steps: Option<Steps>,
}

/// **What the anchor costs**, which is one `Instant` and one `Duration`.
///
/// Quoted against [`crate::disclose::TWEEN_BYTES`] by
/// `tests::the_anchor_is_smaller_than_the_tween_the_collapsible_already_ships`. The niche pays for
/// the `Option`, so the field costs what the anchor does.
pub const ANCHOR_BYTES: usize = size_of::<Option<Steps>>();

impl SpinState {
    /// A stopped spinner.
    #[must_use]
    pub const fn new() -> SpinState {
        SpinState { steps: None }
    }

    /// **Start spinning at `now`, one ladder frame every `per`.**
    ///
    /// A zero `per` is **static and asks for nothing**, which is the arrangement one component
    /// over: `Collapse::set` takes a `Duration` and a zero one steps rather than tweening. There is
    /// no theme bit to read instead — `Distinction` has ten entries and none of them is *motion is
    /// visible*, `Fade` being about whether a run of intermediate colours survives quantisation — so
    /// **The motion switch is the caller's `Duration`**, exactly as the collapsible's is.
    pub const fn start(&mut self, now: Instant, per: Duration) {
        self.steps = if per.is_zero() {
            None
        } else {
            Some(Steps::new(now, per))
        };
    }

    /// Stop. **Nothing lands and nothing runs out**, so the next frame asks for nothing.
    pub const fn stop(&mut self) {
        self.steps = None;
    }

    /// Whether it is spinning.
    #[must_use]
    pub const fn spinning(&self) -> bool {
        self.steps.is_some()
    }

    /// **Which ladder frame is showing at the frame's `now`.**
    #[must_use]
    pub fn index(&self, now: Instant, frames: usize) -> usize {
        let n = frames.max(1) as u64;
        self.steps.map_or(0, |s| s.cycle(now, n)) as usize
    }

    /// **When it wants the next frame**, `None` when it wants none.
    ///
    /// From the **anchor** and never from `now`, which is what keeps a screen that woke late on the
    /// grid it started on: `Steps::next_at` is the closed form and `vitui_runtime::anim` measures
    /// what the other spelling costs — 352 steps in thirty seconds against 375.
    #[must_use]
    pub fn wake(&self, now: Instant) -> Option<Instant> {
        self.steps.map(|s| s.next_at(now))
    }
}

/// [`spinner`]'s options.
///
/// A `Default` struct, never a required builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SpinOpts {
    /// The role the ladder frame is drawn in.
    pub mark: Role,
    /// The role the label beside it is drawn in.
    pub label: Role,
}

impl Default for SpinOpts {
    fn default() -> SpinOpts {
        SpinOpts {
            mark: Role::Title,
            label: Role::Body,
        }
    }
}

/// **A one-cell ladder frame that advances on the frame's clock, and a label beside it.**
///
/// **Hostile axes:** none.
///
/// The twenty-ninth row of the freeze and the last to be built. It declares no region — a
/// spinner is a pure drawer, as [`crate::text::text`] with no interest is — and it **partitions its
/// rectangle**: the mark, the label, the pad, and a run for every row below the first.
///
/// # It owns an anchor and it does not own a clock
///
/// [`SpinState`] is the anchor and `Ctx::now` is the clock. See [`SpinState`] for the rule and what
/// it costs, and the measurement that refuses the other arm is recorded with the decision.
///
/// # It asks only if the draw put a cell on the screen
///
/// *Undrawn* is already answered by the caller culling — the same mechanism a closed
/// [`crate::disclose::collapsible`] and an off-viewport row use, since the body is not called. A
/// **clipped** spinner is the case that is not the same: it runs, writes nothing, and can still ask.
/// Both screens look identical and one of them keeps the terminal awake, so the rule is one line and
/// [`defective::spinner_asking_always_into`] is the arm it replaced.
///
/// ```
/// use std::time::Duration;
/// use vitui_components::indicate::{SpinState, spinner};
/// use vitui_runtime::Rect;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(24, 1).expect("a sink attaches");
/// let mut st = SpinState::new();
/// driver.frame(|cx| {
///     // The anchor is the frame's `now`, which is the only clock a component may read.
///     st.start(cx.now(), Duration::from_millis(80));
///     let resp = spinner(cx, Rect::new(0, 0, 24, 1), &st, "indexing");
///     assert_eq!(resp.rect.w, 24);
/// });
/// assert!(st.spinning());
/// // Stopping is the whole of it: nothing lands, so the next frame asks for nothing.
/// st.stop();
/// assert!(!st.spinning());
/// ```
#[track_caller]
pub fn spinner(cx: &mut Ctx<'_, '_>, area: Rect, st: &SpinState, label: &str) -> Response {
    spinner_with(cx, area, st, label, &SpinOpts::default())
}

/// [`spinner`], with the options spelled out.
#[track_caller]
pub fn spinner_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &SpinState,
    label: &str,
    opts: &SpinOpts,
) -> Response {
    spinner_into(&mut Direct, cx, area, st, label, opts)
}

/// **[`spinner`], drawing through an [`Ink`] so a counter can see every cell.**
///
/// `#[track_caller]` all the way up, and here it is load-bearing twice over: `Ctx::id` mints from
/// `Location::caller()`, so two spinners at two call sites are two widgets; and the runaway a
/// spinning screen produces is blamed on the **application's** line rather than on this file, which
/// is `vitui_runtime::anim`'s own note about the attribute running in the right direction.
#[track_caller]
pub fn spinner_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &SpinState,
    label: &str,
    opts: &SpinOpts,
) -> Response {
    ask_into(ink, cx, area, st, label, opts, Ask::WhenWritten)
}

/// **When a spinner asks for its next frame.** The shipped rule and the arm it replaced.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum Ask {
    /// **The rule.** Ask only if the draw put a cell on the screen.
    #[default]
    WhenWritten,
    /// Ask whenever the anchor says it is spinning, written or not. [`defective`]'s.
    Always,
    /// Ask through `Ctx::deadline`, which names no widget. [`defective`]'s.
    WithoutTheId,
}

/// The one body all three arms share, so the negative cases are one field away from the shipped one
/// rather than a second implementation.
#[track_caller]
fn ask_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &SpinState,
    label: &str,
    opts: &SpinOpts,
    ask: Ask,
) -> Response {
    let id = cx.id();
    let now = cx.now();
    let asked = |cx: &mut Ctx<'_, '_>, written: bool| {
        let wants = match ask {
            Ask::WhenWritten => written,
            Ask::Always | Ask::WithoutTheId => true,
        };
        if !wants {
            return;
        }
        let Some(at) = st.wake(now) else { return };
        match ask {
            // **`deadline_for` and not `deadline`**, because the id buys the census: twelve
            // spinners on a screen asked through `Ctx::deadline` answer `Id::ROOT` twelve times,
            // and a screen that is awake cannot then be asked which widget is keeping it awake.
            Ask::WhenWritten | Ask::Always => cx.deadline_for(id, at),
            Ask::WithoutTheId => cx.deadline(at),
        }
    };

    if area.is_empty() {
        asked(cx, false);
        return Response::inert(id, area);
    }

    let frames = ladder(cx.theme().glyphs());
    let mark = if st.spinning() {
        frames[st.index(now, frames.len())]
    } else {
        // Stopped, and it is still one cell: a blank would make the row jump the frame the work
        // finishes, and `Glyph`'s own rule is *no spelling blank*.
        frames[0]
    };

    let theme = cx.theme();
    let (mark_paint, label_paint) = (theme.paint(opts.mark), theme.paint(opts.label));

    // **The rectangle is a partition** (spec §2): the mark, the label padded to what is left, and a
    // run for every row below the first.
    let mut written = u32::from(ink.text(cx, area.x, area.y, mark, mark_paint));
    if area.w > 1 {
        written += u32::from(ink.pad_to(cx, area.x + 1, area.y, label, area.w - 1, label_paint));
    }
    for row in 1..i32::from(area.h) {
        written += u32::from(ink.run(cx, area.x, area.y + row, " ", area.w, label_paint));
    }

    asked(cx, written > 0);
    Response::inert(id, area)
}

/// **The two spellings a spinner may not have, kept runnable rather than described.**
///
/// Both draw the identical picture, which is the whole reason they are here: the difference between
/// each of them and the shipped body is invisible on the rendered surface and visible only in what
/// the frame **asks for**.
pub mod defective {
    use super::{Ask, Ink, Rect, Response, SpinOpts, SpinState, ask_into};
    use vitui_runtime::Ctx;

    /// **Asks whether or not the draw wrote a cell.**
    ///
    /// A clipped spinner runs, writes nothing and — on this arm — still registers a deadline, so a
    /// screen stays awake for a widget the clip took away. The shipped rule is *ask only if the draw
    /// put a cell on the screen*; the two screens are cell for cell identical.
    #[track_caller]
    pub fn spinner_asking_always_into<I: Ink>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        st: &SpinState,
        label: &str,
        opts: &SpinOpts,
    ) -> Response {
        ask_into(ink, cx, area, st, label, opts, Ask::Always)
    }

    /// **Asks through `Ctx::deadline`, which names no widget.**
    ///
    /// The attribution is the same either way — both spellings record the caller's line — so what
    /// this arm loses is the **census**: `WakeLedger::asked_by` answers zero for the spinner's own
    /// id, and a screen with twelve spinners cannot be asked which one is spinning.
    #[track_caller]
    pub fn spinner_asking_without_its_id_into<I: Ink>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        st: &SpinState,
        label: &str,
        opts: &SpinOpts,
    ) -> Response {
        ask_into(ink, cx, area, st, label, opts, Ask::WithoutTheId)
    }
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

    /// **Criterion 1: `spinner` is three constructions, and the number is derived.**
    ///
    /// The prototype left `constructions: 2` as an *argument* — every other
    /// multi-construction row asserts `row.constructions == distinct(...)` against a shipped table
    /// and `spinner`'s table was on a branch, so nothing in the tree would have noticed the ladder
    /// changing shape. This is that derivation, and running it moved the number.
    #[test]
    fn a_spinner_is_three_constructions_and_the_ladder_is_its_own() {
        let counts: Vec<usize> = RUNGS.iter().map(|&set| ladder(set).len()).collect();
        assert_eq!(counts, vec![4, 4, 10], "the frame counts, rung by rung");
        assert_eq!(constructions(), 3);
        let row = INVENTORY
            .iter()
            .find(|c| c.id == "spinner")
            .expect("the freeze has a `spinner`");
        assert_eq!(row.constructions as usize, constructions());
        assert_eq!(row.tier, Tier::Three);
        // **The column stays empty, and that is the other half of *the ladder is its own*.** A
        // `Glyph` is one lookup; a ladder is `n` spellings that differ from each other.
        assert!(row.glyphs.is_empty());

        // **The count alone does not decide it, and that is why `constructions` compares the
        // tables.** Two of the three rungs have four frames and they are two different ladders —
        // read off the counts the answer would be 2, which is exactly the number ticket 42
        // reported.
        let by_count: std::collections::BTreeSet<usize> = counts.iter().copied().collect();
        assert_eq!(by_count.len(), 2, "the reading that gives the wrong answer");

        // **Every spelling is one cell and no spelling is blank** — §16's two rules, true of all
        // three ladders and deciding none of them.
        for &set in &RUNGS {
            let l = ladder(set);
            for frame in l {
                assert_eq!(
                    vitui_runtime::layout::text::width(frame),
                    1,
                    "`{frame}` is not one cell"
                );
                assert!(!frame.trim().is_empty(), "a blank spelling");
            }
            let distinct: std::collections::BTreeSet<&&str> = l.iter().collect();
            assert_eq!(distinct.len(), l.len(), "a ladder repeats a frame");
        }

        // **And braille is at the top rung and nowhere below it**, which is ticket 42's correction:
        // the engine's `GlyphSet` says `Unicode` is *Unicode a normal text font covers*, so a
        // braille frame there is tofu on a terminal that kept its promise.
        let braille = |l: &[&str]| {
            l.iter()
                .any(|f| f.chars().any(|c| ('\u{2800}'..='\u{28ff}').contains(&c)))
        };
        assert!(!braille(ladder(RUNGS[0])));
        assert!(!braille(ladder(RUNGS[1])));
        assert!(braille(ladder(RUNGS[2])));
    }

    /// **The anchor is smaller than the tween the collapsible already ships.**
    ///
    /// What makes *a component may own an anchor* a statement about a mechanism already in the
    /// crate rather than a new permission: `spinner` asks for less in bytes **and** in kind — no
    /// target, no `from`, and nothing to land on.
    #[test]
    fn the_anchor_is_smaller_than_the_tween_the_collapsible_already_ships() {
        // Through a `let`, so clippy sees a comparison of two values rather than of two constants.
        let (spinner, collapsible) = (ANCHOR_BYTES, crate::disclose::TWEEN_BYTES);
        assert_eq!(
            spinner, 32,
            "an `Instant` and a `Duration`, and the niche pays for the Option"
        );
        assert_eq!(
            collapsible, 40,
            "the tween slot the collapsible carries as a field"
        );
        assert!(spinner < collapsible, "{spinner} against {collapsible}");
    }

    /// **A stopped spinner asks for nothing on the next frame, and a zero period never asked.**
    ///
    /// The half `collapsible` cannot have: a tween has somewhere to land, so *frames to quiet*
    /// is a cadence. A spinner has no transient at all, so it is a **count**, and the count is one.
    #[test]
    fn a_stopped_spinner_is_quiet_at_once_and_a_zero_period_never_asked() {
        let now = Instant::now();
        let mut st = SpinState::new();
        st.start(now, Duration::from_millis(80));
        assert!(st.spinning());
        assert!(st.wake(now).is_some());
        st.stop();
        assert_eq!(st.wake(now), None, "one frame to quiet, and no transient");

        let mut zero = SpinState::new();
        zero.start(now, Duration::ZERO);
        assert!(!zero.spinning(), "a zero `Duration` is static");
        assert_eq!(zero.wake(now), None);
        // And it is still one cell, because `Glyph`'s own rule is *no spelling blank* and a row
        // that lost a cell the moment the work finished would jump.
        assert_eq!(zero.index(now, 10), 0);
    }

    /// **The anchor answers the same question the same way, however long the screen was asleep.**
    ///
    /// *The state is a function of `now`*, which is the whole of the rule — and the next step is on
    /// the anchor's grid rather than `now + per`, which is what a screen that woke late keeps.
    #[test]
    fn the_answer_is_a_function_of_now_and_the_grid_is_the_anchors() {
        let now = Instant::now();
        let mut st = SpinState::new();
        st.start(now, Duration::from_millis(80));
        assert_eq!(st.index(now, 10), 0);
        assert_eq!(st.index(now + Duration::from_millis(240), 10), 3);
        // Asleep for an hour, and the answer is the same one a screen drawing at sixty hertz has.
        let hour = now + Duration::from_secs(3_600);
        assert_eq!(st.index(hour, 10), st.index(hour, 10));
        assert_eq!(st.index(hour, 4), (3_600_000 / 80) % 4);
        // Late by 15 ms, and the next step is still on the grid the anchor set.
        assert_eq!(
            st.wake(now + Duration::from_millis(95)),
            Some(now + Duration::from_millis(160))
        );
    }

    /// **A clipped spinner writes nothing and asks for nothing, and the defective arm asks anyway.**
    ///
    /// The first of the two gated rules, and the reason it needs a runnable arm rather
    /// than a paragraph: **the two screens are cell for cell identical**. What separates them is one
    /// deadline, so a comparison of pictures reports them the same and only the ledger disagrees.
    ///
    /// *Undrawn* is deliberately not the subject. A caller that culls never calls the body, which is
    /// the same mechanism a closed `collapsible` and an off-viewport row use, so it is already
    /// answered and answering it again here would be measuring the harness.
    #[test]
    fn a_clipped_spinner_asks_for_nothing_and_the_defective_arm_asks_anyway() {
        let per = Duration::from_millis(80);
        // A rectangle with no cells in it: the component runs and every verb is a no-op.
        let clipped = Rect::new(0, 0, 0, 1);
        let visible = Rect::new(0, 0, 12, 1);

        let asks = |area: Rect, defective: bool| -> (usize, u64) {
            let mut driver = Driver::headless(12, 1).expect("a sink cannot fail to attach");
            let mut st = SpinState::new();
            let mut pen = Pen::over(Canvas::new(12, 1));
            let mut id = None;
            driver.frame(|cx| {
                st.start(cx.now(), per);
                let resp = if defective {
                    defective::spinner_asking_always_into(
                        &mut pen,
                        cx,
                        area,
                        &st,
                        "working",
                        &SpinOpts::default(),
                    )
                } else {
                    spinner_into(&mut pen, cx, area, &st, "working", &SpinOpts::default())
                };
                id = Some(resp.id);
            });
            let wakes = driver.inspect().wakes();
            (wakes.line_count(), wakes.asked_by(id.expect("a response")))
        };

        // Visible: both arms ask, once, and the id is on the ask.
        assert_eq!(asks(visible, false), (1, 1), "a visible spinner asks");
        assert_eq!(asks(visible, true), (1, 1));
        // Clipped: the shipped rule is silent and the defective arm keeps the terminal awake.
        assert_eq!(
            asks(clipped, false),
            (0, 0),
            "a clipped spinner wrote no cell and asked for a frame"
        );
        assert_eq!(
            asks(clipped, true),
            (1, 1),
            "the negative arm no longer asks, so the rule is gated against nothing"
        );

        // **And the two clipped screens are the same picture**, which is why the rule cannot be
        // read off a surface.
        let shot = |defective: bool| {
            let mut driver = Driver::headless(12, 1).expect("a sink cannot fail to attach");
            let mut st = SpinState::new();
            let mut pen = Pen::over(Canvas::new(12, 1));
            driver.frame(|cx| {
                st.start(cx.now(), per);
                if defective {
                    defective::spinner_asking_always_into(
                        &mut pen,
                        cx,
                        clipped,
                        &st,
                        "working",
                        &SpinOpts::default(),
                    );
                } else {
                    spinner_into(&mut pen, cx, clipped, &st, "working", &SpinOpts::default());
                }
            });
            pen.into_canvas()
        };
        assert_eq!(shot(false).diff(&shot(true)).cells, 0);
    }

    /// **The ask names the widget, and the defective arm names nobody.**
    ///
    /// The second rule. `Ctx::deadline` and `Ctx::deadline_for` attribute **the same way** — both
    /// record the caller's line — so what the id buys is the *census*, and a screen with three
    /// spinners on it is where the difference is visible: `asked_by` answers per widget, and three
    /// asks through `deadline` leave the census empty while every line is attributed correctly.
    #[test]
    fn three_spinners_name_three_widgets_and_the_defective_arm_names_none() {
        let per = Duration::from_millis(80);
        let census = |defective: bool| -> (usize, usize, u64) {
            let mut driver = Driver::headless(30, 3).expect("a sink cannot fail to attach");
            let mut st = SpinState::new();
            let mut pen = Pen::over(Canvas::new(30, 3));
            let mut first = None;
            driver.frame(|cx| {
                st.start(cx.now(), per);
                for row in 0..3 {
                    let area = Rect::new(0, row, 30, 1);
                    // **Keyed**, because one call in one loop is one `Location::caller()` — the trap
                    // `Ctx::id` documents, and the one `crate::gallery` meets per tile.
                    let resp = cx.with_key(u64::try_from(row).unwrap_or(0), |cx| {
                        if defective {
                            defective::spinner_asking_without_its_id_into(
                                &mut pen,
                                cx,
                                area,
                                &st,
                                "working",
                                &SpinOpts::default(),
                            )
                        } else {
                            spinner_into(&mut pen, cx, area, &st, "working", &SpinOpts::default())
                        }
                    });
                    if row == 0 {
                        first = Some(resp.id);
                    }
                }
            });
            let wakes = driver.inspect().wakes();
            (
                wakes.census_len(),
                wakes.line_count(),
                wakes.asked_by(first.expect("a response")),
            )
        };

        // Three widgets, one line — the line is the caller's own call site, which is one line
        // inside one loop, and the ids are three because the loop keys them.
        assert_eq!(census(false), (3, 1, 1));
        // The defective arm attributes the same line and names nobody at all.
        assert_eq!(
            census(true),
            (0, 1, 0),
            "the negative arm names a widget, so the census is gated against nothing"
        );
    }

    /// **A spinner writes every cell of the rectangle it was handed**, at every size from
    /// 1x1 to 17x5.
    #[test]
    fn a_spinner_partitions_every_rectangle_it_is_handed() {
        let per = Duration::from_millis(80);
        for w in 1..=17u16 {
            for h in 1..=5u16 {
                let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
                let mut st = SpinState::new();
                let mut tally = Tally::new();
                driver.frame(|cx| {
                    st.start(cx.now(), per);
                    spinner_into(
                        &mut tally,
                        cx,
                        Rect::new(0, 0, w, h),
                        &st,
                        "indexing the workspace",
                        &SpinOpts::default(),
                    );
                });
                let cells = u64::from(w) * u64::from(h);
                assert_eq!(tally.writes(), cells, "{w}x{h}: writes");
                assert_eq!(tally.distinct(), cells, "{w}x{h}: a cell written twice");
            }
        }
    }

    /// **No component body in this crate samples its own clock.**
    ///
    /// The scan this owes, and it is `crate::order`'s arrangement: an absence has no
    /// expression, so what keeps it true is a source scan rather than a type. The rule is *the clock
    /// is the frame's* — `Ctx::now` — and the edit that would break it is somebody reaching for
    /// `Instant::now()` because it is in scope in every crate.
    ///
    /// # The population is a **function**, not a file, and that is what makes it a join
    ///
    /// Fifty-six library-half `Instant::now()` calls live in this crate's *instruments* — every one
    /// of them a `let started = Instant::now()` in a timing report or an event fixture's timestamp —
    /// and six of those are inside `collect.rs`, which is a component module. A scan by **file**
    /// would therefore have to carry a growing exception list, and an exception list that grows is
    /// the shape this crate keeps finding defects behind.
    ///
    /// What separates a report from a draw is the **`Ctx`**: a function that takes one is drawing a
    /// frame and has the frame's clock in its hand, and a function that does not is a report and may
    /// read the machine's. So the scan walks each component module's library half, finds every `fn`
    /// item, and reports the ones that both name a `Ctx` in their signature and spell
    /// `Instant::now()` in their body. Watched in both directions over fixtures, because a scan
    /// whose needle has quietly stopped matching reports every module clean.
    #[test]
    fn no_component_body_in_this_crate_samples_its_own_clock() {
        let root = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        let mut modules: Vec<String> = crate::INVENTORY
            .iter()
            .filter_map(|c| c.module())
            .map(|m| format!("{m}.rs"))
            .collect();
        modules.sort_unstable();
        modules.dedup();
        assert_eq!(modules.len(), 10, "the modules the freeze homes a row in");
        // **The media family is the stated exception, and it is two files.** `crate::media` is *no
        // row of the freeze at all* — §14's own *no v1 component* — so a population derived from
        // the freeze's `families` column reaches none of its six drawers, and the chrome under it
        // is not at a family module's top level either. A rule about what draws that skipped seven
        // drawers because a column has no row for them is a rule with a hole in it, which is
        // `crate::order`'s own recorded defect met from the other side.
        modules.push("media.rs".to_owned());
        modules.push("media/player.rs".to_owned());
        assert_eq!(
            modules.len(),
            12,
            "ten homing modules and the media family's two"
        );

        let mut sampling = Vec::new();
        for m in &modules {
            let source =
                std::fs::read_to_string(root.join(m)).unwrap_or_else(|e| panic!("{m}: {e}"));
            for name in clock_samplers(&source) {
                sampling.push(format!("{m}: {name}"));
            }
        }
        assert_eq!(
            sampling,
            Vec::<String>::new(),
            "a component body reads `Instant::now()`. The clock is the frame's — `Ctx::now` — and \
             a component that samples its own is invisible to `Driver::pin_clock`, which is the \
             entire test regime of this workspace: every screen it appears on loses the ability to \
             advance time"
        );

        // **Both directions**, or a scan that has stopped finding `fn` reports every module clean.
        let hostile =
            "pub fn spin_into(cx: &mut Ctx<'_, '_>) {\n    let now = Instant::now();\n}\n";
        assert_eq!(clock_samplers(hostile), vec!["spin_into".to_owned()]);
        // A report takes no `Ctx` and may read the machine's clock — which is what fifty-six calls
        // in this crate's instruments are.
        let report =
            "pub fn cost(rounds: u32) -> u128 {\n    let started = Instant::now();\n    0\n}\n";
        assert!(clock_samplers(report).is_empty());
        // A drawing function that does not sample is not reported either.
        let shipped = "pub fn draw(cx: &mut Ctx<'_, '_>) {\n    let now = cx.now();\n}\n";
        assert!(clock_samplers(shipped).is_empty());
        // And a commented mention is not a call, which is `crate::dense::declares`'s own rule.
        let quoted =
            "pub fn draw(cx: &mut Ctx<'_, '_>) {\n    // never Instant::now()\n    let _ = 1;\n}\n";
        assert!(clock_samplers(quoted).is_empty());
    }

    /// Whether a line names a `Ctx`, which is what separates a draw from a report.
    fn names_a_ctx(line: &str) -> bool {
        line.contains("Ctx<") || line.contains("&mut Ctx")
    }

    /// The `fn` items of `source`'s library half that take a `Ctx` **and** spell `Instant::now()`.
    ///
    /// A function ends where the next one begins, which is enough here: the population is top-level
    /// items in a crate that runs `cargo fmt`, so a nested `fn` is attributed to its parent — and
    /// attributing a nested sampler to the body it is nested in is the answer this scan wants
    /// anyway.
    fn clock_samplers(source: &str) -> Vec<String> {
        let library = source
            .split_once("\n#[cfg(test)]\n")
            .map_or(source, |(head, _)| head);
        let mut out = Vec::new();
        let mut open: Option<(String, bool, bool)> = None;
        for line in library.lines() {
            let trimmed = line.trim_start();
            if let Some(rest) = trimmed
                .strip_prefix("pub fn ")
                .or_else(|| trimmed.strip_prefix("fn "))
                .or_else(|| trimmed.strip_prefix("pub(crate) fn "))
            {
                if let Some((name, ctx, sampled)) = open.take()
                    && ctx
                    && sampled
                {
                    out.push(name);
                }
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                    .collect();
                // **The declaration line is read, not skipped.** A one-line signature carries the
                // `Ctx` on the same line as the `fn`, so a scan that opened the item and moved on
                // reported every such body clean — which the hostile fixture below is watched
                // catching.
                open = Some((name, names_a_ctx(line), false));
                continue;
            }
            let Some((_, ctx, sampled)) = open.as_mut() else {
                continue;
            };
            if trimmed.starts_with("//") {
                continue;
            }
            // The signature may span lines, so `Ctx` is looked for in the whole body: a function
            // that mentions one is a function that has a frame in its hand.
            if names_a_ctx(line) {
                *ctx = true;
            }
            if line.contains("Instant::now()") {
                *sampled = true;
            }
        }
        if let Some((name, ctx, sampled)) = open
            && ctx
            && sampled
        {
            out.push(name);
        }
        out
    }

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
    /// repertoire, and the one stated exception is worth exactly one file.
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
    /// values, which is the number the column carries.
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
