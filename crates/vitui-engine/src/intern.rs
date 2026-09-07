//! The grapheme interner: the table a handle above `0x0010_FFFF` points into.
//!
//! # Where it lives
//!
//! **One per layer stack**, minted by `attach`, reached by the drawing verbs through the draw
//! context — which on the engine's side of the seam is the [`View`](crate::View). A `Surface`
//! outside a stack carries one of its own, because `Surface::new` and `Surface::root` are public and
//! a `View` from that door has no engine to reach through; that table stays empty for every string
//! of Latin, CJK, box drawing and single-scalar emoji, which is nearly all of them. See spec §3 and
//! `docs/adr/0011-the-handle-tables-belong-to-the-engine-and-the-packet-carries-copies.md`, amended
//! by `.scratch/vitui-engine-architecture/issues/19-the-standalone-surface-and-the-interner.md`.
//!
//! # Interning is identity for single scalars, and that is what makes the rule free
//!
//! A cell holding `a` or `漢` *is* its own handle, with no table and no hash. The table is
//! reached only by a cluster of more than one scalar — a combining mark, a ZWJ sequence, a flag, an
//! Indic conjunct, a VS16 emoji — which is well under 1% of the cells on a realistic screen. An
//! empty interner holds no allocation at all: `Vec::new` and `HashMap::new` both take none until the
//! first insert, which is what lets every surface carry one without paying for it.
//!
//! # The bytes are stored twice, deliberately
//!
//! Once in `clusters` to resolve a handle, once as the map's key to find one. The alternative is a
//! shared `Arc<str>` and `Surface` holds no `Arc` — so the copy is the price of keeping
//! the surface donatable. It is paid per *distinct* cluster, of which a hyperlinked full screen has
//! about a hundred, and never per cell.

use std::collections::HashMap;

use crate::cell::GraphemeId;
use crate::sweep;
use crate::ucd;

/// The clusters one handle space knows about.
#[derive(Debug, Default)]
pub(crate) struct Interner {
    /// Indexed by cluster id. A handle is stable until [`compact`](Interner::compact) renumbers
    /// it, which is spec §3's mark-and-compact sweep and happens where allocation is already
    /// permitted, never inside a frame.
    clusters: Vec<Box<str>>,
    index: HashMap<Box<str>, u32>,
}

impl Interner {
    pub(crate) fn new() -> Interner {
        Interner::default()
    }

    /// Whether this table has never been reached.
    ///
    /// What `add_content_with` tests before walking a donated surface: an empty table means no cell
    /// carries a handle that needs renumbering, which is the common case.
    pub(crate) fn is_empty(&self) -> bool {
        self.clusters.is_empty()
    }

    /// Every cluster this table holds, in handle order.
    ///
    /// The donation walk's first phase: `add_content_with` re-interns each of these into the
    /// stack's table **once** and indexes the result by the donor's own id, so the per-cell pass
    /// after it is an array lookup rather than a hash of the cluster's bytes. A donated surface of
    /// a thousand combining-mark cells holds a handful of distinct clusters, and the walk should
    /// cost the handful rather than the thousand.
    pub(crate) fn entries(&self) -> impl ExactSizeIterator<Item = &str> {
        self.clusters.iter().map(|c| &**c)
    }

    /// The handle for one extended grapheme cluster, minting one if this is the first sight of it.
    ///
    /// `None` for a cluster that occupies no column — a lone combining mark, a format character, a
    /// control. Spec §3's width table says *nothing advances*, so there is no cell to put it in and
    /// the verb steps over it.
    pub(crate) fn handle(&mut self, cluster: &str) -> Option<GraphemeId> {
        // The path nearly every cell takes: one byte of printable ASCII, one column, its own handle.
        if let [b] = *cluster.as_bytes()
            && (b.is_ascii_graphic() || b == b' ')
        {
            return Some(GraphemeId::scalar(b as char));
        }
        let width = ucd::cluster_width(cluster);
        if width == 0 {
            return None;
        }
        let wide = width == 2;

        // The fast path, and the reason a screen of CJK never reaches the table.
        let mut chars = cluster.chars();
        if let (Some(c), None) = (chars.next(), chars.next()) {
            return Some(if wide {
                GraphemeId::wide_scalar(c)
            } else {
                GraphemeId::scalar(c)
            });
        }

        let id = match self.index.get(cluster) {
            Some(id) => *id,
            None => {
                let id = u32::try_from(self.clusters.len()).expect("a cluster id fits in a u32");
                assert!(
                    GraphemeId::FIRST_CLUSTER + id <= GraphemeId::LAST_CLUSTER,
                    "the cluster table is full"
                );
                let owned: Box<str> = cluster.into();
                self.clusters.push(owned.clone());
                self.index.insert(owned, id);
                id
            }
        };
        Some(GraphemeId::cluster(id, wide))
    }

    /// The bytes a handle names: the scalar it *is*, or the cluster it points at.
    ///
    /// `None` for `EMPTY` and `CONTINUATION`, which name no text. The scalar case writes through
    /// `scratch` because a `char` has nowhere else to live long enough to be returned as a `&str`.
    pub(crate) fn render<'a>(&'a self, g: GraphemeId, scratch: &'a mut [u8; 4]) -> Option<&'a str> {
        if let Some(id) = g.cluster_id() {
            return self.clusters.get(id as usize).map(|c| &**c);
        }
        let ch = g.as_scalar()?;
        Some(ch.encode_utf8(scratch))
    }

    /// How many clusters this table holds.
    ///
    /// The marker slot count the sweep allocates, and the number its high-water mark is compared
    /// against (spec §3, [`crate::sweep`]).
    pub(crate) fn len(&self) -> usize {
        self.clusters.len()
    }

    /// Drop every cluster `live` does not mark, keeping the survivors **in table order**.
    ///
    /// Returns the new id for each old id, with [`sweep::DEAD`](crate::sweep::DEAD) where the entry
    /// was dropped — or `None` when no survivor moved, which is the case a sweep does not have to
    /// rewrite a single cell for. Spec §3: *a sweep that frees nothing below a live entry does not
    /// renumber at all.*
    pub(crate) fn compact(&mut self, live: &[bool]) -> Option<Vec<u32>> {
        debug_assert_eq!(
            live.len(),
            self.clusters.len(),
            "the marker has one slot per table entry"
        );
        let sweep::Remap { map, renumbered } = sweep::remap(live)?;
        // `swap` rather than `remove`: everything below `kept` is already final, so the entry
        // swapped up to `old` is always one that is about to be truncated away.
        let mut kept = 0;
        for (old, &alive) in live.iter().enumerate() {
            if alive {
                self.clusters.swap(kept, old);
                kept += 1;
            }
        }
        self.clusters.truncate(kept);
        // The keys are unchanged, so the index is rewritten in place rather than rebuilt: the bytes
        // that hashed to a bucket still hash to it, and only the id beside them moved.
        self.index.retain(|_, id| {
            *id = map[*id as usize];
            *id != sweep::DEAD
        });
        renumbered.then_some(map)
    }

    /// The bytes a handle names, for a handle that names a cluster or a scalar that is already
    /// somewhere. Test-facing convenience over [`Interner::render`].
    #[cfg(test)]
    pub(crate) fn resolve(&self, g: GraphemeId) -> Option<String> {
        let mut scratch = [0u8; 4];
        self.render(g, &mut scratch).map(str::to_owned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_fresh_interner_holds_no_allocation() {
        let i = Interner::new();
        assert!(i.is_empty());
    }

    #[test]
    fn a_single_scalar_is_its_own_handle_and_never_enters_the_table() {
        let mut i = Interner::new();
        assert_eq!(i.handle("a"), Some(GraphemeId::scalar('a')));
        assert_eq!(i.handle("漢"), Some(GraphemeId::wide_scalar('漢')));
        assert!(i.is_empty());
    }

    #[test]
    fn the_same_cluster_twice_is_the_same_handle() {
        let mut i = Interner::new();
        let first = i.handle("e\u{301}").unwrap();
        let second = i.handle("e\u{301}").unwrap();
        assert_eq!(first, second);
        assert_eq!(i.clusters.len(), 1, "the second sight minted nothing");
    }

    #[test]
    fn two_different_clusters_are_two_handles() {
        let mut i = Interner::new();
        let a = i.handle("e\u{301}").unwrap();
        let b = i.handle("a\u{308}").unwrap();
        assert_ne!(a, b);
        assert_eq!(i.resolve(a).as_deref(), Some("e\u{301}"));
        assert_eq!(i.resolve(b).as_deref(), Some("a\u{308}"));
    }

    #[test]
    fn a_zwj_family_emoji_is_one_wide_handle_that_round_trips() {
        let family = "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}";
        let mut i = Interner::new();
        let g = i.handle(family).unwrap();
        assert!(g.is_wide_head());
        assert_eq!(g.columns(), 2);
        assert_eq!(i.resolve(g).as_deref(), Some(family));
    }

    #[test]
    fn a_cluster_that_occupies_no_column_has_no_handle() {
        let mut i = Interner::new();
        assert_eq!(i.handle("\u{301}"), None, "a lone combining acute");
        assert_eq!(i.handle("\u{200B}"), None, "a zero-width space");
        assert!(i.is_empty());
    }

    #[test]
    fn the_sentinels_name_no_text() {
        let i = Interner::new();
        assert_eq!(i.resolve(GraphemeId::EMPTY), None);
        assert_eq!(i.resolve(GraphemeId::CONTINUATION), None);
    }

    #[test]
    fn a_handle_from_one_table_is_meaningless_in_another() {
        // The invariant every surface in a layer stack holds by construction, stated as a test so
        // that `add_content_with`'s renumbering (ticket 10) has something to violate if it forgets.
        let mut ours = Interner::new();
        let mut theirs = Interner::new();
        theirs.handle("a\u{308}");
        let stranger = theirs.handle("e\u{301}").unwrap();
        let ours_same = ours.handle("e\u{301}").unwrap();
        assert_ne!(stranger, ours_same);
        assert_eq!(ours.resolve(stranger), None, "id 1 does not exist here yet");
    }
}
