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
//! # There are twenty-nine rows, and the two that are not §21's say so in a field
//!
//! §21's table is closed at twenty-seven and this list is longer, because **the backlog asks for
//! scenes §21 does not carry**: `issues/09`'s narrow axis is one, `issues/11`'s *narrow collection*
//! beside §21's four collection rows is the second, and `issues/23` will ask for the cluster corpus.
//! The arrangement is [`crate::gates::Row::on_spec_table`]'s, arriving here for its reason —
//! `tests::the_specs_table_is_twenty_seven_rows_and_this_list_carries_them_all` counts the spec's
//! rows against the rows that *claim* to be the spec's, so a twenty-eighth cannot arrive as a
//! one-line diff pretending to be §21's.
//!
//! The same ticket says *including the three marked as owed*. **One row of §21 carries `(owed)`** —
//! scene 19, the two scroll areas with overlay bars, whose ×8.4 amplification was never measured.
//! The three are scenes 4, 5 and 6, which the ticket's own opening paragraph names correctly as the
//! three that exist because a defect survived every gate. Both facts are fields here
//! ([`Scene::owed`] and [`Scene::from_a_survived_defect`]), because the value can hold both and a
//! count cannot.
//!
//! # Twenty-one scenes are `Unsubjected`, five are `Red` and three are `Evaluated`
//!
//! [`crate::gates::Standing::Unsubjected`] means *it could run and there is nothing to run it over*.
//! Twenty-four of the twenty-nine components do not exist, so most of these screens cannot be stood
//! up at all. Filing any of them as `Evaluated` would be
//! [`crate::obligations::Verdict::of`]'s vacuity accident arriving on the scene list.
//!
//! **Components ticket 11 moved four rows off `Unsubjected` and added a fifth**, and the direction
//! is *up*: `Unsubjected` says *nothing runs*, and scenes 3, 4, 5, 6 and 29 now run a great deal —
//! the inverted sign is refused by an equality at 75 of 80 rows, the stale tail at 71 of 80, the
//! missing ellipsis at one cell a row, and the three volumes agree on writes, regions and tab stops
//! to the unit. What they do not have is `collection`, which is [`crate::listing::standing`]'s
//! verdict and components ticket 12's job.
//!
//! **One of the five is not waiting for a subject and that is the point of the split.** Scene 6, the
//! twenty wheel clicks, is pinned to **components 20**: its failing set is the unconditional
//! `scroll_into_view` itself, which `CONTEXT.md` forbids and which four *resolved* tickets wrote
//! anyway. Ticket 12's own criterion says so — *every scene of ticket 11 is green except the wheel
//! gate, which stays pinned red for ticket 20* — and
//! `tests::twenty_one_scenes_have_nothing_to_run_over_five_are_red_and_three_are_stood_up` asserts
//! the pairing rather than the count alone, because a single `inverted_by` across the five would
//! erase the distinction while keeping the number right.
//!
//! **Scenes 1, 2 and 28 are `Evaluated`, and components ticket 10 is what moved them.** They were
//! `Red` for one ticket, which is the state worth reading this file for: ticket 09 built the dense
//! screen out of the three helpers that ship — `fit`, `block` and `press` — so the 338 regions, the
//! metric row, the equality against the naive twin at both sizes and all five of ADR 0026's
//! re-damage instances were measured rather than owed, and the only thing missing was the
//! **subject**. A screen made of a component's construction is not a screen made of the component,
//! so the three rows were pinned in their failing state with that as the exact failing set — §21's
//! own rule for a red gate, *it asserts its exact failing set, fires in both directions, and says
//! what to invert*.
//!
//! Ticket 10 declared `text`, `chip`, `button` and `panel`, and [`crate::dense`]'s screen is now
//! drawn **through** them: the same 338 regions, the same 24 000-of-24 000 partition, the same
//! equality at both sizes. Filing them `Unsubjected` at the time would have been the softer lie —
//! it says *nothing can run*, and a great deal ran; filing them `Evaluated` would have been the
//! harder one. **Which is why the move is a deliberate edit in three files**, here, in
//! `crate::dense`'s two inverted tests and in [`crate::gates::REGISTER`]'s row 61.
//!
//! One number moved with them and it is written down rather than absorbed:
//! [`crate::dense::CHIP_FILLED_FACE`] is **1 095** where ticket 09 measured 1 149, because a chip's
//! label wears its own face and the space inside the elided fourth value stopped differing from the
//! fill. See that constant.
//!
//! What is run over a fixture rather than over a component is still the *shape* of ten of these, by
//! [`crate::runner`] and [`crate::dense`] — that is [`Scene::rehearsed_by`], and it is deliberately
//! not `standing`. A rehearsal says *the instrument catches this defect*; a standing says *this
//! screen is on a terminal*. Conflating them is how a register comes to report a gate that nothing
//! runs.
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
/// is the failure mode ADR 0033 records for prose and this file inherits for numbers. The ten rows
/// that do state one — nine of §21's and components 09's — are two at one size, four at two, two
/// over a domain of pairs and two in cells with no dimensions; `examples/scene_numbers.rs` prints
/// the split.
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
    Rect {
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
    /// Whether it is a row of spec §21's twenty-seven-row table, or one a ticket on this backlog
    /// wrote beside it.
    ///
    /// **[`crate::gates::Row::on_spec_table`]'s arrangement, and it arrived here for its reason.**
    /// §21's table is closed and the count test reads it off the spec file; a scene a ticket adds
    /// has to say which side of the line it is on rather than moving that count. The backlog needs
    /// the field: `.scratch/vitui-components-impl/issues/11` asks for **five** collection scenes and
    /// §21 carries four of them, and `issues/09`'s narrow axis is not a row of §21 at all.
    pub on_spec_table: bool,
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
    /// **The components that are on the screen**, whether or not they are evidence for an axis.
    ///
    /// [`Scene::covers`] is O5's join and it is `(component, axis)`, so a component that declares no
    /// hostile axis can never appear in it — `panel` and `button` declare none, and a `(panel, …)`
    /// pair would fail
    /// `tests::fourteen_of_the_thirty_four_axis_obligations_have_a_scene_and_twenty_do_not`'s check
    /// that every claimed pair is an axis its component actually declares. That check is the reason
    /// this is a second field rather than a loosened first one: **a scene stands a component up; O5
    /// asks whether an axis has a scene, and the two are different questions about the same screen.**
    ///
    /// [`scenes_for`] reads both, because criterion 2's enumeration is *which scenes stand this
    /// component up*.
    pub stands: &'static [&'static str],
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

/// The dense screen's own file, which is where components ticket 09's three scenes are played.
const DENSE: &str = "crates/vitui-components/src/dense.rs";

/// The listing's own file, which is where components ticket 11's five scenes are played.
const LISTING: &str = "crates/vitui-components/src/listing.rs";

/// The scroll area's own file, which is where components ticket 18's four scenes are played.
const AREA: &str = "crates/vitui-components/src/area.rs";

/// **What pins components ticket 18's four scenes**, and it is the same three instruments on each:
/// the verdict over the three undeclared subjects, the sentence that says *waiting for its subject*
/// rather than *this is a defect*, and the scan that opens `scroll.rs` and reads what is declared
/// there. The scene's own measurement is the fourth, and it is different on every row.
const PINS_THE_AREA: [Instrument; 3] = [
    Instrument::Unit {
        file: AREA,
        name: "the_screens_are_waiting_for_three_subjects",
    },
    Instrument::Unit {
        file: AREA,
        name: "the_waiting_message_says_which_failure_it_is_and_stops_when_it_should",
    },
    Instrument::Unit {
        file: AREA,
        name: "the_scan_does_not_mistake_the_helper_for_the_component",
    },
];

/// The three components of F3 scrolling, which all four of ticket 18's scenes wait on.
const SCROLLING: &[&str] = &["scroll_area", "scrollbar", "sticky"];

/// The failing set every one of ticket 18's four scenes shares: **no subject**.
const OWED_ITS_SCROLL_AREA: &str = "`scroll_area`, `scrollbar` and `sticky` are undeclared, so what      stands on these screens is a stand-in row loop, a bar drawn from `scroll::thumb`'s own      arithmetic and two rectangles. Every number the screen itself can be asked is measured: 0      failures over 5 475 600 pairs in at most 3 passes, a row and a column lost permanently to      hysteresis, 99 flips in 99 frames, row 799 999 of 999 999 with no counter separating the two      builds, and 398 cells re-damaged against 0";

/// **What the dense screen is a screen of, and none of the four is declared yet.**
///
/// [`crate::dense::SUBJECTS`], reached through this alias so the three rows below cannot drift from
/// the list the failing set is computed over.
const SUBJECTS: &[&str] = &crate::dense::SUBJECTS;

/// **What the listing is a screen of, and it is not declared yet.**
///
/// [`crate::listing::SUBJECTS`], reached through this alias for [`SUBJECTS`]'s reason: five rows
/// below claim `collection` stands on their screen, and the failing set they are pinned in is
/// computed by opening the file the freeze homes `collection` in.
const COLLECTION: &[&str] = &crate::listing::SUBJECTS;

/// **The pair every one of components ticket 11's five scenes is pinned by**, written once because
/// it is one fact: `collection` is not declared, and the listing says which failure that is.
///
/// This is ticket 09's `STANDS_ON_ITS_COMPONENTS` one ticket later and on the other side of the
/// line. There it had already been inverted; here it is live, and the two halves are the two
/// directions criterion 7 asks for — [`crate::listing::standing`] is the verdict over the one
/// subject, and [`crate::listing::owed_message`] is the sentence that separates *waiting for its
/// subject* from *the code is wrong*.
const WAITING_FOR_COLLECTION: &[Instrument] = &[
    Instrument::Unit {
        file: LISTING,
        name: "the_listing_is_red_because_collection_is_not_declared",
    },
    Instrument::Unit {
        file: LISTING,
        name: "the_waiting_message_separates_unimplemented_from_wrong",
    },
];

/// The failing set every scene of components ticket 11 except the wheel is pinned in.
///
/// One sentence and not five, because it is one fact: the screen is measured and green and the
/// component is missing. The wheel scene's failing set is a **different** fact and carries its own —
/// see [`PINS_SCENE_6`].
const OWED_ITS_COLLECTION: &str = "`collection` is not declared in `crates/vitui-components/src/collect.rs`, so the five screens \
     stand on a stand-in row loop. Everything the screens themselves can be asked is measured and \
     green — 75 of 80 rows for the inverted sign, 71 of 80 for the stale tail, one cell a row for \
     the missing ellipsis, and identical writes, regions and tab stops at 1 000, 100 000 and \
     1 000 000. What is missing is the subject";

/// What pins scene 3, beyond the pair all five share.
const PINS_SCENE_3: &[Instrument] = &[
    Instrument::Unit {
        file: LISTING,
        name: "the_listing_writes_and_declares_the_same_at_a_thousand_rows_and_at_a_million",
    },
    Instrument::Unit {
        file: LISTING,
        name: "a_listing_that_iterates_its_whole_content_writes_the_same_and_declares_a_thousand_\
               times_more",
    },
    WAITING_FOR_COLLECTION[0],
    WAITING_FOR_COLLECTION[1],
];

/// What pins scene 4. See [`PINS_SCENE_3`].
const PINS_SCENE_4: &[Instrument] = &[
    Instrument::Unit {
        file: LISTING,
        name: "the_scrolled_listing_is_refused_by_the_equality_and_approved_by_every_counter",
    },
    WAITING_FOR_COLLECTION[0],
    WAITING_FOR_COLLECTION[1],
];

/// What pins scene 5. See [`PINS_SCENE_3`].
const PINS_SCENE_5: &[Instrument] = &[
    Instrument::Unit {
        file: LISTING,
        name: "the_stale_tail_is_seventy_one_of_eighty_rows_and_the_resize_spelling_misses_it",
    },
    WAITING_FOR_COLLECTION[0],
    WAITING_FOR_COLLECTION[1],
];

/// What pins scene 6, **the wheel gate**, and it is the one of the five that is not waiting for a
/// subject.
///
/// It is red because of the defect: `CONTEXT.md` forbids the unconditional `scroll_into_view` and
/// four *resolved* tickets wrote it anyway. Inverted by components 20 and not by components 12,
/// which is ticket 12's own criterion — *every scene of ticket 11 is green except the wheel gate,
/// which stays pinned red for ticket 20*.
const PINS_SCENE_6: &[Instrument] = &[
    Instrument::Unit {
        file: LISTING,
        name: "twenty_wheel_clicks_move_the_offset_twenty_and_an_unconditional_reveal_takes_it_back",
    },
    Instrument::Unit {
        file: "crates/vitui-components/tests/gates.rs",
        name: "a_frame_that_asks_for_no_reveal_moves_no_offset_and_one_that_asks_does",
    },
    Instrument::Barrier {
        file: "crates/vitui-runtime/src/line.rs",
        line: "name: \"Mouse\",",
    },
    WAITING_FOR_COLLECTION[0],
    WAITING_FOR_COLLECTION[1],
];

/// What pins scene 29, the narrow collection. See [`PINS_SCENE_3`].
const PINS_SCENE_29: &[Instrument] = &[
    Instrument::Unit {
        file: LISTING,
        name: "a_listing_row_narrows_through_fit_and_the_ellipsis_is_one_cell",
    },
    WAITING_FOR_COLLECTION[0],
    WAITING_FOR_COLLECTION[1],
];

/// **What stands scenes 1, 2 and 28 up**, written once because it is one fact: the four components
/// are declared, and the screen is drawn through them.
///
/// It was the *failing set* of three red rows for exactly one ticket, and ticket 09's criterion 7 is
/// why the pair below still has two halves. The distinction it draws is the subtle one — *a scene
/// that fails because it is unimplemented is indistinguishable from one that fails because the code
/// is wrong, unless the message distinguishes them* — and both directions are still live:
/// [`crate::dense::owed_message`] builds the waiting sentence over any declaration list, so the day
/// a component moves out of the file the freeze homes it in, the failure names the file rather than
/// reading as a defect in the screen.
#[cfg(test)]
const STANDS_ON_ITS_COMPONENTS: &[Instrument] = &[
    Instrument::Unit {
        file: DENSE,
        name: "the_dense_screen_stands_on_its_four_declared_components",
    },
    Instrument::Unit {
        file: DENSE,
        name: "the_waiting_message_still_says_which_failure_it_is",
    },
];

/// What stands scene 1 up, beyond the pair every one of the three shares.
///
/// **These three were `rehearsed_by` until components ticket 10** and they are `standing` now,
/// because what they run over changed underneath them without one character of the tests moving:
/// [`crate::dense::draw_into`] used to call `fit`, `block` and `press` directly and now calls
/// `text`, `chip`, `button` and `panel`. That is the whole distinction [`Scene::rehearsed_by`]
/// draws — *over a fixture or a stand-in* against *over the thing itself* — and it is why the field
/// is separate from `standing` rather than a flag on it.
const STANDS_SCENE_1: &[Instrument] = &[
    Instrument::Unit {
        file: DENSE,
        name: "the_dense_screen_stands_three_hundred_and_thirty_eight_regions_at_three_hundred_by_\
               eighty",
    },
    Instrument::Unit {
        file: DENSE,
        name: "the_dense_screen_declares_no_colliding_ids",
    },
    Instrument::Unit {
        file: DENSE,
        name: "the_dense_screen_reports_the_metric_row_and_never_prints_marked_zero",
    },
    Instrument::Unit {
        file: DENSE,
        name: "all_five_re_damage_instances_are_measured_on_this_screen",
    },
    Instrument::Unit {
        file: DENSE,
        name: "clearing_once_re_damages_nothing_and_clearing_every_frame_costs_nine_thousand_cells",
    },
    Instrument::Unit {
        file: DENSE,
        name: "the_dense_screen_stands_on_its_four_declared_components",
    },
    Instrument::Unit {
        file: DENSE,
        name: "the_waiting_message_still_says_which_failure_it_is",
    },
];

/// What stands scene 2 up. See [`STANDS_SCENE_1`].
const STANDS_SCENE_2: &[Instrument] = &[
    Instrument::Unit {
        file: DENSE,
        name: "the_same_screen_drawn_naive_and_correct_is_zero_cells_apart_at_both_sizes",
    },
    Instrument::Unit {
        file: DENSE,
        name: "the_naive_twin_is_declared_in_this_file_and_named_by_path",
    },
    Instrument::Unit {
        file: DENSE,
        name: "the_correct_screen_is_a_partition_of_twenty_four_thousand_cells",
    },
    Instrument::Unit {
        file: DENSE,
        name: "the_dense_screen_stands_on_its_four_declared_components",
    },
    Instrument::Unit {
        file: DENSE,
        name: "the_waiting_message_still_says_which_failure_it_is",
    },
];

/// What stands scene 28 up. See [`STANDS_SCENE_1`].
const STANDS_SCENE_28: &[Instrument] = &[
    Instrument::Unit {
        file: DENSE,
        name: "a_label_that_does_not_narrow_is_green_at_three_hundred_and_red_at_a_hundred_and_\
               twenty",
    },
    Instrument::Unit {
        file: DENSE,
        name: "the_narrow_screen_shrinks_its_content_inside_a_rectangle_that_does_not_move",
    },
    Instrument::Unit {
        file: DENSE,
        name: "at_a_hundred_and_twenty_columns_the_label_column_truncates_and_the_ellipsis_is_one_\
               cell",
    },
    Instrument::Unit {
        file: "crates/vitui-components/src/text.rs",
        name: "chip_writes_a_partition_of_its_rectangle_and_narrows_into_it",
    },
    Instrument::Unit {
        file: DENSE,
        name: "the_dense_screen_stands_on_its_four_declared_components",
    },
    Instrument::Unit {
        file: DENSE,
        name: "the_waiting_message_still_says_which_failure_it_is",
    },
];

/// Spec §21's scene list, row for row, and this backlog's scenes beside it. **Twenty-eight, of which
/// twenty-seven are §21's table and §21's table is the authority.**
pub const SCENES: [Scene; 30] = [
    Scene {
        number: 1,
        on_spec_table: true,
        name: "the dense screen, 300x80, 267-338 regions",
        size: Size::Screen { w: 300, h: 80 },
        content: Content::Assembled { parts: 338 },
        gestures: &[],
        decided: "the partition rule: 214.58 us / 6 662 damaged against 84.96 / 0",
        covers: &[],
        stands: SUBJECTS,
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Evaluated { by: STANDS_SCENE_1 },
        // **Empty, and it was three entries until components ticket 10.** Everything that ran over
        // a stand-in now runs over the components, so it is a standing and not a rehearsal.
        rehearsed_by: &[],
    },
    Scene {
        number: 2,
        on_spec_table: true,
        name: "the same screen drawn naive and correct",
        // **§2 states the two sizes in the same sentence as the number** — *0 of 24 000 cells
        // differ, at 300x80 and at 120x40* — and components ticket 09 runs it at both. It is a
        // `Two` and not a `Screen` for the reason the arm exists: the scene **is** the pair.
        size: Size::Two {
            wide: (300, 80),
            narrow: (120, 40),
        },
        content: Content::Assembled { parts: 338 },
        gestures: &[],
        decided: "0 of 24 000 cells apart - what keeps the damage count honest",
        covers: &[],
        stands: SUBJECTS,
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Evaluated { by: STANDS_SCENE_2 },
        // **One entry left, and it is a real rehearsal.** `runner`'s own equality runs over a
        // synthetic fixture and not over this screen, so it stays on the side of the line that says
        // *the instrument catches this defect* rather than *this screen is drawn*.
        rehearsed_by: &[Instrument::Unit {
            file: RUNNER,
            name: "the_same_screen_drawn_both_ways_is_equal_cell_for_cell",
        }],
    },
    Scene {
        number: 3,
        on_spec_table: true,
        name: "four collections at 1k / 100k / 1M",
        size: Size::Unstated,
        content: Content::Rows { rows: 1_000_000 },
        gestures: &[
            Gesture::Volume { rows: 1_000 },
            Gesture::Volume { rows: 100_000 },
            Gesture::Volume { rows: 1_000_000 },
        ],
        decided: "one store, one `Mode`; 63.87 / 63.87 / 64.08 us. Identical writes (3 200), \
                  regions (81) and tab stops (1) at all three volumes",
        covers: &[],
        stands: COLLECTION,
        owed: false,
        from_a_survived_defect: false,
        // **The one of the five that is not a hostile axis**, and the one that found why criterion
        // 6 asks for `regions`: a listing that iterates its whole content writes *exactly* what the
        // windowed one writes, because the engine reports a fully clipped verb as zero columns. So
        // §21's register row 4 — *writes flat 1k -> 1M* — is green on it and the hit index is not.
        standing: Standing::Red {
            by: PINS_SCENE_3,
            failing: OWED_ITS_COLLECTION,
            inverted_by: "components 12",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 4,
        on_spec_table: true,
        name: "a scrolled collection",
        size: Size::Unstated,
        content: Content::Rows { rows: 1_000_000 },
        gestures: &[Gesture::Scroll {
            rows: 1_000,
            cols: 0,
        }],
        decided: "the inverted scroll sign: 12.21 us / 3 058 writes against 62.96 / 20 418, and \
                  *faster*. Found three times independently. 75 of 80 rows here, and every counter \
                  this crate can read approves of the arm that is wrong",
        covers: &[("collection", Axis::Scrolled)],
        stands: COLLECTION,
        owed: false,
        from_a_survived_defect: true,
        standing: Standing::Red {
            by: PINS_SCENE_4,
            failing: OWED_ITS_COLLECTION,
            inverted_by: "components 12",
        },
        rehearsed_by: &[Instrument::Unit {
            file: RUNNER,
            name: "the_runner_catches_an_inverted_scroll_sign",
        }],
    },
    Scene {
        number: 5,
        on_spec_table: true,
        name: "a collection shorter than its viewport",
        size: Size::Unstated,
        content: Content::Rows { rows: 9 },
        gestures: &[Gesture::Shrink { to_rows: 9 }],
        decided: "the stale tail: 71 of 80 rows, the defective build 2.3x faster marking 226x less. \
                  2 840 cells, and the resize spelling of the same gate scores the same painter \
                  clean",
        covers: &[("collection", Axis::Shrunk)],
        stands: COLLECTION,
        owed: false,
        from_a_survived_defect: true,
        standing: Standing::Red {
            by: PINS_SCENE_5,
            failing: OWED_ITS_COLLECTION,
            inverted_by: "components 12",
        },
        rehearsed_by: &[Instrument::Unit {
            file: RUNNER,
            name: "the_runner_catches_a_stale_tail_after_content_shrinks_inside_a_rectangle_that_\
                   does_not_move",
        }],
    },
    Scene {
        number: 6,
        on_spec_table: true,
        name: "twenty wheel clicks over a collection",
        size: Size::Unstated,
        content: Content::Rows { rows: 1_000_000 },
        gestures: &[Gesture::Wheel { clicks: 20 }],
        decided: "the unconditional scroll-into-view: 0 against 16, in four resolved tickets' code. \
                  0 against 20 here, and deleting the call passes that half while losing the \
                  keyboard reveal, which is why the gate is pinned in both directions",
        covers: &[("collection", Axis::Wheeled)],
        stands: COLLECTION,
        owed: false,
        from_a_survived_defect: true,
        // **The one scene of the five that components 12 does not turn.** Its failing set is the
        // defect and not the missing subject, and ticket 12's own criterion says so: *every scene
        // of ticket 11 is green except the wheel gate, which stays pinned red for ticket 20.*
        standing: Standing::Red {
            by: PINS_SCENE_6,
            failing: "0 against 20 over twenty wheel clicks. `CONTEXT.md` forbids the \
                      unconditional `scroll_into_view` and four *resolved* tickets wrote it \
                      anyway, each by someone who had read the rule. Both directions are pinned: \
                      `Reveal::EveryFrame` settles at 0 against 20, and `Reveal::Never` — deleting \
                      the call, which passes that half — moves the offset 0 when a keyboard reveal \
                      really asks. The click's *delta* is handed to the arithmetic \
                      `Response::scrolled` would have delivered it to, because a `Mouse` cannot be \
                      named from this crate; the substitution is on both arms",
            inverted_by: "components 20",
        },
        rehearsed_by: &[Instrument::Unit {
            file: RUNNER,
            name: "the_runner_catches_twenty_wheel_clicks_that_move_nothing",
        }],
    },
    Scene {
        number: 7,
        on_spec_table: true,
        name: "a twelve-column 1M-row table, pinned both edges, horizontal overflow",
        size: Size::Unstated,
        content: Content::Grid {
            rows: 1_000_000,
            cols: 12,
        },
        gestures: &[Gesture::Scroll { rows: 0, cols: 40 }],
        decided: "verbs as the currency: 913 -> 2 497; the band's 345 re-damaged cells",
        covers: &[("table", Axis::Scrolled), ("table", Axis::Narrow)],
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 14",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 8,
        on_spec_table: true,
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
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 16",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 9,
        on_spec_table: true,
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
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 16",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 10,
        on_spec_table: true,
        name: "a 200 000-line document with 4 167 folds and an insert above them",
        size: Size::Unstated,
        content: Content::Rows { rows: 200_000 },
        gestures: &[Gesture::Fold { rows: 4_167 }, Gesture::Insert { above: 1 }],
        decided: "the anchor: 4 166 of 4 167 wrong, reanchored for 1.04 us",
        covers: &[],
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 16",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 11,
        on_spec_table: true,
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
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 21",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 12,
        on_spec_table: true,
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
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 23",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 13,
        on_spec_table: true,
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
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 23",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 14,
        on_spec_table: true,
        name: "a select, a menu with a submenu, a modal and a scrim",
        size: Size::Unstated,
        content: Content::Layers { layers: 5 },
        gestures: &[Gesture::Open { bodies: 5 }],
        decided: "the family, and the 138.04 us opening frame",
        covers: &[],
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 25",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 15,
        on_spec_table: true,
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
        stands: &[],
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
        on_spec_table: true,
        name: "175 712 axis viewport x dataset pairs",
        size: Size::Domain { pairs: 175_712 },
        content: Content::Sweep { pairs: 175_712 },
        gestures: &[Gesture::Sweep { pairs: 175_712 }],
        decided: "the axis loop oscillates on 464; from the whole domain, 0",
        covers: &[],
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 27",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 17,
        on_spec_table: true,
        name: "5 475 600 viewport x extent pairs",
        size: Size::Domain { pairs: 5_475_600 },
        content: Content::Sweep { pairs: 5_475_600 },
        gestures: &[Gesture::Sweep { pairs: 5_475_600 }],
        decided: "the bar fixpoint: 0 failures, <= 3 passes over the whole domain, and 0 bars \
                  shown that the reduced rectangle did not need. The spelling a reader writes \
                  instead has hysteresis: over content that fits with no bars at all it keeps \
                  both, losing a row and a column permanently — 19 of 20 rows and 19 of 20 \
                  columns — on a screen that looks correct. A body whose extent is not antitone \
                  in its viewport flips the decision 99 times in 99 frames with no input",
        // §9's *narrow* axis for both components, and the freeze cites this scene by number on
        // `scrollbar`'s own row: *§20's scene 17: the bar fixpoint over 5 475 600 viewport x
        // extent pairs*. `scroll_area`'s `narrow` comment cites the other half — the reserved
        // auto-hiding bars, whose hysteresis is what this scene stands up.
        covers: &[("scroll_area", Axis::Narrow), ("scrollbar", Axis::Narrow)],
        stands: SCROLLING,
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Red {
            by: &[
                PINS_THE_AREA[0],
                PINS_THE_AREA[1],
                PINS_THE_AREA[2],
                Instrument::Unit {
                    file: AREA,
                    name: "the_bar_decision_is_a_fixpoint_over_five_million_pairs",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "the_incremental_spelling_keeps_both_bars_over_content_that_fits",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "a_body_that_is_not_antitone_flips_the_decision_every_frame",
                },
            ],
            failing: OWED_ITS_SCROLL_AREA,
            inverted_by: "components 19",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 18,
        on_spec_table: true,
        name: "a 1M-row scroll area with one row in eight three cells tall",
        size: Size::Unstated,
        content: Content::Rows { rows: 1_000_000 },
        gestures: &[Gesture::Scroll {
            rows: 799_999,
            cols: 0,
        }],
        decided: "`sum h` is the extent — 1 250 000 and not 1 000 000: the row-measured spelling \
                  reaches row 799 999 of 999 999, a fifth of the content unreachable, and every \
                  count identical. Writes, distinct, verbs, regions, tab stops and allocations \
                  agree to the unit at both ends and `counters_that_separate_them` returns the \
                  empty list. The same unit error reaches the thumb, 14 cells of 69 at the end of \
                  the reachable range and 7 at 45.6% of it",
        covers: &[("scroll_area", Axis::Scrolled)],
        stands: SCROLLING,
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Red {
            by: &[
                PINS_THE_AREA[0],
                PINS_THE_AREA[1],
                PINS_THE_AREA[2],
                Instrument::Unit {
                    file: AREA,
                    name: "the_row_measured_extent_cannot_reach_the_last_row",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "nothing_a_frame_counts_separates_a_cell_extent_from_a_row_extent",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "the_thumb_and_the_extent_share_a_unit",
                },
            ],
            failing: OWED_ITS_SCROLL_AREA,
            inverted_by: "components 19",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 19,
        on_spec_table: true,
        name: "two scroll areas far apart with overlay bars",
        size: Size::Unstated,
        content: Content::Assembled { parts: 2 },
        gestures: &[Gesture::Scroll { rows: 1, cols: 0 }],
        decided: "a bar over a drawing body: 398 cells re-damaged every steady frame against 0 \
                  for the reserved twin on the same screen with the same content — the two bars' \
                  footprint, minus the corner they share, twice. §21's own figure for this row is \
                  x8.4 amplification and it is stated under a damage model the engine measured and \
                  rejected: the shipped per-row bitset is 1.00x by construction, and ADR 0029's \
                  3 535 is ~423 cells under one span per surface row",
        covers: &[],
        stands: SCROLLING,
        // **Still `(owed)`, and the reason is the finding rather than the work.** §21 marks this
        // row owed and states its figure as an *amplification*; the amplification is not
        // measurable on the engine that ships, because `crates/vitui-engine/src/damage.rs` is a
        // per-row bitset that is exact by construction. What components ticket 18 collected is the
        // number underneath it — the double write — and striking the mark here while §21 still
        // prints `(owed)` would put this value and the spec on two different sides of a sentence
        // only the spec may change.
        owed: true,
        from_a_survived_defect: false,
        standing: Standing::Red {
            by: &[
                PINS_THE_AREA[0],
                PINS_THE_AREA[1],
                PINS_THE_AREA[2],
                Instrument::Unit {
                    file: AREA,
                    name: "an_overlay_bar_re_damages_what_the_body_draws_under_it",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "the_amplification_is_one_under_the_structure_that_ships",
                },
                Instrument::Unit {
                    file: AREA,
                    name: "the_owed_figure_is_the_span_model_and_the_double_write_is_the_measurable_one",
                },
            ],
            failing: OWED_ITS_SCROLL_AREA,
            inverted_by: "components 19",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 20,
        on_spec_table: true,
        name: "the nine cells of the repertoire x tier matrix",
        size: Size::Unstated,
        content: Content::Matrix { cells: 9 },
        gestures: &[Gesture::Swap {
            what: Swap::Repertoire,
        }],
        decided: "21 -> 1 677 signal pairs; 2 distinctions of 10 lost",
        covers: &[],
        stands: &[],
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
        on_spec_table: true,
        name: "a repertoire swap with a memoised glyph prefix",
        size: Size::Unstated,
        content: Content::Matrix { cells: 598 },
        gestures: &[Gesture::Swap {
            what: Swap::Repertoire,
        }],
        decided: "the theme's revision in the key: 598 wrong characters",
        covers: &[],
        stands: &[],
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
        on_spec_table: true,
        name: "a full-screen picture at three colour depths",
        size: Size::Screen { w: 300, h: 80 },
        content: Content::Picture { depths: 3 },
        gestures: &[Gesture::Swap { what: Swap::Depth }],
        decided: "24 000 customs; 50.7% of horizontal distinctions gone at C16",
        covers: &[],
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 29",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 23,
        on_spec_table: true,
        name: "twenty selections through a directory of photographs",
        size: Size::Unstated,
        content: Content::Files { files: 20 },
        gestures: &[Gesture::Select { n: 20 }],
        decided: "R18 §5's crossover: 1 picture / 536 KB against 20 / 10.7 MB",
        covers: &[],
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 29",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 24,
        on_spec_table: true,
        name: "a directory streamed in seven batches under a sort",
        size: Size::Unstated,
        content: Content::Files { files: 200 },
        gestures: &[Gesture::Deliver { batches: 7 }, Gesture::Sort],
        decided: "the dedupe key: wrong after 7 of 7 batches, with no user in it",
        covers: &[],
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 31",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 25,
        on_spec_table: true,
        name: "a re-sort under a preview pane, 200 files",
        size: Size::Unstated,
        content: Content::Files { files: 200 },
        gestures: &[Gesture::Sort],
        decided: "197 of 200 positions move; wrong on 100 of 100 frames",
        covers: &[],
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 31",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 26,
        on_spec_table: true,
        name: "the assembled gallery, twelve panels, 53 280 cells",
        size: Size::Rect { cells: 53_280 },
        content: Content::Assembled { parts: 12 },
        gestures: &[],
        decided: "9 956 cells nobody writes; the swap excess equal to it on five of six",
        covers: &[],
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 39",
        },
        rehearsed_by: &[],
    },
    Scene {
        number: 27,
        on_spec_table: true,
        name: "a theme swap over the gallery",
        size: Size::Rect { cells: 53_280 },
        content: Content::Assembled { parts: 12 },
        gestures: &[Gesture::Swap { what: Swap::Theme }],
        decided: "R20 §3 has no caller; six panels keep the old palette permanently",
        covers: &[],
        stands: &[],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Unsubjected {
            inverted_by: "components 41",
        },
        rehearsed_by: &[],
    },
    // ── this backlog's scenes, beside §21's table ────────────────────────────────────────────────
    //
    // **The first scene on this list that §21 does not carry**, and it is not the last: `issues/11`
    // asks for a *narrow collection* beside §21's four collection rows, and `issues/23` for the
    // cluster corpus. §21's table is closed at twenty-seven and `on_spec_table` is what keeps a
    // twenty-eighth from reading as a spec change.
    Scene {
        number: 28,
        on_spec_table: false,
        name: "the narrow axis over the dense screen, at 300x80 and 120x40",
        size: Size::Two {
            wide: (300, 80),
            narrow: (120, 40),
        },
        content: Content::Assembled { parts: 338 },
        // **A shrink and not a resize.** §21 refuses to bank the shrink gate written against a
        // terminal resize, because a fresh rectangle has nowhere for the residue to survive. The
        // content that shrinks here is the widget count and the rectangle is 300x80 on every step —
        // `crate::runner::play` refuses a step that moves it.
        gestures: &[Gesture::Shrink { to_rows: 11 }],
        decided: "the one-cell ellipsis, and a label that runs into its sibling's rectangle: 324 \
                  cells re-damaged, and a label that does not narrow is green at 300x80 and red at \
                  120x40",
        covers: &[("text", Axis::Narrow), ("chip", Axis::Narrow)],
        stands: SUBJECTS,
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Evaluated {
            by: STANDS_SCENE_28,
        },
        // Empty for scene 1's reason, with `chip`'s own narrowing sweep joining the three.
        rehearsed_by: &[],
    },
    // **The second scene §21 does not carry**, and the ticket that asked for it says so:
    // `issues/11` asks for a *narrow collection* beside §21's four collection rows. §21's table is
    // closed at twenty-seven and `on_spec_table` is what keeps a twenty-ninth from reading as a
    // spec change.
    Scene {
        number: 29,
        on_spec_table: false,
        name: "a narrow collection, at 40x80 and 16x80",
        size: Size::Two {
            wide: (40, 80),
            narrow: (16, 80),
        },
        content: Content::Rows { rows: 80 },
        // **A resize and not a shrink**, and that is the pairing §21 sanctions rather than the
        // conflation it warns about: scene 5 is this component's shrink — *content shrinking inside
        // a rectangle that does not move* — so the resize spelling here stands **beside** it and
        // not instead of it. The narrow axis is the one axis on which the rectangle really is what
        // changed.
        gestures: &[Gesture::Resize { w: 16, h: 80 }],
        decided: "truncation and §16's one-cell ellipsis inside a row: 80 cells over 80 rows at 16 \
                  columns and 0 of 3 200 at 40, which is the last cell of every row",
        // **No `(collection, narrow)` pair, deliberately.** The freeze sets `narrow: false` on
        // `collection` and says why in a comment on the row: *three of the four axes are literally
        // this component's defects; the fourth is not — a collection's rows truncate through
        // `text::fit`, which is `text`'s flag*, and scene 28 already carries `(text, narrow)`. A
        // scene covering an axis the freeze does not set would inflate O5 silently, which
        // `tests::fourteen_of_the_thirty_four_axis_obligations_have_a_scene_and_twenty_do_not`
        // refuses. What this row does is *stand `collection` up*, which is the other question.
        covers: &[],
        stands: COLLECTION,
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Red {
            by: PINS_SCENE_29,
            failing: OWED_ITS_COLLECTION,
            inverted_by: "components 12",
        },
        rehearsed_by: &[Instrument::Unit {
            file: RUNNER,
            name: "a_construction_that_changes_with_the_width_is_green_wide_and_red_narrow",
        }],
    },
    // ── components ticket 18's scene, which §21's table does not carry ───────────────────────────
    Scene {
        number: 30,
        on_spec_table: false,
        name: "a scroll_area over unbounded data",
        size: Size::Screen {
            w: crate::area::AW,
            h: crate::area::AH,
        },
        content: Content::Rows { rows: 100_000 },
        gestures: &[Gesture::Volume { rows: 100_000 }],
        decided: "the wrong pairing, as the count that is the mechanism rather than the timing \
                  that is the weather: 100 000 rows iterated against 69, with **identical \
                  writes** — the engine reports a fully clipped verb as zero columns, so the \
                  counter a reader reaches for first does not move. §9 prices it at 7 907 us, 79 \
                  budgets, on a screen this ticket does not own",
        covers: &[],
        // `collection` is named in what this scene decided and is deliberately not stood up by it:
        // the screen is a `scroll_area` drawn where a `collection` belongs, and adding the other
        // half would make this a fifth scene of components ticket 11's component.
        stands: &["scroll_area"],
        owed: false,
        from_a_survived_defect: false,
        standing: Standing::Red {
            by: &[
                PINS_THE_AREA[0],
                PINS_THE_AREA[1],
                PINS_THE_AREA[2],
                Instrument::Unit {
                    file: AREA,
                    name: "a_scroll_area_costs_its_content_and_a_collection_costs_its_window",
                },
            ],
            failing: OWED_ITS_SCROLL_AREA,
            inverted_by: "components 19",
        },
        rehearsed_by: &[],
    },
];

/// **Every scene that stands `component` up.** Criterion 2's enumeration, from the scene's side.
///
/// Both [`Scene::covers`] and [`Scene::stands`], because a component with no declared hostile axis
/// can never appear in the first — `panel` and `button` declare none, and O5's join is
/// `(component, axis)`. A screen those two are drawn on is still a screen they are drawn on, and
/// `.scratch/vitui-components-impl/issues/09`'s criterion 1 asks this question about all four.
pub fn scenes_for(component: &str) -> impl Iterator<Item = &'static Scene> + '_ {
    SCENES.iter().filter(move |s| {
        s.covers.iter().any(|(id, _)| *id == component) || s.stands.contains(&component)
    })
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
            // **A measured line still says where the scene stands.** A rehearsal replacing the
            // standing outright would print components ticket 11's wheel gate as a healthy frame,
            // which is the defective build's own self-portrait one column further left: three of
            // the four axes were *faster*.
            Some((_, row)) => {
                let pin = match scene.standing {
                    Standing::Red { inverted_by, .. } => {
                        format!("; red, pinned: `{inverted_by}` inverts it")
                    }
                    _ => String::new(),
                };
                let _ = writeln!(
                    out,
                    "{label:<52}  {}  [rehearsed over a fixture{pin}]",
                    row.columns()
                );
            }
            None => match scene.standing {
                Standing::Unsubjected { inverted_by } => {
                    let _ = writeln!(
                        out,
                        "{label:<52}  not played: `{inverted_by}` builds the subject"
                    );
                }
                // **A stood-up scene says what stands it up.** Not *not played*: the screen is
                // drawn, its components exist and its numbers are asserted — this line is where a
                // reader sees which of the twenty-eight are screens rather than intentions. It was
                // `red, pinned: waiting for its components` for exactly one ticket, which is
                // criterion 7's distinction on the line a reader actually looks at.
                Standing::Evaluated { by } => {
                    let _ = writeln!(
                        out,
                        "{label:<52}  stood up on its own components, by {} instruments",
                        by.len()
                    );
                }
                // **A red scene is not an unplayed one, and the line has to say which.** Components
                // ticket 11's five screens are drawn and measured; four are waiting for
                // `collection` and one is pinned on the defect itself. Printing them as *not
                // played* would lose the distinction criterion 7 exists for, and printing them as
                // *stood up* would claim a component that does not exist.
                Standing::Red {
                    by, inverted_by, ..
                } => {
                    let _ = writeln!(
                        out,
                        "{label:<52}  red, pinned: `{inverted_by}` inverts it, by {} instruments",
                        by.len()
                    );
                }
                other => {
                    let _ = writeln!(out, "{label:<52}  {}", other.word());
                }
            },
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
    fn the_specs_table_is_twenty_seven_rows_and_this_list_carries_them_all() {
        assert_eq!(SCENES.len(), 30);
        let numbers: BTreeSet<u8> = SCENES.iter().map(|s| s.number).collect();
        assert_eq!(numbers, (1..=30).collect::<BTreeSet<u8>>());
        let names: BTreeSet<&str> = SCENES.iter().map(|s| s.name).collect();
        assert_eq!(names.len(), SCENES.len(), "two scenes share a name");

        // **The split, not the total** — `crate::gates`'s arrangement. §21's rows come first and a
        // ticket's come after, so a row claiming to be the spec's is a claim about a position as
        // well as about a flag.
        let on_table = SCENES.iter().filter(|s| s.on_spec_table).count();
        assert_eq!(on_table, 27);
        assert_eq!(
            SCENES.len() - on_table,
            3,
            "components 09's narrow axis, components 11's narrow collection and components 18's \
             wrong pairing"
        );
        for (index, scene) in SCENES.iter().enumerate() {
            assert_eq!(
                scene.on_spec_table,
                index < on_table,
                "scene #{} is out of the spec-table block",
                scene.number
            );
        }

        // And the spec's own table, counted off the file.
        let spec = read(".scratch/vitui-components-architecture/spec.md");
        let rows = spec
            .lines()
            .skip_while(|l| !l.starts_with("| scene | what it decided |"))
            .skip(2)
            .take_while(|l| l.starts_with('|'))
            .count();
        assert_eq!(
            rows, on_table,
            "§21's table and the rows of this list that claim to be §21's disagree about how many \
             scenes there are. That is a spec change or a deletion, and either has to argue for \
             itself here"
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

    /// **Eighteen scenes have nothing to run over, nine are pinned red and three are stood up.**
    ///
    /// `obligations.rs`'s arrangement and `gates.rs`'s: a count makes every change of colour a
    /// deliberate edit here rather than a quiet one. It was **twenty-five, none and three** until
    /// components ticket 11 and **twenty-one, five and three** until components ticket 18, and
    /// every scene that moved off `Unsubjected` moved *up*: `Unsubjected` says *it could run and
    /// there is nothing to run it over*, and a great deal now runs.
    ///
    /// # Four of the nine are pinned to components 12, four to components 19 and one to 20
    ///
    /// That split is the whole of criterion 7 and it is asserted rather than described. Scenes 3, 4,
    /// 5 and 29 are waiting for `collection` — their screens are drawn and their numbers measured,
    /// and what is missing is the subject. **Scene 6 is not waiting for anything**: it is red
    /// because the defect is real, `CONTEXT.md` forbids it and four *resolved* tickets wrote it
    /// anyway. Components ticket 12's own criterion says so — *every scene of ticket 11 is green
    /// except the wheel gate, which stays pinned red for ticket 20* — and a single `inverted_by`
    /// across the five would erase it.
    #[test]
    fn eighteen_scenes_have_nothing_to_run_over_nine_are_red_and_three_are_stood_up() {
        let red: Vec<u8> = SCENES
            .iter()
            .filter(|s| matches!(s.standing, Standing::Red { .. }))
            .map(|s| s.number)
            .collect();
        assert_eq!(
            red,
            vec![3, 4, 5, 6, 17, 18, 19, 29, 30],
            "components ticket 11's five — the three volumes, the inverted sign, the stale tail, \
             the twenty wheel clicks and the narrow collection — and components ticket 18's four: \
             the bar fixpoint, the `sum h` extent, the two areas far apart and the wrong pairing. \
             A tenth is a new one, and it owes an exact failing set and a ticket that inverts it"
        );
        let evaluated: Vec<u8> = SCENES
            .iter()
            .filter(|s| s.standing.evaluated())
            .map(|s| s.number)
            .collect();
        assert_eq!(
            evaluated,
            vec![1, 2, 28],
            "the dense screen, the same screen drawn naive and correct, and the narrow axis. A \
             screen played over a stand-in for its components is rehearsed and not stood up, so a \
             fourth arriving here is a deliberate edit to this module's header and to \
             `crate::gates::REGISTER`"
        );
        assert_eq!(SCENES.len() - evaluated.len() - red.len(), 18);

        let mut pinned_to = Vec::new();
        for scene in SCENES {
            let inverted_by = match scene.standing {
                Standing::Unsubjected { inverted_by } => Some(inverted_by),
                Standing::Evaluated { by } => {
                    assert!(
                        !by.is_empty(),
                        "scene #{} is evaluated by nothing, which is a sentence about a screen",
                        scene.number
                    );
                    None
                }
                Standing::Red {
                    by,
                    failing,
                    inverted_by,
                } => {
                    assert!(
                        !by.is_empty(),
                        "scene #{} is pinned red by nothing, which is a claim and not a gate",
                        scene.number
                    );
                    assert!(
                        failing.len() > 60 && failing.chars().any(|c| c.is_ascii_digit()),
                        "scene #{} is red without an exact failing set. §21: a red gate asserts \
                         its exact failing set, fires in both directions and says what to invert",
                        scene.number
                    );
                    pinned_to.push((scene.number, inverted_by));
                    Some(inverted_by)
                }
                other => panic!("scene #{} is {}", scene.number, other.word()),
            };
            if let Some(inverted_by) = inverted_by {
                assert!(
                    inverted_by.starts_with("components "),
                    "scene #{} does not name the ticket that will stand it up",
                    scene.number
                );
            }
        }
        assert_eq!(
            pinned_to,
            vec![
                (3, "components 12"),
                (4, "components 12"),
                (5, "components 12"),
                (6, "components 20"),
                (17, "components 19"),
                (18, "components 19"),
                (19, "components 19"),
                (29, "components 12"),
                (30, "components 19"),
            ],
            "the wheel gate is not waiting for its subject and the other eight are. Merging the \
             two loses the distinction components 11's criterion 7 is about, and components ticket \
             18's four are all on the subject side of it: `scroll_area`, `scrollbar` and `sticky` \
             are undeclared and no defect of theirs is pinned here"
        );
    }

    /// **The three scenes stand on the same four components, and the fact is computed rather than
    /// typed.**
    ///
    /// Ticket 09's criterion 7, inverted by ticket 10. [`crate::dense::subjects_declared`] opens the
    /// four files the freeze homes the components in and reads what is declared there, so the day
    /// one of them moves this test fails and the standing is a deliberate edit — in the same
    /// direction it was made in.
    #[test]
    fn the_three_stood_up_scenes_rest_on_the_same_four_components() {
        assert_eq!(SUBJECTS, ["text", "chip", "button", "panel"]);
        assert_eq!(
            crate::dense::subjects_declared(),
            SUBJECTS.to_vec(),
            "a component of the dense screen is no longer declared where the freeze homes it, so \
             scenes 1, 2 and 28 are standing on a screen made of their construction again"
        );
        for number in [1u8, 2, 28] {
            let scene = SCENES
                .iter()
                .find(|s| s.number == number)
                .expect("a numbered scene");
            let Standing::Evaluated { by } = scene.standing else {
                panic!("scene #{number} is not evaluated");
            };
            // **The pair every one of the three shares**, whatever else stands each of them up: the
            // four subjects are declared, and the sentence for the day one of them is not.
            for shared in STANDS_ON_ITS_COMPONENTS {
                assert!(
                    by.contains(shared),
                    "scene #{number} does not name `{shared:?}`, so its standing no longer rests \
                     on the four components being declared"
                );
            }
            assert_eq!(scene.stands, SUBJECTS);
        }
        // And the sentence a reader would see if one of them went away, watched saying it — over a
        // declaration list rather than over the crate, because the crate no longer produces it.
        let message = crate::dense::owed_message(&["text", "chip"], "scene 1")
            .expect("two of the four missing is a scene that is not standing");
        assert!(message.contains("components 10"), "{message}");
        assert!(
            message.contains("waiting for its subject rather than failing"),
            "a failure that could be read as a defect in the screen: {message}"
        );
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

    /// **O5 moves from 34 of 34 to 18 of 34, and it still fails loudly.**
    ///
    /// This ticket cannot turn O5 and does not pretend to. Sixteen of the thirty-four
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
    fn sixteen_of_the_thirty_four_axis_obligations_have_a_scene_and_eighteen_do_not() {
        let evidence = axis_scenes();
        assert_eq!(evidence.len(), 16);
        assert_eq!(
            crate::obligations::AXIS_SCENES.to_vec(),
            evidence,
            "the constant O5 reads and the list it is derived from have drifted"
        );

        let coverage = coverage();
        assert_eq!(coverage.len(), 34, "the population §17 states");
        let bare = coverage.iter().filter(|(_, _, s)| s.is_empty()).count();
        assert_eq!(bare, 18);

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
    ///
    /// Components ticket 09's criterion 1: *three scenes registered in the scene list, each
    /// answering `scenes_for` for `text`, `chip`, `button` and `panel`*. Two of those four declare
    /// no hostile axis at all, which is why the answer comes from [`Scene::stands`] as well as from
    /// [`Scene::covers`] — see `scenes_for`.
    #[test]
    fn scenes_for_a_component_lists_the_scenes_that_stand_it_up() {
        let collection: Vec<u8> = scenes_for("collection").map(|s| s.number).collect();
        assert_eq!(
            collection,
            vec![3, 4, 5, 6, 29],
            "components ticket 11's five: the three that exist because a defect survived every \
             gate, the three volumes and the narrow collection"
        );

        // **Criterion 1, and it is the reason the narrow collection is a scene at all**: those five
        // answer for all four axis columns. Three come through `covers`, because `collection`
        // declares those three axes; the fourth comes through `stands`, because the freeze sets
        // `narrow: false` on `collection` — a row truncates through `text::fit`, which is `text`'s
        // flag — and a `(collection, narrow)` pair here would be evidence for an axis the freeze
        // does not claim.
        let claimed: Vec<Axis> = SCENES
            .iter()
            .filter(|s| collection.contains(&s.number))
            .flat_map(|s| s.covers.iter().filter(|(id, _)| *id == "collection"))
            .map(|(_, axis)| *axis)
            .collect();
        assert_eq!(claimed, vec![Axis::Scrolled, Axis::Shrunk, Axis::Wheeled]);
        let narrow = SCENES
            .iter()
            .find(|s| s.number == 29)
            .expect("the narrow collection");
        assert_eq!(narrow.covers, &[] as &[(&str, Axis)]);
        assert_eq!(narrow.stands, ["collection"]);
        assert!(matches!(narrow.gestures, [Gesture::Resize { .. }]));
        assert_eq!(
            scenes_for("table").map(|s| s.number).collect::<Vec<_>>(),
            vec![7]
        );
        assert_eq!(
            scenes_for("checkbox").count(),
            0,
            "a component with no declared axis and no screen has no scene, and that is not a failure"
        );
        // Criterion 1, for all four of the dense screen's components.
        for id in SUBJECTS {
            assert_eq!(
                scenes_for(id).map(|s| s.number).collect::<Vec<_>>(),
                vec![1, 2, 28],
                "`{id}` does not answer `scenes_for` for the three scenes components 09 stood up"
            );
        }
        // And two of them are in **no** `(component, axis)` pair, which is the distinction the two
        // fields exist for: `panel` and `button` declare no hostile axis, so O5 asks nothing about
        // them and a screen still stands them up.
        for id in ["panel", "button"] {
            assert!(!axis_scenes().iter().any(|(c, _)| *c == id));
        }
    }

    /// **Seven scenes are rehearsed, and a rehearsal names a live test.**
    ///
    /// It was ten until components ticket 10. Scenes 1 and 28 lost theirs entirely and scene 2 kept
    /// one, because what those instruments run over stopped being a stand-in: they are `standing`
    /// now, and `a_rehearsal_is_never_what_stands_a_scene_up` is where the line between the two is
    /// asserted rather than described.
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
        assert_eq!(rehearsed, vec![2, 4, 5, 6, 15, 20, 21, 29]);

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

    /// **A rehearsal is never a standing**, and this test is where that is written down.
    ///
    /// A rehearsal says *the instrument catches this defect over a fixture or over a stand-in*; a
    /// standing says *this screen is on a terminal*. Conflating them is how a register comes to
    /// report a gate that nothing runs, which is §21's three-for-three finding from the other
    /// direction.
    ///
    /// **The three rows components 09 built are the case that made this sharper**, and components
    /// ticket 10 is where the line was crossed *by the code moving rather than by the field moving*.
    /// Their screen was drawn out of the four components' **construction**; it is now drawn out of
    /// the four components, so the instruments that were `rehearsed_by` are `standing` and not one
    /// character of them changed.
    ///
    /// What the rule still forbids is promotion by rehearsal: a scene may be `Evaluated` **only**
    /// over the components it is a screen of, which is what the file check below is — every
    /// instrument standing one of the three up lives in `crate::dense` or in a component's own
    /// module, and none of them is a `crate::runner` fixture.
    ///
    /// # A red scene is held to the same rule, and it is the rule that makes red mean anything
    ///
    /// Components ticket 11's five are pinned red on instruments in `crate::listing` — the screen's
    /// own file — and on the one barrier that keeps half the wheel gate unwritable. **A red row
    /// pinned on a `crate::runner` fixture would be claiming a screen it does not have**, which is
    /// exactly the promotion this test refuses one arm over.
    #[test]
    fn a_rehearsal_is_never_what_stands_a_scene_up() {
        // The files where a screen or one of its components is measured. `crate::runner`'s is not
        // one of them, and that is the whole check.
        const SCREEN_FILES: [&str; 7] = [
            DENSE,
            LISTING,
            AREA,
            "crates/vitui-components/src/text.rs",
            "crates/vitui-components/src/input.rs",
            "crates/vitui-components/src/structure.rs",
            "crates/vitui-components/tests/gates.rs",
        ];
        for scene in SCENES {
            match scene.standing {
                Standing::Unsubjected { .. } => {}
                Standing::Evaluated { by } | Standing::Red { by, .. } => {
                    for instrument in by {
                        let file = instrument.file();
                        let barrier = matches!(instrument, Instrument::Barrier { .. });
                        assert!(
                            barrier || SCREEN_FILES.contains(&file),
                            "scene #{} rests on `{file}`, which is not a file where this screen or \
                             one of its components is measured. A fixture is a rehearsal",
                            scene.number,
                        );
                    }
                    assert!(
                        by.len() >= 3,
                        "scene #{} rests on fewer than three instruments",
                        scene.number
                    );
                }
                other => panic!("scene #{} is {}", scene.number, other.word()),
            }
            // And nothing rehearses a scene that is already stood up on its own components: a
            // rehearsal beside a standing would let the standing be argued from the fixture.
            if scene.standing.evaluated() {
                for instrument in scene.rehearsed_by {
                    assert_eq!(
                        instrument.file(),
                        RUNNER,
                        "scene #{} is stood up and still rehearsed in `{}`. The runner's own \
                         fixtures are the one thing that may sit beside a standing, because they \
                         are about the instrument and not about the screen",
                        scene.number,
                        instrument.file()
                    );
                }
            }
        }
    }

    /// **The shrink gesture and the resize gesture are both present, and they are not the same.**
    ///
    /// §21 refuses to bank *the surface after a shrink equals a freshly built one* precisely because
    /// the version written against a terminal resize passes on all twelve panels. Scenes 5 and 28
    /// shrink the content and scenes 13 and 15 resize the rectangle, and a list carrying only the
    /// second kind would have banked the gate.
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
        assert_eq!(shrinks, vec![5, 28]);
        assert_eq!(
            resizes,
            vec![13, 15, 29],
            "components ticket 11's narrow collection is a resize, and it stands **beside** scene \
             5's shrink rather than instead of it — which is the pairing §21 sanctions"
        );
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
        assert_eq!(printed.matches("not played: `components ").count(), 18);
        assert_eq!(
            printed
                .matches("stood up on its own components, by ")
                .count(),
            3,
            "a stood-up scene says what stands it up, rather than reading as unplayed"
        );
        assert_eq!(
            printed.matches("red, pinned: `components ").count(),
            9,
            "components ticket 11's five and components ticket 18's four, and a red line is \
             neither *not played* nor *stood up*"
        );
        assert_eq!(printed.matches("[rehearsed over a fixture").count(), 1);
        assert!(
            printed.contains("[rehearsed over a fixture; red, pinned: `components 12` inverts it]"),
            "a measured line over a red scene still says it is red, or a report of the wheel gate \
             would read as a healthy frame: {printed}"
        );
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
        assert_eq!(SCENES.len() - still.len(), 27);
    }
}
