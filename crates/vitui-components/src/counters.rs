//! The nine per-frame counters, **as a value that says which of them this crate can read**.
//!
//! Every screen here is priced in the same nine columns — `writes`, `distinct cells
//! touched`, `verbs`, `marked`, `regions`, `tab stops`, `merges`, `content layers`, `allocations` —
//! and twelve prototypes each computed them their own way, inside their own binary, on their own
//! screen. This module is the one place they come from, so that a gate written against `merges` and
//! a report printing `merges` are reading the same number.
//!
//! # Eight of the nine are reachable and one is not, and the one is a result rather than a gap
//!
//! `vitui-components` depends on `vitui-runtime` **and nothing else** — the constraint C6,
//! enforced by cargo and by `deny.toml`'s `{ name = "vitui-engine", wrappers = ["vitui-runtime",
//! "vitui"] }`. So a counter is available here only if the runtime hands it over, and one does not:
//!
//! | counter | where it comes from | standing |
//! |---|---|---|
//! | `writes` | the engine's own report — the `cells` field of what `Ctx::text` returns | reachable, for the four verbs that return one |
//! | `distinct cells touched` | [`Tally`]'s union over the spans those verbs reported | reachable, **as a model** |
//! | `verbs` | [`Tally`]'s own call count | reachable |
//! | `marked` | the engine's damage structure | **unreachable** — see [`Counters::marked`] |
//! | `regions` | `Frame::hits().len()` | reachable |
//! | `tab stops` | `Frame::stop_count()` | reachable |
//! | `merges` | `Frame::ids().merges()` | reachable |
//! | `content layers` | `Driver::layers_live()` | reachable |
//! | `allocations` | `vitui-alloc-probe`, as a **total** | reachable |
//!
//! **`marked` is the engine's alone.** `crates/vitui-engine/src/damage.rs` declares every one of its
//! methods `pub(crate)`, so the damaged-cell count is not on the engine's *public* surface either —
//! this is not a runtime omission that a re-export would fix. `Counters::marked` therefore reads
//! [`Reading::Unreachable`] and naming it aloud is the whole of what this module can do about it.
//!
//! # Why a `Reading` and not an `Option<u64>`
//!
//! An absent counter that arrives as `0` is the failure mode the rules are built out of: *a threshold on
//! the wrong side of the question is not a weak gate, it is a green one*, and `marked == 0` is a
//! gate that passes trivially when nothing is counting. So [`Reading::get`] **panics**, with the
//! counter's name and with what would have to become public — the shape
//! [`crate::obligations::Verdict::assert_met`] already uses one file over, and for the same reason:
//! a query that cannot run yet must fail loudly rather than answer.
//!
//! # `writes` is the engine's number and `distinct` is ours, which is what makes the pair a gate
//!
//! `Ctx::text`, `Ctx::set`, `Ctx::blit` and `Ctx::label` return the engine's `Written`, whose `cells`
//! field is **how many columns were actually written** after clipping. [`Tally::text`] folds that in.
//! The union in [`Tally::distinct`] is built by this crate from the same spans. So *writes ==
//! distinct* compares a number the engine produced against a set we produced, and it separates when
//! two verbs overlap — which is the first half, *no cell twice*.
//!
//! **`Ctx::fill`, `Ctx::clear` and `Ctx::restyle` return `()`**, so for those there is no engine
//! report to fold and both sides of the pair would be ours. [`Tally::filled`] therefore takes the
//! rectangle as four scalars and records it as **modelled**, and [`Tally::reported`] is the fraction
//! of `writes` that came from the engine. A gate that wants the pair to mean something asserts
//! `reported == writes` first.
//!
//! Taking four scalars is not an ergonomic choice: `Rect` is named by twenty-seven of the runtime's
//! public declarations and reachable through **none** of them
//! (`vitui_runtime::line::ENGINE_NAMES`), so a `fn filled(&mut self, r: Rect)`
//! in this crate is `error[E0412]: cannot find type` before anything runs.

use std::collections::BTreeSet;

use vitui_runtime::ctx::Driver;
use vitui_runtime::{Ctx, Paint};

/// One of the nine per-frame columns.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Counter {
    /// Columns written, as the engine reported them.
    Writes,
    /// How many distinct cells were touched at all.
    Distinct,
    /// How many drawing calls were made.
    Verbs,
    /// How many cells the frame marked damaged.
    Marked,
    /// How many entries the hit index holds.
    Regions,
    /// How many tab stops the ring declares.
    TabStops,
    /// How many claims landed on an id that had already claimed this frame.
    Merges,
    /// How many overlay layers the census is keeping alive.
    ContentLayers,
    /// How many allocations the run made, **as a total**.
    Allocations,
}

impl Counter {
    /// All nine, in the column order.
    pub const ALL: [Counter; 9] = [
        Counter::Writes,
        Counter::Distinct,
        Counter::Verbs,
        Counter::Marked,
        Counter::Regions,
        Counter::TabStops,
        Counter::Merges,
        Counter::ContentLayers,
        Counter::Allocations,
    ];

    /// The column heading a report prints.
    pub fn word(self) -> &'static str {
        match self {
            Counter::Writes => "writes",
            Counter::Distinct => "distinct cells touched",
            Counter::Verbs => "verbs",
            Counter::Marked => "marked",
            Counter::Regions => "regions",
            Counter::TabStops => "tab stops",
            Counter::Merges => "merges",
            Counter::ContentLayers => "content layers",
            Counter::Allocations => "allocations",
        }
    }
}

/// A counter's value, or the reason this crate cannot have one.
///
/// **Two arms and no third**, which is [`crate::obligations::Verdict`]'s arrangement and its
/// argument: an arm meaning *not measured this time* is what lets a counter report nothing and be
/// read as zero.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reading {
    /// Measured, and this is the number.
    Measured(u64),
    /// Not reachable across the crate line.
    Unreachable {
        /// **Exactly what would have to become public**, named as an item and not as a wish.
        needs: &'static str,
    },
}

impl Reading {
    /// The number, or a panic naming what is missing.
    ///
    /// # Panics
    ///
    /// Panics when the reading is [`Reading::Unreachable`]. That is the point: a counter nobody can
    /// read must not be able to arrive at a comparison as `0`, because every gate on this map is
    /// written in the direction where `0` passes.
    #[track_caller]
    pub fn get(self, counter: Counter) -> u64 {
        match self {
            Reading::Measured(n) => n,
            Reading::Unreachable { needs } => panic!(
                "`{}` is unreachable across the crate line: it needs {needs}",
                counter.word()
            ),
        }
    }

    /// The number, without the panic, for a report that prints a dash instead.
    pub fn measured(self) -> Option<u64> {
        match self {
            Reading::Measured(n) => Some(n),
            Reading::Unreachable { .. } => None,
        }
    }
}

/// **An allocation figure, which is a total over a run and never a mean over `n`.**
///
/// > A mean cannot see anything below `n`; a total can see one.
///
/// Every component prototype reported `allocs / n` with `n` between 40 and 200, so a frame
/// allocating on `n − 1` of `n` frames reported **0**. Run as a total, eleven panels are 0 and one
/// is **40 over 40** — `player::chrome` collecting a `Vec<f32>` on the draw path.
///
/// **This type has no `mean` and that is the encoding.** The rule cannot live in a comment beside a
/// division, because the division is what a reader writes when the number looks large; it lives here,
/// as an accessor that does not exist. [`Allocations::frames`] is kept so a report can say *0 over
/// 200 frames* rather than *0*, which is the honest way to print a total.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Allocations {
    total: u64,
    frames: u32,
}

impl Allocations {
    /// A total, and how many frames it was taken over.
    ///
    /// # Panics
    ///
    /// Panics on zero frames. A total over no frames is the vacuity [`crate::obligations::Verdict`]
    /// refuses in its constructor, arriving in the one place where the answer would be a plausible
    /// `0` rather than an obvious one.
    #[track_caller]
    pub fn over(frames: u32, total: u64) -> Allocations {
        assert!(
            frames > 0,
            "an allocation total over zero frames is not a measurement of anything"
        );
        Allocations { total, frames }
    }

    /// The total. **The only number this type will hand over.**
    pub fn total(self) -> u64 {
        self.total
    }

    /// How many frames the total was taken over, for a report that prints both.
    pub fn frames(self) -> u32 {
        self.frames
    }
}

/// **The draw-side counters, accumulated by the caller because nothing else can accumulate them.**
///
/// A `Tally` is an instrument and not a component-facing API: the shape for a component is
/// `fn button(cx: &mut Ctx, area: Rect, label: &str) -> Response`, and nothing on this map has
/// decided that a component draws through a wrapper. What it is for is the three counters the
/// runtime does not keep — `writes`, `distinct cells touched` and `verbs` — which no frame structure
/// holds because the engine clears its damage inside `present` and the runtime keeps no drawing log.
///
/// # It counts one context's coordinates, and that is the scope anyway
///
/// The spans it unions are in the coordinates of the `Ctx` the verbs were called on. Two sibling
/// `Ctx::child` contexts can hand out the same local `(x, y)` for two different screen cells, so a
/// `Tally` is a probe over **one rectangle** — which is exactly the rectangle the partition rule is
/// about: *every component and every helper writes a partition of its rectangle*.
///
/// # The one place the union is a model rather than a report
///
/// A verb starting left of the clip consumes clusters that are discarded (
/// clamp-and-discard), so `x + cells` is not where the write landed in that case and the union is
/// wrong by the discarded prefix. [`Tally::text`] is honest for a fixture that draws inside its
/// context, which is what the partition rule requires of a component in the first place, and a fixture that draws
/// outside one is already failing the rule the tally is measuring.
#[derive(Clone, Default, Debug)]
pub struct Tally {
    verbs: u64,
    writes: u64,
    reported: u64,
    asked: u64,
    cells: BTreeSet<(i32, i32)>,
}

impl Tally {
    /// An empty tally.
    pub fn new() -> Tally {
        Tally::default()
    }

    /// Draw a string and fold in what the engine says it wrote.
    ///
    /// Returns the engine's own column count, so a caller can advance by what actually landed rather
    /// than by what it asked for.
    pub fn text(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, s: &str, st: Paint) -> u16 {
        let written = cx.text(x, y, s, st);
        let columns = written.cells;
        self.verbs += 1;
        self.writes += u64::from(columns);
        self.reported += u64::from(columns);
        self.asked += vitui_runtime::layout::text::width(s) as u64;
        // **Root coordinates**, which is the correction: the union used to be
        // taken where the verb was *called*, so a header at `(0, 0)` inside a band and a body row
        // at content `(0, 0)` inside the scroll scope beside it counted as one cell. See
        // [`Tally::distinct`].
        let (ox, oy) = cx.origin();
        for dx in 0..i32::from(columns) {
            self.cells.insert((x + ox + dx, y + oy));
        }
        columns
    }

    /// Draw one cluster and fold in what the engine says it wrote.
    ///
    /// The verb a per-cell oracle is written on — [`crate::runner::reference`] visits every cell of
    /// its rectangle and writes it with one of these. Reported rather than modelled, for the same
    /// reason [`Tally::text`] is: `Ctx::set` returns the engine's `Written`.
    pub fn set(&mut self, cx: &mut Ctx<'_, '_>, x: i32, y: i32, cluster: &str, st: Paint) -> u16 {
        let written = cx.set(x, y, cluster, st);
        let columns = written.cells;
        self.verbs += 1;
        self.writes += u64::from(columns);
        self.reported += u64::from(columns);
        self.asked += vitui_runtime::layout::text::width(cluster) as u64;
        let (ox, oy) = cx.origin();
        for dx in 0..i32::from(columns) {
            self.cells.insert((x + ox + dx, y + oy));
        }
        columns
    }

    /// Fold in a rectangle the caller has just filled. **Modelled, not reported.**
    ///
    /// `Ctx::fill` returns `()`, so there is nothing for the engine to tell us; the geometry arrives
    /// as four scalars read off the rectangle's public fields at the call site, which is stated in
    /// this module's header rather than hidden behind a convenience. The `cx` is not a fifth
    /// scalar's worth of ceremony: it carries the origin the union is taken in, and without it this
    /// verb would be the one hole left in [`Tally::distinct`]'s promise.
    pub fn filled(&mut self, cx: &Ctx<'_, '_>, x: i32, y: i32, w: u16, h: u16) {
        self.verbs += 1;
        self.writes += u64::from(w) * u64::from(h);
        self.asked += u64::from(w) * u64::from(h);
        // **The context is here only for its origin**, and it is here because the union has to be in
        // one coordinate system: a fixture that mixed `Ctx::fill` inside a `Ctx::child` with
        // `Ctx::text` at the root would otherwise reproduce exactly the phantom double write
        // an earlier pass took out of the other two verbs. See [`Tally::distinct`].
        let (ox, oy) = cx.origin();
        for dy in 0..i32::from(h) {
            for dx in 0..i32::from(w) {
                self.cells.insert((x + ox + dx, y + oy + dy));
            }
        }
    }

    /// How many drawing calls were made.
    pub fn verbs(&self) -> u64 {
        self.verbs
    }

    /// Columns written, reported plus modelled.
    pub fn writes(&self) -> u64 {
        self.writes
    }

    /// The part of [`Tally::writes`] the engine itself reported.
    ///
    /// A gate that wants *writes == distinct* to be a comparison between two different sources
    /// asserts `reported == writes` first; where they differ, both sides of the pair are this
    /// crate's arithmetic and the equality is checking a model against itself.
    pub fn reported(&self) -> u64 {
        self.reported
    }

    /// Columns the caller asked for, before clipping.
    ///
    /// `asked - reported` is what the clip discarded, and a component whose content leaves its
    /// rectangle is the only way for a fixture drawn inside one context to make it non-zero.
    pub fn asked(&self) -> u64 {
        self.asked
    }

    /// How many distinct cells were touched, **in the coordinates of the frame's root**.
    ///
    /// # This moved into root coordinates, and the old reading was a defect
    ///
    /// The union used to be taken in the coordinates each verb was *called* in, which is exact for
    /// a fixture drawn straight into the frame and wrong for every component that narrows. A pinned column
    /// met it first — *both recorders union in the coordinates of the `Ctx` the verb was called on,
    /// so a translated band makes `distinct` meaningless* — and components 19 met it as a number: a
    /// scroll area with a sticky header reported **299 double writes on a frame that has none**,
    /// because the band's first cell and the body's first cell are both `(0, 0)` in their own
    /// contexts and land 299 columns and one row apart on the surface.
    ///
    /// The repair is [`vitui_runtime::Ctx::origin`], which the runtime did not publish at first.
    /// Nothing that draws at the root moved: every fixture written before it draws at origin
    /// `(0, 0)`, where the two readings are the same number.
    pub fn distinct(&self) -> u64 {
        self.cells.len() as u64
    }

    /// Whether a cell was touched. The sentinel's question, asked of the model instead of the screen.
    ///
    /// **This is not the sentinel and must not be sold as one** — see [`sentinel`]. It answers from
    /// the same arithmetic that produced [`Tally::distinct`], so a verb that reported a span it did
    /// not write is invisible to both.
    pub fn touched(&self, x: i32, y: i32) -> bool {
        self.cells.contains(&(x, y))
    }
}

/// The nine, gathered from one frame.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Counters {
    /// Columns written. See [`Tally::writes`].
    pub writes: Reading,
    /// Distinct cells touched. See [`Tally::distinct`].
    pub distinct: Reading,
    /// Drawing calls. See [`Tally::verbs`].
    pub verbs: Reading,
    /// **Rect marked damaged — the one that is not here.**
    ///
    /// `crates/vitui-engine/src/damage.rs` is `pub(crate)` from top to bottom: `RowBits::mark`,
    /// `mark_all`, `clear` and `for_each_run` are all crate-private, and nothing on `Surface`,
    /// `View`, `Screen` or `Presented` returns a damaged-cell count. So this is not a re-export the
    /// runtime forgot — the number is not on the engine's public surface for anyone.
    ///
    /// What would have to become public is named in the [`Reading::Unreachable`] arm this field
    /// carries, and it is two items and not one: a count on the engine (`Presented::marked`, which
    /// `present` already computes when it clears the damage) and a runtime door to it, because this
    /// crate cannot name `vitui_engine` at all.
    pub marked: Reading,
    /// Hit-index entries. `Frame::hits().len()`.
    pub regions: Reading,
    /// Tab stops. `Frame::stop_count()`.
    pub tab_stops: Reading,
    /// Duplicate claims. `Frame::ids().merges()`.
    pub merges: Reading,
    /// Overlay layers alive. `Driver::layers_live()`.
    ///
    /// **The census's number, and the column counts one thing differently**: its table reads 3 for
    /// a modal with its scrim, where `layers_live` counts the owner and folds the scrim into it. The
    /// discrepancy is stated rather than corrected, because the runtime's number is the one a gate
    /// can read and the other is a prototype's.
    pub content_layers: Reading,
    /// Allocations, **as a total**. See [`Allocations`].
    pub allocations: Reading,
}

impl Counters {
    /// What `marked` needs, kept as a constant so the register and this module cannot drift.
    const MARKED_NEEDS: &'static str = "a damaged-cell count on the engine's public surface (`damage.rs` is `pub(crate)` \
         throughout, so `Presented` carries no such field) and a runtime accessor for it, because \
         this crate cannot name `vitui_engine`";

    /// Gather the nine from a driver that has just run a frame, plus the caller's own tally.
    ///
    /// The driver is read **after** the draw and not inside it: the five frame structures belong to
    /// `Frame` and a `Ctx` does not carry them, which is a fact about the surface that only a
    /// consumer finds (`tests/crate_line.rs`).
    pub fn of(driver: &Driver, drawn: &Tally, allocations: Allocations) -> Counters {
        let frame = driver.inspect();
        Counters {
            writes: Reading::Measured(drawn.writes()),
            distinct: Reading::Measured(drawn.distinct()),
            verbs: Reading::Measured(drawn.verbs()),
            marked: Reading::Unreachable {
                needs: Counters::MARKED_NEEDS,
            },
            regions: Reading::Measured(frame.hits().len() as u64),
            tab_stops: Reading::Measured(frame.stop_count() as u64),
            merges: Reading::Measured(u64::from(frame.ids().merges())),
            content_layers: Reading::Measured(driver.layers_live() as u64),
            allocations: Reading::Measured(allocations.total()),
        }
    }

    /// One counter by name, so a report can loop over [`Counter::ALL`].
    pub fn get(&self, counter: Counter) -> Reading {
        match counter {
            Counter::Writes => self.writes,
            Counter::Distinct => self.distinct,
            Counter::Verbs => self.verbs,
            Counter::Marked => self.marked,
            Counter::Regions => self.regions,
            Counter::TabStops => self.tab_stops,
            Counter::Merges => self.merges,
            Counter::ContentLayers => self.content_layers,
            Counter::Allocations => self.allocations,
        }
    }

    /// How many of the nine this crate can actually read. **Eight.**
    pub fn reachable(&self) -> usize {
        Counter::ALL
            .iter()
            .filter(|c| self.get(**c).measured().is_some())
            .count()
    }
}

/// **The sentinel: the cells of a recorded surface that no verb wrote.**
///
/// > `distinct cells touched == area.w * area.h` (no cell never)
///
/// This is the second half of the partition rule, and the half that **had no counter on either
/// map** — a cell nobody writes keeps what was already there, and what was already there is almost
/// always right. It has since inverted; what follows is why it is a reading of a
/// [`crate::runner::Canvas`] rather than the surface probe originally prescribed.
///
/// # A screen probe was asked for, and the recorder is the stricter instrument
///
/// The prescription is: stamp a `Theme::custom` paint no role can produce over the **base** layer
/// between frames, draw one more, count the cells still carrying it. It has to be the base layer,
/// because the engine filters a write whose value equals the cell's current value — so downstream of
/// that filter a cell rewritten with what it already held is indistinguishable from one never
/// written. Three barriers stood in front of it, and the third is a decision rather than a gap:
///
/// 1. **The stamp's value — lifted** when the runtime re-exported it. `Rgb` is
///    `vitui_runtime::Rgb` now, so `Theme::custom(fg, bg)` is callable here.
/// 2. **The stamp's reach — never a barrier.** `Ctx::clear` writes the whole of a context.
/// 3. **The readback — no cell is visible outside the engine, and it holds.** No `Surface`, `View`, `Screen` or `Presented`
///    method returns a cell, a handle or a style bit, to this crate or to the engine's own callers.
///
/// **It does not need lifting, and the reason is the pair.** `writes == distinct` — the rule's
/// *first* half — has always been read off [`Tally`], whose union has been in the coordinates of the
/// frame's root since components 19. The second half is that same union compared against the area,
/// so a screen probe would make one equality out of two different instruments; and the recorder is
/// the **conservative** one of the two, because a verb that does not go through the caller's `Ink`
/// is invisible to it and makes this number *larger*. A surface probe counts the engine's own clear
/// and passes quietly — which is the shape of a defect this map has met twice: components 15's
/// `ledger` drew a table cell with `cx.text` rather than through the ink it was handed (78 cells and
/// 10 verbs on a frame that wrote 1 560, on a screen that looked correct), and components 39's
/// gallery cleared through `Direct`.
///
/// # It is a count and it fails loudly by being non-zero
///
/// Returning `0` where nothing could be counted would satisfy *no cell never* on every screen for
/// ever, which is the first refinement — *a threshold on the wrong side of the question is not
/// a weak gate, it is a green one*. There is nothing to fake now: the count comes from the surface
/// the frame was recorded on, and its gates are `crate::gallery`'s assembled sweep and the
/// per-construction sweep in `tests/golden.rs`.
pub fn sentinel(canvas: &crate::runner::Canvas) -> Reading {
    let cells = usize::from(canvas.w()) * usize::from(canvas.h());
    Reading::Measured((cells - canvas.written()) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vitui_runtime::layout::{Col, Constraint::Weight};
    use vitui_runtime::{Id, Interest, Role};

    /// **The nine are eight and one, and the number is written down.**
    ///
    /// `obligations.rs`'s arrangement: a count makes the first counter to become reachable a
    /// deliberate edit here rather than a quiet change of colour. The one is `marked`.
    #[test]
    fn eight_of_the_nine_counters_are_reachable_and_marked_is_not() {
        let mut driver = Driver::headless(40, 6).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            let band = cx.area();
            tally.text(cx, band.x, band.y, "counted", body);
        });
        let counters = Counters::of(&driver, &tally, Allocations::over(1, 0));

        assert_eq!(counters.reachable(), 8);
        let absent: Vec<&str> = Counter::ALL
            .iter()
            .filter(|c| counters.get(**c).measured().is_none())
            .map(|c| c.word())
            .collect();
        assert_eq!(
            absent,
            vec!["marked"],
            "a counter has changed standing. That is the point of the backlog and it is also a \
             deliberate edit here, to this module's header and to `crate::gates::REGISTER`"
        );
    }

    /// **`marked` fails loudly.** A gate nobody has watched fail is not a gate.
    #[test]
    #[should_panic(expected = "`marked` is unreachable across the crate line")]
    fn marked_panics_rather_than_answering_zero() {
        let mut driver = Driver::headless(10, 2).expect("a sink cannot fail to attach");
        driver.frame(|_cx| {});
        let counters = Counters::of(&driver, &Tally::new(), Allocations::over(1, 0));
        let _ = counters.marked.get(Counter::Marked);
    }

    /// **The sentinel answers, and it answers about the surface it was handed.**
    ///
    /// It used to be the loud one beside [`marked_panics_rather_than_answering_zero`] — three
    /// barriers, `Reading::Unreachable`, and a `should_panic` naming the cell rule. It
    /// inverted it: see [`sentinel`] for why the readback that decision forbids is not what this
    /// question needs.
    ///
    /// Both directions, on one surface: a row written and a row left alone, so a `0` here is a
    /// reading and not a constant.
    #[test]
    fn the_sentinel_counts_the_cells_no_verb_wrote() {
        use crate::ink::Ink as _;
        let mut driver = Driver::headless(10, 2).expect("a sink cannot fail to attach");
        let mut pen = crate::runner::Pen::new(10, 2);
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            pen.pad_to(cx, 0, 0, "ten", 10, body);
        });
        assert_eq!(sentinel(pen.canvas()).get(Counter::Distinct), 10);

        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            pen.pad_to(cx, 0, 1, "ten", 10, body);
        });
        assert_eq!(sentinel(pen.canvas()).get(Counter::Distinct), 0);
    }

    /// **A measured reading does not panic**, which is what makes the two above evidence.
    #[test]
    fn a_measured_counter_is_silent() {
        assert_eq!(Reading::Measured(7).get(Counter::Verbs), 7);
    }

    /// **`writes == distinct` separates when two verbs overlap, and the two sides come from two
    /// places.** The first half, and the only form of it this crate can run.
    ///
    /// The left-hand side is the engine's — `Ctx::text` returns how many columns it wrote — and the
    /// right-hand side is this crate's union over the same spans. A gate over one source could not
    /// tell them apart.
    #[test]
    fn writes_equals_distinct_until_two_verbs_overlap() {
        let mut driver = Driver::headless(40, 4).expect("a sink cannot fail to attach");

        let mut clean = Tally::new();
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            clean.text(cx, 0, 0, "abcde", body);
            clean.text(cx, 5, 0, "fghij", body);
        });
        assert_eq!(clean.writes(), clean.distinct(), "no cell twice");
        assert_eq!(clean.writes(), 10);
        assert_eq!(
            clean.reported(),
            clean.writes(),
            "every column here came from the engine, so the equality is two sources and not one"
        );
        assert_eq!(clean.asked(), clean.reported(), "nothing was clipped");

        // And the other direction, which is what makes it a gate: one column of overlap.
        let mut overlapping = Tally::new();
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            overlapping.text(cx, 0, 0, "abcde", body);
            overlapping.text(cx, 4, 0, "fghij", body);
        });
        assert_eq!(overlapping.writes(), 10);
        assert_eq!(overlapping.distinct(), 9, "column 4 was written twice");
        assert_ne!(overlapping.writes(), overlapping.distinct());
    }

    /// **The clip is visible as `asked - reported`, and the engine is the one reporting it.**
    ///
    /// A component whose content leaves its rectangle is the other failure, and this is the number
    /// that shows it without a cell readback.
    #[test]
    fn a_verb_that_runs_off_its_context_is_reported_short_by_the_engine() {
        let mut driver = Driver::headless(8, 2).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            tally.text(cx, 4, 0, "abcdefgh", body);
        });
        assert_eq!(tally.asked(), 8, "eight columns were asked for");
        assert_eq!(tally.reported(), 4, "four fitted");
        assert_eq!(tally.distinct(), 4);
    }

    /// **`verbs <= writes`, and never an equality across sizes.** C08 row, as a relation.
    ///
    /// The relation and not the equality, because a verb that writes nothing is legitimate — a label
    /// clipped entirely out of its context is one — so `verbs == writes` is a number belonging to
    /// the fixture rather than to the mechanism.
    #[test]
    fn verbs_never_exceed_writes_and_the_relation_is_not_an_equality() {
        let mut driver = Driver::headless(20, 3).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            let band = cx.area();
            let mut lanes = [band; 3];
            let n = Col::new().split_into(band, &[Weight(1); 3], &mut lanes);
            for lane in &lanes[..n] {
                tally.text(cx, lane.x, lane.y, "row", body);
            }
        });
        assert!(tally.verbs() <= tally.writes());
        assert_eq!((tally.verbs(), tally.writes()), (3, 9));
    }

    /// **`filled` is modelled, and the tally says so.**
    ///
    /// The rectangle arrives as four scalars because `Rect` cannot be named here, and `Ctx::fill`
    /// returns `()` so there is no engine number to fold. `reported < writes` is how a reader knows
    /// the pair above has stopped being two sources.
    #[test]
    fn a_filled_rectangle_is_modelled_and_reported_by_nobody() {
        let mut driver = Driver::headless(10, 3).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| {
            let body = cx.theme().paint(Role::Body);
            let band = cx.area();
            cx.fill(band, " ", body);
            tally.filled(cx, band.x, band.y, band.w, band.h);
        });
        assert_eq!(tally.writes(), 30);
        assert_eq!(tally.distinct(), 30);
        assert_eq!(tally.reported(), 0, "the engine reported none of it");
        assert!(tally.touched(9, 2));
        assert!(!tally.touched(10, 2));
    }

    /// **An allocation total over no frames is refused in the constructor.**
    ///
    /// [`crate::obligations::Verdict::of`]'s vacuity rule, arriving where the vacuous answer would
    /// be a plausible `0` rather than an obvious one.
    #[test]
    #[should_panic(expected = "over zero frames is not a measurement")]
    fn an_allocation_total_over_no_frames_is_refused() {
        let _ = Allocations::over(0, 0);
    }

    /// **A mean cannot see anything below `n`; a total can see one.** The refinement 2, as
    /// arithmetic rather than as a sentence.
    ///
    /// This is the number the map actually met: `player::chrome` allocated on **40 of 40** frames
    /// and every prototype printed `allocs / n`. The instructive half is the neighbouring case —
    /// one frame in forty — where the integer mean is `0` and the gate under it is green.
    #[test]
    fn a_mean_over_n_cannot_see_one_frame_in_n() {
        let frames = 40u64;
        for allocations in [1u64, 39] {
            assert_eq!(
                allocations / frames,
                0,
                "the mean is what every prototype printed"
            );
            assert_ne!(allocations, 0, "the total is what a gate has to read");
        }
        // And the total is the whole of what `Allocations` will hand over.
        let seen = Allocations::over(40, 1);
        assert_eq!((seen.total(), seen.frames()), (1, 40));
    }

    /// **`merges` is free, already computed, and this is the wiring.** C01 row.
    ///
    /// Both directions in one test: three lanes drawn from **one call site** are one id and two
    /// merges — `Ctx::id()` is `#[track_caller]` — and the keyed loop beside it is what a container
    /// owes. A gate asserting only the zero would pass on a frame that drew nothing.
    #[test]
    fn merges_are_zero_when_a_container_keys_its_children_and_not_before() {
        let mut driver = Driver::headless(30, 3).expect("a sink cannot fail to attach");

        driver.frame(|cx| {
            let band = cx.area();
            let mut lanes = [band; 3];
            let n = Col::new().split_into(band, &[Weight(1); 3], &mut lanes);
            for lane in &lanes[..n] {
                let id = cx.id();
                let _ = cx.interact(id, *lane, Interest::CLICK);
            }
        });
        let counters = Counters::of(&driver, &Tally::new(), Allocations::over(1, 0));
        assert_eq!(counters.merges.get(Counter::Merges), 2);

        driver.frame(|cx| {
            let band = cx.area();
            let mut lanes = [band; 3];
            let n = Col::new().split_into(band, &[Weight(1); 3], &mut lanes);
            for (i, lane) in lanes[..n].iter().enumerate() {
                cx.with_key(i as u64, |cx| {
                    let id = cx.id();
                    let _ = cx.interact(id, *lane, Interest::CLICK);
                });
            }
        });
        let counters = Counters::of(&driver, &Tally::new(), Allocations::over(1, 0));
        assert_eq!(counters.merges.get(Counter::Merges), 0);
        assert_eq!(counters.regions.get(Counter::Regions), 3);
    }

    /// **The other five reachable counters, read through `Counters` rather than through `Frame`.**
    ///
    /// One frame with two stops, one overlay and a hit index, so that a counter wired to the wrong
    /// accessor is visible as a number rather than as a zero.
    #[test]
    fn the_frame_counters_come_from_the_frame_and_the_layer_count_from_the_driver() {
        use vitui_runtime::OverlayOpts;

        let mut driver = Driver::headless(40, 10).expect("a sink cannot fail to attach");
        driver.frame(|cx| {
            let band = cx.area();
            let mut lanes = [band; 2];
            let n = Col::new().split_into(band, &[Weight(1); 2], &mut lanes);
            for (i, lane) in lanes[..n].iter().enumerate() {
                cx.with_key(i as u64, |cx| {
                    let id = cx.id();
                    let _ = cx.interact(id, *lane, Interest::CLICK.with(Interest::FOCUS));
                });
            }
            cx.overlay(
                Id::named("popup"),
                lanes[0],
                OverlayOpts::sized(10, 3),
                |cx: &mut Ctx<'_, '_>| {
                    let body = cx.theme().paint(Role::Body);
                    cx.text(0, 0, "popup", body);
                },
            );
        });
        let counters = Counters::of(&driver, &Tally::new(), Allocations::over(1, 0));
        assert_eq!(counters.regions.get(Counter::Regions), 2);
        assert_eq!(counters.tab_stops.get(Counter::TabStops), 2);
        assert_eq!(counters.merges.get(Counter::Merges), 0);
        assert_eq!(counters.content_layers.get(Counter::ContentLayers), 1);
    }
}
