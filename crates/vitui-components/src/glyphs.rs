//! Spec §16's catalogue as a value: **six families over twenty entries, ten distinctions, and the
//! nine-cell matrix as a count rather than as nine screenshots.**
//!
//! > A distinction survives the whole matrix iff it is carried on both axes. (ADR 0032)
//!
//! The runtime owns the mechanism — [`vitui_runtime::Glyph`], [`Theme::glyph`],
//! [`vitui_runtime::Distinction`], [`Theme::shows`] — and it owns the two counts that make an entry
//! a lookup rather than a branch: no spelling blank, none other than one cell. **This file owns the
//! demand set and the counts over it**, which is the half a component crate can be wrong about.
//!
//! # Why a family, and why the pairwise gate is the wrong gate
//!
//! §17 first wrote the collapse gate pairwise, and run literally it fires on `panel`, `table` and
//! everything else with a border — **the nine box-drawing entries are all `+` at ASCII on purpose**,
//! which is exactly the 36 of 190 pairs §16 measures collapsing there, `9 × 8 / 2`. A corner
//! collapsing onto a corner loses nothing.
//!
//! The defect the gate has to catch is C09's, and it is a collapse **across** families: the shadow
//! table spelled the ASCII ellipsis `>`, which is precisely an ASCII `ArrowRight`, so **468
//! truncated labels ended in the collapsed-node marker** — and `tree` draws both. So the gate is
//! [`within_component_collapses`]: *for one component, two glyphs from two different families must
//! never spell alike*. The ASCII ellipsis is `~`.
//!
//! # The axis is named here exactly nowhere
//!
//! Every function below takes a `&Theme` and asks it how it spells things. That is not a style
//! choice: **occurrences of the repertoire type's path in `vitui-components/src` == 0** is the count
//! that replaces the type `Paint` was able to be (§16), and it is checked by
//! `crate::gates::tests::no_component_here_names_a_glyph_set_and_none_has_a_private_missing_table`
//! and again by `crate::inventory::tests::no_component_source_names_the_repertoire`.
//!
//! **Naming the exception rather than loosening the gate** — [`crate::gates`]'s own refinement 3.
//! Something has to build three themes to sweep three rungs, and that something is
//! `crates/vitui-components/tests/glyph_matrix.rs` and
//! `crates/vitui-components/examples/glyph_numbers.rs`. Neither is a component, both are outside
//! `src/`, and the sweep asserts its own file list so that a third one is a deliberate edit.

use vitui_runtime::layout::text;
use vitui_runtime::theme::{Distinction, Glyph, Role, Theme};

use crate::{Component, INVENTORY};

/// The six families the twenty entries fall into.
///
/// A family is **the set inside which a collapse is legitimate**, and nothing else. It is not a
/// taxonomy of what the glyphs look like: [`GlyphFamily::Rule`] and [`GlyphFamily::Box`] are both
/// box-drawing characters and are two families, because `│` and `├` spell `|` and `+` at ASCII and
/// the difference is one a `tree` acts on.
///
/// **Spelled `GlyphFamily` and not `Family`**, because [`crate::Family`] is §18's fifteen component
/// families and is a different thing. *Two types with one name across a module boundary* is the
/// review finding `GlyphSet` itself carries one crate down.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum GlyphFamily {
    /// The four arrow ends. §9's steppers and §7's disclosure markers are **one family**, and
    /// entering them twice under two names would be a collapse rather than two entries.
    Arrow,
    /// The two rules, `│` and `─`.
    Rule,
    /// The four corners, the four tees and the cross. **Nine entries that are all `+` at ASCII**,
    /// which is 36 of the 190 glyph pairs and every collapse the ASCII rung has.
    Box,
    /// A list marker and a checkmark.
    Mark,
    /// A scrollbar's thumb and its track.
    Fill,
    /// The one cell a truncated label ends in.
    Cut,
}

impl GlyphFamily {
    /// Every family, which is what the collapse census iterates.
    pub const ALL: [GlyphFamily; 6] = [
        GlyphFamily::Arrow,
        GlyphFamily::Rule,
        GlyphFamily::Box,
        GlyphFamily::Mark,
        GlyphFamily::Fill,
        GlyphFamily::Cut,
    ];

    /// The word a report prints.
    pub fn word(self) -> &'static str {
        match self {
            GlyphFamily::Arrow => "arrow",
            GlyphFamily::Rule => "rule",
            GlyphFamily::Box => "box",
            GlyphFamily::Mark => "mark",
            GlyphFamily::Fill => "fill",
            GlyphFamily::Cut => "cut",
        }
    }
}

/// Which family an entry belongs to.
///
/// Exhaustive over [`Glyph::ALL`] on purpose and with no `_` arm: warnings are denied workspace-wide,
/// so the day the runtime grows a twenty-first entry **this stops compiling** rather than quietly
/// filing it under something. That is the join criterion 11 asks for, kept by the compiler.
pub const fn family(g: Glyph) -> GlyphFamily {
    match g {
        Glyph::ArrowUp | Glyph::ArrowDown | Glyph::ArrowLeft | Glyph::ArrowRight => {
            GlyphFamily::Arrow
        }
        Glyph::VLine | Glyph::HLine => GlyphFamily::Rule,
        Glyph::TopLeft
        | Glyph::TopRight
        | Glyph::BottomLeft
        | Glyph::BottomRight
        | Glyph::TeeTop
        | Glyph::TeeBottom
        | Glyph::TeeLeft
        | Glyph::TeeRight
        | Glyph::Cross => GlyphFamily::Box,
        Glyph::Bullet | Glyph::Tick => GlyphFamily::Mark,
        Glyph::Thumb | Glyph::Track => GlyphFamily::Fill,
        Glyph::Ellipsis => GlyphFamily::Cut,
    }
}

/// One cell of §16's repertoire × tier matrix, as four counts.
///
/// **A count, not a screenshot.** §16 is explicit that nine screens is not the instrument: what a
/// component acts on is the last field, and it is far smaller than the pair count — because most
/// role pairs are never asked to be told apart.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Census {
    /// Indistinguishable role pairs, of [`ROLE_PAIRS`].
    pub roles: usize,
    /// Indistinguishable glyph pairs, of [`GLYPH_PAIRS`]. **Independent of the tier**: a glyph is a
    /// cluster and a cluster does not quantise, which is why §16's glyph column repeats itself
    /// across each row.
    pub glyphs: usize,
    /// Indistinguishable **signal** pairs, of [`SIGNAL_PAIRS`]. A signal is `(Option<Glyph>, Role)`
    /// — what actually reaches a cell — and two collapse when both halves do.
    pub signals: usize,
    /// [`Distinction`]s the theme no longer shows, of ten.
    pub lost: usize,
}

/// Unordered pairs of the thirteen roles. **78.**
pub const ROLE_PAIRS: usize = Role::ALL.len() * (Role::ALL.len() - 1) / 2;

/// Unordered pairs of the twenty glyphs. **190.**
pub const GLYPH_PAIRS: usize = Glyph::ALL.len() * (Glyph::ALL.len() - 1) / 2;

/// Signals: twenty glyphs plus *no glyph*, against thirteen roles. **273.**
///
/// The `+ 1` is [`Option::None`], and it is a real signal half rather than an off-by-one: a cell
/// with a glyph and a cell without one are two different things, which is the whole of *absence is
/// not representable*.
pub const SIGNALS: usize = (Glyph::ALL.len() + 1) * Role::ALL.len();

/// Unordered pairs of signals. **37 128.**
pub const SIGNAL_PAIRS: usize = SIGNALS * (SIGNALS - 1) / 2;

/// Every glyph pair this theme spells the same way.
///
/// Includes the within-family ones, which is what makes the number 36 at ASCII. Use
/// [`cross_family_collapses`] for the gate; this is the census.
pub fn glyph_collapses(theme: &Theme) -> Vec<(Glyph, Glyph)> {
    let mut out = Vec::new();
    for (i, &a) in Glyph::ALL.iter().enumerate() {
        for &b in &Glyph::ALL[i + 1..] {
            if theme.glyph(a) == theme.glyph(b) {
                out.push((a, b));
            }
        }
    }
    out
}

/// Every glyph pair from **two different families** this theme spells the same way.
///
/// **This is the gate, and it must be empty at every rung.** The pairwise form fires on every
/// border; this one fires on C09's defect and on nothing else.
pub fn cross_family_collapses(theme: &Theme) -> Vec<(Glyph, Glyph)> {
    glyph_collapses(theme)
        .into_iter()
        .filter(|&(a, b)| family(a) != family(b))
        .collect()
}

/// Every **within-component** cross-family collapse, over the whole freeze.
///
/// The gate spec §16 states, joined to [`Component::glyphs`]: a collapse matters when one component
/// draws both halves of it. `tree` draws `Ellipsis` and `ArrowRight`, which is how the ASCII `>`
/// became visible; nothing draws two corners that are ever confusable, because the corners only ever
/// collapse onto each other.
pub fn within_component_collapses(theme: &Theme) -> Vec<(&'static str, Glyph, Glyph)> {
    let mut out = Vec::new();
    for c in INVENTORY {
        for (i, &a) in c.glyphs.iter().enumerate() {
            for &b in &c.glyphs[i + 1..] {
                if family(a) != family(b) && theme.glyph(a) == theme.glyph(b) {
                    out.push((c.id, a, b));
                }
            }
        }
    }
    out
}

/// Indistinguishable role pairs at the tier this theme was resolved for.
pub fn role_collapses(theme: &Theme) -> Vec<(Role, Role)> {
    let mut out = Vec::new();
    for (i, &a) in Role::ALL.iter().enumerate() {
        for &b in &Role::ALL[i + 1..] {
            if !theme.roles_differ_on_wire(a, b) {
                out.push((a, b));
            }
        }
    }
    out
}

/// Indistinguishable signal pairs — the number ADR 0032 reports as `21 → 1 677 of 37 128`.
///
/// # It is the product of the two censuses, and that is worth stating rather than discovering
///
/// Two signals collapse when their glyph halves spell alike **and** their roles land on one key, so
/// the equivalence classes on signals are the products of the two partitions and the count is
/// `Σ C(a·b, 2)` over them. That is why the ASCII column is so much larger than the top two: one
/// glyph class of nine multiplies every role class it meets, and `C(9·b, 2)` grows quadratically
/// where `C(b, 2)` does not.
pub fn signal_collapses(theme: &Theme) -> usize {
    // Glyph classes, with `None` — *no glyph at all* — as a class of its own.
    let mut sizes: Vec<usize> = vec![1];
    let mut seen: Vec<&'static str> = Vec::new();
    for g in Glyph::ALL {
        let s = theme.glyph(g);
        match seen.iter().position(|&t| t == s) {
            Some(at) => sizes[at + 1] += 1,
            None => {
                seen.push(s);
                sizes.push(1);
            }
        }
    }

    // Role classes, from the wire keys. Equality of keys is an equivalence, so a representative is
    // enough and no transitive closure is owed.
    let mut roles: Vec<Vec<Role>> = Vec::new();
    for r in Role::ALL {
        match roles
            .iter_mut()
            .find(|class| !theme.roles_differ_on_wire(class[0], r))
        {
            Some(class) => class.push(r),
            None => roles.push(vec![r]),
        }
    }

    let mut n = 0usize;
    for a in &sizes {
        for b in &roles {
            let size = a * b.len();
            n += size * (size - 1) / 2;
        }
    }
    n
}

/// The distinctions this theme no longer shows, of ten.
///
/// **The number a component acts on**, and §16's point is that it is far smaller than the pair
/// count — 2 of 10 against 13 of 78 — because most role pairs are never asked to be told apart.
pub fn distinctions_lost(theme: &Theme) -> Vec<Distinction> {
    Distinction::ALL
        .into_iter()
        .filter(|&d| !theme.shows(d))
        .collect()
}

/// One matrix cell, as a [`Census`].
pub fn census(theme: &Theme) -> Census {
    Census {
        roles: role_collapses(theme).len(),
        glyphs: glyph_collapses(theme).len(),
        signals: signal_collapses(theme),
        lost: distinctions_lost(theme).len(),
    }
}

/// Which distinctions a component's demand set puts it at the mercy of.
///
/// Criterion 11's join from the other side: a component depends on a [`Distinction`] when it draws
/// **both halves** of that distinction's carrier. `tree` draws `Ellipsis` and `ArrowRight`, so it
/// depends on [`Distinction::Truncation`] — which is C09's defect stated as a dependency rather than
/// as an anecdote.
///
/// The three uncarried distinctions — `Hover`, `Fade`, `Status` — never appear here, because they
/// have no glyph halves to draw. That is not an omission: a component depends on those through its
/// **roles**, and §17's freeze does not carry a role column.
pub fn distinctions_of(c: &Component) -> Vec<Distinction> {
    Distinction::ALL
        .into_iter()
        .filter(|d| match d.carried_by() {
            Some((a, b)) => c.glyphs.contains(&a) && c.glyphs.contains(&b),
            None => false,
        })
        .collect()
}

/// A label cut to `w` cells, as **the head and the one-cell marker** the caller writes after it.
///
/// Two slices and no allocation, because this is the shape a drawing verb wants: `text::fit` writes
/// the head, then the marker, then the remainder (§3). The marker is `""` when nothing was cut.
///
/// **The one-cell rule made callable**, which is the only form of it §16 says is enforceable: a
/// three-cell `...` where one cell was reserved writes three cells where the reserved one was, and
/// `writes`, `verbs` and `marked` are identical either way — so the defect has no signature at any
/// counter and only the surface disagrees.
pub fn elide<'a>(theme: &Theme, s: &'a str, w: u16) -> (&'a str, &'static str) {
    if text::width(s) <= w {
        return (s, "");
    }
    if w == 0 {
        return ("", "");
    }
    (text::truncate(s, w - 1), theme.glyph(Glyph::Ellipsis))
}

/// **An elided label written as a partition of the room it was given**, marker and all.
///
/// [`elide`] reserves the marker's cell and hands back two slices; writing them is where the mistake
/// is, and it is the same mistake every time — pad the head to the **whole** width and then write
/// the marker over the pad's last cell, and one cell of every *truncated* widget is written twice.
/// It is invisible on the screen, invisible at `writes`, `verbs` and `marked`, and visible only to
/// the pair. Components 26 found it in `crate::input::select` and components 32 found the same
/// drawing transcribed into `crate::files::file_picker`.
///
/// So the drawing is one function and both call it: the pad stops where the marker starts, and there
/// is one place for that to be wrong. Answers the columns written, which is `room` unless the room
/// was zero.
///
/// **A partition of `room` and nothing wider**, which is what makes it callable from a component that
/// has already spent columns on a chevron.
pub(crate) fn elided_row_into<I: crate::ink::Ink>(
    ink: &mut I,
    cx: &mut vitui_runtime::Ctx<'_, '_>,
    x: i32,
    y: i32,
    label: &str,
    room: u16,
    paint: vitui_runtime::Paint,
) -> u16 {
    let (shown, tail) = elide(cx.theme(), label, room);
    let marker = text::width(tail);
    let head = room.saturating_sub(marker);
    let mut written = ink.pad_to(cx, x, y, shown, head, paint);
    if marker > 0 {
        written += ink.text(cx, x + i32::from(head), y, tail, paint);
    }
    written
}

/// A memo key for a value **made of glyphs**, which is the word ADR 0032 adds to R20's rule.
///
/// > A memo carries the theme in its key iff its value is made of paints **or glyphs**.
///
/// The key is [`Theme::revision`] through [`Theme::memo_key`] and **never the axes**. Keyed
/// `(data, tier)` — the plausible fix — survives a palette swap and hits after a repertoire one, at
/// half the cost of the rebuild it owed and holding the previous rung's characters.
///
/// Both directions matter. A memo whose value does **not** move with the theme and carries it anyway
/// pays a full recomputation on every swap for a value that is bit-identical afterwards, which the
/// map priced at two and four times the whole frame budget with two and four such memos on the dense
/// screen. This function exists so that the rule is a call a reviewer can look for, not a habit.
pub fn glyph_memo_key(theme: &Theme, data: vitui_runtime::Revision) -> vitui_runtime::Revision {
    theme.memo_key(data)
}

/// A tree gutter as a memoised value: **598 glyph cells over 200 rows**, which is scene 21.
///
/// Two of the rows are roots and draw no indent guide, so the count is `198 × 3 + 2 × 2` rather than
/// `200 × 3`. That is the corpus §10 priced the repertoire-blind memo key against, and it is built
/// here rather than in the test so that the report and the gate measure the same thing.
pub fn gutter(theme: &Theme, rows: usize) -> Vec<String> {
    (0..rows)
        .map(|i| {
            let mut row = String::new();
            // A root draws no guide. Two of them, which is what makes the cell count 598 and not
            // 600 — and a gutter with no roots is not a tree.
            if i % 100 != 0 {
                row.push_str(theme.glyph(if i % 3 == 0 {
                    Glyph::TeeLeft
                } else {
                    Glyph::VLine
                }));
            }
            row.push_str(theme.glyph(if i % 2 == 0 {
                Glyph::ArrowRight
            } else {
                Glyph::ArrowDown
            }));
            row.push_str(theme.glyph(Glyph::Bullet));
            row
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A theme at the rung a component gets when nobody has declared one, resolved at the tier the
    /// freeze's numbers are quoted at.
    ///
    /// **`src/` may not name the axis** (§16), so this is the only rung reachable from here — and it
    /// is reached the way a component reaches one, through a theme somebody else declared. The
    /// three-rung sweep is `crates/vitui-components/tests/glyph_matrix.rs`.
    fn declared() -> Theme {
        Theme::default().resolve(Default::default())
    }

    /// **The join is total and it is by family**, which is criterion 11's shape.
    #[test]
    fn every_entry_has_a_family_and_the_families_partition_the_table() {
        let mut counted = 0usize;
        for f in GlyphFamily::ALL {
            let n = Glyph::ALL.iter().filter(|&&g| family(g) == f).count();
            assert!(n > 0, "{f:?} has no entries, so it is not a family");
            counted += n;
        }
        assert_eq!(counted, Glyph::ALL.len(), "an entry is in two families");
        assert_eq!(Glyph::ALL.len(), 20);
        assert_eq!(
            Glyph::ALL
                .iter()
                .filter(|&&g| family(g) == GlyphFamily::Box)
                .count(),
            9,
            "the nine that all spell `+` at ASCII"
        );
        // 36 = C(9, 2), and that identity is the whole reason the gate is cross-family: every
        // collapse the ASCII rung has is one of these.
        assert_eq!(9 * 8 / 2, 36);
        assert_eq!(GLYPH_PAIRS, 190);
    }

    /// **Every component's demand set is drawn from the table and names nothing twice.**
    ///
    /// The type makes the first half true; the second is the one that bites, because §9's steppers
    /// and §7's disclosure markers are one family and a row that entered `ArrowDown` twice under two
    /// readings would look like two demands.
    #[test]
    fn the_demand_column_is_filled_and_joined_against_the_table() {
        let mut demanded: Vec<Glyph> = Vec::new();
        for c in INVENTORY {
            let mut seen: Vec<Glyph> = Vec::new();
            for &g in c.glyphs {
                assert!(!seen.contains(&g), "`{}` demands {g:?} twice", c.id);
                seen.push(g);
                if !demanded.contains(&g) {
                    demanded.push(g);
                }
            }
        }
        // Every debt §16 records closed is a debt something now draws. An entry nobody asks for is
        // a table row written for a reader rather than for a caller.
        for g in Glyph::ALL {
            assert!(
                demanded.contains(&g),
                "{g:?} is in the table and no row of the freeze draws it"
            );
        }
        let drawing = INVENTORY.iter().filter(|c| !c.glyphs.is_empty()).count();
        assert_eq!(
            drawing, 21,
            "twenty-one of the twenty-nine rows draw at least one glyph. **It was twenty until \
             components ticket 35**, which added `form` and corrected three rows that could draw an \
             `Ellipsis` and did not declare one"
        );

        // And every distinction with a glyph carrier has at least one component that draws both of
        // its halves. A carrier nobody draws is a bit nobody reads, which is the mirror of an entry
        // nobody demands.
        for d in Distinction::ALL {
            if d.carried_by().is_none() {
                continue;
            }
            assert!(
                INVENTORY.iter().any(|c| distinctions_of(c).contains(&d)),
                "{d:?} is carried by a glyph pair no component draws both halves of"
            );
        }
    }

    /// **Within-component cross-family collapse == 0 at the declared rung**, and the three-rung
    /// sweep is in `tests/glyph_matrix.rs` because it has to name the axis.
    #[test]
    fn no_component_confuses_two_families() {
        let theme = declared();
        assert_eq!(within_component_collapses(&theme), Vec::new());
        assert_eq!(cross_family_collapses(&theme), Vec::new());
    }

    /// **The detector fires, on the exact defect it exists for.**
    ///
    /// A gate nobody has watched fail is not a gate — §21's three-for-three finding, and the shape
    /// [`crate::obligations`] is built around. The fixture is C09's: a component that draws both
    /// `Ellipsis` and `ArrowRight` under a table that spells them alike.
    ///
    /// The rule is applied to a **spelling function** rather than to a theme, because a theme cannot
    /// be given a wrong table — the runtime owns it, which is the point of the whole section.
    #[test]
    fn the_detector_finds_c09s_collapse_and_not_a_corner_on_a_corner() {
        fn collapses(
            spell: impl Fn(Glyph) -> &'static str,
            drawn: &[Glyph],
        ) -> Vec<(Glyph, Glyph)> {
            let mut out = Vec::new();
            for (i, &a) in drawn.iter().enumerate() {
                for &b in &drawn[i + 1..] {
                    if family(a) != family(b) && spell(a) == spell(b) {
                        out.push((a, b));
                    }
                }
            }
            out
        }

        let tree = [
            Glyph::VLine,
            Glyph::TeeLeft,
            Glyph::ArrowRight,
            Glyph::Ellipsis,
        ];

        // The shadow table: ASCII, with the ellipsis spelled `>`.
        let shadow = |g: Glyph| match g {
            Glyph::VLine => "|",
            Glyph::TeeLeft => "+",
            Glyph::ArrowRight | Glyph::Ellipsis => ">",
            _ => "?",
        };
        assert_eq!(
            collapses(shadow, &tree),
            vec![(Glyph::ArrowRight, Glyph::Ellipsis)],
            "the detector does not see the collapse 468 truncated labels were drawn through"
        );

        // The shipped table, corrected. `~` is not `>`.
        let shipped = |g: Glyph| match g {
            Glyph::VLine => "|",
            Glyph::TeeLeft => "+",
            Glyph::ArrowRight => ">",
            Glyph::Ellipsis => "~",
            _ => "?",
        };
        assert_eq!(collapses(shipped, &tree), Vec::new());

        // And the within-family collapse the pairwise gate would have fired on stays invisible: all
        // four corners are `+` at ASCII on purpose.
        assert_eq!(
            collapses(|_| "+", &[Glyph::TopLeft, Glyph::BottomRight]),
            Vec::new(),
            "a corner collapsing onto a corner loses nothing, and a gate that says otherwise is \
             the pairwise version §17 first wrote"
        );
    }

    /// **`tree` is the row the whole argument runs through**, and the freeze now says so.
    #[test]
    fn tree_draws_both_halves_of_the_truncation_distinction() {
        let tree = INVENTORY
            .iter()
            .find(|c| c.id == "tree")
            .expect("the freeze has a tree");
        assert!(
            distinctions_of(tree).contains(&Distinction::Truncation),
            "`tree` draws both halves of the carrier and the join does not report the dependency"
        );
        // It needs no entry of its own: the chevron pair *is* `ArrowDown`/`ArrowRight` and the
        // indent guides are `VLine`, `TeeLeft`, `BottomLeft` (§16).
        for g in [
            Glyph::ArrowDown,
            Glyph::ArrowRight,
            Glyph::VLine,
            Glyph::TeeLeft,
            Glyph::BottomLeft,
            Glyph::Ellipsis,
        ] {
            assert!(tree.glyphs.contains(&g), "`tree` no longer draws {g:?}");
        }
        // The disclosure carrier is the same arrow pair the scrollbar's steppers come from, which
        // is why entering the two mechanisms separately would have been a collapse.
        assert!(distinctions_of(tree).contains(&Distinction::Disclosure));
    }

    /// **The marker is one cell, and the three-cell form moves the surface without moving a
    /// counter.**
    ///
    /// §16's rule, and the reason it is enforceable only on the table: `writes`, `verbs` and
    /// `marked` are identical either way, so nothing but the rendered cells disagrees.
    #[test]
    fn the_elision_marker_is_one_cell_and_the_three_cell_form_moves_the_surface() {
        let theme = declared();
        assert_eq!(text::width(theme.glyph(Glyph::Ellipsis)), 1);

        // 78 rows, which is §16's own row count, each with a label too long for its column.
        const ROWS: usize = 78;
        const W: u16 = 24;

        let mut moved = 0usize;
        for i in 0..ROWS {
            let label = format!("a directory entry that does not fit, number {i}");
            let (head, marker) = elide(&theme, &label, W);
            assert_eq!(text::width(head) + text::width(marker), W);
            assert_eq!(text::width(marker), 1);

            // The three-cell form, written where one cell was reserved. It overruns by two.
            let overrun = text::width(head) + 3;
            // The reserved cell is rewritten and two more are taken: three cells a row.
            moved += 1 + usize::from(overrun - W);
        }
        assert_eq!(
            moved,
            ROWS * 3,
            "three cells a row: the reserved one, rewritten, and the two that overrun"
        );
        // §16 records **468 cells over 78 rows**, which is six a row and therefore two truncating
        // fields a row — a tree row's label and its trailing annotation. What is asserted is the
        // per-field rule, because the total belongs to the scene and the rule belongs to the table.
        assert_eq!(moved * 2, 468);
    }

    /// **`census` is arithmetic over the two partitions**, and the identity is worth pinning: the
    /// signal count is determined by them and is not an independent measurement.
    #[test]
    fn the_signal_count_is_the_product_of_the_two_partitions() {
        let theme = declared();
        let c = census(&theme);
        assert_eq!(ROLE_PAIRS, 78);
        assert_eq!(GLYPH_PAIRS, 190);
        assert_eq!(SIGNALS, 273);
        assert_eq!(SIGNAL_PAIRS, 37_128);

        // At the declared rung every glyph class is a singleton, so the signal count is exactly the
        // role count times the twenty-one glyph classes.
        assert_eq!(c.glyphs, 0);
        assert_eq!(c.signals, c.roles * (Glyph::ALL.len() + 1));
    }

    /// **A component branches on a bool and names neither axis**, which is what `shows` is for —
    /// and `shows(Hover)` **is** R10's `hover_distinct`.
    #[test]
    fn a_resolved_theme_answers_ten_bits_and_an_unresolved_one_answers_none() {
        assert_eq!(Distinction::ALL.len(), 10);
        assert_eq!(
            distinctions_lost(&Theme::default()).len(),
            10,
            "a theme nobody resolved must promise nothing, and seven of the ten are now carried by \
             a glyph pair that a repertoire alone would have been enough to set"
        );
        let theme = declared();
        assert_eq!(
            theme.hover_interest().wants_hover(),
            theme.shows(Distinction::Hover),
            "`shows(Hover)` is `hover_distinct`, generalised rather than duplicated"
        );
    }

    /// **The gutter is 598 glyph cells over 200 rows**, which is scene 21's content.
    #[test]
    fn the_memoised_gutter_is_five_hundred_and_ninety_eight_cells() {
        let theme = declared();
        let cells: usize = gutter(&theme, 200).iter().map(|r| r.chars().count()).sum();
        assert_eq!(cells, 598, "198 rows of three and two roots of two");
    }
}
