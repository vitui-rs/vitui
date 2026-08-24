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
