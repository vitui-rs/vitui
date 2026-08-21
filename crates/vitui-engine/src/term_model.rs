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
//! | `CHA` — `CSI x G`, `CUF` — `CSI n C`, `CR`, `LF` | ticket 13 |
//! | `SGR 58`/`59`, in both spellings, and `OSC 8` | ticket 13 |
//! | `DECAWM` — `CSI ?7 h`/`l`, and mode 2026 — `CSI ?2026 h`/`l` | ticket 13 |
//!
//! Not parsed yet, each with the ticket that adds it: `DECSTBM`, `SU`, `SD` (15); the pending-wrap
//! state (15, and only reachable if auto-wrap is ever switched back on inside a session). An
//! unrecognised sequence is counted rather than ignored, so a serializer that starts emitting one
//! before this model understands it fails the round trip instead of passing it silently.
//!
//! Auto-wrap is off for the lifetime of the alt screen, so this model does not wrap: a print in the
//! last column leaves the cursor in the last column. The state is **tracked** rather than assumed
//! from ticket 13 on, because the prologue that switches it off is now bytes this model sees, and a
//! model that silently accepted `CSI ?7 h` would let the epilogue's restoration go unasserted.
//!
//! # An extended cell is read back through the engine's own tables
//!
//! The model interns clusters into the engine's interner (ticket 06) for one reason, and SGR 58 and
//! OSC 8 need the same reason applied to two more tables: **a cell is compared whole, handle
//! included**, and two tables cannot produce equal handles for one entry. So an underline colour and
//! a URI parsed off the wire are minted back through the engine's [`Tables`], which is where the
//! frame's own handle came from — and a value the engine never wrote gets a handle nobody has, the
//! comparison fails, and that is the point.
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
use crate::exts::{ExtStyle, LinkId};
use crate::style::{Color, Style};
use crate::tables::Tables;
use crate::ucd;

/// How this terminal measures a cluster.
///
/// Spec §15 owes a measurement about the `CHA`-after-non-ASCII rule, and the rule is about a terminal
/// that disagrees with our UAX #11 tables. This is that terminal: **`Narrow` gives every wide cluster
/// one column**, which is the `wcwidth`-era answer and the disagreement the rule exists to bound.
///
/// It is a property of the *model* rather than of the engine, which is the whole point — the engine's
/// tables stay authoritative and the rule is what keeps the resulting corruption to one cell instead
/// of letting it compound along the row.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(crate) enum WidthOpinion {
    /// The engine's own tables, which is what every real terminal in tier 1 agrees with.
    #[default]
    Ours,
    /// One column for everything, however wide our tables say a cluster is.
    Narrow,
}

/// A terminal, as far as the serializer's output can tell.
pub(crate) struct TermModel {
    width: u16,
    height: u16,
    cells: Vec<Cell>,
    cursor: (u16, u16),
    /// The attribute bits, and the two colours while nothing extended is set.
    ///
    /// **Never itself an extended word.** The extended channels live in the two fields below and the
    /// word is minted at print time, because that is the only moment all four are known and the only
    /// moment a handle is wanted.
    style: Style,
    /// What SGR 58 named, or [`Color::DEFAULT`] for SGR 59.
    ul: Color,
    /// What OSC 8 opened, or [`LinkId::NONE`].
    link: LinkId,
    /// DECAWM. **Off for the lifetime of the alt screen**, so this exists to assert the prologue and
    /// the epilogue rather than to change what `print` does — a model that silently accepted
    /// `CSI ?7 h` would let the restoration go unasserted.
    autowrap: bool,
    /// Whether a synchronised-output block is open, and how many have closed.
    ///
    /// §8's *the frame is never split on purpose*: a frame is one block, so an open inside an open
    /// is a defect and is counted.
    sync_open: bool,
    sync_blocks: usize,
    /// Bytes of a sequence that was cut in half by a partial write.
    buf: Vec<u8>,
    /// Sequences this model does not understand. The round trip asserts it is zero.
    unrecognised: usize,
    widths: WidthOpinion,
}

impl TermModel {
    pub(crate) fn new(width: u16, height: u16) -> TermModel {
        TermModel {
            width,
            height,
            cells: vec![Cell::BLANK; width as usize * height as usize],
            cursor: (0, 0),
            style: Style::DEFAULT,
            ul: Color::DEFAULT,
            link: LinkId::NONE,
            // A terminal in the alt screen has auto-wrap on until the prologue turns it off, which
            // is a byte this model now sees.
            autowrap: true,
            sync_open: false,
            sync_blocks: 0,
            buf: Vec::new(),
            unrecognised: 0,
            widths: WidthOpinion::default(),
        }
    }

    /// The same terminal, measuring clusters the `wcwidth` way.
    ///
    /// Spec §15's owed measurement, as an instrument: what the `CHA`-after-non-ASCII rule buys is
    /// only visible against a terminal that disagrees about width, and no real tier-1 terminal does.
    #[cfg(test)]
    pub(crate) fn disagreeing_about_width(width: u16, height: u16) -> TermModel {
        TermModel {
            widths: WidthOpinion::Narrow,
            ..TermModel::new(width, height)
        }
    }

    /// Whether auto-wrap is on, which the session prologue and epilogue are asserted through.
    #[cfg(test)]
    pub(crate) fn autowrap(&self) -> bool {
        self.autowrap
    }

    /// How many synchronised-output blocks have opened and closed.
    #[cfg(test)]
    pub(crate) fn sync_blocks(&self) -> usize {
        self.sync_blocks
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
    pub(crate) fn feed(&mut self, bytes: &[u8], tables: &mut Tables) {
        // Taken out so the segmenter can borrow it while the cells are being written into.
        let mut buf = std::mem::take(&mut self.buf);
        buf.extend_from_slice(bytes);
        let mut at = 0usize;
        while at < buf.len() {
            let b = buf[at];
            if b == 0x1b {
                match self.parse_escape(&buf, at, tables) {
                    Some(next) => at = next,
                    // Incomplete: keep what is left and wait for the rest.
                    None => break,
                }
            } else if b == b'\r' {
                self.cursor.0 = 0;
                at += 1;
            } else if b == b'\n' {
                // **A terminal scrolls on `LF` from the last row**, whatever auto-wrap says, and
                // `shortest` never asks for one: runs arrive in ascending row order, so a feed is
                // always a downward move onto a row that exists. Counted rather than modelled,
                // because a scroll the serializer did not intend is a defect and not a picture.
                if self.cursor.1 + 1 < self.height {
                    self.cursor.1 += 1;
                } else {
                    self.unrecognised += 1;
                }
                at += 1;
            } else if b < 0x20 || b == 0x7F {
                // No other control character is emitted by this serializer.
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
                            self.print(cluster, tables);
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
    fn parse_escape(&mut self, buf: &[u8], at: usize, tables: &mut Tables) -> Option<usize> {
        if at + 1 >= buf.len() {
            return None;
        }
        match buf[at + 1] {
            b'[' => self.parse_csi(buf, at),
            b']' => self.parse_osc(buf, at, tables),
            _ => {
                self.unrecognised += 1;
                Some(at + 2)
            }
        }
    }

    fn parse_csi(&mut self, buf: &[u8], at: usize) -> Option<usize> {
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
            // `CHA` — absolute column, same row.
            b'G' => self.cursor.0 = column(first(params).max(1) - 1, self.width),
            // `CUF` — relative, and the one encoding §10's rule refuses after a non-ASCII cluster.
            b'C' => {
                let by = first(params).max(1);
                self.cursor.0 = column(self.cursor.0 as u32 + by, self.width);
            }
            // The DEC private modes: `?7` is auto-wrap and `?2026` is synchronised output. Both are
            // parsed so that the serializer cannot emit one this model silently accepts.
            b'h' | b'l' => self.private_mode(params, final_byte == b'h'),
            _ => self.unrecognised += 1,
        }
        Some(end + 1)
    }

    /// `OSC 8 ; params ; uri ST` — the only OSC this serializer emits.
    ///
    /// Terminated by `ST` (`ESC \`) or `BEL`, because both are in the wild and a model that accepted
    /// only the one we emit would not notice the other arriving.
    fn parse_osc(&mut self, buf: &[u8], at: usize, tables: &mut Tables) -> Option<usize> {
        let body = at + 2;
        let mut end = body;
        let (payload_end, next) = loop {
            if end >= buf.len() {
                return None;
            }
            if buf[end] == 0x07 {
                break (end, end + 1);
            }
            if buf[end] == 0x1b {
                if end + 1 >= buf.len() {
                    return None;
                }
                if buf[end + 1] == b'\\' {
                    break (end, end + 2);
                }
                // An escape that is not `ST` inside an OSC: the string never terminated.
                self.unrecognised += 1;
                return Some(end);
            }
            end += 1;
        };
        let payload = &buf[body..payload_end];
        // `8` then the parameter list then the URI. The parameters are `id=`-shaped and this
        // serializer emits none, so anything in them is something nobody wrote.
        let mut fields = payload.splitn(3, |b| *b == b';');
        match (fields.next(), fields.next(), fields.next()) {
            (Some(b"8"), Some(b""), Some(uri)) => match std::str::from_utf8(uri) {
                Ok("") => self.link = LinkId::NONE,
                Ok(uri) => self.link = tables.link(uri),
                Err(_) => self.unrecognised += 1,
            },
            _ => self.unrecognised += 1,
        }
        Some(next)
    }

    /// `CSI ? Pm h` and `CSI ? Pm l`.
    fn private_mode(&mut self, params: &[u8], set: bool) {
        let Some(digits) = params.strip_prefix(b"?") else {
            // A non-private `SM`/`RM`. Nothing here emits one.
            self.unrecognised += 1;
            return;
        };
        match first(digits) {
            // DECAWM.
            7 => self.autowrap = set,
            // Synchronised output. One frame is one block, so an open inside an open is a defect.
            2026 => {
                if set {
                    if self.sync_open {
                        self.unrecognised += 1;
                    }
                    self.sync_open = true;
                } else {
                    if !self.sync_open {
                        self.unrecognised += 1;
                    }
                    self.sync_open = false;
                    self.sync_blocks += 1;
                }
            }
            _ => self.unrecognised += 1,
        }
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
            self.reset_style();
            return;
        }
        let mut i = 0;
        while i < parsed.len() {
            let p = &parsed[i];
            let head = p.first().copied().unwrap_or(0);
            match head {
                0 => self.reset_style(),
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
                59 => self.ul = Color::DEFAULT,
                // The three parameterised colour channels, each in **both** spellings: T.416's
                // colon form arrives as sub-parameters of one `;`-separated part, and xterm's
                // pre-ITU-T form as the parts that follow. Which one the serializer sent is
                // `legacy_sgr`'s and `Underlines`' business, and this model reads either — because a
                // model that understood only the form we emit would agree with a wrong one.
                38 | 48 | 58 => {
                    let (colour, used) = if p.len() > 1 {
                        // The colon form: one `;`-separated part, and the whole colour is inside it.
                        (parameterised_colour(&p[1..]), 1)
                    } else {
                        // The pre-ITU-T form, where the channels are parts of their own. **It is
                        // read with its own layout rather than through the matcher above**, because
                        // the two are genuinely ambiguous once flattened: `38;2;r;g;b` and
                        // `38:2::r:g:b` both become five numbers beginning with `2`, and reading one
                        // as the other paints `r` where `g` belongs. What distinguishes them is
                        // which side of a `;` they arrived on, and that is known only here.
                        let sub = |k: usize| parsed.get(i + k).and_then(|v| v.first().copied());
                        match sub(1) {
                            Some(5) => (sub(2).map(|n| Color::indexed(n as u8)), 3),
                            Some(2) => match (sub(2), sub(3), sub(4)) {
                                (Some(r), Some(g), Some(b)) => {
                                    (Some(Color::rgb(r as u8, g as u8, b as u8)), 5)
                                }
                                _ => (None, 1),
                            },
                            _ => (None, 1),
                        }
                    };
                    match colour {
                        Some(c) => {
                            self.style = match head {
                                38 => self.style.fg(c),
                                48 => self.style.bg(c),
                                _ => {
                                    self.ul = c;
                                    self.style
                                }
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

    /// SGR 0, which resets the attributes and all three colours — and **not** the hyperlink.
    ///
    /// An open OSC 8 survives a reset on a real terminal, which is why the serializer closes one
    /// explicitly before a frame ends. A model that cleared the link here would make that close
    /// unnecessary and hide the bug it prevents.
    fn reset_style(&mut self) {
        self.style = Style::DEFAULT;
        self.ul = Color::DEFAULT;
    }

    /// The style word a cell printed now would carry, minted through the engine's own tables.
    ///
    /// A cell is extended **iff** it carries an underline colour or a hyperlink, which is
    /// [`ExtStyle::is_extended`] and is the same predicate `restyle` applies — so an inline cell on
    /// the wire reads back as an inline word and compares in one instruction.
    ///
    /// **The mint is a lookup on every green run.** `ExtStyles::handle` deduplicates, so an entry the
    /// engine already has answers with the engine's own handle and nothing is created; a channel the
    /// engine never wrote mints a handle nobody has and the comparison fails, which is the point.
    fn current_style(&self, tables: &mut Tables) -> Style {
        let e = ExtStyle {
            fg: self.style.foreground(),
            bg: self.style.background(),
            ul: self.ul,
            link: self.link,
        };
        if !e.is_extended() {
            return self.style;
        }
        Style::extended(self.style.attr_word(), tables.exts.handle(e))
    }

    /// Put one cluster in the cell under the cursor, and the pair a wide one occupies.
    ///
    /// A cluster occupying no column is dropped rather than attached to the cell on its left. That
    /// is a simplification of what a terminal does and it is safe here for one reason: the drawing
    /// verbs never put one in a cell, so the serializer never emits one — and if it ever did, the
    /// round trip would say so rather than silently agreeing.
    fn print(&mut self, cluster: &str, tables: &mut Tables) {
        if self.width == 0 || self.height == 0 {
            return;
        }
        let Some(g) = tables.interner.handle(cluster) else {
            return;
        };
        let style = self.current_style(tables);
        let (x, y) = self.cursor;
        let row = y as usize * self.width as usize;
        let columns = match self.widths {
            WidthOpinion::Ours => g.columns(),
            // The disagreement §10's rule is about, and it is one line: this terminal advances by
            // one column over a cluster our tables call two.
            WidthOpinion::Narrow => 1,
        };
        if columns == 2 && x + 1 >= self.width {
            // Half a glyph in the last column. Our serializer writes a space there instead (rule 4),
            // so reaching this is a defect — and it is counted rather than papered over.
            self.unrecognised += 1;
            return;
        }
        self.cells[row + x as usize] = Cell::new(g, style);
        if columns == 2 {
            self.cells[row + x as usize + 1] = Cell::new(GraphemeId::CONTINUATION, style);
        }
        // Auto-wrap is off, so the cursor stops rather than wrapping.
        if x + columns < self.width {
            self.cursor = (x + columns, y);
        } else {
            self.cursor = (self.width - 1, y);
        }
    }
}

/// One `38`/`48`/`58` colour, from the parameters that follow the channel number.
///
/// **The colon form only.** `[5, n]` for a palette entry, `[2, _, r, g, b]` for T.416's empty
/// colour-space id, and `[2, r, g, b]` for the variant that omits it. Anything else is `None`, which
/// the caller counts. The pre-ITU-T spelling is read at the call site, because flattening the two
/// makes them ambiguous.
fn parameterised_colour(sub: &[u32]) -> Option<Color> {
    match sub {
        [5, n, ..] => Some(Color::indexed(*n as u8)),
        // T.416: the colour-space id sits between the `2` and the channels, and is empty.
        [2, _, r, g, b, ..] if sub.len() >= 5 => Some(Color::rgb(*r as u8, *g as u8, *b as u8)),
        [2, r, g, b] => Some(Color::rgb(*r as u8, *g as u8, *b as u8)),
        _ => None,
    }
}

/// The first numeric parameter of a CSI, or zero where there is none.
fn first(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .take_while(|b| b.is_ascii_digit())
        .fold(0u32, |acc, b| acc.saturating_mul(10) + (b - b'0') as u32)
}

/// A column, clamped to the screen the way every terminal clamps it.
fn column(x: u32, width: u16) -> u16 {
    x.min(width.saturating_sub(1) as u32) as u16
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

    /// A set of tables per test. Every scene here is scalars, and a scalar handle is identity in
    /// any table — the shared set is load-bearing only in [`crate::testing::Harness`], where the
    /// model has to produce the *engine's* handle rather than an equal one.
    fn feed(t: &mut TermModel, tables: &mut Tables, bytes: &[u8]) {
        t.feed(bytes, tables);
    }

    fn glyphs(t: &TermModel, y: u16) -> String {
        (0..t.width)
            .map(|x| t.cell(x, y).grapheme.as_scalar().unwrap_or('?'))
            .collect()
    }

    #[test]
    fn printable_bytes_land_at_the_cursor() {
        let mut t = TermModel::new(8, 2);
        let mut i = Tables::new();
        feed(&mut t, &mut i, b"abc");
        assert_eq!(glyphs(&t, 0), "abc     ");
    }

    #[test]
    fn cup_is_one_based() {
        let mut t = TermModel::new(8, 2);
        let mut i = Tables::new();
        feed(&mut t, &mut i, b"\x1b[2;3Hx");
        assert_eq!(glyphs(&t, 1), "  x     ");
    }

    #[test]
    fn sgr_applies_to_what_is_printed_after_it() {
        let mut t = TermModel::new(4, 1);
        let mut i = Tables::new();
        feed(&mut t, &mut i, b"\x1b[1ma\x1b[22mb");
        assert_eq!(t.cell(0, 0).style, Style::new().bold());
        assert_eq!(t.cell(1, 0).style, Style::DEFAULT);
    }

    #[test]
    fn sgr_zero_resets_everything() {
        let mut t = TermModel::new(4, 1);
        let mut i = Tables::new();
        feed(&mut t, &mut i, b"\x1b[1;4:3;38;2;1;2;3m\x1b[0mx");
        assert_eq!(t.cell(0, 0).style, Style::DEFAULT);
    }

    #[test]
    fn an_empty_sgr_is_a_reset() {
        let mut t = TermModel::new(4, 1);
        let mut i = Tables::new();
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
            let mut i = Tables::new();
            feed(&mut t, &mut i, bytes);
            assert_eq!(t.cell(0, 0).style, expected);
            assert_eq!(t.unrecognised(), 0);
        }
    }

    #[test]
    fn a_sequence_split_across_two_feeds_is_reassembled() {
        let mut t = TermModel::new(8, 2);
        let mut i = Tables::new();
        feed(&mut t, &mut i, b"\x1b[2");
        feed(&mut t, &mut i, b";3Hx");
        assert_eq!(glyphs(&t, 1), "  x     ");
        assert_eq!(t.unrecognised(), 0);
    }

    #[test]
    fn a_multi_byte_scalar_split_across_two_feeds_is_reassembled() {
        let mut t = TermModel::new(4, 1);
        let mut i = Tables::new();
        let bytes = "\u{e9}".as_bytes();
        feed(&mut t, &mut i, &bytes[..1]);
        feed(&mut t, &mut i, &bytes[1..]);
        assert_eq!(t.cell(0, 0).grapheme.as_scalar(), Some('\u{e9}'));
    }

    #[test]
    fn the_cursor_stops_at_the_last_column_because_auto_wrap_is_off() {
        let mut t = TermModel::new(3, 2);
        let mut i = Tables::new();
        feed(&mut t, &mut i, b"abcd");
        assert_eq!(glyphs(&t, 0), "abd");
        assert_eq!(glyphs(&t, 1), "   ", "nothing wrapped onto the next row");
    }

    #[test]
    fn an_unknown_sequence_is_counted_rather_than_ignored() {
        let mut t = TermModel::new(4, 1);
        let mut i = Tables::new();
        feed(&mut t, &mut i, b"\x1b[3J");
        assert_eq!(t.unrecognised(), 1);
    }

    /// `CR` and `LF` are `shortest`'s two one-byte moves, and the rest are still nobody's.
    #[test]
    fn cr_and_lf_move_the_cursor_and_no_other_control_is_legal() {
        let mut t = TermModel::new(4, 2);
        let mut i = Tables::new();
        feed(&mut t, &mut i, b"ab\r\nc");
        assert_eq!(t.unrecognised(), 0);
        assert_eq!(glyphs(&t, 0), "ab  ");
        assert_eq!(glyphs(&t, 1), "c   ");

        let mut t = TermModel::new(4, 1);
        feed(&mut t, &mut i, b"\t\x07");
        assert_eq!(t.unrecognised(), 2);
    }

    /// **A terminal scrolls on `LF` from the last row**, and `shortest` never asks for one: a feed is
    /// always a downward move onto a row that exists, because runs arrive in ascending row order. So
    /// the model counts it rather than modelling it — a scroll nobody asked for is a defect.
    #[test]
    fn an_lf_from_the_last_row_is_a_scroll_and_is_counted() {
        let mut t = TermModel::new(4, 2);
        let mut i = Tables::new();
        feed(&mut t, &mut i, b"\x1b[2;1H\n");
        assert_eq!(t.unrecognised(), 1);
    }
}
