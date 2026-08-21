//! The round trip: the primary instrument, and it stores nothing.
//!
//! Composite a frame, serialise it, replay the bytes through the terminal model, assert the
//! replayed screen equals the frame that was composited — cell for cell, glyph and style. All four
//! defects the architecture map found were found this way (spec §14). There is no file to review,
//! nothing to bless and no maintenance, and a golden byte string would have pinned the encoding,
//! which is exactly the part that is allowed to change.
//!
//! Every scene ticket 03 can express is driven through here. Ticket 05 adds the golden frames,
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
    let mut h = Harness::new(40, 2);
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
    let mut h = Harness::new(300, 80);
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
    let mut h = Harness::new(300, 80);
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
    let mut h = Harness::new(40, 8);
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
    let mut h = Harness::new(40, 8);
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
    let mut h = Harness::new(40, 4);
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
    let mut h = Harness::new(300, 80);
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
    let mut h = Harness::new(300, 80);
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
    // Spec §8's honest partial-write test: the fragmentation is deterministic, because whether a
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
    let mut h = Harness::new(60, 10);
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
    let mut h = Harness::new(20, 3);
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
