//! Spec §21's register: **sixty-seven gates as a value, one row per gate, and a number for how many
//! of them anything runs.**
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
//!   the exact shape that returns green by accident*. Fifteen gates here could run and have **nothing
//!   to run over** — no component exists — and a register that filed those as `Evaluated` would be
//!   claiming fifteen green gates over an empty population.
//!
//! **Forty-two evaluated, four red, six unreachable, fifteen unsubjected**, and
//! `tests::eighty_one_rows_are_evaluated_and_the_rest_say_why_not` is what makes the next change a
//! deliberate edit rather than a quiet one. It was eighteen / four / six / sixteen until components
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

/// The scroll area's own file, which is where components ticket 18's six rows run.
const AREA: &str = "crates/vitui-components/src/area.rs";

/// The forest's own file, which is where components ticket 16's rows run.
const FOREST: &str = "crates/vitui-components/src/forest.rs";

/// The grid's own file, which is where components ticket 14's four rows run.
const GRID: &str = "crates/vitui-components/src/grid.rs";

/// The accordion's own file, which is where components ticket 21's rows run.
const ACCORDION: &str = "crates/vitui-components/src/accordion.rs";

/// The document's own file, which is where components ticket 23's rows run.
const DOCUMENT: &str = "crates/vitui-components/src/document.rs";
const CLUSTERS: &str = "crates/vitui-components/src/clusters.rs";

/// The popup's own file, which is where components ticket 25's rows run.
const POPUP: &str = "crates/vitui-components/src/popup.rs";

/// The collection's own file, which is where components ticket 12's rows run.
const COLLECT: &str = "crates/vitui-components/src/collect.rs";

/// The order's own file, which is where components ticket 13's rows run.
const ORDER: &str = "crates/vitui-components/src/order.rs";
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
pub const EVALUATED: usize = 81;

/// Spec §21's register, row for row, and this ticket's gates beside it.
pub const REGISTER: [Row; 102] = [
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
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: "crates/vitui-components/src/counters.rs",
                name: "verbs_never_exceed_writes_and_the_relation_is_not_an_equality",
            }],
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
        standing: Standing::Unsubjected {
            inverted_by: "components 22",
        },
    },
    Row {
        number: 25,
        on_spec_table: true,
        gate: "`open` is never ambiguous mid-transition",
        kind: Kind::Invariant,
        owner: "C14",
        section: "spec §8",
        standing: Standing::Unsubjected {
            inverted_by: "components 22",
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
                    name: "the_axis_is_named_in_two_files_and_neither_is_a_component",
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
        // **The barrier has lifted, and the substitution it forced is now the thing to remove.**
        // Components ticket 11 made the other half run while the wire stayed out of reach: the row
        // read *half cannot: a wheel click is a posted `Mouse`*, so the click's **delta** was handed
        // straight to the arithmetic `Response::scrolled` would have delivered it to, on both arms.
        // That substitution was honest and is now unnecessary. Runtime architecture issue 22
        // re-exported `Mouse` **and the three types needed to build one** — `Buttons`, `MouseKind`
        // and the notch — which is precisely what this comment recorded as missing: *"a `Mouse`
        // needs a `Buttons` and a `MouseKind`, neither of which is in `ENGINE_NAMES` at all."*
        // `crate::gates::tests::the_four_are_reachable_by_writing_them` posts one.
        //
        // **The row stays red and the figure is unchanged**, because the defect was never the wire:
        // it is the unconditional scroll-into-view in four resolved tickets' code. What components
        // 20 inherits is a gate that can now drive the real channel instead of its arithmetic.
        standing: Standing::Red {
            by: &[
                Instrument::Unit {
                    file: LISTING,
                    name: "twenty_wheel_clicks_move_the_offset_twenty_and_an_unconditional_reveal_\
                           takes_it_back",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/tests/gates.rs",
                    name: "a_frame_that_asks_for_no_reveal_moves_no_offset_and_one_that_asks_does",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/listing_numbers.rs",
                },
            ],
            failing: "0 against 16 over twenty clicks, in four *resolved* tickets' code, each \
                      written by someone who had read the rule in `CONTEXT.md` forbidding it. **0 \
                      against 20 here**, and both directions are pinned: `Reveal::EveryFrame` \
                      settles the offset at 0 where the conditional arm settles at 20, and \
                      `Reveal::Never` — deleting the call, which passes that half — moves the \
                      offset 0 when a keyboard reveal really asks. The click is postable as of \
                      runtime issue 22 and this gate does not yet post one: the delta is still \
                      handed to `Response::scrolled`'s own arithmetic on both arms, which components \
                      20 replaces with the wire it can now reach",
            inverted_by: "components 20",
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
                    name: "sixteen_of_the_thirty_four_axis_obligations_have_a_scene_and_\
                           eighteen_do_not",
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
                    name: "ten_scenes_have_nothing_to_run_over_fifteen_are_red_and_seven_are_\
                           stood_up",
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
        standing: Standing::Red {
            by: &[
                Instrument::Unit {
                    file: FOREST,
                    name: "the_forest_is_red_because_tree_is_not_declared",
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
                    name: "ten_scenes_have_nothing_to_run_over_fifteen_are_red_and_seven_are_\
                           stood_up",
                },
            ],
            failing: "`tree` is undeclared, so scenes 8 and 9 are pinned red — both of them, and \
                      both waiting for the subject rather than for a fix. \
                      `crates/vitui-components/src/collect.rs` carries no `pub fn tree(`, which is \
                      what `crate::forest::subjects_declared` opens the file to find out, and \
                      `crate::forest::standing` is `Unmet { over: 1, failing: 1 }` rather than \
                      `Met` over nothing",
            inverted_by: "components 17",
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
        standing: Standing::Red {
            by: &[
                Instrument::Unit {
                    file: GRID,
                    name: "the_grid_is_red_because_table_is_not_declared",
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
            failing: "`table` is undeclared, so scenes 7 and 30 are pinned red and both are \
                      waiting for it. `crates/vitui-components/src/collect.rs` carries no \
                      `pub fn table(`, which is what `crate::grid::subjects_declared` opens the \
                      file to find out, and `crate::grid::standing` is `Unmet { over: 1, failing: \
                      1 }` rather than `Met` over nothing",
            inverted_by: "components 15",
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
                    name: "ten_scenes_have_nothing_to_run_over_fifteen_are_red_and_seven_are_\
                           stood_up",
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
                    name: "ten_scenes_have_nothing_to_run_over_fifteen_are_red_and_seven_are_\
                           stood_up",
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
    fn eighty_one_rows_are_evaluated_and_the_rest_say_why_not() {
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
            vec![7, 8, 29, 74, 78, 85, 89],
            "the four gates that are red and pinned: the sentinel, the palette after a swap, \
             twenty wheel clicks and the collection's five scenes waiting for their subject. The \
             glyph-set count was one of them and components ticket 05 inverted it; row 61 was \
             another and components ticket 10 inverted it, which took rewriting the gate rather \
             than the standing — the row asserted an *absence*"
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
            unsubjected, 8,
            "and the fourteen with nothing to run over. **It was fifteen until components ticket \
             18**, which supplied the subject for row 20 — the bar fixpoint, which is arithmetic \
             over a domain and needs no component to be run against, and whose row had been \
             citing spec §13 and components 28 for two tickets"
        );
        assert_eq!(evaluated + red.len() + unreachable.len() + unsubjected, 102);
    }

    /// **The split, not the total.**
    ///
    /// §21's table is thirty-two rows and it is closed; everything after it is a gate a ticket on
    /// this lineage wrote. Asserting the split is what made the forty-fifth row — components ticket
    /// 05's matrix barrier — say which side of the line it is on, and a thirty-third row claiming to
    /// be §21's is a spec change, which should not be able to arrive as a one-line diff in this
    /// file.
    #[test]
    fn thirty_two_rows_are_the_specs_and_seventy_are_this_lineages() {
        let on_table = REGISTER.iter().filter(|r| r.on_spec_table).count();
        assert_eq!(on_table, SPEC_ROWS);
        assert_eq!(REGISTER.len() - on_table, 70);
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
    #[test]
    fn no_component_here_names_a_glyph_set_and_none_has_a_private_missing_table() {
        let glyph_set = concat!("GlyphSet", "::");
        let private_table = concat!("mod ", "missing");

        let mut files = Vec::new();
        rust_files(
            &PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")),
            &mut files,
        );
        let mut scanned = 0usize;
        let mut offenders = Vec::new();
        for path in &files {
            let source = std::fs::read_to_string(path).unwrap_or_default();
            scanned += 1;
            if carries(&source, glyph_set) || carries(&source, private_table) {
                offenders.push(path.to_string_lossy().into_owned());
            }
        }
        assert!(scanned > 0, "the walk found no source at all");
        assert_eq!(offenders, Vec::<String>::new());

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
                "tree_numbers.rs".to_string()
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
