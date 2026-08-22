//! The parser, the queue and the two things neither of them may ever do.
//!
//! Register entries #13, #14 and #16 are **not** here: they are spec §14's, and §14's rule is that
//! a register entry runs where the register can point at it, which is `crate::gates`. What is here
//! is everything else — the wire spellings, the boundaries, and the cases that produced a defect
//! while this file was being written.

use std::time::Instant;

use crate::input::parse::Parser;
use crate::input::{
    Button, Buttons, Event, Key, KeyCode, KeyKind, KeyText, Keypad, Media, Modifier, Mods, Mouse,
    MouseKind, Paste, Queue, Wheel,
};

/// One read's worth of bytes, parsed whole.
pub(crate) fn parse(bytes: &[u8]) -> Vec<Event> {
    parse_with(1 << 20, &[bytes])
}

/// Several reads, each ended as the input thread ends one.
pub(crate) fn parse_with(paste_limit: usize, reads: &[&[u8]]) -> Vec<Event> {
    let mut parser = Parser::new(paste_limit);
    let mut out = Vec::new();
    for read in reads {
        let at = Instant::now();
        let mut sink = |event: Event| out.push(event);
        parser.feed(read, at, &mut sink);
        parser.end_of_read(at, &mut sink);
    }
    out
}

/// The same events with every timestamp replaced by `at`.
///
/// Two runs of the same bytes are never equal otherwise, because `at` is the real clock — which is
/// the point of the field. A gate that compares two parses compares everything **except** the one
/// value it deliberately cannot control.
pub(crate) fn stamped(events: Vec<Event>, at: Instant) -> Vec<Event> {
    events
        .into_iter()
        .map(|event| match event {
            Event::Key(key) => Event::Key(Key { at, ..key }),
            Event::Mouse(mouse) => Event::Mouse(Mouse { at, ..mouse }),
            Event::Paste(paste) => {
                Event::Paste(Paste::new(paste.bytes().to_vec(), paste.truncated(), at))
            }
            other => other,
        })
        .collect()
}

/// The one key in `bytes`, or a panic naming what came out instead.
fn one_key(bytes: &[u8]) -> Key {
    let events = parse(bytes);
    match events.as_slice() {
        [Event::Key(key)] => *key,
        other => panic!("expected one key from {bytes:?}, got {other:?}"),
    }
}

fn one_mouse(bytes: &[u8]) -> Mouse {
    let events = parse(bytes);
    match events.as_slice() {
        [Event::Mouse(mouse)] => *mouse,
        other => panic!("expected one mouse event from {bytes:?}, got {other:?}"),
    }
}

#[test]
fn printable_ascii_is_its_own_code_and_its_own_text() {
    let key = one_key(b"a");
    assert_eq!(key.code, KeyCode::Char('a'));
    assert_eq!(key.text.as_str(), "a");
    assert_eq!(key.mods, Mods::NONE);
    assert_eq!(key.kind, KeyKind::Press);
}

/// **A capital on the wire says nothing about which key produced it.** Inferring `SHIFT` would be
/// the smallest possible piece of inventing, and it is wrong on a layout where the capital is on an
/// unshifted key, on a dead-key sequence, and with caps lock on.
#[test]
fn a_capital_below_the_kitty_protocol_carries_no_shift() {
    let key = one_key(b"A");
    assert_eq!(key.code, KeyCode::Char('A'));
    assert_eq!(key.text.as_str(), "A");
    assert_eq!(key.mods, Mods::NONE);
}

/// And with the protocol on, the two fields are what say it: the base key in `code`, the capital in
/// `text`, and the modifier the terminal actually reported.
#[test]
fn the_kitty_protocol_separates_the_base_key_from_what_it_printed() {
    let key = one_key(b"\x1b[97;2;65u");
    assert_eq!(key.code, KeyCode::Char('a'));
    assert_eq!(key.mods, Mods::SHIFT);
    assert_eq!(key.text.as_str(), "A");
}

/// **`code` is the base layout when the terminal sent one**, which is the whole reason `code` and
/// `text` are two fields. A Cyrillic `ф` reports `1092` as the key and `97` as its base layout, and
/// an application binding `Ctrl+A` needs the second while a text field needs the first.
#[test]
fn the_base_layout_key_is_what_code_reports_when_the_terminal_sends_one() {
    let key = one_key(b"\x1b[1092::97;5u");
    assert_eq!(
        key.code,
        KeyCode::Char('a'),
        "the base-layout key, for the binding"
    );
    assert_eq!(key.mods, Mods::CTRL);

    // With no flag-16 text and no chord, the text comes from the **primary** field rather than from
    // `code` — reporting an `a` here would say the key printed a letter it did not.
    let typed = one_key(b"\x1b[1092::97u");
    assert_eq!(typed.code, KeyCode::Char('a'));
    assert_eq!(typed.text.as_str(), "\u{444}");
}

/// An omitted sub-parameter is `None` and not a zero, **by position**. `1092::97` is *primary, no
/// shifted key, base-layout 97*; filtering the empty one out and then indexing reads the base layout
/// as the shifted key, and `;1:3` reads a release as an `Alt` chord.
#[test]
fn an_empty_sub_parameter_holds_its_place() {
    let released = one_key(b"\x1b[97;:3u");
    assert_eq!(
        released.mods,
        Mods::NONE,
        "an empty modifier field is no modifiers"
    );
    assert_eq!(released.kind, KeyKind::Release);
}

/// Associated text of more than one codepoint is one cluster, which is what `KeyText` exists for: a
/// dead-key accent commits as a base plus a combining mark, from one keystroke.
#[test]
fn associated_text_of_several_codepoints_is_one_cluster() {
    let key = one_key(b"\x1b[101;1;101:769u");
    assert_eq!(key.code, KeyCode::Char('e'));
    assert_eq!(key.text.as_str(), "e\u{301}");
}

#[test]
fn a_control_chord_is_the_letter_it_is_typed_with() {
    let key = one_key(b"\x03");
    assert_eq!(key.code, KeyCode::Char('c'));
    assert_eq!(key.mods, Mods::CTRL);
    assert!(key.text.is_empty(), "a chord printed nothing");
}

/// The three the kitty spec carves out on purpose, so that `reset` stays typeable after a program
/// crashes with the mode set. They are **permanently** ambiguous below the protocol and this asserts
/// the ambiguity rather than papering over it.
#[test]
fn enter_tab_and_backspace_are_the_named_keys_and_never_a_control_chord() {
    assert_eq!(one_key(b"\r").code, KeyCode::Enter);
    assert_eq!(one_key(b"\t").code, KeyCode::Tab);
    assert_eq!(one_key(b"\x7f").code, KeyCode::Backspace);
    assert_eq!(
        one_key(b"\r").mods,
        Mods::NONE,
        "Ctrl+M is not reported as a chord"
    );
    assert_eq!(
        one_key(b"\t").mods,
        Mods::NONE,
        "Ctrl+I is not reported as a chord"
    );
}

/// `Ctrl+X` is `X & 0x1f` for every `X` in `@A-Z[\]^_`, so `0x1c` is `Ctrl+\` and not `Ctrl+4`.
/// The digit reading is a US-layout coincidence and `code` is meant to be the base key.
#[test]
fn the_four_control_codes_above_the_letters_are_punctuation_and_not_digits() {
    for (byte, expected) in [(0x1cu8, '\\'), (0x1d, ']'), (0x1e, '^'), (0x1f, '_')] {
        let key = one_key(&[byte]);
        assert_eq!(key.code, KeyCode::Char(expected), "byte {byte:#04x}");
        assert_eq!(key.mods, Mods::CTRL);
    }
}

/// A modifier field wider than a byte is clamped rather than cast. `as u8` on 257 answers
/// `Mods::NONE`, which is a terminal saying *eight modifiers* and this crate hearing *none*.
#[test]
fn a_modifier_field_wider_than_a_byte_is_clamped_rather_than_wrapped() {
    let key = one_key(b"\x1b[97;258u");
    assert!(
        !key.mods.is_empty(),
        "257 modifier bits is not no modifiers"
    );
}

#[test]
fn a_meta_prefix_is_alt_and_the_key_after_it_is_unchanged() {
    let key = one_key(b"\x1ba");
    assert_eq!(key.code, KeyCode::Char('a'));
    assert_eq!(key.mods, Mods::ALT);
    assert_eq!(key.text.as_str(), "a");
}

#[test]
fn a_bare_escape_is_the_escape_key_at_the_end_of_the_read() {
    assert_eq!(one_key(b"\x1b").code, KeyCode::Escape);
}

/// The read boundary is what resolves it, and this is the shape that proves the boundary is doing
/// the work: the same bytes in one read are Alt+a, in two reads they are Escape then `a`.
#[test]
fn an_escape_held_across_a_read_boundary_becomes_escape_and_then_the_key() {
    let split = parse_with(1 << 20, &[b"\x1b", b"a"]);
    let codes: Vec<_> = split
        .iter()
        .map(|e| match e {
            Event::Key(k) => (k.code, k.mods),
            other => panic!("unexpected {other:?}"),
        })
        .collect();
    assert_eq!(
        codes,
        vec![
            (KeyCode::Escape, Mods::NONE),
            (KeyCode::Char('a'), Mods::NONE)
        ]
    );
}

#[test]
fn the_legacy_arrows_arrive_from_both_introducers() {
    assert_eq!(one_key(b"\x1b[A").code, KeyCode::Up);
    assert_eq!(one_key(b"\x1bOA").code, KeyCode::Up);
    assert_eq!(one_key(b"\x1b[D").code, KeyCode::Left);
    assert_eq!(one_key(b"\x1bOP").code, KeyCode::F(1));
}

#[test]
fn a_modified_arrow_carries_the_modifier_and_nothing_else() {
    let key = one_key(b"\x1b[1;5A");
    assert_eq!(key.code, KeyCode::Up);
    assert_eq!(key.mods, Mods::CTRL);
    assert_eq!(key.kind, KeyKind::Press);
}

#[test]
fn the_tilde_block_covers_the_navigation_keys_and_f5_upward() {
    assert_eq!(one_key(b"\x1b[2~").code, KeyCode::Insert);
    assert_eq!(one_key(b"\x1b[3~").code, KeyCode::Delete);
    assert_eq!(one_key(b"\x1b[5~").code, KeyCode::PageUp);
    assert_eq!(one_key(b"\x1b[15~").code, KeyCode::F(5));
    assert_eq!(one_key(b"\x1b[24~").code, KeyCode::F(12));
}

#[test]
fn shift_tab_is_its_own_spelling_on_a_legacy_terminal() {
    let key = one_key(b"\x1b[Z");
    assert_eq!(key.code, KeyCode::BackTab);
    assert_eq!(key.mods, Mods::SHIFT);
}

#[test]
fn focus_is_two_variants_and_no_payload() {
    assert_eq!(parse(b"\x1b[I"), vec![Event::FocusGained]);
    assert_eq!(parse(b"\x1b[O"), vec![Event::FocusLost]);
}

/// Kitty's private-use block, at both ends and in the three families it splits into.
#[test]
fn the_kitty_functional_block_reaches_the_keypad_the_media_keys_and_the_modifiers() {
    assert_eq!(
        one_key(b"\x1b[57399u").code,
        KeyCode::Keypad(Keypad::Digit(0))
    );
    assert_eq!(one_key(b"\x1b[57427u").code, KeyCode::Keypad(Keypad::Begin));
    assert_eq!(
        one_key(b"\x1b[57430u").code,
        KeyCode::Media(Media::PlayPause)
    );
    assert_eq!(
        one_key(b"\x1b[57441u").code,
        KeyCode::Modifier(Modifier::LeftShift)
    );
    assert_eq!(
        one_key(b"\x1b[57454u").code,
        KeyCode::Modifier(Modifier::IsoLevel5Shift)
    );
    assert_eq!(one_key(b"\x1b[57376u").code, KeyCode::F(13));
    assert_eq!(one_key(b"\x1b[57398u").code, KeyCode::F(35));
    assert_eq!(
        parse(b"\x1b[57364u"),
        vec![],
        "the gap between MENU and F13 is unassigned, and an unassigned private-use codepoint is \
         not a `Char` in somebody's text field"
    );
}

/// **All eight, including the two locks**, and the locks are state rather than chord.
#[test]
fn every_modifier_bit_survives_the_wire_and_the_locks_leave_the_chord() {
    // 1 + shift(1) + ctrl(4) + caps(64) = 70.
    let key = one_key(b"\x1b[97;70u");
    assert!(key.mods.shift() && key.mods.ctrl() && key.mods.caps());
    assert!(!key.mods.num());
    assert_eq!(key.mods.chord(), Mods::SHIFT.with(Mods::CTRL));
}

#[test]
fn an_sgr_press_is_zero_based_and_remembers_what_is_held() {
    let mouse = one_mouse(b"\x1b[<0;10;5M");
    assert_eq!((mouse.x, mouse.y), (9, 4));
    assert_eq!(mouse.kind, MouseKind::Down(Button::Left));
    assert!(mouse.buttons.left());
}

#[test]
fn a_release_clears_the_button_it_names() {
    let events = parse(b"\x1b[<0;10;5M\x1b[<0;10;5m");
    match events.as_slice() {
        [Event::Mouse(down), Event::Mouse(up)] => {
            assert!(down.buttons.left());
            assert_eq!(up.kind, MouseKind::Up(Button::Left));
            assert!(up.buttons.is_empty());
        }
        other => panic!("expected a press and a release, got {other:?}"),
    }
}

/// **What makes a move a drag is the button set**, which is why it is on the event rather than left
/// for the application to remember.
#[test]
fn a_move_with_a_button_held_is_a_drag_and_says_so() {
    let events = parse(b"\x1b[<0;10;5M\x1b[<32;11;5M");
    match events.as_slice() {
        [_, Event::Mouse(drag)] => {
            assert_eq!(drag.kind, MouseKind::Move);
            assert!(drag.buttons.left(), "the drag carries the held button");
        }
        other => panic!("expected a press and a move, got {other:?}"),
    }
}

#[test]
fn the_wheel_is_four_directions_and_no_magnitude() {
    assert_eq!(
        one_mouse(b"\x1b[<64;1;1M").kind,
        MouseKind::Wheel(Wheel::Up)
    );
    assert_eq!(
        one_mouse(b"\x1b[<65;1;1M").kind,
        MouseKind::Wheel(Wheel::Down)
    );
    assert_eq!(
        one_mouse(b"\x1b[<66;1;1M").kind,
        MouseKind::Wheel(Wheel::Left)
    );
    assert_eq!(
        one_mouse(b"\x1b[<67;1;1M").kind,
        MouseKind::Wheel(Wheel::Right)
    );
}

#[test]
fn the_thumb_buttons_are_named_rather_than_numbered() {
    assert_eq!(
        one_mouse(b"\x1b[<128;1;1M").kind,
        MouseKind::Down(Button::Back)
    );
    assert_eq!(
        one_mouse(b"\x1b[<129;1;1M").kind,
        MouseKind::Down(Button::Forward)
    );
}

/// The three bits the 1006 encoding has room for, and no claim about the other five.
#[test]
fn a_mouse_event_reports_the_three_modifiers_the_encoding_carries() {
    let mouse = one_mouse(b"\x1b[<16;1;1M");
    assert_eq!(mouse.mods, Mods::CTRL);
}

/// **The reason SGR encoding is on whenever the mouse is on**, read from the parser's end.
///
/// The legacy encoding puts each coordinate in one byte biased by 32, so it stops at column 223 —
/// and the performance budget is written against a 300-column screen. This is the column past that
/// cliff: it arrives exactly, and it arrives because there is only one encoding in this engine.
#[test]
fn an_sgr_press_past_column_two_hundred_and_twenty_three_arrives_exactly() {
    let mouse = one_mouse(b"\x1b[<0;224;5M");
    assert_eq!((mouse.x, mouse.y), (223, 4));
    assert_eq!(mouse.kind, MouseKind::Down(Button::Left));

    // And the far end of a 300-column screen, which is what the budget is measured on.
    assert_eq!(one_mouse(b"\x1b[<0;300;80M").x, 299);
}

/// A column zero on the wire describes a cell that does not exist. It is discarded rather than
/// wrapped to 65 535, which is what a plain `- 1` would have produced.
#[test]
fn a_mouse_coordinate_of_zero_is_discarded_rather_than_underflowed() {
    assert_eq!(parse(b"\x1b[<0;0;5M"), vec![]);
}

#[test]
fn a_paste_arrives_whole_and_owns_its_bytes() {
    let events = parse(b"\x1b[200~hello\x1b[201~");
    match events.as_slice() {
        [Event::Paste(paste)] => {
            assert_eq!(paste.bytes(), b"hello");
            assert_eq!(paste.text(), "hello");
            assert!(!paste.truncated());
        }
        other => panic!("expected one paste, got {other:?}"),
    }
}

/// **Bytes, not a `String`.** A paste that is not UTF-8 survives byte for byte and only `text` is
/// lossy — a latin-1 file is the ordinary case and a panic here would be an application crashing on
/// somebody's clipboard.
#[test]
fn a_paste_that_is_not_utf8_keeps_every_byte_and_only_text_is_lossy() {
    let events = parse(b"\x1b[200~caf\xe9\x1b[201~");
    match events.as_slice() {
        [Event::Paste(paste)] => {
            assert_eq!(paste.bytes(), b"caf\xe9");
            assert_eq!(paste.text(), "caf\u{fffd}");
        }
        other => panic!("expected one paste, got {other:?}"),
    }
}

#[test]
fn a_paste_at_the_ceiling_keeps_the_prefix_and_says_it_was_truncated() {
    let events = parse_with(4, &[b"\x1b[200~abcdefgh\x1b[201~"]);
    match events.as_slice() {
        [Event::Paste(paste)] => {
            assert_eq!(paste.bytes(), b"abcd");
            assert!(paste.truncated());
        }
        other => panic!("expected one truncated paste, got {other:?}"),
    }
}

/// Exactly at the ceiling is **not** truncated, which is the boundary the ticket asks for by name.
#[test]
fn a_paste_of_exactly_the_ceiling_is_not_truncated() {
    let events = parse_with(4, &[b"\x1b[200~abcd\x1b[201~"]);
    match events.as_slice() {
        [Event::Paste(paste)] => {
            assert_eq!(paste.bytes(), b"abcd");
            assert!(!paste.truncated());
        }
        other => panic!("expected one paste, got {other:?}"),
    }
}

/// **No timer rescues it** (§9): if the closing marker never comes, the channel is already broken.
/// Nothing is emitted and nothing is lost, because there was nothing complete to hand up.
#[test]
fn an_unterminated_paste_is_not_rescued_by_anything() {
    assert_eq!(parse_with(1 << 20, &[b"\x1b[200~hello", b"more"]), vec![]);
}

/// The near-miss that the terminator scan has to survive: content that begins to look like the
/// closing marker and then does not.
#[test]
fn a_paste_containing_a_near_miss_of_its_own_terminator_keeps_it() {
    let events = parse(b"\x1b[200~a\x1b[20b\x1b[201~");
    match events.as_slice() {
        [Event::Paste(paste)] => assert_eq!(paste.bytes(), b"a\x1b[20b"),
        other => panic!("expected one paste, got {other:?}"),
    }
}

#[test]
fn a_multi_byte_scalar_reassembles_across_a_read_boundary() {
    let key = match parse_with(1 << 20, &[b"\xe6\xbc", b"\xa2"]).as_slice() {
        [Event::Key(key)] => *key,
        other => panic!("expected one key, got {other:?}"),
    };
    assert_eq!(key.code, KeyCode::Char('漢'));
    assert_eq!(key.text.as_str(), "漢");
}

/// A scalar cut off by something that is not a continuation byte drops the partial bytes rather
/// than guessing at a replacement character, and the **partial scalar** is what the diagnostic keeps.
#[test]
fn a_truncated_scalar_is_dropped_and_its_bytes_are_what_the_diagnostic_reports() {
    let mut parser = Parser::new(1 << 20);
    let at = Instant::now();
    let mut out = Vec::new();
    parser.feed(b"\xe6\xbca", at, &mut |e| out.push(e));
    match out.as_slice() {
        [Event::Key(key)] => assert_eq!(key.code, KeyCode::Char('a'), "no U+FFFD is invented"),
        other => panic!("expected only the `a`, got {other:?}"),
    }
    let (count, last) = parser.diagnostics();
    assert_eq!(count, 1);
    assert_eq!(
        last, b"\xe6\xbc",
        "the two bytes that were cut off, and nothing else"
    );
}

/// **`ESC` aborts a control sequence; it is not a parameter byte.** A truncated sequence followed by
/// a real one used to eat the second and type its final byte into the application as a capital
/// letter — a keystroke swallowed and a phantom character in its place.
#[test]
fn an_escape_inside_a_control_sequence_abandons_it_rather_than_absorbing_it() {
    let events = parse(b"\x1b[1;5\x1b[A");
    match events.as_slice() {
        [Event::Key(key)] => assert_eq!(key.code, KeyCode::Up, "the real arrow survives"),
        other => panic!("expected the arrow and nothing else, got {other:?}"),
    }
}

/// A control sequence that runs past the guard **keeps consuming to its final byte**, and what is
/// left of its parameters is refused rather than parsed. Dropping to Ground mid-sequence emitted the
/// tail as key presses, and a truncated `1;5` under a final `A` is a perfectly good `Ctrl+Up` the
/// terminal never sent.
#[test]
fn a_runaway_control_sequence_never_spills_its_tail_as_keystrokes() {
    let mut runaway = b"\x1b[".to_vec();
    runaway.extend(std::iter::repeat_n(b'1', 400));
    runaway.extend_from_slice(b";5A");
    runaway.extend_from_slice(b"x");

    let events = parse(&runaway);
    match events.as_slice() {
        [Event::Key(key)] => assert_eq!(key.code, KeyCode::Char('x'), "only the key after it"),
        other => panic!("expected one key, got {other:?}"),
    }
}

/// A well-formed length over an ill-formed scalar — here an encoded surrogate. Nothing is invented
/// and the diagnostic keeps the bytes rather than a count with nothing attached.
#[test]
fn an_ill_formed_scalar_is_counted_with_its_own_bytes() {
    let mut parser = Parser::new(1 << 20);
    let at = Instant::now();
    let mut out = Vec::new();
    parser.feed(b"\xed\xa0\x80", at, &mut |e| out.push(e));
    assert_eq!(out, vec![]);
    let (count, last) = parser.diagnostics();
    assert_eq!(count, 1);
    assert_eq!(last, b"\xed\xa0\x80");
}

/// **A motion report says what is held, and `3` means nothing is.** Press inside the window, drag
/// out, release out: the terminal sends no release, and a button set that only ever grows makes
/// every later hover a stuck drag for the rest of the session.
#[test]
fn a_hover_after_a_release_outside_the_window_is_not_a_stuck_drag() {
    // Press, drag, then a plain hover — with no release ever arriving.
    let events = parse(b"\x1b[<0;10;5M\x1b[<32;11;5M\x1b[<35;12;5M");
    match events.as_slice() {
        [_, Event::Mouse(drag), Event::Mouse(hover)] => {
            assert!(drag.buttons.left(), "the drag is still a drag");
            assert!(hover.buttons.is_empty(), "the hover is a hover");
        }
        other => panic!("expected a press, a drag and a hover, got {other:?}"),
    }
}

/// The other direction: a drag whose press this parser never saw — the button went down before the
/// application started, or the press was lost — still reports the button the terminal names.
#[test]
fn a_drag_whose_press_was_never_seen_still_reports_the_held_button() {
    let mouse = match parse(b"\x1b[<33;11;5M").as_slice() {
        [Event::Mouse(mouse)] => *mouse,
        other => panic!("expected one move, got {other:?}"),
    };
    assert_eq!(mouse.kind, MouseKind::Move);
    assert!(mouse.buttons.middle());
}

#[test]
fn an_unrecognised_sequence_is_counted_and_the_last_one_is_kept() {
    let mut parser = Parser::new(1 << 20);
    let at = Instant::now();
    let mut out = Vec::new();
    parser.feed(b"\x1b[99999q\x1b[?7$y", at, &mut |e| out.push(e));
    assert_eq!(
        out,
        vec![],
        "nothing is invented out of a sequence nobody knows"
    );
    let (count, last) = parser.diagnostics();
    assert_eq!(count, 2);
    assert_eq!(last, b"\x1b[?7$y");
}

/// An OSC reply arriving after detection is over is not forty phantom key presses. It is consumed
/// to its terminator and counted once — the same rule detection's own parser has, for the same
/// reason.
#[test]
fn a_string_sequence_is_consumed_whole_rather_than_typed() {
    let mut parser = Parser::new(1 << 20);
    let at = Instant::now();
    let mut out = Vec::new();
    parser.feed(b"\x1b]11;rgb:1111/2222/3333\x1b\\a", at, &mut |e| {
        out.push(e)
    });
    assert_eq!(out.len(), 1, "one key, and it is the `a` after the reply");
    assert_eq!(parser.diagnostics().0, 1);
}

#[test]
fn key_text_holds_a_cluster_and_empties_itself_rather_than_cutting_one_in_half() {
    let flag = KeyText::from_str("\u{1F1EF}\u{1F1F5}");
    assert_eq!(flag.as_str(), "\u{1F1EF}\u{1F1F5}");

    let too_long = KeyText::from_str(&"漢".repeat(16));
    assert!(
        too_long.is_empty(),
        "half a cluster is a different cluster, not a shorter one"
    );
}

#[test]
fn key_text_is_inline_and_small_enough_to_copy() {
    assert_eq!(size_of::<KeyText>(), KeyText::CAPACITY + 1);
    fn assert_copy<T: Copy>() {}
    assert_copy::<Key>();
}

#[test]
fn buttons_debug_names_what_is_held() {
    let mut buttons = Buttons::NONE;
    buttons.press(Button::Left);
    buttons.press(Button::Right);
    assert_eq!(format!("{buttons:?}"), "Buttons(left+right)");
    buttons.release(Button::Left);
    assert_eq!(format!("{buttons:?}"), "Buttons(right)");
}

#[test]
fn mods_debug_names_what_is_held() {
    assert_eq!(format!("{:?}", Mods::NONE), "Mods::NONE");
    assert_eq!(
        format!("{:?}", Mods::CTRL.with(Mods::CAPS)),
        "Mods(ctrl+caps)"
    );
}

/// The queue drains in arrival order, and a non-motion event is never merged with anything.
#[test]
fn the_queue_drains_in_arrival_order() {
    let queue = Queue::new();
    for c in ['a', 'b', 'c'] {
        queue.push(press(c));
    }
    let drained: Vec<_> = std::iter::from_fn(|| queue.pop())
        .map(|event| match event {
            Event::Key(key) => key.code,
            other => panic!("unexpected {other:?}"),
        })
        .collect();
    assert_eq!(
        drained,
        vec![KeyCode::Char('a'), KeyCode::Char('b'), KeyCode::Char('c')]
    );
    assert_eq!(queue.len(), 0);
}

/// A move only coalesces into a move. A press between two of them is a fence, because the press is
/// intent and the pointer was somewhere when it happened.
#[test]
fn a_press_between_two_moves_stops_them_coalescing() {
    let queue = Queue::new();
    queue.push(motion(1, 1));
    queue.push(press('a'));
    queue.push(motion(2, 2));
    assert_eq!(queue.len(), 3);
}

/// The newest position supersedes the older one **whole** — buttons and modifiers included, not
/// only the coordinates.
#[test]
fn a_coalesced_move_is_the_newer_event_entire() {
    let queue = Queue::new();
    queue.push(motion(1, 1));
    queue.push(motion(7, 9));
    assert_eq!(queue.len(), 1);
    match queue.pop() {
        Some(Event::Mouse(mouse)) => assert_eq!((mouse.x, mouse.y), (7, 9)),
        other => panic!("expected one move, got {other:?}"),
    }
}

pub(crate) fn press(c: char) -> Event {
    Event::Key(Key {
        code: KeyCode::Char(c),
        mods: Mods::NONE,
        kind: KeyKind::Press,
        text: KeyText::from_str(&c.to_string()),
        at: Instant::now(),
    })
}

pub(crate) fn motion(x: u16, y: u16) -> Event {
    Event::Mouse(Mouse {
        x,
        y,
        kind: MouseKind::Move,
        buttons: Buttons::NONE,
        mods: Mods::NONE,
        at: Instant::now(),
    })
}
