//! A spreadsheet viewer whose offset belongs to the application, not to a component.
//!
//! Components ticket 45's application, and the seventeenth in this crate. It exists because O7 —
//! *every component this crate declares is exercised by an application* — was owed three rows when
//! it was built, and this screen is what closed them: [`scrollbar`], [`sticky`] and
//! [`file_picker`](vitui_components::files::file_picker).
//!
//! ```text
//! ┌─ sheet — the caller owns the offset ──────────────────────────────────────┐
//! │  ▶ ledger-2026.csv                                                       │  file_picker, shut
//! │     #  region      quarter     units       revenue     cost           ▲  │  gutter │ header │ bar
//! │      1         0         911        1822        2733        3644      █  │  pinned │ body   │
//! │      2        37         948        1859        2770        3681      ░  │
//! │      3        74         985        1896        2807        3718      ░  │
//! │            24000       25000       26000       27000       28000      ▼  │  corner │ footer │
//! │  ◀████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░▶  │  the horizontal bar
//! │  20000 x 24  offset (0, 0)  max (205, 19989)  tail on  pinned on  …       │
//! └──────────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! Three [`sticky`] bands and a corner, two [`scrollbar`]s, a
//! [`file_picker`](vitui_components::files::file_picker) along the top — and **the offset is a field
//! of `App`**. At 120 x 30 that is 3 600 writes over 3 600 distinct: every cell of the terminal,
//! written once.
//!
//! # Why it is not a `scroll_area`
//!
//! [`scroll_area`](vitui_components::scroll::scroll_area) is the component that **owns** the offset:
//! it reserves the bars, computes the four bands, applies the reveal, clamps, draws the tail and
//! consumes the wheel. Spec §9 states [`sticky`] and [`scrollbar`] as the pieces underneath it —
//! *one band construction with an axis argument*, and *a bar over one `Span`, which does not own an
//! offset* — and **nothing in this workspace had ever called either of them from outside
//! `scroll_area`.** Both were `built: true` with no consumer anywhere but the component that homes
//! them, which is precisely the shape O7 exists to find.
//!
//! So this screen assembles them by hand, with the offset in `App`. What that costs the caller is
//! the point rather than a complaint, and it is four things `scroll_area` does for nothing:
//!
//! | what | where it is here | what happens without it |
//! |---|---|---|
//! | the clamp | `App::clamp` | the body draws nothing and the bars point off the end |
//! | the tail | `App::tail` | §2's partition rule breaks: cells past the extent keep whatever was there. Press `t` |
//! | the wheel | `Ctx::scrollable` + `Response::scrolled` | the wheel chains outward to the panel and the grid never moves |
//! | the reveal | not written at all | see *what this application cannot say* |
//!
//! # What it demonstrates
//!
//! | key | what it shows |
//! |---|---|
//! | `↑` `↓` `←` `→` | move the offset. The header moves sideways with the body and not down; the pinned column moves down and not sideways |
//! | `PgUp` `PgDn` | a page of rows |
//! | `Home` `End` | the first and the last row |
//! | `Tab` | move the focus between the grid and the picker. **The grid is a tab stop that reads no key** |
//! | `Enter` `Space` | open the focused picker |
//! | `Esc` | shut it |
//! | `t` | **stop writing the tail.** Open `tiny.csv`, scroll, and watch the cells nobody writes |
//! | `p` | pin no column. The three bands become two and the gutter becomes a corner of nothing |
//! | `v` | the live counters, through a `Tally` |
//! | `q` `Ctrl+Q` `Ctrl+C` | quit. `q` is bindable here because nothing on this screen reads text: the grid takes no key at all and the picker's shut face takes four |
//!
//! **`t` is the one to watch, and it needs `tiny.csv`.** A sheet larger than the viewport has no
//! tail: every cell of the body is a cell of the document. Open the six-row file, and the rows past
//! the end of it belong to nobody — `scroll_area` writes them and this screen has to, because
//! **spec §2 assigns the remainder to whoever owns the rectangle** and here that is the
//! application.
//!
//! **`Tab` is the second.** The grid declares `Interest::FOCUS` and reads no key of its own, so
//! every keystroke falls through to [`Driver::unhandled`] — which is components ticket 38's control
//! arm as a screen: *a tab stop that reads no key*. Move the focus to the picker and the arrows stop
//! being the grid's, because a shut picker binds `Down` to *open the list*.
//!
//! # What this application cannot say
//!
//! **The picker's popup has no keyboard at all**, so a file can be opened with `Enter` and chosen
//! only with the mouse. That is components architecture issue 23, found by ticket 38's sweep and
//! reproduced here by a person pressing keys: the list draws a plain `collection_into`, seats no
//! focus and declares no refusal, so an open picker has no arrows, no `Home`/`End`, no type-ahead
//! and no way to choose a file — on a screen that renders perfectly.
//!
//! **There is no reveal.** `Ctx::request_into_view` addresses the widget that owns the offset, and
//! here that is an application rather than a widget: nothing declared a scroll scope's id to ask
//! about, so `End` is arithmetic on `App::offset` and not a request. What that means in practice is
//! that a component drawn *inside* this grid could not ask to be scrolled to, which is the one thing
//! a hand-assembled area cannot buy back.
//!
//! **The bars are the caller's rectangles.** `scroll_area`'s hysteresis — §9's fixpoint, where an
//! auto-hiding bar loses a row and a column permanently — cannot happen here, because nothing on
//! this screen decides whether a bar stands. That is not the caller doing better; it is the caller
//! having declared the question away.
//!
//! ```text
//! cargo run -p vitui-apps --example sheet
//! cargo run -p vitui-apps --example sheet -- --probe
//! ```
//!
//! [`sticky`]: vitui_components::scroll::sticky
//! [`scrollbar`]: vitui_components::scroll::scrollbar
//! [`Driver::unhandled`]: vitui_runtime::ctx::Driver::unhandled

use std::fmt::Write as _;

use vitui_components::counters::Tally;
use vitui_components::files::{
    Entry, PickerBody, PickerOpts, PickerState, Preview, file_picker_into,
};
use vitui_components::ink::{Direct, Ink};
use vitui_components::scroll::{
    BarOpts, Orient, ScrollbarOpts, Shares, Span, scrollbar_into, sticky,
};
use vitui_components::structure::{PanelOpts, panel_into};
use vitui_components::text::{FitOpts, fit_into};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::layout::rect;
use vitui_runtime::work::{Cancel, Task, Wake, Worker};
use vitui_runtime::{Ctx, Id, Interest, Rect, Role, Scrollable};

// ── the documents ────────────────────────────────────────────────────────────────────────────────

/// How wide one column of the grid is, the separating space included.
const COL_W: u16 = 12;

/// How wide the gutter and the pinned column are.
const GUTTER: u16 = 6;

/// The files the picker offers. **`tiny.csv` is not decoration**: a sheet larger than the viewport
/// has no tail, so the one rule the caller has to buy back is invisible without a short document.
const FILES: [(u64, &str); 4] = [
    (1, "ledger-2026.csv"),
    (2, "inventory.tsv"),
    (3, "payroll.csv"),
    (4, "tiny.csv"),
];

/// **A decoded sheet, and it holds no cells.** Every value is a formula over `(row, column)`, so the
/// frame costs its visible window and never the document — which is the engine's own invariant read
/// one crate up, and the reason this screen can offer twenty thousand rows without an allocation.
#[derive(Clone, Copy)]
struct Sheet {
    /// The file's identity, which is what [`Preview::shows`] is.
    id: u64,
    rows: u32,
    cols: u16,
}

impl Sheet {
    /// The document `id` names.
    fn of(id: u64) -> Sheet {
        match id {
            2 => Sheet {
                id,
                rows: 4_096,
                cols: 9,
            },
            3 => Sheet {
                id,
                rows: 812,
                cols: 16,
            },
            4 => Sheet {
                id,
                rows: 6,
                cols: 3,
            },
            _ => Sheet {
                id,
                rows: 20_000,
                cols: 24,
            },
        }
    }

    /// The column's heading.
    fn heading(self, col: u16) -> &'static str {
        const HEADINGS: [&str; 8] = [
            "region", "quarter", "units", "revenue", "cost", "margin", "owner", "note",
        ];
        HEADINGS[usize::from(col) % HEADINGS.len()]
    }

    /// One cell, written into `out` rather than returned, because a `String` a frame is an
    /// allocation a frame — `crate::media::player`'s finding, met by `pagination` and by a fixture
    /// since.
    fn cell(self, out: &mut String, row: u32, col: u16) {
        out.clear();
        let v = u64::from(row)
            .wrapping_mul(37)
            .wrapping_add(u64::from(col) * 911)
            % 100_000;
        let _ = write!(out, "{v:>9}");
    }

    /// The column's total, which is what the footer band draws.
    fn total(self, out: &mut String, col: u16) {
        out.clear();
        let _ = write!(out, "{:>9}", u64::from(col) * 1_000 + 24_000);
    }

    /// The content extent, in cells on each axis. **`Σ h`, never a row count** — here the rows are
    /// one cell tall, so the two happen to agree and the arithmetic still says which it means.
    fn extent(self) -> (i32, i32) {
        (
            i32::from(self.cols) * i32::from(COL_W),
            i32::try_from(self.rows).unwrap_or(i32::MAX),
        )
    }
}

impl Preview for Sheet {
    fn shows(&self) -> u64 {
        self.id
    }

    fn extent(&self) -> (u32, u32) {
        let (w, h) = Sheet::extent(*self);
        (u32::try_from(w).unwrap_or(0), u32::try_from(h).unwrap_or(0))
    }
}

/// **The picker's decode, and it is a free function over an identity.** A job must be
/// `Send + 'static` and a body is `FnMut`, so the thing that makes one has to be callable more than
/// once and may capture nothing that dies with the frame — `crate::files`'s own sentence.
fn decode(id: u64, _cancel: &Cancel) -> Sheet {
    Sheet::of(id)
}

/// One row of the picker's preview pane.
fn preview_line(cx: &mut Ctx<'_, '_>, row: Rect, sheet: &Sheet, i: u32) {
    let paint = cx.theme().paint(Role::Dim);
    let mut buf = String::with_capacity(32);
    sheet.cell(&mut buf, i, 0);
    let _ = cx.text(row.x, row.y, &buf, paint);
}

// ── the application ──────────────────────────────────────────────────────────────────────────────

/// The grid's own id, so that the tab stop and the scrollable region are one widget rather than two.
const GRID: Id = Id::named("sheet.grid");

/// What the last frame reported, for the status line.
#[derive(Clone, Copy, Default)]
struct Counters {
    writes: u64,
    distinct: u64,
    verbs: u64,
    regions: usize,
    stops: usize,
}

/// The screen.
struct App {
    files: Vec<Entry<'static>>,
    picker: PickerState,
    body: PickerBody<Sheet>,
    task: Task<Sheet>,
    worker: Worker,
    sheet: Sheet,
    /// **The offset, and it is the whole point of the screen.** In content cells on both axes.
    offset: (i32, i32),
    /// Whether the caller writes the cells past the extent. Spec §2's partition rule, under `t`.
    tail: bool,
    /// Whether a column is pinned at all.
    pinned: bool,
    counting: bool,
    counters: Counters,
    status: String,
    exit: bool,
}

impl App {
    fn new(worker: Worker) -> App {
        let task = Task::new(&worker);
        let mut picker = PickerState::new();
        picker.choose(FILES[0].0);
        App {
            files: FILES
                .iter()
                .map(|(id, name)| Entry { id: *id, name })
                .collect(),
            picker,
            body: PickerBody::new(),
            task,
            worker,
            sheet: Sheet::of(FILES[0].0),
            offset: (0, 0),
            tail: true,
            pinned: true,
            counting: false,
            counters: Counters::default(),
            status: String::with_capacity(128),
            exit: false,
        }
    }

    /// **The clamp, which `scroll_area` does for nothing.** Recomputed every frame, because the
    /// extent moves when the file does and an offset past a shrunk extent draws nothing at all.
    fn max(&self, view: Rect) -> (i32, i32) {
        let (ex, ey) = self.sheet.extent();
        (
            (ex - i32::from(view.w)).max(0),
            (ey - i32::from(view.h)).max(0),
        )
    }

    fn clamp(&mut self, view: Rect) {
        let max = self.max(view);
        self.offset = (self.offset.0.clamp(0, max.0), self.offset.1.clamp(0, max.1));
    }

    /// One frame, top to bottom.
    fn ui<'f, I: Ink>(&'f mut self, ink: &mut I, cx: &mut Ctx<'f, '_>) {
        let whole = cx.area();
        let panel = panel_into(
            ink,
            cx,
            whole,
            " sheet — the caller owns the offset ",
            &PanelOpts::default(),
        );
        let interior = panel.interior;
        if interior.h < 8 || interior.w < GUTTER + COL_W + 4 {
            return;
        }

        let (head, rest) = rect::split_at_v(interior, 1);
        let (grid, foot) = rect::split_at_v(rest, rest.h.saturating_sub(2));
        let (hbar, status) = rect::split_at_v(foot, 1);

        // **The two bars are rectangles the caller reserved**, which is the half of §9 a component
        // usually decides. Nothing on this screen asks whether they should stand.
        let (grid, vbar) = rect::split_at_h(grid, grid.w.saturating_sub(1));
        let pinned_w = if self.pinned { GUTTER } else { 0 };
        let (left, right) = rect::split_at_h(grid, pinned_w);
        let (corner, below_left) = rect::split_at_v(left, 1);
        let (pinned, left_foot) = rect::split_at_v(below_left, below_left.h.saturating_sub(1));
        let (header, below) = rect::split_at_v(right, 1);
        let (view, footer) = rect::split_at_v(below, below.h.saturating_sub(1));

        self.clamp(view);
        let max = self.max(view);
        let (ex, ey) = self.sheet.extent();

        // **The wheel, declared by the thing that owns the offset.** A region that publishes an
        // axis instead of a direction eats every click at its own end; `Scrollable::between` is the
        // arithmetic that says which way it can still move.
        let resp = cx.scrollable(
            GRID,
            view,
            Interest::FOCUS.with(Interest::CLICK).with(Interest::HOVER),
            Scrollable::between(self.offset, max),
        );
        // **This frame's wheel, read inside the draw.** The one pointer outcome that is not awarded
        // at `end`, so a delta read next frame is a delta read a frame late.
        self.offset = (
            (self.offset.0 + resp.scrolled.0).clamp(0, max.0),
            (self.offset.1 + resp.scrolled.1).clamp(0, max.1),
        );
        if resp.clicked {
            cx.focus(GRID);
        }

        let theme = cx.theme();
        let body_paint = theme.paint(Role::Body);
        let dim = theme.paint(Role::Dim);
        let title = theme.paint(Role::Title);
        let border = theme.paint(Role::Border);

        let sheet = self.sheet;
        let offset = self.offset;
        let fill = self.tail;
        let mut line = String::with_capacity(512);
        let mut buf = String::with_capacity(32);

        // **The gutter: `Shares::Neither`.** Where a horizontal band and the pinned column meet, and
        // the one of the three arms that moves on neither axis.
        if !corner.is_empty() {
            sticky(cx, corner, Shares::Neither, offset, |cx| {
                let _ = ink.text(cx, 0, 0, "   #  ", title);
            });
        }

        // **The header: `Shares::X`.** It draws at *content* columns, which is what makes a title
        // and the cells under it unable to disagree about where a column is — they are the same
        // number.
        sticky(cx, header, Shares::X, offset, |cx| {
            let span = grid_row(
                &mut line,
                &mut buf,
                header.w,
                offset.0,
                sheet.cols,
                true,
                |b, c| {
                    b.clear();
                    let _ = write!(b, "{:<11} ", sheet.heading(c));
                },
            );
            let _ = ink.text(cx, offset.0, 0, &line[span], title);
        });

        // **The pinned column: `Shares::Y`.** Content rows, column zero — and the rows past the
        // document are the same tail the body has, on the other axis.
        if !pinned.is_empty() {
            sticky(cx, pinned, Shares::Y, offset, |cx| {
                for i in 0..i32::from(pinned.h) {
                    let row = offset.1 + i;
                    line.clear();
                    if row < ey {
                        let _ = write!(line, "{:>5} ", row + 1);
                    } else if fill {
                        line.push_str("      ");
                    } else {
                        continue;
                    }
                    let _ = ink.text(cx, 0, row, &line, dim);
                }
            });
        }
        if !left_foot.is_empty() {
            let _ = ink.text(cx, left_foot.x, left_foot.y, "      ", border);
        }

        // **The footer: `Shares::X` again**, which is what makes §9's four bands three arms.
        sticky(cx, footer, Shares::X, offset, |cx| {
            let span = grid_row(
                &mut line,
                &mut buf,
                footer.w,
                offset.0,
                sheet.cols,
                true,
                |b, c| {
                    sheet.total(b, c);
                },
            );
            let _ = ink.text(cx, offset.0, 0, &line[span], border);
        });

        // **The body, at content coordinates**, under the grid's own id so that the scroll scope and
        // the hit entry are one widget. `Ctx::with_id` is outside the scope and not around it
        // because the id names the whole area — which was also the only thing that worked until
        // runtime architecture issue 31: inside, `with_id` re-childed the view at the *content's*
        // rectangle.
        cx.with_id(GRID, |cx| {
            cx.scroll_scope(GRID, view, offset, max, |cx| {
                for i in 0..i32::from(view.h) {
                    let row = offset.1 + i;
                    // **The tail, and §2 assigns it to whoever owns the rectangle.** Here that is an
                    // application: a row past the end of the document belongs to nobody, and `t`
                    // is what stops this screen writing it.
                    if row >= ey && !fill {
                        continue;
                    }
                    let span = grid_row(
                        &mut line,
                        &mut buf,
                        view.w,
                        offset.0,
                        if row < ey { sheet.cols } else { 0 },
                        fill,
                        |b, c| sheet.cell(b, u32::try_from(row).unwrap_or(0), c),
                    );
                    if span.is_empty() {
                        continue;
                    }
                    let _ = ink.text(cx, offset.0, row, &line[span], body_paint);
                }
            });
        });

        // **The two bars, over one `Span` each.** A `scrollbar` does not own an offset: the span
        // arrives from whoever does, which on this screen is three fields up. The id is a parameter
        // because two bars cannot both take `Location::caller()` from one line.
        let opts = ScrollbarOpts::default();
        if !vbar.is_empty() {
            let _ = scrollbar_into(
                ink,
                cx,
                Id::keyed(GRID, 1),
                vbar,
                Span {
                    viewport: u32::from(view.h),
                    extent: u32::try_from(ey).unwrap_or(0),
                    offset: u32::try_from(offset.1).unwrap_or(0),
                },
                &opts,
            );
        }
        let across = ScrollbarOpts {
            bar: BarOpts {
                orient: Orient::Horizontal,
                ..opts.bar
            },
            ..opts
        };
        let _ = scrollbar_into(
            ink,
            cx,
            Id::keyed(GRID, 2),
            hbar,
            Span {
                viewport: u32::from(view.w),
                extent: u32::try_from(ex).unwrap_or(0),
                offset: u32::try_from(offset.0).unwrap_or(0),
            },
            &across,
        );

        // **The picker, last, because its overlay is placed on top of everything above it.**
        let App {
            picker,
            body,
            files,
            task,
            ..
        } = self;
        let picker_resp = file_picker_into(
            ink,
            cx,
            Id::keyed(GRID, 3),
            head,
            picker,
            body,
            files,
            task,
            decode,
            preview_line,
            &PickerOpts::default(),
        );

        // **Nothing holds the focus until an application says so** (architecture issue 25), and the
        // grid is what it is seated on: a tab stop that reads no key, so every keystroke reaches
        // `Driver::unhandled`.
        if cx.focused().is_none() {
            cx.focus(GRID);
        }

        self.status.clear();
        let _ = write!(
            self.status,
            "{} x {}  offset ({}, {})  max ({}, {})  tail {}  pinned {}  picker {}",
            self.sheet.rows,
            self.sheet.cols,
            self.offset.0,
            self.offset.1,
            max.0,
            max.1,
            if self.tail { "on" } else { "OFF" },
            if self.pinned { "on" } else { "off" },
            if picker_resp.rect.h == 1 {
                "shut"
            } else {
                "open"
            },
        );
        if self.counting {
            let _ = write!(
                self.status,
                "  writes {} distinct {} verbs {} regions {} stops {}",
                self.counters.writes,
                self.counters.distinct,
                self.counters.verbs,
                self.counters.regions,
                self.counters.stops,
            );
        }
        let _ = write!(
            self.status,
            "   t tail  p pin  v counters  Tab focus  q quit"
        );
        let _ = fit_into(ink, cx, status, &self.status, &FitOpts::default());
    }

    /// Read what the frame that has just drawn reported.
    fn read_frame(&mut self, driver: &Driver, tally: Option<&Tally>) {
        self.counters.regions = driver.inspect().hits().len();
        self.counters.stops = driver.inspect().ring().len();
        if let Some(t) = tally {
            self.counters.writes = t.writes();
            self.counters.distinct = t.distinct();
            self.counters.verbs = t.verbs();
        }
    }

    /// **What nothing wanted, read from the frame that has just drawn.** `Driver::unhandled` is a
    /// window onto the same queue, valid until the next frame begins, so an application that reads
    /// it before its own frame acts one wake late — which for a single keystroke means never.
    fn take_unhandled(&mut self, keys: &[Pressed], page: i32) {
        for k in keys {
            match k.code {
                Code::Char('q') | Code::Char('Q') => self.exit = true,
                Code::Char('c') if k.mods.ctrl() => self.exit = true,
                Code::Char('t') => self.tail = !self.tail,
                Code::Char('p') => self.pinned = !self.pinned,
                Code::Char('v') => self.counting = !self.counting,
                Code::Up => self.offset.1 -= 1,
                Code::Down => self.offset.1 += 1,
                Code::Left => self.offset.0 -= i32::from(COL_W),
                Code::Right => self.offset.0 += i32::from(COL_W),
                Code::PageUp => self.offset.1 -= page,
                Code::PageDown => self.offset.1 += page,
                Code::Home => self.offset.1 = 0,
                Code::End => self.offset.1 = i32::MAX / 2,
                _ => {}
            }
        }
        // The chosen file is the picker's answer, read once. A sheet is a formula, so opening one
        // is arithmetic rather than a decode — the worker's copy is what the picker's own preview
        // pane shows.
        if let Some(chosen) = self.picker.chosen()
            && chosen != self.sheet.id
        {
            self.sheet = Sheet::of(chosen);
            self.offset = (0, 0);
        }
        self.offset = (self.offset.0.max(0), self.offset.1.max(0));
    }
}

/// **Build one full-width row of the grid at content columns, and return the slice to write.**
///
/// One verb a row and never one a cell, which is `crate::text::pad_to`'s finding one crate over: a
/// row is a partition of its width, so writing the cells and then the gaps between them makes
/// `verbs` a function of how full the row happens to be rather than of what it drew.
///
/// `fill` is the tail. With it off the line stops at the document's last column and the cells past
/// it are written by nobody — which is what `t` is for.
fn grid_row(
    out: &mut String,
    buf: &mut String,
    w: u16,
    ox: i32,
    cols: u16,
    fill: bool,
    mut cell: impl FnMut(&mut String, u16),
) -> std::ops::Range<usize> {
    let col_w = usize::from(COL_W);
    out.clear();
    let first = u16::try_from(ox.max(0) / i32::from(COL_W)).unwrap_or(0);
    let skip = usize::try_from(ox.max(0) - i32::from(first) * i32::from(COL_W)).unwrap_or(0);
    let need = skip + usize::from(w);
    let mut c = first;
    while out.len() < need {
        let base = out.len();
        if c < cols {
            cell(buf, c);
            out.push_str(buf);
        } else if !fill {
            break;
        }
        // **Every column is exactly `COL_W` cells**, padded and truncated here rather than trusted
        // of each caller: a cell one character wide of its slot puts every column after it on a
        // different one, and the screen still looks like a spreadsheet.
        while out.len() < base + col_w {
            out.push(' ');
        }
        out.truncate(base + col_w);
        let Some(next) = c.checked_add(1) else { break };
        c = next;
    }
    let start = skip.min(out.len());
    let end = (start + usize::from(w)).min(out.len());
    start..end
}

/// One headless frame, and what it cost. `browse`'s door, for its reason: the subject of this screen
/// is a set of counts and a person reading a report should not have to hold a terminal open.
fn probe(app: &mut App) {
    let mut driver = match Driver::headless(120, 30) {
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

    println!("vitui sheet — one headless frame at 120 x 30\n");
    println!("  {}", app.status);
    println!(
        "\n  writes {}  distinct {}  verbs {}  regions {}  tab stops {}",
        app.counters.writes,
        app.counters.distinct,
        app.counters.verbs,
        app.counters.regions,
        app.counters.stops,
    );
    println!(
        "  spawns {}  — the picker is shut, so its pane has asked nothing",
        app.worker.asked()
    );
}

fn main() {
    if std::env::args().any(|a| a == "--probe") {
        let mut app = App::new(Worker::queueing());
        probe(&mut app);
        return;
    }

    let mut driver = match Driver::attach(Default::default(), Default::default()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };
    let mut app = App::new(Worker::hire(driver.wake()));

    loop {
        let mut tally = Tally::new();
        let mut direct = Direct;
        let counting = app.counting;
        // **One call site, whichever ink.** Two calls would be two `Location::caller()`s and the
        // widgets drawn under them would be two different widgets — `explorer` is the caller that
        // found it.
        let mut ink: &mut dyn Ink = if counting { &mut tally } else { &mut direct };
        driver.frame(|cx| app.ui(&mut ink, cx));
        app.read_frame(&driver, counting.then_some(&tally));

        let unhandled: Vec<Pressed> = driver.unhandled().to_vec();
        let page = i32::from(driver.size().1.saturating_sub(8)).max(1);
        app.take_unhandled(&unhandled, page);
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
