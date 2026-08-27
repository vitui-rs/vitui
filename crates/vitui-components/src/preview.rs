//! **The file preview pane's three screens: a re-sort, a streamed directory and twenty selections.**
//!
//! Components ticket 31. Spec §15, §21. This is the screen the three preview-pane scenes are scenes
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
//! [`Offset`] is §15's table as an enum, on a 4 000-row file, an 800-row file and a 74-row
//! viewport. Four of the five are defects and each is a **different** one — that is the whole
//! reason the row is a table and not a sentence — and the fifth, the per-file map, is right in both
//! directions and refused on release: [`map_bytes`] against [`SLOT_BYTES`].
//!
//! # What tears, and why it tears without a race
//!
//! [`Task::take`] is **destructive**: a landing is taken once. A view that asks for it wherever it
//! happens to need it therefore hands the *first* caller the answer and the second `None`, on the
//! same frame, with no thread having done anything at all. [`Taken`] is that axis and
//! [`torn`] is the count — R18 §7's unenforced ordering as **20 torn frames of 20** against **0**,
//! with a status row on each side of the pane and the tear read back off the drawn surface.
//!
//! # The screen partitions and §15's headline number does not
//!
//! §15 prices the unclamped offset at *1 650 writes against 4 166*, and a screen that writes every
//! cell of its rectangle — §2's rule, which every component in this crate obeys — writes
//! [`CELLS`] either way. So this screen reports **two** numbers: [`Shape::writes`], which is 24 000
//! on every arm because the partition holds, and [`Drawn::content_writes`], which is the body's own
//! document cells and is **0** when the body draws nothing at all. §15's pair is printed
//! beside them rather than engineered into them, and one thing is read *out* of it: 4 166 − 1 650 is
//! **2 516 = 74 × 34**, so the prototype's document was 34 columns wide, which is where
//! [`LINE_COLUMNS`] comes from.

use std::collections::HashMap;
use std::fmt::Write as _;

use vitui_runtime::Rect;
use vitui_runtime::ctx::Driver;
use vitui_runtime::theme::Role;
use vitui_runtime::work::{Requested, Task, Worker};

use crate::counters::Tally;
use crate::ink::{Direct, Ink};
use crate::obligations::Verdict;
use crate::runner::{Canvas, Pen};

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The screen's width. §20's full-screen class, and the width §15's 61.0% is a percentage of.
pub const W: u16 = 300;

/// The screen's height.
pub const H: u16 = 80;

/// How many cells that is.
pub const CELLS: u32 = W as u32 * H as u32;

/// **The preview body's width.** §15's own: `198 x 74 = 14 652 cells, 61.0% of the screen`.
pub const BODY_W: u16 = 198;

/// The preview body's height, which is also the viewport §15's offset table is written against.
pub const BODY_H: u16 = 74;

/// How many cells the body is. **14 652**, and [`body_percent`] is what §15 states beside it.
pub const BODY_CELLS: u32 = BODY_W as u32 * BODY_H as u32;

/// The file list's width. `W - BODY_W - 1`, the one being the vertical rule between them.
pub const LIST_W: u16 = W - BODY_W - 1;

/// How many rows of chrome sit above the band, and below it. A status row on **each** side of the
/// pane is §15's own arrangement for the torn-frame case, and it is what [`torn`] reads back.
pub const CHROME_ROWS: u16 = 3;

/// **The document's line width, read out of §15's own pair.**
///
/// §15 prices the unclamped offset at *1 650 writes against 4 166*. The difference is the body's own
/// content — **2 516** — and 2 516 is **74 × 34** exactly, over a 74-row viewport. So the
/// prototype's preview document was 34 columns wide and this one is too. The number is derived
/// rather than chosen, which is the difference between reproducing a figure and engineering one.
pub const LINE_COLUMNS: u16 = 34;

/// What §15's *the body draws nothing at all* is, as this screen's own counter: 74 rows of
/// [`LINE_COLUMNS`].
pub const CONTENT_WRITES: u64 = BODY_H as u64 * LINE_COLUMNS as u64;

/// §15's own pair for the same fact, on a screen that did not partition its rectangle.
pub const SECTION_15_WRITES: [u64; 2] = [1_650, 4_166];

/// The whole screen.
pub fn whole() -> Rect {
    Rect::new(0, 0, W, H)
}

/// The file list's rectangle.
pub fn list_area() -> Rect {
    Rect::new(0, i32::from(CHROME_ROWS), LIST_W, BODY_H)
}

/// The one-column vertical rule between the list and the body. `file_preview_pane` declares
/// [`Glyph::VLine`](vitui_runtime::Glyph::VLine) in [`crate::INVENTORY`]; this is the column it is
/// drawn in.
pub fn rule_area() -> Rect {
    Rect::new(i32::from(LIST_W), i32::from(CHROME_ROWS), 1, BODY_H)
}

/// **The preview body's rectangle.** 198 x 74.
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

/// The row the bottom status is written on. **A status row on each side of the pane** is what makes
/// a torn frame visible: one value, read twice, printed twice.
pub const BOTTOM_STATUS_Y: i32 = (H - CHROME_ROWS + 1) as i32;

/// What the body is as a percentage of the screen, to two places. §15 states **61.0%**.
pub fn body_percent() -> f64 {
    f64::from(BODY_CELLS) * 100.0 / f64::from(CELLS)
}

// ── the axes of a build ──────────────────────────────────────────────────────────────────────────

/// **What the pane names its question with.**
///
/// The whole of §10's memo-key rule, arriving through the one door where the natural spelling is the
/// wrong one: a pane's question is a *file* and a cursor is a *position*.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    /// The cursor's index in the ordered listing. **The defect**, and the one anybody writes first.
    Position,
    /// The file's own identity. What ships.
    Identity,
}

/// **Where the offset goes when the selection changes.** §15's five-row table.
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

    /// The name §15's table gives it.
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

/// **Where the landing is taken.** R18 §7's unenforced ordering, as an axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Taken {
    /// Once, at the top of the view, into a local both status rows read.
    TopOfView,
    /// Wherever the value happens to be wanted. [`Task::take`] is destructive, so the second reader
    /// gets `None` — **on the same frame, with no thread having done anything**.
    InsideDraw,
}

/// **How fast the selection repeats, against how long a decode takes.**
///
/// §15 states the crossover as a relation and not as a number of milliseconds: *1 picture when the
/// key repeats faster than the decode, 20 pictures at a 30 ms repeat when it does not*. The arms
/// are named for the relation for that reason — the ticket's own paraphrase (*a 30 ms key repeat
/// and a slower one*) reads the crossover the other way round, and §15 is the authority.
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

/// **One screen, with the three arms as fields.** A comparison is one field changed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Build {
    /// What the question is named with.
    pub key: Key,
    /// Where the offset goes.
    pub offset: Offset,
    /// Where the landing is taken.
    pub taken: Taken,
}

impl Build {
    /// What ships: an identity key, the offset reset on the landing, the answer taken once.
    pub const fn correct() -> Build {
        Build {
            key: Key::Identity,
            offset: Offset::ResetOnLanding,
            taken: Taken::TopOfView,
        }
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
}

/// How many files the two 200-file scenes list. §21's own count for both.
pub const FILES: usize = 200;

/// **Where the cursor sits.** Not one of [`FIXED`], because a re-sort that left the cursor's own
/// file alone would be a re-sort with nothing to decide.
pub const CURSOR: usize = 42;

/// **The three positions a re-sort leaves alone**, which is what makes the moved count 197 and not
/// 200. §21 states *197 of 200 positions move*; three fixed points is what states it.
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

/// **How many positions hold a different file in `b` than in `a`.** §21's *197 of 200*.
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

/// The 4 000-row file §15's offset table is written against.
pub fn long_file() -> File {
    File {
        id: 0xA1,
        name: String::from("long"),
        rows: 4_000,
        size: 0,
    }
}

/// The 800-row file beside it.
pub fn short_file() -> File {
    File {
        id: 0xB2,
        name: String::from("short"),
        rows: 800,
        size: 1,
    }
}

/// The long file's last page. **3 926**, and what a user who pressed `End` is looking at.
pub const LONG_LAST_PAGE: u32 = 4_000 - BODY_H as u32;

/// The short file's last page. **726**, which is §15's own number for *clamped and kept*.
pub const SHORT_LAST_PAGE: u32 = 800 - BODY_H as u32;

// ── one frame ────────────────────────────────────────────────────────────────────────────────────

/// **What one frame of the pane drew**, as counters and as the two status rows.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Drawn {
    /// **Cells of the body carrying the document.** The counter that is not blind: zero when the
    /// body draws nothing at all.
    pub content_writes: u64,
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

/// **The screen, as a thing that can be handed a listing and a cursor and asked for a frame.**
///
/// It holds a [`Worker::queueing`] rather than a resident one, which is [`Worker::queueing`]'s own
/// argument: *a gate over staleness arithmetic is a gate over an order, and the only thing a real
/// thread contributes to it is that order.* Every count on this screen is a count over a schedule
/// the caller wrote down.
pub struct Screen {
    /// Which arms are standing.
    pub build: Build,
    driver: Driver,
    worker: Worker,
    task: Task<Doc>,
    shown: Option<Doc>,
    offset: u32,
    remembered: HashMap<u64, u32>,
    requests: u32,
    frames: u32,
    ran: usize,
}

impl Screen {
    /// A screen with nothing asked and nothing shown.
    pub fn open(build: Build) -> Screen {
        let worker = Worker::queueing();
        let task = Task::new(&worker);
        Screen {
            build,
            driver: Driver::headless(W, H).expect("a sink cannot fail to attach"),
            worker,
            task,
            shown: None,
            offset: 0,
            remembered: HashMap::new(),
            requests: 0,
            frames: 0,
            ran: 0,
        }
    }

    /// How many times the pane has called [`Task::request`]. **Every frame, on every arm.**
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

    /// How many questions actually reached the worker. **The counter that separates the two keys.**
    pub fn spawns(&self) -> u64 {
        self.worker.asked()
    }

    /// The document offset the body is drawing from.
    pub fn offset(&self) -> u32 {
        self.offset
    }

    /// Move the offset, the way a `PageDown` would.
    pub fn scroll_to(&mut self, offset: u32) {
        self.offset = offset;
    }

    /// Which file the pane is showing.
    pub fn showing(&self) -> Option<&Doc> {
        self.shown.as_ref()
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

    /// Take the landing, if one is waiting, and put the offset where this build's spelling says.
    fn land(&mut self) -> bool {
        let Some(doc) = self.task.take() else {
            return false;
        };
        match self.build.offset {
            // Left exactly where it was. A landing is a shrink and nothing shrank with it.
            Offset::Unclamped => {}
            Offset::ClampedKept => self.offset = self.offset.min(doc.max_offset()),
            // Already zeroed, at the request.
            Offset::ResetOnRequest => {}
            Offset::ResetOnLanding => self.offset = 0,
            Offset::PerFileMap => {
                self.offset = self.remembered.get(&doc.id).copied().unwrap_or(0);
            }
        }
        self.shown = Some(doc);
        true
    }

    /// **One frame**, drawn through `ink`.
    ///
    /// The order is §15's: the landing is taken at the top of the view, the question is asked, and
    /// then the screen is drawn from one value. [`Taken::InsideDraw`] is the same frame with the
    /// first of those three moved into the third, which is the whole of the torn-frame case.
    ///
    /// # Panics
    ///
    /// Panics when `cursor` is past the end of `listing`, which is a schedule that did not happen.
    pub fn frame_into<I: Ink>(&mut self, ink: &mut I, listing: &[File], cursor: usize) -> Drawn {
        assert!(cursor < listing.len(), "the cursor is past the listing");
        let file = listing[cursor].clone();

        if self.build.taken == Taken::TopOfView {
            self.land();
        }

        let key = match self.build.key {
            Key::Position => cursor as u64,
            Key::Identity => file.id,
        };
        let (id, name, rows) = (file.id, file.name.clone(), file.rows);
        self.requests += 1;
        let requested = self
            .task
            .request(key, move |_cancel| Doc { id, name, rows });
        if requested == Requested::Asked {
            // The map files the offset it is about to leave. It is the only spelling that has
            // anywhere to put one, which is the same sentence as its refusal.
            if self.build.offset == Offset::PerFileMap
                && let Some(doc) = &self.shown
            {
                self.remembered.insert(doc.id, self.offset);
            }
            if self.build.offset == Offset::ResetOnRequest {
                self.offset = 0;
            }
        }

        self.draw(ink, listing, cursor)
    }

    /// One frame through [`Direct`], which is the path a component takes.
    pub fn frame(&mut self, listing: &[File], cursor: usize) -> Drawn {
        self.frame_into(&mut Direct, listing, cursor)
    }

    /// The draw itself. Split out because `Driver::frame` borrows the driver and the body needs the
    /// rest of the screen — [`crate::picture::Session`]'s own arrangement, for its reason.
    fn draw<I: Ink>(&mut self, ink: &mut I, listing: &[File], cursor: usize) -> Drawn {
        self.frames += 1;
        let Screen {
            build,
            driver,
            task,
            shown,
            offset,
            ..
        } = self;
        let build = *build;
        let offset = *offset;
        let mut content_writes = 0u64;
        let mut body_rows = 0u32;
        let mut first_row = None;
        let mut top_status = String::new();
        let mut bottom_status = String::new();
        // What a mid-draw take took, filed once the frame is over. The state ends up correct, which
        // is exactly why a torn frame leaves nothing behind for the next frame to notice.
        let mut taken_mid: Option<Doc> = None;

        driver.frame(|cx| {
            let theme = cx.theme();
            let body_paint = theme.paint(Role::Body);
            let dim = theme.paint(Role::Dim);
            let border = theme.paint(Role::Border);
            let selection = theme.paint(Role::Selection);
            let title = theme.paint(Role::Title);

            // The top chrome: a title, the status, and a rule.
            ink.pad_to(cx, 0, 0, "vitui preview pane", W, title);
            // **The tear lives here, and no thread is involved in it.** `Task::take` is
            // destructive, so a view that asks for the answer where it happens to want it takes it
            // in the *first* consumer — and the second consumer, three quarters of a screen further
            // down, is looking at the value that was there before. A view has no `&mut` to the
            // pane's state, which is R18 §7's point: it cannot put back what it took, so the two
            // rows of one frame are drawn from two versions of one value.
            let top_name: Option<String> = match build.taken {
                Taken::TopOfView => shown.as_ref().map(|d| d.name.clone()),
                Taken::InsideDraw => match task.take() {
                    Some(doc) => {
                        let name = doc.name.clone();
                        taken_mid = Some(doc);
                        Some(name)
                    }
                    None => shown.as_ref().map(|d| d.name.clone()),
                },
            };
            top_status = status(top_name.as_deref());
            ink.pad_to(cx, 0, TOP_STATUS_Y, &top_status, W, dim);
            ink.run(cx, 0, 2, "-", W, border);

            // The listing.
            let list = list_area();
            for row in 0..BODY_H {
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
            for row in 0..BODY_H {
                ink.run(cx, rule.x, rule.y + i32::from(row), "|", 1, border);
            }

            // The body.
            let body = body_area();
            for row in 0..BODY_H {
                let y = body.y + i32::from(row);
                let line = shown.as_ref().and_then(|doc| {
                    let index = offset.checked_add(u32::from(row))?;
                    (index < doc.rows).then(|| doc.line(index))
                });
                match line {
                    Some(text) => {
                        if first_row.is_none() {
                            first_row = Some(offset + u32::from(row));
                        }
                        body_rows += 1;
                        content_writes += u64::from(LINE_COLUMNS);
                        ink.pad_to(cx, body.x, y, &text, body.w, body_paint);
                    }
                    None => {
                        ink.run(cx, body.x, y, " ", body.w, body_paint);
                    }
                }
            }

            // The bottom chrome: a rule, the status again, and the hints.
            ink.run(cx, 0, i32::from(H - CHROME_ROWS), "-", W, border);
            // The second reader. Under `TopOfView` it is reading the same local; under
            // `InsideDraw` there is nothing left to take and it prints what was already on screen.
            let bottom_name: Option<String> = shown.as_ref().map(|d| d.name.clone());
            bottom_status = status(bottom_name.as_deref());
            ink.pad_to(cx, 0, BOTTOM_STATUS_Y, &bottom_status, W, dim);
            ink.pad_to(cx, 0, i32::from(H - 1), "up/down select   q quit", W, dim);
        });

        if let Some(doc) = taken_mid {
            *shown = Some(doc);
        }

        Drawn {
            content_writes,
            body_rows,
            first_row,
            showing: self.shown.as_ref().map(|d| d.name.clone()),
            top_status,
            bottom_status,
        }
    }
}

/// **What one steady frame of the screen costs**, as counters rather than as a picture.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Shape {
    /// Cells written, from the engine's own column reports. **[`CELLS`]**: the screen partitions its
    /// rectangle.
    pub writes: u64,
    /// Distinct cells touched. Equal to [`Shape::writes`], which is §2's *no cell twice*.
    pub distinct: u64,
    /// Drawing verbs.
    pub verbs: u64,
    /// Interactive regions. **Zero**: nothing on this screen declares one, because the components
    /// that would are what it is waiting for.
    pub regions: usize,
    /// Cells of the body carrying the document.
    pub content_writes: u64,
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
    }
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

/// How many frames the re-sort scene counts after the sort. §21's *wrong on 100 of 100 frames*.
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

/// **Scene 25.** 200 files, one re-sort, and a hundred frames afterwards.
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

/// How many batches follow it. §21's seven.
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
    /// **The longest run of them.** §15's *wrong for exactly 1 frame, the decode's latency* is a
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

/// **Scene 24.** Seven batches, no input at all, and a cursor whose file changes under it.
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

/// How many selections the photograph scene plays. §21's twenty.
pub const SELECTIONS: usize = 20;

/// **What a picture costs on the wire**, in bytes: [`BODY_CELLS`] at §14's measured **37.5 B/cell**.
///
/// It is arithmetic and not a measurement, and the difference is register row 161's: no crate above
/// the engine can read a byte the engine wrote. What is measured here is the **count** — how many
/// pictures were drawn — and the bytes are that count times a figure the engine's own map states.
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

    /// The picture's own size in KiB. **536.57**, which §15 prints as *536 KB* — truncated, not
    /// rounded, and that one character is what the total beside it is built out of.
    pub fn picture_kib() -> f64 {
        PICTURE_BYTES as f64 / 1024.0
    }

    /// The total in decimal MB, which is the unit §15's rate is in.
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
        })
        .collect()
}

/// **Scene 23.** Twenty selections at a key repeat, against a decode that does or does not fit
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

/// **What one offset spelling does**, on §15's own three numbers: a 4 000-row file, an 800-row file
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
/// `entries * (8 + 4 + 1) * 8 / 7`, which at a million is **14 857 142** — §15's own number,
/// reproduced exactly from the arithmetic that produces it. See [`measured_map_bytes`] for what the
/// shipped map really takes, which is worse and makes the refusal stronger rather than weaker.
pub fn map_bytes(entries: u64) -> u64 {
    entries * (MAP_KEY_BYTES + MAP_VALUE_BYTES + MAP_CONTROL_BYTES) * 8 / 7
}

/// **What the shipped `HashMap<u64, u32>` really takes at `entries`.**
///
/// The buckets are rounded to a power of two and the pair is padded to 16 bytes rather than packed
/// to 12, so the real figure is not §15's estimate — **35 651 584 at a million entries here**. It is
/// read off the map's own `capacity` rather than assumed.
///
/// **What is gated is the relation and not that number.** A count of the standard library's own
/// allocation is a gate on somebody else's implementation, and §21's rule is that a gate is a
/// property of the mechanism; the property here is that the shipped map is *larger* than §15's
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

// ── the subjects ─────────────────────────────────────────────────────────────────────────────────

/// The components these three screens stand on. **Neither is declared**, which is why all three
/// scenes are red rather than `Unsubjected`: the screens run.
pub const SUBJECTS: [&str; 2] = ["file_preview_pane", "file_picker"];

/// Where [`SUBJECTS`] belong, as `(module file, the declaration)`.
///
/// The home is [`crate::Family::F12Files`]'s, whose module is `files.rs`. A component is
/// `fn(&mut Ctx, Rect, …) -> Response` (spec §1, rule 1), so the thing to look for is a public
/// function of the component's own name in its own family's module.
///
/// # The needle is a parenthesis and the load-bearing half is a separate scan
///
/// This is the third ticket on this backlog to meet the needle problem — components 26 read
/// `pub fn select(` for a component §1 already says costs two lifetime annotations, and components
/// 30 read `pub fn picture(` for one whose `…` is a type parameter. Both would have gone green *by
/// deleting the thing that mattered*.
///
/// The answer here is not a longer needle, because a longer one would be this ticket dictating
/// ticket 32's parameter list. What §15 actually settles is a **negative**: *a job's lifetime is the
/// question's, a memo's is the data's, and neither is the widget's* — R02's sweep refused twice.
/// So the existence scan is the parenthesis, and [`mints_its_own_task`] is the half that cannot be
/// satisfied by deleting anything: a pane that called `Task::new` inside itself would pay **10
/// spawns and 10 decodes over ten tab switches against 1 and 1**, whatever its signature said.
pub const DECLARATIONS: [(&str, &str); 2] = [
    ("files.rs", "pub fn file_preview_pane("),
    ("files.rs", "pub fn file_picker("),
];

/// The file [`SUBJECTS`] are homed in.
const FILES_RS: &str = "files.rs";

/// Read `src/<file>`, or an empty string when it is not there.
fn source(file: &str) -> String {
    let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).join(file);
    std::fs::read_to_string(&path).unwrap_or_default()
}

/// **Which of [`SUBJECTS`] this crate declares. Today: neither.**
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
/// It answers `false` today because there is no pane, which is why it is a **companion** to
/// [`standing`] and not a second verdict: `tests::the_task_scan_fires_in_both_directions` hands it
/// the two sources it must separate.
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
        "neither `file_preview_pane` nor `file_picker` is declared in this crate, so what stands on \
         the three preview-pane screens is a pane written beside them and not the components. The \
         re-sort's 100 of 100 frames against 0 at 103 questions either way, the seven batches with \
         no user in them, the twenty selections at 1 picture against 20, the five offset spellings, \
         the per-file map's 14 857 142 bytes against a slot's 4 and the 20 torn frames of 20 are \
         measured and green; what is missing is the subject",
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
/// Panics while [`SUBJECTS`] are undeclared, which is **today**. Components ticket 32 inverts it.
pub fn assert_stands_up(scene: &str) {
    if let Some(message) = owed_message(&subjects_declared(), scene) {
        panic!("{message}");
    }
}

// ── the report ───────────────────────────────────────────────────────────────────────────────────

/// **The offset table, one row a spelling**, in §15's own column order.
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

/// **The rate the twenty-picture arm runs at, in bytes a second. 18 315 000 — §15's 18.3 MB/s.**
pub const WIRE_RATE: u64 = 18_315_000;

/// **What §15 states the total is**, in decimal MB. It is the rounded 536 KiB read as 536 kB and
/// multiplied by twenty; the rate printed in the same sentence needs the unrounded product.
pub const SECTION_15_TOTAL_MB: f64 = 10.7;

/// **A per-file map at a million entries. 14 857 142 bytes against [`SLOT_BYTES`].**
pub const MAP_AT_A_MILLION: u64 = 14_857_142;

/// **Torn frames, by where the answer is taken: 20 of 20 against 0.**
pub const TORN: [u32; 2] = [SELECTIONS as u32, 0];

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

    /// **Scene 25.** The memo-key rule, through the door nothing else opens.
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

    /// **Scene 24.** The same hole with no user in it.
    ///
    /// Seven batches, no input at all. The settled pane is wrong after **7 of 7** under a position
    /// key and after none under an identity key, and §15's *wrong for exactly 1 frame* is the
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

    /// **Scene 23.** The crossover, read as a count.
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

    /// **§15's three wire figures cannot all be readings of one product, and two of them are.**
    ///
    /// 14 652 cells at 37.5 B/cell is 549 450 bytes — **536.57 KiB**, which §15 prints as *536 KB*
    /// by truncating it — and twenty of them is 10 989 000 bytes, which is **10.99 MB** and is
    /// exactly what *18.3 MB/s over 600 ms* requires. §15's **10.7 MB** is that truncated 536 read
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

        // The reading §15 printed, reconstructed: the rounded KiB figure, read as kB.
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

    /// **The four defects are four different defects**, which is why §15's row is a table.
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

    /// **The per-file map is refused on release and the arithmetic is §15's own.**
    ///
    /// `1 000 000 * 13 * 8 / 7` is **14 857 142**, and the shipped map is worse than that: buckets
    /// round to a power of two and `(u64, u32)` pads to sixteen bytes rather than packing to twelve.
    /// The estimate is gated because it is §15's; the measurement is asserted to be larger, because
    /// a refusal that got easier when it was checked would be worth checking again.
    #[test]
    fn a_per_file_map_at_a_million_is_fourteen_million_bytes_against_a_slots_four() {
        assert_eq!(map_bytes(1_000_000), MAP_AT_A_MILLION);
        assert_eq!(SLOT_BYTES, 4);
        assert_eq!(map_bytes(1_000_000) / SLOT_BYTES, 3_714_285);

        // **The relation and not the figure.** The shipped map is 35 651 584 bytes here, and that
        // number is the standard library's rather than this crate's — gating it would be gating
        // somebody else's implementation. What is a property of the mechanism is that the map is
        // *larger* than §15's estimate, so the refusal does not get easier when it is checked.
        let measured = measured_map_bytes(1_000_000);
        assert!(
            measured > MAP_AT_A_MILLION,
            "the shipped map is {measured} bytes, which is not larger than the estimate"
        );
    }

    /// **R18 §7's unenforced ordering, and no thread is involved in it.**
    ///
    /// `Task::take` is destructive. A view that asks for the answer where it wants it takes it in
    /// the first consumer and leaves the second looking at what was there before — 20 torn frames
    /// of 20 selections against 0, read off the drawn surface.
    #[test]
    fn a_landing_taken_inside_the_draw_tears_every_frame_it_lands_on() {
        assert_eq!([torn(Taken::InsideDraw), torn(Taken::TopOfView)], TORN);
    }

    /// **The screen partitions its rectangle, and that is what makes it blind.**
    ///
    /// Every arm writes [`CELLS`] once, so `writes == distinct` and the sentinel would be green —
    /// and the offset defect is invisible at both counters. `content_writes` is the one that is not
    /// blind. §15's own pair, 1 650 against 4 166, is a screen that did not partition; the
    /// difference between them is 74 x 34, which is where [`LINE_COLUMNS`] comes from.
    #[test]
    fn every_cell_is_written_once_and_the_counter_that_sees_the_defect_is_the_content() {
        let shape = shape(Build::correct());
        assert_eq!(shape.writes, u64::from(CELLS));
        assert_eq!(shape.distinct, shape.writes, "no cell twice");
        assert_eq!(shape.regions, 0);
        assert!(shape.verbs <= shape.writes);
        assert_eq!(shape.content_writes, CONTENT_WRITES);

        assert_eq!(CONTENT_WRITES, 2_516);
        assert_eq!(
            SECTION_15_WRITES[1] - SECTION_15_WRITES[0],
            CONTENT_WRITES,
            "§15's own pair decomposes into 74 rows of 34 columns"
        );
        assert_eq!(u64::from(BODY_H) * u64::from(LINE_COLUMNS), CONTENT_WRITES);
    }

    /// **The body is 198 x 74 and 61.0% of the screen**, which is §15's own arithmetic.
    #[test]
    fn the_body_is_fourteen_thousand_six_hundred_and_fifty_two_cells() {
        assert_eq!(BODY_CELLS, 14_652);
        assert!((body_percent() - 61.05).abs() < 0.01);
        assert_eq!(u32::from(LIST_W) + 1 + u32::from(BODY_W), u32::from(W));
        assert_eq!(u32::from(CHROME_ROWS) * 2 + u32::from(BODY_H), u32::from(H));
        assert_eq!(LONG_LAST_PAGE, 3_926);
        assert_eq!(SHORT_LAST_PAGE, 726);
    }

    /// **The three screens are waiting for their subject, and the sentence says which failure it
    /// is.**
    ///
    /// A scene that fails because it is unimplemented and a scene that fails because the code is
    /// wrong are the same failure unless the message separates them.
    #[test]
    fn the_screens_are_waiting_for_file_preview_pane_and_file_picker() {
        assert_eq!(subjects_declared(), Vec::<&str>::new());
        let verdict = standing();
        assert!(!verdict.met(), "{verdict:?}");
        let message = owed_message(&subjects_declared(), "scene 25")
            .expect("neither subject is declared today");
        assert!(message.contains("file_preview_pane"));
        assert!(message.contains("file_picker"));
        assert!(message.contains("components 32"));
        assert!(message.contains("src/files.rs"));
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
    /// §15 refuses R02's sweep twice, and the second refusal is about a job's lifetime: a pane that
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
