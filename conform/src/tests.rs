//! Tests over the **committed** captures in `fixtures/`.
//!
//! These bytes came out of a real terminal once and are evidence. A test that fails here is either a
//! parser defect or a claim in `FINDINGS.md` that stopped being true — never a reason to regenerate
//! the fixture.

use super::*;

const ATTRS: &[u8] = include_bytes!("../fixtures/tmux-3.7c-attrs-and-colours.vt");
const WIDE: &[u8] = include_bytes!("../fixtures/tmux-3.7c-wide-no-padding.vt");

// ── The refusal, which is the first thing this parser had to do ──────────────────────────────────

#[test]
fn an_empty_capture_is_refused_and_never_a_match() {
    // `screen -X hardcopy` exits 0 and writes this. Parsed permissively it compares equal against a
    // blank region, and the row goes green — a missing result hiding inside a passing one.
    assert_eq!(parse(b"", 1), Err(DumpError::Empty));
    assert_eq!(parse(b"\n\n\n\n", 1), Err(DumpError::Empty));
    assert_eq!(parse(b"   \n  \n", 1), Err(DumpError::Empty));
}

#[test]
fn a_capture_that_raced_the_paint_is_refused_rather_than_padded() {
    let err = parse(ATTRS, 99).unwrap_err();
    assert_eq!(
        err,
        DumpError::ShortScreen {
            expected: 99,
            found: 6
        }
    );
    // And it says both numbers, because "short" without them is not actionable.
    assert!(err.to_string().contains("99") && err.to_string().contains('6'));
}

#[test]
fn a_truncated_escape_is_refused() {
    assert_eq!(
        parse(b"AB\x1b[38;2;1", 1),
        Err(DumpError::UnterminatedEscape)
    );
    assert_eq!(parse(b"AB\x1b", 1), Err(DumpError::UnterminatedEscape));
}

// ── Both SGR spellings, which is what the instrument exists to ask about ─────────────────────────

#[test]
fn the_colon_and_semicolon_spellings_resolve_to_the_same_channels() {
    // The whole point of scene 03. If these two disagreed, the spelling would be a wire difference
    // rather than a compatibility one, and arch 23's answer would be wrong.
    let colon = parse(b"\x1b[38:2::255:0:0mX\n", 1).unwrap();
    let semi = parse(b"\x1b[38;2;255;0;0mX\n", 1).unwrap();
    assert_eq!(colon.rows[0].clusters[0].style.fg, Colour::Rgb(255, 0, 0));
    assert_eq!(
        colon.rows[0].clusters[0].style,
        semi.rows[0].clusters[0].style
    );
}

#[test]
fn an_indexed_colour_is_the_same_length_and_the_same_value_either_way() {
    // `38:5:200` and `38;5;200` are both eight bytes — the finding that made arch 23's byte
    // correction narrower than it was estimated to be.
    let colon = parse(b"\x1b[38:5:200mX\n", 1).unwrap();
    let semi = parse(b"\x1b[38;5;200mX\n", 1).unwrap();
    assert_eq!(colon.rows[0].clusters[0].style.fg, Colour::Indexed(200));
    assert_eq!(
        colon.rows[0].clusters[0].style,
        semi.rows[0].clusters[0].style
    );
}

#[test]
fn an_underline_colour_is_its_own_axis() {
    // SGR 58 is selected separately from 38/48, because a terminal can want the semicolon form for
    // 58 and never be asked about 38.
    let d = parse(b"\x1b[58:2::0:0:255mX\n", 1).unwrap();
    let s = d.rows[0].clusters[0].style;
    assert_eq!(s.underline, Colour::Rgb(0, 0, 255));
    assert_eq!(s.fg, Colour::Default, "58 must not touch the foreground");
}

// ── The reset asymmetry, which reads as a detail until it mis-attributes a style ─────────────────

#[test]
fn a_colour_only_run_closes_with_39_and_leaves_attributes_alone() {
    // tmux emits `ESC[39m` after a colour-only run and `ESC[0m` after an attribute-bearing one. A
    // parser treating the reset as one sequence carries the style into the next cell.
    let d = parse(b"\x1b[1m\x1b[31mA\x1b[39mB\n", 1).unwrap();
    let a = &d.rows[0].clusters[0];
    let b = &d.rows[0].clusters[1];
    assert_eq!(a.style.fg, Colour::Indexed(1));
    assert_eq!(b.style.fg, Colour::Default, "39 clears the foreground");
    assert!(b.style.attrs.has(Attrs::BOLD), "39 must not clear bold");
}

#[test]
fn zero_clears_everything() {
    let d = parse(b"\x1b[1;4;31mA\x1b[0mB\n", 1).unwrap();
    assert!(d.rows[0].clusters[1].style.attrs.is_empty());
    assert_eq!(d.rows[0].clusters[1].style.fg, Colour::Default);
}

// ── The committed captures ───────────────────────────────────────────────────────────────────────

#[test]
fn the_attrs_fixture_carries_every_attribute_it_was_sent() {
    let d = parse(ATTRS, 6).unwrap();
    assert_eq!(d.rows.len(), 6);

    assert_eq!(d.rows[0].text(), "RED-COLON");
    assert_eq!(d.rows[0].clusters[0].style.fg, Colour::Rgb(255, 0, 0));

    let ul = d.rows[1].clusters[0].style;
    assert!(ul.attrs.has(Attrs::BOLD) && ul.attrs.has(Attrs::UNDERLINE));
    assert_eq!(ul.underline, Colour::Rgb(0, 0, 255));

    assert_eq!(d.rows[2].clusters[0].style.fg, Colour::Rgb(0, 255, 0));
    assert_eq!(d.rows[5].clusters[0].style.fg, Colour::Indexed(200));

    // Reverse, italic and strikethrough share row 4, separated by unstyled spaces.
    let row = &d.rows[4];
    let of = |needle: char, attr: Attrs| {
        row.clusters
            .iter()
            .find(|c| c.text.starts_with(needle))
            .is_some_and(|c| c.style.attrs.has(attr))
    };
    assert!(of('R', Attrs::REVERSE));
    assert!(of('I', Attrs::ITALIC));
    assert!(of('S', Attrs::STRIKE));
}

#[test]
fn a_wide_glyph_arrives_with_no_padding_and_the_parser_does_not_invent_one() {
    // 28 bytes that decide how ticket 06's scene has to be built. `AB漢CD` is five clusters, and
    // there is nothing in the dump to distinguish it from five narrow ones — so this asserts the
    // *limitation*, which is the finding, rather than asserting a column.
    let d = parse(WIDE, 2).unwrap();
    assert_eq!(d.rows[0].text(), "AB漢CD");
    assert_eq!(
        d.rows[0].clusters.len(),
        5,
        "no padding cell and no continuation marker — see FINDINGS.md"
    );
    assert_eq!(d.rows[0].clusters[2].text, "漢");
    // The glyph after the wide one is at sequence index 3 and screen column 4. Nothing here knows
    // that, deliberately: recovering it needs a width table, which is what the instrument checks.
    assert_eq!(d.rows[0].clusters[3].text, "C");

    // The coloured variant: the SGR boundary lands around the whole run, not per cell.
    assert_eq!(d.rows[1].text(), "X漢Y");
    assert_eq!(d.rows[1].clusters[1].style.fg, Colour::Indexed(1));
}

#[test]
fn an_osc_header_is_skipped_rather_than_read_as_content() {
    // Ghostty's `vt` dump leads with OSC 10/11 carrying the real default fg/bg. The instrument reads
    // that header separately; the row parser must not treat it as text.
    let bytes = b"\x1b]10;rgb:ea/ea/ea\x1b\\\x1b]11;rgb:00/00/00\x1b\\ABC\n";
    let d = parse(bytes, 1).unwrap();
    assert_eq!(d.rows[0].text(), "ABC");
}
