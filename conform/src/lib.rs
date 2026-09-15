//! What a terminal shows, read back from a screen dump.
//!
//! The instrument this belongs to compares a real emulator's screen against what the engine
//! composited. **This half does not talk to a terminal**, and that is deliberate: it parses bytes
//! that were captured once and committed, so `cargo test` verifies the comparison logic with no
//! emulator, no window server and no automation grant in the loop.
//!
//! It is the same trade `fuzz/` makes — the committed corpus is the gate and the live run is the soak
//! — and it is why this is stage 2 of an earlier pass rather than the last thing built. A comparator nobody
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
//! # A capture format is a dialect, and two of them are not the same language
//!
//! [`parse`] takes a [`Dialect`] because the two capture formats this instrument reads **disagree
//! about what a colon means**, and reading one as the other invents attributes.
//!
//! Ghostty's `vt` dump is ECMA-48: `4:2` is parameter 4 with sub-parameter 2, double underline.
//! tmux's `capture-pane -e` is not. `grid.c`'s `grid_string_cells_code` writes every attribute code
//! it knows, and then:
//!
//! ```c
//! if (s[i] < 10)
//!         xsnprintf(tmp, sizeof tmp, "%d", s[i]);
//! else
//!         xsnprintf(tmp, sizeof tmp, "%d:%d", s[i] / 10, s[i] % 10);
//! ```
//!
//! **A colon there is a divided-by-ten, not a sub-parameter.** tmux's own codes for the underline
//! styles are 42–45, chosen so that the division lands on ECMA-48's `4:2`–`4:5` — the two readings
//! agree, and that is why one parser got away with it. **Overline is 53, and it rides the same
//! branch**: it comes back as `5:3`, which in ECMA-48 is *blink*.
//!
//! So a parser told nothing reports blink for a cell tmux is holding an overline in — the exact
//! defect `FINDINGS.md` records for `4:2` a stage earlier, and the reason the dialect is a required
//! parameter rather than a default. A caller that does not know which format it captured cannot be
//! trusted to have captured either.
//!
//! **A third capture format was asked this question and answered no.** kitty 0.48.2's
//! `kitten @ get-text --extent screen --ansi` was assumed to be a third dialect until it was probed,
//! because the tmux lesson is that assuming otherwise is how the instrument invents an attribute. It
//! is not one: every construct it emits is ECMA-48 and means what ECMA-48 says it means — a leading
//! `CSI m` per row, `CSI 22 ; 1 m` for bold, `4:2` and `4:3` for the underline styles, and the colon
//! colour forms without T.416's empty colour-space id. Nothing needs a fold, so kitty is
//! [`Dialect::Ecma48`] and the enum does not grow. **A dialect is a disagreement about meaning, not a
//! difference in style**, and the difference is worth a paragraph because only one of the two costs
//! an attribute.
//!
//! **A fourth capture format was asked and answered no as well, and it cost two arms rather than a
//! dialect.** WezTerm's `wezterm cli get-text --escapes` re-serialises its own grid into a
//! **classic** SGR repertoire — a sub-parameter is normalised away (`4:1` comes back as bare `4`),
//! a double underline is spelled ECMA-48's `21`, and `4:3`, `4:4`, `4:5`, `53` and `58` come back
//! as nothing at all — and every construct it *does* emit means what ECMA-48 says. So it is
//! [`Dialect::Ecma48`] too, and what it needed was two arms in `sgr` and `escape`: **`ESC ( B`** is
//! three bytes and the two-byte fallback left the `B` as content, and **SGR 21** had never been
//! reached because the engine always writes `4:n`.
//!
//! **One hazard comes with that second arm and is left standing deliberately.** xterm and much of
//! its family read 21 as *bold off*, so a capture format spelling it that way would be parsed here
//! as bold-still-set plus a double underline — an invented attribute, which is what this enum
//! exists to stop. It is not conditioned on a dialect because a variant with no probed capture
//! behind it is a branch nothing can test; `sgr`'s own comment says so, and `tests.rs` asserts that
//! no other committed fixture contains `CSI 21 m`, so the day one does an arm is being added.
//!
//! # A lossy spelling is not a dropped attribute, and the difference decides a `quirks.rs` row
//!
//! kitty writes a **dotted** underline as `CSI 4 : m` — parameter 4 with an *empty* sub-parameter —
//! and a dashed one identically. Its serialiser's table has an entry for `4:2` and `4:3` and none
//! for the other two; both strings are in the shipped `kitty.fast_data_types.so` and neither `4:4;`
//! nor `4:5;` is.
//!
//! ECMA-48 says an omitted parameter is the default, and SGR 4's default is 1, so `sgr` reads
//! `4:` as [`Underline::Single`] — the correct reading of what arrived, and **not** what kitty is
//! holding. That distinction is the whole point: a comparison run against this row would report
//! *kitty does not render dotted underlines*, which is false, and would earn a `quirks.rs` entry the
//! table exists to keep out.
//!
//! What separates the two here is a control the emulator itself provides: `4:0` comes back as
//! **nothing at all**, so a `4:` in the capture proves the cell holds a *non-zero* decoration and
//! only its number was lost. So the row is unanswerable rather than failed, and an arm says which of
//! its rows are unanswerable **in advance and with the reason** — see `conform/SCENES.md`, which had
//! to grow a fourth kind of non-number to say it.
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
//! **sub**-parameters, so that is what `sgr` does now. The colour arms accept either spelling
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
use std::path::Path;

mod buffer;
mod grid;

/// Which serialisation a capture is written in. See the module docs — this is not a formatting
/// preference, the first two disagree about what a colon means and the third is not an escape
/// stream at all.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dialect {
    /// ECMA-48, as a terminal emits it: `;` separates parameters and `:` separates a parameter's
    /// sub-parameters. Ghostty's `write_screen_file:…,vt`, and **kitty's
    /// `kitten @ get-text --ansi`**, which was probed for a dialect of its own and had none — see
    /// the module docs.
    Ecma48,
    /// tmux's `capture-pane -e`, where a colon is `code / 10` and `code % 10` rather than a
    /// sub-parameter.
    ///
    /// The fold is deliberately **not** applied to 38, 48 and 58: those carry a real ITU-T T.416
    /// tail, and tmux was observed to write all three with semicolons anyway
    /// (`fixtures/tmux-3.7c-attrs-and-colours.vt`), so nothing is lost by refusing to guess there
    /// and a genuine colon colour would survive unmangled.
    TmuxCapturePane,
    /// `alacritty --ref-test`'s `grid.json`: the `Term`'s own grid, one JSON object per cell, with
    /// no SGR anywhere in the path.
    ///
    /// **A serialisation and not a dialect of the other two**, which is why it is on this enum
    /// rather than beside it: [`parse`]'s callers ask *which capture format is this*, and an arm
    /// that could not answer could not be trusted to have captured either. See `src/grid.rs` for
    /// what it buys — no emulator serialiser between the cell and the reader — and what it costs:
    /// a grid is what the terminal **stores**.
    AlacrittyGrid,
    /// iTerm2's Python API answering a `GetBufferRequest`: a protobuf `GetBufferResponse`, one
    /// `LineContents` per row, with one `CellStyle` per run of cells.
    ///
    /// **The second capture here that is not an escape stream, and it is not the third dialect
    /// either** — see `src/buffer.rs`. What it shares with [`Dialect::AlacrittyGrid`] is that no
    /// serialiser of the emulator's stands between the cell and the reader; what it does not share
    /// is that a `CellStyle` is a *projection* of iTerm2's own `screen_char_t` rather than the
    /// struct itself, so a bit the cell holds and the projection drops is a reading this format has
    /// and a grid does not.
    Iterm2Buffer,
}

impl Dialect {
    /// The extension a capture in this serialisation is filed under.
    ///
    /// **Named apart for the reason `.cpr` and `.decrqm` are.** Handing a grid to the SGR parser
    /// produces a refusal rather than a wrong number, and a reader should not have to open a file
    /// to find out which it is.
    #[must_use]
    pub fn capture_ext(self) -> &'static str {
        match self {
            Self::Ecma48 | Self::TmuxCapturePane => "vt",
            Self::AlacrittyGrid => "json",
            // **Not `.json` and not `.vt`.** A `GetBufferResponse` is protobuf: a reader that opened
            // it expecting either would find neither, and the rule this method exists for is that a
            // capture handed to the wrong reader produces a refusal rather than a number.
            Self::Iterm2Buffer => "pb",
        }
    }
}

impl Dialect {
    /// Fold one parameter's colon form back into the code it stands for, where this dialect says a
    /// colon is arithmetic. `None` leaves the parameter to be read as ECMA-48 wrote it.
    fn fold(self, parameter: &[&str]) -> Option<u16> {
        if self != Self::TmuxCapturePane || parameter.len() != 2 {
            return None;
        }
        let tens: u16 = parameter[0].parse().ok()?;
        let units: u16 = parameter[1].parse().ok()?;
        // One digit each, which is all the `%d:%d` above can produce.
        if !(1..=9).contains(&tens) || units > 9 {
            return None;
        }
        match tens * 10 + units {
            // The three that carry a real T.416 tail, so a colon after them is a sub-parameter and
            // not arithmetic. `3:8`, `4:8` and `5:8` are the only spellings this can reach.
            38 | 48 | 58 => None,
            code => Some(code),
        }
    }
}

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
    /// The capture is in a structured format and is not the shape this reader knows.
    ///
    /// **Only the two structured formats can produce one** — [`Dialect::AlacrittyGrid`] and
    /// [`Dialect::Iterm2Buffer`] — and it is a refusal rather than a tolerated field: a terminal that
    /// grows an attribute flag, renames one, or serialises a colour the renderer resolved must arrive
    /// as a failed run. A reader that skipped what it did not recognise would report a terminal that
    /// stopped doing something.
    Malformed(String),
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
            Self::Malformed(why) => {
                write!(f, "the capture is not the shape this reader knows: {why}")
            }
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
    /// Nothing set. [`Attrs::default`]'s value as a constant, because a table of expectations wants
    /// to be a `const` and `Default::default` is not callable in one.
    pub const NONE: Self = Self(0);
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
/// `dialect` is which capture format these bytes are, and it has no default for the same reason. See
/// the module docs: the two formats disagree about what a colon means, and a caller that cannot say
/// which one it captured cannot be trusted to have captured either.
///
/// # Errors
///
/// [`DumpError::Empty`] for no rows at all, [`DumpError::ShortScreen`] for fewer than declared, and
/// [`DumpError::UnterminatedEscape`] for a truncated CSI.
pub fn parse(bytes: &[u8], expected_rows: usize, dialect: Dialect) -> Result<Dump, DumpError> {
    // Dispatched before the emptiness check below, which is about an escape stream: a grid is
    // refused as empty by its own reader, on the screen rather than on the bytes.
    if dialect == Dialect::AlacrittyGrid {
        return grid::parse(bytes, expected_rows);
    }
    if dialect == Dialect::Iterm2Buffer {
        return buffer::parse(bytes, expected_rows);
    }
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
                let (consumed, next) = escape(&bytes[i..], style, dialect)?;
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
fn escape(bytes: &[u8], style: Style, dialect: Dialect) -> Result<(usize, Style), DumpError> {
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
                sgr(&bytes[2..end], style, dialect)
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
        // **A charset designation is three bytes and carries no style.** `ESC ( B` designates
        // US-ASCII as G0 and WezTerm's `get-text --escapes` leads every row with one; the
        // two-byte fallback below consumed `ESC (` and left the `B` to be read as content, so
        // every row of that arm's scene 01 arrived as `Bbold`. The whole SCS family is listed
        // because a capture with one designator may carry any of them and none of them is a glyph.
        b'(' | b')' | b'*' | b'+' | b'-' | b'.' | b'/' if bytes.len() >= 3 => Ok((3, style)),
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
fn sgr(params: &[u8], mut style: Style, dialect: Dialect) -> Style {
    let text = String::from_utf8_lossy(params);
    // An empty body is `CSI m`, which ECMA-48 defines as `CSI 0 m`.
    if text.is_empty() {
        return Style::default();
    }
    let parameters: Vec<Vec<&str>> = text.split(';').map(|p| p.split(':').collect()).collect();
    let mut k = 0;
    while k < parameters.len() {
        let parameter = &parameters[k];
        // **The dialect decides what the colon was.** In a tmux capture `5:3` is the number 53 and
        // in an ECMA-48 one it is parameter 5 with a sub-parameter — blink either way if nobody asks.
        let folded = dialect.fold(parameter);
        let code: u16 = folded.unwrap_or_else(|| parameter[0].parse().unwrap_or(0));
        let subs: &[&str] = match folded {
            Some(_) => &[],
            None => &parameter[1..],
        };
        match code {
            0 => style = Style::default(),
            1 => style.attrs.set(Attrs::BOLD),
            2 => style.attrs.set(Attrs::DIM),
            3 => style.attrs.set(Attrs::ITALIC),
            // The one parameter with a sub-parameter that is not a colour. Bare `4` is single;
            // `4:n` is the style the engine's three-bit underline field spells.
            4 => {
                style.underline = match subs.first() {
                    // **An empty sub-parameter is ECMA-48's omitted one, and SGR 4's default is 1.**
                    // Not a tolerance for garbage: kitty 0.48.2 writes exactly `CSI 4 : m` for a
                    // dotted underline and for a dashed one, because its serialiser has a string for
                    // `4:2` and `4:3` and none for the other two. Reading it as single is the right
                    // reading of what arrived and the wrong description of what kitty is holding,
                    // which is why the arm declares that row unanswerable rather than failing it.
                    None | Some(&"") => Underline::Single,
                    Some(n) => Underline::from_sgr(n.parse().unwrap_or(1)),
                }
            }
            // tmux's own numbering for the underline styles, and **the guard is the whole of why
            // this is safe**: 42–45 sit inside ECMA-48's 40–47 background colours, so without
            // `folded.is_some()` this arm would shadow them and a green background would become a
            // double underline. It is reachable only from a colon that
            // [`Dialect::TmuxCapturePane`] folded — a bare `CSI 42 m` is a background colour
            // everywhere, tmux included.
            42..=45 if folded.is_some() => {
                style.underline = Underline::from_sgr((code - 40) as u8);
            }
            5 => style.attrs.set(Attrs::BLINK),
            7 => style.attrs.set(Attrs::REVERSE),
            8 => style.attrs.set(Attrs::HIDDEN),
            9 => style.attrs.set(Attrs::STRIKE),
            // **ECMA-48 spends 21 on *doubly underlined***, and 22 on normal intensity. *Bold
            // off* is xterm's reading of 21 and a deviation from the standard this parser is
            // named after. Nothing needed the arm while every capture spelled the style `4:2`;
            // WezTerm's normalises the sub-parameter away and writes `21`.
            //
            // **The hazard, and why it is not conditioned on the dialect.** A future arm whose
            // serialiser writes *bold off* as 21 would be parsed here as bold-still-set **plus** a
            // double underline the terminal never drew — the instrument inventing an attribute,
            // which is the whole thing [`Dialect`] exists to prevent. It is left unconditional
            // because a `Dialect` variant with no capture behind it is a branch nothing can test:
            // the enum's own rule is that a dialect is added when a capture is **probed** and found
            // to disagree, and none here does. What stands in for a gate is
            // `the_wezterm_capture_is_written_in_two_constructs_no_other_arm_sent`, which asserts
            // no other committed fixture contains `CSI 21 m` — so the day one does, an arm is
            // being added and this paragraph is what its author has to read. See the module docs.
            21 => style.underline = Underline::Double,
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
                let colour = if !subs.is_empty() {
                    parameterised(subs).0
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

// ── Scene 05: the terminal's own answer to `CSI 6n`, which is not a screen ───────────────────────

/// One `CSI r ; c R` the terminal sent back, exactly as it reported it.
///
/// **No arithmetic.** The scene homes the cursor to column 1, writes one cluster and asks; the
/// advance is `column - 1`, and that subtraction is the caller's one line rather than a step inside
/// the instrument. What this type carries is what arrived.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Reply {
    /// The row the terminal says the cursor is on, one-based.
    pub row: u16,
    /// The column the terminal says the cursor is on, one-based.
    pub column: u16,
}

/// Why a batch of cursor reports is not a measurement.
///
/// **The refusals come first here for the reason they came first for the dump**, and the shape of
/// the accident is the same one an earlier pass predicted: a missing answer that reads as agreement.
/// `screen -X hardcopy` exits 0 and writes a zero-byte file; a terminal that does not implement DSR
/// answers nothing at all, and a survey with no rows in it prints as a clean survey.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum CprError {
    /// No device-attributes reply, so nothing says the terminal finished with the batch.
    ///
    /// **This is the one that separates *slow* from *silent*.** A read that stopped early has some
    /// replies and no sentinel; a terminal that does not answer DSR has the sentinel and no
    /// replies. Reported as different errors because they have different causes and only one of
    /// them is a fact about the terminal.
    NoSentinel,
    /// Fewer — or more — cursor reports before the sentinel than the scene asked for.
    Count {
        /// How many clusters the scene probed.
        expected: usize,
        /// How many answers arrived before the sentinel.
        found: usize,
    },
    /// Two replies name two different rows.
    ///
    /// The scene homes the cursor to one row before every cluster, so this is a screen that
    /// scrolled or a cluster that wrapped — and a column measured on a row the scene did not write
    /// is not a measurement of anything.
    RowMoved {
        /// The row the first reply named.
        first: u16,
        /// The row that disagreed with it.
        then: u16,
    },
    /// A `CSI` began and the bytes ran out before it ended.
    UnterminatedReply,
}

impl fmt::Display for CprError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSentinel => write!(
                f,
                "no device-attributes reply, so nothing says the terminal finished with the batch \
                 — the read gave up before the answers arrived, or they never will"
            ),
            Self::Count { expected, found } => write!(
                f,
                "the scene probed {expected} clusters and {found} cursor reports came back before \
                 the sentinel — a batch that lost an answer cannot say which cluster the rest \
                 belong to"
            ),
            Self::RowMoved { first, then } => write!(
                f,
                "the replies name row {first} and then row {then}; the scene writes every cluster \
                 on one row, so the screen scrolled or a cluster wrapped and no column here is a \
                 width"
            ),
            Self::UnterminatedReply => {
                write!(
                    f,
                    "a control sequence began and the bytes ran out before it ended"
                )
            }
        }
    }
}

impl std::error::Error for CprError {}

/// The terminal's answers to a batch of `CSI 6n`, refused unless there are exactly `expected` of
/// them on one row with a device-attributes reply behind them.
///
/// # This is the one instrument here with none of the engine's tables in the loop
///
/// Every other comparison in this directory reads a *dump*, which is a terminal re-serialising its
/// own grid — and a grid-to-text dump emits no padding cell for a double-width glyph, so *how many
/// columns did that cluster take* is not a question it can answer at all (see the module docs, and
/// `SCENES.md`'s scene 02, which is kept for being unable to answer it).
///
/// A cursor report can. Print a cluster at a known column, ask, and the number that comes back is
/// **the emulator's own UAX #11 verdict**, arrived at by the emulator's tables and reported by the
/// emulator. Nothing in this repository is in that path.
///
/// # The sentinel, and why it is not a timeout
///
/// The batch ends with `CSI c`, whose reply the terminal cannot send before it has processed
/// everything ahead of it. So the read stops on an **observed** condition rather than on a delay
/// tuned until it passed — the same discipline as the scene's quiescence handshake, and the shape
/// `detect.rs` established one crate over.
///
/// # Errors
///
/// [`CprError`], and the four of them say four different things. See the type.
pub fn cursor_reports(bytes: &[u8], expected: usize) -> Result<Vec<Reply>, CprError> {
    let mut replies: Vec<Reply> = Vec::new();
    let mut sentinel = false;
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != 0x1b {
            i += 1;
            continue;
        }
        // `ESC` `[` then parameter and intermediate bytes, then one final byte in `@`..=`~`.
        let Some(b'[') = bytes.get(i + 1) else {
            i += 1;
            continue;
        };
        let mut end = i + 2;
        while end < bytes.len() && !(0x40..=0x7e).contains(&bytes[end]) {
            end += 1;
        }
        if end == bytes.len() {
            return Err(CprError::UnterminatedReply);
        }
        let body = &bytes[i + 2..end];
        match bytes[end] {
            // The device-attributes reply. Everything after it belongs to some other question.
            b'c' => {
                sentinel = true;
                break;
            }
            b'R' if !body.starts_with(b"?") => {
                let text = String::from_utf8_lossy(body);
                let mut parts = text.split(';');
                let row = parts.next().and_then(|p| p.parse().ok());
                let column = parts.next().and_then(|p| p.parse().ok());
                if let (Some(row), Some(column)) = (row, column) {
                    replies.push(Reply { row, column });
                }
            }
            _ => {}
        }
        i = end + 1;
    }

    if !sentinel {
        return Err(CprError::NoSentinel);
    }
    if replies.len() != expected {
        return Err(CprError::Count {
            expected,
            found: replies.len(),
        });
    }
    // Checked after the count, because a batch that lost a reply is the more likely cause of a row
    // that moved and is the more useful thing to be told.
    //
    // `first` and not `replies[0]`: a caller may legitimately ask for none — and a sentinel with no
    // replies behind it satisfies the count above, so an index here panics on a capture that is
    // otherwise perfectly well formed.
    let Some(first) = replies.first().map(|r| r.row) else {
        return Ok(replies);
    };
    if let Some(moved) = replies.iter().find(|r| r.row != first) {
        return Err(CprError::RowMoved {
            first,
            then: moved.row,
        });
    }
    Ok(replies)
}

// ── Scene 06: mode 2026, which the terminal answers about itself ─────────────────────────────────

/// What a DECRQM reply says about one mode, by the numbers DEC gave those answers.
///
/// **These are the standard's own values and not this repository's policy**, which is what makes
/// scene 06's first five rows a comparison where scene 05's twelve are a survey: a terminal that
/// reports a mode *set* after it was asked to reset it is wrong by the definition of the reply it
/// sent, not by a table we chose.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModeState {
    /// `0` — the terminal does not recognise the mode. A legitimate answer, and the one a terminal
    /// with no synchronised output gives; see `SCENES.md` for what an arm then owes.
    NotRecognised,
    /// `1` — set.
    Set,
    /// `2` — reset.
    Reset,
    /// `3` — permanently set, so nothing the scene sends can change it.
    PermanentlySet,
    /// `4` — permanently reset, so nothing the scene sends can change it.
    PermanentlyReset,
}

impl ModeState {
    /// The reply's second parameter, as a state.
    ///
    /// `None` for a number DEC never defined, which is refused rather than guessed at: a state this
    /// instrument invented would be a state a report printed.
    #[must_use]
    pub fn from_ps(ps: u16) -> Option<Self> {
        match ps {
            0 => Some(Self::NotRecognised),
            1 => Some(Self::Set),
            2 => Some(Self::Reset),
            3 => Some(Self::PermanentlySet),
            4 => Some(Self::PermanentlyReset),
            _ => None,
        }
    }

    /// The `ps` this state came from, for a report that wants the wire value beside the word.
    #[must_use]
    pub fn ps(self) -> u16 {
        match self {
            Self::NotRecognised => 0,
            Self::Set => 1,
            Self::Reset => 2,
            Self::PermanentlySet => 3,
            Self::PermanentlyReset => 4,
        }
    }
}

impl fmt::Display for ModeState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let word = match self {
            Self::NotRecognised => "not recognised",
            Self::Set => "set",
            Self::Reset => "reset",
            Self::PermanentlySet => "permanently set",
            Self::PermanentlyReset => "permanently reset",
        };
        write!(f, "{word} ({})", self.ps())
    }
}

/// One `CSI ? mode ; ps $ y` the terminal sent back.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ModeReply {
    /// The mode the terminal says it is answering about. Carried rather than assumed — see
    /// [`ModeError::OtherMode`].
    pub mode: u16,
    /// What it says about it.
    pub state: ModeState,
}

/// Why a batch of DECRQM replies is not a measurement.
///
/// **The refusals come first here for the third time in this crate**, and the accident is the same
/// one every time: an answer that never arrived, read as an answer. A missing reply in this scene
/// would not read as a blank cell — it would shorten the batch, and a shortened batch read
/// positionally reports *the state after the close* under the heading *the state while open*. Every
/// row of the table would be one question out and every one of them would look like a number.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ModeError {
    /// No device-attributes reply, so nothing says the terminal finished with the batch.
    NoSentinel,
    /// The sentinel arrived and **nothing came before it**: the terminal processed the whole batch
    /// and said not one word about the mode.
    ///
    /// # Not a short batch, and the difference is the whole finding
    ///
    /// [`ModeError::Count`] is *the terminal lost an answer*, which is a defect in the run. This is
    /// *the terminal does not answer this question at all*, which is a fact about the terminal — and
    /// the sentinel is what makes it one rather than a timeout: a device-attributes reply cannot be
    /// sent before everything ahead of it has been processed, so silence in front of it is silence
    /// the terminal chose.
    ///
    /// DEC's own answer for a mode a terminal does not have is `not recognised (0)`, and the four
    /// arms that existed before this variant all *have* mode 2026 — so none of them ever produced
    /// it, and a terminal without the mode was a case this reader had only ever been described.
    /// **Terminal.app 2.15 answers nothing at all**: its parser does not take `$` as an
    /// intermediate byte, so
    /// `CSI ? 2026 $ p` is not a query it declines — it is a sequence it never finishes reading, and
    /// the `p` lands on the screen as text. An instrument that read that as a short batch would
    /// report a lost answer about a terminal that never had one to lose.
    ///
    /// **Silence, and not merely no answers of the right shape.** A capture from another channel —
    /// scene 05's fifteen cursor reports, say — also has a sentinel and no mode reports, and that is
    /// [`ModeError::Count`] rather than this: the terminal spoke, and what it said was an answer to
    /// a different question. Collapsing the two would let a `.cpr` file pass as a terminal without
    /// synchronised output.
    Unanswered {
        /// How many questions the scene asked and got no answer to.
        expected: usize,
    },
    /// Fewer — or more — mode reports before the sentinel than the scene asked for.
    Count {
        /// How many questions the scene asked.
        expected: usize,
        /// How many answers arrived before the sentinel.
        found: usize,
    },
    /// A reply about a mode the scene never asked about.
    ///
    /// **Not ignored, refused.** The scene asks about one mode and counts the answers positionally,
    /// so a stranger's reply in the stream does not merely add a row — it shifts every row after it
    /// on to the wrong question.
    OtherMode {
        /// The mode the scene asked about.
        wanted: u16,
        /// The mode a reply named instead.
        then: u16,
    },
    /// A second parameter DEC never defined. See [`ModeState::from_ps`].
    UnknownState {
        /// The number that arrived where a state belongs.
        ps: u16,
    },
    /// A private-mode DECRPM reply whose two numbers are not two numbers.
    ///
    /// **Refused rather than skipped**, and the difference is the diagnosis. Dropped, it would
    /// arrive downstream as a short [`ModeError::Count`] — *the terminal lost an answer* about a
    /// terminal that answered every question and spelled one of them in a way this reader does not
    /// know. The two have different causes and only one of them is a fact about the terminal.
    Malformed {
        /// The reply's parameter bytes, as they arrived.
        body: String,
    },
    /// A `CSI` began and the bytes ran out before it ended.
    UnterminatedReply,
}

impl fmt::Display for ModeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSentinel => write!(
                f,
                "no device-attributes reply, so nothing says the terminal finished with the batch \
                 — the read gave up before the answers arrived, or they never will"
            ),
            Self::Count { expected, found } => write!(
                f,
                "the scene asked {expected} questions and {found} mode reports came back before \
                 the sentinel — the answers are counted positionally, so a batch that lost one \
                 reports every later row under the wrong question"
            ),
            Self::OtherMode { wanted, then } => write!(
                f,
                "a reply names mode {then} where every question was about mode {wanted}; the \
                 stranger shifts every answer after it on to the wrong question"
            ),
            Self::UnknownState { ps } => write!(
                f,
                "a reply carries state {ps}, which DECRPM does not define — a state this \
                 instrument invented is a state a report would print"
            ),
            Self::Malformed { body } => write!(
                f,
                "a private-mode reply reads `CSI {body} y` and its two parameters are not two \
                 numbers; dropped it would arrive as a lost answer, which is a different cause"
            ),
            Self::Unanswered { expected } => write!(
                f,
                "the sentinel arrived with nothing in front of it, so the terminal processed all \
                 {expected} questions and answered none of them — it does not have this mode's \
                 report, which is a fact about the terminal and not a lost answer"
            ),
            Self::UnterminatedReply => write!(
                f,
                "a control sequence began and the bytes ran out before it ended"
            ),
        }
    }
}

impl std::error::Error for ModeError {}

/// The terminal's answers to a batch of `CSI ? mode $ p`, refused unless there are exactly
/// `expected` of them about that mode with a device-attributes reply behind them.
///
/// # This asks the terminal what `quirks.rs` reads out of a source tree
///
/// Every row of `quirks.rs`'s synchronised-output table has the same provenance — *the
/// implementation, read* — because a force flush is a **rendering** event and nothing a process
/// inside a terminal can ask reports whether the terminal painted. That sentence is still true, and
/// it is not the whole of what mode 2026 is. Whether the terminal *recognises* the mode, and
/// whether its own state machine tracks the set and the reset it was sent, are questions the
/// terminal answers about itself, in band, on the scene's own tty — with no capture surface, no
/// window server and no automation grant in the path.
///
/// # The sentinel, and why it is not a timeout
///
/// `CSI c` behind the batch, whose reply the terminal cannot send before it has processed
/// everything ahead of it. The read stops on an **observed** condition rather than on a delay tuned
/// until it passed — scene 05's discipline, and `detect.rs`'s one crate over.
///
/// # Errors
///
/// [`ModeError`], and the five of them say five different things. See the type.
pub fn mode_reports(bytes: &[u8], mode: u16, expected: usize) -> Result<Vec<ModeReply>, ModeError> {
    let mut replies: Vec<ModeReply> = Vec::new();
    let mut sentinel = false;
    // Whether the terminal said **anything** before the sentinel, of any shape. It is what separates
    // [`ModeError::Unanswered`] from [`ModeError::Count`]: a terminal that answered a different
    // question spoke, and one that answered none did not. See that variant.
    //
    // The DECRPM arm below sets it too, where the emptiness test already covers that path. That is
    // deliberate: the flag then means what its name says rather than *spoke, except about this*, and
    // a later reader adding a fourth final byte has one rule to follow instead of two.
    let mut spoke = false;
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != 0x1b {
            i += 1;
            continue;
        }
        let Some(b'[') = bytes.get(i + 1) else {
            i += 1;
            continue;
        };
        // Parameter and intermediate bytes, then one final byte in `@`..=`~`. `$` is an
        // intermediate (0x24) and falls inside the run, which is what lets one scan read a `$y`
        // reply and a `c` one.
        let mut end = i + 2;
        while end < bytes.len() && !(0x40..=0x7e).contains(&bytes[end]) {
            end += 1;
        }
        if end == bytes.len() {
            return Err(ModeError::UnterminatedReply);
        }
        let body = &bytes[i + 2..end];
        match bytes[end] {
            // The device-attributes reply. Everything after it belongs to some other question.
            b'c' => {
                sentinel = true;
                break;
            }
            // A DECRPM reply about a **private** mode. The `?` is not decoration: `CSI 4 ; 2 $ y`
            // is the ANSI-mode reply and is a different question with the same shape.
            b'y' if body.starts_with(b"?") => {
                let text = String::from_utf8_lossy(&body[1..]);
                let text = text.strip_suffix('$').unwrap_or(&text);
                let mut parts = text.split(';');
                let which: Option<u16> = parts.next().and_then(|p| p.parse().ok());
                let ps: Option<u16> = parts.next().and_then(|p| p.parse().ok());
                let (Some(which), Some(ps)) = (which, ps) else {
                    return Err(ModeError::Malformed {
                        body: String::from_utf8_lossy(body).into_owned(),
                    });
                };
                if which != mode {
                    return Err(ModeError::OtherMode {
                        wanted: mode,
                        then: which,
                    });
                }
                let Some(state) = ModeState::from_ps(ps) else {
                    return Err(ModeError::UnknownState { ps });
                };
                spoke = true;
                replies.push(ModeReply { mode: which, state });
            }
            _ => spoke = true,
        }
        i = end + 1;
    }

    if !sentinel {
        return Err(ModeError::NoSentinel);
    }
    if replies.is_empty() && expected > 0 && !spoke {
        return Err(ModeError::Unanswered { expected });
    }
    if replies.len() != expected {
        return Err(ModeError::Count {
            expected,
            found: replies.len(),
        });
    }
    Ok(replies)
}

// ── Scene 06, part B: when the terminal stopped reporting the mode as set ────────────────────────

/// One open, one wait, one question: what the terminal said about the mode `at_ms` after it was set.
///
/// **One probe per open, and that is a finding rather than a shape choice.** See
/// [`next_delay`].
/// # Two clocks, because a bracket built from one of them is unsound in one direction
///
/// The terminal processes the question at some instant between the write and the reply, and which
/// end of that interval is safe depends on the answer. *Still set* at some instant implies still
/// set at every earlier one, so the **request** is the sound end — a sleep is a floor, so the
/// question cannot have been asked before it. *Already reset* at some instant implies reset at
/// every later one, so the **reply** is the sound end. One clock read for both reports a bracket
/// the run does not support: the reply for both pushes `still_set_at` past anything observed, and
/// the request for both pulls `reset_by` below it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Probe {
    /// How long the block was asked to stay open before the question, in milliseconds.
    ///
    /// The **requested** delay and not the achieved one, which is what makes it a sound lower
    /// bound. It is also the key [`next_delay`] looks a probe up by, because it is the only one of
    /// the two that side knows before the round has run.
    pub at_ms: u32,
    /// Elapsed, in milliseconds from the open, when the reply arrived. Measured, and a sound upper
    /// bound on when the terminal processed the question.
    pub answered_at_ms: u32,
    /// What came back, or `None` where the terminal answered nothing at all.
    pub state: Option<ModeState>,
}

/// What a run of probes says about when the terminal let go of the mode.
///
/// **This is the flag and it is not the paint.** A force flush is a rendering event; DECRQM reports
/// a *mode*. A terminal may paint without clearing the flag or clear it without painting, and
/// nothing here can tell those apart — what is measured is when the terminal stopped reporting the
/// mode as set, which is the event Ghostty's own source calls *reset the synchronized output flag*.
/// Reading it as the paint would be the instrument claiming a measurement it did not take.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bracket {
    /// Still set at `still_set_at` ms and reset by `reset_by` ms. The event is in between; a
    /// bracket and never a point, because a probe is a sample.
    Between {
        /// The largest delay at which the terminal still reported the mode set.
        still_set_at: u32,
        /// The smallest delay at which it reported it reset.
        reset_by: u32,
    },
    /// Still set at the largest delay probed, so this terminal's limit — if it has one — is beyond
    /// the ceiling this run used.
    NeverReset {
        /// That largest delay, as it was **requested**: nothing here was observed to have reset by
        /// anything, so the reply clock has nothing to bound.
        ceiling: u32,
    },
    /// Already reset by the earliest reply this run saw.
    ///
    /// **Three facts look like this and the third arrived with the sixth arm.** The limit may be
    /// under the floor; the open may never have taken; or the terminal's DECRQM may never report
    /// the mode *set*, which makes a bisection over *when did it stop saying set* a search with
    /// nothing to find. Alacritty 0.17.0 is the third — `Term::report_private_mode` answers mode
    /// 2026 with a constant — and on such a terminal this figure is a fact about the **reply**
    /// rather than about the flag: a question asked inside an open block comes back when the block
    /// drains, so the floor is a bound on the force flush and not on the mode.
    ///
    /// Nothing here can tell the three apart, which is why the variant carries the number and the
    /// arm carries the reading. [`FLUSH_FLOOR_MS`]'s own doc is where the first two are separated.
    AlreadyReset {
        /// The elapsed at which that earliest reply arrived.
        floor: u32,
    },
}

/// Why a run of probes is not a bracket.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BracketError {
    /// Nothing was probed.
    NoProbes,
    /// A probe got no reply, so the sequence has a hole and the two ends of the bracket may be on
    /// opposite sides of it.
    NoAnswer {
        /// Where the hole is.
        at_ms: u32,
    },
    /// A probe came back in a state a bracket cannot be built from — the mode not recognised, or
    /// pinned permanently one way. Each of those is a real answer about the terminal and none of
    /// them is a *timing*, so the row says so rather than reporting a number.
    NotUsable {
        /// Where.
        at_ms: u32,
        /// What came back.
        state: ModeState,
    },
    /// The terminal reported the mode set at a longer delay than one at which it reported it reset.
    ///
    /// **A timeout that has expired stays expired**, and that assumption is what makes a bisection
    /// sound rather than a search over an arbitrary function. A run that breaks it has measured
    /// something other than a timeout, and inventing a bracket from it would hide that.
    NotMonotone {
        /// The delay at which it was still set.
        set_at: u32,
        /// The smaller delay at which it had already reset.
        reset_at: u32,
    },
}

impl fmt::Display for BracketError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoProbes => write!(f, "no probes, so there is nothing to bracket"),
            Self::NoAnswer { at_ms } => write!(
                f,
                "the probe at {at_ms} ms got no reply, so the sequence has a hole and the event \
                 may be on either side of it"
            ),
            Self::NotUsable { at_ms, state } => write!(
                f,
                "the probe at {at_ms} ms came back {state}, which is an answer about the terminal \
                 and not a timing"
            ),
            Self::NotMonotone { set_at, reset_at } => write!(
                f,
                "the mode was reported set at {set_at} ms and already reset at {reset_at} ms; a \
                 timeout that has expired stays expired, so this run measured something other than \
                 one"
            ),
        }
    }
}

impl std::error::Error for BracketError {}

/// Fold a run of probes into the interval the reset happened in.
///
/// # Errors
///
/// [`BracketError`]. A hole, a state that is not a timing, and a non-monotone pair are each refused
/// rather than bracketed around.
pub fn flush_bracket(probes: &[Probe]) -> Result<Bracket, BracketError> {
    if probes.is_empty() {
        return Err(BracketError::NoProbes);
    }
    let (mut last_set, mut first_reset): (Option<u32>, Option<u32>) = (None, None);
    for probe in probes {
        let Some(state) = probe.state else {
            return Err(BracketError::NoAnswer { at_ms: probe.at_ms });
        };
        match state {
            // The request for the set end and the reply for the reset end — see [`Probe`]. One
            // clock read for both is unsound in whichever direction it is read.
            ModeState::Set => {
                last_set = Some(last_set.map_or(probe.at_ms, |m: u32| m.max(probe.at_ms)))
            }
            ModeState::Reset => {
                first_reset = Some(
                    first_reset.map_or(probe.answered_at_ms, |m: u32| m.min(probe.answered_at_ms)),
                );
            }
            other => {
                return Err(BracketError::NotUsable {
                    at_ms: probe.at_ms,
                    state: other,
                });
            }
        }
    }
    match (last_set, first_reset) {
        (Some(set_at), Some(reset_at)) if set_at >= reset_at => {
            Err(BracketError::NotMonotone { set_at, reset_at })
        }
        (Some(still_set_at), Some(reset_by)) => Ok(Bracket::Between {
            still_set_at,
            reset_by,
        }),
        (Some(ceiling), None) => Ok(Bracket::NeverReset { ceiling }),
        (None, Some(floor)) => Ok(Bracket::AlreadyReset { floor }),
        (None, None) => Err(BracketError::NoProbes),
    }
}

/// The smallest delay scene 06 opens a block for, and the mode must still be set there.
///
/// Small enough that no terminal's force-flush limit can be under it — the smallest documented one
/// is Alacritty's 150 ms — so a floor that reports the mode already reset is evidence that the open
/// never took, which is a different fact and gets its own row.
///
/// **That inference needs a terminal whose DECRQM tracks the mode, and the sixth arm is one whose
/// does not.** Alacritty 0.17.0 answers `reset` inside an open block and outside one alike, so its
/// floor is neither of the two facts above; see [`Bracket::AlreadyReset`], where the third is
/// written down. The constant is unchanged — it is still below every documented limit — and what
/// changed is the sentence a reader is entitled to draw from reaching it.
pub const FLUSH_FLOOR_MS: u32 = 50;

/// The largest, and the mode must be reset by there.
///
/// Above every documented limit `quirks.rs` carries — kitty's 2000 ms is the largest — with enough
/// margin that a terminal at that figure brackets rather than falling off the end. A terminal still
/// holding the mode here is reported as such and never as a number.
pub const FLUSH_CEILING_MS: u32 = 3000;

/// How tight a bracket scene 06 halves for before it stops.
///
/// **A resolution and not an accuracy.** It bounds the interval the answer is reported in; it says
/// nothing about where in that interval the terminal's own timer is. 128 ms is what separates the
/// two figures this scene exists to tell apart — a 1000 ms limit from a 2000 ms one — with an order
/// of magnitude to spare, and every halving past it costs an open worth up to three seconds.
pub const FLUSH_RESOLUTION_MS: u32 = 128;

/// The most opens the bisection above can cost, the two ends included.
///
/// **A bound and not a count.** Halving 2 950 ms down to 128 takes five steps, so the run is seven
/// opens — which `the_halving_converges_on_the_boundary_and_stops_at_the_resolution` asserts against
/// this constant rather than against a literal, so a change to any of the three parameters moves
/// the budget and the test together. It is what an arm's readiness timeout is derived from: a
/// driver that gave up while the scene was still measuring would report *the scene never presented
/// a frame*, which is the timeout blaming the scene for the measurement it asked for.
pub const FLUSH_OPENS: usize = 7;

/// The next delay to open a block for, or `None` when the run is finished.
///
/// # One probe per open, and the control probe is why
///
/// The obvious instrument opens a block once and polls it, which costs one open where this costs
/// nine. It was written first and it is **wrong on one of the three families measured**. Polling a
/// Ghostty 1.3.1 every 250 ms brought the flag's reset forward from 1002 ms to somewhere under
/// 517 ms, while the same polling left tmux 3.7c at 1007 ms and kitty 0.48.2 at 2261 ms — their
/// documented figures. An instrument that polls is inside its own measurement, and the two arms it
/// happens not to disturb are exactly what would have made that invisible.
///
/// So each probe is a fresh open, a wait, one question and a close. The cost is the bisection, and
/// the bisection is what keeps it to nine.
///
/// # Bisection, and the assumption it rests on
///
/// A timeout that has expired stays expired, so *still set* and *already reset* partition the
/// delays and the boundary can be halved for. That assumption is not free and [`flush_bracket`]
/// refuses a run that breaks it. The two ends are probed **first**, before any halving, because a
/// bisection between two ends that were never established is a bisection over a guess.
#[must_use]
pub fn next_delay(
    probes: &[Probe],
    floor_ms: u32,
    ceiling_ms: u32,
    resolution_ms: u32,
) -> Option<u32> {
    let at = |ms: u32| probes.iter().find(|p| p.at_ms == ms).map(|p| p.state);
    // The ends first, in this order, so a terminal that never reset costs the ceiling once rather
    // than once per halving.
    let floor = match at(floor_ms) {
        None => return Some(floor_ms),
        Some(state) => state,
    };
    let ceiling = match at(ceiling_ms) {
        None => return Some(ceiling_ms),
        Some(state) => state,
    };
    // Anything but a clean `still set at the floor, reset by the ceiling` has nothing to bisect,
    // and `flush_bracket` is where each of those becomes the row it deserves.
    if floor != Some(ModeState::Set) || ceiling != Some(ModeState::Reset) {
        return None;
    }
    let mut lo = floor_ms;
    let mut hi = ceiling_ms;
    for probe in probes {
        match probe.state {
            Some(ModeState::Set) if probe.at_ms > lo && probe.at_ms < hi => lo = probe.at_ms,
            Some(ModeState::Reset) if probe.at_ms < hi && probe.at_ms > lo => hi = probe.at_ms,
            _ => {}
        }
    }
    if hi - lo <= resolution_ms {
        return None;
    }
    Some(lo + (hi - lo) / 2)
}

/// WezTerm's GUI socket path as a **committed report** may carry it: `$HOME` as `~`, and the
/// process id as `<pid>`.
///
/// A report in this directory is evidence a stranger reads, so it may not carry the operator's home
/// directory or the process id of one run. Neither reproduces — the pid is that run's child and the
/// home is whoever ran it — and what the sentence around the path is about, that the variable names
/// the arm's *own* socket rather than the shared mux server's, survives the substitution intact.
/// That is how you can tell nothing was lost by making the report reproducible.
///
/// **`home` is a parameter rather than read from the environment**, which is what makes this
/// testable: `std::env::set_var` is `unsafe` in this edition and a test that set `$HOME` would be
/// reaching into every other test in the process. The caller reads it once, the same way the arm's
/// own `runtime_dir` does.
///
/// Stripping is component-wise, so a `$HOME` with a trailing slash is handled; a path that is not
/// under `home`, an absent `home`, and a name without the needle each keep the part the
/// substitution does not reach.
///
/// **The separator after the tilde is always `/`**, whatever the host thinks. The first Windows run
/// of this crate caught the alternative: `Path::join` uses the platform's separator, so the one
/// segment this function contributes came back as `\` while every segment it had merely copied
/// stayed `/`. The output is report text, and a report whose bytes depend on who ran it cannot be
/// diffed — which is the whole reason the path is redacted rather than printed.
pub fn socket_for_report(socket: &Path, home: Option<&Path>) -> String {
    let shown = match home.and_then(|home| socket.strip_prefix(home).ok()) {
        // **`format!` and not `Path::join`.** A join inserts the *platform's* separator, so on
        // Windows this produced `~\.local/share/...` — one backslash spliced into a path that keeps
        // forward slashes everywhere else, because `display` prints a path as stored and only the
        // join had an opinion. What this returns is a line of a committed report rather than a path
        // anything opens.
        Some(rest) => format!("~/{}", rest.display()),
        None => socket.display().to_string(),
    };
    match shown.rsplit_once("gui-sock-") {
        Some((head, _)) => format!("{head}gui-sock-<pid>"),
        None => shown,
    }
}

#[cfg(test)]
mod tests;
