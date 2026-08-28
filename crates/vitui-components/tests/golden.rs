//! Spec §17's obligation O3, over the axis this crate can reach — and the equalities that make the
//! count mean something.
//!
//! > O3 — one golden screen **per construction**, not per matrix cell: count `goldens ==
//! > constructions`, **and** an equality: screens declared identical must be identical.
//!
//! # Why the rung is named here and nowhere else
//!
//! §16's replacement for the type `Paint` was able to be is a count — `GlyphSet::` occurrences in
//! `vitui-components/src` == 0, with no private `mod missing` — and a screen table that could not
//! say which rung it was at would be a table with nine of its thirty-three screens missing. So
//! `vitui_components::golden::Rung` is three arms with no spelling, no table and no ordering claim
//! in it, and [`at`] is the **one match in one file** that joins it to the runtime's repertoire.
//!
//! **Naming the exception rather than loosening the gate** — `crate::gates`'s own refinement 3.
//! This file is a test rather than a component, and `tests/glyph_matrix.rs`'s own
//! `the_axis_is_named_in_five_files_and_one_of_them_is_a_components` asserts the list so that a
//! sixth is a deliberate edit. The fifth is `examples/golden_numbers.rs`, which needs the same
//! `match` to print what each rung costs.

use vitui_components::golden::{self, Rung, SCREENS, Screen};
use vitui_components::obligations::{self, GOLDENS};
use vitui_components::{Component, INVENTORY};
use vitui_runtime::theme::{GlyphSet, Theme};

/// **The one match that joins [`Rung`] to a repertoire.**
///
/// Three arms and no fourth on either side, so a rung that stopped having a repertoire would fail to
/// compile here rather than silently take the default.
fn at(rung: Rung) -> impl FnOnce(Theme) -> Theme {
    move |theme: Theme| {
        theme.with_glyphs(match rung {
            Rung::Ascii => GlyphSet::Ascii,
            Rung::Unicode => GlyphSet::Unicode,
            Rung::Extended => GlyphSet::Extended,
        })
    }
}

/// The screen a scene name belongs to.
fn screen(scene: &str) -> &'static Screen {
    SCREENS
        .iter()
        .find(|s| s.scene == scene)
        .unwrap_or_else(|| panic!("no screen called {scene}"))
}

/// The freeze row a scene is evidence for.
fn row(id: &str) -> &'static Component {
    INVENTORY
        .iter()
        .find(|c| c.id == id)
        .unwrap_or_else(|| panic!("{id} is not a row of the freeze"))
}

/// **Every screen is the picture on file.** The gate O3 exists for: a wrong cell.
#[test]
fn every_screen_is_the_picture_on_file() {
    for s in SCREENS {
        golden::assert_screen(s, at(s.rung));
    }
}

/// **`goldens == constructions`, over the rows that have a component to draw.**
///
/// Three sources and not two: [`SCREENS`] is what is drawn, [`GOLDENS`] is what
/// `vitui_components::obligations::o3` is asked about, and `golden::on_disk` opens the directory.
/// Neither of the first two would notice a golden that had been deleted.
#[test]
fn the_three_sources_agree_about_how_many_screens_there_are() {
    let mut counted = golden::counted();
    counted.sort();
    let mut written: Vec<(&str, u8)> = GOLDENS.to_vec();
    written.sort();
    assert_eq!(counted, written, "`GOLDENS` and the screen table disagree");

    let on_disk = golden::on_disk();
    assert_eq!(
        on_disk.len(),
        SCREENS.len(),
        "{} scene directories against {} screens",
        on_disk.len(),
        SCREENS.len()
    );
    for (scene, files) in &on_disk {
        assert_eq!(
            *files, 1,
            "{scene} has {files} files; a screen is one picture"
        );
        assert!(
            SCREENS.iter().any(|s| s.scene == scene),
            "{scene} is on disk and no screen draws it"
        );
    }

    // And the sum, against the freeze's own column.
    let owed: u32 = INVENTORY
        .iter()
        .filter(|c| c.built)
        .map(|c| u32::from(c.constructions))
        .sum();
    assert_eq!(SCREENS.len() as u32, owed, "the construction sum");
    obligations::o3(GOLDENS).assert_met("O3");
}

/// **Every screen's scene name names its own freeze row**, which is what makes the prefix the join.
#[test]
fn a_scene_name_starts_with_the_row_it_is_evidence_for() {
    for s in SCREENS {
        let r = row(s.id);
        assert!(r.built, "{}: a screen for an unbuilt row", s.id);
        assert!(
            s.scene == s.id || s.scene.starts_with(&format!("{}-", s.id)),
            "{}: the scene {} does not name its row",
            s.id,
            s.scene
        );
        // A row with one construction has one screen and no suffix; a row with more has a suffix
        // on every one of them, so `chart` and `chart-ascii` can never both exist.
        let alone = r.constructions == 1;
        assert_eq!(
            s.scene == s.id,
            alone,
            "{}: {} constructions and the scene is `{}`",
            s.id,
            r.constructions,
            s.scene
        );
    }
}

/// **The equality: screens declared identical are identical, cell for cell.**
///
/// A component that draws the same construction at two rungs would otherwise produce a golden for
/// each, and two files that can only ever drift apart. What ships instead is one file and this
/// assertion — and it is the half that carries the claim, because the count alone would not notice
/// the day a bar chart started spelling itself differently at `Extended`.
#[test]
fn a_rung_that_adds_nothing_draws_the_same_screen() {
    for scene in ["chart-unicode", "meter-unicode", "sparkline-unicode"] {
        let s = screen(scene);
        let (_, unicode) = golden::shot(s, at(Rung::Unicode));
        let (_, extended) = golden::shot(s, at(Rung::Extended));
        // **`Canvas::diff` and not `golden::divergence`**, and the difference is the whole claim: a
        // plane's key is first-appearance order, so two screens whose clusters are swapped one for
        // one have identical planes and `divergence` reports `(0, 0)` — which is precisely the
        // regression this test says it catches. `Canvas::diff` compares clusters and paints.
        // `golden::tests::divergence_is_blind_to_a_rename_and_the_canvas_is_not` is that fact
        // watched, and the header carries the tier so the two files are not comparable line-wise
        // either.
        unicode.diff(&extended).assert_clean(&format!(
            "{scene}: Unicode and Extended draw different screens"
        ));

        // And the other direction, so the equality is not one that holds because nothing was drawn.
        let (_, ascii) = golden::shot(s, at(Rung::Ascii));
        assert!(
            !unicode.diff(&ascii).clean(),
            "{scene}: Ascii draws the same screen as Unicode, so the ladder is not a ladder"
        );
    }
}

/// **A picture at Extended is the picture at Unicode**, which is the second equality the ticket
/// names — and it belongs to no row of the freeze.
///
/// `media` is §14's own *no v1 component*, so there is no `constructions` column to count against
/// and no golden to file; what there is is the claim, and the claim is checkable. A picture's ladder
/// is **1 / 2 / 2** sub-rows (components 29), so the top two rungs offer the same half block and
/// draw the same cells — `Extended == Unicode` firing in the direction that says *nothing was
/// gained*, beside the `Extended != Unicode` on the mark ladder that says the rungs are real.
#[test]
fn a_picture_at_extended_is_the_picture_at_unicode() {
    use vitui_components::picture::{Build, render};

    let unicode = render(Build::correct().at(GlyphSet::Unicode));
    let extended = render(Build::correct().at(GlyphSet::Extended));
    unicode
        .diff(&extended)
        .assert_clean("a picture at Extended is not the picture at Unicode");

    // And the other direction, so the equality is not one that holds because nothing was drawn.
    let ascii = render(Build::correct().at(GlyphSet::Ascii));
    assert!(
        !unicode.diff(&ascii).clean(),
        "Ascii draws the same picture as Unicode, so the ladder is not a ladder"
    );
}

/// **The pairs declared different are different, and their counts are asserted rather than merely
/// non-zero.**
///
/// A count of zero and a count of one are the same `> 0`, and three separate defects on this map
/// scored *different* by drawing something wrong rather than something else. What each number is,
/// and what §16 remembers it as, is [`vitui_components::golden`]'s own report.
#[test]
fn the_rungs_that_change_the_picture_change_it_by_this_much() {
    // **`Canvas::diff` and never `golden::divergence`, and the difference is a number on this very
    // screen.** A plane's key is `GLYPH_KEYS[i]` in first-appearance order, so a count over the
    // planes is blind to a cluster that swapped places with another: `divergence` reports **12**
    // cells here where the cells themselves are **21** apart. That is the review finding this
    // ticket's own instrument carried, and it is why every equality below is over the canvas.
    //
    // **`plot` is the one row whose three rungs are three pictures**, and its Unicode/Extended pair
    // is the one §17 quotes at 882 cells. That is a prototype's screen: this one is 12x4 for the
    // reason `golden::alphabet` gives, and its two block rungs are **21 cells over 3 rows**. The
    // paints are identical in all three, which is the shape of the claim — a rung changes the
    // spelling and never the colour.
    let s = screen("plot-unicode");
    let (uni_theme, uni) = golden::shot(s, at(Rung::Unicode));
    let (ext_theme, ext) = golden::shot(s, at(Rung::Extended));
    let d = uni.diff(&ext);
    assert_eq!((d.cells, d.rows), (21, 3), "plot, Unicode against Extended");
    // The under-count, watched: **12 against 21**, on the screen this row is about.
    let planes = golden::divergence(
        &golden::render(s.scene, 0, &uni_theme, &uni),
        &golden::render(s.scene, 0, &ext_theme, &ext),
        s.size.1,
    );
    assert_eq!(
        planes.cells, 12,
        "the planes stopped under-counting, so the note above has gone stale"
    );

    // **Ascii against the rung each row is drawn at, over all thirty-three screens**: this crate's
    // whole glyph axis as one number.
    let (mut cells, mut rows) = (0usize, 0usize);
    for s in SCREENS {
        let (_, ascii) = golden::shot(s, at(Rung::Ascii));
        let (_, own) = golden::shot(s, at(s.rung));
        let d = ascii.diff(&own);
        cells += d.cells;
        rows += d.rows;
    }
    assert_eq!(
        (cells, rows),
        (399, 55),
        "the crate, Ascii against its own rungs"
    );
}

/// **§16's own screen at §16's own size, and its figure does not reproduce.**
///
/// §16 measures *545 cells and 80 of 80 rows* for the blank fallback and quotes **7 276 cells over
/// 80 of 80 rows** for Ascii against Unicode. The dense screen this crate ships is **1 065 cells
/// over 78 of 80 rows** — asserted as measured, beside the number it is not.
///
/// **The two rows that do not move are the finding inside it**, and they are why the row count is
/// worth carrying: a screen where every row changes says *the repertoire is everywhere*, and one
/// where two rows do not says exactly which two have no glyph on them at all.
#[test]
fn the_dense_screen_between_ascii_and_unicode() {
    let (_, ascii) = dense(Rung::Ascii);
    let (_, unicode) = dense(Rung::Unicode);
    let d = ascii.diff(&unicode);
    assert_eq!(
        (d.cells, d.rows),
        (1065, 78),
        "§16 remembers 7 276 cells over 80 of 80 rows; this screen is what it is"
    );
}

/// **At the ASCII rung a component's own drawing is printable ASCII, and every screen says so.**
///
/// §16's glyph column is *36 of 190 collapsing pairs*, which is a statement about the table.
/// This is the same claim about the **picture**: `golden::alphabet` counts the legend keys a screen
/// needs, and a screen that needs none is a screen with nothing outside printable ASCII on it.
/// Thirty-three of thirty-three.
///
/// **The dense screen keeps exactly one, and it is not a glyph.** `dense::HEADER` is
/// `"vitui — components"` and the em dash is a **fixture's own title**, which is §16's own sentence
/// — *text is legitimately a component's own content and there is nothing to forbid* — arriving as
/// a number: 1 cell of 24 000, at (6, 0), on the densest screen in the crate.
#[test]
fn no_screen_keeps_a_non_ascii_cluster_at_the_ascii_rung() {
    for s in SCREENS {
        let (_, canvas) = golden::shot(s, at(Rung::Ascii));
        assert_eq!(
            golden::alphabet(&canvas),
            0,
            "{}: a component drew a cluster the ASCII rung does not spell",
            s.scene
        );
    }

    let (_, ascii) = dense(Rung::Ascii);
    assert_eq!(golden::alphabet(&ascii), 1, "the dense screen's own title");
    let found: Vec<(u16, u16, String)> = (0..80u16)
        .flat_map(|y| (0..300u16).map(move |x| (x, y)))
        .filter_map(|(x, y)| {
            let c = ascii.get(x, y)?;
            (!c.cluster.is_empty()
                && c.cluster != " "
                && !c.cluster.chars().all(|ch| ch.is_ascii_graphic()))
            .then(|| (x, y, c.cluster.clone()))
        })
        .collect();
    assert_eq!(
        found,
        vec![(6, 0, "—".to_string())],
        "the em dash of the title"
    );
}

/// **The legend has twenty-six keys and braille has two hundred and fifty-six states a cell.**
///
/// The one construction on this map whose alphabet can outgrow the format, and the reason `plot`'s
/// screen is 12x4 rather than the 24x6 every other charting screen uses. Both numbers, so that a
/// change to the rasteriser that made the small screen overflow fails here — with the count — rather
/// than inside the renderer with *not reviewable by eye*.
#[test]
fn a_plot_at_extended_can_outgrow_the_legend() {
    assert_eq!(golden::GLYPH_KEY_COUNT, 26);

    let shipped = screen("plot-extended");
    assert_eq!(shipped.size, (12, 4));
    let (_, small) = golden::shot(shipped, at(Rung::Extended));
    assert_eq!(golden::alphabet(&small), 20, "the shipped plot screen");
    assert!(golden::alphabet(&small) < golden::GLYPH_KEY_COUNT);

    let big = Screen {
        size: (24, 6),
        ..*shipped
    };
    let (_, wide) = golden::shot(&big, at(Rung::Extended));
    assert_eq!(golden::alphabet(&wide), 41, "the same plot at 24x6");
    assert!(golden::alphabet(&wide) > golden::GLYPH_KEY_COUNT);

    // And the two rungs below it stay inside, which is what makes the ceiling braille's and not
    // charting's: the same plot at 24x6 needs 16 keys at Unicode and none at all at Ascii.
    let unicode = Screen {
        size: (24, 6),
        rung: Rung::Unicode,
        ..*shipped
    };
    let (_, u) = golden::shot(&unicode, at(Rung::Unicode));
    assert_eq!(golden::alphabet(&u), 16);
    let (_, a) = golden::shot(&unicode, at(Rung::Ascii));
    assert_eq!(golden::alphabet(&a), 0);
}

/// **The format's owner is named by path, and the four sentences are in the file.**
///
/// A second format cannot appear unnoticed, because the day one does the sentence it was supposed to
/// inherit is no longer where this points. Read out of a **flattened** source, which is
/// `crate::overlay::OWED_SENTENCE`'s arrangement and its reason: a doc comment is not an item and
/// `rustfmt` breaks a phrase across a line.
#[test]
fn the_format_is_the_engines_and_the_owner_is_named_by_path() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(golden::FORMAT_OWNER);
    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", golden::FORMAT_OWNER));
    let flat = source.split_whitespace().collect::<Vec<_>>().join(" ");
    for phrase in golden::FORMAT_PROPERTIES {
        let needle = phrase.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            flat.contains(&needle),
            "{}: `{phrase}` has moved, so the format this crate says it inherits is not the format \
             that file states",
            golden::FORMAT_OWNER
        );
    }
    // The other direction, through the same predicate: a scan that has quietly stopped scanning
    // reports a clean file as loudly as a moved sentence.
    assert!(!flat.contains("Three planes, not two"));

    // **And the bless command is the owner's**, spelled against this crate. One variable, one
    // meaning, no second switch.
    assert!(source.contains("VITUI_BLESS"));
}

/// **Every screen is taken at the default density**, and that is a source scan because there is
/// nothing else it could be.
///
/// Density is theme data and it changes rectangles (spec §3), so a golden taken at another one is a
/// golden of another screen. The header line carries the tier and the rung and has **no field for
/// it** — adding one would be a second format — so what keeps it true is that `golden::shot` names
/// exactly one density and it is the default.
#[test]
fn every_screen_is_taken_at_the_default_density() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/golden.rs"),
    )
    .expect("the module reads");
    let named = named_densities(&source);
    // **Every one of them, and not just the one in `shot`.** The module's own tests build drivers
    // too, and a screen rendered at `Density::Cosy` anywhere here is a golden of another screen —
    // so the assertion is over the whole file rather than over one function, and it is what a
    // second density would have to get past.
    assert!(!named.is_empty(), "the scan found nothing to check");
    for line in &named {
        assert!(
            line.contains("Density::default()"),
            "a density that is not the default: {line}"
        );
    }
    // **And the other direction, through the scan itself and not through a literal.** An arm that
    // asserts a property of its own string holds for any implementation of the predicate, including
    // one whose `filter` has quietly stopped filtering — which is the shape every other negative
    // arm in this crate is written against.
    let hostile = "let a = 1;\n    let d = Density::Cosy;\n// Density::Cosy in a comment\n";
    let caught: Vec<&str> = named_densities(hostile);
    assert_eq!(
        caught,
        vec!["let d = Density::Cosy;"],
        "the scan does not find a density that is not the default, or it reads comments as code"
    );
}

/// The predicate `every_screen_is_taken_at_the_default_density` runs, over any source.
///
/// One definition for the real file and for the hostile one: a negative arm run through a second
/// copy of the logic proves the copy and not the gate.
fn named_densities(source: &str) -> Vec<&str> {
    source
        .lines()
        .map(str::trim)
        .filter(|l| !l.starts_with("//") && l.contains("Density::"))
        .collect()
}

/// §16's own screen, at §16's own size, at a rung.
fn dense(rung: Rung) -> (Theme, vitui_components::runner::Canvas) {
    use vitui_components::dense::{Arm, REQUESTED, draw_into};
    use vitui_components::runner::Pen;

    let mut driver =
        vitui_components::runner::driver_at(300, 80, vitui_runtime::Density::default());
    let theme = at(rung)(*driver.env().theme());
    driver.set_theme(theme);
    let mut pen = Pen::new(300, 80);
    driver.frame(|cx| {
        let _ = draw_into(&mut pen, cx, Arm::Correct, REQUESTED);
    });
    (theme, pen.into_canvas())
}
