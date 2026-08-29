//! The engine's own Unicode tables, and the two questions it asks of them.
//!
//! Where does one extended grapheme cluster end and the next begin (UAX #29), and how many
//! terminal columns does a cluster occupy (UAX #11, plus emoji presentation). `deny.toml` bans
//! `unicode-width` and `unicode-segmentation`; `build.rs` generates the tables instead, from the
//! UCD data vendored in `ucd/`.
//!
//! Nothing here is public. Ticket 06 exports `graphemes()` and `width_of()` over it, and no public
//! signature ever names a table.
//!
//! # Our tables are authoritative
//!
//! Terminals disagree with each other and with the standard — only 7 of 23 surveyed widen a VS16
//! emoji correctly, kitty sums a ZWJ family emoji to 6 where the answer is 2, Windows Terminal
//! draws a combining mark at width 1 — and the engine does not follow the terminal here. §8's
//! `CHA`-after-non-ASCII rule bounds the disagreement instead. See
//! `.scratch/vitui-engine-architecture/research/02-grapheme-clustering-and-width.md`.
//!
//! **Those three figures are a survey in a research document and not a measurement of ours**, and
//! two of them were checked on 2026-08-29 and did not reproduce: `conform/`'s scene 05 asks a
//! terminal what a cluster is worth with `CSI 6n`, and Ghostty 1.3.1, kitty 0.48.2 and tmux 3.7c
//! all widen VS16 correctly and all answer **2** for a ZWJ family. The decision is unchanged —
//! following the terminal was never the alternative, and a terminal that agrees today is not a
//! promise — but the evidence for the disagreement is older than the terminals it names. See
//! `conform/FINDINGS.md`, 2026-08-29.
//!
//! Three answers are policy rather than standard, and each is pinned by name in [`tests`]:
//! ambiguous width (UAX #11 class `A`) is **narrow**; a cluster's width is its base's width,
//! **never the sum** of its code points; and **VS15 does not change a width**, only a
//! presentation, which is what `terminal-unicode-core` says of the mode 2027 the engine requests.

/// The generated tables: three three-stage tries, `O(1)` lookup, no binary search on the path.
///
/// `UCD_VERSION` is stamped into the generated source and read only by the gates, which is where a
/// version is worth asserting on.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "UCD_VERSION is read by this module's gates")
)]
mod tables {
    include!(concat!(env!("OUT_DIR"), "/ucd_tables.rs"));
}

#[cfg(test)]
mod tests;

/// U+FE0F VARIATION SELECTOR-16 — forces emoji presentation, and with it width 2.
const VS16: char = '\u{FE0F}';

/// A code point's `Grapheme_Cluster_Break` value (UAX #29, table 2).
///
/// The discriminants are the codes `build.rs` writes into the `BREAK` table, and the two agree by
/// test rather than by inspection: `break_class` is re-derived for every code point straight from
/// `GraphemeBreakProperty.txt`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Break {
    /// Everything the property file does not name.
    Other = 0,
    /// `CR`.
    Cr = 1,
    /// `LF`.
    Lf = 2,
    /// `Control`.
    Control = 3,
    /// `Extend`.
    Extend = 4,
    /// `ZWJ`, U+200D.
    Zwj = 5,
    /// `Regional_Indicator`.
    RegionalIndicator = 6,
    /// `Prepend`.
    Prepend = 7,
    /// `SpacingMark`.
    SpacingMark = 8,
    /// Hangul `L`.
    L = 9,
    /// Hangul `V`.
    V = 10,
    /// Hangul `T`.
    T = 11,
    /// Hangul `LV`.
    Lv = 12,
    /// Hangul `LVT`.
    Lvt = 13,
}

/// A code point's `Indic_Conjunct_Break` value, which GB9c is written in terms of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Conjunct {
    /// Not part of an Indic conjunct.
    None = 0,
    /// A consonant, which both opens a conjunct and can close one.
    Consonant = 1,
    /// Transparent inside a conjunct. U+200D ZWJ is one of these.
    Extend = 2,
    /// The virama that makes a conjunct.
    Linker = 3,
}

/// The properties beyond `Grapheme_Cluster_Break` that UAX #29 and the VS16 rule need.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Aux {
    /// `Indic_Conjunct_Break`, for GB9c.
    pub(crate) conjunct: Conjunct,
    /// `Extended_Pictographic`, for GB11.
    pub(crate) ext_pict: bool,
    /// `Emoji` — what decides whether a VS16 in a cluster means anything. `#` is `Emoji` and is
    /// not `Extended_Pictographic`, and the keycap sequence is the case that separates the two.
    pub(crate) emoji: bool,
}

/// The columns one code point occupies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub(crate) enum Columns {
    /// A combining mark, a format character, a control: nothing advances.
    Zero = 0,
    /// One column.
    Narrow = 1,
    /// Two columns, by UAX #11 class `W` or `F`, or by default emoji presentation.
    Wide = 2,
}

/// The `Grapheme_Cluster_Break` value of one code point.
#[inline]
pub(crate) fn break_class(cp: char) -> Break {
    // `build.rs` only ever writes 0..=13, and `break_class_agrees_with_grapheme_break_property`
    // holds it to that over the whole codespace. The fallback keeps the lookup total rather than
    // introducing a panic path on the hot path.
    match tables::break_class(cp as u32) {
        1 => Break::Cr,
        2 => Break::Lf,
        3 => Break::Control,
        4 => Break::Extend,
        5 => Break::Zwj,
        6 => Break::RegionalIndicator,
        7 => Break::Prepend,
        8 => Break::SpacingMark,
        9 => Break::L,
        10 => Break::V,
        11 => Break::T,
        12 => Break::Lv,
        13 => Break::Lvt,
        _ => Break::Other,
    }
}

/// The `Indic_Conjunct_Break`, `Extended_Pictographic` and `Emoji` properties of one code point.
#[inline]
pub(crate) fn aux_class(cp: char) -> Aux {
    let bits = tables::aux_class(cp as u32);
    Aux {
        conjunct: match bits & 0b11 {
            1 => Conjunct::Consonant,
            2 => Conjunct::Extend,
            3 => Conjunct::Linker,
            _ => Conjunct::None,
        },
        ext_pict: bits & 0b100 != 0,
        emoji: bits & 0b1000 != 0,
    }
}

/// The columns one code point occupies, before the cluster it sits in has its say.
#[inline]
pub(crate) fn width_class(cp: char) -> Columns {
    match tables::width_class(cp as u32) {
        1 => Columns::Narrow,
        2 => Columns::Wide,
        _ => Columns::Zero,
    }
}

/// The state UAX #29 needs between two code points.
///
/// Three rules are not decidable from the adjacent pair alone, and each is a field here: GB9c
/// scans back over `[Extend Linker]*` for a consonant, GB11 needs to know whether a ZWJ was
/// preceded by `Extended_Pictographic Extend*`, and GB12/13 are a parity over the regional
/// indicators since the last code point that was not one.
///
/// The state is bounded, so segmentation is one linear pass with no backtracking.
#[derive(Clone, Debug, Default)]
pub(crate) struct Cursor {
    /// The previous code point's `Grapheme_Cluster_Break` value, absent at the start of text.
    prev: Option<Break>,
    /// Consecutive `Regional_Indicator` code points ending at the previous one (GB12/13).
    regional_indicators: usize,
    /// How far `Consonant [Extend Linker]* Linker` has been matched (GB9c).
    conjunct: ConjunctState,
    /// How far an `Extended_Pictographic Extend* ZWJ` prefix has been matched (GB11).
    pictographic: PictographicState,
}

/// How much of GB9c's left-hand side has been matched.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum ConjunctState {
    /// No consonant is open.
    #[default]
    Closed,
    /// A consonant is open, with no linker yet.
    Consonant,
    /// A consonant is open and a linker has been seen, so the next consonant joins.
    Linked,
}

/// How much of GB11's left-hand side has been matched.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum PictographicState {
    /// Nothing pictographic is open.
    #[default]
    Closed,
    /// Inside `Extended_Pictographic Extend*`.
    Open,
    /// A ZWJ closed it, so the next `Extended_Pictographic` joins.
    Joined,
}

impl Cursor {
    /// A cursor at the start of text.
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Whether a cluster boundary falls immediately before `cp`, advancing the state past it.
    pub(crate) fn is_break(&mut self, cp: char) -> bool {
        let class = break_class(cp);
        let aux = aux_class(cp);
        // GB1: break at the start of text.
        let is_break = self.prev.is_none_or(|prev| self.decide(prev, class, aux));
        self.advance(class, aux);
        is_break
    }

    /// GB3 through GB999, in the order UAX #29 states them.
    fn decide(&self, prev: Break, class: Break, aux: Aux) -> bool {
        use Break::{
            Control, Cr, Extend, L, Lf, Lv, Lvt, Prepend, RegionalIndicator, SpacingMark, T, V, Zwj,
        };

        // GB3, GB4, GB5: CR LF is one cluster, and a control stands alone.
        if prev == Cr && class == Lf {
            return false;
        }
        if matches!(prev, Control | Cr | Lf) || matches!(class, Control | Cr | Lf) {
            return true;
        }

        // GB6, GB7, GB8: Hangul syllables.
        if prev == L && matches!(class, L | V | Lv | Lvt) {
            return false;
        }
        if matches!(prev, Lv | V) && matches!(class, V | T) {
            return false;
        }
        if matches!(prev, Lvt | T) && class == T {
            return false;
        }

        // GB9, GB9a, GB9b: what attaches to the left, and what attaches to the right.
        if matches!(class, Extend | Zwj | SpacingMark) || prev == Prepend {
            return false;
        }

        // GB9c: a virama-linked Indic consonant cluster is one grapheme.
        if self.conjunct == ConjunctState::Linked && aux.conjunct == Conjunct::Consonant {
            return false;
        }

        // GB11: Extended_Pictographic Extend* ZWJ x Extended_Pictographic — the rule that keeps a
        // ZWJ family emoji in one cell.
        if self.pictographic == PictographicState::Joined && aux.ext_pict {
            return false;
        }

        // GB12, GB13: regional indicators pair off, so one flag is one cluster and two are two.
        if prev == RegionalIndicator
            && class == RegionalIndicator
            && self.regional_indicators % 2 == 1
        {
            return false;
        }

        // GB999.
        true
    }

    /// Folds one code point into the three accumulated rules.
    fn advance(&mut self, class: Break, aux: Aux) {
        self.prev = Some(class);

        self.regional_indicators = if class == Break::RegionalIndicator {
            self.regional_indicators + 1
        } else {
            0
        };

        self.conjunct = match (aux.conjunct, self.conjunct) {
            (Conjunct::Consonant, _) => ConjunctState::Consonant,
            (_, ConjunctState::Closed) => ConjunctState::Closed,
            (Conjunct::Linker, _) => ConjunctState::Linked,
            (Conjunct::Extend, open) => open,
            (Conjunct::None, _) => ConjunctState::Closed,
        };

        self.pictographic = match (class, aux.ext_pict, self.pictographic) {
            (Break::Zwj, _, PictographicState::Open) => PictographicState::Joined,
            (_, true, _) => PictographicState::Open,
            (Break::Extend, _, PictographicState::Open) => PictographicState::Open,
            _ => PictographicState::Closed,
        };
    }
}

/// A printable ASCII byte: `0x20..=0x7E`.
///
/// The fast path's whole predicate. **Two printable ASCII bytes always break between them**, and no
/// UAX #29 rule can join them: `Extend`, `SpacingMark`, `ZWJ` and `Prepend` are all non-ASCII, so
/// are Hangul jamo, regional indicators and Indic consonants, and the one ASCII rule — GB3's
/// `CR LF` — is about control bytes, which this range excludes. That is what lets the segmenter
/// take one byte and skip the tables entirely, and it is asserted over the whole 95x95 grid in
/// [`tests`] rather than argued.
#[inline]
const fn is_ascii_printable(b: u8) -> bool {
    b >= 0x20 && b < 0x7F
}

/// The extended grapheme clusters of `s`, as slices borrowed from it.
pub(crate) fn clusters(s: &str) -> Clusters<'_> {
    Clusters {
        rest: s,
        scanned: 0,
        cursor: Cursor::new(),
    }
}

/// The iterator [`clusters`] returns. Borrows the caller's string; never allocates.
///
/// # The cursor is carried across boundaries, and that is worth a factor of two
///
/// Restarting it at every cluster is *sound* — each accumulated rule matches a pattern whose
/// interior contains no boundary — and it costs every code point being classified **twice**, once
/// as the last character looked at for one cluster and again as the first character of the next.
/// Classification is two three-stage trie walks, so a screen of CJK paid six dependent loads it did
/// not need per cell. Carrying the cursor makes it once, and the boundary character is remembered
/// in `scanned` rather than re-read.
#[derive(Clone, Debug)]
pub(crate) struct Clusters<'a> {
    /// What has not been returned yet.
    rest: &'a str,
    /// Bytes at the front of `rest` the cursor has already consumed: zero, or one character.
    scanned: usize,
    cursor: Cursor,
}

impl<'a> Iterator for Clusters<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if self.rest.is_empty() {
            return None;
        }

        // The ASCII fast path (spec §4). Nearly every cell ever written takes it, and it costs one
        // compare against the byte after: the tables, the cursor and the UTF-8 decode are all
        // skipped, and what is skipped is what made a screen of Latin cost 20.8 ns a cell.
        //
        // It tests the *next* byte as well, and that is the whole of its correctness: `e` followed
        // by a combining acute is one cluster, and the second byte of U+0301 is not ASCII, so this
        // path is not taken. Resetting the cursor is right because a fresh one means start-of-text,
        // and a printable ASCII character always breaks before whatever a printable ASCII character
        // can be followed by here.
        let bytes = self.rest.as_bytes();
        if is_ascii_printable(bytes[0]) && bytes.get(1).is_none_or(|b| is_ascii_printable(*b)) {
            let (cluster, rest) = self.rest.split_at(1);
            self.rest = rest;
            self.scanned = 0;
            self.cursor = Cursor::new();
            return Some(cluster);
        }

        if self.scanned == 0 {
            let first = self.rest.chars().next().expect("`rest` is not empty");
            self.cursor.is_break(first);
            self.scanned = first.len_utf8();
        }

        let end = loop {
            let Some(ch) = self.rest[self.scanned..].chars().next() else {
                break self.rest.len();
            };
            if self.cursor.is_break(ch) {
                break self.scanned;
            }
            self.scanned += ch.len_utf8();
        };

        let (cluster, rest) = self.rest.split_at(end);
        self.rest = rest;
        // The character at `end` is already through the cursor; only its length is carried.
        self.scanned = rest.chars().next().map_or(0, char::len_utf8);
        Some(cluster)
    }
}

/// The columns one extended grapheme cluster occupies.
///
/// The rules, in the order they apply:
///
/// 1. A **regional-indicator pair** is one flag and two columns; a lone one is a letter in a box
///    and takes one.
/// 2. The **base** is the first code point that is not zero-width, so a `Prepend` or a leading
///    format character cannot swallow the cluster's width.
/// 3. **VS16** over an `Emoji` base is two columns. This is the case only 7 of 23 surveyed
///    terminals get right, and the engine does not follow them.
/// 4. Otherwise the base's own class — **never a sum**, which is what makes a ZWJ family emoji 2
///    rather than kitty's 6.
///
/// **VS15 is not a rule here, and its absence is the decision.** `terminal-unicode-core`, the
/// mode 2027 spec: VS15 "forces the grapheme cluster to emoji text presentation. This will
/// **NOT** change the underlying width, but only change the display." The opposite rule — narrow
/// a default-emoji base back to one column — is `unicode-width`'s, and taking it would put the
/// engine's own answer at odds with the protocol it asks the terminal to speak.
pub(crate) fn cluster_width(cluster: &str) -> u16 {
    // The same fast path as the segmenter's, for the same reason: printable ASCII is one column and
    // no rule below can say otherwise.
    if cluster.len() == 1 && is_ascii_printable(cluster.as_bytes()[0]) {
        return 1;
    }

    // One scalar, which is every CJK cell, every box-drawing cell and every single-scalar emoji.
    // Rules 1 and 3 need two code points to apply — a lone regional indicator is the one exception
    // and it is tested for by name — so the base *is* the cluster and its class is the answer.
    let mut chars = cluster.chars();
    if let (Some(only), None) = (chars.next(), chars.next()) {
        if break_class(only) == Break::RegionalIndicator {
            return 1;
        }
        return width_class(only) as u16;
    }

    let Some(base) = cluster.chars().find(|cp| width_class(*cp) != Columns::Zero) else {
        return 0;
    };

    if break_class(base) == Break::RegionalIndicator {
        let indicators = cluster
            .chars()
            .filter(|cp| break_class(*cp) == Break::RegionalIndicator)
            .count();
        return if indicators >= 2 { 2 } else { 1 };
    }

    if cluster.contains(VS16) && aux_class(base).emoji {
        return 2;
    }

    if width_class(base) == Columns::Wide {
        2
    } else {
        1
    }
}
