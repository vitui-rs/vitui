//! Bytes on the wire: the mirror, the emit loop, and the differential SGR.
//!
//! # Scope
//!
//! The tracer bullet's emit loop: `CUP`-only moves, a differential SGR, one grapheme per cell, and
//! the mirror updated as bytes go out. The `shortest` cursor encoding is ticket 13, the equality
//! filter ticket 14, the scroll region ticket 15, and synchronised output arrives with the
//! capability that gates it at ticket 16.
//!
//! # An extended style's two new channels do not reach the wire yet
//!
//! Ticket 07 put the extended bit in the style word and `restyle` on the `View`. SGR
//! `58:2::r:g:b` / `59` and OSC 8 are spec §8's and belong to ticket 13, which is also what teaches
//! the terminal model to read them back — so **the round trip cannot close on an extended cell
//! until then**, and no gate here drives one.
//!
//! What does *not* wait is the two channels this loop already emits. An extended word spends bits
//! 51..0 on a handle, so [`colors_of`] resolves the colours through `packet.ext` instead of reading
//! the word — and reading that handle as a 26-bit foreground and a 26-bit background is exactly the
//! silent wrong answer `Style::with_fg_bg` was deleted for, one layer down. The packet already
//! carries both side tables; ticket 07 put them there rather than at ticket 18 precisely so this
//! loop would not have to wait for the compositor.
//!
//! So a hyperlinked cell presented today is painted in the right colours and loses its hyperlink.
//! That is degradation, which this engine has a model for; a handle emitted as a colour is not.
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
///
/// # The unknown row
///
/// A row of the mirror that cannot be trusted is **unknown**, and that is how a full repaint
/// expresses itself without a separate mode (ADR 0006, §8). Three things make a row unknown, and
/// only the third is new:
///
/// - **at startup**, because the mirror records what the terminal shows and nobody recorded that;
/// - **after a resize**, for the same reason — a terminal reflows on `SIGWINCH`, it does not clear;
/// - **after a sweep renumbered a handle table**, which is [`Packet::repaint`].
///
/// A row leaves the unknown state when one frame has written **every column of it**, because that is
/// the point at which every cell of the row was put there by this serializer.
///
/// # What an unknown row means for ticket 14's filter, in the one case §8 does not cover
///
/// ADR 0006's words are *written whole rather than compared*, and they are exactly right for the two
/// cases it names: at startup and after a resize every cell is damaged, so the packet carries the
/// whole row and *whole* is achievable. **After a sweep it is not** — the sweep marks no damage, so
/// the packet carries only what actually changed.
///
/// The property the filter has to keep is nevertheless the same one, and it survives the difference:
/// **on an unknown row, do not compare — emit every cell the packet carries.** That is safe for the
/// reason the comparison is unsafe. A stale mirror cell holds an old handle naming text that is
/// still on the screen; the danger is not the false *inequality* (which re-emits, and is merely
/// bytes) but the false *equality* — a later frame whose new handle happens to equal the recorded
/// old one, compared equal, skipped, and the terminal left showing the wrong text. Never skipping on
/// an unknown row makes that unreachable.
///
/// What it costs is bytes: rows that a sweep marked unknown stay unfiltered until some frame writes
/// one whole, and on a screen whose damage is always narrow that can be a long time. That is the
/// *one full frame on the render thread* spec §3 prices the sweep at, spread out. Ticket 14 owns the
/// filter and is where a tighter answer — per-cell rather than per-row knowledge — would be paid for
/// or refused.
#[derive(Clone, Debug)]
pub(crate) struct Mirror {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
    /// One flag per row: whether this serializer has written every column of it since the row was
    /// last invalidated. A `Vec<bool>` of eighty bytes rather than a bitset, because it is read once
    /// per row and never per cell.
    known: Vec<bool>,
}

impl Mirror {
    pub(crate) fn new(width: u16, height: u16) -> Mirror {
        Mirror {
            width,
            height,
            cells: vec![Cell::BLANK; width as usize * height as usize],
            // Unknown, not blank. The cells say blank because they have to say something, and the
            // flag is what stops anybody believing them.
            known: vec![false; height as usize],
        }
    }

    #[cfg(test)]
    pub(crate) fn cell(&self, x: u16, y: u16) -> Cell {
        self.cells[y as usize * self.width as usize + x as usize]
    }

    /// Whether row `y` records what the terminal is showing.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "ticket 14's equality filter is the first thing that branches on this; \
                      ticket 08 is what makes the state correct for it to read"
        )
    )]
    pub(crate) fn is_known(&self, y: u16) -> bool {
        self.known[y as usize]
    }

    /// Mark every row unknown: what [`Packet::repaint`] asks for.
    fn forget(&mut self) {
        self.known.fill(false);
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
    /// The cluster emitted last, when the next cell's bytes would follow it with nothing between.
    /// See [`Serializer::joins_left`].
    prev: Option<GraphemeId>,
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
            prev: None,
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

        // Before anything is emitted: a sweep renumbered a table, so every handle this mirror
        // recorded names an entry that has moved. Nothing about the *screen* changed — the cells
        // still say the same thing — which is exactly why there is no damage to go with it and a
        // flag is what carries it (spec §3).
        if packet.repaint() {
            self.mirror.forget();
        }

        self.out.extend_from_slice(b"\x1b[0m");
        self.style = Style::DEFAULT;
        self.cursor = None;
        self.prev = None;

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
                    // The head already painted both columns, so nothing is emitted — but the mirror
                    // still records the pair, because the mirror is what the terminal *shows* and
                    // the terminal shows both halves. A run that *begins* on a continuation is the
                    // case that made the column below come from the index rather than from an
                    // advance counter: its head was not damaged, so it was not re-emitted, and the
                    // mirror already agreed about it.
                    self.mirror.set(run.lo + i as u16, run.y, cell);
                    continue;
                }
                let x = run.lo + i as u16;
                // Two cells emitted back to back arrive with nothing between them, and UAX #29 does
                // not know where one cell ended. Forcing a move breaks the adjacency, and a `CUP` is
                // something every terminal has always treated as ending a run of text.
                //
                // An SGR landing between them would separate them too, and skipping the move when
                // the style changes was written and then taken back out: *whether a terminal's own
                // clustering survives an SGR is a claim about other people's software*, and the mode
                // 2027 specification does not make it. The saving was six bytes on a case that needs
                // two adjacent cells holding joinable clusters.
                if self.cursor == Some((x, run.y)) && self.joins_left(cell.grapheme, packet) {
                    self.cursor = None;
                }
                self.move_to(x, run.y);
                if cell.style != self.style {
                    emit_sgr_delta(&mut self.out, self.style, cell.style, packet);
                    self.style = cell.style;
                }
                emit_grapheme(&mut self.out, cell.grapheme, packet);
                self.mirror.set(x, run.y, cell);
                self.advance(x, run.y, cell.grapheme.columns());
                self.prev = Some(cell.grapheme);
            }
        }
        self.note_whole_rows(packet);
        &self.out
    }

    /// Mark known every row this frame wrote every column of.
    ///
    /// Runs are disjoint and arrive in row order (§14's gate #2), so a row's columns are the sum of
    /// its runs' lengths and consecutive runs with the same `y` are all of that row's. A row written
    /// in pieces across several frames stays unknown, which is conservative in the safe direction:
    /// unknown costs bytes and known costs correctness.
    fn note_whole_rows(&mut self, packet: &Packet) {
        let mut runs = packet.runs().iter().peekable();
        while let Some(first) = runs.next() {
            let mut columns = first.len();
            while let Some(next) = runs.peek().filter(|r| r.y == first.y) {
                columns += next.len();
                runs.next();
            }
            if columns == self.mirror.width as usize {
                self.mirror.known[first.y as usize] = true;
            }
        }
    }

    /// Where the terminal's cursor ends up after printing one cell at `(x, y)`.
    ///
    /// With auto-wrap off it stops in the last column instead of advancing off it, so that is what
    /// is recorded rather than a position one past the end. Nothing in this ticket can tell the two
    /// apart — a mutation swapping them leaves every test green, because runs are disjoint and
    /// ascending, so no later move on the same row ever targets a column already written. It starts
    /// mattering at ticket 13, where a relative `CUF` is priced against an absolute `CUP` and a
    /// cursor model that is off by one compounds along the row.
    fn advance(&mut self, x: u16, y: u16, columns: u16) {
        self.cursor = Some(((x + columns).min(self.mirror.width.saturating_sub(1)), y));
    }

    /// Whether the cluster about to be emitted would join the one before it into a single cluster.
    ///
    /// This is spec §10's rule — *a width disagreement is permanent once a mirror exists* — arriving
    /// one ticket early and for the neighbouring reason. §10 states it as `CHA` rather than `CUF`
    /// after a non-ASCII run, which is about the cursor *compounding* an error; this is about the
    /// bytes themselves re-segmenting. Both are the same underlying fact: **what the engine put in
    /// two cells is not what the terminal reads unless something separates them.**
    ///
    /// Found by the round trip, not by reading. A cluster ending in ZWJ followed by a pictograph
    /// (GB11) and a lone regional indicator followed by another (GB12/13) both merge, and the frame
    /// then holds two cells where the terminal shows one.
    ///
    /// The question is asked of the same tables the verbs segmented with, rather than of a list of
    /// rules copied out of UAX #29 — a second list is a second thing to keep in step. Two ASCII
    /// scalars can never join, which is the fast path and covers nearly every cell ever emitted.
    fn joins_left(&self, next: GraphemeId, packet: &Packet) -> bool {
        let Some(prev) = self.prev else {
            return false;
        };
        if is_ascii_scalar(prev) && is_ascii_scalar(next) {
            return false;
        }
        let (mut prev_buf, mut next_buf) = ([0u8; 4], [0u8; 4]);
        let Some(prev_text) = text_of(prev, packet, &mut prev_buf) else {
            return false;
        };
        let Some(next_text) = text_of(next, packet, &mut next_buf) else {
            return false;
        };
        let Some(first) = next_text.chars().next() else {
            return false;
        };
        let mut cursor = crate::ucd::Cursor::new();
        for ch in prev_text.chars() {
            cursor.is_break(ch);
        }
        !cursor.is_break(first)
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

/// Write one cell's cluster.
///
/// A scalar handle *is* its own bytes; anything above the scalars is a lookup in the packet's own
/// arena, which is where the app thread put a copy at pack time. **The engine table is never
/// reached from here** — that is the whole of ADR 0011's second sentence, and it is what lets the
/// app thread sweep its interner while a frame is being written.
fn emit_grapheme(out: &mut Vec<u8>, g: GraphemeId, packet: &Packet) {
    let mut buf = [0u8; 4];
    let cluster = text_of(g, packet, &mut buf).unwrap_or(" ");
    out.extend_from_slice(cluster.as_bytes());
}

/// The bytes a handle names: the packet's copy for a cluster, the scalar itself for a scalar.
fn text_of<'a>(g: GraphemeId, packet: &'a Packet, scratch: &'a mut [u8; 4]) -> Option<&'a str> {
    if let Some(cluster) = packet.cluster(g) {
        return Some(cluster);
    }
    Some(g.as_scalar()?.encode_utf8(scratch))
}

/// The cell that costs nothing to check: an ASCII scalar, which nothing can join to.
fn is_ascii_scalar(g: GraphemeId) -> bool {
    g.payload() < 0x80
}

/// Write the SGR that turns `old` into `new`.
///
/// A differential SGR is worth it and costs nothing to decide, because the decision is one style
/// compare and the decomposition is off the hot path by construction: a realistic full-screen frame
/// emits **one** SGR sequence for 24 000 cells.
fn emit_sgr_delta(out: &mut Vec<u8>, old: Style, new: Style, packet: &Packet) {
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

    let (old_fg, old_bg) = colors_of(old, packet);
    let (new_fg, new_bg) = colors_of(new, packet);
    if old_fg != new_fg {
        emit_color(out, &mut params, new_fg, true);
    }
    if old_bg != new_bg {
        emit_color(out, &mut params, new_bg, false);
    }

    if params == 0 {
        // Nothing to say. An empty `CSI m` is `CSI 0 m`, a reset, which is emphatically not
        // nothing — so the opener is taken back instead.
        out.truncate(mark);
        return;
    }
    out.push(b'm');
}

/// The two colours a style word paints with, whichever side of the extended bit it is on.
///
/// An extended word has no inline colours — bits 51..0 are the handle — so they come from the
/// packet's own copy of the table, which is where the app thread put them at pack time. The engine
/// table is never reached from here, and that is ADR 0011's second sentence again.
///
/// A handle the packet does not carry means `pack` and `serialize` disagree about which frame this
/// is, and there is nothing truthful to paint: the terminal's own colours are the one answer that
/// invents nothing.
fn colors_of(style: Style, packet: &Packet) -> (Color, Color) {
    let Some(handle) = style.ext_handle() else {
        return (style.foreground(), style.background());
    };
    match packet.ext(handle) {
        Some(e) => (e.fg, e.bg),
        None => (Color::DEFAULT, Color::DEFAULT),
    }
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
        packet.pack(&runs, frame, frame.tables(), false);
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
        // A table of its own is enough here: every scene in this module is scalars, and a scalar
        // handle is identity in any table. `Harness` is where a shared one is load-bearing.
        let mut interner = crate::intern::Interner::new();
        term.feed(&bytes_for(frame), &mut interner);
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
        emit_sgr_delta(
            &mut out,
            Style::new().bold().dim(),
            Style::new().dim(),
            &Packet::new(),
        );
        assert_eq!(text(&out), "ESC[22;2m");
    }

    #[test]
    fn removing_dim_reapplies_bold() {
        let mut out = Vec::new();
        emit_sgr_delta(
            &mut out,
            Style::new().bold().dim(),
            Style::new().bold(),
            &Packet::new(),
        );
        assert_eq!(text(&out), "ESC[22;1m");
    }

    #[test]
    fn adding_bold_alone_is_one_parameter() {
        let mut out = Vec::new();
        emit_sgr_delta(&mut out, Style::new(), Style::new().bold(), &Packet::new());
        assert_eq!(text(&out), "ESC[1m");
    }

    #[test]
    fn sgr_21_is_never_emitted_for_a_double_underline() {
        let mut out = Vec::new();
        emit_sgr_delta(
            &mut out,
            Style::new(),
            Style::new().underline_double(),
            &Packet::new(),
        );
        assert_eq!(text(&out), "ESC[4:2m");
    }

    #[test]
    fn dropping_an_underline_emits_24() {
        let mut out = Vec::new();
        emit_sgr_delta(
            &mut out,
            Style::new().underline_curly(),
            Style::new(),
            &Packet::new(),
        );
        assert_eq!(text(&out), "ESC[24m");
    }

    #[test]
    fn the_low_sixteen_palette_entries_are_spelled_short() {
        let mut out = Vec::new();
        emit_sgr_delta(
            &mut out,
            Style::new(),
            Style::new().fg(Color::indexed(3)),
            &Packet::new(),
        );
        assert_eq!(text(&out), "ESC[33m");
        out.clear();
        emit_sgr_delta(
            &mut out,
            Style::new(),
            Style::new().fg(Color::indexed(9)),
            &Packet::new(),
        );
        assert_eq!(text(&out), "ESC[91m");
        out.clear();
        emit_sgr_delta(
            &mut out,
            Style::new(),
            Style::new().bg(Color::indexed(1)),
            &Packet::new(),
        );
        assert_eq!(text(&out), "ESC[41m");
        out.clear();
        emit_sgr_delta(
            &mut out,
            Style::new(),
            Style::new().bg(Color::indexed(15)),
            &Packet::new(),
        );
        assert_eq!(text(&out), "ESC[107m");
    }

    #[test]
    fn a_palette_entry_above_fifteen_needs_the_long_form() {
        let mut out = Vec::new();
        emit_sgr_delta(
            &mut out,
            Style::new(),
            Style::new().fg(Color::indexed(200)),
            &Packet::new(),
        );
        assert_eq!(text(&out), "ESC[38;5;200m");
    }

    #[test]
    fn truecolor_is_three_channels() {
        let mut out = Vec::new();
        emit_sgr_delta(
            &mut out,
            Style::new(),
            Style::new().bg(Color::rgb(1, 2, 255)),
            &Packet::new(),
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
            &Packet::new(),
        );
        assert_eq!(text(&out), "ESC[39;49m");
    }

    #[test]
    fn an_extended_cell_is_painted_in_the_colours_its_table_entry_names() {
        // The defect this exists for: an extended word spends bits 51..0 on a handle, so reading
        // them as a 26-bit foreground and a 26-bit background emits whatever the handle happens to
        // look like. That is `Style::with_fg_bg`'s silent wrong answer one layer down, and it is a
        // *release* failure — the `debug_assert` on `Style::foreground` only catches the debug half.
        let mut frame = Surface::new(4, 1);
        let link = frame.tables_mut().link("https://example.com/");
        frame.root().text(0, 0, "ab", Style::new());
        frame.root().restyle(
            Rect::new(0, 0, 2, 1),
            &crate::restyle::Restyle {
                fg: Some(Color::indexed(3)),
                bg: Some(Color::rgb(1, 2, 255)),
                link: Some(link),
                ..Default::default()
            },
        );
        let out = text(&bytes_for(&frame));
        assert_eq!(
            out, "ESC[0mESC[1;1HESC[33;48;2;1;2;255mab",
            "the entry's colours reached the wire, not its handle"
        );
    }

    #[test]
    fn a_hyperlink_is_dropped_rather_than_emitted_as_anything() {
        // The honest half of the same sentence. SGR 58/59 and OSC 8 are ticket 13's, so a cell
        // presented today loses its hyperlink — degradation, which this engine has a model for.
        // A handle painted as a colour is not degradation, which is why the test above exists.
        let mut frame = Surface::new(4, 1);
        let link = frame.tables_mut().link("https://example.com/");
        frame.root().text(0, 0, "ab", Style::new());
        frame.root().restyle(
            Rect::new(0, 0, 2, 1),
            &crate::restyle::Restyle {
                link: Some(link),
                ..Default::default()
            },
        );
        let out = text(&bytes_for(&frame));
        assert!(!out.contains("]8;"), "no OSC 8 until ticket 13: {out}");
        assert!(
            !out.contains("58:"),
            "and no underline colour either: {out}"
        );
        assert_eq!(replay(&frame).cell(0, 0).grapheme, GraphemeId::scalar('a'));
    }

    #[test]
    fn an_sgr_that_would_say_nothing_emits_nothing() {
        let mut out = Vec::new();
        emit_sgr_delta(
            &mut out,
            Style::new().bold(),
            Style::new().bold(),
            &Packet::new(),
        );
        assert!(out.is_empty(), "an empty CSI m is a reset, not a no-op");
    }

    #[test]
    fn the_mirror_records_what_was_emitted() {
        let mut f = Surface::new(8, 2);
        f.root().text(3, 1, "z", Style::new().bold());
        let mut runs = Vec::new();
        f.damage().for_each_run(|r| runs.push(r));
        let mut packet = Packet::new();
        packet.pack(&runs, &f, f.tables(), false);
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
