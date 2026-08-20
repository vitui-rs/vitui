//! A miniature terminal that parses the serializer's own bytes back into a grid.
//!
//! This is not test scaffolding, it is the primary instrument (spec §8, §14). A serializer is a
//! program whose output is only checkable by a program that reads it, and building that program is
//! a third of the work. All four defects the architecture map found were found by the round trip:
//! composite a frame, serialise it, replay the bytes here, assert the replayed screen equals the
//! frame. It stores nothing, so there is no file to review, nothing to bless and no maintenance —
//! and a golden byte string would have pinned the encoding, which is exactly the part that is
//! allowed to change.
//!
//! # The parsed set
//!
//! Written down rather than discovered, so tickets 13-15 extend a list rather than guess one.
//!
//! | sequence            | since      |
//! |---------------------|------------|
//! | `CUP` — `CSI y;x H` | ticket 03  |
//! | `SGR` — `CSI … m`   | ticket 03  |
//! | printable scalars   | ticket 03  |
//! | grapheme clusters, and the cell pair a wide one occupies | ticket 06 |
//!
//! Not parsed yet, each with the ticket that adds it: `CHA`, `CUF`, `CR`, `LF` (13); `DECSTBM`,
//! `SU`, `SD` (15); `DECAWM` and the pending-wrap state (15); `SGR 58`/`59` and `OSC 8` (07);
//! mode 2026 (16). An unrecognised sequence is counted rather than ignored, so a serializer that
//! starts emitting one before this model understands it fails the round trip instead of passing it
//! silently.
//!
//! Auto-wrap is off for the lifetime of the alt screen, so this model does not wrap: a print in the
//! last column leaves the cursor in the last column.
//!
//! # Clusters, and where this model draws a boundary
//!
//! A terminal groups arriving code points into cells by UAX #29, exactly as the drawing verbs do —
//! so this model runs the **same** segmenter over each contiguous run of printable bytes, and
//! interns into the **same** table the engine drew through. Both halves matter: a model with its own
//! segmenter would agree with a wrong one, and a model with its own table would compare handles that
//! cannot be equal.
//!
//! A run of printable bytes ends at an escape or a control, and a feed ends at a frame boundary —
//! `write_frame` loops until the sink has taken everything, so the harness never hands this model
//! half a frame. Within a run, two cells emitted back to back arrive with nothing between them, and
//! whether that re-segments into one cluster is exactly what the serializer has to be careful about.

use crate::cell::{Cell, GraphemeId};
use crate::intern::Interner;
use crate::style::{Color, Style};
use crate::ucd;

/// A terminal, as far as the serializer's output can tell.
pub(crate) struct TermModel {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
    cursor: (u16, u16),
    style: Style,
    /// Bytes of a sequence that was cut in half by a partial write.
    buf: Vec<u8>,
    /// Sequences this model does not understand. The round trip asserts it is zero.
    unrecognised: usize,
}

impl TermModel {
    pub(crate) fn new(width: u16, height: u16) -> TermModel {
        TermModel {
            width,
            height,
            cells: vec![Cell::BLANK; width as usize * height as usize],
            cursor: (0, 0),
            style: Style::DEFAULT,
            buf: Vec::new(),
            unrecognised: 0,
        }
    }

    pub(crate) fn cell(&self, x: u16, y: u16) -> Cell {
        self.cells[y as usize * self.width as usize + x as usize]
    }

    pub(crate) fn unrecognised(&self) -> usize {
        self.unrecognised
    }

    /// Feed bytes. Any number of bytes, split anywhere — a sequence cut in half is held until the
    /// rest arrives, because the partial-write loop is one of the things this model is here to
    /// check.
    pub(crate) fn feed(&mut self, bytes: &[u8], interner: &mut Interner) {
        // Taken out so the segmenter can borrow it while the cells are being written into.
        let mut buf = std::mem::take(&mut self.buf);
        buf.extend_from_slice(bytes);
        let mut at = 0usize;
        while at < buf.len() {
            let b = buf[at];
            if b == 0x1b {
                match self.parse_escape(&buf, at) {
                    Some(next) => at = next,
                    // Incomplete: keep what is left and wait for the rest.
                    None => break,
                }
            } else if b < 0x20 || b == 0x7F {
                // No control character is emitted by this serializer.
                self.unrecognised += 1;
                at += 1;
            } else {
                match printable_run(&buf[at..]) {
                    // Nothing complete yet: the run starts with a code point cut in half.
                    Run::Text(0) => break,
                    Run::Text(len) => {
                        let text = std::str::from_utf8(&buf[at..at + len])
                            .expect("printable_run stops at the first byte that is not UTF-8");
                        for cluster in ucd::clusters(text) {
                            self.print(cluster, interner);
                        }
                        at += len;
                    }
                    Run::Invalid => {
                        self.unrecognised += 1;
                        at += 1;
                    }
                }
            }
        }
        buf.drain(..at);
        self.buf = buf;
    }

    /// Returns the index just past the sequence, or `None` when it is not all here yet.
    fn parse_escape(&mut self, buf: &[u8], at: usize) -> Option<usize> {
        if at + 1 >= buf.len() {
            return None;
        }
        if buf[at + 1] != b'[' {
            self.unrecognised += 1;
            return Some(at + 2);
        }
        let mut end = at + 2;
        while end < buf.len() && !(0x40..=0x7E).contains(&buf[end]) {
            end += 1;
        }
        if end >= buf.len() {
            return None;
        }
        let final_byte = buf[end];
        let params = &buf[at + 2..end];
        match final_byte {
            b'H' => self.cup(params),
            b'm' => self.sgr(params),
            _ => self.unrecognised += 1,
        }
        Some(end + 1)
    }

    fn cup(&mut self, params: &[u8]) {
        let p = split_params(params);
        let y = p.first().and_then(|v| v.first().copied()).unwrap_or(1);
        let x = p.get(1).and_then(|v| v.first().copied()).unwrap_or(1);
        self.cursor = (
            (x.max(1) - 1).min(self.width.saturating_sub(1) as u32) as u16,
            (y.max(1) - 1).min(self.height.saturating_sub(1) as u32) as u16,
        );
    }

    fn sgr(&mut self, params: &[u8]) {
        let parsed = split_params(params);
        if parsed.is_empty() {
            self.style = Style::DEFAULT;
            return;
        }
        let mut i = 0;
        while i < parsed.len() {
            let p = &parsed[i];
            let head = p.first().copied().unwrap_or(0);
            match head {
                0 => self.style = Style::DEFAULT,
                1 => self.style = self.style.bold(),
                2 => self.style = self.style.dim(),
                3 => self.style = self.style.italic(),
                4 => {
                    self.style = match p.get(1).copied().unwrap_or(1) {
                        0 => self.style.no_underline(),
                        1 => self.style.underline(),
                        2 => self.style.underline_double(),
                        3 => self.style.underline_curly(),
                        4 => self.style.underline_dotted(),
                        5 => self.style.underline_dashed(),
                        _ => {
                            self.unrecognised += 1;
                            self.style
                        }
                    }
                }
                5 => self.style = self.style.blink(),
                7 => self.style = self.style.reverse(),
                8 => self.style = self.style.conceal(),
                9 => self.style = self.style.strikethrough(),
                22 => self.style = clear(self.style, crate::style::BOLD | crate::style::DIM),
                23 => self.style = clear(self.style, crate::style::ITALIC),
                24 => self.style = self.style.no_underline(),
                25 => self.style = clear(self.style, crate::style::BLINK),
                27 => self.style = clear(self.style, crate::style::REVERSE),
                28 => self.style = clear(self.style, crate::style::CONCEAL),
                29 => self.style = clear(self.style, crate::style::STRIKETHROUGH),
                53 => self.style = self.style.overline(),
                55 => self.style = clear(self.style, crate::style::OVERLINE),
                30..=37 => self.style = self.style.fg(Color::indexed((head - 30) as u8)),
                39 => self.style = self.style.fg(Color::DEFAULT),
                40..=47 => self.style = self.style.bg(Color::indexed((head - 40) as u8)),
                49 => self.style = self.style.bg(Color::DEFAULT),
                90..=97 => self.style = self.style.fg(Color::indexed((head - 90 + 8) as u8)),
                100..=107 => self.style = self.style.bg(Color::indexed((head - 100 + 8) as u8)),
                38 | 48 => {
                    let foreground = head == 38;
                    let kind = parsed.get(i + 1).and_then(|v| v.first().copied());
                    let (colour, used) = match kind {
                        Some(5) => (
                            parsed
                                .get(i + 2)
                                .and_then(|v| v.first().copied())
                                .map(|n| Color::indexed(n as u8)),
                            3,
                        ),
                        Some(2) => {
                            let ch = |k: usize| {
                                parsed
                                    .get(i + k)
                                    .and_then(|v| v.first().copied())
                                    .map(|n| n as u8)
                            };
                            (
                                match (ch(2), ch(3), ch(4)) {
                                    (Some(r), Some(g), Some(b)) => Some(Color::rgb(r, g, b)),
                                    _ => None,
                                },
                                5,
                            )
                        }
                        _ => (None, 1),
                    };
                    match colour {
                        Some(c) => {
                            self.style = if foreground {
                                self.style.fg(c)
                            } else {
                                self.style.bg(c)
                            }
                        }
                        None => self.unrecognised += 1,
                    }
                    i += used;
                    continue;
                }
                _ => self.unrecognised += 1,
            }
            i += 1;
        }
    }

    /// Put one cluster in the cell under the cursor, and the pair a wide one occupies.
    ///
    /// A cluster occupying no column is dropped rather than attached to the cell on its left. That
    /// is a simplification of what a terminal does and it is safe here for one reason: the drawing
    /// verbs never put one in a cell, so the serializer never emits one — and if it ever did, the
    /// round trip would say so rather than silently agreeing.
    fn print(&mut self, cluster: &str, interner: &mut Interner) {
        if self.width == 0 || self.height == 0 {
            return;
        }
        let Some(g) = interner.handle(cluster) else {
            return;
        };
        let (x, y) = self.cursor;
        let row = y as usize * self.width as usize;
        let columns = g.columns();
        if columns == 2 && x + 1 >= self.width {
            // Half a glyph in the last column. Our serializer writes a space there instead (rule 4),
            // so reaching this is a defect — and it is counted rather than papered over.
            self.unrecognised += 1;
            return;
        }
        self.cells[row + x as usize] = Cell::new(g, self.style);
        if columns == 2 {
            self.cells[row + x as usize + 1] = Cell::new(GraphemeId::CONTINUATION, self.style);
        }
        // Auto-wrap is off, so the cursor stops rather than wrapping.
        if x + columns < self.width {
            self.cursor = (x + columns, y);
        } else {
            self.cursor = (self.width - 1, y);
        }
    }
}

fn clear(s: Style, bits: u64) -> Style {
    Style::from_bits(s.bits() & !bits)
}

/// `1;2:3` becomes `[[1], [2, 3]]`.
fn split_params(bytes: &[u8]) -> Vec<Vec<u32>> {
    if bytes.is_empty() {
        return Vec::new();
    }
    bytes
        .split(|b| *b == b';')
        .map(|part| {
            part.split(|b| *b == b':')
                .map(|n| {
                    n.iter()
                        .filter(|b| b.is_ascii_digit())
                        .fold(0u32, |acc, b| acc.saturating_mul(10) + (b - b'0') as u32)
                })
                .collect()
        })
        .collect()
}

/// How far a run of printable bytes reaches.
enum Run {
    /// This many bytes are complete, printable and valid UTF-8. Zero means the very first code
    /// point is cut in half by the end of the buffer.
    Text(usize),
    /// The first byte is not the start of a valid code point.
    Invalid,
}

/// The length of the printable, complete UTF-8 prefix of `bytes`.
///
/// Stops at a control byte, an escape, an incomplete code point or an invalid one — so the caller
/// can hand the prefix to the segmenter as a `&str` and keep the rest for the next feed.
fn printable_run(bytes: &[u8]) -> Run {
    let mut at = 0usize;
    while at < bytes.len() {
        let b = bytes[at];
        if b == 0x1b || b < 0x20 || b == 0x7F {
            break;
        }
        match decode_utf8(&bytes[at..]) {
            Decoded::Char(len) => at += len,
            Decoded::Incomplete => break,
            Decoded::Invalid => {
                if at == 0 {
                    return Run::Invalid;
                }
                break;
            }
        }
    }
    Run::Text(at)
}

enum Decoded {
    /// A complete code point, this many bytes long. Which code point it is stopped mattering when
    /// the segmenter took over: `printable_run` hands the whole run to it as a `&str`.
    Char(usize),
    Incomplete,
    Invalid,
}

fn decode_utf8(bytes: &[u8]) -> Decoded {
    let len = match bytes[0] {
        0x00..=0x7F => 1,
        0xC2..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF4 => 4,
        _ => return Decoded::Invalid,
    };
    if bytes.len() < len {
        return Decoded::Incomplete;
    }
    match std::str::from_utf8(&bytes[..len]) {
        Ok(_) => Decoded::Char(len),
        Err(_) => Decoded::Invalid,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A table per test. Every scene here is scalars, and a scalar handle is identity in any table.
    fn feed(t: &mut TermModel, i: &mut Interner, bytes: &[u8]) {
        t.feed(bytes, i);
    }

    fn glyphs(t: &TermModel, y: u16) -> String {
        (0..t.width)
            .map(|x| t.cell(x, y).grapheme.as_scalar().unwrap_or('?'))
            .collect()
    }

    #[test]
    fn printable_bytes_land_at_the_cursor() {
        let mut t = TermModel::new(8, 2);
        let mut i = Interner::new();
        feed(&mut t, &mut i, b"abc");
        assert_eq!(glyphs(&t, 0), "abc     ");
    }

    #[test]
    fn cup_is_one_based() {
        let mut t = TermModel::new(8, 2);
        let mut i = Interner::new();
        feed(&mut t, &mut i, b"\x1b[2;3Hx");
        assert_eq!(glyphs(&t, 1), "  x     ");
    }

    #[test]
    fn sgr_applies_to_what_is_printed_after_it() {
        let mut t = TermModel::new(4, 1);
        let mut i = Interner::new();
        feed(&mut t, &mut i, b"\x1b[1ma\x1b[22mb");
        assert_eq!(t.cell(0, 0).style, Style::new().bold());
        assert_eq!(t.cell(1, 0).style, Style::DEFAULT);
    }

    #[test]
    fn sgr_zero_resets_everything() {
        let mut t = TermModel::new(4, 1);
        let mut i = Interner::new();
        feed(&mut t, &mut i, b"\x1b[1;4:3;38;2;1;2;3m\x1b[0mx");
        assert_eq!(t.cell(0, 0).style, Style::DEFAULT);
    }

    #[test]
    fn an_empty_sgr_is_a_reset() {
        let mut t = TermModel::new(4, 1);
        let mut i = Interner::new();
        feed(&mut t, &mut i, b"\x1b[1m\x1b[mx");
        assert_eq!(t.cell(0, 0).style, Style::DEFAULT);
    }

    #[test]
    fn every_colour_form_round_trips() {
        for (bytes, expected) in [
            (&b"\x1b[33mx"[..], Style::new().fg(Color::indexed(3))),
            (&b"\x1b[91mx"[..], Style::new().fg(Color::indexed(9))),
            (&b"\x1b[41mx"[..], Style::new().bg(Color::indexed(1))),
            (&b"\x1b[107mx"[..], Style::new().bg(Color::indexed(15))),
            (
                &b"\x1b[38;5;200mx"[..],
                Style::new().fg(Color::indexed(200)),
            ),
            (
                &b"\x1b[48;2;1;2;255mx"[..],
                Style::new().bg(Color::rgb(1, 2, 255)),
            ),
        ] {
            let mut t = TermModel::new(4, 1);
            let mut i = Interner::new();
            feed(&mut t, &mut i, bytes);
            assert_eq!(t.cell(0, 0).style, expected);
            assert_eq!(t.unrecognised(), 0);
        }
    }

    #[test]
    fn a_sequence_split_across_two_feeds_is_reassembled() {
        let mut t = TermModel::new(8, 2);
        let mut i = Interner::new();
        feed(&mut t, &mut i, b"\x1b[2");
        feed(&mut t, &mut i, b";3Hx");
        assert_eq!(glyphs(&t, 1), "  x     ");
        assert_eq!(t.unrecognised(), 0);
    }

    #[test]
    fn a_multi_byte_scalar_split_across_two_feeds_is_reassembled() {
        let mut t = TermModel::new(4, 1);
        let mut i = Interner::new();
        let bytes = "\u{e9}".as_bytes();
        feed(&mut t, &mut i, &bytes[..1]);
        feed(&mut t, &mut i, &bytes[1..]);
        assert_eq!(t.cell(0, 0).grapheme.as_scalar(), Some('\u{e9}'));
    }

    #[test]
    fn the_cursor_stops_at_the_last_column_because_auto_wrap_is_off() {
        let mut t = TermModel::new(3, 2);
        let mut i = Interner::new();
        feed(&mut t, &mut i, b"abcd");
        assert_eq!(glyphs(&t, 0), "abd");
        assert_eq!(glyphs(&t, 1), "   ", "nothing wrapped onto the next row");
    }

    #[test]
    fn an_unknown_sequence_is_counted_rather_than_ignored() {
        let mut t = TermModel::new(4, 1);
        let mut i = Interner::new();
        feed(&mut t, &mut i, b"\x1b[3J");
        assert_eq!(t.unrecognised(), 1);
    }

    #[test]
    fn a_control_character_is_counted() {
        let mut t = TermModel::new(4, 1);
        let mut i = Interner::new();
        feed(&mut t, &mut i, b"\r\n");
        assert_eq!(
            t.unrecognised(),
            2,
            "ticket 13 is what makes CR and LF legal"
        );
    }
}
