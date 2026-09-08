//! iTerm2's own screen buffer, read as a [`Dump`].
//!
//! # Why this is a third serialisation rather than a third SGR dialect
//!
//! [`crate::Dialect`]'s first two arms are escape streams a terminal wrote back, and they disagree
//! about what a colon means. The third — Alacritty's `grid.json` — is not an escape stream at all,
//! and neither is this one. iTerm2's Python API answers a `GetBufferRequest` with a protobuf
//! `GetBufferResponse`: one `LineContents` per row, carrying the row's text, a run-length map from
//! cells to code points, and — when the request sets `include_styles` — one `CellStyle` per run of
//! cells. **The cell's own fields, with no serialiser of the emulator's in the path**, which is
//! Alacritty's property arriving through a completely different mechanism.
//!
//! It costs the same thing and the report says so: a buffer is what the terminal **stores**, one
//! step away from what it paints. And it costs one thing Alacritty's does not — `CellStyle` is a
//! *projection* of `screen_char_t` rather than the struct itself, so a bit the cell holds and the
//! projection drops is a reading this format has and the grid does not. Exactly one attribute in
//! this suite is in that position, and the arm declares it as [`crate::Dialect`]'s first genuine
//! `cannot ask` on a non-escape capture: see `REPORT-iterm2.md`.
//!
//! # The three shapes that had to be read off a real capture rather than assumed
//!
//! 1. **`text` is code points and `style` is cells, and the two lengths differ.** A row is not a
//!    string with a style beside it: `code_points_per_cell` is the join, run-length encoded, and a
//!    reader that zipped `text.chars()` against the style runs would drift by one cell per wide
//!    glyph and hand back a row whose styles are all correct and all in the wrong place.
//! 2. **A double-width cluster's far half is a cell with *zero* code points**, and no character of
//!    its own appears in `text`. Dropping it is what makes this capture agree with every other
//!    arm's — `SCENES.md` the engine's design established that a grid-to-text dump emits no padding cell, and scene
//!    04's six rows are written against that.
//! 3. **A never-written cell is not reported at all.** `bold` on a forty-column screen comes back as
//!    four cells, not forty. That is the kitty arm's *a never-written cell is not a painted blank*
//!    seen through a third surface, and it is why nothing here pads a short row: the scene's own
//!    row count is the refusal, as it is for every other format.
//! 4. **A cell the terminal emptied is `U+0000`, and a space somebody wrote is `U+0020`.** This is
//!    the only capture surface in this directory that tells the two apart — every other one
//!    re-serialises both as a space — and scene 04's orphaned halves come back as the first. It is
//!    projected down to a space here, because the alternative is to report *iTerm2 does not blank
//!    the head of a bisected pair* about a terminal that blanks it harder than anyone: the cell is
//!    not merely spaced, it is empty. What the projection costs is stated in `FINDINGS.md` rather
//!    than hidden — the distinction is real information and this comparison throws it away.
//!
//! # What it refuses
//!
//! A field number this file does not know, inside a message this file does read, is
//! [`DumpError::Malformed`] rather than a cell quietly losing an attribute. That is the grid
//! reader's rule and the reason is identical: a future iTerm2 that adds a style field must arrive as
//! a failed run and not as a terminal that stopped doing something. The rule reaches `fgAlternate`
//! too — `REVERSED_DEFAULT` and `SYSTEM_MESSAGE` are the *renderer's* resolutions and cannot be in a
//! cell, so one appearing means this file is reading a shape it was not written for.
//!
//! Unknown fields of the two **envelope** messages are skipped rather than refused, and the
//! difference is deliberate: `ServerOriginatedMessage` and `GetBufferResponse` carry notifications,
//! cursor positions and ranges that have nothing to do with a cell, and refusing those would make
//! this reader fail on a capture whose every cell it understood.

use crate::{Attrs, Cluster, Colour, Dump, DumpError, Row, Style, Underline};

/// Parse a `ServerOriginatedMessage` carrying a `GetBufferResponse` into a [`Dump`].
///
/// `expected_rows` is the scene's declaration, refused the same way [`crate::parse`] refuses it.
pub(crate) fn parse(bytes: &[u8], expected_rows: usize) -> Result<Dump, DumpError> {
    if bytes.is_empty() {
        return Err(DumpError::Empty);
    }
    let mut response: Option<&[u8]> = None;
    for field in Fields::new(bytes) {
        let field = field?;
        match field.number {
            // `error`, and it is read rather than skipped: iTerm2 answers a malformed or unauthorised
            // request with a message here and no response at all, and a reader that skipped it would
            // report an empty screen where the API had said why there was none.
            2 => {
                return Err(DumpError::Malformed(format!(
                    "the API answered with an error rather than a buffer: {}",
                    field.text("the error")?
                )));
            }
            100 => response = Some(field.bytes("get_buffer_response")?),
            _ => {}
        }
    }
    let Some(response) = response else {
        return Err(DumpError::Malformed(
            "the message carries no `get_buffer_response`".into(),
        ));
    };

    let mut rows = Vec::new();
    for field in Fields::new(response) {
        let field = field?;
        match field.number {
            // `status`, and a non-`OK` one is refused here rather than read as a short screen: a
            // session that has gone away answers with `SESSION_NOT_FOUND` and no contents, which is
            // an empty capture wearing a successful reply's clothes.
            1 => {
                let status = field.varint("status")?;
                if status != 0 {
                    return Err(DumpError::Malformed(format!(
                        "`GetBufferResponse.status` is {status}, and only `OK` (0) carries a screen"
                    )));
                }
            }
            3 => rows.push(row(field.bytes("a line's contents")?)?),
            _ => {}
        }
    }

    if rows.is_empty() || rows.iter().all(|r| r.text().trim().is_empty()) {
        // The refusal [`crate::parse`] makes about a capture of nothing at all, made about the same
        // screen through a third channel. An arm whose scene never drew must not be able to reach a
        // table through this format when it could not through the other two.
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

/// One `LineContents`, with the far half of every double-width pair dropped.
fn row(bytes: &[u8]) -> Result<Row, DumpError> {
    let mut text = "";
    // `(num_code_points, repeats)` and `(style, repeats)`, both run-length encoded over **cells**.
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut styles: Vec<(Style, usize)> = Vec::new();

    for field in Fields::new(bytes) {
        let field = field?;
        match field.number {
            1 => text = field.text("a line's text")?,
            2 => spans.push(code_points_per_cell(field.bytes("a cell span")?)?),
            // `continuation` — whether the row wraps. Read and discarded: every scene here writes
            // rows shorter than the surface, and a reader that folded soft-wrapped rows together
            // would be deciding a question `SCENES.md` puts to the scenes rather than to the parser.
            3 => {}
            4 => styles.push(style(field.bytes("a cell style")?)?),
            _ => {}
        }
    }

    // **Code points and not bytes**, and not `char`s either where a cluster spans several: the span
    // says how many code points this cell holds, so the cursor walks `char_indices` and the cluster
    // is the slice between two of them.
    // **A ceiling, because a `repeats` off the wire is not bounded by anything else here.** A cell
    // with no code points consumes nothing from `text`, so a span claiming a huge repeat count would
    // otherwise turn the loop below into one that never returns — every other overrun in this file
    // is an explicit refusal and this was the gap. The bound is the row's own two accounts of its
    // width: no honest row has more cells than `text` has code points plus the cells the style runs
    // cover, and a screen is a few hundred either way.
    let ceiling = text.chars().count() + styles.iter().map(|(_, n)| *n).sum::<usize>() + 1;
    let claimed: usize = spans.iter().map(|(_, n)| *n).sum();
    if claimed > ceiling {
        return Err(DumpError::Malformed(format!(
            "`code_points_per_cell` claims {claimed} cells on a row that can hold at most {ceiling}"
        )));
    }

    let mut chars = text.chars();
    let mut runs = styles
        .into_iter()
        .flat_map(|(s, n)| std::iter::repeat_n(s, n));
    let mut clusters = Vec::new();

    for (points, repeats) in spans {
        for _ in 0..repeats {
            let mut cluster = String::new();
            for _ in 0..points {
                let Some(c) = chars.next() else {
                    return Err(DumpError::Malformed(format!(
                        "`code_points_per_cell` claims more code points than `text` holds, on a row \
                         of {} code points",
                        text.chars().count()
                    )));
                };
                cluster.push(c);
            }
            // **The empty cell, projected to a space.** See the module docs: this surface is the
            // only one here that distinguishes a cell the terminal emptied from a space somebody
            // wrote, and scene 04's expectations are written in the alphabet every other arm speaks.
            // One code point exactly, because a cluster whose *base* were a NUL would be a shape
            // this reader has never seen and must not quietly rewrite.
            if cluster == "\0" {
                cluster = " ".to_string();
            }
            let Some(style) = runs.next() else {
                return Err(DumpError::Malformed(
                    "the style runs cover fewer cells than `code_points_per_cell` does, so a cell's \
                     style would have to be guessed"
                        .into(),
                ));
            };
            // **A cell with no code points is the far half of a double-width pair, and it is
            // dropped.** See the module docs: every other capture format in this directory emits no
            // padding cell, and scene 04's six rows are written against that.
            if points == 0 {
                continue;
            }
            clusters.push(Cluster {
                text: cluster,
                style,
            });
        }
    }

    if chars.next().is_some() {
        return Err(DumpError::Malformed(
            "`text` holds code points no cell claims, so the row's join is not a partition".into(),
        ));
    }
    // **The other end of the same join, and it is refused for the same reason.** Style runs covering
    // *more* cells than `code_points_per_cell` does is the mismatch the loop above cannot notice —
    // it stops when the spans do — and a reader that let it pass would be reading a row whose two
    // descriptions of its own width disagree, which is exactly the state a wrong answer comes out of.
    if runs.next().is_some() {
        return Err(DumpError::Malformed(
            "the style runs cover more cells than `code_points_per_cell` does, so the row's two \
             accounts of its own width disagree"
                .into(),
        ));
    }
    Ok(Row { clusters })
}

/// One `CodePointsPerCell`: how many code points each of `repeats` cells holds.
fn code_points_per_cell(bytes: &[u8]) -> Result<(usize, usize), DumpError> {
    let (mut points, mut repeats) = (0u64, 0u64);
    for field in Fields::new(bytes) {
        let field = field?;
        match field.number {
            1 => points = field.varint("num_code_points")?,
            2 => repeats = field.varint("repeats")?,
            n => {
                return Err(DumpError::Malformed(format!(
                    "`CodePointsPerCell` field {n} is not one this reader knows"
                )));
            }
        }
    }
    Ok((points as usize, repeats as usize))
}

/// One `CellStyle`, as the eight-flag-plus-underline style this suite compares.
///
/// **The projection is where this arm's one `cannot ask` lives.** iTerm2's own cell — `screen_char_t`
/// — carries a three-bit `underlineStyle`, and this message carries a `bool`. A double underline and
/// a dotted one are both `underline: true` here, and mapping them to [`Underline::Single`] is the
/// only honest reading available: it is what the field says. The arm declares those two rows before
/// the run rather than reporting a misbehaviour that is not happening — kitty's `CSI 4 : m` in a
/// different alphabet.
fn style(bytes: &[u8]) -> Result<(Style, usize), DumpError> {
    let mut style = Style::default();
    let mut repeats = 0u64;
    let (mut fg_standard, mut bg_standard) = (None, None);
    let (mut fg_alternate, mut bg_alternate) = (0u64, 0u64);
    let mut inverse = false;

    for field in Fields::new(bytes) {
        let field = field?;
        let flag = |bit: Attrs, style: &mut Style| -> Result<(), DumpError> {
            if field.varint("an attribute flag")? != 0 {
                style.attrs.set(bit);
            }
            Ok(())
        };
        match field.number {
            1 => fg_standard = Some(field.varint("fgStandard")?),
            2 => fg_alternate = field.varint("fgAlternate")?,
            3 => style.fg = rgb(field.bytes("fgRgb")?)?,
            // `fgAlternatePlacementX` / `bgAlternatePlacementY` — where in a gradient an alternate
            // colour sits. Not a colour and not an attribute; read and discarded.
            4 | 8 => {}
            5 => bg_standard = Some(field.varint("bgStandard")?),
            6 => bg_alternate = field.varint("bgAlternate")?,
            7 => style.bg = rgb(field.bytes("bgRgb")?)?,
            9 => flag(Attrs::BOLD, &mut style)?,
            10 => flag(Attrs::DIM, &mut style)?,
            11 => flag(Attrs::ITALIC, &mut style)?,
            12 => flag(Attrs::BLINK, &mut style)?,
            13 => {
                if field.varint("underline")? != 0 {
                    style.underline = Underline::Single;
                }
            }
            14 => flag(Attrs::STRIKE, &mut style)?,
            15 => flag(Attrs::HIDDEN, &mut style)?,
            16 => {
                inverse = field.varint("inverse")? != 0;
                if inverse {
                    style.attrs.set(Attrs::REVERSE);
                }
            }
            // `guarded` (DECSCA), `image`, `blockID` and `url` (OSC 8). None is a style bit this
            // suite compares, and all four are named rather than skipped so that a *new* field
            // arrives at the refusal below instead of joining them.
            17 | 18 | 20 | 21 => {}
            19 => style.underline_colour = rgb(field.bytes("underlineColor")?)?,
            22 => repeats = field.varint("repeats")?,
            n => {
                return Err(DumpError::Malformed(format!(
                    "`CellStyle` field {n} is not one this reader knows — a future iTerm2 that grew \
                     an attribute must arrive as a failed run and not as a terminal that stopped \
                     doing something"
                )));
            }
        }
    }

    // **After the loop, because the two colour spellings arrive in either order and the reverse flag
    // decides how to read one of them.** See [`alternate`]: a reversed cell's colours come back
    // already swapped, so `fgAlternate` and `inverse` are one fact between them and neither can be
    // resolved where it was parsed.
    style.fg = alternate(fg_alternate, inverse, style.fg, "fgAlternate")?;
    style.bg = alternate(bg_alternate, inverse, style.bg, "bgAlternate")?;
    if style.fg == Colour::Default
        && let Some(index) = fg_standard
    {
        style.fg = indexed(index, "fgStandard")?;
    }
    if style.bg == Colour::Default
        && let Some(index) = bg_standard
    {
        style.bg = indexed(index, "bgStandard")?;
    }
    // **The swap this reader will not undo, and it is a refusal rather than an arithmetic.** A cell
    // written `SGR 7 ; 31` comes back as `fgAlternate: REVERSED_DEFAULT` and `bgStandard: 1` — the
    // red has moved to the *background*, because the projection applies the reverse and then reports
    // the result. Recovering the cell's own pair would be one swap, and no scene in `SCENES.md`
    // colours a reversed cell, so that swap would be un-exercised code deciding a comparison nobody
    // has watched fail. The scene that adds one is the run that earns it.
    if inverse && (style.fg != Colour::Default || style.bg != Colour::Default) {
        return Err(DumpError::Malformed(
            "a reversed cell carries a colour, and this projection reports the pair already \
             swapped — no scene here produces one, so the swap is refused rather than guessed at"
                .into(),
        ));
    }
    Ok((style, repeats as usize))
}

/// An `AlternateColor`, which is `DEFAULT` or — in a reversed cell — `REVERSED_DEFAULT`.
///
/// # `REVERSED_DEFAULT` is not a renderer's resolution, and the first draft of this file said it was
///
/// The grid reader refuses a `DimRed` because the *renderer* resolved it and no cell can hold one,
/// and this enum looks like the same shape. It is not. `REVERSED_DEFAULT` is what iTerm2 calls the
/// default colour **of a cell whose `inverse` bit is set**: the flag is reported too, on the same
/// style, so nothing has been resolved away — the pair has been swapped and named. A run of scene
/// 01 is what said so, by failing on its `reverse` row against a reader that had assumed otherwise.
///
/// So it resolves to [`Colour::Default`] and requires `inverse` beside it. `SYSTEM_MESSAGE` stays
/// refused: that one *is* the renderer, colouring a line iTerm2 wrote itself.
fn alternate(value: u64, inverse: bool, already: Colour, what: &str) -> Result<Colour, DumpError> {
    match value {
        0 => Ok(already),
        3 if inverse => Ok(already),
        3 => Err(DumpError::Malformed(format!(
            "`{what}` is `REVERSED_DEFAULT` on a cell whose `inverse` bit is not set, which is a \
             shape this reader was not written for"
        ))),
        n => Err(DumpError::Malformed(format!(
            "`{what}` is {n}, which is the renderer's own resolution and cannot be a cell's colour"
        ))),
    }
}

/// A palette index, refused rather than truncated where it is not one.
fn indexed(value: u64, what: &str) -> Result<Colour, DumpError> {
    u8::try_from(value).map(Colour::Indexed).map_err(|_| {
        DumpError::Malformed(format!("`{what}` is {value}, which is not a palette index"))
    })
}

/// An `RGBColor`, whose three channels are each refused rather than truncated.
fn rgb(bytes: &[u8]) -> Result<Colour, DumpError> {
    let (mut r, mut g, mut b) = (0u64, 0u64, 0u64);
    for field in Fields::new(bytes) {
        let field = field?;
        match field.number {
            1 => r = field.varint("red")?,
            2 => g = field.varint("green")?,
            3 => b = field.varint("blue")?,
            n => {
                return Err(DumpError::Malformed(format!(
                    "`RGBColor` field {n} is not one this reader knows"
                )));
            }
        }
    }
    let channel = |v: u64, name: &str| {
        u8::try_from(v).map_err(|_| {
            DumpError::Malformed(format!("`RGBColor.{name}` is {v}, which is not a channel"))
        })
    };
    Ok(Colour::Rgb(
        channel(r, "red")?,
        channel(g, "green")?,
        channel(b, "blue")?,
    ))
}

// ── The wire format, which is four wire types and a varint ───────────────────────────────────────

/// One field of a protobuf message: its number, and its value still in the encoding it arrived in.
struct Field<'a> {
    number: u32,
    wire: u8,
    varint: u64,
    bytes: &'a [u8],
}

impl<'a> Field<'a> {
    /// The field's value as a varint, refusing a field that is not one.
    ///
    /// **The refusal is not pedantry.** A `bool` and a length-delimited message differ by their wire
    /// type alone, so a reader that took whatever was there would read a nested message's *length*
    /// as an attribute flag and set it — silently, and true for every message longer than nothing.
    fn varint(&self, what: &str) -> Result<u64, DumpError> {
        match self.wire {
            0 => Ok(self.varint),
            n => Err(DumpError::Malformed(format!(
                "{what} arrived as wire type {n}, which is not a varint"
            ))),
        }
    }

    /// The field's value as bytes, refusing a field that is not length-delimited.
    fn bytes(&self, what: &str) -> Result<&'a [u8], DumpError> {
        match self.wire {
            2 => Ok(self.bytes),
            n => Err(DumpError::Malformed(format!(
                "{what} arrived as wire type {n}, which is not length-delimited"
            ))),
        }
    }

    /// The field's value as UTF-8.
    fn text(&self, what: &str) -> Result<&'a str, DumpError> {
        str::from_utf8(self.bytes(what)?)
            .map_err(|e| DumpError::Malformed(format!("{what} is not UTF-8: {e}")))
    }
}

/// The fields of one protobuf message, in the order they were written.
///
/// **An iterator over the wire and not a decoded struct**, which is what keeps this file short
/// enough to read: the four messages this reader knows are walked once each, and a field nobody
/// asked about costs a `match` arm rather than a type.
struct Fields<'a> {
    rest: &'a [u8],
    done: bool,
}

impl<'a> Fields<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            rest: bytes,
            done: false,
        }
    }

    /// A base-128 varint, or the truncation that says the capture is cut short.
    fn varint(&mut self) -> Result<u64, DumpError> {
        let (mut value, mut shift) = (0u64, 0u32);
        loop {
            let Some((&byte, rest)) = self.rest.split_first() else {
                return Err(DumpError::UnterminatedEscape);
            };
            self.rest = rest;
            // Ten groups of seven bits is the most a `u64` can hold, and a longer one is a
            // corruption rather than a big number — refused, so it cannot wrap into a small one.
            if shift >= 64 {
                return Err(DumpError::Malformed(
                    "a varint runs past sixty-four bits".into(),
                ));
            }
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return Ok(value);
            }
            shift += 7;
        }
    }
}

impl<'a> Iterator for Fields<'a> {
    type Item = Result<Field<'a>, DumpError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done || self.rest.is_empty() {
            return None;
        }
        Some(self.field().inspect_err(|_| self.done = true))
    }
}

impl<'a> Fields<'a> {
    fn field(&mut self) -> Result<Field<'a>, DumpError> {
        let tag = self.varint()?;
        let wire = (tag & 0x7) as u8;
        let number = u32::try_from(tag >> 3)
            .map_err(|_| DumpError::Malformed(format!("field number {} is not one", tag >> 3)))?;
        let (varint, bytes) = match wire {
            0 => (self.varint()?, &self.rest[..0]),
            // 64-bit and 32-bit: no field this reader knows uses either, and they are consumed
            // rather than refused so that an envelope carrying one stays readable.
            1 | 5 => {
                let width = if wire == 1 { 8 } else { 4 };
                if self.rest.len() < width {
                    return Err(DumpError::UnterminatedEscape);
                }
                self.rest = &self.rest[width..];
                (0, &self.rest[..0])
            }
            2 => {
                let len = usize::try_from(self.varint()?)
                    .map_err(|_| DumpError::Malformed("a length runs past this machine".into()))?;
                if self.rest.len() < len {
                    return Err(DumpError::UnterminatedEscape);
                }
                let (bytes, rest) = self.rest.split_at(len);
                self.rest = rest;
                (0, bytes)
            }
            n => {
                return Err(DumpError::Malformed(format!(
                    "wire type {n} is not one protobuf defines"
                )));
            }
        };
        Ok(Field {
            number,
            wire,
            varint,
            bytes,
        })
    }
}
