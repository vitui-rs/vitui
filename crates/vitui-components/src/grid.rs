//! **The grid: twelve columns over a million rows, both edges pinned, and the band that overflows.**
//!
//! Components ticket 14. Spec §6, §21. This is the screen `table`'s two scenes are scenes *of*, and
//! it exists for the reason [`crate::listing`] exists one file over: **the defect it is about passed
//! every gate then in force and looked healthier than the correct build** — identical writes,
//! identical verbs, identical time, and a band of cells re-damaged on every steady frame forever.
//!
//! | scene | what it decides |
//! |---|---|
//! | a twelve-column 1M-row table, pinned both edges, horizontal overflow | verbs as the currency, and the band's re-damaged cells |
//! | the same table under a horizontal offset, against itself | what an equality against a reference render can and cannot see |
//!
//! # The `+` in `table = collection + column rectangles` is paid in verbs
//!
//! §6's headline, and it reproduces here **as a structure and not as a magnitude**: the same
//! rectangle, the same [`CELLS`] cells, and a twelve-column table costs [`VERBS_TWELVE`] drawing
//! calls where the same screen drawn as one column costs [`VERBS_ONE`]. Damage is marked once per
//! verb, and C02's partition rule makes each column **two** verbs — the text, then the padding after
//! it — because the alternative is a cell written twice. Two verbs a cell is the floor.
//!
//! §6's own figures are 913 → 2 497 for 23 030 cells at 165.6 µs, on a **prototype screen this
//! ticket does not own**: two tables, a menu bar, a toolbar and a status bar on one 300x80 terminal.
//! One table filling the same terminal is a different screen and reports different magnitudes;
//! `examples/grid_numbers.rs` prints both columns rather than engineering the difference away, which
//! is `crate::dense`'s arrangement and its reason.
//!
//! # The band defect, and the two instruments that see half of it each
//!
//! §6 states it as one sentence: *written the tempting way — the offset as arithmetic, no view — the
//! scrolling band's edge column overwrites the pinned band: identical writes, identical verbs,
//! identical time, and 345 cells re-damaged on every steady frame forever. No gate left by C01, C02
//! or C03 sees it.*
//!
//! Both halves were measured here and **neither reproduces as stated**, in the same direction:
//!
//! 1. **`writes` is not identical, and the reason is a fact about the prototype's counter rather
//!    than about the defect.** `crates/proto-c04-app/src/count.rs` folds a `fill` in as
//!    `r.w * r.h` — the rectangle it *asked* for — while folding a `text` in as `Written::cells`,
//!    which is what landed after clipping. A padded remainder discarded by the band's clip is
//!    therefore counted on the correct arm and counted again on the defective one, and the two come
//!    out level. [`Tally`] keeps [`Tally::asked`] and [`Tally::reported`] apart, so the arms differ
//!    by exactly the overrun: [`BAND_OVERRUN`] columns a row, [`H`] rows, [`RE_DAMAGED`] cells.
//! 2. **The equality against a reference render is blind to it, and that is the scene's finding.**
//!    §6's own sentence says why in the clause before it — *the screen is correct, because the
//!    pinned band draws afterwards and wins* — so a comparison of the two surfaces reports **0 cells
//!    over 0 rows**. Watched doing exactly that in
//!    `tests::the_equality_is_blind_to_the_band_and_the_pair_is_not`.
//!
//! What does see it is the pair `writes` against `distinct`, which is §21's register row 6 — C02's
//! *no cell twice*, filed there as a **report per component**. So the honest form of §6's sentence
//! is that no gate C01, C02 or C03 left behind was *being run*; C02 had already named the counter.
//! The equality's job on this screen is the **other** defect on the same axis, §3.3's inverted
//! horizontal sign, which draws a partly blank band and measures as an improvement — and that one no
//! counter separates.
//!
//! That is [`crate::dense`]'s *two instruments, two halves* one axis over, and it is why §21 lists
//! this screen once and this module stands two scenes on it.
//!
//! # `visible_cols` is as load-bearing as `visible_rows`
//!
//! The column half of *frame cost is proportional to visible cells*. [`ColVirt::ClipOnly`] declares
//! every column and lets the clip discard what does not fit; §6 prices it at **3.3x the frame and
//! 7.2x the verbs** at 120 declared columns. Here the frame is a report and the verbs are a ratio —
//! `tests::the_clip_only_spelling_costs_verbs_and_no_writes_at_all` — and the **writes are
//! identical**, because the engine reports a fully clipped verb as zero columns. That is
//! `crate::listing`'s finding arriving on the column axis: the write count cannot see this defect
//! either.
//!
//! [`Ctx::visible_cols`](vitui_runtime::Ctx::visible_cols) itself is the oracle rather than the
//! fast path, and deliberately: [`ColVirt::Virtualised`] answers the window by binary search over
//! the cumulative array the solve already produced, because **the arithmetic arm has no view to
//! ask**. Two spellings of one window, swept over every offset the content admits, is
//! `tests::the_column_window_agrees_with_the_runtimes_own_visible_cols`.
//!
//! # There is no sticky header on this screen, and the reason is a coordinate system
//!
//! §6's sticky header is a rect split and it costs one `cut`. It is not on this screen, because a
//! header drawn at the table's own row 0 and a body drawn inside
//! [`Ctx::scroll_scope`](vitui_runtime::Ctx::scroll_scope) are **two coordinate spaces**, and both
//! of this crate's recorders — [`Tally`] and [`crate::runner::Pen`] — union what they see in the
//! coordinates of the `Ctx` the verb was called on. `crate::listing` states the same fact from the
//! other side: *a scrolled context **is** a content coordinate system*. Two spaces in one tally make
//! `distinct` meaningless, which is the one counter this scene is decided by.
//!
//! The band is the exception and it is why the two arms are one expression apart:
//! [`Ctx::child`](vitui_runtime::Ctx::child) translates as well as clips, so the band view is
//! immediately scrolled **back** by its own origin. The clip is what the band needs; the translation
//! is not, and undoing it leaves both arms writing the same numbers at the same call sites with one
//! `child` between them.
//!
//! Ticket 15 inherits the omission as a real problem rather than as a simplification: a `table` with
//! a sticky header cannot be measured by either recorder without one of them learning the frame's
//! own origin, and `Ctx` has no public accessor for it.
//!
//! # The gate is red, and it is red for one reason
//!
//! `table` is not declared. [`standing`] is a [`Verdict`] over one subject, [`subjects_declared`]
//! opens the file the freeze homes it in, and [`owed_message`] is the sentence that separates
//! *waiting for its subject* from *the code is wrong* — ticket 09's criterion 7, inherited whole.
//! Inverted by **components 15**.
//!
//! [`Tally`]: crate::counters::Tally
//! [`Tally::asked`]: crate::counters::Tally::asked
//! [`Tally::reported`]: crate::counters::Tally::reported
//! [`Verdict`]: crate::obligations::Verdict

use std::fmt;
use std::time::{Duration, Instant};

use vitui_runtime::layout::text::{truncate, width};
use vitui_runtime::layout::{Constraint, solve};
use vitui_runtime::{Ctx, Density, Id, Interest, Rect, Role, Scrollable};

use crate::counters::{Allocations, Counter, Counters, Tally};
use crate::ink::{Direct, Ink};
use crate::obligations::Verdict;
use crate::runner::{Diff, Fixture, Pen, compare};

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The terminal's width. **Three hundred**, which is §6's own screen and §20's budget line.
pub const W: u16 = 300;
/// The terminal's height. **Eighty**, likewise.
pub const H: u16 = 80;
/// Cells of the rectangle. `300 x 80`, and every arm of every scene here writes a partition of it.
pub const CELLS: u64 = W as u64 * H as u64;

/// The three volumes §21's collection scenes state, reused because a table's row axis is a
/// collection's row axis and §6 asserts the frame flat at all three.
pub const VOLUMES: [u64; 3] = [1_000, 100_000, 1_000_000];

/// The declared-column counts §6 sweeps. **Twelve is the scene's; the other three are the sweep.**
pub const DECLARED: [usize; 4] = [12, 40, 120, 240];

/// The most columns a declaration list may hold. A fixed array, because a `Vec` per table per frame
/// is an allocation per frame and the budget is zero.
pub const MAX_COLS: usize = 256;

/// **The vertical offset every frame here is drawn at. Zero, and it is a finding rather than a
/// default.**
///
/// §21's row 7 states the gesture as `rows: 0` — this scene's axis is the horizontal one, and the
/// vertical one is §21's row 4, `collection`'s. So a zero here would be the right answer anyway.
///
/// It is also **the only answer available**, because
/// [`Ctx::scroll_scope`](vitui_runtime::Ctx::scroll_scope) scrolls the wrong way. See
/// `tests::a_scroll_scope_at_a_nonzero_offset_shows_the_rows_above_the_content`, which pins the
/// measurement: at `offset = (0, 1 000)` the scope answers `visible_rows() == -1000..-920` and a
/// write at content row 1 000 reports **zero columns**. The engine's own rule is that *a viewport
/// scrolled `n` rows down is `scrolled(0, -n)`* and the scope passes `+n`, while
/// `vitui_runtime::scroll::Area::into_view` and `Scrollable::between` both read the offset as
/// positive — so the two halves of the runtime's own scrolling disagree about a sign.
///
/// **It is C03's inverted scroll sign, in the runtime's own verb**, and it is invisible for exactly
/// the reason §21 gives for the scene existing at all: every caller that draws through a scroll
/// scope draws at offset zero. Filed as `.scratch/vitui-runtime-architecture/issues/26`.
pub const OFFSET: i32 = 0;

// ── the columns ──────────────────────────────────────────────────────────────────────────────────

/// Where a column sits when the table is scrolled sideways.
///
/// **A pinned column may not be elastic**, and it is enforced by construction rather than
/// documented: `Left`/`Right` carry their width and the solver is never consulted for them. §6's
/// reason is two denominators — a pin claims a share of the *table* and a scrolling lane a share of
/// the *content*, and `max(viewport, Σ minima)` is not the table.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pin {
    /// Pinned to the table's left edge, this many cells wide.
    Left(u16),
    /// In the scrolling band.
    None,
    /// Pinned to the table's right edge, this many cells wide.
    Right(u16),
}

/// One declared column.
///
/// The `key` is **stable identity**, independent of position and of visibility: hiding and
/// reordering columns are two of §6's twenty-two grid features and both move positions, so nothing
/// stored may be keyed on one.
#[derive(Clone, Copy, Debug)]
pub struct ColSpec {
    /// Stable identity.
    pub key: u16,
    /// What the column is called. Declared here even though this screen draws no header — see this
    /// module's header for why there is none.
    pub title: &'static str,
    /// Consulted only when `pin` is [`Pin::None`].
    pub width: Constraint,
    /// Which band it lands in.
    pub pin: Pin,
}

impl ColSpec {
    /// A scrolling column.
    const fn new(key: u16, title: &'static str, width: Constraint) -> ColSpec {
        ColSpec {
            key,
            title,
            width,
            pin: Pin::None,
        }
    }

    /// The same column, pinned to the left edge at its fixed width.
    const fn pinned_left(self, w: u16) -> ColSpec {
        ColSpec {
            pin: Pin::Left(w),
            ..self
        }
    }

    /// The same column, pinned to the right edge at its fixed width.
    const fn pinned_right(self, w: u16) -> ColSpec {
        ColSpec {
            pin: Pin::Right(w),
            ..self
        }
    }
}

/// Which of §6's three bands a solved column landed in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Band {
    /// Pinned to the table's left edge.
    Left,
    /// Scrolling.
    Scroll,
    /// Pinned to the table's right edge.
    Right,
}

/// **The twelve columns of §21's scene**, or the sweep's forty, hundred-and-twenty or
/// two-hundred-and-forty.
///
/// Two pinned left, one pinned right, and the rest scrolling — the prototype's own shape. The
/// scrolling columns' widths are **wider than the prototype's** and that is the scene's requirement
/// rather than a preference: §21's row says *horizontal overflow*, and a twelve-column table whose
/// columns sum to less than a 300-cell terminal has none. Σ of the nine scrolling minima is
/// [`CONTENT_W`] against a [`VIEW_W`]-cell viewport, so the band overflows at twelve columns and the
/// scene can be played at all.
///
/// Extra columns for the sweep are appended to the scrolling band **after** the nine, so the window
/// a viewport admits at a given offset does not move — which is what makes *writes and verbs
/// identical across declared column counts* a statement about the mechanism.
///
/// # Panics
///
/// Panics above [`MAX_COLS`], which is the fixed array the solve writes into.
pub fn columns(declared: usize) -> Vec<ColSpec> {
    assert!(declared <= MAX_COLS, "{declared} columns is past the array");
    if declared <= 1 {
        // **The list, expressed as a table.** One column, the full width, no pins: the same
        // rectangle, the same rows, the same cells, and one verb pair a row instead of one a cell.
        // It is the control for §6's headline and what it isolates is the price of the `+`.
        return vec![ColSpec::new(0, "row", Constraint::Weight(1))];
    }
    let mut v = vec![
        ColSpec::new(0, "pid", Constraint::Fixed(8)).pinned_left(8),
        ColSpec::new(1, "name", Constraint::Fixed(26)).pinned_left(26),
        ColSpec::new(2, "path", Constraint::Fixed(96)),
        ColSpec::new(3, "command", Constraint::Fixed(64)),
        ColSpec::new(4, "args", Constraint::Fixed(64)),
        ColSpec::new(5, "user", Constraint::Fixed(16)),
        ColSpec::new(6, "state", Constraint::Fixed(12)),
        ColSpec::new(7, "rss", Constraint::Fixed(12)),
        ColSpec::new(8, "started", Constraint::Fixed(20)),
        ColSpec::new(9, "io", Constraint::Fixed(14)),
        ColSpec::new(10, "env", Constraint::Fixed(96)),
        ColSpec::new(11, "cpu", Constraint::Fixed(7)).pinned_right(7),
    ];
    v.truncate(declared.max(3));
    const EXTRA: [&str; 8] = ["io.r", "io.w", "nice", "prio", "ctx", "flt", "swap", "shr"];
    let mut key = 12u16;
    while v.len() < declared {
        // Before the right pin, which keeps it the last column and keeps the nine that decide the
        // window where they were.
        v.insert(
            v.len() - 1,
            ColSpec::new(key, EXTRA[usize::from(key) % 8], Constraint::Fixed(11)),
        );
        key += 1;
    }
    v
}

/// Cells the two left pins claim. `8 + 26`.
pub const LEFT_W: u16 = 34;
/// Cells the right pin claims.
pub const RIGHT_W: u16 = 7;
/// The scrolling band's viewport. `W - LEFT_W - RIGHT_W`.
pub const VIEW_W: u16 = W - LEFT_W - RIGHT_W;
/// The scrolling band's content width at twelve declared columns: Σ of the nine scrolling widths.
///
/// **The only place in a table where a width is not a viewport** (§6), and what a horizontal
/// scrollbar would be a fraction of.
pub const CONTENT_W: u16 = 96 + 64 + 64 + 16 + 12 + 12 + 20 + 14 + 96;
/// The largest horizontal offset twelve columns admit. `CONTENT_W - VIEW_W`.
pub const MAX_HOFF: i32 = CONTENT_W as i32 - VIEW_W as i32;

/// **The horizontal offset both scenes are played at.** `96`, which is `path`'s right edge.
///
/// A **column boundary**, and that is the scene's own control rather than a convenience. At an
/// offset inside a column the first visible column starts left of the band and the band's clip
/// discards its prefix — which is correct, and which makes both recorders wrong by exactly that
/// prefix (`crate::counters::Tally`'s documented caveat, ADR 0022's clamp-and-discard). At a
/// boundary nothing is discarded on the left, every write on both arms starts inside its own
/// rectangle, and the only asymmetry left between them is the one under test.
pub const HOFF: i32 = 96;

/// **The offset the straddling case is played at**, for the half of the argument the boundary hides.
///
/// `HOFF + 20`. Here the first visible column starts twenty cells left of the band, so the
/// arithmetic arm writes them into the **left** pin — which the pins do not draw over, because they
/// draw first. The picture is wrong, the equality fires, and that is the case §6's sentence is not
/// about: §6's *the screen is correct* holds only where the overrun is on the right.
pub const HOFF_STRADDLE: i32 = HOFF + 20;

/// **Columns of the table the band's last visible column overwrites, at [`HOFF`].**
///
/// `env` spans content `298..394` and the viewport ends at `HOFF + VIEW_W = 355`, so it reaches
/// **39** columns past the band. Seven of those thirty-nine land on the table and the rest fall off
/// the terminal, which the engine discards; the seven are the right pin's, all of them — **the pin
/// is overwritten whole**, and [`RIGHT_W`] rather than the reach is what the re-damage is bounded
/// by. A wider pin would take a proportional share and a narrower table none at all.
pub const BAND_OVERRUN: u16 = RIGHT_W;

/// **Cells the arithmetic band re-damages on every steady frame, forever.** `BAND_OVERRUN x H`.
///
/// §6 states **345** on the prototype's screen. The number here is different and it is the same
/// quantity: it is arithmetic over this screen's geometry rather than a measurement of it, which is
/// what makes it checkable — see [`BAND_OVERRUN`].
pub const RE_DAMAGED: u64 = BAND_OVERRUN as u64 * H as u64;

/// **Columns the row loop draws at [`HOFF`] and twelve declared columns.** Two pins, eight of the
/// nine scrolling columns, one pin.
///
/// The ninth is `path`, whose right edge is exactly [`HOFF`] — the boundary the offset was chosen to
/// sit on.
pub const COLUMNS_DRAWN: usize = 11;

/// **Columns of [`COLUMNS_DRAWN`] whose own text fills them exactly**, and which therefore cost
/// **one** verb rather than two: `pid` at eight columns and `cpu` at seven.
///
/// §6 says *two verbs a cell is the floor and there is no cheaper correct shape*, and the floor is
/// exact for a cell with any padding in it. A cell narrower than its own content has none —
/// [`Ink::run`] is handed a zero-length run and refuses to issue one — so the honest form of the
/// sentence is **at most two a cell, two whenever anything is padded**. It is a refinement of §6's
/// figure rather than a disagreement with it: no cell here costs three.
pub const FULL_COLUMNS: usize = 2;

/// **Drawing verbs the twelve-column table costs.** `2 x H x COLUMNS_DRAWN - H x FULL_COLUMNS`.
///
/// §6's figure is **2 497** on its own screen — two tables and three bars on one terminal. What
/// reproduces is the shape and not the magnitude; see [`VERBS_ONE`].
pub const VERBS_TWELVE: u64 = 2 * H as u64 * COLUMNS_DRAWN as u64 - H as u64 * FULL_COLUMNS as u64;

/// **Drawing verbs the same rectangle costs drawn as one column** — a list, expressed as a table.
///
/// §6's figure is **913**, and the ratio it reports is 2.7x. This screen's is 10x, because §6's
/// one-column arm shared its terminal with a four-column table and three bars and this one does not.
pub const VERBS_ONE: u64 = 2 * H as u64;

/// **Columns the caller asked for over one frame**, before the clip.
///
/// [`CELLS`] plus [`ASKED_DISCARDED`]. The same on **both** arms of the band, which is why `asked`
/// cannot separate them and `reported` can.
pub const ASKED: u64 = CELLS + ASKED_DISCARDED;

/// **Columns the clip discarded.** The last visible column reaches 39 past the band on every one of
/// [`H`] rows.
pub const ASKED_DISCARDED: u64 = 39 * H as u64;

/// Columns the first visible column reaches into the **left** pin at [`HOFF_STRADDLE`].
///
/// `HOFF_STRADDLE - HOFF`, and it is the straddle itself: a column whose content x is `HOFF` drawn
/// at an offset `d` past it lands `d` cells left of the band.
pub const STRADDLE_OVERRUN: u16 = (HOFF_STRADDLE - HOFF) as u16;

/// **Cells the arithmetic band re-damages a frame at [`HOFF_STRADDLE`].**
///
/// `(STRADDLE_OVERRUN + BAND_OVERRUN) x H` — both edges at once, where [`HOFF`] has only the right
/// one. The left is the expensive one and not because it is wider: the pins draw **first**, so the
/// cells it takes are never drawn again and the picture is wrong as well as the frame.
pub const STRADDLE_RE_DAMAGED: u64 = (STRADDLE_OVERRUN + BAND_OVERRUN) as u64 * H as u64;

/// **Cells the equality reports different for the arithmetic band at [`HOFF_STRADDLE`].**
///
/// Fewer than `STRADDLE_OVERRUN x H = 1 600`, and the shortfall is the fixture's rather than the
/// defect's: the column that should be there and the column that is there are both a key and a row
/// index, so they agree wherever their digits do — 88 cells of 1 600, and the very first cell of the
/// screen is one of them. Measured and written down rather than tuned away, which is
/// [`crate::listing::SCROLLED_ROWS`]'s arrangement one file over.
pub const STRADDLE_CELLS: usize = 712;

/// **Cells the equality reports different for the *correct* arm at [`HOFF_STRADDLE`], which is a
/// number about the recorder and not about the painter.**
///
/// `10 x H`. At an offset inside a column the band's clip discards a **prefix**, and ADR 0022's
/// clamp-and-discard means [`crate::runner::Pen`] — like [`Tally`] — is then wrong by exactly that
/// prefix: it advances from the `x` the verb was given by the columns the engine says landed, and
/// those are not the same columns. Both instruments state the caveat; this is it as a number.
///
/// **It is the whole reason [`HOFF`] is a column boundary.** A scene played only at a straddling
/// offset would be reading its own recorder's error as a defect in the table.
pub const STRADDLE_RECORDER_CELLS: usize = 800;

/// **Cells the equality reports different for the inverted horizontal sign at [`HOFF`].**
pub const INVERTED_CELLS: usize = 17_022;

/// **Cells the inverted sign writes**, against [`CELLS`] for every correct arm.
///
/// §3.3 measured the same defect as **4 271 writes fewer** and forty microseconds faster with the
/// same verbs issued. Here it is 15 360 fewer, and the verbs are identical to the verb.
pub const INVERTED_WRITES: u64 = 8_640;

// ── the solve ────────────────────────────────────────────────────────────────────────────────────

/// What one frame's solve produced. Plain arrays: no allocation, no per-frame scratch.
#[derive(Clone)]
pub struct Solved {
    /// How many columns were placed.
    pub n: usize,
    /// Index into the caller's `&[ColSpec]`, so `key` and `title` are one hop away.
    pub spec: [u16; MAX_COLS],
    /// Which band each landed in.
    pub band: [Band; MAX_COLS],
    /// x **in the coordinates of the column's own band**.
    pub x: [i32; MAX_COLS],
    /// Its width.
    pub w: [u16; MAX_COLS],
    /// Cells the left pins claimed.
    pub left_w: u16,
    /// Cells the right pins claimed.
    pub right_w: u16,
    /// The scrolling band's viewport.
    pub view_w: u16,
    /// The scrolling band's content width, `max(view_w, Σ minima)`.
    pub content_w: u16,
    /// `[lo, hi)` into the arrays for the left band.
    pub left: (usize, usize),
    /// `[lo, hi)` for the scrolling band.
    pub scroll: (usize, usize),
    /// `[lo, hi)` for the right band.
    pub right: (usize, usize),
}

impl Default for Solved {
    fn default() -> Solved {
        Solved {
            n: 0,
            spec: [0; MAX_COLS],
            band: [Band::Scroll; MAX_COLS],
            x: [0; MAX_COLS],
            w: [0; MAX_COLS],
            left_w: 0,
            right_w: 0,
            view_w: 0,
            content_w: 0,
            left: (0, 0),
            scroll: (0, 0),
            right: (0, 0),
        }
    }
}

impl Solved {
    /// The largest horizontal offset the scrolling band admits.
    pub fn max_hoff(&self) -> i32 {
        i32::from(self.content_w) - i32::from(self.view_w)
    }
}

/// The floor a constraint shrinks to when the band overflows its viewport.
const fn floor(c: Constraint) -> u16 {
    match c {
        Constraint::Fixed(v) | Constraint::Min(v) | Constraint::Max(v) => v,
        Constraint::Percent(_) | Constraint::Ratio(_, _) | Constraint::Weight(_) => 6,
    }
}

/// **The whole of a table's layout.** Runs once a frame, touches no row, allocates nothing.
///
/// The three bands are solved separately and that is §6's decision rather than an implementation
/// detail: a pin is a *band*, so it is a rect split, and the scrolling columns then have exactly one
/// denominator. The band handed to the solver is `content_w` and not `view_w` — when the columns fit
/// the two are the same and the elastic ones share the viewport; when they do not, the band is
/// Σ minima and the difference is what a horizontal offset scrolls over.
pub fn solve_columns(area_w: u16, specs: &[ColSpec]) -> Solved {
    let mut out = Solved::default();
    let mut left_w: u32 = 0;
    let mut right_w: u32 = 0;
    for s in specs {
        match s.pin {
            Pin::Left(w) => left_w += u32::from(w),
            Pin::Right(w) => right_w += u32::from(w),
            Pin::None => {}
        }
    }
    let total = u32::from(area_w);
    let left_w = left_w.min(total);
    let right_w = right_w.min(total - left_w);
    out.left_w = left_w as u16;
    out.right_w = right_w as u16;
    out.view_w = (total - left_w - right_w) as u16;

    let mut n = 0usize;
    let mut x = 0i32;
    out.left.0 = n;
    for (i, s) in specs.iter().enumerate() {
        if let Pin::Left(w) = s.pin {
            out.spec[n] = i as u16;
            out.band[n] = Band::Left;
            out.x[n] = x;
            out.w[n] = w;
            x += i32::from(w);
            n += 1;
        }
    }
    out.left.1 = n;

    let mut spec_buf = [Constraint::Fixed(0); MAX_COLS];
    let mut idx_buf = [0u16; MAX_COLS];
    let mut m = 0usize;
    let mut min_sum: u32 = 0;
    for (i, s) in specs.iter().enumerate() {
        if s.pin != Pin::None || m == MAX_COLS {
            continue;
        }
        spec_buf[m] = s.width;
        idx_buf[m] = i as u16;
        min_sum += u32::from(floor(s.width));
        m += 1;
    }
    let content_w = min_sum.max(u32::from(out.view_w)).min(u32::from(u16::MAX));
    out.content_w = content_w as u16;

    let mut rects = [Rect::default(); MAX_COLS];
    let (k, _) = solve(content_w, &spec_buf[..m], &mut rects[..m]);
    out.scroll.0 = n;
    let mut cx = 0i32;
    for (j, rect) in rects[..k].iter().enumerate() {
        out.spec[n] = idx_buf[j];
        out.band[n] = Band::Scroll;
        out.x[n] = cx;
        out.w[n] = rect.w;
        cx += i32::from(rect.w);
        n += 1;
    }
    out.scroll.1 = n;

    out.right.0 = n;
    let mut rx = 0i32;
    for (i, s) in specs.iter().enumerate() {
        if let Pin::Right(w) = s.pin {
            out.spec[n] = i as u16;
            out.band[n] = Band::Right;
            out.x[n] = rx;
            out.w[n] = w;
            rx += i32::from(w);
            n += 1;
        }
    }
    out.right.1 = n;
    out.n = n;
    out
}

/// **The column half of the invariant**: `[lo, hi)` into the scrolling band, the columns a viewport
/// `view_w` wide at horizontal offset `hoff` can show and nothing else.
///
/// Binary search rather than a walk, because the walk stays correct while the declared column count
/// grows and the cost grows with it. At forty columns the two are indistinguishable; the point of
/// writing the search is that at two hundred and forty they are not.
///
/// **This is the fast path and [`Ctx::visible_cols`](vitui_runtime::Ctx::visible_cols) is its
/// oracle**, not the other way round: [`BandShape::Arithmetic`] opens no view, so there is nothing
/// for it to ask. `tests::the_column_window_agrees_with_the_runtimes_own_visible_cols` sweeps every
/// offset the content admits and asserts the two name the same columns.
pub fn visible(s: &Solved, hoff: i32) -> (usize, usize) {
    let (lo0, hi0) = s.scroll;
    if hi0 <= lo0 || s.view_w == 0 {
        return (lo0, lo0);
    }
    let left = hoff;
    let right = hoff + i32::from(s.view_w);
    let lo = partition(lo0, hi0, |i| s.x[i] + i32::from(s.w[i]) <= left);
    let hi = partition(lo, hi0, |i| s.x[i] < right);
    (lo, hi)
}

/// The first index in `[lo, hi)` for which `pred` is false. `pred` must be monotone.
fn partition(lo: usize, hi: usize, pred: impl Fn(usize) -> bool) -> usize {
    let (mut a, mut b) = (lo, hi);
    while a < b {
        let mid = a + (b - a) / 2;
        if pred(mid) { a = mid + 1 } else { b = mid }
    }
    a
}

// ── the cells ────────────────────────────────────────────────────────────────────────────────────

/// A stack buffer a cell's text is formatted into.
///
/// **The frame allocates nothing** (§20), and `format!` allocates. `Ctx::stage`/`Ctx::blit` is the
/// runtime's own answer to the same problem and it is not reachable through [`Ink`], which takes a
/// `&str` — so the fixture keeps its own, sixty-four bytes on the stack, and
/// `tests::a_frame_of_the_grid_allocates_nothing` is what says it worked.
struct Buf {
    bytes: [u8; 64],
    len: usize,
}

impl Default for Buf {
    fn default() -> Buf {
        Buf {
            bytes: [0; 64],
            len: 0,
        }
    }
}

impl fmt::Write for Buf {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let end = self.len + s.len();
        if end > self.bytes.len() {
            return Err(fmt::Error);
        }
        self.bytes[self.len..end].copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

impl Buf {
    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}

/// **What one cell says**, written into `buf` and returned as a `&str`.
///
/// `kk.rrrrrrr` — the column's key and the row's index. Every cell of the screen is distinct and no
/// two cells of one row share a prefix, which is [`crate::runner::Fixture::lines`]'s property and
/// its reason: a wrong cell costs exactly its own columns, so *n cells over m rows* means what it
/// says. A fixture whose cells shared a prefix would report a **smaller** count for the same wrong
/// column.
fn cell_text(buf: &mut Buf, key: u16, row: u64) -> &str {
    use fmt::Write as _;
    buf.len = 0;
    let _ = write!(buf, "{key:02}.{row:07}");
    buf.as_str()
}

// ── what is drawn, and the three ways to get it wrong ────────────────────────────────────────────

/// **How the scrolling band reaches the surface**, which is the whole of scene 30's first arm.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BandShape {
    /// **The rule.** The band is a view: a column at the viewport's edge is clipped, and the cells
    /// past the edge belong to the pinned band.
    View,
    /// **The defect.** One pass, no view, the offset applied as arithmetic. Renders almost right —
    /// the pinned band draws afterwards and wins — and re-damages [`RE_DAMAGED`] cells a frame for
    /// ever.
    Arithmetic,
}

impl BandShape {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            BandShape::View => "a view per row",
            BandShape::Arithmetic => "the offset as arithmetic",
        }
    }
}

/// **Whether the column window is asked for or left to the clip.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ColVirt {
    /// **The rule.** Only the columns the viewport admits are drawn.
    Virtualised,
    /// **The defect.** Every declared column is drawn and the clip discards what does not fit.
    ClipOnly,
}

impl ColVirt {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            ColVirt::Virtualised => "virtualised",
            ColVirt::ClipOnly => "clip-only",
        }
    }
}

/// **The sign of the horizontal offset**, which is C03's trap on the other axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HSign {
    /// **The rule.** The content moves the other way from the offset.
    Plus,
    /// **The defect.** The offset added where it should be subtracted, which is C03's `scrolled(0,
    /// -offset)` on the other axis: the band shows a partly blank stretch, issues every verb it
    /// would have issued, and measures as an improvement because the clip eats the writes.
    Minus,
}

impl HSign {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            HSign::Plus => "+hoff",
            HSign::Minus => "-hoff",
        }
    }
}

/// One arm of the scene: three independent choices, each of which is a way for
/// `table = collection + column rectangles` to be false.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Opts {
    /// How the scrolling band reaches the surface.
    pub band: BandShape,
    /// Whether the column window is asked for.
    pub cols: ColVirt,
    /// The sign of the horizontal offset.
    pub hsign: HSign,
}

impl Default for Opts {
    /// Every choice made the way §6 makes it.
    fn default() -> Opts {
        Opts {
            band: BandShape::View,
            cols: ColVirt::Virtualised,
            hsign: HSign::Plus,
        }
    }
}

impl Opts {
    /// The correct arm, spelled out where a reader would otherwise have to trust `Default`.
    pub fn correct() -> Opts {
        Opts::default()
    }

    /// The arm §6 refuses: the offset as arithmetic, no view.
    pub fn arithmetic_band() -> Opts {
        Opts {
            band: BandShape::Arithmetic,
            ..Opts::default()
        }
    }

    /// The arm §3.3 refuses: the horizontal sign inverted.
    pub fn inverted_sign() -> Opts {
        Opts {
            hsign: HSign::Minus,
            ..Opts::default()
        }
    }

    /// The arm §6 prices at 3.3x the frame and 7.2x the verbs: every declared column drawn.
    pub fn clip_only() -> Opts {
        Opts {
            cols: ColVirt::ClipOnly,
            ..Opts::default()
        }
    }

    /// A one-line description, for a report's row label.
    pub fn word(self) -> String {
        format!(
            "{}, {}, {}",
            self.band.word(),
            self.cols.word(),
            self.hsign.word()
        )
    }
}

/// **Draw one frame of the grid**, and return how many rows the body iterated.
///
/// Generic over [`Ink`] so that a [`Tally`], a [`Pen`] and a [`Direct`] measure **the same drawing
/// path** rather than a copy of it — [`crate::ink`]'s whole argument.
///
/// The rectangle is the whole context. One hit entry, one tab stop, and the cell under the pointer
/// is arithmetic on both axes (ADR 0028 on the row axis, §6's cumulative array on the column axis) —
/// which is why there is no per-cell target and no per-row one.
///
/// [`Direct`]: crate::ink::Direct
pub fn draw_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    specs: &[ColSpec],
    o: Opts,
    rows: u64,
    offset: i32,
    hoff: i32,
) -> u64 {
    let id = Id::named("grid");
    let view = cx.area();
    let s = solve_columns(view.w, specs);
    let max = (s.max_hoff(), max_offset(rows));
    let at = (hoff.clamp(0, max.0), offset.clamp(0, max.1));
    let body = cx.theme().paint(Role::Body);

    // **One hit entry for the table**, and it is the wheel chain's entry as well.
    let _ = cx.scrollable(
        id,
        view,
        Interest::CLICK.with(Interest::FOCUS),
        Scrollable::between(at, max),
    );

    let (vlo, vhi) = match o.cols {
        ColVirt::Virtualised => visible(&s, at.0),
        ColVirt::ClipOnly => s.scroll,
    };
    let signed = match o.hsign {
        HSign::Plus => at.0,
        HSign::Minus => -at.0,
    };
    let left_w = i32::from(s.left_w);
    let band_x = left_w;
    let mut buf = Buf::default();
    let mut iterated = 0u64;

    // The vertical axis is the runtime's, and `visible_rows` is the row half of the invariant.
    // **Nothing is drawn outside this scope**, which is what keeps the whole screen in one
    // coordinate space — see this module's header.
    cx.scroll_scope(id, view, (0, at.1), (0, max.1), |cx| {
        let window = cx.visible_rows();
        let lo = window.start.max(0);
        let hi = window.end.min(i32::try_from(rows).unwrap_or(i32::MAX));
        for i in lo..hi {
            iterated += 1;
            let row = i as u64;
            cx.with_key(row, |cx| {
                for slot in s.left.0..s.left.1 {
                    let key = specs[usize::from(s.spec[slot])].key;
                    let text = cell_text(&mut buf, key, row);
                    emit(ink, cx, s.x[slot], i, s.w[slot], text, body);
                }
                match o.band {
                    // **The clip, and the translation immediately undone.** `Ctx::child` narrows
                    // and moves the origin; the band needs the first and not the second, and
                    // undoing it is what leaves the two arms writing identical arguments at
                    // identical call sites with one `child` between them.
                    BandShape::View => {
                        let rect = Rect::new(band_x, i, s.view_w, 1);
                        let mut clipped = cx.child(rect);
                        let mut band = clipped.scrolled(-rect.x, -rect.y);
                        for slot in vlo..vhi {
                            let key = specs[usize::from(s.spec[slot])].key;
                            let text = cell_text(&mut buf, key, row);
                            let x = band_x + s.x[slot] - signed;
                            emit(ink, &mut band, x, i, s.w[slot], text, body);
                        }
                    }
                    BandShape::Arithmetic => {
                        for slot in vlo..vhi {
                            let key = specs[usize::from(s.spec[slot])].key;
                            let text = cell_text(&mut buf, key, row);
                            let x = band_x + s.x[slot] - signed;
                            emit(ink, cx, x, i, s.w[slot], text, body);
                        }
                    }
                }
                let right_x = band_x + i32::from(s.view_w);
                for slot in s.right.0..s.right.1 {
                    let key = specs[usize::from(s.spec[slot])].key;
                    let text = cell_text(&mut buf, key, row);
                    emit(ink, cx, right_x + s.x[slot], i, s.w[slot], text, body);
                }
            });
        }
    });
    iterated
}

/// **One cell: the text, then the padding after it.** §6's floor, and there is no cheaper correct
/// shape.
///
/// The pad is placed from the text's **own** width and not from what the engine reported, because a
/// component partitions its rectangle in its own coordinates and the clip is entitled to discard
/// whatever leaves it (ADR 0022). Advancing by the reported count instead would make a clipped cell
/// pad the wrong columns, which is a partition defect the clip would then hide.
fn emit<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    x: i32,
    y: i32,
    w: u16,
    text: &str,
    paint: vitui_runtime::Paint,
) {
    let cut = truncate(text, w);
    let used = width(cut);
    let _ = ink.text(cx, x, y, cut, paint);
    let _ = ink.run(
        cx,
        x + i32::from(used),
        y,
        " ",
        w.saturating_sub(used),
        paint,
    );
}

/// The largest vertical offset `rows` rows admit in an [`H`]-row viewport.
pub fn max_offset(rows: u64) -> i32 {
    i32::try_from(rows.saturating_sub(u64::from(H))).unwrap_or(i32::MAX)
}

// ── what a frame turned out to be ────────────────────────────────────────────────────────────────

/// **What one frame of the grid turned out to be**, returned rather than printed.
///
/// A number only a report prints is a number no gate can read — [`crate::dense::Shape`]'s rule.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Shape {
    /// Columns the engine reported written.
    pub writes: u64,
    /// Distinct cells touched. **The counter this scene is decided by.**
    pub distinct: u64,
    /// Columns the caller asked for, before clipping.
    pub asked: u64,
    /// Drawing calls made.
    pub verbs: u64,
    /// Entries in the runtime's hit index.
    pub regions: usize,
    /// Entries in the focus ring.
    pub stops: usize,
    /// Rows the body iterated.
    pub iterated: u64,
    /// Columns the row loop drew, per row.
    pub columns: usize,
}

impl Shape {
    /// **Cells written more than once in one frame.** `writes - distinct`.
    ///
    /// C01's rule as a number: *no cell may be written twice with different values in one frame*.
    /// Every cell of the overrun is written exactly twice — once by the band's edge column and once
    /// by the pin that follows it — so the excess is the count of re-damaged cells rather than a
    /// count of writes.
    pub fn re_damaged(&self) -> u64 {
        self.writes.saturating_sub(self.distinct)
    }
}

/// **One frame of the grid at `declared` columns, as a shape.**
///
/// One warm-up frame and one measured one: the five frame structures take their allocation on the
/// first frame that needs one and keep it, so a cold frame is not a frame.
pub fn frame(o: Opts, declared: usize, rows: u64, hoff: i32) -> Shape {
    frame_over(o, declared, rows, hoff, 1).0
}

/// [`frame`], over `frames` measured frames, with the **per-frame** time beside it.
///
/// **The duration is a report and never a gate.** R15's rule, inherited unchanged, and the reason is
/// on this page: §6's refused arm is *identical time*.
///
/// # Panics
///
/// Panics on zero frames. A per-frame figure over no frames is a division by zero dressed as a
/// measurement.
pub fn frame_over(
    o: Opts,
    declared: usize,
    rows: u64,
    hoff: i32,
    frames: u32,
) -> (Shape, Duration) {
    assert!(frames > 0, "a per-frame figure needs a frame");
    let specs = columns(declared);
    let solved = solve_columns(W, &specs);
    let (vlo, vhi) = match o.cols {
        ColVirt::Virtualised => visible(&solved, hoff.clamp(0, solved.max_hoff())),
        ColVirt::ClipOnly => solved.scroll,
    };
    let drawn = (solved.left.1 - solved.left.0) + (vhi - vlo) + (solved.right.1 - solved.right.0);

    let mut driver = crate::runner::driver_at(W, H, Density::default());
    let mut tally = Tally::new();
    let mut iterated = 0;
    driver.frame(|cx| iterated = draw_into(&mut tally, cx, &specs, o, rows, OFFSET, hoff));

    let mut elapsed = Duration::ZERO;
    let mut shape = Shape::default();
    for _ in 0..frames {
        let mut t = Tally::new();
        let started = Instant::now();
        driver.frame(|cx| iterated = draw_into(&mut t, cx, &specs, o, rows, OFFSET, hoff));
        elapsed += started.elapsed();
        let f = driver.inspect();
        shape = Shape {
            writes: t.writes(),
            distinct: t.distinct(),
            asked: t.asked(),
            verbs: t.verbs(),
            regions: f.hits().len(),
            stops: f.stop_count(),
            iterated,
            columns: drawn,
        };
    }
    (shape, elapsed / frames)
}

/// **What the frame costs a component**, drawn through [`Direct`] rather than through a [`Tally`].
///
/// The one number here comparable with §6's own 165.6 µs, and separate from [`frame_over`]'s for a
/// reason worth the second function: a `Tally` keeps a `BTreeSet` of every cell it sees, so a figure
/// taken with one in the loop is a report about the instrument.
///
/// A report and never a gate.
///
/// # Panics
///
/// [`frame_over`]'s, unchanged.
///
/// [`Direct`]: crate::ink::Direct
pub fn frame_cost(o: Opts, declared: usize, rows: u64, hoff: i32, frames: u32) -> Duration {
    assert!(frames > 0, "a per-frame figure needs a frame");
    let specs = columns(declared);
    let mut driver = crate::runner::driver_at(W, H, Density::default());
    let mut ink = Direct;
    driver.frame(|cx| {
        let _ = draw_into(&mut ink, cx, &specs, o, rows, OFFSET, hoff);
    });
    let started = Instant::now();
    for _ in 0..frames {
        driver.frame(|cx| {
            let _ = draw_into(&mut ink, cx, &specs, o, rows, OFFSET, hoff);
        });
    }
    started.elapsed() / frames
}

/// **The shape at every one of [`DECLARED`]**, which is what §6's column-axis equality is asserted
/// over.
pub fn across_declared(o: Opts) -> Vec<(usize, Shape)> {
    DECLARED
        .into_iter()
        .map(|d| (d, frame(o, d, VOLUMES[2], HOFF)))
        .collect()
}

/// **The shape at every one of [`VOLUMES`]**, which is the row axis of the same invariant.
pub fn across_volumes(o: Opts) -> Vec<(u64, Shape)> {
    VOLUMES
        .into_iter()
        .map(|n| (n, frame(o, 12, n, HOFF)))
        .collect()
}

/// **Every counter this crate can read, over two arms of one screen.** Returns `(a, b)`.
///
/// The allocation totals are the caller's, for [`crate::runner::Run::counters`]'s reason: a library
/// cannot install a global allocator on a consumer's behalf, and *a figure defaulted to zero is a
/// counter that prints `0` when it means nobody counted*.
pub fn counters_of(
    a: Opts,
    b: Opts,
    allocs_a: Allocations,
    allocs_b: Allocations,
) -> (Counters, Counters) {
    (counters_one(a, allocs_a), counters_one(b, allocs_b))
}

fn counters_one(o: Opts, allocs: Allocations) -> Counters {
    let specs = columns(12);
    let mut driver = crate::runner::driver_at(W, H, Density::default());
    let mut tally = Tally::new();
    driver.frame(|cx| {
        let _ = draw_into(&mut tally, cx, &specs, o, VOLUMES[2], OFFSET, HOFF);
    });
    let mut t = Tally::new();
    driver.frame(|cx| {
        let _ = draw_into(&mut t, cx, &specs, o, VOLUMES[2], OFFSET, HOFF);
    });
    Counters::of(&driver, &t, allocs)
}

/// **Which of §20's nine counters tell two arms apart.**
///
/// An empty answer means the equality against a reference render is the only detector there is; a
/// non-empty one names a cheaper gate. A counter that is
/// [`Reading::Unreachable`](crate::counters::Reading::Unreachable) on both arms — `marked`, and it
/// will stay that way — contributes nothing either way and is skipped rather than counted as
/// agreement.
pub fn counters_that_separate(
    a: Opts,
    b: Opts,
    allocs_a: Allocations,
    allocs_b: Allocations,
) -> Vec<Counter> {
    let (x, y) = counters_of(a, b, allocs_a, allocs_b);
    Counter::ALL
        .into_iter()
        .filter(|c| {
            let (left, right) = (x.get(*c).measured(), y.get(*c).measured());
            left.is_some() != right.is_some() || (left.is_some() && left != right)
        })
        .collect()
}

// ── the equality: the same table under a horizontal offset, against itself ───────────────────────

/// **The reference render's content, computed the other way round.**
///
/// The fast path maps *column to screen x*; this maps *screen x to column*, cell by cell, over every
/// cell of the rectangle. That is the only thing that makes it an oracle rather than a copy — the
/// engine's own obviously-correct-and-far-too-slow compositor is unreachable from this crate for the
/// four independent reasons in [`crate::runner`]'s header, so a components-side render is the rule.
///
/// Truncation goes through the same [`truncate`] both arms use, which is the engine's own width
/// table reached through the runtime's re-export: a disagreement between the two is never about
/// UAX #29.
pub fn oracle_rows(declared: usize, hoff: i32) -> Vec<String> {
    let specs = columns(declared);
    let s = solve_columns(W, &specs);
    let hoff = hoff.clamp(0, s.max_hoff());
    let left_w = i32::from(s.left_w);
    let right_x = left_w + i32::from(s.view_w);
    let mut buf = Buf::default();
    let mut rows = Vec::with_capacity(usize::from(H));
    for y in 0..H {
        let row = u64::from(y);
        let mut line = String::with_capacity(usize::from(W));
        for x in 0..i32::from(W) {
            let (band, bx) = if x < left_w {
                (s.left, x)
            } else if x < right_x {
                (s.scroll, x - left_w + hoff)
            } else {
                (s.right, x - right_x)
            };
            line.push(char_at(&s, &specs, band, bx, row, &mut buf));
        }
        rows.push(line);
    }
    rows
}

/// The character belonging at band-local `bx` of `row`, or a space where no column reaches.
fn char_at(
    s: &Solved,
    specs: &[ColSpec],
    band: (usize, usize),
    bx: i32,
    row: u64,
    buf: &mut Buf,
) -> char {
    let (lo, hi) = band;
    if hi <= lo || bx < 0 {
        return ' ';
    }
    let i = partition(lo, hi, |i| s.x[i] + i32::from(s.w[i]) <= bx);
    if i >= hi || bx < s.x[i] {
        return ' ';
    }
    let key = specs[usize::from(s.spec[i])].key;
    let text = cell_text(buf, key, row);
    let cut = truncate(text, s.w[i]);
    // ASCII by construction, so a byte index is a column index and the `nth` is not a walk over
    // clusters. A fixture with a wide cluster in it is a fixture about UAX #29, which is
    // `crate::runner`'s question and not this one.
    cut.chars().nth((bx - s.x[i]) as usize).unwrap_or(' ')
}

/// The oracle as a [`Fixture`], for [`crate::runner::compare`].
pub fn oracle(declared: usize, hoff: i32) -> Fixture {
    Fixture::of(W, H, oracle_rows(declared, hoff))
}

/// **The oracle drawn one cell at a time**, which is what the equality's first arm is.
///
/// [`crate::runner::reference`] is the same program and it is **250x too slow at this size**: it
/// asks the fixture for every cell independently and a `Fixture::cell` segments the whole row to
/// answer, which is 300 cluster walks a cell over 24 000 cells. That is the right shape for the
/// 40x80 screen it was written for and not for a 300-cell-wide one, so this asks the fixture for a
/// **row** and indexes it — ASCII by construction, so a byte is a column.
///
/// It is still one [`Ctx::set`](vitui_runtime::Ctx::set) a cell, no runs and no offset it did not
/// compute itself. `tests::the_two_per_cell_oracles_are_the_same_program` compares it against
/// components ticket 04's over a fixture small enough to run that one, which is where the claim
/// *the same program* is checked rather than asserted.
pub fn per_cell(pen: &mut Pen, cx: &mut Ctx<'_, '_>, fx: &Fixture) {
    let body = cx.theme().paint(Role::Body);
    let (w, h) = fx.size();
    for y in 0..h {
        let line = fx.row_text(y);
        let bytes = line.as_bytes();
        for x in 0..w {
            let byte = [bytes.get(usize::from(x)).copied().unwrap_or(b' ')];
            let cluster = std::str::from_utf8(&byte).unwrap_or(" ");
            pen.set(cx, i32::from(x), i32::from(y), cluster, body);
        }
    }
}

/// **The correct table, drawn through a [`Pen`] at [`HOFF`].** The equality's first arm.
///
/// The vertical offset is zero here and [`OFFSET`] in the counted frame, and the difference is the
/// recorder rather than the scene: a [`crate::runner::Canvas`] is [`H`] rows tall and a scrolled
/// context is a content coordinate system, so a frame parked a thousand rows down writes at
/// `y = 1 000` and the canvas discards it. `crate::listing` states the same fact one file over.
pub fn banded(pen: &mut Pen, cx: &mut Ctx<'_, '_>, _fx: &Fixture) {
    let specs = columns(12);
    let _ = draw_into(pen, cx, &specs, Opts::correct(), VOLUMES[2], 0, HOFF);
}

/// **The band written by arithmetic instead of into a view**, at [`HOFF`]. §6's refused arm.
pub fn arithmetic(pen: &mut Pen, cx: &mut Ctx<'_, '_>, _fx: &Fixture) {
    let specs = columns(12);
    let _ = draw_into(
        pen,
        cx,
        &specs,
        Opts::arithmetic_band(),
        VOLUMES[2],
        0,
        HOFF,
    );
}

/// **The horizontal sign inverted**, at [`HOFF`]. §3.3's refused arm, and C03's trap on the other
/// axis.
pub fn inverted(pen: &mut Pen, cx: &mut Ctx<'_, '_>, _fx: &Fixture) {
    let specs = columns(12);
    let _ = draw_into(pen, cx, &specs, Opts::inverted_sign(), VOLUMES[2], 0, HOFF);
}

/// **The band written by arithmetic at an offset inside a column**, where the overrun is on the
/// left.
///
/// The same defect and the other half of the picture: at [`HOFF_STRADDLE`] the first visible column
/// starts left of the band and the arithmetic arm writes into the **left** pin, which draws first
/// and does not draw again. §6's *the screen is correct* is a statement about the right edge only.
pub fn arithmetic_straddling(pen: &mut Pen, cx: &mut Ctx<'_, '_>, _fx: &Fixture) {
    let specs = columns(12);
    let _ = draw_into(
        pen,
        cx,
        &specs,
        Opts::arithmetic_band(),
        VOLUMES[2],
        0,
        HOFF_STRADDLE,
    );
}

/// **The correct table at [`HOFF_STRADDLE`]**, so the straddling comparison has both arms.
pub fn banded_straddling(pen: &mut Pen, cx: &mut Ctx<'_, '_>, _fx: &Fixture) {
    let specs = columns(12);
    let _ = draw_into(
        pen,
        cx,
        &specs,
        Opts::correct(),
        VOLUMES[2],
        0,
        HOFF_STRADDLE,
    );
}

/// **The table against a reference render of itself, under a horizontal offset.** Register row 12's
/// shape, and scene 30.
///
/// `compare(per_cell, arm, …)` — one `Ctx::set` a cell over the oracle's own rows, against the
/// table drawn the way the arm draws it.
pub fn equality(arm: crate::runner::Painter, hoff: i32) -> Diff {
    compare(per_cell, arm, &[oracle(12, hoff)])
}

// ── the subject, and the scan that says whether it is here ───────────────────────────────────────

/// **The component these two scenes are scenes of, and it is not declared yet.**
///
/// One subject and not two, which is [`Verdict::of`]'s vacuity refusal doing its job on a population
/// of one: *no component exists* is `Unmet` over one rather than `Met` over nothing.
pub const SUBJECTS: [&str; 1] = ["table"];

/// Where [`SUBJECTS`] is declared, as `(module file, the declaration)`.
///
/// The home is the freeze's, joined through [`crate::Family`]: `table`'s first family is
/// `F7Collections`, whose module is `collect.rs`.
pub const DECLARATIONS: [(&str, &str); 1] = [("collect.rs", "pub fn table(")];

/// **Which of [`SUBJECTS`] this crate actually declares. Today: none.**
///
/// A source scan and not a `use`, for [`crate::dense::subjects_declared`]'s reason: *the item does
/// not exist* has no expression, and a `compile_fail` fence would pass today and pass again the day
/// somebody renames the module.
pub fn subjects_declared() -> Vec<&'static str> {
    let mut out = Vec::new();
    for (subject, (file, declaration)) in SUBJECTS.into_iter().zip(DECLARATIONS) {
        let path = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).join(file);
        let source = std::fs::read_to_string(&path).unwrap_or_default();
        if crate::dense::declares(&source, declaration) {
            out.push(subject);
        }
    }
    out
}

/// **Whether the grid stands on its subject, as a verdict rather than as a sentence.**
///
/// `Unmet` over one, inverted by **components 15**.
pub fn standing() -> Verdict {
    let declared = subjects_declared();
    Verdict::of(
        SUBJECTS.len(),
        SUBJECTS.len() - declared.len(),
        "`table` is not declared in this crate, so what stands on the grid is a stand-in row loop \
         and not the component. The screen, its one region and one tab stop, its equality against a \
         reference render at two horizontal offsets, its identical writes and verbs at 12, 40, 120 \
         and 240 declared columns and at 1k / 100k / 1M rows, and the arithmetic band's re-damaged \
         cells are measured and green; what is missing is the subject",
        "components 15",
    )
}

/// **The sentence a scene waiting for its subject fails with**, or `None` once it is declared.
///
/// # This is criterion 7, and it is the distinction the whole ticket rests on
///
/// A scene that fails because it is unimplemented and a scene that fails because the code is wrong
/// are **the same failure** unless the message separates them. This one names the subject, the file
/// it belongs in, the declaration to look for, and the ticket — and says in as many words that it is
/// not a defect in the screen.
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
         components are undeclared — {}. This is not a defect in the screen. The grid is drawn, its \
         region and its tab stop are counted, its equality against a reference render holds at two \
         horizontal offsets, the arithmetic band re-damages {} cells a frame and the clip-only \
         spelling costs verbs and no writes at all — see `crate::grid::tests`. Inverted by \
         `components 15`",
        owed.len(),
        SUBJECTS.len(),
        owed.join(", "),
        RE_DAMAGED,
    ))
}

/// **Fail with the subject that is missing, the file it belongs in, and the ticket.**
///
/// # Panics
///
/// Panics while [`SUBJECTS`] is undeclared, which is **today**. Components ticket 15 inverts it.
pub fn assert_stands_up(scene: &str) {
    if let Some(message) = owed_message(&subjects_declared(), scene) {
        panic!("{message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runner::{play, reference, rows_at_a_time};

    /// The offset the scroll-scope sign is measured at. A thousand rows into a million.
    const OFF: i32 = 1_000;

    /// **The geometry the whole file's arithmetic rests on, asserted once.**
    ///
    /// Every constant below is derivable from [`columns`] and the terminal's width, so a column
    /// width edited without the constants moving is a failure here rather than a number that has
    /// quietly stopped meaning what it says.
    #[test]
    fn the_twelve_columns_overflow_their_viewport_and_the_constants_say_by_how_much() {
        let specs = columns(12);
        assert_eq!(specs.len(), 12);
        let s = solve_columns(W, &specs);
        assert_eq!((s.left_w, s.view_w, s.right_w), (LEFT_W, VIEW_W, RIGHT_W));
        assert_eq!(
            u32::from(s.left_w) + u32::from(s.view_w) + u32::from(s.right_w),
            u32::from(W),
            "the three bands tile the table"
        );
        assert_eq!(s.content_w, CONTENT_W);
        assert!(
            s.content_w > s.view_w,
            "§21's row says *horizontal overflow*, and a band that fits has none"
        );
        assert_eq!(s.max_hoff(), MAX_HOFF);
        assert_eq!(s.n, 12);
        assert_eq!(s.left, (0, 2));
        assert_eq!(s.scroll, (2, 11));
        assert_eq!(s.right, (11, 12));

        // The scrolling columns tile their content band with no gap and no overlap, which is the
        // part R03 never saw: it proved it of one band and this is three.
        for band in [s.left, s.scroll, s.right] {
            let mut x = 0i32;
            for i in band.0..band.1 {
                assert_eq!(s.x[i], x, "a gap or an overlap at slot {i}");
                x += i32::from(s.w[i]);
            }
        }

        // **`HOFF` is a column boundary and `HOFF_STRADDLE` is not.** The two constants are the
        // scene's control and its contrast, and neither is a convenience.
        assert!((s.scroll.0..s.scroll.1).any(|i| s.x[i] == HOFF));
        assert!(!(s.scroll.0..s.scroll.1).any(|i| s.x[i] == HOFF_STRADDLE));
        assert!(HOFF_STRADDLE <= s.max_hoff());

        // **The overrun, in two numbers rather than one.** The last visible column reaches past the
        // band by `reach`; what lands on the table is `reach` clipped to the terminal, and it is
        // the right pin's own width because the reach is larger than the pin.
        let (lo, hi) = visible(&s, HOFF);
        assert_eq!((lo, hi), (3, 11), "eight of the nine scrolling columns");
        assert_eq!(
            2 + (hi - lo) + 1,
            COLUMNS_DRAWN,
            "two pins, the window, one pin"
        );
        let last = hi - 1;
        let band_end = i32::from(s.left_w) + i32::from(s.view_w);
        let reach = (s.x[last] + i32::from(s.w[last])) - (HOFF + i32::from(s.view_w));
        assert_eq!(reach, 39, "how far past the band `env` is drawn");
        let on_screen = (i32::from(s.left_w) + s.x[last] - HOFF + i32::from(s.w[last]))
            .min(i32::from(W))
            - band_end;
        assert_eq!(
            on_screen,
            i32::from(BAND_OVERRUN),
            "and how much of it lands on the table"
        );
        assert_eq!(
            on_screen,
            i32::from(s.right_w),
            "the pin is overwritten whole, so the re-damage is bounded by the pin"
        );
    }

    /// **The column window this crate computes is the one the runtime's own view admits.**
    ///
    /// [`Ctx::visible_cols`](vitui_runtime::Ctx::visible_cols) as an oracle rather than as the fast
    /// path, swept over **every** offset the content admits rather than sampled at three. The fast
    /// path has to exist because [`BandShape::Arithmetic`] opens no view and so has nothing to ask;
    /// the sweep is what stops the two spellings drifting.
    #[test]
    fn the_column_window_agrees_with_the_runtimes_own_visible_cols() {
        let specs = columns(40);
        let s = solve_columns(W, &specs);
        let mut driver = crate::runner::driver_at(W, H, Density::default());
        let mut checked = 0usize;
        for hoff in 0..=s.max_hoff() {
            let (lo, hi) = visible(&s, hoff);
            let mut from_runtime = (0usize, 0usize);
            driver.frame(|cx| {
                let rect = Rect::new(i32::from(s.left_w), 0, s.view_w, 1);
                let mut clipped = cx.child(rect);
                let band = clipped.scrolled(-hoff, 0);
                let cols = band.visible_cols();
                let a = partition(s.scroll.0, s.scroll.1, |i| {
                    s.x[i] + i32::from(s.w[i]) <= cols.start
                });
                let b = partition(a, s.scroll.1, |i| s.x[i] < cols.end);
                from_runtime = (a, b);
            });
            assert_eq!(
                (lo, hi),
                from_runtime,
                "the two spellings of the column window disagree at hoff {hoff}"
            );
            assert!(hi > lo, "the viewport admits nothing at hoff {hoff}");
            checked += 1;
        }
        assert_eq!(checked as i32, s.max_hoff() + 1);
        assert!(checked > 100, "a sweep of {checked} offsets is a sample");
    }

    /// **§6's headline, as a structure: the same cells, and the verbs are the difference.**
    ///
    /// One column and twelve, over one rectangle. The cells are identical to the cell — every arm
    /// writes a partition of [`CELLS`] — and the verb count is what moves, because a list writes one
    /// row as one run and a table cuts it into columns, each of which C02's partition rule makes two
    /// verbs.
    ///
    /// §6's own 913 → 2 497 for 23 030 cells belongs to a prototype screen with two tables and three
    /// bars on it. `examples/grid_numbers.rs` prints both columns.
    #[test]
    fn the_same_rectangle_costs_the_same_cells_and_many_more_verbs_as_a_table() {
        let one = frame(Opts::correct(), 1, VOLUMES[2], 0);
        let twelve = frame(Opts::correct(), 12, VOLUMES[2], HOFF);

        assert_eq!(one.writes, CELLS, "one column, and it is a partition");
        assert_eq!(twelve.writes, CELLS, "twelve columns, and so is this");
        assert_eq!(one.distinct, CELLS, "no cell twice");
        assert_eq!(twelve.distinct, CELLS);
        assert_eq!(twelve.re_damaged(), 0, "the view arm re-damages nothing");

        assert_eq!(one.verbs, VERBS_ONE);
        assert_eq!(twelve.verbs, VERBS_TWELVE);
        assert!(
            twelve.verbs >= one.verbs * 10,
            "the `+` in `table = collection + column rectangles` is paid in verbs: {} against {}",
            twelve.verbs,
            one.verbs
        );

        // **Two verbs a cell, minus the cells with no padding in them.** §6's floor, refined by the
        // count rather than restated: `pid` at eight columns and `cpu` at seven are narrower than
        // the ten characters they carry, so the pad is a zero-length run and `Ink::run` does not
        // issue one. No cell costs three, which is the half of the sentence that matters.
        assert_eq!(
            one.verbs,
            2 * u64::from(H) * one.columns as u64,
            "nothing fills"
        );
        assert_eq!(
            twelve.verbs,
            2 * u64::from(H) * twelve.columns as u64 - u64::from(H) * FULL_COLUMNS as u64,
        );
        let full = columns(12)
            .into_iter()
            .filter(|c| {
                let w = match c.pin {
                    Pin::Left(w) | Pin::Right(w) => w,
                    Pin::None => return false,
                };
                let mut b = Buf::default();
                width(cell_text(&mut b, c.key, 0)) >= w
            })
            .count();
        assert_eq!(
            full, FULL_COLUMNS,
            "`pid` and `cpu`, and they are both pins"
        );

        // §21's row 19, and it is a relation on every arm rather than an equality on any.
        for shape in [one, twelve] {
            assert!(shape.verbs <= shape.writes);
            assert_ne!(shape.verbs, shape.writes);
        }

        // One region and one tab stop, whatever the column count: the cell under the pointer is
        // arithmetic on both axes and a virtualised collection is one tab stop.
        assert_eq!((one.regions, one.stops), (1, 1));
        assert_eq!((twelve.regions, twelve.stops), (1, 1));
    }

    /// **The column axis of the data-volume invariant: identical writes and identical verbs at 12,
    /// 40, 120 and 240 declared columns.**
    ///
    /// §21's register row 11, evaluated over this screen rather than over `table`. The extras are
    /// appended after the nine that decide the window, so the viewport admits the same columns at
    /// every declared count — which is what makes the equality a statement about the mechanism
    /// rather than about the declaration list.
    ///
    /// §6's own sweep steps once between 12 and 40 and is flat after. It does not step here, and
    /// that is the scene's requirement rather than a better result: §21's row asks for *horizontal
    /// overflow*, so this table's twelve columns already overflow and the viewport has nothing left
    /// to admit.
    #[test]
    fn writes_and_verbs_are_identical_across_declared_column_counts() {
        let across = across_declared(Opts::correct());
        assert_eq!(across.len(), 4);
        let first = across[0].1;
        assert_eq!(first.writes, CELLS);
        for (declared, shape) in &across {
            assert_eq!(
                *shape, first,
                "the grid at {declared} declared columns is not the grid at {} — and the field \
                 that moved is not `writes` alone, which is the reason the equality is over the \
                 whole shape",
                across[0].0
            );
        }

        // And the row axis of the same invariant, which is what a table inherits from a collection.
        let volumes = across_volumes(Opts::correct());
        for (rows, shape) in &volumes {
            assert_eq!(
                *shape, first,
                "the grid at {rows} rows is not the grid at 1 000"
            );
        }
        assert_eq!(first.iterated, u64::from(H), "the window and nothing else");
    }

    /// **The clip-only spelling costs verbs and no writes at all**, which is the finding.
    ///
    /// `crate::listing`'s discovery arriving on the column axis: the engine reports a fully clipped
    /// verb as **zero columns**, so a table that declares every column and lets the clip discard the
    /// rest writes exactly what the virtualised one writes. §21's register row 4 — *writes flat
    /// 1k → 1M* — and every write-count gate in the stack are green on it.
    ///
    /// What moves is `verbs`, and `asked` beside it is where the cells went: the columns were
    /// measured, truncated and handed to a verb, and the clip threw them away. §6's *3.3x the frame
    /// and 7.2x the verbs* is one screen's version of the ratio below; this screen's is larger,
    /// because §6's prototype capped a frame at 128 drawn columns and this one does not.
    #[test]
    fn the_clip_only_spelling_costs_verbs_and_no_writes_at_all() {
        let virtualised = frame(Opts::correct(), 120, VOLUMES[2], HOFF);
        let clip_only = frame(Opts::clip_only(), 120, VOLUMES[2], HOFF);

        assert_eq!(
            clip_only.writes, virtualised.writes,
            "the write count cannot see this defect"
        );
        assert_eq!(
            clip_only.distinct, virtualised.distinct,
            "and neither can `distinct`"
        );
        assert_eq!(
            clip_only.regions, virtualised.regions,
            "and neither can the index"
        );
        assert_eq!(
            clip_only.stops, virtualised.stops,
            "and neither can the ring"
        );

        assert!(
            clip_only.verbs >= virtualised.verbs * 10,
            "verbs are the counter that sees it: {} against {}",
            clip_only.verbs,
            virtualised.verbs
        );
        assert!(
            clip_only.asked >= virtualised.asked * 4,
            "and `asked` is where the cells went: {} against {}",
            clip_only.asked,
            virtualised.asked
        );
        assert_eq!(
            (virtualised.asked, virtualised.writes),
            (ASKED, CELLS),
            "the virtualised arm still asks for more than lands — the band's clip cuts the column \
             that overruns it — so `asked` is a measure of the clip and never of the window"
        );
        assert_eq!(clip_only.columns, 120, "every declared column drawn");
        assert_eq!(virtualised.columns, COLUMNS_DRAWN);

        // **Which of §20's nine separate them. One, and it is `verbs`.**
        let inert = Allocations::over(1, 0);
        assert_eq!(
            counters_that_separate(Opts::correct(), Opts::clip_only(), inert, inert),
            vec![Counter::Verbs],
            "at twelve declared columns, where the clip-only arm draws one extra column and the \
             write count still cannot see it"
        );
    }

    /// **The band defect: the equality is blind to it and the pair is not.**
    ///
    /// Criterion 3, and the scene's whole finding. §6 says *no gate left by C01, C02 or C03 sees
    /// it*; measured here, one does — C02's `writes == distinct`, §21's register row 6, which §21
    /// itself files as a **report per component**. What is true is the half §6 states in the clause
    /// before: the *screen* is correct, so the equality against a reference render reports **0 cells
    /// over 0 rows** on a build that will re-damage [`RE_DAMAGED`] cells every frame for ever.
    ///
    /// Both directions, because a gate that only fires is a gate that might fire on anything: the
    /// correct arm is compared against the same oracle and is clean, and the defective arm's excess
    /// is asserted to the cell.
    #[test]
    fn the_equality_is_blind_to_the_band_and_the_pair_is_not() {
        // The equality, on the arm that is wrong.
        let blind = equality(arithmetic, HOFF);
        assert_eq!(
            (blind.cells, blind.rows),
            (0, 0),
            "the pinned band draws afterwards and wins, so the picture is right: {blind}"
        );
        // And on the arm that is right, which is what stops the line above meaning *the instrument
        // is broken*.
        assert!(equality(banded, HOFF).clean());

        // The pair, on the same two arms.
        let correct = frame(Opts::correct(), 12, VOLUMES[2], HOFF);
        let broken = frame(Opts::arithmetic_band(), 12, VOLUMES[2], HOFF);
        assert_eq!(correct.re_damaged(), 0);
        assert_eq!(
            broken.re_damaged(),
            RE_DAMAGED,
            "{} columns a row over {H} rows, and the columns are the right pin's own width",
            BAND_OVERRUN
        );
        assert_eq!(
            broken.distinct, CELLS,
            "it still covers the screen exactly once"
        );
        assert_eq!(
            broken.writes,
            CELLS + RE_DAMAGED,
            "**`writes` is not identical**, and §6's sentence that it is belongs to a counter that \
             folded a `fill` in as the rectangle it asked for"
        );
        assert_eq!(
            broken.verbs, correct.verbs,
            "verbs are identical, which is the half of §6's sentence that does reproduce"
        );
        assert_eq!(
            broken.asked, correct.asked,
            "and so is what was asked for — which is why `asked` cannot separate them either: both \
             arms hand the same columns to the same verbs and only one of them has a clip in the \
             way"
        );
        assert_eq!(correct.asked, ASKED);
        assert_eq!(correct.asked - correct.writes, ASKED_DISCARDED);
        assert_eq!(
            (broken.regions, broken.stops),
            (correct.regions, correct.stops)
        );

        // **Which of §20's nine separate the arms.** One, and it is not one anybody was running:
        // §21 files `writes == distinct` as a report per component.
        let inert = Allocations::over(1, 0);
        let separating =
            counters_that_separate(Opts::correct(), Opts::arithmetic_band(), inert, inert);
        assert_eq!(
            separating,
            vec![Counter::Writes],
            "`writes` alone moves, and only because this crate keeps `asked` and `reported` apart"
        );
        assert_eq!(
            correct.writes,
            broken.writes - RE_DAMAGED,
            "and the difference is exactly the overrun"
        );
    }

    /// **The other half of the band, and the half §6's sentence is not about.**
    ///
    /// At an offset inside a column the overrun is on the **left**, where the pins have already
    /// drawn. The picture is wrong, so the equality does fire — and the pair fires too, which is
    /// what makes the boundary case worth a constant of its own: a scene played only here would
    /// conclude the equality catches the band defect, and a scene played only at a boundary would
    /// conclude the picture is always right. Neither is true.
    #[test]
    fn at_an_offset_inside_a_column_the_overrun_is_on_the_left_and_the_picture_is_wrong() {
        let straddling = equality(arithmetic_straddling, HOFF_STRADDLE);
        assert!(
            !straddling.clean(),
            "the left pin is written over and nothing draws it again: {straddling}"
        );
        assert_eq!(
            straddling.rows,
            usize::from(H),
            "every row of the screen, because every row has the same pins: {straddling}"
        );
        assert_eq!(
            straddling.cells, STRADDLE_CELLS,
            "ten cells a row of the eighty rows, less the 88 where the column that should be there \
             and the column that is there carry the same digit: {straddling}"
        );
        assert_eq!(
            straddling.first,
            Some((15, 0)),
            "and not (14, 0): the very first cell of the overrun is one of the coincidences"
        );

        // The pair sees **both** edges here, and the arithmetic is the two overruns added.
        let broken = frame(Opts::arithmetic_band(), 12, VOLUMES[2], HOFF_STRADDLE);
        assert_eq!(broken.distinct, CELLS);
        assert_eq!(broken.re_damaged(), STRADDLE_RE_DAMAGED);
        assert_eq!(
            broken.re_damaged(),
            RE_DAMAGED + u64::from(STRADDLE_OVERRUN) * u64::from(H),
            "the right edge it has at every offset, plus the left edge it has only here"
        );

        // **And the correct arm at the same offset is not clean, which is a fact about the
        // recorder.** The band's clip discards a prefix here, and ADR 0022's clamp-and-discard
        // makes `Pen` wrong by exactly that prefix — it is the reason `HOFF` is a column boundary,
        // asserted rather than described so that the day an instrument learns the frame's origin
        // this line fails and says so.
        let recorder = equality(banded_straddling, HOFF_STRADDLE);
        assert_eq!(
            (recorder.cells, recorder.rows),
            (STRADDLE_RECORDER_CELLS, usize::from(H)),
            "the correct arm disagrees with the oracle at a straddling offset, and the painter is \
             not what disagrees: {recorder}"
        );
        assert_eq!(recorder.first, Some((68, 0)));
    }

    /// **The inverted horizontal sign: caught by the equality, and by nothing else.**
    ///
    /// C03's trap on the other axis, and §21's register row 12 doing the job the research assigned
    /// it. The defective arm issues **every verb** it would have issued and the clip eats most of
    /// the writes, so it draws a partly blank band and measures as an improvement — fewer writes, a
    /// shorter frame, the same verbs.
    #[test]
    fn the_inverted_horizontal_sign_is_refused_by_the_equality_and_flattered_by_the_counters() {
        let diff = equality(inverted, HOFF);
        assert!(!diff.clean(), "a partly blank band is a different picture");
        assert_eq!(diff.rows, usize::from(H), "every row of the screen: {diff}");
        assert_eq!(diff.cells, INVERTED_CELLS, "{diff}");

        let correct = frame(Opts::correct(), 12, VOLUMES[2], HOFF);
        let broken = frame(Opts::inverted_sign(), 12, VOLUMES[2], HOFF);
        assert_eq!(
            broken.verbs, correct.verbs,
            "every verb is issued, which is why a verb count is on the wrong side of this one"
        );
        assert_eq!(broken.asked, correct.asked, "and every cell is asked for");
        assert!(
            broken.writes < correct.writes,
            "and the defective arm writes **fewer** cells: {} against {}",
            broken.writes,
            correct.writes
        );
        assert_eq!(broken.writes, INVERTED_WRITES);
        assert_eq!(
            broken.distinct, broken.writes,
            "it writes no cell twice — it simply leaves 15 360 of them alone, which is the \
             sentinel's question and the one gate this crate cannot write"
        );

        // **Two of §20's nine separate these arms**, and they are the two the band defect has one
        // of. That is the contrast the scene is for: a defect that draws *less* is visible to a
        // counter and a defect that draws the same cells twice is visible to a pair.
        let inert = Allocations::over(1, 0);
        assert_eq!(
            counters_that_separate(Opts::correct(), Opts::inverted_sign(), inert, inert),
            vec![Counter::Writes, Counter::Distinct]
        );
    }

    /// **The correct arm agrees with the oracle cell for cell**, which is what makes every
    /// disagreement above evidence about the painter rather than about the instrument.
    ///
    /// And the oracle is not the fast path in disguise: it is built by inverting the map — screen x
    /// to column, one cell at a time — where the row loop applies it column to screen x.
    #[test]
    fn the_oracle_and_the_grid_are_two_programs_that_agree() {
        assert!(equality(banded, HOFF).clean());
        let run = play(banded, &[oracle(12, HOFF)]);
        assert_eq!(
            run.canvas().written(),
            CELLS as usize,
            "every cell of the rectangle written at least once, which is the other direction of \
             the partition"
        );
        assert_eq!(run.tally().writes(), CELLS, "and exactly once");
        assert_eq!(
            run.tally().asked(),
            ASKED,
            "with the clip taking the difference"
        );
    }

    /// **The cheap per-cell oracle and components ticket 04's are the same program.**
    ///
    /// [`per_cell`] exists because [`crate::runner::reference`] asks a [`Fixture`] for every cell
    /// independently and the fixture segments a whole row to answer — 250x too slow at 300 columns
    /// and exactly right at the forty this compares them over. The claim being checked is that the
    /// substitution is a substitution: same cells, same clusters, same count.
    ///
    /// It runs over [`crate::runner::Fixture::lines`] rather than over this screen's oracle for the
    /// reason the substitution exists: the expensive one cannot be afforded on this screen, so a
    /// comparison there would be a comparison nobody runs.
    #[test]
    fn the_two_per_cell_oracles_are_the_same_program() {
        let fx = crate::runner::Fixture::lines(40, 12, 200).scrolled_to(7);
        let diff = compare(reference, per_cell, std::slice::from_ref(&fx));
        assert_eq!((diff.cells, diff.rows), (0, 0), "{diff}");

        // And both are still a partition of the rectangle, which is what stops *they agree* from
        // meaning *neither drew anything*.
        let mine = play(per_cell, std::slice::from_ref(&fx));
        assert_eq!(mine.canvas().written(), 40 * 12);
        assert_eq!(mine.tally().verbs(), 40 * 12, "one `Ctx::set` a cell");
        assert!(compare(per_cell, rows_at_a_time, &[fx]).clean());
    }

    /// **A frame of the grid allocates nothing**, as a total.
    ///
    /// §21's refinement 2: *a mean cannot see anything below n; a total can see one*. The probe is
    /// installed by the caller — `examples/grid_numbers.rs` — because a library cannot install a
    /// global allocator on a consumer's behalf; what is asserted here is the thing that makes the
    /// total assertable at all, which is that the draw path holds no `Vec` and no `format!`.
    #[test]
    fn a_frame_of_the_grid_allocates_nothing_it_can_be_asked_about() {
        // The declaration list is the caller's and is built once, outside the frame. What may not
        // allocate is the row loop, and the two things that would are a per-frame scratch vector
        // and a formatted cell — the first is a fixed array on `Solved` and the second is `Buf`.
        let mut buf = Buf::default();
        assert_eq!(cell_text(&mut buf, 7, 1_234), "07.0001234");
        assert_eq!(cell_text(&mut buf, 11, 0), "11.0000000");
        assert_eq!(
            cell_text(&mut buf, 3, 999_999),
            "03.0999999",
            "a million rows still fits the buffer"
        );

        // And every cell of one row is distinct in its **first** characters, which is what makes a
        // wrong column cost its own columns rather than its suffix.
        let specs = columns(12);
        let mut seen = std::collections::BTreeSet::new();
        for spec in &specs {
            let mut b = Buf::default();
            assert!(
                seen.insert(cell_text(&mut b, spec.key, 42).to_string()),
                "two columns of one row say the same thing"
            );
        }
    }

    /// **`Ctx::scroll_scope` scrolls the wrong way, and this is the measurement.**
    ///
    /// Not a gate on this crate's own code, and it is here because it is the reason [`OFFSET`] is
    /// zero. The runtime reads a scroll offset as **positive** everywhere it does arithmetic —
    /// `vitui_runtime::scroll::Area::into_view` compares against `offset..offset + window`, and
    /// `Scrollable::between((0, 0), (0, max))` answers `DOWN` — and the engine's rule is that *a
    /// viewport scrolled `n` rows down is `scrolled(0, -n)`*. `scroll_scope` passes `+n`.
    ///
    /// So the scope's own window is the rows **above** the content: at an offset of a thousand it
    /// answers `-1000..-920`, a write at content row 1 000 reports zero columns, and a write at
    /// −1 000 lands. **It is C03's inverted scroll sign inside the runtime's own verb**, and it has
    /// survived because every caller that draws through a scroll scope draws at offset zero —
    /// which is §21's argument for the scene list, arriving one layer down.
    ///
    /// Filed as `.scratch/vitui-runtime-architecture/issues/26`. Both directions are asserted, so
    /// the day the sign is corrected this test fails rather than quietly passing.
    #[test]
    fn a_scroll_scope_at_a_nonzero_offset_shows_the_rows_above_the_content() {
        use vitui_runtime::Role;

        let mut driver = crate::runner::driver_at(W, H, Density::default());
        let id = Id::named("grid");
        let view = Rect::new(0, 0, W, H);
        let (mut window, mut at_content, mut at_negative) = (0..0, 0u16, 0u16);
        driver.frame(|cx| {
            cx.scroll_scope(id, view, (0, OFF), (0, max_offset(VOLUMES[2])), |cx| {
                let paint = cx.theme().paint(Role::Body);
                window = cx.visible_rows();
                at_content = cx.text(0, OFF, "x", paint).cells;
                at_negative = cx.text(0, -OFF, "x", paint).cells;
            });
        });
        assert_eq!(
            window,
            -OFF..-OFF + i32::from(H),
            "the scope's window is the rows above the content"
        );
        assert_eq!(at_content, 0, "and content row {OFF} is off the clip");
        assert_eq!(at_negative, 1, "while row -{OFF} is on the screen");

        // The other direction, at the offset this screen actually uses: at zero the sign cannot be
        // seen at all, which is why nothing has seen it.
        driver.frame(|cx| {
            cx.scroll_scope(id, view, (0, 0), (0, max_offset(VOLUMES[2])), |cx| {
                window = cx.visible_rows();
            });
        });
        assert_eq!(window, 0..i32::from(H), "and at zero the two signs agree");
        assert_eq!(
            OFFSET, 0,
            "which is the offset every frame here is drawn at"
        );
    }

    /// **Criterion 7: the two scenes are red because the subject is missing, and they say so.**
    ///
    /// The inversion of this test is components ticket 15's, and it is a deliberate edit in three
    /// files — here, in `crate::scenes`'s two standings and in `crate::gates::REGISTER`.
    #[test]
    fn the_grid_is_red_because_table_is_not_declared() {
        assert_eq!(
            subjects_declared(),
            Vec::<&str>::new(),
            "`table` is declared. That inverts two scenes and one register row, and it is a \
             deliberate edit in all three files"
        );
        let verdict = standing();
        assert!(!verdict.met());
        match verdict {
            Verdict::Unmet {
                over,
                failing,
                inverted_by,
                ..
            } => {
                assert_eq!((over, failing), (1, 1), "one subject, and it is missing");
                assert_eq!(inverted_by, "components 15");
            }
            Verdict::Met { over } => {
                unreachable!("{over} declared, which the assertion above caught")
            }
        }

        let panicked = std::panic::catch_unwind(|| {
            assert_stands_up("a twelve-column 1M-row table, pinned both edges")
        });
        assert!(
            panicked.is_err(),
            "a scene with no subject does not stand up"
        );
    }

    /// **The waiting message says which failure it is**, which is the distinction criterion 7 is
    /// about.
    ///
    /// Fired in both directions over a declaration list rather than over the crate, so that the day
    /// `table` is declared this half is still live: a message no test can read is a message that
    /// rots.
    #[test]
    fn the_waiting_message_separates_unimplemented_from_wrong() {
        let message = owed_message(&[], "a twelve-column table").expect("no subject is declared");
        assert!(
            message.contains("waiting for its subject rather than failing"),
            "{message}"
        );
        assert!(
            message.contains("This is not a defect in the screen"),
            "{message}"
        );
        assert!(message.contains("components 15"), "{message}");
        assert!(message.contains("src/collect.rs"), "{message}");
        assert!(message.contains("pub fn table("), "{message}");
        assert!(message.contains(&RE_DAMAGED.to_string()), "{message}");

        // The other direction: with the subject declared there is no message at all.
        assert_eq!(owed_message(&SUBJECTS, "a twelve-column table"), None);
    }

    /// **The subject scan finds a declaration when there is one**, and the freeze homes `table`
    /// where the scan opens.
    #[test]
    fn the_subject_scan_finds_a_declaration_and_the_freeze_agrees_with_it() {
        for (_, declaration) in DECLARATIONS {
            assert!(crate::dense::declares(
                &format!("// a comment\n{declaration}cx: &mut Ctx) {{}}\n"),
                declaration
            ));
            assert!(
                !crate::dense::declares(
                    &format!("// {declaration}…) is components 15's\n"),
                    declaration
                ),
                "a mention in a comment is not a declaration"
            );
        }
        for (file, _) in DECLARATIONS {
            let path =
                std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).join(file);
            assert!(path.is_file(), "{}", path.display());
        }

        let component = crate::INVENTORY
            .iter()
            .find(|c| c.id == SUBJECTS[0])
            .expect("`table` is in the freeze");
        assert_eq!(component.families[0].module(), Some("collect"));
        assert!(component.families[0].members().contains(&SUBJECTS[0]));
        assert_eq!(DECLARATIONS[0].0, "collect.rs");

        // The two axes the scenes claim, and the freeze sets both.
        assert!(component.declares(crate::Axis::Scrolled));
        assert!(component.declares(crate::Axis::Narrow));
    }
}
