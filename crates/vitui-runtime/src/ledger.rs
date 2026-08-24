//! **Every gated or reported number in this crate has exactly one home, and this is it.**
//!
//! Ticket 20. The engine's `crate::ledger` is the same instrument for the same reason, and the
//! reason is a measured one: the audit that produced the engine's found its watchdog threshold
//! copied into nine files. This crate had the same defect in a more embarrassing place — **the
//! frame budget itself, `const FRAME_NS: f64 = 100_000.0`, written out twelve times across
//! `examples/`**, once per report that wanted to divide by it. Eleven copies of a figure spec §19
//! says may not move without a new map decision is twelve places to move it from.
//!
//! # What this ledger says that the spec's did not
//!
//! Spec §19 carried a headroom ledger whose every row came from a **prototype** — the architecture
//! map's `R NN` tickets, measured before this crate existed — and subtracted those rows from a base
//! it recorded as **32.85–33.05 µs**. This file is that table re-measured against the shipped
//! runtime, and the headline is that the base was never the shipped frame:
//!
//! **The dense frame is 89.75–89.92 µs, not 33.** The `≈67 µs of headroom` the spec states is
//! arithmetic against a frame that does not exist. Real headroom against the 100 µs budget is
//! **1.11×**.
//!
//! That is not a regression, and checking rather than assuming is what this ledger is for.
//! **Ticket 17 already measured it** — *"the dense 300×80 frame is 91–94 µs on both arms and both
//! settings"* — and recorded in the same breath that R14's prototype magnitude *"is not reproducible
//! here"*. The number was found, written into the ticket, and never propagated into §19. A figure
//! with no single home is a figure that can be discovered twice and fixed neither time.
//!
//! # The rows do not sum to the frame, and that is the finding
//!
//! The engine's ledger totals its `Cost` rows and compares the total to a budget, because on the
//! app thread every row *is* the engine's own work. Here the same arithmetic would lie. **92% of the
//! dense frame is the engine serialising 7 488 damaged cells** (ticket 17), which is work this
//! crate causes and does not do. So the rows below sum to [`runtime_share_ns`] — the runtime's own
//! declared work, **≈8.6 µs** — and the difference between that and the measured frame belongs to
//! the engine's budget, not to this one.
//!
//! Stating it that way is what makes a failure name a layer. A gate that fires at 100 µs on a frame
//! that is 90% engine tells you the frame got slower; it does not tell you who slowed it down.
//! [`table`] prints both shares so that the diff does.
//!
//! # The one budget row, and why the other one is not here
//!
//! Spec §19 inherits two figures from the engine map: **full-screen 300×80 composition < 1 ms** and
//! **typical damage-tracked frame < 100 µs**. Only the second is a number this crate divides by, so
//! only the second is a row here. The first lives in `vitui_engine`'s ledger and is *deliberately
//! not copied* — a ledger that duplicates the figure it exists to de-duplicate has argued itself out
//! of a job.

/// The machine a row was taken on.
///
/// There is no `GitLabRunner` arm and no `Derived` arm. Warnings are denied workspace-wide, so a
/// variant nothing constructs is a build failure rather than a spare part, and the engine's ledger
/// says the same thing in the same place for the same reason.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Machine {
    /// Apple M1 Max, 10 cores, macOS 26.5.2, rustc 1.97.1, `--release`, unloaded.
    ///
    /// Every row carrying this was taken by ticket 20 on 2026-08-24, minimum of 40 rounds,
    /// round-robin, with nothing else running. Under a concurrent `cargo build` the same frame
    /// reads **94.2 µs** rather than 89.9 — a 5% load penalty, measured rather than assumed, and
    /// the reason [`table`] prints the spread rather than a single figure.
    M1Max,
    /// The architecture map's prototype, before this crate existed.
    ///
    /// A row wearing this has **not** been reproduced against the shipped runtime, and
    /// [`tests::the_table_says_how_many_rows_are_still_the_prototypes`] counts them so that the
    /// number is visible rather than discovered. Three rows are here because the shipped reports do
    /// not print the delta the prototype printed — not because the cost is believed to be gone.
    Prototype,
    /// A figure the map decided rather than measured.
    Decided,
}

impl Machine {
    /// The token [`table`] prints.
    pub fn word(self) -> &'static str {
        match self {
            Machine::M1Max => "M1 Max",
            Machine::Prototype => "PROTOTYPE",
            Machine::Decided => "decided",
        }
    }
}

/// Whether a row is something measured or something the map ruled.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// A figure spec §19 inherits from the engine map. Not moveable without a new map decision.
    Budget,
    /// Something this crate puts on the frame path, measured.
    Cost,
    /// The frame itself, measured directly rather than accumulated.
    ///
    /// Exactly one row is this, and it is the row the prototype's ledger got wrong. It is kept
    /// distinct from [`Kind::Cost`] so that [`runtime_share_ns`] cannot accidentally add the whole
    /// frame to the parts of it.
    Frame,
    /// A frame deliberately outside the budget: a cliff by construction, with a detector.
    ///
    /// **This arm exists because filing one of these as a [`Kind::Cost`] is a category error the
    /// ledger made and its own test caught.** A cliff frame is a whole frame of a different shape,
    /// not a marginal cost on the dense one, so summing it into [`runtime_share_ns`] charges this
    /// crate 42 µs of work it does not do — which is what the first draft of this file did, and
    /// what [`tests::the_runtimes_own_work_is_a_small_share_of_the_frame`] refused.
    ///
    /// Every row wearing this names its detector, and
    /// [`tests::every_cliff_names_the_detector_that_catches_it`] checks that it does. A frame
    /// allowed past the budget without one is not a cliff, it is a regression.
    Cliff,
}

impl Kind {
    /// The token [`table`] prints.
    pub fn word(self) -> &'static str {
        match self {
            Kind::Budget => "budget",
            Kind::Cost => "cost",
            Kind::Frame => "frame",
            Kind::Cliff => "cliff",
        }
    }
}

/// One number, with everything needed to argue about it.
pub struct Row {
    /// What was measured.
    pub what: &'static str,
    /// Nanoseconds. A cost may be negative: two of them are.
    pub value: f64,
    /// Measured cost, inherited budget, or the frame itself.
    pub kind: Kind,
    /// The figure this row replaced, when it replaced one.
    pub was: Option<f64>,
    /// Where the number came from.
    pub machine: Machine,
    /// The implementation ticket that took it, spelled `R NN`.
    ///
    /// **Not the engine's `impl NN` / `arch NN`.** This crate's register, its scene list and its
    /// spec table all say `R NN`, and a ledger that spelled it differently would be the only file
    /// in the crate that did.
    pub ticket: &'static str,
    /// Where the number is used, and the provenance sentence for it.
    pub used_at: &'static str,
    /// For a [`Kind::Budget`] row only: the decision that set it.
    pub decision: Option<&'static str>,
}

/// The frame budget, in nanoseconds. Spec §19, inherited from the engine map.
///
/// **This is the one home of the figure that was copied into twelve example files.** Read it;
/// do not write `100_000.0`.
pub fn frame_budget_ns() -> f64 {
    LEDGER
        .iter()
        .find(|r| r.kind == Kind::Budget && r.what == "typical damage-tracked frame")
        .expect("the frame budget is a row of this ledger")
        .value
}

/// The dense frame, measured directly.
pub fn dense_frame_ns() -> f64 {
    LEDGER
        .iter()
        .find(|r| r.kind == Kind::Frame)
        .expect("the dense frame is a row of this ledger")
        .value
}

/// What this crate's own declared work costs the frame — the sum of the [`Kind::Cost`] rows.
///
/// **This is not the frame.** See the module documentation: the difference between this and
/// [`dense_frame_ns`] is the engine serialising damaged cells, which is the engine's budget to
/// defend.
pub fn runtime_share_ns() -> f64 {
    LEDGER
        .iter()
        .filter(|r| r.kind == Kind::Cost)
        .map(|r| r.value)
        .sum()
}

/// The frames deliberately outside the budget, each with the detector that catches it.
pub fn cliffs() -> impl Iterator<Item = &'static Row> {
    LEDGER.iter().filter(|r| r.kind == Kind::Cliff)
}

/// Everything, in the order it was argued.
pub const LEDGER: [Row; 18] = [
    Row {
        what: "typical damage-tracked frame",
        value: 100_000.0,
        kind: Kind::Budget,
        was: None,
        machine: Machine::Decided,
        ticket: "R 20",
        used_at: "every `examples/*_numbers.rs` ratio, read through `frame_budget_ns` so that there \
                  is one copy of it. Before ticket 20 there were twelve, one per example, each \
                  spelled `const FRAME_NS: f64 = 100_000.0`. Counted from the tree rather \
                  than from memory: the first draft of this row said eleven",
        decision: Some(
            "spec §19, inherited from the engine map's Standing requirements. Not moveable without \
             a new map decision.",
        ),
    },
    Row {
        what: "the dense 300x80 frame, text-first",
        value: 89_875.0,
        kind: Kind::Frame,
        was: Some(33_000.0),
        machine: Machine::M1Max,
        ticket: "R 20",
        used_at: "`crate::line::tests::the_frame_measured_inside_the_crate` and \
                  `examples/crate_line_numbers.rs`, which are the two arms of one measurement; the \
                  gate is `examples/frame.rs`. Provenance: R 20, 2026-08-24, Apple M1 Max, macOS \
                  26.5.2, rustc 1.97.1, --release, unloaded, minimum of 40 rounds. Three quiet runs \
                  of each arm: crate line 89.79 / 89.92 / 89.88, in-binary 89.92 / 89.75 / 89.83 — \
                  a 0.19% spread and the two arms indistinguishable. **Confirmed on the devkit \
                  GitLab runner by pipeline 52 at 91.88 us**, 2.2% slower, which is the runner \
                  penalty on an idle six-slot machine. It replaces the prototype's 32.85-33.05, which R 17 had already found unreproducible (`91-94 us on both arms \
                  and both settings`) without the finding reaching spec §19",
        decision: None,
    },
    Row {
        what: "a modal, with the base layer and draw-before-pad",
        value: 7_960.0,
        kind: Kind::Cost,
        was: Some(1_190.0),
        machine: Machine::M1Max,
        ticket: "R 13",
        used_at: "`examples/overlay_numbers.rs`, `+ a modal, both obligations`. **The row that \
                  moved most: 6.7x the prototype's figure**, and the one that puts a dense frame \
                  with a modal standing at 98-102 us against a 100 us budget. Its obligation-1 \
                  detector is gated at 1.5x and measures 1.98x; obligation 2, `padding drawn \
                  before text`, prints 103.3% of the budget and is marked NOT GATED in the report \
                  itself",
        decision: None,
    },
    Row {
        what: "routing a realistic batch",
        value: 245.85,
        kind: Kind::Cost,
        was: Some(221.0),
        machine: Machine::M1Max,
        ticket: "R 11",
        used_at: "`examples/routing_numbers.rs`. The gate beside it is a growth ratio over a slot \
                  count, not this timing",
        decision: None,
    },
    Row {
        what: "intrinsic sizing, exit (a) made real",
        value: 264.85,
        kind: Kind::Cost,
        was: None,
        machine: Machine::Prototype,
        ticket: "R 15",
        used_at: "`examples/sizing_numbers.rs`, which prints the sizing function's **absolute** cost \
                  (998.08 ns) and the dry run's (3.69x, against spec §12's 13.3x) but not the \
                  delta this row is. Still the prototype's figure, and it is here saying so rather \
                  than absent: the cost is not believed to be gone, it is unmeasured",
        decision: None,
    },
    Row {
        what: "focus: the ring, scopes and the vanish rule",
        value: 243.75,
        kind: Kind::Cost,
        was: Some(310.0),
        machine: Machine::M1Max,
        ticket: "R 12",
        used_at: "`examples/focus_numbers.rs`, `the whole of focus`. The prototype stated a ceiling \
                  of <= +0.31 us rather than a figure, and the shipped runtime is under it. The \
                  gate beside it is the vanish rule's probe ratio, 601 against 90 300",
        decision: None,
    },
    Row {
        what: "theme: `Paint` and `resolve(tier)`",
        value: 0.0,
        kind: Kind::Cost,
        was: None,
        machine: Machine::M1Max,
        ticket: "R 04",
        used_at: "`examples/theme_numbers.rs`. `paint / raw = 1.000x` against the prototype's \
                  1.002x — at the noise floor, which is what the prototype claimed and what the \
                  shipped runtime confirms. `resolve` is 256.12 ns and runs once per theme rather \
                  than once per frame, so it is not on this path at all",
        decision: None,
    },
    Row {
        what: "deadlines and the wake ledger",
        value: 3.62,
        kind: Kind::Cost,
        was: None,
        machine: Machine::Prototype,
        ticket: "R 06",
        used_at: "`examples/anim_numbers.rs`, which prints `the ask costs -116.47 ns a frame` and \
                  in the same sentence that **two runs of the identical frame differ by -106.88 \
                  ns** — a delta smaller than the instrument's own spread, so the shipped report \
                  measures nothing here and says so. Still the prototype's 3.62 ns, which is \
                  itself below that floor",
        decision: None,
    },
    Row {
        what: "key maps: declare + scoped match",
        value: 186.29,
        kind: Kind::Cost,
        was: Some(135.6),
        machine: Machine::M1Max,
        ticket: "R 07",
        used_at: "`examples/keys_numbers.rs`, `declare 35.04 ns + batch 151.25 ns`. The prototype \
                  measured 45.90 + 89.70; the batch half is what moved. `Pending` is 32 bytes of \
                  `Copy` and the machine allocates nothing, which is a count in `tests/alloc.rs` \
                  rather than a timing here",
        decision: None,
    },
    Row {
        what: "reactivity",
        value: 0.0,
        kind: Kind::Cost,
        was: None,
        machine: Machine::Decided,
        ticket: "R 18",
        used_at: "`crates/vitui-signals/examples/signals_numbers.rs`, and **it is off the frame \
                  path by construction rather than by measurement**: `vitui-signals` is a detached \
                  workspace that nothing here may depend on, which `deny.toml`'s empty `wrappers` \
                  list enforces against the crate's presence in the graph. Zero because there is no \
                  edge, not because the edge is cheap",
        decision: None,
    },
    Row {
        what: "the crate line",
        value: 0.0,
        kind: Kind::Cost,
        was: None,
        machine: Machine::M1Max,
        ticket: "R 17",
        used_at: "`examples/crate_line_numbers.rs` against `crate::line`'s in-binary arm. **The one \
                  prototype row that held exactly**: nothing with ThinLTO, confirmed by R 20 at \
                  three quiet runs an arm where the two arms are indistinguishable (89.79-89.92 \
                  against 89.75-89.92). R14's `+0.7 us with LTO off` came from a monolith-versus- \
                  four-crates pair the shipped shape cannot build, and R 17 recorded that its \
                  magnitude does not reproduce",
        decision: None,
    },
    Row {
        what: "scrolling",
        value: -19.0,
        kind: Kind::Cost,
        was: None,
        machine: Machine::M1Max,
        ticket: "R 14",
        used_at: "`examples/scroll_numbers.rs`, `four directions 0.008 us, two axes 0.027 us, \
                  difference -0.019 us`. The prototype recorded it as *of both signs* over a \
                  -0.033 to +0.190 us range; this run is one sign and inside that range. The gate \
                  beside it is a row count, 1 000 000 against 24, not this timing",
        decision: None,
    },
    Row {
        what: "async: the handoff",
        value: -180.0,
        kind: Kind::Cost,
        was: None,
        machine: Machine::Prototype,
        ticket: "R 16",
        used_at: "`examples/work_numbers.rs`, which prints the handoff's parts (`Worker::ask` 71.0 \
                  ns, `Task::request, deduplicated` 2.3 ns, `Slot::put + take` 18.4 ns) and no \
                  frame delta at all. Still the prototype's -0.18 us. A negative row that is not \
                  reproduced is the one worth flagging: it is currently *buying* this ledger \
                  headroom it has not earned",
        decision: None,
    },
    Row {
        what: "theme registry and switching",
        value: 0.95,
        kind: Kind::Cost,
        was: Some(15.0),
        machine: Machine::M1Max,
        ticket: "R 05",
        used_at: "`examples/theme_set_numbers.rs`, `theme(), the read a frame does`. Sixteen times \
                  smaller than the prototype's +0.015 us. Everything else the registry does — \
                  `set_tier` at 284.79 ns, the swap frame — is per-swap and priced as a cliff below",
        decision: None,
    },
    Row {
        what: "the overlay body queue, in place of the arena",
        value: 0.0,
        kind: Kind::Cost,
        was: None,
        machine: Machine::M1Max,
        ticket: "R 21",
        used_at: "`examples/overlay_numbers.rs` and `tests/alloc.rs::\
                  one_standing_overlay_costs_one_allocation_a_frame`. Below the noise floor in \
                  time, and **+1 allocation a standing body a frame** — the marginal count is the \
                  gate, because it stays true as the queue grows. Spec §19 carries the exception in \
                  the same sentence as the count; ADR 0034 is the trade",
        decision: None,
    },
    Row {
        what: "an unchanged frame",
        value: 13_164.79,
        kind: Kind::Cliff,
        was: Some(23_580.0),
        machine: Machine::M1Max,
        ticket: "R 06",
        used_at: "`examples/anim_numbers.rs`, `the same frame again`. **A cliff by construction \
                  with a detector**: nothing in this runtime skips composition, so a frame that \
                  changed nothing still costs one. Its detector is `Ctx::theme_changed()`, register \
                  entry 26, which is true for exactly one frame. Counted as a cliff rather than a \
                  cost because it is a whole frame of another shape, and adding it to the dense \
                  frame's parts is the category error `Kind::Cliff` exists to prevent",
        decision: None,
    },
    Row {
        what: "an animating frame",
        value: 20_278.75,
        kind: Kind::Cliff,
        was: Some(34_170.0),
        machine: Machine::M1Max,
        ticket: "R 06",
        used_at: "`examples/anim_numbers.rs`, `the frame it asks for`. **The conclusion strengthened \
                  when the number fell**: the fading chip is 15.6 ns of it, so **99.92% of what an \
                  animation costs is redrawing a screen that did not change**, against the \
                  prototype's 99.7%. Its detector is the wake ledger — `anim::WakeLedger`, register \
                  entry 29, wired to three tests in `src/anim.rs`",
        decision: None,
    },
    Row {
        what: "a theme swap",
        value: 309_000.0,
        kind: Kind::Cliff,
        was: Some(41_920.0),
        machine: Machine::M1Max,
        ticket: "R 05",
        used_at: "`examples/theme_set_numbers.rs`, `the swap` against `steady`. **The two \
                  microsecond columns are not comparable to the prototype's**: the shipped fixture \
                  is a full-screen fill rather than the dense IDE screen, so 309 us against a 118 \
                  us steady frame is a different measurement from 41.92 against 33.20, and the \
                  report says so in as many words. What survived the fixture change is the shape \
                  of the cliff, and it is now stated in **bytes**, where it is exact and the \
                  fixture cannot blur it: a steady frame is **0 bytes on the wire** and a swap \
                  frame is **26 272**. That is not a ratio, it is a floor against a screen. Its \
                  detector is the memo pair key — `tests/swap.rs::\
                  a_pair_keyed_memo_misses_once_per_swap_and_a_data_keyed_one_never_does`",
        decision: None,
    },
];

/// The ledger as a printed table, for `examples/frame.rs`.
///
/// Prints `now` beside `was` so that a row which moved is visible without opening the spec, and
/// closes with the two shares rather than one total — see the module documentation for why a single
/// total would lie.
pub fn table() -> String {
    use std::fmt::Write as _;
    let us = |ns: f64| ns / 1e3;
    let mut out = String::new();

    let _ = writeln!(
        out,
        "  {:<52} {:>11}  {:>11}  {:<7} {:<10} ticket",
        "what", "now", "was", "kind", "machine"
    );
    for row in &LEDGER {
        let was = match row.was {
            Some(w) => format!("{:.2} us", us(w)),
            None => "-".to_string(),
        };
        let _ = writeln!(
            out,
            "  {:<52} {:>8.2} us  {:>11}  {:<7} {:<10} {}",
            row.what,
            us(row.value),
            was,
            row.kind.word(),
            row.machine.word(),
            row.ticket
        );
    }

    let frame = dense_frame_ns();
    let budget = frame_budget_ns();
    let share = runtime_share_ns();
    let engine = frame - share;

    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  {:<52} {:>8.2} us  {:.2}x of headroom against {:.0} us",
        "THE DENSE FRAME, measured directly",
        us(frame),
        budget / frame,
        us(budget)
    );
    let _ = writeln!(
        out,
        "  {:<52} {:>8.2} us  {:.1}% of the frame — this crate's own declared work",
        "  of which the runtime's rows account for",
        us(share),
        100.0 * share / frame
    );
    let _ = writeln!(
        out,
        "  {:<52} {:>8.2} us  {:.1}% of the frame — the engine serialising damaged cells",
        "  and the rest belongs to",
        us(engine),
        100.0 * engine / frame
    );

    let prototypes = LEDGER
        .iter()
        .filter(|r| r.machine == Machine::Prototype)
        .count();
    let _ = writeln!(out);
    let _ = writeln!(
        out,
        "  {prototypes} rows are still the prototype's figure and have not been reproduced against \
         the shipped runtime."
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every row says who took it and where it is used.**
    #[test]
    fn every_row_names_its_ticket_and_where_it_is_used() {
        for row in &LEDGER {
            assert!(
                row.ticket.starts_with("R "),
                "`{}` names `{}`, which is not an implementation ticket of this crate — the engine \
                 spells them `impl NN`, this crate spells them `R NN`",
                row.what,
                row.ticket
            );
            assert!(
                row.used_at.len() > 20,
                "`{}` does not say where its number is used",
                row.what
            );
        }
    }

    /// **A budget figure names the decision that set it, and nothing else carries one.**
    ///
    /// Two-sided on purpose: the engine's ledger makes the same assertion, and the half that earns
    /// its keep is the second — a `Cost` row growing a `decision` is a cost quietly promoting
    /// itself to something that may not move.
    #[test]
    fn a_budget_figure_names_the_decision_that_set_it() {
        for row in &LEDGER {
            if row.kind == Kind::Budget {
                assert!(
                    row.decision.is_some_and(|d| d.contains("map decision")),
                    "`{}` is a budget and does not name the decision that set it",
                    row.what
                );
            } else {
                assert!(
                    row.decision.is_none(),
                    "`{}` is not a budget and carries a decision",
                    row.what
                );
            }
        }
    }

    /// **The budget is the one the map decided.**
    #[test]
    fn the_frame_budget_is_the_one_the_map_decided() {
        assert_eq!(
            frame_budget_ns(),
            100_000.0,
            "the frame budget has moved, and spec §19 says it may not without a new map decision"
        );
    }

    /// **Exactly one row is the frame itself.**
    ///
    /// [`runtime_share_ns`] sums `Cost` rows, so a second `Frame` row would not corrupt it — but a
    /// `Frame` row demoted to `Cost` would, by adding the whole frame to the parts of it. This is
    /// the assertion that would catch that edit.
    #[test]
    fn exactly_one_row_is_the_frame() {
        let frames: Vec<&str> = LEDGER
            .iter()
            .filter(|r| r.kind == Kind::Frame)
            .map(|r| r.what)
            .collect();
        assert_eq!(
            frames.len(),
            1,
            "the frame is measured directly and exactly once; found {frames:?}"
        );
    }

    /// **The dense frame is inside the budget, and this is the assertion the ledger exists to make.**
    ///
    /// Register entry 12's gate is `examples/frame.rs`, which measures. This one is over the
    /// *recorded* figure, so a commit that edits the ledger to make a red gate green has to walk
    /// past a second assertion in a different file.
    #[test]
    fn the_dense_frame_is_inside_the_budget() {
        let frame = dense_frame_ns();
        let budget = frame_budget_ns();
        assert!(
            frame < budget,
            "the recorded dense frame is {:.2} us against a {:.0} us budget",
            frame / 1e3,
            budget / 1e3
        );
    }

    /// **The runtime's own rows are a small share of the frame, and the ledger may not pretend
    /// otherwise.**
    ///
    /// This is the correction to the spec's table stated as a gate. The prototype's ledger
    /// subtracted its rows from a 33 µs base and reported ≈67 µs of headroom, which only makes
    /// sense if the rows are most of the frame. They are under a tenth of it. If this assertion
    /// ever fails it means either the runtime grew enormously or the engine got much faster, and
    /// both of those are things a person should be told about rather than have averaged away.
    #[test]
    fn the_runtimes_own_work_is_a_small_share_of_the_frame() {
        let share = runtime_share_ns();
        let frame = dense_frame_ns();
        assert!(
            share < frame * 0.25,
            "the runtime's rows are {:.2} us of an {:.2} us frame ({:.1}%), which is no longer the \
             shape this ledger describes — R 17 measured 92% of the frame as the engine serialising \
             7 488 damaged cells",
            share / 1e3,
            frame / 1e3,
            100.0 * share / frame
        );
    }

    /// **Every cliff names the detector that catches it.**
    ///
    /// A frame allowed past the budget without one is not a cliff, it is a regression with a note
    /// attached. The three the map declared are here, and the check is that each says the word —
    /// spec §19's table gives each of them a *why it is allowed*, and a detector is what makes that
    /// sentence falsifiable rather than reassuring.
    #[test]
    fn every_cliff_names_the_detector_that_catches_it() {
        let named: Vec<&str> = cliffs().map(|r| r.what).collect();
        assert_eq!(
            named,
            vec!["an unchanged frame", "an animating frame", "a theme swap"],
            "the cliffs have changed. A fourth frame outside the budget is a map decision, not a \
             ledger edit"
        );
        for row in cliffs() {
            assert!(
                row.used_at.contains("detector"),
                "`{}` is outside the budget and does not name the detector that catches it",
                row.what
            );
        }
    }

    /// **The table says how many rows are still the prototype's.**
    ///
    /// Three, and each of them is a row whose shipped report prints the absolute cost but not the
    /// delta. Naming the count here is what stops a fourth arriving unremarked — the engine's
    /// ledger carries the identical test for the identical reason.
    #[test]
    fn the_table_says_how_many_rows_are_still_the_prototypes() {
        let still: Vec<&str> = LEDGER
            .iter()
            .filter(|r| r.machine == Machine::Prototype)
            .map(|r| r.what)
            .collect();
        assert_eq!(
            still,
            vec![
                "intrinsic sizing, exit (a) made real",
                "deadlines and the wake ledger",
                "async: the handoff",
            ],
            "the unreproduced rows have changed. One that stopped being a prototype's should have \
             taken its `Machine::Prototype` with it; a new one has to be argued for here"
        );
        assert!(
            table().contains("3 rows are still the prototype's figure"),
            "the table does not print the count"
        );
    }

    /// **The table prints, and prints both shares rather than one total.**
    #[test]
    fn the_table_prints_both_shares() {
        let t = table();
        assert!(t.contains("THE DENSE FRAME, measured directly"));
        assert!(t.contains("this crate's own declared work"));
        assert!(t.contains("the engine serialising damaged cells"));
    }
}
