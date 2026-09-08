//! **The file preview pane's three screens: a re-sort, a streamed directory and twenty selections.**
//!
//! This is the screen the three preview-pane scenes are scenes
//! *of*, and it takes the shape every other one on this map has — [`crate::dense`],
//! [`crate::listing`], [`crate::grid`], [`crate::forest`], [`crate::area`], [`crate::accordion`],
//! [`crate::document`], [`crate::clusters`], [`crate::popup`], [`crate::series`], [`crate::wheel`]
//! and [`crate::picture`]: one [`Build`] whose fields are the arms, so a comparison is the same
//! screen with one thing changed.
//!
//! | scene | what it decides |
//! |---|---|
//! | twenty selections through a directory of photographs | the crossover: **1 picture against 20**, and the four offset spellings |
//! | a directory streamed in seven batches under a sort | the dedupe key, **with no user in it** |
//! | a re-sort under a preview pane, 200 files | the memo key: **wrong on 100 of 100 frames**, from one line of application code |
//!
//! # The door nothing else opens
//!
//! A preview pane's question is a **file**, and every natural way to name it names a **position**.
//! That is the sixth arrival of the memo-key rule and the only one whose trigger is not a gesture:
//! [`resort`] plays one re-sort — one line of application code, no keystroke — and the
//! position-keyed pane is wrong on **100 of 100 frames afterwards**. Both arms call
//! [`Task::request`] on every frame, **103 either way**; what differs is
//! [`Worker::asked`] — 1 against 2 — *because the question still matches, so nothing posts, so
//! nothing wakes, so no frame corrects it.*
//!
//! [`batches`] is the same hole with **no user in it**, which is why it is a separate scene rather
//! than a second gesture on the first: a directory still being listed splices batches into a sorted
//! order and the cursor's file changes under it.
//!
//! # The offset belongs to neither side, and there are five spellings
//!
//! [`Offset`] is the table as an enum, on a 4 000-row file, an 800-row file and a 74-row
//! viewport. Four of the five are defects and each is a **different** one — that is the whole
//! reason the row is a table and not a sentence — and the fifth, the per-file map, is right in both
//! directions and refused on release: [`map_bytes`] against [`SLOT_BYTES`].
//!
//! # What tears, and why it tears without a race
//!
//! [`Task::take`] is **destructive**: a landing is taken once. A view that asks for it wherever it
//! happens to need it therefore hands the *first* caller the answer and the second `None`, on the
//! same frame, with no thread having done anything at all. [`Taken`] is that axis and
//! [`torn`] is the count — R18 the unenforced ordering as **20 torn frames of 20** against **0**,
//! with a status row on each side of the pane and the tear read back off the drawn surface.
//!
//! # The screen partitions and the headline number does not
//!
//! The unclamped offset is priced at *1 650 writes against 4 166*, and a screen that writes every
//! cell of its rectangle — the rule, which every component in this crate obeys — writes
//! [`CELLS`] either way. So this screen reports **two** numbers: [`Shape::writes`], which is 24 000
//! on every arm because the partition holds, and [`Drawn::content_writes`], which is the body's own
//! document cells and is **0** when the body draws nothing at all. The pair is printed
//! beside them rather than engineered into them, and one thing is read *out* of it: 4 166 − 1 650 is
//! **2 516 = 74 × 34**, so the prototype's document was 34 columns wide, which is where
//! [`LINE_COLUMNS`] comes from.

use std::cell::Cell;
use std::collections::HashMap;
use std::fmt::Write as _;

use vitui_runtime::ctx::Driver;
use vitui_runtime::data::Revision;
use vitui_runtime::theme::Role;
use vitui_runtime::work::{Cancel, Task, Worker};
use vitui_runtime::{Ctx, Rect};

use crate::counters::Tally;
use crate::files::{
    self, Extent, Landed, PaneOpts, PaneShape, PaneState, Preview, Reset, defective::Mapped,
};
use crate::ink::{Direct, Ink};
use crate::obligations::Verdict;
use crate::order::Keyed;
use crate::runner::{Canvas, Pen};

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The screen's width. The full-screen class, and the width the 61.0% is a percentage of.
pub const W: u16 = 300;

/// The screen's height.
pub const H: u16 = 80;

/// How many cells that is.
pub const CELLS: u32 = W as u32 * H as u32;

/// **The preview body's width.** The original's: `198 x 74 = 14 652 cells, 61.0% of the screen`.
///
/// It is the pane's **viewport** and not the pane's rectangle — see [`PANE_W`], which is the
/// finding underneath: a preview pane is a positive case for bars-reserved, so it costs one gutter
/// on each axis, and the 198 x 74 is what is left after them.
pub const BODY_W: u16 = 198;

/// The preview body's height, which is also the viewport the offset table is written against.
pub const BODY_H: u16 = 74;

/// How many cells the body is. **14 652**, and [`body_percent`] is what is stated beside it.
pub const BODY_CELLS: u32 = BODY_W as u32 * BODY_H as u32;

/// **The pane's rectangle's width: the body plus its reserved vertical gutter.**
///
/// A preview pane is *a positive case for bars-reserved rather than a new question for it*,
/// and [`crate::files::PaneOpts`] takes that as [`Hide::Never`](crate::scroll::Hide::Never): the
/// gutter is cut whether or not there is anything to scroll, because a pane's extent is a property
/// of the **file** and a gutter that came and went would make the furniture jump every time the
/// selection moved. So the component is handed 199 x 75 and hands its line drawer 198 x 74.
pub const PANE_W: u16 = BODY_W + 1;

/// The pane's rectangle's height. See [`PANE_W`].
pub const PANE_H: u16 = BODY_H + 1;

/// The file list's width. `W - PANE_W - 1`, the one being the vertical rule between them.
pub const LIST_W: u16 = W - PANE_W - 1;

/// How many rows of chrome sit above the pane: a title, the top status and a rule.
pub const CHROME_ROWS: u16 = 3;

/// How many sit below it: a rule and the bottom status. A status row on **each** side of the pane
/// is the arrangement for the torn-frame case, and it is what [`torn`] reads back.
pub const FOOT_ROWS: u16 = 2;

/// **The document's line width, read out of the pair.**
///
/// The unclamped offset is priced at *1 650 writes against 4 166*. The difference is the body's own
/// content — **2 516** — and 2 516 is **74 × 34** exactly, over a 74-row viewport. So the
/// prototype's preview document was 34 columns wide and this one is too. The number is derived
/// rather than chosen, which is the difference between reproducing a figure and engineering one.
pub const LINE_COLUMNS: u16 = 34;

/// What *the body draws nothing at all* is, as this screen's own counter: 74 rows of
/// [`LINE_COLUMNS`].
pub const CONTENT_WRITES: u64 = BODY_H as u64 * LINE_COLUMNS as u64;

/// The pair for the same fact, on a screen that did not partition its rectangle.
pub const SECTION_15_WRITES: [u64; 2] = [1_650, 4_166];

/// The whole screen.
pub fn whole() -> Rect {
    Rect::new(0, 0, W, H)
}

/// The file list's rectangle.
pub fn list_area() -> Rect {
    Rect::new(0, i32::from(CHROME_ROWS), LIST_W, PANE_H)
}

/// The one-column vertical rule between the list and the pane. `file_preview_pane` declares
/// [`Glyph::VLine`](vitui_runtime::Glyph::VLine) in [`crate::INVENTORY`]; this is the column it is
/// drawn in.
pub fn rule_area() -> Rect {
    Rect::new(i32::from(LIST_W), i32::from(CHROME_ROWS), 1, PANE_H)
}

/// **The rectangle the component is handed.** 199 x 75 — see [`PANE_W`].
pub fn pane_area() -> Rect {
    Rect::new(
        i32::from(LIST_W) + 1,
        i32::from(CHROME_ROWS),
        PANE_W,
        PANE_H,
    )
}

/// **The preview body's rectangle**, which is the viewport the pane hands its line drawer: 198 x 74,
/// the original's. [`pane_area`] less one reserved gutter on each axis.
pub fn body_area() -> Rect {
    Rect::new(
        i32::from(LIST_W) + 1,
        i32::from(CHROME_ROWS),
        BODY_W,
        BODY_H,
    )
}

/// The row the top status is written on.
pub const TOP_STATUS_Y: i32 = 1;

/// The row the rule below the pane is written on.
pub const FOOT_RULE_Y: i32 = (H - FOOT_ROWS) as i32;

/// The row the bottom status is written on. **A status row on each side of the pane** is what makes
/// a torn frame visible: one value, read twice, printed twice.
pub const BOTTOM_STATUS_Y: i32 = (H - 1) as i32;

/// What the body is as a percentage of the screen, to two places. The recorded figure is **61.0%**.
pub fn body_percent() -> f64 {
    f64::from(BODY_CELLS) * 100.0 / f64::from(CELLS)
}

// ── the axes of a build ──────────────────────────────────────────────────────────────────────────

/// **What the pane names its question with.**
///
/// The whole of the memo-key rule, arriving through the one door where the natural spelling is the
/// wrong one: a pane's question is a *file* and a cursor is a *position*.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    /// The cursor's index in the ordered listing. **The defect**, and the one anybody writes first.
    Position,
    /// The file's own identity. What ships.
    Identity,
}

/// **Where the offset goes when the selection changes.** The five-row table.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Offset {
    /// Left where it was, and not clamped. **The body draws nothing at all**, because a landing
    /// *is* a shrink — from another thread, for the first time.
    Unclamped,
    /// Left where it was, clamped to the new file. **The new file opens at its last page.**
    ClampedKept,
    /// Zeroed when the question is asked. **Scrolls the previous file, still on screen, to its
    /// top** for the length of the decode.
    ResetOnRequest,
    /// Zeroed when the answer lands. Right on arrival, forgets on return — the trade rather than a
    /// defect, and what ships.
    ResetOnLanding,
    /// Remembered per file. Right in both directions, and **nothing releases it** — see
    /// [`map_bytes`].
    PerFileMap,
}

impl Offset {
    /// Every spelling, which is what the report iterates.
    pub const ALL: [Offset; 5] = [
        Offset::Unclamped,
        Offset::ClampedKept,
        Offset::ResetOnRequest,
        Offset::ResetOnLanding,
        Offset::PerFileMap,
    ];

    /// The name the table gives it.
    pub const fn name(self) -> &'static str {
        match self {
            Offset::Unclamped => "unclamped",
            Offset::ClampedKept => "clamped and kept",
            Offset::ResetOnRequest => "reset on the request",
            Offset::ResetOnLanding => "reset on the landing",
            Offset::PerFileMap => "a per-file map",
        }
    }
}

/// **Where the landing is taken.** R18 the unenforced ordering, as an axis.
///
/// It is [`crate::files::Taken`] and not a second enum of the same name: this crate's own rule is
/// that two types with one name make every mismatch read *expected `Taken`, found `Taken`*, and the
/// scene's axis and the component's field are the same fact. `crate::preview::Offset` **is** the
/// scene's own, because the five spellings are not the component's four — one of them is on the
/// extent axis and one is a structure.
pub use crate::files::{Bump, Taken};

/// **How fast the selection repeats, against how long a decode takes.**
///
/// The crossover is stated as a relation and not as a number of milliseconds: *1 picture when the
/// key repeats faster than the decode, 20 pictures at a 30 ms repeat when it does not*. The arms
/// are named for the relation for that reason — the ticket's own paraphrase (*a 30 ms key repeat
/// and a slower one*) reads the crossover the other way round, and the relation is the authority.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Repeat {
    /// A 30 ms repeat over a decode that finishes inside it. **Every selection lands.**
    SlowerThanTheDecode,
    /// A repeat inside the decode. Every question but the last is cancelled: **one picture.**
    FasterThanTheDecode,
}

impl Repeat {
    /// Milliseconds between selections.
    pub const fn repeat_ms(self) -> u32 {
        match self {
            Repeat::SlowerThanTheDecode => 30,
            Repeat::FasterThanTheDecode => 8,
        }
    }

    /// Milliseconds a decode takes.
    pub const fn decode_ms(self) -> u32 {
        match self {
            Repeat::SlowerThanTheDecode => 12,
            Repeat::FasterThanTheDecode => 30,
        }
    }
}

/// **One screen, with the four arms as fields.** A comparison is one field changed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Build {
    /// What the question is named with.
    pub key: Key,
    /// Where the offset goes.
    pub offset: Offset,
    /// Where the landing is taken.
    pub taken: Taken,
    /// When the revision advances.
    pub bump: Bump,
}

impl Build {
    /// What ships: an identity key, the offset reset on the landing, the answer taken once, the
    /// revision advanced on a landing.
    pub const fn correct() -> Build {
        Build {
            key: Key::Identity,
            offset: Offset::ResetOnLanding,
            taken: Taken::AtTheTopOfTheView,
            bump: Bump::OnTheLanding,
        }
    }

    /// The same screen, with the revision advanced somewhere else.
    pub const fn bump(self, bump: Bump) -> Build {
        Build { bump, ..self }
    }

    /// The same screen, keyed differently.
    pub const fn keyed(self, key: Key) -> Build {
        Build { key, ..self }
    }

    /// The same screen, with the offset spelled differently.
    pub const fn offset(self, offset: Offset) -> Build {
        Build { offset, ..self }
    }

    /// The same screen, with the landing taken somewhere else.
    pub const fn taken(self, taken: Taken) -> Build {
        Build { taken, ..self }
    }
}

// ── the directory ────────────────────────────────────────────────────────────────────────────────

/// **What a file's document is made of**, which is what decides how wide it is and what its cells
/// cost.
///
/// The join with the media family: a preview pane is the one component here whose body may be *either*
/// text or a picture, and both are priced — a text document is [`LINE_COLUMNS`] wide and its cells
/// are the theme's, a photograph is as wide as the viewport and every one of its cells is outside
/// the theme.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Kind {
    /// A text document, [`LINE_COLUMNS`] columns wide.
    #[default]
    Text,
    /// A photograph, as wide as the body and one [`Theme::custom`](vitui_runtime::Theme::custom) a
    /// cell. *one `Theme::custom` per cell of the pane*.
    Photograph,
}

/// One entry of the listing.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct File {
    /// **The file's own identity**, which is what an identity key is keyed on. A path hash in an
    /// application; a stated number here, because a hash of a synthetic name is a hash of a
    /// synthetic name.
    pub id: u64,
    /// Its name, which is what the list draws.
    pub name: String,
    /// How many lines its document has.
    pub rows: u32,
    /// **Its rank in the by-size order**, which is what makes [`Order::BySize`] a sort rather than a
    /// stated permutation.
    pub size: u64,
    /// What its document is made of.
    pub kind: Kind,
}

/// How many files the two 200-file scenes list. The count for both.
pub const FILES: usize = 200;

/// **Where the cursor sits.** Not one of [`FIXED`], because a re-sort that left the cursor's own
/// file alone would be a re-sort with nothing to decide.
pub const CURSOR: usize = 42;

/// **The three positions a re-sort leaves alone**, which is what makes the moved count 197 and not
/// 200. The claim is *197 of 200 positions move*; three fixed points is what states it.
pub const FIXED: [usize; 3] = [0, 100, 199];

/// Which order the listing is in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Order {
    /// By name, which for `f000..f199` is the order they were created in.
    ByName,
    /// By size. **One line of application code**, and the whole of the re-sort scene's gesture.
    BySize,
}

/// The by-size order, as the file index at each position.
///
/// A rotation of the 197 movable positions by one, which is the smallest permutation with exactly
/// [`FIXED`]`.len()` fixed points: every movable position shows the next movable file, and a
/// rotation by one over a run longer than one has no fixed point of its own.
fn by_size_order() -> Vec<usize> {
    let mut order: Vec<usize> = (0..FILES).collect();
    let movable: Vec<usize> = (0..FILES).filter(|i| !FIXED.contains(i)).collect();
    for (k, &position) in movable.iter().enumerate() {
        order[position] = movable[(k + 1) % movable.len()];
    }
    order
}

/// **The directory, as files rather than as a permutation.**
///
/// [`File::size`] is each file's rank in the by-size order, so sorting the listing by size really is
/// a sort — the permutation is a property of the data and not an argument the fixture passes to
/// itself. The order itself is a rotation of the movable positions by one, which is the smallest
/// permutation with exactly [`FIXED`]`.len()` fixed points.
pub fn directory() -> Vec<File> {
    let ranks = by_size_order();
    let mut size = vec![0u64; FILES];
    for (position, &file) in ranks.iter().enumerate() {
        size[file] = position as u64;
    }
    (0..FILES)
        .map(|i| File {
            id: 0x9E37_79B9_0000_0000 ^ i as u64,
            name: format!("f{i:03}"),
            rows: 120 + (i as u32 % 7) * 13,
            size: size[i],
            kind: Kind::Text,
        })
        .collect()
}

/// The listing in `order`.
pub fn listing(order: Order) -> Vec<File> {
    let mut files = directory();
    match order {
        Order::ByName => files.sort_by(|a, b| a.name.cmp(&b.name)),
        Order::BySize => files.sort_by_key(|f| f.size),
    }
    files
}

/// **How many positions hold a different file in `b` than in `a`.** *197 of 200*.
///
/// # Panics
///
/// Panics on listings of different lengths, which have no position-for-position comparison to make.
pub fn positions_moved(a: &[File], b: &[File]) -> u32 {
    assert_eq!(a.len(), b.len(), "two listings of different lengths");
    a.iter().zip(b).filter(|(x, y)| x.id != y.id).count() as u32
}

// ── the document ─────────────────────────────────────────────────────────────────────────────────

/// **What a decode produces**: the file's identity and its lines.
///
/// It carries the id rather than only the text, because [`Offset::PerFileMap`] has to key on
/// something the landing knows and a landing that only carried text could not be filed.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Doc {
    /// Which file this is the document of.
    pub id: u64,
    /// Its name, which is what both status rows print.
    pub name: String,
    /// How many lines it has.
    pub rows: u32,
    /// What it is made of.
    pub kind: Kind,
}

/// **What makes a `Doc` an answer**: the identity of the question it answers, and how large it is.
///
/// The identity is the eight bytes on the payload, and the equality against it is the
/// pane's — [`PaneState::land`] drops an answer that fails it without a cell written. The extent is
/// the precondition as a field of the answer, which is what makes a preview pane a positive case
/// for bars-reserved.
impl Preview for Doc {
    fn shows(&self) -> u64 {
        self.id
    }

    fn extent(&self) -> (u32, u32) {
        let cols = match self.kind {
            Kind::Text => u32::from(LINE_COLUMNS),
            // **As wide as the body**, which is what makes a photograph 198 x 74 = 14 652 cells and
            // The custom count a partition of the pane's viewport rather than a coincidence.
            Kind::Photograph => u32::from(BODY_W),
        };
        (cols, self.rows)
    }
}

impl Doc {
    /// Line `i`, exactly [`LINE_COLUMNS`] columns wide.
    pub fn line(&self, i: u32) -> String {
        let mut s = format!("{}:{i:05} ", self.name);
        while s.len() < usize::from(LINE_COLUMNS) {
            s.push('.');
        }
        s.truncate(usize::from(LINE_COLUMNS));
        s
    }

    /// The largest offset that still shows a row. `rows - 74`, floored at zero.
    pub fn max_offset(&self) -> u32 {
        self.rows.saturating_sub(u32::from(BODY_H))
    }
}

/// The 4 000-row file the offset table is written against.
pub fn long_file() -> File {
    File {
        id: 0xA1,
        name: String::from("long"),
        rows: 4_000,
        size: 0,
        kind: Kind::Text,
    }
}

/// The 800-row file beside it.
pub fn short_file() -> File {
    File {
        id: 0xB2,
        name: String::from("short"),
        rows: 800,
        size: 1,
        kind: Kind::Text,
    }
}

/// The long file's last page. **3 926**, and what a user who pressed `End` is looking at.
pub const LONG_LAST_PAGE: u32 = 4_000 - BODY_H as u32;

/// The short file's last page. **726**, which is the number for *clamped and kept*.
pub const SHORT_LAST_PAGE: u32 = 800 - BODY_H as u32;

// ── one frame ────────────────────────────────────────────────────────────────────────────────────

/// **What one frame of the pane drew**, as counters and as the two status rows.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Drawn {
    /// **Cells of the body carrying the document.** The counter that is not blind: zero when the
    /// body draws nothing at all.
    pub content_writes: u64,
    /// **[`Theme::custom`](vitui_runtime::Theme::custom) calls.** Zero for a text document and one a
    /// cell for a photograph — *one per cell of the pane*.
    pub customs: u64,
    /// How many document rows the body drew.
    pub body_rows: u32,
    /// The document row the body opened at, or `None` when it drew none.
    pub first_row: Option<u32>,
    /// Which file the body is showing, or `None` before the first landing.
    pub showing: Option<String>,
    /// What the top status row says.
    pub top_status: String,
    /// What the bottom status row says.
    pub bottom_status: String,
}

impl Drawn {
    /// Whether the two status rows disagree about one value. **A torn frame.**
    pub fn torn(&self) -> bool {
        self.top_status != self.bottom_status
    }
}

/// The status line, from what the pane is showing.
fn status(showing: Option<&str>) -> String {
    match showing {
        Some(name) => format!("preview: {name}"),
        None => String::from("preview: -"),
    }
}

// ── the pane ─────────────────────────────────────────────────────────────────────────────────────

/// **Which pane the screen is holding**, because one of the five offset spellings is a
/// *structure* and not a field.
///
/// Four of the five are [`crate::files::PaneShape`] with one field changed; the per-file map is
/// [`crate::files::defective::Mapped`], a wrapper the shipped pane has nowhere to put — which is
/// the release argument as a type rather than as a paragraph.
enum Pane {
    /// The shipped pane, or one of the three spellings that are one field away from it.
    Plain(PaneState<Doc>),
    /// The fifth spelling: a slot per file, and nothing releases it.
    Mapped(Mapped<Doc>),
}

impl Pane {
    /// The pane underneath, whichever arm this is.
    fn state(&self) -> &PaneState<Doc> {
        match self {
            Pane::Plain(p) => p,
            Pane::Mapped(m) => &m.pane,
        }
    }

    /// The pane underneath, to draw with.
    fn state_mut(&mut self) -> &mut PaneState<Doc> {
        match self {
            Pane::Plain(p) => p,
            Pane::Mapped(m) => &mut m.pane,
        }
    }

    /// Take the landing, at the top of the view. The map's arm puts the offset back afterwards.
    fn land(&mut self, task: &Task<Doc>) -> Landed {
        match self {
            Pane::Plain(p) => p.land(task),
            Pane::Mapped(m) => m.land(task),
        }
    }

    /// File the offset the pane is about to leave, on the arm that has anywhere to put one.
    fn filing(&mut self, leaving: u64) {
        if let Pane::Mapped(m) = self {
            m.filing(leaving);
        }
    }
}

/// **The four axes of a [`Build`], as the shape the component is spelled with.**
///
/// The mapping is where the table stops being five rows on one axis. Four of the spellings move
/// [`Reset`]; *unclamped* moves [`Extent`] instead, and the reason is measured rather than argued —
/// [`crate::scroll::scroll_area`] clamps against the extent it is handed on **every** frame, so an
/// offset cannot be left unclamped at all. The only way to *the body draws nothing* is a **stale
/// extent**, which is the same defect with its cause named: a landing is a shrink, and what shrinks
/// is the extent.
fn shape_of(build: Build) -> PaneShape {
    PaneShape {
        reset: match build.offset {
            Offset::Unclamped | Offset::ClampedKept | Offset::PerFileMap => Reset::Never,
            Offset::ResetOnRequest => Reset::OnTheRequest,
            Offset::ResetOnLanding => Reset::OnTheLanding,
        },
        extent: match build.offset {
            Offset::Unclamped => Extent::Unbounded,
            _ => Extent::OfTheAnswer,
        },
        bump: build.bump,
        taken: build.taken,
    }
}

/// **The screen, as a thing that can be handed a listing and a cursor and asked for a frame.**
///
/// It holds a [`Worker::queueing`] rather than a resident one, which is [`Worker::queueing`]'s own
/// argument: *a gate over staleness arithmetic is a gate over an order, and the only thing a real
/// thread contributes to it is that order.* Every count on this screen is a count over a schedule
/// the caller wrote down.
///
/// **The worker, the task and the pane's storage are all the screen's**, which is the second
/// refusal as an arrangement: a job's lifetime is the question's, a memo's is the data's, and
/// neither is the widget's. [`sweeps`] is what that costs when it is got wrong.
pub struct Screen {
    /// Which arms are standing.
    pub build: Build,
    driver: Driver,
    worker: Worker,
    task: Task<Doc>,
    pane: Pane,
    /// A memo over the document, standing in for the highlighter: it re-folds the whole file, and
    /// what it keys on is [`PaneState::revision`].
    ///
    /// [`crate::order::Keyed`] and not [`vitui_runtime::data::Memo`], which is this crate's own
    /// standing rule with a source scan behind it: a memo whose key is a bare `Revision` has nowhere
    /// to put a second input, and *the inputs that are not data are the ones that get forgotten*.
    highlighter: Keyed<Revision, usize>,
    requests: u32,
    frames: u32,
    forwarded: u32,
    ran: usize,
    /// Units of decode that ran while the app thread was inside `Driver::frame`. **Zero.**
    in_the_frame: u64,
    /// Customs spent, over every frame.
    customs: u64,
    /// Whether the decode answers a question nobody asked. [`mismatched`]'s axis.
    lying: bool,
    /// Whether the decode is called from the view. [`DecodeAt::InTheView`].
    in_the_view: bool,
}

impl Screen {
    /// A screen with nothing asked and nothing shown.
    pub fn open(build: Build) -> Screen {
        let worker = Worker::queueing();
        let task = Task::new(&worker);
        let pane = match build.offset {
            Offset::PerFileMap => Pane::Mapped(Mapped::new()),
            _ => Pane::Plain(PaneState::shaped(shape_of(build))),
        };
        Screen {
            build,
            driver: Driver::headless(W, H).expect("a sink cannot fail to attach"),
            worker,
            task,
            pane,
            highlighter: Keyed::new(),
            requests: 0,
            frames: 0,
            forwarded: 0,
            ran: 0,
            in_the_frame: 0,
            customs: 0,
            lying: false,
            in_the_view: false,
        }
    }

    /// How many questions the screen has handed the pane. **One a frame, on every arm.**
    pub fn requests(&self) -> u32 {
        self.requests
    }

    /// How many frames it has drawn.
    ///
    /// Counted in the draw and not derived from [`Screen::requests`], so that *the pane asks on
    /// every frame* is a fact two counters agree on rather than one counter written twice.
    pub fn frames(&self) -> u32 {
        self.frames
    }

    /// **How many of those questions the component actually forwarded**, read off
    /// [`Task::asking`] after the pane has run.
    ///
    /// The third counter, and it is the one that makes the other two mean something: a screen that
    /// counted the questions it *handed over* would report 103 for a component that asked none.
    pub fn forwarded(&self) -> u32 {
        self.forwarded
    }

    /// How many questions actually reached the worker. **The counter that separates the two keys.**
    pub fn spawns(&self) -> u64 {
        self.worker.asked()
    }

    /// How many answers the pane refused on [`Preview::shows`] — a question nobody asked.
    pub fn refused(&self) -> u64 {
        self.pane.state().refused()
    }

    /// How many answers landed.
    pub fn landings(&self) -> u64 {
        self.pane.state().landings()
    }

    /// **How many times the highlighter re-folded the whole file.** *119 against 20*.
    pub fn recomputes(&self) -> u32 {
        self.highlighter.recomputes
    }

    /// **Decode units that ran while the app thread was inside its frame.** The requirement 9, as
    /// a count — and the name is the measurement rather than a paraphrase of it.
    ///
    /// # It is not *on the app thread*, and the difference is the instrument's
    ///
    /// This screen holds a [`Worker::queueing`], which answers its questions **inline on the calling
    /// thread** when [`Screen::settle`] says so — that is `Worker::queueing`'s whole argument, *a
    /// gate over staleness arithmetic is a gate over an order, and the only thing a real thread
    /// contributes to it is that order*. So the units a settle spends are spent on the app thread,
    /// and what this counter says is that **none of them is spent inside a frame**: the pane never
    /// calls the job, and the only thing that can is the caller's own schedule.
    ///
    /// The stronger claim — *0 on the app thread, full stop* — is true by construction under
    /// [`Worker::hire`](vitui_runtime::work::Worker::hire) and is not reachable from a deterministic
    /// gate, because a gate that hired a thread would be measuring the thread rather than the
    /// design. `crates/vitui-apps/examples/browse.rs` is where a real worker runs it.
    pub fn decode_units_in_the_frame(&self) -> u64 {
        self.in_the_frame
    }

    /// How many [`Theme::custom`](vitui_runtime::Theme::custom) calls the screen has spent.
    pub fn customs(&self) -> u64 {
        self.customs
    }

    /// The document offset the body is drawing from.
    pub fn offset(&self) -> u32 {
        self.pane.state().offset()
    }

    /// Move the offset, the way a `PageDown` would.
    pub fn scroll_to(&mut self, offset: u32) {
        self.pane.state_mut().scroll_to(offset);
    }

    /// Which file the pane is showing.
    pub fn showing(&self) -> Option<&Doc> {
        self.pane.state().showing()
    }

    /// How many entries the refused per-file map is holding, on the arm that has one.
    pub fn mapped_entries(&self) -> usize {
        match &self.pane {
            Pane::Mapped(m) => m.entries(),
            Pane::Plain(_) => 0,
        }
    }

    /// Answer the question asked `index`-th, on this thread. The decode finishing, for a caller
    /// holding a schedule of its own.
    ///
    /// **A caller drives the queue with this or with [`Screen::settle`] and not with both.** It
    /// advances the settle cursor past `index`, so a question skipped over stays unanswered — which
    /// is what a caller running its own clock wants and would be a hole under a caller alternating
    /// the two.
    ///
    /// # Panics
    ///
    /// Panics on an index that was never asked or has already been answered — [`Worker::run`]'s own
    /// contract, and for its reason.
    pub fn run_job(&mut self, index: usize) -> bool {
        self.ran = self.ran.max(index + 1);
        self.worker.run(index)
    }

    /// **Answer every outstanding question**, on this thread, in the order it was asked.
    ///
    /// The decode finishing. A question that was superseded lands nothing, which is
    /// [`Task::request`]'s own bracket and not this function's arithmetic.
    pub fn settle(&mut self) {
        while (self.ran as u64) < self.worker.asked() {
            let _landed = self.worker.run(self.ran);
            self.ran += 1;
        }
    }

    /// **One frame**, drawn through `ink`.
    ///
    /// The order is fixed: the landing is taken at the top of the view, and then the screen is drawn
    /// — the pane asks its question inside its own draw, which is where a component's question
    /// belongs. [`Taken::InsideTheDraw`] is the same frame with the first of those moved into the
    /// second, which is the whole of the torn-frame case.
    ///
    /// # Panics
    ///
    /// Panics when `cursor` is past the end of `listing`, which is a schedule that did not happen.
    pub fn frame_into<I: Ink>(&mut self, ink: &mut I, listing: &[File], cursor: usize) -> Drawn {
        assert!(cursor < listing.len(), "the cursor is past the listing");
        let file = listing[cursor].clone();
        let key = match self.build.key {
            Key::Position => cursor as u64,
            Key::Identity => file.id,
        };

        if self.build.taken == Taken::AtTheTopOfTheView {
            self.pane.land(&self.task);
        }

        // **The map files the offset it is about to leave**, and it has to be the caller who does
        // it: there is no frame on which the pane knows a question is *about to* change, because
        // `Task::request` tells it afterwards. That is the same sentence as the refusal.
        if self.task.asking() != Some(key)
            && let Some(doc) = self.pane.state().showing()
        {
            let leaving = doc.id;
            self.pane.filing(leaving);
        }

        self.requests += 1;
        let drawn = self.draw(ink, listing, cursor, key);
        if self.task.asking() == Some(key) {
            self.forwarded += 1;
        }
        drawn
    }

    /// One frame through [`Direct`], which is the path a component takes.
    pub fn frame(&mut self, listing: &[File], cursor: usize) -> Drawn {
        self.frame_into(&mut Direct, listing, cursor)
    }

    /// **One frame whose decode answers a question nobody asked.**
    ///
    /// The generation is current, so [`Task::take`] hands the answer over; the payload is another
    /// file's, and the equality that catches it is the pane's. See [`mismatched`].
    pub fn frame_lying(&mut self, listing: &[File], cursor: usize, lying: bool) -> Drawn {
        self.lying = lying;
        let drawn = self.frame_into(&mut Direct, listing, cursor);
        self.lying = false;
        drawn
    }

    /// **One frame that decodes in the view**, with the answer still arriving. See [`decoded`].
    pub fn frame_decoding_in_the_view(&mut self, listing: &[File], cursor: usize) {
        self.in_the_view = true;
        let _ = self.frame_into(&mut Direct, listing, cursor);
        self.in_the_view = false;
    }

    /// The draw itself. Split out because `Driver::frame` borrows the driver and the body needs the
    /// rest of the screen — [`crate::picture::Session`]'s own arrangement, for its reason.
    fn draw<I: Ink>(&mut self, ink: &mut I, listing: &[File], cursor: usize, key: u64) -> Drawn {
        self.frames += 1;
        let Screen {
            driver,
            task,
            pane,
            highlighter,
            in_the_frame,
            ..
        } = self;
        let mut content_writes = 0u64;
        let mut customs = 0u64;
        let mut census = crate::media::Census::default();
        let mut body_rows = 0u32;
        let mut first_row = None;
        let mut top_status = String::new();
        let mut bottom_status = String::new();
        let file = listing[cursor].clone();
        let before = decode_units();
        let lying = self.lying;
        let in_the_view = self.in_the_view;

        driver.frame(|cx| {
            if in_the_view {
                // **The same decode, called from the view.** Requirement 9's other arm: the units
                // are the same units and the thread is the app thread.
                spend_decode_units(u64::from(file.rows));
            }
            let theme = cx.theme();
            let body_paint = theme.paint(Role::Body);
            let dim = theme.paint(Role::Dim);
            let border = theme.paint(Role::Border);
            let selection = theme.paint(Role::Selection);
            let title = theme.paint(Role::Title);

            // The top chrome: a title, the status, and a rule.
            ink.pad_to(
                cx,
                0,
                0,
                "vitui preview pane   up/down select   q quit",
                W,
                title,
            );
            // **The first of the two readers**, and it reads the pane's state rather than the
            // outbox. Under `Taken::InsideTheDraw` the component below takes the landing after this
            // row is written and before the second one is, so the two rows of one frame are drawn
            // from two versions of one value — with no thread having done anything at all.
            top_status = status(pane.state().showing().map(|d| d.name.as_str()));
            ink.pad_to(cx, 0, TOP_STATUS_Y, &top_status, W, dim);
            ink.run(cx, 0, 2, "-", W, border);

            // The listing.
            let list = list_area();
            for row in 0..list.h {
                let y = list.y + i32::from(row);
                let index = usize::from(row);
                let paint = if index == cursor {
                    selection
                } else {
                    body_paint
                };
                let text = listing
                    .get(index)
                    .map(|f| format!("{index:>4}  {}", f.name))
                    .unwrap_or_default();
                ink.pad_to(cx, list.x, y, &text, list.w, paint);
            }

            // The rule between them. `file_preview_pane` declares `VLine`, and this is it.
            let rule = rule_area();
            for row in 0..rule.h {
                ink.run(cx, rule.x, rule.y + i32::from(row), "|", 1, border);
            }

            // **The component.** Every cell of `pane_area()` is its own: the viewport it hands the
            // line drawer, the two reserved gutters, the corner, and the cells past the extent.
            let id = cx.id();
            let (id_of, name, rows, kind) = (file.id, file.name.clone(), file.rows, file.kind);
            let opts = PaneOpts::default();
            let _resp = files::file_preview_pane_into(
                ink,
                cx,
                id,
                pane_area(),
                pane.state_mut(),
                task,
                files::asking(key, move |_cancel| {
                    // **The decode**, counted in units so that *0 on the app thread* is a number.
                    spend_decode_units(u64::from(rows));
                    Doc {
                        // **The lie.** A job started for the right question that answers a
                        // different one — which is not out of order and is therefore invisible to
                        // any test on the answer's arrival.
                        id: if lying { !id_of } else { id_of },
                        name,
                        rows,
                        kind,
                    }
                }),
                &opts,
                |ink: &mut I, cx: &mut Ctx<'_, '_>, row: Rect, doc: &Doc, i: u32| {
                    if first_row.is_none() {
                        first_row = Some(i);
                    }
                    body_rows += 1;
                    match doc.kind {
                        Kind::Text => {
                            let text = doc.line(i);
                            content_writes +=
                                u64::from(ink.pad_to(cx, row.x, row.y, &text, row.w, body_paint));
                        }
                        // **One custom and one verb a cell**, and it is `crate::media::picture`
                        // that spends them rather than this screen.
                        //
                        // That is the F11/F12 join `INVENTORY` already declares — `file_preview_pane`
                        // names both families — and it is also what keeps the one-palette rule at
                        // two calling lines: a picture's cells are outside the theme *by
                        // construction*, which is the stated exception, and a screen that spelled
                        // `Theme::custom` itself would be a third palette rather than a second use
                        // of the exception.
                        Kind::Photograph => {
                            let strip = Strip {
                                top: i,
                                sub: u32::from(crate::media::sub_rows(cx.theme().glyphs())),
                            };
                            let before = census.customs;
                            let _ = crate::media::picture_into(
                                ink,
                                cx,
                                row,
                                &strip,
                                &crate::media::PictureOpts::default(),
                                &mut census,
                            );
                            customs += census.customs - before;
                            content_writes += u64::from(row.w);
                        }
                    }
                },
            );

            // **The highlighter**, keyed on the pane's revision. A landing that did not happen is
            // still a drop, so a pane that bumps on every frame re-folds the whole file on every
            // frame — on a screen that is cell-identical.
            // **A highlighter with no document folds nothing**, which is why the run's first frame
            // is not one of the 119: the fold is over the file and there is not one yet.
            if let Some(doc) = pane.state().showing() {
                let rev = pane.state().revision();
                let rows = doc.rows as usize;
                let _ = highlighter.get(rev, || rows);
            }

            // The bottom chrome: a rule, then the status again.
            ink.run(cx, 0, FOOT_RULE_Y, "-", W, border);
            // **The second reader.** Under `TopOfView` the landing was taken before this frame
            // began and both rows read one value; under `InsideTheDraw` the component above has
            // taken it in between.
            bottom_status = status(pane.state().showing().map(|d| d.name.as_str()));
            ink.pad_to(cx, 0, BOTTOM_STATUS_Y, &bottom_status, W, dim);
        });

        *in_the_frame += decode_units() - before;

        self.customs += customs;
        Drawn {
            content_writes,
            customs,
            body_rows,
            first_row,
            showing: self.pane.state().showing().map(|d| d.name.clone()),
            top_status,
            bottom_status,
        }
    }
}

/// **One row of the photograph, as a [`Pixels`](crate::media::Pixels) source.**
///
/// A picture never tells its source where on the screen it landed, so a pane that draws one row at a
/// time hands each row its own strip and the strip does the offsetting. That is the *virtualisation*
/// the whole of `CONTEXT.md`'s invariant is about arriving on a picture: the frame costs the
/// rectangle and never the image.
struct Strip {
    /// Which document row this strip is.
    top: u32,
    /// How many sub-rows a cell has, read off the theme at the call site.
    ///
    /// **Read and not named**: a component names a role and a glyph and never a repertoire, and
    /// `crate::media` is the one file here excepted from that. A screen
    /// that spelled the repertoire type would be a second exception.
    sub: u32,
}

impl crate::media::Pixels for Strip {
    fn pixel(&self, x: u16, sub_y: u32) -> vitui_runtime::Rgb {
        let x = u32::from(x);
        let y = self.top * self.sub + sub_y;
        // A gradient rather than a photograph, because what is being counted is the **structure** —
        // one custom and one verb a cell, no run available at any size — and `crate::picture` has
        // already measured what a real photograph's adjacencies cost.
        vitui_runtime::Rgb::new(
            (x * 251 + y * 17) as u8,
            (y * 193 + x * 7) as u8,
            (x ^ y) as u8,
        )
    }
}

thread_local! {
    /// **How many units of decode have run on *this* thread.**
    ///
    /// Thread-local and not global, and that is the measurement rather than an implementation
    /// detail: the requirement 9 is *0 decode units **on the app thread***, so the counter has to
    /// be one the app thread can read about itself. A job holds no reference to the task that
    /// started it — which is `!Sync`, and the whole of what
    /// [`crate::files::WhyTheAutoTraitIsSync`] is about — so under a hired worker its units accrue
    /// on the worker's thread and are invisible here, which is the answer being asserted. Under
    /// [`Worker::queueing`] the schedule is the caller's and every `run` is outside a frame, so the
    /// same zero is reached by the same route.
    static DECODE_UNITS: Cell<u64> = const { Cell::new(0) };
}

/// The counter, read.
fn decode_units() -> u64 {
    DECODE_UNITS.with(Cell::get)
}

/// Spend `n` units of decode. Called from inside a job, wherever that job is running.
fn spend_decode_units(n: u64) {
    DECODE_UNITS.with(|c| c.set(c.get() + n));
}

/// **What one steady frame of the screen costs**, as counters rather than as a picture.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shape {
    /// Cells written, from the engine's own column reports. **[`CELLS`]**: the screen partitions its
    /// rectangle.
    pub writes: u64,
    /// Distinct cells touched. Equal to [`Shape::writes`], which is *no cell twice*.
    pub distinct: u64,
    /// Drawing verbs.
    pub verbs: u64,
    /// Interactive regions. **[`REGIONS`]** since components 32 — the pane's own entry and one for
    /// each of its two reserved bars — and it was zero while the screen was waiting for its
    /// subject, because nothing on it declared anything.
    pub regions: usize,
    /// Cells of the body carrying the document.
    pub content_writes: u64,
    /// [`Theme::custom`](vitui_runtime::Theme::custom) calls. **14 652 with a photograph selected**,
    /// which is one a cell of the pane's viewport.
    pub customs: u64,
}

/// **One steady frame of the pane, through a [`Tally`].**
///
/// Two frames and not one: the frame structures take their allocation on the first frame that needs
/// one and keep it, so a cold frame is not a frame.
pub fn shape(build: Build) -> Shape {
    let files = vec![long_file(), short_file()];
    let mut screen = Screen::open(build);
    screen.frame(&files, 0);
    screen.settle();
    screen.frame(&files, 0);
    let mut tally = Tally::new();
    let drawn = screen.frame_into(&mut tally, &files, 0);
    Shape {
        writes: tally.writes(),
        distinct: tally.distinct(),
        verbs: tally.verbs(),
        regions: screen.driver.inspect().hits().len(),
        content_writes: drawn.content_writes,
        customs: drawn.customs,
    }
}

/// **What the screen costs on the frame after a landing that shrank the document**, which is the
/// only frame the unclamped spelling is visible on.
///
/// [`shape`] plays one file and one landing, so every spelling is identical there — the defect is
/// a **shrink**, from another thread for the first time. This plays the long file, scrolls it to its
/// last page, selects the short one, and measures the frame the short one arrives on.
pub fn shape_after_the_shrink(spelling: Offset) -> Shape {
    let files = vec![long_file(), short_file()];
    let mut screen = Screen::open(Build::correct().offset(spelling));
    screen.frame(&files, 0);
    screen.settle();
    screen.frame(&files, 0);
    screen.scroll_to(LONG_LAST_PAGE);
    screen.frame(&files, 0);
    screen.frame(&files, 1);
    screen.settle();
    let mut tally = Tally::new();
    let drawn = screen.frame_into(&mut tally, &files, 1);
    Shape {
        writes: tally.writes(),
        distinct: tally.distinct(),
        verbs: tally.verbs(),
        regions: screen.driver.inspect().hits().len(),
        content_writes: drawn.content_writes,
        customs: drawn.customs,
    }
}

/// **The nine counters over one steady frame, with the one this crate cannot read saying so.**
///
/// `marked` is the engine's alone — `crate::counters`'s own barrier — so *0 marked on a steady
/// frame* is **recorded as unreachable and not printed as a zero**: a figure defaulted to zero is a
/// counter that prints `0` when it means nobody counted. `allocations` is handed in, because the
/// counting allocator lives in the test binaries; see [`Alone`] and `tests/budget.rs`.
pub fn counters(allocations: crate::counters::Allocations) -> crate::counters::Counters {
    let files = many(1_000);
    let mut screen = Screen::open(Build::correct());
    screen.frame(&files, 0);
    screen.settle();
    screen.frame(&files, 0);
    let mut tally = Tally::new();
    let _ = screen.frame_into(&mut tally, &files, 0);
    crate::counters::Counters::of(&screen.driver, &tally, allocations)
}

/// **The screen as a recorded surface**, after one selection has landed.
pub fn render(build: Build) -> Canvas {
    let files = vec![long_file(), short_file()];
    let mut screen = Screen::open(build);
    let mut pen = Pen::new(W, H);
    screen.frame_into(&mut pen, &files, 0);
    screen.settle();
    screen.frame_into(&mut pen, &files, 0);
    pen.into_canvas()
}

// ── scene 25: a re-sort under a preview pane ─────────────────────────────────────────────────────

/// How many frames the re-sort scene counts after the sort. *wrong on 100 of 100 frames*.
pub const WINDOW: u32 = 100;

/// How many frames it plays in all. Three before the window: the question, the answer, and the frame
/// the sort arrives on.
pub const RESORT_FRAMES: u32 = WINDOW + 3;

/// **What a re-sort under a preview pane costs**, by key.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Resort {
    /// Which key was used.
    pub key: Key,
    /// Positions whose file changed. **197 of 200**, and a property of the permutation.
    pub moved: u32,
    /// Frames played. [`RESORT_FRAMES`].
    pub frames: u32,
    /// Calls to [`Task::request`]. **Identical on both arms**, which is the point.
    pub requests: u32,
    /// Questions that reached the worker. **The only counter that moves.**
    pub spawns: u64,
    /// Frames of the window on which the body showed a file that is not the cursor's.
    pub wrong: u32,
}

/// **The re-sort screen.** 200 files, one re-sort, and a hundred frames afterwards.
///
/// The gesture is one line of application code and no keystroke at all, which is what makes this the
/// memo-key rule's sharpest arrival: there is no wake to correct it, because *the question still
/// matches, so nothing posts.*
pub fn resort(build: Build) -> Resort {
    let a = listing(Order::ByName);
    let b = listing(Order::BySize);
    let moved = positions_moved(&a, &b);
    let mut screen = Screen::open(build);

    // The question, and the answer.
    screen.frame(&a, CURSOR);
    screen.settle();
    screen.frame(&a, CURSOR);

    // The re-sort. Nobody pressed anything.
    screen.frame(&b, CURSOR);
    screen.settle();

    let want = b[CURSOR].name.clone();
    let mut wrong = 0;
    for _ in 0..WINDOW {
        let drawn = screen.frame(&b, CURSOR);
        if drawn.showing.as_deref() != Some(want.as_str()) {
            wrong += 1;
        }
    }
    Resort {
        key: build.key,
        moved,
        frames: screen.frames(),
        requests: screen.requests(),
        spawns: screen.spawns(),
        wrong,
    }
}

// ── scene 24: a directory streamed in seven batches ──────────────────────────────────────────────

/// How many files the first readdir chunk brings. The pane is already drawing when the batches
/// start, which is what makes *wrong after 7 of 7* seven and not six.
pub const SEED: usize = 32;

/// How many batches follow it. The seven.
pub const BATCHES: usize = 7;

/// How many files each batch brings. `SEED + BATCHES * BATCH == FILES`.
pub const BATCH: usize = (FILES - SEED) / BATCHES;

/// Where the cursor sits while a directory is being listed. Inside the seed, so that there is
/// something to be wrong about from the first batch.
pub const BATCH_CURSOR: usize = 20;

/// **What a streamed directory costs**, by key.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Batches {
    /// Which key was used.
    pub key: Key,
    /// Batches delivered.
    pub batches: usize,
    /// Batches after which the settled pane was still showing the wrong file.
    pub wrong_after: usize,
    /// Frames on which the body showed a file that is not the cursor's.
    pub wrong_frames: u32,
    /// **The longest run of them.** *wrong for exactly 1 frame, the decode's latency* is a
    /// statement about a run and not about a total: seven batches each cost their own latency
    /// frame, so a total of one would need six of the seven not to move the cursor's file.
    pub longest_wrong_run: u32,
    /// Frames played.
    pub frames: u32,
}

/// The directory in the order a readdir hands it over: **largest first**, so that every batch
/// splices in ahead of everything already on screen.
///
/// That is the worst case and it is stated rather than smuggled: a listing is displayed sorted and
/// arrives in whatever order the filesystem keeps it in, and the batch scene is about what happens
/// when the two disagree.
pub fn arrival() -> Vec<File> {
    let mut files = directory();
    files.sort_by_key(|f| std::cmp::Reverse(f.size));
    files
}

/// **The streamed screen.** Seven batches, no input at all, and a cursor whose file changes under it.
pub fn batches(build: Build) -> Batches {
    let arriving = arrival();
    let mut have: Vec<File> = arriving[..SEED].to_vec();
    have.sort_by_key(|f| f.size);
    let mut screen = Screen::open(build);

    screen.frame(&have, BATCH_CURSOR);
    screen.settle();
    screen.frame(&have, BATCH_CURSOR);

    let mut wrong_after = 0;
    let mut wrong_frames = 0;
    let mut longest = 0;
    let mut run = 0;
    for batch in 0..BATCHES {
        let from = SEED + batch * BATCH;
        have.extend_from_slice(&arriving[from..from + BATCH]);
        have.sort_by_key(|f| f.size);
        let want = have[BATCH_CURSOR].name.clone();

        let mut settled_wrong = false;
        for step in 0..3 {
            if step == 1 {
                screen.settle();
            }
            let drawn = screen.frame(&have, BATCH_CURSOR);
            let wrong = drawn.showing.as_deref() != Some(want.as_str());
            if wrong {
                wrong_frames += 1;
                run += 1;
                longest = longest.max(run);
            } else {
                run = 0;
            }
            if step == 2 {
                settled_wrong = wrong;
            }
        }
        if settled_wrong {
            wrong_after += 1;
        }
    }
    Batches {
        key: build.key,
        batches: BATCHES,
        wrong_after,
        wrong_frames,
        longest_wrong_run: longest,
        frames: screen.frames(),
    }
}

// ── scene 23: twenty selections through a directory of photographs ───────────────────────────────

/// How many selections the photograph scene plays. The twenty.
pub const SELECTIONS: usize = 20;

/// **What a picture costs on the wire**, in bytes: [`BODY_CELLS`] at the stated **37.5 B/cell**.
///
/// It is arithmetic and not a measurement, and it stays arithmetic. What row 161 measures since
/// the two-colour question is one screen's bytes through one engine; this figure is a *price
/// list* over a count — how many pictures were drawn, times a B/cell — and multiplying by a
/// measurement taken on the picture screen would make this scene's number a fact about that one.
/// **The figure is 4.1% under what that screen measures** (39.05 against 37.5), which is the
/// size of the error being carried and is why the count is what this scene gates.
pub const PICTURE_BYTES: u64 = BODY_CELLS as u64 * 75 / 2;

/// **What a wire figure is**, and the count underneath it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Wire {
    /// Which regime.
    pub repeat: Repeat,
    /// Selections played. Twenty either way.
    pub selections: u32,
    /// **Pictures actually drawn.** The gate: 1 against 20.
    pub pictures: u32,
    /// What that is in bytes, at [`PICTURE_BYTES`] each.
    pub bytes: u64,
    /// How long the twenty selections took, in milliseconds.
    pub millis: u32,
}

impl Wire {
    /// Bytes a second.
    pub fn rate(self) -> u64 {
        self.bytes * 1_000 / u64::from(self.millis)
    }

    /// The picture's own size in KiB. **536.57**, printed elsewhere as *536 KB* — truncated, not
    /// rounded, and that one character is what the total beside it is built out of.
    pub fn picture_kib() -> f64 {
        PICTURE_BYTES as f64 / 1024.0
    }

    /// The total in decimal MB, which is the unit the rate is in.
    pub fn total_mb(self) -> f64 {
        self.bytes as f64 / 1_000_000.0
    }
}

/// The directory of photographs.
pub fn photographs_directory() -> Vec<File> {
    (0..SELECTIONS)
        .map(|i| File {
            id: 0xC000 + i as u64,
            name: format!("photo{i:02}"),
            rows: u32::from(BODY_H),
            size: i as u64,
            kind: Kind::Photograph,
        })
        .collect()
}

/// **The crossover screen.** Twenty selections at a key repeat, against a decode that does or does not fit
/// inside it.
///
/// The schedule is a virtual clock in milliseconds and the mechanism decides the answer: a question
/// that is superseded while it runs lands nothing, which is [`Task::request`]'s own bracket and not
/// this function's arithmetic. Neither arm is branched on.
pub fn photographs(repeat: Repeat) -> Wire {
    let files = photographs_directory();
    let mut screen = Screen::open(Build::correct());
    let mut queued: Vec<(usize, u32)> = Vec::new();
    let mut pictures = 0u32;
    let mut showing: Option<String> = None;

    for i in 0..SELECTIONS {
        let now = i as u32 * repeat.repeat_ms();
        while queued.first().is_some_and(|&(_, finish)| finish <= now) {
            let (index, _) = queued.remove(0);
            screen.run_job(index);
        }
        let before = screen.spawns();
        let drawn = screen.frame(&files, i);
        if screen.spawns() > before {
            queued.push(((screen.spawns() - 1) as usize, now + repeat.decode_ms()));
        }
        if drawn.showing != showing {
            showing = drawn.showing.clone();
            pictures += 1;
        }
    }

    // The tail: the last decode finishes, and one more frame draws what it brought.
    for (index, _) in std::mem::take(&mut queued) {
        screen.run_job(index);
    }
    let drawn = screen.frame(&files, SELECTIONS - 1);
    if drawn.showing != showing {
        pictures += 1;
    }

    Wire {
        repeat,
        selections: SELECTIONS as u32,
        pictures,
        bytes: u64::from(pictures) * PICTURE_BYTES,
        millis: SELECTIONS as u32 * repeat.repeat_ms(),
    }
}

// ── the offset, and its five spellings ───────────────────────────────────────────────────────────

/// **What one offset spelling does**, on the three numbers: a 4 000-row file, an 800-row file
/// and a 74-row viewport.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct OffsetCase {
    /// Which spelling.
    pub spelling: Offset,
    /// Which file the body is drawing while the new one decodes. **The previous one, still on
    /// screen.**
    pub decoding_shows: Option<String>,
    /// The row it is drawing it from. `Some(0)` is *scrolled the previous file to its top*.
    pub decoding_first_row: Option<u32>,
    /// The row the new file opens at, or `None` when the body drew nothing at all.
    pub arrival_first_row: Option<u32>,
    /// How many rows the new file's body drew.
    pub arrival_rows: u32,
    /// Content cells the new file's body wrote. **Zero is *the body draws nothing at all*.**
    pub arrival_content: u64,
    /// The row the first file opens at when it is selected again. **`Some(0)` is *forgets on
    /// return*.**
    pub return_first_row: Option<u32>,
}

/// **The four defects and the one refusal, each played on the same schedule.**
///
/// Select the long file, scroll it to its last page, select the short one, and select the long one
/// again. Every spelling is right about at least one of those four moments and only one is right
/// about all of them.
pub fn offsets(spelling: Offset) -> OffsetCase {
    let files = vec![long_file(), short_file()];
    let mut screen = Screen::open(Build::correct().offset(spelling));

    screen.frame(&files, 0);
    screen.settle();
    screen.frame(&files, 0);
    screen.scroll_to(LONG_LAST_PAGE);
    screen.frame(&files, 0);

    let decoding = screen.frame(&files, 1);
    screen.settle();
    let arrival = screen.frame(&files, 1);

    screen.frame(&files, 0);
    screen.settle();
    let returned = screen.frame(&files, 0);

    OffsetCase {
        spelling,
        decoding_shows: decoding.showing,
        decoding_first_row: decoding.first_row,
        arrival_first_row: arrival.first_row,
        arrival_rows: arrival.body_rows,
        arrival_content: arrival.content_writes,
        return_first_row: returned.first_row,
    }
}

// ── the per-file map, and the release argument ───────────────────────────────────────────────────

/// What one slot costs: a `u32` offset. **4 bytes.**
pub const SLOT_BYTES: u64 = 4;

/// The key a per-file map is keyed on: the file's own identity.
pub const MAP_KEY_BYTES: u64 = std::mem::size_of::<u64>() as u64;

/// The value it holds: the offset.
pub const MAP_VALUE_BYTES: u64 = std::mem::size_of::<u32>() as u64;

/// The control byte a hash table spends per bucket.
pub const MAP_CONTROL_BYTES: u64 = 1;

/// **What a per-file map costs at `entries`**, at a hash table's 7/8 load factor.
///
/// `entries * (8 + 4 + 1) * 8 / 7`, which at a million is **14 857 142** — the number,
/// reproduced exactly from the arithmetic that produces it. See [`measured_map_bytes`] for what the
/// shipped map really takes, which is worse and makes the refusal stronger rather than weaker.
pub fn map_bytes(entries: u64) -> u64 {
    entries * (MAP_KEY_BYTES + MAP_VALUE_BYTES + MAP_CONTROL_BYTES) * 8 / 7
}

/// **What the shipped `HashMap<u64, u32>` really takes at `entries`.**
///
/// The buckets are rounded to a power of two and the pair is padded to 16 bytes rather than packed
/// to 12, so the real figure is not the estimate — **35 651 584 at a million entries here**. It is
/// read off the map's own `capacity` rather than assumed.
///
/// **What is gated is the relation and not that number.** A count of the standard library's own
/// allocation is a gate on somebody else's implementation, and the rule is that a gate is a
/// property of the mechanism; the property here is that the shipped map is *larger* than
/// estimate, so the refusal does not get easier when it is checked. The figure itself is printed.
pub fn measured_map_bytes(entries: usize) -> u64 {
    let map: HashMap<u64, u32> = HashMap::with_capacity(entries);
    let buckets = map.capacity() as u64 * 8 / 7;
    let per_bucket = std::mem::size_of::<(u64, u32)>() as u64 + MAP_CONTROL_BYTES;
    buckets.next_power_of_two() * per_bucket
}

// ── the torn frame ───────────────────────────────────────────────────────────────────────────────

/// **How many of [`SELECTIONS`] frames drew their two status rows from two versions of one value.**
///
/// Twenty of twenty when the answer is taken inside the draw, none when it is taken once at the top
/// of the view. The count is read off the drawn surface and not off a return value, because the
/// whole claim is that the *screen* disagrees with itself.
pub fn torn(taken: Taken) -> u32 {
    let files = photographs_directory();
    let mut screen = Screen::open(Build::correct().taken(taken));
    let mut torn = 0;
    for i in 0..SELECTIONS {
        screen.frame(&files, i);
        screen.settle();
        let mut pen = Pen::new(W, H);
        screen.frame_into(&mut pen, &files, i);
        let canvas = pen.into_canvas();
        let top = canvas.row_text(TOP_STATUS_Y as u16);
        let bottom = canvas.row_text(BOTTOM_STATUS_Y as u16);
        if top != bottom {
            torn += 1;
        }
    }
    torn
}

// ── the answer's own identity ────────────────────────────────────────────────────────────────────

/// **What a mismatched answer costs**, which is nothing at all — and that is the gate.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Mismatched {
    /// Answers that landed carrying a current generation.
    pub answered: u64,
    /// **Answers the pane refused on [`Preview::shows`].** A question nobody asked.
    pub refused: u64,
    /// Answers the pane showed.
    pub landings: u64,
    /// **Content cells the mismatched answer wrote.** Zero.
    pub content_writes: u64,
}

/// **An answer to a question nobody asked**, and the door an answer-ordering test never opens.
///
/// The generation is current — the job was started for the right question, so [`Task::take`] hands
/// it over — and the payload is another file's. That is the defect the eight bytes on the
/// payload for, and *a test on the answer cannot see it*: an ordering test asserts that answers
/// arrive in the order they were asked for, and a question that was never asked is not out of order.
///
/// `honest` is whether the decode answers the question it was handed.
pub fn mismatched(honest: bool) -> Mismatched {
    let files = vec![long_file(), short_file()];
    let mut screen = Screen::open(Build::correct());
    screen.frame_lying(&files, 0, !honest);
    screen.settle();
    let drawn = screen.frame_lying(&files, 0, !honest);
    Mismatched {
        answered: screen.spawns(),
        refused: screen.refused(),
        landings: screen.landings(),
        content_writes: drawn.content_writes,
    }
}

// ── requirement 9, as a count ────────────────────────────────────────────────────────────────────

/// Where the decode runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DecodeAt {
    /// **What ships.** On the worker, which is a thread holding no reference to the task that
    /// started the job — `!Sync`, and [`crate::files::WhyTheAutoTraitIsSync`] is the pair.
    OnTheWorker,
    /// The same decode, called from the view.
    InTheView,
}

/// **Decode units that ran while the app thread was inside `Driver::frame`.**
///
/// The requirement 9 as a count rather than as a microsecond: **0 against
/// [`DECODE_UNITS_IN_THE_VIEW`]** for the same decode, over the same twenty selections, with the
/// answer still arriving.
///
/// See [`Screen::decode_units_in_the_frame`] for why *inside the frame* and not *on the app thread*
/// is the honest reading of what a `Worker::queueing` can be asked.
pub fn decoded(at: DecodeAt) -> u64 {
    let files = photographs_directory();
    let mut screen = Screen::open(Build::correct());
    for i in 0..SELECTIONS {
        match at {
            DecodeAt::OnTheWorker => {
                screen.frame(&files, i);
                screen.settle();
            }
            DecodeAt::InTheView => screen.frame_decoding_in_the_view(&files, i),
        }
    }
    screen.decode_units_in_the_frame()
}

// ── R02's sweep, refused twice ───────────────────────────────────────────────────────────────────

/// **What ten tab switches cost**, by where the job and the derived value live.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Sweep {
    /// Questions that reached the worker.
    pub spawns: u64,
    /// Decodes performed.
    pub decodes: u64,
    /// **Full folds of the document.** The derived value's half of the same refusal.
    pub folds: u32,
}

/// How many times the pane is tabbed away from and back to. The ten.
pub const TAB_SWITCHES: usize = 10;

/// **R02's sweep, refused twice, over two different things and for two different reasons.**
///
/// A pending job swept by the widget registry costs **10 spawns and 10 decodes** against 1 and 1
/// with the slot in application state; a derived value swept the same way costs **10 full folds
/// against 1**. *A job's lifetime is the question's, a memo's is the data's, and neither is the
/// widget's.*
///
/// `swept` is whether the pane's storage is rebuilt each time the widget comes back — which is
/// exactly what a sweep keyed on an [`Id`](vitui_runtime::Id) does, because it releases what did not
/// draw.
pub fn sweeps(swept: bool) -> Sweep {
    let files = vec![long_file(), short_file()];
    if !swept {
        // **One screen, kept across the switches.** The pane does not draw while the tab is away,
        // and neither the task nor the memo notices: the question has not changed, so nothing is
        // asked, and the document has not changed, so nothing is folded.
        let mut screen = Screen::open(Build::correct());
        let before = decode_units();
        for _ in 0..TAB_SWITCHES {
            screen.frame(&files, 0);
            screen.settle();
            screen.frame(&files, 0);
        }
        return Sweep {
            spawns: screen.spawns(),
            decodes: (decode_units() - before) / u64::from(long_file().rows),
            folds: screen.recomputes(),
        };
    }
    // **A screen a frame**, which is what a registry sweep leaves: the slot went away with the
    // widget, so the question is new every time and so is the memo's key.
    let mut spawns = 0;
    let mut folds = 0;
    let before = decode_units();
    for _ in 0..TAB_SWITCHES {
        let mut screen = Screen::open(Build::correct());
        screen.frame(&files, 0);
        screen.settle();
        screen.frame(&files, 0);
        spawns += screen.spawns();
        folds += screen.recomputes();
    }
    Sweep {
        spawns,
        decodes: (decode_units() - before) / u64::from(long_file().rows),
        folds,
    }
}

// ── R09's edit, and the drop that did not land ───────────────────────────────────────────────────

/// How many frames each selection is held for. Six, and the run is twenty of them.
pub const HOLD: u32 = 6;

/// How many frames the revision run plays. **120** — [`SELECTIONS`] selections held [`HOLD`] frames.
pub const REVISION_FRAMES: u32 = SELECTIONS as u32 * HOLD;

/// **What a revision run costs**, by when the revision advances.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Revisions {
    /// Frames played. [`REVISION_FRAMES`] either way.
    pub frames: u32,
    /// Answers that landed. Twenty either way.
    pub landings: u64,
    /// **Full folds of the document.** 20 against 119.
    pub folds: u32,
    /// Cells the run drew, so that *the two screens are cell-identical* is a number.
    pub writes: u64,
}

/// **R09's `Edit` bumps on drop, and a landing that did not happen is still a drop.**
///
/// Written without the branch, the pane's revision advances on every frame and the highlighter
/// re-folds the whole file — **119 computations against 20 landings** — on two screens that are
/// cell-identical. The 119 is every frame but the first, which is the one frame with no document to
/// fold.
pub fn revisions(bump: Bump) -> Revisions {
    let files = photographs_directory();
    let mut screen = Screen::open(Build::correct().bump(bump));
    let mut tally = Tally::new();
    for i in 0..SELECTIONS {
        for step in 0..HOLD {
            if step == 1 {
                screen.settle();
            }
            screen.frame_into(&mut tally, &files, i);
        }
    }
    Revisions {
        frames: screen.frames(),
        landings: screen.landings(),
        folds: screen.recomputes(),
        writes: tally.writes(),
    }
}

// ── the steady frame, and the one with a photograph in it ────────────────────────────────────────

/// A listing of `entries` files, built cheaply enough to be worth doing at a million.
///
/// The names are the only heap in it, and they have to be: the list draws them.
pub fn many(entries: usize) -> Vec<File> {
    (0..entries)
        .map(|i| File {
            id: 0x5EED_0000_0000_0000 ^ i as u64,
            name: format!("f{i:07}"),
            rows: 120 + (i as u32 % 7) * 13,
            size: i as u64,
            kind: Kind::Text,
        })
        .collect()
}

/// The three volumes the frame section is measured at.
pub const VOLUMES: [usize; 3] = [1_000, 100_000, 1_000_000];

/// **The component alone, on a frame with nothing else in it**, for a caller that wants to open an
/// allocation window over it.
///
/// The allocation budget cannot be measured from inside this module — the counting allocator is
/// `vitui-alloc-probe`, installed in the test binaries, which is `crate::gates` row 32's
/// arrangement — so what this hands over is the *ingredients* and
/// `crates/vitui-components/tests/budget.rs` opens the window.
///
/// # It is the component and not the screen, and the difference is 452 allocations a frame
///
/// [`Screen`] formats a `String` for every list row and every document line, which is what an
/// instrument does and what [`crate::ink`] says out loud about `Tally` and `Pen`: *an instrument
/// allocating is not a defect.* A window over the screen measures the screen. The budget is the
/// **component's**, so the window goes over `file_preview_pane_into` with a line drawer that stages
/// rather than formats — which is the path a component actually takes.
pub struct Alone {
    /// The driver the frames are drawn on.
    pub driver: Driver,
    /// The worker the questions go to.
    pub worker: Worker,
    /// The task the answer comes through.
    pub task: Task<Doc>,
    /// The pane's state.
    pub pane: PaneState<Doc>,
}

impl Alone {
    /// A pane with a landing in its history, warmed with two identical frames — the frame
    /// structures take their allocation on the first frame that needs one and keep it, so a window
    /// containing first touch measures the loader rather than the steady state.
    pub fn warmed() -> Alone {
        let worker = Worker::queueing();
        let task = Task::new(&worker);
        let mut alone = Alone {
            driver: Driver::headless(PANE_W, PANE_H).expect("a sink cannot fail to attach"),
            worker,
            task,
            pane: PaneState::new(),
        };
        alone.frame();
        // The answer, run outside the frame, which is where a decode belongs.
        let _landed = alone.worker.run(0);
        alone.frame();
        alone.frame();
        alone
    }

    /// **One frame of the component and nothing else.**
    ///
    /// The line drawer writes a fixed cluster through [`Ink::run`], which stages into the frame's
    /// own buffer: no `format!`, no `String`, and the same verb the shipped path takes.
    pub fn frame(&mut self) {
        let Alone {
            driver, task, pane, ..
        } = self;
        driver.frame(|cx| {
            let id = cx.id();
            let paint = cx.theme().paint(Role::Body);
            let opts = PaneOpts::default();
            let _ = files::file_preview_pane_into(
                &mut Direct,
                cx,
                id,
                Rect::new(0, 0, PANE_W, PANE_H),
                pane,
                task,
                files::asking(ALONE_FILE, move |_cancel| Doc {
                    id: ALONE_FILE,
                    name: String::new(),
                    rows: u32::from(BODY_H) * 4,
                    kind: Kind::Text,
                }),
                &opts,
                |ink: &mut Direct, cx: &mut Ctx<'_, '_>, row: Rect, _doc: &Doc, _i: u32| {
                    let _ = ink.run(cx, row.x, row.y, ".", row.w, paint);
                },
            );
        });
    }
}

/// The identity the [`Alone`] pane's one question is named with.
pub const ALONE_FILE: u64 = 0xA10E;

/// **What one frame of [`Screen`] allocates**, which is the instrument's own cost and not the
/// component's — 452 a frame, every one of them a `format!` the screen writes and a component does
/// not. Printed rather than gated, and named so that a reader of the budget gate knows which of the
/// two it is about.
pub const SCREEN_ALLOCATIONS_A_FRAME: u64 = 452;

/// **One steady frame at `entries` files**, with a worker in the frame and a landing in its history.
pub fn steady(entries: usize) -> Shape {
    let files = many(entries);
    let mut screen = Screen::open(Build::correct());
    screen.frame(&files, 0);
    screen.settle();
    screen.frame(&files, 0);
    let mut tally = Tally::new();
    let drawn = screen.frame_into(&mut tally, &files, 0);
    Shape {
        writes: tally.writes(),
        distinct: tally.distinct(),
        verbs: tally.verbs(),
        regions: screen.driver.inspect().hits().len(),
        content_writes: drawn.content_writes,
        customs: drawn.customs,
    }
}

/// **One steady frame at a million entries with a photograph selected.**
///
/// *One `Theme::custom` per cell of the pane*, which is [`BODY_CELLS`] and is what this
/// returns — the pane's viewport is a partition and the photograph is as wide as it.
pub fn photograph_frame(entries: usize) -> Shape {
    let mut files = many(entries);
    files[0].kind = Kind::Photograph;
    files[0].rows = u32::from(BODY_H);
    let mut screen = Screen::open(Build::correct());
    screen.frame(&files, 0);
    screen.settle();
    screen.frame(&files, 0);
    let mut tally = Tally::new();
    let drawn = screen.frame_into(&mut tally, &files, 0);
    Shape {
        writes: tally.writes(),
        distinct: tally.distinct(),
        verbs: tally.verbs(),
        regions: screen.driver.inspect().hits().len(),
        content_writes: drawn.content_writes,
        customs: drawn.customs,
    }
}

// ── the picker, and R3's claim ───────────────────────────────────────────────────────────────────

/// **What a frame with a picker standing declares.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Parts {
    /// Interactive regions.
    pub regions: usize,
    /// Tab stops.
    pub stops: usize,
    /// Overlay layers placed.
    pub overlays: u32,
    /// Overlay requests that named an owner who had already asked. **Zero**, or two widgets are one.
    pub merges: u32,
}

/// The width the picker's own listing takes inside its overlay.
pub const PICKER_LIST_W: u16 = 24;

/// **What a shut picker's face writes at one width.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shut {
    /// The width it was drawn at.
    pub w: u16,
    /// Cells written, from the engine's own column reports.
    pub writes: u64,
    /// Distinct cells touched. **Equal to [`Shut::writes`]**, or a cell was written twice.
    pub distinct: u64,
    /// How many rows it was handed. See [`shut_faces`].
    pub h: u16,
}

/// **The shut face at every width from 1 to 40**, which is the rule on the one drawing this
/// component inherited from somewhere else.
///
/// `file_picker`'s shut face is [`crate::input::select`]'s, transcribed: a chevron, a space, an
/// elided label padded to what is left, and the marker. **That drawing has been wrong before** — the
/// pad written to the *whole* width with the ellipsis over its last cell writes one cell twice, on
/// every truncated widget, invisible on the screen and invisible to every counter but the pair.
/// Components 26 found it in `select` and `select` grew a gate; the copy inherited the shape, so it
/// gets the gate.
///
/// **A rectangle and not a row, which is components 40 correcting this sentence.** It read *one row
/// and not a rectangle, which is the answer `select` gives: the widget is a line, and a caller
/// handing it a taller rectangle is handing it more than it claims* — and that was the register's
/// seventh row being red in prose. Handed thirteen rows this component wrote **one**, 576 cells of a
/// 48x13 tile left to whatever was already in them, and `crate::input::select` did the same: the two
/// overlay owners were the only two rows of the freeze that did not write every cell of a rectangle
/// taller than their content. A `Response` has no field a remainder could be named in (the third
/// clause), so the face is the rectangle. Swept over both axes here.
pub fn shut_faces() -> Vec<Shut> {
    use crate::files::{Entry, PickerBody, PickerOpts, PickerState, file_picker_into};

    let names: Vec<String> = (0..4usize)
        .map(|i| format!("a-rather-long-file-name-{i:03}"))
        .collect();
    let entries: Vec<Entry<'_>> = names
        .iter()
        .enumerate()
        .map(|(i, n)| Entry {
            id: 0xF11E_0000 + i as u64,
            name: n.as_str(),
        })
        .collect();
    let opts = PickerOpts::default();
    let mut out = Vec::new();
    for (w, h) in (1..=40u16).flat_map(|w| (1..=3u16).map(move |h| (w, h))) {
        let worker = Worker::queueing();
        let task: Task<Doc> = Task::new(&worker);
        let mut body: PickerBody<Doc> = PickerBody::new();
        let mut st = PickerState::new();
        st.choose(entries[2].id);
        let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        // Two frames and not one: the frame structures take their allocation on the first frame
        // that needs one, so a cold frame is not a frame.
        driver.frame(|cx| {
            let id = cx.id();
            let _ = file_picker_into(
                &mut Direct,
                cx,
                id,
                Rect::new(0, 0, w, h),
                &mut st,
                &mut body,
                &entries,
                &task,
                picker_decode,
                picker_line,
                &opts,
            );
        });
        driver.frame(|cx| {
            let id = cx.id();
            let _ = file_picker_into(
                &mut tally,
                cx,
                id,
                Rect::new(0, 0, w, h),
                &mut st,
                &mut body,
                &entries,
                &task,
                picker_decode,
                picker_line,
                &opts,
            );
        });
        out.push(Shut {
            w,
            h,
            writes: tally.writes(),
            distinct: tally.distinct(),
        });
    }
    out
}

/// **The picker, drawn shut and then open**, so that what the overlay costs is a subtraction.
pub fn picker(open: bool) -> Parts {
    use crate::files::{Entry, PickerBody, PickerOpts, PickerState, file_picker};

    let names: Vec<String> = (0..8usize).map(|i| format!("f{i:03}")).collect();
    let entries: Vec<Entry<'_>> = names
        .iter()
        .enumerate()
        .map(|(i, n)| Entry {
            id: 0xF11E_0000 + i as u64,
            name: n.as_str(),
        })
        .collect();
    let worker = Worker::queueing();
    let task: Task<Doc> = Task::new(&worker);
    let mut body: PickerBody<Doc> = PickerBody::new();
    let mut st = PickerState::new();
    if open {
        st.open();
    }
    let opts = PickerOpts {
        list_w: PICKER_LIST_W,
        ..PickerOpts::default()
    };
    let mut driver = Driver::headless(80, 24).expect("a sink cannot fail to attach");
    driver.frame(|cx| {
        let area = Rect::new(0, 0, 40, 1);
        let _ = file_picker(
            cx,
            area,
            &mut st,
            &mut body,
            &entries,
            &task,
            picker_decode,
            picker_line,
            &opts,
        );
    });
    let frame = driver.inspect();
    Parts {
        regions: frame.hits().len(),
        stops: frame.ring().len(),
        overlays: frame.overlays_placed(),
        merges: frame.overlays_merged(),
    }
}

/// The picker's decode: a free function over an identity, which is what a decode is in every
/// application that has one. See [`crate::files::file_picker`] for why it may not be a closure.
fn picker_decode(id: u64, _cancel: &Cancel) -> Doc {
    Doc {
        id,
        name: String::new(),
        rows: 40,
        kind: Kind::Text,
    }
}

/// The picker's line drawer, likewise.
fn picker_line(cx: &mut Ctx<'_, '_>, row: Rect, doc: &Doc, i: u32) {
    let paint = cx.theme().paint(Role::Body);
    let mut ink = Direct;
    let _ = ink.pad_to(
        cx,
        row.x,
        row.y,
        &format!("{:016x}:{i}", doc.id),
        row.w,
        paint,
    );
}

/// **What `file_picker` may not contain**, read out of its own source.
///
/// R3's claim is *composition with no new mechanism*, and that is exactly the claim that turns out
/// to be false when it is false — so it is read rather than asserted. Each needle is a mechanism
/// one of the three parts already owns: a task is the application's, a listing's state is
/// `collection`'s, a scroll offset is the pane's, and a shell is `overlay`'s.
pub const PICKER_MAY_NOT: [(&str, &str); 4] = [
    (
        "Task::new(",
        "a job's lifetime is the question's and the task is the application's",
    ),
    (
        "AreaState",
        "the offset is the pane's, and the pane is `scroll_area`'s",
    ),
    ("Selection::new(", "the listing's state is `collection`'s"),
    (
        "scroll_scope(",
        "the only scrolled scope in a picker is the one the pane opens",
    ),
];

/// **What `file_picker` must contain**: its three parts, by name.
///
/// **`collection_shaped` and not `collection_into`**, which is where the row
/// loop went when the picker's listing gained the refused spellings `shrunk` and `wheeled`
/// axes need: the picker *is* `collection` inside a layer, so the mistakes `collection` has arms for
/// are mistakes a picker's listing can make too, and until scenes 41 and 42 asked there was no way
/// to write either down on this side. It is `crate::collect`'s own note about `table` one component
/// over.
pub const PICKER_CALLS: [&str; 3] = [
    "collection_shaped(",
    "overlay_into(",
    "file_preview_pane_with(",
];

/// The source of the module the two components are homed in.
pub fn files_source() -> String {
    source(FILES_RS)
}

/// **The picker and its body, and nothing else in the file.**
///
/// The scan is over the picker and not over the module, because the pane is in the same file and
/// the pane is *allowed* to open a scrolled scope — it is the scroll area's caller. It is bounded at
/// both ends rather than run to the end of the file: unbounded it would sweep
/// [`crate::files::WhyTheAutoTraitIsSync`] and the whole of `defective`, which is a scan that gets
/// **looser** as the module grows, and a needle it may not contain would then be satisfiable by
/// putting the offending line in a refused spelling.
///
/// The end marker is the section rule this crate writes between modules' parts, so a `file_picker`
/// that grew past it would fail here rather than escape the scan.
pub fn picker_source() -> String {
    let src = files_source();
    let Some(at) = src.find("pub fn file_picker<") else {
        return String::new();
    };
    let tail = &src[at..];
    match tail.find("\n// \u{2500}") {
        Some(end) => tail[..end].to_string(),
        None => tail.to_string(),
    }
}

// ── the subjects ─────────────────────────────────────────────────────────────────────────────────

/// The components these three screens stand on. **Both are declared** since components 32, which
/// is what turned all three scenes from red to standing — one fact and not three, because they were
/// pinned on one and it was the subject.
pub const SUBJECTS: [&str; 2] = ["file_preview_pane", "file_picker"];

/// Where [`SUBJECTS`] belong, as `(module file, the declaration)`.
///
/// The home is [`crate::Family::F12Files`]'s, whose module is `files.rs`. A component is
/// `fn(&mut Ctx, Rect, …) -> Response`, so the thing to look for is a public
/// function of the component's own name in its own family's module.
///
/// # The needle problem, met a fourth time, and this time by the ticket that owns the answer
///
/// One scan read `pub fn select(` for a component that already costs two lifetime
/// annotations; components 30 read `pub fn picture(` for one whose `…` is a type parameter; both
/// would have gone green *by deleting the thing that mattered*. Components 31 met it a third time
/// and answered it differently — a longer needle would have been that ticket dictating this one's
/// parameter list — so it left the parenthesis and put the weight on [`mints_its_own_task`], a
/// **negative** scan nothing can satisfy by deletion.
///
/// **Neither parenthesis could ever have matched.** The shipped declarations are
/// `pub fn file_preview_pane<T, F>(` and `pub fn file_picker<'f, T>(`, and both generic lists are
/// load-bearing for the same two reasons the earlier three were: `T` is the answer, which the pane
/// may not construct, and `'f` is the mechanism an overlay body rests on. This is the ticket that
/// owns the parameter list, so it is the ticket that may write the needle — and it writes it long
/// enough that deleting either would fail the scan.
///
/// The negative half stays, because it is the half no signature can buy: a pane that called
/// `Task::new` inside itself would pay **10 spawns and 10 decodes over ten tab switches against 1
/// and 1** ([`sweeps`]), whatever its signature said.
pub const DECLARATIONS: [(&str, &str); 2] = [
    ("files.rs", "pub fn file_preview_pane<T, F>("),
    ("files.rs", "pub fn file_picker<'f, T>("),
];

/// **What the pane's signature owes, beyond its own name.**
///
/// Each of these is a parameter that could be deleted while leaving the declaration scan green, and
/// each is a mechanism assigned somewhere other than the widget. The task is the application's
/// (R02's sweep, refused); the state is the caller's; the line drawer is the caller's, which is what
/// makes the frame's cost the *rectangle*'s rather than the document's.
pub const PANE_OWES: [(&str, &str); 3] = [
    (
        "task: &Task<T>,",
        "a job's lifetime is the question's, so the task is a parameter and never a field",
    ),
    (
        "st: &mut PaneState<T>,",
        "the pane's storage is the caller's, for the same reason",
    ),
    (
        "line: &mut dyn FnMut(&mut Ctx<'_, '_>, Rect, &T, u32),",
        "the document's cells are the caller's, so the frame costs the rectangle and not the file",
    ),
];

/// The file [`SUBJECTS`] are homed in.
const FILES_RS: &str = "files.rs";

/// Read `src/<file>`, or an empty string when it is not there.
fn source(file: &str) -> String {
    let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).join(file);
    std::fs::read_to_string(&path).unwrap_or_default()
}

/// **Which of [`SUBJECTS`] this crate declares. Since components 32: both.**
pub fn subjects_declared() -> Vec<&'static str> {
    let mut out = Vec::new();
    for (subject, (file, declaration)) in SUBJECTS.into_iter().zip(DECLARATIONS) {
        if crate::dense::declares(&source(file), declaration) {
            out.push(subject);
        }
    }
    out
}

/// **Whether the pane mints its own [`Task`]**, which is R02's sweep arriving on a job's lifetime.
///
/// A pane that owns the task cannot survive its widget not drawing — the sweep drops what did not
/// draw — so ten tab switches cost **10 spawns and 10 decodes against 1 and 1**. The scan is
/// `pub(crate)`-blind on purpose: what it is looking for is the constructor, wherever it is spelled.
///
/// It answers `false` over a file that now has a pane in it, which is the direction that matters:
/// the pane takes its task as a parameter. `tests::the_task_scan_fires_in_both_directions` hands the
/// predicate the two sources it must separate, because a scan watched in one direction is a scan
/// nobody has watched.
pub fn mints_its_own_task() -> bool {
    task_minted_in(&source(FILES_RS))
}

/// The predicate behind [`mints_its_own_task`], so that both directions can be watched.
pub fn task_minted_in(source: &str) -> bool {
    crate::dense::declares(source, "Task::new(")
}

/// **Whether the three screens stand on their subjects, as a verdict rather than as a sentence.**
///
/// [`Verdict::of`] refuses vacuity in its constructor, which is what makes this the right shape: the
/// population is the two subjects, so *no component exists* is `Unmet` over two rather than `Met`
/// over nothing.
pub fn standing() -> Verdict {
    let declared = subjects_declared();
    Verdict::of(
        SUBJECTS.len(),
        SUBJECTS.len() - declared.len(),
        "`file_preview_pane` and `file_picker` are declared in `src/files.rs`, so all three \
         preview-pane screens are played over the components. A subject that went away would turn \
         this red again and say which one",
        "components 32",
    )
}

/// **The sentence a scene waiting for its subject fails with**, or `None` once both are declared.
pub fn owed_message(declared: &[&str], scene: &str) -> Option<String> {
    if declared.len() == SUBJECTS.len() {
        return None;
    }
    let owed: Vec<String> = SUBJECTS
        .into_iter()
        .zip(DECLARATIONS)
        .filter(|(id, _)| !declared.contains(id))
        .map(|(id, (file, declaration))| format!("`{id}` (`src/{file}`: `{declaration}…)`)"))
        .collect();
    Some(format!(
        "{scene} is not standing, and it is waiting for its subject rather than failing: {} of {} \
         components are undeclared — {owed}. This is not a defect in the screen. The re-sort is \
         wrong on 100 of 100 frames under a position key and 0 under an identity key with 103 \
         questions either way, the seven batches are wrong after 7 of 7 with no user in them, the \
         twenty selections draw 1 picture against 20, the five offset spellings each produce their \
         own defect, the per-file map costs 14 857 142 bytes against a slot's 4 and the tear is 20 \
         frames of 20 — see `crate::preview::tests`. Inverted by `components 32`",
        owed.len(),
        SUBJECTS.len(),
        owed = owed.join(", "),
    ))
}

/// **Fail with the subjects that are missing, the file they belong in, and the ticket.**
///
/// # Panics
///
/// Panics while any of [`SUBJECTS`] is undeclared, which is **no longer today**: components 32
/// declared both. It is kept live rather than deleted, because a subject that is removed or renamed
/// has to fail here and say so — see `tests::the_waiting_message_says_which_failure_it_is_and_stops_when_it_should`.
pub fn assert_stands_up(scene: &str) {
    if let Some(message) = owed_message(&subjects_declared(), scene) {
        panic!("{message}");
    }
}

// ── the report ───────────────────────────────────────────────────────────────────────────────────

/// **The offset table, one row a spelling**, in the column order.
pub fn offset_table() -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "  {:<22}  {:>10}  {:>10}  {:>8}  {:>8}  {:>8}",
        "the offset is", "decoding", "opens at", "rows", "content", "on return"
    );
    for spelling in Offset::ALL {
        let case = offsets(spelling);
        let row = |v: Option<u32>| match v {
            Some(n) => n.to_string(),
            None => String::from("-"),
        };
        let _ = writeln!(
            out,
            "  {:<22}  {:>10}  {:>10}  {:>8}  {:>8}  {:>8}",
            spelling.name(),
            row(case.decoding_first_row),
            row(case.arrival_first_row),
            case.arrival_rows,
            case.arrival_content,
            row(case.return_first_row),
        );
    }
    out
}

/// **The wire table**, with the three readings of one product beside each other.
pub fn wire_table() -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "  {:<24}  {:>8}  {:>12}  {:>10}  {:>10}",
        "the repeat is", "pictures", "bytes", "MB", "MB/s"
    );
    for repeat in [Repeat::SlowerThanTheDecode, Repeat::FasterThanTheDecode] {
        let wire = photographs(repeat);
        let _ = writeln!(
            out,
            "  {:<24}  {:>8}  {:>12}  {:>10.2}  {:>10.2}",
            match repeat {
                Repeat::SlowerThanTheDecode => "30 ms over a 12 ms decode",
                Repeat::FasterThanTheDecode => "8 ms under a 30 ms decode",
            },
            wire.pictures,
            wire.bytes,
            wire.total_mb(),
            wire.rate() as f64 / 1_000_000.0,
        );
    }
    out
}

// ── the numbers these screens are measured at ────────────────────────────────────────────────────

/// **Positions a re-sort moves. 197 of 200.**
pub const MOVED: u32 = 197;

/// **Frames of the window the position-keyed pane is wrong on. 100 of 100.**
pub const WRONG_UNDER_A_POSITION_KEY: u32 = WINDOW;

/// **Questions asked, either way. 103.** The counter that does not move.
pub const QUESTIONS: u32 = RESORT_FRAMES;

/// **Jobs spawned, by key: 1 under a position key and 2 under an identity key.**
pub const SPAWNS: [u64; 2] = [1, 2];

/// **Batches after which the settled pane is still wrong, by key: 7 of 7 and 0 of 7.**
pub const WRONG_AFTER: [usize; 2] = [BATCHES, 0];

/// **The longest run of wrong frames under an identity key. One — the decode's latency.**
pub const LATENCY_FRAMES: u32 = 1;

/// **Pictures drawn, by regime: 1 against 20.**
pub const PICTURES: [u32; 2] = [1, SELECTIONS as u32];

/// **The rate the twenty-picture arm runs at, in bytes a second. 18 315 000 — the 18.3 MB/s.**
pub const WIRE_RATE: u64 = 18_315_000;

/// **What the total is stated as**, in decimal MB. It is the rounded 536 KiB read as 536 kB and
/// multiplied by twenty; the rate printed in the same sentence needs the unrounded product.
pub const SECTION_15_TOTAL_MB: f64 = 10.7;

/// **A per-file map at a million entries. 14 857 142 bytes against [`SLOT_BYTES`].**
pub const MAP_AT_A_MILLION: u64 = 14_857_142;

/// **Torn frames, by where the answer is taken: 20 of 20 against 0.**
pub const TORN: [u32; 2] = [SELECTIONS as u32, 0];

/// **Interactive regions the screen declares. Three, and every one of them is the component's**:
/// the scroll area's single entry, and one for each of its two reserved bars.
///
/// It was **0** while the screens waited for their subject, because nothing on them declared
/// anything. The frame section has no region column at all, so the number is this ticket's and
/// what is gated beside it is that it does not move with the volume.
pub const REGIONS: usize = 3;

/// **Verbs on a steady text frame. 381**, flat at every volume.
pub const VERBS: u64 = 381;

/// **What the unclamped spelling writes on the frame the document shrinks: 21 484 of 24 000.**
///
/// The 2 516 it leaves unwritten is exactly [`CONTENT_WRITES`] — the body cannot write rows the
/// document has not got, and over an unbounded extent the area has no tail with which to cover
/// them.
///
/// **What is measured and what is definitional are different halves of that sentence**, and the
/// difference is worth being exact about. [`LINE_COLUMNS`] was *derived* by components 31 from
/// own pair — `4 166 − 1 650 = 2 516 = 74 × 34` — so `CONTENT_WRITES == 2 516` is a **definition**
/// and its agreeing with the gap is arithmetic rather than evidence. What components 32 measures
/// is the other half: that the cells the unclamped arm leaves unwritten are **exactly the
/// document's own** and not some other number, which nothing before this ticket could have known.
pub const UNCLAMPED_WRITES: u64 = 21_484;

/// **Decode units that run on the app thread when the decode is called from the view.**
///
/// [`SELECTIONS`] photographs of [`BODY_H`] rows each. The recorded figure is **5 076**, which is a prototype's
/// document; what is gated is the pair — **0 against this** — because the claim is about the thread
/// and not about the size of the decode.
pub const DECODE_UNITS_IN_THE_VIEW: u64 = SELECTIONS as u64 * BODY_H as u64;

/// The figure for the same arm.
pub const SECTION_15_DECODE_UNITS: u64 = 5_076;

/// **Spawns and decodes over ten tab switches, by where the job lives: 1 against 10.**
pub const SWEPT_SPAWNS: [u64; 2] = [1, TAB_SWITCHES as u64];

/// **Full folds over the same ten, by where the memo lives: 1 against 10.**
pub const SWEPT_FOLDS: [u32; 2] = [1, TAB_SWITCHES as u32];

/// **Full folds over the revision run, by when the revision advances: 20 against 119.**
///
/// The 119 is every frame of [`REVISION_FRAMES`] but the first, which is the one frame with no
/// document to fold.
pub const REVISION_FOLDS: [u32; 2] = [SELECTIONS as u32, REVISION_FRAMES - 1];

/// **Customs a photograph in the pane spends: one a cell of the viewport, 14 652.** The original's.
pub const PHOTOGRAPH_CUSTOMS: u64 = BODY_CELLS as u64;

/// The steady frame: **3 557 writes and 197 verbs**, which is a prototype's screen. This one
/// writes [`CELLS`] and [`VERBS`], and what reproduces is that neither moves with the volume.
pub const SECTION_15_STEADY: [u64; 2] = [3_557, 197];

/// The photograph frame: **17 782 writes, 14 833 verbs, 14 652 customs**. The third reproduces
/// exactly and the first two are the same prototype's screen.
pub const SECTION_15_PHOTOGRAPH: [u64; 3] = [17_782, 14_833, 14_652];

/// **Regions a picker declares, shut and open: 1 against 6.**
///
/// The six are a subtraction and not a magnitude: the owner's shut face, the shell's blur position,
/// the collection's one entry however many rows it has, and the pane's three ([`REGIONS`]).
/// **Not one of them is a mechanism the three parts did not already have**, which is R3's claim.
pub const PICKER_REGIONS: [usize; 2] = [1, 6];

/// The picker's six, decomposed. `owner + shell + collection + pane`.
pub const PICKER_DECOMPOSED: [usize; 4] = [1, 1, 1, REGIONS];

#[cfg(test)]
mod tests {
    use super::*;

    /// **The permutation is a property of the data.** 197 of 200 positions move, and the three that
    /// do not are [`FIXED`].
    #[test]
    fn a_re_sort_moves_a_hundred_and_ninety_seven_of_two_hundred_positions() {
        let a = listing(Order::ByName);
        let b = listing(Order::BySize);
        assert_eq!(a.len(), FILES);
        assert_eq!(positions_moved(&a, &b), MOVED);
        for position in FIXED {
            assert_eq!(
                a[position].id, b[position].id,
                "position {position} was supposed to be a fixed point"
            );
        }
        assert_ne!(
            a[CURSOR].id, b[CURSOR].id,
            "the cursor's own file has to move, or the scene decides nothing"
        );
    }

    /// **The re-sort screen.** The memo-key rule, through the door nothing else opens.
    ///
    /// Both arms ask on every frame — **103 either way** — and the only counter that moves is the
    /// spawn count, because the question still matches, so nothing posts, so nothing wakes, so no
    /// frame corrects it.
    #[test]
    fn a_position_key_is_wrong_on_a_hundred_of_a_hundred_frames_after_one_re_sort() {
        let position = resort(Build::correct().keyed(Key::Position));
        let identity = resort(Build::correct().keyed(Key::Identity));

        assert_eq!(position.wrong, WRONG_UNDER_A_POSITION_KEY);
        assert_eq!(identity.wrong, 0);

        assert_eq!(position.requests, QUESTIONS);
        assert_eq!(identity.requests, QUESTIONS);
        assert_eq!(
            position.frames, RESORT_FRAMES,
            "the frame counter and the request counter are two counters, and their agreeing is \
             what *the pane asks on every frame* means"
        );

        assert_eq!([position.spawns, identity.spawns], SPAWNS);
        assert_eq!(position.moved, MOVED);
        assert_eq!(identity.moved, MOVED);
    }

    /// **The streamed screen.** The same hole with no user in it.
    ///
    /// Seven batches, no input at all. The settled pane is wrong after **7 of 7** under a position
    /// key and after none under an identity key, and *wrong for exactly 1 frame* is the
    /// **longest run** rather than the total — seven batches each cost their own latency frame.
    #[test]
    fn seven_batches_move_the_cursors_file_under_it_with_nobody_pressing_anything() {
        let position = batches(Build::correct().keyed(Key::Position));
        let identity = batches(Build::correct().keyed(Key::Identity));

        assert_eq!(
            [position.wrong_after, identity.wrong_after],
            WRONG_AFTER,
            "the pane is settled at each checkpoint: a position key never re-asks, so it is still \
             wrong after every one of the seven"
        );
        assert_eq!(
            identity.longest_wrong_run, LATENCY_FRAMES,
            "an identity key is wrong for exactly the decode's latency and no longer"
        );
        assert!(
            position.longest_wrong_run > identity.longest_wrong_run * 2,
            "a position key is wrong for the rest of the run: {} against {}",
            position.longest_wrong_run,
            identity.longest_wrong_run
        );
        assert_eq!(identity.wrong_frames, BATCHES as u32);
        assert_eq!(position.frames, identity.frames);
        assert_eq!(SEED + BATCHES * BATCH, FILES);
    }

    /// **The crossover screen**, read as a count.
    ///
    /// *The regime in which cancellation saves nothing is exactly the regime in which every picture
    /// is drawn* — so the bytes and the wasted CPU peak together. Neither arm is branched on: a
    /// question superseded while it runs lands nothing, which is `Task::request`'s own bracket.
    #[test]
    fn twenty_selections_draw_one_picture_or_twenty_and_the_bytes_peak_with_the_waste() {
        let fast = photographs(Repeat::FasterThanTheDecode);
        let slow = photographs(Repeat::SlowerThanTheDecode);

        assert_eq!([fast.pictures, slow.pictures], PICTURES);
        assert_eq!(fast.selections, SELECTIONS as u32);
        assert_eq!(slow.selections, SELECTIONS as u32);
        assert_eq!(slow.rate(), WIRE_RATE, "§15's 18.3 MB/s");
        assert!(
            slow.rate() > fast.rate() * 4,
            "the cheap regime is the one that cancels: {} against {}",
            slow.rate(),
            fast.rate()
        );
    }

    /// **The three wire figures cannot all be readings of one product, and two of them are.**
    ///
    /// 14 652 cells at 37.5 B/cell is 549 450 bytes — **536.57 KiB**, printed elsewhere as *536 KB*
    /// by truncating it — and twenty of them is 10 989 000 bytes, which is **10.99 MB** and is
    /// exactly what *18.3 MB/s over 600 ms* requires. **10.7 MB** is that truncated 536 read
    /// as decimal kB and multiplied by twenty, so the total and the rate printed in one sentence are
    /// computed from two different readings of one product. The two that agree with each other are
    /// gated; the third is recorded.
    #[test]
    fn the_pictures_size_and_the_rate_agree_and_the_total_printed_beside_them_does_not() {
        assert_eq!(PICTURE_BYTES, 549_450);
        assert_eq!(
            Wire::picture_kib().trunc(),
            536.0,
            "§15's 536 is 536.57 KiB truncated; rounded it is 537, and the total printed beside it \
             is built out of the truncation"
        );

        let slow = photographs(Repeat::SlowerThanTheDecode);
        assert_eq!(slow.bytes, 10_989_000);
        assert_eq!(slow.rate(), WIRE_RATE);
        assert!(
            (slow.total_mb() - 10.989).abs() < 0.001,
            "the product is {} MB",
            slow.total_mb()
        );

        // The reading the design printed, reconstructed: the rounded KiB figure, read as kB.
        let as_printed = Wire::picture_kib().trunc() * f64::from(slow.pictures) / 1_000.0;
        assert!(
            (as_printed - SECTION_15_TOTAL_MB).abs() < 0.03,
            "10.7 MB is {as_printed} MB, which is the rounded picture read in the other unit"
        );
        assert!(
            (slow.total_mb() - SECTION_15_TOTAL_MB).abs() > 0.2,
            "and it is not the product the rate beside it is computed from"
        );
    }

    /// **The four defects are four different defects**, which is why the row is a table.
    ///
    /// Unclamped draws nothing at all; clamped and kept opens the new file at row 726, its last
    /// page; reset on the request scrolls the previous file — still on screen — to its top for the
    /// length of the decode; reset on the landing is right on arrival and forgets on return.
    #[test]
    fn each_of_the_five_offset_spellings_is_wrong_in_its_own_way() {
        let unclamped = offsets(Offset::Unclamped);
        assert_eq!(unclamped.arrival_first_row, None);
        assert_eq!(unclamped.arrival_rows, 0);
        assert_eq!(
            unclamped.arrival_content, 0,
            "the body draws nothing at all: a landing is a shrink, from another thread for the \
             first time"
        );

        let kept = offsets(Offset::ClampedKept);
        assert_eq!(
            kept.arrival_first_row,
            Some(SHORT_LAST_PAGE),
            "the new file opens at row 726, its last page"
        );
        assert_eq!(kept.arrival_rows, u32::from(BODY_H));

        let on_request = offsets(Offset::ResetOnRequest);
        assert_eq!(
            on_request.decoding_shows.as_deref(),
            Some("long"),
            "the previous file is still on screen while the next one decodes"
        );
        assert_eq!(
            on_request.decoding_first_row,
            Some(0),
            "and it has been scrolled to its top for the length of the decode"
        );

        let on_landing = offsets(Offset::ResetOnLanding);
        assert_eq!(on_landing.arrival_first_row, Some(0), "right on arrival");
        assert_eq!(on_landing.return_first_row, Some(0), "forgets on return");

        let map = offsets(Offset::PerFileMap);
        assert_eq!(map.arrival_first_row, Some(0));
        assert_eq!(
            map.return_first_row,
            Some(LONG_LAST_PAGE),
            "right in both directions, which is what the release argument has to be made against"
        );
    }

    /// **The three spellings that are not reset-on-the-landing are all right about the decode**, so
    /// the arm that ships is separated from them by one moment and not by four.
    #[test]
    fn only_the_request_reset_moves_the_file_that_is_still_on_screen() {
        for spelling in Offset::ALL {
            let case = offsets(spelling);
            assert_eq!(case.decoding_shows.as_deref(), Some("long"));
            let want = if spelling == Offset::ResetOnRequest {
                Some(0)
            } else {
                Some(LONG_LAST_PAGE)
            };
            assert_eq!(
                case.decoding_first_row,
                want,
                "{} moved the previous file while it was still on screen",
                spelling.name()
            );
        }
    }

    /// **The per-file map is refused on release and the arithmetic is the original's.**
    ///
    /// `1 000 000 * 13 * 8 / 7` is **14 857 142**, and the shipped map is worse than that: buckets
    /// round to a power of two and `(u64, u32)` pads to sixteen bytes rather than packing to twelve.
    /// The estimate is gated because it is the recorded one; the measurement is asserted to be larger, because
    /// a refusal that got easier when it was checked would be worth checking again.
    #[test]
    fn a_per_file_map_at_a_million_is_fourteen_million_bytes_against_a_slots_four() {
        assert_eq!(map_bytes(1_000_000), MAP_AT_A_MILLION);
        assert_eq!(SLOT_BYTES, 4);
        assert_eq!(map_bytes(1_000_000) / SLOT_BYTES, 3_714_285);

        // **The relation and not the figure.** The shipped map is 35 651 584 bytes here, and that
        // number is the standard library's rather than this crate's — gating it would be gating
        // somebody else's implementation. What is a property of the mechanism is that the map is
        // *larger* than the estimate, so the refusal does not get easier when it is checked.
        let measured = measured_map_bytes(1_000_000);
        assert!(
            measured > MAP_AT_A_MILLION,
            "the shipped map is {measured} bytes, which is not larger than the estimate"
        );
    }

    /// **R18 the unenforced ordering, and no thread is involved in it.**
    ///
    /// `Task::take` is destructive. A view that asks for the answer where it wants it takes it in
    /// the first consumer and leaves the second looking at what was there before — 20 torn frames
    /// of 20 selections against 0, read off the drawn surface.
    #[test]
    fn a_landing_taken_inside_the_draw_tears_every_frame_it_lands_on() {
        assert_eq!(
            [torn(Taken::InsideTheDraw), torn(Taken::AtTheTopOfTheView)],
            TORN
        );
    }

    /// **The screen partitions its rectangle, and that is what makes it blind.**
    ///
    /// Every arm writes [`CELLS`] once, so `writes == distinct` and the sentinel would be green —
    /// and the offset defect is invisible at both counters. `content_writes` is the one that is not
    /// blind. The pair, 1 650 against 4 166, is a screen that did not partition; the
    /// difference between them is 74 x 34, which is where [`LINE_COLUMNS`] comes from.
    #[test]
    fn every_cell_is_written_once_and_the_counter_that_sees_the_defect_is_the_content() {
        let shape = shape(Build::correct());
        assert_eq!(shape.writes, u64::from(CELLS));
        assert_eq!(shape.distinct, shape.writes, "no cell twice");
        assert_eq!(shape.regions, REGIONS, "the pane's entry and its two bars");
        assert_eq!(shape.verbs, VERBS);
        assert!(shape.verbs <= shape.writes);
        assert_eq!(shape.content_writes, CONTENT_WRITES);
        assert_eq!(shape.customs, 0, "a text document's cells are the theme's");

        assert_eq!(CONTENT_WRITES, 2_516);
        assert_eq!(
            SECTION_15_WRITES[1] - SECTION_15_WRITES[0],
            CONTENT_WRITES,
            "§15's own pair decomposes into 74 rows of 34 columns"
        );
        assert_eq!(u64::from(BODY_H) * u64::from(LINE_COLUMNS), CONTENT_WRITES);
    }

    /// **The headline pair is a partition failure, and its *difference* reproduces exactly.**
    ///
    /// [`shape`] plays one file and one landing, where every spelling is identical: the defect is
    /// a **shrink**, from another thread for the first time, so it is only visible on the frame the
    /// document gets smaller. There the unclamped arm writes [`UNCLAMPED_WRITES`] of [`CELLS`] —
    /// and the 2 516 it leaves behind is [`CONTENT_WRITES`], because the body cannot write rows the
    /// document has not got and an unbounded extent leaves the area no tail to cover them with.
    ///
    /// The magnitudes are another screen's — 4 166 against 1 650 — and **the gap is 2 516 in
    /// both**, which is what makes [`LINE_COLUMNS`] derived rather than chosen.
    #[test]
    fn the_unclamped_spelling_leaves_the_documents_own_cells_unwritten_on_the_frame_it_shrinks() {
        let unclamped = shape_after_the_shrink(Offset::Unclamped);
        assert_eq!(unclamped.writes, UNCLAMPED_WRITES);
        assert_eq!(unclamped.distinct, unclamped.writes, "no cell twice");
        assert_eq!(unclamped.content_writes, 0);
        assert_eq!(
            u64::from(CELLS) - unclamped.writes,
            CONTENT_WRITES,
            "the cells nobody writes are exactly the document's own"
        );
        assert_eq!(
            SECTION_15_WRITES[1] - SECTION_15_WRITES[0],
            u64::from(CELLS) - unclamped.writes,
            "two different screens, one gap — and the half that is evidence is that the deficit is \
             the document's own cells, because `CONTENT_WRITES` is derived from §15's pair and so \
             its agreeing with it is arithmetic"
        );

        // **And the other four are a partition on the same frame**, which is what makes the row a
        // property of the extent rather than of the shrink.
        for spelling in Offset::ALL {
            if spelling == Offset::Unclamped {
                continue;
            }
            let shape = shape_after_the_shrink(spelling);
            assert_eq!(
                shape.writes,
                u64::from(CELLS),
                "{} did not partition its rectangle",
                spelling.name()
            );
            assert_eq!(shape.content_writes, CONTENT_WRITES);
        }
    }

    /// **An answer to a question nobody asked writes nothing**, and no test on the answer's arrival
    /// could have seen it.
    ///
    /// The generation is current — the job was started for the right question — so [`Task::take`]
    /// hands the payload over and the runtime's own half is satisfied. The equality that catches it
    /// is [`Preview::shows`] against [`Task::asking`], eight bytes on the payload, and the pane
    /// drops the answer **before a cell is written**.
    #[test]
    fn an_answer_to_a_question_nobody_asked_is_dropped_without_a_cell_written() {
        let honest = mismatched(true);
        let lying = mismatched(false);

        assert_eq!(
            [honest.answered, lying.answered],
            [1, 1],
            "the same question was asked and the same job ran"
        );
        assert_eq!([honest.landings, lying.landings], [1, 0]);
        assert_eq!([honest.refused, lying.refused], [0, 1]);
        assert_eq!(
            [honest.content_writes, lying.content_writes],
            [CONTENT_WRITES, 0],
            "the mismatched answer writes nothing at all"
        );
    }

    /// **Requirement 9, as a count and not a microsecond: 0 decode units on the app thread.**
    ///
    /// The same decode, over the same twenty selections, with the answer still arriving. The claim is
    /// the other arm at 5 076 units and this document is smaller; what is gated is the pair, because
    /// the claim is about which thread ran them.
    #[test]
    fn no_decode_unit_runs_while_the_app_thread_is_inside_its_frame() {
        assert_eq!(decoded(DecodeAt::OnTheWorker), 0);
        assert_eq!(decoded(DecodeAt::InTheView), DECODE_UNITS_IN_THE_VIEW);
        assert_eq!(DECODE_UNITS_IN_THE_VIEW, 1_480);
        const {
            assert!(
                SECTION_15_DECODE_UNITS > DECODE_UNITS_IN_THE_VIEW,
                "§15's document is larger than this one, which is why the gate is the pair"
            );
        }
    }

    /// **R02's sweep, refused twice, over two different things and for two different reasons.**
    ///
    /// *A job's lifetime is the question's, a memo's is the data's, and neither is the widget's.*
    /// Ten tab switches cost 1 spawn and 1 decode with the slot in application state and 10 and 10
    /// with it swept, and the derived value the same 1 against 10 — one number a refusal.
    #[test]
    fn ten_tab_switches_cost_one_spawn_and_one_fold_and_not_ten_of_each() {
        let kept = sweeps(false);
        let swept = sweeps(true);

        assert_eq!([kept.spawns, swept.spawns], SWEPT_SPAWNS);
        assert_eq!([kept.decodes, swept.decodes], SWEPT_SPAWNS);
        assert_eq!([kept.folds, swept.folds], SWEPT_FOLDS);
        assert_eq!(TAB_SWITCHES, 10);
    }

    /// **A landing that did not happen is still a drop**, which is R09's `Edit` arriving where
    /// nobody looks for it.
    ///
    /// The guard is taken inside the landing branch, so the revision advances 20 times over 120
    /// frames; taken outside it, the highlighter re-folds the whole file **119** times — every frame
    /// but the one with no document yet — and the two screens are cell-identical.
    #[test]
    fn the_revision_advances_on_a_landing_and_not_on_a_drop_that_did_not_land() {
        let shipped = revisions(Bump::OnTheLanding);
        let every = revisions(Bump::EveryFrame);

        assert_eq!([shipped.folds, every.folds], REVISION_FOLDS);
        assert_eq!(REVISION_FOLDS, [20, 119]);
        assert_eq!([shipped.landings, every.landings], [20, 20]);
        assert_eq!([shipped.frames, every.frames], [REVISION_FRAMES; 2]);
        assert_eq!(
            shipped.writes, every.writes,
            "the two screens are cell-identical, which is why no counter that reads a cell can \
             separate them"
        );
    }

    /// **The steady frame is identical at 1 000, 100 000 and 1 000 000 entries**, with a worker in
    /// the frame and a landing in its history.
    ///
    /// That is `CONTEXT.md`'s invariant — *frame cost is proportional to visible cells, never to
    /// data volume* — through a component that holds an asynchronous answer. The magnitudes,
    /// 3 557 writes and 197 verbs, are a prototype's screen; this one partitions 300 x 80.
    #[test]
    fn the_steady_frame_is_the_same_at_a_thousand_a_hundred_thousand_and_a_million() {
        let first = steady(VOLUMES[0]);
        for volume in VOLUMES {
            let shape = steady(volume);
            assert_eq!(shape, first, "the frame moved at {volume} entries");
        }
        assert_eq!(first.writes, u64::from(CELLS));
        assert_eq!(first.verbs, VERBS);
        assert_eq!(first.regions, REGIONS);
        assert_eq!(first.customs, 0);
        assert_ne!(
            first.writes, SECTION_15_STEADY[0],
            "§15's 3 557 is a prototype's screen; what reproduces is that nothing moves"
        );
    }

    /// **A million entries with a photograph selected: 14 652 customs**, one a cell of the pane's
    /// viewport, which is the number and the join with the media family.
    ///
    /// A preview pane is *not a cheap version of a full-screen picture; it is most of one*. What
    /// what this screen adds is that the customs are a **partition** of the pane's viewport
    /// rather than of the terminal, so the number is [`BODY_CELLS`] and derivable rather than
    /// measured.
    #[test]
    fn a_photograph_in_the_pane_spends_one_custom_a_cell_of_the_body() {
        let shape = photograph_frame(VOLUMES[2]);
        assert_eq!(shape.customs, PHOTOGRAPH_CUSTOMS);
        assert_eq!(PHOTOGRAPH_CUSTOMS, 14_652);
        assert_eq!(shape.customs, u64::from(BODY_CELLS));
        assert_eq!(shape.customs, SECTION_15_PHOTOGRAPH[2], "§15's own");
        assert_eq!(
            shape.content_writes, PHOTOGRAPH_CUSTOMS,
            "one verb a cell, because a run needs one paint and two adjacent cells have not got one"
        );
        assert_eq!(shape.writes, u64::from(CELLS), "and still a partition");
        assert!(
            shape.verbs > steady(VOLUMES[2]).verbs * 30,
            "a photograph is one verb a cell and a text document is one a row"
        );
    }

    /// **A shut picker's face is a partition of its row at every width from 1 to 40.**
    ///
    /// The rule on the one drawing this component inherited from somewhere else, and the reason it
    /// is gated rather than read: the shape has been wrong before. `glyphs::elide` reserves the
    /// marker's cell, so padding to the *whole* width and then writing the marker writes one cell
    /// **twice** — on every truncated widget, invisible on the screen and invisible to every counter
    /// but the pair. Components 26 found it in `select`; the copy inherited the shape and now
    /// inherits the gate.
    ///
    /// **And the counter is watched seeing it**, on the same three verbs at the same width, because
    /// a pair asserted only in the direction that passes is a pair nobody has watched.
    ///
    /// **Over heights as well**, which is where the other half of the partition rule met this
    /// component: the face is the rectangle it was handed and not the first row of it. See
    /// [`shut_faces`], whose header carried the sentence this corrected.
    #[test]
    fn a_shut_pickers_face_is_a_partition_of_its_row_at_every_width() {
        let faces = shut_faces();
        assert_eq!(faces.len(), 120);
        for face in &faces {
            assert_eq!(
                face.writes,
                u64::from(face.w) * u64::from(face.h),
                "the shut face left a cell of its {}x{} rectangle unwritten",
                face.w,
                face.h
            );
            assert_eq!(
                face.distinct, face.writes,
                "the shut face wrote a cell twice at {}x{}",
                face.w, face.h
            );
        }

        // **The mistake, watched.** The same row written the way the defect writes it: the label
        // padded to the whole width and the marker over the pad's last cell.
        let w = 12u16;
        let mut driver = Driver::headless(w, 1).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| {
            let paint = cx.theme().paint(Role::Body);
            let (shown, tail) = crate::glyphs::elide(cx.theme(), "a-long-file-name", w);
            let _ = tally.pad_to(cx, 0, 0, shown, w, paint);
            let _ = tally.text(cx, i32::from(w) - 1, 0, tail, paint);
        });
        assert_eq!(
            tally.writes(),
            u64::from(w) + 1,
            "the defective spelling writes one cell twice"
        );
        assert_eq!(tally.distinct(), u64::from(w));
    }

    /// **The picker is `collection` + `overlay` + the pane, and it introduces no mechanism the three
    /// do not already have.**
    ///
    /// R3's claim is the class that turns out to be false when it is false, so it is checked twice:
    /// as a **source scan** for the four mechanisms it may not mint, and as a **subtraction** over
    /// what a standing picker declares — the owner's shut face, the shell's blur position, the
    /// collection's one entry however many rows it has, and the pane's three.
    #[test]
    fn a_file_picker_is_three_components_and_no_fourth_mechanism() {
        let src = picker_source();
        assert!(!src.is_empty(), "`file_picker` is not declared");
        for (needle, why) in PICKER_MAY_NOT {
            assert!(
                !crate::dense::declares(&src, needle),
                "`file_picker` names `{needle}`, and {why}"
            );
        }
        for call in PICKER_CALLS {
            assert!(
                crate::dense::declares(&src, call),
                "`file_picker` does not call `{call}`, so it is not that composition"
            );
        }

        let shut = picker(false);
        let open = picker(true);
        assert_eq!([shut.regions, open.regions], PICKER_REGIONS);
        assert_eq!(
            open.regions,
            PICKER_DECOMPOSED.iter().sum::<usize>(),
            "the six are a subtraction: {PICKER_DECOMPOSED:?}"
        );
        assert_eq!([shut.overlays, open.overlays], [0, 1]);
        assert_eq!(
            [shut.merges, open.merges],
            [0, 0],
            "two widgets under one id would be one, and the count is the only thing that says so"
        );
    }

    /// **The body is 198 x 74 and 61.0% of the screen**, which is the arithmetic.
    #[test]
    fn the_body_is_fourteen_thousand_six_hundred_and_fifty_two_cells() {
        assert_eq!(BODY_CELLS, 14_652);
        assert!((body_percent() - 61.05).abs() < 0.01);
        // **The pane's rectangle is the body plus one reserved gutter each way**, which is the design's
        // *positive case for bars-reserved* as arithmetic: 198 x 74 is what the component hands its
        // line drawer and 199 x 75 is what it is handed.
        assert_eq!(PANE_W, BODY_W + 1);
        assert_eq!(PANE_H, BODY_H + 1);
        assert_eq!(u32::from(LIST_W) + 1 + u32::from(PANE_W), u32::from(W));
        assert_eq!(
            u32::from(CHROME_ROWS) + u32::from(PANE_H) + u32::from(FOOT_ROWS),
            u32::from(H)
        );
        assert_eq!(pane_area().w, PANE_W);
        assert_eq!(pane_area().h, PANE_H);
        assert_eq!(LONG_LAST_PAGE, 3_926);
        assert_eq!(SHORT_LAST_PAGE, 726);
    }

    /// **The pane really asks, on every frame, with the question it was handed.**
    ///
    /// Three counters and not two: a screen that counted the questions it *handed over* would
    /// report 103 for a component that asked none, so what is read is [`Task::asking`] **after** the
    /// component has run. It agrees with the other two on every frame, deduplicated ones included.
    #[test]
    fn the_pane_forwards_every_question_it_is_handed_including_the_deduplicated_ones() {
        let files = vec![long_file(), short_file()];
        let mut screen = Screen::open(Build::correct());
        for step in 0..6 {
            screen.frame(&files, usize::from(step >= 3));
        }
        assert_eq!(screen.frames(), 6);
        assert_eq!(screen.requests(), 6);
        assert_eq!(screen.forwarded(), 6);
        assert_eq!(
            screen.spawns(),
            2,
            "six questions, two distinct: the dedupe is the runtime's and the count is not"
        );
    }

    /// **The three screens are waiting for their subject, and the sentence says which failure it
    /// is.**
    ///
    /// A scene that fails because it is unimplemented and a scene that fails because the code is
    /// wrong are the same failure unless the message separates them.
    #[test]
    fn the_screens_stand_on_file_preview_pane_and_file_picker() {
        assert_eq!(subjects_declared(), SUBJECTS.to_vec());
        let verdict = standing();
        assert!(verdict.met(), "{verdict:?}");
        assert!(
            owed_message(&subjects_declared(), "scene 25").is_none(),
            "both subjects are declared, so nothing is owed"
        );
        assert_stands_up("scene 23");
        assert_stands_up("scene 24");
        assert_stands_up("scene 25");

        // **And the signature owes more than its own name.** Each of these could be deleted while
        // leaving the declaration scan green, and each is a mechanism the design assigns away from the
        // widget.
        let src = files_source();
        for (needle, why) in PANE_OWES {
            assert!(
                crate::dense::declares(&src, needle),
                "`file_preview_pane` does not take `{needle}`, and {why}"
            );
        }
    }

    /// **The waiting message stops when it should**, which is the half nobody watches.
    #[test]
    fn the_waiting_message_says_which_failure_it_is_and_stops_when_it_should() {
        assert!(owed_message(&[], "scene 23").is_some());
        assert!(owed_message(&["file_preview_pane"], "scene 23").is_some());
        assert!(owed_message(&SUBJECTS, "scene 23").is_none());
    }

    /// **The scan fires in both directions.**
    ///
    /// A source scan that has only ever been run over a file with nothing in it is a scan nobody has
    /// watched. Both needles are handed a source that carries them and one that does not, and the
    /// comment case is the one that matters — the paragraph above `DECLARATIONS` mentions both
    /// declarations by name.
    #[test]
    fn the_subject_scan_finds_a_declaration_when_there_is_one() {
        for (_, declaration) in DECLARATIONS {
            let body = format!("{declaration}cx: &mut Ctx) {{}}");
            assert!(crate::dense::declares(&body, declaration));
            assert!(!crate::dense::declares(&format!("// {body}"), declaration));
            assert!(!crate::dense::declares("fn nothing() {}", declaration));
        }
    }

    /// **The task scan fires in both directions**, and it is the half a longer needle could not buy.
    ///
    /// The sweep is refused twice, and the second refusal is about a job's lifetime: a pane that
    /// minted its own `Task` would cost **10 spawns and 10 decodes over ten tab switches against 1
    /// and 1**, whatever its signature said. Nothing is declared today, so the scan answers `false`
    /// over an empty file — which is why the predicate is watched over two sources instead.
    #[test]
    fn the_task_scan_fires_in_both_directions() {
        assert!(task_minted_in(
            "pub fn file_preview_pane(cx: &mut Ctx) { let task = Task::new(&worker); }"
        ));
        assert!(!task_minted_in(
            "pub fn file_preview_pane(cx: &mut Ctx, task: &Task<Doc>) {}"
        ));
        assert!(!mints_its_own_task(), "there is no pane to mint one");
    }

    /// **The reports run**, which is what keeps a `fn main()` from being the only thing that has.
    #[test]
    fn the_two_tables_have_a_row_for_every_arm() {
        assert_eq!(offset_table().lines().count(), Offset::ALL.len() + 1);
        assert_eq!(wire_table().lines().count(), 3);
    }
}
