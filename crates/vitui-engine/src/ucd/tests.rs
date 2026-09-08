//! The gates over the generated tables.
//!
//! Five of them are equalities over the whole codespace: every code point's value from the trie
//! against the same value read out of a sorted range table built straight from the vendored UCD.
//!
//! What that gates is the **trie** — the stage splits, the block deduplication and the bit
//! packing, which is where the machinery is and where a bug would be silent. It does not gate the
//! *parse*: both halves read the UCD line format the same way, and the default-wide block list
//! from `EastAsianWidth.txt`'s header is stated in both. A parser bug would pass. The conformance
//! suite is what covers that flank, because its expectations come from Unicode rather than from
//! us.
//!
//! The rest pin the four cases the terminal survey named, each with the correct answer rather
//! than the popular one.

use std::fs;

use super::tables::UCD_VERSION;
use super::{
    Break, Columns, Conjunct, Cursor, aux_class, break_class, cluster_width, clusters, tables,
    width_class,
};

/// The vendored UCD, read at test time rather than embedded: the same bytes `build.rs` parsed.
fn ucd(name: &str) -> String {
    let path = format!("{}/ucd/{name}", env!("CARGO_MANIFEST_DIR"));
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"))
}

/// One code point range and its fields, from the shared UCD property-file format.
struct Row {
    first: u32,
    last: u32,
    fields: Vec<String>,
}

fn rows(text: &str) -> Vec<Row> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or_default().trim();
        if line.is_empty() {
            continue;
        }
        let mut fields = line.split(';').map(|f| f.trim().to_owned());
        let range = fields.next().unwrap_or_default();
        let (first, last) = match range.split_once("..") {
            Some((a, b)) => (hex(a), hex(b)),
            None => (hex(&range), hex(&range)),
        };
        out.push(Row {
            first,
            last,
            fields: fields.collect(),
        });
    }
    out
}

fn hex(s: &str) -> u32 {
    u32::from_str_radix(s.trim(), 16).unwrap_or_else(|e| panic!("bad code point {s:?}: {e}"))
}

/// Every code point, as a `char`. The surrogate range is not one and is skipped.
fn code_points() -> impl Iterator<Item = char> {
    (0..=0x10_FFFF_u32).filter_map(char::from_u32)
}

/// A sorted range table, searched rather than indexed — deliberately not the shape under test.
struct Ranges {
    ranges: Vec<(u32, u32, String)>,
}

impl Ranges {
    fn of(text: &str, property: Option<&str>) -> Self {
        let mut ranges: Vec<_> = rows(text)
            .into_iter()
            .filter_map(|row| {
                let value = match property {
                    // `DerivedCoreProperties.txt` and `emoji-data.txt` carry many properties, so
                    // the name is field 0 and the value, when there is one, is field 1.
                    Some(name) => {
                        if row.fields.first().map(String::as_str) != Some(name) {
                            return None;
                        }
                        row.fields
                            .get(1)
                            .cloned()
                            .unwrap_or_else(|| "Yes".to_owned())
                    }
                    None => row.fields.first()?.clone(),
                };
                Some((row.first, row.last, value))
            })
            .collect();
        ranges.sort_by_key(|r| r.0);
        Self { ranges }
    }

    fn get(&self, cp: char) -> Option<&str> {
        let cp = cp as u32;
        let i = self.ranges.partition_point(|r| r.0 <= cp).checked_sub(1)?;
        let (_, last, value) = &self.ranges[i];
        (cp <= *last).then_some(value.as_str())
    }
}

mod segmentation {
    use super::*;

    /// The whole of `GraphemeBreakTest.txt`, every line, with the failing line's own text in the
    /// message. This is the gate: UAX #29 is not a rule set anybody gets right by reading.
    #[test]
    fn agrees_with_the_conformance_suite_on_every_line() {
        let text = ucd("GraphemeBreakTest.txt");
        let mut checked = 0;

        for line in text.lines() {
            let case = line.split('#').next().unwrap_or_default().trim();
            if case.is_empty() {
                continue;
            }

            let mut subject = String::new();
            let mut expected = Vec::new();
            for token in case.split_whitespace() {
                match token {
                    "÷" => expected.push(true),
                    "×" => expected.push(false),
                    hex_cp => subject.push(
                        char::from_u32(hex(hex_cp))
                            .unwrap_or_else(|| panic!("{hex_cp} is not a code point: {line}")),
                    ),
                }
            }
            // The trailing marker is the break at end of text, which the cursor never reports.
            expected.pop();

            let mut cursor = Cursor::new();
            let actual: Vec<bool> = subject.chars().map(|cp| cursor.is_break(cp)).collect();

            assert_eq!(actual, expected, "GraphemeBreakTest.txt: {line}");
            checked += 1;
        }

        assert_eq!(checked, 766, "the conformance suite changed size");
    }

    /// The same suite through the iterator, which is where the two shortcuts live: the ASCII fast
    /// path, and the cursor carried across a boundary instead of restarted at one. Neither is
    /// touched by a conformance pass over [`Cursor`] alone, and both were introduced for speed —
    /// which is exactly the kind of change that passes every hand-written test and fails here.
    #[test]
    fn the_iterator_agrees_with_the_conformance_suite_on_every_line() {
        let text = ucd("GraphemeBreakTest.txt");

        for line in text.lines() {
            let case = line.split('#').next().unwrap_or_default().trim();
            if case.is_empty() {
                continue;
            }

            let mut subject = String::new();
            let mut expected: Vec<String> = Vec::new();
            for token in case.split_whitespace() {
                match token {
                    "÷" => expected.push(String::new()),
                    "×" => {}
                    hex_cp => {
                        let cp = char::from_u32(hex(hex_cp))
                            .unwrap_or_else(|| panic!("{hex_cp} is not a code point: {line}"));
                        subject.push(cp);
                        expected
                            .last_mut()
                            .unwrap_or_else(|| panic!("case does not open with a break: {line}"))
                            .push(cp);
                    }
                }
            }
            // The trailing marker opens an empty cluster past the end of text.
            expected.pop();

            let actual: Vec<String> = clusters(&subject).map(str::to_owned).collect();

            assert_eq!(actual, expected, "GraphemeBreakTest.txt: {line}");
        }
    }

    #[test]
    fn keeps_a_zwj_family_emoji_in_one_cluster() {
        let family = "\u{1F469}\u{200D}\u{1F467}";

        assert_eq!(clusters(family).collect::<Vec<_>>(), vec![family]);
    }

    #[test]
    fn pairs_regional_indicators_into_one_flag_each() {
        let two_flags = "\u{1F1FA}\u{1F1F8}\u{1F1EB}\u{1F1F7}";

        assert_eq!(
            clusters(two_flags).collect::<Vec<_>>(),
            vec!["\u{1F1FA}\u{1F1F8}", "\u{1F1EB}\u{1F1F7}"]
        );
    }

    /// GB9c, added in Unicode 15.1: the virama links the two consonants into one grapheme.
    #[test]
    fn keeps_a_devanagari_conjunct_in_one_cluster() {
        let conjunct = "\u{0915}\u{094D}\u{0937}";

        assert_eq!(clusters(conjunct).collect::<Vec<_>>(), vec![conjunct]);
    }

    #[test]
    fn keeps_crlf_in_one_cluster() {
        assert_eq!(clusters("\r\n").collect::<Vec<_>>(), vec!["\r\n"]);
    }

    #[test]
    fn yields_nothing_for_an_empty_string() {
        assert_eq!(clusters("").count(), 0);
    }
    /// The ASCII fast path takes one byte and skips the tables. This is the claim that lets it.
    ///
    /// Every ordered pair of printable ASCII is segmented into exactly two clusters of one column
    /// each — 9 025 pairs, checked against the same tables the slow path uses rather than against
    /// the reasoning in the comment beside the fast path. A cached property is only as honest as
    /// the test that re-derives it.
    #[test]
    fn every_pair_of_printable_ascii_is_two_clusters_of_one_column() {
        for a in 0x20u8..0x7F {
            for b in 0x20u8..0x7F {
                let s = String::from_utf8(vec![a, b]).expect("ASCII is UTF-8");
                let got: Vec<&str> = super::clusters(&s).collect();
                assert_eq!(
                    got.len(),
                    2,
                    "{:?} segmented into {got:?}, and the fast path assumes two",
                    s
                );
                assert_eq!(super::cluster_width(got[0]), 1);
                assert_eq!(super::cluster_width(got[1]), 1);
            }
        }
    }

    /// The fast path must not fire where a combining mark follows, which is the one case that would
    /// make it wrong and the reason it looks at the byte after.
    #[test]
    fn printable_ascii_followed_by_a_combining_mark_is_one_cluster() {
        let got: Vec<&str> = super::clusters("e\u{301}x").collect();
        assert_eq!(got, vec!["e\u{301}", "x"]);
        assert_eq!(super::cluster_width(got[0]), 1);
    }

    /// The cursor is carried across cluster boundaries now, so a rule whose left-hand side spans one
    /// still has to work. Two flags are two clusters; three regional indicators are two.
    #[test]
    fn carrying_the_cursor_across_a_boundary_does_not_change_what_it_decides() {
        let two_flags = "\u{1F1FA}\u{1F1F8}\u{1F1EC}\u{1F1E7}";
        assert_eq!(super::clusters(two_flags).count(), 2);
        let three = "\u{1F1FA}\u{1F1F8}\u{1F1EC}";
        assert_eq!(super::clusters(three).count(), 2, "a pair, then a lone one");
        let family = "\u{1F468}\u{200D}\u{1F469}\u{1F600}";
        assert_eq!(
            super::clusters(family).count(),
            2,
            "the ZWJ joins the first two and the third stands alone"
        );
    }
}

mod cluster_width {
    use super::*;

    /// The correct answer is 2. kitty sums the three code points and draws 6.
    #[test]
    fn counts_a_zwj_family_emoji_as_two_columns_not_six() {
        assert_eq!(cluster_width("\u{1F469}\u{200D}\u{1F467}"), 2);
    }

    /// Only 7 of 23 surveyed terminals widen this correctly. We are one of them.
    #[test]
    fn counts_a_vs16_emoji_as_two_columns() {
        assert_eq!(cluster_width("\u{2764}\u{FE0F}"), 2);
    }

    /// The VS15 policy, and the case that pins it: U+26A1 is `W` in UAX #11, so text
    /// presentation does not make it narrow. `terminal-unicode-core` — VS15 "will **NOT** change
    /// the underlying width, but only change the display". `unicode-width` would answer 1.
    #[test]
    fn does_not_let_vs15_narrow_a_wide_base() {
        assert_eq!(cluster_width("\u{26A1}\u{FE0E}"), 2);
    }

    /// The other half of the same policy: a base that was already narrow stays narrow. U+2764 is
    /// `Emoji` but not `Emoji_Presentation`, so only a VS16 could have widened it.
    #[test]
    fn counts_a_text_presentation_emoji_as_one_column() {
        assert_eq!(cluster_width("\u{2764}\u{FE0E}"), 1);
    }

    /// Windows Terminal, cmd.exe and ConsoleZ all draw this at width 1, which breaks about a
    /// hundred languages. It is zero.
    #[test]
    fn counts_a_lone_combining_mark_as_zero_columns() {
        assert_eq!(cluster_width("\u{0301}"), 0);
    }

    #[test]
    fn counts_a_letter_with_a_combining_mark_as_one_column() {
        assert_eq!(cluster_width("e\u{0301}"), 1);
    }

    #[test]
    fn counts_a_cjk_ideograph_as_two_columns() {
        assert_eq!(cluster_width("中"), 2);
    }

    #[test]
    fn counts_a_flag_as_two_columns() {
        assert_eq!(cluster_width("\u{1F1FA}\u{1F1F8}"), 2);
    }

    /// An unpaired regional indicator is a letter in a box, not half a flag.
    #[test]
    fn counts_a_lone_regional_indicator_as_one_column() {
        assert_eq!(cluster_width("\u{1F1FA}"), 1);
    }

    /// Why the `Emoji` bit is carried and `Extended_Pictographic` would not do: `#` is `Emoji`
    /// and is not `Extended_Pictographic`, and the keycap sequence is two columns.
    #[test]
    fn counts_a_keycap_sequence_as_two_columns() {
        assert_eq!(cluster_width("#\u{FE0F}\u{20E3}"), 2);
    }

    /// The other side of that bit: a VS16 after something that is not emoji at all widens nothing.
    #[test]
    fn does_not_widen_a_non_emoji_base_under_vs16() {
        assert_eq!(cluster_width("a\u{FE0F}"), 1);
    }

    /// The policy choice: UAX #11 class `A` is narrow. UAX #11 disclaims terminal use of the
    /// property, so there is no standard answer to defer to.
    #[test]
    fn counts_an_ambiguous_width_em_dash_as_one_column() {
        assert_eq!(cluster_width("\u{2014}"), 1);
    }

    /// The base is the first code point that is not zero-width, so a `Prepend` cannot swallow it.
    #[test]
    fn counts_a_prepended_digit_as_one_column() {
        assert_eq!(cluster_width("\u{0600}1"), 1);
    }
}

mod tables_match_the_ucd {
    use super::*;

    #[test]
    fn break_class_agrees_with_grapheme_break_property() {
        let text = ucd("GraphemeBreakProperty.txt");
        let expected = Ranges::of(&text, None);

        for cp in code_points() {
            let want = match expected.get(cp) {
                Some("CR") => Break::Cr,
                Some("LF") => Break::Lf,
                Some("Control") => Break::Control,
                Some("Extend") => Break::Extend,
                Some("ZWJ") => Break::Zwj,
                Some("Regional_Indicator") => Break::RegionalIndicator,
                Some("Prepend") => Break::Prepend,
                Some("SpacingMark") => Break::SpacingMark,
                Some("L") => Break::L,
                Some("V") => Break::V,
                Some("T") => Break::T,
                Some("LV") => Break::Lv,
                Some("LVT") => Break::Lvt,
                Some(other) => panic!("unknown Grapheme_Cluster_Break value {other:?}"),
                None => Break::Other,
            };
            assert_eq!(break_class(cp), want, "U+{:04X}", cp as u32);
        }
    }

    #[test]
    fn conjunct_class_agrees_with_derived_core_properties() {
        let text = ucd("DerivedCoreProperties.txt");
        let expected = Ranges::of(&text, Some("InCB"));

        for cp in code_points() {
            let want = match expected.get(cp) {
                Some("Consonant") => Conjunct::Consonant,
                Some("Extend") => Conjunct::Extend,
                Some("Linker") => Conjunct::Linker,
                Some(other) => panic!("unknown Indic_Conjunct_Break value {other:?}"),
                None => Conjunct::None,
            };
            assert_eq!(aux_class(cp).conjunct, want, "U+{:04X}", cp as u32);
        }
    }

    #[test]
    fn extended_pictographic_agrees_with_emoji_data() {
        let text = ucd("emoji-data.txt");
        let expected = Ranges::of(&text, Some("Extended_Pictographic"));

        for cp in code_points() {
            let want = expected.get(cp).is_some();
            assert_eq!(aux_class(cp).ext_pict, want, "U+{:04X}", cp as u32);
        }
    }

    #[test]
    fn emoji_agrees_with_emoji_data() {
        let text = ucd("emoji-data.txt");
        let expected = Ranges::of(&text, Some("Emoji"));

        for cp in code_points() {
            let want = expected.get(cp).is_some();
            assert_eq!(aux_class(cp).emoji, want, "U+{:04X}", cp as u32);
        }
    }

    /// The width gate: `EastAsianWidth.txt`, its own default-wide blocks, the zero-width general
    /// categories, and emoji presentation — re-derived over sorted ranges rather than the trie.
    #[test]
    fn width_class_agrees_with_east_asian_width_and_emoji_presentation() {
        let eaw_text = ucd("EastAsianWidth.txt");
        let category_text = ucd("DerivedGeneralCategory.txt");
        let emoji_text = ucd("emoji-data.txt");
        let eaw = Ranges::of(&eaw_text, None);
        let category = Ranges::of(&category_text, None);
        let presentation = Ranges::of(&emoji_text, Some("Emoji_Presentation"));

        for cp in code_points() {
            // The file's default-wide blocks are deliberately not restated here — at 17.0.0 every
            // code point in them is listed explicitly, which
            // `the_default_wide_blocks_are_listed_explicitly` is what holds.
            let wide = matches!(eaw.get(cp), Some("W" | "F")) || presentation.get(cp).is_some();
            let zero = matches!(category.get(cp), Some("Mn" | "Me" | "Cf" | "Cc"));

            let want = match (zero, wide) {
                (true, _) => Columns::Zero,
                (false, true) => Columns::Wide,
                (false, false) => Columns::Narrow,
            };
            assert_eq!(width_class(cp), want, "U+{:04X}", cp as u32);
        }
    }

    /// `EastAsianWidth.txt`'s header says unassigned code points in five blocks default to `W`.
    /// `build.rs` implements that, and at 17.0.0 it is inert: the file lists every one of them.
    ///
    /// This is the test that says when that stops being true. While it passes, the width gate
    /// above can derive `wide` from the file's explicit rows alone and does not have to restate
    /// the block list — which is what would otherwise make the two agree by construction.
    #[test]
    fn the_default_wide_blocks_are_listed_explicitly() {
        let eaw_text = ucd("EastAsianWidth.txt");
        let eaw = Ranges::of(&eaw_text, None);

        let unlisted: Vec<u32> = [
            (0x3400, 0x4DBF),
            (0x4E00, 0x9FFF),
            (0xF900, 0xFAFF),
            (0x2_0000, 0x2_FFFD),
            (0x3_0000, 0x3_FFFD),
        ]
        .into_iter()
        .flat_map(|(first, last)| first..=last)
        .filter_map(char::from_u32)
        .filter(|cp| eaw.get(*cp).is_none())
        .map(|cp| cp as u32)
        .collect();

        assert!(
            unlisted.is_empty(),
            "{} code points in the default-wide blocks are unlisted, first U+{:04X}: the width \
             gate must restate the block list again",
            unlisted.len(),
            unlisted[0]
        );
    }
}

mod rodata {
    use super::*;

    /// The measured size of each table, so a growth lands in a diff rather than in a profile.
    ///
    /// These numbers move when the UCD version does; update them in the same commit that updates
    /// `ucd/`, and if one of them jumps, that is the finding.
    const BREAK_BYTES: usize = 5_792;
    const AUX_BYTES: usize = 8_208;
    const WIDTH_BYTES: usize = 4_956;

    /// The budget for all three properties: 20–30 KiB of static `.rodata`.
    const BUDGET: usize = 30 * 1024;

    fn break_bytes() -> usize {
        size_of_val(&tables::BREAK_ROOT)
            + size_of_val(&tables::BREAK_MID)
            + size_of_val(&tables::BREAK_LEAVES)
    }

    fn aux_bytes() -> usize {
        size_of_val(&tables::AUX_ROOT)
            + size_of_val(&tables::AUX_MID)
            + size_of_val(&tables::AUX_LEAVES)
    }

    fn width_bytes() -> usize {
        size_of_val(&tables::WIDTH_ROOT)
            + size_of_val(&tables::WIDTH_MID)
            + size_of_val(&tables::WIDTH_LEAVES)
    }

    #[test]
    fn break_table_is_the_measured_size() {
        assert_eq!(break_bytes(), BREAK_BYTES);
    }

    #[test]
    fn aux_table_is_the_measured_size() {
        assert_eq!(aux_bytes(), AUX_BYTES);
    }

    #[test]
    fn width_table_is_the_measured_size() {
        assert_eq!(width_bytes(), WIDTH_BYTES);
    }

    #[test]
    fn all_three_tables_stay_under_the_budget() {
        let total = break_bytes() + aux_bytes() + width_bytes();
        println!(
            "UCD {UCD_VERSION}: BREAK {} + AUX {} + WIDTH {} = {total} bytes of .rodata",
            break_bytes(),
            aux_bytes(),
            width_bytes()
        );

        assert!(
            total <= BUDGET,
            "UCD {UCD_VERSION} tables are {total} bytes, over the {BUDGET}-byte budget: \
             BREAK {} + AUX {} + WIDTH {}",
            break_bytes(),
            aux_bytes(),
            width_bytes()
        );
    }
}
