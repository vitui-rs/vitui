//! Spec §17's five obligations as **queries over [`crate::INVENTORY`]**, each returning a count or
//! an equality.
//!
//! > Every documentation and verification obligation is a query over it, not a sentence in a
//! > document. (ADR 0033)
//!
//! # Three of the five still cannot run, and that is the load-bearing half of this file
//!
//! **O1 is green since components ticket 36 and O3 since components ticket 37**, the first two of
//! the six queries to turn. A gallery panel and a keyboard contract are still things later tickets
//! build, and **a query over an obligation nobody has met yet is the exact shape that returns green
//! by accident**:
//!
//! - *every panel in the gallery is in the freeze* over an empty gallery is **vacuously true**;
//! - *every component has at least one scene per declared axis* over an empty scene list, written
//!   as a loop over scenes rather than over components, is **vacuously true**;
//! - *goldens == constructions* written as `for g in goldens` is **vacuously true** — which is why
//!   [`o3`] iterates the freeze and not the evidence, and why its population is `built` rather than
//!   all twenty-nine: a row nothing on this backlog can draw a screen for reads **red** for ever,
//!   which is the same failure in mirror image.
//!
//! Three of the five have that shape, and §21 has already been bitten by the neighbouring version
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
//! [`Verdict::assert_met`] panics with the failing set and the ticket that inverts it. Each of the
//! five is watched panicking by a `#[should_panic]` test below, because **a gate nobody has watched
//! fail is not a gate** — §21's own three-for-three finding, from the other direction.
//! `tests::not_one_of_the_five_obligations_is_met` writes the number down, so the first one to turn
//! green is a deliberate edit here rather than a silent change of colour.
//!
//! # The evidence is an argument, not a file read
//!
//! Every query takes what it is checking against as a slice, and this crate shipped each of those
//! slices **empty with the ticket that fills it named**. That is what keeps the five pure functions
//! over the freeze: when ticket 39 builds the gallery, [`PANELS`] stops being empty and `o2` starts
//! answering a real question with no change to the query.
//!
//! **[`DOC_TESTED`] is the first one to be filled, and filling it needed a second value beside it.**
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
        why: &'static str,
        /// The implementation ticket that makes it possible to meet, as `components NN`.
        inverted_by: &'static str,
    },
}

impl Verdict {
    /// Build a verdict, refusing vacuous truth.
    ///
    /// **An empty population is `Unmet`, whatever the failing count says.** A query asked about
    /// nothing has not been answered, and the arithmetic that says otherwise — zero failures out of
    /// zero — is exactly how three of the five would read green today.
    pub const fn of(
        over: usize,
        failing: usize,
        why: &'static str,
        inverted_by: &'static str,
    ) -> Verdict {
        if over == 0 {
            return Verdict::Unmet {
                over: 0,
                failing: 0,
                why,
                inverted_by,
            };
        }
        if failing == 0 {
            return Verdict::Met { over };
        }
        Verdict::Unmet {
            over,
            failing,
            why,
            inverted_by,
        }
    }

    /// Whether it holds.
    pub const fn met(self) -> bool {
        matches!(self, Verdict::Met { .. })
    }

    /// Panic with the failing set and the ticket that inverts it.
    ///
    /// This is the loud half. It is what a later ticket's gate calls, and what the
    /// `#[should_panic]` tests below watch, because a failure nobody has read is a failure nobody
    /// can act on.
    pub fn assert_met(self, obligation: &str) {
        if let Verdict::Unmet {
            over,
            failing,
            why,
            inverted_by,
        } = self
        {
            panic!(
                "{obligation} is unmet: {failing} of {over} — {why}. Inverted by `{inverted_by}`"
            );
        }
    }
}

/// The ids that carry a rustdoc page with a **compiled** example. O1's evidence.
///
/// **Twenty-eight of twenty-nine, and filled by components ticket 36.** Every built row of the
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
    "file_picker",
    "file_preview_pane",
];

/// The ids the gallery binary shows a panel for. O2's evidence.
///
/// Empty, and ticket 39 fills it. **The gallery is a gate rather than a demo** (§21): every defect
/// on the map that survived every gate then in force was invisible on the screen of the ticket that
/// owned the mechanism and visible on the screen where the components meet.
pub const PANELS: &[&str] = &[];

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
    ("file_picker", 1),
    ("file_preview_pane", 1),
];

/// The ids whose keyboard contract is **documented** and rendered as help. O4's first half.
///
/// Empty, and ticket 38 fills it.
pub const KEYBOARD_DOCUMENTED: &[&str] = &[];

/// The ids whose keyboard contract is **registered** in the runtime's key map. O4's second half.
///
/// Empty, and ticket 38 fills it. O4 is an equality between the two and not a subset in either
/// direction: a binding that is registered and undocumented is a feature nobody can find, and one
/// that is documented and unregistered is a help bar that lies.
pub const KEYBOARD_REGISTERED: &[&str] = &[];

/// The scenes that exist, as `(component, axis)` pairs. O5's evidence.
///
/// **Seventeen of thirty-four.** Ticket 04 put twelve here off spec §21's own rows, ticket 09 added
/// two, ticket 20 added one, and neither components ticket 10 nor components ticket 11 moved it at
/// all.
///
/// **Ticket 20's is the pair §21 could not state**, and it is worth separating from the two below it
/// for the same reason they are separated from each other. §21 carries the wheel as one row over one
/// component, which is exactly what was writable while the click was an arithmetic substitution — a
/// delta added to an offset has no second axis to be wrong on. `scroll_area` has declared
/// `owns_offset` since ticket 01 and had no scene claiming the axis; scene 33 is that scene, and
/// what it decided is a pair rather than a number: a body dead downward is alive sideways.
///
/// **Ticket 11 built five scenes for `collection` and added no pair either, for a reason worth
/// separating from ticket 10's.** Ticket 10's was *a component is not a scene*; this one is *a
/// standing is not a pair*. The three pairs `collection` declares — scrolled, shrunk, wheeled — were
/// already claimed here by ticket 04, off §21's own **scene** rows 4, 5 and 6. What ticket 11 changed is that
/// those three scenes stopped being `Unsubjected` and became `Red` with an exact failing set, and
/// [`axis_scenes_of`](crate::scenes::axis_scenes_of) does not read `standing` at all: its one filter
/// is `owed`, because §21's `(owed)` marks a scene *whose number was never measured*, and a red
/// scene's numbers are measured. The fifth scene, the narrow collection, deliberately claims **no**
/// pair — `INVENTORY` sets `narrow: false` on `collection` because a row truncates through
/// `text::fit`, which is `text`'s flag, and scene 28 already carries `(text, narrow)`. A pair here
/// for an axis the freeze does not set is evidence for nothing and would inflate the count silently.
///
/// Ticket 10 built four components and added no pair, which is not an oversight and is worth the
/// sentence: `panel` and `button` declare **no** hostile axis at all, and the axes `text` and `chip`
/// do declare — [`Axis::Narrow`], both of them — already have scene 28, which ticket 09 wrote
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
    // §21 scene 4 — the inverted scroll sign, 12.21 us against 62.96 and *faster*.
    ("collection", Axis::Scrolled),
    // §21 scene 5 — the stale tail, 71 of 80 rows.
    ("collection", Axis::Shrunk),
    // §21 scene 6 — twenty wheel clicks move the offset 0 against 16.
    ("collection", Axis::Wheeled),
    // §21 scene 7 — a twelve-column table under a horizontal offset, pinned both edges.
    ("table", Axis::Scrolled),
    ("table", Axis::Narrow),
    // §21 scene 8 — a million-node forest at depth 59 999, windowed by the flatten index.
    ("tree", Axis::Scrolled),
    // §21 scene 9 — a fold and an unfold: content shrinking inside a rectangle that does not move.
    ("tree", Axis::Shrunk),
    // §21 scene 11 — the accordion, 478 hit entries against 70.
    ("collapsible", Axis::Shrunk),
    // §21 scene 13 — the wrap memo at 300 and at 120, 625 rows drawn where 875 are needed.
    ("field", Axis::Narrow),
    // §21 scene 15 — 60x20, where C08's overlap is red.
    ("chart", Axis::Narrow),
    ("plot", Axis::Narrow),
    // §21 scene 17 — the bar fixpoint over 5 475 600 pairs. The freeze cites this scene by number
    // on `scrollbar`'s own `narrow` row, and `scroll_area`'s cites the other half of it: reserved
    // auto-hiding bars, whose hysteresis loses a row and a column permanently.
    ("scroll_area", Axis::Narrow),
    ("scrollbar", Axis::Narrow),
    // §21 scene 18 — a 1M-row scroll area, row 799 999 of 999 999.
    ("scroll_area", Axis::Scrolled),
    // §21 scenes 23 and 25 — the preview pane's shrink and scroll, and the picker's scroll.
    // **Empty until components 32**, which is not a filing decision: while all three scenes were
    // red they were waiting for their subject, and a scene waiting for its subject is not yet
    // evidence of anything. §15 states both of the pane's axes in its own words — *a landing is a
    // shrink*, from another thread for the first time, and the four offset spellings that follow
    // from the offset belonging to neither side.
    ("file_preview_pane", Axis::Shrunk),
    ("file_preview_pane", Axis::Scrolled),
    ("file_picker", Axis::Scrolled),
    // Scene 28 — components 09's narrow axis over the dense screen, at 300x80 and at 120x40. Not a
    // row of §21: the first two scenes of that table are the dense screen and its twin and neither
    // names an axis, and the defect this one is about — *a label that runs into its sibling's
    // rectangle* — is §2's third re-damage instance rather than §21's.
    ("text", Axis::Narrow),
    ("chip", Axis::Narrow),
    // Scene 33 — components 20's, and the pair §21 had no way to state. Its wheel row is one
    // component and one axis, which is exactly what an arithmetic click could express: a delta
    // added to an offset has no second axis to be wrong on. The posted notch found the pair.
    //
    // **Its position is scene order and not importance.** `crate::scenes::axis_scenes` derives this
    // list from the scene list, and the test that compares the two is an equality over ordered
    // vectors — so a pair written in the place a reader would put it is a failing test rather than a
    // tidy constant.
    ("scroll_area", Axis::Wheeled),
];

/// **O1 — a rustdoc page with a compiled example, for every component.**
///
/// The count is *components with 0 doc-tests == 0*. O1 and O2 do not substitute for each other: O1
/// catches an API that cannot be called from outside the crate, O2 catches an inventory that has
/// drifted from what ships, and **neither catches a wrong cell** — that is O3 and O5.
///
/// # The population is `built`, and that is a finding rather than a convenience
///
/// §17 states O2's second equality as *everything **`built`** must have a panel* and states O1's
/// count with no population at all — *components with 0 doc-tests == 0* — so the reading is owed
/// rather than given. **It is the same population, for the same reason.** `spinner` is the one row
/// of the twenty-nine that no ticket has built: its mechanism is *a component that owns a clock*,
/// it is still §22's, and it is components ticket 42's to prototype. A doc page for a function that
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
        "components 36",
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
        "components 39",
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
        "components 39",
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
/// §17 states O2's second equality over `built` and states O3's count over nothing at all, so the
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
        "components 37",
    )
}

/// **O4 — the declared keyboard contract, rendered as help.**
///
/// An equality: documented == registered. §21 adds the two checks that keep it honest — the
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
/// That is §21's first refinement arriving from a direction it did not name — **a threshold on the
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
        "components 38",
    )
}

/// **O5 — at least one scene per hostile axis a component declares.**
///
/// > **O5 is worth more than the other four together.** (§17)
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
        "components 04",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The wheel gate's subjects are counted here and nowhere else.** Register row 129, and the
    /// second half of components ticket 20's criterion 6.
    ///
    /// `crate::wheel` asserts the freeze *declares* the axis for both subjects it plays over; this
    /// asserts the other direction, which is the one that can go quietly wrong: **O5 holds a pair
    /// for each of them**. The two are not the same question and neither implies the other — a
    /// subject the freeze declares and no scene claims is an axis with no evidence, and a pair
    /// claimed for a component the gate never runs against is evidence for nothing.
    ///
    /// It is written over [`AXIS_SCENES`] rather than over the scene list because that is the list
    /// [`o5`] actually reads. `crate::scenes` already gates the two against each other, so a pair
    /// present here and absent from a scene fails there instead of silently passing both.
    #[test]
    fn o5_counts_a_pair_for_every_axis_the_wheel_gates_subjects_declare() {
        use crate::wheel::Subject;

        for subject in Subject::ALL {
            assert!(
                AXIS_SCENES
                    .iter()
                    .any(|(id, axis)| *id == subject.id() && *axis == Axis::Wheeled),
                "the wheel gate runs over `{}` and O5 holds no `(component, wheeled)` pair for it, \
                 so the axis has a gate and no evidence",
                subject.id()
            );
        }
        // And the join is exactly the gate's subject list: a third pair here would be an axis
        // claimed for a component nothing plays a wheel over.
        let wheeled: Vec<&str> = AXIS_SCENES
            .iter()
            .filter(|(_, axis)| *axis == Axis::Wheeled)
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(
            wheeled,
            Subject::ALL.map(|s| s.id()).to_vec(),
            "the pairs and the gate's subject list are the same population, derived from the same \
             value — written out by hand on each side, a third subject escapes both halves while \
             both stay green"
        );
    }

    /// **Two of the five are met, and the number is written down.**
    ///
    /// The runtime `register.rs`'s arrangement, one crate up: a list that says how many are green makes
    /// the next change a deliberate edit rather than a quiet one. Today the answer is **two of
    /// six** queries — O2 is two equalities — and every one of the other four names the ticket that
    /// inverts it.
    ///
    /// **O1 was the first**, and components ticket 36 was the deliberate edit this test was written
    /// to force: it stood at zero for thirty-five tickets, twenty-five of which shipped a component
    /// entitled to add a row to [`DOC_TESTED`] and none of which did. **O3 is the second**, and
    /// components ticket 37 is its edit — thirty-three screens over the twenty-eight built rows,
    /// with the equalities in `crates/vitui-components/tests/golden.rs`.
    #[test]
    fn two_of_the_five_obligations_are_met_and_they_are_o1_and_o3() {
        let all = [
            ("O1", o1(DOC_TESTED)),
            ("O2a", o2_nothing_shown_is_absent_from_the_freeze(PANELS)),
            ("O2b", o2_everything_built_has_a_panel(PANELS)),
            ("O3", o3(GOLDENS)),
            ("O4", o4(KEYBOARD_DOCUMENTED, KEYBOARD_REGISTERED)),
            ("O5", o5(AXIS_SCENES)),
        ];
        let met: Vec<&str> = all
            .iter()
            .filter(|(_, v)| v.met())
            .map(|(n, _)| *n)
            .collect();
        assert_eq!(
            met,
            vec!["O1", "O3"],
            "an obligation has changed colour. That is the point of the backlog and it is also a \
             deliberate edit to this test, to this module's header and to the ticket that inverted \
             it — the number is here so a green one cannot arrive unremarked"
        );

        for (name, verdict) in all.into_iter().filter(|(n, _)| *n != "O1" && *n != "O3") {
            let Verdict::Unmet {
                over,
                failing,
                why,
                inverted_by,
            } = verdict
            else {
                unreachable!("checked above");
            };
            assert!(why.len() > 40, "{name} does not say what is wrong");
            assert!(
                inverted_by.starts_with("components "),
                "{name} does not name the implementation ticket that inverts it"
            );
            assert!(failing <= over);
        }
    }

    /// **The exact failing sets, so that a change in either direction is visible.**
    ///
    /// §21's rule for a pinned red gate: *it asserts its exact failing set, fires in both
    /// directions, and says what to invert when it is fixed*. Twenty-nine components, nineteen
    /// built, thirty-four axis obligations and thirty-four goldens owed.
    #[test]
    fn each_query_reports_the_population_it_could_not_answer_for() {
        let unmet = |v: Verdict| match v {
            Verdict::Unmet { over, failing, .. } => (over, failing),
            Verdict::Met { .. } => panic!("met"),
        };
        // **O1 is `Met` over the twenty-eight built rows**, so it has no failing set to report and
        // is asserted from the other side. See `o1` for why the population is `built` and not all
        // twenty-nine.
        assert_eq!(o1(DOC_TESTED), Verdict::Met { over: 28 }, "O1");
        assert_eq!(DOC_TESTED.len(), 28);
        // The two halves of O2 differ in population, which is exactly ADR 0033's point that they do
        // not substitute for each other. The first is over the gallery and is **empty**, so it is
        // the vacuity refusal firing; the second is over the twenty-eight built rows — twenty-five
        // until ticket 35, and it is the one obligation whose population *grows* every time a
        // component ships, which is what makes it a check on drift rather than on effort.
        assert_eq!(
            unmet(o2_nothing_shown_is_absent_from_the_freeze(PANELS)),
            (0, 0),
            "O2a"
        );
        assert_eq!(
            unmet(o2_everything_built_has_a_panel(PANELS)),
            (28, 28),
            "O2b"
        );
        // **O3 is `Met` over the same twenty-eight built rows**, so it is asserted from the other
        // side too. See `o3` for why the population is `built`: `spinner` has no component to draw,
        // and a row nothing on this backlog can invert is a row that reads red for ever.
        assert_eq!(o3(GOLDENS), Verdict::Met { over: 28 }, "O3");
        // **O4's population is 0, and that is a finding rather than an oversight.** Written over
        // `INVENTORY` it returned `Met` over twenty-nine, because two empty lists agree about every
        // row — the one query of the six that read green, and the reason this test exists at all.
        // See `o4`'s own documentation.
        assert_eq!(
            unmet(o4(KEYBOARD_DOCUMENTED, KEYBOARD_REGISTERED)),
            (0, 0),
            "O4"
        );
        // **O5 has moved three times and it is still red.** Ticket 04's scene list covered twelve
        // of the thirty-four `(component, axis)` pairs from §21's own rows, ticket 09's narrow axis
        // added `text` and `chip`, and ticket 20 added `(scroll_area, wheeled)` — the pair §21 had
        // no way to state, because its single wheel row was written while a click was an arithmetic
        // substitution and a delta added to an offset has no second axis to be wrong on. The other
        // seventeen were the per-component scenes tickets', and components 32 took three of them:
        // the preview pane's shrink and scroll and the picker's scroll, which the three scenes had
        // left empty on purpose while they were red. A query that moves is a query that is
        // measuring something.
        assert_eq!(unmet(o5(AXIS_SCENES)), (34, 14), "O5");

        // **The construction sum, both ways round.** Over the whole freeze it is 34 — 29 rows plus
        // `chart`, `meter` and `sparkline` at 2 and `plot` at 3 — and over the rows that have a
        // component to draw it is **33**, which is what `GOLDENS` adds up to and what
        // `crate::golden::SCREENS` holds.
        let owed: u32 = INVENTORY.iter().map(|c| u32::from(c.constructions)).sum();
        assert_eq!(owed, 34);
        let buildable: u32 = INVENTORY
            .iter()
            .filter(|c| c.built)
            .map(|c| u32::from(c.constructions))
            .sum();
        assert_eq!(buildable, 33);
        assert_eq!(GOLDENS.iter().map(|(_, n)| u32::from(*n)).sum::<u32>(), 33);
    }

    /// **Vacuous truth is refused in the constructor, and this is where that is asserted.**
    ///
    /// Not a hypothetical: `o2_nothing_shown_is_absent_from_the_freeze` over an empty [`PANELS`]
    /// has **zero failures out of zero**, and the arithmetic that calls that success is the same
    /// arithmetic that reported `allocs / n == 0` for a frame allocating on n−1 of n frames.
    #[test]
    fn an_obligation_asked_about_nothing_is_unmet_and_not_met() {
        assert!(
            !Verdict::of(
                0,
                0,
                "nothing to check, which is the failure",
                "components 00"
            )
            .met()
        );
        assert!(Verdict::of(1, 0, "-", "components 00").met());
        assert!(!Verdict::of(1, 1, "-", "components 00").met());
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
    }

    /// **The five, each watched failing.** A gate nobody has watched fail is not a gate.
    ///
    /// §21 records that three for three: engine ticket 13 left a `cargo test --workspace` step that
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
    #[should_panic(expected = "O1 is unmet: 1 of 28")]
    fn o1_fails_loudly() {
        let one_short: Vec<&str> = DOC_TESTED.iter().copied().skip(1).collect();
        o1(&one_short).assert_met("O1");
    }

    /// See [`o1_fails_loudly`]. This is the vacuity arm: zero of zero, and it still panics.
    #[test]
    #[should_panic(expected = "O2 (nothing shown is absent from the freeze) is unmet: 0 of 0")]
    fn o2_nothing_shown_fails_loudly() {
        o2_nothing_shown_is_absent_from_the_freeze(PANELS)
            .assert_met("O2 (nothing shown is absent from the freeze)");
    }

    /// See [`o1_fails_loudly`].
    #[test]
    #[should_panic(expected = "O2 (everything built has a panel) is unmet: 28 of 28")]
    fn o2_everything_built_fails_loudly() {
        o2_everything_built_has_a_panel(PANELS).assert_met("O2 (everything built has a panel)");
    }

    /// See [`o1_fails_loudly`]. **O3 has turned, so it is watched failing over the shipped list
    /// with one screen taken away** — the arm that matters now, which is that the query still
    /// notices a construction whose screen has gone. `plot` is the row it is taken from, because a
    /// row at three constructions can lose one and still have two: a query comparing against
    /// *non-zero* would not see it.
    #[test]
    #[should_panic(expected = "O3 is unmet: 1 of 28")]
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

    /// See [`o1_fails_loudly`].
    #[test]
    #[should_panic(expected = "O4 is unmet: 0 of 0")]
    fn o4_fails_loudly() {
        o4(KEYBOARD_DOCUMENTED, KEYBOARD_REGISTERED).assert_met("O4");
    }

    /// See [`o1_fails_loudly`]. **The one worth more than the other four together**, and the one
    /// whose population is `(component, axis)` pairs rather than scenes.
    #[test]
    #[should_panic(expected = "O5 is unmet: 14 of 34")]
    fn o5_fails_loudly() {
        o5(AXIS_SCENES).assert_met("O5");
    }

    /// **A met verdict does not panic**, which is the other direction of `assert_met` and the
    /// reason the six `should_panic` tests above are evidence rather than decoration.
    #[test]
    fn a_met_obligation_is_silent() {
        Verdict::of(1, 0, "-", "components 00").assert_met("a met obligation");
    }
}
