//! A build pipeline whose spinners and playhead are the two things on this map that own an anchor.
//!
//! The spinner's application. It exists because the last row of the v1 freeze —
//! [`spinner`](vitui_components::indicate::spinner) — is the
//! last one built, and because the interesting number about a component that animates is **not the
//! motion**. It is the *cadence*: how often the screen has to wake up, which is what an anchor buys
//! and a stored phase cannot.
//!
//! ```text
//! ┌─ pipeline — a component may own an anchor and may not own a clock ─────────┐
//! │ ⠹ fetch                        done                                       │
//! │ ⠸ resolve                      done                                       │
//! │ ⠼ compile                      working                                    │   spinners
//! │ ⠴ link                         queued                                     │
//! │ ─────────────────────────────────────────────────────────────────────────  │
//! │ ▶  ⏮  ⏭   ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░   │   the playhead
//! │ 04:12 / 120:00                                                            │
//! │ wakes 41  asks/frame 4  named 5  cadence column  spin 80 ms  rung extended │
//! └───────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! # What it demonstrates
//!
//! | key | what it shows |
//! |---|---|
//! | `space` | play and pause the playhead |
//! | `c` | **the cadence.** Swap *ask when the drawn column moves* for *ask every frame* and watch the wake counter |
//! | `s` | stop and start the spinners. A stopped one is quiet on the **next** frame |
//! | `f` | make the spinner faster, so the two cadences are visibly different rates |
//! | `g` | step the glyph rung: `\|/-\` at ASCII, the quadrant orbit at Unicode, braille at Extended |
//! | `x` | clip the third spinner to nothing. The screen does not change and the wake counter does |
//! | `v` | the live counters, through a `Tally` |
//! | `q` `Ctrl+Q` `Ctrl+C` | quit |
//!
//! **`c` is the one to press, and it is the reason this application exists.** Both arms draw the
//! identical screen. The status line's `wakes` counter is what separates them, and over a whole run
//! the ratio is **7 199×**: a two-hour film on a sixty-column bar is 60 wakes when the playhead asks
//! for *the instant the drawn column moves* and 431 991 when it asks at the frame rate. The first is
//! only computable from an anchor — it is a function of `(anchor, duration, width)` — so the cadence
//! is a **consequence** of owning one rather than an optimisation available to any component that
//! animates.
//!
//! **`x` is the rule that has no picture.** The third spinner is handed a rectangle with no cells in
//! it: it runs, writes nothing, and under the shipped rule asks for nothing. The screen is cell for
//! cell the same either way and one of the two keeps the terminal awake — which is why
//! `vitui_components::indicate::defective::spinner_asking_always_into` is a runnable arm rather than
//! a paragraph, and why this key prints a counter instead of showing you something.
//!
//! **`g` is the ladder, and the middle rung is the correction.** The prototype
//! put the braille spinner at `Unicode`; the engine's own `GlyphSet` says `Unicode` is *Unicode a
//! normal text font covers* and braille is `Extended`, so a terminal that kept the middle promise
//! would have rendered tofu. The middle rung is the quadrant blocks — an orbiting dot rather than a
//! rotating line — which is why `spinner` is **three** constructions and not two.
//!
//! # What this application cannot say
//!
//! **It cannot show you a spinner that samples its own clock**, because no such spelling ships: the
//! rule is *stored state may be an anchor, never a phase*, and
//! `vitui_components::indicate::tests::no_component_body_in_this_crate_samples_its_own_clock` is the
//! scan that keeps it true. What a screen would show is nothing at all — the defect is invisible
//! until a **gate** pins the clock, and then every screen the component appears on loses the ability
//! to advance time.
//!
//! **It cannot name the widget that is keeping it awake.** `Driver::inspect().wakes()` answers *how
//! many widgets named themselves* and *how many times this id asked*, and the chrome's own id is not
//! on the value `chrome_playing_into` returns — that is the **track's**, so a caller can hand it to
//! `scrub`. The runaway's **line** is reachable and is what the status bar prints, which is
//! `#[track_caller]` working: it is this file's line and never the library's.
//!
//! ```text
//! cargo run -p vitui-apps --example pipeline
//! cargo run -p vitui-apps --example pipeline -- --probe
//! ```

use std::fmt::Write as _;
use std::time::Duration;

use vitui_components::chart::raster::RUNGS;
use vitui_components::counters::Tally;
use vitui_components::indicate::{SpinOpts, SpinState, spinner_into};
use vitui_components::ink::{Direct, Ink};
use vitui_components::media::Census;
use vitui_components::media::player::{
    Cadence, Chapter, Player, Playhead, chrome_playing_into, scrub,
};
use vitui_components::structure::{PanelOpts, panel_into};
use vitui_components::text::{FitOpts, fit_into};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::layout::rect;
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Rect, Themes};

/// The stages the spinners stand beside.
const STAGES: [&str; 4] = ["fetch", "resolve", "compile", "link"];

/// How long one ladder frame lasts at rest. Eighty milliseconds is what a person reads as motion.
const SLOW: Duration = Duration::from_millis(80);

/// What `f` swaps it for.
const FAST: Duration = Duration::from_millis(30);

/// The nominal frame interval `Cadence::EveryFrame` reaches for. **Nominal is the point**: the arm
/// has nothing better, and `vitui_runtime::anim` measures a nominal `dt` at 16.9% slow over three
/// seconds — a shortfall that is a *rate* and so grows with the run.
const FRAME_DT: Duration = Duration::from_micros(16_667);

/// **How long the thing being played is: two hours**, which is the length the prototype
/// measured the cadence over — so the number this application prints is the number the map records
/// rather than a scaled version of it.
const FILM_S: f32 = 7_200.0;

/// What the last frame reported.
#[derive(Clone, Copy, Default)]
struct Counters {
    writes: u64,
    distinct: u64,
    verbs: u64,
    regions: usize,
    /// **Cumulative**, and the status line prints the delta beside it — the prototype's own
    /// recorded trap: `WakeLedger`'s counters are never reset, so a total saturates and a spinner
    /// nobody stops measures exactly what a stopped one does.
    asks: u64,
    per_frame: u64,
    named: usize,
}

/// The screen.
struct App {
    /// One anchor a stage. Four call sites, so four widgets, so four `SpinState`s.
    spin: [SpinState; STAGES.len()],
    per: Duration,
    spinning: bool,
    /// Whether the third spinner is handed a rectangle with no cells in it.
    clip: bool,
    player: Player,
    head: Playhead,
    rung: usize,
    counting: bool,
    counters: Counters,
    status: String,
    exit: bool,
}

impl App {
    fn new() -> App {
        App {
            spin: [SpinState::new(); STAGES.len()],
            per: SLOW,
            spinning: true,
            clip: false,
            player: Player::new(
                FILM_S,
                vec!["release build".to_owned()],
                vec![
                    Chapter {
                        at: 0.0,
                        name: "cold".to_owned(),
                    },
                    Chapter {
                        at: 0.4,
                        name: "codegen".to_owned(),
                    },
                    Chapter {
                        at: 0.8,
                        name: "link".to_owned(),
                    },
                ],
            ),
            head: Playhead::paused(),
            rung: 2,
            counting: false,
            counters: Counters::default(),
            status: String::with_capacity(160),
            exit: false,
        }
    }

    /// One frame, top to bottom.
    fn ui<I: Ink>(&mut self, ink: &mut I, cx: &mut Ctx<'_, '_>) {
        let whole = cx.area();
        let panel = panel_into(
            ink,
            cx,
            whole,
            " pipeline — a component may own an anchor and may not own a clock ",
            &PanelOpts::default(),
        );
        let interior = panel.interior;
        if interior.h < 8 || interior.w < 40 {
            return;
        }
        let (stages, rest) = rect::split_at_v(interior, STAGES.len() as u16);
        let (chrome, status) = rect::split_at_v(rest, rest.h.saturating_sub(1));

        // **The anchor is re-set from the frame's own clock every frame, and that is not a
        // contradiction.** A `Steps` re-anchored at `now` and one anchored an hour ago answer the
        // same question at the same `now` while the period is the same, so what this actually does
        // is make `f` take effect on the frame that pressed it. The clock is `Ctx::now` either way.
        let now = cx.now();
        let per = self.per;
        let spinning = self.spinning;
        for (row, (st, stage)) in self.spin.iter_mut().zip(STAGES).enumerate() {
            if spinning {
                st.start(now, per);
            } else {
                st.stop();
            }
            let mut area = Rect::new(
                stages.x,
                stages.y + i32::try_from(row).unwrap_or(0),
                stages.w,
                1,
            );
            // **The clipped case**, which is the rule that has no picture: the component runs and
            // every verb is a no-op.
            if self.clip && row == 2 {
                area.w = 0;
            }
            let label = format!("{stage:<12}{}", if spinning { "working" } else { "idle" });
            // **Keyed**, because one call in one loop is one `Location::caller()` and four spinners
            // under one id would be one widget the census could not tell apart.
            cx.with_key(u64::try_from(row).unwrap_or(0), |cx| {
                spinner_into(ink, cx, area, st, &label, &SpinOpts::default());
            });
        }

        // **The playhead, which is the chrome's seventh part.** The position comes from the anchor,
        // the scrub goes back into it, and the ask fires when the drawn column will move.
        let track = chrome_playing_into(
            ink,
            cx,
            chrome,
            &mut self.player,
            &mut self.head,
            &mut Census::default(),
            FRAME_DT,
        );
        // The grab is the caller's to read too — a press on the track is a seek, and it is the same
        // `Response` the component already used.
        let _ = scrub(&track);

        self.status.clear();
        let _ = write!(
            self.status,
            "wakes {}  asks/frame {}  named {}  cadence {}  spin {} ms  rung {}  {}",
            self.counters.asks,
            self.counters.per_frame,
            self.counters.named,
            match self.head.cadence {
                Cadence::NextCellChange => "column",
                Cadence::EveryFrame => "EVERY FRAME",
            },
            self.per.as_millis(),
            rung_word(self.rung),
            if self.clip { "3rd clipped" } else { "" },
        );
        if self.counting {
            let _ = write!(
                self.status,
                "  writes {} distinct {} verbs {} regions {}",
                self.counters.writes,
                self.counters.distinct,
                self.counters.verbs,
                self.counters.regions,
            );
        }
        let _ = write!(
            self.status,
            "   space play  c cadence  s spin  f faster  g rung  x clip  v counters  q quit"
        );
        let _ = fit_into(ink, cx, status, &self.status, &FitOpts::default());
    }

    /// Read what the frame that has just drawn reported.
    fn read_frame(&mut self, driver: &Driver, tally: Option<&Tally>) {
        let wakes = driver.inspect().wakes();
        let asks: u64 = wakes.lines().map(|l| u64::from(l.asks)).sum();
        self.counters.per_frame = asks.saturating_sub(self.counters.asks);
        self.counters.asks = asks;
        self.counters.named = wakes.census_len();
        self.counters.regions = driver.inspect().hits().len();
        if let Some(t) = tally {
            self.counters.writes = t.writes();
            self.counters.distinct = t.distinct();
            self.counters.verbs = t.verbs();
        }
    }

    /// **What nothing wanted, read from the frame that has just drawn.** `Driver::unhandled` is a
    /// window onto the same queue, valid until the next frame begins, so an application that reads
    /// it before its own frame acts one wake late — which for a single keystroke means never.
    ///
    /// Returns whether the rung moved, because **the swap happens between frames and cannot do
    /// better**: a component may not write to the frame's environment mid-frame, so the key records
    /// a selection and the loop applies it.
    fn take_unhandled(&mut self, keys: &[Pressed], now: std::time::Instant) -> bool {
        let mut rung_moved = false;
        for k in keys {
            match k.code {
                Code::Char('q') | Code::Char('Q') => self.exit = true,
                Code::Char('c') if k.mods.ctrl() => self.exit = true,
                Code::Char(' ') => {
                    if self.head.running() {
                        self.head.pause(now, &mut self.player);
                    } else {
                        let from = if self.player.position >= 1.0 {
                            0.0
                        } else {
                            self.player.position
                        };
                        self.head.play(now, from);
                    }
                }
                Code::Char('c') => {
                    self.head.cadence = match self.head.cadence {
                        Cadence::NextCellChange => Cadence::EveryFrame,
                        Cadence::EveryFrame => Cadence::NextCellChange,
                    };
                }
                Code::Char('s') => self.spinning = !self.spinning,
                Code::Char('f') => self.per = if self.per == SLOW { FAST } else { SLOW },
                Code::Char('x') => self.clip = !self.clip,
                Code::Char('v') => self.counting = !self.counting,
                Code::Char('g') => {
                    self.rung = (self.rung + 1) % RUNGS.len();
                    rung_moved = true;
                }
                _ => {}
            }
        }
        rung_moved
    }
}

/// What the status line calls it.
fn rung_word(step: usize) -> &'static str {
    match step {
        0 => "ascii",
        1 => "unicode",
        _ => "extended",
    }
}

/// One headless run, and what it cost. The subject of this screen is a set of counts, so a person
/// reading a report should not have to hold a terminal open — `browse`'s door, for its reason.
fn probe(app: &mut App) {
    let mut driver = match Driver::headless(100, 20) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };
    let mut ink = Direct;
    driver.frame(|cx| app.ui(&mut ink, cx));
    driver.frame(|cx| app.ui(&mut ink, cx));
    let mut tally = Tally::new();
    driver.frame(|cx| app.ui(&mut tally, cx));
    app.read_frame(&driver, Some(&tally));

    println!("vitui pipeline — one headless frame at 100 x 20\n");
    println!("  {}", app.status);
    // **`writes` exceeds `distinct` by the chapter marks plus the thumb, and that is the chrome's
    // own stated exception rather than this screen's defect**: a mark is written over the track run
    // it sits on, and `media::player`'s partition gate asserts the excess is exactly that. Printed
    // as the subtraction so a reader can see which number it is.
    let marks = app.player.marks().len() as u64 + 1;
    println!(
        "\n  writes {}  distinct {}  ({} of them the marks and the thumb)  verbs {}  regions {}",
        app.counters.writes,
        app.counters.distinct,
        app.counters.writes - app.counters.distinct,
        app.counters.verbs,
        app.counters.regions,
    );
    println!("  the chrome's stated overwrite is {marks}");
    println!(
        "  asks this frame {}  widgets that named themselves {}",
        app.counters.per_frame, app.counters.named,
    );

    // **The cadence, as the number the anchor buys.** Both arms draw the identical screen; this is
    // the only place the difference is visible without a stopwatch.
    let mut cell = Playhead::paused();
    cell.cadence = Cadence::NextCellChange;
    let mut every = Playhead::paused();
    every.cadence = Cadence::EveryFrame;
    let w = 60;
    let a = cell.wakes_over_a_run(&app.player, w, FRAME_DT);
    let b = every.wakes_over_a_run(&app.player, w, FRAME_DT);
    println!(
        "\n  one {:.0}-second run on a {w}-column bar:\n    when the column moves  {a}\n    \
         every frame            {b}\n    ratio                  {}x",
        app.player.duration_s,
        b / a.max(1),
    );
}

fn main() {
    if std::env::args().any(|a| a == "--probe") {
        let mut app = App::new();
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
    let mut app = App::new();
    app.head.play(driver.env().now(), 0.0);

    loop {
        let mut tally = Tally::new();
        let mut direct = Direct;
        let counting = app.counting;
        // **One call site, whichever ink.** Two calls would be two `Location::caller()`s, and every
        // widget on this screen would change identity when `v` was pressed — `explorer` is the
        // caller that found it.
        let mut ink: &mut dyn Ink = if counting { &mut tally } else { &mut direct };
        driver.frame(|cx| app.ui(&mut ink, cx));
        app.read_frame(&driver, counting.then_some(&tally));

        let unhandled: Vec<Pressed> = driver.unhandled().to_vec();
        let rung_moved = app.take_unhandled(&unhandled, driver.env().now());
        if app.exit {
            break;
        }
        if rung_moved {
            themes.set_glyphs(RUNGS[app.rung]);
            driver.set_theme(*themes.theme());
        }
        if rung_moved || !unhandled.is_empty() {
            continue;
        }
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
    }
}
