//! What a terminal shows, read back from a screen dump.
//!
//! The instrument this belongs to compares a real emulator's screen against what the engine
//! composited. **This half does not talk to a terminal**, and that is deliberate: it parses bytes
//! that were captured once and committed, so `cargo test` verifies the comparison logic with no
//! emulator, no window server and no automation grant in the loop.
//!
//! It is the same trade `fuzz/` makes — the committed corpus is the gate and the live run is the soak
//! — and it is why this is stage 2 of ticket 04 rather than the last thing built. A comparator nobody
//! can run is worth less than one that runs on every commit against bytes a terminal really sent.
//!
//! # What a dump can and cannot say
//!
//! **It cannot say which column a glyph is in.** Both capture formats measured — tmux 3.7c's
//! `capture-pane -p -e` and Ghostty 1.3.1's `write_screen_file:…,vt` — emit a double-width glyph as
//! one cluster with **no padding cell and no continuation marker**, so `AB漢CD` comes back as five
//! clusters and nothing distinguishes it from five narrow ones. Recovering the column would need a
//! width table, and a width table is what the instrument is trying to check.
//!
//! So [`Row::clusters`] is a sequence and **not** a grid, and there is deliberately no
//! `cell_at(column)`. A scene that needs to know what sits at a column puts a unique ASCII sentinel
//! there and asks whether the sentinel survived; that is `conform/SCENES.md`'s scene 02 and its
//! successor. Pretending to a column index here would put the assumption under test inside the
//! instrument.
//!
//! # A parameter is not a token, and flattening the two separators is a defect
//!
//! The first version of this parser split an SGR body on `;` **and** `:` into one flat token stream,
//! because that makes the two colour spellings read through one path. A Ghostty control probe run
//! while building the live arm showed what that costs: `CSI 4:2 m` is *double underline*, and
//! flattened it reads as `4` then `2` — underline **and dim**. An attribute the terminal never
//! rendered, invented by the instrument.
//!
//! ECMA-48 is explicit that `;` separates parameters and `:` separates a parameter's
//! **sub**-parameters, so that is what [`sgr`] does now. The colour arms accept either spelling
//! because they must — that is the question scene 03 asks — but they accept it by looking for the
//! tail in two places, not by pretending the two separators mean the same thing.
//!
//! # The refusal comes first
//!
//! `screen -X hardcopy` was observed to exit 0 and write a **zero-byte file**. Parsed permissively,
//! an empty dump compares equal against a blank region and the row goes green — a missing result
//! hiding inside a passing one, which is the worst form of `compare/`'s *a missing row reads as a
//! win*. [`parse`] therefore takes the number of rows the caller expected and returns
//! [`DumpError`] rather than a short screen.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::fmt;

/// Why a dump was refused. **Never a short screen and never an empty one** — see the module docs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DumpError {
    /// The capture produced nothing. The `screen -X hardcopy` failure mode, and the one this whole
    /// type exists for.
    Empty,
    /// Fewer rows than the scene declared. A capture that raced the paint looks exactly like this,
    /// and it must not be allowed to look like agreement.
    ShortScreen {
        /// What the scene said it would produce.
        expected: usize,
        /// What arrived.
        found: usize,
    },
    /// A CSI sequence ran to the end of the input without its final byte. Truncation, and the same
    /// class of defect as `Empty` one level down.
    UnterminatedEscape,
}

impl fmt::Display for DumpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "the capture was empty — FAILED, never a match"),
            Self::ShortScreen { expected, found } => {
                write!(
                    f,
                    "the capture has {found} rows, the scene declared {expected}"
                )
            }
            Self::UnterminatedEscape => write!(f, "the capture ends inside an escape sequence"),
        }
    }
}

impl std::error::Error for DumpError {}

/// A colour as the dump reported it. **Resolved channels or a palette index, never a handle** — the
/// engine's own colour type is not visible here and could not be compared against one if it were.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Colour {
    /// `39` / `49`: whatever the terminal's default is. The dump's OSC 10/11 header carries the
    /// actual values where an emulator sends one.
    #[default]
    Default,
    /// `30`–`37`, `90`–`97` and the background forms, or `38;5;n`.
    Indexed(u8),
    /// `38;2;r;g;b`, however it was spelled on the way in.
    Rgb(u8, u8, u8),
}

/// How a cell is underlined, numbered as SGR `4:n` numbers it.
///
/// Its own axis rather than a bit in [`Attrs`], and that is not a tidiness choice: the style word
/// the engine serialises from spends **three** of its eleven attribute bits on this field, so a
/// single underline boolean here could not tell a dotted underline from a double one and scene 01
/// needs exactly that distinction to light one bit per row.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Underline {
    /// SGR 24, and the initial state.
    #[default]
    None,
    /// `4` or `4:1`.
    Single,
    /// `4:2`. **Never SGR 21** — ECMA-48 assigns 21 to doubly-underlined and a meaningful
    /// population of terminals implements it as bold-off, so nothing emits it and nothing here
    /// reads it.
    Double,
    /// `4:3`.
    Curly,
    /// `4:4`.
    Dotted,
    /// `4:5`.
    Dashed,
    /// A style number nobody has defined. Kept rather than folded into [`Underline::Single`], so a
    /// terminal inventing one is visible as a disagreement instead of silently becoming a match.
    Other(u8),
}

impl Underline {
    fn from_sgr(n: u8) -> Self {
        match n {
            0 => Self::None,
            1 => Self::Single,
            2 => Self::Double,
            3 => Self::Curly,
            4 => Self::Dotted,
            5 => Self::Dashed,
            n => Self::Other(n),
        }
    }
}

/// The eight attribute flags, as a dump can report them.
///
/// One `u16` rather than eight `bool`s because the comparison is an equality on the whole style and
/// a bitset makes that one instruction. The bit order is this type's own and matches nothing in the
/// engine on purpose: a dump is not a cell.
///
/// **Eight here plus [`Underline`]'s three-bit field is the engine's eleven.** Underline is not a
/// flag; see [`Underline`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Attrs(u16);

impl Attrs {
    /// SGR 1.
    pub const BOLD: Self = Self(1 << 0);
    /// SGR 2.
    pub const DIM: Self = Self(1 << 1);
    /// SGR 3.
    pub const ITALIC: Self = Self(1 << 2);
    /// SGR 5.
    pub const BLINK: Self = Self(1 << 3);
    /// SGR 7.
    pub const REVERSE: Self = Self(1 << 4);
    /// SGR 8. Named for what the SGR does rather than for what the screen shows: a dump reports the
    /// cell's text whether or not the terminal painted it.
    pub const HIDDEN: Self = Self(1 << 5);
    /// SGR 9.
    pub const STRIKE: Self = Self(1 << 6);
    /// SGR 53.
    pub const OVERLINE: Self = Self(1 << 7);

    /// Whether every bit of `other` is set here.
    #[must_use]
    pub fn has(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether nothing is set.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    fn set(&mut self, other: Self) {
        self.0 |= other.0;
    }

    fn clear(&mut self, other: Self) {
        self.0 &= !other.0;
    }
}

/// The style a run of clusters carried.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Style {
    /// Foreground.
    pub fg: Colour,
    /// Background.
    pub bg: Colour,
    /// Underline colour, SGR 58 — **its own axis**, because a terminal can want the semicolon
    /// spelling for 58 and never be asked about 38.
    pub underline_colour: Colour,
    /// How the cell is underlined, SGR 4 and 24.
    pub underline: Underline,
    /// The attribute flags.
    pub attrs: Attrs,
}

/// One grapheme cluster and the style it was wearing.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Cluster {
    /// The cluster's bytes, as the dump gave them.
    pub text: String,
    /// The style in force when it was emitted.
    pub style: Style,
}

/// One row of a dump. **A sequence, not a grid** — see the module docs.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Row {
    /// The clusters, in the order they appeared.
    pub clusters: Vec<Cluster>,
}

impl Row {
    /// The row's text with every style dropped. What a plain-text arm — Terminal.app — can also
    /// produce, so it is the widest comparison the instrument has.
    #[must_use]
    pub fn text(&self) -> String {
        self.clusters.iter().map(|c| c.text.as_str()).collect()
    }
}

/// A parsed dump.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Dump {
    /// The rows.
    pub rows: Vec<Row>,
}

/// Parse a screen dump, refusing an empty or short one.
///
/// `expected_rows` is what the scene declared. It is a parameter rather than a field on a config
/// because the refusal is the point: a caller that does not know how many rows it asked for cannot
/// tell a raced capture from a blank screen.
///
/// # Errors
///
/// [`DumpError::Empty`] for no rows at all, [`DumpError::ShortScreen`] for fewer than declared, and
/// [`DumpError::UnterminatedEscape`] for a truncated CSI.
pub fn parse(bytes: &[u8], expected_rows: usize) -> Result<Dump, DumpError> {
    if bytes.iter().all(|b| matches!(b, b'\n' | b'\r' | b' ')) {
        return Err(DumpError::Empty);
    }

    let mut rows = Vec::new();
    let mut row = Row::default();
    // **The SGR stream is state, not a pair of delimiters.** tmux closes a colour-only run with
    // `ESC[39m` and an attribute-bearing one with `ESC[0m`, so a parser that expects the reset to be
    // one sequence silently carries a style into the following cell.
    let mut style = Style::default();
    let mut pending: Vec<u8> = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            0x1b => {
                flush(&mut row, &mut pending, style);
                let (consumed, next) = escape(&bytes[i..], style)?;
                style = next;
                i += consumed;
            }
            b'\n' => {
                flush(&mut row, &mut pending, style);
                rows.push(std::mem::take(&mut row));
                i += 1;
            }
            b'\r' => i += 1,
            b => {
                pending.push(b);
                i += 1;
            }
        }
    }
    flush(&mut row, &mut pending, style);
    if !row.clusters.is_empty() {
        rows.push(row);
    }

    // Trailing all-blank rows are the terminal's empty screen below the payload, not content. They
    // are dropped *after* the row count is known, so a short capture is still short.
    while rows.last().is_some_and(|r| r.clusters.is_empty()) {
        rows.pop();
    }

    if rows.is_empty() {
        return Err(DumpError::Empty);
    }
    if rows.len() < expected_rows {
        return Err(DumpError::ShortScreen {
            expected: expected_rows,
            found: rows.len(),
        });
    }
    Ok(Dump { rows })
}

/// Break the pending bytes into clusters and give each the style in force.
///
/// **Not UAX #29.** Splitting on `char` boundaries is enough for every scene in `SCENES.md` and
/// pulling in the engine's segmenter would put the code under test inside the instrument — the same
/// reason there is no `cell_at(column)`. A scene that needs real clustering is a scene this function
/// has to grow for, deliberately and with the reason written down.
fn flush(row: &mut Row, pending: &mut Vec<u8>, style: Style) {
    if pending.is_empty() {
        return;
    }
    // Invalid UTF-8 in a dump is a defect in the capture rather than content to be guessed at, so
    // `from_utf8_lossy`'s replacement character is kept and pushed like any other cluster. A
    // comparison against it then fails loudly, where dropping it would quietly shorten the row.
    let text = String::from_utf8_lossy(pending);
    for ch in text.chars() {
        row.clusters.push(Cluster {
            text: ch.to_string(),
            style,
        });
    }
    pending.clear();
}

/// Consume one escape sequence, returning its length and the style after it.
fn escape(bytes: &[u8], style: Style) -> Result<(usize, Style), DumpError> {
    // Only CSI is interpreted. An OSC — the OSC 10/11 default-colour header an emulator may lead
    // with — is skipped to its terminator, because the instrument reads the header separately.
    if bytes.len() < 2 {
        return Err(DumpError::UnterminatedEscape);
    }
    match bytes[1] {
        b'[' => {
            let end = bytes[2..]
                .iter()
                .position(|b| (0x40..=0x7e).contains(b))
                .ok_or(DumpError::UnterminatedEscape)?
                + 2;
            let next = if bytes[end] == b'm' {
                sgr(&bytes[2..end], style)
            } else {
                style
            };
            Ok((end + 1, next))
        }
        b']' => {
            let end = bytes
                .windows(2)
                .position(|w| w == [0x1b, b'\\'])
                .map(|p| p + 2)
                .or_else(|| bytes.iter().position(|b| *b == 0x07).map(|p| p + 1))
                .ok_or(DumpError::UnterminatedEscape)?;
            Ok((end, style))
        }
        _ => Ok((2, style)),
    }
}

/// Apply one SGR sequence's parameters to the style.
///
/// **`;` separates parameters and `:` separates sub-parameters**, and the difference is load-bearing:
/// `4:2` is one parameter meaning double underline, and reading it as two parameters produces
/// underline plus dim — an attribute the terminal never rendered. See the module docs.
///
/// Both colour spellings are accepted, which is not a convenience: the scene that asks whether a
/// terminal parses ITU-T T.416's colon form has to be able to read either answer back. They are
/// accepted by looking for the tail in the two places it can be — inside this parameter's
/// sub-parameters, or in the parameters that follow — never by flattening the separators.
fn sgr(params: &[u8], mut style: Style) -> Style {
    let text = String::from_utf8_lossy(params);
    // An empty body is `CSI m`, which ECMA-48 defines as `CSI 0 m`.
    if text.is_empty() {
        return Style::default();
    }
    let parameters: Vec<Vec<&str>> = text.split(';').map(|p| p.split(':').collect()).collect();
    let mut k = 0;
    while k < parameters.len() {
        let parameter = &parameters[k];
        let code: u16 = parameter[0].parse().unwrap_or(0);
        match code {
            0 => style = Style::default(),
            1 => style.attrs.set(Attrs::BOLD),
            2 => style.attrs.set(Attrs::DIM),
            3 => style.attrs.set(Attrs::ITALIC),
            // The one parameter with a sub-parameter that is not a colour. Bare `4` is single;
            // `4:n` is the style the engine's three-bit underline field spells.
            4 => {
                style.underline = match parameter.get(1) {
                    Some(n) => Underline::from_sgr(n.parse().unwrap_or(1)),
                    None => Underline::Single,
                }
            }
            5 => style.attrs.set(Attrs::BLINK),
            7 => style.attrs.set(Attrs::REVERSE),
            8 => style.attrs.set(Attrs::HIDDEN),
            9 => style.attrs.set(Attrs::STRIKE),
            22 => style.attrs.clear(Attrs(Attrs::BOLD.0 | Attrs::DIM.0)),
            23 => style.attrs.clear(Attrs::ITALIC),
            24 => style.underline = Underline::None,
            25 => style.attrs.clear(Attrs::BLINK),
            27 => style.attrs.clear(Attrs::REVERSE),
            28 => style.attrs.clear(Attrs::HIDDEN),
            29 => style.attrs.clear(Attrs::STRIKE),
            30..=37 => style.fg = Colour::Indexed((code - 30) as u8),
            90..=97 => style.fg = Colour::Indexed((code - 90 + 8) as u8),
            40..=47 => style.bg = Colour::Indexed((code - 40) as u8),
            100..=107 => style.bg = Colour::Indexed((code - 100 + 8) as u8),
            39 => style.fg = Colour::Default,
            49 => style.bg = Colour::Default,
            53 => style.attrs.set(Attrs::OVERLINE),
            55 => style.attrs.clear(Attrs::OVERLINE),
            59 => style.underline_colour = Colour::Default,
            38 | 48 | 58 => {
                // The colon form carries its tail inside this parameter; the semicolon form carries
                // it in the parameters after it, each of which is a one-element sub-parameter list.
                let colour = if parameter.len() > 1 {
                    parameterised(&parameter[1..]).0
                } else {
                    let tail: Vec<&str> = parameters[k + 1..]
                        .iter()
                        .map(|p| p.first().copied().unwrap_or(""))
                        .collect();
                    let (colour, used) = parameterised(&tail);
                    k += used;
                    colour
                };
                match code {
                    38 => style.fg = colour,
                    48 => style.bg = colour,
                    _ => style.underline_colour = colour,
                }
            }
            _ => {}
        }
        k += 1;
    }
    style
}

/// Read a `5;n` or `2;r;g;b` tail, tolerating T.416's empty colour-space id.
///
/// The empty id is what makes the colon spelling one byte longer than the semicolon one, and it
/// arrives here as an empty token. Skipping empties is therefore the whole of what lets one function
/// read both a sub-parameter list and a run of following parameters.
fn parameterised(tail: &[&str]) -> (Colour, usize) {
    let mut nums = Vec::new();
    let mut used = 0;
    for t in tail {
        used += 1;
        if t.is_empty() {
            continue; // T.416's colour-space id
        }
        nums.push(t.parse::<u16>().unwrap_or(0));
        // `5` takes one argument, `2` takes three. Stop as soon as enough have arrived, so a
        // following SGR code is not eaten.
        match nums.first() {
            Some(5) if nums.len() == 2 => break,
            Some(2) if nums.len() == 4 => break,
            _ => {}
        }
    }
    let colour = match nums.as_slice() {
        [5, n] => Colour::Indexed(*n as u8),
        [2, r, g, b] => Colour::Rgb(*r as u8, *g as u8, *b as u8),
        _ => Colour::Default,
    };
    (colour, used)
}

/// The terminal's own default foreground and background, from the `vt` dump's OSC 10/11 header.
///
/// **Not decoration.** A scene that writes with [`Colour::Default`] can only be checked against what
/// the emulator resolves that to, and the emulator is the only party that knows. Ghostty leads every
/// dump with the pair; a capture without one returns `None` for that side rather than the xterm
/// guess, because guessing here would be the instrument inventing the answer it came to measure.
///
/// Returns `(foreground, background)`.
#[must_use]
pub fn default_colours(bytes: &[u8]) -> (Option<Colour>, Option<Colour>) {
    let text = String::from_utf8_lossy(bytes);
    let read = |code: &str| {
        let at = text.find(&format!("\x1b]{code};rgb:"))?;
        // `\x1b` `]` code `;rgb:` — two chars, the code, then five.
        let rest = &text[at + code.len() + 7..];
        let spec = rest.split(['\x1b', '\x07']).next()?;
        let mut channels = spec.split('/').map(|c| {
            // `rgb:ea/ea/ea` is 8-bit per channel; the X spec also allows 4 and 16. Scale from
            // whatever width arrived rather than assuming two hex digits.
            let value = u32::from_str_radix(c, 16).ok()?;
            let max = (1u32 << (4 * c.len() as u32)) - 1;
            Some((value * 255 / max) as u8)
        });
        match (channels.next()?, channels.next()?, channels.next()?) {
            (Some(r), Some(g), Some(b)) => Some(Colour::Rgb(r, g, b)),
            _ => None,
        }
    };
    (read("10"), read("11"))
}

#[cfg(test)]
mod tests;
