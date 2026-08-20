//! Bytes on the wire: the mirror, the emit loop, and the differential SGR.
//!
//! # Scope
//!
//! The tracer bullet's emit loop: `CUP`-only moves, a differential SGR, one grapheme per cell, and
//! the mirror updated as bytes go out. The `shortest` cursor encoding is ticket 13, the equality
//! filter ticket 14, the scroll region ticket 15, and synchronised output arrives with the
//! capability that gates it at ticket 16.
//!
//! # The frame's framing
//!
//! Every frame that has anything to say opens with SGR 0. That is tcell's "make no style
//! assumptions", four bytes, and it is what lets the emit loop start from a style it knows rather
//! than from one it inherited.

use crate::cell::{Cell, GraphemeId};
use crate::packet::Packet;
use crate::style::{Color, Style, TAG_DEFAULT, TAG_INDEXED, TAG_RGB};

/// What the terminal is currently showing.
///
/// Distinct from a frame, which is what the application *wants* shown. The mirror is what lets the
/// serializer skip a cell the frame rewrote without changing, and what lets a scroll be proved
/// before it is emitted (ADR 0006). It holds no application state and no handle, which is why it
/// does not reopen "the render thread holds no screen-sized state".
///
/// Ticket 14 is what reads it — the equality filter is the first consumer. Here it is written and
/// asserted against the terminal model, which is the gate that says the two agree at all.
#[derive(Clone, Debug)]
pub(crate) struct Mirror {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
}

impl Mirror {
    pub(crate) fn new(width: u16, height: u16) -> Mirror {
        Mirror {
            width,
            height,
            cells: vec![Cell::BLANK; width as usize * height as usize],
        }
    }

    #[cfg(test)]
    pub(crate) fn cell(&self, x: u16, y: u16) -> Cell {
        self.cells[y as usize * self.width as usize + x as usize]
    }

    fn set(&mut self, x: u16, y: u16, c: Cell) {
        self.cells[y as usize * self.width as usize + x as usize] = c;
    }
}

/// Turns a packet into the bytes that go to the terminal.
pub(crate) struct Serializer {
    out: Vec<u8>,
    mirror: Mirror,
    /// Where the terminal's cursor is, or `None` before the first move of a frame. With auto-wrap
    /// off the cursor stops in the last column rather than advancing off it, and that is what is
    /// recorded — see [`Serializer::advance`].
    cursor: Option<(u16, u16)>,
    style: Style,
}

impl Serializer {
    pub(crate) fn new(width: u16, height: u16) -> Serializer {
        Serializer {
            // A realistic full-screen frame is 24 430 bytes (spec §8). Reserving it up front is
            // what keeps the steady state free of a growth reallocation.
            out: Vec::with_capacity(32 * 1024),
            mirror: Mirror::new(width, height),
            cursor: None,
            style: Style::DEFAULT,
        }
    }

    #[cfg(test)]
    pub(crate) fn mirror(&self) -> &Mirror {
        &self.mirror
    }

    /// Serialise a packet. The returned slice is valid until the next call.
    pub(crate) fn serialize(&mut self, packet: &Packet) -> &[u8] {
        self.out.clear();
        if packet.is_empty() {
            return &self.out;
        }
        // A packet carries its own size so that a stale one can be refused on its own (spec §2).
        // Nothing can produce a stale one until ticket 22 brings the resize that makes sizes move,
        // so this is the debug-only assertion that shape of invariant gets rather than a runtime
        // check on the frame path.
        debug_assert_eq!(
            packet.size(),
            (self.mirror.width, self.mirror.height),
            "a packet packed at one size is being written into a mirror of another"
        );

        self.out.extend_from_slice(b"\x1b[0m");
        self.style = Style::DEFAULT;
        self.cursor = None;

        let mut at = 0usize;
        for run in packet.runs() {
            let cells = &packet.cells()[at..at + run.len()];
            at += run.len();
            for (i, &cell) in cells.iter().enumerate() {
                // The head of a wide pair already painted both columns, so its continuation is
                // skipped. Nothing in the engine mints one yet — ticket 06 is what writes a wide
                // glyph, and it is also what teaches the cursor to advance two columns over one —
                // but the skip is here because it is free: the column below comes from the index
                // rather than from an advance counter, which is what spec §8 records the first
                // version getting wrong in the one case where a run *begins* on a continuation.
                if cell.grapheme.is_continuation() {
                    continue;
                }
                let x = run.lo + i as u16;
                self.move_to(x, run.y);
                if cell.style != self.style {
                    emit_sgr_delta(&mut self.out, self.style, cell.style);
                    self.style = cell.style;
                }
                emit_grapheme(&mut self.out, cell.grapheme);
                self.mirror.set(x, run.y, cell);
                self.advance(x, run.y);
            }
        }
        &self.out
    }

    /// Where the terminal's cursor ends up after printing one cell at `(x, y)`.
    ///
    /// With auto-wrap off it stops in the last column instead of advancing off it, so that is what
    /// is recorded rather than a position one past the end. Nothing in this ticket can tell the two
    /// apart — a mutation swapping them leaves every test green, because runs are disjoint and
    /// ascending, so no later move on the same row ever targets a column already written. It starts
    /// mattering at ticket 13, where a relative `CUF` is priced against an absolute `CUP` and a
    /// cursor model that is off by one compounds along the row.
    fn advance(&mut self, x: u16, y: u16) {
        self.cursor = Some(((x + 1).min(self.mirror.width.saturating_sub(1)), y));
    }

    fn move_to(&mut self, x: u16, y: u16) {
        if self.cursor == Some((x, y)) {
            return;
        }
        self.out.extend_from_slice(b"\x1b[");
        push_num(&mut self.out, y as u32 + 1);
        self.out.push(b';');
        push_num(&mut self.out, x as u32 + 1);
        self.out.push(b'H');
        self.cursor = Some((x, y));
    }
}

fn emit_grapheme(out: &mut Vec<u8>, g: GraphemeId) {
    // Ticket 06 is what makes this a lookup in the cluster arena for anything above a scalar.
    let ch = g.as_scalar().unwrap_or(' ');
    let mut buf = [0u8; 4];
    out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
}

/// Write the SGR that turns `old` into `new`.
///
/// A differential SGR is worth it and costs nothing to decide, because the decision is one style
/// compare and the decomposition is off the hot path by construction: a realistic full-screen frame
/// emits **one** SGR sequence for 24 000 cells.
fn emit_sgr_delta(out: &mut Vec<u8>, old: Style, new: Style) {
    let mark = out.len();
    out.extend_from_slice(b"\x1b[");
    let mut params = 0u32;

    let removed = old.attrs() & !new.attrs();
    let added = new.attrs() & !old.attrs();

    // SGR 22 resets bold *and* dim together. There is no un-bold that leaves dim standing, so
    // removing either means emitting 22 and reapplying the other. A naive per-attribute on/off loop
    // silently drops dim whenever bold turns off (spec §8).
    if removed & (crate::style::BOLD | crate::style::DIM) != 0 {
        param(out, &mut params, 22);
        if new.attrs() & crate::style::BOLD != 0 {
            param(out, &mut params, 1);
        }
        if new.attrs() & crate::style::DIM != 0 {
            param(out, &mut params, 2);
        }
    } else {
        if added & crate::style::BOLD != 0 {
            param(out, &mut params, 1);
        }
        if added & crate::style::DIM != 0 {
            param(out, &mut params, 2);
        }
    }

    for (bit, on, off) in [
        (crate::style::ITALIC, 3, 23),
        (crate::style::BLINK, 5, 25),
        (crate::style::REVERSE, 7, 27),
        (crate::style::CONCEAL, 8, 28),
        (crate::style::STRIKETHROUGH, 9, 29),
        (crate::style::OVERLINE, 53, 55),
    ] {
        if added & bit != 0 {
            param(out, &mut params, on);
        } else if removed & bit != 0 {
            param(out, &mut params, off);
        }
    }

    if old.underline_style() != new.underline_style() {
        match new.underline_style() {
            0 => param(out, &mut params, 24),
            // SGR 21 is never emitted: ECMA-48 assigns it "doubly underlined" and a meaningful
            // population of terminals implements it as "bold off". Double underline is `4:2`.
            n => {
                param(out, &mut params, 4);
                out.push(b':');
                push_num(out, n as u32);
            }
        }
    }

    if old.foreground() != new.foreground() {
        emit_color(out, &mut params, new.foreground(), true);
    }
    if old.background() != new.background() {
        emit_color(out, &mut params, new.background(), false);
    }

    if params == 0 {
        // Nothing to say. An empty `CSI m` is `CSI 0 m`, a reset, which is emphatically not
        // nothing — so the opener is taken back instead.
        out.truncate(mark);
        return;
    }
    out.push(b'm');
}

/// Colour emission, spec §8: default is 39/49; indices under 16 use 30-37 / 90-97 and their
/// background forms rather than `38;5;n`; truecolor is `38;2;r;g;b`.
fn emit_color(out: &mut Vec<u8>, params: &mut u32, c: Color, foreground: bool) {
    let base = if foreground { 30 } else { 40 };
    match c.tag() {
        TAG_DEFAULT => param(out, params, base + 9),
        TAG_INDEXED => {
            let i = c.payload();
            if i < 8 {
                param(out, params, base + i);
            } else if i < 16 {
                param(out, params, base + 60 + (i - 8));
            } else {
                param(out, params, base + 8);
                out.push(b';');
                push_num(out, 5);
                out.push(b';');
                push_num(out, i);
            }
        }
        TAG_RGB => {
            let v = c.payload();
            param(out, params, base + 8);
            out.push(b';');
            push_num(out, 2);
            for shift in [16, 8, 0] {
                out.push(b';');
                push_num(out, (v >> shift) & 0xFF);
            }
        }
        // `tag()` returns a `u32`, so the match has to be total. Tag 3 is reserved, `Color` has no
        // constructor that produces it, and this arm exists to satisfy the compiler rather than to
        // handle a case — the safe total answer being the terminal's own colour.
        _ => param(out, params, base + 9),
    }
}

fn param(out: &mut Vec<u8>, params: &mut u32, n: u32) {
    if *params > 0 {
        out.push(b';');
    }
    *params += 1;
    push_num(out, n);
}

/// Decimal, into the buffer, with no formatting machinery and no allocation.
fn push_num(out: &mut Vec<u8>, n: u32) {
    if n == 0 {
        out.push(b'0');
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = buf.len();
    let mut n = n;
    while n > 0 {
        i -= 1;
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
    }
    out.extend_from_slice(&buf[i..]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::GraphemeId;
    use crate::geom::Rect;
    use crate::style::Style;
    use crate::surface::Surface;
    use crate::term_model::TermModel;

    fn bytes_for(frame: &Surface) -> Vec<u8> {
        let mut runs = Vec::new();
        frame.damage().for_each_run(|r| runs.push(r));
        let mut packet = Packet::new();
        packet.pack(&runs, frame);
        let (w, h) = frame.size();
        let mut s = Serializer::new(w, h);
        s.serialize(&packet).to_vec()
    }

    fn text(s: &[u8]) -> String {
        String::from_utf8_lossy(s).replace('\x1b', "ESC")
    }

    /// Replay the bytes and read the screen back. The round trip is the instrument for anything
    /// about *where a glyph lands*; a byte string would pin the encoding, which is the part
    /// tickets 13 to 15 are allowed to change.
    fn replay(frame: &Surface) -> TermModel {
        let (w, h) = frame.size();
        let mut term = TermModel::new(w, h);
        term.feed(&bytes_for(frame));
        assert_eq!(term.unrecognised(), 0);
        term
    }

    fn glyphs(term: &TermModel, y: u16, w: u16) -> String {
        (0..w)
            .map(|x| term.cell(x, y).grapheme.as_scalar().unwrap_or('?'))
            .collect()
    }

    /// How many cursor-positioning sequences the frame spent. Counting them is a property; naming
    /// the bytes they are spelled with is not.
    fn moves(out: &[u8]) -> usize {
        csi_finals(out)
            .filter(|b| matches!(b, b'H' | b'G' | b'C' | b'd' | b'f'))
            .count()
    }

    /// How many SGR sequences the frame spent, not counting the reset it opens with.
    fn style_changes(out: &[u8]) -> usize {
        csi_finals(out).filter(|b| *b == b'm').count() - 1
    }

    fn csi_finals(out: &[u8]) -> impl Iterator<Item = u8> {
        let mut in_csi = false;
        out.iter().enumerate().filter_map(move |(i, b)| {
            if *b == 0x1b {
                in_csi = false;
                return None;
            }
            if i > 0 && out[i - 1] == 0x1b && *b == b'[' {
                in_csi = true;
                return None;
            }
            if in_csi && (0x40..=0x7E).contains(b) {
                in_csi = false;
                return Some(*b);
            }
            None
        })
    }

    #[test]
    fn an_empty_packet_emits_nothing() {
        assert!(bytes_for(&Surface::new(8, 2)).is_empty());
    }

    #[test]
    fn a_frame_opens_with_an_sgr_reset() {
        let mut f = Surface::new(8, 2);
        f.root().text(0, 0, "a", Style::new());
        assert!(bytes_for(&f).starts_with(b"\x1b[0m"));
    }

    #[test]
    fn a_run_costs_one_cursor_move() {
        let mut f = Surface::new(8, 2);
        f.root().text(2, 1, "abc", Style::new());
        assert_eq!(moves(&bytes_for(&f)), 1);
        assert_eq!(glyphs(&replay(&f), 1, 8), "  abc   ");
    }

    #[test]
    fn two_runs_on_one_row_cost_two_moves() {
        let mut f = Surface::new(20, 1);
        f.root().text(0, 0, "ab", Style::new());
        f.root().text(10, 0, "cd", Style::new());
        assert_eq!(moves(&bytes_for(&f)), 2);
        assert_eq!(glyphs(&replay(&f), 0, 20), "ab        cd        ");
    }

    #[test]
    fn a_style_change_inside_a_run_emits_one_sgr() {
        let mut f = Surface::new(8, 1);
        f.root().text(0, 0, "ab", Style::new());
        f.root().text(2, 0, "cd", Style::new().bold());
        assert_eq!(style_changes(&bytes_for(&f)), 1);
        assert_eq!(replay(&f).cell(2, 0).style, Style::new().bold());
        assert_eq!(replay(&f).cell(1, 0).style, Style::DEFAULT);
    }

    #[test]
    fn one_style_covers_a_whole_run() {
        let mut f = Surface::new(300, 1);
        f.root()
            .fill(Rect::new(0, 0, 300, 1), "x", Style::new().bold());
        assert_eq!(
            style_changes(&bytes_for(&f)),
            1,
            "one SGR covers a run, however many cells it has"
        );
    }

    #[test]
    fn removing_bold_reapplies_dim() {
        // SGR 22 clears both, so the naive per-attribute loop drops dim here and nothing looks
        // wrong until a dim run silently turns normal.
        let mut out = Vec::new();
        emit_sgr_delta(&mut out, Style::new().bold().dim(), Style::new().dim());
        assert_eq!(text(&out), "ESC[22;2m");
    }

    #[test]
    fn removing_dim_reapplies_bold() {
        let mut out = Vec::new();
        emit_sgr_delta(&mut out, Style::new().bold().dim(), Style::new().bold());
        assert_eq!(text(&out), "ESC[22;1m");
    }

    #[test]
    fn adding_bold_alone_is_one_parameter() {
        let mut out = Vec::new();
        emit_sgr_delta(&mut out, Style::new(), Style::new().bold());
        assert_eq!(text(&out), "ESC[1m");
    }

    #[test]
    fn sgr_21_is_never_emitted_for_a_double_underline() {
        let mut out = Vec::new();
        emit_sgr_delta(&mut out, Style::new(), Style::new().underline_double());
        assert_eq!(text(&out), "ESC[4:2m");
    }

    #[test]
    fn dropping_an_underline_emits_24() {
        let mut out = Vec::new();
        emit_sgr_delta(&mut out, Style::new().underline_curly(), Style::new());
        assert_eq!(text(&out), "ESC[24m");
    }

    #[test]
    fn the_low_sixteen_palette_entries_are_spelled_short() {
        let mut out = Vec::new();
        emit_sgr_delta(&mut out, Style::new(), Style::new().fg(Color::indexed(3)));
        assert_eq!(text(&out), "ESC[33m");
        out.clear();
        emit_sgr_delta(&mut out, Style::new(), Style::new().fg(Color::indexed(9)));
        assert_eq!(text(&out), "ESC[91m");
        out.clear();
        emit_sgr_delta(&mut out, Style::new(), Style::new().bg(Color::indexed(1)));
        assert_eq!(text(&out), "ESC[41m");
        out.clear();
        emit_sgr_delta(&mut out, Style::new(), Style::new().bg(Color::indexed(15)));
        assert_eq!(text(&out), "ESC[107m");
    }

    #[test]
    fn a_palette_entry_above_fifteen_needs_the_long_form() {
        let mut out = Vec::new();
        emit_sgr_delta(&mut out, Style::new(), Style::new().fg(Color::indexed(200)));
        assert_eq!(text(&out), "ESC[38;5;200m");
    }

    #[test]
    fn truecolor_is_three_channels() {
        let mut out = Vec::new();
        emit_sgr_delta(
            &mut out,
            Style::new(),
            Style::new().bg(Color::rgb(1, 2, 255)),
        );
        assert_eq!(text(&out), "ESC[48;2;1;2;255m");
    }

    #[test]
    fn returning_to_the_default_colour_emits_39_and_49() {
        let mut out = Vec::new();
        emit_sgr_delta(
            &mut out,
            Style::new().fg(Color::rgb(1, 2, 3)).bg(Color::indexed(4)),
            Style::new(),
        );
        assert_eq!(text(&out), "ESC[39;49m");
    }

    #[test]
    fn an_sgr_that_would_say_nothing_emits_nothing() {
        let mut out = Vec::new();
        emit_sgr_delta(&mut out, Style::new().bold(), Style::new().bold());
        assert!(out.is_empty(), "an empty CSI m is a reset, not a no-op");
    }

    #[test]
    fn the_mirror_records_what_was_emitted() {
        let mut f = Surface::new(8, 2);
        f.root().text(3, 1, "z", Style::new().bold());
        let mut runs = Vec::new();
        f.damage().for_each_run(|r| runs.push(r));
        let mut packet = Packet::new();
        packet.pack(&runs, &f);
        let mut s = Serializer::new(8, 2);
        s.serialize(&packet);
        assert_eq!(s.mirror().cell(3, 1), f.row(1)[3]);
        assert_eq!(s.mirror().cell(0, 0), Cell::BLANK, "nothing else moved");
    }

    #[test]
    fn a_continuation_cell_emits_no_glyph() {
        // Planted by hand, because no verb can write one until ticket 06. What is pinned is only
        // that the second half of a pair is not printed; the cursor advance over a wide head is
        // ticket 06's and is deliberately not modelled here.
        let mut f = Surface::new(6, 1);
        f.root().text(0, 0, "ab", Style::new());
        f.row_mut(0)[1] = Cell::new(GraphemeId::CONTINUATION, Style::DEFAULT);
        assert_eq!(
            glyphs(&replay(&f), 0, 6),
            "a     ",
            "the `b` was replaced and never printed"
        );
    }

    #[test]
    fn a_run_beginning_on_a_continuation_still_places_the_rest_correctly() {
        // The one case spec §8 records the first version getting wrong: the column comes from the
        // index, so skipping the first cell of a run must not shift what follows.
        let mut f = Surface::new(6, 1);
        f.root().text(2, 0, "ab", Style::new());
        f.row_mut(0)[2] = Cell::new(GraphemeId::CONTINUATION, Style::new().bold());
        let term = replay(&f);
        assert_eq!(
            glyphs(&term, 0, 6),
            "   b  ",
            "skipping the first cell of a run must not shift what follows"
        );
        assert_eq!(term.cell(2, 0), Cell::BLANK);
    }

    #[test]
    fn push_num_writes_decimal() {
        let mut out = Vec::new();
        for n in [0u32, 1, 9, 10, 255, 4294967295] {
            out.clear();
            push_num(&mut out, n);
            assert_eq!(text(&out), n.to_string());
        }
    }
}
