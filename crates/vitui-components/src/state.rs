//! **`press` — the face a widget draws and the face it asks to be awarded, collapsed into one
//! value.**
//!
//! Spec §3's table gives it one job — *the face drawn **and** the face awarded* — and one shape:
//! **one role, so the two cannot disagree.**
//!
//! # It exists for a disagreement, not for a state machine
//!
//! The `PressState` `architecture.md` §3 named **was built and its field was never read.**
//! `Response` already carries `hovered`, `pressed`, `press_began`, `released`, `clicked`,
//! `double_clicked` and `long_pressed`, all seven resolved by the runtime from the index that has
//! just drawn, and there is no cross-frame fact left for a component to keep. **`press_began` is
//! the seventh, and it arrived by deleting two** (runtime architecture 29): `collection` was
//! keeping a copy of last frame's `pressed` to reconstruct the edge the runtime already had, which
//! is the cross-frame fact this argument said did not exist — a finding, and it got its own
//! ticket. So the type is not here, and its absence is gated by path rather than remembered —
//! [`WhyThereIsNoPressState`], and `tests::no_source_file_in_this_crate_declares_a_press_state`
//! for the private spelling that a `compile_fail` cannot see. **If a cross-frame fact is later
//! found, it is a finding and gets its own ticket**; a field nobody reads is what this ticket
//! removed.
//!
//! # What the disagreement is, exactly
//!
//! A widget under the pointer makes **two** statements about its face, and nothing in the type
//! system connects them:
//!
//! 1. it **draws** one, from `Response::hovered` — a fact the runtime resolved last frame;
//! 2. it **declares** one, through `Ctx::hover_style` — the deferred hover award, which the runtime
//!    applies as a background restyle at `end`, from the index that has just drawn, so that the
//!    hover lands on the *first* frame of an overlap instead of a frame later.
//!
//! ADR 0026 prices the relation between them: *a restyle is free only when the component's own next
//! draw already produces the value the restyle produced.* A deferred hover award satisfies that —
//! **when the widget also branches on `hovered`, to the same face.** Written by hand the two
//! statements are ten lines apart, the screen is **correct on every frame** because the restyle
//! repaints it, and the chip re-damages its **8 cells for as long as the pointer rests on it**.
//! That is the defect [`defective::chip_by_hand`] is, and [`resting`] is the number.
//!
//! [`press`] reads [`Faces::hover`] **once**, into one local, declares the award from it and
//! returns it as the face to draw. There is no second argument, no second call and no path on which
//! the two can be given different values — which is criterion 1, structurally rather than by
//! convention.
//!
//! # Focus is a `Faces` field, because `frame::focus_ring` is deleted
//!
//! Spec §3 strikes the helper out and says where it went: *a `Role` into `block`, a `Faces` into
//! `press`*. [`crate::frame::BlockOpts::border`] is the first half and [`Faces::focus`] is the
//! second. A focused chip is drawn in a different role **before its cells are written**, never
//! restyled after — see [`crate::frame::WhyThereIsNoFocusRing`] for what the restyle form costs.
//!
//! # The pointer could not be driven from this crate, and that shaped the measurement
//!
//! When this module was written: `Driver::post_mouse` takes a `vitui_engine::Mouse`, and
//! `crates/vitui-runtime/src/line.rs` filed it `EngineName { name: "Mouse", reachable_as: None }`;
//! `Driver::plant` reaches the grab, the focus and the click and **not** the pointer. So
//! `Response::hovered` could not be made true by a gesture here, and neither could the runtime's
//! award winner.
//!
//! **Runtime architecture issue 22 lifted that**: `Mouse` is `vitui_runtime::Mouse` and the three
//! types needed to build one came with it. The substitution below is kept because it is what the
//! shipped fixture does and because it measures the **state** deliberately; what it is no longer is
//! forced.
//!
//! What [`resting`] therefore stands up is the **state** rather than the gesture: the fixture sets
//! `Response::hovered` itself — every field of a `Response` is public — and
//! [`crate::runner::Pen::end_frame`] applies the award where `Driver::frame` applies it. Both
//! substitutions are named on the number they produce, and neither is on the side of the helper:
//! the correct arm and the hand-written one are measured through the same two.

use vitui_runtime::{Ctx, Interest, Response, Role};

use crate::ink::{Direct, Ink};
use crate::runner::{Canvas, Pen, driver_at};
use crate::text::{FitOpts, Justify, fit_into};
use vitui_runtime::Rect;

/// The components homed in this module. **None**, and it is not an oversight: [`press`] is a
/// helper, and spec §17's freeze homes `button`, `chip` and `switch` in F6 and F1.
pub const MEMBERS: &[&str] = &[];

/// **The faces a widget wears, as one value.**
///
/// Four roles and not four arguments, for spec §1's rule 3 — *options are a `Default` struct, never
/// a required builder* — and because the four have to be handed over **together**: [`press`]'s
/// whole claim is that the face drawn and the face awarded come out of one value, and four
/// positional arguments are four places a caller can pass a different one.
///
/// # There is no `disabled` here, and that is not the same omission as `Face::disabled`
///
/// A disabled widget declares no interactive region, so there is no `Response` and `press` is not
/// called at all. [`crate::frame::Face`] has the bit because a **row** can be disabled inside a
/// list that is perfectly interactive, which is a different question with a different answer.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Faces {
    /// At rest.
    pub rest: Role,
    /// Under the pointer. **The one field this whole helper is about**: it is what `press` returns
    /// when the widget is hovered *and* what it declares to the award, read once.
    pub hover: Role,
    /// While a button is down on it.
    pub active: Role,
    /// While it holds the focus. **This is where `frame::focus_ring` went** (spec §3).
    pub focus: Role,
}

impl Default for Faces {
    fn default() -> Faces {
        Faces {
            rest: Role::Face,
            hover: Role::FaceHover,
            active: Role::FaceActive,
            focus: Role::Focus,
        }
    }
}

/// **The face to draw, and the same value has already been declared to the award.**
///
/// ```
/// use vitui_runtime::Rect;
/// use vitui_components::state::{Faces, press};
/// use vitui_components::text::{FitOpts, Justify, fit_with};
/// use vitui_runtime::ctx::Driver;
/// use vitui_runtime::{Interest, Role};
///
/// let mut driver = Driver::headless(20, 1).expect("a sink attaches");
/// driver.frame(|cx| {
///     let chip = Rect::new(2, 0, 8, 1);
///     let id = cx.id();
///     let resp = cx.interact(id, chip, Interest::CLICK.with(Interest::HOVER));
///     let face = press(cx, chip, &resp, &Faces::default());
///     // One role, and it is what the chip's cells are written with. Nothing else declared a face.
///     assert_eq!(face, Role::Face);
///     fit_with(cx, chip, "on", &FitOpts { justify: Justify::Middle, role: face, pad: face });
/// });
/// ```
pub fn press(cx: &mut Ctx<'_, '_>, cells: Rect, resp: &Response, faces: &Faces) -> Role {
    press_into(&mut Direct, cx, cells, resp, faces)
}

/// **[`press`], declaring the award through an [`Ink`] so an instrument can see it.**
///
/// The entry point a gate takes; `press` is this with [`Direct`]. See [`crate::ink`] for why the
/// seam exists rather than a second implementation, and [`Ink::award`] for why the award is one of
/// its methods rather than a call this function makes directly.
///
/// # The whole helper is four lines, and the first one is the point
///
/// `hover` is read out of [`Faces`] **once**. The award is declared from that local and the hovered
/// branch returns that local, so *the face drawn* and *the face awarded* are one expression
/// evaluated once. A caller cannot pass a second role: there is no parameter for one.
pub fn press_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    cells: Rect,
    resp: &Response,
    faces: &Faces,
) -> Role {
    // **Read once, used twice.** Criterion 1 is this line and the two uses below it.
    let hover = faces.hover;
    ink.award(cx, cells, resp, hover);
    // **Pressed beats hovered beats focused.** A widget under a held button is being acted on and
    // says so; a focused widget under the pointer shows the transient state, because the pointer is
    // where the user is looking.
    if resp.pressed {
        faces.active
    } else if resp.hovered {
        hover
    } else if resp.focused {
        faces.focus
    } else {
        faces.rest
    }
}

// ── the chip, and the sixty frames it rests for ──────────────────────────────────────────────────

/// The screen the chip stands on. Small on purpose: [`resting`] runs sixty frames of it, and the
/// number the ticket is about is a property of the chip and not of the screen around it.
pub const W: u16 = 40;
/// The screen's height.
pub const H: u16 = 3;
/// **How wide the chip is. Eight cells, which is the ticket's own number.**
pub const CHIP: u16 = 8;
/// Where the chip sits.
pub const AT: (u16, u16) = (2, 1);
/// What the chip says. Two columns, centred in eight, so `fit` writes three runs and no cell twice.
pub const LABEL: &str = "on";
/// How many frames the pointer rests for.
pub const FRAMES: u32 = 60;

// ── the ledger ───────────────────────────────────────────────────────────────────────────────────
//
// **Every number this fixture is gated or reported on has one home and it is here** — the runtime's
// ledger rule (`crates/vitui-runtime/src/ledger.rs`), inherited by `crate::form` one ticket ago.
// Every figure is a count over a deterministic screen, so none carries a machine.

/// **Rect the chip re-damages per steady frame through [`press`]. Zero.**
///
/// The award restyles a background the chip's own cells already carry, so it writes nothing; and
/// the next frame draws the same value into every one of the eight, so that writes nothing either.
/// Both halves are the same sentence of ADR 0026 — *a restyle is free only when the component's own
/// next draw already produces the value the restyle produced*.
pub const PRESS_STEADY: u64 = 0;
/// **Rect the hand-written chip re-damages per steady frame. Eight**, which is the chip.
///
/// Spec §3 and ADR 0026 both state it as *8 cells for as long as the pointer rests on the chip*,
/// and it reproduces exactly because the number **is** the chip's width: every cell of the face is
/// drawn in one role and restyled to another, every frame, for ever.
pub const HAND_STEADY: u64 = 8;
/// What either arm costs on the frame the chip first appears. Eight — the first paint of eight
/// cells, which is not re-damage and is counted separately for that reason.
pub const FIRST_FRAME: u64 = 8;

/// The chip's rectangle.
pub fn chip() -> Rect {
    Rect::new(i32::from(AT.0), i32::from(AT.1), CHIP, 1)
}

/// The chip's `Response`, **with the pointer resting on it**.
///
/// `hovered` is assigned rather than gestured, and this module's header says why in full: `Mouse`
/// is `reachable_as: None`, so no pointer event can be posted from this crate. Every other field is
/// the runtime's own, from a real `Ctx::interact` against a real hit index.
fn resting_on(cx: &mut Ctx<'_, '_>, cells: Rect) -> Response {
    let id = cx.id();
    let mut resp = cx.interact(id, cells, Interest::CLICK.with(Interest::HOVER));
    resp.hovered = true;
    resp
}

/// **The chip, drawn through [`press`].** The arm that ships.
///
/// One role out of the helper, and it is what every one of the eight cells is written with — the
/// label included, because a chip is one face and `fit`'s two role fields are what let it be
/// written as a partition instead of a fill under a draw.
pub fn chip_pressed<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>) {
    let cells = chip();
    let resp = resting_on(cx, cells);
    let face = press_into(ink, cx, cells, &resp, &Faces::default());
    fit_into(
        ink,
        cx,
        cells,
        LABEL,
        &FitOpts {
            justify: Justify::Middle,
            role: face,
            pad: face,
        },
    );
}

/// **The two statements, written by hand.**
///
/// `pub` for the reason [`crate::frame::defective`] and [`crate::runner::defective`] are: an
/// instrument crate's fixtures are part of the instrument, and a gate validated only against a
/// correct build reports zero for the same reason a broken one would.
pub mod defective {
    use super::{Ctx, FitOpts, Ink, Justify, LABEL, Role, chip, fit_into, resting_on};

    /// **A chip that draws one face and asks for another.**
    ///
    /// The two statements are here, eight lines apart, and **both are correct**: the chip is drawn
    /// in its resting face, and the award restyles it to the hover face at `end`, so the screen
    /// shows a hovered chip on every frame including the first. Nothing about it looks wrong,
    /// nothing is slower, and `writes == distinct` holds — this arm writes each of its eight cells
    /// exactly once, so components ticket 06's whole gate is green on it.
    ///
    /// What it costs is [`super::HAND_STEADY`] cells re-damaged on every frame the pointer rests,
    /// for ever. This is ADR 0026's *a restyle used to replace the branch never can* be free, at
    /// the smallest size it comes in.
    pub fn chip_by_hand<I: Ink>(ink: &mut I, cx: &mut Ctx<'_, '_>) {
        let cells = chip();
        let resp = resting_on(cx, cells);

        // Statement one: the face it draws. The `hovered` branch was never written — and it does
        // not need to be, because the award below makes the screen right.
        let face = Role::Face;
        fit_into(
            ink,
            cx,
            cells,
            LABEL,
            &FitOpts {
                justify: Justify::Middle,
                role: face,
                pad: face,
            },
        );

        // Statement two: the face it asks to be awarded. Nothing connects it to the one above.
        ink.award(cx, cells, &resp, Role::FaceHover);
    }
}

/// What a chip arm is drawn by. Two of these and [`resting`] make the measurement.
pub type Chip = fn(&mut Pen, &mut Ctx<'_, '_>);

/// **What sixty frames of a resting pointer cost.**
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Resting {
    /// How many frames were drawn.
    pub frames: u32,
    /// Rect changed on the **first** frame, when the chip appears. Not re-damage.
    pub first: u64,
    /// Rect changed on every frame after the first, summed.
    pub steady: u64,
    /// Rect changed on each steady frame, which is constant or this is not a steady state.
    pub per_frame: u64,
}

/// **Rest the pointer on the chip for `frames` frames and count what changes.**
///
/// # What is counted, and why it is not `marked`
///
/// Spec §20's `marked` is [`crate::counters::Reading::Unreachable`] and stays that way:
/// `crates/vitui-engine/src/damage.rs` is `pub(crate)` from top to bottom and `Presented` carries
/// no count, so no crate above the engine can read the engine's damage — components ticket 03
/// established it and [`crate::counters::Counters::marked`] panics rather than answering `0`.
///
/// What is knowable from here is the **rule** that decides it. The engine filters a write whose
/// value equals the cell's current value, so the quantity that becomes damage is *a write whose
/// value differs from what is already there*, and that is computable at the verb boundary over a
/// surface this crate keeps: [`Canvas::repaints`]. Its exactness is stated on it, in both
/// directions — exact for the draw verbs, exact for an award onto a cell already in the awarded
/// role, and conservative for an award onto any other cell, which
/// `tests::the_three_faces_this_chip_wears_are_distinct_on_the_wire` closes for the roles this
/// fixture uses.
///
/// # The frames are separated because the first one is not re-damage
///
/// A chip appearing on an untouched surface writes eight cells that were nobody's. Folding that
/// into the total would report the correct arm as costing eight cells for ever, which is a number
/// about the fixture and not about the helper.
pub fn resting(paint: Chip, frames: u32) -> Resting {
    assert!(frames >= 2, "re-damage is a relation between two frames");
    let mut driver = driver_at(W, H, vitui_runtime::Density::Compact);
    let mut canvas = Canvas::new(W, H);
    let mut first = 0u64;
    let mut steady = 0u64;
    let mut per_frame: Option<u64> = None;
    for n in 0..frames {
        let mut pen = Pen::over(canvas);
        driver.frame(|cx| paint(&mut pen, cx));
        // Where `Driver::frame` applies it: after the draw, before `present`.
        pen.end_frame();
        canvas = pen.into_canvas();
        let changed = canvas.take_repaints();
        if n == 0 {
            first = changed;
        } else {
            steady += changed;
            let seen = *per_frame.get_or_insert(changed);
            assert_eq!(
                seen, changed,
                "frame {n} changed {changed} cells and the frame before it {seen}. A pointer that \
                 has not moved for {n} frames is a steady state, and a steady state that is not \
                 constant is a defect this measurement cannot summarise"
            );
        }
    }
    Resting {
        frames,
        first,
        steady,
        per_frame: per_frame.unwrap_or_default(),
    }
}

/// **The pair that keeps `PressState` deleted, naming both items by path.**
///
/// # Why it is deleted
///
/// The proposal named it, it was built, and **its field was never read**. `Response` carries
/// `hovered`, `pressed`, `released`, `clicked`, `double_clicked` and `long_pressed`, every one of
/// them resolved by the runtime from the index that has just drawn (`Ctx::interact`), and a
/// component that keeps a copy of any of them keeps a stale one — ADR 0012, *what survives one draw
/// is five flat structures rebuilt from the next draw*. There is no cross-frame fact left for this
/// type to hold. **If one is found, that is a finding and it gets its own ticket**, because a field
/// nobody reads is the shape being removed and not the shape being asked for.
///
/// # The twin, naming the protected items by path
///
/// A lone `compile_fail` also passes when the item it names has been **renamed**: `E0432` for *the
/// thing you must not have does not exist* and `E0432` for *the thing you meant moved* are the same
/// diagnostic, and the mechanism cannot tell them apart. So the shape that ships is pinned first —
/// renaming [`press`] or [`Faces`] fails **this** half rather than making the half below pass for
/// the wrong reason.
///
/// ```
/// use vitui_runtime::Rect;
/// use vitui_components::state::{Faces, press};
/// use vitui_runtime::ctx::Driver;
/// use vitui_runtime::{Interest, Role};
///
/// let mut driver = Driver::headless(20, 1).expect("a sink attaches");
/// driver.frame(|cx| {
///     let chip = Rect::new(2, 0, 8, 1);
///     let id = cx.id();
///     let resp = cx.interact(id, chip, Interest::CLICK.with(Interest::HOVER));
///     // Seven pointer facts on the response and no eighth anywhere: the helper reads them and
///     // keeps nothing.
///     assert!(!(resp.hovered || resp.pressed || resp.press_began || resp.released));
///     assert!(!(resp.clicked || resp.double_clicked || resp.long_pressed));
///     let faces = Faces { rest: Role::Dim, ..Faces::default() };
///     assert_eq!(press(cx, chip, &resp, &faces), Role::Dim);
/// });
/// ```
///
/// # The hostile half
///
/// **Protects:** [`press`], written `vitui_components::state::press` at the path the twin uses, by
/// way of the item that must not stand beside it.
///
/// ```compile_fail,E0432
/// use vitui_components::state::PressState;
///
/// fn main() {}
/// ```
#[cfg(doc)]
pub struct WhyThereIsNoPressState;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counters::Tally;
    use std::path::PathBuf;
    use vitui_runtime::Density;

    /// Every `.rs` file under this crate's `src`, for the two source scans below.
    fn sources() -> Vec<PathBuf> {
        fn walk(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
            let Ok(entries) = std::fs::read_dir(dir) else {
                return;
            };
            for entry in entries {
                let path = entry.expect("a readable entry").path();
                if path.is_dir() {
                    walk(&path, out);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    out.push(path);
                }
            }
        }
        let mut out = Vec::new();
        walk(
            &PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")),
            &mut out,
        );
        out
    }

    /// Whether `source` carries `needle` on a line that is not a comment. **One predicate for both
    /// scans and for both of their negative halves**, which is what makes *fires in both
    /// directions* mean something: a hostile fixture run through a second copy of the logic proves
    /// the copy and not the gate. `crate::gates`' own scans are written this way for this reason.
    fn carries(source: &str, needle: &str) -> bool {
        source
            .lines()
            .map(str::trim)
            .any(|line| !line.starts_with("//") && line.contains(needle))
    }

    /// Both arms, so a sweep does not have to spell them.
    fn arms() -> [(&'static str, Chip, u64); 2] {
        [
            ("press", chip_pressed as Chip, PRESS_STEADY),
            ("by_hand", defective::chip_by_hand as Chip, HAND_STEADY),
        ]
    }

    /// **The model's one conservative direction, closed for the roles this chip wears.**
    ///
    /// [`Canvas::award`] records a change whenever the cell is not already painted in the awarded
    /// role. Two roles sharing a background would make that an over-count — the restyle would write
    /// a background the cell already carries and the engine would filter it — so the fixture asks
    /// the theme whether its three faces are distinguishable at all. They are; and if a theme change
    /// ever made them collapse, this fails rather than the number quietly becoming a different
    /// number.
    #[test]
    fn the_three_faces_this_chip_wears_are_distinct_on_the_wire() {
        let mut driver = driver_at(W, H, Density::Compact);
        driver.frame(|cx| {
            let theme = cx.theme();
            for (a, b) in [
                (Role::Face, Role::FaceHover),
                (Role::Face, Role::FaceActive),
                (Role::FaceHover, Role::FaceActive),
            ] {
                assert!(
                    theme.roles_differ_on_wire(a, b),
                    "{a:?} and {b:?} are one value on the wire, so a restyle between them writes \
                     nothing and this fixture is measuring a hover nobody can see"
                );
            }
        });
    }

    /// **Criterion 3: sixty frames of a resting pointer re-damage 0 cells through `press`, against
    /// 8 written by hand.**
    ///
    /// Both arms go through the same instrument, the same substitutions and the same screen; what
    /// differs is one call. And the defective arm is not worse by any other counter — the test
    /// below asserts that too, because *the defective build is always faster and always marks less*
    /// is what makes this pair the only gate that can see it.
    #[test]
    fn a_pointer_resting_for_sixty_frames_re_damages_nothing_through_press() {
        let pressed = resting(chip_pressed, FRAMES);
        let by_hand = resting(defective::chip_by_hand, FRAMES);

        assert_eq!(pressed.frames, FRAMES);
        assert_eq!(
            pressed.per_frame, PRESS_STEADY,
            "the helper is supposed to make the award free, and this frame changed {} cells",
            pressed.per_frame
        );
        assert_eq!(
            pressed.steady, 0,
            "fifty-nine steady frames, nothing changed"
        );
        assert_eq!(
            by_hand.per_frame, HAND_STEADY,
            "the hand-written chip is eight cells wide and re-damages all eight, every frame"
        );
        assert_eq!(
            by_hand.steady,
            HAND_STEADY * u64::from(FRAMES - 1),
            "eight cells a frame, for as long as the pointer rests"
        );
        assert_eq!(
            (pressed.first, by_hand.first),
            (FIRST_FRAME, FIRST_FRAME),
            "the two arms cost the same on the frame the chip appears, which is what makes the \
             steady difference the helper's and not the fixture's"
        );
    }

    /// **The defective arm is green on every other gate**, which is why the ticket needed a new one.
    ///
    /// It writes each of its eight cells exactly once — components ticket 06's `writes == distinct`
    /// passes — it draws the same number of cells with the same number of verbs, and the screen it
    /// produces is a correctly hovered chip on every frame. *A gate nobody has watched fail is not
    /// a gate*, and this is the other direction of that: a defect nobody has watched pass the
    /// existing gates is a defect somebody will argue is already covered.
    #[test]
    fn the_hand_written_chip_passes_every_gate_that_existed_before_this_one() {
        let mut counted: [(u64, u64, u64); 2] = [(0, 0, 0); 2];
        for (slot, (_, arm, _)) in arms().into_iter().enumerate() {
            let mut driver = driver_at(W, H, Density::Compact);
            let mut pen = Pen::new(W, H);
            driver.frame(|cx| arm(&mut pen, cx));
            let tally: &Tally = pen.tally();
            assert_eq!(
                tally.writes(),
                tally.distinct(),
                "a cell was written twice, which is a different defect from this ticket's"
            );
            assert_eq!(tally.reported(), tally.writes());
            counted[slot] = (tally.writes(), tally.distinct(), tally.verbs());
        }
        assert_eq!(
            counted[0], counted[1],
            "the two arms differ in writes, distinct or verbs — {:?} against {:?} — so some \
             counter other than the pair could have found this, and the ticket's argument is \
             weaker than it says",
            counted[0], counted[1]
        );
        assert_eq!(counted[0].0, u64::from(CHIP), "eight cells of chip");
    }

    /// **`press` awards exactly the role it returns when hovered, in every pointer state.**
    ///
    /// Criterion 1's runnable half. The award is read back off the instrument rather than inferred:
    /// [`Pen::awards`] is what the helper actually declared, so a `press` that declared one role and
    /// returned another would fail here whatever its documentation said.
    ///
    /// Swept over all eight combinations of the three facts `press` branches on, because the claim
    /// is *there is no path*, and a sample of one path is a claim about that path.
    #[test]
    fn the_face_awarded_is_the_face_a_hovered_widget_draws_on_every_path() {
        let faces = Faces {
            rest: Role::Dim,
            hover: Role::Ok,
            active: Role::Warn,
            focus: Role::Danger,
        };
        for bits in 0u8..8 {
            let (hovered, pressed, focused) = (bits & 1 != 0, bits & 2 != 0, bits & 4 != 0);
            let mut driver = driver_at(W, H, Density::Compact);
            let mut pen = Pen::new(W, H);
            let mut returned = Role::Body;
            driver.frame(|cx| {
                let cells = chip();
                let id = cx.id();
                let mut resp = cx.interact(id, cells, Interest::CLICK.with(Interest::HOVER));
                resp.hovered = hovered;
                resp.pressed = pressed;
                resp.focused = focused;
                returned = press_into(&mut pen, cx, cells, &resp, &faces);
            });
            let declared: Vec<Role> = pen.awards().iter().map(|a| a.role).collect();
            assert_eq!(
                declared,
                vec![faces.hover],
                "hovered={hovered} pressed={pressed} focused={focused}: `press` declared a \
                 different award, or more than one"
            );
            let expected = if pressed {
                faces.active
            } else if hovered {
                faces.hover
            } else if focused {
                faces.focus
            } else {
                faces.rest
            };
            assert_eq!(returned, expected);
            if hovered && !pressed {
                assert_eq!(
                    returned, declared[0],
                    "hovered and not pressed: the face drawn and the face awarded are the two \
                     statements this helper exists to collapse"
                );
            }
        }
    }

    /// **The award is applied after the draw, and applying it before inverts the answer.**
    ///
    /// The instrument's own correctness, watched in both directions. `Driver::frame` applies the
    /// restyle *after* the draw and before `present`; a model that applied it at the declaration
    /// would have the chip's own cells overwrite it, and the correct arm — which declares before it
    /// draws — would then score eight changed cells a frame while the hand-written one scored the
    /// same. The ordering is the difference between a gate and a coin toss.
    #[test]
    fn the_award_is_applied_after_the_draw_and_not_before() {
        let mut driver = driver_at(W, H, Density::Compact);
        let mut canvas = Canvas::new(W, H);
        // Frame one: the chip appears and its award is applied at the end.
        let mut pen = Pen::over(canvas);
        driver.frame(|cx| chip_pressed(&mut pen, cx));
        assert_eq!(pen.awards().len(), 1, "one award, declared by the helper");
        assert!(pen.awards()[0].applied, "the pointer is resting on it");
        pen.end_frame();
        canvas = pen.into_canvas();
        assert_eq!(canvas.take_repaints(), FIRST_FRAME);

        // The award landed on cells already painted in the awarded role, so it changed nothing.
        // That is what `end_frame` running *after* the draw means, and it is checked rather than
        // assumed: the cell carries no restyle.
        for x in AT.0..AT.0 + CHIP {
            let cell = canvas.get(x, AT.1).expect("the chip wrote this cell");
            assert_eq!(
                cell.hover, None,
                "({x}, {}) carries a standing restyle, so the award changed it and the next \
                 frame's draw will change it back",
                AT.1
            );
        }
    }

    /// **`PressState` is not declared anywhere in this crate** — criterion 2's source half.
    ///
    /// The `compile_fail` pair on [`WhyThereIsNoPressState`] catches the item coming back on the
    /// *public* surface. This catches it coming back as a private one, which is how a deleted type
    /// actually returns: somebody writes it `struct PressState` inside a component "just for the
    /// chip", and the pair stays green because nothing outside can name it.
    ///
    /// Assembled from fragments for `crate::frame`'s reason: the first run of that file's twin
    /// reported *its own scanner line* as an offender.
    #[test]
    fn no_source_file_in_this_crate_declares_a_press_state() {
        let files = sources();
        assert!(files.len() > 10, "the scan found no source to scan");
        let needles = [
            ["struct ", "PressState"].concat(),
            ["enum ", "PressState"].concat(),
            ["type ", "PressState"].concat(),
        ];
        let mut offenders = Vec::new();
        for file in &files {
            let source = std::fs::read_to_string(file).expect("a readable source file");
            // A mention in prose is the point — this module argues about the deleted type at
            // length — so only a declaration counts, and `carries` skips comment lines.
            if needles.iter().any(|needle| carries(&source, needle)) {
                offenders.push(file.to_string_lossy().into_owned());
            }
        }
        assert_eq!(
            offenders,
            Vec::<String>::new(),
            "`PressState` was built once and its field was never read: `Response` carries all \
             seven pointer facts and there is no cross-frame fact left. If one has been found, \
             that is a finding and gets its own ticket rather than a field nobody reads"
        );
        // **The other direction, through the same predicate**, because a scan that has quietly
        // stopped scanning reports zero as loudly as a clean crate does.
        assert!(carries(
            &format!("pub {} {{ at: u32 }}", needles[0]),
            &needles[0]
        ));
        assert!(carries(
            &format!("    {} {{ Idle }}", needles[1]),
            &needles[1]
        ));
        assert!(!carries(
            &format!("// {} in a comment", needles[0]),
            &needles[0]
        ));
    }

    /// **The hover award is declared in one file, and it is the one that collapses the pair** —
    /// criterion 1's structural half.
    ///
    /// `Ctx::hover_style` is the only verb that makes the second of the two statements. If a second
    /// module could call it, *there is no path on which the two can be given different values*
    /// would be a claim about this file rather than about the crate.
    ///
    /// **This was two files and is now one**, which is the one place components architecture issue
    /// 17 made the crate simpler rather than merely different. The verb used to reach the runtime
    /// through two hops — `cells.rs` declared `Cells::hover_style` and `ink.rs` routed it through
    /// the seam — because a component could not name `Rect` and needed a method on the crate's own
    /// rectangle to get one. With `Rect` nameable there is no first hop: `ink.rs` spells
    /// `Ctx::hover_style` directly and is the only file that may. And the only **helper** that
    /// declares an award is still this one — `state.rs`, [`press_into`] plus the fixture beside it
    /// that exists to be watched failing.
    #[test]
    fn the_hover_award_is_declared_in_one_file_and_it_is_the_one_that_collapses_it() {
        let verb = ["hover_", "style("].concat();
        let declaration = ["ink.", "award("].concat();
        let mut spells = Vec::new();
        let mut declares = Vec::new();
        for file in &sources() {
            let source = std::fs::read_to_string(file).expect("a readable source file");
            let name = file
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();
            if carries(&source, &verb) {
                spells.push(name.clone());
            }
            if carries(&source, &declaration) {
                declares.push(name);
            }
        }
        spells.sort();
        declares.sort();
        assert_eq!(
            spells,
            vec!["ink.rs".to_string()],
            "a second file spells the runtime's hover verb. It is routed exactly once, through \
             `Ink::award`, and everything else in this crate reaches it from there"
        );
        assert_eq!(
            declares,
            vec!["state.rs".to_string()],
            "a second helper declares a hover award. The face drawn and the face awarded are two \
             statements, and `state::press` is the only place in this crate where they are one"
        );
        // The other direction, through the same predicate.
        assert!(carries(
            &format!("    cx.{verb}&resp, cells, role);"),
            &verb
        ));
        assert!(carries(
            &format!("    {declaration}cx, cells, &resp, role);"),
            &declaration
        ));
    }

    /// **The three helpers carry no `Style` literal and name no `GlyphSet`** — criterion 7.
    ///
    /// `Style` is `vitui_engine::Style` and this crate cannot name it at all (§19's C6), so the
    /// half that bites is the other one: a helper that reaches a repertoire is a helper that has
    /// started branching on the glyph axis, which is the twenty-four repertoire-qualified
    /// occurrences across four component crates that ADR 0032 exists because of — the literal is
    /// not written here, because `inventory::tests::no_component_source_names_the_repertoire` scans
    /// this file too and a scan its own documentation satisfies is a scan that always fails. Both
    /// needles are scanned anyway,
    /// because *the compiler already stops it* is exactly the argument that was made about the
    /// wildcard dependency versions right up to the thirteen `error[wildcard]` in §19.
    ///
    /// `frame.rs` is scanned whole, which is stricter than the criterion asks: `block` lives beside
    /// `face_paint` and neither may spell one.
    #[test]
    fn the_three_helpers_carry_no_style_literal_and_name_no_glyph_set() {
        let glyph_set = concat!("Glyph", "Set");
        let style = concat!("Sty", "le");
        let mut offenders = Vec::new();
        let mut scanned = 0usize;
        for name in ["state.rs", "frame.rs", "scroll.rs"] {
            let path = PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/src")).join(name);
            let source = std::fs::read_to_string(&path).expect("a helper's source file");
            scanned += 1;
            for (n, line) in source.lines().enumerate() {
                let line = line.trim();
                if line.starts_with("//") || line.starts_with("///") {
                    continue;
                }
                if line.contains(glyph_set) || line.contains(style) {
                    offenders.push(format!("{name}:{}", n + 1));
                }
            }
        }
        assert_eq!(scanned, 3, "three helpers, three files");
        assert_eq!(
            offenders,
            Vec::<String>::new(),
            "a helper names a glyph repertoire or builds a style. A component names a **role** \
             (ADR 0018) and a **glyph** (§16), and never either axis"
        );
        // The other direction, through the same predicate.
        assert!(format!("    let g = {glyph_set}::Ascii;").contains(glyph_set));
        assert!(format!("    let s = {style}::new().bold();").contains(style));
    }
}
