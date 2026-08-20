//! The style word: eleven attribute bits and two colours, packed into exactly 64 bits.
//!
//! The packing is the whole finding of spec §3 and it is a performance decision rather than a
//! tidiness one. The serializer's inner comparison is one style against another, once per cell of
//! every damaged run, and its cost is decided by whether the style fits in a machine word: an
//! inline `u64` scans a full screen in 5.45 us where a 12-byte `Style` with a derived `PartialEq`
//! takes 19.5 us and a masked `u128` takes 47.8 us.
//!
//! ```text
//! bit  63     extended - 0: colours inline; 1: bits 51..0 are a handle into the extended-style table
//! bits 62..52 attrs (11): bold, dim, italic, reverse, blink, strikethrough, conceal,
//!                         overline, + 3-bit underline style
//! bits 51..26 fg  [tag:2][payload:24]   tag: 0 default, 1 indexed, 2 rgb, 3 reserved
//! bits 25..0  bg  [tag:2][payload:24]
//! ```
//!
//! Exactly 64 bits with nothing spare. Bit 63 is reserved here and given its meaning by ticket 07;
//! until then no `Style` a caller can hold is extended.

pub(crate) const BOLD: u64 = 1 << 62;
pub(crate) const DIM: u64 = 1 << 61;
pub(crate) const ITALIC: u64 = 1 << 60;
pub(crate) const REVERSE: u64 = 1 << 59;
pub(crate) const BLINK: u64 = 1 << 58;
pub(crate) const STRIKETHROUGH: u64 = 1 << 57;
pub(crate) const CONCEAL: u64 = 1 << 56;
pub(crate) const OVERLINE: u64 = 1 << 55;

/// The low bit of the three-bit underline-style field, at 54..52.
pub(crate) const UNDERLINE_SHIFT: u32 = 52;
pub(crate) const UNDERLINE_MASK: u64 = 0b111 << UNDERLINE_SHIFT;

/// The low bit of the foreground colour, at 51..26.
pub(crate) const FG_SHIFT: u32 = 26;
/// A colour is 26 bits: a two-bit tag over a 24-bit payload.
pub(crate) const COLOR_MASK: u64 = (1 << 26) - 1;

pub(crate) const TAG_DEFAULT: u32 = 0;
pub(crate) const TAG_INDEXED: u32 = 1;
pub(crate) const TAG_RGB: u32 = 2;

/// How a cell is underlined. The numbering is SGR 4:n, so the serializer emits the value itself.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub(crate) enum Underline {
    None = 0,
    Single = 1,
    Double = 2,
    Curly = 3,
    Dotted = 4,
    Dashed = 5,
}

/// A colour: the terminal's default, a palette index, or 24-bit RGB.
///
/// Twenty-six bits wide, which is what lets two of them and eleven attribute bits share one `u64`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct Color(u32);

impl Color {
    /// The terminal's own foreground or background, whichever side of a style this lands on.
    pub const DEFAULT: Color = Color(TAG_DEFAULT << 24);

    /// Palette entry `i`. Entries under 16 are the ones the serializer can spell in two digits.
    pub const fn indexed(i: u8) -> Color {
        Color((TAG_INDEXED << 24) | i as u32)
    }

    /// A 24-bit colour.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Color {
        Color((TAG_RGB << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32)
    }

    pub(crate) const fn tag(self) -> u32 {
        self.0 >> 24
    }

    pub(crate) const fn payload(self) -> u32 {
        self.0 & 0x00FF_FFFF
    }

    pub(crate) const fn from_bits(bits: u32) -> Color {
        Color(bits & (COLOR_MASK as u32))
    }

    pub(crate) const fn bits(self) -> u32 {
        self.0
    }
}

/// How a cell is painted: eleven attributes and two colours.
///
/// Built by naming what it is, never by editing what a cell already has — `Style::with_bg` and
/// `Style::with_fg_bg` were removed from this API before it was implemented, because both were
/// silently wrong on an extended style (spec §3). Changing what is already in a cell will be
/// `restyle`'s job, and that verb will own the table; ticket 07 is what brings it.
///
/// # Layout
///
/// Eight bytes, aligned to four, so that [`Cell`](crate) stays sixteen bytes with no padding hole.
/// The two halves are compared together and the compiler folds them back into one 64-bit compare.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Style {
    lo: u32,
    hi: u32,
}

impl std::fmt::Debug for Style {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Style({:#018x})", self.bits())
    }
}

impl Style {
    /// Default foreground, default background, no attributes.
    pub const DEFAULT: Style = Style::from_bits(0);

    /// Default foreground, default background, no attributes.
    pub const fn new() -> Style {
        Style::DEFAULT
    }

    /// With this foreground colour.
    pub const fn fg(self, c: Color) -> Style {
        self.with_color(FG_SHIFT, c)
    }

    /// With this background colour.
    pub const fn bg(self, c: Color) -> Style {
        self.with_color(0, c)
    }

    /// Bold. Note that SGR has no un-bold that leaves dim standing: the serializer emits SGR 22 for
    /// either and reapplies the other.
    pub const fn bold(self) -> Style {
        self.set(BOLD)
    }

    /// Dim.
    pub const fn dim(self) -> Style {
        self.set(DIM)
    }

    /// Italic.
    pub const fn italic(self) -> Style {
        self.set(ITALIC)
    }

    /// Foreground and background swapped by the terminal.
    pub const fn reverse(self) -> Style {
        self.set(REVERSE)
    }

    /// Blinking. SGR 6 rapid blink is not offered: terminals that implement it fold it into this one.
    pub const fn blink(self) -> Style {
        self.set(BLINK)
    }

    /// Struck through.
    pub const fn strikethrough(self) -> Style {
        self.set(STRIKETHROUGH)
    }

    /// Concealed.
    pub const fn conceal(self) -> Style {
        self.set(CONCEAL)
    }

    /// Overlined.
    pub const fn overline(self) -> Style {
        self.set(OVERLINE)
    }

    /// Singly underlined.
    pub const fn underline(self) -> Style {
        self.with_underline(Underline::Single)
    }

    /// Doubly underlined, emitted as `4:2` — SGR 21 is never emitted, because a meaningful
    /// population of terminals implements it as "bold off".
    pub const fn underline_double(self) -> Style {
        self.with_underline(Underline::Double)
    }

    /// Underlined with a curl.
    pub const fn underline_curly(self) -> Style {
        self.with_underline(Underline::Curly)
    }

    /// Underlined with dots.
    pub const fn underline_dotted(self) -> Style {
        self.with_underline(Underline::Dotted)
    }

    /// Underlined with dashes.
    pub const fn underline_dashed(self) -> Style {
        self.with_underline(Underline::Dashed)
    }

    /// Not underlined.
    pub const fn no_underline(self) -> Style {
        self.with_underline(Underline::None)
    }

    pub(crate) const fn bits(self) -> u64 {
        ((self.hi as u64) << 32) | self.lo as u64
    }

    pub(crate) const fn from_bits(bits: u64) -> Style {
        Style {
            lo: bits as u32,
            hi: (bits >> 32) as u32,
        }
    }

    pub(crate) const fn attrs(self) -> u64 {
        self.bits() & (BOLD | DIM | ITALIC | REVERSE | BLINK | STRIKETHROUGH | CONCEAL | OVERLINE)
    }

    pub(crate) const fn underline_style(self) -> u8 {
        ((self.bits() & UNDERLINE_MASK) >> UNDERLINE_SHIFT) as u8
    }

    pub(crate) const fn foreground(self) -> Color {
        Color::from_bits(((self.bits() >> FG_SHIFT) & COLOR_MASK) as u32)
    }

    pub(crate) const fn background(self) -> Color {
        Color::from_bits((self.bits() & COLOR_MASK) as u32)
    }

    const fn set(self, bit: u64) -> Style {
        Style::from_bits(self.bits() | bit)
    }

    const fn with_underline(self, u: Underline) -> Style {
        let bits = (self.bits() & !UNDERLINE_MASK) | ((u as u64) << UNDERLINE_SHIFT);
        Style::from_bits(bits)
    }

    const fn with_color(self, shift: u32, c: Color) -> Style {
        let bits = (self.bits() & !(COLOR_MASK << shift)) | ((c.bits() as u64) << shift);
        Style::from_bits(bits)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // The gate spec §3 asks for: the bit layout asserted field by field against the diagram, so
    // that moving a field is a test failure rather than a wire change nobody notices.

    #[test]
    fn the_style_word_is_eight_bytes_aligned_to_four() {
        assert_eq!(size_of::<Style>(), 8);
        assert_eq!(align_of::<Style>(), 4);
    }

    #[test]
    fn bit_63_is_reserved_and_no_builder_can_reach_it() {
        // Ticket 07 is what gives the bit its meaning. What is pinned here is that until then
        // nothing a caller can hold is extended, which is what makes `fg` and `bg` safe where
        // `Style::with_bg` was not.
        let everything = Style::new()
            .bold()
            .dim()
            .italic()
            .reverse()
            .blink()
            .strikethrough()
            .conceal()
            .overline()
            .underline_dashed()
            .fg(Color::rgb(255, 255, 255))
            .bg(Color::rgb(255, 255, 255));
        assert_eq!(everything.bits() & (1 << 63), 0);
    }

    #[test]
    fn the_eight_attribute_flags_sit_at_62_down_to_55() {
        assert_eq!(Style::new().bold().bits(), 1 << 62);
        assert_eq!(Style::new().dim().bits(), 1 << 61);
        assert_eq!(Style::new().italic().bits(), 1 << 60);
        assert_eq!(Style::new().reverse().bits(), 1 << 59);
        assert_eq!(Style::new().blink().bits(), 1 << 58);
        assert_eq!(Style::new().strikethrough().bits(), 1 << 57);
        assert_eq!(Style::new().conceal().bits(), 1 << 56);
        assert_eq!(Style::new().overline().bits(), 1 << 55);
    }

    #[test]
    fn the_underline_style_is_three_bits_at_54_down_to_52() {
        assert_eq!(UNDERLINE_MASK, 0b111 << 52);
        assert_eq!(Style::new().underline().bits(), 1 << 52);
        assert_eq!(Style::new().underline_double().bits(), 2 << 52);
        assert_eq!(Style::new().underline_curly().bits(), 3 << 52);
        assert_eq!(Style::new().underline_dotted().bits(), 4 << 52);
        assert_eq!(Style::new().underline_dashed().bits(), 5 << 52);
        // Five values in three bits, so the field cannot overflow into the flags above it.
        assert_eq!(Style::new().underline_dashed().attrs(), 0);
    }

    #[test]
    fn the_eleven_attribute_bits_occupy_62_down_to_52_and_nothing_else() {
        let all = Style::new()
            .bold()
            .dim()
            .italic()
            .reverse()
            .blink()
            .strikethrough()
            .conceal()
            .overline()
            .underline_dashed();
        // 62..52 inclusive is eleven bits; the highest underline value in use is 5, so bit 54 is
        // the only one of the three that a builder can leave clear while a higher one is set.
        assert_eq!(all.bits() & !0x7FF0_0000_0000_0000, 0);
        assert_eq!(all.bits().count_ones(), 8 + 2);
    }

    #[test]
    fn the_foreground_is_twenty_six_bits_at_51_down_to_26() {
        assert_eq!(FG_SHIFT, 26);
        let s = Style::new().fg(Color::rgb(0xAB, 0xCD, 0xEF));
        assert_eq!(s.bits(), (((TAG_RGB as u64) << 24) | 0x00AB_CDEF) << 26);
        assert_eq!(s.bits() >> 52, 0);
    }

    #[test]
    fn the_background_is_twenty_six_bits_at_25_down_to_0() {
        let s = Style::new().bg(Color::rgb(0xAB, 0xCD, 0xEF));
        assert_eq!(s.bits(), ((TAG_RGB as u64) << 24) | 0x00AB_CDEF);
        assert_eq!(s.bits() >> 26, 0);
    }

    #[test]
    fn the_two_colours_do_not_overlap() {
        let s = Style::new()
            .fg(Color::indexed(9))
            .bg(Color::rgb(1, 2, 3))
            .bold();
        assert_eq!(s.foreground(), Color::indexed(9));
        assert_eq!(s.background(), Color::rgb(1, 2, 3));
        assert_eq!(s.attrs(), BOLD);
    }

    #[test]
    fn setting_a_colour_twice_replaces_rather_than_accumulates() {
        let s = Style::new().fg(Color::rgb(255, 0, 0)).fg(Color::indexed(2));
        assert_eq!(s.foreground(), Color::indexed(2));
    }

    #[test]
    fn a_colour_round_trips_through_its_tag_and_payload() {
        for c in [
            Color::DEFAULT,
            Color::indexed(0),
            Color::indexed(255),
            Color::rgb(0, 0, 0),
            Color::rgb(255, 255, 255),
            Color::rgb(0x12, 0x34, 0x56),
        ] {
            assert_eq!(Color::from_bits(c.bits()), c);
        }
    }

    #[test]
    fn default_is_the_zero_word() {
        assert_eq!(Style::default().bits(), 0);
        assert_eq!(Style::DEFAULT.foreground(), Color::DEFAULT);
        assert_eq!(Style::DEFAULT.background(), Color::DEFAULT);
    }
}
