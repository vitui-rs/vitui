//! **Components ticket 35's report**: the three Tier 2 composites, and the numbers a gate cannot
//! carry.
//!
//! ```text
//! cargo run --release --example composite_numbers -p vitui-components
//! ```
//!
//! # What is here and what is not
//!
//! Nothing below is a gate. The gates are `crate::gates::REGISTER` rows 201–209, and this file is
//! cited beside two of them as a [`Report`](vitui_components::gates::Instrument::Report) and never
//! instead of one — R15's refinement 2, which components ticket 20 found a whole crate had been
//! breaking: `cargo test` does not run an example, so an `assert!` here is compiled by
//! `cargo clippy --all-targets` and evaluated by nobody.
//!
//! What the report is *for* is the three things the register's rows compress into a sentence:
//!
//! - **what a shared offset does to a band whose content has no rows** — the three values of
//!   `Shares` side by side at four offsets, so that *the axis argument is the band's and not the
//!   bar's* is a table rather than a claim;
//! - **the two densities' form**, with both write counts and both standing-field counts printed
//!   rather than the difference engineered away — spec §3's own arrangement;
//! - **the pager's window**, which is the store's own `offset` read on the other axis, swept across
//!   the strip widths where the page count and the cell width do not divide.

use vitui_components::collect::{CollState, PageOpts, pagination_into};
use vitui_components::composed::{TIER_TWO, survey};
use vitui_components::counters::Tally;
use vitui_components::edit::Text;
use vitui_components::input::{FormOpts, FormState, form_into};
use vitui_components::runner::{Canvas, Pen};
use vitui_components::scroll::Shares;
use vitui_components::structure::{Fill, StatusOpts, status_bar_into};
use vitui_components::{INVENTORY, Tier};
use vitui_runtime::ctx::{Ctx, Driver};
use vitui_runtime::{Density, Rect};

/// Read a workspace-relative file, which is what the composition survey needs.
fn read(relative: &str) -> String {
    let path =
        std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(relative);
    std::fs::read_to_string(&path).unwrap_or_default()
}

/// The six labels the form below is measured over.
const LABELS: [&str; 6] = ["name", "email", "role", "team", "location", "pronouns"];

/// Six empty single-line fields.
fn texts() -> [Text; 6] {
    [
        Text::input(),
        Text::input(),
        Text::input(),
        Text::input(),
        Text::input(),
        Text::input(),
    ]
}

/// A tally over one frame of `f`, on a `w` by `h` sink.
fn tallied(w: u16, h: u16, f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>)) -> Tally {
    let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
    let mut tally = Tally::new();
    driver.frame(|cx| f(&mut tally, cx));
    tally
}

/// The bar drawn at one axis, one fill and one offset, as a surface.
fn bar_at(shares: Shares, fill: Fill, offset: (i32, i32)) -> Canvas {
    let segments = ["ready", "utf-8", "ln 1, col 1", "spaces: 4", "rust"];
    let mut driver = Driver::headless(30, 2).expect("a sink cannot fail to attach");
    let mut pen = Pen::new(30, 2);
    driver.frame(|cx| {
        status_bar_into(
            &mut pen,
            cx,
            Rect::new(0, 0, 30, 2),
            &segments,
            offset,
            &StatusOpts {
                shares,
                fill,
                ..StatusOpts::default()
            },
        );
    });
    pen.end_frame();
    pen.into_canvas()
}

fn main() {
    println!("== the freeze's Tier 2, all nine built ==\n");
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
    println!("built   {built:?}");
    println!("unbuilt {unbuilt:?}\n");

    for found in survey(read) {
        let row = TIER_TWO
            .iter()
            .find(|r| r.id == found.id)
            .expect("the survey iterates the table");
        println!(
            "{:<12} {:>3} lines · reaches {} · mints none of {} · {}",
            found.id,
            found.lines,
            row.uses.len(),
            row.mints.len(),
            if found.holds() { "holds" } else { "FAILS" }
        );
    }

    println!("\n== the axis argument is the band's and not the bar's ==\n");
    println!("  cells differing from `Shares::Neither` at the same offset, 30x2:\n");
    println!("  {:<10} {:<8} {:>6} {:>6}", "fill", "offset", "y", "x");
    for fill in [Fill::Even, Fill::Natural] {
        for offset in [(0, 0), (6, 0), (0, 4), (6, 4)] {
            let base = bar_at(Shares::Neither, fill, offset);
            println!(
                "  {:<10} {:<8} {:>6} {:>6}",
                format!("{fill:?}"),
                format!("{},{}", offset.0, offset.1),
                base.diff(&bar_at(Shares::Y, fill, offset)).cells,
                base.diff(&bar_at(Shares::X, fill, offset)).cells,
            );
        }
    }
    println!(
        "\n  A bar's content is one row derived from its own segments, so the shared *vertical*\n  \
         offset has nothing to move — and the horizontal one has nothing to move either until the\n  \
         content is wider than the band, which is what `Fill::Natural` is."
    );

    println!("\n== `Compact` against `Cosy` on the same form ==\n");
    println!(
        "  {:<9} {:>7} {:>9} {:>8} {:>9}",
        "density", "writes", "distinct", "fields", "dropped"
    );
    for density in [Density::Compact, Density::Cosy] {
        let mut driver = vitui_components::runner::driver_at(30, 8, density);
        let mut st = FormState::new();
        let mut fields = texts();
        let mut tally = Tally::new();
        driver.frame(|cx| {
            let area = cx.area();
            let interior = vitui_components::frame::block_into(
                &mut tally,
                cx,
                area,
                &vitui_components::frame::BlockOpts {
                    title: " who ",
                    ..vitui_components::frame::BlockOpts::default()
                },
            );
            form_into(
                &mut tally,
                cx,
                interior,
                &mut st,
                &LABELS,
                &mut fields,
                &FormOpts::default(),
            );
        });
        let standing = driver.inspect().hits().len();
        println!(
            "  {:<9} {:>7} {:>9} {:>8} {:>9}",
            format!("{density:?}"),
            tally.writes(),
            tally.distinct(),
            standing,
            LABELS.len() - standing
        );
    }
    println!(
        "\n  Spec §3 reports 20 804 / 267 against 20 992 / 263 with four widgets off the bottom.\n  \
         Those are `crate::form`'s three-arm screen, which is components ticket 06's; this is the\n  \
         **component** in a 30x8 panel. What reproduces is the structure — the two densities cover\n  \
         the same screen, neither writes a cell twice, and two rows of padding is two fields."
    );

    println!("\n== a grouped form is one tab stop, and an ungrouped one is every field ==\n");
    println!(
        "  {:<10} {:>6} {:>7} {:>6}",
        "group", "ring", "stops", "walk"
    );
    for group in [true, false] {
        let mut driver = Driver::headless(40, 6).expect("a sink cannot fail to attach");
        let mut st = FormState::new();
        let mut fields = texts();
        driver.frame(|cx| {
            form_into(
                &mut vitui_components::ink::Direct,
                cx,
                Rect::new(0, 0, 40, 6),
                &mut st,
                &LABELS,
                &mut fields,
                &FormOpts {
                    group,
                    ..FormOpts::default()
                },
            );
        });
        let f = driver.inspect();
        println!(
            "  {:<10} {:>6} {:>7} {:>6}",
            group,
            f.ring().len(),
            f.stop_count(),
            f.tab_walk().count()
        );
    }
    println!(
        "\n  And only the grouped arm has arrows at all: a container hears what its children hand\n  \
         back through `Ctx::scope`'s after-the-body moment, and `ScopeKind`'s other two arms are a\n  \
         modal and a code editor."
    );

    println!("\n== the pager's window is the store's own offset ==\n");
    println!(
        "  {:<7} {:>7} {:>7} {:>9} {:>9}",
        "strip", "shown", "writes", "distinct", "regions"
    );
    for w in [7u16, 11, 20, 40, 137] {
        let mut st = CollState::new();
        let opts = PageOpts {
            cell: 3,
            ..PageOpts::default()
        };
        let mut driver = Driver::headless(w, 1).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| {
            pagination_into(&mut tally, cx, Rect::new(0, 0, w, 1), &mut st, 137, &opts);
        });
        // The window is `shown` pages wide, which is what the strip has room for between the two
        // steppers — the pager keeps no second number for it.
        let shown = usize::from((w.saturating_sub(2) / 3).max(1)).min(137);
        println!(
            "  {:<7} {:>7} {:>7} {:>9} {:>9}",
            w,
            shown,
            tally.writes(),
            tally.distinct(),
            driver.inspect().hits().len()
        );
    }
    println!(
        "\n  One hit entry at every width, and `writes == distinct == w` at every width: the gap\n  \
         the pages do not reach is the pager's own, which is `collection`'s tail on the other axis."
    );

    println!("\n== the partition, swept ==\n");
    let mut worst = 0u64;
    for w in 1u16..=60 {
        for h in 1u16..=4 {
            let mut st = FormState::new();
            let mut fields = texts();
            let tally = tallied(w, h, |tally, cx| {
                form_into(
                    tally,
                    cx,
                    Rect::new(0, 0, w, h),
                    &mut st,
                    &LABELS,
                    &mut fields,
                    &FormOpts::default(),
                );
            });
            worst = worst.max(tally.writes() - tally.distinct());
        }
    }
    println!("  240 form rectangles from 1x1 to 60x4: {worst} cells written twice");
}
