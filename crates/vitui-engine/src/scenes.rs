//! The twelve scenes of spec §14, as a normative list rather than an appendix.
//!
//! **Three scenes that score identically on every candidate validate the wrong design while
//! reporting success**, and that is exactly what happened to the damage model: the ticket named a
//! blinking caret, a scrolling list and twenty stacked popups, and all three scored 1.00x on every
//! candidate structure. The two that discriminated — a status bar across three dialogs, and a
//! sub-cell chart — were not on the list, and a model validated only against its own scenes would
//! have been **wrong by 37.07x** on a component the map had already committed to supporting.
//!
//! So a scene list is part of a gate, and omitting a scene is a defect. Three rules govern this one:
//!
//! - **Numbers are kept per scene and never summed.** The 27x scroll-detector regression was
//!   visible only that way.
//! - **A scene is removed only by a ticket that names the property it can no longer distinguish.**
//!   Twenty popups discriminates nothing today and stays anyway, because "discriminates nothing
//!   today" is a statement about the current candidates.
//! - **The sparse sub-cell chart is not optional.** It has decided three tickets.
//!
//! # Where §14 says this lives, and where it actually lives
//!
//! §14's own pointer is "the list lives in `crates/vitui-engine/examples/budget.rs`". It lives one
//! file across from there, and the example `#[path]`-includes it, for a reason the spec could not
//! have known: **an example cannot assert the reference compositor's equality**, because ADR 0023
//! keeps cells off the public surface and an example has nothing but the public surface. The
//! example still owns, prints and times all twelve; the gates that need cells are inside the crate.
//!
//! # Compiled twice, on purpose
//!
//! This file is a module of the library under `cfg(test)`, where the gates in `crate::gates` drive
//! it against the reference compositor and the terminal model, and it is `#[path]`-included by
//! `examples/budget.rs`, where the same scenes are timed. One definition, two consumers, and the
//! two can never drift.
//!
//! It therefore uses **only the public API** — `use vitui_engine::…` resolves inside the library
//! through `extern crate self as vitui_engine`. That is not a trick for its own sake: spec §14's
//! rule for the comparative suite is that **a scene is defined by what the user sees, never by what
//! a framework does**, and a scene that could reach past the public surface would be defined by
//! what this framework does.
//!
//! # The contract every scene owes: what `step` returns
//!
//! `step` returns **how many distinct cells its verbs wrote this frame**, and gate #3 divides the
//! cells the damage structure reported by that number. The ratio is 1.00x for `RowBits` on every
//! scene because the bitset marks exactly the cells the verbs marked — that is a property of the
//! **mechanism**, not of the data, which is what spec §14 requires of an equality gate.
//!
//! Two things follow, and both are why the count is declared rather than summed from what the
//! drawing verbs return:
//!
//! - A scene whose verbs overlap inside one layer — clearing a row and then writing over part of it
//!   — writes some cells twice, and `Written::cells` would count them twice. The scene knows the
//!   distinct number and says so.
//! - The count is per **layer damage summed**, never the frame's union, because two popups that
//!   overlap on screen still write into two separate surfaces and each surface's bitset is exact
//!   about its own.
//!
//! A wrong declaration is not silently absorbed: too high and gate #3 reports below 1.00x, too low
//! and it reports above. And a genuine under-report is caught by gate #1 independently, because
//! that one is generated from the reference compositor rather than from anything a scene says.

use std::fmt::Write as _;

use vitui_engine::{Color, LayerId, Rect, Screen, Style};

use crate::register::State;

/// The screen every scene is measured at: spec §13's full screen.
pub const W: u16 = 300;
/// The screen every scene is measured at: spec §13's full screen.
pub const H: u16 = 80;

/// One scene of spec §14's normative list.
pub trait Scene {
    /// The scene's name, which is also its case name in the bench report.
    fn name(&self) -> &'static str;

    /// What this scene decided, with the architecture ticket that decided it. Printed by the
    /// example, so the list carries its own reason for existing.
    fn decided(&self) -> &'static str;

    /// Whether the scene can run today.
    ///
    /// The same [`State`] the register's entries use, because it is the same statement: this runs
    /// **here**, or it does not run and **that ticket** is what lights it. A scene quietly dropped
    /// until its mechanism arrives is a scene nobody puts back.
    fn status(&self) -> State {
        State::Wired {
            at: "crate::gates, and timed in examples/budget.rs",
        }
    }

    /// Add the scene's layers. Called once, before any [`Scene::step`].
    fn build(&mut self, screen: &mut Screen);

    /// Draw frame `t`, and return how many **distinct** cells the verbs wrote.
    fn step(&mut self, screen: &mut Screen, t: u32) -> u32;

    /// How many times the example repeats draw-and-present inside one timed sample.
    ///
    /// No default: the twelve scenes span three orders of magnitude — a caret frame is nanoseconds
    /// and the adversarial page is most of a millisecond — so a number that suits one of them suits
    /// none of the others, and a default would be a wrong answer that nobody had to type.
    fn iters(&self) -> u32;
}

/// The twelve scenes, in spec §14's own order.
pub fn scenes() -> Vec<Box<dyn Scene>> {
    vec![
        Box::new(CaretBlink::new()),
        Box::new(ScrollingList::label_only()),
        Box::new(ScrollingList::rows_cleared()),
        Box::new(TwentyPopups::new()),
        Box::new(ThreeDialogs::new()),
        Box::new(SparseChart::new()),
        Box::new(ProgressBar::new()),
        Box::new(VirtualisedTree::new(1_000_000)),
        Box::new(TableTwoWays::new(1_000_000)),
        Box::new(FullScreenChange::new()),
        Box::new(EveryCellADistinctStyle::new()),
        Box::new(HyperlinkedPageUnderAnOperator),
    ]
}

/// A full-screen opaque base layer, which every scene stands on.
fn base(screen: &mut Screen) -> LayerId {
    screen.layers().add_content(0, Rect::new(0, 0, W, H), true)
}

/// Twenty popups, each with a shadow under it, over a base.
///
/// Spec §5 measured 107.3 us for a full-screen composite **at forty layers**, and these are those
/// forty; the base is the forty-first because a screen has to have something under them. The shadow
/// is a non-opaque content layer here — it becomes an operator layer at ticket 12, which is where
/// the `Mix` and the atomic glyph rule arrive.
///
/// # Why the shadow has rounded corners
///
/// A shadow drawn as a full rectangle does not exercise anything. It sits directly under its own
/// popup, offset by one cell, so every cell of it that a caller left transparent is covered by the
/// popup anyway — and a compositor that painted `EMPTY` instead of skipping it would produce an
/// identical screen. Measured, not reasoned: deleting the skip test from the reference compositor
/// left every gate on this list green.
///
/// So the shadow is the **L** a real drop shadow actually is, with its two far corners left
/// transparent — which is spec §5's own example of what `opaque: false` is for, and which puts four
/// cells per popup where what is underneath must show through. Those eighty cells are what make the
/// non-opaque path load-bearing in the scene list rather than merely present in it.
fn popup_stack(screen: &mut Screen) -> Vec<LayerId> {
    const N: i32 = 20;
    const WIDE: u16 = 90;
    const TALL: u16 = 14;
    let mut popups = Vec::with_capacity(N as usize);
    for i in 0..N {
        let x = (i * 11) % (W as i32 - 92);
        let y = (i * 3) % (H as i32 - 15);

        // The popup first, then a shadow *under* it, which is how anyone would write it: the
        // shadow is a consequence of the popup, not a thing you plan for.
        //
        // It is worth saying what this does **not** buy, because it looks like it should. Insertion
        // order now disagrees with z order — and a stack sorting on `(seq, z)` instead of `(z, seq)`
        // still passes every gate on this list, because a shadow's painted cells are exactly the
        // ones its own popup does not cover, so the two are never ordered against each other where
        // anyone can see. No scene of spec §14's twelve discriminates z order; `layer.rs`'s own
        // `layers_added_out_of_z_order_still_paint_in_z_order` is what holds that property, and it
        // catches the mutation this comment describes.
        popups.push(
            screen
                .layers()
                .add_content(2 * i + 2, Rect::new(x, y, WIDE, TALL), true),
        );
        let shadow =
            screen
                .layers()
                .add_content(2 * i + 1, Rect::new(x + 2, y + 1, WIDE, TALL), false);
        let dark = Style::new().bg(Color::indexed(236));
        let mut view = screen.layers().view(shadow).expect("just added");
        // The two-column strip down the right, and the one-row strip along the bottom. Neither
        // reaches the corner the other one would round off.
        view.fill(Rect::new(88, 1, 2, TALL - 2), " ", dark);
        view.fill(Rect::new(2, TALL as i32 - 1, WIDE - 2, 1), " ", dark);
    }
    popups
}

/// Write `label` into `buf` without allocating after the first frame.
fn label(buf: &mut String, prefix: &str, n: u32, width: usize) {
    buf.clear();
    let _ = write!(buf, "{prefix}{n}");
    while buf.len() < width {
        buf.push(' ');
    }
    buf.truncate(width);
}

// ---------------------------------------------------------------------------------------------
// 1. A blinking caret
// ---------------------------------------------------------------------------------------------

/// One cell toggles, over the forty-layer stack the number was taken at.
struct CaretBlink {
    top: Option<LayerId>,
}

impl CaretBlink {
    fn new() -> CaretBlink {
        CaretBlink { top: None }
    }
}

impl Scene for CaretBlink {
    fn name(&self) -> &'static str {
        "caret-blink"
    }

    fn decided(&self) -> &'static str {
        "171 ns composite at 40 layers; the 29-byte frame that makes framing visible (arch 06, 08)"
    }

    fn build(&mut self, screen: &mut Screen) {
        base(screen);
        let popups = popup_stack(screen);
        self.top = popups.last().copied();
    }

    fn step(&mut self, screen: &mut Screen, t: u32) -> u32 {
        let style = if t % 2 == 0 {
            Style::new().reverse()
        } else {
            Style::new()
        };
        let id = self.top.expect("build ran");
        screen
            .layers()
            .view(id)
            .expect("the layer is still there")
            .text(10, 5, " ", style);
        1
    }

    fn iters(&self) -> u32 {
        1_000
    }
}

// ---------------------------------------------------------------------------------------------
// 2 and 3. A scrolling list, label only, and with its rows cleared
// ---------------------------------------------------------------------------------------------

/// A list of `H` rows that scrolls by one row a frame.
///
/// Two scenes, one implementation, because the difference between them **is** the property: the
/// label-only arm rewrites a 20-column label and nothing else; the cleared arm blanks the whole row
/// first, which is what makes the scroll region worth having.
struct ScrollingList {
    layer: Option<LayerId>,
    clear_rows: bool,
    buf: String,
}

impl ScrollingList {
    fn label_only() -> ScrollingList {
        ScrollingList {
            layer: None,
            clear_rows: false,
            buf: String::new(),
        }
    }

    fn rows_cleared() -> ScrollingList {
        ScrollingList {
            layer: None,
            clear_rows: true,
            buf: String::new(),
        }
    }

    const LABEL: usize = 20;
}

impl Scene for ScrollingList {
    fn name(&self) -> &'static str {
        if self.clear_rows {
            "scrolling-list-rows-cleared"
        } else {
            "scrolling-list-label-only"
        }
    }

    fn decided(&self) -> &'static str {
        if self.clear_rows {
            "the scroll region: 1 726 -> 60 bytes; 34.5 us of packing (arch 08)"
        } else {
            "span damage against the equality filter, 1.84x (arch 08)"
        }
    }

    fn build(&mut self, screen: &mut Screen) {
        self.layer = Some(base(screen));
    }

    fn step(&mut self, screen: &mut Screen, t: u32) -> u32 {
        let id = self.layer.expect("build ran");
        let mut view = screen.layers().view(id).expect("the layer is still there");
        for y in 0..H as u32 {
            if self.clear_rows {
                view.fill(Rect::new(0, y as i32, W, 1), " ", Style::new());
            }
            label(&mut self.buf, "row ", t + y, ScrollingList::LABEL);
            view.text(0, y as i32, &self.buf, Style::new());
        }
        if self.clear_rows {
            // Every cell of every row, written once by the fill and some of them a second time by
            // the label. Distinct cells, which is what gate #3 divides by.
            W as u32 * H as u32
        } else {
            ScrollingList::LABEL as u32 * H as u32
        }
    }

    fn iters(&self) -> u32 {
        if self.clear_rows { 50 } else { 200 }
    }
}

// ---------------------------------------------------------------------------------------------
// 4. Twenty stacked popups with shadows
// ---------------------------------------------------------------------------------------------

/// Forty layers, and every popup repaints its whole rectangle.
struct TwentyPopups {
    popups: Vec<LayerId>,
    buf: String,
}

impl TwentyPopups {
    fn new() -> TwentyPopups {
        TwentyPopups {
            popups: Vec::new(),
            buf: String::new(),
        }
    }
}

impl Scene for TwentyPopups {
    fn name(&self) -> &'static str {
        "twenty-popups-with-shadows"
    }

    fn decided(&self) -> &'static str {
        "107.3 us full-screen composite at 40 layers; discriminates nothing between damage \
         structures and stays anyway (arch 06)"
    }

    fn build(&mut self, screen: &mut Screen) {
        base(screen);
        self.popups = popup_stack(screen);
    }

    fn step(&mut self, screen: &mut Screen, t: u32) -> u32 {
        let style = Style::new().fg(Color::indexed((t % 16) as u8));
        for i in 0..self.popups.len() {
            let id = self.popups[i];
            let mut view = screen.layers().view(id).expect("the layer is still there");
            for y in 0..14 {
                label(&mut self.buf, "popup ", t + i as u32 + y as u32, 90);
                view.text(0, y, &self.buf, style);
            }
        }
        90 * 14 * self.popups.len() as u32
    }

    fn iters(&self) -> u32 {
        20
    }
}

// ---------------------------------------------------------------------------------------------
// 5. Three dialogs standing apart
// ---------------------------------------------------------------------------------------------

/// Three status regions with wide gaps between them: the scene per-row spans lose 2.53x on.
struct ThreeDialogs {
    layer: Option<LayerId>,
    buf: String,
}

impl ThreeDialogs {
    fn new() -> ThreeDialogs {
        ThreeDialogs {
            layer: None,
            buf: String::new(),
        }
    }

    /// `(x, width)` for each dialog. The gaps are the point: a per-row span model merges the three
    /// into one run 298 cells wide and carries 14 283 unchanged cells with it.
    const COLUMNS: [(i32, usize); 3] = [(2, 60), (140, 50), (271, 29)];
    const ROWS: std::ops::Range<i32> = 4..18;
}

impl Scene for ThreeDialogs {
    fn name(&self) -> &'static str {
        "three-dialogs-apart"
    }

    fn decided(&self) -> &'static str {
        "per-row spans 2.53x overdraw (arch 07); the gap merge, 1 491 -> 1 203 bytes (arch 08)"
    }

    fn build(&mut self, screen: &mut Screen) {
        self.layer = Some(base(screen));
    }

    fn step(&mut self, screen: &mut Screen, t: u32) -> u32 {
        let id = self.layer.expect("build ran");
        let style = Style::new().fg(Color::indexed((t % 16) as u8));
        let mut view = screen.layers().view(id).expect("the layer is still there");
        for y in ThreeDialogs::ROWS {
            for (x, width) in ThreeDialogs::COLUMNS {
                label(&mut self.buf, "dialog ", t + y as u32, width);
                view.text(x, y, &self.buf, style);
            }
        }
        let per_row: usize = ThreeDialogs::COLUMNS.iter().map(|(_, w)| w).sum();
        per_row as u32 * ThreeDialogs::ROWS.len() as u32
    }

    fn iters(&self) -> u32 {
        200
    }
}

// ---------------------------------------------------------------------------------------------
// 6. A sub-cell chart, 400 points
// ---------------------------------------------------------------------------------------------

/// Four hundred one-cell writes scattered over the screen. **Not optional.**
///
/// It has decided three tickets: 37.07x overdraw against per-row spans (arch 07), the 4.88x
/// serializer walk that made the packet carry runs rather than the grid (arch 09), and the
/// 166.76 us iteration that set the watchdog threshold to a frame interval rather than to 100 us
/// (arch 18).
struct SparseChart {
    layer: Option<LayerId>,
}

impl SparseChart {
    fn new() -> SparseChart {
        SparseChart { layer: None }
    }

    const POINTS: i32 = 400;

    /// The four hundred positions are **distinct within a frame**, which is what makes the declared
    /// write count exact: `y = 13i mod 80` repeats every 80 indices, and two indices 80k apart land
    /// 260k columns apart mod 300, which is never zero for k in 1..=4.
    fn point(i: i32, t: u32) -> (i32, i32) {
        ((i * 7 + t as i32) % W as i32, (i * 13) % H as i32)
    }
}

impl Scene for SparseChart {
    fn name(&self) -> &'static str {
        "sparse-chart-400-points"
    }

    fn decided(&self) -> &'static str {
        "37.07x overdraw (arch 07), the 4.88x serializer walk (arch 09), the 166.76 us watchdog \
         iteration (arch 18)"
    }

    fn build(&mut self, screen: &mut Screen) {
        self.layer = Some(base(screen));
    }

    fn step(&mut self, screen: &mut Screen, t: u32) -> u32 {
        let id = self.layer.expect("build ran");
        let mut view = screen.layers().view(id).expect("the layer is still there");
        for i in 0..SparseChart::POINTS {
            let (x, y) = SparseChart::point(i, t);
            view.text(x, y, "*", Style::new());
        }
        SparseChart::POINTS as u32
    }

    fn iters(&self) -> u32 {
        200
    }
}

// ---------------------------------------------------------------------------------------------
// 7. A progress bar advancing 1%
// ---------------------------------------------------------------------------------------------

/// A hundred cells rewritten, one of them changed.
struct ProgressBar {
    layer: Option<LayerId>,
    buf: String,
}

impl ProgressBar {
    fn new() -> ProgressBar {
        ProgressBar {
            layer: None,
            buf: String::new(),
        }
    }

    const WIDTH: u32 = 100;
}

impl Scene for ProgressBar {
    fn name(&self) -> &'static str {
        "progress-bar-one-percent"
    }

    fn decided(&self) -> &'static str {
        "100 cells rewritten, one changed: 30.8x on the equality filter (arch 08)"
    }

    fn build(&mut self, screen: &mut Screen) {
        self.layer = Some(base(screen));
    }

    fn step(&mut self, screen: &mut Screen, t: u32) -> u32 {
        let id = self.layer.expect("build ran");
        let filled = t % (ProgressBar::WIDTH + 1);
        self.buf.clear();
        for i in 0..ProgressBar::WIDTH {
            self.buf.push(if i < filled { '#' } else { '-' });
        }
        screen
            .layers()
            .view(id)
            .expect("the layer is still there")
            .text(10, 40, &self.buf, Style::new());
        ProgressBar::WIDTH
    }

    fn iters(&self) -> u32 {
        1_000
    }
}

// ---------------------------------------------------------------------------------------------
// 8. A virtualised tree, 1k / 100k / 1M rows
// ---------------------------------------------------------------------------------------------

/// The data-volume invariant, which is the one the whole engine exists to keep.
///
/// **Frame cost is proportional to visible cells, never to data volume.** The engine never iterates
/// application data, so the scene does the culling — the component draws the rows the viewport can
/// show and nothing else, whether it is backed by a thousand rows or a million. Gate #20 is the
/// ratio between the arms; the example is where it runs, because a ratio between timings is a
/// timing.
///
/// **The bound on the loop is `scrolled` and `visible_rows`, not a hand-written `0..H`** (impl
/// 09). The difference is not cosmetic: a hand-written bound gates the loop the scene wrote, while
/// the query gates the loop every component written against this API will write. A scene that
/// bounds its own loop would keep reporting 1.00x on a `visible_rows` that had started to iterate
/// something.
struct VirtualisedTree {
    layer: Option<LayerId>,
    rows: Vec<u32>,
    buf: String,
}

impl VirtualisedTree {
    fn new(rows: usize) -> VirtualisedTree {
        VirtualisedTree {
            layer: None,
            // Held rather than generated, so the arms genuinely differ in the memory they walk past.
            rows: (0..rows as u32).collect(),
            buf: String::new(),
        }
    }

    const LABEL: usize = 40;
}

impl Scene for VirtualisedTree {
    fn name(&self) -> &'static str {
        "virtualised-tree"
    }

    fn decided(&self) -> &'static str {
        "the data-volume invariant: flat at 54 us, and 14.4-15.0 us with component code \
         (arch 05, 14)"
    }

    fn build(&mut self, screen: &mut Screen) {
        self.layer = Some(base(screen));
    }

    fn step(&mut self, screen: &mut Screen, t: u32) -> u32 {
        let id = self.layer.expect("build ran");
        let first = t as usize % (self.rows.len() - H as usize);
        let mut layer = screen.layers().view(id).expect("the layer is still there");
        // The viewport, in content coordinates: row `first` is drawn at the top.
        let mut view = layer.scrolled(0, -(first as i32));
        for i in view.visible_rows() {
            // The two lines that keep the invariant: a bounded range, and an index into it — never
            // a scan, and never a loop over `self.rows`.
            let value = self.rows[i as usize];
            label(&mut self.buf, "node ", value, VirtualisedTree::LABEL);
            view.text(0, i, &self.buf, Style::new());
        }
        VirtualisedTree::LABEL as u32 * H as u32
    }

    fn iters(&self) -> u32 {
        200
    }
}

/// A tree scene of a stated size, for gate #20's arms.
pub fn virtualised_tree(rows: usize) -> Box<dyn Scene> {
    Box::new(VirtualisedTree::new(rows))
}

/// A two-layer table scene of a stated size, for gate #20's other pair of arms.
pub fn table_two_ways(rows: usize) -> Box<dyn Scene> {
    Box::new(TableTwoWays::new(rows))
}

// ---------------------------------------------------------------------------------------------
// 9. One table as a list and a bar chart
// ---------------------------------------------------------------------------------------------

/// The same data rendered twice, into two layers, in one frame.
///
/// The normative list carries the 1M arm, because §14's row is a pair of numbers "at 1k **and
/// 1M**" and the larger one is the half that could go wrong. Both arms are measured together in
/// gate #20, beside the virtualised tree's three: a scan that only appears once the same rows are
/// walked twice would be invisible on a scene that walks them once.
struct TableTwoWays {
    list: Option<LayerId>,
    chart: Option<LayerId>,
    rows: Vec<u32>,
    buf: String,
}

impl TableTwoWays {
    fn new(rows: usize) -> TableTwoWays {
        TableTwoWays {
            list: None,
            chart: None,
            rows: (0..rows as u32).map(|i| i * 37 % 41).collect(),
            buf: String::new(),
        }
    }

    const LABEL: usize = 30;
    const BAR: u32 = 40;
}

impl Scene for TableTwoWays {
    fn name(&self) -> &'static str {
        "table-as-list-and-bar-chart"
    }

    fn decided(&self) -> &'static str {
        "precedence rule 2; 22.35 / 22.97 us at 1k and 1M rows (arch 14)"
    }

    fn build(&mut self, screen: &mut Screen) {
        base(screen);
        self.list = Some(screen.layers().add_content(
            1,
            Rect::new(0, 0, TableTwoWays::LABEL as u16, H),
            true,
        ));
        self.chart = Some(screen.layers().add_content(
            1,
            Rect::new(200, 0, TableTwoWays::BAR as u16, H),
            true,
        ));
    }

    fn step(&mut self, screen: &mut Screen, t: u32) -> u32 {
        let (list, chart) = (
            self.list.expect("build ran"),
            self.chart.expect("build ran"),
        );
        let first = t as usize % (self.rows.len() - H as usize);

        let mut layer = screen.layers().view(list).expect("still there");
        let mut view = layer.scrolled(0, -(first as i32));
        for i in view.visible_rows() {
            label(
                &mut self.buf,
                "item ",
                self.rows[i as usize],
                TableTwoWays::LABEL,
            );
            view.text(0, i, &self.buf, Style::new());
        }

        let mut layer = screen.layers().view(chart).expect("still there");
        let mut view = layer.scrolled(0, -(first as i32));
        for i in view.visible_rows() {
            let n = self.rows[i as usize] % TableTwoWays::BAR;
            self.buf.clear();
            for c in 0..TableTwoWays::BAR {
                self.buf.push(if c < n { '#' } else { ' ' });
            }
            view.text(0, i, &self.buf, Style::new());
        }

        (TableTwoWays::LABEL as u32 + TableTwoWays::BAR) * H as u32
    }

    fn iters(&self) -> u32 {
        200
    }
}

// ---------------------------------------------------------------------------------------------
// 10. A full-screen change
// ---------------------------------------------------------------------------------------------

/// Every cell of 300x80 rewritten, in one style that changes each frame.
struct FullScreenChange {
    layer: Option<LayerId>,
    row: String,
}

impl FullScreenChange {
    fn new() -> FullScreenChange {
        FullScreenChange {
            layer: None,
            row: std::iter::repeat_n('x', W as usize).collect(),
        }
    }
}

impl Scene for FullScreenChange {
    fn name(&self) -> &'static str {
        "full-screen-change"
    }

    fn decided(&self) -> &'static str {
        "the equality filter's floor, 1.46x; `natural` worth 5.4% and `shortest` nothing (arch 08)"
    }

    fn build(&mut self, screen: &mut Screen) {
        self.layer = Some(base(screen));
    }

    fn step(&mut self, screen: &mut Screen, t: u32) -> u32 {
        let id = self.layer.expect("build ran");
        let style = Style::new().fg(Color::indexed((t % 16) as u8));
        let mut view = screen.layers().view(id).expect("the layer is still there");
        for y in 0..H as i32 {
            view.text(0, y, &self.row, style);
        }
        W as u32 * H as u32
    }

    fn iters(&self) -> u32 {
        20
    }
}

// ---------------------------------------------------------------------------------------------
// 11. Every cell a distinct style
// ---------------------------------------------------------------------------------------------

/// The adversarial wire scene: 24 000 cells, no two sharing a style word.
///
/// Spec §14 records 805 657 bytes and 286 ms on a 4 MB/s link, which is what set the
/// synchronised-output time limit. The differential SGR has nothing to elide here by construction.
struct EveryCellADistinctStyle {
    layer: Option<LayerId>,
}

impl EveryCellADistinctStyle {
    fn new() -> EveryCellADistinctStyle {
        EveryCellADistinctStyle { layer: None }
    }

    /// Distinct for every `i` below 24 000, because 179 * 24 000 stays inside the 24-bit space.
    fn colour(i: u32, t: u32) -> Color {
        let v = i.wrapping_mul(179).wrapping_add(t.wrapping_mul(7));
        Color::rgb((v >> 16) as u8, (v >> 8) as u8, v as u8)
    }
}

impl Scene for EveryCellADistinctStyle {
    fn name(&self) -> &'static str {
        "every-cell-a-distinct-style"
    }

    fn decided(&self) -> &'static str {
        "805 657 bytes, 286 ms on a 4 MB/s link — the synchronised-output time limit (arch 08)"
    }

    fn build(&mut self, screen: &mut Screen) {
        self.layer = Some(base(screen));
    }

    fn step(&mut self, screen: &mut Screen, t: u32) -> u32 {
        let id = self.layer.expect("build ran");
        let mut view = screen.layers().view(id).expect("the layer is still there");
        for y in 0..H as u32 {
            for x in 0..W as u32 {
                let i = y * W as u32 + x;
                view.text(
                    x as i32,
                    y as i32,
                    "#",
                    Style::new().fg(EveryCellADistinctStyle::colour(i, t)),
                );
            }
        }
        W as u32 * H as u32
    }

    fn iters(&self) -> u32 {
        5
    }
}

// ---------------------------------------------------------------------------------------------
// 12. A hyperlinked page under an animating operator
// ---------------------------------------------------------------------------------------------

/// **Red on purpose.** The one scene with no measurement behind it.
///
/// It is the only measured shape that grows a handle table without bound, and it decides table
/// lifetime the way the sparse chart decided damage: table growth, the sweep, `repaint` and the
/// memo, all at once. It needs two mechanisms that do not exist yet, and it is on the list anyway —
/// a scene quietly dropped until its mechanism arrives is a scene nobody puts back.
struct HyperlinkedPageUnderAnOperator;

impl Scene for HyperlinkedPageUnderAnOperator {
    fn name(&self) -> &'static str {
        "hyperlinked-page-under-an-animating-operator"
    }

    fn decided(&self) -> &'static str {
        "table growth, the sweep, `repaint` and the memo, all at once — nothing yet (arch 16)"
    }

    fn status(&self) -> State {
        State::Red {
            inverted_by: "impl 08",
            why: "impl 08 owns this row and is where it gets its measurement; the extended-style \
                  bit and the link table it stands on landed with impl 07, and what is still \
                  missing is the operator layer with its Mix (impl 12) — which is what makes the \
                  table grow without bound in the first place",
        }
    }

    fn build(&mut self, _screen: &mut Screen) {}

    fn step(&mut self, _screen: &mut Screen, _t: u32) -> u32 {
        0
    }

    /// Never read: nothing drives a red scene. It is here because the trait is the list's shape,
    /// and a scene that could opt out of part of it would be half on the list.
    fn iters(&self) -> u32 {
        1
    }
}
