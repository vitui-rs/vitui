//! The five obligations and the two stated after the map closed, as **queries over
//! [`crate::INVENTORY`]**, each returning a count or an equality.
//!
//! > Every documentation and verification obligation is a query over it, not a sentence in a
//! > document.
//!
//! # All seven run, and why that does not make the vacuity refusal a spare part
//!
//! **All nine queries are green**, and O5 turned last, which is all nine
//!
//! queries. O5 was last on purpose: it is the one worth more than the other four
//! together, its evidence is thirty-four `(component, axis)` pairs, and production tickets 05 to 09
//! took the fourteen of them that had no scene. **A query over an obligation nobody has met yet is
//! the exact shape that returns green by accident**, and three of them had it:
//!
//! - *every panel in the gallery is in the freeze* over an empty gallery is **vacuously true** —
//!   which is what [`PANELS`] read for a while, and the reason the query is written over the
//!   panel list rather than over the freeze;
//! - *every component has at least one scene per declared axis* over an empty scene list, written
//!   as a loop over scenes rather than over components, is **vacuously true**;
//! - *goldens == constructions* written as `for g in goldens` is **vacuously true** — which is why
//!   [`o3`] iterates the freeze and not the evidence, and why its population is `built` rather than
//!   all twenty-nine: a row nothing on this backlog can draw a screen for reads **red** for ever,
//!   which is the same failure in mirror image.
//!
//! **O7's population is read out of the source rather than off a column**, and that is the one place
//! the seven differ about what *the freeze says* means: [`o1`] and [`o3`] trust `built`, and
//! that column read `built: true` for `slider` for a while
//! with no `slider` anywhere in the crate. [`crate::consumer::declared`] opens each row's home
//! module instead.
//!
//! **O6 is the one whose population is derived rather than written out**, and it is the only one of
//! the seven that could go quietly *smaller*: [`o6`] reads the freeze's `Layer::L2` column and the
//! memo census, so a component that stops declaring either leaves the population without leaving a
//! failing set. That is why [`crate::volume::COVERED`] is compared against the derivation as an
//! **ordered list** rather than as a subset — a row that leaves the population is a failing test in
//! `crate::volume` before it is a smaller `over` here.
//!
//! Three of the seven have that shape, and the neighbouring version has already bitten
//! of it twice: the gallery's theme-swap gate asserted `changed > 0` — *the theme changed and not
//! one cell moved* — and **one cell of 4 800 satisfies it** while 3 583 carried the old palette;
//! every prototype reported `allocs / n` with n between 40 and 200, so **a frame allocating on n−1
//! of n frames reports 0**.
//!
//! So [`Verdict`] has two arms and vacuity is refused **in the constructor**: [`Verdict::of`]
//! returns [`Verdict::Unmet`] when the population it was asked about is empty, before it ever looks
//! at the failing set. An obligation cannot report itself met by having nothing to check.
//!
//! # How they fail loudly
//!
//! [`Verdict::assert_met`] panics with the failing set and what is wrong with it. Each of the
//! nine is watched panicking by a `#[should_panic]` test below, because **a gate nobody has watched
//! fail is not a gate** — the three-for-three finding, from the other direction.
//! `tests::all_nine_obligation_queries_are_met_and_o5_was_the_last_to_turn` writes the number down,
//! so each one that turns is a deliberate edit here rather than a silent change of colour. **All
//! seven have turned and each cost that edit** — and this sentence itself said *five* while six
//! were green, which is the defect a later pass was opened for, met inside the file it was opened
//! about. What changes when one turns is the arm it is watched failing on: a `Met` verdict cannot
//! be watched panicking, so the `#[should_panic]` moves from
//! *The evidence is empty* to *the evidence is one row wrong*, and for O2 and O7 that is one arm
//! per direction. **Every one of the nine is now watched on the second arm**, so the refusal above
//! is no longer load-bearing for any live verdict — and it stays, because the population an
//! obligation is asked about can go empty on its own: [`o6`] derives its from the freeze, and a
//! component that stops declaring a memo census leaves it smaller with no failing set.
//!
//! # The evidence is an argument, not a file read
//!
//! Every query takes what it is checking against as a slice, and this crate shipped each of those
//! slices **empty with the ticket that fills it named**. That is what keeps the queries pure functions
//! over the freeze: the gallery was built, [`PANELS`] stopped being empty, and `o2` started
//! answering a real question **with no change to either query**.
//!
//! **[`DOC_TESTED`] was the first one to be filled, and filling it needed a second value beside
//! it.** [`PANELS`] is the fourth, [`VOLUME_MEASURED`] the fifth and [`APPLIED`] the sixth, and each
//! needed the same thing, for the same reason and with the same shape: [`crate::gallery::panel_ids`] derives the list from the table the screen is drawn
//! from.
//! A written-out evidence list is a claim about twenty-eight files, so [`crate::doc`] opens each of
//! them and derives the same list; `crate::doc::tests::the_written_list_and_the_scan_agree` is the
//! comparison. Two lists a test compares is the arrangement [`KEYBOARD_DOCUMENTED`] and
//! [`KEYBOARD_REGISTERED`] already use, and for O4's reason: an equality between two things derived
//! from each other holds.

use crate::{Axis, Component, INVENTORY};

/// Whether an obligation is met, with enough in the `Unmet` arm to act on.
///
/// **Two arms and no third.** A *not yet applicable* arm is what would let an obligation report
/// nothing rather than report failure, which is the whole failure mode this file exists to close.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Verdict {
    /// It holds, over a population that was not empty.
    Met {
        /// How many things were checked.
        over: usize,
    },
    /// It does not hold, or it could not be asked.
    Unmet {
        /// How many things were checked. **Zero is itself a failure** — see [`Verdict::of`].
        over: usize,
        /// How many of them failed. Equal to `over` when the population was empty.
        failing: usize,
        /// What is wrong, in one line, ending in something a reader can act on.
        ///
        /// **The whole of it**: what would have to change is said here, in words, and not as a
        /// pointer at a ticket the reader has no copy of.
        why: &'static str,
    },
}

impl Verdict {
    /// Build a verdict, refusing vacuous truth.
    ///
    /// **An empty population is `Unmet`, whatever the failing count says.** A query asked about
    /// nothing has not been answered, and the arithmetic that says otherwise — zero failures out of
    /// zero — is exactly how three of them would read green today.
    pub const fn of(over: usize, failing: usize, why: &'static str) -> Verdict {
        if over == 0 {
            return Verdict::Unmet {
                over: 0,
                failing: 0,
                why,
            };
        }
        if failing == 0 {
            return Verdict::Met { over };
        }
        Verdict::Unmet { over, failing, why }
    }

    /// Whether it holds.
    pub const fn met(self) -> bool {
        matches!(self, Verdict::Met { .. })
    }

    /// Panic with the failing set and what is wrong with it.
    ///
    /// This is the loud half. It is what a later ticket's gate calls, and what the
    /// `#[should_panic]` tests below watch, because a failure nobody has read is a failure nobody
    /// can act on.
    pub fn assert_met(self, obligation: &str) {
        if let Verdict::Unmet { over, failing, why } = self {
            panic!("{obligation} is unmet: {failing} of {over} — {why}");
        }
    }
}

/// The ids that carry a rustdoc page with a **compiled** example. O1's evidence.
///
/// **Twenty-eight of twenty-nine.** Every built row of the
/// freeze carries a doctest that calls it from outside the crate — which is C01's actual question —
/// beside the `**Hostile axes:**` line its page owes and the `#![deny(missing_docs)]` the crate
/// compiles under. The twenty-ninth is `spinner`; see [`o1`].
///
/// It is written out rather than computed, for [`AXIS_SCENES`]'s reason, and it is **not** the only
/// value that knows this: [`crate::doc::doc_tested`] derives the same list by opening each page's
/// file, and `crate::doc::tests::the_written_list_and_the_scan_agree` compares them. A list filled
/// by whichever ticket happened to write a doctest is a list nobody audits — which is why this one
/// stayed empty through twenty-five tickets that were each entitled to add a row to it.
///
/// The order is [`INVENTORY`]'s, so a row added to the freeze in the middle is a failing test here
/// rather than a tidy append.
pub const DOC_TESTED: &[&str] = &[
    "text",
    "panel",
    "chip",
    "button",
    "field",
    "collection",
    "table",
    "tree",
    "select",
    "overlay",
    "scroll_area",
    "scrollbar",
    "sticky",
    "collapsible",
    "chart",
    "plot",
    "checkbox",
    "radio",
    "switch",
    "meter",
    "sparkline",
    "rule",
    "status_bar",
    "pagination",
    "form",
    "slider",
    "spinner",
    "file_picker",
    "file_preview_pane",
];

/// The ids the gallery binary shows a panel for. O2's evidence.
///
/// **Twenty-eight.** The gallery is a gate rather than a demo:
/// every defect on the map that survived every gate then in force was invisible on the
/// screen of the ticket that owned the mechanism and visible on the screen where the components
/// meet.
///
/// Written out, for [`DOC_TESTED`]'s reason — a `const fn` over the freeze would make the
/// population and the evidence one expression, and an equality between two things derived from each
/// other holds. What holds it honest is [`crate::gallery::panel_ids`], which derives the same list
/// from the panel table the screen is drawn from, joined by
/// `crate::gallery::tests::the_written_list_and_the_table_agree`. `spinner` is absent, and that is
/// [`o2_everything_built_has_a_panel`]'s population rather than an omission here.
pub const PANELS: &[&str] = &[
    "text",
    "panel",
    "chip",
    "button",
    "field",
    "collection",
    "table",
    "tree",
    "select",
    "overlay",
    "scroll_area",
    "scrollbar",
    "sticky",
    "collapsible",
    "chart",
    "plot",
    "checkbox",
    "radio",
    "switch",
    "meter",
    "sparkline",
    "rule",
    "status_bar",
    "pagination",
    "form",
    "slider",
    "spinner",
    "file_picker",
    "file_preview_pane",
];

/// How many golden screens each id has. O3's evidence.
///
/// **One golden per construction, not per matrix cell**: nine screenshots per component is nine
/// times the maintenance for a claim the count in the theme's matrix already makes, and screens
/// declared identical are asserted identical instead.
///
/// Written out, for [`DOC_TESTED`]'s reason: a `const fn` over the freeze would make the population
/// and the evidence one expression, and an equality between two things derived from each other
/// holds. What holds it honest is **two** other sources — `crate::golden::counted`, which is what
/// the screen table adds up to, and `crate::golden::on_disk`, which opens the directory — joined by
/// `tests/golden.rs`'s `the_three_sources_agree_about_how_many_screens_there_are`. Neither of the
/// first two would notice a golden that had been deleted.
///
/// `spinner` is absent, and that is [`o3`]'s population rather than an omission here.
pub const GOLDENS: &[(&str, u8)] = &[
    ("text", 1),
    ("panel", 1),
    ("chip", 1),
    ("button", 1),
    ("field", 1),
    ("collection", 1),
    ("table", 1),
    ("tree", 1),
    ("select", 1),
    ("overlay", 1),
    ("scroll_area", 1),
    ("scrollbar", 1),
    ("sticky", 1),
    ("collapsible", 1),
    ("chart", 2),
    ("plot", 3),
    ("checkbox", 1),
    ("radio", 1),
    ("switch", 1),
    ("meter", 2),
    ("sparkline", 2),
    ("rule", 1),
    ("status_bar", 1),
    ("pagination", 1),
    ("form", 1),
    ("slider", 1),
    ("spinner", 3),
    ("file_picker", 1),
    ("file_preview_pane", 1),
];

/// The ids whose keyboard contract is **documented** and rendered as help. O4's first half.
///
/// Thirteen. Written out for [`AXIS_SCENES`]'s reason: a
/// `const fn` over [`crate::contract::CONTRACTS`] would make the population and the evidence one
/// expression. What holds it honest is `crate::contract::tests::the_written_lists_and_the_sweep_agree`.
///
/// **The sixteen rows that are not here read no key at all**, and that is the faithful reading
/// rather than a gap — see [`o4`] for why the population is the union of the two lists and not the
/// whole freeze.
pub const KEYBOARD_DOCUMENTED: &[&str] = &[
    "collection",
    "table",
    "tree",
    "pagination",
    "select",
    "file_picker",
    "field",
    "form",
    "collapsible",
    "slider",
    "checkbox",
    "radio",
    "switch",
];

/// The ids whose keyboard contract is **registered** — the ones the machine is observed to answer.
///
/// O4 is an equality between the two and not a subset in either direction: a binding that is
/// registered and undocumented is a feature nobody can find, and one that is documented and
/// unregistered is a help bar that lies.
///
/// **The two lists are the same thirteen ids and the equality is not at this level.** A roll-up to
/// ids is what [`o4`] can ask over the freeze, and it would be satisfied by a component declaring
/// one binding and answering a hundred. The chord-for-chord equality is
/// `crate::contract::tests::documented_equals_registered_for_every_component`, over a corpus of
/// **162** triggers a component could bind, with the runtime's own share subtracted by a control
/// arm — and it is what turned four defects up in code that was already green.
pub const KEYBOARD_REGISTERED: &[&str] = &[
    "collection",
    "table",
    "tree",
    "pagination",
    "select",
    "file_picker",
    "field",
    "form",
    "collapsible",
    "slider",
    "checkbox",
    "radio",
    "switch",
];

/// The scenes that exist, as `(component, axis)` pairs. O5's evidence.
///
/// **Twenty-five of thirty-four**, filled in over many passes: twelve off the rows to begin with,
/// then two, then one, then three, then `field`'s last three and `table`'s last two.
///
/// **The three are the first entries here that no components ticket could have added**,
/// and the distinction is worth keeping beside the others. One was
/// finding was *a component is not a scene*; the original's was *a standing is not a pair*; ticket
/// another was *a pair there was no way to state*. This one is neither: there is one `field`
/// row, the freeze declares four axes for the component, and the other three were expressible from
/// the day the freeze was written. Nothing had scheduled them — which is the gap
/// the production backlog exists to close, and the reason an assertion that is accurate about
/// the present is not a plan.
///
/// **The first is the pair that could not be stated**, and it is worth separating from the two below
/// it for the same reason they are separated from each other. The wheel is one row over one
/// component, which is exactly what was writable while the click was an arithmetic substitution — a
/// delta added to an offset has no second axis to be wrong on. `scroll_area` has declared
/// `owns_offset` from the start and had no scene claiming the axis; one exists now, and
/// what it decided is a pair rather than a number: a body dead downward is alive sideways.
///
/// **Five scenes were built for `collection` and added no pair either, for a reason worth
/// separating from the last.** That one was *a component is not a scene*; this one is *a
/// standing is not a pair*. The three pairs `collection` declares — scrolled, shrunk, wheeled — were
/// already claimed here, off three **scene** rows. What changed is that
/// those three scenes stopped being `Unsubjected` and became `Red` with an exact failing set, and
/// [`axis_scenes_of`](crate::scenes::axis_scenes_of) does not read `standing` at all: its one filter
/// is `owed`, because the `(owed)` marks a scene *whose number was never measured*, and a red
/// scene's numbers are measured. The fifth scene, the narrow collection, deliberately claims **no**
/// pair — `INVENTORY` sets `narrow: false` on `collection` because a row truncates through
/// `text::fit`, which is `text`'s flag, and a scene already carries `(text, narrow)`. A pair here
/// for an axis the freeze does not set is evidence for nothing and would inflate the count silently.
///
/// Four components were built and added no pair, which is not an oversight and is worth the
/// sentence: `panel` and `button` declare **no** hostile axis at all, and the axes `text` and `chip`
/// do declare — [`Axis::Narrow`], both of them — already have that scene, which was written
/// *before* either component existed. **O5 is a query about axes and not about components**, which
/// is exactly why it was written over `INVENTORY` rather than over the scene list: building a
/// component cannot move it, and only a scene can. Each names a component **and** the mechanism of an axis it declares; the join lives on
/// [`crate::scenes::Scene::covers`] and
/// `scenes::tests::seventeen_of_the_thirty_four_axis_obligations_have_a_scene_and_seventeen_do_not`
/// asserts that this constant and [`crate::scenes::axis_scenes`] have not drifted.
///
/// The other twenty are the per-component scenes tickets'. **The first ticket of each component
/// on this backlog is its hostile-axis scenes, not its drawing**, and a scenes ticket is red on
/// purpose until its component ticket lands.
///
/// The list is written out rather than computed, because [`o5`] takes a slice and a `const fn` over
/// `SCENES` would make the population and the evidence one expression. Two lists that a test
/// compares is the arrangement `KEYBOARD_DOCUMENTED` and `KEYBOARD_REGISTERED` already use one file
/// over, and for O4's reason: an equality between two things derived from each other holds.
pub const AXIS_SCENES: &[(&str, Axis)] = &[
    // the design scene 4 — the inverted scroll sign, 12.21 us against 62.96 and *faster*.
    ("collection", Axis::Scrolled),
    // the design scene 5 — the stale tail, 71 of 80 rows.
    ("collection", Axis::Shrunk),
    // the design scene 6 — twenty wheel clicks move the offset 0 against 16.
    ("collection", Axis::Wheeled),
    // the design scene 7 — a twelve-column table under a horizontal offset, pinned both edges.
    ("table", Axis::Scrolled),
    ("table", Axis::Narrow),
    // the design scene 8 — a million-node forest at depth 59 999, windowed by the flatten index.
    ("tree", Axis::Scrolled),
    // the design scene 9 — a fold and an unfold: content shrinking inside a rectangle that does not move.
    ("tree", Axis::Shrunk),
    // the design scene 11 — the accordion, 478 hit entries against 70.
    ("collapsible", Axis::Shrunk),
    // the design scene 13 — the wrap memo at 300 and at 120, 625 rows drawn where 875 are needed.
    ("field", Axis::Narrow),
    // the design scene 15 — 60x20, where C08's overlap is red.
    ("chart", Axis::Narrow),
    ("plot", Axis::Narrow),
    // the design scene 17 — the bar fixpoint over 5 475 600 pairs. The freeze cites this scene by number
    // on `scrollbar`'s own `narrow` row, and `scroll_area`'s cites the other half of it: reserved
    // auto-hiding bars, whose hysteresis loses a row and a column permanently.
    ("scroll_area", Axis::Narrow),
    ("scrollbar", Axis::Narrow),
    // the design scene 18 — a 1M-row scroll area, row 799 999 of 999 999.
    ("scroll_area", Axis::Scrolled),
    // the design scenes 23 and 25 — the preview pane's shrink and scroll, and the picker's scroll.
    // **Empty until an earlier pass**, which is not a filing decision: while all three scenes were
    // red they were waiting for their subject, and a scene waiting for its subject is not yet
    // evidence of anything. The design states both of the pane's axes in its own words — *a landing is a
    // shrink*, from another thread for the first time, and the four offset spellings that follow
    // from the offset belonging to neither side.
    ("file_preview_pane", Axis::Shrunk),
    ("file_preview_pane", Axis::Scrolled),
    ("file_picker", Axis::Scrolled),
    // Scene 28 — an earlier pass's narrow axis over the dense screen, at 300x80 and at 120x40. Not a
    // row of the design: the first two scenes of that table are the dense screen and its twin and neither
    // names an axis, and the defect this one is about — *a label that runs into its sibling's
    // rectangle* — is the third re-damage instance rather than the design's.
    ("text", Axis::Narrow),
    ("chip", Axis::Narrow),
    // Scene 33 — an earlier pass's, and the pair the design had no way to state. Its wheel row is one
    // component and one axis, which is exactly what an arithmetic click could express: a delta
    // added to an offset has no second axis to be wrong on. The posted notch found the pair.
    //
    // **Its position is scene order and not importance.** `crate::scenes::axis_scenes` derives this
    // list from the scene list, and the test that compares the two is an equality over ordered
    // vectors — so a pair written in the place a reader would put it is a failing test rather than a
    // tidy constant.
    ("scroll_area", Axis::Wheeled),
    // Scenes 34, 35 and 36 — a later pass's, and the three axes the design stated over one row.
    //
    // **The design carries one `field` row and it is the narrow one.** The freeze declares all four for
    // this component and three had no scene at all, which is the shape the entry above
    // describes one component over: a table written while a defect was not yet expressible states
    // the axes it could state. Here nothing was unexpressible — the pairs were simply never
    // scheduled, which is the production backlog's whole argument.
    //
    // **Their position is scene order and not importance**, for the reason the entry gives:
    // `crate::scenes::axis_scenes` derives this list from the scene list and the test that compares
    // the two is an equality over ordered vectors.
    ("field", Axis::Scrolled),
    ("field", Axis::Shrunk),
    ("field", Axis::Wheeled),
    // Scenes 37 and 38 — a later pass's, and `table`'s last two axes.
    //
    // **The design carries one `table` row and it is scene 7**, which covers `scrolled` and `narrow` in
    // one screen. The other two were expressible from the day an earlier pass declared the component
    // — the shrink is `collection`'s tail and the wheel is `collection`'s reveal, both reached by
    // calling it — and nothing had scheduled them, which is the entry above one
    // component over.
    //
    // **They are two files where the three were one**, and the two entries here are
    // adjacent for the same reason the original's is where it is: this list is scene order, and
    // `crate::scenes::axis_scenes` derives it from the scene list.
    ("table", Axis::Shrunk),
    ("table", Axis::Wheeled),
    // Scenes 39 to 42 — a later pass's, and they are the overlay family's last four.
    //
    // **The design carries one overlay-family row and it is scene 14**, whose `covers` is empty on purpose:
    // that screen measures what a layer costs the frame and plays no wheel and no scroll, and
    // `crate::popup` says so beside the assertion that `select` declares two of the four axes.
    // So all four pairs here were expressible from the day both components were declared — both
    // bodies draw the collection and both specs say *reached by calling it* — and nothing had
    // scheduled the question, which is the original's and 06's entries above for the third time.
    //
    // **All four are one file** where 06's two were two, and the reason is that one thing decides
    // all four: the list is behind a **layer**, which changes who can see the arithmetic rather
    // than the arithmetic. See `crate::dropped`.
    //
    // **Their position is scene order and not importance**, for the entry's reason.
    ("select", Axis::Scrolled),
    ("select", Axis::Wheeled),
    ("file_picker", Axis::Shrunk),
    ("file_picker", Axis::Wheeled),
    // Scenes 43, 44 and 45 — a later pass's, and they are the scroll family's last three.
    //
    // **The design carries two rows for this family and neither is a shrink or a band**: scene 17 is the
    // bar fixpoint and scene 18 is `Σ h` as the extent, and scene 33 is an earlier pass's wheel over
    // an area. So these three pairs had no scene at all — and unlike every earlier group on this
    // list, **all three had an instrument that could not fail on its own axis**: the tail's was one
    // frame over a state nothing had moved, the band's was played at offset `(0, 0)` where four of
    // the five spellings draw the same screen, and the pane's did not exist because the wheel gate
    // had three subjects and the pane was not one of them.
    //
    // **Two files and three scenes**, which is the rule rather than a new one: a wheel
    // is a posted notch and belongs where the drive loop is — `crate::wheel::Subject::Pane`, the
    // fourth arm — because a fourth copy of that loop would be the substitution an earlier pass spent
    // a ticket removing.
    //
    // **Their position is scene order and not importance**, for the entry's reason.
    ("scroll_area", Axis::Shrunk),
    ("sticky", Axis::Scrolled),
    ("file_preview_pane", Axis::Wheeled),
    // Scenes 46 and 47 — a later pass's, and they are `tree`'s last two and this list's last two.
    //
    // **The design carries two rows for this component and neither is a width or a wheel**: scene 8 is the
    // flatten index at depth 59 999 and scene 9 is a fold. So these two pairs had no scene — and
    // unlike every other group on this list they were **blocked** rather than merely unscheduled:
    // a components decision was open on whether `tree` draws the three indent guides its
    // freeze row declared, and *a guide column takes cells from the label*. A narrow scene written
    // before that answer would have been a scene of a different screen, which is the only edge on
    // this backlog that a scenes ticket could not have argued its way past.
    //
    // 20 resolved by taking the three glyphs off the row, so the partition scene 46 reads is the
    // one the design always described: one `Ink::run` of spaces, one chevron cell and the label taking the
    // rest.
    //
    // **Two files and two scenes**, the rule: a screen is `crate::forest`, where this
    // component's other two already live, and a wheel is `crate::wheel::Subject::Tree`.
    //
    // **Their position is scene order and not importance**, for the entry's reason.
    ("tree", Axis::Narrow),
    ("tree", Axis::Wheeled),
];

/// **The ids whose data-volume cost has been measured and answers both of O6's bounds.** O6's
/// evidence.
///
/// Seven. Written out for [`DOC_TESTED`]'s reason: a `const fn`
/// over [`crate::volume::COVERED`] would make the population and the evidence one expression, and
/// an equality between two things derived from each other holds. What holds it honest is
/// [`crate::volume::met`], which **runs the shipped component** at ten thousand, a hundred thousand
/// and a million inputs and answers with a step count;
/// `crate::volume::tests::every_covered_row_holds_both_bounds_at_a_million` is the comparison.
///
/// The order is [`crate::volume::population`]'s, which is [`INVENTORY`]'s.
pub const VOLUME_MEASURED: &[&str] = &[
    "field",
    "collection",
    "table",
    "tree",
    "chart",
    "plot",
    "sparkline",
];

/// **O1 — a rustdoc page with a compiled example, for every component.**
///
/// The count is *components with 0 doc-tests == 0*. O1 and O2 do not substitute for each other: O1
/// catches an API that cannot be called from outside the crate, O2 catches an inventory that has
/// drifted from what ships, and **neither catches a wrong cell** — that is O3 and O5.
///
/// # The population is `built`, and that is a finding rather than a convenience
///
/// The second equality is *everything **`built`** must have a panel*, and the first states
/// count with no population at all — *components with 0 doc-tests == 0* — so the reading is owed
/// rather than given. **It is the same population, for the same reason.** `spinner` is the one row
/// of the twenty-nine that no ticket has built: its mechanism is *a component that owns a clock*,
/// it is still out of scope, and somebody else's to prototype. A doc page for a function that
/// does not exist is not a page anybody can write, and asking for one puts a permanent row in the
/// failing set that **no ticket on this backlog can invert** — which turns a query that is
/// measuring something into a query that always reads red and is therefore never read.
///
/// The cost of getting it the other way round is the one this file exists to refuse, in mirror
/// image: a query stuck red is as uninformative as a query vacuously green, and both of them stop
/// being evidence. What keeps this one honest is that **the population moves**: the day `spinner`
/// ships, [`crate::doc::pages`] returns twenty-nine and this query asks about twenty-nine, with no
/// edit here.
pub fn o1(doc_tested: &[&str]) -> Verdict {
    let built: Vec<&Component> = INVENTORY.iter().filter(|c| c.built).collect();
    let failing = built.iter().filter(|c| !doc_tested.contains(&c.id)).count();
    Verdict::of(
        built.len(),
        failing,
        "built components carry no compiled doc example, so nothing proves their API is callable \
         from outside this crate",
    )
}

/// **O2, first equality — nothing shown is absent from the freeze.**
///
/// This is the query that would be vacuously true over an empty gallery, and [`Verdict::of`] is
/// what stops it: the population is the panel list, so an empty gallery is `Unmet` over zero rather
/// than `Met` over zero.
pub fn o2_nothing_shown_is_absent_from_the_freeze(panels: &[&str]) -> Verdict {
    let failing = panels
        .iter()
        .filter(|p| !INVENTORY.iter().any(|c| c.id == **p))
        .count();
    Verdict::of(
        panels.len(),
        failing,
        "the gallery does not exist, so there is no screen for the freeze to have drifted from — \
         which is the vacuous green this equality is written to refuse",
    )
}

/// **O2, second equality — everything `built` has a panel.**
///
/// Over `built` rather than over all twenty-nine, because ten entries have nothing to show. ADR
/// 0033 records that finding this out **grew the gallery from ten panels to twelve**.
pub fn o2_everything_built_has_a_panel(panels: &[&str]) -> Verdict {
    let built: Vec<&Component> = INVENTORY.iter().filter(|c| c.built).collect();
    let failing = built.iter().filter(|c| !panels.contains(&c.id)).count();
    Verdict::of(
        built.len(),
        failing,
        "built components have no gallery panel, so nothing would notice the inventory drifting \
         from what ships",
    )
}

/// **O3 — one golden screen per construction.**
///
/// A count, `goldens == constructions`, with the equality beside it — screens declared identical
/// must be identical — in `crates/vitui-components/tests/golden.rs`, because that half compares two
/// pictures and this one counts.
///
/// # The population is `built`, and it is [`o1`]'s finding a second time
///
/// The second equality is over `built` and the count is over nothing at all, so the
/// reading was owed here as it was there. **A golden for a function that does not exist is not a
/// screen anybody can draw**: `spinner` is the one unbuilt row of the freeze — *a component that
/// owns a clock*, which is components 42's prototype and no ticket on this backlog ships — so
/// asking for its screen would put a permanent row in the failing set that nothing can invert.
/// *A query stuck red is [`Verdict::of`]'s vacuity failure in mirror image*: it reads red whatever
/// happens, so nobody reads it.
///
/// The population **moves**, so the day `spinner` ships the query asks about twenty-nine rows and
/// thirty-four screens with no edit here.
///
/// The sum over the twenty-eight built rows is **33**: twenty-four at one construction each, plus
/// two for `chart`, two for `meter`, two for `sparkline` and three for `plot`.
pub fn o3(goldens: &[(&str, u8)]) -> Verdict {
    let built: Vec<&Component> = INVENTORY.iter().filter(|c| c.built).collect();
    let failing = built
        .iter()
        .filter(|c| {
            let have: u8 = goldens
                .iter()
                .filter(|(id, _)| *id == c.id)
                .map(|(_, n)| *n)
                .sum();
            have != c.constructions
        })
        .count();
    Verdict::of(
        built.len(),
        failing,
        "components have a golden-screen count that is not their construction count, so a rung \
         builds something different and no screen says what",
    )
}

/// **O4 — the declared keyboard contract, rendered as help.**
///
/// An equality: documented == registered, with two checks that keep it honest — the
/// walkthrough (*the walk repeats no id, and reaches every stop unless a trap is standing*) and *a
/// chord pressed into every focusable types nothing* — and both need a form to run over.
///
/// # The population is the evidence, and finding that out cost a green test
///
/// Written over [`INVENTORY`] — twenty-nine rows, failing when the two lists disagree about one —
/// **O4 returns `Met` today**, and it was the only one of the six that did. Two empty lists agree
/// about every component: `false != false` is false, so nothing fails, and twenty-nine is not zero
/// so the constructor's vacuity refusal never fires. *An equality between two things that do not
/// exist holds.*
///
/// That is the first refinement arriving from a direction it did not name — **a threshold on the
/// wrong side of the question is not a weak gate, it is a green one** — and it is the same shape as
/// the gallery gate that asserted `changed > 0` while 3 583 cells of 4 800 carried the old palette.
/// So the population here is the **union of the two lists**: the components that have declared
/// anything at all. Empty evidence is `over == 0`, which [`Verdict::of`] refuses.
///
/// It is also the faithful reading. Some rows have no keyboard contract to declare — `text`,
/// `rule` and `panel` are pure drawers — so *every component appears in both lists* is not the
/// obligation; the obligation is that the two agree wherever either speaks. Whether a row that
/// declares nothing is a defect is O1's and O2's question, over populations that really are all
/// twenty-nine.
pub fn o4(documented: &[&str], registered: &[&str]) -> Verdict {
    let speaks: Vec<&str> = INVENTORY
        .iter()
        .map(|c| c.id)
        .filter(|id| documented.contains(id) || registered.contains(id))
        .collect();
    let failing = speaks
        .iter()
        .filter(|id| documented.contains(id) != registered.contains(id))
        .count();
    Verdict::of(
        speaks.len(),
        failing,
        "no component has declared a keyboard contract in either direction, so the equality has \
         nothing to be about — and a binding registered and undocumented is a feature nobody \
         finds, while one documented and unregistered is a help bar that lies",
    )
}

/// **O5 — at least one scene per hostile axis a component declares.**
///
/// > **O5 is worth more than the other four together.**
///
/// The population is *(component, axis) pairs where the flag is set* — thirty-four of them — and
/// **not** the scene list. Written the other way round it is a loop over scenes, which over an
/// empty scene list is vacuously true and is precisely the defect shape the four axes exist to
/// catch: each was established by a build that passed every gate then in force and looked
/// *healthier*.
pub fn o5(scenes: &[(&str, Axis)]) -> Verdict {
    let mut over = 0usize;
    let mut failing = 0usize;
    for c in INVENTORY {
        for axis in Axis::ALL {
            if !c.declares(axis) {
                continue;
            }
            over += 1;
            if !scenes.iter().any(|(id, a)| *id == c.id && *a == axis) {
                failing += 1;
            }
        }
    }
    Verdict::of(
        over,
        failing,
        "declared hostile axes have no scene, and three of the four axes were caught only by an \
         equality against a reference render",
    )
}

/// **O6 — every component that takes a data volume holds sixty hertz at a million inputs.**
///
/// The sixth obligation, stated after the map closed and filed here rather
/// than in an architecture issue because the instrument is buildable without reopening anything.
/// [`crate::volume`] is that instrument and this is the query.
///
/// # The population is derived and it is not `built`
///
/// [`o1`] and [`o3`] are over `built` because a page or a screen for a function that does not exist
/// is not a thing anybody can write. This one is over *rows that take a volume*, read off
/// [`INVENTORY`]'s [`Layer::L2`](crate::inventory::Layer::L2) column and off the memo census —
/// and every one of the seven it reaches happens to be built, so the two readings do not disagree
/// today. If they ever do, this population is the right one anyway: `Layer::L2` is a claim about a
/// component's **construction**, and a row that has declared it and not shipped it has made a claim
/// nothing is measuring.
///
/// # What "holds" means, and why it is two numbers
///
/// A growth relation and a per-input ceiling, both counts. **The defect this obligation exists for
/// passes the relation** — the whole column prefix painted once a point is `O(subh)` inside a visit,
/// which is linear with `subh` in front of it — and **a ceiling alone is met by any constant chosen
/// large enough**, which is the first refinement by name. `crate::volume` carries an arm for each
/// half and `crate::volume::tests::neither_criterion_would_do_on_its_own` is where that is measured
/// rather than argued.
pub fn o6(measured: &[&str]) -> Verdict {
    let takes_a_volume = crate::volume::population();
    let failing = takes_a_volume
        .iter()
        .filter(|id| !measured.contains(*id))
        .count();
    Verdict::of(
        takes_a_volume.len(),
        failing,
        "components that take a data volume have no measured cost, so this crate can prove a \
         frame's output is flat in the volume and not its work — which is how a fold at 476 ns a \
         point shipped past every gate here",
    )
}

/// **The ids at least one application in `crates/vitui-apps/examples/` exercises.** O7's evidence.
///
/// Twenty-eight. Written out for [`DOC_TESTED`]'s reason: a
/// `const fn` over [`crate::consumer::applied`] would make the population and the evidence one
/// expression, and an equality between two things derived from each other holds. What holds it
/// honest is [`crate::consumer::coverage`], which opens every application in the directory and
/// joins its **import paths** against the spellings each row's home module declares;
/// `tests::the_written_list_and_the_scan_agree_about_o7` is the comparison.
///
/// The order is [`INVENTORY`]'s, so a row added to the freeze in the middle is a failing test here
/// rather than a tidy append. `spinner` is absent, and that is [`o7_everything_declared_has_an_application`]'s
/// population rather than an omission here.
pub const APPLIED: &[&str] = &[
    "text",
    "panel",
    "chip",
    "button",
    "field",
    "collection",
    "table",
    "tree",
    "select",
    "overlay",
    "scroll_area",
    "scrollbar",
    "sticky",
    "collapsible",
    "chart",
    "plot",
    "checkbox",
    "radio",
    "switch",
    "meter",
    "sparkline",
    "rule",
    "status_bar",
    "pagination",
    "form",
    "slider",
    "spinner",
    "file_picker",
    "file_preview_pane",
];

/// **O7, first equality — nothing an application exercises is absent from the freeze.**
///
/// [`o2_nothing_shown_is_absent_from_the_freeze`]'s direction, one instrument over, and it is here
/// for that half's reason: it is the equality that catches the **inventory** drifting from what
/// ships, which the second half cannot see at all. The population is the evidence, so an empty
/// [`APPLIED`] is `Unmet` over zero rather than `Met` over zero.
pub fn o7_nothing_exercised_is_absent_from_the_freeze(applied: &[&str]) -> Verdict {
    let failing = applied
        .iter()
        .filter(|id| !INVENTORY.iter().any(|c| c.id == **id))
        .count();
    Verdict::of(
        applied.len(),
        failing,
        "no application exercises anything the freeze has heard of, so there is nothing for the \
         inventory to have drifted from — which is the vacuous green this equality is written to \
         refuse",
    )
}

/// **O7, second equality — every component this crate declares is exercised by an application.**
///
/// > **A gate exercises the component where its author put it, and an application puts it somewhere
/// > else.**
///
/// Four defects argue it and three of the four were found by a person running the thing:
/// `counter` found that no loop could be written at all and that nothing holds the focus until an
/// application says so, `latency` found `chart`'s rasteriser painting the whole column prefix for
/// every point — 952.61 ms against 2.72 at a million — and `ledger` found a table drawing its header
/// one column into the border. Every gate in this crate was green on all four.
///
/// # The population is `declared` and not `built`, and that is the finding
///
/// [`o1`] and [`o3`] are over `built`, because a page or a screen for a function that does not exist
/// is not a thing anybody can write, and the reading is the same one here — with the column
/// replaced by the source it claims. **`built` is a claim**: `slider`'s
/// row reading `built: true` for two tickets with no `slider` anywhere in the crate, and the two
/// joins that look as though they should have caught it were each blind for a stated reason. So the
/// population arrives as a slice from [`crate::consumer::declared`], which opens each row's home
/// module and looks for the declaration.
///
/// It **moves**: the day `spinner` ships, the population is twenty-nine and this query asks about
/// twenty-nine with no edit here.
pub fn o7_everything_declared_has_an_application(declared: &[&str], applied: &[&str]) -> Verdict {
    let failing = declared.iter().filter(|id| !applied.contains(*id)).count();
    Verdict::of(
        declared.len(),
        failing,
        "components this crate declares are in no application, so nothing exercises them anywhere \
         but where their own author put them — which is how four defects every gate here was green \
         on reached a person running the thing",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// A source, by a path relative to the workspace root. [`crate::doc`]'s and
    /// [`crate::consumer`]'s arrangement: the scan takes the reader, so its hostile arms are one
    /// call away.
    fn read(relative: &str) -> String {
        let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../..")).join(relative);
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
    }

    /// The applications on disk, in name order. See [`crate::consumer`].
    fn applications() -> Vec<String> {
        let dir = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
            .join(crate::consumer::EXAMPLES);
        let mut out: Vec<String> = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
            .map(|entry| entry.expect("a readable entry").path())
            .filter(|p| p.extension().is_some_and(|e| e == "rs"))
            .map(|p| {
                p.file_stem()
                    .expect("a .rs file has a stem")
                    .to_str()
                    .expect("a utf-8 file name")
                    .to_owned()
            })
            .collect();
        out.sort();
        out
    }

    /// **The written-out evidence and the scan are two sources, and this is the comparison.**
    ///
    /// `crate::doc::tests::the_written_list_and_the_scan_agree` for O1, and
    /// `crate::gallery::tests::the_written_list_and_the_table_agree` for O2. Here the second source
    /// is [`crate::consumer::applied`], which opens every application in the directory and joins its
    /// import paths against the spellings each row's home module declares — so a component that
    /// stops being drawn anywhere is a failing test rather than a list nobody edited.
    #[test]
    fn the_written_list_and_the_scan_agree_about_o7() {
        assert_eq!(
            APPLIED.to_vec(),
            crate::consumer::applied(read, &applications()),
            "`APPLIED` and the scan disagree about which components an application exercises. \
             Neither is the authority on its own: the scan is what moves when an application is \
             written, and the list is what a reader audits"
        );
    }

    /// **The subjects a posted wheel notch is played over are counted here and nowhere else.**
    /// The second half of criterion 6.
    ///
    /// `crate::wheel` asserts the freeze *declares* the axis for both subjects it plays over; this
    /// asserts the other direction, which is the one that can go quietly wrong: **O5 holds a pair
    /// for each of them**. The two are not the same question and neither implies the other — a
    /// subject the freeze declares and no scene claims is an axis with no evidence, and a pair
    /// claimed for a component nothing plays a wheel over is evidence for nothing.
    ///
    /// It is written over [`AXIS_SCENES`] rather than over the scene list because that is the list
    /// [`o5`] actually reads. `crate::scenes` already gates the two against each other, so a pair
    /// present here and absent from a scene fails there instead of silently passing both.
    ///
    /// # The pane and the tree joined `Subject::ALL` rather than a fourth source, and that is the
    /// shape holding twice
    ///
    /// The third scene is a **fourth arm of `crate::wheel`'s own drive loop**, not a
    /// screen of its own — a pane's notch has an area's cadence exactly, because the pane hands its
    /// rectangle to `scroll_area`. The second is the **fifth**, for the same reason one
    /// family over: a tree's notch has a collection's cadence exactly, because `tree` hands its
    /// rectangle to `collection_shaped`. So the join below is still over three sources and
    /// [`crate::wheel::Subject::ALL`] is at **five**. **The rule is: a new cadence is a new source
    /// and a new subject of an existing cadence is not** — the two screens that post
    /// no notch at all, and neither does the narrow one.
    ///
    /// # The population is two gates' subject lists, not one
    ///
    /// This read `Subject::ALL` alone for a long time, and then a wheel
    /// scene arrived in a **second** file — [`crate::window`]'s, twenty posted notches over a
    /// `field`, whose notch path is the component consuming `Response::scrolled` itself rather than
    /// `crate::wheel`'s two-subject drive loop. A **third** followed:
    /// [`crate::dropped`]'s scenes 40 and 42, over the two overlay owners, whose cadence is two
    /// opening frames rather than one because a layer's entries are only in the hit index a notch is
    /// resolved against once the layer has been placed. So the population is
    /// [`crate::wheel::Subject::ALL`] plus [`crate::window::WHEELED_SUBJECTS`] plus
    /// [`crate::dropped::WHEELED_SUBJECTS`], **every one derived from the gate that plays them**,
    /// which is what the sentence below is about: written out by hand on either side, a further
    /// subject escapes every half while all of them stay green.
    ///
    /// **Three sources is the point at which the shape stops being an exception**: a wheel gate is
    /// not one drive loop, because a notch's cadence is a property of *what consumes it* — a
    /// component reading `Response::scrolled` in its own draw, a drive loop over a base-pass
    /// subject, and a body inside a layer are three cadences and no one loop has all three.
    ///
    /// # The equality is over **sorted** lists, and it stopped being over ordered ones in
    /// a later pass
    ///
    /// The two sides carry two different orders and neither is wrong. [`AXIS_SCENES`] is **scene
    /// order** — that is what [`crate::scenes::axis_scenes`] derives and what the ordered
    /// comparison one module over is written on — and `Subject::ALL` is the order the drive loop
    /// enumerates its subjects in. They coincided while the wheel gate's subjects happened to be
    /// the first wheel scenes written; `table` was added to a gate whose subjects predate
    /// `field`'s scene and whose scenes follow it, and the two orders parted.
    ///
    /// **What this test is about is a population and not an order**, so sorting both sides is the
    /// question asked exactly rather than a weakened one: an element on one side and not the other
    /// still fails, and a coincidence of two unrelated orderings is no longer load-bearing.
    #[test]
    fn o5_counts_a_pair_for_every_axis_a_posted_notch_is_played_over() {
        use crate::wheel::Subject;

        let mut played: Vec<&str> = Subject::ALL
            .iter()
            .map(|s| s.id())
            .chain(crate::window::WHEELED_SUBJECTS.iter().copied())
            .chain(crate::dropped::WHEELED_SUBJECTS.iter().copied())
            .collect();
        played.sort_unstable();
        for id in &played {
            assert!(
                AXIS_SCENES
                    .iter()
                    .any(|(c, axis)| c == id && *axis == Axis::Wheeled),
                "a posted notch is played over `{id}` and O5 holds no `(component, wheeled)` pair \
                 for it, so the axis has a gate and no evidence"
            );
        }
        // And the join is exactly those subject lists: a further pair here would be an axis
        // claimed for a component nothing plays a wheel over.
        let mut wheeled: Vec<&str> = AXIS_SCENES
            .iter()
            .filter(|(_, axis)| *axis == Axis::Wheeled)
            .map(|(id, _)| *id)
            .collect();
        wheeled.sort_unstable();
        assert_eq!(
            wheeled, played,
            "the pairs and the two gates' subject lists are the same population, derived from the \
             gates that play them — written out by hand on each side, a third subject escapes both \
             halves while both stay green. Sorted, because the two sides carry two different orders \
             and the question is a population"
        );
    }

    /// **Seven of the seven are met, and the number is written down.**
    ///
    /// The runtime `register.rs`'s arrangement, one crate up: a list that says how many are green
    /// makes the next change a deliberate edit rather than a quiet one. Today the answer is **seven
    /// of seven** obligations — nine queries, because O2 and O7 are two equalities each — and the
    /// list below is written out rather than counted, so that an obligation going *red* is a
    /// deliberate edit too. No query is unmet, which is what this test existed to make impossible
    /// to reach quietly.
    ///
    /// **O1 was the first**, and the deliberate edit this test was written
    /// to force: it stood at zero for thirty-five tickets, twenty-five of which shipped a component
    /// entitled to add a row to [`DOC_TESTED`] and none of which did. **O3 is the second**, and
    /// its edit is the golden screens — thirty-three over the twenty-eight built rows,
    /// with the equalities in `crates/vitui-components/tests/golden.rs`. **O4 is the third**, and
    /// its edit is the keyboard contracts — thirteen of them, and the equality that matters is one
    /// level down in `crate::contract`, chord for chord against a sweep that runs the component.
    /// **O2 is the fourth, in both halves at once**, and its edit is the gallery — the
    /// twenty-eight panels of `crate::gallery`, whose table the application iterates and whose ids
    /// [`crate::gallery::panel_ids`] derives so that the two lists are two sources rather than one
    /// read twice. **O6 is the fifth**, and its edit is the volume screens — seven rows that take
    /// a data volume, each measured at a million inputs by an instrument that runs the shipped
    /// component and counts its steps. **O5 was the last and it is the one worth more than the
    /// other four together** — production tickets 05 to 09 took the fourteen `(component, axis)`
    /// pairs that had no scene, and the `tree` pair was the one that emptied the
    /// failing set. Nothing here turned it: [`o5`] is a query and its evidence is
    /// [`AXIS_SCENES`], so the edit that turned it was a scene.
    #[test]
    // **Renamed by a later pass, from
    // `eight_of_the_nine_obligation_queries_are_met_and_the_one_left_is_o5`.** a later pass
    // emptied O5's failing set and left the name stale on purpose, carrying the note the recorded
    // policy requires — *a stale name that carries a note saying so stays; one that does not is
    // corrected* — because a name is also how a reader and its register citations find a row, and
    // 07 edited here only what a green build cannot be left without: the `met` vector below, the
    // `Met` assertion, and `o5_fails_loudly`'s arm. This ticket is what the note deferred to, and
    // the rename is not free: it is this line, the module header above, the two register citations
    // in `crate::gates`, `CLAUDE.md`'s sentence and the components README. The other lineage on
    // this file chose the other arm and kept its name — see
    // `crate::scenes::tests::seventeen_of_the_thirty_four_axis_obligations_have_a_scene_and_seventeen_do_not`,
    // stale through five moves and still carrying its note.
    //
    // **What separates the two arms is not the citation count, and a review of this ticket's first
    // draft is what established that.** That draft said *renamed because it had two citations where
    // the other has five*, and the measurement runs the other way: this name has **two** register
    // rows and the kept one has **one**. The five belong to a third test,
    // `two_scenes_have_nothing_to_run_over_…`, which is the one that *was* renamed — three times —
    // so the five was never the kept name's price. What buys the keep is `PINS_SCENE_22`'s stated
    // reason, *`crate::gates`'s row and that list are read together and a rename would make the two
    // look like two different scenes*, plus the note the policy requires. The rename here is bought
    // by neither: 07's note named this ticket, so the deferral was already priced.
    //
    // **The name is a summary sentence and the assertion is the authority.** Nine of nine is
    // asserted below whatever this line says, which is what makes a stale name a wrong pointer
    // rather than a wrong gate.
    fn all_nine_obligation_queries_are_met_and_o5_was_the_last_to_turn() {
        let declared = crate::consumer::declared(read);
        let all = [
            ("O1", o1(DOC_TESTED)),
            ("O2a", o2_nothing_shown_is_absent_from_the_freeze(PANELS)),
            ("O2b", o2_everything_built_has_a_panel(PANELS)),
            ("O3", o3(GOLDENS)),
            ("O4", o4(KEYBOARD_DOCUMENTED, KEYBOARD_REGISTERED)),
            ("O5", o5(AXIS_SCENES)),
            ("O6", o6(VOLUME_MEASURED)),
            (
                "O7a",
                o7_nothing_exercised_is_absent_from_the_freeze(APPLIED),
            ),
            (
                "O7b",
                o7_everything_declared_has_an_application(&declared, APPLIED),
            ),
        ];
        let met: Vec<&str> = all
            .iter()
            .filter(|(_, v)| v.met())
            .map(|(n, _)| *n)
            .collect();
        assert_eq!(
            met,
            vec!["O1", "O2a", "O2b", "O3", "O4", "O5", "O6", "O7a", "O7b"],
            "an obligation has changed colour. That is the point of the backlog and it is also a \
             deliberate edit to this test and to this module's header — the number is here so a \
             green one cannot arrive unremarked"
        );

        for (name, verdict) in all.into_iter().filter(|(_, v)| !v.met()) {
            let Verdict::Unmet { over, failing, why } = verdict else {
                unreachable!("checked above");
            };
            assert!(why.len() > 40, "{name} does not say what is wrong");
            assert!(failing <= over);
        }
    }

    /// **The exact failing sets, so that a change in either direction is visible.**
    ///
    /// The rule for a pinned red gate: *it asserts its exact failing set, fires in both
    /// directions, and says what to invert when it is fixed*. Twenty-nine components, nineteen
    /// built, thirty-four axis obligations and thirty-four goldens owed.
    #[test]
    fn each_query_reports_the_population_it_could_not_answer_for() {
        // **The `unmet` reader is gone, and its absence is the record.** It unwrapped a
        // `Verdict::Unmet` into `(over, failing)` and had exactly one caller left — O5's — because
        // every other query on this list had already turned. A later pass turned that one, so
        // every assertion below is now *asserted from the other side*, which is what O1's comment
        // has said since it turned: a met query has no failing set to report. Keeping the closure
        // would leave a `warnings = "deny"` build failure or an `#[expect]` for a reader nothing
        // reads.
        // **O1 is `Met` over the twenty-eight built rows**, so it has no failing set to report and
        // is asserted from the other side. See `o1` for why the population is `built` and not all
        // twenty-nine.
        assert_eq!(o1(DOC_TESTED), Verdict::Met { over: 29 }, "O1");
        assert_eq!(DOC_TESTED.len(), 29);
        // **The two halves of O2 differ in population, and that is exactly the point that
        // they do not substitute for each other** — so both are asserted from the other side now,
        // and the two numbers are different on purpose. The first is over the **gallery** and reads
        // 28 because that is how many panels there are; the second is over the twenty-eight
        // **built rows** and reads 28 because every one of them has a panel. The day `spinner`
        // ships they are 28 and 29 until its panel arrives, which is the drift this pair is for.
        assert_eq!(
            o2_nothing_shown_is_absent_from_the_freeze(PANELS),
            Verdict::Met { over: 29 },
            "O2a"
        );
        assert_eq!(
            o2_everything_built_has_a_panel(PANELS),
            Verdict::Met { over: 29 },
            "O2b"
        );
        assert_eq!(PANELS.len(), 29);
        // **O3 is `Met` over the same twenty-eight built rows**, so it is asserted from the other
        // side too. See `o3` for why the population is `built`: `spinner` has no component to draw,
        // and a row nothing on this backlog can invert is a row that reads red for ever.
        assert_eq!(o3(GOLDENS), Verdict::Met { over: 29 }, "O3");
        // **O4's population was 0 for thirty-seven tickets, and that was a finding rather than an
        // oversight.** Written over `INVENTORY` it returned `Met` over twenty-nine, because two
        // empty lists agree about every row — the one query of the six that read green, and the
        // reason this test exists at all. It is `Met` over **thirteen** now, which is the union of
        // the two lists and not the freeze: sixteen rows read no key.
        assert_eq!(
            o4(KEYBOARD_DOCUMENTED, KEYBOARD_REGISTERED),
            Verdict::Met { over: 13 },
            "O4"
        );
        // **O5 has moved five times and it is still red.** The scene list covered twelve
        // of the thirty-four `(component, axis)` pairs from the rows, the narrow axis
        // added `text` and `chip`, and an earlier pass added `(scroll_area, wheeled)` — the pair the design had
        // no way to state, because its single wheel row was written while a click was an arithmetic
        // substitution and a delta added to an offset has no second axis to be wrong on. Components
        // 32 took three: the preview pane's shrink and scroll and the picker's scroll, which the
        // three scenes had left empty on purpose while they were red. **A later pass took
        // `field`'s last three** — scrolled, shrunk and wheeled, none of which was ever
        // unexpressible and none of which anything had scheduled. 06 took `table`'s two, 08 the
        // overlay family's four, and **09 the scroll family's three**, which are the first on this
        // list where all three pairs already had an instrument that *could not fail on its own
        // axis*. And **A later pass took the last two, which were `tree`'s** — the only pair on
        // the list whose edge was a *decision* rather than an instrument, because components
        // architecture 20 was open on whether the component draws the three indent guides its
        // freeze row declared. A query that moves is a query that is measuring something, and this
        // one has stopped moving because the population is exhausted rather than because nothing
        // is asking.
        //
        // **07 could not leave CI red to keep a boundary tidy**, so it wrote this count, the `met`
        // vector above and `o5_fails_loudly`'s expectation, and deferred the rest to a later pass
        // — the summary test's name, the two comments citing a test name that no longer exists,
        // `CLAUDE.md`'s sentence and the components README. **10 has taken them**, and the sentence
        // it replaced here was *Two are left and they are `tree`'s, blocked on components
        // architecture 20*, sitting directly above the line that contradicted it. A deferral note
        // is worth its space until the ticket it names lands; after that it is one more summary
        // sentence disagreeing with the assertion beside it.
        assert_eq!(o5(AXIS_SCENES), Verdict::Met { over: 34 }, "O5");
        assert_eq!(AXIS_SCENES.len(), 34);
        // **O6 is `Met` over the seven rows that take a volume**, so it is asserted from the other
        // side too. The population is derived rather than written out — the `Layer::L2` column plus
        // the rows that keep a memo keyed on a data revision — and it answered **seven** where
        // The parenthesis named six: `sparkline` is `chart`'s body with the chrome
        // deleted and folds the same million points through the same `Raster`. A query whose
        // population moves without an edit here is a query that is measuring something.
        assert_eq!(o6(VOLUME_MEASURED), Verdict::Met { over: 7 }, "O6");
        assert_eq!(VOLUME_MEASURED.len(), 7);
        // **O7 is two equalities and both are `Met`, so both are asserted from the other side** —
        // O2's arrangement, and for O2's reason: the first is over the **evidence** and reads 28
        // because that is how many rows an application exercises, the second is over the
        // **declared** rows and reads 28 because every one of them is in one. The day `spinner`
        // ships they are 28 and 29 until its application arrives, which is the drift this pair is
        // for — and unlike O1's and O3's, this population is read out of the source rather than off
        // the `built` column that lied for two tickets.
        let declared = crate::consumer::declared(read);
        assert_eq!(declared.len(), 29);
        assert_eq!(
            o7_nothing_exercised_is_absent_from_the_freeze(APPLIED),
            Verdict::Met { over: 29 },
            "O7a"
        );
        assert_eq!(
            o7_everything_declared_has_an_application(&declared, APPLIED),
            Verdict::Met { over: 29 },
            "O7b"
        );
        assert_eq!(APPLIED.len(), 29);

        // **The construction sum, both ways round.** Over the whole freeze it is 36 — 29 rows plus
        // `chart`, `meter` and `sparkline` at 2 and `plot` and `spinner` at 3 — and over the rows
        // that have a component to draw it is **the same 36**, which is what `GOLDENS` adds up to
        // and what `crate::golden::SCREENS` holds.
        //
        // **It was 34, then 35, and the row that moved both times is the last one built.** the design gave
        // `spinner` a 1 on the grounds that every spelling is one cell and no spelling is blank;
        // an earlier pass found both true of its ladder and neither deciding it and read 2 off the frame
        // counts; an earlier pass shipped the table and the derivation, and the answer is 3 — because a
        // count is not a ladder, and two rungs with four frames each can be two different tables.
        //
        // **Every one of them is derived now, and an earlier pass is where the last literal
        // went.** `chart`, `plot`, `meter` and `sparkline` each assert `row.constructions ==
        // distinct(...)` against a shipped table; `spinner`'s table was the prototype's and lived
        // on a branch, so nothing here would have noticed its ladder changing shape.
        // `crate::indicate::constructions` is that derivation, and running it moved the number:
        // **2 became 3**, because the ladder put the braille spinner at the `Unicode` rung
        // and the engine's own `GlyphSet` says braille is `Extended`. The middle rung is the
        // quadrant blocks, so the three ladders are three distinct tables.
        let owed: u32 = INVENTORY.iter().map(|c| u32::from(c.constructions)).sum();
        assert_eq!(owed, 36);
        let buildable: u32 = INVENTORY
            .iter()
            .filter(|c| c.built)
            .map(|c| u32::from(c.constructions))
            .sum();
        assert_eq!(buildable, 36);
        assert_eq!(GOLDENS.iter().map(|(_, n)| u32::from(*n)).sum::<u32>(), 36);
        // **The two sums are equal for the first time**, and that is what a complete freeze looks
        // like from here: every row is built, so nothing is owed that is not buildable. It was
        // 35 against 33 until an earlier pass.
        assert_eq!(owed, buildable);
    }

    /// **Vacuous truth is refused in the constructor, and this is where that is asserted.**
    ///
    /// Not a hypothetical: `o2_nothing_shown_is_absent_from_the_freeze` over an empty [`PANELS`]
    /// has **zero failures out of zero**, and the arithmetic that calls that success is the same
    /// arithmetic that reported `allocs / n == 0` for a frame allocating on n−1 of n frames.
    #[test]
    fn an_obligation_asked_about_nothing_is_unmet_and_not_met() {
        assert!(!Verdict::of(0, 0, "nothing to check, which is the failure").met());
        assert!(Verdict::of(1, 0, "-").met());
        assert!(!Verdict::of(1, 1, "-").met());
        assert!(!o2_nothing_shown_is_absent_from_the_freeze(&[]).met());
        // And with something in it, the same query answers a real question rather than a vacuous
        // one — in both directions.
        assert!(o2_nothing_shown_is_absent_from_the_freeze(&["chart", "plot"]).met());
        assert!(!o2_nothing_shown_is_absent_from_the_freeze(&["gauge"]).met());
        // **The arm that caught O4.** An equality between two empty lists is not an answer, and
        // over a population of twenty-nine it reads as one. With evidence on one side only it is a
        // real disagreement; with evidence agreeing on both it is a real agreement.
        assert!(!o4(&[], &[]).met());
        assert!(!o4(&["field"], &[]).met());
        assert!(!o4(&[], &["field"]).met());
        assert!(o4(&["field"], &["field"]).met());
        // **O6's vacuity arm is the one that cannot happen from this side**, and it is asserted
        // anyway: its population is derived from the freeze, so an empty one would mean no row of
        // the freeze takes a data volume at all. What *can* happen is evidence going missing, and
        // that is the arm below and `o6_fails_loudly`.
        assert!(!o6(&[]).met());
        assert!(o6(VOLUME_MEASURED).met());
        // **O7's two, and the first is the arm that needs the constructor.** *Nothing an
        // application exercises is absent from the freeze* over an empty evidence list is zero
        // failures out of zero, which is the same arithmetic that reported `allocs / n == 0` for a
        // frame allocating on n−1 of n frames.
        assert!(!o7_nothing_exercised_is_absent_from_the_freeze(&[]).met());
        assert!(o7_nothing_exercised_is_absent_from_the_freeze(&["text", "panel"]).met());
        assert!(!o7_nothing_exercised_is_absent_from_the_freeze(&["gauge"]).met());
        assert!(!o7_everything_declared_has_an_application(&[], &[]).met());
        assert!(!o7_everything_declared_has_an_application(&["text"], &[]).met());
        assert!(o7_everything_declared_has_an_application(&["text"], &["text"]).met());
    }

    /// **The five, each watched failing.** A gate nobody has watched fail is not a gate.
    ///
    /// That is three for three: a `cargo test --workspace` step once
    /// had never passed, R15 found a `clippy` step that had never passed under its own
    /// `-D warnings`, and C11 found a `cargo deny` job that had never passed at all. The
    /// corresponding mistake for a query is a `Verdict` nobody ever asserts on, which reads as a
    /// gate and is a value.
    ///
    /// **O1 is the one that has turned, so it is watched failing over a list with a row removed**
    /// rather than over the shipped one. That is the arm that matters now: the query has to still
    /// notice a page that stops carrying an example, and a `Met` verdict cannot be watched
    /// panicking.
    #[test]
    #[should_panic(expected = "O1 is unmet: 1 of 29")]
    fn o1_fails_loudly() {
        let one_short: Vec<&str> = DOC_TESTED.iter().copied().skip(1).collect();
        o1(&one_short).assert_met("O1");
    }

    /// See [`o1_fails_loudly`]. **O2's first half has turned, so it is watched failing over the
    /// shipped list with a panel added that the freeze has never heard of** — which is the drift
    /// this half exists to catch, and the direction the second half cannot see at all.
    ///
    /// The vacuity arm it used to be — zero of zero, panicking because the gallery did not exist —
    /// is kept in `an_obligation_asked_about_nothing_is_unmet_and_not_met`, over `&[]`. A `Met`
    /// verdict cannot be watched panicking, and the refusal is still the constructor's.
    #[test]
    #[should_panic(expected = "O2 (nothing shown is absent from the freeze) is unmet: 1 of 30")]
    fn o2_nothing_shown_fails_loudly() {
        let mut shown: Vec<&str> = PANELS.to_vec();
        shown.push("gauge");
        o2_nothing_shown_is_absent_from_the_freeze(&shown)
            .assert_met("O2 (nothing shown is absent from the freeze)");
    }

    /// See [`o1_fails_loudly`]. **O2's second half has turned, so it is watched failing over the
    /// shipped list with a panel taken away** — the other direction, and the one a gallery that
    /// quietly stopped drawing a component would fail.
    #[test]
    #[should_panic(expected = "O2 (everything built has a panel) is unmet: 1 of 29")]
    fn o2_everything_built_fails_loudly() {
        let one_short: Vec<&str> = PANELS.iter().copied().skip(1).collect();
        o2_everything_built_has_a_panel(&one_short).assert_met("O2 (everything built has a panel)");
    }

    /// See [`o1_fails_loudly`]. **O3 has turned, so it is watched failing over the shipped list
    /// with one screen taken away** — the arm that matters now, which is that the query still
    /// notices a construction whose screen has gone. `plot` is the row it is taken from, because a
    /// row at three constructions can lose one and still have two: a query comparing against
    /// *non-zero* would not see it.
    #[test]
    #[should_panic(expected = "O3 is unmet: 1 of 29")]
    fn o3_fails_loudly() {
        let short: Vec<(&str, u8)> = GOLDENS
            .iter()
            .map(|(id, n)| {
                if *id == "plot" {
                    (*id, n - 1)
                } else {
                    (*id, *n)
                }
            })
            .collect();
        o3(&short).assert_met("O3");
    }

    /// See [`o1_fails_loudly`]. **Watched failing with one id struck**, because O4 is met and a
    /// met query cannot be watched failing on its own evidence.
    #[test]
    #[should_panic(expected = "O4 is unmet: 1 of 13")]
    fn o4_fails_loudly() {
        let short: Vec<&str> = KEYBOARD_REGISTERED
            .iter()
            .copied()
            .filter(|id| *id != "slider")
            .collect();
        o4(KEYBOARD_DOCUMENTED, &short).assert_met("O4");
    }

    /// See [`o1_fails_loudly`]. **The one worth more than the other four together**, and the one
    /// whose population is `(component, axis)` pairs rather than scenes.
    #[test]
    #[should_panic(expected = "O5 is unmet: 1 of 34")]
    fn o5_fails_loudly() {
        // **A later pass turned this arm and the header above says how**: *a `Met` verdict cannot
        // be watched panicking, so the `#[should_panic]` moves from `the evidence is empty` to `the
        // evidence is one row wrong`.* O1, O3, O4, O6 and O7 all made this move first; this is the
        // sixth and last, and it is O6's expression with a pair struck instead of an id.
        //
        // **`(tree, wheeled)` is the pair it is taken from**, because it is the newest and because
        // it is the one whose absence had no instrument at all: a wheel over a tree was not a
        // subject of `crate::wheel`'s drive loop until this ticket, so a build that lost the arm
        // would lose the evidence *and* the gate together. The query still notices.
        //
        // **07 wrote the minimum a green build cannot be left without and no more, and production
        // 10 took the rest**: the summary test's own name and its two register citations, the two
        // comments citing a test name that no longer exists, `CLAUDE.md`'s sentence and the components
        // README. Nothing on this arm moved with them — a paperwork ticket that edits a watch is a
        // paperwork ticket that has changed a gate.
        //
        // **The population growing is caught elsewhere and not here.** This arm strikes a pair from
        // the evidence; a freeze that declares a thirty-fifth axis leaves the evidence untouched
        // and is caught by the coverage test in `crate::scenes`
        // (`seventeen_of_the_thirty_four_axis_obligations_have_a_scene_and_seventeen_do_not`),
        // which asserts `coverage().len()` against 34 and the bare count against 0. Two
        // directions, two instruments.
        let short: Vec<(&str, Axis)> = AXIS_SCENES
            .iter()
            .copied()
            .filter(|pair| *pair != ("tree", Axis::Wheeled))
            .collect();
        o5(&short).assert_met("O5");
    }

    /// See [`o1_fails_loudly`]. **O6 is met, so it is watched failing over the shipped list with
    /// one id struck** — the arm that matters now, which is that the query still notices a
    /// component whose data-volume cost has stopped being measured.
    ///
    /// `chart` is the row it is taken from, because `chart` is the reason the obligation exists: a
    /// fold at 476 ns a point that every gate in this workspace was green on.
    #[test]
    #[should_panic(expected = "O6 is unmet: 1 of 7")]
    fn o6_fails_loudly() {
        let short: Vec<&str> = VOLUME_MEASURED
            .iter()
            .copied()
            .filter(|id| *id != "chart")
            .collect();
        o6(&short).assert_met("O6");
    }

    /// See [`o1_fails_loudly`]. **O7's first half is met, so it is watched failing over the shipped
    /// list with an id added that the freeze has never heard of** — the drift this half exists to
    /// catch, and the direction the second half cannot see at all.
    #[test]
    #[should_panic(expected = "O7 (nothing exercised is absent from the freeze) is unmet: 1 of 30")]
    fn o7_nothing_exercised_fails_loudly() {
        let mut applied: Vec<&str> = APPLIED.to_vec();
        applied.push("gauge");
        o7_nothing_exercised_is_absent_from_the_freeze(&applied)
            .assert_met("O7 (nothing exercised is absent from the freeze)");
    }

    /// See [`o1_fails_loudly`]. **O7's second half is met, so it is watched failing over the shipped
    /// list with one id struck** — the arm that matters now, which is that the query still notices a
    /// component every application has quietly stopped drawing.
    ///
    /// `file_picker` is the row it is taken from, because `file_picker` is one of the three this
    /// obligation was owed when it was built: it had no application at all, and its popup has no
    /// keyboard, which is the kind of thing only a person pressing keys finds.
    #[test]
    #[should_panic(expected = "O7 is unmet: 1 of 29")]
    fn o7_everything_declared_fails_loudly() {
        let short: Vec<&str> = APPLIED
            .iter()
            .copied()
            .filter(|id| *id != "file_picker")
            .collect();
        o7_everything_declared_has_an_application(&crate::consumer::declared(read), &short)
            .assert_met("O7");
    }

    /// **A met verdict does not panic**, which is the other direction of `assert_met` and the
    /// reason the nine `should_panic` tests above are evidence rather than decoration. **This line
    /// said seven while nine arms stood above it** — one per query, since O2 and O7 are watched in
    /// both directions — and it was corrected: the same stale count, in the same file,
    /// as the header two hundred lines up that it was opened to fix.
    #[test]
    fn a_met_obligation_is_silent() {
        Verdict::of(1, 0, "-").assert_met("a met obligation");
    }
}
