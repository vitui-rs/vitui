//! Bytes on the wire: the mirror, the emit loop, and the differential SGR.
//!
//! # Scope
//!
//! Spec §8 in full: the `shortest` cursor encoding, a differential SGR with its three traps, the
//! two extended channels, synchronised output, and the mirror updated as bytes go out. The equality
//! filter is ticket 14 and the scroll region ticket 15; **nothing else about §8 is deferred.**
//!
//! # The serializer has no knobs
//!
//! The prototype was `Options`-shaped only so the variants could be measured. The engine ships
//! exactly one configuration and none of it is a choice an application makes — **no `Options` type
//! reaches the public surface**, and everything wire-specific lives below the packet. What the
//! [`Capabilities`] argument selects is not a preference: it is what the terminal on the other end
//! can parse.
//!
//! # An extended style's two channels, and the capability that gates one of them
//!
//! An extended word spends bits 51..0 on a handle, so [`channels_of`] resolves all four channels
//! through `packet.ext` instead of reading the word — reading that handle as a 26-bit foreground and
//! a 26-bit background is exactly the silent wrong answer `Style::with_fg_bg` was deleted for, one
//! layer down.
//!
//! The underline colour reaches the wire unconditionally, as SGR 58 or 59. **The hyperlink does
//! not**: OSC 8 is emitted only where [`Capabilities::hyperlinks`] says the terminal implements it,
//! and where it does not, a hyperlinked cell is painted in the right colours and loses its link.
//! That is degradation, which this engine has a model for; a handle emitted as a colour is not.
//!
//! Because `hyperlinks` is **inferred** rather than detected — OSC 8 has no query — it is the one
//! capability a caller can correct, through `Overrides::hyperlinks` (architecture ticket 22). That
//! is also what lets a headless round trip close on a hyperlinked cell at all.
//!
//! # The frame's framing, and the session's
//!
//! Every frame that has anything to say opens with SGR 0. That is tcell's "make no style
//! assumptions", four bytes, and it is what lets the emit loop start from a style it knows rather
//! than from one it inherited. Where the terminal has mode 2026 the frame is wrapped in it, and
//! `?2026h` + `0m` + `?2026l` is §8's **20 bytes of fixed framing** — which matters only because a
//! caret blink is a 29-byte frame.
//!
//! **A hyperlink is closed before the frame ends, and SGR 0 is not what closes it.** An open OSC 8
//! survives a reset, so a frame that left one open would hyperlink whatever the next frame wrote
//! next to it. The reset at the top is what makes a frame self-contained in *style*; the close at
//! the bottom is what makes it self-contained in *link*.
//!
//! Auto-wrap is off for the lifetime of the alt screen rather than per frame, so it is not here —
//! see [`crate::engine`]. It deletes two of cellbuf's bug-driven workarounds outright, and neither
//! is ported: with no wrap there is no pending-wrap state and no bottom-right corner that scrolls.

use crate::caps::Capabilities;
use crate::cell::{Cell, GraphemeId};
use crate::exts::LinkId;
use crate::packet::Packet;
use crate::quirks::Underlines;
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
    /// The hyperlink the terminal currently has open, or [`LinkId::NONE`].
    ///
    /// Tracked apart from [`style`](Serializer::style) because OSC 8 is not an SGR and **SGR 0 does
    /// not close it**: a frame that left one open would hyperlink whatever the next frame wrote.
    link: LinkId,
    /// The cluster emitted last, when the next cell's bytes would follow it with nothing between.
    /// See [`Serializer::joins_left`].
    prev: Option<GraphemeId>,
    /// Whether a non-ASCII cluster has gone out on the row the cursor is on.
    ///
    /// Spec §10's rule, as one boolean carried through a scan that is already running: **after a run
    /// containing any non-ASCII cluster, the next move on that row is `CHA` rather than `CUF`.** A
    /// width disagreement is *permanent* once a mirror exists — the mirror records what was
    /// intended, so an overpainted neighbour is never re-emitted — and `CUF` is relative and
    /// compounds the error along the row. An absolute move bounds the corruption to the one cell the
    /// terminal measured differently.
    ///
    /// Cleared whenever the cursor's row changes, because the fact is about a row.
    non_ascii_on_row: bool,
    /// What §10's rule has cost, in bytes, since this serializer was built.
    ///
    /// **A report, and spec §15's second owed measurement.** The rule refuses `CUF` on a row that has
    /// emitted a non-ASCII cluster, and what that costs is exactly the difference between the
    /// encoding actually chosen and the `CUF` that was refused — summed here rather than obtained by
    /// running a second serializer, because a second serializer would be a second configuration and
    /// this crate ships one.
    #[cfg(test)]
    cha_rule_bytes: usize,
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
            link: LinkId::NONE,
            prev: None,
            non_ascii_on_row: false,
            #[cfg(test)]
            cha_rule_bytes: 0,
        }
    }

    /// What §10's `CHA`-after-non-ASCII rule has cost in bytes. See
    /// [`cha_rule_bytes`](Serializer::cha_rule_bytes).
    #[cfg(test)]
    pub(crate) fn cha_rule_bytes(&self) -> usize {
        self.cha_rule_bytes
    }

    #[cfg(test)]
    pub(crate) fn mirror(&self) -> &Mirror {
        &self.mirror
    }

    /// Serialise a packet. The returned slice is valid until the next call.
    ///
    /// `caps` is what the terminal can parse, not what anybody prefers: the two SGR forms, ConPTY's
    /// underline-colour spelling, whether OSC 8 is worth emitting and whether the frame is wrapped
    /// in mode 2026 are all read from it and from nowhere else.
    pub(crate) fn serialize(&mut self, packet: &Packet, caps: &Capabilities) -> &[u8] {
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

        // Mode 2026 where the terminal has it, outside the reset: `?2026h` + `0m` + `?2026l` is
        // §8's twenty bytes of fixed framing. **The frame is never split on purpose** — a
        // synchronised-output block spanning two `write` calls is still one block to the terminal,
        // and a frame split into two blocks tears — so `write_frame`'s partial-write loop is about
        // the kernel's buffer being smaller than the frame and about nothing else.
        if caps.sync_output() {
            self.out.extend_from_slice(SYNC_BEGIN);
        }
        self.out.extend_from_slice(b"\x1b[0m");
        self.style = Style::DEFAULT;
        self.cursor = None;
        self.prev = None;
        self.non_ascii_on_row = false;
        // SGR 0 does not close an OSC 8, so the frame's own close at the bottom is what makes this
        // true rather than the reset above.
        self.link = LinkId::NONE;

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
                //
                // **The forced move is the cheapest one that is not nothing, not a `CUP`.** The
                // first draft of `shortest` expressed the force by clearing the cursor, which is
                // what ticket 03 did when `CUP` was the only encoding — and that quietly spent
                // eight bytes where `CR` spends one. The flag says *move*, and the encoder still
                // says *how*.
                let force =
                    self.cursor == Some((x, run.y)) && self.joins_left(cell.grapheme, packet);
                self.move_to(x, run.y, force);
                if cell.style != self.style {
                    emit_sgr_delta(&mut self.out, self.style, cell.style, packet, caps);
                    self.style = cell.style;
                    self.retarget_link(cell.style, packet, caps);
                }
                emit_grapheme(&mut self.out, cell.grapheme, packet);
                self.mirror.set(x, run.y, cell);
                self.advance(x, run.y, cell.grapheme.columns());
                self.prev = Some(cell.grapheme);
                self.non_ascii_on_row |= !is_ascii_scalar(cell.grapheme);
            }
        }
        // An open hyperlink outlives the frame that opened it, and SGR 0 is not what closes one.
        if !self.link.is_none() {
            emit_osc8(&mut self.out, None);
            self.link = LinkId::NONE;
        }
        if caps.sync_output() {
            self.out.extend_from_slice(SYNC_END);
        }
        self.note_whole_rows(packet);
        &self.out
    }

    /// Open, change or close the terminal's hyperlink to match the style just emitted.
    ///
    /// Called only where the style word changed, which is the same place the SGR delta goes out:
    /// cells in a run share a word, so this is a per-distinct-style cost and not a per-cell one.
    ///
    /// Where the terminal has no OSC 8 nothing is emitted and [`link`](Serializer::link) stays
    /// [`LinkId::NONE`], so the close at the end of the frame has nothing to do either.
    fn retarget_link(&mut self, style: Style, packet: &Packet, caps: &Capabilities) {
        if !caps.hyperlinks {
            return;
        }
        let want = channels_of(style, packet).link;
        if want == self.link {
            return;
        }
        // A URI the packet does not carry is the same disagreement `channels_of` answers
        // `Color::DEFAULT` for: `pack` and `serialize` think this is a different frame. Closing is
        // the answer that invents nothing.
        let uri = packet.link(want).filter(|_| !want.is_none());
        emit_osc8(&mut self.out, uri);
        self.link = if uri.is_some() { want } else { LinkId::NONE };
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
    /// is recorded rather than a position one past the end. Ticket 03 could not tell the two apart —
    /// a mutation swapping them left every test green, because runs are disjoint and ascending, so no
    /// later move on the same row ever targets a column already written — and **`shortest` is what
    /// makes it matter**: `CUF`'s distance is measured from this position, so a cursor model that is
    /// off by one puts every relative move on the row one column out.
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

    /// Put the cursor at `(x, y)` in the fewest bytes §8's encoding set allows.
    ///
    /// `CUP`, `CHA`, `CUF`, `CR`, `CR`+`LF`s — **priced by digit count, with no lookup table.**
    /// Every candidate's cost is a small sum and the cheapest wins; there is no per-move search over
    /// five encodings including content overwrite, which is cellbuf's version and is refused along
    /// with its `ICH`/`DCH` line editing that ticket 14's filter subsumes.
    ///
    /// It is worth **0.2% to 15%**, and the largest win is the chart — the scene with the most runs.
    /// Even there the win is not the *encoding*: absolute and natural tie, because a chart's runs are
    /// never contiguous, and the 848 bytes come almost entirely from choosing `CUF` and `CHA` over
    /// `CUP`.
    ///
    /// # `force`
    ///
    /// True when the cursor is already where the next cell goes and the bytes must nevertheless be
    /// separated — [`joins_left`](Serializer::joins_left). The cheapest non-empty move is then
    /// emitted, which is `CR` in column zero and `CHA` anywhere else. Both are absolute, which is
    /// also what [`non_ascii_on_row`](Serializer::non_ascii_on_row) would have demanded: a forced
    /// move only ever happens because a non-ASCII cluster is involved.
    ///
    /// # Two things a candidate must not be
    ///
    /// **`CUF` is refused on a row that has emitted a non-ASCII cluster** (spec §10) — that is the
    /// one rule here that is correctness rather than byte count.
    ///
    /// **An `LF` is only ever a downward move**, so it can never scroll: runs arrive in ascending row
    /// order, so `y > cy` implies `cy < height - 1`, and the feeds land on `y` at the latest. A
    /// terminal scrolls on `LF` from the last row whatever auto-wrap says, and this is why that never
    /// happens rather than a mode that prevents it.
    fn move_to(&mut self, x: u16, y: u16, force: bool) {
        if !force && self.cursor == Some((x, y)) {
            return;
        }
        let Some((cx, cy)) = self.cursor else {
            // Nothing is known about where the cursor is — the first move of a frame — so the only
            // truthful encoding is the absolute one.
            self.cup(x, y);
            self.cursor = Some((x, y));
            self.non_ascii_on_row = false;
            return;
        };

        // Every candidate priced, then the cheapest taken. Written as a list rather than as nested
        // branches because the price is a sum over candidates and the winner is only known at the
        // end — and because a branch that emits as it prices is how the first version of this ended
        // up spending eight bytes on a forced move.
        let same_row = y == cy;
        let candidates = [
            // One byte, and nothing beats it. `CR` is absolute in the column, so §10's rule has no
            // objection to it.
            (same_row && x == 0).then_some((Move::Cr, 1)),
            same_row.then(|| (Move::Cha, 3 + omissible(x as u32 + 1))),
            (same_row && x > cx && !self.non_ascii_on_row)
                .then(|| (Move::Cuf(x - cx), 3 + omissible((x - cx) as u32))),
            // `CR` then one `LF` per row, or the feeds alone when the cursor is already in column
            // zero. One-byte controls against a parameterised CSI, which is where §8's "fewest
            // bytes" and "fewest sequences" stop pulling in opposite directions.
            (!same_row && x == 0 && y > cy)
                .then(|| (Move::Feed, usize::from(cx != 0) + (y - cy) as usize)),
        ];
        let mut best = Move::Cup;
        let mut cost = 4 + digits(y as u32 + 1) + digits(x as u32 + 1);
        for (candidate, priced) in candidates.into_iter().flatten() {
            if priced < cost {
                best = candidate;
                cost = priced;
            }
        }

        // The report §15 is owed: what the rule cost on this move, which is the gap between what
        // was chosen and the `CUF` that was refused. Zero on every move where `CUF` would not have
        // won anyway, which is most of them.
        #[cfg(test)]
        if same_row && x > cx && self.non_ascii_on_row {
            self.cha_rule_bytes += cost.saturating_sub(3 + omissible((x - cx) as u32));
        }

        match best {
            Move::Cup => self.cup(x, y),
            Move::Cha => {
                self.out.extend_from_slice(b"\x1b[");
                if x > 0 {
                    push_num(&mut self.out, x as u32 + 1);
                }
                self.out.push(b'G');
            }
            Move::Cuf(by) => {
                self.out.extend_from_slice(b"\x1b[");
                if by > 1 {
                    push_num(&mut self.out, by as u32);
                }
                self.out.push(b'C');
            }
            Move::Cr => self.out.push(b'\r'),
            Move::Feed => {
                if cx != 0 {
                    self.out.push(b'\r');
                }
                for _ in cy..y {
                    self.out.push(b'\n');
                }
            }
        }
        if y != cy {
            self.non_ascii_on_row = false;
        }
        self.cursor = Some((x, y));
    }

    /// `CSI y;x H`, and nothing else: the cursor and §10's row flag are the caller's to update, so
    /// that there is one place each is written.
    fn cup(&mut self, x: u16, y: u16) {
        self.out.extend_from_slice(b"\x1b[");
        push_num(&mut self.out, y as u32 + 1);
        self.out.push(b';');
        push_num(&mut self.out, x as u32 + 1);
        self.out.push(b'H');
    }
}

/// Which of §8's five cursor encodings a move is spelled with.
///
/// An enum rather than five branches writing bytes as they are priced, because the price is a sum
/// over candidates and the winner is only known at the end. `CUB` is deliberately absent: runs are
/// disjoint and ascending, so nothing ever moves left except to column zero, which `CR` already
/// spells in one byte.
enum Move {
    /// `CSI y;x H`.
    Cup,
    /// `CSI x G` — absolute column, same row.
    Cha,
    /// `CSI n C` — relative, and refused after a non-ASCII cluster on this row.
    Cuf(u16),
    /// `\r`.
    Cr,
    /// `\r` where needed, then one `\n` per row.
    Feed,
}

/// DEC mode 2026 on: the terminal holds the frame back until the block closes.
const SYNC_BEGIN: &[u8] = b"\x1b[?2026h";
/// DEC mode 2026 off, which is what makes the frame appear at once.
const SYNC_END: &[u8] = b"\x1b[?2026l";

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
fn emit_sgr_delta(out: &mut Vec<u8>, old: Style, new: Style, packet: &Packet, caps: &Capabilities) {
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

    let was = channels_of(old, packet);
    let now = channels_of(new, packet);
    let legacy = caps.legacy_sgr();
    if was.fg != now.fg {
        emit_color(out, &mut params, now.fg, FOREGROUND, legacy);
    }
    if was.bg != now.bg {
        emit_color(out, &mut params, now.bg, BACKGROUND, legacy);
    }
    // The underline colour reaches the wire whatever the terminal answered, because there is nothing
    // to answer: SGR 58 has no query and a terminal that does not implement it ignores the sequence.
    // What *is* selected is the spelling — ConPTY parses only the semicolon form, which is
    // `legacy_sgr` one parameter along and invisible to whoever is looking at 38 and 48.
    if was.ul != now.ul {
        let conpty = caps.underlines() == Underlines::ConPty;
        emit_color(out, &mut params, now.ul, UNDERLINE, conpty);
    }

    if params == 0 {
        // Nothing to say. An empty `CSI m` is `CSI 0 m`, a reset, which is emphatically not
        // nothing — so the opener is taken back instead.
        out.truncate(mark);
        return;
    }
    out.push(b'm');
}

/// The four channels a style word paints with, whichever side of the extended bit it is on.
///
/// An extended word has no inline colours — bits 51..0 are the handle — so all four come from the
/// packet's own copy of the table, which is where the app thread put them at pack time. The engine
/// table is never reached from here, and that is ADR 0011's second sentence again.
///
/// A handle the packet does not carry means `pack` and `serialize` disagree about which frame this
/// is, and there is nothing truthful to paint: the terminal's own colours and no hyperlink are the
/// one answer that invents nothing.
fn channels_of(style: Style, packet: &Packet) -> crate::exts::ExtStyle {
    let inline = |fg, bg| crate::exts::ExtStyle {
        fg,
        bg,
        ul: Color::DEFAULT,
        link: LinkId::NONE,
    };
    let Some(handle) = style.ext_handle() else {
        return inline(style.foreground(), style.background());
    };
    match packet.ext(handle) {
        Some(e) => e,
        None => inline(Color::DEFAULT, Color::DEFAULT),
    }
}

/// Open, retarget or close the terminal's hyperlink.
///
/// `None` closes: `OSC 8 ; ; ST` with an empty URI is how the sequence says *no link here*, so
/// opening and closing are one shape with one spelling. The `id=` parameter is deliberately empty —
/// it exists so a terminal can join two runs of one logical link across a wrap, and with auto-wrap
/// off for the lifetime of the alt screen there are no wrapped runs to join.
///
/// `ST` is spelled `ESC \` rather than `BEL`: two bytes against one, and the one is xterm's
/// deviation rather than the standard.
fn emit_osc8(out: &mut Vec<u8>, uri: Option<&str>) {
    out.extend_from_slice(b"\x1b]8;;");
    if let Some(uri) = uri {
        out.extend_from_slice(uri.as_bytes());
    }
    out.extend_from_slice(b"\x1b\\");
}

/// Which of the three colour channels a `Color` is being emitted into.
///
/// The three differ in more than a base number, which is why this is a type: only the first two have
/// short forms for the low sixteen palette entries, and only the third spells *no colour* as a number
/// that is not `base + 9`.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Channel {
    /// The parameterised selector: `38`, `48`, `58`. **Not the short-form base plus eight** — that
    /// arithmetic is true of the first two and false of the third, and writing `base + 8` here is how
    /// the first version of this emitted `SGR 30:2::r:g:b` and painted every truecolor foreground as
    /// palette entry zero.
    extended: u32,
    /// What *the terminal's own colour* is spelled as: `39`, `49`, `59`.
    default: u32,
    /// Where `SGR 30`-`37` and `90`-`97` begin for this channel, where they exist at all.
    short: Option<u32>,
}

const FOREGROUND: Channel = Channel {
    extended: 38,
    default: 39,
    short: Some(30),
};
const BACKGROUND: Channel = Channel {
    extended: 48,
    default: 49,
    short: Some(40),
};
/// The underline colour, which has **no** short forms: there is no `SGR 58`-equivalent of `31`, so
/// palette entry 1 is spelled the long way like entry 200.
const UNDERLINE: Channel = Channel {
    extended: 58,
    default: 59,
    short: None,
};

/// Colour emission, spec §8: default is 39/49; indices under 16 use 30-37 / 90-97 and their
/// background forms rather than the long one; truecolor is three channels.
///
/// # The two spellings, and which is the default
///
/// The parameterised forms exist twice over. **Modern** is ITU-T T.416's colon form —
/// `38:2::r:g:b`, with the empty colour-space id the standard puts there — and **legacy** is xterm's
/// pre-ITU-T semicolon form, `38;2;r;g;b`. The modern one is what goes out unless something says
/// otherwise, and `Overrides::legacy_sgr`, `VITUI_FORCE_LEGACY_SGR` and three quirk entries are the
/// somethings: ConPTY, Termux and VSCode's integrated terminal all mis-parse the colon form.
///
/// **§8's own table spells the semicolon form and §10 names the legacy one *pre-ITU-T*, and the two
/// sentences cannot both be about the default.** The reading taken here is §10's, because it is the
/// one with a mechanism: three quirk entries force `legacy` on terminals that parse only semicolons,
/// so `legacy` cannot be what a terminal with no quirk entry receives. §8's spellings are the
/// configuration its byte tables were measured on, which is why the colon form's one extra byte per
/// parameterised colour shows up as a wire-budget number that had to be re-measured. Recorded in
/// impl 13's Progress rather than resolved by this comment.
fn emit_color(out: &mut Vec<u8>, params: &mut u32, c: Color, ch: Channel, legacy: bool) {
    /// `;` for the pre-ITU-T form, `:` for the modern one.
    fn sep(out: &mut Vec<u8>, legacy: bool) {
        out.push(if legacy { b';' } else { b':' });
    }
    match c.tag() {
        TAG_DEFAULT => param(out, params, ch.default),
        TAG_INDEXED => {
            let i = c.payload();
            match ch.short {
                Some(base) if i < 8 => param(out, params, base + i),
                Some(base) if i < 16 => param(out, params, base + 60 + (i - 8)),
                _ => {
                    param(out, params, ch.extended);
                    sep(out, legacy);
                    push_num(out, 5);
                    sep(out, legacy);
                    push_num(out, i);
                }
            }
        }
        TAG_RGB => {
            let v = c.payload();
            param(out, params, ch.extended);
            sep(out, legacy);
            push_num(out, 2);
            // T.416's colour-space id, which is empty and has to be there: `38:2::r:g:b`. The
            // semicolon form has no such parameter, and a terminal that wants one form and is given
            // the other reads the channels off by one and paints the wrong colour.
            if !legacy {
                out.push(b':');
            }
            for shift in [16, 8, 0] {
                sep(out, legacy);
                push_num(out, (v >> shift) & 0xFF);
            }
        }
        // `tag()` returns a `u32`, so the match has to be total. Tag 3 is reserved, `Color` has no
        // constructor that produces it, and this arm exists to satisfy the compiler rather than to
        // handle a case — the safe total answer being the terminal's own colour.
        _ => param(out, params, ch.default),
    }
}

fn param(out: &mut Vec<u8>, params: &mut u32, n: u32) {
    if *params > 0 {
        out.push(b';');
    }
    *params += 1;
    push_num(out, n);
}

/// How many decimal digits `n` takes, which is the whole of §8's pricing.
///
/// A loop rather than a table: the numbers here are a row, a column or a distance on an 80x24 to
/// 300x80 screen, so this runs one to three times and a table would be a table to keep in step.
fn digits(n: u32) -> usize {
    let mut d = 1;
    let mut n = n;
    while n >= 10 {
        n /= 10;
        d += 1;
    }
    d
}

/// The digits a parameter costs when a value of one may be left out entirely.
///
/// `CSI C` and `CSI G` both default their parameter to one, so `CUF` by one column and `CHA` to
/// column one are three bytes rather than four. It is one byte and it is the caret's kind of frame
/// that notices.
fn omissible(n: u32) -> usize {
    if n == 1 { 0 } else { digits(n) }
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
    use crate::caps::ColorDepth;
    use crate::cell::GraphemeId;
    use crate::geom::Rect;
    use crate::style::Style;
    use crate::surface::Surface;
    use crate::term_model::TermModel;

    /// A modern terminal with everything this file can reach: truecolor, OSC 8, the colon SGR form,
    /// the standard underline spelling and **no** synchronised output.
    ///
    /// 2026 is off because it would put eight bytes on either end of every byte string below, and
    /// the frames that are *about* the framing pin it on for themselves.
    fn modern() -> Capabilities {
        Capabilities::on_the_wire(ColorDepth::TrueColor, false, Underlines::Standard, false)
    }

    fn bytes_with(frame: &Surface, caps: &Capabilities) -> Vec<u8> {
        let mut runs = Vec::new();
        frame.damage().for_each_run(|r| runs.push(r));
        let mut packet = Packet::new();
        packet.pack(&runs, frame, frame.tables(), false);
        let (w, h) = frame.size();
        let mut s = Serializer::new(w, h);
        s.serialize(&packet, caps).to_vec()
    }

    fn bytes_for(frame: &Surface) -> Vec<u8> {
        bytes_with(frame, &modern())
    }

    fn text(s: &[u8]) -> String {
        String::from_utf8_lossy(s).replace('\x1b', "ESC")
    }

    /// The SGR one style change spells, on a terminal `caps` describes.
    fn sgr(old: Style, new: Style, caps: &Capabilities) -> String {
        let mut out = Vec::new();
        emit_sgr_delta(&mut out, old, new, &Packet::new(), caps);
        text(&out)
    }

    /// Replay the bytes and read the screen back. The round trip is the instrument for anything
    /// about *where a glyph lands*; a byte string would pin the encoding, which is the part
    /// tickets 14 and 15 are still allowed to change.
    fn replay(frame: &Surface) -> TermModel {
        let (w, h) = frame.size();
        let mut term = TermModel::new(w, h);
        // A set of its own is enough here: every scene in this module is scalars, and a scalar
        // handle is identity in any table. `Harness` is where a shared one is load-bearing.
        let mut tables = crate::tables::Tables::new();
        term.feed(&bytes_for(frame), &mut tables);
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
        assert_eq!(
            sgr(Style::new().bold().dim(), Style::new().dim(), &modern()),
            "ESC[22;2m"
        );
    }

    #[test]
    fn removing_dim_reapplies_bold() {
        assert_eq!(
            sgr(Style::new().bold().dim(), Style::new().bold(), &modern()),
            "ESC[22;1m"
        );
    }

    #[test]
    fn adding_bold_alone_is_one_parameter() {
        assert_eq!(sgr(Style::new(), Style::new().bold(), &modern()), "ESC[1m");
    }

    #[test]
    fn sgr_21_is_never_emitted_for_a_double_underline() {
        assert_eq!(
            sgr(Style::new(), Style::new().underline_double(), &modern()),
            "ESC[4:2m"
        );
    }

    #[test]
    fn dropping_an_underline_emits_24() {
        assert_eq!(
            sgr(Style::new().underline_curly(), Style::new(), &modern()),
            "ESC[24m"
        );
    }

    #[test]
    fn the_low_sixteen_palette_entries_are_spelled_short() {
        let caps = modern();
        for (style, expected) in [
            (Style::new().fg(Color::indexed(3)), "ESC[33m"),
            (Style::new().fg(Color::indexed(9)), "ESC[91m"),
            (Style::new().bg(Color::indexed(1)), "ESC[41m"),
            (Style::new().bg(Color::indexed(15)), "ESC[107m"),
        ] {
            assert_eq!(sgr(Style::new(), style, &caps), expected);
        }
    }

    #[test]
    fn a_palette_entry_above_fifteen_needs_the_long_form() {
        assert_eq!(
            sgr(
                Style::new(),
                Style::new().fg(Color::indexed(200)),
                &modern()
            ),
            "ESC[38:5:200m"
        );
    }

    #[test]
    fn truecolor_is_three_channels() {
        assert_eq!(
            sgr(
                Style::new(),
                Style::new().bg(Color::rgb(1, 2, 255)),
                &modern()
            ),
            "ESC[48:2::1:2:255m",
            "T.416's empty colour-space id is between the 2 and the channels"
        );
        // **Both channels, and the foreground is the half that was wrong.** The parameterised
        // selector is 38/48/58 and the short-form base is 30/40/none, and `base + 8` is true of the
        // first two and false of the third — so a `Channel` carrying only the base emitted
        // `SGR 30:2::r:g:b` and every truecolor foreground arrived as palette entry zero. The round
        // trip caught it; neither of the two colour tests that existed did, because one used a
        // background and the other used the low sixteen.
        assert_eq!(
            sgr(
                Style::new(),
                Style::new().fg(Color::rgb(3, 4, 5)),
                &modern()
            ),
            "ESC[38:2::3:4:5m"
        );
    }

    #[test]
    fn returning_to_the_default_colour_emits_39_and_49() {
        assert_eq!(
            sgr(
                Style::new().fg(Color::rgb(1, 2, 3)).bg(Color::indexed(4)),
                Style::new(),
                &modern()
            ),
            "ESC[39;49m"
        );
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
            out,
            "ESC[0mESC[1;1HESC[33;48:2::1:2:255mESC]8;;https://example.com/ESC\\abESC]8;;ESC\\",
            "the entry's colours reached the wire, not its handle"
        );
    }

    /// A hyperlinked surface, and the one URI on it.
    fn hyperlinked(uri: &str) -> Surface {
        let mut frame = Surface::new(4, 1);
        let link = frame.tables_mut().link(uri);
        frame.root().text(0, 0, "ab", Style::new());
        frame.root().restyle(
            Rect::new(0, 0, 2, 1),
            &crate::restyle::Restyle {
                link: Some(link),
                ..Default::default()
            },
        );
        frame
    }

    /// **A frame closes the hyperlink it opened, and SGR 0 is not what closes one.**
    ///
    /// An open OSC 8 survives a reset on a real terminal, so a frame that left one open would
    /// hyperlink whatever the next frame wrote beside it — including a cell whose style word says
    /// nothing about links, which is the failure that has no cell to blame.
    #[test]
    fn a_frame_closes_the_hyperlink_it_opened() {
        let out = text(&bytes_for(&hyperlinked("https://example.com/")));
        assert!(out.ends_with("ESC]8;;ESC\\"), "{out}");
        assert_eq!(
            out.matches("ESC]8;;").count(),
            2,
            "one open, one close: {out}"
        );
    }

    /// The degradation half, and it is a capability rather than a deferral now.
    ///
    /// Where the terminal has no OSC 8 a hyperlinked cell is painted in the right colours and loses
    /// its link. A handle emitted as a colour would not be degradation, which is what the test above
    /// this one exists for.
    #[test]
    fn a_terminal_without_osc8_gets_the_colours_and_not_the_link() {
        let frame = hyperlinked("https://example.com/");
        let mute = Capabilities::answering(ColorDepth::TrueColor, None, None);
        assert!(
            !mute.hyperlinks,
            "kitty answered nothing, so nothing is inferred"
        );
        let out = text(&bytes_with(&frame, &mute));
        assert!(!out.contains("]8;"), "{out}");
        assert!(out.contains("ESC[1;1H"), "the cells still went out: {out}");
    }

    /// **The round trip closes on a hyperlinked cell**, which is what ticket 13 is for: the model
    /// mints the URI back through the engine's own link table, so the replayed cell carries the
    /// frame's own extended-style handle rather than an equal-looking one.
    #[test]
    fn the_round_trip_closes_on_a_hyperlinked_cell() {
        let mut frame = hyperlinked("https://example.com/");
        assert!(
            frame.row(0)[0].style.ext_handle().is_some(),
            "the cell being compared has to be genuinely extended, or this closes on nothing"
        );
        let expected: Vec<Cell> = frame.row(0).to_vec();
        let bytes = bytes_for(&frame);
        let mut term = TermModel::new(4, 1);
        term.feed(&bytes, frame.tables_mut());
        assert_eq!(term.unrecognised(), 0);
        for x in 0..4 {
            assert_eq!(term.cell(x, 0), expected[x as usize], "column {x}");
        }
        assert_eq!(term.cell(0, 0).grapheme, GraphemeId::scalar('a'));
    }

    /// An underline colour round-trips the same way, and needs no capability to do it.
    #[test]
    fn the_round_trip_closes_on_an_underline_colour() {
        let mut frame = Surface::new(4, 1);
        frame.root().text(0, 0, "ab", Style::new());
        frame.root().restyle(
            Rect::new(0, 0, 2, 1),
            &crate::restyle::Restyle {
                ul: Some(Color::rgb(9, 8, 7)),
                ..Default::default()
            },
        );
        let bytes = bytes_for(&frame);
        let out = text(&bytes);
        assert!(out.contains("58:2::9:8:7"), "{out}");
        let expected = frame.row(0)[0];
        let mut term = TermModel::new(4, 1);
        term.feed(&bytes, frame.tables_mut());
        assert_eq!(term.unrecognised(), 0);
        assert_eq!(term.cell(0, 0), expected);
    }

    /// Dropping an underline colour is SGR 59, which is *the text's colour* and not a colour.
    #[test]
    fn dropping_an_underline_colour_is_fifty_nine() {
        // Built through the table, because that is the only way a style word carries one.
        let mut frame = Surface::new(4, 1);
        frame.root().text(0, 0, "ab", Style::new());
        frame.root().restyle(
            Rect::new(0, 0, 1, 1),
            &crate::restyle::Restyle {
                ul: Some(Color::rgb(9, 8, 7)),
                ..Default::default()
            },
        );
        let with = frame.row(0)[0].style;
        let mut packet = Packet::new();
        let mut runs = Vec::new();
        frame.damage().for_each_run(|r| runs.push(r));
        packet.pack(&runs, &frame, frame.tables(), false);
        let mut out = Vec::new();
        emit_sgr_delta(&mut out, with, Style::new(), &packet, &modern());
        assert_eq!(text(&out), "ESC[59m");
    }

    /// ConPTY parses only the semicolon form of SGR 58, which is `legacy_sgr` one parameter along.
    #[test]
    fn conpty_gets_its_own_underline_colour_spelling() {
        let mut frame = Surface::new(4, 1);
        frame.root().text(0, 0, "ab", Style::new());
        frame.root().restyle(
            Rect::new(0, 0, 2, 1),
            &crate::restyle::Restyle {
                ul: Some(Color::rgb(9, 8, 7)),
                ..Default::default()
            },
        );
        let conpty =
            Capabilities::on_the_wire(ColorDepth::TrueColor, false, Underlines::ConPty, true);
        let out = text(&bytes_with(&frame, &conpty));
        assert!(out.contains("58;2;9;8;7"), "{out}");
        assert!(!out.contains("58:"), "{out}");
    }

    /// The pre-ITU-T spelling of 38 and 48, which three quirk entries force.
    #[test]
    fn legacy_sgr_spells_a_colour_the_way_xterm_did() {
        let legacy =
            Capabilities::on_the_wire(ColorDepth::TrueColor, false, Underlines::Standard, true);
        assert_eq!(
            sgr(
                Style::new(),
                Style::new().bg(Color::rgb(1, 2, 255)),
                &legacy
            ),
            "ESC[48;2;1;2;255m"
        );
        assert_eq!(
            sgr(Style::new(), Style::new().fg(Color::indexed(200)), &legacy),
            "ESC[38;5;200m"
        );
    }

    /// A model that reads only the form we emit would agree with a wrong one, so it reads both.
    #[test]
    fn the_model_reads_a_colour_in_either_spelling() {
        for bytes in [
            &b"\x1b[48:2::1:2:255mx"[..],
            &b"\x1b[48;2;1;2;255mx"[..],
            &b"\x1b[48:2:1:2:255mx"[..],
        ] {
            let mut t = TermModel::new(4, 1);
            let mut tables = crate::tables::Tables::new();
            t.feed(bytes, &mut tables);
            assert_eq!(t.unrecognised(), 0, "{:?}", text(bytes));
            assert_eq!(
                t.cell(0, 0).style,
                Style::new().bg(Color::rgb(1, 2, 255)),
                "{:?}",
                text(bytes)
            );
        }
    }

    #[test]
    fn an_sgr_that_would_say_nothing_emits_nothing() {
        assert!(
            sgr(Style::new().bold(), Style::new().bold(), &modern()).is_empty(),
            "an empty CSI m is a reset, not a no-op"
        );
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
        s.serialize(&packet, &modern());
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

    // -----------------------------------------------------------------------------------------
    // `shortest`, and the one rule in it that is correctness rather than byte count.
    // -----------------------------------------------------------------------------------------

    /// Every encoding in §8's set, each on the frame that makes it the cheapest.
    ///
    /// **The costs are asserted, not the spellings' presence**, because the property is *fewest
    /// bytes* and a test that only checked which letter appeared would pass a `CUF` that cost more
    /// than the `CUP` it replaced. Each row is one move, so the cursor byte count is the frame's
    /// length minus the reset and the glyph.
    #[test]
    fn each_encoding_is_used_where_it_is_the_cheapest() {
        // Same row, one column along: `CUF` with the parameter left out, three bytes.
        let mut f = Surface::new(20, 3);
        f.root().text(0, 0, "a", Style::new());
        f.root().text(2, 0, "b", Style::new());
        let out = text(&bytes_for(&f));
        assert_eq!(out, "ESC[0mESC[1;1HaESC[Cb", "{out}");

        // A genuine tie, and **a tie goes to the absolute encoding.** From column one to column
        // twenty is a distance of 19 and a target of 21, both two digits, so `CUF` and `CHA` cost
        // five bytes each. Preferring `CHA` is not arbitrary: §10's rule exists because a relative
        // move compounds an error, so where the price is equal the encoding that cannot compound is
        // the one taken.
        let mut f = Surface::new(40, 1);
        f.root().text(0, 0, "a", Style::new());
        f.root().text(20, 0, "b", Style::new());
        let out = text(&bytes_for(&f));
        assert!(out.contains("ESC[21G"), "{out}");

        // And where `CUF` is strictly cheaper it wins: two columns along from column eight is four
        // bytes relative against five absolute.
        let mut f = Surface::new(20, 1);
        f.root().text(6, 0, "ab", Style::new());
        f.root().text(10, 0, "cd", Style::new());
        let out = text(&bytes_for(&f));
        assert!(out.contains("ESC[2C"), "{out}");

        // Down the left margin: `CR` and one `LF` per row, two one-byte controls against a
        // six-byte `CUP`.
        let mut f = Surface::new(20, 4);
        f.root().text(1, 0, "a", Style::new());
        f.root().text(0, 1, "b", Style::new());
        let out = text(&bytes_for(&f));
        assert!(out.contains("a\r\nb"), "{out}");

        // Three rows down: the same `CR` and one more byte a row, where a `CUP` would cost six
        // however far it went.
        let mut f = Surface::new(20, 4);
        f.root().text(0, 0, "a", Style::new());
        f.root().text(0, 3, "b", Style::new());
        let out = text(&bytes_for(&f));
        assert!(out.contains("a\r\n\n\nb"), "{out}");
    }

    /// The frame's cursor cost, priced, against the encoding ticket 03 shipped.
    ///
    /// §8 puts `shortest` at 0.2% to 15% and says the win is not the encoding but the *choice*. This
    /// is that as a byte count on the shape the win comes from: many short runs on one row, which is
    /// the sub-cell chart's shape.
    #[test]
    fn shortest_beats_cup_only_on_a_row_of_short_runs() {
        let mut f = Surface::new(300, 1);
        for x in (0..300).step_by(3) {
            f.root().text(x, 0, "x", Style::new());
        }
        let bytes = bytes_for(&f).len();
        // What ticket 03's `CUP`-only loop would have spent: four fixed bytes plus the digits of
        // both coordinates, per run, plus one glyph, plus the four-byte reset.
        let cup_only = 4
            + (0..300)
                .step_by(3)
                .map(|x| 4 + 1 + digits(x + 1) + 1)
                .sum::<usize>();
        assert!(
            bytes < cup_only,
            "shortest spent {bytes} where CUP-only spends {cup_only}"
        );
        // And the picture is still right, which is the half a byte count cannot say.
        assert_eq!(replay(&f).cell(297, 0).grapheme, GraphemeId::scalar('x'));
    }

    /// **Spec §10's rule.** After a run containing a non-ASCII cluster, the next move on that row is
    /// absolute — and the frame it is about is a frame where `CUF` would otherwise have won.
    #[test]
    fn a_non_ascii_run_forces_the_next_move_on_its_row_to_be_absolute() {
        let ascii = {
            let mut f = Surface::new(20, 1);
            f.root().text(6, 0, "ab", Style::new());
            f.root().text(10, 0, "cd", Style::new());
            bytes_for(&f)
        };
        assert!(
            csi_finals(&ascii).any(|b| b == b'C'),
            "CUF is strictly cheaper here, so the rule has something to refuse: {}",
            text(&ascii)
        );

        let mut f = Surface::new(20, 1);
        f.root().text(6, 0, "a\u{e9}", Style::new());
        f.root().text(10, 0, "cd", Style::new());
        let out = bytes_for(&f);
        assert!(
            csi_finals(&out).all(|b| b != b'C'),
            "no CUF after a non-ASCII cluster: {}",
            text(&out)
        );
        assert!(
            text(&out).contains("ESC[11G"),
            "the move is CHA instead: {}",
            text(&out)
        );
    }

    /// And the flag is about a *row*: the next row starts trusting `CUF` again.
    #[test]
    fn the_rule_is_cleared_by_moving_to_another_row() {
        let mut f = Surface::new(20, 2);
        f.root().text(0, 0, "\u{e9}", Style::new());
        f.root().text(6, 1, "ab", Style::new());
        f.root().text(10, 1, "cd", Style::new());
        let out = text(&bytes_for(&f));
        assert!(
            out.contains("ESC[2C"),
            "row 1 has emitted nothing wide: {out}"
        );
    }

    /// **The corruption a width disagreement causes stays bounded to one cell instead of
    /// compounding**, which is the whole reason the rule is correctness rather than byte count.
    ///
    /// The counterfactual is written as bytes rather than as a second serializer configuration,
    /// because this crate ships one: the two streams below say the same thing, one absolutely and one
    /// relatively, and only the relative one moves the third run.
    #[test]
    fn a_width_disagreement_does_not_compound_along_the_row() {
        // **Two** wide clusters in separate runs, then an ASCII one. Two is the smallest number
        // that can tell *bounded* from *compounding* apart: with one, a relative move is one column
        // out and stays one column out, and the fixture would report the rule working whether it
        // was there or not.
        let mut f = Surface::new(20, 1);
        f.root().text(0, 0, "漢", Style::new());
        f.root().text(4, 0, "漢", Style::new());
        f.root().text(8, 0, "cd", Style::new());
        let bytes = bytes_for(&f);
        assert!(
            csi_finals(&bytes).all(|b| b != b'C'),
            "the serializer chose absolute moves: {}",
            text(&bytes)
        );

        // Replayed on a terminal that gives 漢 one column: what is wrong is the column each pair's
        // continuation should have covered, and nothing else. **Every run still starts where it
        // belongs**, which is the whole of what the rule buys.
        let mut narrow = TermModel::disagreeing_about_width(20, 1);
        narrow.feed(&bytes, f.tables_mut());
        assert_eq!(
            narrow.cell(0, 0).grapheme.columns(),
            2,
            "our tables' opinion"
        );
        assert_eq!(narrow.cell(4, 0).grapheme.columns(), 2);
        assert_eq!(narrow.cell(8, 0).grapheme, GraphemeId::scalar('c'));
        assert_eq!(narrow.cell(9, 0).grapheme, GraphemeId::scalar('d'));

        // The same picture with every move spelled relatively, which is what the rule refuses: two
        // columns from the cursor after the first cluster, two more after the second. The first run
        // lands one column early and the last lands **two**, and on a longer row it keeps going —
        // that is *compounds*, and a mirror is what makes it permanent, because the mirror records
        // what was intended and an overpainted neighbour is never re-emitted.
        let mut compounding = TermModel::disagreeing_about_width(20, 1);
        let mut tables = crate::tables::Tables::new();
        compounding.feed(
            "\x1b[0m\x1b[1;1H漢\x1b[2C漢\x1b[2Ccd".as_bytes(),
            &mut tables,
        );
        assert_eq!(
            compounding.cell(3, 0).grapheme.columns(),
            2,
            "one column early"
        );
        assert_eq!(
            compounding.cell(6, 0).grapheme,
            GraphemeId::scalar('c'),
            "and this one is two columns early, not one"
        );
    }

    /// **Spec §15's second owed measurement, paid.** What the `CHA`-after-non-ASCII rule costs in
    /// bytes, against the 848 bytes `shortest` won on the chart.
    ///
    /// A report rather than a gate — the numbers belong to the fixtures, not to the mechanism — and
    /// three rows rather than one, because a single number here would have been either meaningless or
    /// alarming depending on which fixture produced it.
    ///
    /// **The answer is that it costs nothing on anything §14 measures, and up to a third of the frame
    /// on a shape §14 does not have.** The rule only ever charges for a move that is *along a row the
    /// serializer has already put a non-ASCII cluster on*, and that needs two things at once: several
    /// runs on one row, and non-ASCII inside them. §14's twelve have the first — the chart is 113
    /// moves — and none of them has the second, because the chart plots with `*`. A full screen of
    /// CJK has the second and not the first: its runs are whole rows, so every move is a row change
    /// and `CUF` was never a candidate.
    ///
    /// So the 848 bytes `shortest` won on the chart stand undiminished, and the ceiling is a shape
    /// that is real but is not on the list: a chart drawn in braille or a table of CJK columns. **The
    /// last row is what a component library has to know**, and it is the reason this is reported per
    /// fixture rather than summed.
    #[test]
    fn what_the_cha_rule_costs_in_bytes() {
        /// One fixture's bytes and what the rule charged for them.
        fn priced(f: &Surface) -> (usize, usize) {
            let mut runs = Vec::new();
            f.damage().for_each_run(|r| runs.push(r));
            let mut packet = Packet::new();
            packet.pack(&runs, f, f.tables(), false);
            let (w, h) = f.size();
            let mut s = Serializer::new(w, h);
            let bytes = s.serialize(&packet, &modern()).len();
            (bytes, s.cha_rule_bytes())
        }

        // §14's twelve, through the screens they are actually measured on.
        let mut worst = ("", 0usize, 0usize);
        for mut scene in crate::scenes::scenes() {
            let mut harness = crate::testing::Harness::with_overrides(
                crate::scenes::W,
                crate::scenes::H,
                scene.overrides(),
            );
            scene.build(&mut harness.screen);
            harness.present();
            for t in 1..=3 {
                scene.step(&mut harness.screen, t);
                harness.present();
            }
            let cost = harness.screen.cha_rule_bytes();
            if cost >= worst.1 {
                worst = (scene.name(), cost, harness.bytes_written());
            }
        }

        // A full screen of mixed CJK, whole rows: the second ingredient without the first.
        let mut cjk = Surface::new(300, 80);
        for y in 0..80 {
            let row = crate::testing::bisecting_cjk::row(y, 300);
            cjk.root().text(0, y as i32, &row, Style::new());
        }
        let (cjk_bytes, cjk_cost) = priced(&cjk);

        // Both ingredients: the chart's shape with a wide cluster in place of the `*`. Not on §14's
        // list, and entirely constructible by a component — a braille chart is this.
        let mut dense = Surface::new(300, 80);
        for y in 0..80 {
            for x in (0..300).step_by(3) {
                dense.root().text(x, y, "漢", Style::new());
            }
        }
        let (dense_bytes, dense_cost) = priced(&dense);

        println!(
            "\nwhat the CHA-after-non-ASCII rule costs, spec §15's second owed measurement:\n  \
             §14's twelve, worst row ({}) {} bytes of {}\n  \
             a full screen of mixed CJK, whole rows          {cjk_cost} bytes of {cjk_bytes}\n  \
             a chart's runs drawn in a wide cluster          {dense_cost} bytes of {dense_bytes} \
             ({:.1}%)\n  \
             The rule charges only for a move along a row that has already emitted a non-ASCII \
             cluster, so it\n  \
             needs several runs on one row *and* non-ASCII inside them. §14's twelve have the first \
             and not\n  \
             the second — the chart plots with `*` — and a screen of CJK has the second and not the \
             first,\n  \
             because whole-row runs make every move a row change and `CUF` was never a candidate. \
             **So the\n  \
             848 bytes `shortest` won on the chart are not paid back**, and the third row is the \
             ceiling: a\n  \
             shape that is real, that a component library will write, and that is not on the list. \
             Report, not\n  \
             a gate.",
            worst.0,
            worst.1,
            worst.2,
            100.0 * dense_cost as f64 / dense_bytes as f64,
        );
        assert_eq!(
            worst.1, 0,
            "a scene on §14's list started paying for the rule"
        );
        assert!(
            dense_cost > 0,
            "the ceiling fixture has to exercise the rule to price it"
        );
    }

    // -----------------------------------------------------------------------------------------
    // The frame's framing, and the session's.
    // -----------------------------------------------------------------------------------------

    /// §8's twenty bytes of fixed framing, on the frame that is the reason they are counted.
    #[test]
    fn the_frame_is_wrapped_in_mode_2026_where_the_terminal_has_it() {
        // §8's caret frame: an eight-byte `CUP` and one ASCII cell is the nine bytes of *the
        // caret's own change*, and the framing is the other twenty.
        let mut f = Surface::new(80, 24);
        f.root().text(40, 12, "x", Style::new());
        let syncing =
            Capabilities::on_the_wire(ColorDepth::TrueColor, true, Underlines::Standard, false);
        let out = bytes_with(&f, &syncing);
        assert!(out.starts_with(b"\x1b[?2026h\x1b[0m"), "{}", text(&out));
        assert!(out.ends_with(b"\x1b[?2026l"), "{}", text(&out));
        // `?2026h` + `0m` + `?2026l` is twenty bytes, and the caret's own change is the rest.
        assert_eq!(SYNC_BEGIN.len() + 4 + SYNC_END.len(), 20);
        assert_eq!(
            out.len(),
            29,
            "§8's 29-byte caret frame, which no test could construct before impl 13: {}",
            text(&out)
        );

        // And the model agrees it was one block, which is §8's *the frame is never split on
        // purpose* read from the other end.
        let mut term = TermModel::new(80, 24);
        let mut tables = crate::tables::Tables::new();
        term.feed(&out, &mut tables);
        assert_eq!(term.unrecognised(), 0);
        assert_eq!(term.sync_blocks(), 1);
    }

    /// A terminal without mode 2026 gets no framing at all, rather than framing it will ignore.
    #[test]
    fn a_terminal_without_mode_2026_is_sent_none() {
        let mut f = Surface::new(8, 1);
        f.root().text(0, 0, "a", Style::new());
        assert!(!text(&bytes_for(&f)).contains("2026"));
    }

    // -----------------------------------------------------------------------------------------
    // Register #12, over the configurations the terminal model can express.
    // -----------------------------------------------------------------------------------------

    /// **Gate, equality (register #12): the round trip stays green under every wire configuration.**
    ///
    /// There are four axes below the packet that change the bytes without changing the picture — the
    /// two SGR spellings, the two underline-colour spellings, mode 2026 on or off, and OSC 8 on or
    /// off — and the property is that none of them changes the *screen*. Sixteen combinations, one
    /// frame, and the frame is chosen to touch every branch there is: a wide cluster, a combining
    /// one, a palette colour under sixteen, one above it, a truecolor pair, an underline colour and a
    /// hyperlink.
    ///
    /// Two of the axes are declarable and two are not, and that is why this gate is here rather than
    /// in `crate::gates`: `sync_output` and `underlines` have no `Overrides` field — nobody at the
    /// terminal can name mode 2026 or ConPTY's underline-colour form — so their arms are reached from
    /// inside the crate through a synthetic `Detected` put through `assemble`
    /// (`Capabilities::on_the_wire`). A gate above `Screen` could not construct half of this table.
    ///
    /// The OSC 8 axis is the one asymmetry: where the terminal has no hyperlinks the replayed cell is
    /// *deliberately* not equal, because the link is dropped. So that arm asserts the degradation
    /// instead — same glyphs, same colours, no link — which is a statement about the same bytes and
    /// not a weaker one.
    #[test]
    fn the_round_trip_closes_under_every_wire_configuration() {
        /// The frame every arm replays: one of everything the encoding set can spell.
        fn frame() -> Surface {
            let mut f = Surface::new(24, 2);
            let link = f.tables_mut().link("https://example.com/vitui#1");
            f.root()
                .text(0, 0, "漢a\u{301}b", Style::new().bold().dim());
            f.root().text(
                0,
                1,
                "cd",
                Style::new()
                    .fg(Color::indexed(3))
                    .bg(Color::indexed(200))
                    .underline_curly(),
            );
            f.root()
                .text(4, 1, "ef", Style::new().fg(Color::rgb(1, 2, 3)));
            f.root().restyle(
                Rect::new(0, 1, 2, 1),
                &crate::restyle::Restyle {
                    ul: Some(Color::rgb(9, 8, 7)),
                    link: Some(link),
                    ..Default::default()
                },
            );
            f
        }

        /// The four channels a cell paints with, resolved through the surface's own tables.
        ///
        /// The **same** tables the terminal model minted into, which is what makes the two
        /// comparable: a channel the engine wrote resolves to the entry the engine made, and one it
        /// did not gets an entry nobody else has.
        fn resolve(cell: Cell, f: &Surface) -> crate::exts::ExtStyle {
            match cell.style.ext_handle() {
                Some(h) => f
                    .tables()
                    .exts
                    .get(h)
                    .expect("the handle came out of this table"),
                None => crate::exts::ExtStyle {
                    fg: cell.style.foreground(),
                    bg: cell.style.background(),
                    ul: Color::DEFAULT,
                    link: LinkId::NONE,
                },
            }
        }

        for legacy in [false, true] {
            for underlines in [Underlines::Standard, Underlines::ConPty] {
                for sync in [false, true] {
                    for hyperlinks in [false, true] {
                        let caps = Capabilities::on_the_wire(
                            ColorDepth::TrueColor,
                            sync,
                            underlines,
                            legacy,
                        );
                        // The one axis that is a field, applied on top of the synthetic terminal.
                        let caps = Capabilities { hyperlinks, ..caps };
                        let mut f = frame();
                        let expected: Vec<Cell> = (0..2).flat_map(|y| f.row(y).to_vec()).collect();
                        let bytes = bytes_with(&f, &caps);
                        let mut term = TermModel::new(24, 2);
                        term.feed(&bytes, f.tables_mut());
                        let what = format!(
                            "legacy {legacy}, {underlines:?}, sync {sync}, hyperlinks {hyperlinks}"
                        );
                        assert_eq!(term.unrecognised(), 0, "{what}: {}", text(&bytes));
                        assert_eq!(term.sync_blocks(), usize::from(sync), "{what}");

                        for y in 0..2u16 {
                            for x in 0..24u16 {
                                let want = expected[y as usize * 24 + x as usize];
                                let got = term.cell(x, y);
                                assert_eq!(got.grapheme, want.grapheme, "{what} at ({x}, {y})");
                                if hyperlinks {
                                    assert_eq!(got.style, want.style, "{what} at ({x}, {y})");
                                    continue;
                                }
                                // The degradation, asserted rather than skipped: **the link is the
                                // only thing that may differ.** The underline colour still arrives —
                                // SGR 58 needs no capability — so the cell is extended either way,
                                // and comparing attributes alone would let a dropped colour pass as
                                // a dropped link.
                                assert_eq!(
                                    got.style.attr_word(),
                                    want.style.attr_word(),
                                    "{what} at ({x}, {y})"
                                );
                                let (had, has) = (resolve(want, &f), resolve(got, &f));
                                assert_eq!(
                                    (had.fg, had.bg, had.ul),
                                    (has.fg, has.bg, has.ul),
                                    "{what} at ({x}, {y}): a colour was dropped, not a link"
                                );
                                assert!(has.link.is_none(), "{what} at ({x}, {y})");
                            }
                        }
                        if !hyperlinks {
                            assert!(!text(&bytes).contains("]8;"), "{what}: {}", text(&bytes));
                        }
                    }
                }
            }
        }
    }

    /// **Report: the worst case, and the tearing that is a known consequence rather than a bug.**
    ///
    /// §8's row is 805 657 bytes, 24 000 SGR sequences and 925 µs for a frame where every cell has a
    /// distinct style — 33.6 bytes a cell against 1.0 for a realistic one. The byte count and the SGR
    /// count are exact and machine-independent, which is why they are the numbers this prints; the
    /// microseconds are a build and a machine, so run it with `--release` before believing them.
    ///
    /// ```text
    /// cargo test --release -p vitui-engine the_worst_case_frame -- --nocapture
    /// ```
    ///
    /// The consequence is the point. **On a 4 MB/s link this frame is nearly twice Alacritty's 150 ms
    /// force-flush limit**, so a worst-case frame over a slow link *will* be force-flushed and *will*
    /// tear. Nothing mitigates it and nothing needs to: a screen where every cell has a unique colour
    /// is a test fixture. It is recorded so that the tearing is a known consequence and not a bug
    /// report — and the byte limit is emphatically **not** what binds: 805 KB is 0.38× of Alacritty's
    /// 2 MiB.
    #[test]
    fn the_worst_case_frame() {
        const W: u16 = 300;
        const H: u16 = 80;
        /// Bytes a second on the link §8 prices this against.
        const LINK_BYTES_PER_SEC: f64 = 4.0 * 1024.0 * 1024.0;
        /// Alacritty's, which is the shortest of the three §8 names.
        const FORCE_FLUSH_MS: f64 = 150.0;

        let mut f = Surface::new(W, H);
        for y in 0..H {
            for x in 0..W {
                // Every cell a distinct 24-bit **pair**, which is what makes every cell its own
                // style run and therefore its own SGR — and what makes the row 33.6 bytes a cell
                // rather than 17.5. A fixture that varied one channel is the same shape at half the
                // size, and half of the worst case is not the worst case.
                let n = y as u32 * W as u32 + x as u32;
                let rgb = |v: u32| Color::rgb((v >> 16) as u8, (v >> 8) as u8, v as u8);
                f.root().text(
                    x as i32,
                    y as i32,
                    "m",
                    Style::new().fg(rgb(n ^ 0xFF_FFFF)).bg(rgb(n)),
                );
            }
        }
        let mut runs = Vec::new();
        f.damage().for_each_run(|r| runs.push(r));
        let mut packet = Packet::new();
        packet.pack(&runs, &f, f.tables(), false);
        let mut s = Serializer::new(W, H);
        let caps = modern();
        let at = std::time::Instant::now();
        let bytes = s.serialize(&packet, &caps).len();
        let took = at.elapsed().as_secs_f64() * 1e6;

        let sgr = {
            let mut again = Serializer::new(W, H);
            let out = again.serialize(&packet, &caps).to_vec();
            csi_finals(&out).filter(|b| *b == b'm').count() - 1
        };
        let cells = u32::from(W) * u32::from(H);
        let transmit_ms = bytes as f64 / LINK_BYTES_PER_SEC * 1000.0;
        println!(
            "\nthe worst case, {W}x{H}, every cell a distinct style:\n  \
             {bytes} bytes, {sgr} SGR, {took:.0} us to serialise, {:.1} bytes a cell\n  \
             spec §8 measured 805 657 bytes, 24 000 SGR, 925 us and 33.6 bytes a cell\n  \
             on a 4 MB/s link that is {transmit_ms:.0} ms, against Alacritty's {FORCE_FLUSH_MS:.0} ms \
             force-flush limit — {:.1}x\n  \
             so this frame **will** be force-flushed and **will** tear. The byte limit is not what \
             binds: 2 MiB is {:.2}x this.\n  \
             A realistic full-screen frame is 1.0 bytes a cell. Report, not a gate.",
            bytes as f64 / cells as f64,
            transmit_ms / FORCE_FLUSH_MS,
            2.0 * 1024.0 * 1024.0 / bytes as f64,
        );
        assert_eq!(
            sgr, cells as usize,
            "one SGR a cell is what makes this the worst case"
        );
        // **The sentence above is asserted, not just printed.** It is a byte count against two
        // constants, which is the register's own favourite shape, and it exists so that a change
        // making this frame smaller cannot leave a report claiming a tear that no longer happens.
        assert!(
            transmit_ms > FORCE_FLUSH_MS,
            "{transmit_ms:.0} ms is inside the force-flush limit, so this frame no longer tears \
             and the paragraph above is no longer true"
        );
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
