//! Tests over the **committed** captures in `fixtures/`.
//!
//! These bytes came out of a real terminal once and are evidence. A test that fails here is either a
//! parser defect or a claim in `FINDINGS.md` that stopped being true — never a reason to regenerate
//! the fixture.

use super::*;

const ATTRS: &[u8] = include_bytes!("../fixtures/tmux-3.7c-attrs-and-colours.vt");
const WIDE: &[u8] = include_bytes!("../fixtures/tmux-3.7c-wide-no-padding.vt");
const SCENE01: &[u8] = include_bytes!("../fixtures/ghostty-1.3.1-scene01-attrs.vt");

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
    assert_eq!(s.underline_colour, Colour::Rgb(0, 0, 255));
    assert_eq!(s.fg, Colour::Default, "58 must not touch the foreground");
    assert_eq!(
        s.underline,
        Underline::None,
        "an underline colour is not an underline"
    );
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
    assert_eq!(d.rows[0].clusters[1].style.underline, Underline::None);
    assert_eq!(d.rows[0].clusters[1].style.fg, Colour::Default);
}

// ── A parameter is not a token, which a Ghostty control probe had to teach this parser ────────

#[test]
fn an_underline_style_is_one_parameter_and_not_two() {
    // The defect the live arm found before it drew a single frame. Splitting on `;` and `:` alike
    // turns `4:2` into `4` then `2` — double underline read as underline **and dim**, an attribute
    // the terminal never rendered and the instrument invented.
    let d = parse(b"\x1b[4:2mX\n", 1).unwrap();
    let s = d.rows[0].clusters[0].style;
    assert_eq!(s.underline, Underline::Double);
    assert!(
        !s.attrs.has(Attrs::DIM),
        "`4:2` is one parameter; its sub-parameter is not SGR 2"
    );
    assert!(s.attrs.is_empty(), "and it is not any other flag either");
}

#[test]
fn every_underline_style_the_engine_can_spell_reads_back() {
    // The engine's three-bit underline field, and it always emits the `4:n` form — never bare `4`,
    // and never SGR 21. Ghostty answers bare `4` for single, so both spellings have to arrive here.
    for (bytes, want) in [
        (&b"\x1b[4m"[..], Underline::Single),
        (&b"\x1b[4:1m"[..], Underline::Single),
        (&b"\x1b[4:2m"[..], Underline::Double),
        (&b"\x1b[4:3m"[..], Underline::Curly),
        (&b"\x1b[4:4m"[..], Underline::Dotted),
        (&b"\x1b[4:5m"[..], Underline::Dashed),
    ] {
        let mut input = bytes.to_vec();
        input.extend_from_slice(b"X\n");
        let d = parse(&input, 1).unwrap();
        assert_eq!(d.rows[0].clusters[0].style.underline, want);
    }
    // An undefined style stays visible as one rather than collapsing into single.
    let d = parse(b"\x1b[4:9mX\n", 1).unwrap();
    assert_eq!(d.rows[0].clusters[0].style.underline, Underline::Other(9));
}

#[test]
fn overline_is_one_of_the_eleven_and_has_its_own_off() {
    let d = parse(b"\x1b[53mA\x1b[55mB\n", 1).unwrap();
    assert!(d.rows[0].clusters[0].style.attrs.has(Attrs::OVERLINE));
    assert!(!d.rows[0].clusters[1].style.attrs.has(Attrs::OVERLINE));
}

#[test]
fn twenty_four_clears_the_underline_and_leaves_its_colour_alone() {
    // Two axes, two resets. SGR 24 is the style; SGR 59 is the colour.
    let d = parse(b"\x1b[4:3;58;2;0;0;255mA\x1b[24mB\x1b[59mC\n", 1).unwrap();
    let (a, b, c) = (
        d.rows[0].clusters[0].style,
        d.rows[0].clusters[1].style,
        d.rows[0].clusters[2].style,
    );
    assert_eq!(a.underline, Underline::Curly);
    assert_eq!(a.underline_colour, Colour::Rgb(0, 0, 255));
    assert_eq!(b.underline, Underline::None);
    assert_eq!(b.underline_colour, Colour::Rgb(0, 0, 255), "24 is not 59");
    assert_eq!(c.underline_colour, Colour::Default);
}

#[test]
fn a_colon_colour_does_not_swallow_the_parameter_after_it() {
    // The colon form's tail lives inside its own parameter, so the `1` that follows is bold and not
    // a channel. Flattening got this right by accident and only because the tail was full-length.
    let d = parse(b"\x1b[38:2::255:0:0;1mX\n", 1).unwrap();
    let s = d.rows[0].clusters[0].style;
    assert_eq!(s.fg, Colour::Rgb(255, 0, 0));
    assert!(s.attrs.has(Attrs::BOLD));
}

#[test]
fn the_osc_header_gives_up_the_emulator_s_own_default_colours() {
    // Ghostty 1.3.1 leads with this pair, and it is the only party that knows what `Colour::Default`
    // resolves to. A capture without one says so rather than guessing.
    let bytes = b"\x1b]10;rgb:ea/ea/ea\x1b\\\x1b]11;rgb:00/00/00\x1b\\ABC\n";
    assert_eq!(
        default_colours(bytes),
        (
            Some(Colour::Rgb(0xea, 0xea, 0xea)),
            Some(Colour::Rgb(0, 0, 0))
        )
    );
    assert_eq!(default_colours(b"ABC\n"), (None, None));
}

// ── The committed captures ───────────────────────────────────────────────────────────────────────

#[test]
fn the_attrs_fixture_carries_every_attribute_it_was_sent() {
    let d = parse(ATTRS, 6).unwrap();
    assert_eq!(d.rows.len(), 6);

    assert_eq!(d.rows[0].text(), "RED-COLON");
    assert_eq!(d.rows[0].clusters[0].style.fg, Colour::Rgb(255, 0, 0));

    let ul = d.rows[1].clusters[0].style;
    assert!(ul.attrs.has(Attrs::BOLD));
    assert_eq!(ul.underline, Underline::Single);
    assert_eq!(ul.underline_colour, Colour::Rgb(0, 0, 255));

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

// ── Scene 01, as Ghostty 1.3.1 gave it back ──────────────────────────────────────────────────────

#[test]
fn ghostty_carried_all_eleven_attribute_bits_back() {
    // The live arm's own capture, committed. `cargo run --example ghostty` produced it and agreed
    // 11/11; this is that run turned into a gate, so the comparison logic behind the agreement is
    // checked on every commit with no Ghostty, no window server and no automation grant in the loop.
    //
    // **A failure here is a parser defect or a claim that stopped being true — never a reason to
    // regenerate the fixture.** Re-running the live arm is `CONFORM_SAVE_CAPTURE=…`, and it is a
    // deliberate act.
    let d = parse(SCENE01, 11).unwrap();

    let want: [(&str, Attrs, Underline); 11] = [
        ("bold", Attrs::BOLD, Underline::None),
        ("dim", Attrs::DIM, Underline::None),
        ("italic", Attrs::ITALIC, Underline::None),
        ("reverse", Attrs::REVERSE, Underline::None),
        ("blink", Attrs::BLINK, Underline::None),
        ("strikethru", Attrs::STRIKE, Underline::None),
        ("conceal", Attrs::HIDDEN, Underline::None),
        ("overline", Attrs::OVERLINE, Underline::None),
        ("under-sgl", Attrs::default(), Underline::Single),
        ("under-dbl", Attrs::default(), Underline::Double),
        ("under-dot", Attrs::default(), Underline::Dotted),
    ];

    for (i, (label, attrs, underline)) in want.iter().enumerate() {
        let row = &d.rows[i];
        // The engine paints the whole surface, so the row is the label followed by real spaces —
        // unlike the never-written cells a `printf` leaves, which Ghostty trims.
        assert_eq!(row.text().trim_end(), *label, "row {i}");
        let style = Style {
            attrs: *attrs,
            underline: *underline,
            ..Style::default()
        };
        for c in row.clusters.iter().take(label.chars().count()) {
            assert_eq!(c.style, style, "row {i} cluster {:?}", c.text);
        }
        // And the attribute stopped where the label did. A reverse block running to the right edge
        // is a real defect that comparing the first cell alone would call a pass.
        for c in row.clusters.iter().skip(label.chars().count()) {
            assert_eq!(c.style, Style::default(), "row {i} padding is styled");
        }
    }
}

#[test]
fn ghostty_reports_its_own_default_colours_and_they_are_not_the_xterm_guess() {
    // The instrument cannot check a `Colour::Default` cell against anything without this, and the
    // emulator is the only party that knows. This machine's Ghostty is not on white-on-black or the
    // xterm defaults, which is exactly why the header has to be read rather than assumed.
    assert_eq!(
        default_colours(SCENE01),
        (
            Some(Colour::Rgb(0xea, 0xea, 0xea)),
            Some(Colour::Rgb(0x00, 0x00, 0x00))
        )
    );
}
