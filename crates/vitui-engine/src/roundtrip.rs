//! The round trip: the primary instrument, and it stores nothing.
//!
//! Composite a frame, serialise it, replay the bytes through the terminal model, assert the
//! replayed screen equals the frame that was composited — cell for cell, glyph and style. All four
//! defects the architecture map found were found this way. There is no file to review,
//! nothing to bless and no maintenance, and a golden byte string would have pinned the encoding,
//! which is exactly the part that is allowed to change.
//!
//! Every scene the tracer bullet can express is driven through here. The golden frames come next,
//! which cover the one thing this cannot reach: the composited picture itself.

use crate::geom::Rect;
use crate::style::{Color, Style};
use crate::surface::Surface;
use crate::testing::{Harness, Recorder};

#[test]
fn a_screen_with_no_layers_presents_nothing() {
    let mut h = Harness::new(80, 24);
    let p = h.present();
    assert!(!p.submitted);
    assert_eq!(h.bytes_written(), 0);
    assert_eq!(h.writes(), 0);
}

#[test]
fn one_line_of_text_survives_the_round_trip() {
    let mut h = Harness::new(80, 24);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 80, 24), true);
    h.screen
        .layers()
        .view(id)
        .unwrap()
        .text(3, 1, "hello", Style::new());
    assert!(h.present().submitted);
}

#[test]
fn cjk_survives_the_round_trip() {
    let mut h = Harness::new(20, 2);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 20, 2), true);
    h.screen
        .layers()
        .view(id)
        .unwrap()
        .text(2, 0, "漢字テスト", Style::new());
    assert!(h.present().submitted);
}

#[test]
fn a_multi_scalar_cluster_survives_the_round_trip() {
    let mut h = Harness::new(20, 2);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 20, 2), true);
    let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
    let mut v = h.screen.layers().view(id).unwrap();
    v.text(0, 0, "e\u{301}a\u{308}", Style::new());
    v.text(0, 1, family, Style::new().bold());
    assert!(h.present().submitted);
}

#[test]
fn a_wide_glyph_broken_in_half_survives_the_round_trip() {
    // The five repair rules, all the way to the wire: the blanked half has to reach the terminal or
    // it keeps showing the glyph that is no longer there.
    let mut h = Harness::new(12, 1);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 12, 1), true);
    h.screen
        .layers()
        .view(id)
        .unwrap()
        .text(0, 0, "漢字", Style::new());
    h.present();
    h.screen
        .layers()
        .view(id)
        .unwrap()
        .text(1, 0, "x", Style::new());
    assert!(h.present().submitted);
}

#[test]
fn two_cells_whose_clusters_would_join_on_the_wire_survive_the_round_trip() {
    // Cells are emitted back to back with nothing between them, and UAX #29 does not know where one
    // cell ended. A cluster ending in ZWJ followed by a pictograph is one cluster when concatenated,
    // and so is a lone regional indicator followed by another — so what the terminal draws is not
    // what the frame holds unless the serializer breaks the run.
    let mut h = Harness::new(20, 2);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 20, 2), true);
    let mut v = h.screen.layers().view(id).unwrap();
    v.text(0, 0, "\u{1F468}\u{200D}", Style::new());
    v.text(2, 0, "\u{1F469}", Style::new());
    v.text(0, 1, "\u{1F1FA}", Style::new());
    v.text(1, 1, "\u{1F1F8}", Style::new());
    assert!(h.present().submitted);
}

#[test]
fn every_attribute_survives_the_round_trip() {
    let mut h = Harness::new(40, 2);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 40, 2), true);
    let mut v = h.screen.layers().view(id).unwrap();
    let styles = [
        Style::new().bold(),
        Style::new().dim(),
        Style::new().bold().dim(),
        Style::new().italic(),
        Style::new().reverse(),
        Style::new().blink(),
        Style::new().strikethrough(),
        Style::new().conceal(),
        Style::new().overline(),
        Style::new().underline(),
        Style::new().underline_double(),
        Style::new().underline_curly(),
        Style::new().underline_dotted(),
        Style::new().underline_dashed(),
    ];
    for (i, st) in styles.iter().enumerate() {
        v.text(i as i32, 0, "x", *st);
    }
    h.present();
}

#[test]
fn every_colour_form_survives_the_round_trip() {
    let mut h = Harness::truecolor(40, 2);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 40, 2), true);
    let mut v = h.screen.layers().view(id).unwrap();
    let colours = [
        Color::DEFAULT,
        Color::indexed(0),
        Color::indexed(7),
        Color::indexed(8),
        Color::indexed(15),
        Color::indexed(16),
        Color::indexed(255),
        Color::rgb(0, 0, 0),
        Color::rgb(255, 255, 255),
        Color::rgb(0x12, 0x34, 0x56),
    ];
    for (i, c) in colours.iter().enumerate() {
        v.text(i as i32, 0, "f", Style::new().fg(*c));
        v.text(i as i32, 1, "b", Style::new().bg(*c));
    }
    h.present();
}

#[test]
fn dropping_bold_while_keeping_dim_survives_the_round_trip() {
    // SGR 22 clears both. This is the scene that catches a per-attribute emit loop.
    let mut h = Harness::new(8, 1);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 8, 1), true);
    let mut v = h.screen.layers().view(id).unwrap();
    v.text(0, 0, "ab", Style::new().bold().dim());
    v.text(2, 0, "cd", Style::new().dim());
    h.present();
}

#[test]
fn two_spans_on_one_row_survive_the_round_trip() {
    // Three dialogs standing apart: the scene that decided the damage structure.
    let mut h = Harness::new(300, 4);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 300, 4), true);
    let mut v = h.screen.layers().view(id).unwrap();
    v.text(0, 0, "left", Style::new());
    v.text(150, 0, "middle", Style::new());
    v.text(290, 0, "right", Style::new());
    h.present();
}

#[test]
fn a_sparse_scatter_survives_the_round_trip() {
    // The sub-cell chart's shape: many short runs, which is the case the bitset exists for.
    let mut h = Harness::truecolor(300, 80);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 300, 80), true);
    let mut v = h.screen.layers().view(id).unwrap();
    for i in 0..400 {
        let x = (i * 7) % 300;
        let y = (i * 13) % 80;
        v.text(x, y, "*", Style::new().fg(Color::indexed(i as u8 % 16)));
    }
    h.present();
}

#[test]
fn a_full_screen_survives_the_round_trip() {
    let mut h = Harness::truecolor(300, 80);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 300, 80), true);
    let mut v = h.screen.layers().view(id).unwrap();
    let row: String = (0..300)
        .map(|i| char::from(b'!' + (i % 90) as u8))
        .collect();
    for y in 0..80 {
        v.text(0, y, &row, Style::new().fg(Color::rgb(y as u8, 128, 32)));
    }
    h.present();
}

#[test]
fn a_fill_survives_the_round_trip() {
    let mut h = Harness::truecolor(40, 8);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 40, 8), true);
    h.screen.layers().view(id).unwrap().fill(
        Rect::new(2, 1, 20, 4),
        "#",
        Style::new().bg(Color::indexed(4)),
    );
    h.present();
}

#[test]
fn two_layers_survive_the_round_trip() {
    let mut h = Harness::truecolor(40, 8);
    let below = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 40, 8), true);
    let above = h
        .screen
        .layers()
        .add_content(1, Rect::new(10, 2, 12, 3), true);
    h.screen.layers().view(below).unwrap().fill(
        Rect::new(0, 0, 40, 8),
        ".",
        Style::new().fg(Color::indexed(8)),
    );
    h.screen.layers().view(above).unwrap().fill(
        Rect::new(0, 0, 12, 3),
        " ",
        Style::new().bg(Color::indexed(4)),
    );
    h.screen
        .layers()
        .view(above)
        .unwrap()
        .text(1, 1, "popup", Style::new().bold());
    h.present();
}

#[test]
fn a_non_opaque_layer_survives_the_round_trip() {
    let mut h = Harness::truecolor(40, 4);
    let below = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 40, 4), true);
    let above = h
        .screen
        .layers()
        .add_content(1, Rect::new(0, 0, 40, 4), false);
    h.screen
        .layers()
        .view(below)
        .unwrap()
        .fill(Rect::new(0, 0, 40, 4), ".", Style::new());
    h.screen
        .layers()
        .view(above)
        .unwrap()
        .text(5, 1, "over", Style::new().fg(Color::rgb(9, 9, 9)));
    h.present();
}

#[test]
fn a_layer_hanging_off_an_edge_survives_the_round_trip() {
    let mut h = Harness::new(20, 4);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(-3, -1, 10, 3), true);
    h.screen
        .layers()
        .view(id)
        .unwrap()
        .fill(Rect::new(0, 0, 10, 3), "#", Style::new());
    h.present();
}

#[test]
fn a_write_in_the_last_column_survives_the_round_trip() {
    // Auto-wrap is off, so this must not scroll the screen: the cursor stops in the last column
    // rather than wrapping, and both the serializer and the terminal model say so.
    let mut h = Harness::new(8, 2);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 8, 2), true);
    let mut v = h.screen.layers().view(id).unwrap();
    v.text(7, 0, "x", Style::new());
    v.text(7, 1, "y", Style::new());
    h.present();
}

#[test]
fn a_second_frame_with_nothing_changed_emits_nothing() {
    let mut h = Harness::new(80, 24);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 80, 24), true);
    h.screen
        .layers()
        .view(id)
        .unwrap()
        .text(0, 0, "steady", Style::new());
    assert!(h.present().submitted);
    let after_first = h.bytes_written();
    assert!(after_first > 0);

    let second = h.present();
    assert!(
        !second.submitted,
        "nothing was damaged, so nothing was submitted"
    );
    assert_eq!(
        h.bytes_written(),
        after_first,
        "and nothing reached the sink"
    );
}

#[test]
fn a_change_between_frames_emits_only_what_changed() {
    let mut h = Harness::new(80, 24);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 80, 24), true);
    h.screen
        .layers()
        .view(id)
        .unwrap()
        .fill(Rect::new(0, 0, 80, 24), ".", Style::new());
    h.present();
    let after_first = h.bytes_written();

    h.screen
        .layers()
        .view(id)
        .unwrap()
        .text(40, 12, "!", Style::new());
    h.present();
    let second_frame = h.bytes_written() - after_first;
    assert!(
        second_frame < 32,
        "one changed cell cost {second_frame} bytes; the frame's own framing is 4 plus one move"
    );
}

#[test]
fn every_frame_is_one_write() {
    let mut h = Harness::truecolor(300, 80);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 300, 80), true);
    let row: String = std::iter::repeat_n('x', 300).collect();
    for frame in 0..5 {
        let mut v = h.screen.layers().view(id).unwrap();
        for y in 0..80 {
            v.text(0, y, &row, Style::new().fg(Color::indexed(frame)));
        }
        h.present();
    }
    assert_eq!(h.writes(), 5, "five frames, five writes");
}

/// **And a frame the equality filter emptied is no write at all.**
///
/// The same fixture with the colour held still, so every cell of every frame after the first equals
/// what the mirror already holds. Four frames of twenty-four thousand damaged cells reach the sink as
/// nothing — not as the four bytes of a style reset announcing that they have nothing to say, which is
/// what a serializer that wrote its framing up front would have to send.
///
/// `submitted` stays true, and the distinction is deliberate: `present` submitted a frame, because
/// damage existed and the composite ran. What the *wire* got is a separate question, and this is where
/// it is asked.
#[test]
fn a_frame_the_filter_emptied_is_no_write_at_all() {
    let mut h = Harness::truecolor(300, 80);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 300, 80), true);
    let row: String = std::iter::repeat_n('x', 300).collect();
    for _ in 0..5 {
        let mut v = h.screen.layers().view(id).unwrap();
        for y in 0..80 {
            v.text(0, y, &row, Style::new().fg(Color::indexed(3)));
        }
        assert!(h.present().submitted, "the cells were damaged either way");
    }
    assert_eq!(h.writes(), 1, "the birth frame, and silence after it");
    assert!(h.bytes_written() > 0, "the birth frame did write");
}

#[test]
fn a_frame_reassembles_through_a_sink_that_takes_seven_bytes_at_a_time() {
    // Spec the honest partial-write test: the fragmentation is deterministic, because whether a
    // real pipe fragments a write is the kernel's business.
    let mut h = Harness::with_sink(300, 80, Recorder::awkward(7, 5));
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 300, 80), true);
    let row: String = std::iter::repeat_n('z', 300).collect();
    let mut v = h.screen.layers().view(id).unwrap();
    for y in 0..80 {
        v.text(0, y, &row, Style::new());
    }
    // The harness asserts the reassembled stream replays to the composited frame.
    h.present();
    assert!(h.writes() > 100, "the sink took seven bytes at a time");
    assert!(h.retries() > 0, "and refused every fifth call");
}

#[test]
fn five_frames_of_a_moving_selection_bar_survive_the_round_trip() {
    let mut h = Harness::truecolor(60, 10);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 60, 10), true);
    for y in 0..10 {
        h.screen
            .layers()
            .view(id)
            .unwrap()
            .text(0, y, "a list row", Style::new());
    }
    h.present();

    for row in 0..5 {
        let mut v = h.screen.layers().view(id).unwrap();
        if row > 0 {
            v.text(0, row - 1, "a list row", Style::new());
        }
        v.text(0, row, "a list row", Style::new().bg(Color::indexed(4)));
        assert!(h.present().submitted);
    }
}

#[test]
fn a_frame_that_blanks_what_it_drew_survives_the_round_trip() {
    let mut h = Harness::truecolor(20, 3);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 20, 3), true);
    h.screen
        .layers()
        .view(id)
        .unwrap()
        .text(2, 1, "temporary", Style::new().fg(Color::indexed(2)));
    h.present();

    h.screen
        .layers()
        .view(id)
        .unwrap()
        .fill(Rect::new(2, 1, 9, 1), " ", Style::new());
    assert!(h.present().submitted);
}

#[test]
fn a_layer_added_over_a_painted_screen_repaints_what_it_covers() {
    let mut h = Harness::new(30, 5);
    let below = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 30, 5), true);
    h.screen
        .layers()
        .view(below)
        .unwrap()
        .fill(Rect::new(0, 0, 30, 5), ".", Style::new());
    h.present();

    let above = h
        .screen
        .layers()
        .add_content(1, Rect::new(5, 1, 10, 2), true);
    h.screen
        .layers()
        .view(above)
        .unwrap()
        .fill(Rect::new(0, 0, 10, 2), "#", Style::new());
    assert!(h.present().submitted);
}

#[test]
fn a_layer_removed_from_a_painted_screen_repaints_what_it_covered() {
    // The other direction, and the one that needed `composite_run`'s ground fill: what the popup
    // covered belongs to nobody once it is gone, so the frame's own blank is what has to reach the
    // wire. The round trip is what says it did.
    let mut h = Harness::new(30, 5);
    let below = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 20, 5), true);
    h.screen
        .layers()
        .view(below)
        .unwrap()
        .fill(Rect::new(0, 0, 20, 5), ".", Style::new());
    let above = h
        .screen
        .layers()
        .add_content(1, Rect::new(15, 1, 10, 2), true);
    h.screen
        .layers()
        .view(above)
        .unwrap()
        .fill(Rect::new(0, 0, 10, 2), "#", Style::new());
    h.present();

    assert!(h.screen.layers().remove(above));
    assert!(h.present().submitted);
}

#[test]
fn a_raised_layer_and_its_return_survive_the_round_trip() {
    let mut h = Harness::new(20, 3);
    let a = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 20, 3), true);
    let b = h
        .screen
        .layers()
        .add_content(0, Rect::new(4, 1, 8, 1), true);
    h.screen
        .layers()
        .view(a)
        .unwrap()
        .fill(Rect::new(0, 0, 20, 3), ".", Style::new());
    h.screen
        .layers()
        .view(b)
        .unwrap()
        .fill(Rect::new(0, 0, 8, 1), "#", Style::new());
    h.present();

    assert!(h.screen.layers().set_z(b, -1));
    h.present();
    assert!(h.screen.layers().set_z(b, 0));
    h.present();
}

#[test]
fn a_moved_layer_survives_the_round_trip() {
    let mut h = Harness::new(20, 3);
    let back = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 20, 3), true);
    let window = h
        .screen
        .layers()
        .add_content(1, Rect::new(0, 0, 6, 2), true);
    h.screen
        .layers()
        .view(back)
        .unwrap()
        .fill(Rect::new(0, 0, 20, 3), ".", Style::new());
    h.screen
        .layers()
        .view(window)
        .unwrap()
        .fill(Rect::new(0, 0, 6, 2), "#", Style::new());
    h.present();

    assert!(h.screen.layers().set_rect(window, Rect::new(12, 1, 6, 2)));
    assert!(h.present().submitted);
}

#[test]
fn a_donated_surface_of_clusters_survives_the_round_trip() {
    // The renumbering, end to end. The harness compares whole cells — handle and style word — so a
    // handle that landed in the wrong row of the interner fails here rather than showing the wrong
    // glyph on someone's terminal.
    //
    // **The extended half of the donation is driven through here since impl 13**, and it was this
    // file's boundary rather than a gap in ticket 10: until SGR 58/59 and OSC 8 reached the wire an
    // extended cell could not close the round trip at all, so what the renumbering did to an
    // underline colour was asserted against the composited frame instead, in `layer::tests`. It is
    // asserted end to end below — the donor's own extended-style entries are re-minted into the
    // stack's table at donation, and the terminal model resolves the bytes back through that same
    // table, so a handle that landed in the wrong row fails here.
    //
    // The **hyperlink** half is still not driven, and that is architecture ticket 21 rather than this
    // file: no public door reaches a standalone surface's link table, so the only id a caller can put
    // on a donated surface already belongs to the destination stack, and there is nothing for the
    // donation to renumber. `layer::tests::a_screen_minted_link_survives_a_donation_it_was_not_minted_for`
    // is that case.
    let mut h = Harness::with_overrides(20, 2, crate::testing::pinned_extended());

    // Something is already in this screen's tables, so the donor's ids are not this screen's.
    let seeded = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 20, 2), true);
    h.screen
        .layers()
        .view(seeded)
        .unwrap()
        .text(0, 0, "e\u{301}a\u{308}", Style::new());
    h.present();

    let mut off = Surface::new(10, 1);
    off.root().text(
        0,
        0,
        "n\u{303}o\u{308}漢",
        Style::new().fg(Color::rgb(3, 4, 5)),
    );
    // Two distinct underline colours over the clusters, which is what puts entries in the donor's
    // *extended-style* table as well as in its interner — the half a plain donation never reaches.
    for (x, ul) in [(0, Color::rgb(9, 8, 7)), (1, Color::rgb(7, 8, 9))] {
        off.root().restyle(
            Rect::new(x, 0, 1, 1),
            &crate::restyle::Restyle {
                ul: Some(ul),
                ..Default::default()
            },
        );
    }
    h.screen
        .layers()
        .add_content_with(1, Rect::new(2, 1, 10, 1), true, off);
    assert!(h.present().submitted);
    // The harness compares whole cells, so the assertion above already covers this; naming it is
    // what stops a future edit deleting the `restyle` calls and leaving a test that says *clusters*
    // in its name and means it.
    assert!(
        h.screen.frame().row(1)[2].style.ext_handle().is_some(),
        "the donated cell has to be extended, or the extended half is not being driven"
    );
}

// -------------------------------------------------------------------------------------------------
// The scroll region. Every one of these closes the round trip, which is the point: the
// defect a scroll optimisation ships is a column nobody put back, and only a replayed screen sees it.
// -------------------------------------------------------------------------------------------------

/// The screen the scroll fixtures are written at. Small enough to read a failure, tall enough that a
/// band has rows to move.
const SW: u16 = 40;
const SH: u16 = 8;

/// A list whose item `n` is drawn on row `y`, blanking the row first — the idiom found to be
/// load-bearing rather than a wart.
fn draw_list(h: &mut Harness, id: crate::layer::LayerId, top_item: u32, clear: bool) {
    let mut v = h.screen.layers().view(id).unwrap();
    for y in 0..SH {
        if clear {
            v.fill(Rect::new(0, y as i32, SW, 1), " ", Style::new());
        }
        v.text(
            0,
            y as i32,
            &format!("row {}", top_item + u32::from(y)),
            Style::new(),
        );
    }
}

/// The whole point, at its smallest: a list that scrolls one row a frame becomes one `SU` and the row
/// it exposes.
///
/// Three properties in one fixture, because they are one mechanism: the pre-pass runs on every steady
/// frame, the round trip closes on all of them, and the frame costs what a scroll costs rather than
/// what eighty rows cost.
#[test]
fn a_cleared_row_list_scroll_is_one_scroll_and_the_row_it_exposes() {
    let mut h = Harness::new(SW, SH);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, SW, SH), true);
    draw_list(&mut h, id, 0, true);
    h.present();
    let birth = h.bytes_written();

    let mut frames = Vec::new();
    for t in 1..=3 {
        let at = h.bytes_written();
        draw_list(&mut h, id, t, true);
        h.present();
        frames.push(h.bytes_written() - at);
    }
    let (scrolls, verifies) = h.scrolls();
    assert_eq!(
        scrolls, 3,
        "every steady frame of a scrolling list is a scroll"
    );
    assert_eq!(verifies, 3, "one candidate a frame, never more");

    // The whole band, so no `DECSTBM` and no reset: `SGR 0`, `SU`, one move, and the label of the
    // row the scroll exposed. Against a birth frame that wrote every one of the 320 cells.
    assert!(
        frames.iter().all(|f| *f < 24),
        "a scrolled frame is a scroll and one label; these cost {frames:?} bytes against a birth \
         frame's {birth}"
    );
}

/// **The bug that nearly shipped**, as the spec describes it: two text verbs with a one-column gap
/// between them, and the gap is a column no later pass could put back.
///
/// The gap column is painted once and never repainted. A *speculative* scroll would move it up with
/// everything else and leave the bottom one blank, and the filter behind it could not repair that
/// because the packet does not carry the column. **Obligation 2 is what refuses it** — the row the
/// scroll would expose is not blank in a column this frame does not repaint — which is why that
/// obligation is checked first.
#[test]
fn two_text_verbs_with_a_one_column_gap_do_not_take_the_scroll_path() {
    const GAP: u16 = 12;
    let mut h = Harness::new(SW, SH);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, SW, SH), true);
    {
        let mut v = h.screen.layers().view(id).unwrap();
        for y in 0..SH {
            v.text(GAP as i32, y as i32, "|", Style::new());
        }
    }
    let draw = |h: &mut Harness, t: u32| {
        let mut v = h.screen.layers().view(id).unwrap();
        for y in 0..SH {
            let n = t + u32::from(y);
            v.text(0, y as i32, &format!("row {n:<7}"), Style::new());
            v.text(
                GAP as i32 + 1,
                y as i32,
                &format!("item {n:<10}"),
                Style::new(),
            );
        }
    };
    draw(&mut h, 0);
    h.present();
    for t in 1..=3 {
        draw(&mut h, t);
        h.present();
    }

    assert_eq!(
        h.scrolls().0,
        0,
        "the column between the two verbs is one no later pass could put back"
    );
    assert_eq!(
        h.terminal_glyph(GAP, SH - 1),
        Some('|'),
        "and it is still on the terminal, on the row a scroll would have exposed"
    );
}

/// The same list, with content past the label that the label-only idiom leaves in place.
///
/// **This is *label only* arm and the reason it is refused**: the tail belongs to the screen row
/// rather than to the item, so what the frame wants on row `y` is not what the mirror holds on row
/// `y + 1`, and obligation 1 says so. The `scrolling-list-label-only` is blank past its label
/// and therefore *is* scrollable — see `crate::gates::the_scroll_region_over_spec_8s_two_arms`.
#[test]
fn a_list_whose_tail_does_not_scroll_with_its_labels_is_refused() {
    let mut h = Harness::new(SW, SH);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, SW, SH), true);
    {
        let mut v = h.screen.layers().view(id).unwrap();
        for y in 0..SH {
            v.text(10, y as i32, &format!("lane {y}"), Style::new());
        }
    }
    for t in 0..=3 {
        {
            let mut v = h.screen.layers().view(id).unwrap();
            for y in 0..SH {
                v.text(
                    0,
                    y as i32,
                    &format!("row {:<5}", t + u32::from(y)),
                    Style::new(),
                );
            }
        }
        h.present();
    }
    assert_eq!(
        h.scrolls().0,
        0,
        "the tail is the screen row's, so nothing lands where the mirror has it"
    );
}

/// A pane narrower than the screen: the case that scrolls and the case that must not, **each of them
/// scrolling each way.**
///
/// `SU` has no horizontal margins, so a scroll of a band moves every column of it. The pane may
/// therefore scroll only where the columns it does not own hold the same thing after the shift as
/// before it — blank in both frames being the ordinary case of that.
///
/// Four arms rather than two, because the two axes are independent: whether the columns beside the
/// pane are blank decides obligation 1, and which way the list moves decides which edge of the band
/// the probe reads and which end of it the terminal erases. A pane that scrolled forwards and not
/// back would pass a two-arm version of this.
#[test]
fn a_pane_narrower_than_the_screen_scrolls_only_where_the_columns_beside_it_are_blank() {
    const PANE: Rect = Rect::new(6, 0, 16, SH);

    let run = |decorate: bool, forwards: bool| {
        let mut h = Harness::new(SW, SH);
        let base = h
            .screen
            .layers()
            .add_content(0, Rect::new(0, 0, SW, SH), true);
        if decorate {
            let mut v = h.screen.layers().view(base).unwrap();
            for y in 0..SH {
                v.text(30, y as i32, &format!("#{y}"), Style::new());
            }
        }
        let pane = h.screen.layers().add_content(1, PANE, true);
        // Forwards is `SU` and backwards is `SD`. The same four item numbers either way, so the two
        // arms differ in direction and in nothing else.
        let steps: Vec<u32> = if forwards {
            (0..=3).collect()
        } else {
            (0..=3).rev().collect()
        };
        for t in steps {
            {
                let mut v = h.screen.layers().view(pane).unwrap();
                for y in 0..SH {
                    v.fill(Rect::new(0, y as i32, 16, 1), " ", Style::new());
                    v.text(
                        0,
                        y as i32,
                        &format!("row {}", t + u32::from(y)),
                        Style::new(),
                    );
                }
            }
            h.present();
        }
        h.scrolls().0
    };

    for (forwards, way) in [(true, "forwards, `SU`"), (false, "backwards, `SD`")] {
        assert_eq!(
            run(false, forwards),
            3,
            "{way}: the columns beside the pane are blank in both frames"
        );
        assert_eq!(
            run(true, forwards),
            0,
            "{way}: the columns beside the pane belong to the screen row rather than to the item, \
             and a scroll of the band would move them"
        );
    }
}

/// The other direction: a list scrolled backwards is `SD`, and it is the same mechanism.
///
/// The band's edge is the other one — `SD` leaves the **bottom** row holding what was above it — so
/// this is the half of the probe the forward case never reaches.
#[test]
fn a_list_scrolled_backwards_takes_the_scroll_path_the_other_way() {
    let mut h = Harness::new(SW, SH);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, SW, SH), true);
    for t in (0..=3).rev() {
        draw_list(&mut h, id, t, true);
        h.present();
    }
    assert_eq!(
        h.scrolls(),
        (3, 3),
        "three frames back up the list, three scrolls"
    );
    assert_eq!(
        h.terminal_glyph(4, 0),
        Some('0'),
        "and the row the scroll exposed at the top carries item 0"
    );
}

/// The exposed row is recorded as **blank**, not as unknown, and both halves of that are asserted.
///
/// *Whole*, because the mirror knows every column of it afterwards even though the frame repainted
/// only the label — a row the terminal erased is a row the terminal erased. And *blank* rather than
/// unknown, because the blanks the frame wants beside the label then compare equal and cost nothing:
/// recorded unknown they would all be re-emitted, which is the whole row again and is what the
/// optimisation was for.
#[test]
fn the_row_a_scroll_exposes_is_recorded_blank_and_recorded_whole() {
    let mut h = Harness::new(SW, SH);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, SW, SH), true);
    draw_list(&mut h, id, 0, true);
    h.present();
    let before = h.bytes_written();

    draw_list(&mut h, id, 1, true);
    h.present();
    assert_eq!(h.scrolls().0, 1);
    assert!(
        h.screen.mirror().is_known(SH - 1),
        "the terminal erased every column of the row it exposed, so the mirror knows every column"
    );
    let frame = h.bytes_written() - before;
    assert!(
        frame < SW as usize,
        "the exposed row was damaged in all {SW} columns and only its label changed; the frame cost \
         {frame} bytes"
    );
}

/// A band narrower than the screen: `DECSTBM`, the scroll, and `DECSTBM` reset — one of each.
///
/// The full-screen case omits both region escapes, because the region a terminal starts in *is* the
/// screen, so this is the only fixture that puts them on the wire at all. It is also the one that says
/// the region is put **back**: a serializer that left it set would change what an `LF` means for every
/// frame after, and `shortest` prices an `LF` as a move rather than as a scroll.
#[test]
fn a_band_narrower_than_the_screen_sets_the_region_and_puts_it_back() {
    const TOP: u16 = 2;
    const ROWS: u16 = 4;

    let mut h = Harness::new(SW, SH);
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, SW, SH), true);
    {
        // Rows outside the band, painted once and never again: they are what keeps the band a band.
        let mut v = h.screen.layers().view(id).unwrap();
        for y in 0..SH {
            if !(TOP..TOP + ROWS).contains(&y) {
                v.text(0, y as i32, "static", Style::new());
            }
        }
    }
    let draw = |h: &mut Harness, t: u32| {
        let mut v = h.screen.layers().view(id).unwrap();
        for y in TOP..TOP + ROWS {
            v.fill(Rect::new(0, y as i32, SW, 1), " ", Style::new());
            v.text(
                0,
                y as i32,
                &format!("row {}", t + u32::from(y)),
                Style::new(),
            );
        }
    };
    draw(&mut h, 0);
    h.present();
    let before = h.wire().len();

    draw(&mut h, 1);
    h.present();
    assert_eq!(h.scrolls(), (1, 1));

    let frame = h.wire()[before..].to_vec();
    let finals: Vec<u8> = crate::serial::csi_finals(&frame).collect();
    assert_eq!(
        finals.iter().filter(|b| **b == b'r').count(),
        2,
        "one `DECSTBM` and one reset: {:?}",
        String::from_utf8_lossy(&frame)
    );
    assert_eq!(
        finals.iter().filter(|b| **b == b'S').count(),
        1,
        "one scroll: {:?}",
        String::from_utf8_lossy(&frame)
    );
    assert!(
        frame.windows(3).any(|w| w == b"\x1b[r"),
        "the region is put back, not left set: {:?}",
        String::from_utf8_lossy(&frame)
    );
    assert_eq!(
        h.terminal_glyph(0, TOP + ROWS),
        Some('s'),
        "and the row below the band is untouched by a scroll of it"
    );
}

/// **The scroll region has to survive narrowing, and for three frames it did not.**
///
/// The pre-pass proves a scroll by comparing what the frame wants against what the mirror holds, and
/// since impl 17 the mirror holds what the terminal was **sent** — narrowed. A comparison that read
/// the packet's own cells against it made every colour below truecolor look like a change, so
/// obligation 1 failed on the first row and **every scroll on every terminal that narrows anything**
/// was forfeited: correct output, 32x the bytes, and nothing red.
///
/// It was invisible to the whole suite for one reason worth keeping in view: impl 17 also made
/// `Scene::overrides` pin **truecolor**, which is the arm where narrowing is the identity. That was
/// the right default — it preserves every byte count on the register — and it moved the twelve
/// scenes off the only depth that could see this. So the gate is here, at the depth, and it is a
/// **relation between arms** rather than a count: a scroll taken at truecolor must still be taken
/// when the terminal has less colour, because narrowing is about what a cell *looks like* and a
/// scroll is about where it *is*.
#[test]
fn a_scroll_is_taken_at_every_depth_and_not_only_where_nothing_narrows() {
    fn scrolls(depth: crate::caps::ColorDepth) -> (usize, usize) {
        let mut h = Harness::with_overrides(
            SW,
            SH,
            crate::caps::Overrides {
                colors: Some(depth),
                ..Default::default()
            },
        );
        let id = h
            .screen
            .layers()
            .add_content(0, Rect::new(0, 0, SW, SH), true);
        // **An RGB ink, which is the whole fixture.** `draw_list` above paints in the default
        // colours, and a default colour is the one thing no depth narrows — so the shipped scroll
        // tests could not have caught this and neither could a copy of them.
        let ink = Style::new().fg(Color::rgb(0xcc, 0x02, 0x01));
        let paint = |h: &mut Harness, top: u32| {
            let mut v = h
                .screen
                .layers()
                .view(id)
                .expect("the layer is still there");
            for y in 0..SH {
                v.fill(Rect::new(0, y as i32, SW, 1), " ", ink);
                v.text(0, y as i32, &format!("row {}", top + u32::from(y)), ink);
            }
        };
        paint(&mut h, 0);
        h.present();
        for t in 1..=3 {
            paint(&mut h, t);
            h.present();
        }
        h.scrolls()
    }

    let reference = scrolls(crate::caps::ColorDepth::TrueColor);
    assert_eq!(
        reference,
        (3, 3),
        "the fixture stopped scrolling at all, so the arms below compare nothing"
    );
    for depth in [
        crate::caps::ColorDepth::Indexed256,
        crate::caps::ColorDepth::Ansi16,
        crate::caps::ColorDepth::None,
    ] {
        assert_eq!(
            scrolls(depth),
            reference,
            "{depth:?} did not take the scroll truecolor took — the pre-pass is comparing what the \
             application asked for against what the terminal was sent"
        );
    }
}

/// **Gate, equality: the round trip closes on a terminal that drops a flag.**
///
/// The two gates in `crate::serial` say what goes on the wire and what the model ends up holding.
/// This one is the instrument itself: `Harness::present` asserts the replayed screen against the
/// frame *and* the mirror against the frame, both through `quant::OnTheWire`, and a frame asking for
/// overline against a terminal whose bytes deliberately do not carry it is a real inequality until
/// the expectation is narrowed too. Without the narrowing this test fails — which is the whole reason
/// the fix was a change of its own and not a two-character patch.
///
/// **It narrows the one flag and not the ten**, so this is stronger than an exemption: a serializer
/// that dropped italic here, or one that kept the overline the quirk table says tmux throws away,
/// fails inside `present` with no assertion of its own.
#[test]
fn the_round_trip_closes_on_a_terminal_that_drops_an_attribute() {
    let mut h = Harness::declaring(
        8,
        1,
        crate::caps::Capabilities::identified_as("tmux 3.7c"),
        crate::input::InputConfig::default(),
    );
    let id = h
        .screen
        .layers()
        .add_content(0, Rect::new(0, 0, 8, 1), true);
    let all = Style::new()
        .bold()
        .dim()
        .italic()
        .reverse()
        .blink()
        .strikethrough()
        .conceal()
        .overline()
        .underline_double()
        .fg(Color::rgb(1, 2, 3));
    h.screen
        .layers()
        .view(id)
        .expect("the layer is still there")
        .text(0, 0, "abcd", all);
    assert!(h.present().submitted);

    // The bit is absent from the *bytes* and not merely from the comparison, which is the half a
    // narrowed expectation could hide on its own.
    let wire = String::from_utf8_lossy(&h.wire()).replace('\x1b', "ESC");
    assert!(!wire.contains("53"), "{wire}");
    assert!(wire.contains("28"), "conceal still goes out: {wire}");

    let shown = h.terminal().cell(0, 0).style;
    assert_eq!(
        shown.attrs(),
        all.attrs() & !crate::style::OVERLINE,
        "overline and nothing else"
    );
    assert_eq!(shown.underline_style(), all.underline_style());
}

/// **Gate, equality: a scroll is still taken on a terminal that drops a
/// flag.**
///
/// The sibling of `a_scroll_is_taken_at_every_depth_and_not_only_where_nothing_narrows`, and it exists
/// for the same defect one field along. The scroll pre-pass compares the packet's own cells against a
/// mirror holding what the terminal was **sent**, and `Quantiser::narrows` is the fast path that
/// decides whether that comparison may be a slice `==`. A truecolor terminal that drops a flag
/// narrows something, so a `narrows` keyed on `depth` alone takes the slice path with a masked mirror
/// against an unmasked packet: every cell carrying the flag reads as a change, obligation 1 fails on
/// the first row, and **every scroll on that terminal is forfeited** — correct output, 32x the bytes,
/// nothing red.
///
/// The two arms differ in the quirk entry and in nothing else, which is why the ink has to carry the
/// dropped bit: paint in anything the entry does not touch and both arms are the same test.
#[test]
fn a_scroll_is_taken_on_a_terminal_that_drops_an_attribute() {
    fn scrolls(version: &str) -> (usize, usize) {
        let mut h = Harness::declaring(
            SW,
            SH,
            crate::caps::Capabilities::identified_as(version),
            crate::input::InputConfig::default(),
        );
        let id = h
            .screen
            .layers()
            .add_content(0, Rect::new(0, 0, SW, SH), true);
        // **Overlined, which is the whole fixture**: it is the one bit tmux's entry masks, so an ink
        // without it makes the two arms identical and the gate vacuous.
        let ink = Style::new().overline();
        let paint = |h: &mut Harness, top: u32| {
            let mut v = h
                .screen
                .layers()
                .view(id)
                .expect("the layer is still there");
            for y in 0..SH {
                v.fill(Rect::new(0, y as i32, SW, 1), " ", ink);
                v.text(0, y as i32, &format!("row {}", top + u32::from(y)), ink);
            }
        };
        paint(&mut h, 0);
        h.present();
        for t in 1..=3 {
            paint(&mut h, t);
            h.present();
        }
        h.scrolls()
    }

    let reference = scrolls("ghostty 1.3.1");
    assert_eq!(
        reference,
        (3, 3),
        "the fixture stopped scrolling at all, so the arm below compares nothing"
    );
    assert_eq!(
        scrolls("tmux 3.7c"),
        reference,
        "tmux did not take the scroll a terminal with no quirk entry took — the pre-pass is \
         comparing what the application asked for against what the terminal was sent"
    );
}
