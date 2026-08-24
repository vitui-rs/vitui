//! Spec §14's register: twenty-seven properties, each one either wired or pinned red — and one
//! more that §14 could not have had.
//!
//! > **A gate is a count, a ratio, an equality or a compile outcome. A timing is a report, and is a
//! > gate only at cliff granularity, with the headroom written next to the number.**
//!
//! Almost every defect the architecture map found was catchable without a stopwatch, and this list
//! says so: the 37.07x overdraw cliff is a ratio, the 27x scroll-detector regression was visible
//! only because numbers were kept per scene, the 284x wire regression is a byte count, zero wakeups
//! is a count, byte-identical packed cells is an equality. **A 10% timing regression is not
//! detectable on a shared runner, and a gate that claims to detect one is a flaky test wearing a
//! budget's clothes.**
//!
//! Three refinements the runtime and components maps sent back, folded in rather than left as
//! folklore:
//!
//! 1. **A gate is an equality only when the number is a property of the mechanism**, not of the
//!    data. A number that belongs to the data must be a relation, or it becomes a gate that is
//!    edited rather than fixed.
//! 2. **A report may not be load-bearing for a gate.**
//! 3. **The budget catches a cliff; a growth ratio catches a slope**, and neither substitutes for
//!    the other.
//!
//! # Why the list is in the repository rather than in the spec only
//!
//! §14 closed with "the gate list has twenty-seven entries and nothing to run them against". A
//! property that is only in a document is not verified by anything, and — worse — one that quietly
//! never arrives is indistinguishable from one that was decided against. So every entry is here,
//! and every entry is in exactly one of two states: **wired**, naming where it runs, or **red**,
//! naming the implementation ticket that inverts it. **No entry is silently absent**, and the test
//! at the bottom of this file is what keeps that true.
//!
//! `source` names the **architecture** ticket the property came from
//! (`.scratch/vitui-engine-architecture/issues/`); `inverted_by` names the **implementation**
//! ticket that lights it (`.scratch/vitui-engine-impl/issues/`). The two are different numbering
//! schemes and confusing them sends a reader to the wrong document.
//!
//! # Entry 28, and why the list is no longer exactly §14's
//!
//! Entries 1–27 are §14's table. **Entry 28 is not**, and it is here rather than in a document
//! because of what it is about: §14 could enumerate twenty-seven properties of the engine and had
//! no entry for *whether the engine's bytes mean to a real terminal what they mean to the engine's
//! model of one*. Architecture ticket 20 is where that gap was noticed, from the inside:
//!
//! > Every gate stays green with §3's pairing invariant **false**, because the serializer and the
//! > terminal model are wrong in the same direction.
//!
//! A register whose whole purpose is that *a property which quietly never arrives is
//! indistinguishable from one that was decided against* cannot answer that by staying at
//! twenty-seven. The count test below therefore asserts the split rather than the total, so a
//! twenty-ninth entry has to say which side of the line it is on.

/// What a register entry costs when it disagrees with the code.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// Fails the build.
    Gate,
    /// An ordinary assertion that happens to be cheap.
    Test,
    /// Prints a number a human reads. May never be load-bearing for a gate.
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

/// Whether something on §14's register runs today — an entry of the gate list, or a scene of the
/// normative list the gates are driven over.
///
/// One type for both, because it is one statement: this runs **here**, or it does not run and
/// **that implementation ticket** is what lights it. There is no third arm, and that is the point —
/// something silently absent is indistinguishable from something that was decided against.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum State {
    /// It runs, here.
    Wired {
        /// Where it runs.
        at: &'static str,
    },
    /// It does not run, and the implementation ticket that makes it possible is named.
    ///
    /// **Nothing constructs this any more, and impl 26 is where that became true**: entry #27, the
    /// comparative suite, was the last red row on §14's register, and the suite has now been run.
    /// All twenty-seven are wired.
    ///
    /// `expect(dead_code)` rather than deletion, and the reason is the whole design of this file.
    /// The register exists because **a property that quietly never arrives is indistinguishable from
    /// one that was decided against**, and this arm is how a new property says *not yet, and here is
    /// the ticket*. Deleting it would leave the next entry with only one thing it could be, which is
    /// the pressure that produces a row claiming to be wired somewhere vague. The two-sided
    /// assertion in [`State::assert_names_a_destination`] is still compiled and still what a red row
    /// would have to satisfy.
    #[expect(
        dead_code,
        reason = "every entry is wired since impl 26; the arm stays so a new one can be red"
    )]
    Red {
        /// The implementation ticket that inverts this entry.
        inverted_by: &'static str,
        /// What is missing.
        why: &'static str,
    },
}

impl State {
    /// Both arms must name somewhere to look, and a red one must name an **implementation** ticket.
    ///
    /// `source` on an [`Entry`] is the architecture ticket a property came from and `inverted_by`
    /// is the implementation ticket that lights it; the two are different numbering schemes and
    /// confusing them sends a reader to the wrong document.
    #[cfg(test)]
    pub fn assert_names_a_destination(self, what: &str) {
        match self {
            State::Wired { at } => {
                assert!(!at.is_empty(), "{what} is wired without saying where")
            }
            State::Red { inverted_by, why } => {
                assert!(
                    inverted_by.starts_with("impl "),
                    "{what} is red against `{inverted_by}`, which is not an implementation ticket"
                );
                assert!(!why.is_empty(), "{what} is red without a reason");
            }
        }
    }
}

/// One property of spec §14's register.
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    /// Its number in §14's table, which is how everything else refers to it.
    pub number: u8,
    /// The property, in §14's own words.
    pub property: &'static str,
    /// Gate, test or report.
    pub kind: Kind,
    /// What shape of gate: a count, a ratio, an equality, a compile outcome.
    pub qualifier: &'static str,
    /// The architecture ticket the property came from.
    pub source: &'static str,
    /// Wired, or red with the implementation ticket that inverts it.
    pub state: State,
}

/// Spec §14's register, entry for entry.
pub const REGISTER: [Entry; 28] = [
    Entry {
        number: 1,
        property: "No damage structure under-reports",
        kind: Kind::Gate,
        qualifier: "property, generated from the reference compositor",
        source: "arch 07",
        state: State::Wired {
            at: "crate::gates::no_damage_structure_under_reports over §14's twelve scenes, and \
                 `crate::fuzz::draw_sequence` over every input of a committed corpus — the same two \
                 halves of the same property, generated from the same oracle. The second is what \
                 §14's fuzzing section asks for, and impl 25 wired it: the corpus replayed as an \
                 ordinary test is the gate and the fuzzer is a soak. It found a fourth defect in \
                 the operator's reach on its first run, which is the argument for the entry having \
                 two homes rather than one",
        },
    },
    Entry {
        number: 2,
        property: "Runs arrive in serializer order",
        kind: Kind::Gate,
        qualifier: "ordering",
        source: "arch 07",
        state: State::Wired {
            at: "crate::gates::runs_arrive_in_serializer_order",
        },
    },
    Entry {
        number: 3,
        property: "Overdraw is 1.00x on every scene",
        kind: Kind::Gate,
        qualifier: "ratio, per scene",
        source: "arch 07",
        state: State::Wired {
            at: "crate::gates::overdraw_is_one_times_on_every_scene",
        },
    },
    Entry {
        number: 4,
        property: "Zero allocations over 1 000 compose cycles",
        kind: Kind::Gate,
        qualifier: "count",
        source: "arch 04, 09",
        state: State::Wired {
            at: "tests/alloc.rs::the_steady_state_allocates_nothing",
        },
    },
    Entry {
        number: 5,
        property: "Zero allocations over 1 000 lease-pack-submit-take-finish",
        kind: Kind::Gate,
        qualifier: "count",
        source: "arch 09",
        state: State::Wired {
            at: "tests/handoff.rs::the_steady_state_of_the_handoff_allocates_nothing, first \
                 phase — on the real three-thread path, because that is the only place all five \
                 of those words exist: lease, pack and submit on the app thread and take and \
                 finish on the render thread. It is also the one allocation gate in this \
                 repository with a thread inside its window, and its module documentation argues \
                 §14's attribution rule rather than avoiding it: what the rule forbids is work \
                 the gate is not about, and here both threads are the subject",
        },
    },
    Entry {
        number: 6,
        property: "`pack` allocates zero on warm tables, at every density",
        kind: Kind::Gate,
        qualifier: "count",
        source: "arch 16",
        state: State::Wired {
            at: "tests/handoff.rs::the_steady_state_of_the_handoff_allocates_nothing, second \
                 phase — §7's four densities, none of the 24 000 cells carrying a handle through \
                 to every one of them naming a distinct entry. What it is about is impl 18's \
                 generation-stamped marker, whose slots grow to the handle tables' high-water \
                 mark and never again; the `HashMap` it replaced would have passed the plain arm \
                 and failed the dense ones. Asserted over the whole frame rather than over `pack` \
                 alone, which is stronger and is the only shape reachable from outside the crate",
        },
    },
    Entry {
        number: 7,
        property: "A settled operator allocates zero; a fading one allocates per distinct extended \
                   style and never per cell",
        kind: Kind::Gate,
        qualifier: "shape",
        source: "arch 16",
        state: State::Wired {
            at: "tests/alloc.rs's a_settled_operator_over_a_hyperlinked_screen_allocates_nothing \
                 for the settled half, and \
                 crate::gates::a_fading_operator_mints_per_distinct_style_and_never_per_cell for \
                 the fading one",
        },
    },
    Entry {
        number: 8,
        property: "The pool does not starve at 10 000 cycles",
        kind: Kind::Gate,
        qualifier: "count",
        source: "arch 09",
        state: State::Wired {
            at: "crate::gates::a_packet_is_never_superseded_and_the_pool_of_two_does_not_starve, \
                 10 000 cycles on the real three-thread path. Two is provably enough rather than \
                 empirically: the app may fill one while the renderer holds the other, and the \
                 ready gate is what stops a third ever being wanted",
        },
    },
    Entry {
        number: 9,
        property: "A packet is never superseded",
        kind: Kind::Gate,
        qualifier: "count",
        source: "arch 09",
        state: State::Wired {
            at: "crate::gates::a_packet_is_never_superseded_and_the_pool_of_two_does_not_starve, \
                 the same 10 000 cycles. **This is the backpressure decision and not a health \
                 check**: if the counter ever leaves zero, the pacing gate has moved off the app \
                 thread and frames are being composed in order to be thrown away — a defect that \
                 costs 107 us a frame with the screen still correct, which is why it is a count \
                 and not a stopwatch. crate::handoff::Mailbox::submit is where the path it counts \
                 lives, reachable only by ignoring the lease's answer",
        },
    },
    Entry {
        number: 10,
        property: "A packed cell is byte-identical to the surface cell",
        kind: Kind::Gate,
        qualifier: "equality",
        source: "arch 16",
        state: State::Wired {
            at: "crate::gates::a_packed_cell_is_byte_identical_to_the_surface_cell — and since \
                 impl 14 the equality filter is what rests on it: the filter compares a packed cell \
                 against the mirror, so a cell that packed differently from the surface it came from \
                 compares unequal for ever and the wire is 284x fatter with the screen still correct",
        },
    },
    Entry {
        number: 11,
        property: "After a sweep every live cell resolves to the same channels; `renumbered` is \
                   false when nothing below a live entry was freed",
        kind: Kind::Gate,
        qualifier: "equality",
        source: "arch 16",
        state: State::Wired {
            at: "crate::gates::a_sweep_preserves_every_live_cells_channels and \
                 a_sweep_that_frees_only_the_top_of_a_table_does_not_renumber, with the interner's \
                 half in a_sweep_preserves_a_wide_clusters_channels_and_its_width",
        },
    },
    Entry {
        number: 12,
        property: "Round trip: replayed screen == composited frame",
        kind: Kind::Gate,
        qualifier: "equality",
        source: "arch 08, 16",
        state: State::Wired {
            at: "crate::roundtrip, crate::gates::the_round_trip_closes_on_every_scene — all twelve \
                 scenes since impl 13 took the twelfth off red — and \
                 crate::serial::tests::the_round_trip_closes_under_every_wire_configuration, which \
                 is the sixteen combinations of the two SGR spellings, the two underline-colour \
                 spellings, mode 2026 and OSC 8. Impl 14 puts every scene through it four more \
                 times: crate::gates::steady_bytes drives the filter's four configurations through \
                 the same Harness, so a variant that skipped a cell it may not have skipped fails as \
                 a wrong screen rather than as a small number in a report. \
                 **Production ticket 10 adds the axis the quirk table owns**: \
                 crate::roundtrip::the_round_trip_closes_on_a_terminal_that_drops_an_attribute is \
                 the round trip on a tmux-identified terminal, where the bytes deliberately do not \
                 carry a bit the frame asked for — so the expectation is narrowed by \
                 quant::OnTheWire for the one flag and for no other, and the sibling \
                 a_scroll_is_taken_on_a_terminal_that_drops_an_attribute is beside it because \
                 Quantiser::narrows deciding this axis wrongly costs a scroll rather than a colour",
        },
    },
    Entry {
        number: 13,
        property: "A presses-only terminal never yields `Release` or `Repeat`",
        kind: Kind::Gate,
        qualifier: "absence",
        source: "arch 10, ADR 0007",
        state: State::Wired {
            at: "crate::gates::a_presses_only_terminal_never_yields_release_or_repeat — every \
                 legacy shape there is, and the same keys again from a terminal that has kitty \
                 flag 2, so that the absence is a property of the wire rather than of a parser \
                 that produces nothing",
        },
    },
    Entry {
        number: 14,
        property: "Motion floods collapse to one per wake; press floods do not collapse",
        kind: Kind::Gate,
        qualifier: "count",
        source: "arch 10, ADR 0008",
        state: State::Wired {
            at: "crate::gates::motion_floods_collapse_and_press_floods_do_not — 1 000 motion \
                 reports are one event, 1 000 keystrokes are 1 000, and five wheel notches are \
                 five, all in the same test so that the asymmetry cannot be half deleted",
        },
    },
    Entry {
        number: 15,
        property: "Panic restore pops keyboard flags and disables mouse, focus reporting and \
                   bracketed paste",
        kind: Kind::Gate,
        qualifier: "byte sequence",
        source: "arch 10, 01",
        state: State::Wired {
            at: "crate::gates::a_panic_restores_the_terminal_before_the_backtrace_prints — a child \
                 process that declared all four, raised the mouse above its floor and then \
                 panicked, with its two output streams on one open file so that the parent reads \
                 them in write order. The whole epilogue is captured: the kitty pop, all three \
                 input modes, the caret, auto-wrap and the alt screen. It is there **before** the \
                 panic message rather than painted into a page the terminal is about to discard, \
                 and it is there **once** although both the hook and the unwinding `Screen`'s \
                 `Drop` ran",
        },
    },
    Entry {
        number: 16,
        property: "Parser survives five adversarial splits",
        kind: Kind::Test,
        qualifier: "cheap assertion",
        source: "arch 10",
        state: State::Wired {
            at: "crate::gates::the_parser_survives_five_adversarial_splits — a kitty key with \
                 text, an SGR mouse press, a bracketed paste, a modified arrow and a three-byte \
                 scalar, each cut at every internal boundary; the one index that legitimately \
                 differs is the bare `ESC`, asserted two-sided",
        },
    },
    Entry {
        number: 17,
        property: "Idle: zero wakeups, `0.00 user 0.00 sys`",
        kind: Kind::Gate,
        qualifier: "count, 30 s minimum per run",
        source: "arch 09, 18",
        state: State::Wired {
            at: "in two halves, because the two numbers need two instruments. The **count** is \
                 crate::gates::an_idle_application_parks_once_and_wakes_for_nothing — the app \
                 thread enters its blocking wait once and comes back zero times, the render \
                 thread's wait comes back zero times, and no frame is painted; a 120 Hz ticker \
                 would put 18 in the second number over the 150 ms window and 3 600 over thirty \
                 seconds. The **process** half is `scripts/idle-gate.sh`, which runs \
                 `examples/idle.rs` under /usr/bin/time for thirty seconds and reads user time, \
                 system time and voluntary context switches out of the report: measured \
                 `30.01 real, 0.00 user, 0.00 sys, 0 voluntary context switches`. Thirty is a \
                 floor rather than a preference — at three the difference is below the tool's \
                 resolution — and it is a two-runner job, `-l` on macOS and `-v` on GNU, because \
                 Windows has neither",
        },
    },
    Entry {
        number: 18,
        property: "No observer thread linked into a release binary",
        kind: Kind::Gate,
        qualifier: "compile outcome",
        source: "arch 18",
        state: State::Wired {
            at: "scripts/observer-gate.sh, which is a compile outcome about *absent* code and \
                 therefore checked in the one place absent code is visible — the binary. The \
                 observer's sanction message is a string literal in the only function that prints \
                 it, so the literal is in `target/*/examples/idle` if and only if the code is, and \
                 the script asserts it is in the debug build and not in the release one. Both \
                 directions, because a grep that finds nothing passes for any reason including the \
                 message having been reworded. The observer's *behaviour* is \
                 crate::gates::the_observers_sanction_restores_the_terminal_then_prints_the_stall_then_aborts, \
                 a child process that reads the order of three events out of one open file",
        },
    },
    Entry {
        number: 19,
        property: "The negative cases do not compile",
        kind: Kind::Gate,
        qualifier: "compile outcome, paired doctests",
        source: "arch 18, 13",
        state: State::Wired {
            at: "the paired doctests on Screen, Config, View, LayerStack, Slot, Permit, Mix, \
                 Style, Overrides, Capabilities and the crate root — **thirty-seven** \
                 compile_fail cases, each with a positive twin that names the protected item by \
                 path, because a compile_fail alone passes for any reason including the type having \
                 been renamed, at which point it fails for E0433 instead of E0277 and this \
                 mechanism cannot tell those apart. Impl 23 added §11's six: five E0277 on \
                 `PhantomData<*const ()>` and one on the escape hatch, where the marker the ticket \
                 predicted (`Cell<()>`) became `Rc<Perf>` when `Permit` had to stop borrowing the \
                 `Screen`. Impl 24 completed it with twenty more — one per refusal that has a type \
                 to hang on, three on the crate root for the ones that do not, `Overrides.mouse` \
                 for *a declaration cannot make an event arrive*, and `Config.packets` / \
                 `Config.resolver` for the two of §12's six fields that the implementation \
                 settled away — and **both** counts are gates in `crate::audit`, the thirty-seven \
                 hostile lines and the forty-six runnable examples beside them, since a case \
                 deleted together with its twin leaves every other test green and a twin deleted \
                 alone leaves the rename undetectable. Arch 21 moved both: `LinkId`'s own case went \
                 with the type, two took its place on the crate root — the type and the mint that \
                 made one — and `Screen::link`'s runnable example went with the verb",
        },
    },
    Entry {
        number: 20,
        property: "1M elements cost the same as 1k",
        kind: Kind::Gate,
        qualifier: "ratio in a stated band",
        source: "arch 05, 14",
        state: State::Wired {
            at: "examples/budget.rs, the `tree/*` and `table/*` cases — both scenes now cull \
                 through `View::scrolled` and `visible_rows`, so the gate is over the loop every \
                 component will write rather than over one the scene hand-bounded (impl 09)",
        },
    },
    Entry {
        number: 21,
        property: "Wire bytes per scene",
        kind: Kind::Gate,
        qualifier: "count, an upper bound per scene",
        source: "arch 08, 16",
        state: State::Wired {
            at: "crate::gates::wire_bytes_per_scene, twelve rows re-measured by impl 15 with the \
                 scroll region on — four fell again, by 2.2x to 2.6x over three frames and by 30x \
                 to 85x on the one frame of the three that is a steady scroll, and two of the four \
                 are scenes §8 did not name. The per-scene four-column breakdown against §8's own \
                 table is crate::gates::the_equality_filter_reproduces_spec_8s_table, measured with \
                 the pre-pass off so that it stays a claim about the filter, and \
                 crate::gates::which_of_spec_14s_twelve_the_scroll_region_reaches is the same twelve \
                 with it on",
        },
    },
    Entry {
        number: 22,
        property: "Overrun detector under 50 ns",
        kind: Kind::Gate,
        qualifier: "timing",
        source: "arch 18",
        state: State::Wired {
            at: "examples/budget.rs, `the_overrun_detector` — and **the 50 ns is the report rather \
                 than the gate**, which is this register's own rule applied to its own entry. \
                 Measured **43.9 to 46.9 ns across four runs on one unloaded M1** against a 50 ns \
                 budget: 1.07x to 1.14x. **And 50.85 ns on the GitLab runner, over the budget \
                 on the very first pipeline that ran it** — which settles the argument rather than \
                 making it: a 50 ns gate would have been red on the commit that introduced it, for a \
                 detector that is doing exactly what it was measured to do. The 3.0 ns spread \
                 between runs on one machine said the same thing more quietly. What is gated is two properties of the mechanism instead — a \
                 **ratio**, that the pair costs under 2x the two `Instant::now()` calls it is built \
                 out of (1.25x measured), which is what fails when an allocation, a lock, a syscall \
                 or a format lands on the frame path and is immune to a slow clock because the \
                 baseline moves with it; and a **cliff** at 1 µs, 1% of §13's typical frame and 20x \
                 the measurement. The absolute number is printed with its headroom on every run, \
                 and impl 26's ledger is where it becomes a gate against a machine somebody has \
                 measured",
        },
    },
    Entry {
        number: 23,
        property: "Full-screen 300x80 under 1 ms",
        kind: Kind::Gate,
        qualifier: "timing, at the budget",
        source: "budget",
        state: State::Wired {
            at: "examples/budget.rs, every scene of the full-screen class. The scenes on \
                 REPORTED_NOT_GATED named impl 18, and impl 18 delivered the number rather than \
                 the gate: `the_app_threads_share` measures `present` with serialisation on the \
                 render thread, which is what §13's budget is written about, and all four come \
                 under their budgets there. They stay reported because a sample on that path can \
                 contain a wait — that loop spins rather than parking in impl 19's `wait`, since a \
                 frame gap inside a sample is a sleep inside a budget — and because a budget gate \
                 has to hold on a runner somebody has measured, which is impl 26's ledger. **Impl \
                 26 measured it and left all four reported**, which is the outcome the sentence \
                 above was written to allow: on the M1 Max every one of the four is under its \
                 budget on the threaded path — 704.09, 58.02, 57.04 and 532.00 us, headroom 1.42x \
                 to 1.88x — and the thinnest of those margins is inside the spread this list has \
                 already seen between this machine and the runner (113.9 us against 75.7 us on \
                 `virtualised-tree`, pipeline 41). A row promoted on the strength of one machine's \
                 report is the same mistake as a row exempted on the strength of an expectation, \
                 and one machine is what impl 26 had. **The budget figure itself now has one home**: \
                 `crate::ledger::full_screen_budget_ns`, because the audit found it written four \
                 times in two files and a budget figure may not move without a new map decision. \
                 Provenance: impl 26, Apple M1 Max, macOS 26.5.2, rustc 1.97.1, --release, unloaded",
        },
    },
    Entry {
        number: 24,
        property: "Typical damage-tracked frame under 100 us",
        kind: Kind::Gate,
        qualifier: "timing, at the budget",
        source: "budget",
        state: State::Wired {
            at: "examples/budget.rs, every scene of the incremental class, the caret at forty \
                 layers among them. The figure has one home, `crate::ledger::incremental_budget_ns`, \
                 for the reason on #23. Provenance: impl 26, Apple M1 Max, macOS 26.5.2, rustc \
                 1.97.1, --release, unloaded — worst headroom gated here is 1.6x on \
                 `scrolling-list-label-only`",
        },
    },
    Entry {
        number: 25,
        property: "60 fps steady state under 5% of a core",
        kind: Kind::Report,
        qualifier: "a process measurement over a real 60 Hz loop; the derived figure is kept beside it",
        source: "budget",
        state: State::Wired {
            at: "**measured, by impl 26**: `examples/steady.rs` and `scripts/steady-report.sh` \
                 run the real three-thread 60 Hz loop for thirty seconds and read /usr/bin/time — \
                 **0.133% of one core over 1 800 frames**, 37.6x of headroom against the 5% budget, \
                 22.2 us of process CPU a frame with every thread and the wire in it. The derived \
                 arithmetic stays beside it in examples/budget.rs and read **0.0029%**, so it was \
                 46x optimistic — and it was not wrong about anything it contained: what it left \
                 out is the whole of what the word *core* covers, the render thread that wakes \
                 sixty times a second to composite nothing, the serializer and the write on that \
                 thread, and sixty condvar round trips of scheduler time. The gate under the report \
                 is a frame **count** inside examples/steady.rs, because a percentage taken over \
                 four frames is not a measurement of a steady state. Provenance: impl 26, Apple M1 \
                 Max, macOS 26.5.2, rustc 1.97.1, --release, unloaded",
        },
    },
    Entry {
        number: 26,
        property: "Wake-up latency p50 / p99 / max",
        kind: Kind::Report,
        qualifier: "distribution over a real run",
        source: "arch 09",
        state: State::Wired {
            at: "crate::gates::what_a_wake_up_costs_with_the_app_thread_parked, stamped in \
                 crate::handoff on submit and on take, with the app thread parked in `wait` for \
                 every sample — which is what impl 19 added and impl 18 could not do. Beside it, \
                 crate::gates::what_a_leading_edge_costs_after_a_quiet_period reports the same \
                 hop from the other end and **in two arms**, which is a correction rather than a \
                 flourish: §7's 250 ns p50 is the leading edge with a reason already pending, so \
                 `wait` never touches the condvar, and a genuinely parked app thread costs a \
                 scheduler hop at the same order as §7's own 4.58 µs handoff figure. Two \
                 quantities, and printing only the first would advertise 250 ns for something \
                 that costs twenty times it. Reports, not gates: this is OS scheduler latency on \
                 the way to the wire and it does not spend §13's 100 µs of app-thread CPU",
        },
    },
    Entry {
        number: 27,
        property: "Comparative suite",
        kind: Kind::Report,
        qualifier: "committed file, on a pinned runner",
        source: "arch 11, 13",
        state: State::Wired {
            at: "`compare/` — a detached workspace, `SCENES.md` normative, `harness.py` the \
                 instrument, `REPORT.md` committed and regenerated, `FINDINGS.md` written by hand \
                 and dated. **Run on 2026-08-22 by impl 26, which is §15's fifth owed measurement \
                 paid**, and the ticket's prediction was right: *expect the \"same scene\" \
                 definition to break first.* It broke in eleven places, two of them genuine \
                 framework limits and nine of them SCENES.md under-specifying — the worst being \
                 scene 5's dim, which had no colour to be halfway *from*, so it was a silent no-op \
                 rather than a disagreement. All eleven are recorded at the bottom of SCENES.md. \
                 The reading of the numbers is FINDINGS.md and the headline is a method finding: a \
                 per-scene **total** over 120 frames reversed the caret row, because a total is a \
                 weighted mean of a one-off and a steady state, so the report leads with the \
                 marginal cost of one frame and keeps the total underneath it. Two arms are not \
                 built on the development machine — notcurses, whose Homebrew formula pulls \
                 ffmpeg, and Textual — and their cells read `not built here` with the reason, \
                 which is deliberately **not** the same cell as `cannot express`: a missing row \
                 reads as a win and that is the single easiest way for this suite to become \
                 dishonest. **And the one `cannot express` cell on the table carries a \
                 withdrawal**, which is the sharpest thing this ticket found: arch 13 says \
                 notcurses does not composite layers, and it does — `NCALPHA_BLEND` over the plane \
                 stack, built from the 3.0.17 tarball and demonstrated, halving a plane below \
                 without the caller computing one dimmed colour. So the rule written to stop the \
                 suite being dishonest would itself have shipped a cell claiming a competitor \
                 cannot do something it can. The arm still refuses the scene, because SCENES.md is \
                 normative and flipping that cell is the map's call; the evidence is appended to \
                 arch 13 and the withdrawal sits beside the cell in FINDINGS.md and README.md. `.github/workflows/compare.yml` is the pinned-runner home where all \
                 four arms build, scheduled monthly and never on a pull request, uploading its \
                 report rather than pushing one. **It reports; it does not block** — four external \
                 projects' versions cannot gate this repository's pull requests, and what makes \
                 the requirement falsifiable instead is that REPORT.md is committed, so a \
                 worsening number arrives as a review-visible diff",
        },
    },
    Entry {
        number: 28,
        property: "Conformance against a real terminal emulator",
        kind: Kind::Report,
        qualifier: "committed file, on a machine with a window server",
        source: "arch 20",
        state: State::Wired {
            at: "`conform/` — a detached workspace, `SCENES.md` normative,                  `cargo run --example ghostty` and `--example tmux` the instruments, one committed and                  regenerated `REPORT-<arm>.md` per arm — one file for two arms would delete the                  rows of whichever ran first, and a missing row reads as a win — `FINDINGS.md` written by hand and dated. **The first instrument in                  this repository that asks a terminal rather than our model of one.** Every other                  entry above is checked by code that lives in this crate: `roundtrip` replays the                  serializer's bytes through `term_model` and `testing` asserts a three-way                  agreement between the frame, `serial::Mirror` and `TermModel` — **two of those                  three are this engine's own code**, which is arch 20's finding and the reason                  this row exists. Scene 01 is the eleven attribute bits of the style word, one per                  row, because `attrs_dropped` is the field with **no query**: the eleven facts are                  in the capability set precisely because nothing can ask a terminal for them, and                  a screen dump is the only thing that can. Ghostty 1.3.1 agreed 11/11 on                  2026-08-23. **Production ticket 05 added the second family and it disagreed**:                  scene 01 through tmux 3.7c into Ghostty is 10/11, because tmux accepts SGR 53,                  stores it in the cell, hands it back to `capture-pane` and never puts it on the                  wire. That is the fourth `quirks.rs` entry and the first this repository                  gathered rather than inherited, and it took three captures of one scene to                  attribute — Ghostty direct, tmux's own grid, and tmux forwarded through Ghostty.                  All three are committed to `conform/fixtures/` where                  `cargo test` compares them with no emulator, no window server and no automation                  grant in the loop — the same trade `fuzz/` makes, the committed corpus being the                  gate and the live run the soak. **It reports; it does not block**, and here that                  is stronger than in `compare/`: the run needs a macOS automation grant a fresh                  runner cannot have, and the `vt` dump format it reads is undocumented, found by                  probing `+validate-config`. A worsening result arrives as a review-visible diff                  in a committed report. **The instrument's own second defect was in the                  instrument**, which is the pattern this row should be read for: the parser read                  one capture format as the other, and tmux writes any two-digit attribute code as                  `code/10 : code%10`, so overline arrived as `5:3` and was reported as *blink* —                  an attribute tmux never rendered. `parse` takes a `Dialect` now, and it has no                  default. **What it cannot see is written down beside it**: a                  grid-to-text dump emits a double-width glyph with no padding cell, so arch 20's                  own question is not answerable by this instrument and waits on production ticket                  06's sentinels or on CPR",
        },
    },
];

/// How many entries are wired, and how many are pinned red.
pub fn tally() -> (usize, usize) {
    let wired = REGISTER
        .iter()
        .filter(|e| matches!(e.state, State::Wired { .. }))
        .count();
    (wired, REGISTER.len() - wired)
}

/// The register as a table, for the example to print.
pub fn table() -> String {
    use std::fmt::Write as _;

    let mut out = String::new();
    for e in REGISTER {
        let mark = match e.state {
            State::Wired { .. } => "wired",
            State::Red { .. } => " RED ",
        };
        let _ = writeln!(
            out,
            "  {:>2}  {mark}  {:<6} {:<14}  {}",
            e.number,
            e.kind.word(),
            e.source,
            e.property
        );
        let _ = writeln!(out, "                {}", e.qualifier);
        match e.state {
            State::Wired { at } => {
                let _ = writeln!(out, "                at {at}");
            }
            State::Red { inverted_by, .. } => {
                let _ = writeln!(out, "                inverted by {inverted_by}");
            }
        }
    }
    let (wired, red) = tally();
    let _ = writeln!(
        out,
        "\n  {wired} wired, {red} pinned red, {} in all",
        REGISTER.len()
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// §14's "the gate list has twenty-seven entries and nothing to run them against" stops being
    /// true here, and this is what keeps it from becoming true again.
    ///
    /// **The assertion is the split, not the total.** Entries 1–27 are §14's table and 28 is the one
    /// production ticket 04 added for a property §14 had no way to state — see the module docs. A
    /// bare length check would let a twenty-ninth entry arrive without anyone deciding which of
    /// those two things it is.
    #[test]
    fn every_entry_of_spec_14s_register_is_present_exactly_once() {
        const FROM_SPEC_14: usize = 27;
        assert_eq!(REGISTER.len(), FROM_SPEC_14 + 1);
        let mut seen = [false; 29];
        for e in REGISTER {
            let n = e.number as usize;
            assert!(
                (1..=REGISTER.len()).contains(&n),
                "entry {n} is neither in §14's table nor an entry this repository added"
            );
            assert!(!seen[n], "entry {n} appears twice");
            seen[n] = true;
        }
        for (n, present) in seen.iter().enumerate().skip(1).take(REGISTER.len()) {
            assert!(present, "entry {n} is silently absent");
        }
        assert_eq!(
            REGISTER[FROM_SPEC_14].source, "arch 20",
            "entry 28 is the conformance suite, and it exists because arch 20 found that every \
             instrument in this crate is checked against code in this crate"
        );
    }

    /// Every entry is in exactly one of two states, and both of them name somewhere to look.
    ///
    /// A red entry without an inverting ticket is indistinguishable from a property that was
    /// decided against, and a wired entry that does not say where it runs is a claim rather than a
    /// gate.
    #[test]
    fn every_entry_is_either_wired_somewhere_or_red_against_a_ticket() {
        for e in REGISTER {
            match e.state {
                State::Wired { at } => assert!(
                    !at.is_empty(),
                    "entry {} is wired without saying where",
                    e.number
                ),
                State::Red { inverted_by, why } => {
                    assert!(
                        inverted_by.starts_with("impl "),
                        "entry {} is red against `{inverted_by}`, which is not an implementation \
                         ticket. `source` is the architecture ticket the property came from and \
                         `inverted_by` is the implementation ticket that lights it; the two are \
                         different numbering schemes.",
                        e.number
                    );
                    assert!(
                        !why.is_empty(),
                        "entry {} is red without a reason",
                        e.number
                    );
                }
            }
            assert!(!e.property.is_empty());
            assert!(!e.qualifier.is_empty());
            assert!(!e.source.is_empty());
        }
    }

    /// A report may never be load-bearing for a gate (§14's second refinement), so the entries
    /// that are reports say so in their own kind rather than in a comment.
    #[test]
    fn the_reports_are_marked_as_reports() {
        let reports: Vec<u8> = REGISTER
            .iter()
            .filter(|e| e.kind == Kind::Report)
            .map(|e| e.number)
            .collect();
        assert_eq!(reports, vec![25, 26, 27, 28]);
    }

    #[test]
    fn the_tally_is_the_register() {
        let (wired, red) = tally();
        assert_eq!(wired + red, REGISTER.len());
        assert!(!table().is_empty());
    }
}
