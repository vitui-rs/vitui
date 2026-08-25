//! components ticket 05 — the glyph catalogue and the distinction set, as numbers.
//!
//! A **report**, not a gate: R15's refinement 2 says a row may cite one and may never rest on one,
//! and `cargo test` does not run a `fn main`. What gates the numbers below is
//! `tests/glyph_matrix.rs` and `src/glyphs.rs`'s own test module.
//!
//! It prints three things §16 asks for and one it does not:
//!
//! 1. **The table**, twenty entries at three rungs, with the family each belongs to.
//! 2. **The matrix**, over the axis this crate can reach — and the barrier that is the other one.
//! 3. **The demand join**, per component, with the distinctions each set puts it at the mercy of.
//! 4. **What does not reproduce**, said out loud rather than engineered away.
//!
//! This file and `tests/glyph_matrix.rs` are the only two in the crate that name the repertoire
//! type, and `tests::the_axis_is_named_in_four_files_and_one_of_them_is_a_components` asserts exactly that
//! pair.

use vitui_components::INVENTORY;
use vitui_components::glyphs::{
    GLYPH_PAIRS, GlyphFamily, ROLE_PAIRS, SIGNAL_PAIRS, SIGNALS, census, cross_family_collapses,
    distinctions_lost, distinctions_of, family, glyph_collapses, gutter,
    within_component_collapses,
};
use vitui_runtime::ctx::Driver;
use vitui_runtime::theme::{Distinction, Glyph, GlyphSet, Theme};

const RUNGS: [GlyphSet; 3] = [GlyphSet::Extended, GlyphSet::Unicode, GlyphSet::Ascii];

fn at(rung: GlyphSet) -> Theme {
    let mut driver = Driver::headless(40, 10).expect("a sink cannot fail to attach");
    let mut theme = Theme::default();
    driver.frame(|cx| theme = *cx.theme());
    theme.with_glyphs(rung)
}

fn main() {
    println!("components ticket 05 — the glyph catalogue and the distinction set\n");

    // ── 1. the table ─────────────────────────────────────────────────────────────────────────────
    let themes: Vec<Theme> = RUNGS.iter().map(|&r| at(r)).collect();
    println!(
        "report  the table, {} entries at three rungs:",
        Glyph::ALL.len()
    );
    println!(
        "  {:<14}{:<8}{:>10}{:>10}{:>8}",
        "entry", "family", "Extended", "Unicode", "ASCII"
    );
    for g in Glyph::ALL {
        println!(
            "  {:<14}{:<8}{:>10}{:>10}{:>8}",
            format!("{g:?}"),
            family(g).word(),
            themes[0].glyph(g),
            themes[1].glyph(g),
            themes[2].glyph(g)
        );
    }
    println!(
        "\n        Entries whose Unicode and Extended spellings differ: **{}**. That is a\n        \
         decision and not an omission — anything that would distinguish the top two rungs\n        \
         changes the construction and the number of samples asked of the data, so it is a\n        \
         branch and §13 earns the third rung on its own.",
        Glyph::ALL
            .iter()
            .filter(|&&g| themes[0].glyph(g) != themes[1].glyph(g))
            .count()
    );
    for f in GlyphFamily::ALL {
        let n = Glyph::ALL.iter().filter(|&&g| family(g) == f).count();
        print!("        {}: {n}   ", f.word());
    }
    println!("\n        The nine box-drawing entries are all `+` at ASCII **on purpose**.");

    // ── 2. the matrix ────────────────────────────────────────────────────────────────────────────
    println!("\nreport  the matrix, over the axis this crate can reach:");
    println!(
        "  {:<10}{:>8}{:>10}{:>12}{:>10}{:>18}",
        "rung", "tier", "roles/78", "glyphs/190", "signals", "distinctions lost"
    );
    for (rung, theme) in RUNGS.iter().zip(&themes) {
        let c = census(theme);
        println!(
            "  {:<10}{:>8}{:>10}{:>12}{:>10}{:>18}",
            format!("{rung:?}"),
            "True",
            c.roles,
            c.glyphs,
            format!("{}/{SIGNAL_PAIRS}", c.signals),
            format!("{}/{}", c.lost, Distinction::ALL.len())
        );
    }
    println!(
        "\n        `ROLE_PAIRS` {ROLE_PAIRS}   `GLYPH_PAIRS` {GLYPH_PAIRS}   `SIGNALS` {SIGNALS} \
         (twenty glyphs plus *no glyph*, against thirteen roles)"
    );
    for line in [
        "        **The tier column is a barrier, not an omission.** `ColorDepth` is",
        "        `reachable_as: None` in `crates/vitui-runtime/src/line.rs` and `Theme::resolve`",
        "        takes one, so the only tier this crate can hold is the one a theme arrives",
        "        already resolved for. C256 and C16 are measured by `cargo run --release --example",
        "        theme_numbers -p vitui-runtime`, which owns the mechanism and may name both axes.",
    ] {
        println!("{line}");
    }
    println!(
        "        Cross-family glyph collapse: {} / {} / {} against {} / {} / {} pairwise. Every",
        cross_family_collapses(&themes[0]).len(),
        cross_family_collapses(&themes[1]).len(),
        cross_family_collapses(&themes[2]).len(),
        glyph_collapses(&themes[0]).len(),
        glyph_collapses(&themes[1]).len(),
        glyph_collapses(&themes[2]).len(),
    );
    println!(
        "        ASCII collapse is a corner or a tee on another one, which is `C(9, 2) == 36`."
    );
    println!(
        "        Within-component cross-family collapse: {} at every rung, which is the gate.",
        within_component_collapses(&themes[2]).len()
    );

    // ── 3. the demand join ───────────────────────────────────────────────────────────────────────
    println!("\nreport  the demand column, joined against the table:");
    println!(
        "  {:<20}{:<6}{:<38}distinctions it rests on",
        "component", "n", "glyphs"
    );
    for c in INVENTORY {
        if c.glyphs.is_empty() {
            continue;
        }
        let names: Vec<String> = c.glyphs.iter().map(|g| format!("{g:?}")).collect();
        let rests: Vec<String> = distinctions_of(c)
            .iter()
            .map(|d| format!("{d:?}"))
            .collect();
        println!(
            "  {:<20}{:<6}{:<38}{}",
            c.id,
            c.glyphs.len(),
            names.join(" ").chars().take(37).collect::<String>(),
            rests.join(" ")
        );
    }
    let drawing = INVENTORY.iter().filter(|c| !c.glyphs.is_empty()).count();
    println!(
        "\n        {drawing} of {} rows draw at least one glyph, and every one of the {} entries",
        INVENTORY.len(),
        Glyph::ALL.len()
    );
    println!("        has a caller. `tree` needs no entry of its own: its chevron pair *is*");
    println!(
        "        `ArrowDown`/`ArrowRight` and its guides are `VLine`, `TeeLeft`, `BottomLeft`."
    );

    // ── 4. the two rules that leave §16 ──────────────────────────────────────────────────────────
    let extended = &themes[0];
    let ascii = &themes[2];
    let cached = gutter(extended, 200);
    let owed = gutter(ascii, 200);
    let wrong: usize = cached
        .iter()
        .zip(&owed)
        .map(|(a, b)| a.chars().zip(b.chars()).filter(|(x, y)| x != y).count())
        .sum();
    let cells: usize = cached.iter().map(|r| r.chars().count()).sum();
    println!("\nreport  the two rules that leave §16:");
    println!("  the memo rule");
    println!("        A repertoire-blind key — the plausible `(data, tier)` — **hits** after a");
    println!("        repertoire swap, at half the cost of the rebuild it owed, and is wrong in");
    println!("        **{wrong} characters** over scene 21's gutter: 200 rows, two of them roots,");
    println!("        198 x 3 + 2 x 2 = {cells} glyph cells.");
    println!("  the one-cell rule");
    println!(
        "        `Ellipsis` is {} cell at every rung. A three-cell `...` where one was reserved",
        vitui_runtime::layout::text::width(ascii.glyph(Glyph::Ellipsis))
    );
    println!(
        "        moves **3 cells a row** — 468 over 78 rows with two truncating fields a row —"
    );
    println!(
        "        with `writes / verbs / marked` identical either way, which is why the rule is"
    );
    println!("        enforceable on the table and nowhere else.");

    // ── what does not reproduce ──────────────────────────────────────────────────────────────────
    println!("\nreport  what §16 records and this measures:");
    println!("  glyphs 0 / 0 / 36 of 190            reproduces exactly, and the 36 is `C(9, 2)`.");
    println!(
        "  roles 1 / 2 / 13 of 78              **does not.** This palette measures {} at truecolor.",
        census(extended).roles
    );
    for line in [
        "                                      R10's 1 was `Dim`/`Border`, both on indexed(8) in a",
        "                                      stub palette that no longer exists; `pick` separates",
        "                                      them. Gated as a **relation**, because the number",
        "                                      belongs to the palette (§20), and ticket 05",
        "                                      deliberately did not swap the palette to make a",
        "                                      remembered number come back.",
    ] {
        println!("{line}");
    }
    println!(
        "  distinctions lost 0 / 1 / 2 of 10   the first column reproduces here: {} lost at truecolor.",
        distinctions_lost(extended).len()
    );
    for line in [
        "                                      C256 and C16 are behind the tier barrier, and the",
        "                                      runtime measures 1 and **3** rather than 2: this",
        "                                      palette's two face backgrounds land on one index at",
        "                                      sixteen colours, so `Hover` goes with `Fade` and",
        "                                      `Status`.",
        "  a traffic light at C16              `Danger`, `Warn` and `Ok` do not all reach *bright*",
        "                                      white — `Warn` and `Ok` are index 7 and `Danger` is",
        "                                      8 — so one of the two pairs goes, which is what makes",
        "                                      the light monochrome. **No repertoire rescues it**:",
        "                                      `Status` has no glyph half to fall back on, which is",
        "                                      ADR 0032's sentence read the other way round.",
    ] {
        println!("{line}");
    }

    // The **shape**, so that a report which has quietly started measuring something smaller fails
    // rather than looking good. R15's rule, and the reason a report is allowed to carry asserts.
    assert_eq!(Glyph::ALL.len(), 20);
    assert_eq!(Distinction::ALL.len(), 10);
    assert_eq!(glyph_collapses(ascii).len(), 36);
    assert_eq!(cross_family_collapses(ascii).len(), 0);
    assert_eq!(within_component_collapses(ascii).len(), 0);
    assert_eq!(wrong, 598);
}
