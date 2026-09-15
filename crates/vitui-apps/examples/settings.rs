//! A settings screen of twelve sections, where every claim the collapsible makes is a key you can press.
//!
//! The collapsible's application. It is the first thing to put a
//! [`collapsible`] anywhere, and it exists for the reason every file here exists: **the surface's
//! only consumer is an application**, and six of the six before it found a defect their own gates
//! could not see.
//!
//! ```text
//!   the sections          the caller's data: twelve `Collapse`s, four bytes of live state each
//!     |
//!   collapsible(…)        a header row, and a body that is called only when it has height
//!     |
//!   Disclosure::used      the rows it took. Everything below is the caller's, and `t` proves it
//! ```
//!
//! # What it demonstrates
//!
//! | key | what it shows |
//! |---|---|
//! | `Tab` `Shift+Tab` | the focus ring over twelve headers, plus whatever an open body carries |
//! | `Enter` `Space` | toggle the focused section. **The component's own keys**, read inside the draw |
//! | `↑` `↓` `PgUp` `PgDn` | the offset, in rows. The caller culls; a section off screen declares nothing |
//! | `a` | a 200 ms tween on the height. Watch `open` flip **now** and only the height move |
//! | `x` | exclusive: opening one closes the rest — and this is where the vanish rule answers |
//! | `k` | on `x`, whether the application moves the focus before it collapses a body out from under it |
//! | `w` | the open height from the **drawn extent** instead of the sizing function. Watch it latch |
//! | `t` | stop writing the tail. Watch the old body stay on the screen under a correct header |
//! | `v` | the live counters, through a `Tally` |
//! | `q` `Ctrl+Q` | quit |
//!
//! **`w` is the one to watch, and it is the scroll family's precondition on the height axis.** The bodies here
//! paint a background over every row of the rectangle they are handed — which is what a padding ring
//! *is* — so a height measured inside the rectangle the last decision produced can never come back
//! down. Press `w` and open a section: it opens to the **whole panel** and stays there, every row
//! drawn, the chevron correct, nothing on the screen wrong. The drawn extent is priced at *8
//! rows wrong, settled in 3 frames*; over a body that fills what it is given it is not eight rows and
//! it does not settle.
//!
//! **`t` is the partition rule and the seam sentence, on one key.** A section names the rows
//! it took — [`Disclosure::used`] — and every row below is the caller's. Press `t`, then close a
//! section: the header turns, the section's rows go, and the body that was underneath **stays on the
//! screen**. Nothing throws. The screen is plausible. `vitui_components::disclose`'s own gate reports
//! 240 cells over 4 rows for the same mistake.
//!
//! **`x` with `k` is the focus sentence, and the finding is which half of it is reachable.** A
//! section closed by a gesture on its own header cannot reach R08's vanish rule at all — the press
//! award has already moved the focus, three different ways for three different gestures — so the only
//! arm that pays is a collapse **nobody clicked for**. Exclusive mode is one: open section 7 while the
//! focus is inside section 3's body and section 3 goes without being asked. `k` is the other
//! sentence — *`Stash` belongs to whoever owns the content's identity, and the caller does it by
//! capturing `Frame::focus`* — and the probe count on the status line is the difference.
//!
//! # What it deliberately does not do
//!
//! **There is no scroll area here**, and the offset is the application's own. Still open is
//! *whether a collapsible inside a scroll area may learn its content height one frame late under the
//! extent shape*; putting one inside the other and pressing `w` would answer that question by
//! accident, in an application, which is the last place a map decision should arrive from. The
//! culling is the caller's either way — a section entirely below the panel is not drawn and therefore
//! declares nothing, which is *1.05× six open rather than 2×*.
//!
//! # It does not depend on the facade
//!
//! `vitui-runtime` and `vitui-components`, like every file here. See `crates/vitui-apps/src/lib.rs`.
//!
//! [`collapsible`]: vitui_components::disclose::collapsible
//! [`Disclosure::used`]: vitui_components::disclose::Disclosure::used

use std::time::Duration;

use vitui_components::counters::Tally;
use vitui_components::disclose::{Collapse, DiscloseOpts, Focus, Height, collapsible_into};
use vitui_components::frame::{Face, face_paint};
use vitui_components::ink::{Direct, Ink};
use vitui_components::input::{ButtonOpts, button_into};
use vitui_components::structure::{PanelOpts, panel_with};
use vitui_components::text::{ChipOpts, FitOpts, chip_into, fit_into, fit_with};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Id, Rect, Role, Themes};

/// The twelve sections, which is the count the family was measured at.
const TITLES: [&str; 12] = [
    "General",
    "Appearance",
    "Editor",
    "Keyboard",
    "Terminal",
    "Extensions",
    "Version control",
    "Search",
    "Diagnostics",
    "Telemetry",
    "Experimental",
    "About",
];

/// The chips each body carries, per section. Different lengths, so a sizing function is answering a
/// question rather than returning a constant.
const OPTIONS: [&[&str]; 12] = [
    &[
        "start in the last folder",
        "restore tabs",
        "check for updates",
    ],
    &["theme", "density", "ligatures", "cursor shape", "italics"],
    &["tab width", "trim on save", "format on save", "wrap"],
    &["vim bindings", "leader key"],
    &[
        "shell",
        "scrollback",
        "bell",
        "cursor blink",
        "font",
        "opacity",
    ],
    &["auto-update", "sandbox"],
    &["auto-fetch", "sign commits", "prune on fetch"],
    &[
        "case sensitivity",
        "follow symlinks",
        "ignore files",
        "max results",
    ],
    &["inline hints", "underline style", "severity floor"],
    &["crash reports", "usage metrics"],
    &["gpu compositing", "predictive echo", "wide glyph repair"],
    &["version", "licence"],
];

/// How many chips fit on one row of a body.
const CHIPS_PER_ROW: usize = 3;
/// How many rows of buttons every body ends with.
const BUTTON_ROWS: u16 = 1;
/// How long an animated collapse takes: two hundred milliseconds.
const TWEEN: Duration = Duration::from_millis(200);

/// **The height a section's body wants, in rows, at width `w`.**
///
/// The sizing function, and it is the one decision both the component and the status line read — spec
/// *The height is an argument.* It is a free function rather than a closure so that the shape
/// `vitui_runtime::sizing::check` sweeps a component against is what this application actually
/// passes: `FnMut(u16) -> u16`.
fn body_rows(section: usize, w: u16) -> u16 {
    let per_row = usize::from(w / 20).clamp(1, CHIPS_PER_ROW);
    let chips = OPTIONS[section].len().div_ceil(per_row);
    u16::try_from(chips).unwrap_or(u16::MAX) + BUTTON_ROWS
}

/// Whether opening a section closes the others.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Any number open. The default.
    Many,
    /// One at a time — and the only arm on this screen where a section closes with no gesture on it.
    Exclusive,
}

struct App {
    /// **Twelve `Collapse`s and nothing else per section.** Four bytes of live state each; the tween
    /// slot is forty whether or not it holds one, which is the finding `disclose` records.
    sections: Vec<Collapse>,
    /// The first row of the content that is drawn, in rows.
    offset: i32,
    mode: Mode,
    /// Whether the height is animated.
    animate: bool,
    /// Where the open height comes from.
    height: Height,
    /// Whether the caller writes the tail below the last section.
    tail: bool,
    /// Whether the caller moves the focus before an exclusive collapse takes a body away.
    keep_focus: bool,
    counters: bool,
    exit: bool,
    /// **The header ids the last frame declared, indexed by section and not by draw order.**
    ///
    /// A `Vec` filled by `push` would be indexed by *what was drawn*, and the exclusive-mode focus
    /// below is keyed by section — so any culled section above the one that opened would move the
    /// keyboard to a different widget, silently defeating the one claim the `x`/`k` pair exists to
    /// demonstrate.
    heads: Vec<Option<Id>>,
    /// The rows the last frame's sections took, so the offset can be clamped against real content.
    content: i32,
    /// The viewport the last frame's panel handed over.
    view: Rect,
    /// What the last frame declared and cost.
    seen: Seen,
}

/// What the status line prints, all of it read from the frame that has just drawn.
#[derive(Clone, Copy, Default)]
struct Seen {
    /// **Whether a frame has been read at all.** The status line is drawn *inside* a frame from the
    /// previous frame's reading, so on the very first one there is none — and a default of `false`
    /// for `focused` would print `NOTHING FOCUSED` about a frame nobody had looked at. A figure
    /// defaulted to zero is a counter that prints `0` when it means *nobody counted*.
    read: bool,
    writes: u64,
    verbs: u64,
    regions: usize,
    stops: usize,
    /// **How many slots the vanish rule touched.** Zero on every frame nothing vanished on, which is
    /// why the `x`/`k` pair is the only thing on this screen that moves it.
    probes: u64,
    /// Whether anything holds the focus at all. `Ctx::focused`, and the finding
    /// is that this can be `false` on a screen that looks correct.
    focused: bool,
}

impl App {
    fn new() -> App {
        App {
            sections: (0..TITLES.len()).map(|_| Collapse::shut()).collect(),
            offset: 0,
            mode: Mode::Many,
            animate: false,
            height: Height::Sized,
            tail: true,
            keep_focus: true,
            counters: false,
            exit: false,
            heads: vec![None; TITLES.len()],
            content: 0,
            view: Rect::new(0, 0, 1, 1),
            seen: Seen::default(),
        }
    }

    fn opts(&self) -> DiscloseOpts {
        DiscloseOpts {
            height: self.height,
            focus: Focus::Header,
            dur: if self.animate { TWEEN } else { Duration::ZERO },
            ..DiscloseOpts::default()
        }
    }

    /// How many are open, for the status line.
    fn open(&self) -> usize {
        self.sections.iter().filter(|c| c.open()).count()
    }

    /// **The application's own keys, read from what nothing wanted.**
    ///
    /// `explorer`'s arrangement and its reason: a key an application owns cannot be read inside the
    /// draw, because a component that declines one **ends the level's turn at the queue**. `Enter`
    /// and `Space` are not here — they are the component's, and it reads them inside its own draw.
    fn take_unhandled(&mut self, keys: &[Pressed]) {
        let page = i32::from(self.view.h.max(1));
        let max = (self.content - i32::from(self.view.h)).max(0);
        for key in keys {
            match key.code {
                Code::Char('q') => self.exit = true,
                Code::Char('a') => self.animate = !self.animate,
                Code::Char('x') => {
                    self.mode = match self.mode {
                        Mode::Many => Mode::Exclusive,
                        Mode::Exclusive => Mode::Many,
                    };
                }
                Code::Char('k') => self.keep_focus = !self.keep_focus,
                Code::Char('w') => {
                    self.height = match self.height {
                        Height::Sized => Height::Watermark,
                        Height::Watermark => Height::Sized,
                    };
                }
                Code::Char('t') => self.tail = !self.tail,
                Code::Char('v') => self.counters = !self.counters,
                Code::Up => self.offset = (self.offset - 1).clamp(0, max),
                Code::Down => self.offset = (self.offset + 1).clamp(0, max),
                Code::PageUp => self.offset = (self.offset - page).clamp(0, max),
                Code::PageDown => self.offset = (self.offset + page).clamp(0, max),
                Code::Home => self.offset = 0,
                Code::End => self.offset = max,
                _ => {}
            }
        }
    }

    fn ui(&mut self, cx: &mut Ctx<'_, '_>) {
        let screen = cx.area();
        let (top, status) = split_last_row(screen);
        let panel = panel_with(cx, top, " settings ", "", &PanelOpts::default());
        self.view = panel.interior;

        let mut tally = Tally::new();
        let mut direct = Direct;
        // **One call site and one type**, so the counters toggle does not replace the widget:
        // `&mut dyn Ink` satisfies `I: Ink` through the crate's blanket impl. `explorer` found that
        // a toggle which swaps *which function* draws a widget swaps the widget, because `Ctx::id`
        // mints from `Location::caller()`.
        let mut sink: &mut dyn Ink = match self.counters {
            true => &mut tally,
            false => &mut direct,
        };
        // **A child clipped to the interior, and it is not decoration.** The offset can put a
        // section's header *above* the panel's first row, and a header drawn at a negative row of the
        // root context lands on the border: the clip stack is the call stack, so the only thing that
        // stops it is a context narrowed to the rectangle the panel handed over. Coordinates inside
        // are the interior's own, which is why the loop below starts at `-offset`.
        let interior = panel.interior;
        {
            let mut inner = cx.child(interior);
            self.sections_into(&mut sink, &mut inner, interior);
        }
        if self.counters {
            self.seen.writes = tally.writes();
            self.seen.verbs = tally.verbs();
        }
        self.status(cx, status);
    }

    /// The twelve sections, stacked, culled, and the tail below them.
    fn sections_into(&mut self, ink: &mut &mut dyn Ink, cx: &mut Ctx<'_, '_>, area: Rect) {
        let opts = self.opts();
        let body_paint = cx.theme().paint(Role::Body);
        let bottom = i32::from(area.h);
        self.heads.fill(None);

        // **Whose turn it is to be the only one open**, decided after the draw and applied before the
        // next one — a component reports the gesture and the *caller* decides what exclusivity means.
        let mut opened = None;

        let mut y = -self.offset;
        let mut content = 0i32;
        // The index is what keys the section and what the sizing function is asked about, so the
        // loop is over it rather than over `TITLES` — and `#[expect]` rather than `#[allow]`, so a
        // later rewrite that stops needing it fails here.
        #[expect(
            clippy::needless_range_loop,
            reason = "the index keys the widget (`Ctx::with_key`), indexes `self.sections` and is \
                      what the sizing function is asked about; iterating `TITLES` would need three \
                      more `[i]`s rather than fewer"
        )]
        for i in 0..TITLES.len() {
            let want = 1 + i32::from(self.sections[i].height());
            // **The caller's culling, and both bounds are the *child's*.** A section entirely outside
            // the panel is not drawn and therefore declares nothing at all, which is *1.05x six
            // open rather than 2x* — and the top edge is `0` rather than `area.y`, because this
            // context is `cx.child(interior)` and its coordinates start at its own corner. Compared
            // against the interior's root-space `y` instead, the first three sections of a
            // 3-row-inset panel are culled for standing above a row they are standing on.
            if y + want <= 0 || y >= bottom {
                y += want;
                content += want;
                continue;
            }
            let room = u16::try_from((bottom - y).max(0)).unwrap_or(u16::MAX);
            let rect = Rect::new(0, y, area.w, room.min(u16::try_from(want).unwrap_or(1)));
            let (id, gesture) = cx.with_key(i as u64, |cx| {
                let d = collapsible_into(
                    ink,
                    cx,
                    rect,
                    &mut self.sections[i],
                    TITLES[i],
                    &opts,
                    |w| body_rows(i, w),
                    |ink, cx| body_into(ink, cx, i),
                );
                (d.response.id, d.gesture)
            });
            self.heads[i] = Some(id);
            if let Some(toggle) = gesture
                && toggle.open
            {
                opened = Some(i);
            }
            // **The height the component settled on**, read after the call rather than before it: a
            // section that opened on this frame is taller than the `want` computed above, and an
            // offset clamped against the stale total is a page that stops one screen short for a
            // frame.
            let settled = 1 + i32::from(self.sections[i].height());
            y += settled;
            content += settled;
        }
        self.content = content;

        // **The tail, and it is the caller's** — every row of the panel below the last section is
        // inside the panel's rectangle and no section owns it. `t` is what happens when
        // nobody writes it.
        if self.tail {
            for row in y.max(0)..bottom {
                let _ = ink.run(cx, 0, row, " ", area.w, body_paint);
            }
        }

        // **Nothing holds the focus until an application says so** —
        // and the reason `Ctx::focused` exists: a `Ctx` could ask `is_focused(id)` and had no way to
        // ask whether *anything* held the focus. Without this the screen draws correctly, `Tab`
        // works, and `Enter` does nothing at all until the user has pressed `Tab` once. It is
        // `is_none` and not `!is_focused(first)`, because the second spelling drags the keyboard back
        // to the first header the frame after the user tabs away.
        if cx.focused().is_none()
            && let Some(first) = self.heads.iter().flatten().next().copied()
        {
            cx.focus(first);
        }

        // **Exclusive mode, applied after the draw**, which is the only place a caller may apply it:
        // the component has already collapsed what it was asked to and the rest is the caller's
        // policy about its own content.
        if self.mode == Mode::Exclusive
            && let Some(open) = opened
        {
            let now = cx.now();
            for (i, section) in self.sections.iter_mut().enumerate() {
                if i != open && section.open() {
                    section.set(now, false, 0, Duration::ZERO);
                }
            }
            // **The other sentence**: `Stash` belongs to whoever owns the content's identity, and
            // the caller does it by capturing the focus. Without this the focused widget inside the
            // section that just went vanishes and R08 answers with the nearest surviving entry — one
            // section too far, while the section the user acted on is still on screen.
            if self.keep_focus
                && let Some(id) = self.heads[open]
            {
                cx.focus(id);
            }
        }
    }

    fn status(&mut self, cx: &mut Ctx<'_, '_>, row: Rect) {
        // **The frame's own numbers come first**, because they are the ones a claim rests on and
        // `fit` truncates from the right: at eighty columns a status line that led with its toggles
        // would drop `probes` and print an ellipsis where the evidence was.
        let mut line = format!(
            " r{} s{} p{}  {}/{} open  off {}/{}",
            self.seen.regions,
            self.seen.stops,
            self.seen.probes,
            self.open(),
            TITLES.len(),
            self.offset,
            (self.content - i32::from(self.view.h)).max(0),
        );
        if self.counters {
            line.push_str(&format!(" w{} v{}", self.seen.writes, self.seen.verbs));
        }
        if self.seen.read && !self.seen.focused {
            line.push_str("  NOTHING FOCUSED");
        }
        line.push_str(&format!(
            "  {}{}{}{}{}",
            match self.mode {
                Mode::Many => "",
                Mode::Exclusive => "x ",
            },
            if self.animate { "a " } else { "" },
            match self.height {
                Height::Sized => "",
                Height::Watermark => "w ",
            },
            if self.tail { "" } else { "t " },
            if self.keep_focus { "" } else { "k " },
        ));
        line.push_str(" · a x k w t v q");
        let _ = fit_with(
            cx,
            row,
            &line,
            &FitOpts {
                role: Role::Dim,
                pad: Role::Dim,
                ..FitOpts::default()
            },
        );
    }
}

/// **One body: rows of chips and a row of buttons, and it paints every row of what it is handed.**
///
/// The padding is why `w` latches: a body whose extent reaches the bottom of any rectangle it is
/// given cannot be measured inside the rectangle the last measurement produced. That is not a defect
/// in the body — a padding ring is what makes a section look like a section — which is exactly why
/// the precondition is on the *body* and the spelling is refused unconditionally.
fn body_into<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, section: usize) {
    let area = cx.area();
    if area.is_empty() {
        return;
    }
    let paint = cx.theme().paint(Role::Body);
    for row in 0..i32::from(area.h) {
        let _ = ink.run(cx, area.x, area.y + row, " ", area.w, paint);
    }

    let per_row = usize::from(area.w / 20).clamp(1, CHIPS_PER_ROW);
    let cw = area.w / u16::try_from(per_row).unwrap_or(1);
    let chip_opts = ChipOpts::default();
    for (n, option) in OPTIONS[section].iter().enumerate() {
        let (row, col) = (n / per_row, n % per_row);
        let y = area.y + i32::try_from(row).unwrap_or(0);
        if y >= area.y + i32::from(area.h) {
            break;
        }
        let cells = Rect::new(
            area.x + 2 + i32::from(cw) * i32::try_from(col).unwrap_or(0),
            y,
            cw.saturating_sub(2),
            1,
        );
        cx.with_key(n as u64, |cx| {
            let _ = chip_into(ink, cx, cells, option, &chip_opts);
        });
    }

    let buttons = i32::try_from(OPTIONS[section].len().div_ceil(per_row)).unwrap_or(0);
    let at = area.y + buttons;
    if at < area.y + i32::from(area.h) {
        let cells = Rect::new(area.x + 2, at, area.w.saturating_sub(4).min(20), 1);
        cx.with_key(0x100, |cx| {
            let _ = button_into(ink, cx, cells, "restore defaults", &ButtonOpts::default());
        });
        let rest = Rect::new(
            cells.x + i32::from(cells.w) + 2,
            at,
            area.w.saturating_sub(cells.w + 6),
            1,
        );
        let _ = fit_into(
            ink,
            cx,
            rest,
            "the sizing function answers this height",
            &FitOpts {
                role: Role::Dim,
                pad: Role::Body,
                ..FitOpts::default()
            },
        );
    }

    // A face resolved before a cell is written, so the section reads as active while it is open.
    let _ = face_paint(cx.theme(), Face::REST);
}

/// The screen, less its last row. The status line is the application's.
fn split_last_row(screen: Rect) -> (Rect, Rect) {
    let h = screen.h.saturating_sub(1);
    (
        Rect::new(screen.x, screen.y, screen.w, h),
        Rect::new(screen.x, screen.y + i32::from(h), screen.w, screen.h - h),
    )
}

fn main() {
    let mut app = App::new();

    // **The picture arm**: one screen, drawn headlessly and written as SVG. A
    // program nobody outside this machine can see is a program nobody believes in.
    if vitui_apps::pictures::wanted() {
        let mut driver = vitui_apps::pictures::driver(100, 30, *Themes::standard().theme());
        for _ in 0..vitui_apps::pictures::FRAMES {
            driver.frame(|cx| app.ui(cx));
        }
        vitui_apps::pictures::write("settings", &driver);
        return;
    }

    let mut driver = match Driver::attach(Default::default(), *Themes::standard().theme()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };

    loop {
        driver.frame(|cx| app.ui(cx));
        app.read_frame(&driver);
        // **What nothing wanted, read from the frame that has just drawn.**
        //
        // `Driver::unhandled` is *a window onto the same queue, valid until the next frame begins*,
        // so an application that reads it **before** its frame reads the previous frame's window and
        // acts one wake late — which for a single keystroke means never, because nothing will wake it
        // again. Measured on the shipped binary: `q` did not quit.
        let unhandled: Vec<Pressed> = driver.unhandled().to_vec();
        app.take_unhandled(&unhandled);
        if app.exit {
            break;
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

impl App {
    /// **What the frame that has just drawn declared**, read after it rather than during it.
    ///
    /// `regions`, `stops`, the vanish probes and whether anything holds the focus at all are frame
    /// facts, and a status line that computed them itself would be printing its own opinion.
    fn read_frame(&mut self, driver: &Driver) {
        let frame = driver.inspect();
        self.seen.read = true;
        self.seen.regions = frame.hits().len();
        self.seen.stops = frame.stop_count();
        self.seen.probes = frame.vanish_probes();
        self.seen.focused = frame.focused().is_some();
    }
}
