//! **The forest: a million nodes at depth 59 999, a fold over 349 524 rows, and §7's partition at
//! four widths.**
//!
//! Components ticket 16 and production ticket 07. Spec §7, §17 (O5), §21. This is the screen
//! `tree`'s **three** scenes are scenes *of*, and it stands beside [`crate::listing`] one file over
//! for the same reason that file stands beside [`crate::dense`]: **the scene list is what makes the
//! defect catchable, not the gate.**
//!
//! | scene | what it decides |
//! |---|---|
//! | a million-node forest at depth 59 999 | the flatten index; and the unclamped indent, which asks for 400x the cells and measures **faster** |
//! | a fold and an unfold over 349 524 rows | splice against permutation: **1 run** against a shattered list, and the round trip that comes back wrong |
//! | the partition at 300, 40, 22 and 21 columns | that it is exact at every width; that the **clamp binding** and not the truncation makes the flag visible; that a chevron is *pressed* where its own rectangle draws it |
//!
//! # The width is the second dimension of §7's own statement about the scene list
//!
//! §7's sentence about the unclamped indent ends *on a shallow tree the same flag is invisible: a
//! statement about the scene list, not about the gate.* That reads as a fact about the
//! **depth**, and the scene above it fixes the width at [`W`] and varies only the depth. Production
//! 07's scene fixes the depth at [`SHALLOW`] — where scene 8 says the flag is invisible — and
//! varies the **width**, and the flag reappears at twenty-two columns.
//!
//! So it is a fact about the **pair**, and the two halves are not interchangeable: at [`DEEP`] the
//! arms are separated by *cells asked for* and by nothing else; at [`BINDS_BOTH`] they are
//! separated by the **picture** and by nothing else, because [`WRITES`] is `w * H` on both arms at
//! every one of the four widths. **The narrow axis has no counter at all.**
//!
//! # Three readings, and the third changes no cell
//!
//! [`narrow_screen`] against [`narrow_reference`] is the picture; [`MISDRAWN`], [`WRITES`] and
//! [`VERBS`] are what the counters say about it; and [`pressed`] is the **chevron's hit column**,
//! which is [`crate::collect::Chevron`]'s two spellings and is a reading no equality can reach. A
//! build that draws the chevron in the right column and presses it in the wrong one is a correct
//! screen a pointer cannot use, and it is invisible at three of the four widths.
//!
//! # The trap is the indent, and it is a statement about the scene list
//!
//! > **An unclamped indent makes a row's cost proportional to its depth** — 3 812 970 cells asked
//! > for against 23 030, **165x** — and it measures **18% faster**, because the clip eats them and
//! > the label rectangle collapses. On a shallow tree the same flag is invisible: a statement about
//! > the scene list, not about the gate.
//!
//! Everything in that sentence except the magnitudes reproduces here, and the magnitudes are this
//! screen's rather than the prototype's — see [`the_numbers`]. What matters is the **direction**:
//! the defective arm makes fewer verbs, writes exactly as many cells, touches exactly the same
//! cells, declares the same regions and the same stops, and runs **faster**. Eight of §20's nine
//! counters are on its side, and the ninth is [`crate::counters::Reading::Unreachable`].
//!
//! The one quantity that separates the arms is **cells asked for**, and the scene is where it can be
//! asked at all: at [`SHALLOW`] the same flag is invisible, because at depth ten the indent fits.
//!
//! # The instrument saturates before the defect does, and that is a finding
//!
//! [`crate::counters::Tally::asked`] folds `vitui_runtime::layout::text::width`, which is
//! `vitui_engine::width_of`, which returns a **`u16` and saturates**. An indent of `2 x 59 999` is
//! **119 998** columns, so the tally reports **65 535** — it under-reports the ask by 45.4% and
//! still reports it as 218x the correct arm's.
//!
//! The prototype hit the same `u16` from the other side and got a different wrong answer: its
//! `indent()` computed `depth * 2` in a `u32` and cast, so **119 998 truncated to 54 462**. Neither
//! is a defect in the type — a column count is a `u16` because a terminal is — and both are why the
//! ask is carried here as the caller's own `u64` beside the tally's, with
//! `tests::the_tally_saturates_where_the_caller_does_not` asserting the gap rather than choosing a
//! side.
//!
//! # What the two scenes are red for, and it is one reason
//!
//! `tree` is not declared. [`standing`] is a [`Verdict`] over one subject, [`subjects_declared`]
//! opens the file the freeze homes it in, and [`owed_message`] is the sentence that separates
//! *waiting for its subject* from *the code is wrong* — components ticket 09's criterion 7,
//! inherited whole through components ticket 11. Inverted by **components 17**.
//!
//! Unlike ticket 11's five, both of these are waiting for the same thing, so there is one pin and
//! not two. That is not a simplification of ticket 11's split: the split existed because scene 6's
//! failing set is *the defect itself*, and neither of these two has a defect standing in shipped
//! code. What the unclamped indent is, is a **negative case** — a spelling stood up here so the
//! instrument can be watched catching it — and a negative case is not a red gate.
//!
//! # Three instruments, and none of them is a timing
//!
//! 1. **The frame**, through [`crate::ink::Ink`], so a [`Tally`] and a [`Direct`] measure the same
//!    drawing path rather than a copy of it. [`frame`] returns a [`Shape`]; [`frame_cost`] returns a
//!    [`Duration`] and is a **report**.
//! 2. **The index**, [`Flat`], spliced and rebuilt, with the two compared as an equality.
//! 3. **The selection**, [`Runs`], transformed as an interval and as a permutation, with the
//!    prototype's own straddling-run defect stood up beside the correct splice.
//!
//! [`Direct`]: crate::ink::Direct
//! [`Tally`]: crate::counters::Tally

use std::time::{Duration, Instant};

use vitui_runtime::ctx::Driver;
use vitui_runtime::theme::Glyph;
use vitui_runtime::{
    Button, Buttons, Ctx, Density, Id, Interest, Mods, Mouse, MouseKind, Rect, Role, Scrollable,
};

use crate::collect::{
    Chevron, TreeOpts, TreeState, defective as coll_defective, has_children, tree_into,
};
use crate::counters::Tally;
use crate::frame::Face;
use crate::ink::{Direct, Ink};
use crate::obligations::Verdict;
use crate::order::{Ask, Entry, Order};
use crate::runner::{Canvas, Pen};

// ── the screen ───────────────────────────────────────────────────────────────────────────────────

/// The viewport's width. **Three hundred**, which is §7's own screen and §6's own headline.
pub const W: u16 = 300;
/// The viewport's height. **Eighty**, likewise.
pub const H: u16 = 80;

/// **The depth §21's scene names.** Fifty-nine thousand nine hundred and ninety-nine.
///
/// It fits a `u16` with 5 536 to spare, and that is not slack: the prototype's own note is that the
/// chain forest is built at 60 000 *so the limit is exercised and not crossed*. A tree that deep
/// cannot be navigated by any human, which is exactly why nobody would build the screen that makes
/// the indent visible unless a list said to.
pub const DEEP: u16 = 59_999;

/// The depth the same flag is invisible at. **Ten**, where an indent of twenty fits in three
/// hundred columns with two hundred and eighty to spare.
pub const SHALLOW: u16 = 10;

/// The widest indent [`Indent::Unclamped`] can ask for at [`DEEP`]. **119 998**, and it is larger
/// than a `u16`.
pub const MAX_INDENT: usize = 2 * DEEP as usize;

/// **What a row says.** Twenty-one columns, which fits every column of the twelve-column control.
pub const LABEL: &str = "vnode_ingest_pipeline";

/// How many nodes the forest holds. **One million.**
pub const NODES: u64 = 1_000_000;

/// **How many rows the fold removes. §21's own number.**
///
/// `4 x 87 381`, and 87 381 is `(4^9 - 1) / 3` — the four complete four-ary subtrees under one
/// node. [`Forest::folded`] is built to make it exact rather than approximated, because every other
/// figure of scene 9 is arithmetic over it: 500 000 selected minus 349 524 folded away is
/// [`KEPT`], and the round trip the prototype's defect returns is [`ROUND_TRIP_WRONG`].
pub const FOLD_ROWS: u64 = 349_524;

/// The row the fold is performed at. **One**, and not zero.
///
/// At row 0 a contiguous selection anchored at the top does not straddle the insertion point, and
/// **the defect this scene exists to catch is invisible**: the prototype's `Selection::splice`
/// shifted every run whose `start >= at` and forgot to split a run straddling `at`, which a run
/// starting at 0 never asks it to do. One filler row before the folded node is the whole difference
/// between a round trip that comes back exact and one that comes back [`ROUND_TRIP_WRONG`].
pub const FOLD_AT: usize = 1;

/// **How many rows are selected before the fold.** Half the tree, contiguous, from row 0.
pub const SELECTED: u64 = 500_000;

/// **What survives the fold under `Drop`. 150 476**, and it is arithmetic: `500 000 - 349 524`.
pub const KEPT: u64 = SELECTED - FOLD_ROWS;

/// **What the prototype's defective splice brings back from the round trip. 349 526.**
///
/// Not a near miss and not a crash: the screen is perfectly correct, the run count is 1 before and
/// 1 after, and the timing does not move. `[0, 150 476)` is left unshifted because its `start` is
/// below the insertion point, the parked run `[2, 349 526)` is restored on top of it, and the union
/// is `[0, 349 526)`. **A number that is not 500 000 and looks like a number.**
pub const ROUND_TRIP_WRONG: u64 = 349_526;

/// **How many runs the same selection breaks into under a sort. 249 940 against one.**
///
/// §21 remembers **297 180**, and that figure is a property of *that* forest's key shuffle rather
/// than of the mechanism — which is why this one is a constant with a seed behind it
/// ([`Forest::key_of`]'s shuffle has a seed) instead of a number a report prints. A contiguous half of a
/// million rows through a uniform permutation is expected to break into about `n·p·(1−p)` maximal
/// runs, which is 250 000; this shuffle gives 249 940.
///
/// **What is asserted is the side of the cliff**, and the constant exists so the count has one home
/// rather than three.
pub const PERMUTED_RUNS: usize = 249_940;

/// **How many `Run`s a half-tree fold parks under `Policy::Stash`. One — sixteen bytes.**
///
/// > The parked runs are bounded by runs, not rows.
pub const PARKED_RUNS: usize = 1;

/// How wide one column of the twelve-column control is. `300 / 12`.
pub const COL: u16 = W / 12;

/// **The remembered verb counts §6's headline states**, in `(as a list, as a tree, as a
/// twelve-column table)` order.
///
/// A prototype screen this ticket does not own — two tree panels of 111 rows between them, plus
/// C02's menu bar, toolbar and status bar, at 300x80. `examples/tree_numbers.rs` prints this column
/// beside the measured one rather than engineering the difference away, which is components ticket
/// 06's arrangement and its reason.
///
/// **The chrome is a constant and it can be subtracted.** `640 - 418` is 222, which is
/// `2 x 111` — §7's *the `+` costs two verbs a row, 222 verbs over 111 rows* — so the remembered
/// numbers decompose as `111 x 2 + 196` and `111 x 4 + 196`. The 196 is the screen's furniture, and
/// **the per-row figures reproduce here exactly**: see [`VERBS_A_ROW`].
pub const REMEMBERED_VERBS: (u64, u64, u64) = (418, 640, 2_497);

/// **Verbs a row, by drawing**, and this is the part of [`REMEMBERED_VERBS`] that reproduces.
///
/// `(list, tree, twelve-column table)`. A list is a label and its trailing pad; a tree is those two
/// plus §7's own pair — *the indent run and the chevron cell*; a table is a label and a pad per
/// declared column.
///
/// # A verb is a structural currency here and not a timing one
///
/// §6 prices the three drawings in time as well as in verbs — 48.79 µs for a tree against 165.6 for
/// a twelve-column table, **3.4x for the same 23 030 cells**. On this runtime the same rectangle
/// drawn with **160, 320 and 1 920 verbs costs the same frame**, within 1%
/// (`examples/tree_numbers.rs`). That is R15's rule arriving from the other side: a gate written on
/// the timing here would report a tree and a table as one construction, which is why the gate is
/// the count.
pub const VERBS_A_ROW: (u64, u64, u64) = (2, 4, 24);

/// The three volumes the frame is asserted flat across. §7: *flat at 1k / 100k / 1M nodes*.
pub const VOLUMES: [u64; 3] = [1_000, 100_000, 1_000_000];

/// **Verbs the tree costs at [`DEEP`], against [`VERBS_A_ROW`]'s four a row at [`SHALLOW`].**
///
/// Three a row, not four. §7 says the frame is *unchanged at depth 59 999* and that is true of every
/// counter here except this one: a clamped indent of `w - 2` leaves the label one column, the label
/// fills it, and **the trailing pad has nothing to pad**. The prototype's own table says the same
/// thing in the same column — 634 at the deepest page against 640 at 1M all-open — so this is the
/// remembered shape rather than a departure from it.
pub const DEEP_VERBS: u64 = 3 * H as u64;

/// **Interactive regions a correct frame declares.** One scrollable plus one target a drawn row.
///
/// §7's frame states **59**, over the prototype's two tree panels with per-row hits off on one of
/// them; this screen is one tree filling one terminal, so the number is this screen's and the
/// remembered one is printed beside it.
pub const REGIONS: usize = 1 + H as usize;

/// **Tab stops a correct frame declares. One**, for [`crate::listing::STOPS`]'s reason: a
/// virtualised collection is one tab stop, and a tree is a collection plus a flatten index.
pub const STOPS: usize = 1;

// ── the forest, and the flatten index over it ────────────────────────────────────────────────────

/// **The caller's forest, as a depth array in pre-order.**
///
/// One `u16` a node and one `u32` key a node, and the subtree of node `i` is the contiguous run of
/// following nodes whose depth is greater than `depth[i]`. That is §7's own structure seen from the
/// data side: *`depth` is there so a collapse can find the interval it removes without touching the
/// forest*, and a pre-order depth array is the smallest thing that makes the sentence true on both
/// sides of it.
///
/// **The `key` is what a sort sorts by**, and it is shuffled on purpose. A permutation reconciliation
/// asks the caller where every selected row went; if the keys were the pre-order positions the
/// answer would be the identity and the arm would report *a permutation costs nothing*, which is the
/// vacuity accident [`crate::obligations::Verdict::of`] refuses one file over.
#[derive(Clone, Debug)]
pub struct Forest {
    depth: Vec<u16>,
    key: Vec<u32>,
}

impl Forest {
    /// **A chain of `chain + 1` levels, with the rest of `nodes` as roots beside it.**
    ///
    /// The forest scene's own shape: every row of the deepest page sits `chain`-odd levels down, and
    /// the frame is asked whether it is the same frame.
    pub fn chained(nodes: u64, chain: u16) -> Forest {
        let n = usize::try_from(nodes).expect("a forest that fits an index");
        let mut depth = Vec::with_capacity(n);
        for level in 0..=u32::from(chain) {
            if depth.len() == n {
                break;
            }
            depth.push(u16::try_from(level).expect("chain fits a u16"));
        }
        while depth.len() < n {
            depth.push(0);
        }
        Forest::with_shuffled_keys(depth)
    }

    /// **The fold scene's forest**, built so that one node's subtree is exactly [`FOLD_ROWS`].
    ///
    /// Row 0 is a leaf, row [`FOLD_AT`] is the folded node, and under it sit four subtrees of
    /// 87 381 nodes each — `4 x (1 + 87 380)`, which is `4 x (4^9 - 1) / 3` and is 349 524 exactly.
    /// Everything after is a root at depth 0, so the fold removes a contiguous interval and nothing
    /// else moves.
    ///
    /// The leaf at row 0 is [`FOLD_AT`]'s whole reason and is not padding: see that constant.
    pub fn folded() -> Forest {
        let n = usize::try_from(NODES).expect("a million fits an index");
        let mut depth = Vec::with_capacity(n);
        depth.push(0); // the filler leaf
        depth.push(0); // the folded node
        for _ in 0..4 {
            depth.extend(std::iter::once(1).chain(std::iter::repeat_n(2, 87_380)));
        }
        assert_eq!(
            depth.len() as u64,
            2 + FOLD_ROWS,
            "the folded subtree is not §21's own number"
        );
        while depth.len() < n {
            depth.push(0);
        }
        Forest::with_shuffled_keys(depth)
    }

    /// Attach a deterministic shuffle of `0..len` as the sort key.
    ///
    /// A fixed xorshift and a Fisher-Yates, so the number a permutation costs is reproducible on any
    /// machine and is still a shuffle rather than a pattern the run-coalescer can exploit.
    fn with_shuffled_keys(depth: Vec<u16>) -> Forest {
        let n = depth.len();
        let mut key: Vec<u32> = (0..u32::try_from(n).expect("fits")).collect();
        let mut state = 0x2545_F491_4F6C_DD1Du64;
        for i in (1..n).rev() {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let j = (state % (i as u64 + 1)) as usize;
            key.swap(i, j);
        }
        Forest { depth, key }
    }

    /// How many nodes it holds.
    pub fn len(&self) -> usize {
        self.depth.len()
    }

    /// Whether it holds none. A legitimate state and not one any scene here plays.
    pub fn is_empty(&self) -> bool {
        self.depth.is_empty()
    }

    /// The depth of node `i`.
    ///
    /// # Panics
    ///
    /// Panics past the end, which is a bug in the caller and not a state.
    pub fn depth_of(&self, i: usize) -> u16 {
        self.depth[i]
    }

    /// The sort key of node `i`. See [`Forest`].
    ///
    /// # Panics
    ///
    /// Panics past the end.
    pub fn key_of(&self, i: usize) -> u32 {
        self.key[i]
    }

    /// **How many nodes hang under node `i`**, read from the forest by walking it.
    ///
    /// The expensive half of §7's sentence, kept here so [`Flat::span`] has something to be compared
    /// against: *from the index that is a contiguous scan, from the data it is one random access per
    /// removed row*.
    pub fn descendants(&self, i: usize) -> usize {
        let mine = self.depth[i];
        let mut j = i + 1;
        while j < self.depth.len() && self.depth[j] > mine {
            j += 1;
        }
        j - i - 1
    }
}

/// One row of the flatten index.
///
/// **Deliberately not spec §7's record.** `Row { node: u32, depth: u16, flags: u8, h: u8 }` and the
/// gate that asserts it is eight bytes are components ticket 17's, and a scenes ticket that shipped
/// them would leave that ticket asserting something already here. What this holds is the two fields
/// a *scene* needs — which node, and how deep — and nothing else.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Slot {
    /// The node, in the caller's own numbering.
    pub node: u32,
    /// How deep it sits. **The field the fold reads**, and the field the indent must not.
    pub depth: u16,
}

/// **The flatten index: the expanded forest in display order.**
///
/// A stand-in, for [`Slot`]'s reason. What it is here for is that the two equalities §21's row 14
/// names — *a fold/unfold round trip, and `splice == rebuild`* — are questions about an index and
/// cannot be asked of a component that does not exist.
#[derive(Clone, Debug)]
pub struct Flat {
    rows: Vec<Slot>,
    /// The nodes whose subtrees are not in [`Flat::rows`], in the caller's numbering.
    collapsed: Vec<u32>,
}

/// **What a fold or an unfold did to the index**, in the three numbers the component knows exactly.
///
/// > A subtree is contiguous in pre-order display coordinates, so the edit is
/// > `Splice { at, removed, inserted }` and the component *performed* it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Splice {
    /// Where in the index.
    pub at: usize,
    /// How many rows went away.
    pub removed: usize,
    /// How many arrived.
    pub inserted: usize,
}

impl Flat {
    /// **The whole forest, expanded.** One depth-first pass, proportional to the rows it produces.
    pub fn expanded(forest: &Forest) -> Flat {
        Flat {
            rows: (0..forest.len())
                .map(|i| Slot {
                    node: u32::try_from(i).expect("fits"),
                    depth: forest.depth_of(i),
                })
                .collect(),
            collapsed: Vec::new(),
        }
    }

    /// **The index a fresh walk of the forest produces**, given the collapsed set.
    ///
    /// The right-hand side of `splice == rebuild`. Proportional to what *remains*, which is §7's own
    /// reason there is no rebuild path in the shipped component: at 52% folded the rebuild is
    /// 40 393 µs against the splice's 257.
    pub fn rebuilt(forest: &Forest, collapsed: &[u32]) -> Flat {
        let mut rows = Vec::new();
        let mut i = 0usize;
        while i < forest.len() {
            let node = u32::try_from(i).expect("fits");
            rows.push(Slot {
                node,
                depth: forest.depth_of(i),
            });
            if collapsed.contains(&node) {
                i += 1 + forest.descendants(i);
            } else {
                i += 1;
            }
        }
        let mut collapsed = collapsed.to_vec();
        collapsed.sort_unstable();
        Flat { rows, collapsed }
    }

    /// How many rows the index holds.
    pub fn len(&self) -> usize {
        self.rows.len()
    }

    /// Whether it holds none.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// The row at display position `i`.
    pub fn row(&self, i: usize) -> Slot {
        self.rows[i]
    }

    /// The collapsed set, sorted.
    pub fn collapsed(&self) -> &[u32] {
        &self.collapsed
    }

    /// **How many rows hang under display position `i`, read from the index.**
    ///
    /// §7's own sentence, as the cheap half: a contiguous scan over 8-byte-class records, with no
    /// access to the forest at all.
    pub fn span(&self, i: usize) -> usize {
        let mine = self.rows[i].depth;
        let mut j = i + 1;
        while j < self.rows.len() && self.rows[j].depth > mine {
            j += 1;
        }
        j - i - 1
    }

    /// **Fold the subtree at display position `i`, by splice.**
    ///
    /// No threshold and no rebuild path. The crossover is at about **99.7% of the index removed** —
    /// in a tree, only at the root, where the rebuild wins by doing nothing — and a branch that has
    /// to stay correct is not worth 340 µs once. That sentence is here rather than in a `match`
    /// because §7 asks for it that way.
    pub fn fold(&mut self, i: usize) -> Splice {
        let removed = self.span(i);
        self.rows.drain(i + 1..i + 1 + removed);
        let node = self.rows[i].node;
        if let Err(at) = self.collapsed.binary_search(&node) {
            self.collapsed.insert(at, node);
        }
        Splice {
            at: i + 1,
            removed,
            inserted: 0,
        }
    }

    /// **Unfold the subtree at display position `i`, by splice.**
    ///
    /// Proportional to what it inserts, which beats a rebuild by `total / inserted` — 1.15x at the
    /// root and **700x at a 340-row subtree**.
    ///
    /// The subtree is regenerated from the forest, which is the one place this stand-in reaches back
    /// into the data: nothing was kept when it was folded, because keeping it is what a `Vec<u32>`
    /// order does and §7 says the index is not one.
    pub fn unfold(&mut self, forest: &Forest, i: usize) -> Splice {
        let node = self.rows[i].node;
        let root = node as usize;
        let n = forest.descendants(root);
        let inserted: Vec<Slot> = (root + 1..=root + n)
            .map(|j| Slot {
                node: u32::try_from(j).expect("fits"),
                depth: forest.depth_of(j),
            })
            .collect();
        let count = inserted.len();
        self.rows.splice(i + 1..i + 1, inserted);
        if let Ok(at) = self.collapsed.binary_search(&node) {
            self.collapsed.remove(at);
        }
        Splice {
            at: i + 1,
            removed: 0,
            inserted: count,
        }
    }

    /// The nodes the index holds, in display order. The permutation arm's input.
    pub fn nodes(&self) -> impl Iterator<Item = u32> + '_ {
        self.rows.iter().map(|s| s.node)
    }
}

// ── the selection, as an interval and as a permutation ───────────────────────────────────────────

/// One half-open run of selected rows.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Run {
    /// The first selected row.
    pub start: u64,
    /// One past the last.
    pub end: u64,
}

impl Run {
    /// How many rows it holds.
    pub fn rows(self) -> u64 {
        self.end - self.start
    }
}

/// **What a fold does to the runs that were nowhere near it.**
///
/// All three real policies are O(runs) and all three are free — 0.00 / 0.08 / 0.04 µs — so the
/// choice is behavioural rather than forced, which is the opposite of a sort's, where the cost
/// decides.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Policy {
    /// **The default.** The selected rows inside the fold are forgotten.
    Drop,
    /// **The caller's opt-in.** They are parked, bounded by runs and not by rows, and restored
    /// exactly on the unfold.
    Stash,
    /// **Refused for a splice**, and stood up here so the refusal is a measurement: it throws away
    /// the runs that were nowhere near the fold for no saving at all.
    Clear,
}

impl Policy {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Policy::Drop => "drop",
            Policy::Stash => "stash",
            Policy::Clear => "clear",
        }
    }
}

/// **Whether the splice splits a run that straddles the edit.**
///
/// The prototype shipped the arm that does not, and *nothing in the report moved*: the run count was
/// 1 before and 1 after, the timing was 0.04 µs, and the frame was identical. What caught it was an
/// equality asserting the round trip restores the selection **exactly**.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Straddle {
    /// **The rule.** A run containing the edit point is split.
    Split,
    /// **The defect.** Every run whose `start >= at` is shifted and the rest is left alone.
    Forget,
}

/// **A sorted, disjoint run list.** The selection store, from the side a fold touches.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Runs(Vec<Run>);

impl Runs {
    /// One contiguous run, `[start, end)`.
    pub fn of(start: u64, end: u64) -> Runs {
        Runs(if start < end {
            vec![Run { start, end }]
        } else {
            Vec::new()
        })
    }

    /// The runs, in order.
    pub fn runs(&self) -> &[Run] {
        &self.0
    }

    /// How many runs. **The number a permutation shatters and an interval does not.**
    pub fn count(&self) -> usize {
        self.0.len()
    }

    /// How many rows are selected.
    pub fn rows(&self) -> u64 {
        self.0.iter().map(|r| r.rows()).sum()
    }

    /// **Transform the selection under an interval edit, in O(runs).**
    ///
    /// Every run is wholly before the interval, wholly after it (one addition), or clipped by
    /// it — which splits at most one run and may merge two. Nothing here is proportional to a row.
    ///
    /// Returns what [`Policy::Stash`] parked, which is empty for the other two.
    pub fn splice(&mut self, edit: Splice, policy: Policy, straddle: Straddle) -> Runs {
        if policy == Policy::Clear {
            self.0.clear();
            return Runs::default();
        }
        let at = edit.at as u64;
        if edit.removed > 0 {
            let removed = edit.removed as u64;
            let end = at + removed;
            let mut parked = Vec::new();
            let mut next = Vec::new();
            for run in &self.0 {
                let lead = Run {
                    start: run.start,
                    end: run.end.min(at),
                };
                if lead.start < lead.end {
                    next.push(lead);
                }
                let inside = Run {
                    start: run.start.max(at),
                    end: run.end.min(end),
                };
                if inside.start < inside.end && policy == Policy::Stash {
                    parked.push(Run {
                        start: inside.start,
                        end: inside.end,
                    });
                }
                let tail = Run {
                    start: run.start.max(end),
                    end: run.end,
                };
                if tail.start < tail.end {
                    next.push(Run {
                        start: tail.start - removed,
                        end: tail.end - removed,
                    });
                }
            }
            self.0 = next;
            coalesce(&mut self.0);
            return Runs(parked);
        }
        if edit.inserted > 0 {
            let inserted = edit.inserted as u64;
            let mut next = Vec::new();
            for run in &self.0 {
                if run.start >= at {
                    next.push(Run {
                        start: run.start + inserted,
                        end: run.end + inserted,
                    });
                } else if run.end > at && straddle == Straddle::Split {
                    // **The half the prototype forgot.** A run containing the insertion point
                    // becomes two, and the second one moves.
                    next.push(Run {
                        start: run.start,
                        end: at,
                    });
                    next.push(Run {
                        start: at + inserted,
                        end: run.end + inserted,
                    });
                } else {
                    next.push(*run);
                }
            }
            self.0 = next;
            coalesce(&mut self.0);
        }
        Runs::default()
    }

    /// Put a parked selection back. The other half of [`Policy::Stash`]'s round trip.
    pub fn restore(&mut self, parked: &Runs) {
        self.0.extend_from_slice(&parked.0);
        self.0.sort_by_key(|r| r.start);
        coalesce(&mut self.0);
    }

    /// **The permutation answer, and what it costs.**
    ///
    /// The caller replaced the order; only the caller has both, so the component asks it where every
    /// selected row went. A run is contiguous **in the order it was recorded in**, and a permutation
    /// has no obligation to keep it so — so the transform is proportional to the selected **rows**
    /// and the run list shatters.
    ///
    /// Returns `(the new selection, how many bytes the remap materialised)`. The byte figure is
    /// `positions * size_of::<u64>()`, stated rather than estimated.
    pub fn permuted(&self, order: &[u32], inverse: &[u32]) -> (Runs, usize) {
        let mut positions: Vec<u64> = Vec::new();
        for run in &self.0 {
            for row in run.start..run.end {
                let node = order[row as usize];
                let to = inverse[node as usize];
                if to != u32::MAX {
                    positions.push(u64::from(to));
                }
            }
        }
        positions.sort_unstable();
        let bytes = positions.len() * std::mem::size_of::<u64>();
        let mut runs = Vec::new();
        let mut i = 0usize;
        while i < positions.len() {
            let start = positions[i];
            let mut j = i;
            while j + 1 < positions.len() && positions[j + 1] == positions[j] + 1 {
                j += 1;
            }
            runs.push(Run {
                start,
                end: positions[j] + 1,
            });
            i = j + 1;
        }
        (Runs(runs), bytes)
    }
}

/// Merge runs that touch. A selection is a set, and two names for one row is a defect.
fn coalesce(runs: &mut Vec<Run>) {
    let mut out: Vec<Run> = Vec::with_capacity(runs.len());
    for run in runs.iter() {
        match out.last_mut() {
            Some(last) if last.end >= run.start => last.end = last.end.max(run.end),
            _ => out.push(*run),
        }
    }
    *runs = out;
}

/// **What one reconciliation cost**, in the three quantities that decide it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Reconciled {
    /// Runs afterwards. **The number that shatters.**
    pub runs: usize,
    /// Rows still selected.
    pub rows: u64,
    /// Bytes the transform materialised.
    pub bytes: usize,
}

/// **Both arms of scene 9's reconciliation, over one selection and one fold.**
///
/// Returns `(under a splice, under a permutation)`. The permutation arm is a **sort**, which is the
/// gesture §7 contrasts a fold with: *a sort is a permutation and a fold is an interval*.
pub fn reconciled(forest: &Forest) -> (Reconciled, Reconciled, Duration, Duration) {
    let flat = Flat::expanded(forest);
    let order: Vec<u32> = flat.nodes().collect();

    // The permutation: the caller re-sorts the whole display order by key.
    let mut sorted: Vec<u32> = order.clone();
    sorted.sort_unstable_by_key(|&node| forest.key_of(node as usize));
    let mut inverse = vec![u32::MAX; forest.len()];
    for (to, &node) in sorted.iter().enumerate() {
        inverse[node as usize] = u32::try_from(to).expect("fits");
    }

    let selection = Runs::of(0, SELECTED);
    let started = Instant::now();
    let (permuted, bytes) = selection.permuted(&order, &inverse);
    let permutation_took = started.elapsed();

    let mut spliced = selection.clone();
    let mut folding = flat;
    let edit = folding.fold(FOLD_AT);
    let started = Instant::now();
    let _ = spliced.splice(edit, Policy::Drop, Straddle::Split);
    let splice_took = started.elapsed();

    (
        Reconciled {
            runs: spliced.count(),
            rows: spliced.rows(),
            bytes: 0,
        },
        Reconciled {
            runs: permuted.count(),
            rows: permuted.rows(),
            bytes,
        },
        splice_took,
        permutation_took,
    )
}

/// **The fold/unfold round trip, over both spellings of the splice.**
///
/// Returns how many rows come back selected. Under [`Straddle::Split`] it is [`SELECTED`]; under
/// [`Straddle::Forget`] it is [`ROUND_TRIP_WRONG`], and the screen is perfectly correct either way.
pub fn round_trip(forest: &Forest, straddle: Straddle) -> u64 {
    let mut flat = Flat::expanded(forest);
    let mut selection = Runs::of(0, SELECTED);
    let folded = flat.fold(FOLD_AT);
    let parked = selection.splice(folded, Policy::Stash, straddle);
    assert_eq!(
        parked.count(),
        PARKED_RUNS,
        "a half-tree fold parks one run"
    );
    let unfolded = flat.unfold(forest, FOLD_AT);
    let _ = selection.splice(unfolded, Policy::Stash, straddle);
    selection.restore(&parked);
    selection.rows()
}

// ── the frame ────────────────────────────────────────────────────────────────────────────────────

/// **The component's own [`Indent`], re-exported rather than declared a second time.**
///
/// This module used to carry its own copy, because components ticket 16 had no component to reach
/// for. [`crate::collect::indent_columns`] is the one decision now, and it is the function
/// [`crate::collect::tree`] calls — so the number this screen sums and the number the run is drawn
/// with come out of the same place rather than out of two that agree today.
pub use crate::collect::{Indent, indent_columns};

/// **Which of §6's four drawings of one rectangle.**
///
/// The same screen, the same rectangle and the same cells, drawn three ways, which is what makes
/// verbs legible as a currency rather than as an accident of the content.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Drawn {
    /// A plain list: a label and its trailing pad.
    List,
    /// A tree: those two, plus §7's own pair — the indent run and the chevron cell.
    Tree,
    /// A twelve-column table, as the control. **Components ticket 14 owns the table's own screen**;
    /// this arm is here so the verb comparison has its third column and nothing else.
    Table,
}

impl Drawn {
    /// The word a report prints.
    pub const fn word(self) -> &'static str {
        match self {
            Drawn::List => "a plain list",
            Drawn::Tree => "a tree",
            Drawn::Table => "a twelve-column table",
        }
    }
}

/// **What one frame is asked to be**, so that a scene is a value rather than six arguments.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Plan {
    /// Which of §6's drawings.
    pub drawn: Drawn,
    /// Clamped or not.
    pub indent: Indent,
    /// The depth every drawn row sits at.
    pub depth: u16,
    /// How many rows the content holds.
    pub rows: u64,
    /// Where the window starts.
    pub offset: i32,
}

impl Plan {
    /// The tree, clamped, at `depth`, over a million rows, scrolled to the deepest page.
    pub fn tree_at(depth: u16) -> Plan {
        Plan {
            drawn: Drawn::Tree,
            indent: Indent::Clamped,
            depth,
            rows: NODES,
            offset: 0,
        }
    }

    /// The same plan, drawn another way.
    pub fn as_drawn(self, drawn: Drawn) -> Plan {
        Plan { drawn, ..self }
    }

    /// The same plan, with the indent unclamped.
    pub fn unclamped(self) -> Plan {
        Plan {
            indent: Indent::Unclamped,
            ..self
        }
    }

    /// The same plan, over a different volume.
    pub fn over(self, rows: u64) -> Plan {
        Plan { rows, ..self }
    }
}

/// **What one frame of the forest turned out to be**, returned rather than printed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Shape {
    /// Columns the engine reported written.
    pub writes: u64,
    /// Distinct cells touched.
    pub distinct: u64,
    /// Drawing calls made.
    pub verbs: u64,
    /// **Cells the caller asked for, in the caller's own `u64`.**
    ///
    /// The one quantity that separates the two indents, and the reason it is not
    /// [`crate::counters::Tally::asked`] is in this module's header: that one saturates at 65 535 a
    /// verb.
    pub asked: u64,
    /// **[`crate::counters::Tally::asked`] over the same frame**, kept beside [`Shape::asked`] so
    /// the saturation is a number rather than a footnote.
    pub tallied_ask: u64,
    /// Entries in the runtime's hit index.
    pub regions: usize,
    /// Entries in the focus ring.
    pub stops: usize,
    /// Rows the body iterated. The mechanism, where the counters are the symptom.
    pub iterated: u64,
}

/// **The flatten index this screen's rows are read out of, built once and not in a frame.**
///
/// Every row sits at [`Plan::depth`], which is what makes the frame at depth 10 and the frame at
/// depth 59 999 the *same* frame with one number changed. Building it is proportional to the data
/// by construction — that is what materialising is — and §10 prices doing it in a frame at 211 frame
/// budgets, so it happens here, once, outside every measurement bracket.
pub fn index_for(plan: Plan) -> Order {
    let rows = usize::try_from(plan.rows).unwrap_or(usize::MAX);
    Order::built(
        (0..rows)
            .map(|i| Entry::of(u32::try_from(i).unwrap_or(u32::MAX)).at_depth(plan.depth))
            .collect(),
    )
}

/// **A flatten index whose rows alternate between [`SHALLOW`] and one deeper, so half of them have
/// a child.** Production 07's, and both of that ticket's scenes take it.
///
/// # Why [`index_for`] would not do, and it is a fact about scenes 8 and 9
///
/// `index_for` puts **every** row at one depth. [`crate::collect::has_children`] is *the next row is
/// deeper or it is not*, so under it no row has a child, every row is a leaf, and §7's second verb
/// — the chevron — is a **space** on every one of the eighty rows scenes 8 and 9 draw. That is
/// correct for what those two scenes decide (the flatten index and a fold, neither of which is about
/// a chevron) and it is the wrong fixture for an axis that is about the row's *partition*: an arm
/// that lost the chevron entirely would be invisible on a screen where the chevron is a space.
///
/// A parent at depth *d* with one child at *d + 1*, repeated, is a real forest in pre-order display
/// coordinates: `rows / 2` parents, each drawn with [`Glyph::ArrowDown`] because nothing here is
/// folded, and `rows / 2` leaves drawn with a space. Nothing is [`Entry::FOLDED`], so
/// [`Glyph::ArrowRight`] is not on this screen and [`crate::glyphs`] is where that half is joined.
///
/// [`Glyph::ArrowDown`]: vitui_runtime::theme::Glyph::ArrowDown
/// [`Glyph::ArrowRight`]: vitui_runtime::theme::Glyph::ArrowRight
pub fn nested(rows: usize) -> Order {
    Order::built(
        (0..rows)
            .map(|i| {
                let depth = if i % 2 == 0 { SHALLOW } else { SHALLOW + 1 };
                Entry::of(u32::try_from(i).unwrap_or(u32::MAX)).at_depth(depth)
            })
            .collect(),
    )
}

/// **Draw one frame of the forest.** Returns `(rows iterated, cells asked for)`.
///
/// Generic over [`Ink`] so a [`Tally`] and a [`Direct`] measure the same drawing path rather than a
/// copy of it.
///
/// # The tree arm draws through the component and the other two do not
///
/// [`Drawn::Tree`] is [`crate::collect::tree`]; [`Drawn::List`] and [`Drawn::Table`] are the two
/// controls §6's verb comparison needs and are drawn by hand, because a *control* drawn through the
/// thing under test is not a control. That is the split `crate::grid` took one component over.
///
/// # The row loop is the component's now, and that is where the trap is measured
///
/// A tree that iterated its content would be [`crate::listing::Volume::WholeContent`]'s defect, one
/// component over and already caught; `collection` asks [`Ctx::visible_rows`] and this screen
/// inherits it. What the loop is asked instead is whether **a row's own cost** is proportional to a
/// number that came out of the data — and the window is what makes the question answerable at all,
/// because a frame that draws eighty rows either way is a frame two arms can be compared over.
///
/// [`Ctx::visible_rows`]: vitui_runtime::Ctx::visible_rows
pub fn draw_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    plan: Plan,
    index: &Order,
) -> (u64, u64) {
    let view = cx.area();
    let w = view.w;
    let body = cx.theme().paint(Role::Body);
    let mut iterated = 0u64;
    let mut asked = 0u64;

    match plan.drawn {
        Drawn::Tree => {
            let mut st = TreeState::new();
            st.coll.offset = plan.offset;
            let opts = TreeOpts::default();
            // **The indent's half of the ask comes from the function the component called**, and
            // the label's half is measured by the drawer that writes it. The two are joined by the
            // rectangle the component hands over: `label.x` is the indent plus the chevron, which
            // `tests::the_component_places_the_label_where_the_indent_says` asserts rather than
            // assumes.
            let ind = indent_columns(plan.indent, plan.depth, w);
            let mut find = |_: &str, _: std::ops::Range<usize>| None;
            let mut label =
                |ink: &mut I, cx: &mut Ctx<'_, '_>, r: Rect, n: crate::collect::Node, _f: Face| {
                    iterated += 1;
                    // **The row's target, declared by the row and not by the component** (§7's 59
                    // regions). The rectangle is the row's whole width — a tree row is clickable
                    // where its label is not — and `Node::id` is what the component minted for it.
                    let _ = cx.interact(n.id, Rect::new(0, r.y, w, 1), Interest::CLICK);
                    asked += ind as u64;
                    if r.w > 0 {
                        // **The chevron, then the label's own rectangle.** `r.x` is where the
                        // component put it, so this arm reads the placement rather than modelling
                        // it — see `tests::the_component_places_the_label_where_the_indent_says`.
                        asked += 1;
                        asked += label_and_pad(ink, cx, r.x, r.y, r.w, body);
                    }
                };
            // **One value between the two arms**, which is `crate::grid`'s arrangement one
            // component over: the defect is a function in `collect::defective` rather than a
            // branch this screen takes inside a row.
            let _ = match plan.indent {
                Indent::Clamped => {
                    tree_into(ink, cx, view, &mut st, &opts, index, &mut find, &mut label)
                }
                Indent::Unclamped => coll_defective::unclamped_indent(
                    ink, cx, view, &mut st, &opts, index, &mut find, &mut label,
                ),
            };
        }
        Drawn::List | Drawn::Table => {
            let id = Id::named("forest");
            let max = (0, max_offset(plan.rows));
            let offset = (0, plan.offset.clamp(0, max.1));
            let _ = cx.scrollable(
                id,
                view,
                Interest::CLICK.with(Interest::FOCUS),
                Scrollable::between(offset, max),
            );
            cx.scroll_scope(id, view, offset, max, |cx| {
                for y in cx.visible_rows() {
                    iterated += 1;
                    cx.with_key(y as u64, |cx| {
                        let row = cx.id();
                        let _ = cx.interact(row, Rect::new(0, y, w, 1), Interest::CLICK);
                        asked += match plan.drawn {
                            Drawn::List => list_row(ink, cx, y, w, body),
                            Drawn::Table => table_row(ink, cx, y, w, body),
                            Drawn::Tree => unreachable!("the tree arm is drawn through `tree`"),
                        };
                    });
                }
            });
        }
    }
    (iterated, asked)
}

/// A list's row: the label and its trailing pad. Two verbs, and the partition is exact.
fn list_row<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    y: i32,
    w: u16,
    body: vitui_runtime::Paint,
) -> u64 {
    label_and_pad(ink, cx, 0, y, w, body)
}

/// **The label truncated to the room it has, then padded to fill it.** `fit`'s arithmetic without
/// its ellipsis, which is [`crate::text::fit`]'s and components ticket 09's scene rather than this
/// one's.
///
/// Truncating rather than letting the clip do it is what keeps *cells asked for* a statement about
/// the caller: a label written whole into a one-column remainder asks for twenty-one columns and is
/// reported one, which is the same overrun the unclamped indent is — at one five-thousandth of the
/// size, on the arm that is supposed to be correct.
///
/// Returns the columns asked for.
fn label_and_pad<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    x: i32,
    y: i32,
    room: u16,
    body: vitui_runtime::Paint,
) -> u64 {
    if room == 0 {
        return 0;
    }
    let head = vitui_runtime::layout::text::truncate(LABEL, room);
    let written = ink.text(cx, x, y, head, body);
    let pad = room.saturating_sub(written);
    if pad > 0 {
        let _ = ink.run(cx, x + i32::from(written), y, " ", pad, body);
    }
    u64::from(room)
}

/// A twelve-column table's row: a label and a pad a column.
fn table_row<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    y: i32,
    w: u16,
    body: vitui_runtime::Paint,
) -> u64 {
    debug_assert_eq!(
        w,
        12 * COL,
        "twelve columns are a partition of the rectangle"
    );
    (0..12u16)
        .map(|c| label_and_pad(ink, cx, i32::from(c * COL), y, COL, body))
        .sum()
}

/// The largest offset `rows` rows admit in an [`H`]-row viewport.
pub fn max_offset(rows: u64) -> i32 {
    i32::try_from(rows.saturating_sub(u64::from(H))).unwrap_or(i32::MAX)
}

/// **One frame as a shape.** The instrument every count on this page is asserted through.
///
/// One warm-up frame and one measured one: the five frame structures take their allocation on the
/// first frame that needs one and keep it, and a cold frame is not a frame.
pub fn frame(plan: Plan) -> Shape {
    let index = index_for(plan);
    let mut driver = crate::runner::driver_at(W, H, Density::default());
    let mut warm = Tally::new();
    driver.frame(|cx| {
        let _ = draw_into(&mut warm, cx, plan, &index);
    });

    let mut tally = Tally::new();
    let mut iterated = 0;
    let mut asked = 0;
    driver.frame(|cx| {
        let (rows, cells) = draw_into(&mut tally, cx, plan, &index);
        iterated = rows;
        asked = cells;
    });
    let inspected = driver.inspect();
    Shape {
        writes: tally.writes(),
        distinct: tally.distinct(),
        verbs: tally.verbs(),
        asked,
        tallied_ask: tally.asked(),
        regions: inspected.hits().len(),
        stops: inspected.stop_count(),
        iterated,
    }
}

/// **What the frame costs a component**, drawn through [`Direct`] rather than through a [`Tally`].
///
/// A report and never a gate, and the separation from [`frame`] is [`crate::listing::volume_cost`]'s
/// and for its reason: a `Tally` keeps a set of every cell it sees and folds a width over every
/// string, so a figure taken with one in the loop is a report about the instrument. **On this page
/// that is not a nicety** — the width fold is `O(the ask)`, which is the quantity under test.
///
/// # Panics
///
/// Panics on zero frames.
///
/// [`Direct`]: crate::ink::Direct
pub fn frame_cost(plan: Plan, frames: u32) -> Duration {
    assert!(frames > 0, "a per-frame figure needs a frame");
    let index = index_for(plan);
    let mut driver = crate::runner::driver_at(W, H, Density::default());
    let mut ink = Direct;
    driver.frame(|cx| {
        let _ = draw_into(&mut ink, cx, plan, &index);
    });
    let started = Instant::now();
    for _ in 0..frames {
        driver.frame(|cx| {
            let _ = draw_into(&mut ink, cx, plan, &index);
        });
    }
    started.elapsed() / frames
}

/// **A warmed driver a caller can turn one frame at a time.**
///
/// [`frame_cost`] builds its own driver, which is right for a timing — `Driver::headless` plus
/// `Theme::authored` is tens of microseconds of one-off work and a figure carrying it would be a
/// report about attaching a terminal. It is **wrong for an allocation total**, where the same
/// one-off work is dozens of allocations and a total that carried them would say a frame allocates
/// when the frame allocates nothing. §21's second refinement, one step further along: *a mean cannot
/// see anything below n; a total can see one* — and a total measured around the wrong bracket
/// cannot see zero.
pub struct Frames {
    driver: vitui_runtime::Driver,
    plan: Plan,
    /// **The caller's index, built once**, because a frame that materialised its own order would be
    /// a frame measuring §10's 21 158 µs rather than this screen's.
    index: Order,
}

impl Frames {
    /// A driver at [`W`] by [`H`], with one frame already drawn through it.
    pub fn warmed(plan: Plan) -> Frames {
        let index = index_for(plan);
        let mut driver = crate::runner::driver_at(W, H, Density::default());
        driver.frame(|cx| {
            let _ = draw_into(&mut Direct, cx, plan, &index);
        });
        Frames {
            driver,
            plan,
            index,
        }
    }

    /// Draw one more, through [`Direct`].
    ///
    /// [`Direct`]: crate::ink::Direct
    pub fn draw(&mut self) {
        let plan = self.plan;
        let index = &self.index;
        self.driver.frame(|cx| {
            let _ = draw_into(&mut Direct, cx, plan, index);
        });
    }
}

/// **The frame at every one of [`VOLUMES`]**, which is what §7's *flat at 1k / 100k / 1M* is
/// asserted over.
pub fn across_volumes(plan: Plan) -> Vec<(u64, Shape)> {
    VOLUMES
        .into_iter()
        .map(|rows| (rows, frame(plan.over(rows))))
        .collect()
}

/// **The three drawings of one rectangle, as verbs.** `(list, tree, table)`.
pub fn verbs_by_drawing(depth: u16) -> (u64, u64, u64) {
    let plan = Plan::tree_at(depth);
    (
        frame(plan.as_drawn(Drawn::List)).verbs,
        frame(plan.as_drawn(Drawn::Tree)).verbs,
        frame(plan.as_drawn(Drawn::Table)).verbs,
    )
}

/// **Which of §20's counters tell the two indents apart at [`DEEP`], and which prefer the defect.**
///
/// Returns `(clamped, unclamped)`. The scene's whole argument is in the pair: every counter this
/// crate can read is either identical or **better** on the arm that asks for four hundred times the
/// cells.
pub fn the_numbers(depth: u16) -> (Shape, Shape) {
    let plan = Plan::tree_at(depth);
    (frame(plan), frame(plan.unclamped()))
}

// ── the narrow axis: §7's partition read as a function of the width ──────────────────────────────

/// **The four widths the narrow scene is played at.** Production 07.
///
/// [`W`] is the control, and the other three are the three regimes §7's clamp has. `min(depth * 2,
/// w - 2)` reserves two columns — the chevron and one cell of label — so a row at depth *d* passes
/// through three states as the rectangle narrows, and **which state it is in is a function of the
/// row's own depth**:
///
/// | width | the indent | the label | the two arms |
/// |---|---|---|---|
/// | [`W`] = 300 | `2d` on both rows | whole, 21 of 279 columns | the **same screen** |
/// | [`TRUNCATES`] = 40 | `2d` on both rows | truncated to 19 and to 17 | the **same screen** |
/// | [`BINDS_DEEPER`] = 22 | clamped on the deeper row only | one column | 40 cells over 40 rows |
/// | [`BINDS_BOTH`] = 21 | clamped on both | one column, or none | 120 cells over 80 rows |
///
/// **The label truncating is not what makes the flag visible; the clamp binding is** — which is the
/// distinction a scene played at one narrow width cannot draw, and the reason there are four widths
/// here rather than two. See [`MISDRAWN`].
pub const WIDTHS: [u16; 4] = [W, TRUNCATES, BINDS_DEEPER, BINDS_BOTH];

/// **The width where the label truncates and the clamp does not bind. Forty.**
///
/// At [`SHALLOW`] the indent is 20 and the deeper row's is 22, so the label gets 19 and 17 columns
/// of the 21 it wants. The ticket's own criterion — *the width where the label truncates is one of
/// them* — and it is the width that answers it while leaving the indent alone.
pub const TRUNCATES: u16 = 40;

/// **The width where the clamp binds on the deeper row and not on the shallower one.
/// Twenty-two.**
///
/// `min(22, 20) = 20` against `min(20, 20) = 20`: the parents are unchanged and the leaves are not,
/// so the unclamped arm is wrong on **half** the rows. That is the sharpest form the axis takes —
/// a defect whose reach is a function of the row's depth *and* the rectangle's width at once.
pub const BINDS_DEEPER: u16 = 22;

/// **The width where the clamp binds on both rows. Twenty-one.**
///
/// `min(20, 19) = 19` and `min(22, 19) = 19`, so every row is drawn differently on the two arms —
/// and in **two different ways**: a parent keeps its chevron and loses its label cell, a leaf loses
/// everything. It is also the one width of the four where [`Chevron`]'s two spellings disagree; see
/// [`PRESSED`].
///
/// [`Chevron`]: crate::collect::Chevron
pub const BINDS_BOTH: u16 = 21;

/// **How many rows the narrow screen's content holds. A hundred and sixty**, twice [`H`].
///
/// More than the viewport, so the tree is virtualised and the screen is a real one; **not** [`NODES`]
/// , because the width is this scene's variable and the row count is scene 8's. A million-entry
/// `Order` built four times over two arms would price the fixture and not the axis.
pub const NARROW_ROWS: usize = 2 * H as usize;

/// **What the unclamped indent misdraws at each of [`WIDTHS`], as `(cells, rows)`.**
///
/// Against [`narrow_reference`], which is the correct arm's own oracle. The first two entries are
/// `(0, 0)` and that is the measurement rather than a gap: **at 300 and at 40 the two builds are
/// the same screen, cell for cell** — §7's *on a shallow tree the same flag is invisible*, which
/// that scene states as a fact about the **depth** and which is a fact about the **pair**.
///
/// The last two are 40 cells over 40 rows and 120 over 80. Neither is a cell count a reader can
/// guess from the other: at 22 only the deeper half of the rows moves and each moves by one cell,
/// at 21 every row moves and the parents move by two.
pub const MISDRAWN: [(usize, usize); 4] = [(0, 0), (0, 0), (40, 40), (120, 80)];

/// **What the correct arm writes at each of [`WIDTHS`], and the unclamped arm writes the same.**
///
/// `w * H` at every width — 24 000, 3 200, 1 760, 1 680 — **on both arms**, which is the narrow
/// axis's own version of §7's finding and a stronger one. At [`DEEP`] the two arms differ by
/// *cells asked for* and agree on everything the engine reports; here they agree on the engine's
/// report at every width including the two where the pictures differ, because the clip eats exactly
/// what the collapse loses. **No write counter can see this axis at any width.**
pub const WRITES: [u64; 4] = [
    W as u64 * H as u64,
    TRUNCATES as u64 * H as u64,
    BINDS_DEEPER as u64 * H as u64,
    BINDS_BOTH as u64 * H as u64,
];

/// **What the two arms make of it in verbs, as `(clamped, unclamped)` per width.**
///
/// The one counter that moves, and **it moves the wrong way**: 320 → 240 → 160 → 120 on the
/// defective arm against 320 → 240 → 240 → 240 on the correct one, so a verb *ceiling* approves of
/// every arm it cannot see and prefers the two it can.
///
/// The correct arm's own figure moves too — 320 at [`W`] and 240 everywhere else — because a label
/// that fills its room needs no trailing pad. So a verb count is not even a fixed expectation on
/// this axis, which is what makes the equality the gate and this a report.
pub const VERBS: [(u64, u64); 4] = [(320, 320), (240, 240), (240, 160), (240, 120)];

/// **Which column a press has to land on for the chevron to fold, per width, as `(the rectangle's,
/// the context's)`.**
///
/// The drawn column first — `indent_columns` of the rectangle `tree` was handed, which is what
/// [`crate::collect::Chevron::FromTheRectangle`] reads — and the column
/// [`crate::collect::Chevron::FromTheContext`] would read second, which is the indent of the
/// **screen** the tree is drawn on. The rig is a `w`-column tree inside a [`W`]-column screen.
///
/// **They are the same number at three of the four widths**, and every gate on this map that draws
/// a tree draws it at the full width of its screen — where they are the same number at *every*
/// width. That is `header_row`'s recorded defect (*every gate passed because every one of them
/// plays at `x == 0`*) on the pointer axis, and it is why this reading is the narrow scene's rather
/// than a press scene's: the two spellings are indistinguishable until one width clamps and the
/// other does not.
pub const PRESSED: [(usize, usize); 4] = [(20, 20), (20, 20), (20, 20), (19, 20)];

/// **One narrow frame, recorded through a [`Pen`].** Returns `(the screen, writes, verbs)`.
///
/// The tree is drawn into a `width`-column rectangle of a `width`-column screen, so this is the
/// picture half and nothing about it depends on the context; [`pressed`] is the half that needs the
/// two to differ.
///
/// The row drawer is `label_and_pad`'s arithmetic — truncate to the room, then fill it — because
/// §2 gives the row drawer every cell of the rectangle it was handed and a caller that wrote past
/// it would be measuring its own defect rather than the component's.
pub fn narrow_screen(width: u16, indent: Indent) -> (Canvas, u64, u64) {
    let index = nested(NARROW_ROWS);
    let mut driver = crate::runner::driver_at(width, H, Density::default());
    let mut pen = Pen::new(width, H);
    let mut st = TreeState::new();
    let opts = TreeOpts::default();
    driver.frame(|cx| {
        let area = cx.area();
        let body = cx.theme().paint(Role::Body);
        let mut find = |_: &str, _: std::ops::Range<usize>| None;
        let mut row =
            |ink: &mut Pen, cx: &mut Ctx<'_, '_>, r: Rect, _n: crate::collect::Node, _f: Face| {
                let _ = label_and_pad(ink, cx, r.x, r.y, r.w, body);
            };
        let _ = match indent {
            Indent::Clamped => tree_into(
                &mut pen, cx, area, &mut st, &opts, &index, &mut find, &mut row,
            ),
            Indent::Unclamped => coll_defective::unclamped_indent(
                &mut pen, cx, area, &mut st, &opts, &index, &mut find, &mut row,
            ),
        };
    });
    let (writes, verbs) = (pen.tally().writes(), pen.tally().verbs());
    (pen.into_canvas(), writes, verbs)
}

/// **The obviously-correct, far-too-slow rendering of the same screen: one `Ctx::set` a cell.**
///
/// `crate::runner::reference`'s shape and its rule, restated for a tree: *share no code with the
/// fast path*. It visits every cell of every row, decides what belongs there from the row's own
/// depth, and writes it one cell at a time — where the component writes one run, one cell and one
/// text a row.
///
/// # What it shares is [`indent_columns`], and that is this crate's own settled policy
///
/// > `tree` calls it and so does every instrument that wants to know what the component asked for,
/// > which is what keeps the two from being a model and a copy of a model: the number the screen
/// > sums and the number the run is drawn with come out of this function.
///
/// So the equality cannot fail *on the clamp's arithmetic*, and it can fail on everything the
/// partition is made of: where the chevron goes, which glyph it is, how wide the label rectangle is,
/// whether a collapsed row still draws one, and whether the pad reaches the edge. The unclamped arm
/// is what proves it has teeth — [`MISDRAWN`]'s last two entries are this oracle catching it.
pub fn narrow_reference(width: u16) -> Canvas {
    let index = nested(NARROW_ROWS);
    let mut driver = crate::runner::driver_at(width, H, Density::default());
    let mut pen = Pen::new(width, H);
    driver.frame(|cx| {
        let body = cx.theme().paint(Role::Body);
        let open = cx.theme().glyph(Glyph::ArrowDown).to_string();
        let shut = cx.theme().glyph(Glyph::ArrowRight).to_string();
        let label: Vec<String> = LABEL.chars().map(|c| c.to_string()).collect();
        for y in 0..H {
            let i = usize::from(y);
            let Some(e) = index.at(i) else { continue };
            let ind = indent_columns(Indent::Clamped, e.depth, width);
            let chevron = if e.is_folded() {
                shut.as_str()
            } else if has_children(&index, i) {
                open.as_str()
            } else {
                " "
            };
            let room = usize::from(width).saturating_sub(ind + 1);
            for x in 0..usize::from(width) {
                let cluster: &str = if x < ind {
                    " "
                } else if x == ind {
                    chevron
                } else {
                    let k = x - ind - 1;
                    if k < room.min(label.len()) {
                        label[k].as_str()
                    } else {
                        " "
                    }
                };
                let _ = pen.set(
                    cx,
                    i32::try_from(x).unwrap_or(i32::MAX),
                    i32::from(y),
                    cluster,
                    body,
                );
            }
        }
    });
    pen.into_canvas()
}

/// **Whether a press at column `col` folded the row under it**, on a `width`-column tree drawn
/// inside a [`W`]-column screen.
///
/// # It takes four frames and every one of them is the runtime's cadence
///
/// `crate::wheel::tapped`'s cadence exactly, and for its reason: *the grab is awarded at `end` from
/// the index that has just drawn, so `Response::pressed` is false on the frame that processes the
/// `Down` and true on the one after* — and §7's fold reads `Response::press_began`. So the pointer
/// needs a frame to have a position, the `Down` needs one to be processed, and the edge needs one to
/// land. A gate that posted the press and read the answer off the next frame reads `None` on every
/// arm and every column, which is a dead instrument reporting a clean refusal.
pub fn pressed(width: u16, chevron: Chevron, col: u16) -> Option<Ask> {
    let index = nested(NARROW_ROWS);
    let mut driver = crate::runner::driver_at(W, H, Density::default());
    let mut st = TreeState::new();
    let rect = Rect::new(0, 0, width, H);
    let opts = TreeOpts::default();
    let once = |driver: &mut Driver, st: &mut TreeState| {
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            let mut find = |_: &str, _: std::ops::Range<usize>| None;
            let mut row = |ink: &mut Direct,
                           cx: &mut Ctx<'_, '_>,
                           r: Rect,
                           _n: crate::collect::Node,
                           _f: Face| {
                let _ = label_and_pad(ink, cx, r.x, r.y, r.w, body);
            };
            let _ = match chevron {
                Chevron::FromTheRectangle => tree_into(
                    &mut Direct,
                    cx,
                    rect,
                    st,
                    &opts,
                    &index,
                    &mut find,
                    &mut row,
                ),
                Chevron::FromTheContext => coll_defective::chevron_from_the_context(
                    &mut Direct,
                    cx,
                    rect,
                    st,
                    &opts,
                    &index,
                    &mut find,
                    &mut row,
                ),
            };
        });
    };
    driver.post_mouse(pointer(col, MouseKind::Move));
    once(&mut driver, &mut st);
    driver.post_mouse(pointer(col, MouseKind::Down(Button::Left)));
    once(&mut driver, &mut st);
    once(&mut driver, &mut st);
    st.ask.standing()
}

/// A pointer event on row 0 at `x`.
fn pointer(x: u16, kind: MouseKind) -> Mouse {
    Mouse {
        x,
        y: 0,
        kind,
        buttons: Buttons::NONE,
        mods: Mods::NONE,
        at: Instant::now(),
    }
}

// ── the subject, and the scan that says whether it is here ───────────────────────────────────────

/// **The component these two scenes are scenes of, and it is not declared yet.**
pub const SUBJECTS: [&str; 1] = ["tree"];

/// Where [`SUBJECTS`] is declared, as `(module file, the declaration)`.
///
/// The home is the freeze's, joined through [`crate::Family`]: `tree`'s first family is
/// `F7Collections`, whose module is `collect.rs`.
pub const DECLARATIONS: [(&str, &str); 1] = [("collect.rs", "pub fn tree(")];

/// **Which of [`SUBJECTS`] this crate actually declares. Today: none.**
///
/// A source scan and not a `use`, for [`crate::dense::subjects_declared`]'s reason: *the item does
/// not exist* has no expression, and a `compile_fail` fence would pass today and pass again the day
/// somebody renames the module. The predicate is `crate::dense::declares` and it is shared rather
/// than copied.
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

/// **Whether the forest stands on its subject, as a verdict rather than as a sentence.**
pub fn standing() -> Verdict {
    let declared = subjects_declared();
    Verdict::of(
        SUBJECTS.len(),
        SUBJECTS.len() - declared.len(),
        "`tree` is not declared in this crate, so what stands on the forest is a stand-in row loop \
         over a stand-in flatten index. The screen, its 81 regions, its identical writes and verbs \
         at 1k / 100k / 1M nodes and at depth 10 and 59 999, its unclamped indent asking for four \
         hundred times the cells while every counter prefers it, its fold of 349 524 rows at one \
         run against a permutation's hundreds of thousands, and its round trip coming back 349 526 \
         of 500 000 under the prototype's own splice are measured and green; what is missing is the \
         subject",
        "components 17",
    )
}

/// **The sentence a scene waiting for its subject fails with**, or `None` once it is declared.
///
/// Criterion 6, and it is the distinction the whole ticket rests on: a scene that fails because it
/// is unimplemented and a scene that fails because the code is wrong are **the same failure** unless
/// the message separates them.
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
         components are undeclared — {}. This is not a defect in the screen. The forest is drawn at \
         depth 59 999, its verbs are counted against a plain list's and a twelve-column table's, \
         its unclamped indent is stood up and caught by the one counter that can see it, its fold \
         removes 349 524 rows at one run, and its round trip comes back 349 526 of 500 000 under \
         the splice the prototype shipped — see `crate::forest::tests`. Inverted by \
         `components 17`",
        owed.len(),
        SUBJECTS.len(),
        owed.join(", "),
    ))
}

/// **Fail with the subject that is missing, the file it belongs in, and the ticket.**
///
/// # Panics
///
/// Panics while [`SUBJECTS`] is undeclared, which is **today**. Components ticket 17 inverts it.
pub fn assert_stands_up(scene: &str) {
    if let Some(message) = owed_message(&subjects_declared(), scene) {
        panic!("{message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The frame is the same frame at 1k, 100k and 1M nodes, and at depth 10 and 59 999.**
    ///
    /// §7's *flat at 1k / 100k / 1M nodes and unchanged at depth 59 999*, as an equality over the
    /// whole [`Shape`] rather than over the counter that is easiest to move. The timing is not here:
    /// `examples/tree_numbers.rs` prints it, and §20's rule is why.
    #[test]
    fn the_forest_draws_the_same_frame_at_every_volume_and_at_every_depth() {
        let across = across_volumes(Plan::tree_at(SHALLOW));
        let first = across[0].1;
        assert_eq!(first.writes, u64::from(W) * u64::from(H));
        assert_eq!(first.distinct, first.writes, "and it is a partition");
        assert_eq!(first.regions, REGIONS, "one tree and eighty rows");
        assert_eq!(
            first.stops, STOPS,
            "a virtualised collection is one tab stop"
        );
        assert_eq!(first.iterated, u64::from(H), "the window and nothing else");
        for (rows, shape) in &across {
            assert_eq!(
                *shape, first,
                "the forest at {rows} nodes is not the forest at {} nodes",
                across[0].0
            );
        }

        // **The axis a list does not have.** Same screen, 59 989 levels further down.
        let deep = frame(Plan::tree_at(DEEP));
        assert_eq!(
            (
                deep.writes,
                deep.distinct,
                deep.asked,
                deep.regions,
                deep.stops,
                deep.iterated
            ),
            (
                first.writes,
                first.distinct,
                first.asked,
                first.regions,
                first.stops,
                first.iterated
            ),
            "every counter but the verbs is unchanged at depth 59 999"
        );
        assert_eq!(
            (first.verbs, deep.verbs),
            (VERBS_A_ROW.1 * u64::from(H), DEEP_VERBS),
            "and the verbs are not"
        );
        assert_eq!(
            first.verbs - deep.verbs,
            u64::from(H),
            "the trailing pad, one verb a row, with nothing left to pad: a clamped indent of \
             `w - 2` leaves the label one column and the label fills it. §7's *unchanged at depth \
             59 999* is a statement about the cells, and the prototype's own table says 634 against \
             640 in the same column"
        );
    }

    /// **The `+` costs two verbs a row, and that is what reproduces of §6's headline.**
    ///
    /// 640 against 418 is 222 over 111 rows. The absolute figures are a prototype screen this
    /// ticket does not own — two tree panels plus a menu bar, a toolbar and a status bar — and
    /// subtracting the furniture makes both of them exact: `111 x 2 + 196` and `111 x 4 + 196`.
    /// What is asserted here is the per-row figure, which is the mechanism.
    #[test]
    fn a_tree_costs_two_verbs_a_row_more_than_a_list_and_fewer_than_twelve_columns() {
        let (list, tree, table) = verbs_by_drawing(SHALLOW);
        assert_eq!(
            (list, tree, table),
            (
                VERBS_A_ROW.0 * u64::from(H),
                VERBS_A_ROW.1 * u64::from(H),
                VERBS_A_ROW.2 * u64::from(H)
            ),
            "two, four and twenty-four verbs a row over eighty rows"
        );
        assert_eq!(
            tree - list,
            2 * u64::from(H),
            "§7's own sentence: the `+` costs two verbs a row — the indent run and the chevron cell"
        );
        assert!(tree < table, "and a tree is not a table");

        // The remembered figures decompose the same way, which is why the difference between them
        // and the ones above is a screen and not a mechanism.
        let (r_list, r_tree, _) = REMEMBERED_VERBS;
        assert_eq!(r_tree - r_list, 222, "222 verbs over 111 rows");
        assert_eq!((r_tree - r_list) / 111, VERBS_A_ROW.1 - VERBS_A_ROW.0);
        assert_eq!(r_list - 111 * VERBS_A_ROW.0, 196, "the screen's furniture");
        assert_eq!(r_tree - 111 * VERBS_A_ROW.1, 196, "the same furniture");
    }

    /// **The trap: an unclamped indent asks for four hundred times the cells and every counter
    /// prefers it.**
    ///
    /// Both halves, because the contrast is the scene's entire argument.
    ///
    /// - The engine reports **the same number of columns written** on both arms, and the same
    ///   distinct cells: the row is a partition either way, because the clip eats the overrun.
    /// - The defective arm makes **fewer verbs**, because the label rectangle collapses.
    /// - The defective arm declares the same regions and the same stops.
    /// - The only quantity that moves is **cells asked for**.
    #[test]
    fn an_unclamped_indent_asks_for_four_hundred_times_the_cells_and_costs_fewer_verbs() {
        let (clamped, unclamped) = the_numbers(DEEP);

        assert_eq!(
            clamped.writes, unclamped.writes,
            "the engine reported the same columns on both arms, which is why no write count can \
             see this"
        );
        assert_eq!(clamped.distinct, unclamped.distinct, "and the same cells");
        assert_eq!(clamped.regions, unclamped.regions);
        assert_eq!(clamped.stops, unclamped.stops);
        assert_eq!(
            clamped.iterated, unclamped.iterated,
            "the window is the same"
        );
        assert!(
            unclamped.verbs < clamped.verbs,
            "the defective arm made {} verbs against {}, which is the direction that matters: \
             every counter but the ask is on its side",
            unclamped.verbs,
            clamped.verbs
        );

        // **The one that sees it.**
        let asked = u64::from(W) * u64::from(H);
        assert_eq!(
            clamped.asked, asked,
            "the clamped row asks for its rectangle"
        );
        assert_eq!(clamped.asked, clamped.writes, "and asks for what it wrote");
        assert_eq!(
            unclamped.asked,
            MAX_INDENT as u64 * u64::from(H),
            "119 998 columns a row over eighty rows"
        );
        assert_eq!(unclamped.asked, 9_599_840);
        assert_eq!(
            unclamped.asked * 100 / clamped.asked,
            39_999,
            "399.99x on this screen. §7's own is 165x, over a screen with 111 tree rows on it and \
             a menu bar, a toolbar and a status bar around them — a prototype screen this ticket \
             does not own"
        );

        // **And at depth ten the same flag is invisible**, which is the statement about the scene
        // list rather than about the gate.
        let (shallow, shallow_broken) = the_numbers(SHALLOW);
        assert_eq!(
            shallow, shallow_broken,
            "at depth ten the indent fits, so the two arms are one frame and removing this scene \
             removes the ability to distinguish them"
        );
    }

    /// **Scene 46: §7's partition is exact at every width, and the width is the second dimension
    /// of §7's own statement about the scene list.**
    ///
    /// Production 07, `(tree, narrow)`. Three readings, and no one of them can see the other two.
    ///
    /// # The equality is the gate
    ///
    /// The shipped tree against [`narrow_reference`] at all four of [`WIDTHS`] — **0 cells over 0
    /// rows** every time, which is §7's *one `Ink::run` of spaces, one chevron cell and the label
    /// taking the rest* holding from three hundred columns down to twenty-one. The oracle writes
    /// one cell at a time where the component writes one run, one cell and one text, so it can fail
    /// on where the chevron goes, which glyph it is, how wide the label rectangle is and whether the
    /// pad reaches the edge; [`MISDRAWN`]'s last two entries are it failing.
    ///
    /// # The counts say what the equality cannot, and what they say is that they are blind
    ///
    /// **[`WRITES`] is `w * H` on both arms at every width**, including the two where the pictures
    /// differ — the clip eats exactly what the collapse loses. So this axis is invisible to every
    /// write counter at every width, which is a stronger reading than §7's own: that scene needs
    /// *cells asked for* to separate the arms at [`DEEP`], and here nothing separates them but the
    /// picture. [`VERBS`] moves, downward, on the defective arm.
    ///
    /// # The label truncating is not what makes the flag visible
    ///
    /// [`MISDRAWN`] is `(0, 0)` at 300 **and at 40**, where the label is already cut from 21 columns
    /// to 19 and 17. What makes it visible is the **clamp binding**, at 22 on half the rows and at
    /// 21 on all of them — so a narrow scene played at one narrow width would report a number and
    /// name the wrong cause.
    #[test]
    fn a_trees_partition_is_exact_at_every_width_and_the_clamp_is_what_the_flag_needs() {
        assert_stands_up("the narrow forest");

        for (k, width) in WIDTHS.into_iter().enumerate() {
            let (subject, writes, verbs) = narrow_screen(width, Indent::Clamped);
            let reference = narrow_reference(width);
            subject
                .diff(&reference)
                .assert_clean(&format!("the tree's partition at {width} columns"));

            let (broken, broken_writes, broken_verbs) = narrow_screen(width, Indent::Unclamped);
            let misdrawn = broken.diff(&reference);
            assert_eq!(
                (misdrawn.cells, misdrawn.rows),
                MISDRAWN[k],
                "the unclamped indent misdraws a different amount at {width} columns"
            );

            assert_eq!(writes, WRITES[k], "the tree wrote its rectangle at {width}");
            assert_eq!(
                broken_writes, writes,
                "the two arms wrote a different number of columns at {width}, which would make \
                 this axis visible to a write counter — it is not, at any width"
            );
            assert_eq!((verbs, broken_verbs), VERBS[k], "verbs at {width}");
        }

        // **`MISDRAWN`'s first two entries are what says the two arms are one screen at 300 and at
        // 40**, and a direct `unclamped.diff(clamped)` beside them was **deleted**: both arms are
        // already asserted equal to the reference at every width, so the third comparison holds by
        // transitivity and cannot fail. Production 09's precedent, and a code review caught this
        // one — *the reading that can fail is the one the runs already make.*
        let cut = narrow_screen(TRUNCATES, Indent::Clamped).0;
        // **The head is derived and not typed**, because a needle written out beside a screen it is
        // supposed to be read off is a second copy of the fixture: the room a depth-10 row has at
        // this width is `w − (indent + 1)`, and the label is what fits it.
        let room =
            usize::from(TRUNCATES) - (indent_columns(Indent::Clamped, SHALLOW, TRUNCATES) + 1);
        let head: String = LABEL.chars().take(room).collect();
        assert!(
            room < LABEL.chars().count(),
            "the label has to be longer than the room, or this width is not the truncating one"
        );
        assert!(
            cut.row_text(0).contains(&head),
            "the label is truncated to {room} columns at {TRUNCATES} and this is the width that \
             says so: {:?}",
            cut.row_text(0)
        );
        assert!(
            !cut.row_text(0).contains(LABEL),
            "the whole label still fits at {TRUNCATES} columns, so this width is not the \
             truncating one the ticket asked for"
        );
        assert!(
            narrow_screen(W, Indent::Clamped)
                .0
                .row_text(0)
                .contains(LABEL),
            "and it is whole at three hundred, which is what makes the pair a reading"
        );

        // **A real chevron is on the screen and a leaf's is a space**, which is what [`nested`]
        // buys over [`index_for`]: under that builder no row has a child, so an arm that lost the
        // chevron entirely would be invisible. The glyph is read **off the cell** rather than
        // compared against a literal — the theme owns the repertoire, and a gate naming `\u{25bc}`
        // would fail on a correct build at the ASCII rung.
        let clamped = narrow_screen(W, Indent::Clamped).0;
        let at = |y: u16, depth: u16| {
            let column = u16::try_from(indent_columns(Indent::Clamped, depth, W))
                .expect("a column of this screen");
            clamped
                .get(column, y)
                .map(|c| c.cluster.clone())
                .unwrap_or_default()
        };
        let parent = at(0, SHALLOW);
        let leaf = at(1, SHALLOW + 1);
        assert_ne!(
            parent, " ",
            "row 0 has a child, so its chevron cell carries a glyph rather than a space"
        );
        assert_eq!(
            leaf, " ",
            "row 1 is a leaf, so its chevron is a space — the partition is the same partition on \
             every row (§7), which is why it is a space and not nothing"
        );
    }

    /// **Scene 46's third reading: the chevron's *pressed* column is the rectangle's and not the
    /// context's, and the two are one number at every width a gate on this map has ever played.**
    ///
    /// [`crate::collect::Chevron`]. `tree` draws the chevron at the indent of the row rectangle it
    /// hands over and decides whether a press landed on it from the indent of the rectangle *it* was
    /// handed — two calls to [`indent_columns`] with two widths, agreeing because
    /// `collect::draw_with` gives a row the whole of `area.w`. A third fact, on the other component.
    ///
    /// # It is a narrow-axis reading and could not have been anything else
    ///
    /// `min(depth * 2, w - 2)` is the same number for two widths whenever the clamp binds on
    /// neither, so [`PRESSED`] is `(20, 20)` at three of the four widths and `(19, 20)` at
    /// [`BINDS_BOTH`]. A tree drawn at the full width of its screen has **one** width and the two
    /// spellings are then identical at every width — which is every gate on this map, and is
    /// `header_row`'s own recorded defect (*every gate passed because every one of them plays at
    /// `x == 0`*) with the width in place of the origin.
    ///
    /// What the refusal costs a user is stated as the reading: at twenty-one columns a press on the
    /// chevron a reader can see does **nothing**, and a press on the blank column beside it folds.
    #[test]
    fn a_chevron_is_pressed_at_the_column_its_own_rectangle_puts_it_at() {
        assert_stands_up("the narrow forest");

        for (k, width) in WIDTHS.into_iter().enumerate() {
            let rect = indent_columns(Indent::Clamped, SHALLOW, width);
            let context = indent_columns(Indent::Clamped, SHALLOW, W);
            assert_eq!(
                (rect, context),
                PRESSED[k],
                "the two indents at {width} columns inside a {W}-column screen"
            );

            let at_rect = u16::try_from(rect).expect("a column of a screen");
            let at_context = u16::try_from(context).expect("a column of a screen");

            assert_eq!(
                pressed(width, Chevron::FromTheRectangle, at_rect),
                Some(Ask::Collapse(0)),
                "the shipped tree folds row 0 when the chevron a reader can see is pressed, at \
                 {width} columns"
            );
            assert_eq!(
                pressed(width, Chevron::FromTheContext, at_context),
                Some(Ask::Collapse(0)),
                "and the refusal folds at the column the context puts it at"
            );

            // **The cross pair, as one biconditional a width and not as an `if`/`else`.**
            //
            // The branch this replaced asserted, on the three widths where the two indents agree,
            // that the two builds answer the same thing at the same column — which the two
            // assertions above had **already pinned to `Some(Ask::Collapse(0))` each**, so it could
            // not fail once they passed. `crate::CLAUDE.md`'s trap, *an equality between two
            // derivations of one declaration*, and a code review caught it. What replaces it is the
            // reading that can fail in both directions at every width: **a build folds at the other
            // build's column exactly when the two widths clamp to the same number.** Where they
            // agree that is a second `Some`; where they do not it is `None`, and nothing about the
            // assertion changes shape between the two cases.
            let agree = rect == context;
            assert_eq!(
                pressed(width, Chevron::FromTheContext, at_rect).is_some(),
                agree,
                "the refusal folds at the column the *rectangle* draws the chevron in only where \
                 the two widths clamp alike — at {width} columns they are {rect} and {context}"
            );
            assert_eq!(
                pressed(width, Chevron::FromTheRectangle, at_context).is_some(),
                agree,
                "and the shipped build folds at the column the *context* would have used only \
                 there too, at {width} columns"
            );
        }

        // **Exactly one of the four widths separates the two builds**, which is the finding rather
        // than a property of these four numbers: the pair moves only where one width clamps and the
        // other does not.
        let separating = WIDTHS
            .into_iter()
            .filter(|w| {
                indent_columns(Indent::Clamped, SHALLOW, *w)
                    != indent_columns(Indent::Clamped, SHALLOW, W)
            })
            .count();
        assert_eq!(
            separating, 1,
            "the narrow scene has to play a width where the clamp binds on the component's \
             rectangle and not on its context, or the press reading compares a build with itself"
        );
    }

    /// **The tally saturates where the caller does not, and the gap is 45.4%.**
    ///
    /// `vitui_engine::width_of` returns a `u16` and saturates, so an indent of 119 998 columns is
    /// reported as 65 535. The instrument still says *this arm asks for two hundred and eighteen
    /// times what the other one does* — it is not blind, it is short — and the number the ratio
    /// above is taken over is the caller's own `u64`.
    ///
    /// The prototype met the same `u16` from the other side and got a *different* wrong answer:
    /// `depth * 2` cast to `u16` is 54 462. Neither is a defect in the type.
    #[test]
    fn the_tally_saturates_where_the_caller_does_not() {
        let (clamped, unclamped) = the_numbers(DEEP);
        assert_eq!(
            clamped.tallied_ask, clamped.asked,
            "nothing saturates on the arm that fits"
        );
        assert_eq!(
            unclamped.tallied_ask,
            u64::from(u16::MAX) * u64::from(H),
            "65 535 a row, which is the widest a `u16` column count can express"
        );
        assert!(
            unclamped.tallied_ask < unclamped.asked,
            "the instrument under-reports the ask"
        );
        assert_eq!(
            (unclamped.asked - unclamped.tallied_ask) * 1_000 / unclamped.asked,
            453,
            "45.3% of the ask is past the end of a `u16`"
        );
        assert_eq!(
            unclamped.tallied_ask / clamped.tallied_ask,
            218,
            "and the short answer still separates the arms"
        );

        // The prototype's own collision with the same type, as arithmetic rather than as a story.
        assert_eq!(MAX_INDENT % (1 << 16), 54_462, "`119 998 as u16`");
    }

    /// **The index: a splice equals a rebuild, and a fold's interval is the forest's subtree.**
    ///
    /// Half of §21's row 14. The equality is against a walk of the forest, which is the *slow*
    /// producer §7 keeps only as an oracle — proportional to what remains, at 85 ns a row on a
    /// shuffled forest.
    #[test]
    fn a_spliced_index_equals_a_rebuilt_one() {
        let forest = Forest::folded();
        let mut flat = Flat::expanded(&forest);
        assert_eq!(flat.len(), forest.len());

        // The interval, from the index and from the data, which is §7's own pair.
        assert_eq!(flat.span(FOLD_AT) as u64, FOLD_ROWS);
        assert_eq!(forest.descendants(FOLD_AT) as u64, FOLD_ROWS);

        let edit = flat.fold(FOLD_AT);
        assert_eq!(
            edit,
            Splice {
                at: FOLD_AT + 1,
                removed: FOLD_ROWS as usize,
                inserted: 0
            }
        );
        let rebuilt = Flat::rebuilt(&forest, flat.collapsed());
        assert_eq!(flat.rows, rebuilt.rows, "splice == rebuild after a fold");
        assert_eq!(flat.len() as u64, NODES - FOLD_ROWS);

        let back = flat.unfold(&forest, FOLD_AT);
        assert_eq!(back.inserted as u64, FOLD_ROWS);
        assert_eq!(
            flat.rows,
            Flat::expanded(&forest).rows,
            "splice == rebuild after an unfold, and the unfold is the fold's inverse"
        );
        assert!(flat.collapsed().is_empty());
    }

    /// **The other half of row 14: the fold/unfold round trip, over both spellings of the splice.**
    ///
    /// The correct one restores the selection **exactly**; the one the prototype shipped brings back
    /// 349 526 rows of 500 000 with the screen perfectly correct, one run before and one run after,
    /// and a timing that does not move. This is the third time on this map that a positional store's
    /// silent failure was invisible to every count.
    #[test]
    fn a_fold_and_an_unfold_restore_the_selection_exactly_and_the_shipped_splice_did_not() {
        let forest = Forest::folded();
        assert_eq!(
            round_trip(&forest, Straddle::Split),
            SELECTED,
            "the round trip is exact"
        );
        assert_eq!(
            round_trip(&forest, Straddle::Forget),
            ROUND_TRIP_WRONG,
            "a run straddling the insertion point was not split, so the parked run came back on \
             top of a selection that had never moved"
        );
        assert_ne!(ROUND_TRIP_WRONG, SELECTED);

        // And `Drop` keeps exactly what is left, which is arithmetic and not a measurement.
        let mut flat = Flat::expanded(&forest);
        let mut selection = Runs::of(0, SELECTED);
        let edit = flat.fold(FOLD_AT);
        let parked = selection.splice(edit, Policy::Drop, Straddle::Split);
        assert_eq!(selection.rows(), KEPT, "500 000 - 349 524");
        assert_eq!(selection.count(), 1, "and it is still one run");
        assert_eq!(parked.count(), 0, "`Drop` parks nothing");

        // `Clear` throws away the runs that were nowhere near the fold, which is the refusal as a
        // measurement rather than as a sentence.
        let mut cleared = Runs::of(0, SELECTED);
        let _ = cleared.splice(edit, Policy::Clear, Straddle::Split);
        assert_eq!(cleared.rows(), 0);
        assert!(
            cleared.rows() < KEPT,
            "`Clear` is refused for a splice because it loses {} rows the fold never touched",
            KEPT
        );
    }

    /// **Scene 9: a sort is a permutation and a fold is an interval.**
    ///
    /// One selection, one million rows, 349 524 removed. Under the splice it is **one run, no bytes,
    /// and a transform proportional to the runs**; under the permutation the run list shatters into
    /// hundreds of thousands and the transform is proportional to the selected rows.
    ///
    /// §21's figures are 1 run / 0.04 µs against 297 180 / 24 338 µs. **The run count is a property
    /// of that forest's key shuffle and not of the mechanism** — a contiguous half of a million rows
    /// through a uniform shuffle breaks into about 250 000 runs — so what is asserted here is the
    /// side of the cliff and the direction, and `examples/tree_numbers.rs` prints both columns.
    #[test]
    fn a_fold_transforms_a_selection_in_one_run_and_a_sort_shatters_it() {
        let forest = Forest::folded();
        let (spliced, permuted, splice_took, permutation_took) = reconciled(&forest);

        assert_eq!(spliced.runs, 1, "§21's own number, and it is the mechanism");
        assert_eq!(spliced.rows, KEPT);
        assert_eq!(spliced.bytes, 0, "an interval edit materialises nothing");

        assert_eq!(
            permuted.runs, PERMUTED_RUNS,
            "the permutation left {} runs. A run list that does not shatter under a sort would \
             mean the two gestures are one, and the count is deterministic because the shuffle has \
             a seed — see `PERMUTED_RUNS` for why it is not §21's 297 180",
            permuted.runs
        );
        assert!(permuted.runs < SELECTED as usize);
        assert_eq!(
            permuted.rows, SELECTED,
            "a sort loses no row, it moves them"
        );
        assert_eq!(permuted.bytes, SELECTED as usize * 8);
        assert!(
            permuted.runs > spliced.runs * 100_000,
            "{} runs against {}",
            permuted.runs,
            spliced.runs
        );

        // The timings are a report and are asserted only at cliff granularity — three orders of
        // magnitude, with the headroom in the message.
        assert!(
            permutation_took > splice_took * 100,
            "the permutation took {permutation_took:?} and the splice {splice_took:?}; the gate is \
             the run count and this is the cliff beside it"
        );
    }

    /// **A half-tree fold parks one `Run`, and the round trip is exact.** Sixteen bytes.
    #[test]
    fn a_half_tree_fold_parks_one_run_of_sixteen_bytes() {
        let forest = Forest::folded();
        let mut flat = Flat::expanded(&forest);
        let mut selection = Runs::of(0, SELECTED);
        let edit = flat.fold(FOLD_AT);
        let parked = selection.splice(edit, Policy::Stash, Straddle::Split);
        assert_eq!(parked.count(), PARKED_RUNS);
        assert_eq!(parked.rows(), FOLD_ROWS, "bounded by runs, not by rows");
        assert_eq!(
            std::mem::size_of::<Run>() * parked.count(),
            16,
            "sixteen bytes, which is the whole of what `Stash` costs"
        );
    }

    /// **Criterion 6, inverted: the two scenes stand on the component they are scenes of.**
    ///
    /// It read *the forest is red because `tree` is not declared* until components ticket 17, and
    /// inverting it was a deliberate edit in three files — here, in [`crate::scenes`]'s two
    /// standings and in [`crate::gates::REGISTER`].
    ///
    /// **Two halves, because the subject scan alone is not enough**, which is `crate::grid`'s
    /// precedent: the scan says `tree` is declared where the freeze homes it, and what it cannot
    /// say is that *this screen draws through it*. A screen that kept its stand-in row loop beside
    /// a declared component would pass the first half for ever, so the second is read out of the
    /// source — *this function calls that one* has no expression a test can write.
    #[test]
    fn the_forest_stands_on_the_tree_it_is_a_screen_of() {
        assert_eq!(
            subjects_declared(),
            SUBJECTS.to_vec(),
            "`tree` has stopped being declared where the freeze homes it. That is not a defect in \
             the screen: `crate::forest::DECLARATIONS` names the file and the signature it is \
             looked for at"
        );
        let verdict = standing();
        assert!(verdict.met());
        match verdict {
            Verdict::Met { over } => assert_eq!(over, 1, "one subject, and it is here"),
            Verdict::Unmet { over, failing, .. } => {
                unreachable!("{failing} of {over} undeclared, which the assertion above caught")
            }
        }
        // And the scenes do not panic any more, which is the whole ticket in one call.
        assert_stands_up("a million-node forest at depth 59 999");
        assert_stands_up("a fold and an unfold over 349 524 rows");

        // **Both arms of the tree drawing are the component**, which is what makes every number on
        // this page a claim about `tree` rather than about a row loop this file owns.
        let source = include_str!("forest.rs");
        for call in ["tree_into(", "coll_defective::unclamped_indent("] {
            assert!(
                crate::dense::declares(source, call),
                "`draw_into` no longer calls `{call}`, so the scenes have stopped standing on the \
                 component even though the subject scan still finds it"
            );
        }
    }

    /// **The waiting message says which failure it is**, fired in both directions over a
    /// declaration list rather than over the crate — so the day `tree` is declared this half is
    /// still live.
    #[test]
    fn the_waiting_message_separates_unimplemented_from_wrong() {
        let message = owed_message(&[], "a fold over 349 524 rows").expect("no subject");
        assert!(
            message.contains("waiting for its subject rather than failing"),
            "{message}"
        );
        assert!(
            message.contains("This is not a defect in the screen"),
            "{message}"
        );
        assert!(message.contains("components 17"), "{message}");
        assert!(message.contains("src/collect.rs"), "{message}");
        assert!(message.contains("pub fn tree("), "{message}");
        assert!(message.contains("349 526"), "{message}");

        assert_eq!(owed_message(&SUBJECTS, "a fold over 349 524 rows"), None);
    }

    /// **The subject scan finds a declaration when there is one**, and the freeze homes `tree`
    /// where the scan opens.
    #[test]
    fn the_subject_scan_finds_a_declaration_when_there_is_one() {
        for (_, declaration) in DECLARATIONS {
            assert!(crate::dense::declares(
                &format!("// a comment\n{declaration}cx: &mut Ctx) {{}}\n"),
                declaration
            ));
            assert!(
                !crate::dense::declares(
                    &format!("// {declaration}…) is components 17's\n"),
                    declaration
                ),
                "a mention in a comment is not a declaration"
            );
        }

        let component = crate::INVENTORY
            .iter()
            .find(|c| c.id == SUBJECTS[0])
            .expect("`tree` is in the freeze");
        assert_eq!(component.families[0].module(), Some("collect"));
        assert!(component.families[0].members().contains(&SUBJECTS[0]));
        assert_eq!(DECLARATIONS[0].0, "collect.rs");

        // The two axes §21's own rows claim for `tree`, and the freeze sets both.
        assert!(component.declares(crate::Axis::Scrolled));
        assert!(component.declares(crate::Axis::Shrunk));
    }
}
