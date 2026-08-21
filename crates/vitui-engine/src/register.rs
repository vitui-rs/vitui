//! Spec §14's register: twenty-seven properties, each one either wired or pinned red.
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
pub const REGISTER: [Entry; 27] = [
    Entry {
        number: 1,
        property: "No damage structure under-reports",
        kind: Kind::Gate,
        qualifier: "property, generated from the reference compositor",
        source: "arch 07",
        state: State::Wired {
            at: "crate::gates::no_damage_structure_under_reports",
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
                 a wrong screen rather than as a small number in a report",
        },
    },
    Entry {
        number: 13,
        property: "A presses-only terminal never yields `Release` or `Repeat`",
        kind: Kind::Gate,
        qualifier: "absence",
        source: "arch 10, ADR 0007",
        state: State::Red {
            inverted_by: "impl 20",
            why: "there is no input pipeline yet",
        },
    },
    Entry {
        number: 14,
        property: "Motion floods collapse to one per wake; press floods do not collapse",
        kind: Kind::Gate,
        qualifier: "count",
        source: "arch 10, ADR 0008",
        state: State::Red {
            inverted_by: "impl 20",
            why: "there is no input pipeline yet",
        },
    },
    Entry {
        number: 15,
        property: "Panic restore pops keyboard flags and disables mouse, focus reporting and \
                   bracketed paste",
        kind: Kind::Gate,
        qualifier: "byte sequence",
        source: "arch 10, 01",
        state: State::Red {
            inverted_by: "impl 22",
            why: "nothing takes the terminal yet, so nothing has to give it back",
        },
    },
    Entry {
        number: 16,
        property: "Parser survives five adversarial splits",
        kind: Kind::Test,
        qualifier: "cheap assertion",
        source: "arch 10",
        state: State::Red {
            inverted_by: "impl 20",
            why: "there is no input parser yet",
        },
    },
    Entry {
        number: 17,
        property: "Idle: zero wakeups, `0.00 user 0.00 sys`",
        kind: Kind::Gate,
        qualifier: "count, 30 s minimum per run",
        source: "arch 09, 18",
        state: State::Red {
            inverted_by: "impl 19",
            why: "the render thread parks on the mailbox since impl 18, and it is the app thread \
                  that cannot: there is no `wait` to park in, so an idle process still turns in \
                  whatever loop its caller wrote. An idle measured against that is a measurement of \
                  the caller",
        },
    },
    Entry {
        number: 18,
        property: "No observer thread linked into a release binary",
        kind: Kind::Gate,
        qualifier: "compile outcome",
        source: "arch 18",
        state: State::Red {
            inverted_by: "impl 23",
            why: "there is no debug observer yet",
        },
    },
    Entry {
        number: 19,
        property: "The negative cases do not compile",
        kind: Kind::Gate,
        qualifier: "compile outcome, paired doctests",
        source: "arch 18, 13",
        state: State::Wired {
            at: "the paired doctests on Screen, View, Style and Capabilities — nine compile_fail \
                 cases, each with a positive twin, because a compile_fail alone passes for any \
                 reason including the type having been renamed; impl 24 completes the corpus",
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
        state: State::Red {
            inverted_by: "impl 23",
            why: "there is no overrun detector yet",
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
                 contain a wait until impl 19's parking point, and because a budget gate has to \
                 hold on a runner somebody has measured — impl 26's ledger",
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
                 layers among them",
        },
    },
    Entry {
        number: 25,
        property: "60 fps steady state under 5% of a core",
        kind: Kind::Report,
        qualifier: "derived from the typical frame, not a process measurement",
        source: "budget",
        state: State::Wired {
            at: "examples/budget.rs, derived from the typical frame; impl 26 carries this report \
                 in its own criteria and replaces the arithmetic with a measured steady state",
        },
    },
    Entry {
        number: 26,
        property: "Wake-up latency p50 / p99 / max",
        kind: Kind::Report,
        qualifier: "distribution over a real run",
        source: "arch 09",
        state: State::Red {
            inverted_by: "impl 19",
            why: "there is a wake-up to time since impl 18 — app submit to the render thread \
                  holding the packet — and no instrument on the render thread to stamp it, because \
                  the distribution §7 reports is taken with the app thread genuinely parked and \
                  impl 19 is what parks it",
        },
    },
    Entry {
        number: 27,
        property: "Comparative suite",
        kind: Kind::Report,
        qualifier: "committed file, on a pinned runner",
        source: "arch 11, 13",
        state: State::Red {
            inverted_by: "impl 26",
            why: "the suite has never been run",
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
    #[test]
    fn every_entry_of_spec_14s_register_is_present_exactly_once() {
        assert_eq!(REGISTER.len(), 27);
        let mut seen = [false; 28];
        for e in REGISTER {
            let n = e.number as usize;
            assert!((1..=27).contains(&n), "entry {n} is not in §14's table");
            assert!(!seen[n], "entry {n} appears twice");
            seen[n] = true;
        }
        for (n, present) in seen.iter().enumerate().skip(1) {
            assert!(present, "entry {n} is silently absent");
        }
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

    /// A report may never be load-bearing for a gate (§14's second refinement), so the three
    /// entries that are reports say so in their own kind rather than in a comment.
    #[test]
    fn the_three_reports_are_marked_as_reports() {
        let reports: Vec<u8> = REGISTER
            .iter()
            .filter(|e| e.kind == Kind::Report)
            .map(|e| e.number)
            .collect();
        assert_eq!(reports, vec![25, 26, 27]);
    }

    #[test]
    fn the_tally_is_the_register() {
        let (wired, red) = tally();
        assert_eq!(wired + red, REGISTER.len());
        assert!(!table().is_empty());
    }
}
