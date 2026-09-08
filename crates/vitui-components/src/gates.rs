//! The register: **a hundred and thirty-five gates as a value, one row per gate, and a number
//! for how many of them anything runs.**
//!
//! > The register is data, not prose — one row per gate with its kind, its owner, where it stood at
//! > the branch point and where it stands now, so the delta is a number a test asserts. The reason
//! > is this: every obligation stated as a sentence here has been broken by someone who had
//! > read it.
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
//! - [`Standing::Unreachable`] is a **result**. Five gates cannot be written from this crate at all,
//!   and each says exactly what would have to become public. That is not *not yet*: `vitui-components`
//!   depends on `vitui-runtime` and nothing else (C6), so a gate needing an engine name is a
//!   compile error and no amount of component code changes it.
//! - [`Standing::Unsubjected`] is the vacuity arm, and it exists because [`crate::obligations`]
//!   already proved it necessary one file over: *a query over an obligation nobody has met yet is
//!   the exact shape that returns green by accident*. **No gate here stands on it any more** — it
//!   was fifteen when no component existed, then six, then four —
//!   and the arm stays on the type, because what it refuses is a standing and not a
//!   moment: the next row written against a component this crate has not declared belongs here. The
//!   count in the line below is the authority; this is a summary of it.
//!
//! **Two hundred and twenty-eight evaluated, none red, five unreachable, none unsubjected**, and
//! `tests::two_hundred_and_twenty_eight_rows_are_evaluated_and_the_rest_say_why_not` is what makes
//! the next change a deliberate edit rather than a quiet one. It was eighteen / four / six / sixteen until components
//! the glyph-set count was inverted — red because it had nothing to be about —
//! and the cross-family collapse gate was subjected; five more arrived, and none of them
//! moved a standing that was already taken. **Four more arrived and one inverted**, and the
//! inversion is the one to read.
//!
//! **Five more arrived and one of them is red on purpose.** Four of them are the dense screen —
//! 338 regions, the metric row, the equality against a naive twin at two sizes, and the five
//! re-damage instances each standing on a screen instead of in a sentence — and they are
//! `Evaluated` over a screen rather than over a component, which is the same standing the
//! four rows have and for the same reason: what they gate is that *the instrument separates a
//! correct build from a defective one*. **Row 61 is the fourth red row**, and it is the only one on
//! this register that is red because a *scene* has no subject rather than because a gate fails. The
//! distinction it encodes is criterion 7's: a scene that fails because it is unimplemented and one
//! that fails because the code is wrong are the same failure unless the message separates them, so
//! the failing set is computed by opening the four files components 10 will declare in and the
//! panic names all four and the ticket.
//!
//! **Two more arrived and a third was rewritten, and the three are three different kinds of
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
//! the click cannot be posted*, which is the distinction one row got wrong. **The
//! re-export has since lifted that barrier too**; the substitution stays for now, and the
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
//! **That last observation is what the re-export acted on**, and it is why the rule
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
//! checked — so a row whose test has been renamed still reads as wired. That was fixed
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
//! # Two of the rows are corrected here, from the shipped code rather than from the prototypes
//!
//! The rules were written against twelve prototypes, and two of their sentences do not survive contact with
//! the crates that shipped. Both corrections are recorded on the row and neither changes its
//! standing:
//!
//! - **The tab-stop row.** The claim is that its geometric form is impossible because *the ring
//!   does not carry geometry* — `RingEnt::rect` is zero unless `Frame::ring_geometry` is on. The
//!   shipped type is `vitui_runtime::focus::Stop` and its `rect` is filled on **every** entry,
//!   unconditionally. The conclusion holds for a better reason: the rectangle recorded is the one the
//!   widget *declared*, intersected with nothing at all.
//! - **The auto-trait row.** The claim is that `Task` and `Worker` are `!Sync`. `Worker` is `Sync` — it is an
//!   `Arc<Inbox>` — and the half that is load-bearing is `Task`, which is `!Send` and `!Sync` on a
//!   `PhantomData<*const ()>` brand.

/// What shape of gate a row is.
///
/// R15's list, unchanged: **a gate is a count, a ratio, an equality or a compile outcome; a timing is
/// a report.** [`Kind::Relation`] and [`Kind::Invariant`] are the two the table adds, and they
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
    /// the example, and it says *never verb equality across sizes* in as many words.
    Relation,
    /// Something that must not compile, with a positive twin naming the protected item by path.
    CompileOutcome,
    /// A property that must hold at every point of a transition, not only at its ends.
    Invariant,
}

impl Kind {
    /// The word the table prints.
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
    /// Whether it is a row of the thirty-two-row table, or one this ticket wrote beside it.
    ///
    /// The count test asserts **the split** rather than the total, so a forty-fifth row has to say
    /// which side of the line it is on — the engine's and the runtime's arrangement, for its reason.
    pub on_spec_table: bool,
    /// The gate, in the words the rules have for it.
    pub gate: &'static str,
    /// Count, ratio, equality, relation, compile outcome or invariant.
    pub kind: Kind,
    /// Whose gate it is, in the `C NN` numbering — the **architecture** map's tickets, which is
    /// what that table's owner column names. Implementation tickets are spelled `components NN` and
    /// appear only in the `inverted_by` fields.
    pub owner: &'static str,
    /// The section that states it.
    pub section: &'static str,
    /// Where it stands, and on what.
    pub standing: Standing,
}

/// The dense screen's own file, which is where the four rows run.
const DENSE: &str = "crates/vitui-components/src/dense.rs";

/// The listing's own file, which is where the two rows run.
const LISTING: &str = "crates/vitui-components/src/listing.rs";

/// The wheel gate's file, and the one row on this register whose instrument
/// posts a pointer event.
const WHEEL: &str = "crates/vitui-components/src/wheel.rs";

/// The scroll area's own file, which is where the six rows run.
const AREA: &str = "crates/vitui-components/src/area.rs";

/// The forest's own file, which is where the rows run.
const FOREST: &str = "crates/vitui-components/src/forest.rs";

/// The grid's own file, which is where the four rows run.
const GRID: &str = "crates/vitui-components/src/grid.rs";

/// The accordion's own file, which is where the rows run.
const ACCORDION: &str = "crates/vitui-components/src/accordion.rs";
/// **`collapsible`'s own file**, which is where the rows run. The component and
/// its two refused spellings are one file, so a reviewer's diff between them is a field.
const DISCLOSE: &str = "crates/vitui-components/src/disclose.rs";

/// The document's own file, which is where the rows run.
const DOCUMENT: &str = "crates/vitui-components/src/document.rs";
const CLUSTERS: &str = "crates/vitui-components/src/clusters.rs";

/// The popup's own file, which is where the rows run.
const POPUP: &str = "crates/vitui-components/src/popup.rs";

/// The collection's own file, which is where the rows run.
const COLLECT: &str = "crates/vitui-components/src/collect.rs";

/// The three scrolling components' own file. The seven rows are measured here.
const SCROLL: &str = "crates/vitui-components/src/scroll.rs";

/// The order's own file, which is where the rows run.
const ORDER: &str = "crates/vitui-components/src/order.rs";

/// The series screen's own file, which is where the rows run.
const SERIES: &str = "crates/vitui-components/src/series.rs";

/// The chart's own file, which is where the rows run.
const CHART: &str = "crates/vitui-components/src/chart.rs";
const FIELD_NUMBERS: &str = "crates/vitui-components/examples/field_numbers.rs";

/// **The text machine's own file**, which is where the rows run. The buffer, the
/// caret pair, the anchored selection, the undo ring and the wrap index are one module, and the
/// refused spellings are one `defective` module inside it — so a reviewer's diff between the
/// shipped build and any of them is one value.
const EDIT: &str = "crates/vitui-components/src/edit.rs";

/// **The overlay family's own file**, which is where the rows run. The three
/// kinds, the two axes, the sizing function, the gutter and the two halves of a modal are one module,
/// and the refused spellings are one `defective` module inside it.
const OVERLAY: &str = "crates/vitui-components/src/overlay.rs";

/// The overlay family's report.
const POPUP_NUMBERS: &str = "crates/vitui-components/examples/popup_numbers.rs";

/// **The picture screen's own file**, which is where the rows run. The screen,
/// the two false ladders, the two traps and the wire probe are one module, so a reviewer's diff
/// between the correct build and any of them is a field on `Build`.
const PICTURE: &str = "crates/vitui-components/src/picture.rs";

/// The picture screen's report.
const MEDIA_NUMBERS: &str = "crates/vitui-components/examples/media_numbers.rs";

/// **The media family's own file**, which is where the components live.
const MEDIA: &str = "crates/vitui-components/src/media.rs";

/// **The preview pane's three screens**, which is where the rows run. The pane,
/// its two keys, its five offset spellings and the two places a landing can be taken are one
/// module, so a reviewer's diff between the arm that ships and any of the others is a field on
/// `Build`.
const PREVIEW: &str = "crates/vitui-components/src/preview.rs";

/// The module the two F12 components are homed in.
const FILES: &str = "crates/vitui-components/src/files.rs";

/// The preview pane's report.
const PREVIEW_NUMBERS: &str = "crates/vitui-components/examples/preview_numbers.rs";

/// **The player chrome's file.** Six of the ten parts, the grab, and the shape that is kept
/// because it was wrong.
const PLAYER: &str = "crates/vitui-components/src/media/player.rs";

/// **The F5 module's file**, which is where the two indicator rows run. `meter`
/// and `sparkline` are one module because the family homes them there, and their two claims are two calls
/// into `chart`.
const INDICATE: &str = "crates/vitui-components/src/indicate.rs";

/// **The F2 module's file**, where `panel`, `rule` and `status_bar` live.
const STRUCTURE: &str = "crates/vitui-components/src/structure.rs";

/// **The report**, which prints the three things its rows compress into a
/// sentence: what a shared offset does to a band whose content has no rows, the two densities' form,
/// and the pager's window.
const COMPOSITE_NUMBERS: &str = "crates/vitui-components/examples/composite_numbers.rs";

/// **O4's own file.** The thirteen contracts, the sweep that runs the component, and the control
/// arm.
const CONTRACT: &str = "crates/vitui-components/src/contract.rs";

/// **The report**, which prints the chord that differs where the equality prints
/// a count.
const CONTRACT_NUMBERS: &str = "crates/vitui-components/examples/contract_numbers.rs";

/// **The report**, which prints the two things its rows compress into a sentence:
/// the two eighth-block runs side by side, and the sparkline's three verb counts over one write
/// count.
const TIER_TWO_NUMBERS: &str = "crates/vitui-components/examples/tier_two_numbers.rs";

/// **O1's own file.** The page per built component, the axis join and the six refusals, as values a
/// scan runs over rather than as a review habit.
const DOC: &str = "crates/vitui-components/src/doc.rs";

/// **The report**, which prints the page table O1's count compresses into a
/// number, the axis join with the freeze's answer beside the page's, and the fence census.
const DOC_NUMBERS: &str = "crates/vitui-components/examples/doc_numbers.rs";

/// **The Tier 2 composition claim's own file.** Six rows, each naming what its component reaches and
/// what it may not mint, and the scan that opens the file and answers both halves.
const COMPOSED: &str = "crates/vitui-components/src/composed.rs";

/// The golden screens and the format they are written in.
const GOLDEN: &str = "crates/vitui-components/src/golden.rs";

/// The three-rung sweep, which is the one file outside `src/` that may name a repertoire — and the
/// fourth entry of `tests/glyph_matrix.rs`'s own exception list.
const GOLDEN_TESTS: &str = "crates/vitui-components/tests/golden.rs";

/// The report.
const GOLDEN_NUMBERS: &str = "crates/vitui-components/examples/golden_numbers.rs";

/// **`field`'s own file**, which is where the component's rows run. `input.rs` and not `edit.rs`,
/// because that is where the freeze homes `field`: F6, and [`crate::document::DECLARATIONS`] opens
/// it to find out whether the three scenes have a subject.
const INPUT: &str = "crates/vitui-components/src/input.rs";

/// **The assembled gallery**, which is O2's evidence and the screen rows 7 and 8 are measured on.
/// The library crate and not `crates/vitui-apps/examples/gallery.rs`, for the reason its own header
/// gives: a screen only an application can reach is a screen no `cargo test` can measure.
const GALLERY: &str = "crates/vitui-components/src/gallery.rs";

/// **The memo census** — the rule with a population it can be false on, and the finding that
/// the population is one row and it had to be built.
const MEMOS: &str = "crates/vitui-components/src/memos.rs";

/// See [`GALLERY`].
const GALLERY_NUMBERS: &str = "crates/vitui-components/examples/gallery_numbers.rs";

/// **The pty gate**, which is the one instrument on this register that is not a `cargo` invocation:
/// a test has no terminal to leave in a state.
const PANIC_GATE: &str = "scripts/gallery-panic-gate.sh";

/// Where the standing budget counts run from a crate that cannot name the engine.
const BUDGET_TESTS: &str = "crates/vitui-components/tests/budget.rs";

/// Where the five obligations are queries over the freeze.
const OBLIGATIONS: &str = "crates/vitui-components/src/obligations.rs";

/// **O6's own file** — the seventh query, the seven covered rows, the two bounds and the seven
/// deliberate defects.
const VOLUME: &str = "crates/vitui-components/src/volume.rs";

/// The report, which is where the clock and its 16.7 ms denominator live.
const VOLUME_NUMBERS: &str = "crates/vitui-components/examples/volume_numbers.rs";

/// **O7's own file** — the join between the freeze and `crates/vitui-apps/examples/`, by import
/// path.
const CONSUMER: &str = "crates/vitui-components/src/consumer.rs";

/// **The applications' own list**, which is where the `uses` column and the scan are compared. It
/// is another crate's file, and that is the point: O7 is the one obligation whose evidence is not in
/// this crate at all.
const APPS: &str = "crates/vitui-apps/src/lib.rs";

/// How many rows of [`REGISTER`] are the table. **Thirty-two, and it is closed** — a
/// thirty-third would be a spec change.
pub const SPEC_ROWS: usize = 32;

/// How many rows anything evaluates today. **Forty-eight.**
///
/// The number is the point of the file. The count was **2 of 18** at the branch point and **11 of 18**
/// after C11's own pass, both over the prototypes; this is the first count taken over shipped code,
/// and it is forty-eight of seventy-two because twenty-four of the rows are about components that
/// do not exist or need a name the crate line refuses.
///
/// **It was fourteen of forty once**, before the reference-render runner and its
/// four rows. Every one of the four is `Evaluated` over a **fixture** rather than over a component,
/// which is a real standing and not a promoted one: what those rows gate is that *the instrument
/// separates a correct build from a defective one*, and each is watched doing it in both directions.
/// The scenes themselves stay `Unsubjected` in [`crate::scenes`], and that file says why the two are
/// not the same claim.
///
/// **The five are the same kind of standing**, over the chip and the two bars its own
/// helpers stand up rather than over a component: rows 48–52. Row 48 is the one worth reading
/// twice — it is the **reachable form of row 1**, which is `Unreachable` and stays that way, and
/// the two rows now sit side by side saying which question each of them can answer.
///
/// **The four are rows 53–56, and its fifth is row 5** — which moved from `Unreachable` to
/// `Evaluated` without anything in the runtime changing, because the barrier had been misread. See
/// this module's header: that is the only inversion on this register that corrected a *standing*
/// rather than supplying a *subject*, and it is the one worth being suspicious about the next time
/// an `Unreachable` is written.
///
/// **It went from forty-one to forty-two, and the one row is** the
/// regions equality, `Evaluated` over the listing's two arms rather than over `collection`, which is
/// the standing and its reason: what it gates is that *the instrument separates a
/// correct build from a defective one*. The **scene** stays red, and `crate::scenes` says at length
/// why those are not one claim. Its second row, 67, is `Red` on purpose.
///
/// **It went from thirty-six to forty-one, and one of the five is not a new
/// row.** Rows 62–65 are the four primitives' — the partition sweep, the fill scan, the clear and
/// the hovered chip — and the fifth is **row 61**, the one row on this register whose gate was a
/// statement about an absence. Inverting it rewrote the *gate* and not only the standing, because a
/// row still asserting *the four components do not exist* would now be asserting they are gone.
/// That is the second inversion here worth being suspicious about: a red row phrased as an absence
/// cannot be turned green by editing one field.
/// **It went from a hundred and eight to a hundred and ten**, and one of the
/// two is a new row: row 29 was **pinned red for nine tickets** and went green over the shipped
/// path, and row 129 is criterion 6's join between the gate's subject list and the freeze. A red row
/// inverted by supplying a *subject* is the ordinary case on this register; this one was red on a
/// **defect**, which is why it needed the shipped code to be checked rather than written.
///
/// **It went from a hundred and eighteen to a hundred and twenty-seven**, and
/// the seven are one inversion and six new rows. The inversion is row 85 — `field`'s three scenes
/// had nothing to run over, and all three turned together, which is what the ticket predicted
/// because they were pinned on one fact and it was the subject.
///
/// **Two of the six are absences.** Row 137 is the deletion — *there is no such thing as "put
/// the caret at byte N"* — as three `compile_fail` pairs, and it is two deletions rather than one
/// because `Caret`'s public fields would be `set_caret` three keystrokes shorter. Row 139 is the
/// per-cluster region count, which is the defect from the other side: there, widgets merged
/// into one id and the screen still rendered correctly; here, one widget declares hundreds of
/// entries and the screen still renders correctly.
///
/// **Two of the nine are a review's and not the ticket's** — rows 142 and 143, and both are gaps
/// the ticket's own gates did not cover. 142 is that the rows do not *tile* the buffer, so a caret
/// placed by asking which drawn row contains its byte is lost by one trailing space; 143 is a
/// declared `Interest::SCROLL` the component never consumed, which is the defect
/// class arriving on a component that gate has no subject for.
///
/// Row 21 stays `Unreachable` and row 130 is its crate-own form, which is row 41's standing to row
/// 2's.
///
/// **It went from a hundred and ninety-three to a hundred and ninety-six**,
/// and none of the three is an inversion: rows 210, 211 and 212 are O1's other three halves, which
/// row 30 could not carry because it is `Kind::Count` and O1 is a count *and* a compile outcome —
/// which is what the `mixed` in the kind column was hiding.
///
/// **It went from two hundred and eight to two hundred and nine, and the one
/// is an inversion**: row 7, the sentinel — *every cell of the rectangle written at least once* —
/// which had been pinned red from the start with its exact failing set. It is the **second** red
/// row on this register whose failing set was a *defect* rather than a missing subject (row 29 was
/// the first), and the only one whose prescribed instrument could not be built at all: it asks for a
/// stamp on the base layer and a count of survivors on the composited surface, and no cell reachable outside the engine forbids
/// the readback as a decision. The count is read off `crate::runner::Pen` instead — which is the
/// instrument the rule's *other* half has always been read off, and the stricter of the two, since a
/// verb that skips the caller's `Ink` makes it larger rather than smaller.
///
/// **The identity-verb fix moved it from two hundred and twenty to two hundred and twenty-one,
/// and it is the first of two inversions this register has recorded that no components ticket
/// did.** Row 112 was red on a defect one crate down: `Ctx::with_id` re-childed its view at
/// `self.area()`, which inside a scroll scope is the *content's* rectangle, so a keyed child past
/// the first screenful drew nothing. `with_id` and `Ctx::scope` reborrow now. **The register is
/// green: no row is pinned red**, and the two populations that read the standing — this count and
/// the list beside it — are the two edits.
///
/// **A later pass moved it from two hundred and twenty-four to two hundred and twenty-eight,
/// and it emptied the `Unsubjected` column.** Rows 15 to 18 are the original's, `field`'s four, and all
/// four named components 24 — which built the component and then filed six *new* rows for it instead
/// of standing these up. Three of the four were stale the way rows 11 and 12 were: the instruments
/// ran over the shipped `field` and the standing said nothing ran. **Two of the four were missing a
/// half**, and both halves are the same mistake in two places — *the instrument inspects a state
/// nothing has moved*:
///
/// - rows 15 and 16 were watched over a `step_right` walk of the corpus and over the gestures, both
///   of which read a caret over a buffer that does not move. The two caret defects arrived on an
///   **edit**. `document::edit_walk` plays a script at each of 144 seats — both ends, the seat, an
///   insert, a backspace, a delete and three undos — and counts the three producers apart, because
///   gate 1 is *structurally* unable to fail on the one that walks and has teeth on the one that
///   **restores**. A `Text::edit` that seats a byte-addressed caret puts 715 of 1 296 columns wrong
///   and no caret off a boundary at all.
/// - row 18 read an inequality between two indexes **neither of which was drawn**, and its own gate
///   is about *the width being drawn*. It now reads `built_at()` off the state after
///   `crate::input::field_into` has drawn it.
///
/// Row 17 was the one already carrying its population — 500 deterministic edits, because the original's
/// defect agreed with a rebuild 499 times in 500 — and what it gained is that the population runs
/// through `Text::insert` as well as through `Index::spliced`, and that the two report the same three
/// numbers. **Four unsubjected became none**, and every row of this register is now `Evaluated` or
/// `Unreachable` and nothing else.
///
/// **The four `gate` strings are the words and were left alone**, which is a rule this ticket
/// nearly broke: extending row 15's to *at both ends and across an edit* and row 17's to *over five
/// hundred edits* reads as clarification and is a **spec edit**, arriving as a one-line diff in this
/// file, exactly what `tests::thirty_two_rows_are_the_specs_and_two_hundred_and_two_are_this_\
/// lineages` exists to make deliberate. What a ticket adds to a spec row goes in its comment.
///
/// **Another moved it from two hundred and twenty-two to two hundred and twenty-four,
/// and neither of the two is an inversion — both are standings that had gone stale.** Rows 11 and 12
/// are the original's, `table`'s two, and both named components 15 as the ticket that would subject them;
/// components 15 declared the component and did not edit the rows, and nothing left on any backlog
/// was going to. The instruments were already there and already green — `crate::grid` has drawn
/// through `crate::collect::table` since components 15 — so what this ticket added is the half that
/// made each of them able to fail: row 11's sweep now asserts that its four declared column counts
/// are **four different tables** before asserting they draw one screen, and carries the clip-only
/// sweep beside it as the control that steps at every arm. **Six unsubjected became four**, and all
/// four of those are on the table too — they are the field's, and a later pass takes them.
///
/// **The indent-guide decision moved it from two hundred and twenty-eight to two hundred and
/// twenty-nine**, and it is not an inversion: row 234 is a property the freeze had been asserting
/// with a join that could not fail on it. Its `glyphs` column was joined to another declaration, so
/// a row demanding an entry it draws nothing of satisfied *every entry has a demander* — four such
/// entries stood for four tickets and five more are still standing, held in `crate::glyphs::UNDRAWN`
/// with the issue that owns them.
///
/// **The two-colour question moved it from two hundred and twenty-one to two hundred and
/// twenty-two, and it is the second of those** — one crate down again, and this time it lifted a
/// barrier rather than fixing a defect. Row 161 wanted bytes on the wire and needed a driver that
/// hands its sink back: `Clock`, `Output`, `Overrides`, `WidthSource` and `InputConfig` were not in
/// `vitui_runtime::line::ENGINE_NAMES` at all, reachable or not, so no `Config` this crate could
/// build sent its bytes anywhere it could read. All five are re-exported now. **Six unreachable
/// became five**, and that is the whole of the change to the split.
///
/// **The spinner moved it from two hundred and eighteen to two hundred and twenty**, and
/// neither of the two is an inversion: rows 232 and 233 are the twenty-ninth component's — *stored
/// state may be an anchor, never a phase*, and the playhead's cadence beside it. It is the last row
/// of the freeze to be built, so **every row of that table is `built` from here on** and the four
/// populations that read the column — O1's, O2's, O3's and O7's — moved to twenty-nine with no edit
/// to any of them.
///
/// **The consumer join moved it from two hundred and sixteen to two hundred and eighteen**, and
/// neither of the two is an inversion: rows 230 and 231 are O7's — *every component this crate
/// declares is exercised by an application*, and the `uses` column agreeing with the scan that
/// answers it. It is the seventh obligation and the one whose **evidence is in another crate**,
/// which is why the second row's instruments live in `crates/vitui-apps/src/lib.rs`.
///
/// **The keyboard contracts moved it from a hundred and ninety-nine to two hundred and two**, and
/// none of the three is an inversion either: rows 216, 217 and 218 are O4's, at the level O4 means
/// it. Row 30's own instrument compares two lists of *ids*, which is the most a query over the
/// freeze can ask; the chord-for-chord equality needs a value with a machine in it, and
/// `crate::contract::Contract::live` is that machine — it runs the shipped component.
pub const EVALUATED: usize = 229;

/// The register, row for row, and this ticket's gates beside it.
#[expect(
    clippy::large_const_arrays,
    reason = "a `static` is what the lint asks for and it would cost every reader of this table a \
              `.iter().copied()`: `Row` is `Copy` and forty-odd sites iterate this by value. The \
              array is read at compile time by nothing and at run time by tests, so the copy the \
              lint is warning about is one a test makes once"
)]
pub const REGISTER: [Row; 234] = [
    // ── the table, in its order ───────────────────────────────────────────────────────────
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
                // **Components ticket 35 gave the row a component to stand on.** Every focusable of
                // a real `form` is focused in turn and `Ctrl+S` pressed into it: nothing lands in
                // any of the six buffers and all six keys reach the application, with a plain `s`
                // beside it typing, so the gate is not measuring a form that has stopped accepting
                // anything.
                Instrument::Unit {
                    file: INPUT,
                    name: "a_chord_pressed_into_every_focusable_in_a_form_types_nothing",
                },
                Instrument::Report {
                    file: "crates/vitui-components/examples/keys_numbers.rs",
                },
                // **Components ticket 38 found four chord leaks this row cannot see, and that is a
                // fact about what it measures rather than a defect in it.** *Types nothing* is a
                // claim about a **buffer**: `Ctrl+Left` moved a caret, `Ctrl+Down` opened a
                // `select`'s list and `Ctrl+Esc` cleared a collection's selection, and every one of
                // those types nothing. What they did was **take the key**, so the application's
                // accelerator never arrived — which is row 216's question and not this one.
                Instrument::Unit {
                    file: CONTRACT,
                    name: "the_four_chord_leaks_are_shut",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "an_open_popup_does_not_eat_a_chord_built_on_its_own_two_keys",
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
                // Watched firing, in both directions, on the 15-cell instance.
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
        // **Inverted by components 40, and the detector is not the one spec §2 prescribes.**
        //
        // The failing set this row was pinned with, kept because it is the record of what it caught:
        // *9 956 cells of 53 280 (18.7%) over six panels of twelve — the chip screen 4 189, the
        // preview pane 2 159, the scroll area 1 799, media 1 286, the chart 496, the collection pair
        // 27* on the prototype's gallery, and **10 252 of 24 000 at 300x80 and 525 of 3 000 at
        // 100x30** on the shipped one, which is the figure components 39 left this ticket to start
        // from. Both are gone; the shipped screen's are 0 at every size and on every page.
        //
        // **The sentinel is a reading of a recorded surface and not a probe of the screen.** ADR
        // 0023 forbids the readback and that is a decision, not a gap — and it does not need
        // lifting: the rule's *first* half has always been read off `Tally`, whose union has been in
        // root coordinates since components 19, so the second half is that same union against the
        // area. Read on the screen, the pair would be an equality between two instruments. And the
        // recorder is the **stricter** one, because a verb that skips the caller's `Ink` makes the
        // count larger. So the `Barrier` citation this row carried is struck rather than left
        // pointing at a line that still exists — runtime 22's own near-miss — and
        // `crate::counters::sentinel` answers.
        //
        // **Three mechanisms and a fourth that is nobody's component**, which is the sentence
        // arriving as four numbers: `panel` hands back `Frame::interior` (294), `collapsible` hands
        // back `Disclosure::used` (432), a `scrollbar` is three columns of a wider tile because this
        // caller narrowed it (630), and two slots of a six-by-five grid hold no panel at all
        // (1 600). Beside them, the one component-side finding: `select` and `file_picker` wrote
        // **one row of any rectangle** and their `Response` has no field a remainder could be named
        // in, so they write it.
        standing: Standing::Evaluated {
            by: &[
                // The assembled form, swept over sizes and pages, with the spelling it replaced
                // watched leaving its exact set behind.
                Instrument::Unit {
                    file: GALLERY,
                    name: "no_cell_of_the_assembled_gallery_is_written_by_nobody",
                },
                Instrument::Unit {
                    file: GALLERY,
                    name: "the_remainder_left_alone_is_three_drawings_and_the_grids_own_slack",
                },
                // The per-component form, which §21 records as *report-only across nine to twelve
                // binaries* — over the shipped call site at five rectangles, and over the
                // thirty-three constructions at all three rungs.
                Instrument::Unit {
                    file: GALLERY,
                    name: "every_panel_writes_every_cell_of_the_interior_it_was_handed",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/tests/golden.rs",
                    name: "every_construction_writes_every_cell_of_its_rectangle",
                },
                // The two rows of the freeze that did not meet it, each swept over both axes now.
                Instrument::Unit {
                    file: "crates/vitui-components/src/overlay.rs",
                    name: "a_shut_selects_face_is_a_partition_of_its_rectangle_at_every_width",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/preview.rs",
                    name: "a_shut_pickers_face_is_a_partition_of_its_row_at_every_width",
                },
                // The counter itself, answering, in both directions on one surface.
                Instrument::Unit {
                    file: "crates/vitui-components/src/counters.rs",
                    name: "the_sentinel_counts_the_cells_no_verb_wrote",
                },
                Instrument::Report {
                    file: GALLERY_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 8,
        on_spec_table: true,
        gate: "no cell keeps the previous palette a frame after a swap",
        kind: Kind::Count,
        owner: "C11 / R20 §3",
        section: "spec §16",
        // **The gate is a count over the surface against an oracle**, and neither the delta nor its
        // complement could have been it. The spelling that shipped asserted `changed > 0` and one
        // cell of 4 800 satisfies it while 3 583 carry the old palette (refinement 1); the
        // complement reads `kept` **17 884 of 24 000** under a rung change with nothing wrong,
        // because a rung change moves the cells drawn from the theme's glyph table and no others,
        // and **23 990 of 24 000** under a tier change because the colour axis moves nothing on any
        // canvas. Both numbers are the delta read from its two ends, and neither can tell
        // *the swap reached nothing* from *the swap had nothing to reach*.
        //
        // So `gallery::swap_on` plays a second arm — the same gallery, the same page, the same four
        // frames, with the destination theme in place from the first — and `Swap::stale` is what the
        // two disagree about. It is the engine's own `reference.rs` arrangement one crate up: the
        // expectation is generated rather than hand-written, *because a hand-written expectation
        // about damage is written by the person who wrote the damage*.
        //
        // **The barrier this row carried was ADR 0023 and it did not need lifting**, for row 7's
        // reason one ticket earlier: the count is read off `crate::runner::Pen`, which is where the
        // partition rule's other half has always been read, and where the two arms of a swap can be
        // compared cell for cell without any crate above the engine reading a cell.
        //
        // **And the memo enumeration really did have nothing to enumerate.** `crate::memos` is the
        // census and its finding is that **no shipped memo in this crate holds a paint or a
        // cluster** — three of them hold sub-cell bits, an axis domain and a wrap index — so ADR
        // 0030's rule is true here **vacuously**. A gate resting on it would be green for ever, so
        // the memo the rule is about is built as an axis instead: `gallery::Keying`, right arm and
        // wrong arm over the same twenty-eight call sites, and `Keying::DataAndTier` is what this
        // row is watched failing on — **2 403 of 3 000 at 100x30 with `changed` reading 758**, which
        // is refinement 1 measured on this screen rather than recalled.
        standing: Standing::Evaluated {
            by: &[
                // The gate: every axis, every page, both arms of the remainder, so this row and
                // row 7 are green at the same time.
                Instrument::Unit {
                    file: GALLERY,
                    name: "no_cell_of_the_assembled_gallery_carries_the_previous_palette",
                },
                // Watched failing, on the memo the rule is about, with the old spelling green on it.
                Instrument::Unit {
                    file: GALLERY,
                    name: "the_tier_keyed_memo_is_wrong_on_this_screen_and_the_old_gate_passes_on_it",
                },
                // And the rule's own arm, with the memo proved live: a memo that never hits cannot
                // be stale.
                Instrument::Unit {
                    file: GALLERY,
                    name: "a_memo_keyed_on_the_themes_own_revision_is_never_stale_and_does_hit",
                },
                // The arithmetic §21 states, kept beside the screen.
                Instrument::Unit {
                    file: "crates/vitui-components/tests/gates.rs",
                    name: "changed_greater_than_zero_passes_on_the_exact_set_it_had_to_catch",
                },
                // The 598-character form, over a value that really is made of glyphs — components
                // 05 built it and this row is where it was owed.
                Instrument::Unit {
                    file: "crates/vitui-components/tests/glyph_matrix.rs",
                    name: "a_tier_keyed_memo_survives_a_palette_swap_and_is_wrong_after_a_repertoire_one",
                },
                Instrument::Report {
                    file: GALLERY_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 9,
        on_spec_table: true,
        gate: "regions identical at 1k and 1M",
        kind: Kind::Equality,
        owner: "C03",
        section: "spec §5",
        // **The spelling of what row 66 measures**, and components 12 gave it a subject: the
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
        // Subjected by components 12. The 208 does not reproduce — see
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
        // **Subjected by production ticket 03, and the standing was stale rather than the property
        // unreachable.** `table` has been declared since components 15 and `crate::grid::draw_into`
        // has drawn through it since — row 78 is the row that says so — so the subject this row was
        // waiting for arrived without anything editing the row.
        //
        // The counts are **12, 40, 120 and 240** over one 300x80 rectangle, and the whole `Shape` is
        // equal at all four: 24 000 writes, 1 600 verbs, 27 120 asked, eleven columns drawn. The
        // extras are appended *after* the nine that decide the window, so what stays flat is the
        // mechanism rather than the declaration list.
        //
        // **The four arms are asserted to be four different tables before they are asserted to draw
        // one screen**, because `crate::grid::columns` truncates and then appends: a list that had
        // stopped growing would leave this an equality between four copies of one thing, which
        // holds for ever. Each arm declares its own count and the band's `content_w` strictly grows
        // with it. The control is the same sweep drawn `Opts::clip_only`, which steps at every arm —
        // 1 760, 6 240, 17 440, 27 040 verbs — on **identical writes**, because the engine reports a
        // fully clipped verb as zero columns.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: GRID,
                name: "writes_and_verbs_are_identical_across_declared_column_counts",
            }],
        },
    },
    Row {
        number: 12,
        on_spec_table: true,
        gate: "the same table under a horizontal offset, against itself",
        kind: Kind::Equality,
        owner: "C04",
        section: "spec §6",
        // **Subjected by production ticket 03**, for row 11's reason and on row 11's screen.
        //
        // **The two sides are two programs.** One is `crate::grid::oracle_rows`, which walks the
        // 300x80 rectangle a cell at a time and inverts the map — screen x to band, band-local x to
        // column, by binary search over the cumulative array — and is drawn one
        // `Ctx::set` a cell through `crate::runner::Pen`. The other is `crate::collect::table_into`
        // at the same offset, which applies the map the other way: column to screen x, in runs. So
        // this is not two derivations of one declaration, and neither side is empty — the correct
        // arm is asserted to write `CELLS` cells exactly once, which is the partition from the
        // other direction.
        //
        // What the equality decides is **what an equality against a reference render can and cannot
        // see**. It refuses the inverted horizontal sign at 17 022 cells over all eighty rows —
        // the defect on this axis no counter *refuses*, because it issues every verb the correct
        // build issues and the clip eats 15 360 of the writes, so the two counters that do move
        // move the flattering way.
        //
        // **What it cannot see is the arithmetic band, and that is not this row's to gate.** The
        // band draws the right picture and re-damages 560 cells a frame for ever; the pair `writes`
        // against `distinct` is what sees it, filed at row 6, and the screen's own count of it is
        // row 76 — whose `the_equality_is_blind_to_the_band_and_the_pair_is_not` is therefore
        // **row 76's alone** and is deliberately not cited here. What this row takes from the
        // straddling instrument it *does* share with row 76 is the other half: the **correct** arm's
        // 800 cells, which is the recorder's own error under a clamp-and-discard clip and is why
        // `HOFF` is a column boundary. Priced rather than hidden.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: GRID,
                    name: "the_oracle_and_the_grid_are_two_programs_that_agree",
                },
                Instrument::Unit {
                    file: GRID,
                    name: "the_inverted_horizontal_sign_is_refused_by_the_equality_and_flattered_\
                           by_the_counters",
                },
                Instrument::Unit {
                    file: GRID,
                    name: "at_an_offset_inside_a_column_the_overrun_is_on_the_left_and_the_picture_\
                           is_wrong",
                },
            ],
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
        // The standing and its reason: what this row gates is that *the
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
        // **A stale standing rather than an unreachable property, and one half really was
        // missing.** It named components 24, which built `field` and then filed six *new* rows for
        // it instead of standing this one up — so it read *it could run and there is nothing to run
        // it over* for the rest of that backlog while two instruments ran over the shipped
        // component. Production ticket 04 found the gap: both of them inspect a caret over a buffer
        // **that does not move**. A `step_right` walk of the corpus and a sweep of the gestures are
        // constructions; the two defects arrived on an *edit*, where `Text::edit` re-seats the
        // pair by walking from the start of the **new** visual row.
        //
        // `document::edit_walk` is that script — both ends of the buffer, the seat, an insert of one
        // cluster spelled as two `char`s, a backspace, a delete, and then **three** undos, which is
        // the whole history back to the buffer the seat was made in — at each of 144 seats, 1 296
        // inspections.
        //
        // **The three producers are counted apart, and that is the finding.** A review of this
        // ticket's first version found one number over all three and a zero being read as *the
        // shipped verbs are right across an edit*, when for one of the three it could not have been
        // anything else. `document::Placed` carries the split:
        //
        // - **`Walked` is structurally zero.** `Text::edit` re-seats the pair by walking cluster
        //   steps from the new row's start, so it returns a member of its own walk — and a checker
        //   walking from byte 0 is no more independent, because it passes through that row start and
        //   continues with the same steps. A `Text::edit` patched to seat a byte-addressed caret
        //   leaves **715 wrong columns and zero off-boundary carets**, and a `walked_to` patched to
        //   land one cluster past every edit leaves zero as well. Gate 1 asks nothing here; **gate 2
        //   is what fires**, which is this row and row 16 being two rows rather than one.
        // - **`Restored` is where gate 1 has teeth after an edit.** `Text::undo` restores the pair
        //   rather than recomputing it (§11: 0.0007 µs against 6 109), so a restored pair is only as
        //   good as the buffer it is restored into. The defective arm demonstrates it **without
        //   patching `undo`**: the third undo puts the seat's own pair back, and on the `AtByte` arm
        //   that pair is inside a cluster — 45 of them, the same 45 as the seat's, so the arm's
        //   total is 90.
        // - **`Seated`** is where a caret enters from outside, and the other 45.
        //
        // **Both ends of the buffer, and neither of them is `Home` or `End`** — those are the ends
        // of a caret's own *visual row*, so on row 0 neither is an end of a wrapped document. The
        // far end is `select_all` and byte 0 is a click on row 0 column 0, **in that order**: a
        // `Text` is constructed with its caret already at byte 0, so a `Home` counted first is a
        // comparison that cannot fail — which is what the first version did, and substituting `home`
        // for the click now reports 144 ends against 288.
        //
        // **The third instrument is the second one's precondition and not a third claim.** The
        // boundary check walks from the caret's own **row start**, because
        // `boundaries(buf).contains(&byte)` at 1 296 inspections is a gate that takes twenty seconds
        // instead of four — and a walk begun at a row start is only the same walk if a row start is
        // a boundary. `every_row_start_is_a_cluster_boundary_of_the_whole_buffer` is one full walk
        // that says so. Three vacuity refusals inside the gate: all three undos are asserted to have
        // **applied an entry**, the two arms are asserted to differ by the seat and by nothing else
        // — every non-failure count, ends and undos included, because the failure counts are read
        // off the defective arm — and a caret off a `char` boundary is counted rather than sliced,
        // which a patched `undo` found by taking the instrument down with a slicing message.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_caret_is_on_a_boundary_at_both_ends_and_after_every_edit",
                },
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_caret_is_always_on_a_cluster_boundary_and_the_off_boundary_arm_is_\
                           not",
                },
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "every_row_start_is_a_cluster_boundary_of_the_whole_buffer",
                },
                Instrument::Unit {
                    file: EDIT,
                    name: "every_gesture_lands_on_a_boundary_at_the_engines_own_column",
                },
                Instrument::Unit {
                    file: EDIT,
                    name: "the_two_caret_defects_fail_the_two_caret_gates_and_ascii_hides_both",
                },
                Instrument::Report {
                    file: FIELD_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 16,
        on_spec_table: true,
        gate: "the caret's column equals the engine's tables over the prefix",
        kind: Kind::Equality,
        owner: "C06",
        section: "spec §11",
        // **The tables are the engine's and not a count**, which is the whole of this row:
        // `vitui_runtime::layout::text::width` is `vitui_engine::width_of` under another name, and
        // `crate::edit::defective::column_by_chars` — one column a code point — is the *defect*
        // rather than the check. It is right on ASCII and wrong on every cluster of
        // `crate::clusters::corpus`, which is why a caret gate run over ASCII said nothing for two
        // tickets.
        //
        // **The prefix is the caret's own visual row and not the buffer**, which is the half §11's
        // wording leaves to be read: a caret's column is a *screen* column, re-seated at every row
        // start, so over the 64-line document a full-prefix column is a screen column on row 0 and
        // nowhere else. That is what makes `defective::at_byte` — whose column *is* the tables over
        // the whole prefix — wrong here on 143 of 144 seats while being right on the one-line
        // corpus its own doc was written against, and it is why this row and row 15 stay two rows
        // while sharing two instruments: one arm can be wrong on either gate alone.
        //
        // Watched failing on a `Text::edit` that re-seats the pair with `at_byte`: 715 of 1 296
        // inspections carry a column the tables disagree with, **while gate 1 stays at zero** — and
        // row 15 says why that zero is structural rather than a pass. This is the row that fires on
        // the whole `Placed::Walked` class, which is the sharp form of §11 stating two gates and not
        // one.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_carets_column_is_the_engines_tables_and_counting_code_points_is_not",
                },
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_caret_is_on_a_boundary_at_both_ends_and_after_every_edit",
                },
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "a_char_caret_lands_inside_clusters_and_changes_widths_nobody_typed",
                },
                Instrument::Unit {
                    file: EDIT,
                    name: "every_gesture_lands_on_a_boundary_at_the_engines_own_column",
                },
            ],
        },
    },
    Row {
        number: 17,
        on_spec_table: true,
        gate: "a spliced wrap index equals a rebuilt one",
        kind: Kind::Equality,
        owner: "C06",
        section: "spec §11",
        // **The population is stated because one case is what a 499-of-500 defect hides behind.**
        // The finding is a splice restarted at `row_of(at)` that agreed with a rebuild *499
        // times in 500* and drew a screen that looked right; a gate over one edit is green with
        // probability 0.998. `document::splice_trials` is 500 deterministic insertions — xorshift64
        // at a fixed seed, and **whitespace is among the inserts**, without which the population
        // cannot reach the defect at all, since the case that moves a break backwards is a word too
        // long for the previous row acquiring a break opportunity inside it.
        //
        // **Two sweeps over one population, and the shipped one is cited first.**
        // `component_splice_sweep` plays all 500 through `Text::insert`, where the restart point is
        // the one `Text::edit` chose and the naive arm is one field on the state
        // (`defective::restarted_at_row_of`); `splice_sweep` asks `Index::spliced` at a point the
        // test computes. **The two report the same three numbers**, which is what makes the cheap
        // one a stand-in rather than a second population that happens to agree — and it is not an
        // equality between two derivations of one declaration, because the restart point is spelled
        // once inside `Text::edit` and once inside the sweep, in two files.
        //
        // **Nine of 500 and not one.** The remembered figure is asserted as remembered
        // (`document::REMEMBERED_SPLICE_AGREEMENTS`) and reported beside the measured one; the count
        // is gated as a floor rather than as a number, because it is a property of the five hundred
        // insertions and not of the mechanism. Watched failing by giving the shipped arm the naive
        // restart: 9 disagreements in 500 against 0.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_components_five_hundred_splices_agree_with_a_rebuild_and_the_naive_\
                           point_does_not",
                },
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
                    file: EDIT,
                    name: "a_keystroke_at_a_megabyte_splices_and_the_splice_is_the_rebuilds_answer",
                },
                Instrument::Report {
                    file: FIELD_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 18,
        on_spec_table: true,
        gate: "the index's recorded width equals the width being drawn",
        kind: Kind::Equality,
        owner: "C06",
        section: "spec §11",
        // **`the width being drawn` is the load-bearing half, and nothing was asking it.** The two
        // instruments that were here read an *inequality* between two indexes neither of which was
        // drawn — one built at 300, one at 120 — and a count of the rows the surface differs by.
        // Production the gate reads the width off the state **after
        // `crate::input::field_into` has drawn it**, so the number it is compared against is the
        // rectangle the component was handed rather than one the test chose:
        // `built_at() == screen.w`.
        //
        // Both arms enter the frame holding an index built at 300 and draw at 120, which is the
        // resize. The correct key — `(revision, width)` — misses and rebuilds; the defective one
        // **hits**, so what it draws with was built before the resize. Watched failing in both
        // directions on the same frame: the shipped arm given `defective::keyed_on_revision`
        // answers 300 where 120 is drawn, and the stale arm with the key restored answers 120 where
        // the row asserts 300.
        //
        // **`recomputes` points the wrong way and may not be gated on**: the defect recomputes once
        // where the correct key recomputes twice, so the cheaper number is the wrong build. That is
        // why the width is a *field* of `Index` and not only a key — an index that did not record
        // its width would leave this row with nothing to fail.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_index_the_field_drew_with_was_built_at_the_width_it_drew_at",
                },
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "a_memo_keyed_on_the_revision_alone_draws_an_index_built_at_another_\
                           width",
                },
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_resize_differs_on_sixty_nine_of_eighty_rows",
                },
                Instrument::Unit {
                    file: EDIT,
                    name: "two_hundred_caret_steps_are_one_index_and_a_resize_is_a_second",
                },
            ],
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
        // The table records this as one of the **two** gates that were being run at the
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
        // **Components ticket 21 put the accordion under it.** `tests/gates.rs` states the
        // rule at twelve sections of five focusables — 72 against 12, the same shape — and the
        // accordion is the shape at the scale: 421 hit entries against 13 and 420 tab stops
        // against 12, which is 408 on both columns and the `478 - 70 == 423 - 15` exactly.
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
        // block against 0 reanchored, both the numbers, both exact.
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
        // rather than between one and itself: *one open detail row is two comparisons and a
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
                    name: "the_axis_is_named_in_five_files_and_one_of_them_is_a_components",
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
        // for *the offset* cannot say either, which is why criterion 4 asks for the subjects
        // separately.
        //
        // **The third subject is the original's and it asks a different question.** `table` owns
        // one offset in rows, like a collection, and every number it reports **is** a collection's
        // — which is the point rather than a redundancy: spec §6 opens by claiming a table's *row
        // axis, wheel, keyboard, type-ahead and reveal are all `collection`'s, reached by calling
        // it*, and until that ticket nothing had asked it a wheel question. It is asked as an
        // equality — *`table` equals `collection`, arm for arm*, over all three `Reveal` arms and
        // all four columns — so a column split that grew an offset or a reveal of its own fails
        // here, where three constants written twice would have passed. Scene 38.
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
                    file: WHEEL,
                    name: "twenty_posted_clicks_move_a_tables_offset_twenty_and_the_numbers_are_\
                           the_collections",
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
        gate: "O1-O7, as queries over `INVENTORY`",
        kind: Kind::Count,
        owner: "C10",
        section: "spec §17",
        // **Six of the seven are green: O1 since components ticket 36, O3 since 37, O4 since 38,
        // O2 since 39, O6 since 44 and O7 since 45.** O7 is the seventh, stated after the map
        // closed like O6 and filed on the implementation backlog for O6's reason — the instrument
        // is buildable without reopening anything — and it is the one query whose **evidence is in
        // another crate**: `crate::consumer` joins the freeze against the import paths in
        // `crates/vitui-apps/examples/`, because *a gate exercises the component where its author
        // put it, and an application puts it somewhere else*. Its population is neither `built` nor
        // derived but **read out of the source**, which is components 33's finding as a population:
        // the `built` column read `true` for `slider` through two tickets with no `slider` anywhere
        // in the crate. O6 is the sixth obligation, stated after the map closed, and it is
        // the one query of the seven whose **population is derived** rather than written out —
        // `crate::volume::population` reads the freeze's `Layer::L2` column and the memo census, and
        // it answered seven where the ticket's own parenthesis named six. O4's population is the **union of the two lists** — thirteen of twenty-nine — and
        // not the freeze, because sixteen rows read no key at all and *every component appears in
        // both lists* is not the obligation. Its own three halves are rows 216, 217 and 218; the
        // roll-up to ids that this row can ask over the freeze would be satisfied by a component
        // declaring one binding and answering a hundred.
        // It stood at zero of six for thirty-five tickets — O2 is two equalities — twenty-five of
        // which shipped a component entitled to add a row to `DOC_TESTED` and none of which did,
        // because a list filled by whichever ticket happened to write a doctest is a list nobody
        // audits.
        //
        // **Both populations are `built`, and the second one is the first one's finding a second
        // time.** §17 states O2's second equality over `built` and states O1's count and O3's count
        // over nothing at all, so the reading was owed twice; `spinner` is the one row no ticket has
        // built, a doc page for a function that does not exist is not a page anybody can write and a
        // golden of one is not a screen anybody can draw, and asking for either puts a permanent row
        // in the failing set that no ticket on this backlog can invert. *A query stuck red is as
        // uninformative as a query vacuously green.* Both populations move on their own: the day
        // `spinner` ships, `crate::doc::pages` returns twenty-nine and O3 asks for thirty-four
        // screens.
        //
        // **This row is `Kind::Count` and O1 is two gates**, which is what the `mixed` in the
        // kind column was hiding. The compile-outcome half is row 210, the axis equality row 211
        // and the refusals row 212 — three rows rather than a fourth kind, because a row that is a
        // count and a compile outcome at once is a row no `Kind` can print.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/obligations.rs",
                    name: "all_nine_obligation_queries_are_met_and_o5_was_the_last_to_turn",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/obligations.rs",
                    name: "each_query_reports_the_population_it_could_not_answer_for",
                },
                Instrument::Unit {
                    file: DOC,
                    name: "every_built_component_carries_a_page_with_a_compiled_example",
                },
                Instrument::Unit {
                    file: DOC,
                    name: "the_written_list_and_the_scan_agree",
                },
                Instrument::Report { file: DOC_NUMBERS },
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
                // **G10b, added by components 32 where the component that needs it lives.** §15
                // rewrites G10 as a sentence about `Cell` and `RefCell` as well as about `Task`,
                // and that half had nowhere to hang until `crate::files` existed. It is the
                // compiling case: what `Cell` and `RefCell` remove is `Sync`, not `Send`, and
                // `Worker` is both — which is the correction this row's own note already carried.
                Instrument::Pair {
                    file: FILES,
                    hostile: "needs_sync(&task);",
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
    // ── the row, and it is a barrier rather than a gate ────────────────────────
    Row {
        number: 45,
        on_spec_table: false,
        gate: "§16's matrix at all nine cells — the repertoire axis against the colour axis",
        kind: Kind::Count,
        owner: "C09, C10, C11",
        section: "spec §16",
        // **Inverted by components 39, and the `Barrier` it carried had decayed into the near-miss
        // runtime architecture issue 22 warned about.** The row read `Unreachable { needs:
        // "`ColorDepth`" }` and named issue 22 as its inverter — *the issue that lifted the
        // barrier*. `crates/vitui-runtime/src/line.rs` files `ColorDepth` as
        // `reachable_as: Some("vitui_runtime::ColorDepth")`, so both axes are values this crate can
        // hold and `Theme::resolve` is a call it can make: **a citation that still passes while
        // meaning the opposite**, which is exactly what that issue's own header says a `Barrier`
        // invites.
        //
        // **The nine cells are taken over the assembled gallery**, which is the half neither the
        // runtime's `theme_numbers` nor its `glyph_numbers` can do: those own the mechanism and
        // measure it over the palette, and this one measures what a human is looking at.
        //
        // **And the colour half is measured on the theme rather than on the screen, which is the
        // finding.** `Theme::resolve` returns the **same paint** for all thirteen roles at all four
        // depths — asserted, not described — because quantisation is the engine's and happens before
        // the mirror. So the screen's own paint count reads ten in all nine cells and would read ten
        // on a monochrome terminal; the axis is `roles_differ_on_wire` and `Theme::shows`. That is
        // ADR 0018 working rather than a hole: a component is handed the palette's colour whatever
        // the terminal can show.
        //
        // **The criterion's own sentence does not reproduce.** *`Danger`, `Warn` and `Ok` all
        // quantise to bright white at sixteen colours* is true of **eight of the fourteen** shipped
        // schemes and of **fourteen of fourteen** at `ColorDepth::None` — so it is the right claim
        // about the wrong rung, asserted as measured on this map's own discipline.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: GALLERY,
                    name: "the_matrix_is_nine_cells_and_the_two_axes_move_in_different_columns",
                },
                Instrument::Unit {
                    file: GALLERY,
                    name: "the_traffic_light_is_read_on_the_wire_and_a_paint_count_cannot_see_it",
                },
                Instrument::Report {
                    file: GALLERY_NUMBERS,
                },
            ],
        },
    },
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
    // ── the five ──────────────────────────────────────────────────────────────
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
    // ── the rows ──────────────────────────────────────────────────────────────
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
    // ── the rows: the dense screen ────────────────────────────────────────────
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
    // ── the rows: the four primitives ─────────────────────────────────────────
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
                // The sentence: `distinct == w × h − interior`, with the interior the panel named
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
                // Row 48 is the form of the same eight cells, measured through `press`
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
    // ── the two, and they are two different kinds of row ──────────────────────
    Row {
        number: 66,
        on_spec_table: false,
        gate: "regions identical at 1 000, 100 000 and 1 000 000 rows",
        kind: Kind::Equality,
        owner: "C11",
        section: "spec §5, §20",
        // **This is not row 4 restated, and the difference is the finding.** Row 4 is *writes flat
        // 1k -> 1M* — `Unsubjected` when this comment was written and subjected by components 12
        // since; this row asks about the **hit index**, and the reason
        // it has to is that the write count cannot see the defect at all: the engine reports a
        // fully clipped verb as zero columns, so a listing that iterates its whole content and lets
        // the clip reject the rest writes exactly what the windowed one writes — 3 200 at every
        // volume — while declaring 1 000 001 regions against 81. Row 4 would be green on it.
        //
        // `Evaluated` over the listing rather than over `collection`, which is the same standing
        // The four rows have and for the same reason: what it gates is that
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
                    name: "two_scenes_have_nothing_to_run_over_none_is_red_and_\
                           forty_five_are_stood_up",
                },
            ],
        },
    },
    // ── the rows ──────────────────────────────────────────────────────────────
    Row {
        number: 68,
        on_spec_table: false,
        gate: "the reserved decision has hysteresis when it is computed from last frame's reduced \
               rectangle: both bars over content that fits, 19 of 20 rows for ever",
        kind: Kind::Equality,
        owner: "C13",
        section: "spec §9",
        // The second sentence, as a gate: **reserved auto-hiding bars require a declared
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
        // build whose row-measured frame is visibly different, and the whole claim is that it is
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
        // shipped code: `crate::scroll::thumb` is the original's and this row hands it two
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
        // **The scene-19 row asks for an amplification factor and this row deliberately does not
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
        // `iterated` rather than on `writes` — the finding arriving on the
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
    // ── the two ───────────────────────────────────────────────────────────────
    Row {
        number: 73,
        on_spec_table: false,
        gate: "cells asked for is independent of depth, and no other counter is",
        kind: Kind::Count,
        owner: "C05",
        section: "spec §7",
        // **The counter §7 names and the table does not carry.** A row's cost may read a depth
        // and may not be proportional to one, and the whole of what an unclamped indent does is
        // invisible to every counter that ships: the engine reports the same columns written, the
        // same distinct cells, the same regions and the same stops — and **fewer verbs**, because
        // the label rectangle collapses, and **less time**, because the clip discards the overrun
        // before the row is built. Eight of the nine counters prefer the defect and the ninth is
        // `Unreachable`.
        //
        // `Evaluated` over the forest's two arms rather than over `tree`, for row 66's reason. It
        // is a `Count` and not a `Ratio` because the ratio is this screen's — 399.99x here against
        // The 165x over a screen with a menu bar on it — while *the ask does not move with the
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
                    name: "two_scenes_have_nothing_to_run_over_none_is_red_and_\
                           forty_five_are_stood_up",
                },
            ],
        },
    },
    // -- The four, and the third is not about this crate ----------------------
    Row {
        number: 75,
        on_spec_table: false,
        gate: "writes and verbs identical at 12, 40, 120 and 240 declared columns",
        kind: Kind::Equality,
        owner: "C04",
        section: "spec §6",
        // **This was row 11 evaluated over a screen rather than row 11 turned green**, while row
        // 11 waited for `table` to exist — components 15's — and **production ticket 03 turned row
        // 11 green on the same instrument**, so the two rows now share it and the division of
        // labour between them is gone. That is not a duplicate: what *this* row gates is that the
        // instrument separates a correct build from a defective one, watched in both directions —
        // the virtualised arm flat across all four declared counts, the clip-only arm at 10.9x the
        // verbs for the same 24 000 writes — where row 11 gates the property itself. Components
        // row 66 has the standing this one had, and its reason.
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
        // defect: the clause says why, *the screen is correct, because the pinned band draws
        // afterwards and wins*, so the two surfaces are 0 cells over 0 rows apart. What sees it is
        // the pair `writes` against `distinct`, which is row 6, C02's, filed by §21 as a *report
        // per component*. So *no gate left by C01, C02 or C03 sees it* is half true: C02 named
        // the counter and nobody was running it.
        //
        // **The straddling instrument is shared with row 12 and the halves are different**, which
        // production ticket 03 had to say out loud when it subjected that row: this row takes the
        // arithmetic arm's 1 600 re-damaged cells at an offset inside a column, and row 12 takes
        // the *correct* arm's 800 — the recorder's own error under a clamp-and-discard clip, which
        // is a number about the equality and is why `HOFF` is a column boundary. `the_equality_is_\
        // blind_to_the_band_and_the_pair_is_not` is **this row's alone**; a row 12 that cited it
        // would be resting on a gate the register says does not decide its property.
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
        // and it is here rather than nowhere because the rule for a red gate is *assert the exact
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
    // ── the two, and both are about what a count can see ──────────────────────
    Row {
        number: 79,
        on_spec_table: false,
        gate: "a body declares a count proportional to the rectangle it was handed, and one that \
               ignores it declares a count flat in the height",
        kind: Kind::Relation,
        owner: "C14",
        section: "spec §8, §21",
        // **A relation and not an equality, because the number belongs to the body.** The original's
        // example is `verbs <= writes` and its own sentence is *never verb equality across sizes*;
        // what is asserted here is the *shape* of the two curves over every height from one to ten,
        // not a pair of magnitudes. §8 states one point on them — 273 ring entries against 247 —
        // and the point is the height where the difference is 26, which is two rows of body left.
        //
        // `Evaluated` over the accordion rather than over `collapsible`, which is components
        // The standing. The scene stays red; `crate::scenes` says why those are not one
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
        // The nine: the two surfaces are 0 cells over 0 rows apart, `writes`, `distinct` and
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
    // ── the five, and four of them are about what a counter cannot see ────────
    Row {
        number: 81,
        on_spec_table: false,
        gate: "every gate of spec §11 runs on clusters that are not one code point, and the same \
               gate over ASCII reports nothing",
        kind: Kind::Count,
        owner: "C06",
        section: "spec §11",
        // **The row that makes the corpus a value rather than a string literal.** The sentence is
        // *every number and every gate runs on clusters that are not one code point*, and a
        // sentence is what gets broken by somebody who has read it. What is asserted is the
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
        // This is the `counters_that_separate_them` one component over, run over
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
        // scene 13, as an equality against a correct render rather than as a row count. The
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
        // comment because the whole argument is that an obligation stated as a sentence gets
        // broken by someone who has read it.
        //
        // `CompileOutcome` for row 61's reason: what is asserted is that a *file* declares an item,
        // read by opening it — the one thing a `compile_fail` fence cannot say, because a fence
        // over a missing item passes today and passes again the day the module is renamed.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DOCUMENT,
                    name: "the_document_stands_on_its_declared_subject",
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
                    name: "two_scenes_have_nothing_to_run_over_none_is_red_and_\
                           forty_five_are_stood_up",
                },
            ],
        },
    },
    // ── the four ──────────────────────────────────────────────────────────────
    Row {
        number: 86,
        on_spec_table: false,
        gate: "the regions and the stops of §12's five configurations, exactly",
        kind: Kind::Equality,
        owner: "C07",
        section: "spec §12",
        // **Two of the seven columns reproduce to the unit and two of them cannot.** The two that
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
        // **The row that contradicts a closed map, and it is asserted as measured.** The table
        // reads `allocations = 0` in all five rows; the shipped figure is `n + 1` for `n` overlays
        // standing and 0 for none, because runtime ticket 21 deleted the bump arena that made the
        // zero true. The gate is the **marginal** equality rather than an absolute,
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
        // **The third refinement, and this is the first screen it can run on.** *Name the
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
                // **Components ticket 35 ran it over a form**, which is the population O4
                // names — spec §18 calls `form` R3's own example, and the impl backlog calls it *the
                // natural host for the walkthrough gate*. The ungrouped arm is the one that says
                // something: 6 of 6 with nothing standing, 1 of 7 with a modal over it, and the trap
                // naming itself.
                Instrument::Unit {
                    file: INPUT,
                    name: "the_walk_over_a_form_repeats_no_id_and_reaches_every_stop_unless_a_\
                           trap_is_standing",
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
        // **Green since components 26, and the direction is what makes the row worth keeping.** It
        // was `Unmet { over: 2, failing: 2 }` with every number on the screen already reproducing:
        // what was missing was the subject, and the scan is what could say so. It says the opposite
        // now, and the *same* scan is the thing that would notice either subject going away.
        //
        // The needle for `select` had to change with it, and that is a finding rather than an edit:
        // it read `pub fn select(`, and spec §1 already says that a component which opens an overlay
        // costs two lifetime annotations — so a scene that had gone green on the old needle would
        // have gone green by deleting the `'f`, which is a different component.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: POPUP,
                    name: "the_screen_stands_on_its_subjects",
                },
                Instrument::Unit {
                    file: POPUP,
                    name: "the_needle_the_red_scene_carried_could_not_have_matched_a_component_\
                           that_opens_an_overlay",
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
                    name: "two_scenes_have_nothing_to_run_over_none_is_red_and_\
                           forty_five_are_stood_up",
                },
            ],
        },
    },
    // ── the rows: the collection ──────────────────────────────────────────────
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
        // The second half is the other direction of *six inventory entries collapse into
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
        // refused are `crate::collect::stores`, kept runnable so the table is a measurement rather
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
        // **`O(log k + h)` against `O(h · log k)` as a count**, which is the rule: a gate is a
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
        // **Two claims and one screen, because they fail together.** *one hit entry per
        // collection* is what makes per-row hover arithmetic off `Response::local` rather than 389
        // regions for a frame-old answer; *the row loop is wrapped in `cx.with_id`* is
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
        // The 208 does **not** reproduce and `crate::collect::COLL_STATE_BYTES` says why rather
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
    // ── the rows: the order, the index and the memo ───────────────────────────
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
        // the drift *three names for one mechanism is already one too many* is about.
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
        // The magnitudes are this crate's corpus and the original's are C06's; the *shape* is what
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
    // ── the three ─────────────────────────────────────────────────────────────
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
        // verb fewer**. Every counter but the pair prefers it, which is the signature
        // arriving on this screen.
        //
        // `Evaluated` over the screen rather than over `chart` and `plot`, which is components
        // The standing and its reason: what it gates is that *the instrument separates a
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
                    name: "two_scenes_have_nothing_to_run_over_none_is_red_and_\
                           forty_five_are_stood_up",
                },
            ],
        },
    },
    // ── the three ─────────────────────────────────────────────────────────────
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
        // narrower key folds *fewer* times and is wrong. The fifth independent arrival of that
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
    // ── the four, and one of them is a defect one crate down ──────────────────
    Row {
        number: 109,
        on_spec_table: false,
        gate: "one click on a column header costs one run at any length, and the flattened index \
               costs one a row",
        kind: Kind::Ratio,
        owner: "C04",
        section: "spec §6",
        // **A ratio and not a timing, which is the whole of why the two candidates are decided by
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
        // precedent that says it belongs here. It is the one row on this register that a runtime
        // ticket inverted: `Ctx::with_id` — which `Ctx::with_key` is written on —
        // re-childed the view at `self.area()`, and `area()` is `Rect::new(0, 0, w, h)` in the
        // *current* coordinate system. Inside a scroll scope that origin is the content's, so the
        // clip it intersected with was content rows `0..h` while the window sat at the offset.
        //
        // **Runtime architecture issue 31 inverted it**, and the failing set it named is what the
        // gate now asserts green: at offset 100 over an 8-row view the keyed loop lands 8 of 8,
        // where it landed 0. The unkeyed arm and the two offset-0 arms stay in the test as the
        // control — the defect was invisible at the one offset every caller on this map draws at,
        // so a gate that dropped them could not tell a broken `with_key` from a broken
        // `scroll_scope`.
        //
        // `collection` and `sheet` still push their id outside the scope, which is now a choice
        // rather than a workaround; `table` still hands a cell's id down on `Cell::id`, whose
        // arithmetic is `with_key`'s and whose value a consumer can hold.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: COLLECT,
                name: "a_with_key_inside_a_scroll_scope_reaches_the_window",
            }],
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
    // ── the eight ─────────────────────────────────────────────────────────────
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
        // three of the four. The criterion is *a gate asserts its size*, and that criterion is
        // this ticket's, so nothing had run over it. Narrowing it is what makes the memory figures
        // reproduce: 7.63 MiB at a million rows and 11.44 with the prefix sum, against the 7 → 11.
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
        // pair is two ways of computing one number rather than two numbers. The 115 µs against
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
        // `Order::fold` rather than a branch, which is the instruction followed literally.
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
        // does — 1 against 80. The 0.019 µs against 0.0006 is the report beside it.
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
        // **The sentence**, measured against the same rectangle drawn as a plain list — the
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
        // the label identically — which is the statement about the scene list — and past the
        // rectangle the clamp reserves two columns while the defect leaves none.
        //
        // The third instrument is the lesson applied before it could be paid
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
        // over a real fold, with two trees on one screen for the half.
        //
        // The pointer half reads `Response::press_began` rather than keeping a second `pressing`
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
        // **Spec the pinned-column finding on the other axis, which is what §9 asks to be shown.**
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
        // It is the finding from the third side — *a translated band makes
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
        // **criterion 6, and it is a join rather than a claim.**
        //
        // The gate above runs over **four** subjects since production 09, and *which* four is a
        // decision this crate makes in one file. `file_preview_pane` is the fourth and it is
        // `Subject::Table`'s shape on another family: the pane hands its rectangle to
        // `scroll_area`, so its offset, its clamp and its reveal are all reached by calling it, and
        // the arm's assertions are *equal to the area's, arm for arm and axis for axis*. `INVENTORY`'s `owns_offset` column is exactly
        // `Axis::Wheeled`, so asking whether the freeze agrees is one question — and the failure it catches is a gate run over a
        // component the freeze says owns no offset, which reports a number while measuring nothing.
        // It is the same shape as O5 and for the same reason: **O5 is a query about axes and not
        // about components**, so a pair claimed here and unclaimed there is evidence for nothing.
        //
        // **The equality behind it is over sorted lists since production 06, and it was over
        // ordered ones.** `AXIS_SCENES` is scene order — what `crate::scenes::axis_scenes` derives
        // — and `Subject::ALL` is the drive loop's enumeration order. The two coincided only while
        // the gate's subjects happened to be the first wheel scenes written; `table` joined a gate
        // that predates `field`'s scene and took a scene number after it, and the orders parted.
        // Sorting both sides asks the population question exactly: an element on one side and not
        // the other still fails, and a coincidence of two unrelated orderings stops being
        // load-bearing.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: WHEEL,
                    name: "every_subject_of_this_gate_declares_the_wheeled_axis",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/obligations.rs",
                    name: "o5_counts_a_pair_for_every_axis_a_posted_notch_is_played_over",
                },
            ],
        },
    },
    // ── the six, and three of them are findings rather than criteria ──────────
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
        // **The sentence on the height axis, and it is the reason `Height::Watermark` is off by
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
        // ink, so the watermark arm makes the body's verbs twice. *+11.5% of the frame* is a
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
        // ***0 ring probes against 405*, followed to the end.** §8 says out loud why the left
        // half is free — *which is what a focusable widget does on a click anyway* — and the
        // consequence is that all three self-close gestures leave the focus off the body before the
        // vanish rule looks, each for a different reason: the press award focuses a focusable header;
        // the press award **defocuses** on one that is not a tab stop, because a press landing on
        // nothing interested is read as intent; and `Enter` needs the header to hold the focus
        // already. So the arm that pays is a collapse with **no gesture behind it**, which is where
        // `crate::accordion` runs it — a collapse-all with and without the caller capturing
        // `Frame::focus`, which is the answer to *`Stash` belongs to whoever owns the content's
        // identity*.
        //
        // What `Focus::Header` buys is measured rather than argued, and it is not the probe count: on
        // a header that is not a tab stop the runtime's answer is `None`, so the click **loses the
        // keyboard entirely** — the finding one component over.
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
        // ***an animated fold is refused* as a count rather than as a doc comment.** The removed
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
        // folding and inplace edit are one machine* is the claim the whole F4 family exists to
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
        // ***5 B of live state, 72 B with a tween slot*, and neither number reproduces** —
        // asserted as a disagreement, which is the standing one family over where
        // The stated widths were the number that was wrong.
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
    // ── the six ───────────────────────────────────────────────────────────────
    Row {
        number: 136,
        on_spec_table: false,
        gate: "`input` and `textarea` are one type and one flag, and the reduction that looks like \
               the flag has one boundary and it is byte 0",
        kind: Kind::Equality,
        owner: "C06",
        section: "spec §11, §18",
        // The headline is *one component and one flag* and §18 counts it as R1's largest single
        // collapse — sixteen named input variants including `textarea`. What makes it checkable
        // rather than a claim is that the two spellings are the **same type**: there is nothing a
        // textarea carries that an input does not, and `WrapKind` is the whole difference.
        //
        // **The near-miss is the row worth having.** *A ruler is a words index at a very large
        // width* reads as the same reduction and is not one: at `u16::MAX` a megabyte is one row,
        // so the caret's row start is byte 0 and `Left` is back to O(prefix) — which is the one
        // thing the flag exists to prevent. Measured, 1 row against 500 and byte 0 against one
        // window back.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: EDIT,
                    name: "an_input_and_a_textarea_are_one_type_and_one_flag",
                },
                Instrument::Unit {
                    file: EDIT,
                    name: "an_input_spelled_as_a_wide_words_index_has_one_boundary_and_it_is_\
                           byte_zero",
                },
                Instrument::Report {
                    file: FIELD_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 137,
        on_spec_table: false,
        gate: "there is no public way to place a caret at a byte offset, and no way to spell a \
               block selection",
        kind: Kind::CompileOutcome,
        owner: "C06",
        section: "spec §11",
        // **The API consequence is a deletion and a deletion has to be gated as one.** A caret is
        // placed by a gesture — a click's column, a cluster step, `Home`, or a pair some earlier
        // state stored — and each of those lands on a boundary by construction. An arbitrary offset
        // does not, and everything downstream preserves the bad caret faithfully: the column is
        // computed from it, the row is looked up from it, and **the screen looks entirely correct**.
        //
        // Two pairs and not one, because `Caret`'s fields are the same deletion three keystrokes
        // shorter: a `pub byte` is `set_caret` with a different spelling. Each pair carries a twin
        // naming the protected item by path, for the reason the runtime measured — rustdoc on
        // stable ignores the error code beside `compile_fail`, so `E0599` for *the method you must
        // not have was never built* and `E0599` for *the method you meant moved* are the same
        // diagnostic.
        //
        // The third deletion is block selection, and it is refused on a shape rather than on scope:
        // it is a rectangle over **visual** rows, which neither the run list nor the anchored range
        // expresses.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Pair {
                    file: INPUT,
                    hostile: "st.set_caret(7);",
                },
                Instrument::Pair {
                    file: INPUT,
                    hostile: "let _ = c.byte;",
                },
                Instrument::Pair {
                    file: INPUT,
                    hostile: "let _ = st.block_selection();",
                },
                Instrument::Unit {
                    file: EDIT,
                    name: "undo_restores_the_pair_and_the_byte_spelling_has_to_segment_the_prefix",
                },
            ],
        },
    },
    Row {
        number: 138,
        on_spec_table: false,
        gate: "the undo ring is bounded on entries and on bytes, and sixteen undos that each \
               answered yes leave the document unrestored",
        kind: Kind::Count,
        owner: "C06",
        section: "spec §11",
        // **The bound is where the silent defect lives**, and §11 says so in as many words: a ring
        // of sixteen entries after sixty-four edits runs sixteen steps, returns `true` every time,
        // and the document is not back to the original. That is indistinguishable from success
        // unless the ring says so — an undo that ran out of history and an undo that finished both
        // answer *yes, I undid something*.
        //
        // **Two bounds and not one, because one large paste is one entry.** A ring bounded only on
        // entries holds a megabyte per paste and 256 of them; one bounded only on bytes drops a
        // hundred keystrokes to make room for one. Both are watched evicting.
        //
        // **Coalescing is an entry count and never a time win**, and the rule is the word break
        // rather than the line break: §11 gives the reason in the same sentence as the ratio —
        // *undoing a sentence is 414 presses instead of 1 012* — and a run that closed only at a
        // line would make undoing a sentence **one** press, which is a checkpoint and not a
        // history. The 2.4x is a ratio over a corpus and is printed rather than pinned.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: EDIT,
                    name: "sixty_four_edits_through_a_sixteen_entry_ring",
                },
                Instrument::Unit {
                    file: EDIT,
                    name: "coalescing_is_an_entry_count_and_a_word_break_opens_the_next_entry",
                },
                Instrument::Unit {
                    file: EDIT,
                    name: "a_ring_at_cap_and_a_ring_emptied_index_the_same_document",
                },
                Instrument::Unit {
                    file: EDIT,
                    name: "backspace_at_the_start_and_delete_at_the_end_change_nothing_at_all",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "the_ring_at_cap_and_the_ring_emptied_are_the_same_frame",
                },
                Instrument::Report {
                    file: FIELD_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 139,
        on_spec_table: false,
        gate: "a text widget declares one region and the per-cluster spelling declares hundreds, \
               and the two draw the same screen",
        kind: Kind::Count,
        owner: "C06",
        section: "spec §11, §20",
        // §11: *a text widget declares 79 regions against 621 for one per visible cluster*. The two
        // magnitudes are a prototype's screen — this crate's is measured in
        // `examples/field_numbers.rs` — and what is a gate is the **structure**: one entry for the
        // widget, and a surface no reader can tell apart.
        //
        // It is the same shape as the 110-of-338 defect from the other side. There, widgets
        // merged into one id and the screen still rendered correctly; here, one widget declares
        // hundreds of entries and the screen still renders correctly. Both are invisible to every
        // counter that reads a cell, which is why the region count is the gate.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "a_field_is_one_region_and_the_per_cluster_spelling_is_hundreds",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "two_fields_at_two_call_sites_are_two_widgets",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "a_field_with_no_room_declares_nothing",
                },
                Instrument::Report {
                    file: FIELD_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 140,
        on_spec_table: false,
        gate: "a hundred kilobytes and a megabyte are the same frame, and the frame's verbs are the \
               window's rows",
        kind: Kind::Equality,
        owner: "C06",
        section: "spec §11, §20",
        // §11 prices the frame at *82.4 us / 0 marked / 19 634 writes / 631 verbs / 79 regions /
        // 0 merges / 0 allocations, **flat at 100 kB and 1 MB**.* The magnitudes are a screen this
        // ticket does not own; **flat** is the claim that is a gate, and it is the engine's own
        // invariant one layer up — *frame cost is proportional to visible cells, never to data
        // volume* — arriving on the component that holds the most data of any in the freeze.
        //
        // **`verbs` is a bound and not an equality, and that is a finding rather than a weakening.**
        // A row that exactly fills its width costs one verb and a row that does not costs two, so
        // twelve bytes of content and six hundred kilobytes of it differ by the **pads** rather
        // than by the buffer. `Ink::pad_to` exists for the sharper form of the same fact: written
        // as a text and a run, `verbs` separates the memo-key arms in the direction that approves
        // the defect, because a stale row is the whole line truncated and fills exactly.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "a_hundred_kilobytes_and_a_megabyte_are_the_same_frame",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "the_verbs_are_the_window_and_not_the_buffer",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "a_field_writes_each_cell_of_its_rectangle_exactly_once",
                },
                Instrument::Report {
                    file: FIELD_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 141,
        on_spec_table: false,
        gate: "the caret's shape is the one the widget asked for, and a chord types nothing",
        kind: Kind::Equality,
        owner: "C06",
        section: "spec §3, §11",
        // Two claims about what reaches the terminal, and neither is visible on a rendered surface.
        //
        // **The shape.** `Screen::set_cursor` applies all three of `{ x, y, shape }`, so a widget
        // that placed a caret and left the shape alone renders as a block on most terminals and a
        // bar on some — the same program looking different on two machines, which is the thing a
        // component library exists to stop. Both spellings ask for a bar, and the option is
        // watched changing it.
        //
        // **The keyboard.** `keys::text` is the helper and it is the whole rule: a release, a
        // chord and a control character never type. `keys::defective::on_code_alone` is watched
        // putting the accelerator's letter in the buffer, which is the code the runtime found in a
        // shipped field six tickets old.
        //
        // **And the caret is gated on `Ctx::is_focused` rather than on `Response::focused`**, which
        // is one frame apart: a `Response` carries the focus as it stood when the widget declared,
        // so a field just clicked would place no caret until the frame after.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "both_spellings_ask_for_the_shape_that_reaches_the_sink",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "a_chord_types_nothing_and_a_capital_types_a_capital",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "the_three_reveal_arms_are_three_different_programs",
                },
            ],
        },
    },
    Row {
        number: 142,
        on_spec_table: false,
        gate: "the rows do not tile the buffer, and every byte of it is still on a drawn row with a \
               caret on it",
        kind: Kind::Equality,
        owner: "C06",
        section: "spec §11",
        // **The row a review found and the gates did not**, and it is one fact with two failures.
        // `vitui_runtime::layout::text::wrap` trims each piece, so the whitespace a greedy break
        // consumed and any trailing whitespace belong to no row's *content* — the rows are not a
        // partition of the buffer, only of what is drawn.
        //
        // A component that placed its caret by asking which drawn row *contains* the byte therefore
        // loses it outright for an ordinary typing state: **one trailing space is enough**, and
        // `Frame::caret` is then `None`, so the terminal cursor disappears on a screen that is
        // otherwise correct. On a contiguous `Ruler` index it fails the other way — a byte on a
        // shared boundary satisfies the row *before* it first, and the caret jumps to the start of
        // the row it has just left. `Index::row_of` answers both, once.
        //
        // The same fact reaches the walkers: `End` on `"hi "` landed at byte 2 and `Ctrl+A`
        // selected `"hi"`, because both stopped at the trimmed content's end. `Index::row_bound` is
        // the row's *caret* extent and it makes two subtractions the click sweep found — a hard
        // break is stripped, and a row followed by another stops one cluster short of that row's
        // start, because that byte is the next row's own column 0.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: EDIT,
                    name: "end_reaches_a_trailing_space_and_every_byte_of_the_buffer_is_on_a_row",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "the_caret_survives_a_trailing_space_and_a_soft_wrap_boundary",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "a_selected_row_is_still_a_partition_of_its_width",
                },
            ],
        },
    },
    Row {
        number: 143,
        on_spec_table: false,
        gate: "a field consumes the notch it declared, and a window past the last row is pulled \
               back",
        kind: Kind::Count,
        owner: "C06",
        section: "spec §11, §17",
        // **Components the defect class, and `field` is not a subject of that gate.**
        // `INVENTORY` gives `field` `owns_offset: true` and `scrolled: true` and the options fold in
        // `Interest::SCROLL`, and for one review the component read `Response::scrolled` nowhere at
        // all. A widget that declares the pointer and does nothing with it is **worse** than one
        // that declares nothing: it is the topmost region over its rectangle, so the wheel does
        // nothing there *and* an enclosing `scroll_area` never sees the notch either.
        //
        // **The gate is watched catching the sign**, which is the mistake the repair itself made
        // first: `crate::collect` establishes the convention as `offset + resp.scrolled.1`, and
        // `-resp.scrolled.1` leaves the offset at 0 through four notches. And **there is no
        // horizontal axis to be wrong on**, which is a fact about the component rather than a hole —
        // both break rules wrap to the rectangle's width, so a row never overflows it. That is
        // *a body dead downward is alive sideways* answered by construction.
        //
        // Beside it, two out-of-range answers that read as data: `Index::row_start` answers 0 out of
        // range, so a window past the last row paints the caret on a blank filler row whose start is
        // row 0's, and `Index::spliced` at `rows()` keeps every row as the head and rebuilds the tail
        // from byte 0 — a **doubled** index rather than a panic.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "a_notch_over_a_field_moves_its_window",
                },
                Instrument::Unit {
                    file: EDIT,
                    name: "the_wheel_moves_the_window_and_stops_at_both_ends",
                },
                Instrument::Unit {
                    file: EDIT,
                    name: "a_window_past_the_last_row_is_pulled_back_by_the_reveal",
                },
                Instrument::Unit {
                    file: EDIT,
                    name: "a_splice_restarted_past_the_last_row_is_clamped",
                },
            ],
        },
    },
    // ── the rows: `select` and the overlay family ─────────────────────────────
    Row {
        number: 144,
        on_spec_table: false,
        gate: "the family is three kinds on two axes over eight entries, and modality is a `bool` \
               on the request",
        kind: Kind::Count,
        owner: "C07",
        section: "spec §12",
        // **The table is a value a test iterates, not a table in a comment** —
        // `crate::disclose::SPLIT`'s arrangement one family over, and its reason. What makes it two
        // axes rather than one is the *diagonal*: Axis A separates exactly `Kind::Transient` and
        // Axis B exactly `Kind::Dialog`, so neither column is derivable from the other.
        //
        // The narrower question the shell actually branches on is `Kind::has_blur_position`, and it
        // is **not** Axis A: a dialog declares plenty and still has no blur position, because its
        // barrier already withholds the pointer from everything outside it and a modal cannot be
        // dismissed by blur.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: OVERLAY,
                    name: "the_family_is_three_kinds_on_two_axes_and_eight_entries",
                },
                Instrument::Unit {
                    file: OVERLAY,
                    name: "a_transient_declares_nothing_and_a_popups_blur_position_is_not_a_stop",
                },
                Instrument::Unit {
                    file: POPUP,
                    name: "an_overlay_that_declares_and_covers_its_anchor_takes_its_own_hover",
                },
            ],
        },
    },
    Row {
        number: 145,
        on_spec_table: false,
        gate: "a popup's size comes from a sizing function, and one taken from the drawn extent is \
               a fixpoint at zero rows",
        kind: Kind::Equality,
        owner: "C07",
        section: "spec §12",
        // **Two frames and not one**, because the defect is a fixpoint rather than a cold start: the
        // first frame is granted zero because there is no extent, and the second is granted zero
        // because the first drew nothing. One frame cannot tell that apart from a warm-up.
        //
        // The three arms are one field of `crate::input::Sizing`, so what a gate plays is the shipped
        // component with one value changed.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: OVERLAY,
                    name: "the_size_comes_from_a_sizing_function_and_the_drawn_extent_is_a_fixpoint",
                },
                Instrument::Report {
                    file: POPUP_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 146,
        on_spec_table: false,
        gate: "a popup's gutter is decided in 0 passes against §9's <= 3, and a screen too short \
               leaves 0 of 4 rows unreachable",
        kind: Kind::Relation,
        owner: "C07",
        section: "spec §12, §9",
        // **The fixpoint does not arise here and the reason is structural**: a popup's content is as
        // wide as the viewport it was granted, because its labels truncate to it, so the two booleans
        // §9 couples have nothing to couple through. The gate runs the `decide` over the same two
        // numbers so the *0 against <= 3* is a comparison rather than a claim.
        //
        // The unreachable-row half is arithmetic over `place`, `popup_size` and
        // `CollState::max_offset` and says so: no cell of a composited surface is readable from
        // outside the engine, so *the row was drawn where the screen is not* has no
        // observable form at this layer. Its two behavioural halves — the granted height and whether a
        // bar stands — are asserted beside it and both move.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: OVERLAY,
                    name: "the_gutter_is_decided_in_no_passes_and_section_nines_fixpoint_takes_three",
                },
                Instrument::Unit {
                    file: OVERLAY,
                    name: "a_short_screen_leaves_no_row_unreachable_and_the_owner_side_spelling_\
                           leaves_one",
                },
            ],
        },
    },
    Row {
        number: 147,
        on_spec_table: false,
        gate: "one owner is one layer, and a second overlay from one component mints its own id",
        kind: Kind::Count,
        owner: "C07",
        section: "spec §12, §4",
        // **Three regions and not two**, and the extra one is *an `Id` is a hash* as
        // arithmetic: a `select`'s own id **is** its overlay's owner, so a component cannot share a
        // layer without sharing its own identity — the second widget's hit entry is merged away with
        // the second layer. There is no spelling that shares the one and not the other, which is why
        // a second overlay has to mint rather than reuse. `crate::popup::SUBMENU_OWNER` is the
        // shipped instance and `crate::input::CATCHER_KEY` the second.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: OVERLAY,
                    name: "one_owner_is_one_layer_and_the_second_overlay_mints_its_own_id",
                },
                Instrument::Unit {
                    file: POPUP,
                    name: "every_configuration_declares_what_the_specs_table_says",
                },
            ],
        },
    },
    Row {
        number: 148,
        on_spec_table: false,
        gate: "blur is qualified by a position, and the press and catcher spellings each lose \
               exactly one thing",
        kind: Kind::Count,
        owner: "C07",
        section: "spec §12",
        // **Two halves, and each spelling gets one of them wrong** — which is why the gate is not one
        // boolean. The rule keeps a popup the pointer is standing on *and* lets an outside press
        // through; a press-qualified blur dismisses the popup, because `begin`'s optimistic focus
        // arrives a frame before the body can report anything; a catcher keeps the popup and swallows
        // the press it exists to report.
        //
        // The qualification is `Response::local` and not `Response::hovered`, and the word is
        // *position* for exactly that reason: `hovered` is `hover_guess`, resolved from the
        // **previous** frame's index, and the frame that matters is the one the layer was placed on.
        // A blur qualified on `hovered` reads false on that frame and dismisses itself.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: OVERLAY,
                    name: "blur_is_qualified_by_a_position_and_the_other_two_spellings_each_lose_\
                           one_thing",
                },
                Instrument::Unit {
                    file: OVERLAY,
                    name: "an_open_popup_holds_the_keyboard_and_gives_it_back",
                },
                Instrument::Report {
                    file: POPUP_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 149,
        on_spec_table: false,
        gate: "the barrier stops the pointer and only the pointer, and the trap stops the keyboard \
               and only the keyboard",
        kind: Kind::Count,
        owner: "C07",
        section: "spec §12",
        // **Two verbs and two gates, and each arm keeps the other half standing** — which is what
        // turns *they are not one verb* into a measurement. The pointer half plays a press at a widget
        // under a modal that still has its trap; the keyboard half presses six `Tab`s at a modal that
        // still has its barrier. Both refusals are one field of `crate::overlay::defective`.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: OVERLAY,
                    name: "the_barrier_stops_the_pointer_and_only_the_pointer",
                },
                Instrument::Unit {
                    file: POPUP,
                    name: "the_walk_reaches_every_stop_unless_a_trap_is_standing",
                },
            ],
        },
    },
    Row {
        number: 150,
        on_spec_table: false,
        gate: "`&'f mut` makes requesting the overlay anywhere but last a borrow error, and the \
               sentence `Ctx::overlay` owed is written where it is owed",
        kind: Kind::CompileOutcome,
        owner: "C07",
        section: "spec §12, §1",
        // **A pair, and the twin is what holds it to its subject**: rustdoc on stable ignores the
        // error code beside `compile_fail`, so the positive half names `select` and `PopupState` by
        // path and does the same thing in the order that works. The hostile half's real diagnostic was
        // read off the compiler rather than assumed — `E0502: cannot borrow popup as immutable
        // because it is also borrowed as mutable`, pointing at *the caller's own read* and never
        // mentioning the overlay, which is the finding arriving one crate up.
        //
        // The second half is the sentence §1 says `Ctx::overlay`'s documentation owes and components
        // ticket 10 could not write. It is written, on the runtime's own item, and the instrument is a
        // scan of that file: a doc comment is not an item, so nothing a compiler can be asked about
        // changes when it is deleted. The phrases are matched against a *flattened* source, because in
        // the shipped file one of them is broken across a line by the formatter.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Pair {
                    file: INPUT,
                    hostile: "let _ = popup.granted();",
                },
                Instrument::Unit {
                    file: OVERLAY,
                    name: "the_sentence_ctx_overlay_owed_is_written_where_it_is_owed",
                },
            ],
        },
    },
    Row {
        number: 151,
        on_spec_table: false,
        gate: "a body holding a `Copy` of the offset moves it 0 in twenty notches, and `&'f mut` \
               moves it twenty",
        kind: Kind::Count,
        owner: "C07",
        section: "spec §12, §7",
        // **The literal `Copy`-only body, one family over.** The body writes into a value that dies
        // with the frame and the owner hands it the same number again next frame, so the wheel is
        // dead and *nothing else about it moves* — the screen is identical, and the only counter that
        // separates the two arms is the offset itself. Both arms are one field of
        // `crate::input::Holds`.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: OVERLAY,
                    name: "a_copy_only_body_moves_the_offset_nothing_in_twenty_notches",
                },
                Instrument::Report {
                    file: POPUP_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 155,
        on_spec_table: false,
        gate: "a popup dismissed either way can be opened again",
        kind: Kind::Count,
        owner: "C07",
        section: "spec §12",
        // **Found by reading the diff and not by running anything**, which is the half of a review a
        // gate cannot do: every other instrument on this section opens a *fresh* popup, and what the
        // body reports survives the dismissal. A popup shut **by a blur** comes back with *the focus
        // is not inside me* still on record, so the blur clause fires on the frame after the
        // reopening — before the body has had a frame to hand the keyboard over — and the popup is
        // unreopenable by exactly the gesture that dismissed it.
        //
        // The latch is the **owner's** and not the body's, and that is where the fix had to go:
        // `SelectState::open` is the only thing that knows an opening has begun, and an application
        // that opens a popup by its own verb rather than by a keystroke gets the same clearing.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: OVERLAY,
                name: "a_popup_dismissed_either_way_can_be_opened_again",
            }],
        },
    },
    Row {
        number: 154,
        on_spec_table: false,
        gate: "a shut `select`'s face is a partition of its rectangle at every width, truncated or \
               not",
        kind: Kind::Equality,
        owner: "C07",
        section: "spec §2, §12",
        // **§2 over the component this ticket built, and it is here because the diff had the defect
        // it catches.** The label was padded to the whole width and the ellipsis written over the
        // pad's last cell, so one cell of every *truncated* `select` was written twice —
        // `glyphs::elide` has already reserved the marker's cell, which is exactly what makes the
        // mistake easy. Nothing on the screen shows it and no counter but the pair moves.
        //
        // Swept over widths that straddle the label, because a partition asserted only where nothing
        // truncates is a partition that has not been asked the question: writes and distinct agree at
        // every width from 3 to 24, and `verbs` is 4 truncated against 3 whole.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: OVERLAY,
                name: "a_shut_selects_face_is_a_partition_of_its_rectangle_at_every_width",
            }],
        },
    },
    Row {
        number: 153,
        on_spec_table: false,
        gate: "an open popup holds the keyboard and gives it back to its owner on the way out",
        kind: Kind::Count,
        owner: "C07",
        section: "spec §12",
        // **The row this ticket's application earned, and it is the only one here whose subject the
        // gates could not have reached.** Every other instrument on this section drives the popup by
        // posting keys at the *owner*, which works whether or not the popup ever took the keyboard —
        // so a popup with dead arrows passes all of them. What the application showed is a popup that
        // opens, draws correctly, and dismisses itself on the next wake with nothing on screen having
        // gone wrong.
        //
        // The finding underneath it: **`Response::focus_left` on the owner stops being the blur
        // signal the moment the popup takes the keyboard**, because the owner no longer holds the
        // focus and has none to lose. *`focus_left` is what an outside click already produces*
        // is true of the **popup's** id and not of its owner's, and what a blur is from the owner's
        // side is `seated && !inside && !over` — three facts the body reports and none of them a
        // press. `Blur::PositionAlone` is the arm that forgets the middle one.
        //
        // **And the second tab stop in the rig is load-bearing.** *on the way out the owner
        // refocuses itself* was missing from the first build and this gate passed anyway, because
        // with one widget on the screen the **vanish rule** picks the owner by itself. The defect
        // only shows where there is somewhere else to go — which the application has, and where it
        // sent the keyboard to the *other* `select`.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: OVERLAY,
                    name: "an_open_popup_holds_the_keyboard_and_gives_it_back",
                },
                Instrument::Report {
                    file: POPUP_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 152,
        on_spec_table: false,
        gate: "the shell hands its interior over unwritten, and the interior and the bar tile the \
               rectangle at every size",
        kind: Kind::Equality,
        owner: "C07",
        section: "spec §12, §2",
        // **An equality swept over every size a popup can be**, because a partition that holds at one
        // size is an arithmetic coincidence — the argument for the reserved
        // bar, one family over. The `Tally` beside it asserts the second half: the shell writes the
        // bar's column and **not one cell of the interior**, which is what keeps the fill-first
        // defect a caller's mistake rather than the component's.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: OVERLAY,
                    name: "the_interior_and_the_bar_tile_the_rectangle_at_every_size",
                },
                Instrument::Unit {
                    file: POPUP,
                    name: "the_base_pass_is_a_partition_of_the_screen",
                },
            ],
        },
    },
    Row {
        number: 156,
        on_spec_table: false,
        gate: "a picture's ladder is 1 / 2 / 2 where a bar's is 1 / 8 / 8, `Extended == Unicode`, \
               and the whole thing is three constructions",
        kind: Kind::Equality,
        owner: "C15",
        section: "spec §14",
        // **Both directions, and the second one is what makes the first mean anything.** A picture
        // draws the identical surface at the top two rungs; the *mark* ladder — 1 / 4 / 8 bits —
        // draws a different one, so the equality is reading a screen that could have changed rather
        // than one that had stopped. `crate::media::sub_rows` derives the ladder from
        // `crate::chart::raster::geom` instead of branching a second time, which keeps row 26's
        // named exception at one file: a rung that offers a bar eight sub-rows offers a picture the
        // half block, and `COLOURS_PER_CELL` is the ceiling.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PICTURE,
                    name: "extended_equals_unicode_for_a_picture_and_not_for_the_ladder_it_is_not_\
                           drawn_on",
                },
                Instrument::Unit {
                    file: PICTURE,
                    name: "a_picture_at_ascii_is_a_picture_and_the_only_thing_it_loses_is_a_sub_row",
                },
                // **The `constructions: 1..=3`, and the count is a union rather than a product.**
                // Two glyph spellings and two palettes are four combinations and three
                // constructions, because the description does not move with the repertoire — its
                // paints are the theme's and its cells read one pixel each.
                Instrument::Unit {
                    file: MEDIA,
                    name: "a_picture_is_three_constructions_because_the_two_axes_are_not_\
                           independent",
                },
                Instrument::Report {
                    file: MEDIA_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 157,
        on_spec_table: false,
        gate: "a picture spends one `Theme::custom` a cell and a QR spends four, and no two adjacent \
               cells of a photograph share a paint",
        kind: Kind::Count,
        owner: "C15",
        section: "spec §14",
        // **The census `Theme::custom`'s own documentation asks a caller to publish**, and the
        // adjacency count is what turns *no `fill` is available at any size* from an argument into
        // arithmetic: `Ctx::fill` takes one paint for a rectangle, and the largest rectangle of this
        // screen one paint is right for is one cell. The gradient is 3 520 of 47 620 beside the
        // photograph's 0, printed rather than engineered away.
        //
        // **The finding beside it is that the census is not a cost.** The same 24 000 cells drawn
        // from the thirteen roles spend zero customs and take the same frame, so what the number is
        // evidence for is structural — a cell outside the theme is a cell no run can reach.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PICTURE,
                    name: "a_picture_is_twenty_four_thousand_cells_twenty_four_thousand_verbs_and_\
                           no_region",
                },
                Instrument::Unit {
                    file: PICTURE,
                    name: "no_two_adjacent_cells_of_a_photograph_share_a_paint_and_seven_percent_\
                           of_a_gradient_do",
                },
                Instrument::Unit {
                    file: PICTURE,
                    name: "the_screen_calls_no_fill_and_no_rectangle_larger_than_a_cell_has_one_\
                           paint",
                },
                Instrument::Unit {
                    file: PICTURE,
                    name: "a_qr_spends_four_customs_and_they_are_every_paint_on_its_surface",
                },
                Instrument::Unit {
                    file: PICTURE,
                    name: "a_picture_drawn_from_roles_spends_no_customs_and_writes_the_same_cells",
                },
                // **The zero §14 states in the same sentence as the census.** The µs beside it are
                // a timing and a report; the zero is a count, and this is the only window in the
                // workspace with 24 000 style constructions inside it.
                Instrument::Unit {
                    file: "crates/vitui-components/tests/budget.rs",
                    name: "a_steady_frame_of_a_full_screen_picture_allocates_nothing_as_a_total",
                },
            ],
        },
    },
    Row {
        number: 158,
        on_spec_table: false,
        gate: "a picture keeps 0 of 23 920 horizontal distinctions at no colour, and the four tiers \
               are monotone",
        kind: Kind::Count,
        owner: "C15",
        section: "spec §14, §16",
        // **The axis with a floor of nothing, as a count.** Every other component degrades to a
        // worse drawing of itself and this one degrades to a description: 23 920 / 23 899 / 15 347
        // / 0 for a photograph, 20 400 / 473 / 174 / 0 for a gradient.
        //
        // **The instrument was a contrivance and it is now the runtime's own verb**, and the
        // sequence is the whole argument for `Evaluated`. `Theme::custom` said the caller owes a
        // branch on the terminal's capabilities and offered no verb to ask with —
        // `roles_differ_on_wire` compares two of the thirteen **roles**, and a picture's cells are
        // outside them by construction — so `picture::wire_differ` authored a theme per colour
        // pair, putting the two colours on `Role::Danger` and `Role::Warn`, which
        // `Roles::from_palette` takes verbatim from `base08` and `base0A` over one ground with no
        // attributes. **773 ns a call, and it was right.** The friction was filed as runtime
        // architecture issue 34, which answered it with `Theme::colours_differ_on_wire(Rgb, Rgb)` —
        // the same question asked of two colours — and the same counts come back out of it.
        //
        // **The verb alone bought nothing, and that is the part worth recording.** Dropping it into
        // the old body — `Theme::default().resolve(tier).colours_differ_on_wire(..)` — measures
        // **811 ns**, because `Theme::default()` authors a thirteen-role theme and the two-slot
        // palette mutation the rewrite deleted was the free part. The theme is hoisted to a
        // `static` of four, one a tier, and the call is **57 ns**. All three arms are in
        // `picture::wire_differ`'s own documentation, as a report and not a gate.
        //
        // The row was `Evaluated` before the verb existed and that is the point: *it cannot be
        // written from this crate at all* is a claim about the whole space of programs, and this
        // file's row 5 is what a wrong one costs. A probe that were silently always-true or
        // always-false would make every count above whatever the total is, so it is asserted
        // against answers known independently — in both directions, which is the condition for
        // using a contrivance and is what carried over unchanged.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PICTURE,
                    name: "a_picture_keeps_none_of_its_distinctions_at_no_colour_and_the_ladder_is_\
                           monotone",
                },
                Instrument::Unit {
                    file: PICTURE,
                    name: "the_wire_probe_separates_at_truecolor_and_collapses_at_the_floor",
                },
                // **The hoist's own gate**, and it belongs to this row because this row's four
                // numbers are what a wrong one would corrupt. The themes are a literal array and
                // `tier_index` is a match: two derivations of one mapping, and a permutation of
                // them would simply report each tier's count under another tier's name.
                Instrument::Unit {
                    file: PICTURE,
                    name: "every_resolved_theme_is_at_its_own_tiers_index",
                },
                Instrument::Report {
                    file: MEDIA_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 159,
        on_spec_table: false,
        gate: "the module readback catches an inverted pairing, and two modules a cell is exactly \
               twice one at every cell aspect",
        kind: Kind::Equality,
        owner: "C15",
        section: "spec §14",
        // **Two traps and two gates, and the reason they cannot be one is measured.** The inverted
        // pairing writes the same cells with the same four paints one character apart, and 219 of
        // 441 modules come back wrong. One module a cell reads back **perfectly** — 0 of 441 — and
        // is still not a QR, because a module half as tall as it is wide is not a module. The
        // readback is blind to the aspect and the aspect is blind to the pairing.
        //
        // **The readback models the terminal rather than the record**, and the first version did
        // not: decoding a cell's two modules from its paint alone passed the inverted build 0 of
        // 441, because a paint is not a picture — what is on the screen is decided by the paint and
        // the glyph together.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PICTURE,
                    name: "the_readback_catches_the_pairing_and_says_nothing_at_all_about_the_\
                           aspect",
                },
                // **A rectangle is an extent and not only an origin**, and the recorder cannot see
                // the difference: `Pen` records at root coordinates with no clip, so a symbol
                // painted past its rectangle reads back perfectly. `qr_into` stops at the
                // rectangle, and one row short of the symbol is 21 modules unread and none of them
                // written below it.
                Instrument::Unit {
                    file: PICTURE,
                    name: "a_symbol_that_does_not_fit_its_rectangle_is_clipped_rather_than_drawn_\
                           past_it",
                },
            ],
        },
    },
    Row {
        number: 160,
        on_spec_table: false,
        gate: "a picture translated by one whole cell row changes every cell, and a still one \
               changes none after its first frame",
        kind: Kind::Count,
        owner: "C15",
        section: "spec §14, §20",
        // **The reachable half of both of the wire figures**, and it is row 48's question asked of
        // the one screen where the answer is the whole rectangle: 24 000 of 24 000 changed by a
        // translation of one row, and `[24 000, 0, 0, 0]` over a still picture's first four frames.
        // What the same two frames cost in **bytes** is row 161's, and it is reachable since
        // runtime architecture issue 34 — where it inverts this row's own reading: 24 000 of 24 000
        // cells change value and the wire costs **1.25% of a full repaint**, because the engine
        // prices a translation by the rows it exposed.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PICTURE,
                    name: "a_translation_by_one_row_changes_every_cell_and_a_still_picture_changes_\
                           none",
                },
                Instrument::Report {
                    file: MEDIA_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 161,
        on_spec_table: false,
        gate: "bytes on the wire: three zeros after the first frame, a whole-row translation under \
               2% of a full repaint and linear in the rows it exposed, and a strictly poorer wire \
               at each poorer tier",
        kind: Kind::Ratio,
        owner: "C15",
        section: "spec §14",
        // **Inverted by runtime architecture issue 34, and it is the second inversion on this
        // register that no components ticket did.** Row 112 was the first. What the row needed was
        // a driver that hands its sink back: `Driver::headless` moves a `Box::new(Vec::new())` into
        // the engine and never returns it, and `Clock`, `Output`, `Overrides`, `WidthSource` and
        // `InputConfig` were not in `vitui_runtime::line::ENGINE_NAMES` at all, reachable or not —
        // The rule arriving on `Config` itself. All five are re-exported now, so
        // `picture::tapped_driver_for` builds a `Config` with a sink this crate owns **and** a tier
        // pinned on the `Overrides`, which is the half `Driver::set_theme` could never reach: a
        // resolved theme narrows the thirteen roles and every cell of this screen is outside them.
        //
        // **None of the three numbers §14 states is the gate**, and the reason is this workspace's
        // own rule about encodings: a golden byte *string* is refused because the encoding is
        // exactly the part allowed to change, and a byte *count* is one step from a byte string.
        // What is gated is what survives an encoding change — the three zeros, the share, the
        // linearity and the monotonicity — and the totals are reported beside §14's:
        //
        // - **39.05 B/cell** against the 37.5, and **937 233** against its 900 134. The same
        //   measurement on a different photograph.
        // - **11 731 bytes for a whole-row translation** against the 5 885 — and the pair
        //   cannot be reconciled with itself: a 300-cell row at 37.5 B/cell is 11 250, not 5 885.
        //   The measured pair is internally consistent, because 11 731 is one row's own 11 715 plus
        //   sixteen bytes of scroll sequence.
        //
        // **The translation is where the row earns its place beside row 160.** Row 160 says 24 000
        // of 24 000 cells change value; this row says the wire costs 1.2517% of a full repaint,
        // because the engine's scroll pre-pass prices the rows the shift exposed. A repaint priced
        // by the cell — which is what row 160 alone reads like — would have priced it at the screen.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PICTURE,
                    name: "a_still_picture_costs_nothing_and_a_translation_costs_a_row",
                },
                Instrument::Report {
                    file: MEDIA_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 162,
        on_spec_table: false,
        gate: "the picture scene is played over `picture` and `qr` rather than over a stand-in \
               painter",
        kind: Kind::Count,
        owner: "C15",
        section: "spec §14, §21",
        // Criterion 6's join, one family over from row 129's: the scene names its subjects, the
        // subjects are read out of the file the freeze homes them in, and the sentence a reader
        // sees separates *unimplemented* from *wrong*.
        // **Inverted by components 30, and the needle had to move with it.** This is components
        // The finding a second time: the scan read `pub fn picture(` and the shipped
        // declaration is `pub fn picture<P: Pixels>(`, because the `…` is a **type parameter**
        // here — a picture that took a buffer would make the frame cost the image rather than the
        // rectangle. A row green on the parenthesis needle would have been green *by deleting the
        // type parameter*, which is the one thing about that signature that is load-bearing.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PICTURE,
                    name: "the_picture_screen_stands_on_picture_and_qr",
                },
                // And the screen really does draw through them: every figure above is now taken
                // over `crate::media::picture_into` and `crate::media::qr_into`, with the two
                // defect axes routed to `crate::media::defective`.
                Instrument::Unit {
                    file: PICTURE,
                    name: "a_picture_is_twenty_four_thousand_cells_twenty_four_thousand_verbs_and_\
                           no_region",
                },
            ],
        },
    },
    Row {
        number: 163,
        on_spec_table: false,
        gate: "a picture spends one `Theme::custom` a cell, a QR four, a barcode two and the audio \
               three none at all, and the family costs 0 allocations as a total",
        kind: Kind::Count,
        owner: "C15",
        section: "spec §14",
        // **The census as a contrast, which is what makes it a gate rather than four numbers.** §14
        // states the split in the family — *a waveform spends zero because its colours are roles; a
        // QR spends four not because it has too many distinctions but because it has exactly two
        // and they must be those two, which no theme can promise* — and any one of those figures
        // alone is a number with nothing to be surprising against.
        //
        // **The barcode's two is this ticket's own subtraction.** A barcode carries no information
        // across a cell's own height, so two of a QR's four cell states are unreachable — which is
        // `COLOURS_PER_CELL` read from the other end, and the same fact that gives it *runs where a
        // picture has none*.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: MEDIA,
                    name: "the_custom_census_is_one_a_cell_then_four_then_two_then_none",
                },
                Instrument::Unit {
                    file: MEDIA,
                    name: "the_audio_half_climbs_the_bar_ladder_and_adds_no_rung_of_its_own",
                },
                Instrument::Unit {
                    file: PLAYER,
                    name: "the_chrome_spends_no_theme_custom_at_all",
                },
                Instrument::Report {
                    file: MEDIA_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 167,
        on_spec_table: false,
        gate: "every media drawer writes a partition of its rectangle at four sizes, and a QR's \
               remainder is the quiet zone its own specification asks for",
        kind: Kind::Equality,
        owner: "C15",
        section: "spec §2, §14",
        // §2 over a family whose subjects are routinely **smaller than the rectangle they are
        // handed** — a QR at two modules a cell fills eleven of twenty-one rows, a bar pattern is
        // shorter than its band, an audio column is mostly empty. Every one of those cells is the
        // component's, and the arm that would have been easy is a `continue`.
        //
        // **The QR's remainder is the one place on this map where the partition rule and the
        // subject's own standard are the same sentence**: a reader asks for a light margin, so the
        // run that satisfies §2 makes the symbol *more* readable rather than merely tidier.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: MEDIA,
                    name: "every_drawer_writes_each_cell_of_its_rectangle_exactly_once",
                },
                Instrument::Unit {
                    file: MEDIA,
                    name: "the_drawers_declare_nothing_and_the_chrome_declares_something",
                },
            ],
        },
    },
    Row {
        number: 168,
        on_spec_table: false,
        gate: "two pictures and two chromes at two call sites are two ids and 0 merges, and the \
               same call site drawn twice is one",
        kind: Kind::Equality,
        owner: "C15",
        section: "spec §4, §14",
        // **A defect this ticket wrote and this row is why it did not ship.** `Ctx::id` mints from
        // `Location::caller()` and `#[track_caller]` propagates only through functions that carry
        // it, so the first `picture_into` — which took its id inside a plain private body one frame
        // down — gave **every picture in a program the same id**, whatever the call site.
        //
        // Both ways to notice it are quiet: a picture declares no region, so nothing merges and
        // nothing is lost on the screen, and `Response::id` is a value a caller may key on rather
        // than one the runtime checks. `barcode` was correct beside them for the only reason that
        // matters — it takes its id in the frame that carries the attribute — which is what made the
        // diagnosis a comparison rather than a guess.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: MEDIA,
                    name: "two_pictures_at_two_call_sites_are_two_widgets",
                },
                // **And the chrome, where the same defect is not quiet at all.** A review found it
                // left in the sibling file on the one construction in this family that interacts:
                // `Ctx::interact` makes a merged claim **inert**, so a second player's seek bar
                // took no press, no drag and no hover and its transport buttons never clicked, on a
                // screen that rendered perfectly. It is the rule — *a container roots its
                // children inside its own id* — and it needs both halves, because `#[track_caller]`
                // decides *whose* id and `with_id` decides what the keyed children hang from.
                Instrument::Unit {
                    file: PLAYER,
                    name: "two_chromes_at_two_call_sites_are_two_players_and_nothing_merges",
                },
            ],
        },
    },
    Row {
        number: 164,
        on_spec_table: false,
        gate: "a drag is `Response::local` over `Response::rect` and nothing else: press 20/299, \
               move 60/299, release unchanged",
        kind: Kind::Equality,
        owner: "C15",
        section: "spec §14, §17",
        // **The mechanism §17 froze `slider` in Tier 3 for**, measured over the shipped chrome with
        // a **posted** pointer rather than over a `Response` written beside the gate. There is no
        // press origin, no stored anchor and no *was I dragging last frame*, which is the whole
        // claim: a delta-only API cannot express a press that jumps.
        //
        // **The cadence had to be written around and it is the usual two frames.**
        // `Response::pressed` is `frame.grab == id` and the grab is awarded at `end` from the index
        // that has just drawn, so the frame that delivers the `Down` reads `false`. A gate playing
        // one frame a phase would have measured the cadence and called it the mechanism.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PLAYER,
                    name: "a_posted_press_jumps_a_posted_move_carries_and_the_release_moves_nothing",
                },
                Instrument::Unit {
                    file: PLAYER,
                    name: "a_press_jumps_a_move_carries_and_nothing_else_is_read",
                },
            ],
        },
    },
    Row {
        number: 165,
        on_spec_table: false,
        gate: "the player's chrome costs 0 allocations as a total over 200 frames, and the shape \
               it replaced costs 40 — an integer mean of 0",
        kind: Kind::Count,
        owner: "C15",
        section: "spec §14, §20, §21",
        // **refinement 2, with both halves measured in one run.** The defect this is
        // downstream of was `player::chrome` collecting a `Vec<f32>` of chapter positions on the
        // draw path, *found only when the figure was computed as a total rather than an integer
        // mean*. The marks are a field now; `defective::chrome_collecting_into` is the shape they
        // replaced, kept because the ticket says **do not fix it again**.
        //
        // The collected arm allocates on the frames that draw the **chapter list**, which a short
        // screen has no room for — so 40 of 200 frames pay, the total is the defect and `allocs / n`
        // is 0. Both numbers come out of the same run.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: "crates/vitui-components/tests/budget.rs",
                name: "the_chrome_allocates_nothing_and_the_shape_it_replaced_allocates_a_total_no_\
                       mean_would_show",
            }],
        },
    },
    Row {
        number: 166,
        on_spec_table: false,
        gate: "six of the video player's ten chrome parts ship, and the other four each name the \
               mechanism they wait for",
        kind: Kind::Count,
        owner: "C15",
        section: "spec §14, §17",
        // **The survey's ✅ as a table with the column it did not have.** *Transport, seek bar,
        // timeline, volume, subtitles, playlist, chapters: all ✅ cells* is a claim about the
        // engine, and the engine is not what the chrome was waiting for: two of the ten are
        // `slider` (components 33, on the mechanism row 164 measures), one is a component that owns
        // a clock (components 42) and one is survey the passthrough.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PLAYER,
                    name: "seven_parts_ship_and_the_other_three_each_name_what_they_wait_for",
                },
                Instrument::Unit {
                    file: PLAYER,
                    name: "the_chrome_writes_every_cell_of_its_rectangle_exactly_once",
                },
            ],
        },
    },
    Row {
        number: 169,
        on_spec_table: false,
        gate: "the three preview-pane scenes are played over `file_preview_pane` and `file_picker` \
               rather than over a pane written beside them",
        kind: Kind::Count,
        owner: "C16",
        section: "spec §15, §21",
        // Criterion 6's join, one family over from rows 129's and 162's: the scenes name their
        // subjects, the subjects are read out of the file the freeze homes them in, and the
        // sentence a reader sees separates *unimplemented* from *wrong*.
        //
        // **The needle is a parenthesis on purpose**, and the load-bearing half is a second scan.
        // Components 26 read `pub fn select(` for a component §1 already says costs two lifetime
        // annotations and components 30 read `pub fn picture(` for one whose `…` is a type
        // parameter; both would have gone green *by deleting the thing that mattered*. A longer
        // needle here would be this ticket dictating the parameter list. What §15 settles
        // is a **negative** — *a job's lifetime is the question's, a memo's is the data's, and
        // neither is the widget's* — so `crate::preview::mints_its_own_task` is the half that
        // cannot be satisfied by deleting anything, and it is watched in both directions.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PREVIEW,
                    name: "the_screens_stand_on_file_preview_pane_and_file_picker",
                },
                Instrument::Unit {
                    file: PREVIEW,
                    name: "the_subject_scan_finds_a_declaration_when_there_is_one",
                },
                Instrument::Unit {
                    file: PREVIEW,
                    name: "the_task_scan_fires_in_both_directions",
                },
            ],
        },
    },
    Row {
        number: 170,
        on_spec_table: false,
        gate: "a re-sort under a preview pane is wrong on 100 of 100 frames under a position key \
               and 0 under an identity key, at 103 questions either way",
        kind: Kind::Count,
        owner: "C16",
        section: "spec §10, §15",
        // **The sixth arrival of the memo-key rule, and the only one whose trigger is not a
        // gesture.** A preview pane's question is a file and every natural way to name it names a
        // position: one re-sort is one line of application code and no keystroke at all. The
        // counter that separates the arms is `Worker::asked` — 1 against 2 — and every other
        // counter this crate can read is identical, *because the question still matches, so
        // nothing posts, so nothing wakes, so no frame corrects it.*
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PREVIEW,
                    name: "a_position_key_is_wrong_on_a_hundred_of_a_hundred_frames_after_one_re_\
                           sort",
                },
                // And the permutation is a property of the data rather than a stated one: 197 of
                // 200 positions move because the movable positions are rotated by one.
                Instrument::Unit {
                    file: PREVIEW,
                    name: "a_re_sort_moves_a_hundred_and_ninety_seven_of_two_hundred_positions",
                },
                Instrument::Report {
                    file: PREVIEW_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 171,
        on_spec_table: false,
        gate: "a directory streamed in seven batches is wrong after 7 of 7 under a position key \
               and for a run of exactly 1 frame under an identity key",
        kind: Kind::Count,
        owner: "C16",
        section: "spec §15",
        // **The same hole with no user in it**, which is why §21 carries it as a scene of its own.
        // *identity keying is wrong for exactly 1 frame, the decode's latency* is a statement
        // about a **run** and not about a total, and it is asserted as one: seven batches each cost
        // their own latency frame, so a total of one would need six of the seven not to move the
        // cursor's file. 7 wrong frames against 21, longest run 1 against 21.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PREVIEW,
                    name: "seven_batches_move_the_cursors_file_under_it_with_nobody_pressing_\
                           anything",
                },
                Instrument::Report {
                    file: PREVIEW_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 172,
        on_spec_table: false,
        gate: "twenty selections draw 1 picture or 20, and §15's three wire figures are two \
               readings of one product",
        kind: Kind::Count,
        owner: "C16",
        section: "spec §15, R18 §5",
        // **The crossover as a count**, which is the only form of it this crate can reach: row 161
        // says why no crate above the engine can read a byte it wrote. The bytes here are that
        // count times a figure the engine's own map states, and the schedule is a virtual clock —
        // neither arm is branched on, because a question superseded while it runs lands nothing.
        //
        // **The finding is inside the sentence.** 14 652 cells at 37.5 B/cell is 549 450
        // bytes — 536.57 KiB, which §15 prints as *536 KB* by truncating — and twenty of them is
        // 10 989 000 bytes, 10.99 MB, which is exactly what *18.3 MB/s over 600 ms* requires.
        // **10.7 MB** is the truncated 536 read as decimal kB and multiplied by twenty, so
        // the total and the rate printed in one sentence come from two readings of one number. The
        // two that agree are gated and the third is recorded.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PREVIEW,
                    name: "twenty_selections_draw_one_picture_or_twenty_and_the_bytes_peak_with_\
                           the_waste",
                },
                Instrument::Unit {
                    file: PREVIEW,
                    name: "the_pictures_size_and_the_rate_agree_and_the_total_printed_beside_them_\
                           does_not",
                },
                Instrument::Report {
                    file: PREVIEW_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 173,
        on_spec_table: false,
        gate: "each of the five offset spellings produces its own defect on a 4 000-row file, an \
               800-row file and a 74-row viewport",
        kind: Kind::Equality,
        owner: "C16",
        section: "spec §15, §9",
        // **The offset belongs to neither side**, and the row is a table because the five are
        // five different defects rather than five degrees of one: 0 content cells against 2 516;
        // row 726, the new file's last page; the *previous* file, still on screen, scrolled to its
        // top for the length of the decode; right on arrival and 0 on return; and right in both
        // directions.
        //
        // **The headline pair is a screen that did not partition its rectangle.** 1 650
        // writes against 4 166 cannot be reproduced by a screen that obeys §2 — it writes 24 000
        // either way — so this one reports both, and reads one thing *out* of the pair: the
        // difference is 2 516, which is 74 x 34 exactly, so the prototype's document was 34 columns
        // wide and `crate::preview::LINE_COLUMNS` is derived from that rather than chosen.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PREVIEW,
                    name: "each_of_the_five_offset_spellings_is_wrong_in_its_own_way",
                },
                Instrument::Unit {
                    file: PREVIEW,
                    name: "only_the_request_reset_moves_the_file_that_is_still_on_screen",
                },
                Instrument::Unit {
                    file: PREVIEW,
                    name: "every_cell_is_written_once_and_the_counter_that_sees_the_defect_is_the_\
                           content",
                },
                Instrument::Report {
                    file: PREVIEW_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 174,
        on_spec_table: false,
        gate: "a per-file offset map is 14 857 142 bytes at a million entries against one slot's 4",
        kind: Kind::Count,
        owner: "C16",
        section: "spec §15",
        // **R02's sweep releases what an `Id` stopped drawing and a file is not an `Id`**, so the
        // one spelling that is right in both directions is the one nothing releases. The number
        // is reproduced from the arithmetic that produces it — `entries * (8 + 4 + 1) * 8 / 7` —
        // and the shipped `HashMap<u64, u32>` is **35 651 584**, because buckets round to a power
        // of two and the pair pads to sixteen bytes rather than packing to twelve. The estimate is
        // gated because it is §15's; the measurement is asserted to be larger, because a refusal
        // that got easier when it was checked would be worth checking again.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PREVIEW,
                    name: "a_per_file_map_at_a_million_is_fourteen_million_bytes_against_a_slots_\
                           four",
                },
                Instrument::Report {
                    file: PREVIEW_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 175,
        on_spec_table: false,
        gate: "a landing taken inside the draw tears 20 frames of 20 and taken at the top of the \
               view tears none",
        kind: Kind::Count,
        owner: "C16",
        section: "spec §15, R18 §7",
        // **R18 the unenforced ordering, and no thread is involved in it.** `Task::take` is
        // destructive: a view that asks for the answer where it happens to want it takes it in the
        // *first* consumer and leaves the second — three quarters of a screen further down —
        // looking at the value that was there before. A view has no `&mut` to the pane's state, so
        // it cannot put back what it took, and the two status rows of one frame are drawn from two
        // versions of one value. The count is read off the drawn surface, because the claim is that
        // the *screen* disagrees with itself.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PREVIEW,
                    name: "a_landing_taken_inside_the_draw_tears_every_frame_it_lands_on",
                },
                Instrument::Report {
                    file: PREVIEW_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 176,
        on_spec_table: false,
        gate: "an answer to a question nobody asked is dropped without a cell written, and no test \
               on the answer's arrival could see it",
        kind: Kind::Count,
        owner: "C16",
        section: "spec §15",
        // **Eight bytes on the payload, and they are the component's rather than the runtime's.**
        // `Task` already drops a landing whose generation is not the newest — an answer to a
        // question nobody is asking *any more*. This is the other one: an answer to a question
        // nobody ever asked, which arrives with a perfectly current generation because the job was
        // started for the right question and answered a different one. The sentence for why
        // an ordering test cannot reach it: *a question that was never asked is not out of order.*
        // Measured at 1 answered / 1 refused / 0 landings / 0 content cells against 1 / 0 / 1 /
        // 2 516.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: PREVIEW,
                name: "an_answer_to_a_question_nobody_asked_is_dropped_without_a_cell_written",
            }],
        },
    },
    Row {
        number: 177,
        on_spec_table: false,
        gate: "0 decode units run while the app thread is inside its frame, against 1 480 for the \
               same decode called from the view",
        kind: Kind::Count,
        owner: "C16",
        section: "spec §15",
        // **Requirement 9 as a count and not a microsecond.** The counter is thread-local, which is
        // the measurement rather than an implementation detail: the claim is *0 on the app thread*,
        // so it has to be a counter the app thread can read about itself. §15 states the other arm
        // at 5 076 units over a larger document; what is gated is the pair.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: PREVIEW,
                name: "no_decode_unit_runs_while_the_app_thread_is_inside_its_frame",
            }],
        },
    },
    Row {
        number: 178,
        on_spec_table: false,
        gate: "ten tab switches cost 1 spawn, 1 decode and 1 fold against 10 of each, and R02's \
               sweep is refused twice for two different reasons",
        kind: Kind::Count,
        owner: "C16",
        section: "spec §15, R02",
        // ***A job's lifetime is the question's, a memo's is the data's, and neither is the
        // widget's.*** Two refusals over two different things: a pending job swept by the widget
        // registry re-asks on every return, and a derived value swept the same way re-folds. The
        // component's half of it is a signature — the task and the state are parameters — and
        // `crate::preview::mints_its_own_task` is the scan that keeps them there, because it is the
        // half no signature can be *deleted* past.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PREVIEW,
                    name: "ten_tab_switches_cost_one_spawn_and_one_fold_and_not_ten_of_each",
                },
                Instrument::Unit {
                    file: PREVIEW,
                    name: "the_task_scan_fires_in_both_directions",
                },
            ],
        },
    },
    Row {
        number: 179,
        on_spec_table: false,
        gate: "the pane's revision advances on a landing and not on a drop that did not land: 20 \
               folds against 119, on two screens that are cell-identical",
        kind: Kind::Count,
        owner: "C16",
        section: "spec §15, R09",
        // **R09's `Edit` bumps on drop**, and the whole of the defect is a guard taken outside the
        // landing branch: `Versioned::edit` returns a guard whose `Drop` mints a fresh revision, so
        // a pane that takes one before it knows whether anything landed advances on every frame.
        // The 119 is every frame of the run but the first, which is the one frame with no document
        // to fold — and the two arms write the same 2 880 000 cells, which is why nothing that
        // reads a cell can separate them.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: PREVIEW,
                name: "the_revision_advances_on_a_landing_and_not_on_a_drop_that_did_not_land",
            }],
        },
    },
    Row {
        number: 180,
        on_spec_table: false,
        gate: "the steady frame is identical at 1 000, 100 000 and 1 000 000 entries, and a \
               photograph in the pane spends 14 652 customs",
        kind: Kind::Equality,
        owner: "C16",
        section: "spec §15, §20",
        // **`CONTEXT.md`'s invariant through a component that holds an asynchronous answer**: frame
        // cost is proportional to visible cells, never to data volume. The magnitudes — 3 557
        // writes and 197 verbs — are a prototype's screen; this one partitions 300 x 80 at 24 000
        // and 381, and what reproduces is that neither moves.
        //
        // **The photograph's 14 652 is the original's and it is derivable rather than measured**: the
        // pane's viewport is 198 x 74, the photograph is as wide as it, and `crate::media::picture`
        // spends one custom and one verb a cell. That is also what keeps the one-palette rule at
        // two calling lines — a screen that spelled `Theme::custom` itself would be a third
        // palette rather than a second use of the stated exception.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PREVIEW,
                    name: "the_steady_frame_is_the_same_at_a_thousand_a_hundred_thousand_and_a_\
                           million",
                },
                Instrument::Unit {
                    file: PREVIEW,
                    name: "a_photograph_in_the_pane_spends_one_custom_a_cell_of_the_body",
                },
            ],
        },
    },
    Row {
        number: 181,
        on_spec_table: false,
        gate: "`file_picker` is `collection` + `overlay` + the pane and introduces no fourth \
               mechanism: 6 regions decomposing as 1 + 1 + 1 + 3",
        kind: Kind::Equality,
        owner: "C16",
        section: "spec §15, §18 R3",
        // **R3's claim is the class that turns out to be false when it is false**, so it is checked
        // twice and in two different ways. A **source scan** over the picker's own body for the
        // four mechanisms it may not mint — a task, an offset, a selection store, a scrolled scope
        // — and for the three calls it must make; and a **subtraction** over what a standing picker
        // declares, because a count that is merely plausible is not a decomposition. The six are
        // the owner's shut face, the shell's blur position, the collection's one entry however many
        // rows it has, and the pane's three.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: PREVIEW,
                name: "a_file_picker_is_three_components_and_no_fourth_mechanism",
            }],
        },
    },
    Row {
        number: 182,
        on_spec_table: false,
        gate: "`slider` reads `Response::local` over `Response::rect` and nothing else: press \
               20/299, move 60/299, release unchanged, and the component mints no fifth \
               cross-frame fact",
        kind: Kind::Equality,
        owner: "C15",
        section: "spec §14, §17",
        // **Row 164's mechanism wrapped in the component it was measured for**, and the two halves
        // are two kinds of evidence: the three fractions played through the shipped `slider` from a
        // posted pointer, and a **source scan** over its own half of the file for the four things it
        // may not mint — a press origin, a stored anchor, a *was I dragging last frame* and a
        // `SliderState`. The scan is the load-bearing half, because the fractions are also what a
        // component with a stored anchor would produce on the frames it happened to be right on.
        //
        // The needles are assembled from fragments: `crate::frame`'s own first run reported the
        // module it defends, and this scan's first run reported `press` + `_origin` on its own line.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "a_posted_press_jumps_a_posted_move_carries_and_the_release_moves_nothing",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "the_slider_mints_no_press_origin_and_no_drag_phase_field",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "the_grab_is_the_chromes_own_on_the_axis_it_has",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "the_vertical_arm_is_inverted_and_not_transposed",
                },
            ],
        },
    },
    Row {
        number: 183,
        on_spec_table: false,
        gate: "a slider's step is an integer index: fifty `Right`s land on 0.5 and cell 150 of 300, \
               where a `f32` step lands on 0.4999998 and cell 149",
        kind: Kind::Equality,
        owner: "C15",
        section: "spec §14, §17",
        // **The arithmetic finding, and its interest is that the two defects hide each other.** A
        // step added to a `f32` accumulates: at fifty `Right`s the *value* looks right to every
        // digit a status row prints and the thumb is **one cell short of the middle**; at a hundred
        // the thumb is in exactly the right place and the value is **0.99999934**, so a slider
        // driven from the keyboard never reaches its own maximum and a caller testing `v == 1.0` has
        // a branch that cannot be taken. The round trip is 1.4901161e-8 rather than 0.
        //
        // `defective::float_stepped` is a component and not only a function, so a reviewer's diff
        // between the two spellings is one field on one call.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "fifty_rights_land_on_the_middle_and_the_float_step_lands_one_cell_short",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "the_two_stepping_arms_are_one_component_with_one_value_between_them",
                },
            ],
        },
    },
    Row {
        number: 184,
        on_spec_table: false,
        gate: "a slider's cursor pairing is not a list's, and the disagreement with `nav::step` is \
               exactly 4 of the 8 cursor codes",
        kind: Kind::Count,
        owner: "C15",
        section: "spec §3, §14",
        // ***A list's index grows downward and a slider's value grows upward.*** `nav::step` pairs
        // `Up` with `Left` because it moves an index; a slider pairs `Up` with `Right`. So the two
        // agree at `Left`, `Right`, `Home` and `End` and disagree at `Up`, `Down`, `PageUp` and
        // `PageDown` — the finding from the other side, where a container could
        // not read `←` and `→` through that helper because it reads them *as* `↑` and `↓`.
        //
        // **The count is the assertion.** A slider that called `nav::step` would be right for four
        // keys and silently backwards for four, on a screen where the thumb visibly moves either
        // way. Beside it, *a chord types nothing* on a component with no key map: all eight
        // codes at three modifier states and both edges.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "the_sliders_pairing_is_not_a_lists_and_the_disagreement_is_four_codes",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "a_chord_moves_a_slider_nothing_and_a_release_is_not_a_step",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "an_arrow_reaches_a_focused_slider_and_the_press_is_what_focused_it",
                },
            ],
        },
    },
    Row {
        number: 185,
        on_spec_table: false,
        gate: "a slider partitions its whole rectangle at both orientations, a settled one \
               changes 0 cells a frame — the sequence is 40, 1, 0 — and a value from outside the \
               range is reported changed once",
        kind: Kind::Count,
        owner: "C02",
        section: "spec §2, §20",
        // **The two equalities, swept where the arithmetic runs out** — a one-cell track, a
        // two-cell one, the thumb against both ends, and a value that is `NaN` or outside `0..=1`.
        //
        // **The steady half is a sequence and not a total, and that is the finding.** `marked` is
        // the engine's own counter and unreachable here, so the reachable form is row 48's: carry
        // the surface across frames and count the cells whose value changed. `Response::hovered` is
        // resolved from the *previous* frame's index, so the frame the pointer arrives on draws the
        // thumb at rest and the frame after it draws the thumb hovered — **one cell, once**. Read as
        // a single total over 59 frames that is `1` and looks like a defect; it is components ticket
        // 22's warming discipline arriving on a counter rather than on an allocator.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "a_slider_writes_a_partition_of_its_whole_rectangle",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "a_slider_nobody_touches_changes_nothing_after_its_first_two_frames",
                },
                // **And a value from outside the range is reported changed exactly once**, which is
                // the same claim from the caller's side and was wrong first: compared against the
                // *sanitised* value a caller handing in `1.5` sees `changed == false` for ever while
                // the component rewrites the value under it. `NaN` is what makes it load-bearing —
                // `NAN.clamp(0.0, 1.0)` is `NaN` and `NaN != NaN`, so a component that kept a
                // non-finite value would report a change on every frame for ever. Watched firing at
                // `[false, false, false]` against `[true, false, false]`.
                Instrument::Unit {
                    file: INPUT,
                    name: "a_value_from_outside_the_range_is_reported_changed_once_and_then_never",
                },
            ],
        },
    },
    Row {
        number: 186,
        on_spec_table: false,
        gate: "a slider's thumb is on the cell `scroll::thumb` would put it on, at every track \
               length from 1 to 200, and the bar is drawn thumb-first through `scroll::stripe`",
        kind: Kind::Equality,
        owner: "C02",
        section: "spec §3, §9",
        // **Criterion 6, answered with evidence rather than with prose.** The verbs are the
        // scrollbar's own `stripe` — one definition of *write `n` cells along an axis*, one place the
        // orientation branch lives — and the order is `bar`'s: thumb, then the stretches either
        // side. What is *not* reached is `bar` itself, for two reasons that are both counts: a bar
        // has two parts and a slider has three, so `BarOpts`'s single `track` role cannot express
        // the picture; and a `Span` is three counts of **content cells**, which a slider has none
        // of. So the *answer* is shared instead, swept at
        // `Span { viewport: 1, extent: length, offset: at }` over every track a terminal can hold.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "the_thumb_sits_where_scroll_thumb_would_put_it",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "the_three_spellings_are_one_component_and_each_answers_with_a_response",
                },
            ],
        },
    },
    Row {
        number: 187,
        on_spec_table: false,
        gate: "every `built` row of the freeze is declared in the module that homes it: 19 of 19, \
               and 0 of the 10 unbuilt rows are declared",
        kind: Kind::Equality,
        owner: "C11",
        section: "spec §17, ADR 0033",
        // **The row this ticket exists to add, because its own row had been lying for two tickets.**
        // §14 measured drag capture and concluded *`slider` leaves Tier 3*, so components ticket 30
        // set `built: true` and listed the name in `input::MEMBERS`. **No `slider` existed**, and
        // neither join that looks as though it should have seen it could:
        // `the_module_tree_and_the_families_column_agree` compares a module's `MEMBERS` against the
        // `families` **column** and never against the source, and the tier/built gate accepts any
        // row in `MOVED` — which for `slider` recorded something true, because the *mechanism* had
        // been built. **A mechanism being built is not the component being built.**
        //
        // O2 would have caught it eventually — *everything `built` must have a panel* — and it is
        // `Unmet`, which is exactly the shape ADR 0033 exists to refuse: *every obligation this map
        // has stated as a sentence has been broken by someone who had read it.*
        //
        // **The needle is the name and the boundary is either delimiter**, because a join over
        // twenty-nine rows cannot dictate nineteen signatures: `pub fn picture(` could never have
        // matched `pub fn picture<P: Pixels>(`, which is the finding met for the
        // fourth time by ticket 32. Both spellings are watched being accepted, and the reverse
        // direction is counted too — a declared row marked unbuilt is the same drift with the sign
        // flipped, and a gate over the `true` rows alone cannot see it.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: "crates/vitui-components/src/inventory.rs",
                name: "every_built_row_is_declared_in_the_module_that_homes_it",
            }],
        },
    },
    Row {
        number: 188,
        on_spec_table: false,
        gate: "there is no range slider, and the spelling that avoids storing which thumb moves \
               two values in one drag: 0.8 to 0.6 on a thumb nobody touched",
        kind: Kind::Count,
        owner: "C15",
        section: "spec §14, §17",
        // **Criterion 4 answered as a measurement rather than as a preference.** Two thumbs need a
        // third answer neither `Response::local` nor `Response::rect` carries — *which* thumb — and
        // there are two ways to get it. Remembering where the press landed is exactly the **fifth
        // cross-frame fact** the headline is that drag capture does not need. Deriving the
        // boundary from the two values stores nothing and does not work: the boundary moves with the
        // values it separates, so a pointer crossing it mid-gesture **abandons the thumb it was
        // dragging and yanks the other one** — from `(0.2, 0.8)` with the pointer going `0.3 → 0.6`,
        // the high thumb jumps `0.8 → 0.6` in a drag that never released.
        //
        // Both arms are in the same test, because *the stored answer works* is what makes this a
        // cost rather than an impossibility. Two thumbs as two **widgets** is not a third answer:
        // `Response::local` is in the coordinates of the rectangle it was declared with, so a
        // one-cell thumb's own `local` says nothing about the track, and two overlapping regions
        // resolve the pointer to the topmost one.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: INPUT,
                name: "one_drag_of_a_derived_boundary_range_moves_two_values",
            }],
        },
    },
    Row {
        number: 189,
        on_spec_table: false,
        gate: "a slider declares no `Interest::SCROLL`, and two notches over one move its value 0",
        kind: Kind::Count,
        owner: "C15",
        section: "spec §17, §21",
        // **Components the rule on a new subject**, and the reason the absence has to be
        // checked rather than assumed: a widget that declares the wheel and consumes nothing is
        // **worse** than one that declares nothing at all, because it is the topmost region over its
        // rectangle and an enclosing `scroll_area` never sees the notch either. That is the defect
        // ticket 20 found on `field`, which had declared `SCROLL` and consumed it nowhere.
        //
        // The row states the other half — *a slider's value is not an offset and no wheel event
        // moves it* — so `owns_offset` is `false` and there is nothing for a notch to move. Both are
        // read: the declaration off `SliderOpts::default()` and the value off a posted notch.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: INPUT,
                name: "a_slider_declares_no_wheel_and_a_notch_over_it_moves_nothing",
            }],
        },
    },
    Row {
        number: 190,
        on_spec_table: false,
        gate: "three sliders with the keyboard driven cost 0 allocations as a total over 60 frames",
        kind: Kind::Count,
        owner: "C15",
        section: "spec §20, §21",
        // **A total and not an integer mean** — refinement 2, and `crate::media::player`'s
        // defect is why: a chrome that collected a `Vec` on the frames tall enough to draw a chapter
        // list allocated 40 times over 200 frames and reported `allocs / n == 0`.
        //
        // The window **drives the keyboard**, because the arrow is the path a value that formatted
        // itself would allocate on — and that is where this gate found the third instance of one
        // rule: *the window must warm the path it prices*. Warmed by drawing alone it read **1 over
        // 60 frames**, the frame's key queue taking its own first allocation, attributed to the
        // slider. Components ticket 22 met the same discipline on a rectangle that changes every
        // frame and the steady-cell sequence meets it on the hover.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: "crates/vitui-components/tests/budget.rs",
                name: "a_steady_frame_of_a_slider_allocates_nothing_as_a_total",
            }],
        },
    },
    Row {
        number: 191,
        on_spec_table: false,
        gate: "each of the six Tier 2 components reaches the mechanisms it names and mints none \
               beyond them: 6 of 6, watched failing in three directions",
        kind: Kind::Equality,
        owner: "C11",
        section: "spec §17, §18 R3",
        // **Tier 2's own claim, and it is the claim that turns out to be false when it is false.**
        // *Composed of proved mechanisms* is not a quality judgement and a gate over it cannot be
        // one either, so what is checked is the shape: `crate::composed::TIER_TWO` names each
        // component's reaches and its refusals and the scan opens the file.
        //
        // **A scan for absences alone goes green when the section is deleted**, which is why every
        // row carries `uses` beside `mints` and why the negative case has three arms rather than
        // two: a section that is gone has to fail separately from one that is present and wrong.
        // The third terminator — a `defective` module — is load-bearing for the same reason: every
        // one of them in this crate is a deliberate collection of the spellings its component
        // refuses, so a scan that ran into one would report the refusals as the component's own.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: COMPOSED,
                    name: "each_tier_two_component_composes_what_it_says_and_mints_nothing",
                },
                Instrument::Unit {
                    file: COMPOSED,
                    name: "the_survey_is_watched_failing_in_both_directions",
                },
                Instrument::Unit {
                    file: COMPOSED,
                    name: "a_section_stops_before_the_next_banner_and_before_the_refused_spellings",
                },
                Instrument::Unit {
                    file: COMPOSED,
                    name: "every_built_tier_two_row_is_surveyed",
                },
            ],
        },
    },
    Row {
        number: 192,
        on_spec_table: false,
        gate: "`meter` is 2 constructions and the ladder is `chart`'s: 1 / 8 / 8 over three rungs, \
               two values, and the third rung is 0 cells",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §17, ADR 0009",
        // **The derivation, checked rather than transcribed.** Block elements are the
        // *Unicode* rung by `CONTEXT.md`'s definition of it — *Unicode with box drawing and block
        // elements* — so an operator who has promised block elements has promised all of them and
        // there is no third thing for a prefix to build. The count comes off `geom(Kind::Bars, ..)`
        // and not off a table here, which is what *shares `chart`'s prefix ladder rather than
        // reimplementing it* means as a line of code.
        //
        // The two constructions are also drawn: the bottom rung differs and the top two are 0 cells
        // apart, which is `chart`'s own finding arriving on a second component.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INDICATE,
                    name: "a_meter_is_two_constructions_and_the_ladder_is_charts",
                },
                Instrument::Report {
                    file: TIER_TWO_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 193,
        on_spec_table: false,
        gate: "the prefix ladder is shared and the prefix spelling is not: the two eighth-block \
               runs agree at 0 of the 7 partial eighths and at both ends",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §13, §17",
        // **The finding under the `meter`, and it is a fact about Unicode rather than about
        // this crate.** `chart`'s bars grow upward, so its partial cell is one of U+2581…U+2588; a
        // horizontal meter's grows rightward, and the left eighth blocks are a *different*
        // contiguous run — U+258F…U+2588, running the other way. So the vertical arm **reaches**
        // `chart::raster::cluster` and the horizontal arm cannot.
        //
        // **The disagreement is the assertion**, because the failure it protects against is silent:
        // a meter that had transcribed `chart`'s table onto the wrong axis would draw a bar growing
        // upward inside a row, and every counter in this crate would report it correct.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INDICATE,
                    name: "the_two_prefix_runs_are_one_ladder_and_two_spellings",
                },
                Instrument::Report {
                    file: TIER_TWO_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 194,
        on_spec_table: false,
        gate: "every one of the six Tier 2 components writes a partition of its whole rectangle, \
               at both orientations, at every size the arithmetic runs out at, and at an origin \
               that is not the screen's",
        kind: Kind::Equality,
        owner: "C02",
        section: "spec §2, §20",
        // The two equalities on six new subjects, swept where the arithmetic runs out — a one-cell
        // rectangle, a rectangle narrower than a toggle's mark field, a caption wider than the line
        // it sits on, a value that is `NaN` or outside `0..=1`, and a rectangle tall enough that the
        // rows either side of the drawn one are somebody's.
        //
        // **And at `(5, 2)` as well as at the origin**, which is the arm this crate has now had to
        // add three times. `crate::collect`'s `table` drew its header from `x = 0` rather than from
        // the band it was handed and **every gate in the crate passed**, because every one of them
        // played at the origin, where the two agree. `distinct` alone cannot see it either — the
        // same count lands on the same number one column over — so each sweep also asks
        // `Tally::touched` about every cell of the rectangle it named.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "a_toggle_writes_a_partition_of_its_whole_rectangle",
                },
                Instrument::Unit {
                    file: INDICATE,
                    name: "a_meter_writes_a_partition_of_its_whole_rectangle",
                },
                Instrument::Unit {
                    file: INDICATE,
                    name: "a_sparkline_writes_a_partition_and_folds_once_however_much_data_there_is",
                },
                Instrument::Unit {
                    file: STRUCTURE,
                    name: "a_rule_writes_a_partition_of_its_whole_rectangle",
                },
            ],
        },
    },
    Row {
        number: 195,
        on_spec_table: false,
        gate: "a sparkline's raster is its whole rectangle where a chart's is 4 rows of 5, it folds \
               1 time over 20 frames, and its 120 writes are flat at 1k, 100k and 1M points",
        kind: Kind::Relation,
        owner: "C08",
        section: "spec §13, §17",
        // **The measurable half of *no axes, no gutter, no axis loop*.** `chart` at 20x5 reserves a
        // gutter whose width comes from the domain and an axis row underneath, so its raster is
        // smaller than its rectangle on both axes; a sparkline's is the rectangle.
        // `crate::composed` is the other half — a component that had the machinery and did not
        // happen to use it would pass this and fail that.
        //
        // **A relation and not a count, because the verbs are the data's.** `writes` is flat at 120
        // over three orders of magnitude and `verbs` is **16 / 13 / 11**: a verb here is a *run*,
        // and a run ends where a cell's owner changes, which is a property of where the fixture's
        // empty cells fall. §21 says it in as many words — *`verbs <= writes`, never verb equality
        // across sizes* — so the three figures are the report and the relation is the gate.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INDICATE,
                    name: "a_sparkline_is_the_whole_rectangle_and_a_chart_is_not",
                },
                Instrument::Unit {
                    file: INDICATE,
                    name: "a_sparkline_writes_a_partition_and_folds_once_however_much_data_there_is",
                },
                Instrument::Report {
                    file: TIER_TWO_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 196,
        on_spec_table: false,
        gate: "the three toggles demand exactly the freeze's glyphs, and 1 of the 3 states itself \
               with no glyph at any rung",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §16, §17",
        // **§16 on three components at once.** A checkbox and a radio carry their state on the
        // **glyph** axis — `✓` becomes `x` and `•` becomes `*`, both still one cell and both still
        // present, which is what *no spelling blank* buys — and they differ in exactly **one** cell
        // between on and off, at every rung. A switch carries its state on **three** axes and only
        // one of them is the palette: two words, the side its knob sits on, and the face.
        //
        // So the freeze's empty `glyphs` column for `switch` is a fact rather than a hole, and this
        // is the number underneath it. The join runs in both directions: a mark that grew a second
        // glyph without the freeze moving and a freeze that grew a glyph nothing draws are the same
        // drift with the sign flipped.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "the_three_toggles_demand_exactly_what_the_freeze_says_they_do",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "a_switch_states_itself_at_every_rung_and_a_checkbox_needs_its_glyph",
                },
            ],
        },
    },
    Row {
        number: 197,
        on_spec_table: false,
        gate: "a radio set is `collection` at `Mode::Options` — exactly 1 selected at every one of \
               5 gestures — and the standalone row holds no selection store",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §5, §17",
        // **Criterion 6, and the mode's name is a finding rather than a change.** The backlog spells
        // it `Mode::Radio` and §5 shipped it as `Mode::Options` — *exactly one, and it can never
        // become zero*, which is the whole difference from `Mode::Single`. A mode called `Radio`
        // would be the thirteen match arms wearing one component's name, so the disagreement is
        // recorded here and the code is left alone.
        //
        // Two halves, because either alone is satisfiable by the wrong thing: the **set** holds
        // exactly one at every gesture a set can be given, and the **standalone** row's whole
        // cross-frame fact is the caller's `&mut bool` — a source scan over its own section for a
        // `CollState` is what says so, since *there is no selection store here* has no expression.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: INPUT,
                name: "a_radio_set_is_a_collection_and_not_a_second_store",
            }],
        },
    },
    Row {
        number: 198,
        on_spec_table: false,
        gate: "no role variant is a component row: 0 of 18 appear among the freeze's 29",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §18 R2, ADR 0018",
        // **R2's larger half as a gate.** *`primary / secondary / ghost / danger`, link buttons,
        // status LEDs, health pills, dot indicators, badge variants, inline messages, banners,
        // alerts and callouts are an argument, not a component* — and a collapse recorded only in
        // prose is a collapse a later ticket undoes by adding a row.
        //
        // **Spelled as component ids would be spelled**, because that is the form the drift takes:
        // nobody adds a row called *ghost*, they add one called `ghost_button`. Both directions are
        // asserted — the eighteen are absent and the freeze is twenty-nine rather than empty.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: INDICATE,
                name: "no_role_variant_is_a_component_row",
            }],
        },
    },
    Row {
        number: 199,
        on_spec_table: false,
        gate: "a `rule` is `fit`'s four skippable parts with a `Glyph` where the padding was: \
               1 / 2 / 3 / 2 verbs for an empty, a leading, a centred and a trailing caption",
        kind: Kind::Count,
        owner: "C02",
        section: "spec §3, §17",
        // `fit`'s own note is the rule being obeyed: *a label that exactly fills its row is one verb
        // and not three*, because each of the four parts is skipped when it is empty. A rule is that
        // shape with a glyph run in place of each padding run, so the four counts are what say the
        // parts are still four and still skippable.
        //
        // **The spaces around a caption are the caller's**, exactly as `panel`'s title's are, and
        // that is priced rather than preferred: the spelling that spaced them here needed a `String`
        // and allocated **2 a frame, 120 over 60**, which row 200's window caught on its first run.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: STRUCTURE,
                    name: "a_rule_is_fits_four_skippable_parts_and_an_empty_caption_is_one_verb",
                },
                Instrument::Unit {
                    file: STRUCTURE,
                    name: "a_vertical_rule_is_one_column_and_has_no_caption",
                },
                Instrument::Unit {
                    file: STRUCTURE,
                    name: "a_rule_is_a_pure_drawer_and_the_three_spellings_are_one",
                },
            ],
        },
    },
    Row {
        number: 200,
        on_spec_table: false,
        gate: "the six Tier 2 components together allocate 0 as a total over 60 frames, and a \
               settled toggle changes 0 cells after its first frame",
        kind: Kind::Count,
        owner: "C02",
        section: "spec §20, §21",
        // **A total and not an integer mean** — refinement 2, and `crate::media::player`'s
        // defect is why: a chrome that collected a `Vec` on the frames tall enough to draw a chapter
        // list allocated 40 times over 200 frames and reported `allocs / n == 0`.
        //
        // **All six in one window**, because *zero* is the claim for the tier and running each alone
        // lets one hide behind the next one's warm-up — and the window **warms the path it prices**,
        // posting a key on each warm frame because a toggle reads the keyboard and the frame's key
        // queue takes its own first allocation on the first key that reaches it. That is components
        // The discipline met for the fourth time on this map.
        //
        // **It found a real defect on its first run**: `rule` spaced its caption with a `format!`,
        // which is 2 allocations a frame and 120 over 60 — invisible to every other gate here,
        // because the picture is identical either way.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/tests/budget.rs",
                    name: "a_steady_frame_of_the_six_tier_two_components_allocates_nothing_as_a_\
                           total",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "a_toggle_nobody_touches_changes_nothing_after_its_first_frame",
                },
            ],
        },
    },
    Row {
        number: 201,
        on_spec_table: false,
        gate: "a `status_bar` is §9's one band construction with an axis argument, and declares one \
               hit entry however many segments it has",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §9, §17",
        // **The rule is §9's, one component over**: *one hit entry for all four bands, because a
        // band that were a second scroll area would win the wheel from the body it is a header of.*
        // A status bar is the one member of the family that declares anything at all, and what it
        // declares is **one** region — the count does not move with the segment count, and it is
        // never a tab stop.
        //
        // **The axis argument is the band's and not the bar's**, which is the finding rather than
        // the criterion: a bar's content is one row derived from its own segments, so the shared
        // *vertical* offset has nothing to move. `Shares::Y` and `Shares::Neither` draw the same
        // bar at every offset — 0 cells of 30×2 apart at four of them — and only `Shares::X` moves
        // anything, and only at `Fill::Natural`, where the content can be wider than the band.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: STRUCTURE,
                    name: "a_status_bar_declares_one_hit_entry_however_many_segments_it_has",
                },
                Instrument::Unit {
                    file: STRUCTURE,
                    name: "the_two_offsets_a_band_can_share_are_not_two_bars",
                },
                Instrument::Unit {
                    file: STRUCTURE,
                    name: "a_status_bar_is_stickys_construction_and_not_a_copy_of_it",
                },
                Instrument::Unit {
                    file: STRUCTURE,
                    name: "two_status_bars_on_one_screen_are_two_widgets_and_merge_nothing",
                },
                // **And the demand column is joined against what the section can draw.** §17's
                // `glyphs` column is what the within-component collapse gate runs over, so a
                // component that draws a marker its column does not declare is a gate running over
                // less than the component draws — the quiet direction, where nothing fails and the
                // collapse simply is not looked for. It found three rows under-declaring
                // `Ellipsis`, one of them `rule`'s since components ticket 34.
                Instrument::Unit {
                    file: COMPOSED,
                    name: "a_tier_two_row_declares_the_ellipsis_it_can_draw_and_no_row_declares_\
                           one_it_cannot",
                },
            ],
        },
    },
    Row {
        number: 202,
        on_spec_table: false,
        gate: "a status bar writes every visible cell of its band exactly once at every offset, and \
               a segment wider than the band is clipped rather than written beside it",
        kind: Kind::Equality,
        owner: "C02",
        section: "spec §2, §9",
        // **The clip is why a status bar is a band and not arithmetic**, and it is priced one module
        // over: a segment written by arithmetic into the caller's context lands on whatever is
        // beside it, is overdrawn by that neighbour, and re-damages those cells on every steady
        // frame for ever (`crate::scroll::defective::arithmetic_band`).
        //
        // The offset is what makes the partition worth sweeping rather than asserting once: the
        // segments are laid out in the band's **content** coordinates, so a bar scrolled past the
        // end of its own text has cells the segments cannot reach — and those are cells nobody
        // writes unless the trailing run runs to the end of the window. That is components ticket
        // 32's unbounded-extent finding arriving as a run rather than as a hole, and the sweep is
        // watched failing on it: with the tail dropped, a 1×1 band at offset 40 covers 0 of 1.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: STRUCTURE,
                    name: "a_status_bar_writes_every_visible_cell_of_its_band_exactly_once",
                },
                Instrument::Unit {
                    file: STRUCTURE,
                    name: "a_segment_wider_than_its_band_is_clipped_and_not_written_beside_it",
                },
            ],
        },
    },
    Row {
        number: 203,
        on_spec_table: false,
        gate: "a pager and a collection land on the same index for every key in the vocabulary and \
               at every length: no second store, no second navigation model",
        kind: Kind::Equality,
        owner: "C11",
        section: "spec §5, §17, §18",
        // **`pagination` is `collection` at a small length, and the axis is the finding.** `table`
        // is `collection` plus a column split and `tree` is `collection` plus a flatten index, and
        // both of them *call* `collection_into`. A pager cannot: the row loop hands its drawer
        // `Rect::new(0, y, area.w, 1)` and asks `Ctx::visible_rows` which rows are reachable, both
        // of which are the vertical axis by construction. A transpose is not a rectangle split — it
        // is a different `Ctx` — so what a pager reaches is the half of `collection` that has no
        // axis at all: the store, the thirteen arms of `apply`, and the one drain loop.
        //
        // The equality is what says the sharing is real rather than transcribed, and **it was
        // vacuous when it was first written**: the first frame and the drive loop were two call
        // sites, so `Ctx::id` minted two ids, the planted focus named a widget no later frame
        // declared, not one key was delivered, and both arms stood still at zero for all
        // thirty-nine rows. It asserts the ids agree and that more than one index was reached.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_pager_and_a_collection_land_on_the_same_index",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_pager_reaches_the_one_drain_loop_and_mints_no_second_reading_of_it",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_pagers_window_is_the_stores_own_offset_and_follows_only_a_key",
                },
                // The pointer half, which is `from_click` and `from_key` as two readings of one
                // vocabulary: the pager resolves *which column* by arithmetic and hands the answer
                // to the same `apply` the keyboard reaches.
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_click_on_a_page_selects_it_and_a_click_on_a_stepper_steps",
                },
            ],
        },
    },
    Row {
        number: 204,
        on_spec_table: false,
        gate: "a pager writes every cell of its rectangle exactly once and declares one hit entry \
               however many pages it has",
        kind: Kind::Count,
        owner: "C02",
        section: "spec §2, §17",
        // The two equalities over the one component in `crate::collect` whose content does not fill
        // its own rectangle by construction: a strip of `shown` cells of `cell` columns leaves a gap
        // before the trailing stepper whenever the two do not divide, and that gap is the pager's
        // own — `collection`'s tail on the other axis.
        //
        // **A page's label is `Digits` on the stack**, and that is here because the allocation
        // window found the obvious spelling: `(i + 1).to_string()` is one allocation a page a frame
        // and read **9 240 over 60 frames** at 137 pages, with the writes, the verbs, the regions
        // and the picture identical either way.
        //
        // **And the sweep draws the pager two cells inside a larger screen, which is what makes it
        // a sweep at all.** Its first spelling put the strip at the screen's own edge, where the
        // screen clipped every overrun — and the pager was writing **5 cells into a 4-cell strip**
        // and **2 into a 1-cell one**, once because `fits` floors at one page and once because
        // `Ink::pad_to` pads a short label and writes a long one whole. The recorder and the defect
        // shared a coordinate system, which is the third time on this map (components 19's
        // `Tally::distinct`, components 29's `qr_into`).
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_pager_writes_every_cell_of_its_rectangle_exactly_once",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_pager_declares_one_hit_entry_however_many_pages_it_has",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "two_pagers_on_one_screen_are_two_widgets_and_merge_nothing",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_pages_label_is_digits_on_the_stack_and_typing_one_finds_it",
                },
            ],
        },
    },
    Row {
        number: 205,
        on_spec_table: false,
        gate: "`size_of::<FormState>() == size_of::<TypeAhead>()`: a form introduces no mechanism \
               `field`, `nav::cursor` and the ring do not already have",
        kind: Kind::Equality,
        owner: "C11",
        section: "spec §18 R3",
        // **R3 is the class that requires the most care**, in its own words, *because
        // composition without a new mechanism is exactly the claim that turns out to be false when
        // it is false* — so the gate is a `size_of` rather than a reading. `FormState` is exactly
        // the one thing `nav::cursor` cannot borrow from the ring: the type-ahead buffer that has to
        // survive a frame. There is **no cursor** — the cursor is the focus, read back with
        // `Ctx::is_focused` during the draw — no selection, no per-row slot and no geometry.
        //
        // Beside it, the source half: R3 names three things and `FORM_IS` is those three as a value,
        // joined against a scan of the component's own section for the calls that reach them and for
        // the three spellings of a cursor of its own.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "a_form_adds_nothing_to_what_nav_cursor_already_needs",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "a_form_is_the_three_things_r3_says_it_is",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "two_forms_on_one_screen_are_two_sets_of_ids_and_merge_nothing",
                },
            ],
        },
    },
    Row {
        number: 206,
        on_spec_table: false,
        gate: "a grouped form is one tab stop over six ring entries and an ungrouped one is six of \
               six, and only the grouped arm has arrows at all",
        kind: Kind::Count,
        owner: "C02",
        section: "spec §3, §17",
        // Spec §3: *`nav::cursor`'s placement is the decision, not its contents* — a collection
        // opens its own `Group` scope, so *a list is one tab stop*, and a form is the same
        // arrangement over fields.
        //
        // **Both arms are measured because the difference is not cosmetic.** A container receives
        // the keys its children hand back **only** through `Ctx::scope`'s after-the-body moment, and
        // `ScopeKind` has three arms of which the other two are a modal's `Trap` and the code
        // editor's `Isolated`. So *no group* is not *a form without a group scope*: it is a form
        // whose arrow keys reach nothing at all, and the gate presses `Down` on both arms and
        // asserts the focus moves on exactly one of them.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "a_grouped_form_is_one_tab_stop_and_an_ungrouped_one_is_every_field",
                },
                // **The cursor is carried through the drain loop and not read once before it.**
                // `Ctx::next_key` closes the level's queue on a *decline* and not on a take, so a
                // form can be handed several keys in one frame — and two `Down`s then move **one**
                // row, on a screen where nothing else is wrong. Watched failing.
                Instrument::Unit {
                    file: INPUT,
                    name: "two_arrows_in_one_batch_move_two_rows",
                },
                Instrument::Report {
                    file: COMPOSITE_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 207,
        on_spec_table: false,
        gate: "`Compact` against `Cosy` on the same form: 240 writes and 240 distinct either way, \
               four fields standing against two, and two rows of padding is two fields",
        kind: Kind::Count,
        owner: "C02",
        section: "spec §3, §20",
        // Spec the sentence — *`Compact` against `Cosy`: 20 804 writes and 267 regions against
        // 20 992 and 263, four widgets fall off the bottom of the form because the padding is real,
        // **and neither writes a cell twice*** — run against the **component** rather than against
        // `crate::form`'s three-arm screen, which is where ticket 06 measured it.
        //
        // The magnitudes are this screen's and are printed rather than engineered to match: 240
        // cells is a 30×8 panel, and what reproduces is the **structure** — the two densities cover
        // the same screen, neither writes a cell twice, and the count that falls off the bottom is
        // a count of *drawn fields* (the frame's hit entries) rather than an inference from a
        // height. One cell of padding against two, on both edges, is two rows of interior, and two
        // rows of a one-row entry is two fields.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "the_same_form_stands_fewer_fields_at_cosy_and_neither_writes_a_cell_\
                           twice",
                },
                Instrument::Report {
                    file: COMPOSITE_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 208,
        on_spec_table: false,
        gate: "a field declines a cursor key it could not act on, so a container above it is not \
               deaf",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §11, §18 R3",
        // **The defect components ticket 35 found in code that was already green**, and it is
        // *declared and consumed nothing* arriving on the keyboard axis.
        //
        // The one flag makes `input` and `textarea` one component, so `field` reads `Up` and
        // `Down` as a caret row step — and a one-row `input`, which is most fields anybody writes,
        // has no row to step to. It consumed the key anyway. What that costs is not visible on the
        // field at all: a `form` is a `Group` whose `nav::cursor` **never sees an arrow**, on a
        // screen that renders perfectly, and the gate for it reads *the focus did not move* rather
        // than *a cell is wrong*.
        //
        // The answer is the caret's own position rather than a flag on the kind, because the same
        // line is right at the top of a textarea and at the bottom of one.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INPUT,
                    name: "a_grouped_form_is_one_tab_stop_and_an_ungrouped_one_is_every_field",
                },
                Instrument::Unit {
                    file: INPUT,
                    name: "a_field_declines_a_cursor_key_it_could_not_act_on",
                },
            ],
        },
    },
    Row {
        number: 209,
        on_spec_table: false,
        gate: "the three Tier 2 composites allocate 0 as a total over 60 frames, and the \
               record-shaped form allocates 60",
        kind: Kind::Count,
        owner: "C02",
        section: "spec §20, §21",
        // **A total and not an integer mean** — refinement 2, for the fourth time on this map.
        //
        // **The negative case is what this row is for.** `nav::cursor` takes `&[&str]`, so a form
        // written over a slice of *records* — a label and a `Text` in one struct, which reads better
        // and is what a reviewer expects — has to build that slice every frame. The picture, the
        // writes and the verbs are identical either way, and this window is the only instrument in
        // the workspace that can tell the two apart. Measured in one run: shipped 0, record-shaped
        // 60 over 60.
        //
        // **It found a real defect on its first run**, which is the third time a `format!` or a
        // `Vec` on a draw path has been caught here: `pagination` spelled its page labels with
        // `to_string()` and read **9 240 over 60 frames** at 137 pages.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/tests/budget.rs",
                    name: "the_three_tier_two_composites_allocate_nothing_and_the_record_shaped_\
                           form_allocates_a_frame",
                },
                Instrument::Unit {
                    file: COLLECT,
                    name: "a_pages_label_is_digits_on_the_stack_and_typing_one_finds_it",
                },
            ],
        },
    },
    Row {
        number: 210,
        on_spec_table: false,
        gate: "the crate compiles under `#![deny(missing_docs)]` and the pipeline runs the doctests",
        kind: Kind::CompileOutcome,
        owner: "C10",
        section: "spec §17",
        // **O1's other half**, and neither part of it is a thing this process can observe about
        // itself: a test cannot ask what lint level the crate it is linked into was compiled at,
        // and it cannot watch a pipeline. So the instrument reads the two files that decide, which
        // is `crate::line`'s arrangement for exactly this shape.
        //
        // **`deny` and `warn` are the same gate here, measured rather than assumed.** The workspace
        // carries `[workspace.lints.rust] warnings = "deny"`, so an undocumented `pub fn` added to
        // `canvas.rs` failed the build under both spellings with the same diagnostic. `deny` ships
        // anyway, because the two are the same gate only while that table exists — written `warn`,
        // this crate stops being documented the day somebody relaxes a lint table three files up,
        // and nothing here would say so.
        //
        // **There is no sixth CI job, and that is `.gitlab-ci.yml`'s own decision rather than a
        // shortfall.** Its note above the jobs says a sixth means this pipeline alone can saturate
        // a runner shared with every other repo on the machine, and the `test` job's own comment
        // already records that its one serialised invocation *also runs the doctests, and the
        // doctests are where the negative gates live*. A `cargo test --doc` line beside it runs the
        // same seventy-six a second time for a duplicate green.
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: DOC,
                name: "the_crate_denies_missing_docs_and_the_pipeline_runs_the_doctests",
            }],
        },
    },
    Row {
        number: 211,
        on_spec_table: false,
        gate: "every component's page states the hostile axes its freeze row sets, and no others",
        kind: Kind::Equality,
        owner: "C10",
        section: "spec §17",
        // **The equality is the load-bearing half and the count is not.** O1's fifth criterion is
        // *a reader knows which of the four apply before they hit one*, and **thirteen of the
        // twenty-eight built components declare none at all** — so a page that mentions the axes it
        // has is silent on nearly half the freeze, and silence is indistinguishable from a page
        // that forgot. That is `crate::obligations::Verdict::of`'s vacuity refusal arriving on the
        // documentation axis, and the answer is the same: every page says `none` out loud, in the
        // same place, and the gate compares the two sets in **both** directions. A page listing all
        // four fails exactly as loudly as one listing none.
        //
        // The line and not a paragraph, because a scan for a claim inside prose passes on the day
        // somebody writes *it does not narrow*.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DOC,
                    name: "a_page_states_the_axes_its_row_sets_and_no_others",
                },
                Instrument::Unit {
                    file: DOC,
                    name: "a_page_missing_any_one_of_the_four_does_not_hold",
                },
                Instrument::Report { file: DOC_NUMBERS },
            ],
        },
    },
    Row {
        number: 212,
        on_spec_table: false,
        gate: "the six refused spellings each still carry the doc line that says they are refused",
        kind: Kind::Count,
        owner: "C10",
        section: "spec §17, §21",
        // **A refusal is a doc comment, so nothing a compiler can be asked about sees it go.** The
        // item it defends compiles identically the day the paragraph is deleted, no `use` and no
        // `compile_fail` can name it, and the next reader re-derives the argument from scratch and
        // usually gets it wrong. All six were already written — by the ticket that made the refusal,
        // which is the right place and the place nothing was watching.
        //
        // Five carry a `WhyThereIsNo…` item beside the paragraph, which is this crate's shape for a
        // refusal that also has a compile outcome to show. The sixth — the animated fold — has
        // none, and that is not an omission: what it refuses is a *stored third state*, and there
        // is no type to point a `compile_fail` at because the whole claim is that the type does not
        // exist.
        //
        // Watched failing over an empty source, because six needles that all happen to be present
        // is a scan nobody has seen work.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: DOC,
                    name: "every_refused_spelling_says_it_is_refused",
                },
                Instrument::Report { file: DOC_NUMBERS },
            ],
        },
    },
    Row {
        number: 213,
        on_spec_table: false,
        gate: "every construction of every built row has a golden screen, and it is the picture on \
               file",
        kind: Kind::Count,
        owner: "C10",
        section: "spec §17",
        // **The one instrument on this map that catches a wrong cell.** O1 catches an API that
        // cannot be called from outside the crate and O2 an inventory that has drifted from what
        // ships; neither of them looks at a picture, and §17 says so in as many words.
        //
        // **Three sources and not two.** `crate::golden::SCREENS` is what is drawn,
        // `crate::obligations::GOLDENS` is what O3 is asked about, and `crate::golden::on_disk`
        // opens the directory — because neither of the first two would notice a golden that had
        // been deleted, and a count that agrees with itself is what ADR 0033 is against.
        //
        // **`spinner` has no screen and that is the population rather than a hole**, which is O1's
        // finding a second time: a golden of a function that does not exist is not a screen anybody
        // can draw, and asking for one would pin a row nothing on this backlog can invert.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: GOLDEN_TESTS,
                    name: "every_screen_is_the_picture_on_file",
                },
                Instrument::Unit {
                    file: GOLDEN_TESTS,
                    name: "the_three_sources_agree_about_how_many_screens_there_are",
                },
                Instrument::Unit {
                    file: GOLDEN_TESTS,
                    name: "a_scene_name_starts_with_the_row_it_is_evidence_for",
                },
                Instrument::Report {
                    file: GOLDEN_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 214,
        on_spec_table: false,
        gate: "screens declared identical are identical, and the pairs declared different differ by \
               an asserted count",
        kind: Kind::Equality,
        owner: "C10",
        section: "spec §16, §17",
        // **The half that carries the claim.** A component that draws the same construction at two
        // rungs would otherwise file a golden for each — two files that can only ever drift apart —
        // and the count alone would not notice the day a bar chart started spelling itself
        // differently at `Extended`. Three rows are declared identical (`chart`, `meter`,
        // `sparkline`) and `media`'s picture is a fourth with no freeze row at all, which is §14's
        // own *no v1 component*.
        //
        // **The equalities run over `Canvas::diff` and never over `golden::divergence`, and that
        // was a review finding in this ticket's own instrument.** A plane's key is
        // `GLYPH_KEYS[i]` in first-appearance order, so it encodes the *pattern* of distinct
        // clusters and not the clusters: two screens whose non-ASCII cells are swapped one for one
        // have identical planes, and a count over them reports `(0, 0)` for *a bar chart that
        // started spelling itself differently at `Extended`* — which is the one regression the
        // equality half exists to catch. `divergence` is a report about a **file**, where the
        // whole-file line comparison has already caught the legend; the cell truth is
        // `crate::runner::Canvas::diff`, and the blindness is **a number on this row's own
        // subject**: the plot's two block rungs read 12 cells apart over the planes and 21 over the
        // cells. Both are asserted, so the note cannot go stale in either direction.
        //
        // **And the other direction is a number, not a `> 0`.** A count of zero and a count of one
        // are the same inequality, and three separate defects on this map scored *different* by
        // drawing something wrong rather than something else. Two of the three figures do not
        // reproduce and are asserted as measured beside what the map remembers: `plot`'s two block
        // rungs are **12 cells over 3 rows** where §17 says 882, and the dense screen at 300x80 is
        // **1 065 cells over 78 of 80 rows** where §16 says 7 276 over 80 of 80. The row count is
        // why the second one is worth carrying: two rows of that screen carry no glyph at all.
        //
        // **The ASCII rung's own claim, as a picture**: thirty-three of thirty-three screens keep
        // **0** clusters outside printable ASCII, and the dense screen keeps exactly **1** — the em
        // dash of `dense::HEADER`, which is a fixture's title and not a glyph. That is *text
        // is legitimately a component's own content and there is nothing to forbid* as a number.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: GOLDEN_TESTS,
                    name: "a_rung_that_adds_nothing_draws_the_same_screen",
                },
                Instrument::Unit {
                    file: GOLDEN_TESTS,
                    name: "a_picture_at_extended_is_the_picture_at_unicode",
                },
                Instrument::Unit {
                    file: GOLDEN_TESTS,
                    name: "the_rungs_that_change_the_picture_change_it_by_this_much",
                },
                Instrument::Unit {
                    file: GOLDEN_TESTS,
                    name: "the_dense_screen_between_ascii_and_unicode",
                },
                Instrument::Unit {
                    file: GOLDEN_TESTS,
                    name: "no_screen_keeps_a_non_ascii_cluster_at_the_ascii_rung",
                },
                Instrument::Unit {
                    file: GOLDEN,
                    name: "divergence_is_blind_to_a_rename_and_the_canvas_is_not",
                },
                Instrument::Report {
                    file: GOLDEN_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 215,
        on_spec_table: false,
        gate: "the golden format is the engine's, named by path, and a plot at Extended can outgrow \
               its legend",
        kind: Kind::Equality,
        owner: "C10",
        section: "spec §17",
        // **There is no second format and no second bless command**, which is what the ticket asks
        // for and what a path makes checkable. The engine's `golden.rs` is `pub(crate)` throughout
        // — a cell, a handle and a style bit are unreadable from outside the engine — so
        // what cannot be shared is the code; what must not be forked is the format. The four
        // load-bearing sentences are `crate::golden::FORMAT_PROPERTIES`, read out of the owner's own
        // header against a **flattened** source, which is `crate::overlay::OWED_SENTENCE`'s
        // arrangement and its reason: `rustfmt` breaks a phrase across a line.
        //
        // **The ceiling is the finding beside it, and it is the one place on this map where the
        // format decides how big a screen may be.** The legend has twenty-six keys because a
        // twenty-seventh is *not reviewable by eye*, and braille is 256 states a cell — so a `plot`
        // at `Extended` needs **41** keys at 24x6 and **20** at 12x4, which is why its screen is the
        // size it is. The two rungs below it stay inside at 24x6: **16** at Unicode and **0** at
        // Ascii, so the ceiling is braille's and not charting's.
        //
        // **The density is a source scan and there is nothing else it could be.** Density is theme
        // data and it changes rectangles, so a golden taken at another one is a golden of
        // another screen — and the header line has no field for it, because adding one would be the
        // second format this row exists to prevent.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: GOLDEN_TESTS,
                    name: "the_format_is_the_engines_and_the_owner_is_named_by_path",
                },
                Instrument::Unit {
                    file: GOLDEN_TESTS,
                    name: "a_plot_at_extended_can_outgrow_the_legend",
                },
                Instrument::Unit {
                    file: GOLDEN_TESTS,
                    name: "every_screen_is_taken_at_the_default_density",
                },
                Instrument::Unit {
                    file: GOLDEN,
                    name: "the_tier_reads_the_themes_own_two_axes",
                },
                Instrument::Unit {
                    file: GOLDEN,
                    name: "a_failing_golden_reports_the_section_the_column_and_the_totals",
                },
                Instrument::Unit {
                    file: GOLDEN,
                    name: "the_thirteen_role_paints_are_distinct_so_the_legend_names_one_role",
                },
            ],
        },
    },
    Row {
        number: 216,
        on_spec_table: false,
        gate: "every component's bindings are declared data and the machine answers exactly them",
        kind: Kind::Equality,
        owner: "C10",
        section: "spec §17",
        // **The equality O4 states, at the level it means it.** `crate::obligations::o4` compares
        // two lists of *ids*, which is the most the freeze can be asked about and would be
        // satisfied by a component declaring one binding and answering a hundred. This row is the
        // chord-for-chord form, over the thirteen components that read a key.
        //
        // **The two sides come from two places, which is the whole of the row.** One is
        // `crate::contract::CONTRACTS`, written; the other is a sweep of a hundred and sixty-two
        // triggers that **runs the shipped component** and reads `Driver::unhandled` — *the keys
        // this frame's batch carried that nobody took*. A `registered` list derived from the
        // declaration would agree with it for ever, whatever either said about the component, which
        // is `crate::obligations`'s own header one file over.
        //
        // **The control arm is what makes it a component's answer**, and it is a finding as well as
        // a mechanism: the focus walk takes `Tab` and `BackTab` at all five modifier states before
        // any component sees them, so without a subtraction all thirteen contracts register ten
        // chords they have never heard of — and the equality would have been reconciled by
        // *declaring* them. `button` is the control, which is a row of the freeze rather than a
        // fixture: **a tab stop that reads no key**.
        //
        // **It found five chord leaks in code that was already green**, none of them reachable from
        // row 5: the cursor and edit arms of `field`, the owner loops of `select` and
        // `file_picker`, `collect::from_key`'s `Esc` and `Space`, and `input::popup_body`'s own
        // refusal. Every one was a drain loop matching on `k.code` with no modifier guard, and
        // every one types nothing. `Ctrl+Left` is the sharpest, because it is what a user pressing
        // for **word motion** means and word motion is `crate::contract::ABSENT`'s one row — the
        // widget was swallowing the accelerator *and* answering it with a cluster. The fifth is the
        // one with nowhere else to go: `popup_body`'s loop runs *inside* a trapless overlay, so the
        // application has no other reader for the key it just lost.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: CONTRACT,
                    name: "documented_equals_registered_for_every_component",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "a_contract_wrong_in_both_directions_is_caught_in_both_directions",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "the_control_is_a_component_of_the_freeze_that_reads_no_key",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "the_sweep_and_every_answer_are_the_size_they_are",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "the_written_lists_and_the_sweep_agree",
                },
                Instrument::Report {
                    file: CONTRACT_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 217,
        on_spec_table: false,
        gate: "the help is rendered from the declaration through the runtime's own renderer, and \
               there is no second list",
        kind: Kind::Equality,
        owner: "C10",
        section: "spec §17",
        // **R12's `Binding` carries a help string precisely so that bindings are declared once and
        // read twice**, and the second reading is this. `crate::contract::help` builds the
        // runtime's `KeyMap` out of a contract's own binds and prints each through
        // `vitui_runtime::keys::write_help`; the class and the click borrow the same `write_chord`
        // for their spelling, so what a help bar prints and what the sweep reports are one function
        // of one value.
        //
        // **`Bind::ignores` is the modelling decision underneath it**, and it is the rule
        // rather than a convenience: `crate::keys::SIGNIFICANT` is `CTRL | ALT` and Shift is
        // deliberately not in it, so a component that filters through `is_chord` and then matches
        // on `code` answers `Shift+X` exactly as it answers `X`. The sweep finds two chords where a
        // help bar should print one line, and the field is that fact declared. It is not a licence:
        // `collection` declares `Up` and `Shift+Up` as two binds, because there the second is a
        // different action.
        //
        // **And what is absent is absent, with the fact that makes it so.** Word motion is
        // `ABSENT`'s one row, and the gate reads both halves — no help line offers it, and the
        // engine still exports no word iterator. *Missing* and *not yet done* are the same thing to
        // a reader.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: CONTRACT,
                    name: "the_runtimes_renderer_and_the_spelling_agree",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "a_pointer_gesture_is_spelled_by_the_same_function_a_chord_is",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "the_absent_binding_is_absent_from_every_help_line",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "an_action_is_declared_once_within_a_contract",
                },
                Instrument::Report {
                    file: CONTRACT_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 218,
        on_spec_table: false,
        gate: "a component's contract is read at the configuration where it is whole, and what the \
               other one removes is a number",
        kind: Kind::Count,
        owner: "C10",
        section: "spec §17, §21",
        // **Two components on this map have a contract that moves with their options**, and a sweep
        // that read either at the wrong one would report the component as broken rather than as
        // configured.
        //
        // `field` is one flag — the break rule — and at `WrapKind::Ruler` it declines `Up`,
        // `Down` and `Enter`, which is ADR 0042 as six spellings. `collection` is `Mode`, and the
        // finding there is the direction: at `Mode::Single` a ctrl-click *is* a plain click, so it
        // leaves the same picture and reads **deaf**, while the two extends the mode refuses leave
        // a different one and read **fluent**. The gesture that disappears is the one that works.
        //
        // **The three collections declare one contract and the pager declares its keys alone**,
        // which is *a table is a collection plus a column split* and *a tree is a
        // collection plus a flatten index* as an equality between three declarations, with
        // components 35's store-without-a-row-loop as the difference: four binds, the type-ahead
        // and the three pointer gestures.
        //
        // **And the overlay family's two owners declare one contract, since components architecture
        // issue 23.** They did not for eight tickets and that was the defect: `select`'s popup takes
        // the keyboard from its owner and reads `Enter` and `Esc` through a `Refusal`, and
        // `file_picker`'s seated no focus and declared none, so **an open picker could only be used
        // with a mouse** — 35 against 8 over one family, on a screen that rendered perfectly. The
        // two now share one `&[Bind]`, so the declaration's equality is the type system's; what this
        // row still asserts is that the two **bodies** answer the same thirty-five spellings, and
        // `PICKER_IS_MISSING` is kept at **0** as the number that records the repair.
        //
        // And the text class is a class, asked of three letters rather than of one — with a
        // collection's own refusal beside it, because `seek` declines a letter no row starts with.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: CONTRACT,
                    name: "the_break_rule_decides_three_of_a_fields_binds",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "the_mode_decides_the_pointer_half_and_hides_the_gesture_that_works",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "the_three_collections_declare_the_pager_plus_the_listing",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "the_three_toggles_declare_one_contract",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "the_two_overlay_owners_declare_one_contract",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "every_seeking_line_states_its_deadline_and_the_typing_one_has_none",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "the_text_class_is_a_class",
                },
                Instrument::Unit {
                    file: CONTRACT,
                    name: "a_collections_type_ahead_declines_a_letter_no_row_starts_with",
                },
                Instrument::Report {
                    file: CONTRACT_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 219,
        on_spec_table: false,
        gate: "O2's two equalities — nothing shown is absent from the freeze, and everything \
               `built` has a panel",
        kind: Kind::Equality,
        owner: "C10, C11",
        section: "spec §17",
        // **The fourth of the six obligation queries to turn**, and filling
        // `crate::obligations::PANELS` is the whole of what it took — which is what *the evidence is
        // an argument, not a file read* was for: neither query changed.
        //
        // **Two lists and not one read twice.** `PANELS` is written out and `crate::gallery::PANELS`
        // is the table the screen is drawn from; `panel_ids` derives the second from the second, and
        // the equality is between a claim and a thing that draws. That is `crate::doc`'s arrangement
        // for O1 and `crate::contract`'s for O4, for their reason.
        //
        // **The direction neither equality can see is the application's**, so there is a negative
        // scan beside them: `crates/vitui-apps/examples/gallery.rs` may not spell `Panel {`, a
        // second `PANELS`, or any component's own call. An application free to list its own panels
        // drifts from the freeze where no equality over the table is looking.
        //
        // **And the direction neither of the three can see is an empty body**, so the third
        // instrument photographs each panel alone and counts cells **inside the interior**:
        // `panel_into` writes the frame and the title whatever the body does, so a count over the
        // tile is a count of the gallery's own drawing.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: GALLERY,
                    name: "the_written_list_and_the_table_agree",
                },
                Instrument::Unit {
                    file: GALLERY,
                    name: "both_halves_of_o2_are_met_over_the_twenty_nine_built_rows",
                },
                Instrument::Unit {
                    file: GALLERY,
                    name: "every_panel_names_a_built_row_and_the_order_is_the_freezes",
                },
                Instrument::Unit {
                    file: GALLERY,
                    name: "every_panel_draws_at_least_one_cell_of_its_own_interior",
                },
                Instrument::Unit {
                    file: GALLERY,
                    name: "the_application_draws_the_table_and_mints_no_panel_of_its_own",
                },
                Instrument::Unit {
                    file: OBLIGATIONS,
                    name: "all_nine_obligation_queries_are_met_and_o5_was_the_last_to_turn",
                },
                Instrument::Unit {
                    file: OBLIGATIONS,
                    name: "o2_nothing_shown_fails_loudly",
                },
                Instrument::Unit {
                    file: OBLIGATIONS,
                    name: "o2_everything_built_fails_loudly",
                },
            ],
        },
    },
    Row {
        number: 220,
        on_spec_table: false,
        gate: "the gallery's tiles tile its grid exactly, and every tile is its own widget",
        kind: Kind::Count,
        owner: "C01, C04",
        section: "spec §2, §4",
        // **Two rules of this map meeting on the one screen where twenty-eight components do.**
        //
        // The partition half is arithmetic swept from 20x8 to 119x29 rather than asserted at one
        // size, because the spelling that fails is the one that is right wherever the width happens
        // to divide: `defective::tiles_by_multiplication` puts tile `k` at `k * (w / n)` and leaves
        // the columns past `n * (w / n)` to nobody. Watched losing the last column at 100 into three
        // and watched **agreeing** at 99 into three, which is why the sweep is a sweep.
        //
        // The identity half is ADR 0027 and components 30's `player::chrome` defect in the loop that
        // would have produced it: one call to `panel_into` is one `Location::caller()`, so
        // twenty-eight tiles from one loop are twenty-eight widgets under one `Id` — and
        // `Ctx::interact` makes a merged claim **inert**, so every panel but the first stops hearing
        // the pointer on a screen that renders perfectly. `Ctx::with_key` around each tile is the
        // fix and `defective::tiles_under_one_id` is watched merging two of three.
        //
        // **The clear-once rule is here rather than in a fourth row**, because it is the same
        // sentence about the same rectangle from the other side: a screen whose gaps are never
        // painted is not the same screen, and `crate::app::Clears` is a value the caller owns.
        //
        // **And it grew a second trigger, because a resize is not the only thing that decides every
        // cell of a screen.** This one pages twenty-eight panels through twelve tiles, so `Ctrl+N`
        // puts a different component in the same rectangle — and with the second half unmet on the
        // screen (525 cells of 3 000 at 100x30 written by nobody) it left **286 cells over 12 rows**
        // of the previous page standing inside the new page's frames: a `radio` panel with a
        // `collection`'s rows in it. `Clears::relaid_into` takes the caller's own key beside the
        // size.
        //
        // **Every gate here was green while that shipped, and the reason is the instrument rather
        // than the gate.** `gallery::swap` rendered the *after* picture onto a **fresh** `Pen`, and a
        // recorder that starts blank cannot see residue — which is `crate::golden`'s own note about
        // its multi-frame surface, in as many words. The equality that catches it is *a carried
        // surface after a change equals a fresh surface of what it changed to*, and both arms are
        // watched failing on the spelling that shipped.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: GALLERY,
                    name: "the_tiles_tile_the_grid_exactly_at_every_size",
                },
                Instrument::Unit {
                    file: GALLERY,
                    name: "tiles_by_multiplication_leave_the_remainder_to_nobody",
                },
                Instrument::Unit {
                    file: GALLERY,
                    name: "no_tile_shares_an_id_with_another",
                },
                Instrument::Unit {
                    file: GALLERY,
                    name: "the_screen_clears_once_and_again_when_its_layout_or_its_theme_moves",
                },
                Instrument::Unit {
                    file: GALLERY,
                    name: "a_carried_surface_after_a_change_equals_a_fresh_one",
                },
                Instrument::Report {
                    file: GALLERY_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 221,
        on_spec_table: false,
        gate: "the gallery gives the terminal back **before** a panic prints, under a real pty",
        kind: Kind::Invariant,
        owner: "C11",
        section: "spec §21",
        // **The mechanism is the engine's and what this row adds is that it happens in this
        // binary.** `crates/vitui-engine/src/shutdown.rs` restores first and then lets the default
        // hook print, idempotent under one atomic; engine ticket 22 built it and gated it over a
        // recorder. Two ways for an application to lose it are invisible to any in-process test: the
        // process never goes through `Screen::drop` or the hook at all, and the epilogue is written
        // *after* the backtrace — which puts the backtrace on a page the terminal is about to
        // discard, at the one moment a developer needs it.
        //
        // **So the gate is an order on captured bytes from a process that really panicked**, and it
        // is the one instrument on this register that is not a `cargo` invocation: a `cargo test`
        // has no terminal to leave in a state. `scripts/gallery-panic-gate.sh` takes a pty with
        // `script(1)` — whose two spellings are not compatible, which is why it probes for both —
        // and compares two byte offsets. It runs in the `idle` CI job, beside the two gates that are
        // already shell.
        //
        // **A presence is not the gate**, and the unit instrument beside the script is what says so:
        // it reads the script and asserts that it compares offsets rather than greps for the
        // epilogue. A capture where the backtrace came first contains both strings.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: GALLERY,
                    name: "the_panic_gate_runs_this_binary_and_checks_an_order",
                },
                Instrument::Report { file: PANIC_GATE },
            ],
        },
    },
    Row {
        number: 222,
        on_spec_table: false,
        gate: "zero allocations on a steady frame of the assembled gallery, as a total",
        kind: Kind::Count,
        owner: "C01, C11",
        section: "spec §20",
        // **The budget measured in the gallery and not only in isolated harnesses**, which is §20's
        // own criterion and the one thing twenty-seven other allocation gates in this workspace
        // cannot answer between them: each prices one component.
        //
        // **Every page**, because the six panels that keep a memo are not all on page one and a
        // window over one page prices twelve of the twenty-eight. The warm-up is two identical
        // frames on the shape the window prices — components 22 measured what warming on the wrong
        // shape costs, **1 over 12**, which is amortised zero and exactly what `Allocations`'s
        // missing `mean` refuses.
        //
        // **It found one, and it is components 30's `player::chrome` finding a second time**: the two
        // preview drawers spelled their row labels with `format!`, so page three paid **4
        // allocations a frame, 200 over 50**, while every other page read zero and the screen
        // rendered perfectly. `gallery::ROWS` is eight literals now. The gallery's own chrome is
        // two reused `String`s rewritten with `write!`, so the total covers the whole frame rather
        // than only the tiles.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: BUDGET_TESTS,
                    name: "a_steady_frame_of_the_gallery_allocates_nothing_as_a_total",
                },
                Instrument::Report {
                    file: GALLERY_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 223,
        on_spec_table: false,
        gate: "the keyboard alone reaches every panel — the pages partition the table at every size",
        kind: Kind::Count,
        owner: "C10, C11",
        section: "spec §17, §21",
        // **A count and not a walk**, because *the walk keys cannot be printable characters* is O4's
        // finding rather than this ticket's: a focused `field` consumes every text-bearing key and a
        // focused `collection` eats one into its type-ahead buffer, so `Ctrl+N`/`Ctrl+P` are chords
        // and `Bind::ignores` is why a chord passes through. This gallery has a `field`, a `form`,
        // three collections and a picker on it, all one `Tab` away.
        //
        // What is asserted is that the pages **partition** the twenty-eight in the freeze's order at
        // five sizes, that the walk is a cycle, and that a named panel is one `go_to` away — which
        // is what makes the screen navigable by somebody who knows what they are looking for. The
        // walk itself is register rows 34 and 88 and was already green; it is cited there rather
        // than rebuilt.
        //
        // **A resize is the one event that changes the page count without going through a key**, and
        // it is the case a per-size sweep over fresh galleries cannot reach: `next_page` wraps
        // against the size it is handed, `pages()` shrinks when the terminal grows, and an index past
        // the end yields an empty range rather than a clamp — a title and a status bar reading *0
        // panels · page 4/1* over a grid nobody writes, standing until the next keystroke.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: GALLERY,
                    name: "the_pages_reach_every_panel_at_every_size",
                },
                Instrument::Unit {
                    file: GALLERY,
                    name: "every_panel_can_be_gone_to_by_name",
                },
                Instrument::Unit {
                    file: GALLERY,
                    name: "no_size_panics_and_every_key_is_pressable_at_every_size",
                },
                Instrument::Unit {
                    file: GALLERY,
                    name: "a_resize_that_shrinks_the_page_count_does_not_blank_the_screen",
                },
            ],
        },
    },
    Row {
        number: 224,
        on_spec_table: false,
        gate: "every memo in this crate carries the theme in its key iff its value is made of \
               paints or glyphs",
        kind: Kind::Equality,
        owner: "C11 / R20 §3",
        section: "spec §16, ADR 0030, ADR 0032",
        // **The rule as a population rather than as a sentence**, which is components ticket 41's
        // own criterion: *a gate enumerates them from `INVENTORY` rather than from a grep*.
        // `crate::memos::MEMOS` is five rows, `held_by` is the join, and the rule is checked as the
        // `iff` it is — a memo carrying the theme in the key of a value the theme has no say in is
        // as much a defect as one that leaves it out, because it is a value rebuilt on every `t` for
        // nothing.
        //
        // **The finding is that the rule is true here vacuously**, and it is the reason this row is
        // not the gate row 8 rests on. Three shipped memos: a sub-cell bit grid, an axis domain and
        // a wrap index — bits, floats and byte offsets — and every cluster and every paint on every
        // panel is derived from the theme in front of the frame that draws it. *An equality between
        // two things that do not exist holds*, which is `Verdict::of`'s vacuity failure in the shape
        // this map keeps meeting, and the answer is the same one O4 gave: build the subject. The one
        // row the rule can be false about is `gallery::Panels`, and it is off unless a gate turns it
        // on.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: MEMOS,
                    name: "every_memo_carries_the_theme_iff_its_value_is_made_of_paints_or_glyphs",
                },
                Instrument::Unit {
                    file: MEMOS,
                    name: "no_shipped_memo_in_this_crate_holds_a_paint_or_a_cluster",
                },
                Instrument::Unit {
                    file: MEMOS,
                    name: "the_census_joins_to_the_freeze_in_both_directions",
                },
            ],
        },
    },
    Row {
        number: 225,
        on_spec_table: false,
        gate: "the census names every memo in the source, and the scan that checks it recurses",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §10, ADR 0030",
        // **A census nobody compares against the source is a list that decays**, so the grep is kept
        // — as the completeness check rather than as the enumeration.
        //
        // **And it found a scan that could not see two thirds of the crate.** `crate::order`'s own
        // claim was *no memo in this crate's library half takes a bare `Revision`*, checked with one
        // non-recursive `read_dir` over `src/` — and `src/chart/raster.rs` builds two
        // `vitui_runtime::Memo`s. The scan had been green beside two uses of the spelling it forbids
        // for as long as `PlotState` has existed. It is `crate::inventory`'s own recorded defect
        // (*the walk was one `read_dir` over `src/`, so a component in a subdirectory was never
        // scanned*) arriving a second time in a different file, and the repair is both halves: the
        // recursion, and the claim restated to what is true — `Memo::get` takes one `Revision`, so a
        // compound key is a **fold** into eight bytes, which is the runtime's own documented door,
        // and `Keyed` is for the key that cannot be folded.
        //
        // **Two of the five are invisible to any needle**, which is why the enumeration is a value:
        // a hand-rolled memo is a field and a comparison and has no spelling to search for.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: MEMOS,
                    name: "the_census_names_every_memo_in_the_source",
                },
                Instrument::Unit {
                    file: MEMOS,
                    name: "two_of_the_five_are_invisible_to_any_needle",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/order.rs",
                    name: "every_memo_spells_its_key_and_records_what_it_was_built_at",
                },
            ],
        },
    },
    Row {
        number: 226,
        on_spec_table: false,
        gate: "a theme swap allocates nothing, and it happens between two frames",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §16, §20",
        // **§16 states it as a timing and §21 makes a timing a report**, so what is gated is the
        // half a count can carry: *off the frame path*. `Gallery::theme` returns a theme and
        // `Driver::set_theme` is the loop's, so the whole of a swap — the import, the resolve, the
        // re-narrowing of the ten distinction bits — happens between two frames, and the window is
        // drawn round exactly that. **0 over 168 presses**, which wraps all fourteen schemes, all
        // three repertoires and all four depths.
        //
        // **The window is round the press and not round the frame, and that is a finding.** A window
        // covering the frame cannot be zero on this instrument: `Driver::headless` moves a `Vec<u8>`
        // into the engine and never drains it, so a themed frame — a full-screen repaint — feeds a
        // buffer that only grows. Measured over 6 000 presses at 100x30 it reallocates at **92, 275,
        // 641, 1 372, 2 836 and 5 762**, exact doubling, inside `vitui_engine::engine::write_frame`;
        // at 300x80 the same 300 presses allocate nothing, because the buffer is eight times larger
        // before the window opens. Runtime the sentence arriving as a
        // number, and an instrument's property rather than an application's — a real screen's output
        // is a `Stdout` and the bytes leave.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/tests/budget.rs",
                    name: "the_theme_swap_itself_allocates_nothing",
                },
                Instrument::Unit {
                    file: GALLERY,
                    name: "the_screen_clears_once_and_again_when_its_layout_or_its_theme_moves",
                },
                Instrument::Report {
                    file: GALLERY_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 227,
        on_spec_table: false,
        gate: "every component that takes a data volume is inside a growth relation and a \
               per-input ceiling at a million inputs",
        kind: Kind::Relation,
        owner: "C11",
        section: "spec §17, §20, §21",
        // **The hole this closes, stated once: this crate can prove a frame's *output* is flat in
        // the data volume and it could not prove its *work* was.** `chart`'s rasteriser painted the
        // whole column prefix for every point — 952.61 ms against 2.72, 476.3 ns a point against
        // 1.4 — and every gate here was green on it. The round trip compares a replayed screen
        // against the frame that produced it and both spellings produce the identical frame; the
        // flat-in-n gates are all **output** counters; `Raster::touched` was right at 2 000 000 both
        // ways, because the cost was `O(subh)` *inside* a visit it counts as one.
        //
        // **Two numbers, and neither implies the other.** A growth relation `work(n) / work(n / 10)
        // <= 12`, and a ceiling `work(n) <= fixed + per_input * n`. The relation alone reads the
        // defect that shipped as healthy, because `O(subh)` a point is linear; the ceiling alone is
        // met by any constant chosen large enough, which is the first refinement by name.
        //
        // **Both are counts and neither is a clock**, which is the runtime's scene 19 one crate
        // down: a step count is the same number on every machine, where the microseconds are three
        // orders apart between a debug binary and a release one. `examples/volume_numbers.rs` is
        // where the clock and its 16.7 ms denominator live, as a report.
        //
        // **A virtualised row's ceiling has no `n` in it at all**, which is `Layer::L2`'s own
        // sentence — *it costs its visible window and never the data volume* — as arithmetic rather
        // than as a rounding of it.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: VOLUME,
                    name: "every_covered_row_holds_both_bounds_at_a_million",
                },
                Instrument::Unit {
                    file: VOLUME,
                    name: "the_seven_shipped_arms_cost_what_they_cost",
                },
                Instrument::Unit {
                    file: VOLUME,
                    name: "a_virtualised_rows_ceiling_has_no_n_in_it_and_a_folding_rows_does",
                },
                Instrument::Report {
                    file: VOLUME_NUMBERS,
                },
            ],
        },
    },
    Row {
        number: 228,
        on_spec_table: false,
        gate: "every covered row is watched failing on a deliberate superlinearity, and each half \
               of the pair is watched failing separately",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §21",
        // **A gate nobody has watched fail is not a gate**, and for O6 that is seven arms rather
        // than one: *every other covered component owes an equivalent deliberate superlinearity, or
        // the gate is a claim about a defect nobody has expressed*.
        //
        // Six of the seven are *the instrument separates a correct build from a defective
        // one*; `chart`'s is the defect that actually shipped, kept runnable since the reduce.
        //
        // **Building the four row-shape arms found that `whole_content` existed for one caller of
        // three.** `table` and `tree` *are* `collection` plus a rectangle split and a flatten index,
        // so they inherit **the single most expensive mistake available above this runtime** — and
        // neither could be asked to make it, because `table_with` and `tree_with` both called
        // `collection_into`, which hardcodes the shape. `collect::defective::table_whole_content`
        // and `tree_whole_content` are that arm on the other two, one field each.
        //
        // **And the pair is watched failing from both ends**, which is what makes it a pair rather
        // than a belt and braces: `naive_bars` is *inside* the relation at ten a decade and fails
        // only the ceiling, and `domain_per_point` is inside a ceiling raised until it admits the
        // arm and fails only the relation, at a hundred a decade.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: VOLUME,
                    name: "every_covered_row_is_watched_failing_on_a_deliberate_defect",
                },
                Instrument::Unit {
                    file: VOLUME,
                    name: "neither_criterion_would_do_on_its_own",
                },
                Instrument::Unit {
                    file: VOLUME,
                    name: "an_arm_that_did_no_work_does_not_hold",
                },
            ],
        },
    },
    Row {
        number: 229,
        on_spec_table: false,
        gate: "O6's population is derived from the freeze, and the derivation is what says which \
               components it covers",
        kind: Kind::Equality,
        owner: "C11",
        section: "spec §17, ADR 0033",
        // **A query over `INVENTORY`, not a hand-written list** — the rule on the one
        // obligation whose population is not `built`. *Rows that take a volume* is
        // `Layer::L2`, which is already the column that says **it costs its visible window and
        // never the data volume**, plus the rows that fold on the edit — which
        // `crate::memos` already answers, because a `Spelling::Folded` memo is a value keyed on a
        // data revision and that is what folding on the edit is.
        //
        // **It answered seven where the parenthesis named six**, and that is the
        // criterion working rather than failing: `sparkline` holds `PlotState`'s two folded memos as
        // well — it is `chart`'s body with the chrome deleted (components 34) and it folds a million
        // points through the same `Raster`. *Derived, so a component added later joins without
        // anyone remembering*, and `sparkline` shipped ten tickets after the sentence that missed
        // it.
        //
        // **The comparison is against an ordered list and not a subset**, because this is the one
        // obligation of the seven that can go quietly *smaller*: a row that stops declaring
        // `Layer::L2` leaves the population without leaving a failing set, and an ordered equality
        // is what fails first.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: VOLUME,
                    name: "the_covered_rows_are_the_derived_population_in_the_freezes_order",
                },
                Instrument::Unit {
                    file: VOLUME,
                    name: "the_derivation_reaches_sparkline_and_a_written_out_six_would_not_have",
                },
                Instrument::Unit {
                    file: VOLUME,
                    name: "every_volume_set_is_a_decade_apart",
                },
            ],
        },
    },
    Row {
        number: 230,
        on_spec_table: false,
        gate: "every component this crate declares is exercised by an application, and nothing an \
               application exercises is absent from the freeze",
        kind: Kind::Equality,
        owner: "C11",
        section: "spec §17, ADR 0050",
        // **O7, and the argument for it is four defects rather than a preference.** `counter` found
        // that no loop could be written at all and that nothing holds the focus until an
        // application says so; `latency` found `chart`'s rasteriser painting the whole column
        // prefix for every point, 952.61 ms against 2.72 at a million; `ledger` found a table
        // drawing its header one column into the border when handed a panel's interior. Every gate
        // in this crate was green on all four, for four different reasons and with one shape: *a
        // gate exercises the component where its author put it, and an application puts it
        // somewhere else*. Three of the four were found by a person running the thing.
        //
        // **Two equalities, the way O2 has two**, and the first is the one that catches the
        // inventory drifting from what ships — the direction the second cannot see at all.
        //
        // **The join is by path and a bare-name scan is wrong twice over**, with both false
        // positives already in the tree: `keys.rs` declares a `pub fn text(` that is not the `text`
        // component and `gates::table()` prints this register. An import is a `(module, item)`
        // pair, the freeze homes every row through `Component::module`, and `keys::text` and
        // `text::text` never meet. Both are watched **not** counting.
        //
        // **`App::uses` stays and stops being load-bearing.** It is a hand-written column, which is
        // what this register exists to replace; row 231 is the test that makes it agree with the
        // scan rather than be trusted.
        //
        // It was owed **three** rows when it was built — `scrollbar`, `sticky` and `file_picker` —
        // and `crates/vitui-apps/examples/sheet.rs` is what closed them: two of the three are what
        // a caller assembles when it owns the offset itself, which is precisely the case
        // `scroll_area` is not, and until that file nothing in this workspace had called either
        // from outside the component that homes them.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: CONSUMER,
                    name: "every_declared_component_is_exercised_by_an_application",
                },
                Instrument::Unit {
                    file: CONSUMER,
                    name: "the_two_bare_name_collisions_already_in_the_tree_do_not_count",
                },
                Instrument::Unit {
                    file: CONSUMER,
                    name: "the_shared_machine_counts_only_with_the_kind_that_names_the_row",
                },
                Instrument::Unit {
                    file: OBLIGATIONS,
                    name: "the_written_list_and_the_scan_agree_about_o7",
                },
            ],
        },
    },
    Row {
        number: 231,
        on_spec_table: false,
        gate: "the `uses` column of every application agrees with the scan, in both directions",
        kind: Kind::Equality,
        owner: "C11",
        section: "spec §17, ADR 0050",
        // **A list a human maintains is exactly what this register exists to replace**, and this
        // column would go stale in the direction that reads as green: it is what somebody choosing
        // what to open reads. Components 39's review found the sharp instance — the gallery's row
        // listed `gallery::PANELS`, which `crate::gallery`'s own scan **forbids** that file from
        // spelling, so the column documented an application doing exactly what a gate one crate
        // over refuses, unchecked for fifteen applications.
        //
        // **The reverse arm skips the shared machine and that is precision rather than leniency.**
        // `input::toggle_into` is `checkbox`'s, `radio`'s and `switch`'s third spelling at once —
        // one path for three rows — so *this entry implies this component* cannot be read off it.
        // What separates them is the `Toggle` variant the file spells, which a column of paths has
        // no way to say.
        //
        // Forty-nine `(application, component)` pairs, counted, because two empty lists agree about
        // everything — O4's finding, and the reason `Verdict::of` refuses vacuity in its
        // constructor.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: APPS,
                    name: "the_uses_column_agrees_with_the_scan",
                },
                Instrument::Unit {
                    file: APPS,
                    name: "every_name_in_a_uses_column_is_spelled_by_its_own_file",
                },
            ],
        },
    },
    Row {
        number: 232,
        on_spec_table: false,
        gate: "stored state may be an anchor, never a phase: a spinner asks only when the draw put \
               a cell on the screen, and the ask names the widget",
        kind: Kind::Count,
        owner: "C11",
        section: "spec §17, §8, ADR 0051",
        // **The twenty-ninth row of the freeze, and the two rules its own ticket owes.** Spec §8
        // refused a stored transition state, and read as *a component may not store anything a
        // clock moves* that forbids a spinner; the rule is narrower and `disclose::Collapse` is the
        // proof, since it stores a `Tween` across frames. What separates them is what the stored
        // thing **is**: an anchor is a value the state is recoverable from at any `now`, and a
        // phase is only meaningful relative to a frame that already ran.
        //
        // **Both rules are gated with a runnable negative arm, because neither is visible on the
        // rendered surface.** A clipped spinner and a clipped `Ask::Always` spinner are cell for
        // cell identical and one of them keeps the terminal awake; a `Ctx::deadline` and a
        // `Ctx::deadline_for` attribute the same line and only one of them names a widget, so a
        // screen with three spinners is where the census can tell them apart — 3 widgets against 0.
        //
        // **And the clock is the frame's**, which is a scan rather than a type: `Instant::now()`
        // appears in no function of a component module that takes a `Ctx`. The population is a
        // *function* and not a file, because fifty-six library-half calls live in this crate's
        // instruments — six of them inside `collect.rs`, which is a component module — and a scan by
        // file would have carried a growing exception list.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: INDICATE,
                    name: "a_clipped_spinner_asks_for_nothing_and_the_defective_arm_asks_anyway",
                },
                Instrument::Unit {
                    file: INDICATE,
                    name: "three_spinners_name_three_widgets_and_the_defective_arm_names_none",
                },
                Instrument::Unit {
                    file: INDICATE,
                    name: "no_component_body_in_this_crate_samples_its_own_clock",
                },
                Instrument::Unit {
                    file: INDICATE,
                    name: "a_stopped_spinner_is_quiet_at_once_and_a_zero_period_never_asked",
                },
            ],
        },
    },
    Row {
        number: 233,
        on_spec_table: false,
        gate: "a playhead's cadence is the drawn column and not the frame: 60 wakes against 431 991 \
               over a two-hour run",
        kind: Kind::Ratio,
        owner: "C11",
        section: "spec §14, ADR 0051",
        // **The seventh part of the video player's chrome**, and the one row of `PARTS` whose
        // `Needs` was `Clock`. It is the spinner's opposite on two axes and both decide something:
        // it **lands**, so the instant it goes quiet is arithmetic on the anchor; and its visible
        // state changes far more slowly than a frame, because its step is the width of one track
        // column — two minutes over a two-hour film on a sixty-column bar.
        //
        // **The ratio is a consequence of the anchor rather than an optimisation.** The instant the
        // drawn column next moves is a function of `(anchor, duration, width)`: a component that
        // samples its own clock has a `now` nobody else on the screen agrees with, and one that
        // accumulates has nothing to project from at all. Both arms draw the identical screen, so
        // the wake counter is the only thing that separates them.
        //
        // **A scrub is a re-anchor and nothing else** — one assignment, no stored velocity and no
        // second clock — and `ends_at` moves with it, which is asserted rather than claimed.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: PLAYER,
                    name: "only_an_anchored_playhead_can_ask_when_the_column_moves",
                },
                Instrument::Unit {
                    file: PLAYER,
                    name: "a_playhead_lands_and_the_landing_frame_asks_for_nothing",
                },
                Instrument::Unit {
                    file: PLAYER,
                    name: "a_scrub_is_a_re_anchor_and_the_landing_moves_with_it",
                },
                Instrument::Unit {
                    file: PLAYER,
                    name: "the_chrome_advances_the_playhead_and_asks_under_its_own_id",
                },
            ],
        },
    },
    Row {
        number: 234,
        on_spec_table: false,
        gate: "every entry of §16's twenty is drawn on a named line or recorded as undrawn: 15 and \
               5, partitioned",
        kind: Kind::Count,
        owner: "C10, corrected by components architecture 20",
        section: "spec §16, §17",
        // **The join the freeze had was declaration to declaration**, and that is the whole reason
        // this row exists. `the_glyph_table_is_twenty_entries_and_every_one_of_them_has_a_demander`
        // asks *does some row demand this entry*, and a row demanding an entry it draws nothing of
        // answers yes — so `tree` carried three indent guides and `select` a stepper `ArrowUp` for
        // four tickets, and five more went unseen behind `table`'s row.
        //
        // **The needle is assembled from the entry rather than written beside it.** `DRAWERS` lives
        // in the same file as `elide`, which is `Ellipsis`'s drawer, so a literal needle there is a
        // scanner its own source satisfies — this map's most-repeated trap, and the one row that
        // would have sprung it.
        //
        // **One row is `Reach::Computed` and it is `ArrowUp`**: `scrollbar` picks its cap pair by
        // orientation, so `glyph(Glyph::ArrowUp)` is written nowhere and a literal-only join would
        // have put the one arrow that *is* drawn into `UNDRAWN`. An entry wrongly recorded as
        // undrawn is a worse lie than the one this join exists to end.
        //
        // The five that are genuinely undrawn are `table`'s and are one family — the box junctions,
        // and no component here draws two rules that meet. Components architecture 25.
        standing: Standing::Evaluated {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/src/glyphs.rs",
                    name: "every_entry_of_the_table_is_drawn_on_a_line_or_recorded_as_undrawn",
                },
                // **The bounded half.** Crate-wide, `VLine` is drawn and `ArrowUp` is drawn — by
                // `frame` and by `scrollbar`. What was false was that *these two components* draw
                // them, and the smallest region that can say so is the component's own body — one
                // function, not one file, because `collect.rs` homes four components and one of
                // them is `table`, whose row still demands all three and whose rules are components
                // architecture 25's open question. A positive half stops an absence scan passing on
                // an emptied body.
                Instrument::Unit {
                    file: "crates/vitui-components/src/glyphs.rs",
                    name: "trees_body_draws_chevrons_and_no_guides_and_selects_has_no_steppers",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/inventory.rs",
                    name: "the_glyph_table_is_twenty_entries_and_every_one_of_them_has_a_demander",
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
/// G10.
///
/// ```compile_fail,E0277
/// use vitui_runtime::work::Task;
///
/// fn assert_sync<T: Sync>() {}
/// assert_sync::<Task<u32>>();
/// ```
///
/// # `Worker` was said to be `!Sync`, and the shipped one is not
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
                // **This arm runs over nothing since runtime architecture 31 inverted row 112**,
                // and that is worth saying out loud rather than deleting: the discipline for a red
                // row is what the *next* one has to satisfy, and a variant with no constructor is
                // still checked by the compiler. It is the register's own trap — *a gate that cannot
                // fail* — in its least harmful form, because the arm's subject is a row that does
                // not exist rather than a property that is not measured. `crate::scenes` keeps its
                // own `Red` rows, so the variant stays live.
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

    /// **Two hundred and twenty-eight evaluated, and the other five each say why not.**
    ///
    /// This is the number asked for: *how many gates are actually evaluated is a number a test
    /// asserts rather than a claim in a document*. Saying it out loud is what stops the next change
    /// arriving unremarked — a row that quietly stops running has to edit this line, and a row that
    /// starts running has to edit it too.
    #[test]
    fn two_hundred_and_twenty_eight_rows_are_evaluated_and_the_rest_say_why_not() {
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
            Vec::<u8>::new(),
            "**no row of this register is pinned red**, and row 112 was the last. It was a defect \
             in another crate — `Ctx::with_id` re-childed its view at `self.area()`, which inside \
             a scroll scope is the content's rectangle, so a keyed child past the first screenful \
             drew 0 cells of 8 — and **runtime architecture issue 31 inverted it**, which makes it \
             the one inversion here that no components ticket did. §21's rule for a red row is \
             *assert the exact failing set, fire in both directions, say what to invert*, and the \
             set it named is what the gate now asserts green at both offsets and in both \
             directions. **The palette after a swap left with components 41** — row 8, the last of \
             §21's two, and the one whose gate had to be **rewritten** rather than run: \
             `changed > 0` is green on the exact set it exists to catch and its complement is \
             green on a screen with nothing wrong, so the gate is a count over the surface against \
             a second arm played at the destination theme. Its barrier was ADR 0023 and did not \
             need lifting, for row 7's reason; its memo enumeration really did have nothing to \
             enumerate, so the memo the rule is about is built as `gallery::Keying` and the row is \
             watched failing on it. **The sentinel left with components 40** — row 7, the second \
             of the three that was red on a *defect* rather than on a missing subject, and the \
             only one whose prescribed detector could not be built at all: ADR 0023 forbids the \
             readback, so the count is read off `crate::runner::Pen`, which is the instrument the \
             rule's other half has always been read off and the stricter of the two. **Row 169 \
             left with components 32**, which declared `pub fn file_preview_pane<T, F>(` and \
             `pub fn file_picker<'f, T>(` in `src/files.rs` and rewrote `crate::preview::Screen` \
             to draw through the first of them — and not one of the figures rows 170 to 175 carry \
             moved, which is what a screen written from the component's own construction is \
             supposed to buy and is not usually checked. **Row 162 is the picture's**, and row 89 \
             was the previous one of that kind: it was the last until components 26 inverted it. \
             **Twenty wheel clicks is not among them since components 20** — row 29 was red on a \
             *defect* rather than on a missing subject, which is why it took a gate over the \
             shipped path and two removed substitutions rather than a standing edit. Seven have \
             been inverted and each says what it took — the glyph-set count by components 05, row \
             61 by components 10 (which took rewriting the *gate* rather than the standing, \
             because the row asserted an *absence*), row 78 by components 15, which took `table` \
             being declared **and** `crate::grid::draw_into` calling it, row 74 by components 17 \
             on the same two conditions for `tree`, row 89 by components 26, which took `select` \
             and `overlay` being declared **and** the scan's own needle being wrong — it read \
             `pub fn select(` for a component spec §1 already says costs two lifetime annotations \
             — and row 112 by the runtime. **Row 169 is components 31's**, and it is row 162's \
             shape one family over: the three preview-pane screens run — the memo key at 100 of \
             100 frames, the seven batches, the crossover, the five offset spellings, the map and \
             the tear — and what they run over is a pane written beside them, because \
             `src/files.rs` declares neither of the two components. Its needle is deliberately the \
             parenthesis, with the load-bearing half a *second* scan: §15 settles a negative — a \
             job's lifetime is the question's — and a longer needle would be this ticket dictating \
             ticket 32's parameter list"
        );
        assert_eq!(
            unreachable,
            vec![1, 2, 21, 28, 33],
            "the five that cannot be written from a crate whose dependency list is \
             `vitui-runtime` and nothing else. **It was six until runtime architecture issue 34**, \
             which lifted row 161's — the *bytes* the engine wrote, which is row 1's shape one seam \
             further out: `Driver::headless` moves a `Vec` into the engine and `Clock`, `Output`, \
             `Overrides`, `WidthSource` and `InputConfig` were not on `ENGINE_NAMES` at all, so no \
             `Config` this crate could build sent them anywhere it could read. All five are \
             re-exported and the row runs. **It was seven until components ticket 08**, which \
             found row 5's barrier misread: `Mods` is unnameable here and `Chord::mods` hands over \
             the value anyway, so a chord can be pressed after all"
        );
        assert_eq!(
            unsubjected, 0,
            "**and the column is empty.** It was four until production ticket 04, which took the \
             field's rows 15 to 18 — the last four, and the last of §21's own table to stand on \
             nothing. All four named components 24, which built `field` and filed six *new* rows \
             for it instead; three of the four were stale the way 11 and 12 were, and two were \
             missing a half, both the same mistake: **the instrument inspected a state nothing had \
             moved**. Rows 15 and 16 were watched over a walk of the corpus and over the gestures, \
             and §11's two caret defects arrive on an *edit*; row 18's gate says *the width being \
             drawn* and its instruments compared two indexes neither of which was drawn. **It was \
             six until production ticket 03**, which found rows 11 and 12 standing on a stale \
             reading rather than on a missing subject: both named components 15, which declared \
             `table` and left the rows where they were, and `crate::grid` has drawn through the \
             component ever since. **It was eight until components ticket 22**, \
             which supplied the subject for rows 24 and 25 — the inplace map and *`open` is never \
             ambiguous mid-transition*. **That sentence used to call them the last two on §21's \
             own table and it was never true**: rows 11, 12 and 15 to 18 were all `on_spec_table` \
             and all `Unsubjected` at the time. It was \
             fifteen until ticket 18, which supplied row 20's: the bar fixpoint is arithmetic over a \
             domain and needs no component to be run against, and its row had been citing spec §13 \
             and components 28 for two tickets"
        );
        assert_eq!(evaluated + red.len() + unreachable.len() + unsubjected, 234);
    }

    /// **The split, not the total.**
    ///
    /// The table is thirty-two rows and it is closed; everything after it is a gate a ticket on
    /// this lineage wrote. Asserting the split is what made the forty-fifth row — components ticket
    /// 05's matrix barrier — say which side of the line it is on, and a thirty-third row claiming to
    /// be one of them is a design change, which should not be able to arrive as a one-line diff in this
    /// file.
    #[test]
    fn thirty_two_rows_are_the_specs_and_two_hundred_and_two_are_this_lineages() {
        let on_table = REGISTER.iter().filter(|r| r.on_spec_table).count();
        assert_eq!(on_table, SPEC_ROWS);
        assert_eq!(REGISTER.len() - on_table, 202);
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
    /// The test the whole file is shaped around, inherited from the runtime. A `Unit` must be
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
    /// inequality out — `verbs <= writes`, the example of *never verb equality across sizes* —
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
    ///
    /// **The needle is a word and a row can be written past it**, which was found
    /// by drafting one that said *allocates*: it walked straight past the filter, and a scan that
    /// misses the row it is aimed at reports green for the population it happened to match. The
    /// answer is not a wider needle — `alloc` also matches the row about the *allocator*, which
    /// correctly says nothing about totals — so the row was spelled to be caught instead. Both
    /// attempts to tighten this test failed on a row that mentions allocations without being about
    /// a figure, which is why the count is the assertion and the word is only a floor.
    #[test]
    fn an_allocation_row_says_it_is_a_total() {
        let rows: Vec<&Row> = REGISTER
            .iter()
            .filter(|r| r.gate.contains("allocation"))
            .collect();
        assert_eq!(
            rows.len(),
            8,
            "rows 32 and 36, ticket 23's row 82, ticket 25's row 87, ticket 30's two — 163 and \
             165 — ticket 33's row 190, which the needle caught on the first run because it was \
             spelled to be caught, and ticket 39's row 222, which is the first one whose subject is \
             a whole screen rather than one component"
        );
        assert!(
            rows.iter().any(|r| r.gate.contains("total")),
            "the steady-frame allocation row must say `total`"
        );
        // **`any` and not `for`, and the attempt to strengthen it is the second half of the note
        // above.** Row 36 is about the *probe* — *a zeroed allocation is counted exactly once* —
        // and correctly says nothing about totals, so the population is *rows that mention
        // allocations* and the property is *at least one of them is the steady-frame figure and
        // says so*. A `for` here fires on row 36 and names the wrong thing.
    }

    /// **`vitui-alloc-probe` is the only counting allocator in the workspace.**
    ///
    /// The finding: twelve component prototypes each hand-rolled one. The scan is over every `.rs`
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
    /// The glyph-set row, **since inverted.** It was pinned red over an empty population
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
    /// The second half is widened to the workspace, which is what is asked for and what this could
    /// not do while it had nothing to be about: **no private fallback module anywhere**, engine,
    /// runtime and components together.
    ///
    /// The needles are assembled from fragments so that **this file** does not contain them. A source
    /// scan whose own source matches it is the vacuous shape the engine's register records having
    /// shipped once already.
    ///
    /// # One file is excepted, by name and by count
    ///
    /// `CONTEXT.md` says both halves of a collision in two adjacent paragraphs. **Repertoire**: *a
    /// component branches on it rather than the engine substituting behind its back.* **Glyph**:
    /// *anything failing either rule is not a glyph, it is a branch — **the sub-cell ladders are the
    /// case**, because the number of samples asked of the data changes with the rung and no table
    /// can carry that … a component names no repertoire.* The sub-cell ladder is named in the second
    /// sentence as the thing that is a branch, and a branch on the repertoire is a component naming
    /// the repertoire. The same is said in as many words: *the theme owns the lookup; the branch is
    /// two `match`es and a 16-entry array in the component crate.*
    ///
    /// Both cannot hold, and the refinement 3 says what to do about it: **name the exception;
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
    /// The table records **0 against the runtime's 19** at the branch point. The convention is the
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
                "composite_numbers.rs".to_string(),
                "contract_numbers.rs".to_string(),
                "dense_numbers.rs".to_string(),
                "doc_numbers.rs".to_string(),
                "field_numbers.rs".to_string(),
                "gallery_numbers.rs".to_string(),
                "gates_numbers.rs".to_string(),
                "glyph_numbers.rs".to_string(),
                "golden_numbers.rs".to_string(),
                "grid_numbers.rs".to_string(),
                "keys_numbers.rs".to_string(),
                "listing_numbers.rs".to_string(),
                "media_numbers.rs".to_string(),
                "nav_numbers.rs".to_string(),
                "order_numbers.rs".to_string(),
                "partition_numbers.rs".to_string(),
                "popup_numbers.rs".to_string(),
                "press_numbers.rs".to_string(),
                "preview_numbers.rs".to_string(),
                "primitive_numbers.rs".to_string(),
                "scene_numbers.rs".to_string(),
                "series_numbers.rs".to_string(),
                "table_numbers.rs".to_string(),
                "tier_two_numbers.rs".to_string(),
                "tree_numbers.rs".to_string(),
                "volume_numbers.rs".to_string(),
                "wheel_numbers.rs".to_string()
            ],
            "the count on this lineage was 0 against the runtime's 19"
        );
    }

    /// **The named barriers have lifted, and this is the deliberate edit they asked for.**
    ///
    /// This test used to assert the opposite, and its own failure message named the procedure:
    /// *"`{name}` has become reachable through the runtime. That inverts a row of `REGISTER` and is
    /// a deliberate edit here."* The re-export made all four reachable at once, so
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
    /// survived: `Driver::post_mouse` takes one, and until the re-export no crate on this side of the
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
