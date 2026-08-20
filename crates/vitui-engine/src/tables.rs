//! The handle tables, as one thing: what a drawing verb reaches through.
//!
//! Spec §3 calls them *the handle tables* in the plural and treats them as a set — **one set per
//! layer stack**, minted by `attach`, reached by the verbs through the draw context (ADR 0011).
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
#[derive(Debug, Default)]
pub(crate) struct Tables {
    pub(crate) interner: Interner,
    pub(crate) exts: ExtStyles,
    pub(crate) links: Links,
}

impl Tables {
    pub(crate) fn new() -> Tables {
        Tables {
            interner: Interner::new(),
            exts: ExtStyles::new(),
            links: Links::new(),
        }
    }

    /// The id for one URI in this handle space, minting one if this is the first sight of it.
    pub(crate) fn link(&mut self, uri: &str) -> LinkId {
        self.links.mint(uri)
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
    fn one_uri_has_one_id_per_handle_space() {
        let mut ours = Tables::new();
        assert_eq!(
            ours.link("https://example.com/"),
            ours.link("https://example.com/")
        );
    }
}
