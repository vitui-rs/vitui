//! Spec §21's register: **a hundred and thirty-five gates as a value, one row per gate, and a number
//! for how many of them anything runs.**
//!
//! > The register is data, not prose — one row per gate with its kind, its owner, where it stood at
//! > the branch point and where it stands now, so the delta is a number a test asserts. The reason
//! > is §17's: every obligation stated as a sentence on this map has been broken by someone who had
//! > read it. (§21)
//!
//! The finding this exists for is C11's, and it is the one R15 made on the runtime's map one layer
//! up: **the corpus was not being run.** Of the eighteen gates the twelve component tickets
//! produced, **two** were evaluated by anything `cargo test` runs; six were asserted inside one
//! prototype's own binary on one screen, and ten were a number printed by a `fn main()` — which
//! `cargo test` does not run — or absent.
//!
//! # What a gate is, and the three rules this map adds
//!
//! R15's rules are inherited unchanged and are what [`Kind`] means: **a gate is a count, a ratio, an
//! equality or a compile outcome; a timing is a report** · a gate is an equality only when the number
//! is a property of the mechanism · a report may not be load-bearing for a gate · a positive twin
//! must name the protected item by path. This map adds three, each from a gate that was **green and
//! wrong**, and each of the three is encoded where it can bite rather than written down here:
//!
//! 1. **A threshold on the wrong side of the question is not a weak gate, it is a green one.** The
//!    gallery's theme-swap gate asserted `changed > 0` — and one cell of 4 800 satisfies it while
//!    3 583 carry the old palette. Row 39 keeps the old spelling as a negative case, run against the
//!    exact set it failed to catch; and it is why [`crate::counters::Reading`] panics instead of
//!    answering `0`.
//! 2. **A mean cannot see anything below `n`; a total can see one.** Every prototype reported
//!    `allocs / n` with `n` between 40 and 200, so a frame allocating on `n − 1` of `n` frames
//!    reports 0. [`crate::counters::Allocations`] has a `total` and **no `mean`**, which is the rule
//!    as a type rather than as a comment; row 32's instrument measures a frame that allocates on one
//!    frame of two hundred and watches the two arithmetics disagree.
//! 3. **Name the exception; do not loosen the gate.** R08's *a walkthrough visits every tab stop
//!    exactly once* fails on exactly one panel of twelve and **correctly** — a modal's `Trap` is what
//!    a modal is. Row 34 is the conjunction R08 meant, with `Frame::trap_scopes()` so the exception
//!    can be named, and its instrument asserts the loosened spelling passing on the case it must not.
//!
//! # Four standings, and the fourth is the vacuity arm
//!
//! The runtime's register has two — wired, or red with the ticket that lights it — and argues at
//! length that a third would let a property quietly never arrive. It is right, and this map needs
//! two more anyway, because it is a register kept by a crate **with no components in it**:
//!
//! - [`Standing::Evaluated`] and [`Standing::Red`] are the runtime's two. A red row is pinned in its
//!   failing state, asserts its exact failing set, fires in both directions and names the ticket that
//!   inverts it.
//! - [`Standing::Unreachable`] is a **result**. Six gates cannot be written from this crate at all,
//!   and each says exactly what would have to become public. That is not *not yet*: `vitui-components`
//!   depends on `vitui-runtime` and nothing else (§19's C6), so a gate needing an engine name is a
//!   compile error and no amount of component code changes it.
//! - [`Standing::Unsubjected`] is the vacuity arm, and it exists because [`crate::obligations`]
//!   already proved it necessary one file over: *a query over an obligation nobody has met yet is
//!   the exact shape that returns green by accident*. **Six gates here could run and have nothing to
//!   run over** — it was fifteen when no component existed — and a register that filed those as
//!   `Evaluated` would be claiming six green gates over an empty population. The count in the line
//!   below is the authority; this is a summary of it.
//!
//! **A hundred and eighteen evaluated, five red, six unreachable, six unsubjected**, and
//! `tests::a_hundred_and_eighteen_rows_are_evaluated_and_the_rest_say_why_not` is what makes the next
//! change a deliberate edit rather than a quiet one. It was eighteen / four / six / sixteen until components
//! ticket 05, which inverted row 26 — the glyph-set count, red because it had nothing to be about —
//! and subjected row 27, the cross-family collapse gate; ticket 07 added five, and none of them
//! moved a standing that was already taken. **Ticket 08 added four and inverted one**, and the
//! inversion is the one to read.
//!
//! **Ticket 09 added five and one of them is red on purpose.** Rows 57–60 are the dense screen —
//! 338 regions, the metric row, the equality against a naive twin at two sizes, and ADR 0026's five
//! re-damage instances each standing on a screen instead of in a sentence — and they are
//! `Evaluated` over a screen rather than over a component, which is the same standing ticket 04's
//! four rows have and for the same reason: what they gate is that *the instrument separates a
//! correct build from a defective one*. **Row 61 is the fourth red row**, and it is the only one on
//! this register that is red because a *scene* has no subject rather than because a gate fails. The
//! distinction it encodes is criterion 7's: a scene that fails because it is unimplemented and one
//! that fails because the code is wrong are the same failure unless the message separates them, so
//! the failing set is computed by opening the four files components 10 will declare in and the
//! panic names all four and the ticket.
//!
//! **Components ticket 11 added two and rewrote a third, and the three are three different kinds of
//! change.** Row 66 is `Evaluated` — *regions identical at 1 000, 100 000 and 1 000 000 rows*, over
//! the listing's two arms, and it exists because **row 4 cannot see the defect it is about**: the
//! engine reports a fully clipped verb as zero columns, so a listing that iterates its whole content
//! writes exactly what a windowed one writes — 3 200 at every volume — while declaring 1 000 001 hit
//! entries against 81. *Writes flat 1k -> 1M* is green on that build. Row 67 is `Red` and is row 61
//! one ticket later: five scenes are pinned because `collection` is undeclared, and the sentence
//! that says which failure that is is the whole of components 11's criterion 7. **Row 29 kept its
//! standing and gained its missing half**: the wheel gate's *twenty wheel clicks move the offset
//! twenty* direction now runs, with the click's delta handed to `Response::scrolled`'s own
//! arithmetic on both arms — and at the time the `Mouse` barrier was untouched and *still the reason
//! the click cannot be posted*, which is the distinction row 5 got wrong. **Runtime architecture
//! issue 22 has since lifted that barrier too**; the substitution stays until components 20, and the
//! row stays red because the defect it names is the unconditional reveal.
//!
//! # Row 5 was not *not yet*. It was wrong, and an `Unreachable` that is wrong is the worst standing
//!
//! Row 5 read: *`Mods` … is `reachable_as: None`. So no key can be posted and no chord can be
//! pressed.* Both halves of that sentence were checked against `crates/vitui-runtime/src/line.rs`
//! and the first is true. **The second does not follow from it.** A struct literal needs a *value*
//! for each field and not a name for its type, and `vitui_runtime::keys::Chord` carries a
//! `pub mods: Mods` field with three `const` builders — so `Chord::key('s').ctrl().mods` is
//! `Mods::CTRL`, written from a crate that cannot spell `Mods`. `Driver::post_key` is public for
//! `post_mouse`'s stated reason, and the gate runs.
//!
//! That is a standing this register got **wrong for five tickets**, and the shape is worth naming
//! because [`Standing::Unreachable`] invites it: *it cannot be written from this crate at all* is a
//! claim about the whole space of programs, and every other standing here is a claim about one. The
//! discipline that catches it is the one this file already has — `needs` must name *an item, not a
//! wish* — applied one step further: **the item has to be the thing actually required.** Row 5
//! needed a `Mods` **value** and filed a barrier against the `Mods` **name**.
//!
//! Row 45's barrier survives the same test and rows 7 and 29's did too: those needed `Rgb` and
//! `Mouse`, and neither had any nameable box a value could travel inside — `Mouse` needs a `Buttons`
//! and a `MouseKind`, and **neither of those was in `ENGINE_NAMES` at all**, reachable or not. The
//! difference between row 5 and row 45 is not the strength of the barrier, it is that one of them
//! was checked by trying it.
//!
//! **That last observation is what runtime architecture issue 22 acted on**, and it is why the rule
//! it settled is about *construction* rather than about naming: a name a consumer can write but not
//! build is a barrier wearing a re-export's clothes. `Rgb`, `Mouse`, `Buttons`, `MouseKind`, the
//! notch, `Rect` and `Mods` are all reachable now, the three `Barrier` citations into
//! `crates/vitui-runtime/src/line.rs` are struck from this register, and
//! `tests::the_named_barriers_have_lifted` watches the direction they came from.
//!
//! **Row 48 is the one to read beside row 1.** Row 1 wants *cells marked on a steady frame == 0*
//! and is `Unreachable`: `damage.rs` is `pub(crate)` throughout and nothing above the engine can
//! read the engine's damage. Row 48 asks the question the engine's own equality filter turns that
//! into — *writes whose value differs from what is already there* — which is computable at the verb
//! boundary. It does not invert row 1 and is not filed as doing so; it is the input where row 1 is
//! the output, and both rows now say so.
//!
//! # The instruments are values with files in them
//!
//! The engine's register names where an entry runs in a sentence of English, and a sentence cannot be
//! checked — so a row whose test has been renamed still reads as wired. Runtime ticket 19 fixed that
//! by making an [`Instrument`] a value with a **file** in it, and this register inherits the
//! arrangement and adds one arm: [`Instrument::Barrier`], a line in **somebody else's** source that
//! is the reason a row cannot run. `EngineName { name: "Rgb", reachable_as: None }` is not a claim
//! about the world, it is a line in `crates/vitui-runtime/src/line.rs`, and a test opens that file
//! and fails the day the barrier lifts.
//!
//! **All four of the named ones have since lifted**, and the arrangement is what made that legible:
//! the citation is a line, so `reachable_as: Some(..)` appearing under it is a *failing test* and
//! not a quiet change of meaning. `tests::the_named_barriers_have_lifted` is that test, inverted in
//! place. **Note the failure mode it was one edit away from**: `name: "Rgb",` is still in
//! `ENGINE_NAMES` — only the line beneath it changed — so a `Barrier` compared against
//! `name: "Rgb",` alone would have gone on passing while meaning the opposite. The citations are
//! struck for that reason and not because the rows went green.
//!
//! # Two of §21's rows are corrected here, from the shipped code rather than from the prototypes
//!
//! §21 was written against twelve prototypes, and two of its sentences do not survive contact with
//! the crates that shipped. Both corrections are recorded on the row and neither changes its
//! standing:
//!
//! - **Row 33.** §21 says the geometric form of the tab-stop gate is impossible because *the ring
//!   does not carry geometry* — `RingEnt::rect` is zero unless `Frame::ring_geometry` is on. The
//!   shipped type is `vitui_runtime::focus::Stop` and its `rect` is filled on **every** entry,
//!   unconditionally. The conclusion holds for a better reason: the rectangle recorded is the one the
//!   widget *declared*, intersected with nothing at all.
//! - **Row 31.** §21 says `Task` and `Worker` are `!Sync`. `Worker` is `Sync` — it is an
//!   `Arc<Inbox>` — and the half that is load-bearing is `Task`, which is `!Send` and `!Sync` on a
//!   `PhantomData<*const ()>` brand.

/// What shape of gate a row is.
///
/// R15's list, unchanged: **a gate is a count, a ratio, an equality or a compile outcome; a timing is
/// a report.** [`Kind::Relation`] and [`Kind::Invariant`] are the two §21's own table adds, and they
/// are not loopholes — a relation is what a row says when its number belongs to the **data** rather
/// than to the mechanism, which is refinement 1's whole distinction.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// A number that must be exact, and the number is a property of the mechanism or of a normative
    /// scene.
    Count,
    /// A proportion, gated at cliff granularity with the headroom written beside it.
    Ratio,
    /// Two things computed two ways, compared.
    Equality,
    /// An inequality. **The honest form when the number belongs to the data** — `verbs <= writes` is
    /// §21's own example, and it says *never verb equality across sizes* in as many words.
    Relation,
    /// Something that must not compile, with a positive twin naming the protected item by path.
    CompileOutcome,
    /// A property that must hold at every point of a transition, not only at its ends.
    Invariant,
}

impl Kind {
    /// The word §21's table prints.
    pub fn word(self) -> &'static str {
        match self {
            Kind::Count => "count",
            Kind::Ratio => "ratio",
            Kind::Equality => "equality",
            Kind::Relation => "relation",
            Kind::CompileOutcome => "compile outcome",
            Kind::Invariant => "invariant",
        }
    }
}

/// One thing that runs a register row — or, for [`Instrument::Barrier`], one thing that stops it.
///
/// Every arm carries a **file**, relative to the workspace root, and something to look for inside it.
/// That is the difference between this register and a document: a row cannot claim to be wired
/// somewhere vague, because `tests::every_instrument_names_something_that_exists` opens the file
/// and fails if the thing is not in it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Instrument {
    /// A live `#[test]` function. `name` is the bare function name.
    Unit {
        /// The file, relative to the workspace root.
        file: &'static str,
        /// The test function's name.
        name: &'static str,
    },
    /// A paired doctest: a `compile_fail` fence and the runnable twin beside it.
    ///
    /// `hostile` is the line the must-fail half is *about*, and it is looked for **inside a
    /// `compile_fail` block** — a mention in prose does not satisfy this, and neither does the twin.
    Pair {
        /// The file, relative to the workspace root.
        file: &'static str,
        /// A fragment of the hostile line.
        hostile: &'static str,
    },
    /// **A line in somebody else's source that is the reason a row cannot run.**
    ///
    /// The arm this register adds to the runtime's three, and it exists because six of these
    /// rows are stopped by a fact rather than by an absence. A fact can be pointed at: `Rgb` being
    /// unreachable is the line `name: "Rgb",` in `crates/vitui-runtime/src/line.rs`, not an opinion
    /// about the runtime's surface. Compared against a **trimmed** line, which is the engine's own
    /// correction to its vacuous version of this shape.
    Barrier {
        /// The file, relative to the workspace root.
        file: &'static str,
        /// The line, as it is written, with leading and trailing space ignored.
        line: &'static str,
    },
    /// A file that prints numbers and gates nothing.
    ///
    /// R15's refinement 2 in a type: a row whose **only** instrument is one of these may not be
    /// `Evaluated`. `cargo clippy --all-targets` compiles `examples/*.rs` and nothing evaluates an
    /// `assert!` inside one.
    Report {
        /// The file, relative to the workspace root.
        file: &'static str,
    },
}

impl Instrument {
    /// The file this instrument lives in.
    pub fn file(self) -> &'static str {
        match self {
            Instrument::Unit { file, .. }
            | Instrument::Pair { file, .. }
            | Instrument::Barrier { file, .. }
            | Instrument::Report { file } => file,
        }
    }
}

/// Where a gate stands **in this crate**, today.
///
/// See this module's header for why there are four arms and why the last two are results rather than
/// excuses.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Standing {
    /// It runs, on these instruments, and it passes.
    Evaluated {
        /// What runs it. Never empty.
        by: &'static [Instrument],
    },
    /// **It is pinned in its failing state**, asserting its exact failing set in both directions.
    Red {
        /// What runs it, or what proves it cannot be run. Never empty.
        by: &'static [Instrument],
        /// The exact failing set, with its numbers.
        failing: &'static str,
        /// The implementation ticket that inverts it, as `components NN`.
        inverted_by: &'static str,
    },
    /// **It cannot be written from this crate at all.**
    ///
    /// `needs` names exactly what would have to become public — an item, not a wish.
    Unreachable {
        /// What would have to exist.
        needs: &'static str,
        /// Where the change would be argued, as `components NN` or an architecture issue.
        inverted_by: &'static str,
    },
    /// **It could run and there is nothing to run it over.** No component exists yet.
    ///
    /// The vacuity arm. Filing one of these as `Evaluated` is exactly the accident
    /// [`crate::obligations::Verdict::of`] refuses in its constructor: *zero failures out of zero* is
    /// not an answer.
    Unsubjected {
        /// The implementation ticket that builds the subject, as `components NN`.
        inverted_by: &'static str,
    },
}

impl Standing {
    /// The word the table prints.
    pub fn word(self) -> &'static str {
        match self {
            Standing::Evaluated { .. } => "evaluated",
            Standing::Red { .. } => "red, pinned",
            Standing::Unreachable { .. } => "unreachable",
            Standing::Unsubjected { .. } => "unsubjected",
        }
    }

    /// Whether anything runs this row today.
    pub fn evaluated(self) -> bool {
        matches!(self, Standing::Evaluated { .. })
    }
}

/// One gate.
#[derive(Clone, Copy, Debug)]
pub struct Row {
    /// Its number here, which is how everything else refers to it.
    pub number: u8,
    /// Whether it is a row of spec §21's thirty-two-row table, or one this ticket wrote beside it.
    ///
    /// The count test asserts **the split** rather than the total, so a forty-fifth row has to say
    /// which side of the line it is on — the engine's and the runtime's arrangement, for its reason.
    pub on_spec_table: bool,
    /// The gate, in §21's own words where §21 has words for it.
    pub gate: &'static str,
    /// Count, ratio, equality, relation, compile outcome or invariant.
    pub kind: Kind,
    /// Whose gate it is, in §21's `C NN` numbering — the **architecture** map's tickets, which is
    /// what that table's owner column names. Implementation tickets are spelled `components NN` and
    /// appear only in the `inverted_by` fields.
    pub owner: &'static str,
    /// The section that states it.
    pub section: &'static str,
    /// Where it stands, and on what.
    pub standing: Standing,
}

/// The dense screen's own file, which is where components ticket 09's four rows run.
const DENSE: &str = "crates/vitui-components/src/dense.rs";

/// The listing's own file, which is where components ticket 11's two rows run.
const LISTING: &str = "crates/vitui-components/src/listing.rs";

/// The wheel gate's file. Components ticket 20, and the one row on this register whose instrument
/// posts a pointer event.
const WHEEL: &str = "crates/vitui-components/src/wheel.rs";

/// The scroll area's own file, which is where components ticket 18's six rows run.
const AREA: &str = "crates/vitui-components/src/area.rs";

/// The forest's own file, which is where components ticket 16's rows run.
const FOREST: &str = "crates/vitui-components/src/forest.rs";

/// The grid's own file, which is where components ticket 14's four rows run.
const GRID: &str = "crates/vitui-components/src/grid.rs";

/// The accordion's own file, which is where components ticket 21's rows run.
const ACCORDION: &str = "crates/vitui-components/src/accordion.rs";
/// **`collapsible`'s own file**, which is where components ticket 22's rows run. The component and
/// its two refused spellings are one file, so a reviewer's diff between them is a field.
const DISCLOSE: &str = "crates/vitui-components/src/disclose.rs";

/// The document's own file, which is where components ticket 23's rows run.
const DOCUMENT: &str = "crates/vitui-components/src/document.rs";
const CLUSTERS: &str = "crates/vitui-components/src/clusters.rs";

/// The popup's own file, which is where components ticket 25's rows run.
const POPUP: &str = "crates/vitui-components/src/popup.rs";

/// The collection's own file, which is where components ticket 12's rows run.
const COLLECT: &str = "crates/vitui-components/src/collect.rs";

/// The three scrolling components' own file. Components ticket 19's seven rows are measured here.
const SCROLL: &str = "crates/vitui-components/src/scroll.rs";

/// The order's own file, which is where components ticket 13's rows run.
const ORDER: &str = "crates/vitui-components/src/order.rs";

/// The series screen's own file, which is where components ticket 27's rows run.
const SERIES: &str = "crates/vitui-components/src/series.rs";

/// The chart's own file, which is where components ticket 28's rows run.
const CHART: &str = "crates/vitui-components/src/chart.rs";
const FIELD_NUMBERS: &str = "crates/vitui-components/examples/field_numbers.rs";

/// How many rows of [`REGISTER`] are spec §21's own table. **Thirty-two, and it is closed** — a
/// thirty-third would be a spec change.
pub const SPEC_ROWS: usize = 32;

/// How many rows anything evaluates today. **Forty-eight.**
///
/// The number is the point of the file. §21 counted **2 of 18** at the branch point and **11 of 18**
/// after C11's own pass, both over the prototypes; this is the first count taken over shipped code,
/// and it is forty-eight of seventy-two because twenty-four of the rows are about components that
/// do not exist or need a name the crate line refuses.
///
/// **It was fourteen of forty until ticket 04**, which added the reference-render runner and its
/// four rows. Every one of the four is `Evaluated` over a **fixture** rather than over a component,
/// which is a real standing and not a promoted one: what those rows gate is that *the instrument
/// separates a correct build from a defective one*, and each is watched doing it in both directions.
/// The scenes themselves stay `Unsubjected` in [`crate::scenes`], and that file says why the two are
/// not the same claim.
///
/// **Ticket 07's five are the same kind of standing**, over the chip and the two bars its own
/// helpers stand up rather than over a component: rows 48–52. Row 48 is the one worth reading
/// twice — it is the **reachable form of row 1**, which is `Unreachable` and stays that way, and
/// the two rows now sit side by side saying which question each of them can answer.
///
/// **Ticket 08's four are rows 53–56, and its fifth is row 5** — which moved from `Unreachable` to
/// `Evaluated` without anything in the runtime changing, because the barrier had been misread. See
/// this module's header: that is the only inversion on this register that corrected a *standing*
/// rather than supplying a *subject*, and it is the one worth being suspicious about the next time
/// an `Unreachable` is written.
///
/// **Components ticket 11 moved it from forty-one to forty-two, and the one row is row 66** — the
/// regions equality, `Evaluated` over the listing's two arms rather than over `collection`, which is
/// components ticket 04's standing and its reason: what it gates is that *the instrument separates a
/// correct build from a defective one*. The **scene** stays red, and `crate::scenes` says at length
/// why those are not one claim. Its second row, 67, is `Red` on purpose.
///
/// **Components ticket 10 moved it from thirty-six to forty-one, and one of the five is not a new
/// row.** Rows 62–65 are the four primitives' — the partition sweep, the fill scan, the clear and
/// the hovered chip — and the fifth is **row 61**, the one row on this register whose gate was a
/// statement about an absence. Inverting it rewrote the *gate* and not only the standing, because a
/// row still asserting *the four components do not exist* would now be asserting they are gone.
/// That is the second inversion here worth being suspicious about: a red row phrased as an absence
/// cannot be turned green by editing one field.
/// **Components ticket 20 moved it from a hundred and eight to a hundred and ten**, and one of the
/// two is a new row: row 29 was **pinned red for nine tickets** and went green over the shipped
/// path, and row 129 is criterion 6's join between the gate's subject list and the freeze. A red row
/// inverted by supplying a *subject* is the ordinary case on this register; this one was red on a
/// **defect**, which is why it needed the shipped code to be checked rather than written.
///
/// **Components ticket 22 moved it from a hundred and ten to a hundred and eighteen**, and two of the
/// eight are the last `Unsubjected` rows on §21's own table: rows 24 and 25, which had nothing to run
/// over because `collapsible` did not exist. Six are new, and **three of those are findings rather
/// than criteria** — row 131, where §8's watermark figures are replaced by §9's own sentence on the
/// height axis; row 132, where §8's *0 against 405* turns out to be unreachable from any header
/// gesture; and row 135, where §8's byte pair cannot both be a `size_of` of one type. Row 21 stays
/// `Unreachable` and row 130 is its crate-own form, which is row 41's standing to row 2's.
pub const EVALUATED: usize = 118;

/// Spec §21's register, row for row, and this ticket's gates beside it.
pub const REGISTER: [Row; 135] = [
    // ── spec §21's table, in its order ───────────────────────────────────────────────────────────
    Row {
        number: 1,
        on_spec_table: true,
        gate: "cells marked on a steady frame == 0",
        kind: Kind::Count,
        owner: "C01 — every component",
        section: "spec §20",
        standing: Standing::Unreachable {
            needs: "a damaged-cell count on the engine's public surface. `crates/vitui-engine/src/\
                    damage.rs` is `pub(crate)` from top to bottom, so this is not a re-export the \
                    runtime forgot — `Presented` carries `submitted`, `coalesced` and \
                    `discarded_for_resize`, and no count of cells. It needs a field there and a \
                    `Driver` accessor for it, because this crate cannot name `vitui_engine`. \
                    **Row 48 is the reachable form of this question** and does not invert it: the \
                    engine filters a write whose value equals the resident value, so *writes whose \
                    value differs from what is already there* is computable at the verb boundary \
                    (`crate::runner::Canvas::repaints`) and is the input to the damage this row \
                    wants the output of",
            inverted_by: "runtime architecture issue 22",
        },
    },
    Row {
        number: 2,
        on_spec_table: true,
        gate: "the same screen drawn both ways is equal cell for cell",
        kind: Kind::Equality,
        owner: "C01",
        section: "spec §2",
        standing: Standing::Unreachable {
            needs: "a way to compare two composited surfaces. ADR 0023 is a decision and not an \
                    oversight — no `Surface`, `View`, `Screen` or `Presented` method returns a \
                    cell, a handle or a style bit, and it holds against the engine's own callers \
                    as well as this crate's. A comparison that hands over no cell (an equality \
                    between two surfaces, or a differing-cell count) would survive the ADR",
            inverted_by: "components 37",
        },
    },
    Row {
        number: 3,
        on_spec_table: true,
        gate: "duplicate merges a frame == 0",
        kind: Kind::Count,
        owner: "C01",
        section: "spec §4",
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: "crates/vitui-components/src/counters.rs",
                name: "merges_are_zero_when_a_container_keys_its_children_and_not_before",
            }],
        },
    },
    Row {
        number: 4,
        on_spec_table: true,
        gate: "writes flat 1k -> 1M",
        kind: Kind::Count,
        owner: "C01",
        section: "spec §20",
        // **Subjected by components 12, and row 66 is why this one is not enough on its own.** The
        // engine reports a fully clipped verb as zero columns, so a listing that iterates its whole
        // content writes exactly what a windowed one writes — 3 200 at every volume — while
        // declaring 1 000 001 hit entries against 81. This row is green on that build. It is
        // registered anyway, and beside rather than instead of 66, because *writes flat* is a real
        // claim about the shipped component and the pair is what separates it from the defect.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: LISTING,
                    name: "the_listing_writes_and_declares_the_same_at_a_thousand_rows_and_at_a_\
                           million",
                },
                Instrument::Unit {
                    file: LISTING,
                    name: "a_listing_that_iterates_its_whole_content_writes_the_same_and_declares_\
                           a_thousand_times_more",
                },
            ],
        },
    },
    // **Components ticket 08 inverted this row, and the standing it left was wrong rather than
    // stale.** It read: *`Mods` … is `reachable_as: None`. So no key can be posted and no chord can
    // be pressed.* The premise is true — there is no path to `Mods` from here and
    // `crates/vitui-runtime/src/line.rs` is right about that — and **the conclusion does not
    // follow**: a struct literal needs a *value* for each field, not a name for its type, and
    // `vitui_runtime::keys::Chord` has a `pub mods: Mods` field with three `const` builders that
    // set it. `Chord::key('s').ctrl().mods` **is** `Mods::CTRL`, written without the word. See
    // `crate::keys`'s header, which carries the whole argument and the two barriers that survive it.
    Row {
        number: 5,
        on_spec_table: true,
        gate: "a chord pressed into every focusable types nothing",
        kind: Kind::Count,
        owner: "C01, C06",
        section: "spec §11",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/keys.rs",
                    name: "a_chord_pressed_into_every_focusable_types_nothing",
                },
                // The population, named rather than assumed: seven rows of the freeze, none of
                // which is declared in this crate — so what this row stands over is seven sinks,
                // and the row says so instead of implying seven components.
                Instrument::Unit {
                    file: "crates/vitui-components/src/keys.rs",
                    name: "every_text_bearing_name_is_in_the_freeze",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/keys_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 6,
        on_spec_table: true,
        gate: "`writes == distinct cells touched` (no cell twice)",
        kind: Kind::Equality,
        owner: "C02",
        section: "spec §2",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/counters.rs",
                    name: "writes_equals_distinct_until_two_verbs_overlap",
                },
                // Components ticket 06: the row stops being a property of the instrument and
                // becomes a **gate over the two helpers every other component writes through**.
                Instrument::Unit {
                    file: "crates/vitui-components/src/text.rs",
                    name: "fit_writes_a_partition_of_its_row_at_every_width_and_justification",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/frame.rs",
                    name: "block_writes_each_cell_once_and_never_the_interior",
                },
                // And over a whole screen built out of nothing but the two, which is the half a
                // per-helper test cannot reach: the interesting double writes are between two
                // branches that are each correct alone.
                Instrument::Unit {
                    file: "crates/vitui-components/src/form.rs",
                    name: "neither_helper_writes_a_cell_twice_at_either_density",
                },
                // Watched firing, in both directions, on ADR 0026's 15-cell instance.
                Instrument::Unit {
                    file: "crates/vitui-components/src/frame.rs",
                    name: "a_border_run_written_over_its_own_title_is_fifteen_double_writes",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/form.rs",
                    name: "the_naive_arm_writes_forty_thousand_cells_it_had_already_written",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/partition_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 7,
        on_spec_table: true,
        gate: "every cell of the rectangle written at least once (no cell never) — the sentinel",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §2",
        standing: Standing::Red {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/counters.rs",
                    name: "the_sentinel_panics_rather_than_reporting_no_survivors",
                },
                // **The `Rgb` barrier lifted with runtime architecture issue 22 and is struck from
                // this list rather than left pointing at the line.** `name: "Rgb",` is still in
                // `ENGINE_NAMES` and the row below it now reads `reachable_as: Some(..)`, so the
                // citation would have gone on passing while meaning the opposite — which is a
                // `Barrier` decaying into exactly the citation this arm was added to stop being.
                // `Theme::custom(fg: Rgb, bg: Rgb)` is callable from this crate now, so the stamp
                // can be minted; two of `sentinel`'s three barriers remain and the row stays red.
                Instrument::Barrier {
                    file: "docs/adr/0023-the-cell-is-never-visible-in-the-public-api.md",
                    line: "# The cell is never visible in the public API",
                },
            ],
            failing: "9 956 cells of 53 280 (18.7%) over six panels of twelve: the chip screen \
                      4 189, the preview pane 2 159, the scroll area 1 799, media 1 286, the chart \
                      496, the collection pair 27. And the detector itself is still unreachable \
                      here — `crate::counters::sentinel` named three barriers, issue 22 lifted the \
                      first (`Rgb`), and the readback ADR 0023 forbids is the one that matters",
            inverted_by: "components 40",
        },
    },
    Row {
        number: 8,
        on_spec_table: true,
        gate: "no cell keeps the previous palette a frame after a swap",
        kind: Kind::Count,
        owner: "C11 / R20 §3",
        section: "spec §16",
        standing: Standing::Red {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/tests/gates.rs",
                    name: "changed_greater_than_zero_passes_on_the_exact_set_it_had_to_catch",
                },
                Instrument::Barrier {
                    file: "docs/adr/0023-the-cell-is-never-visible-in-the-public-api.md",
                    line: "# The cell is never visible in the public API",
                },
            ],
            failing: "six panels of twelve keep the old palette permanently, because the rule *a \
                      memo carries the theme in its key iff its value is made of paints or glyphs* \
                      has no caller. The gate that was there asserted `changed > 0`, which one cell \
                      of 4 800 satisfies while 3 583 are wrong. Both halves are unrunnable here: \
                      the surface count needs the readback, and the memo enumeration has 0 memos \
                      to enumerate",
            inverted_by: "components 41",
        },
    },
    Row {
        number: 9,
        on_spec_table: true,
        gate: "regions identical at 1k and 1M",
        kind: Kind::Equality,
        owner: "C03",
        section: "spec §5",
        // **§21's own spelling of what row 66 measures**, and components 12 gave it a subject: the
        // listing's two arms are `crate::collect::collection_into` and its `defective` twin, so the
        // equality is over the component rather than over a stand-in row loop.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: LISTING,
                    name: "the_listing_writes_and_declares_the_same_at_a_thousand_rows_and_at_a_\
                           million",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "two_collections_on_one_screen_declare_two_entries_and_merge_nothing",
                },
            ],
        },
    },
    Row {
        number: 10,
        on_spec_table: true,
        gate: "`size_of::<CollState>()` independent of length",
        kind: Kind::Count,
        owner: "C03",
        section: "spec §5",
        // Subjected by components 12. §5's 208 does not reproduce — see
        // `crate::collect::COLL_STATE_BYTES`, which says why rather than padding the struct — and
        // what this row asks is the invariance, which does.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: COLLECT,
                name: "the_state_is_the_same_size_at_every_length",
            }],
        },
    },
    Row {
        number: 11,
        on_spec_table: true,
        gate: "writes and verbs identical across declared column counts",
        kind: Kind::Equality,
        owner: "C04",
        section: "spec §6",
        standing: Standing::Unsubjected {
            inverted_by: "components 15",
        },
    },
    Row {
        number: 12,
        on_spec_table: true,
        gate: "the same table under a horizontal offset, against itself",
        kind: Kind::Equality,
        owner: "C04",
        section: "spec §6",
        standing: Standing::Unsubjected {
            inverted_by: "components 15",
        },
    },
    Row {
        number: 13,
        on_spec_table: true,
        gate: "an interval edit on the selection store against a bit vector",
        kind: Kind::Equality,
        owner: "C05",
        section: "spec §10",
        // **A bit's position *is* its index**, so a splice moves every bit after the interval; a
        // sorted span list moves the spans that meet it. One against nine hundred thousand at a
        // million rows, which is the difference between a store proportional to the gestures and
        // one proportional to the rows — on the single operation a bit vector cannot do cheaply,
        // having been chosen for the two it can.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: ORDER,
                    name: "a_splice_cannot_shatter_a_span_list_and_a_permutation_always_does",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/order_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 14,
        on_spec_table: true,
        gate: "a fold/unfold round trip, and `splice == rebuild`",
        kind: Kind::Equality,
        owner: "C05",
        section: "spec §7",
        // **`Evaluated` over the forest's stand-in index rather than over `tree`**, which is
        // components ticket 04's standing and its reason: what this row gates is that *the
        // instrument separates a correct build from a defective one*, and both halves are watched
        // doing it. `splice == rebuild` is compared against a walk of the forest — the slow
        // producer §7 keeps only as an oracle — and the round trip is run over **both** spellings
        // of the splice, the second being the one the prototype shipped, which comes back with
        // 349 526 rows of 500 000 while the run count, the timing and the screen are all
        // identical. The **scenes** stay red; `crate::scenes` says why those are not one claim.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: FOREST,
                    name: "a_spliced_index_equals_a_rebuilt_one",
                },
                Instrument::Unit {
                    file: FOREST,
                    name: "a_fold_and_an_unfold_restore_the_selection_exactly_and_the_shipped_\
                           splice_did_not",
                },
                Instrument::Unit {
                    file: FOREST,
                    name: "a_half_tree_fold_parks_one_run_of_sixteen_bytes",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/tree_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 15,
        on_spec_table: true,
        gate: "the caret is always on a cluster boundary",
        kind: Kind::Invariant,
        owner: "C06",
        section: "spec §11",
        standing: Standing::Unsubjected {
            inverted_by: "components 24",
        },
    },
    Row {
        number: 16,
        on_spec_table: true,
        gate: "the caret's column equals the engine's tables over the prefix",
        kind: Kind::Equality,
        owner: "C06",
        section: "spec §11",
        standing: Standing::Unsubjected {
            inverted_by: "components 24",
        },
    },
    Row {
        number: 17,
        on_spec_table: true,
        gate: "a spliced wrap index equals a rebuilt one",
        kind: Kind::Equality,
        owner: "C06",
        section: "spec §11",
        standing: Standing::Unsubjected {
            inverted_by: "components 24",
        },
    },
    Row {
        number: 18,
        on_spec_table: true,
        gate: "the index's recorded width equals the width being drawn",
        kind: Kind::Equality,
        owner: "C06",
        section: "spec §11",
        standing: Standing::Unsubjected {
            inverted_by: "components 24",
        },
    },
    Row {
        number: 19,
        on_spec_table: true,
        gate: "`verbs <= writes`",
        kind: Kind::Relation,
        owner: "C08",
        section: "spec §20",
        // **Components ticket 27 gave this row the screen it is about.** It had been evaluated over
        // `crate::counters`' own fixtures, which is the mechanism; the series screen is where the
        // relation earns its wording — 2 554 / 3 775 / 2 475 verbs at 1 000 / 100 000 / 1 000 000
        // points on one rectangle, **not monotone in `n`**, so a verb *equality* across sizes would
        // be a gate on a number that belongs to the picture.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/counters.rs",
                    name: "verbs_never_exceed_writes_and_the_relation_is_not_an_equality",
                },
                Instrument::Unit {
                    file: SERIES,
                    name: "the_frame_costs_the_rectangle_and_the_verbs_track_the_picture",
                },
                Instrument::Unit {
                    file: SERIES,
                    name: "the_verb_count_rises_and_falls_with_the_picture_and_not_with_the_volume",
                },
            ],
        },
    },
    Row {
        number: 20,
        on_spec_table: true,
        gate: "the bar fixpoint over 5 475 600 pairs",
        kind: Kind::Count,
        owner: "C13",
        // **Two misattributions, corrected by components ticket 18 and recorded rather than
        // quietly fixed.** This row read `spec §13` and `components 28`, and both belong to the
        // chart: §13 is `chart` and `plot`, ticket 28 builds them, and the rows immediately above
        // this one are theirs. C13's bar fixpoint is **spec §9** and it is **components 19** that
        // builds its subject. The shape is worth naming because it is not a typo: a row copied
        // from its neighbour inherits the neighbour's citation, and every field of it looks
        // plausible.
        section: "spec §9",
        // §21's own table records this as one of the **two** gates that were being run at the
        // branch point, and it is green here for the reason it was green there: it is arithmetic
        // over a domain, and the domain is visited exhaustively rather than sampled. The **scene**
        // stays red — `crate::scenes` says at length why a gate and a screen are not one claim.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: AREA,
                    name: "the_bar_decision_is_a_fixpoint_over_five_million_pairs",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "a_bar_that_is_not_needed_is_not_shown",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/area_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 21,
        on_spec_table: true,
        gate: "the surface after a collapse equals a freshly built one",
        kind: Kind::Equality,
        owner: "C14",
        section: "spec §8",
        standing: Standing::Unreachable {
            needs: "the same surface comparison row 2 needs, for the same reason (ADR 0023). §21 \
                    also records that the neighbouring green form should not be banked: written \
                    against a terminal resize it passes because `Gallery::resize` allocates a new \
                    `Surface` and the residue has nowhere to survive, which tests the resize path \
                    and not the defect",
            inverted_by: "components 37",
        },
    },
    Row {
        number: 22,
        on_spec_table: true,
        gate: "a closed body declares no ring entry and no hit entry — the mechanism form of the \
               tab-stop gate",
        kind: Kind::Count,
        owner: "C14",
        section: "spec §8",
        // **Components ticket 21 put §8's own accordion under it.** `tests/gates.rs` states the
        // rule at twelve sections of five focusables — 72 against 12, the same shape — and the
        // accordion is the shape at §8's own scale: 421 hit entries against 13 and 420 tab stops
        // against 12, which is 408 on both columns and §8's `478 - 70 == 423 - 15` exactly.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/tests/gates.rs",
                    name: "a_component_drawn_into_a_zero_height_rectangle_declares_no_tab_stops",
                },
                Instrument::Unit {
                    file: ACCORDION,
                    name: "a_closed_body_is_not_called_and_the_h_zero_spelling_declares_four_\
                           hundred_and_eight_more",
                },
                Instrument::Unit {
                    file: ACCORDION,
                    name: "the_two_surfaces_are_identical_and_no_counter_that_reads_a_cell_can_\
                           see_it",
                },
            ],
        },
    },
    Row {
        number: 23,
        on_spec_table: true,
        gate: "a fold anchored on a position is reconciled",
        kind: Kind::Count,
        owner: "C14",
        section: "spec §8",
        // **`Unsubjected` until components ticket 21, and it did not need `collapsible` after
        // all.** §8 is explicit that the closed folds are *caller state* — a `Vec<u32>` beside a
        // document, for the same forced reason a collection's selection is — so the whole of this
        // gate is standable with no component built: 4 166 of 4 167 folds on a line that opens no
        // block against 0 reanchored, both §8's own numbers, both exact.
        //
        // `Evaluated` over the fold set rather than over `collapsible`, which is components ticket
        // 04's standing and its reason: what it gates is that *the instrument separates a correct
        // build from a defective one*, watched in both directions. The **scene** stays red, because
        // a scene is a screen and this is not one.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: ACCORDION,
                    name: "four_thousand_one_hundred_and_sixty_six_folds_of_four_thousand_one_\
                           hundred_and_sixty_seven_land_wrong",
                },
                Instrument::Unit {
                    file: ACCORDION,
                    name: "the_folds_that_survive_are_exactly_the_ones_above_the_edit",
                },
                Instrument::Unit {
                    file: ACCORDION,
                    name: "the_document_opens_sixteen_thousand_blocks_and_a_quarter_of_them_are_\
                           closed",
                },
            ],
        },
    },
    Row {
        number: 24,
        on_spec_table: true,
        gate: "the inplace map round-trips",
        kind: Kind::Equality,
        owner: "C14",
        section: "spec §8",
        // **Components ticket 22 supplied the subject**, and the equality is between two spellings
        // rather than between one and itself: §8's *one open detail row is two comparisons and a
        // subtraction* is `Inplace::one`, C05's `ytop` is `Inplace::many`, and both directions of the
        // map are asserted over the whole length on each — then the two are asserted equal on the
        // one-open case, which is what makes *needs no prefix sum* a claim about cost rather than a
        // second answer.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DISCLOSE,
                    name: "the_inplace_map_round_trips_and_one_open_row_needs_no_prefix_sum",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/collapsible_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 25,
        on_spec_table: true,
        gate: "`open` is never ambiguous mid-transition",
        kind: Kind::Invariant,
        owner: "C14",
        section: "spec §8",
        // **An invariant needs both halves of *there is no third state*, and they are two different
        // kinds of check.** Behaviourally, `Collapse::set` writes `open` at the instant the gesture
        // lands and only the height moves — asserted at every frame of a two-hundred-millisecond
        // collapse, which is what an `Invariant` means. Structurally, a **source scan** for a stored
        // third state, because *the item does not exist* has no expression and a `compile_fail`
        // naming a variant nobody built passes today and passes again the day somebody adds one.
        //
        // The scan is watched finding a declaration when there is one, and the two words it looks
        // for are assembled from fragments rather than written out — a scanner that named its own
        // needle would report the module it is defending, which is `crate::frame`'s own first run.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DISCLOSE,
                    name: "open_is_never_ambiguous_over_every_frame_of_a_two_hundred_\
                           millisecond_collapse",
                },
                Instrument::Unit {
                    file: DISCLOSE,
                    name: "there_is_no_stored_third_state_and_the_scan_finds_one_when_there_is",
                },
                Instrument::Unit {
                    file: ACCORDION,
                    name: "a_two_hundred_millisecond_collapse_is_monotone_and_goes_quiet_at_the_\
                           duration",
                },
            ],
        },
    },
    Row {
        number: 26,
        on_spec_table: true,
        gate: "the glyph-set name appears zero times in `vitui-components`, and no private \
               fallback-table module exists",
        kind: Kind::Count,
        owner: "C09, C10",
        section: "spec §16",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/gates.rs",
                    name: "no_component_here_names_a_glyph_set_and_none_has_a_private_missing_table",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/inventory.rs",
                    name: "no_component_source_names_the_repertoire",
                },
                // **The exception, named rather than the gate loosened** — refinement 3. Somebody
                // has to *declare* three rungs to sweep them, so the count over `src/` is joined to
                // a list of the two files outside it that may carry the needle. A third one is a
                // failing test rather than a drift back to twenty-four occurrences.
                Instrument::Unit {
                    file: "crates/vitui-components/tests/glyph_matrix.rs",
                    name: "the_axis_is_named_in_three_files_and_one_of_them_is_a_components",
                },
            ],
        },
    },
    Row {
        number: 27,
        on_spec_table: true,
        gate: "within-component cross-family glyph collapse == 0 at every rung",
        kind: Kind::Count,
        owner: "C10, corrected by C11",
        section: "spec §16",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/tests/glyph_matrix.rs",
                    name: "within_component_cross_family_collapse_is_zero_at_every_rung",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/glyphs.rs",
                    name: "the_detector_finds_c09s_collapse_and_not_a_corner_on_a_corner",
                },
            ],
        },
    },
    Row {
        number: 28,
        on_spec_table: true,
        gate: "a too-long label at ASCII does not end in `ArrowRight`",
        kind: Kind::Equality,
        owner: "C09",
        section: "spec §16",
        standing: Standing::Unreachable {
            needs: "the drawn cell. §21 is explicit that the end-to-end form is the gate and the \
                    source count is the symptom, and the end-to-end form is a readback (ADR 0023). \
                    The defect it exists to catch is C09's: the shadow table's ASCII ellipsis was \
                    `>`, exactly an ASCII `ArrowRight`, so 468 truncated labels ended in the \
                    collapsed-node marker and `tree` draws both",
            inverted_by: "components 37",
        },
    },
    Row {
        number: 29,
        on_spec_table: true,
        gate: "twenty wheel clicks move the offset twenty",
        kind: Kind::Count,
        owner: "C07, C10",
        section: "spec §12",
        // **Components ticket 20 took both substitutions out, and the row went green.**
        //
        // It was red for nine tickets, and the failing set it was red on was the defect rather than
        // a missing subject: `CONTEXT.md` forbids the unconditional scroll-into-view and **four
        // *resolved* tickets wrote it anyway**, each by someone who had read the rule. What kept the
        // row from being written over the shipped path was two stand-ins, and each was honest at the
        // time it was written:
        //
        // - **the click.** `Driver::post_mouse` takes a `vitui_engine::Mouse`, and *"a `Mouse` needs
        //   a `Buttons` and a `MouseKind`, neither of which is in `ENGINE_NAMES` at all"* — this
        //   row's own recorded sentence, and the one runtime architecture issue 22 acted on. The
        //   delta was handed straight to the arithmetic `Response::scrolled` would have delivered it
        //   to, on both arms. It is a posted notch now, routed through the previous frame's index.
        // - **the subject.** A row loop written beside the gate, because `collection` did not exist.
        //   It is `collection_into` and `crate::scroll::scroll_area` now, against
        //   `collect::defective::every_frame` and `never_reveals` — the shipped build with one value
        //   changed, so a reviewer's diff is one line.
        //
        // **Neither stand-in was on the side of either arm, and the pinned figure did not move**:
        // `0 against 20` is what components 11 reported and what the posted click reports. What they
        // cost was the *second subject and the second axis*, which is where the finding is — a body
        // dead downward is alive sideways, and `scroll_area` at `Along::Rows` settles a vertical
        // click at 0 and a horizontal one at 20 while the transpose does the opposite. One number
        // for *the offset* cannot say either, which is why criterion 4 asks for the two subjects
        // separately.
        //
        // **The rule's second clause is asserted here for the first time**, over the shipped
        // component: *a press already proves the widget was on screen*, so a press selects a row at
        // a scrolled offset and pulls nothing. It could only ever be asserted at a **non-zero**
        // offset, and the unconditional arm cannot be watched failing it — by the frame the press
        // edge lands on it has already converged, so the loudest arm is silent for the quietest
        // reason. That is the defect and not an escape from it, and the comparison is made on a
        // frame with no gesture on it at all.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: WHEEL,
                    name: "twenty_posted_clicks_move_a_collections_offset_twenty_and_an_unconditional_\
                           reveal_takes_it_back",
                },
                Instrument::Unit {
                    file: WHEEL,
                    name: "a_body_dead_downward_is_alive_sideways_and_one_offset_cannot_say_so",
                },
                Instrument::Unit {
                    file: WHEEL,
                    name: "a_press_selects_a_row_and_does_not_pull_the_viewport_to_it",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/tests/gates.rs",
                    name: "a_frame_that_asks_for_no_reveal_moves_no_offset_and_one_that_asks_does",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/wheel_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 30,
        on_spec_table: true,
        gate: "O1-O5, as queries over `INVENTORY`",
        kind: Kind::Count,
        owner: "C10",
        section: "spec §17",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/obligations.rs",
                    name: "not_one_of_the_five_obligations_is_met",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/obligations.rs",
                    name: "each_query_reports_the_population_it_could_not_answer_for",
                },
            ],
        },
    },
    Row {
        number: 31,
        on_spec_table: true,
        gate: "a `Task` is neither `Send` nor `Sync`, and a `Worker` is both",
        kind: Kind::CompileOutcome,
        owner: "C16 (G10, G10b)",
        section: "spec §17",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Pair {
                    file: "crates/vitui-components/src/gates.rs",
                    hostile: "assert_sync::<Task<u32>>()",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/gates.rs",
                    name: "the_app_threads_half_cannot_cross_a_thread_and_the_workers_half_can",
                },
            ],
        },
    },
    Row {
        number: 32,
        on_spec_table: true,
        gate: "zero allocations in a steady frame, as a total",
        kind: Kind::Count,
        owner: "all",
        section: "spec §20",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/tests/gates.rs",
                    name: "a_steady_component_frame_allocates_nothing_as_a_total_over_the_run",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/tests/gates.rs",
                    name: "a_total_sees_one_frame_in_two_hundred_and_a_mean_cannot",
                },
                // Components ticket 06: the same budget, on the two helpers every component will
                // write through. `crate::ink::Direct` stages into the frame's own buffer instead of
                // materialising a run, and this is the counter behind that sentence.
                Instrument::Unit {
                    file: "crates/vitui-components/tests/gates.rs",
                    name: "the_two_partition_helpers_allocate_nothing_as_a_total_over_the_run",
                },
                // **Components ticket 10: the same budget on a frame of components**, which is the
                // first time this row has had one to run over. Fifty frames of the 338-region
                // screen, drawn through `text`, `chip`, `button` and `panel` with `Direct`, and the
                // assertion is on the total — one allocation on one of the fifty fails it.
                Instrument::Unit {
                    file: "crates/vitui-components/tests/budget.rs",
                    name: "a_steady_frame_of_the_dense_screen_allocates_nothing_as_a_total",
                },
                // **Components ticket 22: the same budget over a frame that is not steady**, which
                // is the one shape this row had never been asked about. Every frame of a transition
                // hands the body a **different rectangle**, so the warm-two-frames discipline every
                // other window here uses does not warm it: a height nothing has drawn yet is a first
                // touch *inside* the window, and the arm warmed that way reads 1 allocation over 12
                // frames — amortised zero, and exactly the shape `crate::counters::Allocations`
                // refuses to average away. Warmed on the *shape* it is zero, and the test's own
                // header says which warming it uses and why.
                Instrument::Unit {
                    file: "crates/vitui-components/tests/budget.rs",
                    name: "a_two_hundred_millisecond_collapse_allocates_nothing_over_its_own_frames",
                },
            ],
        },
    },
    // ── this ticket's rows ───────────────────────────────────────────────────────────────────────
    Row {
        number: 33,
        on_spec_table: false,
        gate: "tab stops == visible focusables — the geometric form C14 asked for",
        kind: Kind::Equality,
        owner: "C14",
        section: "spec §21",
        standing: Standing::Unreachable {
            needs: "a ring entry that says what is on **screen**. §21's stated reason is the \
                    prototype's — `RingEnt::rect` zero unless `Frame::ring_geometry` is on — and \
                    the shipped `vitui_runtime::focus::Stop` carries a populated `rect` on every \
                    entry, which this crate can read. The conclusion holds for a better reason: \
                    the rectangle recorded is the one the widget **declared**, translated by the \
                    enclosing scroll offset and intersected with nothing — not the `Ctx::child` \
                    clip, not the scroll area's viewport, not an overlay standing over it. \
                    Deciding *visible* needs a clip intersection the ring does not do",
            inverted_by: "components 39",
        },
    },
    Row {
        number: 34,
        on_spec_table: false,
        gate: "the walk repeats no id, and reaches every stop unless a trap is standing",
        kind: Kind::Count,
        owner: "C11 — refinement 3",
        section: "spec §21",
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: "crates/vitui-components/tests/gates.rs",
                name: "the_walk_repeats_no_id_and_reaches_every_stop_unless_a_trap_is_standing",
            }],
        },
    },
    Row {
        number: 35,
        on_spec_table: false,
        gate: "`vitui-alloc-probe` is the only counting allocator in the workspace",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §19",
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: "crates/vitui-components/src/gates.rs",
                name: "vitui_alloc_probe_is_the_only_counting_allocator_in_the_workspace",
            }],
        },
    },
    Row {
        number: 36,
        on_spec_table: false,
        gate: "a zeroed allocation is counted exactly once, like any other",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §21",
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: "crates/vitui-components/tests/gates.rs",
                name: "a_zeroed_allocation_is_counted_exactly_once_like_any_other",
            }],
        },
    },
    Row {
        number: 37,
        on_spec_table: false,
        gate: "this lineage carries at least one `*_numbers.rs` example, against the runtime's 19",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §21",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/gates.rs",
                    name: "this_lineage_has_at_least_one_numbers_example",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/gates_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 38,
        on_spec_table: false,
        gate: "eight of §20's nine per-frame counters are readable here, and the ninth fails loudly",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §20",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/counters.rs",
                    name: "eight_of_the_nine_counters_are_reachable_and_marked_is_not",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/counters.rs",
                    name: "marked_panics_rather_than_answering_zero",
                },
                Instrument::Barrier {
                    file: "crates/vitui-engine/src/damage.rs",
                    line: "pub(crate) fn clear(&mut self) -> usize {",
                },
            ],
        },
    },
    Row {
        number: 39,
        on_spec_table: false,
        gate: "`changed > 0` passes on the exact set it had to catch — refinement 1's negative case",
        kind: Kind::Count,
        owner: "C11 — refinement 1",
        section: "spec §21",
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: "crates/vitui-components/tests/gates.rs",
                name: "changed_greater_than_zero_passes_on_the_exact_set_it_had_to_catch",
            }],
        },
    },
    Row {
        number: 40,
        on_spec_table: false,
        gate: "a verb that leaves its rectangle is reported short by the engine",
        kind: Kind::Relation,
        owner: "C02",
        section: "spec §2",
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: "crates/vitui-components/src/counters.rs",
                name: "a_verb_that_runs_off_its_context_is_reported_short_by_the_engine",
            }],
        },
    },
    Row {
        number: 41,
        on_spec_table: false,
        gate: "two draw implementations of one scene are equal cell for cell, at the verb boundary",
        kind: Kind::Equality,
        owner: "C01",
        section: "spec §21",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/runner.rs",
                    name: "the_same_screen_drawn_both_ways_is_equal_cell_for_cell",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/runner.rs",
                    name: "the_runner_catches_an_inverted_scroll_sign",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/runner.rs",
                    name: "the_runner_catches_twenty_wheel_clicks_that_move_nothing",
                },
                // **Why the reference arm is this crate's and not engine ticket 04's**, as a line
                // rather than as an argument. `issues/04` asks for *engine 04's reference
                // compositor* and names none of the three barriers: the module is declared `mod`
                // and not `pub mod`, so it is unreachable from any other crate at all; it is
                // `#[cfg(any(test, feature = "fuzz"))]`, so no dependent ever compiles it; and this
                // crate cannot name `vitui_engine` under any circumstances (row 2's `needs`). The
                // day that line becomes `pub mod reference;` this test fails and the first barrier
                // has lifted — which is a change of standing and not a broken test.
                Instrument::Barrier {
                    file: "crates/vitui-engine/src/lib.rs",
                    line: "mod reference;",
                },
            ],
        },
    },
    Row {
        number: 42,
        on_spec_table: false,
        gate: "content shrinking inside a rectangle that does not move leaves no stale tail",
        kind: Kind::Count,
        owner: "C11 — §21's unbanked gate",
        section: "spec §21",
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: "crates/vitui-components/src/runner.rs",
                name: "the_runner_catches_a_stale_tail_after_content_shrinks_inside_a_rectangle_\
                       that_does_not_move",
            }],
        },
    },
    Row {
        number: 43,
        on_spec_table: false,
        gate: "a scene's content survives a narrowing, and a size that wrote nothing is refused",
        kind: Kind::Equality,
        owner: "C08",
        section: "spec §13",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/runner.rs",
                    name: "a_construction_that_changes_with_the_width_is_green_wide_and_red_narrow",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/runner.rs",
                    name: "a_two_size_comparison_over_a_blank_arm_is_refused",
                },
            ],
        },
    },
    Row {
        number: 44,
        on_spec_table: false,
        gate: "§21's scene list is a value, and it is the same length as §21's table",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §21",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/scenes.rs",
                    name: "the_specs_table_is_twenty_seven_rows_and_this_list_carries_them_all",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/scenes.rs",
                    name: "seventeen_of_the_thirty_four_axis_obligations_have_a_scene_and_\
                           seventeen_do_not",
                },
            ],
        },
    },
    // ── components ticket 05's row, and it is a barrier rather than a gate ────────────────────────
    Row {
        number: 45,
        on_spec_table: false,
        gate: "§16's matrix at all nine cells — the repertoire axis against the colour axis",
        kind: Kind::Count,
        owner: "C09, C10, C11",
        section: "spec §16",
        standing: Standing::Unreachable {
            needs: "`ColorDepth`. **The repertoire half is measured here in full** — glyph pairs \
                    0 / 0 / 36 of 190, every ASCII collapse inside the box family, and the \
                    within-component cross-family gate at all three rungs — because a `GlyphSet` \
                    is nameable through the runtime's re-export. The colour half is not: \
                    `Theme::resolve` takes a `ColorDepth`, `crates/vitui-runtime/src/line.rs` \
                    files it `reachable_as: None`, and the only tier a crate whose dependency list \
                    is `vitui-runtime` and nothing else can hold is the one a theme arrives already \
                    resolved for. So the role column, the distinctions-lost column and the traffic \
                    light at sixteen colours are measured in the runtime, which owns the mechanism \
                    and may name both axes: `examples/theme_numbers.rs` prints them",
            inverted_by: "runtime architecture issue 22",
        },
    },
    // ── components ticket 06's two ───────────────────────────────────────────────────────────────
    Row {
        number: 46,
        on_spec_table: false,
        gate: "`fit` against the same order written by hand is 0 cells apart at equal write counts",
        kind: Kind::Equality,
        owner: "C02",
        section: "spec §3",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/form.rs",
                    name: "routing_through_fit_is_zero_cells_different_at_equal_write_counts",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/form.rs",
                    name: "a_block_that_clears_what_it_hands_over_writes_sixteen_thousand_cells_\
                            twice",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/partition_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 47,
        on_spec_table: false,
        gate: "`frame::focus_ring` does not exist, and a twin names `frame::block` by path",
        kind: Kind::CompileOutcome,
        owner: "C02, C03",
        section: "spec §3",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Pair {
                    file: "crates/vitui-components/src/frame.rs",
                    hostile: "use vitui_components::frame::focus_ring;",
                },
                // The pair catches the item coming back on the **public** surface. This catches it
                // coming back as a private one, which is how a deleted helper actually returns.
                Instrument::Unit {
                    file: "crates/vitui-components/src/frame.rs",
                    name: "no_source_file_in_this_crate_declares_a_focus_ring",
                },
            ],
        },
    },
    // ── components ticket 07's five ──────────────────────────────────────────────────────────────
    Row {
        number: 48,
        on_spec_table: false,
        gate: "a pointer resting on a chip for sixty frames changes 0 cells through `press`, \
               against 8 written by hand",
        kind: Kind::Count,
        owner: "C02",
        section: "spec §3",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/state.rs",
                    name: "a_pointer_resting_for_sixty_frames_re_damages_nothing_through_press",
                },
                // The half that makes it structural rather than measured: the role awarded and the
                // role returned come out of one field, on all eight pointer paths.
                Instrument::Unit {
                    file: "crates/vitui-components/src/state.rs",
                    name: "the_face_awarded_is_the_face_a_hovered_widget_draws_on_every_path",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/state.rs",
                    name: "the_hover_award_is_declared_in_one_file_and_it_is_the_one_that_\
                            collapses_it",
                },
                // The instrument's own correctness, in both directions: applied before the draw
                // instead of after it, the answer inverts.
                Instrument::Unit {
                    file: "crates/vitui-components/src/state.rs",
                    name: "the_award_is_applied_after_the_draw_and_not_before",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/press_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 49,
        on_spec_table: false,
        gate: "`state::PressState` does not exist, and a twin names `state::press` by path",
        kind: Kind::CompileOutcome,
        owner: "C02",
        section: "spec §3",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Pair {
                    file: "crates/vitui-components/src/state.rs",
                    hostile: "use vitui_components::state::PressState;",
                },
                // The pair catches the type coming back on the **public** surface. This catches it
                // coming back as a private one, which is how a deleted type actually returns.
                Instrument::Unit {
                    file: "crates/vitui-components/src/state.rs",
                    name: "no_source_file_in_this_crate_declares_a_press_state",
                },
            ],
        },
    },
    Row {
        number: 50,
        on_spec_table: false,
        gate: "`face_paint`'s precedence holds at all 32 states a `Face`'s five bits can be in",
        kind: Kind::Equality,
        owner: "C02, C03",
        section: "spec §3",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/frame.rs",
                    name: "the_precedence_holds_over_all_thirty_two_states",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/frame.rs",
                    name: "face_paint_resolves_through_the_same_ladder_at_every_state",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/frame.rs",
                    name: "a_face_is_five_independent_bools_in_five_bytes",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/press_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 51,
        on_spec_table: false,
        gate: "a scrollbar's `writes == distinct` on a two-bar screen: 0, against the two thumbs \
               track-first",
        kind: Kind::Equality,
        owner: "C02",
        section: "spec §3",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/scroll.rs",
                    name: "a_track_written_under_its_own_thumb_is_two_hundred_and_twenty_four_\
                            double_writes",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/scroll.rs",
                    name: "a_bar_writes_each_cell_once_at_every_length_and_offset",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/press_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 52,
        on_spec_table: false,
        gate: "the three helpers of components 07 build no style and name no glyph repertoire",
        kind: Kind::Count,
        owner: "C02, C03",
        section: "spec §3",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/state.rs",
                    name: "the_three_helpers_carry_no_style_literal_and_name_no_glyph_set",
                },
                // The crate-wide half of the same question, which row 26 already runs: this row is
                // the three files, that one is every file and every private fallback table.
                Instrument::Unit {
                    file: "crates/vitui-components/src/inventory.rs",
                    name: "no_component_source_names_the_repertoire",
                },
            ],
        },
    },
    // ── components ticket 08's rows ──────────────────────────────────────────────────────────────
    Row {
        number: 53,
        on_spec_table: false,
        gate: "`keys::text` is `CTRL | ALT` and R12's `INTENT` is exactly one bit wider",
        kind: Kind::Equality,
        owner: "C01, C06",
        section: "spec §3",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/keys.rs",
                    name: "the_mask_is_ctrl_and_alt_and_shift_is_not_in_it",
                },
                // A mask nobody reads and a predicate nobody compares it against are two rules that
                // drift, so the decomposition is gated against the constant rather than trusted.
                Instrument::Unit {
                    file: "crates/vitui-components/src/keys.rs",
                    name: "the_mask_and_the_predicate_are_one_statement",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/keys.rs",
                    name: "the_predicate_agrees_with_the_mask_on_every_reachable_state",
                },
                // **The barrier that survived the inversion of row 5 has now lifted too.** It read:
                // eight of the 256 modifier states are constructible here, because `Chord` has three
                // builders and `Mods` has eight bits — so the *exhaustive table* is short even
                // though the rule is not. Runtime architecture issue 22 re-exported `Mods`, so all
                // 256 are constructible and `the_predicate_agrees_with_the_mask_on_every_reachable_\
                // state` is now a statement about every state rather than about the eight a `Chord`
                // could spell. The citation is struck rather than kept: `name: "Mods",` is still in
                // `ENGINE_NAMES` and now reads `reachable_as: Some(..)` beneath, so it would have
                // gone on passing while meaning the opposite.
            ],
        },
    },
    Row {
        number: 54,
        on_spec_table: false,
        gate: "the three readings of a key give three different answers on the same two keys",
        kind: Kind::Equality,
        owner: "C01, C02, C06",
        section: "spec §3",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/keys.rs",
                    name: "the_three_way_measurement_over_a_focused_field",
                },
                // Spec §3 changes the input mid sentence and this is why: on `Ctrl+S, h, i` the
                // `INTENT` reading is indistinguishable from the fix, and its defect is only
                // visible on a capital.
                Instrument::Unit {
                    file: "crates/vitui-components/src/keys.rs",
                    name: "a_capital_is_not_a_chord_and_the_intent_reading_says_it_is",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/keys.rs",
                    name: "the_textarea_arithmetic_is_two_and_zero_against_one_and_one",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/keys_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 55,
        on_spec_table: false,
        gate: "a type-ahead buffer survives to the next keypress, and the census names the widget \
               that asked",
        kind: Kind::Count,
        owner: "C02",
        section: "spec §3",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/nav.rs",
                    name: "the_buffer_survives_to_the_next_keypress_rather_than_expiring_at_it",
                },
                // **The direction that matters**: the two arms agree about what was typed on every
                // frame a key arrives on, so no assertion about the buffer can separate them. What
                // separates them is that nothing woke the screen at the moment the buffer lapsed.
                Instrument::Unit {
                    file: "crates/vitui-components/src/nav.rs",
                    name: "without_the_deadline_nothing_wakes_the_screen_when_the_buffer_lapses",
                },
                // The ticket's own correction to itself, as a number: 1 against 0 in the census,
                // with the same one line asking either way.
                Instrument::Unit {
                    file: "crates/vitui-components/src/nav.rs",
                    name: "the_id_is_the_census_and_not_the_attribution",
                },
            ],
        },
    },
    Row {
        number: 56,
        on_spec_table: false,
        gate: "a collection is one tab stop: the walk collapses to three a panel and the ring keeps \
               every row",
        kind: Kind::Count,
        owner: "C02, C14",
        section: "spec §3",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/nav.rs",
                    name: "a_collection_is_one_tab_stop_and_the_ring_still_holds_its_rows",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/nav.rs",
                    name: "the_collapsed_walk_repeats_no_id",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/nav_numbers.rs",
                },
            ],
        },
    },
    // ── components ticket 09's rows: the dense screen ────────────────────────────────────────────
    Row {
        number: 57,
        on_spec_table: false,
        gate: "the dense screen stands 338 interactive regions at 300x80 and reports §20's metric \
               row",
        kind: Kind::Count,
        owner: "C01",
        section: "spec §21",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DENSE,
                    name: "the_dense_screen_stands_three_hundred_and_thirty_eight_regions_at_three_\
                           hundred_by_eighty",
                },
                // Two counts and not one: what the draw believes it declared and what the runtime's
                // hit index holds. **110 of 338 widgets were inert on the first screen written for
                // components ticket 01 and the screen rendered pixel for pixel correctly**, which is
                // the defect a single count cannot see.
                Instrument::Unit {
                    file: DENSE,
                    name: "the_dense_screen_declares_no_colliding_ids",
                },
                Instrument::Unit {
                    file: DENSE,
                    name: "the_dense_screen_reports_the_metric_row_and_never_prints_marked_zero",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/dense_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 58,
        on_spec_table: false,
        gate: "the same screen drawn naive and correct is 0 cells apart, at 300x80 and at 120x40",
        kind: Kind::Equality,
        owner: "C01, C02",
        section: "spec §2",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DENSE,
                    name: "the_same_screen_drawn_naive_and_correct_is_zero_cells_apart_at_both_\
                           sizes",
                },
                // **The twin is kept, and that is a gate rather than a habit.** It is the reference
                // the correct build is proved equal to, so deleting it deletes the argument — §21's
                // own rule, and here it is a scan for the declaration plus a doctest naming the
                // public path.
                Instrument::Unit {
                    file: DENSE,
                    name: "the_naive_twin_is_declared_in_this_file_and_named_by_path",
                },
                // The half that stops the equality being satisfied by drawing nothing: both arms
                // cover 24 000 of 24 000 cells, and the correct one writes each exactly once.
                Instrument::Unit {
                    file: DENSE,
                    name: "the_correct_screen_is_a_partition_of_twenty_four_thousand_cells",
                },
            ],
        },
    },
    Row {
        number: 59,
        on_spec_table: false,
        gate: "each of ADR 0026's five re-damage instances is reachable from the dense screen, and \
               the correct arm re-damages 0",
        kind: Kind::Count,
        owner: "C01, C02",
        section: "spec §2",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DENSE,
                    name: "all_five_re_damage_instances_are_measured_on_this_screen",
                },
                // The other direction: every defective arm covers **at least as many cells** as the
                // correct one, so *no cell never* scores each of them as the healthier build and the
                // pair is the only detector — which is what the table is an argument for.
                Instrument::Unit {
                    file: DENSE,
                    name: "every_defective_arm_covers_at_least_as_many_cells_as_the_correct_one",
                },
                Instrument::Unit {
                    file: DENSE,
                    name: "a_scrim_under_the_dialog_costs_the_dialogs_own_six_hundred_writes",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/dense_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 60,
        on_spec_table: false,
        gate: "a label that does not narrow is green at 300x80 and red at 120x40",
        kind: Kind::Equality,
        owner: "C02, C08",
        section: "spec §21",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DENSE,
                    name: "a_label_that_does_not_narrow_is_green_at_three_hundred_and_red_at_a_\
                           hundred_and_twenty",
                },
                // The gesture half, and it is a **shrink** and not a resize: §21 refuses to bank the
                // shrink gate written against a terminal resize, because a fresh rectangle has
                // nowhere for the residue to survive.
                Instrument::Unit {
                    file: DENSE,
                    name: "the_narrow_screen_shrinks_its_content_inside_a_rectangle_that_does_not_\
                           move",
                },
                Instrument::Unit {
                    file: DENSE,
                    name: "at_a_hundred_and_twenty_columns_the_label_column_truncates_and_the_\
                           ellipsis_is_one_cell",
                },
            ],
        },
    },
    // ── the row that was red on purpose, inverted by components ticket 10 ────────────────────────
    //
    // **It read *scenes 1, 2 and 28 are red because their four components do not exist, and the
    // failure says so*, with `0 of 4` as its failing set.** That is the one row on this register
    // whose gate was a statement about an *absence*, and inverting it therefore had to rewrite the
    // gate rather than only its standing — a row still asserting the absence would now be a row
    // asserting the components are gone.
    Row {
        number: 61,
        on_spec_table: false,
        gate: "scenes 1, 2 and 28 stand on `text`, `chip`, `button` and `panel`, and the failure a \
               missing one produces still says which failure it is",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §21",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DENSE,
                    name: "the_dense_screen_stands_on_its_four_declared_components",
                },
                // The other direction, and it is the half that would otherwise have been deleted
                // with the red row. **A scene that fails because it is unimplemented is
                // indistinguishable from one that fails because the code is wrong, unless the
                // message distinguishes them** — so the message takes a declaration list and the
                // hostile case is one call away for ever.
                Instrument::Unit {
                    file: DENSE,
                    name: "the_waiting_message_still_says_which_failure_it_is",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/scenes.rs",
                    name: "the_three_stood_up_scenes_rest_on_the_same_four_components",
                },
                // The scan finds a declaration when there is one and not when there is a comment,
                // so *all four declared* is an answer rather than a scanner that matches anything.
                Instrument::Unit {
                    file: DENSE,
                    name: "the_subject_scan_finds_a_declaration_when_there_is_one",
                },
            ],
        },
    },
    // ── components ticket 10's rows: the four primitives ─────────────────────────────────────────
    Row {
        number: 62,
        on_spec_table: false,
        gate: "each of `text`, `chip`, `button` and `panel` writes a partition of its rectangle, at \
               every width and every justification",
        kind: Kind::Equality,
        owner: "C01, C02",
        section: "spec §2",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/text.rs",
                    name: "text_writes_a_partition_of_its_whole_rectangle",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/text.rs",
                    name: "chip_writes_a_partition_of_its_rectangle_and_narrows_into_it",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/input.rs",
                    name: "button_writes_a_partition_of_its_whole_rectangle",
                },
                // **The container's form of the same rule**, and the one that needs both halves of
                // §2's sentence: `distinct == w × h − interior`, with the interior the panel named
                // in its return value.
                Instrument::Unit {
                    file: "crates/vitui-components/src/structure.rs",
                    name: "a_panel_writes_its_frame_exactly_once_and_hands_the_interior_over_\
                           untouched",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/primitive_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 63,
        on_spec_table: false,
        gate: "no primitive names a fill, and every one of them reaches `fit` or `block`",
        kind: Kind::Invariant,
        owner: "C01",
        section: "spec §3",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DENSE,
                    name: "no_primitive_names_a_fill_and_every_one_reaches_a_partition_helper",
                },
                // Watched failing, in **two different ways**: the filled face breaks
                // `writes == distinct` and covers exactly the right cells; the unnarrowed label
                // keeps `writes == distinct` and covers too many. One gate catches one of them.
                Instrument::Unit {
                    file: "crates/vitui-components/src/text.rs",
                    name: "a_chip_that_fills_its_face_and_one_that_does_not_narrow_fail_two_\
                           different_gates",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/structure.rs",
                    name: "a_panel_that_writes_its_title_over_its_border_costs_fifteen_cells",
                },
            ],
        },
    },
    Row {
        number: 64,
        on_spec_table: false,
        gate: "the application clears once — its first frame and each resize — and a steady frame \
               re-damages 0 rather than 9 024",
        kind: Kind::Count,
        owner: "C01",
        section: "spec §2",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/app.rs",
                    name: "one_clear_a_size_and_never_a_third",
                },
                // **Both directions are defects**, and the third spelling is the one worth reading
                // twice: a clear keyed on nothing passes every steady-frame gate on this register
                // and still misses the resize.
                Instrument::Unit {
                    file: "crates/vitui-components/src/app.rs",
                    name: "the_every_frame_spelling_clears_every_frame_and_the_deaf_one_misses_the_\
                           resize",
                },
                Instrument::Unit {
                    file: DENSE,
                    name: "clearing_once_re_damages_nothing_and_clearing_every_frame_costs_nine_\
                           thousand_cells",
                },
            ],
        },
    },
    Row {
        number: 65,
        on_spec_table: false,
        gate: "the frame one chip is hovered re-damages 8 cells and not the screen",
        kind: Kind::Count,
        owner: "C01",
        section: "spec §2",
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/text.rs",
                    name: "a_hovered_chip_re_damages_its_own_eight_cells_and_not_the_screen",
                },
                // Row 48 is ticket 07's form of the same eight cells, measured through `press`
                // rather than through `chip`. **This one is the component's**, and it is where the
                // label's own paint stopped being a second one — see `crate::text::ChipOpts`.
                Instrument::Unit {
                    file: "crates/vitui-components/src/state.rs",
                    name: "the_hand_written_chip_passes_every_gate_that_existed_before_this_one",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/primitive_numbers.rs",
                },
            ],
        },
    },
    // ── components ticket 11's two, and they are two different kinds of row ──────────────────────
    Row {
        number: 66,
        on_spec_table: false,
        gate: "regions identical at 1 000, 100 000 and 1 000 000 rows",
        kind: Kind::Equality,
        owner: "C11",
        section: "spec §5, §20",
        // **This is not row 4 restated, and the difference is the finding.** Row 4 is *writes flat
        // 1k -> 1M* and it is `Unsubjected`; this row asks about the **hit index**, and the reason
        // it has to is that the write count cannot see the defect at all: the engine reports a
        // fully clipped verb as zero columns, so a listing that iterates its whole content and lets
        // the clip reject the rest writes exactly what the windowed one writes — 3 200 at every
        // volume — while declaring 1 000 001 regions against 81. Row 4 would be green on it.
        //
        // `Evaluated` over the listing rather than over `collection`, which is the same standing
        // components ticket 04's four rows have and for the same reason: what it gates is that
        // *the instrument separates a correct build from a defective one*, watched in both
        // directions. The **scene** stays red, and `crate::scenes` says why the two are not one
        // claim.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: LISTING,
                    name: "the_listing_writes_and_declares_the_same_at_a_thousand_rows_and_at_a_\
                           million",
                },
                Instrument::Unit {
                    file: LISTING,
                    name: "a_listing_that_iterates_its_whole_content_writes_the_same_and_declares_\
                           a_thousand_times_more",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/listing_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 67,
        on_spec_table: false,
        gate: "a scene with no subject fails differently from a scene whose code is wrong",
        kind: Kind::CompileOutcome,
        owner: "C11",
        section: "spec §21",
        // **It was the fourth red row and it is row 61 one ticket later, in both halves.** Ticket 09
        // pinned the dense screen's three scenes in exactly this shape and ticket 10 inverted it;
        // ticket 11 pinned the listing's five and **ticket 12 inverted four of them** — which is
        // the interesting half, because a single `inverted_by` across the five would have turned
        // the wheel gate too and erased the distinction the row is about.
        //
        // `CompileOutcome` for row 61's reason: what is asserted is that a *file* declares an item,
        // read by opening it — the one thing a `compile_fail` fence cannot say, because a fence
        // over a missing item passes today and passes again the day the module is renamed.
        //
        // Both directions stay live. `crate::listing::owed_message` still builds the waiting
        // sentence over any declaration list, and `the_waiting_message_separates_unimplemented_from_
        // wrong` hands it an empty one — so the message this row is about is exercised on the side
        // of the line the crate is no longer on.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: LISTING,
                    name: "the_listing_stands_on_the_collection_it_is_a_screen_of",
                },
                Instrument::Unit {
                    file: LISTING,
                    name: "the_waiting_message_separates_unimplemented_from_wrong",
                },
                Instrument::Unit {
                    file: LISTING,
                    name: "the_subject_scan_finds_a_declaration_when_there_is_one",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/scenes.rs",
                    name: "eight_scenes_have_nothing_to_run_over_four_are_red_and_\
                           twenty_one_are_stood_up",
                },
            ],
        },
    },
    // ── components ticket 18's rows ──────────────────────────────────────────────────────────────
    Row {
        number: 68,
        on_spec_table: false,
        gate: "the reserved decision has hysteresis when it is computed from last frame's reduced \
               rectangle: both bars over content that fits, 19 of 20 rows for ever",
        kind: Kind::Equality,
        owner: "C13",
        section: "spec §9",
        // ADR 0029's second sentence, as a gate: **reserved auto-hiding bars require a declared
        // content size.** The row is separate from row 20 because it fires the other way — row 20
        // asserts the fixpoint *terminates*, this one asserts the spelling that always terminates
        // is *wrong*, and a build that deleted the loop would pass row 20.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: AREA,
                    name: "the_incremental_spelling_keeps_both_bars_over_content_that_fits",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "a_body_that_is_not_antitone_flips_the_decision_every_frame",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/area_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 69,
        on_spec_table: false,
        gate: "`sum h` is the extent and the row count is not: row 799 999 of 999 999, with every \
               counter a frame carries identical",
        kind: Kind::Equality,
        owner: "C13",
        section: "spec §9",
        // **Two assertions in one row on purpose.** The reachability is the defect and the
        // equality is what makes it a false green: a row asserting only the first would pass on a
        // build whose row-measured frame is visibly different, and §9's whole claim is that it is
        // not.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: AREA,
                    name: "the_row_measured_extent_cannot_reach_the_last_row",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "nothing_a_frame_counts_separates_a_cell_extent_from_a_row_extent",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/area_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 70,
        on_spec_table: false,
        gate: "the thumb and the extent share a unit: 0 cells of drift against 14 of 69",
        kind: Kind::Count,
        owner: "C13",
        section: "spec §9",
        // The same unit error one helper down, and it is a separate row because it runs over
        // shipped code: `crate::scroll::thumb` is components ticket 07's and this row hands it two
        // extents of the same content.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: AREA,
                    name: "the_thumb_and_the_extent_share_a_unit",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/scroll.rs",
                    name: "the_two_thumbs_are_one_cell_and_two_hundred_and_twenty_three",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/area_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 71,
        on_spec_table: false,
        gate: "an overlay bar re-damages the cells its body drew under it — 398 against 0 for the \
               reserved twin on the same screen with the same content",
        kind: Kind::Count,
        owner: "C13",
        section: "spec §9",
        // **§21's scene-19 row asks for an amplification factor and this row deliberately does not
        // gate one.** The factor is a property of the damage *structure*, and the engine measured
        // one span per surface row against a per-row bitset and shipped the bitset, which is
        // 1.00x by construction. What is gateable is the double write, and it is asserted from the
        // bars' own footprint as well as measured.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: AREA,
                    name: "an_overlay_bar_re_damages_what_the_body_draws_under_it",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "the_amplification_is_one_under_the_structure_that_ships",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/area_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 72,
        on_spec_table: false,
        gate: "a `scroll_area` costs its content and a virtualised `collection` costs its window: \
               100 000 rows iterated against 69, at identical writes",
        kind: Kind::Count,
        owner: "C13, C21",
        section: "spec §9",
        // **The counter a reader reaches for does not move**, which is why this row is on
        // `iterated` rather than on `writes` — components ticket 11's finding arriving on the
        // other half of the pair. The 7 907 us §9 states is a report and lives in the example.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: AREA,
                    name: "a_scroll_area_costs_its_content_and_a_collection_costs_its_window",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "a_scrolled_scope_translates_the_content_the_right_way",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/area_numbers.rs",
                },
            ],
        },
    },
    // ── components ticket 16's two ───────────────────────────────────────────────────────────────
    Row {
        number: 73,
        on_spec_table: false,
        gate: "cells asked for is independent of depth, and no other counter is",
        kind: Kind::Count,
        owner: "C05",
        section: "spec §7",
        // **The counter §7 names and §21's table does not carry.** A row's cost may read a depth
        // and may not be proportional to one, and the whole of what an unclamped indent does is
        // invisible to every counter that ships: the engine reports the same columns written, the
        // same distinct cells, the same regions and the same stops — and **fewer verbs**, because
        // the label rectangle collapses, and **less time**, because the clip discards the overrun
        // before the row is built. Eight of §20's nine counters prefer the defect and the ninth is
        // `Unreachable`.
        //
        // `Evaluated` over the forest's two arms rather than over `tree`, for row 66's reason. It
        // is a `Count` and not a `Ratio` because the ratio is this screen's — 399.99x here against
        // §7's 165x over a screen with a menu bar on it — while *the ask does not move with the
        // depth* is the mechanism.
        //
        // The instrument's own limit is a row of its own file rather than a footnote:
        // `Tally::asked` folds a `u16` column count and **saturates at 65 535 a verb**, so the
        // ask it reports is 45.3% short and still 218x the correct arm's. The gate is taken over
        // the caller's `u64`, and the second test asserts the gap.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: FOREST,
                    name: "an_unclamped_indent_asks_for_four_hundred_times_the_cells_and_costs_\
                           fewer_verbs",
                },
                Instrument::Unit {
                    file: FOREST,
                    name: "the_tally_saturates_where_the_caller_does_not",
                },
                Instrument::Unit {
                    file: FOREST,
                    name: "the_forest_draws_the_same_frame_at_every_volume_and_at_every_depth",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/tree_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 74,
        on_spec_table: false,
        gate: "a tree scene with no subject fails differently from a scene whose code is wrong",
        kind: Kind::CompileOutcome,
        owner: "C05",
        section: "spec §21",
        // **The fifth red row, and it is row 67 for a second subject.** The gate is the same
        // sentence and the *failing set* is what a red row is, which is why this is a row rather
        // than a citation of row 67: 67's failing set names `collection` and five scenes pinned to
        // components 12, and nothing in it would go false the day `tree` arrives.
        //
        // `CompileOutcome` for row 61's and row 67's reason: what is asserted is that a *file*
        // declares an item, read by opening it — the one thing a `compile_fail` fence cannot say,
        // because a fence over a missing item passes today and passes again the day the module is
        // renamed.
        //
        // **Inverted by components ticket 17, and inverting it kept the gate.** The sentence is
        // about the *distinction* — a scene with no subject and a scene whose code is wrong fail
        // differently — and that distinction is still live now `tree` exists, because
        // `owed_message` is fired over a declaration list rather than over the crate. Row 61 is the
        // precedent for the shape a red row cannot be turned green by editing one field of: this
        // one could, because it never asserted an absence.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: FOREST,
                    name: "the_forest_stands_on_the_tree_it_is_a_screen_of",
                },
                Instrument::Unit {
                    file: FOREST,
                    name: "the_waiting_message_separates_unimplemented_from_wrong",
                },
                Instrument::Unit {
                    file: FOREST,
                    name: "the_subject_scan_finds_a_declaration_when_there_is_one",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/scenes.rs",
                    name: "eight_scenes_have_nothing_to_run_over_four_are_red_and_\
                           twenty_one_are_stood_up",
                },
            ],
        },
    },
    // -- components ticket 14's four, and the third is not about this crate ----------------------
    Row {
        number: 75,
        on_spec_table: false,
        gate: "writes and verbs identical at 12, 40, 120 and 240 declared columns",
        kind: Kind::Equality,
        owner: "C04",
        section: "spec §6",
        // **This is row 11 evaluated over a screen rather than row 11 turned green.** Row 11 is
        // §21's and it stays `Unsubjected` until `table` exists, which is components 15's; what
        // this row gates is that *the instrument separates a correct build from a defective one*,
        // watched in both directions — the virtualised arm is flat across all four declared counts
        // and the clip-only arm costs 10.9x the verbs for the same 24 000 writes. Components
        // ticket 11's row 66 has the same standing and the same reason.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: GRID,
                    name: "writes_and_verbs_are_identical_across_declared_column_counts",
                },
                Instrument::Unit {
                    file: GRID,
                    name: "the_clip_only_spelling_costs_verbs_and_no_writes_at_all",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/grid_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 76,
        on_spec_table: false,
        gate: "a band written by arithmetic re-damages the pinned band it overruns",
        kind: Kind::Count,
        owner: "C04",
        section: "spec §6",
        // **Not row 12, and the difference is the finding.** Row 12 is *the same table under a
        // horizontal offset, against itself* — an equality — and the equality is **blind** to this
        // defect: §6's own clause says why, *the screen is correct, because the pinned band draws
        // afterwards and wins*, so the two surfaces are 0 cells over 0 rows apart. What sees it is
        // the pair `writes` against `distinct`, which is row 6, C02's, filed by §21 as a *report
        // per component*. So §6's *no gate left by C01, C02 or C03 sees it* is half true: C02 named
        // the counter and nobody was running it.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: GRID,
                    name: "the_equality_is_blind_to_the_band_and_the_pair_is_not",
                },
                Instrument::Unit {
                    file: GRID,
                    name: "at_an_offset_inside_a_column_the_overrun_is_on_the_left_and_the_\
                           picture_is_wrong",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/grid_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 77,
        on_spec_table: false,
        gate: "a scroll scope at a nonzero offset shows its content and not the rows above it",
        kind: Kind::Count,
        owner: "C04",
        section: "spec §21",
        // **Filed red by components ticket 14, inverted by components ticket 12, and the two were
        // built in parallel.** It was the only row on this register whose subject is another crate,
        // and it is here rather than nowhere because §21's rule for a red gate is *assert the exact
        // failing set, fire in both directions, say what to invert*, and all three were writable.
        //
        // What inverted it is a sign in `vitui_runtime::ctx::Ctx::scroll_scope`, which passed the
        // offset through unnegated where the engine's rule is *a viewport scrolled `n` rows down is
        // `scrolled(0, -n)`*. **Three tickets found it independently** — 14 as a first frame that
        // drew nothing, 18 as `visible_rows() == -5..1`, 12 as a collection that would not scroll —
        // which is why the row is kept with its history rather than deleted: a gate that was red for
        // one integration and green after it is the register working, not noise.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: GRID,
                    name: "a_scroll_scope_at_a_nonzero_offset_shows_its_content",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "a_scrolled_scope_translates_the_content_the_right_way",
                },
            ],
        },
    },
    Row {
        number: 78,
        on_spec_table: false,
        gate: "a table scene with no subject fails differently from one whose code is wrong",
        kind: Kind::CompileOutcome,
        owner: "C11",
        section: "spec §21",
        // **Row 67 one component over**, and it is a row rather than a comment for row 67's reason:
        // the whole argument of §21 is that an obligation stated as a sentence gets broken by
        // someone who has read it. `CompileOutcome` because what is asserted is that a *file*
        // declares an item, read by opening it - the one thing a `compile_fail` fence cannot say.
        // **Inverted by components 15**, and the row is kept with its history for row 77's reason:
        // a gate that was red for one ticket and green after it is the register working. What
        // inverted it is `pub fn table(` in `collect.rs` **and** `crate::grid::draw_into` calling
        // it — the second half is the one a subject scan cannot see, and it is why the standing
        // names three instruments rather than one.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: GRID,
                    name: "the_grid_stands_on_the_table_it_is_a_screen_of",
                },
                Instrument::Unit {
                    file: GRID,
                    name: "the_waiting_message_separates_unimplemented_from_wrong",
                },
                Instrument::Unit {
                    file: GRID,
                    name: "the_subject_scan_finds_a_declaration_and_the_freeze_agrees_with_it",
                },
            ],
        },
    },
    // ── components ticket 21's two, and both are about what a count can see ──────────────────────
    Row {
        number: 79,
        on_spec_table: false,
        gate: "a body declares a count proportional to the rectangle it was handed, and one that \
               ignores it declares a count flat in the height",
        kind: Kind::Relation,
        owner: "C14",
        section: "spec §8, §21",
        // **A relation and not an equality, because the number belongs to the body.** §21's own
        // example is `verbs <= writes` and its own sentence is *never verb equality across sizes*;
        // what is asserted here is the *shape* of the two curves over every height from one to ten,
        // not a pair of magnitudes. §8 states one point on them — 273 ring entries against 247 —
        // and the point is the height where the difference is 26, which is two rows of body left.
        //
        // `Evaluated` over the accordion rather than over `collapsible`, which is components
        // ticket 04's standing. The scene stays red; `crate::scenes` says why those are not one
        // claim.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: ACCORDION,
                    name: "a_body_that_does_not_cull_declares_a_count_flat_in_the_rectangle_it_\
                           was_handed",
                },
                Instrument::Unit {
                    file: ACCORDION,
                    name: "the_mid_transition_pair_is_this_screen_plus_the_same_chrome",
                },
                Instrument::Unit {
                    file: ACCORDION,
                    name: "twelve_open_sections_declare_fewer_than_twice_six_because_one_off_\
                           screen_declares_nothing",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/collapsible_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 80,
        on_spec_table: false,
        gate: "no counter that reads a cell separates a closed body from one declared and not \
               drawn: `regions` and `tab stops`, and nothing else",
        kind: Kind::Count,
        owner: "C14",
        section: "spec §8, §20",
        // **This is the row that says why row 22 has to be a count over the hit index.** §8's
        // sentence is *no golden-cell gate can see it*, and this is that sentence as a list over
        // §20's nine: the two surfaces are 0 cells over 0 rows apart, `writes`, `distinct` and
        // `asked` are equal to the unit, and the two counters that move are the two the defect is
        // about.
        //
        // **It found one thing §8 does not record.** `verbs` separates the spelling §8 measured —
        // a body called at `h = 0` that *draws* every row makes 1 328 drawing calls against 104 —
        // so a gate written on the cheapest counter that happened to work would look green. It is
        // blind to the body beside it, which declares every row and paints only the admitted ones:
        // same verbs, same writes, same asked, same surface, and the same 408 entries nobody can
        // reach. Both arms are in the instrument.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: ACCORDION,
                    name: "the_two_surfaces_are_identical_and_no_counter_that_reads_a_cell_can_\
                           see_it",
                },
                Instrument::Unit {
                    file: ACCORDION,
                    name: "this_screen_opens_no_group_so_its_ring_and_its_stop_count_are_the_same_\
                           number",
                },
                Instrument::Unit {
                    file: ACCORDION,
                    name: "the_rest_of_8s_declaration_figures_are_this_screen_plus_the_chrome_\
                           fitted_from_the_first",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/collapsible_numbers.rs",
                },
            ],
        },
    },
    // ── components ticket 23's five, and four of them are about what a counter cannot see ────────
    Row {
        number: 81,
        on_spec_table: false,
        gate: "every gate of spec §11 runs on clusters that are not one code point, and the same \
               gate over ASCII reports nothing",
        kind: Kind::Count,
        owner: "C06",
        section: "spec §11",
        // **The row that makes the corpus a value rather than a string literal.** §11's sentence is
        // *every number and every gate runs on clusters that are not one code point*, and a
        // sentence is what gets broken by somebody who has read it (§17). What is asserted is the
        // pair: the corpus carries all seven kinds §11 names, and the same sweep over a corpus of
        // nothing but ASCII reports 0 steps inside a cluster and 0 width surprises — so a gate that
        // has quietly started running on ASCII fails rather than passing.
        //
        // **The barrier is the finding underneath it.** `vitui_engine::graphemes` is a free
        // function, not a type, so it is on no `pub use vitui_engine::` line in the runtime and
        // `crate::line::ENGINE_NAMES` — a list of types — cannot carry it. A forward cluster step
        // is reconstructed here from two `truncate` probes, which is exact while no cluster is zero
        // columns wide; a **line break is zero columns wide**, so the step is wrong across one and
        // `crate::document::step` handles the break itself. Components ticket 24 is where the cost
        // of that is argued, and this row is where it is pointed at.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: CLUSTERS,
                    name: "the_corpus_carries_every_kind_section_eleven_names",
                },
                Instrument::Unit {
                    file: CLUSTERS,
                    name: "no_cluster_in_the_corpus_is_zero_columns_wide",
                },
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "a_char_caret_lands_inside_clusters_and_changes_widths_nobody_typed",
                },
                Instrument::Barrier {
                    file: "crates/vitui-runtime/src/layout/text.rs",
                    line: "use vitui_engine::{graphemes, width_of};",
                },
                Instrument::Report {
                    file: FIELD_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 82,
        on_spec_table: false,
        gate: "no counter of §20's nine separates a build that violates one of §11's four gates \
               from one that does not, and the allocation total prefers the defective build",
        kind: Kind::Count,
        owner: "C06",
        section: "spec §11, §20",
        // **§11 heads its four gates *none of them visible on the rendered screen*, and measured
        // that sentence splits in two.** Gates 1 and 2 are invisible in the strong sense — the
        // caret is `Screen::set_cursor` and not a cell, so both arms draw the same 24 000 cells.
        // Gates 3 and 4 change the screen a great deal and **no counter can tell**: the memo-key
        // defect draws 69 of 80 rows wrong while writing the same cells, making the same verbs,
        // declaring the same regions and recomputing *once* against the correct build's twice.
        //
        // This is components ticket 11's `counters_that_separate_them` one component over, run over
        // four defects rather than one, and the answer is the empty list on all four. A non-empty
        // one would mean a cheaper gate than the equality exists and the scenes are optional.
        //
        // **The ninth column is the caller's and it is not a hole.** `vitui-alloc-probe` is a
        // dev-dependency, so a library cannot measure it; the tests below hand both arms the same
        // value, which makes that column inert rather than false, and
        // `examples/field_numbers.rs` measures it with the probe installed. Measured, it separates
        // exactly one of the four — **in the direction that approves the defect**, because a stale
        // index has 625 rows where 875 are needed and the defective frame allocates less. §21's
        // refinement 1 in one number: a counter on the wrong side of the question is not a weak
        // gate, it is a green one.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "no_counter_separates_a_field_that_violates_one_of_the_four_gates_from_\
                           one_that_does_not",
                },
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_two_caret_gates_change_no_cell_and_the_two_index_gates_change_many",
                },
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_caret_is_where_the_two_caret_defects_are_visible_and_it_is_not_a_\
                           cell",
                },
                Instrument::Report {
                    file: FIELD_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 83,
        on_spec_table: false,
        gate: "a splice restarted one row before the edit's agrees with a rebuild 500 times in 500, \
               and one restarted at the edit's own row does not",
        kind: Kind::Equality,
        owner: "C06",
        section: "spec §10, §11",
        // §11 states *499 times in 500* about the naive restart point and *provably enough* about
        // the one a row earlier. Both halves are run over the same five hundred deterministic
        // edits, and the screen the surface figure comes from is drawn at the **first** of the five
        // hundred the naive point gets wrong — so the count and the picture are about one edit.
        //
        // **`provably enough` was false when this row was first written, and the cause was a defect
        // rather than the rule.** `vitui_runtime::layout::text::wrap` discarded a row that exactly
        // filled its band and fell through to the overhang branch, so a line's break structure could
        // change arbitrarily far back and five of the five hundred disagreed. Fixed by this ticket,
        // with the regression in the runtime's own module.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "five_hundred_splices_separate_the_two_restart_points",
                },
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "a_splice_restarted_at_the_row_of_the_edit_keeps_a_break_the_edit_\
                           invalidated",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/layout/text.rs",
                    name: "a_line_that_exactly_fills_its_band_is_not_cut_at_the_first_space",
                },
                Instrument::Report {
                    file: FIELD_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 84,
        on_spec_table: false,
        gate: "a wrap memo keyed on the revision alone draws 625 rows where 875 are needed, and \
               `recomputes` prefers it 1 against 2",
        kind: Kind::Equality,
        owner: "C06",
        section: "spec §11, §21",
        // §21's scene 13, as an equality against a correct render rather than as a row count. The
        // second clause is the reason it cannot be gated on `recomputes`: the defective key is a
        // cache **hit** on the resize, so the counter a reader would reach for first reports the
        // defect as the cheaper build.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_resize_differs_on_sixty_nine_of_eighty_rows",
                },
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_defective_memo_recomputes_once_and_the_correct_one_twice",
                },
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "a_memo_keyed_on_the_revision_alone_draws_an_index_built_at_another_\
                           width",
                },
                Instrument::Report {
                    file: FIELD_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 85,
        on_spec_table: false,
        gate: "a `field` scene with no subject fails differently from one whose code is wrong",
        kind: Kind::CompileOutcome,
        owner: "C06",
        section: "spec §21",
        // **Row 67 one ticket later, for a different component.** Ticket 09 pinned the dense
        // screen's three scenes in this shape and ticket 10 inverted it; ticket 11 pinned the
        // listing's five; this is the same distinction for `field`, and it is a row rather than a
        // comment because §21's whole argument is that an obligation stated as a sentence gets
        // broken by someone who has read it.
        //
        // `CompileOutcome` for row 61's reason: what is asserted is that a *file* declares an item,
        // read by opening it — the one thing a `compile_fail` fence cannot say, because a fence
        // over a missing item passes today and passes again the day the module is renamed.
        standing: Standing::Red {
            by: &[
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_document_is_red_because_field_is_not_declared",
                },
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_waiting_message_separates_unimplemented_from_wrong",
                },
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_subject_scan_finds_a_declaration_when_there_is_one",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/scenes.rs",
                    name: "eight_scenes_have_nothing_to_run_over_four_are_red_and_\
                           twenty_one_are_stood_up",
                },
            ],
            failing: "`field` is undeclared, so three scenes are pinned red — 12, 13 and 30 — and \
                      all three are waiting for it. `crates/vitui-components/src/input.rs` carries \
                      no `pub fn field` declaration, which is what \
                      `crate::document::subjects_declared` opens the file to find out, and \
                      `crate::document::standing` is `Unmet { over: 1, failing: 1 }` rather than \
                      `Met` over nothing",
            inverted_by: "components 24",
        },
    },
    // ── components ticket 25's four ──────────────────────────────────────────────────────────────
    Row {
        number: 86,
        on_spec_table: false,
        gate: "the regions and the stops of §12's five configurations, exactly",
        kind: Kind::Equality,
        owner: "C07",
        section: "spec §12",
        // **Two of §12's seven columns reproduce to the unit and two of them cannot.** The two that
        // do are the two that are properties of the frame rather than of the machine, and they are
        // arithmetic here rather than a measurement: 312 chips + 2 selects + 1 bar + 2 titles is
        // 317 against 316, a dropdown adds one collection entry and one blur position, a menu row is
        // a target and there are three of them twice, and a modal is two buttons inside a scope that
        // declares nothing. The two that cannot are rows 69 and `content layers` — see
        // `crate::popup`'s header, and `crate::counters::Counters::content_layers` for the half of
        // the second that was already written down.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: POPUP,
                    name: "every_configuration_declares_what_the_specs_table_says",
                },
                Instrument::Unit {
                    file: POPUP,
                    name: "the_three_deltas_are_the_familys_own_arithmetic",
                },
                Instrument::Unit {
                    file: POPUP,
                    name: "the_base_screen_is_317_regions_and_316_stops_and_the_bar_is_the_\
                           difference",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/popup_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 87,
        on_spec_table: false,
        gate: "n overlays standing cost n + 1 allocations a frame, as a total over the run",
        kind: Kind::Equality,
        owner: "C07",
        section: "spec §12, §19",
        // **The row that contradicts a closed map, and it is asserted as measured.** §12's table
        // reads `allocations = 0` in all five rows; the shipped figure is `n + 1` for `n` overlays
        // standing and 0 for none, because runtime ticket 21 deleted the bump arena that made the
        // zero true (ADR 0034). The gate is the **marginal** equality rather than an absolute,
        // which is the runtime's own form: an absolute is a fact about this screen and the margin is
        // a fact about the mechanism.
        //
        // A second case beside it, because the obvious spelling of this gate goes green while
        // meaning something else: `Box::new` of a zero-sized value does not allocate, so two bodies
        // written as bare `fn` items cost **one** allocation a frame and satisfy *at most n + 1*.
        // `crate::popup`'s menu and submenu were written that way and read 1 against 3.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/tests/popup.rs",
                    name: "one_more_overlay_standing_is_exactly_one_more_allocation_a_frame",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/tests/popup.rs",
                    name: "a_body_that_captures_nothing_costs_no_allocation_to_box",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/tests/popup.rs",
                    name: "the_specs_zero_is_the_arena_that_was_deleted_and_not_the_shipped_number",
                },
            ],
        },
    },
    Row {
        number: 88,
        on_spec_table: false,
        gate: "the walk repeats no id, and reaches every stop unless a trap is standing",
        kind: Kind::Invariant,
        owner: "C11",
        section: "spec §21",
        // **§21's third refinement, and this is the first screen it can run on.** *Name the
        // exception; do not loosen the gate.* R08's *a walkthrough visits every tab stop exactly
        // once* fails on exactly one panel of twelve and **correctly**, because a dialog is open and
        // a `Trap` is what a modal is — so the gate is the conjunction, with `Frame::trap_scopes` as
        // the only thing that can answer the second half. 2 of 318 with the modal up, 316 of 316
        // with it down. Driven by posted keys, with the frame's own `tab_walk` beside it as a second
        // expression rather than as the same one.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: POPUP,
                    name: "a_standing_trap_pulls_the_focus_in_and_six_tabs_do_not_take_it_out",
                },
                Instrument::Unit {
                    file: POPUP,
                    name: "the_walk_reaches_every_stop_unless_a_trap_is_standing",
                },
                Instrument::Unit {
                    file: POPUP,
                    name: "where_the_keyboard_goes_when_a_modal_closes_is_three_different_programs",
                },
            ],
        },
    },
    Row {
        number: 89,
        on_spec_table: false,
        gate: "the overlay family's scene has no subject, and says so rather than failing",
        kind: Kind::CompileOutcome,
        owner: "C07",
        section: "spec §12, §21",
        // **The fifth red row, and it is row 67 one ticket later.** Same shape, same reason, a
        // different pair of components — and the standing count is what makes a scene going quiet a
        // deliberate edit rather than a silent one.
        standing: Standing::Red {
            by: &[
                Instrument::Unit {
                    file: POPUP,
                    name: "the_screen_is_red_because_select_and_overlay_are_not_declared",
                },
                Instrument::Unit {
                    file: POPUP,
                    name: "the_waiting_message_separates_unimplemented_from_wrong",
                },
                Instrument::Unit {
                    file: POPUP,
                    name: "the_subject_scan_finds_a_declaration_when_there_is_one",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/scenes.rs",
                    name: "eight_scenes_have_nothing_to_run_over_four_are_red_and_\
                           twenty_one_are_stood_up",
                },
            ],
            failing: "neither `select` nor `overlay` is declared, so scene 14 is pinned red. \
                      `crates/vitui-components/src/input.rs` carries no `pub fn select(` and \
                      `crates/vitui-components/src/overlay.rs` no `pub fn overlay(`, which is what \
                      `crate::popup::subjects_declared` opens both files to find out, and \
                      `crate::popup::standing` is `Unmet { over: 2, failing: 2 }` rather than `Met` \
                      over nothing. Everything the screen itself can be asked is measured: 317 \
                      regions against 316 stops, every delta of §12's table, 2 visited of 318 \
                      declared with the trap standing, 30 cliffs over 30 openings, 600 / 0 / 0 \
                      re-damaged for the scrim's three spellings, 99 flips in 100 frames, 3 of 8 \
                      frames for a dialog owned by a menu row, and three different ids for the \
                      three answers to a closing modal",
            inverted_by: "components 26",
        },
    },
    // ── components ticket 12's rows: the collection ──────────────────────────────────────────────
    Row {
        number: 90,
        on_spec_table: false,
        gate: "one component, one `Mode`, thirteen match arms, and no inventory row duplicates one \
               of them",
        kind: Kind::Count,
        owner: "C03",
        section: "spec §5",
        // **The arm count is read out of the source and not declared**, which is the register's own
        // rule applied to the one number §5 leads with: an instrument is a value with a file in it,
        // and a constant asserting `13 == 13` is a tautology wearing a gate's clothes.
        //
        // The second half is the other direction of ADR 0028's *six inventory entries collapse into
        // one*: `list`, `option_list`, `menu`, `multi_select`, `tabs`, `radio_group` and
        // `segmented_control` are `Mode`s, and a row reappearing under one of those names is that
        // collapse being quietly undone.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: COLLECT,
                    name: "one_component_one_mode_and_thirteen_match_arms",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_radio_group_and_a_file_manager_differ_by_the_arms_and_nothing_else",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/collection_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 91,
        on_spec_table: false,
        gate: "select-all is one span and 16 bytes at every length, and the pathological selection \
               is proportional to the gestures",
        kind: Kind::Equality,
        owner: "C03",
        section: "spec §5",
        // **An equality across four volumes and not a threshold**, because *one span* is a property
        // of the mechanism: `[0, len)` is one interval whatever `len` is. The three stores that were
        // refused are `crate::collect::stores`, kept runnable so §5's table is a measurement rather
        // than a memory — and one of them is a keystroke at 234x the whole frame budget.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: COLLECT,
                    name: "select_all_is_one_span_and_sixteen_bytes_at_every_length",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "the_span_list_is_sorted_disjoint_and_non_adjacent_under_every_gesture",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/collection_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 92,
        on_spec_table: false,
        gate: "the scan cursor seeks once a frame and the naive form seeks once a row",
        kind: Kind::Count,
        owner: "C03",
        section: "spec §5",
        // **`O(log k + h)` against `O(h · log k)` as a count**, which is §21's rule: a gate is a
        // count, a ratio, an equality or a compile outcome, and a timing is a report. Timed instead,
        // the two are indistinguishable at the sizes a terminal reaches — which is exactly why the
        // shape is worth gating rather than measuring.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: COLLECT,
                    name: "the_scan_cursor_seeks_once_and_the_naive_form_seeks_a_row",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/collection_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 93,
        on_spec_table: false,
        gate: "a collection declares one hit entry, and `merges == 0` on a screen with two of them",
        kind: Kind::Count,
        owner: "C03, C01",
        section: "spec §5, §4",
        // **Two claims and one screen, because they fail together.** ADR 0028's *one hit entry per
        // collection* is what makes per-row hover arithmetic off `Response::local` rather than 389
        // regions for a frame-old answer; ADR 0027's *the row loop is wrapped in `cx.with_id`* is
        // what stops the second collection on the screen being inert. The second is the one that
        // renders **pixel for pixel correctly** when it is broken, which is why `merges` is the
        // only instrument that reports it — `crate::collect::defective::unkeyed_rows` is that build.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: COLLECT,
                    name: "two_collections_on_one_screen_declare_two_entries_and_merge_nothing",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_row_is_hovered_by_arithmetic_on_this_frames_pointer",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/collection_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 94,
        on_spec_table: false,
        gate: "`size_of::<CollState>()` is the same at every length, and no field is keyed by a row",
        kind: Kind::Count,
        owner: "C03",
        section: "spec §5",
        // §5's 208 does **not** reproduce and `crate::collect::COLL_STATE_BYTES` says why rather
        // than padding the struct: the 208 is C03's prototype struct, which carried the three
        // refused stores and a discriminant to choose between them. What §5 gates is the
        // invariance, and that reproduces exactly — the type mentions no length, so `size_of`
        // cannot depend on one.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: COLLECT,
                    name: "the_state_is_the_same_size_at_every_length",
                },
                // **The frame's own zero-allocation half, and it is a `tests/` gate rather than a
                // number in the report.** A collection over a million rows draws through the
                // *shipped* entry point with the row drawer a component author would write —
                // `stage` and `blit` — and a `format!` a row would be eighty allocations a frame.
                // Register entry 12's finding one crate down is why it is not left in the example:
                // an example is compiled and evaluated by nothing.
                Instrument::Unit {
                    file: "crates/vitui-components/tests/gates.rs",
                    name: "a_collection_over_a_million_rows_allocates_nothing_in_a_steady_frame",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/collection_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 95,
        on_spec_table: false,
        gate: "ctrl-click and shift-click are the same gestures as their keyboard twins, and no \
               component reads a modifier from anywhere else",
        kind: Kind::Equality,
        owner: "C03",
        section: "spec §5, §22",
        // **The row §5 said could not exist.** It records ctrl-click and shift-click as
        // inexpressible, because `rt::Input` was `Move`, `Down`, `Up`, `Wheel` and `Key` and only
        // `Key` carried a modifier byte. Runtime 10 carries `mods: Mods` on `Response`, so the two
        // readings are one vocabulary and the equality is over the arms rather than over a
        // substitution.
        //
        // The *and no component reads a modifier from anywhere else* half is a source scan, for the
        // reason `crate::state`'s press gate is: an absence has no expression, and a second reader
        // arriving is exactly the edit that would make the equality above true and meaningless.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: COLLECT,
                    name: "ctrl_click_and_shift_click_are_the_same_gestures_as_their_keyboard_twins",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "the_modifier_byte_is_read_in_one_function_and_it_takes_a_response",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/collection_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 96,
        on_spec_table: false,
        gate: "one type-ahead keystroke looks at the budget and not at the content",
        kind: Kind::Ratio,
        owner: "C03",
        section: "spec §5",
        // **A ratio of rows looked at, not of microseconds.** §5 states 70.79 us bounded against
        // 1 507.83 and the search alone at 211x; the 211 is the *ratio* and it is a property of the
        // bound, so what is gated is `content / budget` and the microseconds are printed beside it.
        // `crate::collect::type_ahead_cost` measures both arms and the example prints them.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: COLLECT,
                    name: "one_keystroke_into_a_million_rows_looks_at_the_budget_and_not_at_the_\
                           content",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_collection_declines_a_chord_and_swallows_no_accelerator",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/collection_numbers.rs",
                },
            ],
        },
    },
    // ── components ticket 13's rows: the order, the index and the memo ───────────────────────────
    Row {
        number: 97,
        on_spec_table: false,
        gate: "one structure has five names, and every field of the record is earned by one of them",
        kind: Kind::Count,
        owner: "C05, C06",
        section: "spec §10",
        // **Both directions, which is what makes it a gate rather than a table.** Every use must
        // name only fields that exist, and every field must be spent by at least one use — a field
        // nothing spends is a fifth structure's field living in the shared one, which is exactly
        // the drift ADR 0031's *three names for one mechanism is already one too many* is about.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: ORDER,
                    name: "one_structure_has_five_names_and_every_field_is_earned",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/order_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 98,
        on_spec_table: false,
        gate: "`Rows { len, rev }` is one `u64`, every edit stamps it, and a revision the component \
               has not seen clears every position it holds",
        kind: Kind::Count,
        owner: "C04, C05",
        section: "spec §10",
        // **The failure this gates is a perfectly correct frame.** A sort changes no data and no
        // length, so nothing inside a collection can notice: the frame after draws a correct list
        // with the wrong rows selected and the editor open on the wrong row. No counter anywhere in
        // the stack moves, which is why the gate is on the *store* and not on a screen.
        //
        // Three directions, because the middle one is the mechanism: an unrecognised revision
        // clears, `CollState::reconciled` says the caller carried the positions across and the
        // branch does not fire, and `Revision::UNKNOWN` never matches — so a caller with no order
        // is never told its positions went stale, which is right, since it has nothing that can
        // permute them.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: ORDER,
                    name: "the_revision_is_one_u64_and_every_edit_stamps_it",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_revision_the_component_has_not_seen_clears_every_position_it_holds",
                },
            ],
        },
    },
    Row {
        number: 99,
        on_spec_table: false,
        gate: "the request is one slot, and a component holding the order shared cannot edit it",
        kind: Kind::CompileOutcome,
        owner: "C05",
        section: "spec §10",
        // **`E0502`, which is a compile outcome and therefore a gate rather than a sentence.** §10
        // states *a component may only ask* and gives two reasons; the second is this one, and it
        // is the half that cannot be worked around. The pair sits on `order::Asked` with a positive
        // twin naming `Order::splice` **by path** — a lone `compile_fail` passes when the item has
        // been renamed, and the mechanism cannot tell `E0433` from `E0502`.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Pair {
                    file: ORDER,
                    hostile: "let _ = order.splice(1..3, [Entry::default()]);",
                },
                Instrument::Unit {
                    file: ORDER,
                    name: "the_request_is_one_slot_and_it_answers_once",
                },
            ],
        },
    },
    Row {
        number: 100,
        on_spec_table: false,
        gate: "a splice transforms a span list in O(spans) and cannot shatter; a permutation always \
               does",
        kind: Kind::Ratio,
        owner: "C05",
        section: "spec §10",
        // **A count and a ratio, not a timing.** Whatever the interval, a contiguous selection
        // comes out of a splice as *at most two* spans — a head and a tail — because a splice is an
        // interval and every span is entirely before it, entirely after it, or crosses it. A
        // permutation of the same edit comes out as one span a row. The microseconds are the
        // example's; the bound is the mechanism.
        //
        // Select-all escapes both at O(1) and **not by special-casing**: `[0, len)` is invariant
        // under any permutation of that length, so the answer is known without touching a row.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: ORDER,
                    name: "a_splice_cannot_shatter_a_span_list_and_a_permutation_always_does",
                },
                Instrument::Unit {
                    file: ORDER,
                    name: "a_collapse_splices_the_interval_and_a_rebuild_builds_the_forest",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/order_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 101,
        on_spec_table: false,
        gate: "the four reconcile policies, their settled defaults, and the one that is refused",
        kind: Kind::Count,
        owner: "C05",
        section: "spec §10, §7",
        // **The refusal is the row.** `Clear` under a splice throws away spans that were nowhere
        // near the fold for no saving at all, and the three real policies are all O(spans) and all
        // free — so there is nothing to buy and the choice between them is *behavioural*. The
        // admissions table is asserted in full rather than at the interesting corner, because a
        // refusal stated for one arm and forgotten for another is the shape this register exists
        // to catch.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: ORDER,
                    name: "the_four_policies_have_settled_defaults_and_clear_is_refused_for_a_\
                           splice",
                },
                Instrument::Unit {
                    file: ORDER,
                    name: "the_three_real_policies_are_all_proportional_to_the_spans",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/order_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 102,
        on_spec_table: false,
        gate: "a memo keyed on the revision alone recomputes less, runs faster, and is wrong on the \
               rendered surface",
        kind: Kind::Ratio,
        owner: "C06, C08, C09",
        section: "spec §10",
        // **The row that exists because the natural detector points the wrong way.** `recomputes`
        // goes *down* when the key has forgotten an input, so the instrument the runtime provides
        // for exactly this question reports an improvement — 1 against 2 on the 300 → 120 resize,
        // at a third of the cost. The two detectors that work are the memoised object recording the
        // input it was built at (`Keyed::built_at`) and the rendered surface, and the gate asserts
        // both.
        //
        // The magnitudes are this crate's corpus and §10's are C06's; the *shape* is what
        // reproduces — fewer rows than the narrow width needs, most of the window carrying the
        // wrong source line, and one recomputation against two.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: ORDER,
                    name: "a_memo_keyed_on_the_revision_alone_recomputes_less_and_is_wrong_on_the_\
                           screen",
                },
                Instrument::Unit {
                    file: ORDER,
                    name: "every_memo_spells_its_key_and_records_what_it_was_built_at",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/order_numbers.rs",
                },
            ],
        },
    },
    // ── components ticket 27's three ─────────────────────────────────────────────────────────────
    Row {
        number: 103,
        on_spec_table: false,
        gate: "the series screen is a partition at 300x80 and at 60x20, and a legend that does not \
               narrow is green at the first and red at the second",
        kind: Kind::Equality,
        owner: "C08",
        section: "spec §13, §21",
        // **The two-size axis with the arms one boolean apart**, which is what makes a reviewer's
        // diff one line. At 300x80 the two arms are *indistinguishable* — identical writes,
        // identical verbs, 0 cells apart on the rendered surface — and at 60x20 the defective one
        // writes 2 cells twice, touches exactly the cells the correct one touches, and costs **one
        // verb fewer**. Every counter but the pair prefers it, which is ADR 0026's signature
        // arriving on this screen.
        //
        // `Evaluated` over the screen rather than over `chart` and `plot`, which is components
        // ticket 04's standing and its reason: what it gates is that *the instrument separates a
        // correct build from a defective one*, watched in both directions. The **scenes** stay red.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: SERIES,
                    name: "the_legend_that_does_not_narrow_is_green_at_three_hundred_and_red_at_\
                           sixty",
                },
                Instrument::Unit {
                    file: SERIES,
                    name: "the_frame_costs_the_rectangle_and_the_verbs_track_the_picture",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/series_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 104,
        on_spec_table: false,
        gate: "the axis gutter computed from the whole domain oscillates on 0 of 175 712 viewport \
               x dataset pairs, and the fixpoint form on 464",
        kind: Kind::Count,
        owner: "C08",
        section: "spec §13",
        // **A count and not a ratio**, and it fires in both directions by construction: the same
        // sweep reports the naive form oscillating on 464 and the hysteresis form settling on a
        // gutter that is not a fixed point of its own rule on all 464 of them. A gate that only
        // asserted *the whole-domain form converges* would be green on a sweep that had stopped
        // sweeping.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: SERIES,
                    name: "the_axis_loop_oscillates_on_four_hundred_and_sixty_four_and_the_whole_\
                           domain_on_none",
                },
                Instrument::Unit {
                    file: SERIES,
                    name: "a_narrower_plotting_area_can_produce_a_wider_label",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/series_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 105,
        on_spec_table: false,
        gate: "scenes 15 and 16 stand on `chart` and `plot`, and the failure a missing one produces \
               still says which failure it is",
        kind: Kind::CompileOutcome,
        owner: "C08, C11",
        section: "spec §21",
        // **Red for one ticket, and inverting it rewrote the gate rather than the standing.** It
        // read *the series screen's two scenes fail differently from a screen whose code is wrong*
        // with `0 of 2` as its failing set. Row 61 is the precedent and it is the same shape: a row
        // that asserts an **absence** cannot be turned green by editing one field, because a row
        // still asserting the absence would now be asserting the components are gone.
        //
        // `CompileOutcome` for row 61's reason: what is asserted is that a *file* declares an item,
        // read by opening it — the one thing a `compile_fail` fence cannot say, because a fence over
        // a missing item passes today and passes again the day the module is renamed.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: SERIES,
                    name: "the_series_screen_stands_on_its_two_declared_components",
                },
                // The other direction, and it is the half that would otherwise have been deleted
                // with the red row. `owed_message` takes a declaration list rather than reading the
                // crate, so the hostile case is one call away for ever.
                Instrument::Unit {
                    file: SERIES,
                    name: "the_waiting_message_separates_unimplemented_from_wrong",
                },
                Instrument::Unit {
                    file: SERIES,
                    name: "the_screen_stands_up_and_the_refusal_is_still_watched",
                },
                Instrument::Unit {
                    file: SERIES,
                    name: "the_subject_scan_finds_a_declaration_when_there_is_one",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/scenes.rs",
                    name: "eight_scenes_have_nothing_to_run_over_four_are_red_and_\
                           twenty_one_are_stood_up",
                },
            ],
        },
    },
    // ── components ticket 28's three ─────────────────────────────────────────────────────────────
    Row {
        number: 106,
        on_spec_table: false,
        gate: "the raster memo's key is every input, and the range memo is a separate one it chains \
               from: a resize invalidates the raster and not the range",
        kind: Kind::Count,
        owner: "C08",
        section: "spec §13",
        // **The chain as two counts and the narrow key as a surface.** The counts are the mechanism
        // — one range fold over two rectangles against two raster folds — and the surface is the
        // detector, because the miss counter **points the wrong way** on this class of defect: a
        // narrower key folds *fewer* times and is wrong. §13's fifth independent arrival of that
        // rule on this map.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: CHART,
                    name: "the_memo_is_a_chain_and_a_resize_invalidates_the_raster_and_not_the_\
                           range",
                },
                Instrument::Unit {
                    file: CHART,
                    name: "a_raster_keyed_on_the_revision_alone_is_wrong_and_folds_less_often",
                },
                Instrument::Unit {
                    file: CHART,
                    name: "the_gutter_comes_from_the_whole_domain_and_a_window_scaled_axis_must_\
                           declare_its_range",
                },
            ],
        },
    },
    Row {
        number: 107,
        on_spec_table: false,
        gate: "no downsampling spelling exists in `vitui-components`, and each absent one is named \
               by path",
        kind: Kind::CompileOutcome,
        owner: "C08",
        section: "spec §13",
        // **A pair and a scan, because neither half holds alone.** The `compile_fail` fences name
        // `downsample`, `stride_sample` and `every_nth` by path with a twin naming `Raster::build`
        // by path beside them — a lone fence also passes when the module has been renamed. The scan
        // is what catches the spelling coming back **`pub(crate)`**, which is how a deleted helper
        // actually returns; `crate::frame`'s deleted focus ring is the precedent, and its own scan
        // reported itself on its first run.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Pair {
                    file: "crates/vitui-components/src/chart.rs",
                    hostile: concat!("vitui_components::chart::raster::down", "sample"),
                },
                Instrument::Unit {
                    file: CHART,
                    name: "no_downsampling_spelling_exists_anywhere_in_this_crate",
                },
                Instrument::Unit {
                    file: SERIES,
                    name: "striding_loses_the_peaks_and_the_union_does_not",
                },
            ],
        },
    },
    Row {
        number: 108,
        on_spec_table: false,
        gate: "series colours are one six-entry `Theme::custom` table, `Style` literals == 0, and a \
               threshold is carried on both axes",
        kind: Kind::Count,
        owner: "C08, C09",
        section: "spec §13, §16",
        // **Three counts about one rule.** §16: *a distinction survives the whole matrix iff it is
        // carried on both axes*, and a component reading `false` from `roles_differ_on_wire` owes a
        // second axis — a glyph, a rule, a position — and never a darker colour. The palette is the
        // one legitimate use of `Theme::custom` and it is counted rather than scattered: **one call
        // site in the crate**, read off the source.
        //
        // The `Style`-literal half is worth reading twice. Two looser needles were tried and both
        // reported `crate::form`'s own private `enum Style` — a form's word for a wrapping mode —
        // so the needle is the runtime's *re-export path*, which is the only way this crate could
        // reach the engine's type at all.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: CHART,
                    name: "the_series_palette_is_six_customs_and_the_crate_holds_no_style_literal",
                },
                Instrument::Unit {
                    file: CHART,
                    name: "a_threshold_is_carried_on_both_axes_and_a_paint_alone_dies_at_sixteen_\
                           colours",
                },
                Instrument::Unit {
                    file: CHART,
                    name: "role_derived_series_collapse_and_the_difference_is_style_only",
                },
            ],
        },
    },
    // ── components ticket 15's four, and one of them is a defect one crate down ──────────────────
    Row {
        number: 109,
        on_spec_table: false,
        gate: "one click on a column header costs one run at any length, and the flattened index \
               costs one a row",
        kind: Kind::Ratio,
        owner: "C04",
        section: "spec §6",
        // **A ratio and not a timing, which is the whole of why §6's two candidates are decided by
        // a gesture.** The frame does not distinguish them at all — both answer a cell through the
        // same `Scan`, amortised O(1) — so the microseconds are a report and the runs are the
        // gate. Both directions: the per-column store's own loss (one run a column for a whole
        // row) is asserted beside the win, because a store measured only where it wins is a store
        // whose trade nobody wrote down.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: COLLECT,
                name: "cell_selection_is_one_run_per_column_key_and_the_flattening_is_a_million",
            }],
        },
    },
    Row {
        number: 110,
        on_spec_table: false,
        gate: "two tables on one screen with a target in every visible cell merge nothing, and the \
               row-keyed spelling leaves every cell of a row inert but the first",
        kind: Kind::Count,
        owner: "C04",
        section: "spec §4, §6",
        // **Row 71 one axis over**, and §4 says so in as many words: *one axis out, the same defect
        // has a different arithmetic*. The screen is identical on both arms — drawing does not
        // consume an id — so `merges` is the only counter that separates them, and it is free and
        // already computed.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: COLLECT,
                name: "identity_is_per_cell_and_merges_is_the_only_counter_that_says_so",
            }],
        },
    },
    Row {
        number: 111,
        on_spec_table: false,
        gate: "a pinned column may not be elastic, and the same constraint unpinned is honoured",
        kind: Kind::Invariant,
        owner: "C04",
        section: "spec §6",
        // **Enforced and not documented**, which is the ticket's own wording: `Pin::width` is the
        // only reader of a pinned column's width and no `Constraint` reaches it, so the failure the
        // rule is about has no spelling. Both directions, because a rule that is enforced by
        // dropping the value on the floor and a rule that is enforced by a type look identical from
        // the failing side.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: COLLECT,
                name: "a_pinned_column_may_not_be_elastic",
            }],
        },
    },
    Row {
        number: 112,
        on_spec_table: false,
        gate: "a container may key its children inside a scroll scope",
        kind: Kind::Count,
        owner: "C04",
        section: "spec §4",
        // **The second row on this register whose subject is another crate**, and row 77 is the
        // precedent that says it belongs here: §21's rule for a red gate is *assert the exact
        // failing set, fire in both directions, say what to invert*, and all three are writable.
        //
        // `Ctx::with_id` — which `Ctx::with_key` is written on — re-childs the view at
        // `self.area()`, and `area()` is `Rect::new(0, 0, w, h)` in the *current* coordinate
        // system. Inside a scroll scope that origin is the content's, so the clip it intersects
        // with is content rows `0..h` while the window is at the offset. `collection` already
        // works around it by pushing its id outside the scope; `table` cannot, because a cell's key
        // is per row, so it mints with `Id::keyed` and hands the value down.
        standing: Standing::Red {
            by: &[Instrument::Unit {
                file: COLLECT,
                name: "a_with_key_inside_a_scroll_scope_draws_nothing_past_the_first_screenful",
            }],
            failing: "at a vertical offset of 100 over an 8-row view, a `cx.with_key` around each \
                      row's write lands 0 cells of 8; without it the same loop lands 8, and at \
                      offset 0 both land 8. So the defect is invisible at the one offset every \
                      caller on this map draws at",
            inverted_by: "runtime architecture issue 31",
        },
    },
    Row {
        number: 113,
        on_spec_table: false,
        gate: "a table draws its header where its body starts",
        kind: Kind::Count,
        owner: "C04",
        section: "spec §6",
        // **Found by a consumer and not by a gate**, which is what `vitui-apps` is for. The body
        // draws inside `Ctx::scroll_scope`, which childs at the body's rectangle, so a body cell's
        // `x` is relative to the table; a header cell's is relative to the caller's context.
        // Written without `head.x` the header started at column 0 whatever the rectangle said —
        // and every test on this register still passed, because they all play at `x == 0`, where
        // the two agree. `ledger` put a table inside a panel and the header landed one column into
        // the border.
        //
        // The gate is the reported double write at a non-zero origin: 276 of 2 304 when the origin
        // is right and 288 when it is not, because a header inside the body's columns is contained
        // by them. No accessor had to be added to the recorder — the number was already there.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: COLLECT,
                name: "the_header_writes_a_partition_of_its_row_in_the_same_columns",
            }],
        },
    },
    // ── components ticket 17's eight ─────────────────────────────────────────────────────────────
    Row {
        number: 114,
        on_spec_table: false,
        gate: "the flatten index's record is eight bytes, and the five names spend all four of its \
               fields",
        kind: Kind::Count,
        owner: "C05",
        section: "spec §7, §10",
        // **It was sixteen for four tickets, and the divergence had nowhere to be caught.** §7
        // states the record with its widths and §10 — where *one structure, five names* lives —
        // states the names and no widths at all; components 13 built the type from §10 and widened
        // three of the four. §7's own criterion is *a gate asserts its size*, and that criterion is
        // this ticket's, so nothing had run over it. Narrowing it is what makes §7's memory figures
        // reproduce: 7.63 MiB at a million rows and 11.44 with the prefix sum, against §7's 7 → 11.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: ORDER,
                name: "one_structure_has_five_names_and_every_field_is_earned",
            }],
        },
    },
    Row {
        number: 115,
        on_spec_table: false,
        gate: "the interval a collapse removes is read from `depth`, and the same answer read from \
               the data is one random access a row",
        kind: Kind::Equality,
        owner: "C05",
        section: "spec §7",
        // **The whole reason `depth` is in the record**, and it is an equality first: both arms
        // answer the same interval over the same forest, asserted inside the measurement, so the
        // pair is two ways of computing one number rather than two numbers. §7's 115 µs against
        // 1 187 is a timing and is `examples/tree_numbers.rs`'s.
        //
        // The index arm's *without touching the forest* is a fact about the signature —
        // `Order::subtree(&self, usize)` has nowhere in it to reach the caller's data.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: ORDER,
                    name: "depth_finds_the_interval_a_collapse_removes_without_touching_the_forest",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/tree_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 116,
        on_spec_table: false,
        gate: "a fold is a splice with no threshold and no rebuild path, and a fold on a leaf does \
               not stamp the revision",
        kind: Kind::Equality,
        owner: "C05",
        section: "spec §7, §10",
        // **`splice == rebuild` over the rows**, which is row 14's equality moved from the scene's
        // stand-in index onto the caller-owned one. The crossover at 99.7% is a doc comment on
        // `Order::fold` rather than a branch, which is §7's own instruction followed literally.
        //
        // The leaf half is the way this verb can be wrong with the screen saying nothing: a
        // revision moved for an edit that removed nothing is a `Policy::Clear` on the next frame,
        // so a user pressing `←` on a leaf would watch the selection disappear.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: ORDER,
                name: "a_fold_splices_the_interval_and_a_leaf_is_left_alone",
            }],
        },
    },
    Row {
        number: 117,
        on_spec_table: false,
        gate: "variable row height is a fourth field and a prefix sum built only when rows can \
               differ, and the window is one binary search rather than one a row",
        kind: Kind::Count,
        owner: "C05",
        section: "spec §7",
        // **A count and not a timing**, because the two spellings answer the *same rows*: no gate
        // on the picture separates `O(log n + h)` from `O(h log n)`, and the number of searches
        // does — 1 against 80. §7's 0.019 µs against 0.0006 is the report beside it.
        //
        // *Only when rows can differ* is answerable from the index (`Heights::needed`), and the
        // splice's re-accumulation is asserted as *the head is untouched* rather than as a timing.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: ORDER,
                    name: "variable_row_height_is_a_prefix_sum_built_only_when_rows_can_differ",
                },
                Instrument::Unit {
                    file: ORDER,
                    name: "the_window_is_one_binary_search_and_then_a_step",
                },
            ],
        },
    },
    Row {
        number: 118,
        on_spec_table: false,
        gate: "`tree` is `collection` plus two verbs a row, and the row it hands over is the part \
               it did not write",
        kind: Kind::Count,
        owner: "C05",
        section: "spec §7, §2",
        // **§7's own sentence**, measured against the same rectangle drawn as a plain list — the
        // difference is the mechanism and the absolutes are `crate::forest`'s screen. The second
        // half is §2 on both components at once: `writes == distinct` over the frame either way,
        // so the indent, the chevron and the label are a partition.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_tree_is_a_collection_and_the_sentence_is_read_out_of_the_file",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "the_plus_is_two_verbs_a_row_and_the_row_is_a_partition",
                },
            ],
        },
    },
    Row {
        number: 119,
        on_spec_table: false,
        gate: "the indent is clamped, and the label lands where the indent says",
        kind: Kind::Equality,
        owner: "C05",
        section: "spec §7",
        // **The equality that stops `indent_columns` and the component from being a model and a
        // copy of one.** `crate::forest` sums the function and `tree` draws the run; the rectangle
        // handed to the row drawer is where the two meet, so `label.x == indent + 1` is what pins
        // the screen's ask to the component's behaviour.
        //
        // Both directions, and at two depths: at depth three the clamped and unclamped builds place
        // the label identically — which is §21's statement about the scene list — and past the
        // rectangle the clamp reserves two columns while the defect leaves none.
        //
        // The third instrument is components ticket 15's lesson applied before it could be paid
        // for again: every other gate on this component plays at `x == 0`, where a rectangle's own
        // origin and the context's agree. `table`'s header drew from `0` and every gate passed.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: COLLECT,
                    name: "the_component_places_the_label_where_the_indent_says",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_tree_at_a_non_zero_origin_draws_inside_the_rectangle_it_was_given",
                },
                Instrument::Unit {
                    file: FOREST,
                    name: "an_unclamped_indent_asks_for_four_hundred_times_the_cells_and_costs_\
                           fewer_verbs",
                },
            ],
        },
    },
    Row {
        number: 120,
        on_spec_table: false,
        gate: "a container built on `collection` reads its own keys inside the one drain loop, and \
               they do not reach the cursor",
        kind: Kind::Count,
        owner: "C05",
        section: "spec §5, §7",
        // **A fact about the runtime rather than a preference.** `Ctx::decline` hands a key back
        // *and ends the level's turn at the queue*, so a container that drained before `collection`
        // would leave it nothing and one that drained after would find the queue closed. And the
        // collision is real: `crate::nav::step` reads `←` and `→` as `↑` and `↓`, which are exactly
        // the two keys a tree folds with.
        //
        // Fired in both directions on one screen: `←` through `tree` leaves a request and does not
        // move the cursor, and the same key through a plain `collection` moves it. `→` on a row
        // that is *not* folded is not the tree's, so a hook that swallowed every arrow would pass
        // the first assertion and lose the keyboard.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: COLLECT,
                name: "the_fold_keys_leave_a_request_and_do_not_move_the_cursor",
            }],
        },
    },
    Row {
        number: 121,
        on_spec_table: false,
        gate: "a tree row's id is keyed on the caller's node and survives a fold above it, and a \
               press on the chevron asks where a press on the label selects",
        kind: Kind::Count,
        owner: "C05",
        section: "spec §4, §7",
        // **Keyed on the node and not on the display position**, which is the half a screen cannot
        // see: a fold above a row moves the position and does not move the row, so a
        // position-keyed id would hand the row's target to whatever slid into its place. Watched
        // over a real fold, with two trees on one screen for ADR 0027's half.
        //
        // The pointer half reads `CollState::press_edge` rather than keeping a second `pressing`
        // bool — `Response::pressed` is a level, so a component reading it raw would re-ask for as
        // long as the user leaned on the button.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: COLLECT,
                    name: "identity_is_keyed_on_the_node_and_two_trees_are_two_trees",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_press_on_the_chevron_asks_and_a_press_on_the_label_does_not",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "the_frame_does_not_move_with_the_height_field",
                },
            ],
        },
    },
    // ── components ticket 19: `scroll_area`, `scrollbar` and `sticky` ────────────────────────────
    Row {
        number: 122,
        on_spec_table: false,
        gate: "the parts of a reserved scroll area tile its rectangle exactly, at every size",
        kind: Kind::Equality,
        owner: "C13",
        section: "spec §9",
        // **ADR 0029 as arithmetic rather than as a measurement.** Every cell of the rectangle
        // belongs to exactly one part — the body's viewport, the two bars, the corner and the four
        // bands — at every size a rectangle comes in and under both `Hide` spellings. **An overlay
        // bar cannot satisfy this at all**, because the cells under it belong to two, which is why
        // the rule is unconditional and not a default: *free where nothing is drawn under it* is
        // not a property any component can guarantee of its body.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: SCROLL,
                    name: "the_parts_of_a_reserved_area_tile_its_rectangle_exactly",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "an_overlay_bar_re_damages_what_the_body_draws_under_it",
                },
            ],
        },
    },
    Row {
        number: 123,
        on_spec_table: false,
        gate: "a band is one construction with an axis argument, and the four bands declare no hit \
               entry of their own",
        kind: Kind::Count,
        owner: "C13",
        section: "spec §9",
        // **The count is that the hit count does not move.** A frame with four bands standing and
        // the same frame with none declare the same three regions — the area and its two bars —
        // because a band that published one would be a second scroll area and would win the wheel
        // from the body it is a header of (R17 §5). The construction half is a source scan: one
        // `scrolled` band view in `scroll.rs` and the negative case beside it.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: SCROLL,
                    name: "a_band_is_one_construction_with_an_axis_argument",
                },
                Instrument::Unit {
                    file: SCROLL,
                    name: "four_bands_standing_declare_no_hit_entry_of_their_own",
                },
            ],
        },
    },
    Row {
        number: 124,
        on_spec_table: false,
        gate: "a band drawn by arithmetic instead of into a view re-damages its overrun on every \
               steady frame, at identical verbs",
        kind: Kind::Count,
        owner: "C13",
        section: "spec §9, §6",
        // **Spec §6's pinned-column finding on the other axis, which is what §9 asks to be shown.**
        // The two arms are one `Ctx::child` apart: the same cells at the same coordinates through
        // the same verbs, and the same origin. The view clips the overrun, the arithmetic spelling
        // writes it onto the body below, the body draws afterwards and wins — so **the picture is
        // identical** and the only counter that moves is the damage.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: SCROLL,
                name: "a_band_is_a_view_and_the_arithmetic_spelling_re_damages_what_is_under_it",
            }],
        },
    },
    Row {
        number: 125,
        on_spec_table: false,
        gate: "`[extent, offset + viewport)` is written by the owner of the rectangle: a shrunk \
               extent leaves no unwritten cell",
        kind: Kind::Count,
        owner: "C13",
        section: "spec §9, §2",
        // §9 assigns this line by name: *the offset clamp is free, because `max` is recomputed
        // every frame; the tail is not.* The body draws what the content admits and the rest of the
        // rectangle is the component's — left out, a scroll area whose extent has just shrunk shows
        // the old rows under a correct offset.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: SCROLL,
                name: "a_shrunk_extent_leaves_no_unwritten_tail",
            }],
        },
    },
    Row {
        number: 126,
        on_spec_table: false,
        gate: "the extent, the offset, the thumb and scroll-into-view are all content cells, and \
               an offset past the end clamps on the next frame",
        kind: Kind::Equality,
        owner: "C05",
        section: "spec §9",
        // **C05's debt, collected at the four sites rather than at one.** Over content where one
        // row in eight is three cells tall the row count and `Σ h` differ by a quarter; the extent
        // bounds the offset in cells, the thumb is sized from the same two numbers, and the reveal
        // arrives as a delta in the same unit and is applied without a conversion. A reveal
        // measured in rows lands a quarter short at the bottom of the content, on a screen that
        // looks perfectly healthy — which is `crate::area`'s row 799 999 of 999 999 one verb down.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: SCROLL,
                    name: "the_extent_the_offset_the_thumb_and_the_reveal_are_all_content_cells",
                },
                Instrument::Unit {
                    file: SCROLL,
                    name: "an_offset_past_the_end_clamps_on_the_next_frame",
                },
            ],
        },
    },
    Row {
        number: 127,
        on_spec_table: false,
        gate: "a thousand rows and a million are the same frame — identical writes and identical \
               verbs",
        kind: Kind::Equality,
        owner: "C21",
        section: "spec §9",
        // The counter form of the standing budget's own invariant — *frame cost is proportional to
        // visible cells, never to data volume* — over a body that reads the window the scope
        // published. The other half of C21 is `crate::area`'s scene 30, where a body that iterates
        // its whole content instead is measured at 100 000 rows against 69 **at identical writes**,
        // because the engine reports a fully clipped verb as zero columns.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: SCROLL,
                    name: "a_thousand_rows_and_a_million_are_the_same_frame",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "a_scroll_area_costs_its_content_and_a_collection_costs_its_window",
                },
            ],
        },
    },
    Row {
        number: 128,
        on_spec_table: false,
        gate: "a recorded surface holds a component's cells where they landed, not where the verb \
               was called",
        kind: Kind::Count,
        owner: "C13",
        section: "spec §9, §6",
        // **The instrument's own row, and it is a defect this ticket found in the instrument.**
        // `crate::runner::Pen` models the surface a component draws on and recorded a verb at the
        // coordinates it was *called* with: two scroll areas sixty columns apart recorded their
        // bands as one, and a component narrowed to `x == 5` recorded its first cell at column 0.
        // It is components ticket 15's finding from the third side — *a translated band makes
        // `distinct` meaningless* — and the repair is `vitui_runtime::Ctx::origin`, which the
        // runtime did not publish until runtime architecture issue 32.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_tree_at_a_non_zero_origin_draws_inside_the_rectangle_it_was_given",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "the_two_areas_scene_stands_on_a_shipped_scroll_area",
                },
            ],
        },
    },
    Row {
        number: 129,
        on_spec_table: false,
        gate: "every subject the wheel gate runs against declares `Axis::Wheeled` and is built",
        kind: Kind::Count,
        owner: "C10",
        section: "spec §17",
        // **Ticket 20's criterion 6, and it is a join rather than a claim.**
        //
        // The gate above runs over two subjects, and *which* two is a decision this crate makes in
        // one file. `INVENTORY`'s `owns_offset` column is exactly `Axis::Wheeled`, so asking whether
        // the freeze agrees is one question — and the failure it catches is a gate run over a
        // component the freeze says owns no offset, which reports a number while measuring nothing.
        // It is the same shape as O5 and for the same reason: **O5 is a query about axes and not
        // about components**, so a pair claimed here and unclaimed there is evidence for nothing.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: WHEEL,
                    name: "every_subject_of_this_gate_declares_the_wheeled_axis",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/obligations.rs",
                    name: "o5_counts_a_pair_for_every_axis_the_wheel_gates_subjects_declare",
                },
            ],
        },
    },
    // ── components ticket 22's six, and three of them are findings rather than criteria ──────────
    Row {
        number: 130,
        on_spec_table: false,
        gate: "the surface after a collapse equals a freshly built one, at the verb boundary",
        kind: Kind::Equality,
        owner: "C14",
        section: "spec §8, §21",
        // **This is not row 21, and the distinction is row 41's to row 2's.** Row 21 asks for the
        // equality between two *composited* surfaces and ADR 0023 hands over no cell; this is the
        // crate's own model of what it drew, recorded by a `Pen` carried **across** frames
        // (`Pen::over`, which is what `crate::area` uses for re-damage) — because residue is a
        // relation between two frames and a pen built per frame has nothing to relate to.
        //
        // **Both arms are compared with the focus seated on the header and no pointer anywhere**,
        // which is why the gesture is `Enter` and not a click: a click leaves the pointer on the
        // header, and the two arms would then differ by one cell of hover paint rather than by a
        // residue. That was measured before it was designed around.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DISCLOSE,
                    name: "the_surface_after_a_collapse_equals_a_freshly_built_one",
                },
                Instrument::Unit {
                    file: DISCLOSE,
                    name: "the_tail_is_the_rectangles_owners_and_the_residue_is_what_it_costs",
                },
            ],
        },
    },
    Row {
        number: 131,
        on_spec_table: false,
        gate: "a measured extent taken inside the rectangle the last decision produced latches and \
               never comes down: 19 rows where 4 are right",
        kind: Kind::Count,
        owner: "C13, C14",
        section: "spec §8, §9",
        // **§9's own sentence on the height axis, and it is the reason `Height::Watermark` is off by
        // default.** §8 prices the drawn extent at *83 cells, 8 rows wrong, settled in 3 frames*
        // against a sizing function's *22, 0, 2*, and those three figures are a prototype's **body**
        // — they do not reproduce and are not made to. What reproduces is stronger and is already on
        // this map one section over: *a measured extent and a hideable reserved bar are incompatible,
        // because the measurement is taken inside the rectangle the decision produced.*
        //
        // The precondition is on the **body**, exactly as §9 states it: over a body that draws only
        // its content the two arms are **indistinguishable**, which is what makes the rule
        // unconditional rather than a preference. Both bodies are in the instrument.
        //
        // And the price is a **count** rather than a clock: the dry run goes through the caller's
        // ink, so the watermark arm makes the body's verbs twice. §8's *+11.5% of the frame* is a
        // report beside it.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DISCLOSE,
                    name: "the_sizing_function_is_right_on_every_frame_and_a_measured_extent_latches",
                },
                Instrument::Unit {
                    file: DISCLOSE,
                    name: "the_watermark_pays_for_a_second_pass_over_the_body_and_the_sizing_\
                           function_pays_for_none",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/collapsible_numbers.rs",
                },
            ],
        },
    },
    Row {
        number: 132,
        on_spec_table: false,
        gate: "no gesture that closes a section from its own header reaches the vanish rule, and \
               the rule is what keeps the keyboard on a header that is not a tab stop",
        kind: Kind::Count,
        owner: "C14",
        section: "spec §8",
        // **§8's *0 ring probes against 405*, followed to the end.** §8 says out loud why the left
        // half is free — *which is what a focusable widget does on a click anyway* — and the
        // consequence is that all three self-close gestures leave the focus off the body before the
        // vanish rule looks, each for a different reason: the press award focuses a focusable header;
        // the press award **defocuses** on one that is not a tab stop, because a press landing on
        // nothing interested is read as intent; and `Enter` needs the header to hold the focus
        // already. So the arm that pays is a collapse with **no gesture behind it**, which is where
        // `crate::accordion` runs it — a collapse-all with and without the caller capturing
        // `Frame::focus`, which is §8's own answer to *`Stash` belongs to whoever owns the content's
        // identity*.
        //
        // What `Focus::Header` buys is measured rather than argued, and it is not the probe count: on
        // a header that is not a tab stop the runtime's answer is `None`, so the click **loses the
        // keyboard entirely** — runtime architecture issue 25's finding one component over.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DISCLOSE,
                    name: "no_self_close_gesture_reaches_the_vanish_rule_and_the_rule_is_what_\
                           keeps_the_keyboard",
                },
                Instrument::Unit {
                    file: ACCORDION,
                    name: "a_collapse_with_no_gesture_behind_it_is_where_the_vanish_rule_answers",
                },
            ],
        },
    },
    Row {
        number: 133,
        on_spec_table: false,
        gate: "a fold steps and a region animates: no tween is started under `Collapses::OnRequest`, \
               and a region collapse lands on the frame the gesture does",
        kind: Kind::Count,
        owner: "C14",
        section: "spec §8",
        // **§8's *an animated fold is refused* as a count rather than as a doc comment.** The removed
        // rows would have to still be in the index while they shrink, and the splice is an edit a
        // frame may not perform — so the index arm leaves the state untouched, starts nothing, and
        // asks for no further frame, while the region arm is already collapsed before its frame ends.
        //
        // The screen-scale half is the declarations: a section that closed a frame late would still
        // declare its body's 34 entries on the frame the click landed, so *on the same frame* is
        // checkable rather than a sentence.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DISCLOSE,
                    name: "a_region_collapses_on_the_frame_and_an_index_collapse_is_a_request_the_\
                           caller_drains",
                },
                Instrument::Unit {
                    file: DISCLOSE,
                    name: "a_region_with_a_duration_animates_and_asks_for_its_own_frames_until_it_\
                           lands",
                },
                Instrument::Unit {
                    file: ACCORDION,
                    name: "a_click_on_an_open_header_goes_six_open_to_five_on_the_frame_it_lands",
                },
            ],
        },
    },
    Row {
        number: 134,
        on_spec_table: false,
        gate: "§8's three-row split is three rows and each is exactly one configuration of one \
               machine",
        kind: Kind::Equality,
        owner: "C14",
        section: "spec §8, §18",
        // **Criterion 1, and it is a bijection rather than a list.** *Accordion, tree node, code
        // folding and inplace edit are one machine* is the claim §18's whole F4 family exists to
        // make, and the shape of claim §17 says gets broken by people who have read it — so the
        // table is a value and the join is asserted: three rows, three arms of `Collapses`, and no
        // fourth on either side. `animates()` is `applies()` everywhere, which is *a fold steps; a
        // region animates* as arithmetic over the same value.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: DISCLOSE,
                name: "the_split_is_three_rows_and_each_one_is_one_configuration_of_one_machine",
            }],
        },
    },
    Row {
        number: 135,
        on_spec_table: false,
        gate: "a `Collapse` is four bytes live and the tween slot costs forty whether or not it \
               holds one",
        kind: Kind::Count,
        owner: "C14",
        section: "spec §8",
        // **§8's *5 B of live state, 72 B with a tween slot*, and neither number reproduces** —
        // asserted as a disagreement, which is components ticket 17's standing one family over where
        // §7's stated widths were the number that was wrong.
        //
        // The live half is `open: bool` and a `u16`; five is what a third `u16` would cost and there
        // is no third, because `Tween::to` is the target while a tween runs and the height is it
        // afterwards. **And the pair cannot both be a `size_of` of one type**, which is the part
        // worth keeping: `Option<Tween<u16>>` is a field, so it costs its forty bytes empty. A record
        // that is 5 B without a tween and 72 B with one is a record whose tween lives somewhere else,
        // and nothing on this map has anywhere else to put one.
        //
        // The half that does hold is *never per row of content*: the fold set is line numbers beside
        // the caller's document, four bytes each, and twelve sections of state is cheaper than the
        // 4 167 folds it is not.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DISCLOSE,
                    name: "a_collapse_is_four_bytes_live_and_the_slot_costs_forty_whether_or_not_\
                           it_holds_one",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/collapsible_numbers.rs",
                },
            ],
        },
    },
];
/// **The compile-outcome pair row 31 names, and its positive twin.**
///
/// # The twin, naming the protected items by path
///
/// A lone `compile_fail` passes when the type has been renamed — `E0433` instead of `E0277`, and the
/// mechanism cannot tell those apart — so the shape that ships is pinned here first. Renaming
/// [`vitui_runtime::work::Task`] or [`vitui_runtime::work::Worker`] fails **this** half rather than
/// making the half below pass for the wrong reason.
///
/// ```
/// use vitui_runtime::work::{Task, Worker};
///
/// fn assert_send<T: Send>() {}
/// fn assert_send_sync<T: Send + Sync>() {}
///
/// // The half that crosses a thread boundary.
/// assert_send::<Worker>();
/// assert_send_sync::<Worker>();
///
/// // And the half that does not, exercised so a rename cannot pass silently.
/// let worker = Worker::queueing();
/// let task: Task<u32> = Task::new(&worker);
/// assert_eq!(task.asking(), None);
/// ```
///
/// # The hostile half
///
/// **Protects:** [`vitui_runtime::work::Task`].
///
/// `Task` holds three `Cell`s and a `PhantomData<*const ()>` brand, so a shared reference to one
/// cannot cross a thread. That is what makes a component's async state the app thread's alone — spec
/// §17's G10.
///
/// ```compile_fail,E0277
/// use vitui_runtime::work::Task;
///
/// fn assert_sync<T: Sync>() {}
/// assert_sync::<Task<u32>>();
/// ```
///
/// # §21 says `Worker` is `!Sync`, and the shipped one is not
///
/// That row reads "`Task` and `Worker` are `!Sync`; `Cell`, `RefCell`, `Worker` **are** `Send`".
/// `Worker` is an `Arc<Inbox>` and is `Sync` as well as `Send`, which the twin above asserts rather
/// than works around. The half that is load-bearing survives: what a component may not share is its
/// own `Task`, and the pair is written on that.
///
/// # Why the pair hangs on a marker
///
/// The protected items are in **another crate**, so there is no declaration here to put the fences
/// on. A `#[cfg(doc)]` carrier is the engine's answer to that shape — a type whose whole purpose is
/// to hold a pair — and it costs the shipped surface nothing: rustdoc sets `--cfg doc` when it
/// documents and when it collects doctests, so both halves run under `cargo test --doc` and neither
/// exists in a build.
#[cfg(doc)]
pub struct WhyATaskCannotCrossAThread;

/// The register as a table, for [`crate::gates`]'s report and for a human.
///
/// One line a row plus a heading, which is what `tests::the_register_prints` asserts.
pub fn table() -> String {
    use std::fmt::Write as _;
    let mut out = String::from("  #  kind             owner                 section   standing\n");
    for row in REGISTER {
        let _ = writeln!(
            out,
            "{:>3}  {:<15}  {:<20}  {:<8}  {}",
            row.number,
            row.kind.word(),
            row.owner,
            row.section,
            row.standing.word()
        );
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

    fn workspace_root() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    fn read(relative: &str) -> String {
        let path = workspace_root().join(relative);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// Every `.rs` file under `dir`, recursively, skipping build output, git metadata and
    /// nested checkouts.
    ///
    /// **`.claude` is skipped because a nested checkout is not this workspace.** Agent worktrees
    /// land at `.claude/worktrees/<name>/`, and a worktree's `.git` is a *file* rather than a
    /// directory, so the `.git` arm below does not reach them. Without this arm every scan that
    /// asks *how many places in the workspace do X* answers with one copy per worktree standing:
    /// the counting-allocator scan read eleven `vitui-alloc-probe/src/lib.rs` against one, and
    /// failed naming ten paths that are the same file. That is the scan's subject drifting, not a
    /// defect it caught — so the arm is a correction rather than a loosening, and the gate still
    /// fails for a second allocator committed anywhere under `crates/`.
    fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let path = entry.expect("a readable entry").path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.is_dir() {
                if name == "target" || name == ".git" || name == ".claude" {
                    continue;
                }
                rust_files(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    /// Every doc code block in a source string, as `(attributes, body)`.
    ///
    /// One state machine for both halves of a pair, which is the engine's arrangement and its
    /// reason: counting only the `compile_fail` fences protects the hostile line and leaves the twin
    /// unguarded, and the twin is the half that catches a rename.
    fn doc_blocks(source: &str) -> Vec<(String, String)> {
        let mut out = Vec::new();
        let mut open: Option<(String, Vec<&str>)> = None;
        for line in source.lines() {
            let trimmed = line.trim_start();
            let Some(doc) = trimmed
                .strip_prefix("///")
                .or_else(|| trimmed.strip_prefix("//!"))
            else {
                continue;
            };
            let doc = doc.strip_prefix(' ').unwrap_or(doc);
            match doc.strip_prefix("```") {
                Some(attributes) => match open.take() {
                    Some((attrs, body)) => out.push((attrs, body.join("\n"))),
                    None => open = Some((attributes.trim().to_string(), Vec::new())),
                },
                None => {
                    if let Some((_, body)) = open.as_mut() {
                        body.push(doc);
                    }
                }
            }
        }
        assert!(open.is_none(), "a doc code block never closed its fence");
        out
    }

    /// Whether `source` declares `name` as a **runnable `#[test]`**.
    ///
    /// A raw `source.contains("fn {name}(")` is the vacuous version this file exists to refuse: it
    /// matches inside a doc comment, inside a string literal and inside a commented-out block, and it
    /// matches a plain helper just as well as a test. A `#[test]` demoted to a helper — or left in
    /// place with `#[ignore]` on it — would keep a row reading `Evaluated` while nothing ran.
    fn declares_a_live_test(source: &str, name: &str) -> bool {
        let signature = format!("fn {name}(");
        let lines: Vec<&str> = source.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if !line.trim_start().starts_with(&signature) {
                continue;
            }
            let mut attributed = false;
            let mut ignored = false;
            for above in lines[..index].iter().rev() {
                let above = above.trim();
                if above.starts_with("#[test]") {
                    attributed = true;
                } else if above.starts_with("#[ignore") {
                    ignored = true;
                } else if !(above.starts_with("#[")
                    || above.starts_with("///")
                    || above.starts_with("//")
                    || above.is_empty())
                {
                    break;
                }
            }
            if attributed && !ignored {
                return true;
            }
        }
        false
    }

    /// Whether `source` carries `needle` on a line that is **not a comment**.
    ///
    /// One function for the two source scans below and for both of their negative halves, which is
    /// what makes *fires in both directions* mean something here: a hostile fixture run through a
    /// second copy of the logic proves the copy and not the gate.
    fn carries(source: &str, needle: &str) -> bool {
        source
            .lines()
            .map(str::trim)
            .any(|line| !line.starts_with("//") && line.contains(needle))
    }

    /// **Every row names a destination, and the numbers are 1..=61 once each.**
    #[test]
    fn every_row_names_a_destination() {
        for row in REGISTER {
            assert!(
                !row.gate.is_empty() && !row.owner.is_empty(),
                "row #{} has no gate or no owner",
                row.number
            );
            assert!(
                row.section.starts_with("spec §"),
                "row #{} cites `{}`, which is not a section of the components spec",
                row.number,
                row.section
            );
            let inverted_by = match row.standing {
                Standing::Evaluated { by } => {
                    assert!(
                        !by.is_empty(),
                        "row #{} is evaluated without saying on what",
                        row.number
                    );
                    None
                }
                Standing::Red {
                    by,
                    failing,
                    inverted_by,
                } => {
                    assert!(
                        !by.is_empty(),
                        "row #{} is red without an instrument. A red row that nothing runs is a \
                         sentence about a gate, which is what this register replaces",
                        row.number
                    );
                    assert!(
                        failing.chars().any(|c| c.is_ascii_digit()),
                        "row #{} is pinned red without a number in its failing set",
                        row.number
                    );
                    Some(inverted_by)
                }
                Standing::Unreachable { needs, inverted_by } => {
                    assert!(
                        needs.len() > 60,
                        "row #{} is unreachable without naming what would have to become public",
                        row.number
                    );
                    Some(inverted_by)
                }
                Standing::Unsubjected { inverted_by } => Some(inverted_by),
            };
            if let Some(inverted_by) = inverted_by {
                assert!(
                    inverted_by.starts_with("components ")
                        || inverted_by.starts_with("runtime architecture issue "),
                    "row #{}'s `{inverted_by}` is neither an implementation ticket nor a filed \
                     architecture issue",
                    row.number
                );
            }
        }

        let seen: BTreeSet<u8> = REGISTER.iter().map(|r| r.number).collect();
        assert_eq!(seen.len(), REGISTER.len(), "a number appears twice");
        let expected: BTreeSet<u8> = (1..=u8::try_from(REGISTER.len()).expect("fits")).collect();
        assert_eq!(seen, expected);
    }

    /// **Forty-eight evaluated, and the other twenty-four each say why not.**
    ///
    /// This is the number §21 asks for: *how many gates are actually evaluated is a number a test
    /// asserts rather than a claim in a document*. Saying it out loud is what stops the next change
    /// arriving unremarked — a row that quietly stops running has to edit this line, and a row that
    /// starts running has to edit it too.
    #[test]
    fn a_hundred_and_eighteen_rows_are_evaluated_and_the_rest_say_why_not() {
        let mut evaluated = 0usize;
        let mut red = Vec::new();
        let mut unreachable = Vec::new();
        let mut unsubjected = 0usize;
        for row in REGISTER {
            match row.standing {
                Standing::Evaluated { .. } => evaluated += 1,
                Standing::Red { .. } => red.push(row.number),
                Standing::Unreachable { .. } => unreachable.push(row.number),
                Standing::Unsubjected { .. } => unsubjected += 1,
            }
        }
        assert_eq!(evaluated, EVALUATED, "the count §21 asks a test to assert");
        assert_eq!(
            red,
            vec![7, 8, 85, 89, 112],
            "the gates that are red and pinned: the sentinel, the palette after a swap and the \
             scenes still waiting for their subject. **Twenty wheel clicks is not among them since \
             components 20** — row 29 was red on a *defect* rather than on a missing subject, which \
             is why it took a gate over the shipped path and two removed substitutions rather than \
             a standing edit. Five have been \
             inverted and each says what it took — the glyph-set count by components 05, row 61 \
             by components 10 (which took rewriting the *gate* rather than the standing, because \
             the row asserted an *absence*), row 78 by components 15, which took `table` being \
             declared **and** `crate::grid::draw_into` calling it, and row 74 by components 17 on \
             the same two conditions for `tree`"
        );
        assert_eq!(
            unreachable,
            vec![1, 2, 21, 28, 33, 45],
            "the six that cannot be written from a crate whose dependency list is \
             `vitui-runtime` and nothing else. **It was seven until components ticket 08**, which \
             found row 5's barrier misread: `Mods` is unnameable here and `Chord::mods` hands over \
             the value anyway, so a chord can be pressed after all"
        );
        assert_eq!(
            unsubjected, 6,
            "and the six with nothing to run over. **It was eight until components ticket 22**, \
             which supplied the subject for rows 24 and 25 — the inplace map and *`open` is never \
             ambiguous mid-transition*, the last two `Unsubjected` rows on §21's own table. It was \
             fifteen until ticket 18, which supplied row 20's: the bar fixpoint is arithmetic over a \
             domain and needs no component to be run against, and its row had been citing spec §13 \
             and components 28 for two tickets"
        );
        assert_eq!(evaluated + red.len() + unreachable.len() + unsubjected, 135);
    }

    /// **The split, not the total.**
    ///
    /// §21's table is thirty-two rows and it is closed; everything after it is a gate a ticket on
    /// this lineage wrote. Asserting the split is what made the forty-fifth row — components ticket
    /// 05's matrix barrier — say which side of the line it is on, and a thirty-third row claiming to
    /// be §21's is a spec change, which should not be able to arrive as a one-line diff in this
    /// file.
    #[test]
    fn thirty_two_rows_are_the_specs_and_a_hundred_and_three_are_this_lineages() {
        let on_table = REGISTER.iter().filter(|r| r.on_spec_table).count();
        assert_eq!(on_table, SPEC_ROWS);
        assert_eq!(REGISTER.len() - on_table, 103);
        for (index, row) in REGISTER.iter().enumerate() {
            assert_eq!(
                row.on_spec_table,
                index < SPEC_ROWS,
                "row #{} is out of the spec-table block",
                row.number
            );
        }
    }

    /// **Every instrument names something that exists.**
    ///
    /// The test the whole file is shaped around, inherited from runtime ticket 19. A `Unit` must be
    /// a live `#[test]` in its file, a `Pair`'s hostile line must appear **inside a `compile_fail`
    /// block** — a mention in prose does not count, and neither does the twin — a `Barrier` is
    /// compared against a **trimmed** line, and a `Report` must be a file that is there and not
    /// empty.
    #[test]
    fn every_instrument_names_something_that_exists() {
        for row in REGISTER {
            let by = match row.standing {
                Standing::Evaluated { by } | Standing::Red { by, .. } => by,
                Standing::Unreachable { .. } | Standing::Unsubjected { .. } => continue,
            };
            for instrument in by {
                let source = read(instrument.file());
                match *instrument {
                    Instrument::Unit { file, name } => assert!(
                        declares_a_live_test(&source, name),
                        "row #{}: `{file}` has no live `#[test] fn {name}`. A helper `fn` of that \
                         name, or one carrying `#[ignore]`, leaves this row reading as run while \
                         nothing runs",
                        row.number
                    ),
                    Instrument::Pair { file, hostile } => {
                        let found = doc_blocks(&source).iter().any(|(attrs, body)| {
                            attrs.contains("compile_fail") && body.contains(hostile)
                        });
                        assert!(
                            found,
                            "row #{}: no `compile_fail` block in `{file}` contains `{hostile}`. A \
                             mention in prose is not the gate",
                            row.number
                        );
                    }
                    Instrument::Barrier { file, line } => assert!(
                        source.lines().any(|l| l.trim() == line),
                        "row #{}: `{file}` no longer carries the line `{line}`, so the barrier this \
                         row is stopped by has moved or lifted. That is a change of standing and \
                         not a broken test",
                        row.number
                    ),
                    Instrument::Report { file } => assert!(
                        !source.is_empty(),
                        "row #{}: the report `{file}` is empty",
                        row.number
                    ),
                }
            }
        }
    }

    /// **No row rests on a report** — R15's refinement 2, as a test.
    ///
    /// `cargo clippy --all-targets` compiles every `examples/*.rs` and evaluates none of them, so an
    /// `assert!` inside one is a gate only if a CI job runs the binary. This workspace's jobs run
    /// `cargo test` and three named examples, and `gates_numbers` is not among them. A row may
    /// **cite** a report — the number a human reads beside the gate is worth having — and never rest
    /// on one.
    #[test]
    fn no_row_rests_on_a_report() {
        for row in REGISTER {
            let Standing::Evaluated { by } = row.standing else {
                continue;
            };
            assert!(
                by.iter().any(|i| !matches!(i, Instrument::Report { .. })),
                "row #{} is evaluated by a report and nothing else",
                row.number
            );
        }
    }

    /// **An equality is claimed only where the number belongs to the mechanism.**
    ///
    /// R15's refinement 1. There is no way to decide from the outside whether a number belongs to
    /// the mechanism or to the data, so what is checked is that the row **says which**: a row whose
    /// gate is an inequality has to be a [`Kind::Relation`], and the one row that spells an
    /// inequality out — `verbs <= writes`, §21's own example of *never verb equality across sizes* —
    /// is the case this catches.
    #[test]
    fn an_inequality_is_never_filed_as_an_equality() {
        for row in REGISTER {
            let inequality = row.gate.contains("<=") || row.gate.contains(">=");
            if inequality {
                assert_eq!(
                    row.kind,
                    Kind::Relation,
                    "row #{} states an inequality and calls itself a {}",
                    row.number,
                    row.kind.word()
                );
            }
        }
    }

    /// **An allocation row says `total`, because the word is the rule.**
    ///
    /// Refinement 2 has no type to live in on this side of the register — [`crate::counters::Allocations`]
    /// is where it lives in code — so what is enforced here is that a row about allocations cannot be
    /// written without the word. Every prototype's spelling was `allocs / n`, and a row saying
    /// *zero allocations in a steady frame* with no qualifier is that spelling with the division
    /// hidden.
    #[test]
    fn an_allocation_row_says_it_is_a_total() {
        let rows: Vec<&Row> = REGISTER
            .iter()
            .filter(|r| r.gate.contains("allocation"))
            .collect();
        assert_eq!(
            rows.len(),
            4,
            "rows 32 and 36, ticket 23's row 82 and ticket 25's row 87"
        );
        assert!(
            rows.iter().any(|r| r.gate.contains("total")),
            "the steady-frame allocation row must say `total`"
        );
    }

    /// **`vitui-alloc-probe` is the only counting allocator in the workspace.**
    ///
    /// §19's finding: twelve component prototypes each hand-rolled one. The scan is over every `.rs`
    /// file in the repository — including the four detached workspaces, which `cargo test
    /// --workspace` cannot reach — for a `GlobalAlloc` implementation, and there must be exactly one.
    ///
    /// A `#[global_allocator]` **static** is not one of these and is not counted: installing the
    /// probe in a test binary is the intended use, and seven files do it.
    #[test]
    fn vitui_alloc_probe_is_the_only_counting_allocator_in_the_workspace() {
        let mut files = Vec::new();
        rust_files(&workspace_root(), &mut files);
        assert!(files.len() > 50, "the walk found nothing to look at");

        // Assembled from fragments so that **this file** does not match itself. A source scan whose
        // own source is a hit is the vacuous shape the engine's register records having shipped once
        // already, and it is the same reason the glyph-set scan below builds its needles.
        //
        // The trait name and `for`, with no `impl` in front: a hand-rolled allocator is as likely to
        // be written path-qualified, and the narrower spelling missed exactly that when it was first
        // tried against a real one.
        let definition = concat!("GlobalAlloc", " for");

        let mut definitions = Vec::new();
        for path in &files {
            let source = std::fs::read_to_string(path).unwrap_or_default();
            if carries(&source, definition) {
                definitions.push(
                    path.strip_prefix(workspace_root())
                        .unwrap_or(path)
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            }
        }
        definitions.sort();
        assert_eq!(
            definitions,
            vec!["crates/vitui-alloc-probe/src/lib.rs".to_string()],
            "the crate that exists for this is the one to use (§19). A second counting allocator \
             is twelve prototypes' mistake arriving again"
        );

        // **The other direction, through the same function.** A scan that has quietly stopped
        // scanning reports zero as loudly as a clean workspace does. Both spellings are found — the
        // narrower needle missed the path-qualified one when it was first tried against a real
        // hand-rolled allocator — and a mention in a comment is not a definition.
        //
        // The fixtures are **interpolated** rather than written out, for the reason the needle is:
        // a literal here is a line of this file, and this file is inside the walk. Writing them out
        // is what the first version did, and it failed on itself.
        assert!(carries(
            &format!("unsafe impl {definition} Counting {{}}"),
            definition
        ));
        assert!(carries(
            &format!("unsafe impl std::alloc::{definition} Counting {{}}"),
            definition
        ));
        assert!(!carries(
            &format!("// a note about {definition} a reader"),
            definition
        ));
    }

    /// **No component here names a glyph set, and no crate in the workspace carries a private
    /// fallback table.**
    ///
    /// Row 26, and **components ticket 05 inverted it.** It was pinned red over an empty population
    /// — 0 occurrences over 0 components, which is `Verdict::of`'s vacuity refusal stated in a
    /// different file — and what was missing was the subject rather than the instrument.
    ///
    /// # What made it a subject, which is not *a component was built*
    ///
    /// The thing a private fallback module substitutes for is **code that has to spell a glyph and
    /// cannot add an entry to the table**. Nine crates of twelve grew one; the gate asks that a
    /// crate doing that work names no repertoire. [`crate::glyphs`] is that work: it reads
    /// `Theme::glyph` for all twenty entries across the whole freeze, joins them into six families,
    /// takes the collapse census and runs the memo rule — and it names a role and a glyph and never
    /// a repertoire. So the count is over something now.
    ///
    /// The second half is widened to the workspace, which is what §16 asks for and what this could
    /// not do while it had nothing to be about: **no private fallback module anywhere**, engine,
    /// runtime and components together.
    ///
    /// The needles are assembled from fragments so that **this file** does not contain them. A source
    /// scan whose own source matches it is the vacuous shape the engine's register records having
    /// shipped once already.
    ///
    /// # One file is excepted, by name and by count — components ticket 27
    ///
    /// `CONTEXT.md` says both halves of a collision in two adjacent paragraphs. **Repertoire**: *a
    /// component branches on it rather than the engine substituting behind its back.* **Glyph**:
    /// *anything failing either rule is not a glyph, it is a branch — **the sub-cell ladders are the
    /// case**, because the number of samples asked of the data changes with the rung and no table
    /// can carry that … a component names no repertoire.* The sub-cell ladder is named in the second
    /// sentence as the thing that is a branch, and a branch on the repertoire is a component naming
    /// the repertoire. Spec §13 says the same in as many words: *§16 owns the lookup; the branch is
    /// two `match`es and a 16-entry array in the component crate.*
    ///
    /// Both cannot hold, and §21's own refinement 3 says what to do about it: **name the exception;
    /// do not loosen the gate.** So `crate::chart::raster::geom` is excepted by file name, and the number
    /// of occurrences in that file is asserted — a second file fails, and a fourth occurrence in
    /// this one fails. What is *not* excepted is the thing row 26 is actually about: no private
    /// fallback table, anywhere, including there.
    #[test]
    fn no_component_here_names_a_glyph_set_and_none_has_a_private_missing_table() {
        let glyph_set = concat!("GlyphSet", "::");
        let private_table = concat!("mod ", "missing");

        // **The one exception, and it is a file rather than a rule.** The sub-cell ladder, which
        // `CONTEXT.md` names as a branch in the same paragraph that forbids naming a repertoire.
        // Components ticket 28 moved it from `series.rs` — the screen — to `chart/raster.rs`, which
        // is where the branch belongs, and the three lines moved with it.
        const LADDER: &str = "chart/raster.rs";
        // **The lines that may spell one, exactly.** A list rather than a count, because a count
        // says *how many* and this has to say *which*: two arms of one `match`, plus the rung the
        // correct build is drawn at. Anything else — a second branch, a repertoire test moved out
        // of the test module, a spelling table — fails here and the failure prints the line.
        //
        // **Assembled from `glyph_set` rather than written out**, for the reason the needles above
        // are: a scanner whose own source matches its needle reports itself, which is the vacuous
        // shape the engine's register records having shipped once already — and this test watched
        // it happen on its first run with these three lines written literally.
        let ladder_lines = [
            format!("(_, {glyph_set}Ascii) => Geom {{ sx: 1, sy: 1 }},"),
            format!("(Kind::Marks, {glyph_set}Unicode) => Geom {{ sx: 2, sy: 2 }},"),
            format!(
                "pub const RUNGS: [GlyphSet; 3] = [{glyph_set}Ascii, {glyph_set}Unicode, \
                 {glyph_set}Extended];"
            ),
        ];

        let mut files = Vec::new();
        rust_files(
            &PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")),
            &mut files,
        );
        let mut scanned = 0usize;
        let mut offenders = Vec::new();
        let mut excepted = 0usize;
        for path in &files {
            let source = std::fs::read_to_string(path).unwrap_or_default();
            scanned += 1;
            let names_a_set = carries(&source, glyph_set);
            let is_the_ladder = path.to_string_lossy().ends_with(LADDER);
            if names_a_set && is_the_ladder {
                excepted += 1;
                // **The shipped half of the file**, which is what the rule is about: a test
                // that sweeps the three rungs names them by definition, and forbidding that would
                // forbid *measuring* the ladder rather than forbidding a second one.
                let shipped = source
                    .split("#[cfg(test)]")
                    .next()
                    .expect("split always yields one");
                let naming: Vec<&str> = shipped
                    .lines()
                    .filter(|line| carries(line, glyph_set))
                    .map(str::trim)
                    .collect();
                assert_eq!(
                    naming,
                    ladder_lines.iter().map(String::as_str).collect::<Vec<_>>(),
                    "the excepted file's repertoire branch has changed. §21: name the exception, \
                     do not loosen the gate — a line that is not one of these three is a second \
                     branch and needs its own argument, not this one's"
                );
                continue;
            }
            if names_a_set || carries(&source, private_table) {
                offenders.push(path.to_string_lossy().into_owned());
            }
        }
        assert!(scanned > 0, "the walk found no source at all");
        assert_eq!(offenders, Vec::<String>::new());
        assert_eq!(
            excepted, 1,
            "the one named exception is not there. It is `crate::series::geom`, and if it has gone \
             this test should lose the exception rather than keep counting to one"
        );

        // **The other direction, through the same function**, because a scan that has quietly
        // stopped scanning also reports zero.
        let hostile = format!("    let arrow = {glyph_set}Ascii.arrow_right();");
        assert!(carries(&hostile, glyph_set));
        assert!(carries(&format!("{private_table} {{ }}"), private_table));
        assert!(!carries(&format!("// {glyph_set} in a comment"), glyph_set));

        // **No private fallback table anywhere in the workspace**, which is the half §16 states
        // over every crate and not only this one. It is checkable now for the same reason the
        // count above is: something in this workspace finally spells a glyph.
        let mut everywhere = Vec::new();
        rust_files(
            &PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")),
            &mut everywhere,
        );
        assert!(
            everywhere.len() > 100,
            "the workspace walk found only {} files",
            everywhere.len()
        );
        let mut private = Vec::new();
        for path in &everywhere {
            let source = std::fs::read_to_string(path).unwrap_or_default();
            if carries(&source, private_table) {
                private.push(path.to_string_lossy().into_owned());
            }
        }
        assert_eq!(private, Vec::<String>::new());

        // And the population the count is over: the module that does the glyph work, and the
        // twenty entries it does it for. A gate whose subject went away should go red again rather
        // than stay green over nothing, which is what this line is.
        assert_eq!(vitui_runtime::Glyph::ALL.len(), 20);
        assert!(
            crate::INVENTORY
                .iter()
                .filter(|c| !c.glyphs.is_empty())
                .count()
                >= 20,
            "the demand column is a stub again, and a count over a stub is a count over nothing"
        );
    }

    /// **This lineage carries at least one `*_numbers.rs` example.**
    ///
    /// §21's table records **0 against the runtime's 19** at the branch point. The convention is the
    /// runtime's: a file in `examples/` named `<subject>_numbers.rs` that prints the numbers a human
    /// reads and asserts the **shape** — counts, so that a report which has quietly started measuring
    /// something smaller fails instead of looking good.
    #[test]
    fn this_lineage_has_at_least_one_numbers_example() {
        let dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/examples"));
        let mut found: Vec<String> = std::fs::read_dir(&dir)
            .expect("the examples directory is beside this crate's manifest")
            .filter_map(|e| e.ok())
            .filter_map(|e| e.file_name().to_str().map(str::to_owned))
            .filter(|name| name.ends_with("_numbers.rs"))
            .collect();
        found.sort();
        assert_eq!(
            found,
            vec![
                "area_numbers.rs".to_string(),
                "collapsible_numbers.rs".to_string(),
                "collection_numbers.rs".to_string(),
                "dense_numbers.rs".to_string(),
                "field_numbers.rs".to_string(),
                "gates_numbers.rs".to_string(),
                "glyph_numbers.rs".to_string(),
                "grid_numbers.rs".to_string(),
                "keys_numbers.rs".to_string(),
                "listing_numbers.rs".to_string(),
                "nav_numbers.rs".to_string(),
                "order_numbers.rs".to_string(),
                "partition_numbers.rs".to_string(),
                "popup_numbers.rs".to_string(),
                "press_numbers.rs".to_string(),
                "primitive_numbers.rs".to_string(),
                "scene_numbers.rs".to_string(),
                "series_numbers.rs".to_string(),
                "table_numbers.rs".to_string(),
                "tree_numbers.rs".to_string(),
                "wheel_numbers.rs".to_string()
            ],
            "the count on this lineage was 0 against the runtime's 19"
        );
    }

    /// **The named barriers have lifted, and this is the deliberate edit they asked for.**
    ///
    /// This test used to assert the opposite, and its own failure message named the procedure:
    /// *"`{name}` has become reachable through the runtime. That inverts a row of `REGISTER` and is
    /// a deliberate edit here."* Runtime architecture issue 22 made all four reachable at once, so
    /// the edit is here and the assertion is inverted rather than deleted — a barrier that lifts and
    /// takes its own gate with it leaves nothing watching the direction it came from.
    ///
    /// **Why it is worth keeping in the new direction.** `Rgb`, `Mouse`, `Rect` and `Mods` are the
    /// four this crate was built around not having: `cells.rs` existed because of `Rect`,
    /// `crate::counters::sentinel`'s first barrier was `Rgb`, and `crate::listing` could not post a
    /// wheel click because of `Mouse`. If any of them goes back to `reachable_as: None` those three
    /// modules are wrong again, and this is the only place that would say so.
    ///
    /// **`Buttons` and `MouseKind` are asserted beside them and were never in the inventory at all**,
    /// which is what made the `Mouse` barrier survive a check that only ever looked at `Mouse`:
    /// a name a consumer can write but not **build** is a barrier wearing a re-export's clothes.
    ///
    /// Read from the shipped inventory rather than from a `use`, which is what this crate did while
    /// it could not name the engine and is still the arrangement that reads `ENGINE_NAMES` itself
    /// rather than a path that happens to compile.
    #[test]
    fn the_named_barriers_have_lifted() {
        let source = read("crates/vitui-runtime/src/line.rs");
        let lines: Vec<&str> = source.lines().map(str::trim).collect();
        for name in ["Rgb", "Mouse", "Rect", "Mods", "Buttons", "MouseKind"] {
            let needle = format!("name: \"{name}\",");
            let at = lines
                .iter()
                .position(|l| *l == needle)
                .unwrap_or_else(|| panic!("`{name}` is no longer an entry of `ENGINE_NAMES`"));
            assert!(
                lines[at + 1].starts_with("reachable_as: Some("),
                "`{name}` is `{}` and runtime architecture issue 22 says every engine name on the \
                 runtime's surface is reachable through it. The partition helpers, \
                 `crate::counters::sentinel` and `crate::listing` are all written against these \
                 four being reachable",
                lines[at + 1]
            );
        }
    }

    /// **The four are reachable by writing them**, which the test above cannot do.
    ///
    /// Reading `ENGINE_NAMES` proves the inventory says so; this proves the compiler agrees, and the
    /// two together are what `cells.rs`'s header meant by *there is no second spelling*. A
    /// `Mouse` is **built** here rather than named, because construction is the half the old barrier
    /// survived: `Driver::post_mouse` takes one, and until issue 22 no crate on this side of the
    /// line could reach `Buttons` or `MouseKind` to make one.
    #[test]
    fn the_four_are_reachable_by_writing_them() {
        use vitui_runtime::{Buttons, Driver, Mods, Mouse, MouseKind, Notch, Rect, Rgb};

        let area: Rect = Rect::new(0, 0, 10, 4);
        assert_eq!((area.w, area.h), (10, 4));

        let _: Rgb = Rgb { r: 1, g: 2, b: 3 };

        let wheel = Mouse {
            x: 3,
            y: 2,
            kind: MouseKind::Wheel(Notch::Down),
            buttons: Buttons::NONE,
            mods: Mods::NONE,
            at: std::time::Instant::now(),
        };
        assert_eq!(wheel.kind, MouseKind::Wheel(Notch::Down));

        let mut driver = Driver::headless(10, 4).expect("a sink cannot fail to attach");
        driver.post_mouse(wheel);
    }

    /// **The app thread's half cannot cross a thread and the worker's half can** — row 31's runnable
    /// side, beside the `compile_fail` pair on `WhyATaskCannotCrossAThread`.
    ///
    /// A `Send` bound that is *not* satisfied cannot be written as a passing call, which is why the
    /// negative half is a doctest and this half asserts only the positives.
    #[test]
    fn the_app_threads_half_cannot_cross_a_thread_and_the_workers_half_can() {
        use vitui_runtime::work::{Task, Worker};

        fn assert_send<T: Send>() {}
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send::<Worker>();
        assert_send_sync::<Worker>();
        assert_send::<std::cell::Cell<u32>>();
        assert_send::<std::cell::RefCell<u32>>();

        let worker = Worker::queueing();
        let task: Task<u32> = Task::new(&worker);
        assert_eq!(task.asking(), None);
    }

    /// The register prints as a table, which is how a human reads it.
    ///
    /// A test rather than a report, because the `Debug` derive is the only other way to look at this
    /// value and it is unreadable at forty-seven rows.
    #[test]
    fn the_register_prints() {
        assert_eq!(table().lines().count(), REGISTER.len() + 1);
    }
}
