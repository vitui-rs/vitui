//! **F6 input**, ~110 entries — the largest family — expressed by `field`, `button`, `chip`,
//! `select`, `collection` and `slider`.
//!
//! The reductions are R1, R2 and R3 (spec §18). §11's one flag absorbs sixteen named input
//! variants including `textarea`; §5's `Mode` absorbs the radio set and the segmented control;
//! a `Role` absorbs every button variant. The colour wheel, the dial and the font picker are R4 —
//! sub-cell rasterisation, one rasteriser and a different mapping.
//!
//! **Two entries are open rather than reduced**: ctrl-click and shift-click, because `rt::Input`
//! carries no modifier byte on a pointer event. The keyboard half of multi-select is complete; the
//! pointer half is inexpressible, and that is the runtime map's to change (§22).

//! # `button` is spec §1's own example, and the shape is the rule with one substitution
//!
//! §1 writes it out: `pub fn button(cx: &mut Ctx, area: Rect, label: &str) -> Response`. **`Rect`
//! cannot be named from this package** — it is `vitui_engine::Rect`, `reachable_as: None`, and C6
//! says the dependency table is `vitui-runtime` and nothing else — so what ships takes
//! [`Cells`], this crate's own rectangle in the `Ctx`'s coordinates, swept operator for operator
//! against `vitui_runtime::layout::rect`. See [`crate::cells`] for the four candidates and why this
//! is the one; the rule is obeyed with `Cells` in `Rect`'s place, not set aside.

use vitui_runtime::{Ctx, Interest, Response, Role};

use crate::cells::Cells;
use crate::ink::{Direct, Ink};
use crate::state::Faces;
use crate::text::{ChipOpts, Justify, chip_drawn};

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &[
    "button", "field", "select", "checkbox", "radio", "switch", "form", "slider",
];

/// [`button`]'s options.
///
/// Spec §1's rule 3: a `Default` struct, never a required builder.
///
/// # It is [`ChipOpts`] under another name today, and that is a finding rather than an oversight
///
/// Spec §17 gives `button` and `chip` `constructions: 1` apiece, and *one construction* is exactly
/// what this is: a face out of [`crate::state::press`] with a label written into it. The two entries
/// are two components in the freeze — two demands, two vocabularies, two places in the tree — and
/// **they are not two drawings.** The drawing is one function, `crate::text`'s `face_and_label`,
/// which is why R07's fill-then-draw defect cannot be present in one of them and absent from the
/// other, and why the day one of them grows a real difference there is a struct here to put it in.
///
/// What was expected to be the difference — *a button's label is [`Role::Title`] and a chip's is
/// [`Role::Dim`]* — turned out to be a **defect in both**. See [`ChipOpts`]: a label wearing a role
/// its face does not is repainted by the hover award every frame the pointer rests, for ever. So
/// there is no `label` field here either, and the reason is one sentence of ADR 0026 rather than a
/// simplification.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ButtonOpts {
    /// The four faces, handed to [`crate::state::press`] whole.
    pub faces: Faces,
    /// Where the label sits on the face.
    pub justify: Justify,
    /// What the button declares.
    ///
    /// **`HOVER` is not decoration.** `press` returns [`Faces::hover`] when `Response::hovered` is
    /// true, and `hovered` is `frame.hover_guess == id`, resolved only for a region that asked for
    /// the pointer. A button drawing a hover face without declaring hover is a branch nothing can
    /// ever take.
    pub interest: Interest,
}

impl Default for ButtonOpts {
    fn default() -> ButtonOpts {
        ButtonOpts {
            faces: Faces::default(),
            justify: Justify::Middle,
            interest: Interest::CLICK.with(Interest::HOVER).with(Interest::FOCUS),
        }
    }
}

/// **A label on a face that reacts, and a tab stop.** Spec §1's own signature, with `Cells` for
/// `Rect`.
///
/// ```
/// use vitui_components::cells::Cells;
/// use vitui_components::input::button;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(20, 1).expect("a sink attaches");
/// driver.frame(|cx| {
///     let resp = button(cx, Cells::at(0, 0, 10, 1), "reset");
///     // Rule 4: a `Response` back, and nothing has happened to it on a frame with no input.
///     assert!(!resp.clicked);
/// });
/// // One tab stop, because `Interest::FOCUS` is in the default.
/// assert_eq!(driver.inspect().stop_count(), 1);
/// ```
#[track_caller]
pub fn button(cx: &mut Ctx<'_, '_>, area: Cells, label: &str) -> Response {
    button_with(cx, area, label, &ButtonOpts::default())
}

/// [`button`], with the options spelled out.
#[track_caller]
pub fn button_with(cx: &mut Ctx<'_, '_>, area: Cells, label: &str, opts: &ButtonOpts) -> Response {
    button_into(&mut Direct, cx, area, label, opts)
}

/// **[`button`], drawing through an [`Ink`] so a counter can see the verbs.**
///
/// `#[track_caller]` for [`Ctx::id`]'s documented reason: the attribute is on `id`, so a component
/// that draws **one** widget has to carry it or every call site in an application collides on the
/// line inside this file.
#[track_caller]
pub fn button_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Cells,
    label: &str,
    opts: &ButtonOpts,
) -> Response {
    let id = cx.id();
    let resp = area.interact(cx, id, opts.interest);
    button_drawn(ink, cx, area, label, &resp, opts);
    resp
}

/// **[`button`]'s drawing half, with the [`Response`] supplied rather than declared.** Returns the
/// face it drew.
///
/// The same door [`chip_drawn`] is, for the same reason and with the same warning: the pointer
/// cannot be driven from this crate — `Mouse` is `reachable_as: None` — so a hovered or pressed
/// button has no gesture to play, and a container that already holds an id has no other way in.
pub fn button_drawn<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Cells,
    label: &str,
    resp: &Response,
    opts: &ButtonOpts,
) -> Role {
    // **One construction, reached rather than copied.** See [`ButtonOpts`]: writing the four lines
    // out again here is how one of the two comes to fill its face and the other does not.
    chip_drawn(
        ink,
        cx,
        area,
        label,
        resp,
        &ChipOpts {
            faces: opts.faces,
            justify: opts.justify,
            interest: opts.interest,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::counters::Tally;
    use vitui_runtime::ctx::Driver;

    /// A tally over one frame of `f`, on a `w` by `h` sink.
    fn tallied(w: u16, h: u16, f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>)) -> Tally {
        let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| f(&mut tally, cx));
        tally
    }

    /// **Criterion 2: `button` writes each cell of its rectangle exactly once, and writes every
    /// one.**
    ///
    /// §2's two equalities on the component. Swept over the widths where the arithmetic runs out: a
    /// rectangle narrower than its label, one exactly as wide, one a single cell, and one three rows
    /// tall — where the label takes the middle row and the two either side are face.
    #[test]
    fn button_writes_a_partition_of_its_whole_rectangle() {
        for (w, h) in [(1, 1), (5, 1), (10, 1), (10, 3), (10, 4), (24, 2)] {
            for label in ["", "reset", "an action with a very long name"] {
                let opts = ButtonOpts::default();
                let tally = tallied(w.max(2), h, |tally, cx| {
                    button_into(tally, cx, Cells::at(0, 0, w, h), label, &opts);
                });
                assert_eq!(
                    tally.writes(),
                    tally.distinct(),
                    "{w}x{h} `{label}`: a button cell written twice"
                );
                assert_eq!(
                    tally.distinct(),
                    u64::from(w) * u64::from(h),
                    "{w}x{h} `{label}`: the button does not cover its own face"
                );
                assert_eq!(
                    tally.asked(),
                    tally.reported(),
                    "{w}x{h} `{label}`: the label left the button's rectangle"
                );
            }
        }
    }

    /// **Criterion 1: a button is one region and one tab stop, and it answers with a `Response`.**
    ///
    /// The `FOCUS` bit is what separates it from a panel on [`crate::dense`]'s screen — 338 regions
    /// and 333 stops — and it is a **bit on a declaration** rather than a second call, which is
    /// `Interest::FOCUS`'s own note: *costs no tracking*.
    #[test]
    fn a_button_is_one_region_and_one_tab_stop() {
        let mut driver = Driver::headless(20, 1).expect("a sink attaches");
        driver.frame(|cx| {
            let resp = button(cx, Cells::at(0, 0, 10, 1), "reset");
            assert_eq!((resp.rect.w, resp.rect.h), (10, 1));
            assert!(!resp.clicked && !resp.focused);
        });
        assert_eq!(driver.inspect().hits().len(), 1);
        assert_eq!(driver.inspect().stop_count(), 1);
        assert_eq!(
            driver.inspect().ring().len(),
            1,
            "and the focus ring holds it, built during the draw"
        );
    }

    /// **Two buttons at two call sites are two widgets**, which is what `#[track_caller]` on the
    /// component buys and what its absence would silently cost.
    ///
    /// `Ctx::id` mints from `Location::caller()`, so a component without the attribute reports **its
    /// own** line for every call in the application: the second button merges into the first, gets
    /// an inert `Response`, no hit entry and no tab stop, and **the screen still renders pixel for
    /// pixel correctly**. That is ADR 0027's 110-of-338 defect, and this is the two-line version of
    /// it.
    #[test]
    fn two_buttons_at_two_call_sites_are_two_widgets() {
        let mut driver = Driver::headless(24, 2).expect("a sink attaches");
        driver.frame(|cx| {
            button(cx, Cells::at(0, 0, 10, 1), "reset");
            button(cx, Cells::at(0, 1, 10, 1), "pause");
        });
        assert_eq!(
            driver.inspect().hits().len(),
            2,
            "the two buttons collided on one id, so one of them is inert and the screen is right"
        );
        assert_eq!(driver.inspect().stop_count(), 2);

        // The same two from **one** call site are one widget unless the caller keys them, which is
        // the other half of the rule and what `crate::dense`'s loop does.
        let mut driver = Driver::headless(24, 2).expect("a sink attaches");
        driver.frame(|cx| {
            for row in 0..2u16 {
                button(cx, Cells::at(0, row, 10, 1), "reset");
            }
        });
        assert_eq!(
            driver.inspect().hits().len(),
            1,
            "one call site with no key is one id, which is the merge policy and not an error"
        );
        let mut driver = Driver::headless(24, 2).expect("a sink attaches");
        driver.frame(|cx| {
            for row in 0..2u16 {
                cx.with_key(u64::from(row), |cx| {
                    button(cx, Cells::at(0, row, 10, 1), "reset")
                });
            }
        });
        assert_eq!(driver.inspect().hits().len(), 2, "keyed, they are two");
    }
}
