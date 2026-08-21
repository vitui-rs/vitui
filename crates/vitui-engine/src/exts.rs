//! The extended-style table and the link table: the two channels that did not fit in a `u64`.
//!
//! # Why they are behind a bit rather than beside the style word
//!
//! Underline colour and the OSC 8 hyperlink are the two things spec §3 could not fit in sixty-four
//! bits, and keeping an `extras` handle *inline* beside the colours costs the serializer's
//! full-screen scan 5.45 → 13.3 µs. Folding them behind bit 63 pays for itself 2.4x, because the
//! interning lands only on the **under 1%** of cells that carry one: everything else is an inline
//! word and compares in one instruction.
//!
//! Because both tables are deduplicated, two cells carrying the same hyperlink hold the same handle
//! and still compare correctly with one `u64` compare. That is what keeps the equality the mirror
//! and ticket 14's filter rest on exact.
//!
//! # Where they live
//!
//! Beside the interner, in [`Tables`](crate::tables::Tables) — one set per layer stack, minted by
//! `attach` and reached by the verbs through the draw context (ADR 0011). A `Surface` outside a
//! stack carries its own set, and both tables stay empty for every cell that is neither hyperlinked
//! nor coloured-underlined, which is nearly all of them: `Vec::new` and `HashMap::new` take no
//! allocation until the first insert.

use std::collections::HashMap;

use crate::style::Color;

/// A hyperlink's identity, stable for the life of the table that minted it.
///
/// Opaque: no public field, no `From<u32>`, and [`Screen::link`](crate::Screen::link) is the only
/// mint. A caller must be able to say *this hyperlink again* without being able to say *entry 7* —
/// exactly the rule [`LayerId`](crate::LayerId) already follows, and the reason this does not
/// breach `docs/adr/0023-the-cell-is-never-visible-in-the-public-api.md`: it names a handle's
/// identity and exposes no table, no index arithmetic and no way to build one from a number.
///
/// The last clause is gated rather than promised, and paired with a positive twin naming the type
/// by path so that renaming it away breaks both halves rather than quietly satisfying the first:
///
/// ```compile_fail,E0423
/// let _ = vitui_engine::LinkId(7);
/// ```
///
/// ```
/// let config = vitui_engine::Config {
///     // Headless, because a doctest must not reach for the developer's terminal.
///     output: vitui_engine::Output::Sink(Box::new(Vec::new())),
///     ..Default::default()
/// };
/// let (mut screen, _wake) = vitui_engine::Engine::new(config).attach().unwrap();
/// let _: vitui_engine::LinkId = screen.link("https://example.com/");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct LinkId(u32);

impl LinkId {
    /// No hyperlink. What a cell holds unless something put one there, and what
    /// [`Restyle::link`](crate::Restyle::link) is set to in order to take one away.
    pub const NONE: LinkId = LinkId(0);

    /// Whether this names a hyperlink at all.
    pub(crate) const fn is_none(self) -> bool {
        self.0 == LinkId::NONE.0
    }

    /// The index into [`Links`], for an id that names one.
    pub(crate) const fn index(self) -> Option<usize> {
        if self.is_none() {
            None
        } else {
            Some(self.0 as usize - 1)
        }
    }
}

/// The URIs one handle space knows about.
///
/// Ids start at one, because zero is [`LinkId::NONE`] and a table whose first entry is the absence
/// of an entry is a table with an off-by-one waiting in it.
#[derive(Debug, Default)]
pub(crate) struct Links {
    uris: Vec<Box<str>>,
    index: HashMap<Box<str>, LinkId>,
}

impl Links {
    pub(crate) fn new() -> Links {
        Links::default()
    }

    /// The id for one URI, minting one if this is the first sight of it.
    ///
    /// Deduplicated, because §3's sentence — *link ids are few, an application holds handles to
    /// them* — is only true if asking twice for the same URI answers the same thing. A page of a
    /// hundred distinct links is a hundred entries however many cells carry them.
    pub(crate) fn mint(&mut self, uri: &str) -> LinkId {
        if let Some(id) = self.index.get(uri) {
            return *id;
        }
        let id = LinkId(u32::try_from(self.uris.len() + 1).expect("a link id fits in a u32"));
        let owned: Box<str> = uri.into();
        self.uris.push(owned.clone());
        self.index.insert(owned, id);
        id
    }

    /// The URI an id names. `None` for [`LinkId::NONE`] and for an id this table never minted.
    pub(crate) fn uri(&self, id: LinkId) -> Option<&str> {
        self.uris.get(id.index()?).map(|u| &**u)
    }

    /// Whether this table has never been reached.
    pub(crate) fn is_empty(&self) -> bool {
        self.uris.is_empty()
    }

    /// Every URI this table holds, in id order — so index `i` is [`LinkId`] `i + 1`.
    ///
    /// The donation walk re-mints these into the stack's table before it touches an extended-style
    /// entry, because an entry names a `LinkId` and re-interning it against the donor's id would
    /// dedup on the wrong key (ticket 10).
    pub(crate) fn entries(&self) -> impl ExactSizeIterator<Item = &str> {
        self.uris.iter().map(|u| &**u)
    }
}

/// One entry of the extended-style table: the four channels an extended style word points at.
///
/// `fg` and `bg` are here as well as the two that forced the fold, and that is the whole point of
/// the fold: an extended word spends bits 51..0 on the handle, so the colours have nowhere inline
/// left to be. A cell is extended **iff** it carries an underline colour or a hyperlink — anything
/// else goes back inline, which is what keeps the table from growing for free.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub(crate) struct ExtStyle {
    pub(crate) fg: Color,
    pub(crate) bg: Color,
    /// The underline's own colour. [`Color::DEFAULT`] means *the text's colour*, which is SGR 59.
    pub(crate) ul: Color,
    pub(crate) link: LinkId,
}

impl ExtStyle {
    /// Whether this entry still needs a table entry at all.
    ///
    /// False when both extended channels are clear, and then the style goes back inline —
    /// *extended is a cost, not a state*, and this predicate is that sentence.
    pub(crate) fn is_extended(self) -> bool {
        self.ul != Color::DEFAULT || !self.link.is_none()
    }
}

/// The extended styles one handle space knows about.
#[derive(Debug, Default)]
pub(crate) struct ExtStyles {
    /// Indexed by handle. Never shrinks: a handle is stable for the life of the table, and the
    /// mark-and-compact sweep that renumbers it is spec §3's, owned by ticket 08.
    ///
    /// **Ticket 07 is what makes the growth reachable from public API, one ticket ahead of the
    /// sweep that bounds it.** A caller whose descriptor names a *changing* colour over hyperlinked
    /// cells mints one entry per distinct result per frame, for the life of the process — a fade is
    /// about 1 700 entries and a pulsing dim about 5 700 a second (spec §3). A settled one converges
    /// after a single frame, which is the common case and what `tests/alloc.rs` gates. Register
    /// entry #7 and the `hyperlinked-page-under-an-animating-operator` scene are red against ticket
    /// 08 for exactly this, so it is a deferral rather than an oversight.
    entries: Vec<ExtStyle>,
    index: HashMap<ExtStyle, u32>,
}

impl ExtStyles {
    pub(crate) fn new() -> ExtStyles {
        ExtStyles::default()
    }

    /// The handle for one extended style, minting one if this is the first sight of it.
    ///
    /// Deduplicated, which is what lets two extended cells compare with one `u64` compare — and
    /// what makes a settled operator converge after one frame: every later frame asks for entries
    /// that are already there.
    pub(crate) fn handle(&mut self, e: ExtStyle) -> u32 {
        if let Some(h) = self.index.get(&e) {
            return *h;
        }
        let h = u32::try_from(self.entries.len()).expect("an extended-style handle fits in a u32");
        self.entries.push(e);
        self.index.insert(e, h);
        h
    }

    /// The entry a handle names, or `None` for a handle this table never minted.
    pub(crate) fn get(&self, h: u32) -> Option<ExtStyle> {
        self.entries.get(h as usize).copied()
    }

    /// How many distinct extended styles this table holds.
    ///
    /// What ticket 08's growth measurement counts, and what the gates here assert converges.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "ticket 08 owns the growth measurement this is the counter for"
        )
    )]
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether this table has never been reached.
    pub(crate) fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Every extended style this table holds, in handle order.
    ///
    /// Read by the donation walk, which re-interns each entry — with its `link` already renumbered
    /// — into the stack's table and indexes the result by the donor's own handle (ticket 10).
    pub(crate) fn entries(&self) -> impl ExactSizeIterator<Item = ExtStyle> {
        self.entries.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn linked(link: LinkId) -> ExtStyle {
        ExtStyle {
            fg: Color::DEFAULT,
            bg: Color::DEFAULT,
            ul: Color::DEFAULT,
            link,
        }
    }

    #[test]
    fn a_fresh_link_table_holds_no_allocation() {
        assert!(Links::new().is_empty());
    }

    #[test]
    fn the_absent_link_names_no_uri() {
        let links = Links::new();
        assert_eq!(links.uri(LinkId::NONE), None);
        assert!(LinkId::NONE.is_none());
        assert_eq!(LinkId::default(), LinkId::NONE);
    }

    #[test]
    fn the_same_uri_twice_is_the_same_id() {
        let mut links = Links::new();
        let first = links.mint("https://example.com/a");
        let second = links.mint("https://example.com/a");
        assert_eq!(first, second);
        assert_eq!(links.uris.len(), 1, "the second sight minted nothing");
    }

    #[test]
    fn a_minted_id_is_never_the_absent_one() {
        // Ids start at one, so `LinkId::NONE` can never collide with a real link — which is what
        // makes `Some(LinkId::NONE)` a clear rather than a link to whatever entry zero happens
        // to be.
        let mut links = Links::new();
        let id = links.mint("https://example.com/");
        assert_ne!(id, LinkId::NONE);
        assert_eq!(links.uri(id), Some("https://example.com/"));
    }

    #[test]
    fn an_id_from_one_table_is_meaningless_in_another() {
        let ours = Links::new();
        let mut theirs = Links::new();
        theirs.mint("https://example.com/a");
        let stranger = theirs.mint("https://example.com/b");
        assert_eq!(ours.uri(stranger), None, "id 2 does not exist here yet");
    }

    #[test]
    fn a_fresh_extended_style_table_holds_no_allocation() {
        assert!(ExtStyles::new().is_empty());
    }

    #[test]
    fn the_same_extended_style_twice_is_the_same_handle() {
        let mut exts = ExtStyles::new();
        let e = linked(LinkId(7));
        assert_eq!(exts.handle(e), exts.handle(e));
        assert_eq!(exts.len(), 1);
    }

    #[test]
    fn two_extended_styles_differing_in_one_channel_are_two_handles() {
        let mut exts = ExtStyles::new();
        let a = linked(LinkId(1));
        let b = ExtStyle {
            ul: Color::rgb(1, 2, 3),
            ..a
        };
        let (ha, hb) = (exts.handle(a), exts.handle(b));
        assert_ne!(ha, hb);
        assert_eq!(exts.get(hb), Some(b));
        assert_eq!(exts.get(ha), Some(a));
    }

    #[test]
    fn an_entry_with_neither_extended_channel_does_not_need_the_table() {
        // The pressure valve, as a predicate: `restyle` asks this before it interns, so a
        // descriptor that clears both channels puts the cell back inline instead of minting a
        // handle nobody needed.
        let inline = ExtStyle {
            fg: Color::rgb(9, 9, 9),
            bg: Color::indexed(4),
            ul: Color::DEFAULT,
            link: LinkId::NONE,
        };
        assert!(!inline.is_extended());
        assert!(linked(LinkId(1)).is_extended());
        assert!(
            ExtStyle {
                ul: Color::rgb(0, 0, 0),
                ..inline
            }
            .is_extended(),
            "black is a colour; only DEFAULT means `the text's own`"
        );
    }

    #[test]
    fn a_handle_this_table_never_minted_resolves_to_nothing() {
        assert_eq!(ExtStyles::new().get(0), None);
    }
}
