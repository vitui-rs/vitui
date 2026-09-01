//! Spec §20's register: fifteen gates as a table, and the twenty-nine more the backlog wrote.
//!
//! > **A gate is a count, a ratio, an equality or a compile outcome. A timing is a report, and a
//! > gate only at cliff granularity, with the headroom written next to the number.**
//!
//! The engine keeps the same list one layer down (`crates/vitui-engine/src/register.rs`) and its
//! module comment argues the shape at length. What is repeated here is only the part that is
//! *this* crate's: **a property that quietly never arrives is indistinguishable from one that was
//! decided against**, so every gate is here and every one of them is in exactly one of two states —
//! [`State::Wired`], naming the instruments that run it, or [`State::Red`], naming the
//! implementation ticket that inverts it. **Forty-six wired, none red.** Ticket 19 left this
//! register at thirty-eight and one — entry 12, the dense frame, which was *measured* in two places
//! and *gated* in neither — and ticket 20 built the gate rather than reworded the row. What made
//! that possible is that the red row named the missing instrument precisely enough to build it: a
//! release example plus the CI line that runs it.
//!
//! # The three refinements, which were defects first
//!
//! Spec §20 is engine ticket 13's split with three corrections the runtime had to make, and each of
//! them is enforced by a test at the bottom of this file rather than left as folklore:
//!
//! 1. **A gate is an equality only when the number is a property of the mechanism.**
//!    `walk.len() == 43` is legitimate — 43 is a property of the dense screen and the scene list is
//!    normative. `t_true == 1` is not: 1 was a property of ticket 01's stub palette, and ticket 05
//!    replaced the palette. A number that belongs to the *data* must be a relation, or it becomes
//!    **a gate that is edited rather than fixed**. [`Entry::qualifier`] is where a row says which it
//!    is, and [`tests::an_equality_gate_names_the_mechanism_its_number_belongs_to`] is the check.
//! 2. **A report may not be load-bearing for a gate.** Five negative cases were kept honest only by
//!    a `size_of` line in a benchmark, which nobody had decided and `cargo test` compiled by
//!    accident. [`Instrument::Report`] exists so that a row can *say* it prints a number, and
//!    [`tests::no_gate_rests_on_a_report`] is what stops one being the only thing behind a gate.
//! 3. **A positive twin must name the protected item by path.** The runtime's pairs are
//!    *behavioural*, and a behavioural twin exercises a mechanism without naming it — so renaming
//!    the protected item leaves every binary green while the must-fail half fails identically,
//!    `E0308` having quietly become `E0599`. [`tests::every_negative_case_has_a_twin_that_names_a_path`]
//!    is the gate, and it found the corpus's one unpaired case: [`crate::ctx::Frame::tab_walk`]'s
//!    two fences, which named neither `tab_walk` nor `ring`.
//!
//! # Why the instruments are structured and not prose
//!
//! The engine's register names where an entry runs in a `&'static str` of English, and ticket 19's
//! own brief names the failure mode that invites: *the engine's source-scanning version of this gate
//! was vacuous until it compared a trimmed line*. A sentence cannot be checked, so a row that has
//! quietly stopped running still reads as wired.
//!
//! So [`Instrument`] is a value with a file in it, and
//! [`tests::every_instrument_names_something_that_exists`] opens each file and looks for the thing.
//! A test renamed, a file moved or a `compile_fail` case deleted turns the register red **here**,
//! which is the whole difference between a register and a document.
//!
//! # `source` and `inverted_by` are different numbering schemes
//!
//! `source` names the **implementation** ticket the gate was written by
//! (`.scratch/vitui-runtime-impl/issues/`); `inverted_by` on a red row names the implementation
//! ticket that will light it. The architecture map (`.scratch/vitui-runtime-architecture/`) is a
//! third scheme and is not used here — spec §20's table names implementation tickets as `R NN`, and
//! that is the spelling every row uses.

use std::path::PathBuf;

/// What a register entry costs when it disagrees with the code.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// Fails the build.
    Gate,
    /// An ordinary assertion that happens to be cheap.
    ///
    /// Nothing constructs this or [`Kind::Report`] today, and
    /// [`tests::every_row_of_this_register_is_a_gate`] is where that is written down rather than
    /// discovered. The arms stay for [`State::Red`]'s reason: a distinction the type cannot make is
    /// one a row stops making, and the pressure that produces is a report filed as a gate.
    #[expect(
        dead_code,
        reason = "every row is a gate; the arm stays so a row can say it is not one"
    )]
    Test,
    /// Prints a number a human reads. **May never be load-bearing for a gate** — refinement 2.
    ///
    /// See [`Kind::Test`] for why an arm nothing constructs is kept. Note that
    /// [`Instrument::Report`] is a different thing and is very much constructed: a *gate* may cite
    /// the report that prints its number, and refinement 2 forbids only resting on one.
    #[expect(
        dead_code,
        reason = "every row is a gate; the arm stays so a row can say it is not one"
    )]
    Report,
}

impl Kind {
    /// The word the table prints.
    pub fn word(self) -> &'static str {
        match self {
            Kind::Gate => "gate",
            Kind::Test => "test",
            Kind::Report => "report",
        }
    }
}

/// One thing that runs a register entry, named so that a test can find it.
///
/// Every arm carries a **file**, relative to the workspace root, and something to look for inside
/// it. That is the difference between this register and the engine's: a row here cannot claim to be
/// wired somewhere vague, because [`tests::every_instrument_names_something_that_exists`] opens the
/// file and fails if the thing is not in it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Instrument {
    /// A `#[test]` function. `name` is the bare function name, which is unique enough within a file.
    Unit {
        /// The file, relative to the workspace root.
        file: &'static str,
        /// The test function's name.
        name: &'static str,
    },
    /// A paired doctest: a `compile_fail` fence and the runnable twin beside it.
    ///
    /// `hostile` is the line the must-fail half is *about*, and it is looked for **inside a
    /// `compile_fail` block** — so a mention of it in prose does not satisfy this, and neither does
    /// the twin.
    Pair {
        /// The file, relative to the workspace root.
        file: &'static str,
        /// A fragment of the hostile line.
        hostile: &'static str,
    },
    /// An attribute or a declaration that *is* the gate, with no test to write.
    ///
    /// `#![forbid(unsafe_code)]` is the whole of entry 38: there is no number to tune and no
    /// assertion to make, and a source scan for the word `unsafe` would be the vacuous version.
    /// Compared against a **trimmed** line, which is the engine's own correction to this shape.
    Attribute {
        /// The file, relative to the workspace root.
        file: &'static str,
        /// The line, as it is written, with leading and trailing space ignored.
        line: &'static str,
    },
    /// A file that prints numbers and gates nothing.
    ///
    /// Refinement 2 in a type: a row whose only instrument is one of these may not be a
    /// [`Kind::Gate`].
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
            | Instrument::Attribute { file, .. }
            | Instrument::Report { file } => file,
        }
    }
}

/// Whether something on §20's register runs today.
///
/// One type for both the gate list and the scene list (`crate::scenes`), because it is one
/// statement: this runs on **these instruments**, or it does not run and **that implementation
/// ticket** is what lights it. There is no third arm, and that is the point.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    /// It runs, on these instruments.
    Wired {
        /// What runs it. Never empty.
        by: &'static [Instrument],
    },
    /// It does not run, and the implementation ticket that makes it possible is named.
    ///
    /// **Nothing constructs this today, and the arm stays**, which is the same decision
    /// [`Kind::Test`] records for the same reason: a distinction the type cannot make is one a row
    /// stops making, and the pressure that produces is a red property filed as a wired one. Both
    /// lists reached all-green in ticket 20 — the register's entry 12 first, then
    /// [`crate::scenes`]'s scene 19 — and the second of those is what made the variant dead.
    ///
    /// It arrived as a demonstration rather than as a warning, and both demonstrations are worth
    /// keeping in view. Entry 12 was red the moment the instruments stopped being prose: the dense
    /// frame under the budget was measured in two places and gated in neither, and a sentence saying
    /// where it ran could not tell the difference. Scene 19 was red for the neighbouring reason —
    /// its only instrument was a file `cargo test` compiles and never evaluates. *A property that
    /// quietly never arrives is indistinguishable from one that was decided against.*
    ///
    /// The `#[expect]` is deliberately the two-way form. Writing a red row again makes the
    /// expectation unfulfilled, which is a denied warning, which is a build failure naming this line
    /// — so the next red row costs a one-line deletion here **and cannot arrive without one**. That
    /// is the count tests' *a deliberate edit rather than a quiet one*, enforced by the compiler
    /// instead of by a reviewer.
    #[expect(
        dead_code,
        reason = "both lists are all-green; the arm stays so a row can say it is not, and \
                  constructing it again has to delete this attribute"
    )]
    Red {
        /// The implementation ticket that inverts this entry, as `R NN`.
        inverted_by: &'static str,
        /// What is missing.
        why: &'static str,
    },
}

/// One property of spec §20's register.
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    /// Its number here, which is how everything else refers to it.
    pub number: u8,
    /// Whether it is a row of spec §20's fifteen-row table, or one the backlog wrote beside it.
    ///
    /// The count test asserts **the split** rather than the total, so a fortieth entry has to
    /// say which side of the line it is on — the engine's arrangement, and for its reason.
    pub on_spec_table: bool,
    /// The property, in §20's own words where §20 has words for it.
    pub property: &'static str,
    /// Gate, test or report.
    pub kind: Kind,
    /// What shape of gate: a count, a ratio, an equality, a relation, a compile outcome.
    ///
    /// Refinement 1 lives here: a row whose number belongs to the **data** says `relation`, and a
    /// row that says `equality` is claiming the number is a property of the **mechanism**.
    pub qualifier: &'static str,
    /// The implementation ticket that wrote the gate, as `R NN`.
    pub source: &'static str,
    /// Wired, with its instruments, or red with the ticket that inverts it.
    pub state: State,
}

/// How many rows of [`REGISTER`] are spec §20's table.
///
/// Fifteen, and the table is closed: a sixteenth would be a spec change.
pub const SPEC_ROWS: usize = 15;

/// Spec §20's register, entry for entry, and the backlog's gates beside it.
///
/// **The count is 41 where the ticket's estimate was "roughly twice the register".** That estimate
/// is left in the ticket rather than corrected into it, because it was an estimate: the backlog's
/// eighteen tickets declare sixty-nine gate bullets between them, many of which are the same gate
/// stated from two sides, and twenty-four survive deduplication against §20's fifteen. A row is
/// here when it is a gate somebody can break; a bullet that restates a neighbour is not a second
/// row.
pub const REGISTER: [Entry; 46] = [
    // ── spec §20's table, in its order ───────────────────────────────────────────────────────────
    Entry {
        number: 1,
        on_spec_table: true,
        property: "A sizing function against its component, over a width sweep",
        kind: Kind::Gate,
        qualifier: "equality per width — the two sides are the sizing function and the component \
                    beside it, so what is compared is a mechanism against a mechanism and never \
                    against a recorded number",
        source: "R 15",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/sizing.rs",
                    name: "a_sizing_function_laid_out_from_one_arithmetic_agrees_at_every_width",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/sizing.rs",
                    name: "the_detector_finds_a_component_that_drifted_from_its_sizing_function",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/sizing.rs",
                    name: "the_detector_finds_a_sizing_function_that_claims_a_row_nobody_draws",
                },
            ],
        },
    },
    Entry {
        number: 2,
        on_spec_table: true,
        property: "A keyboard walkthrough visits every tab stop exactly once",
        kind: Kind::Gate,
        qualifier: "count (43, deduped) — an equality, and 43 is a property of the dense screen, \
                    which is normative",
        source: "R 12",
        state: State::Wired {
            by: &[Instrument::Unit {
                file: "crates/vitui-runtime/src/ctx.rs",
                name: "a_keyboard_walkthrough_visits_every_tab_stop_exactly_once",
            }],
        },
    },
    Entry {
        number: 3,
        on_spec_table: true,
        property: "The vanish rule's probe count",
        kind: Kind::Gate,
        qualifier: "count + ratio (601 / 90 002, >= 100x) — the count is the scene's and the ratio \
                    is the mechanism's",
        source: "R 12",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "the_vanish_rule_is_bounded_by_the_ring_and_not_by_the_ring_squared",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/focus.rs",
                    name: "a_quiet_frame_pays_no_probes",
                },
            ],
        },
    },
    Entry {
        number: 4,
        on_spec_table: true,
        property: "Role pairs indistinguishable at each tier",
        kind: Kind::Gate,
        qualifier: "relation per tier — refinement 1's own example: the count belongs to the \
                    palette, and R 05 replaced the palette",
        source: "R 04, R 05",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/theme.rs",
                    name: "narrowing_never_adds_a_distinction",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/theme/registry.rs",
                    name: "narrowing_never_adds_a_distinction_for_any_shipped_theme",
                },
            ],
        },
    },
    Entry {
        number: 5,
        on_spec_table: true,
        property: "`Face`/`FaceHover` distinct at C256, every shipped theme",
        kind: Kind::Gate,
        qualifier: "universal",
        source: "R 05",
        state: State::Wired {
            by: &[Instrument::Unit {
                file: "crates/vitui-runtime/src/theme/registry.rs",
                name: "face_and_face_hover_are_distinct_at_256_colours_for_every_shipped_theme",
            }],
        },
    },
    Entry {
        number: 6,
        on_spec_table: true,
        property: "Every shipped theme names an author",
        kind: Kind::Gate,
        qualifier: "universal",
        source: "R 05",
        state: State::Wired {
            by: &[Instrument::Unit {
                file: "crates/vitui-runtime/src/theme/registry.rs",
                name: "every_shipped_theme_names_an_author",
            }],
        },
    },
    Entry {
        number: 7,
        on_spec_table: true,
        property: "A memo's value moves with the theme <=> the theme is in its key",
        kind: Kind::Gate,
        qualifier: "count, both directions — reading the rule as `every memo` is itself the defect",
        source: "R 05",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/tests/swap.rs",
                    name: "a_memos_value_moves_with_the_theme_exactly_when_the_theme_is_in_its_key",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/tests/swap.rs",
                    name: "a_pair_keyed_memo_misses_once_per_swap_and_a_data_keyed_one_never_does",
                },
            ],
        },
    },
    Entry {
        number: 8,
        on_spec_table: true,
        property: "Zero allocations in a steady frame",
        kind: Kind::Gate,
        qualifier: "count — needs `--test-threads=1`, because the probe's counter is process-global",
        source: "all",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/tests/alloc.rs",
                    name: "a_steady_frame_allocates_nothing",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/tests/alloc.rs",
                    name: "a_whole_screen_of_layout_allocates_zero",
                },
            ],
        },
    },
    Entry {
        number: 9,
        on_spec_table: true,
        property: "The id table does not grow in a steady frame",
        kind: Kind::Gate,
        qualifier: "count (`grows == 0`) — a property of the initial capacity, so an equality",
        source: "R 09",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/id.rs",
                    name: "the_table_does_not_grow_in_a_steady_frame",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "the_id_table_does_not_grow_in_a_steady_frame",
                },
            ],
        },
    },
    Entry {
        number: 10,
        on_spec_table: true,
        property: "A worker cannot reach the app thread's half",
        kind: Kind::Gate,
        qualifier: "compile outcome, paired",
        source: "R 16",
        state: State::Wired {
            by: &[
                Instrument::Pair {
                    file: "crates/vitui-runtime/src/work.rs",
                    hostile: "std::thread::spawn",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/work.rs",
                    name: "the_workers_half_is_send_and_the_app_threads_half_is_not",
                },
            ],
        },
    },
    Entry {
        number: 11,
        on_spec_table: true,
        property: "The negative cases",
        kind: Kind::Gate,
        qualifier: "compile outcome, paired — 31 fences and 58 runnable blocks, counted. **The \
                    error code beside each fence is a convention and not a compile outcome**: \
                    rustdoc on stable ignores it, measured rather than assumed, and the test that \
                    checks the declaration says so at length",
        source: "all",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/register.rs",
                    name: "the_negative_corpus_is_the_size_it_was_left",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/register.rs",
                    name: "every_negative_case_has_a_twin_that_names_a_path",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/register.rs",
                    name: "every_negative_case_declares_the_error_it_expects",
                },
            ],
        },
    },
    Entry {
        number: 12,
        on_spec_table: true,
        property: "The dense frame under the budget",
        kind: Kind::Gate,
        qualifier: "timing at cliff granularity, with the budget reported beside it — the one \
                    shape §20 exempts from `a timing is a report`. **The gate is `examples/frame.rs` \
                    plus the line in `.gitlab-ci.yml` that runs it**, which is what R 20 added and \
                    what this row was red for the absence of. It sits at 2x and not at the budget \
                    because the frame has 1.11x of headroom against a measured 5.3% load penalty, \
                    and `budget.rs` refuses that trade in as many words; the ratio is printed on \
                    every run and the two `Unit` rows above assert the recorded figure under \
                    `cargo test`, so nothing is silently absent",
        source: "R 15, R 17, R 20",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ledger.rs",
                    name: "the_dense_frame_is_inside_the_budget",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ledger.rs",
                    name: "the_runtimes_own_work_is_a_small_share_of_the_frame",
                },
                Instrument::Report {
                    file: "crates/vitui-runtime/examples/frame.rs",
                },
            ],
        },
    },
    Entry {
        number: 13,
        on_spec_table: true,
        property: "Growth at 200 -> 800 widgets",
        kind: Kind::Gate,
        qualifier: "ratio (<= 6x) — the detector the 100 us budget walks past at 1.19x",
        source: "R 09, R 15",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/id.rs",
                    name: "growth_is_sub_quadratic",
                },
                Instrument::Report {
                    file: "crates/vitui-runtime/examples/id_numbers.rs",
                },
            ],
        },
    },
    Entry {
        number: 14,
        on_spec_table: true,
        property: "A `scroll_area` against a `list` over 1M rows",
        kind: Kind::Gate,
        qualifier: "ratio — 887-889x against 0.997-1.003x, and the number belongs to the data, so \
                    the gate is the relation",
        source: "R 14",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/scroll.rs",
                    name: "a_scroll_area_costs_the_content_and_a_list_costs_the_window",
                },
                Instrument::Report {
                    file: "crates/vitui-runtime/examples/scroll_numbers.rs",
                },
            ],
        },
    },
    Entry {
        number: 15,
        on_spec_table: true,
        property: "Bindings reachable per (layout, tier); the lock-key sweep",
        kind: Kind::Gate,
        qualifier: "counts — 33 / 28 / 14 of 33 with 0 wrong, and 33 of 33 with the locks on",
        source: "R 07",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/keys.rs",
                    name: "a_us_layout_reaches_everything_at_every_tier",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/keys.rs",
                    name: "the_loss_is_one_family_and_the_survivor_is_ascii",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/keys.rs",
                    name: "every_binding_still_fires_with_the_locks_on",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/keys.rs",
                    name: "an_equality_match_loses_every_binding_when_caps_lock_is_on",
                },
            ],
        },
    },
    // ── the backlog's gates, which §20's table does not have a row for ───────────────────────────
    Entry {
        number: 16,
        on_spec_table: false,
        property: "The revision counter is process-global: two independently-created `Versioned` \
                   values never share a revision",
        kind: Kind::Gate,
        qualifier: "count — and the swap between them is visible to a memo keyed on either",
        source: "R 01",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/data.rs",
                    name: "two_independently_created_values_never_share_a_revision",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/data.rs",
                    name: "a_per_value_revision_counter_lets_a_data_swap_go_unnoticed",
                },
            ],
        },
    },
    Entry {
        number: 17,
        on_spec_table: false,
        property: "`DerefMut` on `Versioned<T>` does not exist, and the data is exclusive while the \
                   guard is held",
        kind: Kind::Gate,
        qualifier: "compile outcome, paired — the twin names `Versioned` by path",
        source: "R 01",
        state: State::Wired {
            by: &[
                Instrument::Pair {
                    file: "crates/vitui-runtime/src/data.rs",
                    hostile: "v.push(4)",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/data.rs",
                    name: "the_edit_guard_makes_the_same_mistake_unreachable",
                },
            ],
        },
    },
    Entry {
        number: 18,
        on_spec_table: false,
        property: "A `Revision` is nominal — it is not a `u64` wearing a hat",
        kind: Kind::Gate,
        qualifier: "compile outcome, paired. **Gated in the only form that is constructible**: two \
                    instances of this crate cannot be built inside one doctest, so nominality \
                    across crate instances follows from the language and the doc comment says so \
                    rather than implying a gate that does not exist",
        source: "R 01",
        state: State::Wired {
            by: &[Instrument::Pair {
                file: "crates/vitui-runtime/src/data.rs",
                hostile: "let r: Revision = 1;",
            }],
        },
    },
    Entry {
        number: 19,
        on_spec_table: false,
        property: "Zero gaps across every comparable layout spec",
        kind: Kind::Gate,
        qualifier: "equality — 2 406 exhaustive and 200 000 random cases. Contiguity is proved by \
                    construction and the sweep is the check, which is why it is an equality: the \
                    number belongs to the mechanism",
        source: "R 02",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/layout.rs",
                    name: "zero_gaps_over_2406_exhaustive_weight_only_cases",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/layout.rs",
                    name: "zero_gaps_over_200_000_random_cases",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/layout.rs",
                    name: "the_naive_scheme_leaves_gaps_and_this_one_does_not",
                },
            ],
        },
    },
    Entry {
        number: 20,
        on_spec_table: false,
        property: "The crate contains no Unicode table of its own",
        kind: Kind::Gate,
        qualifier: "count — a source scan, and it is the shape the engine's own version of this was \
                    vacuous in until it compared a trimmed line",
        source: "R 03",
        state: State::Wired {
            by: &[Instrument::Unit {
                file: "crates/vitui-runtime/src/layout/text.rs",
                name: "the_runtime_ships_no_unicode_table_of_its_own",
            }],
        },
    },
    Entry {
        number: 21,
        on_spec_table: false,
        property: "`wrap_height(s, w) == wrap(s, w).count()` over a corpus with CJK, a ZWJ family \
                   emoji, VS15/VS16 and combining marks",
        kind: Kind::Gate,
        qualifier: "equality — `wrap_height` and `wrap` are two spellings of one traversal, so the \
                    number is a property of the mechanism at every input the corpus has",
        source: "R 03",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/layout/text.rs",
                    name: "wrap_height_equals_the_number_of_wrapped_lines",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/layout/text.rs",
                    name: "wrapping_preserves_every_cluster_that_is_not_whitespace",
                },
            ],
        },
    },
    Entry {
        number: 22,
        on_spec_table: false,
        property: "There is no route to an arbitrary `Roles`, and no `Paint` without a live \
                   `&Theme`",
        kind: Kind::Gate,
        qualifier: "compile outcome, paired, **narrowed** — not `no route to an arbitrary Paint`, \
                    which read literally makes a chart's fifth series inexpressible",
        source: "R 04",
        state: State::Wired {
            by: &[
                Instrument::Pair {
                    file: "crates/vitui-runtime/src/theme.rs",
                    hostile: "Roles",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "restyle_takes_a_descriptor_over_roles",
                },
            ],
        },
    },
    Entry {
        number: 23,
        on_spec_table: false,
        property: "No glyph spelling is blank, or wider or narrower than one cell",
        kind: Kind::Gate,
        qualifier: "two counts over `Glyph::ALL x GlyphSet`, both 0. **Absence is not \
                    representable**: a blank fallback is no slower, marks the same damage and \
                    allocates the same, so every counter approves of it and only the rendered \
                    surface does not. Twenty entries against sixty spellings since components 05 \
                    grew the table from seven, and the invariant is unchanged because it was \
                    written over `Glyph::ALL` rather than over a list",
        source: "R 04, components 05",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/theme.rs",
                    name: "no_glyph_spelling_is_blank_or_wider_than_one_cell",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/theme.rs",
                    name: "the_glyph_table_has_two_rows_and_not_three",
                },
                // Components ticket 05's half: the table is twenty and the distinction set is ten,
                // and **seven of the ten are carried by a glyph pair whose two halves must stay
                // apart at every rung**. A carrier collapsing is how a bit a component reads goes
                // quiet with no spelling missing and no width wrong — which is the failure the two
                // counts above structurally cannot see.
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/theme.rs",
                    name: "the_table_is_twenty_entries_and_the_distinction_set_is_ten",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/theme.rs",
                    name: "seven_distinctions_are_carried_by_a_glyph_and_three_by_the_palette_alone",
                },
            ],
        },
    },
    Entry {
        number: 24,
        on_spec_table: false,
        property: "Nothing in `theme` names crossterm, and no public signature moves (ADR 0001)",
        kind: Kind::Gate,
        qualifier: "count — a source scan beside `cargo deny`'s graph check, which is the half that \
                    catches a dependency and cannot catch a name",
        source: "R 04, R 17",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/theme.rs",
                    name: "nothing_here_names_the_backend",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/line.rs",
                    name: "the_runtimes_manifest_names_one_dependency",
                },
            ],
        },
    },
    Entry {
        number: 25,
        on_spec_table: false,
        property: "A theme and two swaps allocate zero; `Theme` owns its styles",
        kind: Kind::Gate,
        qualifier: "count",
        source: "R 04",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/tests/alloc.rs",
                    name: "a_theme_and_two_swaps_allocate_zero",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/tests/alloc.rs",
                    name: "resolve_with_glyphs_and_mix_allocate_zero",
                },
            ],
        },
    },
    Entry {
        number: 26,
        on_spec_table: false,
        property: "`Ctx::theme_changed()` is true for exactly one frame",
        kind: Kind::Gate,
        qualifier: "count — the stale count is 0 when it is consulted, and the rest of the screen \
                    is stale when it is not, which is the negative case beside it",
        source: "R 05",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "theme_changed_is_true_for_exactly_one_frame",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/tests/swap.rs",
                    name: "a_consulted_swap_leaves_nothing_carrying_the_previous_palette",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/tests/swap.rs",
                    name: "an_ignored_swap_leaves_the_rest_of_the_screen_stale_and_the_runtime_may_not_fix_it",
                },
            ],
        },
    },
    Entry {
        number: 27,
        on_spec_table: false,
        property: "`Tween<T>` is 48 bytes, `Copy` and heap-free, and a hundred animating frames \
                   allocate 0",
        kind: Kind::Gate,
        qualifier: "count — and the size is an equality because 48 is a property of the type",
        source: "R 06",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/anim.rs",
                    name: "a_tween_is_forty_eight_bytes_and_copy",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/tests/alloc.rs",
                    name: "a_hundred_animating_frames_with_four_animations_allocate_nothing",
                },
            ],
        },
    },
    Entry {
        number: 28,
        on_spec_table: false,
        property: "Every animation helper is a closed form over `(now, start, duration)` — no \
                   helper computes a `dt`",
        kind: Kind::Gate,
        qualifier: "equality — a closed form is a property of the mechanism, and the spinner's 375 \
                    steps over 30 s is what the anchored one gives against the accumulated 353",
        source: "R 06",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/anim.rs",
                    name: "no_helper_computes_a_dt",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/anim.rs",
                    name: "the_anchored_spinner_takes_three_hundred_and_seventy_five_steps",
                },
            ],
        },
    },
    Entry {
        number: 29,
        on_spec_table: false,
        property: "The wake ledger is unconditional, both profiles, no feature flag: a quiet dense \
                   screen is 1 frame then blocks for ever",
        kind: Kind::Gate,
        qualifier: "count — a 300 ms fade is 19 and costs 19 / 2 / 2 gated by tier against 19 at \
                    every tier ungated; **and this is where R 16's `0 wakeups over sixty frames \
                    with a job in flight` was re-gated over the ledger it was owed**",
        source: "R 06, R 16",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/anim.rs",
                    name: "a_quiet_screen_runs_one_frame_and_blocks",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/anim.rs",
                    name: "the_same_fade_is_nineteen_ungated_and_nineteen_two_two_gated",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/anim.rs",
                    name: "sixty_frames_with_a_job_in_flight_ask_for_no_wakeups",
                },
            ],
        },
    },
    Entry {
        number: 30,
        on_spec_table: false,
        property: "Wake attribution is the call site: twelve animated chips attribute to twelve \
                   distinct lines, not to the root id",
        kind: Kind::Gate,
        qualifier: "count",
        source: "R 06",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/anim.rs",
                    name: "twelve_chips_attribute_to_twelve_lines",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/anim.rs",
                    name: "attributing_to_the_id_the_frame_has_answers_root_twelve_of_twelve",
                },
            ],
        },
    },
    Entry {
        number: 31,
        on_spec_table: false,
        property: "A `Binding` stores its chords inline: `keys: &'static [Chord]` at a call site is \
                   `E0716`",
        kind: Kind::Gate,
        qualifier: "compile outcome, paired",
        source: "R 07",
        state: State::Wired {
            by: &[
                Instrument::Pair {
                    file: "crates/vitui-runtime/src/keys.rs",
                    hostile: "&'static [Chord]",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/keys.rs",
                    name: "inline_chords_cost_twenty_four_bytes_over_a_static_slice",
                },
            ],
        },
    },
    Entry {
        number: 32,
        on_spec_table: false,
        property: "An overlay body capturing a base-pass local does not compile, and the twin \
                   naming `Ctx` by path does",
        kind: Kind::Gate,
        qualifier: "compile outcome, paired, **both directions**. The single-lifetime version is \
                    kept as the negative case's comment, because it *compiles* and that is the point",
        source: "R 08, R 13",
        state: State::Wired {
            by: &[
                Instrument::Pair {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    hostile: "cx.overlay(",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "a_body_drawn_inline_and_in_an_overlay_produce_different_ids",
                },
            ],
        },
    },
    Entry {
        number: 33,
        on_spec_table: false,
        property: "`begin` cannot be skipped: a never-`begin`-ed frame fails rather than spins",
        kind: Kind::Gate,
        qualifier: "count — **a failure and not a timeout**, which is why every CI job carries a \
                    timeout as well: `cargo test` has no per-test one and this property is a hang \
                    when it breaks",
        source: "R 08",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "a_claim_terminates_rather_than_spinning",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "a_frame_is_only_reachable_through_a_driver_that_begins_it",
                },
            ],
        },
    },
    Entry {
        number: 34,
        on_spec_table: false,
        property: "Zero id collisions across the enumerated corpus, and the merge policy is a named \
                   test rather than an assumption",
        kind: Kind::Gate,
        qualifier: "count",
        source: "R 09",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/id.rs",
                    name: "zero_collisions_across_the_enumerated_corpus",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/id.rs",
                    name: "a_merge_gives_the_first_claimant_the_id",
                },
            ],
        },
    },
    Entry {
        number: 35,
        on_spec_table: false,
        property: "`Ctx::child`, `Ctx::scope` and `scroll_scope` scope no identity",
        kind: Kind::Gate,
        qualifier: "equality — the ids inside a scope equal the ids without it, entry for entry, \
                    which is a property of the mechanism: nothing about the two lists is recorded",
        source: "R 09, R 14",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "a_scope_scopes_no_identity",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "a_rectangle_returning_split_leaves_its_panes_siblings",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/scroll.rs",
                    name: "a_scroll_scope_scopes_no_identity",
                },
            ],
        },
    },
    Entry {
        number: 36,
        on_spec_table: false,
        property: "The hit index entry is 16 bytes and holds no `Rect` and no layer id, and the \
                   entry count is identical over 2 000 and 1 000 000 rows",
        kind: Kind::Gate,
        qualifier: "equality (16 bytes, a property of the type) + count (**the invariant is gated; \
                    the total is reported and differs by screen**)",
        source: "R 10",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "the_index_entry_is_sixteen_bytes_and_holds_no_geometry",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "the_entry_count_is_identical_over_two_thousand_and_a_million_rows",
                },
            ],
        },
    },
    Entry {
        number: 37,
        on_spec_table: false,
        property: "Every routing edge is a closing edge, and an 8 000-key paste is linear",
        kind: Kind::Gate,
        qualifier: "count (sixteen edges, sixteen frames; eight clicks, none lost) + ratio. **The \
                    growth-ratio detector and not a budget gate** — the quadratic form passes a \
                    100 us gate",
        source: "R 11",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/route.rs",
                    name: "every_routing_edge_that_ships_is_a_closing_edge",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "sixteen_edges_are_sixteen_frames_and_eight_clicks",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/route.rs",
                    name: "draining_an_eight_thousand_key_paste_is_linear",
                },
            ],
        },
    },
    Entry {
        number: 38,
        on_spec_table: false,
        property: "No `unsafe` in any shipped crate above the engine",
        kind: Kind::Gate,
        qualifier: "compile outcome — **the instrument is the attribute rather than a source \
                    scan**. `forbid` and not `deny`, so an inner `allow` cannot lift it; a scan for \
                    the word would be the vacuous version, and the engine's own source-scanning \
                    twin of this was vacuous until it compared a trimmed line. \
                    `vitui-alloc-probe` is the stated exemption (`GlobalAlloc` cannot be safe; \
                    `publish = false`)",
        source: "R 21, ADR 0034",
        state: State::Wired {
            by: &[
                Instrument::Attribute {
                    file: "crates/vitui-runtime/src/lib.rs",
                    line: "#![forbid(unsafe_code)]",
                },
                Instrument::Attribute {
                    file: "crates/vitui-components/src/lib.rs",
                    line: "#![forbid(unsafe_code)]",
                },
                Instrument::Attribute {
                    file: "crates/vitui/src/lib.rs",
                    line: "#![forbid(unsafe_code)]",
                },
                Instrument::Attribute {
                    file: "crates/vitui-engine/src/lib.rs",
                    line: "#![forbid(unsafe_code)]",
                },
            ],
        },
    },
    Entry {
        number: 39,
        on_spec_table: false,
        property: "Zero restricted-visibility items on the component-facing surface",
        kind: Kind::Gate,
        qualifier: "count — and it is here because it is **the honest form of two compile outcomes \
                    that were refused**. Spec §20 records that eight of the old corpus's cases were \
                    ported and two refused: `tab_target` and `hash_loc` would each have cost a \
                    `#[doc(hidden)] pub` to be nameable from a doctest, which is a hole opened in \
                    the surface in order to gate the surface. A count says the same thing and opens \
                    nothing",
        source: "R 17",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/line.rs",
                    name: "no_restricted_item_is_in_the_crate_root",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/line.rs",
                    name: "no_restricted_item_is_re_exported",
                },
                Instrument::Unit {
                    file: "crates/vitui-components/tests/crate_line.rs",
                    name: "a_frame_runs_and_the_id_table_holds_what_drew",
                },
            ],
        },
    },
    // ── written after the backlog closed ─────────────────────────────────────────────────────────
    Entry {
        number: 40,
        on_spec_table: false,
        property: "The app thread can park, and the \u{a7}17 handoff is reachable from a `Driver`",
        kind: Kind::Gate,
        qualifier: "equality \u{2014} between *which wake returned* and *which handle was posted \
                    through*, and it is a property of the mechanism rather than of the data because \
                    **nothing varies**: there is no input, no size and no corpus, only whether the \
                    two ends are the same engine. A handle cloned from a different one parks for \
                    ever. **The park is the gate and not the take**: a polling loop passes a \
                    take-shaped version of this with no `WakeHandle` at all, by running frames \
                    until the slot fills, and pays 60 wakeups against 0 over 60 frames with a job \
                    in flight",
        source: "issue 23",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "a_quit_through_the_drivers_handle_is_the_wake_the_driver_returns",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "a_worker_hired_from_the_driver_wakes_it_and_the_answer_is_there",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "the_handle_is_send_and_survives_a_frame",
                },
            ],
        },
    },
    Entry {
        number: 41,
        on_spec_table: false,
        property: "Nothing holds the focus until an application seats it, and the two seating forms \
                   are different programs",
        kind: Kind::Gate,
        qualifier: "equality \u{2014} between *where the focus ends up after the user moves it* on \
                    the two forms an application might write. `focused().is_none()` and \
                    `!is_focused(sink)` read alike and diverge only once there is somewhere else to \
                    be: the second drags the keyboard back every frame the user has tabbed away, \
                    which presents as a `Tab` that does nothing. **Both arms seat it correctly on \
                    the first frame**, which is why a one-widget program cannot tell them apart and \
                    why this is a gate rather than a review note. The first frame having nobody \
                    focused is asserted on both arms, because that is the finding the verb exists \
                    for. It is **a property of the mechanism rather than of the data**: nothing \
                    varies across the two arms but which of two lines the application wrote, and \
                    there is no size, no corpus and no input",
        source: "issue 25",
        state: State::Wired {
            by: &[Instrument::Unit {
                file: "crates/vitui-runtime/src/ctx.rs",
                name: "the_guarded_seating_form_seats_once_and_the_is_focused_form_steals_it_back",
            }],
        },
    },
    Entry {
        number: 42,
        on_spec_table: false,
        property: "A scroll scope's offset is a position, and the window, a verb and a press all \
                   read it that way",
        kind: Kind::Gate,
        qualifier: "equality \u{2014} between *the content rows `visible_rows` names* and *the rows a \
                    verb and a press actually reach*, at a nonzero offset. It is a property of the \
                    mechanism rather than of the data: nothing varies but which sign each of three \
                    fields takes, and there is no corpus and no size. **The offset has to be \
                    nonzero and something has to be drawn through it**, which is the whole finding \
                    \u{2014} at offset 0 every sign agrees, and the register's other two scroll \
                    rows assert a ratio and an identity, neither of which is a cell landing \
                    anywhere. The press half is a second row's worth of \
                    care in one row because it is the same negation read backwards: `pointer` takes \
                    the translation with the sign **opposite** to `view` and `origin`, and a \
                    version where the three agree puts a click `2 \u{b7} offset` rows from where the \
                    user pointed",
        source: "issue 26",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "a_scroll_scopes_offset_is_a_position_and_the_window_is_the_rows_it_names",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/ctx.rs",
                    name: "a_press_inside_a_scroll_scope_lands_on_the_row_under_the_pointer",
                },
            ],
        },
    },
    Entry {
        number: 43,
        on_spec_table: false,
        property: "A chord on a character the user typed matches every spelling the wire has for \
                   that character, and one on a named key still compares all six intent bits",
        kind: Kind::Gate,
        qualifier: "equality \u{2014} between *the action a `Typed` chord names* and *the action four \
                    different `Key` values fire*, where the four are one keystroke as four terminals \
                    describe it. It is a property of the mechanism and not of the data: the corpus is \
                    the wire's own spellings and there are exactly four of them. **Three fire and the \
                    fourth is asserted not to**, which is the row's load-bearing half \u{2014} \
                    `CSI 61;2u` reports the unshifted key with no associated text, so nothing in the \
                    process knows a `+` was produced, and a gate that made it fire would be asserting \
                    that this crate reads a keyboard layout it has refused to consult. The guard \
                    against over-fixing is a second instrument rather than a second row: masking \
                    shift on every character chord \u{2014} the one-line repair the ticket rejected \
                    \u{2014} makes `Chord::key('a').shift()` equal `Chord::key('a')`, and \
                    `shift_still_means_shift_on_a_base_layout_chord` is what fails when it does",
        source: "issue 28",
        state: State::Wired {
            by: &[
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/keys.rs",
                    name: "a_typed_chord_matches_the_three_wire_spellings_of_one_character",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/keys.rs",
                    name: "shift_still_means_shift_on_a_base_layout_chord",
                },
                Instrument::Unit {
                    file: "crates/vitui-runtime/src/keys.rs",
                    name: "shift_on_a_typed_chord_is_a_no_op_and_is_recorded_as_one",
                },
            ],
        },
    },
    Entry {
        number: 44,
        on_spec_table: false,
        property: "The press is published as an edge beside the level, and holding the button does \
                   not repeat it",
        kind: Kind::Gate,
        qualifier: "equality \u{2014} between *the four frames one press produces* and *the two \
                    bools each of them publishes*, as a sequence. It is a property of the \
                    mechanism and not of the data: there is one widget, one press and one release, \
                    and nothing varies but which frame each bool falls on. **The held frame is \
                    what the row is for** \u{2014} `pressed` is `grab == Some(id)` and so is true \
                    from the press until the release, while `press_began` is `Awarded::pressed`, \
                    set once in the `Down` arm. A component that applied a gesture on the level \
                    ran it once a frame for as long as the button was down, which a **plain** \
                    click hides completely: `Gesture::Plain(at)` is idempotent, so the lead, the \
                    anchor and the one selected row are the same however many times it runs, and \
                    only a ctrl-click flickers. That is why the reconstruction this row deletes \
                    survived four component tickets",
        source: "issue 29",
        state: State::Wired {
            by: &[Instrument::Unit {
                file: "crates/vitui-runtime/src/ctx.rs",
                name: "the_press_is_published_as_an_edge_and_holding_does_not_repeat_it",
            }],
        },
    },
    Entry {
        number: 45,
        on_spec_table: false,
        property: "An identity verb narrows identity and never the view, so a keyed or scoped child \
                   inside a scroll scope reaches the window at every offset",
        kind: Kind::Gate,
        qualifier: "equality \u{2014} between *the cells three spellings of one row loop land* and \
                    *the cells the loop without an identity verb lands*, at two offsets. It is a \
                    property of the mechanism and not of the data: there is one scope, one column \
                    and eight rows, and nothing varies but which verb wraps the write. **The \
                    unkeyed arm is the control and is why the row is an equality rather than a \
                    count** \u{2014} `with_id` and `Ctx::scope` both built their body with \
                    `view: self.view.child(self.area())`, and `area()` is `Rect::new(0, 0, w, h)` \
                    in the *current* coordinate system, which inside a scroll scope is the \
                    content's: the rectangle named content rows `0..h` while the window sat at the \
                    offset, so past the first screenful the intersection was empty and the keyed \
                    arm landed 0 of 8. At offset zero the re-child is the identity in every field, \
                    which is why every caller on this map drew through it \u{2014} they all play \
                    at 0, and a gate that asserted the keyed arm alone could not tell a broken \
                    `with_key` from a broken `scroll_scope`",
        source: "issue 31",
        state: State::Wired {
            by: &[Instrument::Unit {
                file: "crates/vitui-runtime/src/ctx.rs",
                name: "an_identity_verb_inside_a_scroll_scope_reaches_the_window",
            }],
        },
    },
    Entry {
        number: 46,
        on_spec_table: false,
        property: "A frame that leaves a scroll-into-view request asks for the frame that reads it",
        kind: Kind::Gate,
        qualifier: "count \u{2014} of the wakeup sink after **one** frame, which is the whole \
                    difficulty: a reveal is a two-frame gesture, and every other instrument on \
                    this map drives its own second frame because it wants to observe one, so the \
                    harness supplied the wake the runtime did not and the lag was invisible by \
                    construction. Both producers are arms \u{2014} the explicit `Ctx::request_into_view` \
                    and the ring's keyboard pull, which no application spells \u{2014} and **the first \
                    arm is a control**: the same scope drawn with nothing asking parks, so the two \
                    below measure a request rather than a driver that always wants a frame. It is \
                    a count of a fold and not of a timing: the sink is one `Option<Instant>` and \
                    the question is whether it is inhabited",
        source: "issue 33",
        state: State::Wired {
            by: &[Instrument::Unit {
                file: "crates/vitui-runtime/src/scroll.rs",
                name: "a_frame_that_leaves_an_into_view_request_wakes_the_screen",
            }],
        },
    },
];

/// How many `compile_fail` fences the crate carries.
///
/// An equality and not a floor, because the number is a property of the mechanism rather than of the
/// data: **the pair is the unit**, and a case deleted without its twin is precisely the edit this
/// number exists to catch. Thirty-one at ticket 19, where the corpus's own history begins: spec §20
/// counts 55 compile outcomes in 15 files behind six `cfg` names with **0 evaluated by any CI
/// command**, and what ships is a smaller number that runs.
pub const NEGATIVE_CASES: usize = 31;

/// How many **runnable** doc blocks the crate carries — the positive twins, and the ordinary
/// examples beside them.
///
/// Counted for the same reason as [`NEGATIVE_CASES`], and it is the half a corpus count over the
/// fences alone leaves unguarded: *the twin is precisely the half that catches a rename*. Deleting
/// one on its own leaves every other test green, at which point a later rename makes the surviving
/// negative case fail for the wrong error and report `ok`.
///
/// Fifty-eight at ticket 19; **fifty-nine since architecture issue 23**, whose one addition is the
/// `no_run` loop on [`Driver::wait`](crate::ctx::Driver::wait) — the only doc block in the crate
/// that cannot be `run`, because a loop with nothing pending parks for ever by design; **sixty since
/// architecture issue 34**, whose addition is on
/// [`Theme::colours_differ_on_wire`](crate::Theme::colours_differ_on_wire) and shows the whole of
/// the verb: two colours one unit of blue apart, separate at truecolor and one colour at sixteen.
pub const RUNNABLE_EXAMPLES: usize = 60;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// The workspace root, from this crate's manifest directory.
    fn workspace_root() -> PathBuf {
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    fn read(relative: &str) -> String {
        let path = workspace_root().join(relative);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// Every `.rs` file under this crate's `src/`, recursively.
    fn src_files() -> Vec<PathBuf> {
        fn walk(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
            for entry in std::fs::read_dir(dir).expect("src/ is beside this file") {
                let path = entry.expect("a readable entry").path();
                if path.is_dir() {
                    walk(&path, out);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    out.push(path);
                }
            }
        }
        let mut out = Vec::new();
        walk(
            &PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")),
            &mut out,
        );
        out.sort();
        out
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
    /// A raw `source.contains("fn {name}(")` was the first version of this and it is the vacuous
    /// shape this file exists to refuse: it matches inside a doc comment, inside a string literal
    /// and inside a commented-out block, and it matches a plain helper just as well as a test. So a
    /// `#[test]` demoted to a helper — or left in place with `#[ignore]` on it — kept the register
    /// reading `Wired` while nothing ran, which is precisely the state the module comment says the
    /// register makes impossible.
    ///
    /// What is required instead: a line whose trimmed form starts `fn {name}(`, with `#[test]`
    /// among the attributes directly above it and no `#[ignore]` among them.
    fn declares_a_live_test(source: &str, name: &str) -> bool {
        let signature = format!("fn {name}(");
        let lines: Vec<&str> = source.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            if !line.trim_start().starts_with(&signature) {
                continue;
            }
            let mut attributed = false;
            let mut ignored = false;
            // Walk back over the attribute and doc-comment block immediately above the signature.
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

    /// The names a doc run's `**Protects:**` line declares, if it has one.
    ///
    /// # Why an opt-out exists at all, and why it is written down rather than guessed
    ///
    /// A pair usually sits on the item it protects, and requiring the twin to name that item is
    /// what makes refinement 3 a gate. **Twice in this corpus it does not**, both for good reasons
    /// and neither of them guessable:
    ///
    /// - `id::WhyAnIdIsNotOrdered` is a `#[cfg(doc)]` marker that exists only to hold a pair. The
    ///   protected item is `Id`; asking the twin to name the marker would be asking for a sentence
    ///   about a type nobody calls.
    /// - `sizing::Extent` carries the `E0499` case for the trait that **was not built** — the
    ///   module comment says so — and its twin correctly names `sizing::check` and `Agreement`,
    ///   which are what shipped instead.
    ///
    /// The first version of this gate met those two by loosening to *the twin shares an identifier
    /// with the fence*, and that is worse than it sounds: a twin of one `use vitui_runtime::…` line
    /// and an unrelated assertion shares `Driver` and `ctx` with every fence in the file, so the
    /// rule would have been satisfied by boilerplate. **An exception that is written into the
    /// document is a decision; an exception a heuristic infers is a hole**, and this file's whole
    /// argument is that the difference matters.
    ///
    /// The line is ordinary rustdoc, so a reader sees it too:
    ///
    /// ```text
    /// /// **Protects:** `sizing::check`, `Agreement`
    /// ```
    fn protects_line(run: &str) -> Option<BTreeSet<String>> {
        let line = run.lines().find(|l| l.contains("**Protects:**"))?;
        // Only the names in the declaration itself, which ends at the first full stop or em dash.
        // The prose after it explains the exception and routinely quotes other things — the first
        // version of this read `#[cfg(doc)]` out of `Id`'s own reason and demanded a twin name it.
        let declaration = line
            .split_once("**Protects:**")
            .expect("the line contains it")
            .1;
        let declaration = declaration
            .split_once(" — ")
            .map_or(declaration, |(before, _)| before);
        let declaration = declaration
            .split_once(". ")
            .map_or(declaration, |(before, _)| before);
        let names: BTreeSet<String> = declaration
            .split('`')
            .skip(1)
            .step_by(2)
            .map(|name| {
                // `sizing::check` protects the last segment, which is what a twin writes.
                name.rsplit("::").next().unwrap_or(name).trim().to_string()
            })
            .filter(|name| !name.is_empty())
            .collect();
        (!names.is_empty()).then_some(names)
    }

    /// Whether a fence's attributes make it a negative case.
    fn is_negative(attributes: &str) -> bool {
        attributes.contains("compile_fail")
    }

    /// Whether a fence's attributes make it a runnable doctest — the shape a positive twin has.
    ///
    /// `text` is neither: it is prose in a box. `ignore` and `no_run` are not used here, and
    /// `no_run` is accepted anyway because it still *compiles*, which is the whole of what a twin
    /// has to do.
    fn is_runnable(attributes: &str) -> bool {
        matches!(attributes, "" | "rust" | "no_run")
    }

    /// One item's doc comment, with the name of the item it is attached to.
    ///
    /// A pair is a property of an **item**, not of a file: the twin has to sit on the thing the
    /// hostile line is about, or a reader cannot tell which of a file's twenty runnable blocks is
    /// the twin of which fence — and, worse, the gate over it cannot tell either.
    ///
    /// The name is what makes refinement 3 checkable rather than merely stated. It is the
    /// identifier declared on the first non-attribute line after the run: `pub fn tab_walk` gives
    /// `tab_walk`, `pub struct Revision(u64)` gives `Revision`. A module-level `//!` run has no
    /// item and yields `None`, and so does a `#[cfg(doc)]` **carrier** — a marker type whose whole
    /// purpose is to hold a pair, where the protected item is what the fences name rather than the
    /// marker.
    fn doc_runs(source: &str) -> Vec<(Option<String>, String)> {
        let lines: Vec<&str> = source.lines().collect();
        let mut out = Vec::new();
        let mut index = 0;
        while index < lines.len() {
            let trimmed = lines[index].trim_start();
            let marker = if trimmed.starts_with("///") {
                "///"
            } else if trimmed.starts_with("//!") {
                "//!"
            } else {
                index += 1;
                continue;
            };
            let mut body: Vec<&str> = Vec::new();
            while index < lines.len() && lines[index].trim_start().starts_with(marker) {
                body.push(lines[index]);
                index += 1;
            }
            // The declaration the run is attached to, skipping the attributes between the two.
            //
            // A `#[cfg(doc)]` declaration is a **carrier**: a marker type that exists only to hang a
            // pair on, so the protected item is something the fences name rather than the marker
            // itself. `id::WhyAnIdIsNotOrdered` is one, and asking its twin to name it would be
            // asking for a sentence about a type nobody calls.
            let mut item = None;
            let mut carrier = false;
            if marker == "///" {
                let mut look = index;
                while look < lines.len() {
                    let line = lines[look].trim_start();
                    if line.starts_with("#[cfg(doc)]") {
                        carrier = true;
                    }
                    if line.starts_with("#[") || line.starts_with("#!") || line.is_empty() {
                        look += 1;
                        continue;
                    }
                    item = declared_name(line);
                    break;
                }
            }
            out.push((if carrier { None } else { item }, body.join("\n")));
        }
        out
    }

    /// The identifier a declaration line declares, if it declares one.
    ///
    /// Deliberately generous about the keyword — `fn`, `struct`, `enum`, `const`, `type`, a field —
    /// because the twin has to name whatever the fence is about, and being strict here would turn a
    /// missed keyword into a silently skipped item, which is the vacuity this file refuses.
    fn declared_name(line: &str) -> Option<String> {
        let line = line.trim();
        let line = line
            .strip_prefix("pub(crate) ")
            .or_else(|| line.strip_prefix("pub(super) "))
            .or_else(|| line.strip_prefix("pub "))
            .unwrap_or(line);
        for keyword in [
            "const fn ",
            "async fn ",
            "unsafe fn ",
            "fn ",
            "struct ",
            "enum ",
            "trait ",
            "union ",
            "type ",
            "const ",
            "static ",
            "mod ",
            "macro_rules! ",
        ] {
            if let Some(rest) = line.strip_prefix(keyword) {
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                return (!name.is_empty()).then_some(name);
            }
        }
        // A struct field: `name: Type,`. The last shape a fence is plausibly about.
        let name: String = line
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .collect();
        (!name.is_empty() && line[name.len()..].starts_with(':')).then_some(name)
    }

    /// **Every entry names somewhere to look, and a red one names an implementation ticket.**
    ///
    /// `issue NN` joins `R NN` and `all` at entry 40 and again at 41, and the widening is the
    /// honest form rather
    /// than a loosening: the implementation backlog **closed** on 2026-08-24, so a gate written
    /// after it has no `R` number to cite, and giving it one would be a citation to a file that
    /// does not exist. An architecture issue is somewhere to look, which is what this test is
    /// named for.
    #[test]
    fn every_entry_names_a_destination() {
        for entry in REGISTER {
            assert!(
                !entry.property.is_empty() && !entry.qualifier.is_empty(),
                "entry #{} has no property or no qualifier",
                entry.number
            );
            assert!(
                entry.source.starts_with("R ")
                    || entry.source.starts_with("issue ")
                    || entry.source == "all",
                "entry #{}'s source `{}` is neither an implementation ticket, an architecture \
                 issue, nor `all`",
                entry.number,
                entry.source
            );
            match entry.state {
                State::Wired { by } => assert!(
                    !by.is_empty(),
                    "entry #{} is wired without saying on what",
                    entry.number
                ),
                State::Red { inverted_by, why } => {
                    assert!(
                        inverted_by.starts_with("R "),
                        "entry #{} is red against `{inverted_by}`, which is not an implementation \
                         ticket",
                        entry.number
                    );
                    assert!(
                        !why.is_empty(),
                        "entry #{} is red without a reason",
                        entry.number
                    );
                }
            }
        }
    }

    /// **The numbers are 1..=N, once each.**
    #[test]
    fn the_numbers_are_contiguous_and_unique() {
        let seen: BTreeSet<u8> = REGISTER.iter().map(|e| e.number).collect();
        assert_eq!(seen.len(), REGISTER.len(), "a number appears twice");
        let expected: BTreeSet<u8> = (1..=u8::try_from(REGISTER.len()).expect("fits")).collect();
        assert_eq!(seen, expected, "the numbers are not 1..={}", REGISTER.len());
    }

    /// **Forty-six wired, none red.**
    ///
    /// This register was thirty-eight and one from ticket 19 until ticket 20 built the gate entry
    /// 12 was red for the absence of; forty since architecture issue 23 — the first row here whose
    /// source is an *architecture* issue rather than an implementation ticket, because the backlog
    /// was closed when the gap was found — forty-one since issue 25, which is the second and
    /// arrived the same way, forty-two since issue 26, which is the third: a defect three
    /// tickets one layer up found before this register had a row that could — forty-three since
    /// issue 28, the fourth, and the first of them this register could not have had a row for at
    /// all, the key it compares against having been unbuildable outside the engine until that
    /// ticket put `KeyText::of` on the engine's surface — forty-four since issue 29, the
    /// fifth, which is the one that *deletes* a store one layer up rather than gating a sign: the
    /// press had an edge all along and published only the level — and **forty-five since issue
    /// 31**, the sixth, which is the one that inverts a *red row in another crate's register*:
    /// `Ctx::with_id` and `Ctx::scope` childed their body at `self.area()`, which inside a scroll
    /// scope is the content's rectangle, and the row exists here because the property is this
    /// crate's even though three components tickets are what met it — and **forty-six since
    /// issue 33**, the seventh, which is the one whose gate could not be written the way every
    /// other row on this map is written: the property is that *a frame asks for the next one*, and
    /// an instrument that drives its own second frame supplies exactly the thing under test, so
    /// this one draws a single frame per arm and asks the wakeup sink instead. Saying *how many* is
    /// what stops a red row arriving unremarked, and it
    /// has the second job the engine's has: **a register at all-green says so**, so the next red row
    /// is a deliberate edit to this number rather than a quiet one.
    #[test]
    fn forty_six_are_wired_and_none_are_red() {
        let red: Vec<u8> = REGISTER
            .iter()
            .filter(|e| matches!(e.state, State::Red { .. }))
            .map(|e| e.number)
            .collect();
        assert_eq!(
            red,
            Vec::<u8>::new(),
            "a red row is back. A new one is fine and has to be argued for here, in this test's \
             documentation, and in the module comment above — the count is the thing that stops it \
             arriving unremarked"
        );
        assert_eq!(REGISTER.len() - red.len(), 46);
    }

    /// **The split, not the total.**
    ///
    /// Spec §20's table is fifteen rows and it is closed; everything else is a gate the backlog
    /// wrote beside it. Asserting the split rather than the sum is what makes a fortieth entry
    /// say which side of the line it is on — and a sixteenth row claiming to be §20's is a spec
    /// change, which should not be able to arrive as a one-line diff in this file.
    #[test]
    fn fifteen_rows_are_the_specs_and_the_rest_are_the_backlogs() {
        let on_table = REGISTER.iter().filter(|e| e.on_spec_table).count();
        assert_eq!(on_table, SPEC_ROWS, "spec §20's table is fifteen rows");
        assert_eq!(
            REGISTER.len() - on_table,
            31,
            "the backlog's gates, deduplicated against §20's fifteen, plus issues 23's, 25's, \
             26's, 28's, 29's, 31's and 33's"
        );
        // And §20's fifteen come first, so the table reads in the spec's order.
        for (index, entry) in REGISTER.iter().enumerate() {
            assert_eq!(
                entry.on_spec_table,
                index < SPEC_ROWS,
                "entry #{} is out of the spec-table block",
                entry.number
            );
        }
    }

    /// **Every instrument names something that exists.**
    ///
    /// This is the test the whole file is shaped around. The engine's register names where an entry
    /// runs in a sentence, and a sentence cannot be checked — so a row whose test has been renamed
    /// still reads as wired, which is exactly the state the register exists to make impossible.
    ///
    /// A `Unit` must be a function in its file, a `Pair`'s hostile line must appear **inside a
    /// `compile_fail` block** (a mention in prose does not count, and neither does the twin), an
    /// `Attribute` is compared against a **trimmed** line — the engine's correction to its own
    /// vacuous version of this — and a `Report` must be a file that is there.
    #[test]
    fn every_instrument_names_something_that_exists() {
        for entry in REGISTER {
            let State::Wired { by } = entry.state else {
                continue;
            };
            for instrument in by {
                let source = read(instrument.file());
                match *instrument {
                    Instrument::Unit { file, name } => assert!(
                        declares_a_live_test(&source, name),
                        "entry #{}: `{file}` has no live `#[test] fn {name}`. A helper `fn` of \
                         that name, or one carrying `#[ignore]`, leaves this row reading `Wired` \
                         while nothing runs",
                        entry.number
                    ),
                    Instrument::Pair { file, hostile } => {
                        let found = doc_blocks(&source)
                            .iter()
                            .any(|(attrs, body)| is_negative(attrs) && body.contains(hostile));
                        assert!(
                            found,
                            "entry #{}: no `compile_fail` block in `{file}` contains `{hostile}`. \
                             A mention in prose is not the gate",
                            entry.number
                        );
                    }
                    Instrument::Attribute { file, line } => assert!(
                        source.lines().any(|l| l.trim() == line),
                        "entry #{}: `{file}` does not carry the line `{line}`",
                        entry.number
                    ),
                    Instrument::Report { file } => assert!(
                        !source.is_empty(),
                        "entry #{}: the report `{file}` is empty",
                        entry.number
                    ),
                }
            }
        }
    }

    /// **No gate rests on a report** — refinement 2, as a test.
    ///
    /// Five negative cases were once kept honest only by a `size_of` line in a benchmark, which
    /// nobody had decided and `cargo test` compiled by accident. The shape to not repeat is a gate
    /// whose only instrument is a file nothing runs: this workspace's CI runs `cargo test` and two
    /// named examples, so an `assert!` inside any other `examples/*.rs` is compiled and never
    /// evaluated. A row may cite a report — a number a human reads next to the gate is worth having
    /// — but never *only* a report.
    #[test]
    fn no_gate_rests_on_a_report() {
        for entry in REGISTER {
            let State::Wired { by } = entry.state else {
                continue;
            };
            if entry.kind != Kind::Gate {
                continue;
            }
            assert!(
                by.iter().any(|i| !matches!(i, Instrument::Report { .. })),
                "entry #{} is a gate whose only instrument is a report. An `assert!` in an \
                 `examples/*.rs` this workspace's CI does not run is compiled and never evaluated",
                entry.number
            );
        }
    }

    /// **An equality gate names the mechanism its number belongs to** — refinement 1, as a test.
    ///
    /// `t_true == 1` was a gate on a number that belonged to a stub palette, and replacing the
    /// palette made it a gate that is edited rather than fixed. There is no way to decide from the
    /// outside whether a number belongs to the mechanism or to the data, so what is checked is that
    /// the row **says which** — an `equality` qualifier has to carry its reason, and a row whose
    /// number belongs to the data has to say `relation` or `ratio` instead.
    #[test]
    fn an_equality_gate_names_the_mechanism_its_number_belongs_to() {
        for entry in REGISTER {
            let q = entry.qualifier;
            let shapes = [
                "count",
                "ratio",
                "equality",
                "relation",
                "compile outcome",
                "universal",
            ];
            assert!(
                shapes.iter().any(|s| q.contains(s)) || q.contains("timing"),
                "entry #{}'s qualifier `{q}` names no gate shape. A gate is a count, a ratio, an \
                 equality, a relation or a compile outcome; a timing is a report, and a gate only \
                 at cliff granularity",
                entry.number
            );
            if q.contains("equality") {
                assert!(
                    q.contains("property of the")
                        || q.contains("mechanism")
                        || q.contains("normative")
                        || q.contains("proved by construction")
                        || q.contains("a property of the type"),
                    "entry #{} is an equality and does not say why its number is a property of the \
                     mechanism rather than of the data",
                    entry.number
                );
            }
        }
    }

    /// **The negative corpus is the size it was left, and so is the twin corpus.**
    ///
    /// Both counts, for the reason [`RUNNABLE_EXAMPLES`] gives: a count over the fences alone leaves
    /// the twins unguarded, and a case removed with its twin leaves every other test in the
    /// workspace green.
    #[test]
    fn the_negative_corpus_is_the_size_it_was_left() {
        let mut negative = 0;
        let mut runnable = 0;
        for file in src_files() {
            let source = std::fs::read_to_string(&file).expect("a readable module");
            for (attributes, _) in doc_blocks(&source) {
                if is_negative(&attributes) {
                    negative += 1;
                } else if is_runnable(&attributes) {
                    runnable += 1;
                }
            }
        }
        assert_eq!(
            negative, NEGATIVE_CASES,
            "the negative corpus changed size. A case removed with its twin leaves every other test \
             green"
        );
        assert_eq!(
            runnable, RUNNABLE_EXAMPLES,
            "the twin corpus changed size. Guarding the hostile lines and not the twins leaves \
             unguarded the half that catches a rename"
        );
    }

    /// **Every negative case has a twin, and the twin names a path** — refinement 3, as a test.
    ///
    /// A behavioural twin exercises a mechanism without naming it, so renaming the protected item
    /// leaves every binary green while the must-fail half fails identically — `E0308` having quietly
    /// become `E0599`. Checked per **item**: a runnable block somewhere else in the file is not the
    /// twin of this fence, and the path it names has to be one a caller could write, which is what
    /// `vitui_runtime::` (or the engine, for a seam case) spells.
    ///
    /// This is the test that found [`crate::ctx::Frame::tab_walk`], the corpus's one unpaired item:
    /// two fences about a focus verb that does not exist and a ring that cannot be sorted, and
    /// neither of them named `tab_walk` or `ring`.
    ///
    /// # What "names the protected item" is, precisely
    ///
    /// The first version of this test asked only whether the twin contained `vitui_runtime::`,
    /// which **every twin's first `use` line satisfies unconditionally** — a twin made of one
    /// import and an unrelated assertion passed, and the rename it exists to catch went through
    /// green. That is a gate weaker than its own contract, which on this file is the same defect as
    /// no gate at all.
    ///
    /// So the item's own name is extracted from the declaration the doc run is attached to, and the
    /// twin has to contain it. A run whose item cannot be named — a `//!` module header — still has
    /// to carry a path, because there is nothing narrower to ask of it.
    #[test]
    fn every_negative_case_has_a_twin_that_names_a_path() {
        let mut checked = 0;
        for file in src_files() {
            let source = std::fs::read_to_string(&file).expect("a readable module");
            for (item, run) in doc_runs(&source) {
                let blocks = doc_blocks(&run);
                if !blocks.iter().any(|(a, _)| is_negative(a)) {
                    continue;
                }
                checked += 1;
                let twins: Vec<&String> = blocks
                    .iter()
                    .filter(|(a, _)| is_runnable(a))
                    .map(|(_, body)| body)
                    .collect();
                let names_a_path = twins
                    .iter()
                    .any(|b| b.contains("vitui_runtime::") || b.contains("vitui_engine::"));
                assert!(
                    names_a_path,
                    "in {}: the item `{}` carries a `compile_fail` case with no runnable twin \
                     naming a path. A lone `compile_fail` also passes when the protected item has \
                     been renamed",
                    file.display(),
                    item.as_deref().unwrap_or("(module)")
                );
                // What the pair protects: the run's own declaration, unless it says otherwise.
                let protects =
                    protects_line(&run).unwrap_or_else(|| item.iter().cloned().collect());
                assert!(
                    !protects.is_empty(),
                    "in {}: a `compile_fail` case sits on a declaration this scan cannot name and \
                     carries no `**Protects:**` line, so nothing says what a rename would move",
                    file.display()
                );
                for name in protects {
                    assert!(
                        twins.iter().any(|b| b.contains(&name)),
                        "in {}: a `compile_fail` case protects `{name}` and no twin names it. A \
                         twin that exercises the mechanism without naming the item leaves a rename \
                         green — which is refinement 3, and a `use` line is not enough to satisfy \
                         it",
                        file.display()
                    );
                }
            }
        }
        assert!(
            checked >= 17,
            "only {checked} items were checked, so this test has stopped finding the corpus"
        );
    }

    /// **Every negative case declares the error it expects — and rustdoc does not check the
    /// declaration, which is a finding of this ticket rather than a claim it makes.**
    ///
    /// The obvious sentence to write here is the one four documents in this repository briefly
    /// carried: *a bare `compile_fail` passes for any error, so naming the code is what makes the
    /// fence about the property rather than about compiling at all.* **The first half is true and
    /// the second does not follow**, because rustdoc on stable ignores the code. Measured on
    /// rustc 1.97.1 rather than argued — two fences, both written ` ```compile_fail,E0599 `, one
    /// body an unresolved import (`E0432`) and the other a type mismatch (`E0308`):
    ///
    /// ```text
    /// test ecode.rs - probe (line 5) - compile fail ... ok
    /// test ecode.rs - probe (line 9) - compile fail ... ok
    /// ```
    ///
    /// So this is a **convention gate and not a compile outcome**, and it is named
    /// `declares` rather than `names` for that reason. What it still buys is real and small: the
    /// code records which error the author meant, so a reader diffing a fence can see whether the
    /// hostile line still fails for the reason it was written for. What it does **not** buy is the
    /// one thing a compile outcome would — a fence whose `use` line has rotted still reports `ok`,
    /// and only [`tests::every_negative_case_has_a_twin_that_names_a_path`] stands against that,
    /// because a rotted path in the **twin** is a hard failure.
    ///
    /// **The instrument that would close it is deliberately not built.** It is an out-of-band
    /// compile: extract each body, invoke `rustc --error-format=json` against this crate's rlib,
    /// and match the emitted code. Spec §20 rules that out in as many words — *no `RUSTFLAGS`, no
    /// second job, no `trybuild`* — and locating a fresh rlib from inside a test is exactly the
    /// kind of machinery that sentence refuses. Recorded here rather than left as an idea, so the
    /// next person to want it can see it was considered and what it costs.
    #[test]
    fn every_negative_case_declares_the_error_it_expects() {
        for file in src_files() {
            let source = std::fs::read_to_string(&file).expect("a readable module");
            for (attributes, _) in doc_blocks(&source) {
                if !is_negative(&attributes) {
                    continue;
                }
                let declared = attributes.split(',').map(str::trim).find(|a| {
                    // `E` followed by exactly four digits, and nothing else. `,E`, `,Edition` and
                    // `,Everything` all satisfied the substring test this replaces.
                    a.len() == 5 && a.starts_with('E') && a[1..].bytes().all(|b| b.is_ascii_digit())
                });
                assert!(
                    declared.is_some(),
                    "in {}: `{attributes}` declares no well-formed error code. It must carry \
                     exactly one `EXXXX`, and note that rustdoc does not check it — see this \
                     test's documentation",
                    file.display()
                );
            }
        }
    }

    /// **Every row of this register is a gate, and that is a claim rather than an accident.**
    ///
    /// Spec §20 keeps two lists: the fifteen-row gate table, and a paragraph of *reports, committed
    /// and gating nothing* — every `examples/*_numbers.rs`, the headroom ledger, the comparative
    /// suite, the idle-cost measurement. This file is the first list. The reports have two homes
    /// already: [`Instrument::Report`], which is how a gate cites the number a human reads beside
    /// it, and [`crate::scenes`], where a scene may be nothing but a report.
    ///
    /// Asserting it here is what makes [`Kind::Test`] and [`Kind::Report`] honest arms rather than
    /// dead ones: they exist so a future row can say *this is not a gate*, and this test is what
    /// that row would have to change.
    #[test]
    fn every_row_of_this_register_is_a_gate() {
        for entry in REGISTER {
            assert_eq!(
                entry.kind,
                Kind::Gate,
                "entry #{} is a {}. That is allowed, but it is a change to what this file is: the \
                 reports live in `crate::scenes` and in §20's own paragraph",
                entry.number,
                entry.kind.word()
            );
        }
    }

    /// The register prints as a table, which is how a human reads it.
    ///
    /// A test rather than a report, because the `Debug` derive above is the only other way to look
    /// at this value and it is unreadable at 38 rows.
    #[test]
    fn the_register_prints() {
        use std::fmt::Write as _;
        let mut out = String::new();
        for entry in REGISTER {
            let state = match entry.state {
                State::Wired { by } => format!("wired, {} instrument(s)", by.len()),
                State::Red { inverted_by, .. } => format!("red against {inverted_by}"),
            };
            writeln!(
                out,
                "{:>2}  {:<6}  {:<8}  {}",
                entry.number,
                entry.kind.word(),
                entry.source,
                state
            )
            .expect("a string");
        }
        assert_eq!(out.lines().count(), REGISTER.len());
    }
}
