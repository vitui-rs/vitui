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
        state: State::Red {
            inverted_by: "impl 18",
            why: "there is no mailbox and no pool while there is one thread",
        },
    },
    Entry {
        number: 6,
        property: "`pack` allocates zero on warm tables, at every density",
        kind: Kind::Gate,
        qualifier: "count",
        source: "arch 16",
        state: State::Red {
            inverted_by: "impl 18",
            why: "impl 06 mints the handle tables, but the gate is on `pack`, which only becomes \
                  a thing to measure once there is a packet crossing a mailbox — impl 18 carries \
                  it in its own criteria",
        },
    },
    Entry {
        number: 7,
        property: "A settled operator allocates zero; a fading one allocates per distinct extended \
                   style and never per cell",
        kind: Kind::Gate,
        qualifier: "shape",
        source: "arch 16",
        state: State::Red {
            inverted_by: "impl 08",
            why: "impl 12 built the operator layer and its memo, so a *settled* operator is now \
                  measurable and does converge after one frame — what is still missing is the \
                  sweep and `repaint`, which bound a *fading* one, and impl 08 owns both and \
                  carries the gate in its own criteria",
        },
    },
    Entry {
        number: 8,
        property: "The pool does not starve at 10 000 cycles",
        kind: Kind::Gate,
        qualifier: "count",
        source: "arch 09",
        state: State::Red {
            inverted_by: "impl 18",
            why: "there is no pool while there is one thread",
        },
    },
    Entry {
        number: 9,
        property: "A packet is never superseded",
        kind: Kind::Gate,
        qualifier: "count",
        source: "arch 09",
        state: State::Red {
            inverted_by: "impl 18",
            why: "nothing can supersede a packet that is consumed inline. This is the \
                  backpressure decision, not a health check: if it ever trips, the pacing gate has \
                  moved off the app thread and frames are being composed to be thrown away",
        },
    },
    Entry {
        number: 10,
        property: "A packed cell is byte-identical to the surface cell",
        kind: Kind::Gate,
        qualifier: "equality",
        source: "arch 16",
        state: State::Wired {
            at: "crate::gates::a_packed_cell_is_byte_identical_to_the_surface_cell",
        },
    },
    Entry {
        number: 11,
        property: "After a sweep every live cell resolves to the same channels; `renumbered` is \
                   false when nothing below a live entry was freed",
        kind: Kind::Gate,
        qualifier: "equality",
        source: "arch 16",
        state: State::Red {
            inverted_by: "impl 08",
            why: "impl 07 built the two tables and impl 06 the third, so there is something to \
                  sweep now — what is missing is the sweep, and impl 08 owns it",
        },
    },
    Entry {
        number: 12,
        property: "Round trip: replayed screen == composited frame",
        kind: Kind::Gate,
        qualifier: "equality",
        source: "arch 08, 16",
        state: State::Wired {
            at: "crate::roundtrip, and crate::gates::the_round_trip_closes_on_every_scene",
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
            why: "there is no frame clock and nothing parks, so there is no idle to measure",
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
            at: "crate::gates::wire_bytes_per_scene",
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
            at: "examples/budget.rs, every scene of the full-screen class; three scenes are \
                 reported rather than gated and name impl 18",
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
            why: "nothing parks, so there is no wake-up to time",
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
