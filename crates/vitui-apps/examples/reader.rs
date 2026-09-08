//! A build log of 120 000 entries in a scroll area whose bars are reserved and whose bands are views.
//!
//! The scroll area's application. It is the first thing to put a
//! `scroll_area` anywhere, and it exists for the reason every file here exists: **the surface's
//! only consumer is an application**, and five of the six before it found a defect their own gates
//! could not see.
//!
//! ```text
//!   the log               the caller's data: heights and a prefix sum, never iterated in a frame
//!     |
//!   scroll_area(…)        reduces its rectangle, opens the scope, draws four bands and two bars
//!     |
//!   cx.visible_rows()     the window, in **content cells**; the body iterates that and nothing else
//! ```
//!
//! # What it demonstrates, and every claim the scroll family makes is a key you can press
//!
//! | key | what it shows |
//! |---|---|
//! | `↑` `↓` `PgUp` `PgDn` | the vertical offset, in content cells |
//! | `←` `→` | the horizontal offset — watch the **header and footer move with it** and the line numbers not |
//! | `Home` `End` | `scroll_into_view`, which is a **delta in content cells** the component applies |
//! | `u` | measure the extent in **rows** instead of `Σ h`. Press `End` afterwards |
//! | `b` | auto-hiding bars: `Hide::Never` against `Hide::WhenItFits` |
//! | `n` | the four bands off and on — and watch `regions` **not move** |
//! | `v` | the live counters, through a `Tally` |
//! | `q` `Ctrl+Q` | quit |
//!
//! **`u` is the one to watch, and it is the whole of the unit rule.** One entry in eight is three
//! rows tall, so `Σ h` is **150 000 cells over 120 000 entries**. Measured in rows the area's
//! furthest offset is 119 986 instead of 149 986, `End` reaches **entry 95 999 of 119 999** and
//! calls it the end, and **nothing on the screen looks wrong** — the thumb is a plausible size in a
//! plausible place, every row is drawn, and the counters do not move. Press `v` first and then `u`:
//! writes, distinct, verbs and regions are the same numbers at both ends of the content. A fifth of
//! the log is unreachable and the only thing that says so is the entry number in the gutter.
//!
//! **`n` is the one-hit-entry rule, and the reason it is a key rather than a sentence.** Four bands
//! come and go and `regions` stays at one plus the two bars. A band that published a scrollable
//! region of its own would be a second scroll area and would win the wheel from the body it is a
//! header of; that this cannot happen is not a promise the component makes, it is a number on the
//! status line.
//!
//! **The band's drawer writes one row more than the band is tall**, deliberately — a second line of
//! the ruler that nothing should ever see. Drawn into a view the overrun is clipped and nothing
//! happens; drawn by arithmetic it would land on the body's first row and the two would fight for
//! those cells for ever. There is no key for the second arm here **because the component gives the
//! caller no way to write it**: a band's body is called inside `sticky`, and `sticky` is a view.
//! `vitui_components::scroll::defective::arithmetic_band` is where the negative case is stood up,
//! and it is measured rather than demonstrated for the same reason `Bars::Overlay` is.
//!
//! # The bands are the caller's content and the offsets are the component's
//!
//! Four bands stand — a header, a footer, a pinned column of entry numbers and the gutter where they
//! meet — and **not one of them declares a hit entry**: the status line's `regions` never moves when
//! they come and go, and why: a band that were a second scroll area would win the wheel
//! from the body it is a header of, and the wheel over this screen belongs to one region.
//!
//! # It does not depend on the facade
//!
//! `vitui-runtime` and `vitui-components`, like every file here. See `crates/vitui-apps/src/lib.rs`.

use std::fmt::Write as _;

use vitui_components::counters::Tally;
use vitui_components::ink::{Direct, Ink};
use vitui_components::scroll::{AreaOpts, AreaState, Hide, Which, parts, scroll_area_into};
use vitui_components::structure::{PanelOpts, panel_with};
use vitui_components::text::{FitOpts, fit_with};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::layout::text::{truncate, width};
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Id, Rect, Role, Themes};

/// How many entries the log holds.
const ENTRIES: u32 = 120_000;
/// One entry in this many is three rows tall — a wrapped stack trace.
const TALL_EVERY: u32 = 8;
/// How tall a tall entry is, in content cells.
const TALL_H: u32 = 3;
/// How wide the content is, in content cells. Wider than any terminal, so the horizontal bar stands.
const COLS: u32 = 200;
/// How many columns of entry number the pinned band takes.
const GUTTER: u16 = 9;

/// **The caller's index, and the component never sees it.**
///
/// `ytop` is the prefix sum of the heights — `ENTRIES + 1` entries, four bytes each — and it is the
/// only structure in this file a frame is not allowed to walk. The engine lays nothing
/// out, and by the same rule a scroll area iterates nothing.
struct Log {
    ytop: Vec<u32>,
}

impl Log {
    fn build() -> Log {
        let mut ytop = Vec::with_capacity(ENTRIES as usize + 1);
        let mut acc = 0u32;
        ytop.push(0);
        for i in 0..ENTRIES {
            acc += if i % TALL_EVERY == 0 { TALL_H } else { 1 };
            ytop.push(acc);
        }
        Log { ytop }
    }

    /// **`Σ h`, in content cells.** The extent, and never the entry count.
    fn cells(&self) -> u32 {
        *self.ytop.last().unwrap_or(&0)
    }

    /// The top of entry `i`, in content cells.
    fn top_of(&self, i: usize) -> u32 {
        self.ytop[i.min(self.ytop.len() - 1)]
    }

    /// How tall entry `i` is.
    fn height(&self, i: usize) -> u32 {
        self.top_of(i + 1) - self.top_of(i)
    }

    /// **The entry whose span contains content cell `y`.** A binary search, once a frame.
    fn entry_at(&self, y: u32) -> usize {
        self.ytop
            .partition_point(|&top| top <= y)
            .saturating_sub(1)
            .min(ENTRIES as usize - 1)
    }
}

/// Which unit the extent is declared in. **`Cells` is the answer and `Rows` is the defect.**
#[derive(Clone, Copy, PartialEq, Eq)]
enum Unit {
    /// `Σ h`.
    Cells,
    /// The entry count, handed over as if it were a length in cells.
    Rows,
}

struct App {
    log: Log,
    st: AreaState,
    unit: Unit,
    hide: Hide,
    bands: bool,
    counters: bool,
    reveal: Option<i32>,
    exit: bool,
    /// What the last frame wrote, when the counters are on.
    seen: (u64, u64, u64, usize),
    /// **How many hit entries the frame before declared**, read from `Driver::inspect` after the
    /// draw. It is on the screen because it is the gate stated as a sentence: four bands stand
    /// and this number does not move.
    regions: usize,
    /// **The viewport and the furthest offset the last frame actually produced.**
    ///
    /// Kept from the draw rather than recomputed in `main`, and the difference was a defect in this
    /// file: the loop rebuilt the panel's interior by hand as *the screen inset by two*, while
    /// `frame::draw` insets by the border and then by the theme's density — three under `Cosy`. So
    /// `PgDn` paged two rows past the real viewport and clamped against a `max` two cells away from
    /// the one the status line printed, which reads the interior the panel returned. **The geometry
    /// has one source and it is the frame that drew it.**
    geometry: (Rect, i32),
}

impl App {
    fn new() -> App {
        App {
            log: Log::build(),
            st: AreaState::default(),
            unit: Unit::Cells,
            hide: Hide::Never,
            bands: true,
            counters: false,
            reveal: None,
            exit: false,
            seen: (0, 0, 0, 0),
            regions: 0,
            geometry: (Rect::new(0, 0, 1, 1), 0),
        }
    }

    /// The extent the area is told about, on both axes.
    fn extent(&self) -> (u32, u32) {
        (
            COLS,
            match self.unit {
                Unit::Cells => self.log.cells(),
                Unit::Rows => ENTRIES,
            },
        )
    }

    fn opts(&self) -> AreaOpts {
        AreaOpts {
            hide: self.hide,
            header: u16::from(self.bands),
            footer: u16::from(self.bands),
            pinned: if self.bands { GUTTER } else { 0 },
            ..AreaOpts::default()
        }
    }

    /// **The application's own keys, read from what nothing wanted.**
    ///
    /// `explorer`'s arrangement and its reason: a key an application owns cannot be read inside the
    /// draw, because a component that declines one ends the level's turn at the queue.
    fn take_unhandled(&mut self, keys: &[Pressed]) {
        let (view, max) = self.geometry;
        let page = i32::from(view.h.max(1));
        for key in keys {
            match key.code {
                Code::Char('q') => self.exit = true,
                Code::Char('u') => {
                    self.unit = match self.unit {
                        Unit::Cells => Unit::Rows,
                        Unit::Rows => Unit::Cells,
                    };
                }
                Code::Char('b') => {
                    self.hide = match self.hide {
                        Hide::Never => Hide::WhenItFits,
                        Hide::WhenItFits => Hide::Never,
                    };
                }
                Code::Char('n') => self.bands = !self.bands,
                Code::Char('v') => self.counters = !self.counters,
                Code::Up => self.st.offset.1 = (self.st.offset.1 - 1).clamp(0, max),
                Code::Down => self.st.offset.1 = (self.st.offset.1 + 1).clamp(0, max),
                Code::PageUp => self.st.offset.1 = (self.st.offset.1 - page).clamp(0, max),
                Code::PageDown => self.st.offset.1 = (self.st.offset.1 + page).clamp(0, max),
                Code::Left => self.st.offset.0 = (self.st.offset.0 - 4).max(0),
                Code::Right => self.st.offset.0 += 4,
                // **A reveal and not a position**, which is the fourth site: the component
                // takes it as a delta in content cells and clamps it against the extent it was
                // declared. Under `u` the last entry is not reachable and `End` stops short.
                Code::Home => self.reveal = Some(0),
                Code::End => {
                    self.reveal = Some(i32::try_from(self.log.cells()).unwrap_or(i32::MAX))
                }
                _ => {}
            }
        }
    }

    fn ui(&mut self, cx: &mut Ctx<'_, '_>) {
        let screen = cx.area();
        let (top, status) = split_last_row(screen);
        let panel = panel_with(cx, top, " build log ", &PanelOpts::default());

        // **The geometry the keys are clamped against comes from here**, where the panel has just
        // told this frame what it handed over. See `App::geometry`.
        let opts = self.opts();
        let extent = self.extent();
        let p = parts(panel.interior, extent, &opts);
        self.geometry = (p.view, p.max_offset(extent).1);

        let mut tally = Tally::new();
        let mut direct = Direct;
        // **One call site and one type**, so the counters toggle does not replace the widget:
        // `&mut dyn Ink` satisfies `I: Ink` through the crate's blanket impl.
        let mut sink: &mut dyn Ink = match self.counters {
            true => &mut tally,
            false => &mut direct,
        };
        self.draw_area(&mut sink, cx, panel.interior);
        if self.counters {
            self.seen = (
                tally.writes(),
                tally.distinct(),
                tally.verbs(),
                cx.area().w as usize,
            );
        }
        self.status(cx, status, panel.interior);
    }

    fn draw_area(&mut self, ink: &mut &mut dyn Ink, cx: &mut Ctx<'_, '_>, area: Rect) {
        let id = Id::named("log");
        let opts = self.opts();
        let extent = self.extent();
        let theme = cx.theme();
        let body = theme.paint(Role::Body);
        let dim = theme.paint(Role::Dim);
        let title = theme.paint(Role::Title);

        let log = &self.log;
        let reveal = self.reveal.take();
        let mut line = String::with_capacity(COLS as usize);

        scroll_area_into(
            ink,
            cx,
            id,
            area,
            &mut self.st,
            &opts,
            extent,
            |ink, cx, band| {
                let paint = match band.which {
                    Which::Header | Which::Footer => title,
                    _ => dim,
                };
                match band.which {
                    // **The ruler, which shares `x`.** It draws at *content* columns, so its ticks
                    // and the body's columns are the same number and cannot disagree.
                    Which::Header | Which::Footer => {
                        for tick in 0..(COLS / 10) {
                            let x = i32::try_from(tick * 10).unwrap_or(0);
                            let mut mark = String::new();
                            let _ = write!(mark, "{:<10}", format!("|{}", tick * 10));
                            ink.text(cx, x, 0, &mark, paint);
                        }
                        // **The overrun, and it never reaches the surface.** One row more than the
                        // band is tall: a view clips it and the arithmetic spelling would put it on
                        // the body's first row for ever. See this file's header.
                        ink.run(cx, 0, 1, "~", 40, paint);
                    }
                    // **The entry numbers, which share `y`.** They draw at content *rows*, so a
                    // number and the entry beside it are the same cell row.
                    Which::Pinned => {
                        let window = cx.visible_rows();
                        let lo = log.entry_at(u32::try_from(window.start.max(0)).unwrap_or(0));
                        for i in lo..ENTRIES as usize {
                            let y = i32::try_from(log.top_of(i)).unwrap_or(i32::MAX);
                            if y >= window.end {
                                break;
                            }
                            let mut n = String::new();
                            let _ = write!(n, "{i:>8} ");
                            ink.text(cx, 0, y, &n, paint);
                        }
                    }
                    // **The gutter shares neither**, so it is the one band whose contents do not
                    // move when anything scrolls.
                    Which::Gutter => {
                        ink.run(cx, 0, 0, " ", band.rect.w, paint);
                    }
                }
            },
            |ink, cx| {
                if let Some(at) = reveal {
                    cx.request_into_view(Rect::new(0, at, 1, 1));
                }
                let window = cx.visible_rows();
                let lo = log.entry_at(u32::try_from(window.start.max(0)).unwrap_or(0));
                for i in lo..ENTRIES as usize {
                    let y = i32::try_from(log.top_of(i)).unwrap_or(i32::MAX);
                    if y >= window.end {
                        break;
                    }
                    for row in 0..log.height(i) {
                        line.clear();
                        entry_line(&mut line, i, row);
                        let cut = truncate(&line, u16::try_from(COLS).unwrap_or(u16::MAX));
                        let at = y + i32::try_from(row).unwrap_or(0);
                        ink.text(cx, 0, at, cut, body);
                        // The rest of the content row, so the body writes a partition of what the
                        // content admits and the component's tail writes the rest.
                        //
                        // **The next content column is the string's own width, not what the engine
                        // reported.** `Written::cells` is columns *written*, which inside a
                        // scrolled scope is what survived the clip: at a horizontal offset of 100
                        // the text lands on content 100..180 and reports 80, and a run started at
                        // content column 80 would blank every visible cell of the row. clamp-and-discard's
                        // clamp-and-discard, met from the caller's side.
                        let used = width(cut);
                        ink.run(
                            cx,
                            i32::from(used),
                            at,
                            " ",
                            u16::try_from(COLS).unwrap_or(u16::MAX) - used,
                            body,
                        );
                    }
                }
            },
        );
    }

    fn status(&self, cx: &mut Ctx<'_, '_>, row: Rect, area: Rect) {
        let opts = self.opts();
        let extent = self.extent();
        let p = parts(area, extent, &opts);
        let max = p.max_offset(extent);
        let entry = self
            .log
            .entry_at(u32::try_from(self.st.offset.1.max(0)).unwrap_or(0));
        let mut s = String::new();
        let _ = write!(
            s,
            "off {}/{} x {}/{}  entry {entry}/{}  extent {} {}  bars v={} h={} passes {}  ",
            self.st.offset.1,
            max.1,
            self.st.offset.0,
            max.0,
            ENTRIES - 1,
            extent.1,
            match self.unit {
                Unit::Cells => "cells (Σ h)",
                Unit::Rows => "ROWS  (the defect)",
            },
            p.shown.v,
            p.shown.h,
            p.passes,
        );
        if self.counters {
            let _ = write!(
                s,
                "writes {} distinct {} verbs {}  ",
                self.seen.0, self.seen.1, self.seen.2
            );
        }
        let _ = write!(
            s,
            "regions {}  {}{}  q quit",
            self.regions,
            if self.bands {
                "n:4 bands "
            } else {
                "n:no bands "
            },
            match self.hide {
                Hide::Never => "b:never",
                Hide::WhenItFits => "b:when-it-fits",
            }
        );
        let _ = fit_with(cx, row, &s, &FitOpts::default());
    }
}

/// One row of one entry, written into the caller's buffer. **No allocation a frame.**
fn entry_line(into: &mut String, i: usize, row: u32) {
    const LEVELS: [&str; 4] = ["INFO ", "WARN ", "ERROR", "DEBUG"];
    // **Invented targets, and one of them used to name the engine.** `tests::no_application_names_
    // _the_engine` reads source files rather than manifests, so a *string literal* spelling the
    // crate this crate must not reach fails it — which is the scan being right: a file that can
    // write the name can write the `use`.
    const TARGETS: [&str; 5] = [
        "damage::rows",
        "runtime::ctx",
        "components::scroll",
        "cargo::core::compiler",
        "rustc_codegen_ssa",
    ];
    if row > 0 {
        let _ = write!(
            into,
            "        at {}:{}:{}",
            TARGETS[(i + row as usize) % TARGETS.len()],
            (i * 7) % 900 + 1,
            (i * 13) % 80 + 1
        );
        return;
    }
    let _ = write!(
        into,
        "{} {:<28} entry {i} of {ENTRIES}: the frame composited {} damaged cells in {}.{:02} us",
        LEVELS[i % LEVELS.len()],
        TARGETS[i % TARGETS.len()],
        (i * 31) % 24_000,
        (i * 17) % 900,
        i % 100
    );
}

/// The screen, less its last row. The status line is the application's and not the area's.
fn split_last_row(screen: Rect) -> (Rect, Rect) {
    let h = screen.h.saturating_sub(1);
    (
        Rect::new(screen.x, screen.y, screen.w, h),
        Rect::new(screen.x, screen.y + i32::from(h), screen.w, screen.h - h),
    )
}

fn main() {
    let mut app = App::new();

    let mut driver = match Driver::attach(Default::default(), *Themes::standard().theme()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };

    loop {
        driver.frame(|cx| app.ui(cx));

        // **What nothing wanted, read from the frame that has just drawn — and *before* any other
        // frame runs.**
        //
        // `Driver::unhandled` is *a window onto the same queue, valid until the next frame begins*,
        // so reading it before this application's frame read the *previous* frame's window and acted
        // one wake late — which for a single keystroke means never. Measured on the shipped binary:
        // `q` did not quit. An application found it and every loop in this crate
        // had it.
        //
        // **This loop once drew a reveal's second frame itself, right here**, and it no longer does.
        // A reveal is a two-frame gesture — `Ctx::request_into_view` leaves the request on the frame
        // and the component applies it through `take_into_view` on the next one — and nothing asked
        // for that second frame, so `End` drew, requested and parked: the list moved on the *next*
        // keystroke. This program is where it was found, because it is the first to drive a reveal
        // from a key an **application** owns. The ask went in the
        // runtime, where both producers meet and where a wake policy belongs; what stands here is
        // the ordinary loop, and the reveal arrives as a `Wake::Deadline` below.
        let unhandled: Vec<Pressed> = driver.unhandled().to_vec();
        app.take_unhandled(&unhandled);

        app.regions = driver.inspect().hits().len();

        if app.exit {
            break;
        }
        if !unhandled.is_empty() {
            continue;
        }
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
    }
}
