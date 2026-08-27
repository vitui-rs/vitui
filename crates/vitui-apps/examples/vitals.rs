//! **`vitals` — six components composed of proved mechanisms, and `g` is the key to press.**
//!
//! Components ticket 34's application, and the only consumer of [`vitui_components::input::checkbox`],
//! [`radio`](vitui_components::input::radio), [`switch`](vitui_components::input::switch),
//! [`meter`](vitui_components::indicate::meter),
//! [`sparkline`](vitui_components::indicate::sparkline) and
//! [`rule`](vitui_components::structure::rule) that is not a gate. Spec §17, §18 R2.
//!
//! ```text
//! cargo run -p vitui-apps --example vitals            # the dashboard
//! cargo run -p vitui-apps --example vitals -- --probe # one headless frame, and what it cost
//! ```
//!
//! # What it is for, and `g` is the key to press
//!
//! §17 froze nine rows at **Tier 2 — composed of proved mechanisms**, and the risk in shipping one
//! is that the composition turns out to introduce something new. Six of them are here, and what a
//! screen can show that a gate cannot is the **repertoire axis**: `g` steps the glyph rung through
//! [`RUNGS`], and three things happen at once.
//!
//! - The **meter** loses its partial cell. At the top two rungs a core at 43% fills two whole cells
//!   and three eighths of a third; at ASCII there are no eighths, so it rounds — which is
//!   `chart`'s ladder `1 / 8 / 8` on a component that is not a chart, and the whole of why the
//!   freeze gives `meter` `constructions: 2`.
//! - The **checkbox** and the **radio** change their marks — `✓` becomes `x`, `•` becomes `*` —
//!   because their state is carried on the glyph axis and §16's rule is that no spelling is blank.
//! - The **switch** does not change at all. Its `glyphs` column in the freeze is **empty**, and that
//!   is the point rather than a hole: its state is two words, the side its knob sits on and the
//!   face — three axes, and only one of them is the palette.
//!
//! # The sparkline is where *frame cost is proportional to visible cells* is visible
//!
//! `a` pushes one sample onto a series of a hundred thousand points and the status bar prints the
//! **fold count**. Press it ten times and the count goes up by ten; hold `Tab` down and it does not
//! move at all. The memo is the caller's — this application owns the `PlotState` — which is rule 2
//! of spec §1 and the reason the component takes it by `&mut`.
//!
//! # Keys
//!
//! | key | what |
//! |---|---|
//! | `Tab` | the next toggle — the ring's key, never the widget's |
//! | `Space` `Enter` | flip the focused toggle |
//! | `a` | push one sample onto the load series, and re-fold |
//! | `g` | step the glyph rung: extended, unicode, ascii |
//! | `r` | reset |
//! | `q` · `Ctrl+Q` · `Esc` | quit |
//!
//! # `q` is a quit key here, and it is the second application on this map where that is true
//!
//! `compose` cannot bind `q` because a focused `field` consumes every text-bearing key into its
//! buffer, and `ledger` and `explorer` cannot because a focused `collection` eats it into a
//! type-ahead buffer. A focused toggle takes `Space` and `Enter` and **nothing else** — every other
//! code is declined and reaches `Driver::unhandled`. `mixer` was the first, for the same shape of
//! reason.
//!
//! # The toggles are drawn in a loop, so the caller keys them
//!
//! `Ctx::id` mints from `Location::caller()` and there is one call site inside the loop below, so
//! without `cx.with_key` all three toggles would be **one** widget: the first would be clickable and
//! the other two inert, with every mark in the right place and the screen looking perfect. That is
//! ADR 0027's defect, and `vitui_components::media`'s chrome shipped it twice.

use std::env;

use vitui_components::chart::Series;
use vitui_components::chart::raster::{Kind, PlotState, RUNGS, geom};
use vitui_components::counters::Tally;
use vitui_components::indicate::{MeterOpts, SparkOpts, meter_into, sparkline_into};
use vitui_components::ink::{Direct, Ink};
use vitui_components::input::{Toggle, ToggleOpts, toggle_into};
use vitui_components::scroll::Orient;
use vitui_components::structure::{PanelOpts, RuleOpts, panel_into, rule_into};
use vitui_components::text::{Justify, TextOpts, text_into};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::layout::rect;
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Mods, Rect, Role, Themes};

/// The three toggles, in draw order: what each is called and which configuration it is.
const TOGGLES: [(&str, Toggle); 3] = [
    ("wrap long lines", Toggle::Check),
    ("auto-refresh", Toggle::Radio),
    ("dark", Toggle::Switch),
];

/// The cores the meters are drawn for. Six, because that is enough to show the partial cell landing
/// on different eighths on one screen.
const CORES: [&str; 6] = ["cpu0", "cpu1", "cpu2", "cpu3", "cpu4", "cpu5"];

/// Where each core starts, chosen so the eighths differ across the column.
const LOAD: [f32; 6] = [0.435, 0.125, 0.875, 0.51, 0.99, 0.03];

/// How wide the label column beside a horizontal meter is.
const LABEL_W: u16 = 8;

/// How wide the reading column after it is.
const READING_W: u16 = 6;

/// How wide the settings column on the left is.
const SETTINGS_W: u16 = 30;

/// How many rows the status bar takes.
const STATUS_H: u16 = 3;

/// How many points the load series holds. **A hundred thousand**, so that *the frame costs the
/// rectangle and the edit costs the data* is a claim with a magnitude behind it.
const POINTS: usize = 100_000;

/// Everything this application knows.
struct App {
    /// The three toggles' values. **The widgets' own, never application data** — spec §1's rule 2,
    /// and the reason each of them takes a `&mut bool` rather than an index into this.
    on: [bool; 3],
    /// The six cores' loads, `0.0..=1.0`.
    cores: [f32; 6],
    /// How full memory is, drawn as the one vertical meter on the screen.
    memory: f32,
    /// The load history the sparkline folds. **The memo is the caller's**, which is what makes the
    /// fold count on the status bar this application's to print.
    history: Series,
    /// That memo.
    spark: PlotState,
    /// Which of [`RUNGS`] the theme is built at. Stepped by `g`.
    rung: usize,
    /// Set inside `take_unhandled`, read by the loop.
    exit: bool,
    /// Whether the rung changed and the theme owes a rebuild between frames.
    rung_moved: bool,
    /// What the frame that has just drawn reported.
    seen: Seen,
    /// Whether `--probe` asked for the counters.
    counters: bool,
}

/// What the status bar prints, gathered during the draw.
#[derive(Clone, Copy, Default)]
struct Seen {
    /// How many sub-cells the current rung offers a prefix. **Read off `chart`'s own ladder** and
    /// not off a table here, which is what `meter` composing on `chart` means at a call site.
    sub: u8,
    /// How many regions the frame declared.
    regions: usize,
    /// How many tab stops it declared.
    stops: usize,
    /// How many times the sparkline's memo has folded, over the life of the program.
    folds: u32,
    /// Cells written. `--probe` only.
    writes: u64,
    /// How many of them were distinct. **The partition equality as a number** — this application
    /// assembles its own rectangles, so §2's rule is its to keep, and three of this crate's tickets
    /// found the same hole in their own applications first.
    distinct: u64,
    /// How many verbs it took.
    verbs: u64,
}

impl App {
    /// The dashboard as it opens.
    fn new() -> App {
        App {
            on: [true, false, true],
            cores: LOAD,
            memory: 0.62,
            history: Series::build(POINTS, 1),
            spark: PlotState::new(),
            // `RUNGS[2]` is the richest, which is what a terminal that has been told nothing gets.
            rung: 2,
            exit: false,
            rung_moved: false,
            seen: Seen::default(),
            counters: false,
        }
    }

    /// One frame, through whichever ink `--probe` chose.
    ///
    /// **One call site and one type.** `&mut dyn Ink` satisfies `I: Ink` through the components
    /// crate's own `impl<T: Ink + ?Sized> Ink for &mut T`, so the counters are a runtime choice
    /// rather than a second copy of the draw — and a second copy is two `Location::caller()`s,
    /// therefore two ids, therefore a screen that looks identical and loses the focus when the
    /// counters are toggled.
    fn ui(&mut self, cx: &mut Ctx<'_, '_>) {
        let mut tally = Tally::new();
        let mut direct = Direct;
        let mut sink: &mut dyn Ink = if self.counters {
            &mut tally
        } else {
            &mut direct
        };
        self.draw(&mut sink, cx);
        if self.counters {
            self.seen.writes = tally.writes();
            self.seen.distinct = tally.distinct();
            self.seen.verbs = tally.verbs();
        }
    }

    /// One frame, top to bottom.
    fn draw<I: Ink>(&mut self, ink: &mut I, cx: &mut Ctx<'_, '_>) {
        // **The ladder, read from `chart` rather than from a constant here.** It is the one number
        // on the status bar that is a component's rather than this application's.
        self.seen.sub = geom(Kind::Bars, cx.theme().glyphs()).sy;

        let block = panel_into(
            ink,
            cx,
            cx.area(),
            " vitals — press g to step the glyph rung ",
            &PanelOpts::default(),
        );
        let interior = block.interior;
        if interior.h < STATUS_H + 8 || interior.w < SETTINGS_W + 24 {
            text_into(
                ink,
                cx,
                interior,
                "vitals needs a larger terminal",
                &TextOpts {
                    justify: Justify::Middle,
                    role: Role::Warn,
                    ..Default::default()
                },
            );
            return;
        }

        let (body, status) = rect::split_at_v(interior, interior.h - STATUS_H);
        let (settings, gauges) = rect::split_at_h(body, SETTINGS_W);
        self.settings(ink, cx, settings);
        self.gauges(ink, cx, gauges);
        self.status(ink, cx, status);
    }

    /// The left column: a rule, the three toggles, a rule, and the memory meter.
    fn settings<I: Ink>(&mut self, ink: &mut I, cx: &mut Ctx<'_, '_>, at: Rect) {
        let dim = RuleOpts {
            justify: Justify::Middle,
            ..RuleOpts::default()
        };
        let (head, rest) = rect::split_at_v(at, 1);
        rule_into(ink, cx, head, " settings ", &dim);

        let (rows, rest) = rect::split_at_v(rest, TOGGLES.len() as u16);
        // **One call site and three widgets**, which is what `cx.with_key` buys. See this file's
        // header for what its absence costs and why the screen would not show it.
        //
        // **The bound is `rows.h` and not `TOGGLES.len()`**, and the tail below is why: a loop that
        // assumed all three fitted would draw the third *outside* the band it was given, over
        // whatever comes next, and the rows nobody reached would keep what the previous screen put
        // there. `mixer` shipped that subtraction wrong on the other axis, and this is the same one.
        let mut drawn = 0u16;
        for (i, (label, kind)) in TOGGLES.iter().enumerate() {
            if i as u16 >= rows.h {
                break;
            }
            drawn += 1;
            let row = Rect::new(rows.x, rows.y + i as i32, rows.w, 1);
            let on = &mut self.on[i];
            cx.with_key(i as u64, |cx| {
                toggle_into(
                    ink,
                    cx,
                    row,
                    label,
                    on,
                    &ToggleOpts {
                        kind: *kind,
                        ..ToggleOpts::default()
                    },
                );
            });
        }
        if drawn < rows.h {
            let tail = Rect::new(rows.x, rows.y + i32::from(drawn), rows.w, rows.h - drawn);
            text_into(ink, cx, tail, "", &TextOpts::default());
        }

        let (gap, rest) = rect::split_at_v(rest, 1);
        rule_into(ink, cx, gap, " memory ", &dim);
        // The one vertical meter on the screen: up is more, and its partial cell comes out of
        // `chart::raster::cluster` rather than out of the module's own run.
        let (bar, right) = rect::split_at_h(rest, 4);
        let (bar, bar_rest) = rect::split_at_v(bar, rest.h.saturating_sub(1));
        meter_into(
            ink,
            cx,
            bar,
            self.memory,
            &MeterOpts {
                orient: Orient::Vertical,
                ..MeterOpts::default()
            },
        );
        text_into(ink, cx, bar_rest, "", &TextOpts::default());
        text_into(
            ink,
            cx,
            right,
            &format!("  {:.0}% of 32 GiB", self.memory * 100.0),
            &TextOpts {
                role: Role::Dim,
                ..Default::default()
            },
        );
    }

    /// The right column: the load sparkline over a rule, then one horizontal meter a core.
    fn gauges<I: Ink>(&mut self, ink: &mut I, cx: &mut Ctx<'_, '_>, at: Rect) {
        let dim = RuleOpts {
            justify: Justify::Middle,
            ..RuleOpts::default()
        };
        let (head, rest) = rect::split_at_v(at, 1);
        rule_into(ink, cx, head, " load, 100 000 samples ", &dim);

        let cores_h = CORES.len() as u16;
        let spark_h = rest.h.saturating_sub(cores_h + 1);
        let (spark, rest) = rect::split_at_v(rest, spark_h);
        sparkline_into(
            ink,
            cx,
            spark,
            &self.history,
            &mut self.spark,
            &SparkOpts::default(),
        );
        self.seen.folds = self.spark.misses();

        let (gap, rows) = rect::split_at_v(rest, 1);
        rule_into(ink, cx, gap, " cores ", &dim);
        for (i, name) in CORES.iter().enumerate() {
            if i as u16 >= rows.h {
                break;
            }
            let row = Rect::new(rows.x, rows.y + i as i32, rows.w, 1);
            let (label, rest) = rect::split_at_h(row, LABEL_W.min(row.w));
            let (track, reading) =
                rect::split_at_h(rest, rest.w.saturating_sub(READING_W.min(rest.w)));
            text_into(
                ink,
                cx,
                label,
                name,
                &TextOpts {
                    role: Role::Title,
                    ..Default::default()
                },
            );
            meter_into(ink, cx, track, self.cores[i], &MeterOpts::default());
            text_into(
                ink,
                cx,
                reading,
                &format!("{:>4.0}%", self.cores[i] * 100.0),
                &TextOpts {
                    justify: Justify::End,
                    role: Role::Dim,
                    ..Default::default()
                },
            );
        }
        // The rows the cores did not reach. **`CORES.len()` is not what fills them** — a first draft
        // of `mixer` computed its tail from the whole list while the loop broke early, leaving the
        // skipped columns written by nobody, and this is the same subtraction on the other axis.
        let drawn = (CORES.len() as u16).min(rows.h);
        if drawn < rows.h {
            let tail = Rect::new(rows.x, rows.y + i32::from(drawn), rows.w, rows.h - drawn);
            text_into(ink, cx, tail, "", &TextOpts::default());
        }
    }

    /// The status bar: what the rung buys, what the memo has done, and the key legend.
    fn status<I: Ink>(&mut self, ink: &mut I, cx: &mut Ctx<'_, '_>, at: Rect) {
        let dim = TextOpts {
            role: Role::Dim,
            ..Default::default()
        };
        let (first, rest) = rect::split_at_v(at, 1);
        let (second, third) = rect::split_at_v(rest, 1);
        text_into(
            ink,
            cx,
            first,
            &format!(
                "rung {}  ·  {} sub-cell{} a prefix  ·  meter constructions {}  ·  \
                 folds {}  ·  regions {}  ·  tab stops {}",
                rung_word(self.rung),
                self.seen.sub,
                if self.seen.sub == 1 { "" } else { "s" },
                if self.seen.sub == 1 {
                    "the ASCII one"
                } else {
                    "the block one"
                },
                self.seen.folds,
                self.seen.regions,
                self.seen.stops,
            ),
            &TextOpts {
                role: if self.seen.sub == 1 {
                    Role::Warn
                } else {
                    Role::Dim
                },
                ..Default::default()
            },
        );
        text_into(
            ink,
            cx,
            second,
            "a checkbox and a radio carry their state on the glyph axis; a switch carries its on \
             three, and only one of them is the palette",
            &dim,
        );
        text_into(
            ink,
            cx,
            third,
            "Tab toggle · Space flip · a one sample · g rung · r reset · q quit",
            &dim,
        );
    }

    /// What the frame that has just drawn declared. `--probe` reads it; the dashboard prints the
    /// two counts on every frame, because they are what a reader is being asked to compare.
    fn read_frame(&mut self, driver: &Driver) {
        self.seen.regions = driver.inspect().hits().len();
        self.seen.stops = driver.inspect().stop_count();
    }

    /// **The application's own keys, read from what nothing wanted.** See this file's header.
    fn take_unhandled(&mut self, keys: &[Pressed]) -> bool {
        let mut moved = false;
        for k in keys {
            match k.code {
                Code::Escape => self.exit = true,
                Code::Char('q') if k.mods.chord().contains(Mods::CTRL) => self.exit = true,
                Code::Char('q') => self.exit = true,
                Code::Char('a') => {
                    // One sample onto a hundred thousand. The memo's key is the revision, so this is
                    // the only thing on this screen that costs a fold.
                    self.history.push(&[next_sample(&self.history)]);
                    moved = true;
                }
                Code::Char('g') => {
                    self.rung = (self.rung + RUNGS.len() - 1) % RUNGS.len();
                    self.rung_moved = true;
                    moved = true;
                }
                Code::Char('r') => {
                    self.on = [true, false, true];
                    self.cores = LOAD;
                    self.memory = 0.62;
                    moved = true;
                }
                _ => {}
            }
        }
        moved
    }
}

/// The word the status bar prints for a rung. **The index is the name here** — `RUNGS` is what
/// `chart::raster` publishes so that a caller who wants a rung does not name a repertoire.
fn rung_word(at: usize) -> &'static str {
    match at {
        0 => "ascii",
        1 => "unicode",
        _ => "extended",
    }
}

/// The next sample, derived from the series' own length so the screen is deterministic.
fn next_sample(series: &Series) -> f32 {
    let n = series.len() as f32;
    (n * 0.37).sin().mul_add(0.4, 0.5)
}

/// **One headless frame, and what it cost.** See the module header's `--probe`.
///
/// Two frames and not one: the frame structures take their allocation on the first that needs one,
/// so a cold frame is not a frame.
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
    app.counters = true;
    driver.frame(|cx| app.ui(cx));
    driver.frame(|cx| app.ui(cx));
    app.read_frame(&driver);
    let cells = u64::from(W) * u64::from(H);
    println!("vitals, one frame at {W}x{H}:");
    println!(
        "  regions      {:>8}   the panel and the three toggles. Six of the components on this \
         screen declare nothing at all: two meters, a sparkline and four rules",
        app.seen.regions
    );
    println!(
        "  tab stops    {:>8}   the three toggles. A meter, a sparkline and a rule are pure \
         drawers",
        app.seen.stops
    );
    println!("  writes       {:>8}   of {cells}", app.seen.writes);
    println!(
        "  distinct     {:>8}   equal to `writes` and to every cell of the panel's interior",
        app.seen.distinct
    );
    println!("  verbs        {:>8}", app.seen.verbs);
    println!(
        "  folds        {:>8}   one, over two frames of a hundred thousand points",
        app.seen.folds
    );

    // The ladder, at all three rungs, which is what `g` steps through on the screen.
    for (at, rung) in RUNGS.iter().enumerate() {
        println!(
            "  {:<9}    {:>8}   sub-cells a prefix",
            rung_word(at),
            geom(Kind::Bars, *rung).sy
        );
    }

    // **And the partition, swept.** An application that assembles its own rectangles owes §2's
    // equality over the ones it made up, and three of this crate's tickets found the hole in their
    // own application rather than in a component — `mixer` left four columns a channel to whatever
    // was there before, and `reader` blanked its body at a horizontal offset. One size proves
    // nothing about the sizes where a band runs out of rows.
    let mut swept = 0usize;
    let mut torn: Vec<(u16, u16)> = Vec::new();
    for w in [20u16, 40, 56, 60, 80, 100, 140, 200] {
        for h in [6u16, 8, 11, 12, 16, 24, 30, 50] {
            swept += 1;
            let mut driver = match Driver::headless(w, h) {
                Ok(driver) => driver,
                Err(_) => continue,
            };
            let mut probe = App::new();
            probe.counters = true;
            driver.frame(|cx| probe.ui(cx));
            driver.frame(|cx| probe.ui(cx));
            let cells = u64::from(w) * u64::from(h);
            if probe.seen.writes != probe.seen.distinct || probe.seen.distinct != cells {
                torn.push((w, h));
            }
        }
    }
    println!(
        "\n  partition    {:>8}   sizes swept, {} of them not a partition{}",
        swept,
        torn.len(),
        if torn.is_empty() {
            String::new()
        } else {
            format!(": {torn:?}")
        }
    );
}

fn main() {
    let mut app = App::new();
    if env::args().any(|a| a == "--probe") {
        probe(&mut app);
        return;
    }

    let mut themes = Themes::standard();
    let mut driver = match Driver::attach(Default::default(), *themes.theme()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };

    loop {
        driver.frame(|cx| app.ui(cx));
        app.read_frame(&driver);
        // **Read immediately after this application's own frame and never before it.** The window is
        // valid until the next frame begins, so a loop that read it before its own frame acted on
        // the previous frame's window — one wake late, which for a single keystroke means never.
        let unhandled: Vec<Pressed> = driver.unhandled().to_vec();
        let moved = app.take_unhandled(&unhandled);
        if app.exit {
            break;
        }
        if app.rung_moved {
            // **The swap happens between frames and it cannot do better**: a component may not write
            // to the frame's environment mid-frame, so the key returns a selection and the loop
            // applies it. `Themes` is what re-imports and re-resolves in one line.
            app.rung_moved = false;
            themes.set_glyphs(RUNGS[app.rung]);
            driver.set_theme(*themes.theme());
        }
        // Something arrived, so the next frame is drawn without parking: the state the keys changed
        // is only on the screen once it has been drawn again.
        if moved || !unhandled.is_empty() {
            continue;
        }
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
    }
}
