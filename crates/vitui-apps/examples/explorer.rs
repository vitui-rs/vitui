//! A workspace of 262 145 nodes, folded and unfolded through a caller-owned flatten index.
//!
//! Components ticket 17's application, and the fifth in this crate. It is the first thing to put a
//! [`tree`] anywhere, and — like `ledger` one ticket before it — it exists because **the surface's
//! only consumer is an application**: four of this crate's five files have found a defect its own
//! gates could not see, and every one of them was found by drawing something the gates draw at
//! coordinates the gates never use.
//!
//! ```text
//!   the forest            the caller's data: a pre-order depth array, never iterated in a frame
//!     |
//!   the flatten index     `Order` — one `Entry` a *visible* row, eight bytes, spliced on the edit
//!     |
//!   tree(cx, …, &index)   two verbs a row and a one-slot request; it may not edit the index
//!     |
//!   st.ask.drain()        the caller answers, after the draw, with `Order::fold` / `unfold`
//! ```
//!
//! # What it demonstrates, and every claim of §7 is a key you can press
//!
//! | key | what it shows |
//! |---|---|
//! | `↑` `↓` `PgUp` `PgDn` `Home` `End` | the row axis, which is `collection`'s and not the tree's |
//! | `←` `→` | fold and unfold — **a request**, drained after the draw, spliced by the caller |
//! | click a chevron | the same request from the pointer, on the press edge |
//! | `Space` | the selection, which survives a fold **because the fold is an interval** |
//! | `f` | fold every crate: 262 145 rows out in one pass |
//! | `u` | unfold everything |
//! | `d` | draw through `collect::defective::unclamped_indent` — §7's negative case, live |
//! | `v` | the live counters, through a `Tally` |
//! | `q` | quit |
//!
//! **The selection is the one to watch.** Select a range with `Space` inside a subtree, fold the
//! crate above it, and the status bar's `sel` does not change: a subtree is contiguous in pre-order
//! display coordinates, so the edit is a [`Splice`] and the caller reconciles it exactly under
//! [`Policy::Drop`]. Sort the same rows instead and the answer is a shattered span list — which is
//! §10's table and is why `Clear` is a permutation's honest default.
//!
//! **`d` is the whole of §7's trap, on a screen.** The deep branch is a module chain 200 levels
//! down; at eighty columns an unclamped indent has eaten the row long before it gets there. Press
//! `v` and then `d` and watch it: the cells written do not move, the distinct cells do not move,
//! and the **verbs go down**. The one number that moves is `asked`, and the `Tally`'s own version of
//! it saturates at 65 535 a verb — so even the instrument that can see it under-reports.
//!
//! # What it cannot say, and it is the surface's to fix
//!
//! 1. **There are no indent guides.** `INVENTORY` declares `VLine`, `TeeLeft` and `BottomLeft` for
//!    `tree` and the component draws none of them: a guide column at depth *d* is a fact about *d*
//!    ancestors, so a correct guide run is either data-proportional per row or a fifth field on a
//!    record §7 fixes at eight bytes. Filed rather than faked — see this ticket's answer.
//! 2. **A fold cannot be animated.** §8's `Collapse` is components ticket 22's, and until it lands a
//!    fold is one frame.
//!
//! # Run it
//!
//! ```text
//! cargo run -p vitui-apps --example explorer
//! ```
//!
//! [`tree`]: vitui_components::collect::tree
//! [`Splice`]: vitui_components::order::Splice
//! [`Policy::Drop`]: vitui_components::order::Policy::Drop

use vitui_components::collect::{
    CollOpts, Node, TreeOpts, TreeState, defective as coll_defective, tree_into,
};
use vitui_components::counters::Tally;
use vitui_components::frame::{Face, face_paint};
use vitui_components::ink::{Direct, Ink};
use vitui_components::order::{Ask, Entry, Order, Policy, Splice, reconcile_splice};
use vitui_components::structure::{PanelOpts, panel_with};
use vitui_components::text::{FitOpts, Justify, fit_with};
use vitui_runtime::ctx::Driver;
use vitui_runtime::keys::{ActionId, Chord, KeyMap};
use vitui_runtime::layout::rect;
use vitui_runtime::work::Wake;
use vitui_runtime::{Ctx, Rect, Role, Themes};

// ── the caller's data ────────────────────────────────────────────────────────────────────────────

/// How many crates the workspace holds.
const CRATES: usize = 64;

/// How many modules each crate holds.
const MODULES: usize = 64;

/// How many items each module holds.
const ITEMS: usize = 62;

/// How deep the one deliberately deep branch goes.
///
/// **Two hundred levels is four hundred indent columns**, which is past any terminal — so `d`'s
/// negative case has something to be about. §7's own scene is at 59 999 and needs no screen.
const DEEP: u16 = 200;

/// **The caller's forest: a pre-order depth array and a name a node.**
///
/// Never iterated in a frame — the frame reads the *index*, and the index is `O(1)` per row. This
/// is the data materialising it is proportional to, which is why materialising happens on the edit.
struct Forest {
    depth: Vec<u16>,
    name: Vec<String>,
}

impl Forest {
    /// A workspace, plus one branch two hundred levels down.
    fn workspace() -> Forest {
        let mut depth = Vec::new();
        let mut name = Vec::new();
        for c in 0..CRATES {
            depth.push(0);
            name.push(format!("vitui-crate-{c:02}"));
            for m in 0..MODULES {
                depth.push(1);
                name.push(format!("mod {}", MODULE_NAMES[m % MODULE_NAMES.len()]));
                for i in 0..ITEMS {
                    depth.push(2);
                    name.push(format!("fn {}_{i:02}", ITEM_NAMES[i % ITEM_NAMES.len()]));
                }
            }
        }
        // **The deep branch**, so `d` has a row whose indent can eat it.
        depth.push(0);
        name.push(format!("vitui-deep ({DEEP} levels)"));
        for level in 1..=DEEP {
            depth.push(level);
            name.push(format!("mod level_{level:03}"));
        }
        Forest { depth, name }
    }

    /// How many nodes it holds.
    fn len(&self) -> usize {
        self.depth.len()
    }

    /// **How many nodes hang under `i`, walked in the data.** The expensive half of §7's sentence,
    /// and it is here rather than in the frame: this is what the index exists so a frame never does.
    fn descendants(&self, i: usize) -> usize {
        let mine = self.depth[i];
        let mut j = i + 1;
        while j < self.depth.len() && self.depth[j] > mine {
            j += 1;
        }
        j - i - 1
    }

    /// One index row for node `i`, carrying [`Entry::FOLDED`] when the caller has it folded.
    fn entry(&self, i: usize, folded: &[u32]) -> Entry {
        let node = u32::try_from(i).expect("a workspace that fits a caller's key");
        let mut e = Entry::of(node).at_depth(self.depth[i]);
        if folded.binary_search(&node).is_ok() {
            e.flags |= Entry::FOLDED;
        }
        e
    }

    /// **The whole index**, honouring the fold set. Proportional to the data, and done on the edit.
    fn flattened(&self, folded: &[u32]) -> Order {
        Order::built(self.rows(0..self.len(), folded))
    }

    /// The rows of `range`, skipping the subtree of anything folded.
    fn rows(&self, range: std::ops::Range<usize>, folded: &[u32]) -> Vec<Entry> {
        let mut out = Vec::new();
        let mut i = range.start;
        while i < range.end {
            out.push(self.entry(i, folded));
            let key = u32::try_from(i).expect("fits");
            i += match folded.binary_search(&key) {
                Ok(_) => 1 + self.descendants(i),
                Err(_) => 1,
            };
        }
        out
    }
}

/// Module names, cycled. A workspace's own vocabulary, so a truncated label reads like one.
const MODULE_NAMES: [&str; 8] = [
    "compositor",
    "serializer",
    "damage_map",
    "cluster_table",
    "layout_solver",
    "focus_ring",
    "hit_index",
    "theme_registry",
];

/// Item names, cycled.
const ITEM_NAMES: [&str; 8] = [
    "resolve",
    "reconcile",
    "measure",
    "advance",
    "collapse",
    "publish",
    "settle",
    "quantise",
];

// ── the application ──────────────────────────────────────────────────────────────────────────────

const QUIT: ActionId = 1;
const FOLD_ALL: ActionId = 2;
const UNFOLD_ALL: ActionId = 3;
const COUNTERS: ActionId = 4;
const DEFECT: ActionId = 5;

/// The bindings, built once.
fn key_map() -> KeyMap {
    KeyMap::new()
        .bind(&[Chord::key('q')], QUIT, "Quit")
        .bind(&[Chord::key('f')], FOLD_ALL, "Fold every crate")
        .bind(&[Chord::key('u')], UNFOLD_ALL, "Unfold everything")
        .bind(&[Chord::key('v')], COUNTERS, "Live counters")
        .bind(&[Chord::key('d')], DEFECT, "Unclamped indent")
}

/// Everything the application knows.
struct App {
    forest: Forest,
    /// **The flatten index. The caller's, and the component may only ask.**
    index: Order,
    /// The fold set, sorted. Caller state, which is §8's own table: *rows in a caller-owned index
    /// are collapsed by the caller, on request*.
    folded: Vec<u32>,
    tree: TreeState,
    /// What the last drained request was, printed so the loop is visible.
    last: String,
    /// Whether to draw through a [`Tally`].
    counters: bool,
    /// Whether to draw through [`coll_defective::unclamped_indent`].
    defect: bool,
    /// `(writes, distinct, verbs, asked)` from the last counted frame.
    counted: (u64, u64, u64, u64),
    exit: bool,
}

impl App {
    fn new() -> App {
        let forest = Forest::workspace();
        // Start with every crate folded, so the first screen is the workspace rather than 262 145
        // rows of items — and so the first `→` is a splice with something to insert.
        let mut folded: Vec<u32> = (0..forest.len())
            .filter(|i| forest.depth[*i] == 0)
            .map(|i| u32::try_from(i).expect("fits"))
            .collect();
        folded.sort_unstable();
        let index = forest.flattened(&folded);
        App {
            forest,
            index,
            folded,
            tree: TreeState::new(),
            last: String::from("nothing yet"),
            counters: false,
            defect: false,
            counted: (0, 0, 0, 0),
            exit: false,
        }
    }

    /// One frame, top to bottom.
    fn ui(&mut self, cx: &mut Ctx<'_, '_>, map: &KeyMap) {
        cx.key_map(map);
        let block = panel_with(
            cx,
            cx.area(),
            &format!(
                " Workspace — {} nodes, {} rows in the index, {} folded ",
                self.forest.len(),
                self.index.len(),
                self.folded.len()
            ),
            &PanelOpts {
                padded: false,
                ..Default::default()
            },
        );
        let (body, status) = rect::split_at_v(block.interior, block.interior.h.saturating_sub(1));

        let id = self.draw_tree(cx, body);
        self.draw_status(cx, status);

        // **Give it the keyboard while nobody has it** — architecture issue 25, and `counter.rs`
        // says at length why it is `focused().is_none()` and not `!is_focused(id)`.
        if cx.focused().is_none() {
            cx.focus(id);
        }
        // The application's own keys, taken **after** the component has drained what is its. A key
        // the tree declined is still on the queue; one it took is gone, which is the contract.
        while let Some(key) = cx.next_key(id) {
            match cx.action(&key) {
                Some(QUIT) => self.exit = true,
                Some(FOLD_ALL) => self.fold_all(),
                Some(UNFOLD_ALL) => self.unfold_all(),
                Some(COUNTERS) => self.counters = !self.counters,
                Some(DEFECT) => self.defect = !self.defect,
                _ => cx.decline(key),
            }
        }
    }

    /// The tree, drawn through one of four (ink × indent) pairs — **the same code path**.
    fn draw_tree(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) -> vitui_runtime::Id {
        let opts = TreeOpts {
            coll: CollOpts::default(),
            furniture: Role::Dim,
        };
        // Everything the row drawer reads, taken out of `self` before the closure: `tree_into`
        // borrows `self.tree` and `self.index` for the whole draw.
        let names = &self.forest.name;
        let mut find = |_: &str, _: std::ops::Range<usize>| None;

        let resp = match (self.counters, self.defect) {
            (true, false) => {
                let mut tally = Tally::new();
                let resp = tree_into(
                    &mut tally,
                    cx,
                    area,
                    &mut self.tree,
                    &opts,
                    &self.index,
                    &mut find,
                    |ink: &mut Tally, cx: &mut Ctx<'_, '_>, r, n, f| label(ink, cx, r, n, f, names),
                );
                self.counted = (
                    tally.writes(),
                    tally.distinct(),
                    tally.verbs(),
                    tally.asked(),
                );
                resp
            }
            (true, true) => {
                let mut tally = Tally::new();
                let resp = coll_defective::unclamped_indent(
                    &mut tally,
                    cx,
                    area,
                    &mut self.tree,
                    &opts,
                    &self.index,
                    &mut find,
                    |ink: &mut Tally, cx: &mut Ctx<'_, '_>, r, n, f| label(ink, cx, r, n, f, names),
                );
                self.counted = (
                    tally.writes(),
                    tally.distinct(),
                    tally.verbs(),
                    tally.asked(),
                );
                resp
            }
            (false, false) => {
                self.counted = (0, 0, 0, 0);
                tree_into(
                    &mut Direct,
                    cx,
                    area,
                    &mut self.tree,
                    &opts,
                    &self.index,
                    &mut find,
                    |ink: &mut Direct, cx: &mut Ctx<'_, '_>, r, n, f| {
                        label(ink, cx, r, n, f, names)
                    },
                )
            }
            (false, true) => {
                self.counted = (0, 0, 0, 0);
                coll_defective::unclamped_indent(
                    &mut Direct,
                    cx,
                    area,
                    &mut self.tree,
                    &opts,
                    &self.index,
                    &mut find,
                    |ink: &mut Direct, cx: &mut Ctx<'_, '_>, r, n, f| {
                        label(ink, cx, r, n, f, names)
                    },
                )
            }
        };
        resp.id
    }

    /// The cursor, the selection, the last request, and the counters when they are on.
    fn draw_status(&mut self, cx: &mut Ctx<'_, '_>, area: Rect) {
        let (writes, distinct, verbs, asked) = self.counted;
        let counters = match self.counters {
            false => String::from("counters off (v)"),
            true => {
                format!("{writes} written / {distinct} distinct / {verbs} verbs / {asked} asked")
            }
        };
        let line = format!(
            " {}  cursor {}  sel {} in {} spans  |  {}  |  {}  |  \
             ←→ fold  Space select  f fold all  u unfold all  d indent  q quit ",
            match self.defect {
                true => "UNCLAMPED INDENT",
                false => "clamped",
            },
            self.tree.coll.sel.lead,
            self.tree.coll.sel.count(),
            self.tree.coll.sel.span_count(),
            self.last,
            counters,
        );
        fit_with(
            cx,
            area,
            &line,
            &FitOpts {
                role: Role::Dim,
                justify: Justify::Start,
                ..Default::default()
            },
        );
    }

    /// **The caller's half of the loop: drain the request, splice the index, carry the positions.**
    ///
    /// Run **after** the draw, when nothing borrows the index — which is not a convention: a body
    /// holding the index shared cannot also hand out `&mut`, and `order::Asked`'s own documentation
    /// carries that as an `E0502` beside its positive twin.
    fn answer(&mut self) {
        let Some(ask) = self.tree.ask.drain() else {
            return;
        };
        match ask {
            Ask::Collapse(node) => {
                let key = node as u32;
                let Some(at) = self.index.position_of(key) else {
                    return;
                };
                let edit = self.index.fold(at);
                self.remember(key);
                self.reconcile(&edit);
                self.last = format!("folded {key}: {} rows out", edit.removed.len());
            }
            Ask::Expand(node) => {
                let key = node as u32;
                let Some(at) = self.index.position_of(key) else {
                    return;
                };
                self.forget(key);
                let root = key as usize;
                let rows = self.forest.rows(
                    root + 1..root + 1 + self.forest.descendants(root),
                    &self.folded,
                );
                let edit = self.index.unfold(at, rows);
                self.reconcile(&edit);
                self.last = format!("unfolded {key}: {} rows in", edit.inserted);
            }
            // Neither is this application's: it does not sort and it does not filter.
            Ask::Sort(_) | Ask::Filter(_) => {}
        }
    }

    /// **`Drop`, and the caller says it carried the positions across.**
    ///
    /// The third of ADR 0031's three things. Without the `reconciled` call the next frame sees a
    /// revision it does not recognise and clears every position the component holds — which is
    /// correct for an edit nobody explained and wrong for one the caller performed.
    fn reconcile(&mut self, edit: &Splice) {
        let _ = reconcile_splice(&mut self.tree.coll.sel, edit, Policy::Drop);
        self.tree.coll.reconciled(self.index.rows());
        let last = self.index.len().saturating_sub(1);
        self.tree.coll.sel.lead = self.tree.coll.sel.lead.min(last);
    }

    fn remember(&mut self, key: u32) {
        if let Err(at) = self.folded.binary_search(&key) {
            self.folded.insert(at, key);
        }
    }

    fn forget(&mut self, key: u32) {
        if let Ok(at) = self.folded.binary_search(&key) {
            self.folded.remove(at);
        }
    }

    /// Fold every crate, in one rebuild rather than one splice a crate.
    ///
    /// **The one place a rebuild is the right answer**, and it is the crossover §7 names: this
    /// removes 99.9% of the index, which is the far side of it.
    fn fold_all(&mut self) {
        let before = self.index.len();
        self.folded = (0..self.forest.len())
            .filter(|i| self.forest.depth[*i] == 0)
            .map(|i| u32::try_from(i).expect("fits"))
            .collect();
        self.rebuild();
        self.last = format!("folded every crate: {} rows out", before - self.index.len());
    }

    fn unfold_all(&mut self) {
        let before = self.index.len();
        self.folded.clear();
        self.rebuild();
        self.last = format!("unfolded everything: {} rows in", self.index.len() - before);
    }

    /// A whole-index rebuild, which is a **permutation** as far as a component can tell — so the
    /// positions go, and `Policy::Clear` is the honest default rather than a limitation.
    fn rebuild(&mut self) {
        self.index = self.forest.flattened(&self.folded);
        self.tree.coll.sel.clear();
        self.tree.coll.sel.anchor = None;
        self.tree.coll.offset = 0;
        self.tree.coll.sel.lead = 0;
        self.tree.coll.reconciled(self.index.rows());
    }
}

/// One row's label: the part of the rectangle the component did not write.
///
/// **Through the ink it was handed and never through `cx.text`.** `ledger` got that wrong on its
/// first build and the counters read 78 cells on a frame that wrote 1 560 — on a screen that looked
/// perfectly correct.
fn label<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>, r: Rect, n: Node, f: Face, names: &[String]) {
    if r.w == 0 {
        return;
    }
    let paint = face_paint(cx.theme(), f);
    let text = names
        .get(n.node as usize)
        .map_or("—", std::string::String::as_str);
    let written = ink.text(
        cx,
        r.x,
        r.y,
        vitui_runtime::layout::text::truncate(text, r.w),
        paint,
    );
    let _ = ink.run(
        cx,
        r.x + i32::from(written),
        r.y,
        " ",
        r.w.saturating_sub(written),
        paint,
    );
}

fn main() {
    let map = key_map();
    let mut app = App::new();

    let mut driver = match Driver::attach(Default::default(), *Themes::standard().theme()) {
        Ok(driver) => driver,
        Err(why) => {
            eprintln!("vitui could not attach: {why}");
            return;
        }
    };

    driver.frame(|cx| app.ui(cx, &map));
    app.answer();

    while !app.exit {
        match driver.wait() {
            Wake::Quit => break,
            Wake::Input | Wake::Posted | Wake::Deadline => {}
        }
        driver.frame(|cx| app.ui(cx, &map));
        // **After the draw, and this is the whole of *a component may only ask*.**
        app.answer();
    }
}
