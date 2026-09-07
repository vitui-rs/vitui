//! `restyle`'s descriptor: what to rewrite, and the promise about everything else.
//!
//! # Why a descriptor and not a closure
//!
//! `restyle` cannot take `impl Fn(Style) -> Style`. A closure receives a [`Style`] and a `Style`
//! carries no table, so on an extended word it is handed a 52-bit handle it cannot resolve —
//! which is exactly how the architecture's own prototype ended up returning extended styles
//! untouched, silently, for the cells that most needed changing. A descriptor says
//! *what* to change and lets the verb, which owns the tables, decide *how*.
//!
//! The contract, written down rather than implied:
//!
//! > **`restyle` rewrites what the descriptor names and preserves every channel it does not, on
//! > either side of the extended bit.**
//!
//! And the pressure valve that keeps the table from growing for nothing: a descriptor that clears
//! **both** extended channels puts the cell back inline, bit 63 clear. *Extended is a cost, not a
//! state.*

use crate::exts::{ExtStyle, LinkId};
use crate::style::{Color, Style, UNDERLINE_SHIFT};
use crate::tables::Tables;

/// A hyperlink named at the drawing verb: the URI itself, or *none*.
///
/// **The URI travels with the verb.** The [`View`](crate::View) interns it into whatever handle space
/// it draws into — a layer's surface into the stack's tables, a standalone
/// [`Surface`](crate::Surface) into its own — which is exactly what
/// [`text`](crate::View::text) has always done with a grapheme cluster. That symmetry is the whole of
/// architecture ticket 21: a handle a caller holds belongs to one table and cannot say which, and a
/// URI belongs to none of them, so there is no second mint for a surface outside a stack to need.
///
/// The table deduplicates on the URI, so *this hyperlink again* is the same URI again and costs one
/// hash probe per verb call that names one.
///
/// **[`Link::None`] clears the hyperlink**; a descriptor whose [`link`](Restyle::link) field is
/// `None` leaves it alone. The two spellings are the descriptor's usual *name it or do not*, one
/// level in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Link<'a> {
    /// No hyperlink: whatever the cell carried is taken away.
    None,
    /// This URI.
    Uri(&'a str),
}

/// Which attributes a [`Restyle`] adds or removes.
///
/// The eleven attribute bits of the style word, shifted down to 0: bit 10 is bold and bits 2..0 are
/// the underline style. The constants below are the only sanctioned way to name one.
const FLAGS: u16 = 0b111_1111_1000;
/// The three-bit underline style, at 2..0. A **value**, not a set of flags.
const UNDERLINE_FIELD: u16 = 0b111;
/// The highest underline style that names anything. SGR `4:6` and `4:7` are undefined, and a `set`
/// that lands on one selects no style rather than emitting one.
const UNDERLINE_MAX: u16 = 5;

/// What to change about the cells a [`restyle`](crate::View::restyle) covers.
///
/// Every field is *optional* in the sense that matters: `None`, or a zero mask, means **leave that
/// channel exactly as it is** — including on the far side of the extended bit, where "as it is"
/// costs a table lookup rather than nothing.
///
/// Built with struct update syntax, because naming six fields to change one is what makes people
/// reach for a closure:
///
/// ```
/// use vitui_engine::{Color, Restyle};
///
/// let selection = Restyle {
///     bg: Some(Color::indexed(4)),
///     set: Restyle::BOLD,
///     ..Default::default()
/// };
/// assert_eq!(selection.fg, None);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Restyle<'a> {
    /// The new foreground, or `None` to keep the old one.
    pub fg: Option<Color>,
    /// The new background, or `None` to keep the old one.
    pub bg: Option<Color>,
    /// Attributes to add. `Restyle::BOLD | Restyle::ITALIC`, and so on.
    pub set: u16,
    /// Attributes to remove.
    ///
    /// Removal happens first, so an attribute named by **both** masks ends up set. Bits the
    /// constants below do not name — 15..11 — mean nothing and are ignored rather than rejected;
    /// there is nothing above the style word's eleven for them to be about.
    pub clear: u16,
    /// The underline's own colour. `Some(Color::DEFAULT)` clears it — that is SGR 59, *underline
    /// in the text's colour* — and `None` leaves it alone.
    pub ul: Option<Color>,
    /// The hyperlink, as a URI. `Some(`[`Link::None`]`)` clears it and `None` leaves it alone.
    ///
    /// This is the field that gives the descriptor its lifetime, and the only one — see [`Link`] for
    /// why the URI is here rather than a handle the caller minted somewhere else.
    pub link: Option<Link<'a>>,
}

impl Restyle<'_> {
    /// Bold.
    pub const BOLD: u16 = 1 << 10;
    /// Dim.
    pub const DIM: u16 = 1 << 9;
    /// Italic.
    pub const ITALIC: u16 = 1 << 8;
    /// Foreground and background swapped by the terminal.
    pub const REVERSE: u16 = 1 << 7;
    /// Blinking.
    pub const BLINK: u16 = 1 << 6;
    /// Struck through.
    pub const STRIKETHROUGH: u16 = 1 << 5;
    /// Concealed.
    pub const CONCEAL: u16 = 1 << 4;
    /// Overlined.
    pub const OVERLINE: u16 = 1 << 3;

    /// Singly underlined.
    ///
    /// # The underline style is a value, not a flag
    ///
    /// The six underline constants share one three-bit field, and `restyle` replaces that field as
    /// a **unit**: naming one in `set` selects it whatever was there before, and naming any of them
    /// in `clear` removes the underline. So `set: Restyle::UNDERLINE_CURLY` is *curly*, with no
    /// need to clear dotted first — which is the trap a per-bit rule would have left, since
    /// dotted (4) with single (1) added is dashed (5).
    ///
    /// **Name exactly one.** Two of them **or**-ed together is not two underlines and is not an
    /// error: a three-bit field in a `u16` cannot be made to reject one, so `UNDERLINE_DOUBLE |
    /// UNDERLINE_DOTTED` is 6, which names no style, and a `set` that lands on 6 or 7 leaves the
    /// underline as it was. Discarded rather than refused, which is
    /// `docs/adr/0022-drawing-verbs-clamp-and-discard.md`'s rule for every other out-of-range
    /// argument a verb takes.
    pub const UNDERLINE: u16 = 1;
    /// Doubly underlined, emitted as `4:2` — SGR 21 is never emitted.
    pub const UNDERLINE_DOUBLE: u16 = 2;
    /// Underlined with a curl.
    pub const UNDERLINE_CURLY: u16 = 3;
    /// Underlined with dots.
    pub const UNDERLINE_DOTTED: u16 = 4;
    /// Underlined with dashes.
    pub const UNDERLINE_DASHED: u16 = 5;
    /// What to put in [`clear`](Restyle::clear) to remove an underline of any style.
    ///
    /// **For `clear` only.** Any of the five values does the same thing there — `clear` takes the
    /// whole field — and this one says so at the call site. In `set` it is 7, which names no style,
    /// so it selects nothing and leaves the underline as it was.
    pub const UNDERLINE_ANY: u16 = UNDERLINE_FIELD;
}

/// The eleven attribute bits after a descriptor has been applied to them, still shifted down to 0.
fn attrs_of(old: Style, d: &Restyle<'_>) -> u16 {
    let mut attrs = (old.attr_word() >> UNDERLINE_SHIFT) as u16;
    attrs = (attrs & !(d.clear & FLAGS)) | (d.set & FLAGS);

    let set = d.set & UNDERLINE_FIELD;
    let clear = d.clear & UNDERLINE_FIELD;
    // A `set` that names no style is not there at all, which is what leaves `clear` to act.
    if set != 0 && set <= UNDERLINE_MAX {
        attrs = (attrs & !UNDERLINE_FIELD) | set;
    } else if clear != 0 {
        attrs &= !UNDERLINE_FIELD;
    }
    attrs
}

/// The four channels a style carries, whichever side of the extended bit it is on.
///
/// An inline word has no underline colour and no hyperlink by construction — that is what *inline*
/// means — so the two extended channels come back as their absent values rather than as `None`.
///
/// `pub(crate)` for [`crate::mix`], which has to *read* the colours before it can name their
/// replacements: an operator recolours what is already there, where a descriptor names a value
/// outright. It reads them through this and writes them back through [`apply`], so the
/// preserve-everything-else contract has exactly one implementation.
pub(crate) fn channels(tables: &Tables, old: Style) -> ExtStyle {
    match old.ext_handle() {
        Some(h) => tables.exts.get(h).expect(
            "every surface in a layer stack speaks that stack's handle space (spec §3, ADR 0011)",
        ),
        None => ExtStyle {
            fg: old.foreground(),
            bg: old.background(),
            ul: Color::DEFAULT,
            link: LinkId::NONE,
        },
    }
}

/// The [`LinkId`] a descriptor names, interned into `tables` — **once per verb call**.
///
/// Separate from [`apply`] and called above `restyle`'s memo, which is what makes the hash probe
/// per *verb* rather than per cell or per distinct style word. `None` means the descriptor names no
/// hyperlink and the channel is left alone; `Some(LinkId::NONE)` is [`Link::None`], which clears it.
pub(crate) fn intern_link(tables: &mut Tables, d: &Restyle<'_>) -> Option<LinkId> {
    match d.link? {
        Link::None => Some(LinkId::NONE),
        Link::Uri(uri) => Some(tables.link(uri)),
    }
}

/// Apply a descriptor to one style word, minting a table entry only if the result needs one.
///
/// This is the whole of the contract: every channel the descriptor does not name is carried across
/// untouched, and the result lands inline whenever both extended channels are clear.
///
/// **`link` arrives already interned**, from [`intern_link`], because this function runs once per
/// distinct style word and interning a URI runs once per verb call. The two are different rates and
/// the descriptor's own field is the URI, so the resolved handle is a parameter rather than a field.
pub(crate) fn apply(
    tables: &mut Tables,
    d: &Restyle<'_>,
    link: Option<LinkId>,
    old: Style,
) -> Style {
    let attrs = (attrs_of(old, d) as u64) << UNDERLINE_SHIFT;
    let mut e = channels(tables, old);
    if let Some(c) = d.fg {
        e.fg = c;
    }
    if let Some(c) = d.bg {
        e.bg = c;
    }
    if let Some(c) = d.ul {
        e.ul = c;
    }
    if let Some(l) = link {
        e.link = l;
    }
    // **The key, not the entry**, and on a terminal with no OSC 8 the two differ by a hyperlink —
    // spec §10's one narrow exception to *degrade at serialise time*. A cell that was extended only
    // because of a link then goes back **inline**, which is `is_extended` asked of the key rather
    // than of the descriptor's result: *extended is a cost, not a state*, and a link the terminal
    // cannot express is not a cost worth paying. See [`Tables::key`](crate::tables::Tables::key).
    let e = tables.key(e);
    if e.is_extended() {
        Style::extended(attrs, tables.exts.handle(e))
    } else {
        Style::inline(attrs, e.fg, e.bg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const URI: &str = "https://example.com/";

    /// The verb's own two steps in the order [`crate::View::restyle`] takes them: intern the URI
    /// once, then apply the descriptor to a style word. Every test below goes through this rather
    /// than calling [`apply`] with a handle, so none of them can pass a link the tables never saw.
    fn applied(tables: &mut Tables, d: &Restyle<'_>, old: Style) -> Style {
        let link = intern_link(tables, d);
        apply(tables, d, link, old)
    }

    /// The id `URI` has in `tables`, for the assertions that compare handles.
    fn link(tables: &mut Tables) -> LinkId {
        tables.link(URI)
    }

    /// An extended cell: default colours, one hyperlink, one underline colour.
    fn extended(tables: &mut Tables) -> Style {
        let s = applied(
            tables,
            &Restyle {
                ul: Some(Color::rgb(9, 9, 9)),
                link: Some(Link::Uri(URI)),
                ..Default::default()
            },
            Style::new().fg(Color::indexed(1)).bg(Color::indexed(2)),
        );
        assert!(s.is_extended(), "the fixture is the extended corner");
        s
    }

    // ---- the four corners --------------------------------------------------------------------
    //
    // The gate ticket 07 asks for: `restyle` rewrites what the descriptor names and preserves every
    // channel it does not, asserted on all four corners of the extended bit.

    #[test]
    fn inline_to_inline_preserves_every_channel_the_descriptor_does_not_name() {
        let mut t = Tables::new();
        let old = Style::new()
            .bold()
            .underline_dotted()
            .fg(Color::indexed(1))
            .bg(Color::indexed(2));
        let new = applied(
            &mut t,
            &Restyle {
                bg: Some(Color::rgb(1, 2, 3)),
                ..Default::default()
            },
            old,
        );
        assert!(!new.is_extended());
        assert_eq!(new.background(), Color::rgb(1, 2, 3), "named, so rewritten");
        assert_eq!(new.foreground(), Color::indexed(1));
        assert_eq!(new.attr_word(), old.attr_word());
        assert!(t.exts.is_empty(), "nothing needed the table");
    }

    #[test]
    fn inline_to_extended_carries_the_colours_and_the_attributes_across() {
        let mut t = Tables::new();
        let old = Style::new()
            .italic()
            .underline_curly()
            .fg(Color::indexed(1))
            .bg(Color::indexed(2));
        let new = applied(
            &mut t,
            &Restyle {
                link: Some(Link::Uri(URI)),
                ..Default::default()
            },
            old,
        );
        let l = link(&mut t);
        assert!(new.is_extended());
        let e = t.exts.get(new.ext_handle().unwrap()).unwrap();
        assert_eq!(e.fg, Color::indexed(1));
        assert_eq!(e.bg, Color::indexed(2));
        assert_eq!(e.link, l);
        assert_eq!(e.ul, Color::DEFAULT);
        assert_eq!(new.attr_word(), old.attr_word());
    }

    #[test]
    fn extended_to_extended_preserves_the_channels_the_descriptor_does_not_name() {
        let mut t = Tables::new();
        let old = extended(&mut t);
        let was = t.exts.get(old.ext_handle().unwrap()).unwrap();
        let new = applied(
            &mut t,
            &Restyle {
                bg: Some(Color::rgb(4, 5, 6)),
                set: Restyle::BOLD,
                ..Default::default()
            },
            old,
        );
        assert!(new.is_extended());
        let e = t.exts.get(new.ext_handle().unwrap()).unwrap();
        assert_eq!(e.bg, Color::rgb(4, 5, 6), "named, so rewritten");
        assert_eq!(e.fg, was.fg, "the hyperlinked cell keeps its foreground");
        assert_eq!(e.ul, was.ul, "and its underline colour");
        assert_eq!(e.link, was.link, "and its hyperlink");
    }

    #[test]
    fn a_background_over_a_hyperlink_no_longer_deletes_it() {
        // `Style::with_fg_bg` cleared bit 63 and overwrote the handle with colours, so a shadow
        // falling across a hyperlink deleted the hyperlink with nothing to see. This is that case,
        // as a test rather than as a paragraph.
        let mut t = Tables::new();
        let old = extended(&mut t);
        let new = applied(
            &mut t,
            &Restyle {
                fg: Some(Color::indexed(7)),
                bg: Some(Color::indexed(0)),
                ..Default::default()
            },
            old,
        );
        let e = t.exts.get(new.ext_handle().unwrap()).unwrap();
        assert!(!e.link.is_none(), "the hyperlink survived the shadow");
        assert_eq!(t.links.uri(e.link), Some("https://example.com/"));
    }

    #[test]
    fn extended_to_inline_needs_both_extended_channels_cleared() {
        let mut t = Tables::new();
        let old = extended(&mut t);

        let one = applied(
            &mut t,
            &Restyle {
                link: Some(Link::None),
                ..Default::default()
            },
            old,
        );
        assert!(
            one.is_extended(),
            "the underline colour is still a reason to be extended"
        );

        let both = applied(
            &mut t,
            &Restyle {
                link: Some(Link::None),
                ul: Some(Color::DEFAULT),
                ..Default::default()
            },
            old,
        );
        assert!(!both.is_extended());
        assert_eq!(both.bits() & (1 << 63), 0, "bit 63 is clear");
        assert_eq!(
            both.foreground(),
            Color::indexed(1),
            "the colours came back"
        );
        assert_eq!(both.background(), Color::indexed(2));
    }

    #[test]
    fn going_back_inline_lands_on_the_word_a_caller_would_have_built() {
        // The equality that matters for the wire: a cell that has been to the table and back must
        // compare equal to one that never went, or the mirror re-emits it for ever.
        let mut t = Tables::new();
        let plain = Style::new().fg(Color::indexed(1)).bg(Color::indexed(2));
        let old = extended(&mut t);
        let there_and_back = applied(
            &mut t,
            &Restyle {
                ul: Some(Color::DEFAULT),
                link: Some(Link::None),
                ..Default::default()
            },
            old,
        );
        assert_eq!(there_and_back, plain);
    }

    // ---- attributes --------------------------------------------------------------------------

    #[test]
    fn set_and_clear_name_the_same_bits_the_style_word_does() {
        let mut t = Tables::new();
        let all = Restyle::BOLD
            | Restyle::DIM
            | Restyle::ITALIC
            | Restyle::REVERSE
            | Restyle::BLINK
            | Restyle::STRIKETHROUGH
            | Restyle::CONCEAL
            | Restyle::OVERLINE;
        let new = applied(
            &mut t,
            &Restyle {
                set: all,
                ..Default::default()
            },
            Style::new(),
        );
        let built = Style::new()
            .bold()
            .dim()
            .italic()
            .reverse()
            .blink()
            .strikethrough()
            .conceal()
            .overline();
        assert_eq!(new.attr_word(), built.attr_word());
    }

    #[test]
    fn clear_removes_only_what_it_names() {
        let mut t = Tables::new();
        let old = Style::new().bold().italic();
        let new = applied(
            &mut t,
            &Restyle {
                clear: Restyle::BOLD,
                ..Default::default()
            },
            old,
        );
        assert_eq!(new, Style::new().italic());
    }

    #[test]
    fn an_underline_style_replaces_the_field_rather_than_or_ing_into_it() {
        // Dotted is 4 and single is 1; a per-bit rule would make "set single" mean dashed.
        let mut t = Tables::new();
        let new = applied(
            &mut t,
            &Restyle {
                set: Restyle::UNDERLINE,
                ..Default::default()
            },
            Style::new().underline_dotted(),
        );
        assert_eq!(new, Style::new().underline());
    }

    #[test]
    fn an_underline_value_that_names_no_style_selects_nothing() {
        // `UNDERLINE_DOUBLE | UNDERLINE_DOTTED` is 6 and `UNDERLINE_ANY` is 7; SGR 4:6 and 4:7 are
        // undefined, so neither may reach the wire. Discarded rather than refused (ADR 0022), and
        // in both build profiles rather than asserting in one and ignoring in the other.
        let mut t = Tables::new();
        let old = Style::new().underline_curly();
        for set in [
            Restyle::UNDERLINE_DOUBLE | Restyle::UNDERLINE_DOTTED,
            Restyle::UNDERLINE_ANY,
        ] {
            let new = applied(
                &mut t,
                &Restyle {
                    set,
                    ..Default::default()
                },
                old,
            );
            assert_eq!(new, old, "set: {set:#b}");
            assert!(new.underline_style() <= 5);
        }
    }

    #[test]
    fn a_set_that_names_no_style_leaves_clear_to_act() {
        let mut t = Tables::new();
        let new = applied(
            &mut t,
            &Restyle {
                set: Restyle::UNDERLINE_ANY,
                clear: Restyle::UNDERLINE_ANY,
                ..Default::default()
            },
            Style::new().underline_dotted().bold(),
        );
        assert_eq!(new, Style::new().bold());
    }

    #[test]
    fn clearing_any_underline_takes_the_whole_field() {
        let mut t = Tables::new();
        let new = applied(
            &mut t,
            &Restyle {
                clear: Restyle::UNDERLINE_ANY,
                ..Default::default()
            },
            Style::new().underline_curly().bold(),
        );
        assert_eq!(new, Style::new().bold());
    }

    #[test]
    fn an_attribute_named_by_both_masks_ends_up_set() {
        let mut t = Tables::new();
        let new = applied(
            &mut t,
            &Restyle {
                set: Restyle::BOLD,
                clear: Restyle::BOLD,
                ..Default::default()
            },
            Style::new(),
        );
        assert_eq!(new, Style::new().bold());
    }

    #[test]
    fn bits_no_constant_names_are_ignored_rather_than_written_into_the_word() {
        // There is nothing above the style word's eleven attribute bits for 15..11 to be about.
        let mut t = Tables::new();
        let old = Style::new().bold();
        let new = applied(
            &mut t,
            &Restyle {
                set: 0xF800,
                clear: 0xF800,
                ..Default::default()
            },
            old,
        );
        assert_eq!(new, old);
    }

    #[test]
    fn a_descriptor_that_names_nothing_changes_nothing() {
        let mut t = Tables::new();
        let old = Style::new().bold().fg(Color::indexed(3));
        assert_eq!(applied(&mut t, &Restyle::default(), old), old);
        assert!(t.exts.is_empty());
    }

    #[test]
    fn a_descriptor_that_names_nothing_leaves_an_extended_cell_extended() {
        let mut t = Tables::new();
        let old = extended(&mut t);
        assert_eq!(applied(&mut t, &Restyle::default(), old), old);
        assert_eq!(t.exts.len(), 1, "and mints nothing a second time");
    }

    #[test]
    fn the_same_result_twice_is_the_same_handle() {
        // What makes a settled operator converge after one frame, and what keeps two cells that
        // look identical comparing equal.
        let mut t = Tables::new();
        let d = Restyle {
            link: Some(Link::Uri(URI)),
            ..Default::default()
        };
        let a = applied(&mut t, &d, Style::new().fg(Color::indexed(1)));
        let b = applied(&mut t, &d, Style::new().fg(Color::indexed(1)));
        assert_eq!(a, b);
        assert_eq!(t.exts.len(), 1);
    }

    #[test]
    fn the_same_uri_twice_is_the_same_id_and_mints_once() {
        // The other half of *this hyperlink again* now that no caller holds a handle: the token an
        // application lost is the URI it already had, and asking for it twice costs one entry.
        let mut t = Tables::new();
        let d = Restyle {
            link: Some(Link::Uri(URI)),
            ..Default::default()
        };
        let first = intern_link(&mut t, &d);
        let second = intern_link(&mut t, &d);
        assert_eq!(first, second);
        assert_eq!(t.links.entries().len(), 1);
    }

    #[test]
    fn a_descriptor_that_names_no_link_interns_nothing() {
        let mut t = Tables::new();
        assert_eq!(intern_link(&mut t, &Restyle::default()), None);
        assert_eq!(
            intern_link(
                &mut t,
                &Restyle {
                    link: Some(Link::None),
                    ..Default::default()
                }
            ),
            Some(LinkId::NONE),
            "`Link::None` is a clear, and a clear reaches no table"
        );
        assert!(t.links.is_empty());
    }
}
