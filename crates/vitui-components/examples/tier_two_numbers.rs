//! **Components ticket 34's report**: the six Tier 2 components, what each composes, and the two
//! numbers a gate cannot carry.
//!
//! ```text
//! cargo run --release --example tier_two_numbers -p vitui-components
//! ```
//!
//! # What is here and what is not
//!
//! Nothing below is a gate. The gates are `crate::gates::REGISTER` rows 191–200, and this file is
//! cited beside them as a [`Report`](vitui_components::gates::Instrument::Report) and never instead
//! of one — R15's refinement 2, which components ticket 20 found a whole crate had been breaking:
//! `cargo test` does not run an example, so an `assert!` here is compiled by
//! `cargo clippy --all-targets` and evaluated by nobody.
//!
//! What the report is *for* is the two things the register's rows compress into a sentence:
//!
//! - **the ladder, printed rather than derived** — `1 / 8 / 8` sub-cells a prefix, and the two
//!   contiguous runs of block elements side by side, so that *the ladder is shared and the spelling
//!   is not* is a picture rather than a claim;
//! - **the verb counts the sparkline's relation is written around** — 16 / 13 / 11 over the same
//!   120 writes, which is §21's *never verb equality across sizes* with the three figures under it.

use vitui_components::chart::Series;
use vitui_components::chart::raster::{Geom, Kind, PlotState, RUNGS, cluster, geom};
use vitui_components::composed::{TIER_TWO, survey};
use vitui_components::counters::Tally;
use vitui_components::indicate::{MeterOpts, SparkOpts, meter_into, sparkline_into};
use vitui_components::input::{Toggle, ToggleOpts, toggle_into};
use vitui_components::scroll::Orient;
use vitui_components::structure::{RuleOpts, rule_into};
use vitui_components::text::Justify;
use vitui_components::{INVENTORY, Tier};
use vitui_runtime::Rect;
use vitui_runtime::ctx::{Ctx, Driver};

/// Read a workspace-relative file, which is what the composition survey needs.
fn read(relative: &str) -> String {
    let path =
        std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(relative);
    std::fs::read_to_string(&path).unwrap_or_default()
}

/// A tally over one frame of `f`, on a `w` by `h` sink.
fn tallied(w: u16, h: u16, f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>)) -> Tally {
    let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
    let mut tally = Tally::new();
    driver.frame(|cx| f(&mut tally, cx));
    tally
}

fn main() {
    println!("== the freeze's Tier 2, and what six of the nine compose ==\n");
    let built: Vec<&str> = INVENTORY
        .iter()
        .filter(|c| c.tier == Tier::Two && c.built)
        .map(|c| c.id)
        .collect();
    let unbuilt: Vec<&str> = INVENTORY
        .iter()
        .filter(|c| c.tier == Tier::Two && !c.built)
        .map(|c| c.id)
        .collect();
    println!("built    {built:?}");
    println!("ticket 35 {unbuilt:?}\n");

    for found in survey(read) {
        let row = TIER_TWO
            .iter()
            .find(|r| r.id == found.id)
            .expect("the survey iterates the table");
        println!(
            "{:<10} {:>3} lines · reaches {} · mints none of {} · {}",
            found.id,
            found.lines,
            row.uses.len(),
            row.mints.len(),
            if found.holds() { "holds" } else { "FAILS" }
        );
    }

    println!("\n== the prefix ladder is one and the spellings are two ==\n");
    for (at, rung) in RUNGS.iter().enumerate() {
        println!(
            "  rung {at}   {} sub-cells a prefix",
            geom(Kind::Bars, *rung).sy
        );
    }
    let g = Geom { sx: 1, sy: 8 };
    let upward: String = (0..=8u32)
        .map(|k| cluster(Kind::Bars, g, ((1u16 << k) - 1) as u8))
        .collect();
    // The horizontal run is `crate::indicate`'s and private, so it is spelled out here rather than
    // reached — a report may read a private table only by writing it down, and writing it down is
    // what makes the two runs comparable on one line.
    let rightward = " \u{258F}\u{258E}\u{258D}\u{258C}\u{258B}\u{258A}\u{2589}\u{2588}";
    println!("\n  chart, upward     [{upward}]");
    println!("  meter, rightward  [{rightward}]");
    println!(
        "  they agree at {} of the 7 partial eighths, and at both ends",
        (1..8)
            .filter(|&k| upward.chars().nth(k) == rightward.chars().nth(k))
            .count()
    );

    println!("\n== a meter's two constructions, drawn ==\n");
    for (at, rung) in RUNGS.iter().enumerate() {
        let mut driver = Driver::headless(20, 1).expect("a sink attaches");
        driver.set_theme(
            vitui_runtime::Theme::authored(
                &vitui_runtime::theme::CATPPUCCIN_MOCHA,
                *rung,
                vitui_runtime::theme::Density::default(),
            )
            .resolve(vitui_runtime::ColorDepth::TrueColor),
        );
        let mut pen =
            vitui_components::runner::Pen::over(vitui_components::runner::Canvas::new(20, 1));
        driver.frame(|cx| {
            let area = cx.area();
            meter_into(&mut pen, cx, area, 0.435, &MeterOpts::default());
        });
        println!("  rung {at}  [{}]", pen.into_canvas().row_text(0));
    }

    println!("\n== the sparkline: 120 writes, and the verbs are the data's ==\n");
    for n in [1_000usize, 100_000, 1_000_000] {
        let data = Series::build(n, 1);
        let mut st = PlotState::new();
        let mut driver = Driver::headless(30, 4).expect("a sink attaches");
        let mut tally = Tally::new();
        for _ in 0..20 {
            tally = Tally::new();
            driver.frame(|cx| {
                let area = cx.area();
                sparkline_into(&mut tally, cx, area, &data, &mut st, &SparkOpts::default());
            });
        }
        println!(
            "  {n:>9} points   {} writes   {} verbs   {} fold",
            tally.writes(),
            tally.verbs(),
            st.misses()
        );
    }

    println!("\n== a rule is `fit`'s four skippable parts ==\n");
    for (caption, justify) in [
        ("", Justify::Start),
        (" Section ", Justify::Start),
        (" Section ", Justify::Middle),
        (" Section ", Justify::End),
    ] {
        let tally = tallied(30, 1, |tally, cx| {
            rule_into(
                tally,
                cx,
                Rect::new(0, 0, 30, 1),
                caption,
                &RuleOpts {
                    justify,
                    ..RuleOpts::default()
                },
            );
        });
        println!(
            "  {:<11} {justify:?}   {} verbs   {} writes",
            format!("`{caption}`"),
            tally.verbs(),
            tally.writes()
        );
    }

    println!("\n== the three toggles, and the one that needs no glyph ==\n");
    for kind in Toggle::ALL {
        for (at, rung) in RUNGS.iter().enumerate() {
            let painted = |on: bool| {
                let mut driver = Driver::headless(12, 1).expect("a sink attaches");
                driver.set_theme(
                    vitui_runtime::Theme::authored(
                        &vitui_runtime::theme::CATPPUCCIN_MOCHA,
                        *rung,
                        vitui_runtime::theme::Density::default(),
                    )
                    .resolve(vitui_runtime::ColorDepth::TrueColor),
                );
                let mut pen = vitui_components::runner::Pen::over(
                    vitui_components::runner::Canvas::new(12, 1),
                );
                let mut value = on;
                driver.frame(|cx| {
                    let area = cx.area();
                    toggle_into(
                        &mut pen,
                        cx,
                        area,
                        "x",
                        &mut value,
                        &ToggleOpts {
                            kind,
                            ..ToggleOpts::default()
                        },
                    );
                });
                pen.into_canvas()
            };
            let off = painted(false);
            let on = painted(true);
            println!(
                "  {:<7} rung {at}   off [{}]   on [{}]   {} cells apart",
                kind.id(),
                off.row_text(0),
                on.row_text(0),
                off.diff(&on).cells
            );
        }
    }

    println!("\n== a vertical meter's partial cell is `chart`'s own ==\n");
    let mut driver = Driver::headless(4, 6).expect("a sink attaches");
    let mut pen = vitui_components::runner::Pen::over(vitui_components::runner::Canvas::new(4, 6));
    driver.frame(|cx| {
        let area = cx.area();
        meter_into(
            &mut pen,
            cx,
            area,
            0.4,
            &MeterOpts {
                orient: Orient::Vertical,
                ..MeterOpts::default()
            },
        );
    });
    for line in pen.into_canvas().content() {
        println!("  [{line}]");
    }
}
