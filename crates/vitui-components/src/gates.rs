//! Spec §21's register: **fifty-two gates as a value, one row per gate, and a number for how many of
//! them anything runs.**
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
//!   the exact shape that returns green by accident*. Sixteen gates here could run and have **nothing
//!   to run over** — no component exists — and a register that filed those as `Evaluated` would be
//!   claiming sixteen green gates over an empty population.
//!
//! **Thirty-two evaluated, three red, six unreachable, fifteen unsubjected**, and
//! `tests::thirty_two_rows_are_evaluated_and_the_rest_say_why_not` is what makes the next change a
//! deliberate edit rather than a quiet one. It was eighteen / four / six / sixteen until components
//! ticket 05, which inverted row 26 — the glyph-set count, red because it had nothing to be about —
//! and subjected row 27, the cross-family collapse gate; ticket 07 added five, and none of them
//! moved a standing that was already taken. **Ticket 08 added four and inverted one**, and the
//! inversion is the one to read.
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
//! Row 45's barrier survives the same test and rows 7 and 29's do too: those need `Rgb` and `Mouse`,
//! and neither has any nameable box a value could travel inside — `Mouse` needs a `Buttons` and a
//! `MouseKind`, and **neither of those is in `ENGINE_NAMES` at all**, reachable or not. The
//! difference between row 5 and row 45 is not the strength of the barrier, it is that one of them
//! was checked by trying it.
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
//! about the world, it is a line in `crates/vitui-runtime/src/line.rs`, and
//! `tests::the_named_barriers_are_still_barriers` opens that file and fails the day either barrier
//! lifts.
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

/// How many rows of [`REGISTER`] are spec §21's own table. **Thirty-two, and it is closed** — a
/// thirty-third would be a spec change.
pub const SPEC_ROWS: usize = 32;

/// How many rows anything evaluates today. **Thirty-two.**
///
/// The number is the point of the file. §21 counted **2 of 18** at the branch point and **11 of 18**
/// after C11's own pass, both over the prototypes; this is the first count taken over shipped code,
/// and it is thirty-two of fifty-six because twenty-four of the rows are about components that do
/// not exist or need a name the crate line refuses.
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
pub const EVALUATED: usize = 32;

/// Spec §21's register, row for row, and this ticket's gates beside it.
pub const REGISTER: [Row; 56] = [
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
        standing: Standing::Unsubjected {
            inverted_by: "components 12",
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
                Instrument::Barrier {
                    file: "crates/vitui-runtime/src/line.rs",
                    line: "name: \"Rgb\",",
                },
                Instrument::Barrier {
                    file: "docs/adr/0023-the-cell-is-never-visible-in-the-public-api.md",
                    line: "# The cell is never visible in the public API",
                },
            ],
            failing: "9 956 cells of 53 280 (18.7%) over six panels of twelve: the chip screen \
                      4 189, the preview pane 2 159, the scroll area 1 799, media 1 286, the chart \
                      496, the collection pair 27. And the detector itself is unreachable here — \
                      `crate::counters::sentinel` names the three barriers",
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
        standing: Standing::Unsubjected {
            inverted_by: "components 12",
        },
    },
    Row {
        number: 10,
        on_spec_table: true,
        gate: "`size_of::<CollState>()` independent of length",
        kind: Kind::Count,
        owner: "C03",
        section: "spec §5",
        standing: Standing::Unsubjected {
            inverted_by: "components 12",
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
        standing: Standing::Unsubjected {
            inverted_by: "components 13",
        },
    },
    Row {
        number: 14,
        on_spec_table: true,
        gate: "a fold/unfold round trip, and `splice == rebuild`",
        kind: Kind::Equality,
        owner: "C05",
        section: "spec §7",
        standing: Standing::Unsubjected {
            inverted_by: "components 17",
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
        section: "spec §13",
        standing: Standing::Unsubjected {
            inverted_by: "components 28",
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
        standing: Standing::Evaluated {
            by: &[Instrument::Unit {
                file: "crates/vitui-components/tests/gates.rs",
                name: "a_component_drawn_into_a_zero_height_rectangle_declares_no_tab_stops",
            }],
        },
    },
    Row {
        number: 23,
        on_spec_table: true,
        gate: "a fold anchored on a position is reconciled",
        kind: Kind::Count,
        owner: "C14",
        section: "spec §8",
        standing: Standing::Unsubjected {
            inverted_by: "components 22",
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
        standing: Standing::Red {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-components/tests/gates.rs",
                    name: "a_frame_that_asks_for_no_reveal_moves_no_offset_and_one_that_asks_does",
                },
                Instrument::Barrier {
                    file: "crates/vitui-runtime/src/line.rs",
                    line: "name: \"Mouse\",",
                },
            ],
            failing: "0 against 16 over twenty clicks, in four *resolved* tickets' code, each \
                      written by someone who had read the rule in `CONTEXT.md` forbidding it. Half \
                      the gate runs here — an offset that moves when nothing asked is a failure, \
                      which is the direction that stops the fix being *delete the call* — and half \
                      cannot: a wheel click is a posted `Mouse`, and `Mouse` is `reachable_as: None`",
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
                    name: "the_scene_list_is_the_twenty_seven_rows_of_the_specs_table",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/src/scenes.rs",
                    name: "twelve_of_the_thirty_four_axis_obligations_have_a_scene_and_twenty_two_\
                           do_not",
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
                // **The barrier that survived the inversion of row 5.** Eight of the 256 modifier
                // states are constructible here, because `Chord` has three builders and `Mods` has
                // eight bits — so the *exhaustive table* is short even though the rule is not.
                Instrument::Barrier {
                    file: "crates/vitui-runtime/src/line.rs",
                    line: "name: \"Mods\",",
                },
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

    /// Every `.rs` file under `dir`, recursively, skipping build output and git metadata.
    fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let path = entry.expect("a readable entry").path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.is_dir() {
                if name == "target" || name == ".git" {
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

    /// **Every row names a destination, and the numbers are 1..=52 once each.**
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

    /// **Thirty-two evaluated, and the other twenty-four each say why not.**
    ///
    /// This is the number §21 asks for: *how many gates are actually evaluated is a number a test
    /// asserts rather than a claim in a document*. Saying it out loud is what stops the next change
    /// arriving unremarked — a row that quietly stops running has to edit this line, and a row that
    /// starts running has to edit it too.
    #[test]
    fn thirty_two_rows_are_evaluated_and_the_rest_say_why_not() {
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
            vec![7, 8, 29],
            "the three gates that are red and pinned: the sentinel, the palette after a swap and \
             twenty wheel clicks. The glyph-set count was the fourth, and components ticket 05 \
             inverted it by writing the first code in this workspace that has to spell a glyph"
        );
        assert_eq!(
            unreachable,
            vec![1, 2, 21, 28, 33, 45],
            "the six that cannot be written from a crate whose dependency list is \
             `vitui-runtime` and nothing else. **It was seven until components ticket 08**, which \
             found row 5's barrier misread: `Mods` is unnameable here and `Chord::mods` hands over \
             the value anyway, so a chord can be pressed after all"
        );
        assert_eq!(unsubjected, 15, "and the fifteen with nothing to run over");
        assert_eq!(evaluated + red.len() + unreachable.len() + unsubjected, 56);
    }

    /// **The split, not the total.**
    ///
    /// §21's table is thirty-two rows and it is closed; everything after it is a gate a ticket on
    /// this lineage wrote. Asserting the split is what made the forty-fifth row — components ticket
    /// 05's matrix barrier — say which side of the line it is on, and a thirty-third row claiming to
    /// be §21's is a spec change, which should not be able to arrive as a one-line diff in this
    /// file.
    #[test]
    fn thirty_two_rows_are_the_specs_and_twenty_four_are_this_lineages() {
        let on_table = REGISTER.iter().filter(|r| r.on_spec_table).count();
        assert_eq!(on_table, SPEC_ROWS);
        assert_eq!(REGISTER.len() - on_table, 24);
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
        assert_eq!(rows.len(), 2, "row 32 and row 36");
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
                "gates_numbers.rs".to_string(),
                "glyph_numbers.rs".to_string(),
                "keys_numbers.rs".to_string(),
                "nav_numbers.rs".to_string(),
                "partition_numbers.rs".to_string(),
                "press_numbers.rs".to_string(),
                "scene_numbers.rs".to_string()
            ],
            "the count on this lineage was 0 against the runtime's 19"
        );
    }

    /// **The named barriers are still barriers.**
    ///
    /// The half that makes [`Instrument::Barrier`] a gate rather than a citation: `Rgb` and `Mouse`
    /// are unreachable because `crates/vitui-runtime/src/line.rs` says so, and the day either grows
    /// a `reachable_as: Some(..)` this test fails and rows 7 and 29 change standing.
    ///
    /// Read from the shipped inventory rather than from a `use` that would not compile, which is the
    /// only way a crate that cannot name the engine can assert anything about it at all.
    #[test]
    fn the_named_barriers_are_still_barriers() {
        let source = read("crates/vitui-runtime/src/line.rs");
        let lines: Vec<&str> = source.lines().map(str::trim).collect();
        for name in ["Rgb", "Mouse", "Rect", "Mods"] {
            let needle = format!("name: \"{name}\",");
            let at = lines
                .iter()
                .position(|l| *l == needle)
                .unwrap_or_else(|| panic!("`{name}` is no longer an entry of `ENGINE_NAMES`"));
            assert_eq!(
                lines[at + 1],
                "reachable_as: None,",
                "`{name}` has become reachable through the runtime. That inverts a row of \
                 `REGISTER` and is a deliberate edit here"
            );
        }
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
