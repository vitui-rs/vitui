//! Tests over the **committed** captures in `fixtures/`.
//!
//! These bytes came out of a real terminal once and are evidence. A test that fails here is either a
//! parser defect or a claim in `FINDINGS.md` that stopped being true — never a reason to regenerate
//! the fixture.

use super::*;

const ATTRS: &[u8] = include_bytes!("../fixtures/tmux-3.7c-attrs-and-colours.vt");
const WIDE: &[u8] = include_bytes!("../fixtures/tmux-3.7c-wide-no-padding.vt");
const TMUX_SCENE01: &[u8] = include_bytes!("../fixtures/tmux-3.7c-scene01-attrs.vt");
const VIA_TMUX: &[u8] = include_bytes!("../fixtures/ghostty-1.3.1-via-tmux-3.7c-scene01-attrs.vt");
const SCENE01: &[u8] = include_bytes!("../fixtures/ghostty-1.3.1-scene01-attrs.vt");
const KITTY_SCENE01: &[u8] = include_bytes!("../fixtures/kitty-0.48.2-scene01-attrs.vt");

// ── Scene 04, the four captures that answered architecture ticket 20 ─────────────────────────────

const SCENE04: &[u8] = include_bytes!("../fixtures/ghostty-1.3.1-scene04-pairs.vt");
const KITTY_SCENE04: &[u8] = include_bytes!("../fixtures/kitty-0.48.2-scene04-pairs.vt");
const TMUX_SCENE04: &[u8] = include_bytes!("../fixtures/tmux-3.7c-scene04-pairs.vt");
const VIA_TMUX_SCENE04: &[u8] =
    include_bytes!("../fixtures/ghostty-1.3.1-via-tmux-3.7c-scene04-pairs.vt");

// ── The refusal, which is the first thing this parser had to do ──────────────────────────────────

#[test]
fn an_empty_capture_is_refused_and_never_a_match() {
    // `screen -X hardcopy` exits 0 and writes this. Parsed permissively it compares equal against a
    // blank region, and the row goes green — a missing result hiding inside a passing one.
    assert_eq!(parse(b"", 1, Dialect::Ecma48), Err(DumpError::Empty));
    assert_eq!(
        parse(b"\n\n\n\n", 1, Dialect::Ecma48),
        Err(DumpError::Empty)
    );
    assert_eq!(
        parse(b"   \n  \n", 1, Dialect::Ecma48),
        Err(DumpError::Empty)
    );
}

#[test]
fn a_capture_that_raced_the_paint_is_refused_rather_than_padded() {
    let err = parse(ATTRS, 99, Dialect::TmuxCapturePane).unwrap_err();
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
        parse(b"AB\x1b[38;2;1", 1, Dialect::Ecma48),
        Err(DumpError::UnterminatedEscape)
    );
    assert_eq!(
        parse(b"AB\x1b", 1, Dialect::Ecma48),
        Err(DumpError::UnterminatedEscape)
    );
}

// ── Both SGR spellings, which is what the instrument exists to ask about ─────────────────────────

#[test]
fn the_colon_and_semicolon_spellings_resolve_to_the_same_channels() {
    // The whole point of scene 03. If these two disagreed, the spelling would be a wire difference
    // rather than a compatibility one, and arch 23's answer would be wrong.
    let colon = parse(b"\x1b[38:2::255:0:0mX\n", 1, Dialect::Ecma48).unwrap();
    let semi = parse(b"\x1b[38;2;255;0;0mX\n", 1, Dialect::Ecma48).unwrap();
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
    let colon = parse(b"\x1b[38:5:200mX\n", 1, Dialect::Ecma48).unwrap();
    let semi = parse(b"\x1b[38;5;200mX\n", 1, Dialect::Ecma48).unwrap();
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
    let d = parse(b"\x1b[58:2::0:0:255mX\n", 1, Dialect::Ecma48).unwrap();
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
    let d = parse(b"\x1b[1m\x1b[31mA\x1b[39mB\n", 1, Dialect::Ecma48).unwrap();
    let a = &d.rows[0].clusters[0];
    let b = &d.rows[0].clusters[1];
    assert_eq!(a.style.fg, Colour::Indexed(1));
    assert_eq!(b.style.fg, Colour::Default, "39 clears the foreground");
    assert!(b.style.attrs.has(Attrs::BOLD), "39 must not clear bold");
}

#[test]
fn zero_clears_everything() {
    let d = parse(b"\x1b[1;4;31mA\x1b[0mB\n", 1, Dialect::Ecma48).unwrap();
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
    let d = parse(b"\x1b[4:2mX\n", 1, Dialect::Ecma48).unwrap();
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
        let d = parse(&input, 1, Dialect::Ecma48).unwrap();
        assert_eq!(d.rows[0].clusters[0].style.underline, want);
    }
    // An undefined style stays visible as one rather than collapsing into single.
    let d = parse(b"\x1b[4:9mX\n", 1, Dialect::Ecma48).unwrap();
    assert_eq!(d.rows[0].clusters[0].style.underline, Underline::Other(9));
}

#[test]
fn overline_is_one_of_the_eleven_and_has_its_own_off() {
    let d = parse(b"\x1b[53mA\x1b[55mB\n", 1, Dialect::Ecma48).unwrap();
    assert!(d.rows[0].clusters[0].style.attrs.has(Attrs::OVERLINE));
    assert!(!d.rows[0].clusters[1].style.attrs.has(Attrs::OVERLINE));
}

#[test]
fn twenty_four_clears_the_underline_and_leaves_its_colour_alone() {
    // Two axes, two resets. SGR 24 is the style; SGR 59 is the colour.
    let d = parse(
        b"\x1b[4:3;58;2;0;0;255mA\x1b[24mB\x1b[59mC\n",
        1,
        Dialect::Ecma48,
    )
    .unwrap();
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
    let d = parse(b"\x1b[38:2::255:0:0;1mX\n", 1, Dialect::Ecma48).unwrap();
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
    let d = parse(ATTRS, 6, Dialect::TmuxCapturePane).unwrap();
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
    let d = parse(WIDE, 2, Dialect::TmuxCapturePane).unwrap();
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
    let d = parse(bytes, 1, Dialect::Ecma48).unwrap();
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
    let d = parse(SCENE01, 11, Dialect::Ecma48).unwrap();

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

#[test]
fn the_ghostty_capture_still_has_the_carriage_returns_the_terminal_sent() {
    // **The fixture is bytes, and git does not know that.** With `core.autocrlf = input` — the
    // setting on the machine this was captured on — `git add` rewrote CRLF to LF on the way into the
    // index, and 1949 bytes of what a terminal really sent became a 1926-byte blob. Every test above
    // still passed, because the parser skips CR the way a terminal does, so the fixture stopped
    // being evidence with nothing to show for it.
    //
    // `.gitattributes` marks `*.vt` as `-text`. This is the assertion that says so out loud, because
    // an attributes file nobody checks is the same kind of thing as an MSRV nobody compiles.
    let crlf = SCENE01.windows(2).filter(|w| w == b"\r\n").count();
    assert_eq!(
        crlf, 23,
        "the Ghostty capture is CRLF-separated and this one is not — check `.gitattributes`, \
         and do not regenerate the fixture to make this pass"
    );
}

// ── The dialect, which is the second defect this parser had in itself ────────────────────────────

#[test]
fn a_colon_in_a_tmux_capture_is_arithmetic_and_in_an_ecma48_one_is_a_sub_parameter() {
    // **Both readings are pinned, so the dialect is asserted rather than assumed.** `grid.c` writes
    // any attribute code of two digits as `code/10 : code%10`, so tmux spells overline `5:3` — and
    // ECMA-48 spells blink-with-a-sub-parameter exactly the same way. There is no reading of these
    // five bytes that is right for both formats, which is why the caller has to say which it has.
    let tmux = parse(b"\x1b[5:3mX\n", 1, Dialect::TmuxCapturePane).unwrap();
    let s = tmux.rows[0].clusters[0].style;
    assert!(s.attrs.has(Attrs::OVERLINE), "53 is overline");
    assert!(
        !s.attrs.has(Attrs::BLINK),
        "and reading it as blink is the instrument inventing an attribute tmux never rendered"
    );

    let ecma = parse(b"\x1b[5:3mX\n", 1, Dialect::Ecma48).unwrap();
    let s = ecma.rows[0].clusters[0].style;
    assert!(s.attrs.has(Attrs::BLINK), "parameter 5 is blink");
    assert!(!s.attrs.has(Attrs::OVERLINE));
}

#[test]
fn the_underline_styles_read_the_same_in_both_dialects_and_that_is_by_tmux_s_design() {
    // tmux numbers its underline styles 42–45 **so that** dividing by ten lands on ECMA-48's
    // `4:2`–`4:5`. The two readings agreeing here is the reason a parser told nothing about the
    // dialect passed every test stage 0 and stage 1 had: the only code where the coincidence breaks
    // is 53, and nothing had captured an overline through tmux yet.
    for (bytes, want) in [
        (&b"\x1b[4:2m"[..], Underline::Double),
        (&b"\x1b[4:3m"[..], Underline::Curly),
        (&b"\x1b[4:4m"[..], Underline::Dotted),
        (&b"\x1b[4:5m"[..], Underline::Dashed),
    ] {
        let mut input = bytes.to_vec();
        input.extend_from_slice(b"X\n");
        for dialect in [Dialect::Ecma48, Dialect::TmuxCapturePane] {
            let d = parse(&input, 1, dialect).unwrap();
            assert_eq!(
                d.rows[0].clusters[0].style.underline, want,
                "{dialect:?} on {bytes:?}"
            );
        }
    }
}

#[test]
fn the_tmux_dialect_does_not_mangle_a_colon_colour() {
    // The fold must not reach 38, 48 or 58: those carry a real T.416 tail, and `3:8` folding to 38
    // would eat the parameter that names the colour space. tmux writes all three with semicolons
    // anyway, so this asserts that nothing was broken for a spelling tmux does not emit — which is
    // the spelling a *future* tmux is most likely to start emitting.
    let d = parse(b"\x1b[38:2::255:0:0;1mX\n", 1, Dialect::TmuxCapturePane).unwrap();
    let s = d.rows[0].clusters[0].style;
    assert_eq!(s.fg, Colour::Rgb(255, 0, 0));
    assert!(s.attrs.has(Attrs::BOLD), "and the 1 after it is still bold");

    let d = parse(b"\x1b[58:2::0:0:255mX\n", 1, Dialect::TmuxCapturePane).unwrap();
    assert_eq!(
        d.rows[0].clusters[0].style.underline_colour,
        Colour::Rgb(0, 0, 255)
    );
}

// ── Scene 01 on the second family, which is what production ticket 05 asked for ──────────────────

/// The eleven rows of scene 01 and the one attribute each is supposed to be wearing.
///
/// Shared by the three scene-01 fixtures below rather than written out three times: the point of
/// those three is that they are the **same scene** through three paths, and a table per fixture
/// would let one drift and turn a real disagreement into an unattributable one.
const SCENE01_ROWS: [(&str, Attrs, Underline); 11] = [
    ("bold", Attrs::BOLD, Underline::None),
    ("dim", Attrs::DIM, Underline::None),
    ("italic", Attrs::ITALIC, Underline::None),
    ("reverse", Attrs::REVERSE, Underline::None),
    ("blink", Attrs::BLINK, Underline::None),
    ("strikethru", Attrs::STRIKE, Underline::None),
    ("conceal", Attrs::HIDDEN, Underline::None),
    ("overline", Attrs::OVERLINE, Underline::None),
    ("under-sgl", Attrs::NONE, Underline::Single),
    ("under-dbl", Attrs::NONE, Underline::Double),
    ("under-dot", Attrs::NONE, Underline::Dotted),
];

/// What one row of a scene-01 capture actually carried: the label, and the style every cluster of it
/// wore. `None` where the row's clusters do not all agree, which is a disagreement of its own.
fn row_style(dump: &Dump, i: usize, label: &str) -> Option<Style> {
    let row = &dump.rows[i];
    assert_eq!(row.text().trim_end(), label, "row {i}");
    let mut styles = row
        .clusters
        .iter()
        .take(label.chars().count())
        .map(|c| c.style);
    let first = styles.next()?;
    styles.all(|s| s == first).then_some(first)
}

#[test]
fn tmux_holds_all_eleven_attribute_bits_in_its_own_grid() {
    // **The second family production ticket 05 required.** `capture-pane -e` re-serialises tmux's
    // own cells, so this says what tmux *stored* — and it stored all eleven, overline included. The
    // fixture beside it says what tmux *forwards*, and those two are not the same number.
    let d = parse(TMUX_SCENE01, 11, Dialect::TmuxCapturePane).unwrap();
    for (i, (label, attrs, underline)) in SCENE01_ROWS.iter().enumerate() {
        let want = Style {
            attrs: *attrs,
            underline: *underline,
            ..Style::default()
        };
        assert_eq!(row_style(&d, i, label), Some(want), "row {i} — {label}");
    }
}

#[test]
fn tmux_does_not_forward_overline_and_forwards_the_other_ten() {
    // **The quirk, as a gate.** The engine drew scene 01 inside tmux inside Ghostty, and this is what
    // Ghostty's own dump said. Ghostty alone agrees 11/11 and tmux's grid holds all eleven, so the
    // one bit missing here was dropped by tmux on the way out — `attrs_dropped`, observed rather than
    // inferred, and the first row of `quirks.rs` this repository gathered itself.
    //
    // tmux 3.0 added overline output gated on the `Smol` capability, and `xterm-ghostty`'s terminfo
    // defines `Smulx` and not `Smol` — which is why the underline styles below survive and this one
    // does not. `set -as terminal-features ",xterm-ghostty:overline"` restores it; see `FINDINGS.md`.
    let d = parse(VIA_TMUX, 11, Dialect::Ecma48).unwrap();
    for (i, (label, attrs, underline)) in SCENE01_ROWS.iter().enumerate() {
        let dropped = *attrs == Attrs::OVERLINE;
        let want = Style {
            attrs: if dropped { Attrs::NONE } else { *attrs },
            underline: *underline,
            ..Style::default()
        };
        assert_eq!(
            row_style(&d, i, label),
            Some(want),
            "row {i} — {label}{}",
            match dropped {
                true => ", which tmux is expected to have dropped",
                false => "",
            }
        );
    }
}

#[test]
fn the_three_scene01_captures_disagree_in_exactly_one_place() {
    // The three paths as one assertion, because the *comparison* is the finding and three separate
    // tests would let it be read as three unrelated results. Ghostty direct and tmux's own grid agree
    // with the scene on all eleven; tmux's forwarding agrees on ten.
    let direct = parse(SCENE01, 11, Dialect::Ecma48).unwrap();
    let grid = parse(TMUX_SCENE01, 11, Dialect::TmuxCapturePane).unwrap();
    let forwarded = parse(VIA_TMUX, 11, Dialect::Ecma48).unwrap();

    let disagreements = |a: &Dump, b: &Dump| {
        SCENE01_ROWS
            .iter()
            .enumerate()
            .filter(|(i, (label, ..))| row_style(a, *i, label) != row_style(b, *i, label))
            .map(|(_, (label, ..))| *label)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        disagreements(&direct, &grid),
        Vec::<&str>::new(),
        "Ghostty's cells and tmux's cells hold the same eleven bits"
    );
    assert_eq!(
        disagreements(&direct, &forwarded),
        vec!["overline"],
        "and the only thing tmux loses on the way out is overline"
    );
}

#[test]
fn a_bare_two_digit_code_is_still_what_ecma48_says_it_is_in_both_dialects() {
    // **42–45 sit inside ECMA-48's 40–47 background colours.** The arm that reads tmux's folded
    // underline styles is guarded on the fold for exactly this reason; without the guard a green
    // background would come back as a double underline, in *both* dialects, and the fixtures have no
    // background colour on a row that would have caught it.
    for dialect in [Dialect::Ecma48, Dialect::TmuxCapturePane] {
        let d = parse(b"\x1b[42mX\n", 1, dialect).unwrap();
        let s = d.rows[0].clusters[0].style;
        assert_eq!(
            s.bg,
            Colour::Indexed(2),
            "{dialect:?}: bare 42 is a green background"
        );
        assert_eq!(
            s.underline,
            Underline::None,
            "{dialect:?}: and it is not an underline"
        );
    }
    // Where the colon *is* there, only the tmux dialect reads it as 42.
    let d = parse(b"\x1b[4:2mX\n", 1, Dialect::TmuxCapturePane).unwrap();
    let s = d.rows[0].clusters[0].style;
    assert_eq!(s.underline, Underline::Double);
    assert_eq!(s.bg, Colour::Default, "and it is not a background colour");
}

// ── The third emulator family, and it is the first that disagreed twice ──────────────────────────

#[test]
fn kitty_has_nowhere_to_put_conceal_or_overline_and_holds_the_other_nine() {
    // **The third family**, and the second `quirks.rs` row this repository gathered itself. kitty
    // 0.48.2 renders nine of the eleven, drops two, and cannot be *asked* about the eleventh — see
    // the row below this one, which is a different sentence and has to stay a different sentence.
    //
    // The two it drops are not a dump's inference. kitty's `Cursor` repr — in the shipped
    // `kitty.fast_data_types.so` — enumerates every formatting attribute the cursor carries: `bold
    // italic reverse strikethrough dim decoration decoration_fg text_blink`, and the attribute
    // constants beside it say `BOLD ITALIC REVERSE MARK STRIKETHROUGH DECORATION BLINK`. SGR is a
    // mutation of the cursor, so an attribute the cursor cannot carry is one no cell can hold and no
    // paint can consult.
    let d = parse(KITTY_SCENE01, 11, Dialect::Ecma48).unwrap();
    for (i, (label, attrs, underline)) in SCENE01_ROWS.iter().enumerate() {
        let dropped = *attrs == Attrs::HIDDEN || *attrs == Attrs::OVERLINE;
        // What the capture says, which for `under-dot` is not what kitty is holding.
        let want = Style {
            attrs: if dropped { Attrs::NONE } else { *attrs },
            underline: match *underline {
                Underline::Dotted => Underline::Single,
                other => other,
            },
            ..Style::default()
        };
        assert_eq!(
            row_style(&d, i, label),
            Some(want),
            "row {i} — {label}{}",
            match dropped {
                true => ", which kitty has no attribute to store",
                false => "",
            }
        );
    }
}

#[test]
fn a_dotted_underline_reaches_this_capture_as_an_empty_sub_parameter() {
    // **The bytes that make one row of the scene unaskable**, pinned here rather than only asserted
    // about in prose. kitty's serialiser has a string for `4:2` and `4:3` and none for `4:4` or
    // `4:5`, so a cell holding a dotted underline is written out as `CSI 4 : m`.
    //
    // ECMA-48 reads an omitted parameter as the default and SGR 4's default is 1, so the reading
    // above is *single* — the correct reading of what arrived and the wrong description of what
    // kitty is holding. Compared anyway it would say *kitty does not render dotted underlines*,
    // which is false, and would earn a `quirks.rs` entry the table exists to keep out.
    let contains = |needle: &[u8]| KITTY_SCENE01.windows(needle.len()).any(|w| w == needle);
    assert!(
        contains(b"\x1b[4:m"),
        "the capture spells a dotted underline `CSI 4 : m`"
    );
    assert!(
        !contains(b"\x1b[4:4m"),
        "and it never spells one `CSI 4:4 m`, which is what the arm's declaration rests on"
    );
    // And the separation is not an assumption: `4:0` comes back as nothing at all, so a `4:` proves
    // the cell holds a non-zero decoration and only its number was lost.
    for dialect in [Dialect::Ecma48, Dialect::TmuxCapturePane] {
        let d = parse(b"\x1b[4:mX\n", 1, dialect).unwrap();
        assert_eq!(
            d.rows[0].clusters[0].style.underline,
            Underline::Single,
            "{dialect:?}: an empty sub-parameter is the default, which for SGR 4 is 1"
        );
    }
}

#[test]
fn kitty_pads_every_row_to_the_full_width_where_tmux_trims_to_the_label() {
    // Two capture formats, opposite habits, and the check that rides on it is the same one: an
    // attribute must **stop where its label does**. tmux trims the default-styled blanks the engine
    // painted, so there is nothing to the right to inspect; kitty keeps all eighty columns, so a
    // reverse block running to the right edge would be visible here as eighty styled cells.
    //
    // Neither trims a *styled* blank, which is why the check survives both.
    let kitty = parse(KITTY_SCENE01, 11, Dialect::Ecma48).unwrap();
    let tmux = parse(TMUX_SCENE01, 11, Dialect::TmuxCapturePane).unwrap();
    assert_eq!(kitty.rows[0].clusters.len(), 80, "kitty pads to the width");
    assert_eq!(tmux.rows[0].clusters.len(), 4, "tmux trims to `bold`");
    assert!(
        kitty.rows[0]
            .clusters
            .iter()
            .skip(4)
            .all(|c| c.style == Style::default()),
        "and every padding cell kitty kept is unstyled, so nothing leaked past the label"
    );
}

#[test]
fn the_three_emulators_disagree_about_exactly_which_bits_they_have() {
    // The point of a third family, as one assertion. Ghostty holds all eleven; kitty holds nine and
    // has nowhere to put the other two; tmux's *forwarding* loses one, and it is not either of
    // kitty's — so the three results are three different facts about three different parties and not
    // one flaky number.
    //
    // `under-dot` is excluded on the kitty side and only there, because that row is not a
    // disagreement at all: it is a question this capture format cannot ask. Excluding it here rather
    // than everywhere is what keeps the exclusion honest — the same row is compared for the other
    // two arms and does agree.
    let ghostty = parse(SCENE01, 11, Dialect::Ecma48).unwrap();
    let kitty = parse(KITTY_SCENE01, 11, Dialect::Ecma48).unwrap();
    let forwarded = parse(VIA_TMUX, 11, Dialect::Ecma48).unwrap();

    let differs = |other: &Dump, skip: &[&str]| {
        SCENE01_ROWS
            .iter()
            .enumerate()
            .filter(|(_, (label, ..))| !skip.contains(label))
            .filter(|(i, (label, ..))| {
                row_style(&ghostty, *i, label) != row_style(other, *i, label)
            })
            .map(|(_, (label, ..))| *label)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        differs(&kitty, &["under-dot"]),
        vec!["conceal", "overline"],
        "kitty has no cursor attribute for either"
    );
    assert_eq!(
        differs(&forwarded, &[]),
        vec!["overline"],
        "and tmux forwards everything else it was sent"
    );
}

// ── Scene 04 — a pair bisected, and what the terminal does with the orphan ───────────────────────

/// The four rows every family agrees on, as text, in the order the scene draws them.
///
/// Written out rather than taken from `examples/common.rs`: these tests are the **gate** and that
/// file is the *instrument*, so a comparator that read its expectations from the thing it is
/// checking would be the arrangement this whole directory exists to break. It is the same rule as
/// the parser having no `cell_at(column)`.
const SCENE04_ROWS: [(&str, &str); 6] = [
    ("control", "AB漢CD"),
    ("over-cont", "AB xCD"),
    ("over-head", "ABx CD"),
    ("over-wide", "AB 漢D"),
    ("keeps-style", "AB xCD"),
    ("ruler", "0123456789"),
];

/// One capture's six rows as trimmed text.
fn scene04_text(bytes: &[u8], dialect: Dialect) -> Vec<String> {
    parse(bytes, SCENE04_ROWS.len(), dialect)
        .expect("a committed scene 04 capture parses")
        .rows
        .iter()
        .take(SCENE04_ROWS.len())
        .map(|r| r.text().trim_end().to_string())
        .collect()
}

#[test]
fn every_family_blanks_the_orphaned_half_and_they_do_not_disagree_about_it() {
    // **Architecture ticket 20, as an assertion over bytes four terminals produced.**
    //
    // The question was which of two spec sentences yields at a `View::child` clip: §3's *a wide head
    // is always followed by a `CONTINUATION`*, or §4's *a child cannot widen its clip*. Both could
    // be argued from the document, which is how the ticket came to be filed rather than decided.
    //
    // These bytes decide it. Row `over-cont` prints `AB漢CD` and then one narrow cluster over the
    // continuation at column 3; every family comes back `AB xCD`, with the head at column 2 blanked.
    // Row `over-head` is the mirror image and comes back `ABx CD`. **No terminal here has any notion
    // of a clip to consult**, so a surface holding a wide head with no continuation is a surface none
    // of them can be made to show — the engine's mirror would believe a cell the screen does not
    // have, damage tracking would never repaint it, and the artifact would stand until something
    // else wrote there. That is the corruption §3 names, and it is why §3 keeps its sentence.
    for (bytes, dialect, who) in [
        (SCENE04, Dialect::Ecma48, "Ghostty 1.3.1"),
        (KITTY_SCENE04, Dialect::Ecma48, "kitty 0.48.2"),
        (TMUX_SCENE04, Dialect::TmuxCapturePane, "tmux 3.7c"),
        (VIA_TMUX_SCENE04, Dialect::Ecma48, "tmux 3.7c forwarded"),
    ] {
        let rows = scene04_text(bytes, dialect);
        for (i, (label, want)) in SCENE04_ROWS.iter().enumerate() {
            assert_eq!(&rows[i], want, "{who}, row {label}");
        }
    }
}

#[test]
fn the_families_disagree_about_what_the_blanked_half_wears_and_that_is_the_sharper_finding() {
    // The four rows above are unanimous. This one is not, and it is the row that turns architecture
    // ticket 20's answer from *the engine may as well repair* into *the engine must*.
    //
    // The wide glyph carries a red background and the cluster written over its continuation does
    // not. **kitty keeps the orphan's own background; Ghostty and tmux blank it to the SGR state in
    // force.** So a repair delegated to the terminal is not merely a repair the mirror would not
    // know about — it is a repair whose *result differs by terminal*, and there is no single mirror
    // state that could be right on all three. The engine has to do it itself and serialise the
    // outcome, which is exactly what the answer makes it do.
    //
    // Asserted here rather than in the live arm's table, because a per-terminal fact belongs to a
    // capture: the arm reports what it saw and the fixture is what holds it still.
    let at = |bytes, dialect| {
        parse(bytes, SCENE04_ROWS.len(), dialect).unwrap().rows[4].clusters[2].style
    };
    assert_eq!(
        at(KITTY_SCENE04, Dialect::Ecma48).bg,
        Colour::Indexed(1),
        "kitty 0.48.2 keeps the background of the half it blanked"
    );
    for (bytes, dialect, who) in [
        (SCENE04, Dialect::Ecma48, "Ghostty 1.3.1"),
        (TMUX_SCENE04, Dialect::TmuxCapturePane, "tmux 3.7c"),
        (VIA_TMUX_SCENE04, Dialect::Ecma48, "tmux 3.7c forwarded"),
    ] {
        assert_eq!(
            at(bytes, dialect),
            Style::default(),
            "{who} blanks to the SGR state in force, not to the orphan's own style"
        );
    }
}

#[test]
fn the_scene_04_captures_are_short_screens_when_a_row_is_missing() {
    // The refusal, asserted against the scene that is most able to hide behind it: scene 04's rows
    // are six short strings, and a capture that lost the top of the screen would still parse into
    // *something*. A capture with fewer rows than the scene declared never reaches a comparison.
    //
    // This is not hypothetical here. The first `--through-tmux` capture of this scene had the whole
    // picture on it **twice**, at rows 38 and 76, because the probe repainted with `CSI 2 J` and tmux
    // pushes a cleared screen into the pane's history. Six rows of `FAILED` against a screen that
    // had the right answer on it — the instrument's defect, found by the arm that has two parsers in
    // the path and therefore the most ways to go wrong. `EL` per row is what it does now.
    assert_eq!(
        parse(SCENE04, 99, Dialect::Ecma48).unwrap_err(),
        DumpError::ShortScreen {
            expected: 99,
            found: 6
        },
        "six rows, because the parser drops the empty ones below the payload after counting them"
    );
}

// ── Scene 05, the terminal's own answer to `CSI 6n` ──────────────────────────────────────────────
//
// **These bytes are not a screen and the refusals are not the dump's.** A capture that raced the
// paint is a *short* screen; a cursor report that never came is *no reply at all*, and an
// instrument that read a missing reply as a width would report a number no terminal ever said. So
// the first tests written here are the four ways [`cursor_reports`] refuses, in the same order the
// dump parser's were.
//
// The hand-written byte strings below are legitimate where a hand-written *screen* would not be: a
// cursor report's grammar is ECMA-48's and not any terminal's, and what is being asserted is that
// an absent, short or unterminated one is refused. What a terminal actually answers is the
// fixtures' job, below.

#[test]
fn a_terminal_that_never_answered_is_refused_and_never_a_width() {
    // The scene 05 shape of `screen -X hardcopy`'s zero-byte file: read permissively, a run with no
    // replies has no rows to disagree, and an empty survey prints as a clean one.
    assert_eq!(
        cursor_reports(b"", 3),
        Err(CprError::NoSentinel),
        "no sentinel and no replies is a terminal that answered nothing at all"
    );
    assert_eq!(
        cursor_reports(b"\x1b[?62;c", 3),
        Err(CprError::Count {
            expected: 3,
            found: 0
        }),
        "the sentinel arrived, so the terminal was alive and answered no cursor report"
    );
}

#[test]
fn a_short_batch_is_refused_rather_than_padded() {
    let bytes = b"\x1b[1;2R\x1b[1;3R\x1b[?62;c";
    assert_eq!(
        cursor_reports(bytes, 5),
        Err(CprError::Count {
            expected: 5,
            found: 2
        })
    );
    let err = cursor_reports(bytes, 5).unwrap_err();
    assert!(
        err.to_string().contains('5') && err.to_string().contains('2'),
        "both numbers, because \"short\" without them is not actionable"
    );
}

#[test]
fn a_reply_that_arrives_after_the_sentinel_is_not_counted() {
    // The sentinel is what says the terminal has finished with the batch. A reply after it is a
    // reply to something else — a stray keystroke's answer, or the next run's — and counting it
    // would let a batch that lost a reply be made up to length by a stranger's.
    assert_eq!(
        cursor_reports(b"\x1b[1;2R\x1b[?62;c\x1b[1;9R", 2),
        Err(CprError::Count {
            expected: 2,
            found: 1
        })
    );
}

#[test]
fn every_reply_must_be_on_the_row_the_scene_wrote() {
    // The scene homes the cursor to the same row before each cluster, so two replies on two rows
    // means the screen scrolled or a cluster wrapped — and a column measured on a row the scene did
    // not write is not a measurement of anything.
    assert_eq!(
        cursor_reports(b"\x1b[1;2R\x1b[4;3R\x1b[?62;c", 2),
        Err(CprError::RowMoved { first: 1, then: 4 })
    );
}

#[test]
fn a_truncated_reply_is_refused() {
    assert_eq!(
        cursor_reports(b"\x1b[1;2R\x1b[1;", 2),
        Err(CprError::UnterminatedReply)
    );
}

#[test]
fn a_batch_of_none_is_a_batch_and_not_a_panic() {
    // A caller asking for no clusters is a caller with nothing to measure, and a sentinel with
    // nothing behind it is what such a run answers with. It reaches the row check with no rows,
    // which is a legitimate empty batch rather than the one accident this reader exists to refuse.
    assert_eq!(cursor_reports(b"\x1b[?62;c", 0), Ok(Vec::new()));
}

#[test]
fn the_column_is_taken_as_the_terminal_reported_it() {
    // One column and no arithmetic: the subtraction that turns a reported column into an advance is
    // the caller's, and it is one line in the report rather than a table in the instrument.
    let got = cursor_reports(b"\x1b[1;2R\x1b[1;3R\x1b[1;1R\x1b[?62;1;6c", 3).unwrap();
    assert_eq!(
        got,
        vec![
            Reply { row: 1, column: 2 },
            Reply { row: 1, column: 3 },
            Reply { row: 1, column: 1 },
        ]
    );
}

// ── Scene 05, over the captures four terminals actually sent ─────────────────────────────────────

const GHOSTTY_WIDTHS: &[u8] = include_bytes!("../fixtures/ghostty-1.3.1-scene05-widths.cpr");
const KITTY_WIDTHS: &[u8] = include_bytes!("../fixtures/kitty-0.48.2-scene05-widths.cpr");
const TMUX_WIDTHS: &[u8] = include_bytes!("../fixtures/tmux-3.7c-scene05-widths.cpr");
const VIA_TMUX_WIDTHS: &[u8] =
    include_bytes!("../fixtures/ghostty-1.3.1-via-tmux-3.7c-scene05-widths.cpr");

/// Scene 05's corpus, in order, with the advance every arm reported for it on 2026-08-29.
///
/// **Hand-written here and not read out of `width_of`.** This half of the crate has never heard of
/// the engine, which is the arrangement that makes it a gate rather than a mirror — so what these
/// numbers are is *what four terminals said*, and the engine agreeing with all fifteen is a
/// separate sentence in the reports.
const OBSERVED: &[(&str, u16)] = &[
    ("ascii", 1),
    ("ascii-pair", 2),
    ("cjk", 2),
    ("hangul", 2),
    ("fullwidth", 2),
    ("ambiguous", 1),
    ("combining", 1),
    ("zero-width", 0),
    ("emoji", 2),
    ("vs16", 2),
    ("vs15", 1),
    ("zwj-family", 2),
    ("flag", 2),
    ("skin-tone", 2),
    ("keycap", 2),
];

#[test]
fn four_terminals_answered_the_same_fifteen_widths() {
    for (bytes, who) in [
        (GHOSTTY_WIDTHS, "Ghostty 1.3.1"),
        (KITTY_WIDTHS, "kitty 0.48.2"),
        (TMUX_WIDTHS, "tmux 3.7c"),
        (VIA_TMUX_WIDTHS, "tmux 3.7c under Ghostty"),
    ] {
        let replies = cursor_reports(bytes, OBSERVED.len())
            .unwrap_or_else(|e| panic!("{who}'s capture is not a batch: {e}"));
        for (reply, (label, advance)) in replies.iter().zip(OBSERVED) {
            assert_eq!(
                reply.column - 1,
                *advance,
                "{who} moved the cursor differently for {label}"
            );
        }
    }
}

#[test]
fn a_cursor_report_never_leaves_the_innermost_terminal() {
    // **The evidence for the sentence the through-tmux arm's scene-05 heading makes.** That arm's
    // photograph is the only instrument here that can see what tmux *forwards* to a terminal
    // downstream of it; its cursor reports never get that far, because tmux answers `CSI 6n` from
    // its own grid on the pane's pty. Byte-identical, device attributes included — where the two
    // arms' *screen* captures are two different serialisations of two different grids.
    assert_eq!(
        TMUX_WIDTHS, VIA_TMUX_WIDTHS,
        "the same terminal answered both, so the same bytes came back"
    );
}

#[test]
fn the_three_families_are_three_terminals_and_the_sentinel_says_so() {
    // The four captures above carry the same fifteen answers, so nothing in them distinguishes the
    // terminals — which is exactly the shape of a fixture accidentally copied from another arm. The
    // device-attributes reply is what separates them, and it is in the same bytes for free.
    let da1 = |bytes: &[u8]| {
        let text = String::from_utf8_lossy(bytes).to_string();
        let at = text
            .rfind("\x1b[?")
            .expect("every arm answered the sentinel");
        text[at..].to_string()
    };
    assert_eq!(da1(GHOSTTY_WIDTHS), "\x1b[?62;22;52c", "Ghostty 1.3.1");
    assert_eq!(
        da1(KITTY_WIDTHS),
        "\x1b[?62;52;c",
        "kitty 0.48.2, whose reply carries an empty parameter"
    );
    assert_eq!(
        da1(TMUX_WIDTHS),
        "\x1b[?1;2;4c",
        "tmux 3.7c answers as a VT100 with AVO"
    );
    assert_ne!(
        da1(GHOSTTY_WIDTHS),
        da1(TMUX_WIDTHS),
        "two arms whose sentinels agreed would be one arm's capture under two names"
    );
}

#[test]
fn the_two_channels_are_not_interchangeable() {
    // A `.vt` is a screen and a `.cpr` is a terminal's own answers. Handing either to the other's
    // reader is refused rather than answered wrongly, which is why the fixtures are named apart.
    //
    // The **named** error and not `is_err`, because the two refusals have to be refusals for the
    // right reason: a screen carries no sentinel, and a batch of replies carries no printable
    // content at all. An `is_err` here would go on passing if either reader started refusing
    // everything.
    assert_eq!(
        cursor_reports(SCENE01, OBSERVED.len()),
        Err(CprError::NoSentinel),
        "a screen has no device-attributes reply in it"
    );
    assert_eq!(
        parse(TMUX_WIDTHS, OBSERVED.len(), Dialect::Ecma48),
        Err(DumpError::Empty),
        "a batch of cursor reports has no cells in it"
    );
}

// ── Scene 06, part A: what the terminal says about a mode ────────────────────────────────────────

/// The five questions scene 06 asks in one batch, and the states DECRPM defines for the answers.
///
/// Hand-written here as they are hand-written in the scene, because a test that took its
/// expectations from the scene would be checking one copy of the arithmetic against another.
const SCENE06_STATES: [ModeState; 5] = [
    ModeState::Reset,
    ModeState::Set,
    ModeState::Reset,
    ModeState::Set,
    ModeState::Reset,
];

/// A well-formed batch: five answers about 2026 and a device-attributes reply behind them.
const SCENE06_BATCH: &[u8] =
    b"\x1b[?2026;2$y\x1b[?2026;1$y\x1b[?2026;2$y\x1b[?2026;1$y\x1b[?2026;2$y\x1b[?1;2;4c";

#[test]
fn a_mode_batch_with_no_sentinel_is_refused_and_never_a_state() {
    // The same accident as the dump's zero bytes and the width batch's missing reply: an answer
    // that never arrived, read as one. Here it would not even leave a blank — the batch is counted
    // positionally, so a short one reports every later row under the wrong question.
    let no_sentinel = &SCENE06_BATCH[..SCENE06_BATCH.len() - "\x1b[?1;2;4c".len()];
    assert_eq!(
        mode_reports(no_sentinel, 2026, 5),
        Err(ModeError::NoSentinel)
    );
}

#[test]
fn a_short_batch_of_mode_reports_is_refused_rather_than_read_positionally() {
    let mut short = SCENE06_BATCH.to_vec();
    // Drop the first answer and keep the sentinel, which is what a terminal that missed one looks
    // like from here.
    let cut = short
        .windows(11)
        .position(|w| w == b"\x1b[?2026;2$y")
        .expect("the batch opens with a reset");
    short.drain(cut..cut + 11);
    assert_eq!(
        mode_reports(&short, 2026, 5),
        Err(ModeError::Count {
            expected: 5,
            found: 4
        })
    );
}

#[test]
fn a_reply_about_another_mode_is_refused_rather_than_skipped() {
    // Skipping it would be worse than counting it: the count would then come out right for a batch
    // whose answers are one question out of step.
    let stranger = b"\x1b[?2026;2$y\x1b[?7;1$y\x1b[?2026;1$y\x1b[?1;2;4c";
    assert_eq!(
        mode_reports(stranger, 2026, 2),
        Err(ModeError::OtherMode {
            wanted: 2026,
            then: 7
        })
    );
}

#[test]
fn a_state_decrpm_does_not_define_is_refused_rather_than_guessed() {
    let odd = b"\x1b[?2026;9$y\x1b[?1;2;4c";
    assert_eq!(
        mode_reports(odd, 2026, 1),
        Err(ModeError::UnknownState { ps: 9 })
    );
    assert_eq!(ModeState::from_ps(9), None);
}

#[test]
fn a_private_mode_reply_that_is_not_two_numbers_is_refused_rather_than_dropped() {
    // Dropped, it arrives downstream as a short batch — *the terminal lost an answer* about a
    // terminal that answered every question and spelled one of them in a way this reader does not
    // know. Two causes, and only one of them is about the terminal.
    let odd = b"\x1b[?2026;$y\x1b[?1;2;4c";
    assert_eq!(
        mode_reports(odd, 2026, 1),
        Err(ModeError::Malformed {
            body: "?2026;$".to_string()
        })
    );
}

#[test]
fn a_truncated_mode_reply_is_refused() {
    assert_eq!(
        mode_reports(b"\x1b[?2026;1", 2026, 1),
        Err(ModeError::UnterminatedReply)
    );
}

#[test]
fn a_mode_reply_after_the_sentinel_is_not_counted() {
    // Everything past the device-attributes reply belongs to some other question, and a batch that
    // lost an answer could otherwise be made up to length by a stranger's.
    let mut trailing = SCENE06_BATCH.to_vec();
    trailing.extend_from_slice(b"\x1b[?2026;1$y");
    assert_eq!(
        mode_reports(&trailing, 2026, 5).map(|r| r.len()),
        Ok(5),
        "the batch is what arrived before the sentinel"
    );
    assert_eq!(
        mode_reports(&trailing, 2026, 6),
        Err(ModeError::Count {
            expected: 6,
            found: 5
        })
    );
}

#[test]
fn an_ansi_mode_reply_is_not_a_private_one() {
    // `CSI 4 ; 2 $ y` is DECRPM for the *ANSI* mode 4 and has the same shape as the private reply
    // with the `?` taken off. Reading one as the other would report insert-replace mode's state
    // under mode 2026's heading, and the count would come out right.
    let ansi = b"\x1b[4;2$y\x1b[?1;2;4c";
    assert_eq!(
        mode_reports(ansi, 2026, 0),
        Ok(vec![]),
        "an ANSI-mode reply is not an answer to a private-mode question"
    );
}

#[test]
fn the_five_states_come_back_in_the_order_the_scene_asked_them() {
    let seen = mode_reports(SCENE06_BATCH, 2026, 5).expect("a well-formed batch");
    assert_eq!(
        seen.iter().map(|r| r.state).collect::<Vec<_>>(),
        SCENE06_STATES.to_vec()
    );
    assert!(
        seen.iter().all(|r| r.mode == 2026),
        "every reply carries the mode it is about, and it is the one that was asked"
    );
}

// ── Scene 06, part B: the bracket, which is the flag and not the paint ───────────────────────────

/// A probe whose reply landed `LATENCY` after the question, which is what a local pty costs.
fn probe(at_ms: u32, state: ModeState) -> Probe {
    Probe {
        at_ms,
        answered_at_ms: at_ms + LATENCY,
        state: Some(state),
    }
}

/// The gap between the question and its answer in these fixtures.
///
/// Not zero, deliberately: with the two clocks equal every test here would pass under a fold that
/// read one of them for both ends, which is the defect the two fields exist to prevent.
const LATENCY: u32 = 40;

#[test]
fn no_probes_is_not_a_bracket() {
    assert_eq!(flush_bracket(&[]), Err(BracketError::NoProbes));
}

#[test]
fn a_hole_in_the_sequence_is_refused_rather_than_bracketed_around() {
    // A probe that got no reply is not evidence that the mode was still set, and it is not evidence
    // that it was reset. Bracketing around it would put the event on whichever side the two
    // surviving probes happen to fall.
    let run = [
        probe(50, ModeState::Set),
        Probe {
            at_ms: 900,
            answered_at_ms: 940,
            state: None,
        },
        probe(3000, ModeState::Reset),
    ];
    assert_eq!(
        flush_bracket(&run),
        Err(BracketError::NoAnswer { at_ms: 900 })
    );
}

#[test]
fn a_probe_that_is_not_a_timing_is_refused_with_what_it_said() {
    // A terminal without the mode, and one that pins it, are each a real answer about the terminal.
    // Neither is a number, and a row that printed one would be inventing it.
    for state in [
        ModeState::NotRecognised,
        ModeState::PermanentlySet,
        ModeState::PermanentlyReset,
    ] {
        assert_eq!(
            flush_bracket(&[probe(50, state)]),
            Err(BracketError::NotUsable { at_ms: 50, state })
        );
    }
}

#[test]
fn a_timeout_that_unexpired_itself_is_refused() {
    // Monotonicity is what makes a bisection sound rather than a search over an arbitrary function,
    // and a run that breaks it has measured something that is not a timeout.
    let run = [probe(400, ModeState::Reset), probe(900, ModeState::Set)];
    assert_eq!(
        flush_bracket(&run),
        Err(BracketError::NotMonotone {
            set_at: 900,
            reset_at: 400 + LATENCY
        })
    );
}

#[test]
fn the_bracket_is_the_pair_the_event_is_between() {
    let run = [
        probe(50, ModeState::Set),
        probe(3000, ModeState::Reset),
        probe(904, ModeState::Set),
        probe(1002, ModeState::Reset),
    ];
    assert_eq!(
        flush_bracket(&run),
        Ok(Bracket::Between {
            still_set_at: 904,
            reset_by: 1002 + LATENCY
        }),
        "the tightest straddling pair, and each end read off the clock that makes it sound"
    );
}

#[test]
fn a_mode_that_never_reset_inside_the_ceiling_says_so() {
    let run = [probe(50, ModeState::Set), probe(3000, ModeState::Set)];
    assert_eq!(
        flush_bracket(&run),
        Ok(Bracket::NeverReset { ceiling: 3000 }),
        "the ceiling is the request, because nothing was observed to have reset by anything"
    );
}

#[test]
fn a_mode_already_reset_at_the_floor_says_so() {
    // Either the limit is under the floor or the open never took, and those are different facts.
    // The row prints the floor so a reader can tell which question to ask next.
    let run = [probe(50, ModeState::Reset), probe(3000, ModeState::Reset)];
    assert_eq!(
        flush_bracket(&run),
        Ok(Bracket::AlreadyReset {
            floor: 50 + LATENCY
        })
    );
}

#[test]
fn the_two_ends_are_probed_before_anything_is_halved() {
    // A bisection between two ends that were never established is a bisection over a guess.
    assert_eq!(next_delay(&[], 50, 3000, 128), Some(50));
    assert_eq!(
        next_delay(&[probe(50, ModeState::Set)], 50, 3000, 128),
        Some(3000)
    );
}

#[test]
fn a_run_with_nothing_to_bisect_stops_rather_than_halving_a_guess() {
    let never = [probe(50, ModeState::Set), probe(3000, ModeState::Set)];
    assert_eq!(next_delay(&never, 50, 3000, 128), None);
    let already = [probe(50, ModeState::Reset), probe(3000, ModeState::Reset)];
    assert_eq!(next_delay(&already, 50, 3000, 128), None);
    let absent = [
        probe(50, ModeState::NotRecognised),
        probe(3000, ModeState::NotRecognised),
    ];
    assert_eq!(next_delay(&absent, 50, 3000, 128), None);
}

#[test]
fn the_halving_converges_on_the_boundary_and_stops_at_the_resolution() {
    // A terminal whose flag resets at exactly 1000 ms, played against the real strategy. What is
    // asserted is the *property* — the bracket straddles the boundary and is no wider than the
    // resolution — rather than the ladder, so a better bisection does not fail this test.
    const BOUNDARY: u32 = 1000;
    let mut probes: Vec<Probe> = Vec::new();
    let mut opens = 0;
    while let Some(at_ms) = next_delay(
        &probes,
        FLUSH_FLOOR_MS,
        FLUSH_CEILING_MS,
        FLUSH_RESOLUTION_MS,
    ) {
        opens += 1;
        assert!(opens < 32, "the strategy did not converge");
        probes.push(probe(
            at_ms,
            if at_ms < BOUNDARY {
                ModeState::Set
            } else {
                ModeState::Reset
            },
        ));
    }
    let Ok(Bracket::Between {
        still_set_at,
        reset_by,
    }) = flush_bracket(&probes)
    else {
        panic!("a terminal with a boundary brackets it: {probes:?}");
    };
    assert!(
        still_set_at < BOUNDARY && reset_by >= BOUNDARY,
        "the bracket must straddle the boundary: ({still_set_at}, {reset_by}]"
    );
    assert!(
        reset_by - still_set_at <= FLUSH_RESOLUTION_MS + LATENCY,
        "the bracket must be no wider than the resolution it was asked for, plus the round trip \
         the two clocks are honest about"
    );
    assert_eq!(
        opens, 7,
        "the readiness timeout is derived from this count, so the two move together"
    );
}

#[test]
fn the_probes_may_arrive_in_any_order() {
    // The scene appends them in the order it took them and the strategy chooses that order, so
    // nothing guarantees it is sorted. A fold that depended on the order would be right for every
    // run that happened to be sorted.
    let forward = [
        probe(50, ModeState::Set),
        probe(904, ModeState::Set),
        probe(1002, ModeState::Reset),
        probe(3000, ModeState::Reset),
    ];
    let mut backward = forward;
    backward.reverse();
    assert_eq!(flush_bracket(&forward), flush_bracket(&backward));
    assert_eq!(
        next_delay(&forward, 50, 3000, 32),
        next_delay(&backward, 50, 3000, 32)
    );
}

#[test]
fn each_end_of_the_bracket_is_read_off_the_clock_that_makes_it_sound() {
    // The terminal processed each question somewhere between the write and the reply, so a *set*
    // answer is only evidence back to the request and a *reset* answer only forward to the reply.
    // Read the other way round, this run would claim the mode was still set at 940 and already
    // reset at 900 — a bracket that runs backwards over an event nothing observed.
    let run = [
        Probe {
            at_ms: 900,
            answered_at_ms: 940,
            state: Some(ModeState::Set),
        },
        Probe {
            at_ms: 1000,
            answered_at_ms: 1400,
            state: Some(ModeState::Reset),
        },
    ];
    assert_eq!(
        flush_bracket(&run),
        Ok(Bracket::Between {
            still_set_at: 900,
            reset_by: 1400
        }),
        "a slow reply widens the bracket; it may never be allowed to narrow it"
    );
}

// ── Scene 06, the four captures of what each terminal says about mode 2026 ───────────────────────

const GHOSTTY_SYNC: &[u8] = include_bytes!("../fixtures/ghostty-1.3.1-scene06-sync.decrqm");
const KITTY_SYNC: &[u8] = include_bytes!("../fixtures/kitty-0.48.2-scene06-sync.decrqm");
const TMUX_SYNC: &[u8] = include_bytes!("../fixtures/tmux-3.7c-scene06-sync.decrqm");
const VIA_TMUX_SYNC: &[u8] =
    include_bytes!("../fixtures/ghostty-1.3.1-via-tmux-3.7c-scene06-sync.decrqm");

const SYNC_ARMS: [(&str, &[u8]); 4] = [
    ("Ghostty 1.3.1", GHOSTTY_SYNC),
    ("kitty 0.48.2", KITTY_SYNC),
    ("tmux 3.7c", TMUX_SYNC),
    ("Ghostty 1.3.1 via tmux 3.7c", VIA_TMUX_SYNC),
];

#[test]
fn all_three_families_have_mode_2026_and_their_state_machines_track_it() {
    // The three families of spec §10's tier 1, answering about themselves. `serial.rs` wraps every
    // frame in this mode where the terminal has it, and until this scene the evidence that any of
    // them does was `detect.rs` believing a reply it also wrote the parser for.
    for (who, bytes) in SYNC_ARMS {
        let seen = mode_reports(bytes, 2026, SCENE06_STATES.len())
            .unwrap_or_else(|e| panic!("{who}: {e}"));
        assert_eq!(
            seen.iter().map(|r| r.state).collect::<Vec<_>>(),
            SCENE06_STATES.to_vec(),
            "{who} disagrees with DECRPM's own definitions"
        );
    }
}

#[test]
fn one_reset_undoes_two_sets_because_a_dec_mode_is_not_a_counter() {
    // The last two rows of the batch, read on their own because this is the one row of the five
    // whose answer the engine depends on: §8 wraps a frame in a **balanced** pair, and a terminal
    // that counted would hold a frame past the close that was sent for it.
    for (who, bytes) in SYNC_ARMS {
        let seen = mode_reports(bytes, 2026, 5).unwrap_or_else(|e| panic!("{who}: {e}"));
        assert_eq!(seen[3].state, ModeState::Set, "{who}: two `h` is set");
        assert_eq!(
            seen[4].state,
            ModeState::Reset,
            "{who}: one `l` after two `h` is reset, not set-with-one-left"
        );
    }
}

#[test]
fn the_reply_never_leaves_the_innermost_terminal_and_the_two_tmux_fixtures_are_one_capture() {
    // Scene 05's property, and it holds for scene 06 for the same reason: the question is asked in
    // band on the scene's own tty, so the terminal that answers is the innermost one — which for
    // the through-tmux arm is tmux and not the Ghostty it is photographed through. Asserted rather
    // than described, because a column headed *Ghostty* over tmux's answers is the dishonesty
    // `SCENES.md` opens by naming.
    assert_eq!(
        TMUX_SYNC, VIA_TMUX_SYNC,
        "the two tmux paths answered one question, so their captures are one capture"
    );
    // And the other two are not, or one arm's capture would be sitting under two names. The device
    // attributes are what separate them: three families, three answers.
    assert_ne!(GHOSTTY_SYNC, KITTY_SYNC);
    assert_ne!(GHOSTTY_SYNC, TMUX_SYNC);
    assert_ne!(KITTY_SYNC, TMUX_SYNC);
}

#[test]
fn a_sync_capture_is_not_a_width_capture_and_neither_is_a_screen() {
    // Three channels now, and each reader refuses the other two's bytes for its own reason. The
    // **named** errors, because a refusal for the wrong reason goes on passing after a reader
    // starts refusing everything.
    assert_eq!(
        cursor_reports(TMUX_SYNC, OBSERVED.len()),
        Err(CprError::Count {
            expected: OBSERVED.len(),
            found: 0
        }),
        "a batch of mode reports has a sentinel and no cursor reports in it"
    );
    assert_eq!(
        mode_reports(TMUX_WIDTHS, 2026, 5),
        Err(ModeError::Count {
            expected: 5,
            found: 0
        }),
        "a batch of cursor reports has a sentinel and no mode reports in it"
    );
    assert_eq!(
        mode_reports(SCENE01, 2026, 5),
        Err(ModeError::NoSentinel),
        "a screen has no device-attributes reply in it"
    );
    assert_eq!(
        parse(TMUX_SYNC, 5, Dialect::Ecma48),
        Err(DumpError::Empty),
        "a batch of mode reports has no cells in it"
    );
}
