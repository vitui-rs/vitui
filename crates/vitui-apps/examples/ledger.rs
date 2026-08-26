//! **A general ledger of a million entries, twelve columns, and both edges pinned.**
//!
//! The application components ticket 15 is for. `table` is `collection` **plus a column rect split**
//! (spec §6, ADR 0028), and the three claims that sentence carries are all things you have to watch
//! rather than be told:
//!
//! ```text
//! ┌ Ledger — 1 000 000 entries ────────────────────────────────────────────────────────┐
//! │ id         date        account            description             …      balance   │
//! │ #0000000   2019-01-01  1000 Cash          Opening balance         …    12 340.00   │
//! │ #0000001   2019-01-02  4000 Revenue       Invoice INV-0001        …    13 180.00   │
//! │ #0000002   2019-01-03  5100 Rent          Monthly office rent     …    11 980.00   │
//! │ …                                                                                  │
//! │ cell (41, credit)  ·  selected 80 in 1 span, 16 B  ·  Σ 6 400.00  ·  rev 7          │
//! │ 24 000 cells / 24 000 distinct / 1 600 verbs  ·  cols 3..9 of 12  ·  hoff 96/34     │
//! └────────────────────────────────────────────────────────────────────────────────────┘
//!   ^^ pinned            ^^^^^^^^^^^^^^^^ scrolls ^^^^^^^^^^^^^^^^            pinned ^^
//! ```
//!
//! # The `+` is paid in verbs, and the counter is on the screen
//!
//! §6's headline is that a table and a list write **the same cells** and differ in **drawing
//! calls** — a table cuts each row into one short run per column where a list writes one long one,
//! and §2's partition rule then makes each column two verbs, the text and the padding after it.
//! `Alt+1` draws the same rectangle as one column and `Alt+2` as twelve: the cell count does not
//! move and the verb count does, by an order of magnitude.
//!
//! The numbers on the status line are the **table's own**, not a model of it. `Alt+v` swaps the
//! ink the component draws through from `Direct` to [`Tally`] — the same code path, the same verbs,
//! with a recorder folded in — which is the seam `crate::ink` exists for and the reason an
//! application can print these at all. It is not free: the tally keeps a set of every cell it sees,
//! so leaving it on costs an allocation and a tree walk a cell. That is the instrument's price and
//! it is why it is a key rather than the default.
//!
//! # Three bands, and the pins do not move
//!
//! `id` and `date` are pinned left, `balance` is pinned right, and the nine between them scroll.
//! Hold `Alt+→` and watch the middle slide under two columns that stay put. **A pinned column may
//! not be elastic** — a pin claims a share of the *table* and a scrolling lane a share of the
//! *content*, and `max(viewport, Σ minima)` gives them two different denominators — so a
//! `Constraint` on a pinned column is not consulted at all. The status line prints
//! `cols lo..hi of n`: only the columns the viewport admits are drawn, which is `visible_cols`
//! doing on the column axis what `visible_rows` does on the row axis.
//!
//! # Cell selection is a run list per column key, and the footer is why
//!
//! `Alt+x` toggles the cell under the cursor, `Alt+c` selects the whole column, `Alt+r` the whole
//! row. The footer sums the selected cells of the cursor's column — which is *why* the store is
//! per column: summing a column is the gesture a ledger actually has.
//!
//! Press `Alt+c` on `debit` at a million rows and read the span count: **1 span, 16 bytes**. Under
//! the flattened `row · ncols + col` index C03 proposed, the same click is **1 000 000 runs and
//! 16 MB**, because a column of a row-major flattening is a stride and a stride of length one is
//! not a run. Then press `Alt+r` and watch the count go to twelve — that is the store's *loss*, and
//! it is bounded by the declared column count while the win is bounded by nothing.
//!
//! # In-cell editing is one position, and `Alt+s` is the proof
//!
//! `Enter` opens an editor on the cell under the cursor. The slot is `Option<(row, col)>` and no
//! wider: the row half is a **position** and the column half is a **key**, because hiding and
//! reordering columns move positions and nothing stored may be keyed on one.
//!
//! Open the editor, then sort:
//!
//! | | |
//! |---|---|
//! | `Alt+s` | reverse the order and **do not** reconcile. The revision the component compares once a frame stops matching, and the editor closes and the cell selection empties — because every one of those is a position in an order that has been replaced |
//! | `Alt+S` | reverse the order **and carry the positions across**. The editor follows its row, the cursor follows, and the spans are remapped — one call each, at any length, because a slot is one position |
//!
//! Only the caller can do the second one, and that is ADR 0031 rather than a limitation: the
//! component was not told what the permutation was. `Alt+s` is the honest default for an edit
//! nobody explained.
//!
//! # Identity is per cell, and nothing on the screen would tell you
//!
//! Every visible cell declares a target, so hovering lights one cell rather than one row. Keyed per
//! **row** instead — one microsecond cheaper and correct until a cell declares a target — the first
//! cell of each row would answer and the other eleven would be inert, on a screen that renders
//! pixel for pixel identically. `merges` is the only counter that sees it, which is why components
//! ticket 15 gates it with a count and not with a picture.
//!
//! # A million entries and no ledger
//!
//! There is no `Vec<Entry>`. Every column of every row is computed from its index, which is the
//! honest way to show the invariant the whole stack rests on — *frame cost is proportional to
//! visible cells, never to data volume*. The order is a function too: reversing it is arithmetic,
//! so `Alt+s` is `O(1)` and the thing being demonstrated is the **revision**, not a sort.
//!
//! # Two things this application cannot say, and they are the surface's to fix
//!
//! 1. **A table has no horizontal wheel.** The vertical offset is `collection`'s and is applied
//!    from *this frame's* wheel inside the draw; `TableState::hoff` is the caller's, because the
//!    column solve runs **before** the row pass and a wheel read where the vertical one is read
//!    would land a frame late — which is the defect runtime ticket 14 found and removed. So the
//!    band moves under `Alt+←/→` and not under a trackpad.
//! 2. **The header cannot be measured while it is drawn.** `Alt+h` turns it on, and with it on the
//!    body's view starts a row down: a recorder that unions in the coordinates of the `Ctx` the
//!    verb was called on then sees the header's row and the body's first row as one, and reports a
//!    double write of exactly one header row. It is the recorder's and not the component's — `Ctx`
//!    publishes no accessor for the frame's own origin — and the `distinct` figure on the status
//!    line is the number saying so.
//!
//! # The keyboard
//!
//! | | |
//! |---|---|
//! | `↑` `↓` `Home` `End` `PgUp` `PgDn` | the row cursor. `collection`'s, unchanged |
//! | `,` `.` | the **column** cursor, scrolling the band to keep it in view (`Alt+←` `Alt+→` too) |
//! | `[` `]` | scroll the band without moving the cursor |
//! | `x` | toggle the cell under the cursor |
//! | `c` / `r` | select the whole column / the whole row |
//! | `z` | clear the cell selection |
//! | `Enter` | open the in-cell editor. `Enter` commits, `Esc` cancels |
//! | `s` / `S` | reverse the order, dropping / carrying the positions |
//! | `1` / `2` | draw as one column / as twelve |
//! | `v` | the live counters, through a `Tally` |
//! | `h` | the header |
//! | `Ctrl+Q` | quit. **`q` alone is racy here** and the pair is the point: a focused `collection` consumes every text-bearing key into its type-ahead buffer (spec §5), so a plain `q` reaches the application only on a frame where nothing is focused. `Ctrl` is not text |
//!
//! **The plain letters are this application's luck and not the component's gift.** A key reaches
//! the application only when the component declines it, and `collection` declines a letter only
//! because the search this file hands it always answers `None` — a ledger over a million computed
//! rows has nothing to type-ahead to. Give the table a real search and every letter is claimed by
//! the type-ahead buffer, and every binding above has to become a chord; `triage` is that
//! application, which is why `q` is the only plain key it has.
//!
//! **The column cursor is `,` and `.` and not `Alt+←/→`, and that is a fact about macOS.** The
//! arrows themselves are gone — `nav::step` takes `←` and `→` as row-cursor moves whatever the
//! search does — so the column axis needs a second spelling, and the obvious one does not arrive:
//! Ghostty's `macos-option-as-alt` is **unset by default**, so Option composes `é` rather than
//! reporting Alt and the chord never reaches the application at all. `Alt+←/→` is bound anyway for
//! anyone who has set it (`macos-option-as-alt = true` in `~/.config/ghostty/config`, or the
//! equivalent in iTerm2 and Terminal.app), but nothing here needs it. An application that requires
//! a terminal setting to be usable is an application with a defect.
//!
//! # Run it
//!
//! ```text
//! cargo run -p vitui-apps --example ledger
//! ```
//!
//! [`Tally`]: vitui_components::counters::Tally

use std::fmt::{self, Write as _};

use vitui_components::collect::{
    Cell, CellSel, Column, Selection, TableOpts, TableState, solve_columns, table_into,
    visible_columns,
};
use vitui_components::counters::Tally;
use vitui_components::frame::{Face, face_paint};
use vitui_components::ink::{Direct, Ink};
use vitui_components::order::Rows;
use vitui_components::structure::{PanelOpts, panel_with};
use vitui_components::text::{FitOpts, Justify, fit_with};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::layout::Constraint::{Fixed, Weight};
use vitui_runtime::layout::{Col, text::truncate};
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Interest, Rect, Revision, Role, Themes};

// ── the ledger, which is a function and not a store ──────────────────────────────────────────────

/// How many entries the ledger holds. **A million, and none of them exists.**
const ENTRIES: usize = 1_000_000;

/// The accounts, cycled.
const ACCOUNTS: [&str; 9] = [
    "1000 Cash",
    "1200 Receivable",
    "2000 Payable",
    "3000 Equity",
    "4000 Revenue",
    "5100 Rent",
    "5200 Salaries",
    "5300 Software",
    "6000 Interest",
];

/// What the entry is for, cycled against a different modulus so the pairs do not repeat.
const NARRATIVES: [&str; 7] = [
    "Opening balance",
    "Monthly office rent",
    "Invoice settled in full",
    "Payroll run, semi-monthly",
    "Annual licence renewal",
    "Bank interest received",
    "Refund against credit note",
];

/// Who the other side is.
const PARTIES: [&str; 11] = [
    "Aldridge & Co",
    "Bergstrom AB",
    "Caldera Ltd",
    "Duplessis SARL",
    "Erikson Bros",
    "Fenwick PLC",
    "Grimaldi SpA",
    "Halvorsen AS",
    "Ibarra SL",
    "Jankowski sp.",
    "Kuroda KK",
];

/// The tags, cycled.
const TAGS: [&str; 5] = ["q1", "recurring", "reviewed", "accrual", "fx"];

/// The currencies, cycled.
const CURRENCIES: [&str; 4] = ["EUR", "USD", "GBP", "SEK"];

/// What entry `i` debits, in minor units. Zero on the credit rows.
fn debit(i: usize) -> i64 {
    if i.is_multiple_of(3) {
        0
    } else {
        ((i as i64 * 7919) % 480_000) + 500
    }
}

/// What entry `i` credits, in minor units. Zero on the debit rows.
fn credit(i: usize) -> i64 {
    if i.is_multiple_of(3) {
        ((i as i64 * 6151) % 320_000) + 500
    } else {
        0
    }
}

/// The running balance after entry `i`. Arithmetic, so it costs the same at row 999 999.
fn balance(i: usize) -> i64 {
    1_234_000 + (i as i64 % 97) * 1_100 - (i as i64 % 41) * 700
}

/// The day entry `i` is dated, as an offset from 2019-01-01.
fn date(buf: &mut Line, i: usize) -> &str {
    let day = i % 28 + 1;
    let month = (i / 28) % 12 + 1;
    let year = 2019 + (i / (28 * 12)) % 6;
    buf.clear();
    let _ = write!(buf, "{year:04}-{month:02}-{day:02}");
    buf.as_str()
}

/// An amount in minor units, as a string with two decimals and a thousands separator.
fn amount(buf: &mut Line, v: i64) -> &str {
    buf.clear();
    if v == 0 {
        return "";
    }
    let (units, cents) = (v / 100, (v % 100).abs());
    let mut group = String::new();
    let _ = write!(group, "{}", units.abs());
    let mut out = String::new();
    for (n, ch) in group.chars().enumerate() {
        if n > 0 && (group.len() - n).is_multiple_of(3) {
            out.push(' ');
        }
        out.push(ch);
    }
    let sign = if v < 0 { "-" } else { "" };
    let _ = write!(buf, "{sign}{out}.{cents:02}");
    buf.as_str()
}

// ── the columns ──────────────────────────────────────────────────────────────────────────────────

/// The key of the column that holds the running balance, which is the one the footer sums by
/// default and the one pinned to the right edge.
const BALANCE: u16 = 11;

/// **The twelve declared columns.** Two pinned left, nine scrolling, one pinned right.
///
/// The keys are `0..12` here and that is a coincidence of this application rather than a rule: a
/// key is **stable identity**, and hiding or reordering a column moves its position and not its
/// key. Everything this file stores per column — the cursor's column, the editing slot's column
/// half, every entry in [`CellSel`] — takes the key.
fn columns() -> Vec<Column> {
    vec![
        Column::new(0, "id", Fixed(10)).pinned_left(10),
        Column::new(1, "date", Fixed(11)).pinned_left(11),
        Column::new(2, "account", Fixed(24)),
        Column::new(3, "description", Fixed(40)),
        Column::new(4, "counterparty", Fixed(26)),
        Column::new(5, "reference", Fixed(18)),
        Column::new(6, "memo", Fixed(36)),
        Column::new(7, "tags", Fixed(20)),
        Column::new(8, "ccy", Fixed(5)),
        Column::new(9, "debit", Fixed(14)),
        Column::new(10, "credit", Fixed(14)),
        Column::new(BALANCE, "balance", Fixed(16)).pinned_right(16),
    ]
}

/// **The same rectangle as one column**, which is `Alt+1` and §6's control.
///
/// A list, expressed as a table: the same rows, the same cells, and one verb pair a row instead of
/// one a cell. What it isolates is the price of the `+`.
fn one_column() -> Vec<Column> {
    vec![Column::new(0, "entry", Weight(1))]
}

// ── a stack buffer, because the frame allocates nothing ──────────────────────────────────────────

/// Sixty-four bytes on the stack, for a cell's text.
///
/// `format!` allocates and a frame draws twenty-four thousand cells. `Ctx::stage`/`Ctx::blit` is the
/// runtime's own answer to the same problem and is not reachable through the ink seam, which takes a
/// `&str` — so an application that cares keeps its own, exactly as `crate::grid`'s fixture does one
/// crate down.
struct Line {
    bytes: [u8; 64],
    len: usize,
}

impl Default for Line {
    fn default() -> Line {
        Line {
            bytes: [0; 64],
            len: 0,
        }
    }
}

impl fmt::Write for Line {
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

impl Line {
    fn clear(&mut self) {
        self.len = 0;
    }

    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.bytes[..self.len]).unwrap_or("")
    }
}

/// What cell `(row, key)` says.
fn cell_text(buf: &mut Line, key: u16, row: usize) -> &str {
    match key {
        1 => date(buf, row),
        2 => ACCOUNTS[row % ACCOUNTS.len()],
        3 => NARRATIVES[(row / ACCOUNTS.len()) % NARRATIVES.len()],
        4 => PARTIES[row % PARTIES.len()],
        7 => TAGS[row % TAGS.len()],
        8 => CURRENCIES[row % CURRENCIES.len()],
        9 => amount(buf, debit(row)),
        10 => amount(buf, credit(row)),
        BALANCE => amount(buf, balance(row)),
        0 => {
            buf.clear();
            let _ = write!(buf, "#{row:07}");
            buf.as_str()
        }
        5 => {
            buf.clear();
            let _ = write!(buf, "INV-{:06}", row % 999_983);
            buf.as_str()
        }
        _ => {
            buf.clear();
            let _ = write!(buf, "posted by batch {}", row % 512);
            buf.as_str()
        }
    }
}

/// The value the footer sums for a column, or `None` where the column is not an amount.
fn value(key: u16, row: usize) -> Option<i64> {
    match key {
        9 => Some(debit(row)),
        10 => Some(credit(row)),
        BALANCE => Some(balance(row)),
        _ => None,
    }
}

/// What a cell needs to know that is not in its [`Cell`], taken out of the application before the
/// draw because `table_into` borrows the state for the whole of it.
struct Look {
    cells: CellSel,
    editing: Option<String>,
    reversed: bool,
    cursor_col: u16,
    cursor_row: usize,
}

/// **One cell, drawn through the ink the component was handed.**
///
/// Generic over [`Ink`] and **not** written against `cx` directly, which is the whole of why the
/// counters on the status line are the table's own. Drawn with `cx.text` instead — the first shape
/// this file took — the recorder sees the component's header and nothing else, and the screen still
/// looks right: **78 cells and 10 verbs on a frame that wrote 1 560**. That is `crate::ink`'s
/// argument arriving from the consumer's side.
fn paint_cell<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    r: Rect,
    c: Cell,
    face: Face,
    look: &Look,
) {
    let mut buf = Line::default();
    let row = match look.reversed {
        true => ENTRIES - 1 - c.row,
        false => c.row,
    };
    let here = c.row == look.cursor_row && c.key == look.cursor_col;
    let edited = here && look.editing.is_some();
    let text = match (&look.editing, here) {
        (Some(edit), true) => edit.as_str(),
        _ => cell_text(&mut buf, c.key, row),
    };
    // **A target per cell**, which is what makes the hover land on one cell rather than on one row.
    // The id is the component's, minted from the table's own and handed over — see
    // `collect::Cell::id` for why it is handed over rather than pushed.
    let _ = cx.interact(c.id, r, Interest::CLICK.with(Interest::HOVER));
    let paint = match edited {
        true => cx.theme().paint(Role::FaceActive),
        false => face_paint(
            cx.theme(),
            Face {
                selected: face.selected || look.cells.contains(row, c.key),
                cursor: here,
                ..face
            },
        ),
    };
    // **A partition of the cell**, which is the rule every drawer owes (§2): the text, then the
    // padding after it, and no cell written twice. Both verbs go through the ink, so `writes`,
    // `distinct` and `verbs` are the frame's and not the header's.
    let cut = truncate(text, r.w);
    let n = ink.text(cx, r.x, r.y, cut, paint);
    let _ = ink.run(cx, r.x + i32::from(n), r.y, " ", r.w - n, paint);
}

// ── the application ──────────────────────────────────────────────────────────────────────────────

/// Everything the ledger keeps between frames.
struct App {
    /// The component's state: the row axis, the horizontal offset, the cell selection and the
    /// column half of the editing slot.
    table: TableState,
    /// The declared columns, rebuilt only when `Alt+1`/`Alt+2` change the shape.
    cols: Vec<Column>,
    /// The **key** of the column the cell cursor is in. Never its position.
    cursor_col: u16,
    /// The revision the order is at. Stamped here, compared once a frame inside the component.
    rev: Revision,
    /// Whether the order is reversed. The whole sort, because the order is a function.
    reversed: bool,
    /// Whether a header row is drawn.
    header: bool,
    /// Whether the component draws through a [`Tally`] so the counters can be printed.
    counters: bool,
    /// The last counted frame, `(writes, distinct, verbs)`.
    counted: (u64, u64, u64),
    /// The window the last solve admitted, `(lo, hi, declared)`.
    window: (usize, usize, usize),
    /// The editor's buffer, while one is open.
    editing: Option<String>,
    /// What the footer sums, recomputed when the selection changes.
    sum: i64,
    /// What the last gesture was, for the status line.
    last: &'static str,
    /// Set by `Alt+q`.
    exit: bool,
}

impl App {
    fn new() -> App {
        App {
            table: TableState::new(),
            cols: columns(),
            cursor_col: 9,
            rev: Revision::fresh(),
            reversed: false,
            header: true,
            counters: true,
            counted: (0, 0, 0),
            window: (0, 0, 12),
            editing: None,
            sum: 0,
            last: "—",
            exit: false,
        }
    }

    /// The source row a display row shows. The whole of the sort, and it is arithmetic.
    fn source(&self, display: usize) -> usize {
        match self.reversed {
            true => ENTRIES - 1 - display,
            false => display,
        }
    }

    /// Where the cursor's column sits in the declaration list.
    fn cursor_slot(&self) -> usize {
        self.cols
            .iter()
            .position(|c| c.key == self.cursor_col)
            .unwrap_or(0)
    }

    /// Move the column cursor `by` columns and scroll the band so it is visible.
    fn move_column(&mut self, area_w: u16, by: isize) {
        let at = self.cursor_slot() as isize + by;
        let at = at.clamp(0, self.cols.len() as isize - 1) as usize;
        self.cursor_col = self.cols[at].key;

        // **Keeping it visible is the caller's**, and it is three lines because the solve is
        // public: a component that scrolled the band on the application's behalf would be a
        // component with an opinion about where the cursor should be.
        let solved = solve_columns(area_w, &self.cols);
        for slot in solved.scroll.0..solved.scroll.1 {
            if self.cols[usize::from(solved.spec[slot])].key != self.cursor_col {
                continue;
            }
            let (x, w) = (solved.x[slot], i32::from(solved.w[slot]));
            let view = i32::from(solved.view_w);
            if x < self.table.hoff {
                self.table.hoff = x;
            } else if x + w > self.table.hoff + view {
                self.table.hoff = x + w - view;
            }
        }
    }

    /// Recompute what the footer sums: the selected cells of the cursor's column.
    ///
    /// Over the **spans** and not over the rows, which is the store's shape showing through: a
    /// whole-column selection is one span, so this is one loop of one iteration however many
    /// million cells it names.
    fn resum(&mut self) {
        let key = self.cursor_col;
        self.sum = match self.table.cells.column(key) {
            None => 0,
            Some(sel) => sel
                .spans()
                .iter()
                .flat_map(|s| s.start..s.end)
                .filter_map(|row| value(key, row))
                .sum(),
        };
    }

    /// **Reverse the order.** `carry` is the difference between ADR 0031's two answers.
    fn sort(&mut self, carry: bool) {
        self.reversed = !self.reversed;
        self.rev = Revision::fresh();
        self.last = match carry {
            true => "Alt+S — reversed, positions carried",
            false => "Alt+s — reversed, positions dropped",
        };
        if !carry {
            // Nothing to do. The component compares one `u64` next frame, does not recognise it,
            // and clears every position it holds — the cursor is clamped rather than dropped,
            // because a table with no cursor has nowhere to put the keyboard.
            return;
        }
        // **Only the caller has both orders**, so only the caller can do this. One call for the
        // editing slot, one assignment for the cursor, and one remap a span for the cells.
        let flip = |row: usize| ENTRIES - 1 - row;
        self.table.follow(|row| Some(flip(row)));
        self.table.coll.sel.lead = flip(self.table.coll.sel.lead);
        self.table.coll.sel.anchor = self.table.coll.sel.anchor.map(flip);
        let keys: Vec<u16> = self.cols.iter().map(|c| c.key).collect();
        for key in keys {
            let Some(old) = self.table.cells.column(key) else {
                continue;
            };
            let mut next = Selection::new();
            for span in old.spans() {
                // A reversal turns `[start, end)` into `[len - end, len - start)`.
                next.insert(ENTRIES - span.end, ENTRIES - span.start);
            }
            *self.table.cells.column_mut(key) = next;
        }
        self.table.coll.reconciled(Rows::new(ENTRIES, self.rev));
        self.resum();
    }

    /// One frame, top to bottom.
    fn ui(&mut self, cx: &mut Ctx<'_, '_>) {
        let outer = panel_with(
            cx,
            cx.area(),
            " Ledger — 1 000 000 entries ",
            &PanelOpts {
                padded: false,
                ..Default::default()
            },
        );
        let [body, status, help] =
            Col::new().split(outer.interior, [Weight(1), Fixed(1), Fixed(1)]);

        let table_id = self.draw_table(cx, body);
        self.draw_status(cx, status);
        self.draw_help(cx, help);

        // **The editor takes the keyboard while it is open**, which is what stops a keystroke meant
        // for a cell going into the table's type-ahead buffer. The two sinks are two ids and the
        // focus is the thing that chooses between them.
        let editor = cx.id();
        let _ = cx.interact(editor, Rect::new(0, 0, 0, 0), Interest::FOCUS);
        match self.editing.is_some() {
            true => {
                if !cx.is_focused(editor) {
                    cx.focus(editor);
                }
                self.type_into_cell(cx, editor);
            }
            false => {
                // Architecture issue 25's rule, with the wrinkle a component that mints its own id
                // adds: the id is only knowable after the draw, so this is the last statement
                // rather than the first.
                if cx.focused().is_none() || cx.is_focused(editor) {
                    cx.focus(table_id);
                }
            }
        }
    }

    /// The table, drawn through `Direct` or through a [`Tally`] — **the same code path**.
    ///
    /// The two arms differ in one value, which is [`vitui_components::ink`]'s whole argument: a
    /// second implementation written against the recorder would be a recorder measuring a copy.
    fn draw_table(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) -> vitui_runtime::Id {
        let solved = solve_columns(area.w, &self.cols);
        let (lo, hi) = visible_columns(&solved, self.table.hoff);
        self.window = (lo - solved.scroll.0, hi - solved.scroll.0, self.cols.len());

        let opts = TableOpts {
            header: self.header,
            ..Default::default()
        };
        let rows = Rows::new(ENTRIES, self.rev);

        // Everything the cell drawer reads, taken out of `self` before the closure — `table_into`
        // borrows `self.table` for the whole draw, so the drawer cannot reach through it.
        //
        // **The clone is the application's and not the component's**, and it is the one allocation
        // a frame here: `CellSel` is one span list per column with an entry, so a whole-column
        // selection over a million rows is twelve entries of one span. Taking it out with
        // `mem::take` would be free and is wrong — the component **clears** it when the revision
        // stops matching, and a value taken out before the draw would be put back after it.
        let look = Look {
            cells: self.table.cells.clone(),
            editing: self.editing.clone(),
            reversed: self.reversed,
            cursor_col: self.cursor_col,
            cursor_row: self.table.coll.sel.lead,
        };

        let mut find = |_: &str, _: std::ops::Range<usize>| None;
        if self.counters {
            let mut tally = Tally::new();
            let resp = table_into(
                &mut tally,
                cx,
                area,
                &mut self.table,
                &opts,
                &self.cols,
                rows,
                &mut find,
                |ink: &mut Tally, cx: &mut Ctx<'_, '_>, r: Rect, c: Cell, f: Face| {
                    paint_cell(ink, cx, r, c, f, &look);
                },
            );
            self.counted = (tally.writes(), tally.distinct(), tally.verbs());
            resp.id
        } else {
            self.counted = (0, 0, 0);
            let resp = table_into(
                &mut Direct,
                cx,
                area,
                &mut self.table,
                &opts,
                &self.cols,
                rows,
                &mut find,
                |ink: &mut Direct, cx: &mut Ctx<'_, '_>, r: Rect, c: Cell, f: Face| {
                    paint_cell(ink, cx, r, c, f, &look);
                },
            );
            resp.id
        }
    }

    /// The cell cursor, the selection, the sum and the revision.
    fn draw_status(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let key = self.cursor_col;
        let name = self
            .cols
            .iter()
            .find(|c| c.key == key)
            .map_or("—", |c| c.title);
        let (cells, spans, bytes) = match self.table.cells.column(key) {
            None => (0, 0, 0),
            Some(sel) => (sel.count(), sel.span_count(), sel.bytes()),
        };
        let mut money = Line::default();
        // `amount` answers "" for zero, which is right in a cell and wrong in a total.
        let sum = match self.sum {
            0 => "0.00",
            v => amount(&mut money, v),
        };
        // **`last` first, because it is the feedback.** At eighty columns everything after the
        // first sixty characters is cut, and a status line whose most recent news is the part that
        // gets truncated is a status line nobody reads.
        let line = format!(
            " {}  ·  cell ({}, {name})  ·  selected {cells} in {spans} span(s), {bytes} B  ·  \
             Σ {sum}  ·  rev {}  ·  editing {:?} ",
            self.last,
            self.table.coll.sel.lead,
            self.rev.raw(),
            self.table.editing(),
        );
        fit_with(
            cx,
            area,
            &line,
            &FitOpts {
                justify: Justify::Start,
                role: Role::Dim,
                pad: Role::Dim,
            },
        );
    }

    /// The counters, the column window and the offset.
    fn draw_help(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let (w, d, v) = self.counted;
        let counted = match self.counters {
            true => format!("{w} cells / {d} distinct / {v} verbs"),
            false => "counters off (Alt+v)".to_owned(),
        };
        let line = format!(
            " {counted}  ·  cols {}..{} of {}  ·  hoff {}  ·  1/2 shape · ,/. column · \
             c/r/x/z select · Enter edit · s/S sort · v counters · h header · q quit ",
            self.window.0, self.window.1, self.window.2, self.table.hoff,
        );
        fit_with(
            cx,
            area,
            &line,
            &FitOpts {
                justify: Justify::Start,
                role: Role::Dim,
                pad: Role::Dim,
            },
        );
    }

    /// The editor's keys, drained from the sink that holds the focus while one is open.
    fn type_into_cell(&mut self, cx: &mut Ctx<'_, '_>, editor: vitui_runtime::Id) {
        while let Some(key) = cx.next_key(editor) {
            let Some(edit) = self.editing.as_mut() else {
                cx.decline(key);
                continue;
            };
            match key.code {
                Code::Escape => {
                    self.editing = None;
                    self.table.stop_editing();
                    self.last = "Esc — edit cancelled";
                }
                Code::Enter => {
                    self.editing = None;
                    self.table.stop_editing();
                    self.last = "Enter — edit committed (and discarded: the ledger is a function)";
                }
                Code::Backspace => {
                    edit.pop();
                }
                _ => {
                    let mut typed = String::new();
                    if vitui_components::keys::text(&key, &mut typed) {
                        edit.push_str(&typed);
                    } else {
                        cx.decline(key);
                    }
                }
            }
        }
    }

    /// **The application's own keys, read from what nothing wanted.**
    ///
    /// `crate::triage`'s arrangement: a component drains the keys addressed to it and **declines**
    /// what it does not want, so an application key is one that has come all the way back out.
    ///
    /// # Which keys are available here is a fact about this application, not about `table`
    ///
    /// A plain letter reaches this function only because the search handed to the component always
    /// answers `None`: a ledger over a million computed rows has nothing to type-ahead *to*, so
    /// every letter is declined. Give the table a real search and `c`, `r`, `s`, `v`, `q` and the
    /// rest stop arriving — the buffer would claim them — and every binding here would have to
    /// become a chord. `triage` is that application, and it is why `q` is the only plain key it
    /// has.
    ///
    /// The arrows are the exception in the other direction: `nav::step` takes `Left` and `Right` as
    /// cursor moves whatever the search does, so the **column** cursor is `Alt+←/→` and there is no
    /// spelling that would make it plain.
    fn take_unhandled(&mut self, keys: &[Pressed], area_w: u16) {
        for key in keys {
            let alt = key.mods.alt();
            if key.mods.ctrl() && key.code == Code::Char('q') {
                self.exit = true;
                continue;
            }
            match (alt, key.code) {
                (_, Code::Enter) => {
                    let row = self.table.coll.sel.lead;
                    self.table.edit(row, self.cursor_col);
                    let mut buf = Line::default();
                    self.editing =
                        Some(cell_text(&mut buf, self.cursor_col, self.source(row)).to_owned());
                    self.last = "Enter — editing";
                }
                // **Four spellings of one gesture, and the plain pair is the one that works.**
                // `,`/`.` and `<`/`>` need no terminal configuration; `Alt+←/→` is the spelling a
                // reader expects and is **off by default on macOS** — Ghostty's
                // `macos-option-as-alt` is unset unless you set it, so Option composes `é` rather
                // than reporting Alt, and the chord never arrives. Both are bound rather than one:
                // an application that needs a terminal setting to be usable is an application with
                // a defect, and an application that will not answer the obvious chord for someone
                // who *has* set it is the same defect from the other side.
                (true, Code::Left) | (_, Code::Char(',' | '<')) => {
                    self.move_column(area_w, -1);
                    self.last = ", — column left";
                }
                (true, Code::Right) | (_, Code::Char('.' | '>')) => {
                    self.move_column(area_w, 1);
                    self.last = ". — column right";
                }
                (_, Code::Char('[')) => {
                    self.table.hoff = self.table.hoff.saturating_sub(8);
                    self.last = "[ — band left";
                }
                (_, Code::Char(']')) => {
                    self.table.hoff = self.table.hoff.saturating_add(8);
                    self.last = "] — band right";
                }
                (_, Code::Char('x')) => {
                    let row = self.source(self.table.coll.sel.lead);
                    self.table.cells.column_mut(self.cursor_col).toggle(row);
                    self.resum();
                    self.last = "x — cell toggled";
                }
                (_, Code::Char('c')) => {
                    self.table.cells.select_column(self.cursor_col, ENTRIES);
                    self.resum();
                    self.last = "c — whole column: one span at a million rows";
                }
                (_, Code::Char('r')) => {
                    let keys: Vec<u16> = self.cols.iter().map(|c| c.key).collect();
                    let row = self.source(self.table.coll.sel.lead);
                    self.table.cells.select_row(&keys, row);
                    self.resum();
                    self.last = "r — whole row: one span per column, and that is the loss";
                }
                (_, Code::Char('z')) => {
                    self.table.cells.clear();
                    self.resum();
                    self.last = "z — cleared";
                }
                (_, Code::Char('s')) => self.sort(false),
                (_, Code::Char('S')) => self.sort(true),
                (_, Code::Char('1')) => {
                    self.cols = one_column();
                    self.cursor_col = 0;
                    self.table.hoff = 0;
                    self.last = "1 — one column";
                }
                (_, Code::Char('2')) => {
                    self.cols = columns();
                    self.cursor_col = 9;
                    self.last = "2 — twelve columns";
                }
                (_, Code::Char('v')) => {
                    self.counters = !self.counters;
                    self.last = "v — counters";
                }
                (_, Code::Char('h')) => {
                    self.header = !self.header;
                    self.last = "h — header";
                }
                // **`Ctrl+Q` beside `q`, and both are bound** — the same arrangement as the four
                // spellings above and for a sharper reason. A focused `collection` consumes every
                // text-bearing key into its type-ahead buffer (spec §5), so `q` reaches this
                // function only on a frame where nothing is focused: it is racy, it always was, and
                // it read as working because the old loop happened to read the window of the frame
                // *before* the focus was seated. `Ctrl` is not text (`crate::keys::text` excludes
                // it), so the chord is declined all the way out every time.
                (_, Code::Char('q')) => self.exit = true,
                _ => {}
            }
        }
    }
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

    // The first frame is drawn before the first park, for `counter`'s reason: parking with nothing
    // pending is indefinite by design and a screen that appears on the first keystroke is a bug.
    driver.frame(|cx| app.ui(cx));

    loop {
        driver.frame(|cx| app.ui(cx));
        // **What nothing wanted, read from the frame that has just drawn.**
        //
        // `Driver::unhandled` is *a window onto the same queue, valid until the next frame begins*,
        // so reading it **before** this application's frame read the previous frame's window and
        // acted one wake late — which for a single keystroke means never, because nothing wakes it
        // again. Measured on the shipped binary: `q` did not quit. Components ticket 22's
        // application found it and every loop in this crate had it. A key this application owns
        // still cannot be read inside the draw — see `App::take_unhandled`; what moved is *which*
        // frame's window is read.
        let width = driver.size().0;
        let unhandled: Vec<_> = driver.unhandled().to_vec();
        app.take_unhandled(&unhandled, width);
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
