//! **One screen, written once, so that "the same screen three ways" is a fact about the source.**
//!
//! Every gate and report in this crate draws *this* screen. The three drivers in [`crate::board`]'s
//! callers differ in exactly one axis — **where the state lives and when it is written** — and the
//! drawing half is shared so that nothing else can drift.
//!
//! `#[path]`-included by the tests and the example rather than exported, which is `vitui-runtime`'s
//! own arrangement for `src/screen.rs` and the engine's for its scene list, and for the same reason:
//! **an instrument is not part of the library.** Putting a fixture on the public surface would put
//! it in front of the crate's users.
//!
//! ```text
//! #[path = "../src/board.rs"]
//! mod board;
//! ```
//!
//! # The shape, and why it is a count
//!
//! [`REGIONS`] interactive regions in three groups of 7, 12 and 8, then 40 loose tab stops, then 245
//! plain hover targets: **312 hit entries and 43 tab stops**, because a group collapses its members
//! onto one stop and 27 entries become 3. That is runtime spec §20's dense IDE screen and runtime
//! ticket 12's `focus_numbers` shape, and it is the pair of numbers this whole ticket turns on.
//!
//! Over the top of it: a header, twenty-four list rows and a footer carrying the count, the
//! memoised aggregate and the selection. Those are the cells that make the *picture* comparable.
//!
//! # Two things this fixture cannot do, and one of them is the ticket's headline
//!
//! **No cell is readable from outside the engine** (ADR 0023), and this crate does not depend on the
//! engine anyway. So the cell equality is [`Ink`] — *the application's own ledger of its own writes*
//! — and it is stated that way rather than implied to be a read-back of the composited screen. The
//! runtime's `tests/swap.rs` reached the identical arrangement one crate down, for the identical
//! reason, and said so in the same words.
//!
//! **No event can be posted.** `Driver::post_mouse` takes a `vitui_engine::Mouse` and
//! `Driver::post_key` a `vitui_engine::Key`; neither type is nameable here. That is runtime ticket
//! 17's finding — *six of sixteen gate families do not cross, and one fact covers all six* — arriving
//! in a crate that was written after it. So the edge a component would produce from a click is
//! handed to [`draw`] as a [`Press`] instead, applied at the point in the draw where the component
//! would have written its own state. **The axis under test is untouched**: what is being compared is
//! where the state lives and when it is written, not how a byte became a `Mouse`.

use vitui_runtime::data::{Revision, Versioned};
use vitui_runtime::focus::ScopeKind;
use vitui_runtime::layout::rect;
use vitui_runtime::{Ctx, Id, Interest, Role};

/// The terminal this screen is drawn on. 300 × 80 is runtime spec §20's dense IDE screen.
pub const W: u16 = 300;
/// See [`W`].
pub const H: u16 = 80;
/// Cells on it: **24 000**, the denominator of every equality in this crate.
pub const CELLS: usize = W as usize * H as usize;

/// Interactive regions the full draw declares: [`BODY`] plus the footer's one.
pub const REGIONS: usize = 312;
/// Tab stops the full draw declares: 40 loose ones plus one for each of three groups.
pub const STOPS: usize = 43;
/// The three groups, and how many entries each collapses.
pub const GROUPS: [(&str, u64); 3] = [("menu bar", 7), ("toolbar", 12), ("tab strip", 8)];
/// Loose tab stops, outside any group.
pub const LOOSE: u64 = 40;
/// Regions that are hover targets and nothing more.
pub const PLAIN: u64 = 244;
/// What [`declare`] alone declares: 27 grouped, 40 loose, 244 plain.
///
/// **311 and not 312, and the one it is short of is the footer's.** The whole ticket turns on a
/// frame that redraws *one region*, so that region has to be interactive — a footer carrying a count
/// nobody can click is a label, and a subscription that redraws a label declares nothing at all,
/// which would make the gate read `0` and say nothing.
pub const BODY: u64 = 311;

/// Rows in the sidebar list.
pub const ROWS: usize = 24;

/// What the sidebar rows are labelled with. Latin and short — the same reason `vitui-runtime`'s
/// `screen.rs` gives for having two corpora: a frame drawn out of stacked combining marks and family
/// emoji is a measurement of UAX #29 rather than of a frame.
pub const LABELS: [&str; 8] = [
    "src/main.rs",
    "modified",
    "crates/vitui-signals/src/lib.rs",
    "ok",
    "build",
    "a moderately long label",
    "target/release/deps",
    "done",
];

// ── the ledger ───────────────────────────────────────────────────────────────────────────────────

/// One cell of [`Ink`]: which role painted it, which string, and which column of that string.
///
/// Not the cell the engine holds — that one is unreachable by construction and by policy. This is
/// what the *caller* declared it wrote, which is the strongest thing a crate outside the engine can
/// compare, and the comparison is exact: two drivers that painted the same string at the same place
/// in the same role produce equal marks, and any difference in any of the three is a difference.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Mark {
    /// The role the cell was painted with, as its discriminant.
    pub role: u8,
    /// FNV-1a over the string that painted it. Zero for a fill.
    pub source: u64,
    /// Which column of that string this cell is.
    pub column: u16,
}

/// The application's own ledger of its own writes, one entry a cell.
///
/// **Retained between frames, because the engine's surfaces are.** Nothing clears a surface at frame
/// start — `Ctx::clear` is a verb a caller chooses — so a frame that redraws one region leaves the
/// rest of the screen holding what the previous frame put there. The ledger models that by simply
/// not being reset, which is what makes *cells still correct after a partial redraw* a question with
/// an answer.
pub struct Ink {
    at: Vec<Option<Mark>>,
}

impl Default for Ink {
    fn default() -> Ink {
        Ink::new()
    }
}

impl Ink {
    /// An empty screen: no cell has been painted, which is not the same as a cell painted blank.
    pub fn new() -> Ink {
        Ink {
            at: vec![None; CELLS],
        }
    }

    /// How many of the 24 000 differ from `other`.
    pub fn differing(&self, other: &Ink) -> usize {
        self.at
            .iter()
            .zip(other.at.iter())
            .filter(|(a, b)| a != b)
            .count()
    }

    /// How many of the 24 000 agree with `other`. The complement, spelled out because every report
    /// in this crate states it as *n of 24 000 still correct*.
    pub fn agreeing(&self, other: &Ink) -> usize {
        CELLS - self.differing(other)
    }

    /// What one cell holds, or `None` if nothing has painted it.
    pub fn at(&self, x: i32, y: i32) -> Option<Mark> {
        if x < 0 || y < 0 || x >= i32::from(W) || y >= i32::from(H) {
            return None;
        }
        self.at[y as usize * W as usize + x as usize]
    }

    fn put(&mut self, x: i32, y: i32, mark: Mark) {
        if x < 0 || y < 0 || x >= i32::from(W) || y >= i32::from(H) {
            return;
        }
        let i = y as usize * W as usize + x as usize;
        self.at[i] = Some(mark);
    }

    /// Record `cells` columns of `s` painted at `(x, y)` in `role`.
    pub fn text(&mut self, x: i32, y: i32, s: &str, role: Role, cells: u16) {
        let source = fnv(s.as_bytes());
        for c in 0..cells {
            self.put(
                x + i32::from(c),
                y,
                Mark {
                    role: role_tag(role),
                    source,
                    column: c,
                },
            );
        }
    }

    /// Record one filled cell.
    pub fn fill(&mut self, x: i32, y: i32, role: Role) {
        self.put(
            x,
            y,
            Mark {
                role: role_tag(role),
                source: 0,
                column: 0,
            },
        );
    }
}

/// A role as a byte, taken the one way a consumer can take it.
///
/// `Role::index` is private and the enum has no numeric cast on its public surface, so the tag is
/// the position in [`Role::ALL`] — which is `pub`, is what the theme's own pair gate iterates, and is
/// **exact rather than folded**: thirteen roles into thirteen values, no collision to argue about.
/// `tests/drivers.rs` gates that it is a bijection.
pub fn role_tag(role: Role) -> u8 {
    let i = Role::ALL
        .iter()
        .position(|r| *r == role)
        .expect("Role::ALL is every role");
    u8::try_from(i).unwrap_or(u8::MAX)
}

/// FNV-1a, the runtime's own hash, so that a source string folds the same way everywhere here.
pub fn fnv(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= u64::from(*b);
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

// ── the state, and the edge a component produces from inside the draw ────────────────────────────

/// What the screen is a function of. **`Copy` and `PartialEq`** — the components map's constraint,
/// and this crate's documentation is where what depends on it is written down.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Board {
    /// Which sidebar row is selected. The component writes this **during the draw**.
    pub selected: u32,
    /// The footer counter.
    pub count: i32,
    /// Whether the screen is drawn dense. Changes what is painted, not only how.
    pub dense: bool,
}

/// The interaction, handed in rather than posted.
///
/// See the module documentation: `Driver::post_mouse` takes an engine type this crate cannot name,
/// so a `Press` stands where the click would be, applied at the point in [`draw`] where the
/// component would have written its own state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Press {
    /// Nothing happened this frame.
    #[default]
    Quiet,
    /// The list moved the selection to a row.
    Select(u32),
    /// A footer button was clicked.
    Bump(i32),
    /// The density button was clicked.
    Toggle,
}

/// What came back out of the draw.
///
/// **Not a message type and not a signal**: it is the return value of the draw, which is the only
/// channel this runtime has, because a `Response` is returned at the call site and nothing is
/// dispatched.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Edges {
    /// What the list wrote into the scratch copy it was handed.
    pub selected: u32,
    /// The footer's net click.
    pub bump: i32,
    /// Whether density was toggled.
    pub toggled: bool,
}

impl Edges {
    /// Whether the draw produced nothing that would move `before`.
    pub fn quiet(&self, before: &Board) -> bool {
        self.bump == 0 && !self.toggled && self.selected == before.selected
    }
}

/// The data the footer's aggregate is folded out of, with the revision a [`crate::Computed`] keys on.
pub struct Data {
    /// One reading a row, over enough rows that folding them is worth not doing twice.
    pub cpu: Versioned<Vec<f32>>,
}

impl Default for Data {
    fn default() -> Data {
        Data::new(600)
    }
}

impl Data {
    /// `n` readings, deterministic, and none of them equal.
    pub fn new(n: usize) -> Data {
        let cpu = (0..n).map(|i| (i % 977) as f32 * 0.1).collect::<Vec<_>>();
        Data {
            cpu: Versioned::new(cpu),
        }
    }

    /// The revision a memo keys on.
    pub fn revision(&self) -> Revision {
        self.cpu.revision()
    }

    /// The fold: **the one value on this screen worth not recomputing**, and the 132× is what
    /// skipping it is worth.
    pub fn max(&self) -> f32 {
        self.cpu.iter().copied().fold(0.0, f32::max)
    }
}

// ── the drawing half, shared by all three drivers ────────────────────────────────────────────────

/// Draw the whole screen.
///
/// `scratch` is the list's own state, handed to it as a copy by a driver that does not own a `&mut`
/// to the real one — which is every driver but the direct one. The list writes it **during the
/// draw**, and the difference becomes an edge. That is copy-and-diff, and it is forced rather than
/// chosen: the alternative is a component library written twice, once for callers who own their
/// state and once for callers who do not.
pub fn draw(
    cx: &mut Ctx<'_, '_>,
    ink: &mut Ink,
    b: &Board,
    max: f32,
    press: Press,
    scratch: &mut u32,
    out: &mut Edges,
) {
    declare(cx, ink, b.dense);

    let title = Role::Title;
    let tp = cx.theme().paint(title);
    let w = cx.text(0, 0, "vitui — signals", tp).cells;
    ink.text(0, 0, "vitui — signals", title, w);

    // The list. It is handed a scratch copy and writes it, exactly as a real one would.
    *scratch = b.selected;
    if let Press::Select(row) = press {
        *scratch = row;
    }
    let body = Role::Body;
    let dim = Role::Dim;
    for row in 0..ROWS {
        let role = if row as u32 == *scratch { body } else { dim };
        let p = cx.theme().paint(role);
        let label = LABELS[row % LABELS.len()];
        let y = 2 + row as i32;
        let w = cx.text(0, y, label, p).cells;
        ink.text(0, y, label, role, w);
    }
    out.selected = *scratch;

    match press {
        Press::Bump(d) => out.bump += d,
        Press::Toggle => out.toggled = true,
        Press::Select(_) | Press::Quiet => {}
    }

    footer(cx, ink, b, max);
}

/// The footer, drawn on its own so that a fine-grained subscription has one region to redraw.
///
/// **This is the region `tests/declaration.rs` redraws alone**, and it is the whole of what a signal
/// on `count` would claim to reach. It declares **one** interactive region and **no** tab stop —
/// a hover target with a click, which is what the numbers 1 and 0 in that file are counting.
pub fn footer(cx: &mut Ctx<'_, '_>, ink: &mut Ink, b: &Board, max: f32) {
    let role = Role::Body;
    let p = cx.theme().paint(role);
    let y = i32::from(H) - 1;
    let text = format!(
        "count {:>4}   max {max:>5.1}%   sel {:>4}",
        b.count, b.selected
    );
    let w = cx.text(0, y, &text, p).cells;
    ink.text(0, y, &text, role, w);

    let area = cx.area();
    let strip = rect::shrink(area, 0, H - 1, 0, 0);
    let interest = cx.theme().hover_interest().with(Interest::CLICK);
    cx.interact(Id::named("footer"), strip, interest);
}

/// The 312 regions: three groups, forty loose stops, two hundred and forty-five hover targets.
///
/// The rectangles are one cell each and are cut out of `cx.area()` with [`rect::shrink`], because a
/// crate that cannot name `vitui_engine::Rect` cannot write a helper that returns one — runtime
/// ticket 17's finding again, met here as an ergonomic rather than as a wall.
pub fn declare(cx: &mut Ctx<'_, '_>, ink: &mut Ink, dense: bool) {
    let area = cx.area();
    let face = if dense { Role::Face } else { Role::Border };
    let paint = cx.theme().paint(face);
    let base = cx.theme().hover_interest().with(Interest::CLICK);
    let stop = base.with(Interest::FOCUS);
    let cell = |i: u64| {
        let x = u16::try_from(i % u64::from(W)).unwrap_or(0);
        let y = u16::try_from(i / u64::from(W)).unwrap_or(0);
        rect::shrink(area, x, y, W - 1 - x, H - 1 - y)
    };

    let mut n = 0u64;
    for (name, count) in GROUPS {
        cx.scope(Id::named(name), ScopeKind::Group, |cx| {
            for i in 0..count {
                cx.interact(Id::keyed(Id::named(name), i), cell(n + i), stop);
            }
        });
        n += count;
    }
    for i in 0..LOOSE {
        cx.interact(Id::keyed(Id::named("stop"), i), cell(n + i), stop);
    }
    n += LOOSE;
    for i in 0..PLAIN {
        cx.interact(Id::keyed(Id::named("row"), i), cell(n + i), base);
    }

    // Painted after they are declared, and one cell each: the picture and the declaration are the
    // two halves this ticket is about, and they are drawn from the same loop so that a partial frame
    // cannot quietly keep one and drop the other.
    for i in 0..BODY {
        let r = cell(i);
        cx.fill(r, " ", paint);
        ink.fill(r.x, r.y, face);
    }
}

// ── the three drivers ────────────────────────────────────────────────────────────────────────────
//
// One screen, three places to keep the state it is a function of. **The direct driver is the
// control**, and without it neither reactive shape means anything — *TEA works* is not a result
// unless something says what working costs.

/// **Driver A — direct.** State is the application's, read where it is read and written where it is
/// written, on the frame the edge arrived. No queue, no graph, no dirty bit.
///
/// This is the idiom the runtime was designed around, and it is here as the control.
#[derive(Default)]
pub struct Direct {
    /// The whole of the state.
    pub board: Board,
    /// The aggregate's cache. A `Memo` is *storage*, keyed by where it is stored.
    pub max: vitui_runtime::data::Memo<f32>,
    scratch: u32,
}

impl Direct {
    /// One frame's body. Returns whether anything moved, which is what a loop would need.
    pub fn view(&mut self, cx: &mut Ctx<'_, '_>, ink: &mut Ink, d: &Data, press: Press) -> bool {
        let max = *self.max.get(d.revision(), || d.max());
        let mut out = Edges::default();
        draw(
            cx,
            ink,
            &self.board,
            max,
            press,
            &mut self.scratch,
            &mut out,
        );
        let quiet = out.quiet(&self.board);
        // The write happens here, on the same frame, at the call site. That is the whole of it.
        self.board.selected = out.selected;
        self.board.count += out.bump;
        if out.toggled {
            self.board.dense = !self.board.dense;
        }
        footer(cx, ink, &self.board, max);
        !quiet
    }
}

/// A message. **Carrying the whole state and not a delta** for `Selected` — the shape the runtime
/// forces, because the component wrote a scratch copy rather than describing what it did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Msg {
    /// The list wrote itself.
    Selected(u32),
    /// The footer's net click.
    Bump(i32),
    /// The density button.
    ToggleDensity,
}

/// The model. TEA's, and it holds nothing else.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Model {
    /// The board.
    pub board: Board,
}

/// The reducer. **Pure, total, and it names no runtime type at all** — not `Ctx`, not `Frame`, not
/// `Response`, not `Id`. That is the property the boundary is really claiming, and it survives.
pub fn update(m: &mut Model, msg: Msg) {
    match msg {
        Msg::Selected(row) => m.board.selected = row,
        Msg::Bump(d) => m.board.count += d,
        Msg::ToggleDensity => m.board.dense = !m.board.dense,
    }
}

/// **Driver B — TEA.** A pure reducer, a message queue, and a view that may not write.
///
/// The memo is the first thing TEA cannot put where it wants it. A `Memo` is storage, so it is
/// neither model — `update` would have to be handed it and stop being pure — nor view-local, because
/// a view is a function that returns. It lives on the pump, beside the model and outside it, which
/// is the same place a command runner would live.
pub struct Tea {
    /// The model.
    pub model: Model,
    /// What the view emitted this frame.
    pub queue: Vec<Msg>,
    /// The aggregate's cache; see the type documentation.
    pub max: vitui_runtime::data::Memo<f32>,
    /// How many messages this pump has applied. A count, for the gates.
    pub applied: u32,
    scratch: u32,
}

impl Default for Tea {
    fn default() -> Tea {
        Tea {
            model: Model::default(),
            queue: Vec::with_capacity(8),
            max: vitui_runtime::data::Memo::new(),
            applied: 0,
            scratch: 0,
        }
    }
}

impl Tea {
    /// The view. Takes `&Model` — shared, as TEA requires — and emits messages.
    ///
    /// **Copy-and-diff, forced rather than chosen.** The list writes its state during the draw, a
    /// `&Model` cannot be written, so the component is handed a scratch copy and the difference
    /// becomes a message. There is no third option that keeps `update` pure, and no runtime change
    /// would remove it.
    pub fn view(&mut self, cx: &mut Ctx<'_, '_>, ink: &mut Ink, d: &Data, press: Press) {
        let max = *self.max.get(d.revision(), || d.max());
        let mut out = Edges::default();
        let b = self.model.board;
        draw(cx, ink, &b, max, press, &mut self.scratch, &mut out);
        if out.selected != b.selected {
            self.queue.push(Msg::Selected(out.selected));
        }
        if out.bump != 0 {
            self.queue.push(Msg::Bump(out.bump));
        }
        if out.toggled {
            self.queue.push(Msg::ToggleDensity);
        }
        // **The footer draws from the pre-message model**, which is TEA's own asymmetry and not the
        // runtime's: a view draws the whole screen from one model, so anything drawn after the
        // reducer is still reading the borrow the reducer wants exclusively. This is why TEA shows a
        // new value one frame later than the other two.
        footer(cx, ink, &self.model.board, max);
    }

    /// The pump half. Returns whether the model moved.
    pub fn pump(&mut self) -> bool {
        let moved = !self.queue.is_empty();
        for msg in self.queue.drain(..) {
            update(&mut self.model, msg);
            self.applied += 1;
        }
        moved
    }

    /// **View and pump, both inside the frame — and the reason is hook 1.**
    ///
    /// `request_frame()` is a method on `Ctx`, and `update` is by definition *after* the view, so a
    /// pump that runs after the frame body returns cannot ask for the frame it has just made
    /// necessary: every `Ctx` has been dropped by then. Draining inside the frame is one of the two
    /// fixes and needs no runtime change — `update` is still pure and still names no runtime type;
    /// what moved is where it is called from. The other fix is a loop that does not sleep while its
    /// own queue is non-empty, and there is no loop here to put it in.
    pub fn view_and_pump(
        &mut self,
        cx: &mut Ctx<'_, '_>,
        ink: &mut Ink,
        d: &Data,
        press: Press,
    ) -> bool {
        self.view(cx, ink, d, press);
        let moved = self.pump();
        if moved {
            cx.request_frame();
        }
        moved
    }
}

/// **Driver C — signals.** Interior mutability, handles that clone, one dirty flag.
///
/// Every field of the board is its own signal, which is the point: a graph holding one signal
/// containing the whole board would be TEA with a `RefCell`.
pub struct Signals {
    /// The graph, and the dirty flag every signal shares with it.
    pub graph: vitui_signals::Graph,
    /// The selection.
    pub selected: vitui_signals::Signal<u32>,
    /// The footer counter.
    pub count: vitui_signals::Signal<i32>,
    /// The density toggle.
    pub dense: vitui_signals::Signal<bool>,
    /// The aggregate — `Memo` with the key filled in.
    pub max: vitui_signals::Computed<f32>,
    scratch: u32,
}

impl Default for Signals {
    fn default() -> Signals {
        Signals::new()
    }
}

impl Signals {
    /// A graph and the three signals the screen is a function of.
    pub fn new() -> Signals {
        let graph = vitui_signals::Graph::new();
        Signals {
            selected: graph.signal(0),
            count: graph.signal(0),
            dense: graph.signal(false),
            max: vitui_signals::Computed::default(),
            scratch: 0,
            graph,
        }
    }

    /// Read the graph back into the shape the drawing half takes.
    ///
    /// Three `Rc` derefs and a copy of twelve bytes — which is what *fine-grained* costs when the
    /// draw is a call tree rather than a set of subscriptions.
    pub fn board(&self) -> Board {
        Board {
            selected: self.selected.get(),
            count: self.count.get(),
            dense: self.dense.get(),
        }
    }

    /// One frame. Returns whether the graph asked for another.
    pub fn flush(&mut self, cx: &mut Ctx<'_, '_>, ink: &mut Ink, d: &Data, press: Press) -> bool {
        let max = self.max.get(d.revision(), || d.max());
        let b = self.board();
        let mut out = Edges::default();
        draw(cx, ink, &b, max, press, &mut self.scratch, &mut out);

        self.selected.set(out.selected);
        if out.bump != 0 {
            self.count.set(b.count + out.bump);
        }
        if out.toggled {
            self.dense.set(!b.dense);
        }

        // Read the graph again *below* the write, which is what interior mutability buys and what
        // TEA refuses on purpose. The footer is therefore current on the same frame.
        footer(cx, ink, &self.board(), max);
        self.graph.settle(cx)
    }

    /// The same frame with the **undiffed** write — the version a signal library ships first.
    ///
    /// Kept, and used by exactly one gate, so that what it costs is a number rather than a warning.
    pub fn flush_undiffed(
        &mut self,
        cx: &mut Ctx<'_, '_>,
        ink: &mut Ink,
        d: &Data,
        press: Press,
    ) -> bool {
        let max = self.max.get(d.revision(), || d.max());
        let b = self.board();
        let mut out = Edges::default();
        draw(cx, ink, &b, max, press, &mut self.scratch, &mut out);

        let got = out.selected;
        self.selected.edit_undiffed(|s| *s = got);
        if out.bump != 0 {
            self.count.edit_undiffed(|c| *c += out.bump);
        }
        if out.toggled {
            self.dense.edit_undiffed(|d| *d = !*d);
        }

        footer(cx, ink, &self.board(), max);
        self.graph.settle(cx)
    }
}
