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

use vitui_runtime::keys::{Code, Edge, Pressed};
use vitui_runtime::layout::text::{truncate, width};
use vitui_runtime::layout::{Constraint, rect, solve};
use vitui_runtime::{Ctx, Glyph, Id, Interest, Mods, Rect, Response, Revision, Role, Scrollable};

use crate::frame::Face;
use crate::ink::{Direct, Ink};
use crate::nav::{self, Cursor, TypeAhead};
use crate::order::{Ask, Asked, Order, Rows};

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
    /// **Whether the pointer was already down on this collection last frame.**
    ///
    /// The rising edge of [`Response::pressed`], which the runtime does not publish: `pressed` is
    /// `grab == Some(id)` and so is *true every frame the button is held*, while the edge lives in
    /// the accumulator and is never surfaced. One bool here turns the level back into the edge.
    ///
    /// **Private and not a slot per row**, so ADR 0028's rule is untouched: this is one fact about
    /// the collection, not a fact about a row.
    pressing: bool,
    /// **Whether this frame was the rising edge of the press.** Published, not duplicated.
    ///
    /// A container built on this component that has a gesture of its own on the press —
    /// [`tree`]'s click on a chevron is the one — needs the same edge, and the two ways to get it
    /// are to read it here or to keep a second `pressing` bool beside this one. The second is the
    /// shape ADR 0028 refuses one axis over: two stores of one fact, which disagree the first time
    /// one of them is updated in a branch the other is not.
    edge: bool,
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
            pressing: false,
            edge: false,
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

    /// **Whether the frame just drawn was the rising edge of a press on this collection.**
    ///
    /// `Response::pressed` is a *level* — true every frame the button is held — and the runtime does
    /// not publish the edge (runtime architecture issue 29). [`collection`] reconstructs it to stop
    /// a ctrl-click re-toggling for as long as the user leans on the button; this is that same one
    /// fact, read rather than kept twice.
    pub const fn press_edge(&self) -> bool {
        self.edge
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
/// **Hostile axes:** `scrolled`, `shrunk`, `wheeled`.
///
/// Scenes 4, 5 and 6, and each of the three was established by a defect that passed every gate then
/// in force **and looked healthier than the correct build**. The fourth is deliberately absent: a row
/// truncates through [`crate::text::fit`], which is `text`'s flag and scene 28's.
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
        &mut no_refusal,
        Shape::Virtualised,
        Reveal::WhenAsked,
    )
}

/// **A component built on [`collection`] that owns keys of its own gets first refusal, and it has to
/// happen inside the one drain loop.**
///
/// The hook is called for every key addressed to the collection, before `crate::nav::step` and
/// before [`from_key`]; `true` means *mine, consumed*. It carries the cursor because every gesture
/// that has wanted one so far is *about* the cursor's row — [`tree`]'s fold and unfold are, and the
/// row is a position the caller's index is read at.
///
/// # It cannot be a second loop, and that is a fact about the runtime rather than a preference
///
/// `Ctx::decline` hands a key back **and ends the level's turn at the queue**: `next_key` answers
/// `None` afterwards, however many keys are left, until the routing target moves outward. A
/// container that drained keys before `collection` would have to decline everything that was not
/// its own, and `collection` would then see nothing at all; one that drained after would find the
/// queue already closed. So the hook is a parameter of the one loop.
///
/// And the collision is real rather than hypothetical: [`crate::nav::step`] reads `←` and `→` as
/// `↑` and `↓`, which are exactly the two keys a tree folds and unfolds with.
///
/// **It is `Refusal` and not `Chord`**, because [`vitui_runtime::Chord`] is a modifier byte and a
/// key code and this is a *decision about* one. Two meanings of one word, both carrying
/// measurements, is the collision `CONTEXT.md` exists to prevent — and it is the one
/// `crate::collect::Span` did not get out of the way of in time.
pub(crate) type Refusal<'a> = &'a mut dyn FnMut(&Pressed, usize) -> bool;

/// The hook a collection with no container over it passes: **nothing is anybody else's**.
fn no_refusal(_: &Pressed, _: usize) -> bool {
    false
}

/// **[`collection_into`] with a [`Refusal`], which is the entry a container built on it takes.**
///
/// Crate-private, because the hook is not part of spec §1's component shape: it is the seam one
/// component reaches another through, and a public one would invite an application to spell a
/// keyboard for a collection it did not write. `pub(crate)` and not module-private since components
/// ticket 26: [`crate::input::select`]'s popup body is the second container to need the hook and the
/// first one that is not in this file, and a copy of the drain loop beside it would be a second place
/// §5's lockstep scan cursor is advanced.
#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "`collection_into`'s eight plus the hook. The eight are spec §5's and the ninth is               what makes a container's own keys expressible at all"
)]
pub(crate) fn collection_chorded<I, F, R>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut CollState,
    opts: &CollOpts,
    rows: Rows,
    mut find: F,
    mut row: R,
    first: Refusal<'_>,
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
        first,
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
/// [`crate::wheel::Reveal`], whose three arms these are.
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
    first: Refusal<'_>,
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
    // **A row selects on the press and not on the release**, which is the opposite of a button and
    // is deliberate. `Response::clicked` is *pressed and released without leaving* — right for a
    // button, because sliding off before the release is how a user cancels one — and wrong for a
    // list, where every platform selects under the finger the moment it lands. Driven through a
    // pty, the release-driven spelling reads as **nothing on the press and the row selected on the
    // release**, which is what the defect looks like from the outside.
    //
    // It also costs a gesture the component is supposed to have: a range cannot be drag-selected
    // if the selection does not begin until the button comes up.
    //
    // The edge is reconstructed here rather than read, because `Response::pressed` is a level —
    // applied every frame it is held, `from_click` would re-toggle a ctrl-click for as long as the
    // user leans on the button. See runtime architecture issue 29.
    let press_edge = resp.pressed && !st.pressing;
    st.pressing = resp.pressed;
    st.edge = press_edge;
    if press_edge && let Some(at) = over {
        apply(opts.mode, &mut st.sel, len, from_click(resp.mods, at));
        resp.changed = true;
    }

    // The keyboard half, and the two are one vocabulary. **A page is the viewport's height and
    // only the caller knows that** (`crate::nav::Cursor`), so the rectangle's own height is what
    // goes in rather than a constant.
    let asked = keyboard(
        cx,
        id,
        st,
        opts,
        len,
        find,
        usize::from(area.h.max(1)),
        first,
    );
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
                if asks_for_a_reveal(reveal, asked.reveal) {
                    let at = i32::try_from(st.sel.lead).unwrap_or(i32::MAX);
                    cx.request_into_view(Rect::new(0, at, area.w, 1));
                }
            }
        });
    });
    resp
}

/// **Whether this frame asks the enclosing area to bring the cursor's row into view.**
///
/// # The rule, and the four resolved tickets that broke it
///
/// > It fires only for a keyboard-driven focus move. A press already proves the widget was on
/// > screen, and an unconditional pull is the list's old bug: it fights the wheel, dragging the
/// > viewport back to the selection every time the user scrolls away from it.
/// > (`crates/vitui-runtime/src/scroll.rs`, and `CONTEXT.md` forbids the unconditional form by name)
///
/// **Four *resolved* prototype tickets called it on every frame anyway** — four builds, each written
/// by someone who had read that rule. The pointer then cannot scroll the list at all: twenty wheel
/// clicks moved the offset **0 against 16**, and nothing else moved, so every counting gate passed
/// and the defective build looked healthier than the correct one. That is why the reason is here and
/// not only in the glossary:
///
/// > Every obligation this map has stated as a sentence has been broken by someone who had read it.
///
/// # The condition, and why it is not `resp.focused` or `resp.clicked`
///
/// The one input is [`Handled::reveal`], which the drain loop sets when — and only when — a key moved
/// the cursor: [`crate::nav::step`], [`ctrl_step`] or a type-ahead [`seek`] that landed. A press is
/// deliberately not on that list; it already proves the row was on screen, and a reveal on a click
/// fights the wheel for the rest of the session.
///
/// # Three arms, because deleting the call passes half a gate
///
/// [`Reveal::EveryFrame`] and [`Reveal::Never`] are [`defective`]'s two arms and neither is
/// reachable from a caller. They are here rather than in a copy of this function because a gate
/// written against a copy tests the copy — see [`crate::ink`] — and because they must differ from
/// the shipped build by **one value**, so that a reviewer's diff is one line. `crate::wheel` fires
/// all three: `WhenAsked` settles twenty clicks at twenty, `EveryFrame` at zero, and `Never` also at
/// twenty — which is why the gate cannot be written on the wheel alone. Deleting this call is not a
/// fix; it loses the keyboard instead of fixing the pointer.
const fn asks_for_a_reveal(reveal: Reveal, cursor_moved: bool) -> bool {
    match reveal {
        Reveal::WhenAsked => cursor_moved,
        Reveal::EveryFrame => true,
        Reveal::Never => false,
    }
}

/// What one frame's keys did: whether the store changed, and whether anything asked for a reveal.
///
/// **Not `Asked`**, which is what it was called until [`tree`] arrived: [`crate::order::Asked`] is
/// the one-slot *request* a component leaves for its caller, and two of that name in one file is
/// exactly the `expected Asked, found Asked` collision `CONTEXT.md`'s glossary exists to stop.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct Handled {
    changed: bool,
    reveal: bool,
}

/// **Drain the keys addressed to this collection.**
///
/// `Ctx::next_key` answers nobody but the routing target, so this is the whole keyboard surface and
/// a key that is not this component's is declined rather than swallowed — which is what stops a list
/// eating `Ctrl+S`.
#[expect(
    clippy::too_many_arguments,
    reason = "the seven this drain loop already needed plus the one hook that makes a container's \
              own keys expressible. A second loop is not available — see `Refusal` — so the hook has \
              to be a parameter of this one"
)]
fn keyboard<F>(
    cx: &mut Ctx<'_, '_>,
    id: Id,
    st: &mut CollState,
    opts: &CollOpts,
    len: usize,
    find: &mut F,
    page: usize,
    first: Refusal<'_>,
) -> Handled
where
    F: FnMut(&str, Range<usize>) -> Option<usize>,
{
    let mut out = Handled::default();
    while let Some(k) = cx.next_key(id) {
        // **A release is not a gesture, and it is dropped rather than declined.** The engine pushes
        // kitty flag 31 and bit 2 of that is *report event types*, so on a terminal that speaks the
        // protocol every key arrives twice. Unguarded, one `Down` moved the cursor two rows and one
        // `z` put two `z`s in the type-ahead buffer — `crate::nav::step` refuses the first half now
        // and `seek` would still have taken the second, because a released character is a character.
        //
        // **Dropped, not declined**: declining hands it back to the router, and the next component
        // to read it has the same defect for the same reason. Nobody wants a release.
        if k.kind == Edge::Release {
            continue;
        }
        // **First refusal, inside the one drain loop** — see [`Refusal`]. A component built on this
        // one that owns keys of its own cannot read them before or after this call: `Ctx::decline`
        // sets a flag on the level, so the moment `collection` hands one key back, `next_key`
        // answers `None` to everything else at this id for the rest of the frame. `←` and `→` are
        // also the two keys `crate::nav::step` reads as `↑` and `↓`, so a hook that ran second would
        // arrive after the cursor had already moved.
        if first(&k, st.sel.lead) {
            out.changed = true;
            continue;
        }
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
        Band, BandShape, Cell, CellKeys, ColVirt, CollOpts, CollState, Column, Ctx, Face, HSign,
        Id, Indent, Ink, Node, Order, Range, Rect, Response, Reveal, Rows, Scan, Shape, TableOpts,
        TableShape, TableState, TreeOpts, TreeShape, TreeState, draw_with, no_refusal, table_with,
        tree_with,
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
            &mut no_refusal,
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
            &mut no_refusal,
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
            &mut no_refusal,
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

    // ── `tree`'s one ─────────────────────────────────────────────────────────────────────────────

    /// **The tree whose indent is the depth, and the depth is data.**
    ///
    /// Spec §7's negative case, and it is [`super::tree`] with one field of `TreeShape` changed.
    /// **An unclamped indent makes a row's cost proportional to its depth** — and every counter this
    /// crate can read is either identical or *better* on it: the engine reports the same columns
    /// written and the same distinct cells, because the clip eats the overrun; it declares the same
    /// regions and the same stops; it makes **fewer** verbs, because the label rectangle collapses
    /// and there is nothing left of the row to draw; and it runs *faster* for the same reason.
    ///
    /// The one quantity that moves is **cells asked for**, and at depth ten the two builds are one
    /// frame — which is a statement about the scene list (§21) rather than about the gate.
    #[track_caller]
    #[expect(
        clippy::too_many_arguments,
        reason = "`tree_into`'s eight exactly, because a defective arm that took a different \
                  signature would be a different function rather than the same one with one value \
                  changed"
    )]
    pub fn unclamped_indent<I, F, R>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        st: &mut TreeState,
        opts: &TreeOpts,
        index: &Order,
        find: F,
        row: R,
    ) -> Response
    where
        I: Ink,
        F: FnMut(&str, Range<usize>) -> Option<usize>,
        R: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, Node, Face),
    {
        tree_with(
            ink,
            cx,
            area,
            st,
            opts,
            index,
            find,
            row,
            TreeShape {
                indent: Indent::Unclamped,
            },
        )
    }

    // ── `table`'s four, and the one that is refused while being correct ──────────────────────────

    /// **The band written by arithmetic instead of into a view.** §6's refused arm.
    ///
    /// One `child` fewer than [`super::table_into`] and nothing else changed, so a reviewer's diff
    /// is one field. The band's edge column reaches past the viewport and overwrites the pinned band
    /// — **identical verbs, identical time**, and a band of cells re-damaged on every steady frame
    /// for ever. Where the overrun is on the *right* the picture even survives, because the pin
    /// draws afterwards and wins; the pair `writes` against `distinct` is what sees it, and an
    /// equality against a reference render does not.
    #[track_caller]
    #[expect(
        clippy::too_many_arguments,
        reason = "`table_into`'s nine exactly, because a defective arm that took a different \
                  signature would be a different function rather than the same one with one value \
                  changed"
    )]
    pub fn arithmetic_band<I, F, C>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        st: &mut TableState,
        opts: &TableOpts,
        cols: &[Column],
        rows: Rows,
        find: F,
        cell: C,
    ) -> Response
    where
        I: Ink,
        F: FnMut(&str, Range<usize>) -> Option<usize>,
        C: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, Cell, Face),
    {
        table_with(
            ink,
            cx,
            area,
            st,
            opts,
            cols,
            rows,
            find,
            cell,
            TableShape {
                band: BandShape::Arithmetic,
                ..TableShape::default()
            },
        )
    }

    /// **Every declared column drawn, with the clip left to discard what does not fit.**
    ///
    /// The column half of *frame cost is proportional to visible cells*, refused. **The writes are
    /// identical** — the engine reports a fully clipped verb as zero columns — so the one counter a
    /// reader reaches for first cannot see it at all; what it costs is verbs, and they grow with the
    /// *declared* column count rather than with the visible one.
    #[track_caller]
    #[expect(
        clippy::too_many_arguments,
        reason = "[`arithmetic_band`]'s, unchanged"
    )]
    pub fn clip_only<I, F, C>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        st: &mut TableState,
        opts: &TableOpts,
        cols: &[Column],
        rows: Rows,
        find: F,
        cell: C,
    ) -> Response
    where
        I: Ink,
        F: FnMut(&str, Range<usize>) -> Option<usize>,
        C: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, Cell, Face),
    {
        table_with(
            ink,
            cx,
            area,
            st,
            opts,
            cols,
            rows,
            find,
            cell,
            TableShape {
                cols: ColVirt::ClipOnly,
                ..TableShape::default()
            },
        )
    }

    /// **The horizontal offset added where it should be subtracted.** §3.3's trap on the column
    /// axis, and C03's inverted scroll sign one axis over.
    ///
    /// The band shows a partly blank stretch, issues every verb it would have issued, and **measures
    /// as an improvement** because the clip eats the writes. Nothing but an equality against a
    /// reference render separates it from the correct build.
    #[track_caller]
    #[expect(
        clippy::too_many_arguments,
        reason = "[`arithmetic_band`]'s, unchanged"
    )]
    pub fn inverted_sign<I, F, C>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        st: &mut TableState,
        opts: &TableOpts,
        cols: &[Column],
        rows: Rows,
        find: F,
        cell: C,
    ) -> Response
    where
        I: Ink,
        F: FnMut(&str, Range<usize>) -> Option<usize>,
        C: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, Cell, Face),
    {
        table_with(
            ink,
            cx,
            area,
            st,
            opts,
            cols,
            rows,
            find,
            cell,
            TableShape {
                hsign: HSign::Minus,
                ..TableShape::default()
            },
        )
    }

    /// **A row key and no column key**, which is one microsecond cheaper and correct until a cell
    /// declares a target.
    ///
    /// Then every cell of a row derives one id and the row is one widget: the first cell answers and
    /// the rest are inert, on a screen that renders correctly. `merges` is the counter that sees it
    /// and nothing that reads a cell does.
    #[track_caller]
    #[expect(
        clippy::too_many_arguments,
        reason = "[`arithmetic_band`]'s, unchanged"
    )]
    pub fn row_keyed_cells<I, F, C>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        st: &mut TableState,
        opts: &TableOpts,
        cols: &[Column],
        rows: Rows,
        find: F,
        cell: C,
    ) -> Response
    where
        I: Ink,
        F: FnMut(&str, Range<usize>) -> Option<usize>,
        C: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, Cell, Face),
    {
        table_with(
            ink,
            cx,
            area,
            st,
            opts,
            cols,
            rows,
            find,
            cell,
            TableShape {
                keys: CellKeys::PerRow,
                ..TableShape::default()
            },
        )
    }

    /// **One row pass with three bands inside it — the shipped loop structure, standing alone.**
    ///
    /// The correct half of the pair §6's *4% cheaper and refused* is measured over. It is not
    /// [`super::table`]: the shipped component also runs the keyboard, the type-ahead, the reveal
    /// and the tail, none of which is the subject, and a timing that included them would be a timing
    /// of `collection`. Both halves of this pair are this one function with `per_band` flipped, so
    /// the 4% is the loop structure and nothing else.
    pub fn one_row_pass<I, C>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        st: &mut TableState,
        cols: &[Column],
        len: usize,
        cell: C,
    ) where
        I: Ink,
        C: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, Cell, Face),
    {
        passes(ink, cx, area, st, cols, len, false, cell);
    }

    /// **A pass per band: correct, measured cheaper, and refused.**
    ///
    /// §6: *the band view is opened per row inside one row pass, not once per band. A pass per band
    /// measures 4% cheaper and is refused, because three loops share nothing but the author writing
    /// the same bounds three times and cannot share §5's lockstep scan cursor.*
    ///
    /// Both halves of the refusal are visible here. It **is** cheaper — one
    /// [`Ctx::child`](vitui_runtime::Ctx::child) for the whole band instead of one a row. And it
    /// **cannot share the cursor**: each of the three passes seeks its own [`Scan`], so the
    /// `O(log k + h)` §5 buys becomes `3·(log k + h)`, and the three sets of bounds are three chances
    /// for an author to write one of them differently.
    ///
    /// It is here to be priced rather than argued about, and it is the one arm in this module that
    /// is **correct**: it writes the same cells in a different order.
    pub fn per_band_pass<I, C>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        st: &mut TableState,
        cols: &[Column],
        len: usize,
        cell: C,
    ) where
        I: Ink,
        C: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, Cell, Face),
    {
        passes(ink, cx, area, st, cols, len, true, cell);
    }

    /// The pair above, one boolean apart.
    #[track_caller]
    #[expect(
        clippy::too_many_arguments,
        reason = "the seven both halves take plus the one value between them, which is the whole \
                  point of writing them as one function"
    )]
    fn passes<I, C>(
        ink: &mut I,
        cx: &mut Ctx<'_, '_>,
        area: Rect,
        st: &mut TableState,
        cols: &[Column],
        len: usize,
        per_band: bool,
        mut cell: C,
    ) where
        I: Ink,
        C: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, Cell, Face),
    {
        let tid = cx.id();
        let solved = super::solve_columns(area.w, cols);
        let hoff = st.hoff.clamp(0, solved.max_hoff());
        let (vlo, vhi) = super::visible_columns(&solved, hoff);
        let max = (0, CollState::max_offset(len, area.h));
        let _ = cx.scrollable(
            tid,
            area,
            vitui_runtime::Interest::CLICK,
            vitui_runtime::Scrollable::between((0, st.coll.offset), max),
        );
        let band_x = i32::from(solved.left_w);
        let right_x = band_x + i32::from(solved.view_w);
        let sel = &st.coll.sel;
        cx.scroll_scope(tid, area, (0, st.coll.offset), max, |cx| {
            let window = cx.visible_rows();
            let lo = window.start.max(0);
            let hi = window.end.min(i32::try_from(len).unwrap_or(i32::MAX));
            let first = usize::try_from(lo).unwrap_or(0);
            let face_of = |scan: &mut Scan<'_>, i: usize| Face {
                selected: scan.at(i),
                cursor: sel.lead == i,
                active: false,
                hovered: false,
                disabled: false,
            };
            let mut one = |ink: &mut I,
                           cx: &mut Ctx<'_, '_>,
                           slot: usize,
                           x: i32,
                           y: i32,
                           row: usize,
                           face: Face| {
                let key = cols[usize::from(solved.spec[slot])].key;
                let id = Id::keyed(Id::keyed(tid, row as u64), u64::from(key));
                let r = Rect::new(x, y, solved.w[slot], 1);
                cell(ink, cx, r, Cell { row, key, id }, face);
            };
            if per_band {
                // **Three passes, three seeks, three copies of the bounds.** The band's view is
                // opened once for the whole band here, which is where the 4% comes from — and the
                // three [`Scan`]s are what §6 means by *cannot share the lockstep scan cursor*.
                for band in [Band::Left, Band::Scroll, Band::Right] {
                    let (blo, bhi) = match band {
                        Band::Scroll => (vlo, vhi),
                        other => solved.range(other),
                    };
                    let mut scan = Scan::seek(sel, first);
                    if band == Band::Scroll {
                        let h = u16::try_from(hi - lo).unwrap_or(u16::MAX);
                        let rect = Rect::new(band_x, lo, solved.view_w, h);
                        let mut clipped = cx.child(rect);
                        let mut view = clipped.scrolled(-rect.x, -rect.y);
                        for y in lo..hi {
                            let Ok(i) = usize::try_from(y) else { continue };
                            let face = face_of(&mut scan, i);
                            for slot in blo..bhi {
                                let x = band_x + solved.x[slot] - hoff;
                                one(ink, &mut view, slot, x, y, i, face);
                            }
                        }
                    } else {
                        let base = if band == Band::Left { 0 } else { right_x };
                        for y in lo..hi {
                            let Ok(i) = usize::try_from(y) else { continue };
                            let face = face_of(&mut scan, i);
                            for slot in blo..bhi {
                                one(ink, cx, slot, base + solved.x[slot], y, i, face);
                            }
                        }
                    }
                }
            } else {
                let mut scan = Scan::seek(sel, first);
                for y in lo..hi {
                    let Ok(i) = usize::try_from(y) else { continue };
                    let face = face_of(&mut scan, i);
                    for slot in solved.left.0..solved.left.1 {
                        one(ink, cx, slot, solved.x[slot], y, i, face);
                    }
                    {
                        let rect = Rect::new(band_x, y, solved.view_w, 1);
                        let mut clipped = cx.child(rect);
                        let mut view = clipped.scrolled(-rect.x, -rect.y);
                        for slot in vlo..vhi {
                            let x = band_x + solved.x[slot] - hoff;
                            one(ink, &mut view, slot, x, y, i, face);
                        }
                    }
                    for slot in solved.right.0..solved.right.1 {
                        one(ink, cx, slot, right_x + solved.x[slot], y, i, face);
                    }
                }
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

// ── `table` = `collection` + column rectangles ───────────────────────────────────────────────────

/// **Where a column sits when the table is scrolled sideways.**
///
/// **A pinned column may not be elastic, and it is enforced by construction rather than
/// documented**: `Left` and `Right` carry their own width, and the [`Column::width`] a pinned column
/// declares is not reachable from [`solve_columns`] at all — every reader of a pin's width goes
/// through [`Pin::width`], and there is nowhere for a [`Constraint`] to enter. §6's reason is two
/// denominators: a pin claims a share of the *table* and a scrolling lane a share of the *content*,
/// and `max(viewport, Σ minima)` is not the table.
///
/// The enforcement is a *type* and not a check, which is the difference between this and a
/// `debug_assert`: the failure the rule is about has no spelling.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pin {
    /// Pinned to the table's left edge, this many cells wide.
    Left(u16),
    /// In the scrolling band.
    None,
    /// Pinned to the table's right edge, this many cells wide.
    Right(u16),
}

impl Pin {
    /// The fixed width a pin claims, or `None` for a scrolling column.
    pub const fn width(self) -> Option<u16> {
        match self {
            Pin::Left(w) | Pin::Right(w) => Some(w),
            Pin::None => None,
        }
    }
}

/// **One declared column.**
///
/// The `key` is **stable identity**, independent of position and of visibility: hiding and
/// reordering columns are two of §6's twenty-two grid features and both move positions, so nothing
/// stored may be keyed on one. [`CellSel`] and the column half of [`TableState::editing`] are the
/// two things a table stores per column, and both take the key.
#[derive(Clone, Copy, Debug)]
pub struct Column {
    /// **Stable identity.** Not a position.
    pub key: u16,
    /// What the column is called. Written by the header when [`TableOpts::header`] asks for one.
    pub title: &'static str,
    /// **Consulted only when `pin` is [`Pin::None`].** See [`Pin`].
    pub width: Constraint,
    /// Which of the three bands it lands in.
    pub pin: Pin,
}

impl Column {
    /// A scrolling column.
    pub const fn new(key: u16, title: &'static str, width: Constraint) -> Column {
        Column {
            key,
            title,
            width,
            pin: Pin::None,
        }
    }

    /// The same column, pinned to the left edge at a fixed width.
    pub const fn pinned_left(self, w: u16) -> Column {
        Column {
            pin: Pin::Left(w),
            ..self
        }
    }

    /// The same column, pinned to the right edge at a fixed width.
    pub const fn pinned_right(self, w: u16) -> Column {
        Column {
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

/// **The most columns one declaration list may hold.**
///
/// A fixed array, because a `Vec` per table per frame is an allocation per frame and the standing
/// budget is zero. Two hundred and fifty-six is past §6's own sweep, whose widest arm is 240.
pub const MAX_COLS: usize = 256;

/// **What one frame's column solve produced.** Plain arrays: no allocation, no per-frame scratch.
///
/// Built by [`solve_columns`] **once a frame, over the declared columns, touching no row** — §6's
/// sentence, and the one [`table`] is measured against. The three bands are three ranges into the
/// same arrays rather than three vectors, for the same reason.
#[derive(Clone)]
pub struct Solved {
    /// How many columns were placed.
    pub n: usize,
    /// Index into the caller's `&[Column]`, so `key` and `title` are one hop away.
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
    /// **The scrolling band's content width, `max(view_w, Σ minima)`** — the one width in a table
    /// that is not a viewport, and the denominator §6's two-denominator sentence is about.
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

    /// One band's `[lo, hi)`.
    pub const fn range(&self, b: Band) -> (usize, usize) {
        match b {
            Band::Left => self.left,
            Band::Scroll => self.scroll,
            Band::Right => self.right,
        }
    }
}

/// The floor a constraint shrinks to when the band overflows its viewport.
const fn col_floor(c: Constraint) -> u16 {
    match c {
        Constraint::Fixed(v) | Constraint::Min(v) | Constraint::Max(v) => v,
        Constraint::Percent(_) | Constraint::Ratio(_, _) | Constraint::Weight(_) => 6,
    }
}

/// **The whole of a table's layout. Runs once a frame, touches no row, allocates nothing.**
///
/// The three bands are solved separately and that is §6's decision rather than an implementation
/// detail: a pin is a *band*, so it is a rect split, and the scrolling columns then have exactly one
/// denominator. The band handed to R03's solver is `content_w` and not `view_w` — when the columns
/// fit the two are the same and the elastic ones share the viewport; when they do not, the band is
/// Σ minima and the difference is what a horizontal offset scrolls over.
///
/// Columns past [`MAX_COLS`] are dropped rather than panicking: the caller of a component is an
/// application, and [`Solved::n`] says how many were placed.
pub fn solve_columns(area_w: u16, specs: &[Column]) -> Solved {
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
        if n == MAX_COLS {
            break;
        }
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
        min_sum += u32::from(col_floor(s.width));
        m += 1;
    }
    let content_w = min_sum.max(u32::from(out.view_w)).min(u32::from(u16::MAX));
    out.content_w = content_w as u16;

    let mut rects = [Rect::default(); MAX_COLS];
    let (k, _) = solve(content_w, &spec_buf[..m], &mut rects[..m]);
    out.scroll.0 = n;
    let mut bx = 0i32;
    for (j, rect) in rects[..k].iter().enumerate() {
        if n == MAX_COLS {
            break;
        }
        out.spec[n] = idx_buf[j];
        out.band[n] = Band::Scroll;
        out.x[n] = bx;
        out.w[n] = rect.w;
        bx += i32::from(rect.w);
        n += 1;
    }
    out.scroll.1 = n;

    out.right.0 = n;
    let mut rx = 0i32;
    for (i, s) in specs.iter().enumerate() {
        if n == MAX_COLS {
            break;
        }
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

/// **The column half of the invariant**: `[lo, hi)` into the scrolling band — the columns a viewport
/// `view_w` wide at horizontal offset `hoff` can show, and nothing else.
///
/// Binary search rather than a walk, because the walk stays correct while the declared column count
/// grows and the cost grows with it. At forty columns the two are indistinguishable; the point of
/// writing the search is that at two hundred and forty they are not.
///
/// **This is the fast path and [`Ctx::visible_cols`](vitui_runtime::Ctx::visible_cols) is its
/// oracle**, not the other way round — the arithmetic band opens no view, so there is nothing for it
/// to ask. `crate::grid`'s sweep over every offset the content admits is where the two are held to
/// each other.
pub fn visible_columns(s: &Solved, hoff: i32) -> (usize, usize) {
    let (lo0, hi0) = s.scroll;
    if hi0 <= lo0 || s.view_w == 0 {
        return (lo0, lo0);
    }
    let left = hoff;
    let right = hoff + i32::from(s.view_w);
    let lo = col_partition(lo0, hi0, |i| s.x[i] + i32::from(s.w[i]) <= left);
    let hi = col_partition(lo, hi0, |i| s.x[i] < right);
    (lo, hi)
}

/// The first index in `[lo, hi)` for which `pred` is false. `pred` must be monotone.
fn col_partition(lo: usize, hi: usize, pred: impl Fn(usize) -> bool) -> usize {
    let (mut a, mut b) = (lo, hi);
    while a < b {
        let mid = a + (b - a) / 2;
        if pred(mid) { a = mid + 1 } else { b = mid }
    }
    a
}

// ── cell selection: a run list per column key ────────────────────────────────────────────────────

/// **Cell selection: one [`Selection`] per column key, and no second store.**
///
/// §6's decision, and it is decided **entirely by what a gesture costs** — the frame does not
/// distinguish the two candidates at all, because both are amortised `O(1)` a cell through the same
/// [`Scan`] the row loop already runs.
///
/// The alternative is a flattened `row · ncols + col` index into one [`Selection`], which is
/// [`flattened_header_click`] and is kept runnable rather than remembered: one click on a column
/// header at a million rows is **one run and [`SPAN_BYTES`] bytes here** against a million runs and
/// sixteen megabytes there — *C03's rejected `HashSet` arriving as a different type*, because a
/// column of a row-major flattening is a stride and a stride of length one is not a run.
///
/// **The losses are real and they are bounded by the declared column count**: select-all and
/// select-one-row are one run under the flattening and one run *per column* here. A table's row
/// count is unbounded and its column count is not, which is the whole of why the trade goes this
/// way.
///
/// It is not a second selection store: the entries are [`Selection`] — the same span list
/// [`CollState`] keeps, queried through the same [`Scan`].
#[derive(Clone, Default, PartialEq, Eq, Debug)]
pub struct CellSel {
    /// Sorted by key, so a lookup is a binary search and the ordering is not the caller's problem.
    by_key: Vec<(u16, Selection)>,
}

impl CellSel {
    /// Nothing selected, in no column.
    pub fn new() -> CellSel {
        CellSel::default()
    }

    /// The rows selected in `key`, if the column has an entry.
    pub fn column(&self, key: u16) -> Option<&Selection> {
        let i = self.by_key.binary_search_by_key(&key, |(k, _)| *k).ok()?;
        Some(&self.by_key[i].1)
    }

    /// The rows selected in `key`, minting the column's entry if it has none.
    pub fn column_mut(&mut self, key: u16) -> &mut Selection {
        match self.by_key.binary_search_by_key(&key, |(k, _)| *k) {
            Ok(i) => &mut self.by_key[i].1,
            Err(i) => {
                self.by_key.insert(i, (key, Selection::new()));
                &mut self.by_key[i].1
            }
        }
    }

    /// Whether `(row, key)` is selected.
    pub fn contains(&self, row: usize, key: u16) -> bool {
        self.column(key).is_some_and(|s| s.contains(row))
    }

    /// **Select every row of one column. §6's header click**, and the gesture the store was chosen
    /// for: one run and [`SPAN_BYTES`] bytes at any length.
    pub fn select_column(&mut self, key: u16, len: usize) {
        self.column_mut(key).select_all(len);
    }

    /// **Select one row across `keys`. The gesture this store is worse at**, and the loss is bounded
    /// by the declared column count rather than by the length.
    pub fn select_row(&mut self, keys: &[u16], row: usize) {
        for &k in keys {
            self.column_mut(k).select_only(row);
        }
    }

    /// Runs held, over every column. The counter §6's 1 000 000-against-1 is stated in.
    pub fn span_count(&self) -> usize {
        self.by_key.iter().map(|(_, s)| s.span_count()).sum()
    }

    /// Bytes the spans hold, over every column.
    pub fn bytes(&self) -> usize {
        self.by_key.iter().map(|(_, s)| s.bytes()).sum()
    }

    /// Columns with an entry.
    pub fn columns(&self) -> usize {
        self.by_key.len()
    }

    /// Nothing selected anywhere. **Every span here is a row position**, so this is what a revision
    /// change costs the cell store — see [`table`].
    pub fn clear(&mut self) {
        self.by_key.clear();
    }
}

/// **The flattened spelling, kept runnable: one header click under `row · ncols + col`.**
///
/// Returns `(runs, bytes)`. A column of a row-major flattening is a stride, and a stride of length
/// one is a run of length one — so selecting a column of `rows` rows is `rows` runs, which is what
/// §6 prices at 1 000 000 and 16 MB against [`CellSel::select_column`]'s one and sixteen.
///
/// A function rather than a variant of [`CellSel`], for [`stores::Alt`]'s reason: a refused store
/// kept as an arm of the shipped one is a shipped store somebody will reach for.
pub fn flattened_header_click(rows: usize, ncols: usize, col: usize) -> (usize, usize) {
    let mut sel = Selection::new();
    for r in 0..rows {
        let at = r * ncols + col;
        sel.insert(at, at + 1);
    }
    (sel.span_count(), sel.bytes())
}

/// **Which cell this is, and the id it may declare a target under.**
///
/// # The id is handed over rather than pushed, and that is a measured workaround
///
/// ADR 0027's rule is *a container roots its children inside its own id*, and §4 says `with_key` is
/// the only verb that mints one. **It is not available here**, and the reason is a defect one crate
/// down rather than a preference: [`Ctx::with_id`](vitui_runtime::Ctx::with_id) — which
/// `Ctx::with_key` is written on — re-childs the view at `self.area()`, and `area()` is
/// `Rect::new(0, 0, w, h)` in the *current* coordinate system. Inside a scroll scope that origin is
/// the **content's**, so the clip it intersects with is content rows `0..h` while the window is at
/// the offset: at any offset past the first screenful the two do not overlap and a cell keyed that
/// way **draws nothing at all**.
///
/// [`collection`] states the same fact from one level up — it is why the row loop's `cx.with_id` is
/// outside the scope rather than around the loop — and `tests::a_with_key_inside_a_scroll_scope_/// draws_nothing_past_the_first_screenful` is it as a measurement rather than as an argument.
/// Filed as `.scratch/vitui-runtime-architecture/issues/31`.
///
/// So the container mints with [`Id::keyed`](vitui_runtime::Id::keyed) — the same arithmetic
/// `with_key` performs — and hands the result down. §4's *every workaround on this map that looks
/// like a hack is the id being opaque* is the same sentence one axis over: the value is a hash, so
/// **handing it over is the only way to root a child whose context cannot be re-clipped**.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Cell {
    /// The content row.
    pub row: usize,
    /// The column's **key**, never its position.
    pub key: u16,
    /// **The id this cell may declare a target under**, rooted in the table's own id.
    ///
    /// `keyed(keyed(table, row), key)` in the shipped build, and `keyed(table, row)` in
    /// [`defective::row_keyed_cells`] — where every cell of a row is one widget, the first
    /// claimant wins, and `merges` is the only counter that says so.
    pub id: Id,
}

// ── the table's state and options ────────────────────────────────────────────────────────────────

/// **Everything a table keeps across frames, and it is [`CollState`] plus three fields.**
///
/// The row axis — the offset, the row selection, the type-ahead buffer, the editing row and the
/// revision — is [`CollState`] and is not duplicated here. What a table adds is the horizontal
/// offset, the per-column cell selection, and the **column half** of the editing slot.
///
/// # The editing slot is `Option<(row, col)>`, stored as its two halves
///
/// §6: *`Option<(row, col)>` and no wider — 0.29 µs to follow a million-row sort, because it is one
/// position. The row half is revalidated against the order's revision, which is a field beside the
/// slot rather than a widening of it. The column half needs nothing, because it is a **key** and not
/// a position.*
///
/// The row half is [`CollState::editing`] — the one slot ADR 0028 allows, and the one [`collection`]
/// already clears when the revision it was stamped at stops matching, so the revalidation is
/// inherited rather than written twice. The column half is here, and it is a `u16` rather than an
/// `Option<u16>` because it means nothing while the row half is `None`: two `Option`s would admit a
/// state — a column with no row — that [`TableState::editing`] would then have to invent an answer
/// for.
///
/// # The horizontal offset is the caller's, and the asymmetry is deliberate
///
/// The vertical offset is [`collection`]'s, applied from **this frame's** wheel inside the draw.
/// [`TableState::hoff`] is not: the column solve runs *before* the row pass, so a horizontal wheel
/// read where the vertical one is read would be applied a frame after it arrived — which is exactly
/// the defect runtime ticket 14 found and removed. Until a table can read the wheel before it
/// solves, the honest shape is the one `CONTEXT.md` states for every offset: *the application owns
/// it*.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct TableState {
    /// The row axis. §6's `+` is on the other one.
    pub coll: CollState,
    /// **The first visible content column of the scrolling band**, clamped by [`table`] to what the
    /// solve admits.
    pub hoff: i32,
    /// Cell selection, one run list per column key.
    pub cells: CellSel,
    /// The column half of the editing slot. Meaningless while [`CollState::editing`] is `None`.
    edit_col: u16,
}

impl TableState {
    /// A table at the top left of its content with nothing selected.
    pub fn new() -> TableState {
        TableState::default()
    }

    /// **The cell being edited, as §6's `Option<(row, col)>`.**
    ///
    /// The row half comes back through [`CollState::editing`], so a reorder the caller did not
    /// reconcile has already closed the editor by the time this is asked.
    pub const fn editing(&self) -> Option<(usize, u16)> {
        match self.coll.editing {
            Some(row) => Some((row, self.edit_col)),
            None => None,
        }
    }

    /// Open the inline editor on `(row, key)`.
    pub const fn edit(&mut self, row: usize, key: u16) {
        self.coll.editing = Some(row);
        self.edit_col = key;
    }

    /// Close the inline editor.
    pub const fn stop_editing(&mut self) {
        self.coll.editing = None;
    }

    /// **Carry the editing slot across a reorder — §6's 0.29 µs, as one lookup.**
    ///
    /// `to` answers *where did the row that was at `i` go*, and it is the caller's because only the
    /// caller has both orders (ADR 0031). The cost is one call at any length **because the slot is
    /// one position**; the column half is untouched, because it is a key.
    pub fn follow(&mut self, to: impl FnOnce(usize) -> Option<usize>) {
        if let Some(row) = self.coll.editing {
            self.coll.editing = to(row);
        }
    }
}

/// [`table`]'s options. Spec §1's rule 3: a `Default` struct, never a required builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct TableOpts {
    /// The row axis's options, unchanged. **No second [`Mode`].**
    pub coll: CollOpts,
    /// Whether to write a header row of [`Column::title`]s above the body.
    ///
    /// **Off by default, and the default is a measurement decision rather than a taste one** — see
    /// [`table`]'s *the header* section.
    pub header: bool,
}

// ── the component ────────────────────────────────────────────────────────────────────────────────

/// **`table` — [`collection`] plus a column rect split, and the `+` is paid in verbs.**
///
/// **Hostile axes:** `scrolled`, `shrunk`, `wheeled`, `narrow`.
///
/// Scene 7 is its own — a twelve-column table under a horizontal offset, both edges pinned — and the
/// other three arrive with the store, because a table is [`collection`] and a column split.
///
/// Spec §6, ADR 0028. Spec §1's shape exactly: `fn(&mut Ctx, Rect, …) -> Response`.
///
/// # It is `collection`, and the sentence is checkable rather than decorative
///
/// There is **no second selection store** — [`CellSel`] holds the same [`Selection`] the row axis
/// holds — **no second scan cursor**, because the row's [`Face`] arrives from `collection`'s own
/// lockstep [`Scan`] — and **no second [`Mode`]**. The row axis, the wheel, the keyboard, the
/// type-ahead, the reveal, the tail below the content and the revision check are all `collection`'s,
/// reached by calling it. What this function adds is [`solve_columns`] once a frame and a cell loop
/// inside the row drawer.
///
/// # The `+` is paid in verbs
///
/// Damage is marked once per verb, and a table cuts each row into one short run per column where a
/// list writes one long one — then §2's partition rule makes each column **two** verbs, the text and
/// then the padding after it, because the alternative is a cell written twice. **Two verbs a cell is
/// the floor**, and it is a floor rather than an equality: a cell whose own text fills its column
/// has no padding and costs one.
///
/// # Three bands, each a view, opened per row inside one row pass
///
/// A band is a rect split, so the clip is what makes it one: a cell that writes past its own column
/// cannot reach the band beside it, whatever the caller's cell drawer does.
/// [`Ctx::child`](vitui_runtime::Ctx::child) translates as well as clips, and a band needs only the
/// clip — so each view is immediately scrolled back by its own origin, which is what keeps every
/// cell of the row in **one** coordinate system. The pair `writes` against `distinct` is
/// meaningless across two.
///
/// That also leaves this build and [`defective::arithmetic_band`] writing the same arguments at the
/// same call sites with one `child` between them, and the arithmetic one lets the band's edge column
/// overwrite the pinned band: identical verbs, identical time, and a band of cells re-damaged on
/// every steady frame for ever.
///
/// §6 also refuses a spelling that is **correct**: one pass per band instead of one pass per row.
/// [`defective::per_band_pass`] is that build with [`defective::one_row_pass`] beside it, one value
/// apart, so §6's *4% cheaper and refused* is a measurement rather than a memory — and **the 4%
/// does not reproduce.** Over sixty interleaved rounds the difference is inside ±1.5% and its sign
/// flips between runs; the saving (one band view for the whole window instead of one a row) and its
/// price (three row loops, three `Scan::seek`s) cancel on a screen whose pins are three columns of
/// eleven. The refusal stands untouched, because §6 refuses the spelling on an argument and not on
/// a timing: three loops share nothing but the author writing the same bounds three times, and
/// cannot share §5's lockstep scan cursor at all.
///
/// # Identity is per cell, and only where a cell declares a target
///
/// Each row is keyed by its index and each cell by its column **key**, so a cell that calls
/// [`Ctx::interact`](vitui_runtime::Ctx::interact) gets an id of its own. Keying per row alone is
/// cheaper and correct **until a cell declares a target**, and then every cell of a row is one
/// widget: [`defective::row_keyed_cells`] is that build and `merges` is the counter that sees it.
/// Keying costs no region — a table declares **one** hit entry, `collection`'s, however many cells
/// are on screen.
///
/// # The header, and what cannot be measured while one is drawn
///
/// [`TableOpts::header`] writes [`Column::title`] into the same column rectangles, one row above the
/// body, through the same three bands and the same window — so a column's title and its cells cannot
/// disagree about where the column is. It is **off by default** because a header moves the body's
/// view off the table's own row 0 and the body draws inside a scroll scope, so the header's cells and
/// the body's cells land in two coordinate spaces and neither [`crate::counters::Tally`] nor
/// [`crate::runner::Pen`] can union them. That is components ticket 14's finding, inherited
/// unchanged: a limit of the recorders and not of the component, and `Ctx` publishes no accessor for
/// the frame's own origin that would lift it.
///
/// # Arguments
///
/// `cell` is handed `(cx, rect, row, key, face)` and **owes every cell of the rectangle** (§2). The
/// `Face` is the row's, because §5's five independent bits are a row's bits; a cell that wants the
/// cell selection reads [`TableState::cells`], which is the caller's to consult and the caller's to
/// gesture on.
///
/// ```
/// use vitui_components::collect::{Column, TableOpts, TableState, table};
/// use vitui_components::frame::face_paint;
/// use vitui_components::order::Rows;
/// use vitui_runtime::ctx::Driver;
/// use vitui_runtime::layout::Constraint;
///
/// let cols = [
///     Column::new(0, "id", Constraint::Fixed(4)).pinned_left(4),
///     Column::new(1, "name", Constraint::Weight(1)),
/// ];
/// let mut st = TableState::new();
/// let mut driver = Driver::headless(20, 3).expect("a sink attaches");
/// driver.frame(|cx| {
///     let area = cx.area();
///     let opts = TableOpts::default();
///     let _ = table(
///         cx,
///         area,
///         &mut st,
///         &opts,
///         &cols,
///         Rows::of(3),
///         &mut |_buf, _range| None,
///         &mut |cx, r, c, face| {
///             let paint = face_paint(cx.theme(), face);
///             let text = if c.key == 0 { "id" } else { "name" };
///             cx.text(r.x, r.y, text, paint);
///             let _ = c.row;
///         },
///     );
/// });
/// ```
#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "`collection`'s seven with the column list added, which is exactly what §6's \
              `table = collection + column rectangles` says the signature is. Folding any of them \
              into a parameter struct would put the difference between the two components \
              somewhere a reader cannot see it"
)]
pub fn table(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut TableState,
    opts: &TableOpts,
    cols: &[Column],
    rows: Rows,
    find: &mut dyn FnMut(&str, Range<usize>) -> Option<usize>,
    cell: &mut dyn FnMut(&mut Ctx<'_, '_>, Rect, Cell, Face),
) -> Response {
    table_into(
        &mut Direct,
        cx,
        area,
        st,
        opts,
        cols,
        rows,
        find,
        |_ink, cx, r, c, face| cell(cx, r, c, face),
    )
}

/// **[`table`], drawing through an [`Ink`] so an instrument can see every cell.**
///
/// The entry point a gate takes; [`table`] is this with [`Direct`]. See [`crate::ink`] for why the
/// seam exists rather than a second implementation written against a `Tally`.
#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "`table`'s own eight plus the `Ink` seam's writer, which is `collection_into`'s shape \
              one component down"
)]
pub fn table_into<I, F, C>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut TableState,
    opts: &TableOpts,
    cols: &[Column],
    rows: Rows,
    find: F,
    cell: C,
) -> Response
where
    I: Ink,
    F: FnMut(&str, Range<usize>) -> Option<usize>,
    C: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, Cell, Face),
{
    table_with(
        ink,
        cx,
        area,
        st,
        opts,
        cols,
        rows,
        find,
        cell,
        TableShape::default(),
    )
}

/// **The three ways `table = collection + column rectangles` can be false**, as one value.
///
/// One struct rather than three booleans in a signature, so that a reviewer's diff between the
/// shipped build and any refused one is a single field. Every one of them **compiles, renders almost
/// right, and passes at least one gate the correct build passes** — which is the property
/// [`defective`] exists for.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct TableShape {
    /// How the scrolling band reaches the surface.
    band: BandShape,
    /// Whether the column window is asked for.
    cols: ColVirt,
    /// Whether a cell gets a key of its own.
    keys: CellKeys,
    /// The sign of the horizontal offset.
    hsign: HSign,
}

/// How the scrolling band reaches the surface.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum BandShape {
    /// **The rule.** A view per row: a column at the viewport's edge is clipped, and the cells past
    /// the edge belong to the pinned band.
    #[default]
    View,
    /// **The defect.** No view, the offset applied as arithmetic.
    Arithmetic,
}

/// Whether the column window is asked for or left to the clip.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum ColVirt {
    /// **The rule.** Only the columns the viewport admits are drawn.
    #[default]
    Virtualised,
    /// **The defect.** Every declared column is drawn and the clip discards what does not fit.
    ClipOnly,
}

/// Whether a cell gets a key of its own.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum CellKeys {
    /// **The rule.** The row key, then the column key.
    #[default]
    PerCell,
    /// **The defect.** The row key alone.
    PerRow,
}

/// The sign of the horizontal offset — §3.3's trap on the column axis.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
enum HSign {
    /// **The rule.** The content moves the other way from the offset.
    #[default]
    Plus,
    /// **The defect.** The offset added where it should be subtracted. The band shows a partly blank
    /// stretch, issues every verb it would have issued, and measures as an improvement because the
    /// clip eats the writes.
    Minus,
}

/// **`#[track_caller]` all the way down**, for [`draw_with`]'s reason: `Ctx::id` mints from
/// `Location::caller()`, and an attribute that stops one frame short of the call makes every table
/// in the application one table.
#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "`table_into`'s nine plus the shape that separates the correct build from the four \
              refused ones. Splitting it would put the defects in a second function where a \
              reviewer's diff could not be one line"
)]
fn table_with<I, F, C>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut TableState,
    opts: &TableOpts,
    cols: &[Column],
    rows: Rows,
    find: F,
    mut cell: C,
    shape: TableShape,
) -> Response
where
    I: Ink,
    F: FnMut(&str, Range<usize>) -> Option<usize>,
    C: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, Cell, Face),
{
    // **The table's own id, taken outside every closure** (ADR 0027). `#[track_caller]` all the
    // way down makes it the *application's* call site, so two tables on one screen are two tables;
    // every cell's id is minted from it and handed over — see [`Cell::id`] for why it is minted
    // rather than pushed.
    let tid = cx.id();
    // **Once a frame, over the declared columns, touching no row** (§6). Above the row loop and
    // above `collection`, so there is nowhere for a row to reach it.
    let solved = solve_columns(area.w, cols);
    st.hoff = st.hoff.clamp(0, solved.max_hoff());
    let hoff = st.hoff;
    let signed = match shape.hsign {
        HSign::Plus => hoff,
        HSign::Minus => -hoff,
    };
    let (vlo, vhi) = match shape.cols {
        ColVirt::Virtualised => visible_columns(&solved, hoff),
        ColVirt::ClipOnly => solved.scroll,
    };

    // **Every span in the cell store is a row position**, so the rule ADR 0031 states for the row
    // axis is the same rule here: an order the caller has not reconciled invalidates them. The
    // comparison is `collection`'s own and is read *before* the call, because `collection` stamps
    // the revision on its way through and there is nothing left to notice afterwards.
    let held = st.coll.revision();
    if held.is_known() && rows.rev.is_known() && held != rows.rev {
        st.cells.clear();
    }

    // The header, and the body's rectangle beneath it. §6's one `cut`.
    let body = if opts.header {
        let (head, rest) = split_header(area);
        header_row(ink, cx, &solved, cols, head, signed, (vlo, vhi));
        rest
    } else {
        area
    };

    collection_into(
        ink,
        cx,
        body,
        &mut st.coll,
        &opts.coll,
        rows,
        find,
        |ink, cx, r, row, face| {
            let row_id = Id::keyed(tid, row as u64);
            let mut draw = |ink: &mut I, cx: &mut Ctx<'_, '_>, slot: usize, x: i32| {
                let key = cols[usize::from(solved.spec[slot])].key;
                let rect = Rect::new(x, r.y, solved.w[slot], 1);
                let id = match shape.keys {
                    CellKeys::PerCell => Id::keyed(row_id, u64::from(key)),
                    CellKeys::PerRow => row_id,
                };
                cell(ink, cx, rect, Cell { row, key, id }, face);
            };
            // **One row pass, three bands, and each band is a view.** A band is a rect split, so
            // the clip is what makes it one: a cell that writes past its own column cannot reach
            // the band beside it, whatever the caller's cell drawer does.
            //
            // [`Ctx::child`](vitui_runtime::Ctx::child) translates as well as clips and the band
            // needs only the clip, so each view is immediately scrolled back by its own origin.
            // That is what keeps every cell of the row in **one** coordinate system — the pair
            // `writes` against `distinct` is meaningless across two — and it is what leaves this
            // build and [`defective::arithmetic_band`] writing the same arguments at the same call
            // sites with one `child` between them.
            let band_x = i32::from(solved.left_w);
            let right_x = band_x + i32::from(solved.view_w);
            let in_band =
                |ink: &mut I,
                 cx: &mut Ctx<'_, '_>,
                 at: i32,
                 w: u16,
                 range: (usize, usize),
                 shift: i32,
                 draw: &mut dyn FnMut(&mut I, &mut Ctx<'_, '_>, usize, i32)| {
                    let rect = Rect::new(at, r.y, w, 1);
                    let mut clipped = cx.child(rect);
                    let mut view = clipped.scrolled(-rect.x, -rect.y);
                    for slot in range.0..range.1 {
                        draw(ink, &mut view, slot, at + solved.x[slot] - shift);
                    }
                };
            in_band(ink, cx, 0, solved.left_w, solved.left, 0, &mut draw);
            match shape.band {
                BandShape::View => {
                    in_band(
                        ink,
                        cx,
                        band_x,
                        solved.view_w,
                        (vlo, vhi),
                        signed,
                        &mut draw,
                    );
                }
                // **The defect, and it is one `child` and nothing else.** Written straight into the
                // row's context, the band's edge column reaches past the viewport and overwrites
                // the pinned band that follows it.
                BandShape::Arithmetic => {
                    for slot in vlo..vhi {
                        draw(ink, cx, slot, band_x + solved.x[slot] - signed);
                    }
                }
            }
            in_band(ink, cx, right_x, solved.right_w, solved.right, 0, &mut draw);
        },
    )
}

/// The header's row and the body's rectangle beneath it. §6's one `cut`.
fn split_header(area: Rect) -> (Rect, Rect) {
    let h = area.h.min(1);
    (
        Rect::new(area.x, area.y, area.w, h),
        Rect::new(area.x, area.y + i32::from(h), area.w, area.h - h),
    )
}

/// **The header: [`Column::title`] in the same column rectangles, one row above the body.**
///
/// The same three bands and the same window, so a column's title and its cells cannot disagree about
/// where the column is. It writes a partition of `head` — the title, then the padding after it — for
/// the same reason a cell does.
fn header_row<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    s: &Solved,
    cols: &[Column],
    head: Rect,
    signed: i32,
    window: (usize, usize),
) {
    if head.is_empty() {
        return;
    }
    let paint = cx.theme().paint(Role::Title);
    let y = head.y;
    // **`head.x`, and leaving it out was a real defect.** The body draws inside
    // `Ctx::scroll_scope`, which childs at the body's rectangle — so a body cell's `x` is relative
    // to the table while a header cell's is relative to the **caller's** context. A table handed
    // the interior of a panel then drew its header one column into the border. Found by
    // `vitui-apps`'s `ledger`, which is the first thing to put a table inside anything.
    let base = head.x;
    let mut write = |ink: &mut I, cx: &mut Ctx<'_, '_>, slot: usize, x: i32| {
        let title = cols[usize::from(s.spec[slot])].title;
        let cut = truncate(title, s.w[slot]);
        let used = width(cut);
        let _ = ink.text(cx, x, y, cut, paint);
        let _ = ink.run(cx, x + i32::from(used), y, " ", s.w[slot] - used, paint);
    };
    let band_x = i32::from(s.left_w);
    let right_x = band_x + i32::from(s.view_w);
    // Three views, the same three the body opens, so a column's title and its cells cannot
    // disagree about where the column is or about what clips it.
    let in_band = |ink: &mut I,
                   cx: &mut Ctx<'_, '_>,
                   at: i32,
                   w: u16,
                   range: (usize, usize),
                   shift: i32,
                   write: &mut dyn FnMut(&mut I, &mut Ctx<'_, '_>, usize, i32)| {
        let rect = Rect::new(at, y, w, 1);
        let mut clipped = cx.child(rect);
        let mut view = clipped.scrolled(-rect.x, -rect.y);
        for slot in range.0..range.1 {
            write(ink, &mut view, slot, at + s.x[slot] - shift);
        }
    };
    in_band(ink, cx, base, s.left_w, s.left, 0, &mut write);
    in_band(ink, cx, base + band_x, s.view_w, window, signed, &mut write);
    in_band(ink, cx, base + right_x, s.right_w, s.right, 0, &mut write);
}

// ── `tree` = `collection` + a flatten index ──────────────────────────────────────────────────────

/// **Whether the indent is clamped, and this is the whole of §7's negative case.**
///
/// One type and not two: `crate::forest` used to declare its own while `tree` did not exist, and two
/// of these in one crate makes every mismatch read *expected `Indent`, found `Indent`* — which is
/// the collision [`crate::frame`] refused to mint when it named its rectangle `Cells`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Indent {
    /// **The rule.** `min(depth * 2, w - 2)` — `O(1)` in the depth, which is what lets a row *read*
    /// a depth without being proportional to one.
    #[default]
    Clamped,
    /// **The defect.** The indent is the depth, and the depth is data.
    Unclamped,
}

impl Indent {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Indent::Clamped => "clamped",
            Indent::Unclamped => "unclamped",
        }
    }
}

/// **How many columns a row at `depth` indents by, in a `w`-column rectangle. The one decision.**
///
/// [`tree`] calls it and so does every instrument that wants to know what the component asked for,
/// which is what keeps the two from being a model and a copy of a model: the number the screen sums
/// and the number the run is drawn with come out of this function.
///
/// **A `usize` and not a `u16`**, which is the arithmetic §7's own prototype got wrong: `depth * 2`
/// at 59 999 is 119 998, and `as u16` makes that 54 462 — a truncation that reads as a clamp.
///
/// The clamp reserves two columns, which is the chevron and one cell of label. A row whose indent
/// has eaten its rectangle draws the indent and stops, and [`tree`] never calls the row drawer for
/// it — which is observable and is how the unclamped arm is caught.
pub fn indent_columns(indent: Indent, depth: u16, w: u16) -> usize {
    let raw = usize::from(depth) * 2;
    match indent {
        Indent::Clamped => raw.min(usize::from(w.saturating_sub(2))),
        Indent::Unclamped => raw,
    }
}

/// **Whether display row `i` has children in the index, in `O(1)`.**
///
/// The next row is deeper or it is not. Asking [`Order::descendants`] instead would be correct and
/// would make a row's cost proportional to its subtree — 349 524 rows walked to decide one chevron
/// — which is the same class of defect as the unclamped indent, on the other axis. The index is
/// `O(1)` per row and this is what keeps it so.
pub fn has_children(index: &Order, i: usize) -> bool {
    match (index.at(i), index.at(i + 1)) {
        (Some(a), Some(b)) => b.depth > a.depth,
        _ => false,
    }
}

/// **One row of a tree, as the row drawer sees it.** [`Cell`]'s twin one component over.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Node {
    /// The display position, which is where the index was read.
    pub row: usize,
    /// The caller's own key for it. Never an [`Id`].
    pub node: u32,
    /// How deep it sits.
    pub depth: u16,
    /// Whether its subtree has been taken out of the index ([`Entry::FOLDED`](crate::order::Entry::FOLDED)).
    pub folded: bool,
    /// Whether it has no children at all, which is not the same as folded.
    pub leaf: bool,
    /// **The id this row may declare a target under**, rooted in the tree's own id.
    ///
    /// `Id::keyed(tree, node)` — the **caller's key** and not the display position, because a fold
    /// above this row moves the position and does not move the row. See [`Cell::id`] for why it is
    /// minted and handed over rather than pushed with `Ctx::with_key`.
    pub id: Id,
}

/// **Everything a tree keeps across frames: [`CollState`] and the one-slot request.**
///
/// There is no fold set here, and that is ADR 0031 rather than an omission — *the order is the
/// caller's and a component may only ask*. What is folded is [`Entry::FOLDED`](crate::order::Entry::FOLDED) in the caller's own
/// index, and the way it changes is that the caller drains [`TreeState::ask`] after the draw and
/// calls [`Order::fold`].
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct TreeState {
    /// The row axis, unchanged. §7's `+` is the indent and the chevron.
    pub coll: CollState,
    /// **What the last frame asked the caller to do to the index.** Drained after the draw.
    pub ask: Asked,
}

impl TreeState {
    /// A tree at the top of its index with nothing selected and nothing asked.
    pub fn new() -> TreeState {
        TreeState::default()
    }
}

/// [`tree`]'s options. Spec §1's rule 3: a `Default` struct, never a required builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TreeOpts {
    /// The row axis's options, unchanged. **No second [`Mode`].**
    pub coll: CollOpts,
    /// The role the indent run and the chevron are painted in.
    pub furniture: Role,
}

impl Default for TreeOpts {
    fn default() -> TreeOpts {
        TreeOpts {
            coll: CollOpts::default(),
            furniture: Role::Body,
        }
    }
}

/// **`tree` — [`collection`] plus a flatten index, and the `+` costs two verbs a row.**
///
/// **Hostile axes:** `scrolled`, `shrunk`, `wheeled`, `narrow`.
///
/// Scenes 8 and 9: a million-node forest at depth 59 999 windowed by the flatten index, and a fold
/// and an unfold, which is content shrinking inside a rectangle that does not move.
///
/// Spec §7, ADR 0028, ADR 0031. Spec §1's shape exactly: `fn(&mut Ctx, Rect, …) -> Response`.
///
/// # It is `collection`, and the sentence is checkable rather than decorative
///
/// `tree_with` calls `collection_chorded`, which is [`collection`] with one parameter. There is
/// **no second selection store**, **no second scan cursor** — the row's [`Face`] arrives from
/// `collection`'s own lockstep [`Scan`] — **no second [`Mode`]**, **no second offset** and **no
/// second press edge** ([`CollState::press_edge`]). The row axis, the wheel, the keyboard, the
/// type-ahead, the reveal, the tail below the content and the revision check are all `collection`'s,
/// reached by calling it. What this function adds is two verbs a row and a one-slot request.
///
/// # The `+` is two verbs a row: the indent run and the chevron cell
///
/// 222 verbs over 111 rows (§7). The indent is **one** [`Ink::run`] whatever its width — a padding
/// band is exactly the verb a run is — and the chevron is one cell, a space on a leaf so that the
/// partition is the same partition on every row. What is left of the rectangle goes to the row
/// drawer, which owes every cell of it (§2, ADR 0026).
///
/// # The index is the caller's, and this component may only ask
///
/// `index` arrives by **shared** reference and there is no verb here that edits it. `←` collapses
/// the cursor's subtree and `→` expands it, and both leave an [`Ask`] in [`TreeState::ask`] which
/// the caller drains **after** the draw and answers with [`Order::fold`] or [`Order::unfold`]. Two
/// things force the arrangement rather than one: a `Vec::splice` allocates against a frame budget of
/// zero, and taking `&mut` to the index while the draw holds it shared is `E0502` — see [`Asked`],
/// which carries that as a compile outcome.
///
/// # `←` and `→` have to be read before `collection` sees them
///
/// [`crate::nav::step`] reads them as `↑` and `↓`, and `Ctx::decline` ends the level's turn at the
/// key queue — so neither a loop before this call nor one after it can work. The hook is `Refusal`,
/// a parameter of `collection`'s own drain loop, and that is a fact about the runtime rather than a
/// preference.
///
/// # Arguments
///
/// `row` is handed `(cx, rect, Node, Face)`. The `Face` is `collection`'s five independent bits;
/// the tree's own two facts — folded, leaf — are on the [`Node`], because they are facts about the
/// caller's index rather than about the selection.
///
/// ```
/// use vitui_components::collect::{TreeOpts, TreeState, tree};
/// use vitui_components::frame::face_paint;
/// use vitui_components::order::{Entry, Order};
/// use vitui_runtime::ctx::Driver;
///
/// // A root with two children, in pre-order display coordinates.
/// let index = Order::built(vec![
///     Entry::of(0),
///     Entry::of(1).at_depth(1),
///     Entry::of(2).at_depth(1),
/// ]);
/// let mut st = TreeState::new();
/// let mut driver = Driver::headless(20, 3).expect("a sink attaches");
/// driver.frame(|cx| {
///     let area = cx.area();
///     let opts = TreeOpts::default();
///     let _ = tree(
///         cx,
///         area,
///         &mut st,
///         &opts,
///         &index,
///         &mut |_buf, _range| None,
///         &mut |cx, r, n, face| {
///             let paint = face_paint(cx.theme(), face);
///             cx.text(r.x, r.y, if n.leaf { "leaf" } else { "root" }, paint);
///         },
///     );
/// });
/// // Nothing was asked, so the caller's index is untouched.
/// assert!(st.ask.standing().is_none());
/// ```
#[track_caller]
pub fn tree(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut TreeState,
    opts: &TreeOpts,
    index: &Order,
    find: &mut dyn FnMut(&str, Range<usize>) -> Option<usize>,
    row: &mut dyn FnMut(&mut Ctx<'_, '_>, Rect, Node, Face),
) -> Response {
    tree_into(
        &mut Direct,
        cx,
        area,
        st,
        opts,
        index,
        find,
        |_ink, cx, r, n, face| row(cx, r, n, face),
    )
}

/// **[`tree`], drawing through an [`Ink`] so an instrument can see every cell.**
///
/// The entry point a gate takes; [`tree`] is this with [`Direct`]. See [`crate::ink`] for why the
/// seam exists rather than a second implementation written against a `Tally`.
#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "`tree`'s own seven plus the `Ink` seam's writer, which is `collection_into`'s shape \
              one component down"
)]
pub fn tree_into<I, F, R>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut TreeState,
    opts: &TreeOpts,
    index: &Order,
    find: F,
    row: R,
) -> Response
where
    I: Ink,
    F: FnMut(&str, Range<usize>) -> Option<usize>,
    R: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, Node, Face),
{
    tree_with(
        ink,
        cx,
        area,
        st,
        opts,
        index,
        find,
        row,
        TreeShape::default(),
    )
}

/// **The one way `tree = collection + a flatten index` can be false**, as one value.
///
/// One field and not a boolean in a signature, so a reviewer's diff between the shipped build and
/// the refused one is a single line — [`TableShape`]'s arrangement one component over.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
struct TreeShape {
    /// Whether the indent is clamped to the rectangle.
    indent: Indent,
}

/// **`#[track_caller]` all the way down**, for [`draw_with`]'s reason: `Ctx::id` mints from
/// `Location::caller()`, and an attribute that stops one frame short of the call makes every tree in
/// the application one tree.
#[track_caller]
#[expect(
    clippy::too_many_arguments,
    reason = "`tree_into`'s eight plus the shape that separates the correct build from the refused \
              one. Splitting it would put the defect in a second function where a reviewer's diff \
              could not be one line"
)]
fn tree_with<I, F, R>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut TreeState,
    opts: &TreeOpts,
    index: &Order,
    find: F,
    mut row: R,
    shape: TreeShape,
) -> Response
where
    I: Ink,
    F: FnMut(&str, Range<usize>) -> Option<usize>,
    R: FnMut(&mut I, &mut Ctx<'_, '_>, Rect, Node, Face),
{
    // **The tree's own id, taken outside every closure** (ADR 0027), and every row's id is minted
    // from it and handed over — see [`Cell::id`] for why `Ctx::with_key` is not available inside a
    // scroll scope.
    let tid = cx.id();
    let rows = index.rows();
    // The pointer half's width, which is the *component's* rectangle rather than a row's: `local`
    // is measured from the hit entry, and the hit entry is `area`.
    let w = area.w;
    let paint = cx.theme().paint(opts.furniture);
    let open = cx.theme().glyph(Glyph::ArrowDown);
    let shut = cx.theme().glyph(Glyph::ArrowRight);

    // **Two `&mut` borrows of one state, taken apart here**, because the chord writes the request
    // while `collection` writes the row axis and they are different fields.
    let TreeState { coll, ask } = st;

    // `←` folds, `→` unfolds, on the cursor's row — and both only leave a request. A chord (`Ctrl`,
    // `Alt`) is an accelerator and is not this component's, which is `crate::nav::step`'s own rule.
    let mut refuse = |k: &Pressed, lead: usize| -> bool {
        if k.mods.ctrl() || k.mods.alt() {
            return false;
        }
        let Some(e) = index.at(lead) else {
            return false;
        };
        match k.code {
            Code::Left if !e.is_folded() && has_children(index, lead) => {
                ask.ask(Ask::Collapse(u64::from(e.node)));
                true
            }
            Code::Right if e.is_folded() => {
                ask.ask(Ask::Expand(u64::from(e.node)));
                true
            }
            _ => false,
        }
    };

    let resp = collection_chorded(
        ink,
        cx,
        area,
        coll,
        &opts.coll,
        rows,
        find,
        |ink, cx, r, i, face| {
            let Some(e) = index.at(i) else {
                return;
            };
            // **§7's first verb.** One run whatever its width, and the width is the one decision.
            //
            // **`r.w` and `r.x`, never `area`.** `collection` hands over one row of its own
            // rectangle in its own coordinates, and a component that reached past that for the
            // width or the origin is `crate::collect::header_row`'s defect: it drew from `x = 0`
            // rather than from the rectangle it was given, and every gate passed because every one
            // of them plays at `x == 0`.
            let ind = indent_columns(shape.indent, e.depth, r.w);
            if ind > 0 {
                // **The narrowing is here and nowhere else, and it saturates rather than wraps.**
                // [`indent_columns`] returns a `usize` because `depth * 2` at 59 999 is 119 998 and
                // `as u16` makes that 54 462 — a truncation that reads as a clamp. `Ink::run` takes
                // a `u16`, so a row *can* only request 65 535 columns however many it asked for:
                // that is a saturation, it is stated here, and it is exactly where
                // [`crate::counters::Tally::asked`] saturates too (`layout::text::width` returns a
                // `u16`), so the instrument's number and the request agree. **The ask itself is the
                // caller's `usize`** and `crate::forest::Shape` prints both beside each other.
                let requested = u16::try_from(ind).unwrap_or(u16::MAX);
                debug_assert!(
                    matches!(shape.indent, Indent::Unclamped) || usize::from(requested) == ind,
                    "a clamped indent is bounded by the rectangle and may never saturate"
                );
                let _ = ink.run(cx, r.x, r.y, " ", requested, paint);
            }
            // **The label rectangle collapses when the indent has eaten it**, which only the
            // unclamped arm can do: a clamped indent reserves two columns by construction. The
            // chevron is then skipped and the row drawer is handed an **empty** rectangle rather
            // than not called — §2's contract is *the cells it does not write are named in its
            // return value*, and *none* is an answer. Not calling it would make *rows the body
            // iterated* a second counter that separates the two arms, and §7's whole claim is that
            // only the ask does.
            let collapsed = ind >= usize::from(r.w);
            let indent = i32::try_from(ind.min(usize::from(r.w)))
                .expect("an indent inside the rectangle fits an i32");
            // **§7's second verb**, and a space on a leaf rather than nothing at all, so that every
            // row of the tree is the same partition of its rectangle.
            let folded = e.is_folded();
            let leaf = !folded && !has_children(index, i);
            let chevron = if leaf {
                " "
            } else if folded {
                shut
            } else {
                open
            };
            let (label, room) = if collapsed {
                (r.x + indent, 0)
            } else {
                let _ = ink.text(cx, r.x + indent, r.y, chevron, paint);
                (
                    r.x + indent + 1,
                    r.w - u16::try_from(ind + 1).expect("inside the rectangle"),
                )
            };
            row(
                ink,
                cx,
                Rect::new(label, r.y, room, 1),
                Node {
                    row: i,
                    node: e.node,
                    depth: e.depth,
                    folded,
                    leaf,
                    id: Id::keyed(tid, u64::from(e.node)),
                },
                face,
            );
        },
        &mut refuse,
    );

    // **The pointer half, and it reads `collection`'s edge rather than keeping one.** A press on the
    // chevron column of a row that has one leaves the same request the keyboard does.
    //
    // **It also selects the row, and that is deliberate rather than overlooked.** `collection` owns
    // the selection and has already applied the press by the time this runs; a component built on
    // it may add a gesture and may not *un-apply* one, which would need the press to be intercepted
    // before `collection` sees it — the pointer's version of `Refusal`, and there is no shape for
    // it: `Response::local` is computed inside `declare`, so there is nothing to refuse until the
    // hit entry exists.
    if coll.press_edge()
        && let Some((lx, ly)) = resp.local
        && let Ok(i) = usize::try_from(coll.offset + ly)
        && let Some(e) = index.at(i)
        && lx == i32::try_from(indent_columns(shape.indent, e.depth, w)).unwrap_or(i32::MAX)
    {
        if e.is_folded() {
            ask.ask(Ask::Expand(u64::from(e.node)));
        } else if has_children(index, i) {
            ask.ask(Ask::Collapse(u64::from(e.node)));
        }
    }
    resp
}

// ── what the table's two stores and its one slot cost ────────────────────────────────────────────

/// **The row count §6 prices the two cell stores at.** One million.
pub const CELL_ROWS: usize = 1_000_000;

/// **The declared column count the two stores are priced over.** Forty, which is §6's own
/// *select-all, select-one-row: 40 runs against 1* — the loss the per-column store takes, stated as
/// a column count rather than as a row count, because that is the half that is bounded.
pub const CELL_COLS: usize = 40;

/// **What one click on a column header costs, both ways.** `(runs, bytes, nanoseconds)` each.
///
/// The per-column store first, the flattened one second. §6 states **1 / 16 B / 0.08 µs** against
/// **1 000 000 / 16 MB / 62 535 µs**; what reproduces exactly is the *runs*, which are arithmetic
/// over the store's shape, and the bytes follow from them. The microseconds are a report and the
/// ratio is the gate — see [`crate::grid`]'s rule, which is §21's.
pub fn header_click_costs(
    rows: usize,
    ncols: usize,
    col: usize,
) -> ((usize, usize, u128), (usize, usize, u128)) {
    let mut cells = CellSel::new();
    let key = u16::try_from(col).unwrap_or(u16::MAX);
    let started = Instant::now();
    cells.select_column(key, rows);
    let per_column = (
        cells.span_count(),
        cells.bytes(),
        started.elapsed().as_nanos(),
    );

    let started = Instant::now();
    let (runs, bytes) = flattened_header_click(rows, ncols, col);
    ((per_column), (runs, bytes, started.elapsed().as_nanos()))
}

/// **The gesture the per-column store is worse at**, so the trade is stated from both sides.
///
/// `(per_column_runs, flattened_runs)` for selecting one whole row across `ncols` columns. §6:
/// *the per-column store's only losses (select-all, select-one-row: 40 runs against 1) are bounded
/// by the declared column count.* One run a column here; one run there, because a row is contiguous
/// under a row-major flattening.
pub fn row_click_costs(ncols: usize, row: usize) -> (usize, usize) {
    let keys: Vec<u16> = (0..ncols)
        .map(|c| u16::try_from(c).unwrap_or(u16::MAX))
        .collect();
    let mut cells = CellSel::new();
    cells.select_row(&keys, row);
    let mut flat = Selection::new();
    flat.insert(row * ncols, row * ncols + ncols);
    (cells.span_count(), flat.span_count())
}

/// **What it costs to follow a reorder, one slot against a map keyed per cell.** `(slot, map)` in
/// nanoseconds.
///
/// §6's **0.29 µs**, and its reason: *it is one position*. The slot is one lookup at any length; the
/// map is one lookup a stored cell, and a per-cell map over a sorted million-row table is what the
/// slot exists instead of. The map arm is priced over `entries` rather than over `len`, because a
/// map that held one entry a row would not be a *map* argument at all — it would be the length
/// argument §5 already settled.
pub fn edit_follow_costs(len: usize, entries: usize) -> (u128, u128) {
    let mut st = TableState::new();
    st.edit(len / 2, 7);
    let started = Instant::now();
    st.follow(|row| Some(len - 1 - row));
    let slot = started.elapsed().as_nanos();

    let mut map: Vec<(usize, u16)> = (0..entries).map(|i| (i, 7)).collect();
    let started = Instant::now();
    for e in &mut map {
        e.0 = len - 1 - e.0;
    }
    map.sort_unstable();
    (slot, started.elapsed().as_nanos())
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// `pagination` — §17's Tier 2 pager: `collection` at a small length, on the other axis
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// [`pagination`]'s options.
///
/// Spec §1's rule 3: a `Default` struct, never a required builder.
///
/// **There is no `mode` here.** A pager is [`Mode::Options`] — *exactly one, and it can never become
/// zero* — and that is not a caller's choice: a paginator with nothing selected is a paginator
/// showing no page. It is the one place in this module where a [`CollOpts`] field is fixed rather
/// than forwarded, and `page_opts` — one private function — is where that happens once.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PageOpts {
    /// How many rows one type-ahead keystroke may look at. [`CollOpts::search`], forwarded whole.
    pub search: usize,
    /// **A stepper at each end**, drawn in the two arrow glyphs the freeze declares for this
    /// component (spec §17). Dropped when the strip has no room for a page between them, because a
    /// pager with no page in it is two buttons and a lie — `crate::scroll::ScrollbarOpts::caps`
    /// makes the same call for the same reason.
    pub steppers: bool,
    /// How wide one page's cell is, or `0` to derive it from the widest label.
    pub cell: u16,
    /// The role the steppers are drawn in.
    pub stepper: Role,
    /// The role the strip's own cells — the gaps the pages do not reach — are written in.
    pub tail: Role,
    /// Where a page's label sits in its cell.
    pub justify: crate::text::Justify,
}

impl Default for PageOpts {
    fn default() -> PageOpts {
        PageOpts {
            search: SEARCH_BUDGET,
            steppers: true,
            cell: 0,
            stepper: Role::Border,
            tail: Role::Body,
            justify: crate::text::Justify::Middle,
        }
    }
}

/// **The [`CollOpts`] a pager is**, and the one line that makes *no second navigation model* a fact
/// rather than a claim.
///
/// [`keyboard`] reads `mode` and `search` and nothing else, so a pager that reached the drain loop
/// with its own options would be a second policy wearing the same call. This is the whole
/// translation, and it is one function so that a reader can see there is no second one.
fn page_opts(opts: &PageOpts) -> CollOpts {
    CollOpts {
        mode: Mode::Options,
        search: opts.search,
        tail: opts.tail,
    }
}

/// **How many pages fit in `room` cells at `cell` cells each.** At least one, because a strip with
/// no page in it has nothing to be a pager of.
fn fits(room: u16, cell: u16) -> u16 {
    (room / cell.max(1)).max(1)
}

/// **A strip of page numbers with a stepper at each end — `collection` at a small length.**
///
/// **Hostile axes:** none.
///
/// It is [`collection`]'s store at a small length: the pages *are* the whole content, so there is no
/// window onto anything larger and no offset a notch could move. The row loop — which is where the
/// other three would live — is the axis this component does not reach.
///
/// Spec §18's R3: *a composition of shipped components with no new mechanism.* The store is
/// [`CollState`], the policy is [`Mode::Options`], the keyboard is [`collection`]'s own drain loop
/// and therefore [`crate::nav::cursor`], and what this component adds is a **rectangle split**.
///
/// # It is `collection`'s store and not `collection`'s row loop, and the reason is an axis
///
/// [`table`] is `collection` plus a column split and [`tree`] is `collection` plus a flatten index,
/// and both of them *call* [`collection_into`]. A pager cannot: the row loop hands its drawer
/// `Rect::new(0, y, area.w, 1)` and asks [`Ctx::visible_rows`](vitui_runtime::Ctx::visible_rows)
/// which rows are reachable, both of which are **the vertical axis by construction**, and a pager is
/// a row of cells. A transpose is not a rectangle split — it is a different `Ctx` — so what this
/// reaches is the half of `collection` that has no axis at all: the store, the thirteen arms of
/// [`apply`], and the one drain loop.
///
/// That is not a weaker claim than `table`'s, and `tests::a_pager_and_a_collection_land_on_the_same_index`
/// is why: the same key sequence over the same length moves both to the same index, at every length
/// and for every key in the vocabulary.
///
/// ```
/// use vitui_components::collect::{CollState, pagination};
/// use vitui_runtime::Rect;
/// use vitui_runtime::ctx::Driver;
///
/// let mut st = CollState::new();
/// let mut driver = Driver::headless(40, 3).expect("a sink attaches");
/// driver.frame(|cx| {
///     let resp = pagination(cx, Rect::new(0, 2, 40, 1), &mut st, 9);
///     // Rule 4: a `Response` back, and nothing has happened on a frame with no input.
///     assert!(!resp.clicked);
/// });
/// // Exactly one, and it can never be zero: the first page is current before anybody has pressed
/// // anything, because that is what `Mode::Options` means.
/// assert_eq!(st.sel.lead, 0);
/// // One hit entry for the strip, never one per page.
/// assert_eq!(driver.inspect().hits().len(), 1);
/// ```
#[track_caller]
pub fn pagination(cx: &mut Ctx<'_, '_>, area: Rect, st: &mut CollState, pages: usize) -> Response {
    pagination_with(cx, area, st, pages, &PageOpts::default())
}

/// [`pagination`], with the options spelled out.
#[track_caller]
pub fn pagination_with(
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut CollState,
    pages: usize,
    opts: &PageOpts,
) -> Response {
    pagination_into(&mut Direct, cx, area, st, pages, opts)
}

/// **[`pagination`], drawing through an [`Ink`] so a counter can see every cell.**
///
/// The entry point a gate takes; [`pagination`] is this with [`Direct`].
#[track_caller]
pub fn pagination_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut CollState,
    pages: usize,
    opts: &PageOpts,
) -> Response {
    // **The id, taken outside every closure** (ADR 0027), and `#[track_caller]` all the way down.
    let id = cx.id();
    if area.is_empty() {
        return Response::inert(id, area);
    }
    let len = pages.max(1);
    let coll = page_opts(opts);

    // **One hit entry for the strip, never one per page**, and no `Interest::SCROLL`: a pager does
    // not own an offset the wheel may move (spec §17's own column), so a notch over it chains
    // outward to whatever is beneath.
    let mut resp = cx.interact(
        id,
        area,
        Interest::CLICK.with(Interest::HOVER).with(Interest::FOCUS),
    );

    let (strip, below) = rect::split_at_v(area, 1);
    let cell = page_cell(len, opts);
    let caps = u16::from(opts.steppers && strip.w >= 2 + cell);
    let room = strip.w.saturating_sub(caps.saturating_mul(2));
    let shown = usize::from(fits(room, cell)).min(len);

    // **The window's first page is `CollState::offset`**, which is the store's own field read on the
    // axis this component lays out on. A second field for it would be ADR 0028's *two stores of one
    // fact* with the count as the argument rather than the shape.
    let max = i32::try_from(len - shown).unwrap_or(i32::MAX);
    st.offset = st.offset.clamp(0, max);

    // The pointer half. `local` is this frame's, so the page is arithmetic and there is no frame lag
    // and no per-page hit entry — `collection`'s own reading, one axis over.
    let over = resp
        .local
        .filter(|(_, ly)| *ly == 0)
        .and_then(|(lx, _)| page_at(lx, st.offset, strip.w, caps, cell, len, shown));
    let press_edge = resp.pressed && !st.pressing;
    st.pressing = resp.pressed;
    st.edge = press_edge;
    if press_edge {
        // **`from_click` and not a `Gesture` written here**, which is the pointer half of *no second
        // navigation model*: `from_click` and `from_key` are two readings of one vocabulary (§5),
        // and at `Mode::Options` `apply` collapses ctrl and shift onto `select_only` in one arm. A
        // pager that spelled `Gesture::Plain` directly would be right today and would stop being
        // right the day that arm moves.
        let at = match over {
            Some(Hit::Page(at)) => Some(at),
            Some(Hit::Prev) => Some(st.sel.lead.saturating_sub(1)),
            Some(Hit::Next) => Some((st.sel.lead + 1).min(len - 1)),
            None => None,
        };
        if let Some(at) = at {
            apply(coll.mode, &mut st.sel, len, from_click(resp.mods, at));
            cx.focus(id);
            resp.changed = true;
        }
    }

    // **The keyboard is `collection`'s own drain loop and there is no second one.** `nav::step`
    // reads `←`/`→` as `↑`/`↓` — components ticket 17's finding, which is a collision for a `tree`
    // and is exactly right here, because a pager's axis *is* the horizontal one.
    // **The caller's search, which for a pager is a prefix over its own page numbers** — and it
    // allocates nothing, because a label is [`Digits`] on the stack. `collection` owns the buffer,
    // the deadline and the bound; this is the half §5 says is the caller's.
    let mut find = |buf: &str, range: Range<usize>| {
        range
            .into_iter()
            .find(|&i| Digits::of(i).as_str().starts_with(buf))
    };
    let asked = keyboard(
        cx,
        id,
        st,
        &coll,
        len,
        &mut find,
        shown.max(1),
        &mut no_refusal,
    );
    resp.changed |= asked.changed;

    // **The window follows the cursor here rather than through `request_into_view`**, because the
    // offset is this component's own and a reveal is a request to whatever owns the *enclosing*
    // area. Conditional on the key having moved something, which is `CONTEXT.md`'s rule and the one
    // four resolved prototypes broke.
    if asked.reveal {
        let lead = i32::try_from(st.sel.lead).unwrap_or(0);
        let span = i32::from(u16::try_from(shown).unwrap_or(u16::MAX)).max(1);
        st.offset = st.offset.clamp(lead - span + 1, lead).clamp(0, max);
    }

    strip_into(
        ink, cx, strip, st, len, shown, caps, cell, opts, &resp, over,
    );
    crate::text::pad_rows(ink, cx, below, cx.theme().paint(opts.tail));
    resp
}

/// **What a column of the strip is**, resolved from `Response::local` by arithmetic.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Hit {
    /// The leading stepper.
    Prev,
    /// One of the visible pages.
    Page(usize),
    /// The trailing stepper.
    Next,
}

/// **A page's label, formatted into the stack.**
///
/// One-based, because a paginator is read by a person — and a `String`, because the obvious
/// spelling is what it costs: `(i + 1).to_string()` is **one allocation a page a frame**, which over
/// 137 pages and 60 frames is 9 240 against a budget of zero. The pager pays it twice, once to draw
/// each visible cell and once for the type-ahead's search, and **every other counter is blind to
/// it** — the writes, the verbs, the regions and the picture are identical either way. It was caught
/// by `tests::the_three_tier_two_composites_allocate_nothing_and_the_record_shaped_form_allocates_a_frame`
/// on its first run, which is `crate::structure::rule`'s `format!` and `crate::media::player`'s
/// `Vec` for the third time on this map.
///
/// Twenty digits is `usize::MAX`, so the buffer cannot be short.
struct Digits {
    buf: [u8; 20],
    at: usize,
}

impl Digits {
    /// The one-based label for index `i`.
    fn of(i: usize) -> Digits {
        let mut d = Digits {
            buf: [b'0'; 20],
            at: 20,
        };
        let mut n = i.saturating_add(1);
        loop {
            d.at -= 1;
            d.buf[d.at] = b'0' + u8::try_from(n % 10).unwrap_or(0);
            n /= 10;
            if n == 0 || d.at == 0 {
                break;
            }
        }
        d
    }

    /// What it spells. ASCII digits, so the conversion cannot fail.
    fn as_str(&self) -> &str {
        std::str::from_utf8(&self.buf[self.at..]).unwrap_or("0")
    }
}

/// **How wide one page's cell is.** The caller's, or the widest label plus a cell either side.
fn page_cell(len: usize, opts: &PageOpts) -> u16 {
    if opts.cell > 0 {
        return opts.cell.max(1);
    }
    width(Digits::of(len.saturating_sub(1)).as_str())
        .saturating_add(2)
        .max(3)
}

/// Which part of the strip column `lx` is, or `None` for a gap the pages do not reach.
fn page_at(
    lx: i32,
    offset: i32,
    w: u16,
    caps: u16,
    cell: u16,
    len: usize,
    shown: usize,
) -> Option<Hit> {
    if lx < 0 || lx >= i32::from(w) {
        return None;
    }
    if caps > 0 {
        if lx < i32::from(caps) {
            return Some(Hit::Prev);
        }
        if lx >= i32::from(w - caps) {
            return Some(Hit::Next);
        }
    }
    let into = lx - i32::from(caps);
    let slot = usize::try_from(into / i32::from(cell.max(1))).ok()?;
    if slot >= shown {
        return None;
    }
    let at = usize::try_from(offset).unwrap_or(0) + slot;
    (at < len).then_some(Hit::Page(at))
}

/// **The strip: the two steppers, the visible pages, and the gap the pages do not reach.**
#[expect(
    clippy::too_many_arguments,
    reason = "the strip's own geometry, resolved once by `pagination_into` and handed over whole. \
              Recomputing it here would be two derivations of one layout, which is the shape the \
              pointer half and the drawing half must not be allowed to disagree about"
)]
fn strip_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    strip: Rect,
    st: &CollState,
    len: usize,
    shown: usize,
    caps: u16,
    cell: u16,
    opts: &PageOpts,
    resp: &Response,
    over: Option<Hit>,
) {
    if strip.is_empty() {
        return;
    }
    let theme = cx.theme();
    let tail = theme.paint(opts.tail);
    let stepper = theme.paint(opts.stepper);
    let (left, right) = (
        theme.glyph(Glyph::ArrowLeft),
        theme.glyph(Glyph::ArrowRight),
    );

    let mut x = strip.x;
    if caps > 0 {
        ink.pad_to(cx, x, strip.y, left, caps, stepper);
        x += i32::from(caps);
    }
    let first = usize::try_from(st.offset).unwrap_or(0);
    // **The last column a page may write.** `fits` floors at one, so a strip with no room for a
    // whole cell still stands a page — and a cell written at its full width there runs **past the
    // rectangle**, onto whatever is beside it. It is `crate::scroll`'s `arithmetic_band` in a
    // component that has no view to be clipped by, and the sweep that should have caught it drew
    // the pager at the screen's own edge, where the screen clipped the overrun: the recorder and
    // the defect shared a coordinate system, which is the third time on this map.
    let stop = strip.right() - i32::from(caps);
    // **Seeked once and advanced in lockstep**, which is `collection`'s own scan and the reason a
    // pager needs no second selection reading.
    let mut scan = Scan::seek(&st.sel, first);
    for slot in 0..shown {
        let i = first + slot;
        if i >= len || x >= stop {
            break;
        }
        let face = Face {
            selected: scan.at(i),
            cursor: st.sel.lead == i,
            active: resp.focused,
            hovered: over == Some(Hit::Page(i)),
            disabled: false,
        };
        let paint = crate::frame::face_paint(cx.theme(), face);
        let room = u16::try_from(stop - x).unwrap_or(cell).min(cell);
        // **`elided_row_into` and not `pad_to`**, and the difference is a partition: `pad_to` pads a
        // short label and writes a long one **whole**, so a page cell narrower than its own number
        // writes past its share — a `137` in a one-cell strip is two cells, one of them the
        // neighbour's. Elided, §16's one-cell marker rule holds here as it does on a label, and the
        // cell is exactly `room` wide whatever the number is.
        crate::glyphs::elided_row_into(ink, cx, x, strip.y, Digits::of(i).as_str(), room, paint);
        x += i32::from(room);
    }
    // **The gap between the last page and the trailing stepper is the pager's own**, and writing it
    // is the same half of §2's rule that `collection`'s tail is: the cells the content does not
    // reach belong to the component that was handed the rectangle.
    if x < stop {
        let w = u16::try_from(stop - x).unwrap_or(0);
        ink.run(cx, x, strip.y, " ", w, tail);
    }
    if caps > 0 {
        ink.pad_to(cx, stop, strip.y, right, caps, stepper);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counters::Tally;
    use crate::order::Entry;
    use vitui_runtime::ctx::Driver;

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

    // ── `pagination` ─────────────────────────────────────────────────────────────────────────────

    /// A tally over one frame of `f` on a sink two cells larger than `w` by `h` on every side.
    ///
    /// **The margin is the point.** A component drawn at the screen's own edge is clipped by the
    /// screen, so a verb that runs past its rectangle costs nothing a counter can see — which is how
    /// the first spelling of the sweep below missed a pager writing **5 cells into a 4-cell strip**.
    /// The recorder and the defect shared a coordinate system, which is the third time this crate
    /// has met that shape (components 19's `Tally::distinct`, components 29's `qr_into`).
    fn tallied_page(w: u16, h: u16, f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>)) -> Tally {
        let mut driver = Driver::headless(w + 4, h + 4).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| f(&mut tally, cx));
        tally
    }

    /// **A pager writes every cell of its rectangle exactly once**, at every size, page count and
    /// window position.
    ///
    /// §2's two equalities over the one component in this module whose content does not fill its own
    /// rectangle by construction: a strip of `shown` cells of `cell` columns leaves a gap before the
    /// trailing stepper whenever the two do not divide, and that gap is the pager's own.
    #[test]
    fn a_pager_writes_every_cell_of_its_rectangle_exactly_once() {
        for (w, h) in [(1, 1), (4, 1), (12, 1), (40, 1), (40, 3), (13, 2), (200, 1)] {
            for pages in [1usize, 2, 9, 10, 137, 5_000] {
                for offset in [0i32, 3, 900] {
                    for steppers in [true, false] {
                        let opts = PageOpts {
                            steppers,
                            ..PageOpts::default()
                        };
                        let mut st = CollState::new();
                        st.offset = offset;
                        let tally = tallied_page(w, h, |tally, cx| {
                            pagination_into(
                                tally,
                                cx,
                                Rect::new(2, 2, w, h),
                                &mut st,
                                pages,
                                &opts,
                            );
                        });
                        let cells = u64::from(w) * u64::from(h);
                        assert_eq!(
                            tally.writes(),
                            tally.distinct(),
                            "{w}x{h} {pages} pages at {offset}: {} cells written twice",
                            tally.writes() - tally.distinct()
                        );
                        assert_eq!(
                            tally.distinct(),
                            cells,
                            "{w}x{h} {pages} pages at {offset}: the strip is {} cells rather than \
                             {cells}, and the margin around it is what makes an overrun visible \
                             rather than clipped",
                            tally.distinct()
                        );
                    }
                }
            }
        }
    }

    /// **One hit entry for the strip, never one per page, and the count does not move with the page
    /// count.**
    #[test]
    fn a_pager_declares_one_hit_entry_however_many_pages_it_has() {
        for pages in [1usize, 9, 137, 100_000] {
            let mut st = CollState::new();
            let mut driver = Driver::headless(60, 3).expect("a sink cannot fail to attach");
            driver.frame(|cx| {
                pagination(cx, Rect::new(0, 2, 60, 1), &mut st, pages);
            });
            let frame = driver.inspect();
            assert_eq!(frame.hits().len(), 1, "{pages} pages");
            // One tab stop, because a pager is reached with `Tab` and moved with the arrows.
            assert_eq!(frame.stop_count(), 1);
        }
    }

    /// **A click on a page selects it and a click on a stepper steps**, both through `from_click`.
    ///
    /// `from_click` and `from_key` are two readings of one vocabulary (§5), and this is the pointer
    /// half of *no second navigation model*: the pager resolves *which column* by arithmetic and
    /// hands the answer to the same `apply` the keyboard reaches.
    ///
    /// **A press is two frames**, because the grab is awarded at `end` — components ticket 30's
    /// cadence, and a gate playing one frame a phase would measure the cadence and call it the
    /// mechanism.
    #[test]
    fn a_click_on_a_page_selects_it_and_a_click_on_a_stepper_steps() {
        /// Ten pages over an 11-cell strip with room for three, from one call site.
        fn frame(driver: &mut Driver, st: &mut CollState) {
            let opts = PageOpts {
                cell: 3,
                ..PageOpts::default()
            };
            driver.frame(|cx| {
                pagination_with(cx, Rect::new(0, 0, 11, 1), st, 10, &opts);
            });
        }

        let mut st = CollState::new();
        let mut driver = Driver::headless(11, 1).expect("a sink cannot fail to attach");
        frame(&mut driver, &mut st);

        // Column 4 is the second visible page: one stepper, then three-cell cells.
        press_at(&mut driver, 4);
        frame(&mut driver, &mut st);
        frame(&mut driver, &mut st);
        assert_eq!(
            st.sel.lead, 1,
            "a click on the second page did not select it"
        );
        release_at(&mut driver, 4);
        frame(&mut driver, &mut st);

        // And the trailing stepper is the last column.
        press_at(&mut driver, 10);
        frame(&mut driver, &mut st);
        frame(&mut driver, &mut st);
        assert_eq!(st.sel.lead, 2, "the trailing stepper did not step");
    }

    /// A pointer press at column `x` on the strip's own row.
    fn press_at(driver: &mut Driver, x: u16) {
        driver.post_mouse(vitui_runtime::Mouse {
            x,
            y: 0,
            kind: vitui_runtime::MouseKind::Down(vitui_runtime::Button::Left),
            buttons: vitui_runtime::Buttons::NONE,
            mods: Mods::NONE,
            at: Instant::now(),
        });
    }

    /// The release that ends it.
    fn release_at(driver: &mut Driver, x: u16) {
        driver.post_mouse(vitui_runtime::Mouse {
            x,
            y: 0,
            kind: vitui_runtime::MouseKind::Up(vitui_runtime::Button::Left),
            buttons: vitui_runtime::Buttons::NONE,
            mods: Mods::NONE,
            at: Instant::now(),
        });
    }

    /// **Two pagers on one screen are two widgets and merge nothing** — ADR 0027.
    ///
    /// A pager keys nothing per page, so what this catches is the one thing that could go wrong at
    /// its own level: a `#[track_caller]` that stopped one frame short of the call would make both
    /// pagers one widget, and `Ctx::interact` makes a merged claim **inert** — so the second pager
    /// would take no press, no drag and no hover on a screen that renders perfectly.
    #[test]
    fn two_pagers_on_one_screen_are_two_widgets_and_merge_nothing() {
        let (mut a, mut b) = (CollState::new(), CollState::new());
        let mut driver = Driver::headless(40, 4).expect("a sink cannot fail to attach");
        let mut ids = (None, None);
        driver.frame(|cx| {
            ids.0 = Some(pagination(cx, Rect::new(0, 0, 40, 1), &mut a, 9).id);
            ids.1 = Some(pagination(cx, Rect::new(0, 3, 40, 1), &mut b, 40).id);
        });
        assert_ne!(ids.0, ids.1, "two pagers are one pager");
        let frame = driver.inspect();
        assert_eq!(frame.hits().len(), 2);
        assert_eq!(frame.stop_count(), 2);
        assert_eq!(frame.ids().merges(), 0);
    }

    /// **Criterion 3: no second store and no second navigation model** — the same keys move a pager
    /// and a collection to the same index, at every length and for every key in the vocabulary.
    ///
    /// This is the load-bearing half of *`pagination` is `collection` at a small length*. The two
    /// components lay their content out on different axes and share [`keyboard`], so an equality
    /// between the indices they land on is what says the sharing is real: a pager that had grown its
    /// own reading of `←`/`→` would still draw correctly and would diverge here.
    ///
    /// `crate::nav::step` reads `←` and `→` as `↑` and `↓`, which is components ticket 17's collision
    /// for a `tree` and is exactly what a pager wants — so the horizontal keys are in the sweep
    /// beside the vertical ones and both must agree.
    #[test]
    fn a_pager_and_a_collection_land_on_the_same_index() {
        use vitui_runtime::keys::Chord;

        let codes = [
            Code::Left,
            Code::Right,
            Code::Up,
            Code::Down,
            Code::Home,
            Code::End,
            Code::PageUp,
            Code::PageDown,
        ];
        let mut landed = std::collections::BTreeSet::new();
        for len in [1usize, 2, 9, 40] {
            for code in codes {
                for presses in [1usize, 3] {
                    let chords = vec![Chord::new(code); presses];
                    let pager = drive_pager(len, &chords);
                    let coll = drive_collection(len, &chords);
                    assert_eq!(
                        pager, coll,
                        "{len} pages, {presses}x {code:?}: a pager and a collection disagree about \
                         where the cursor is"
                    );
                    landed.insert(pager);
                }
            }
        }
        // **The sweep is not vacuous**, and this line is here because the first spelling of it was:
        // the first frame and the drive loop were two call sites, so `Ctx::id` minted two ids, the
        // planted focus named a widget no later frame declared, **not one key was delivered**, and
        // both arms stood still at zero for every row of the sweep. Thirty-nine equal zeroes.
        assert!(
            landed.len() > 1,
            "every drive landed on the same index, so the equality is between two things that did \
             not move: {landed:?}"
        );
    }

    /// **One frame of the pager, from one call site.**
    ///
    /// `Ctx::id` mints from `Location::caller()` and this function does **not** carry
    /// `#[track_caller]`, which is the whole point: every frame of a drive comes from the line below
    /// and is therefore the same widget. Written with the first frame at one call site and the rest
    /// at another — the obvious spelling — the planted focus names an id no later frame declares,
    /// **no key is ever delivered**, and the sweep above passes with both arms standing still. That
    /// is components ticket 33's finding arriving in the instrument again, and
    /// `tests::a_pager_and_a_collection_land_on_the_same_index` asserts the ids agree so it cannot
    /// come back.
    fn pager_frame(driver: &mut Driver, st: &mut CollState, len: usize) -> Id {
        let opts = PageOpts {
            // **A strip with room for exactly one page cell**, so that a pager's page and a
            // collection's viewport are the same jump and `PageUp`/`PageDown` can be compared at
            // all. Left at the default the strip fits fourteen and the two arms disagree about one
            // key out of eight — which is a difference in the *screen*, not in the model.
            cell: 3,
            ..PageOpts::default()
        };
        let mut id = None;
        driver.frame(|cx| id = Some(pagination_with(cx, Rect::new(0, 0, 5, 1), st, len, &opts).id));
        id.expect("a frame ran")
    }

    /// The same, for `collection` at `Mode::Options` — which is what a pager is.
    fn collection_frame(driver: &mut Driver, st: &mut CollState, len: usize) -> Id {
        let opts = CollOpts {
            mode: Mode::Options,
            ..CollOpts::default()
        };
        let mut id = None;
        driver.frame(|cx| {
            id = Some(
                collection(
                    cx,
                    Rect::new(0, 0, 60, 1),
                    st,
                    &opts,
                    Rows::of(len),
                    &mut |_, _| None,
                    &mut |_, _, _, _| {},
                )
                .id,
            );
        });
        id.expect("a frame ran")
    }

    /// Press `chords` into a focused pager over `len` pages, one key a frame, and answer the cursor.
    fn drive_pager(len: usize, chords: &[vitui_runtime::keys::Chord]) -> usize {
        let mut st = CollState::new();
        let mut driver = Driver::headless(5, 1).expect("a sink cannot fail to attach");
        let first = pager_frame(&mut driver, &mut st, len);
        for &c in chords {
            driver.plant(None, Some(first), None);
            driver.post_key(crate::keys::press(c));
            let again = pager_frame(&mut driver, &mut st, len);
            assert_eq!(again, first, "two frames of one pager are two widgets");
        }
        st.sel.lead
    }

    /// The same drive over `collection` at `Mode::Options`.
    ///
    /// **One row of viewport on both sides**, so that `PageUp`/`PageDown` mean the same jump: a
    /// pager's page is how many page cells fit, which the sweep pins at one by giving the strip room
    /// for exactly one cell.
    fn drive_collection(len: usize, chords: &[vitui_runtime::keys::Chord]) -> usize {
        let mut st = CollState::new();
        let mut driver = Driver::headless(60, 1).expect("a sink cannot fail to attach");
        let first = collection_frame(&mut driver, &mut st, len);
        for &c in chords {
            driver.plant(None, Some(first), None);
            driver.post_key(crate::keys::press(c));
            let again = collection_frame(&mut driver, &mut st, len);
            assert_eq!(again, first, "two frames of one collection are two widgets");
        }
        st.sel.lead
    }

    /// **A page's label is formatted into the stack**, and the type-ahead is a prefix over the
    /// numbers.
    ///
    /// The obvious spelling — `(i + 1).to_string()` — is one allocation a page a frame, and the
    /// allocation window caught it at **9 240 over 60 frames** on its first run with every other
    /// counter reading identically. What is asserted here is the two things that would make the
    /// repair wrong: the digits, and that typing still finds a page.
    #[test]
    fn a_pages_label_is_digits_on_the_stack_and_typing_one_finds_it() {
        for (i, spelled) in [
            (0usize, "1"),
            (8, "9"),
            (9, "10"),
            (136, "137"),
            (99_998, "99999"),
        ] {
            assert_eq!(Digits::of(i).as_str(), spelled);
        }
        // `usize::MAX` is twenty digits and the buffer is twenty, so the largest label a pager can
        // carry is the one the type can name.
        assert_eq!(Digits::of(usize::MAX - 1).as_str(), usize::MAX.to_string());

        // And the type-ahead is a prefix over those, which is the caller's search §5 asks for.
        use vitui_runtime::keys::Chord;
        let mut st = CollState::new();
        let mut driver = Driver::headless(60, 1).expect("a sink cannot fail to attach");
        let opts = PageOpts::default();
        fn frame(driver: &mut Driver, st: &mut CollState, opts: &PageOpts) -> Id {
            let mut id = None;
            driver.frame(|cx| {
                id = Some(pagination_with(cx, Rect::new(0, 0, 60, 1), st, 137, opts).id);
            });
            id.expect("a frame ran")
        }
        let id = frame(&mut driver, &mut st, &opts);
        for c in ['1', '2'] {
            driver.plant(None, Some(id), None);
            driver.post_key(crate::keys::press(Chord::typed(c)));
            frame(&mut driver, &mut st, &opts);
        }
        // `1` lands on page 1 (index 0) and `12` on page 12 (index 11) — one buffer, one deadline,
        // and both of them `collection`'s.
        assert_eq!(st.sel.lead, 11);
    }

    /// **A pager mints no second store and no second navigation model**, read off the source.
    ///
    /// The scan `crate::composed` runs over the whole Tier 2 table, restricted here to the one thing
    /// that table cannot say: the needles are checked against *this* file's section rather than
    /// against a claim, and the negative half is watched being satisfiable — a section that reached
    /// `nav::step` directly would be a second reading of the two keys `keyboard` already owns.
    #[test]
    fn a_pager_reaches_the_one_drain_loop_and_mints_no_second_reading_of_it() {
        let source =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/collect.rs"))
                .expect("this file is here");
        let section = crate::composed::section(
            &source,
            "// `pagination` — §17's Tier 2 pager: `collection` at a small length, on the other axis",
        );
        assert!(!section.is_empty());
        for used in [
            "keyboard(",
            "apply(coll.mode",
            "Mode::Options",
            "Scan::seek(",
        ] {
            assert!(
                crate::dense::declares(section, used),
                "a pager no longer reaches `{used}`"
            );
        }
        for minted in [
            "nav::step(",
            "from_key(",
            "struct PageState",
            "Selection::new()",
            "cx.scrollable(",
        ] {
            assert!(
                !crate::dense::declares(section, minted),
                "a pager mints `{minted}`, which is a second navigation model or a second store"
            );
        }
    }

    /// **The window is `CollState::offset` and it follows the cursor only when a key moved it.**
    ///
    /// `CONTEXT.md` forbids the unconditional form by name and four resolved prototypes shipped it
    /// anyway. Here the offset is this component's own rather than a request to an enclosing area,
    /// so the arm that would be wrong is a clamp taken every frame — and what that costs is
    /// measured: a caller that moved the window by hand cannot keep it, on a frame with no key in
    /// it at all.
    #[test]
    fn a_pagers_window_is_the_stores_own_offset_and_follows_only_a_key() {
        use vitui_runtime::keys::{Chord, Code};

        /// Ten pages over a strip with room for three, from one call site — see [`pager_frame`].
        fn frame(driver: &mut Driver, st: &mut CollState) -> Id {
            let opts = PageOpts {
                cell: 3,
                ..PageOpts::default()
            };
            let mut id = None;
            driver.frame(|cx| {
                id = Some(pagination_with(cx, Rect::new(0, 0, 11, 1), st, 10, &opts).id);
            });
            id.expect("a frame ran")
        }

        let mut st = CollState::new();
        let mut driver = Driver::headless(11, 1).expect("a sink cannot fail to attach");
        let id = frame(&mut driver, &mut st);
        assert_eq!((st.offset, st.sel.lead), (0, 0));

        // A caller moving the window by hand is left alone: no key moved anything.
        st.offset = 5;
        frame(&mut driver, &mut st);
        assert_eq!(
            st.offset, 5,
            "the window moved on a frame with no key in it"
        );

        // And a key that moves the cursor drags the window exactly far enough to show it.
        driver.plant(None, Some(id), None);
        driver.post_key(crate::keys::press(Chord::new(Code::Home)));
        frame(&mut driver, &mut st);
        assert_eq!((st.offset, st.sel.lead), (0, 0));

        driver.plant(None, Some(id), None);
        driver.post_key(crate::keys::press(Chord::new(Code::End)));
        frame(&mut driver, &mut st);
        assert_eq!(
            (st.offset, st.sel.lead),
            (7, 9),
            "`End` did not bring the last page into view"
        );
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

    /// **A row selects on the press, and holding the button does not select it again.**
    ///
    /// The runtime sets `Response::clicked` on `MouseKind::Up` — *a click is a release over the
    /// widget that was pressed* — which is right for a button and wrong for a list. Driven through
    /// a pty against the shipped `triage` application, the release-driven spelling reported
    /// **nothing on the press and the row selected on the release**, which is what a user reports
    /// as *it works on key-up*.
    ///
    /// The other half is why the edge is reconstructed rather than read: `Response::pressed` is
    /// `grab == Some(id)`, true for **every frame the button is held**, so applying on the level
    /// would re-run `from_click` all the way down — and a ctrl-click, which toggles, would flicker
    /// for as long as the user leaned on the button. Both directions are asserted here.
    #[test]
    fn a_row_selects_on_the_press_and_holding_does_not_select_it_twice() {
        use vitui_runtime::{Button, Buttons, Mods, Mouse, MouseKind};

        /// One frame per entry; `None` draws without posting anything.
        ///
        /// The first is the frame that builds the hit index — nothing can be over a region that has
        /// not been declared yet (ADR 0012) — and the second is a `Move`, because the runtime reads
        /// the pointer's position once per frame before it walks the batch, so a `Down` arriving at
        /// a position nothing has moved to is a press over nothing.
        fn run(kinds: &[Option<MouseKind>], mods: Mods) -> (usize, Vec<usize>) {
            let mut driver = crate::runner::driver_at(40, 8, vitui_runtime::Density::default());
            let mut st = CollState::new();
            let opts = CollOpts {
                mode: Mode::Multi,
                ..CollOpts::default()
            };
            let mut applied = Vec::new();
            for kind in kinds {
                if let Some(kind) = *kind {
                    driver.post_mouse(Mouse {
                        x: 5,
                        y: 3,
                        kind,
                        buttons: Buttons::NONE,
                        mods,
                        at: Instant::now(),
                    });
                }
                driver.frame(|cx| {
                    let area = cx.area();
                    let _ = collection(
                        cx,
                        area,
                        &mut st,
                        &opts,
                        Rows::of(1_000),
                        &mut |_: &str, _: Range<usize>| None,
                        &mut |_cx: &mut Ctx<'_, '_>, _r: Rect, _i: usize, _f: Face| {},
                    );
                });
                applied.push(st.sel.count());
            }
            (st.sel.lead, applied)
        }

        // The press selects. The award is assembled at the end of the frame that carried the
        // event, so the frame that *sees* the press is the one after it — which is why there is a
        // trailing `None` here and why `hovered` is documented as the previous frame's guess.
        let (lead, applied) = run(
            &[
                None,
                Some(MouseKind::Move),
                Some(MouseKind::Down(Button::Left)),
                None,
            ],
            Mods::NONE,
        );
        assert_eq!(
            applied,
            vec![0, 0, 0, 1],
            "the press selects the row: {applied:?}",
        );
        assert_eq!(
            lead, 3,
            "and the row is this frame's pointer, by arithmetic"
        );

        // **Holding it does not apply the gesture again, and the case has to be a ctrl-click to
        // show it.** A plain click is idempotent — `Gesture::Plain(at)` sets the lead, the anchor
        // and one selected row, and running it on every frame of a held button leaves exactly the
        // same three facts, so a plain click cannot tell the level from the edge. A ctrl-click
        // *toggles*, so on the level it flickers: selected, not selected, selected, once a frame
        // for as long as the button is down. Watched failing with `press_edge = resp.pressed`.
        let (_, applied) = run(
            &[
                None,
                Some(MouseKind::Move),
                Some(MouseKind::Down(Button::Left)),
                None,
                None,
                Some(MouseKind::Up(Button::Left)),
                None,
            ],
            Mods::CTRL,
        );
        assert_eq!(
            applied,
            vec![0, 0, 0, 1, 1, 1, 1],
            "one press is one gesture however many frames it is held for: {applied:?}",
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

    // ── components ticket 17: `tree` ─────────────────────────────────────────────────────────────

    /// **Criterion: `tree` is `collection` plus a flatten index, and nothing else.**
    ///
    /// The sentence read out of the file rather than asserted about it, which is components ticket
    /// 15's arrangement one component over: *no second selection store, no second scan cursor, no
    /// second `Mode`, no second offset, no second press edge*. The one thing `tree` adds to the
    /// row axis is a **request**, and the request is not a store — it is one slot the caller
    /// drains.
    #[test]
    fn a_tree_is_a_collection_and_the_sentence_is_read_out_of_the_file() {
        let source = include_str!("collect.rs");
        assert!(
            crate::dense::declares(source, "collection_chorded("),
            "`tree_with` no longer reaches `collection`, so `tree = collection + a flatten index` \
             has stopped being a claim about this file"
        );
        // Counted by declaration and not by mention, which is `table`'s own scan two tests down:
        // a doc comment naming `Mode` is not a second `Mode`.
        for declaration in [
            "pub enum Mode {",
            "pub struct Selection {",
            "pub struct Scan<'a> {",
            "pub struct CollState {",
        ] {
            let n = source
                .lines()
                .map(str::trim)
                .filter(|l| !l.starts_with("//") && l.starts_with(declaration))
                .count();
            assert_eq!(
                n, 1,
                "`{declaration}` is declared twice, so one of the components has a store of its own"
            );
        }
        // **`TreeState` holds the row axis rather than replacing it**, which is the half a count
        // cannot state.
        assert!(
            source.contains("pub coll: CollState,"),
            "`TreeState` no longer holds a `CollState`"
        );
        assert_eq!(
            size_of::<TreeState>(),
            size_of::<CollState>() + size_of::<Asked>(),
            "a tree is a collection and one slot, to the byte"
        );
    }

    /// **Criterion: the `+` costs two verbs a row — the indent run and the chevron cell.**
    ///
    /// §7's own sentence, measured against the same rectangle drawn as a plain list. The absolute
    /// figures are `crate::forest`'s screen; what is asserted here is the *difference*, which is the
    /// mechanism.
    ///
    /// And the second half, which is §2: **the row drawer is handed what the component did not
    /// write**, so the row is a partition — `writes == distinct` over the whole frame.
    #[test]
    fn the_plus_is_two_verbs_a_row_and_the_row_is_a_partition() {
        fn frame(as_tree: bool, depth: u16) -> (u64, u64, u64) {
            let index = Order::built(
                (0..1_000u32)
                    .map(|n| Entry::of(n).at_depth(depth))
                    .collect(),
            );
            let mut driver = crate::runner::driver_at(60, 8, vitui_runtime::Density::default());
            let mut st = TreeState::new();
            let mut coll = CollState::new();
            let mut out = (0, 0, 0);
            // A warm frame first: the frame structures take their allocation once, and a cold
            // frame is not a frame.
            for _ in 0..2 {
                let mut tally = crate::counters::Tally::new();
                driver.frame(|cx| {
                    let area = cx.area();
                    let body = cx.theme().paint(Role::Body);
                    let label =
                        move |ink: &mut crate::counters::Tally, cx: &mut Ctx<'_, '_>, r: Rect| {
                            let cut = truncate("a-node-label", r.w);
                            let used = width(cut);
                            let _ = ink.text(cx, r.x, r.y, cut, body);
                            let _ = ink.run(cx, r.x + i32::from(used), r.y, " ", r.w - used, body);
                        };
                    if as_tree {
                        let _ = tree_into(
                            &mut tally,
                            cx,
                            area,
                            &mut st,
                            &TreeOpts::default(),
                            &index,
                            |_: &str, _: Range<usize>| None,
                            |ink: &mut crate::counters::Tally,
                             cx: &mut Ctx<'_, '_>,
                             r: Rect,
                             _n: Node,
                             _f: Face| label(ink, cx, r),
                        );
                    } else {
                        let _ = collection_into(
                            &mut tally,
                            cx,
                            area,
                            &mut coll,
                            &CollOpts::default(),
                            index.rows(),
                            |_: &str, _: Range<usize>| None,
                            |ink: &mut crate::counters::Tally,
                             cx: &mut Ctx<'_, '_>,
                             r: Rect,
                             _i: usize,
                             _f: Face| label(ink, cx, r),
                        );
                    }
                });
                out = (tally.verbs(), tally.writes(), tally.distinct());
            }
            out
        }

        let (list, list_writes, list_distinct) = frame(false, 0);
        let (tree, tree_writes, tree_distinct) = frame(true, 3);
        assert_eq!(
            tree - list,
            2 * 8,
            "§7: the `+` costs two verbs a row — the indent run and the chevron cell — over eight \
             rows"
        );
        // **§2, both ways.** The same cells, each written once, whichever component drew them.
        assert_eq!((tree_writes, tree_distinct), (60 * 8, 60 * 8));
        assert_eq!((list_writes, list_distinct), (60 * 8, 60 * 8));
    }

    /// **Criterion: the indent is clamped, and the label goes where the indent says.**
    ///
    /// The equality is what stops [`indent_columns`] and the component from being a model and a
    /// copy of one: `crate::forest` sums the function and the component draws the run, and the
    /// rectangle handed to the row drawer is where the two meet.
    ///
    /// The other direction is [`defective::unclamped_indent`], whose label rectangle is **empty**
    /// at a depth the clamped one still fits — which is the only observable that separates them
    /// this side of `crate::forest`'s ask.
    #[test]
    fn the_component_places_the_label_where_the_indent_says() {
        fn placed(indent: Indent, depth: u16, w: u16) -> Vec<(i32, u16)> {
            let index = Order::built((0..4u32).map(|n| Entry::of(n).at_depth(depth)).collect());
            let mut driver = crate::runner::driver_at(w, 4, vitui_runtime::Density::default());
            let mut st = TreeState::new();
            let mut out = Vec::new();
            driver.frame(|cx| {
                let area = cx.area();
                let mut find = |_: &str, _: Range<usize>| None;
                let mut row =
                    |_ink: &mut Direct, _cx: &mut Ctx<'_, '_>, r: Rect, _n: Node, _f: Face| {
                        out.push((r.x, r.w))
                    };
                let _ = match indent {
                    Indent::Clamped => tree_into(
                        &mut Direct,
                        cx,
                        area,
                        &mut st,
                        &TreeOpts::default(),
                        &index,
                        &mut find,
                        &mut row,
                    ),
                    Indent::Unclamped => defective::unclamped_indent(
                        &mut Direct,
                        cx,
                        area,
                        &mut st,
                        &TreeOpts::default(),
                        &index,
                        &mut find,
                        &mut row,
                    ),
                };
            });
            out
        }

        // At depth three the indent is six columns and both builds agree, which is the statement
        // about the scene list: **a shallow tree cannot tell them apart.**
        assert_eq!(placed(Indent::Clamped, 3, 40), vec![(7, 33); 4]);
        assert_eq!(
            placed(Indent::Unclamped, 3, 40),
            placed(Indent::Clamped, 3, 40)
        );
        for (x, w) in placed(Indent::Clamped, 3, 40) {
            assert_eq!(x, indent_columns(Indent::Clamped, 3, 40) as i32 + 1);
            assert_eq!(w, 40 - 7);
        }

        // At a depth past the rectangle the clamp reserves two columns and the defect leaves none.
        assert_eq!(placed(Indent::Clamped, 400, 40), vec![(39, 1); 4]);
        assert_eq!(
            placed(Indent::Unclamped, 400, 40),
            vec![(40, 0); 4],
            "the label rectangle has collapsed, and the row drawer is still called with it"
        );
    }

    /// **A tree handed a rectangle that does not start at column zero draws inside it.**
    ///
    /// The lesson components ticket 15 paid for one component over: `table`'s `header_row` drew its
    /// bands from `x = 0` rather than from the rectangle it was given, a table inside a panel put
    /// its header one column into the border, and **every gate in the crate passed**, because every
    /// one of them plays at `x == 0` where the two agree.
    ///
    /// So this one does not play there. What it can assert and what it cannot are both worth being
    /// exact about, because the difference is the whole of why `table` had the defect and `tree`
    /// cannot:
    ///
    /// - **Everything `tree` draws goes through `collection`'s scroll scope**, which childs at the
    ///   body's rectangle — so there is exactly **one** coordinate space and both recorders union in
    ///   it. `table`'s header is the thing that did not: it draws in the *caller's* context, beside a
    ///   body drawn in the scope's, and the two disagreed about where column zero was.
    /// - So the assertion is not *nothing before the margin* — a recorder that never sees the margin
    ///   cannot answer that. It is that the component partitions **the width it was handed**, writes
    ///   nothing past it, and measures every row's indent from `r.x` rather than from zero. Reaching
    ///   for `area.w` instead of `r.w`, or for `0` instead of `r.x`, moves one of those.
    #[test]
    fn a_tree_at_a_non_zero_origin_draws_inside_the_rectangle_it_was_given() {
        const MARGIN: u16 = 5;
        const W: u16 = 40;
        const H: u16 = 4;

        let index = Order::built(vec![
            Entry::of(0),
            Entry::of(1).at_depth(1),
            Entry::of(2).at_depth(2),
            Entry::of(3),
        ]);
        let mut driver = crate::runner::driver_at(W, H, vitui_runtime::Density::default());
        let mut st = TreeState::new();
        let mut pen = crate::runner::Pen::new(W, H);
        let mut handed: Vec<Rect> = Vec::new();
        driver.frame(|cx| {
            let area = cx.area();
            let inside = Rect::new(i32::from(MARGIN), 0, area.w - MARGIN, area.h);
            let body = cx.theme().paint(Role::Body);
            let _ = tree_into(
                &mut pen,
                cx,
                inside,
                &mut st,
                &TreeOpts::default(),
                &index,
                |_: &str, _: Range<usize>| None,
                |ink: &mut crate::runner::Pen,
                 cx: &mut Ctx<'_, '_>,
                 r: Rect,
                 _n: Node,
                 _f: Face| {
                    handed.push(r);
                    let _ = ink.run(cx, r.x, r.y, "x", r.w, body);
                },
            );
        });

        // **The width it was handed, partitioned.** Not `W`: a component that read the context's
        // width instead of its rectangle's would write 40 columns a row here and still look right.
        let cells = u64::from(W - MARGIN) * u64::from(H);
        assert_eq!(
            (pen.tally().writes(), pen.tally().distinct()),
            (cells, cells),
            "a partition of the rectangle it was handed, and of nothing else"
        );
        // **And nothing outside the rectangle, in the coordinate space the surface is in.**
        //
        // The margin is on the **left** here and it used to be read on the right, because until
        // components ticket 19 `Pen` recorded a verb where it was *called* rather than where it
        // landed: a component narrowed to `x == MARGIN` recorded its first cell at column 0. That
        // is runtime issue 32's whole subject, and this test is where the correction is visible —
        // the assertion did not change its meaning, it changed which columns satisfy it.
        for y in 0..H {
            for x in 0..MARGIN {
                assert!(
                    pen.canvas().get(x, y).is_none(),
                    "column {x} of row {y} is left of the rectangle and was written"
                );
            }
            assert!(
                pen.canvas().get(W - 1, y).is_some(),
                "the rectangle reaches the last column and row {y} did not"
            );
        }
        // The rectangles handed over are measured from `r.x`, two indent columns a level plus the
        // chevron — and their width is what is left of `r.w`, not of the context.
        assert_eq!(
            handed,
            vec![
                Rect::new(1, 0, W - MARGIN - 1, 1),
                Rect::new(3, 1, W - MARGIN - 3, 1),
                Rect::new(5, 2, W - MARGIN - 5, 1),
                Rect::new(1, 3, W - MARGIN - 1, 1),
            ],
            "the indent is two columns a level and the chevron is one, measured from the row's own \
             origin"
        );
    }

    /// **Criterion: `←` and `→` leave a request, and they do not move the cursor.**
    ///
    /// The whole of [`Refusal`] in one measurement. `crate::nav::step` reads `←` as `↑`, so a tree
    /// whose fold keys arrived after `collection`'s own would find the cursor already moved — and
    /// a tree that read them *before* by draining the queue itself would leave `collection` with
    /// nothing, because `Ctx::decline` ends the level's turn.
    ///
    /// Both directions: the same key through a plain [`collection`] **does** move the cursor.
    #[test]
    fn the_fold_keys_leave_a_request_and_do_not_move_the_cursor() {
        use vitui_runtime::keys::Code;

        // A root with two children, then a second root: row 0 has children and row 1 does not.
        let index = Order::built(vec![
            Entry::of(0),
            Entry::of(1).at_depth(1),
            Entry::of(2).at_depth(1),
            Entry::of(3),
        ]);

        fn play(index: &Order, code: Code, as_tree: bool) -> (Option<Ask>, usize) {
            let mut driver = crate::runner::driver_at(20, 4, vitui_runtime::Density::default());
            let mut st = TreeState::new();
            let mut coll = CollState::new();
            st.coll.sel.lead = 0;
            for frame in 0..2 {
                if frame == 1 {
                    driver.post_key(crate::keys::press(vitui_runtime::Chord::new(code)));
                }
                driver.frame(|cx| {
                    let area = cx.area();
                    let seated = if as_tree {
                        tree(
                            cx,
                            area,
                            &mut st,
                            &TreeOpts::default(),
                            index,
                            &mut |_: &str, _: Range<usize>| None,
                            &mut |_cx: &mut Ctx<'_, '_>, _r: Rect, _n: Node, _f: Face| {},
                        )
                    } else {
                        collection(
                            cx,
                            area,
                            &mut coll,
                            &CollOpts::default(),
                            index.rows(),
                            &mut |_: &str, _: Range<usize>| None,
                            &mut |_cx: &mut Ctx<'_, '_>, _r: Rect, _i: usize, _f: Face| {},
                        )
                    };
                    if cx.focused().is_none() {
                        cx.focus(seated.id);
                    }
                });
            }
            match as_tree {
                true => (st.ask.standing(), st.coll.sel.lead),
                false => (None, coll.sel.lead),
            }
        }

        // `←` on a node with children: a request, and the cursor stayed.
        assert_eq!(
            play(&index, Code::Left, true),
            (Some(Ask::Collapse(0)), 0),
            "`←` folds the cursor's row and leaves the cursor where it was"
        );
        // **The same key through a plain collection moves the cursor**, which is what the hook is
        // for and what a tree without one would inherit.
        assert_eq!(play(&index, Code::Left, false).1, 0, "`←` is `↑` at row 0");
        assert_eq!(
            play(&index, Code::Down, false).1,
            1,
            "and `↓` is `↓`, so the harness really does deliver a key"
        );

        // `→` on a node that is not folded is **not** the tree's, so `collection` takes it and the
        // cursor moves. A hook that swallowed every arrow would pass the assertion above and lose
        // the keyboard.
        assert_eq!(
            play(&index, Code::Right, true),
            (None, 1),
            "`→` on an expanded node is a cursor move and not a request"
        );

        // And on a folded one it is a request. One value changed.
        let mut folded = index.clone();
        let _ = folded.fold(0);
        assert_eq!(
            play(&folded, Code::Right, true),
            (Some(Ask::Expand(0)), 0),
            "`→` expands a folded node"
        );
    }

    /// **Criterion: a folded node, an expanded one and a leaf wear three different chevrons.**
    ///
    /// §16's `ArrowDown`/`ArrowRight` pair is `tree`'s, and the third state is a space rather than
    /// nothing at all — so every row of a tree is the same partition of its rectangle whatever it
    /// is. `has_children` is `O(1)`, which is the half that matters: asking `Order::descendants`
    /// would make one chevron cost a subtree walk.
    #[test]
    fn a_chevron_says_folded_expanded_or_leaf_and_the_test_is_one_row_ahead() {
        let index = Order::built(vec![Entry::of(0), Entry::of(1).at_depth(1), Entry::of(2)]);
        assert!(has_children(&index, 0), "the next row is deeper");
        assert!(!has_children(&index, 1), "the next row is not");
        assert!(!has_children(&index, 2), "there is no next row");

        let mut driver = crate::runner::driver_at(20, 3, vitui_runtime::Density::default());
        let mut st = TreeState::new();
        let mut seen: Vec<(bool, bool)> = Vec::new();
        driver.frame(|cx| {
            let area = cx.area();
            let _ = tree(
                cx,
                area,
                &mut st,
                &TreeOpts::default(),
                &index,
                &mut |_: &str, _: Range<usize>| None,
                &mut |_cx: &mut Ctx<'_, '_>, _r: Rect, n: Node, _f: Face| {
                    seen.push((n.folded, n.leaf));
                },
            );
        });
        assert_eq!(seen, vec![(false, false), (false, true), (false, true)]);

        // Folded is not leaf, and the difference is the whole reason there are two bits.
        let mut folded = index.clone();
        let _ = folded.fold(0);
        assert_eq!(folded.len(), 2, "the child went out of the index");
        assert!(!has_children(&folded, 0), "and so it has none to find");
        assert!(
            folded.at(0).expect("the root").is_folded(),
            "which is exactly why the flag is in the record and not derived"
        );
    }

    /// **Criterion: two trees on one screen are two trees, and a row's id survives a fold above it.**
    ///
    /// ADR 0027 on the container axis, and the second half is why [`Node::id`] is keyed on the
    /// caller's **node** rather than on the display position: a fold above a row moves the position
    /// and does not move the row, so a position-keyed id would hand the row's target to whatever
    /// slid into its place.
    #[test]
    fn identity_is_keyed_on_the_node_and_two_trees_are_two_trees() {
        fn ids(index: &Order) -> Vec<Id> {
            let mut driver = crate::runner::driver_at(20, 8, vitui_runtime::Density::default());
            let mut left = TreeState::new();
            let mut right = TreeState::new();
            let mut out = Vec::new();
            driver.frame(|cx| {
                let area = cx.area();
                let mut collect = |_cx: &mut Ctx<'_, '_>, _r: Rect, n: Node, _f: Face| {
                    out.push(n.id);
                };
                let _ = tree(
                    cx,
                    area,
                    &mut left,
                    &TreeOpts::default(),
                    index,
                    &mut |_: &str, _: Range<usize>| None,
                    &mut collect,
                );
                let _ = tree(
                    cx,
                    area,
                    &mut right,
                    &TreeOpts::default(),
                    index,
                    &mut |_: &str, _: Range<usize>| None,
                    &mut collect,
                );
            });
            out
        }

        let index = Order::built(vec![
            Entry::of(10),
            Entry::of(11).at_depth(1),
            Entry::of(12),
        ]);
        let both = ids(&index);
        assert_eq!(both.len(), 6, "three rows in each of two trees");
        let (first, second) = both.split_at(3);
        assert_ne!(first[0], second[0], "two trees on one screen are two trees");
        let all: std::collections::HashSet<Id> = both.iter().copied().collect();
        assert_eq!(all.len(), 6, "and no two rows anywhere share an id");

        // **A fold above a row does not move the row's id.** Row `12` slides from position 2 to
        // position 1 and keeps the id it had.
        let mut folded = index.clone();
        let _ = folded.fold(0);
        let after = ids(&folded);
        assert_eq!(folded.len(), 2);
        assert_eq!(
            after[1], first[2],
            "the id followed the node and not the position"
        );

        // **`merges == 0` with two trees on screen, and the row-keyed spelling is not available
        // here to fail against** — a tree keys per *node* and there is no coarser key a row could
        // take. What the pair separates instead is `with_key` from `Id::keyed`: `Ctx::with_key`
        // inside a scroll scope re-childs at the content's origin, so the arm that uses it draws
        // nothing past the first screenful (runtime architecture issue 31, register row 112).
        let (regions, merges) = declared(&index);
        assert_eq!(merges, 0, "two trees on one screen merge nothing");
        assert_eq!(
            regions,
            2 * (1 + 3),
            "one hit entry a tree — the component's own, whatever the row count — plus one a \
             visible row that declared a target of its own"
        );
    }

    /// Two trees over `index`, each row declaring a target: `(regions, merges)`.
    fn declared(index: &Order) -> (usize, u64) {
        let mut driver = crate::runner::driver_at(40, 3, vitui_runtime::Density::default());
        let (mut left, mut right) = (TreeState::new(), TreeState::new());
        driver.frame(|cx| {
            let area = cx.area();
            let w = area.w / 2;
            let mut find = |_: &str, _: Range<usize>| None;
            let mut row = |_ink: &mut Direct, cx: &mut Ctx<'_, '_>, r: Rect, n: Node, _f: Face| {
                let _ = cx.interact(n.id, r, Interest::CLICK);
            };
            let _ = tree_into(
                &mut Direct,
                cx,
                Rect::new(0, 0, w, area.h),
                &mut left,
                &TreeOpts::default(),
                index,
                &mut find,
                &mut row,
            );
            let _ = tree_into(
                &mut Direct,
                cx,
                Rect::new(i32::from(w), 0, w, area.h),
                &mut right,
                &TreeOpts::default(),
                index,
                &mut find,
                &mut row,
            );
        });
        let frame = driver.inspect();
        (frame.hits().len(), u64::from(frame.ids().merges()))
    }

    /// **Criterion: the frame does not move with `Entry::h`.**
    ///
    /// §7's *the frame does not move at all* — 48.88 against 48.79, identical writes and verbs — and
    /// here it is a fact about what the component reads rather than a coincidence: `tree` draws one
    /// screen row a content row and never touches `h`, because the prefix sum is the caller's
    /// (`order::Heights`) and so is the stepping that uses it. The gate is the equality; the ~1.00×
    /// on the clock is `examples/tree_numbers.rs`'s.
    #[test]
    fn the_frame_does_not_move_with_the_height_field() {
        fn frame(varying: bool) -> (u64, u64, u64) {
            let mut entries: Vec<Entry> = (0..64u32).map(Entry::of).collect();
            if varying {
                for (i, e) in entries.iter_mut().enumerate() {
                    e.h = u8::try_from(1 + i % 4).expect("small");
                }
            }
            let index = Order::built(entries);
            let mut driver = crate::runner::driver_at(40, 8, vitui_runtime::Density::default());
            let mut st = TreeState::new();
            let mut out = (0, 0, 0);
            for _ in 0..2 {
                let mut tally = crate::counters::Tally::new();
                driver.frame(|cx| {
                    let area = cx.area();
                    let body = cx.theme().paint(Role::Body);
                    let _ = tree_into(
                        &mut tally,
                        cx,
                        area,
                        &mut st,
                        &TreeOpts::default(),
                        &index,
                        |_: &str, _: Range<usize>| None,
                        |ink: &mut crate::counters::Tally,
                         cx: &mut Ctx<'_, '_>,
                         r: Rect,
                         _n: Node,
                         _f: Face| {
                            let _ = ink.run(cx, r.x, r.y, " ", r.w, body);
                        },
                    );
                });
                out = (tally.writes(), tally.distinct(), tally.verbs());
            }
            out
        }
        assert_eq!(
            frame(true),
            frame(false),
            "the component reads `h` nowhere, so a frame cannot move with it"
        );
        // And the field is not inert: the caller's own index does move with it.
        let mut entries: Vec<Entry> = (0..64u32).map(Entry::of).collect();
        for (i, e) in entries.iter_mut().enumerate() {
            e.h = u8::try_from(1 + i % 4).expect("small");
        }
        let varying = Order::built(entries);
        assert!(crate::order::Heights::needed(&varying));
        assert_eq!(
            crate::order::Heights::built(&varying).screen_rows(),
            16 * (1 + 2 + 3 + 4),
            "sixty-four rows of heights one to four"
        );
    }

    /// **Criterion: a press on the chevron asks, and a press anywhere else selects.**
    ///
    /// The tree reads [`CollState::press_edge`] rather than keeping a second `pressing` bool, which
    /// is the one fact `collection` already reconstructs — `Response::pressed` is a level, so a
    /// component that read it raw would re-ask for as long as the user leaned on the button.
    #[test]
    fn a_press_on_the_chevron_asks_and_a_press_on_the_label_does_not() {
        use vitui_runtime::{Button, Buttons, Mods, Mouse, MouseKind};

        let index = Order::built(vec![Entry::of(0), Entry::of(1).at_depth(1), Entry::of(2)]);

        fn click(index: &Order, at: (u16, u16), frames: u32) -> (Option<Ask>, usize) {
            let mut driver = crate::runner::driver_at(20, 4, vitui_runtime::Density::default());
            let mut st = TreeState::new();
            for frame in 0..frames {
                // **A move, then a press.** `Response::local` is this frame's pointer and the hit
                // index is the previous frame's, so a press over a position nothing has moved to
                // is a press over nothing — `crate::collect`'s own click harness one test up.
                if let Some(kind) = match frame {
                    1 => Some(MouseKind::Move),
                    2 => Some(MouseKind::Down(Button::Left)),
                    _ => None,
                } {
                    driver.post_mouse(Mouse {
                        x: at.0,
                        y: at.1,
                        kind,
                        buttons: Buttons::NONE,
                        mods: Mods::NONE,
                        at: Instant::now(),
                    });
                }
                driver.frame(|cx| {
                    let area = cx.area();
                    let _ = tree(
                        cx,
                        area,
                        &mut st,
                        &TreeOpts::default(),
                        index,
                        &mut |_: &str, _: Range<usize>| None,
                        &mut |_cx: &mut Ctx<'_, '_>, _r: Rect, _n: Node, _f: Face| {},
                    );
                });
            }
            (st.ask.standing(), st.coll.sel.lead)
        }

        // Column 0 of row 0 is the chevron: the root has children, so it is a request.
        assert_eq!(click(&index, (0, 0), 4), (Some(Ask::Collapse(0)), 0));
        // Column 5 of row 0 is the label: `collection`'s selection gesture and nothing else.
        assert_eq!(click(&index, (5, 0), 4), (None, 0));
        // Row 1's chevron sits two columns in, and row 1 is a leaf — so nothing is asked there
        // either, which is the half that keeps the chevron column from being a blanket.
        assert_eq!(click(&index, (2, 1), 4), (None, 1));
    }

    // ── components ticket 15: `table` ────────────────────────────────────────────────────────────

    /// **Criterion 1: `table` is `collection` plus a column rect split, and nothing else.**
    ///
    /// *No second selection store, no second scan cursor, no second `Mode`* has no expression a
    /// type system can carry, so it is read out of this file: the table's body calls
    /// `collection_into`, and the three things it must not have are counted by the declarations
    /// this module makes. [`crate::listing`]'s inversion is the precedent — *this function calls
    /// that one* has no expression a test can write, and a scan is the honest substitute.
    #[test]
    fn table_is_collection_plus_a_column_split_and_names_no_second_store() {
        let source = include_str!("collect.rs");
        assert!(
            crate::dense::declares(source, "    collection_into("),
            "`table_with` no longer draws through `collection`, so `table = collection + column \
             rectangles` has stopped being true of the code"
        );

        // **One `Mode`, one selection store, one scan cursor**, counted by their declarations
        // rather than asserted. A second of any of them is a second `pub enum Mode` / `pub struct
        // Selection` / `pub struct Scan` in this module, which is what the freeze homes here.
        for (kind, declaration) in [
            ("mode", "pub enum Mode {"),
            ("selection store", "pub struct Selection {"),
            ("scan cursor", "pub struct Scan<'a> {"),
        ] {
            let n = source
                .lines()
                .map(str::trim)
                .filter(|l| !l.starts_with("//") && l.starts_with(declaration))
                .count();
            assert_eq!(n, 1, "a second {kind} arrived in this module");
        }

        // And the cell store is not one of them: it *holds* `Selection`, which is the whole of
        // §6's *no second store*.
        let mut cells = CellSel::new();
        let sel: &mut Selection = cells.column_mut(0);
        sel.select_all(9);
        assert_eq!(cells.span_count(), 1);
    }

    /// **Criterion 2: the rect split runs once a frame over the declared columns and touches no
    /// row.**
    ///
    /// *Touches no row* is a fact about the signature and is asserted as one: [`solve_columns`]
    /// takes a width and a column list, so there is no row for it to reach. *Once a frame* is read
    /// out of the source, and the position matters as much as the count — the call has to be
    /// **above** the one that opens the row loop.
    #[test]
    fn the_column_solve_touches_no_row_and_runs_once_a_frame() {
        // No row in the signature, and the compiler is the one saying so.
        let _: fn(u16, &[Column]) -> Solved = solve_columns;

        let source = include_str!("collect.rs");
        let body = source
            .split_once("fn table_with<I, F, C>(")
            .expect("`table_with` is this file's")
            .1
            .split_once("\n/// The header's row")
            .expect("and it ends before the header split")
            .0;
        assert_eq!(
            body.matches("solve_columns(area.w, cols)").count(),
            1,
            "the solve runs more than once a frame"
        );
        let solve_at = body
            .find("solve_columns(area.w, cols)")
            .expect("it is there");
        let rows_at = body
            .find("collection_into(")
            .expect("the row loop is there");
        assert!(
            solve_at < rows_at,
            "the solve is inside the row loop, which is what *once a frame* forbids"
        );

        // And the answer does not depend on the row count, which is the same statement measured:
        // the sweep in `crate::grid` reports identical writes and verbs at 1k, 100k and 1M.
        let specs = crate::grid::columns(12);
        let a = solve_columns(300, &specs);
        let b = solve_columns(300, &specs);
        assert_eq!((a.n, a.content_w, a.view_w), (b.n, b.content_w, b.view_w));
    }

    /// **Criterion 5: a pinned column may not be elastic, enforced rather than documented.**
    ///
    /// Both directions. A `Weight` on a pinned column is ignored because [`Pin::width`] is the only
    /// reader of a pin's width and no [`Constraint`] reaches it; the same `Weight` on the same
    /// column unpinned is honoured, so the constraint is not being dropped on the floor.
    #[test]
    fn a_pinned_column_may_not_be_elastic() {
        let pinned = [
            Column::new(0, "pinned", Constraint::Weight(9)).pinned_left(8),
            Column::new(1, "free", Constraint::Weight(1)),
        ];
        let s = solve_columns(100, &pinned);
        assert_eq!(
            s.w[0], 8,
            "the pin's own width, not nine tenths of the table"
        );
        assert_eq!(s.left_w, 8);
        assert_eq!(s.view_w, 92, "and the band's denominator is what is left");
        assert_eq!(s.w[1], 92, "the one elastic column takes the whole band");

        let free = [
            Column::new(0, "was pinned", Constraint::Weight(9)),
            Column::new(1, "free", Constraint::Weight(1)),
        ];
        let t = solve_columns(100, &free);
        assert_eq!(t.w[0], 90, "unpinned, the same weight is honoured");

        // The two denominators §6 names: a pin claims a share of the *table*, a lane a share of the
        // *content*, and `max(viewport, Σ minima)` is not the table.
        let overflowing = [
            Column::new(0, "pin", Constraint::Fixed(10)).pinned_left(10),
            Column::new(1, "wide", Constraint::Fixed(500)),
        ];
        let u = solve_columns(100, &overflowing);
        assert_eq!((u.left_w, u.view_w, u.content_w), (10, 90, 500));
        assert_eq!(u.max_hoff(), 410);
    }

    /// **Criterion 6: cell selection is a run list per column key.**
    ///
    /// §6's `1 000 000 / 16 MB` against `1 / 16 B` on one header click, and the loss beside it so
    /// the trade is stated from both sides. The runs and the bytes are the gate; the microseconds
    /// are `examples/table_numbers.rs`'s and are a report.
    #[test]
    fn cell_selection_is_one_run_per_column_key_and_the_flattening_is_a_million() {
        let (per_column, flat) = header_click_costs(CELL_ROWS, CELL_COLS, 3);
        assert_eq!(per_column.0, 1, "one run, and it is one at any length");
        assert_eq!(per_column.1, SPAN_BYTES, "sixteen bytes");
        assert_eq!(
            flat.0, CELL_ROWS,
            "a column of a row-major flattening is a stride, and a stride of length one is not a \
             run"
        );
        assert_eq!(flat.1, CELL_ROWS * SPAN_BYTES);

        // Independent of the length, which is the half that makes it a store decision rather than
        // a measurement of this row count.
        let short = header_click_costs(1_000, CELL_COLS, 3);
        assert_eq!((short.0.0, short.0.1), (1, SPAN_BYTES));
        assert_eq!(short.1.0, 1_000);

        // The loss, bounded by the declared column count and not by the length.
        assert_eq!(row_click_costs(CELL_COLS, 17), (CELL_COLS, 1));
        assert_eq!(row_click_costs(4, 17), (4, 1));

        // **And the frame does not distinguish them at all**, which is why the store is decided
        // entirely by what a gesture costs: both answer a cell through one `Scan`.
        let mut cells = CellSel::new();
        cells.select_column(3, CELL_ROWS);
        assert!(cells.contains(999_999, 3));
        assert!(!cells.contains(999_999, 4));
        assert_eq!(cells.columns(), 1);
    }

    /// **Criterion 7: in-cell editing is `Option<(row, col)>`, the row half revalidates and the
    /// column half is a key.**
    #[test]
    fn the_editing_slot_is_one_position_and_the_column_half_is_a_key() {
        let mut st = TableState::new();
        assert_eq!(st.editing(), None);
        st.edit(42, 7);
        assert_eq!(st.editing(), Some((42, 7)));

        // **One position, so following a reorder is one call at any length.** The column half does
        // not move, because hiding and reordering columns move *positions* and a key is not one.
        st.follow(|row| Some(999_999 - row));
        assert_eq!(st.editing(), Some((999_957, 7)));

        // A row that the reorder removed closes the editor rather than pointing at a stranger.
        st.follow(|_| None);
        assert_eq!(st.editing(), None);

        // And the slot is one slot: there is nowhere here to put a value per row (ADR 0028).
        st.edit(1, 3);
        st.edit(2, 4);
        assert_eq!(st.editing(), Some((2, 4)));
        st.stop_editing();
        assert_eq!(st.editing(), None);
    }

    /// **The row half is revalidated against the order's revision, and it is `collection`'s
    /// revalidation rather than a second copy of one.**
    ///
    /// Both halves in one frame: an order the caller did not reconcile closes the editor **and**
    /// clears the cell selection, because every span in it is a row position. A caller that says
    /// it has carried the positions across keeps both.
    #[test]
    fn a_reorder_the_caller_did_not_reconcile_clears_the_cells_and_the_slot() {
        let sorted = |st: &mut TableState, rows: Rows| {
            let specs = crate::grid::columns(12);
            let opts = TableOpts::default();
            let mut driver = crate::runner::driver_at(60, 8, vitui_runtime::Density::default());
            driver.frame(|cx| {
                let area = cx.area();
                let _ = table(
                    cx,
                    area,
                    st,
                    &opts,
                    &specs,
                    rows,
                    &mut |_, _| None,
                    &mut |_, _, _, _| {},
                );
            });
        };

        let first = Rows::new(100, Revision::fresh());
        let mut st = TableState::new();
        sorted(&mut st, first);
        st.edit(9, 3);
        st.cells.select_column(3, 100);
        st.coll.sel.select_only(9);
        assert_eq!(st.cells.span_count(), 1);

        // The same length and the same data, in a different order: the revision is the only thing
        // that makes it noticeable.
        let resorted = Rows::new(100, Revision::fresh());
        sorted(&mut st, resorted);
        assert_eq!(st.editing(), None, "the editor closed");
        assert_eq!(
            st.cells.span_count(),
            0,
            "and every cell span was a position"
        );

        // The other direction: a caller that reconciled keeps both.
        let mut kept = TableState::new();
        sorted(&mut kept, first);
        kept.edit(9, 3);
        kept.cells.select_column(3, 100);
        let next = Rows::new(100, Revision::fresh());
        kept.coll.reconciled(next);
        sorted(&mut kept, next);
        assert_eq!(kept.editing(), Some((9, 3)));
        assert_eq!(kept.cells.span_count(), 1);
    }

    /// **Criterion 8: identity is per cell where a cell declares a target.**
    ///
    /// Two tables on one screen, every visible cell declaring one: `merges == 0` on the shipped
    /// build and every cell of a row inert but the first on the row-keyed one. §4's *one axis out,
    /// the same defect has a different arithmetic*, as the count the runtime already makes.
    ///
    /// **The screen is identical either way** — drawing does not consume an id — so a cell gate
    /// cannot see it. That is the whole of why the counter is `merges`.
    #[test]
    fn identity_is_per_cell_and_merges_is_the_only_counter_that_says_so() {
        let (keyed_regions, keyed_merges) = crate::grid::identity_merges(true);
        let (row_regions, row_merges) = crate::grid::identity_merges(false);
        let (inert_regions, inert_merges) = crate::grid::inert_cells();

        assert_eq!(keyed_merges, 0, "the shipped build merges nothing");
        assert_eq!(inert_merges, 0);
        assert!(
            row_merges > 0,
            "the row-keyed build is the defect and has stopped reproducing it"
        );

        // **Every cell of a row but the first is inert**, which is the arithmetic §4 states.
        let cells = keyed_regions - inert_regions;
        let rows_on_screen = row_regions - inert_regions;
        assert_eq!(
            u64::try_from(cells - rows_on_screen).expect("a count"),
            row_merges,
            "one target a row survives and the rest merge"
        );

        // And the component's own entry count is one a table, however many cells are on screen.
        assert_eq!(
            inert_regions, 2,
            "one hit entry per table, and there are two"
        );
    }

    /// **`Ctx::with_key` inside a scroll scope draws nothing past the first screenful, and that is
    /// why a cell's id is handed over rather than pushed.**
    ///
    /// [`Cell::id`] carries the argument; this is the measurement behind it. `Ctx::with_id` — which
    /// `with_key` is written on — re-childs the view at `self.area()`, and `area()` is
    /// `Rect::new(0, 0, w, h)` in the **current** coordinate system. Inside a scroll scope that
    /// origin is the content's, so the clip it intersects with is content rows `0..h` while the
    /// window is at the offset.
    ///
    /// Both directions and two offsets, so the day the runtime fixes it this test fails rather than
    /// quietly passing. Filed as `.scratch/vitui-runtime-architecture/issues/31`.
    #[test]
    fn a_with_key_inside_a_scroll_scope_draws_nothing_past_the_first_screenful() {
        let landed = |offset: i32, keyed: bool| {
            let id = vitui_runtime::Id::named("scope");
            let view = Rect::new(0, 0, 20, 8);
            let mut driver = crate::runner::driver_at(20, 8, vitui_runtime::Density::default());
            let mut cells = 0u32;
            driver.frame(|cx| {
                cx.scroll_scope(id, view, (0, offset), (0, 1_000), |cx| {
                    let paint = cx.theme().paint(Role::Body);
                    for y in cx.visible_rows() {
                        if keyed {
                            cx.with_key(y as u64, |cx| {
                                cells += u32::from(cx.text(0, y, "x", paint).cells);
                            });
                        } else {
                            cells += u32::from(cx.text(0, y, "x", paint).cells);
                        }
                    }
                });
            });
            cells
        };

        assert_eq!(landed(0, false), 8, "eight rows, eight cells");
        assert_eq!(
            landed(0, true),
            8,
            "and at offset zero the key costs nothing"
        );
        assert_eq!(
            landed(100, false),
            8,
            "the scope itself is correct at an offset"
        );
        assert_eq!(
            landed(100, true),
            0,
            "a `with_key` inside the scope clips the whole window away, which is why `table` \
             mints a cell's id with `Id::keyed` and hands it over"
        );
    }

    /// **The header writes a partition of its row, in the same columns the body writes.**
    ///
    /// And the reason it is off by default is asserted rather than described: with a header the
    /// body's view starts one row down, so the two land in two coordinate spaces and a recorder
    /// that unions in the coordinates of the `Ctx` the verb was called on cannot compare them.
    /// Components ticket 14's finding, inherited.
    #[test]
    fn the_header_writes_a_partition_of_its_row_in_the_same_columns() {
        let specs = crate::grid::columns(12);
        let one = |header: bool, x: i32| {
            let opts = TableOpts {
                header,
                ..TableOpts::default()
            };
            let mut st = TableState::new();
            st.hoff = crate::grid::HOFF;
            let mut driver = crate::runner::driver_at(300, 8, vitui_runtime::Density::default());
            let mut tally = crate::counters::Tally::new();
            driver.frame(|cx| {
                let area = Rect::new(x, 0, u16::try_from(300 - x).expect("a width"), 8);
                let _ = table_into(
                    &mut tally,
                    cx,
                    area,
                    &mut st,
                    &opts,
                    &specs,
                    Rows::of(1_000),
                    &mut |_: &str, _: Range<usize>| None,
                    |ink: &mut crate::counters::Tally,
                     cx: &mut Ctx<'_, '_>,
                     r: Rect,
                     _c: Cell,
                     _f: Face| {
                        let paint = cx.theme().paint(Role::Body);
                        let _ = ink.run(cx, r.x, r.y, "-", r.w, paint);
                    },
                );
            });
            (tally.writes(), tally.distinct(), tally.verbs(), tally)
        };

        let (bare_w, bare_d, bare_v, _) = one(false, 0);
        assert_eq!(bare_w, bare_d, "no cell twice");
        assert_eq!(bare_w, 300 * 8, "and every cell of the rectangle once");

        let (head_w, head_d, head_v, _) = one(true, 0);
        assert!(
            head_v > bare_v,
            "the header is verbs the body did not issue"
        );
        assert_eq!(
            head_w,
            300 * 8,
            "every cell of the rectangle, still written once"
        );

        // **This used to be 300, and it was the recorder's number rather than the table's.**
        // `Ctx::scroll_scope` childs at the body's view, so the body's content row 0 is one row
        // down on the terminal and row 0 *in its own coordinates* — which is exactly where the
        // header wrote. A `Tally` unioned in the coordinates of the `Ctx` the verb was called on,
        // so the two spaces landed on top of each other and the pair reported a double write of
        // precisely one header row on a table that has none.
        //
        // **Components ticket 19 moved the union into root coordinates** — `Ctx::origin`, runtime
        // architecture issue 32 — so the pair is meaningful over a table with a header now, and
        // `crate::grid` playing its scene without one is a choice rather than a workaround.
        assert_eq!(
            head_w - head_d,
            0,
            "a table with a header writes no cell twice, and the counter can finally say so"
        );
        assert_eq!(head_d, 300 * 8);
        assert_eq!(
            bare_w - bare_d,
            0,
            "and with no header, unchanged: the two readings were only ever different where a \
             component narrows"
        );

        // **The origin, and it is the half `vitui-apps`'s `ledger` found.** The body draws inside
        // `Ctx::scroll_scope`, which childs at the body's rectangle, so a body cell's `x` is
        // relative to the table; a header cell's is relative to the **caller's** context. Written
        // without `head.x` the header started at column 0 whatever the rectangle said, and every
        // assertion above still passed — they all play at `x == 0`, where the two agree. A table
        // handed the interior of a panel drew its header one column into the border.
        //
        // Offset the table by twelve. **Components ticket 19 changed how this half is asserted and
        // not what it asserts**: the recorder now unions in root coordinates, so both spaces are
        // one space, the correct table is a partition of its rectangle, and the *defect* is what
        // shows up as a double write instead of the correct build showing up as one.
        //
        // Written without `head.x` the header sits at root `0..288` while the body sits at
        // `12..300`: they share **276** columns, which the pair would report as 276 double writes,
        // and the header's twelve cells left of the body would land outside the rectangle
        // altogether. So the gate is the geometry as well as the pair, and both are read off the
        // recorder that already had the accessor.
        let (off_w, off_d, _, off_tally) = one(true, 12);
        assert_eq!(
            off_w,
            288 * 8,
            "the rectangle is 288 wide and still written once"
        );
        assert_eq!(
            off_d,
            288 * 8,
            "and the header's row is part of that partition rather than a second reading of the \
             body's"
        );
        assert_eq!(off_w - off_d, 0);
        assert!(
            off_tally.touched(299, 0),
            "the header reaches the rectangle's last column, so it is drawn from `head.x`"
        );
        assert!(
            !off_tally.touched(0, 0),
            "and nothing was written left of the rectangle: a header at column 0 is the defect \
             `vitui-apps`'s `ledger` found, one column into the panel's border"
        );
        let (flat_w, flat_d, ..) = one(false, 12);
        assert_eq!(
            (flat_w, flat_d),
            (288 * 8, 288 * 8),
            "and with no header the body alone is a partition wherever it starts"
        );
    }
}
