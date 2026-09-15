//! **A live service-latency monitor**, written against the vitui component surface.
//!
//! An invention rather than a port, because nothing anybody has published makes the four things
//! this component is *for* visible at once: the memo chain, the sub-cell ladder, the threshold's
//! second axis, and the split between what a frame costs and what an edit costs.
//!
//! ```text
//!   ┌ p50 / p99 latency ─────────────────┬ requests ────┬ readouts ┐
//!   │  120 ⡇⢀⠄       ⢀⡀                  │ ▂▅█▃▁▄▇▂     │ pts 1M   │
//!   │      ⠓⠚⠦⣀⣀⣀⣀⡠⠔⠊⠉⠑⠢⢄⣀⣀             │ ▁▂▅█▃▁▄▇     │ fold 8ms │
//!   │  ────────────────────────  SLO ────│              │ wire dim │
//!   └────────────────────────────────────┴──────────────┴──────────┘
//! ```
//!
//! # What it demonstrates, and why each of them needs a running program
//!
//! **The frame costs the rectangle and the edit costs the data.** `+` and `-` move the series
//! between 1 000, 100 000 and 1 000 000 points. The readouts show `cells` — the raster's own size —
//! unmoved and the *fold* moving with it. A screenshot cannot show that; two screenshots side by
//! side cannot either, because the interesting part is that one number moved and the other did not.
//!
//! **`fold` and `edit` are two different costs and the program times them apart.** `edit` is
//! `Feed::push`, which is one `Vec::push` per trace and is flat in the volume by construction.
//! `fold` is the [`plot_with`] call, which is where the raster is built — the number the volume
//! actually moves, and the one the memo chain exists to skip on a still frame.
//!
//! **It moves by two orders of magnitude and not three**, and the three was a ratio read off the
//! wrong axis. Over the thousandfold volume this program offers, one `plot_with` call at 80x24 in
//! release measures [`series::FOLD_MICROS`] — [`series::FOLD_RATIO`], because a fold walks the
//! points and then writes a raster whose size is the rectangle. Those three are a **report and
//! never a gate**, and **they are quoted from one home rather than spelled here**: the call being
//! timed is the component crate's, so the number is too, and the `ref` readout reads the constant
//! rather than three digits this file would have to keep true. The **1 849 000x** figure is the *hit
//! against miss* ratio of one fold against a memo that skipped it, which is the chain's figure
//! rather than the volume's, and quoting it as this axis is how the sentence got a third order it
//! never had.
//!
//! **What no readout here can show is the write count.** The flat figure — 21 872 writes at 1k,
//! 100k and 1M — is measured by the crate's own instruments
//! (`cargo run --release --example series_numbers -p vitui-components`), because nothing an
//! application can hold carries a write counter: `Presented` says whether a frame was submitted,
//! and `Driver::inspect` reports the frame's hits, ring and wakes and no verb totals at all. What
//! stands in its place here is `cells`, whose *size is the rectangle* at every volume — the same
//! sentence one measurement over. Claiming a write-count readout is what two of this file's own
//! paragraphs used to do, over a readout column that never had the row.
//!
//! **The ladder degrades, and only one of the two panes notices.** `g` cycles the glyph repertoire.
//! The bar chart is **byte-identical** at Unicode and Extended — block elements are the middle rung
//! by `CONTEXT.md`'s own definition — and the plot is not: braille buys exactly one bit of vertical
//! resolution. At ASCII both become a different construction rather than a worse-looking one.
//!
//! **A threshold carried by a paint alone disappears and nobody is told.** `c` cycles the colour
//! depth and `t` takes the rule off the glyph axis. The paint-only spelling draws a **space** in
//! `Role::Danger` across a row of the plot's own `Role::Dim` cells, so whether it can be seen at
//! all is `roles_differ_on_wire(Danger, Dim)` — and on the shipped Catppuccin Mocha palette that is
//! **false at sixteen colours**, where `(Danger, Body)` is still true and collapses only at `none`.
//! That is *carried on both axes*, and the `wire` readout answers it for the live theme
//! rather than asking the reader to trust this paragraph: the pair that collapses is a fact about a
//! palette, and swapping the palette to make an older sentence true was refused and recorded.
//!
//! **The memo chain is two memos.** The readouts count raster folds against range folds. A resize
//! moves the first and not the second; a new sample moves both; a still frame moves neither.
//!
//! # What writing it found, and it is closed
//!
//! **`Theme::custom` needed a background colour and a component could not read the page's.** There
//! was no `Theme::page()`, no `Role::Page` and no accessor for a role's background, so the series
//! palette derived one from `Theme::is_dark` — the single bit about the page that was readable,
//! right at both ends of the shipped ladder and a guess in the middle. It was recorded in
//! `vitui_components::chart::series_paint` rather than worked around here, and the runtime answered
//! it: [`vitui_runtime::Theme::page`] exists and that function reads it. What the guess cost while
//! it stood is in its documentation — a black halo around every curve on every dark theme, and no
//! gate in the workspace could see it, because none of them reads a background.
//!
//! # Run it
//!
//! ```text
//! cargo run -p vitui-apps --example latency
//! ```
//!
//! `g` repertoire · `c` colour depth · `t` threshold axis · `space` pause · `+`/`-` volume ·
//! `q`/`Ctrl+Q` quit

use std::time::{Duration, Instant};

use vitui_components::chart::raster::{PlotState, RUNGS};
use vitui_components::chart::{Opts, Series, chart_with, plot_with};
use vitui_components::series;
use vitui_components::structure::{PanelOpts, panel_with};
use vitui_components::text::{Justify, TextOpts, text_with};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{ActionId, Chord, Code, KeyMap};
use vitui_runtime::layout::rect;
use vitui_runtime::theme::{CATPPUCCIN_MOCHA, Density};
use vitui_runtime::work::{Task, Wake, WakeHandle, Worker};
use vitui_runtime::{ColorDepth, Config, Ctx, Interest, Rect, Role, Theme};

// ── the feed ─────────────────────────────────────────────────────────────────────────────────────

/// How often a new sample arrives. **Ten a second**, which is fast enough to look live and slow
/// enough that the fold at a million points is visible in the readout rather than a blur.
const TICK: Duration = Duration::from_millis(100);

/// The three data volumes, which is what makes *the frame costs the rectangle* something a reader
/// can watch rather than read.
const VOLUMES: [usize; 3] = [1_000, 100_000, 1_000_000];

/// **The word a readout prints for a volume**, so the widest row fits the column it is written in.
///
/// `pts   1000043 → 1M` is eighteen cells, which is exactly what [`right_width`]'s twenty leaves
/// once the panel has taken its two. Spelling the target `1000000` instead is twenty-three, and the
/// row this column exists to keep readable elides to `…`.
const VOLUME_WORDS: [&str; 3] = ["1k", "100k", "1M"];

/// The SLO the threshold marks, in milliseconds.
const SLO: f32 = 90.0;

/// The colour depths `c` cycles, richest first.
const DEPTHS: [ColorDepth; 4] = [
    ColorDepth::TrueColor,
    ColorDepth::Indexed256,
    ColorDepth::Ansi16,
    ColorDepth::None,
];

/// **The word a readout prints for a depth.** `ColorDepth` is the engine's and this application is
/// not in the business of naming its variants twice, so the word is the ladder's index.
const DEPTH_WORDS: [&str; 4] = ["true", "256", "16", "none"];

/// The word a readout prints for each of [`RUNGS`].
const RUNG_WORDS: [&str; 3] = ["ascii", "unicode", "extended"];

/// **A word table shorter than the ladder it words is a build failure**, and not a readout that
/// repeats its last entry. Both ladders are the other crates' and either may grow a rung.
const _: () = assert!(DEPTH_WORDS.len() == DEPTHS.len());
const _: () = assert!(VOLUME_WORDS.len() == VOLUMES.len());

/// The word for `at` in `words`. `get` and not a clamp: a clamp answers the *last* word for an
/// index off the end, which is a wrong answer wearing a right one's clothes. The `?` is unreachable
/// while the assertions above hold, and it is what makes them the only thing holding.
fn word(words: &[&'static str], at: usize) -> &'static str {
    words.get(at).copied().unwrap_or("?")
}

/// **How wide the readout column is.** Twenty where there is room and a third of the screen where
/// there is not, with twelve as the floor, so the numbers are readable on a wide terminal and still
/// legible on a narrow one rather than elided to `…`. A third of eighty is twenty-six, so eighty
/// columns already takes the ceiling: the divisor binds below sixty and the floor below thirty-six.
fn right_width(total: u16) -> u16 {
    total.saturating_div(3).clamp(12, 20)
}

/// **The synthetic latency trace.** Two series — a p50 that sits well under the SLO and a p99 that
/// crosses it — with a slow drift and occasional one-sample spikes.
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

/// **The synthetic throughput trace**, in requests a second: one series, a slow swell and jitter.
///
/// No spikes, and that is a decision rather than an omission — the bar pane carries no threshold,
/// so a one-sample spike there reads as a defect in the bars rather than as data.
fn throughput(step: u64) -> f32 {
    let t = step as f32 / 40.0;
    let swell = (t * 0.21).cos() * 240.0;
    let jitter = f32::from(((step.wrapping_mul(2_246_822_519) >> 11) % 23) as u16) * 4.0;
    900.0 + swell + jitter
}

/// **The two traces the two panes draw, and the one thing the worker builds.**
///
/// Two series and not one: the pane labelled ` requests ` is a throughput chart, and drawing the
/// latency series into it would make the label fiction — which is what it was until a review read
/// the pane against the ticket. They travel together everywhere, built at one volume, pushed on one
/// tick and replaced by one landing, which is what makes them one type rather than two fields, two
/// `Task`s and two chances to disagree about which volume is in hand.
struct Feed {
    /// p50 and p99, the plot's data.
    latency: Series,
    /// Requests a second, the bar chart's.
    requests: Series,
}

impl Feed {
    /// Both traces at `n` points. **The worker's whole job**, and at a million points it is 250 ms.
    fn build(n: usize) -> Feed {
        Feed {
            latency: Series::build(n, 2),
            requests: Series::build(n, 1),
        }
    }

    /// **One sample onto each trace, which bumps both revisions.** The *edit*, and it is one
    /// `Vec::push` per trace: flat in the volume, which is the half of the sentence this verb is.
    fn push(&mut self, step: u64) {
        self.latency.push(&sample(step));
        self.requests.push(&[throughput(step)]);
    }

    /// How many points each trace holds.
    fn len(&self) -> usize {
        self.latency.len()
    }
}

// ── the application ──────────────────────────────────────────────────────────────────────────────

/// Everything this application knows.
struct App {
    /// The two traces.
    feed: Feed,
    /// The plot's own state, and the memo chain lives in it.
    latency: PlotState,
    /// The chart's.
    requests: PlotState,
    /// Which of [`VOLUMES`] the user has asked for.
    volume: usize,
    /// **Which of [`VOLUMES`] is actually in hand**, and `None` until the worker's first answer
    /// lands. Not `feed.len()`: the traces grow by a sample a tick, so a length comparison reports
    /// *building* for ever a second after it finished. What the readout is about is which question
    /// has been answered.
    loaded: Option<usize>,
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
    /// **What the last edit cost** — `Feed::push`, which is one `Vec::push` a trace and is flat
    /// in the volume. **A report and never a gate.**
    last_edit: Duration,
    /// **What the last fold cost**, timed around the [`plot_with`] call, which is where the raster
    /// is built. The one number that moves when the volume does, and the whole demonstration.
    last_fold: Duration,
    /// What the last frame cost.
    last_frame: Duration,
    /// **The resident thread the feed is rebuilt on, and the one slot its answer lands in.**
    ///
    /// `None` until [`App::employ`], because [`Worker::hire`] takes a `WakeHandle` and the only
    /// place one exists is `Driver::wake` — which needs a driver, which needs a theme, which needs
    /// the application. The order is forced; the `Option` is that order written down.
    off: Option<Offload>,
    /// Whether the theme needs rebuilding before the next frame.
    restyle: bool,
    exit: bool,
}

/// The worker and the one-slot task the feed lands in.
///
/// **The resident worker thread, from an application rather than from a test.** This used to be
/// reachable from nowhere else: `Worker::hire` was public and
/// uninvokable because `Driver` owned its `Screen` privately and dropped the `WakeHandle`.
struct Offload {
    /// **Held to keep the thread alive**, and read by nothing. Dropping a `Worker` closes its
    /// inbox and joins it, so a field nobody names is what the resident thread's lifetime *is*.
    #[expect(dead_code, reason = "the thread lives as long as this field does")]
    worker: Worker,
    /// Keyed on the index into [`VOLUMES`], so two frames asking for the same volume are one
    /// question and the deduplicated call allocates nothing.
    feed: Task<Feed>,
}

/// **Why this application declares its own frames slow**, and it is the thing it exists to show.
///
/// A million-point series is folded into a raster inside one draw. That fold is the demonstration —
/// the readouts count it and time it, and *the frame costs the rectangle and the edit costs the
/// data* is the sentence the whole screen is an argument for — so the work is on the app thread on
/// purpose and `perf.rs`'s detector is right about it: a debug build overran the 16.7 ms budget and
/// aborted.
///
/// **The build was moved off the thread and that was not enough.** `Feed::build` at a million
/// points cost 250 ms and now runs on [`Offload`]'s worker; what remains is the fold, which needs
/// the rectangle and so cannot leave the draw. The engine's diagnostic names both repairs and this
/// is the second one.
///
/// **Scoped, not global.** The permit is a guard around one `frame` call, so the detector is live
/// for everything else this process does — declaring the loop once would delete the only
/// instrument for the whole run.
const FOLDING: &str = "folding a million-point series into a raster, which is the demonstration";

const QUIT: ActionId = 1;
const GLYPHS: ActionId = 2;
const DEPTH: ActionId = 3;
const RULE: ActionId = 4;
const PAUSE: ActionId = 5;
const MORE: ActionId = 6;
const FEWER: ActionId = 7;

fn key_map() -> KeyMap {
    KeyMap::new()
        // **`Ctrl+Q` beside `q`**, which is the workspace's rule and not this screen's taste. No
        // widget here consumes a letter — the keyboard sink takes every key — so `q` works today;
        // the binding is what keeps that true the first time a `field` or a `collection` is added
        // to the readout column, because both eat text-bearing keys before the sink sees them.
        .bind(
            &[
                Chord::key('q'),
                Chord::key('q').ctrl(),
                Chord::new(Code::Escape),
            ],
            QUIT,
            "Quit",
        )
        .bind(&[Chord::key('g')], GLYPHS, "Repertoire")
        .bind(&[Chord::key('c')], DEPTH, "Colour depth")
        .bind(&[Chord::key('t')], RULE, "Threshold axis")
        .bind(&[Chord::new(Code::Char(' '))], PAUSE, "Pause")
        // **`typed` and not `key`, because `+` cannot be pressed without shift.**
        //
        // `Chord::typed('+')` compares `keys::TYPED_INTENT` — the five intent modifiers that are
        // not `SHIFT` — so it matches whichever of the three spellings the terminal sends for the
        // character: the legacy byte, `CSI 43;2u`, and `CSI 61;2;43u` where the base layout is in
        // `code` and the `+` is in `text`. This used to be three
        // chords and a paragraph explaining why none of them worked.
        //
        // The alternate is the fourth spelling and is not a workaround: at kitty flag 1 alone the
        // terminal reports the *unshifted* key with no associated text, so nothing in the process
        // knows a `+` was produced and the only thing left to bind is the keypress itself.
        .bind(
            &[Chord::typed('+'), Chord::key('=').shift()],
            MORE,
            "More points",
        )
        .bind(&[Chord::key('-')], FEWER, "Fewer points")
}

impl App {
    fn new() -> App {
        App {
            // **Empty, and the worker fills it.** [`App::employ`] asks for volume 0 before the
            // first frame, so the feed has exactly one producer at every moment of the process
            // including startup — which is what makes the first frame's request a deduplicated one
            // rather than a second question with an identical answer.
            feed: Feed::build(0),
            latency: PlotState::new(),
            requests: PlotState::new(),
            volume: 0,
            loaded: None,
            rung: 2,
            depth: 0,
            rule: true,
            live: true,
            step: 0,
            last_edit: Duration::ZERO,
            last_fold: Duration::ZERO,
            last_frame: Duration::ZERO,
            off: None,
            restyle: false,
            exit: false,
        }
    }

    /// The theme this application is asking for.
    fn theme(&self) -> Theme {
        Theme::authored(&CATPPUCCIN_MOCHA, RUNGS[self.rung], Density::default())
            .resolve(DEPTHS[self.depth])
    }

    /// **One sample, and the edit it costs.**
    ///
    /// The push bumps both revisions, so the next frame misses both memos of the chain. Timing it
    /// here rather than inside the draw is what keeps the two costs separable: this is the *edit*,
    /// and the fold timed around the plot call is the fold.
    fn tick(&mut self) {
        self.step += 1;
        let started = Instant::now();
        self.feed.push(self.step);
        self.last_edit = started.elapsed();
    }

    /// **Choose another volume.** The whole point of `+` and `-`, and it rebuilds nothing: this
    /// sets the question, [`App::pump`] asks it and the worker answers it. It was called
    /// `resize_data` and resized no data.
    fn select_volume(&mut self, by: i32) {
        self.volume = (self.volume as i32 + by).clamp(0, VOLUMES.len() as i32 - 1) as usize;
    }

    /// **Hire the worker and ask the first question.** Called once, after the driver exists and
    /// before the first frame.
    ///
    /// The priming asks for `self.volume` — the key [`App::pump`] will ask for on the first frame —
    /// so that frame's call is deduplicated and the volume is built **once**, on the worker, like
    /// every later volume. A priming key no frame ever asks for buys nothing at all:
    /// [`Task::request`] dedupes on key equality alone, so the first frame asks a second question,
    /// and the answer to it replaces the feed with an identical copy whose fresh revision costs a
    /// fold nothing changed. That is what an out-of-range sentinel key did here for six commits,
    /// under a comment saying it prevented exactly that.
    fn employ(&mut self, wake: WakeHandle) {
        let worker = Worker::hire(wake);
        let feed = Task::new(&worker);
        let want = self.volume;
        let _ = feed.request(want as u64, move |_| Feed::build(VOLUMES[want]));
        self.off = Some(Offload { worker, feed });
    }

    /// **Ask for the volume the user has chosen, and take the answer if it has arrived.**
    ///
    /// Called unconditionally at the top of every frame, which is what [`Task::request`] is written
    /// for: the key is the volume, two frames asking the same thing are one question, and the
    /// deduplicated call — every frame but the one that changed it — allocates nothing.
    ///
    /// **This is the fix for a panic rather than a flourish.** `Feed::build` at a million points
    /// took **250 ms on the app thread**, and `perf.rs`'s in-loop detector said so and aborted: *the
    /// app thread's iteration took 250.1 ms against a 16.7 ms frame budget*. It was unreachable
    /// until the `+` binding was fixed, and it is the diagnostic working exactly as
    /// describes. The other repair it offers — declaring the work — **could not be written here at
    /// all** until the runtime forwarded it: `permit_slow` is an inherent method on the
    /// engine's `Screen` and this crate may not name the engine, so the diagnostic named a method
    /// the program could not call. `Driver::permit_slow` forwards it now, and the frames below are
    /// bracketed with it.
    fn pump(&mut self) {
        let Some(off) = self.off.as_ref() else {
            return;
        };
        let want = self.volume;
        let _ = off.feed.request(want as u64, move |cancel| {
            let built = Feed::build(VOLUMES[want]);
            // The bracket is a bracket and not a promise: a volume the user has already changed
            // away from is dropped on the worker's thread rather than landed.
            let _ = cancel.cancelled();
            built
        });
        if let Some(landed) = off.feed.take() {
            self.feed = landed;
            self.loaded = Some(want);
        }
    }

    /// Whether a rebuild is in flight, for the readout.
    fn building(&self) -> bool {
        self.loaded != Some(self.volume)
    }

    /// One frame, top to bottom.
    fn ui(&mut self, cx: &mut Ctx<'_, '_>, map: &KeyMap) {
        cx.key_map(map);

        // The rebuild is asked for here and taken here, and it happens on the worker's thread.
        self.pump();

        // **Ask for the next sample before anything is drawn.** A deadline is the one wake sink, so
        // a paused application registers nothing and costs no CPU at all — which is the same budget
        // `scripts/idle-gate.sh` holds the engine to, met by an application.
        if self.live {
            cx.deadline(cx.now() + TICK);
        }

        let area = cx.area();
        let (head, rest) = rect::split_at_v(area, 1);
        let (body, foot) = rect::split_at_v(rest, rest.h.saturating_sub(1));
        header(cx, head);
        footer(cx, foot);

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
        let panel = panel_with(
            cx,
            wide,
            " p50 / p99 latency, ms ",
            "",
            &PanelOpts::default(),
        );
        if !panel.interior.is_empty() {
            // **The fold, timed where it happens.** `Raster::build` runs inside this call and
            // nowhere else, so this is the number the volume moves — by two orders over the
            // thousandfold this program offers, which is `series::FOLD_RATIO` and not the three
            // this comment claimed — and it reads a few nanoseconds on the frames the memo chain
            // hits.
            let started = Instant::now();
            let _ = plot_with(
                cx,
                panel.interior,
                &self.feed.latency,
                &mut self.latency,
                &opts,
            );
            self.last_fold = started.elapsed();
        }

        let panel = panel_with(cx, bars, " requests ", "", &PanelOpts::default());
        if !panel.interior.is_empty() {
            let _ = chart_with(
                cx,
                panel.interior,
                &self.feed.requests,
                &mut self.requests,
                &Opts {
                    threshold: None,
                    ..opts
                },
            );
        }

        let panel = panel_with(cx, readouts, " readouts ", "", &PanelOpts::default());
        if !panel.interior.is_empty() {
            self.readouts(cx, panel.interior);
        }

        // **The keyboard sink**, and nothing holds the focus until this application seats it —
        // `focused().is_none()` and not `!is_focused(sink)`: the second takes
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
                Some(MORE) => self.select_volume(1),
                Some(FEWER) => self.select_volume(-1),
                _ => cx.decline(key),
            }
        }
    }

    /// **The numbers, and they are the argument.**
    ///
    /// Every row is something the component is claiming: the raster's size, which does not move
    /// with the data; the fold, which does; the two memos of the chain counted separately; and
    /// `wire`, which is the live theme's own answer to whether a paint-only threshold could be seen
    /// at this colour depth at all.
    fn readouts(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        // **Paint the ground before writing on it.**
        //
        // `text` paints the band it is given, so the rows below wrote their own background and the
        // rest of this column wrote nothing at all — and a cell nothing writes shows the
        // *terminal's* background, not the theme's. On a page of `#1e1e2e` against a terminal
        // defaulting to black that is a black rectangle from the last readout to the bottom of the
        // panel, with a hard rectangular edge where the text stops.
        //
        // Invisible to every gate here for the same reason the halo was: nothing this crate can run
        // reads a background, and a cell that is never written is a cell no counter counts.
        cx.fill(area, " ", cx.theme().paint(Role::Body));
        // **The pair the paint-only threshold actually stands or falls on.** A rule with
        // `threshold_glyph` off is a space in `Role::Danger` on a row of the plot's `Role::Dim`
        // cells; on this palette the two stop differing on the wire at sixteen colours, while
        // `(Danger, Body)` survives sixteen and collapses only at `none`.
        let wire = cx.theme().roles_differ_on_wire(Role::Danger, Role::Dim);
        let rows = [
            if self.building() {
                format!(
                    "pts   {} \u{2192} {}",
                    self.feed.len(),
                    word(&VOLUME_WORDS, self.volume)
                )
            } else {
                format!("pts   {}", self.feed.len())
            },
            format!("new   {}", self.step),
            format!("rung  {}", word(&RUNG_WORDS, self.rung)),
            format!("col   {}", word(&DEPTH_WORDS, self.depth)),
            format!("rule  {}", if self.rule { "both" } else { "paint" }),
            format!(
                "wire  {}",
                if wire {
                    "danger\u{2260}dim"
                } else {
                    "danger=dim"
                }
            ),
            format!("feed  {}", if self.live { "live" } else { "paused" }),
            String::new(),
            format!("cells {}", self.latency.raster().bytes()),
            format!("folds {}", self.latency.misses()),
            format!("range {}", self.latency.range_misses()),
            String::new(),
            format!("edit  {:.0}us", self.last_edit.as_secs_f64() * 1e6),
            format!("fold  {:.0}us", self.last_fold.as_secs_f64() * 1e6),
            // **The recorded figure for this volume, read and not quoted.** The row above is what
            // the fold cost on this machine a moment ago; this one is what it cost on the machine
            // the number was taken on, and the two being a row apart is the point. It reads
            // `series::FOLD_MICROS` rather than spelling three digits, so the figure has one home
            // in the crate whose call is being timed and a drift is a compile error rather than a
            // paragraph nobody re-measured.
            format!("ref   {}us", series::FOLD_MICROS[self.volume]),
            format!("frame {:.0}us", self.last_frame.as_secs_f64() * 1e6),
        ];
        // **The timings are the last four rows**, and they are found by position rather than by
        // their own text: `starts_with("fold")` also answers `folds`, which is a miss count and not
        // a duration. Four and not three because `ref` is the same quantity as `fold`, measured
        // elsewhere — a recorded figure and not this run's.
        let timings = rows.len() - 4;
        for (row, y) in rows.iter().zip(0u16..) {
            if y >= area.h {
                break;
            }
            let opts = TextOpts {
                role: if usize::from(y) >= timings {
                    Role::Ok
                } else {
                    Role::Body
                },
                ..Default::default()
            };
            // **A row at a time, and each is a partition of its own band.** `text` returns a
            // `Response` rather than the rows below it — that is `text::fit`'s shape and not
            // `text`'s — so the caller cuts the band, which is the partition rule with the caller on
            // the correct side of it.
            let band = rect::shrink(area, 0, y, 0, area.h.saturating_sub(y + 1));
            let _ = text_with(cx, band, row, &opts);
        }
    }
}

/// The title band. A free function, because it reads no application state and `browse` keeps its
/// equivalents the same way.
fn header(cx: &mut Ctx<'_, '_>, area: Rect) {
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

/// The key line. See [`header`].
fn footer(cx: &mut Ctx<'_, '_>, area: Rect) {
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

/// **One frame, timed, with the fold declared.**
///
/// Two call sites — the frame before the first park, and the frame after every wake — and they were
/// the same five lines twice.
fn draw(driver: &mut Driver, app: &mut App, map: &KeyMap) {
    let started = Instant::now();
    {
        let _slow = driver.permit_slow(FOLDING);
        driver.frame(|cx| app.ui(cx, map));
    }
    app.last_frame = started.elapsed();
}

fn main() {
    let map = key_map();
    let mut app = App::new();

    // **Sixty, and the feed's ten hertz is a different number.**
    //
    // `Config::max_frame_rate` is a *ceiling on painting* — a minimum gap, not a tick —
    // and `perf.rs` derives both of its limits from it: the in-loop budget is one frame interval
    // and the observer's stall limit is 64 of them. This application damages on a tick and on
    // input, so it presents about ten times a second; what it is willing to do is sixty, and that
    // is what the field states.
    //
    // **It said ten for one commit and that was a workaround wearing a contract's clothes.** The
    // bars fold cost 476 ns a point then, so a million-point frame took a second and the only way
    // past the stall limit was to widen it. Declaring a low ceiling to survive a slow frame is
    // exactly the shape of edit that is forbidden — a budget moved to fit a measurement — and the
    // fold being 350x faster is what makes the honest number affordable again.
    let config = Config {
        max_frame_rate: 60.0,
        ..Default::default()
    };
    // **The picture arm**: one screen, drawn headlessly and written as SVG. A
    // program nobody outside this machine can see is a program nobody believes in.
    if vitui_apps::pictures::wanted() {
        let mut driver = vitui_apps::pictures::driver(100, 30, app.theme());
        for _ in 0..vitui_apps::pictures::FRAMES {
            driver.frame(|cx| app.ui(cx, &map));
        }
        vitui_apps::pictures::write("latency", &driver);
        return;
    }

    let mut driver = match Driver::attach(config, app.theme()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };

    // **The worker is hired here and nowhere earlier.** `Worker::hire` takes a `WakeHandle`, and
    // the only way to hold one is `Driver::wake` — which was added for
    // exactly this reason, and which is what makes the resident worker thread reachable from an
    // application at all.
    app.employ(driver.wake());

    // The first frame is drawn before the first park: a screen that appears on the first keystroke
    // is a bug.
    draw(&mut driver, &mut app, &map);

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
        draw(&mut driver, &mut app, &map);
    }
}
