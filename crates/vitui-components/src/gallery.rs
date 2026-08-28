//! **O2's evidence as a value: one screen carrying every built row of the freeze, and the drift
//! gate is two equalities over the table it is drawn from.**
//!
//! > O2 — a panel in the gallery binary | **two** equalities: nothing shown may be absent from the
//! > freeze, and everything `built` must have a panel. (spec §17)
//!
//! Components ticket 39. The binary is `crates/vitui-apps/examples/gallery.rs`; what lives here is
//! the **screen**, and the reason it lives here rather than in the application is that spec §21
//! names two defects to be measured *on the assembled gallery* — the sentinel (register row 7) and
//! the palette swap (row 8) — and both are components tickets whose gate is `cargo test`. A screen
//! only an application can reach is a screen no gate can measure.
//!
//! # The gallery is a gate, not a demo, and the table is what makes that checkable
//!
//! [`PANELS`] is the panel list as a value: one row per built row of [`crate::INVENTORY`], in the
//! freeze's own order, each carrying the function that draws the shipped component. The application
//! **iterates it** and mints no panel of its own, which is what closes the loop O2 is about — an
//! application free to list its own panels is an application that can drift from the freeze in a
//! direction no equality over `PANELS` can see. `tests::the_application_draws_the_table_and_mints_
//! no_panel_of_its_own` is the negative scan, and it is a scan for what the file **must not**
//! contain, so it cannot be satisfied by deleting anything.
//!
//! [`crate::obligations::PANELS`] is the written-out list the two equalities run over, and
//! [`panel_ids`] derives the same list from this table. Two lists a test compares is
//! [`crate::doc`]'s arrangement for O1 and [`crate::contract`]'s for O4, and for their reason: an
//! equality between two things derived from each other holds for ever.
//!
//! # The population is `built`, for the third time on this map
//!
//! `spinner` is the twenty-ninth row and has no panel: *a panel for a function that does not exist
//! is not a screen anybody can draw*. O1 chose `built` (ADR 0043), O3 chose `built` (ADR 0044), and
//! O2's second equality is **already written over `built`** in the spec, so nothing had to be
//! decided here — the population moves on its own the day `spinner` ships.
//!
//! # `'f` costs the table one field, and it is components 26's finding rather than a new one
//!
//! Two of the twenty-eight own an overlay — [`crate::input::select`] and
//! [`crate::files::file_picker`] — and both take `&'f mut` of the state their popup body captures
//! (spec §1's sentence about the fifth component). A table of `fn(&mut Bag, …)` pointers cannot
//! hand that over: a fn pointer's elided lifetimes are fresh and unrelated, so a reborrow out of
//! the bag is shorter than `'f` whatever the caller does.
//!
//! So the two `'f`-bound borrows are split off the bag **once**, at the top of the view, and arrive
//! as [`Owners`] — a pair of `Option`s a panel **takes**. That is components 26's own answer to the
//! same borrow (*a nested overlay's state has to arrive as an `Option` the body takes*), and the
//! `take` is load-bearing twice: it is the only way to get a `'f` borrow out of a `&mut`, and a
//! second take panics — which is right, because a panel drawn twice in one frame is two widgets
//! under one id.
//!
//! # The nine-cell matrix is reachable from this crate, and register row 45 said it was not
//!
//! Row 45 is `Unreachable { needs: "`ColorDepth`" }`, and its own `inverted_by` names runtime
//! architecture issue 22 — **the issue that lifted the barrier**. `crates/vitui-runtime/src/line.rs`
//! now files `ColorDepth` as `reachable_as: Some("vitui_runtime::ColorDepth")`, so the colour axis
//! is a value this crate can hold and `Theme::resolve` is a call it can make. That is the near-miss
//! runtime 22 warned about arriving on this register: *a `Barrier` citation that still passes while
//! meaning the opposite*. The row inverts here, because this ticket's own criterion is that matrix.
//!
//! # What is deliberately **not** done here
//!
//! Register rows 7 and 8 stay red. This ticket delivers the screen they are measured on; components
//! 40 inverts the sentinel and components 41 the palette swap. [`shape`] and [`swap`] print both
//! numbers as
//! they are measured on this screen, so that the two tickets start from a figure rather than from a
//! prototype's — which is the discipline every other row of this crate's map has needed.

use std::fmt::Write as _;

use vitui_runtime::layout::{Constraint, rect};
use vitui_runtime::work::{Cancel, Task, Worker};
use vitui_runtime::{ColorDepth, Ctx, Density, GlyphSet, Rect, Role, Theme, Themes};

use crate::app::Clears;
use crate::chart::raster::{PlotState, RUNGS};
use crate::chart::{Opts as PlotOpts, Series, chart_into, plot_into};
use crate::collect::{
    CollOpts, CollState, Column, PageOpts, TableOpts, TableState, TreeOpts, TreeState,
    collection_into, pagination_into, table_into, tree_into,
};
use crate::disclose::{Collapse, DiscloseOpts, collapsible_into};
use crate::edit::Text;
use crate::files::{
    Entry, PaneOpts, PaneState, PickerBody, PickerOpts, PickerState, Preview, asking,
    file_picker_into, file_preview_pane_into,
};
use crate::frame::face_paint;
use crate::indicate::{MeterOpts, SparkOpts, meter_into, sparkline_into};
use crate::ink::Ink;
use crate::input::{
    ButtonOpts, FieldOpts, FormOpts, FormState, SelectOpts, SelectState, SliderOpts, Toggle,
    ToggleOpts, button_into, field_into, form_into, select_into, slider_into, toggle_into,
};
use crate::order::{Entry as Node, Order, Rows};
use crate::overlay::{PopupState, ShellOpts, overlay_into};
use crate::scroll::{
    AreaOpts, AreaState, ScrollbarOpts, Shares, Span, scroll_area_into, scrollbar_into,
    sticky as sticky_band,
};
use crate::structure::{PanelOpts, RuleOpts, StatusOpts, panel_into, rule_into, status_bar_into};
use crate::text::{ChipOpts, TextOpts, chip_into, text_into};

// ── the fixtures every panel draws from ──────────────────────────────────────────────────────────

/// Every panel's rows, so that two panels drawn from the same shape differ because the component
/// differs and not because the fixture does. [`crate::golden`]'s arrangement, for its reason.
pub const NAMES: [&str; 4] = ["alpha", "beta", "gamma", "delta"];

/// What the two overlay owners offer.
pub const OPTIONS: [&str; 3] = ["name", "date modified", "size"];

/// The files the picker lists. A `static` rather than a field, because
/// [`crate::files::file_picker_into`] takes `&'f [Entry<'f>]` and a `'static` slice satisfies every
/// `'f` there is.
pub static FILES: [Entry<'static>; 4] = [
    Entry {
        id: 0,
        name: "alpha",
    },
    Entry {
        id: 1,
        name: "beta",
    },
    Entry {
        id: 2,
        name: "gamma",
    },
    Entry {
        id: 3,
        name: "delta",
    },
];

/// The rows a preview draws, as literals.
///
/// **Literals and not `format!`**, and the gallery's own allocation gate is what says so: the two
/// preview drawers spelled their row labels with `format!` and page three of the screen paid **4
/// allocations a frame, 200 over 50**, on a screen that renders perfectly and whose every other page
/// read zero. It is components 30's `player::chrome` finding a second time — *a `Vec<f32>` on the
/// draw path is a field now, derived at construction* — with the derivation being that there are
/// only ever eight rows to name.
pub const ROWS: [&str; 8] = [
    "row 00", "row 01", "row 02", "row 03", "row 04", "row 05", "row 06", "row 07",
];

/// The one preview a panel needs: an identity and an extent, and nothing else.
#[derive(Clone, Copy, Debug)]
pub struct Doc(u64);

impl Preview for Doc {
    fn shows(&self) -> u64 {
        self.0
    }
    fn extent(&self) -> (u32, u32) {
        (8, 6)
    }
}

/// A decode is a free function over an identity — never a closure (spec §15).
fn decode(id: u64, _cancel: &Cancel) -> Doc {
    Doc(id)
}

/// One row of a preview. A `fn` and not a closure, because
/// [`crate::files::file_picker_into`] takes a function pointer for it — a job must be
/// `Send + 'static` and a body is `FnMut`.
fn preview_line(cx: &mut Ctx<'_, '_>, row: Rect, _doc: &Doc, i: u32) {
    let paint = cx.theme().paint(Role::Body);
    let _ = cx.text(row.x, row.y, ROWS[i as usize % ROWS.len()], paint);
}

// ── the panel table ──────────────────────────────────────────────────────────────────────────────

/// **What a panel draws through.** `&mut dyn Ink` satisfies `I: Ink` through
/// [`crate::ink`]'s blanket impl, so the gallery has one call site whichever ink it chose — which
/// matters here for the reason `explorer` found: two call sites are two `Location::caller()`s, so
/// toggling the counters would re-declare every widget under a new id.
pub type Sink<'a> = &'a mut dyn Ink;

/// **The frame-long borrows the two overlay owners need**, split off the bag once at the top of the
/// view and taken by the panel that owns each.
///
/// See this module's header: a fn pointer cannot carry a `'f`-tied `&mut` out of a bag, and
/// `Option::take` is the one operation that yields a `'f` borrow through a shorter one.
pub struct Owners<'f> {
    /// [`crate::input::select`]'s popup.
    pub select: Option<&'f mut PopupState>,
    /// [`crate::files::file_picker`]'s overlay body.
    pub picker: Option<&'f mut PickerBody<Doc>>,
    /// What the picker lists.
    pub files: &'f [Entry<'f>],
    /// Where the picker's answers land.
    pub task: &'f Task<Doc>,
}

/// What a panel's drawing function is.
///
/// **`'f` is tied to the `Ctx` deliberately**, and it is the one thing the signature had to state:
/// with elided lifetimes the bag's borrow and the frame's are unrelated and neither overlay owner
/// can be written at all.
pub type Draw = for<'f> fn(&mut Bag, &mut Owners<'f>, &mut Sink<'_>, &mut Ctx<'f, '_>, Rect);

/// One panel: the freeze row it shows, the title over it, and what it draws.
pub struct Panel {
    /// The [`crate::INVENTORY`] row this panel is evidence for. O2 joins on it.
    pub id: &'static str,
    /// What the panel's frame says. The id plus what a reader is meant to look at.
    pub title: &'static str,
    /// The shipped component, drawn in the interior the panel handed over.
    pub draw: Draw,
}

/// **The panels, one per built row of the freeze, in the freeze's own order.**
///
/// Twenty-eight. `spinner` is the twenty-ninth row and has none — see this module's header and
/// [`crate::obligations::o2_everything_built_has_a_panel`], whose population is `built`.
pub const PANELS: &[Panel] = &[
    Panel {
        id: "text",
        title: "text",
        draw: draws::text,
    },
    Panel {
        id: "panel",
        title: "panel",
        draw: draws::panel,
    },
    Panel {
        id: "chip",
        title: "chip",
        draw: draws::chip,
    },
    Panel {
        id: "button",
        title: "button",
        draw: draws::button,
    },
    Panel {
        id: "field",
        title: "field",
        draw: draws::field,
    },
    Panel {
        id: "collection",
        title: "collection",
        draw: draws::collection,
    },
    Panel {
        id: "table",
        title: "table",
        draw: draws::table,
    },
    Panel {
        id: "tree",
        title: "tree",
        draw: draws::tree,
    },
    Panel {
        id: "select",
        title: "select",
        draw: draws::select,
    },
    Panel {
        id: "overlay",
        title: "overlay",
        draw: draws::overlay,
    },
    Panel {
        id: "scroll_area",
        title: "scroll_area",
        draw: draws::scroll_area,
    },
    Panel {
        id: "scrollbar",
        title: "scrollbar",
        draw: draws::scrollbar,
    },
    Panel {
        id: "sticky",
        title: "sticky",
        draw: draws::sticky,
    },
    Panel {
        id: "collapsible",
        title: "collapsible",
        draw: draws::collapsible,
    },
    Panel {
        id: "chart",
        title: "chart",
        draw: draws::chart,
    },
    Panel {
        id: "plot",
        title: "plot",
        draw: draws::plot,
    },
    Panel {
        id: "checkbox",
        title: "checkbox",
        draw: draws::checkbox,
    },
    Panel {
        id: "radio",
        title: "radio",
        draw: draws::radio,
    },
    Panel {
        id: "switch",
        title: "switch",
        draw: draws::switch,
    },
    Panel {
        id: "meter",
        title: "meter",
        draw: draws::meter,
    },
    Panel {
        id: "sparkline",
        title: "sparkline",
        draw: draws::sparkline,
    },
    Panel {
        id: "rule",
        title: "rule",
        draw: draws::rule,
    },
    Panel {
        id: "status_bar",
        title: "status_bar",
        draw: draws::status_bar,
    },
    Panel {
        id: "pagination",
        title: "pagination",
        draw: draws::pagination,
    },
    Panel {
        id: "form",
        title: "form",
        draw: draws::form,
    },
    Panel {
        id: "slider",
        title: "slider",
        draw: draws::slider,
    },
    Panel {
        id: "file_picker",
        title: "file_picker",
        draw: draws::file_picker,
    },
    Panel {
        id: "file_preview_pane",
        title: "file_preview_pane",
        draw: draws::file_preview_pane,
    },
];

/// The ids the gallery shows, derived from [`PANELS`].
///
/// This is the second of the two lists [`crate::obligations::PANELS`] is compared against, and the
/// comparison is `tests::the_written_list_and_the_table_agree`. A written-out list is a claim about
/// twenty-eight rows of a table; the table is the screen.
pub fn panel_ids() -> Vec<&'static str> {
    PANELS.iter().map(|p| p.id).collect()
}

// ── the state ────────────────────────────────────────────────────────────────────────────────────

/// **Everything the twenty-eight panels keep between frames.**
///
/// One value the caller owns, because the runtime has no retained structure (ADR 0012): what
/// survives a frame is what the application holds.
pub struct Bag {
    /// The worker the two file components send their questions to.
    pub worker: Worker,
    field: Text,
    coll: CollState,
    table: TableState,
    tree: TreeState,
    forest: Order,
    select: SelectState,
    area: AreaState,
    collapse: Collapse,
    chart: Series,
    chart_state: PlotState,
    plot: Series,
    plot_state: PlotState,
    spark: Series,
    spark_state: PlotState,
    checkbox: bool,
    radio: bool,
    switch: bool,
    meter: f32,
    slider: f32,
    pager: CollState,
    form: FormState,
    form_texts: [Text; 3],
    picker: PickerState,
    pane: PaneState<Doc>,
    pane_task: Task<Doc>,
    pane_key: u64,
}

impl Bag {
    /// Every panel's state, at rest.
    ///
    /// The worker arrives from the caller because there are two: a hired one, which is what an
    /// application has, and [`Worker::queueing`], which is what a gate drives a frame at a time.
    pub fn new(worker: Worker) -> Bag {
        let mut field = Text::input();
        field.insert(22, "hello");
        let pane_task = Task::new(&worker);
        let mut picker = PickerState::new();
        picker.choose(1);
        Bag {
            worker,
            field,
            coll: CollState::new(),
            table: TableState::new(),
            tree: TreeState::new(),
            forest: Order::built(vec![
                Node::of(0),
                Node::of(1).at_depth(1),
                Node::of(2).at_depth(1),
                Node::of(3),
            ]),
            select: SelectState::at(1),
            area: AreaState { offset: (0, 2) },
            collapse: Collapse::open_at(3),
            chart: Series::build(1_000, 2),
            chart_state: PlotState::new(),
            plot: Series::build(1_000, 2),
            plot_state: PlotState::new(),
            spark: Series::build(1_000, 1),
            spark_state: PlotState::new(),
            checkbox: true,
            radio: true,
            switch: true,
            meter: 0.625,
            slider: 0.4,
            pager: CollState::new(),
            form: FormState::new(),
            form_texts: [Text::input(), Text::input(), Text::input()],
            picker,
            pane: PaneState::new(),
            pane_task,
            pane_key: 7,
        }
    }

    /// Which file the preview pane is asking about.
    ///
    /// **The key is the file and never the position**, which is components 31's finding through the
    /// one door that is not a gesture: a pane keyed by a list position is wrong on 100 frames of 100
    /// after one re-sort, with nothing to wake a frame that would correct it.
    pub const fn showing(&self) -> u64 {
        self.pane_key
    }

    /// Ask about the next file.
    pub fn show_next(&mut self) {
        self.pane_key = (self.pane_key + 1) % 16;
    }

    /// Answer every question a [`Worker::queueing`] worker is holding, on this thread.
    ///
    /// **A gate's verb, and it panics on a hired worker** — which is right: an application's worker
    /// answers its own questions and a test asserting otherwise is asserting against a schedule
    /// that did not happen. The loop is over `started..asked` because the two file components ask
    /// separately and a fixed index answers whichever happened to be first.
    pub fn answer_queued(&self) {
        while self.worker.started() < self.worker.asked() {
            let _ = self.worker.run(self.worker.started() as usize);
        }
    }

    /// Land whatever the worker has answered, if anything.
    ///
    /// **The two fields are split here rather than at the call site**, because a landing is taken at
    /// the top of the view (ADR 0039) and the view holds one `&mut Bag`.
    pub fn land_here(&mut self) -> crate::files::Landed {
        let Bag {
            pane, pane_task, ..
        } = self;
        pane.land(pane_task)
    }
}

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// **The smallest tile a panel is legible in.** Interior `(24, 5)` under a one-cell frame.
///
/// The measurement is [`crate::golden::SCREENS`]'s own rectangles, which are what a reviewer needs to
/// see each construction: **27 of the 33 fit inside 24x5**, the widest is 40 (`file_picker`, shut to
/// one row) and the tallest is 6 (`plot`'s braille rung and `chart`). It is a **floor**, not the
/// size a panel gets — [`grid`] grows the tiles into whatever the terminal has left, so at 300x80
/// a tile is 50x15.
pub const MIN_TILE: (u16, u16) = (26, 7);

/// The rows the gallery keeps for itself: a title over the grid and a status bar under it.
pub const CHROME_ROWS: u16 = 2;

/// **The tile grid: columns, rows, and panels per page.**
///
/// Two regimes, and the second is the interesting one. Where the terminal has room for every panel
/// at [`MIN_TILE`], the column count is the one whose tile comes closest to [`ASPECT`] and the tiles
/// grow into what is left — so a big terminal shows twenty-eight legible panels rather than
/// twenty-eight small ones in a corner, and not three columns a hundred cells wide either. Where it
/// does not, the grid is as large as it fits and the gallery pages.
///
/// **The tiles tile the grid exactly** either way; see [`Gallery::ui_into`], where the boundaries
/// are `grid.w * k / cols` rather than `k * tile_w`. A gap between tiles is a cell nobody writes,
/// and spec §2's second half is the whole subject of register row 7.
pub const fn grid(w: u16, h: u16) -> (u16, u16, usize) {
    let body = h.saturating_sub(CHROME_ROWS);
    let max_cols = w / MIN_TILE.0;
    let max_rows = body / MIN_TILE.1;
    if max_cols == 0 || max_rows == 0 {
        return (0, 0, 0);
    }
    let n = PANELS.len() as u16;
    if max_cols as u32 * max_rows as u32 >= n as u32 {
        // Every panel fits: choose the column count whose tile is closest to [`ASPECT`], so a wide
        // terminal shows twenty-eight legible panels rather than three columns of a hundred cells.
        let mut best = 0u16;
        let mut best_err = u32::MAX;
        let mut c = 1;
        while c <= max_cols {
            let r = n.div_ceil(c);
            if r <= max_rows {
                let have = (w / c) as u32;
                let want = ASPECT * (body / r) as u32;
                let err = have.abs_diff(want);
                if err < best_err {
                    best_err = err;
                    best = c;
                }
            }
            c += 1;
        }
        if best == 0 {
            best = max_cols;
        }
        return (best, n.div_ceil(best), PANELS.len());
    }
    (
        max_cols,
        max_rows,
        (max_cols as usize) * (max_rows as usize),
    )
}

/// **How many columns a cell is tall, roughly.** A terminal cell is about twice as tall as it is
/// wide and a panel's chrome costs two rows against two columns, so a tile that *looks* square is
/// about three times as wide as it is high. It decides nothing but the arrangement: every count on
/// this screen is over whatever grid it produces, and [`grid`] is the one place it is read.
pub const ASPECT: u32 = 3;

/// How many pages the twenty-eight panels need at this size. Never zero.
pub fn pages(w: u16, h: u16) -> usize {
    let (_, _, per) = grid(w, h);
    if per == 0 {
        return 1;
    }
    PANELS.len().div_ceil(per)
}

/// The `k`th boundary of `span` cut into `n`, so that the pieces tile it exactly.
const fn cut(origin: i32, span: u16, n: u16, k: u16) -> i32 {
    origin + (span as u32 * k as u32 / n as u32) as i32
}

/// The tile a slot of the current page occupies inside `grid_rows`.
pub const fn tile(grid_rows: Rect, cols: u16, rows: u16, slot: u16) -> Rect {
    let col = slot % cols;
    let row = slot / cols;
    let x0 = cut(grid_rows.x, grid_rows.w, cols, col);
    let x1 = cut(grid_rows.x, grid_rows.w, cols, col + 1);
    let y0 = cut(grid_rows.y, grid_rows.h, rows, row);
    let y1 = cut(grid_rows.y, grid_rows.h, rows, row + 1);
    Rect::new(x0, y0, (x1 - x0) as u16, (y1 - y0) as u16)
}

/// **The assembled gallery.**
pub struct Gallery {
    /// Every panel's state, except the three borrows a popup body captures.
    pub bag: Bag,
    /// `select`'s popup. **Beside the bag and not in it**: a fn pointer can hand over a `'f` borrow
    /// only out of a field the view has split off, and a field split off `Bag` would leave the rest
    /// of the bag unreachable behind the same borrow.
    select_popup: PopupState,
    /// `file_picker`'s overlay body, for [`Gallery::select_popup`]'s reason.
    picker_body: PickerBody<Doc>,
    /// Where the picker's answers land.
    picker_task: Task<Doc>,
    themes: Themes,
    rung: GlyphSet,
    tier: ColorDepth,
    page: usize,
    clears: Clears,
    shown: usize,
    /// The heading, staged into one reused buffer.
    ///
    /// **A field and not a `format!`**, because *zero allocations during frame composition* is a
    /// budget the gallery inherits like anything else and a `String` per frame is two of them. It is
    /// the arrangement `Ctx::stage` uses one crate down, spelled with a `String` here because
    /// `text_into` takes a `&str` and the crate's own components take their labels the same way.
    heading: String,
    /// The status bar's own segment, for [`Gallery::heading`]'s reason.
    left: String,
}

impl Gallery {
    /// A gallery at the registry's first scheme, the default repertoire and true colour.
    pub fn new(worker: Worker) -> Gallery {
        let mut themes = Themes::standard();
        // The richest rung, which is also `GlyphSet`'s own default — written as `RUNGS[2]` because a
        // file here may not spell a repertoire. See [`Gallery::next_rung`].
        let rung = RUNGS[2];
        let tier = ColorDepth::TrueColor;
        themes.set_glyphs(rung);
        themes.set_tier(tier);
        let picker_task = Task::new(&worker);
        Gallery {
            bag: Bag::new(worker),
            select_popup: PopupState::new(),
            picker_body: PickerBody::new(),
            picker_task,
            themes,
            rung,
            tier,
            page: 0,
            clears: Clears::new(),
            shown: 0,
            heading: String::new(),
            left: String::new(),
        }
    }

    /// The theme the loop is to hand the driver.
    ///
    /// **A pick reaches the swap one frame later and cannot do better**: a component may not write
    /// to the frame's environment mid-frame, so the gallery returns a theme and the loop applies it
    /// between frames.
    pub const fn theme(&self) -> &Theme {
        self.themes.theme()
    }

    /// Which scheme is showing, for the status bar.
    pub fn scheme(&self) -> &'static str {
        self.themes.scheme().name()
    }

    /// Which of the fourteen, and how many there are.
    pub const fn scheme_at(&self) -> (usize, usize) {
        (self.themes.selected(), self.themes.len())
    }

    /// The repertoire axis of §16's matrix.
    pub const fn rung(&self) -> GlyphSet {
        self.rung
    }

    /// The colour axis of §16's matrix.
    pub const fn tier(&self) -> ColorDepth {
        self.tier
    }

    /// Which page is showing, and how many there are at this size.
    ///
    /// **Clamped, because the answer must not depend on whether a frame has been drawn since the
    /// last resize.** [`Gallery::ui_into`] clamps the field itself; this reads the same clamp so an
    /// application's status line and the gallery's own agree on the frame a resize arrives.
    pub fn paging(&self, w: u16, h: u16) -> (usize, usize) {
        let of = pages(w, h);
        (self.page.min(of.saturating_sub(1)), of)
    }

    /// How many times the screen has been cleared. `crate::app::Clears`'s own count, forwarded so
    /// that *once, and then never again until the size changes* is checkable from outside.
    pub fn cleared(&self) -> u32 {
        self.clears.cleared()
    }

    /// How many panels the last frame drew. **A count and not an arithmetic**: the last page is
    /// short, and a gallery that reported `per_page` would report a page it did not draw.
    pub const fn shown(&self) -> usize {
        self.shown
    }

    /// The next theme of the fourteen, wrapping.
    ///
    /// The whole theme is re-imported and re-resolved here, on a live frame, which is what makes `t`
    /// the test of *degradation is resolved at construction* rather than a decoration: if
    /// construction were expensive, this key would stutter.
    pub fn next_theme(&mut self) {
        let at = (self.themes.selected() + 1) % self.themes.len();
        self.themes.select(at);
    }

    /// The previous theme of the fourteen, wrapping.
    pub fn prev_theme(&mut self) {
        let at = (self.themes.selected() + self.themes.len() - 1) % self.themes.len();
        self.themes.select(at);
    }

    /// The next of the three repertoires, wrapping. §16's first axis.
    ///
    /// **Through [`RUNGS`] and never by naming a variant.** A file in this crate that spells
    /// `GlyphSet::` fails `crate::gates`'s repertoire scan, whose one named exception is
    /// `chart/raster.rs` — and that file's own note says why `RUNGS[2]` is what a caller writes
    /// instead: *a test that sweeps the ladder, a report that prints it and a caller that wants the
    /// richest rung all need to name one, and every one of them naming one directly would turn a
    /// stated exception into a spreading one.* This module is the third of those three at once.
    pub fn next_rung(&mut self) {
        let at = RUNGS.iter().position(|r| *r == self.rung).unwrap_or(0);
        self.set_rung(RUNGS[(at + 1) % RUNGS.len()]);
    }

    /// The next of the three colour depths a human can tell apart, wrapping. §16's second axis.
    ///
    /// **Three and not four.** `ColorDepth` has four arms; the matrix is three by three because
    /// `Indexed256` and `TrueColor` differ on the wire and not on the traffic light, and what this
    /// key exists to show is the traffic light going monochrome. `Indexed256` is the middle rung and
    /// `TrueColor` the top; the fourth arm is reachable through [`Gallery::set_tier`], which is what
    /// the matrix gate sweeps.
    pub fn next_tier(&mut self) {
        let next = match self.tier {
            ColorDepth::None => ColorDepth::Ansi16,
            ColorDepth::Ansi16 => ColorDepth::Indexed256,
            ColorDepth::Indexed256 | ColorDepth::TrueColor => ColorDepth::None,
        };
        self.set_tier(next);
    }

    /// Put the gallery at one cell of the matrix.
    pub fn set_tier(&mut self, tier: ColorDepth) {
        self.tier = tier;
        self.themes.set_tier(tier);
    }

    /// Put the gallery at one cell of the matrix.
    pub fn set_rung(&mut self, rung: GlyphSet) {
        self.rung = rung;
        self.themes.set_glyphs(rung);
    }

    /// The density every theme in the set is built with. Spec §3: density is theme data.
    pub fn set_density(&mut self, density: Density) {
        self.themes.set_density(density);
    }

    /// The next page, wrapping.
    pub fn next_page(&mut self, w: u16, h: u16) {
        self.page = (self.page + 1) % pages(w, h);
    }

    /// The previous page, wrapping.
    pub fn prev_page(&mut self, w: u16, h: u16) {
        let n = pages(w, h);
        self.page = (self.page + n - 1) % n;
    }

    /// Put the gallery on the page a named panel is on. `false` if there is no such panel.
    pub fn go_to(&mut self, id: &str, w: u16, h: u16) -> bool {
        let Some(at) = PANELS.iter().position(|p| p.id == id) else {
            return false;
        };
        let (_, _, per) = grid(w, h);
        self.page = at.checked_div(per).unwrap_or(0);
        true
    }

    /// Which panels this size and page put on the screen.
    pub fn page_panels(&self, w: u16, h: u16) -> &'static [Panel] {
        let (_, _, per) = grid(w, h);
        if per == 0 {
            return &[];
        }
        let from = (self.paging(w, h).0 * per).min(PANELS.len());
        let to = (from + per).min(PANELS.len());
        &PANELS[from..to]
    }

    /// **One frame, top to bottom.**
    ///
    /// `&'f mut self` and `Ctx<'f, '_>` are one lifetime deliberately: two of the panels take `&'f
    /// mut` of the state their popup body captures, and `console`'s own header records what the
    /// alternative costs — `E0502` at a call site three lines from a popup nobody was thinking
    /// about.
    pub fn ui_into<'f, I: Ink>(&'f mut self, ink: &mut I, cx: &mut Ctx<'f, '_>, note: &str) {
        let (w, h) = cx.size();
        let whole = cx.area();

        // **The preview pane's landing, taken at the top of the view** (ADR 0039). `Task::take` is
        // destructive and a view has no `&mut` with which to put back what it took, so the landing
        // is a verb on the state before anything draws — components 31 measured what taking it
        // where a view happens to want it costs: **20 torn frames of 20**, on a screen whose state
        // ends up correct either way.
        let _ = self.bag.land_here();

        // **The screen clears once, on its first frame and on a resize** (spec §2, ADR 0026).
        // Clearing every frame is 9 024 cells on a screen that is not moving; clearing never leaves
        // whatever the shell had there showing through the gaps.
        // **Through the caller's ink, not through `Direct`.** The clear is `w * h` writes and it is
        // the application's `Ctrl+O` counters that would otherwise be blind to them — on exactly the
        // frame a reader turns the counters on to look at. What it costs is stated rather than
        // discovered: [`shape`] measures a **steady** frame on a fresh recorder for this reason, so
        // `Shape::unwritten` is *cells no verb wrote this frame* and not *cells nobody ever wrote*,
        // which over a screen that clears once is zero by construction.
        let _ = self.clears.frame_into(ink, cx);

        let (title, rest) = rect::split_at_v(whole, 1);
        let (grid_rows, status) = rect::split_at_v(rest, rest.h.saturating_sub(1));

        let (cols, rows, per) = grid(w, h);
        let scheme = self.scheme();
        let (at, of) = self.scheme_at();
        let of_pages = self.paging(w, h).1;
        let (rung, tier) = (self.rung, self.tier);

        self.heading.clear();
        let _ = write!(
            self.heading,
            " vitui gallery — {} of {} panels · {scheme} ({}/{of}) · {} · {}",
            PANELS.len(),
            crate::INVENTORY.len(),
            at + 1,
            rung_word(rung),
            tier_word(tier),
        );
        let _ = text_into(
            ink,
            cx,
            title,
            &self.heading,
            &TextOpts {
                role: Role::Title,
                ..Default::default()
            },
        );

        // **The page is clamped against *this* size, and it has to be here.** `next_page` and
        // `prev_page` take a size and wrap; nothing they do survives the terminal being made
        // **larger**, because `pages()` shrinks under them and an index past the end yields an empty
        // range rather than a clamp — a title and a status bar reading *0 panels · page 4/1* over a
        // grid nobody writes, which then stays until the user presses a key. A resize is the one
        // event that changes the page count without going through a key.
        self.page = self.page.min(of_pages.saturating_sub(1));
        let page = self.page;
        let from = if per == 0 {
            0
        } else {
            (page * per).min(PANELS.len())
        };
        let to = if per == 0 {
            0
        } else {
            (from + per).min(PANELS.len())
        };

        let Gallery {
            bag,
            select_popup,
            picker_body,
            picker_task,
            shown,
            left,
            ..
        } = self;
        let mut owners = Owners {
            select: Some(select_popup),
            picker: Some(picker_body),
            files: &FILES,
            task: picker_task,
        };
        let drawn = tiles_into(
            bag,
            &mut owners,
            ink,
            cx,
            grid_rows,
            cols.max(1),
            rows.max(1),
            from..to,
        );
        *shown = drawn;

        // **The caller's own segment, and the gallery owns the layout.** What a status bar says
        // about a frame is read *after* the frame — the counters are the previous frame's ring — so
        // the text arrives as an argument rather than being computed here from numbers this frame
        // does not have yet.
        left.clear();
        let _ = write!(
            left,
            "page {}/{of_pages} · {drawn} panels · {} · {}",
            page + 1,
            rung_word(rung),
            tier_word(tier),
        );
        const KEYS: &str = "Tab focus · t/Ctrl+T theme · Ctrl+G rung · Ctrl+L colour · Ctrl+N/P page · Ctrl+Q quit";
        let _ = status_bar_into(
            ink,
            cx,
            status,
            &[left.as_str(), note, KEYS],
            (0, 0),
            &StatusOpts::default(),
        );
    }

    /// **One named panel, alone, over the whole area.**
    ///
    /// The instrument [`shot`] draws through, and it goes through the same tile loop the screen does
    /// — so what a gate photographs is the panel the gallery draws rather than a second arrangement
    /// of the same call. `false`, and nothing drawn, if no panel has that id.
    pub fn one_into<'f, I: Ink>(&'f mut self, ink: &mut I, cx: &mut Ctx<'f, '_>, id: &str) -> bool {
        let Some(at) = PANELS.iter().position(|p| p.id == id) else {
            return false;
        };
        let whole = cx.area();
        let _ = self.bag.land_here();
        let Gallery {
            bag,
            select_popup,
            picker_body,
            picker_task,
            shown,
            ..
        } = self;
        let mut owners = Owners {
            select: Some(select_popup),
            picker: Some(picker_body),
            files: &FILES,
            task: picker_task,
        };
        *shown = tiles_into(bag, &mut owners, ink, cx, whole, 1, 1, at..at + 1);
        true
    }
}

/// **The tile loop, and the only place a panel is drawn.**
///
/// A free function over the borrows [`Gallery::ui_into`] has already split, because a method taking
/// `&'f mut self` leaves nothing of `self` to write the count back to.
///
/// `slots` indexes [`PANELS`]; the tile a slot lands on is its **position in the range**, so one
/// panel alone gets the whole area and a page's twelve get twelve tiles.
#[expect(
    clippy::too_many_arguments,
    reason = "every one of them is a borrow the caller had to split off before the loop could \
              start, and folding them into a struct would give that struct the `'f` the fn pointer \
              cannot carry"
)]
fn tiles_into<'f, I: Ink>(
    bag: &'f mut Bag,
    owners: &mut Owners<'f>,
    ink: &mut I,
    cx: &mut Ctx<'f, '_>,
    area: Rect,
    cols: u16,
    rows: u16,
    slots: std::ops::Range<usize>,
) -> usize {
    let opts = PanelOpts {
        padded: false,
        ..Default::default()
    };
    let mut drawn = 0usize;
    for (slot, panel) in PANELS[slots].iter().enumerate() {
        let here = tile(area, cols, rows, slot as u16);
        if here.is_empty() {
            break;
        }
        let bag = &mut *bag;
        let owners = &mut *owners;
        let ink = &mut *ink;
        // **`with_key`, and the call site alone is not enough.** `panel_into` takes `cx.id()` and
        // there is one call to it here, so twenty-eight tiles drawn from this loop would be
        // twenty-eight widgets under one `Id` — components 30's `chrome` defect exactly, where
        // `Ctx::interact` makes a merged claim **inert** and every tile but the first stops hearing
        // the pointer on a screen that renders perfectly. `defective::tiles_under_one_id` is the
        // spelling this replaced and the gate watches it merging.
        cx.with_key(slot as u64, |cx| {
            let frame = panel_into(&mut *ink, cx, here, panel.title, &opts);
            let mut sink: Sink<'_> = ink;
            (panel.draw)(bag, owners, &mut sink, cx, frame.interior);
        });
        drawn += 1;
    }
    drawn
}

/// The three repertoires' words, in [`RUNGS`]'s order.
///
/// `GlyphSet::word` is `pub(crate)` in the engine, so the spelling has to be this crate's — and it is
/// a table indexed by position rather than a `match`, because a `match` on the enum spells
/// `GlyphSet::` and this file is not the one named exception to the scan that forbids it. Pinned
/// against `Debug` in `tests::the_rung_and_tier_words_are_the_engines_own`, which is
/// [`crate::golden::tier`]'s arrangement for the same reason.
pub const RUNG_WORDS: [&str; 3] = ["ascii", "unicode", "extended"];

/// The repertoire's own word. See [`RUNG_WORDS`].
pub fn rung_word(g: GlyphSet) -> &'static str {
    RUNG_WORDS[RUNGS.iter().position(|r| *r == g).unwrap_or(0)]
}

/// The colour depth's own word, for the same reason.
pub const fn tier_word(t: ColorDepth) -> &'static str {
    match t {
        ColorDepth::None => "none",
        ColorDepth::Ansi16 => "ansi16",
        ColorDepth::Indexed256 => "indexed256",
        ColorDepth::TrueColor => "truecolor",
    }
}

// ── the twenty-eight drawings ────────────────────────────────────────────────────────────────────

/// **One function per built row of the freeze, each drawing the shipped component.**
///
/// Private, for [`crate::golden`]'s reason: every one is named after a component and this crate
/// already carries three homonyms that are not components. [`PANELS`] carries the pointers.
mod draws {
    use super::*;

    pub fn text(
        _b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let _ = text_into(
            ink,
            cx,
            r,
            "the quick brown fox jumps",
            &TextOpts::default(),
        );
    }

    pub fn panel(
        _b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let _ = panel_into(ink, cx, r, "General", &PanelOpts::default());
    }

    pub fn chip(
        _b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let _ = chip_into(ink, cx, one_row(r), "draft", &ChipOpts::default());
    }

    pub fn button(
        _b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let _ = button_into(ink, cx, one_row(r), "Save", &ButtonOpts::default());
    }

    pub fn field(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let _ = field_into(ink, cx, r, &mut b.field, &FieldOpts::default());
    }

    pub fn collection(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let _ = collection_into(
            ink,
            cx,
            r,
            &mut b.coll,
            &CollOpts::default(),
            Rows::of(NAMES.len()),
            |_buf, _range| None,
            |ink, cx, row, i, face| {
                let paint = face_paint(cx.theme(), face);
                ink.pad_to(cx, row.x, row.y, NAMES[i], row.w, paint);
            },
        );
    }

    pub fn table(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let cols = [
            Column::new(0, "id", Constraint::Fixed(4)),
            Column::new(1, "name", Constraint::Weight(1)),
        ];
        let _ = table_into(
            ink,
            cx,
            r,
            &mut b.table,
            &TableOpts::default(),
            &cols,
            Rows::of(NAMES.len()),
            |_buf, _range| None,
            |ink, cx, cell, c, face| {
                let paint = face_paint(cx.theme(), face);
                let text = if c.key == 0 {
                    NAMES[c.row].get(..2).unwrap_or("--")
                } else {
                    NAMES[c.row]
                };
                ink.pad_to(cx, cell.x, cell.y, text, cell.w, paint);
            },
        );
    }

    pub fn tree(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let Bag { tree, forest, .. } = b;
        let _ = tree_into(
            ink,
            cx,
            r,
            tree,
            &TreeOpts::default(),
            forest,
            |_buf, _range| None,
            |ink, cx, row, node, face| {
                let paint = face_paint(cx.theme(), face);
                ink.pad_to(cx, row.x, row.y, NAMES[node.node as usize], row.w, paint);
            },
        );
    }

    /// **Shut, and the popup is a layer.** `select`'s own screen is its face; the popup's interior
    /// is measured by `crate::popup`, because an overlay body takes no ink (spec §1, components 26).
    pub fn select<'f>(
        b: &mut Bag,
        o: &mut Owners<'f>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'f, '_>,
        r: Rect,
    ) {
        let popup = o
            .select
            .take()
            .expect("a panel is drawn once a frame; a second take is two widgets under one id");
        let id = cx.id();
        let _ = select_into(
            ink,
            cx,
            id,
            one_row(r),
            &mut b.select,
            popup,
            &OPTIONS,
            &SelectOpts::default(),
        );
    }

    pub fn overlay(
        _b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let id = cx.id();
        let _ = overlay_into(
            ink,
            cx,
            id,
            r,
            12,
            &ShellOpts::default(),
            |ink, cx, body| {
                let paint = cx.theme().paint(Role::Body);
                for y in 0..body.h {
                    ink.pad_to(
                        cx,
                        body.x,
                        body.y + i32::from(y),
                        NAMES[usize::from(y) % NAMES.len()],
                        body.w,
                        paint,
                    );
                }
            },
        );
    }

    pub fn scroll_area(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let id = cx.id();
        let _ = scroll_area_into(
            ink,
            cx,
            id,
            r,
            &mut b.area,
            &AreaOpts::default(),
            (u32::from(r.w), 40),
            |_ink, _cx, _band| {},
            |ink, cx| {
                let paint = cx.theme().paint(Role::Body);
                let cols = cx.visible_cols();
                let width = (cols.end - cols.start).max(0) as u16;
                for y in cx.visible_rows() {
                    ink.pad_to(
                        cx,
                        cols.start,
                        y,
                        NAMES[(y as usize) % NAMES.len()],
                        width,
                        paint,
                    );
                }
            },
        );
    }

    pub fn scrollbar(
        _b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let id = cx.id();
        let span = Span {
            viewport: u32::from(r.h),
            extent: u32::from(r.h) * 4,
            offset: u32::from(r.h),
        };
        let bar = Rect::new(r.x, r.y, 3.min(r.w), r.h);
        let _ = scrollbar_into(ink, cx, id, bar, span, &ScrollbarOpts::default());
    }

    /// **The band, which is all `sticky` is** (spec §9): the header shares `x` with a body scrolled
    /// four columns right, and the clip is what keeps it inside its own rectangle.
    pub fn sticky(
        _b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let title = cx.theme().paint(Role::Title);
        let body = cx.theme().paint(Role::Body);
        let (head, rows) = rect::split_at_v(r, 1);
        // **The band's own coordinates, not the caller's.** `sticky` childs at the band and puts
        // the shared axis back into content columns, so the header's first visible column is
        // content column 4 and it is written at `x == 4`.
        let _ = sticky_band(cx, head, Shares::X, (4, 0), |cx| {
            ink.pad_to(cx, 4, 0, "name          size", head.w, title);
        });
        for y in 0..rows.h {
            ink.pad_to(
                cx,
                rows.x,
                rows.y + i32::from(y),
                NAMES[usize::from(y) % NAMES.len()],
                rows.w,
                body,
            );
        }
    }

    pub fn collapsible(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let _ = collapsible_into(
            ink,
            cx,
            r,
            &mut b.collapse,
            "General",
            &DiscloseOpts::default(),
            |_w| 3,
            |ink, cx| {
                let paint = cx.theme().paint(Role::Body);
                let w = cx.area().w;
                for y in 0..3u16 {
                    ink.pad_to(cx, 0, i32::from(y), NAMES[usize::from(y)], w, paint);
                }
            },
        );
    }

    pub fn chart(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let Bag {
            chart, chart_state, ..
        } = b;
        let _ = chart_into(ink, cx, r, chart, chart_state, &PlotOpts::chart());
    }

    pub fn plot(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let Bag {
            plot, plot_state, ..
        } = b;
        let _ = plot_into(ink, cx, r, plot, plot_state, &PlotOpts::plot());
    }

    pub fn checkbox(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let opts = ToggleOpts {
            kind: Toggle::Check,
            ..Default::default()
        };
        let _ = toggle_into(ink, cx, one_row(r), "enabled", &mut b.checkbox, &opts);
    }

    pub fn radio(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let opts = ToggleOpts {
            kind: Toggle::Radio,
            ..Default::default()
        };
        let _ = toggle_into(ink, cx, one_row(r), "enabled", &mut b.radio, &opts);
    }

    pub fn switch(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let opts = ToggleOpts {
            kind: Toggle::Switch,
            ..Default::default()
        };
        let _ = toggle_into(ink, cx, one_row(r), "enabled", &mut b.switch, &opts);
    }

    pub fn meter(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let _ = meter_into(ink, cx, one_row(r), b.meter, &MeterOpts::default());
    }

    pub fn sparkline(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let Bag {
            spark, spark_state, ..
        } = b;
        let _ = sparkline_into(ink, cx, r, spark, spark_state, &SparkOpts::default());
    }

    pub fn rule(
        _b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let _ = rule_into(ink, cx, one_row(r), " limits ", &RuleOpts::default());
    }

    pub fn status_bar(
        _b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let _ = status_bar_into(
            ink,
            cx,
            one_row(r),
            &["ready", "3 of 9"],
            (0, 0),
            &StatusOpts::default(),
        );
    }

    pub fn pagination(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let _ = pagination_into(ink, cx, one_row(r), &mut b.pager, 9, &PageOpts::default());
    }

    pub fn form(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        const LABELS: [&str; 3] = ["name", "email", "role"];
        let Bag {
            form, form_texts, ..
        } = b;
        let _ = form_into(ink, cx, r, form, &LABELS, form_texts, &FormOpts::default());
    }

    pub fn slider(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let _ = slider_into(ink, cx, one_row(r), &mut b.slider, &SliderOpts::default());
    }

    /// **Shut, one row** — [`select`]'s reason one family over.
    pub fn file_picker<'f>(
        b: &mut Bag,
        o: &mut Owners<'f>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'f, '_>,
        r: Rect,
    ) {
        let body = o
            .picker
            .take()
            .expect("a panel is drawn once a frame; a second take is two widgets under one id");
        let id = cx.id();
        let _ = file_picker_into(
            ink,
            cx,
            id,
            one_row(r),
            &mut b.picker,
            body,
            o.files,
            o.task,
            decode,
            preview_line,
            &PickerOpts::default(),
        );
    }

    pub fn file_preview_pane(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let id = cx.id();
        let Bag {
            pane,
            pane_task,
            pane_key,
            ..
        } = b;
        let key = *pane_key;
        let _ = file_preview_pane_into(
            ink,
            cx,
            id,
            r,
            pane,
            pane_task,
            asking(key, move |_cancel| Doc(key)),
            &PaneOpts::default(),
            |ink, cx, row, _doc: &Doc, i| {
                let paint = cx.theme().paint(Role::Body);
                ink.pad_to(
                    cx,
                    row.x,
                    row.y,
                    ROWS[i as usize % ROWS.len()],
                    row.w,
                    paint,
                );
            },
        );
    }

    /// The top row of an interior, for the eleven components whose drawing is one row.
    const fn one_row(r: Rect) -> Rect {
        Rect::new(r.x, r.y, r.w, 1)
    }
}

// ── the report ───────────────────────────────────────────────────────────────────────────────────

/// What one frame of the gallery cost, at one size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shape {
    /// Panels drawn.
    pub panels: usize,
    /// Cells the components asked the engine to write.
    pub writes: u64,
    /// Distinct cells any verb touched.
    pub distinct: u64,
    /// Drawing verbs.
    pub verbs: u64,
    /// Hit entries the frame declared.
    pub regions: usize,
    /// Tab stops the frame declared.
    pub stops: usize,
    /// Widgets that arrived under an id another widget had already claimed.
    pub merges: u32,
    /// **Cells no verb wrote on the frame measured.** Register row 7's subject, reported and not
    /// gated — components 40 owns it.
    ///
    /// A *steady* frame, and the distinction is the whole number: `crate::app::Clears` writes every
    /// cell of the screen on the first frame and on a resize, so *cells nobody ever wrote* is zero
    /// on any screen that clears and says nothing about any component. What this counts is the cells
    /// the twenty-eight panels and the chrome leave to whatever was already there.
    pub unwritten: usize,
}

/// **One frame of the gallery at `(w, h)`, and what it cost.**
///
/// The instrument is [`crate::runner::Pen`], so the numbers are of the shipped drawing and not of a
/// copy: every panel routes through the same `Ink` an application's does.
pub fn shape(w: u16, h: u16, frames: u32) -> Shape {
    let mut driver = crate::runner::driver_at(w, h, Density::default());
    let mut gallery = Gallery::new(Worker::queueing());
    driver.set_theme(*gallery.theme());
    // **Warmed, then measured on a fresh recorder.** The warm frames are what the two file
    // components need — the answer arrives on a frame after the one that asked (spec §15) — and the
    // fresh recorder is what makes `unwritten` a **steady frame's** number: `crate::app::Clears`
    // writes every cell of the screen on the first frame and on a resize, so a recorder carried
    // across that frame answers *nobody ever left a cell alone*, which is zero on any screen that
    // clears and says nothing about any component.
    let mut warm = crate::runner::Pen::new(w, h);
    for _ in 0..frames.max(2) - 1 {
        gallery.bag.answer_queued();
        warm.end_frame();
        let mut sink: Sink<'_> = &mut warm;
        driver.frame(|cx| gallery.ui_into(&mut sink, cx, ""));
    }
    let mut pen = crate::runner::Pen::new(w, h);
    {
        gallery.bag.answer_queued();
        let mut sink: Sink<'_> = &mut pen;
        driver.frame(|cx| gallery.ui_into(&mut sink, cx, ""));
    }
    let cells = usize::from(w) * usize::from(h);
    let written = pen.canvas().written();
    Shape {
        panels: gallery.shown(),
        writes: pen.tally().writes(),
        distinct: pen.tally().distinct(),
        verbs: pen.tally().verbs(),
        regions: driver.inspect().hits().len(),
        stops: driver.inspect().stop_count(),
        merges: driver.inspect().ids().merges(),
        unwritten: cells - written,
    }
}

/// **One named panel, photographed alone over `w x h`.**
///
/// `frames` is the warm-up: two is what a gallery needs for the hover index and the ring, and three
/// is what the preview pane needs — the answer arrives on a frame after the one that asked (spec
/// §15).
pub fn shot(id: &str, w: u16, h: u16, frames: u32) -> crate::runner::Canvas {
    let mut driver = crate::runner::driver_at(w, h, Density::default());
    let mut gallery = Gallery::new(Worker::queueing());
    driver.set_theme(*gallery.theme());
    let mut pen = crate::runner::Pen::new(w, h);
    let mut found = false;
    for _ in 0..frames.max(1) {
        gallery.bag.answer_queued();
        pen.end_frame();
        let mut sink: Sink<'_> = &mut pen;
        driver.frame(|cx| found = gallery.one_into(&mut sink, cx, id));
    }
    assert!(found, "no panel is named {id}");
    pen.into_canvas()
}

/// What a live key changes about the theme. [`swap`]'s axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Change {
    /// The next of the fourteen schemes — `t`. Every role's colour moves.
    Scheme,
    /// The next of the three repertoires — `Ctrl+G`. Every **glyph** moves and no colour does.
    Rung,
    /// The next colour depth — `Ctrl+L`. The same palette, re-quantised.
    Tier,
}

/// What a theme swap moved, and what it did not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Swap {
    /// Which key was pressed.
    pub change: Change,
    /// Cells whose value differs after the swap.
    pub changed: usize,
    /// **Cells that carry the old theme's value a frame after the swap.** Register row 8's subject,
    /// reported and not gated — components 41 owns it.
    pub kept: usize,
    /// Cells anybody wrote, which is the denominator both of the above are read against.
    pub written: usize,
    /// **Of [`Swap::changed`], how many are on a panel** — outside the heading row and the status
    /// row, which are the gallery's own chrome.
    ///
    /// It exists because `changed > 0` is satisfied by the chrome alone, and for [`Change::Tier`]
    /// that is **all** it is satisfied by: both chrome rows print the depth's own name, so
    /// `truecolor` becoming `none` moves ten cells at 300x80 — five in the heading and five in the
    /// status bar — and **not one cell of any panel**. A gate on `changed` cannot tell *the colour
    /// axis moves nothing on a canvas*, which is this module's own finding, from *the gallery
    /// stopped redrawing*.
    ///
    /// **Both rows, and excluding only the first was a defect in this field.** At 100x30 the status
    /// bar's left segment is elided before it reaches the depth, so the heading alone accounts for
    /// all five cells and the number came out 0; at 300x80 there is room for both and it came out
    /// **5**, which reads as a panel moving. One size agreeing with a law the other breaks is how a
    /// gate over one size stays green.
    pub changed_on_a_panel: usize,
}

/// **One theme swap over the assembled gallery, read cell for cell.**
///
/// The screen register row 8 is measured on. It is a **report** here and deliberately so: the row is
/// pinned red with components 41 named as its inverter, and a ticket that also inverted it would
/// leave nothing for the one that owns the rule — *a memo carries the theme in its key iff its value
/// is made of paints or glyphs*.
///
/// **`changed` is printed beside `kept` because `changed > 0` is the gate this map already got
/// wrong**: one cell of 4 800 satisfies it while 3 583 carry the old palette (§21's refinement 1).
pub fn swap(w: u16, h: u16, change: Change) -> Swap {
    let mut driver = crate::runner::driver_at(w, h, Density::default());
    let mut gallery = Gallery::new(Worker::queueing());
    driver.set_theme(*gallery.theme());
    let mut before = crate::runner::Pen::new(w, h);
    for _ in 0..3 {
        gallery.bag.answer_queued();
        before.end_frame();
        let mut sink: Sink<'_> = &mut before;
        driver.frame(|cx| gallery.ui_into(&mut sink, cx, ""));
    }
    let first = before.into_canvas();

    match change {
        Change::Scheme => gallery.next_theme(),
        Change::Rung => gallery.next_rung(),
        Change::Tier => gallery.next_tier(),
    }
    driver.set_theme(*gallery.theme());
    let mut after = crate::runner::Pen::new(w, h);
    for _ in 0..2 {
        gallery.bag.answer_queued();
        after.end_frame();
        let mut sink: Sink<'_> = &mut after;
        driver.frame(|cx| gallery.ui_into(&mut sink, cx, ""));
    }
    let second = after.into_canvas();

    let mut changed = 0;
    let mut changed_on_a_panel = 0;
    let mut kept = 0;
    let mut written = 0;
    for y in 0..h {
        for x in 0..w {
            match (first.get(x, y), second.get(x, y)) {
                (Some(a), Some(b)) => {
                    written += 1;
                    // **What *keeping the old theme* means depends on which key was pressed**, and
                    // reading one axis for the other is how a swap gate goes green: a scheme change
                    // moves paints and not clusters, and a rung change moves clusters and not
                    // paints. Compared on the pair, a rung change reports every cell stale.
                    let stale = match change {
                        Change::Scheme | Change::Tier => a.paint == b.paint,
                        Change::Rung => a.cluster == b.cluster,
                    };
                    if stale {
                        kept += 1;
                    } else {
                        changed += 1;
                        if y > 0 && y + 1 < h {
                            changed_on_a_panel += 1;
                        }
                    }
                }
                (_, Some(_)) => written += 1,
                _ => {}
            }
        }
    }
    Swap {
        change,
        changed,
        kept,
        written,
        changed_on_a_panel,
    }
}

/// **The spellings this module refuses, kept runnable so a gate can watch each one fail.**
///
/// Every `defective` module in this crate is a deliberate collection of what its subject replaced,
/// and none of them is reachable from the shipped path.
pub mod defective {
    use super::*;

    /// **The tile loop without [`Ctx::with_key`]**, which is what one call site to `panel_into`
    /// costs: every tile arrives under the id `Location::caller()` mints for that one line, so the
    /// second and every later one is a **merge** — and `Ctx::interact` makes a merged claim inert,
    /// so the panels stop hearing the pointer on a screen that renders perfectly.
    pub fn tiles_under_one_id<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, area: Rect, n: u16) {
        let opts = PanelOpts {
            padded: false,
            ..Default::default()
        };
        for slot in 0..n {
            let here = tile(area, n, 1, slot);
            if here.is_empty() {
                break;
            }
            let _ = panel_into(ink, cx, here, "tile", &opts);
        }
    }

    /// **Tiles laid out by multiplication instead of by the boundary arithmetic [`tile`] uses**, which
    /// leaves the remainder
    /// of a width that does not divide by the column count written by nobody. It is the gap between
    /// tiles §2's second half is about, and it is invisible at every size that happens to divide.
    pub fn tiles_by_multiplication<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, area: Rect, n: u16) {
        let opts = PanelOpts {
            padded: false,
            ..Default::default()
        };
        let each = area.w / n.max(1);
        for slot in 0..n {
            let here = Rect::new(area.x + i32::from(slot * each), area.y, each, area.h);
            if here.is_empty() {
                break;
            }
            let _ = panel_into(ink, cx, here, "tile", &opts);
        }
    }
}

// ── §16's nine cells ─────────────────────────────────────────────────────────────────────────────

/// One cell of §16's matrix: the repertoire axis against the colour axis, on this screen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MatrixCell {
    /// The repertoire the theme was built with.
    pub rung: GlyphSet,
    /// The depth it was resolved for.
    pub tier: ColorDepth,
    /// Cells written.
    pub writes: u64,
    /// Drawing verbs.
    pub verbs: u64,
    /// **How many distinct clusters the screen ends up carrying.** The repertoire axis as a number:
    /// a rung that spells two glyphs the same collapses two clusters into one.
    pub clusters: usize,
    /// **How many distinct paints the screen carries.**
    ///
    /// **Ten in all nine cells**, and that is the finding rather than a column: `Theme::resolve`
    /// does **not change what `Theme::paint` returns** — measured over all thirteen roles at all
    /// four depths, the paint is identical — because quantisation happens in the engine, before the
    /// mirror. What `resolve` changes is the wire triple `roles_differ_on_wire` reads and the ten
    /// distinction bits.
    ///
    /// So *the colour axis of §16's matrix is not observable on a canvas above the engine at all*,
    /// and that is ADR 0018 working rather than a hole: a component is handed the same paint whatever
    /// the terminal can show, which is what *a component names a role and never a colour* means. The
    /// column is reported so that the nine rows show the axis moving in
    /// [`MatrixCell::roles_collapsed`] and standing still here — §21's refinement 1, *a counter on
    /// the wrong side of the question is not a weak gate, it is a green one*, as a table.
    pub paints: usize,
    /// **Pairs of the thirteen roles that are one colour on the wire.** The colour axis, measured on
    /// the theme because it is not measurable on the screen.
    pub roles_collapsed: usize,
    /// **How many of the ten distinctions the theme does not show.** The colour axis again, at the
    /// level a component actually asks about (ADR 0032).
    pub distinctions_lost: usize,
}

/// **The three repertoires against the three colour depths, over this screen.**
///
/// §16's matrix is *a count, not nine screenshots*, and this is where the count is taken over an
/// assembled screen rather than over one component. The application's `--matrix` prints it and
/// `Ctrl+G`/`Ctrl+L` walk the same nine cells live, which is the half neither the runtime's
/// `theme_numbers` nor its `glyph_numbers` can do: those own the mechanism and measure it over the
/// palette, and this one measures what a human is looking at.
///
/// **Three colour rungs and not four**, deliberately: `Indexed256` and `TrueColor` differ on the
/// wire, not on the traffic light, and what the axis is about here is a distinction surviving or
/// dying. [`traffic_light`] is the fourth arm's own reading.
pub fn matrix(w: u16, h: u16) -> Vec<MatrixCell> {
    let mut out = Vec::with_capacity(9);
    for rung in RUNGS {
        for tier in [ColorDepth::None, ColorDepth::Ansi16, ColorDepth::TrueColor] {
            let mut driver = crate::runner::driver_at(w, h, Density::default());
            let mut gallery = Gallery::new(Worker::queueing());
            gallery.set_rung(rung);
            gallery.set_tier(tier);
            driver.set_theme(*gallery.theme());
            let mut clusters = std::collections::BTreeSet::new();
            let mut paints = std::collections::BTreeSet::new();
            let mut writes = 0;
            let mut verbs = 0;
            // **Every page, because the matrix is over the gallery and not over a page.** Read on
            // page one alone the repertoire axis reports `Extended == Unicode` — which is true of
            // that page and false of the screen, because `plot` is the one construction whose rung
            // is braille and it is on page two.
            for _ in 0..pages(w, h) {
                let mut pen = crate::runner::Pen::new(w, h);
                for _ in 0..2 {
                    gallery.bag.answer_queued();
                    pen.end_frame();
                    let mut sink: Sink<'_> = &mut pen;
                    driver.frame(|cx| gallery.ui_into(&mut sink, cx, ""));
                }
                pen.end_frame();
                let mut pen = crate::runner::Pen::over(pen.into_canvas());
                {
                    let mut sink: Sink<'_> = &mut pen;
                    driver.frame(|cx| gallery.ui_into(&mut sink, cx, ""));
                }
                writes += pen.tally().writes();
                verbs += pen.tally().verbs();
                for y in 0..h {
                    for x in 0..w {
                        if let Some(cell) = pen.canvas().get(x, y) {
                            clusters.insert(cell.cluster.clone());
                            paints.insert(format!("{:?}", cell.paint));
                        }
                    }
                }
                gallery.next_page(w, h);
            }
            let theme = *gallery.theme();
            let mut collapsed = 0;
            let roles = vitui_runtime::theme::Role::ALL;
            for (i, a) in roles.iter().enumerate() {
                for b in &roles[i + 1..] {
                    if !theme.roles_differ_on_wire(*a, *b) {
                        collapsed += 1;
                    }
                }
            }
            let lost = vitui_runtime::theme::Distinction::ALL
                .iter()
                .filter(|d| !theme.shows(**d))
                .count();
            out.push(MatrixCell {
                rung,
                tier,
                writes,
                verbs,
                clusters: clusters.len(),
                paints: paints.len(),
                roles_collapsed: collapsed,
                distinctions_lost: lost,
            });
        }
    }
    out
}

/// **What the three status roles are, at one colour depth, read three ways.**
///
/// The gallery's own criterion is *`Danger`, `Warn` and `Ok` all quantise to bright white at sixteen
/// colours — the gallery is where a human sees the traffic light go monochrome*. Two of the three
/// readings below say it does not, and the third says why a count could not tell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TrafficLight {
    /// The depth the fourteen shipped schemes were resolved for.
    pub tier: ColorDepth,
    /// How many of the fourteen **declare** the distinction — `Theme::shows(Distinction::Status)`.
    pub declares: usize,
    /// How many keep the three **distinct on the wire** — `Theme::roles_differ_on_wire` twice.
    pub on_the_wire: usize,
    /// How many have three distinct [`vitui_runtime::Paint`]s.
    ///
    /// **Fourteen at every depth, including `ColorDepth::None`**, which is why this field exists —
    /// and the reason is not that a paint is a coarse comparison. It is that `Theme::resolve`
    /// **returns the same paint at every depth**: quantisation is the engine's, before the mirror,
    /// and a component is handed the palette's own colour whatever the terminal can show. A gate
    /// written over paint equality reports the traffic light healthy on a terminal with no colour at
    /// all, and it would do so for ever.
    pub distinct_paints: usize,
}

/// [`TrafficLight`] over the fourteen shipped schemes at one depth.
pub fn traffic_light(tier: ColorDepth) -> TrafficLight {
    let mut declares = 0;
    let mut on_the_wire = 0;
    let mut distinct_paints = 0;
    for at in 0..Themes::standard().len() {
        let mut registry = Themes::standard();
        registry.select(at);
        registry.set_tier(tier);
        let theme = *registry.theme();
        if theme.shows(vitui_runtime::theme::Distinction::Status) {
            declares += 1;
        }
        if theme.roles_differ_on_wire(Role::Danger, Role::Warn)
            && theme.roles_differ_on_wire(Role::Warn, Role::Ok)
        {
            on_the_wire += 1;
        }
        let mut seen = std::collections::BTreeSet::new();
        for role in [Role::Danger, Role::Warn, Role::Ok] {
            seen.insert(format!("{:?}", theme.paint(role)));
        }
        if seen.len() == 3 {
            distinct_paints += 1;
        }
    }
    TrafficLight {
        tier,
        declares,
        on_the_wire,
        distinct_paints,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::ink::Direct;

    use std::collections::BTreeSet;
    use std::path::PathBuf;

    /// The application, which is the other half of this module.
    const APP: &str = "crates/vitui-apps/examples/gallery.rs";

    fn read(relative: &str) -> String {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(relative);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// **The two lists, compared.** [`crate::obligations::PANELS`] is written out and this table is
    /// the screen; an equality between two things derived from each other holds for ever, which is
    /// why one of them is a list and the other is the thing that draws.
    #[test]
    fn the_written_list_and_the_table_agree() {
        assert_eq!(crate::obligations::PANELS.to_vec(), panel_ids());
        assert_eq!(PANELS.len(), 28);
        // And no id twice, which a written list cannot check about itself: two panels for one row
        // satisfies both halves of O2 and shows twenty-seven components.
        let unique: BTreeSet<&str> = PANELS.iter().map(|p| p.id).collect();
        assert_eq!(unique.len(), PANELS.len());
    }

    /// **O2, both halves, over the twenty-eight built rows.**
    ///
    /// The queries are `crate::obligations`'s and are unchanged by this ticket — filling [`PANELS`]
    /// is the whole of what turning them green took, which is what *the evidence is an argument, not
    /// a file read* was for.
    #[test]
    fn both_halves_of_o2_are_met_over_the_twenty_eight_built_rows() {
        use crate::obligations::{
            self, Verdict, o2_everything_built_has_a_panel,
            o2_nothing_shown_is_absent_from_the_freeze,
        };
        assert_eq!(
            o2_nothing_shown_is_absent_from_the_freeze(obligations::PANELS),
            Verdict::Met { over: 28 }
        );
        assert_eq!(
            o2_everything_built_has_a_panel(obligations::PANELS),
            Verdict::Met { over: 28 }
        );
    }

    /// **The join, in both directions, and the one row that has no panel.**
    ///
    /// The freeze's order is the panel order, so the join is a zip rather than a search: a panel that
    /// drifted out of order would still satisfy both halves of O2 and would put `chart`'s drawing
    /// under `tree`'s title.
    #[test]
    fn every_panel_names_a_built_row_and_the_order_is_the_freezes() {
        let built: Vec<&str> = crate::INVENTORY
            .iter()
            .filter(|c| c.built)
            .map(|c| c.id)
            .collect();
        assert_eq!(built, panel_ids());
        let unbuilt: Vec<&str> = crate::INVENTORY
            .iter()
            .filter(|c| !c.built)
            .map(|c| c.id)
            .collect();
        assert_eq!(unbuilt, vec!["spinner"]);
        // The title is the id, so a reader looking at the screen and a gate reading the table are
        // looking at the same word.
        for panel in PANELS {
            assert_eq!(panel.id, panel.title, "a panel's title is its freeze id");
        }
    }

    /// **Every panel draws at least one cell of its own interior**, which is the drift neither half
    /// of O2 can see: an id in the table with an empty body satisfies both equalities and shows a
    /// frame with nothing in it.
    ///
    /// Read on the **interior** and not on the tile, because `panel_into` writes the frame and the
    /// title whatever the body does — so a count over the tile is a count of the panel this module
    /// drew, not of the component.
    #[test]
    fn every_panel_draws_at_least_one_cell_of_its_own_interior() {
        // Large enough that no panel is clipped, small enough to photograph twenty-eight times.
        let (w, h) = (44u16, 9u16);
        let mut empty = Vec::new();
        for panel in PANELS {
            let canvas = shot(panel.id, w, h, 3);
            let mut inside = 0;
            for y in 1..h - 1 {
                for x in 1..w - 1 {
                    if canvas.get(x, y).is_some() {
                        inside += 1;
                    }
                }
            }
            if inside == 0 {
                empty.push(panel.id);
            }
        }
        assert_eq!(
            empty,
            Vec::<&str>::new(),
            "these panels are a title over nothing"
        );
    }

    /// **The tiles tile the grid exactly**: no cell in two of them, no cell in none of them.
    ///
    /// Swept rather than asserted at one size, because the arithmetic that fails is the one that
    /// happens to be right wherever the width divides — `defective::tiles_by_multiplication` is that
    /// spelling and the arm below watches it leave a column behind.
    #[test]
    fn the_tiles_tile_the_grid_exactly_at_every_size() {
        let area = |w: u16, h: u16| Rect::new(0, 1, w, h);
        for w in 20..120u16 {
            for h in 6..30u16 {
                let (cols, rows, per) = grid(w, h + CHROME_ROWS);
                if per == 0 {
                    continue;
                }
                let mut seen = vec![0u8; usize::from(w) * usize::from(h)];
                let whole = area(w, h);
                for slot in 0..(cols as usize * rows as usize) {
                    let t = tile(whole, cols, rows, slot as u16);
                    for y in t.y..t.y + i32::from(t.h) {
                        for x in t.x..t.x + i32::from(t.w) {
                            let at = (y - 1) as usize * usize::from(w) + x as usize;
                            seen[at] += 1;
                        }
                    }
                }
                assert!(
                    seen.iter().all(|n| *n == 1),
                    "the grid at {w}x{h} is {cols}x{rows} and does not tile: {} cells covered twice, \
                     {} covered by nobody",
                    seen.iter().filter(|n| **n > 1).count(),
                    seen.iter().filter(|n| **n == 0).count()
                );
            }
        }
    }

    /// See [`the_tiles_tile_the_grid_exactly_at_every_size`]. **The spelling it replaced, watched
    /// leaving a column behind** — and watched agreeing at the one width that divides, which is why
    /// the sweep above is a sweep.
    #[test]
    fn tiles_by_multiplication_leave_the_remainder_to_nobody() {
        // **The last column, and not the written total.** `panel_into` hands its interior over and
        // does not write it (§2's second half), so a count over the whole surface is a count of
        // frames — which both arms draw. What the multiplication loses is the column past
        // `n * (w / n)`, and a panel's own right border is what would have been in it.
        let last_column = |width: u16, n: u16, multiplied: bool| -> usize {
            let mut driver = crate::runner::driver_at(width, 3, Density::default());
            let mut pen = crate::runner::Pen::new(width, 3);
            let area = Rect::new(0, 0, width, 3);
            driver.frame(|cx| {
                if multiplied {
                    defective::tiles_by_multiplication(&mut pen, cx, area, n);
                } else {
                    defective::tiles_under_one_id(&mut pen, cx, area, n);
                }
            });
            (0..3)
                .filter(|y| pen.canvas().get(width - 1, *y).is_some())
                .count()
        };
        // 100 columns into three: the cut reaches the last column and the multiplication stops one
        // short of it.
        assert_eq!(last_column(100, 3, false), 3);
        assert_eq!(last_column(100, 3, true), 0);
        // And at a width that divides, the two are indistinguishable — which is why the sweep above
        // is a sweep and this arm is one width of it.
        assert_eq!(last_column(99, 3, false), last_column(99, 3, true));
        assert_eq!(last_column(99, 3, true), 3);
    }

    /// **Every tile is its own widget**, and the spelling that made them one is watched merging.
    ///
    /// One call to `panel_into` in one loop is one `Location::caller()`, so the id is the same for
    /// every tile and `Ctx::interact` makes every claim after the first **inert** — a gallery whose
    /// panels stop hearing the pointer on a screen that renders perfectly. Components 30 found this
    /// in `player::chrome`; here it is the same defect in the loop that would have produced it.
    #[test]
    fn no_tile_shares_an_id_with_another() {
        let (w, h) = (100u16, 30u16);
        let shape = shape(w, h, 3);
        assert_eq!(shape.merges, 0, "tiles share an id");
        assert_eq!(shape.panels, 12);

        let mut driver = crate::runner::driver_at(w, 7, Density::default());
        let mut pen = crate::runner::Pen::new(w, 7);
        let area = Rect::new(0, 0, w, 7);
        driver.frame(|cx| defective::tiles_under_one_id(&mut pen, cx, area, 3));
        assert_eq!(
            driver.inspect().ids().merges(),
            2,
            "the spelling `with_key` replaced no longer merges, so the gate above is over nothing"
        );
    }

    /// **The keyboard alone reaches every panel**: the pages partition `0..28` at every size the
    /// gallery draws at.
    ///
    /// A count and not a walk, because *the walk keys cannot be printable characters* is O4's
    /// finding rather than this ticket's — `Ctrl+N`/`Ctrl+P` are chords and a focused `field` passes
    /// a chord through (components 38, `Bind::ignores`).
    #[test]
    fn the_pages_reach_every_panel_at_every_size() {
        for (w, h) in [(300u16, 80u16), (100, 30), (80, 24), (52, 16), (26, 9)] {
            let mut gallery = Gallery::new(Worker::queueing());
            let mut seen: Vec<&str> = Vec::new();
            for _ in 0..pages(w, h) {
                seen.extend(gallery.page_panels(w, h).iter().map(|p| p.id));
                gallery.next_page(w, h);
            }
            assert_eq!(
                seen,
                panel_ids(),
                "the pages at {w}x{h} do not cover the table"
            );
            // And the walk is a cycle: one more page returns to the first.
            assert_eq!(gallery.paging(w, h).0, 0);
        }
    }

    /// **A named panel is one page away**, which is what makes a gallery navigable by a person who
    /// knows what they are looking for rather than only by paging through.
    #[test]
    fn every_panel_can_be_gone_to_by_name() {
        let (w, h) = (100u16, 30u16);
        let mut gallery = Gallery::new(Worker::queueing());
        for panel in PANELS {
            assert!(gallery.go_to(panel.id, w, h));
            assert!(
                gallery.page_panels(w, h).iter().any(|p| p.id == panel.id),
                "{} is not on the page `go_to` chose",
                panel.id
            );
        }
        assert!(!gallery.go_to("spinner", w, h));
        assert!(!gallery.go_to("gauge", w, h));
    }

    /// **§16's matrix, nine cells, and the two axes move in different columns.**
    ///
    /// Register row 45 filed this `Unreachable { needs: "`ColorDepth`" }` with runtime architecture
    /// issue 22 — *the issue that lifted the barrier* — as its inverter, so the citation had been
    /// passing while meaning the opposite. `vitui_runtime::ColorDepth` is a name this crate can
    /// write and `Theme::resolve` is a call it can make.
    ///
    /// **The colour axis is measured on the theme and not on the screen**, and that is the finding
    /// rather than a shortcut: `Theme::resolve` returns the **same paint at every depth** for all
    /// thirteen roles — quantisation is the engine's, before the mirror — so the paint count reads
    /// **ten in all nine cells** and would read ten however much colour the terminal lost. Both
    /// columns are asserted, so a change that made the paint count move would fail here rather than
    /// quietly become the measurement.
    #[test]
    fn the_matrix_is_nine_cells_and_the_two_axes_move_in_different_columns() {
        let cells = matrix(100, 30);
        assert_eq!(cells.len(), 9);

        // The repertoire axis: three distinct cluster counts, and `Extended` is not `Unicode`.
        let clusters: BTreeSet<usize> = cells.iter().map(|c| c.clusters).collect();
        assert_eq!(clusters.len(), 3, "the repertoire axis does not move");
        for tier in [ColorDepth::None, ColorDepth::Ansi16, ColorDepth::TrueColor] {
            let at = |rung: GlyphSet| {
                cells
                    .iter()
                    .find(|c| c.rung == rung && c.tier == tier)
                    .expect("nine cells")
                    .clusters
            };
            assert!(at(RUNGS[0]) < at(RUNGS[1]));
            assert!(at(RUNGS[1]) < at(RUNGS[2]));
        }

        // The colour axis: the roles collapse and the distinctions go, and neither depends on the
        // repertoire — which is ADR 0032's two axes being two axes.
        for rung in RUNGS {
            let at = |tier: ColorDepth| {
                let c = cells
                    .iter()
                    .find(|c| c.rung == rung && c.tier == tier)
                    .expect("nine cells");
                (c.roles_collapsed, c.distinctions_lost, c.paints)
            };
            assert_eq!(at(ColorDepth::None), (66, 3, 10));
            assert_eq!(at(ColorDepth::Ansi16), (9, 2, 10));
            assert_eq!(at(ColorDepth::TrueColor), (0, 0, 10));
        }
    }

    /// **The traffic light is read on the wire, and the criterion does not reproduce.**
    ///
    /// The gallery's criterion is *`Danger`, `Warn` and `Ok` all quantise to bright white at sixteen
    /// colours*. Over the fourteen shipped schemes: **six of fourteen keep the three distinct at
    /// sixteen colours** and **none of fourteen does at `ColorDepth::None`** — so the sentence is
    /// true of eight schemes and of the wrong rung. Asserted as measured, on this map's own
    /// discipline, rather than bent to fit; the palette was deliberately not swapped to make the old
    /// number reappear.
    ///
    /// **And a count over `Paint` reads fourteen of fourteen at every depth**, including one with no
    /// colour at all. The cause is asserted below and it is not a coarse comparison: `Theme::resolve`
    /// hands back the **same paint** for every role at every depth, because quantisation is the
    /// engine's and happens before the mirror. §21's refinement 1 in a third place, and the reason
    /// [`TrafficLight`] carries three numbers instead of one.
    #[test]
    fn the_traffic_light_is_read_on_the_wire_and_a_paint_count_cannot_see_it() {
        let of = Themes::standard().len();
        assert_eq!(of, 14);
        let readings = [
            (ColorDepth::None, 0),
            (ColorDepth::Ansi16, 6),
            (ColorDepth::Indexed256, 14),
            (ColorDepth::TrueColor, 14),
        ];
        for (tier, on_the_wire) in readings {
            let t = traffic_light(tier);
            assert_eq!(t.on_the_wire, on_the_wire, "{tier:?} on the wire");
            // **The declared bit and the wire agree**, which is ADR 0032's own claim: a distinction
            // is one bit resolved at construction and not a guess a component makes at the draw.
            assert_eq!(t.declares, t.on_the_wire, "{tier:?} declares what it shows");
            assert_eq!(t.distinct_paints, 14, "{tier:?} paint equality is blind");
            // **And the reason, asserted rather than described.** `resolve` is what the colour axis
            // is, and it changes no paint a component can see.
            let base = *Themes::standard().theme();
            for role in vitui_runtime::theme::Role::ALL {
                assert_eq!(
                    base.resolve(tier).paint(role),
                    base.resolve(ColorDepth::TrueColor).paint(role),
                    "{role:?} at {tier:?}"
                );
            }
        }
    }

    /// **The words are the engine's own, read off `Debug`.**
    ///
    /// `GlyphSet::word` and `ColorDepth::word` are both `pub(crate)` in the engine, so a crate above
    /// it that prints a rung has to spell it — and a spelling nothing checks is a spelling that
    /// drifts. `crate::golden::tier` pins the same two ladders the same way, for the same reason.
    #[test]
    fn the_rung_and_tier_words_are_the_engines_own() {
        for (at, rung) in RUNGS.iter().enumerate() {
            assert_eq!(format!("{rung:?}").to_lowercase(), RUNG_WORDS[at]);
            assert_eq!(rung_word(*rung), RUNG_WORDS[at]);
        }
        for tier in [
            ColorDepth::None,
            ColorDepth::Ansi16,
            ColorDepth::Indexed256,
            ColorDepth::TrueColor,
        ] {
            assert_eq!(format!("{tier:?}").to_lowercase(), tier_word(tier));
        }
    }

    /// **The application iterates the table and mints no panel of its own.**
    ///
    /// A scan for what the file must **not** contain, so it cannot be satisfied by deleting
    /// anything — `crate::preview::mints_its_own_task`'s shape. An application free to list its own
    /// panels can drift from the freeze in a direction no equality over [`PANELS`] can see, which is
    /// the whole of why O2 is two equalities and not one.
    #[test]
    fn the_application_draws_the_table_and_mints_no_panel_of_its_own() {
        let source = read(APP);
        // It reaches the screen through the one entry point, and the panel table through the one
        // list. Both fragments are deletable while leaving the file compiling, which is what makes
        // the positive half worth asserting.
        assert!(source.contains("gallery::{self, Gallery, Sink}"), "{APP}");
        assert!(source.contains("gallery.ui_into("), "{APP}");
        // And it mints nothing: no `Panel {` of its own, no second `PANELS`, and none of the
        // twenty-eight components called directly.
        for forbidden in ["Panel {", "PANELS", "panel_into(", "collection_into("] {
            assert!(
                !source.contains(forbidden),
                "{APP} spells `{forbidden}`, so the gallery has two panel lists"
            );
        }
        // The negative scan is watched catching what it is for: the needles are real.
        assert!("let p = Panel { id: \"x\" };".contains("Panel {"));
        assert!("for p in PANELS {}".contains("PANELS"));
    }

    /// **Nothing in the gallery names crossterm, and `deny.toml`'s wrapper list is unchanged.**
    ///
    /// The ticket asks for the gallery to join ADR 0001's wrapper list *because it uses crossterm for
    /// raw mode, the alternate screen and input*. It uses none: `Driver::attach` enters raw mode and
    /// the alternate screen, reads the input and restores both. So the criterion is met by the
    /// exception being unnecessary, and the line that would have had to change says so.
    #[test]
    fn the_gallery_needs_no_crossterm_and_the_wrapper_list_is_unchanged() {
        // **Code, not prose.** The file's own header explains at length why it names no crossterm,
        // so a scan of the whole text finds the word in the sentence that says it is absent — which
        // is *a scanner looking for a literal contains that literal*, for the fourth time on this
        // map.
        let source = read(APP);
        let code: String = source
            .lines()
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(!code.contains("crossterm"), "{APP} names crossterm in code");
        assert!(
            source.contains("crossterm"),
            "{APP} no longer says why it needs none, so nothing tells the next reader"
        );
        let manifest = read("crates/vitui-apps/Cargo.toml");
        assert!(
            !manifest.contains("crossterm"),
            "vitui-apps names crossterm"
        );
        let deny = read("deny.toml");
        assert!(
            deny.contains(r#"{ name = "crossterm", wrappers = ["vitui-engine"] }"#),
            "ADR 0001's wrapper list has moved; the gallery is the reason it did not have to"
        );
    }

    /// **The pty gate runs this binary and checks an order, not a presence.**
    ///
    /// `scripts/gallery-panic-gate.sh` is what actually runs under a pty — a `cargo test` has no
    /// terminal to leave in a state — so what is checkable from here is that the script points at
    /// this binary, drives the flag that exists for it, and compares two byte **offsets**. A gate
    /// that only looked for the epilogue would pass on a capture where the backtrace came first,
    /// which is the one thing `shutdown.rs`'s ordering is for.
    #[test]
    fn the_panic_gate_runs_this_binary_and_checks_an_order() {
        let gate = read("scripts/gallery-panic-gate.sh");
        assert!(gate.contains("examples/gallery"), "it runs another binary");
        assert!(gate.contains("--panic"), "it does not drive the flag");
        // The order, and both halves of the three-way check.
        assert!(
            gate.contains(r"\[\?1049l"),
            "it does not look for the epilogue, or the `?` is unescaped — in an ERE `\\[?` is an \
             *optional* literal bracket, so the pattern reduces to `1049l` and the gate whose whole \
             subject is byte-exactness matches any capture containing that substring"
        );
        assert!(
            gate.contains(r#"[ "$alt_at" -lt "$panic_at" ]"#),
            "the gate checks a presence rather than an order"
        );
        // And the flag is really in the application, spelled the way the script drives it.
        let app = read(APP);
        assert!(app.contains(r#"a == "--panic""#), "{APP} has no `--panic`");
        assert!(
            app.contains("the panic gate's own panic"),
            "{APP}'s panic message is not the one the gate greps for"
        );
    }

    /// **The screen clears once and no frame after it does.**
    ///
    /// `crate::app::Clears` is the mechanism and the gallery is a caller of it; what this asserts is
    /// that the caller passes it a `Ctx` once a frame rather than at start-up, because the other half
    /// of the rule is *and on a resize*.
    #[test]
    fn the_screen_clears_once_and_again_only_on_a_resize() {
        let (w, h) = (60u16, 16u16);
        let mut driver = crate::runner::driver_at(w, h, Density::default());
        let mut gallery = Gallery::new(Worker::queueing());
        driver.set_theme(*gallery.theme());
        let mut cleared = 0;
        for _ in 0..4 {
            let before = gallery.cleared();
            let mut sink: Sink<'_> = &mut Direct;
            driver.frame(|cx| gallery.ui_into(&mut sink, cx, ""));
            cleared += usize::from(gallery.cleared() != before);
        }
        assert_eq!(cleared, 1, "the gallery clears more than once at one size");
    }

    /// **No size panics, and every key is pressable at every size.**
    ///
    /// A terminal smaller than one tile is a real state and it is the one the arithmetic breaks in:
    /// `grid` answers `(0, 0, 0)` there, and every consumer of it — `tile`'s two modulos,
    /// `pages`'s division, `go_to`'s and `next_page`'s — has to survive that rather than be
    /// unreachable. Swept over ninety-nine sizes from 1x1 up, with the page walk run one step past
    /// its own cycle and all four axis keys pressed at each.
    #[test]
    fn no_size_panics_and_every_key_is_pressable_at_every_size() {
        for w in [1u16, 2, 5, 10, 25, 26, 27, 51, 52, 100, 300] {
            for h in [1u16, 2, 3, 8, 9, 10, 16, 30, 80] {
                let mut driver = crate::runner::driver_at(w, h, Density::default());
                let mut gallery = Gallery::new(Worker::queueing());
                driver.set_theme(*gallery.theme());
                for _ in 0..3 {
                    gallery.bag.answer_queued();
                    let mut sink: Sink<'_> = &mut Direct;
                    driver.frame(|cx| gallery.ui_into(&mut sink, cx, "x"));
                }
                // One step past the cycle, because `% pages()` is where a zero would land.
                for _ in 0..=pages(w, h) {
                    gallery.next_page(w, h);
                    let mut sink: Sink<'_> = &mut Direct;
                    driver.frame(|cx| gallery.ui_into(&mut sink, cx, "x"));
                }
                gallery.prev_page(w, h);
                gallery.next_rung();
                gallery.next_tier();
                gallery.next_theme();
                gallery.prev_theme();
                assert!(gallery.go_to("plot", w, h));
                let mut sink: Sink<'_> = &mut Direct;
                driver.frame(|cx| gallery.ui_into(&mut sink, cx, "x"));
                // **A tile is never drawn where there is no room for one**, which is what makes the
                // count on the status bar the panels a person can see rather than the panels the
                // page holds.
                assert_eq!(
                    gallery.shown() == 0,
                    grid(w, h).2 == 0,
                    "at {w}x{h} the panels drawn and the room for them disagree"
                );
            }
        }
    }

    /// **A resize that shrinks the page count does not blank the screen.**
    ///
    /// `next_page` and `prev_page` wrap against the size they are handed, and nothing they do
    /// survives the terminal being made **larger**: `pages()` shrinks under them, and an index past
    /// the end yields an empty range rather than a clamp — a title and a status bar reading *0
    /// panels · page 4/1* over a grid nobody writes. A resize is the one event that changes the page
    /// count without going through a key, so it is the one case a per-size sweep over fresh
    /// galleries cannot reach.
    #[test]
    fn a_resize_that_shrinks_the_page_count_does_not_blank_the_screen() {
        let mut gallery = Gallery::new(Worker::queueing());
        // Four pages at 80x24, one at 200x60.
        assert_eq!(pages(80, 24), 4);
        assert_eq!(pages(200, 60), 1);
        for _ in 0..3 {
            gallery.next_page(80, 24);
        }
        assert_eq!(gallery.paging(80, 24), (3, 4));

        // The reader that does not draw agrees with the one that does.
        assert_eq!(gallery.paging(200, 60), (0, 1));
        assert_eq!(gallery.page_panels(200, 60).len(), PANELS.len());

        let mut driver = crate::runner::driver_at(200, 60, Density::default());
        driver.set_theme(*gallery.theme());
        for _ in 0..2 {
            gallery.bag.answer_queued();
            let mut sink: Sink<'_> = &mut Direct;
            driver.frame(|cx| gallery.ui_into(&mut sink, cx, ""));
        }
        assert_eq!(
            gallery.shown(),
            PANELS.len(),
            "the frame after a resize drew nothing"
        );
    }

    /// **What this ticket delivers rather than gates**, printed by `examples/gallery_numbers.rs` and
    /// asserted here only for the shape a later ticket has to move.
    ///
    /// Register rows 7 and 8 are pinned red with components 40 and 41 named. This asserts that both
    /// numbers are **non-trivial on this screen** — a screen where nothing is unwritten and nothing
    /// keeps a stale palette would leave those two tickets with no subject, and a screen where
    /// *everything* does would mean the gallery had stopped drawing.
    #[test]
    fn the_two_pinned_rows_have_a_subject_on_this_screen() {
        let (w, h) = (100u16, 30u16);
        let cells = usize::from(w) * usize::from(h);
        let shape = shape(w, h, 3);
        assert!(shape.unwritten > 0 && shape.unwritten < cells, "{shape:?}");
        assert_eq!(shape.writes, shape.distinct, "a cell is written twice");

        // **Read per axis, and `changed > 0` is not the reading.** A scheme change moves every
        // paint on the screen; a rung change moves clusters on the panels whose repertoire matters;
        // and a **tier change moves nothing on any panel at all** — the cells it moves are the two
        // chrome rows printing the depth's own name. That last row is the module's colour-axis
        // finding arriving as a count, and asserting `changed > 0` there would pass on the chrome
        // while saying nothing about the screen.
        //
        // **At two sizes, because one size agreeing with a law the other breaks is how a gate over
        // one size stays green** — which this arm was, until the chrome was two rows rather than one.
        for (w, h) in [(100u16, 30u16), (300, 80)] {
            for change in [Change::Scheme, Change::Rung] {
                let swap = swap(w, h, change);
                assert!(swap.written > 0, "{change:?} at {w}x{h}");
                assert!(
                    swap.changed_on_a_panel > 0,
                    "{change:?} at {w}x{h} moved no cell of any panel, so row 8 has nothing to \
                     measure"
                );
            }
            let tier = swap(w, h, Change::Tier);
            assert_eq!(
                tier.changed_on_a_panel, 0,
                "a colour depth moved a cell of a panel at {w}x{h}, so `Theme::resolve` has started \
                 handing components a quantised paint and the colour axis is on the canvas after all"
            );
            assert!(
                tier.changed > 0,
                "the chrome stopped printing the depth at {w}x{h}"
            );
        }
    }
}
