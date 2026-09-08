//! Charts and plots: a series in cells, at whatever resolution the terminal allows.
//!
//! [`chart`] is the framed one — axes, labels, a plotting area — and [`plot`] is the plotting area
//! on its own. Both take a [`Series`] and a [`raster::PlotState`], and both draw through the same
//! rasteriser: the difference between them is the chrome, not the marks.
//!
//! # Examples
//!
//! ```
//! use vitui_components::chart::raster::PlotState;
//! use vitui_components::chart::{Series, plot};
//! use vitui_runtime::ctx::Driver;
//!
//! let data = Series::build(1_000, 2);
//! let mut state = PlotState::new();
//! let mut driver = Driver::headless(40, 10).expect("a sink attaches");
//!
//! driver.frame(|cx| {
//!     plot(cx, cx.area(), &data, &mut state);
//! });
//! ```
//!
//! # The marks degrade with the terminal
//!
//! A cell can carry one mark, four quadrants or eight braille dots depending on what the terminal
//! can draw, and the same data is rasterised into whichever is available. A chart therefore has
//! more resolution on a modern terminal and the same *shape* on an old one, with no branch in the
//! calling code.
//!
//! # A plot is not a picture
//!
//! There is no pixel path here and no image protocol: the marks are text, they compose with
//! everything else on the screen, and they cost what the cells they cover cost.

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["chart", "plot"];

pub mod axes;
pub mod raster;

use std::fmt::Write as _;

use vitui_runtime::data::Revision;
use vitui_runtime::{Ctx, Glyph, Id, Interest, Paint, Rect, Response, Rgb, Role, Theme};

use crate::ink::{Direct, Ink};
use axes::{decimals, gutter, nice_step, tick_count, tick_value};
use raster::{Kind, OWNER_THRESHOLD, PlotState, Range, Reach, SHARED, cluster, geom};

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

    /// **Append one sample to every series, and bump the revision.** The *edit*.
    ///
    /// This is the only mutating verb on the data, and it is where the second sentence lands: the
    /// frame costs the rectangle and **the edit costs the data**. A push bumps
    /// [`Series::revision`], which misses both memos of the chain — the range because the extremes
    /// may have moved, and the raster because the range is in its key — so a live series pays one
    /// fold a sample and a still one pays none at all.
    ///
    /// # Panics
    ///
    /// Panics when `samples` is not one value per series. A push that silently filled the shortfall
    /// would leave one series a sample behind the others for ever, which is a defect no counter can
    /// see.
    pub fn push(&mut self, samples: &[f32]) {
        assert_eq!(
            samples.len(),
            self.points.len(),
            "a push is one sample per series"
        );
        for (series, sample) in self.points.iter_mut().zip(samples) {
            series.push(*sample);
        }
        self.rev = Revision::fresh();
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
/// [`crate::series::Build`]'s arrangement and [`crate::listing::Volume`]'s, for the same reason: a
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
    /// *carried on both axes*, in the place a chart meets it: at sixteen colours `Danger`,
    /// `Warn` and `Ok` need not differ on the wire, so a threshold carried by a paint alone is
    /// invisible and a rule drawn in `HLine` is not.
    pub threshold_glyph: bool,
    /// **Whether series colours are role-derived rather than named.**
    ///
    /// The failure this exists to price: there are thirteen roles, and the three that read as series
    /// colours collapse to one at sixteen colours.
    pub role_series: bool,
    /// **The range, declared by the caller.**
    ///
    /// `None` is the decision: the gutter is computed from the **whole domain**, which is data, so
    /// it does not depend on the rectangle and there is no layout loop to solve. Over
    /// [`axes::AXIS_PAIRS`] pairs that is 0 oscillations and always 1 pass, against 464 for the
    /// window-scaled fixpoint.
    ///
    /// **A caller who insists on a window-scaled axis must declare the range** — the same shape of
    /// answer as reserved bars, and the same reason: the component cannot both let the data decide
    /// the axis and let the rectangle decide the data. A declared range is an input like any other,
    /// so it is in the raster's key and the range memo never runs.
    pub declared: Option<(f32, f32)>,
    /// **What the raster's memo is keyed on.** [`raster::KeyMode::Full`] is the only correct answer
    /// and the other two are here to be priced.
    pub key: raster::KeyMode,
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
            declared: None,
            key: raster::KeyMode::Full,
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
/// # `Theme::custom` needs a background, and the page is now readable
///
/// It takes two [`Rgb`], and the thirteen roles are `Paint`s rather than colours. There was no way
/// to ask what the body is painted on, so this derived the background from `Theme::is_dark` — the
/// one bit about the page a component could read — and substituted `Rgb::new(0, 0, 0)` for a dark
/// theme.
///
/// **The substitution was wrong on a real terminal and looked it.** Catppuccin Mocha's page is
/// `#1e1e2e`, not black, so every plotted cell went out as `48:5:16` while the text around it went
/// out as `48:5:235` — a black halo around every curve and every bar, on every dark theme this
/// library ships. It was invisible to the gates because none of them reads a background: the round
/// trip compares a replayed screen against the frame that produced it, and both carry the same
/// wrong colour.
///
/// [`Theme::page`] removes it. The two colours are still the caller's, which is what keeps the
/// obligations below true.
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
    theme.custom(Rgb::new(r, g, b), theme.page())
}

// ── the stand-in painter ─────────────────────────────────────────────────────────────────────────

/// **`chart`: bottom-anchored bars over one or more series.**
///
/// **Hostile axes:** `narrow`.
///
/// The overlap is **green at 300x80 and red at 60x20**, which is the one axis on which a
/// component's construction changes rather than its contents.
///
/// The shape — `fn(&mut Ctx, Rect, …) -> Response` — with the data and the plot's own state
/// in the argument list (rule 2) and an options struct that `Default`s (rule 3).
///
/// A cell is a **prefix** of its column, so its states are ordered and the sub-column axis is
/// unused. The ladder is **1 / 8 / 8**: block elements are the *Unicode* rung by `CONTEXT.md`'s own
/// definition of it, so the third repertoire rung buys a bar chart **nothing** — 0 cells differ
/// between Unicode and Extended on `crate::series`' screen.
///
/// ```
/// use vitui_components::chart::{Series, chart, raster::PlotState};
/// use vitui_runtime::ctx::Driver;
///
/// let data = Series::build(1_000, 2);
/// let mut state = PlotState::new();
/// let mut driver = Driver::headless(40, 12).expect("a sink attaches");
/// driver.frame(|cx| {
///     let response = chart(cx, cx.area(), &data, &mut state);
///     assert!(!response.hovered);
/// });
/// // The data was folded once, into a raster whose size is the rectangle.
/// assert_eq!(state.misses(), 1);
/// ```
#[track_caller]
pub fn chart(cx: &mut Ctx<'_, '_>, area: Rect, data: &Series, st: &mut PlotState) -> Response {
    chart_with(cx, area, data, st, &Opts::chart())
}

/// [`chart`], with the options spelled out. The [`Opts::kind`] is `chart`'s whatever is passed.
#[track_caller]
pub fn chart_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    data: &Series,
    st: &mut PlotState,
    opts: &Opts,
) -> Response {
    chart_into(&mut Direct, cx, area, data, st, opts)
}

/// **[`chart`], drawing through an [`Ink`] so a counter can see the verbs.**
///
/// The construction is forced here rather than trusted from `opts`: a caller cannot ask `chart` to
/// draw marks, which is what makes *`chart` is 2 constructions and `plot` is 3* a property of the
/// entry point rather than of the call site.
#[track_caller]
pub fn chart_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    data: &Series,
    st: &mut PlotState,
    opts: &Opts,
) -> Response {
    let id = cx.id();
    draw(
        ink,
        cx,
        area,
        id,
        &Opts {
            kind: Kind::Bars,
            ..*opts
        },
        data,
        st,
    )
}

/// **`plot`: arbitrary marks over one or more series.**
///
/// **Hostile axes:** `narrow`.
///
/// The same screen as [`chart`]'s at 60x20, and the mark ladder is what changes under it.
///
/// A cell is a **set** of positions, so its states are a power set and both sub-axes are live. The
/// ladder is **1x1 / 2x2 / 2x4** — 2, 16 and 256 states — and the third rung is `plot`'s alone,
/// buying exactly one bit of vertical resolution and paying for it in series colour on the cells
/// where two series meet.
///
/// ```
/// use vitui_components::chart::{Series, plot, raster::PlotState};
/// use vitui_runtime::ctx::Driver;
///
/// let data = Series::build(100_000, 2);
/// let mut state = PlotState::new();
/// let mut driver = Driver::headless(40, 12).expect("a sink attaches");
/// driver.frame(|cx| { let _ = plot(cx, cx.area(), &data, &mut state); });
/// let first = state.raster().bytes();
/// // A second frame folds nothing: the frame costs the rectangle and the edit costs the data.
/// driver.frame(|cx| { let _ = plot(cx, cx.area(), &data, &mut state); });
/// assert_eq!(state.misses(), 1);
/// assert_eq!(state.raster().bytes(), first);
/// ```
#[track_caller]
pub fn plot(cx: &mut Ctx<'_, '_>, area: Rect, data: &Series, st: &mut PlotState) -> Response {
    plot_with(cx, area, data, st, &Opts::plot())
}

/// [`plot`], with the options spelled out. The [`Opts::kind`] is `plot`'s whatever is passed.
#[track_caller]
pub fn plot_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    data: &Series,
    st: &mut PlotState,
    opts: &Opts,
) -> Response {
    plot_into(&mut Direct, cx, area, data, st, opts)
}

/// **[`plot`], drawing through an [`Ink`] so a counter can see the verbs.** See [`chart_into`].
#[track_caller]
pub fn plot_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    data: &Series,
    st: &mut PlotState,
    opts: &Opts,
) -> Response {
    let id = cx.id();
    draw(
        ink,
        cx,
        area,
        id,
        &Opts {
            kind: Kind::Marks,
            ..*opts
        },
        data,
        st,
    )
}

/// **The body both components are, and the `Kind` is the only thing between them.**
///
/// The data arrives as `&Series` and is **never iterated here** — every loop over `n` is inside
/// `Raster::build`, which runs on a memo miss.
///
/// It writes a **partition** of `area`: the gutter, the axis column, the body and the axis row are
/// disjoint and together they are every cell.
fn draw<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    id: Id,
    o: &Opts,
    data: &Series,
    st: &mut PlotState,
) -> Response {
    let response = cx.interact(id, area, Interest::HOVER);
    if area.w < 6 || area.h < 3 {
        return response;
    }

    let g = geom(o.kind, cx.theme().glyphs());
    // **The range memo, and the raster memo whose key contains its answer.** Two memos and not one:
    // the gutter below narrows the rectangle, and a resize must invalidate the raster without
    // refolding the data.
    let dom = match o.declared {
        // A declared range is the caller's, so nothing is folded at all — which is what makes
        // *declare the range* a real answer rather than a slogan.
        Some((y0, y1)) => raster::Domain { y0, y1 }.padded(),
        None => st.range(data.revision(), o.range, data.points()),
    };
    let plot_h = area.h - 1;
    // **The gutter comes from the domain and the domain is the whole series'**, so it does not
    // depend on the plotting width and there is no edge to iterate. See `Opts::declared`.
    let gut = gutter(dom, plot_h).min(area.w - 2);
    let plot_w = area.w - gut;
    // What the list instinct would compute: the index window a virtualised collection uses.
    let cull = (0usize, usize::from(plot_h).min(data.len()));
    st.refresh_keyed(
        o.key,
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
    // **The memo's value and the row buffer at once**, which is why the two are separate fields
    // rather than one struct behind one borrow: `Memo::get` has just filled the first, and the
    // second is the thing the body spells into.
    let body = Body {
        kind: o.kind,
        geom: g,
        series: series_paints,
        dim,
        threshold: threshold_row.map(|row| {
            (
                row,
                if o.threshold_glyph {
                    hline.chars().next().unwrap_or('-')
                } else {
                    ' '
                },
                threshold_paint,
            )
        }),
    };
    let at = Rect::new(area.x + i32::from(gut), area.y, plot_w, plot_h);
    let (raster, row) = st.raster_and_row();
    body_into(ink, cx, at, raster, row, &body);

    // ── the axis row: the blank gutter, the corner, then the rule ────────────────────────────────
    let y = area.y + i32::from(plot_h);
    let _ = ink.run(cx, area.x, y, " ", label_w, dim);
    let _ = ink.run(cx, area.x + i32::from(label_w), y, corner, 1, axis);
    let _ = ink.run(cx, area.x + i32::from(gut), y, hline, plot_w, axis);
    response
}

/// **What the body loop needs that is not the raster**: the construction, the paints and the
/// threshold's row.
///
/// A struct rather than eight parameters, and it exists because [`crate::indicate::sparkline`] is
/// this loop with the chrome deleted. The claim about that component is *`chart` at a small
/// rectangle, no axes, no gutter, no axis loop* — and a sparkline that had its own copy of the loop
/// would make the claim untestable, which is `crate::ink`'s argument one file over: a gate written
/// against a copy of the code tests the copy.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Body {
    /// Bars or marks.
    pub kind: Kind,
    /// The sub-cell ladder the raster was built at.
    pub geom: raster::Geom,
    /// The six series paints, by owner.
    pub series: [Paint; SERIES_RGB.len()],
    /// What an empty cell is painted in.
    pub dim: Paint,
    /// The threshold's row, the cluster it is drawn with and its paint — `None` when there is none.
    pub threshold: Option<(u16, char, Paint)>,
}

/// **The body: one row at a time, split into runs by who owns the cell.**
///
/// `at` is where the body's own top-left is, in the caller's coordinates; the raster's `w` and `h`
/// are the extent. Nothing here knows about a gutter, an axis row or a tick — which is exactly what
/// makes it shareable with a component that has none of the three.
pub(crate) fn body_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    at: Rect,
    raster: &raster::Raster,
    row: &mut String,
    b: &Body,
) {
    for y in 0..at.h {
        let mut run_owner = 0u8;
        let mut run_x = 0u16;
        row.clear();
        for x in 0..at.w {
            let (bits, owner) = raster.at(x, y);
            let (ch, own) = if bits != 0 {
                (cluster(b.kind, b.geom, bits), owner)
            } else if let Some((_, ch, _)) = b.threshold.filter(|&(t, _, _)| t == y) {
                // **The threshold's second axis.** A rule the terminal can draw whatever the palette
                // did, which is what *carried on both axes* asks for.
                (ch, OWNER_THRESHOLD)
            } else {
                (' ', 0u8)
            };
            if own != run_owner && !row.is_empty() {
                let paint = paint_of(run_owner, &b.series, b.dim, b.threshold_paint());
                let _ = ink.text(cx, at.x + i32::from(run_x), at.y + i32::from(y), row, paint);
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
            let paint = paint_of(run_owner, &b.series, b.dim, b.threshold_paint());
            let _ = ink.text(cx, at.x + i32::from(run_x), at.y + i32::from(y), row, paint);
        }
    }
}

impl Body {
    /// The threshold's paint, or the dim one when there is no threshold — which is unreachable,
    /// because no cell is owned by `OWNER_THRESHOLD` unless the threshold put it there.
    fn threshold_paint(&self) -> Paint {
        self.threshold.map_or(self.dim, |(_, _, p)| p)
    }
}

/// The paint a run of cells owned by `owner` is drawn in.
///
/// **A shared cell can carry only one colour**, which is the third rung's price: a braille cell has
/// one `Paint` for all eight dots where a quadrant carries a foreground *and* a background. The
/// per-cell quadrant fallback that would recover it is named and not built.
fn paint_of(owner: u8, series: &[Paint; 6], dim: Paint, threshold: Paint) -> Paint {
    match owner {
        0 => dim,
        OWNER_THRESHOLD => threshold,
        SHARED => series[0],
        n => series[(usize::from(n) - 1) % series.len()],
    }
}

/// **Why there is no `downsample`, and the pair that keeps it that way.**
///
/// The rasteriser is a **union**, and a union is idempotent: two points in one cell set the union of
/// their bits, so a column's extrema survive *by construction*. Downsampling is therefore not a
/// cheaper way to get this answer — it is a different, wrong answer at the same frame cost, and
/// [`crate::series`] prices all three spellings on the rendered surface: 5 546 cells for culling to
/// the visible index window, 3 431 for an axis range from a stride sample and 2 953 for every
/// *n*-th point.
///
/// The hostile half names the absent item **by path**, and the twin beside it names the item that
/// must exist by the same path — because a lone `compile_fail` also passes when the module has been
/// renamed, and `E0433` for *the module is gone* and `E0425` for *the function is not there* are the
/// same diagnostic to the mechanism.
///
/// ```compile_fail,E0425
/// // There is no downsampling path, and there is not going to be one.
/// let _ = vitui_components::chart::raster::downsample(&[1.0f32], 10);
/// ```
///
/// ```compile_fail,E0425
/// let _ = vitui_components::chart::raster::stride_sample(&[1.0f32], 10);
/// ```
///
/// ```compile_fail,E0425
/// let _ = vitui_components::chart::raster::every_nth(&[1.0f32], 10);
/// ```
///
/// **Protects:** [`raster::Raster::build`] — the twin, which is the only half of a pair that holds.
///
/// ```
/// use vitui_components::chart::raster::{Domain, Kind, RUNGS, Raster, Reach, geom};
///
/// let mut raster = Raster::empty();
/// let geometry = geom(Kind::Marks, RUNGS[2]);
/// let domain = Domain { y0: 0.0, y1: 1.0 };
/// raster.build(
///     4,
///     4,
///     Kind::Marks,
///     geometry,
///     domain,
///     &[vec![0.0, 1.0, 0.5]],
///     Reach::Mapped,
///     (0, 4),
/// );
/// assert_eq!(raster.bytes(), 32);
/// ```
#[derive(Debug)]
pub struct WhyThereIsNoDownsample;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::series::{self, Build, H, NARROW_H, NARROW_W, W};
    use raster::{KeyMode, RUNGS, geom};
    use std::collections::BTreeSet;
    use std::path::PathBuf;
    use vitui_runtime::ColorDepth;
    use vitui_runtime::ctx::Driver;

    /// **The memo is a chain, and a resize invalidates the raster and not the range.**
    ///
    /// Two memos and not one, which is the whole of the sentence as counts: the range is folded
    /// **once** over two rectangles, and the raster twice. Folding them together would make a resize
    /// re-read the data, which is the cost the split exists to avoid.
    #[test]
    fn the_memo_is_a_chain_and_a_resize_invalidates_the_raster_and_not_the_range() {
        let data = Series::build(50_000, 2);
        let mut state = raster::PlotState::new();
        for (w, h) in [(80u16, 24u16), (80, 24), (60, 24), (60, 24)] {
            let mut driver = Driver::headless(w, h).expect("a sink attaches");
            driver.frame(|cx| {
                let _ = plot(cx, cx.area(), &data, &mut state);
            });
        }
        assert_eq!(
            state.range_misses(),
            1,
            "the range was refolded by a resize, so the two memos are one memo"
        );
        assert_eq!(
            state.misses(),
            2,
            "the raster survived a resize, so its key has no rectangle in it"
        );

        // And the same rectangle twice folds nothing at all, which is the hit half.
        let before = state.misses();
        let mut driver = Driver::headless(60, 24).expect("a sink attaches");
        driver.frame(|cx| {
            let _ = plot(cx, cx.area(), &data, &mut state);
        });
        assert_eq!(state.misses(), before);
    }

    /// **A raster keyed on the revision alone survives a resize, recomputes less often, and is
    /// wrong.**
    ///
    /// The fourth false green, and the one whose detector has to be the surface: the miss counter
    /// points the **wrong way**, because the narrower key folds fewer times. What a reader sees is a
    /// smaller, older plot inside a bigger rectangle — no panic, no counter moved, fewer writes.
    #[test]
    fn a_raster_keyed_on_the_revision_alone_is_wrong_and_folds_less_often() {
        let wrong = series::resize_wrong_cells(KeyMode::DataOnly);
        assert!(wrong > 0, "the narrow key was not caught by the surface");
        assert_eq!(
            series::resize_wrong_cells(KeyMode::Full),
            0,
            "the full key is not clean, so the comparison is measuring something else"
        );

        let (narrow_misses, _) = series::resize_misses(KeyMode::DataOnly);
        let (full_misses, _) = series::resize_misses(KeyMode::Full);
        assert!(
            narrow_misses < full_misses,
            "the wrongly-keyed memo folded {narrow_misses} times against {full_misses}, so the \
             counter is not pointing the wrong way and a cheaper gate would exist"
        );

        // **Gradual and not binary**: the data-and-rectangle key survives a repertoire swap and a
        // change of domain, and it is clean across this resize. A gate that only tried the narrowest
        // spelling would report the middle one healthy.
        assert_eq!(series::resize_wrong_cells(KeyMode::DataRect), 0);
    }

    /// **There is no downsampling path in this crate, and each absent spelling is named by path.**
    ///
    /// The `compile_fail` fences are on [`WhyThereIsNoDownsample`]; this is the half that catches
    /// the spelling coming back **`pub(crate)`**, which is how a deleted helper actually returns —
    /// `crate::frame`'s deleted focus ring is the precedent, and its own scan reported *itself* on
    /// its first run, which is why the needles here are assembled from fragments.
    #[test]
    fn no_downsampling_spelling_exists_anywhere_in_this_crate() {
        fn walk(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries {
                let path = entry.expect("a readable entry").path();
                if path.is_dir() {
                    walk(&path, out);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    out.push(path);
                }
            }
        }

        let needles = [
            format!("{}{}", "down", "sample"),
            format!("{}_{}", "stride", "sample"),
            format!("{}_{}", "every", "nth"),
        ];
        let src = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        let mut files = Vec::new();
        walk(&src, &mut files);
        assert!(files.len() > 20, "the walk found only {}", files.len());

        for path in &files {
            let source = std::fs::read_to_string(path).expect("a readable source file");
            for (n, line) in source.lines().enumerate() {
                let code = line.trim_start();
                if code.starts_with("//") {
                    continue;
                }
                for needle in &needles {
                    assert!(
                        !code.contains(needle.as_str()),
                        "{}:{} carries `{needle}`. The rasteriser is a union and a union is \
                         idempotent, so a column's extrema survive by construction — a \
                         downsampling path is a different, wrong answer at the same frame cost",
                        path.display(),
                        n + 1
                    );
                }
            }
        }

        // The other direction, through the same predicate: a scan that has stopped scanning also
        // reports nothing.
        let hostile = format!("    fn {}(xs: &[f32]) -> Vec<f32> {{}}", needles[0]);
        assert!(hostile.contains(needles[0].as_str()));
    }

    /// **`chart` is two constructions and `plot` is three, from `CONTEXT.md`'s rungs.**
    ///
    /// Not from a runtime table: the `constructions` column is derived from `CONTEXT.md`, and
    /// the freeze does not wait on the runtime. The count is *distinct sub-cell geometries over the
    /// three rungs*, which is what a construction is here.
    #[test]
    fn chart_is_two_constructions_and_plot_is_three() {
        let distinct = |kind: Kind| -> usize {
            RUNGS
                .iter()
                .map(|set| {
                    let g = geom(kind, *set);
                    (g.sx, g.sy)
                })
                .collect::<BTreeSet<_>>()
                .len()
        };
        assert_eq!(distinct(Kind::Bars), 2, "chart");
        assert_eq!(distinct(Kind::Marks), 3, "plot");

        // And the freeze says the same thing about the same two rows, which is the join that makes
        // this a count over the map rather than over this file.
        for row in crate::INVENTORY {
            match row.id {
                "chart" => assert_eq!(usize::from(row.constructions), distinct(Kind::Bars)),
                "plot" => assert_eq!(usize::from(row.constructions), distinct(Kind::Marks)),
                _ => {}
            }
        }
    }

    /// **The entry points force their construction**, so `chart` cannot be asked to draw marks.
    #[test]
    fn the_entry_points_force_their_construction() {
        let data = Series::build(1_000, 2);
        let mut bars = raster::PlotState::new();
        let mut marks = raster::PlotState::new();
        let mut driver = Driver::headless(40, 12).expect("a sink attaches");
        driver.frame(|cx| {
            let area = cx.area();
            // Both are handed the *other* construction's options, and both ignore it.
            let _ = chart_with(cx, area, &data, &mut bars, &Opts::plot());
            let _ = plot_with(cx, area, &data, &mut marks, &Opts::chart());
        });
        assert_eq!(bars.raster().kind(), Kind::Bars);
        assert_eq!(marks.raster().kind(), Kind::Marks);
    }

    /// **The gutter is computed from the whole domain, and a window-scaled axis must declare its
    /// range.**
    ///
    /// The decoupling as a property of the component rather than of the sweep: the domain a frame
    /// draws against does not depend on the rectangle, so the gutter cannot feed back into itself.
    /// A caller who wants the other thing declares it, and then the range is an input like any
    /// other — the range memo never runs at all.
    #[test]
    fn the_gutter_comes_from_the_whole_domain_and_a_window_scaled_axis_must_declare_its_range() {
        let data = Series::build(20_000, 2);
        let mut wide = raster::PlotState::new();
        let mut narrow = raster::PlotState::new();
        let mut driver = Driver::headless(200, 40).expect("a sink attaches");
        driver.frame(|cx| {
            let _ = plot(cx, cx.area(), &data, &mut wide);
        });
        let mut driver = Driver::headless(40, 40).expect("a sink attaches");
        driver.frame(|cx| {
            let _ = plot(cx, cx.area(), &data, &mut narrow);
        });
        assert_eq!(
            wide.domain(),
            narrow.domain(),
            "the domain moved with the rectangle, which is the edge §13 cuts"
        );

        // A declared range is the caller's, and nothing is folded for it.
        let mut declared = raster::PlotState::new();
        let opts = Opts {
            declared: Some((0.0, 200.0)),
            ..Opts::plot()
        };
        let mut driver = Driver::headless(200, 40).expect("a sink attaches");
        driver.frame(|cx| {
            let _ = plot_with(cx, cx.area(), &data, &mut declared, &opts);
        });
        assert_eq!(
            declared.range_misses(),
            0,
            "a declared range folded the data anyway"
        );
        assert_eq!(declared.domain().y0, 0.0);
        assert_ne!(declared.domain(), wide.domain());
    }

    /// **The series palette is six `Theme::custom`s and this crate holds no `Style` literal.**
    ///
    /// The one legitimate use of `custom`, counted rather than scattered: there are as many series
    /// as the data says and thirteen roles, so no theme can enumerate them. A `Style` literal would
    /// not be legitimate and there are none — read off the source, because a rule enforced only by
    /// review is a rule the next edit undoes silently.
    #[test]
    fn the_series_palette_is_six_customs_and_the_crate_holds_no_style_literal() {
        assert_eq!(SERIES_RGB.len(), 6);
        assert_eq!(
            SERIES_RGB.iter().collect::<BTreeSet<_>>().len(),
            6,
            "two series share a colour"
        );

        fn walk(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries {
                let path = entry.expect("a readable entry").path();
                if path.is_dir() {
                    walk(&path, out);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    out.push(path);
                }
            }
        }
        // **Assembled, so this file does not match its own needle** — and it is the *re-export
        // path* rather than a shape or a bare name.
        //
        // Two looser spellings were tried and both are wrong for the same reason: `crate::form`
        // carries its own private `enum Style`, a form's word for a wrapping mode, and it has
        // nothing to do with the engine's. `Style {` reported its declaration and `Style::`
        // reported its ten call sites. **The claim is about the type this crate could reach for and
        // must not**, and the only way to reach it is through the runtime's re-export — so that is
        // the needle, and the two false positives are recorded here rather than excepted by file.
        let needle = concat!("vitui_run", "time::Style");
        let src = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        let mut files = Vec::new();
        walk(&src, &mut files);
        let mut offenders = Vec::new();
        for path in &files {
            let source = std::fs::read_to_string(path).unwrap_or_default();
            if source
                .lines()
                .map(str::trim_start)
                .any(|line| !line.starts_with("//") && line.contains(needle))
            {
                offenders.push(path.to_string_lossy().into_owned());
            }
        }
        assert_eq!(offenders, Vec::<String>::new());
        assert!(format!("let s: {needle} = Default::default();").contains(needle));

        // **And `Theme::custom` is called from two files and one line of each**, which is what
        // *counted, not scattered* means as a number. A second **line** is a second palette.
        //
        // # The second file is an earlier pass's, and it is the exception the design states
        //
        // This assertion read `["chart.rs"]` for one ticket, and widening it is a deliberate edit
        // rather than a loosening — refinement 3, the same procedure row 26 uses for the
        // repertoire branch. The sentence is *a picture is the first caller whose every cell
        // is outside the theme*: a chart calls `custom` once to build a **palette** of six series
        // colours, and `crate::media` calls it once to build a **pixel**, a QR's four and a
        // barcode's two. Those are different claims, and the thing this gate protects — one
        // palette, decided in one place — is untouched by the second, because a picture has no
        // palette to decide and a symbol's two colours are a specification rather than a choice.
        //
        // **It moved from `picture.rs` to `media.rs` with an earlier pass**, which is the screen
        // handing the verb back to the component that owes the census.
        //
        // The line count is what keeps the exception at its argument. Two files, one calling line
        // each: a third line anywhere is a second palette again, whichever file it is in.
        // **The shipped half, split at `#[cfg(test)]`**, which the review caught this scan not
        // doing: a test that spells the verb is *measuring* the palette rather than adding one, and
        // firing a gate whose message says "a second palette" at one would be the failure naming
        // the wrong thing. `crate::picture`'s own `.fill(` scan splits the same way.
        let custom = concat!(".cus", "tom(");
        let calls = |path: &PathBuf| {
            std::fs::read_to_string(path)
                .unwrap_or_default()
                .split("#[cfg(test)]")
                .next()
                .unwrap_or_default()
                .lines()
                .map(str::trim_start)
                .filter(|line| !line.starts_with("//") && line.contains(custom))
                .count()
        };
        // **Sorted, and the CI runner is what asked for it.** `walk` yields `read_dir` order, which
        // is the filesystem's: this assertion held on one machine and failed on the Linux runner
        // with `[("media.rs", 1), ("chart.rs", 1)]` against the same pair the other way round.
        // The single-element version this replaces was order-independent by accident, which is
        // exactly the kind of accident a second element removes.
        let mut sites: Vec<(String, usize)> = files
            .iter()
            .filter(|path| calls(path) > 0)
            .map(|path| {
                (
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("?")
                        .to_owned(),
                    calls(path),
                )
            })
            .collect();
        sites.sort();
        assert_eq!(
            sites,
            vec![("chart.rs".to_string(), 1), ("media.rs".to_string(), 1)],
            "`Theme::custom` is called from somewhere new. §16 asks for one palette decided in one \
             place; the two lines this permits are `chart::series_paint`'s and \
             `crate::media::custom`'s, and the second is §14's stated exception — a picture's \
             cells, a QR's two and a barcode's two are outside the theme by construction and there \
             is no palette for a second call to disagree with"
        );
    }

    /// **A threshold is carried on both axes, and a paint alone dies at sixteen colours.**
    ///
    /// The rule in the place a chart meets it. At [`ColorDepth::Ansi16`] the shipped palette need
    /// not keep `Danger`, `Warn` and `Ok` apart on the wire, and a component reading `false` from
    /// `Theme::roles_differ_on_wire` owes a **second axis** — a rule the terminal can draw — and
    /// never a darker colour.
    #[test]
    fn a_threshold_is_carried_on_both_axes_and_a_paint_alone_dies_at_sixteen_colours() {
        let build = Build::correct().tier(ColorDepth::Ansi16);
        let with = series::render(build, (W, H), 200_000);
        let without = series::render(build.both(|o| o.threshold_glyph = false), (W, H), 200_000);
        let diff = series::split_diff(&with, &without, series::whole((W, H)));
        assert!(
            diff.cluster > 0,
            "the glyph axis changed no cluster, so the threshold is carried on one axis after all"
        );
        assert_eq!(
            diff.rows, 1,
            "a threshold is one row, and it moved {} of them",
            diff.rows
        );

        // **The palette's own answer, and half of the sentence does not reproduce.**
        //
        // the design states `roles_differ_on_wire(Danger, Warn)` and `(Warn, Ok)` **both false** at
        // sixteen colours. On the shipped Catppuccin Mocha palette the first is **true** and only
        // the second is false — asserted as measured rather than bent to fit, and the palette was
        // deliberately not swapped to make the old number, which is the move an earlier pass
        // recorded refusing for the same reason.
        //
        // The obligation is unchanged by which of the pairs collapses: **one** collapsing pair is
        // enough to owe a second axis, and the 215 cells above are that axis being drawn.
        let theme = build.theme();
        assert!(
            theme.roles_differ_on_wire(Role::Danger, Role::Warn),
            "§13's first pair now collapses on this palette too, which is a bigger claim than the \
             one this test was written for — say so rather than widening the assertion"
        );
        assert!(
            !theme.roles_differ_on_wire(Role::Warn, Role::Ok),
            "the shipped palette keeps the whole traffic light apart at sixteen colours, so this \
             screen is no longer where §16's rule is met — say so rather than asserting a number \
             about a palette that is not this one"
        );
    }

    /// **Role-derived series collapse to one colour at sixteen, and the signature is style-only.**
    ///
    /// There are thirteen roles and as many series as the data says. The three that read as series
    /// colours are `Danger`, `Warn` and `Ok`, and at sixteen colours they need not differ on the
    /// wire — so the two arms differ **by style and not by cluster**, which is the signature of a
    /// colour-only distinction dying.
    #[test]
    fn role_derived_series_collapse_and_the_difference_is_style_only() {
        let build = Build::correct().tier(ColorDepth::Ansi16);
        let named = series::render(build, (W, H), 200_000);
        let roled = series::render(build.both(|o| o.role_series = true), (W, H), 200_000);
        let diff = series::split_diff(&named, &roled, series::whole((W, H)));
        assert_eq!(
            diff.cluster, 0,
            "the two arms drew different characters, so this is not a colour-only difference"
        );
        assert!(
            diff.style_only() > 0,
            "the two arms are identical, so `Theme::custom` bought nothing"
        );
    }

    /// **A steady frame allocates nothing at either size, and the two buffers are why.**
    ///
    /// The totals live in `examples/series_numbers.rs`, which installs the probe; this is the
    /// property that makes them zero — both buffers are fields of [`raster::PlotState`], and each
    /// was measured costing one allocation a frame as a local.
    #[test]
    fn the_draw_path_holds_its_buffers_rather_than_allocating_them() {
        let state = raster::PlotState::new();
        assert!(state.row_capacity() > 0 && state.label_capacity() > 0);
    }

    /// **The frame at both sizes is the screen's, and it is flat.** `crate::series`' own numbers,
    /// asserted here so `chart`'s file carries the claim its component is about.
    #[test]
    fn the_frame_is_flat_at_both_sizes() {
        for (size, writes) in [
            ((W, H), series::WRITES),
            ((NARROW_W, NARROW_H), series::NARROW_WRITES),
        ] {
            for (points, shape) in series::across_volumes(Build::correct(), size) {
                assert_eq!(shape.writes, writes, "at {points} points");
                assert_eq!(shape.misses, 0, "a steady frame folded the data");
                assert!(shape.verbs <= shape.writes);
            }
        }
    }
}
