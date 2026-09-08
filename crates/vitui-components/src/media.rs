//! Pictures, codes and audio shapes — the family that draws with cells where a graphic would go.
//!
//! [`qr`] draws a QR code, [`barcode`] a barcode, [`waveform`], [`spectrum`] and [`vu_meter`] the
//! three shapes an audio application needs, and [`player::chrome`] the transport row under a
//! video. What is deliberately absent is the picture itself: this crate emits no out-of-band
//! graphics bytes, so an image is the terminal's business and not this library's.
//!
//! # Two modules a cell, or one
//!
//! A code is drawn as solid blocks or as half blocks, which is the difference between one module a
//! cell and two: a QR that fits in eleven rows at half resolution needs twenty-one at full. The
//! choice is the caller's, because it is a legibility decision about a particular terminal and a
//! particular scanner rather than a capability question.
//!
//! # An audio shape is a series, not a stream
//!
//! [`waveform`] and [`spectrum`] take the samples you already have. Nothing here reads a device,
//! keeps a buffer or owns a clock: a frame draws what the application has decided is current, which
//! is the same contract every other component in this crate works under.

/// The components homed in this module. See [`crate::Family::members`].
///
/// **Empty on purpose, and checked.** See this module's header: F11 has no row in the freeze.
pub const MEMBERS: &[&str] = &[];

use vitui_runtime::{Ctx, GlyphSet, Id, Paint, Rect, Response, Rgb, Role, Theme};

use crate::chart::raster::{Geom, Kind, cluster, geom};
use crate::ink::{Direct, Ink};

pub mod player;

// ── the ladder ───────────────────────────────────────────────────────────────────────────────────

/// **How many colours a cell carries, whatever glyph it holds. Two.**
///
/// The whole of why the ladder does not apply. A plot puts one *bit* per sub-cell and spells the
/// bitmask with a glyph, so a rung that buys more sub-cells buys more resolution; a picture puts a
/// *colour* in every sub-cell, and a cell is a foreground and a background and nothing else. So the
/// sub-cells a rung offers are a **ceiling this number is below** at every rung above the first.
pub const COLOURS_PER_CELL: u8 = 2;

/// **A picture's sub-rows at one repertoire: 1, 2, 2.**
///
/// # It is derived from the bar ladder rather than branching a second time
///
/// [`crate::gates::REGISTER`]'s row 26 is *`GlyphSet::` in `vitui-components` == 0*, with exactly
/// one named exception — [`crate::chart::raster::geom`], held to three lines. A picture needs the
/// same fact that exception already carries, and taking it from there rather than writing a second
/// `match` is what keeps the exception at one. The refinement 3 is *name the exception, do not
/// loosen the gate*; a second file would have been a second exception argued from the first one's
/// argument.
///
/// **The derivation is the argument and not a trick.** What the bar ladder's `sy` reports is
/// availability: [`crate::chart::raster`]'s own note says the eighth blocks and the half blocks are
/// one contiguous run of block elements and *no font has the half block and not the eighth blocks*,
/// so a rung that offers a bar eight sub-rows offers a picture the half block. [`COLOURS_PER_CELL`]
/// is then the ceiling, and 1, 8, 8 becomes **1, 2, 2**.
///
/// > **Braille's 2x4 buys a picture nothing.** `geom(Kind::Marks, ..)` climbs 1 / 4 / 8 bits; this
/// > climbs 1 / 2 / 2 and stops, and `Extended == Unicode` follows from the arithmetic rather than
/// > from a table.
#[must_use]
pub fn sub_rows(set: GlyphSet) -> u8 {
    geom(Kind::Bars, set).sy.min(COLOURS_PER_CELL)
}

/// **The sub-cell geometry a picture is drawn at**, in the shape [`crate::chart::raster`] states
/// one.
///
/// One sub-column at every rung: the half block splits a cell **vertically**, and there is no
/// spelling that splits it horizontally without also splitting it vertically — the quadrants do
/// both, and a quadrant cell still carries two colours, so the second sub-column would be a fourth
/// pixel with nowhere to put its colour.
#[must_use]
pub fn picture_geom(set: GlyphSet) -> Geom {
    Geom {
        sx: 1,
        sy: sub_rows(set),
    }
}

/// **How many constructions a picture is. Three**, and `constructions: 1..=3` holds.
///
/// They are reached along **two different axes**, which is why the number is not simply the ladder's
/// length:
///
/// | construction | how it is reached | what it is |
/// |---|---|---|
/// | a space with a background | [`GlyphSet::Ascii`] | a picture at one sub-row, full colour depth |
/// | the half block | any richer repertoire | two sub-rows, two colours |
/// | thirteen roles | [`Palette::Roles`] | a **description** of the picture, and zero customs |
///
/// There is no fourth, and the arithmetic says so rather than a table: [`sub_rows`] is the bar
/// ladder under a ceiling of [`COLOURS_PER_CELL`], so the two richer rungs are one construction, and
/// the palette has two arms. `tests::a_picture_is_three_constructions_and_the_top_two_rungs_are_one`
/// renders all six `(rung, palette)` pairs and counts the distinct surfaces.
pub const CONSTRUCTIONS: usize = 3;

/// The quadrant geometry the half blocks are spelled out of. See [`half_block`].
const QUADRANTS: Geom = Geom { sx: 2, sy: 2 };

/// The bitmask of the **upper** half of a quadrant cell, which is `▀`.
const UPPER_HALF: u8 = 0b0000_0011;

/// The bitmask of the **lower** half, which is `▄` — one character from [`UPPER_HALF`] and the whole
/// of what `crate::picture::readback` exists to catch.
const LOWER_HALF: u8 = 0b0000_1100;

/// **The cluster a picture cell wears at `sub`, upper-half spelling.**
///
/// One sub-row is a **space**: a picture at [`GlyphSet::Ascii`] is a background colour, full colour
/// depth, and no glyph table has an entry for it. Two is the half block, taken out of the quadrant
/// table rather than written as a literal so that this file names no glyph of its own.
fn half_block(sub: u8, bits: u8) -> char {
    if sub == 1 {
        ' '
    } else {
        cluster(Kind::Marks, QUADRANTS, bits)
    }
}

// ── what a draw spent ────────────────────────────────────────────────────────────────────────────

/// **What a draw spent, counted by the component rather than by the surface.**
///
/// [`Theme::custom`]'s own documentation asks for this in as many words — *a component that calls
/// it at all should publish its own call census* — and it is a count of **calls**, not of distinct
/// paints: the two are 24 000 and 24 000 for a full-screen picture and 4 and 4 for a QR, and they
/// are equal for both only because neither construction has anywhere to cache one.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct Census {
    /// [`Theme::custom`] calls.
    pub customs: u64,
    /// [`Theme::paint`](vitui_runtime::Theme::paint) calls — a role lookup, which is an array read.
    pub roles: u64,
}

// **There is no `fills` field, and its absence is the deliberate part.** *No `fill` is available at
// any size* is the claim, and a counter for it could only ever read zero — which is
// `crate::counters::Reading`'s own rule from the other side: *a figure defaulted to zero is a
// counter that prints `0` when it means nobody counted*. What does say it is
// `crate::picture::adjacent_equal`, which is why no rectangle larger than one cell has one paint.

// ── the picture ──────────────────────────────────────────────────────────────────────────────────

/// **Where a picture's cells come from.** The caller's, in the picture's own coordinates.
///
/// # It is a trait because the alternative iterates the data
///
/// A buffer parameter would make the component's cost the *image*'s size; a sampler makes it the
/// **rectangle**'s, which is `CONTEXT.md`'s invariant one layer up — *frame cost is proportional to
/// visible cells, never to data volume* — and the same split [`crate::chart::raster::Raster`] takes
/// between the frame and the edit. Scaling, cropping and decoding are the caller's, and they are the
/// caller's because none of them is a fact about a terminal.
///
/// `x` is a **column of the rectangle** and `sub_y` a **sub-row of the rectangle**, both zero-based:
/// a picture never tells its source where on the screen it landed, so the same source draws the same
/// picture wherever it is placed.
pub trait Pixels {
    /// The colour at one sub-cell.
    fn pixel(&self, x: u16, sub_y: u32) -> Rgb;
}

/// **A `&` to a source is a source**, so a caller holding one behind a reference does not have to
/// name the concrete type twice.
impl<T: Pixels + ?Sized> Pixels for &T {
    fn pixel(&self, x: u16, sub_y: u32) -> Rgb {
        (**self).pixel(x, sub_y)
    }
}

/// **Where a picture's cells take their paint from.** The third construction, and the floor drawn.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Palette {
    /// [`Theme::custom`], one call a cell. What a picture is.
    #[default]
    Custom,
    /// The theme's thirteen roles, bucketed by luminance. **Zero customs and a description of the
    /// picture rather than the picture** — the floor of the colour axis, drawn on purpose rather
    /// than arrived at by a poor terminal.
    Roles,
}

/// [`picture`]'s options. rule 3: a `Default` struct, never a required builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct PictureOpts {
    /// Where the cells take their paint from.
    pub palette: Palette,
}

/// **The thirteen roles, in the order a luminance bucket walks them.** [`Palette::Roles`]'s
/// vocabulary, public because it is the *ceiling* on what that construction can draw: a screen
/// painted from it collapses onto at most thirteen paints however many cells it has, which is what
/// makes it a description of the picture rather than the picture.
pub const ROLE_RAMP: [Role; 13] = [
    Role::Body,
    Role::Title,
    Role::Dim,
    Role::Disabled,
    Role::Border,
    Role::Face,
    Role::FaceHover,
    Role::FaceActive,
    Role::Selection,
    Role::Focus,
    Role::Danger,
    Role::Warn,
    Role::Ok,
];

/// **The one line in this crate's media family that calls [`Theme::custom`].**
///
/// `crate::chart`'s `the_series_palette_is_six_customs_and_the_crate_holds_no_style_literal` counts
/// the *calling lines*, because one palette is decided in one place and a second line is
/// a second palette. This family has three constructions whose colours are outside the theme — a
/// picture's pixels, a QR's two and a barcode's two — and **not one of them is a palette**: the
/// first is data and the other two are specifications. So they share one line, and a fourth
/// construction that wanted its own colour would have to come through here.
fn custom(theme: &Theme, fg: Rgb, bg: Rgb) -> Paint {
    theme.custom(fg, bg)
}

/// Integer luminance of an [`Rgb`], weighted 2:4:3 over nine — the quantiser's own weights.
fn luma(v: Rgb) -> u32 {
    (2 * u32::from(v.r) + 4 * u32::from(v.g) + 3 * u32::from(v.b)) / 9
}

/// **A picture, filling the rectangle it was handed.** The pure drawer, and the one component
/// on this map whose every cell is outside the theme.
///
/// ```
/// use vitui_components::media::{Pixels, picture};
/// use vitui_runtime::ctx::Driver;
/// use vitui_runtime::{Rect, Rgb};
///
/// struct Ramp;
/// impl Pixels for Ramp {
///     fn pixel(&self, x: u16, sub_y: u32) -> Rgb {
///         Rgb::new(x as u8, sub_y as u8, 0x80)
///     }
/// }
///
/// let mut driver = Driver::headless(20, 4).expect("a sink attaches");
/// driver.frame(|cx| {
///     let resp = picture(cx, cx.area(), &Ramp);
///     // A pure drawer: it declared nothing, so nothing happened to it.
///     assert!(!resp.hovered && !resp.clicked);
/// });
/// // Zero hit entries — a picture is not a region.
/// assert_eq!(driver.inspect().hits().len(), 0);
/// ```
#[track_caller]
pub fn picture<P: Pixels>(cx: &mut Ctx<'_, '_>, area: Rect, src: &P) -> Response {
    picture_with(cx, area, src, &PictureOpts::default())
}

/// [`picture`], with the options spelled out.
#[track_caller]
pub fn picture_with<P: Pixels>(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    src: &P,
    opts: &PictureOpts,
) -> Response {
    picture_into(&mut Direct, cx, area, src, opts, &mut Census::default())
}

/// **[`picture`], drawing through an [`Ink`] and publishing its [`Census`].**
///
/// One verb a cell, because there is nothing else available: a run needs one paint and a picture's
/// two adjacent cells do not share one. See `crate::picture::ADJACENT_EQUAL`, which is that
/// sentence as **0 of 47 620**.
#[track_caller]
pub fn picture_into<I: Ink, P: Pixels>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    src: &P,
    opts: &PictureOpts,
    census: &mut Census,
) -> Response {
    // **The id is taken here and not inside `draw_picture`**, and the difference is every call
    // site in a program. `Ctx::id` mints from `Location::caller()` and the attribute propagates only
    // through functions that carry it, so an id taken in a plain private body is *that body's line*
    // — one value for every picture ever drawn. See
    // `tests::two_pictures_at_two_call_sites_are_two_widgets`.
    let id = cx.id();
    let sub = sub_rows(cx.theme().glyphs());
    draw_picture(
        ink,
        cx,
        area,
        id,
        src,
        opts,
        census,
        sub,
        half_block(sub, UPPER_HALF),
    )
}

/// The body every picture arm is, with the sub-row count and the cluster supplied.
///
/// The two are parameters rather than reads so that [`defective`]'s arms are **this function with
/// one argument changed** — a reviewer's diff over a defect that draws the same cells with the same
/// paints is one line, which is `crate::frame`'s own arrangement for the fifteen-cell instance.
#[allow(clippy::too_many_arguments)]
fn draw_picture<I: Ink, P: Pixels>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    id: Id,
    src: &P,
    opts: &PictureOpts,
    census: &mut Census,
    sub: u8,
    glyph: char,
) -> Response {
    let theme = *cx.theme();
    let sub = u32::from(sub);
    let mut buf = [0u8; 4];
    let glyph = &*glyph.encode_utf8(&mut buf);
    for row in 0..area.h {
        let top_sub = u32::from(row) * sub;
        for col in 0..area.w {
            let top = src.pixel(col, top_sub);
            let bottom = src.pixel(col, top_sub + sub.saturating_sub(1));
            let paint = match opts.palette {
                Palette::Custom => {
                    census.customs += 1;
                    custom(&theme, top, bottom)
                }
                Palette::Roles => {
                    census.roles += 1;
                    // Thirteen buckets over the luminance of the cell's own top pixel. A
                    // description of the picture: the shape survives and the colours are the
                    // theme's.
                    theme.paint(ROLE_RAMP[(luma(top) as usize * ROLE_RAMP.len()) / 256])
                }
            };
            ink.run(
                cx,
                area.x + i32::from(col),
                area.y + i32::from(row),
                glyph,
                1,
                paint,
            );
        }
    }
    Response::inert(id, area)
}

// ── the symbols ──────────────────────────────────────────────────────────────────────────────────

/// The dark module's colour. Black, and it has to be: a QR's two colours are a specification.
pub const QR_DARK: Rgb = Rgb::new(0x00, 0x00, 0x00);

/// The light module's colour. White, for the same reason.
pub const QR_LIGHT: Rgb = Rgb::new(0xff, 0xff, 0xff);

/// **[`Theme::custom`] calls a QR spends. Four.**
///
/// Not because it has too many distinctions but because it has exactly two and they must be
/// **those** two, which no theme can promise. Four rather than two because a cell carries a *pair*
/// of modules, and the pair has four states.
pub const QR_CUSTOMS: u64 = 4;

/// **[`Theme::custom`] calls a barcode spends. Two.**
///
/// The same specification and half the count, and the subtraction is the finding: a barcode has no
/// vertical structure **inside** a cell, so its cells are only ever wholly dark or wholly light and
/// two of a QR's four states are unreachable. It is [`COLOURS_PER_CELL`] read from the other end.
pub const BARCODE_CUSTOMS: u64 = 2;

/// The four paints a QR spends: *not because it has too many distinctions but because it has
/// exactly two and they must be those two, which no theme can promise.*
#[derive(Clone, Copy, Debug)]
pub struct QrPaints {
    /// Both halves dark.
    pub both_dark: Paint,
    /// Both halves light.
    pub both_light: Paint,
    /// Upper dark, lower light.
    pub dark_light: Paint,
    /// Upper light, lower dark.
    pub light_dark: Paint,
}

impl QrPaints {
    /// Build the four, counting them.
    pub fn new(theme: &Theme, census: &mut Census) -> QrPaints {
        census.customs += QR_CUSTOMS;
        QrPaints {
            both_dark: custom(theme, QR_DARK, QR_DARK),
            both_light: custom(theme, QR_LIGHT, QR_LIGHT),
            dark_light: custom(theme, QR_DARK, QR_LIGHT),
            light_dark: custom(theme, QR_LIGHT, QR_DARK),
        }
    }

    /// The paint for a pair of modules, upper first.
    pub fn of(&self, upper: bool, lower: bool) -> Paint {
        match (upper, lower) {
            (true, true) => self.both_dark,
            (false, false) => self.both_light,
            (true, false) => self.dark_light,
            (false, true) => self.light_dark,
        }
    }

    /// **Decode a cell's paint back into its two modules**, upper first, or `None` for a paint that
    /// is not one of the four.
    pub fn decode(&self, paint: Paint) -> Option<(bool, bool)> {
        for (upper, lower) in [(true, true), (true, false), (false, true), (false, false)] {
            if self.of(upper, lower) == paint {
                return Some((upper, lower));
            }
        }
        None
    }
}

/// **A two-valued module matrix — the caller's symbol.**
///
/// Not an encoder, and the absence is the point: encoding a payload into a QR is a specification
/// with a version, an error-correction level and a mask, none of which is a fact about a terminal.
/// What this crate owes the symbol is that its modules are **square, two-valued and positioned**,
/// which is [`crate::picture::module_aspect`] and `crate::picture::readback`.
#[derive(Clone, Debug)]
pub struct Modules {
    n: u16,
    dark: Vec<bool>,
}

impl Modules {
    /// A `side` by `side` matrix from `dark` in row-major order.
    ///
    /// # Panics
    ///
    /// Panics unless `dark` is exactly `side * side` long — a matrix whose length disagrees with its
    /// side is a matrix that reads back correctly for the wrong reason.
    pub fn new(side: u16, dark: Vec<bool>) -> Modules {
        assert_eq!(
            dark.len(),
            usize::from(side) * usize::from(side),
            "a {side}x{side} matrix needs {} modules",
            usize::from(side) * usize::from(side)
        );
        Modules { n: side, dark }
    }

    /// The side, in modules.
    pub fn side(&self) -> u16 {
        self.n
    }

    /// How many modules there are.
    pub fn count(&self) -> u32 {
        u32::from(self.n) * u32::from(self.n)
    }

    /// Whether a module is dark. `false` for anything outside the symbol.
    pub fn dark(&self, x: u16, y: u16) -> bool {
        if x >= self.n || y >= self.n {
            return false;
        }
        self.dark[usize::from(y) * usize::from(self.n) + usize::from(x)]
    }
}

/// **A QR symbol, two modules to a cell wherever the half block exists.**
///
/// The half block is what makes the symbol *correct* rather than prettier: a module must be square
/// and a cell is not, so one module a cell is a third kind of degradation the matrix has no column
/// for — **not worse, invalid**. See [`crate::picture::module_aspect`].
///
/// ```
/// use vitui_components::media::{Modules, qr};
/// use vitui_runtime::ctx::Driver;
///
/// let modules = Modules::new(2, vec![true, false, false, true]);
/// let mut driver = Driver::headless(8, 4).expect("a sink attaches");
/// driver.frame(|cx| { let _ = qr(cx, cx.area(), &modules); });
/// assert_eq!(driver.inspect().hits().len(), 0);
/// ```
#[track_caller]
pub fn qr(cx: &mut Ctx<'_, '_>, area: Rect, modules: &Modules) -> Response {
    qr_with(cx, area, modules, &mut Census::default())
}

/// [`qr`], publishing its [`Census`]. There are no options: a QR's two colours are a specification
/// and its geometry is [`sub_rows`], so there is nothing left for a caller to choose.
#[track_caller]
pub fn qr_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    modules: &Modules,
    census: &mut Census,
) -> Response {
    qr_into(&mut Direct, cx, area, modules, census)
}

/// **[`qr`], drawing through an [`Ink`].**
#[track_caller]
pub fn qr_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    modules: &Modules,
    census: &mut Census,
) -> Response {
    // See `picture_into`: the id belongs to the `#[track_caller]` frame.
    let id = cx.id();
    let sub = sub_rows(cx.theme().glyphs());
    draw_qr(
        ink,
        cx,
        area,
        id,
        modules,
        census,
        sub,
        half_block(sub, UPPER_HALF),
    )
}

/// The body both QR arms are. See [`draw_picture`] for why the spelling is a parameter.
#[allow(clippy::too_many_arguments)]
fn draw_qr<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    id: Id,
    modules: &Modules,
    census: &mut Census,
    sub: u8,
    glyph: char,
) -> Response {
    let theme = *cx.theme();
    let paints = QrPaints::new(&theme, census);
    let per_cell = u16::from(sub);
    let mut buf = [0u8; 4];
    let glyph = &*glyph.encode_utf8(&mut buf);
    // **The rectangle is an extent and not only an origin.** A symbol that does not fit is drawn as
    // far as it fits, because a component that paints past its rectangle has left the partition rule
    // and lands on whatever is there.
    let rows = modules.side().div_ceil(per_cell).min(area.h);
    let cols = modules.side().min(area.w);
    for row in 0..area.h {
        let y = area.y + i32::from(row);
        let drawn = if row < rows { cols } else { 0 };
        for col in 0..drawn {
            let upper = modules.dark(col, row * per_cell);
            let lower = if per_cell == 1 {
                upper
            } else {
                modules.dark(col, row * per_cell + 1)
            };
            ink.run(
                cx,
                area.x + i32::from(col),
                y,
                glyph,
                1,
                paints.of(upper, lower),
            );
        }
        // **The quiet zone, and it is the partition rule and the specification at once.** §2 asks a
        // component to write every cell of its rectangle exactly once; a QR reader asks for light
        // margin around the symbol. The same run answers both, so a symbol in a rectangle larger
        // than itself is *more* readable rather than merely tidier.
        ink.run(
            cx,
            area.x + i32::from(drawn),
            y,
            " ",
            area.w - drawn,
            paints.both_light,
        );
    }
    Response::inert(id, area)
}

/// **A one-dimensional symbol: `bars` columns, dark or light, full height.**
///
/// Two customs where a QR spends four, and it has **runs where a picture has none** — a bar three
/// modules wide is one verb. That is the same fact from three sides: a barcode carries no
/// information across a cell's own height, so nothing inside a cell varies and nothing along a row
/// of one bar does either.
///
/// ```
/// use vitui_components::media::barcode;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(8, 3).expect("a sink attaches");
/// driver.frame(|cx| { let _ = barcode(cx, cx.area(), &[true, true, false, true]); });
/// ```
#[track_caller]
pub fn barcode(cx: &mut Ctx<'_, '_>, area: Rect, bars: &[bool]) -> Response {
    barcode_with(cx, area, bars, &mut Census::default())
}

/// [`barcode`], publishing its [`Census`].
#[track_caller]
pub fn barcode_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    bars: &[bool],
    census: &mut Census,
) -> Response {
    barcode_into(&mut Direct, cx, area, bars, census)
}

/// **[`barcode`], drawing through an [`Ink`].**
#[track_caller]
pub fn barcode_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    bars: &[bool],
    census: &mut Census,
) -> Response {
    let id = cx.id();
    let theme = *cx.theme();
    census.customs += BARCODE_CUSTOMS;
    let dark = custom(&theme, QR_DARK, QR_DARK);
    let light = custom(&theme, QR_LIGHT, QR_LIGHT);
    // A partition of the rectangle: every column is a bar or the quiet zone after the last one.
    for row in 0..area.h {
        let y = area.y + i32::from(row);
        let mut col = 0u16;
        while col < area.w {
            let value = bars.get(usize::from(col)).copied();
            let mut run = 1u16;
            while col + run < area.w && bars.get(usize::from(col + run)).copied() == value {
                run += 1;
            }
            ink.run(
                cx,
                area.x + i32::from(col),
                y,
                " ",
                run,
                if value == Some(true) { dark } else { light },
            );
            col += run;
        }
    }
    Response::inert(id, area)
}

// ── the audio three ──────────────────────────────────────────────────────────────────────────────

/// **The bar ladder a bottom-anchored column climbs: 1 / 8 / 8.**
///
/// [`crate::chart::raster::geom`] verbatim, which is *the audio half adds no mechanism and no
/// construction* as a call rather than as a sentence. A waveform, a spectrum and a VU meter are all
/// prefixes of a column, so the ladder applies with nothing added.
fn bar_rows(set: GlyphSet) -> u8 {
    geom(Kind::Bars, set).sy
}

/// The cluster a column cell wears when `filled` of `sub` sub-rows are set.
fn bar_cluster(sub: u8, filled: u8) -> char {
    let g = Geom { sx: 1, sy: sub };
    // `cluster` reads `count_ones`, so any mask with `filled` bits spells the same prefix.
    let bits = ((1u16 << filled.min(8)) - 1) as u8;
    cluster(Kind::Bars, g, bits)
}

/// **A centred waveform: the signal drawn symmetrically about the middle of the rectangle.**
///
/// Zero customs — its colour is [`Role::Ok`] — and one column of the rectangle asks the samples for
/// one bucket, so the frame costs the rectangle and the bucketing costs the data. That is
/// [`crate::chart`]'s own split, and the reason a waveform over an hour of audio is not an hour of
/// work a frame.
///
/// ```
/// use vitui_components::media::waveform;
/// use vitui_runtime::ctx::Driver;
///
/// let samples: Vec<f32> = (0..1000).map(|i| (i as f32 / 40.0).sin()).collect();
/// let mut driver = Driver::headless(40, 6).expect("a sink attaches");
/// driver.frame(|cx| { let _ = waveform(cx, cx.area(), &samples); });
/// ```
#[track_caller]
pub fn waveform(cx: &mut Ctx<'_, '_>, area: Rect, samples: &[f32]) -> Response {
    waveform_with(cx, area, samples, &mut Census::default())
}

/// [`waveform`], publishing its [`Census`] — which is **zero customs**, and that zero is the point.
#[track_caller]
pub fn waveform_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    samples: &[f32],
    census: &mut Census,
) -> Response {
    waveform_into(&mut Direct, cx, area, samples, census)
}

/// **[`waveform`], drawing through an [`Ink`].**
#[track_caller]
pub fn waveform_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    samples: &[f32],
    census: &mut Census,
) -> Response {
    let id = cx.id();
    let sub = bar_rows(cx.theme().glyphs());
    census.roles += 1;
    let paint = cx.theme().paint(Role::Ok);
    if area.w == 0 || area.h == 0 {
        return Response::inert(id, area);
    }
    census.roles += 1;
    let ground = cx.theme().paint(Role::Body);
    let half = f32::from(area.h) / 2.0;
    for col in 0..area.w {
        let peak = bucket_peak(samples, col, area.w);
        // Sub-rows out from the centre, in each direction.
        let reach = (peak * half * f32::from(sub)).round().max(0.0) as u32;
        for row in 0..area.h {
            let distance = ((f32::from(row) + 0.5) - half).abs();
            let below = ((distance - 0.5).max(0.0) * f32::from(sub)) as u32;
            let filled = reach.saturating_sub(below).min(u32::from(sub)) as u8;
            let mut buf = [0u8; 4];
            let glyph = &*bar_cluster(sub, filled).encode_utf8(&mut buf);
            // **Written and not skipped**, which is §2: a cell with no signal in it is a cell this
            // component is responsible for, and a `continue` here leaves whatever was underneath.
            ink.run(
                cx,
                area.x + i32::from(col),
                area.y + i32::from(row),
                glyph,
                1,
                if filled == 0 { ground } else { paint },
            );
        }
    }
    Response::inert(id, area)
}

/// The loudest sample in the bucket column `col` of `cols` maps onto.
fn bucket_peak(samples: &[f32], col: u16, cols: u16) -> f32 {
    if samples.is_empty() || cols == 0 {
        return 0.0;
    }
    let lo = usize::from(col) * samples.len() / usize::from(cols);
    let hi = ((usize::from(col) + 1) * samples.len() / usize::from(cols)).max(lo + 1);
    samples[lo..hi.min(samples.len())]
        .iter()
        .fold(0.0f32, |a, s| a.max(s.abs()))
        .clamp(0.0, 1.0)
}

/// **A spectrum: one bottom-anchored bar per bin.** [`crate::chart::chart`]'s construction with
/// nothing added, which is why it is here as a call and not as a component.
#[track_caller]
pub fn spectrum(cx: &mut Ctx<'_, '_>, area: Rect, bins: &[f32]) -> Response {
    spectrum_with(cx, area, bins, &mut Census::default())
}

/// [`spectrum`], publishing its [`Census`].
#[track_caller]
pub fn spectrum_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    bins: &[f32],
    census: &mut Census,
) -> Response {
    spectrum_into(&mut Direct, cx, area, bins, census)
}

/// **[`spectrum`], drawing through an [`Ink`].**
#[track_caller]
pub fn spectrum_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    bins: &[f32],
    census: &mut Census,
) -> Response {
    let id = cx.id();
    census.roles += 1;
    let paint = cx.theme().paint(Role::Warn);
    census.roles += 1;
    let ground = cx.theme().paint(Role::Body);
    let sub = bar_rows(cx.theme().glyphs());
    columns(ink, cx, area, sub, ground, |col| {
        (bins.get(usize::from(col)).copied().unwrap_or(0.0), paint)
    });
    Response::inert(id, area)
}

/// **A VU meter: one bottom-anchored column a channel, coloured by how loud it is.**
///
/// Three roles and **zero customs**: [`Role::Ok`] below the warning line, [`Role::Warn`] above it,
/// [`Role::Danger`] above the clip line. The peak is the caller's — a meter that held its own peak
/// would need a clock, which is the mechanism `crate::media::player::Needs::Clock` names and
/// the indicator family owns.
#[track_caller]
pub fn vu_meter(cx: &mut Ctx<'_, '_>, area: Rect, levels: &[f32]) -> Response {
    vu_meter_with(cx, area, levels, &mut Census::default())
}

/// [`vu_meter`], publishing its [`Census`].
#[track_caller]
pub fn vu_meter_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    levels: &[f32],
    census: &mut Census,
) -> Response {
    vu_meter_into(&mut Direct, cx, area, levels, census)
}

/// The level a VU meter turns [`Role::Warn`] at.
pub const VU_WARN: f32 = 0.75;

/// The level a VU meter turns [`Role::Danger`] at.
pub const VU_CLIP: f32 = 0.95;

/// **[`vu_meter`], drawing through an [`Ink`].**
#[track_caller]
pub fn vu_meter_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    levels: &[f32],
    census: &mut Census,
) -> Response {
    let id = cx.id();
    census.roles += 3;
    let ok = cx.theme().paint(Role::Ok);
    let warn = cx.theme().paint(Role::Warn);
    let danger = cx.theme().paint(Role::Danger);
    census.roles += 1;
    let ground = cx.theme().paint(Role::Body);
    let sub = bar_rows(cx.theme().glyphs());
    columns(ink, cx, area, sub, ground, |col| {
        let v = levels.get(usize::from(col)).copied().unwrap_or(0.0);
        let paint = if v >= VU_CLIP {
            danger
        } else if v >= VU_WARN {
            warn
        } else {
            ok
        };
        (v, paint)
    });
    Response::inert(id, area)
}

/// **The one column loop the spectrum and the VU meter share.** A bottom-anchored prefix per column,
/// at `sub` sub-rows, with the value and the paint supplied per column.
fn columns<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    sub: u8,
    ground: Paint,
    mut of: impl FnMut(u16) -> (f32, Paint),
) {
    for col in 0..area.w {
        let (value, paint) = of(col);
        let total = (value.clamp(0.0, 1.0) * f32::from(area.h) * f32::from(sub)).round() as u32;
        for row in 0..area.h {
            let from_bottom = u32::from(area.h - 1 - row);
            let filled = total
                .saturating_sub(from_bottom * u32::from(sub))
                .min(u32::from(sub)) as u8;
            let mut buf = [0u8; 4];
            let glyph = &*bar_cluster(sub, filled).encode_utf8(&mut buf);
            // See `waveform_into`: an empty cell is written rather than skipped, because §2 makes
            // it this component's.
            ink.run(
                cx,
                area.x + i32::from(col),
                area.y + i32::from(row),
                glyph,
                1,
                if filled == 0 { ground } else { paint },
            );
        }
    }
}

// ── the arms that are wrong on purpose ───────────────────────────────────────────────────────────

/// **The two spellings a picture is one character away from**, kept here so the gate's subject is
/// the shipped body with one argument changed.
///
/// `crate::ink`'s own rule: *a second implementation written against the alternative would be a gate
/// testing a copy*. Both arms below call the **same private bodies** [`picture_into`] and
/// [`qr_into`] call, with one argument changed — the cluster the cell wears — so a defect that lives
/// in the loop cannot hide in one arm and not the other.
pub mod defective {
    use super::{
        Census, Ctx, Ink, LOWER_HALF, Modules, PictureOpts, Pixels, Rect, Response, draw_picture,
        draw_qr, geom, half_block, sub_rows,
    };
    use crate::chart::raster::Kind;

    /// **`▄` where `▀` is right.** The same cells, the same four paints, one character apart, and
    /// every cell of the picture mirrored inside itself.
    ///
    /// It is invisible to a paint census, to a verb count and to a cell-for-cell comparison of
    /// *paints* — which is why `crate::picture::readback` models the terminal rather than reading
    /// the surface.
    #[track_caller]
    pub fn picture_inverted_into<I: Ink, P: Pixels>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        src: &P,
        opts: &PictureOpts,
        census: &mut Census,
    ) -> Response {
        let id = cx.id();
        let sub = sub_rows(cx.theme().glyphs());
        draw_picture(
            ink,
            cx,
            area,
            id,
            src,
            opts,
            census,
            sub,
            half_block(sub, LOWER_HALF),
        )
    }

    /// **The plot's ladder applied to a picture.** Every sub-cell of the mark rung set, so the eight
    /// dots of a braille cell share one foreground and the rung buys resolution the colours cannot
    /// carry.
    ///
    /// The arm that makes `Extended == Unicode` mean something: drawn this way the screen at
    /// Extended is a **different** screen from the one at Unicode, so an equality over the correct
    /// arm is reading something rather than reading a screen that has stopped changing.
    #[track_caller]
    pub fn picture_braille_into<I: Ink, P: Pixels>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        src: &P,
        opts: &PictureOpts,
        census: &mut Census,
    ) -> Response {
        let id = cx.id();
        let g = geom(Kind::Marks, cx.theme().glyphs());
        let glyph = crate::chart::raster::cluster(Kind::Marks, g, u8::MAX);
        draw_picture(ink, cx, area, id, src, opts, census, g.sy, glyph)
    }

    /// **A QR with the halves swapped.** `crate::picture::QR_WRONG_INVERTED` of
    /// `crate::picture::QR_MODULE_COUNT` modules read back wrong, and the other half right by
    /// coincidence — two dark modules in a cell are two dark modules whichever way round the halves
    /// go, which is why the count is a gate and a glance is not.
    #[track_caller]
    pub fn qr_inverted_into<I: Ink>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        modules: &Modules,
        census: &mut Census,
    ) -> Response {
        let id = cx.id();
        let sub = sub_rows(cx.theme().glyphs());
        draw_qr(
            ink,
            cx,
            area,
            id,
            modules,
            census,
            sub,
            half_block(sub, LOWER_HALF),
        )
    }
}

// ── the refusal ──────────────────────────────────────────────────────────────────────────────────

/// **Why there is no wallpaper here, and why the reason is not the engine's.**
///
/// The survey's boldest ✅ in this family is a background image, and the design agrees about the
/// engine and disagrees about the seam. **No engine change is needed**: `LayerKind::Content
/// { opaque }` and the `EMPTY`-skipping blit already exist, and built on the engine's own
/// `composite` the still-screen damage is the application's own, unchanged, and the frame is
/// **13.7× cheaper**.
///
/// # The three places the runtime's layer host cannot express it
///
/// 1. There is **no slot below the base pass** — a layer host that opens its stack at the base pass
///    has nowhere to put a layer under it.
/// 2. `blit_rows` is **unconditionally opaque**, so a layer above the wallpaper overwrites it
///    wherever it draws rather than wherever it has something to say.
/// 3. A surface **cannot be cleared to transparent**, so the layer above starts as a full rectangle
///    of cells rather than as an empty one.
///
/// The name those three are recorded against — `rt::overlay::LayerHost` — exists on the runtime
/// *architecture* map and on no ticket of the runtime implementation backlog, so this is a finding
/// recorded rather than an edge owed. It is the runtime map's and it is not this family's.
///
/// # Drawn where a component can actually put it, the number goes the other way
///
/// First in the base pass is the only place a component can put a wallpaper, and there the wallpaper
/// and the UI overwrite each other's cells every frame: a **still** screen re-damages **24 000 cells
/// against 1 076** and costs **407 µs against 14**. That is the partition rule in a form
/// **reordering cannot fix**, because a wallpaper must be drawn first by definition.
///
/// And the number the survey's ✅ is really a claim about: an application shows **20 093 of 24 000
/// cells (83.7%)** of its wallpaper with every panel interior open, and **0** once each panel fills
/// its interior. **A wallpaper is not a feature a component provides; it is a discipline the whole
/// screen keeps.**
#[derive(Clone, Copy, Debug)]
pub struct WhyThereIsNoWallpaper;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chart::raster::RUNGS;
    use crate::counters::Tally;
    use vitui_runtime::ctx::Driver;

    /// One construction's draw, so the sweep below is six rows of one thing rather than six blocks.
    type Draw<'a> = dyn Fn(&mut Tally, &mut Ctx<'_, '_>, &mut Census) + 'a;

    /// A grey ramp, so a picture has something to be a picture of.
    struct Ramp;

    impl Pixels for Ramp {
        fn pixel(&self, x: u16, sub_y: u32) -> Rgb {
            Rgb::new((x % 251) as u8, (sub_y % 241) as u8, 0x80)
        }
    }

    /// One frame through a [`Tally`], with the census beside it.
    fn tallied(
        w: u16,
        h: u16,
        mut f: impl FnMut(&mut Tally, &mut Ctx<'_, '_>, &mut Census),
    ) -> (Tally, Census) {
        let mut driver = Driver::headless(w, h).expect("a sink attaches");
        let mut tally = Tally::new();
        let mut census = Census::default();
        // Two frames: the frame structures take their allocation on the first that needs one.
        driver.frame(|cx| f(&mut Tally::new(), cx, &mut Census::default()));
        driver.frame(|cx| f(&mut tally, cx, &mut census));
        (tally, census)
    }

    /// **The census as a contrast, which is what makes it a gate rather than four numbers.**
    ///
    /// A picture spends one [`Theme::custom`] a cell, a QR spends four however many modules it has,
    /// a barcode spends two, and the audio three spend **none at all**. The four are asserted
    /// together because what is stated is the *split in the family*, and any one of them alone is
    /// a number with nothing to be surprising against.
    #[test]
    fn the_custom_census_is_one_a_cell_then_four_then_two_then_none() {
        let (tally, census) = tallied(20, 5, |ink, cx, census| {
            picture_into(ink, cx, cx.area(), &Ramp, &PictureOpts::default(), census);
        });
        assert_eq!(census.customs, 100, "one a cell over 20x5");
        assert_eq!(tally.writes(), 100);
        assert_eq!(tally.verbs(), 100, "a picture has no runs");

        let modules = Modules::new(
            3,
            vec![true, false, true, false, true, false, true, false, true],
        );
        let (_, census) = tallied(20, 5, |ink, cx, census| {
            qr_into(ink, cx, cx.area(), &modules, census);
        });
        assert_eq!(census.customs, QR_CUSTOMS);

        let (tally, census) = tallied(20, 5, |ink, cx, census| {
            barcode_into(
                ink,
                cx,
                cx.area(),
                &[true, true, false, false, true],
                census,
            );
        });
        assert_eq!(census.customs, BARCODE_CUSTOMS);
        assert_eq!(tally.writes(), 100, "a partition of the rectangle");
        assert!(
            tally.verbs() < tally.writes(),
            "a barcode has runs where a picture has none: {} verbs for {} cells",
            tally.verbs(),
            tally.writes()
        );

        let samples: Vec<f32> = (0..500).map(|i| ((i as f32) / 30.0).sin()).collect();
        let bins: Vec<f32> = (0..20u16).map(|i| f32::from(i) / 20.0).collect();
        for (name, spent) in [
            (
                "waveform",
                tallied(20, 5, |ink, cx, census| {
                    waveform_into(ink, cx, cx.area(), &samples, census);
                })
                .1,
            ),
            (
                "spectrum",
                tallied(20, 5, |ink, cx, census| {
                    spectrum_into(ink, cx, cx.area(), &bins, census);
                })
                .1,
            ),
            (
                "vu meter",
                tallied(20, 5, |ink, cx, census| {
                    vu_meter_into(ink, cx, cx.area(), &bins, census);
                })
                .1,
            ),
        ] {
            assert_eq!(spent.customs, 0, "{name} spent a custom");
            assert!(spent.roles > 0, "{name} counted no role either");
        }
    }

    /// **The audio three take the ladder verbatim**, which is `1 / 8 / 8` and not a second table.
    #[test]
    fn the_audio_half_climbs_the_bar_ladder_and_adds_no_rung_of_its_own() {
        assert_eq!(
            [bar_rows(RUNGS[0]), bar_rows(RUNGS[1]), bar_rows(RUNGS[2])],
            [1, 8, 8]
        );
        assert_eq!(
            [sub_rows(RUNGS[0]), sub_rows(RUNGS[1]), sub_rows(RUNGS[2])],
            [1, 2, 2],
            "a picture's ladder is the bar ladder under a ceiling of two"
        );
        // And the prefix a full cell wears is the full block at the top rung and `#` at the bottom.
        assert_eq!(bar_cluster(8, 8), '\u{2588}');
        assert_eq!(bar_cluster(1, 1), '#');
        assert_eq!(bar_cluster(8, 0), ' ');
    }

    /// **Every drawer here writes a partition of its rectangle**, at four sizes, including the two
    /// where the subject is smaller than the rectangle it was handed.
    ///
    /// The partition rule, and the two interesting arms are the symbols: a QR at two modules a cell fills eleven of
    /// twenty-one rows and a barcode's pattern is shorter than its band, so both have a remainder —
    /// and for a QR that remainder is the **quiet zone the specification asks for**, which is the
    /// one place on this map where the partition rule and the subject's own standard are the same
    /// sentence.
    #[test]
    fn every_drawer_writes_each_cell_of_its_rectangle_exactly_once() {
        let modules = Modules::new(
            3,
            vec![true, false, true, false, true, false, true, false, true],
        );
        let bars = [true, true, false, true];
        let samples: Vec<f32> = (0..300).map(|i| ((i as f32) / 17.0).sin()).collect();
        let bins = [0.1f32, 0.9, 0.5];
        for (w, h) in [(1u16, 1u16), (7, 3), (20, 5), (41, 13)] {
            let cells = u64::from(w) * u64::from(h);
            let arms: [(&str, &Draw<'_>); 7] = [
                ("picture", &|ink, cx, census| {
                    picture_into(ink, cx, cx.area(), &Ramp, &PictureOpts::default(), census);
                }),
                ("roles", &|ink, cx, census| {
                    picture_into(
                        ink,
                        cx,
                        cx.area(),
                        &Ramp,
                        &PictureOpts {
                            palette: Palette::Roles,
                        },
                        census,
                    );
                }),
                ("qr", &|ink, cx, census| {
                    qr_into(ink, cx, cx.area(), &modules, census);
                }),
                ("barcode", &|ink, cx, census| {
                    barcode_into(ink, cx, cx.area(), &bars, census);
                }),
                ("waveform", &|ink, cx, census| {
                    waveform_into(ink, cx, cx.area(), &samples, census);
                }),
                ("spectrum", &|ink, cx, census| {
                    spectrum_into(ink, cx, cx.area(), &bins, census);
                }),
                ("vu meter", &|ink, cx, census| {
                    vu_meter_into(ink, cx, cx.area(), &bins, census);
                }),
            ];
            for (name, draw) in arms {
                let (tally, _) = tallied(w, h, |ink, cx, census| draw(ink, cx, census));
                assert_eq!(tally.writes(), cells, "{name} at {w}x{h}: writes");
                assert_eq!(
                    tally.distinct(),
                    cells,
                    "{name} at {w}x{h}: a cell written twice"
                );
            }
        }
    }

    /// **Three constructions, and the count is a union rather than a product.**
    ///
    /// The two axes a picture can be drawn differently on are not independent, which is the whole
    /// reason the number is three and not four:
    ///
    /// 1. **The glyph axis has two values** — a space at [`GlyphSet::Ascii`] and the half block
    ///    above it — because [`sub_rows`] is the bar ladder under a ceiling of two. Asserted by
    ///    rendering all three rungs and finding two distinct cluster surfaces.
    /// 2. **The palette axis has two values**, and the second is a *description*: thirteen paints
    ///    bucketed by luminance. Asserted by rendering both at one rung and finding two distinct
    ///    paint surfaces.
    /// 3. **The description does not move with the rung.** Its paints are the theme's and its cells
    ///    read one pixel each, so it is **one** construction and not two — which is what turns
    ///    `2 x 2` into `2 + 1`.
    ///
    /// The `constructions: 1..=3` therefore holds with nothing to spare, and a fourth spelling is
    /// a visible edit rather than a quiet one.
    #[test]
    fn a_picture_is_three_constructions_because_the_two_axes_are_not_independent() {
        use crate::runner::Pen;

        /// The two surfaces one `(rung, palette)` pair draws: what each cell *says*, and what each
        /// cell is *painted* in.
        fn surfaces(rung: GlyphSet, palette: Palette) -> (Vec<Option<String>>, Vec<Option<Paint>>) {
            let mut driver = Driver::headless(9, 4).expect("a sink attaches");
            driver.set_theme(
                Theme::authored(
                    &vitui_runtime::theme::CATPPUCCIN_MOCHA,
                    rung,
                    vitui_runtime::theme::Density::default(),
                )
                .resolve(vitui_runtime::ColorDepth::TrueColor),
            );
            let mut pen = Pen::new(9, 4);
            for _ in 0..2 {
                driver.frame(|cx| {
                    picture_into(
                        &mut pen,
                        cx,
                        cx.area(),
                        &Ramp,
                        &PictureOpts { palette },
                        &mut Census::default(),
                    );
                });
            }
            let canvas = pen.into_canvas();
            let mut clusters = Vec::new();
            let mut paints = Vec::new();
            for y in 0..canvas.h() {
                for x in 0..canvas.w() {
                    clusters.push(canvas.get(x, y).map(|c| c.cluster.clone()));
                    paints.push(canvas.get(x, y).map(|c| c.paint));
                }
            }
            (clusters, paints)
        }

        // 1. Two cluster surfaces over three rungs.
        let mut said: Vec<Vec<Option<String>>> = Vec::new();
        for rung in RUNGS {
            let (clusters, _) = surfaces(rung, Palette::Custom);
            if !said.contains(&clusters) {
                said.push(clusters);
            }
        }
        assert_eq!(
            said.len(),
            2,
            "the glyph axis is a space and the half block"
        );

        // 2. Two paint surfaces over the two palettes.
        let (_, pixels) = surfaces(RUNGS[2], Palette::Custom);
        let (_, described) = surfaces(RUNGS[2], Palette::Roles);
        assert_ne!(pixels, described, "the palette axis draws one thing");

        // 3. And the description is one construction: the same paints at every rung, and never
        //    more than the thirteen roles it is made of.
        let mut painted: Vec<Vec<Option<Paint>>> = Vec::new();
        for rung in RUNGS {
            let (_, paints) = surfaces(rung, Palette::Roles);
            let mut distinct: Vec<Paint> = Vec::new();
            for paint in paints.iter().flatten() {
                if !distinct.contains(paint) {
                    distinct.push(*paint);
                }
            }
            assert!(
                distinct.len() <= ROLE_RAMP.len(),
                "{} paints from thirteen roles at {rung:?}",
                distinct.len()
            );
            if !painted.contains(&paints) {
                painted.push(paints);
            }
        }
        assert_eq!(
            painted.len(),
            1,
            "the description moved with the repertoire, which would make it two constructions"
        );

        assert_eq!(CONSTRUCTIONS, said.len() + 1);
    }

    /// **Two pictures at two call sites are two widgets**, and one line of this file decides it.
    ///
    /// `Ctx::id` mints from `Location::caller()` and the attribute propagates only through functions
    /// that carry it. The first version of [`picture_into`] took its id inside `draw_picture` — a
    /// plain `fn` — so **every picture in a program had the same id**, whatever the call site, and
    /// the two ways to see it are both quiet: a picture declares no region, so nothing merges and
    /// nothing is lost on the screen; and `Response::id` is a value a caller may key on.
    ///
    /// It is `Ctx::id`'s own documented trap — *put `#[track_caller]` on a function that draws one
    /// widget* — with the half nobody states: the attribute has to reach **the line that calls
    /// `id`**, and a private body one frame down swallows it.
    #[test]
    fn two_pictures_at_two_call_sites_are_two_widgets() {
        let modules = Modules::new(2, vec![true, false, false, true]);
        let mut ids = Vec::new();
        let mut driver = Driver::headless(8, 4).expect("a sink attaches");
        driver.frame(|cx| {
            let area = cx.area();
            ids.push(picture(cx, area, &Ramp).id);
            ids.push(picture(cx, area, &Ramp).id);
            ids.push(qr(cx, area, &modules).id);
            ids.push(qr(cx, area, &modules).id);
            ids.push(barcode(cx, area, &[true]).id);
            ids.push(barcode(cx, area, &[true]).id);
        });
        let mut distinct: Vec<vitui_runtime::Id> = Vec::new();
        for id in &ids {
            if !distinct.contains(id) {
                distinct.push(*id);
            }
        }
        assert_eq!(
            distinct.len(),
            ids.len(),
            "two call sites collided on one id: {ids:?}"
        );

        // The other direction, or the assertion above is one an id function that returned a fresh
        // value every call would also pass: the **same** call site drawn twice is one widget.
        let mut twice = Vec::new();
        driver.frame(|cx| {
            let area = cx.area();
            for _ in 0..2 {
                twice.push(picture(cx, area, &Ramp).id);
            }
        });
        assert_eq!(twice[0], twice[1], "one call site minted two ids");
    }

    /// **Nothing but the chrome declares a region.**
    #[test]
    fn the_drawers_declare_nothing_and_the_chrome_declares_something() {
        let modules = Modules::new(2, vec![true, false, false, true]);
        let mut driver = Driver::headless(20, 6).expect("a sink attaches");
        driver.frame(|cx| {
            let area = cx.area();
            picture(cx, area, &Ramp);
            qr(cx, area, &modules);
            barcode(cx, area, &[true, false]);
            waveform(cx, area, &[0.5, -0.5]);
            spectrum(cx, area, &[0.5]);
            vu_meter(cx, area, &[0.5]);
        });
        assert_eq!(driver.inspect().hits().len(), 0);
    }
}
