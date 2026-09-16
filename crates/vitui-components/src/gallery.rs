//! **O2's evidence as a value: one screen carrying every built row of the freeze, and the drift
//! gate is two equalities over the table it is drawn from.**
//!
//! > O2 — a panel in the gallery binary | **two** equalities: nothing shown may be absent from the
//! > freeze, and everything `built` must have a panel.
//!
//! The binary is `crates/vitui-apps/examples/gallery.rs`; what lives here is
//! the **screen**, and the reason it lives here rather than in the application is that two defects
//! have to be measured *on the assembled gallery* — the sentinel and
//! the palette swap (row 8), green since components 41 — and both are components tickets whose gate
//! is `cargo test`. A screen
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
//! is not a screen anybody can draw*. O1 chose `built`, O3 chose `built`, and
//! O2's second equality is **already written over `built`** in the spec, so nothing had to be
//! decided here — the population moves on its own the day `spinner` ships.
//!
//! # `'f` costs the table one field, and it is components 26's finding rather than a new one
//!
//! Two of the twenty-eight own an overlay — [`crate::input::select`] and
//! [`crate::files::file_picker`] — and both take `&'f mut` of the state their popup body captures
//! (the sentence about the fifth component). A table of `fn(&mut Bag, …)` pointers cannot
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
//! # The nine-cell matrix is reachable from this crate, and it used not to be
//!
//! Row 45 is `Unreachable { needs: "`ColorDepth`" }`, and its own `inverted_by` names runtime
//! the re-export — **what lifted the barrier**. `crates/vitui-runtime/src/line.rs`
//! now files `ColorDepth` as `reachable_as: Some("vitui_runtime::ColorDepth")`, so the colour axis
//! is a value this crate can hold and `Theme::resolve` is a call it can make. That is the near-miss
//! runtime 22 warned about arriving on this register: *a `Barrier` citation that still passes while
//! meaning the opposite*. The row inverts here, because this ticket's own criterion is that matrix.
//!
//! # Register rows 7 and 8 are both green here
//!
//! Components 39 delivered the screen and left both rows red with their figures printed on it;
//! **components 40 inverted row 7** — *every cell of the rectangle written at least once* — and
//! **the second row inverted later**. What the second half of the partition rule cost this screen is four numbers and
//! three mechanisms:
//!
//! | | cells at 300x80 | who owns it |
//! |---|---|---|
//! | `draws::panel` | 294 | `Frame::interior`: a `block` returns the rectangle it did not write |
//! | `draws::scrollbar` | 630 | this module's own narrowing — a bar is three columns of a wider tile |
//! | `draws::collapsible` | 432 | `Disclosure::used`: every row below the section is the caller's |
//! | two slots with no panel | 1 600 | the grid's: twenty-eight panels in a six-by-five grid |
//!
//! `draws::rest` is where the first three meet and `tiles_into` writes the fourth, and
//! [`Remainder`] keeps the spelling they replaced runnable so the gate can be watched failing. **The
//! other twenty-five panels are handed the whole of their tile's interior and fill it themselves**,
//! which is *a component handed a rectangle writes all of that* on the screen rather than in a
//! sentence — twenty-six of the twenty-eight already did, and the two that did not were `select` and
//! `file_picker`, whose remainder no `Response` could name.
//!
//! Components 40 measured that the two rows are **independent** here: see [`swap_on`] itself.
//!
//! # Row 8 is a count against an oracle, and the memo it is about is on this screen because there
//! was none
//!
//! *No cell keeps the previous palette a frame after a swap.* Neither the delta nor its complement
//! could have been that gate — `changed > 0` is green on the exact set it exists to catch, and
//! `kept` reads 17 884 of 24 000 under a repertoire change with nothing whatever wrong — so
//! [`swap_on`] plays a **second gallery** at the destination theme from its first frame and
//! [`Swap::stale`] is what the two disagree about.
//!
//! And [`crate::memos`] found the memo-key rule has **nothing in this crate subject to it**:
//! every shipped memo holds bits, floats or byte offsets, and every cluster and paint is derived
//! from the theme in front of the frame. So the memo is built here, as [`Keying`] — one axis, a
//! right arm and a wrong one, over the same twenty-eight call sites — which is [`Remainder`]'s
//! arrangement one enum over and for the same reason: *a gate that cannot fail is not a gate*.

use std::fmt::Write as _;

use vitui_runtime::ctx::Driver;
use vitui_runtime::data::Revision;
use vitui_runtime::layout::{Constraint, rect};
use vitui_runtime::work::{Cancel, Task, Worker};
use vitui_runtime::{
    ColorDepth, Ctx, Density, GlyphSet, Paint, Rect, Response, Role, Theme, Themes,
};

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
use crate::indicate::{
    MeterOpts, SparkOpts, SpinOpts, SpinState, meter_into, sparkline_into, spinner_into,
};
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

/// A decode is a free function over an identity — never a closure.
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
/// **Twenty-nine, which is every row of the freeze** now that `spinner` is built —
/// the population [`crate::obligations::o2_everything_built_has_a_panel`] asks about is `built`, and
/// it moved by itself the moment that column flipped.
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
        id: "spinner",
        title: "spinner",
        draw: draws::spinner,
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

/// **The one axis this screen can be false on that is neither a key nor a size**, as one value.
///
/// One field on [`Bag`] rather than a second tile loop in [`defective`], which is this crate's own
/// arrangement wherever a refused spelling has to stay runnable — `crate::collect::TreeShape`,
/// `crate::input::SelectShape`, `crate::disclose`'s `Shape`. Its reason is a reviewer's diff: the
/// shipped build and the refused one are **one line apart** and draw through the same twenty-eight
/// call sites, so a gate over the defect is a gate over the code that replaced it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Remainder {
    /// **The rule.** A rectangle a drawing did not write is written by the owner that handed it
    /// over: the second half, and `draws::rest` is where the three of them meet.
    #[default]
    Written,
    /// **The defect**, and the spelling components 40 replaced. `crate::app::Clears` writes every
    /// cell on the first frame and on a resize, so what is left alone here is left alone on a
    /// *steady* frame — and a cell nobody writes keeps what was already there, which on `Ctrl+N` is
    /// the panel that used to be in that tile. [`shape_as`], [`shot_as`], [`screen`] and [`swap_as`]
    /// take this arm, and `tests::the_remainder_left_alone_is_five_drawings_and_the_grids_own_slack`
    /// is the exact set.
    LeftAlone,
}

/// **What a memo over a panel's drawing is keyed on** — [`swap_on`]'s negative axis, and the one
/// thing in this crate whose value really is made of paints and glyphs.
///
/// # It is a fixture, and that it had to be is the finding
///
/// The rule is *a memo carries the theme in its key iff its value is made of paints or
/// glyphs*, and the account of the defect is six panels holding such a memo with the theme
/// left out of the key. **No memo in this crate holds one.** [`crate::memos`] is that reading as a
/// census: the three shipped ones hold a sub-cell bit grid, an axis domain and a wrap index, every
/// cluster and every paint is derived from the theme in front of it on the frame that draws it, and
/// the rule is therefore true here **vacuously**.
///
/// A rule that is true because nothing is subject to it is `Verdict::of`'s vacuity failure in the
/// shape this map keeps meeting, so the memo the rule is about is built here instead — as an axis
/// with a right arm and a wrong one, over the same twenty-eight call sites, which is
/// [`Remainder`]'s own arrangement one enum up. **It is what an application author reaches for
/// first**: a panel's drawing is expensive and its data did not move, so keep the verbs and replay
/// them.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Keying {
    /// **Shipped: there is no memo.** Every panel is drawn on every frame and every cluster and
    /// paint is derived from the theme the frame carries.
    #[default]
    Derived,
    /// **The rule.** The same memo, keyed with `Theme::memo_key` — the theme's own `Revision`
    /// folded with the data's, which is the runtime's own door for a caller with two revisions and
    /// one `Memo::get`.
    Themed,
    /// **The defect, and the plausible one.** `(data, tier)`: the colour depth is in the key and the
    /// theme is not, so it invalidates on `Ctrl+L` — the one axis that moves nothing on any panel
    /// — and is a **hit** on `t` and on `Ctrl+G`, which are the two that move everything.
    ///
    /// It is the named spelling, and its shape is why: an enumeration of axes is a list
    /// the next reader forgets one of, and the theme's own `Revision` is the one key that cannot be
    /// short.
    DataAndTier,
}

/// **How long one ladder frame of the gallery's spinner lasts.**
///
/// Eighty milliseconds is what a person reads as motion, and it is the figure
/// `vitui_runtime::anim::Steps`'s own documentation uses. On this screen it is also what makes the
/// spinner panel a **steady** frame under a pinned clock: nothing here advances time, so the ladder
/// index does not move and `crate::gallery::shape` and `crate::gallery::swap` measure a screen
/// rather than a cadence.
pub const SPIN_PER: std::time::Duration = std::time::Duration::from_millis(80);

/// **Everything the twenty-nine panels keep between frames.**
///
/// One value the caller owns, because the runtime has no retained structure: what
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
    spin: SpinState,
    pager: CollState,
    form: FormState,
    form_texts: [Text; 3],
    picker: PickerState,
    pane: PaneState<Doc>,
    pane_task: Task<Doc>,
    pane_key: u64,
    remainder: Remainder,
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
            // **Anchored at the epoch the caller hands it**, which is `Bag::new`'s and not a frame's
            // — a spinner's state is a function of `now`, so the panel's first frame computes the
            // same ladder index whatever frame it happens to be.
            spin: SpinState::new(),
            pager: CollState::new(),
            form: FormState::new(),
            form_texts: [Text::input(), Text::input(), Text::input()],
            picker,
            pane: PaneState::new(),
            pane_task,
            pane_key: 7,
            remainder: Remainder::Written,
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
    /// the top of the view and the view holds one `&mut Bag`.
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
/// and the second half is the whole subject of the sentinel gate.
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
    /// The per-tile memo, at [`Keying::Derived`] unless a gate asked otherwise — beside the bag for
    /// [`Gallery::select_popup`]'s reason, and inert on the shipped arm.
    panels: Panels,
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
            panels: Panels::new(),
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

    /// **Put the twenty-eight panels behind a memo**, at one of [`Keying`]'s two memoised arms.
    ///
    /// [`Keying::Derived`] is what an application gets and what every other gate in this crate runs
    /// through. The other two exist for [`swap_on`], which is the only caller: they are how *no
    /// cell carries the previous palette* is watched failing over the assembled screen rather than
    /// asserted about an arithmetic beside it.
    pub fn key_panels(&mut self, keying: Keying) {
        self.panels = Panels::new();
        self.panels.keying = keying;
    }

    /// The per-tile memo, for a report that wants its two counters.
    pub const fn panels(&self) -> &Panels {
        &self.panels
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

    /// The repertoire axis of the matrix.
    pub const fn rung(&self) -> GlyphSet {
        self.rung
    }

    /// The colour axis of the matrix.
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

    /// **What decides every cell of this screen, as one number.**
    ///
    /// The page, the scheme, the repertoire and the colour depth. It is what [`Clears::relaid_into`]
    /// compares, and it is a *value* rather than the runtime's `theme_changed` flag because a value
    /// that moved is a fact and a flag is a report of one. Density is the input it cannot carry —
    /// it is theme data and it changes rectangles — and [`Gallery::ui_into`] ORs the flag
    /// in for exactly that.
    fn screen_key(&self) -> u64 {
        let rung = RUNGS.iter().position(|r| *r == self.rung).unwrap_or(0) as u64;
        let tier = match self.tier {
            ColorDepth::None => 0,
            ColorDepth::Ansi16 => 1,
            ColorDepth::Indexed256 => 2,
            ColorDepth::TrueColor => 3,
        };
        (self.page as u64) << 8 | (self.themes.selected() as u64) << 4 | rung << 2 | tier
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

    /// The next of the three repertoires, wrapping. The first axis.
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

    /// The next of the three colour depths a human can tell apart, wrapping. The second axis.
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

    /// The density every theme in the set is built with: density is theme data.
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

        // **The preview pane's landing, taken at the top of the view**. `Task::take` is
        // destructive and a view has no `&mut` with which to put back what it took, so the landing
        // is a verb on the state before anything draws — an earlier pass measured what taking it
        // where a view happens to want it costs: **20 torn frames of 20**, on a screen whose state
        // ends up correct either way.
        let _ = self.bag.land_here();

        // **The screen is repainted when what decides every cell of it changes** — its size, its
        // page, or its theme.
        //
        // A resize is `Clears`'s own trigger and it is not the only one here: this screen pages
        // twenty-eight panels through twelve tiles, so `Ctrl+N` puts a **different component in the
        // same rectangle**. When this was written the second half was not met — 525 cells of 3 000
        // at 100x30 written by nobody, and on `Ctrl+N` that was a `radio` panel with a
        // `collection`'s rows still inside it.
        //
        // **This is not the partition rule and does not stand in for it.** It fires on the
        // transition frame only, so a steady frame is untouched — and since an earlier pass a steady
        // frame writes every cell of itself (a register row, 0 at every size and on every page),
        // which means residue has nowhere left to survive on *this* screen. What this closes is the
        // *sequence* half: `crate::app::Clears` is a guarantee `crate::app` makes to any caller, and
        // the frame it fires on is the one where the **terminal** rather than a component decided
        // what a cell held. `tests::a_carried_surface_after_a_change_equals_a_fresh_one` is what
        // would notice either half going away.
        //
        // The theme is in the key rather than read from `Ctx::theme_changed`, because a value that
        // moved is a fact and a flag is a report of one — and the density, which changes rectangles
        // and is theme data, is the one input the key cannot carry, so the flag is ORed in
        // for it.
        let relaid = self.clears.relaid_into(ink, cx, self.screen_key());
        if !relaid && cx.theme_changed() {
            crate::text::pad_rows(ink, cx, whole, cx.theme().paint(Role::Body));
        }

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
            panels,
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
            panels,
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
            panels,
            shown,
            ..
        } = self;
        let mut owners = Owners {
            select: Some(select_popup),
            picker: Some(picker_body),
            files: &FILES,
            task: picker_task,
        };
        *shown = tiles_into(bag, &mut owners, panels, ink, cx, whole, 1, 1, at..at + 1);
        true
    }
}

// ── the memo the rule is about, as an axis ───────────────────────────────────────────────────────

/// **One verb, kept.** [`Panels`]'s element, and what *a value made of paints and glyphs* is when it
/// is written out: a cluster or a string, a paint, and where they went.
///
/// `Ink::award` is not among them and does not need to be — it writes no cell of its own, and a
/// panel served from the memo is a panel that did not draw, so it declares nothing either. That is
/// the memo's real cost and it is left visible rather than papered over: the arm exists to be wrong
/// about the surface.
#[derive(Clone, Debug)]
enum Verb {
    Text {
        x: i32,
        y: i32,
        s: String,
        st: Paint,
    },
    Run {
        x: i32,
        y: i32,
        cluster: String,
        n: u16,
        st: Paint,
    },
    Pad {
        x: i32,
        y: i32,
        s: String,
        w: u16,
        st: Paint,
    },
}

/// **An [`Ink`] that keeps what it wrote and writes it anyway** — the miss arm of [`Panels`].
///
/// It forwards every verb, so a recorded frame draws exactly the frame that was not recorded: the
/// two arms of [`Keying`] differ on what the *next* frame does and on nothing else.
struct Recording<'a, I: Ink + ?Sized> {
    inner: &'a mut I,
    verbs: Vec<Verb>,
}

impl<I: Ink + ?Sized> Ink for Recording<'_, I> {
    fn text(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, s: &str, st: Paint) -> u16 {
        self.verbs.push(Verb::Text {
            x,
            y,
            s: s.to_owned(),
            st,
        });
        self.inner.text(cx, x, y, s, st)
    }

    fn run(
        &mut self,
        cx: &mut Ctx<'_, '_>,
        x: i32,
        y: i32,
        cluster: &str,
        n: u16,
        st: Paint,
    ) -> u16 {
        self.verbs.push(Verb::Run {
            x,
            y,
            cluster: cluster.to_owned(),
            n,
            st,
        });
        self.inner.run(cx, x, y, cluster, n, st)
    }

    fn pad_to(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, s: &str, w: u16, st: Paint) -> u16 {
        self.verbs.push(Verb::Pad {
            x,
            y,
            s: s.to_owned(),
            w,
            st,
        });
        self.inner.pad_to(cx, x, y, s, w, st)
    }

    fn award(&mut self, cx: &mut Ctx<'_, '_>, cells: Rect, resp: &Response, role: Role) {
        self.inner.award(cx, cells, resp, role);
    }
}

/// **A memo per tile over what that tile drew**, at whichever arm of [`Keying`] the caller chose.
///
/// It is beside [`Bag`] and not in it, for the reason `select_popup` is: the draw closure takes the
/// whole bag, so a field of the bag cannot be borrowed across it. [`Owners`] is the other value that
/// lives here for the same reason.
#[derive(Debug, Default)]
pub struct Panels {
    keying: Keying,
    /// Per slot: the key it was built at, and the verbs it drew.
    slots: Vec<(Revision, Vec<Verb>)>,
    hits: u32,
    misses: u32,
}

impl Panels {
    /// A memo that has drawn nothing, at the shipped arm.
    pub fn new() -> Panels {
        Panels::default()
    }

    /// How many tiles were served from the memo, over every frame. A **report**: a narrower key hits
    /// *more* often and is wrong, so this counter points the wrong way on the class of defect it
    /// looks like it would catch — `chart::raster::PlotState::misses`'s own note, one screen up.
    pub const fn hits(&self) -> u32 {
        self.hits
    }

    /// How many tiles were drawn, over every frame. See [`Panels::hits`].
    pub const fn misses(&self) -> u32 {
        self.misses
    }

    /// The key this arm folds for a theme, with the panel's own data revision.
    ///
    /// **The data is the same on both arms and the theme is the whole difference**, so a hit under
    /// [`Keying::DataAndTier`] is a hit for exactly one reason.
    fn key(&self, theme: &Theme, data: Revision) -> Revision {
        match self.keying {
            // `Derived` never asks for a key; it is folded in with the rule rather than given a
            // panic arm, because a branch a gate cannot exercise is a branch nothing checks.
            Keying::Derived | Keying::Themed => theme.memo_key(data),
            Keying::DataAndTier => {
                let tier = match theme.tier() {
                    ColorDepth::None => 1u64,
                    ColorDepth::Ansi16 => 2,
                    ColorDepth::Indexed256 => 3,
                    ColorDepth::TrueColor => 4,
                };
                Revision::from_raw(data.raw().wrapping_mul(31).wrapping_add(tier).max(1))
            }
        }
    }

    /// **A hit: write the tile's kept verbs and do not draw it.** `false` on a miss.
    fn replay<I: Ink + ?Sized>(
        &mut self,
        slot: usize,
        key: Revision,
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
    ) -> bool {
        let Some((at, verbs)) = self.slots.get(slot) else {
            return false;
        };
        if *at != key || verbs.is_empty() {
            return false;
        }
        for verb in verbs {
            match verb {
                Verb::Text { x, y, s, st } => {
                    let _ = ink.text(cx, *x, *y, s, *st);
                }
                Verb::Run {
                    x,
                    y,
                    cluster,
                    n,
                    st,
                } => {
                    let _ = ink.run(cx, *x, *y, cluster, *n, *st);
                }
                Verb::Pad { x, y, s, w, st } => {
                    let _ = ink.pad_to(cx, *x, *y, s, *w, *st);
                }
            }
        }
        self.hits += 1;
        true
    }

    /// Keep what the tile drew, at the key it was drawn under.
    fn store(&mut self, slot: usize, key: Revision, verbs: Vec<Verb>) {
        if self.slots.len() <= slot {
            self.slots
                .resize_with(slot + 1, || (Revision::UNKNOWN, Vec::new()));
        }
        self.slots[slot] = (key, verbs);
        self.misses += 1;
    }

    /// Which arm this memo is at.
    pub const fn keying(&self) -> Keying {
        self.keying
    }
}

/// **The gallery's panel data never moves**, so the data half of every key is a constant and the
/// theme half is the whole of the difference between [`Keying`]'s two memoised arms.
///
/// It is not [`Revision::UNKNOWN`], which `Theme::memo_key` preserves on purpose — *memoise
/// nothing* means nothing — so a screen keyed on it would never hit and both arms would be right.
const DATA: Revision = Revision::from_raw(1);

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
    panels: &mut Panels,
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
        // twenty-eight widgets under one `Id` — an earlier pass's `chrome` defect exactly, where
        // `Ctx::interact` makes a merged claim **inert** and every tile but the first stops hearing
        // the pointer on a screen that renders perfectly. `defective::tiles_under_one_id` is the
        // spelling this replaced and the gate watches it merging.
        // **The memo, and on the shipped arm there is not one.** [`Keying::Derived`] draws, which
        // is the branch every other gate in this crate runs through; the two memoised arms exist so
        // that *no cell carries the previous palette* is a gate that can be watched failing, over
        // the assembled screen and not over an arithmetic beside it.
        if panels.keying == Keying::Derived {
            cx.with_key(slot as u64, |cx| {
                let frame = panel_into(&mut *ink, cx, here, panel.title, "", &opts);
                let mut sink: Sink<'_> = ink;
                (panel.draw)(bag, owners, &mut sink, cx, frame.interior);
            });
        } else {
            let key = panels.key(cx.theme(), DATA);
            if panels.replay(slot, key, ink, cx) {
                drawn += 1;
                continue;
            }
            let mut rec = Recording {
                inner: &mut *ink,
                verbs: Vec::new(),
            };
            cx.with_key(slot as u64, |cx| {
                let frame = panel_into(&mut rec, cx, here, panel.title, "", &opts);
                let mut sink: Sink<'_> = &mut rec;
                (panel.draw)(bag, owners, &mut sink, cx, frame.interior);
            });
            panels.store(slot, key, rec.verbs);
        }
        drawn += 1;
    }
    // **The slots with no panel in them, which are the grid's own remainder.**
    //
    // `grid` answers `cols x rows` and the last page is short: twenty-eight panels in a six-by-five
    // grid leave **two tiles of 50x16 — 1 600 cells — that no panel is ever drawn into**, and
    // `tests::the_tiles_tile_the_grid_exactly_at_every_size` proves they are inside the rectangle
    // this loop was handed rather than outside it. A cell nobody writes keeps what was there, and on
    // `Ctrl+N` what was there is the previous page's panel; this is the steady-frame half of the same
    // sentence `crate::app::Clears` closes for the transition frame (the design, a register row).
    if bag.remainder == Remainder::Written {
        for slot in drawn..(cols as usize * rows as usize) {
            let here = tile(area, cols, rows, slot as u16);
            if here.is_empty() {
                continue;
            }
            let paint = cx.theme().paint(Role::Body);
            crate::text::pad_rows(ink, cx, here, paint);
        }
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
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let frame = panel_into(ink, cx, r, "General", "", &PanelOpts::default());
        // **A `block` returns the rectangle it did not write**, and the owner writes it.
        // This is the second half in the one shape the rule states outright, and the panel inside a
        // panel is where the gallery meets it: 294 cells of a 50x15 tile.
        rest(b, ink, cx, frame.interior);
    }

    pub fn chip(
        _b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let _ = chip_into(ink, cx, r, "draft", &ChipOpts::default());
    }

    pub fn button(
        _b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let _ = button_into(ink, cx, r, "Save", &ButtonOpts::default());
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
    /// is measured by `crate::popup`, because an overlay body takes no ink.
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
            r,
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
        b: &mut Bag,
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
        // **The narrowing is this module's, so the remainder is too.** A bar is three columns wide
        // whatever it is offered — `scrollbar_into` writes every cell of the rectangle it is handed
        // — so the forty-five columns beside it are the gallery's, and nobody wrote them.
        rest(
            b,
            ink,
            cx,
            Rect::new(
                bar.x + i32::from(bar.w),
                r.y,
                r.w.saturating_sub(bar.w),
                r.h,
            ),
        );
    }

    /// **The band, which is all `sticky` is**: the header shares `x` with a body scrolled
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
        let shown = collapsible_into(
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
        // **`Disclosure::used` is the mechanism and this is its one caller on the screen**: a
        // section occupies its header plus its body's height, *every row below is the caller's, and
        // this is how it is named*. A caller that reads its tail from the height the section had
        // before a collapse leaves 240 cells of the old body standing; a caller that reads it
        // and does nothing leaves them unwritten, which is the same cells and the other defect.
        rest(
            b,
            ink,
            cx,
            Rect::new(
                r.x,
                r.y + i32::from(shown.used),
                r.w,
                r.h.saturating_sub(shown.used),
            ),
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
        let _ = toggle_into(ink, cx, r, "enabled", &mut b.checkbox, &opts);
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
        let _ = toggle_into(ink, cx, r, "enabled", &mut b.radio, &opts);
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
        let _ = toggle_into(ink, cx, r, "enabled", &mut b.switch, &opts);
    }

    /// **One row of a tall tile, and the two beside it are this module's** — [`band`]'s reason,
    /// which `meter` is the second subject of.
    pub fn meter(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let track = band(b, ink, cx, r);
        let _ = meter_into(ink, cx, track, b.meter, &MeterOpts::default());
    }

    pub fn spinner(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        // **Started from the frame's own clock**, which is the only clock a component may read —
        // and started on every frame rather than once, because a `Steps` re-anchored at `now` and a
        // `Steps` anchored an hour ago answer the same question at the same `now`. What that costs
        // is the ladder index being the same on every frame of a *pinned* clock, which is what a
        // golden and a swap gate both want.
        b.spin.start(cx.now(), SPIN_PER);
        let _ = spinner_into(ink, cx, r, &b.spin, "working", &SpinOpts::default());
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
        let _ = rule_into(ink, cx, r, " limits ", &RuleOpts::default());
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
            r,
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
        let _ = pagination_into(ink, cx, r, &mut b.pager, 9, &PageOpts::default());
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

    /// **One row of a tall tile, and the two beside it are this module's** — see [`band`].
    pub fn slider(
        b: &mut Bag,
        _o: &mut Owners<'_>,
        ink: &mut Sink<'_>,
        cx: &mut Ctx<'_, '_>,
        r: Rect,
    ) {
        let track = band(b, ink, cx, r);
        let _ = slider_into(ink, cx, track, &mut b.slider, &SliderOpts::default());
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
            r,
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

    /// **The one row a band component is for, out of however tall a tile the grid handed over.**
    ///
    /// [`meter`] and [`slider`] draw through `crate::scroll::stripe`, which fills the **cross** axis
    /// of the rectangle it is given: a horizontal meter in an eighteen-row tile is an eighteen-row
    /// meter, and a horizontal slider is an eighteen-row track under an eighteen-row thumb. That is
    /// the component being right — a caller brings the rectangle, and a chunky slider is a slider
    /// somebody asked for — and it is this module being wrong, because the tile is the gallery's
    /// arithmetic and not a size anyone chose. `crate::golden`'s own screens play the two at
    /// `16x1` and `20x1`, which is the size a reviewer is shown, and the published picture of each
    /// panel was showing eighteen — one instrument's rectangle disagreeing with another's, with
    /// only the picture on the side a reader meets.
    ///
    /// Centred rather than top-aligned, because that is where every other one-row panel of the
    /// gallery sits — `rule`, `button`, `switch` and `chip` all centre **inside** the component,
    /// pad the rows either side and write every cell. A band cannot: it has no cross-axis padding
    /// to do it with. So the narrowing is the caller's, which makes the two remainders the
    /// caller's too — [`scrollbar`]'s arrangement one axis over, and [`rest`] is where they go.
    fn band(b: &Bag, ink: &mut Sink<'_>, cx: &mut Ctx<'_, '_>, r: Rect) -> Rect {
        if r.h <= 1 {
            return r;
        }
        let (above, below) = rect::split_at_v(r, (r.h - 1) / 2);
        let (track, under) = rect::split_at_v(below, 1);
        rest(b, ink, cx, above);
        rest(b, ink, cx, under);
        track
    }

    /// **The rectangle a drawing was handed and did not write, written by its owner.**
    ///
    /// Five of the twenty-nine hand part of their tile back, each by the mechanism the rule
    /// names — [`panel`] gets `Frame::interior`, [`collapsible`] gets `Disclosure::used`,
    /// [`scrollbar`] is handed a three-column bar out of a wider rectangle by *this* module, and
    /// [`meter`] and [`slider`] are handed one row out of a taller one by [`band`]. The remainder
    /// is the owner's, and the owner is here.
    ///
    /// `Role::Body` and not the panel's face, because what is left of a panel's interior is the
    /// panel's background — the same paint the one clear at the top of the frame writes.
    fn rest(b: &Bag, ink: &mut Sink<'_>, cx: &mut Ctx<'_, '_>, rest: Rect) {
        if b.remainder == Remainder::LeftAlone {
            return;
        }
        let paint = cx.theme().paint(Role::Body);
        crate::text::pad_rows(ink, cx, rest, paint);
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
    /// **Cells no verb wrote on the frame measured.** The sentinel gate, **green since
    /// 40** — `tests::no_cell_of_the_assembled_gallery_is_written_by_nobody` is the sweep and
    /// [`Remainder::LeftAlone`] is the arm it is watched failing on.
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
    shape_as(w, h, frames, Remainder::Written)
}

/// [`shape`], at either arm of [`Remainder`]. The defect's own numbers come out of here.
pub fn shape_as(w: u16, h: u16, frames: u32, remainder: Remainder) -> Shape {
    let (gallery, driver, pen) = play(w, h, 0, frames, remainder);
    read_shape(&gallery, &driver, &pen, w, h)
}

/// [`shape_as`], on one page of the twenty-eight.
///
/// **A page is a different set of drawings in the same rectangles**, so the second half is a
/// different claim on each of them — and the two tiles with no panel in them exist on the last page
/// of a grid that does not divide and nowhere else.
///
/// Three frames, which is the warm-up the two file components need: the answer arrives on a frame
/// after the one that asked.
pub fn shape_on(w: u16, h: u16, page: usize, remainder: Remainder) -> Shape {
    let (gallery, driver, pen) = play(w, h, page, 3, remainder);
    read_shape(&gallery, &driver, &pen, w, h)
}

/// The eight counters of one recorded frame.
fn read_shape(
    gallery: &Gallery,
    driver: &Driver,
    pen: &crate::runner::Pen,
    w: u16,
    h: u16,
) -> Shape {
    let cells = usize::from(w) * usize::from(h);
    Shape {
        panels: gallery.shown(),
        writes: pen.tally().writes(),
        distinct: pen.tally().distinct(),
        verbs: pen.tally().verbs(),
        regions: driver.inspect().hits().len(),
        stops: driver.inspect().stop_count(),
        merges: driver.inspect().ids().merges(),
        unwritten: cells - pen.canvas().written(),
    }
}

/// **The recorded surface [`shape`] counts** — for a gate that has to say *where* the cells nobody
/// wrote are, and not only how many there were.
///
/// The failing set was a table of six panels, and a total cannot be checked against
/// one: `tests::the_remainder_left_alone_is_five_drawings_and_the_grids_own_slack` attributes every
/// cell of it to the tile it is in.
pub fn screen(w: u16, h: u16, frames: u32, remainder: Remainder) -> crate::runner::Canvas {
    play(w, h, 0, frames, remainder).2.into_canvas()
}

/// **Warmed, then measured on a fresh recorder.**
///
/// The warm frames are what the two file components need — the answer arrives on a frame after the
/// one that asked — and the fresh recorder is what makes [`Shape::unwritten`] a **steady
/// frame's** number: `crate::app::Clears` writes every cell of the screen on the first frame and on
/// a resize, so a recorder carried across that frame answers *nobody ever left a cell alone*, which
/// is zero on any screen that clears and says nothing about any component.
fn play(
    w: u16,
    h: u16,
    page: usize,
    frames: u32,
    remainder: Remainder,
) -> (Gallery, Driver, crate::runner::Pen) {
    let mut driver = crate::runner::driver_at(w, h, Density::default());
    let mut gallery = Gallery::new(Worker::queueing());
    gallery.bag.remainder = remainder;
    for _ in 0..page {
        gallery.next_page(w, h);
    }
    driver.set_theme(*gallery.theme());
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
    (gallery, driver, pen)
}

/// **One named panel, photographed alone over `w x h`.**
///
/// `frames` is the warm-up: two is what a gallery needs for the hover index and the ring, and three
/// is what the preview pane needs — the answer arrives on a frame after the one that asked (spec
/// the media family's own note).
pub fn shot(id: &str, w: u16, h: u16, frames: u32) -> crate::runner::Canvas {
    shot_as(id, w, h, frames, Remainder::Written)
}

/// [`shot`], at either arm of [`Remainder`] — which is how the per-panel half of the gate is
/// watched failing on the drawing that owns each cell of it.
pub fn shot_as(
    id: &str,
    w: u16,
    h: u16,
    frames: u32,
    remainder: Remainder,
) -> crate::runner::Canvas {
    let mut driver = crate::runner::driver_at(w, h, Density::default());
    let mut gallery = Gallery::new(Worker::queueing());
    gallery.bag.remainder = remainder;
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

/// **One named panel, as a picture rather than as a canvas.**
///
/// [`shot`]'s twin, and the difference is the ink: a `Pen` records what a component asked for and
/// this draws through [`crate::ink::Direct`], so the cells reach the engine, composite, and come
/// back as the frame's own SVG. What that buys is the sentence a picture of a terminal usually
/// cannot make — this is the frame, not a photograph of one — and it costs nothing in drift,
/// because the panel is the same call to [`Gallery::one_into`] the screen makes.
///
/// `frames` is the warm-up, for [`shot`]'s reason: the hover index and the ring settle on the
/// second, and the preview pane's answer arrives on a third.
///
/// # Panics
///
/// If no panel has that id — which is the same refusal `shot` makes, and it is how a renamed
/// freeze row fails here rather than quietly producing a picture of nothing.
pub fn picture(id: &str, w: u16, h: u16, frames: u32) -> String {
    let mut driver = crate::runner::driver_at(w, h, Density::default());
    let mut gallery = Gallery::new(Worker::queueing());
    driver.set_theme(*gallery.theme());
    let mut found = false;
    for _ in 0..frames.max(1) {
        gallery.bag.answer_queued();
        let mut direct = crate::ink::Direct;
        let mut sink: Sink<'_> = &mut direct;
        driver.frame(|cx| found = gallery.one_into(&mut sink, cx, id));
    }
    assert!(found, "no panel is named {id}");
    driver.to_svg()
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
    /// **Cells whose value did not move**, which is `written - changed` and nothing more.
    ///
    /// **It is on the wrong side of the question and it is printed, never gated**, which is
    /// first refinement in the one place this module can demonstrate it: most of the cells a swap
    /// leaves alone are left alone *correctly*. A rung change moves the cells drawn from the theme's
    /// glyph table and no others, so the letters of every label are `kept` and right; the colour
    /// axis moves nothing on any panel at all, so `kept` reads 100% on a screen with
    /// nothing wrong with it. The swap gate's subject is [`Swap::stale`], which is a different
    /// number measured against a different arm.
    pub kept: usize,
    /// **Cells that carry the previous theme's value a frame after the swap.** The swap gate's
    /// subject, and the gate.
    ///
    /// It is a count over the surface against an **oracle** and not a threshold on a delta: the
    /// same gallery, at the same page, driven the same number of frames with the destination theme
    /// in place from the first one. A cell the two arms disagree about is a cell the swap did not
    /// reach — a memo that did not invalidate, or residue nothing rewrote — and *no cell carries the
    /// previous palette* is `stale == 0`.
    ///
    /// **The complement form is what `changed > 0` could not be.** A screen where the swap reached
    /// nothing and a screen where it had nothing to reach are the same delta and different oracles.
    pub stale: usize,
    /// Of [`Swap::stale`], how many are on a panel — [`Swap::changed_on_a_panel`]'s exclusion, for
    /// the same reason and over both chrome rows.
    pub stale_on_a_panel: usize,
    /// [`Swap::stale`] as **cells over rows**, which is the form every defect on this map was
    /// legible in: 3 583 cells over 30 rows is a screen, 3 583 over 6 is six panels.
    pub divergence: crate::runner::Diff,
    /// **Tiles served from the memo over the four frames**, which is `0` on the shipped arm and is
    /// the anti-vacuity number on the other two: a memo that never hits is a memo that cannot be
    /// stale, so a `Keying::Themed` reading `stale == 0` says nothing until this is above zero.
    pub served: u32,
    /// Tiles drawn over the four frames. On [`Keying::Derived`] it is `0` — nothing is counted
    /// because nothing is memoised — and on the other two it is the misses.
    pub drew: u32,
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
/// The screen the swap gate is measured on, and green. [`Swap::stale`] is the
/// gate; [`Swap::changed`] and [`Swap::kept`] are the delta, printed and never gated, because both
/// are on the wrong side of the question — see [`swap_on`].
pub fn swap(w: u16, h: u16, change: Change) -> Swap {
    swap_as(w, h, change, Remainder::Written)
}

/// [`swap`], at either arm of [`Remainder`] — which is how *the two rows are independent* stops
/// being a sentence. The two rows are pinned together (*the swap excess equal to it on five of six*)
/// and on this screen closing the first moves nothing in the second: the surface is carried across
/// the change and the first frame clears, so a cell nobody writes on a steady frame is inside
/// `written` already.
pub fn swap_as(w: u16, h: u16, change: Change, remainder: Remainder) -> Swap {
    swap_on(w, h, 0, change, remainder, Keying::Derived)
}

/// [`swap_as`], on one page of the twenty-eight — [`shape_on`]'s argument, for [`shape_on`]'s
/// reason: a page is a different set of drawings in the same rectangles, so *no cell carries the
/// previous palette* is a different claim on each of them.
///
/// # The oracle, and why the gate could not be a delta
///
/// Two arms are played. **The swapped arm** is what a terminal does: one surface, three frames at
/// the old theme, the key, one frame more. **The reference arm** is the same gallery on the same
/// page driven the same four frames with the destination theme in place from the first — the screen
/// the swap was *supposed* to produce.
///
/// [`Swap::stale`] is the cells the two disagree about. It is the engine's own arrangement one
/// crate down (`reference.rs`: the obviously-correct, far-too-slow compositor, from which the
/// expectation is **generated** rather than hand-written, *because a hand-written expectation about
/// damage is written by the person who wrote the damage*), and it is the only spelling that can
/// tell **the swap reached nothing** from **the swap had nothing to reach** — which a delta cannot,
/// because on this screen those two are the same number on two of the three axes.
pub fn swap_on(
    w: u16,
    h: u16,
    page: usize,
    change: Change,
    remainder: Remainder,
    keying: Keying,
) -> Swap {
    // **One surface, carried across the change, which is what a terminal is.**
    //
    // This function rendered the *after* picture onto a **fresh** `Pen` for one commit, and every
    // gate over it was green while `Ctrl+N` put the previous page inside the new page's frames on a
    // real screen. A recorder that starts blank cannot see residue — which is `crate::golden`'s own
    // note about a multi-frame shot, in as many words — and residue is the entire subject of
    // register rows 7 and 8.
    //
    // **And it is why `kept` never depended on row 7**, which the design states the other way round (*the
    // swap excess equal to it on five of six*, on a prototype's gallery). The surface is carried and
    // the first frame **clears**, so a cell nobody writes on a steady frame is still a cell somebody
    // wrote once: it is inside `written` and it counts as `kept`. An earlier pass turned every one of
    // those cells into a pad and `kept` at 100x30 did not move by one — 2 005 of 3 000 under a rung
    // change either way. `stale` is measured against the reference arm instead, and that arm carries
    // its own surface across its own four frames, so residue is compared with residue.
    let mut swapped = Played::open(w, h, page, remainder, keying);
    for _ in 0..3 {
        swapped.frame();
    }
    let before = read(&swapped.pen, w, h);
    swapped.apply(change);
    swapped.frame();
    let after = read(&swapped.pen, w, h);

    // The reference arm: the destination theme in place before the first frame, so nothing on it has
    // ever seen the one being left behind.
    // **The reference arm is [`Keying::Derived`] whatever the swapped arm is**, and it has to be:
    // the oracle is *the screen the destination theme produces*, and a memo is a thing the screen
    // under test does. Handing the reference the same memo would compare a stale picture with a
    // picture that never had a chance to go stale — and both arms would agree, which is the vacuity
    // this whole ticket is about arriving one level in.
    let mut reference = Played::open(w, h, page, remainder, Keying::Derived);
    reference.apply(change);
    for _ in 0..4 {
        reference.frame();
    }
    let fresh = read(&reference.pen, w, h);

    let on_a_panel = |i: usize| {
        let y = (i / usize::from(w)) as u16;
        y > 0 && y + 1 < h
    };

    let mut changed = 0;
    let mut changed_on_a_panel = 0;
    let mut kept = 0;
    let mut written = 0;
    for (i, (a, b)) in before.iter().zip(after.iter()).enumerate() {
        match (a, b) {
            (Some(a), Some(b)) => {
                written += 1;
                // **What *the value moved* means depends on which key was pressed**, and reading one
                // axis for the other is how a delta over a swap goes green: a scheme change moves
                // paints and not clusters, and a rung change moves clusters and not paints.
                let same = match change {
                    Change::Scheme | Change::Tier => a.1 == b.1,
                    Change::Rung => a.0 == b.0,
                };
                if same {
                    kept += 1;
                } else {
                    changed += 1;
                    if on_a_panel(i) {
                        changed_on_a_panel += 1;
                    }
                }
            }
            (_, Some(_)) => written += 1,
            _ => {}
        }
    }

    // **The whole cell and not one axis of it.** `kept` reads the axis the key is about, because a
    // rung change leaving a paint alone is not a finding; `stale` reads both, because a cell that
    // disagrees with the reference arm disagrees about something the destination theme decides, and
    // which axis it is on is the report's business rather than the gate's.
    let mut stale = 0;
    let mut stale_on_a_panel = 0;
    let mut rows = 0usize;
    let mut first = None;
    for y in 0..h {
        let mut row_differs = false;
        for x in 0..w {
            let i = usize::from(y) * usize::from(w) + usize::from(x);
            if after[i] != fresh[i] {
                stale += 1;
                row_differs = true;
                if on_a_panel(i) {
                    stale_on_a_panel += 1;
                }
                if first.is_none() {
                    first = Some((x, y));
                }
            }
        }
        if row_differs {
            rows += 1;
        }
    }

    Swap {
        change,
        changed,
        kept,
        served: swapped.gallery.panels().hits(),
        drew: swapped.gallery.panels().misses(),
        stale,
        stale_on_a_panel,
        divergence: crate::runner::Diff {
            cells: stale,
            rows,
            first,
            over: (w, h),
        },
        written,
        changed_on_a_panel,
    }
}

/// **A gallery, its driver and the one surface all its frames land on** — [`swap_on`]'s two arms, so
/// that *the same gallery, on the same page, driven the same number of frames* is one piece of code
/// rather than two that have to be read against each other.
struct Played {
    gallery: Gallery,
    driver: Driver,
    pen: crate::runner::Pen,
}

impl Played {
    /// A gallery at `page`, at the registry's first theme, with nothing drawn yet.
    fn open(w: u16, h: u16, page: usize, remainder: Remainder, keying: Keying) -> Played {
        let mut driver = crate::runner::driver_at(w, h, Density::default());
        let mut gallery = Gallery::new(Worker::queueing());
        gallery.bag.remainder = remainder;
        gallery.key_panels(keying);
        for _ in 0..page {
            gallery.next_page(w, h);
        }
        driver.set_theme(*gallery.theme());
        Played {
            gallery,
            driver,
            pen: crate::runner::Pen::new(w, h),
        }
    }

    /// One frame, onto the surface every other frame landed on.
    fn frame(&mut self) {
        self.gallery.bag.answer_queued();
        self.pen.end_frame();
        let Played {
            gallery,
            driver,
            pen,
        } = self;
        let mut sink: Sink<'_> = pen;
        driver.frame(|cx| gallery.ui_into(&mut sink, cx, ""));
    }

    /// Press the key, and hand the driver what the gallery answers.
    ///
    /// **The theme reaches the driver between frames and cannot do better** — [`Gallery::theme`]'s
    /// own note — so this is the whole of what a swap is on the app thread, in both arms.
    fn apply(&mut self, change: Change) {
        match change {
            Change::Scheme => self.gallery.next_theme(),
            Change::Rung => self.gallery.next_rung(),
            Change::Tier => self.gallery.next_tier(),
        }
        self.driver.set_theme(*self.gallery.theme());
    }
}

/// Every cell of a recorded surface as `(cluster, paint)`, or `None` where nobody has written.
fn read(pen: &crate::runner::Pen, w: u16, h: u16) -> Vec<Option<(String, String)>> {
    let mut out = Vec::with_capacity(usize::from(w) * usize::from(h));
    for y in 0..h {
        for x in 0..w {
            out.push(
                pen.canvas()
                    .get(x, y)
                    .map(|c| (c.cluster.clone(), format!("{:?}", c.paint))),
            );
        }
    }
    out
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
            let _ = panel_into(ink, cx, here, "tile", "", &opts);
        }
    }

    /// **Tiles laid out by multiplication instead of by the boundary arithmetic [`tile`] uses**, which
    /// leaves the remainder
    /// of a width that does not divide by the column count written by nobody. It is the gap between
    /// tiles the second half is about, and it is invisible at every size that happens to divide.
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
            let _ = panel_into(ink, cx, here, "tile", "", &opts);
        }
    }
}

// ── the nine cells ─────────────────────────────────────────────────────────────────────────────

/// One cell of the matrix: the repertoire axis against the colour axis, on this screen.
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
    /// So *the colour axis of the matrix is not observable on a canvas above the engine at all*,
    /// and that is the role rule working rather than a hole: a component is handed the same paint whatever
    /// the terminal can show, which is what *a component names a role and never a colour* means. The
    /// column is reported so that the nine rows show the axis moving in
    /// [`MatrixCell::roles_collapsed`] and standing still here — the refinement 1, *a counter on
    /// the wrong side of the question is not a weak gate, it is a green one*, as a table.
    pub paints: usize,
    /// **Pairs of the thirteen roles that are one colour on the wire.** The colour axis, measured on
    /// the theme because it is not measurable on the screen.
    pub roles_collapsed: usize,
    /// **How many of the nine distinctions the theme does not show.** The colour axis again, at the
    /// level a component actually asks about.
    pub distinctions_lost: usize,
}

/// **The three repertoires against the three colour depths, over this screen.**
///
/// The matrix is *a count, not nine screenshots*, and this is where the count is taken over an
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
        assert_eq!(PANELS.len(), 29);
        // And no id twice, which a written list cannot check about itself: two panels for one row
        // satisfies both halves of O2 and shows twenty-seven components.
        let unique: BTreeSet<&str> = PANELS.iter().map(|p| p.id).collect();
        assert_eq!(unique.len(), PANELS.len());
    }

    /// **O2, both halves, over the twenty-nine built rows — which is every row of the freeze.**
    ///
    /// The queries are `crate::obligations`'s and are unchanged by this ticket — filling [`PANELS`]
    /// is the whole of what turning them green took, which is what *the evidence is an argument, not
    /// a file read* was for.
    #[test]
    fn both_halves_of_o2_are_met_over_the_twenty_nine_built_rows() {
        use crate::obligations::{
            self, Verdict, o2_everything_built_has_a_panel,
            o2_nothing_shown_is_absent_from_the_freeze,
        };
        assert_eq!(
            o2_nothing_shown_is_absent_from_the_freeze(obligations::PANELS),
            Verdict::Met { over: 29 }
        );
        assert_eq!(
            o2_everything_built_has_a_panel(obligations::PANELS),
            Verdict::Met { over: 29 }
        );
    }

    /// **The join, in both directions, and there is no longer a row without a panel.**
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
        // **Zero, and it was `["spinner"]` until an earlier pass.** The direction this still
        // watches is a row arriving unbuilt — a thirtieth component — with the panel table already
        // naming it, which is the drift `o2_everything_built_has_a_panel` cannot see from its side.
        let unbuilt: Vec<&str> = crate::INVENTORY
            .iter()
            .filter(|c| !c.built)
            .map(|c| c.id)
            .collect();
        assert_eq!(unbuilt, Vec::<&str>::new());
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
        // does not write it (the second half), so a count over the whole surface is a count of
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
        // A name the freeze has never heard of. `spinner` was the other arm here until components
        // an earlier pass built it, and it is above now — the sweep is over `PANELS`, so the negative
        // case had to become one nothing can turn.
        assert!(!gallery.go_to("gauge", w, h));
    }

    /// **The matrix, nine cells, and the two axes move in different columns.**
    ///
    /// This was filed `Unreachable { needs: "`ColorDepth`" }` with the re-export — *what lifted the
    /// barrier* — as its inverter, so the claim had been
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
        // repertoire — which is the two axes being two axes.
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
    /// engine's and happens before the mirror. The refinement 1 in a third place, and the reason
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
            // **The declared bit and the wire agree**, which is the claim: a distinction
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
    /// The ticket asks for the gallery to join the wrapper list *because it uses crossterm for
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

    /// **The screen clears once, and again only when what decides every cell of it moves.**
    ///
    /// `crate::app::Clears` is the mechanism and the gallery is a caller of it; what this asserts is
    /// that the caller passes it a `Ctx` once a frame rather than at start-up, because the other half
    /// of the rule is *and on a resize*.
    #[test]
    fn the_screen_clears_once_and_again_when_its_layout_or_its_theme_moves() {
        let (w, h) = (60u16, 16u16);
        let mut driver = crate::runner::driver_at(w, h, Density::default());
        let mut gallery = Gallery::new(Worker::queueing());
        driver.set_theme(*gallery.theme());
        let step = |gallery: &mut Gallery, driver: &mut vitui_runtime::ctx::Driver| -> bool {
            let before = gallery.cleared();
            let mut sink: Sink<'_> = &mut Direct;
            driver.frame(|cx| gallery.ui_into(&mut sink, cx, ""));
            gallery.cleared() != before
        };
        let mut cleared = 0;
        for _ in 0..4 {
            cleared += usize::from(step(&mut gallery, &mut driver));
        }
        assert_eq!(cleared, 1, "the gallery clears more than once at one size");

        // **And again on a page change and on a theme change**, which are the two other things that
        // decide every cell of this screen — see `Clears::relaid_into`. Steady frames in between are
        // untouched, which is what keeps the subject intact.
        gallery.next_page(w, h);
        assert!(
            step(&mut gallery, &mut driver),
            "a page change did not repaint"
        );
        assert!(
            !step(&mut gallery, &mut driver),
            "the page repaints every frame"
        );
        gallery.next_theme();
        driver.set_theme(*gallery.theme());
        assert!(
            step(&mut gallery, &mut driver),
            "a theme change did not repaint"
        );
        assert!(
            !step(&mut gallery, &mut driver),
            "the theme repaints every frame"
        );
        gallery.next_rung();
        driver.set_theme(*gallery.theme());
        assert!(
            step(&mut gallery, &mut driver),
            "a rung change did not repaint"
        );
        gallery.next_tier();
        driver.set_theme(*gallery.theme());
        assert!(
            step(&mut gallery, &mut driver),
            "a depth change did not repaint"
        );
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

    /// **A carried surface after a change equals a fresh surface of what it changed to.**
    ///
    /// The equality residue means, and the one this module shipped without. A page change puts a
    /// **different component in the same rectangle**, and when this was written the second half was
    /// not met on this screen — 525 cells of 3 000 at 100x30 written by nobody — so `Ctrl+N` left the
    /// previous page inside the new page's frames: a `radio` panel with a `collection`'s rows in it,
    /// on a screen whose every gate was green.
    ///
    /// **It holds for two reasons now and it did for one.** Components 40 closed the frame half, so a
    /// page writes every cell of the screen and cannot inherit anything; the clear at the top of
    /// [`Gallery::ui_into`] closes the *sequence* half, which is a caller's guarantee about the frame
    /// where the **terminal** rather than a component decided what a cell held. This equality is what
    /// would notice either one going away.
    ///
    /// **The reason every gate was green is the instrument, and it is the shape this crate keeps
    /// meeting**: [`swap`] rendered the *after* picture onto a **fresh** [`crate::runner::Pen`], and
    /// a recorder that starts blank cannot see residue. `crate::golden`'s multi-frame note says so in
    /// as many words about its own surface. Both now carry one.
    #[test]
    fn a_carried_surface_after_a_change_equals_a_fresh_one() {
        let (w, h) = (100u16, 30u16);
        let fresh = |steps: usize, theme: usize| {
            let mut driver = crate::runner::driver_at(w, h, Density::default());
            let mut gallery = Gallery::new(Worker::queueing());
            for _ in 0..steps {
                gallery.next_page(w, h);
            }
            for _ in 0..theme {
                gallery.next_theme();
            }
            driver.set_theme(*gallery.theme());
            let mut pen = crate::runner::Pen::new(w, h);
            for _ in 0..3 {
                gallery.bag.answer_queued();
                pen.end_frame();
                let mut sink: Sink<'_> = &mut pen;
                driver.frame(|cx| gallery.ui_into(&mut sink, cx, ""));
            }
            pen.into_canvas()
        };

        // The carried arm: page 1, then walk to the page or theme under test, on one surface.
        let carried = |steps: usize, theme: usize| {
            let mut driver = crate::runner::driver_at(w, h, Density::default());
            let mut gallery = Gallery::new(Worker::queueing());
            driver.set_theme(*gallery.theme());
            let mut pen = crate::runner::Pen::new(w, h);
            let go = |gallery: &mut Gallery,
                      driver: &mut vitui_runtime::ctx::Driver,
                      pen: &mut crate::runner::Pen| {
                gallery.bag.answer_queued();
                pen.end_frame();
                let mut sink: Sink<'_> = pen;
                driver.frame(|cx| gallery.ui_into(&mut sink, cx, ""));
            };
            for _ in 0..3 {
                go(&mut gallery, &mut driver, &mut pen);
            }
            for _ in 0..steps {
                gallery.next_page(w, h);
                go(&mut gallery, &mut driver, &mut pen);
            }
            for _ in 0..theme {
                gallery.next_theme();
                driver.set_theme(*gallery.theme());
                go(&mut gallery, &mut driver, &mut pen);
            }
            // Two more, so the arm is compared at rest rather than one frame after the change.
            for _ in 0..2 {
                go(&mut gallery, &mut driver, &mut pen);
            }
            pen.into_canvas()
        };

        for (steps, theme) in [(1usize, 0usize), (2, 0), (0, 1), (1, 1)] {
            let diff = carried(steps, theme).diff(&fresh(steps, theme));
            assert!(
                diff.clean(),
                "after {steps} page change(s) and {theme} theme change(s) the carried surface keeps \
                 what the previous screen wrote: {diff:?}"
            );
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

    /// **No cell of the assembled gallery is written by nobody**, at every size and on every page.
    ///
    /// The sentinel gate, and this is the half stated over the assembled screen: *distinct cells
    /// touched == area.w * area.h*. It was pinned red for nine tickets with **10 252 cells of
    /// 24 000 at 300x80 and 525 of 3 000 at 100x30** as its failing set, and the pin was on this
    /// screen since components 39 built it.
    ///
    /// # The detector is the recorder and not the sentinel, and that is the finding
    ///
    /// A **sentinel** is prescribed: stamp a paint no role can produce over the base layer,
    /// draw one more frame, count the cells still carrying it. Two of its three barriers are gone
    /// and the third is a decision — *the cell is never visible in the public API* — so
    /// counting survivors on the composited surface has no expression to write, in this crate or in
    /// the engine's own callers. It does not need one:
    ///
    /// - **The rule's first half has always been read off the recorder.** `writes == distinct` is
    ///   `Tally`'s two counters, and `distinct` has unioned in the coordinates of the frame's root
    ///   since components 19. *The second half is that same union against the area* — so read on the
    ///   screen it would be an equality between two different instruments, which is what makes the
    ///   pair a pair.
    /// - **The recorder is the stricter of the two.** A verb that does not go through the caller's
    ///   `Ink` is invisible to it, so it **under**-counts what was written and fails loudly; a
    ///   surface probe counts the engine's own clear and passes quietly. A component drawing with
    ///   `cx.text` rather than through the ink it was handed is components 15's own defect — 78
    ///   cells and 10 verbs on a frame that wrote 1 560, on a screen that looked correct — and it is
    ///   this number that moves when it happens.
    ///
    /// So the number this asserts is [`Shape::unwritten`], `crate::runner::Pen`'s, and
    /// [`crate::counters::sentinel`] is the same question asked of a canvas.
    ///
    /// **Swept over sizes and pages**, because a page is a different set of twelve drawings in the
    /// same rectangles and the grid is a different shape at every width: the two empty tiles of a
    /// six-by-five grid exist at 300x80 and at no other size on this list.
    #[test]
    fn no_cell_of_the_assembled_gallery_is_written_by_nobody() {
        for (w, h) in [
            (300u16, 80u16),
            (200, 60),
            (137, 47),
            (100, 30),
            (79, 24),
            (52, 17),
            (26, 9),
            (25, 8),
        ] {
            let cells = usize::from(w) * usize::from(h);
            for page in 0..pages(w, h) {
                let shape = shape_on(w, h, page, Remainder::Written);
                assert_eq!(
                    shape.unwritten, 0,
                    "{w}x{h} page {page}: {} cells of {cells} keep whatever was already in them, \
                     which on `Ctrl+N` is the panel that used to be in that tile",
                    shape.unwritten
                );
                // **Both halves at once, or the fix is a trade.** A blanket fill would make the
                // first number 0 and the second one large, which is the defect the rule was
                // widened to catch.
                assert_eq!(
                    shape.writes, shape.distinct,
                    "{w}x{h} page {page}: a cell was written twice"
                );
                assert_eq!(
                    shape.distinct as usize, cells,
                    "{w}x{h} page {page}: the two halves are one equality"
                );
            }
        }
    }

    /// See [`no_cell_of_the_assembled_gallery_is_written_by_nobody`]. **The spelling it replaced,
    /// watched leaving its exact set behind — five drawings and the grid's own slack.**
    ///
    /// [`Remainder::LeftAlone`] is one line and draws through the same twenty-nine call sites, so
    /// this is the register's failing set attributed to the tile each cell is in rather than a total:
    ///
    /// | at 300x80 | cells | who owns it |
    /// |---|---|---|
    /// | `panel` | **245** | `Frame::interior` — a `block` returns the rectangle it did not write |
    /// | `scrollbar` | **532** | this module's own narrowing: a bar is three columns of a wider tile |
    /// | `collapsible` | **410** | `Disclosure::used` — every row below the section is the caller's |
    /// | `meter` | **492** | this module's own narrowing: a band is one row of a taller tile |
    /// | `slider` | **533** | the same narrowing, and the tile is one row taller |
    /// | six slots with no panel | **4 128** | the grid's: twenty-nine panels in a seven-by-five grid |
    ///
    /// **The two bands are the newest pair and they are a caller's mistake rather than a
    /// component's.** `meter_into` and `slider_into` fill the cross axis of the rectangle they are
    /// handed, which is the right drawing of a thick slider and the wrong drawing of a panel whose
    /// height is the grid's arithmetic; [`draws::band`] narrows each to one row, so the rows either
    /// side became a remainder this module owns. The two figures differ by exactly one row because
    /// the tiles do: a grid boundary at `grid.h * k / rows` leaves some tiles a row taller than
    /// others.
    ///
    /// **Every figure moved when the twenty-ninth panel arrived, and none of them
    /// moved because a component changed.** A page holds `cols * rows` tiles and twenty-nine panels
    /// need a wider grid than twenty-eight, so every tile is smaller and every remainder with it —
    /// which is why these are asserted as measured rather than carried forward. The **shape** is the
    /// finding and it is unchanged: five drawings that hand a rectangle back and a grid whose last
    /// row is not full.
    ///
    /// **At 100x30 it is 145 and all of it is the scrollbar's**, because a page of twelve fills the
    /// grid exactly and `collapsible`, `meter` and `slider` are all on page two — one size agreeing
    /// with a law the other breaks is how a gate over one size stays green, and this module has met
    /// it twice.
    #[test]
    fn the_remainder_left_alone_is_five_drawings_and_the_grids_own_slack() {
        let attribute = |w: u16, h: u16| -> Vec<(&'static str, usize)> {
            let canvas = screen(w, h, 3, Remainder::LeftAlone);
            let (cols, rows, _) = grid(w, h);
            let grid_rows = Rect::new(0, 1, w, h.saturating_sub(CHROME_ROWS));
            let mut out = Vec::new();
            for slot in 0..(cols as usize * rows as usize) {
                let t = tile(grid_rows, cols, rows, slot as u16);
                let mut un = 0;
                for y in t.y..t.y + i32::from(t.h) {
                    for x in t.x..t.x + i32::from(t.w) {
                        if canvas.get(x as u16, y as u16).is_none() {
                            un += 1;
                        }
                    }
                }
                if un > 0 {
                    out.push((
                        PANELS.get(slot).map_or("a slot with no panel", |p| p.id),
                        un,
                    ));
                }
            }
            out
        };
        assert_eq!(
            attribute(300, 80),
            vec![
                ("panel", 245),
                ("scrollbar", 532),
                ("collapsible", 410),
                ("meter", 492),
                ("slider", 533),
                ("a slot with no panel", 688),
                ("a slot with no panel", 688),
                ("a slot with no panel", 688),
                ("a slot with no panel", 688),
                ("a slot with no panel", 688),
                ("a slot with no panel", 688),
            ]
        );
        assert_eq!(attribute(100, 30), vec![("scrollbar", 145)]);
        // And the total is the surface's, so no cell of the set is outside the grid.
        assert_eq!(shape_as(300, 80, 3, Remainder::LeftAlone).unwritten, 6_340);
        assert_eq!(shape_as(100, 30, 3, Remainder::LeftAlone).unwritten, 145);
    }

    /// **Every panel writes every cell of the interior it was handed**, which is the sentinel gate per
    /// component and over the shipped call site rather than over a second arrangement of it.
    ///
    /// The account of why this row survived: *the per-component forms were report-only across
    /// nine to twelve binaries*. [`shot`] draws one panel through the same tile loop the screen uses,
    /// so what is asserted here is what the gallery draws.
    ///
    /// # Twenty-six of the twenty-eight already held, and the two that did not are the overlay
    /// owners
    ///
    /// Measured before anything was changed: handed a rectangle taller and wider than its content,
    /// every row of the freeze wrote all of it except **`select` and `file_picker` — 576 cells of a
    /// 48x13 interior each** — which are exactly the two whose body is in another layer. Their
    /// remainder could not be *named*: the third clause is *the cells it does not write are named
    /// in its return value*, and both return the runtime's `Response`, which has no field for one.
    /// Writing them is the only reachable answer and both now do.
    ///
    /// The five that hand part of a tile back do name it — `Frame::interior`, `Disclosure::used`,
    /// and a bar and two bands this module narrows itself — and the owner writes it; see
    /// [`the_remainder_left_alone_is_five_drawings_and_the_grids_own_slack`].
    #[test]
    fn every_panel_writes_every_cell_of_the_interior_it_was_handed() {
        for (w, h) in [(50u16, 15u16), (34, 7), (44, 9), (26, 7), (60, 20)] {
            let cells = usize::from(w) * usize::from(h);
            for panel in PANELS {
                let canvas = shot(panel.id, w, h, 3);
                assert_eq!(
                    canvas.written(),
                    cells,
                    "{}: {} cells of a {w}x{h} tile were written by nobody",
                    panel.id,
                    cells - canvas.written()
                );
            }
        }
        // **Watched failing on the three that hand a rectangle back**, at the size the table above
        // is read at. `select` and `file_picker` are not on this list and cannot be: their fill is
        // inside the component now, and a component's own partition is not an axis this module can
        // flip.
        let left = |id: &str| -> usize {
            let canvas = shot_as(id, 50, 15, 3, Remainder::LeftAlone);
            50 * 15 - canvas.written()
        };
        assert_eq!(left("panel"), 294);
        assert_eq!(left("scrollbar"), 585);
        assert_eq!(left("collapsible"), 432);
        assert_eq!(
            left("select"),
            0,
            "the component's own fill is not this flag's"
        );
        assert_eq!(left("file_picker"), 0);
    }

    /// **No cell carries the previous palette a frame after a swap.**
    ///
    /// **0 of 24 000 at 300x80 and 0 of 3 000 on every page at 100x30**, on all three axes and at
    /// both arms of [`Remainder`]. It is the last of the two pinned reds and the second whose
    /// failing set was measured on this screen.
    ///
    /// # The gate is a count over the surface against an oracle, and it had to be
    ///
    /// The first refinement is *a threshold on the wrong side of the question is not a weak gate,
    /// it is a green one*, and this row is its own example: the spelling that shipped asserted
    /// `changed > 0`, and one cell of 4 800 satisfies it while 3 583 carry the old palette. The
    /// complement is not the repair either — on this screen `kept` is **17 884 of 24 000** under a
    /// rung change with nothing whatever wrong, because a rung change moves the cells drawn from the
    /// theme's glyph table and no others, and it is **23 990 of 24 000** under a tier change because
    /// the colour axis moves nothing on any canvas at all. *Both numbers are the delta,
    /// read from its two ends.*
    ///
    /// What separates *the swap reached nothing* from *the swap had nothing to reach* is a second
    /// arm: the same gallery, same page, same four frames, with the destination theme in place from
    /// the first. [`Swap::stale`] is what the two disagree about.
    ///
    /// # And it is watched failing, over this screen, on the memo the rule is about
    ///
    /// [`crate::memos`]'s finding is that **no shipped memo in this crate holds a paint or a
    /// cluster**, so the rule is true here vacuously and a gate resting on it would be green
    /// for ever. [`Keying`] is the memo built to be subject to it, at a right arm and a wrong one
    /// over the same twenty-eight call sites, and the wrong one is
    /// `the_tier_keyed_memo_is_wrong_on_this_screen_and_the_old_gate_passes_on_it`.
    #[test]
    fn no_cell_of_the_assembled_gallery_carries_the_previous_palette() {
        for (w, h) in [(100u16, 30u16), (300, 80)] {
            for page in 0..pages(w, h) {
                for change in [Change::Scheme, Change::Rung, Change::Tier] {
                    // **Both arms of `Remainder`, so row 7 and row 8 are green at the same time.**
                    // The criterion, and the reading is that they are independent here:
                    // `swap_on` carries one surface across the change, so a cell nobody writes on a
                    // steady frame was still written once, and the reference arm carries its own.
                    for remainder in [Remainder::Written, Remainder::LeftAlone] {
                        let s = swap_on(w, h, page, change, remainder, Keying::Derived);
                        assert_eq!(
                            (s.stale, s.stale_on_a_panel),
                            (0, 0),
                            "{change:?} at {w}x{h} page {page} ({remainder:?}): {} carry the \
                             previous palette",
                            s.divergence
                        );
                        assert!(
                            s.written > 0,
                            "{change:?} at {w}x{h} page {page} drew nothing"
                        );
                    }
                }
            }
        }
    }

    /// **The rule's own arm, and the proof the memo was live.**
    ///
    /// [`Keying::Themed`] keys the per-tile memo with `Theme::memo_key` — the theme's own
    /// `Revision` folded with the data's — and the gate stays at zero with the memo **hitting**: 56
    /// of 112 tile draws at 300x80 are served from it, which is frames two and three, and the swap
    /// frame is a miss because the theme moved.
    ///
    /// Without the second number this arm would be green on a memo that never hit, which is the same
    /// vacuity one level in: *a memo that cannot be stale says nothing about a rule against
    /// staleness*.
    #[test]
    fn a_memo_keyed_on_the_themes_own_revision_is_never_stale_and_does_hit() {
        for (w, h) in [(100u16, 30u16), (300, 80)] {
            for change in [Change::Scheme, Change::Rung, Change::Tier] {
                let s = swap_on(w, h, 0, change, Remainder::Written, Keying::Themed);
                assert_eq!(
                    (s.stale, s.stale_on_a_panel),
                    (0, 0),
                    "{change:?} at {w}x{h}: the rule's own key left {} behind",
                    s.divergence
                );
                assert!(
                    s.served > 0,
                    "{change:?} at {w}x{h}: the memo never hit, so `stale == 0` is a statement \
                     about a memo that was never used"
                );
                // Four frames of the same tiles: the first misses, the two steady ones hit, and the
                // swap frame misses because the theme's revision moved. An equality rather than a
                // ratio, because the count of tiles is the page's and the count of frames is this
                // function's.
                assert_eq!(
                    s.served, s.drew,
                    "{change:?} at {w}x{h}: two hits and two misses per tile is what a swap on the \
                     fourth of four frames costs, and this arm no longer spends them that way"
                );
            }
        }
    }

    /// **The defect, on this screen: `(data, tier)` keeps the previous palette and `changed > 0` is
    /// green on it.**
    ///
    /// The refinement 1 as a measurement over the assembled gallery rather than as an arithmetic
    /// on recalled numbers — `tests/gates.rs` keeps the arithmetic beside it, and this is the same
    /// sentence with the screen underneath.
    ///
    /// **The sharpest reading is page three at 100x30**: `changed` is **2 159 of 3 000** and 861
    /// cells carry the old palette. A gate on the delta sees a screen that moved two thirds of
    /// itself and passes; the complement sees a screen a third of which is wrong.
    ///
    /// **And `Change::Tier` is 0 on the defective arm**, which is the whole argument for the key
    /// being the theme's own `Revision` rather than an enumeration: the axis this key remembered is
    /// right, and it is the one axis that moves nothing anyway. *An enumeration of axes
    /// is a key the next reader forgets one of, and this one forgot the two that matter.*
    #[test]
    fn the_tier_keyed_memo_is_wrong_on_this_screen_and_the_old_gate_passes_on_it() {
        /// The spelling that shipped. **A threshold on the delta.**
        fn the_gate_that_was_there(s: &Swap) -> bool {
            s.changed > 0
        }

        for (w, h) in [(100u16, 30u16), (300, 80)] {
            for page in 0..pages(w, h) {
                for change in [Change::Scheme, Change::Rung] {
                    let s = swap_on(w, h, page, change, Remainder::Written, Keying::DataAndTier);
                    assert!(
                        s.stale > 0 && s.stale_on_a_panel > 0,
                        "{change:?} at {w}x{h} page {page}: the tier-keyed memo was not stale, so \
                         this gate cannot be watched failing"
                    );
                    assert!(
                        the_gate_that_was_there(&s),
                        "{change:?} at {w}x{h} page {page}: the spelling this replaced would have \
                         failed here, which is the one thing refinement 1 says it does not do"
                    );
                    // The memo was hit **on the swap frame**, which is the defect itself: three of
                    // the four frames are served and only the first is drawn.
                    assert!(
                        s.served > s.drew,
                        "{change:?} at {w}x{h} page {page}: served {} against drew {}",
                        s.served,
                        s.drew
                    );
                }
                // The one axis this key does remember.
                let tier = swap_on(
                    w,
                    h,
                    page,
                    Change::Tier,
                    Remainder::Written,
                    Keying::DataAndTier,
                );
                assert_eq!(
                    (tier.stale, tier.served, tier.drew),
                    (0, tier.drew, tier.served),
                    "the colour depth is in this key, so `Ctrl+L` invalidates it — and it is the \
                     one press that moves nothing on any panel"
                );
            }
        }
    }

    /// **`kept` and `changed` are reports, and this is why.**
    ///
    /// Both are the delta read from one of its two ends, and on two of the three axes the delta is
    /// the same number on a healthy screen and on a broken one. Kept as the record of what the row's
    /// failing set was measured with, and asserted here so that the numbers in
    /// `examples/gallery_numbers.rs` cannot drift without a test moving.
    #[test]
    fn the_delta_is_printed_and_never_gated() {
        for (w, h) in [(100u16, 30u16), (300, 80)] {
            for change in [Change::Scheme, Change::Rung] {
                let swap = swap(w, h, change);
                assert!(swap.written > 0, "{change:?} at {w}x{h}");
                assert!(
                    swap.changed_on_a_panel > 0,
                    "{change:?} at {w}x{h} moved no cell of any panel"
                );
            }
            // **The two rows are independent, measured**: the same swap with every remainder left
            // to whatever was already in the cells keeps the same count.
            for change in [Change::Scheme, Change::Rung, Change::Tier] {
                assert_eq!(
                    swap_as(w, h, change, Remainder::LeftAlone).kept,
                    swap(w, h, change).kept,
                    "{change:?} at {w}x{h}: row 8's number moved with row 7's, so `swap` has \
                     stopped carrying one surface across the change"
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
