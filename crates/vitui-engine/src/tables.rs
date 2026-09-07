//! The handle tables, as one thing: what a drawing verb reaches through.
//!
//! Spec §3 calls them *the handle tables* in the plural and treats them as a set — **one set per
//! layer stack**, minted by `attach`, reached by the verbs through the draw context.
//! They travel together everywhere in this crate: `text` needs the interner, `restyle` needs the
//! extended-style table and the link table, and `pack` needs all three at once. So they are one
//! struct rather than three arguments threaded past each other, and a `View` borrows one field
//! instead of three.
//!
//! A `Surface` outside a layer stack carries its own set, because `Surface::new` and
//! `Surface::root` are public and a `View` from that door has no engine to reach through (spec §3,
//! architecture ticket 19). Every table in a fresh set is empty and an empty table holds no
//! allocation, so the field costs a layer's surface nothing.

use crate::exts::{ExtStyles, LinkId, Links};
use crate::intern::Interner;

/// The grapheme interner, the extended-style table and the link table: one handle space.
///
/// **`Default` is written out rather than derived, and the reason is one field.**
/// [`links_in_key`](Tables::links_in_key) is `true` in a fresh set — a set knows no terminal, so it
/// interns everything and the screen's answer is applied where a surface is reconciled — and a
/// derived `Default` would answer `false`, which is the *collapsing* behaviour. Two constructors
/// disagreeing about one boolean is a wrong table with no failing test, so there is one constructor
/// and `Default` delegates to it.
#[derive(Debug)]
pub(crate) struct Tables {
    pub(crate) interner: Interner,
    pub(crate) exts: ExtStyles,
    pub(crate) links: Links,
    /// Whether a hyperlink is part of an extended style's **identity**.
    ///
    /// **False on a terminal with no OSC 8, and that is spec §10's one narrow exception to
    /// *degrade at serialise time*:**
    ///
    /// > A channel the terminal **cannot express at all** is dropped from the intern *key* on the
    /// > app thread; a channel it expresses **imprecisely** is degraded at serialise time.
    ///
    /// Colour is always the second kind — quantisation is depth-dependent and belongs on the render
    /// thread ([`crate::quant`]) — and a hyperlink is the only instance of the first, because
    /// `hyperlinks` is the only capability that is boolean rather than a ladder and the only channel
    /// whose absence makes a cell *stop needing a table entry at all*.
    ///
    /// What it is worth is **table entries and not bytes**: spec §7 measured 8 against 96 on a page
    /// of hyperlinked text, at byte-identical frames, because two style words differing only in a
    /// channel the serializer will not emit produce an SGR delta with nothing in it and the emit loop
    /// already withdraws the escape it speculatively opened. So this is a growth bound rather than a
    /// frame cost, and it is free, which is why it is taken.
    ///
    /// **True in a fresh set, which is the honest default and not the common one.** A set knows no
    /// terminal — `Surface::new` is public and a standalone surface has no engine to ask (spec §3,
    /// architecture ticket 19) — and a surface drawn off to one side is reconciled at donation, where
    /// `add_content_with` re-mints every entry into the stack's table and the collapse applies there.
    /// `attach` is what tells the stack's own set, through
    /// [`set_links_in_key`](Tables::set_links_in_key).
    links_in_key: bool,
}

impl Default for Tables {
    fn default() -> Tables {
        Tables::new()
    }
}

impl Tables {
    pub(crate) fn new() -> Tables {
        Tables {
            interner: Interner::new(),
            exts: ExtStyles::new(),
            links: Links::new(),
            links_in_key: true,
        }
    }

    /// Tell this handle space whether the terminal can express a hyperlink at all.
    ///
    /// Called once, by `attach`, on the layer stack's own set. See
    /// [`links_in_key`](Tables::links_in_key).
    pub(crate) fn set_links_in_key(&mut self, yes: bool) {
        self.links_in_key = yes;
    }

    /// Keep the hyperlink in the key on a terminal that has no OSC 8: **spec §7's rejected
    /// placement**, which is where the same degradation happens at serialise time only.
    ///
    /// It exists so the claim *the collapse is worth eight entries against ninety-six and not one
    /// byte* is **reproduced rather than quoted** — one arm cannot support a statement about two,
    /// which is the same argument [`Filter`](crate::serial::Filter) has three variants that lost
    /// for. Nothing ships it: the shipping value comes from `attach` and from nowhere else.
    #[cfg(test)]
    pub(crate) fn keep_links_in_key(&mut self) {
        self.links_in_key = true;
    }

    /// The key an extended style is interned under: the entry itself, or the entry with its
    /// hyperlink dropped where this terminal has no OSC 8.
    ///
    /// One function rather than a flag every intern site consults, because there are two sites —
    /// `restyle::apply` and the donation walk — and a collapse applied at one of them would produce
    /// two handles for one wire cell, which is exactly the equality the mirror rests on.
    pub(crate) fn key(&self, e: crate::exts::ExtStyle) -> crate::exts::ExtStyle {
        if self.links_in_key {
            e
        } else {
            crate::exts::ExtStyle {
                link: LinkId::NONE,
                ..e
            }
        }
    }

    /// The id for one URI in this handle space, minting one if this is the first sight of it.
    pub(crate) fn link(&mut self, uri: &str) -> LinkId {
        self.links.mint(uri)
    }

    /// Whether all three tables have never been reached.
    ///
    /// **The test `add_content_with` makes before it walks a donated surface**, and the reason the
    /// walk is skipped for nearly every donation: a surface of Latin, CJK, box drawing or
    /// single-scalar emoji carries no cluster, and one that is neither hyperlinked nor
    /// coloured-underlined carries no extended style, so there is no handle in it that means
    /// anything different in the stack's space than it did in its own (spec §3, architecture
    /// ticket 19).
    pub(crate) fn is_empty(&self) -> bool {
        self.interner.is_empty() && self.exts.is_empty() && self.links.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_set_of_tables_is_three_empty_ones() {
        let t = Tables::new();
        assert!(t.interner.is_empty());
        assert!(t.exts.is_empty());
        assert!(t.links.is_empty());
    }

    #[test]
    fn the_two_constructors_agree_about_the_one_field_that_is_not_a_table() {
        // `Default` is written out rather than derived, because a derived one would answer `false`
        // for `links_in_key` and that is the collapsing behaviour. Two constructors disagreeing
        // about one boolean is a wrong table with no failing test, so here is the test.
        assert_eq!(Tables::default().links_in_key, Tables::new().links_in_key);
        assert!(Tables::new().links_in_key, "a fresh set knows no terminal");
    }

    #[test]
    fn one_uri_has_one_id_per_handle_space() {
        let mut ours = Tables::new();
        assert_eq!(
            ours.link("https://example.com/"),
            ours.link("https://example.com/")
        );
    }
}
