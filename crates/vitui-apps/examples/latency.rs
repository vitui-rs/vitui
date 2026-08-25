//! **A live service-latency monitor**, written against the vitui component surface.
//!
//! An invention rather than a port, because nothing anybody has published makes the four things
//! this component is *for* visible at once: the memo chain, the sub-cell ladder, the threshold's
//! second axis, and the split between what a frame costs and what an edit costs.
//!
//! ```text
//!   ┌ p50 / p99 latency ─────────────────┬ requests ────┬ readouts ┐
//!   │  120 ⡇⢀⠄       ⢀⡀                  │ ▂▅█▃▁▄▇▂     │ ● p50    │
//!   │      ⠓⠚⠦⣀⣀⣀⣀⡠⠔⠊⠉⠑⠢⢄⣀⣀             │ ▁▂▅█▃▁▄▇     │ ● p99    │
//!   │  ────────────────────────  SLO ────│              │ points…  │
//!   └────────────────────────────────────┴──────────────┴──────────┘
//! ```
//!
//! # What it demonstrates, and why each of them needs a running program
//!
//! **The frame costs the rectangle and the edit costs the data.** `+` and `-` move the series
//! between 1 000, 100 000 and 1 000 000 points. The readouts show the frame time and the write
//! count unmoved and the *fold* time moving by three orders of magnitude. A screenshot cannot show
//! that; two screenshots side by side cannot either, because the interesting part is that one
//! number moved and the other did not.
//!
//! **The ladder degrades, and only one of the two panes notices.** `g` cycles the glyph repertoire.
//! The bar chart is **byte-identical** at Unicode and Extended — block elements are the middle rung
//! by `CONTEXT.md`'s own definition — and the plot is not: braille buys exactly one bit of vertical
//! resolution. At ASCII both become a different construction rather than a worse-looking one.
//!
//! **A threshold carried by a paint alone disappears and nobody is told.** `c` cycles the colour
//! depth and `t` takes the rule off the glyph axis. At sixteen colours the SLO line drawn in
//! `Role::Danger` alone stops being distinguishable from the body; the same line drawn as a rule as
//! well is still there. That is §16's *carried on both axes*, and the only way to believe it is to
//! watch it happen.
//!
//! **The memo chain is two memos.** The readouts count raster folds against range folds. A resize
//! moves the first and not the second; a new sample moves both; a still frame moves neither.
//!
//! # What writing it found
//!
//! **`Theme::custom` needs a background colour and a component cannot read the page's.** There is
//! no `Theme::page()`, no `Role::Page` and no accessor for a role's background, so the series
//! palette derives one from `Theme::is_dark` — the single bit about the page that is readable. It
//! is correct on both of the shipped ends and it is a guess in the middle. Recorded in
//! `vitui_components::chart::series_paint` rather than worked around here.
//!
//! # Run it
//!
//! ```text
//! cargo run -p vitui-apps --example latency
//! ```
//!
//! `g` repertoire · `c` colour depth · `t` threshold axis · `space` pause · `+`/`-` volume · `q` quit
//!
//! # Two names this file holds and one it refuses to spell
//!
//! `ColorDepth` arrives through `vitui_runtime`'s re-export of the engine's vocabulary (runtime
//! architecture issue 22), which is the seam working rather than a hole in it: an application may
//! hold the terminal's answers without being able to reach a `View`. What this file does **not** do
//! is name a rung — the ladder is [`RUNGS`], because the component crate is where the sub-cell
//! branch lives and a second list here would be the ninth private fallback table §16 was written
//! about.

use std::time::{Duration, Instant};

use vitui_components::chart::raster::{PlotState, RUNGS};
use vitui_components::chart::{Opts, Series, chart_with, plot_with};
use vitui_components::structure::{PanelOpts, panel_with};
use vitui_components::text::{Justify, TextOpts, text_with};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{ActionId, Chord, Code, KeyMap};
use vitui_runtime::layout::rect;
use vitui_runtime::theme::{CATPPUCCIN_MOCHA, Density};
use vitui_runtime::work::Wake;
use vitui_runtime::{ColorDepth, Ctx, Interest, Rect, Role, Theme};

// ── the feed ─────────────────────────────────────────────────────────────────────────────────────

/// How often a new sample arrives. **Ten a second**, which is fast enough to look live and slow
/// enough that the fold at a million points is visible in the readout rather than a blur.
const TICK: Duration = Duration::from_millis(100);

/// The three data volumes, which is what makes *the frame costs the rectangle* something a reader
/// can watch rather than read.
const VOLUMES: [usize; 3] = [1_000, 100_000, 1_000_000];

/// The SLO the threshold marks, in milliseconds.
const SLO: f32 = 90.0;

/// The colour depths `c` cycles, richest first.
const DEPTHS: [ColorDepth; 4] = [
    ColorDepth::TrueColor,
    ColorDepth::Indexed256,
    ColorDepth::Ansi16,
    ColorDepth::None,
];

/// The word a readout prints for a depth. `ColorDepth` is the engine's and this application is not
/// in the business of naming its variants twice, so the word is derived from the ladder's index.
fn depth_word(at: usize) -> &'static str {
    ["true", "256", "16", "none"][at.min(3)]
}

/// **How wide the readout column is.** Eighteen where there is room and a third of the screen
/// where there is not, so the numbers are readable on a wide terminal and still legible on a narrow
/// one rather than elided to `…`.
fn right_width(total: u16) -> u16 {
    total.saturating_div(3).clamp(12, 20)
}

/// The word a readout prints for a rung.
fn rung_word(at: usize) -> &'static str {
    ["ascii", "unicode", "extended"][at.min(2)]
}

/// **The synthetic trace.** Two series — a p50 that sits well under the SLO and a p99 that crosses
/// it — with a slow drift and occasional one-sample spikes.
///
/// The spikes are the point rather than decoration: they are what a stride sample loses and a union
/// keeps, and at a million points into a hundred columns a spike survives a stride with probability
/// one in ten thousand.
fn sample(step: u64) -> [f32; 2] {
    let t = step as f32 / 40.0;
    let drift = (t * 0.37).sin() * 18.0;
    let jitter = f32::from(((step.wrapping_mul(2_654_435_761) >> 9) % 17) as u16) * 0.6;
    let spike = if (step.wrapping_mul(40_503) >> 5) % 97 == 3 {
        70.0
    } else {
        0.0
    };
    [
        58.0 + drift + jitter,
        96.0 + drift * 1.6 + jitter * 2.2 + spike,
    ]
}

// ── the application ──────────────────────────────────────────────────────────────────────────────

/// Everything this application knows.
struct App {
    data: Series,
    /// The plot's own state, and the memo chain lives in it.
    latency: PlotState,
    /// The chart's.
    requests: PlotState,
    /// Which of [`VOLUMES`] the series holds.
    volume: usize,
    /// Which of [`RUNGS`] the theme is told about.
    rung: usize,
    /// Which of [`DEPTHS`] it is resolved for.
    depth: usize,
    /// Whether the SLO rule is carried on the glyph axis as well as on the paint axis.
    rule: bool,
    /// Whether the feed is running.
    live: bool,
    /// How many samples have arrived.
    step: u64,
    /// What the last fold cost. **A report and never a gate** — but the one number that moves when
    /// the volume does, which is the whole demonstration.
    last_fold: Duration,
    /// What the last frame cost.
    last_frame: Duration,
    /// Whether the theme needs rebuilding before the next frame.
    restyle: bool,
    exit: bool,
}

const QUIT: ActionId = 1;
const GLYPHS: ActionId = 2;
const DEPTH: ActionId = 3;
const RULE: ActionId = 4;
const PAUSE: ActionId = 5;
const MORE: ActionId = 6;
const FEWER: ActionId = 7;

fn key_map() -> KeyMap {
    KeyMap::new()
        .bind(&[Chord::key('q'), Chord::new(Code::Escape)], QUIT, "Quit")
        .bind(&[Chord::key('g')], GLYPHS, "Repertoire")
        .bind(&[Chord::key('c')], DEPTH, "Colour depth")
        .bind(&[Chord::key('t')], RULE, "Threshold axis")
        .bind(&[Chord::new(Code::Char(' '))], PAUSE, "Pause")
        // **Three spellings of one key, and the third is the one a real terminal sends.**
        //
        // `Chord::key(c)` carries `Mods::NONE`, and `Chord::matches` compares `SHIFT` because
        // `keys::INTENT` contains it. So a chord on a character you can only *type* with shift can
        // never match on a terminal that reports the modifier: pressing `Shift+=` on Ghostty
        // arrives as the base-layout `=` with `SHIFT` set, and `Chord::key('+')` and
        // `Chord::key('=')` both miss it. Measured through a pty: unshifted `+` and unshifted `=`
        // work, and every shifted spelling — `=`+shift, `+`+shift, with or without the text field —
        // does nothing at all. It is invisible on a legacy terminal, which reports no modifier for
        // a printable byte.
        //
        // This is an application working around a runtime question rather than a fix: for a
        // *character* chord, shift is how the character was produced and not a modifier the author
        // meant, while for a *named key* chord (`Shift+Tab`) it is exactly the modifier. Recorded
        // as a finding rather than decided here.
        .bind(
            &[
                Chord::key('+'),
                Chord::key('=').shift(),
                Chord::key('+').shift(),
            ],
            MORE,
            "More points",
        )
        .bind(&[Chord::key('-')], FEWER, "Fewer points")
}

impl App {
    fn new() -> App {
        App {
            data: Series::build(VOLUMES[0], 2),
            latency: PlotState::new(),
            requests: PlotState::new(),
            volume: 0,
            rung: 2,
            depth: 0,
            rule: true,
            live: true,
            step: 0,
            last_fold: Duration::ZERO,
            last_frame: Duration::ZERO,
            restyle: false,
            exit: false,
        }
    }

    /// The theme this application is asking for.
    fn theme(&self) -> Theme {
        Theme::authored(&CATPPUCCIN_MOCHA, RUNGS[self.rung], Density::default())
            .resolve(DEPTHS[self.depth])
    }

    /// **One sample, and the fold it costs.**
    ///
    /// The push bumps the data's revision, so the next frame misses both memos of the chain. Timing
    /// it here rather than inside the draw is what keeps the two costs separable: this is the
    /// *edit*, and the frame below is the frame.
    fn tick(&mut self) {
        self.step += 1;
        let started = Instant::now();
        self.data.push(&sample(self.step));
        self.last_fold = started.elapsed();
    }

    /// Rebuild the series at another volume. The whole point of `+` and `-`.
    fn resize_data(&mut self, by: i32) {
        let next = (self.volume as i32 + by).clamp(0, VOLUMES.len() as i32 - 1) as usize;
        if next == self.volume {
            return;
        }
        self.volume = next;
        self.data = Series::build(VOLUMES[next], 2);
        self.step = 0;
    }

    /// One frame, top to bottom.
    fn ui(&mut self, cx: &mut Ctx<'_, '_>, map: &KeyMap) {
        cx.key_map(map);

        // **Ask for the next sample before anything is drawn.** A deadline is the one wake sink, so
        // a paused application registers nothing and costs no CPU at all — which is the same budget
        // `scripts/idle-gate.sh` holds the engine to, met by an application.
        if self.live {
            cx.deadline(cx.now() + TICK);
        }

        let area = cx.area();
        let (head, rest) = rect::split_at_v(area, 1);
        let (body, foot) = rect::split_at_v(rest, rest.h.saturating_sub(1));
        self.header(cx, head);
        self.footer(cx, foot);

        // **The readouts are a fixed width and the two panes share what is left.** They carry
        // numbers rather than a picture, so a proportional cut elides them into `…` on an
        // eighty-column terminal — watched happening, which is `text::fit`'s one-cell marker doing
        // its job on a caller that asked for the wrong rectangle.
        let readout_w = right_width(body.w);
        let (panes, readouts) = rect::split_at_h(body, body.w.saturating_sub(readout_w));
        let (wide, bars) = rect::split_at_h(panes, panes.w * 5 / 8);

        let opts = Opts {
            threshold: Some(SLO),
            threshold_glyph: self.rule,
            ..Opts::plot()
        };
        let panel = panel_with(cx, wide, " p50 / p99 latency, ms ", &PanelOpts::default());
        if !panel.interior.is_empty() {
            let _ = plot_with(cx, panel.interior, &self.data, &mut self.latency, &opts);
        }

        let panel = panel_with(cx, bars, " requests ", &PanelOpts::default());
        if !panel.interior.is_empty() {
            let _ = chart_with(
                cx,
                panel.interior,
                &self.data,
                &mut self.requests,
                &Opts {
                    threshold: None,
                    ..opts
                },
            );
        }

        let panel = panel_with(cx, readouts, " readouts ", &PanelOpts::default());
        if !panel.interior.is_empty() {
            self.readouts(cx, panel.interior);
        }

        // **The keyboard sink**, and nothing holds the focus until this application seats it —
        // architecture issue 25. `focused().is_none()` and not `!is_focused(sink)`: the second takes
        // the keyboard back every frame the user has tabbed away.
        let sink = cx.id();
        let _ = cx.interact(sink, area, Interest::FOCUS);
        if cx.focused().is_none() {
            cx.focus(sink);
        }
        while let Some(key) = cx.next_key(sink) {
            match cx.action(&key) {
                Some(QUIT) => self.exit = true,
                Some(GLYPHS) => {
                    self.rung = (self.rung + 1) % RUNGS.len();
                    self.restyle = true;
                }
                Some(DEPTH) => {
                    self.depth = (self.depth + 1) % DEPTHS.len();
                    self.restyle = true;
                }
                Some(RULE) => self.rule = !self.rule,
                Some(PAUSE) => self.live = !self.live,
                Some(MORE) => self.resize_data(1),
                Some(FEWER) => self.resize_data(-1),
                _ => cx.decline(key),
            }
        }
    }

    fn header(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        text_with(
            cx,
            area,
            " vitui — latency monitor · the frame costs the rectangle, the edit costs the data ",
            &TextOpts {
                justify: Justify::Middle,
                role: Role::Title,
                ..Default::default()
            },
        );
    }

    fn footer(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        text_with(
            cx,
            area,
            " g repertoire · c colour · t threshold axis · space pause · +/- points · q quit ",
            &TextOpts {
                justify: Justify::Middle,
                role: Role::Dim,
                ..Default::default()
            },
        );
    }

    /// **The numbers, and they are the argument.**
    ///
    /// Every row is something the component is claiming: the write count that does not move with
    /// the data, the fold that does, and the two memos of the chain counted separately.
    fn readouts(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let rows = [
            format!("pts   {}", VOLUMES[self.volume]),
            format!("new   {}", self.step),
            format!("rung  {}", rung_word(self.rung)),
            format!("col   {}", depth_word(self.depth)),
            format!("rule  {}", if self.rule { "both" } else { "paint" }),
            format!("feed  {}", if self.live { "live" } else { "paused" }),
            String::new(),
            format!("cells {}", self.latency.raster().bytes()),
            format!("folds {}", self.latency.misses()),
            format!("range {}", self.latency.range_misses()),
            String::new(),
            format!("edit  {:.0}us", self.last_fold.as_secs_f64() * 1e6),
            format!("frame {:.0}us", self.last_frame.as_secs_f64() * 1e6),
        ];
        for (i, row) in rows.iter().enumerate() {
            let Ok(y) = u16::try_from(i) else { break };
            if y >= area.h {
                break;
            }
            let opts = TextOpts {
                role: if row.starts_with("edit") || row.starts_with("frame") {
                    Role::Ok
                } else {
                    Role::Body
                },
                ..Default::default()
            };
            // **A row at a time, and each is a partition of its own band.** `text` returns a
            // `Response` rather than the rows below it — that is `text::fit`'s shape and not
            // `text`'s — so the caller cuts the band, which is spec §2's rule with the caller on
            // the correct side of it.
            let band = rect::shrink(area, 0, y, 0, area.h - y - 1);
            let _ = text_with(cx, band, row, &opts);
        }
    }
}

fn main() {
    let map = key_map();
    let mut app = App::new();

    let mut driver = match Driver::attach(Default::default(), app.theme()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };

    // The first frame is drawn before the first park: a screen that appears on the first keystroke
    // is a bug.
    let started = Instant::now();
    driver.frame(|cx| app.ui(cx, &map));
    app.last_frame = started.elapsed();

    while !app.exit {
        match driver.wait() {
            Wake::Quit => break,
            // **The feed advances on its own deadline and on nothing else.** A paused application
            // registers no deadline, so `wait` parks indefinitely and the process costs nothing.
            Wake::Deadline => app.tick(),
            Wake::Input | Wake::Posted => {}
        }
        if app.restyle {
            driver.set_theme(app.theme());
            app.restyle = false;
        }
        let started = Instant::now();
        driver.frame(|cx| app.ui(cx, &map));
        app.last_frame = started.elapsed();
    }
}
