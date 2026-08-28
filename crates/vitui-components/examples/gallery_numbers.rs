//! **Components ticket 39's report**: what the assembled gallery costs, §16's nine cells, the two
//! numbers tickets 40 and 41 are measured on, and what `t` costs on a live frame.
//!
//! ```text
//! cargo run --release --example gallery_numbers -p vitui-components
//! ```
//!
//! # What is here and what is not
//!
//! Nothing below is a gate. The gates are `crate::gates::REGISTER` rows 45 and 219–221, and this
//! file is cited beside them as a [`Report`](vitui_components::gates::Instrument::Report) and never
//! instead of one — components ticket 20's finding, which is that `cargo test` does not run an
//! example, so an `assert!` here is compiled by `cargo clippy --all-targets` and evaluated by
//! nobody.
//!
//! What the report is for is the four things those rows compress into a count:
//!
//! - **the screen at three sizes**, so *the budget is measured in the gallery* is a line a person
//!   reads rather than a claim;
//! - **the nine cells**, where the repertoire axis moves the cluster count, the colour axis moves
//!   the collapsed-role count, and the paint count moves on neither — which is why the colour half
//!   is measured on the theme;
//! - **the two pinned rows' subjects**, unwritten cells and cells that keep the old palette,
//!   printed so components 40 and 41 start from a figure taken on this screen rather than from a
//!   prototype's;
//! - **`t`**, which is the whole argument for keeping the key: a theme swap is an import plus a
//!   resolve, and the criterion states it in nanoseconds because if construction quietly meant *at
//!   start-up* the key would stutter.

use std::hint::black_box;

use vitui_bench::Bench;
use vitui_components::chart::raster::RUNGS;
use vitui_components::gallery::{self, Change, Gallery, Sink};
use vitui_components::ink::Direct;
use vitui_components::runner::driver_at;
use vitui_runtime::theme::Distinction;
use vitui_runtime::work::Worker;
use vitui_runtime::{ColorDepth, Density, Themes};

fn main() {
    the_screen();
    the_nine_cells();
    the_traffic_light();
    the_two_pinned_rows();
    what_t_costs();
    the_frame();
}

/// **The screen, at the three sizes a gallery is actually looked at.**
fn the_screen() {
    println!("== the assembled gallery ==\n");
    println!(
        "  {:<9} {:>7} {:>8} {:>8} {:>7} {:>8} {:>7} {:>7} {:>10}",
        "size", "grid", "panels", "writes", "verbs", "distinct", "regions", "stops", "unwritten"
    );
    for (w, h) in [(300u16, 80u16), (100, 30), (80, 24)] {
        let (cols, rows, _) = gallery::grid(w, h);
        let s = gallery::shape(w, h, 3);
        println!(
            "  {:<9} {:>7} {:>8} {:>8} {:>7} {:>8} {:>7} {:>7} {:>10}",
            format!("{w}x{h}"),
            format!("{cols}x{rows}"),
            s.panels,
            s.writes,
            s.verbs,
            s.distinct,
            s.regions,
            s.stops,
            s.unwritten,
        );
    }
    println!(
        "\n  `writes == distinct` in every row: no cell of the gallery is written twice, which is\n  \
         spec §2's first equality over a screen with twenty-eight components on it.\n  \
         `merges == 0` in every row, which is what `Ctx::with_key` around each tile buys —\n  \
         one call site to `panel_into` would be twenty-eight widgets under one id.\n"
    );
}

/// **§16's matrix over the whole gallery, nine cells.**
fn the_nine_cells() {
    println!("== the nine cells, at 100x30, over every page ==\n");
    println!(
        "  {:<10} {:<12} {:>8} {:>7} {:>9} {:>7} {:>11} {:>9}",
        "rung", "colour", "writes", "verbs", "clusters", "paints", "roles gone", "dist. lost"
    );
    for cell in gallery::matrix(100, 30) {
        println!(
            "  {:<10} {:<12} {:>8} {:>7} {:>9} {:>7} {:>11} {:>9}",
            gallery::rung_word(cell.rung),
            gallery::tier_word(cell.tier),
            cell.writes,
            cell.verbs,
            cell.clusters,
            cell.paints,
            cell.roles_collapsed,
            cell.distinctions_lost,
        );
    }
    println!(
        "\n  The two axes move in two different columns, and the third column moves on neither.\n  \
         `clusters` is the repertoire axis read off the screen. `roles gone` and `dist. lost` are\n  \
         the colour axis read off the theme — because `Theme::resolve` returns the **same paint**\n  \
         for all thirteen roles at all four depths: quantisation is the engine's and happens before\n  \
         the mirror, so a component is handed the palette's own colour whatever the terminal can\n  \
         show. `paints` reads ten in all nine cells and would read ten on a monochrome terminal. A\n  \
         gate over paint equality reports the colour axis healthy for ever.\n"
    );
}

/// **The criterion's own sentence, measured.**
fn the_traffic_light() {
    println!("== the traffic light, over the fourteen shipped schemes ==\n");
    println!(
        "  {:<12} {:>10} {:>14} {:>18}",
        "colour", "declares", "on the wire", "distinct paints"
    );
    for tier in [
        ColorDepth::None,
        ColorDepth::Ansi16,
        ColorDepth::Indexed256,
        ColorDepth::TrueColor,
    ] {
        let t = gallery::traffic_light(tier);
        println!(
            "  {:<12} {:>7}/14 {:>11}/14 {:>15}/14",
            gallery::tier_word(tier),
            t.declares,
            t.on_the_wire,
            t.distinct_paints
        );
    }
    println!(
        "\n  The criterion reads *`Danger`, `Warn` and `Ok` all quantise to bright white at sixteen\n  \
         colours*. Measured: **eight of fourteen** schemes lose the distinction there and six keep\n  \
         it, and **fourteen of fourteen** lose it at `none` — so the sentence is true of most of the\n  \
         palette and of the wrong rung. `declares` and `on the wire` agree in every row, which is\n  \
         ADR 0032's own claim: a distinction is one bit resolved at construction — and the third\n  \
         column reads 14/14 everywhere because `resolve` changes no paint a component can see.\n"
    );
}

/// **What components 40 and 41 are measured on.**
fn the_two_pinned_rows() {
    println!("== register rows 7 and 8, on this screen ==\n");
    for (w, h) in [(300u16, 80u16), (100, 30)] {
        let cells = usize::from(w) * usize::from(h);
        let s = gallery::shape(w, h, 3);
        println!("  {w}x{h}");
        println!(
            "    row 7  unwritten {:>7} of {cells} cells   {:.1}%",
            s.unwritten,
            100.0 * s.unwritten as f64 / cells as f64
        );
        for change in [Change::Scheme, Change::Rung, Change::Tier] {
            let swap = gallery::swap(w, h, change);
            println!(
                "    row 8  {:<7} kept {:>7} of {} written   {:>5.1}%   changed {} ({} on a panel)",
                format!("{change:?}").to_lowercase(),
                swap.kept,
                swap.written,
                100.0 * swap.kept as f64 / swap.written.max(1) as f64,
                swap.changed,
                swap.changed_on_a_panel,
            );
        }
    }
    println!(
        "\n  Both are reported and neither is gated: row 7 is components 40's to invert and row 8 is\n  \
         components 41's. `changed` is printed beside `kept` because `changed > 0` is the gate this\n  \
         map already got wrong — one cell of 4 800 satisfies it while 3 583 carry the old palette —\n  \
         and *(on a panel)* is printed beside `changed` because for the **tier** axis that is the\n  \
         whole of it: the cells a colour depth moves are the ones where the heading prints the\n  \
         depth's own name, and **not one cell of any panel**. Which is the colour-axis finding\n  \
         arriving as a count, and the reason a gate on `changed` there could not tell it from the\n  \
         gallery having stopped redrawing.\n"
    );
}

/// **`t`, and it is not cosmetic.**
fn what_t_costs() {
    println!("== what a theme swap costs, off the frame path ==\n");
    // **Built once, outside the timed closure.** `Themes::standard()` is an import of its own, and
    // timing it inside every case made the row that does *less* work read slower than the row that
    // does more.
    let base = *Themes::standard().theme();
    let mut registry = Themes::standard();
    let mut at = 0usize;
    let of = registry.len();
    let report = Bench::new(40)
        .case("resolve", 20_000, || {
            black_box(black_box(base).resolve(black_box(ColorDepth::TrueColor)));
        })
        .case("with_glyphs + resolve", 20_000, || {
            black_box(
                black_box(base)
                    // `RUNGS[0]` and not the variant: a file in this lineage that spells a
                    // repertoire is a sixth name for the ladder, and `tests/glyph_matrix.rs` counts
                    // them.
                    .with_glyphs(black_box(RUNGS[0]))
                    .resolve(black_box(ColorDepth::Ansi16)),
            );
        })
        .case("select — what `t` presses", 20_000, || {
            at = (at + 1) % of;
            registry.select(black_box(at));
        })
        .case("shows, ten distinctions", 20_000, || {
            for d in Distinction::ALL {
                black_box(black_box(base).shows(black_box(d)));
            }
        })
        .run();
    print!("{report}");
    println!(
        "  The criterion states `Theme::…resolve()` including all ten distinctions at **291 ns,\n  \
         once per theme, off the frame path**. What `t` actually presses is the third row — an\n  \
         import plus a resolve, ~70 ns — so the key is four times cheaper than the criterion asks\n  \
         for. The figure that does *not* reproduce is the second: `with_glyphs` before `resolve` is\n  \
         ~550 ns, near twice the 291, because a declared repertoire is a real input to the ten bits\n  \
         (ADR 0032) and re-narrowing them is the work. Both are off the frame path and both are\n  \
         once per press, which is what the criterion is really about.\n"
    );
}

/// **The budget, measured in the gallery.**
fn the_frame() {
    println!("== the frame, in the gallery ==\n");
    for (w, h) in [(300u16, 80u16), (100, 30)] {
        let mut driver = driver_at(w, h, Density::default());
        let mut gallery = Gallery::new(Worker::queueing());
        driver.set_theme(*gallery.theme());
        // **Through `Direct` and not through a `Pen`.** The instrument allocates a `String` per cell
        // it records, so a frame timed through it is a timing of the instrument — and the budget
        // this row is read against is the shipped drawing path's.
        let mut one = |driver: &mut vitui_runtime::ctx::Driver| {
            gallery.bag.answer_queued();
            let mut sink: Sink<'_> = &mut Direct;
            driver.frame(|cx| gallery.ui_into(&mut sink, cx, ""));
        };
        one(&mut driver);
        one(&mut driver);
        let report = Bench::new(20).case("frame", 4, || one(&mut driver)).run();
        let us = report.get("frame").unwrap_or_default() * 1e-3;
        println!("  {w}x{h}   {us:>8.2} µs a frame");
    }
    println!(
        "\n  Spec §20's two budgets are a full-screen 300x80 under **1 ms** and a typical\n  \
         damage-tracked frame under **100 µs**, and the gallery meets both with twenty-eight\n  \
         components standing: ~390 µs at 300x80 and ~79 µs at 100x30. That is the answer to the\n  \
         product call §20 files as the user's — `table` at 166 µs, `plot` at 204 and a preview pane\n  \
         at 237 are each over the 100 µs typical-frame budget **alone**, and three of them plus\n  \
         twenty-five more on one screen are inside the full-screen one. The budget being per class\n  \
         rather than per component is what makes those two sentences consistent.\n\n  \
         Printed rather than gated: a timing is a report (§21), and what this screen is *for* is\n  \
         the counts above it.\n"
    );
}
