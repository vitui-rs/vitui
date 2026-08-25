//! **The order, the index and the memo: one structure, five names, and the revision that makes a
//! stale position noticeable.**
//!
//! Components ticket 13. Spec §10, [ADR 0030](../../docs/adr/0030-a-memo-key-is-every-input.md) and
//! half of
//! [ADR 0031](../../docs/adr/0031-a-collection-stores-positions-in-an-order-the-caller-owns.md).
//!
//! It is a **prefactor**: it lands before `table`, `tree`, `field` and `collapsible` so that none of
//! them invents a fifth name for the thing all five of them are.
//!
//! # One structure, five names
//!
//! `table`'s sort/filter order, `tree`'s flatten index, `textarea`'s wrap index, `collapsible`'s fold
//! index and `table`'s variable-row-height prefix sum are **one structure**: a caller-owned,
//! `O(1)`-indexed materialisation of a display order, built on the edit and **spliced rather than
//! rebuilt**. [`Entry`] is the record — `{ node, depth, flags, h }` — and [`USES`] is the five names
//! against it as a value, so *which of the four fields does a wrap index use* is answerable by the
//! machine rather than by a paragraph.
//!
//! It is **forced, not chosen**. A virtualised collection needs `O(1)` access to the *k*-th visible
//! row; neither a comparator, a predicate, a tree walk nor a wrap rule provides one; so the order
//! must be materialised, and materialising it is proportional to the data — **21 158 µs at a million
//! rows, 211 frame budgets**, if it is done in a frame. Moving it from the component to the caller
//! changes none of that. What makes it affordable is a memo on the edit, and what makes an edit
//! affordable is a splice.
//!
//! # The order is the caller's and a component may only ask
//!
//! **No [`Id`](vitui_runtime::Id) is anywhere near it**, because the index is keyed on the caller's
//! node ids — exactly the keys that may be written to disk — so ADR 0013's *an `Id` may never be
//! persisted* is satisfied by construction rather than by a rule.
//!
//! The component holds a **one-slot** request ([`Asked`]) drained by the caller after the draw, and
//! the arrangement is forced twice over: a `Vec::splice` allocates against a zero-allocation frame
//! budget, and taking `&mut` to the index while the draw holds it shared is `E0502`. Both halves are
//! runnable rather than asserted in prose — see [`Asked`]'s documentation, which carries the failing
//! case and its positive twin.
//!
//! > **The edit is three things in one order: splice the index, reconcile the positions, stamp the
//! > revision.**
//!
//! [`Order::splice`] is all three in one call, which is the only shape that makes the sentence
//! checkable: a caller who could do the first without the third would leave the revision behind and
//! get the permutation answer on the next frame, which is exactly the state ADR 0031 says is
//! indistinguishable from a correct screen.
//!
//! # Everything a collection stores is a position
//!
//! The selection spans, the `editing` slot, `offset` and `lead` are all positions — and **a sort
//! changes no data and no length, so nothing inside the component can notice**. The frame after a
//! sort draws a perfectly correct table with the wrong rows selected and the editor open on the
//! wrong row. The only thing that makes it noticeable is a revision on the *length*:
//! [`Rows`] is `{ len, rev }`, one `u64` compared once a frame, and
//! [`crate::collect::collection`] is where the comparison happens.
//!
//! # The four policies, and the one that is refused
//!
//! | edit | default | on request | refused |
//! |---|---|---|---|
//! | a permutation | [`Policy::Clear`] | [`Policy::Remap`] | — |
//! | a splice | [`Policy::Drop`] | [`Policy::Stash`] | [`Policy::Clear`] |
//!
//! `Clear` is the honest default for a permutation because reconciling one *shatters*: a half-table
//! selection is one span and sixteen bytes becoming hundreds of thousands of spans and megabytes.
//! `Remap` is the caller's opt-in **with the remap being the caller's function**, because only the
//! caller has both orders. `Clear` under a splice is refused by name: it throws away spans that were
//! nowhere near the fold for no saving at all — [`splice_vs_permutation`] is the measurement, and
//! [`Policy::admits`] is the refusal as a value.
//!
//! # A memo's key is every input
//!
//! [`Keyed`] is this crate's memo, and it differs from `vitui_runtime::Memo` in exactly the two ways
//! ADR 0030 says matter: **it takes the whole key** rather than one `Revision`, and it **records the
//! input it was built at** ([`Keyed::built_at`]) so a stale hit is detectable by something other
//! than the rendered screen.
//!
//! **The natural detector points the wrong way.** `recomputes` goes *down* when the key is wrong, so
//! the instrument the runtime provides for exactly this question reports improvement — and
//! `tests::a_memo_keyed_on_the_revision_alone_recomputes_less_and_is_wrong_on_the_screen` watches it
//! happening on the 300 → 120 resize, where the revision-keyed index draws a document at the
//! previous width and every counter approves.

use std::ops::Range;

use vitui_runtime::Revision;

use crate::collect::{Selection, Span};

// ── the record, and the five names ───────────────────────────────────────────────────────────────

/// **One row of a materialised display order.**
///
/// `{ node, depth, flags, h }` and no more, which is ADR 0031's *one structure, five names*: a
/// table's order is this record with three fields unused and a wrap index is it with one. Building
/// five is the mistake; three names for one mechanism is already one too many.
///
/// **No `Id`.** `node` is the *caller's* key — a row id, a byte offset, a line number — which is
/// what makes the index persistable and keeps ADR 0013's rule satisfied by construction.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Entry {
    /// The caller's own key for whatever this row shows. Never an [`Id`](vitui_runtime::Id).
    pub node: u64,
    /// How deep it sits. **Not for the indent** — it is there so a collapse can find the interval it
    /// removes without touching the forest: 115 µs against 1 187 at 349 524 rows (ADR 0028).
    pub depth: u16,
    /// Caller-defined bits: expanded, folded, filtered, continuation.
    pub flags: u16,
    /// How many screen rows it occupies. **The fourth field, and it is why variable row height is a
    /// field rather than a fifth structure** (§7).
    pub h: u16,
}

/// One of the five things [`Order`] is, and which of [`Entry`]'s fields it spends.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Use {
    /// The component that has one.
    pub owner: &'static str,
    /// What it is called there.
    pub name: &'static str,
    /// Which of `Entry`'s four fields it uses.
    pub fields: &'static [&'static str],
    /// Where the spec says so.
    pub section: &'static str,
}

/// **The five names one structure has**, as a value rather than as a paragraph.
///
/// The column that matters is `fields`: *a table's order is the same record with three fields unused
/// and a wrap index is the same record with one* is a claim about this table, and
/// `tests::one_structure_has_five_names_and_every_field_is_earned` is what makes it checkable.
pub const USES: [Use; 5] = [
    Use {
        owner: "table",
        name: "the sort/filter order",
        fields: &["node"],
        section: "spec §6",
    },
    Use {
        owner: "tree",
        name: "the flatten index",
        fields: &["node", "depth", "flags"],
        section: "spec §7",
    },
    Use {
        owner: "field",
        name: "the wrap index",
        fields: &["node"],
        section: "spec §11",
    },
    Use {
        owner: "collapsible",
        name: "the fold index",
        fields: &["node", "flags"],
        section: "spec §8",
    },
    Use {
        owner: "table",
        name: "the variable-row-height prefix sum",
        fields: &["node", "h"],
        section: "spec §6, §7",
    },
];

// ── the length and its revision ──────────────────────────────────────────────────────────────────

/// **`{ len, rev }` — one `u64` compared once a frame, and the only thing that makes a stale
/// position noticeable.**
///
/// A sort changes no data and no length, so nothing inside a collection can notice it: the frame
/// after one draws a perfectly correct table with the wrong rows selected and the editor open on the
/// wrong row. The revision is what a component compares, and [`crate::collect::collection`] compares
/// it exactly once.
///
/// [`Rows::of`] is the spelling for a caller with no order at all — a `&[T]` handed straight in —
/// and it carries [`Revision::UNKNOWN`], which never matches, so such a caller is never told its
/// positions went stale. That is correct: a caller with no order cannot permute one.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Rows {
    /// How many rows the order holds.
    pub len: usize,
    /// The revision the order is at.
    pub rev: Revision,
}

impl Rows {
    /// `len` rows at [`Revision::UNKNOWN`] — a caller with no order, whose positions cannot go
    /// stale because nothing can permute them.
    pub const fn of(len: usize) -> Rows {
        Rows {
            len,
            rev: Revision::UNKNOWN,
        }
    }

    /// `len` rows at a stated revision.
    pub const fn new(len: usize, rev: Revision) -> Rows {
        Rows { len, rev }
    }
}

/// **How many bytes the revision costs on the wire between a caller and a component. Eight.**
///
/// §10's *one `u64` compared once a frame*, as a measurement rather than as a sentence.
pub const REVISION_BYTES: usize = size_of::<Revision>();

// ── the order ────────────────────────────────────────────────────────────────────────────────────

/// **A caller-owned, `O(1)`-indexed materialisation of a display order.**
///
/// Built on the edit and **spliced rather than rebuilt** — 157–342 µs against 40 393 for a tree
/// collapse, 34.34 µs against 3 216 for a keystroke at 1 MB, 129.42 against 2 977.71 for a document
/// insert above 4 167 folds.
///
/// The revision is the order's own and is stamped by [`Order::splice`] and [`Order::permute`]. A
/// caller that reaches past both and edits the entries directly cannot: `entries` is private, which
/// is the *anything else that moves the index leaves the revision behind* half of ADR 0031 as a
/// visibility rather than as a rule.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Order {
    entries: Vec<Entry>,
    rev: Revision,
}

impl Default for Order {
    fn default() -> Order {
        Order::new()
    }
}

impl Order {
    /// An empty order at [`Revision::UNKNOWN`].
    pub const fn new() -> Order {
        Order {
            entries: Vec::new(),
            rev: Revision::UNKNOWN,
        }
    }

    /// **The materialisation, done once and not in a frame.**
    ///
    /// It is proportional to the data by construction — that is what materialising *is* — and §10
    /// prices doing it in a frame at 21 158 µs at a million rows, 211 frame budgets. What makes it
    /// affordable is that it happens on the edit and nowhere else.
    pub fn built(entries: Vec<Entry>) -> Order {
        Order {
            entries,
            rev: Revision::fresh(),
        }
    }

    /// The length and the revision, which is what a component is handed.
    pub const fn rows(&self) -> Rows {
        Rows {
            len: self.entries.len(),
            rev: self.rev,
        }
    }

    /// How many rows it holds.
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether it holds none.
    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// **The `k`-th row, in `O(1)`.** The whole reason the order is materialised at all.
    pub fn at(&self, i: usize) -> Option<&Entry> {
        self.entries.get(i)
    }

    /// Every row, for a caller building the next order out of this one.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// **Splice `range` out and `with` in, and stamp the revision it produced.**
    ///
    /// One call and not three, because *the edit is three things in one order* is only checkable if
    /// they cannot be done separately: a caller who spliced without stamping would leave the
    /// revision behind and get the permutation answer on the next frame — a correct-looking screen
    /// with the wrong rows selected.
    ///
    /// The returned [`Splice`] is what [`reconcile_splice`] takes, so the second of the three things
    /// is a value handed from the first to it rather than two numbers a caller recomputes.
    pub fn splice(&mut self, range: Range<usize>, with: impl IntoIterator<Item = Entry>) -> Splice {
        let start = range.start.min(self.entries.len());
        let end = range.end.clamp(start, self.entries.len());
        let before = self.entries.len();
        self.entries.splice(start..end, with);
        let inserted = self.entries.len() + (end - start) - before;
        self.rev = Revision::fresh();
        Splice {
            removed: start..end,
            inserted,
        }
    }

    /// **Reorder the rows and stamp the revision. The length does not change.**
    ///
    /// `to[i]` is where row `i` goes. A permutation is exactly the edit nothing inside a component
    /// can notice — no data moved and no length moved — which is why the revision is the whole
    /// mechanism and why [`Policy::Clear`] is its honest default.
    ///
    /// # Panics
    ///
    /// Panics unless `to` is a permutation of `0..len`. A partial one would silently drop rows, and
    /// an order that has quietly lost rows is the state this whole module exists to make noticeable.
    pub fn permute(&mut self, to: &[usize]) {
        assert_eq!(
            to.len(),
            self.entries.len(),
            "not a permutation of this order"
        );
        let mut seen = vec![false; to.len()];
        let mut out = vec![Entry::default(); to.len()];
        for (from, &at) in to.iter().enumerate() {
            assert!(at < to.len() && !seen[at], "not a permutation: {at} twice");
            seen[at] = true;
            out[at] = self.entries[from];
        }
        self.entries = out;
        self.rev = Revision::fresh();
    }
}

/// **What a [`Order::splice`] did**, and the input [`reconcile_splice`] takes.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Splice {
    /// The half-open interval that was removed, in the order's coordinates **before** the edit.
    pub removed: Range<usize>,
    /// How many rows went in where it was.
    pub inserted: usize,
}

impl Splice {
    /// How far everything after the interval moved. Negative when the order got shorter.
    pub fn shift(&self) -> isize {
        self.inserted as isize - (self.removed.end - self.removed.start) as isize
    }
}

// ── the four policies ────────────────────────────────────────────────────────────────────────────

/// Which kind of edit moved the order.
///
/// **Two and not one, because one answer does not cover both.** A subtree is contiguous in pre-order
/// display coordinates, so under a splice a sorted span list transforms in `O(spans)` and *cannot
/// shatter*; under a permutation of the same edit it shatters into one span a row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditKind {
    /// An interval was removed and another put in its place. A collapse, an expand, a keystroke.
    Splice,
    /// The rows are the same rows in a different order. A sort. **Nothing inside a component can
    /// notice one.**
    Permutation,
}

/// **What to do with the positions an edit moved.**
///
/// Four, and the interesting one is the one that is **refused**: `Clear` under a splice throws away
/// spans that were nowhere near the fold for no saving at all, and [`Policy::admits`] is that
/// refusal as a value rather than as a comment.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Policy {
    /// Throw the positions away. **The default for a permutation**, and the honest one: reconciling
    /// a permutation exactly costs a shattered span list and megabytes.
    Clear,
    /// Carry the positions across with the caller's own function. **The caller's opt-in**, because
    /// only the caller has both orders — and it carries the bound it costs, which is one span a row
    /// in the worst case.
    Remap,
    /// Drop the positions inside the removed interval and shift the rest. **The default for a
    /// splice.**
    Drop,
    /// [`Policy::Drop`], keeping what was dropped so the caller can put it back if the interval
    /// comes back. `Stash` on request.
    Stash,
}

impl Policy {
    /// All four, so a report iterates rather than samples.
    pub const ALL: [Policy; 4] = [Policy::Clear, Policy::Remap, Policy::Drop, Policy::Stash];

    /// **The settled default for each kind of edit.**
    pub const fn default_for(edit: EditKind) -> Policy {
        match edit {
            EditKind::Permutation => Policy::Clear,
            EditKind::Splice => Policy::Drop,
        }
    }

    /// **Whether this policy may be used for this edit.**
    ///
    /// `Clear` under a splice is the one `false`, and it is refused rather than merely discouraged:
    /// it throws away spans that were nowhere near the fold, for no saving at all — the three real
    /// splice policies are all `O(spans)` and all free, so there is nothing to buy.
    ///
    /// `Remap` under a splice is not offered either, and for a different reason: a splice *is*
    /// reconcilable exactly, so a caller's remap function would be doing arithmetic the component
    /// already has.
    pub const fn admits(self, edit: EditKind) -> bool {
        matches!(
            (self, edit),
            (Policy::Clear | Policy::Remap, EditKind::Permutation)
                | (Policy::Drop | Policy::Stash, EditKind::Splice)
        )
    }

    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Policy::Clear => "clear",
            Policy::Remap => "remap",
            Policy::Drop => "drop",
            Policy::Stash => "stash",
        }
    }
}

/// **Reconcile a selection against a splice. `O(spans)`, and it cannot shatter.**
///
/// Returns what was stashed, which is empty under [`Policy::Drop`] and the spans that fell inside
/// the removed interval under [`Policy::Stash`] — in the order's coordinates **before** the edit, so
/// a caller that puts the interval back can put them back with it.
///
/// The property that makes this cheap is that a splice is an **interval**: every span is entirely
/// before it, entirely after it, or crosses it, and the first two cases are a shift. A half-tree
/// contiguous selection is one span before and one span after.
///
/// # Panics
///
/// Panics on a policy the edit does not admit, which is [`Policy::Clear`] and [`Policy::Remap`]. A
/// silent fallback would make the refusal a comment.
pub fn reconcile_splice(sel: &mut Selection, splice: &Splice, policy: Policy) -> Vec<Span> {
    assert!(
        policy.admits(EditKind::Splice),
        "`{}` is not a splice policy. `Clear` throws away spans that were nowhere near the fold \
         for no saving at all, and `Remap` asks the caller for arithmetic a splice already has",
        policy.word()
    );
    let (lo, hi) = (splice.removed.start, splice.removed.end);
    let shift = splice.shift();
    let mut kept: Vec<Span> = Vec::new();
    let mut stashed: Vec<Span> = Vec::new();
    let moved = |i: usize| -> usize { (i as isize + shift).max(0) as usize };
    for span in sel.spans() {
        // Entirely before the interval: untouched.
        if span.end <= lo {
            kept.push(*span);
            continue;
        }
        // Entirely after it: shifted, and that is the whole cost.
        if span.start >= hi {
            kept.push(Span {
                start: moved(span.start),
                end: moved(span.end),
            });
            continue;
        }
        // It touches the interval. At most two pieces survive — a head and a tail — which is why a
        // splice cannot shatter a span list.
        if span.start < lo {
            kept.push(Span {
                start: span.start,
                end: lo,
            });
        }
        if span.end > hi {
            kept.push(Span {
                start: moved(hi),
                end: moved(span.end),
            });
        }
        if policy == Policy::Stash {
            stashed.push(Span {
                start: span.start.max(lo),
                end: span.end.min(hi),
            });
        }
    }
    // Coalescing, because a head and a tail either side of a fully removed interval are now
    // adjacent — and a span list that has stopped being non-adjacent breaks every
    // `partition_point` in `crate::collect`.
    let mut merged: Vec<Span> = Vec::with_capacity(kept.len());
    for span in kept.into_iter().filter(|s| !s.is_empty()) {
        match merged.last_mut() {
            Some(last) if last.end >= span.start => last.end = last.end.max(span.end),
            _ => merged.push(span),
        }
    }
    sel.set_spans(merged);
    stashed
}

/// **Reconcile a selection against a permutation.**
///
/// `Clear` throws it away, which is the default and the honest answer. `Remap` carries it across
/// with the caller's own function — `to(old) -> Option<new>` — and **it is the caller's because only
/// the caller has both orders**.
///
/// Select-all escapes both at `O(1)`, and not by special-casing: `[0, len)` is invariant under any
/// permutation of that length, so the remap of a full span is a full span whatever the function is.
/// [`splice_vs_permutation`] is where that stops being a sentence.
///
/// # Panics
///
/// Panics on a policy the edit does not admit.
pub fn reconcile_permutation(
    sel: &mut Selection,
    len: usize,
    policy: Policy,
    to: &dyn Fn(usize) -> Option<usize>,
) {
    assert!(
        policy.admits(EditKind::Permutation),
        "`{}` is not a permutation policy: a permutation moves every position and there is no \
         interval to shift",
        policy.word()
    );
    if policy == Policy::Clear {
        sel.clear();
        return;
    }
    // **The `O(1)` escape, and it is not a special case.** A full selection is `[0, len)`, which is
    // invariant under any permutation of that length — so the shattering below is skipped because
    // the answer is already known and not because the code checked for a convenient shape.
    if sel.spans().len() == 1 && sel.spans()[0] == (Span { start: 0, end: len }) {
        return;
    }
    let mut moved: Vec<usize> = Vec::new();
    for span in sel.spans() {
        for i in span.start..span.end {
            if let Some(at) = to(i) {
                moved.push(at);
            }
        }
    }
    moved.sort_unstable();
    moved.dedup();
    let mut spans: Vec<Span> = Vec::new();
    for i in moved {
        match spans.last_mut() {
            Some(last) if last.end == i => last.end = i + 1,
            _ => spans.push(Span {
                start: i,
                end: i + 1,
            }),
        }
    }
    sel.set_spans(spans);
}

/// **Reconcile one position — the editing slot, the cursor, the anchor — against a splice.**
///
/// `None` when the row it named was removed. One position, and §6's *0.29 µs to follow a million-row
/// sort, because it is one position* is a statement about this function's shape rather than about
/// its speed.
pub fn reconcile_position(at: Option<usize>, splice: &Splice) -> Option<usize> {
    let i = at?;
    if i < splice.removed.start {
        return Some(i);
    }
    if i < splice.removed.end {
        return None;
    }
    Some((i as isize + splice.shift()).max(0) as usize)
}

// ── the one-slot request ─────────────────────────────────────────────────────────────────────────

/// What a component asked the caller to do to the order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ask {
    /// Bring this node's children into the order.
    Expand(u64),
    /// Take this node's subtree out of the order.
    Collapse(u64),
    /// Re-sort the order on this key.
    Sort(u64),
    /// Re-filter it against this predicate key.
    Filter(u64),
}

/// **The one-slot request a component leaves behind, drained by the caller after the draw.**
///
/// One slot and not a queue, for the reason ADR 0016 gives the routing edge one edge: a frame that
/// could ask for two edits would need the caller to apply them in an order nobody stated, against an
/// index the second one's coordinates are computed in. The last ask of a frame wins, and there is
/// only ever one gesture in a frame that can produce one.
///
/// # The component may not perform the edit, and it is forced twice over
///
/// **A `Vec::splice` allocates**, against a frame budget of zero. And taking `&mut` to the index
/// while the draw holds it shared is `E0502` — which is a compile outcome and therefore a gate
/// rather than a sentence. The protected item is named by path in the positive twin first, so a
/// rename fails **here** and not silently in the failing case below:
///
/// ```
/// use vitui_components::order::{Ask, Asked, Entry, Order};
///
/// // Named by path and exercised: a rename or a changed signature fails HERE.
/// fn protected(order: &mut Order, range: std::ops::Range<usize>) -> vitui_components::order::Splice {
///     Order::splice(order, range, [Entry::default()])
/// }
///
/// let mut order = Order::built(vec![Entry::default(); 4]);
/// let mut asked = Asked::new();
/// // The draw asks. Nothing is edited yet.
/// asked.ask(Ask::Collapse(7));
/// // The caller drains it *after* the draw, when nothing borrows the order.
/// if let Some(Ask::Collapse(_)) = asked.drain() {
///     let splice = protected(&mut order, 1..3);
///     assert_eq!(splice.shift(), -1);
/// }
/// assert_eq!(order.len(), 3);
/// assert!(asked.drain().is_none(), "one slot, and it answers once");
/// ```
///
/// # The hostile half
///
/// **Protects:** [`Order::splice`].
///
/// A body that holds the order shared — which is what a row drawer reading `order.at(i)` *is* —
/// cannot also take `&mut` to it. That is the whole of *a component may only ask*, and it is
/// `E0502` rather than a rule anybody has to remember:
///
/// ```compile_fail,E0502
/// use vitui_components::order::{Entry, Order};
///
/// let mut order = Order::built(vec![Entry::default(); 4]);
/// let rows = order.entries();
/// // The draw is holding the order shared, and the edit wants it exclusively.
/// let _ = order.splice(1..3, [Entry::default()]);
/// let _ = rows[0];
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Asked {
    slot: Option<Ask>,
}

impl Asked {
    /// An empty slot.
    pub const fn new() -> Asked {
        Asked { slot: None }
    }

    /// Ask for an edit. **One slot**: the last ask of a frame wins.
    pub const fn ask(&mut self, ask: Ask) {
        self.slot = Some(ask);
    }

    /// Whether anything is standing, without taking it.
    pub const fn standing(&self) -> Option<Ask> {
        self.slot
    }

    /// **Take the ask.** It answers once, and a request nobody drains lives exactly one frame.
    pub const fn drain(&mut self) -> Option<Ask> {
        self.slot.take()
    }
}

/// **How many bytes the request costs. One slot and no `Rect`.**
pub const ASKED_BYTES: usize = size_of::<Asked>();

// ── the memo ─────────────────────────────────────────────────────────────────────────────────────

/// **A memo whose key is every input, and which records the input it was built at.**
///
/// ADR 0030. `vitui_runtime::Memo<T>::get` takes one `Revision` and cannot express any of the six
/// arrivals §10 tabulates — the width, the sort, the expansion set, the repertoire, the theme's
/// revision, the axis policy — so the components-side decision is that **the key is spelled out at
/// every call site**, and this is the type that makes spelling it possible.
///
/// # Two differences from `Memo`, and both are the ADR
///
/// 1. **It takes the whole key**, so a compound one is a tuple at the call site rather than a
///    hand-rolled hash a reviewer has to trust.
/// 2. **It records what it was built at** ([`Keyed::built_at`]), because the natural detector points
///    the wrong way: `recomputes` goes *down* when the key is wrong, so the instrument the runtime
///    provides for exactly this question reports improvement. The two detectors that work are the
///    memoised object recording its input, and the rendered surface.
///
/// ```
/// use vitui_components::order::Keyed;
///
/// // The key is the whole tuple, written out here where a reviewer can see it.
/// let mut wrapped: Keyed<(u64, u16), usize> = Keyed::new();
/// assert_eq!(*wrapped.get((7, 300), || 625), 625);
/// assert_eq!(wrapped.recomputes, 1);
///
/// // Same key, no recomputation.
/// assert_eq!(*wrapped.get((7, 300), || unreachable!("the memo is warm")), 625);
/// assert_eq!(wrapped.recomputes, 1);
///
/// // The width moved and the revision did not, which is the arrival §10 measures.
/// assert_eq!(*wrapped.get((7, 120), || 875), 875);
/// assert_eq!(wrapped.recomputes, 2);
/// assert_eq!(wrapped.built_at(), Some(&(7, 120)), "and it says what it was built at");
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Keyed<K, T> {
    key: Option<K>,
    value: Option<T>,
    /// How many times the closure has run.
    ///
    /// **Public, and it points the wrong way on purpose.** A memo with a key that has forgotten an
    /// input recomputes *less*, so this number going down is a symptom and not an improvement —
    /// which is why [`Keyed::built_at`] exists beside it.
    pub recomputes: u32,
}

impl<K: PartialEq, T> Keyed<K, T> {
    /// An empty memo.
    pub const fn new() -> Keyed<K, T> {
        Keyed {
            key: None,
            value: None,
            recomputes: 0,
        }
    }

    /// The cached value, computing it if the key is not the one it was computed at.
    pub fn get(&mut self, key: K, compute: impl FnOnce() -> T) -> &T {
        let hit = self.key.as_ref() == Some(&key) && self.value.is_some();
        if !hit {
            self.value = Some(compute());
            self.key = Some(key);
            self.recomputes += 1;
        }
        self.value
            .as_ref()
            .expect("the miss branch above just filled it")
    }

    /// **The input it was built at**, which is the detector that works.
    pub const fn built_at(&self) -> Option<&K> {
        self.key.as_ref()
    }

    /// What is cached, without computing anything.
    pub const fn peek(&self) -> Option<&T> {
        self.value.as_ref()
    }
}

impl<K: PartialEq, T> Default for Keyed<K, T> {
    fn default() -> Keyed<K, T> {
        Keyed::new()
    }
}

// ── the wrap index, which is the resize case ─────────────────────────────────────────────────────

/// **A wrap index and the width it was built at.**
///
/// C06's arrival, as a type: the index records its width, so *this document was wrapped at 300 and
/// is being drawn at 120* is answerable without looking at the screen. That is the first of ADR
/// 0030's two working detectors, and it is a field.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WrapIndex {
    order: Order,
    width: u16,
}

impl WrapIndex {
    /// Wrap `lines` at `width`. **Proportional to the data**, which is why it happens on the edit.
    pub fn built(lines: &[&str], width: u16) -> WrapIndex {
        let w = usize::from(width.max(1));
        let mut entries = Vec::new();
        for (n, line) in lines.iter().enumerate() {
            let rows = line.len().div_ceil(w).max(1);
            for r in 0..rows {
                entries.push(Entry {
                    node: n as u64,
                    depth: 0,
                    // Bit 0: a continuation row. The one field a wrap index spends beyond `node`.
                    flags: u16::from(r > 0),
                    h: 1,
                });
            }
        }
        WrapIndex {
            order: Order::built(entries),
            width,
        }
    }

    /// The order.
    pub const fn order(&self) -> &Order {
        &self.order
    }

    /// **The width it was built at.** The detector.
    pub const fn width(&self) -> u16 {
        self.width
    }

    /// How many display rows it holds.
    pub const fn rows(&self) -> usize {
        self.order.len()
    }

    /// Which source line the `i`-th display row belongs to, or `None` past the end.
    pub fn line_at(&self, i: usize) -> Option<u64> {
        self.order.at(i).map(|e| e.node)
    }
}

// ── the measurements ─────────────────────────────────────────────────────────────────────────────

/// **What one edit costs a selection, both ways.** §10's table, as a value.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EditCost {
    /// Spans before the edit.
    pub before: usize,
    /// Spans after it.
    pub after: usize,
    /// Bytes of span after it.
    pub bytes: usize,
    /// Nanoseconds the reconcile took, minimum of the rounds played.
    pub nanos: u128,
}

/// **The same edit, reconciled as a splice and as a permutation.**
///
/// §10's sharpest pair, and the reason `Clear` is the honest default for one and `Drop` for the
/// other: *a subtree is contiguous in pre-order display coordinates, so under a splice a sorted span
/// list transforms in `O(spans)` and cannot shatter.*
///
/// Returns `(splice, permutation)`. `removed` is the interval a collapse takes out, and the
/// selection is the contiguous half the two rows are measured over.
pub fn splice_vs_permutation(
    len: usize,
    removed: Range<usize>,
    rounds: u32,
) -> (EditCost, EditCost) {
    use std::time::Instant;

    let half = || {
        let mut sel = Selection::new();
        sel.insert(0, len / 2);
        sel
    };
    let splice = Splice {
        removed: removed.clone(),
        inserted: 0,
    };

    let mut spliced = EditCost {
        before: 1,
        after: 0,
        bytes: 0,
        nanos: u128::MAX,
    };
    for _ in 0..rounds.max(1) {
        let mut sel = half();
        let at = Instant::now();
        let _ = reconcile_splice(&mut sel, &splice, Policy::Drop);
        spliced.nanos = spliced.nanos.min(at.elapsed().as_nanos());
        spliced.after = sel.span_count();
        spliced.bytes = sel.bytes();
    }

    // **The permutation, remapped rather than cleared**, because `Clear` is O(1) and would price
    // nothing: what §10 measures is what reconciling one *exactly* costs, which is the number that
    // makes `Clear` the honest default.
    //
    // A deterministic shuffle rather than a random one — `i * 2 mod (len | 1)` visits every index
    // exactly once for an odd modulus — so the figure is replayable.
    let modulus = len | 1;
    let to = move |i: usize| Some((i * 2) % modulus);
    let mut permuted = EditCost {
        before: 1,
        after: 0,
        bytes: 0,
        nanos: u128::MAX,
    };
    for _ in 0..rounds.max(1) {
        let mut sel = half();
        let at = Instant::now();
        reconcile_permutation(&mut sel, len, Policy::Remap, &to);
        permuted.nanos = permuted.nanos.min(at.elapsed().as_nanos());
        permuted.after = sel.span_count();
        permuted.bytes = sel.bytes();
    }
    (spliced, permuted)
}

/// **An interval edit on the selection store, against a bit vector.** §21's register row 13.
///
/// `(spans_touched, bits_touched)`. The span list touches the spans that meet the interval — one,
/// for a contiguous selection — and a bit vector has to move every bit after it, because a bit's
/// position *is* its index and a splice moves all of them. That is the difference between a store
/// proportional to the **gestures** and one proportional to the **rows**, on the one edit where a
/// bit vector otherwise looks like the obvious answer: it is `O(1)` to query and `O(1)` to toggle,
/// and this is the operation it cannot do cheaply at all.
///
/// A count and not a timing, which is what §21 asks a gate to be.
pub fn interval_edit_against_a_bitset(len: usize, removed: Range<usize>) -> (usize, usize) {
    let mut sel = Selection::new();
    sel.insert(0, len / 2);
    let before = sel.span_count();
    let splice = Splice {
        removed: removed.clone(),
        inserted: 0,
    };
    let _ = reconcile_splice(&mut sel, &splice, Policy::Drop);
    let touched = before.max(sel.span_count());
    // The bit vector's half: every bit at or after the interval moves, because a bit's position is
    // its index. There is no cheaper spelling — a bitset has no interval to shift, only bits.
    (touched, len.saturating_sub(removed.start))
}

/// **What each of the three real policies costs**, in nanoseconds, minimum of `rounds`.
///
/// `(clear, drop, stash)`. All three are `O(spans)` and all three are free, which is what makes the
/// choice between them **behavioural rather than a cost** — and `Clear` is here for the permutation
/// it belongs to rather than for the splice it is refused on.
pub fn policy_costs(len: usize, removed: Range<usize>, rounds: u32) -> (u128, u128, u128) {
    use std::time::Instant;

    let half = || {
        let mut sel = Selection::new();
        sel.insert(0, len / 2);
        sel
    };
    let splice = Splice {
        removed,
        inserted: 0,
    };
    let mut clear = u128::MAX;
    let mut drop = u128::MAX;
    let mut stash = u128::MAX;
    for _ in 0..rounds.max(1) {
        let mut sel = half();
        let at = Instant::now();
        reconcile_permutation(&mut sel, len, Policy::Clear, &|_| None);
        clear = clear.min(at.elapsed().as_nanos());

        let mut sel = half();
        let at = Instant::now();
        let _ = reconcile_splice(&mut sel, &splice, Policy::Drop);
        drop = drop.min(at.elapsed().as_nanos());

        let mut sel = half();
        let at = Instant::now();
        let _ = reconcile_splice(&mut sel, &splice, Policy::Stash);
        stash = stash.min(at.elapsed().as_nanos());
    }
    (clear, drop, stash)
}

/// **The corpus the resize case is measured over.**
///
/// Eighty source lines of a plausible document, stated here rather than generated to a target, so
/// that the two row counts are a measurement and not a fixture aimed at a remembered number. See
/// `tests::a_memo_keyed_on_the_revision_alone_recomputes_less_and_is_wrong_on_the_screen`.
pub fn document() -> Vec<&'static str> {
    // **Every line wraps at both widths**, which is what makes the pair a measurement of the
    // *index* rather than of one degenerate side of it: a corpus whose lines fit on one row at 300
    // has no wide index to be stale.
    const JOINED: [&str; 8] = [
        concat!(
            "The engine does not lay anything out: callers bring rectangles, and no layout concept ",
            "may enter through the surface API, which is the door this erodes through. Damage is ",
            "marked at write time and not derived by diffing.",
            " Every number on this page is a measurement over a stated corpus rather than a ",
            "figure carried forward from a prototype, which is why the two row counts are printed ",
            "beside the pair the map remembers instead of being engineered to match it.",
        ),
        concat!(
            "Damage is marked at write time, not derived by diffing; prior art measured a ",
            "full-buffer diff at about a hundred and seventy microseconds on three hundred by ",
            "eighty, which is already over the budget for a whole frame, and that is the draw ",
            "rather than the composite.",
            " Every number on this page is a measurement over a stated corpus rather than a ",
            "figure carried forward from a prototype, which is why the two row counts are printed ",
            "beside the pair the map remembers instead of being engineered to match it.",
        ),
        concat!(
            "A cell holds an interned grapheme-cluster handle and not a char, and no cell, handle ",
            "or style bit is readable from outside the engine at all, which is a decision and not ",
            "an oversight: it holds against the engine's own callers as well.",
            " Every number on this page is a measurement over a stated corpus rather than a ",
            "figure carried forward from a prototype, which is why the two row counts are printed ",
            "beside the pair the map remembers instead of being engineered to match it.",
        ),
        concat!(
            "The runtime has no scene tree and no retained structure: the clip stack is the call ",
            "stack, the id path is the closure tree, and what survives one draw is five flat ",
            "structures rebuilt from the next draw, which is why nothing has to be swept.",
            " Every number on this page is a measurement over a stated corpus rather than a ",
            "figure carried forward from a prototype, which is why the two row counts are printed ",
            "beside the pair the map remembers instead of being engineered to match it.",
        ),
        concat!(
            "A gate is a count, a ratio, an equality or a compile outcome. A timing is a report, ",
            "and a gate only at cliff granularity, with the headroom written next to the number, ",
            "because a gate tuned to the measurement is a flaky test that gets disabled.",
            " Every number on this page is a measurement over a stated corpus rather than a ",
            "figure carried forward from a prototype, which is why the two row counts are printed ",
            "beside the pair the map remembers instead of being engineered to match it.",
        ),
        concat!(
            "One hit entry per collection, because the local pointer is computed from this frame's ",
            "pointer rather than from the previous frame's index — so per-row hover is arithmetic ",
            "with no frame lag and no per-row entry, and that is 229 regions against 389.",
            " Every number on this page is a measurement over a stated corpus rather than a ",
            "figure carried forward from a prototype, which is why the two row counts are printed ",
            "beside the pair the map remembers instead of being engineered to match it.",
        ),
        concat!(
            "Select-all is one span whatever the length, and every other gesture adds at most one, ",
            "so the store is proportional to the gestures a user has made and never to the rows ",
            "there are: sixteen bytes at a million rows, against sixteen and a half megabytes.",
            " Every number on this page is a measurement over a stated corpus rather than a ",
            "figure carried forward from a prototype, which is why the two row counts are printed ",
            "beside the pair the map remembers instead of being engineered to match it.",
        ),
        concat!(
            "A memo's key is every input, and the inputs that are not data are the ones that get ",
            "forgotten: the width, the sort, the expansion set, the repertoire, the theme's own ",
            "revision, the axis policy. Every one of them recomputes less and is wrong on screen.",
            " Every number on this page is a measurement over a stated corpus rather than a ",
            "figure carried forward from a prototype, which is why the two row counts are printed ",
            "beside the pair the map remembers instead of being engineered to match it.",
        ),
    ];
    (0..80).map(|i| JOINED[i % JOINED.len()]).collect()
}

/// The two widths the resize case is played at. §10's own pair.
pub const WIDE: u16 = 300;
/// The narrow half of the resize. See [`WIDE`].
pub const NARROW: u16 = 120;
/// How many display rows the screen shows at once. §10's *69 of 80 rows differ* is over this many.
pub const SCREEN_ROWS: usize = 80;

#[cfg(test)]
mod tests {
    use super::*;

    /// **Criterion 1: one shared shape, with the five uses named against it.**
    ///
    /// The `fields` column is the claim — *a table's order is the same record with three fields
    /// unused* — so every field must be earned by at least one use and every use must name only
    /// fields that exist. A fifth structure arriving is a sixth row here or a field nothing spends.
    #[test]
    fn one_structure_has_five_names_and_every_field_is_earned() {
        assert_eq!(USES.len(), 5, "§10's five, and a sixth is a new structure");
        let all = ["node", "depth", "flags", "h"];
        for use_of in USES {
            assert!(!use_of.fields.is_empty(), "{} spends nothing", use_of.name);
            for field in use_of.fields {
                assert!(
                    all.contains(field),
                    "`{}` names `{field}`, which is not a field of `Entry`",
                    use_of.name
                );
            }
            assert!(
                use_of.fields.contains(&"node"),
                "an index is keyed on `node`"
            );
        }
        for field in all {
            assert!(
                USES.iter().any(|u| u.fields.contains(&field)),
                "`{field}` is a field of `Entry` that no use spends, which is a fifth structure's \
                 field living in the shared one"
            );
        }
        // And the record really is four scalars and no `Id`.
        assert_eq!(size_of::<Entry>(), 16, "`{{ node, depth, flags, h }}`");
    }

    /// **Criterion 2: `Rows { len, rev }`, one `u64`, and the revision moves on every edit.**
    #[test]
    fn the_revision_is_one_u64_and_every_edit_stamps_it() {
        assert_eq!(REVISION_BYTES, 8, "one `u64`");

        let mut order = Order::built(vec![Entry::default(); 10]);
        let first = order.rows();
        assert_eq!(first.len, 10);
        assert!(first.rev.is_known());

        // **A splice is the three things in one call**, so the revision cannot be left behind.
        let splice = order.splice(2..5, [Entry::default()]);
        assert_eq!(splice.shift(), -2);
        let after = order.rows();
        assert_eq!(after.len, 8);
        assert_ne!(after.rev, first.rev, "the splice stamped the revision");

        // **A permutation changes no length**, which is exactly why nothing inside a component can
        // notice it and why the revision is the whole mechanism.
        let to: Vec<usize> = (0..8).map(|i| (i + 3) % 8).collect();
        order.permute(&to);
        let permuted = order.rows();
        assert_eq!(permuted.len, after.len, "a sort changes no length");
        assert_ne!(permuted.rev, after.rev, "and it still stamps the revision");

        // A caller with no order is never told its positions went stale, which is right: it has
        // nothing that can permute them.
        assert_eq!(Rows::of(10).rev, Revision::UNKNOWN);
    }

    /// **Criterion 3: the one-slot request answers once, and the component performs no edit.**
    ///
    /// The `E0502` half is a `compile_fail` pair on [`Asked`] with a positive twin naming
    /// [`Order::splice`] by path — a *proof* rather than a sentence, which is what the criterion
    /// asks for. This half is the slot.
    #[test]
    fn the_request_is_one_slot_and_it_answers_once() {
        let mut asked = Asked::new();
        assert!(asked.standing().is_none());
        asked.ask(Ask::Expand(4));
        asked.ask(Ask::Collapse(9));
        assert_eq!(
            asked.standing(),
            Some(Ask::Collapse(9)),
            "one slot: the last ask of a frame wins"
        );
        assert_eq!(asked.drain(), Some(Ask::Collapse(9)));
        assert!(
            asked.drain().is_none(),
            "a request nobody drains lives exactly one frame, and a drained one is gone"
        );
        assert_eq!(ASKED_BYTES, 16, "a discriminant and a `u64`, and no `Rect`");
    }

    /// **Criterion 4: the four policies, their settled defaults, and the one that is refused.**
    #[test]
    fn the_four_policies_have_settled_defaults_and_clear_is_refused_for_a_splice() {
        assert_eq!(Policy::ALL.len(), 4);
        assert_eq!(Policy::default_for(EditKind::Permutation), Policy::Clear);
        assert_eq!(Policy::default_for(EditKind::Splice), Policy::Drop);

        // The admissions table, in full, because a refusal stated for one arm and forgotten for
        // another is the shape this file exists to avoid.
        for policy in Policy::ALL {
            for edit in [EditKind::Permutation, EditKind::Splice] {
                let want = matches!(
                    (policy, edit),
                    (Policy::Clear | Policy::Remap, EditKind::Permutation)
                        | (Policy::Drop | Policy::Stash, EditKind::Splice)
                );
                assert_eq!(policy.admits(edit), want, "{policy:?} on {edit:?}");
            }
        }
        assert!(
            !Policy::Clear.admits(EditKind::Splice),
            "`Clear` under a splice throws away spans nowhere near the fold for no saving at all"
        );

        // And the refusal fires rather than falling back silently.
        let mut sel = Selection::new();
        sel.insert(0, 10);
        let splice = Splice {
            removed: 2..4,
            inserted: 0,
        };
        let refused = std::panic::catch_unwind(move || {
            let mut sel = Selection::new();
            sel.insert(0, 10);
            reconcile_splice(&mut sel, &splice, Policy::Clear)
        });
        assert!(refused.is_err(), "a refused policy is refused, not ignored");

        // `Stash` keeps what `Drop` throws away, which is the whole difference between them.
        let splice = Splice {
            removed: 2..4,
            inserted: 0,
        };
        let dropped = reconcile_splice(&mut sel, &splice, Policy::Drop);
        assert!(dropped.is_empty());
        assert_eq!(sel.count(), 8, "two rows went with the interval");

        let mut sel = Selection::new();
        sel.insert(0, 10);
        let stashed = reconcile_splice(&mut sel, &splice, Policy::Stash);
        assert_eq!(stashed, vec![Span { start: 2, end: 4 }]);
        assert_eq!(sel.count(), 8);
    }

    /// **Criterion 5: a splice transforms a span list in `O(spans)` and cannot shatter.**
    ///
    /// The structural claim, asserted rather than timed: whatever the interval, a contiguous
    /// selection comes out of a splice as **at most two** spans, and a permutation of the same edit
    /// comes out as one span a row. That is a count, which is what §21 asks a gate to be — the
    /// microseconds are `examples/order_numbers.rs`'s.
    #[test]
    fn a_splice_cannot_shatter_a_span_list_and_a_permutation_always_does() {
        let len = 1_000_000usize;
        let removed = 100_000..449_524; // 349 524 rows, §10's own interval.
        assert_eq!(removed.end - removed.start, 349_524);

        let (spliced, permuted) = splice_vs_permutation(len, removed, 3);
        assert_eq!(
            spliced.after, 1,
            "a half-tree contiguous selection is one span before and one span after"
        );
        assert_eq!(spliced.bytes, 16, "sixteen bytes, at a million rows");
        assert!(
            permuted.after > 100_000,
            "a permutation shatters, and this one is {} spans",
            permuted.after
        );
        assert!(
            permuted.after > spliced.after * 100_000,
            "the ratio is the finding: {} against {}",
            permuted.after,
            spliced.after
        );

        // The structural half, over a sweep rather than one interval: **at most two spans**, and it
        // is two exactly when the interval sits strictly inside the selection.
        for start in [0usize, 1, 10, 40, 49, 50, 60, 99] {
            let mut sel = Selection::new();
            sel.insert(0, 50);
            let splice = Splice {
                removed: start..(start + 5).min(100),
                inserted: 0,
            };
            reconcile_splice(&mut sel, &splice, Policy::Drop);
            assert!(
                sel.span_count() <= 2,
                "a splice at {start} shattered into {} spans",
                sel.span_count()
            );
        }

        // **Register row 13's own form: the interval edit against a bit vector.** A bit's position
        // *is* its index, so a splice moves every bit after the interval; a span list moves the
        // spans that meet it. One against half a million, at a million rows.
        let (spans, bits) = interval_edit_against_a_bitset(len, 100_000..449_524);
        assert_eq!(spans, 1, "a contiguous selection meets the interval once");
        assert_eq!(bits, 900_000, "every bit at or after the interval");
        assert!(
            bits / spans > 100_000,
            "the store is proportional to the gestures and the bit vector to the rows: {bits} \
             against {spans}"
        );

        // **Select-all escapes at `O(1)` and not by special-casing**: `[0, len)` is invariant under
        // any permutation of that length, so the answer is known without touching a row.
        let mut all = Selection::new();
        all.select_all(1_000_000);
        reconcile_permutation(&mut all, 1_000_000, Policy::Remap, &|i| {
            Some((i * 3) % 1_000_001)
        });
        assert_eq!(
            (all.span_count(), all.count()),
            (1, 1_000_000),
            "invariant under any permutation of that length"
        );
    }

    /// **Criterion 6: the three real policies are all `O(spans)`**, which is what makes the choice
    /// behavioural rather than a cost.
    ///
    /// A count and a ratio, not a timing: each of the three touches a number of spans bounded by the
    /// selection's span count, and none of them touches a row. `examples/order_numbers.rs` prints
    /// the nanoseconds beside it.
    #[test]
    fn the_three_real_policies_are_all_proportional_to_the_spans() {
        // The instrument: a selection whose span count is known, reconciled by each policy, with the
        // answer's span count bounded by the input's rather than by the length.
        for spans in [1usize, 10, 1_000] {
            let mut sel = Selection::new();
            for i in 0..spans {
                sel.insert(i * 4, i * 4 + 2);
            }
            assert_eq!(sel.span_count(), spans);

            let splice = Splice {
                removed: 0..2,
                inserted: 0,
            };
            let mut dropped = sel.clone();
            reconcile_splice(&mut dropped, &splice, Policy::Drop);
            assert!(dropped.span_count() <= spans);

            let mut stashed = sel.clone();
            let kept = reconcile_splice(&mut stashed, &splice, Policy::Stash);
            assert!(stashed.span_count() <= spans);
            assert!(kept.len() <= spans);

            let mut cleared = sel.clone();
            reconcile_permutation(&mut cleared, 4_000, Policy::Clear, &|_| None);
            assert_eq!(
                cleared.span_count(),
                0,
                "`Clear` is O(1) and it is why it is cheap"
            );
        }

        // And the timings exist and are all small, which is a report the example prints and this
        // asserts only the relation of: the three are within an order of magnitude of each other,
        // against a permutation that is not.
        let (clear, drop, stash) = policy_costs(1_000_000, 100_000..449_524, 3);
        assert!(clear <= 10_000, "`Clear` is O(1): {clear} ns");
        assert!(drop <= 100_000, "`Drop` is O(spans): {drop} ns");
        assert!(stash <= 100_000, "`Stash` is O(spans): {stash} ns");
    }

    /// **Criterion 7: one position is one position.**
    #[test]
    fn one_position_follows_a_splice_and_is_none_when_its_row_went() {
        let splice = Splice {
            removed: 10..20,
            inserted: 3,
        };
        assert_eq!(reconcile_position(Some(5), &splice), Some(5), "before it");
        assert_eq!(reconcile_position(Some(14), &splice), None, "inside it");
        assert_eq!(reconcile_position(Some(30), &splice), Some(23), "after it");
        assert_eq!(reconcile_position(None, &splice), None);
    }

    /// **Criterion 8: the memo records the input it was built at.**
    ///
    /// And the compound key is spelled out at the call site, which is what the type is for: there is
    /// no `Revision` parameter to reach for and no place to put half a key.
    #[test]
    fn every_memo_spells_its_key_and_records_what_it_was_built_at() {
        let mut memo: Keyed<(Revision, u16), usize> = Keyed::new();
        let rev = Revision::fresh();
        assert_eq!(*memo.get((rev, WIDE), || 1), 1);
        assert_eq!(memo.built_at(), Some(&(rev, WIDE)));
        assert_eq!(memo.recomputes, 1);

        // The half that a bare `Revision` cannot express at all.
        assert_eq!(*memo.get((rev, NARROW), || 2), 2);
        assert_eq!(memo.recomputes, 2);
        assert_eq!(memo.built_at(), Some(&(rev, NARROW)));

        // **No memo in this crate's library half takes a bare `Revision`.** A source scan, for the
        // reason `crate::state`'s press gate is one: an absence has no expression, and the edit that
        // would break the obligation is somebody reaching for `vitui_runtime::Memo` because it is
        // one word shorter.
        let src = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        for entry in std::fs::read_dir(&src).expect("a readable source directory") {
            let path = entry.expect("a directory entry reads").path();
            if !path.extension().is_some_and(|x| x == "rs") {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("a readable source file");
            let library = source
                .split_once("\n#[cfg(test)]\n")
                .map_or(source.as_str(), |(head, _)| head)
                .to_owned();
            for line in library.lines().map(str::trim) {
                assert!(
                    line.starts_with("//") || !line.contains("Memo::new()"),
                    "{} builds a `vitui_runtime::Memo`, whose `get` takes one `Revision` and \
                     cannot express a compound key. `crate::order::Keyed` is the one that can: \
                     {line}",
                    path.display()
                );
            }
        }
    }

    /// **Criterion 9: the 300 → 120 resize, where the wrong key recomputes less and is wrong on the
    /// screen.**
    ///
    /// §10's sharpest arrival and the one that names the detector. A wrap index memoised on the
    /// **revision alone** hits after a resize, so it draws the document at the previous width: fewer
    /// display rows than the narrow width needs, and most of the visible window showing the wrong
    /// source line. Every counter approves — it recomputes **1 against 2** — which is why
    /// `recomputes` is useless here and [`Keyed::built_at`] is not.
    ///
    /// # The magnitudes are this corpus's and the structure is §10's
    ///
    /// §10 records **625 rows drawn where 875 are needed; 69 of 80 rows differ**, over C06's
    /// document, which is not in this repository. [`document`] is stated rather than fitted, so what
    /// reproduces is the shape — fewer rows, most of the window wrong, one recomputation against two
    /// — and the numbers are printed rather than engineered to match.
    #[test]
    fn a_memo_keyed_on_the_revision_alone_recomputes_less_and_is_wrong_on_the_screen() {
        let lines = document();
        let source = Order::built(
            (0..lines.len())
                .map(|n| Entry {
                    node: n as u64,
                    ..Entry::default()
                })
                .collect(),
        );
        let rev = source.rows().rev;

        // **The correct key**: the revision *and* the width, spelled out at the call site.
        let mut right: Keyed<(Revision, u16), WrapIndex> = Keyed::new();
        let wide = right
            .get((rev, WIDE), || WrapIndex::built(&lines, WIDE))
            .clone();
        let narrow = right
            .get((rev, NARROW), || WrapIndex::built(&lines, NARROW))
            .clone();
        assert_eq!(right.recomputes, 2, "a resize is a miss, and it must be");
        assert_eq!(narrow.width(), NARROW);

        // **The key that forgot the width.** No data moved, so the revision did not move, so it
        // hits — and the value it hands back was wrapped at three hundred columns.
        let mut wrong: Keyed<Revision, WrapIndex> = Keyed::new();
        let _ = wrong.get(rev, || WrapIndex::built(&lines, WIDE));
        let stale = wrong.get(rev, || WrapIndex::built(&lines, NARROW)).clone();
        assert_eq!(
            wrong.recomputes, 1,
            "**the natural detector points the wrong way**: the wrong key recomputes 1 against 2"
        );
        assert!(
            wrong.recomputes < right.recomputes,
            "and it is *fewer*, which is what makes `recomputes` useless for this question"
        );

        // The rendered surface, which is the second detector that works.
        assert_eq!(stale.rows(), wide.rows(), "it handed back the wide index");
        assert!(
            stale.rows() < narrow.rows(),
            "{} rows drawn where {} are needed",
            stale.rows(),
            narrow.rows()
        );
        let differ = (0..SCREEN_ROWS)
            .filter(|&i| stale.line_at(i) != narrow.line_at(i))
            .count();
        assert!(
            differ > SCREEN_ROWS / 2,
            "{differ} of {SCREEN_ROWS} rows differ, which is a plausible screen and a wrong one"
        );

        // **And the detector that works catches it**, without looking at the screen at all.
        assert_eq!(stale.width(), WIDE);
        assert_ne!(
            stale.width(),
            NARROW,
            "the index records the width it was built at, which is the whole of ADR 0030's first \
             working detector"
        );
    }

    /// **A splice is spliced and not rebuilt**, which is the other half of §10's table.
    ///
    /// A count rather than a timing: the splice touches the interval and the tail, and the rebuild
    /// touches every row. Measured as *entries the caller had to construct*, because that is the
    /// quantity both arms are proportional to and the one a timing is a proxy for.
    #[test]
    fn a_collapse_splices_the_interval_and_a_rebuild_builds_the_forest() {
        let rows = 100_000usize;
        let mut order = Order::built(
            (0..rows)
                .map(|n| Entry {
                    node: n as u64,
                    depth: (n % 8) as u16,
                    ..Entry::default()
                })
                .collect(),
        );
        let before = order.rows();

        // The splice: one call, and the entries it constructs are the ones going *in*.
        let splice = order.splice(1_000..40_000, []);
        assert_eq!(splice.removed.end - splice.removed.start, 39_000);
        assert_eq!(splice.inserted, 0, "a collapse puts nothing back");
        assert_eq!(order.len(), rows - 39_000);
        assert_ne!(order.rows().rev, before.rev);

        // The rebuild, for the comparison: every surviving row constructed again.
        let rebuilt = Order::built(order.entries().to_vec());
        assert_eq!(rebuilt.len(), order.len());
        assert_ne!(
            rebuilt.rows().rev,
            order.rows().rev,
            "a rebuild is a different order even when it holds the same rows, which is why it is \
             the expensive answer and not merely the slower one"
        );
    }
}
