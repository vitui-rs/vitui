//! The catalogue as a value: **six families over twenty entries, nine distinctions, and the
//! nine-cell matrix as a count rather than as nine screenshots.**
//!
//! > A distinction survives the whole matrix iff it is carried on both axes.
//!
//! The runtime owns the mechanism — [`vitui_runtime::Glyph`], [`Theme::glyph`],
//! [`vitui_runtime::Distinction`], [`Theme::shows`] — and it owns the two counts that make an entry
//! a lookup rather than a branch: no spelling blank, none other than one cell. **This file owns the
//! demand set and the counts over it**, which is the half a component crate can be wrong about.
//!
//! # Why a family, and why the pairwise gate is the wrong gate
//!
//! The collapse gate was first written pairwise, and run literally it fires on `panel`, `table` and
//! everything else with a border — **the nine box-drawing entries are all `+` at ASCII on purpose**,
//! which is exactly the 36 of 190 pairs measured collapsing there, `9 × 8 / 2`. A corner
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
//! that replaces the type `Paint` was able to be, and it is checked by
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
/// **Spelled `GlyphFamily` and not `Family`**, because [`crate::Family`] is the fifteen component
/// families and is a different thing. *Two types with one name across a module boundary* is the
/// review finding `GlyphSet` itself carries one crate down.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum GlyphFamily {
    /// The four arrow ends. The steppers and the disclosure markers are **one family**, and
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

// ── the table joined to a drawer, which is the half a components decision found missing ──────

/// **How an entry of the twenty reaches a cell.**
///
/// Every draw in this crate goes through [`Theme::glyph`], so *is this entry drawn* is a
/// question about that call and about nothing else. Two shapes exist and the second is why this is
/// an enum rather than a needle.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reach {
    /// `…glyph(Glyph::X)`, the entry itself as the argument.
    ///
    /// **The needle is built from the entry and never written down here**, because this file holds
    /// the table: a needle spelled as a literal beside the row it belongs to is a scanner its own
    /// source satisfies, which is the trap this map has met more than once — and `Ellipsis` is the
    /// row that would have sprung it, since [`elide`] is one screen below.
    Literal,
    /// The argument is computed, and this is the fragment the arm that names the entry is spelled
    /// on.
    ///
    /// **One row uses it and it is `ArrowUp`.** `scrollbar`'s caps pick a pair by orientation, so
    /// `glyph(Glyph::ArrowUp)` is written nowhere in the workspace — a literal-only join would
    /// report the one arrow that *is* drawn as undrawn, and an entry wrongly in [`UNDRAWN`] is a
    /// worse lie than the one this whole join exists to end.
    Computed(&'static str),
}

/// **The call that puts an entry of the table on a cell**, as a row a test can open the file for.
#[derive(Clone, Copy, Debug)]
pub struct Drawer {
    /// The entry.
    pub glyph: Glyph,
    /// The file under `crates/vitui-components/src/` the call is in.
    pub file: &'static str,
    /// How the entry reaches the call.
    pub reach: Reach,
}

/// **Fifteen of the twenty, and the line each is drawn on.**
///
/// The freeze's `glyphs` column says which component *demands* an entry; it never said which line
/// *draws* one, and `the_glyph_table_is_twenty_entries_and_every_one_of_them_has_a_demander` joins a
/// declaration to a declaration — so an entry demanded by a row that draws nothing satisfied it.
/// Four such entries stood for a long time and were struck; five more
/// are real and are [`UNDRAWN`].
///
/// **One site per entry, not every site.** The question is *does anything draw this*, and a second
/// call adds a place for the table to go stale without adding an answer.
pub const DRAWERS: &[Drawer] = &[
    Drawer {
        glyph: Glyph::ArrowUp,
        file: "scroll.rs",
        reach: Reach::Computed("Orient::Vertical => (Glyph::ArrowUp, Glyph::ArrowDown),"),
    },
    // The tree's chevron pair, open and shut.
    Drawer {
        glyph: Glyph::ArrowDown,
        file: "collect.rs",
        reach: Reach::Literal,
    },
    Drawer {
        glyph: Glyph::ArrowRight,
        file: "collect.rs",
        reach: Reach::Literal,
    },
    // The pager's left cap. Its right cap is the line above.
    Drawer {
        glyph: Glyph::ArrowLeft,
        file: "collect.rs",
        reach: Reach::Literal,
    },
    // The block's border: two rules and four corners, all six in one function.
    Drawer {
        glyph: Glyph::HLine,
        file: "frame.rs",
        reach: Reach::Literal,
    },
    Drawer {
        glyph: Glyph::VLine,
        file: "frame.rs",
        reach: Reach::Literal,
    },
    Drawer {
        glyph: Glyph::TopLeft,
        file: "frame.rs",
        reach: Reach::Literal,
    },
    Drawer {
        glyph: Glyph::TopRight,
        file: "frame.rs",
        reach: Reach::Literal,
    },
    Drawer {
        glyph: Glyph::BottomLeft,
        file: "frame.rs",
        reach: Reach::Literal,
    },
    Drawer {
        glyph: Glyph::BottomRight,
        file: "frame.rs",
        reach: Reach::Literal,
    },
    // The legend's marker and the option list's tick.
    Drawer {
        glyph: Glyph::Bullet,
        file: "series.rs",
        reach: Reach::Literal,
    },
    Drawer {
        glyph: Glyph::Tick,
        file: "input.rs",
        reach: Reach::Literal,
    },
    // The bar's two halves.
    Drawer {
        glyph: Glyph::Thumb,
        file: "scroll.rs",
        reach: Reach::Literal,
    },
    Drawer {
        glyph: Glyph::Track,
        file: "scroll.rs",
        reach: Reach::Literal,
    },
    // The one cell a truncated label ends in, one screen below.
    Drawer {
        glyph: Glyph::Ellipsis,
        file: "glyphs.rs",
        reach: Reach::Literal,
    },
];

/// **An entry of the twenty that a row of the freeze demands and nothing in this crate draws.**
///
/// Held as a table rather than as prose for the reason — *a table holds a disagreement that
/// prose cannot* — and it is a disagreement rather than a defect: no picture is wrong, because an
/// entry nothing draws cannot collapse onto anything on a screen. What is wrong is a reader of the
/// freeze being told a component draws it.
///
/// **All five are one set and one missing construction: the box junctions.** A junction exists where
/// two rules meet, `panel` draws a border and no component in this crate draws two rules that meet —
/// `table` draws no rule at all, and its cells' separators are its caller's.
///
/// **Since architecture 25 they are `table`'s [`DELEGATED`] entries and not its demand column's**,
/// which is the answer to the question that used to be filed here: this crate not drawing them is a
/// fact about this crate, and the entries exist so a **caller** drawing a table's separators can get
/// them from the theme instead of hard-coding a `┼` that breaks at `Repertoire::Ascii`. `UNDRAWN` is
/// unchanged as a value — it is still *what no line of `vitui-components` draws* — and what changed
/// is that it no longer reads as a disagreement with the freeze.
#[derive(Clone, Copy, Debug)]
pub struct Undrawn {
    /// The entry.
    pub glyph: Glyph,
    /// Which row demands it, so the disagreement names a place and not a mood.
    pub demanded_by: &'static str,
}

/// The five entries of the table this crate does not draw. See [`Undrawn`].
pub const UNDRAWN: &[Undrawn] = &[
    Undrawn {
        glyph: Glyph::TeeTop,
        demanded_by: "table",
    },
    Undrawn {
        glyph: Glyph::TeeBottom,
        demanded_by: "table",
    },
    Undrawn {
        glyph: Glyph::TeeLeft,
        demanded_by: "table",
    },
    Undrawn {
        glyph: Glyph::TeeRight,
        demanded_by: "table",
    },
    Undrawn {
        glyph: Glyph::Cross,
        demanded_by: "table",
    },
];

/// **An entry a component's *caller* must be able to spell, and the component that hands it over.**
///
/// The answer, and it exists because the freeze's `glyphs` column turned out to
/// be answering two questions with one list. The column's verb is settled — it is **draws** —
/// and then five entries were left demanded by `table` and drawn by nothing, which read as the same
/// false claim one row over. It is not the same claim, and the difference is the table's:
///
/// > A table draws its **columns**. Its column separators are its caller's cells (the cell
/// > drawer owes every cell of its rectangle).
///
/// So a caller drawing a table's separators needs a rule down, a rule across where the header
/// divides, and a **junction** wherever the two meet — and it needs them from the *theme*, because
/// a caller that hard-codes `│` and `┼` is a caller whose table breaks at `Repertoire::Ascii`.
/// That is what [`Glyph`] is for. The entries are not undrawn debt; they are vocabulary handed over
/// with the job.
///
/// # Why this is not the reading architecture 20 refused to invent
///
/// 20 struck `tree`'s `VLine`, `TeeLeft` and `BottomLeft` rather than re-homing them, and refused to
/// invent an *allocation* meaning for the column silently. The two cases differ on one fact and it
/// is decisive: **a tree's indent guide cannot be drawn by anyone**, because a guide column at depth
/// *d* is a fact about *d* ancestors and all four routes to it are refused — so there is no caller to
/// delegate to and the entries were owed to nobody. A table's separators can be drawn, by exactly
/// the caller they are assigned to, in the cell drawer it already has.
///
/// Inventing it **deliberately** was the third option, and this is it, with the
/// column split rather than overloaded: `Component::glyphs` stays *what this component draws* and
/// this is *what its caller must be able to spell*. Two lists, two verbs, and
/// `tests::the_demand_column_is_filled_and_joined_against_the_table` reads their union — so no
/// entry of [`Glyph::ALL`] is unowned and none of them is claimed as a drawing.
///
/// **The collapse gate reads the union too**, and that is not bookkeeping: a caller's separators and
/// the table's own ellipsis land on **one screen**, which is exactly the question
/// *within-component cross-family collapse* asks.
#[derive(Clone, Copy, Debug)]
pub struct Delegated {
    /// The row that hands it over.
    pub component: &'static str,
    /// The entry its caller must be able to spell.
    pub glyph: Glyph,
}

/// The eleven entries [`crate::INVENTORY`] hands to a caller rather than drawing. See [`Delegated`].
///
/// All of them are `table`'s and they are the `rule` family, its four corners and its five
/// junctions — the whole vocabulary of a box drawn around and between a table's columns.
pub const DELEGATED: &[Delegated] = &[
    Delegated {
        component: "table",
        glyph: Glyph::HLine,
    },
    Delegated {
        component: "table",
        glyph: Glyph::VLine,
    },
    Delegated {
        component: "table",
        glyph: Glyph::TopLeft,
    },
    Delegated {
        component: "table",
        glyph: Glyph::TopRight,
    },
    Delegated {
        component: "table",
        glyph: Glyph::BottomLeft,
    },
    Delegated {
        component: "table",
        glyph: Glyph::BottomRight,
    },
    Delegated {
        component: "table",
        glyph: Glyph::TeeTop,
    },
    Delegated {
        component: "table",
        glyph: Glyph::TeeBottom,
    },
    Delegated {
        component: "table",
        glyph: Glyph::TeeLeft,
    },
    Delegated {
        component: "table",
        glyph: Glyph::TeeRight,
    },
    Delegated {
        component: "table",
        glyph: Glyph::Cross,
    },
];

/// **Everything one row of the freeze puts on one screen**: what it draws and what its caller draws
/// with the entries it hands over.
///
/// The population every gate about *one component's glyphs* reads, because a collapse is a question
/// about a screen and a caller's separators are on the component's screen.
pub fn on_screen(id: &str) -> Vec<Glyph> {
    let mut out: Vec<Glyph> = crate::INVENTORY
        .iter()
        .find(|c| c.id == id)
        .map(|c| c.glyphs.to_vec())
        .unwrap_or_default();
    for d in DELEGATED {
        if d.component == id && !out.contains(&d.glyph) {
            out.push(d.glyph);
        }
    }
    out
}

/// The needle a [`Drawer`] is looked for with, assembled rather than written down.
///
/// `glyph(` and not `theme.glyph(`, because the receiver is `theme` in one function and
/// `cx.theme()` in the next and the difference is not what is being asked about.
pub fn needle(d: &Drawer) -> String {
    match d.reach {
        Reach::Literal => format!("glyph(Glyph::{:?})", d.glyph),
        Reach::Computed(fragment) => fragment.to_string(),
    }
}

/// A file with its test modules removed, which is the region a [`Drawer`] has to be found in.
///
/// [`composed`](crate::composed)'s rule, for its reason: a test that spells a call is not the crate
/// making it. `Ellipsis` is why this is not optional — its drawer is [`elide`], and the test module
/// below spells `theme.glyph(Glyph::Ellipsis)` in the one-cell gate, so without the cut that row
/// would have been green over a test rather than over the component.
///
/// **Every block, not everything after the first.** `input.rs` homes five components and interleaves
/// five test modules with their code, the first of them at line 192 — cutting at the first marker
/// would have thrown away the whole file and reported `Tick` as undrawn, which is what the first
/// draft of this did.
///
/// **The anchor is the attribute, a newline and `mod `, and it has to be all three.** An attribute
/// alone appears in prose — including in this paragraph — so a cut on it deletes whatever follows
/// the first *sentence* that mentions it, which in this file was `shipped` itself. Joined to the
/// newline before it and the `mod ` after it, the anchor cannot occur inside a `///` line, which is
/// what makes the doc safe to write. A module is then closed by a brace in the **first column**,
/// which is what `rustfmt` guarantees for a top-level item, and
/// `tests::the_shipped_region_drops_a_test_module_and_keeps_what_follows_it` holds both halves.
pub fn shipped(source: &str) -> String {
    const OPEN: &str = "\n#[cfg(test)]\nmod ";
    let mut out = String::with_capacity(source.len());
    let mut rest = source;
    while let Some(at) = rest.find(OPEN) {
        out.push_str(&rest[..at]);
        let after = &rest[at + 1..];
        rest = match after.find("\n}\n") {
            Some(close) => &after[close + 3..],
            None => "",
        };
    }
    out.push_str(rest);
    out
}

/// Regions of shipped source that **name** an entry of the table without drawing one.
///
/// The [`UNDRAWN`] half asks *does any shipped line name this entry*, which is the loose question on
/// purpose: a drawer whose argument is computed writes the name on an arm rather than in the call,
/// and [`Reach::Computed`] exists because one of them does. That looseness needs somewhere to put
/// the places that legitimately name an entry and put nothing on a cell, and it is five, each with
/// its reason rather than a whole file waved through:
///
/// - `inventory.rs`, `INVENTORY` — the **demand** side. A freeze row is what the join is comparing
///   against, so reading it as evidence would be the declaration-to-declaration vacuity again.
/// - `glyphs.rs`, `family` — the classifier, exhaustive over `Glyph::ALL` by design.
/// - `glyphs.rs`, `DRAWERS` — this table's other half. [`Reach::Computed`]'s one fragment quotes an
///   arm verbatim.
/// - `glyphs.rs`, `UNDRAWN` — the five names themselves. Without this the scan reports every one of
///   them and the gate can never be green.
/// - `glyphs.rs`, `gutter` — a `Vec<String>` corpus for the memo-key measurement. It calls
///   [`Theme::glyph`] and **nothing reaches a cell**: the value is a string a test reads, which is
///   the distinction [`UNDRAWN`] is about and the reason this is an exception rather than a
///   [`Drawer`].
///
/// Each row is `(file, head, tail)` — the file below `src/`, and the declaration's marker split in
/// two for the reason the last paragraph gives.
///
/// The first draft excluded `inventory.rs` and `glyphs.rs` whole, and a review caught what that
/// bought: `gutter` names `TeeLeft` on a shipped line, so the one thing the exclusion hid was an
/// entry [`UNDRAWN`] claims nothing names. Five regions is that exception argued rather than
/// assumed, and a junction drawer landing anywhere else in either file now fails.
///
/// # The marker is two fragments, and the first draft of *this* was the trap it is named for
///
/// A whole marker written here is a line of `glyphs.rs` containing it, so `find` located the row
/// rather than the function and cut the table instead of the body — `gutter`'s `TeeLeft` survived
/// and the gate went red for the wrong reason. Splitting each marker at the space after `pub fn` or
/// `pub const` makes the joined needle occur nowhere but the declaration, which is
/// [`Reach::Literal`]'s remedy one level up, met again in the machinery that was written to apply
/// it.
pub const NAMES_WITHOUT_DRAWING: &[(&str, &str, &str)] = &[
    ("inventory.rs", "pub const ", "INVENTORY: &[Component] = &["),
    (
        "glyphs.rs",
        "pub const fn ",
        "family(g: Glyph) -> GlyphFamily {",
    ),
    ("glyphs.rs", "pub const ", "DRAWERS: &[Drawer] = &["),
    ("glyphs.rs", "pub const ", "UNDRAWN: &[Undrawn] = &["),
    // **Architecture 25's second list, and it is this trap's fifth instance.** `DELEGATED` names
    // every entry `table` hands to its caller, and the `UNDRAWN` scan's needle is the *bare*
    // `Glyph::…` path rather than a call — deliberately, because *is this drawn anywhere* is a
    // broader question than *is it called here*. So the table satisfied the scan the moment it
    // existed: a scanner looking for a literal contains that literal.
    ("glyphs.rs", "pub const ", "DELEGATED: &[Delegated] = &["),
    (
        "glyphs.rs",
        "pub fn ",
        "gutter(theme: &Theme, rows: usize) -> Vec<String> {",
    ),
];

/// `source` with every [`NAMES_WITHOUT_DRAWING`] region for `file` cut out of it.
///
/// A region runs from its marker to whichever of a closing brace and a closing bracket in the first
/// column comes first — a function and a table, and this crate's two shapes for *a thing closed in
/// the first column*. The test modules go first, which is [`shipped`].
pub fn drawing_region(file: &str, source: &str) -> String {
    let mut out = shipped(source);
    for (f, head, tail) in NAMES_WITHOUT_DRAWING {
        if *f != file {
            continue;
        }
        let marker = format!("{head}{tail}");
        let Some(at) = out.find(&marker) else {
            continue;
        };
        let after = &out[at..];
        let close = [after.find("\n}\n"), after.find("\n];\n")]
            .into_iter()
            .flatten()
            .min();
        let rest = match close {
            Some(c) => after[c..].to_string(),
            None => String::new(),
        };
        out = format!("{}{rest}", &out[..at]);
    }
    out
}

/// One cell of the repertoire × tier matrix, as four counts.
///
/// **A count, not a screenshot.** Nine screens is explicitly not the instrument: what a
/// component acts on is the last field, and it is far smaller than the pair count — because most
/// role pairs are never asked to be told apart.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Census {
    /// Indistinguishable role pairs, of [`ROLE_PAIRS`].
    pub roles: usize,
    /// Indistinguishable glyph pairs, of [`GLYPH_PAIRS`]. **Independent of the tier**: a glyph is a
    /// cluster and a cluster does not quantise, which is why the glyph column repeats itself
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
/// The gate, joined to [`Component::glyphs`]: a collapse matters when one component
/// draws both halves of it. `tree` draws `Ellipsis` and `ArrowRight`, which is how the ASCII `>`
/// became visible; nothing draws two corners that are ever confusable, because the corners only ever
/// collapse onto each other.
pub fn within_component_collapses(theme: &Theme) -> Vec<(&'static str, Glyph, Glyph)> {
    let mut out = Vec::new();
    for c in INVENTORY {
        // **[`on_screen`] and not `c.glyphs`**, since architecture 25 split the column: a collapse
        // is a question about *one screen*, and a table's caller draws its separators onto the
        // table's screen. Reading the drawn set alone would have taken this crate's loudest figure
        // from forty-two pairs to six by moving eleven entries to another list — which is a report
        // about bookkeeping and not about a screen.
        let on = on_screen(c.id);
        for (i, &a) in on.iter().enumerate() {
            for &b in &on[i + 1..] {
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

/// Indistinguishable signal pairs — the number reported as `21 → 1 677 of 37 128`.
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
/// **The number a component acts on**, and the point is that it is far smaller than the pair
/// count — 2 of 9 against 13 of 78 — because most role pairs are never asked to be told apart.
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
/// **roles**, and the freeze does not carry a role column.
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
/// the head, then the marker, then the remainder. The marker is `""` when nothing was cut.
///
/// **The one-cell rule made callable**, which is the only enforceable form of it: a
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

/// A memo key for a value **made of glyphs**, which is the word the degradation rule adds.
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

/// A tree gutter as a memoised value: **598 glyph cells over 200 rows**.
///
/// Two of the rows are roots and draw no indent guide, so the count is `198 × 3 + 2 × 2` rather than
/// `200 × 3`. That is the corpus the repertoire-blind memo key was priced against, and it is built
/// here rather than in the test so that the report and the gate measure the same thing.
///
/// **It is a corpus and not `tree`'s output.** `collect::tree` draws no guide at all — components
/// architecture 20, and [`UNDRAWN`] is the table that now says so — so this is *a value made of
/// glyphs* of the right size and shape for a memo-key measurement, and nothing on any screen this
/// crate draws. Which is why the memo rule it prices is stated over the value rather than over the
/// component.
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
    /// **`src/` may not name the axis**, so this is the only rung reachable from here — and it
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
    /// The type makes the first half true; the second is the one that bites, because the steppers
    /// and the disclosure markers are one family and a row that entered `ArrowDown` twice under two
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
        // Every debt the design records closed is a debt something now draws **or hands to a caller**. An
        // entry nobody asks for on either count is a table row written for a reader rather than for
        // a caller, which is the sentence this loop has always been.
        //
        // The union is architecture 25's: the column answers *draws* and [`DELEGATED`] answers
        // *what this component's caller must be able to spell*, and `table`'s eleven are the second
        // — the design gives a table's separators to the cell drawer, and a caller that hard-codes `│` and
        // `┼` has a table that breaks at `Repertoire::Ascii`.
        for d in DELEGATED {
            assert!(
                INVENTORY.iter().any(|c| c.id == d.component),
                "`{}` delegates {:?} and is not a row of the freeze",
                d.component,
                d.glyph
            );
            assert!(
                !INVENTORY
                    .iter()
                    .any(|c| c.id == d.component && c.glyphs.contains(&d.glyph)),
                "`{}` both draws and delegates {:?}, so one of the two lists is wrong about it",
                d.component,
                d.glyph
            );
            if !demanded.contains(&d.glyph) {
                demanded.push(d.glyph);
            }
        }
        for g in Glyph::ALL {
            assert!(
                demanded.contains(&g),
                "{g:?} is in the table and no row of the freeze draws it or hands it to a caller"
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
        //
        // **It is not vacuous any more, and that is the other half.** The
        // loop reads the freeze's `glyphs` column, which since architecture 20 means *draws* and
        // since 25 means it for every row: `table`'s eleven undrawn entries are in [`DELEGATED`] and
        // out of the column, so a distinction can no longer be carried by a declaration nobody
        // honours. `Distinction::Guide` was exactly that — `(VLine, TeeLeft)`, carried first by
        // `tree` and then, when 20 struck `tree`'s row, by `table`, neither of which drew either
        // half — and the runtime struck it, because its drawing is an indent guide and the design refuses
        // every route to one.
        //
        // **`DELEGATED` is deliberately not read here.** A caller *can* spell those entries, which
        // is why they stay in the table; whether a caller's screen keeps a distinction alive is a
        // question about a screen this crate does not draw, and answering it from a list would be
        // the vacuity again with one more indirection.
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
    /// A gate nobody has watched fail is not a gate — the three-for-three finding, and the shape
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
        // It needs no entry of its own: the chevron pair *is* `ArrowDown`/`ArrowRight`.
        for g in [Glyph::ArrowDown, Glyph::ArrowRight, Glyph::Ellipsis] {
            assert!(tree.glyphs.contains(&g), "`tree` no longer draws {g:?}");
        }
        // **And the three it declared for a long time and drew never** — a components
        // 20. The positive half above is what stops this reading green on a row that has lost
        // everything.
        for g in [Glyph::VLine, Glyph::TeeLeft, Glyph::BottomLeft] {
            assert!(
                !tree.glyphs.contains(&g),
                "`tree` demands {g:?} again. If the guides are now drawn this is the right \
                 failure and components architecture 20 is what to reopen — the strike was a \
                 decision, not a tidy-up"
            );
        }
        // The disclosure carrier is the same arrow pair the scrollbar's steppers come from, which
        // is why entering the two mechanisms separately would have been a collapse.
        assert!(distinctions_of(tree).contains(&Distinction::Disclosure));
    }

    /// Every `.rs` file under this crate's `src/`, so a scan for an absence cannot pass by looking
    /// in the wrong place.
    ///
    /// **Keyed by its path below `src/`, not by its basename.** This crate nests — `media/player.rs`
    /// — and the module convention here is `foo.rs` beside `foo/`, so two files sharing a basename
    /// is a matter of time; keying on one would make `read` pick whichever `read_dir` handed over
    /// first, which is unspecified and differs by filesystem.
    fn sources() -> Vec<(String, String)> {
        fn walk(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
            let entries =
                std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    walk(&path, out);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    let root =
                        std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
                    let name = path
                        .strip_prefix(&root)
                        .expect("a file under `src/`")
                        .to_string_lossy()
                        // `/` on every platform: the names this pairs against are spelled that
                        // way. See the same line in `crate::memos`.
                        .replace('\\', "/");
                    let body = std::fs::read_to_string(&path)
                        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                    out.push((name, body));
                }
            }
        }
        let mut out = Vec::new();
        walk(
            &std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")),
            &mut out,
        );
        assert!(
            out.len() > 40,
            "the src walk found only {} files",
            out.len()
        );
        out
    }

    /// **The twenty, partitioned into what this crate draws and what it does not — and the first
    /// half joined to a line rather than to another declaration.**
    ///
    /// The freeze's own tripwire asks *does some row demand this entry*, and a row demanding an
    /// entry it draws nothing of answers yes. That is how `tree` carried three indent guides and
    /// `select` a stepper `ArrowUp` for four tickets, and it is why nothing here could see the four
    /// tees and the `Cross` — which are still demanded, by `table`, and are [`UNDRAWN`].
    ///
    /// Both directions, because each half alone is vacuous: [`DRAWERS`] goes green on a table that
    /// has lost rows and [`UNDRAWN`] goes green on a crate that has lost files.
    #[test]
    fn every_entry_of_the_table_is_drawn_on_a_line_or_recorded_as_undrawn() {
        let files = sources();
        let read = |name: &str| -> String {
            files
                .iter()
                .find(|(n, _)| n == name)
                .unwrap_or_else(|| panic!("`src/{name}` is gone and a row of `DRAWERS` names it"))
                .1
                .clone()
        };

        // The partition is total and disjoint over the table, so a twenty-first entry lands in
        // neither and fails here rather than being filed under something.
        let mut seen = std::collections::BTreeSet::new();
        for d in DRAWERS {
            assert!(seen.insert(format!("{:?}", d.glyph)), "{:?} twice", d.glyph);
        }
        for u in UNDRAWN {
            assert!(seen.insert(format!("{:?}", u.glyph)), "{:?} twice", u.glyph);
        }
        assert_eq!(
            seen.len(),
            Glyph::ALL.len(),
            "an entry of §16's twenty is in neither `DRAWERS` nor `UNDRAWN`"
        );
        assert_eq!(
            (DRAWERS.len(), UNDRAWN.len()),
            (15, 5),
            "fifteen of the twenty are drawn and five are not, and the five are the box junctions"
        );

        // **The drawn half, joined to the line.** The needle is assembled, so this file's own table
        // does not satisfy it — which is exactly what `Ellipsis` would have done.
        for d in DRAWERS {
            let n = needle(d);
            assert!(
                crate::dense::declares(&shipped(&read(d.file)), &n),
                "`src/{}` no longer carries `{n}`, so {:?} is drawn somewhere else, drawn \
                 nowhere, or — for a `Computed` row — spelled differently after a reformat. The \
                 first two are what this row exists for; the third is a one-line correction here",
                d.file,
                d.glyph
            );
        }

        // **The undrawn half, over the whole crate and over the loose needle.** A bare name and not
        // a call, because a computed drawer writes the entry on an arm — `Reach::Computed` is there
        // because one of them does — so an entry that reaches a cell some other way still fails
        // here. What that looseness costs is paid in `NAMES_WITHOUT_DRAWING`, four named regions
        // with four reasons, rather than in two waved-through files: the first draft excepted this
        // file whole, and `gutter` names `TeeLeft` inside it on a shipped line.
        //
        // Both halves read the same region now. A test that spells `Glyph::Cross` while writing a
        // hostile fixture for this very gate would otherwise have failed it with *`Cross` is drawn
        // now*, which is the wrong diagnosis for the right observation.
        for u in UNDRAWN {
            let name = format!("Glyph::{:?}", u.glyph);
            let carriers: Vec<&str> = files
                .iter()
                .filter(|(n, body)| crate::dense::declares(&drawing_region(n, body), &name))
                .map(|(n, _)| n.as_str())
                .collect();
            assert_eq!(
                carriers,
                Vec::<&str>::new(),
                "{name} is drawn now, so it belongs in `DRAWERS` and `{}`'s row is no longer a \
                 disagreement",
                u.demanded_by
            );
            assert!(
                INVENTORY.iter().any(|c| c.id == u.demanded_by),
                "`{}` demands {name} and is not a row of the freeze",
                u.demanded_by
            );
            // **The union, since architecture 25.** A row asks for an entry by drawing it or by
            // handing it to its caller, and these five are the second: `table` delegates them, so
            // `on_screen` is where the demand now lives and `c.glyphs` no longer holds it.
            assert!(
                on_screen(u.demanded_by).contains(&u.glyph),
                "`{name}` is asked for by nothing — neither drawn nor delegated by `{}` — so this \
                 row records a disagreement that has ended",
                u.demanded_by
            );
        }

        // **The cut is watched too**, because a region that drops everything is a scan that
        // passes on nothing: `Ellipsis`'s drawer is in this file and so is a test that spells it.
        assert!(
            shipped(&read("glyphs.rs")).contains(&needle(&Drawer {
                glyph: Glyph::Ellipsis,
                file: "glyphs.rs",
                reach: Reach::Literal,
            })),
            "the shipped region of this file no longer holds `elide`'s own call"
        );

        // **Watched in both directions**, on the one predicate the scan uses rather than on a
        // second copy of it.
        assert!(crate::dense::declares(
            "let open = cx.theme().glyph(Glyph::ArrowDown);",
            "glyph(Glyph::ArrowDown)"
        ));
        assert!(!crate::dense::declares(
            "/// the chevron is `cx.theme().glyph(Glyph::ArrowDown)`",
            "glyph(Glyph::ArrowDown)"
        ));
    }

    /// This file's own source, for the one assertion that is about this file.
    fn read_here() -> String {
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/glyphs.rs"))
            .expect("this file")
    }

    /// **A test module is dropped and what follows it is kept**, which is [`shipped`]'s whole claim
    /// and the half its first draft got wrong.
    ///
    /// `input.rs` interleaves five test modules with the code of the five components it homes, the
    /// first of them at line 192 — a region that stopped at the first marker threw the file away and
    /// reported `Tick` as drawn by nothing. The closing brace is in the first column because
    /// `rustfmt` puts it there for a top-level item.
    #[test]
    fn the_shipped_region_drops_a_test_module_and_keeps_what_follows_it() {
        let file = "fn a() {}\n#[cfg(test)]\nmod tests {\n    fn inner() {}\n}\nfn b() {}\n";
        let kept = shipped(file);
        assert!(kept.contains("fn a()") && kept.contains("fn b()"));
        assert!(
            !kept.contains("fn inner()"),
            "the test module survived the cut"
        );
        // **A doc comment that mentions the attribute is not a module**, which is the half the
        // first draft got wrong: it anchored on the attribute alone, and this file's own prose
        // carries it, so `shipped` deleted itself out of the region before reaching `mod tests`.
        let prose = "/// removes its #[cfg(test)] modules.\nfn c() {}\nfn d() {}\n";
        assert_eq!(
            shipped(prose),
            prose,
            "a doc comment naming the attribute was read as a module"
        );
        // And the file this is actually about still holds the function the prose describes.
        assert!(
            shipped(&read_here()).contains("pub fn shipped(source: &str) -> String {"),
            "`shipped` has cut itself out of its own file's region again"
        );
        // And the real file it was written for: five modules gone, five components' code kept.
        let files = sources();
        let input = shipped(
            &files
                .iter()
                .find(|(n, _)| n == "input.rs")
                .expect("`src/input.rs`")
                .1,
        );
        assert!(input.contains("glyph(Glyph::Tick)"));
        assert!(
            !input.contains("mod slider_tests"),
            "a test module of `input.rs` survived the cut"
        );
    }

    /// **`tree`'s own body draws the chevrons and names no guide, and `select`'s names no
    /// stepper** — the decision, where it is falsifiable.
    ///
    /// [`DRAWERS`] is crate-wide and cannot see this: `VLine` is drawn, by `frame`, and `ArrowUp` is
    /// drawn, by `scrollbar`. What was false was that *these two components* draw them, and the
    /// smallest region that can answer is the component's own body.
    ///
    /// # The region is the function, not the file, and a review is why
    ///
    /// The first draft read whole files. `collect.rs` homes four components and `input.rs` five, and
    /// two of the other eight are the reason that could not stand: `table`'s freeze row still
    /// demands `VLine`, `TeeLeft` and `BottomLeft`, and **the decision — filed by this
    /// very ticket — is the question of whether `table` should draw its own rules**. The day it is
    /// answered yes, a file-wide scan fires an assertion whose message says `tree` has learned to
    /// draw indent guides and sends the reader to reopen architecture 20. A `slider` gaining an
    /// up-arrow would do the same to `select`'s half.
    ///
    /// So each half reads one function body, taken from its declaration to the brace that closes it
    /// in the first column. It is the same bounded form `crate::composed` uses and the same one
    /// `collect.rs` already takes for `tree_with` in its own scan.
    ///
    /// The positive half is not decoration. A scan for absences alone goes green the day the body is
    /// emptied, and these two bodies are where the chevrons are actually written.
    #[test]
    fn trees_body_draws_chevrons_and_no_guides_and_selects_has_no_steppers() {
        let files = sources();
        let read = |name: &str| -> String {
            files
                .iter()
                .find(|(n, _)| n == name)
                .unwrap_or_else(|| panic!("`src/{name}` is gone"))
                .1
                .clone()
        };
        /// One function body: from its declaration to the brace that closes it in the first column.
        ///
        /// The needle is assembled, for [`Reach::Literal`]'s reason — this test names both bodies
        /// and would otherwise find its own source.
        fn body(source: &str, head: &str, tail: &str) -> String {
            let marker = format!("{head}{tail}");
            let at = source
                .find(&marker)
                .unwrap_or_else(|| panic!("`{marker}` is not declared where this gate looks"));
            let after = &source[at..];
            let close = after
                .find("\n}\n")
                .unwrap_or_else(|| panic!("`{marker}` has no closing brace in the first column"));
            after[..close].to_string()
        }

        // `tree_with` is where both verbs are written — the indent run and the chevron cell — and
        // it is what `tree` and `tree_into` are, so a guide could not arrive anywhere else.
        let tree_body = body(&read("collect.rs"), "fn ", "tree_with<I, F, R>(");
        // `select_shaped` is the popup owner's body; its gutter is `overlay`'s reserved bar.
        let select_body = body(&read("input.rs"), "fn ", "select_shaped<'f, I: Ink>(");
        assert!(
            tree_body.len() > 400 && select_body.len() > 400,
            "a body this short is a marker that has moved, not a component that has shrunk"
        );

        for g in [Glyph::ArrowDown, Glyph::ArrowRight] {
            let n = format!("glyph(Glyph::{g:?})");
            assert!(
                crate::dense::declares(&tree_body, &n),
                "`tree_with` no longer draws {g:?}, so the negative half below is over nothing"
            );
        }
        for g in [Glyph::VLine, Glyph::TeeLeft, Glyph::BottomLeft] {
            let n = format!("Glyph::{g:?}");
            assert!(
                !crate::dense::declares(&tree_body, &n),
                "`tree_with` names {n}. If `tree` has learned to draw its indent guides then §7's \
                 record and its two-verb row have moved with it, and components architecture 20 is \
                 what says so — reopen it rather than putting the entry back"
            );
        }
        // The popup's own arrow is written in this body, and the stepper is not.
        assert!(
            crate::dense::declares(&select_body, "Glyph::ArrowDown"),
            "`select_shaped` no longer names the popup's chevron"
        );
        assert!(
            !crate::dense::declares(&select_body, "Glyph::ArrowUp"),
            "`select_shaped` names `Glyph::ArrowUp`. A popup's gutter is `scroll::bar_into` — \
             `Thumb` and `Track` — and stepper caps are `scroll::scrollbar`'s; if that has changed, \
             so has ADR 0029's reserved-bar decision"
        );
    }

    /// **The marker is one cell, and the three-cell form moves the surface without moving a
    /// counter.**
    ///
    /// The rule, and the reason it is enforceable only on the table: `writes`, `verbs` and
    /// `marked` are identical either way, so nothing but the rendered cells disagrees.
    #[test]
    fn the_elision_marker_is_one_cell_and_the_three_cell_form_moves_the_surface() {
        let theme = declared();
        assert_eq!(text::width(theme.glyph(Glyph::Ellipsis)), 1);

        // 78 rows, which is the row count, each with a label too long for its column.
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
        // the design records **468 cells over 78 rows**, which is six a row and therefore two truncating
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
    fn a_resolved_theme_answers_nine_bits_and_an_unresolved_one_answers_none() {
        assert_eq!(Distinction::ALL.len(), 9);
        assert_eq!(
            distinctions_lost(&Theme::default()).len(),
            9,
            "a theme nobody resolved must promise nothing, and six of the nine are now carried by \
             a glyph pair that a repertoire alone would have been enough to set"
        );
        let theme = declared();
        assert_eq!(
            theme.hover_interest().wants_hover(),
            theme.shows(Distinction::Hover),
            "`shows(Hover)` is `hover_distinct`, generalised rather than duplicated"
        );
    }

    /// **The gutter is 598 glyph cells over 200 rows**, which is the content.
    #[test]
    fn the_memoised_gutter_is_five_hundred_and_ninety_eight_cells() {
        let theme = declared();
        let cells: usize = gutter(&theme, 200).iter().map(|r| r.chars().count()).sum();
        assert_eq!(cells, 598, "198 rows of three and two roots of two");
    }
}
