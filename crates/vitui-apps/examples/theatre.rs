//! A media viewer where every claim the media family makes is a key you can press.
//!
//! The media family's application. It is the first thing to put a
//! [`picture`](vitui_components::media::picture) anywhere, and it exists for the reason every file
//! here exists: **the surface's only consumer is an application**, and nine of the nine before it
//! found something their own gates could not see.
//!
//! ```text
//!   Photo / Ramp          the caller's data: a `Pixels`, asked for one sub-cell at a time
//!     |
//!   picture(…)            one verb a cell, one `Theme::custom` a cell, no region at all
//!     |
//!   the terminal          two colours a cell, whatever glyph the cell holds
//! ```
//!
//! # What it demonstrates
//!
//! | key | what it shows |
//! |---|---|
//! | `1` `2` `3` `4` | truecolor / 256 / 16 / **none**. The axis with a floor of *nothing* |
//! | `g` | the repertoire rung: Ascii, Unicode, Extended. Watch the top two be identical |
//! | `s` | the source: a photograph or a gradient. It moves the *distinctions*, not the wire |
//! | `r` | draw the picture from the theme's thirteen roles instead. Zero customs, and a description |
//! | `p` | the pairing inverted — `▄` where `▀` is right. **The screen still looks like a picture** |
//! | `b` | the plot's ladder on a picture. Watch `g` start to matter, which is what it must not do |
//! | `m` | collect the chapter marks on the draw path — an allocation per frame, kept as a negative case |
//! | `Space` | play / pause. **Nothing advances by itself** — the playhead is the part that needs a clock |
//! | `←` `→` | seek. The value the drag writes, from the keyboard |
//! | drag on the bar | the grab: press jumps, move carries, release moves nothing |
//! | `v` | the live counters, through a `Tally` |
//! | `q` `Ctrl+Q` | quit |
//!
//! **`4` is the one to watch.** Every other component in this library degrades to a worse drawing of
//! itself — a label truncates, a border becomes `+`, a plot loses sub-rows. Press `4` and the
//! picture becomes a **description** of itself: `vitui_components::picture`'s own census is 0 of
//! 23 920 horizontal distinctions kept at [`ColorDepth::None`], and no other screen in the library
//! can produce that number. The QR beside it is unharmed, because two colours that must be *those*
//! two are still two colours.
//!
//! **`g` is the ladder, and `b` is what makes it mean something.** A picture puts a *colour* in
//! every sub-cell and a cell carries two whatever glyph it holds, so braille's 2×4 buys it nothing:
//! Unicode and Extended draw the identical screen. Press `b` and the picture is drawn on the plot's
//! ladder instead — every sub-cell set, eight dots sharing one foreground — and `g` starts changing
//! the screen at the top rung, which is the equality reading something rather than reading a screen
//! that has stopped changing.
//!
//! **`p` is the trap the readback exists for.** The inverted pairing writes the same cells with the
//! same paints one character apart. Nothing on this screen goes wrong, nothing throws, and the
//! picture is vertically mirrored inside every cell. `vitui_components::picture`'s own readback
//! reports 219 of 441 modules wrong for the same mistake on a QR; here there is nothing to compare
//! against, which is the point.
//!
//! # What it deliberately does not do
//!
//! **Nothing here owns a clock.** `Space` flips a flag the chrome draws and the position does not
//! move on its own, because a component that advances a playhead re-registers a deadline while it
//! runs — the mechanism `vitui_components::media::player::Needs::Clock` names and components ticket
//! 42 owns. Seeking is `←`/`→` and the pointer, both of which are the caller's gesture.
//!
//! # Reading it without a terminal
//!
//! ```text
//! cargo run -p vitui-apps --example theatre -- --probe   # one headless frame, and what it cost
//! ```
//!
//! `--probe` draws one frame at 100x30 into a headless sink and prints the census, the counters and
//! the regions. It exists because the numbers this screen is *about* are counts — a picture spends
//! one `Theme::custom` a cell and declares no region at all — and a person reading the screen
//! cannot see either of those.
//!
//! **There is no wallpaper.** See `vitui_components::media::WhyThereIsNoWallpaper`: drawn where a
//! component can put it — first in the base pass — a still screen re-damages 24 000 cells against
//! 1 076 and costs 407 µs against 14, which is the partition rule in a form reordering cannot fix.

use vitui_components::counters::Tally;
use vitui_components::ink::{Direct, Ink};
use vitui_components::media::player::{
    Chapter, Player, chrome_into, defective as chrome_defective, scrub,
};
use vitui_components::media::{
    Census, Modules, Palette, PictureOpts, Pixels, barcode_into, defective, picture_into, qr_into,
    spectrum_into, vu_meter_into, waveform_into,
};
use vitui_components::structure::{PanelOpts, panel_into};
use vitui_components::text::{FitOpts, fit_into};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::layout::rect;
use vitui_runtime::theme::Density;
use vitui_runtime::work::Wake;
use vitui_runtime::{ColorDepth, Ctx, GlyphSet, Rect, Rgb, Role, Themes};

/// How wide the column of symbols and meters on the right is.
const SIDE_W: u16 = 26;

/// How many rows the chrome is given when there is room for its two lists.
const CHROME_TALL: u16 = 10;

/// How many rows it is given otherwise — exactly the transport block.
const CHROME_SHORT: u16 = 4;

/// How far one `←` or `→` seeks, as a fraction of the whole.
const SEEK: f32 = 0.02;

/// The four tiers `1`–`4` select, in the order the keys are on the keyboard.
const TIERS: [ColorDepth; 4] = [
    ColorDepth::TrueColor,
    ColorDepth::Indexed256,
    ColorDepth::Ansi16,
    ColorDepth::None,
];

/// The three rungs `g` cycles.
const RUNGS: [GlyphSet; 3] = [GlyphSet::Ascii, GlyphSet::Unicode, GlyphSet::Extended];

// ── the caller's picture ─────────────────────────────────────────────────────────────────────────

/// **What the picture is a picture of.**
///
/// Two, because `s` is a key and a key needs somewhere to go. Neither is read from a file: a fixture
/// in the repository is a fixture whose bytes nothing checks, and both of these are one line of
/// arithmetic a reader can follow.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    /// Noise with a bounded dynamic range, which is what a photograph is to a quantiser.
    Photograph,
    /// Two linear ramps and a constant — the smoothest frame anything could produce.
    Gradient,
}

impl Source {
    fn word(self) -> &'static str {
        match self {
            Source::Photograph => "photograph",
            Source::Gradient => "gradient",
        }
    }
}

/// The caller's image, in the picture's own coordinates.
///
/// **`sub_y` is a sub-row of the rectangle and `x` a column of it**, and neither says where on the
/// screen the picture landed — which is what lets the same source draw the same picture wherever it
/// is placed, and what makes the frame cost the rectangle rather than the image.
struct Image {
    source: Source,
    /// How many columns wide the picture is, so the gradient's ramp reaches its end at the right
    /// edge rather than saturating part way across a wide terminal.
    cols: u16,
    /// How many sub-rows tall it is, so the ramp is smooth at every rung.
    sub_rows: u32,
}

impl Pixels for Image {
    fn pixel(&self, x: u16, sub_y: u32) -> Rgb {
        match self.source {
            Source::Photograph => {
                let mut h =
                    u32::from(x).wrapping_mul(0x9e37_79b9) ^ sub_y.wrapping_mul(0x85eb_ca6b);
                h ^= h >> 15;
                h = h.wrapping_mul(0xc2b2_ae35);
                h ^= h >> 13;
                // Banded into the middle of the cube: a photograph is not uniform over it, and a
                // source that were would make the sixteen-colour count a fact about a hash.
                let band = |v: u8| 48 + (u16::from(v) * 160 / 255) as u8;
                Rgb::new(band((h >> 16) as u8), band((h >> 8) as u8), band(h as u8))
            }
            Source::Gradient => {
                let r = (u32::from(x) * 255 / u32::from(self.cols.max(1))).min(255) as u8;
                let g = (sub_y * 255 / self.sub_rows.max(1)).min(255) as u8;
                Rgb::new(r, g, 0x80)
            }
        }
    }
}

/// **The symbol, which the caller brings.** A component takes a matrix; encoding a payload into one
/// is a specification with a version, an error-correction level and a mask, and none of those is a
/// fact about a terminal.
fn symbol(side: u16) -> Modules {
    let mut dark = vec![false; usize::from(side) * usize::from(side)];
    let mut set = |x: u16, y: u16, v: bool| {
        dark[usize::from(y) * usize::from(side) + usize::from(x)] = v;
    };
    for y in 0..side {
        for x in 0..side {
            let h = u32::from(x).wrapping_mul(73_856_093) ^ u32::from(y).wrapping_mul(19_349_663);
            set(x, y, (h >> 7) & 1 == 1);
        }
    }
    for (ox, oy) in [(0, 0), (side - 7, 0), (0, side - 7)] {
        for dy in 0..7 {
            for dx in 0..7 {
                let edge = dx == 0 || dy == 0 || dx == 6 || dy == 6;
                let core = (2..=4).contains(&dx) && (2..=4).contains(&dy);
                set(ox + dx, oy + dy, edge || core);
            }
        }
    }
    Modules::new(side, dark)
}

// ── the application ──────────────────────────────────────────────────────────────────────────────

/// Which ladder the picture is drawn on.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Ladder {
    /// A picture's own: one sub-column, one or two sub-rows, two colours.
    Colour,
    /// The plot's, applied to a picture. `b`.
    Braille,
}

struct App {
    tier: usize,
    rung: usize,
    source: Source,
    palette: Palette,
    inverted: bool,
    ladder: Ladder,
    /// Whether the chrome collects its chapter marks on the draw path. `m`.
    collecting: bool,
    counters: bool,
    exit: bool,
    player: Player,
    modules: Modules,
    /// The samples the waveform buckets, built once.
    samples: Vec<f32>,
    /// The barcode's pattern, built once. See `draw_side`.
    bars: Vec<bool>,
    /// The bins the spectrum and the meter read, rebuilt when the position moves.
    bins: Vec<f32>,
    /// What the last frame declared and cost.
    seen: Seen,
    /// The track's rectangle, so the status line can print what the grab was resolved against.
    track_w: u16,
    /// The rows the QR symbol wanted and the rows it got, so a clipped symbol says so.
    qr_rows: (u16, u16),
}

/// What the status line prints, all of it read from the frame that has just drawn.
#[derive(Clone, Copy, Default)]
struct Seen {
    /// **Whether a frame has been read at all.** A figure defaulted to zero is a counter that prints
    /// `0` when it means *nobody counted*.
    read: bool,
    writes: u64,
    distinct: u64,
    verbs: u64,
    customs: u64,
    roles: u64,
    regions: usize,
    /// The scrub the last frame answered with, or `None` for a frame with no grab on the track.
    scrubbed: Option<f32>,
}

impl App {
    fn new() -> App {
        let samples: Vec<f32> = (0..48_000)
            .map(|i| {
                let t = i as f32 / 48_000.0;
                let env = (1.0 - t).powf(0.4);
                env * (t * 240.0).sin() * (0.5 + 0.5 * (t * 11.0).sin())
            })
            .collect();
        let mut app = App {
            tier: 0,
            rung: 2,
            source: Source::Photograph,
            palette: Palette::Custom,
            inverted: false,
            ladder: Ladder::Colour,
            collecting: false,
            counters: false,
            exit: false,
            player: Player::new(
                3_672.0,
                vec![
                    "01 - engine, cells and layers".to_owned(),
                    "02 - the runtime seam".to_owned(),
                    "03 - components, call site first".to_owned(),
                    "04 - what a picture costs".to_owned(),
                ],
                vec![
                    Chapter {
                        at: 0.0,
                        name: "cold open".to_owned(),
                    },
                    Chapter {
                        at: 0.18,
                        name: "titles".to_owned(),
                    },
                    Chapter {
                        at: 0.42,
                        name: "the seam".to_owned(),
                    },
                    Chapter {
                        at: 0.71,
                        name: "measurements".to_owned(),
                    },
                    Chapter {
                        at: 0.93,
                        name: "credits".to_owned(),
                    },
                ],
            ),
            modules: symbol(21),
            samples,
            bars: (0..SIDE_W)
                .map(|i| {
                    !u32::from(i)
                        .wrapping_mul(2_654_435_761)
                        .wrapping_shr(9)
                        .is_multiple_of(5)
                })
                .collect(),
            bins: Vec::new(),
            seen: Seen::default(),
            track_w: 0,
            qr_rows: (0, 0),
        };
        app.player.position = 0.37;
        app.player.subtitle = "-- and that is the whole of the wire argument.".to_owned();
        app.rebuild_bins();
        app
    }

    /// The spectrum's bins, derived when the position moves rather than on the draw path.
    ///
    /// The same discipline `Player::marks` is under, and for the same reason: a `Vec` built inside
    /// the frame is an allocation a frame, and the figure that hides one is an integer mean.
    fn rebuild_bins(&mut self) {
        self.bins.clear();
        let phase = self.player.position * 37.0;
        self.bins.extend((0..SIDE_W).map(|i| {
            let f = f32::from(i) / f32::from(SIDE_W);
            ((1.0 - f).powf(1.6) * (0.6 + 0.4 * (f * 11.0 + phase).sin())).clamp(0.0, 1.0)
        }));
    }

    fn ui(&mut self, cx: &mut Ctx<'_, '_>) {
        let (body, status) = rect::split_at_v(cx.area(), cx.area().h.saturating_sub(1));

        let mut tally = Tally::new();
        let mut direct = Direct;
        // **One call site and one type.** `&mut dyn Ink` satisfies `I: Ink` through the crate's
        // blanket impl, so toggling the counters does not swap the widget — `explorer` found that a
        // toggle which changes *which function* draws a widget mints a different `Id`.
        let mut sink: &mut dyn Ink = match self.counters {
            true => &mut tally,
            false => &mut direct,
        };

        // **`panel_into` and not `panel_with`**, so the frame's own border and padding ring are in
        // the counters: `panel_with` draws through `Direct`, and a `writes` figure that silently
        // omits the container is a figure about part of the screen.
        let panel = panel_into(&mut sink, cx, body, " theatre ", &PanelOpts::default());
        let interior = panel.interior;

        let chrome_h = if interior.h >= CHROME_TALL + 4 {
            CHROME_TALL
        } else {
            CHROME_SHORT.min(interior.h)
        };
        let (upper, chrome_at) = rect::split_at_v(interior, interior.h - chrome_h);
        let side_w = SIDE_W.min(upper.w / 2);
        let (picture_at, side) = rect::split_at_h(upper, upper.w - side_w);

        let mut census = Census::default();
        self.draw_picture(&mut sink, cx, picture_at, &mut census);
        self.qr_rows = self.draw_side(&mut sink, cx, side, &mut census);

        let track = if self.collecting {
            chrome_defective::chrome_collecting_into(
                &mut sink,
                cx,
                chrome_at,
                &mut self.player,
                &mut census,
            )
        } else {
            chrome_into(&mut sink, cx, chrome_at, &mut self.player, &mut census)
        };
        self.track_w = track.rect.w;
        let scrubbed = scrub(&track);
        if let Some(v) = scrubbed {
            self.player.position = v;
        }

        self.status(&mut sink, cx, status, census, scrubbed);

        if self.counters {
            self.seen.writes = tally.writes();
            self.seen.distinct = tally.distinct();
            self.seen.verbs = tally.verbs();
        }
        self.seen.customs = census.customs;
        self.seen.roles = census.roles;
        self.seen.scrubbed = scrubbed;
    }

    /// The picture, on whichever of the three arms is selected.
    fn draw_picture<I: Ink>(
        &self,
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        at: Rect,
        census: &mut Census,
    ) {
        // Four sub-rows a cell is the most any arm here asks for, so the gradient is smooth on all
        // three.
        let src = Image {
            source: self.source,
            cols: at.w.max(1),
            sub_rows: u32::from(at.h.max(1)) * 4,
        };
        let opts = PictureOpts {
            palette: self.palette,
        };
        match (self.ladder, self.inverted) {
            (Ladder::Colour, false) => {
                picture_into(ink, cx, at, &src, &opts, census);
            }
            (Ladder::Colour, true) => {
                defective::picture_inverted_into(ink, cx, at, &src, &opts, census);
            }
            (Ladder::Braille, _) => {
                defective::picture_braille_into(ink, cx, at, &src, &opts, census);
            }
        }
    }

    /// The column of symbols and meters: a QR, a barcode, a waveform, a spectrum and a VU meter.
    ///
    /// Returns the rows the symbol **wanted** and the rows it **got**, so a clipped QR says so on
    /// the status line instead of being a bottom edge nobody looks at.
    fn draw_side<I: Ink>(
        &self,
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        side: Rect,
        census: &mut Census,
    ) -> (u16, u16) {
        if side.w == 0 || side.h == 0 {
            return (self.modules.side(), 0);
        }
        // **The band is what the symbol needs at this rung, and the review caught it being half the
        // column.** A version-1 symbol is eleven rows at two modules a cell and twenty-one at one,
        // and `qr_into` clamps to its rectangle rather than painting past it — so a band capped at
        // `side.h / 2` clips the bottom of the QR at every ordinary terminal size, silently, on the
        // app whose headline names it. The symbol wins over the meters below it, and the status line
        // prints what it wanted against what it got.
        let per_cell = u16::from(vitui_components::media::sub_rows(cx.theme().glyphs())).max(1);
        let qr_h = self.modules.side().div_ceil(per_cell).min(side.h);
        let (qr_at, rest) = rect::split_at_v(side, qr_h);
        qr_into(ink, cx, qr_at, &self.modules, census);
        let rows = (self.modules.side().div_ceil(per_cell), qr_at.h);

        // **The pattern is a field and not a `collect` on the draw path**, which the review caught:
        // it is a pure function of the column index, so a `Vec` built inside the frame is one
        // allocation a frame for a value that never changes. It is the shape
        // `player::defective::chrome_collecting_into` is kept as a negative case *for*, and the
        // note on `rebuild_bins` says the same thing about the other one.
        let band = (rest.h / 4).max(1);
        let (barcode_at, rest) = rect::split_at_v(rest, band.min(rest.h));
        barcode_into(ink, cx, barcode_at, &self.bars, census);

        let (wave_at, rest) = rect::split_at_v(rest, (rest.h / 3).min(rest.h));
        waveform_into(ink, cx, wave_at, &self.samples, census);

        let (spectrum_at, vu_at) = rect::split_at_v(rest, (rest.h / 2).min(rest.h));
        spectrum_into(ink, cx, spectrum_at, &self.bins, census);
        vu_meter_into(ink, cx, vu_at, &self.bins, census);
        rows
    }

    /// **What the frame has just spent, printed under it.**
    fn status<I: Ink>(
        &self,
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        at: Rect,
        census: Census,
        scrubbed: Option<f32>,
    ) {
        let tier = TIERS[self.tier];
        let rung = RUNGS[self.rung];
        let line = format!(
            " {tier:?} · {rung:?} · {} · {} · customs {} · roles {} · regions {} · {} · scrub {} \
             of {} · qr {}/{} rows · {}{}{}",
            self.source.word(),
            match self.palette {
                Palette::Custom => "pixels",
                Palette::Roles => "roles (a description)",
            },
            census.customs,
            census.roles,
            self.seen.regions,
            if self.seen.read && self.counters {
                format!("{} writes / {} verbs", self.seen.writes, self.seen.verbs)
            } else {
                "counters off (v)".to_owned()
            },
            scrubbed.map_or("-".to_owned(), |v| format!("{v:.4}")),
            self.track_w,
            self.qr_rows.1,
            self.qr_rows.0,
            if self.player.playing {
                "playing"
            } else {
                "paused"
            },
            if self.inverted {
                " · PAIRING INVERTED"
            } else {
                ""
            },
            if self.collecting {
                " · COLLECTING MARKS"
            } else {
                ""
            },
        );
        fit_into(
            ink,
            cx,
            at,
            &line,
            &FitOpts {
                role: Role::Dim,
                pad: Role::Body,
                ..FitOpts::default()
            },
        );
    }

    /// **What the frame that has just drawn declared**, read after it rather than during it.
    fn read_frame(&mut self, driver: &Driver) {
        self.seen.read = true;
        self.seen.regions = driver.inspect().hits().len();
    }

    /// **The application's own keys, read from what nothing wanted.**
    ///
    /// `explorer`'s arrangement and its reason: a key an application owns cannot be read inside the
    /// draw, because a component that declines one ends the level's turn at the queue.
    fn take_unhandled(&mut self, keys: &[Pressed]) -> bool {
        let mut theme_moved = false;
        for key in keys {
            match key.code {
                Code::Char('q') => self.exit = true,
                Code::Char('1' | '2' | '3' | '4') => {
                    if let Code::Char(c) = key.code {
                        self.tier = usize::from(c as u8 - b'1');
                        theme_moved = true;
                    }
                }
                Code::Char('g') => {
                    self.rung = (self.rung + 1) % RUNGS.len();
                    theme_moved = true;
                }
                Code::Char('s') => {
                    self.source = match self.source {
                        Source::Photograph => Source::Gradient,
                        Source::Gradient => Source::Photograph,
                    };
                }
                Code::Char('r') => {
                    self.palette = match self.palette {
                        Palette::Custom => Palette::Roles,
                        Palette::Roles => Palette::Custom,
                    };
                }
                Code::Char('p') => self.inverted = !self.inverted,
                Code::Char('b') => {
                    self.ladder = match self.ladder {
                        Ladder::Colour => Ladder::Braille,
                        Ladder::Braille => Ladder::Colour,
                    };
                }
                Code::Char('m') => self.collecting = !self.collecting,
                Code::Char('v') => self.counters = !self.counters,
                Code::Char(' ') => self.player.playing = !self.player.playing,
                Code::Left => {
                    self.player.position = (self.player.position - SEEK).clamp(0.0, 1.0);
                    self.rebuild_bins();
                }
                Code::Right => {
                    self.player.position = (self.player.position + SEEK).clamp(0.0, 1.0);
                    self.rebuild_bins();
                }
                _ => {}
            }
        }
        theme_moved
    }
}

/// **One headless frame, and what it cost.** See the module header's `--probe`.
///
/// Two frames and not one: the frame structures take their allocation on the first that needs one,
/// so a cold frame is not a frame — the same reason `vitui_components::picture::Session` exists.
fn probe(app: &mut App) {
    const W: u16 = 100;
    const H: u16 = 30;
    let mut driver = match Driver::headless(W, H) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("a headless sink could not attach: {why}");
            return;
        }
    };
    let mut themes = Themes::standard();
    themes.set_glyphs(RUNGS[app.rung]);
    themes.set_tier(TIERS[app.tier]);
    driver.set_theme(*themes.theme());
    app.counters = true;
    driver.frame(|cx| app.ui(cx));
    driver.frame(|cx| app.ui(cx));
    app.read_frame(&driver);
    println!("theatre, one frame at {W}x{H}:");
    let cells = u64::from(W) * u64::from(H);
    println!("  writes       {:>8}   of {cells}", app.seen.writes);
    println!(
        "  distinct     {:>8}   every cell; the {} over it are the chapter marks and the thumb, \
         which sit on the track run they are written over",
        app.seen.distinct,
        app.seen.writes - app.seen.distinct
    );
    println!("  verbs        {:>8}", app.seen.verbs);
    println!(
        "  customs      {:>8}   one a cell for the picture, four for the QR, two for the barcode",
        app.seen.customs
    );
    println!("  roles        {:>8}", app.seen.roles);
    println!(
        "  regions      {:>8}   the panel, the five transport buttons and the track. Not one of \
         them is a picture, a symbol or a meter: those declare nothing at all",
        app.seen.regions
    );
    println!(
        "  qr rows      {:>8}   of the {} a version-1 symbol needs at this rung. Fewer is a \
         clipped symbol, which `qr_into` does rather than paint past its rectangle",
        app.qr_rows.1, app.qr_rows.0
    );
    println!("  scrub        {:>8}   nothing is held", "-");
}

fn main() {
    let mut app = App::new();
    if std::env::args().any(|a| a == "--probe") {
        probe(&mut app);
        return;
    }
    let mut themes = Themes::standard();
    themes.set_glyphs(RUNGS[app.rung]);
    themes.set_tier(TIERS[app.tier]);

    let mut driver = match Driver::attach(Default::default(), *themes.theme()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };
    themes.set_density(Density::default());

    loop {
        driver.frame(|cx| app.ui(cx));
        app.read_frame(&driver);
        // **What nothing wanted, read from the frame that has just drawn.** `Driver::unhandled` is a
        // window onto the same queue, valid until the next frame begins, so an application that
        // reads it before its own frame acts one wake late — which for a single keystroke means
        // never.
        let unhandled: Vec<Pressed> = driver.unhandled().to_vec();
        let theme_moved = app.take_unhandled(&unhandled);
        if app.exit {
            break;
        }
        if theme_moved {
            themes.set_glyphs(RUNGS[app.rung]);
            themes.set_tier(TIERS[app.tier]);
            driver.set_theme(*themes.theme());
        }
        // Something arrived, so the next frame is drawn without parking: the state the keys changed
        // is only on the screen once it has been drawn again.
        if !unhandled.is_empty() {
            continue;
        }
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
    }
}
