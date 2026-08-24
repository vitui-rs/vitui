//! Spec §21's scene list, **as a normative value rather than an appendix**.
//!
//! > **A scene is removed only by a ticket naming the property it can no longer distinguish.**
//!
//! Three of these exist because a defect survived every gate then in force by not being on any
//! screen anybody had built — scenes 4, 5 and 6, the scrolled collection, the collection shorter
//! than its viewport and the twenty wheel clicks. That is the argument for the list being a value:
//! a scene that is a row in a document can stop being run with nothing saying so, and these three
//! are the record of what that costs.
//!
//! # The count is twenty-seven, and the ticket that asked for it says twenty-six
//!
//! `.scratch/vitui-components-impl/issues/04` reads *all twenty-six of §21*. §21's table is
//! **twenty-seven rows**, counted twice off the file. The spec is the authority — `CLAUDE.md` says
//! so in as many words, and the map is closed — so twenty-seven ship here and the disagreement is
//! written down rather than resolved by dropping a row: **the freeze does not get rewritten when a
//! later ticket disagrees with it** (ADR 0033), and a scene deleted to make a count come out is
//! exactly the deletion the rule at the top of this module forbids.
//!
//! The same ticket says *including the three marked as owed*. **One row of §21 carries `(owed)`** —
//! scene 19, the two scroll areas with overlay bars, whose ×8.4 amplification was never measured.
//! The three are scenes 4, 5 and 6, which the ticket's own opening paragraph names correctly as the
//! three that exist because a defect survived every gate. Both facts are fields here
//! ([`Scene::owed`] and [`Scene::from_a_survived_defect`]), because the value can hold both and a
//! count cannot.
//!
//! # Every scene is `Unsubjected`, and that is the honest standing rather than a bleak one
//!
//! [`crate::gates::Standing::Unsubjected`] means *it could run and there is nothing to run it over*.
//! Not one of the twenty-nine components exists, so not one of these twenty-seven screens can be
//! stood up. Filing any of them as `Evaluated` would be
//! [`crate::obligations::Verdict::of`]'s vacuity accident arriving on the scene list.
//!
//! What **is** run today is the *shape* of five of them, over a synthetic fixture, by
//! [`crate::runner`] — that is [`Scene::rehearsed_by`], and it is deliberately not `standing`. A
//! rehearsal says *the runner catches this defect*; a standing says *this screen is on a terminal*.
//! Conflating them is how a register comes to report a gate that nothing runs.
//!
//! # `decided` is the column that stops a scene from being deleted
//!
//! Every row carries what it decided, in §21's own figures. The rule at the top is *a scene is
//! removed only by a ticket naming the property it can no longer distinguish*, and a reader who
//! cannot see what a scene distinguishes will conclude it distinguishes nothing.

use crate::gates::{Instrument, Standing};
use crate::{Axis, INVENTORY};

/// The screen a scene is played on, in §21's own terms.
///
/// **[`Size::Unstated`] is an arm rather than a default.** Eighteen of §21's twenty-seven rows state
/// no `w x h`, and inventing one would put a number in a normative list that no ticket wrote — which
/// is the failure mode ADR 0033 records for prose and this file inherits for numbers. The nine that
/// do are three at one size, two at two, two over a domain of pairs and two in cells with no
/// dimensions; `examples/scene_numbers.rs` prints the split.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Size {
    /// One terminal, `w x h`.
    Screen {
        /// Columns.
        w: u16,
        /// Rows.
        h: u16,
    },
    /// Two, and the scene **is** the difference between them. §13's overlap is green at 300x80 and
    /// red at 60x20.
    Two {
        /// The size it is green at.
        wide: (u16, u16),
        /// The size it is red at.
        narrow: (u16, u16),
    },
    /// A cell count with no dimensions. §21 states **53 280** for the assembled gallery and no
    /// `w x h` anywhere on the map.
    Cells {
        /// How many cells.
        cells: u32,
    },
    /// A domain swept exhaustively. There is no screen: the scene is the domain.
    Domain {
        /// How many pairs.
        pairs: u64,
    },
    /// §21 states no size for this row.
    Unstated,
}

/// What a scene stands up.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Content {
    /// A screen assembled from many components at once — the shape every one of the map's defects
    /// was invisible on the owning ticket's screen and visible on.
    Assembled {
        /// How many parts §21 counts on the screen — interactive regions for the dense screen,
        /// panels for the gallery, sections for the accordion. **The unit is §21's, per row**,
        /// because normalising it would be inventing a number for the rows that state the other
        /// one.
        parts: u16,
    },
    /// Rows in a viewport.
    Rows {
        /// How many.
        rows: u64,
    },
    /// Rows and declared columns.
    Grid {
        /// How many rows.
        rows: u64,
        /// How many columns.
        cols: u16,
    },
    /// A forest, flattened by an index.
    Forest {
        /// How many nodes.
        nodes: u64,
        /// How deep the deepest path is.
        depth: u32,
    },
    /// Editable text.
    Fields {
        /// How many fields.
        fields: u16,
        /// How many bytes the largest holds.
        bytes: u64,
    },
    /// Layers standing over a base pass.
    Layers {
        /// How many.
        layers: u8,
    },
    /// Series of points to be rastered.
    Series {
        /// How many series.
        series: u8,
        /// How many points each.
        points: u64,
    },
    /// A pair space, visited exhaustively.
    Sweep {
        /// How many pairs.
        pairs: u64,
    },
    /// A repertoire matrix — glyphs or colours against rungs.
    Matrix {
        /// How many cells of the matrix.
        cells: u32,
    },
    /// A picture filling its rectangle.
    Picture {
        /// How many colour depths it is drawn at.
        depths: u8,
    },
    /// Files in a directory.
    Files {
        /// How many.
        files: u32,
    },
}

/// What a repertoire or palette swap changes.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Swap {
    /// The glyph repertoire: ASCII, Unicode, Extended.
    Repertoire,
    /// The theme's palette.
    Theme,
    /// The colour depth the terminal answered with.
    Depth,
}

/// What a scene plays over its content.
///
/// **[`Gesture::Shrink`] and [`Gesture::Resize`] are two gestures and not one spelling of one.** §21
/// is explicit: *the surface after a shrink equals a freshly built one*, written against a terminal
/// resize, passes on all twelve panels because `Gallery::resize` allocates a new `Surface` and the
/// residue has nowhere to survive — **that spelling tests the resize path and not the defect**,
/// which is content shrinking inside a rectangle that does not move.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Gesture {
    /// The offset moves, by rows and by columns.
    Scroll {
        /// Rows.
        rows: i64,
        /// Columns.
        cols: i64,
    },
    /// The wheel turns.
    Wheel {
        /// How many clicks.
        clicks: u32,
    },
    /// **The content becomes smaller inside a rectangle that does not move.**
    Shrink {
        /// How many rows are left.
        to_rows: u64,
    },
    /// The rectangle changes. Beside [`Gesture::Shrink`], never instead of it.
    Resize {
        /// The new width.
        w: u16,
        /// The new height.
        h: u16,
    },
    /// The data volume changes and nothing else does.
    Volume {
        /// How many rows now.
        rows: u64,
    },
    /// Rows are folded away.
    Fold {
        /// How many.
        rows: u64,
    },
    /// Rows are folded back.
    Unfold {
        /// How many.
        rows: u64,
    },
    /// Rows are inserted above an anchor.
    Insert {
        /// How many rows above the anchored position.
        above: u64,
    },
    /// Bodies open over the base pass.
    Open {
        /// How many.
        bodies: u8,
    },
    /// Text arrives.
    Type {
        /// How many bytes.
        bytes: u64,
    },
    /// A chord is pressed.
    Press {
        /// Its spelling.
        chord: &'static str,
    },
    /// A domain is visited exhaustively.
    Sweep {
        /// How many pairs.
        pairs: u64,
    },
    /// The repertoire, the palette or the colour depth changes.
    Swap {
        /// Which.
        what: Swap,
    },
    /// Items are selected.
    Select {
        /// How many.
        n: u32,
    },
    /// Data arrives in batches.
    Deliver {
        /// How many batches.
        batches: u8,
    },
    /// The order changes under whatever is looking at it.
    Sort,
}

/// One scene of spec §21's normative list.
#[derive(Clone, Copy, Debug)]
pub struct Scene {
    /// Its row in §21's table, which is how everything else refers to it.
    pub number: u8,
    /// The scene, in §21's own words.
    pub name: &'static str,
    /// The screen it is played on.
    pub size: Size,
    /// What it stands up.
    pub content: Content,
    /// What it plays over that content. Empty for a scene that decides something about a still
    /// screen — five of the twenty-seven, and each is about what the *first* frame draws.
    pub gestures: &'static [Gesture],
    /// The property it decided, with §21's figures.
    pub decided: &'static str,
    /// The `(component, axis)` pairs it is evidence for. **O5's population, from the other side.**
    ///
    /// Claimed only where §21's own row names the component and the axis's mechanism. A scene that
    /// plausibly exercises an axis and does not say so is not evidence for it: O5 exists because
    /// *C11 can write a perfect gate and still not know which twenty-nine components to run it
    /// against*, and a generous join is how that gate stops being enumerable.
    pub covers: &'static [(&'static str, Axis)],
    /// **§21 marks this row `(owed)`.** The scene is normative and its number was never measured.
    ///
    /// An owed scene contributes nothing to [`axis_scenes_of`], because a scene nobody has run is
    /// not evidence that an axis has one.
    pub owed: bool,
    /// **It exists because a defect survived every gate then in force**, by not being on any screen
    /// anybody had built. Three of the twenty-seven.
    pub from_a_survived_defect: bool,
    /// Where it stands. **Every row is [`Standing::Unsubjected`] today** — see this module's header.
    pub standing: Standing,
    /// What drives this scene's **shape** today, over a synthetic fixture rather than over the
    /// component. Not `standing`, and the header says why.
    pub rehearsed_by: &'static [Instrument],
}

/// The runner's own file, which is where every rehearsal lives.
const RUNNER: &str = "crates/vitui-components/src/runner.rs";

/// Spec §21's scene list, row for row. **Twenty-seven, and §21's table is the authority.**
pub const SCENES: [Scene; 27] = [
    Scene {
        number: 1,
        name: "the dense screen, 300x80, 267-338 regions",
        size: Size::Screen { w: 300, h: 80 },
        content: Content::Assembled { parts: 338 },
        gestures: &[],
        decided: "the partition rule: 214.58 us / 6 662 damaged against 84.96 / 0",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 09",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 2,
        name: "the same screen drawn naive and correct",
        size: Size::Screen { w: 300, h: 80 },
        content: Content::Assembled { parts: 338 },
        gestures: &[],
        decided: "0 of 24 000 cells apart - what keeps the damage count honest",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 09",
        },
        rehearsed_by: &[Instrument::Unit {
            file: RUNNER,
            name: "the_same_screen_drawn_both_ways_is_equal_cell_for_cell",
        }],
    },
    Scene {
        number: 3,
        name: "four collections at 1k / 100k / 1M",
        size: Size::Unstated,
        content: Content::Rows { rows: 1_000_000 },
        gestures: &[
            Gesture::Volume { rows: 1_000 },
            Gesture::Volume { rows: 100_000 },
            Gesture::Volume { rows: 1_000_000 },
        ],
        decided: "one store, one `Mode`; 63.87 / 63.87 / 64.08 us",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 11",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 4,
        name: "a scrolled collection",
        size: Size::Unstated,
        content: Content::Rows { rows: 1_000_000 },
        gestures: &[Gesture::Scroll {
            rows: 1_000,
            cols: 0,
        }],
        decided: "the inverted scroll sign: 12.21 us / 3 058 writes against 62.96 / 20 418, and \
                  *faster*. Found three times independently",
        covers: &[("collection", Axis::Scrolled)],
        owed: false,
        from_a_survived_defect: true,
        standing: Standing::Unsubjected {
            inverted_by: "components 11",
        },
        rehearsed_by: &[Instrument::Unit {
            file: RUNNER,
            name: "the_runner_catches_an_inverted_scroll_sign",
        }],
    },
    Scene {
        number: 5,
        name: "a collection shorter than its viewport",
        size: Size::Unstated,
        content: Content::Rows { rows: 9 },
        gestures: &[Gesture::Shrink { to_rows: 9 }],
        decided: "the stale tail: 71 of 80 rows, the defective build 2.3x faster marking 226x less",
        covers: &[("collection", Axis::Shrunk)],
        owed: false,
        from_a_survived_defect: true,
        standing: Standing::Unsubjected {
            inverted_by: "components 11",
        },
        rehearsed_by: &[Instrument::Unit {
            file: RUNNER,
            name: "the_runner_catches_a_stale_tail_after_content_shrinks_inside_a_rectangle_that_\
                   does_not_move",
        }],
    },
    Scene {
        number: 6,
        name: "twenty wheel clicks over a collection",
        size: Size::Unstated,
        content: Content::Rows { rows: 1_000_000 },
        gestures: &[Gesture::Wheel { clicks: 20 }],
        decided: "the unconditional scroll-into-view: 0 against 16, in four resolved tickets' code",
        covers: &[("collection", Axis::Wheeled)],
        owed: false,
        from_a_survived_defect: true,
        standing: Standing::Unsubjected {
            inverted_by: "components 11",
        },
        rehearsed_by: &[Instrument::Unit {
            file: RUNNER,
            name: "the_runner_catches_twenty_wheel_clicks_that_move_nothing",
        }],
    },
    Scene {
        number: 7,
        name: "a twelve-column 1M-row table, pinned both edges, horizontal overflow",
        size: Size::Unstated,
        content: Content::Grid {
            rows: 1_000_000,
            cols: 12,
        },
        gestures: &[Gesture::Scroll { rows: 0, cols: 40 }],
        decided: "verbs as the currency: 913 -> 2 497; the band's 345 re-damaged cells",
        covers: &[("table", Axis::Scrolled), ("table", Axis::Narrow)],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 14",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 8,
        name: "a million-node forest at depth 59 999",
        size: Size::Unstated,
        content: Content::Forest {
            nodes: 1_000_000,
            depth: 59_999,
        },
        gestures: &[Gesture::Scroll {
            rows: 59_999,
            cols: 0,
        }],
        decided: "the flatten index; 640 verbs against 418 and 2 497",
        covers: &[("tree", Axis::Scrolled)],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 16",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 9,
        name: "a fold and an unfold over 349 524 rows",
        size: Size::Unstated,
        content: Content::Forest {
            nodes: 349_524,
            depth: 8,
        },
        gestures: &[
            Gesture::Fold { rows: 349_524 },
            Gesture::Unfold { rows: 349_524 },
        ],
        decided: "splice against permutation: 1 run / 0.04 us against 297 180 / 24 338",
        covers: &[("tree", Axis::Shrunk)],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 16",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 10,
        name: "a 200 000-line document with 4 167 folds and an insert above them",
        size: Size::Unstated,
        content: Content::Rows { rows: 200_000 },
        gestures: &[Gesture::Fold { rows: 4_167 }, Gesture::Insert { above: 1 }],
        decided: "the anchor: 4 166 of 4 167 wrong, reanchored for 1.04 us",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 16",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 11,
        name: "an accordion of twelve sections, 0 / 6 / 12 open",
        size: Size::Unstated,
        content: Content::Assembled { parts: 12 },
        gestures: &[
            Gesture::Open { bodies: 0 },
            Gesture::Open { bodies: 6 },
            Gesture::Open { bodies: 12 },
        ],
        decided: "closed content is not drawn: 478 hit entries against 70",
        covers: &[("collapsible", Axis::Shrunk)],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 21",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 12,
        name: "twenty fields and a 1 MB pasted textarea",
        size: Size::Unstated,
        content: Content::Fields {
            fields: 20,
            bytes: 1_048_576,
        },
        gestures: &[
            Gesture::Type { bytes: 1_048_576 },
            Gesture::Press { chord: "Left" },
        ],
        decided: "the caret pair and the index: 9 655x on one `Left`",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 23",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 13,
        name: "a 300 -> 120 resize with a wrap memo",
        size: Size::Two {
            wide: (300, 80),
            narrow: (120, 80),
        },
        content: Content::Fields {
            fields: 1,
            bytes: 1_048_576,
        },
        gestures: &[Gesture::Resize { w: 120, h: 80 }],
        decided: "the memo key: 625 rows drawn where 875 are needed",
        covers: &[("field", Axis::Narrow)],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 23",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 14,
        name: "a select, a menu with a submenu, a modal and a scrim",
        size: Size::Unstated,
        content: Content::Layers { layers: 5 },
        gestures: &[Gesture::Open { bodies: 5 }],
        decided: "the family, and the 138.04 us opening frame",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 25",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 15,
        name: "two million-point series at 1k / 100k / 1M, at 300x80 and 60x20",
        size: Size::Two {
            wide: (300, 80),
            narrow: (60, 20),
        },
        content: Content::Series {
            series: 2,
            points: 1_000_000,
        },
        gestures: &[
            Gesture::Volume { rows: 1_000 },
            Gesture::Volume { rows: 100_000 },
            Gesture::Volume { rows: 1_000_000 },
            Gesture::Resize { w: 60, h: 20 },
        ],
        decided: "the raster memo, 1 849 000x; and 60x20, where C08's overlap is red",
        covers: &[("chart", Axis::Narrow), ("plot", Axis::Narrow)],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 27",
        },
        rehearsed_by: &[Instrument::Unit {
            file: RUNNER,
            name: "a_construction_that_changes_with_the_width_is_green_wide_and_red_narrow",
        }],
    },
    Scene {
        number: 16,
        name: "175 712 axis viewport x dataset pairs",
        size: Size::Domain { pairs: 175_712 },
        content: Content::Sweep { pairs: 175_712 },
        gestures: &[Gesture::Sweep { pairs: 175_712 }],
        decided: "the axis loop oscillates on 464; from the whole domain, 0",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 27",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 17,
        name: "5 475 600 viewport x extent pairs",
        size: Size::Domain { pairs: 5_475_600 },
        content: Content::Sweep { pairs: 5_475_600 },
        gestures: &[Gesture::Sweep { pairs: 5_475_600 }],
        decided: "the bar fixpoint: 0 failures, <= 3 passes",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 27",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 18,
        name: "a 1M-row scroll area with one row in eight three cells tall",
        size: Size::Unstated,
        content: Content::Rows { rows: 1_000_000 },
        gestures: &[Gesture::Scroll {
            rows: 799_999,
            cols: 0,
        }],
        decided: "`sum h` is the extent: row 799 999 of 999 999, every count identical",
        covers: &[("scroll_area", Axis::Scrolled)],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 18",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 19,
        name: "two scroll areas far apart with overlay bars",
        size: Size::Unstated,
        content: Content::Assembled { parts: 2 },
        gestures: &[Gesture::Scroll { rows: 1, cols: 0 }],
        decided: "damage is one span per row: x8.4 amplification (owed)",
        covers: &[],
        owed: true,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 18",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 20,
        name: "the nine cells of the repertoire x tier matrix",
        size: Size::Unstated,
        content: Content::Matrix { cells: 9 },
        gestures: &[Gesture::Swap {
            what: Swap::Repertoire,
        }],
        decided: "21 -> 1 677 signal pairs; 2 distinctions of 10 lost",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 05",
        },
        // **Rehearsed as a count, and still `Unsubjected`.** Components ticket 05 measures the
        // repertoire axis in full and the colour axis not at all — `ColorDepth` is a barrier, which
        // register row 45 records — and a count over three rungs is not this screen on a terminal.
        rehearsed_by: &[
            Instrument::Unit {
                file: "crates/vitui-components/tests/glyph_matrix.rs",
                name: "the_glyph_column_is_zero_zero_and_thirty_six_of_a_hundred_and_ninety",
            },
            Instrument::Unit {
                file: "crates/vitui-components/tests/glyph_matrix.rs",
                name: "no_glyph_carried_distinction_is_lost_at_any_rung",
            },
        ],
    },
    Scene {
        number: 21,
        name: "a repertoire swap with a memoised glyph prefix",
        size: Size::Unstated,
        content: Content::Matrix { cells: 598 },
        gestures: &[Gesture::Swap {
            what: Swap::Repertoire,
        }],
        decided: "the theme's revision in the key: 598 wrong characters",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 05",
        },
        // The 598 is measured over the gutter this scene names — 200 rows, two of them roots — and
        // in both directions: the theme-keyed memo rebuilds, and the plausible `(data, tier)` key
        // **hits** and hands back the previous rung's characters.
        rehearsed_by: &[
            Instrument::Unit {
                file: "crates/vitui-components/tests/glyph_matrix.rs",
                name: "a_tier_keyed_memo_survives_a_palette_swap_and_is_wrong_after_a_repertoire_one",
            },
            Instrument::Unit {
                file: "crates/vitui-components/tests/glyph_matrix.rs",
                name: "a_glyph_memo_keyed_on_the_theme_rebuilds_after_a_repertoire_swap",
            },
        ],
    },
    Scene {
        number: 22,
        name: "a full-screen picture at three colour depths",
        size: Size::Screen { w: 300, h: 80 },
        content: Content::Picture { depths: 3 },
        gestures: &[Gesture::Swap { what: Swap::Depth }],
        decided: "24 000 customs; 50.7% of horizontal distinctions gone at C16",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 29",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 23,
        name: "twenty selections through a directory of photographs",
        size: Size::Unstated,
        content: Content::Files { files: 20 },
        gestures: &[Gesture::Select { n: 20 }],
        decided: "R18 §5's crossover: 1 picture / 536 KB against 20 / 10.7 MB",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 29",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 24,
        name: "a directory streamed in seven batches under a sort",
        size: Size::Unstated,
        content: Content::Files { files: 200 },
        gestures: &[Gesture::Deliver { batches: 7 }, Gesture::Sort],
        decided: "the dedupe key: wrong after 7 of 7 batches, with no user in it",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 31",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 25,
        name: "a re-sort under a preview pane, 200 files",
        size: Size::Unstated,
        content: Content::Files { files: 200 },
        gestures: &[Gesture::Sort],
        decided: "197 of 200 positions move; wrong on 100 of 100 frames",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 31",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 26,
        name: "the assembled gallery, twelve panels, 53 280 cells",
        size: Size::Cells { cells: 53_280 },
        content: Content::Assembled { parts: 12 },
        gestures: &[],
        decided: "9 956 cells nobody writes; the swap excess equal to it on five of six",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 39",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 27,
        name: "a theme swap over the gallery",
        size: Size::Cells { cells: 53_280 },
        content: Content::Assembled { parts: 12 },
        gestures: &[Gesture::Swap { what: Swap::Theme }],
        decided: "R20 §3 has no caller; six panels keep the old palette permanently",
        covers: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 41",
        },
        rehearsed_by: &[],
    },
];

/// **Every scene that stands `component` up.** Criterion 2's enumeration, from the scene's side.
pub fn scenes_for(component: &str) -> impl Iterator<Item = &'static Scene> + '_ {
    SCENES
        .iter()
        .filter(move |s| s.covers.iter().any(|(id, _)| *id == component))
}

/// **O5's evidence, derived from a scene list rather than written beside one.**
///
/// > **O5 is worth more than the other four obligations together.** (§17)
///
/// An owed scene contributes nothing. §21 marks a row `(owed)` when the scene is normative and its
/// number was never measured, and a scene nobody has run is not evidence that an axis has one — that
/// is the same distinction [`Scene::rehearsed_by`] draws against `standing`, arriving where it
/// changes a count.
pub fn axis_scenes_of(scenes: &[Scene]) -> Vec<(&'static str, Axis)> {
    let mut out = Vec::new();
    for scene in scenes {
        if scene.owed {
            continue;
        }
        for pair in scene.covers {
            if !out.contains(pair) {
                out.push(*pair);
            }
        }
    }
    out
}

/// [`axis_scenes_of`] over the whole list. This is what [`crate::obligations::AXIS_SCENES`] holds.
pub fn axis_scenes() -> Vec<(&'static str, Axis)> {
    axis_scenes_of(&SCENES)
}

/// **O5 by enumeration over [`INVENTORY`]**, as `(component, axis, scenes)` triples.
///
/// The population is *(component, axis) pairs where the flag is set* — thirty-four of them — and
/// **not** the scene list. Written the other way round it is a loop over scenes, which over an empty
/// scene list is vacuously true and is precisely the defect shape the four axes exist to catch.
/// [`crate::obligations::o5`] is the verdict; this is the same walk with the scene numbers attached,
/// so a reader can see *which* pairs have nothing.
pub fn coverage() -> Vec<(&'static str, Axis, Vec<u8>)> {
    let evidence = axis_scenes();
    let mut out = Vec::new();
    for component in INVENTORY {
        for axis in Axis::ALL {
            if !component.declares(axis) {
                continue;
            }
            let numbers: Vec<u8> = SCENES
                .iter()
                .filter(|s| {
                    !s.owed
                        && s.covers
                            .iter()
                            .any(|(id, a)| *id == component.id && *a == axis)
                })
                .map(|s| s.number)
                .collect();
            debug_assert_eq!(
                numbers.is_empty(),
                !evidence.contains(&(component.id, axis)),
                "the two walks disagree about {}/{}",
                component.id,
                axis.name()
            );
            out.push((component.id, axis, numbers));
        }
    }
    out
}

/// **Every scene on one line, in §21's column order**, so that a later ticket's numbers are
/// comparable with the spec's without re-deriving the format.
///
/// > `us / marked / writes / verbs / regions / stops / allocations`
///
/// # A scene with no subject prints the ticket that will build it, and never a row of zeros
///
/// Twenty-seven scenes and no components: nothing here can be played. A report that filled those
/// lines with `0.00 us  marked=0  writes=0` would be §21's first refinement arriving as a table — *a
/// threshold on the wrong side of the question is not a weak gate, it is a green one* — and it would
/// read as twenty-seven cheap frames. So an unplayed scene prints its standing.
///
/// A scene that **is** measured here is measured over a fixture and the line says so. See
/// [`Scene::rehearsed_by`]: a rehearsal is not a standing, and a report that dropped the word would
/// be claiming twenty-seven screens exist.
pub fn report(rehearsals: &[(u8, crate::runner::MetricRow)]) -> String {
    use std::fmt::Write as _;
    let mut out = crate::runner::metric_heading();
    out.push('\n');
    for scene in SCENES {
        let label = format!("{:>3}  {}", scene.number, scene.name);
        match rehearsals.iter().find(|(n, _)| *n == scene.number) {
            Some((_, row)) => {
                let _ = writeln!(
                    out,
                    "{label:<52}  {}  [rehearsed over a fixture]",
                    row.columns()
                );
            }
            None => {
                let Standing::Unsubjected { inverted_by } = scene.standing else {
                    let _ = writeln!(out, "{label:<52}  {}", scene.standing.word());
                    continue;
                };
                let _ = writeln!(
                    out,
                    "{label:<52}  not played: `{inverted_by}` builds the subject"
                );
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::path::PathBuf;

    fn read(relative: &str) -> String {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(relative);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// **Twenty-seven scenes, numbered 1..=27, once each — and §21's table is twenty-seven rows.**
    ///
    /// The count is the gate, and it is checked against the **spec file** rather than against a
    /// number typed here. A list normative by assertion is a list that drifts: §21 says *a scene is
    /// removed only by a ticket naming the property it can no longer distinguish*, and a row deleted
    /// from the spec with nothing deleted here reads as complete coverage of a shorter list.
    ///
    /// This is also where the ticket's *twenty-six* is settled. `issues/04` says twenty-six and §21
    /// has twenty-seven rows; the spec is the authority and the count below reads the spec.
    #[test]
    fn the_scene_list_is_the_twenty_seven_rows_of_the_specs_table() {
        assert_eq!(SCENES.len(), 27);
        let numbers: BTreeSet<u8> = SCENES.iter().map(|s| s.number).collect();
        assert_eq!(numbers, (1..=27).collect::<BTreeSet<u8>>());
        let names: BTreeSet<&str> = SCENES.iter().map(|s| s.name).collect();
        assert_eq!(names.len(), SCENES.len(), "two scenes share a name");

        // And the spec's own table, counted off the file.
        let spec = read(".scratch/vitui-components-architecture/spec.md");
        let rows = spec
            .lines()
            .skip_while(|l| !l.starts_with("| scene | what it decided |"))
            .skip(2)
            .take_while(|l| l.starts_with('|'))
            .count();
        assert_eq!(
            rows,
            SCENES.len(),
            "§21's table and this list disagree about how many scenes there are. That is a spec \
             change or a deletion, and either has to argue for itself here"
        );
    }

    /// **Every scene says what it decided, and says it in §21's figures.**
    ///
    /// A row with an empty `decided` is a row nobody can argue with, which makes it the first one
    /// deleted. A row with no number in it is the same thing one step less obviously.
    #[test]
    fn every_scene_says_what_it_decided_with_a_number_in_it() {
        for scene in SCENES {
            assert!(
                scene.decided.len() > 20,
                "scene #{} does not say what it decided",
                scene.number
            );
            assert!(
                scene.decided.chars().any(|c| c.is_ascii_digit()),
                "scene #{} decides something with no figure in it",
                scene.number
            );
        }
    }

    /// **Every scene is unsubjected, and the number is written down.**
    ///
    /// `obligations.rs`'s arrangement and `gates.rs`'s: a count makes the first scene to be stood up
    /// a deliberate edit here rather than a quiet change of colour. Not one of the twenty-nine
    /// components exists, so not one of the twenty-seven screens can be built — and every row names
    /// the scenes ticket that will build it.
    #[test]
    fn not_one_of_the_twenty_seven_scenes_has_anything_to_run_over() {
        let subjected: Vec<u8> = SCENES
            .iter()
            .filter(|s| !matches!(s.standing, Standing::Unsubjected { .. }))
            .map(|s| s.number)
            .collect();
        assert_eq!(
            subjected,
            Vec::<u8>::new(),
            "a scene has something to run over. That is the point of the backlog and it is also a \
             deliberate edit here, to this module's header and to `crate::gates::REGISTER`"
        );
        for scene in SCENES {
            let Standing::Unsubjected { inverted_by } = scene.standing else {
                unreachable!("checked above");
            };
            assert!(
                inverted_by.starts_with("components "),
                "scene #{} does not name the ticket that will stand it up",
                scene.number
            );
        }
    }

    /// **One row is owed and three exist because a defect survived every gate.**
    ///
    /// The ticket that asked for this list says *the three marked as owed*, and those are two
    /// different facts about two different sets. Both are asserted, so neither can be quietly
    /// merged into the other later.
    #[test]
    fn one_scene_is_owed_and_three_come_from_a_defect_that_survived_every_gate() {
        let owed: Vec<u8> = SCENES.iter().filter(|s| s.owed).map(|s| s.number).collect();
        assert_eq!(owed, vec![19], "§21 marks exactly one row `(owed)`");

        let survived: Vec<u8> = SCENES
            .iter()
            .filter(|s| s.from_a_survived_defect)
            .map(|s| s.number)
            .collect();
        assert_eq!(
            survived,
            vec![4, 5, 6],
            "the scrolled collection, the collection shorter than its viewport and the twenty \
             wheel clicks"
        );

        // And the spec still marks scene 19 and only scene 19.
        let spec = read(".scratch/vitui-components-architecture/spec.md");
        assert_eq!(
            spec.matches("(owed)").count(),
            1,
            "§21 has grown or lost an owed row"
        );
    }

    /// **An owed scene is not evidence that an axis has a scene.**
    ///
    /// Fired in both directions over a fixture, because scene 19 covers no pair today and a rule
    /// that never changes an answer is a rule nobody has watched work.
    #[test]
    fn an_owed_scene_contributes_no_evidence_to_o5() {
        let mut owed = SCENES[3];
        assert!(!owed.owed);
        assert_eq!(
            axis_scenes_of(&[owed]),
            vec![("collection", Axis::Scrolled)]
        );
        owed.owed = true;
        assert_eq!(
            axis_scenes_of(&[owed]),
            Vec::new(),
            "a scene nobody has run is not evidence that an axis has one"
        );
    }

    /// **O5 moves from 34 of 34 to 22 of 34, and it still fails loudly.**
    ///
    /// This ticket cannot turn O5 and does not pretend to. Twelve of the thirty-four
    /// `(component, axis)` pairs have a scene in §21's own list; the other twenty-two are the
    /// per-component scenes tickets' — `.scratch/vitui-components-impl/README.md` slices one before
    /// each component's drawing ticket, and *a scenes ticket is red on purpose until its component
    /// ticket lands*.
    ///
    /// The claim is narrow on purpose. A pair is claimed only where §21's row names the component
    /// **and** the axis's mechanism, because O5's whole argument is that *C11 can write a perfect
    /// gate and still not know which twenty-nine components to run it against* — and a generous
    /// join is how a gate stops being enumerable.
    #[test]
    fn twelve_of_the_thirty_four_axis_obligations_have_a_scene_and_twenty_two_do_not() {
        let evidence = axis_scenes();
        assert_eq!(evidence.len(), 12);
        assert_eq!(
            crate::obligations::AXIS_SCENES.to_vec(),
            evidence,
            "the constant O5 reads and the list it is derived from have drifted"
        );

        let coverage = coverage();
        assert_eq!(coverage.len(), 34, "the population §17 states");
        let bare = coverage.iter().filter(|(_, _, s)| s.is_empty()).count();
        assert_eq!(bare, 22);

        // Every claimed pair is an axis its component actually declares. A scene covering an axis
        // the freeze does not set is evidence for nothing and would inflate the count silently.
        for (id, axis) in &evidence {
            let component = INVENTORY
                .iter()
                .find(|c| c.id == *id)
                .unwrap_or_else(|| panic!("`{id}` is not in the freeze"));
            assert!(
                component.declares(*axis),
                "scene evidence claims `{id}` is {} and the freeze does not",
                axis.name()
            );
        }
    }

    /// **`scenes_for` enumerates from the component's side**, which is criterion 2's shape.
    #[test]
    fn scenes_for_a_component_lists_the_scenes_that_stand_it_up() {
        let collection: Vec<u8> = scenes_for("collection").map(|s| s.number).collect();
        assert_eq!(
            collection,
            vec![4, 5, 6],
            "the three that exist because a defect survived every gate"
        );
        assert_eq!(
            scenes_for("table").map(|s| s.number).collect::<Vec<_>>(),
            vec![7]
        );
        assert_eq!(
            scenes_for("checkbox").count(),
            0,
            "a component with no declared axis has no scene, and that is not a failure"
        );
    }

    /// **Seven scenes are rehearsed, and a rehearsal names a live test.**
    ///
    /// The same non-vacuity rule [`crate::gates`] is built around, and it is what stops
    /// `rehearsed_by` becoming a citation: a `#[test]` demoted to a helper, or left in place with
    /// `#[ignore]` on it, would keep a scene reading as driven while nothing drove it.
    #[test]
    fn every_rehearsal_names_a_live_test_that_exists() {
        let rehearsed: Vec<u8> = SCENES
            .iter()
            .filter(|s| !s.rehearsed_by.is_empty())
            .map(|s| s.number)
            .collect();
        assert_eq!(rehearsed, vec![2, 4, 5, 6, 15, 20, 21]);

        for scene in SCENES {
            for instrument in scene.rehearsed_by {
                let Instrument::Unit { file, name } = *instrument else {
                    panic!(
                        "scene #{} rehearses on something that is not a test. A rehearsal is a \
                         `#[test]` over a fixture, not a report and not a compile outcome",
                        scene.number
                    )
                };
                let source = read(file);
                assert!(
                    source.contains(&format!("fn {name}(")),
                    "scene #{}: `{file}` has no `fn {name}`",
                    scene.number
                );
                let attributed = source
                    .split(&format!("fn {name}("))
                    .next()
                    .is_some_and(|before| before.trim_end().ends_with("#[test]"));
                assert!(
                    attributed,
                    "scene #{}: `{name}` is not a live `#[test]`",
                    scene.number
                );
            }
        }
    }

    /// **A rehearsal is not a standing**, and this test is where that is written down.
    ///
    /// A rehearsal says *the runner catches this defect over a fixture*; a standing says *this
    /// screen is on a terminal*. Conflating them is how a register comes to report a gate that
    /// nothing runs, which is §21's three-for-three finding from the other direction.
    #[test]
    fn a_rehearsed_scene_is_still_unsubjected() {
        for scene in SCENES {
            if scene.rehearsed_by.is_empty() {
                continue;
            }
            assert!(
                matches!(scene.standing, Standing::Unsubjected { .. }),
                "scene #{} is rehearsed and claims to be run",
                scene.number
            );
        }
    }

    /// **The shrink gesture and the resize gesture are both present, and they are not the same.**
    ///
    /// §21 refuses to bank *the surface after a shrink equals a freshly built one* precisely because
    /// the version written against a terminal resize passes on all twelve panels. Scene 5 shrinks
    /// the content and scene 13 resizes the rectangle, and a list carrying only the second would
    /// have banked the gate.
    #[test]
    fn the_list_carries_a_shrink_and_a_resize_and_they_are_different_scenes() {
        let shrinks: Vec<u8> = SCENES
            .iter()
            .filter(|s| {
                s.gestures
                    .iter()
                    .any(|g| matches!(g, Gesture::Shrink { .. }))
            })
            .map(|s| s.number)
            .collect();
        let resizes: Vec<u8> = SCENES
            .iter()
            .filter(|s| {
                s.gestures
                    .iter()
                    .any(|g| matches!(g, Gesture::Resize { .. }))
            })
            .map(|s| s.number)
            .collect();
        assert_eq!(shrinks, vec![5]);
        assert_eq!(resizes, vec![13, 15]);
        assert!(
            shrinks.iter().all(|n| !resizes.contains(n)),
            "one scene spelling both is the conflation §21 warns about"
        );
    }

    /// **Every scene reports one line, and an unplayed one prints its ticket rather than zeros.**
    ///
    /// Criterion 7's shape, and §21's first refinement enforced on a *report*: twenty-seven rows of
    /// `0.00 us  marked=0  writes=0` would read as twenty-seven cheap frames, which is exactly the
    /// self-portrait the defective builds painted — three of the four axes were **faster** and two
    /// **marked less**.
    #[test]
    fn every_scene_reports_one_line_and_an_unplayed_one_says_so() {
        use crate::counters::Allocations;
        use crate::runner::{Fixture, play, rows_at_a_time};

        let run = play(
            rows_at_a_time,
            &[Fixture::lines(40, 10, 100).scrolled_to(3)],
        );
        let rehearsal = run.row("a scrolled collection", Allocations::over(1, 0));
        let printed = report(&[(4, rehearsal)]);

        assert_eq!(
            printed.lines().count(),
            SCENES.len() + 1,
            "one line a scene, plus the heading"
        );
        assert_eq!(printed.matches("not played: `components ").count(), 26);
        assert_eq!(printed.matches("[rehearsed over a fixture]").count(), 1);
        assert!(
            printed.contains("marked=unreachable"),
            "the one measured line names the counter this crate cannot read"
        );
        assert!(
            !printed.contains("marked=0"),
            "a counter that prints `0` when it means *I cannot see* is the green-and-wrong shape"
        );
        for scene in SCENES {
            assert!(
                printed.contains(scene.name),
                "scene #{} is missing from the report",
                scene.number
            );
        }
    }

    /// **Three scenes play no gesture, and each is about what the first frame draws.**
    ///
    /// The dense screen, the same screen drawn both ways, and the assembled gallery. A count rather
    /// than a rule, so that a gesture quietly disappearing from a row is a failing test rather than
    /// a scene that has stopped asking its question — and so that the twenty-four with one are
    /// visibly the majority, which is what the four hostile axes being **gestures** amounts to.
    #[test]
    fn three_scenes_decide_something_about_a_still_screen() {
        let still: Vec<u8> = SCENES
            .iter()
            .filter(|s| s.gestures.is_empty())
            .map(|s| s.number)
            .collect();
        assert_eq!(still, vec![1, 2, 26]);
        assert_eq!(SCENES.len() - still.len(), 24);
    }
}
