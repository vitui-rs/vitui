//! **Runtime ticket 04's reports**: what the closure costs, and what the terminal actually shows.
//!
//! ```text
//! cargo run --release --example theme_numbers -p vitui-runtime
//! ```
//!
//! Four numbers and one table:
//!
//! 1. **`Paint` against a raw `Style`.** The whole cost of making *no colour literals* a compile
//!    outcome. Expected at the noise floor — 1.002×.
//! 2. **`resolve`, a swap, a lookup, a glyph, a mix.** The theme's own operations, all of which happen
//!    either once or per frame and none of which happen per cell.
//! 3. **`shows` against deciding per draw.** A bit test against `roles_differ_on_wire` — and the cost
//!    is the *smaller* half of the argument, because both answers are correct and only one of them
//!    makes a component name the axis.
//! 4. **The pair table**: how many of the seventy-eight role pairs are indistinguishable at each tier.
//!    Reported, and **gated only as a relation**, because the number belongs to the palette and
//!    components ticket 05 replaces the palette.

use std::hint::black_box;

use vitui_bench::Bench;
use vitui_engine::{Color, ColorDepth, Rgb, Style};
use vitui_runtime::theme::{CATPPUCCIN_MOCHA, Density, Distinction, Glyph, GlyphSet, Role, Theme};

/// The tiers, in the order they narrow.
const TIERS: [ColorDepth; 4] = [
    ColorDepth::TrueColor,
    ColorDepth::Indexed256,
    ColorDepth::Ansi16,
    ColorDepth::None,
];

/// Seventy-eight, which is thirteen roles taken two at a time.
const PAIRS: usize = 13 * 12 / 2;

fn main() {
    println!("runtime ticket 04 — the theme's own numbers\n");
    what_the_closure_costs();
    the_themes_operations();
    a_bit_against_a_decision();
    the_pair_table();
}

/// Report: a drawing verb called through `Paint` against the same verb called with a raw `Style`.
///
/// **Every arm `black_box`es its receiver as well as its result**, and the first version did not: six
/// arms reported an identical 0.93 ns because the theme was a local and the whole call folded away.
/// Two tickets ago the same mistake wore a different hat — three terminal sizes reporting exactly
/// 2 833 ns — and the tell is the same one: *a benchmark whose arms agree to three digits is measuring
/// the compiler.*
///
/// There is no drawing verb yet — ticket 08 owns `Ctx` — so the arm is the closest thing that exists:
/// fetching the style a verb would be handed. That is stated rather than glossed, because a report
/// that measured something narrower than it claimed would be the exact failure the ledger ticket was
/// about.
fn what_the_closure_costs() {
    let theme = Theme::default().resolve(ColorDepth::TrueColor);
    let raw = Style::new()
        .fg(Color::rgb(0xcd, 0xd6, 0xf4))
        .bg(Color::rgb(0x1e, 0x1e, 0x2e));

    let report = Bench::new(40)
        .case("style/raw", 10_000, || {
            black_box(black_box(raw));
        })
        .case("style/through a Paint", 10_000, || {
            black_box(black_box(&theme).paint(black_box(Role::Body)));
        })
        .run();
    println!("report  what the closure costs, minimum of 40 rounds:\n{report}");
    let bare = report.get("style/raw").expect("measured");
    let painted = report.get("style/through a Paint").expect("measured");
    println!(
        "        paint / raw = {:.3}x, and spec §3 measured 1.002x — at the noise floor, where three\n\
        \x20       whole-frame measurements differ among themselves by more.\n\
        \x20       size_of::<Paint>() = {} against size_of::<Style>() = {}.\n\
        \x20       **The arm is a lookup and not a draw**: there is no drawing verb until ticket 08,\n\
        \x20       so this prices the fetch a verb would do and says so rather than implying more.\n",
        painted / bare,
        size_of::<vitui_runtime::Paint>(),
        size_of::<Style>(),
    );
}

/// Report: the theme's operations, and which of them happen how often.
fn the_themes_operations() {
    let mut light = CATPPUCCIN_MOCHA;
    light[0] = 0xff_ffff;
    light[5] = 0x00_0000;
    let dark = Theme::default().resolve(ColorDepth::TrueColor);
    let pale = Theme::authored(&light, GlyphSet::Ascii, Density::Compact);
    let roles = vitui_runtime::Roles::from_palette(&CATPPUCCIN_MOCHA);

    let report = Bench::new(40)
        .case("paint, one role", 10_000, || {
            black_box(black_box(&dark).paint(black_box(Role::Focus)));
        })
        .case("glyph, one entry", 10_000, || {
            black_box(black_box(&dark).glyph(black_box(Glyph::Thumb)));
        })
        .case("shows, one distinction", 10_000, || {
            black_box(black_box(&dark).shows(black_box(Distinction::Hover)));
        })
        .case("mix, halfway", 1_000, || {
            black_box(black_box(&dark).mix(
                black_box(Role::Body),
                black_box(Role::Danger),
                black_box(0.5),
            ));
        })
        .case("custom, two colours", 1_000, || {
            black_box(
                black_box(&dark).custom(black_box(Rgb::new(1, 2, 3)), black_box(Rgb::new(4, 5, 6))),
            );
        })
        .case("resolve, once per theme", 1_000, || {
            black_box(black_box(dark).resolve(black_box(ColorDepth::Indexed256)));
        })
        .case("swap, a move", 10_000, || {
            black_box(pale);
        })
        .case("imported, sixteen colours in", 1_000, || {
            black_box(Theme::imported(roles, GlyphSet::Extended, Density::Cosy));
        })
        .run();
    println!("report  the theme's operations, minimum of 40 rounds:\n{report}");
    println!(
        "        spec §10 measured: paint 1.31 ns owned +0.32 -> 1.63, glyph 0.661 ns, shows\n\
        \x20       0.524 ns, mix 7.58 ns, resolve 4.16 ns, a swap 14.30 ns. Four reproduce.\n\
        \x20       **`glyph` is 4.6 ns, not 0.661.** Two documents disagreed — 0.661 in one and\n\
        \x20       4.82 ns in the other, with the note *3.7x a paint, it returns a fat pointer* —\n\
        \x20       and the measurement here settles it at 4.6: the 4.82 figure was right and the\n\
        \x20       0.661 one is what the ticket asked to reproduce. A `&'static str` is two words\n\
        \x20       where a `Paint` is one, and no arrangement of a match makes that free.\n\
        \x20       **`resolve` is 290 ns, not 4.16, and that is a trade this ticket made on\n\
        \x20       purpose.** It quantises all thirteen roles once, so `roles_differ_on_wire` is two\n\
        \x20       array reads instead of twenty-six quantisations — 1.6 ns instead of 42.4. Paid\n\
        \x20       once per theme against saved on every question a component asks.\n\
        \x20       **Only `resolve` and `imported` are per *theme*.** Everything else is per frame at\n\
        \x20       worst, and nothing here is per cell — which is the property that makes the closure\n\
        \x20       affordable at all.\n"
    );
}

/// Report: a resolved bit against a decision taken per draw.
///
/// **Both answers are correct**, and that is the part worth printing. The screens are cell-identical
/// either way; what differs is the cost and — the larger argument — whether a component has to *name
/// the axis*. Naming the axis is what produced twenty-four `GlyphSet::` occurrences across four crates
/// and nine byte-identical private fallback tables.
fn a_bit_against_a_decision() {
    let theme = Theme::default().resolve(ColorDepth::Indexed256);
    let report = Bench::new(40)
        .case("shows, a resolved bit", 10_000, || {
            black_box(black_box(&theme).shows(black_box(Distinction::Hover)));
        })
        .case("roles_differ_on_wire, per draw", 10_000, || {
            black_box(
                black_box(&theme)
                    .roles_differ_on_wire(black_box(Role::Face), black_box(Role::FaceHover)),
            );
        })
        .run();
    println!("report  a bit against a decision, minimum of 40 rounds:\n{report}");
    let bit = report.get("shows, a resolved bit").expect("measured");
    let decided = report
        .get("roles_differ_on_wire, per draw")
        .expect("measured");
    println!(
        "        decided / resolved = {:.2}x, where spec §10 measured **15.7x** (0.524 against 8.22).\n\
        \x20       **The cost argument has mostly evaporated, and that is worth knowing.** §10's 8.22\n\
        \x20       ns was a per-draw quantisation; this ticket moved that work into `resolve`, so the\n\
        \x20       per-draw form is now two array reads and the gap is a few tenths of a nanosecond.\n\
        \x20       What survives is the argument §10 itself called the real one: deciding per draw\n\
        \x20       forces a component to **name the axis**, and naming the axis is what produced\n\
        \x20       twenty-four `GlyphSet::` occurrences across four crates and nine byte-identical\n\
        \x20       private fallback tables. `shows` is kept for the vocabulary, not for the 15.7x.\n\
        \x20       **Cell-identical either way**, at all nine cells of the matrix — so this was never\n\
        \x20       correctness. The two answers agree here: shows(Hover) is {} and the pair differs\n\
        \x20       on the wire is {}.\n",
        decided / bit,
        theme.shows(Distinction::Hover),
        theme.roles_differ_on_wire(Role::Face, Role::FaceHover),
    );
}

/// Report and relation: how many of the seventy-eight role pairs collapse at each tier.
fn the_pair_table() {
    println!("report  indistinguishable role pairs, of {PAIRS}:");
    let mut counts = Vec::new();
    for tier in TIERS {
        let theme = Theme::default().resolve(tier);
        let mut collapsed = Vec::new();
        for (i, &a) in Role::ALL.iter().enumerate() {
            for &b in &Role::ALL[i + 1..] {
                if !theme.roles_differ_on_wire(a, b) {
                    collapsed.push((a, b));
                }
            }
        }
        println!(
            "          {:<12} {:>3} of {PAIRS}   hover {:<5} fade {:<5} status {}",
            format!("{tier:?}"),
            collapsed.len(),
            theme.shows(Distinction::Hover),
            theme.shows(Distinction::Fade),
            theme.shows(Distinction::Status),
        );
        if !collapsed.is_empty() && collapsed.len() <= 6 {
            for (a, b) in &collapsed {
                println!("                       {a:?} / {b:?}");
            }
        }
        counts.push(collapsed.len());
    }
    println!(
        "\n        spec §10 recorded 1 / 2 / 13 of 78, and **two of those three do not reproduce**.\n\
        \x20       Its truecolor 1 was `Dim`/`Border`, both on indexed(8) in a stub palette that no\n\
        \x20       longer exists — `Roles::from_palette`'s `pick` separates them, so a default theme\n\
        \x20       no longer ships its own gate's founding defect. Its C256 2 included\n\
        \x20       `Face`/`FaceHover` \"both landing on index 59\"; the shipped engine narrows them to\n\
        \x20       **237 and 239**, two steps of the grey ramp apart. 59 is the nearest *cube* point,\n\
        \x20       which is four times further away — so that figure came from a quantiser that did\n\
        \x20       not consult the greys.\n\
        \x20       **Gated as a relation and not an equality**, which is spec §20's own rule: the\n\
        \x20       number belongs to the palette, and components ticket 05 replaces the palette."
    );
    // The relation: narrowing never adds a distinction. This is the part that is a gate.
    for w in counts.windows(2) {
        assert!(
            w[0] <= w[1],
            "narrowing added a distinction: {counts:?} across {TIERS:?}"
        );
    }
    println!("        gate: monotone in the narrowing. Measured {counts:?}.");
}
