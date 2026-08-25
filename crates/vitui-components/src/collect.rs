//! **F7 collections**, ~50 entries, expressed by [`collection`] plus columns, an index or tiles.
//!
//! The reduction is R1 and R5 (spec §18). R5 is this family's own and it is the sharpest of the
//! six: twenty-one of the survey's twenty-two data-grid features are caller state or layout, and
//! the twenty-second — variable row height — is a fourth field on §7's record. **The index is the
//! caller's**, which is what makes R5 a reduction rather than a deferral: an entry in that class
//! needs no library mechanism at all, only a documented shape.
//!
//! Org charts, mind maps and pivot tables are layout research, four families away from a mechanism
//! (§22).
//!
//! # `collection` — one component, one [`Mode`], thirteen match arms
//!
//! Components ticket 12. Spec §5, ADR 0028. `list`, option list, menu, multi-select, tabs, radio
//! group and segmented control are **one component and one `Mode`**, and the whole difference
//! between a radio group and a file manager is the thirteen arms of [`apply`] — counted by opening
//! this file, because a constant naming its own arm count is a number nothing evaluates
//! ([`ARMS`], [`arms_in_apply`]).
//!
//! ## Three facts, not one
//!
//! The store is the **cursor** ([`Selection::lead`]), the **anchor** ([`Selection::anchor`]) and
//! **what is selected** ([`Selection::spans`]). A menu keeps only the first; options and
//! single-select keep the third at one element; multi-select uses all three.
//!
//! **What is selected is a sorted, disjoint [`Span`] list**, and the property that decides it is
//! that *select-all is one span whatever the length* while every other gesture adds at most one —
//! so the store is proportional to the number of gestures and never to the number of rows.
//! [`stores`] carries the three that were measured and refused.
//!
//! **The word is `Span` and not `Run`, and that is `CONTEXT.md`'s ruling rather than a preference**:
//! *`Run` is the engine's word and is not available for anything else — a contiguous interval of
//! selected indices in a collection is a `Span`, never a run, and the components map's "run list" is
//! a span list.* Two meanings of one word, both carrying measurements, is the collision that
//! glossary exists to prevent.
//!
//! **And the word collides anyway, one module over.** [`crate::scroll::Span`] is components ticket
//! 07's *viewport / extent / offset* triple, minted after `CONTEXT.md` had already spent the word
//! here. Nothing is renamed from this ticket — a resolved ticket's vocabulary is not edited from a
//! later one — and the collision is recorded as a finding rather than settled. The practical
//! consequence is that this module never imports `scroll::Span`, so no line in the crate can read
//! *expected `Span`, found `Span`*.
//!
//! ## One hit entry per collection
//!
//! `Response::local` is computed inside the runtime's `declare` from *this frame's* pointer, unlike
//! `hovered` — so resolving the row by arithmetic (`offset + local.1`) gives per-row hover with
//! **no frame lag and no per-row index entry**. [`collection`] therefore declares **exactly one**
//! hit entry however many rows it stands. A row with a target of its own declares it, per target,
//! for visible rows only, and that is the row drawer's call rather than the component's.
//!
//! ## Identity: the row loop is wrapped, and `#[track_caller]` is not enough
//!
//! ADR 0027. `Ctx::scroll_scope` roots no identity, deliberately, so rows drawn through a line
//! *inside* this function would derive their ids from `(screen, key, that line)` — identical in
//! every collection on the screen, first claimant wins, and the rest inert. The fix is one call and
//! it is [`Ctx::with_id`](vitui_runtime::Ctx::with_id) around the row loop, taking the collection's
//! own id. **Marking this function `#[track_caller]` does not do it**: the attribute does not cross
//! a closure, so `Location::caller()` evaluated inside the row body is the line in `collection`.
//! [`defective::unkeyed_rows`] is that build, kept runnable, and `merges` is the counter that sees
//! it.
//!
//! ## Per-row state is one slot, never a map
//!
//! > **Per-row state is either derived from the row's data, or it is one slot on the collection
//! > naming the row that has it.**
//!
//! [`CollState`] is offset, selection, type-ahead buffer and `editing: Option<usize>`, because only
//! one row can hold an inline editor. Nothing keyed by row index may exist, and the runtime enforces
//! the other half: a row that did not draw cannot be clicked, focused or hovered (ADR 0012), so
//! state for an undrawn row is state nothing can reach. [`COLL_STATE_BYTES`] is the measured size
//! and it does **not** reproduce §5's 208 — see that constant, which says why rather than padding
//! the type to fit.
//!
//! ## The row signature is `(cx, rect, index, Face)` and no `Sel` enum exists
//!
//! Five independent bits, [`crate::frame::Face`], collapsed by [`crate::frame::face_paint`]. C02's
//! four-variant enum names 4 of the 32 states, and the two it cannot say at all are *selected and
//! hovered* and *the cursor without the selection* — which is what `Ctrl+↓` does and what every file
//! manager draws.
//!
//! ## The pointer half is not keyboard-only any more
//!
//! Spec §5 records ctrl-click and shift-click as **inexpressible**, because `rt::Input` was `Move`,
//! `Down`, `Up`, `Wheel` and `Key` and only `Key` carried a modifier byte. That is no longer true:
//! the engine reports modifiers on every pointer event and `Response::mods` carries them (runtime
//! 10), so ctrl-click and shift-click route through the **same** [`apply`] as `Space` and
//! `Shift+↑/↓`. **No component reads a modifier from anywhere else** — [`from_click`] is the one
//! function that turns a pointer press into a [`Gesture`] and it takes a `Response`, and
//! [`from_key`] is its keyboard twin.
//!
//! ## What the component draws, and what it does not
//!
//! The rows the content admits belong to the **row drawer**, which owes each one the rectangle it
//! is handed (§2, ADR 0026). Every other cell of the collection's rectangle is the collection's own
//! and it writes them: that is the tail below the last content row, and writing it is the whole of
//! the *stale tail* axis — 71 of 80 rows on `crate::listing`'s screen when it is left out.
//!
//! ## The reveal is conditional, and this module does not contain the defect
//!
//! `CONTEXT.md`: a scroll-into-view *fires only for a keyboard-driven focus move; a press already
//! proves the widget was on screen, and an unconditional pull fights the wheel.* [`collection`] asks
//! for a reveal exactly when a **key** moved the cursor, and [`defective::every_frame`] is the arm
//! four resolved tickets shipped. Components 20 owns the gate; what this module owes it is a
//! subject that is on the right side of it.

use std::ops::Range;
use std::time::Instant;

use vitui_runtime::keys::{Code, Pressed};
use vitui_runtime::{Ctx, Id, Interest, Mods, Rect, Response, Revision, Role, Scrollable};

use crate::frame::Face;
use crate::ink::{Direct, Ink};
use crate::nav::{self, Cursor, TypeAhead};
use crate::order::Rows;

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &["collection", "table", "tree", "pagination"];

// ── the span list ────────────────────────────────────────────────────────────────────────────────

/// **A contiguous interval of selected indices, half-open.**
///
/// `CONTEXT.md` mints the word: *a contiguous interval of selected indices in a collection is a
/// `Span`, never a run.* The engine owns `Run` and it is one damaged span on one row.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct Span {
    /// The first selected index.
    pub start: usize,
    /// One past the last selected index.
    pub end: usize,
}

impl Span {
    /// How many indices it covers.
    pub const fn len(self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Whether it covers none.
    pub const fn is_empty(self) -> bool {
        self.end <= self.start
    }
}

/// **How many bytes one [`Span`] costs.** Two `usize`, which is the sixteen in *`Ctrl+A` is 16 B*.
pub const SPAN_BYTES: usize = size_of::<Span>();

/// **What is selected, as a sorted, disjoint, non-adjacent [`Span`] list — plus the other two
/// facts.**
///
/// Three facts and not one, because the survey's families use different subsets: a menu keeps only
/// [`Selection::lead`], options and single-select keep one selected element, and multi-select uses
/// all three.
///
/// # The property that decides the store
///
/// *Select-all is one span whatever the length*, and every other gesture adds at most one — so the
/// store is proportional to the number of gestures the user has made and never to the number of
/// rows. The three alternatives are in [`stores`], and one of them is a keystroke at 234× the whole
/// frame budget.
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct Selection {
    spans: Vec<Span>,
    /// **Where the keyboard is.** Always meaningful, even when nothing is selected — which is what
    /// a menu is made of and what `Ctrl+↓` moves.
    pub lead: usize,
    /// **Where the last range gesture started.** `None` until the first plain gesture.
    pub anchor: Option<usize>,
}

impl Selection {
    /// An empty selection with its cursor at the top.
    pub const fn new() -> Selection {
        Selection {
            spans: Vec::new(),
            lead: 0,
            anchor: None,
        }
    }

    /// The spans, in ascending order and never touching.
    pub fn spans(&self) -> &[Span] {
        &self.spans
    }

    /// How many spans the store holds. **The number the whole choice of store is about.**
    pub fn span_count(&self) -> usize {
        self.spans.len()
    }

    /// What the store costs, in bytes of span. `span_count * `[`SPAN_BYTES`].
    pub fn bytes(&self) -> usize {
        self.spans.len() * SPAN_BYTES
    }

    /// How many indices are selected. **`O(spans)`, never `O(len)`** — which is why a status bar
    /// may print it every frame.
    pub fn count(&self) -> usize {
        self.spans.iter().map(|s| s.len()).sum()
    }

    /// Whether nothing is selected.
    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    /// Forget the selection, keeping the cursor and the anchor.
    pub fn clear(&mut self) {
        self.spans.clear();
    }

    /// **One random-access test, `O(log spans)`.**
    ///
    /// The row loop does **not** use this: it seeks a [`Scan`] once and advances it in lockstep,
    /// which is `O(log k + h)` against this one's `O(h · log k)`.
    pub fn contains(&self, i: usize) -> bool {
        let k = self.spans.partition_point(|s| s.end <= i);
        self.spans.get(k).is_some_and(|s| s.start <= i)
    }

    /// **The whole reason this store was chosen: one push, at any length.**
    pub fn select_all(&mut self, len: usize) {
        self.spans.clear();
        if len > 0 {
            self.spans.push(Span { start: 0, end: len });
        }
    }

    /// Replace the selection with `i` alone, and move both the cursor and the anchor to it.
    pub fn select_only(&mut self, i: usize) {
        self.spans.clear();
        self.spans.push(Span {
            start: i,
            end: i + 1,
        });
        self.lead = i;
        self.anchor = Some(i);
    }

    /// Add or remove `i`, and move both the cursor and the anchor to it.
    pub fn toggle(&mut self, i: usize) {
        if self.contains(i) {
            self.remove(i, i + 1);
        } else {
            self.insert(i, i + 1);
        }
        self.lead = i;
        self.anchor = Some(i);
    }

    /// Replace the selection with the interval from the anchor to `i` — plain shift-click, and
    /// `Shift+↑/↓`.
    pub fn extend_to(&mut self, i: usize) {
        let (lo, hi) = self.range_from_anchor(i);
        self.spans.clear();
        self.spans.push(Span {
            start: lo,
            end: hi + 1,
        });
        self.lead = i;
    }

    /// Add the interval from the anchor to `i` to what is already selected — `Ctrl+Shift`-click.
    pub fn extend_add_to(&mut self, i: usize) {
        let (lo, hi) = self.range_from_anchor(i);
        self.insert(lo, hi + 1);
        self.lead = i;
    }

    fn range_from_anchor(&self, i: usize) -> (usize, usize) {
        let a = self.anchor.unwrap_or(i);
        if a <= i { (a, i) } else { (i, a) }
    }

    /// **Insert `[start, end)`, coalescing with every span it touches. `O(spans)`.**
    ///
    /// Adjacency counts, which is what keeps the list non-adjacent: `[0, 3)` beside `[3, 5)` is one
    /// span and not two, so a user holding `Shift+↓` adds a span on the first press and none after.
    pub fn insert(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }
        let first = self.spans.partition_point(|s| s.end < start);
        let (mut lo, mut hi) = (start, end);
        let mut last = first;
        while last < self.spans.len() && self.spans[last].start <= hi {
            lo = lo.min(self.spans[last].start);
            hi = hi.max(self.spans[last].end);
            last += 1;
        }
        self.spans
            .splice(first..last, std::iter::once(Span { start: lo, end: hi }));
    }

    /// **Replace the whole span list. The one door [`crate::order`]'s reconcile policies write
    /// through**, and it is `pub(crate)` for that reason: a caller who could set the spans directly
    /// could set an unsorted, overlapping or adjacent one, and every `partition_point` in this
    /// module would then be wrong about a store that still looks like a store.
    ///
    /// `crate::order::reconcile_splice` and `reconcile_permutation` both coalesce before they call
    /// it, which is where the invariant is actually kept.
    pub(crate) fn set_spans(&mut self, spans: Vec<Span>) {
        self.spans = spans;
    }

    /// **Remove `[start, end)`, splitting the span it lands inside. `O(spans)`.**
    ///
    /// At most two spans survive from the interval that was touched — a prefix and a suffix — which
    /// is why the replacement is a fixed-size array rather than a second `Vec`.
    pub fn remove(&mut self, start: usize, end: usize) {
        if start >= end {
            return;
        }
        let first = self.spans.partition_point(|s| s.end <= start);
        let mut kept = [Span { start: 0, end: 0 }; 2];
        let mut n = 0usize;
        let mut last = first;
        while last < self.spans.len() && self.spans[last].start < end {
            let s = self.spans[last];
            if s.start < start {
                kept[n] = Span {
                    start: s.start,
                    end: start,
                };
                n += 1;
            }
            if s.end > end {
                kept[n] = Span {
                    start: end,
                    end: s.end,
                };
                n += 1;
            }
            last += 1;
        }
        self.spans.splice(first..last, kept[..n].iter().copied());
    }
}

/// **The query the row loop uses: seeked once, advanced in lockstep.**
///
/// A collection draws its visible window in ascending index order, so a per-row binary search is the
/// wrong shape. This is `O(log k + h)`; [`Selection::contains`] in the loop is `O(h · log k)`.
///
/// It is written this way because it is the shape that stays right when `h` is a table with forty
/// columns and `k` is a user who has ctrl-clicked for a minute — not because the difference is large
/// at the sizes a terminal reaches. [`crate::collect::seek_and_probes`] is what makes that a count
/// rather than a claim: the scan takes one `partition_point` a frame and the naive form takes one a
/// row.
pub struct Scan<'a> {
    spans: &'a [Span],
    i: usize,
}

impl<'a> Scan<'a> {
    /// Seek to the first span that could contain `from`. **One `partition_point`, once a frame.**
    pub fn seek(sel: &'a Selection, from: usize) -> Scan<'a> {
        Scan {
            i: sel.spans.partition_point(|s| s.end <= from),
            spans: &sel.spans,
        }
    }

    /// Whether `row` is selected. `row` must be non-decreasing between calls.
    pub fn at(&mut self, row: usize) -> bool {
        while self.i < self.spans.len() && self.spans[self.i].end <= row {
            self.i += 1;
        }
        self.spans.get(self.i).is_some_and(|s| s.start <= row)
    }
}

// ── the mode, the gesture, and the thirteen arms ─────────────────────────────────────────────────

/// **The policy, and the whole of what separates seven components from one.**
///
/// Four values, and [`MODES`] is the count a gate asserts. Every one of the seven names §5 collapses
/// is one of these plus the caller's own row drawer — see [`ABSORBED`], which no row of
/// [`crate::INVENTORY`] may duplicate.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub enum Mode {
    /// **A menu, a submenu, a context menu.** A cursor and a commit, and nothing is ever selected:
    /// highlighting the row the keyboard is on is `Face::cursor`, not `Face::selected`, which is
    /// why a menu needs no selection store at all.
    Cursor,
    /// **A list.** At most one row selected, and it can become zero — `Escape` clears, and so does
    /// `Space` on the row that is already the only one. The default.
    #[default]
    Single,
    /// **A radio set, a tab strip, a segmented control.** Exactly one, and it can never become zero.
    /// The *only* difference from [`Mode::Single`] is that one branch, which is §5's own sentence.
    Options,
    /// **Multi-select.** Click, ctrl-click, shift-click, `Space`, `Shift+↑/↓`, `Ctrl+A`, `Escape`.
    Multi,
}

impl Mode {
    /// All four, for a gate that iterates rather than samples.
    pub const ALL: [Mode; 4] = [Mode::Cursor, Mode::Single, Mode::Options, Mode::Multi];

    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Mode::Cursor => "cursor",
            Mode::Single => "single",
            Mode::Options => "options",
            Mode::Multi => "multi",
        }
    }
}

/// **How many modes there are. Four.** The count a gate asserts, beside [`ARMS`].
pub const MODES: usize = Mode::ALL.len();

/// **The seven component names §5 collapses into [`collection`] plus a [`Mode`].**
///
/// The other half of criterion 1: *no second component in `INVENTORY` duplicates one of them.* Six
/// inventory entries collapsing into one is most of why the v1 freeze is twenty-nine components
/// rather than thirty-five (ADR 0028), and a row reappearing under one of these names is that
/// collapse being quietly undone.
///
/// `radio` **is** an inventory row and is deliberately not here: standalone it is a `press`, and a
/// radio *set* is `collection` at [`Mode::Options`] — the row is the widget and the set is a
/// different call, which is why `crate::INVENTORY`'s `radio` carries no `COMPOSITIONS` edge.
pub const ABSORBED: [&str; 7] = [
    "list",
    "option_list",
    "menu",
    "multi_select",
    "tabs",
    "radio_group",
    "segmented_control",
];

/// **What a gesture asked for, before the [`Mode`] has had its say.**
///
/// Six, and the pointer and the keyboard produce the same six: [`from_click`] and [`from_key`] are
/// two readings of one vocabulary, which is what *ctrl-click and shift-click route through the same
/// `apply` as their keyboard twins* means as a type.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Gesture {
    /// Click, or an unmodified arrow: select this and nothing else.
    Plain(usize),
    /// Ctrl-click, or `Space`: add or remove this one.
    Toggle(usize),
    /// Shift-click, or `Shift+↑/↓`: replace the selection with the interval from the anchor.
    Extend(usize),
    /// `Ctrl+Shift`-click: add the interval from the anchor to what is already selected.
    ExtendAdd(usize),
    /// `Ctrl+A`.
    All,
    /// `Escape`.
    Nothing,
}

impl Gesture {
    /// Which index the gesture names, when it names one.
    pub const fn at(self) -> Option<usize> {
        match self {
            Gesture::Plain(i) | Gesture::Toggle(i) | Gesture::Extend(i) | Gesture::ExtendAdd(i) => {
                Some(i)
            }
            Gesture::All | Gesture::Nothing => None,
        }
    }
}

/// **The one place a gesture becomes a change to the store, and the thirteen arms are the whole
/// difference between a radio group and a file manager.**
///
/// Every family goes through it, from the pointer and from the keyboard alike. The arms are counted
/// by [`arms_in_apply`], which opens this file — an instrument is a value with a file in it, and a
/// constant that named its own arm count would be a number nothing evaluates.
///
/// Each arm is one line on purpose, so that the count is a property of the source rather than of a
/// formatter.
pub fn apply(mode: Mode, sel: &mut Selection, len: usize, g: Gesture) {
    match (mode, g) {
        (Mode::Cursor, Gesture::Plain(i)) => sel.lead = i,
        (Mode::Cursor, _) => {}
        (Mode::Single, Gesture::Plain(i) | Gesture::Toggle(i)) => single(sel, i),
        (Mode::Single, Gesture::Nothing) => sel.clear(),
        (Mode::Single, _) => {}
        (Mode::Options, Gesture::Plain(i) | Gesture::Toggle(i)) => sel.select_only(i),
        (Mode::Options, _) => {}
        (Mode::Multi, Gesture::Plain(i)) => sel.select_only(i),
        (Mode::Multi, Gesture::Toggle(i)) => sel.toggle(i),
        (Mode::Multi, Gesture::Extend(i)) => sel.extend_to(i),
        (Mode::Multi, Gesture::ExtendAdd(i)) => sel.extend_add_to(i),
        (Mode::Multi, Gesture::All) => sel.select_all(len),
        (Mode::Multi, Gesture::Nothing) => sel.clear(),
    }
}

/// [`Mode::Single`]'s one branch: selecting the row that is already the only selected row clears it.
///
/// Split out so that [`apply`]'s arms stay one line each and the count stays a property of the
/// source. It is the *difference* from [`Mode::Options`], which is the sentence §5 states.
fn single(sel: &mut Selection, i: usize) {
    if sel.contains(i) && sel.count() == 1 {
        sel.clear();
        sel.lead = i;
        sel.anchor = Some(i);
    } else {
        sel.select_only(i);
    }
}

/// **How many match arms [`apply`] has. Thirteen**, and it is read out of this file rather than
/// declared: see [`arms_in_apply`].
pub const ARMS: usize = 13;

/// **Count [`apply`]'s match arms by opening this file.**
///
/// The register's rule — *an instrument is a value with a file in it* — applied to the one number
/// §5 leads with. A constant asserting `13 == 13` is a tautology; this reads the source, so an arm
/// added or a mode collapsed fails here.
///
/// The scan starts at the `pub fn apply(` line, opens at the `match (mode, g) {` beneath it, and
/// counts every subsequent line that carries `=>` until the match closes at its own indentation. A
/// comment line is not an arm, which is the same predicate `crate::dense::declares` applies one
/// file over.
pub fn arms_in_apply() -> usize {
    let source = std::fs::read_to_string(std::path::Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/collect.rs"
    )))
    .expect("this module's own file");
    let mut lines = source
        .lines()
        .skip_while(|l| !l.starts_with("pub fn apply(mode: Mode"))
        .skip_while(|l| !l.trim_start().starts_with("match (mode, g) {"));
    let opened = lines.next().expect("the match opens");
    let indent = opened.len() - opened.trim_start().len();
    let close = format!("{}}}", " ".repeat(indent));
    lines
        .take_while(|l| *l != close)
        .filter(|l| {
            let t = l.trim_start();
            !t.starts_with("//") && t.contains("=>")
        })
        .count()
}

// ── the gestures, from both sides of one vocabulary ──────────────────────────────────────────────

/// **A pointer press as a [`Gesture`], read from `Response::mods` and from nowhere else.**
///
/// Spec §5 records this as inexpressible; runtime 10 carries `mods: Mods` on `Response` — one byte
/// on a sixteen-byte hit entry, free on the tracking level — and the engine had been reporting
/// modifiers on every pointer event all along. Dropping them was a runtime omission and not a
/// terminal limit.
///
/// `at` is resolved by the caller from `Response::local`, which is **this frame's** pointer, so
/// there is no frame lag and no per-row hit entry (ADR 0028).
pub fn from_click(mods: Mods, at: usize) -> Gesture {
    match (mods.ctrl(), mods.shift()) {
        (true, true) => Gesture::ExtendAdd(at),
        (true, false) => Gesture::Toggle(at),
        (false, true) => Gesture::Extend(at),
        (false, false) => Gesture::Plain(at),
    }
}

/// **A keypress as a [`Gesture`], and it is [`from_click`]'s twin over the same vocabulary.**
///
/// `moved` is where [`crate::nav::step`] put the cursor, or `None` when the key moved nothing.
/// Returns `None` for a key this component does not own, which the caller owes a `Ctx::decline`.
///
/// The pairs are the ones §5 measures: `Ctrl+A`, `Space`, `Shift+↑/↓`, `Ctrl+↑/↓`. **`Ctrl+↑/↓` is
/// the one that produces no gesture at all** — it moves the cursor and leaves the selection where it
/// is, which is what every file manager does and what an enum of selection states cannot say.
pub fn from_key(k: &Pressed, lead: usize, moved: Option<usize>) -> Option<Gesture> {
    if k.mods.ctrl() && k.code == Code::Char('a') {
        return Some(Gesture::All);
    }
    if k.code == Code::Escape {
        return Some(Gesture::Nothing);
    }
    if k.code == Code::Char(' ') && !k.mods.ctrl() {
        return Some(Gesture::Toggle(lead));
    }
    let to = moved?;
    match (k.mods.ctrl(), k.mods.shift()) {
        // `Ctrl+↑/↓`: the cursor moves and the selection does not. No gesture, and the caller has
        // already moved the lead.
        (true, _) => None,
        (false, true) => Some(Gesture::Extend(to)),
        (false, false) => Some(Gesture::Plain(to)),
    }
}

// ── the state ────────────────────────────────────────────────────────────────────────────────────

/// **Everything a collection keeps across frames, and every field of it is a position.**
///
/// Offset, selection, type-ahead buffer and the editing slot — and nothing keyed by a row index,
/// which is the rule §5 states and this type is the check of. [`COLL_STATE_BYTES`] is what it costs
/// and it is the same at every length.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CollState {
    /// **The first visible content row.** The application owns the offset (`CONTEXT.md`), and a
    /// collection is the application for its own rows.
    pub offset: i32,
    /// The cursor, the anchor and what is selected.
    pub sel: Selection,
    /// The type-ahead buffer and the one deadline it owes. [`crate::nav::TypeAhead`], because a
    /// second buffer would be a second place the one-second window is written down.
    pub ahead: TypeAhead,
    /// **The row that holds an inline editor, because only one row can.** ADR 0028's *per-row state
    /// is one slot, never a map*, as a field: there is nowhere here to put a value per row.
    pub editing: Option<usize>,
    /// **The revision the positions above were last reconciled at** (components 13, ADR 0031).
    ///
    /// Private, because the only two ways it may move are [`CollState::reconciled`] — a caller
    /// saying *I have carried the positions across* — and [`collection`] itself, which stamps it
    /// after clearing. A public field would make *the revision was left behind* an ordinary
    /// assignment, and leaving it behind is the whole failure §10 is about.
    rev: Revision,
}

impl Default for CollState {
    fn default() -> CollState {
        CollState {
            offset: 0,
            sel: Selection::new(),
            ahead: TypeAhead::new(),
            editing: None,
            // **`UNKNOWN` and not a fresh one**, which is what makes the first frame *not* an
            // unexplained edit: `Revision::UNKNOWN` never matches, and the comparison below is
            // guarded on `is_known()`, so a collection over a caller with no order at all is never
            // told its positions went stale.
            rev: Revision::UNKNOWN,
        }
    }
}

impl CollState {
    /// A collection at the top of its content with nothing selected.
    pub fn new() -> CollState {
        CollState::default()
    }

    /// **The revision the positions are at.**
    pub const fn revision(&self) -> Revision {
        self.rev
    }

    /// **Say the positions have been carried across to `rows`.**
    ///
    /// The caller's half of ADR 0031: it has both orders, so it is the only thing that can splice
    /// the index, reconcile the positions and stamp the revision — and this is the third of the
    /// three. Without it the next frame sees a revision it does not recognise and applies
    /// [`Policy::Clear`](crate::order::Policy::Clear), which is the honest default for an edit
    /// nobody explained.
    pub const fn reconciled(&mut self, rows: Rows) {
        self.rev = rows.rev;
    }

    /// The largest offset `len` rows admit in a viewport `h` rows tall. *Content minus viewport*,
    /// floored.
    pub fn max_offset(len: usize, h: u16) -> i32 {
        i32::try_from(len.saturating_sub(usize::from(h))).unwrap_or(i32::MAX)
    }
}

/// **What [`CollState`] costs, measured. Independent of the number of rows — the gate.**
///
/// # It is not §5's 208, and the difference is written down rather than padded away
///
/// Spec §5 and ADR 0028 both record **208 bytes**. What this type measures is
/// [`COLL_STATE_BYTES`], and the arithmetic is visible: `offset` 4 (padded to 8), `Selection` 48 —
/// a `Vec<Span>` at 24, `lead` at 8 and `Option<usize>` at 16 — `TypeAhead` 40 (a `String` at 24 and
/// an `Option<Instant>` at 16), `editing` 16, and `rev` 8. It was 112 until components ticket 13
/// added the revision, which is §10's *one `u64` compared once a frame* as a field.
///
/// The 208 belongs to **C03's prototype struct**, which is not this one: it carried the three
/// refused stores beside the shipped one ([`stores::Alt`] here, and a field there), a `Store`
/// discriminant to choose between them, and a fixed-size type-ahead buffer inline rather than the
/// crate's own [`crate::nav::TypeAhead`]. Reaching 208 from here would mean adding a field for the
/// number, which is a gate edited rather than met.
///
/// **What §5 actually gates is the invariance**, and that reproduces exactly: the type mentions no
/// length, so `size_of` cannot depend on one. `tests::the_state_is_the_same_size_at_every_length`
/// is that as a measurement over four volumes rather than as an argument about the type.
pub const COLL_STATE_BYTES: usize = size_of::<CollState>();

/// **Spec §5's remembered figure for [`COLL_STATE_BYTES`]. Recorded, not reproduced.**
pub const SPEC_COLL_STATE_BYTES: usize = 208;

// ── the options ──────────────────────────────────────────────────────────────────────────────────

/// [`collection`]'s options. Spec §1's rule 3: a `Default` struct, never a required builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CollOpts {
    /// The policy. [`Mode::Single`] by default, which is the list every application starts with.
    pub mode: Mode,
    /// **How many rows one type-ahead keystroke may look at.**
    ///
    /// The budget half of *type-ahead is the caller's search behind a budget*: the search itself is
    /// the caller's function and the bound is the component's, because the caller has no way to know
    /// what a frame costs. See [`SEARCH_BUDGET`].
    pub search: usize,
    /// The role the collection's own cells — the tail below the content — are written in.
    pub tail: Role,
}

impl Default for CollOpts {
    fn default() -> CollOpts {
        CollOpts {
            mode: Mode::default(),
            search: SEARCH_BUDGET,
            tail: Role::Body,
        }
    }
}

/// **How many rows a type-ahead keystroke may scan. 4 096.**
///
/// Fifty screens of rows at [`crate::listing::H`], and a power of two so the bound reads as a bound
/// rather than as a tuned number. A type-ahead that has not matched within fifty screens is a
/// search, and a search belongs in a field the user can see.
///
/// **The number this buys is the one §5 states**: one `z` into a focused million-row list, bounded
/// against unbounded. [`type_ahead_cost`] measures both arms and `examples/collection_numbers.rs`
/// prints them.
pub const SEARCH_BUDGET: usize = 4_096;

// ── the component ────────────────────────────────────────────────────────────────────────────────

/// **`collection` — the component the rest of this library is mostly made of.**
///
/// Spec §1's shape exactly: `fn(&mut Ctx, Rect, …) -> Response`. The row drawer's shape is §5's
/// exactly: `(cx, rect, index, Face)`, five independent bits and no `Sel` enum.
///
/// `find` is **the caller's search** and it is handed the buffer and the bounded range to look in;
/// the component owns the buffer, the deadline and the bound. `row` owes each rectangle it is handed
/// (§2); every other cell of `area` is written here.
///
/// # The two closures arrive as `&mut dyn`, and that is a signature decision
///
/// [`collection_into`] beneath it is generic, because a gate has to monomorphise over its own
/// [`Ink`]. **The shipped entry point is not**, for two reasons that point the same way: spec §1's
/// rule 1 states a component as `fn(&mut Ctx, Rect, …) -> Response` and a turbofish is not part of
/// that shape; and the crate's own scans read the *source* for `pub fn collection(` — a generic
/// spelling puts `<F, R>` between the name and the parenthesis and every one of them answers
/// *undeclared* about a component that is right there. A `&mut dyn FnMut` allocates nothing, which
/// is the only thing the frame budget has an opinion about.
///
/// ```
/// use vitui_components::collect::{CollOpts, CollState, collection};
/// use vitui_components::frame::face_paint;
/// use vitui_components::order::Rows;
/// use vitui_runtime::ctx::Driver;
///
/// let rows = ["alpha", "beta", "gamma"];
/// let mut st = CollState::new();
/// let mut driver = Driver::headless(20, 3).expect("a sink attaches");
/// driver.frame(|cx| {
///     let area = cx.area();
///     let opts = CollOpts::default();
///     let _ = collection(
///         cx,
///         area,
///         &mut st,
///         &opts,
///         Rows::of(rows.len()),
///         &mut |buf, range: std::ops::Range<usize>| {
///             range.into_iter().find(|&i| rows[i].starts_with(buf))
///         },
///         &mut |cx, r, i, face| {
///             let paint = face_paint(cx.theme(), face);
///             cx.text(r.x, r.y, rows[i], paint);
///         },
///     );
/// });
/// ```
#[track_caller]
pub fn collection(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut CollState,
    opts: &CollOpts,
    rows: Rows,
    find: &mut dyn FnMut(&str, Range<usize>) -> Option<usize>,
    row: &mut dyn FnMut(&mut Ctx<'_, '_>, Rect, usize, Face),
) -> Response {
    collection_into(
        &mut Direct,
        cx,
        area,
        st,
        opts,
        rows,
        find,
        |_ink, cx, r, i, f| row(cx, r, i, f),
    )
}

/// **[`collection`], drawing through an [`Ink`] so an instrument can see every cell.**
///
/// The entry point a gate takes; `collection` is this with [`Direct`] and a row drawer that ignores
/// the writer. See [`crate::ink`] for why the seam exists rather than a second implementation of the
/// component written against a `Tally` — *a gate written against a copy of the code tests the copy*.
///
/// The row drawer gains the writer here and only here, because §5 fixes the shipped row signature at
/// four arguments and a fifth would be a signature invented for the instrument.
#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "`collection`'s own seven plus the `Ink` seam's writer. The seven are spec §5's — a \
              context, a rectangle, the state, the options, the length, the caller's search and \
              the row drawer — and none of them can be folded into another without inventing a \
              parameter struct that exists only to satisfy a lint"
)]
pub fn collection_into<I, F, R>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut CollState,
    opts: &CollOpts,
    rows: Rows,
    mut find: F,
    mut row: R,
) -> Response
where
    I: Ink,
    F: FnMut(&str, Range<usize>) -> Option<usize>,
    R: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, usize, Face),
{
    draw_with(
        ink,
        cx,
        area,
        st,
        opts,
        rows,
        &mut find,
        &mut row,
        Shape::Virtualised,
        Reveal::WhenAsked,
    )
}

/// Which rows the body iterates, and it is the one line between [`collection`] and
/// [`defective::whole_content`].
///
/// Both are legitimate constructions and the runtime says which is which: a scroll area draws the
/// whole content and lets the clip reject what is off screen, which is right for a form and
/// **887–889×** for a million rows. Choosing the wrong one is *the single most expensive mistake
/// available above this runtime*, and it is expensive because **both compile and both look right on
/// a thousand rows**.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Shape {
    /// Ask [`Ctx::visible_rows`](vitui_runtime::Ctx::visible_rows) which rows can be reached.
    Virtualised,
    /// Iterate the content and let the clip decide. The defect.
    WholeContent,
}

/// Whether the reveal is conditional, unconditional, or gone.
///
/// Three arms and not two, because a one-directional gate goes green the moment somebody deletes the
/// call entirely — which loses the keyboard behaviour instead of fixing the pointer one. See
/// [`crate::listing::Reveal`], whose three arms these are.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Reveal {
    WhenAsked,
    EveryFrame,
    Never,
}

/// **`#[track_caller]` all the way down, and the missing attribute is a real defect this fixture
/// caught.**
///
/// `Ctx::id` mints from `Location::caller()`, and an attribute that stops one frame short of the
/// call makes every collection in the application one collection: the first run of
/// `tests::two_collections_on_one_screen_declare_two_entries_and_merge_nothing` read **9 merges on
/// the correct arm**, because `collection` and `collection_into` carried the attribute and this
/// function did not. ADR 0027's *a wrapper that forgets `#[track_caller]` merges the widgets inside
/// its own body*, arriving from the inside.
#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "the shipped signature is `collection`'s seven; this is that plus the two policy \
              values that separate the correct build from the two defective ones, and splitting it \
              would put the defect in a second function where a reviewer's diff could not be one \
              line"
)]
fn draw_with<I, F, R>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut CollState,
    opts: &CollOpts,
    rows: Rows,
    find: &mut F,
    row: &mut R,
    shape: Shape,
    reveal: Reveal,
) -> Response
where
    I: Ink,
    F: FnMut(&str, Range<usize>) -> Option<usize>,
    R: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, usize, Face),
{
    // **The id is taken here, outside every closure** (ADR 0027). `#[track_caller]` on this
    // function makes it the *caller's* line; taken inside the row body it would be the line below,
    // and every collection in the application would be one collection.
    let id = cx.id();
    let len = rows.len;

    // **One `u64`, compared once a frame** (components 13, §10, ADR 0031). Everything this
    // component stores is a *position* in an order the caller can replace between two frames, and a
    // sort changes no data and no length — so the frame after one draws a perfectly correct list
    // with the wrong rows selected and the editor open on the wrong row. The revision is the only
    // thing that makes it noticeable.
    //
    // **The answer here is `Clear` and it is the only one available**, which is not a limitation:
    // `Remap` is the caller's opt-in *because only the caller has both orders*, and `Drop`/`Stash`
    // need the interval a splice removed, which the component was not told. A caller that has
    // carried the positions across says so with `CollState::reconciled` and this branch does not
    // fire. See `crate::order`.
    // **A state at `UNKNOWN` is *adopting* an order and not noticing an edit**, which is the same
    // rule `Revision::UNKNOWN never hits` states one crate down: *I have nothing to compare against*
    // is not *the order changed*. Without it a collection whose state was restored from disk beside
    // the order it belongs to loses its selection on its first frame, and the caller's only repair
    // would be a `CollState::reconciled` call asserting something it has not done.
    if st.rev.is_known() && rows.rev.is_known() && st.rev != rows.rev {
        // **Every field cleared here is a position**, which is the whole of ADR 0031's sentence:
        // the spans, the anchor and the editing slot are positions in an order that has been
        // replaced, and the cursor is one too — it is clamped rather than dropped, because a
        // collection with no cursor at all has nowhere to put the keyboard.
        st.sel.clear();
        st.sel.anchor = None;
        st.editing = None;
        st.sel.lead = st.sel.lead.min(len.saturating_sub(1));
    }
    st.rev = rows.rev;

    let max = (0, CollState::max_offset(len, area.h));

    // **The reveal the frame before asked for**, applied by the widget that owns the offset. A
    // delta and not a position, because the component may have moved the offset in between.
    if let Some((_, dy)) = cx.take_into_view(id) {
        st.offset = (st.offset + dy).clamp(0, max.1);
    }
    st.offset = st.offset.clamp(0, max.1);

    // **One hit entry, and it is the wheel chain's entry as well.** A component that publishes a
    // scrollable region owes the pair per axis, computed from its clamped offset.
    let mut resp = cx.scrollable(
        id,
        area,
        Interest::CLICK.with(Interest::HOVER).with(Interest::FOCUS),
        Scrollable::between((0, st.offset), max),
    );

    // **This frame's wheel, read inside the draw by the widget that owns the offset.**
    st.offset = (st.offset + resp.scrolled.1).clamp(0, max.1);

    // The pointer half. `local` is this frame's, so the row is arithmetic and there is no frame lag
    // and no per-row hit entry.
    let over = resp
        .local
        .map(|(_, ly)| usize::try_from(st.offset + ly).unwrap_or(0))
        .filter(|i| *i < len);
    if resp.clicked
        && let Some(at) = over
    {
        apply(opts.mode, &mut st.sel, len, from_click(resp.mods, at));
        resp.changed = true;
    }

    // The keyboard half, and the two are one vocabulary. **A page is the viewport's height and
    // only the caller knows that** (`crate::nav::Cursor`), so the rectangle's own height is what
    // goes in rather than a constant.
    let asked = keyboard(cx, id, st, opts, len, find, usize::from(area.h.max(1)));
    resp.changed |= asked.changed;
    // **The one call ADR 0027 is about, and it is *outside* the scroll scope rather than around the
    // row loop.** Without it the rows of every collection on the screen derive the same ids and all
    // but the first are inert, on a screen that renders correctly.
    //
    // ADR 0027 says *a collection wraps its row loop in `cx.with_id(id, …)`*, and written literally
    // — inside the scope, around the `for` — it draws **nothing at all**. `Ctx::with_id` re-childs
    // the view at `self.area()`, which is `Rect::new(0, 0, w, h)` in the context's own coordinates;
    // inside a scrolled scope those coordinates are the *content's*, so `area()` names content rows
    // `0..h` and the clip it intersects with is the window at the offset. At any offset past the
    // first screenful the two do not overlap and the clip is empty. Here the id stack is pushed
    // before the scope opens, which pushes the same id over the same rows and moves no coordinate.
    cx.with_id(id, |cx| {
        cx.scroll_scope(id, area, (0, st.offset), max, |cx| {
            {
                let visible = cx.visible_rows();
                let content = 0..i32::try_from(len).unwrap_or(i32::MAX);
                let over_rows = match shape {
                    Shape::Virtualised => visible.clone(),
                    Shape::WholeContent => content.clone(),
                };
                let first = usize::try_from(over_rows.start.max(0)).unwrap_or(0);
                // **Seeked once and advanced in lockstep**, which is the whole of `O(log k + h)`.
                let mut scan = Scan::seek(&st.sel, first);
                for y in over_rows {
                    let Ok(i) = usize::try_from(y) else { continue };
                    if i >= len {
                        continue;
                    }
                    let face = Face {
                        selected: scan.at(i),
                        cursor: st.sel.lead == i,
                        active: resp.focused,
                        hovered: over == Some(i),
                        disabled: false,
                    };
                    row(ink, cx, Rect::new(0, y, area.w, 1), i, face);
                }
                // **The tail is the collection's own, and every cell of it is written.** The rows the
                // content admits belong to the row drawer; this is the rest of the partition, and
                // leaving it out is 71 of 80 rows on `crate::listing`'s screen.
                let tail = i32::try_from(len).unwrap_or(i32::MAX).max(visible.start);
                let paint = cx.theme().paint(opts.tail);
                for y in tail..visible.end {
                    let _ = ink.run(cx, 0, y, " ", area.w, paint);
                }
                // **Only when something asked** (`CONTEXT.md`). A press already proves the row was on
                // screen, so a reveal on a click fights the wheel; `Reveal::EveryFrame` is that build
                // and it lives in `defective`.
                let ask = match reveal {
                    Reveal::WhenAsked => asked.reveal,
                    Reveal::EveryFrame => true,
                    Reveal::Never => false,
                };
                if ask {
                    let at = i32::try_from(st.sel.lead).unwrap_or(i32::MAX);
                    cx.request_into_view(Rect::new(0, at, area.w, 1));
                }
            }
        });
    });
    resp
}

/// What one frame's keys did: whether the store changed, and whether anything asked for a reveal.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct Asked {
    changed: bool,
    reveal: bool,
}

/// **Drain the keys addressed to this collection.**
///
/// `Ctx::next_key` answers nobody but the routing target, so this is the whole keyboard surface and
/// a key that is not this component's is declined rather than swallowed — which is what stops a list
/// eating `Ctrl+S`.
fn keyboard<F>(
    cx: &mut Ctx<'_, '_>,
    id: Id,
    st: &mut CollState,
    opts: &CollOpts,
    len: usize,
    find: &mut F,
    page: usize,
) -> Asked
where
    F: FnMut(&str, Range<usize>) -> Option<usize>,
{
    let mut out = Asked::default();
    while let Some(k) = cx.next_key(id) {
        let cur = Cursor {
            at: st.sel.lead,
            len,
            page,
        };
        let moved = nav::step(&k, cur);
        // `Ctrl+↑/↓` is a chord, so `nav::step` refuses it — correctly, because a chord is an
        // accelerator. The cursor-only move is this component's and is spelled here.
        let moved = moved.or_else(|| ctrl_step(&k, cur));
        if let Some(to) = moved {
            st.sel.lead = to;
            st.ahead.clear();
            out.reveal = true;
            out.changed = true;
        }
        match from_key(&k, st.sel.lead, moved) {
            Some(g) => {
                apply(opts.mode, &mut st.sel, len, g);
                out.changed = true;
            }
            None if moved.is_some() => {}
            None => {
                if let Some(to) = seek(cx, id, st, opts, len, find, &k) {
                    st.sel.lead = to;
                    apply(opts.mode, &mut st.sel, len, Gesture::Plain(to));
                    out.reveal = true;
                    out.changed = true;
                } else {
                    cx.decline(k);
                }
            }
        }
    }
    out
}

/// `Ctrl+↑/↓` and `Ctrl+Home/End`: move the cursor, leave the selection alone.
///
/// The gesture §5 measures beside `Ctrl+A`, `Space` and `Shift+↑/↓`, and the one an enum of
/// selection states cannot express at all — the keyboard cursor without the selection is what every
/// file manager draws and what C02's `Sel` had no variant for.
fn ctrl_step(k: &Pressed, cur: Cursor) -> Option<usize> {
    if !k.mods.ctrl() || k.mods.alt() {
        return None;
    }
    let last = cur.last();
    match k.code {
        Code::Up => Some(cur.at.saturating_sub(1)),
        Code::Down => Some((cur.at + 1).min(last)),
        Code::Home => Some(0),
        Code::End => Some(last),
        _ => None,
    }
}

/// **Type-ahead: the component's buffer, the component's deadline, the component's bound, and the
/// caller's search.**
///
/// The bound is the whole of the number §5 states. One `z` into a focused million-row list is
/// bounded against unbounded, and the search alone is 211× — see [`type_ahead_cost`].
fn seek<F>(
    cx: &mut Ctx<'_, '_>,
    id: Id,
    st: &mut CollState,
    opts: &CollOpts,
    len: usize,
    find: &mut F,
    k: &Pressed,
) -> Option<usize>
where
    F: FnMut(&str, Range<usize>) -> Option<usize>,
{
    let now = cx.now();
    st.ahead.expire(now);
    let mut typed = String::new();
    if !crate::keys::text(k, &mut typed) {
        return None;
    }
    for c in typed.chars() {
        st.ahead.push(c, now);
    }
    // **The one call.** A buffer is standing, so a screen with a type-ahead in it does not block
    // until the next key — the buffer would otherwise lapse at the moment its meaning was needed.
    if let Some(at) = st.ahead.deadline() {
        cx.deadline_for(id, at);
    }
    find(
        st.ahead.buffer(),
        search_range(st.sel.lead, len, opts.search),
    )
}

/// **The bounded range one type-ahead keystroke may look at.**
///
/// From the cursor forward, because a type-ahead is *find the next one* and not *find the first one*
/// — the same reading every file manager has. `budget == 0` means the whole content, which is the
/// unbounded arm [`type_ahead_cost`] prices rather than a configuration anybody should choose.
pub fn search_range(lead: usize, len: usize, budget: usize) -> Range<usize> {
    if budget == 0 {
        return 0..len;
    }
    let from = lead.min(len);
    from..from.saturating_add(budget).min(len)
}

// ── the defective builds, kept runnable ──────────────────────────────────────────────────────────

/// **The builds that are wrong, kept runnable, because a gate validated only against a correct
/// build reports zero for the same reason a broken one would.**
///
/// `pub` for [`crate::runner::defective`]'s reason: an instrument crate's fixtures are part of the
/// instrument. Each of these is [`collection`] with one value changed, so a reviewer's diff is one
/// line and every one of them **passes at least one gate the correct build passes**.
pub mod defective {
    use super::{
        CollOpts, CollState, Ctx, Face, Ink, Range, Rect, Response, Reveal, Rows, Shape, draw_with,
    };

    /// **The listing that iterates its whole content and lets the clip reject the rest.**
    ///
    /// It writes *exactly* what the windowed one writes — the engine reports a fully clipped verb as
    /// zero columns — so §21's register row 4, *writes flat 1k → 1M*, is green on it. What is not
    /// green is the hit index and the iteration count.
    #[track_caller]
    #[expect(
        clippy::too_many_arguments,
        reason = "`collection_into`'s eight exactly, because a defective arm that took a different \
                  signature would be a different function rather than the same one with one value \
                  changed"
    )]
    pub fn whole_content<I, F, R>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        st: &mut CollState,
        opts: &CollOpts,
        rows: Rows,
        mut find: F,
        mut row: R,
    ) -> Response
    where
        I: Ink,
        F: FnMut(&str, Range<usize>) -> Option<usize>,
        R: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, usize, Face),
    {
        draw_with(
            ink,
            cx,
            area,
            st,
            opts,
            rows,
            &mut find,
            &mut row,
            Shape::WholeContent,
            Reveal::WhenAsked,
        )
    }

    /// **The unconditional `scroll_into_view`**, which `CONTEXT.md` forbids and four *resolved*
    /// tickets wrote anyway, each by someone who had read the rule.
    ///
    /// It drags the viewport back to the cursor on every frame, so the wheel is dead. Components 20
    /// owns the gate; this is the arm it fires against, and [`never_reveals`] is the other one.
    #[track_caller]
    #[expect(
        clippy::too_many_arguments,
        reason = "`collection_into`'s eight exactly, because a defective arm that took a different \
                  signature would be a different function rather than the same one with one value \
                  changed"
    )]
    pub fn every_frame<I, F, R>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        st: &mut CollState,
        opts: &CollOpts,
        rows: Rows,
        mut find: F,
        mut row: R,
    ) -> Response
    where
        I: Ink,
        F: FnMut(&str, Range<usize>) -> Option<usize>,
        R: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, usize, Face),
    {
        draw_with(
            ink,
            cx,
            area,
            st,
            opts,
            rows,
            &mut find,
            &mut row,
            Shape::Virtualised,
            Reveal::EveryFrame,
        )
    }

    /// **The way to pass a one-directional wheel gate, and it is not a fix**: the keyboard cursor
    /// can no longer bring anything into view.
    #[track_caller]
    #[expect(
        clippy::too_many_arguments,
        reason = "`collection_into`'s eight exactly, because a defective arm that took a different \
                  signature would be a different function rather than the same one with one value \
                  changed"
    )]
    pub fn never_reveals<I, F, R>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        st: &mut CollState,
        opts: &CollOpts,
        rows: Rows,
        mut find: F,
        mut row: R,
    ) -> Response
    where
        I: Ink,
        F: FnMut(&str, Range<usize>) -> Option<usize>,
        R: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, usize, Face),
    {
        draw_with(
            ink,
            cx,
            area,
            st,
            opts,
            rows,
            &mut find,
            &mut row,
            Shape::Virtualised,
            Reveal::Never,
        )
    }

    /// **The row loop that is not wrapped in `cx.with_id`**, which is ADR 0027's defect as a
    /// runnable build.
    ///
    /// Every row of every collection on the screen derives its id from `(screen, key, the line
    /// inside `collection`)`, so two collections collide and the second one's rows are inert. **The
    /// screen renders pixel for pixel correctly** — drawing does not consume an id — and the only
    /// thing that says so is `merges`.
    ///
    /// It is a fixture rather than a branch in `super::draw_with`, because what it reproduces is a
    /// row loop written at a *different source line*, and a boolean inside one function cannot move
    /// a line.
    ///
    /// **`#[track_caller]` is here on purpose**, so that the *collections* still get two ids and the
    /// only thing colliding is the rows. Without it two defects arrive together and the count stops
    /// naming which one it is about.
    #[track_caller]
    pub fn unkeyed_rows<I: Ink>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        st: &mut CollState,
        len: usize,
    ) {
        let id = cx.id();
        let max = (0, CollState::max_offset(len, area.h));
        let _ = cx.scrollable(
            id,
            area,
            vitui_runtime::Interest::CLICK,
            vitui_runtime::Scrollable::between((0, st.offset), max),
        );
        cx.scroll_scope(id, area, (0, st.offset), max, |cx| {
            let paint = cx.theme().paint(vitui_runtime::Role::Body);
            for y in cx.visible_rows() {
                let Ok(i) = usize::try_from(y) else { continue };
                if i >= len {
                    continue;
                }
                // **No `with_id` above this line.** `with_key` roots at whatever the enclosing stack
                // is, and `scroll_scope` pushed nothing — so the key is all the identity there is.
                cx.with_key(i as u64, |cx| {
                    let row = cx.id();
                    let _ = cx.interact(
                        row,
                        Rect::new(0, y, area.w, 1),
                        vitui_runtime::Interest::CLICK,
                    );
                    let _ = ink.run(cx, 0, y, " ", area.w, paint);
                });
            }
        });
    }
}

// ── the stores that were measured and refused ────────────────────────────────────────────────────

/// **The three stores that were measured and refused**, kept runnable so §5's table is a
/// measurement rather than a memory.
///
/// All four answer the same questions. Three of them answer at least one of them in time or space
/// proportional to the *length*, and the invariant one crate down forbids exactly that: *frame cost
/// is proportional to visible cells, never to data volume.*
pub mod stores {
    use std::collections::HashSet;

    /// Which store is under measurement.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum Store {
        /// The shipped one: a sorted, disjoint [`super::Span`] list.
        Spans,
        /// What a user writes first: the selected indices, tested with `contains`.
        VecIndices,
        /// The obvious fix for `VecIndices`' query cost.
        HashSet,
        /// One bit a row. `O(1)` query, and a store that is by construction proportional to the
        /// data.
        Bitset,
    }

    impl Store {
        /// All four, so a report iterates rather than samples.
        pub const ALL: [Store; 4] = [
            Store::Spans,
            Store::VecIndices,
            Store::HashSet,
            Store::Bitset,
        ];

        /// The word a report prints.
        pub const fn word(self) -> &'static str {
            match self {
                Store::Spans => "span list",
                Store::VecIndices => "Vec<usize>",
                Store::HashSet => "HashSet",
                Store::Bitset => "bitset",
            }
        }
    }

    /// The three refused stores, side by side, so one `select_all` can be timed on each.
    #[derive(Default)]
    pub struct Alt {
        /// The indices, in click order.
        pub vec: Vec<usize>,
        /// The same, hashed.
        pub set: HashSet<usize>,
        /// One bit a row.
        pub bits: Vec<u64>,
    }

    impl Alt {
        /// An empty set of alternatives.
        pub fn new() -> Alt {
            Alt::default()
        }

        /// Select every index of `len`. **The gesture the whole choice of store turns on.**
        pub fn select_all(&mut self, store: Store, len: usize) {
            match store {
                Store::VecIndices => {
                    self.vec.clear();
                    self.vec.extend(0..len);
                }
                Store::HashSet => {
                    self.set.clear();
                    self.set.extend(0..len);
                }
                Store::Bitset => {
                    self.bits.clear();
                    self.bits.resize(len.div_ceil(64), u64::MAX);
                }
                Store::Spans => {}
            }
        }

        /// One random-access test, so the row loop's cost is comparable across the four.
        pub fn contains(&self, store: Store, i: usize) -> bool {
            match store {
                Store::VecIndices => self.vec.binary_search(&i).is_ok(),
                Store::HashSet => self.set.contains(&i),
                Store::Bitset => self
                    .bits
                    .get(i / 64)
                    .is_some_and(|w| w & (1u64 << (i % 64)) != 0),
                Store::Spans => false,
            }
        }

        /// What the store costs in bytes.
        pub fn bytes(&self, store: Store) -> usize {
            match store {
                Store::VecIndices => self.vec.capacity() * size_of::<usize>(),
                // A `HashSet`'s table is one control byte plus one bucket per slot, and it grows at
                // 7/8 load — so the capacity is what it can hold and not what it has taken.
                Store::HashSet => self.set.capacity() * (size_of::<usize>() + 1) * 8 / 7,
                Store::Bitset => self.bits.capacity() * 8,
                Store::Spans => 0,
            }
        }
    }
}

// ── the measurements ─────────────────────────────────────────────────────────────────────────────

/// The three volumes §5 states, shared with [`crate::listing::VOLUMES`] so the two screens are
/// comparable.
pub const VOLUMES: [usize; 3] = [1_000, 100_000, 1_000_000];

/// **How many `partition_point` calls one frame's selection query costs, two ways.**
///
/// `(scan, naive)`. The scan seeks once and advances in lockstep, so it is `1` at every window
/// height; the naive form binary-searches per row, so it is the window height. That is `O(log k + h)`
/// against `O(h · log k)` as a **count** rather than as a timing, which is §21's rule: a gate is a
/// count, a ratio, an equality or a compile outcome.
pub fn seek_and_probes(sel: &Selection, window: Range<usize>) -> (usize, usize) {
    let mut scan = Scan::seek(sel, window.start);
    let mut rows = 0;
    for i in window.clone() {
        let _ = scan.at(i);
        rows += 1;
    }
    (1, rows)
}

/// **What `Ctrl+A` costs the shipped store at `len` rows**: `(spans, bytes)`.
///
/// One span whatever the length, which is [`SPAN_BYTES`] and not a function of `len`.
pub fn select_all_cost(len: usize) -> (usize, usize) {
    let mut sel = Selection::new();
    sel.select_all(len);
    (sel.span_count(), sel.bytes())
}

/// **The pathological selection §5 prices: `n` ctrl-clicks on alternate rows.**
///
/// `n` spans and `n · `[`SPAN_BYTES`] bytes, because alternate rows never coalesce — the worst case
/// the store has, and it is still proportional to the gestures rather than to the rows.
pub fn alternating(n: usize) -> Selection {
    let mut sel = Selection::new();
    for i in 0..n {
        sel.toggle(i * 2);
    }
    sel
}

/// **What one type-ahead keystroke costs, bounded against unbounded**, over `len` synthetic rows.
///
/// The search is the caller's — here a `starts_with` over a label computed from the index — and the
/// bound is the component's. Returns `(bounded, unbounded)` in nanoseconds, minimum of `rounds`.
///
/// A report and never a gate: the **ratio** is the mechanism and the nanoseconds are the weather.
pub fn type_ahead_cost(len: usize, rounds: u32) -> (u128, u128) {
    fn search(range: Range<usize>, buf: &str) -> Option<usize> {
        range.into_iter().find(|&i| label(i).starts_with(buf))
    }
    let mut bounded = u128::MAX;
    let mut unbounded = u128::MAX;
    for _ in 0..rounds.max(1) {
        let at = Instant::now();
        let hit = search(search_range(0, len, SEARCH_BUDGET), "z");
        bounded = bounded.min(at.elapsed().as_nanos());
        std::hint::black_box(hit);

        let at = Instant::now();
        let hit = search(search_range(0, len, 0), "z");
        unbounded = unbounded.min(at.elapsed().as_nanos());
        std::hint::black_box(hit);
    }
    (bounded, unbounded)
}

/// **What the frame the selection moves re-damages, in cells a verb changed.**
///
/// `(first, moved)` — the first frame paints the window, and the frame after moves the selection
/// from row `from` to row `to` and repaints exactly the rows whose [`Face`] changed.
///
/// # It is not `marked`, and the substitution is the same one components ticket 07 made
///
/// §20's `marked` is [`crate::counters::Reading::Unreachable`] and stays that way: the engine's
/// damage module is `pub(crate)` from top to bottom and `Presented` carries no count. What is
/// knowable from here is the **rule** that decides it — the engine filters a write whose value
/// equals the resident value — and that is computable at the verb boundary over a surface this
/// crate keeps ([`crate::runner::Canvas::repaints`]), carried across frames by [`Pen::over`].
///
/// [`Pen::over`]: crate::runner::Pen::over
pub fn selection_move_repaints(len: usize, from: usize, to: usize) -> (u64, u64) {
    use crate::runner::{Canvas, Pen};

    let (w, h) = (crate::listing::W, crate::listing::H);
    let mut driver = crate::runner::driver_at(w, h, vitui_runtime::Density::default());
    let mut st = CollState::new();
    st.sel.select_only(from);
    let mut canvas = Canvas::new(w, h);
    let mut first = 0;
    let mut moved = 0;
    for frame in 0..2u32 {
        if frame == 1 {
            apply(Mode::Single, &mut st.sel, len, Gesture::Plain(to));
        }
        let mut pen = Pen::over(canvas);
        driver.frame(|cx| {
            let area = cx.area();
            let _ = collection_into(
                &mut pen,
                cx,
                area,
                &mut st,
                &CollOpts::default(),
                Rows::of(len),
                |_: &str, _: Range<usize>| None,
                |ink: &mut Pen, cx: &mut Ctx<'_, '_>, r: Rect, i: usize, f: Face| {
                    let paint = crate::frame::face_paint(cx.theme(), f);
                    let written = ink.text(cx, r.x, r.y, label(i), paint);
                    let _ = ink.run(
                        cx,
                        r.x + i32::from(written),
                        r.y,
                        " ",
                        r.w.saturating_sub(written),
                        paint,
                    );
                },
            );
        });
        pen.end_frame();
        canvas = pen.into_canvas();
        let changed = canvas.take_repaints();
        match frame {
            0 => first = changed,
            _ => moved = changed,
        }
    }
    (first, moved)
}

/// The synthetic label of row `i`: a two-letter prefix that never begins with `z`.
///
/// **It never matches on purpose**, because the number §5 states is the cost of a keystroke that
/// finds nothing — which is the worst case and the only one where the bound is what stops the
/// search.
pub fn label(i: usize) -> &'static str {
    const NAMES: [&str; 8] = [
        "alpha", "bravo", "delta", "echo", "gamma", "kilo", "lima", "mike",
    ];
    NAMES[i % NAMES.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Criterion 1: one component, one `Mode`, thirteen match arms.**
    ///
    /// The arm count is read out of the source rather than declared, so an arm added or a mode
    /// collapsed fails here rather than agreeing with itself.
    #[test]
    fn one_component_one_mode_and_thirteen_match_arms() {
        assert_eq!(MODES, 4, "four policies over one store");
        assert_eq!(
            arms_in_apply(),
            ARMS,
            "`apply`'s arms are counted by opening this file, and the constant has drifted from them"
        );
        assert_eq!(ARMS, 13, "§5's own number");

        // The other half of the criterion: the seven names §5 collapses are not inventory rows.
        let ids: Vec<&str> = crate::INVENTORY.iter().map(|c| c.id).collect();
        for absorbed in ABSORBED {
            assert!(
                !ids.contains(&absorbed),
                "`{absorbed}` is an inventory row as well as a `Mode`, which is the collapse ADR \
                 0028 made being quietly undone"
            );
        }
        // And the component they collapse into is one row, built.
        let coll = crate::INVENTORY
            .iter()
            .find(|c| c.id == "collection")
            .expect("`collection` is in the freeze");
        assert!(coll.built);
    }

    /// **Criterion 2: select-all is one span whatever the length.**
    ///
    /// Asserted at four volumes including one the frame budget could not survive under any other
    /// store — and it is an equality across them rather than a threshold, because *one span* is a
    /// property of the mechanism.
    #[test]
    fn select_all_is_one_span_and_sixteen_bytes_at_every_length() {
        for len in [1_000usize, 100_000, 1_000_000, 10_000_000] {
            assert_eq!(
                select_all_cost(len),
                (1, SPAN_BYTES),
                "select-all at {len} rows is not one span"
            );
        }
        assert_eq!(SPAN_BYTES, 16, "two `usize`");

        // Every other gesture adds at most one span, which is the other half of the property.
        let mut sel = Selection::new();
        for i in 0..64usize {
            let before = sel.span_count();
            sel.toggle(i * 2);
            assert!(sel.span_count() <= before + 1);
        }
        assert_eq!(sel.span_count(), 64, "alternate rows never coalesce");
        assert_eq!(sel.count(), 64);

        // And the worst case is still proportional to the gestures.
        let path = alternating(1_000);
        assert_eq!(path.span_count(), 1_000);
        assert_eq!(path.bytes(), 16_000);
    }

    /// **Criterion 3: `size_of::<CollState>()` is independent of length.**
    ///
    /// The gate is the invariance and it holds by construction — the type mentions no length — so
    /// what this measures is that no field has been added that does. §5's 208 does not reproduce
    /// and [`COLL_STATE_BYTES`] says why rather than padding the struct.
    #[test]
    fn the_state_is_the_same_size_at_every_length() {
        let mut sizes = Vec::new();
        for len in [0usize, 1_000, 1_000_000, 100_000_000] {
            let mut st = CollState::new();
            st.sel.select_all(len);
            st.editing = Some(len.saturating_sub(1));
            sizes.push(size_of_val(&st));
        }
        assert!(
            sizes.windows(2).all(|w| w[0] == w[1]),
            "the state is not the same size at every length: {sizes:?}"
        );
        assert_eq!(sizes[0], COLL_STATE_BYTES);
        assert_ne!(
            COLL_STATE_BYTES, SPEC_COLL_STATE_BYTES,
            "if the two have converged, `COLL_STATE_BYTES`'s note about C03's prototype struct has \
             stopped being true and must be rewritten rather than left agreeing by accident"
        );
    }

    /// **Criterion 6: the scan is seeked once and advanced in lockstep.**
    ///
    /// A count and not a timing: one `partition_point` a frame against one a row, at three window
    /// heights, over a selection with a thousand spans in it.
    #[test]
    fn the_scan_cursor_seeks_once_and_the_naive_form_seeks_a_row() {
        let sel = alternating(1_000);
        for h in [10usize, 80, 400] {
            assert_eq!(
                seek_and_probes(&sel, 0..h),
                (1, h),
                "O(log k + h) against O(h · log k) at a window of {h}"
            );
        }
        // And the answers agree with the random-access form, or the cheap query is cheap and wrong.
        let mut scan = Scan::seek(&sel, 0);
        for i in 0..2_000usize {
            assert_eq!(scan.at(i), sel.contains(i), "row {i}");
        }
    }

    /// **The thirteen arms behave, and the difference between `Single` and `Options` is the one
    /// branch §5 says it is.**
    #[test]
    fn a_radio_group_and_a_file_manager_differ_by_the_arms_and_nothing_else() {
        // A menu never selects anything and its cursor moves.
        let mut menu = Selection::new();
        apply(Mode::Cursor, &mut menu, 10, Gesture::Plain(4));
        assert_eq!((menu.lead, menu.count()), (4, 0));
        apply(Mode::Cursor, &mut menu, 10, Gesture::All);
        assert_eq!(menu.count(), 0, "a menu has no selection store to fill");

        // Options can never reach zero; single-select can, and that is the one branch.
        let mut opt = Selection::new();
        apply(Mode::Options, &mut opt, 10, Gesture::Plain(2));
        apply(Mode::Options, &mut opt, 10, Gesture::Toggle(2));
        assert_eq!(opt.count(), 1, "exactly one, and it can never become zero");
        let mut one = Selection::new();
        apply(Mode::Single, &mut one, 10, Gesture::Plain(2));
        apply(Mode::Single, &mut one, 10, Gesture::Toggle(2));
        assert_eq!(one.count(), 0, "the one branch that separates the two");
        apply(Mode::Single, &mut one, 10, Gesture::Extend(7));
        assert_eq!(one.count(), 0, "a range gesture is not single-select's");

        // Multi uses all three facts.
        let mut multi = Selection::new();
        apply(Mode::Multi, &mut multi, 10, Gesture::Plain(2));
        apply(Mode::Multi, &mut multi, 10, Gesture::Extend(5));
        assert_eq!(multi.count(), 4, "2..=5 from the anchor");
        apply(Mode::Multi, &mut multi, 10, Gesture::Toggle(8));
        assert_eq!((multi.count(), multi.span_count()), (5, 2));
        apply(Mode::Multi, &mut multi, 10, Gesture::All);
        assert_eq!((multi.count(), multi.span_count()), (10, 1));
        apply(Mode::Multi, &mut multi, 10, Gesture::Nothing);
        assert!(multi.is_empty());
    }

    /// **The span list stays sorted, disjoint and non-adjacent under every gesture**, which is what
    /// every `partition_point` in this module assumes.
    #[test]
    fn the_span_list_is_sorted_disjoint_and_non_adjacent_under_every_gesture() {
        let mut sel = Selection::new();
        let mut oracle = [false; 200];
        // A deterministic walk rather than a random one: a fixture that cannot be replayed is a
        // fixture nobody can debug.
        for step in 0..400usize {
            let i = (step * 37) % 200;
            match step % 5 {
                0 => {
                    sel.insert(i, (i + 3).min(200));
                    for f in oracle.iter_mut().take((i + 3).min(200)).skip(i) {
                        *f = true;
                    }
                }
                1 => {
                    sel.remove(i, (i + 2).min(200));
                    for f in oracle.iter_mut().take((i + 2).min(200)).skip(i) {
                        *f = false;
                    }
                }
                2 => {
                    let was = oracle[i];
                    sel.toggle(i);
                    oracle[i] = !was;
                }
                3 => {
                    sel.select_only(i);
                    oracle.iter_mut().for_each(|f| *f = false);
                    oracle[i] = true;
                }
                _ => {
                    sel.extend_to(i);
                    oracle.iter_mut().for_each(|f| *f = false);
                    let a = sel.anchor.unwrap_or(i);
                    let (lo, hi) = if a <= i { (a, i) } else { (i, a) };
                    for f in oracle.iter_mut().take(hi + 1).skip(lo) {
                        *f = true;
                    }
                }
            }
            for (i, want) in oracle.iter().enumerate() {
                assert_eq!(sel.contains(i), *want, "step {step}, row {i}");
            }
            let spans = sel.spans();
            assert!(
                spans.windows(2).all(|w| w[0].end < w[1].start),
                "step {step}: sorted, disjoint and non-adjacent, got {spans:?}"
            );
            assert!(
                spans.iter().all(|s| !s.is_empty()),
                "step {step}: empty span"
            );
        }
    }

    /// **The pointer and the keyboard produce the same six gestures.**
    ///
    /// Criterion 12: ctrl-click toggles one span and shift-click extends from the anchor, both read
    /// from `Response::mods`, and both routed through the same [`apply`] as their keyboard twins.
    #[test]
    fn ctrl_click_and_shift_click_are_the_same_gestures_as_their_keyboard_twins() {
        assert_eq!(from_click(Mods::NONE, 5), Gesture::Plain(5));
        assert_eq!(from_click(Mods::CTRL, 5), Gesture::Toggle(5));
        assert_eq!(from_click(Mods::SHIFT, 5), Gesture::Extend(5));
        assert_eq!(
            from_click(Mods::CTRL.with(Mods::SHIFT), 5),
            Gesture::ExtendAdd(5)
        );

        let space = crate::keys::press_with(Code::Char(' '), Mods::NONE);
        assert_eq!(from_key(&space, 5, None), Some(Gesture::Toggle(5)));
        let shift_down = crate::keys::press_with(Code::Down, Mods::SHIFT);
        assert_eq!(from_key(&shift_down, 5, Some(6)), Some(Gesture::Extend(6)));
        let ctrl_a = crate::keys::press_with(Code::Char('a'), Mods::CTRL);
        assert_eq!(from_key(&ctrl_a, 5, None), Some(Gesture::All));
        let escape = crate::keys::press_with(Code::Escape, Mods::NONE);
        assert_eq!(from_key(&escape, 5, None), Some(Gesture::Nothing));

        // **`Ctrl+↓` is the one that produces no gesture at all** — the cursor moves and the
        // selection does not, which is what an enum of selection states cannot say.
        let ctrl_down = crate::keys::press_with(Code::Down, Mods::CTRL);
        assert_eq!(from_key(&ctrl_down, 5, Some(6)), None);
        assert_eq!(
            ctrl_step(&ctrl_down, Cursor::new(10, 8).to(5)),
            Some(6),
            "and the cursor still moves"
        );

        // The two readings agree arm for arm, which is what *one vocabulary* means.
        let pairs = [
            (Mods::NONE, Code::Down, Gesture::Plain(6)),
            (Mods::SHIFT, Code::Down, Gesture::Extend(6)),
        ];
        for (mods, code, want) in pairs {
            let k = crate::keys::press_with(code, mods);
            assert_eq!(from_key(&k, 5, Some(6)), Some(want));
            assert_eq!(from_click(mods, 6), want);
        }
    }

    /// **The type-ahead bound is the whole of §5's 211×**, as a ratio of *rows looked at* rather
    /// than as a timing.
    #[test]
    fn one_keystroke_into_a_million_rows_looks_at_the_budget_and_not_at_the_content() {
        let bounded = search_range(0, VOLUMES[2], SEARCH_BUDGET);
        assert_eq!(bounded.len(), SEARCH_BUDGET);
        let unbounded = search_range(0, VOLUMES[2], 0);
        assert_eq!(unbounded.len(), VOLUMES[2]);
        assert_eq!(
            unbounded.len() / bounded.len(),
            244,
            "the ratio the bound buys at a million rows"
        );
        // The search runs from the cursor, which is *find the next one* and not *find the first*.
        assert_eq!(
            search_range(900_000, VOLUMES[2], SEARCH_BUDGET).start,
            900_000
        );
        // And it never runs off the end.
        let tail = search_range(999_000, VOLUMES[2], SEARCH_BUDGET);
        assert_eq!(tail, 999_000..1_000_000);
    }

    /// A fixture that draws **two collections at two different source lines**, each with `rows`
    /// rows whose own targets are declared, and answers `(regions, merges)`.
    ///
    /// # Two lines and no `cx.with_key`, which is the whole fixture
    ///
    /// This is ADR 0027's screen exactly. `#[track_caller]` gives the two *collections* two ids, so
    /// the outer level is already distinct and nothing here is a lazy call site. What it cannot give
    /// is the **rows**: `cx.id()` mints an id and does not push it, and `Ctx::scroll_scope` roots no
    /// identity — so a row loop written without `cx.with_id` roots at whatever the enclosing stack
    /// is, which is the same node for both. Wrapping the two calls in `cx.with_key` would separate
    /// them at the outer level and the defect would not reproduce at all.
    ///
    /// Both arms are here so the comparison is one function with one value: the correct one goes
    /// through [`collection`], the defective one through [`defective::unkeyed_rows`].
    fn side_by_side(rows: usize, keyed: bool) -> (usize, u64) {
        let mut driver = crate::runner::driver_at(40, 8, vitui_runtime::Density::default());
        let (mut left, mut right) = (CollState::new(), CollState::new());
        driver.frame(|cx| {
            let area = cx.area();
            let opts = CollOpts::default();
            let w = area.w / 2;
            let bands = [
                Rect::new(0, 0, w, area.h),
                Rect::new(i32::from(w), 0, w, area.h),
            ];
            let mut row = |cx: &mut Ctx<'_, '_>, r: Rect, i: usize, _f: Face| {
                cx.with_key(i as u64, |cx| {
                    let id = cx.id();
                    let _ = cx.interact(id, r, Interest::CLICK);
                });
            };
            let mut find = |_: &str, _: Range<usize>| None;
            if keyed {
                let _ = collection(
                    cx,
                    bands[0],
                    &mut left,
                    &opts,
                    Rows::of(rows),
                    &mut find,
                    &mut row,
                );
                let _ = collection(
                    cx,
                    bands[1],
                    &mut right,
                    &opts,
                    Rows::of(rows),
                    &mut find,
                    &mut row,
                );
            } else {
                defective::unkeyed_rows(&mut Direct, cx, bands[0], &mut left, rows);
                defective::unkeyed_rows(&mut Direct, cx, bands[1], &mut right, rows);
            }
        });
        let frame = driver.inspect();
        (frame.hits().len(), u64::from(frame.ids().merges()))
    }

    /// **Two collections on one screen declare two entries and merge nothing.**
    ///
    /// Both halves of register row 71, and they are one screen because they fail together.
    ///
    /// - **One hit entry per collection** (ADR 0028): the component declares exactly one however
    ///   many rows it stands, and the rows' own targets are the row drawer's.
    /// - **`merges == 0`** (ADR 0027): the row loop is wrapped in `cx.with_id`, so two collections
    ///   drawn from *one* line do not collide. The defective arm is the same loop written without
    ///   it, and **the screen it draws is pixel for pixel correct** — which is why `merges` is the
    ///   only instrument that reports it.
    #[test]
    fn two_collections_on_one_screen_declare_two_entries_and_merge_nothing() {
        let rows = 8usize;
        let (regions, merges) = side_by_side(rows, true);
        assert_eq!(merges, 0, "the row loop is wrapped in `cx.with_id`");
        assert_eq!(
            regions,
            2 + 2 * rows,
            "one entry a collection plus one a visible row, and not one a *content* row"
        );

        // A collection whose rows declare nothing is one entry, whatever the volume — which is what
        // *one hit entry per collection* means with the row drawer's own targets taken away.
        let mut driver = crate::runner::driver_at(40, 8, vitui_runtime::Density::default());
        for len in [1_000usize, 1_000_000] {
            let mut st = CollState::new();
            driver.frame(|cx| {
                let area = cx.area();
                let _ = collection(
                    cx,
                    area,
                    &mut st,
                    &CollOpts::default(),
                    Rows::of(len),
                    &mut |_: &str, _: Range<usize>| None,
                    &mut |_cx: &mut Ctx<'_, '_>, _r: Rect, _i: usize, _f: Face| {},
                );
            });
            assert_eq!(
                driver.inspect().hits().len(),
                1,
                "a collection over {len} rows declares one hit entry"
            );
            assert_eq!(driver.inspect().stop_count(), 1, "and it is one tab stop");
        }

        // **The other direction, and it is the whole reason the gate is `merges`.** The unkeyed row
        // loop declares the same number of regions and every row of the second collection is inert.
        let (broken_regions, broken_merges) = side_by_side(rows, false);
        assert_eq!(
            broken_regions,
            2 + rows,
            "the second collection's rows collided with the first's, so they are not in the index \
             at all — and nothing about the picture says so"
        );
        assert_eq!(
            broken_merges,
            rows as u64,
            "one merge a row of the second collection: {rows} of {} targets inert",
            2 * rows
        );
    }

    /// **A row is hovered by arithmetic on this frame's pointer, with no per-row hit entry.**
    ///
    /// `Response::local` is computed inside the runtime's `declare` from *this frame's* pointer,
    /// unlike `hovered` — so `offset + local.1` is right on the frame the pointer arrives, and the
    /// alternative is 389 regions for a frame-old answer.
    ///
    /// The pointer is posted as a real `Mouse`, which spec §5 records as impossible: `Mouse` needed
    /// a `Buttons` and a `MouseKind` and **neither was in `ENGINE_NAMES` at all**. Runtime
    /// architecture issue 22 re-exported all three, so this is a driven gesture rather than an
    /// assigned field.
    #[test]
    fn a_row_is_hovered_by_arithmetic_on_this_frames_pointer() {
        use vitui_runtime::{Buttons, Mods, Mouse, MouseKind};

        let mut driver = crate::runner::driver_at(40, 8, vitui_runtime::Density::default());
        let mut st = CollState::new();
        st.offset = 100;
        let mut seen: Vec<(usize, bool)> = Vec::new();

        // Two frames: the first opens the tracking level the pointer needs, the second carries the
        // move. Both draw the same collection, so nothing about the second frame is special except
        // that a pointer exists by then.
        for frame in 0..2 {
            if frame == 1 {
                driver.post_mouse(Mouse {
                    x: 5,
                    y: 3,
                    kind: MouseKind::Move,
                    buttons: Buttons::NONE,
                    mods: Mods::NONE,
                    at: Instant::now(),
                });
            }
            driver.frame(|cx| {
                let area = cx.area();
                seen.clear();
                let _ = collection(
                    cx,
                    area,
                    &mut st,
                    &CollOpts::default(),
                    Rows::of(1_000),
                    &mut |_: &str, _: Range<usize>| None,
                    &mut |_cx: &mut Ctx<'_, '_>, _r: Rect, i: usize, f: Face| {
                        seen.push((i, f.hovered));
                    },
                );
            });
        }

        let hovered: Vec<usize> = seen.iter().filter(|(_, h)| *h).map(|(i, _)| *i).collect();
        assert_eq!(
            hovered,
            vec![103],
            "the row under the pointer is `offset + local.1`, on the frame the pointer arrived: {seen:?}"
        );
        assert_eq!(seen.len(), 8, "and only the window was iterated");
        assert_eq!(
            driver.inspect().hits().len(),
            1,
            "one hit entry, which is what makes the arithmetic necessary and cheap"
        );
    }

    /// **The modifier byte is read in one function, and that function takes a `Response`'s.**
    ///
    /// Criterion 12's second sentence — *no component reads a modifier from anywhere else* — as a
    /// scan rather than as a claim, because an absence has no expression. The two readers this
    /// crate is allowed are [`from_click`], which is handed `Response::mods`, and [`from_key`],
    /// which is handed a key's; `crate::keys` owns the keyboard half and is where `is_chord` lives.
    #[test]
    fn the_modifier_byte_is_read_in_one_function_and_it_takes_a_response() {
        // **The library half of the file and not the test half**, which is the crate's own recorded
        // shape: `crate::frame`'s deleted-helper scan reported *itself* on its first run, because a
        // scanner looking for a literal contains that literal. This one did too — it matched the
        // line of its own `filter` — so it stops where the tests begin.
        let source = include_str!("collect.rs");
        let library = source
            .split_once("\n#[cfg(test)]\n")
            .map_or(source, |(head, _)| head);
        assert!(
            library.len() < source.len(),
            "the scan no longer stops at the test module, so it is scanning itself again"
        );
        let readers: Vec<&str> = library
            .lines()
            .map(str::trim)
            .filter(|l| !l.starts_with("//"))
            .filter(|l| l.contains(".mods.") || l.contains("resp.mods"))
            .collect();
        // `from_click`'s `match`, `from_key`'s three guards, `ctrl_step`'s guard, and the one call
        // site that hands `resp.mods` over. Everything else in the component reads a `Gesture`.
        assert!(
            !readers.is_empty(),
            "the scan found no modifier reader at all, which means it has stopped scanning"
        );
        for line in &readers {
            assert!(
                line.contains("mods.ctrl()")
                    || line.contains("mods.shift()")
                    || line.contains("mods.alt()")
                    || line.contains("from_click(resp.mods"),
                "a modifier is read at `{line}`, outside `from_click`, `from_key` and `ctrl_step`"
            );
        }
        // And the pointer half really is handed the response's byte rather than a key's.
        assert!(
            crate::dense::declares(source, "from_click(resp.mods, at)"),
            "the pointer gesture no longer comes from `Response::mods`"
        );
    }

    /// **A collection declines a chord and swallows no accelerator.**
    ///
    /// Register row 5's claim — *a chord pressed into every focusable types nothing* — over a real
    /// component for the first time. Type-ahead is the one thing here that eats a printable key, and
    /// `crate::keys::is_chord` is the same one line that stops a field eating `Ctrl+S`.
    ///
    /// The other direction is on the same screen: an unmodified `z` **is** taken, so the decline is
    /// a discrimination rather than a component that declines everything.
    #[test]
    fn a_collection_declines_a_chord_and_swallows_no_accelerator() {
        use vitui_runtime::Chord;

        fn press_into(chord: Chord) -> (usize, String) {
            let mut driver = crate::runner::driver_at(40, 8, vitui_runtime::Density::default());
            let mut st = CollState::new();
            let labels: Vec<&str> = (0..64).map(label).collect();
            // Frame one seats the focus, frame two carries the key.
            for frame in 0..2 {
                if frame == 1 {
                    driver.post_key(crate::keys::press(chord));
                }
                driver.frame(|cx| {
                    let area = cx.area();
                    let resp = collection(
                        cx,
                        area,
                        &mut st,
                        &CollOpts::default(),
                        Rows::of(labels.len()),
                        &mut |buf: &str, range: Range<usize>| {
                            crate::nav::matched(buf, &labels[range.clone()])
                                .map(|hit| range.start + hit)
                        },
                        &mut |_cx: &mut Ctx<'_, '_>, _r: Rect, _i: usize, _f: Face| {},
                    );
                    // **Seated from the response and after the draw**, which is architecture issue
                    // 25's shape with the one wrinkle a component that mints its own id adds: the
                    // application cannot name the id until the component has returned it. So the
                    // guard is `cx.focused().is_none()` — *has anything got the keyboard* — and not
                    // `!cx.is_focused(id)`, which drags it back every frame the user has tabbed
                    // away.
                    if cx.focused().is_none() {
                        cx.focus(resp.id);
                    }
                });
            }
            (driver.unhandled().len(), st.ahead.buffer().to_string())
        }

        // **The accelerator is declined**, so the application's key map gets its turn.
        let (declined, buffer) = press_into(Chord::key('s').ctrl());
        assert_eq!(declined, 1, "`Ctrl+S` reaches the application");
        assert!(buffer.is_empty(), "and nothing of it is in the buffer");

        // The other direction: a bare letter is type-ahead and is not declined.
        let (declined, buffer) = press_into(Chord::key('k'));
        assert_eq!(declined, 0, "a bare letter is the collection's");
        assert_eq!(buffer, "k", "and it is what the buffer holds");

        // **`Tab` never reaches the component at all**, which is a different mechanism and worth
        // separating from the decline: the ring takes it in the runtime's own `end`, so `unhandled`
        // is **0** — nothing declined it, because nothing was offered it. A component that had to
        // decline `Tab` would be a component the ring had already lost.
        let (declined, buffer) = press_into(Chord::new(vitui_runtime::keys::Code::Tab));
        assert_eq!(declined, 0, "the ring took it before the component saw it");
        assert!(buffer.is_empty(), "and nothing of it is in the buffer");
    }

    /// **Everything a collection stores is a position, and a revision it does not recognise clears
    /// them.**
    ///
    /// Components ticket 13, §10 and ADR 0031. A sort changes no data and no length, so nothing
    /// inside the component can notice it — the frame after draws a perfectly correct list with the
    /// wrong rows selected and the editor open on the wrong row. **One `u64`, compared once a
    /// frame**, is the only thing that makes it noticeable.
    ///
    /// All three directions are here, because the middle one is the whole mechanism:
    ///
    /// - a revision the component has not seen **clears** the positions — [`Policy::Clear`], and it
    ///   is the only policy available here because `Remap` is the caller's function and
    ///   `Drop`/`Stash` need the interval a splice removed;
    /// - a caller that has carried the positions across says so with [`CollState::reconciled`], and
    ///   the branch does not fire;
    /// - `Rows::of` carries [`Revision::UNKNOWN`], which never matches — so a caller with no order
    ///   at all is **never** told its positions went stale, which is right, because it has nothing
    ///   that can permute them.
    ///
    /// [`Policy::Clear`]: crate::order::Policy::Clear
    #[test]
    fn a_revision_the_component_has_not_seen_clears_every_position_it_holds() {
        fn draw(driver: &mut vitui_runtime::ctx::Driver, st: &mut CollState, rows: Rows) {
            driver.frame(|cx| {
                let area = cx.area();
                let _ = collection(
                    cx,
                    area,
                    st,
                    &CollOpts {
                        mode: Mode::Multi,
                        ..Default::default()
                    },
                    rows,
                    &mut |_: &str, _: Range<usize>| None,
                    &mut |_cx: &mut Ctx<'_, '_>, _r: Rect, _i: usize, _f: Face| {},
                );
            });
        }

        let mut driver = crate::runner::driver_at(40, 8, vitui_runtime::Density::default());
        let mut st = CollState::new();
        st.sel.select_only(3);
        st.sel.insert(10, 14);
        st.editing = Some(11);
        let first = Rows::new(1_000, Revision::fresh());
        draw(&mut driver, &mut st, first);
        assert_eq!(
            (st.sel.count(), st.editing),
            (5, Some(11)),
            "the first frame at a revision is not an edit"
        );
        assert_eq!(st.revision(), first.rev);

        // Same revision, same positions: a steady frame notices nothing.
        draw(&mut driver, &mut st, first);
        assert_eq!(st.sel.count(), 5);

        // **A sort.** No data moved and no length moved, and the selection goes.
        let sorted = Rows::new(1_000, Revision::fresh());
        draw(&mut driver, &mut st, sorted);
        assert_eq!(
            st.sel.count(),
            0,
            "`Clear` is the honest default for a permutation"
        );
        assert_eq!(st.editing, None, "the editing slot is a position too");
        assert_eq!(st.sel.anchor, None);
        assert_eq!(st.revision(), sorted.rev);

        // **The caller's half**: it carried the positions across, so the component notices nothing.
        let mut st = CollState::new();
        st.sel.select_only(3);
        draw(&mut driver, &mut st, first);
        let remapped = Rows::new(1_000, Revision::fresh());
        st.reconciled(remapped);
        draw(&mut driver, &mut st, remapped);
        assert_eq!(
            st.sel.count(),
            1,
            "a caller that says it has reconciled is believed, which is what makes `Remap` and \
             `Stash` expressible at all"
        );

        // **A caller with no order is never told its positions went stale**, at any number of
        // frames, because `Revision::UNKNOWN` never matches and the branch is guarded on
        // `is_known()`.
        let mut st = CollState::new();
        st.sel.select_only(3);
        for _ in 0..4 {
            draw(&mut driver, &mut st, Rows::of(1_000));
        }
        assert_eq!(st.sel.count(), 1);
        assert_eq!(st.revision(), Revision::UNKNOWN);
    }

    /// **The keyboard moves the cursor and the reveal fires only when it did.**
    ///
    /// `CONTEXT.md`'s rule, on the component rather than on a harness: a frame where nothing asked
    /// leaves no request, and a frame where a key moved the cursor off the bottom of the window
    /// leaves exactly one. The wheel gate itself is components 20's and stays red;
    /// [`defective::every_frame`] is the arm it fires against.
    #[test]
    fn a_reveal_is_left_behind_only_when_a_key_moved_the_cursor() {
        fn play(keys: &[vitui_runtime::Chord], arm: bool) -> (bool, usize) {
            let mut driver = crate::runner::driver_at(40, 8, vitui_runtime::Density::default());
            let mut st = CollState::new();
            st.offset = 200;
            for frame in 0..=keys.len() {
                if let Some(c) = keys.get(frame.wrapping_sub(1)) {
                    driver.post_key(crate::keys::press(*c));
                }
                driver.frame(|cx| {
                    let area = cx.area();
                    let opts = CollOpts::default();
                    let find = &mut |_: &str, _: Range<usize>| None;
                    let row = &mut |_ink: &mut Direct,
                                    _cx: &mut Ctx<'_, '_>,
                                    _r: Rect,
                                    _i: usize,
                                    _f: Face| {};
                    let seated = match arm {
                        true => collection_into(
                            &mut Direct,
                            cx,
                            area,
                            &mut st,
                            &opts,
                            Rows::of(1_000),
                            find,
                            row,
                        ),
                        false => defective::every_frame(
                            &mut Direct,
                            cx,
                            area,
                            &mut st,
                            &opts,
                            Rows::of(1_000),
                            find,
                            row,
                        ),
                    };
                    if cx.focused().is_none() {
                        cx.focus(seated.id);
                    }
                });
            }
            (driver.inspect().into_view().is_some(), st.sel.lead)
        }

        // Nothing asked: no request, and the cursor has not moved.
        assert_eq!(play(&[], true), (false, 0));
        // A key moved the cursor, and the cursor is two hundred rows above the fold.
        let (asked, lead) = play(
            &[vitui_runtime::Chord::new(vitui_runtime::keys::Code::Down)],
            true,
        );
        assert!(asked, "a keyboard-driven move asks to be brought into view");
        assert_eq!(lead, 1);
        // **The defect**: it asks on a frame where nothing moved, for ever, which is what kills the
        // wheel. Components 20 owns the gate; what this asserts is that the arm exists and differs.
        assert!(play(&[], false).0);
    }
}
