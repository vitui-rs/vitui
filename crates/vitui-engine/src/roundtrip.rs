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

use std::sync::{Arc, Mutex};

use crate::engine::{Clock, Config, Engine, Output, Presented, Screen};
use crate::geom::Rect;
use crate::style::{Color, Style};
use crate::term_model::TermModel;
use crate::testing::{Recorder, Recording};

/// A screen, a sink, and the terminal model the sink's bytes are replayed through.
struct Harness {
    screen: Screen,
    recording: Arc<Mutex<Recording>>,
    term: TermModel,
    replayed: usize,
}

impl Harness {
    fn new(w: u16, h: u16) -> Harness {
        Harness::with_sink(w, h, Recorder::new())
    }

    fn with_sink(w: u16, h: u16, sink: Recorder) -> Harness {
        let recording = sink.handle();
        let (screen, _wake) = Engine::new(Config {
            size: (w, h),
            output: Output::Sink(Box::new(sink)),
            // The deterministic mode is public API, not a test fixture: `present` composites,
            // packs, serialises and writes inline on this thread, so a test is a straight-line
            // program with no condvar, no join, no timeout and no flake.
            clock: Clock::Manual,
        })
        .attach()
        .expect("attaching to a sink cannot fail");
        Harness {
            screen,
            recording,
            term: TermModel::new(w, h),
            replayed: 0,
        }
    }

    /// Present, replay whatever is new in the sink, and assert the three things that must agree.
    fn present(&mut self) -> Presented {
        let presented = self.screen.present();
        let fresh = {
            let r = self.recording.lock().unwrap();
            r.bytes[self.replayed..].to_vec()
        };
        self.replayed += fresh.len();
        self.term.feed(&fresh);

        assert_eq!(
            self.term.unrecognised(),
            0,
            "the serializer emitted a sequence the terminal model does not parse"
        );
        self.assert_screen_matches_frame();
        self.assert_mirror_matches_frame();
        presented
    }

    fn assert_screen_matches_frame(&self) {
        let (w, h) = self.screen.size();
        let frame = self.screen.frame();
        for y in 0..h {
            for x in 0..w {
                assert_eq!(
                    self.term.cell(x, y),
                    frame.row(y)[x as usize],
                    "the replayed screen and the composited frame disagree at ({x}, {y})"
                );
            }
        }
    }

    /// The mirror must agree with the frame everywhere.
    ///
    /// What this buys is the cells the frame *wrote*: a mirror that missed an update, or recorded
    /// the wrong style, fails here. What it does not buy is the cells nobody touched — a fresh
    /// mirror and a fresh frame are both blank, so those match by construction.
    ///
    /// The mirror has no *unknown row* yet, which
    /// `docs/adr/0006-the-render-thread-mirrors-what-the-terminal-shows.md` makes load-bearing at
    /// startup and after a resize. Ticket 14's equality filter is the first thing that compares
    /// against the mirror and therefore the first thing that can be wrong without one; ticket 22
    /// brings the resize that makes a row untrustworthy.
    fn assert_mirror_matches_frame(&self) {
        let (w, h) = self.screen.size();
        let frame = self.screen.frame();
        for y in 0..h {
            for x in 0..w {
                assert_eq!(
                    self.screen.mirror().cell(x, y),
                    frame.row(y)[x as usize],
                    "the mirror and the composited frame disagree at ({x}, {y})"
                );
            }
        }
    }

    fn bytes_written(&self) -> usize {
        self.recording.lock().unwrap().bytes.len()
    }

    fn writes(&self) -> usize {
        self.recording.lock().unwrap().writes
    }

    fn retries(&self) -> usize {
        self.recording.lock().unwrap().retries
    }
}

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
