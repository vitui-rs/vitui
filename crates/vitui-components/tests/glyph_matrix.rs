//! Spec §16's matrix, over the axis this crate can reach — and the barrier that is the other one.
//!
//! > A distinction survives the whole matrix iff it is carried on both axes. (ADR 0032)
//!
//! # The two axes are not equally reachable, and that is a fact about the crate line
//!
//! **The repertoire is reachable**: `vitui_runtime::GlyphSet` is re-exported by the runtime, so a
//! theme can be told about any of the three rungs from here. All three columns of §16's glyph row —
//! **0 / 0 / 36 of 190** — are measured below, exactly.
//!
//! **The tier is not.** `ColorDepth` is `reachable_as: None` in
//! `crates/vitui-runtime/src/line.rs`, and `Theme::resolve` takes one. `vitui-components` depends on
//! `vitui-runtime` and nothing else (§19's C6), so the only tier this crate can hold is the one a
//! theme arrives already resolved for — [`Driver::headless`] hands over truecolor and there is no
//! second value in the crate. That is the same barrier register rows 1, 5 and 29 are stopped by, and
//! it is a *result*: no amount of component code changes it.
//!
//! So the role column, the distinctions-lost column and the traffic light at sixteen colours are
//! measured **in the runtime**, which owns the mechanism and can name both axes —
//! `crates/vitui-runtime/src/theme.rs` and `examples/theme_numbers.rs`. What is here is everything
//! the repertoire decides, and the invariant that joins the two: **nothing carried by a glyph is
//! ever lost**, at any rung, at the tier this crate can see.
//!
//! # Why this file may name the axis at all
//!
//! §16's replacement for the type `Paint` was able to be is a count: `GlyphSet::` occurrences in
//! `vitui-components` == 0, with no private `mod missing`. Something has nonetheless to *declare*
//! three rungs to sweep them — a repertoire is declared and never probed (ADR 0010). **Naming the
//! exception rather than loosening the gate**, which is `crate::gates`'s own refinement 3: the
//! exception is this file and `examples/glyph_numbers.rs`, neither is a component, and
//! [`the_axis_is_named_in_five_files_and_one_of_them_is_a_components`] asserts the list so that a
//! sixth is a deliberate edit rather than a drift back to twenty-four occurrences.
//!
//! **The fourth and fifth arrived together with components ticket 37**, and they are a pair rather
//! than a drift: `tests/golden.rs` sweeps O3's screens across three rungs and
//! `examples/golden_numbers.rs` prints what each rung costs, so both need the `match` that joins
//! `vitui_components::golden::Rung` — three arms, no spelling and no table — to a repertoire. The
//! two copies of that match are two lines each and they are deliberate: an example is not a test,
//! and importing one from the other would make the report a dependency of the gate. Neither is in
//! `src/`, which is where the count is.

use vitui_components::glyphs::{
    Census, GLYPH_PAIRS, ROLE_PAIRS, SIGNAL_PAIRS, census, cross_family_collapses,
    distinctions_lost, glyph_collapses, within_component_collapses,
};
use vitui_runtime::ctx::Driver;
use vitui_runtime::theme::{Distinction, Glyph, GlyphSet, Role, Theme};

/// The three rungs, top first, which is the order §16's table prints them in.
const RUNGS: [GlyphSet; 3] = [GlyphSet::Extended, GlyphSet::Unicode, GlyphSet::Ascii];

/// A theme at `rung`, resolved for **the one tier this crate can reach**.
///
/// It comes off a frame rather than out of a constructor, because that is the only route: the
/// headless driver resolves its theme and hands it over, and `Theme` is `Copy`.
fn at(rung: GlyphSet) -> Theme {
    let mut driver = Driver::headless(40, 10).expect("a sink cannot fail to attach");
    let mut theme = Theme::default();
    driver.frame(|cx| theme = *cx.theme());
    // `with_glyphs` re-narrows the ten bits, so the order of the two builders does not matter and
    // this is a resolved theme at a declared rung rather than a resolved theme with seven stale
    // bits in it.
    theme.with_glyphs(rung)
}

/// **The glyph column of §16's matrix, exactly: 0 / 0 / 36 of 190.**
///
/// It does not move with the tier — a glyph is a cluster and a cluster does not quantise — which is
/// why §16's table repeats the same three numbers down its rows, and why one column measured here
/// is the whole of that axis.
#[test]
fn the_glyph_column_is_zero_zero_and_thirty_six_of_a_hundred_and_ninety() {
    assert_eq!(GLYPH_PAIRS, 190);
    let measured: Vec<usize> = RUNGS
        .iter()
        .map(|&r| glyph_collapses(&at(r)).len())
        .collect();
    assert_eq!(
        measured,
        vec![0, 0, 36],
        "§16 measures 0 / 0 / 36 of 190 collapsing glyph pairs"
    );

    // And **every one of the thirty-six is inside the box family**, which is the identity that makes
    // the pairwise gate unusable and the cross-family one exact: `C(9, 2) == 36`.
    let ascii = at(GlyphSet::Ascii);
    assert_eq!(cross_family_collapses(&ascii), Vec::new());
    assert_eq!(9 * 8 / 2, 36);
}

/// **Within-component cross-family collapse == 0 at every rung** — the gate, and not the pairwise
/// version.
///
/// Run pairwise this fires on `panel`, `table` and everything else with a border: the four box
/// corners are all `+` at ASCII **on purpose**, and a corner collapsing onto a corner loses nothing.
/// What it must catch is C09's — the shadow table's ASCII ellipsis was `>`, exactly an ASCII
/// `ArrowRight`, so **468 truncated labels ended in the collapsed-node marker** and `tree` draws
/// both.
#[test]
fn within_component_cross_family_collapse_is_zero_at_every_rung() {
    for rung in RUNGS {
        assert_eq!(
            within_component_collapses(&at(rung)),
            Vec::new(),
            "{rung:?}: a component draws two glyphs from two families that spell alike"
        );
    }

    // The pairwise form on the rung where it is loud, so the difference between the two gates is a
    // number in this file rather than an argument in a document.
    let ascii = at(GlyphSet::Ascii);
    let pairwise: Vec<(&'static str, Glyph, Glyph)> = vitui_components::INVENTORY
        .iter()
        .flat_map(|c| {
            let mut out = Vec::new();
            // The union of what the row draws and what it hands its caller — architecture 25, and
            // `within_component_collapses`'s own population, for its reason: a collapse is about a
            // screen and a table's caller draws its separators onto the table's.
            let on = vitui_components::glyphs::on_screen(c.id);
            for (i, &a) in on.iter().enumerate() {
                for &b in &on[i + 1..] {
                    if ascii.glyph(a) == ascii.glyph(b) {
                        out.push((c.id, a, b));
                    }
                }
            }
            out
        })
        .collect();
    assert_eq!(
        pairwise.len(),
        42,
        "the pairwise gate fires forty-two times at ASCII — six on `panel` and thirty-six on \
         `table` — and every one of them is a corner or a tee against another corner or tee: \
         {pairwise:?}. **It was forty-three until components architecture 20**, and the one that \
         went was `tree`'s `TeeLeft` against its `BottomLeft`: two entries of a demand set the \
         component drew nothing of, so the pair the pairwise form fired on was a pair no screen \
         ever showed. Thirty-six of the forty-two are still `table`'s and they are the same \
         shape — see `glyphs::UNDRAWN`"
    );
    for (_, a, b) in &pairwise {
        assert_eq!(
            vitui_components::glyphs::family(*a),
            vitui_components::glyphs::family(*b),
            "a pairwise hit that is not within one family would be a real defect the cross-family \
             gate had missed"
        );
    }
}

/// **The ASCII ellipsis is `~`, and `>` is an `ArrowRight`.**
///
/// The defect as a spelling. `tree` demands both entries, so the day the table spells them alike the
/// gate above fires with `tree` named — and the bit the pair carries goes dark, which is the second
/// detector and the one a component reads.
#[test]
fn the_ascii_ellipsis_is_not_an_arrow() {
    let ascii = at(GlyphSet::Ascii);
    assert_eq!(ascii.glyph(Glyph::Ellipsis), "~");
    assert_eq!(ascii.glyph(Glyph::ArrowRight), ">");
    assert_ne!(ascii.glyph(Glyph::Ellipsis), ascii.glyph(Glyph::ArrowRight));

    for rung in RUNGS {
        assert!(
            at(rung).shows(Distinction::Truncation),
            "{rung:?}: a truncated label is not distinguishable from a collapsed node"
        );
    }
}

/// **Nothing carried by a glyph is ever lost**, which is the half of ADR 0032 the repertoire
/// decides.
///
/// §16 puts the losses at `0 / 1 / 2 of 9` across the tiers. The tier axis is the barrier this file
/// opens with, so what is gated here is the invariant that holds down every column: six of the nine
/// distinctions are carried by a glyph pair, and at the tier this crate can reach not one of them
/// goes dark at any rung. A spelling that collapsed inside a carrier would show up here rather than
/// in a screenshot.
///
/// **Ten and seven until components architecture 25**, which struck `Distinction::Guide`: its pair
/// was `(VLine, TeeLeft)` and its drawing is a tree's indent guide, which architecture 20
/// established this library does not make. The denominator moved and the **table** did not, which is
/// the point — a glyph is what a theme can spell and a distinction is what a screen can still tell
/// apart.
#[test]
fn no_glyph_carried_distinction_is_lost_at_any_rung() {
    for rung in RUNGS {
        let theme = at(rung);
        assert_eq!(
            distinctions_lost(&theme),
            Vec::new(),
            "{rung:?}: truecolor at any rung loses nothing, which is §16's first column"
        );
        let carried = Distinction::ALL
            .into_iter()
            .filter(|d| d.carried_by().is_some())
            .count();
        assert_eq!(carried, 6);
        for d in Distinction::ALL {
            if let Some((a, b)) = d.carried_by() {
                assert_ne!(
                    theme.glyph(a),
                    theme.glyph(b),
                    "{rung:?}: {d:?}'s carrier collapsed"
                );
                assert!(theme.shows(d));
            }
        }
    }
}

/// **`shows(Hover)` is R10's `hover_distinct`**, at every rung and at the tier this crate reaches.
///
/// §16's generalisation claim, which is only worth making if the baseline is visibly the same one:
/// the bit a component reads and the fact the theme knows about the two face roles agree, and
/// `hover_interest` follows both. **The repertoire does not touch it**, which is the point — a
/// distinction with no glyph carrier is decided by the palette alone.
#[test]
fn hover_distinct_reproduces_and_the_repertoire_does_not_touch_it() {
    for rung in RUNGS {
        let theme = at(rung);
        assert_eq!(
            theme.shows(Distinction::Hover),
            theme.roles_differ_on_wire(Role::Face, Role::FaceHover),
            "{rung:?}: `shows(Hover)` is not `hover_distinct`"
        );
        assert_eq!(
            theme.hover_interest().wants_hover(),
            theme.shows(Distinction::Hover),
            "{rung:?}: the declared interest and the bit disagree"
        );
    }
    // Three rungs, one answer. A repertoire cannot give a colour distinction back, which is what
    // *carried on both axes* means read the other way round.
    let answers: Vec<bool> = RUNGS
        .iter()
        .map(|&r| at(r).shows(Distinction::Hover))
        .collect();
    assert_eq!(answers, vec![answers[0]; 3]);
}

/// **The role and signal columns, as far as one tier reaches**, with the arithmetic pinned.
///
/// The signal count is the product of the two partitions rather than a third measurement: two
/// signals collapse when their glyph halves spell alike **and** their roles land on one key, so one
/// glyph class of nine multiplies every role class it meets. That is why the ASCII row of §16's
/// table is so much larger than the two above it while the role column is identical.
#[test]
fn the_signal_column_is_the_product_of_the_two_partitions() {
    assert_eq!(ROLE_PAIRS, 78);
    assert_eq!(SIGNAL_PAIRS, 37_128);

    let top: Census = census(&at(GlyphSet::Extended));
    let ascii: Census = census(&at(GlyphSet::Ascii));

    // The role column does not move with the repertoire.
    assert_eq!(top.roles, ascii.roles);

    // At the top rung every glyph class is a singleton, so the signal count is the role count times
    // the twenty-one classes. With the shipped palette that role count is **0** at truecolor — §16's
    // `1 of 78` came from a stub palette in which `Dim` and `Border` were both `indexed(8)`, and
    // `Roles::from_palette`'s `pick` separates them — so the top rung costs nothing at all.
    assert_eq!(
        top.roles, 0,
        "the shipped palette collapses nothing at truecolor"
    );
    assert_eq!(top.signals, top.roles * (Glyph::ALL.len() + 1));
    assert_eq!(top.signals, 0);

    // **And the ASCII rung costs 468 signal pairs with no role pair collapsing at all**, which is
    // the half of §16's `21 → 561` that belongs to the mechanism rather than to the palette: the box
    // class of nine contributes `C(9, 2) == 36` inside each of the thirteen role classes.
    assert_eq!(ascii.signals, 36 * Role::ALL.len());
    assert_eq!(ascii.signals, 468);
    assert!(ascii.signals < SIGNAL_PAIRS);
}

/// **The axis is named in three files, and one of them is a component's.**
///
/// The exception, named rather than the gate loosened. This is the whole-crate half; `src/` is
/// scanned by `vitui_components::gates` and by `vitui_components::inventory`, and the register's is
/// the one that carries the *argument*.
///
/// # The fourth file is `src/series.rs`, and it is the one worth reading twice
///
/// Two of the four are reports about the ladder — this file and `examples/glyph_numbers.rs` — and
/// components ticket 27 added a third of that kind, `examples/series_numbers.rs`, which prints the
/// sub-cell ladder as a table. The fourth is inside the crate's own source, which is exactly what
/// the count is over, and `CONTEXT.md` states both halves of the collision in two adjacent
/// paragraphs: **Repertoire** — *a component branches on it rather than the engine substituting
/// behind its back* — and **Glyph** — *anything failing either rule is not a glyph, it is a branch;
/// **the sub-cell ladders are the case** … a component names no repertoire.*
///
/// So the `src/` half of this test is now *three files must not and one may*, and the one that may
/// is held by `vitui_components::gates`' scan to the exact three lines that may spell a repertoire
/// in it. §21's refinement 3: **name the exception; do not loosen the gate.**
#[test]
fn the_axis_is_named_in_five_files_and_one_of_them_is_a_components() {
    use std::path::{Path, PathBuf};

    // Assembled, not written: a scan for a literal its own source carries finds itself in every file
    // that holds the gate. Both crates have shipped that mistake once each, from opposite sides —
    // always-green in one and always-red in the other.
    let needle = concat!("GlyphSet", "::");

    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let path = entry.expect("a readable entry").path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if path.is_dir() {
                if name != "target" {
                    walk(&path, out);
                }
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    walk(&root, &mut files);
    assert!(files.len() > 20, "only {} files walked", files.len());

    let mut naming: Vec<String> = Vec::new();
    for path in &files {
        let source = std::fs::read_to_string(path).unwrap_or_default();
        // Code, not prose: a line that is a comment names nothing.
        let names = source
            .lines()
            .map(str::trim_start)
            .any(|line| !line.starts_with("//") && line.contains(needle));
        if names {
            naming.push(
                path.strip_prefix(&root)
                    .unwrap_or(path)
                    .to_string_lossy()
                    .replace('\\', "/"),
            );
        }
    }
    naming.sort();
    assert_eq!(
        naming,
        vec![
            "examples/glyph_numbers.rs".to_string(),
            "examples/golden_numbers.rs".to_string(),
            "src/chart/raster.rs".to_string(),
            "tests/glyph_matrix.rs".to_string(),
            "tests/golden.rs".to_string(),
        ],
        "the repertoire is named outside the files that measure it and the one branch that is \
         allowed to. A component names a role and a glyph and never a repertoire — that is the \
         count that replaces the type `Paint` was able to be (§16) — and the sub-cell ladder is \
         the stated exception (see this test's documentation)"
    );
    // **The one file inside the crate's own source, named.** A second would be a second branch, and
    // it would need its own argument rather than this one's.
    let inside: Vec<&String> = naming.iter().filter(|n| n.starts_with("src/")).collect();
    assert_eq!(
        inside,
        vec![&"src/chart/raster.rs".to_string()],
        "a second component source names a repertoire. The exception is `src/chart/raster.rs` and \
         it is argued in `vitui_components::gates`"
    );

    // And the other direction, through the same predicate: a scan that has quietly stopped scanning
    // reports an empty list as loudly as a clean crate does.
    let hostile = format!("    let arrow = {needle}Ascii;");
    assert!(hostile.contains(needle));
    assert!(
        !format!("// {needle} in a comment")
            .trim_start()
            .starts_with(needle)
    );
}

// ── the memo rule, which needs two rungs and therefore lives here ────────────────────────────────

/// **A memo carries the theme in its key iff its value is made of paints *or glyphs*.**
///
/// The word ADR 0032 adds to R20's rule, and the key is the theme's own `Revision` — never the axes.
/// A repertoire swap moves it, so a value made of glyphs is rebuilt with the data untouched.
#[test]
fn a_glyph_memo_keyed_on_the_theme_rebuilds_after_a_repertoire_swap() {
    use vitui_components::glyphs::{glyph_memo_key, gutter};
    use vitui_runtime::{Memo, Revision, Versioned};

    let rows = Versioned::new(vec![0u32; 200]);
    let extended = at(GlyphSet::Extended);
    let mut memo: Memo<Vec<String>> = Memo::new();

    let first = memo
        .get(glyph_memo_key(&extended, rows.revision()), || {
            gutter(&extended, 200)
        })
        .clone();
    assert_eq!(memo.recomputes, 1);

    // The same theme again: a hit, because neither the data nor the theme moved.
    let _ = memo.get(glyph_memo_key(&extended, rows.revision()), || {
        gutter(&extended, 200)
    });
    assert_eq!(memo.recomputes, 1, "the key moved without the theme moving");

    // A repertoire swap. **The data did not change** and the value must still be rebuilt, because it
    // is made of glyphs — which is the whole of the extra word.
    let ascii = extended.with_glyphs(GlyphSet::Ascii);
    assert_ne!(
        ascii.revision(),
        extended.revision(),
        "a declared repertoire is a new revision"
    );
    let second = memo
        .get(glyph_memo_key(&ascii, rows.revision()), || {
            gutter(&ascii, 200)
        })
        .clone();
    assert_eq!(memo.recomputes, 2);
    assert_ne!(first, second, "the two rungs spell the gutter differently");

    // `Revision::UNKNOWN` is preserved rather than folded: *memoise nothing* means nothing,
    // including nothing about the theme.
    assert!(!glyph_memo_key(&extended, Revision::UNKNOWN).is_known());
}

/// **Keyed `(data, tier)` — the plausible fix — survives a palette swap and is wrong after a
/// repertoire one, at half the cost.**
///
/// The negative case, and the reason the key is the theme's own revision and not the axes. A
/// tier-keyed memo is right about colour and blind to the half ADR 0032 adds: a repertoire swap does
/// not move `tier`, so the memo **hits** — which is a lookup rather than a rebuild — and hands back
/// the previous rung's characters. §10 and ADR 0032 price that at **598 wrong characters**, and the
/// corpus is scene 21's: a 200-row tree gutter with two roots, `198 × 3 + 2 × 2`.
#[test]
fn a_tier_keyed_memo_survives_a_palette_swap_and_is_wrong_after_a_repertoire_one() {
    use vitui_components::glyphs::gutter;

    const ROWS: usize = 200;
    let extended = at(GlyphSet::Extended);

    // The plausible key, spelled as the pair it is. The tier is read off the theme rather than
    // named, which is the same barrier this file opens with.
    let plausible = |t: &Theme| (0u64, t.tier());

    let key = plausible(&extended);
    let cached = gutter(&extended, ROWS);
    let mut recomputes = 1usize;

    // A palette swap at the same tier and the same rung — the case the plausible fix gets right,
    // because nothing about the glyphs changed either.
    let paler = Theme::authored(&[0x00_ff_ff_ff; 16], extended.glyphs(), extended.density());
    assert_eq!(
        plausible(&paler),
        (0u64, paler.tier()),
        "a palette swap does not move a tier-keyed key"
    );

    // A repertoire swap. This is the one the map priced.
    let ascii = extended.with_glyphs(GlyphSet::Ascii);
    if plausible(&ascii) != key {
        recomputes += 1;
    }
    assert_eq!(
        recomputes, 1,
        "the tier-keyed memo hit, which is half the cost of the rebuild it owed"
    );

    // And the cached value is the previous rung's characters. The count is over **characters**
    // rather than over rows, which is how §10 states it.
    let owed = gutter(&ascii, ROWS);
    let wrong: usize = cached
        .iter()
        .zip(&owed)
        .map(|(a, b)| a.chars().zip(b.chars()).filter(|(x, y)| x != y).count())
        .sum();
    assert_eq!(
        wrong, 598,
        "§10 prices the repertoire-blind key at 598 wrong characters over scene 21's gutter"
    );

    // The rule held the right way round: the theme-keyed form moves.
    use vitui_components::glyphs::glyph_memo_key;
    use vitui_runtime::Revision;
    assert_ne!(
        glyph_memo_key(&extended, Revision::from_raw(1)),
        glyph_memo_key(&ascii, Revision::from_raw(1)),
        "the theme's own revision moves on a repertoire swap and the axes do not"
    );
}
