//! **Every memo in this crate, as a value** — the rule with a population it can be false on.
//!
//! > A memo carries the theme in its key **iff its value is made of paints or glyphs**, and the key
//! > is the theme's own `Revision` and never an enumeration of the axes.
//!
//! The rule is one word longer than it was — *or glyphs* — because a declared
//! repertoire is a real input to the ten distinction bits, so a repertoire swap is a new revision
//! and a value made of clusters is as stale after `Ctrl+G` as a value made of paints is after `t`.
//!
//! # Why a list, and why it is joined to the freeze
//!
//! The criterion is *a gate enumerates them from `INVENTORY` rather
//! than from a grep*, and the two halves are not the same instrument. A grep answers **where the
//! word `Memo` appears**; the join answers **which of the twenty-nine rows of the freeze
//! keeps a value between frames**, which is the question the rule is about. [`MEMOS`] is that
//! reading and [`held_by`] is the join.
//!
//! **The grep is still here and it is the completeness check**, because a census nobody compares
//! against the source is a list that decays: `tests::the_census_names_every_memo_in_the_source`
//! walks `src/` **recursively** and asserts that every site it finds is a row here.
//!
//! # The finding: the rule is true here vacuously, and one scan could not see the crate at all
//!
//! **Not one shipped memo in this crate holds paints or glyphs.** The three are a sub-cell bit
//! grid, an axis domain and a wrap index — offsets, bits and floats — and every cluster and every
//! paint on every panel is derived from the theme the frame carries, on the frame that draws it.
//! The rule is therefore satisfied by there being nothing subject to it, which is
//! `Verdict::of`'s vacuity failure in the shape this map keeps meeting: *an equality between two
//! things that do not exist holds*.
//!
//! So the gate this ticket owes is not this census. It is [`crate::gallery::swap_on`]'s count over
//! the surface, and what makes **that** one non-vacuous is [`crate::gallery::Keying`] — the memo the
//! rule is about, built as an axis with a right arm and a wrong one, because no shipped memo could
//! play the part.
//!
//! **And `crate::order`'s own scan for the spelling was blind to two thirds of the crate.** Its
//! claim is *no memo in this crate's library half takes a bare `Revision`*, checked with one
//! non-recursive `read_dir` over `src/` — and `src/chart/raster.rs` builds two
//! `vitui_runtime::Memo`s. It is `crate::inventory`'s own recorded defect (*the walk was one
//! `read_dir` over `src/`, so a component in a subdirectory was never scanned*) arriving a second
//! time in a different file, and the repair is the recursion plus the claim restated to what is
//! true: a `Memo` whose key is **folded** carries as many inputs as the fold does, and
//! [`crate::order::Keyed`] is for the key that cannot be folded into eight bytes.

use vitui_runtime::Theme;

use crate::inventory::INVENTORY;

/// What a memo's value is made of, which is the whole of what decides whether the theme is owed a
/// place in its key.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MadeOf {
    /// Paints, glyphs, or both — the theme decides it, so the theme is in the key.
    Themed,
    /// Numbers, offsets or bits the theme has no say in.
    Neither,
}

/// How the memo is spelled, which is what the completeness scan can and cannot see.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Spelling {
    /// `vitui_runtime::data::Memo<T>`, whose `get` takes one `Revision` — so a compound key is a
    /// **fold** into eight bytes, which is the runtime's own documented door.
    Folded,
    /// [`crate::order::Keyed`], which holds the key as a value and records what it was built at.
    Keyed,
    /// Neither: a field and a comparison written out. The scan cannot find these and the census
    /// names them.
    HandRolled,
}

/// One memo, and everything the rule needs to be checked against it.
#[derive(Clone, Copy, Debug)]
pub struct Memoised {
    /// Where it is, as a path a reader can open.
    pub site: &'static str,
    /// The rows of [`INVENTORY`] that keep one. **Empty is legal and means a fixture**: this crate's
    /// screens hold memos of their own and they are as subject to the rule as a component's.
    pub holders: &'static [&'static str],
    /// What it holds, in words.
    pub value: &'static str,
    /// Whether the theme decides that value.
    pub made_of: MadeOf,
    /// What its key is.
    pub key: &'static str,
    /// Whether the theme's own `Revision` is in that key.
    pub theme_in_key: bool,
    /// How it is written.
    pub spelling: Spelling,
}

impl Memoised {
    /// **Whether it obeys the memo-key rule.** The rule is an `iff`, so a memo carrying the theme in the key
    /// of a value the theme has no say in is as much a row of [`owed`] as one that leaves it out —
    /// the first is a value rebuilt on every `t` for nothing.
    pub const fn holds(&self) -> bool {
        matches!(self.made_of, MadeOf::Themed) == self.theme_in_key
    }
}

/// **Every memo in this crate's library half.** The census, and the population the rule runs over.
///
/// Five rows, and the shape of the list is the finding: three are shipped and hold nothing the
/// theme decides, one is a screen's, and the one that *is* made of paints and glyphs had to be
/// built for the gate to have a subject.
pub const MEMOS: &[Memoised] = &[
    Memoised {
        site: "crates/vitui-components/src/chart/raster.rs — PlotState::raster",
        holders: &["chart", "plot", "sparkline"],
        value: "the sub-cell bit grid and the owner byte per cell",
        made_of: MadeOf::Neither,
        key: "a ten-part fold: the data revision, the rectangle, the construction, the sub-cell \
              geometry, the series count, the domain and the reach",
        theme_in_key: false,
        spelling: Spelling::Folded,
    },
    Memoised {
        site: "crates/vitui-components/src/chart/raster.rs — PlotState::range",
        holders: &["chart", "plot", "sparkline"],
        value: "the axis domain: two floats",
        made_of: MadeOf::Neither,
        key: "the data revision and the range policy",
        theme_in_key: false,
        spelling: Spelling::Folded,
    },
    Memoised {
        site: "crates/vitui-components/src/edit.rs — Text::index",
        holders: &["field", "form"],
        value: "the wrap index: byte offsets and row starts",
        made_of: MadeOf::Neither,
        key: "(revision, width) — `Key::Revision` is the defect arm and drops the width",
        theme_in_key: false,
        spelling: Spelling::HandRolled,
    },
    Memoised {
        site: "crates/vitui-components/src/preview.rs — Screen::highlighter",
        holders: &[],
        value: "how many times the document has been folded: one `usize`",
        made_of: MadeOf::Neither,
        key: "the pane's own revision",
        theme_in_key: false,
        spelling: Spelling::Keyed,
    },
    Memoised {
        site: "crates/vitui-components/src/gallery.rs — Panels::slots",
        holders: &[],
        value: "the verbs a tile drew: clusters, paints and where they went",
        made_of: MadeOf::Themed,
        key: "`Keying::Themed` folds `Theme::memo_key`; `Keying::DataAndTier` is the defect and \
              folds the colour depth instead",
        theme_in_key: true,
        spelling: Spelling::HandRolled,
    },
];

/// **The rows the rule is false on.** Empty, and [`crate::gallery::Keying::DataAndTier`] is what
/// keeps that from being a sentence nobody can test.
pub fn owed() -> Vec<&'static Memoised> {
    MEMOS.iter().filter(|m| !m.holds()).collect()
}

/// **The memos one row of the freeze keeps**, which is the join the enumeration has to
/// go through.
pub fn held_by(id: &str) -> Vec<&'static Memoised> {
    MEMOS.iter().filter(|m| m.holders.contains(&id)).collect()
}

/// The rows of the freeze that keep no memo at all. **Twenty-four of the twenty-nine**, which is the
/// census's own account of why the rule had nothing to bite on.
pub fn without() -> Vec<&'static str> {
    INVENTORY
        .iter()
        .filter(|c| held_by(c.id).is_empty())
        .map(|c| c.id)
        .collect()
}

/// **The rule's key, spelled once.** `Theme::memo_key` folds the theme's own `Revision` with the
/// data's, which is the runtime's documented door for a caller holding two revisions and a `Memo`
/// whose `get` takes one.
///
/// It is re-exported through [`crate::glyphs::glyph_memo_key`] as well, and deliberately: that
/// module is where the 598-character negative case lives (components 05), and this one is where the
/// population lives.
pub fn key(theme: &Theme, data: vitui_runtime::Revision) -> vitui_runtime::Revision {
    theme.memo_key(data)
}

#[cfg(test)]
mod tests {
    use super::{INVENTORY, MEMOS, MadeOf, Spelling, held_by, owed, without};

    /// **The memo-key rule over the census.** The rule is an `iff` and it is checked as one.
    #[test]
    fn every_memo_carries_the_theme_iff_its_value_is_made_of_paints_or_glyphs() {
        assert!(
            owed().is_empty(),
            "these memos disagree with ADR 0030: {:?}",
            owed().iter().map(|m| m.site).collect::<Vec<_>>()
        );

        // **And the population is not zero**, which is the half a vacuous `Met` would have skipped.
        // One row of five is made of paints and glyphs; the other four are not, and each of them
        // would be a defect the other way round — a value the theme has no say in, rebuilt on every
        // `t`.
        let themed = MEMOS.iter().filter(|m| m.made_of == MadeOf::Themed).count();
        assert_eq!((MEMOS.len(), themed), (5, 1));
        for m in MEMOS {
            assert_eq!(
                m.holds(),
                (m.made_of == MadeOf::Themed) == m.theme_in_key,
                "{}",
                m.site
            );
        }
    }

    /// **The one row the rule bites on is a fixture, and that is the finding.**
    ///
    /// Not one shipped memo in this crate holds a paint or a cluster: every one of them holds bits,
    /// offsets or floats, and the picture is derived from the theme in front of the frame. So the
    /// gate that is owed could not have been this census — it is
    /// [`crate::gallery::swap_on`]'s count over the surface, and
    /// [`crate::gallery::Keying::DataAndTier`] is the arm that makes it fail.
    #[test]
    fn no_shipped_memo_in_this_crate_holds_a_paint_or_a_cluster() {
        let themed: Vec<&str> = MEMOS
            .iter()
            .filter(|m| m.made_of == MadeOf::Themed)
            .map(|m| m.site)
            .collect();
        assert_eq!(
            themed,
            vec!["crates/vitui-components/src/gallery.rs — Panels::slots"],
            "a shipped memo made of paints or glyphs would move this list, and would be the first \
             thing on this map that ADR 0030's rule has ever been able to be false about"
        );
        assert_eq!(
            crate::gallery::Gallery::new(vitui_runtime::work::Worker::queueing())
                .panels()
                .keying(),
            crate::gallery::Keying::Derived,
            "and the one that is subject to it is off unless a gate turns it on: the gallery an \
             application builds keeps no memo over a panel at all"
        );
    }

    /// **The join goes through the freeze**, which is the criterion's own spelling of it.
    #[test]
    fn the_census_joins_to_the_freeze_in_both_directions() {
        // Every holder named here is a row of the freeze. A typo would otherwise be a memo nobody
        // can find from the component that keeps it.
        for m in MEMOS {
            for id in m.holders {
                assert!(
                    INVENTORY.iter().any(|c| c.id == *id),
                    "{} names {id}, which is not a row of the freeze",
                    m.site
                );
            }
        }

        // And the other direction: the rows that keep one, and the rows that keep none.
        let keeping: Vec<&str> = INVENTORY
            .iter()
            .map(|c| c.id)
            .filter(|id| !held_by(id).is_empty())
            .collect();
        assert_eq!(keeping, vec!["field", "chart", "plot", "sparkline", "form"]);
        assert_eq!(
            (keeping.len(), without().len()),
            (5, INVENTORY.len() - 5),
            "twenty-four rows of the freeze keep nothing between frames but their own state"
        );
    }

    /// **The completeness check, and it is the grep** — a census nobody compares against the source
    /// is a list that decays.
    ///
    /// It walks `src/` **recursively**, which is the whole of `crate::order`'s own defect: that
    /// scan's one non-recursive `read_dir` could not see `src/chart/raster.rs`, where two
    /// `vitui_runtime::Memo`s are built.
    #[test]
    fn the_census_names_every_memo_in_the_source() {
        let src = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        let mut found: Vec<String> = Vec::new();
        walk(&src, &mut |path, line| {
            // The two spellings a scan can see. A hand-rolled memo is a field and a comparison and
            // has no needle at all, which is why the census names those rather than deriving them.
            if line.contains("Memo::new()") || line.contains("Keyed::new()") {
                found.push(path.to_owned());
            }
        });
        found.sort_unstable();
        found.dedup();
        assert_eq!(
            found,
            vec!["chart/raster.rs", "order.rs", "preview.rs"],
            "a memo was built in a file the census does not name. `order.rs` is `Keyed`'s own \
             `Default`, which is the definition and not a use"
        );

        // Every file the scan found is a file the census names, `order.rs` excepted for the reason
        // above.
        for path in found.iter().filter(|p| *p != "order.rs") {
            assert!(
                MEMOS.iter().any(|m| m.site.contains(path.as_str())),
                "{path} builds a memo the census does not name"
            );
        }

        // The hand-rolled two, which no needle can find: they are named here so that deleting one
        // fails rather than shrinking the census quietly.
        for site in [
            "src/edit.rs — Text::index",
            "src/gallery.rs — Panels::slots",
        ] {
            assert!(
                MEMOS.iter().any(|m| m.site.ends_with(site)),
                "{site} left the census"
            );
        }

        // And the other direction, so a scan that has quietly stopped scanning is as loud as a
        // clean crate: the needle really does match the line it is looking for.
        assert!("            range: Memo::new(),".contains("Memo::new()"));
    }

    /// Every `.rs` file under `dir`, library half only, one line at a time.
    fn walk(dir: &std::path::Path, f: &mut impl FnMut(&str, &str)) {
        let root = std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        let Ok(entries) = std::fs::read_dir(dir) else {
            panic!("a readable source directory: {}", dir.display());
        };
        for entry in entries {
            let path = entry.expect("a directory entry reads").path();
            if path.is_dir() {
                walk(&path, f);
                continue;
            }
            if !path.extension().is_some_and(|x| x == "rs") {
                continue;
            }
            let source = std::fs::read_to_string(&path).expect("a readable source file");
            let library = source
                .split_once("\n#[cfg(test)]\n")
                .map_or(source.as_str(), |(head, _)| head);
            let rel = path
                .strip_prefix(&root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            for line in library.lines().map(str::trim) {
                if line.starts_with("//") || line.starts_with("///") {
                    continue;
                }
                f(&rel, line);
            }
        }
    }

    /// The spellings, as a census of their own: what the scan can see and what it cannot.
    #[test]
    fn two_of_the_five_are_invisible_to_any_needle() {
        let hand = MEMOS
            .iter()
            .filter(|m| m.spelling == Spelling::HandRolled)
            .count();
        assert_eq!(
            hand, 2,
            "a hand-rolled memo is a field and a comparison, and no scan finds one — which is why \
             the enumeration is a value and the grep is only the completeness check"
        );
    }
}
