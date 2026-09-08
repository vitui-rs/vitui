//! **Tier 2's claim as a value: what each of the six is composed of, and a gate that it introduces
//! nothing else.**
//!
//! > **Tier 2 means the component is composed of mechanisms a Tier 1 ticket already measured**, and
//! > the risk in shipping one is that the composition turns out to introduce something new — which
//! > is the claim that turns out to be false when it is false.
//!
//! The tier is not a quality ranking, so a gate over it cannot be a quality judgement either. What
//! is checkable is the **shape of the claim**: each row names the mechanisms its component reaches
//! and the mechanisms it may not mint, and [`survey`] opens the file and answers both halves.
//!
//! # Why a scan, and why the positive half is not optional
//!
//! *There is no field called `anchor`* cannot be written as Rust, and a `compile_fail` naming a type
//! that was never declared passes today and passes again the day somebody declares it under another
//! name — [`crate::input`]'s slider met that first and this is the same answer. **A scan for
//! absences alone goes green when the whole section is deleted**, so every row carries [`Row::uses`]
//! beside [`Row::mints`] and `tests::the_survey_is_watched_failing_in_both_directions` watches each
//! half fire.
//!
//! # The section, and where it ends
//!
//! A module homes more than one component — `input.rs` homes five — so a scan over the whole file
//! would let `select`'s popup state excuse a toggle's. Each row therefore names a **banner**, and
//! the section runs from it to whichever comes first of the next banner, the test module, or a
//! `defective` module. That last terminator is load-bearing: every `defective` module in this crate
//! is a **deliberate** collection of the spellings its component refuses, so a scan that ran into
//! one would report the refusals as the component's own.

use crate::{Component, INVENTORY, Tier};

/// One Tier 2 component's composition claim.
#[derive(Clone, Copy, Debug)]
pub struct Row {
    /// The freeze's id.
    pub id: &'static str,
    /// The file, relative to the workspace root.
    pub file: &'static str,
    /// The first line of its own section.
    pub banner: &'static str,
    /// **What it reaches**, each fragment naming a mechanism a Tier 1 ticket measured.
    ///
    /// Never empty: see this module's header for why a scan for absences alone is vacuous.
    pub uses: &'static [&'static str],
    /// **What it may not mint.** Each fragment is a mechanism that would make the row's tier a lie.
    pub mints: &'static [&'static str],
}

/// **What a `Row`'s scan found.**
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    /// The id, so a failure names the component and not the index.
    pub id: &'static str,
    /// Fragments of [`Row::uses`] the section does not contain.
    pub missing: Vec<&'static str>,
    /// Fragments of [`Row::mints`] the section does contain.
    pub minted: Vec<&'static str>,
    /// How many non-comment lines the section has, so an empty slice cannot read as a pass.
    pub lines: usize,
}

impl Found {
    /// Whether the claim holds.
    pub fn holds(&self) -> bool {
        self.missing.is_empty() && self.minted.is_empty() && self.lines > 0
    }
}

/// **The nine components Tier 2 is, and what each of them is.**
///
/// Six arrived together and the last three — `status_bar`, `pagination` and `form` — with
///A row for a component that does not exist is a row no scan can falsify, so
/// `tests::every_built_tier_two_row_is_surveyed` counts the two sets against each other in both
/// directions; it is what made this table grow rather than agree with itself the day the last three
/// landed.
///
/// **The three composites are the ones to watch.** A composition is *shipped
/// components with no new mechanism*, and it is **the class that requires the most care, because
/// composition without a new mechanism is exactly the claim that turns out to be false when it is
/// false.* So each of the three carries the mechanism it may not mint in the shape that would make
/// its tier a lie: a second clip for the band, a second store or a second navigation model for the
/// pager, and a cursor of its own for the form.
pub const TIER_TWO: &[Row] = &[
    Row {
        id: "checkbox",
        file: "crates/vitui-components/src/input.rs",
        banner: TOGGLES,
        // `state::press` for the face and the deferred award, the theme's own table for the mark.
        uses: &[
            "pub fn checkbox(",
            "crate::state::press_into(",
            "Glyph::Tick",
            "cx.theme().glyph(glyph)",
        ],
        mints: TOGGLE_MINTS,
    },
    Row {
        id: "radio",
        file: "crates/vitui-components/src/input.rs",
        banner: TOGGLES,
        // The same machine and one different glyph. **A radio *set* is not this** — it is
        // `collection` at `Mode::Options`, which is the collapse and a different call.
        uses: &["pub fn radio(", "Glyph::Bullet", "Toggle::Radio"],
        mints: TOGGLE_MINTS,
    },
    Row {
        id: "switch",
        file: "crates/vitui-components/src/input.rs",
        banner: TOGGLES,
        // No glyph at all: the state is two words, a side and a face. See `ToggleOpts::words`.
        uses: &["pub fn switch(", "opts.words", "Toggle::Switch"],
        mints: TOGGLE_MINTS,
    },
    Row {
        id: "meter",
        file: "crates/vitui-components/src/indicate.rs",
        banner: METER,
        // **The ladder is `chart`'s**, the whole cells are the theme's fill pair, and the stripe is
        // the scrollbar's verb — one definition of *write n cells along an axis* in this crate.
        uses: &[
            "pub fn meter(",
            "geom(Kind::Bars, cx.theme().glyphs())",
            "Glyph::Thumb",
            "Glyph::Track",
            "stripe(ink, cx, area, opts.orient",
        ],
        mints: &[
            "struct MeterState",
            "Vec<",
            "custom(",
            "Style {",
            "cx.interact(",
        ],
    },
    Row {
        id: "sparkline",
        file: "crates/vitui-components/src/indicate.rs",
        banner: SPARKLINE,
        // **`chart`'s body reached rather than copied**, and the memo is the caller's `PlotState`.
        uses: &[
            "pub fn sparkline(",
            "body_into(ink, cx, area, raster, row, &body)",
            "st.refresh_keyed(",
            "st.range(",
        ],
        // **No axis machinery at all** — criterion 5, as five needles. `nice_step`, `tick_count`,
        // `tick_value` and `decimals` are `chart::axes`' four exports and `gutter` is the fifth;
        // a sparkline that reached any of them would have a chrome to put the answer in.
        mints: &[
            "gutter(",
            "tick_value(",
            "nice_step(",
            "tick_count(",
            "decimals(",
            "axes::",
            "struct SparkState",
            "cx.interact(",
        ],
    },
    Row {
        id: "status_bar",
        file: "crates/vitui-components/src/structure.rs",
        banner: STATUS_BAR,
        // **The one band construction, with the axis argument taken verbatim.** The clip is the
        // whole of what a band buys over arithmetic, so `sticky` is the needle rather than
        // `Shares`: a bar that read the axis and drew by hand would satisfy a scan for the enum.
        uses: &[
            "pub fn status_bar(",
            "crate::scroll::sticky(",
            "Glyph::VLine",
            "crate::text::fit_into(",
        ],
        mints: &[
            "struct BarState",
            "cx.child(",
            ".scrolled(",
            "cx.scrollable(",
            "custom(",
            "Style {",
        ],
    },
    Row {
        id: "pagination",
        file: "crates/vitui-components/src/collect.rs",
        banner: PAGINATION,
        // **`collection`'s store, its thirteen arms and its one drain loop.** `keyboard(` is the
        // load-bearing needle: it is the function `collection` itself calls, so a pager that reached
        // it cannot have a second navigation model — and `nav::step(` below is the spelling that
        // would mean it had.
        uses: &[
            "pub fn pagination(",
            "keyboard(",
            "Mode::Options",
            "apply(coll.mode",
            "Scan::seek(",
        ],
        mints: &[
            "struct PageState",
            "nav::step(",
            "from_key(",
            "Selection::new()",
            "cx.scrollable(",
        ],
    },
    Row {
        id: "form",
        file: "crates/vitui-components/src/input.rs",
        banner: FORM,
        // **§18 R3's own example, and its three clauses are three needles**: the fields, the
        // navigation, and the ring — which is `ScopeKind::Group` on the way in and `cx.is_focused`
        // on the way back, because the cursor **is** the focus and is read rather than kept.
        uses: &[
            "pub fn form(",
            "field_keyed(",
            "nav::cursor(",
            "ScopeKind::Group",
            "cx.is_focused(",
        ],
        // A cursor of its own is what would make R3 a lie here, and there are three ways to spell
        // one: a type, a store borrowed from the collection, or a hit entry the form declares so it
        // can hold the keyboard itself instead of handing it to a field.
        mints: &[
            "struct FormCursor",
            "CollState",
            "Selection",
            "cx.interact(",
            "keys::text(",
        ],
    },
    Row {
        id: "rule",
        file: "crates/vitui-components/src/structure.rs",
        banner: RULE,
        // `fit`'s four skippable parts, `elide`'s one-cell marker, and the two glyphs of §16's
        // `rule` family.
        uses: &[
            "pub fn rule(",
            "Glyph::HLine",
            "Glyph::VLine",
            "crate::glyphs::elide(",
            "crate::text::pad_rows(",
        ],
        mints: &[
            "struct RuleState",
            "Vec<",
            "custom(",
            "Style {",
            "cx.interact(",
        ],
    },
];

/// The toggles' banner. One section for three components, because they are one machine.
const TOGGLES: &str = "// `checkbox`, `radio` and `switch` — the three Tier 2 toggles";

/// `meter`'s banner.
const METER: &str = "// `meter` — `chart`'s prefix construction at two rungs";

/// `sparkline`'s banner.
const SPARKLINE: &str = "// `sparkline` — `chart` at a small rectangle";

/// `rule`'s banner.
const RULE: &str = "// `rule` — the Tier 2 divider";

/// `status_bar`'s banner.
const STATUS_BAR: &str = "// `status_bar` — the Tier 2 band: `sticky`'s one construction, an axis argument, one hit entry";

/// `pagination`'s banner.
const PAGINATION: &str =
    "// `pagination` — the Tier 2 pager: `collection` at a small length, on the other axis";

/// `form`'s banner.
const FORM: &str =
    "// `form` — R3 example: `field` + `nav::cursor` + the focus ring the draw builds";

/// **What the three toggles may not mint.** One list, because they are one machine and a
/// per-component list would be three places for the same claim to drift.
///
/// A toggle's whole cross-frame fact is the caller's `&mut bool`: there is no state type, no
/// selection store and no allocation.
const TOGGLE_MINTS: &[&str] = &[
    "struct ToggleState",
    "Vec<",
    "custom(",
    "Style {",
    "CollState",
];

/// **The section of `source` a row's banner opens**, and nothing after it.
///
/// See this module's header for the three terminators and why the third one is not tidiness.
pub fn section<'a>(source: &'a str, banner: &str) -> &'a str {
    let Some(from) = source.find(banner) else {
        return "";
    };
    let rest = &source[from + banner.len()..];
    // **Step over the banner's own closing rule**, or every section is empty: a banner is three
    // lines and the third of them is the terminator this function looks for. The guard is what makes
    // it the *closing* rule rather than the next section's opening one — there is no newline between
    // the title line and the rule beneath it.
    let body_at = match rest.find("\n// ═") {
        Some(at) if !rest[..at].contains('\n') => rest[at + 1..]
            .find('\n')
            .map_or(rest.len(), |n| at + 1 + n + 1),
        _ => 0,
    };
    let body = &rest[body_at..];
    let end = ["\n// ═", "\n#[cfg(test)]", "\npub mod defective"]
        .into_iter()
        .filter_map(|t| body.find(t))
        .min()
        .unwrap_or(body.len());
    &body[..end]
}

/// **Run every row's scan.** The value a gate and a report both read.
pub fn survey(read: impl Fn(&str) -> String) -> Vec<Found> {
    TIER_TWO
        .iter()
        .map(|row| {
            let source = read(row.file);
            let body = section(&source, row.banner);
            Found {
                id: row.id,
                missing: row
                    .uses
                    .iter()
                    .copied()
                    .filter(|needle| !crate::dense::declares(body, needle))
                    .collect(),
                minted: row
                    .mints
                    .iter()
                    .copied()
                    .filter(|needle| crate::dense::declares(body, needle))
                    .collect(),
                lines: body
                    .lines()
                    .map(str::trim)
                    .filter(|l| !l.is_empty() && !l.starts_with("//"))
                    .count(),
            }
        })
        .collect()
}

/// The freeze's Tier 2 rows that are built, which is the population [`TIER_TWO`] must cover.
pub fn built_tier_two() -> Vec<&'static Component> {
    INVENTORY
        .iter()
        .filter(|c| c.tier == Tier::Two && c.built)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn read(relative: &str) -> String {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(relative);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// **Criterion 2: each of the six names the mechanisms it composes, and introduces none beyond
    /// them.**
    #[test]
    fn each_tier_two_component_composes_what_it_says_and_mints_nothing() {
        let found = survey(read);
        assert_eq!(found.len(), TIER_TWO.len());
        for f in &found {
            assert!(
                f.lines > 0,
                "`{}`'s section is empty, so its whole scan is watching nothing",
                f.id
            );
            assert!(
                f.missing.is_empty(),
                "`{}` does not reach {:?} — either the composition moved or the needle did",
                f.id,
                f.missing
            );
            assert!(
                f.minted.is_empty(),
                "`{}` mints {:?}, and Tier 2 is *composed of mechanisms a Tier 1 ticket already \
                 measured*",
                f.id,
                f.minted
            );
            assert!(f.holds());
        }
    }

    /// **The survey is watched failing in both directions**, because a scan nobody has watched fail
    /// reports zero for the same reason a broken one would.
    ///
    /// Three arms and not two: a section that is **absent** has to fail separately from one that is
    /// present and wrong, or deleting a component reads as deleting its defects.
    #[test]
    fn the_survey_is_watched_failing_in_both_directions() {
        // The section is gone.
        let gone = survey(|_| String::new());
        assert!(gone.iter().all(|f| !f.holds()));
        assert!(gone.iter().all(|f| f.lines == 0));

        // The section exists and mints something it may not.
        let minted = survey(|file| {
            let mut s = read(file);
            if file.ends_with("indicate.rs") {
                s = s.replace(
                    "    let (raster, row) = st.raster_and_row();",
                    "    let _ = axes::AXIS_PAIRS;\n    let (raster, row) = st.raster_and_row();",
                );
            }
            s
        });
        let spark = minted
            .iter()
            .find(|f| f.id == "sparkline")
            .expect("the row is in the table");
        assert_eq!(spark.minted, vec!["axes::"]);
        assert!(!spark.holds());
        // And every other row is unaffected, so the failure is attributed rather than global.
        assert!(
            minted
                .iter()
                .filter(|f| f.id != "sparkline")
                .all(Found::holds)
        );

        // The section exists and no longer reaches what it claims to.
        let unhooked =
            survey(|file| read(file).replace("crate::state::press_into(", "no_such_helper("));
        let check = unhooked
            .iter()
            .find(|f| f.id == "checkbox")
            .expect("the row is in the table");
        assert_eq!(check.missing, vec!["crate::state::press_into("]);
        assert!(!check.holds());
    }

    /// **The section really does stop where it says it does.**
    ///
    /// The terminator that matters is `pub mod defective`: `input.rs` keeps every spelling it refuses
    /// in one, so a scan that ran past the end of a section would report the refusals as the
    /// component's own. Watched on the shipped files rather than on a fixture, because the fixture
    /// is what would drift.
    #[test]
    fn a_section_stops_before_the_next_banner_and_before_the_refused_spellings() {
        let input = read("crates/vitui-components/src/input.rs");
        let toggles = section(&input, TOGGLES);
        assert!(!toggles.is_empty());
        assert!(
            !toggles.contains("pub fn slider("),
            "the toggle section runs into the slider's"
        );
        assert!(
            !toggles.contains("pub mod defective"),
            "the toggle section runs into the refused spellings"
        );
        assert!(toggles.contains("pub fn switch("));

        let indicate = read("crates/vitui-components/src/indicate.rs");
        let meter = section(&indicate, METER);
        assert!(meter.contains("pub fn meter("));
        assert!(
            !meter.contains("pub fn sparkline("),
            "the meter's section runs into the sparkline's"
        );
        let spark = section(&indicate, SPARKLINE);
        assert!(spark.contains("pub fn sparkline("));
        assert!(!spark.contains("pub fn meter("));

        // A banner nothing carries yields nothing, rather than the whole file.
        assert_eq!(section(&input, "// `nothing` — not a section"), "");
    }

    /// **Every built Tier 2 row is surveyed, and every surveyed row is a built Tier 2 row.**
    ///
    /// The join shipped one file over, on this table instead of on the module
    /// tree: a claim about six rows that silently covered five would be `crate::obligations`'s
    /// vacuity accident with a different shape.
    #[test]
    fn every_built_tier_two_row_is_surveyed() {
        let surveyed: std::collections::BTreeSet<&str> = TIER_TWO.iter().map(|r| r.id).collect();
        let built: std::collections::BTreeSet<&str> =
            built_tier_two().into_iter().map(|c| c.id).collect();
        assert_eq!(
            surveyed, built,
            "the survey and the freeze's built Tier 2 rows disagree"
        );
        assert_eq!(surveyed.len(), 9);
        // **And there is nothing left on the other side.** This assertion was
        // `["status_bar", "pagination", "form"]` for one ticket, which is the shape a row for a
        // component that does not exist has to take: absent from the survey and *named* as absent,
        // so that the day it is built the table has to grow or this fails.
        let unbuilt: std::collections::BTreeSet<&str> = INVENTORY
            .iter()
            .filter(|c| c.tier == Tier::Two && !c.built)
            .map(|c| c.id)
            .collect();
        assert_eq!(unbuilt, std::collections::BTreeSet::new());
    }

    /// **A Tier 2 row that can elide declares `Glyph::Ellipsis`, and one that cannot does not.**
    ///
    /// The `glyphs` column is *the per-component demand set*, and the within-component
    /// cross-family collapse gate runs over it — so a component that draws a marker its column does
    /// not declare is a gate running over less than the component draws. That is the **quiet**
    /// direction: nothing fails, the screen is right, and the collapse the column was meant to catch
    /// is simply not looked for.
    ///
    /// The join is possible here and nowhere else, because [`TIER_TWO`] is the one place in this
    /// crate that carries a component's own **section** of its file. It found a row that had been
    /// under-declared from the start: `rule` goes through `crate::glyphs::elide` and
    /// declared `HLine` and `VLine` alone.
    ///
    /// Both directions, so that adding `Ellipsis` to a row that cannot draw one fails too.
    #[test]
    fn a_tier_two_row_declares_the_ellipsis_it_can_draw_and_no_row_declares_one_it_cannot() {
        // The three calls in this crate that can put `Glyph::Ellipsis` on a screen.
        const ELIDES: [&str; 3] = [
            "crate::glyphs::elide(",
            "crate::glyphs::elided_row_into(",
            "crate::text::fit_into(",
        ];
        let mut can = Vec::new();
        for row in TIER_TWO {
            // **A shared banner cannot answer for one of its rows.** `checkbox`, `radio` and
            // `switch` are one machine in one section, and the freeze separates them on exactly this
            // axis — `switch`'s empty `glyphs` column is the finding rather than
            // a hole, and a section-level answer would put a glyph in it on behalf of the two rows
            // beside it. The three are excluded by construction rather than by name.
            if TIER_TWO.iter().filter(|r| r.banner == row.banner).count() > 1 {
                continue;
            }
            let source = read(row.file);
            let body = section(&source, row.banner);
            assert!(!body.is_empty(), "`{}`'s section is empty", row.id);
            let elides = ELIDES.iter().any(|n| crate::dense::declares(body, n));
            let declared = crate::INVENTORY
                .iter()
                .find(|c| c.id == row.id)
                .expect("a Tier 2 row is a row of the freeze")
                .glyphs
                .contains(&vitui_runtime::Glyph::Ellipsis);
            assert_eq!(
                elides,
                declared,
                "`{}` {} elide and its `glyphs` column {} `Ellipsis`",
                row.id,
                if elides { "can" } else { "cannot" },
                if declared {
                    "declares"
                } else {
                    "does not declare"
                }
            );
            if elides {
                can.push(row.id);
            }
        }
        // Named, so that a row leaving this set is a deliberate edit rather than a quiet one. Four
        // of the six rows with a section of their own can put a marker on a screen; `meter` and
        // `sparkline` cannot, because neither draws a label at all.
        assert_eq!(can, vec!["status_bar", "pagination", "form", "rule"]);
    }

    /// **No role variant is a component row**, which is [`crate::indicate::ROLE_VARIANTS`]'s gate and
    /// is here rather than there only because this is the module the Tier 2 claims live in.
    ///
    /// See `crate::indicate::tests::no_role_variant_is_a_component_row` for the assertion itself.
    #[test]
    fn the_role_variant_list_is_not_empty_and_names_the_freezes_own_words() {
        assert_eq!(crate::indicate::ROLE_VARIANTS.len(), 18);
        for word in ["ghost_button", "status_led", "badge", "banner", "callout"] {
            assert!(crate::indicate::ROLE_VARIANTS.contains(&word));
        }
    }
}
