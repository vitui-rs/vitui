//! **Ticket 20's gate: the dense frame under the budget, with the headroom written beside it.**
//!
//! This is the runtime's answer to `vitui-engine`'s `examples/budget.rs`, and it exists because
//! register entry 12 was **red**. Both halves of the dense-frame number already existed and neither
//! of them ran: `crate::line::tests::the_frame_measured_inside_the_crate` returns early unless
//! `line=1` is in the environment — correctly, because a timing inside `cargo test` must not become
//! a gate — and `examples/crate_line_numbers.rs` prints it, which no CI job invoked. So the frame
//! was *measured* twice and *gated* nowhere, and the register said `wired` for both until ticket 19
//! made the instruments checkable.
//!
//! A file in `examples/` is compiled by `cargo clippy --all-targets` and evaluated by nothing, so
//! **the gate is not this file — it is this file plus the line in `.gitlab-ci.yml` that runs it.**
//! It is invoked from the `budget` job, beside the engine's, where the release target directory is
//! already warm. Deleting that line turns this back into a report, which is exactly what entry 12
//! was red about.
//!
//! # Why the gate is at the cliff and the budget is reported beside it
//!
//! Spec §20 is unambiguous: *a gate is a count, a ratio, an equality or a compile outcome; a timing
//! is a report*. This is the one shape it exempts, and the exemption is narrow — **a gate only at
//! cliff granularity, with the headroom written next to the number.**
//!
//! **The dense frame has 1.11× of headroom against the 100 µs budget, and that is not enough to
//! gate on.** The engine settled this question first and its answer binds here, because the two
//! crates share a machine, a runner and a spec:
//!
//! > Gating at 1.1x of a budget on [a runner nobody has measured] is a flaky test wearing a
//! > budget's clothes, and a flaky gate gets disabled within a month.
//!
//! So this file does what `budget.rs`'s `REPORTED_NOT_GATED` does for the four engine scenes with
//! under 5× of headroom: it **reports the budget ratio with the budget named, and gates at the
//! cliff**. The distinction is not a softening. A gate at 2× catches the failure the budget is
//! actually about — *a frame that has stopped being proportional to visible cells* — while a gate
//! at 1.11× catches a loaded CI container. Measured: this frame reads 89.9 µs unloaded and **94.7 µs
//! under a concurrent `cargo build`**, a 5.3% penalty against 11% of headroom.
//!
//! **What is refused here is the other repair.** By measurement this frame would sit comfortably in
//! the 1 ms full-screen class, and reclassifying it would buy 11× of headroom and a green budget
//! gate. `budget.rs` names that move and forbids it: *a scene sorted by its measurement instead of
//! by its shape would let anything that grew past 100 µs reclassify itself as a full screen, which
//! is a gate that is edited rather than fixed.* The frame is in the class its shape puts it in, and
//! the thin headroom is reported rather than engineered away.
//!
//! Nothing here is silently absent — the ratio prints on every run, and
//! `ledger::tests::the_dense_frame_is_inside_the_budget` asserts the *recorded* figure against the
//! budget, so the claim is checked by `cargo test` even where the timing is not gated.
//!
//! # What a failure here means, and which layer it names
//!
//! **90.3% of this frame is the engine serialising 7 488 damaged cells**, not runtime work. A bare
//! `assert!(ns < 100_000.0)` would therefore be a runtime gate that fires when the *engine*
//! regresses — a red pipeline pointing at the wrong crate. So the report prints both shares from
//! `crate::ledger`, and the failure message names them: if the runtime's declared rows still sum to
//! ~8.7 µs and the frame has grown, the engine moved; if the rows themselves have grown, this crate
//! did.
//!
//! Provenance: **R 20**, 2026-08-24, Apple M1 Max, macOS 26.5.2, rustc 1.97.1, `--release`,
//! unloaded, minimum of 40 rounds. Three quiet runs read 89.79 / 89.92 / 89.88 µs, a 0.15% spread;
//! under a concurrent `cargo build` the same frame reads 94.2 µs, which is the load penalty a CI
//! container should be expected to show.

use std::hint::black_box;

use vitui_bench::Bench;
use vitui_runtime::ctx::Driver;

// The realistic screen, shared with the allocation gates and the sixteen reports. See
// `src/screen.rs` for why it is included rather than exported.
#[allow(dead_code)]
#[path = "../src/screen.rs"]
mod screen;

// **The ledger, included rather than imported, and that is the whole point of it.** Every ratio
// below divides by a budget figure spec §19 says may not move without a new map decision. Before
// ticket 20 that figure was written out eleven times across `examples/`, once per report. It is
// read from one place now.
#[allow(dead_code)]
#[path = "../src/ledger.rs"]
mod ledger;

use ledger::{Kind, cliffs, dense_frame_ns, frame_budget_ns, runtime_share_ns, table};
use screen::{REGIONS, dense_draw, screen_frame};

fn main() {
    let lto = option_env!("CARGO_PROFILE_RELEASE_LTO").unwrap_or("thin (the manifest's)");
    let budget = frame_budget_ns();

    println!("runtime ticket 20 — the headroom ledger, and the one timing that is a gate");
    println!(
        "machine Apple M1 Max, macOS 26.5.2, --release, minimum of 40 rounds, round robin.\n\
         \x20       the dense screen: 300x80 = 24 000 cells, {REGIONS} interactive regions."
    );
    println!("lto:    {lto}\n");

    // **The shape is asserted before the timing, and it is a count.** A timing about a screen that
    // quietly became a different screen is worse than no timing, because it still looks like one.
    let (splits, lanes) = screen_frame(300, 80);
    assert_eq!(
        (splits, lanes),
        (32, 119),
        "the shape is a count and it has changed, so the timing below is about another screen"
    );

    let mut driver = Driver::headless(300, 80).expect("a sink cannot fail to attach");
    let mut twice = 0;
    driver.frame(|cx| twice = dense_draw(cx, false));
    assert_eq!(
        twice, 5902,
        "the fixture's double-write count has changed, so this is a different screen"
    );

    let report = Bench::new(40)
        .case("frame/text-first", 1, || {
            driver.frame(|cx| {
                black_box(dense_draw(cx, true));
            });
        })
        .case("layout/32-splits", 100, || {
            black_box(screen_frame(300, 80));
        })
        .run();
    let ns = report.get("frame/text-first").expect("measured");

    println!("report  a whole dense frame, minimum of 40 rounds:\n{report}");

    // **The headroom, printed next to the number, which is what §20's exemption requires.**
    let headroom = budget / ns;
    let verb = if ns < budget { "under" } else { "OVER" };
    println!(
        "        {:>8.2} us  {verb} the {:.0} us budget, {headroom:.2}x of headroom",
        ns / 1e3,
        budget / 1e3,
    );

    // **Which layer owns the number**, so a failure names one. See the module documentation.
    let share = runtime_share_ns();
    println!(
        "        {:>8.2} us  of it is this crate's own declared work ({:.1}%)",
        share / 1e3,
        100.0 * share / ns
    );
    println!(
        "        {:>8.2} us  is the engine serialising damaged cells ({:.1}%)",
        (ns - share) / 1e3,
        100.0 * (ns - share) / ns
    );

    println!(
        "\nthe ledger, every row re-measured against the shipped runtime:\n\n{}",
        table()
    );

    println!("the frames deliberately outside the budget, each with its detector:");
    for row in cliffs() {
        assert_eq!(row.kind, Kind::Cliff);
        println!(
            "  {:<24} {:>8.2} us  {}",
            row.what,
            row.value / 1e3,
            if row.value > budget {
                "over the budget, by construction"
            } else {
                "inside the budget, and a cliff by shape rather than by size"
            }
        );
    }

    println!(
        "\n        A cliff is allowed past the budget only because something catches it. The \n\
         \x20       detectors are `Ctx::theme_changed()` for the repaint, `anim::WakeLedger` for \n\
         \x20       the animating frame, and the memo pair key for the swap — each named in the \n\
         \x20       row's `used_at` and checked by `ledger::tests::\n\
         \x20       every_cliff_names_the_detector_that_catches_it`."
    );

    // **THE GATE, at cliff granularity.** See the module documentation for why it is not at the
    // budget: 1.11x of headroom against a 5.3% load penalty is a flaky gate, and the engine's
    // `budget.rs` refuses that trade in as many words. What this catches is the failure the budget
    // is about — a frame that has stopped being proportional to visible cells — and the two shares
    // are in the message so a red pipeline names a layer instead of a crate.
    const CLIFF: f64 = 2.0;
    assert!(
        ns < budget * CLIFF,
        "the dense frame costs {:.2} us, over {CLIFF:.0}x the {:.0} us damage-tracked budget.\n  \
         This crate's declared rows sum to {:.2} us. If that figure is unchanged the engine moved \
         and this is the wrong crate to look in; if it has grown, the runtime did.\n  \
         At this size the frame is no longer proportional to visible cells, which is the property \
         the budget exists to protect.",
        ns / 1e3,
        budget / 1e3,
        share / 1e3,
    );

    // **Reported, not gated, with the budget named** — `budget.rs`'s shape for a scene with under
    // 5x of headroom. This line is the whole of register entry 12's *headroom written beside it*.
    println!(
        "\n        reported, not gated: the dense frame is {:.2}x of the {:.0} us budget \
         ({headroom:.2}x of headroom).\n\
         \x20       Under 5x, so it is reported with its budget named rather than gated at it — \
         see this\n\
         \x20       file's documentation, and `budget.rs`'s REPORTED_NOT_GATED for the four \
         engine scenes\n\
         \x20       that take the same exemption for the same reason.",
        ns / budget,
        budget / 1e3,
    );

    // **The recorded figure and the measured one must still describe the same screen.** The ledger
    // carries 89.875 µs; if a machine reads wildly differently the *ledger* is what is stale, and a
    // gate that passed while its own provenance rotted is the failure mode ticket 19 named. Loose
    // on purpose — a CI container is not this laptop, and 2× is a cliff rather than a tolerance.
    let recorded = dense_frame_ns();
    assert!(
        ns < recorded * 2.0 && ns > recorded * 0.5,
        "the frame measures {:.2} us and the ledger records {:.2} us. One of them is stale, and a \
         gate whose provenance no longer describes the machine is a gate nobody can act on",
        ns / 1e3,
        recorded / 1e3,
    );

    println!("\ngates   all passed. The frame is the gate; the ledger above is the report.");
}
