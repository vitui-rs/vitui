//! Alacritty's own grid, read as a [`Dump`].
//!
//! # Why this is not another SGR dialect
//!
//! [`crate::Dialect`]'s other two arms are serialisations *a terminal wrote back as escape
//! sequences*, and they disagree about what a colon means. This one is not an escape stream at all:
//! `alacritty --ref-test` serialises the `Term`'s grid to JSON when its last window closes, one
//! object per cell, and there is no SGR anywhere in the path. **That is the strongest capture
//! surface in this directory** — no serialiser of the emulator's stands between the cell and the
//! reader, so kitty's *the dump has no spelling for a dotted underline* and WezTerm's *the
//! serialiser dropped one attribute of a styled cell* are both structurally impossible here. An
//! underline colour is a **field** of the cell rather than a sequence that has to survive a
//! re-serialisation, which is what WezTerm's capture has no spelling for — though no scene in this
//! suite sends SGR 58, so what that buys is a surface that could answer one rather than an answer.
//!
//! It costs the opposite thing, and the report says so: a grid is what the terminal **stores**, and
//! what it stores is one step away from what it paints. That is the tmux arm's caveat arriving on an
//! emulator, and it is why the two attributes this arm finds missing are argued from the shipped
//! source and a debug log rather than from the absence alone.
//!
//! # The three shapes that had to be read off a real capture rather than assumed
//!
//! 1. **The rows are stored rotated.** `Storage` is a ring, and `alacritty_terminal`'s own
//!    `compute_index` is `(zero + visible_lines - line - 1) % len` — so `inner[0]` is the *bottom*
//!    row of a screen at `zero = 0`, and a reader that took the array in order would report every
//!    scene upside down and each row's text intact, which is the shape a comparison passes on.
//! 2. **A double-width cluster occupies two cells and the second is a spacer.** `WIDE_CHAR_SPACER`
//!    is dropped here, which is what makes this capture agree with every other arm's: `SCENES.md`
//!    the engine's design established that a grid-to-text dump emits no padding cell, and scene 04's six rows are
//!    written against that. `LEADING_WIDE_CHAR_SPACER` is **kept** — it is a blank cell a wide
//!    cluster declined to start in, not the far half of one, and no scene here produces one.
//! 3. **Combining marks live beside the cluster rather than in it.** `Cell::c` is one `char` and the
//!    rest of the grapheme cluster is `extra.zerowidth`, so a reader that took `c` alone would hand
//!    back a base character wearing the right style and call it the cluster.
//!
//! # What it refuses
//!
//! A name this file does not know is [`DumpError::Malformed`] rather than a cell quietly losing an
//! attribute: a future Alacritty that adds a flag, or renames one, must arrive as a failed run and
//! not as a terminal that stopped doing something. The same rule reaches the colour names — a
//! `DimRed` or a `BrightForeground` is the *renderer's* resolution and cannot be in a cell, so one
//! appearing means this file is reading a shape it was not written for.

use crate::{Attrs, Cluster, Colour, Dump, DumpError, Row, Style, Underline};

/// Parse an `alacritty --ref-test` `grid.json` into a [`Dump`].
///
/// `expected_rows` is the scene's declaration, refused the same way [`crate::parse`] refuses it.
pub(crate) fn parse(bytes: &[u8], expected_rows: usize) -> Result<Dump, DumpError> {
    let json = Json::parse(bytes).map_err(DumpError::Malformed)?;
    let top = json.object("the capture")?;
    let raw = top.field("raw")?.object("raw")?;

    let zero = raw.field("zero")?.usize_at("raw.zero")?;
    let visible = raw.field("visible_lines")?.usize_at("raw.visible_lines")?;
    let len = raw.field("len")?.usize_at("raw.len")?;
    let inner = raw.field("inner")?.array("raw.inner")?;

    if len == 0 || visible == 0 {
        return Err(DumpError::Empty);
    }
    // The ring's arithmetic is only sound when the array is the length the header claims: a shorter
    // one would wrap into cells that are somebody else's row rather than refusing.
    if inner.len() != len || visible > len {
        return Err(DumpError::Malformed(format!(
            "the grid says len {len} and visible_lines {visible} over {} stored rows",
            inner.len()
        )));
    }

    let mut rows = Vec::with_capacity(visible);
    for line in 0..visible {
        // `alacritty_terminal::grid::storage::Storage::compute_index`, and it is a quotation rather
        // than a derivation — see the module docs.
        let index = (zero + visible - line - 1) % len;
        let stored = inner[index].object("a stored row")?;
        rows.push(row(stored.field("inner")?.array("a row's cells")?)?);
    }

    if rows.iter().all(|r| r.text().trim().is_empty()) {
        // The same refusal `parse` makes about a capture of nothing at all, made about the same
        // screen through the other channel: an arm whose scene never drew must not be able to reach
        // a table through this format when it could not through the other.
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

/// One stored row, with the far half of every double-width pair dropped.
fn row(cells: &[Json]) -> Result<Row, DumpError> {
    let mut clusters = Vec::with_capacity(cells.len());
    for cell in cells {
        let cell = cell.object("a cell")?;
        let flags = flags(cell.field("flags")?.string("a cell's flags")?)?;
        if flags.spacer {
            continue;
        }
        let mut text = cell.field("c")?.string("a cell's character")?.to_string();
        // **`CellExtra` carries both of the things a cluster is more than a character**, which is
        // why it is read once: the rest of the grapheme cluster, and the underline colour. It is
        // `Option<Arc<CellExtra>>` on the far side and `null` on all but a few cells, so its absence
        // is the ordinary case — and the two fields *inside* it are not optional, which is what
        // separates a cell that has none from a shape this reader does not know.
        let mut underline_colour = Colour::Default;
        if let Some(extra) = cell.field("extra")?.object_or_null("a cell's extra")? {
            for mark in extra
                .field("zerowidth")?
                .array("a cell's zerowidth marks")?
            {
                text.push_str(mark.string("a zerowidth mark")?);
            }
            underline_colour = colour(extra.field("underline_color")?)?;
        }
        clusters.push(Cluster {
            text,
            style: Style {
                fg: colour(cell.field("fg")?)?,
                bg: colour(cell.field("bg")?)?,
                underline_colour,
                underline: flags.underline,
                attrs: flags.attrs,
            },
        });
    }
    Ok(Row { clusters })
}

/// What one cell's `flags` string says, split into the three things a [`Style`] keeps apart.
struct Flags {
    attrs: Attrs,
    underline: Underline,
    /// The far half of a double-width pair, which is not a cluster of its own. See the module docs.
    spacer: bool,
}

/// Read `alacritty_terminal::term::cell::Flags` as `bitflags` writes it: names, `" | "` between.
///
/// **Every name in that type is handled, including the composites.** `bitflags` yields a composite
/// only when no earlier single covers its bits, which today means `BOLD_ITALIC`, `DIM_BOLD` and
/// `ALL_UNDERLINES` never appear — *today* being a property of the declaration order in somebody
/// else's crate, and one this file would rather not fail on if it changes.
///
/// **There is no arm for blink and none for overline**, and that is the arm's finding rather than an
/// omission: `Flags` has no bit for either. See `REPORT-alacritty.md`.
fn flags(text: &str) -> Result<Flags, DumpError> {
    let mut attrs = Attrs::NONE;
    let mut underline = Underline::None;
    let mut spacer = false;
    for name in text.split('|').map(str::trim).filter(|n| !n.is_empty()) {
        match name {
            "INVERSE" => attrs.set(Attrs::REVERSE),
            "BOLD" => attrs.set(Attrs::BOLD),
            "ITALIC" => attrs.set(Attrs::ITALIC),
            "BOLD_ITALIC" => {
                attrs.set(Attrs::BOLD);
                attrs.set(Attrs::ITALIC);
            }
            "DIM" => attrs.set(Attrs::DIM),
            "DIM_BOLD" => {
                attrs.set(Attrs::DIM);
                attrs.set(Attrs::BOLD);
            }
            "HIDDEN" => attrs.set(Attrs::HIDDEN),
            "STRIKEOUT" => attrs.set(Attrs::STRIKE),
            "UNDERLINE" => underline = Underline::Single,
            "DOUBLE_UNDERLINE" => underline = Underline::Double,
            "UNDERCURL" => underline = Underline::Curly,
            "DOTTED_UNDERLINE" => underline = Underline::Dotted,
            "DASHED_UNDERLINE" => underline = Underline::Dashed,
            // A composite covering five mutually exclusive styles cannot say which one a cell wears,
            // so it is refused rather than resolved to one of them.
            "ALL_UNDERLINES" => {
                return Err(DumpError::Malformed(
                    "a cell carries ALL_UNDERLINES, which names five styles and identifies none"
                        .into(),
                ));
            }
            "WIDE_CHAR_SPACER" => spacer = true,
            // Neither is a style: one says a wide cluster starts here, the other that one declined
            // to, and the third that the row continues into the next. Read and ignored, so a future
            // unknown name is still loud.
            "WIDE_CHAR" | "LEADING_WIDE_CHAR_SPACER" | "WRAPLINE" => {}
            other => {
                return Err(DumpError::Malformed(format!(
                    "a cell carries the flag {other:?}, which this reader does not know — a capture \
                     from an Alacritty that grew one must fail here rather than lose it quietly"
                )));
            }
        }
    }
    Ok(Flags {
        attrs,
        underline,
        spacer,
    })
}

/// One of `alacritty_terminal`'s three colour shapes, as this directory's [`Colour`].
fn colour(value: &Json) -> Result<Colour, DumpError> {
    if matches!(value, Json::Null) {
        return Ok(Colour::Default);
    }
    let object = value.object("a colour")?;
    let (key, inner) = object.single("a colour")?;
    match key {
        "Indexed" => Ok(Colour::Indexed(inner.u8_at("an indexed colour")?)),
        "Spec" => {
            let spec = inner.object("an rgb colour")?;
            Ok(Colour::Rgb(
                spec.field("r")?.u8_at("an rgb colour's red")?,
                spec.field("g")?.u8_at("an rgb colour's green")?,
                spec.field("b")?.u8_at("an rgb colour's blue")?,
            ))
        }
        "Named" => named(inner.string("a named colour")?),
        other => Err(DumpError::Malformed(format!(
            "a colour spelled {other:?}, which is not one of Named, Indexed or Spec"
        ))),
    }
}

/// `NamedColor`, and the sixteen that are a palette index.
///
/// **The rest are refused rather than folded**, and the refusal is the point. `Cursor`, the eight
/// `Dim*` names, `BrightForeground` and `DimForeground` are what Alacritty's *renderer* resolves a
/// cell to; none of them can be what a cell holds, because `terminal_attribute` stores the colour
/// the SGR named. One appearing in a capture means this reader is looking at a shape it was not
/// written for, and mapping it to its base index would report a colour nothing sent.
fn named(name: &str) -> Result<Colour, DumpError> {
    const PALETTE: [&str; 16] = [
        "Black",
        "Red",
        "Green",
        "Yellow",
        "Blue",
        "Magenta",
        "Cyan",
        "White",
        "BrightBlack",
        "BrightRed",
        "BrightGreen",
        "BrightYellow",
        "BrightBlue",
        "BrightMagenta",
        "BrightCyan",
        "BrightWhite",
    ];
    if name == "Foreground" || name == "Background" {
        return Ok(Colour::Default);
    }
    match PALETTE.iter().position(|n| *n == name) {
        // The cast is exact: the array is sixteen long.
        Some(index) => Ok(Colour::Indexed(index as u8)),
        None => Err(DumpError::Malformed(format!(
            "a cell holds the colour {name:?}, which is a name the renderer resolves to and not one \
             a cell can carry"
        ))),
    }
}

// ── The smallest JSON reader that can be right about this file ───────────────────────────────────

/// A JSON value.
///
/// **Hand-written, like the dump parser beside it and for the manifest's stated reason**: this
/// crate has no dependencies, and a reader over one known document is a state machine over bytes.
/// Numbers are integers because every number in a `grid.json` is one — a fractional one is refused
/// rather than truncated, so a shape that grew a float is a failed run.
enum Json {
    Null,
    /// Parsed so the reader can walk past one; the value is **not kept**, because nothing in a grid
    /// is read as a boolean and a field this crate never reads is the spare part `warnings = "deny"`
    /// exists to refuse.
    Bool,
    Int(i64),
    Str(String),
    Arr(Vec<Json>),
    Obj(Vec<(String, Json)>),
}

impl Json {
    fn parse(bytes: &[u8]) -> Result<Json, String> {
        let text =
            std::str::from_utf8(bytes).map_err(|e| format!("the capture is not UTF-8: {e}"))?;
        let mut reader = Reader {
            bytes: text.as_bytes(),
            at: 0,
        };
        let value = reader.value()?;
        reader.space();
        if reader.at != reader.bytes.len() {
            return Err(format!(
                "{} bytes of trailing content after the document",
                reader.bytes.len() - reader.at
            ));
        }
        Ok(value)
    }

    fn object(&self, what: &str) -> Result<&[(String, Json)], DumpError> {
        match self {
            Json::Obj(fields) => Ok(fields),
            _ => Err(DumpError::Malformed(format!("{what} is not an object"))),
        }
    }

    /// The same, admitting `null` — `Cell::extra` is `Option<Arc<CellExtra>>`.
    fn object_or_null(&self, what: &str) -> Result<Option<&[(String, Json)]>, DumpError> {
        match self {
            Json::Null => Ok(None),
            Json::Obj(fields) => Ok(Some(fields)),
            _ => Err(DumpError::Malformed(format!(
                "{what} is neither an object nor null"
            ))),
        }
    }

    fn array(&self, what: &str) -> Result<&[Json], DumpError> {
        match self {
            Json::Arr(items) => Ok(items),
            _ => Err(DumpError::Malformed(format!("{what} is not an array"))),
        }
    }

    fn string(&self, what: &str) -> Result<&str, DumpError> {
        match self {
            Json::Str(text) => Ok(text),
            _ => Err(DumpError::Malformed(format!("{what} is not a string"))),
        }
    }

    fn usize_at(&self, what: &str) -> Result<usize, DumpError> {
        match self {
            Json::Int(n) if *n >= 0 => Ok(*n as usize),
            _ => Err(DumpError::Malformed(format!(
                "{what} is not a non-negative integer"
            ))),
        }
    }

    fn u8_at(&self, what: &str) -> Result<u8, DumpError> {
        match self {
            Json::Int(n) if (0..=255).contains(n) => Ok(*n as u8),
            _ => Err(DumpError::Malformed(format!("{what} is not a byte"))),
        }
    }
}

/// Field lookup, refusing an absent field rather than defaulting it.
trait Fields {
    fn field(&self, name: &str) -> Result<&Json, DumpError>;
    /// The one field of a single-field object — how `serde` writes an enum variant with a payload.
    fn single(&self, what: &str) -> Result<(&str, &Json), DumpError>;
}

impl Fields for &[(String, Json)] {
    fn field(&self, name: &str) -> Result<&Json, DumpError> {
        self.iter()
            .find(|(key, _)| key == name)
            .map(|(_, value)| value)
            .ok_or_else(|| DumpError::Malformed(format!("no {name} field")))
    }

    fn single(&self, what: &str) -> Result<(&str, &Json), DumpError> {
        match self {
            [(key, value)] => Ok((key.as_str(), value)),
            other => Err(DumpError::Malformed(format!(
                "{what} has {} fields and a variant has one",
                other.len()
            ))),
        }
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Reader<'_> {
    fn space(&mut self) {
        while matches!(self.bytes.get(self.at), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.at += 1;
        }
    }

    fn byte(&self) -> Result<u8, String> {
        self.bytes
            .get(self.at)
            .copied()
            .ok_or_else(|| "the document ends early".to_string())
    }

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        if self.byte()? != byte {
            return Err(format!(
                "expected {:?} at byte {}, found {:?}",
                byte as char,
                self.at,
                self.byte()? as char
            ));
        }
        self.at += 1;
        Ok(())
    }

    fn literal(&mut self, word: &str) -> Result<(), String> {
        if !self.bytes[self.at..].starts_with(word.as_bytes()) {
            return Err(format!("expected {word} at byte {}", self.at));
        }
        self.at += word.len();
        Ok(())
    }

    fn value(&mut self) -> Result<Json, String> {
        self.space();
        match self.byte()? {
            b'{' => self.object(),
            b'[' => self.array(),
            b'"' => Ok(Json::Str(self.string()?)),
            b't' => self.literal("true").map(|()| Json::Bool),
            b'f' => self.literal("false").map(|()| Json::Bool),
            b'n' => self.literal("null").map(|()| Json::Null),
            _ => self.number(),
        }
    }

    fn object(&mut self) -> Result<Json, String> {
        self.expect(b'{')?;
        let mut fields = Vec::new();
        self.space();
        if self.byte()? == b'}' {
            self.at += 1;
            return Ok(Json::Obj(fields));
        }
        loop {
            self.space();
            let key = self.string()?;
            self.space();
            self.expect(b':')?;
            fields.push((key, self.value()?));
            self.space();
            match self.byte()? {
                b',' => self.at += 1,
                b'}' => {
                    self.at += 1;
                    return Ok(Json::Obj(fields));
                }
                other => {
                    return Err(format!(
                        "expected , or }} at byte {}, found {other:?}",
                        self.at
                    ));
                }
            }
        }
    }

    fn array(&mut self) -> Result<Json, String> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.space();
        if self.byte()? == b']' {
            self.at += 1;
            return Ok(Json::Arr(items));
        }
        loop {
            items.push(self.value()?);
            self.space();
            match self.byte()? {
                b',' => self.at += 1,
                b']' => {
                    self.at += 1;
                    return Ok(Json::Arr(items));
                }
                other => {
                    return Err(format!(
                        "expected , or ] at byte {}, found {other:?}",
                        self.at
                    ));
                }
            }
        }
    }

    /// A JSON string, with the escapes `serde_json` can write.
    ///
    /// **The surrogate pair is not decoration.** `serde_json` writes a non-ASCII character
    /// literally by default, but a `grid.json` carrying an astral cluster written by any other
    /// producer would arrive as `😀`, and a reader that took the halves separately would
    /// hand back two replacement characters wearing the right style.
    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            match self.byte()? {
                b'"' => {
                    self.at += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.at += 1;
                    match self.byte()? {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            self.at += 1;
                            let first = self.hex4()?;
                            let ch = match first {
                                0xD800..=0xDBFF => {
                                    self.expect(b'\\')?;
                                    self.expect(b'u')?;
                                    let low = self.hex4()?;
                                    if !(0xDC00..=0xDFFF).contains(&low) {
                                        return Err(format!(
                                            "a high surrogate at byte {} is followed by {low:#06x}",
                                            self.at
                                        ));
                                    }
                                    let combined =
                                        0x1_0000 + ((first - 0xD800) << 10) + (low - 0xDC00);
                                    char::from_u32(combined)
                                }
                                other => char::from_u32(other),
                            };
                            out.push(ch.ok_or_else(|| {
                                format!("an escape at byte {} is not a character", self.at)
                            })?);
                            continue;
                        }
                        other => {
                            return Err(format!(
                                "an unknown escape {:?} at byte {}",
                                other as char, self.at
                            ));
                        }
                    }
                    self.at += 1;
                }
                _ => {
                    // A multi-byte character is copied whole: the input was validated as UTF-8, so
                    // the character starting here is a character.
                    let rest = std::str::from_utf8(&self.bytes[self.at..])
                        .map_err(|e| format!("not UTF-8 at byte {}: {e}", self.at))?;
                    let ch = rest.chars().next().ok_or("the document ends early")?;
                    out.push(ch);
                    self.at += ch.len_utf8();
                }
            }
        }
    }

    /// Four hexadecimal digits, as `\u` spells them.
    fn hex4(&mut self) -> Result<u32, String> {
        let end = self.at + 4;
        let digits = self
            .bytes
            .get(self.at..end)
            .ok_or("a \\u escape runs off the end")?;
        let text = std::str::from_utf8(digits).map_err(|e| e.to_string())?;
        let value = u32::from_str_radix(text, 16)
            .map_err(|_| format!("{text:?} is not four hexadecimal digits"))?;
        self.at = end;
        Ok(value)
    }

    /// An integer. See [`Json`] — a fraction or an exponent is refused rather than rounded.
    fn number(&mut self) -> Result<Json, String> {
        let start = self.at;
        if self.byte()? == b'-' {
            self.at += 1;
        }
        while matches!(self.bytes.get(self.at), Some(b'0'..=b'9')) {
            self.at += 1;
        }
        if matches!(self.bytes.get(self.at), Some(b'.' | b'e' | b'E')) {
            return Err(format!(
                "the number at byte {start} is not an integer, and every number in a grid is one"
            ));
        }
        let text = std::str::from_utf8(&self.bytes[start..self.at]).map_err(|e| e.to_string())?;
        text.parse()
            .map(Json::Int)
            .map_err(|_| format!("{text:?} at byte {start} is not a number"))
    }
}
