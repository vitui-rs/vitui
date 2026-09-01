//! **The picture screen: a full-screen picture at three colour depths, and the two gated traps.**
//!
//! Components ticket 29. Spec §14, §21. This is the screen the media family's scene is a scene
//! *of*, and it takes the shape every other one on this map has — [`crate::dense`],
//! [`crate::listing`], [`crate::grid`], [`crate::forest`], [`crate::area`], [`crate::accordion`],
//! [`crate::document`], [`crate::clusters`], [`crate::popup`], [`crate::series`] and
//! [`crate::wheel`]: one `Build` whose fields are the arms, so a comparison is the same screen with
//! one thing changed.
//!
//! | scene | what it decides |
//! |---|---|
//! | a full-screen picture at three colour depths | 24 000 customs; the horizontal distinctions the colour axis takes away |
//!
//! # The axis with a floor of nothing
//!
//! Every other component on this map degrades to a **worse drawing of itself**: a label truncates,
//! a plot loses sub-rows, a border becomes `+`. A picture degrades to a **description** of itself,
//! because there is nothing to re-pair the lost distinctions with — §16's whole mechanism is
//! *carry the distinction on the other axis*, and a picture has no other axis. [`DISTINCTIONS`] is
//! that sentence as a count, and [`Build::tier`] at [`ColorDepth::None`] is its floor: **0 of
//! 23 920**, exactly, which no other screen in this crate can produce.
//!
//! # `Extended == Unicode`, and it is arithmetic rather than a table
//!
//! A plot puts one **bit** per sub-cell and spells the bitmask with a glyph, so braille's 2x4 buys
//! it four more bits than the quadrants. A picture puts a **colour** in every sub-cell, and a cell
//! carries exactly two colours whatever glyph it holds — so the eight dots of a braille cell have
//! one foreground between them and the rung buys **nothing**. [`crate::media::sub_rows`] is 1, 2, 2
//! against [`crate::chart::raster::geom`]'s 1, 8, 8, and it is *derived from* that ladder rather
//! than branching a second time. See it.
//!
//! [`Ladder::Braille`] is the arm that makes the equality fire in both directions: the plot's own
//! ladder applied to a picture draws a **different screen** at Extended than at Unicode, so a gate
//! reading `Extended == Unicode` on the correct build is reading something.
//!
//! # No `fill` is available at any size, and that is measured rather than argued
//!
//! [`Ctx::fill`](vitui_runtime::Ctx::fill) takes one [`Paint`](vitui_runtime::Paint) for a
//! rectangle. A photograph's rectangle has as many paints as it has cells, and [`Shape::adjacent_equal`] is **0 of 47 620**
//! adjacent cell pairs — so the largest rectangle one paint would serve is one cell, and `fill` is
//! not merely unused but unavailable. The gradient is printed beside it, because the claim is about
//! the picture and not about the verb.
//!
//! # The two traps, both of which produced a comfortable number
//!
//! **A picture translated by a whole cell row is a scroll**, and the two halves of that disagree by
//! a factor of eighty. [`Build::shift`] is the translation: the cell half is **24 000 of 24 000
//! cells change value**, against a still picture's 0, and a repaint priced by the cell would
//! therefore price it at the whole screen. The byte half says **11 731 bytes — 1.25% of a full
//! repaint** — because the engine's scroll pre-pass prices the rows the shift exposed, and 11 731
//! is one row's own 11 715 plus sixteen bytes of scroll sequence. Both halves are needed and the
//! byte half is the one that was unreadable until runtime architecture issue 34.
//!
//! **A QR module must be square and a cell is not.** [`Modules`] is the matrix and
//! [`readback`] is the readback: the drawn cells are decoded back into modules through the four
//! paints a QR spends, and compared against the matrix that produced them. One module per cell is
//! **not worse, invalid** — [`module_aspect`] prices it — and [`Pairing::Inverted`] is the
//! one-character mistake the readback exists to catch.
//!
//! # What the screen could not ask, and both answers came back as verbs
//!
//! This is the section that used to say *filed rather than worked around*, and it is kept in that
//! shape because the sequence is the finding. Runtime architecture issue 34 answered both halves.
//!
//! **Two of this ticket's measurements are bytes on the wire, and no crate above the engine could
//! read a byte the engine wrote.** [`vitui_runtime::Config`] was reachable and `Clock`, `Output`,
//! `Overrides`, `WidthSource` and `InputConfig` were not in `vitui_runtime::line::ENGINE_NAMES` at
//! all, so the only headless door was `Driver::headless`, whose tier is hard-coded to truecolor and
//! whose sink is a `Vec` nobody can reach — issue 22's own rule (*a name a consumer can write but
//! not build is a barrier wearing a re-export's clothes*) arriving on `Config` itself. All five are
//! re-exported; [`bytes_over`] and [`bytes_by_shift`] are what that bought, and both halves of the
//! door mattered — the sink for the bytes, the `Overrides` for the tier they are a tier's bytes of.
//!
//! **One measurement is a quantiser, and the route to it was a contrivance that worked.** §14 asks
//! how many of a picture's horizontal distinctions survive at sixteen colours;
//! [`vitui_runtime::Theme::custom`] said in as many words that the component calling it *owes a
//! branch on the terminal's own capabilities* and gave it no verb to ask with —
//! `roles_differ_on_wire` compares two of the thirteen **roles**, and a picture's cells are outside
//! the theme by construction. [`wire_differ`] was the way through, at one whole theme construction
//! per colour pair: `Roles::from_palette` maps `base08` and `base0A` **verbatim** onto
//! `Role::Danger` and `Role::Warn`, over the same ground and with the same attributes. It is now
//! `Theme::colours_differ_on_wire` — the same question asked of two colours — and the counts that
//! come back are the same ones, which is the corroboration a contrivance is owed. **The verb alone
//! bought nothing**: the theme has to be hoisted out of the call as well, because
//! `Theme::default().resolve(tier)` in the body is the contrivance's own cost with the free part
//! removed. 773 ns, 811 ns and 57 ns are the three arms — see [`wire_differ`].

use std::collections::HashMap;
use std::fmt::Write as _;
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant};

use vitui_runtime::ctx::Driver;
use vitui_runtime::theme::{CATPPUCCIN_MOCHA, Density};
use vitui_runtime::{
    Clock, ColorDepth, Config, Ctx, GlyphSet, Output, Overrides, Rect, Rgb, Theme,
};

use crate::chart::raster::{Geom, Kind, RUNGS, cluster, geom};
use crate::counters::Tally;
use crate::ink::{Direct, Ink};
use crate::media::{self, Census, Modules, Palette, PictureOpts, Pixels, QrPaints};
use crate::obligations::Verdict;
use crate::runner::{Canvas, Pen};

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The screen's width. §14's own, and §20's full-screen class.
pub const W: u16 = 300;

/// The screen's height.
pub const H: u16 = 80;

/// How many cells that is. **24 000**, which is §14's custom-call census in the same breath.
pub const CELLS: u32 = W as u32 * H as u32;

/// How many horizontal adjacencies a `W x H` screen has. `(W - 1) * H`.
pub const HORIZONTAL_PAIRS: u64 = (W as u64 - 1) * H as u64;

/// How many adjacencies of either direction it has. `(W - 1) * H + W * (H - 1)`.
pub const ADJACENCIES: u64 = HORIZONTAL_PAIRS + W as u64 * (H as u64 - 1);

/// **The three colour depths the scene is played at**, richest first.
///
/// [`ColorDepth::None`] is a fourth and is not one of them: §14's row is *three colour depths* and
/// the floor is asserted separately, because *every distinction gone* and *half of them gone* are
/// different claims and running them as one row would let the second hide inside the first.
pub const DEPTHS: [ColorDepth; 3] = [
    ColorDepth::TrueColor,
    ColorDepth::Indexed256,
    ColorDepth::Ansi16,
];

/// **What the picture is a picture of.**
///
/// Two, because §14's B/cell claim is *the source barely moves it* and a claim about two sources
/// needs two. The photograph is noise with a bounded dynamic range, which is what a photograph is
/// to a quantiser; the gradient is the smoothest frame anything could produce.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    /// A noisy photograph. No two adjacent cells agree.
    Photograph,
    /// A smooth gradient. Adjacent cells often agree, which is what makes it the other end.
    Gradient,
}

impl Source {
    /// The name a report prints.
    pub fn word(self) -> &'static str {
        match self {
            Source::Photograph => "photograph",
            Source::Gradient => "gradient",
        }
    }

    /// **The pixel at a sub-row, as `0xRRGGBB`.**
    ///
    /// Deterministic and cheap: a report and a gate must see the same picture, and a picture read
    /// from a file would put a fixture in the repository whose bytes nothing checks.
    pub fn pixel(self, x: u16, sub_y: u32) -> u32 {
        match self {
            // A 32-bit integer hash, three bytes of it taken as channels and squeezed into the
            // middle of the range. **The dynamic range is the point**: a photograph is not uniform
            // over the cube, and a source that were would make the sixteen-colour count a fact
            // about `rand` rather than about pictures.
            Source::Photograph => {
                let mut h =
                    u32::from(x).wrapping_mul(0x9e37_79b9) ^ sub_y.wrapping_mul(0x85eb_ca6b);
                h ^= h >> 15;
                h = h.wrapping_mul(0xc2b2_ae35);
                h ^= h >> 13;
                let band = |v: u8| u32::from(48 + (u16::from(v) * 160 / 255) as u8);
                (band((h >> 16) as u8) << 16) | (band((h >> 8) as u8) << 8) | band(h as u8)
            }
            // Two linear ramps and a constant. Smooth in both axes, so the quantiser's steps are
            // wide bands rather than noise.
            //
            // **`min` and not a mask, and the review caught the mask.** The denominator is the
            // sub-row count of the *half-block* rung, and two callers reach past it: `Build::shift`
            // walks off the bottom, and `Ladder::Braille` asks for four sub-rows a cell rather than
            // two. Masked, `sub_y = 159` is `0x00ff80` and `sub_y = 160` is `0x000080` — a hard
            // bright-to-black discontinuity in a source whose whole job is to be smooth, and one
            // that no gate would have shown because both of those arms play over the photograph.
            Source::Gradient => {
                let r = (u32::from(x) * 255 / u32::from(W - 1)).min(255);
                let g = (sub_y * 255 / (u32::from(H) * 2 - 1)).min(255);
                (r << 16) | (g << 8) | 0x80
            }
        }
    }
}

/// **Which half of the cell the foreground paints.**
///
/// `▀` is the **upper** half block, so a foreground is the top pixel and a background the bottom.
/// [`Pairing::Inverted`] is `▄` with the same paint — one character away, drawing the same cells
/// with the same two colours in the same order, and vertically mirrored inside every cell.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pairing {
    /// `▀`, the upper half. Correct.
    Upper,
    /// `▄`, the lower half. The one-character mistake.
    Inverted,
}

/// **Which ladder the cell's glyph is taken from.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ladder {
    /// A picture's own: one sub-column, [`crate::media::sub_rows`] sub-rows, two colours.
    Colour,
    /// **The plot's**, applied to a picture. Every sub-cell of the mark rung is set and the eight
    /// dots share one foreground, so the rung buys resolution the colours cannot carry — and the
    /// screen at Extended stops equalling the screen at Unicode.
    Braille,
}

/// **The screen, as one value with one thing changed at a time.**
///
/// The arrangement [`crate::dense`], [`crate::listing`] and [`crate::series`] all use, and its
/// reason: a second painter written against the alternative would be a gate testing a copy.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Build {
    /// The repertoire the theme is told about.
    pub glyphs: GlyphSet,
    /// The colour depth the theme is resolved for.
    pub depth: ColorDepth,
    /// What the picture is a picture of.
    pub source: Source,
    /// Which half the foreground paints.
    pub pairing: Pairing,
    /// Which ladder the glyph comes from.
    pub ladder: Ladder,
    /// Where the colours come from.
    pub palette: Palette,
    /// **Whole cell rows the picture is translated by.** The scroll trap.
    pub shift: u16,
}

impl Build {
    /// The correct screen: a photograph, the half block at the richest rung, truecolor, unshifted.
    pub fn correct() -> Build {
        Build {
            glyphs: RUNGS[2],
            depth: ColorDepth::TrueColor,
            source: Source::Photograph,
            pairing: Pairing::Upper,
            ladder: Ladder::Colour,
            palette: Palette::Custom,
            shift: 0,
        }
    }

    /// The same screen at another repertoire.
    #[must_use]
    pub fn at(mut self, glyphs: GlyphSet) -> Build {
        self.glyphs = glyphs;
        self
    }

    /// The same screen at another colour depth.
    #[must_use]
    pub fn tier(mut self, depth: ColorDepth) -> Build {
        self.depth = depth;
        self
    }

    /// The same screen of another source.
    #[must_use]
    pub fn of(mut self, source: Source) -> Build {
        self.source = source;
        self
    }

    /// The same screen translated by whole cell rows.
    #[must_use]
    pub fn shifted(mut self, rows: u16) -> Build {
        self.shift = rows;
        self
    }

    /// The theme this build asks for.
    pub fn theme(self) -> Theme {
        Theme::authored(&CATPPUCCIN_MOCHA, self.glyphs, Density::default()).resolve(self.depth)
    }

    /// The sub-cell geometry this build's glyph comes from.
    pub fn geometry(self) -> Geom {
        match self.ladder {
            Ladder::Colour => crate::media::picture_geom(self.glyphs),
            Ladder::Braille => geom(Kind::Marks, self.glyphs),
        }
    }
}

// **The census, the four QR paints and the module matrix all live in `crate::media` now**, because
// they are the component's obligation rather than the screen's: `Theme::custom`'s own documentation
// asks a component that calls it to publish its call census, and a screen that kept its own would be
// counting a copy. See `crate::media::Census`.

/// `0xRRGGBB` as an [`vitui_runtime::Rgb`].
fn rgb(v: u32) -> vitui_runtime::Rgb {
    vitui_runtime::Rgb::new(
        ((v >> 16) & 0xff) as u8,
        ((v >> 8) & 0xff) as u8,
        (v & 0xff) as u8,
    )
}

/// **The scene's source as a [`Pixels`]**, with the translation folded in.
///
/// A component is asked for a cell in the **picture's own** coordinates and is never told where on
/// the screen it landed, so [`Build::shift`] cannot be a component option and does not need to be:
/// a translation of the picture is a translation of the *source*, which is what a scroll is. The
/// offset is in sub-rows, which is why it is taken at [`Build::geometry`] — the braille arm samples
/// four a cell where the correct one samples two.
#[derive(Clone, Copy, Debug)]
pub struct Sampled {
    source: Source,
    offset: u32,
}

impl Sampled {
    /// The source this build draws, translated by its own shift.
    pub fn of(build: Build) -> Sampled {
        Sampled {
            source: build.source,
            offset: u32::from(build.shift) * u32::from(build.geometry().sy),
        }
    }
}

impl Pixels for Sampled {
    fn pixel(&self, x: u16, sub_y: u32) -> Rgb {
        rgb(self.source.pixel(x, sub_y + self.offset))
    }
}

/// The options this build asks the component for.
fn picture_opts(build: Build) -> PictureOpts {
    PictureOpts {
        palette: build.palette,
    }
}

/// **Draw the picture into `area`, through [`crate::media::picture_into`].**
///
/// One verb a cell, because there is nothing else available: a run needs one paint and a picture's
/// two adjacent cells do not share one.
///
/// # The screen is a dispatch and no longer a painter
///
/// Components ticket 29 wrote the loop here because `crate::media` carried no `pub fn picture(`;
/// ticket 30 declared it, so [`Build`]'s three defect axes become **which entry point the screen
/// calls**. [`Pairing::Inverted`] and [`Ladder::Braille`] are `crate::media::defective`'s arms —
/// the shipped body with one argument changed — and every figure this file measures is therefore a
/// measurement of the component.
pub fn picture_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    build: Build,
    census: &mut Census,
) {
    let src = Sampled::of(build);
    let opts = picture_opts(build);
    let _ = match (build.ladder, build.pairing) {
        (Ladder::Colour, Pairing::Upper) => media::picture_into(ink, cx, area, &src, &opts, census),
        (Ladder::Colour, Pairing::Inverted) => {
            media::defective::picture_inverted_into(ink, cx, area, &src, &opts, census)
        }
        (Ladder::Braille, _) => {
            media::defective::picture_braille_into(ink, cx, area, &src, &opts, census)
        }
    };
}

/// The side of a version-1 QR symbol, in modules.
pub const QR_SIDE: u16 = 21;

/// **A rectangle the symbol fits in at both rungs**, which is 21 rows and not 11.
///
/// At two modules a cell the symbol is 21x11; at one it is 21x21, and a rectangle sized for the
/// first **clips the second** — which is what [`crate::media::qr_into`] does with a rectangle too
/// small, and which the aspect arm needs not to be doing while it is being compared against the
/// pairing arm.
pub fn qr_area() -> Rect {
    Rect::new(3, 3, QR_SIDE, QR_SIDE)
}

/// **A version-1 symbol's shape**: three finder patterns, two timing tracks, and a payload.
///
/// Not an encoder — the payload is deterministic noise. What the scene needs of a QR is that its
/// modules are **square, two-valued and positioned**, and an encoder would put a second unchecked
/// fixture in the repository for no additional property.
///
/// # The fixture is the screen's and the type is the component's
///
/// [`Modules`] is `crate::media`'s, because a QR component takes a matrix its caller brings. This
/// function is here, because a symbol whose payload is a hash is a **fixture** and a component that
/// shipped one would be shipping a QR that encodes nothing.
pub fn v1_symbol() -> Modules {
    let n = QR_SIDE;
    let mut dark = vec![false; usize::from(n) * usize::from(n)];
    let set = |dark: &mut Vec<bool>, x: u16, y: u16, v: bool| {
        dark[usize::from(y) * usize::from(n) + usize::from(x)] = v;
    };
    // The payload first, so the fixed patterns overwrite it where they overlap.
    for y in 0..n {
        for x in 0..n {
            let h = u32::from(x).wrapping_mul(73_856_093) ^ u32::from(y).wrapping_mul(19_349_663);
            set(&mut dark, x, y, (h >> 7) & 1 == 1);
        }
    }
    for (ox, oy) in [(0, 0), (n - 7, 0), (0, n - 7)] {
        for dy in 0..7 {
            for dx in 0..7 {
                let edge = dx == 0 || dy == 0 || dx == 6 || dy == 6;
                let core = (2..=4).contains(&dx) && (2..=4).contains(&dy);
                set(&mut dark, ox + dx, oy + dy, edge || core);
            }
        }
    }
    for i in 8..n - 8 {
        set(&mut dark, i, 6, i % 2 == 0);
        set(&mut dark, 6, i, i % 2 == 0);
    }
    Modules::new(n, dark)
}

/// **Draw a QR symbol into `area`, through [`crate::media::qr_into`].**
///
/// [`Pairing::Inverted`] is `crate::media::defective::qr_inverted_into` — the same body with the
/// other half block — so the readback below is comparing the shipped component against the matrix
/// that produced it.
pub fn qr_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    build: Build,
    modules: &Modules,
    census: &mut Census,
) {
    let _ = match build.pairing {
        Pairing::Upper => media::qr_into(ink, cx, area, modules, census),
        Pairing::Inverted => media::defective::qr_inverted_into(ink, cx, area, modules, census),
    };
}

/// **Read the drawn cells back into modules and count the ones that disagree.**
///
/// # It models the terminal, which is the only reading that can catch the pairing
///
/// The first version of this decoded a cell's two modules from its **paint alone**, and
/// [`Pairing::Inverted`] passed it 0 of 441 — because the inverted build writes the same cells with
/// the same four paints and differs by one character. A paint is not a picture: what the terminal
/// puts on the screen is decided by the paint **and the glyph together**, and `▀` and `▄` swap
/// which half the foreground lands in.
///
/// So the decode is the terminal's: `▀` puts the foreground on top, `▄` puts it underneath, and a
/// space puts the background everywhere. That is a model of one glyph pair and not of a terminal in
/// general, which is why it is here rather than in [`crate::runner`] — `crate::runner::Canvas` is a
/// record of what was *written*, and this is the one screen where what was written and what is seen
/// are two different things.
///
/// A cell whose paint is not one of the four counts as a disagreement rather than being skipped: a
/// readback that skips what it cannot read is a readback that passes on a blank screen.
pub fn readback(canvas: &Canvas, area: Rect, build: Build, modules: &Modules) -> u32 {
    let theme = build.theme();
    let paints = QrPaints::new(&theme, &mut Census::default());
    let per_cell = u16::from(crate::media::sub_rows(build.glyphs));
    // **`try_from` and not `as`**, which the review caught: `area.x as u16` wraps a negative origin
    // to ~65 5xx, every `canvas.get` then misses, and the function reports 441 of 441 wrong for a
    // symbol that was drawn perfectly. A readback that answers *everything is wrong* about a
    // coordinate it could not convert is worse than one that stops.
    let x0 = u16::try_from(area.x).expect("a readback needs an on-screen origin");
    let y0 = u16::try_from(area.y).expect("a readback needs an on-screen origin");
    let upper_half = cluster(Kind::Marks, Geom { sx: 2, sy: 2 }, 0b0000_0011);
    let lower_half = cluster(Kind::Marks, Geom { sx: 2, sy: 2 }, 0b0000_1100);
    let mut wrong = 0u32;
    for m_y in 0..modules.side() {
        for m_x in 0..modules.side() {
            let row = m_y / per_cell;
            let Some(cell) = canvas.get(x0 + m_x, y0 + row) else {
                wrong += 1;
                continue;
            };
            let Some((fg_dark, bg_dark)) = paints.decode(cell.paint) else {
                wrong += 1;
                continue;
            };
            // What the terminal shows in the cell's two halves, from the glyph it was given.
            let (top, bottom) = match cell.cluster.chars().next() {
                Some(g) if g == upper_half => (fg_dark, bg_dark),
                Some(g) if g == lower_half => (bg_dark, fg_dark),
                // A space shows the background in both halves. Correct at the one-sub-row rung and
                // a loss of half the symbol at any other, which is what the aspect trap is.
                Some(' ') => (bg_dark, bg_dark),
                _ => {
                    wrong += 1;
                    continue;
                }
            };
            let seen = if per_cell == 1 || m_y % per_cell == 0 {
                top
            } else {
                bottom
            };
            if seen != modules.dark(m_x, m_y) {
                wrong += 1;
            }
        }
    }
    wrong
}

/// **A terminal cell's aspect, width over height, as a nominal 1:2.**
///
/// The number every argument about sub-cells is written against, and the one §14's `0.50` is.
pub const CELL_ASPECT_NOMINAL: f32 = 0.5;

/// **A terminal cell's aspect as a real font gives it**, 8.0 over 16.5 — Menlo at 14 pt, which is
/// what this machine's terminals ship.
pub const CELL_ASPECT_MEASURED: f32 = 8.0 / 16.5;

/// **A module's aspect when `per_cell` of them share a cell's height.**
///
/// One module per cell is the cell's own aspect; two is twice it, because the module is one cell
/// wide and half a cell tall. **Square is 1.0**, and the distance from it is the whole of what
/// *not worse, invalid* means: a QR reader looks for square modules.
pub fn module_aspect(per_cell: u16, cell_aspect: f32) -> f32 {
    cell_aspect * f32::from(per_cell.max(1))
}

// ── the shape of one frame ───────────────────────────────────────────────────────────────────────

/// **What one frame of the picture screen is**, as counters rather than as a picture.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shape {
    /// Cells written, from the engine's own column reports.
    pub writes: u64,
    /// Distinct cells touched. Equal to [`Shape::writes`] on every build here.
    pub distinct: u64,
    /// Drawing verbs. One a cell, and there is nothing else available.
    pub verbs: u64,
    /// Interactive regions. **Zero**: a picture declares nothing.
    pub regions: usize,
    /// What the draw spent.
    pub census: Census,
    /// **Adjacent cell pairs carrying the same value**, in both directions. The number that decides
    /// whether a `fill` exists at any size.
    pub adjacent_equal: u64,
}

/// A driver over a sink at this screen's size, with the build's theme seated.
fn driver_for(build: Build) -> Driver {
    let mut driver = Driver::headless(W, H).expect("a sink cannot fail to attach");
    driver.set_theme(build.theme());
    driver
}

// ── the wire, which is what runtime architecture issue 34 unblocked ──────────────────────────────

/// **A driver whose bytes land somewhere this crate can read, at the build's own tier.**
///
/// Runtime architecture issue 34, part 2. Until it resolved, `Driver::headless` was the only
/// headless door above the engine: its sink is a `Vec` moved into the engine and never returned, and
/// its tier is hard-coded to truecolor. Both halves mattered here — a byte count needs the sink and
/// a byte count *per tier* needs the override — and neither `Output`, `Clock`, `Overrides`,
/// `WidthSource` nor `InputConfig` was in `vitui_runtime::line::ENGINE_NAMES` at all, reachable or
/// not, so no `Config` this crate could build sent its bytes anywhere it could read.
///
/// **The tier is pinned on the `Overrides` and not only on the `Theme`.** `driver_for` seats
/// `build.theme()`, which narrows the *thirteen roles* and says nothing about what the engine will
/// quantise a `custom` paint into — and every cell of this screen is `custom`. A byte count taken
/// through a truecolor engine with a C16 theme is a truecolor byte count.
///
/// **Two `Tap`s share one buffer**: the one boxed into `Output::Sink` is the engine's writer and is
/// gone the moment it crosses, and the one returned is this crate's reader. That split is the shape
/// of the barrier — the engine *owns* its writer — rather than a way around it.
fn tapped_driver_for(build: Build) -> (Driver, Tap) {
    let wire = Tap(Arc::new(Mutex::new(Vec::new())));
    let driver = Driver::attach(
        Config {
            clock: Clock::Manual,
            output: Output::Sink(Box::new(Tap(Arc::clone(&wire.0)))),
            size: (W, H),
            overrides: Overrides {
                colors: Some(build.depth),
                ..Default::default()
            },
            ..Default::default()
        },
        build.theme(),
    )
    .expect("a configured sink cannot fail to attach");
    (driver, wire)
}

/// The buffer, on both sides of the seam: boxed into `Output::Sink` it is the engine's writer, and
/// held by [`tapped_driver_for`]'s caller it is the reader. A type rather than a `Vec` because the
/// box is moved in and nothing hands it back.
///
/// `Send` because `Output::Sink` asks for it — on a real clock the box crosses to the render thread
/// — and shared through an `Arc<Mutex<..>>` because *the engine owns the writer* is the invariant
/// that made this measurement unreachable in the first place. There is no `unsafe` here and none is
/// available: ADR 0034 is workspace-wide.
struct Tap(Arc<Mutex<Vec<u8>>>);

impl Tap {
    /// Bytes written so far, and the counter is **cumulative** — read as deltas, which is the trap
    /// this workspace has met more than once.
    fn len(&self) -> u64 {
        self.0.lock().expect("the tap is not poisoned").len() as u64
    }
}

impl std::io::Write for Tap {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0
            .lock()
            .expect("the tap is not poisoned")
            .extend_from_slice(b);
        Ok(b.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// **Bytes the engine wrote, per frame, over `frames` frames of one build.**
///
/// The still picture's shape: the first frame carries the whole screen and every frame after it
/// carries nothing, because damage is marked at write time and an equal write marks none. The
/// counter behind it is cumulative and is differenced here.
///
/// **`attach` writes before any frame does** — the alternate screen, the negotiation, the mode sets
/// — so the baseline is taken after the driver exists and before the first `frame`, and what is
/// returned is one number a frame and never a total.
pub fn bytes_over(build: Build, frames: u32) -> Vec<u64> {
    assert!(frames > 0, "a per-frame figure needs a frame");
    let (mut driver, wire) = tapped_driver_for(build);
    let mut canvas = Canvas::new(W, H);
    let mut at = wire.len();
    let mut out = Vec::with_capacity(frames as usize);
    for _ in 0..frames {
        // The surface is carried across frames for `repaints_over`'s reason: a fresh `Pen` re-records
        // every cell as a first touch, and the second frame's zero is the whole claim.
        let mut pen = Pen::over(canvas);
        driver.frame(|cx| picture_into(&mut pen, cx, whole(), build, &mut Census::default()));
        pen.end_frame();
        canvas = pen.into_canvas();
        let now = wire.len();
        out.push(now - at);
        at = now;
    }
    out
}

/// **Bytes a translation by `rows` whole cell rows costs, on an already-painted screen.**
///
/// The byte half of [`cells_changed_by_shift`], and the trap §14 asks to be gated: a whole-row
/// translation changes **every** cell, so a repaint that priced it by the cell would price it at the
/// whole screen. The number is what the engine's serializer actually emits, which is the
/// comfortable-number check the cell count cannot perform.
pub fn bytes_by_shift(build: Build, rows: u16) -> u64 {
    let (mut driver, wire) = tapped_driver_for(build);
    let mut canvas = Canvas::new(W, H);
    // Frame one: the still screen, so that what frame two costs is the translation and not the
    // picture.
    let mut pen = Pen::over(canvas);
    driver.frame(|cx| {
        picture_into(
            &mut pen,
            cx,
            whole(),
            build.shifted(0),
            &mut Census::default(),
        )
    });
    pen.end_frame();
    canvas = pen.into_canvas();

    let at = wire.len();
    let mut pen = Pen::over(canvas);
    driver.frame(|cx| {
        picture_into(
            &mut pen,
            cx,
            whole(),
            build.shifted(rows),
            &mut Census::default(),
        )
    });
    pen.end_frame();
    wire.len() - at
}

/// The whole screen.
pub fn whole() -> Rect {
    Rect::new(0, 0, W, H)
}

/// **One steady frame, through a [`Tally`] and a [`Pen`] at once.**
///
/// Two frames and not one: the frame structures take their allocation on the first frame that needs
/// one and keep it, so a cold frame is not a frame.
pub fn shape(build: Build) -> Shape {
    let mut driver = driver_for(build);
    driver.frame(|cx| picture_into(&mut Direct, cx, whole(), build, &mut Census::default()));
    let mut tally = Tally::new();
    let mut census = Census::default();
    driver.frame(|cx| picture_into(&mut tally, cx, whole(), build, &mut census));
    let regions = driver.inspect().hits().len();
    let canvas = render(build);
    Shape {
        writes: tally.writes(),
        distinct: tally.distinct(),
        verbs: tally.verbs(),
        regions,
        census,
        adjacent_equal: adjacent_equal(&canvas),
    }
}

/// **The screen as a recorded surface.** The instrument every trap is caught on.
pub fn render(build: Build) -> Canvas {
    let mut driver = driver_for(build);
    let mut pen = Pen::new(W, H);
    for _ in 0..2 {
        driver.frame(|cx| picture_into(&mut pen, cx, whole(), build, &mut Census::default()));
    }
    pen.into_canvas()
}

/// The QR symbol as a recorded surface, drawn at `area`.
pub fn render_qr(build: Build, area: Rect, modules: &Modules) -> Canvas {
    let mut driver = driver_for(build);
    let mut pen = Pen::new(W, H);
    for _ in 0..2 {
        driver.frame(|cx| qr_into(&mut pen, cx, area, build, modules, &mut Census::default()));
    }
    pen.into_canvas()
}

/// **How many adjacent cell pairs carry the same value**, horizontally and vertically.
///
/// Zero is *no rectangle larger than one cell has one paint*, which is what makes
/// [`Ctx::fill`](vitui_runtime::Ctx::fill) unavailable rather than merely unused.
pub fn adjacent_equal(canvas: &Canvas) -> u64 {
    // **A cell nobody wrote is not a cell that shares a paint**, and the review caught the version
    // that thought so: `Canvas::get` answers `None` outside the drawn region and `None == None`, so
    // a QR symbol in the corner of a screen reported 47 250 of 47 620 — a blank screen reading as
    // maximally fillable, which is the exact opposite of what this number is for. It is
    // `crate::runner::Canvas`'s own rule applied here: *never touched is its own value and not a
    // blank*.
    let mut same = 0u64;
    let equal = |a: Option<&crate::runner::Cell>, b: Option<&crate::runner::Cell>| match (a, b) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    };
    for y in 0..canvas.h() {
        for x in 0..canvas.w() {
            let here = canvas.get(x, y);
            if x + 1 < canvas.w() && equal(here, canvas.get(x + 1, y)) {
                same += 1;
            }
            if y + 1 < canvas.h() && equal(here, canvas.get(x, y + 1)) {
                same += 1;
            }
        }
    }
    same
}

/// **How many cells a translation by whole cell rows changes the value of.**
///
/// The reachable half of the scroll trap. The byte half is unreachable and says so.
pub fn cells_changed_by_shift(build: Build, rows: u16) -> u64 {
    let still = render(build.shifted(0));
    let moved = render(build.shifted(rows));
    let mut changed = 0u64;
    for y in 0..still.h() {
        for x in 0..still.w() {
            if still.get(x, y) != moved.get(x, y) {
                changed += 1;
            }
        }
    }
    changed
}

/// **A screen that has already drawn its first frames**, so that everything asked of it afterwards
/// is a *steady* frame.
///
/// It exists for two reasons and the second is the one that bit. A cold frame is not a frame: the
/// five frame structures take their allocation on the first frame that needs one and keep it. And
/// **a driver built inside a measurement window is what the window measures** — `Driver::headless`
/// attaches an engine and allocates its surfaces, which read as 64 allocations a frame and 3.4x the
/// frame time when a per-frame figure was taken over a function that built one.
pub struct Session {
    driver: Driver,
    build: Build,
}

impl Session {
    /// Attach a sink, seat the build's theme, and draw the warm-up frames.
    pub fn open(build: Build) -> Session {
        let mut session = Session {
            driver: driver_for(build),
            build,
        };
        session.frame();
        session.frame();
        session
    }

    /// **One steady frame, through [`Direct`]** — the path a component takes.
    pub fn frame(&mut self) {
        let Session { driver, build } = self;
        let build = *build;
        driver.frame(|cx| picture_into(&mut Direct, cx, whole(), build, &mut Census::default()));
    }
}

/// **What the frame costs**, drawn through [`Direct`]. A report and never a gate.
///
/// The minimum of `frames`, which is `vitui-bench`'s own rule: a minimum is a measurement of the
/// machine at its least interrupted and a mean is a measurement of everything else that was
/// running.
///
/// # Panics
///
/// Panics on zero frames.
pub fn cost(build: Build, frames: u32) -> Duration {
    assert!(frames > 0, "a per-frame figure needs a frame");
    let mut session = Session::open(build);
    let mut best = Duration::MAX;
    for _ in 0..frames {
        let started = Instant::now();
        session.frame();
        best = best.min(started.elapsed());
    }
    best
}

// ── the colour axis ──────────────────────────────────────────────────────────────────────────────

/// **Do these two colours survive as two at this tier?**
///
/// The verb [`Theme::custom`]'s own documentation says a component owes and the runtime does not
/// offer. `Roles::from_palette` puts `base08` on [`Role::Danger`](vitui_runtime::Role::Danger) and `base0A` on [`Role::Warn`](vitui_runtime::Role::Warn)
/// **verbatim**, both over the page and both with no attributes, so the pair key those two roles
/// are compared by differs exactly when the two colours differ on the wire. Everything else in the
/// palette is held fixed, and neither role goes through the `pick` that could substitute one.
///
/// **It is now the runtime's own verb, and it was a contrivance for one ticket.**
/// `Theme::colours_differ_on_wire` is `roles_differ_on_wire` asked of two colours instead of two
/// roles — runtime architecture issue 34, which this screen filed and which named this function as
/// the friction. What it replaced authored a whole thirteen-role theme per colour pair, which is
/// why [`Distinctions`] memoises.
///
/// # The verb alone bought nothing, and the number says so
///
/// Measured on the M1 Max, `--release`, minimum of twenty over two thousand pairs at C16:
///
/// | | ns a call |
/// |---|---|
/// | the contrivance | **773** |
/// | the verb, theme built per call | **811** |
/// | the verb, theme hoisted | **57** |
///
/// **The middle row is the one worth keeping.** `Theme::default()` is
/// `Theme::authored(&CATPPUCCIN_MOCHA, ..)` — `Roles::from_palette` over sixteen entries, thirteen
/// styles built, thirteen keys minted, then thirteen more at `resolve` — so a
/// `Theme::default().resolve(tier)` inside this body is *the contrivance's own cost with the
/// palette mutation removed*, and the mutation was the free part. The first rewrite of this
/// function did exactly that and recorded the cost as gone; the number is here so the next reader
/// does not have to take the sentence on trust.
///
/// A report and not a gate — a timing is a report (§21) — and the ratio is the point rather than
/// the digits.
///
/// The contrivance is worth recording rather than deleting the memory of: `Roles::from_palette` maps
/// `base08` and `base0A` **verbatim** onto [`Role::Danger`](vitui_runtime::Role::Danger) and [`Role::Warn`](vitui_runtime::Role::Warn), both over the page
/// and both with no attributes, so a theme authored with two arbitrary colours in those two slots
/// answered this question through the shipped quantiser. It worked, and *that it worked* is why
/// components register row 158 is `Evaluated` rather than `Unreachable` — the discipline that
/// catches a wrong `Unreachable` is trying it. The friction was filed instead, and the answer came
/// back as a verb.
///
/// **The theme is hoisted out of the call, and that is the half of this that is not the verb.**
/// `colours_differ_on_wire` reads only `Theme::tier`, so the theme is a parameter of the *tier* and
/// never of the pair — and a `Theme::default().resolve(tier)` inside the body would have kept every
/// byte of the contrivance's cost while the documents recorded it as gone. `RESOLVED` is four
/// themes built once. The signature does not change, because the tier is what the caller has.
pub fn wire_differ(a: u32, b: u32, tier: ColorDepth) -> bool {
    if a == b {
        return false;
    }
    RESOLVED[tier_index(tier)].colours_differ_on_wire(rgb(a), rgb(b))
}

/// **The four tiers' themes, built once.** The whole of what [`wire_differ`] needs from a theme is
/// its tier, and there are four of those.
///
/// A `LazyLock` for `crate::document`'s reason and this file's own: *a driver built inside a
/// measurement window is what the window measures*, and a theme built inside a per-pair call is
/// what the per-pair number measures. `Theme` is thirteen `Spec`s, thirteen `Style`s and a
/// revision — plain data — so a `static` of four costs nothing to share.
static RESOLVED: LazyLock<[Theme; 4]> = LazyLock::new(|| {
    [
        Theme::default().resolve(ColorDepth::TrueColor),
        Theme::default().resolve(ColorDepth::Indexed256),
        Theme::default().resolve(ColorDepth::Ansi16),
        Theme::default().resolve(ColorDepth::None),
    ]
});

/// Where a tier sits in [`RESOLVED`]. **An exhaustive match and not an `as usize`**: the engine's
/// `ColorDepth` is not this crate's to number, and a variant added there must arrive here as a
/// compile error rather than as an index.
const fn tier_index(tier: ColorDepth) -> usize {
    match tier {
        ColorDepth::TrueColor => 0,
        ColorDepth::Indexed256 => 1,
        ColorDepth::Ansi16 => 2,
        ColorDepth::None => 3,
    }
}

/// **The horizontal distinctions a picture has at one tier**, and what it had at truecolor.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Distinctions {
    /// The tier.
    pub tier: ColorDepth,
    /// Adjacent horizontal cell pairs the terminal can tell apart.
    pub kept: u64,
    /// How many there are. [`HORIZONTAL_PAIRS`].
    pub total: u64,
}

impl Distinctions {
    /// How many are gone.
    pub fn lost(self) -> u64 {
        self.total - self.kept
    }

    /// What fraction is gone, in percent.
    pub fn lost_percent(self) -> f64 {
        self.lost() as f64 * 100.0 / self.total as f64
    }
}

/// **The distinction census at one tier**, over the cells this build draws.
///
/// Two adjacent cells are told apart when either their foregrounds or their backgrounds differ on
/// the wire — which is the terminal's own question, since a cell is a foreground and a background
/// and nothing else.
///
/// # It reads the source and not the canvas, and that is why it refuses one arm
///
/// A `Paint` is opaque — `crate::runner::Canvas` records one per cell and no crate above the engine
/// can take a colour back out of it — so the colours have to come from where the painter got them.
/// That is exact for [`Palette::Custom`], where a cell's two colours *are* two pixels. It is a
/// different picture entirely for [`Palette::Roles`], whose cells are thirteen roles, and the
/// direct question for those is `Theme::roles_differ_on_wire`. A silent wrong answer there would be
/// the worst kind: a census over a screen that is not on the screen.
///
/// # Panics
///
/// Panics on a build whose cells are not [`Palette::Custom`], or one that is shifted.
pub fn distinctions(build: Build) -> Distinctions {
    assert!(
        build.palette == Palette::Custom && build.shift == 0,
        "the distinction census reads the source pixels of an unshifted picture, and this build \
         does not draw them. A `Palette::Roles` screen collapses onto thirteen paints — its own \
         `adjacent_equal` is 9 055 — and `Theme::roles_differ_on_wire` is the direct question for \
         those; a shifted one samples rows this reads nothing of"
    );
    let sub = u32::from(build.geometry().sy);
    let mut memo: HashMap<(u32, u32), bool> = HashMap::new();
    let mut differ = |a: u32, b: u32| -> bool {
        if a == b {
            return false;
        }
        let key = if a < b { (a, b) } else { (b, a) };
        match memo.get(&key) {
            Some(&v) => v,
            None => {
                let v = wire_differ(a, b, build.depth);
                memo.insert(key, v);
                v
            }
        }
    };
    let mut kept = 0u64;
    for row in 0..H {
        let sub_top = u32::from(row) * sub;
        let sub_bottom = sub_top + sub.saturating_sub(1);
        for col in 0..W - 1 {
            let (l_fg, l_bg) = (
                build.source.pixel(col, sub_top),
                build.source.pixel(col, sub_bottom),
            );
            let (r_fg, r_bg) = (
                build.source.pixel(col + 1, sub_top),
                build.source.pixel(col + 1, sub_bottom),
            );
            if differ(l_fg, r_fg) || differ(l_bg, r_bg) {
                kept += 1;
            }
        }
    }
    Distinctions {
        tier: build.depth,
        kept,
        total: HORIZONTAL_PAIRS,
    }
}

// ── the subjects ─────────────────────────────────────────────────────────────────────────────────

/// The components this screen stands on. **Both are declared since components 30.**
pub const SUBJECTS: [&str; 2] = ["picture", "qr"];

/// Where [`SUBJECTS`] belong, as `(module file, the declaration)`.
///
/// The home is [`crate::Family::F11Media`]'s, whose module is `media.rs`. A component is
/// `fn(&mut Ctx, Rect, …) -> Response` (spec §1, rule 1), so the thing to look for is a public
/// function of the component's own name in its own family's module.
///
/// # The picture's needle is generic, and a needle that was not could never have matched
///
/// This is components ticket 26's finding a second time, and it arrived the same way: the scan read
/// `pub fn picture(` and the shipped declaration is **`pub fn picture<P: Pixels>(`**, because §1's
/// `…` is a *type parameter* here. It has to be — a picture that took a buffer would make the
/// component's cost the image's size rather than the rectangle's, which is `CONTEXT.md`'s invariant
/// one layer up. So a scene green on the parenthesis needle would have been green **by deleting the
/// type parameter**, which is the one thing about this component's signature that is load-bearing.
///
/// `pub fn qr(` is not generic and is written out as it is, so the pair also says that the two
/// needles are two facts rather than one convention.
pub const DECLARATIONS: [(&str, &str); 2] = [
    ("media.rs", "pub fn picture<P: Pixels>("),
    ("media.rs", "pub fn qr("),
];

/// **Which of [`SUBJECTS`] this crate actually declares. Since components 30: both.**
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
/// [`Verdict::of`] refuses vacuity in its constructor, which is what makes this the right shape: the
/// population is the two subjects, and *no component exists* was `Unmet` over two rather than `Met`
/// over nothing. Since components 30 it is `Met` over two.
pub fn standing() -> Verdict {
    let declared = subjects_declared();
    Verdict::of(
        SUBJECTS.len(),
        SUBJECTS.len() - declared.len(),
        "neither `picture` nor `qr` is declared in this crate, so what stands on the picture screen \
         is a stand-in painter and not the components. The screen, its 24 000 customs, its 24 000 \
         verbs, its zero adjacent equal pairs, its ladder of 1 / 2 / 2 against the plot's 1 / 8 / 8, \
         its `Extended == Unicode` in both directions, its distinction census at four tiers, its \
         module readback and its two traps are measured and green; what is missing is the subject",
        "components 30",
    )
}

/// **The sentence a scene waiting for its subject fails with**, or `None` once both are declared.
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
         components are undeclared — {owed}. This is not a defect in the screen. The picture is drawn \
         at three colour depths and at the floor, its custom census is 24 000 against a QR's 4, its \
         ladder is 1 / 2 / 2 against the plot's 1 / 8 / 8, `Extended == Unicode` fires in both \
         directions, the module readback catches the inverted pairing and the shift trap counts \
         24 000 of 24 000 cells — see `crate::picture::tests`. Inverted by `components 30`",
        owed.len(),
        SUBJECTS.len(),
        owed = owed.join(", "),
    ))
}

/// **Fail with the subjects that are missing, the file they belong in, and the ticket.**
///
/// # Panics
///
/// Panics while [`SUBJECTS`] are undeclared, which is **today**. Components ticket 30 inverts it.
pub fn assert_stands_up(scene: &str) {
    if let Some(message) = owed_message(&subjects_declared(), scene) {
        panic!("{message}");
    }
}

// ── the report ───────────────────────────────────────────────────────────────────────────────────

/// **The distinction table, one row a tier**, with the floor beneath it.
pub fn distinction_table(source: Source) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "  tier          kept of {HORIZONTAL_PAIRS}   lost");
    for tier in DEPTHS.into_iter().chain(std::iter::once(ColorDepth::None)) {
        let d = distinctions(Build::correct().of(source).tier(tier));
        let _ = writeln!(
            out,
            "  {:<12}  {:>8}          {:>6.2}%",
            format!("{tier:?}"),
            d.kept,
            d.lost_percent()
        );
    }
    out
}

/// **Repaints per frame over a still picture.** The first frame changes every cell; every frame
/// after it changes none.
///
/// The reachable half of §14's *a static picture emits 900 134 bytes and then 0, 0, 0*: what a byte
/// count says about the wire, this says about the cells that would have produced it.
///
/// # Panics
///
/// Panics on zero frames.
pub fn repaints_over(build: Build, frames: u32) -> Vec<u64> {
    assert!(frames > 0, "a per-frame figure needs a frame");
    let mut driver = driver_for(build);
    let mut canvas = Canvas::new(W, H);
    let mut out = Vec::with_capacity(frames as usize);
    for _ in 0..frames {
        // **The surface is carried across frames, not rebuilt**, which is the whole of what makes
        // the second frame's zero mean anything: a fresh `Pen` every frame would re-record every
        // cell as a first touch and report 24 000 for ever.
        let mut pen = Pen::over(canvas);
        driver.frame(|cx| picture_into(&mut pen, cx, whole(), build, &mut Census::default()));
        pen.end_frame();
        canvas = pen.into_canvas();
        out.push(canvas.take_repaints());
    }
    out
}

// ── the numbers this screen is measured at ───────────────────────────────────────────────────────

/// **Cells a frame writes. 24 000 — every cell of the terminal, at every rung.**
///
/// §20's full-screen class, and the one screen in this crate for which it is not a choice: a
/// picture that left a cell alone would be a picture with a hole in it.
pub const WRITES: u64 = CELLS as u64;

/// **Drawing verbs a frame costs. 24 000, one a cell.**
///
/// §6's *verbs are a currency for structure* at its ceiling: the dense screen draws the same 24 000
/// cells in 171 verbs and a twelve-column table in 160, because their rows are runs. A picture has
/// no runs — see [`ADJACENT_EQUAL`].
pub const VERBS: u64 = CELLS as u64;

/// **[`Theme::custom`] calls a frame. 24 000, one a cell**, and the same number as [`VERBS`] for
/// the same reason.
pub const CUSTOMS: u64 = CELLS as u64;

/// **[`Theme::custom`] calls a QR spends. Four.**
///
/// Not because it has too many distinctions but because it has exactly two and they must be
/// **those** two, which no theme can promise: a QR's dark and light are a specification, not a
/// palette. Four rather than two because a cell carries a pair.
pub const QR_CUSTOMS: u64 = 4;

/// **Interactive regions a picture declares. Zero.**
pub const REGIONS: usize = 0;

/// **Adjacent cell pairs of a photograph carrying the same value. Zero of 47 620.**
///
/// The measurement behind *no `fill` is available at any size*: `Ctx::fill` takes one [`Paint`](vitui_runtime::Paint) for
/// a rectangle, and the largest rectangle of this screen over which one paint is right is **one
/// cell**.
pub const ADJACENT_EQUAL: u64 = 0;

/// **The same count for a gradient. 3 520 of 47 620**, which is 7.4% — so even the smoothest frame
/// anything could produce leaves 92.6% of its adjacencies needing their own paint.
pub const ADJACENT_EQUAL_GRADIENT: u64 = 3_520;

/// **A picture's sub-rows at Ascii, Unicode and Extended. 1, 2, 2.**
pub const LADDER: [u8; 3] = [1, 2, 2];

/// **A bar's, at the same three rungs. 1, 8, 8** — [`crate::chart::raster::geom`]'s own, and where
/// [`crate::media::sub_rows`] takes its availability from.
pub const BAR_LADDER: [u8; 3] = [1, 8, 8];

/// **A mark's bits, at the same three rungs. 1, 4, 8.**
///
/// The comparison that makes *braille buys a picture nothing* a number rather than a sentence: the
/// third rung doubles a mark's resolution and leaves a picture's alone.
pub const MARK_BITS: [u32; 3] = [1, 4, 8];

/// **Cells a translation by one whole cell row changes. 24 000 of 24 000.**
pub const SHIFTED_CELLS: u64 = CELLS as u64;

/// **Modules an inverted pairing reads back wrong. 219 of 441.**
///
/// Every module whose cell-mate differs from it, which for a symbol with a payload is about half.
/// The other half is right by coincidence — two dark modules in a cell are two dark modules
/// whichever way round the halves go — and that is why the count is a gate and a glance is not.
pub const QR_WRONG_INVERTED: u32 = 219;

/// How many modules a version-1 symbol has. `21 * 21`.
pub const QR_MODULE_COUNT: u32 = (QR_SIDE as u32) * (QR_SIDE as u32);

/// **The horizontal distinctions a photograph keeps at truecolor, 256, 16 and none**, of
/// [`HORIZONTAL_PAIRS`].
///
/// §14 states **50.7% gone at sixteen colours** and this screen's photograph loses **35.84%**. The
/// magnitude is the source's and the shape is not: see [`DISTINCTIONS_GRADIENT`], which loses
/// **99.27%** of the same adjacencies at the same rung.
pub const DISTINCTIONS: [u64; 4] = [23_920, 23_899, 15_347, 0];

/// **The same four for a gradient. 20 400, 473, 174, 0.**
///
/// The finding §14 has no row for: on the *distinction* axis the source decides a great deal.
/// Adjacent samples of noise are far apart in colour space and survive a quantiser; adjacent
/// samples of a ramp are one step apart and do not. **35.84% against 99.27% at sixteen colours** —
/// 2.8x — where §14's B/cell claim is that the source moves the wire by 9%.
pub const DISTINCTIONS_GRADIENT: [u64; 4] = [20_400, 473, 174, 0];

/// **Bytes the engine writes for this screen's first four frames, at truecolor.**
/// `[937 233, 0, 0, 0]`.
///
/// Runtime architecture issue 34 is what made this readable at all: `Driver::headless` moves its
/// `Vec` into the engine and never returns it, so until `Output`, `Clock` and `Overrides` reached
/// `vitui_runtime::line::ENGINE_NAMES` no crate above the engine could read a byte the engine wrote.
///
/// **The three zeros are the gate and the first number is a report**, which is this workspace's own
/// rule about encodings: a golden byte *string* is refused because the encoding is exactly the part
/// allowed to change, and a byte *count* is one step from a byte string. What does not depend on the
/// encoding is the shape — *a still picture costs its screen once and then nothing* — and that is
/// what [`WIRE_STEADY_FRAMES`] asserts.
///
/// §14 prices the same screen at **900 134 and then 0, 0, 0**. The zeros reproduce exactly; the
/// total is 4.1% larger, which is a prototype's photograph and not a defect.
pub const WIRE_FRAMES: [u64; 4] = [937_233, 0, 0, 0];

/// **Frames after the first that must cost exactly zero bytes. Three.**
///
/// The gate [`WIRE_FRAMES`]'s first entry is not. Damage is marked at write time and an equal write
/// marks none, so a picture that has not changed is a frame the serializer has nothing to say about
/// — and a *counter on the wrong side of the question* would have been `frames[0] > 0`, which is
/// green on the build where every frame repaints.
pub const WIRE_STEADY_FRAMES: usize = 3;

/// **Bytes a cell costs on the wire at truecolor. 39.05.**
///
/// [`WIRE_FRAMES`]`[0] / `[`CELLS`]. §14 says **37.5**, and it is the same measurement on a
/// different photograph: reported, never gated.
pub const WIRE_PER_CELL: f64 = 39.05;

/// **Bytes a translation by one whole cell row costs. 11 731 of a 937 233-byte screen — 1.25%.**
///
/// **This is §14's trap, and the number inverts it.** [`SHIFTED_CELLS`] says the translation changes
/// **24 000 of 24 000** cells, and a repaint priced by the cell would therefore price it at the whole
/// screen. The engine's scroll pre-pass emits the region and repaints the one row the shift exposed:
/// 11 731 bytes against a row's own 11 715, so the sixteen extra bytes are the scroll sequence.
///
/// §14 says **5 885**, which cannot be reconciled with its own 37.5 B/cell — a 300-cell row at 37.5
/// is 11 250, not 5 885. The measured pair is internally consistent and §14's is not; both are
/// printed by `examples/media_numbers.rs`.
pub const WIRE_SHIFTED: u64 = 11_731;

/// **The share of a full repaint a one-row translation costs, as a ceiling. 2%.**
///
/// Measured at **1.2517%**, so the headroom is 1.6x. The ratio is the gate rather than
/// [`WIRE_SHIFTED`] because it survives an encoding change: whatever a cell costs, a translation
/// costs the rows it exposed and not the cells whose value changed.
pub const WIRE_SHIFT_SHARE_CEILING: f64 = 0.02;

/// **Bytes the first frame costs at truecolor, 256, 16 and no colour at all.**
/// `[937 233, 520 567, 166 463, 72 168]`.
///
/// The other half of what issue 34 unblocked, and the half `Driver::set_theme` could never have
/// reached: seating a `Theme::resolve(tier)` narrows the **thirteen roles** and says nothing about
/// what the engine quantises a `custom` paint into — and every cell here is `custom`. The tier is
/// pinned on `Config::overrides` so that the bytes are the tier's.
///
/// The gate over it is the **monotonicity**, which is a property of quantisation and not of a
/// spelling: a poorer terminal is told less. Reported at 39.05, 21.69, 6.94 and 3.01 B/cell.
pub const WIRE_BY_TIER: [u64; 4] = [937_233, 520_567, 166_463, 72_168];

/// **A module's aspect at one module a cell and at two, on a nominal 1:2 cell. 0.50 and 1.00.**
pub const ASPECT_NOMINAL: [f32; 2] = [0.5, 1.0];

/// **The same pair on a measured 8.0 x 16.5 cell. 0.48 and 0.97.**
///
/// §14 states the pair as **0.97 against 0.50**, and no single cell aspect produces both: the two
/// are one factor of two apart by construction, so `0.97` needs a cell of 0.485 and `0.50` needs
/// one of 0.500. The pair is a nominal cell's floor beside a measured cell's ceiling. Both columns
/// are printed rather than one being fitted to the other; **the gate is the relation**, which is
/// exact at every cell aspect: two modules a cell is exactly twice one.
pub const ASPECT_MEASURED: [f32; 2] = [0.48, 0.97];

#[cfg(test)]
mod tests {
    use super::*;

    /// The denominator [`ADJACENT_EQUAL_GRADIENT`]'s share is read against.
    const ADJACENCY_BASE: u64 = ADJACENCIES;

    /// **The screen writes every cell of the terminal, once, in one verb each.**
    #[test]
    fn a_picture_is_twenty_four_thousand_cells_twenty_four_thousand_verbs_and_no_region() {
        let shape = shape(Build::correct());
        assert_eq!(shape.writes, WRITES);
        assert_eq!(shape.distinct, WRITES, "a cell written twice");
        assert_eq!(shape.verbs, VERBS);
        assert_eq!(shape.regions, REGIONS);
        assert_eq!(shape.census.customs, CUSTOMS);
    }

    /// **No `fill` is available at any size, and both halves of that are checked.**
    ///
    /// `Ctx::fill` takes one [`Paint`](vitui_runtime::Paint) for a rectangle. Zero adjacent pairs of a photograph share a
    /// value, so the largest rectangle one paint serves is **one cell** — and the screen calls the
    /// verb nowhere, which is a scan rather than a counter because a counter for it could only ever
    /// read zero.
    ///
    /// The needle is assembled from fragments so that **this file's test module** does not match
    /// it, which is `crate::frame`'s own finding: a scanner looking for a literal contains that
    /// literal, and it reported the module it defends on its first run.
    #[test]
    fn the_screen_calls_no_fill_and_no_rectangle_larger_than_a_cell_has_one_paint() {
        assert_eq!(adjacent_equal(&render(Build::correct())), ADJACENT_EQUAL);

        let needle = concat!(".fi", "ll(");
        let source =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/picture.rs"))
                .expect("this file is beside the manifest");
        let shipped = source
            .split("#[cfg(test)]")
            .next()
            .expect("split always yields one");
        let calls: Vec<&str> = shipped
            .lines()
            .map(str::trim_start)
            .filter(|line| !line.starts_with("//") && line.contains(needle))
            .collect();
        assert_eq!(calls, Vec::<&str>::new());
        // The other direction, so a scan that has quietly stopped scanning is not read as a zero.
        assert!(format!("    cx{needle}r, \" \", paint);").contains(needle));
    }

    /// **No `fill` is available at any size, and it is a count rather than an argument.**
    ///
    /// `Ctx::fill` takes one paint for a rectangle. Zero adjacent pairs of a photograph share a
    /// value, so the largest rectangle one paint serves is one cell — and the gradient, printed
    /// beside it, is the other end of the claim rather than a counterexample to it: 3 520 of 47 620
    /// is 7.4%, so 92.6% of the smoothest frame anything could produce still needs its own paint.
    #[test]
    fn no_two_adjacent_cells_of_a_photograph_share_a_paint_and_seven_percent_of_a_gradient_do() {
        let photo = render(Build::correct());
        assert_eq!(adjacent_equal(&photo), ADJACENT_EQUAL);
        let gradient = render(Build::correct().of(Source::Gradient));
        assert_eq!(adjacent_equal(&gradient), ADJACENT_EQUAL_GRADIENT);
        const {
            assert!(ADJACENT_EQUAL_GRADIENT * 100 / ADJACENCY_BASE < 8);
        }
    }

    /// **`Extended == Unicode` for a picture, and it fires in both directions.**
    ///
    /// The correct build draws the identical surface at the top two rungs, because the ladder is
    /// 1 / 2 / 2 and stops. [`Ladder::Braille`] is the plot's ladder applied to a picture — 1 / 4 /
    /// 8 bits — and draws a **different** surface at the two rungs, so the equality is reading
    /// something rather than reading a screen that has stopped changing.
    #[test]
    fn extended_equals_unicode_for_a_picture_and_not_for_the_ladder_it_is_not_drawn_on() {
        let correct = Build::correct();
        render(correct.at(RUNGS[1]))
            .diff(&render(correct.at(RUNGS[2])))
            .assert_clean("a picture at Extended against the same picture at Unicode");

        let braille = Build {
            ladder: Ladder::Braille,
            ..correct
        };
        assert!(
            !render(braille.at(RUNGS[1]))
                .diff(&render(braille.at(RUNGS[2])))
                .clean(),
            "the mark ladder draws the same surface at the top two rungs, so the equality above is \
             vacuous"
        );

        assert_eq!(
            [
                crate::media::sub_rows(RUNGS[0]),
                crate::media::sub_rows(RUNGS[1]),
                crate::media::sub_rows(RUNGS[2]),
            ],
            LADDER
        );
        assert_eq!(
            [
                geom(Kind::Bars, RUNGS[0]).sy,
                geom(Kind::Bars, RUNGS[1]).sy,
                geom(Kind::Bars, RUNGS[2]).sy,
            ],
            BAR_LADDER,
            "the ladder `crate::media::sub_rows` derives from has moved"
        );
        assert_eq!(
            [
                geom(Kind::Marks, RUNGS[0]).bits(),
                geom(Kind::Marks, RUNGS[1]).bits(),
                geom(Kind::Marks, RUNGS[2]).bits(),
            ],
            MARK_BITS
        );
    }

    /// **A picture at the bottom repertoire rung is still a picture**: 24 000 cells, one cluster, full
    /// colour depth, one sub-row.
    ///
    /// The one rung where every other component on this map is drawing a worse version of itself.
    /// What it loses is the second sub-row and **nothing else** — the same 24 000 customs, the same
    /// zero adjacent equal pairs, and every horizontal distinction it had.
    #[test]
    fn a_picture_at_ascii_is_a_picture_and_the_only_thing_it_loses_is_a_sub_row() {
        let ascii = Build::correct().at(RUNGS[0]);
        let shape = shape(ascii);
        assert_eq!(shape.writes, WRITES);
        assert_eq!(shape.census.customs, CUSTOMS);
        assert_eq!(adjacent_equal(&render(ascii)), ADJACENT_EQUAL);
        assert_eq!(crate::media::sub_rows(RUNGS[0]), 1);

        // One cluster over the whole screen, and it is a space. A picture at the bottom rung is a
        // background colour, which no glyph table has an entry for and which is why the rung is
        // still a picture rather than a description.
        let canvas = render(ascii);
        let mut clusters = std::collections::BTreeSet::new();
        for y in 0..canvas.h() {
            for x in 0..canvas.w() {
                clusters.insert(
                    canvas
                        .get(x, y)
                        .expect("every cell written")
                        .cluster
                        .clone(),
                );
            }
        }
        assert_eq!(clusters, [" ".to_owned()].into_iter().collect());

        // And every distinction survives, which is the half that separates the colour axis from
        // the glyph axis: the bottom rung of the glyph ladder costs a picture nothing at all.
        assert_eq!(distinctions(ascii).kept, HORIZONTAL_PAIRS);
    }

    /// **The colour axis has a floor of nothing, and the floor is exact.**
    ///
    /// Every other component degrades to a worse drawing of itself; at [`ColorDepth::None`] a
    /// picture has **0 of 23 920** horizontal distinctions — it is a description of itself, and no
    /// other screen in this crate can produce that number.
    #[test]
    fn a_picture_keeps_none_of_its_distinctions_at_no_colour_and_the_ladder_is_monotone() {
        for source in [Source::Photograph, Source::Gradient] {
            let expected = match source {
                Source::Photograph => DISTINCTIONS,
                Source::Gradient => DISTINCTIONS_GRADIENT,
            };
            let measured: Vec<u64> = DEPTHS
                .into_iter()
                .chain(std::iter::once(ColorDepth::None))
                .map(|tier| distinctions(Build::correct().of(source).tier(tier)).kept)
                .collect();
            assert_eq!(measured, expected.to_vec(), "{}", source.word());
            assert!(
                measured.windows(2).all(|w| w[0] >= w[1]),
                "a poorer terminal told more apart: {measured:?}"
            );
            assert_eq!(*measured.last().expect("four tiers"), 0);
        }
    }

    /// **The probe the distinction census rests on, checked in both directions.**
    ///
    /// **Each of the four themes is at the index `tier_index` sends its tier to.**
    ///
    /// Two derivations of one mapping — the order of a literal array and the arms of a match — and
    /// nothing else in this crate would notice them disagreeing: every count `wire_differ` produces
    /// would simply be another tier's, and `DISTINCTIONS`'s four entries would be a permutation of
    /// themselves. The theme knows its own tier, so the equality is available and costs nothing.
    #[test]
    fn every_resolved_theme_is_at_its_own_tiers_index() {
        for tier in DEPTHS.into_iter().chain(std::iter::once(ColorDepth::None)) {
            assert_eq!(RESOLVED[tier_index(tier)].tier(), tier);
        }
        assert_eq!(
            RESOLVED.len(),
            4,
            "four tiers, and `tier_index` is exhaustive over them"
        );
    }

    /// [`wire_differ`] is a contrivance — it authors a theme per colour pair — and a contrivance
    /// that is silently always-true or always-false would make every count above whatever the total
    /// is. So it is asserted on colours whose answer is known independently: two colours one unit
    /// apart survive truecolor and nothing else, two colours a hundred and twenty-eight apart in
    /// red survive down to sixteen, and no two colours survive `ColorDepth::None`.
    #[test]
    fn the_wire_probe_separates_at_truecolor_and_collapses_at_the_floor() {
        assert!(wire_differ(0x10_2030, 0x10_2031, ColorDepth::TrueColor));
        assert!(!wire_differ(0x10_2030, 0x10_2031, ColorDepth::Indexed256));
        assert!(!wire_differ(0x10_2030, 0x10_2031, ColorDepth::Ansi16));

        for tier in DEPTHS {
            assert!(
                wire_differ(0x10_2030, 0x90_2030, tier),
                "{tier:?} lost a colour distance of 128 in one channel"
            );
            assert!(
                !wire_differ(0x10_2030, 0x10_2030, tier),
                "{tier:?} told a colour apart from itself"
            );
        }
        assert!(!wire_differ(0x00_0000, 0xff_ffff, ColorDepth::None));
    }

    /// **A picture translated by a whole cell row changes every cell it has.**
    ///
    /// The reachable half of the scroll trap: 24 000 of 24 000, against a still picture's 0. What
    /// the engine's pre-pass would put on the *wire* for the same frame is 5 885 bytes, and no
    /// crate above the engine can read a byte it wrote — [`crate::gates::REGISTER`] carries that
    /// half as `Unreachable` with the item it needs.
    #[test]
    fn a_translation_by_one_row_changes_every_cell_and_a_still_picture_changes_none() {
        assert_eq!(cells_changed_by_shift(Build::correct(), 1), SHIFTED_CELLS);
        assert_eq!(cells_changed_by_shift(Build::correct(), 0), 0);

        // And the still picture's own frames, which is the other end of the same claim: the first
        // frame changes every cell and every frame after it changes none.
        let frames = repaints_over(Build::correct(), 4);
        assert_eq!(frames, vec![WRITES, 0, 0, 0]);
    }

    /// **The wire: a still picture costs its screen once and then nothing, and a whole-row
    /// translation costs a row rather than a screen.**
    ///
    /// Components register row 161, and the gate that could not be written from this crate at all
    /// until runtime architecture issue 34 re-exported `Clock`, `Output`, `Overrides`,
    /// `WidthSource` and `InputConfig`. Four assertions and **none of them is a byte total**: a
    /// golden byte string is refused in this workspace because the encoding is the part allowed to
    /// change, and a byte count is one step from a byte string. What is gated is what survives an
    /// encoding change.
    ///
    /// 1. **The three zeros.** A still picture is a frame the serializer has nothing to say about.
    /// 2. **The share.** A one-row translation is under 2% of a full repaint — measured 1.2517%,
    ///    1.6x of headroom — which is §14's trap inverted: [`SHIFTED_CELLS`] is 24 000 of 24 000,
    ///    and the wire is priced by the rows the shift exposed.
    /// 3. **Linearity.** `k` rows cost `k` times one row, to within 1%. A pre-pass that had given up
    ///    and repainted would be flat at the screen.
    /// 4. **Monotonicity by tier.** A poorer terminal is told strictly less — the half
    ///    `Driver::set_theme` could never reach, because seating a resolved theme narrows the
    ///    thirteen roles and every cell of this screen is outside them.
    ///
    /// The totals themselves are [`WIRE_FRAMES`], [`WIRE_SHIFTED`] and [`WIRE_BY_TIER`], and
    /// `examples/media_numbers.rs` prints them beside §14's.
    #[test]
    fn a_still_picture_costs_nothing_and_a_translation_costs_a_row() {
        let correct = Build::correct();

        let frames = bytes_over(correct, 4);
        assert_eq!(frames.len(), 1 + WIRE_STEADY_FRAMES);
        assert!(
            frames[0] > 0,
            "the first frame wrote nothing at all, so the tap is not on the engine's writer"
        );
        assert_eq!(
            &frames[1..],
            &[0; WIRE_STEADY_FRAMES],
            "a picture that did not change cost {frames:?} bytes"
        );

        let full = frames[0];
        let one = bytes_by_shift(correct, 1);
        assert!(
            one > 0,
            "a translation that changed every cell of the screen cost no bytes at all"
        );
        let share = one as f64 / full as f64;
        assert!(
            share < WIRE_SHIFT_SHARE_CEILING,
            "a one-row translation cost {one} bytes of a {full}-byte screen — {:.4}%, against a \
             {:.0}% ceiling. §14's trap is that a repaint priced by the cell would price this at \
             the whole screen, and {SHIFTED_CELLS} of {CELLS} cells change value",
            100.0 * share,
            100.0 * WIRE_SHIFT_SHARE_CEILING
        );
        // Nothing moved is nothing written, which is the control arm: without it the share above is
        // green on an engine that emits nothing at all.
        assert_eq!(bytes_by_shift(correct, 0), 0);

        // **Linear in the rows exposed**, which is what says the pre-pass ran rather than gave up.
        for rows in [2u16, 3, 8] {
            let many = bytes_by_shift(correct, rows);
            let expected = one * rows as u64;
            let drift = (many as f64 - expected as f64).abs() / expected as f64;
            assert!(
                drift < 0.01,
                "{rows} rows cost {many} bytes against {expected} for one row {rows} times \
                 ({:.2}% apart), so the cost is not the rows the shift exposed",
                100.0 * drift
            );
        }

        // **A poorer terminal is told strictly less.** The tier is pinned on `Config::overrides`;
        // seating a resolved theme would leave every one of these a truecolor number, because every
        // cell of this screen is `Theme::custom`.
        let by_tier: Vec<u64> = DEPTHS
            .into_iter()
            .chain(std::iter::once(ColorDepth::None))
            .map(|d| bytes_over(correct.tier(d), 1)[0])
            .collect();
        assert!(
            by_tier.windows(2).all(|w| w[0] > w[1]),
            "the wire did not shrink as the terminal got poorer: {by_tier:?}"
        );
    }

    /// **The module readback catches the pairing, and the aspect trap is invisible to it.**
    ///
    /// Two traps and two gates, and this is why they cannot be one. [`Pairing::Inverted`] writes
    /// the same cells with the same four paints one character apart, and 219 of 441 modules come
    /// back wrong. One module a cell reads back **perfectly** — 0 of 441 — and is still not a QR,
    /// because a module that is half as tall as it is wide is not a module. *Not worse, invalid.*
    #[test]
    fn the_readback_catches_the_pairing_and_says_nothing_at_all_about_the_aspect() {
        let modules = v1_symbol();
        assert_eq!(modules.count(), QR_MODULE_COUNT);
        let area = qr_area();
        let correct = Build::correct();

        assert_eq!(
            readback(&render_qr(correct, area, &modules), area, correct, &modules),
            0
        );
        let inverted = Build {
            pairing: Pairing::Inverted,
            ..correct
        };
        assert_eq!(
            readback(
                &render_qr(inverted, area, &modules),
                area,
                inverted,
                &modules
            ),
            QR_WRONG_INVERTED
        );

        // One module a cell: every module readable, every module the wrong shape.
        let flat = correct.at(RUNGS[0]);
        assert_eq!(
            readback(&render_qr(flat, area, &modules), area, flat, &modules),
            0,
            "the readback is the wrong instrument for the aspect and this is what that looks like"
        );
        let round = |v: f32| (v * 100.0).round() / 100.0;
        assert_eq!(
            [
                round(module_aspect(1, CELL_ASPECT_NOMINAL)),
                round(module_aspect(2, CELL_ASPECT_NOMINAL))
            ],
            ASPECT_NOMINAL
        );
        assert_eq!(
            [
                round(module_aspect(1, CELL_ASPECT_MEASURED)),
                round(module_aspect(2, CELL_ASPECT_MEASURED))
            ],
            ASPECT_MEASURED
        );
        // **The relation, which is exact at every cell aspect and is therefore the gate.** The two
        // figures above are a nominal cell's and a measured cell's; this is neither's.
        for aspect in [CELL_ASPECT_NOMINAL, CELL_ASPECT_MEASURED, 0.4, 0.62] {
            assert!(
                (module_aspect(2, aspect) - 2.0 * module_aspect(1, aspect)).abs() < f32::EPSILON,
                "two modules a cell is not exactly twice one at {aspect}"
            );
        }
    }

    /// **A symbol drawn into a rectangle too small is clipped, and the readback says so.**
    ///
    /// The review's finding, and it is the one place on this screen where the instrument and the
    /// defect share a coordinate system: [`crate::runner::Pen`] records at root coordinates with no
    /// clip at the rectangle, so a symbol painted past its own rectangle lands on whatever is there
    /// and [`readback`] — which reads the *same* coordinates — reports **0 of 441 wrong** for it.
    /// A rectangle is an extent and not only an origin, so [`qr_into`] stops at it, and what is
    /// then missing is visible: one module a cell needs 21 rows, and 20 loses a row of them.
    #[test]
    fn a_symbol_that_does_not_fit_its_rectangle_is_clipped_rather_than_drawn_past_it() {
        let modules = v1_symbol();
        let flat = Build::correct().at(RUNGS[0]);
        let full = qr_area();
        assert_eq!(
            readback(&render_qr(flat, full, &modules), full, flat, &modules),
            0,
            "the symbol fits at one module a cell in {}x{}",
            full.w,
            full.h
        );

        // One row short. Twenty-one modules of the last row are unread, and none of them landed on
        // the row below the rectangle.
        let short = Rect::new(full.x, full.y, full.w, full.h - 1);
        assert_eq!(
            readback(&render_qr(flat, short, &modules), short, flat, &modules),
            u32::from(QR_SIDE)
        );
        let canvas = render_qr(flat, short, &modules);
        for m_x in 0..QR_SIDE {
            assert!(
                canvas
                    .get(
                        u16::try_from(short.x).expect("on screen") + m_x,
                        u16::try_from(short.y).expect("on screen") + short.h
                    )
                    .is_none(),
                "column {m_x} of the clipped row was written outside the rectangle"
            );
        }
    }

    /// **A QR spends four customs and a picture spends 24 000, and the four are the whole palette.**
    #[test]
    fn a_qr_spends_four_customs_and_they_are_every_paint_on_its_surface() {
        let modules = v1_symbol();
        let area = qr_area();
        let build = Build::correct();
        let mut census = Census::default();
        let mut driver = driver_for(build);
        let mut pen = Pen::new(W, H);
        driver.frame(|cx| qr_into(&mut pen, cx, area, build, &modules, &mut census));
        assert_eq!(census.customs, QR_CUSTOMS);

        let canvas = pen.into_canvas();
        let mut paints = Vec::new();
        for y in 0..canvas.h() {
            for x in 0..canvas.w() {
                if let Some(cell) = canvas.get(x, y)
                    && !paints.contains(&cell.paint)
                {
                    paints.push(cell.paint);
                }
            }
        }
        assert_eq!(paints.len(), QR_CUSTOMS as usize);
    }

    /// **The 24 000 customs are not what a picture costs**, and that is the finding under §14's
    /// headline.
    ///
    /// [`Palette::Roles`] draws the same 24 000 cells with **zero** customs — thirteen array reads
    /// instead of 24 000 style constructions — and the two frames are inside each other's noise.
    /// What the census is evidence for is **structural**: a cell outside the theme is a cell no
    /// `fill` and no run can reach. It is not evidence about the clock.
    #[test]
    fn a_picture_drawn_from_roles_spends_no_customs_and_writes_the_same_cells() {
        let roles = Build {
            palette: Palette::Roles,
            ..Build::correct()
        };
        let shape = shape(roles);
        assert_eq!(shape.census.customs, 0);
        assert_eq!(shape.census.roles, CUSTOMS);
        assert_eq!(shape.writes, WRITES);
        assert_eq!(shape.verbs, VERBS);

        // And it really is a description rather than the picture: the thirteen roles cannot carry
        // 24 000 distinct paints, so the surface collapses onto at most thirteen.
        let canvas = render(roles);
        let mut paints = Vec::new();
        for y in 0..canvas.h() {
            for x in 0..canvas.w() {
                if let Some(cell) = canvas.get(x, y)
                    && !paints.contains(&cell.paint)
                {
                    paints.push(cell.paint);
                }
            }
        }
        assert!(
            paints.len() <= media::ROLE_RAMP.len(),
            "{} paints from thirteen roles",
            paints.len()
        );
        assert!(
            adjacent_equal(&canvas) > 0,
            "a description with no runs in it"
        );
    }

    /// **The scene stands on its two components, and the sentence that says otherwise is still
    /// live.**
    ///
    /// Components ticket 30 inverted this. What changed is which side of [`owed_message`] the crate
    /// is on, not whether the sentence exists: the message is handed an empty declaration list below
    /// and read, because *a scene that fails because it is unimplemented and a scene that fails
    /// because the code is wrong are the same failure unless the message separates them*.
    #[test]
    fn the_picture_screen_stands_on_picture_and_qr() {
        assert_eq!(subjects_declared(), SUBJECTS.to_vec());
        assert!(standing().met());
        assert!(owed_message(&subjects_declared(), "scene 22").is_none());

        // And the needle is the generic one, which is the half that could have gone green by
        // deleting the type parameter. See `DECLARATIONS`.
        let source = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/media.rs"))
            .expect("media.rs is beside the manifest");
        assert!(
            !crate::dense::declares(&source, "pub fn picture("),
            "`picture` is not generic any more, which is the one thing about its signature that is \
             load-bearing: a buffer parameter makes the frame cost the image rather than the \
             rectangle"
        );

        // The other direction, so the waiting sentence is not one nothing can stop saying.
        let message = owed_message(&[], "scene 22").expect("neither declared");
        assert!(message.contains("components 30"), "{message}");
        assert!(
            message.contains("waiting for its subject rather than failing"),
            "a failure that could be read as a defect in the screen: {message}"
        );
        assert!(
            message.contains("`picture`") && message.contains("`qr`"),
            "{message}"
        );
    }
}
