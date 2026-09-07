//! The ledger: **every number this repository gates or reports on, once, with where it came from.**
//!
//! Impl ticket 26 asks for two things that turned out to be the same thing:
//!
//! > Every ledger row is re-measured against the shipped engine and the table is rewritten with real
//! > numbers.
//!
//! > Every gate number in the repository carries a provenance comment naming its ticket and machine.
//! > A number without one is treated as already broken.
//!
//! The second criterion is what produced this file, and it produced it by failing. A four-file audit
//! over every numeric threshold in the repository found **three `Provenance:` lines in the whole
//! tree** — `gates.rs`, and two in `examples/budget.rs` — against something over a hundred numbers
//! that gate or report. That is not a hundred missing comments. It is one missing *structure*:
//!
//! > **The ledger rows had no single home.** Each figure lived as prose in every file that reasoned
//! > about it, so *re-measure the ledger* was a grep-and-hope operation rather than an edit to one
//! > table.
//!
//! The audit counted the copies. The realistic `wake → submit` iteration — the one number the whole
//! watchdog threshold rests on — was written in **nine** places. The 40-layer full-screen composite
//! was in eight, plus three ADRs. The mailbox critical section was in four. **And the 1 ms / 100 µs
//! split, which is the one class of number the ticket says may not move without a new map decision,
//! was written four times in two files.** A number that has to be edited in four places to be
//! corrected once is a number that will be corrected in three.
//!
//! **The first of those four counts was still open when this paragraph was three tickets old.** The
//! realistic `wake → submit` iteration had grown from nine copies to ten and had never had a row
//! here — the file built to end this had no home for its own motivating figure. It is
//! [`realistic_iteration_ns`] now, and
//! `crate::gates::the_watchdog_iteration_has_one_home` is what keeps it the only one.
//!
//! So a provenance comment beside each of a hundred literals would have satisfied the criterion and
//! left the defect: the comment is only true where somebody remembered to put it, and the copy
//! nobody found is the one that stays wrong. **One table, and the prose points at it.**
//!
//! # What a row is, and what it is not
//!
//! A [`Row`] is a **figure with a measurement behind it** — the six the ticket tabulates, plus the
//! budget figures the gates sit at. It is not every integer in the engine. A loop count, a pool size
//! that is provably two, a cell that is sixteen bytes: those are properties of a mechanism rather
//! than measurements of one, and [`Kind`] is where that distinction is written down instead of
//! being left to a reader's judgement.
//!
//! # Who may move a number, from the ticket, unchanged
//!
//! > Anyone, in a commit that does three things: states the new number, states the measurement it
//! > came from, and replaces the provenance comment next to it. **The one class that may not move
//! > without a new map decision is a budget figure** — the 1 ms, the 100 µs, the zero allocations,
//! > the zero wakeups.
//!
//! [`Kind::Budget`] is that class, and [`tests::a_budget_figure_names_the_decision_that_set_it`] is
//! what makes the sentence cost something: a budget row must name the map decision, and no other
//! kind may.
//!
//! # Two statements here are not numbers, and they are here for the ledger's own reason
//!
//! [`the_parallel_compositing_ruling`] and [`the_watchdog_over_a_timing_report`] are prose with a
//! gate under each. Both are facts *about* figures on this table rather than figures — one is the
//! verdict the compositing row's re-measurement was taken for, the other is **whether a reported
//! number can fail**, which production ticket 11 found nobody had written down anywhere. A report
//! that a debug-only watchdog can turn red is a report whose provenance is incomplete, and the one
//! home for that is beside the numbers rather than in whichever file noticed.

use std::fmt::Write as _;

/// Where a figure was measured. Named rather than described, because "on my machine" is not
/// provenance and a figure whose machine is unknown cannot be compared with the next one.
/// **Only what the ledger uses.** The workspace denies every warning, so a variant nothing
/// constructs is a build failure rather than a spare part — and that is the right trade here. Two
/// obvious members are therefore absent rather than unused: a `GitLabRunner` arm, for when a row is
/// measured in the container rather than on this machine, and a `Derived` arm, for a figure computed
/// from other rows. Report #25's derived line is the only such figure today and it lives in
/// `examples/budget.rs` beside the measured one; the first row that needs either arm adds it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Machine {
    /// The development machine: Apple M1 Max, 10 cores, macOS 26.5.2, rustc 1.97.1, `--release`,
    /// unloaded. Fine-grained timing comparisons run here, which is the ticket's own arrangement:
    /// *fine-grained timing comparisons run on a developer's machine, and their output goes into a
    /// ticket.*
    M1Max,
    /// A prototype written while charting, before the engine existed. Every one of these is a figure
    /// the ticket exists to replace, and a row still carrying it is a row nobody has re-measured.
    Prototype,
    /// Not a measurement at all — a decision, or a property of a mechanism that argument settles and
    /// a stopwatch cannot.
    Decided,
}

impl Machine {
    /// The word the table prints.
    pub fn word(self) -> &'static str {
        match self {
            Machine::M1Max => "M1 Max",
            Machine::Prototype => "PROTOTYPE",
            Machine::Decided => "decided",
        }
    }
}

/// What kind of number a row holds, which decides what may be done to it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    /// A **budget figure**: the class that may not move without a new map decision. `decision` on
    /// the row must name where it was decided.
    Budget,
    /// A cost measured on the shipped engine. Anyone may move it, in a commit that states the new
    /// number and the measurement it came from.
    Cost,
}

impl Kind {
    /// The word the table prints.
    pub fn word(self) -> &'static str {
        match self {
            Kind::Budget => "budget",
            Kind::Cost => "cost",
        }
    }
}

/// One figure, with everything needed to know whether it is still true.
#[derive(Clone, Copy, Debug)]
pub struct Row {
    /// What the figure is a figure *of*, in the ledger's own words.
    pub what: &'static str,
    /// The figure, in nanoseconds for a cost and dimensionless for a ratio. Nanoseconds throughout
    /// so that a ledger total is an addition rather than a unit conversion — the arithmetic that
    /// produced the ticket's `≈ 205 µs` is the one thing about the ledger a reader should not have
    /// to redo by hand.
    pub value: f64,
    /// Cost, ratio or budget.
    pub kind: Kind,
    /// The figure this row replaced, if it replaced one. **This is the column the ticket is about**:
    /// a row whose `was` is `Some` and whose [`Machine`] is still [`Machine::Prototype`] has not
    /// been re-measured, whatever the prose beside it says.
    pub was: Option<f64>,
    /// Where it was measured.
    pub machine: Machine,
    /// The ticket the figure came from: `impl NN` or `arch NN`.
    pub ticket: &'static str,
    /// Where the number is used, so that moving it is an edit somebody can scope.
    pub used_at: &'static str,
    /// For [`Kind::Budget`] only: the map decision that set it. `None` for every other kind.
    pub decision: Option<&'static str>,
}

/// A nanosecond figure as microseconds, for the arithmetic.
fn us(ns: f64) -> f64 {
    ns / 1e3
}

/// A figure in the unit it is legible in.
///
/// Two of the six rows are tens of nanoseconds and four are tens of microseconds, so one unit across
/// the column prints either `0.03 us` or `43 900 ns`. Neither is a number a reader can compare at a
/// glance, and a ledger whose own table has to be converted before it can be read is one more place
/// for a figure to be misquoted.
fn figure(ns: f64) -> String {
    if ns < 1_000.0 {
        format!("{ns:>9.2} ns")
    } else {
        format!("{:>9.2} us", us(ns))
    }
}

/// **The app thread's worst realistic frame, re-measured against the engine that now exists.**
///
/// The ticket's table had six rows and every figure in it came from a prototype. Five of the six are
/// re-measured here; the sixth cannot be, and says so. The interesting column is `was`.
///
/// # What moved, and every one of the four moved for a different reason
///
/// **Compositing halved, and the prototype's stack turned out not to exist.** Arch 06's 107.3 µs was
/// forty layers *half of them operators*; the shipped `popup_stack` is forty content layers and no
/// operator at all, and a whole-screen composite at **fifty** layers is 39.1–43.9 µs. There is also
/// no forty-layer measurement to take: the depth axis is 1, 3, 20, 50, and there is **no door onto a
/// composite alone on the public surface**, so a per-scene isolated figure does not exist
/// and cannot. The row takes the top of the fifty-layer bracket, which over-states forty rather than
/// flattering it.
///
/// **Packing held, on the density that was never typical.** The prototype's ~53 µs reproduces
/// exactly — as the *hostile* arm, every cell a distinct style. The realistic 1% arm is 22.0–23.7 µs,
/// so the ledger's own worst-realistic-frame framing was carrying a worst-*possible* number.
///
/// **The mailbox critical section got cheaper**, 47 ns to 32.1 ns, and had to be measured by
/// difference because two `Instant::now()` calls cost 36 ns and cannot bracket 32.
///
/// **Damage reproduced**, 7.12 µs of prototype against 5.73–6.88 µs measured over the same four
/// operations. It is the one clean confirmation on the table.
///
/// # And one row that is not a measurement and will not be one for a while
///
/// **Component drawing, ~45 µs, is a third of the total and there are no components.** It stays,
/// because a ledger that dropped it would report a better number by removing the largest thing the
/// app thread does not yet do. It is labelled [`Machine::Prototype`] and the table prints how many
/// such rows there are, so the total cannot be quoted as measured when a third of it is not.
///
/// # The total
///
/// **≈ 149 µs against 1 ms, where the ticket's prototype table said ≈ 205 µs** — so the re-measured
/// engine is *cheaper* than the figures the budget was argued on, and the headroom the
/// parallel-compositing ruling rests on is wider rather than narrower. See
/// [`the_parallel_compositing_ruling`].
///
/// The total is printed by `examples/budget.rs` on every run rather than asserted as an equality: a
/// sum of six independently measured costs is a report, and a report may not be load-bearing for a
/// gate. What *is* asserted is the one thing a decision hangs on — that the total is inside the
/// budget — in [`tests::the_app_threads_worst_realistic_frame_is_inside_the_budget`].
///
/// Provenance for every re-measured row: **impl 26, Apple M1 Max, macOS 26.5.2, rustc 1.97.1,
/// `--release`, unloaded**, and each row's `used_at` names the test that produces it.
pub const LEDGER: [Row; 8] = [
    Row {
        what: "component drawing, five hostile components",
        value: 45_000.0,
        kind: Kind::Cost,
        was: None,
        machine: Machine::Prototype,
        ticket: "arch 14",
        used_at: "no gate, and **this row cannot be re-measured at all**: there are no components, \
                  and the five hostile ones exist as design pressure on the seam rather than as \
                  code. It stays at the prototype's figure with that said out loud, because \
                  deleting it would improve the total by removing the largest thing the app thread \
                  does not yet do.",
        decision: None,
    },
    Row {
        what: "compositing, whole screen at 50 layers",
        value: 43_900.0,
        kind: Kind::Cost,
        was: Some(107_300.0),
        machine: Machine::M1Max,
        ticket: "impl 26",
        used_at: "crate::layer's composite_costs_by_damaged_area_and_depth, the whole-screen \
                  depth-50 case — which is structurally crate::scenes' popup_stack, the same \
                  placement arithmetic over a full-screen opaque base. **Forty is not a measured \
                  point and is not interpolated**: the axis is 1, 3, 20 and 50, and the bracket \
                  around forty is 19.4-23.9 us at twenty and 39.1-43.9 us at fifty. The row takes \
                  the top of the upper bracket, so it over-states rather than flatters. Two things \
                  the re-measurement found and the prototype figure hides: there is no door onto a \
                  composite alone on the public surface (ADR 0023), so no per-scene isolated \
                  figure exists or can; and **the prototype's stack does not exist in this \
                  engine** — arch 06's 107.3 us was forty layers half of them operators, while \
                  popup_stack is forty content layers and no operator. The operator axis is \
                  measured separately at 12.2-12.8 us for a full-screen Mix, where §5 recorded \
                  78.2 us and the memo closed the gap.",
        decision: None,
    },
    Row {
        what: "packing, full screen, every cell a distinct style",
        value: 53_500.0,
        kind: Kind::Cost,
        was: Some(53_000.0),
        machine: Machine::M1Max,
        ticket: "impl 26",
        used_at: "crate::gates' what_pack_costs_at_four_densities, the hostile arm — plain is \
                  22.3-25.4 us and the realistic 1% arm 22.0-23.7, so the prototype's ~53 us \
                  reproduces on the density that was never the typical one. Gate #6 is what rests \
                  on these rows. **The row's original wording no longer describes a frame the \
                  engine has**, and that is a finding rather than an edit: arch 14 wrote it as \
                  packing *with the equality filter's shadow*, and since ADR 0006 the filter is on \
                  the render thread with the mirror — so its cost is not app-thread cost at all, \
                  and the shadow measured as a difference is 0.50-1.33 ns a damaged cell across \
                  three runs against §8's 0.9, a 2.7x spread that cannot carry a ledger row.",
        decision: None,
    },
    Row {
        what: "damage marking and scanning, worst frame",
        value: 6_880.0,
        kind: Kind::Cost,
        was: Some(7_000.0),
        machine: Machine::M1Max,
        ticket: "impl 26",
        used_at: "crate::damage's damage_costs_against_the_spec_table — mark 1.39-1.66, scan \
                  0.77-0.92, union 2.75-3.32 and clear 0.81-0.98 us, the same four rows §6 summed \
                  to 7.12. **The row that reproduced**, and the closest thing on the ledger to a \
                  clean confirmation. Idle costs 2.65 ns. crate::gates' \
                  no_damage_structure_under_reports is the property over it.",
        decision: None,
    },
    Row {
        what: "the mailbox critical section",
        value: 32.1,
        kind: Kind::Cost,
        was: Some(47.0),
        machine: Machine::M1Max,
        ticket: "impl 26",
        used_at: "crate::gates' what_the_handoff_costs, by difference: an 79.0-81.8 ns \
                  four-acquisition cycle less a 17.0-17.6 ns two-acquisition discard, halved. A \
                  difference because two Instant::now() calls cost 36 ns and cannot bracket 32 — \
                  which is the same reason register #22 gates a ratio rather than a nanosecond \
                  count. Gate #9's never-superseded count is what the section protects.",
        decision: None,
    },
    Row {
        what: "the overrun detector, enter and leave",
        value: 45.68,
        kind: Kind::Cost,
        was: Some(42.6),
        machine: Machine::M1Max,
        ticket: "impl 26",
        used_at: "examples/budget.rs `the_overrun_detector`, gate/report #22 — 44.77 to 45.68 ns \
                  across two runs, inside the 43.9-46.9 ns spread #22 already records, at 1.24x \
                  the two clock reads it is built out of.",
        decision: None,
    },
    Row {
        what: "full-screen 300x80",
        value: 1_000_000.0,
        kind: Kind::Budget,
        was: None,
        machine: Machine::Decided,
        ticket: "arch 15",
        used_at: "examples/budget.rs, gate #23, over every scene of the full-screen class, read \
                  through `full_screen_budget_ns` so that there is one copy of it",
        decision: Some(
            "the map's Standing requirements, requirement 3, and §13. Not moveable without a new \
             map decision.",
        ),
    },
    Row {
        what: "typical damage-tracked frame",
        value: 100_000.0,
        kind: Kind::Budget,
        was: None,
        machine: Machine::Decided,
        ticket: "arch 15",
        used_at: "examples/budget.rs, gate #24, over every scene of the incremental class, read \
                  through `incremental_budget_ns` so that there is one copy of it",
        decision: Some(
            "the map's Standing requirements, requirement 3, and §13. Not moveable without a new \
             map decision.",
        ),
    },
];

/// The six cost rows summed, which is the ledger's own claim: *everything the app thread pays for on
/// the worst realistic frame*.
pub fn app_thread_total_ns() -> f64 {
    LEDGER
        .iter()
        .filter(|r| r.kind == Kind::Cost)
        .map(|r| r.value)
        .sum()
}

/// **The worst realistic `wake → submit` iteration, and the number the watchdog threshold rests
/// on.** 166.76 µs, arch 18, on the M1 Max.
///
/// Drawing, compositing under eight layers and packing, measured over
/// `crate::scenes`' `sparse-chart-400-points` — four hundred one-cell writes scattered over the
/// screen, which is the scene that decided this and two other tickets.
///
/// # Why it is a function here and not a row on the table
///
/// [`LEDGER`]'s six [`Kind::Cost`] rows are the **parts** of one frame and
/// [`app_thread_total_ns`] adds them up, so a seventh cost row would be counted into that sum. This
/// is a **measured whole** of a different scene, and the two are not addends of each other: as a
/// row it would silently double the total it sits beside. So it joins
/// [`the_parallel_compositing_ruling`] and [`the_watchdog_over_a_timing_report`] as a member of
/// this module that is not a member of the table — the third such, and the first that is a number.
///
/// # What reads it, and why this file is where it lives
///
/// This is the figure the module doc above counts at **nine copies**, and the count was **ten** by
/// the time it got a home: `engine.rs` twice, `perf.rs` four times, `scenes.rs` twice, `lib.rs` and
/// `gates.rs`. It is not decorative in any of them. `perf::threshold_for` argues the whole *one
/// frame interval rather than 100 µs* ruling from it, [`crate::perf`]'s stall factor prices its
/// 6 400x of headroom against it, and `Config::overrun_threshold` repeats the argument for a
/// caller. A number that has to be edited in ten places to be corrected once is the sentence this
/// file opens with, and it was true of this file's own motivating figure for as long as the file
/// has existed.
///
/// The digits now appear **once in the crate**, here, which
/// `crate::gates::the_watchdog_iteration_has_one_home` is what keeps true — and it derives its
/// needle from this function rather than spelling it, so the gate is not itself a copy.
pub fn realistic_iteration_ns() -> f64 {
    166_760.0
}

/// The full-screen budget, in nanoseconds. **One home**, and gate #23 reads it from here.
pub fn full_screen_budget_ns() -> f64 {
    budget("full-screen 300x80")
}

/// The incremental budget, in nanoseconds. **One home**, and gate #24 reads it from here.
pub fn incremental_budget_ns() -> f64 {
    budget("typical damage-tracked frame")
}

fn budget(what: &str) -> f64 {
    LEDGER
        .iter()
        .find(|r| r.what == what && r.kind == Kind::Budget)
        .expect("every budget the gates read is a row on the ledger")
        .value
}

/// **The parallel-compositing ruling, re-checked against real numbers, which is a criterion of its
/// own.**
///
/// The ruling, from the ticket:
///
/// > Rows are independent and the stack would split into horizontal bands, but the price is a thread
/// > pool that the idle requirement dislikes and `rayon`, which the dependency policy forbids
/// > outright. Writing one by hand for a 107 µs worst case already inside budget is not a trade.
/// > **Ruled out by measurement, not while charting.**
///
/// **Confirmed, and by a wider margin than the figure it was made on** — which is the outcome worth
/// having, because a re-check that merely reproduced the original number would not have tested
/// anything. Three numbers, and the first is the one that decides it:
///
/// 1. **The composite is 43.9 µs, not 107.3 µs.** A whole-screen composite at *fifty* layers is
///    39.1–43.9 µs on the M1 Max; at twenty it is 19.4–23.9. The prototype's figure was forty layers
///    half of them operators, and the operator turned out to be the expensive half — measured
///    separately, a full-screen `Mix` is 12.2–12.8 µs where §5 recorded 78.2, because the memo closed
///    it. So the thing a thread pool would parallelise is **4.4% of the 1 ms budget**, against the
///    10.7% the ruling was made on.
/// 2. **It is a smaller fraction of the frame than it was.** The shipped `twenty-popups-with-shadows`
///    is 337.9 µs inline and 237.3 µs of app-thread share; the composite is a sixth of it, and the
///    rest is packing, the equality filter and the wire — none of which splits into bands. Amdahl is
///    the whole argument, and the fraction moved the wrong way for parallelism.
/// 3. **The price did not move at all.** `rayon` is banned outright by ADR 0001, and a hand-written
///    pool has to stay parked for requirement 11's zero-wakeup idle — which impl 26 has now measured
///    from the other side: a 60 Hz steady state costs 0.133% of a core, and a pool of idle threads is
///    a cost that does not care whether a frame is being composed.
///
/// **Not implemented here either way**, which the ticket states in as many words. And **not raised as
/// a finding against the map**: the ruling's reasoning is what the new numbers support, and they
/// support it harder. The one thing worth carrying forward is that the ledger's compositing row was
/// **2.4x pessimistic**, so any future argument that starts *compositing is the expensive part*
/// should start from 43.9 µs.
pub fn the_parallel_compositing_ruling() -> &'static str {
    "confirmed, wider than before. The composite a band split would parallelise is 43.9 us at \
     fifty layers on the M1 Max, not the prototype's 107.3 us at forty — 4.4% of the 1 ms budget \
     against the 10.7% the ruling was made on, because the operator was the expensive half and the \
     memo closed it. It is also a sixth of the frame it sits in rather than a third, the rest being \
     packing, the filter and the wire, none of which splits. rayon is banned by ADR 0001 and a \
     hand-written pool must stay parked for requirement 11. Not implemented, and not raised as a \
     finding: the new numbers support the ruling harder than the old ones did."
}

/// **Whether a timing report that runs under `cargo test` can be failed by a timing — and it could,
/// which is production ticket 11.**
///
/// `crate::gates::what_a_wake_up_costs_with_the_app_thread_parked` is register entry #26 and it is a
/// *report*: its only assertion is a count, that at least half the submits were observed, and the
/// rest of it prints percentiles. It cannot fail on its own terms for a timing reason. It failed for
/// one anyway — pipeline 31, job 125, **22.3 ms against a 16.7 ms frame budget**, green on the
/// retry, green for the thirteen runs before it, and 0.24 s for all 512 iterations on the
/// development machine with a 2.7 ms worst observed latency. Nothing was slow; the app thread was
/// descheduled for 22 ms on a runner that serves every repository on the machine from six slots.
///
/// The sanction was `crate::perf::Perf::sanction`, which is `cfg(debug_assertions)` and therefore
/// armed in every `cargo test` run. So §14's rule — *a gate is a count, a ratio, an equality or a
/// compile outcome; a timing is a report* — was bypassed by a mechanism that is **not a gate at
/// all**, and a report acquired a timing failure mode it neither chose nor stated.
///
/// # The enumeration, because the one that fired was not the only one
///
/// The exposure is precisely *a timing report that runs under `cargo test` and calls `present`
/// after `wait`*. `examples/budget.rs`, `examples/steady.rs` and `examples/idle.rs` are exempt by
/// profile: they run `--release`, where `debug_assertions` is off and `sanction` never panics. Under
/// `cargo test` there are **two**, and neither is the one the ticket predicted:
///
/// - `what_a_wake_up_costs_with_the_app_thread_parked`, 16.7 ms of budget at its fallback rate — the
///   one that fired;
/// - `the_achieved_rate_lands_under_the_configured_ceiling`, **8.3 ms** at its declared 120 Hz, so
///   tighter than the failure, and named nowhere in the ticket.
///
/// `what_a_leading_edge_costs_after_a_quiet_period` is the sibling the ticket expected and it is
/// **not** exposed: it never presents, and `sanction` is reachable only through `Perf::leave`, which
/// only `Screen::present` calls. The other seven tests where both verbs appear are gates rather than
/// reports, and `crate::gates::REACHED_BY_THE_DETECTOR` is the full list with each one's disposition.
///
/// # The answer, and what it costs
///
/// Both reports hold `Screen::permit_slow` for the whole measurement loop, with a reason that says
/// *why* — that the loop is timed by the scheduler and not by the engine — so the declaration makes
/// the diagnostics better rather than quieter. **They are not armed, and this sentence is where that
/// is written down.** Nothing is lost: neither report gates on what an iteration cost, so a permit
/// cannot mask a regression in what either of them measures, and the observer thread still aborts on
/// an iteration that never returns, because a permit annotates a stall rather than excusing one.
pub fn the_watchdog_over_a_timing_report() -> &'static str {
    "not armed. The two timing reports that run under `cargo test` and reach the in-loop detector — \
     what_a_wake_up_costs_with_the_app_thread_parked at 16.7 ms of budget, and \
     the_achieved_rate_lands_under_the_configured_ceiling at 8.3 ms — each hold a permit_slow for \
     the whole loop, declaring that the loop is timed by the scheduler and not by the engine. \
     Neither gates on an iteration's cost, so the permit cannot mask a regression in what it \
     reports; the observer thread is untouched, because a permit annotates a stall rather than \
     excusing one. what_a_leading_edge_costs_after_a_quiet_period needs no permit and has none: it \
     never presents, and the sanction is reachable only through present. Filed as production 11 \
     after 22.3 ms against a 16.7 ms budget on the shared runner, pipeline 31 job 125."
}

/// The ledger as a table, for `examples/budget.rs` to print.
pub fn table() -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "  {:<50} {:>12}  {:>12}  {:<7} {:<10} ticket",
        "row", "now", "was", "kind", "where"
    );
    for row in LEDGER {
        let now = figure(row.value);
        let was = match row.was {
            Some(v) => figure(v),
            None => "          —".to_string(),
        };
        let _ = writeln!(
            out,
            "  {:<50} {now:>12}  {was:>12}  {:<7} {:<10} {}",
            row.what,
            row.kind.word(),
            row.machine.word(),
            row.ticket
        );
    }
    let total = app_thread_total_ns();
    let budget = full_screen_budget_ns();
    let _ = writeln!(
        out,
        "  {:<50} {:>9.2} us  {:>12}  {:<7} against {:.0} us, {:.1}x of headroom",
        "TOTAL, the app thread's worst realistic frame",
        us(total),
        "",
        "",
        us(budget),
        budget / total,
    );
    let unmeasured = LEDGER
        .iter()
        .filter(|r| r.machine == Machine::Prototype)
        .count();
    let costs = LEDGER.iter().filter(|r| r.kind == Kind::Cost).count();
    let _ = writeln!(
        out,
        "  {unmeasured} of the {costs} cost rows {} still the prototype's figure, and the `was` \
         column is where that is visible rather than inferred.",
        if unmeasured == 1 { "is" } else { "are" }
    );
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every row names a ticket and somewhere it is used. **A number without provenance is treated
    /// as already broken**, and this is where that sentence costs something rather than being
    /// advice.
    #[test]
    fn every_row_names_its_ticket_and_where_it_is_used() {
        for row in LEDGER {
            assert!(
                row.ticket.starts_with("impl ") || row.ticket.starts_with("arch "),
                "{}: `{}` is not a ticket reference",
                row.what,
                row.ticket
            );
            assert!(
                row.used_at.len() > 20,
                "{}: `used_at` says nothing a reader could act on",
                row.what
            );
        }
    }

    /// **A budget figure names the map decision that set it, and nothing else may.**
    ///
    /// The ticket's one asymmetry made structural: anyone may move a cost in a commit that states
    /// the measurement, and a budget moves only with a new map decision. Two-sided, because a
    /// one-sided assertion passes for any reason including every row having been relabelled a
    /// budget.
    #[test]
    fn a_budget_figure_names_the_decision_that_set_it() {
        for row in LEDGER {
            match row.kind {
                Kind::Budget => assert!(
                    row.decision.is_some_and(|d| d.contains("map decision")),
                    "{}: a budget figure whose decision is not named is a budget figure anyone \
                     can move",
                    row.what
                ),
                _ => assert!(
                    row.decision.is_none(),
                    "{}: only a budget figure carries a decision",
                    row.what
                ),
            }
        }
    }

    /// The two budgets the gates read exist and are the numbers the map decided. **This is the gate
    /// on the deduplication itself**: the audit found the 1 ms / 100 µs split written four times in
    /// two files, and the point of one home is that a second copy cannot drift from it. If somebody
    /// reintroduces a literal, this test does not catch it — but `examples/budget.rs` reading these
    /// two functions means the literal would have to be *used* to matter, and there is nowhere left
    /// to use one.
    #[test]
    fn the_two_budgets_are_the_ones_the_map_decided() {
        assert_eq!(
            full_screen_budget_ns(),
            1e6,
            "the full-screen budget is 1 ms"
        );
        assert_eq!(
            incremental_budget_ns(),
            1e5,
            "the incremental budget is 100 us"
        );
    }

    /// The ledger's total is inside the full-screen budget, which is the claim the whole table
    /// exists to make and the premise the parallel-compositing ruling rests on.
    ///
    /// A **gate**, unusually for a number this file mostly reports: it is a sum of costs against a
    /// budget figure, so it is the one arithmetic here whose failure means a decision has to be
    /// reopened rather than a comment updated.
    #[test]
    fn the_app_threads_worst_realistic_frame_is_inside_the_budget() {
        let total = app_thread_total_ns();
        let budget = full_screen_budget_ns();
        assert!(
            total < budget,
            "the ledger totals {:.1} us against a {:.0} us budget — the parallel-compositing \
             ruling was made on this margin and has to be reopened",
            us(total),
            us(budget)
        );
    }

    /// **The parallel-compositing ruling is re-checked and says which way it went.**
    ///
    /// A criterion of the ticket in its own right: *the ruling is re-checked against the real
    /// numbers and either confirmed with the new figure or raised as a finding against the map. It
    /// is **not** implemented here either way.* The assertion is on the two things a re-check owes a
    /// reader — a verdict and a number — because a re-check that concluded nothing would read
    /// exactly like one that was never run.
    #[test]
    fn the_parallel_compositing_ruling_is_rechecked_and_states_a_verdict() {
        let ruling = the_parallel_compositing_ruling();
        assert!(
            ruling.contains("confirmed") || ruling.contains("finding against the map"),
            "the re-check has to land on one of the two answers the ticket allows: {ruling}"
        );
        assert!(
            ruling.contains("43.9 us"),
            "a re-check with no new figure in it is not a re-check: {ruling}"
        );
        assert!(
            ruling.contains("Not implemented"),
            "the ticket says it is not implemented here either way, and the ruling has to say so"
        );
    }

    /// **The ledger says whether a timing report can fail, and it names both of the ones that could.**
    ///
    /// Production ticket 11's own criterion, made into something that costs an edit rather than
    /// staying advice: *whether a timing report can fail is exactly the kind of thing that becomes a
    /// claim if nobody writes it down.* The assertion is on the three things the statement owes a
    /// reader — a verdict, both reports by name, and the mechanism that makes the verdict true —
    /// because a statement that concluded nothing would read exactly like one that was never made.
    #[test]
    fn the_ledger_says_whether_a_timing_report_is_armed() {
        let armed = the_watchdog_over_a_timing_report();
        assert!(
            armed.starts_with("not armed") || armed.starts_with("armed"),
            "the statement has to land on one of the two answers there are: {armed}"
        );
        for report in [
            "what_a_wake_up_costs_with_the_app_thread_parked",
            "the_achieved_rate_lands_under_the_configured_ceiling",
        ] {
            assert!(
                armed.contains(report),
                "a report the detector reaches is not named: {report}"
            );
        }
        assert!(
            armed.contains("permit_slow"),
            "the statement says the reports are not armed and not what makes that true: {armed}"
        );
    }

    /// The table renders, and it names the rows nobody has re-measured.
    ///
    /// **The assertion is on the honesty rather than on the formatting**: a ledger that silently
    /// dropped its prototype rows would print a smaller, better-looking total, and that is the one
    /// way this table can lie.
    #[test]
    fn the_table_says_how_many_rows_are_still_the_prototypes() {
        let table = table();
        assert!(table.contains("PROTOTYPE"), "no row is labelled unmeasured");
        assert!(
            table.contains("still the prototype's figure"),
            "the count of unmeasured rows is not printed"
        );
        // The kind column, which is what makes the budget rows visible as the class they are.
        assert!(
            table.contains(Kind::Budget.word()) && table.contains(Kind::Cost.word()),
            "the table does not say which rows are budgets and which are costs"
        );
    }
}
