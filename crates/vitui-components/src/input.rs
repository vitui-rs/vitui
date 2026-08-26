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
//! §1 writes it out: `pub fn button(cx: &mut Ctx, area: Rect, label: &str) -> Response`. When this
//! module was written **`Rect` could not be named from this package** — it is `vitui_engine::Rect`,
//! `reachable_as: None`, and C6 says the dependency table is `vitui-runtime` and nothing else — so
//! what ships takes [`Rect`], this crate's own rectangle in the `Ctx`'s coordinates, swept operator
//! for operator against `vitui_runtime::layout::rect`. See [`vitui_runtime::layout::rect`] for the four candidates
//! and why this was the one; the rule is obeyed with `Rect` in `Rect`'s place, not set aside.
//!
//! **Runtime architecture issue 22 has since made `Rect` nameable here** (`vitui_runtime::Rect`),
//! which removes the reason and not the code. Whether the signature goes back to §1's own spelling
//! is components architecture issue 17, and it is not decided in this file.

use vitui_runtime::keys::{Code, Edge};
use vitui_runtime::layout::text::{truncate, width};
use vitui_runtime::{Ctx, CursorShape, Interest, Response, Role};

use crate::edit::{Text, WrapKind};
use crate::ink::{Direct, Ink};
use crate::keys;
use crate::state::Faces;
use crate::text::{ChipOpts, Justify, chip_drawn};
use vitui_runtime::Rect;

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

/// **A label on a face that reacts, and a tab stop.** Spec §1's own signature, with `Rect` for
/// `Rect`.
///
/// ```
/// use vitui_runtime::Rect;
/// use vitui_components::input::button;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(20, 1).expect("a sink attaches");
/// driver.frame(|cx| {
///     let resp = button(cx, Rect::new(0, 0, 10, 1), "reset");
///     // Rule 4: a `Response` back, and nothing has happened to it on a frame with no input.
///     assert!(!resp.clicked);
/// });
/// // One tab stop, because `Interest::FOCUS` is in the default.
/// assert_eq!(driver.inspect().stop_count(), 1);
/// ```
#[track_caller]
pub fn button(cx: &mut Ctx<'_, '_>, area: Rect, label: &str) -> Response {
    button_with(cx, area, label, &ButtonOpts::default())
}

/// [`button`], with the options spelled out.
#[track_caller]
pub fn button_with(cx: &mut Ctx<'_, '_>, area: Rect, label: &str, opts: &ButtonOpts) -> Response {
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
    area: Rect,
    label: &str,
    opts: &ButtonOpts,
) -> Response {
    let id = cx.id();
    let resp = cx.interact(id, area, opts.interest);
    button_drawn(ink, cx, area, label, &resp, opts);
    resp
}

/// **[`button`]'s drawing half, with the [`Response`] supplied rather than declared.** Returns the
/// face it drew.
///
/// The same door [`chip_drawn`] is, and for the second of its two reasons: a container that already
/// holds an id has no other way in. The first reason — *the pointer cannot be driven from this
/// crate* — **lifted with runtime architecture issue 22**, which made `Mouse` and the types needed
/// to build one reachable.
pub fn button_drawn<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
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
                    button_into(tally, cx, Rect::new(0, 0, w, h), label, &opts);
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
            let resp = button(cx, Rect::new(0, 0, 10, 1), "reset");
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
            button(cx, Rect::new(0, 0, 10, 1), "reset");
            button(cx, Rect::new(0, 1, 10, 1), "pause");
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
            for row in 0..2i32 {
                button(cx, Rect::new(0, row, 10, 1), "reset");
            }
        });
        assert_eq!(
            driver.inspect().hits().len(),
            1,
            "one call site with no key is one id, which is the merge policy and not an error"
        );
        let mut driver = Driver::headless(24, 2).expect("a sink attaches");
        driver.frame(|cx| {
            for row in 0..2i32 {
                cx.with_key(row as u64, |cx| {
                    button(cx, Rect::new(0, row, 10, 1), "reset")
                });
            }
        });
        assert_eq!(driver.inspect().hits().len(), 2, "keyed, they are two");
    }
}

// ═════════════════════════════════════════════════════════════════════════════════════════════════
// `field` — spec §11, and `input` and `textarea` are one component
// ═════════════════════════════════════════════════════════════════════════════════════════════════

/// [`field`]'s options.
///
/// Spec §1's rule 3: a `Default` struct, never a required builder.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FieldOpts {
    /// The four faces. The body is drawn on [`Faces::rest`] and on [`Faces::focus`] where the
    /// widget holds the keyboard.
    pub faces: Faces,
    /// What the field declares. **One entry for the widget**, never one per visible cluster.
    pub interest: Interest,
    /// **The caret's shape, asked for rather than left to the terminal's default.**
    ///
    /// A bar in both spellings, and it is on the options because it is the caller's to change — a
    /// terminal's own default is a block on most and a bar on some, so a field that did not ask
    /// would look different on two machines showing the same program. `CursorShape` is the
    /// engine's type re-exported through the runtime; this crate names no engine path.
    pub shape: CursorShape,
    /// Whether the caret's row is brought into view when a gesture moved it.
    ///
    /// **Conditional, and `WhenAsked` is the default for `CONTEXT.md`'s reason** — a widget that
    /// scrolls to its caret every frame drags the viewport back the moment the user scrolls away
    /// from it. See [`crate::wheel`] for the same rule one component over, and
    /// [`defective::Reveal`] for the two arms it is measured against.
    pub reveal: defective::Reveal,
}

impl Default for FieldOpts {
    fn default() -> FieldOpts {
        FieldOpts {
            faces: Faces::default(),
            interest: Interest::CLICK
                .with(Interest::HOVER)
                .with(Interest::FOCUS)
                .with(Interest::SCROLL),
            shape: CursorShape::Bar,
            reveal: defective::Reveal::WhenAsked,
        }
    }
}

/// **The API §11 deletes, kept deleted by a pair.**
///
/// > **There is no such thing as "put the caret at byte N."**
///
/// A caret is placed by a gesture — a click's column, a cluster step, `Home`, or a pair some
/// earlier state stored — and each of those lands on a cluster boundary **by construction**. An
/// arbitrary offset does not, and everything downstream preserves the bad caret faithfully: the
/// column is computed from it, the row is looked up from it, the selection is measured from it, and
/// the screen looks entirely correct.
///
/// [`Caret`](crate::edit::Caret)'s fields are private and there is no `Text::set_caret`. What
/// exists instead is [`Text::set_pos`](crate::edit::Text::set_pos), which takes a `Caret` — a value
/// only a gesture makes — and is what undo restores. §11 prices the difference at **0.0007 µs
/// against 6 109** on a 1 MB line, and the reason is the same one: `set_pos` restores a column and
/// `set_caret` would have to segment a megabyte to find one.
///
/// # The twin, naming the protected items by path
///
/// A lone `compile_fail` also passes when the item it names has been *renamed*: `E0599` for *the
/// method you must not have was never built* and `E0599` for *the method you meant moved* are the
/// same diagnostic. So the shape that ships is pinned first.
///
/// ```
/// use vitui_components::edit::{Caret, Text};
/// let mut st = Text::input();
/// // The four gestures, and the pair some earlier state stored.
/// st.right(20, false);
/// st.home(20, false);
/// st.end(20, false);
/// st.click(20, 0, 3, false);
/// st.set_pos(Caret::HOME);
/// // The pair is read and never written.
/// let _: usize = st.caret().byte();
/// let _: u16 = st.caret().col();
/// ```
///
/// # The hostile half
///
/// **Protects:** [`Text::set_pos`](crate::edit::Text::set_pos) and
/// [`Caret`](crate::edit::Caret), written `vitui_components::edit::Text::set_pos` and
/// `vitui_components::edit::Caret` at the paths the twin uses, by way of the method that must not
/// stand beside them.
///
/// ```compile_fail,E0599
/// fn main() {
///     let mut st = vitui_components::edit::Text::input();
///     st.set_caret(7);
/// }
/// ```
///
/// # And the fields, which are the same deletion three keystrokes shorter
///
/// **Protects:** [`Caret::byte`](crate::edit::Caret::byte) and
/// [`Caret::col`](crate::edit::Caret::col), by way of the field access that must not stand beside
/// them.
///
/// ```compile_fail,E0616
/// fn main() {
///     let c = vitui_components::edit::Caret::HOME;
///     let _ = c.byte;
/// }
/// ```
#[cfg(doc)]
pub struct WhyThereIsNoWayToPutTheCaretAtByteN;

/// **Block selection is not built, and a pair keeps it unbuilt.**
///
/// It is a rectangle over **visual** rows, which neither the run list nor the anchored range
/// expresses: an anchored range is one contiguous span of *bytes*, and a column block is a set of
/// spans that share a column pair and share nothing else. Adding it means a second selection
/// representation beside the one every gesture, every edit and every undo entry is written in —
/// and that is the code editor's, not this library's.
///
/// # The twin, naming the protected item by path
///
/// ```
/// use vitui_components::edit::Text;
/// let mut st = Text::textarea();
/// st.select_all(20);
/// // One anchored range, in bytes, and there is exactly one.
/// let _: std::ops::Range<usize> = st.selection();
/// ```
///
/// # The hostile half
///
/// **Protects:** [`Text::selection`](crate::edit::Text::selection), written
/// `vitui_components::edit::Text::selection` at the path the twin uses, by way of the method that
/// must not stand beside it.
///
/// ```compile_fail,E0599
/// fn main() {
///     let mut st = vitui_components::edit::Text::textarea();
///     let _ = st.block_selection();
/// }
/// ```
#[cfg(doc)]
pub struct WhyBlockSelectionIsNotBuilt;

/// **A text field: one component, one flag.** Spec §11.
///
/// `input` and `textarea` are not two functions with two states — they are one machine under two
/// break rules, and the rule is [`WrapKind`] on the state rather than an argument here. Everything
/// else is shared: the caret pair, the anchored selection, the undo ring and the wrap index.
///
/// ```
/// use vitui_components::edit::{Text, WrapKind};
/// use vitui_components::input::field;
/// use vitui_runtime::Rect;
/// use vitui_runtime::ctx::Driver;
///
/// let mut driver = Driver::headless(20, 3).expect("a sink attaches");
///
/// // An input and a textarea are the same call over two states.
/// let mut one = Text::input();
/// let mut many = Text::textarea();
/// driver.frame(|cx| {
///     field(cx, Rect::new(0, 0, 20, 1), &mut one);
///     field(cx, Rect::new(0, 1, 20, 2), &mut many);
/// });
/// assert_eq!(one.kind(), WrapKind::Ruler);
/// assert_eq!(many.kind(), WrapKind::Words);
///
/// // Two widgets, two hit entries, two tab stops — and never one per visible cluster.
/// assert_eq!(driver.inspect().hits().len(), 2);
/// assert_eq!(driver.inspect().stop_count(), 2);
/// ```
#[track_caller]
pub fn field(cx: &mut Ctx<'_, '_>, area: Rect, st: &mut Text) -> Response {
    field_with(cx, area, st, &FieldOpts::default())
}

/// [`field`], with the options spelled out.
#[track_caller]
pub fn field_with(cx: &mut Ctx<'_, '_>, area: Rect, st: &mut Text, opts: &FieldOpts) -> Response {
    field_into(&mut Direct, cx, area, st, opts)
}

/// **[`field`], drawing through an [`Ink`] so a counter can see every cell.**
///
/// The entry point a gate takes; [`field`] is this with [`Direct`].
#[track_caller]
pub fn field_into<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut Text,
    opts: &FieldOpts,
) -> Response {
    draw_with(ink, cx, area, st, opts, defective::Regions::Widget)
}

/// **The one axis `field` can be false on that is not on [`FieldOpts`]**, threaded here so the
/// shipped build and the refused one are one function with one value between them.
#[track_caller]
fn draw_with<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    area: Rect,
    st: &mut Text,
    opts: &FieldOpts,
    regions: defective::Regions,
) -> Response {
    // **The id, taken outside every closure** (ADR 0027).
    let id = cx.id();
    if area.is_empty() {
        return Response::inert(id, area);
    }
    let w = area.w;
    let rows = area.h;

    // **One hit entry for the widget.** §11's *79 regions against 621*: the alternative is one
    // region per visible cluster, which is `Regions::PerCluster` and is what a field that wanted a
    // per-cluster hover would write.
    let mut resp = cx.interact(id, area, opts.interest);

    // **A click places the caret by a column, which is a boundary by construction.**
    // `Response::local` is the press's position inside the widget's own rectangle, so the row is a
    // row of the viewport and the column is a screen column — neither is a byte offset, and that is
    // the whole of §11's *there is no such thing as "put the caret at byte N"* on the pointer path.
    let mut asked = false;
    if resp.clicked {
        cx.focus(id);
        if let Some((lx, ly)) = resp.local {
            let row = st.offset() + usize::try_from(ly.max(0)).unwrap_or(0);
            st.click(w, row, u16::try_from(lx.max(0)).unwrap_or(u16::MAX), false);
            asked = true;
        }
    }
    // ── the keyboard, in one drain loop ──────────────────────────────────────────────────────────
    //
    // **One loop and not two.** `Ctx::decline` hands a key back *and ends the level's turn at the
    // queue* (`crate::collect`'s own finding, components 17), so a field that read some keys here
    // and the rest after would get nothing after the first refusal. Everything this widget owns is
    // read inside this loop, and the first key it does not own ends it.
    let mut typed = String::new();
    while let Some(k) = cx.next_key(id) {
        // A release is not a gesture and is dropped rather than declined — nobody wants one.
        if k.kind == Edge::Release {
            continue;
        }
        // **`keys::text` decides what this widget accepts, and a chord types nothing.** Spec §3's
        // helper is the whole rule; the three refusals it makes — a release, a chord, a control
        // character — are why an accelerator does not land in the buffer.
        typed.clear();
        if keys::text(&k, &mut typed) {
            st.insert(w, &typed);
            resp.changed = true;
            asked = true;
            continue;
        }
        let shift = k.mods.shift();
        let moved = match k.code {
            Code::Left => {
                st.left(w, shift);
                true
            }
            Code::Right => {
                st.right(w, shift);
                true
            }
            Code::Home => {
                st.home(w, shift);
                true
            }
            Code::End => {
                st.end(w, shift);
                true
            }
            Code::Up | Code::Down => {
                let up = k.code == Code::Up;
                st.step_row(w, up, shift);
                true
            }
            _ => false,
        };
        if moved {
            asked = true;
            continue;
        }
        let edited = match k.code {
            Code::Backspace => {
                st.backspace(w);
                true
            }
            Code::Delete => {
                st.delete(w);
                true
            }
            // **A line break is a cluster in a textarea and is somebody else's key in an input**,
            // which is the second place the one flag is visible in behaviour rather than in layout:
            // a form's `Enter` is its submit and a textarea's is a row.
            Code::Enter if st.kind() == WrapKind::Words => {
                st.insert(w, "\n");
                true
            }
            _ => false,
        };
        if edited {
            resp.changed = true;
            asked = true;
            continue;
        }
        // **The two chords this widget owns.** They are chords, so `keys::text` has already
        // declined them — reading them here rather than through a key map is what makes a field
        // work in an application that has declared none.
        if keys::is_chord(&k)
            && let Code::Char(c) = k.code
        {
            match c {
                'z' if k.mods.chord().contains(vitui_runtime::Mods::CTRL) => {
                    if st.undo() {
                        resp.changed = true;
                        asked = true;
                    }
                    continue;
                }
                'a' if k.mods.chord().contains(vitui_runtime::Mods::CTRL) => {
                    st.select_all(w);
                    asked = true;
                    continue;
                }
                _ => {}
            }
        }
        // **Not this widget's key.** `Ctx::decline` gives the level above its turn, and this loop
        // ends rather than spinning — the queue is ordered and shared, so a widget that declined
        // *k* and then took *k+1* would leave the level above seeing the two the wrong way round.
        cx.decline(k);
        break;
    }

    // **The notch is consumed, because the field declares `Interest::SCROLL` and owns its
    // offset.** A widget that declares the pointer and does nothing with it is worse than one that
    // declares nothing: it is the topmost region over its rectangle, so the wheel does nothing here
    // *and* an enclosing `scroll_area` never sees the notch either. Components ticket 20 built
    // `crate::wheel` for exactly this, and a scroll is **not** a reveal — it does not set `asked`,
    // or the reveal below would drag the window straight back to the caret.
    // **And there is no horizontal axis to be wrong on**, which is a fact about the component
    // rather than an omission: both break rules wrap to the rectangle's width, so a row never
    // overflows it and `resp.scrolled.0` has nothing to move. That is the other half of components
    // ticket 20's finding — *a body dead downward is alive sideways* — answered by construction
    // instead of by arithmetic, and it is why `Text::wheel` takes rows and not a pair.
    if resp.scrolled.1 != 0 {
        // **The sign is the runtime's**, and `crate::collect` is the one that establishes it:
        // `st.offset + resp.scrolled.1`. A field that negated it would scroll the wrong way on a
        // screen where every other scrollable widget scrolls the right one.
        st.wheel(w, rows, resp.scrolled.1);
        resp.changed = true;
    }

    // **The reveal is conditional, and the condition is that a gesture asked for it.** The
    // unconditional arm is `Reveal::EveryFrame`; deleting the call entirely is `Reveal::Never`,
    // which costs the keyboard rather than the pointer.
    match opts.reveal {
        defective::Reveal::WhenAsked if asked => {
            st.reveal(w, rows);
        }
        defective::Reveal::EveryFrame => {
            st.reveal(w, rows);
        }
        _ => {}
    }

    // ── the drawing ──────────────────────────────────────────────────────────────────────────────
    //
    // **The face is resolved before a cell is written and never restyled after** (ADR 0026), and
    // the hover award is declared through the ink so an instrument can see it.
    // **The face comes out of [`crate::state::press_into`] and the award with it**, which is that
    // helper's whole criterion: the face drawn and the face awarded are one expression evaluated
    // once. A component that resolved its own face and declared its own award would be the second
    // place in this crate where they are two statements, which is exactly what `state`'s own gate
    // reads this file to refuse.
    let face = crate::state::press_into(ink, cx, area, &resp, &opts.faces);
    let paint = cx.theme().paint(face);
    let selected = cx.theme().paint(Role::Selection);

    // **The index is built once, before the loop.** Inside it every read is shared — the row's
    // start, its length and its bytes — which is what keeps the drawing allocation-free: a loop
    // that asked `Text::index` per row would need `&mut`, and the row's text would have to be
    // copied out to be drawn.
    let _ = st.index(w);
    // **The refused region spelling, declared where it can be**: it walks the rows the index
    // names, so it cannot run before the index exists — which is the first frame of every field.
    if regions == defective::Regions::PerCluster {
        declare_per_cluster(cx, st, area, opts);
    }
    let offset = st.offset();
    let sel = st.selection();
    let caret = st.caret();
    let index = st.indexed().expect("the index was just built");
    let buf = st.text();
    // **The caret's row is the index's answer and never a containment test over the drawn rows.**
    //
    // The rows do **not** tile the buffer: `layout::text::wrap` trims each piece, so the whitespace
    // a greedy break consumed and any trailing whitespace belong to no row's *content*. A loop that
    // set the caret from the first row whose content contains its byte therefore drops it entirely
    // for an ordinary typing state — a single trailing space is enough — and `Frame::caret` stays
    // `None`, so the terminal cursor disappears. On a contiguous `Ruler` index it fails the other
    // way: a byte on a shared boundary satisfies the row *before* it first, and the caret jumps to
    // the start of the row it has just left. `Index::row_of` answers both, once.
    let caret_row = index.row_of(caret.byte());
    let mut caret_at = None;
    for r in 0..rows {
        let y = area.y + i32::from(r);
        let row = offset + usize::from(r);
        let start = index.row_start(row);
        let len = index.row_len(row);
        let text = index.row_text(buf, row);
        // A window taller than the content draws filler rows, and `Index::row_start` answers 0 out
        // of range — so a filler row would compare against row 0's selection. `Text::reveal` clamps
        // the offset, and this is the other half: a row past the last one owns nothing.
        let (start, len, text) = match row < index.rows() {
            true => (start, len, text),
            false => (usize::MAX, 0, ""),
        };
        // **A row is a partition of its width, never a prefix.** The row's content, then the pad.
        let drawn = truncate(text, w);
        // **One verb a row where nothing on it is selected**, and the pad is inside it. See
        // [`Ink::pad_to`]: written as a text and a run, `verbs` separates two builds by how full
        // their rows happen to be — and on the memo-key defect it separates them **in the direction
        // that approves the defect**, because a stale row is the whole line truncated and fills its
        // width exactly.
        if sel.is_empty() || start + len <= sel.start || start >= sel.end {
            let _ = ink.pad_to(cx, area.x, y, drawn, w, paint);
        } else {
            let cells = paint_selected_row(ink, cx, area.x, y, drawn, start, &sel, paint, selected);
            if cells < w {
                let _ = ink.run(cx, area.x + i32::from(cells), y, " ", w - cells, paint);
            }
        }
        // **The caret is not a cell.** It is recorded here and written once, after the rows, so
        // the last write of the frame is the caret's own and not a row's.
        if row == caret_row {
            caret_at = Some((area.x + i32::from(caret.col().min(w.saturating_sub(1))), y));
        }
    }

    // **The caret's shape is asked for rather than left to the terminal's default**, and it is
    // placed only where this widget holds the keyboard — `Ctx::caret_with` refuses it otherwise,
    // because a caret on a screen where nothing is focused says typing goes somewhere it does not.
    // **`cx.is_focused` and not `resp.focused`**, and the difference is one frame. A `Response`
    // carries the focus as it stood when the widget *declared*, so a field that has just been
    // clicked — or that an application seated on this very frame — would place no caret until the
    // frame after. `Ctx::caret_with` refuses one where nothing holds the keyboard anyway, so the
    // check here is about *which* widget holds it and nothing else.
    if cx.is_focused(id)
        && let Some((x, y)) = caret_at
    {
        cx.caret_with(x, y, opts.shape);
    }
    resp
}

/// One row, split at the selection's edges. Three runs at most, and each is one verb.
#[expect(
    clippy::too_many_arguments,
    reason = "the ink, the context, the row's origin and content, where it starts in the buffer, \
              the selection and the two paints. Folding them into a struct would invent a type \
              that exists only to satisfy a lint, and every one of them is read exactly once"
)]
fn paint_selected_row<I: Ink>(
    ink: &mut I,
    cx: &mut Ctx<'_, '_>,
    x: i32,
    y: i32,
    row: &str,
    start: usize,
    sel: &std::ops::Range<usize>,
    paint: vitui_runtime::Paint,
    selected: vitui_runtime::Paint,
) -> u16 {
    let lo = sel.start.saturating_sub(start).min(row.len());
    let hi = (sel.end.saturating_sub(start)).min(row.len());
    let (lo, hi) = (floor_boundary(row, lo), floor_boundary(row, hi));
    let mut at = x;
    for (piece, st) in [
        (&row[..lo], paint),
        (&row[lo..hi], selected),
        (&row[hi..], paint),
    ] {
        if piece.is_empty() {
            continue;
        }
        let _ = ink.text(cx, at, y, piece, st);
        at += i32::from(width(piece));
    }
    u16::try_from(at - x).unwrap_or(u16::MAX)
}

/// The largest char boundary at or before `at`. A selection edge is a cluster boundary of the
/// *buffer*; clipped to a row it can land past the row's end, and a slice needs it to be one here.
fn floor_boundary(s: &str, at: usize) -> usize {
    let mut at = at.min(s.len());
    while !s.is_char_boundary(at) {
        at -= 1;
    }
    at
}

/// **§11's refused region count, built so a gate can watch it.** One entry per visible cluster.
fn declare_per_cluster(cx: &mut Ctx<'_, '_>, st: &Text, area: Rect, opts: &FieldOpts) {
    let mut key = 0u64;
    for r in 0..area.h {
        let Some(index) = st.indexed() else { break };
        let row = st.offset() + usize::from(r);
        let text = index.row_text(st.text(), row);
        let mut col = 0u16;
        let mut at = 0usize;
        while let Some(cluster) = crate::edit::step(&text[at..]) {
            let cw = width(cluster);
            cx.with_key(key, |cx| {
                let id = cx.id();
                let _ = cx.interact(
                    id,
                    Rect::new(area.x + i32::from(col), area.y + i32::from(r), cw.max(1), 1),
                    opts.interest,
                );
            });
            key += 1;
            col = col.saturating_add(cw);
            at += cluster.len();
            if col >= area.w {
                break;
            }
        }
    }
}

/// **The two spellings of `field` that are refused, kept runnable.**
pub mod defective {
    /// **How many regions the widget declares.**
    ///
    /// §11: *a text widget declares 79 regions against 621 for one per visible cluster*. Both arms
    /// draw exactly the same cells — the per-cluster arm declares and paints nothing extra — which
    /// is why the count is the gate and the surface is not.
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub enum Regions {
        /// **The rule.** One entry for the widget.
        #[default]
        Widget,
        /// **The defect.** One entry per visible cluster.
        PerCluster,
    }

    /// Whether the caret's row is brought into view, and on what condition.
    ///
    /// Three arms and not two, for [`crate::wheel`]'s reason: a one-directional gate goes green the
    /// moment somebody deletes the call, so the arm where the call is *gone* has to be expressible.
    #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
    pub enum Reveal {
        /// **The rule.** Only where a gesture moved the caret.
        #[default]
        WhenAsked,
        /// **The defect.** Every frame, which drags the viewport back from any scroll.
        EveryFrame,
        /// **The other defect.** Never, which costs the keyboard: type past the bottom of the
        /// window and the caret is somewhere off screen.
        Never,
    }

    /// [`crate::input::field_into`] with the region spelling stated, which is the entry a gate
    /// takes.
    #[track_caller]
    pub fn field_regions<I: crate::ink::Ink>(
        ink: &mut I,
        cx: &mut vitui_runtime::Ctx<'_, '_>,
        area: vitui_runtime::Rect,
        st: &mut crate::edit::Text,
        opts: &super::FieldOpts,
        regions: Regions,
    ) -> vitui_runtime::Response {
        super::draw_with(ink, cx, area, st, opts, regions)
    }
}

#[cfg(test)]
mod field_tests {
    use super::defective::{Regions, Reveal, field_regions};
    use super::*;
    use crate::counters::{Counter, Counters, Tally};
    use crate::edit::{Ring, WrapKind, defective as bad};
    use crate::runner::Pen;
    use vitui_runtime::Density;
    use vitui_runtime::ctx::Driver;
    use vitui_runtime::keys::Chord;

    /// The document `field`'s frame is measured over: 625 lines of the same rota.
    fn document() -> String {
        crate::document::text()
    }

    /// One frame of `f` on a `w` by `h` sink, through a `Tally`.
    fn tallied(w: u16, h: u16, f: impl FnOnce(&mut Tally, &mut Ctx<'_, '_>)) -> (Tally, Driver) {
        let mut driver = Driver::headless(w, h).expect("a sink cannot fail to attach");
        let mut tally = Tally::new();
        driver.frame(|cx| f(&mut tally, cx));
        (tally, driver)
    }

    /// **One call site for a field drawn on many frames**, seating the focus where asked.
    ///
    /// `field` is `#[track_caller]`, so a test that wrote the call twice would draw **two**
    /// widgets: the second frame mints a different id, the id the first frame focused did not draw,
    /// and the vanish rule clears the focus — a caret that is simply absent, on a screen that looks
    /// right. ADR 0027's own defect, arriving inside the gate for it.
    fn one_field(
        driver: &mut Driver,
        area: Rect,
        st: &mut Text,
        opts: &FieldOpts,
        seat: Option<vitui_runtime::Id>,
    ) -> vitui_runtime::Id {
        let mut id = None;
        driver.frame(|cx| {
            if let Some(seat) = seat {
                cx.focus(seat);
            }
            id = Some(field_with(cx, area, st, opts).id);
        });
        id.expect("a field answers with its id")
    }

    /// **Criterion 10: one hit entry for the widget, and the per-cluster spelling is the negative
    /// case.**
    ///
    /// §11: *a text widget declares 79 regions against 621 for one per visible cluster*. Both arms
    /// draw **exactly the same cells** — the per-cluster arm declares and paints nothing extra —
    /// so the count is the gate and no reader of the surface can see the difference.
    #[test]
    fn a_field_is_one_region_and_the_per_cluster_spelling_is_hundreds() {
        let text = crate::clusters::corpus().text().repeat(4);
        let area = Rect::new(0, 0, 60, 8);
        let opts = FieldOpts::default();

        let mut one = Text::of(text.clone(), WrapKind::Words);
        let mut pen_one = Pen::new(60, 8);
        let mut driver = Driver::headless(60, 8).expect("a sink attaches");
        driver.frame(|cx| {
            field_regions(&mut pen_one, cx, area, &mut one, &opts, Regions::Widget);
        });
        let widget = driver.inspect().hits().len();

        let mut many = Text::of(text, WrapKind::Words);
        let mut pen_many = Pen::new(60, 8);
        let mut driver = Driver::headless(60, 8).expect("a sink attaches");
        driver.frame(|cx| {
            field_regions(
                &mut pen_many,
                cx,
                area,
                &mut many,
                &opts,
                Regions::PerCluster,
            );
        });
        let per_cluster = driver.inspect().hits().len();

        assert_eq!(widget, 1, "a text widget is one region");
        assert!(
            per_cluster > 100,
            "the refused spelling declares {per_cluster}, which is not hundreds"
        );
        // **And the surface cannot tell.** The two arms wrote the same cells in the same order.
        let diff = pen_one.into_canvas().diff(&pen_many.into_canvas());
        assert_eq!(diff.cells, 0, "the two arms drew different screens");
    }

    /// **Criterion 13: `keys::text` is what decides what this widget accepts, and a chord types
    /// nothing.**
    ///
    /// The three refusals it makes are the whole rule: a release, a chord, a control character.
    /// What is watched here is the one an application notices — `Ctrl+S` saving nothing and typing
    /// an `s` — and the capital beside it, which is the over-correction that declines both.
    #[test]
    fn a_chord_types_nothing_and_a_capital_types_a_capital() {
        let mut st = Text::input();
        let mut driver = Driver::headless(20, 1).expect("a sink attaches");
        let area = Rect::new(0, 0, 20, 1);
        let opts = FieldOpts::default();
        let id = one_field(&mut driver, area, &mut st, &opts, None);
        one_field(&mut driver, area, &mut st, &opts, Some(id));
        for chord in [
            Chord::key('h'),
            Chord::key('i'),
            Chord::key('s').ctrl(),
            Chord::key('c').alt(),
        ] {
            driver.post_key(crate::keys::press(chord));
            one_field(&mut driver, area, &mut st, &opts, Some(id));
        }
        assert_eq!(
            st.text(),
            "hi",
            "an accelerator's letter landed in the buffer"
        );

        // **The defective reading, watched putting it there.** `keys::defective::on_code_alone` is
        // the code the runtime found in a shipped field six tickets old, and the whole difference
        // is the two refusals `keys::text` makes.
        let mut typed = String::new();
        let ctrl_s = crate::keys::press(Chord::key('s').ctrl());
        assert!(!crate::keys::text(&ctrl_s, &mut typed));
        assert_eq!(typed, "");
        assert!(crate::keys::defective::on_code_alone(&ctrl_s, &mut typed));
        assert_eq!(typed, "s", "the accelerator typed");
    }

    /// **Criterion 14: the caret's shape is asked for, and it is the one that reaches the sink.**
    ///
    /// `Screen::set_cursor` applies all three of `{ x, y, shape }`, so a widget that placed a caret
    /// and left the shape alone would render as a block on most terminals and a bar on some — the
    /// same program looking different on two machines. Both spellings ask for a bar.
    #[test]
    fn both_spellings_ask_for_the_shape_that_reaches_the_sink() {
        for kind in [WrapKind::Ruler, WrapKind::Words] {
            let mut st = Text::of("hello".into(), kind);
            let mut driver = Driver::headless(20, 3).expect("a sink attaches");
            let area = Rect::new(0, 0, 20, if kind == WrapKind::Ruler { 1 } else { 3 });
            let opts = FieldOpts::default();
            let id = one_field(&mut driver, area, &mut st, &opts, None);
            // Nothing holds the keyboard on the first frame, and `Ctx::caret_with` refuses a caret
            // then — the runtime's own refusal, asserted rather than worked around.
            assert!(driver.inspect().caret().is_none(), "{kind:?}: unfocused");

            one_field(&mut driver, area, &mut st, &opts, Some(id));
            let caret = driver.inspect().caret().expect("focused, so a caret");
            assert_eq!(caret.shape, CursorShape::Bar, "{kind:?}");

            // **The shape is the widget's and not the terminal's**, which is what the option buys:
            // the same field asking for a block gets one.
            let block = FieldOpts {
                shape: CursorShape::Block,
                ..FieldOpts::default()
            };
            one_field(&mut driver, area, &mut st, &block, Some(id));
            assert_eq!(
                driver.inspect().caret().expect("focused").shape,
                CursorShape::Block,
                "{kind:?}"
            );
        }
    }

    /// **A field writes a partition of its whole rectangle**, at every size the arithmetic runs out
    /// at. §2's two equalities on the component.
    #[test]
    fn a_field_writes_each_cell_of_its_rectangle_exactly_once() {
        for (w, h) in [(1, 1), (5, 1), (20, 1), (20, 4), (7, 9), (60, 8)] {
            for content in ["", "hello", "the quick brown fox jumps over the lazy dog"] {
                for kind in [WrapKind::Ruler, WrapKind::Words] {
                    let mut st = Text::of(content.into(), kind);
                    let (tally, _) = tallied(w, h, |tally, cx| {
                        field_into(
                            tally,
                            cx,
                            Rect::new(0, 0, w, h),
                            &mut st,
                            &FieldOpts::default(),
                        );
                    });
                    assert_eq!(
                        tally.writes(),
                        tally.distinct(),
                        "{w}x{h} {kind:?} `{content}`: a cell written twice"
                    );
                    assert_eq!(
                        tally.distinct(),
                        u64::from(w) * u64::from(h),
                        "{w}x{h} {kind:?} `{content}`: the field does not cover its own rectangle"
                    );
                    assert_eq!(
                        tally.asked(),
                        tally.reported(),
                        "{w}x{h} {kind:?} `{content}`: a write left the rectangle"
                    );
                }
            }
        }
    }

    /// **A row with a selection on it is still a partition of its width, and still one cell
    /// each.**
    ///
    /// The selected path is the only place `field` writes a row as more than one verb — three at
    /// most, split at the selection's two edges — so it is the only place the §2 equalities can
    /// come apart, and nothing else in this file reaches it. Swept over every selection a caret can
    /// make from every position, at widths where the arithmetic runs out.
    #[test]
    fn a_selected_row_is_still_a_partition_of_its_width() {
        for (w, h) in [(1, 1), (6, 1), (20, 3), (13, 5)] {
            for content in [
                "hello world",
                "the quick brown fox jumps over the lazy dog",
                crate::clusters::corpus().text(),
            ] {
                for kind in [WrapKind::Ruler, WrapKind::Words] {
                    let mut base = Text::of(content.into(), kind);
                    let steps = crate::edit::boundaries(base.text()).len();
                    let _ = base.index(w);
                    for from in 0..steps {
                        let mut st = base.clone();
                        // The anchor is placed by a gesture and the caret is dragged off it, which
                        // is the only way an anchored range is ever made.
                        for _ in 0..from {
                            st.right(w, false);
                        }
                        for _ in 0..(steps - from) {
                            st.right(w, true);
                        }
                        let (tally, _) = tallied(w, h, |tally, cx| {
                            field_into(
                                tally,
                                cx,
                                Rect::new(0, 0, w, h),
                                &mut st,
                                &FieldOpts::default(),
                            );
                        });
                        let what = format!("{w}x{h} {kind:?} anchor at {from}");
                        assert_eq!(
                            tally.writes(),
                            tally.distinct(),
                            "{what}: a cell written twice"
                        );
                        assert_eq!(
                            tally.distinct(),
                            u64::from(w) * u64::from(h),
                            "{what}: the field does not cover its own rectangle"
                        );
                        assert_eq!(
                            tally.asked(),
                            tally.reported(),
                            "{what}: a write left the rectangle"
                        );
                    }
                }
            }
        }
    }

    /// **The caret survives a trailing space and a soft-wrap boundary**, which is where a
    /// containment test over the drawn rows loses it.
    ///
    /// The rows do not tile the buffer — `layout::text::wrap` trims each piece — so a loop that set
    /// the caret from the first row whose *content* contains its byte drops it for an ordinary
    /// typing state: type one space at the end of a line and the terminal cursor disappears, on a
    /// screen that is otherwise correct. On a contiguous `Ruler` index it fails the other way, and
    /// the caret jumps to the start of the row it has just left. Both are watched here.
    #[test]
    fn the_caret_survives_a_trailing_space_and_a_soft_wrap_boundary() {
        let cases: &[(&str, WrapKind, u16, u16)] = &[
            ("hi ", WrapKind::Words, 20, 3),
            ("hi   ", WrapKind::Words, 20, 3),
            ("hi there and more words here", WrapKind::Words, 8, 6),
            ("abcdefghij", WrapKind::Ruler, 5, 4),
            ("a line\nand another \n", WrapKind::Words, 12, 5),
        ];
        for (content, kind, w, h) in cases.iter().copied() {
            let mut st = Text::of(content.into(), kind);
            let steps = crate::edit::boundaries(content).len();
            let area = Rect::new(0, 0, w, h);
            let opts = FieldOpts::default();
            for step in 0..steps {
                let mut st = {
                    st.set_pos(crate::edit::Caret::HOME);
                    let mut copy = st.clone();
                    for _ in 0..step {
                        copy.right(w, false);
                    }
                    copy
                };
                let mut driver = Driver::headless(w, h).expect("a sink attaches");
                let id = one_field(&mut driver, area, &mut st, &opts, None);
                one_field(&mut driver, area, &mut st, &opts, Some(id));
                let caret = driver.inspect().caret().unwrap_or_else(|| {
                    panic!(
                        "`{content}` {kind:?} at {w}x{h}: no caret at all after {step} steps, \
                         byte {}",
                        st.caret().byte()
                    )
                });
                // **And it is on the row the index says**, which is the half a `Ruler` gets wrong:
                // a byte on a shared boundary satisfies the row before it first.
                let at = st.caret().byte();
                let row = st.index(w).row_of(at);
                assert_eq!(
                    usize::from(caret.y),
                    row - st.offset(),
                    "`{content}` {kind:?} at {w}x{h}: the caret is drawn on the wrong row after \
                     {step} steps"
                );
            }
        }
    }

    /// **A wheel notch over a field moves its window, and the field consumes it.**
    ///
    /// A field declares `Interest::SCROLL` and owns its offset (§17), so a notch it did not consume
    /// is worse than one it never declared: it is the topmost region over its rectangle, so an
    /// enclosing `scroll_area` would never see the notch either. Components ticket 20's defect
    /// class, played through a posted notch over the previous frame's hit index rather than through
    /// a delta added to the offset.
    #[test]
    fn a_notch_over_a_field_moves_its_window() {
        use vitui_runtime::{Buttons, Mods, Mouse, MouseKind, Notch};

        let (w, h) = (60u16, 10u16);
        let area = Rect::new(0, 0, w, h);
        let opts = FieldOpts::default();
        let mut st = Text::of(crate::document::text(), WrapKind::Words);
        let _ = st.index(w);
        let mut driver = Driver::headless(w, h).expect("a sink attaches");
        // **A real notch, posted and routed through the previous frame's hit index** — components
        // ticket 20's arrangement and not a delta added to the offset, which has no second axis to
        // be wrong on. The pointer needs a position before anything is `over` it, and the index it
        // is resolved against is the previous frame's, so one frame with no click opens the run.
        let mouse = |kind: MouseKind| Mouse {
            x: w / 2,
            y: h / 2,
            kind,
            buttons: Buttons::NONE,
            mods: Mods::NONE,
            at: std::time::Instant::now(),
        };
        driver.post_mouse(mouse(MouseKind::Move));
        one_field(&mut driver, area, &mut st, &opts, None);
        assert_eq!(st.offset(), 0);

        for _ in 0..4 {
            driver.post_mouse(mouse(MouseKind::Wheel(Notch::Down)));
            one_field(&mut driver, area, &mut st, &opts, None);
        }
        assert!(
            st.offset() > 0,
            "four notches over a field left its window at row 0, so the field declared the pointer \
             and did nothing with it — and being the topmost region over its rectangle, nothing \
             enclosing it saw the notch either"
        );
        let down = st.offset();

        for _ in 0..8 {
            driver.post_mouse(mouse(MouseKind::Wheel(Notch::Up)));
            one_field(&mut driver, area, &mut st, &opts, None);
        }
        assert_eq!(st.offset(), 0, "notches up past the top stay at the top");
        assert!(down > 0);

        // **The sign is the runtime's and not this component's opinion.** `crate::collect`
        // establishes it as `offset + resp.scrolled.1`, and a field that negated it would scroll
        // the wrong way on a screen where every other scrollable widget scrolls the right one. This
        // gate is watched catching that: writing `-resp.scrolled.1` leaves the offset at 0 above.
        //
        // **And there is no horizontal axis here**, which is a fact about the component: both break
        // rules wrap to the rectangle's width, so a row never overflows it and there is nothing for
        // a tilt notch to move.
        driver.post_mouse(mouse(MouseKind::Wheel(Notch::Right)));
        one_field(&mut driver, area, &mut st, &opts, None);
        assert_eq!(
            st.offset(),
            0,
            "a tilt notch has nothing to move on a wrapped field"
        );
    }

    /// **Criterion 6: the frame does not pay for the ring.**
    ///
    /// §11's *at cap (256 entries, 25 600 B) 160.67 µs / 0 allocs, emptied 160.71 µs / 0 allocs,
    /// same document, same screen*. The timing is the report; what is gated is that **every one of
    /// §20's readable counters is the same number** on the two arms, which is the sentence *the
    /// frame does not pay* without a clock in it.
    #[test]
    fn the_ring_at_cap_and_the_ring_emptied_are_the_same_frame() {
        let (w, h) = (crate::document::WIDE, 20u16);
        let area = Rect::new(0, 0, w, h);

        let mut full = Text::of(document(), WrapKind::Words)
            .with_ring(Ring::bounded(256, 25_600).uncoalesced());
        full.end(w, false);
        for i in 0..320usize {
            full.insert(w, &(i % 10).to_string());
        }
        assert_eq!(full.ring().len(), 256);
        assert!(full.ring().bytes() <= 25_600);
        let mut empty = Text::of(full.text().to_string(), WrapKind::Words);
        assert!(empty.ring().is_empty());

        let play = |st: &mut Text| {
            let mut driver = crate::runner::driver_at(w, h, Density::default());
            let mut pen = Pen::new(w, h);
            driver.frame(|cx| {
                field_into(&mut pen, cx, area, st, &FieldOpts::default());
            });
            let counters = Counters::of(
                &driver,
                pen.tally(),
                crate::counters::Allocations::over(1, 0),
            );
            (counters, pen.into_canvas())
        };
        let (a, one) = play(&mut full);
        let (b, two) = play(&mut empty);
        let differs: Vec<Counter> = Counter::ALL
            .into_iter()
            .filter(|c| {
                let (l, r) = (a.get(*c).measured(), b.get(*c).measured());
                l.is_some() != r.is_some() || (l.is_some() && l != r)
            })
            .collect();
        assert_eq!(differs, Vec::new(), "the frame paid for the ring");
        assert_eq!(one.diff(&two).cells, 0, "and it drew a different screen");
    }

    /// **Criterion 11: the frame is flat at 100 kB and at 1 MB.**
    ///
    /// §11 prices it at *82.4 µs / 0 marked / 19 634 writes / 631 verbs / 79 regions / 0 merges /
    /// 0 allocations, flat at 100 kB and 1 MB*. The magnitudes are a screen this ticket does not
    /// own — `examples/field_numbers.rs` prints this one's — and **flat** is the claim that is a
    /// gate: ten times the document is the same frame, because a field costs its visible window.
    #[test]
    fn a_hundred_kilobytes_and_a_megabyte_are_the_same_frame() {
        let (w, h) = (crate::document::WIDE, crate::document::H);
        let area = Rect::new(0, 0, w, h);
        let mega = crate::document::pasted();
        let hundred = mega[..100_000].to_string();

        let play = |content: String| {
            let mut st = Text::of(content, WrapKind::Words);
            // The index is the wrapping's and a `textarea` has it already; building it inside the
            // measured frame would price the paste rather than the frame.
            let _ = st.index(w);
            let mut driver = crate::runner::driver_at(w, h, Density::default());
            let mut tally = Tally::new();
            driver.frame(|cx| {
                field_into(&mut tally, cx, area, &mut st, &FieldOpts::default());
            });
            let counters = Counters::of(&driver, &tally, crate::counters::Allocations::over(1, 0));
            (
                tally.writes(),
                tally.verbs(),
                counters.get(Counter::Regions).measured(),
                counters.get(Counter::Merges).measured(),
            )
        };
        assert_eq!(
            play(hundred),
            play(mega),
            "the frame moved with the document"
        );
    }

    /// **A field costs its visible window and never its content**, which is the invariant one layer
    /// down stated on this component: the verbs are the rectangle's rows whatever the buffer holds.
    #[test]
    fn the_verbs_are_the_window_and_not_the_buffer() {
        let (w, h) = (60u16, 8u16);
        let seen = |content: String| {
            let mut st = Text::of(content, WrapKind::Words);
            let (tally, _) = tallied(w, h, |tally, cx| {
                field_into(
                    tally,
                    cx,
                    Rect::new(0, 0, w, h),
                    &mut st,
                    &FieldOpts::default(),
                );
            });
            // The cells are the rectangle either way, which is the equality that does hold.
            assert_eq!(tally.writes(), u64::from(w) * u64::from(h));
            tally.verbs()
        };
        let small = seen("hello\nworld\n".into());
        let large = seen(crate::document::text());
        // **A bound and not an equality**, and the reason is a counter that separates two correct
        // frames: a row that exactly fills its width costs one verb and a row that does not costs
        // two, so twelve bytes of content and six hundred kilobytes of it differ by the *pads*
        // rather than by the buffer. Two verbs a row is the window's own bound, whatever it holds.
        assert!(small <= u64::from(h) * 2, "{small} verbs over {h} rows");
        assert!(large <= u64::from(h) * 2, "{large} verbs over {h} rows");
    }

    /// **Criterion 15: the three reveal arms are three different programs.**
    ///
    /// `WhenAsked` moves the window only where a gesture moved the caret; `EveryFrame` drags it
    /// back from any scroll; `Never` loses the caret off the bottom of the window. Three arms and
    /// not two, for [`crate::wheel`]'s reason — a one-directional gate goes green the moment
    /// somebody deletes the call.
    #[test]
    fn the_three_reveal_arms_are_three_different_programs() {
        let (w, h) = (40u16, 4u16);
        let area = Rect::new(0, 0, w, h);
        let seen = |reveal: Reveal, scroll_to: usize| {
            let mut st = Text::of(crate::document::text(), WrapKind::Words);
            let _ = st.index(w);
            // A caret at the top, and a window the reader has scrolled a long way down.
            st.click(w, 0, 0, false);
            st.scroll_to(scroll_to);
            let opts = FieldOpts {
                reveal,
                ..FieldOpts::default()
            };
            let mut driver = Driver::headless(w, h).expect("a sink attaches");
            let mut tally = Tally::new();
            driver.frame(|cx| {
                field_into(&mut tally, cx, area, &mut st, &opts);
            });
            st.offset()
        };
        assert_eq!(
            seen(Reveal::WhenAsked, 400),
            400,
            "a frame with no gesture asked"
        );
        assert_eq!(
            seen(Reveal::EveryFrame, 400),
            0,
            "dragged back to the caret"
        );
        assert_eq!(seen(Reveal::Never, 400), 400);

        // **And the arm the three differ on: a gesture that asks.** `WhenAsked` and `EveryFrame`
        // agree here and `Never` does not, which is the half that costs the keyboard — type past
        // the bottom of the window and the caret is somewhere off screen.
        let after_a_key = |reveal: Reveal| {
            let mut st = Text::of(crate::document::text(), WrapKind::Words);
            let _ = st.index(w);
            st.click(w, 400, 0, false);
            st.scroll_to(0);
            let opts = FieldOpts {
                reveal,
                ..FieldOpts::default()
            };
            let mut driver = Driver::headless(w, h).expect("a sink attaches");
            let id = one_field(&mut driver, area, &mut st, &opts, None);
            // The focus is seated a frame before the key it enables, which is this runtime's own
            // cadence and not this gate's arrangement.
            one_field(&mut driver, area, &mut st, &opts, Some(id));
            driver.post_key(crate::keys::press(Chord::new(Code::Right)));
            one_field(&mut driver, area, &mut st, &opts, Some(id));
            st.offset()
        };
        let asked = after_a_key(Reveal::WhenAsked);
        assert!(
            asked > 0 && asked <= 400,
            "a gesture asked and the window did not follow the caret: offset {asked}"
        );
        assert_eq!(after_a_key(Reveal::EveryFrame), asked, "the two agree here");
        assert_eq!(
            after_a_key(Reveal::Never),
            0,
            "the caret is off the window and stays there, which is what deleting the call costs"
        );
    }

    /// **`field` is `#[track_caller]`, so two fields at two call sites are two widgets.**
    ///
    /// ADR 0027's 110-of-338 defect, at an amplitude of two: without the attribute the second field
    /// merges into the first, gets an inert `Response`, no hit entry and no tab stop — and **the
    /// screen still renders correctly**.
    #[test]
    fn two_fields_at_two_call_sites_are_two_widgets() {
        let mut a = Text::input();
        let mut b = Text::input();
        let mut driver = Driver::headless(20, 2).expect("a sink attaches");
        driver.frame(|cx| {
            field(cx, Rect::new(0, 0, 20, 1), &mut a);
            field(cx, Rect::new(0, 1, 20, 1), &mut b);
        });
        assert_eq!(driver.inspect().hits().len(), 2);
        assert_eq!(driver.inspect().stop_count(), 2);
        assert_eq!(driver.inspect().ids().merges(), 0);
    }

    /// **An empty rectangle declares nothing at all.**
    ///
    /// `Ctx::interact` appends to the hit index and to the focus ring *before* either looks at the
    /// rectangle, so a field a stack has run out of room for would otherwise be an entry and a tab
    /// stop nobody can reach — [`crate::disclose`]'s `Closed::ZeroRect` at an amplitude of one.
    #[test]
    fn a_field_with_no_room_declares_nothing() {
        let mut st = Text::input();
        let mut driver = Driver::headless(20, 1).expect("a sink attaches");
        driver.frame(|cx| {
            let resp = field(cx, Rect::new(0, 0, 20, 0), &mut st);
            assert!(!resp.focused && !resp.clicked);
        });
        assert_eq!(driver.inspect().hits().len(), 0);
        assert_eq!(driver.inspect().stop_count(), 0);
        assert!(driver.inspect().caret().is_none());
    }

    /// **The run list and the anchored range select the same bytes**, which is what makes the
    /// refusal a cost argument rather than a correctness one.
    #[test]
    fn the_two_selection_spellings_agree_and_only_one_of_them_walks() {
        let text = crate::document::text();
        let clusters = crate::edit::boundaries(&text).len() - 1;
        let mut st = Text::of(text.clone(), WrapKind::Words);
        st.select_all(crate::document::WIDE);
        let range = st.selection();

        let mut runs = bad::RunList::of(0, clusters);
        let by_runs = runs.bytes(&text);
        assert_eq!(range, by_runs, "the two spellings disagree about the bytes");
        assert!(runs.walked() > 0, "the run list did not have to walk");
    }
}
