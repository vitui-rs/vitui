//! **Components ticket 37's report**: O3's screens, what each rung costs on the surface, and the
//! three figures §16 and §17 remember differently.
//!
//! ```text
//! cargo run --release --example golden_numbers -p vitui-components
//! ```
//!
//! # What is here and what is not
//!
//! Nothing below is a gate. The gates are `crate::gates::REGISTER` rows 30 and 213–215, and this
//! file is cited beside them as a [`Report`](vitui_components::gates::Instrument::Report) and never
//! instead of one — R15's refinement 2, which components ticket 20 found a whole crate had been
//! breaking: `cargo test` does not run an example, so an `assert!` here is compiled by
//! `cargo clippy --all-targets` and evaluated by nobody.
//!
//! What the report is *for* is the three things those rows compress into a count:
//!
//! - **the screen table**, one row per construction, with the rectangle, the alphabet the legend
//!   needs and what the ASCII rung costs it — so *thirty-three screens* is thirty-three lines;
//! - **the three divergences**, each printed beside the figure §16 or §17 remembers, because two of
//!   the three do not reproduce and this is where that is said out loud;
//! - **the legend's ceiling**, which is the one place on this map where the *format* and not the
//!   component decides how big a screen may be.

use vitui_components::golden::{self, Rung, SCREENS};
use vitui_components::obligations::{GOLDENS, o3};
use vitui_components::runner::{Canvas, Pen};
use vitui_components::{INVENTORY, Tier};
use vitui_runtime::Density;
use vitui_runtime::theme::{GlyphSet, Theme};

/// The one match that joins [`Rung`] to a repertoire, and it is a copy of `tests/golden.rs`'s.
///
/// **A copy on purpose**, and the copy is two lines: an example is not a test, and importing one
/// from the other would make this file a dependency of the gate. Three arms and no fourth.
fn at(rung: Rung) -> impl FnOnce(Theme) -> Theme {
    move |theme: Theme| {
        theme.with_glyphs(match rung {
            Rung::Ascii => GlyphSet::Ascii,
            Rung::Unicode => GlyphSet::Unicode,
            Rung::Extended => GlyphSet::Extended,
        })
    }
}

/// §16's own screen at §16's own size, at a rung.
fn dense(rung: Rung) -> (Theme, Canvas) {
    use vitui_components::dense::{Arm, REQUESTED, draw_into};

    let mut driver = vitui_components::runner::driver_at(300, 80, Density::default());
    let theme = at(rung)(*driver.env().theme());
    driver.set_theme(theme);
    let mut pen = Pen::new(300, 80);
    driver.frame(|cx| {
        let _ = draw_into(&mut pen, cx, Arm::Correct, REQUESTED);
    });
    (theme, pen.into_canvas())
}

fn main() {
    println!("O3 — one golden screen per construction");
    println!("======================================\n");

    println!(
        "{:<20}  {:<10}  {:>7}  {:>8}  {:>16}",
        "scene", "rung", "size", "alphabet", "ascii costs it"
    );
    let (mut cells, mut rows) = (0usize, 0usize);
    for s in SCREENS {
        let (_, own) = golden::shot(s, at(s.rung));
        let (_, ascii) = golden::shot(s, at(Rung::Ascii));
        // `Canvas::diff` and never `golden::divergence`: a plane's key is first-appearance order,
        // so a count over the planes is blind to a one-for-one rename. See `golden::divergence`.
        let d = ascii.diff(&own);
        cells += d.cells;
        rows += d.rows;
        println!(
            "{:<20}  {:<10}  {:>3}x{:<3}  {:>8}  {:>7} cells {:>2} rows",
            s.scene,
            format!("{:?}", s.rung).to_lowercase(),
            s.size.0,
            s.size.1,
            golden::alphabet(&own),
            d.cells,
            d.rows
        );
    }
    println!(
        "\n{:<20}  {cells} cells over {rows} rows",
        "the whole crate"
    );

    println!("\nThe count, against the freeze");
    println!("-----------------------------");
    let built = INVENTORY.iter().filter(|c| c.built).count();
    let owed: u32 = INVENTORY
        .iter()
        .filter(|c| c.built)
        .map(|c| u32::from(c.constructions))
        .sum();
    println!(
        "  built rows                 {built} of {}",
        INVENTORY.len()
    );
    println!("  constructions owed         {owed}");
    println!("  screens                    {}", SCREENS.len());
    println!(
        "  written evidence           {}",
        GOLDENS.iter().map(|(_, n)| u32::from(*n)).sum::<u32>()
    );
    println!("  on disk                    {}", golden::on_disk().len());
    println!("  O3                         {:?}", o3(GOLDENS));
    // The one row with no screen, and why. `Tier` is printed because it is the reason: `spinner` is
    // the row whose mechanism is a component that owns a clock, and that is a prototype's question.
    for c in INVENTORY.iter().filter(|c| !c.built) {
        println!(
            "  no screen                  {} ({:?}) — a golden of a function that does not exist \
             is not a screen anybody can draw",
            c.id,
            match c.tier {
                Tier::One => "tier 1",
                Tier::Two => "tier 2",
                Tier::Three => "tier 3",
            }
        );
    }

    println!("\nWhat a rung costs, against what the map remembers");
    println!("-------------------------------------------------");
    let plot = SCREENS
        .iter()
        .find(|s| s.scene == "plot-unicode")
        .expect("the plot's Unicode screen");
    let (uni_theme, uni) = golden::shot(plot, at(Rung::Unicode));
    let (ext_theme, ext) = golden::shot(plot, at(Rung::Extended));
    let d = uni.diff(&ext);
    println!(
        "  plot, Unicode vs Extended  {} cells over {} rows   (§17 remembers 882 cells)",
        d.cells, d.rows
    );
    // **The under-count, printed rather than described.** A plane's key is first-appearance order,
    // so a count over the planes is blind to a cluster that swapped places with another.
    let planes = golden::divergence(
        &golden::render(plot.scene, 0, &uni_theme, &uni),
        &golden::render(plot.scene, 0, &ext_theme, &ext),
        plot.size.1,
    );
    println!(
        "    over the planes alone    {} cells   — first-appearance keys are the pattern and not \
         the clusters, which is why no equality rests on them",
        planes.cells
    );

    let (_, ascii) = dense(Rung::Ascii);
    let (_, unicode) = dense(Rung::Unicode);
    let d = ascii.diff(&unicode);
    println!(
        "  dense 300x80, Ascii vs Uni {} cells over {} of 80 rows   (§16 remembers 7 276 over 80 \
         of 80)",
        d.cells, d.rows
    );
    println!(
        "  dense, non-ASCII at Ascii  {}   — `dense::HEADER` is \"vitui — components\" and the em \
         dash is a fixture's own title, not a glyph",
        golden::alphabet(&ascii)
    );

    println!("\nThe legend's ceiling");
    println!("--------------------");
    println!(
        "  glyph keys                 {}   (a twenty-seventh is refused: not reviewable by eye)",
        golden::GLYPH_KEY_COUNT
    );
    let wide = vitui_components::golden::Screen {
        size: (24, 6),
        ..*SCREENS
            .iter()
            .find(|s| s.scene == "plot-extended")
            .expect("the plot's Extended screen")
    };
    let (_, big) = golden::shot(&wide, at(Rung::Extended));
    println!(
        "  plot at Extended, 24x6     {}   — braille is 256 states a cell, so the alphabet grows \
         with the rectangle",
        golden::alphabet(&big)
    );
    println!(
        "  plot at Extended, 12x4     {}   — which is why the screen is the size it is",
        golden::alphabet(&ext)
    );
    println!("\n  the format's owner: {}", golden::FORMAT_OWNER);
}
