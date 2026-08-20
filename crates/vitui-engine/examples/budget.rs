//! The performance budget and spec §14's register, both runnable.
//!
//! Run it: `cargo run --release --example budget -p vitui-engine`
//!
//! # What this file is for
//!
//! Two things the map insisted must live in the repository rather than only in a document.
//!
//! **The scene list is normative, not an appendix** — the reason why is in `../src/scenes.rs`,
//! where the list itself lives. This file owns, prints and times all twelve of them.
//!
//! **The register had twenty-seven entries and nothing to run them against.** Every one of them is
//! in `../src/register.rs` now, each either wired — naming where it runs — or pinned red, naming
//! the implementation ticket that inverts it. No entry is silently absent, because an entry that
//! quietly never arrives is indistinguishable from one that was decided against.
//!
//! # What is gated here, and what is not
//!
//! Four entries: two timings **at the budget** rather than at a measurement (#23, #24), one ratio
//! in a stated band (#20), and one report that is explicitly not load-bearing for anything (#25).
//! `../src/register.rs` states the rule those shapes come from.
//!
//! The counts, the equalities and the per-scene overdraw ratios are inside the library, in
//! `crate::gates`, because they need to see cells and spec §12's public surface deliberately does
//! not show them.
//!
//! # What one number here contains
//!
//! In the deterministic single-thread mode `present` also serialises and writes, which on the
//! shipped three-thread path is the render thread's work and not the app thread's. So every number
//! below is the whole inline round measured against a budget written for the app thread's share,
//! which is strictly harsher than the budget asks — and it is why three scenes are reported rather
//! than gated, with impl 18 named as what gates them.

// One definition, two consumers: the library compiles these under `cfg(test)` and drives them
// against the reference compositor and the terminal model; this file times them. `dead_code` is
// allowed because each consumer uses a different part — the register's `Red` reasons are read here,
// the scenes' `step` return value is read there — and a lint that fires on the half you are not
// looking at teaches people to delete the other half.
#[allow(dead_code)]
#[path = "../src/register.rs"]
mod register;
#[allow(dead_code)]
#[path = "../src/scenes.rs"]
mod scenes;

use std::io::{Result, Write};
use std::time::Duration;

use vitui_bench::{Bench, Report};
use vitui_engine::{Config, Engine, Output, Screen, Style, Surface};

use register::{State, table};
use scenes::{H, Scene, W, scenes, table_two_ways, virtualised_tree};

/// Takes everything, keeps none of it: a recording sink would measure a `Vec` growing.
struct Discard;

impl Write for Discard {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

/// A scene with its layers built, its birth frame gone, and the screen it draws into.
///
/// The pair travels together everywhere below — a scene without its screen cannot be stepped, and
/// a screen without its scene has nothing to draw — so it is a type rather than a tuple.
struct Staged {
    scene: Box<dyn Scene>,
    screen: Screen,
    /// The bench case name, which is the scene's own name unless an arm needs distinguishing.
    case: String,
    iters: u32,
}

impl Staged {
    fn new(mut scene: Box<dyn Scene>, case: &str, iters: u32) -> Staged {
        let (mut screen, _wake) = Engine::new(Config {
            size: (W, H),
            output: Output::Sink(Box::new(Discard)),
            ..Default::default()
        })
        .attach()
        .expect("attaching to a sink cannot fail");
        scene.build(&mut screen);
        // Adding a layer damages its whole rectangle. That is the design, and it is not what any of
        // these cases is measuring.
        screen.present();
        Staged {
            scene,
            screen,
            case: case.to_owned(),
            iters,
        }
    }

    fn of(scene: Box<dyn Scene>) -> Staged {
        let (case, iters) = (scene.name().to_owned(), scene.iters());
        Staged::new(scene, &case, iters)
    }
}

/// Time every staged scene against the others, under the same interference.
///
/// Round-robin and minimum-of-forty, which is what makes a *ratio* between two of them mean
/// something on a machine whose load average wanders: running `A x 40` then `B x 40` lets a
/// background job land entirely inside one arm.
fn measure(staged: &mut [Staged]) -> Report {
    let mut bench = Bench::new(40);
    for s in staged.iter_mut() {
        let (case, iters) = (s.case.as_str(), s.iters);
        let (scene, screen) = (&mut s.scene, &mut s.screen);
        let mut t = 0u32;
        bench = bench.case(case, iters, move || {
            t = t.wrapping_add(1);
            scene.step(screen, t);
            std::hint::black_box(screen.present());
        });
    }
    bench.run()
}

fn main() {
    print_the_scene_list();

    let mut staged: Vec<Staged> = scenes()
        .into_iter()
        .filter(|s| matches!(s.status(), State::Wired { .. }))
        .map(Staged::of)
        .collect();
    let report = measure(&mut staged);
    println!("the twelve scenes, minimum of 40 rounds:\n{report}");

    // Numbers are kept per scene and never summed: the 27x scroll-detector regression the map
    // found was visible only that way, and summed across twelve scenes it is a rounding error.
    the_two_budget_gates(&report);
    the_steady_state_share(&report);
    the_data_volume_invariant();
    the_verb_granularity_rule();

    println!("\nspec §14's register:\n{}", table());
    print_what_is_red();
}

/// Which of §13's two budgets each scene's frame belongs under.
///
/// **The class decides the budget, not the number.** §13 budgets a *full-screen composition* at
/// 1 ms and a *typical damage-tracked frame* at 100 us, and it names the frames that are
/// deliberately outside the incremental budget — a full-screen composite at 40 layers, a
/// row-clearing list scroll — as cliffs by construction rather than as failures. A scene sorted by
/// its measurement instead of by its shape would let anything that grew past 100 us reclassify
/// itself as a full screen, which is a gate that is edited rather than fixed.
const FULL_SCREEN: [&str; 4] = [
    "scrolling-list-rows-cleared",
    "twenty-popups-with-shadows",
    "full-screen-change",
    "every-cell-a-distinct-style",
];

/// Scenes whose measurement on the machine below leaves under 5x, and are therefore reported rather
/// than gated.
///
/// The reason is one line of spec §14 and it is not about these scenes being special: **in the
/// deterministic mode `present` also serialises and writes**, which on the shipped three-thread
/// path is the render thread's work and not the app thread's. So the numbers here are the whole
/// inline round against a budget written for the app thread's share, and the headroom they appear
/// to have left is not the headroom the shipped engine has.
///
/// The runner matters too, and the two this repository has differ: the local GitLab runner is this
/// same machine in a container and reproduces these numbers within a few percent, while the GitHub
/// workflow's `ubuntu-latest` is a shared runner nobody here has measured. Gating at 1.1x of a
/// budget on the second one is a flaky test wearing a budget's clothes, and a flaky gate gets
/// disabled within a month.
///
/// Each is reported with its budget named, and **impl 18 is what gates them** — it is the ticket
/// that moves serialization off the app thread, and only then is there a number the budget is
/// about. Nothing here is silently absent.
const REPORTED_NOT_GATED: [(&str, &str); 3] = [
    ("every-cell-a-distinct-style", "1 ms, full-screen: ~1.04x"),
    ("table-as-list-and-bar-chart", "100 us, typical: ~1.8x"),
    ("virtualised-tree", "100 us, typical: ~3.6x"),
];

/// Gates #23 and #24: §13's two budget figures, over every scene of the class each one is about.
///
/// **Neither gate sits at a measurement.** Both sit at the budget, which is the only kind of timing
/// gate the register allows: a 10% regression is not detectable on a shared runner, and every
/// timing cliff this map actually met was 4.88x, 27x, 37x or 284x. A budget figure is also the one
/// class of number that may not be moved without a new map decision.
///
/// Provenance: **impl 04**, 2026-08-20, Apple M1 Max, rustc 1.97.1, `--release`, unloaded, minimum
/// of forty rounds. Impl 03's numbers are the lineage: 154 us for a full screen and 81 ns for a
/// caret on a bare screen.
///
/// | gated at 1 ms                 |         | gated at 100 us              |          |
/// |-------------------------------|---------|------------------------------|----------|
/// | `scrolling-list-rows-cleared` | 127 us  | `caret-blink`, 40 layers     | 313 ns   |
/// | `twenty-popups-with-shadows`  | 140 us  | `progress-bar-one-percent`   | 768 ns   |
/// | `full-screen-change`          | 161 us  | `scrolling-list-label-only`  | 16.3 us  |
/// |                               |         | `three-dialogs-apart`        | 16.6 us  |
/// |                               |         | `sparse-chart-400-points`    | 18.0 us  |
///
/// The worst headroom gated here is 5.6x, on the sparse chart.
fn the_two_budget_gates(report: &Report) {
    for (case, _) in report.rows().collect::<Vec<_>>() {
        if let Some((_, note)) = REPORTED_NOT_GATED.iter().find(|(n, _)| *n == case) {
            println!("  reported, not gated: {case:<44} {note}, gated at impl 18");
            continue;
        }
        let budget = if FULL_SCREEN.contains(&case) {
            Duration::from_millis(1)
        } else {
            Duration::from_micros(100)
        };
        report.assert_under(case, budget);
    }
}

/// Report #25: 60 fps steady state against 5% of a core.
///
/// **Derived, and a report may never be load-bearing for a gate** (§14's second refinement). The
/// arithmetic is one animated frame's cost times sixty against one second of one core, which is an
/// estimate of the app thread's share and not a process measurement — nothing parks yet, so there
/// is no steady state to measure. Impl 26 carries this report in its own criteria and is where the
/// arithmetic is replaced by a measured steady state; entry #17's idle gate arrives at impl 19.
fn the_steady_state_share(report: &Report) {
    let frame_ns = report
        .get("caret-blink")
        .expect("the caret scene was measured");
    let share = frame_ns * 60.0 / 1e9 * 100.0;
    println!(
        "\nreport #25  60 fps steady state: {share:.4}% of one core, derived from a {frame_ns:.0} \
         ns animated frame\n            budget is 5%; impl 26 replaces this with a measured steady \
         state"
    );
}

/// Gate #20: a scene holding 1M elements must paint in the same time as one holding 1k.
///
/// The invariant the whole engine exists to keep — **frame cost is proportional to visible cells,
/// never to data volume** — and the one property here that has to be a ratio rather than a budget.
/// A scanning implementation walks straight through a 100 us ceiling on a small enough screen:
/// §14's third refinement is that the budget catches a cliff and a growth ratio catches a slope,
/// and neither substitutes for the other.
///
/// Two scene shapes, not one. The tree draws one layer from one column of the data and the table
/// draws two layers from the same rows, which is §14's own "22.35 / 22.97 us at 1k and 1M": a
/// scan that only appears once the same data is walked twice would be invisible on the tree.
///
/// The band is wide on purpose. The claim is "the same time", the failure mode is a scan, and a
/// scan of a thousand times more data is not a 2x. Provenance: impl 04, Apple M1 Max, rustc 1.97.1.
/// Spec §4's granularity rule, and the claim that CJK is not a tax.
///
/// **A report, not a gate**, and the distinction is the backlog's rule rather than modesty: a
/// timing is a gate only at cliff granularity. What *is* gated about these two claims is gated on
/// the mechanism instead — `view::tests::a_full_row_of_cjk_interns_nothing` is an equality on the
/// interner being untouched, which is what makes CJK cheap, and it cannot drift by 10% on a busy
/// runner because it is not a stopwatch.
///
/// The rule this exists to keep visible, because it constrains every component ever written against
/// this API: **the verbs are span-shaped by default, and a component that walks cell by cell is
/// choosing to pay double.** Spec §4 measured 150.4 µs against 290.6 µs for the same 24 000 cells,
/// which is 1.93x.
fn the_verb_granularity_rule() {
    let latin: String = std::iter::repeat_n('m', W as usize).collect();
    let cjk: String = std::iter::repeat_n('漢', W as usize / 2).collect();
    let mut span = Surface::new(W, H);
    let mut per_cell = Surface::new(W, H);
    let mut wide = Surface::new(W, H);

    let report = Bench::new(40)
        .case("text/80 verbs of 300", 200, || {
            let mut v = span.root();
            for y in 0..H as i32 {
                v.text(0, y, &latin, Style::new());
            }
        })
        .case("text/24 000 verbs of 1", 200, || {
            let mut v = per_cell.root();
            for y in 0..H as i32 {
                for x in 0..W as i32 {
                    v.text(x, y, "m", Style::new());
                }
            }
        })
        .case("text/80 verbs of 300, CJK", 200, || {
            let mut v = wide.root();
            for y in 0..H as i32 {
                v.text(0, y, &cjk, Style::new());
            }
        })
        .run();

    println!("the verb granularity rule, minimum of 40 rounds:\n{report}");
    let span = report.get("text/80 verbs of 300").expect("measured");
    let per_cell = report.get("text/24 000 verbs of 1").expect("measured");
    let cjk = report.get("text/80 verbs of 300, CJK").expect("measured");
    println!(
        "            per-cell / span = {:.2}x, spec §4 measured 1.93x — report, not a gate",
        per_cell / span
    );
    println!(
        "            CJK / Latin = {:.2}x over the same 24 000 columns — spec §4 measured 0.92x \n\
         \x20           and that did not reproduce. The mechanism is intact: a wide scalar is still \n\
         \x20           its own handle and a full row of CJK interns nothing, gated by equality in \n\
         \x20           `view::tests::a_full_row_of_cjk_interns_nothing`. What costs is segmentation \n\
         \x20           itself, at 14.3 ns a code point against 3.1 ns for one that takes the ASCII \n\
         \x20           fast path. See impl ticket 06's Progress for the candidate and its owner.",
        cjk / span
    );
    println!();
}

fn the_data_volume_invariant() {
    let mut arms = vec![
        Staged::new(virtualised_tree(1_000), "tree/1k", 200),
        Staged::new(virtualised_tree(100_000), "tree/100k", 200),
        Staged::new(virtualised_tree(1_000_000), "tree/1m", 200),
        Staged::new(table_two_ways(1_000), "table/1k", 200),
        Staged::new(table_two_ways(1_000_000), "table/1m", 200),
    ];
    let report = measure(&mut arms);
    println!("gate #20, the data-volume invariant:\n{report}");

    for (small, large) in [("tree/1k", "tree/1m"), ("table/1k", "table/1m")] {
        let ratio = report.get(large).expect("measured") / report.get(small).expect("measured");
        assert!(
            (0.5..2.5).contains(&ratio),
            "gate #20: {large} costs {ratio:.2}x what {small} costs, band is 0.50x..2.50x. \
             The engine never iterates application data; a ratio outside this band means \
             something started to."
        );
        println!("            {large} / {small} = {ratio:.2}x, band is 0.50x..2.50x");
    }
    println!();
}

/// The list, with what each scene decided, so it is not decoration.
fn print_the_scene_list() {
    println!("spec §14's twelve scenes:");
    for s in scenes() {
        let mark = match s.status() {
            State::Wired { .. } => "     ",
            State::Red { .. } => " RED ",
        };
        println!("  {mark} {:<44} {}", s.name(), s.decided());
    }
    println!();
}

/// What is red, and who inverts it. Printed last, because it is the part that is meant to shrink.
fn print_what_is_red() {
    println!("red on purpose:");
    for s in scenes() {
        if let State::Red { inverted_by, why } = s.status() {
            println!("  scene  {:<44} {inverted_by}", s.name());
            println!("        {why}");
        }
    }
    for e in register::REGISTER {
        if let State::Red { inverted_by, why } = e.state {
            println!("  #{:<4} {:<44} {inverted_by}", e.number, e.property);
            println!("        {why}");
        }
    }
}
